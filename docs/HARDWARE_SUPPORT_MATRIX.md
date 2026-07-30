# VectorVolt — Hardware Support Matrix

**Written:** 2026-07-30. Revise whenever a tier's hardware or a feature's gating changes.

Four tiers, best to worst. My reading of the categories — correct me if you meant them differently:

| Tier | Name | Meaning |
|---|---|---|
| **A** | **Best possible** | Reference platform. Where published numbers come from. Every feature on. |
| **B** | **Must have** | Supported production baseline. We *commit* to the latency targets here. |
| **C** | **Should have** | Runs well, some acceleration off. No published guarantees. |
| **D** | **Can manage** | Correct but slow. Supported, never recommended. |

The governing rule, which applies to every row of every table below:

> **Missing hardware costs speed, never correctness.** Every SIMD kernel has a scalar twin that
> is the tested oracle. No AVX-512 → AVX2. No AVX2 → scalar. No huge pages → 4 KB pages. No NUMA
> → one arena. Nothing silently produces different *answers* on different silicon; the equivalence
> tests exist to prove exactly that.

---

## The four tiers

### Tier A — Best possible (reference / benchmark platform)

| | |
|---|---|
| **CPU** | 2-socket Intel Xeon Scalable (Sapphire Rapids+) or AMD EPYC Genoa, **bare metal** |
| **AWS** | `m7i.metal-48xl`, `c7i.metal-24xl`, `m6a.metal`, or `c7g.metal` (ARM) |
| **Why metal** | No hypervisor steal time, real PMU access, `isolcpus` fully effective, huge pages guaranteed |
| **Why 2-socket** | The only way to exercise NUMA at all. Single-socket boxes make VV-HAL-028/030 untestable |
| **Storage** | Local instance-store NVMe (not EBS) — `fsync` latency is the WAL's floor |
| **Network** | EFA / RDMA-capable for replication; ENA + SR-IOV for DPDK |
| **Everything on** | AVX-512, huge pages, NUMA binding, core pinning, `isolcpus`, io_uring, XDP |

**Rent for hours, don't own.** Nothing in normal development needs this. It's for: publishing
latency figures, validating NUMA, and DPDK/RDMA work.

### Tier B — Must have (supported production baseline)

| | |
|---|---|
| **CPU** | x86-64 with AVX2 + SSE4.2 + invariant TSC, ≥8 real cores. **Or** Graviton3+ |
| **AWS** | `c6i/m6i/r6i.4xlarge`+, `c7g/m7g.4xlarge`+ (ARM), `c5.4xlarge` |
| **Your EC2 box** | `g4dn.4xlarge` (Xeon Platinum 8259CL) sits here — *and* has AVX-512, so it reaches into A for SIMD |
| **Storage** | NVMe, local instance-store strongly preferred |
| **RAM** | ≥32 GB |
| **On** | AVX2, prefetch, hw CRC-32C, core pinning, huge pages (with root), io_uring, XDP generic |
| **Off / degraded** | NUMA if single-socket · DPDK unless ENA+SR-IOV · RDMA unless EFA |

**This is the tier the performance targets are written against.** `<100 ns` reads, `p99 <10 µs`
memory-path writes — those are Tier B commitments.

### Tier C — Should have (works well, less acceleration)

| | |
|---|---|
| **CPU** | x86-64 with AVX2, 4+ cores, any clock. Or Graviton2 (`c6g`) |
| **Examples** | `c5.xlarge`, `t3.xlarge` (burstable — see caveat), most laptops 2013+, **your Core Ultra 7 165H** |
| **On** | AVX2, prefetch, hw CRC, io_uring (Linux) |
| **Off** | AVX-512, NUMA, DPDK, RDMA. `isolcpus` impractical. Huge pages need privilege |
| **Caveat** | **Burstable (`t2`/`t3`/`t4g`) instances produce untrustworthy latency figures.** CPU-credit throttling and steal time swamp a 100 ns signal. Fine for correctness, never for benchmarks |

### Tier D — Can manage (correct, slow, unsupported for perf)

| | |
|---|---|
| **CPU** | Any x86-64 without AVX2 (pre-2013), or ARM without our NEON kernels, or anything else |
| **Examples** | Sandy/Ivy Bridge, Raspberry Pi, RISC-V, 32-bit |
| **On** | **Everything, functionally.** Scalar kernels, 4 KB pages, no pinning, buffered I/O |
| **Off** | All silicon acceleration |
| **Cost** | Expect 2–3× slower scans, and reads in the microseconds rather than nanoseconds |

We support this because a database that refuses to start is worse than a slow one — and because
the scalar path is our correctness oracle, so it must stay first-class regardless.

---

## Your laptop: Intel Core Ultra 7 165H — Tier C, with one serious asterisk

```
Intel Core Ultra 7 165H (Meteor Lake)  ·  16 cores / 22 threads  ·  3.8 GHz max
L2 18 MB  ·  L3 24 MB  ·  32 GB DDR5-5600  ·  NVMe SSD  ·  Windows 11
```

**Excellent for development. Not trustworthy for latency numbers.** Two independent reasons:

### 1. No AVX-512 — and it's not coming

Intel **fused AVX-512 off** on every consumer hybrid chip from Alder Lake onward, Meteor Lake
included. The reason is architectural, not a marketing choice: the E-cores don't implement AVX-512,
and an OS scheduler cannot safely migrate a thread mid-execution between cores with different
instruction sets. So it's disabled everywhere.

Consequence: **AVX-512 kernels (VV-HAL-007) can never be tested on your laptop.** That work needs
your `g4dn` Xeon, which does have it. Not a problem — just a permanent division of labour.

### 2. Hybrid P-core / E-core — this is the real problem for benchmarking

Your 16 cores are not 16 equivalent cores:

| Core type | Count | Threads | Character |
|---|---|---|---|
| **P-core** (Redwood Cove) | 6 | 12 (SMT) | High clock, full-width AVX2 |
| **E-core** (Crestmont) | 8 | 8 | Lower clock, narrower, no SMT |
| **LP E-core** (on SoC tile) | 2 | 2 | Very low power, slowest |

Why that hurts us specifically:

- **Thread migration between a P-core and an E-core can swing latency ~2×** mid-measurement. A
  100 ns operation that drifts onto an E-core is no longer 100 ns, and nothing in the number tells
  you it moved.
- **Core pinning becomes a trap.** "Pin the writer to core 14" may land on an E-core running at
  half speed. Our core role map (VV-HAL-025) must become **P-core aware** — pinning latency-critical
  writers to E-cores would be actively harmful. This is a design requirement your laptop surfaced
  that a homogeneous server would have hidden until much later.
- **Laptop thermal and power management** adds its own variance on top: turbo budget, package power
  limits, and on battery, aggressive downclocking.

**How to use it well:** development, correctness, the full test suite, relative A/B comparisons
inside one run, and `VV_SIMD_TIER` tier comparisons. **Don't** publish absolute latency from it.

Note the flip side: your **24 MB L3** is large for a laptop. My earlier "8 MB working set is
DRAM-bound" claim was measured here and is **wrong as a general statement** — 8 MB fits inside this
L3. The `g4dn`'s L3 is 35.8 MB, larger still. Cache thresholds must be derived per machine, never
carried between them.

---

## Feature × tier compatibility

✅ full · 🟨 works, reduced · ⬜ off, graceful fallback · ❌ cannot run

| Feature | A metal 2-socket | B server | C laptop/small | D old/exotic |
|---|---|---|---|---|
| Scalar kernels (oracle) | ✅ | ✅ | ✅ | ✅ |
| **AVX2 kernels** | ✅ | ✅ | ✅ | ⬜ → scalar |
| **AVX-512 kernels** | ✅ | 🟨 *downclocks on Cascade Lake* | ⬜ **fused off on Meteor Lake** | ⬜ |
| NEON / SVE (ARM) | ✅ Graviton3+ | ✅ Graviton | ⬜ | ⬜ |
| L1/L2/L3/NT prefetch | ✅ | ✅ | ✅ | ⬜ no-op |
| Hardware CRC-32C | ✅ | ✅ | ✅ | ⬜ → bitwise |
| Cache-line alignment | ✅ | ✅ | ✅ | ✅ |
| RDTSC cycle timer | ✅ | ✅ | ✅ | 🟨 coarser |
| **Core pinning** | ✅ | ✅ | 🟨 **must avoid E-cores** | ⬜ unpinned |
| **NUMA node binding** | ✅ | ❌ *single socket* | ❌ | ❌ |
| 2 MB huge pages | ✅ | ✅ root | 🟨 Win needs privilege | ⬜ 4 KB |
| `mlock` tier-1 pinning | ✅ | ✅ | 🟨 limit-bound | ⬜ |
| `isolcpus` / `nohz_full` | ✅ | ✅ root+reboot | ❌ impractical | ❌ |
| io_uring | ✅ | ✅ Linux ≥5.1 | ⬜ **not on Windows** | ⬜ |
| `O_DIRECT` | ✅ | ✅ Linux | ⬜ | ⬜ |
| eBPF / AF_XDP | ✅ native | 🟨 generic mode on `ena` | ❌ | ❌ |
| DPDK | ✅ | 🟨 needs ENA+SR-IOV | ❌ | ❌ |
| RDMA | ✅ EFA | ❌ | ❌ | ❌ |
| Intel QAT / DSA | 🟨 specific SKUs | ❌ | ❌ | ❌ |
| WAL, index, storage, query, txn | ✅ | ✅ | ✅ | ✅ |
| Clustering / replication | ✅ | ✅ | 🟨 single-host sim only | 🟨 |

Note the last two rows: **everything that makes VectorVolt a database works on every tier.** The
gated items are all accelerators.

---

## The degradation ladder

Each rung falls back to the next automatically, at runtime, with no rebuild:

```
SIMD        AVX-512 → AVX2 → NEON → scalar
Checksum    hw crc32 → bitwise CRC-32C
Pages       2 MB huge → transparent huge → 4 KB
Placement   NUMA-bound arena → single arena
Affinity    pinned to P-core → pinned to any core → unpinned
File I/O    io_uring + O_DIRECT → mmap + buffered write
Network     DPDK → AF_XDP → busy-poll TCP → plain TCP
Prefetch    PREFETCHT0/T1/T2/NTA → PRFM → no-op
Timer       RDTSC → CNTVCT_EL0 → monotonic clock
```

Two guarantees on that ladder:

1. **The chosen rung is always reported.** `vvctl hal` and `describe()` print the effective
   configuration, so no latency figure is ever detached from the silicon that produced it.
2. **You can force a lower rung.** `VV_SIMD_TIER=scalar` runs the scalar path on AVX2 hardware. Two
   reasons this matters: the fallback path would otherwise never execute on modern CPUs and would
   rot untested; and comparing tiers across *different machines* confounds the kernel with that
   machine's cache, clock, and bandwidth. Same box, one variable.

---

## When a separate build is genuinely required

This is the part most projects get wrong by over-building. **Runtime dispatch covers far more than
people assume.**

### One build handles all of this — no rebuild, no flags

AVX2 · AVX-512 · NEON · scalar · every prefetch tier · hardware CRC · core pinning · NUMA
binding · huge pages · `mlock` · cache alignment · cycle timer

All of it is *instructions* or *syscalls*, both detectable at startup. A single binary built on
your laptop runs optimally on the Xeon and correctly on a 2011 machine.

### Separate build required — 3 cases only

**1. Different architecture** (unavoidable — this is cross-compilation)

```
x86_64-unknown-linux-gnu     Intel/AMD EC2
aarch64-unknown-linux-gnu    Graviton
x86_64-pc-windows-msvc       your laptop
x86_64-unknown-linux-musl    static, for containers
```

Already handled: the release CI builds 5 targets.

**2. Features that link an external library** — you cannot runtime-detect a library you didn't link

| Cargo feature | Links | Why not runtime |
|---|---|---|
| `dpdk` | `librte_*` | Takes exclusive ownership of the NIC; needs root + hugetlbfs + vfio |
| `rdma` | `libibverbs` | Hard dependency on RDMA hardware present at link time |
| `qat` | Intel QAT driver | Kernel driver + userspace lib |
| `io-uring` | `liburing` (optional) | Pure-syscall implementation may avoid this |

Deployment shape: one default build for everyone; `--features dpdk` for the handful of hosts with
the hardware. The `Transport` trait (VV-NET-013) is designed so the *portable* build has a complete
TCP path and DPDK slots in behind the same interface.

**3. `target-cpu` tuning — deliberately NOT used**

`-C target-cpu=native` would let the compiler auto-vectorize for the build machine. We **don't**,
because it produces a binary that crashes with `SIGILL` on any older CPU. Runtime dispatch gets us
the same wins while staying portable. This is a design decision, not an oversight.

---

## x86-64 vs ARM (Graviton) on AWS

You asked which benefits us. The honest answer is **probably ARM, but for a reason that has nothing
to do with price** — and there's a correctness dimension worth more than the performance one.

### The differences that actually matter to us

**1. No SMT on Graviton — 1 vCPU = 1 real core. This is the big one.**

On Intel/AMD, a "vCPU" is one hyperthread — *half* a physical core. Two hyperthread siblings share
L1 and L2. For a thread-per-core database with cache-resident indexes, that means two shards
fighting over the same L1 and evicting each other's hot data. It is the exact opposite of what our
design wants.

Graviton has no SMT. 16 vCPUs = 16 real cores, each with its own L1/L2. Our core-pinning and
shard-per-core model (VV-HAL-024/027, VV-ENG-016) becomes clean and predictable instead of a
sibling-contention puzzle. **For VectorVolt's architecture specifically, this is worth more than
any instruction-set advantage.**

**2. Memory bandwidth — and we already know we're bandwidth-bound**

The S1a measurements showed kernels going DRAM-bandwidth-bound past L3, not compute-bound.
Graviton3/4 have DDR5 with high per-vCPU bandwidth. Since bandwidth is our measured ceiling,
bandwidth is what buys us speed — more than wider vectors would.

**3. SIMD width — x86 wins on paper**

| | Width | `u64` lanes |
|---|---|---|
| AVX-512 (Xeon SP, your `g4dn`) | 512-bit | 8 |
| AVX2 | 256-bit | 4 |
| SVE (Graviton3+) | 256-bit | 4 |
| NEON (all ARM) | 128-bit | **2** |

Our NEON kernels would get *half* the lanes of AVX2. SVE on Graviton3+ closes that back to parity.
So: write NEON for correctness and portability, SVE for Graviton performance.

**4. ⚠️ Weaker memory model — this is a correctness issue, not a performance one**

x86 is **TSO** (total store order): the hardware won't reorder stores relative to other stores.
ARM's model is weaker and permits reordering x86 forbids.

The consequence is sharp: **lock-free code with subtly wrong memory orderings works on x86 and
breaks on ARM.** If a Rust `Ordering::Relaxed` should have been `Acquire`/`Release`, x86 usually
hides the bug; ARM exposes it as rare, load-bearing, near-undebuggable corruption.

We are about to write exactly the code this bites: the **LMAX Disruptor ring buffer**
(VV-TXN-008), MVCC version chains (VV-TXN-010), and epoch reclamation (VV-TXN-013).

**So testing on ARM is a correctness test, and it's worth doing early — before we're deep into
lock-free code, not after.** A cheap `c7g.large` for a few hours running the test suite would be
one of the highest-value uses of your ephemeral-box plan.

**5. ⚠️ Timer resolution — must verify before trusting ARM numbers**

x86 `RDTSC` ticks near core clock (~3 GHz → ~0.3 ns resolution). ARM's `CNTVCT_EL0` ticks at a
*fixed, typically much lower* frequency set by `CNTFRQ_EL0`. If that's tens of MHz, resolution is
tens of nanoseconds — **useless for measuring a 100 ns operation.**

I don't know Graviton's actual `CNTFRQ_EL0` value and won't guess. **Action:** read it first thing
on any ARM box. If it's too coarse we need the PMU cycle counter (`PMCCNTR_EL0`), which needs a
privileged enable. This is a real gap in VV-HAL-037 on ARM and it's flagged, not solved.

**6. Price/performance — real, but the least interesting reason**

Graviton is generally 20–40% better price/perf for equivalent work. True and useful. But it's the
weakest of the arguments above.

### Recommendation

| Purpose | Choose | Why |
|---|---|---|
| **Development** (now) | x86-64 | What you have; AVX-512 available on the `g4dn` for VV-HAL-007 |
| **Correctness validation** (soon, cheap) | **Graviton — `c7g.large`, a few hours** | Memory-model bugs in lock-free code surface *only* here. Do this before writing the ring buffer |
| **Production target** (later) | **Graviton3+, likely** | No SMT, better bandwidth, better price/perf — all three favour our architecture |
| **Published benchmarks** | Bare metal, both | Report both; x86 wins peak vector throughput, ARM likely wins per-core consistency |

Both are cheap to support: a target-triple flag in CI (already building 5 targets) plus NEON/SVE
kernels behind the existing dispatch. **Nothing about supporting both requires an architectural
compromise** — the runtime-dispatch design was chosen partly to make this a non-event.

---

## Immediate actions this analysis produces

| # | Action | Needs | Value |
|---|---|---|---|
| 1 | AVX-512 kernels (VV-HAL-007) + measure the Cascade Lake downclock | Your `g4dn` | Answers whether AVX-512 is a net win at all — a real possible "no" |
| 2 | **Run the test suite on `c7g.large` for one hour** | New ARM box | Catches memory-ordering bugs *before* the lock-free work, not after |
| 3 | Read `CNTFRQ_EL0` on ARM | Same box | Determines whether our cycle timer is even usable on ARM |
| 4 | Make the core role map P-core aware (VV-HAL-025) | Your laptop found this | Prevents pinning writers to E-cores |
| 5 | Re-derive cache thresholds per machine | Both boxes | The "8 MB is DRAM-bound" figure was laptop-specific and does not generalise |
| 6 | NUMA work | 2-socket metal, hours | Currently untestable — both available boxes are single-socket |
