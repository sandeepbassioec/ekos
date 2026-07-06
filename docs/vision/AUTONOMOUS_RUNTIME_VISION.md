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

**D. Dynamic Routing**
Capabilities are addressable by name:
```protobuf
rpc ScanByCapability(CapabilityScanRequest) returns (stream KnowledgeObject);

message CapabilityScanRequest {
  string capability_name = 1;
  string namespace       = 2;
  bytes  params          = 3;  // capability-specific (MessagePack)
}
```

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
  Diagnosis Phase
  ├── Collect: metric history, recent traces, error context
  ├── Identify: which code path is implicated
  └── LLM prompt: "Given this symptom and this source, what is wrong?"
        │
        ▼
  Synthesis Phase
  ├── LLM generates candidate patch (Rust or WASM plugin level)
  ├── If WASM level → compile to plugin, load in shadow
  └── If core level → compile new binary in shadow environment
        │
        ▼
  Validation Phase
  ├── Run patch against shadow traffic (real data, shadowed writes)
  ├── Compare metrics: patched vs unpatched
  └── Score confidence (see Section 5.3)
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
  └── Archive old version (rollback available for 30 days)
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

### 4.3 ARP Constraints (What ARP Will NOT Do)

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

## 6. Reflexive Encrypted Runtime (Phase 6+)

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

### 7.2 Behavioral Model

Trained on normal operation telemetry.  
Input: a sequence of KO operations.  
Output: anomaly score per operation.  
Model type: lightweight time-series anomaly detection (not a large LLM — needs to be fast).

Attacker changes binary/encoding → behavioral signature unchanged → still flagged.

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
