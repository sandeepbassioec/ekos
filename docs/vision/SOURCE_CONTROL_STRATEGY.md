# Internal Source Control Strategy
## VectorVolt + EKOS — Private SCM Built on EKOS Itself

**Status:** Architecture Capture (not scheduled for development)  
**Captured:** 2026-07-07  
**Depends on:** AUTONOMOUS_RUNTIME_VISION.md — Section 6 (Reflexive Encrypted Runtime)

---

## 0. Scope — Both Products, Same System

This strategy applies to **both** VectorVolt and EKOS equally.  
Same SCM engine, same ekosctl tooling, same encryption model, same bootstrap ceremony.

The key difference is that they are **separate products with separate DevOps nodes**:

```
┌─────────────────────────────────┐      artifact dep      ┌─────────────────────────────────┐
│   VectorVolt DevOps Node        │ ──────────────────────► │   EKOS DevOps Node              │
│   (workerruntime.com)           │                         │   (garvionlabs.com)             │
│                                 │                         │                                 │
│  Source KOs:                    │                         │  Source KOs:                    │
│    vectorvolt-wal               │                         │    ekos-kernel                  │
│    vectorvolt-store             │                         │    ekosd                        │
│    vvctl                        │                         │    ekos-scm                     │
│                                 │                         │    ekosctl                      │
│  Publishes:                     │                         │                                 │
│    vectorvolt-store artifact    │                         │  Consumes:                      │
│    (internal registry)          │                         │    vectorvolt-store artifact    │
└─────────────────────────────────┘                         └─────────────────────────────────┘
```

VectorVolt node publishes `vectorvolt-store` to an **internal artifact registry** (not crates.io — that was Phase A).  
EKOS node consumes it as a versioned dependency.  
This mirrors the current path-dep relationship but inside the encrypted runtime.

**Why separate nodes, not one?**
- Different release cadences (VectorVolt is slower-moving, foundational)
- Different teams/access controls (WorkerRuntime vs Garvion Labs)
- EKOS node going down must not affect VectorVolt source
- Clean IP boundary between the two brands

---

## 1. The Problem With External Source Control

GitHub is a general-purpose collaboration platform. For a self-healing, self-patching runtime like EKOS, using GitHub creates a fundamental contradiction:

```
EKOS needs to read its own source to synthesize patches
  → Source lives on GitHub (external)
  → ARP must call out to GitHub at patch time
  → GitHub outage = ARP blind
  → Source is readable by anyone with repo access
  → IP not protected at storage layer
```

The goal is to eliminate this contradiction: **source must live where the runtime lives.**

---

## 2. Three-Phase Transition Strategy

### Phase A — External (Current, Phase 1–5)

```
Source: GitHub (private repo)
CI/CD:  GitHub Actions → crates.io
SCM:    Standard git — branches, PRs, tags
```

Nothing changes during active development phases.  
GitHub is the source of truth. This is intentional — do not rush the transition.

**Exit condition:** Phase 5 complete (ARP working), system proven stable for 90 days in production.

---

### Phase B — Hybrid Mirror (Bridge, ~Phase 5–6)

```
Source: GitHub (still primary) + EKOS mirror (read-only KOs)
```

EKOS begins mirroring its own source as encrypted KOs — but GitHub remains authoritative.  
This gives ARP access to source without requiring the full transition.

```
GitHub push → CI triggers mirror workflow → 
  source files encrypted and upserted into EKOS as KOs →
  ARP can now read source internally during patch synthesis
```

**What this enables:**
- ARP can diagnose and synthesize patches against mirrored source
- Full transition can be tested without risk — GitHub is still the fallback
- Team gets comfortable with source-as-KO workflow

**Exit condition:** Mirror has been accurate for 60 days, bootstrap ceremony rehearsed successfully.

---

### Phase C — EKOS Primary (Full Transition, Phase 6+)

```
Source: EKOS (primary, encrypted KOs)
GitHub: Archived — private, read-only, emergency fallback only
SCM:    EKOS internal SCM (described below)
```

One-time bootstrap ceremony performed (see Section 5).  
All development flows through the EKOS DevOps node from this point.

---

## 3. Internal SCM Architecture

### 3.1 The Single DevOps Node

```
┌─────────────────────────────────────────────────────┐
│                  EKOS DevOps Node                   │
│                                                     │
│  ┌─────────────┐  ┌──────────┐  ┌───────────────┐  │
│  │ Source KOs  │  │  Build   │  │  Shadowboxes  │  │
│  │ (encrypted) │  │ Workflow │  │ dev/qa/uat    │  │
│  └─────────────┘  └──────────┘  └───────────────┘  │
│                                                     │
│  Admin gRPC channel (privileged, mTLS)              │
└─────────────────────────────────────────────────────┘
         ▲                    ▲
    Dev workstation      Dev workstation
    (submits changes)    (reviews, approves)
```

One node holds the authoritative encrypted source.  
Developers do not push to a central server — they submit **change proposals** via the admin gRPC channel.  
The node decides when to accept, build, test, and promote.

---

### 3.2 Branching Model

Git branches do not exist in this model. Instead:

```
Namespace = Branch

main source:    namespace = "ekos/crates/ekos-kernel/main"
feature work:   namespace = "ekos/crates/ekos-kernel/feature-cip"
hotfix:         namespace = "ekos/crates/ekos-kernel/hotfix-20260707"
```

Each namespace is a full copy of the relevant source KOs.  
"Branching" = fork a namespace.  
"Merging" = a MergeWorkflow that reconciles two namespaces into a target.

**Merge workflow behavior:**
- Compares KO versions across source and target namespaces
- Identical files: fast-forward
- Diverged files: LLM-assisted merge (not auto-merge for non-trivial conflicts)
- Conflicts requiring human judgment: flagged, routed to human approval gate

---

### 3.3 Change Proposal (Replaces Pull Request)

```protobuf
message ChangeProposal {
  string id            = 1;  // UUID v7
  string author        = 2;
  string source_ns     = 3;  // "feature-cip"
  string target_ns     = 4;  // "main"
  string description   = 5;  // what and why
  repeated SourceDiff diffs = 6;
  bytes  signature     = 7;  // author's key
}

message SourceDiff {
  string file_path   = 1;
  bytes  old_content = 2;  // encrypted
  bytes  new_content = 3;  // encrypted
}
```

Submitted via admin gRPC → stored as a KO → routed through PromotionEngine.

**ChangeProposal lifecycle:**
```
Submitted → Shadowbox("dev") build triggered
         → Shadowbox("qa") test suite runs
         → Shadowbox("uat") human review
         → PromotionEngine decision
         → Merge into target namespace or reject
```

This is exactly the GitHub PR flow — but internal, encrypted, and with automated confidence scoring.

---

### 3.4 Code Review Model

Reviews are KOs:

```
KO {
  kind:      "code_review"
  namespace: "reviews/<change_proposal_id>"
  payload:   { reviewer, file_path, line_range, comment, verdict }
  tags:      ["proposal:<id>", "file:<path>", "verdict:approved|changes_requested"]
}
```

Reviews are immutable (new version for each edit).  
Approval = PromotionEngine receives minimum N approved review KOs before allowing merge.

---

### 3.5 Audit Trail

Because source files are KOs with full bitemporal history:

```
ko_history("ekos/crates/ekos-kernel/main", "src/store.rs")
  → returns every version of this file ever committed
  → with tx_time (when it was stored) and valid_from (logical commit time)
  → with author tag on each version
  → cannot be deleted or rewritten (EKOS immutability guarantee)
```

This is stronger than `git log` — the history is cryptographically ordered by tx_time and cannot be rebased away.

---

## 4. Access Control Model

```
Namespace-level permissions (EKOS native):

  "ekos/crates/ekos-kernel/main"    → write: [devops-node-key]
                                       read:  [all-dev-keys]

  "ekos/crates/ekos-kernel/feature-*" → write: [author-key]
                                         read:  [all-dev-keys]

  "reviews/*"                          → write: [any-dev-key]
                                         read:  [all-dev-keys]
```

Main namespace: only the DevOps node can write (via PromotionEngine).  
Feature namespaces: author has write, others have read.  
No anonymous read — all access requires a registered dev key.

---

## 5. Bootstrap Ceremony

One-time process, performed once before retiring GitHub:

```
Step 1: Final GitHub build
  → Run full test suite on GitHub
  → Tag as "last-github-release"
  → This is the emergency fallback forever

Step 2: Import source into EKOS
  → For each file in each crate:
       Encrypt with HSM key
       ko_put into "ekos/crates/<crate>/main" namespace
       Tag with: author, git_sha, import_timestamp
  → Verify: ko_history count == git log --follow count per file

Step 3: Verify build workflow
  → Trigger build workflow against freshly imported source KOs
  → Output binary must be byte-for-byte identical to Step 1 binary
  → If not identical: ABORT ceremony, debug, retry

Step 4: Retire GitHub
  → Set GitHub repo to private + archived
  → Remove all contributor write access
  → Document archived URL in EKOS as a config KO (emergency fallback)

Step 5: Key ceremony
  → HSM key shards distributed to N key holders (e.g. 3-of-5 threshold)
  → No single person holds full decryption key
  → Key recovery process documented and tested
```

---

## 6. Emergency Fallback

GitHub archive is never deleted. In catastrophic failure:

```
Scenario: EKOS DevOps node unrecoverable (hardware failure, data corruption)

Recovery:
  1. Spin up new EKOS node from last binary release (GitHub release artifact)
  2. Restore HSM key from shards (key holders convene)
  3. Re-import source from GitHub archive OR restore from EKOS backup KOs
  4. Resume from last known good state
```

RTO (Recovery Time Objective): < 4 hours with documented runbook.

---

## 7. What This Replaces

| GitHub Concept | EKOS Internal SCM Equivalent |
|---|---|
| Repository | EKOS namespace |
| Branch | Namespace fork |
| Commit | KO version (immutable, bitemporal) |
| Pull Request | ChangeProposal KO |
| Code Review | Review KO set |
| Merge | MergeWorkflow |
| git log | ko_history() |
| GitHub Actions | Build Workflow (EKOS-native) |
| GitHub Secrets | HSM-protected config KOs |
| git blame | KO author tag per version |
| Force push | Not possible — EKOS history is append-only |

---

## 8. What This Does Not Replace (Intentionally)

- **Public-facing documentation** — still on GitHub Pages or equivalent
- **Issue tracking** — until EKOS has a purpose-built issue KO schema (future)
- **External contributor workflow** — EKOS SCM is for the core team only; external contributions come in via a separate intake process

---

## 9. Phase Placement

```
Phase 1–5:   GitHub (do not touch this strategy)
Phase 5:     Begin Phase B — hybrid mirror, ARP reads from EKOS source KOs
Phase 6:     Bootstrap ceremony + Phase C transition
Phase 7+:    Multi-node federation — DevOps cluster, not single node
```

---

## 10. Internal Artifact Registry (crates.io Replacement)

Once GitHub is retired, crates.io publishing also stops.  
VectorVolt publishes artifacts internally so EKOS can consume them.

### What It Is

A lightweight gRPC service — `ekos-registry` — that:
- Accepts a compiled `.rlib` / `.crate` package from VectorVolt DevOps node
- Stores it as a KO: `kind="crate_artifact"`, namespace=`"registry/vectorvolt-store/0.2.1"`
- EKOS DevOps node fetches it via Cargo's `--registry` mechanism or a custom resolver

### Cargo Integration

```toml
# ekos/Cargo.toml — Phase 6+
[workspace.dependencies]
vectorvolt-store = { version = "0.2", registry = "garvion-internal" }
```

```toml
# ~/.cargo/config.toml on DevOps nodes
[registries.garvion-internal]
index = "sparse+https://registry.garvionlabs.internal/index/"
```

The registry index itself is served by `ekosd` — a new `RegistryService` alongside `EkosScm` and `EkosKernel`.

### Publish Flow

```
VectorVolt ARP promotes new vectorvolt-store version
  → BuildWorkflow produces .crate artifact
  → EkosRegistry.Publish(artifact) → stored as KO
  → EKOS DevOps node's next build resolves new version automatically
  → No manual crates.io publish step
```

---

## 11. Embedded SCM Engine — Build vs Buy Decision

### What Gets Embedded

The SCM is NOT a separate embedded binary (no libgit2, no gitoxide). It is:

```
ekos-scm crate  ←  thin SCM logic layer
    │
    ├── KoStore          (object store — already built)
    ├── imara-diff       (diff engine — pure Rust, borrowed)
    └── diffy            (three-way merge — pure Rust, borrowed)

ekosctl binary  ←  developer-facing CLI (like git, but gRPC-backed)
```

Total new code: ~1500 lines. Everything else reused or borrowed.

### Why Not Embed Git

- git object model (blobs/trees/commits) duplicates KO model — two parallel histories
- libgit2 is a C dependency — breaks EKOS's pure Rust constraint
- gitoxide (pure Rust git) has incomplete merge operations as of 2026
- We only need diff + merge from gitoxide; borrowing `imara-diff` + `diffy` is cleaner

### Control Plane

New gRPC service added to ekosd (same binary, not a separate server):

```protobuf
service EkosScm {
  rpc ForkNamespace(ForkRequest)        returns (ForkResult);
  rpc SubmitProposal(ChangeProposal)    returns (ProposalHandle);
  rpc GetProposalStatus(ProposalHandle) returns (ProposalStatus);
  rpc ApproveProposal(ApprovalRequest)  returns (ApproveResult);
  rpc FileDiff(DiffRequest)            returns (DiffResult);
  rpc FileLog(LogRequest)              returns (stream KoVersion);
  rpc CheckoutFile(CheckoutRequest)    returns (EncryptedFileContent);
}
```

### Developer Encryption Flow

```
ekosctl push (developer machine, plaintext local)
  → encrypt files with DevOps node's public key (session key exchange)
  → ChangeProposal{encrypted_diffs} over gRPC

DevOps node (ekosd + HSM)
  → unwrap session key with HSM private key
  → decrypt → store as KO (re-encrypt with HSM master key)
  → plaintext never on disk outside HSM boundary
```

Developer never holds master key. Master key never leaves HSM.

### crates in Phase 6

```
ekos/crates/ekos-scm/    ←  SCM engine (ForkNamespace, propose, diff, merge, file_log)
ekos/bin/ekosctl/        ←  developer CLI (checkout, push, status, log, review)
```

---

*Do not implement any part of this document until Phase 5 is stable in production.*  
*Update this document when bootstrap ceremony is rehearsed or key management approach changes.*
