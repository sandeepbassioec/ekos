# VectorVolt Core Architecture
## A Standalone Ultra-Low-Latency OLTP Data OS — Anyone Can Adopt This

**Status:** Authoritative Architecture Reference  
**Scope:** VectorVolt as an independent engine — not EKOS, not EKR, not ESIP  
**Audience:** Engineers evaluating VectorVolt, contributors, PhD examiners, and any team
building latency-sensitive systems who want a proven blueprint

---

## 1. What VectorVolt Is

VectorVolt is a deterministic, memory-first, kernel-bypass OLTP engine targeting
**sub-microsecond transaction latency** with zero garbage collection and zero stop-the-world
pauses.

It is not a wrapper around PostgreSQL or RocksDB. It is not a cache layer. It is a
ground-up reimplementation of every layer between the application and the silicon —
from CPU cache pinning at the HAL layer to zero-copy kernel-bypass networking at the
wire layer.

### Why This Exists

Every mainstream database engine was designed in an era when:
- CPUs had one socket, one NUMA domain, simple caches
- Networking meant TCP/IP kernel stacks with >100 µs latency
- Storage meant spinning disks with millisecond access
- RAM was scarce — data lived on disk, caches were an optimization

None of that is true anymore. A modern server has:
- 128–512 cores across 2–4 NUMA nodes
- NVMe SSDs with 10–50 µs latency (10x better than HDD)
- RDMA-capable NICs with 1–5 µs round-trips
- DDR5/HBM with 50 GB/s+ bandwidth per socket
- CPU caches reaching 300+ MB L3 on EPYC/Xeon

VectorVolt was designed from scratch for this hardware reality.

### The Single Design Principle

> **Zero allocation on the hot path. Every latency-critical operation must be
> expressible as a read from a known memory address.**

Everything else — MVCC, clustering, self-healing, graph traversal, vector search —
is derived from this principle.

---

## 2. The 8-Phase Architecture

VectorVolt is built in 8 phases, each a complete and independently usable layer.
Earlier phases do not depend on later phases. A team can adopt Phase 1–3 alone
(a nanosecond WAL + storage engine) without committing to Phases 4–8.

```
Phase 1:  Silicon / HAL          CPU topology, NUMA arenas, cache-line primitives
Phase 2:  Single-Writer Engine   LMAX Disruptor ring buffer, transaction pipeline
Phase 3:  Durability / WAL       Segmented append-only log, group commit, recovery
Phase 4:  Kernel-Bypass Network  eBPF/XDP, DPDK, RDMA replication, zero-copy
Phase 5:  Read Fabric            SIMD scans, columnar store, HNSW vectors, graph CSR
Phase 6:  Control Plane          Raft consensus, CPU pinning, shard catalog, multi-tenancy
Phase 7:  CLI & Dashboard        vvctl operator CLI, ratatui TUI, full observability
Phase 8:  Testing & Research     Deterministic simulation, chaos testing, proptest
```

---

## 3. Phase 1 — Silicon / HAL

The Hardware Abstraction Layer is the foundation. Every data structure in VectorVolt
is designed to respect the hardware it runs on.

### 3.1 CPU Topology Detection

At startup, VectorVolt reads `/proc/cpuinfo` and the NUMA topology (`/sys/devices/system/node/`)
to build a `CpuTopology` struct:

```rust
pub struct CpuTopology {
    pub numa_nodes:    Vec<NumaNode>,
    pub cores_per_node: usize,
    pub l1_cache_size: usize,   // per core (typically 32–64 KB)
    pub l2_cache_size: usize,   // per core (typically 256 KB–1 MB)
    pub l3_cache_size: usize,   // shared per socket (typically 32–512 MB)
    pub cache_line_bytes: usize, // 64 bytes on x86/ARM
    pub simd_features: SimdFeatures, // AVX2, AVX-512, NEON
}
```

This is not optional. VectorVolt uses this information to make all memory allocation and
core assignment decisions at startup. Running VectorVolt on a machine without reading
its topology is like running a race without knowing the track.

### 3.2 Cache-Line Alignment

Every hot-path data structure is declared `#[repr(C, align(64))]` — aligned to the 64-byte
cache line. This eliminates false sharing: two threads updating adjacent fields in the same
cache line would cause that line to bounce between CPU caches. With explicit alignment,
each struct occupies exactly one (or a known multiple of) cache lines.

```rust
#[repr(C, align(64))]
pub struct CommandFrame {
    pub sequence:   u64,      // 8 bytes — unique per transaction
    pub timestamp:  u64,      // 8 bytes — nanosecond wall clock
    pub payload_ptr: *const u8, // 8 bytes — zero-copy payload reference
    pub payload_len: u32,     // 4 bytes
    pub shard_id:   u16,      // 2 bytes
    pub priority:   u8,       // 1 byte
    pub flags:      u8,       // 1 byte
    _padding:       [u8; 30], // pad to exactly 64 bytes
}
// Total: 64 bytes = 1 cache line. Compiler error if layout changes.
```

### 3.3 NUMA Memory Arenas

NUMA (Non-Uniform Memory Access): on a multi-socket server, memory attached to Socket 0
is ~100 ns for a CPU on Socket 0 but ~160–200 ns for a CPU on Socket 1. That 60–100 ns
penalty per memory access compounds catastrophically on the hot path.

VectorVolt allocates memory from **per-socket arenas**:

```rust
pub struct LocalNumaArena {
    node_id:  usize,
    base_ptr: *mut u8,       // mmap'd from this NUMA node's memory
    cursor:   AtomicUsize,   // bump allocator — zero locking overhead
    capacity: usize,
}

impl LocalNumaArena {
    pub fn new(numa_node: usize, capacity: usize) -> Self {
        // mbind() binds this allocation to the target NUMA node
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                capacity,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_POPULATE,
                -1, 0
            )
        };
        libc::mbind(ptr, capacity, MPOL_BIND, &[1 << numa_node], ...);
        Self { node_id: numa_node, base_ptr: ptr, ... }
    }
}
```

Rust's ownership system enforces NUMA locality at compile time: a `LocalNumaArena`
is `!Send`, so it cannot be moved to a different thread (and therefore a different
NUMA node) accidentally.

### 3.4 Huge Pages

The CPU TLB (Translation Lookaside Buffer) maps virtual → physical addresses. With
4KB pages, a 10 GB working set requires 2.5 million TLB entries — far more than the
TLB can hold (typically 1,500–6,000 entries). Every TLB miss costs 100+ ns.

VectorVolt uses **2 MB Huge Pages** for all index structures and the Cuckoo Hash primary
key index. With 2 MB pages, the same 10 GB working set requires only 5,000 TLB entries
— within TLB capacity. TLB misses on the hot path drop to near zero.

```rust
// Allocate Huge Pages via MAP_HUGETLB
let ptr = libc::mmap(
    null_mut(), size,
    PROT_READ | PROT_WRITE,
    MAP_PRIVATE | MAP_ANONYMOUS | MAP_HUGETLB | MAP_POPULATE,
    -1, 0
);
libc::mlock(ptr, size);  // Pin — OS cannot swap this out
```

`MAP_POPULATE` pre-faults all pages into RAM at allocation time. No page faults on
the hot path.

---

## 4. Phase 2 — Single-Writer Engine

### 4.1 LMAX Disruptor Ring Buffer

The core write pipeline is modeled on the LMAX Disruptor: a single-writer, multi-reader
ring buffer with no locks on the hot path.

```
Client gRPC call
       ↓
  CommandParser (dedicated core)
       ↓
  Ring Buffer (2^N slots, power-of-2 for modulo-free indexing)
       ↓                  ↓                  ↓
  WAL Writer          MVCC Engine       Index Updater
  (dedicated core)    (dedicated core)  (dedicated core)
       ↓                  ↓                  ↓
  Segment File        VersionChain       CuckooHash + RoaringBitmap
```

There is exactly **one writer thread per shard**. Writers never contend with each other.
Readers never block writers. This is the fundamental difference from PostgreSQL's locking model.

### 4.2 Core Isolation Model

VectorVolt assigns CPU cores by function at startup, based on the detected `CpuTopology`:

```
Core 0:       OS + interrupt handling (reserved — VectorVolt never touches this)
Cores 1-3:    Command parsing + gRPC receive (network hot path)
Cores 4-7:    OLTP writers (one per shard, NUMA-local to their shard's memory)
Cores 8-11:   OLTP readers (SIMD scan workers, can run on any NUMA node)
Cores 12-15:  In-process LLM / embedding inference (dedicated, never evicted)
Cores 16-17:  Background tasks (pre-fetch, WAL shipping, ARP, compaction)
Cores 18+:    Available for control plane, admin, monitoring
```

Each group is pinned via `libc::sched_setaffinity`:

```rust
pub fn pin_thread_to_cores(core_ids: &[usize]) {
    let mut cpu_set = unsafe { std::mem::zeroed::<libc::cpu_set_t>() };
    for &core in core_ids {
        unsafe { libc::CPU_SET(core, &mut cpu_set); }
    }
    unsafe { libc::sched_setaffinity(0, std::mem::size_of_val(&cpu_set), &cpu_set) };
}
```

### 4.3 Wait Strategies

The ring buffer supports three wait strategies tuned for different latency/CPU tradeoffs:

| Strategy | Behavior | Latency | CPU Use |
|---|---|---|---|
| `BusySpin` | CAS loop, never yields | Sub-microsecond | 100% on the core |
| `Yielding` | `std::hint::spin_loop()` | ~1–5 µs | High but yields scheduler |
| `Blocking` | Condition variable | ~10–100 µs | Low — thread sleeps |

Financial-grade deployments use `BusySpin` on isolated cores. Cloud deployments use
`Yielding` or `Blocking` depending on the SLA.

---

## 5. Phase 3 — Durability / WAL

The WAL is a **segmented append-only log** with cryptographic hash chaining:

```
Segment 0          Segment 1          Segment 2
[Frame][Frame]...  [Frame][Frame]...  [Frame][Frame]...
64 MiB, auto-rotate at capacity
```

Each frame contains:
- Magic bytes (0xVV_WAL)
- CRC32C checksum (hardware-accelerated, Intel QAT if available)
- SHA3-256 link to previous frame (tamper detection)
- Payload (zero-copy reference — the data is not copied into the frame)

**Group Commit Batcher:** N writes are batched into a single `fsync()`. The batcher
accumulates frames until either (a) N frames are ready or (b) the batch timeout
(configurable, default 50 µs) expires. This reduces `fsync()` calls from N to 1
while keeping durability guarantees intact.

**Recovery:** On restart, VectorVolt replays the last open WAL segment, validates
hash chain integrity, and rebuilds the in-memory index from the replay. No external
recovery coordinator needed.

---

## 6. Phase 4 — Kernel-Bypass Networking

### 6.1 Why Kernel Bypass

The Linux TCP/IP stack adds 50–200 µs of latency per network round-trip:
- `send()` → kernel → NIC driver → wire: ~50 µs
- Wire → NIC interrupt → kernel → `recv()`: ~50–150 µs

For sub-microsecond transactions, this is unacceptable. Kernel bypass eliminates
the OS from the data path entirely.

### 6.2 eBPF / AF_XDP

For moderate-throughput deployments (commodity servers, cloud VMs), VectorVolt uses
**AF_XDP** — a kernel bypass mechanism that redirects packets directly from the NIC
to user-space memory without kernel involvement:

```rust
// ekos-ebpf crate — eBPF programs compiled via aya-rs (pure Rust eBPF)
// XDP program: filter VectorVolt packets at the NIC driver level
// AF_XDP socket: receive those packets directly in user-space ring buffer
```

Latency: ~5–20 µs round-trip (vs 50–200 µs for standard TCP).

### 6.3 DPDK

For maximum-throughput deployments (HFT, financial settlement, telecom),
VectorVolt uses **DPDK (Data Plane Development Kit)** — a poll-mode driver that
completely bypasses the kernel:

```
Standard Linux path:   NIC → Interrupt → Kernel Driver → TCP Stack → recv()
DPDK path:             NIC → DPDK Poll-Mode Driver → User-space ring buffer
```

The NIC is "owned" by DPDK. No interrupts. The DPDK poll loop runs at 100% CPU
on its dedicated core, continuously checking the NIC for new packets.

DPDK is integrated via `dpdk-sys` (Rust bindgen bindings to the DPDK C library):
- Phase 4 target: `dpdk-sys` crate with safe Rust wrappers
- DPDK memory is `MAP_HUGETLB`-allocated — same huge page strategy as the HAL layer
- Zero-copy deserialization: protobuf frames are parsed directly from the DPDK packet
  buffer — no copy to application memory

Latency: ~1–3 µs round-trip. At scale (10M+ TPS), DPDK is the difference between
possible and impossible.

### 6.4 RDMA

For cluster replication (WAL streaming to hot-standby replicas), VectorVolt uses
**RDMA (Remote Direct Memory Access)** when Mellanox/NVIDIA ConnectX or InfiniBand NICs
are present:

```
Without RDMA:  Node A CPU → TCP Stack → NIC → Network → NIC → CPU → Node B RAM
               Latency: ~100 µs

With RDMA:     Node A NIC → Network → Node B NIC → Node B RAM (bypasses Node B CPU)
               Latency: ~1–5 µs — 20x improvement
```

WAL segments are written directly into the replica's memory by the primary's NIC.
The replica CPU is not involved in the receive path.

Integrated via `rdma-core` Rust bindings. Fallback: TCP WAL streaming when RDMA
NICs are absent.

### 6.5 gRPC Server Layer

Client-facing connectivity uses gRPC (tonic) over TLS. The `vectorvolt-server` crate
provides the network gateway:

```rust
// Endpoints:
// Execute    — submit a transaction
// Query      — read-only query (routed to reader cores)
// TxnBegin   — start an explicit multi-step transaction
// TxnCommit  — commit with fingerprint return
// TxnAbort   — abort and release locks
// Health     — grpc.health.v1.Health probe
```

Zero-copy path: gRPC request bytes are routed directly to the ring buffer without
copying payload data. The `x-vv-txn` response header contains the 128-bit transaction
fingerprint for client-side correlation.

---

## 7. Phase 5 — Read Fabric

### 7.1 SIMD Strategy: AVX2 First, AVX-512 When Available

VectorVolt targets **AVX2 as the baseline SIMD tier** — supported by every x86-64 server
CPU from 2013 onwards (Intel Haswell, AMD Ryzen/EPYC). AVX-512 is used as an opportunistic
upgrade when available (Intel Xeon Ice Lake+, some AMD Zen 4).

```rust
pub fn scan_bitmap_simd(bitmap: &[u64], predicate: u64) -> Vec<usize> {
    #[cfg(target_feature = "avx512f")]
    return scan_avx512(bitmap, predicate);  // 512-bit registers: 8 u64 per cycle

    #[cfg(target_feature = "avx2")]
    return scan_avx2(bitmap, predicate);    // 256-bit registers: 4 u64 per cycle

    return scan_scalar(bitmap, predicate);  // fallback: 64-bit, 1 per cycle
}
```

On ARM (AWS Graviton5, Google Axion), the equivalent is **NEON / SVE** — Rust's standard
library auto-emits these intrinsics via `target_feature` detection. The same abstract scan
function works across all architectures.

### 7.2 Read Path Overview

Point lookup path (~120 ns):
```
PK lookup → Cuckoo Hash (L3, Huge Pages) → Row Descriptor (u64: file+partition+offset)
         → Micro-Partition mmap → Column data at offset
```

Secondary index lookup (~500 ns for 1K results):
```
Predicate → Roaring Bitmap index → SIMD iterate → Row Descriptor batch
         → Micro-Partition bulk fetch (columnar, SIMD-friendly)
```

Range scan with pruning (~3 µs for 10K rows):
```
Range bounds → Zone Map index (min/max per partition) → prune non-matching partitions
           → binary search within surviving partitions → SIMD stream
```

Graph traversal (~400 ns for 1K neighbors):
```
Root PK → CSR adjacency array (sequential in memory) → neighbor PKs
        → bulk Cuckoo Hash lookups → neighbor Row Descriptors
```

### 7.3 Micro-Partitions

The on-disk (and in-memory) storage unit is the **Micro-Partition**: a columnar, dictionary-encoded
file of ~100K rows, `mmap`'d with `MAP_POPULATE`.

- **Struct-of-Arrays layout**: all values for column A are contiguous in memory. A SIMD scan
  of column A never touches column B's cache lines — perfect cache utilization.
- **Dictionary encoding**: string/enum columns stored as compact integer codes. 4-byte code
  vs variable-length string = 4–20x compression, better cache density.
- **Zone Maps**: each Micro-Partition records min/max for every column. Range queries prune
  entire partitions without reading a single row.

### 7.4 Vector / Embedding Engine

`VECTOR(n)` is a native column type, not a plugin. HNSW (Hierarchical Navigable Small World)
index for approximate nearest neighbor search. `cosine_similarity()` as a first-class SQL
function. In-process LLM embedding inference on dedicated cores (Phases 3+).

---

## 8. Phase 6 — Control Plane

### 8.1 Clustering — Raft Consensus

VectorVolt cluster topology: N data nodes, each owning M shards. Raft consensus
(`raft-rs` crate — production-proven Rust implementation) manages:

- Leader election per shard group
- WAL log replication (Raft log entries = WAL frames)
- Membership changes (add/remove nodes without downtime)
- Epoch fencing (no split-brain: old leader's writes rejected after epoch change)

```
Cluster (3 nodes, 6 shards):

  Node 0: Shards 0, 1  (Leader for S0, Follower for S2, S4)
  Node 1: Shards 2, 3  (Leader for S2, Follower for S0, S4)
  Node 2: Shards 4, 5  (Leader for S4, Follower for S1, S3)

Write to Shard 2:
  Client → Node 1 (leader for S2) → Raft replication → Nodes 0, 2
  Commit when quorum (2/3 nodes) acknowledge
```

CXL 3.0 (Phase 6+): when CXL fabric-attached memory becomes available, the Raft consensus
is replaceable via `ConsensusBackend` trait with `CxlCasConsensus` — reducing distributed
commit from ~5 ms (Raft) to ~1 µs (CXL atomic CAS).

### 8.2 Shard Catalog and Routing

The shard catalog is stored in KoStore (VectorVolt's own storage) — the system manages itself.
The catalog records: shard → node mapping, epoch, replication factor, health status.

Client SDKs cache the shard catalog locally. A transaction is routed directly to the correct
node without a coordinator hop.

### 8.3 Multi-Tenancy

Each tenant owns an isolated KO namespace:
- Separate memory arenas (data from Tenant A never shares a cache line with Tenant B)
- Separate WAL segments (recovery for one tenant does not scan another's WAL)
- Separate Micro-Partition files (compaction for one tenant does not block another)
- Optional: SEV-SNP / MKTME hardware memory encryption per tenant (Phase 6+)

---

## 9. Phase 7 — CLI & Dashboard

`vvctl` is the operator CLI binary. It communicates with the running `vectorvolt-server`
via gRPC:

```
vvctl status                   — node health, shard map, replication lag
vvctl nodes                    — list all nodes with CPU/RAM/latency stats
vvctl shards                   — shard leader/follower assignments
vvctl wal status               — WAL segment count, last commit, replay position
vvctl wal verify               — hash chain integrity check
vvctl debug txn <fingerprint>  — decode a transaction: timing, tiers, latch waits
vvctl synthetic generate       — schema-aware fake data for load testing
vvctl index list               — show all indexes with hit rates
vvctl index rebuild <table>    — background index rebuild
vvctl backup status            — last verified backup timestamp + result
vvctl qos override             — inject QoS rule at runtime
vvctl failover <shard>         — manually trigger leader election
vvctl drain <node>             — gracefully drain a node for maintenance
vvctl rebalance                — rebalance shard leaders across nodes
```

TUI dashboard (`ratatui`, zero allocations):
```
┌─ VectorVolt v0.x.x ──────────────────────────────────────────────┐
│ Nodes: 3/3 healthy   Shards: 6/6 online   Replication: 0ms lag  │
├─────────────────────────────────────────────────────────────────┤
│ Latency (p99)   │ Write: 890 ns  │ Read: 120 ns  │ Graph: 410 ns│
│ Throughput      │ 4.2M TPS       │ 18.7M QPS     │             │
│ Memory          │ Tier 1: 94%    │ Tier 2: 71%   │ Tier 3: 12% │
├─────────────────────────────────────────────────────────────────┤
│ [nodes] [shards] [wal] [indexes] [prefetch] [arp] [quit]        │
└─────────────────────────────────────────────────────────────────┘
```

---

## 10. Phase 8 — Testing & Research

### Deterministic Simulation Testing

VectorVolt's async runtime (Tokio) supports deterministic simulation: replace the real
clock and real network with mock implementations, run thousands of failure scenarios
in a single-threaded deterministic test. No flaky tests — the same seed produces the
same execution every time.

```rust
#[tokio::test]
async fn test_raft_leader_election_under_network_partition() {
    let sim = SimulationRuntime::new(seed: 42);
    sim.partition_nodes([0, 1], [2]);    // simulate network split
    // ... assert quorum behavior ...
}
```

### Chaos Testing

Systematic failure injection:
- Network partition between any subset of nodes
- Disk I/O failure on specific segments
- CPU stall simulation (sleep injection on hot path)
- Memory pressure (limit arena capacity)
- Clock skew between nodes

---

## 11. Observability — World-Class Telemetry

VectorVolt is designed to be observed at every layer. This is not an afterthought —
it is a first-class design requirement because VectorVolt is intended for adoption
by other teams and organizations.

### 11.1 Latency Histograms (per operation, per shard)

```
vvwrite_latency_ns{shard="0", priority="critical"}  p50=410 p95=680 p99=890
vvread_latency_ns{table="orders", index="primary"}  p50=85  p95=140 p99=210
vvgraph_latency_ns{depth="3"}                       p50=320 p95=500 p99=620
```

### 11.2 Zero-Overhead MVCC Telemetry

Every MVCC row version header stores 7 bytes of inline telemetry (see RESEARCH_FRONTIER.md §3):
- `latency_ns` (u32): how long the write took
- `tier_source` (u8): which memory tier served the pre-write read
- `latch_waits` (u8): how many times this write waited for a latch
- `index_hits` (u8): how many index lookups this transaction performed

Querying this telemetry is a table scan — no separate tracing agent needed.

### 11.3 Prometheus + OpenTelemetry Endpoints

- `/metrics` (Prometheus format) — 150+ counters, gauges, histograms
- OTLP trace export (OpenTelemetry) — distributed tracing across client → server → storage
- eBPF system metrics — CPU stall cycles, TLB miss rate, cache miss rate (Phase 6+)

### 11.4 Admin Panel

Svelte SPA embedded in the server binary. TOTP + bcrypt authentication. Accessible
at `:8443`. Shows: live latency, shard health, WAL lag, pre-fetch coverage, ARP status,
pending approvals, backup verification, trust policy.

---

## 12. Client SDKs

VectorVolt's gRPC API is defined in protobuf. SDKs are generated from the same proto
definitions, ensuring API consistency across all languages:

| SDK | Package | Phase | Notes |
|---|---|---|---|
| Rust | `vectorvolt-client` (crates.io) | 3 | Async/await, zero-copy where possible |
| C# / .NET | `VectorVolt.Client` (NuGet) | 3 | The original VectorVolt prototype reborn as SDK |
| Go | `vectorvolt-go` | 4 | Idiomatic Go client |
| Java / Kotlin | `vectorvolt-java` (Maven) | 4 | Reactive streams support |
| Python | `vectorvolt-py` (PyPI) | 4 | pandas DataFrame integration |

All SDKs cache the shard catalog locally. They route writes to the correct node directly —
no coordinator hop.

---

## 13. VectorVolt as a Standalone Adoptable Engine

VectorVolt is designed so that **any team building a latency-sensitive system can adopt
the phases they need** without adopting the rest of the VectorVolt ecosystem.

### Adoption Paths

**Minimal adoption (Phases 1–3 only):**  
A nanosecond-class WAL + storage engine. Replace your existing KV store or message broker.
No kernel bypass needed. Runs on any Linux server.

**Storage + reads (Phases 1–5):**  
Full OLTP engine — writes, reads, secondary indexes, graph traversal, vector search.
The complete data layer for any application. Add EKOS on top for self-healing intelligence.

**Full engine (Phases 1–8):**  
The complete VectorVolt Data OS including kernel-bypass networking, clustering, CLI,
and world-class observability. Deployable as a distributed system across N nodes.

### What Makes It World-Adoptable

- **Single binary:** `ekosd` (or `vectorvolt-server` standalone) ships as a statically
  linked musl binary. No JVM, no .NET runtime, no Python interpreter. Runs anywhere Linux runs.
- **No external dependencies at runtime:** No ZooKeeper, no etcd, no Kafka. VectorVolt
  manages its own consensus, its own catalog, its own WAL.
- **Polyglot client SDKs:** Any team's language is supported. The gRPC API is language-agnostic.
- **Gradual adoption:** Phase 1–3 can be dropped in as a WAL library. No big-bang migration.
- **Operator-friendly:** `vvctl` and the admin panel give operations teams full visibility
  without requiring Rust knowledge.

---

## 14. Hardware Compatibility Matrix

| Hardware | VectorVolt Benefit | Phase |
|---|---|---|
| Any x86-64 Linux | Full support, AVX2 SIMD | 1+ |
| ARM (Graviton5 / Axion) | 35–65% better price-perf, NEON SIMD | 1+ |
| Intel QAT (built-in Xeon) | Free crypto/compression offload | 3 |
| Intel DSA (built-in Xeon) | Free memory copy acceleration | 4 |
| RDMA NICs (Mellanox/ConnectX) | 20x WAL replication improvement | 5 |
| DPDK-capable NICs | Sub-3µs network round-trips | 4 |
| CXL 3.0 fabric memory | 1000x distributed commit latency | 6+ |
| FPGA (HFT-grade) | Sub-ns constraint evaluation | Specialized |

VectorVolt runs on commodity hardware from day one. Hardware acceleration is
an upgrade path, not a requirement.

---

## 15. Crate Map

```
vectorvolt/
  crates/
    vectorvolt-wal/       Phase 1–3: WAL, group commit, hash chain     ✅ Done
    vectorvolt-store/     Phase 1:   KoStore, KO put/get/history/scan  ✅ Done
    vectorvolt-index/     Phase 2:   Cuckoo Hash, Roaring Bitmap, Zone Map
    vectorvolt-storage/   Phase 2:   Micro-Partition format, mmap, dict encoding
    vectorvolt-query/     Phase 2:   Point lookup, range scan, FK validation
    vectorvolt-vector/    Phase 2:   VECTOR(n), HNSW, cosine_similarity
    vectorvolt-server/    Phase 2:   gRPC server (tonic), connection pool
    vectorvolt-graph/     Phase 3:   CSR, graph traversal, GRAPH_TRAVERSE SQL
    vectorvolt-sdk-rust/  Phase 3:   Async Rust client SDK
    vectorvolt-sdk-dotnet/ Phase 3:  C# / .NET client SDK (NuGet)
    vectorvolt-sdk-go/    Phase 4:   Go client SDK
    vectorvolt-sdk-java/  Phase 4:   Java / Kotlin client SDK
    vectorvolt-sdk-python/ Phase 4:  Python client SDK + pandas
  binaries/
    vvctl/                Phase 2:   Operator CLI (clap + ratatui)
```

---

*VectorVolt is not a product of any one company's roadmap.*
*It is a specification for how a database engine should behave when hardware constraints are respected.*
*Any team that implements these phases correctly will have a system that outperforms every general-purpose database on latency-sensitive workloads.*
