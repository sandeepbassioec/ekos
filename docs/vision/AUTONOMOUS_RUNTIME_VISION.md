# Autonomous Runtime Vision
## Requirements Volume — VectorVolt + EKOS

**Status:** Architecture Capture (not scheduled for development)  
**Captured:** 2026-07-07  
**Scope:** VectorVolt storage engine + EKOS knowledge runtime  

---

## 1. Vision Statement

VectorVolt and EKOS together form a self-contained, self-aware runtime.  
The system detects its own failures, classifies them, decides how to respond,  
and applies the response — autonomously or with human approval — without  
requiring external tooling, external source repositories, or manual intervention.

This document captures the full vision so that every architectural decision  
made during Phase 2–5 moves in this direction. Development happens in phases;  
the direction does not change.

---

## 2. Observability Foundation (The Prerequisite for Everything Else)

Before the system can heal itself, it must know *what kind of thing broke*.  
This is the most important piece — all self-healing logic depends on correct classification.

### 2.1 Error Taxonomy

Every error, anomaly, or degradation is classified into exactly one of four domains:

```
┌─────────────────────────────────────────────────────────────────┐
│                     ERROR TAXONOMY                              │
├──────────────────┬──────────────────────────────────────────────┤
│ BUSINESS ERROR   │ The data or logic is wrong, not the system.  │
│                  │ Examples:                                     │
│                  │  - KO payload fails schema validation         │
│                  │  - ko_put rejected: version conflict          │
│                  │  - CCR query returns semantically wrong KOs   │
│                  │  - Workflow produces incorrect output         │
│                  │ Response: alert owner, do NOT self-patch      │
├──────────────────┼──────────────────────────────────────────────┤
│ SYSTEM ERROR     │ The runtime code has a bug or degradation.   │
│                  │ Examples:                                     │
│                  │  - ko_history() p99 latency > 10x baseline   │
│                  │  - compaction produces corrupt SSTable        │
│                  │  - memory leak in KoStore scan path           │
│                  │  - WAL replay fails on restart                │
│                  │ Response: candidate for self-patch via ARP    │
├──────────────────┼──────────────────────────────────────────────┤
│ INFRA ERROR      │ The compute/storage layer is degraded.       │
│                  │ Examples:                                     │
│                  │  - disk I/O latency spike (not our code)     │
│                  │  - OOM from OS-level memory pressure          │
│                  │  - CPU throttling from cloud provider         │
│                  │  - filesystem permissions changed externally  │
│                  │ Response: self-heal at infra level (restart,  │
│                  │           migrate, scale) — do NOT patch code │
├──────────────────┼──────────────────────────────────────────────┤
│ NETWORK ERROR    │ Communication between components is broken.  │
│                  │ Examples:                                     │
│                  │  - ekosd gRPC client timeout                  │
│                  │  - DNS resolution failure                     │
│                  │  - TLS cert expired                           │
│                  │  - partial packet loss to storage backend     │
│                  │ Response: retry with backoff, circuit break,  │
│                  │           alert — do NOT patch code           │
└──────────────────┴──────────────────────────────────────────────┘
```

### 2.2 Classification Pipeline

```
Anomaly detected
      │
      ▼
ObservabilityAgent
  reads: metrics, traces, logs, kernel state
      │
      ├── Is it reproducible with a synthetic probe?
      │     NO  → transient, log and continue
      │     YES → classify
      │
      ├── Is the error at data/logic boundary?  → BUSINESS ERROR
      ├── Is baseline metric degraded in our code path? → SYSTEM ERROR
      ├── Is the OS/disk/memory showing independent degradation? → INFRA ERROR
      └── Is it at a network/RPC boundary? → NETWORK ERROR
      │
      ▼
RemediationRouter
  routes to appropriate response handler
```

### 2.3 Observability Data Required

Each observable point in the system emits structured telemetry:

```rust
pub struct ObservabilityEvent {
    pub timestamp:    DateTime<Utc>,
    pub source:       EventSource,       // which syscall / component
    pub domain:       ErrorDomain,       // Business | System | Infra | Network
    pub severity:     Severity,          // Info | Warning | Degraded | Critical
    pub metric:       MetricSnapshot,    // latency, error_rate, throughput
    pub context:      serde_json::Value, // structured context for LLM diagnosis
}
```

Minimum instrumentation required (Phase 2 addition to ekos-kernel):
- Per-syscall latency histogram (p50, p95, p99)
- Per-syscall error rate (count + type)
- KvStore compaction duration and output size ratio
- WAL write throughput and segment count
- Memory usage per component

---

## 3. Capability Injection Protocol (CIP)

**Goal:** Add new behavior to the running system without restarting and without pre-planning the capability at design time.

### 3.1 Concept

An authorized admin describes a capability in plain language. The system synthesizes, compiles, sandboxes, and loads it as a live function. The capability is immediately addressable via gRPC.

### 3.2 Components

**A. Admin gRPC Channel (privileged)**
```protobuf
service EkosAdmin {
  rpc InjectCapability(CapabilityIntent) returns (CapabilityHandle);
  rpc ListCapabilities(Empty)            returns (stream CapabilityHandle);
  rpc RevokeCapability(CapabilityHandle) returns (RevokeResult);
}

message CapabilityIntent {
  string name        = 1;  // "stale_contract_scanner"
  string description = 2;  // plain English — what it should do
  string author      = 3;
  bytes  signature   = 4;  // admin auth token — prevents unauthorized injection
}
```

**B. Synthesis Engine (`ekos-cip` crate)**
1. Receives CapabilityIntent
2. Calls LLM (Claude API) with ekos-kernel plugin API schema as context
3. LLM returns Rust source implementing `KoPlugin` trait
4. Compiles to WASM (wasm32-wasi target) in isolated temp dir
5. Loads via wasmtime into sandbox

**C. Plugin API (stable, versioned)**
```rust
// ekos-plugin-api — what plugin authors implement
pub trait KoPlugin {
    fn on_put(&self, ko: &KnowledgeObject) -> PluginResult;
    fn on_scan_filter(&self, ko: &KnowledgeObject) -> bool;
    fn transform(&self, ko: KnowledgeObject) -> KnowledgeObject;
}
```

**D. Dynamic gRPC Method Registration (The Core Innovation)**

Standard gRPC: all methods are fixed at compile time in the `.proto` file.  
CIP breaks this constraint: new methods appear at runtime as capabilities are injected.

The mechanism uses a **capability router** inside ekosd — a single generic gRPC method that dispatches to any registered capability by name:

```protobuf
// Generic dispatch — the only compile-time method needed
rpc InvokeCapability(CapabilityRequest) returns (stream CapabilityResponse);

message CapabilityRequest {
  string capability_name = 1;   // "stale_contract_scanner"
  string namespace       = 2;
  bytes  params          = 3;   // capability-specific input (MessagePack)
}

message CapabilityResponse {
  bytes  payload = 1;           // capability-specific output (MessagePack)
  bool   is_last = 2;
}
```

Inside ekosd, the `CapabilityRegistry` maps names to loaded WASM instances:

```rust
pub struct CapabilityRegistry {
    // name → (wasm instance, input/output schema, metadata)
    capabilities: DashMap<String, Arc<LoadedCapability>>,
}

impl CapabilityRegistry {
    // Called by InjectCapability — new capability immediately addressable
    pub fn register(&self, name: String, cap: LoadedCapability) { .. }

    // Called by InvokeCapability gRPC handler
    pub fn invoke(&self, name: &str, params: &[u8]) -> impl Stream<Item=Vec<u8>> { .. }
}
```

**What this achieves:** From the caller's perspective, every injected capability feels like a first-class gRPC method. The TypeScript SDK discovers available capabilities at runtime via `ListCapabilities()` and can call any of them through `InvokeCapability`. No SDK recompilation, no proto changes, no server restart. The API surface grows dynamically.

### 3.3 Security Model
- Admin channel requires a signed token (separate from regular gRPC auth)
- WASM sandbox: no filesystem access, no network access, memory-capped
- All injected capabilities logged to audit KOs (cannot be deleted)

---

## 4. Autonomous Remediation Protocol (ARP)

**Goal:** System detects a SYSTEM ERROR, diagnoses root cause against its own source, synthesizes a patch, validates it, and applies it — autonomously or with human gate.

### 4.1 ARP Pipeline

```
SYSTEM ERROR detected (from ObservabilityAgent)
        │
        ▼
  Capture Phase  ← THIS IS THE FOUNDATION — metrics alone are not enough
  ├── Capture: exact input payload that triggered the anomaly (gRPC bytes)
  ├── Capture: KoStore state snapshot at time of failure
  ├── Capture: execution context (thread, syscall, stack trace)
  ├── Capture: last N inputs before failure (sliding window corpus)
  └── Store all as: KO { kind: "error_capture", ... }  (see Section 4.4)
        │
        ▼
  Diagnosis Phase
  ├── Collect: metric history, error captures, recent traces
  ├── Identify: which code path is implicated
  └── LLM prompt: source code + captured payload + symptom → root cause
        │
        ▼
  Synthesis Phase
  ├── LLM generates candidate patch (Rust or WASM plugin level)
  ├── If WASM level → compile to plugin, load in shadow
  └── If core level → compile new binary in shadow environment
        │
        ▼
  Validation Phase  ← patch must pass ALL THREE, not just metrics
  ├── (a) Original payload: replay exact captured input → must not error
  ├── (b) Generated variants: auto-fuzz the input → must handle gracefully
  ├── (c) Existing test suite: full regression → zero new failures
  └── Score confidence only after all three pass
        │
        ▼
  PromotionEngine decision
  ├── Confident + policy=auto  → promote without human
  ├── Uncertain + policy=human → notify admin, await approval
  └── Failing                  → discard, escalate alert
        │
        ▼
  Apply (if approved)
  ├── QuiesceKernel()
  ├── Hot-swap capability or binary
  ├── ResumeKernel()
  ├── Archive old version (rollback available for 30 days)
  └── Captured payload promoted to permanent regression corpus
```

### 4.2 Kernel Coordination (Quiesce/Resume)

For ARP to safely swap code, the kernel must cooperate:

```rust
pub trait WorkflowCoordinator: Send + Sync {
    // Caller gets exclusive access; guard drop = auto-resume
    async fn quiesce(&self, handles: &[HandleKind]) -> QuiesceGuard;
}

pub enum HandleKind {
    WalSegments,
    SstableFiles,
    KoIndex,
    All,
}
```

Quiesce behavior:
1. New writes enter backpressure queue
2. In-flight operations drain
3. File handles flushed and released
4. Guard issued — caller has exclusive window

This maps to the "file backup" use case directly:
a workflow acquires the quiesce guard, takes a snapshot, releases the guard — kernel resumes normal operation automatically.

### 4.3 Error Capture — Full Input Preservation

Metrics tell you *that* something broke. Captured payloads tell you *what* broke it.  
Without the actual input, a patch is synthesized blind — it may fix the symptom, not the cause.

**What gets captured at the moment of anomaly:**

```rust
pub struct ErrorCapture {
    pub id:           Uuid,
    pub timestamp:    DateTime<Utc>,
    pub syscall:      String,              // "ko_put", "ko_scan", etc.
    pub input_payload: Vec<u8>,            // exact gRPC request bytes (MessagePack)
    pub store_state:   StoreStateSnapshot, // KoStore metadata at time of failure
    pub stack_trace:   String,
    pub metric_context: MetricSnapshot,   // latency, memory, error rate at t=failure
    pub preceding_inputs: Vec<Vec<u8>>,   // last 50 inputs (sliding window)
                                           // ← sequence context for ordering bugs
}
```

Stored immediately as:
```
KO { kind:      "error_capture"
     namespace: "ekos/arp/captures"
     payload:   encrypted(ErrorCapture)
     tags:      ["syscall:ko_put", "severity:critical", "arp_id:<uuid>"] }
```

**Preceding inputs matter** — some bugs are sequence-dependent (e.g., ko_put of type A followed by ko_scan triggers a bug, but ko_put alone does not). The sliding window preserves ordering context for the LLM to analyze.

### 4.4 Automatic Payload Generation (Fuzzing)

Once a patch is synthesized, it is tested against three layers:

**Layer 1 — Original Capture (must pass)**  
Replay the exact `input_payload` that caused the failure.  
If the patch does not fix this exact input → discard immediately.

**Layer 2 — Generated Variants (must handle gracefully)**  
A lightweight fuzzer generates variants of the captured input:

```rust
pub struct PayloadFuzzer;

impl PayloadFuzzer {
    pub fn generate_variants(original: &[u8]) -> Vec<FuzzVariant> {
        vec![
            // Boundary conditions
            Self::empty_payload(),
            Self::max_size_payload(64 * 1024 * 1024),   // 64MB
            Self::single_byte(),

            // Field mutations (MessagePack-aware)
            Self::null_required_fields(original),
            Self::wrong_field_types(original),
            Self::extra_unknown_fields(original),

            // Sequence attacks
            Self::duplicate_send(original),              // same payload twice
            Self::interleave_with_deletes(original),     // concurrent delete

            // Encoding attacks
            Self::truncated(original, original.len() / 2),
            Self::random_bit_flip(original, 3),          // 3 random bit flips

            // Semantic attacks (KO-specific)
            Self::backdated_valid_from(original),        // valid_from = 1970
            Self::future_valid_from(original),           // valid_from = year 2099
            Self::version_jump(original, 9999),          // version = 9999
            Self::strip_all_tags(original),
        ]
    }
}
```

For each variant, expected outcome is one of:
- **Clean error** (graceful rejection, correct error code) ✅
- **Panic / crash** ❌ → patch fails, variant added to regression corpus permanently
- **Silent data corruption** ❌ → worst case, caught by output diff

**Layer 3 — Existing Test Suite + Growing Regression Corpus**  
Every previously-captured `error_capture` that was fixed becomes a permanent regression test.  
The corpus grows with every ARP cycle. Over time ARP becomes harder to fool.

```
Corpus after Phase 5 launch:       18 tests
Corpus after 6 months in production: 18 + N captured regressions
Corpus after 2 years:               18 + M regressions  (M >> N)
```

The system gets smarter about its own failure modes the longer it runs.

### 4.5 ARP Constraints (What ARP Will NOT Do)

- **Never self-patch for BUSINESS, INFRA, or NETWORK errors** — classification gates this
- **Never patch without at least N shadow runs** (configurable minimum)
- **Never patch if output_divergence > threshold** — safety bound
- **Always archive old version** before promoting
- **Always write audit KO** recording: what was patched, why, confidence, who approved

---

## 5. Workflow Runtime + Shadowbox

**Goal:** Any background workflow (backup, index rebuild, audit, report generation) can be injected at runtime, run against real data in a sandbox, and promoted when ready.

### 5.1 Workflow Definition

```rust
pub struct WorkflowDefinition {
    pub name:       String,
    pub trigger:    WorkflowTrigger,     // Schedule, OnEvent, Manual
    pub pre_hook:   Option<KernelHook>,  // e.g. QuiesceHandles
    pub body:       LoadedCapability,    // WASM plugin
    pub post_hook:  Option<KernelHook>,  // e.g. ReacquireHandles
    pub sandbox:    ShadowMode,          // DryRun | ShadowCommit | Live
    pub promotion:  PromotionPolicy,
}
```

### 5.2 Environments as Shadowboxes

No separate DEV/QA/UAT servers. All environments are shadowboxes within the same runtime:

```
PROD runtime
├── Shadowbox("dev")    ← isolated writes, real data reads, unlimited restarts
├── Shadowbox("qa")     ← automated test suite runs here
├── Shadowbox("uat")    ← human acceptance testing
└── Live                ← production traffic
```

Each shadowbox:
- Mirrors reads from the live KoStore
- Writes go to an isolated shadow KvStore (memory or tempdir)
- Has its own metrics collection
- Can be promoted up (dev → qa → uat → live) or rolled back independently

### 5.3 Confidence Scoring

```rust
pub struct ConfidenceScore {
    pub run_count:           u32,
    pub latency_delta_pct:   f64,   // negative = faster, positive = slower
    pub output_divergence:   f64,   // 0.0 = identical output, 1.0 = completely different
    pub error_delta:         f64,   // new errors introduced
    pub resource_overhead:   f64,   // CPU/memory overhead vs baseline
}

impl ConfidenceScore {
    pub fn score(&self) -> f64 {
        // Weighted formula — tunable
        // High weight on error_delta (new errors are disqualifying)
        // Medium weight on output_divergence
        // Low weight on resource_overhead
    }
}
```

### 5.4 Promotion Policy

```rust
pub enum PromotionPolicy {
    RequireHumanApproval { notify: NotifyChannel },
    AutoIfConfident      { threshold: f64, min_runs: u32, max_divergence: f64 },
    HybridGated          { auto_threshold: f64, fallback_to_human: bool },
}
```

---

## 6. Simulation Capabilities

**Goal:** Because EKOS's data model is bitemporal (every KO has `tx_time`) and Shadowbox already mirrors production state, the system is naturally simulation-capable. The following must be first-class features of the Shadowbox design — not retrofitted later.

### 6.1 Policy Replay

*"If I had used a different trust policy over the last N months, what would have happened?"*

EKOS retains full history of every ARP patch and its confidence score. Policy Replay runs a simulation:

```
Input:  historical patch log (last N months) + proposed new TrustPolicy
Output: which patches would have auto-promoted, which would have been held,
        any patch that would have incorrectly auto-promoted (false confidence)
```

This is how admins safely tune confidence thresholds before enabling `AutoIfConfident` or `FullAuto`. No production risk — runs on historical data.

### 6.2 ARP Training Simulation

*"Can I trust ARP enough to enable FullAuto?"*

A `BugLibrary` — KOs containing known past bugs (already fixed) with their ground-truth patches:

```
KO { kind: "known_bug", payload: { symptom_metrics, root_cause, correct_patch } }
```

Simulation workflow:
1. Inject known bug into Shadowbox("dev")
2. Let ARP detect, diagnose, and synthesize a patch
3. Compare ARP's patch against ground-truth patch
4. Score: correct / partially correct / wrong

Run against the full BugLibrary. If ARP scores > threshold → FullAuto is safe to enable. This is ARP's own test suite — not written by developers, grown from production history.

### 6.3 Time-Shifted Traffic Replay (Digital Twin)

*"How does the new patch behave against real production traffic — not synthetic load?"*

Current Shadowbox runs new code against live traffic as it arrives. Time-shifted replay instead:

```
Record: last 24h of production gRPC requests (anonymized, stored as KOs)
Replay: feed that exact traffic sequence into Shadowbox with new code
Compare: responses from new code vs recorded production responses
```

This gives patch validation against real traffic patterns without waiting for live traffic to exercise the changed code path. A patch that fixes a 2am batch job can be validated immediately, not at 2am.

### 6.4 Bitemporal Forward Projection

*"What does the system look like in 6 months?"*

EKOS knows current write rate, KO size distribution, and compaction behavior. Projection simulation:

```
Input:  current metrics + growth rate (fitted from last 90 days)
Output: predicted KoStore size, query latency p99, compaction frequency at T+30/60/90/180 days
```

Used for capacity planning — predict hardware upgrade trigger before it becomes an emergency. Stored as a `KO { kind: "capacity_projection" }` — updated weekly, visible in admin panel.

### 6.5 What-If Indexing

*"Is a new index worth the rebuild cost?"*

Before committing to a new index (e.g., adding a secondary tag index), simulate:

1. Build the new index in Shadowbox on historical data
2. Replay last N days of scan queries against old index vs new index
3. Compare: query latency delta, storage overhead, compaction impact

Decision is data-driven, not guesswork.

### 6.6 Chaos Confidence

*"Does ARP correctly classify failure types before I trust it in production?"*

Deliberately inject failure scenarios into Shadowbox and verify ARP's classification:

| Injected Failure | Expected Classification | Pass If |
|---|---|---|
| Slow disk I/O | Infra Error | ARP does NOT attempt code patch |
| Buggy ko_history() | System Error | ARP synthesizes WASM patch |
| Malformed gRPC payload | Network Error | ARP triggers retry/circuit-break |
| Invalid KO payload | Business Error | ARP alerts owner, no self-patch |

If ARP misclassifies → it is not safe to enable autonomous remediation.  
Chaos runs in Shadowbox on a schedule before any FullAuto trust policy is activated.

### 6.7 Design Constraint for Shadowbox (Phase 4)

All simulation capabilities above require Shadowbox to support:

```rust
pub enum ShadowboxMode {
    LiveMirror,                          // current: mirrors live traffic
    TimeShiftedReplay { recorded_at: DateTime<Utc> },  // 6.3
    PolicyReplay { policy: TrustPolicy, from: DateTime<Utc> }, // 6.1
    ChaosInjection { scenario: ChaosScenario },        // 6.6
    ForwardProjection { horizon_days: u32 },            // 6.4
}
```

Phase 4 must build Shadowbox with this enum — not just `LiveMirror`. Retrofitting later breaks the interface.

---

## 7. Reflexive Encrypted Runtime (Phase 6+)

**Goal:** After first production deployment, GitHub is retired. Source code lives inside EKOS as encrypted KnowledgeObjects. The runtime is its own development environment.

### 6.1 Source as Knowledge Objects

```
KO {
  kind:      "source_file"
  namespace: "<product>/crates/<crate>"
  payload:   AES-256-GCM( file_contents )
  tags:      ["lang:rust", "crate:<name>", "module:<name>"]
  version:   <monotonic — each commit is a new version>
}
```

Entire codebase = a namespace of encrypted KOs. Git history = KO version history.

### 6.2 Build System as Workflow

The compile-link-test pipeline is itself a WorkflowDefinition.  
Triggered by: new source KO version committed.  
Output: new binary artifact stored as a KO.

### 6.3 Bootstrap Problem

One-time ceremony (performed once, before retiring GitHub):
1. Final build from GitHub source
2. All source files encrypted and imported as KOs into production EKOS
3. GitHub repo archived (private, not deleted — emergency fallback only)
4. From this point: all development flows through EKOS admin channel

### 6.4 Key Management

Decryption keys held in HSM (Hardware Security Module) or TEE (Trusted Execution Environment) within the production node.  
The runtime decrypts source only when ARP needs it for synthesis.  
No human ever sees plaintext source in normal operations after bootstrap.

### 6.5 IP Protection Implication

Source cannot be exfiltrated as readable text — only encrypted blobs are accessible.  
Self-patching loop remains intact: runtime decrypts → LLM synthesizes → re-encrypts → stores new version.

---

## 7. Behavioral Fingerprinting (KO Security)

**Goal:** Detect data poisoning, replay attacks, and anomalous write patterns by learning what "normal" KO operations look like — then flagging deviations even if the attacker changes their approach (like an antivirus that catches new builds of known malware).

### 7.1 What Gets Fingerprinted

```
Normal write signature:
  - version increments by 1 per put
  - payload size stays within ±3σ of historical distribution
  - tags change infrequently
  - valid_from is close to tx_time (not backdated excessively)

Attack signatures:
  - Replay attack:     version jumps (same KO put with old version)
  - Poisoning:         massive payload size spike
  - Metadata stripping: all tags removed suddenly
  - Backdating:        valid_from set years in the past
```

### 7.2 Behavioral Model Architecture

Model type: **online statistical model + sliding window** — not a large LLM.  
Must be fast (sub-millisecond per KO write), deterministic, and updateable without restart.

```rust
pub struct BehavioralModel {
    // Per-namespace, per-kind statistics (rolling window, last 30 days)
    pub baselines: DashMap<(Namespace, KoKind), KoOperationBaseline>,
}

pub struct KoOperationBaseline {
    pub payload_size_mean:   f64,
    pub payload_size_stddev: f64,
    pub version_delta_mode:  u64,      // almost always 1
    pub tag_change_rate:     f64,      // tags per N puts
    pub backdating_p99_secs: f64,      // how far valid_from lags tx_time normally
    pub inter_arrival_ms:    f64,      // time between puts for this KO
    pub sample_count:        u64,
}
```

**Training:** Online — model updates itself after every KO write using exponential moving average. No batch training job, no separate training pipeline.

**Model stored as a KO:**
```
KO { kind: "behavioral_model", namespace: "ekos/security/fingerprint",
     payload: serialized(BehavioralModel), version: <updates continuously> }
```
Model is versioned, backed-up with KoStore, recoverable after crash.

### 7.3 Write Path Integration

Fingerprint check runs **synchronously in the ko_put hot path** — must be sub-millisecond:

```rust
// Inside KoStore::ko_put()
pub async fn ko_put(&self, ko: KnowledgeObject) -> Result<KoId> {
    let score = self.fingerprint.score(&ko);   // ← sub-ms check
    match score {
        AnomalyScore::Normal       => {},
        AnomalyScore::Suspicious(s) => self.emit_alert(ko.id, s),
        AnomalyScore::Critical(s)  => return Err(KoError::FingerprintBlocked(s)),
    }
    // normal write path continues...
}
```

Three response levels:
- **Normal** — write proceeds, model updates
- **Suspicious** — write proceeds but alert KO emitted for admin review
- **Critical** — write blocked, full audit KO written, admin notified immediately

### 7.4 Model Update Cycle

```
Every ko_put (normal):
  → score against baseline
  → update baseline with new sample (EMA, α=0.01 — slow drift allowed)
  → persist updated model as new KO version (async, not on hot path)

Weekly:
  → BackupVerifyWorkflow includes model KO in snapshot
  → Chaos simulation runs known attack scenarios against model (Section 6.6)
  → If model fails chaos → alert admin, do not auto-update thresholds
```

### 7.5 Self-Upgrading Core Binary

**Goal:** EKOS upgrades its own core binary (ekosd itself) through the same sandbox → confidence → promotion pipeline used for patches, but with stricter gates.

This is the highest-risk operation in the system. Architecture reflects that:

```
Trigger: ARP determines WASM plugin cannot fix the issue
         (core data structure change, WAL format change, etc.)
         OR: scheduled major version upgrade
      │
      ▼
Build new ekosd binary in Shadowbox("dev")
  using source KOs + BuildWorkflow
      │
      ▼
Shadowbox("qa"): full test suite (18+ tests)
      │
      ▼
Shadowbox("uat"): 72-hour shadow soak
  real traffic replayed (time-shifted replay, Section 6.3)
  metrics compared continuously
      │
      ▼
PromotionEngine: binary upgrade always RequireHumanApproval
  (cannot be set to FullAuto — architectural constraint)
      │
      ▼ Human approves
QuiesceKernel() → drain all in-flight requests
      │
kexec-style handoff (Linux):
  new ekosd process starts, binds same ports
  old ekosd sends FIN to new ekosd when drain complete
  old process exits cleanly
      │
Old binary archived as KO artifact (rollback: start old binary, same handoff in reverse)
```

**kexec-style handoff in Rust:**
```rust
// Old ekosd, after drain complete:
let new_binary_path = artifact_store.fetch("ekosd", new_version)?;
std::process::Command::new(&new_binary_path)
    .env("EKOS_HANDOFF_FROM", old_pid.to_string())
    .env("EKOS_INHERIT_LISTENERS", serialized_socket_fds)
    .spawn()?;
// New process inherits listening sockets via fd passing — zero port rebind
```

Socket file descriptors passed via environment — new binary inherits open ports. Zero downtime. No load balancer change needed.

---

## 8. Phase Roadmap

```
Phase 1  ✅  KoStore (6 syscalls) + ekosd gRPC — DONE
Phase 2       CCR (Content-Contextual Retrieval) + TypeScript SDK
              + Observability instrumentation added to ekos-kernel
              + QuiesceGuard added to KoStore (foundation for ARP)

Phase 3       CIP — Capability Injection Protocol
              ekos-cip crate, admin gRPC channel, WASM synthesis pipeline

Phase 4       Workflow Runtime + Shadowbox
              ekos-workflow crate, environment-as-shadowbox, PromotionEngine

Phase 5       ARP — Autonomous Remediation Protocol
              Full self-heal loop: ObservabilityAgent → classify → synthesize → promote
              Behavioral fingerprinting for KO security

Phase 6       Reflexive Encrypted Runtime
              Source as KOs, build workflow, bootstrap ceremony, HSM key management

Phase 7       Cluster-aware (Raft + FederationFrame)
              All of the above distributed across nodes
```

---

## 9. What This Is Not

- **Not a plugin system.** CIP goes further: capabilities are synthesized from intent, not compiled by a developer in advance.
- **Not Kubernetes self-healing.** K8s restarts crashed pods (infra level). ARP patches logic bugs in running code (system level).
- **Not CI/CD.** CI/CD requires a human to push code. This pipeline requires only a human to approve (and can be configured to not even require that).
- **Not science fiction.** Every piece (WASM sandboxing, LLM synthesis, shadowbox execution, confidence scoring) is proven technology. The combination in a knowledge runtime is novel.

---

*This document is a living architecture capture. Update when phases complete or vision evolves.*  
*Do not implement from this document directly — each phase has its own detailed design.*
