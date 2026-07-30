# VectorVolt — Master Feature Matrix & Traceability Tracker

**Generated:** 2026-07-30
**Purpose:** Single authoritative source of truth for every VectorVolt feature ever planned, its
implementation status, and the sprint that delivers it. Updated at the end of every sprint.

**Sources of truth consumed:** `MASTER_IMPLEMENTATION_PLAN.md`, `VECTORVOLT_LOCKED_DECISIONS.md`,
`VECTORVOLT_CORE_ARCHITECTURE.md`, `CPU_CACHE_NUMA_ISOLATION.md`, `READ_PATH_ARCHITECTURE.md`,
`QUERY_EXECUTION_MODEL.md`, `STORAGE_ENGINE_ADVANCED.md`, `VECTORVOLT_STORAGE_TIERING.md`,
`VECTORVOLT_DISTRIBUTED_REPLICATION.md`, `PREDICTIVE_PREFETCH_ENGINE.md`, `HARDWARE_FRONTIER.md`,
`GRAPH_RELATIONAL_BRIDGE.md`, `RESEARCH_FRONTIER.md`, `STATUS_VECTORVOLT.md`, plus a line-by-line
audit of all 8 crates in `vectorvolt/crates/`.

## Status legend

| Mark | Meaning |
|---|---|
| ✅ | Implemented, tested, on `main` |
| 🟡 | Partially implemented — works but not to spec (named gap in Notes) |
| ⬜ | Not started — code does not exist |
| 🧊 | Deferred by decision (needs Linux/hardware/production data we don't have) |
| 🔬 | Research frontier — docs explicitly disclaim schedule commitment |

## Headline count

| | Count |
|---|---|
| ✅ Implemented | 47 |
| 🟡 Partial | 6 |
| ⬜ Pending | 265 |
| 🧊 Deferred (env-gated) | 28 |
| 🔬 Research | 9 |
| **Total tracked** | **355** |

> **Revised 2026-07-30:** +34 features. Layer 18 (Deployment & Auto-Configuration, `VV-CFG-*`) was
> added after a gap was identified: runtime dispatch handles CPU *instructions*, but nothing handled
> *system configuration* — huge-page reservation, core role maps, arena sizing, WAL device choice.
> Those need root, or a reboot, or knowledge of what is mounted, so they cannot be decided at
> runtime. See [LAYER-18](vectorvolt-tasks/LAYER-18-DEPLOYMENT-AUTOCONFIG.md).

Current test count on `main`: **140** passing. Zero clippy warnings workspace-wide.

---

## LAYER 0 — Silicon / HAL (`vectorvolt-hal`) — CRATE DOES NOT EXIST

The single largest gap. Everything the project's identity rests on — SIMD, cache prefetch, CPU
pinning, NUMA — lives here and none of it is built. Zero SIMD instructions, zero prefetch hints,
zero affinity calls exist in the entire workspace today (verified by grep for `core::arch`, `_mm_`,
`_mm256_`, `target_feature`, `prefetch`, `affinity`, `numa` — all returned no matches).

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-HAL-001 | `CpuTopology` detection (cores, sockets, cache sizes) | ⬜ | S1 | Windows `GetLogicalProcessorInformationEx` + Linux `/proc/cpuinfo`, `/sys/devices/system/node/` |
| VV-HAL-002 | Cache-line alignment primitives `#[repr(align(64))]` | ⬜ | S1 | `CacheAligned<T>`, `CachePadded<T>` |
| VV-HAL-003 | False-sharing avoidance (one-directional ownership) | ⬜ | S1 | Enforced via Rust `&mut`/`&` split |
| VV-HAL-004 | Runtime SIMD feature detection (`cpuid`) | ⬜ | S1 | Detect SSE2/AVX2/AVX-512F/BMI2; ARM NEON/SVE |
| VV-HAL-005 | SIMD dispatch table (compile all, select at startup) | ⬜ | S1 | `SimdTier` enum + fn-pointer dispatch |
| VV-HAL-006 | **AVX2 baseline kernels** (256-bit, 4×u64/cycle) | ⬜ | S1 | Project's stated minimum tier |
| VV-HAL-007 | **AVX-512 opportunistic kernels** (512-bit, 8×u64/cycle) | ⬜ | S1 | Ice Lake+/Zen4 |
| VV-HAL-008 | ARM NEON / SVE kernels | ⬜ | S1 | Graviton/Axion parity |
| VV-HAL-009 | Scalar fallback kernels (portable) | ⬜ | S1 | Correctness oracle for SIMD tests |
| VV-HAL-010 | SIMD kernel: range filter (`lo <= v <= hi` → bitmask) | ⬜ | S1 | Target ~8× vs scalar |
| VV-HAL-011 | SIMD kernel: equality scan / find-first | ⬜ | S1 | |
| VV-HAL-012 | SIMD kernel: min/max (zone map build) | ⬜ | S1 | |
| VV-HAL-013 | SIMD kernel: popcount / bitmap iterate | ⬜ | S1 | Roaring accel, ~4× |
| VV-HAL-014 | SIMD kernel: sum / aggregate | ⬜ | S1 | |
| VV-HAL-015 | SIMD kernel: dictionary decode (`vpshufb`) | ⬜ | S2 | ~4× |
| VV-HAL-016 | SIMD kernel: cosine similarity FMA (vector search) | ⬜ | S8 | ~8–16×, needs VECTOR type |
| VV-HAL-017 | Hardware CRC-32C (`_mm_crc32_u64`) | ⬜ | S1 | ~20× vs software crc32fast |
| VV-HAL-018 | **L1 prefetch** (`_mm_prefetch` `_MM_HINT_T0`) | ⬜ | S1 | |
| VV-HAL-019 | **L2 prefetch** (`_MM_HINT_T1`) | ⬜ | S1 | |
| VV-HAL-020 | **L3 prefetch** (`_MM_HINT_T2`) | ⬜ | S1 | |
| VV-HAL-021 | **Non-temporal / RAM streaming prefetch** (`_MM_HINT_NTA`) | ⬜ | S1 | Bypass cache for streaming scans |
| VV-HAL-022 | ARM `PRFM` prefetch equivalents | ⬜ | S1 | |
| VV-HAL-023 | Prefetch-distance tuned scan loop | ⬜ | S2 | Prefetch N cache lines ahead |
| VV-HAL-024 | **CPU core pinning** (`sched_setaffinity`/`SetThreadAffinityMask`) | ⬜ | S1 | |
| VV-HAL-025 | Core role map (OS/net/writer/reader/LLM/bg/admin groups) | ⬜ | S1 | Per `CPU_CACHE_NUMA_ISOLATION.md` §5.1 |
| VV-HAL-026 | Writer/reader core isolation invariant | ⬜ | S3 | Avoid MESI serialization |
| VV-HAL-027 | Thread-per-core runtime | ⬜ | S3 | |
| VV-HAL-028 | NUMA topology detection | ⬜ | S1 | |
| VV-HAL-029 | `LocalNumaArena` bump allocator | ⬜ | S1 | `!Send` for compile-time NUMA locality |
| VV-HAL-030 | NUMA node binding (`mbind`/`MPOL_BIND`) | 🧊 | — | Linux-only syscall; Windows uses `VirtualAllocExNuma` |
| VV-HAL-031 | 2MB huge pages (`MAP_HUGETLB` / Win large pages) | ⬜ | S1 | TLB miss reduction |
| VV-HAL-032 | `MAP_POPULATE` pre-faulting | 🟡 | S1 | Already used in `PartitionReader::open` — needs HAL wrapper |
| VV-HAL-033 | `mlock()` tier-1 pinning | ⬜ | S1 | |
| VV-HAL-034 | jemalloc NUMA arenas (`percpu_arena`) | ⬜ | S3 | |
| VV-HAL-035 | `isolcpus`/`nohz_full`/`rcu_nocbs` boot-param docs + validator | 🧊 | — | Linux boot config; ship validator + docs |
| VV-HAL-036 | Wait strategies: BusySpin / Yielding / Blocking | ⬜ | S3 | |
| VV-HAL-037 | RDTSC / cycle-accurate timer | ⬜ | S1 | Needed to *measure* nanoseconds honestly |

## LAYER 1 — Durability / WAL (`vectorvolt-wal`) — LARGELY COMPLETE

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-WAL-001 | Segmented append-only log | ✅ | done | |
| VV-WAL-002 | Frame format (97-byte overhead, exact) | ✅ | done | |
| VV-WAL-003 | CRC32 per frame | ✅ | done | Software crc32fast — VV-HAL-017 will accelerate |
| VV-WAL-004 | SHA3-256 hash chain (tamper detection) | ✅ | done | |
| VV-WAL-005 | 64 MiB segment auto-rotation | ✅ | done | |
| VV-WAL-006 | Crash recovery (chain_hash + next_seq) | ✅ | done | |
| VV-WAL-007 | Replay / `read_all` / `read_range` / `verify` | ✅ | done | |
| VV-WAL-008 | `GroupCommitBatcher` (N writes → 1 fsync) | ✅ | done | |
| VV-WAL-009 | Backpressure on bounded channel | ✅ | done | |
| VV-WAL-010 | `BatcherMetrics` (EMA batch size/latency) | ✅ | done | |
| VV-WAL-011 | **Adaptive batching 1→10,000 cmds/fsync** | ⬜ | S3 | LD-BATCH: currently fixed window |
| VV-WAL-012 | Token-bucket backpressure per shard + 429/Retry-After | ⬜ | S6 | LD-BACK |
| VV-WAL-013 | WAL log compaction (post-snapshot + projection safe point) | ⬜ | S5 | |
| VV-WAL-014 | WAL-as-event-log for projections (LSN offsets) | ⬜ | S5 | LD-KAFKA |
| VV-WAL-015 | WAL Shipper (stream segments to hot standby) | ⬜ | S6 | |
| VV-WAL-016 | RDMA WAL transport (+ TCP fallback) | 🧊 | — | Needs RDMA NIC |
| VV-WAL-017 | io_uring WAL writes | 🧊 | — | Linux-only |
| VV-WAL-018 | O_DIRECT bypass page cache | 🧊 | — | Linux-only |

## LAYER 2 — Index (`vectorvolt-index`) — COMPLETE, NOT YET ACCELERATED

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-IDX-001 | Cuckoo hash PK index (2-probe, O(1) worst case) | ✅ | done | Scalar hashing |
| VV-IDX-002 | `RowDescriptor` bit-packed u64 | ✅ | done | file_id:16/partition:16/offset:32 |
| VV-IDX-003 | Roaring bitmap secondary index | ✅ | done | |
| VV-IDX-004 | Zone map (min/max per partition) | ✅ | done | |
| VV-IDX-005 | Cuckoo filter (FK existence, infallible insert) | ✅ | done | Overflow HashSet added S0 |
| VV-IDX-006 | Auto-grow on load factor | ✅ | done | CuckooMap |
| VV-IDX-007 | Huge-page-backed index allocation | ⬜ | S2 | Needs VV-HAL-031 |
| VV-IDX-008 | Cache-line-packed bucket layout | ⬜ | S2 | Currently 16B buckets, want 64B |
| VV-IDX-009 | SIMD bucket probe (compare 8 fingerprints at once) | ⬜ | S2 | |
| VV-IDX-010 | Primary-Key Hash Cache (9 bytes/row, authoritative) | ⬜ | S2 | Backs cuckoo filter |
| VV-IDX-011 | L3-residency budget enforcement for filter | ⬜ | S2 | |
| VV-IDX-012 | HNSW vector index | ⬜ | S8 | |
| VV-IDX-013 | CSR graph adjacency | ⬜ | S9 | |
| VV-IDX-014 | Reverse CSR (incoming edges / CASCADE) | ⬜ | S9 | |

## LAYER 3 — Storage Engine (`vectorvolt-storage`) — CORE DONE

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-STO-001 | Micro-partition `.vvsst` binary format | ✅ | done | |
| VV-STO-002 | Struct-of-Arrays columnar layout | ✅ | done | |
| VV-STO-003 | Dictionary encoding | ✅ | done | |
| VV-STO-004 | Header + file CRC verification | ✅ | done | |
| VV-STO-005 | mmap reader with `populate()` | ✅ | done | Only `unsafe` block in workspace |
| VV-STO-006 | Zone map population at write time | ✅ | done | |
| VV-STO-007 | **O(1) `read_value` point read** | ✅ | done | 400× GET speedup, 200ns p50 |
| VV-STO-008 | Partition compaction (merge N→1, reclaim tombstones) | ✅ | done | PK-sorted output |
| VV-STO-009 | RLE encoding | ⬜ | S2 | |
| VV-STO-010 | Bit-packing encoding | ⬜ | S2 | |
| VV-STO-011 | Delta / Frame-of-Reference encoding | ⬜ | S2 | |
| VV-STO-012 | LZ4 / Zstd block compression | ⬜ | S2 | No compression codec wired today |
| VV-STO-013 | SIMD-accelerated column scan | ⬜ | S2 | Needs VV-HAL-010 |
| VV-STO-014 | Prefetch-driven sequential scan | ⬜ | S2 | Needs VV-HAL-018..023 |
| VV-STO-015 | Tiered compaction strategy (leveled/size-tiered choice) | ⬜ | S5 | Engine has naive merge-all |
| VV-STO-016 | Cross-partition CRC-32C + `ChecksumVerifyWorkflow` | ⬜ | S5 | |
| VV-STO-017 | Copy-on-Write snapshots (non-blocking) | ⬜ | S5 | LD-SNAP |
| VV-STO-018 | Incremental snapshots (1–5% dirty pages) | ⬜ | S5 | |
| VV-STO-019 | L0–L4 five-level tier hierarchy | ⬜ | S5 | LD-COLD |
| VV-STO-020 | Retention policies (time/size/compliance/count) | ⬜ | S5 | LD-RETAIN |
| VV-STO-021 | WORM compliance lock (S3 Object Lock / Azure / GCS) | ⬜ | S5 | |
| VV-STO-022 | Object-storage archive worker (L4) | ⬜ | S5 | |
| VV-STO-023 | PITR + PITR Catalog (any millisecond) | ⬜ | S5 | LD-PITR |
| VV-STO-024 | Merge-on-Read Delta Log (per-core, append-only) | ⬜ | S3 | |

## LAYER 4 — Transactions / Concurrency (`vectorvolt-txn`) — BASIC WRITE PATH ONLY

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-TXN-001 | WAL-first write ordering | ✅ | done | |
| VV-TXN-002 | Write buffer + threshold auto-flush | ✅ | done | |
| VV-TXN-003 | Index maintenance on insert (PK/secondary/filter) | ✅ | done | |
| VV-TXN-004 | Delete (index removal + WAL tombstone) | ✅ | done | |
| VV-TXN-005 | Flush → partition + descriptor rebuild | ✅ | done | |
| VV-TXN-006 | Index rebuild from partitions on reopen | ✅ | done | |
| VV-TXN-007 | WAL delete replay on reopen | ✅ | done | |
| VV-TXN-008 | **LMAX Disruptor ring buffer (single-writer, lock-free)** | ⬜ | S3 | Currently `Mutex<WriteBuffer>` |
| VV-TXN-009 | One-writer-per-shard model | ⬜ | S3 | |
| VV-TXN-010 | **MVCC version chains** | ⬜ | S4 | Zero MVCC code exists today |
| VV-TXN-011 | Snapshot isolation on read paths | ⬜ | S4 | |
| VV-TXN-012 | 7-byte inline MVCC telemetry header | ⬜ | S4 | latency_ns/tier_source/latch_waits/index_hits |
| VV-TXN-013 | Epoch-based reclamation | ⬜ | S4 | |
| VV-TXN-014 | Explicit BEGIN/COMMIT/ABORT transactions | ⬜ | S4 | No multi-statement txn today |
| VV-TXN-015 | 128-bit transaction fingerprint (FNV-1a, ~5ns) | ⬜ | S4 | |
| VV-TXN-016 | `TransactionContext` async cold store | ⬜ | S4 | |
| VV-TXN-017 | `TransactionPriority` QoS (Critical/High/Normal/Low) | ⬜ | S6 | |
| VV-TXN-018 | Strict-priority per-core scheduler | ⬜ | S6 | |
| VV-TXN-019 | Arena allocation, zero-GC hot path (Live/Stable) | ⬜ | S3 | LD-GC |
| VV-TXN-020 | Saga / 2PC for cross-shard | ⬜ | S7 | LD-ACID: never distributed locking |
| VV-TXN-021 | Deterministic replay (`DeterminismGuard`) | ⬜ | S10 | LD-REPLAY |

## LAYER 5 — Query Execution (`vectorvolt-query`) — WORKS, UNOPTIMIZED

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-QRY-001 | PK point lookup (cuckoo → descriptor → partition) | ✅ | done | |
| VV-QRY-002 | Range scan with zone-map pruning | ✅ | done | |
| VV-QRY-003 | Secondary index scan (roaring) | ✅ | done | |
| VV-QRY-004 | FK existence check (zero I/O) | ✅ | done | |
| VV-QRY-005 | Deterministic algorithm selection (no cost-based optimizer) | ✅ | done | QUERY_EXECUTION_MODEL locked stance |
| VV-QRY-006 | **SIMD range scan** | ⬜ | S2 | `range_scan` decodes all columns eagerly today |
| VV-QRY-007 | **Lazy per-row column fetch in scan** | 🟡 | S2 | `get` is lazy; `scan` is not |
| VV-QRY-008 | SIMD roaring bitmap iterate | ⬜ | S2 | |
| VV-QRY-009 | Late materialization (filter before fetch) | ⬜ | S2 | |
| VV-QRY-010 | Vectorized batch execution (columnar batches) | ⬜ | S2 | |
| VV-QRY-011 | SIMD hash join (AVX2 scatter/gather) | ⬜ | S9 | |
| VV-QRY-012 | SIMD sort-merge join | ⬜ | S9 | |
| VV-QRY-013 | Cascade DELETE via bitmap child discovery | ⬜ | S9 | |
| VV-QRY-014 | Aggregate pushdown (SUM/COUNT/MIN/MAX SIMD) | ⬜ | S2 | |
| VV-QRY-015 | Graph traversal via CSR | ⬜ | S9 | |
| VV-QRY-016 | `GRAPH_TRAVERSE()` + `CREATE GRAPH_EDGE` | ⬜ | S9 | |
| VV-QRY-017 | Graph analytics (path count/connectivity/centrality) | ⬜ | S9 | |
| VV-QRY-018 | SQL parser (`sqlparser-rs`) → plan | ⬜ | S8 | |
| VV-QRY-019 | PostgreSQL wire protocol (port 5432) | ⬜ | S8 | LD-PG-WIRE |
| VV-QRY-020 | Vector search `cosine_similarity()` + `VECTOR(n)` type | ⬜ | S8 | |

## LAYER 6 — Engine / Partitioning & Control (`vectorvolt-engine`) — SINGLE TABLE ONLY

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-ENG-001 | `TableEngine` unified read+write per table | ✅ | done | |
| VV-ENG-002 | Partition recovery on open | ✅ | done | |
| VV-ENG-003 | Auto-flush detection + partition registration | ✅ | done | |
| VV-ENG-004 | Compaction orchestration | ✅ | done | |
| VV-ENG-005 | Stats introspection | ✅ | done | |
| VV-ENG-006 | **Multi-table catalog in one process** | ⬜ | S3 | One engine per table today; no catalog |
| VV-ENG-007 | **Schema registry + typed columns** | ⬜ | S3 | Everything is `u64`; schema is `schema.txt` |
| VV-ENG-008 | Real type system (i64/f64/bool/string/bytes/ts/decimal) | ⬜ | S3 | |
| VV-ENG-009 | DDL: CREATE/ALTER/DROP TABLE | ⬜ | S3 | |
| VV-ENG-010 | **Partition control by Entity** | ⬜ | S3 | User-requested |
| VV-ENG-011 | **Partition control by Table** | ⬜ | S3 | User-requested |
| VV-ENG-012 | **Partition control by Schema** | ⬜ | S3 | User-requested |
| VV-ENG-013 | Partition strategy: hash / range / list / composite | ⬜ | S3 | |
| VV-ENG-014 | Partition pruning at plan time | ⬜ | S3 | |
| VV-ENG-015 | Per-partition placement policy (core/NUMA node) | ⬜ | S3 | |
| VV-ENG-016 | Shard-per-core mapping | ⬜ | S3 | |
| VV-ENG-017 | **Config system (TOML file + env + CLI precedence)** | ⬜ | S3 | Zero config-file loading exists today |
| VV-ENG-018 | Config hot-reload | ⬜ | S3 | |
| VV-ENG-019 | Arena sizing config (64MB edge → 64GB cloud) | ⬜ | S3 | LD-ARENA |
| VV-ENG-020 | Multi-tenancy shard groups | ⬜ | S7 | LD-MT-01..08 |
| VV-ENG-021 | Per-tenant metering | ⬜ | S7 | |
| VV-ENG-022 | Tenant onboarding <100ms | ⬜ | S7 | |

## LAYER 7 — Network / Server (`vectorvolt-net`, `vectorvolt-server`) — CRATES DO NOT EXIST

Nothing in the workspace opens a socket. `tokio` is present but used only as a task scheduler for
WAL group commit. There is no server binary — `vvctl` runs in-process and exits.

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-NET-001 | `vectorvolt-server` daemon binary | ⬜ | S4 | |
| VV-NET-002 | Binary wire protocol (frame + opcode + zero-copy payload) | ⬜ | S4 | |
| VV-NET-003 | TCP transport (baseline, portable) | ⬜ | S4 | |
| VV-NET-004 | gRPC / tonic surface (Execute/Query/Txn*/Health) | ⬜ | S4 | |
| VV-NET-005 | Zero-copy request → ring buffer routing | ⬜ | S4 | |
| VV-NET-006 | Connection pool + per-core shard routing | ⬜ | S4 | |
| VV-NET-007 | TLS + mTLS | ⬜ | S4 | |
| VV-NET-008 | `x-vv-txn` fingerprint response metadata | ⬜ | S4 | |
| VV-NET-009 | gRPC health probe | ⬜ | S4 | |
| VV-NET-010 | Busy-poll socket read (no syscall per packet) | ⬜ | S4 | |
| VV-NET-011 | **eBPF / AF_XDP kernel bypass** (~5–20µs RTT) | 🧊 | — | Linux-only; `aya-rs`. Design + trait now, impl on Linux |
| VV-NET-012 | **DPDK poll-mode driver** (~1–3µs RTT) | 🧊 | — | Linux + supported NIC |
| VV-NET-013 | `Transport` trait abstraction (TCP/XDP/DPDK swap) | ⬜ | S4 | Buildable now — enables 011/012 later |
| VV-NET-014 | io_uring socket I/O | 🧊 | — | Linux-only |
| VV-NET-015 | RDMA transport | 🧊 | — | Needs RDMA NIC |

## LAYER 8 — Cluster / Distribution (`vectorvolt-cluster`) — CRATE DOES NOT EXIST

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-CLU-001 | **SWIM gossip membership** (O(log n), 800+ nodes) | ⬜ | S6 | LD-SWIM — not Raft for membership |
| VV-CLU-002 | Ping / ack / ping-req indirect failure detection | ⬜ | S6 | |
| VV-CLU-003 | Infection-style state propagation | ⬜ | S6 | |
| VV-CLU-004 | **4096 virtual nodes on consistent hash ring** | ⬜ | S6 | LD-VNODE |
| VV-CLU-005 | 1–65,536 physical shard support | ⬜ | S6 | |
| VV-CLU-006 | `ShardCatalog` authoritative routing table | ⬜ | S6 | |
| VV-CLU-007 | Catalog caching per node + per SDK | ⬜ | S6 | |
| VV-CLU-008 | **Master-slave (leader-follower) replication** | ⬜ | S6 | User-requested |
| VV-CLU-009 | **Raft consensus** (`raft-rs`) per shard group | ⬜ | S6 | Log replication + leader election |
| VV-CLU-010 | WAL-backed `raft::Storage` impl | ⬜ | S6 | |
| VV-CLU-011 | Majority quorum writes (configurable sync/async) | ⬜ | S6 | LD-QUORUM |
| VV-CLU-012 | **Epoch fencing** (split-brain prevention) | ⬜ | S6 | LD-EPOCH |
| VV-CLU-013 | **<2 second failover** (500ms heartbeat → election) | ⬜ | S6 | LD-FAILOVER |
| VV-CLU-014 | Client-side automatic failover + backoff | ⬜ | S6 | |
| VV-CLU-015 | Shard rebalancing (move ~1/N on scale) | ⬜ | S6 | |
| VV-CLU-016 | Read replica routing (stale default ≤50ms) | ⬜ | S6 | LD-READ-02 |
| VV-CLU-017 | Consistent read opt-in (`/*+ consistent */`) | ⬜ | S6 | |
| VV-CLU-018 | Read replica load balancer (health + lag + RTT) | ⬜ | S6 | |
| VV-CLU-019 | `vectorvolt-reader` separate binary | ⬜ | S8 | LD-READ-01 CQRS at process level |
| VV-CLU-020 | Role-based binary deployment (write/read/control) | ⬜ | S7 | LD-RUNTIME-01 |
| VV-CLU-021 | Shard recovery <60s for 100GB | ⬜ | S6 | |
| VV-CLU-022 | CXL 3.0 CAS-based consensus | 🔬 | — | "nobody has done this yet" |

## LAYER 9 — Control Plane (`vectorvolt-control`) — CRATE DOES NOT EXIST

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-CTL-001 | `vectorvolt-control` node binary | ⬜ | S7 | |
| VV-CTL-002 | Cluster metadata store | ⬜ | S7 | |
| VV-CTL-003 | Schema / catalog registry service | ⬜ | S7 | |
| VV-CTL-004 | Topology management API | ⬜ | S7 | |
| VV-CTL-005 | Admin HTTP API (:8443, axum, HTTPS) | ⬜ | S7 | |
| VV-CTL-006 | Node join / drain / decommission | ⬜ | S7 | |
| VV-CTL-007 | Manual failover trigger | ⬜ | S7 | |
| VV-CTL-008 | Rebalance orchestration | ⬜ | S7 | |
| VV-CTL-009 | QoS rule injection at runtime | ⬜ | S7 | |
| VV-CTL-010 | Admin auth (bcrypt + TOTP, JWT 4h) | ⬜ | S7 | |
| VV-CTL-011 | Svelte SPA admin panel (embedded `include_bytes!`) | ⬜ | S7 | |
| VV-CTL-012 | Emergency stop / resume | ⬜ | S7 | |
| VV-CTL-013 | Immutable audit log for all control actions | ⬜ | S7 | |
| VV-CTL-014 | HITL Index Advisor | ⬜ | S7 | |
| VV-CTL-015 | Backup status dashboard | ⬜ | S7 | |

## LAYER 10 — SDKs (`vectorvolt-sdk-*`) — NONE EXIST

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-SDK-001 | Shared protobuf schema (single source for all SDKs) | ⬜ | S4 | LD-SDK-01 |
| VV-SDK-002 | **Rust SDK** (`vectorvolt-client`) | ⬜ | S7 | |
| VV-SDK-003 | Typed query builder API | ⬜ | S7 | |
| VV-SDK-004 | Transaction builder | ⬜ | S7 | |
| VV-SDK-005 | Client-side shard catalog cache (direct-to-node, no hop) | ⬜ | S7 | |
| VV-SDK-006 | Exponential backoff + retry + jitter | ⬜ | S7 | |
| VV-SDK-007 | **C# / .NET SDK** (`VectorVolt.Client` NuGet) | ⬜ | S7 | Reborn from original C# prototype |
| VV-SDK-008 | LINQ query provider | ⬜ | S7 | |
| VV-SDK-009 | Python SDK (+ pandas integration) | ⬜ | S10 | |
| VV-SDK-010 | Go SDK | ⬜ | S10 | |
| VV-SDK-011 | Java / Kotlin SDK (reactive streams) | ⬜ | S10 | |
| VV-SDK-012 | TypeScript / JS SDK | ⬜ | S10 | |
| VV-SDK-013 | N-2 schema version compatibility | ⬜ | S7 | LD-SDK-01 |

## LAYER 11 — Observability

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-OBS-001 | `tracing` structured logging | ✅ | done | |
| VV-OBS-002 | `BatcherMetrics` (lock-free atomics + EMA) | ✅ | done | |
| VV-OBS-003 | `vvctl bench` latency percentiles (p50/p99/p999) | ✅ | done | |
| VV-OBS-004 | Per-operation latency histograms (HDR) | ⬜ | S3 | `vvwrite_latency_ns` etc. |
| VV-OBS-005 | Prometheus `/metrics` endpoint (150+ counters) | ⬜ | S4 | No HTTP exposition today |
| VV-OBS-006 | OpenTelemetry OTLP trace export | ⬜ | S4 | |
| VV-OBS-007 | Per-shard metric dimensions | ⬜ | S6 | |
| VV-OBS-008 | eBPF system metrics (CPU stall, TLB miss, cache miss) | 🧊 | — | Linux-only |
| VV-OBS-009 | `ratatui` zero-alloc TUI dashboard | ⬜ | S7 | |
| VV-OBS-010 | Always-on profiling (perf / flame graphs) | 🧊 | — | LD-OBS, Linux |
| VV-OBS-011 | Projection consumer lag metric | ⬜ | S5 | |

## LAYER 12 — Testing / Quality

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-TST-001 | 140 unit + integration tests | ✅ | done | |
| VV-TST-002 | Zero-clippy-warning gate | ✅ | done | |
| VV-TST-003 | CI pipeline (build/test/clippy/fmt) | ✅ | done | |
| VV-TST-004 | Multi-platform release CI (5 targets) | ✅ | done | |
| VV-TST-005 | **SIMD-vs-scalar equivalence tests** | ⬜ | S1 | Scalar is the oracle |
| VV-TST-006 | Property-based tests (`proptest`) | ⬜ | S2 | |
| VV-TST-007 | Criterion benchmark suite | ⬜ | S2 | |
| VV-TST-008 | **CI performance gate (>5% p99 regression → reject)** | ⬜ | S2 | LD-GATE |
| VV-TST-009 | Deterministic simulation testing (mock clock/net) | ⬜ | S10 | LD-REPLAY |
| VV-TST-010 | Chaos testing (partition/disk/CPU stall/clock skew) | ⬜ | S10 | LD-CHAOS |
| VV-TST-011 | Fuzzing (`cargo-fuzz`) on format parsers | ⬜ | S2 | |
| VV-TST-012 | Synthetic data generator (FK-aware, histogram-matched) | ⬜ | S6 | |
| VV-TST-013 | Reference domain: Financial Ledger | ⬜ | S10 | RD-1 |
| VV-TST-014 | Reference domain: Workflow Engine | ⬜ | S10 | RD-2 |
| VV-TST-015 | Reference domain: Order Management | ⬜ | S10 | RD-3 |
| VV-TST-016 | Loom concurrency tests | ⬜ | S4 | For lock-free ring buffer |

## LAYER 13 — Security

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-SEC-001 | TLS on wire protocol | ⬜ | S4 | |
| VV-SEC-002 | mTLS client certs | ⬜ | S4 | |
| VV-SEC-003 | AuthN / AuthZ (users, roles, grants) | ⬜ | S7 | |
| VV-SEC-004 | Encryption at rest (AES-256-GCM) | ⬜ | S5 | |
| VV-SEC-005 | `EncryptionBackend` trait (QAT / software) | ⬜ | S5 | |
| VV-SEC-006 | Audit log (immutable) | ⬜ | S7 | |
| VV-SEC-007 | MFA session gate (`MfaGuard`) | 🧊 | — | Needs Linux PAM |
| VV-SEC-008 | SEV-SNP / MKTME per-tenant memory encryption | 🧊 | — | Needs CPU support |
| VV-SEC-009 | Behavioral fingerprinting / anomaly scoring | ⬜ | S10 | |

## LAYER 14 — Prefetch Intelligence (`vectorvolt-prefetch`)

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-PRE-001 | Markov transaction dependency graph | ⬜ | S5 | |
| VV-PRE-002 | Temporal decay (5-min half-life) | ⬜ | S5 | |
| VV-PRE-003 | Tiered promotion (>95%→mlock, 60–95%→DRAM, <60%→NVMe) | ⬜ | S5 | |
| VV-PRE-004 | Background promote/demote loop (50ms tick) | ⬜ | S5 | |
| VV-PRE-005 | Per-query AST speculation on spare core | ⬜ | S5 | |
| VV-PRE-006 | Hot subgraph pre-computation (top 1%) | ⬜ | S9 | |
| VV-PRE-007 | Negative caching via cuckoo filter | ⬜ | S5 | |
| VV-PRE-008 | Working-set boundary policy (5%/40%/rest) | ⬜ | S5 | |

## LAYER 15 — Projections / CQRS

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-PRJ-001 | `Projection` trait + Identity/KeyOnly/Filter/Pipeline | ✅ | done | In `vectorvolt-store` |
| VV-PRJ-002 | `ProjectionDispatcher` | ✅ | done | |
| VV-PRJ-003 | Projection workers reading WAL as event log | ⬜ | S5 | LD-KAFKA |
| VV-PRJ-004 | Per-worker LSN offset tracking | ⬜ | S5 | |
| VV-PRJ-005 | Eventual consistency guarantee (never sync on command path) | ⬜ | S5 | LD-PROJ |
| VV-PRJ-006 | Dead Letter Queue + worker pause + alert | ⬜ | S5 | LD-DLQ |
| VV-PRJ-007 | `vvctl projection replay-dlq` | ⬜ | S5 | |
| VV-PRJ-008 | Parallel SSTable reads | ⬜ | S5 | Deferred in code comment today |
| VV-PRJ-009 | WASM-sandboxed projections | ⬜ | S10 | |

## LAYER 16 — Hardware Acceleration

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-HW-001 | Intel QAT (AES/DEFLATE/SHA offload) + auto-detect | 🧊 | — | Needs QAT card |
| VV-HW-002 | Intel DSA (memcpy/CRC offload) | 🧊 | — | Needs DSA |
| VV-HW-003 | ARM as default deployment target | ⬜ | S1 | Via VV-HAL-008 NEON |
| VV-HW-004 | CXL 3.0 fabric-attached memory | 🔬 | — | |
| VV-HW-005 | FPGA semantic constraint validation | 🔬 | — | $10k+/chip, explicitly not now |

## LAYER 18 — Deployment & Auto-Configuration (`VV-CFG-*`) — CRATE DOES NOT EXIST

One codebase, one binary per architecture, and the system configures itself per host **at install
time**. Full task breakdown: [LAYER-18](vectorvolt-tasks/LAYER-18-DEPLOYMENT-AUTOCONFIG.md) — 34
features, 207 tasks.

The three configuration layers, which must not be conflated:

| Layer | When | Mechanism | Status |
|---|---|---|---|
| Build | compile | one binary per target triple; Cargo features for linked libs | ✅ 5 CI targets |
| **Install** | package install | detect host → emit annotated config → optionally tune system | ⬜ **this layer** |
| Runtime | process start | `cpuid` → dispatch table | ✅ VV-HAL-004/005 |

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-CFG-001 | `SystemProfile` cross-platform detection | ⬜ | S3 | Rust; `vv-probe-linux.sh` is the shell prototype |
| VV-CFG-002 | **Hybrid P-core/E-core classification** | ⬜ | S3 | Found via the dev laptop; pinning a writer to an E-core would harm p99 |
| VV-CFG-003 | Storage device discovery + `fsync` calibration | ⬜ | S3 | |
| VV-CFG-004 | Kernel/OS feature probe | ⬜ | S3 | Probe io_uring by *creating a ring*, not version-guessing |
| VV-CFG-005 | Hardware fingerprint | ⬜ | S3 | Also stamped into benchmark reports |
| VV-CFG-006 | `vvctl tune` — derive config | ⬜ | S3 | Dry-run by default |
| VV-CFG-007 | Derive arena/memory sizing | ⬜ | S3 | LD-ARENA; respects cgroup limits |
| VV-CFG-008 | **Derive core role map** | ⬜ | S3 | P-cores to writers; never two shards on SMT siblings |
| VV-CFG-009 | Derive huge-page count | ⬜ | S3 | |
| VV-CFG-010 | Derive WAL/data device | ⬜ | S3 | **Never auto-pick ephemeral instance-store for WAL** |
| VV-CFG-011 | Derive batching/backpressure/wait strategy | ⬜ | S3 | Window from measured `fsync`, not a fixed 50 µs |
| VV-CFG-012 | Annotated TOML emission | ⬜ | S3 | Every value carries its reasoning |
| VV-CFG-013 | Size profiles (edge/small/standard/large) | ⬜ | S3 | |
| VV-CFG-014 | `--apply` consent-gated tuning | ⬜ | S3 | Dry-run default; audit record |
| VV-CFG-015 | Huge-page reservation | 🧊 | S3 | Linux |
| VV-CFG-016 | memlock limit | 🧊 | S3 | Linux/Windows privilege |
| VV-CFG-017 | `isolcpus` guidance | 🧊 | S3 | **Advise only — never auto-apply.** Needs reboot |
| VV-CFG-018 | Revert applied tuning | ⬜ | S3 | |
| VV-CFG-019 | Idempotency | ⬜ | S3 | |
| VV-CFG-020 | Hardware-change detection at startup | ⬜ | S3 | Fingerprint mismatch → warn, or refuse if unsafe |
| VV-CFG-021 | `vvctl tune --diff` | ⬜ | S3 | Distinguishes operator edits from stale generated values |
| VV-CFG-022 | Support bundle with provenance | ⬜ | S7 | |
| VV-CFG-023 | `.deb` postinst | ⬜ | S7 | |
| VV-CFG-024 | `.rpm` `%post` | ⬜ | S7 | `%config(noreplace)` preserves edits |
| VV-CFG-025 | Homebrew `post_install` | ⬜ | S7 | |
| VV-CFG-026 | winget / MSI custom action | ⬜ | S7 | |
| VV-CFG-027 | Container entrypoint | ⬜ | S7 | **Respect cgroup limits, not host resources** |
| VV-CFG-028 | systemd unit with derived limits | ⬜ | S7 | |
| VV-CFG-029 | **Install must never fail** | ⬜ | S7 | Tuning failure warns; install still succeeds |
| VV-CFG-030 | Uninstall cleanup | ⬜ | S7 | |
| VV-CFG-031 | Precedence: generated < file < env < CLI | ⬜ | S3 | Auto-tune never overrides an explicit choice |
| VV-CFG-032 | Validation with actionable errors | ⬜ | S3 | Reject unknown keys — a silent typo is a latent outage |
| VV-CFG-033 | `vvctl tune --explain` | ⬜ | S3 | Per-value origin + derivation rule |
| VV-CFG-034 | Config schema versioning | ⬜ | S3 | Forward migration preserving edits |

## LAYER 17 — Autonomy (EKOS-adjacent, deferred until VectorVolt core is done)

| ID | Feature | Status | Sprint | Notes |
|---|---|---|---|---|
| VV-AUT-001 | ARP autonomous remediation protocol | 🔬 | — | Needs LLM + Shadowbox |
| VV-AUT-002 | Evidence correlation engine | 🔬 | — | |
| VV-AUT-003 | Shadowbox namespace isolation | 🧊 | — | Linux namespaces |
| VV-AUT-004 | CIP capability injection (WASM) | 🔬 | — | |
| VV-AUT-005 | In-process LLM embedding | 🔬 | — | |
| VV-AUT-006 | Self-upgrading core binary (kexec fd handoff) | 🔬 | — | |
| VV-AUT-007 | Dependency self-upgrade (Class A/B/C) | 🔬 | — | |
| VV-AUT-008 | Double-write execution | 🔬 | — | RESEARCH_FRONTIER |
| VV-AUT-009 | Retroactive rollback | 🔬 | — | RESEARCH_FRONTIER, NP-hard general case |
| VV-AUT-010 | Knowledge compiler | 🔬 | — | RESEARCH_FRONTIER |

---

## Latency budget — what "nanosecond CRUD" actually means

Your own `VECTORVOLT_LOCKED_DECISIONS.md` already commits to numbers consistent with physics.
This table reconciles the "nanosecond CRUD" goal with the hardware, so the target is precise
rather than aspirational.

| Hardware floor | Cost |
|---|---|
| L1 cache hit | ~1 ns |
| L2 cache hit | ~4 ns |
| L3 cache hit | ~15–40 ns |
| DRAM access | ~80–100 ns |
| Memory barrier / atomic RMW | ~10–20 ns |
| NVMe read | ~20–100 µs |
| **`fsync()` to NVMe** | **~50–500 µs** |
| Kernel TCP round-trip | ~50–200 µs |
| AF_XDP round-trip | ~5–20 µs |
| DPDK round-trip | ~1–3 µs |

### Achievable targets per operation

| Operation | Boundary | Target | Physically possible? |
|---|---|---|---|
| **READ** (point, index L3 + data DRAM) | in-process | **< 100 ns** | ✅ Yes. At 200 ns today; SIMD + prefetch + huge pages + cache-packed buckets get us under 100 ns |
| **READ** (range, 10K rows, pruned) | in-process | **< 3 µs** | ✅ Yes with SIMD |
| **CREATE/UPDATE/DELETE** — memory-visible | in-process | **< 200 ns** | ✅ Yes. Ring-buffer append + index mutation, no fsync in path |
| **CREATE/UPDATE/DELETE** — durable ack (group commit amortized) | in-process | **p99 < 10 µs** | ✅ Yes. 10,000 cmds/fsync amortizes the ~500 µs fsync |
| **CREATE/UPDATE/DELETE** — durable ack (unbatched) | in-process | ~50–500 µs | ⚠️ Floor is the fsync. Cannot be nanoseconds — this is silicon, not software |
| Any CRUD | **after network arrival** | **< 1 µs** processing | ✅ Yes with DPDK + the above |
| Any CRUD | client-observed incl. network | ~1–20 µs | Depends on transport tier |

**The honest headline:** nanosecond-scale is achievable and correct as a target for **reads** and for
the **memory-visible portion of writes**. A *durable* write cannot be nanoseconds on any hardware
that exists — `fsync` is microseconds. The way real systems (and your locked decisions) square this
is group commit: batch 10,000 commands into one fsync so the *amortized per-command* durable cost
lands in the microsecond range while the *enqueue* cost stays in nanoseconds. Everything in Sprint 1–3
below is aimed squarely at hitting the nanosecond numbers where physics permits it.

---

## Sprint plan

Each sprint = one branch, one PR, one traceability report appended to this file.

| Sprint | Scope | Key IDs | Delivers |
|---|---|---|---|
| **S1** | `vectorvolt-hal`: topology, SIMD (AVX2/AVX-512/NEON/scalar), L1/L2/L3/NTA prefetch, CPU pinning, NUMA, huge pages, cycle timer | VV-HAL-001..037 | The silicon foundation. Everything else depends on it |
| **S2** | Wire SIMD + prefetch into storage/query/index. Encodings, compression, criterion bench, CI perf gate | VV-STO-009..014, VV-QRY-006..010, VV-IDX-007..011 | Prove <100 ns read |
| **S3** | Type system, multi-table catalog, DDL, **partition control by entity/table/schema**, config system, ring buffer, arenas | VV-ENG-006..019, VV-TXN-008/009/019 | Real database shape |
| **S4** | `vectorvolt-net` + `vectorvolt-server`: protocol, TCP, transport trait, TLS, metrics endpoint. MVCC + txns | VV-NET-001..013, VV-TXN-010..016 | It becomes a server |
| **S5** | Tiering L0–L4, snapshots, PITR, retention, projections/CQRS, DLQ, prefetch engine | VV-STO-016..024, VV-PRJ-003..008, VV-PRE-001..008 | Durability & intelligence |
| **S6** | `vectorvolt-cluster`: SWIM, vnodes, **master-slave replication**, Raft, epoch fencing, <2s failover | VV-CLU-001..018 | It becomes distributed |
| **S7** | `vectorvolt-control` + admin API/UI, Rust SDK, C# SDK, multi-tenancy, RBAC | VV-CTL-001..015, VV-SDK-001..008 | Operable product |
| **S8** | SQL layer, PG wire protocol, read fabric binary, vector/HNSW | VV-QRY-018..020, VV-CLU-019, VV-IDX-012 | SQL + vector search |
| **S9** | Graph: CSR, traversal, joins, cascade delete, analytics | VV-IDX-013/014, VV-QRY-011..017 | Graph engine |
| **S10** | Polyglot SDKs, chaos + deterministic sim testing, reference domains | VV-SDK-009..012, VV-TST-009..015 | Hardening |
| **Linux** | eBPF/XDP, DPDK, io_uring, O_DIRECT, PAM MFA, perf profiling | all 🧊 | Needs a Linux box — traits land in S4 so these drop in |

---

## Sprint log

*(Appended as each sprint completes. This is the traceability record.)*

### S0 — Baseline (pre-existing, merged PRs #10–#13)
Delivered: `vectorvolt-txn`, `vectorvolt-engine`, `vvctl table`, `vvctl bench`, O(1) `read_value`
(400× GET speedup, 80 µs → 200 ns p50), CuckooFilter overflow.
Tests: 140. Clippy: clean.
Open PR not yet merged: `feat/compaction` (partition compaction, VV-STO-008).

### S1a — HAL: SIMD + cache + prefetch + cycle timer ✅ (branch `feat/hal-simd`)

**Delivered — 12 features moved ⬜ → ✅**

| ID | Feature | Evidence |
|---|---|---|
| VV-HAL-002 | Cache-line alignment (`CacheAligned`, `CachePadded`) | test asserts adjacent entries never share a line |
| VV-HAL-003 | False-sharing avoidance | `CachePadded` size == 64 B enforced by test |
| VV-HAL-004 | Runtime SIMD detection (`cpuid`, cached) | `simd::detect` / `simd::tier` |
| VV-HAL-005 | SIMD dispatch table | `OnceLock`-cached tier, branch per call |
| VV-HAL-006 | **AVX2 kernels** | 6 kernels, 2.6× peak measured |
| VV-HAL-009 | Scalar fallback kernels (oracle) | `simd::scalar` |
| VV-HAL-010 | SIMD range filter | `filter_range`, 1.26–1.38× |
| VV-HAL-011 | SIMD equality scan | `find_first_eq`, 1.75–2.62× |
| VV-HAL-012 | SIMD min/max (zone maps) | `min_max`, 1.44–1.59× |
| VV-HAL-014 | SIMD sum/aggregate | `sum`, 1.24× |
| VV-HAL-017 | Hardware CRC-32C | 44.6× vs bitwise scalar |
| VV-HAL-018/019/020/021 | **L1 / L2 / L3 / non-temporal prefetch** | x86-64 `PREFETCHT0/T1/T2/NTA`, aarch64 `PRFM` |
| VV-HAL-022 | ARM `PRFM` prefetch | implemented (compiles for aarch64; unverified on hardware) |
| VV-HAL-037 | RDTSC cycle timer | `CycleTimer`, calibrated 3.07 GHz |
| VV-TST-005 | SIMD-vs-scalar equivalence tests | lengths 0–300 × 3 data regimes |

**Measured speedup** (`vvctl hal`, i7 @ 3.07 GHz, AVX2, 22 logical cores):

| Kernel | 8 KB (L1) | 64 KB (L2) | 512 KB | 8 MB (DRAM) |
|---|---|---|---|---|
| `count_range` | 2.37× | 2.56× | **2.59×** | 2.18× |
| `find_first_eq` | 2.01× | 2.54× | **2.62×** | 1.75× |
| `min_max` | 1.59× | 1.53× | 1.45× | 1.44× |
| `filter_range` | 1.38× | 1.35× | 1.34× | 1.26× |
| `crc32c` (hw) | — | — | — | 44.57× |

**Findings worth carrying forward**

1. **A single-accumulator AVX2 `min_max` measured 0.76× — slower than scalar.** The reduction
   carried a serial dependency chain; LLVM auto-vectorizes the scalar loop with multiple
   accumulators of its own. Fixed with 4-way independent accumulator unrolling → 1.46×.
   *Lesson: hand-written SIMD must beat `-O3` LLVM, not beat naive scalar. Always measure.*
2. **Speedups peak on cache-resident data and decay at 8 MB.** Past L3 the kernels are
   DRAM-bandwidth bound, not compute bound. This is why the index and hot partitions being
   cache-resident (VV-IDX-007/011) matters more than wider vectors.
3. **`filter_range` is capped ~1.35× by its scalar `Vec::push` per match**, not by the compare.
   Late materialization / bitmask output (VV-QRY-009) is the real fix, not AVX-512.
4. **AVX2 has no unsigned 64-bit compare.** Kernels apply a `XOR i64::MIN` range shift so signed
   compare equals unsigned compare. Without it, every PK above `i64::MAX` sorts below 0 and range
   queries silently return wrong rows. Pinned by adversarial tests.
5. **CRC-32C ≠ the WAL's CRC-32/ISO-HDLC.** Different polynomial, different values. The 44× win is
   available only for *new* formats; adopting it in the WAL is a breaking format change.

**Deliberately not done in S1a** (with reasons, not omissions):
- VV-HAL-007 AVX-512 — intrinsic signatures still churning across toolchains; absent on most
  consumer CPUs; downclocks some Xeons. Enum variant reserved, `detect()` never returns it.
- VV-HAL-008 NEON kernels — cannot execute aarch64 here. Shipping untested vector intrinsics is
  worse than the scalar fallback, which is correct everywhere. aarch64 reports `Scalar` today.
- VV-HAL-001/024–036 topology, CPU pinning, NUMA, huge pages — need platform FFI
  (`SetThreadAffinityMask` / `sched_setaffinity`, `VirtualAllocExNuma` / `mbind`). Scheduled S1b.

Tests: **195** workspace-wide (was 140; +55). Clippy: clean, `-D warnings`.

### S1b — HAL: topology, CPU pinning, NUMA, huge pages ⬜ NEXT
Targets VV-HAL-001, 024–036. Then S2 wires SIMD + prefetch into the storage/query/index scan paths
to attack the <100 ns read target.
