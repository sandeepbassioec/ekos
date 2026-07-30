# Layer 18 — Deployment & Auto-Configuration (`VV-CFG-*`)

**Added 2026-07-30.** This layer did not exist in the original 321-feature matrix. It closes a gap:
runtime dispatch handles *instructions*, but nothing handled *system configuration*. One codebase,
one binary per architecture, and the system configures itself per host at install time.

Status: ✅ done · 🟡 partial · ⬜ pending · 🧊 env-gated · 🔬 research

## Rollup

| Group | Features | Tasks | ✅ | ⬜ | 🧊 |
|---|---|---|---|---|---|
| Detection & profiling | 5 | 34 | 0 | 32 | 2 |
| Config derivation | 8 | 49 | 0 | 49 | 0 |
| System tuning (privileged) | 6 | 37 | 0 | 29 | 8 |
| **Re-tune, drift & reconciliation** | **5** | **37** | 0 | 37 | 0 |
| Package integration | 8 | 46 | 0 | 42 | 4 |
| Precedence & safety | 4 | 24 | 0 | 24 | 0 |
| **Total** | **36** | **227** | **0** | **213** | **14** |

**Revised 2026-07-30** (+2 features, +20 tasks): VV-CFG-035 (startup reconciliation /
clamp-and-report) and VV-CFG-036 (under-utilisation detection) were added after a question exposed
that the original design covered server *downgrade* but not *upgrade* — see the
detection-vs-reconciliation section below.

---

## The three configuration layers

Conflating these is how projects end up with multiple codebases. They are distinct and each has a
different mechanism.

| Layer | When | Detects | Produces | Can it change the system? |
|---|---|---|---|---|
| **Build** | compile time | target architecture | one binary per target triple; optional linked features | no |
| **Install** ← *this layer* | package install / first run | topology, RAM, disks, kernel, privileges | **an annotated config file**, optionally applied system tuning | yes, with explicit consent |
| **Runtime** | process start | CPU instructions (`cpuid`) | dispatch table | no |

**Runtime** is built (VV-HAL-004/005). **Build** is handled (5 CI targets, Cargo features for
DPDK/RDMA). **Install** is this layer, and it was missing entirely.

### Why these specific things cannot be runtime-detected

| Setting | Why not runtime |
|---|---|
| Huge page count | `sysctl vm.nr_hugepages` is a system change needing root — a database process must not mutate kernel settings on startup |
| `isolcpus` / `nohz_full` | Kernel **boot parameters**. Require a reboot. Cannot be set by a running process at all |
| memlock limit | `/etc/security/limits.conf` — applies at session creation, before our process exists |
| Core role map | Needs a *policy decision* (which cores serve writers vs readers), not just a capability reading. On hybrid CPUs it also needs P-core/E-core classification |
| Arena sizes | Depend on total RAM **and** on how much of the box the operator intends to give us |
| WAL device | Depends on what is mounted where — a policy choice between local NVMe (fast, ephemeral) and network storage (durable, slower) |

---

## Non-negotiable safety principles

Every task below is constrained by these. They are what separate "intelligent" from "hostile".

1. **Never silently change the system.** Dry-run is the default. Privileged changes require explicit
   opt-in and print exactly what will be done, before doing it. A package that quietly rewrites
   kernel parameters is malware behaviour, regardless of intent.
2. **Never fail the install.** If tuning cannot run — no root, locked-down container, unknown
   platform — emit safe defaults, warn, and succeed. A database that refuses to install because it
   could not reserve huge pages is broken, not careful.
3. **Generated config is human-readable and hand-editable.** Auto-tune produces a *starting point*,
   never an opaque blob. An operator must be able to read it, understand it, and override it.
4. **Every derived value records its reasoning.** `huge_pages = 2048  # 4 GiB of 62 GiB detected`.
   Without the "why", nobody can safely change it six months later.
5. **Auto-tune never overrides an explicit choice.** Precedence is
   `generated < config file < environment < CLI flag`.
6. **Re-runnable and idempotent.** VMs get resized. Running tune twice must change nothing the
   second time.
7. **Refuse to run on hardware the config was not generated for** — warn loudly rather than silently
   applying a 64-core core-map on a 4-core box.

---

# Group 1 — Detection & profiling

### VV-CFG-001 — `SystemProfile`: cross-platform host capability detection  ⬜
**Blocked by:** VV-HAL-001 (CpuTopology)
**Note:** `scripts/vv-probe-linux.sh` is a shell prototype of this. The real thing must be Rust,
cross-platform, and machine-readable — the shell script stays as a zero-dependency diagnostic.

| Task | Status | Detail |
|---|---|---|
| VV-CFG-001.1 | ⬜ | Define `SystemProfile` struct: cpu, memory, storage, kernel, privileges, network |
| VV-CFG-001.2 | ⬜ | Linux implementation via `/proc`, `/sys` |
| VV-CFG-001.3 | ⬜ | Windows implementation via `GetLogicalProcessorInformationEx`, `GlobalMemoryStatusEx` |
| VV-CFG-001.4 | ⬜ | macOS implementation via `sysctlbyname` |
| VV-CFG-001.5 | ⬜ | Serialize to JSON (`vvctl profile --json`) for tooling and bug reports |
| VV-CFG-001.6 | ⬜ | Detect virtualization (hypervisor bit, DMI) — steal time makes latency targets unmeetable |
| VV-CFG-001.7 | ⬜ | Detect cloud provider + instance type (EC2 IMDS, GCP/Azure metadata) — best-effort, must not hang without network |
| VV-CFG-001.8 | ⬜ | Test: profile parses on all three OSes in CI |

### VV-CFG-002 — Hybrid-core classification (P-core / E-core / LP-E)  ⬜
**Spec:** discovered while profiling the dev laptop (Intel Core Ultra 7 165H, 6 P + 8 E + 2 LP-E)
**Why it matters:** pinning a latency-critical writer to an E-core would actively harm p99. A
homogeneous server would have hidden this requirement until much later.

| Task | Status | Detail |
|---|---|---|
| VV-CFG-002.1 | ⬜ | Detect heterogeneous topology on x86 via `CPUID` leaf `0x1A` (native model ID) |
| VV-CFG-002.2 | ⬜ | Linux: cross-check `/sys/devices/system/cpu/cpuN/cpu_capacity` |
| VV-CFG-002.3 | ⬜ | Windows: `GetSystemCpuSetInformation` efficiency class |
| VV-CFG-002.4 | ⬜ | ARM big.LITTLE equivalent (Graviton is homogeneous; phones/Apple Silicon are not) |
| VV-CFG-002.5 | ⬜ | Classify each logical CPU: `Performance` / `Efficiency` / `LowPower` / `Unknown` |
| VV-CFG-002.6 | ⬜ | Identify SMT siblings — two hyperthreads on one core must never host two write shards |
| VV-CFG-002.7 | ⬜ | Test on the hybrid laptop (6P/8E/2LP-E) and on a homogeneous Xeon; both must classify correctly |
| VV-CFG-002.8 | ⬜ | Fallback: treat all cores as `Performance` when classification is unavailable — never guess wrong in the harmful direction |

### VV-CFG-003 — Storage device discovery and classification  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-CFG-003.1 | ⬜ | Enumerate block devices and their mount points |
| VV-CFG-003.2 | ⬜ | Classify: local NVMe / local SATA SSD / network-attached (EBS, iSCSI) / spinning / tmpfs |
| VV-CFG-003.3 | ⬜ | Detect ephemeral instance-store volumes — fast but **data is lost on stop**; must never be chosen for WAL without explicit consent |
| VV-CFG-003.4 | ⬜ | Measure per-device `fsync` latency with a short calibration write |
| VV-CFG-003.5 | ⬜ | Report free space and whether it can hold the projected working set |
| VV-CFG-003.6 | ⬜ | Test: classification correct on EC2 with both instance-store NVMe and EBS attached |

### VV-CFG-004 — Kernel and OS feature probe  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-CFG-004.1 | ⬜ | Kernel version parse + feature gating (io_uring ≥5.1, XDP ≥4.18) |
| VV-CFG-004.2 | ⬜ | Probe io_uring by actually creating a ring, not by version-guessing — containers block syscalls that the version implies are present |
| VV-CFG-004.3 | ⬜ | Huge page availability + current reservation + `Hugepagesize` |
| VV-CFG-004.4 | ⬜ | Current `isolcpus` / `nohz_full` state from `/proc/cmdline` |
| VV-CFG-004.5 | ⬜ | Effective privileges: root, `CAP_SYS_NICE` (pinning), `CAP_IPC_LOCK` (mlock), `SeLockMemoryPrivilege` on Windows |
| VV-CFG-004.6 | ⬜ | Detect container/cgroup limits — cgroup CPU quota makes host core count a lie |
| VV-CFG-004.7 | ⬜ | Test in a constrained Docker container: probe must report reduced capability, not host capability |

### VV-CFG-005 — Hardware fingerprint  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-CFG-005.1 | ⬜ | Derive a stable hash over the capability-relevant fields of `SystemProfile` |
| VV-CFG-005.2 | ⬜ | Exclude volatile fields (free memory, uptime, IP) so the fingerprint is stable across reboots |
| VV-CFG-005.3 | ⬜ | Embed the fingerprint in generated config as a header comment plus a machine-readable field |
| VV-CFG-005.4 | ⬜ | Embed it in every benchmark report — a latency figure whose hardware is unknown is not evidence (ties to VV-OBS-004) |
| VV-CFG-005.5 | ⬜ | Test: fingerprint stable across reboot, changes when vCPU count or RAM changes |

---

# Group 2 — Config derivation

### VV-CFG-006 — `vvctl tune`: derive configuration from profile  ⬜
**Blocked by:** VV-CFG-001, VV-ENG-017 (the config system itself)

| Task | Status | Detail |
|---|---|---|
| VV-CFG-006.1 | ⬜ | `vvctl tune` — print derived config to stdout, change nothing (the default) |
| VV-CFG-006.2 | ⬜ | `vvctl tune -o <path>` — write a config file |
| VV-CFG-006.3 | ⬜ | `vvctl tune --profile <edge\|small\|standard\|large>` — coarse operator intent override |
| VV-CFG-006.4 | ⬜ | `--reserve-fraction <f>` — what share of the box VectorVolt may claim (default well under 1.0; we are rarely the only tenant) |
| VV-CFG-006.5 | ⬜ | Refuse to overwrite an existing config without `--force`; show a diff instead |
| VV-CFG-006.6 | ⬜ | Exit non-zero if the host cannot meet the Tier D minimum, with a specific reason |

### VV-CFG-007 — Derivation: arena and memory sizing  ⬜
**Spec:** LD-ARENA (64 MB edge → 64 GB large cloud)

| Task | Status | Detail |
|---|---|---|
| VV-CFG-007.1 | ⬜ | Derive per-shard arena size from `total_ram × reserve_fraction ÷ shard_count` |
| VV-CFG-007.2 | ⬜ | Clamp to the LD-ARENA band; warn when clamping rather than silently obeying |
| VV-CFG-007.3 | ⬜ | Respect cgroup memory limits over host RAM when containerized |
| VV-CFG-007.4 | ⬜ | Leave explicit headroom for page cache and the OS — never size to 100% |
| VV-CFG-007.5 | ⬜ | Derive index (Tier 1) budget from **L3 size**, since the cuckoo filter is meant to be L3-resident |
| VV-CFG-007.6 | ⬜ | Test: 62 GB / 16-core box and 2 GB / 2-core box both yield sane, in-band values |

### VV-CFG-008 — Derivation: core role map  ⬜
**Blocked by:** VV-CFG-002 (hybrid classification), VV-HAL-024 (pinning), VV-HAL-025 (role map)
**Spec:** CPU_CACHE_NUMA_ISOLATION.md §5.1

| Task | Status | Detail |
|---|---|---|
| VV-CFG-008.1 | ⬜ | Reserve core 0 (and its SMT sibling) for the OS and IRQs — never ours |
| VV-CFG-008.2 | ⬜ | Assign **P-cores first** to write shards, then readers; E-cores to background/compaction only |
| VV-CFG-008.3 | ⬜ | Never place two write shards on SMT siblings of one physical core |
| VV-CFG-008.4 | ⬜ | Keep a shard's reader and writer on the same NUMA node; separate cores (per §5.4, avoids MESI serialization) |
| VV-CFG-008.5 | ⬜ | Scale gracefully: a 2-core box gets a single combined role, not a broken 6-group map |
| VV-CFG-008.6 | ⬜ | Emit the map as an explicit, editable list — never as an opaque algorithm the operator cannot inspect |
| VV-CFG-008.7 | ⬜ | Test: 6P/8E/2LP-E hybrid laptop → writers land only on P-cores |
| VV-CFG-008.8 | ⬜ | Test: 16-vCPU SMT Xeon → no two write shards share a physical core |
| VV-CFG-008.9 | ⬜ | Test: Graviton (no SMT) → all 16 cores usable as independent shards |

### VV-CFG-009 — Derivation: huge pages  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-CFG-009.1 | ⬜ | Compute the page count needed for index + arena from VV-CFG-007 |
| VV-CFG-009.2 | ⬜ | Emit the exact `sysctl` command rather than running it (see VV-CFG-014) |
| VV-CFG-009.3 | ⬜ | Prefer THP `madvise` when explicit reservation is unavailable |
| VV-CFG-009.4 | ⬜ | Cap the request so we never starve the host of 4 KB pages |
| VV-CFG-009.5 | ⬜ | Detect and reuse an existing sufficient reservation — idempotency |

### VV-CFG-010 — Derivation: WAL and data device selection  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-CFG-010.1 | ⬜ | Rank candidate devices by measured `fsync` latency (VV-CFG-003.4) |
| VV-CFG-010.2 | ⬜ | **Never auto-select ephemeral instance-store for WAL** — it is fastest and loses data on stop. Requires explicit `--allow-ephemeral-wal` |
| VV-CFG-010.3 | ⬜ | Prefer separate devices for WAL and partitions when available (fsync contention) |
| VV-CFG-010.4 | ⬜ | Verify writability and space before recommending |
| VV-CFG-010.5 | ⬜ | Explain the choice in a config comment, including the runner-up and why it lost |
| VV-CFG-010.6 | ⬜ | Test on EC2 with instance-store + EBS: EBS chosen by default, instance-store only with the flag |

### VV-CFG-011 — Derivation: batching, backpressure, wait strategy  ⬜
**Spec:** LD-BATCH, LD-BACK

| Task | Status | Detail |
|---|---|---|
| VV-CFG-011.1 | ⬜ | Derive group-commit window from measured `fsync` latency, not a fixed 50 µs |
| VV-CFG-011.2 | ⬜ | Derive backpressure token-bucket rate from core count and device throughput |
| VV-CFG-011.3 | ⬜ | Choose wait strategy by core budget: BusySpin only when cores are dedicated *and* isolated; Yielding otherwise; Blocking on oversubscribed/containerized hosts |
| VV-CFG-011.4 | ⬜ | Never select BusySpin on a burstable instance — it burns CPU credits for nothing |
| VV-CFG-011.5 | ⬜ | Test: derived window on slow EBS differs from fast local NVMe |

### VV-CFG-012 — Annotated TOML emission  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-CFG-012.1 | ⬜ | Emit valid TOML with a comment above every derived value stating the input and the rule |
| VV-CFG-012.2 | ⬜ | Header block: fingerprint, tool version, timestamp, detected tier (A/B/C/D) |
| VV-CFG-012.3 | ⬜ | Mark values left at default distinctly from values actively derived |
| VV-CFG-012.4 | ⬜ | Include commented-out advanced options with their trade-offs, for discoverability |
| VV-CFG-012.5 | ⬜ | Round-trip test: emitted config parses back to the same effective settings |

### VV-CFG-013 — Deployment size profiles  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-CFG-013.1 | ⬜ | Define `edge` / `small` / `standard` / `large` with concrete resource envelopes |
| VV-CFG-013.2 | ⬜ | Auto-select the profile from detected resources; allow explicit override |
| VV-CFG-013.3 | ⬜ | Document which hardware tier each profile assumes (ties to HARDWARE_SUPPORT_MATRIX.md) |
| VV-CFG-013.4 | ⬜ | Test: each profile produces a config that starts successfully |

---

# Group 3 — System tuning (privileged, opt-in)

### VV-CFG-014 — `vvctl tune --apply`: consent-gated system changes  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-CFG-014.1 | ⬜ | Dry-run is the default; `--apply` is required for any system mutation |
| VV-CFG-014.2 | ⬜ | Print every command that will run, and wait for confirmation unless `--yes` |
| VV-CFG-014.3 | ⬜ | Categorize each change: reversible-now / needs-restart / needs-reboot |
| VV-CFG-014.4 | ⬜ | Skip (never attempt) changes the current privileges cannot make; report them as manual steps |
| VV-CFG-014.5 | ⬜ | Write an audit record of what was changed, when, and by which tool version |
| VV-CFG-014.6 | ⬜ | Test: without root, `--apply` degrades to instructions and exits zero |

### VV-CFG-015 — Huge page reservation  🧊 *Linux*
### VV-CFG-016 — memlock limit adjustment  🧊 *Linux/Windows privilege*
### VV-CFG-017 — `isolcpus` / `nohz_full` guidance  🧊 *needs reboot — advise only, never auto-apply*
### VV-CFG-018 — Revert applied tuning  ⬜
### VV-CFG-019 — Idempotency  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-CFG-015.1 | 🧊 | Apply `vm.nr_hugepages` via sysctl; verify the kernel actually granted it (fragmentation can deny) |
| VV-CFG-015.2 | 🧊 | Persist to `/etc/sysctl.d/` so it survives reboot |
| VV-CFG-015.3 | 🧊 | Detect partial grant and adjust the config down to what was actually obtained |
| VV-CFG-016.1 | 🧊 | Linux: write `/etc/security/limits.d/` entry for memlock |
| VV-CFG-016.2 | 🧊 | Windows: grant `SeLockMemoryPrivilege` |
| VV-CFG-016.3 | ⬜ | Verify the limit took effect in a fresh session before trusting it |
| VV-CFG-017.1 | 🧊 | Generate the exact GRUB/kernel-cmdline edit — print it, never apply it |
| VV-CFG-017.2 | ⬜ | Explain the cost honestly: a reboot, and no benefit until thread pinning exists (VV-HAL-024) |
| VV-CFG-017.3 | ⬜ | Detect that it is already set and skip |
| VV-CFG-018.1 | ⬜ | `vvctl tune --revert` restores every recorded change |
| VV-CFG-018.2 | ⬜ | Revert must work from the audit record alone, with no live config present |
| VV-CFG-018.3 | ⬜ | Test: apply then revert leaves the system byte-identical |
| VV-CFG-019.1 | ⬜ | Second `--apply` run reports "already applied" and changes nothing |
| VV-CFG-019.2 | ⬜ | Test: apply twice, assert no duplicate sysctl.d entries and no drift |

---

# Group 4 — Re-tune and hardware drift

### VV-CFG-020 — Hardware-change detection at startup  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-CFG-020.1 | ⬜ | Compare live fingerprint against the config's recorded fingerprint on every start |
| VV-CFG-020.2 | ⬜ | On mismatch: warn loudly with the specific differences (was 16 vCPU, now 4) |
| VV-CFG-020.3 | ⬜ | Refuse to start only when the config is *unsafe* for the new hardware (core map references cores that do not exist); otherwise warn and continue |
| VV-CFG-020.4 | ⬜ | `--ignore-hardware-change` escape hatch for deliberate cases |
| VV-CFG-020.5 | ⬜ | Test: generate config on a 16-core box, run with 4 cores visible, assert the specific warning |

### VV-CFG-021 — `vvctl tune --diff`  ⬜
### VV-CFG-022 — Provenance in support bundles  ⬜
### VV-CFG-035 — Startup reconciliation: clamp-and-report  ⬜
### VV-CFG-036 — Under-utilisation detection (the silent upgrade)  ⬜

---

## Detection vs reconciliation — and why resize breaks asymmetrically

Runtime detection answers *"what does this machine have?"* It does **not** answer *"the config says
16 cores and I can see 4 — now what?"* That is **reconciliation**, and it is a separate mechanism.

**Downgrade fails loudly. Upgrade fails silently. The silent case is worse.**

| Scenario | What happens | Visibility |
|---|---|---|
| 16 → 4 cores | Pinning to core 8 returns `EINVAL`; a 32 GB arena is OOM-killed | Loud — it tells you |
| 16 → 64 cores | Nothing errors. Config still runs 8 shards on cores 4–11 | **Silent. You pay for 64 cores and get 16 cores of throughput, indefinitely** |

### Which settings self-correct, and which do not

| Setting | Downgrade 16→4 | Upgrade 16→64 | Mechanism needed |
|---|---|---|---|
| SIMD tier | self-corrects | self-corrects | runtime dispatch (built) |
| Hardware CRC-32C | self-corrects | self-corrects | runtime dispatch (built) |
| Prefetch tiers | self-corrects | self-corrects | runtime dispatch (built) |
| Cycle-timer calibration | recalibrates per start | recalibrates | already per-process |
| **Core pinning** | **`EINVAL`** | 48 cores idle | clamp + report |
| **Shard count** | oversubscribed, thrashing | 75% of box unused | clamp + nag |
| **Arena size** | **OOM kill** | under-uses RAM | clamp + report |
| **Huge pages** | reservation absent | under-reserved | re-tune |
| **BusySpin wait strategy** | burns all 4 cores | fine | clamp |
| L3-sized index budget | oversized for smaller L3 | undersized | re-derive |
| **WAL device path** | may not exist | fine | validate |

Only the first four are free. Everything else needs reconciliation, and the *upgrade* column is the
half most easily forgotten.

### The governing rule

**Configuration values are intents, clamped to reality at startup — never absolute commands.**

```
"pin to cores 4-11"   ->  cores 4-11 ∩ actually_available
"8 shards"            ->  min(8, usable_cores)
"4 GB arena/shard"    ->  min(4 GB, available_ram / shard_count)
"BusySpin"            ->  Yielding, unless cores are dedicated AND isolated
```

Every clamp is **reported**. A silently clamped config is how a team spends a week debugging a
"performance regression" that is really a VM resize from three months ago.

---

| Task | Status | Detail |
|---|---|---|
| VV-CFG-021.1 | ⬜ | Show current config vs freshly derived, value by value |
| VV-CFG-021.2 | ⬜ | Distinguish operator-edited values from stale generated ones — never propose clobbering a deliberate choice |
| VV-CFG-021.3 | ⬜ | Exit code signals whether a difference exists, for scripting |
| VV-CFG-022.1 | ⬜ | `vvctl support-bundle` includes profile, fingerprint, config, and tuning audit |
| VV-CFG-022.2 | ⬜ | Redact hostnames/IPs by default; `--include-identifiers` to keep them |
| VV-CFG-022.3 | ⬜ | Test: bundle is reproducible and contains no secrets |
| **VV-CFG-035.1** | ⬜ | Define `Reconciled<T> { intended: T, effective: T, reason: Option<ClampReason> }` so every clamp is representable and inspectable |
| VV-CFG-035.2 | ⬜ | Clamp the core role map to available cores; drop references to cores that no longer exist |
| VV-CFG-035.3 | ⬜ | Clamp shard count to usable cores — never run more write shards than cores, which guarantees context-switch thrashing |
| VV-CFG-035.4 | ⬜ | Clamp arena size to available RAM (and to the cgroup limit when containerized) before allocating, not after failing |
| VV-CFG-035.5 | ⬜ | Downgrade `BusySpin` → `Yielding` when cores are not dedicated and isolated — spinning on a shared 4-core box starves everything else |
| VV-CFG-035.6 | ⬜ | Re-derive the L3-sized index budget from the *live* L3, not the recorded one |
| VV-CFG-035.7 | ⬜ | Validate the WAL/data device paths exist and are writable; fail with a specific message naming the missing path |
| VV-CFG-035.8 | ⬜ | Log every clamp at WARN with intended value, effective value, and cause |
| VV-CFG-035.9 | ⬜ | Expose clamps as a metric (`vv_config_clamped{key}`) so monitoring catches a resize the operator forgot |
| VV-CFG-035.10 | ⬜ | Surface them in `vvctl status` — an operator must be able to ask "am I running the config I think I am?" |
| VV-CFG-035.11 | ⬜ | `--strict-config` refuses to start on any clamp, for environments that require exact provisioning |
| VV-CFG-035.12 | ⬜ | Test: config generated on 16 cores / 62 GB, started with 4 cores / 8 GB visible — starts successfully, every clamp logged, no OOM, no `EINVAL` |
| **VV-CFG-036.1** | ⬜ | At startup, compare effective config against what `tune` *would* derive for the live hardware |
| VV-CFG-036.2 | ⬜ | Compute headroom left unused: cores, RAM, and whether a wider SIMD tier is now available |
| VV-CFG-036.3 | ⬜ | Warn when unused capacity exceeds a threshold (default 25%), naming the specific resource and the concrete gain — "48 of 64 cores unused; `vvctl tune` would configure 32 write shards instead of 8" |
| VV-CFG-036.4 | ⬜ | Expose as a metric (`vv_unused_core_fraction`) so it is alertable rather than buried in a log |
| VV-CFG-036.5 | ⬜ | Include it in `vvctl status` and the admin panel (VV-CTL-011) |
| VV-CFG-036.6 | ⬜ | Never auto-expand into the new capacity. Growing shard count changes the partition→shard mapping and would silently alter data placement; that must be a deliberate operator action |
| VV-CFG-036.7 | ⬜ | Detect a *newly available* wider SIMD tier after a CPU-family change (e.g. moved to a host with AVX-512) — this one needs no config change, so state plainly that it is already in use |
| VV-CFG-036.8 | ⬜ | Test: config generated on 4 cores, started with 64 visible — warns with a specific, actionable recommendation and continues running |

| Task | Status | Detail |
|---|---|---|
| VV-CFG-021.1 | ⬜ | Show current config vs freshly derived, value by value |
| VV-CFG-021.2 | ⬜ | Distinguish operator-edited values from stale generated ones — never propose clobbering a deliberate choice |
| VV-CFG-021.3 | ⬜ | Exit code signals whether a difference exists, for scripting |
| VV-CFG-022.1 | ⬜ | `vvctl support-bundle` includes profile, fingerprint, config, and tuning audit |
| VV-CFG-022.2 | ⬜ | Redact hostnames/IPs by default; `--include-identifiers` to keep them |
| VV-CFG-022.3 | ⬜ | Test: bundle is reproducible and contains no secrets |

---

# Group 5 — Package manager integration

### VV-CFG-023 — Debian/Ubuntu `.deb` postinst  ⬜
### VV-CFG-024 — RHEL/Fedora `.rpm` `%post`  ⬜
### VV-CFG-025 — Homebrew `post_install`  ⬜
### VV-CFG-026 — Windows winget / MSI custom action  ⬜
### VV-CFG-027 — Container entrypoint  ⬜
### VV-CFG-028 — systemd unit with derived limits  ⬜
### VV-CFG-029 — Install must never fail  ⬜
### VV-CFG-030 — Uninstall cleanup  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-CFG-023.1 | ⬜ | `postinst` runs `vvctl tune -o /etc/vectorvolt/vectorvolt.toml` (derive only, no `--apply`) |
| VV-CFG-023.2 | ⬜ | Never overwrite an existing operator-edited config; write `.toml.new` and print a notice |
| VV-CFG-023.3 | ⬜ | Print the recommended privileged steps for the admin to run deliberately |
| VV-CFG-023.4 | ⬜ | `debconf` prompt for the size profile on interactive installs; safe default otherwise |
| VV-CFG-023.5 | ⬜ | Test in a clean Docker `ubuntu:24.04` |
| VV-CFG-024.1 | ⬜ | `%post` scriptlet mirroring the deb behaviour |
| VV-CFG-024.2 | ⬜ | `%config(noreplace)` on the config file so RPM upgrades preserve edits |
| VV-CFG-024.3 | ⬜ | Test in a clean `rockylinux:9` container |
| VV-CFG-025.1 | ⬜ | `post_install` writes to the Homebrew prefix, not `/etc` |
| VV-CFG-025.2 | ⬜ | Handle both Apple Silicon and Intel prefixes |
| VV-CFG-025.3 | ⬜ | Emit a `caveats` block naming the manual steps |
| VV-CFG-026.1 | ⬜ | MSI custom action invoking `vvctl tune` |
| VV-CFG-026.2 | ⬜ | Write to `%ProgramData%\VectorVolt\` |
| VV-CFG-026.3 | ⬜ | Explain the `SeLockMemoryPrivilege` grant rather than silently taking it |
| VV-CFG-026.4 | ⬜ | Test unattended install via `winget --silent` |
| VV-CFG-027.1 | ⬜ | Entrypoint runs tune on first start if no config is mounted |
| VV-CFG-027.2 | ⬜ | Respect cgroup limits, not host resources — the single most common container misconfiguration |
| VV-CFG-027.3 | ⬜ | A mounted config always wins; never regenerate over it |
| VV-CFG-027.4 | ⬜ | Test: container with `--cpus=2 --memory=1g` derives a 2-core/1 GB config, not a host-sized one |
| VV-CFG-028.1 | ⬜ | Generate a unit with `CPUAffinity`, `LimitMEMLOCK`, `LimitNOFILE` from the derived config |
| VV-CFG-028.2 | ⬜ | Ship as a drop-in override so the base unit stays package-owned |
| VV-CFG-028.3 | ⬜ | Test: `systemd-analyze verify` passes |
| VV-CFG-029.1 | ⬜ | Every postinst path exits 0 — tuning failure produces a warning, never a failed install |
| VV-CFG-029.2 | ⬜ | Fall back to Tier D safe defaults when detection fails entirely |
| VV-CFG-029.3 | ⬜ | Test: install succeeds in a container with `/proc` restricted and no root |
| VV-CFG-030.1 | ⬜ | Uninstall reverts applied system tuning (VV-CFG-018) |
| VV-CFG-030.2 | ⬜ | Preserve user data and edited config on remove; delete only on purge |
| VV-CFG-030.3 | ⬜ | Test: install → apply → uninstall leaves no sysctl.d or limits.d residue |

---

# Group 6 — Precedence and safety

### VV-CFG-031 — Configuration precedence  ⬜
### VV-CFG-032 — Validation with actionable errors  ⬜
### VV-CFG-033 — `vvctl tune --explain`  ⬜
### VV-CFG-034 — Config schema versioning  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-CFG-031.1 | ⬜ | Implement `generated < config file < environment < CLI flag` |
| VV-CFG-031.2 | ⬜ | Record each effective value's origin, so `--explain` can attribute it |
| VV-CFG-031.3 | ⬜ | Auto-tune must never silently overwrite an operator's explicit setting |
| VV-CFG-031.4 | ⬜ | Test: all four layers set the same key; CLI wins and origins are reported correctly |
| VV-CFG-032.1 | ⬜ | Validate at load: ranges, cross-field consistency, core IDs exist |
| VV-CFG-032.2 | ⬜ | Errors name the file, key, offending value, allowed range, and a suggested fix |
| VV-CFG-032.3 | ⬜ | Warn (do not fail) when a setting is legal but unachievable on this host — e.g. BusySpin without isolated cores |
| VV-CFG-032.4 | ⬜ | Reject unknown keys rather than ignoring them — a silently-ignored typo is a latent outage |
| VV-CFG-033.1 | ⬜ | `--explain` prints each value, its origin, and the derivation rule |
| VV-CFG-033.2 | ⬜ | `--explain <key>` for a single setting |
| VV-CFG-033.3 | ⬜ | Include the hardware fact that drove each derived value |
| VV-CFG-034.1 | ⬜ | `schema_version` field in generated config |
| VV-CFG-034.2 | ⬜ | Migrate older schema versions forward on load, preserving edits |
| VV-CFG-034.3 | ⬜ | Refuse a newer schema than the binary understands, with a clear upgrade message |
| VV-CFG-034.4 | ⬜ | Test: a v1 config loads in a v2 binary with edits intact |

---

---

# Group 7 — YAML as the authored format, and exhaustive configurability

**Added 2026-07-30.** The original design treated config as something *generated* by `vvctl tune` and
occasionally hand-edited, in TOML. The actual requirement inverts the emphasis: **a developer must be
able to configure the smallest behaviour by hand, in YAML.** Auto-tune supplies a sensible starting
point; hand authoring is a first-class path, not a fallback.

### VV-CFG-037 — YAML as the primary authored format  ⬜

YAML is what operators expect (Kubernetes, Compose, CI, Ansible) and is the right ergonomic choice.
It also has real hazards that must be guarded rather than wished away:

```yaml
country: NO         # YAML 1.1 parses this as boolean false, not the string "NO"
version: 1.10       # becomes the float 1.1
enabled: on         # boolean true, not the string "on"
```

| Task | Status | Detail |
|---|---|---|
| VV-CFG-037.1 | ⬜ | YAML as primary format via `serde_yaml`, pinned to YAML 1.2 semantics where the parser allows |
| VV-CFG-037.2 | ⬜ | Also accept TOML and JSON — nearly free with serde, and JSON suits machine generation |
| VV-CFG-037.3 | ⬜ | **Guard the Norway problem:** strictly type every field, and make a bare `NO`/`on`/`yes` where a string is expected a load error that names the fix |
| VV-CFG-037.4 | ⬜ | Reject duplicate keys — YAML permits them and most parsers silently keep the last |
| VV-CFG-037.5 | ⬜ | Reject tab indentation with a clear message rather than a parser-internal error |
| VV-CFG-037.6 | ⬜ | `vvctl config validate` reports line and column for every problem |
| VV-CFG-037.7 | ⬜ | `vvctl tune` emits YAML by default; `--format toml\|json` for the rest |
| VV-CFG-037.8 | ⬜ | Round-trip test: the same config in YAML, TOML, and JSON yields identical effective settings |

### VV-CFG-038 — Exhaustive configurability + generated config reference  ⬜

The principle: **every tunable behaviour is reachable from the config file.** No behaviour adjustable
only by recompiling; no constant an operator could plausibly need to change left hardcoded.

| Task | Status | Detail |
|---|---|---|
| VV-CFG-038.1 | ⬜ | Audit every hardcoded constant in the workspace; each is promoted to config or documented as deliberately fixed, with the reason |
| VV-CFG-038.2 | ⬜ | Derive the config reference **from the Rust types themselves**, so documentation cannot drift from code |
| VV-CFG-038.3 | ⬜ | Every key documents type, default, valid range, the hardware fact that derives it, and what changing it costs |
| VV-CFG-038.4 | ⬜ | `vvctl config schema` emits JSON Schema — editor autocomplete and CI validation of config files |
| VV-CFG-038.5 | ⬜ | Group keys by subsystem, mirroring the layer structure, so the file is navigable |
| VV-CFG-038.6 | ⬜ | Mark each key `stable` / `advanced` / `unstable` — an operator must know which knobs are load-bearing |
| VV-CFG-038.7 | ⬜ | CI check: a new `pub` config field without documentation fails the build |

### VV-CFG-039 — Layered config files (base + environment overlay)  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-CFG-039.1 | ⬜ | Accept an ordered file list, later overlaying earlier — `base.yaml` + `prod.yaml` |
| VV-CFG-039.2 | ⬜ | Deep-merge maps; **replace, not append, for lists** — silently appending to a core-pinning list would be a nasty surprise |
| VV-CFG-039.3 | ⬜ | `vvctl config effective` prints the merged result with each value's originating file and line |
| VV-CFG-039.4 | ⬜ | `!clear` directive to explicitly empty an inherited list |
| VV-CFG-039.5 | ⬜ | Include-directory support (`conf.d/*.yaml`) in deterministic lexical order |
| VV-CFG-039.6 | ⬜ | Test: three-layer merge with per-key origin correctly attributed |

---

## What this does NOT do

Stated so scope stays honest:

- **It does not remove the need for a per-architecture build.** `x86_64` and `aarch64` are different
  binaries; no amount of install-time cleverness changes that. The installer selects the right
  package for the platform — which package managers already do.
- **It does not enable DPDK/RDMA on a host that lacks them.** Those are linked libraries
  (build-time). Auto-config can *detect* the hardware and tell the operator a different package
  variant would help, but cannot conjure the linkage.
- **It does not auto-apply anything requiring a reboot.** `isolcpus` is advised, never applied.
- **It is not adaptive self-tuning at runtime.** That is LD-ADAPT (closed-loop adjustment of batch
  window and prefetch budget while running) and remains separate. This layer runs at install and
  on demand, producing a static, inspectable config.
