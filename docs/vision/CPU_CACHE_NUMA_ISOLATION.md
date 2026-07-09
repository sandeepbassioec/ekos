# CPU Cache, NUMA, and Core Isolation
## VectorVolt — Silicon-Layer Architecture for Nanosecond Latency

**Status:** Architecture Reference (Phase 1 foundations, applies to all phases)  
**Captured:** 2026-07-09

Every nanosecond saved at this layer compounds. A 10 ns improvement per transaction at
4 million TPS saves 40 ms of CPU time per second — enough to free an entire core.

---

## 1. The CPU Memory Hierarchy

Modern CPUs have a 5-tier memory hierarchy. VectorVolt's entire architecture is designed
around maximizing the fraction of work that happens in the top 3 tiers:

```
Tier      Size (per core)     Latency        Bandwidth        VectorVolt usage
─────────────────────────────────────────────────────────────────────────────────
L1 Cache  32–64 KB            ~1 ns          ~1 TB/s          Ring buffer slots, TxnFrame
L2 Cache  256 KB – 1 MB       ~4 ns          ~200 GB/s        Index hot entries, row descriptors
L3 Cache  8–512 MB (shared)   ~10–40 ns      ~100 GB/s        Cuckoo Hash index, Cuckoo Filter
DRAM      GBs – TBs           ~60–100 ns     ~50 GB/s         Micro-Partitions (hot data)
NVMe SSD  TBs                 ~10–50 µs      ~7 GB/s          Micro-Partitions (cold data)
─────────────────────────────────────────────────────────────────────────────────
```

The goal: **point lookups never leave L3. Scans never leave DRAM.**

---

## 2. Cache Line Fundamentals

A CPU cache line is **64 bytes** on x86-64 and ARM. The cache operates at cache-line
granularity — when the CPU reads one byte, it fetches the entire 64-byte line.

### 2.1 False Sharing

The most common accidental cache performance bug: two threads update different fields
that happen to share the same 64-byte cache line.

```
Without VectorVolt's alignment policy:

struct Counter {
    reads:  u64,  // offset 0
    writes: u64,  // offset 8
}
// Both fields fit in one 64-byte cache line.
// Reader thread updates `reads` → marks cache line dirty.
// Writer thread updates `writes` → must fetch the "dirty" line from the reader's cache.
// This triggers a cache coherence protocol (MESI: Modified → Invalid → Modified).
// Cost: ~200–300 ns per contested write. On the hot path, this is catastrophic.
```

### 2.2 VectorVolt's Solution: Explicit Cache-Line Padding

Every struct that is updated by more than one thread gets its own cache line:

```rust
#[repr(C, align(64))]
pub struct ShardMetrics {
    pub write_count: AtomicU64,     // 8 bytes
    pub read_count:  AtomicU64,     // 8 bytes
    pub error_count: AtomicU64,     // 8 bytes
    pub latency_sum: AtomicU64,     // 8 bytes
    _padding:        [u8; 32],      // pad to 64 bytes — one cache line, no sharing
}
```

The compiler will error if the struct exceeds 64 bytes — the padding is sized exactly.
This is enforced at compile time. False sharing cannot happen silently.

### 2.3 Cache Line Bouncing

Cache line bouncing is false sharing at scale: a cache line ping-pongs between two
CPU caches continuously because both keep writing to it.

```
Writer Core (Core 4):  ring buffer sequence counter → cache line state: Modified
Reader Core (Core 8):  ring buffer sequence counter → must fetch from Core 4's cache → Invalid
Writer Core:           next write → must re-acquire from Core 8's cache → Modified again

Round-trip: Core 4 ↔ Core 8 cache coherence = ~200 ns per bounce
At 4M TPS: 0.8 seconds of CPU time wasted per second on bouncing alone
```

VectorVolt's solution: the ring buffer sequence counter is written only by the writer
thread, and read (never written) by readers. One-directional access = no bounce.
The Rust ownership model makes this a compile-time constraint: the writer owns
`&mut AtomicU64`, readers hold `&AtomicU64`. No accidental write from a reader.

---

## 3. L1/L2 Cache Optimization

### 3.1 What Lives in L1

L1 is 32–64 KB. It holds the hottest data the current core touches in the current
instruction window. VectorVolt reserves L1 for:

- The current `CommandFrame` being processed (64 bytes — exactly one cache line)
- The ring buffer's current read/write position (8 bytes)
- The current MVCC version chain pointer (8 bytes)
- Loop variables and SIMD register state during scans

VectorVolt does **not** try to keep index structures in L1 — they are too large.
L1 is reserved for the per-instruction state of the transaction in flight.

### 3.2 What Lives in L2

L2 is 256 KB – 1 MB. It holds data that is hot within the current function call
but not needed across function boundaries. VectorVolt reserves L2 for:

- The current transaction's Delta Log entry (~512 bytes)
- The current Micro-Partition's Zone Map entry (~128 bytes per partition being scanned)
- The SIMD working buffer during a single Roaring Bitmap pass (~4–64 KB)
- The WAL GroupCommit batch buffer (~16–64 KB)

### 3.3 What Lives in L3

L3 is 8–512 MB (shared across all cores on the socket). VectorVolt pins the following
structures into L3 via `mlock()` and `MAP_POPULATE`:

- **Cuckoo Hash primary key index** (~9 bytes per row): for 1 billion rows, ~9 GB.
  This exceeds typical L3. For smaller datasets (≤100M rows), the entire index fits
  in L3 — point lookups stay at ~10–40 ns instead of going to DRAM (~60–100 ns).
- **Cuckoo Filter** (existence predicate, ~1 bit per row): for 1B rows, ~128 MB.
  Fits in L3 on modern Xeon (300+ MB L3). FK validation in ~10 ns.
- **Zone Map index** (min/max per partition, ~32 bytes per partition): typically small
  enough to fit fully in L3.

The pre-fetch engine (Phase 4) maintains a **Tier 1 budget**: the maximum number of
rows whose PKs are `mlock`'d into L3. The budget is tuned against the server's
actual L3 size.

### 3.4 Hardware Prefetch Hints

CPUs have automatic hardware prefetchers that detect sequential access patterns and
prefetch ahead. VectorVolt's Micro-Partition layout exploits this:

- **Struct-of-Arrays (SoA) column layout**: all values for column A are contiguous.
  When the CPU scans column A, the hardware prefetcher detects the stride and
  prefetches the next 4–8 cache lines automatically — effectively hiding DRAM latency.
- **CSR graph adjacency arrays**: neighbor lists are stored sequentially. Graph
  traversal is a sequential scan — the hardware prefetcher keeps up.

VectorVolt also uses explicit software prefetch hints for non-sequential patterns:

```rust
// Prefetch the next batch of Micro-Partition data while processing current batch
unsafe {
    std::arch::x86_64::_mm_prefetch(
        next_partition_ptr as *const i8,
        std::arch::x86_64::_MM_HINT_T0,  // prefetch into L1
    );
}
```

---

## 4. NUMA Architecture

### 4.1 NUMA Topology

A 2-socket server with 64 cores each looks like this to VectorVolt:

```
Socket 0 (NUMA Node 0)          Socket 1 (NUMA Node 1)
───────────────────────         ───────────────────────
Cores 0–63                      Cores 64–127
L3 Cache: 256 MB (shared)       L3 Cache: 256 MB (shared)
Local DRAM: 512 GB              Local DRAM: 512 GB
        │                               │
        └──────── QPI/UPI link ─────────┘
                 (~100 ns cross-socket)
```

A CPU on Socket 0 reading from Socket 1's DRAM pays ~160–200 ns (local: ~60 ns).
The 100 ns penalty per cross-socket access destroys hot-path latency.

### 4.2 NUMA Detection

At startup, VectorVolt reads the NUMA topology:

```rust
pub fn detect_numa_topology() -> Vec<NumaNode> {
    // Read /sys/devices/system/node/nodeN/cpulist
    // Read /sys/devices/system/node/nodeN/meminfo for memory size
    let node_count = std::fs::read_dir("/sys/devices/system/node")
        .unwrap()
        .filter(|e| e.as_ref().unwrap().file_name().to_str().unwrap().starts_with("node"))
        .count();
    // Build: Vec<NumaNode { id, cpus: Vec<usize>, memory_bytes: u64 }>
}
```

### 4.3 NUMA-Aware Memory Allocation

VectorVolt binds every memory allocation to its target NUMA node using `mbind()`:

```rust
pub struct NumaAllocator {
    node_id: usize,
}

impl NumaAllocator {
    pub fn alloc(&self, size: usize) -> *mut u8 {
        let ptr = unsafe {
            libc::mmap(null_mut(), size,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS | MAP_POPULATE,
                -1, 0)
        };
        // Bind this memory region to the target NUMA node
        let nodemask: u64 = 1 << self.node_id;
        unsafe {
            libc::mbind(ptr, size, MPOL_BIND,
                &nodemask as *const u64,
                64,  // maxnode
                0);
        }
        ptr as *mut u8
    }
}
```

**Per-socket shard assignment:**

```
Shard 0, 1, 2 → NUMA Node 0 (Cores 0–63, DRAM 0)
Shard 3, 4, 5 → NUMA Node 1 (Cores 64–127, DRAM 1)

Writer for Shard 0: pinned to Core 4 (Node 0) → writes to Node 0 DRAM
Reader for Shard 0: pinned to Core 8 (Node 0) → reads from Node 0 DRAM
No cross-socket memory traffic on the hot path.
```

### 4.4 jemalloc NUMA Arenas

For allocations that go through jemalloc (the Rust default allocator on Linux in production builds),
VectorVolt configures per-NUMA-node arenas:

```rust
// In Cargo.toml: jemallocator = "0.5"
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

// At startup, configure NUMA arenas:
// MALLOC_CONF="narenas:2,percpu_arena:percpu"
// This creates one arena per NUMA node, each arena allocates only from its node's memory.
```

With NUMA arenas, even incidental heap allocations (error messages, log strings) don't
cross NUMA node boundaries.

---

## 5. Core Isolation Model

### 5.1 The Full Core Map

For a 48-core server (2 NUMA nodes, 24 cores each):

```
NUMA Node 0 (Socket 0, Cores 0–23):
  Core 0:      OS + IRQ handling (never touched by VectorVolt — isolated via isolcpus=1-23)
  Core 1:      gRPC receive / command parsing (network hot path)
  Core 2:      gRPC receive / command parsing (second stream)
  Cores 3–5:   OLTP Writers for Shards 0, 1, 2 (one writer per shard)
  Cores 6–11:  OLTP Readers — SIMD scan workers (Node 0 shards)
  Cores 12–15: In-process LLM / embedding inference (dedicated, NEVER evicted)
  Cores 16–17: Background (pre-fetch Tier 2 loading, WAL shipping, compaction)
  Cores 18–19: ARP / behavioral fingerprinting
  Cores 20–21: Admin panel, metrics, health probe
  Cores 22–23: Overflow / burst capacity

NUMA Node 1 (Socket 1, Cores 24–47):
  Core 24:     OS + IRQ handling (isolated)
  Core 25:     gRPC receive (Node 1 streams)
  Cores 26–28: OLTP Writers for Shards 3, 4, 5
  Cores 29–34: OLTP Readers — SIMD scan workers (Node 1 shards)
  Cores 35–38: In-process LLM (second instance or shared via message passing)
  Cores 39–40: Background (Node 1 pre-fetch, backup)
  Cores 41–47: Available for multi-tenancy expansion
```

### 5.2 Kernel CPU Isolation

To prevent the OS scheduler from moving VectorVolt threads off their pinned cores,
VectorVolt requires the server to boot with `isolcpus` in the kernel command line:

```
# /etc/default/grub:
GRUB_CMDLINE_LINUX="isolcpus=1-23,25-47 nohz_full=1-23,25-47 rcu_nocbs=1-23,25-47"
```

`isolcpus`: removes these CPUs from the OS scheduler's pool — no OS threads land here.  
`nohz_full`: disables the scheduler tick on these cores — no periodic interrupts.  
`rcu_nocbs`: offloads RCU (Read-Copy-Update) callbacks to other cores.

Result: VectorVolt's cores receive **zero OS interrupts** during transaction processing.
Jitter from scheduler preemption drops from ~10–100 µs to ~0.

### 5.3 CPU Pinning Implementation

```rust
pub fn pin_current_thread_to_core(core_id: usize) -> Result<(), std::io::Error> {
    let mut cpu_set = unsafe { std::mem::zeroed::<libc::cpu_set_t>() };
    unsafe { libc::CPU_SET(core_id, &mut cpu_set) };
    let result = unsafe {
        libc::sched_setaffinity(
            0,  // 0 = current thread
            std::mem::size_of_val(&cpu_set),
            &cpu_set,
        )
    };
    if result == -1 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}
```

This is called at thread creation for every VectorVolt worker thread. The core assignment
is derived from the detected `CpuTopology` and the shard assignment table.

### 5.4 Writer / Reader Isolation Invariant

The most critical isolation: **OLTP writers never share a core with OLTP readers.**

```
Why: A writer updating the Cuckoo Hash index marks those cache lines as Modified.
     If a reader on the same core tries to read those lines simultaneously, the
     CPU must serialize access — destroying the zero-contention design.

VectorVolt enforces: each shard's writer runs on a dedicated core.
     Readers for the same shard run on different cores (any core in the reader pool).
     The Cuckoo Hash index is accessed by readers as Read-Only (MESI Shared state).
     Writers update it as Modified, but only after the reader window has passed
     (MVCC guarantees readers see a consistent snapshot from before the write).
```

---

## 6. SIMD Architecture: AVX2 Baseline, AVX-512 Opportunistic

### 6.1 Why AVX2 is the Baseline

AVX2 (256-bit SIMD) is supported by:
- Intel: Haswell (2013) and all successors
- AMD: Ryzen (2017) and all successors, all EPYC
- AWS Graviton: N/A (ARM uses NEON/SVE, not AVX)

AVX-512 (512-bit SIMD) is supported by:
- Intel: Xeon Ice Lake (2021), Xeon Sapphire Rapids (2023)
- AMD: Zen 4 (EPYC Genoa, 2022) — partial AVX-512
- NOT available on: AMD Rome/Milan EPYC (common cloud workhorses), Intel consumer CPUs

VectorVolt targets AVX2 as the minimum. AVX-512 provides 2x throughput on the same
operation and is used when available:

```rust
// Roaring Bitmap SIMD scan — automatically selects best available
pub fn popcount_simd(words: &[u64]) -> usize {
    #[cfg(target_feature = "avx512vpopcntdq")]
    unsafe { return avx512_popcount(words); }   // 8 u64 per cycle

    #[cfg(target_feature = "avx2")]
    unsafe { return avx2_popcount(words); }      // 4 u64 per cycle

    return scalar_popcount(words);               // 1 u64 per cycle
}
```

At runtime, `cpuid` detects which features are available. The binary compiles all
variants; the right one is selected at startup.

### 6.2 ARM NEON / SVE

On ARM (AWS Graviton5, Google Axion), NEON is the AVX2 equivalent (128-bit).
SVE (Scalable Vector Extension) is the ARM equivalent of AVX-512 with variable
vector width (128–2048 bits).

Rust's standard library and `packed_simd` crate emit NEON/SVE intrinsics automatically
when targeting `aarch64`. The same abstract scan functions work on x86 and ARM without
architecture-specific code.

### 6.3 SIMD Applications in VectorVolt

| Operation | SIMD technique | Speedup vs scalar |
|---|---|---|
| Roaring Bitmap iterate | AVX2 popcount + gather | ~4x |
| Columnar range scan | AVX2 comparison + mask | ~8x |
| FK existence check | AVX2 Cuckoo Filter probe | ~4x |
| Hash join build phase | AVX512 scatter/gather | ~8x |
| Vector cosine similarity | AVX2/AVX512 FMA | ~8–16x |
| CRC-32C checksum | `_mm_crc32_u64` hardware instruction | ~20x |
| Dictionary decode | AVX2 vpshufb lookup table | ~4x |

---

## 7. Huge Pages and TLB Management

### 7.1 The TLB Problem at Scale

The TLB (Translation Lookaside Buffer) caches virtual → physical page mappings.
With 4KB pages:

```
Working set: 10 GB primary key index
Page count:  10 GB / 4 KB = 2.5 million pages
TLB size:    ~1,500 – 6,000 entries (depends on CPU)
TLB coverage: 6,000 × 4 KB = 24 MB

→ With 10 GB working set and 24 MB TLB coverage, only 0.24% of index lookups
  hit the TLB. Every other lookup triggers a TLB miss: ~100 ns page walk.
  At 4M TPS: 40 million TLB misses/second = 4 seconds of CPU time wasted/second.
```

With 2 MB Huge Pages:

```
Working set: 10 GB
Page count:  10 GB / 2 MB = 5,000 pages
TLB size:    ~512 – 2,048 huge-page entries (separate huge-page TLB)
TLB coverage: 2,048 × 2 MB = 4 GB

→ 4 GB TLB coverage for 10 GB working set → 40% TLB hit rate.
  Much better. For common 1–2 GB working sets: 100% TLB hit rate.
```

### 7.2 Transparent Huge Pages vs Explicit Huge Pages

Linux supports two modes:
- **Transparent Huge Pages (THP):** The OS automatically promotes 4KB pages to 2MB pages.
  Convenient but unpredictable — the OS can demote pages at any time, causing latency spikes.
- **Explicit Huge Pages (`MAP_HUGETLB`):** The application requests huge pages directly.
  Predictable — no OS interference.

VectorVolt uses **explicit huge pages** for all latency-critical allocations:

```rust
fn alloc_huge_pages(size_bytes: usize) -> *mut u8 {
    // Round up to 2MB boundary
    let size = (size_bytes + (2 * 1024 * 1024 - 1)) & !(2 * 1024 * 1024 - 1);
    let ptr = unsafe {
        libc::mmap(
            null_mut(), size,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS | MAP_HUGETLB | MAP_POPULATE,
            -1, 0
        )
    };
    assert_ne!(ptr, libc::MAP_FAILED, "Huge page allocation failed — check /proc/sys/vm/nr_hugepages");
    // mlock to prevent swap
    unsafe { libc::mlock(ptr, size) };
    ptr as *mut u8
}
```

System configuration required (`/etc/sysctl.conf`):
```
vm.nr_hugepages = 5120      # Reserve 5120 × 2MB = 10 GB of huge pages
vm.swappiness = 0           # Disable swap — no surprise latency spikes
```

---

## 8. Memory Tier Summary

VectorVolt organizes its data access across 3 named tiers:

```
Tier 1 — L3 Pinned (mlock + MAP_HUGETLB + MAP_POPULATE):
  • Cuckoo Hash primary key index (~9 bytes/row)
  • Cuckoo Filter (existence check, ~1 bit/row)
  • Zone Map index (min/max summaries, ~32 bytes/partition)
  • Top 1% hot PKs (pre-fetch engine decision)
  Access: ~10–40 ns (L3 hit)
  Budget: tuned to L3 size, never exceed

Tier 2 — DRAM (mmap + MAP_POPULATE, not locked):
  • Micro-Partition data (hot tables, recently accessed)
  • Delta Logs (write buffer, merge-on-read)
  • MVCC version chains
  • Pre-fetched PKs (60–95% probability of near-future access)
  Access: ~60–100 ns (DRAM)
  Budget: available DRAM minus Tier 1 minus OS overhead

Tier 3 — NVMe (io_uring async reads):
  • Cold Micro-Partitions (infrequently accessed)
  • WAL segments older than current
  • Backup snapshots
  Access: ~10–50 µs (NVMe)
  Strategy: io_uring async I/O — never blocks the hot path
```

The pre-fetch engine (Phase 4) monitors access patterns via a Markov Chain
TransactionDependencyGraph and promotes data from Tier 3 → Tier 2 → Tier 1 before
it is needed. The goal: **the hot path never waits for I/O.**

---

## 9. Phase Placement Summary

| Concept | Implemented | Phase |
|---|---|---|
| Cache-line aligned structs (`#[repr(C, align(64))]`) | Phase 1 | HAL |
| NUMA topology detection | Phase 1 | HAL |
| Per-socket NUMA memory arenas + `mbind()` | Phase 1 | HAL |
| Huge Pages for index structures | Phase 1 | HAL |
| `MAP_POPULATE` pre-faulting | Phase 1–2 | Storage, Index |
| CPU core isolation (`sched_setaffinity`) | Phase 2 | Engine |
| SIMD AVX2 scans (Roaring Bitmap, range scan) | Phase 2 | Read Fabric |
| AVX-512 upgrade (opportunistic) | Phase 2 | Read Fabric |
| ARM NEON / SVE | Phase 2 | Read Fabric |
| `isolcpus` kernel boot parameter guidance | Phase 2 | Engine |
| `mlock()` for Tier 1 budget | Phase 2–4 | Index, Pre-fetch |
| jemalloc NUMA arenas | Phase 2 | Engine |
| `io_uring` async NVMe reads (Tier 3) | Phase 4 | Pre-fetch |
| eBPF scheduler hints (CPU affinity at kernel level) | Phase 6 | Dependency Upgrade |

---

*Hardware does not care about software abstractions.*
*VectorVolt is designed to earn every nanosecond the hardware offers.*
