# Dependency Self-Upgrade Architecture
## VectorVolt + EKOS — Upgrading Rust, Libraries, and Linux Kernel in Sandbox

**Status:** Architecture Capture (Phase 6+)  
**Captured:** 2026-07-07  

---

## 1. The Problem

EKOS depends on things it does not own:
- Rust compiler (`rustc`, `cargo`, `std`)
- Third-party crates (`tokio`, `tonic`, `wasmtime`, `chrono`, etc.)
- Linux kernel (syscalls, namespaces, eBPF ABI)

Each carries security vulnerabilities, performance improvements, and breaking changes.  
Manual upgrades mean human-in-the-loop for every `cargo update`. At Phase 6+ scale, this contradicts the autonomous runtime vision.

**Goal:** Upgrade all dependencies — including the compiler and kernel — through the same sandbox → confidence → promotion pipeline used for code patches. Human approval gates remain configurable.

---

## 2. Three Upgrade Classes (Different Risk, Different Process)

```
Class A — Library Dependencies (Cargo crates)
  Risk:     Low–Medium (well-defined API contracts, semver)
  Trigger:  New patch/minor version available
  Pipeline: Shadowbox("dev") build → test suite → PromotionEngine
  Gate:     AutoIfConfident(98%) for patch, RequireApproval for minor

Class B — Rust Toolchain (rustc, cargo, std)
  Risk:     Medium (compiler changes can subtly change behavior/output)
  Trigger:  New stable channel release
  Pipeline: Full rebuild of all crates → extended soak in Shadowbox("uat") 72h
  Gate:     RequireHumanApproval always

Class C — Linux Kernel
  Risk:     High (kernel ABI changes affect syscalls, namespaces, eBPF)
  Trigger:  LTS kernel security advisory OR scheduled quarterly upgrade
  Pipeline: Three sub-paths depending on scope (see Section 5)
  Gate:     RequireHumanApproval always + rollback window 7 days
```

---

## 3. Class A — Library Dependency Upgrade

### Trigger

`DependencyWatchWorkflow` (runs weekly):
1. Fetches latest versions from internal artifact registry (Phase 6) or crates.io (Phase 1–5)
2. Compares against current `Cargo.lock`
3. Identifies: patch updates (auto-eligible), minor updates (approval-eligible), major (manual only)

### Pipeline

```
New patch version detected: tokio 1.45.2 → 1.45.3
      │
Shadowbox("dev"):
  cargo update -p tokio --precise 1.45.3
  cargo build --all
      │ build passes
Shadowbox("qa"):
  cargo test --all
      │ tests pass
ConfidenceScore:
  - build clean:          +40 pts
  - tests 100% pass:      +40 pts
  - binary size delta:    ±5% allowed
  - no new unsafe blocks: +20 pts
      │ score ≥ 98%
PromotionEngine → auto-promote
  Cargo.lock updated as new KO version
  Rebuild triggered (BuildWorkflow)
  New binary promoted through normal binary upgrade path
```

### Dependency Vulnerability Scanning

`DependencyWatchWorkflow` also runs `cargo audit` against the advisory database.  
A known CVE in any dependency → immediate alert KO + RequireApproval upgrade triggered, bypassing normal schedule.

---

## 4. Class B — Rust Toolchain Upgrade

### The Bootstrap Non-Problem

Rust compiler is distributed as pre-built binaries via `rustup`.  
We do NOT recompile `rustc` from source. We upgrade the toolchain channel:

```
Current: rust-stable-1.80
New:     rust-stable-1.81

Upgrade does not require self-hosting — just swap the toolchain binary.
```

### Pipeline

```
New rustc stable released (1.81)
      │
Download toolchain KO: encrypted(rust-1.81-x86_64-linux-musl.tar.gz)
      │
Shadowbox("dev"):
  Extract new toolchain to tmpdir
  Rebuild ALL ekos + vectorvolt crates with new compiler
  Capture: any new warnings promoted to errors? any behavior changes?
      │
Shadowbox("qa"):
  Full test suite with new-compiler binary
      │
Shadowbox("uat"):
  72-hour time-shifted traffic replay (Section 6.3 of AUTONOMOUS_RUNTIME_VISION.md)
  Compare: new-compiler binary vs old-compiler binary response-by-response
      │
ConfidenceScore must exceed 99% (higher bar than code patches)
      │
RequireHumanApproval — cannot be set to FullAuto for toolchain upgrades
      │ Approved
Toolchain KO version bumped in EKOS
Old toolchain KO archived (rollback: rebuild with old toolchain KO)
```

### Why 72-Hour Soak

Compiler upgrades can introduce subtle changes:
- Different codegen → different timing behavior under load
- New MIR optimizations → different memory layout → different cache behavior
- Changed default lint levels → new warnings in dependencies

72 hours of real traffic replay surfaces timing-dependent issues that a test suite cannot catch.

---

## 5. Class C — Linux Kernel Upgrade

Three sub-paths, from least to most invasive:

### 5a. eBPF Programs (No Kernel Recompile — Preferred)

Most kernel-level customizations can be done via eBPF without touching the kernel itself:
- Network packet filtering (XDP)
- Syscall tracing and auditing
- Custom scheduler hints
- Namespace isolation helpers

```rust
// ekos-ebpf crate — eBPF programs compiled via aya-rs (pure Rust eBPF)
// Loaded into the running kernel at runtime, no reboot needed
let prog = BpfLoader::new().load(include_bytes!("ko_write_tracer.o"))?;
prog.attach("kprobe/sys_write", ProgramType::KProbe)?;
```

eBPF programs go through the normal CIP pipeline — they are capabilities, not kernel changes.  
Kernel eBPF verifier sandboxes them: a buggy eBPF program cannot crash the kernel.

### 5b. Kernel Module (Loadable, No Recompile)

For deeper integration that eBPF cannot express:
- Custom filesystem drivers
- Hardware-specific optimizations
- Namespace extensions

```bash
# Kernel module loaded/unloaded at runtime — no reboot
insmod ekos_namespace_ext.ko
rmmod ekos_namespace_ext
```

Module goes through full sandbox pipeline. Safety constraint: module must compile for the CURRENT running kernel version — ABI compatibility enforced.

### 5c. Full Kernel Upgrade (kexec — No Hardware Reboot)

For security patches to the kernel itself (CVEs) or LTS version upgrades.  
Uses `kexec` — loads a new kernel into memory and executes it without hardware reboot.  
The user is correct: this is the same technique Linux kernel developers use to test kernel patches.

```
New LTS kernel available (6.6 → 6.12 LTS)
      │
Download kernel source (or pre-built LTS binary from trusted mirror)
      │
Shadowbox(dedicated kernel test VM — separate from main shadowboxes):
  Boot new kernel in VM
  Run EKOS test suite inside VM
  Run Linux Test Project (LTP) suite
  72-hour soak in VM
      │
RequireHumanApproval (always, no exceptions)
      │ Approved
Stage new kernel in /boot
kexec -l /boot/vmlinuz-6.12 --initrd=/boot/initrd-6.12
      │
Quiesce all EKOS services (QuiesceKernel — same as binary upgrade)
      │
kexec -e    ← executes new kernel, no hardware reboot
            ← new kernel boots in ~10 seconds
            ← EKOS services restart automatically (systemd)
            ← ports re-acquired via socket activation
      │
7-day rollback window:
  Old kernel stays in /boot
  One-command rollback: kexec old kernel
  After 7 days: old kernel removed, archived as KO artifact
```

**Why kexec instead of full reboot:**
- Hardware reboot = ~2–5 minute downtime
- kexec = ~10–30 second downtime
- kexec skips BIOS/UEFI POST, goes straight from old kernel to new kernel
- EKOS can drain in-flight requests before kexec — near-zero downtime

### 5d. The Really Bold One — Live Kernel Patching

Linux supports **kpatch** and **livepatch** — applying patches to a running kernel without any restart:

```bash
# Apply security patch to running kernel — zero downtime
kpatch load ekos-kernel-cve-2026-patch.ko
```

EKOS can automate this for critical CVEs:
- CVE advisory received
- Patch synthesized (or downloaded from trusted source)
- Tested in Shadowbox kernel VM
- Applied to production kernel live
- Zero downtime, zero kexec

This is Phase 7+ territory but architecturally follows the same pattern.

---

## 6. Upgrade Decision Matrix

```
Dependency type    │ Patch version │ Minor version │ Major version │ CVE
───────────────────┼───────────────┼───────────────┼───────────────┼──────────────
Cargo crate        │ AutoIfConf 98%│ RequireApproval│ Manual        │ RequireApproval
                   │               │               │ (ChangeProposal│ (expedited)
───────────────────┼───────────────┼───────────────┼───────────────┼──────────────
Rust toolchain     │ RequireApproval│ RequireApproval│ RequireApproval│ RequireApproval
                   │ (patch = same │               │               │ (expedited)
                   │ as minor here)│               │               │
───────────────────┼───────────────┼───────────────┼───────────────┼──────────────
eBPF program       │ AutoIfConf 95%│ RequireApproval│ RequireApproval│ AutoIfConf 95%
───────────────────┼───────────────┼───────────────┼───────────────┼──────────────
Kernel module      │ RequireApproval│ RequireApproval│ Manual        │ RequireApproval
───────────────────┼───────────────┼───────────────┼───────────────┼──────────────
Linux kernel (full)│ n/a           │ n/a           │ RequireApproval│ livepatch (auto)
                   │               │               │               │ kexec (approval)
```

---

## 7. Rollback for Every Upgrade Class

Every upgrade stores the previous artifact as a KO before promotion:

```
KO { kind: "upgrade_rollback_artifact",
     namespace: "ekos/upgrades/<class>/<component>",
     payload:   encrypted(previous artifact or config),
     tags:      ["from_version:X", "to_version:Y", "promoted_at:<timestamp>"],
     valid_until: now + 30_days  // after 30 days, rollback artifact expires
}
```

One-command rollback from admin panel for any upgrade, any class.

---

## 8. Crate Addition

```
ekos/crates/ekos-upgrade/
  src/
    dependency.rs   ← Cargo crate upgrade workflow
    toolchain.rs    ← Rust toolchain upgrade workflow
    kernel.rs       ← eBPF loader, kexec orchestration, livepatch
    advisory.rs     ← CVE advisory watcher, expedited pipeline trigger
  Cargo.toml
```

---

## 9. Phase Placement

```
Phase 4:    ekos-upgrade crate scaffolding
            Class A (library deps) — DependencyWatchWorkflow
Phase 5:    Class B (Rust toolchain upgrade)
Phase 6:    Class C.a (eBPF programs via aya-rs)
            Class C.b (kernel modules)
Phase 7:    Class C.c (kexec full kernel upgrade)
Phase 7+:   Class C.d (livepatch — zero downtime CVE patching)
```

---

*The goal is not a system that never needs upgrades.*  
*The goal is a system where every upgrade — including its own compiler and kernel — follows the same*  
*sandbox → confidence → approval → promote pipeline.*  
*Humans stay in control. The process is automated. The decision is not.*
