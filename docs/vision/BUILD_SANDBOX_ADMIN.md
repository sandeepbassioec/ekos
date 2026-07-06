# Build System, Sandbox Architecture & Admin Control Plane
## VectorVolt + EKOS

**Status:** Architecture Capture (implement Phase 3–4)  
**Captured:** 2026-07-07  

---

## 1. The Toolchain Problem — And Why It's Simpler Than It Looks

### Two Levels of Patching (Different Toolchain Requirements)

```
Level 1 — WASM Plugin Patches (CIP + most ARP cases)
  Input:  Rust source
  Output: .wasm binary (platform-independent)
  Runs:   wasmtime sandbox (same on every OS)
  Toolchain needed: rustc + wasm32-wasi target (ONE build, works everywhere)

Level 2 — Core Binary Patches (rare — only when WASM cannot fix it)
  Input:  Rust source (ekos-kernel, ekosd)
  Output: native binary for specific target
  Runs:   replaces the ekosd process itself
  Toolchain needed: target-specific (see below)
```

**80% of self-patching is Level 1.** WASM output is platform-independent — one build, wasmtime runs it on any Linux distro, any architecture, any kernel version. No distribution problem.

Level 2 is rare and handled separately.

---

## 2. Toolchain Distribution Strategy

### Level 1 — WASM Toolchain

One toolchain, stored as encrypted KOs in EKOS:

```
KO {
  kind:      "build_toolchain"
  namespace: "toolchain/rust-stable/wasm32-wasi"
  payload:   encrypted( rust-toolchain-wasm32.tar.gz )
  tags:      ["rust_version:1.80", "target:wasm32-wasi"]
}
```

Build workflow extracts it to tmpdir, compiles, discards. Output `.wasm` works everywhere.

### Level 2 — Native Toolchain Matrix

For core binary patches, we only support a **fixed set of target triples**:

```
Supported targets (Phase 6):
  x86_64-unknown-linux-musl    ← ALL Linux distributions (static, no glibc dep)
  aarch64-unknown-linux-musl   ← ARM Linux (Raspberry Pi, AWS Graviton)
  x86_64-pc-windows-msvc       ← Windows (MSVC linker)

NOT supported:
  x86_64-unknown-linux-gnu     ← glibc version hell, skip entirely
  x86_64-pc-windows-gnu        ← MinGW, unnecessary complexity
  Individual Linux distro targets ← solved by musl
```

**musl = no glibc = no "which version of glibc" problem.**  
A musl binary built on Ubuntu runs identically on Alpine, RHEL, Debian, Arch.

### Toolchain KOs per target

```
toolchain/rust-stable/x86_64-unknown-linux-musl    → rustc + musl-cross toolchain
toolchain/rust-stable/aarch64-unknown-linux-musl   → rustc + aarch64 musl-cross
toolchain/rust-stable/x86_64-pc-windows-msvc       → rustc + MSVC build tools
toolchain/linker/musl-cross-x86_64                 → musl-gcc linker only
toolchain/sdk/windows-kit-10                       → Windows SDK (for MSVC target)
```

All stored as encrypted KOs. Build workflow fetches only what it needs for the current target.

### Toolchain Update Flow

```
Rust releases new stable version
  → Admin uploads new toolchain to EKOS via ekosctl:
    ekosctl toolchain upload --target wasm32-wasi --rust-version 1.81 ./rust-1.81-wasm.tar.gz
  → Stored as new KO version
  → Old version kept (rollback available)
  → Build workflows automatically use latest unless pinned
```

---

## 3. Sandbox Architecture

### What a Sandbox Actually Is

A sandbox is **not a Docker container**. It is a Linux namespace group created directly via kernel syscalls. No Docker daemon, no container runtime dependency.

```rust
// ekos-workflow/src/sandbox.rs

use nix::sched::{unshare, CloneFlags};
use nix::mount::{mount, MsFlags};

pub struct Shadowbox {
    name:        String,
    overlay_dir: tempfile::TempDir,   // writable layer (tmpfs)
    ko_store:    Arc<KoStore>,        // reads from production, writes to overlay
}

impl Shadowbox {
    pub fn create(name: &str, production: Arc<KoStore>) -> Result<Self> {
        // New mount + network + PID namespace
        unshare(
            CloneFlags::CLONE_NEWNS   |  // filesystem namespace
            CloneFlags::CLONE_NEWNET  |  // network namespace (no external calls)
            CloneFlags::CLONE_NEWPID,    // PID namespace
        )?;

        let overlay = tempfile::tempdir()?;

        // Mount tmpfs for writable layer — disappears on drop
        mount(
            Some("tmpfs"),
            overlay.path(),
            Some("tmpfs"),
            MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID,
            None::<&str>,
        )?;

        Ok(Shadowbox {
            name: name.to_string(),
            overlay_dir: overlay,
            ko_store: production,
        })
    }
}
```

On Windows: **Windows Job Objects** + filesystem redirects (same concept, different API).

### What Each Sandbox Sees

```
Shadowbox("dev"):
  Reads:  production KoStore (real data, read-only view)
  Writes: overlay tmpfs (discarded on sandbox teardown)
  Network: isolated (cannot call external services)
  Purpose: build + basic smoke test

Shadowbox("qa"):
  Reads:  production KoStore
  Writes: overlay tmpfs
  Network: isolated
  Purpose: automated test suite runs here
           18 kernel tests + generated regression tests

Shadowbox("uat"):
  Reads:  production KoStore
  Writes: overlay tmpfs
  Network: internal only (admin panel can reach it)
  Purpose: human acceptance — admin can inspect state,
           run manual queries, compare with production

Live (production):
  Reads:  production KoStore
  Writes: production KoStore
  Network: full
  Purpose: actual traffic
```

### Promotion Flow Across Sandboxes

```
Shadowbox("dev")   →[build passes]→   Shadowbox("qa")
                                           ↓
                                    [test suite passes]
                                           ↓
                                    PromotionEngine.evaluate()
                                           ↓
                              ┌────────────┴───────────────┐
                              │                            │
                        AutoIfConfident              RequireApproval
                              │                            │
                        Live promotion             Shadowbox("uat")
                                                         ↓
                                               Admin reviews in panel
                                                         ↓
                                               Approve → Live promotion
                                               Reject  → Discarded
```

---

## 4. Admin Control Plane

### Architecture

`ekos-admin-ui` is a single-page web app served by `ekosd` on a dedicated admin port.  
No separate server. No cloud dependency. Runs on the same node.

```
ekosd listens on:
  :50051  — gRPC (application traffic)
  :50052  — gRPC (admin channel, mTLS required)
  :8443   — HTTPS (admin web UI, MFA-gated)
```

The web UI is compiled to static assets and embedded in the ekosd binary via `include_bytes!`.  
No separate deploy. Update ekosd → admin UI updates automatically.

```rust
// ekosd/src/admin_ui.rs
static ADMIN_UI_DIST: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/admin-ui.tar.gz"));
```

### Admin Login Flow

```
Admin opens https://<server>:8443/admin
    ↓
Username + Password  (stored as bcrypt-hashed config KO, not plaintext)
    ↓
TOTP code  (same pam_google_authenticator secret, verified in Rust)
    ↓
Session JWT issued (short-lived: 4 hours, signed with server key)
    ↓
Admin panel loads
```

Admin login itself uses the same MFA mandate as SSH (Section 5 of MFA_SESSION_MANDATE.md).  
No MFA = no admin panel. No exceptions.

---

## 5. Admin Panel — What It Shows

### Dashboard

```
┌─────────────────────────────────────────────────────────────────┐
│  EKOS Admin                              Node: prod-01  ● Live  │
├─────────────────────────────────────────────────────────────────┤
│  Pending Approvals (2)                                          │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ ARP-2026-07-07-001  │ System  │ ko_history latency fix    │  │
│  │ Confidence: 94%     │ 3h ago  │ [Shadowbox: qa ✅ uat ✅]  │  │
│  │ [View Diff] [Approve] [Reject] [More Info]                │  │
│  ├───────────────────────────────────────────────────────────┤  │
│  │ CIP-2026-07-07-002  │ Capability │ stale_contract_scan   │  │
│  │ Confidence: 97%     │ 1h ago  │ [Shadowbox: qa ✅ uat ✅]  │  │
│  │ [View Diff] [Approve] [Reject] [More Info]                │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                  │
│  Trust Policy                        Recent Activity (24h)      │
│  ARP:  ● Require Approval            12 patches auto-promoted   │
│  CIP:  ● Auto if > 95%                2 patches pending         │
│  WF:   ● Require Approval             1 patch rejected          │
└─────────────────────────────────────────────────────────────────┘
```

### Diff View

For each pending change:
- Source file before / after (syntax-highlighted)
- Confidence score breakdown (latency delta, error rate, output divergence)
- Shadow test results (which tests passed, which failed)
- Which error was being fixed (the original anomaly that triggered ARP)
- UAT sandbox: live query console to test the patched version manually

---

## 6. Trust Policy Configuration

### What Admin Can Configure

```rust
pub struct TrustPolicy {
    pub subject:   TrustSubject,    // ARP, CIP, Workflow, or specific named item
    pub mode:      TrustMode,
    pub updated_by: String,         // admin username
    pub updated_at: DateTime<Utc>,  // immutable audit trail
}

pub enum TrustSubject {
    AllArpPatches,
    AllCipCapabilities,
    AllWorkflows,
    NamedCapability(String),        // e.g. "stale_contract_scanner"
    NamedWorkflow(String),          // e.g. "nightly_backup"
}

pub enum TrustMode {
    RequireApproval,
    AutoIfConfident { threshold: f64, min_shadow_runs: u32 },
    FullAuto,
}
```

### Admin Panel Trust Settings

```
┌─────────────────────────────────────────────────────────────────┐
│  Trust Policy Configuration                                     │
├──────────────────────────────────┬──────────────────────────────┤
│  Subject                         │  Mode                        │
├──────────────────────────────────┼──────────────────────────────┤
│  ARP — all patches               │  ● Require Approval     [▼]  │
│  CIP — all capabilities          │  ● Auto if > 95%        [▼]  │
│  Workflow: nightly_backup        │  ● Auto if > 90%        [▼]  │
│  Workflow: index_rebuild         │  ● Require Approval     [▼]  │
│  Capability: stale_contract_scan │  ● Full Auto            [▼]  │
├──────────────────────────────────┴──────────────────────────────┤
│  [▼] dropdown options:                                          │
│      Require my approval (always)                               │
│      Auto if confident  → set threshold: [95]% min runs: [50]  │
│      Full auto (I trust this completely)                        │
│                                                                  │
│  [Save Changes]  — change stored as immutable audit KO          │
└─────────────────────────────────────────────────────────────────┘
```

### Overriding at Any Time

Admin can change trust mode at any time — mid-session, mid-approval-queue.  
If mode changes from `FullAuto` to `RequireApproval` while a patch is being promoted:
- In-progress promotion completes (atomic, cannot be half-done)
- Next patch follows new policy immediately

All policy changes stored as immutable KOs:
```
KO {
  kind:      "trust_policy_change"
  namespace: "admin/audit"
  payload:   { admin, subject, old_mode, new_mode, reason }
  tags:      ["admin:<username>", "subject:<name>"]
}
```
Cannot be deleted. Full audit trail forever.

---

## 7. Emergency Override

Admin can force-stop all autonomous activity instantly:

```
Admin panel top-right: [🔴 Emergency Stop]
```

Effect (immediate):
- All pending ARP promotions: paused
- All pending CIP injections: paused
- All Shadowbox workflows: stopped
- System continues serving live traffic normally
- Everything that was queued remains queued (not discarded)

To resume: Admin clicks `[Resume Autonomous Operations]` and re-authenticates with TOTP.

This is the "I don't trust it right now" button. One click. No code change needed.

---

## 8. Tech Stack for Admin UI

```
Frontend:  Svelte + TypeScript (compiles to ~80KB, no React runtime overhead)
           Served as embedded static assets inside ekosd binary
           No CDN, no external JS, air-gap friendly

Backend:   New HTTP service inside ekosd (axum, already a dependency)
           All data from admin gRPC service (no separate DB)
           Session JWT signed with server key (stored in config KO)

Auth:      bcrypt passwords + TOTP (same secret as SSH PAM)
           Short-lived JWT (4h), refresh requires re-TOTP

Port:      :8443 HTTPS only (self-signed cert on first boot, ACME on cloud)
```

Why Svelte (not React/Vue):
- Compiles to vanilla JS — no virtual DOM runtime
- Entire UI embeds in ekosd binary with negligible size impact
- No npm at runtime — air-gap deployments work

---

## 9. Crates Summary

```
ekos/crates/
  ekos-kernel/       ← existing
  ekosd/             ← existing (gains :8443 admin HTTP + build_admin_ui in build.rs)
  ekos-security/     ← Phase 2 (MFA guard)
  ekos-workflow/     ← Phase 4 (Shadowbox + namespace isolation)
  ekos-build/        ← Phase 4 (toolchain KO fetch + compile + artifact store)
  ekos-admin/        ← Phase 4 (TrustPolicy, AdminSession, ProposalQueue gRPC service)

ekos/admin-ui/       ← Phase 4 (Svelte SPA, compiled into ekos-build output)
```

---

## 10. Phase Placement

```
Phase 2:   ekos-security (MFA guard) — already captured
Phase 3:   CIP — capability injection, admin panel minimal (just list/approve capabilities)
Phase 4:   Full admin panel — Shadowbox, PromotionEngine, TrustPolicy, all subjects
Phase 5:   ARP — admin panel gains ARP diff view, source-level patch review
Phase 6:   Toolchain KO store — build workflows with fetched toolchains
```

Admin panel grows phase by phase. Each phase adds subjects to the trust policy UI.

---

*Build toolchain, sandbox isolation, and human approval are one unified system.*  
*musl = no distro fragmentation. Namespaces = no Docker dependency. Embedded UI = no separate deploy.*
