# VectorVolt — Phase-Wise Implementation Plan
*Version: 2.0.0 | Date: 2026-04-05 | Status: DRAFT FOR REVIEW*

---

## Guiding Principles

1. **Learning first, speed second.** Every concept is taught (physics → C# → proof) before code is written.
2. **Test first, code second.** Every component gets failing tests, then implementation to make them pass.
3. **One concept at a time.** No jumping ahead. Each concept is understood by heart before moving on.
4. **Working code only.** Every phase produces compiling, tested, benchmarked code.
5. **This may take years.** That is acceptable. Understanding is the goal.

---

## Phase 1: The Silicon Foundation (Memory & CPU)

*Goal: Control where data lives and who computes it.*

### Concept 1.1: CPU Caches & Cache Lines
- **Learn:** How L1/L2/L3 caches work. Why data is fetched in 64-byte blocks. What a cache miss costs (1ns vs 100ns). The MESI protocol.
- **Test:** Assert sizeof(CommandFrame) == 64. Assert sizeof(WalRecord) == 64.
- **Build:** CacheLineConstants, CommandFrame (64-byte struct), WalRecord (64-byte struct).
- **Prove:** BenchmarkDotNet comparing aligned vs unaligned struct access.

### Concept 1.2: False Sharing
- **Learn:** Why two threads writing to adjacent memory destroy performance. How cache line invalidation works across cores.
- **Test:** Benchmark two threads incrementing counters — with and without padding.
- **Build:** Cache-line padded fields in ring buffer head/tail pointers.
- **Prove:** Show the 10-50x performance difference with/without padding.

### Concept 1.3: Zero-GC Memory Arena
- **Learn:** Why .NET GC is a stop-the-world event (10-50ms). Why NativeMemory bypasses GC. How bump allocation works (O(1) alloc, O(1) reset).
- **Test:** Arena allocates correct sizes. Arena resets to zero. Arena rejects when exhausted. Struct allocations don't overlap.
- **Build:** MemoryArena (NativeMemory.AlignedAlloc, bump pointer, cache-line-aligned).
- **Prove:** BenchmarkDotNet: MemoryArena.Allocate vs `new byte[N]`. Measure GC pauses with .NET diagnostics.

### Concept 1.4: HugePages & TLB
- **Learn:** What the TLB is (CPU's page table cache). Why 4KB pages cause TLB misses with large memory. How 2MB pages reduce misses by 512x. Why 1GB pages are the ultimate — 4 TLB entries cover 4GB of RAM, making TLB misses near-impossible.
- **Test:** Verify HugePage allocation succeeds on supported OS. Test both 2MB and 1GB modes.
- **Build:** HugePageAllocator (2MB default, 1GB configurable; requires Linux boot-time reservation for 1GB).
- **Prove:** TLB miss counters via perf stat (Linux) — compare 4KB vs 2MB vs 1GB pages.

### Concept 1.5: Thread Pinning & NUMA
- **Learn:** What OS preemption is (~10ms scheduler quantum). Why thread migration kills L1/L2 cache. What NUMA is (CPU socket → local RAM bank). Why remote NUMA access costs 2x.
- **Test:** Verify thread stays on assigned core. Verify NUMA-local memory allocation.
- **Build:** AffinityScheduler (P/Invoke for Windows and Linux).
- **Prove:** Benchmark pinned vs unpinned thread performing memory-intensive work.

### Concept 1.6: Platform Detection & Dynamic CPU Dispatching
- **Learn:** How to detect at runtime: OS, CPU features (AVX2/AVX-512/CRC32/POPCNT support), NUMA topology, available cores. Why Intel and AMD have different SIMD capabilities (Intel: AVX-512 on Xeon, AMD: AVX2 on EPYC until Zen 4). How to wire CPU-specific code paths at boot so the engine uses the best available instructions on each chip. Why generic code loses 90% of the CPU's true power.
- **Test:** PlatformDetector returns correct values on current machine. Detects Intel vs AMD. Reports available instruction sets (AVX2, AVX-512, CRC32, POPCNT, AES-NI).
- **Build:** PlatformDetector (feature detection + CPU dispatch table). At boot, selects optimal code paths: AVX-512 path (Intel Xeon), AVX2 path (AMD EPYC/older Intel), scalar fallback (dev machines).

### Concept 1.7: OS Jitter Eradication
- **Learn:** What OS jitter is — the kernel interrupting your pinned thread to run background tasks (1,000-5,000 ns per interruption). How the Linux scheduler tick works (1000 Hz = interrupt every 1ms). How CPU C-states work (deep sleep wake = 50,000 ns). What demand paging is (page fault = 2,000 ns).
- **Test:** Measure jitter on isolated core before and after tuning. Use `cyclictest` (Linux) to measure worst-case latency.
- **Build:** Linux boot configuration guide (nohz_full, isolcpus, intel_idle.max_cstate=0, transparent_hugepage=never). `mlockall()` P/Invoke wrapper to wire physical RAM at startup. Windows MMCSS thread registration via P/Invoke.
- **Prove:** cyclictest showing before/after jitter on isolated cores.

**Phase 1 Deliverables:**
- CacheLineConstants, CommandFrame, WalRecord
- MemoryArena (with 2MB and 1GB HugePage support)
- AffinityScheduler
- PlatformDetector
- OS Jitter configuration guide + mlockall/MMCSS P/Invoke wrappers
- All tests passing, all benchmarks documented

---

## Phase 2: The Single-Writer Engine (Execution Kernel)

*Goal: Process millions of commands per second without a single lock.*

### Concept 2.1: The Single-Writer Principle
- **Learn:** Why locks (Monitor, Mutex, Semaphore) destroy CPU pipelines. What cache line bouncing is. Why one thread = zero contention = maximum throughput.
- **Test:** Single-writer processes N commands in sequence; verify ordering.
- **Build:** ShardExecutor skeleton (the infinite loop).

### Concept 2.2: Ring Buffers (LMAX Disruptor)
- **Learn:** How a ring buffer passes data between threads using memory barriers instead of locks. What Volatile.Read/Write do. What Interlocked operations are. Producer-consumer without allocation. How `BitOperations.PopCount()` (POPCNT hardware instruction) counts set bits in 1 CPU cycle for lock-free slot availability checking — vs a 64-iteration software loop.
- **Test:** Single producer, single consumer. Verify ordering. Verify no data loss. Verify throughput under load. Verify POPCNT produces correct bit counts.
- **Build:** RingBuffer\<T\> (power-of-2 sized, cache-line-padded head/tail, memory barriers). Use `BitOperations.PopCount` and `BitOperations.LeadingZeroCount` for slot management.
- **Prove:** Benchmark ring buffer vs ConcurrentQueue vs BlockingCollection. Benchmark POPCNT vs software bit counting loop.

### Concept 2.3: Configurable Wait Strategies
- **Learn:** Why different ring buffer consumers need different latency/CPU trade-offs. How BusySpin burns a core but achieves nanosecond wake-up. How Yielding uses Thread.Yield() to share the core. How Sleeping uses Thread.Sleep(1) for low-CPU background consumers. How Blocking uses kernel events (epoll/WaitHandle) for the lowest CPU cost. Why LMAX offers 7 strategies and how to pick the right one per consumer.
- **Test:** Measure wake-up latency for each strategy: BusySpin (~ns), Yielding (~μs), Sleeping (~ms), Blocking (~ms). Verify consumer processes all events regardless of strategy. Measure CPU utilization per strategy.
- **Build:** IWaitStrategy interface. BusySpinWaitStrategy, YieldingWaitStrategy, SleepingWaitStrategy, BlockingWaitStrategy. Configurable per consumer via YAML.
- **Prove:** Benchmark: ShardExecutor with BusySpin vs Projection Dispatcher with Sleeping. Verify hot-path latency unchanged, background CPU usage drops.

### Concept 2.4: Consumer Dependency DAGs (Pipeline Orchestration)
- **Learn:** How LMAX arranges consumers in a directed acyclic graph. How gating works: consumer C reads upstream consumers A and B's sequence counters — won't proceed until both have processed that sequence. How this enables parallel journaling + replication, then gated execution. How SequenceBarrier holds references to dependent sequences.
- **Test:** Build a 3-stage pipeline: [Journaler, Replicator] → [ShardExecutor] → [Projector, EventEmitter]. Verify: Journaler and Replicator run in parallel. ShardExecutor only starts after both complete. Projector and EventEmitter run in parallel after execution. Verify ordering is preserved across all stages.
- **Build:** SequenceBarrier (tracks dependent consumer sequences). ConsumerDagBuilder (constructs pipeline from YAML stage definitions). Integrate into RingBuffer.
- **Prove:** Benchmark: DAG pipeline vs single-consumer sequential. Show parallel stages improve throughput without breaking ordering.

### Concept 2.5: Back-Pressure Policies
- **Learn:** What happens when the ring buffer is full (producer has lapped slowest consumer). Why blocking is safest (LMAX default — never lose data). Why reject-with-error is better for latency-sensitive systems. Why drop-oldest is acceptable for telemetry. How per-tenant back-pressure prevents noisy-neighbor problems.
- **Test:** Fill ring buffer to capacity. Verify: Block policy makes producer wait. Reject policy returns error to client. DropOldest overwrites oldest unprocessed. Per-tenant: noisy tenant gets rejected while critical tenant gets blocked.
- **Build:** IBackPressurePolicy interface. BlockPolicy, RejectPolicy, DropOldestPolicy. Integrate with RingBuffer producer path. Per-tenant policy lookup in TenantManager.
- **Prove:** Benchmark: producer throughput under back-pressure with each policy. Verify zero data loss with Block policy.

### Concept 2.6: Idempotency Keys (Exactly-Once Execution)
- **Learn:** Why network retries cause duplicate commands. How TigerBeetle uses client-generated 128-bit IDs as idempotency keys. How a Bloom filter provides O(1) probable duplicate detection (~1 ns). How a verification step against an idempotency log confirms true duplicates. How the idempotency window (time-based eviction) bounds memory usage.
- **Test:** Submit command with ID X → succeeds. Resubmit same ID X → returns cached result, not re-executed. Verify entity state unchanged on duplicate. Submit different ID Y → succeeds normally. After window expiry, old keys are evicted.
- **Build:** BloomFilter (fixed-size, zero-alloc in MemoryArena). IdempotencyLog (circular buffer of last N processed keys per shard). Integrate into ShardExecutor: check bloom → verify log → execute or return cached.
- **Prove:** Benchmark: overhead of idempotency check per command (target: < 5 ns). Verify bloom filter false positive rate matches configured threshold.

### Concept 2.7: Two-Phase Transactions (Pending → Post/Void/Timeout)
- **Learn:** Why payments, escrow, and inventory reservations need a "hold then confirm" pattern. How TigerBeetle implements pending → post/void/timeout. How pending amounts are immediately reflected in balance checks (prevents overdraft during hold). How partial post works (confirm less than reserved). How timeout-based auto-void prevents dangling holds. How all transitions are WAL-logged for replay safety.
- **Test:** Reserve 500 → verify PendingDebit = 500. Post 300 → verify CommittedDebit += 300, PendingDebit -= 300, remaining 200 released. Void a pending → verify full release. Let timeout expire → verify auto-void fires. Attempt to post after void → verify rejection. Replay WAL → verify identical state.
- **Build:** TwoPhaseTransactionManager (tracks pending holds per entity). PendingTimeoutScheduler (engine-clock-driven, deterministic). Integrate with ShardExecutor and YAML command definitions (`type: "two_phase"`, `phase: "pending|post|void"`).
- **Prove:** Benchmark: overhead of two-phase vs single-phase command (~5-10 ns additional for pending lookup). Stress test: 100K concurrent pending holds with interleaved posts/voids/timeouts.

### Concept 2.8: Software Prefetching & Ring Buffer Lookahead
- **Learn:** Why a cache miss costs 60-100 ns (waiting for RAM). How the CPU prefetch instruction fetches data into L1 cache *in the background* while current work continues. How the ring buffer gives us a "peek ahead" — while processing command N, we prefetch command N+1's target entity. By the time we need it, it's in L1 (1 ns instead of 100 ns).
- **Test:** Process 1M commands with and without prefetch. Verify correctness is identical. Measure L1 cache hit rate.
- **Build:** PrefetchHelper (Sse.Prefetch0 wrapper). Integrate lookahead into RingBuffer consumer loop.
- **Prove:** BenchmarkDotNet + perf stat: compare cache miss rates and latency with/without prefetch. This is the single most impactful technique for hitting 20 ns.

### Concept 2.9: Branch Prediction & Branchless Code
- **Learn:** How the CPU predicts which `if` branch to take. What happens on misprediction (~15ns pipeline flush). How to write branchless code using bitwise operations and CMOV.
- **Test:** Branchless min/max/clamp produce correct results.
- **Build:** Branchless utility methods for common command routing decisions.
- **Prove:** BenchmarkDotNet: branchy vs branchless on random input.

### Concept 2.10: Transaction Context & Lifecycle
- **Learn:** What Prepare → Execute → Commit means. Why TransactionContext is immutable (except phase transition). How the engine ensures atomicity.
- **Test:** Transaction phases transition correctly. Compensate path works. Idempotency keys reject duplicates.
- **Build:** TransactionContext, transaction lifecycle in ShardExecutor.

### Concept 2.11: YAML Command Compiler
- **Learn:** How Expression Trees work in C#. How to compile YAML-defined mutations into cached Func\<\> delegates. Why compiled delegates are as fast as hand-written code.
- **Test:** Compile a YAML mutation, execute it, verify state change. Verify no allocations during execution.
- **Build:** YamlCommandCompiler (YAML → Expression Tree → cached delegate).
- **Prove:** Benchmark compiled YAML handler vs hand-written handler. Should be identical.

**Phase 2 Deliverables:**
- ShardExecutor (single-writer infinite loop)
- RingBuffer (LMAX-style, cache-line-padded, power-of-2)
- WaitStrategy implementations (BusySpin, Yielding, Sleeping, Blocking)
- ConsumerDagBuilder + SequenceBarrier (pipeline orchestration)
- BackPressurePolicy implementations (Block, Reject, DropOldest)
- BloomFilter + IdempotencyLog (exactly-once execution)
- TwoPhaseTransactionManager + PendingTimeoutScheduler
- PrefetchHelper (ring buffer lookahead prefetching)
- TransactionContext
- YamlCommandCompiler
- Branchless utilities
- All tests passing

---

## Phase 3: The Safety Net (Durability & WAL)

*Goal: Save data to disk without waiting for the disk.*

### Concept 3.1: Page Cache vs Direct I/O
- **Learn:** What the OS page cache does (double-buffering). Why it adds latency and unpredictability. How O_DIRECT bypasses it. How io_uring batches I/O syscalls.
- **Test:** Write N WalRecords, read them back, verify integrity (checksums match).
- **Build:** WalSegmentWriter (append-only, sequential writes).

### Concept 3.2: Sequential NVMe Writes & Group Commit
- **Learn:** How NVMe drives work internally (pages, channels, parallelism). Why sequential writes are 10-100x faster than random. Why batching 1000 commands into one fsync beats 1000 individual fsyncs.
- **Test:** Group commit batches correctly. Verify no data loss on simulated crash (truncate file mid-write, replay).
- **Build:** GroupCommitBatcher (coalesce commands, single fsync).
- **Prove:** Benchmark: individual fsync vs group commit. Measure IOPS.

### Concept 3.3: Hash Chain Integrity
- **Learn:** How PrevChecksum in WalRecord creates a tamper-evident chain. How torn writes (power failure mid-write) are detected by chain break.
- **Test:** Corrupt a byte in a WAL segment; verify replayer detects it. Truncate mid-record; verify clean truncation.
- **Build:** Checksum computation using Sse42.Crc32 hardware intrinsic (single cycle per 8 bytes, ~3 ns per cache line). AES-NI (Aes.Encrypt) as alternative for non-SSE4.2 platforms. Chain validation logic.
- **Prove:** Benchmark CRC32 hardware vs software CRC vs SHA256. Verify chain detects single-bit corruption.

### Concept 3.4: Deterministic Recovery
- **Learn:** Why WAL stores commands (not state mutations). Why replaying commands through ShardExecutor guarantees identical state. Why this enables point-in-time recovery.
- **Test:** Process 10,000 commands. Snapshot. Process 10,000 more. Simulate crash. Recover from snapshot + WAL replay. Verify state matches exactly.
- **Build:** DeterministicReplayer, SnapshotManager.
- **Prove:** Measure recovery speed (target: millions of ops/sec replay).

### Concept 3.5: WAL Compaction & Tiered Storage
- **Learn:** Why unbounded WAL growth is unsustainable. Key-based compaction (keep latest per AggregateId). Time-based retention. Moving cold segments to object storage.
- **Test:** Compact a WAL with duplicate keys; verify only latest survives. Archive old segments; verify transparent retrieval.
- **Build:** WalCompactor, TieredStorageManager.

### Concept 3.6: Offline WAL Compiler (Bulk Data Loading)
- **Learn:** Why sending billions of records through the network gateway would take days. How to compile CSV/SQL dumps directly into binary WalRecord format at NVMe write speed (~5 GB/sec). How the engine replays this pre-built WAL at startup to load billions of records in seconds.
- **Test:** Compile 1 million records from CSV into WAL file. Boot engine with this WAL. Verify all records loaded correctly. Verify checksum chain integrity.
- **Build:** OfflineWalCompiler (standalone tool: reads CSV/SQL → writes binary WAL segments).

### Concept 3.7: Projection Dispatcher (Async Database Sync)
- **Learn:** Why external systems (SQL Server, PostgreSQL, MongoDB) need the data but can't query the engine directly. How cursor-based WAL consumption works. Why bulk writes (TVP, COPY, bulk insert) give millions of rows/sec throughput. Why projections are eventually consistent (~100-500ms behind) and that's acceptable.
- **Test:** Commit 10,000 commands. Verify projection writes them all to a test database. Kill projection mid-stream; restart; verify it resumes from cursor with no duplicates. Add a new projection target; replay from WAL beginning; verify full rebuild.
- **Build:** ProjectionDispatcher, IProjectionTarget interface, ProjectionCursorStore. Initial adapters: SqlServerTarget (TVP), PostgresTarget (COPY).
- **Prove:** Measure projection throughput (target: millions of rows/sec bulk write).

### Concept 3.8: Event Interceptor Pipeline (IProjectionInterceptor)
- **Learn:** How the middleware pipeline pattern works — ASP.NET Core-style request pipeline applied to event projection. How each interceptor in the chain can inspect, transform, drop, or short-circuit events before they reach projection targets. How pipeline ordering guarantees deterministic behavior. How per-interceptor error policies (Skip, Retry, Dead-letter, Circuit-breaker) isolate failures so one bad interceptor cannot take down the entire projection path.
- **Test:** Pipeline ordering is respected (interceptor A runs before B). Drop behavior works (dropped events never reach targets). Error policy enforcement (Skip continues pipeline, Retry retries N times, Dead-letter routes to DLQ, Circuit-breaker opens after threshold). Interceptor isolation (user interceptor crash does not affect core engine or other interceptors).
- **Build:** IProjectionInterceptor interface (OnEvent, OnError, Priority). InterceptorPipeline executor (ordered chain, short-circuit support). InterceptResult/InterceptAction (Continue, Drop, Transform, ShortCircuit). Per-interceptor error policies configurable via YAML (`error_policy: skip|retry|dead_letter|circuit_breaker`). YAML configuration for pipeline ordering and interceptor registration.
- **Prove:** User interceptor that throws does not crash engine or block other projections. Pipeline order is respected under concurrent load. Dropped events are confirmed absent from all downstream targets.
- **Deliverable:** Working interceptor middleware in projection path.

**Phase 3 Deliverables:**
- WalSegmentWriter (O_DIRECT / io_uring)
- GroupCommitBatcher
- DeterministicReplayer
- SnapshotManager
- WalCompactor
- OfflineWalCompiler (bulk data loading tool)
- ProjectionDispatcher + IProjectionTarget adapters
- InterceptorPipeline + IProjectionInterceptor (event interceptor middleware)
- All tests passing, recovery verified

---

## Phase 4: The Wire (Kernel Bypass Networking)

*Goal: Read network packets without the OS knowing they arrived.*

### Concept 4.1: TCP/IP Stack Overhead
- **Learn:** What happens during a standard network receive: interrupt → kernel → context switch → copy to user space → wake thread. Why this costs 50-100μs.
- **Test:** Benchmark standard Socket receive latency as baseline.

### Concept 4.2: DPDK Poll-Mode Driver
- **Learn:** How DPDK disconnects the NIC from the kernel. How poll-mode drivers (PMD) work. How RSS (Receive Side Scaling) distributes packets to specific cores. How Flow Director (FDIR) filters at hardware level.
- **Test:** Receive a packet via DPDK; verify it arrives in our MemoryArena.
- **Build:** KernelBypassListener (P/Invoke wrappers for DPDK PMD).
- **Prove:** Benchmark DPDK receive vs standard Socket.

### Concept 4.3: Registered I/O (Windows)
- **Learn:** How RIO pre-registers buffers with the kernel, reducing per-operation overhead. The Windows equivalent of high-performance sockets.
- **Test:** Receive a packet via RIO; verify correctness.
- **Build:** RioListener (Windows platform adapter).

### Concept 4.4: Zero-Copy Deserialization
- **Learn:** How FlatBuffers maps binary wire data onto struct fields without copying or parsing. Why Protobuf is slower (requires deserialization/allocation). How we cast a raw byte* directly to a CommandFrame ref.
- **Test:** Deserialize a binary packet into CommandFrame; verify all fields correct; verify zero allocations.
- **Build:** ZeroCopyDeserializer, FlatBuffers schema for CommandFrame.
- **Prove:** Benchmark FlatBuffers vs Protobuf vs JSON deserialization.

### Concept 4.5: RDMA for Replication
- **Learn:** How RDMA writes directly into remote machine's RAM without involving remote CPU. How memory registration works (ibv_reg_mr). How Soft-RoCE enables development without hardware.
- **Test:** Write a WalRecord from Machine A into Machine B's MemoryArena. Verify it arrives intact.
- **Build:** RdmaReplicator (P/Invoke to libibverbs). Soft-RoCE dev setup.
- **Prove:** Benchmark RDMA replication latency vs TCP-based replication.

### Concept 4.6: eBPF/XDP Pre-Filtering
- **Learn:** How eBPF allows running miniature programs inside the Linux NIC driver, before the kernel allocates memory for the packet. How XDP (eXpress Data Path) validates or drops packets in ~10-15 ns — before they even reach our DPDK poll loop. Why this eliminates noise (invalid packets, DDoS) from ever consuming engine CPU cycles.
- **Test:** Send valid VectorVolt packets and garbage packets. Verify XDP drops garbage in <15 ns. Verify valid packets reach DPDK listener.
- **Build:** XdpPreFilter (eBPF program for packet validation, loaded at startup).
- **Prove:** Benchmark latency jitter with/without XDP under mixed traffic (valid + noise).

### Concept 4.7: NIC Flow Steering
- **Learn:** How to program the NIC to filter by port/protocol. How RSS hashes packets to specific CPU cores. How to drop unwanted traffic in hardware.
- **Test:** Send VectorVolt traffic and non-VectorVolt traffic; verify only ours reaches the engine.
- **Build:** FlowSteeringConfigurator.

### Concept 4.8: Precision Time Protocol (PTP / IEEE 1588)
- **Learn:** Why NTP drifts by milliseconds (useless at nanosecond scale). How PTP uses NIC hardware clocks (PHC) synchronized to <100 ns across the cluster. How the NIC stamps packets at the instant the first electron hits the wire. Why this gives us global transaction ordering without a central coordinator.
- **Test:** Read hardware clock via P/Invoke; verify it returns monotonic, high-resolution timestamps. Compare precision vs DateTime.UtcNow.
- **Build:** PtpClockReader (P/Invoke to NIC PHC). Update CommandFrame.Timestamp source from DateTime.UtcNow to PTP when available. Fallback to DateTime.UtcNow on dev machines.
- **Prove:** Measure clock precision: PTP (<100 ns drift) vs NTP (~10 ms drift).

### Concept 4.9: Cloud Network Adapters
- **Learn:** How AWS EFA, Azure InfiniBand, Oracle RDMA, and GCP gVNIC differ. How to write one INetworkTransport interface that adapts to each. How PlatformDetector selects the best transport at startup.
- **Test:** INetworkTransport abstraction works with StandardSocketTransport (fallback). Mock tests for cloud adapters.
- **Build:** INetworkTransport, EfaTransport, InfiniBandTransport, StandardSocketTransport.

**Phase 4 Deliverables:**
- XdpPreFilter (eBPF/XDP pre-filtering, Linux)
- KernelBypassListener (DPDK on Linux)
- RioListener (RIO on Windows)
- ZeroCopyDeserializer (FlatBuffers)
- RdmaReplicator
- FlowSteeringConfigurator
- PtpClockReader (NIC hardware timestamps)
- INetworkTransport + cloud adapters (EFA, InfiniBand, gVNIC, fallback)
- All tests passing

---

## Phase 5: The Read Fabric & Vectorization

*Goal: Query data instantly without slowing down the write engine.*

### Concept 5.1: CQRS & Read/Write Separation
- **Learn:** Why reads and writes have fundamentally different access patterns. Why optimizing for both simultaneously is impossible. How CQRS separates them.
- **Test:** Write path and read path operate independently. Write latency unaffected by concurrent reads.

### Concept 5.2: Document Mode (Denormalized Reads)
- **Learn:** Why relational JOINs cause pointer chasing (60-100ns per hop). How pre-joining data into contiguous blocks eliminates this. How the write path maintains denormalized copies.
- **Test:** Store a Transaction with Details, Type, and Attributes as one block. Retrieve by ID; verify all data present in one fetch.
- **Build:** DocumentStore (pre-joined, contiguous memory blocks).

### Concept 5.3: Columnar Mode (SIMD Scans)
- **Learn:** Why row-oriented storage is slow for scans (cache loads unwanted fields). How column-oriented storage keeps each field contiguous. Why this enables SIMD.
- **Test:** Store 1000 transactions in columnar format. Scan Amount column; verify correct results.
- **Build:** ColumnarStore (field arrays in contiguous memory).

### Concept 5.4: SIMD/AVX2 Vectorized Scanning
- **Learn:** What SIMD registers are (XMM=128-bit, YMM=256-bit). How Vector256\<int\>.GreaterThan compares 8 values in one cycle. How to create result bitmasks. Alignment requirements (32-byte for AVX2).
- **Test:** AVX2 scan of 1 million integers finds correct matches. Results identical to scalar scan.
- **Build:** VectorScanner (AVX2 accelerated column scanning).
- **Prove:** Benchmark AVX2 scan vs scalar loop on 1M, 10M, 100M records.

### Concept 5.5: Compiled Query Filter Engine
- **Learn:** How Expression Trees represent code as data. How to compile them into Func\<\> delegates. How to cache compiled delegates for reuse. How this enables zero-GC filtering.
- **Test:** Compile "Amount > 500 AND Status == Placed". Execute on columnar data. Verify results.
- **Build:** QueryFilterEngine (Expression Tree → delegate → columnar scan).
- **Prove:** Benchmark compiled filter vs LINQ vs manual loop.

### Concept 5.6: Tiered Read Routing
- **Learn:** How the engine determines which tier holds the requested data (hot/warm/cold). How transparent routing works from the client's perspective.
- **Test:** Query for recent data → hot. Query for old data → warm/cold. Verify correct routing.
- **Build:** TieredRouter.

### Concept 5.7: Materialized Views / Continuous Aggregations
- **Learn:** How VoltDB maintains materialized views incrementally as transactions execute — not batch-recomputed. How an atomic command can update both the entity and all dependent aggregate views in the same operation. How pre-computed COUNT/SUM/AVG/MIN/MAX per group key eliminates scan-time aggregation. How views are stored as compact structs in MemoryArena (one struct per group key). How views survive restart via deterministic WAL replay.
- **Test:** Define a "OrdersByStatus" view (group by Status, count + sum Amount). Insert 10K orders. Verify view values match manual aggregation. Update an order's status → verify view updates atomically. Kill and restart → verify view state matches after WAL replay.
- **Build:** MaterializedViewEngine (YAML-defined views, incremental update logic). ViewStore (compact structs in MemoryArena, queryable via Read Fabric). Integrate into ShardExecutor: on entity mutation, update all dependent views in same atomic operation.
- **Prove:** Benchmark: query pre-computed view (~5 ns lookup) vs full columnar scan + aggregation (~μs). Verify zero additional allocations for view maintenance.

**Note:** Prefetching was moved to Phase 2 (Concept 2.8) because it pairs directly with the ring buffer lookahead. In this phase, we extend prefetch hints into VectorScanner and index traversal paths built here.

**Phase 5 Deliverables:**
- DocumentStore (denormalized by-ID reads)
- ColumnarStore (SIMD-friendly column arrays)
- VectorScanner (AVX2 hardware-accelerated, with prefetch hints from Phase 2)
- QueryFilterEngine (Expression Trees → cached delegates)
- TieredRouter (hot → warm → cold)
- MaterializedViewEngine + ViewStore (continuous aggregations)
- All tests passing, benchmarks documented

---

## Phase 6: The Control Plane & Multi-Tenancy

*Goal: Make VectorVolt a configurable "Data OS" for application developers.*

### Concept 6.1: YAML Topology Parsing
- **Learn:** How the YAML configuration maps to internal engine structures. How to validate YAML at startup. How to detect conflicts (same entity in two shards, etc.).
- **Test:** Parse valid YAML → correct internal topology. Reject invalid YAML with clear error.
- **Build:** YamlTopologyParser.

### Concept 6.2: Shard Catalog & Policy Engine
- **Learn:** What the Shard Catalog stores (authoritative metadata). How the Policy Engine enforces rules (throughput limits, retention, rebalancing). How policies are versioned and auditable.
- **Test:** Policy rejects over-quota tenant. Rebalancing triggers correctly on node add/remove.
- **Build:** ShardCatalog, PolicyEngine.

### Concept 6.3: Consistent Hashing (Rendezvous)
- **Learn:** The math behind rendezvous hashing. Why it distributes data evenly. Why it minimizes redistribution when nodes are added/removed (only 1/N keys move).
- **Test:** Distribution is even across shards. Adding a shard moves minimal keys.
- **Build:** ConsistentHashRouter.
- **Prove:** Visualize distribution uniformity.

### Concept 6.4: Multi-Tenancy
- **Learn:** Shared vs dedicated shard isolation. Per-tenant encryption, RBAC, quotas. How tenant boundaries are preserved during replay and failover.
- **Test:** Tenant A cannot see Tenant B's data. Quota enforcement rejects excess. Replay preserves boundaries.
- **Build:** TenantManager, tenant-aware routing.

### Concept 6.5: Epoch Fencing & Failover
- **Learn:** What split-brain is (two leaders think they're in charge). How epoch numbers prevent stale leaders from writing. How follower promotion works.
- **Test:** Simulate leader failure. Follower promotes. Stale leader's writes are rejected (epoch too low).
- **Build:** EpochFencingManager, FailoverCoordinator.

### Concept 6.6: Command Gateway & Polyglot Client SDKs
- **Learn:** How the Gateway is the GC/translation boundary — all client protocols (gRPC, Raw TCP, HTTP/REST) terminate here. How Protocol Buffers define the wire protocol once and generate type-safe clients for every language (C#, Java, Python, Node.js, Go, Rust, C++). How the Gateway deserializes protobuf → packs into MemoryArena → constructs 64-byte CommandFrame → pushes to Ring Buffer. Why YAML (boot-time engine behavior) and SDKs (request-time client communication) are completely independent layers. How the SDK handles leader discovery, shard routing, reconnection on failover, tenant authentication (JWT/mTLS), and retry with exponential backoff.
- **Test:** Java client submits command via gRPC → arrives as valid CommandFrame. Rust client submits via Raw TCP → arrives as identical CommandFrame. Python client via HTTP/REST → same. Verify: kernel cannot distinguish source language. Multi-tenant test: Tenant A (Java) and Tenant B (Node.js) hit same cluster, verify shard isolation. On failover, all SDKs reconnect to new leader within configured timeout.
- **Build:** CommandGateway (unified entry point), ProtobufTranslator (protobuf → CommandFrame), GrpcEndpoint, RawTcpEndpoint, HttpRestEndpoint. Protobuf schema (`vectorvolt.proto`). Tier 1 SDKs: C#, Java, Python, Node.js, Go, Rust, C++ (thin wrappers ~200-500 lines each over generated gRPC client).
- **Prove:** Benchmark per-protocol overhead: Raw TCP (~1-5 μs), gRPC (~10-50 μs), HTTP/REST (~50-500 μs). Verify kernel latency unchanged (~20 ns) regardless of client language.

### Concept 6.7: Schema Registry
- **Learn:** How schema versioning works. Backward/forward compatibility rules. Why replay-unsafe schema changes must be rejected.
- **Test:** Compatible schema change accepted. Incompatible change rejected.
- **Build:** SchemaRegistry.

### Concept 6.8: Auto-Scaling & Shard Splitting
- **Learn:** What triggers auto-scaling (CPU threshold, memory budget, queue depth). How shard splitting works (re-hash a subset of keys to a new shard). How to detect a "hot shard" and split it without downtime.
- **Test:** Shard hits CPU threshold → Policy Engine triggers split. Keys redistribute correctly. No data loss during split.
- **Build:** AutoScaleManager, ShardSplitter.

### Concept 6.9: Live Shard Migration
- **Learn:** How to move a live shard between nodes without downtime. The protocol: pause → snapshot → transfer → WAL catchup → switchover. How to drain a node gracefully before shutdown.
- **Test:** Migrate shard from Node 1 to Node 2. Verify zero data loss. Verify zero downtime (clients reconnect transparently). Drain a node; verify all shards moved.
- **Build:** ShardMigrator, DrainCoordinator.

### Concept 6.10: Zero-Overhead Observability
- **Learn:** Why standard logging (Serilog, Console.WriteLine) destroys nanosecond latency (string allocations, GC, disk I/O). How to use CPU Time Stamp Counter (TSC) for sub-nanosecond timing. How a telemetry ring buffer lets the hot path emit metrics with a single 8-byte integer write (~1 ns). How a shadow thread on a separate core exports to Prometheus/Grafana without touching the hot path.
- **Test:** ShardExecutor emits TSC timestamps. Shadow thread reads them. Verify zero allocations on hot path during metric emission. Verify metrics appear in Prometheus format.
- **Build:** TscMetricsBuffer (lock-free telemetry ring buffer using X86Base.ReadTimeStampCounter for sub-nanosecond timestamps — ~1 ns per read, no syscall). ShadowMetricsExporter (Prometheus/Grafana/OpenTelemetry export on separate core). TSC calibration routine (RDTSC ticks → nanoseconds using invariant TSC frequency from CPUID).
- **Prove:** Benchmark hot path latency with and without telemetry enabled. Difference should be <2 ns. Verify RDTSC overhead is ≤1 ns.

### Concept 6.11: Health Monitoring & Failure Detection
- **Learn:** How heartbeat protocols work. How to distinguish "slow node" from "dead node." How failure detection timeout relates to failover speed. Why false positives (declaring a healthy node dead) are dangerous.
- **Test:** Kill a node; verify detection within configured timeout. Simulate slow node; verify it's not falsely declared dead.
- **Build:** HealthMonitor (heartbeat protocol).

### Concept 6.12: VectorVolt CLI & Interactive Dashboard (`vvctl`)
- **Learn:** How production distributed systems provide operational tooling (kubectl, redis-cli, kafka-topics.sh, cockroach CLI, k9s). How System.CommandLine parses commands/subcommands/flags in .NET. How Spectre.Console / Terminal.Gui renders live-updating TUI dashboards. How the CLI communicates with the cluster via the same gRPC protocol as client SDKs (no special admin backdoor). How to read TscMetricsBuffer output from ShadowMetricsExporter for live metrics display with zero hot-path impact.
- **Build:**
  1. **CLI Framework:** VvctlApp using System.CommandLine — modular command structure (each feature is a Command class).
  2. **Cluster Commands:** `vvctl status`, `vvctl nodes`, `vvctl shards`, `vvctl health` — connect to cluster via gRPC, display formatted output.
  3. **Entity Commands:** `vvctl entities`, `vvctl get <Type> <Id>`, `vvctl query "<filter>"` — execute queries against Read Fabric from terminal.
  4. **WAL Commands:** `vvctl wal tail --shard <id>`, `vvctl wal inspect --segment <id>` — live WAL streaming and integrity inspection.
  5. **Metrics Commands:** `vvctl metrics`, `vvctl bench`, `vvctl profile --shard <id>`, `vvctl cores` — live latency, hardware benchmarks, TSC profiling, core-to-shard mapping.
  6. **Projection Commands:** `vvctl projections`, `vvctl projection pause|resume|replay` — projection target management.
  7. **Ops Commands:** `vvctl snapshot`, `vvctl drain`, `vvctl failover`, `vvctl rebalance`, `vvctl logs` — operational controls.
  8. **Config Commands:** `vvctl init`, `vvctl apply`, `vvctl validate`, `vvctl tenant`, `vvctl schema` — setup and configuration.
  9. **Migration Commands:** `vvctl migrate analyze|import|shadow|cdc` — legacy migration toolkit from terminal.
  10. **TUI Dashboard:** `vvctl dashboard` — full-screen live dashboard with panels for cluster map, shard topology, latency histograms, WAL throughput, core assignments. Keyboard navigation to drill into nodes → shards → entities. Color-coded health indicators.
- **Test:** Launch cluster. Run every `vvctl` command, verify correct output. Kill a node, verify `vvctl status` reflects it within heartbeat timeout. Run `vvctl query`, verify results match direct SDK query. Run `vvctl dashboard`, verify live-updating metrics match Prometheus export. Run `vvctl migrate analyze` on a sample DLL, verify report matches expected classification.
- **Prove:** Benchmark: `vvctl` overhead on cluster — verify zero impact on hot-path latency (CLI reads from ShadowMetricsExporter, never from engine thread). Measure CLI response time for all commands (target: < 100ms for status/health, < 1s for queries/analysis).

### Concept 6.13: Real-Time Web Monitoring Dashboard
- **Physics:** Lock-free atomic counters (Interlocked.Increment, ~1 ns), memory visibility across threads, WebSocket protocol for real-time push, separation of metrics collection (hot path) from metrics reading (dashboard thread).
- **Test:** Counter increments don't affect hot-path latency (benchmark proof), dashboard receives correct values, WebSocket connection handles reconnect, multiple browser clients don't affect engine.
- **Build:**
  1. **MetricsRegistry:** shared struct with Interlocked long counters for every metric (ops/sec per shard, WAL bytes/sec, projection lag, etc.).
  2. **MetricsCollector:** reads MetricsRegistry every 100ms, computes deltas and rates.
  3. **MetricsHttpServer:** Kestrel minimal API serving dashboard HTML + WebSocket endpoint.
  4. **DashboardWebSocketHandler:** pushes JSON metrics snapshot to all connected browsers.
  5. **Dashboard HTML/JS:** animated architecture visualization with live counters, data flow animations, per-shard breakdown, WAL status, projection lag.
- **Prove:** Dashboard shows real metrics from running engine without measurable latency impact.
- **Deliverable:** Browser-based live monitoring dashboard at http://node:port/dashboard

### Concept 6.14: Workload Prioritization / QoS Tiers
- **Learn:** How ScyllaDB assigns service levels with shares (1-1000) per role. How proportional share scheduling works: tier gets `(its shares / total shares) * 100%` of resources when contention exists, 100% when no contention. How to classify workloads: interactive (OLTP, latency-sensitive) vs batch (OLAP, throughput-oriented). How per-shard CPU schedulers enforce shares at the ring buffer consumer level. How Read Fabric scans from batch-tier tenants yield to interactive-tier queries.
- **Test:** Configure critical tier (1000 shares) and batch tier (10 shares). Run both concurrently. Verify: critical-tier latency stays under 50 ns even when batch-tier runs heavy scans. When only critical tier is active, it gets 100% resources. When both active, critical gets ~99% under contention.
- **Build:** WorkloadScheduler (per-shard, share-based priority). WorkloadTier definitions in YAML. Integrate with RingBuffer consumer scheduling and Read Fabric scan scheduling. Per-tenant tier assignment in TenantManager.
- **Prove:** Benchmark: OLTP latency with and without concurrent OLAP scan. Show < 5% latency degradation for critical tier even under heavy batch load.

### Concept 6.15: Cluster-Wide Versionstamps (Global Ordering Tokens)
- **Learn:** How FoundationDB generates cluster-wide monotonic 10-byte versionstamps (8-byte version + 2-byte sequence). Why per-shard sequence numbers are insufficient for cross-shard ordering. How the cluster leader assigns monotonic versions at commit time. How versionstamps enable: total ordering of all mutations across shards, cross-shard change feeds with guaranteed ordering, provable "event A before event B" audit trails. The trade-off: versionstamp assignment requires a leader round-trip (~μs), so only use for cross-shard scenarios. Per-shard SequenceNumber remains for intra-shard ordering (zero overhead).
- **Test:** Two shards process commands concurrently. Each committed transaction receives a versionstamp. Verify: versionstamps are globally monotonic. Cross-shard event feed ordered by versionstamp produces correct causal ordering. Leader failover preserves monotonicity (new leader starts from higher version).
- **Build:** VersionstampGenerator (on cluster leader). VersionstampField (10-byte struct, embeddable in entities via YAML `type: "versionstamp"`). Cross-shard change feed ordered by versionstamp.
- **Prove:** Benchmark: overhead of versionstamp assignment per transaction (~1-5 μs leader round-trip). Verify: intra-shard operations without versionstamp remain at ~20 ns (no regression).

**Phase 6 Deliverables:**
- YamlTopologyParser
- ShardCatalog, PolicyEngine
- ConsistentHashRouter
- TenantManager
- EpochFencingManager, FailoverCoordinator
- CommandGateway, ProtobufTranslator, GrpcEndpoint, RawTcpEndpoint, HttpRestEndpoint, Tier 1 Polyglot SDKs
- SchemaRegistry
- AutoScaleManager, ShardSplitter
- ShardMigrator, DrainCoordinator
- TscMetricsBuffer, ShadowMetricsExporter
- HealthMonitor
- WorkloadScheduler (QoS tiers)
- VersionstampGenerator (global ordering)
- VvctlApp (CLI + TUI Dashboard)
- MetricsRegistry, MetricsCollector, MetricsHttpServer, DashboardWebSocketHandler (Real-Time Web Dashboard)
- All tests passing

---

## Phase 7: Testing, Sample Application & Research

*Goal: Prove VectorVolt works at scale, survives chaos, and document everything for research.*

### 7.1 Deterministic Simulation Testing (DST)
- **Learn:** How FoundationDB and TigerBeetle find all bugs before production. How to replace physical clock, network, and disk with mock hardware. How to run an entire cluster in a single thread. How to simulate millions of failure scenarios per hour.
- **Test:** Build the DST harness. Inject: power loss mid-WAL-write, disk corruption, network partition, split-brain, slow follower, full memory budget, cascading shard failures. Verify engine recovers correctly from every scenario.
- **Build:** DeterministicSimulator (mock clock, mock network, mock disk). FailureInjector (configurable failure scenarios). SimulationRunner (single-threaded cluster execution).
- **Prove:** Run 10 million failure scenarios. Report zero data loss, zero corruption, zero split-brain across all runs.

### 7.2 Sample Application Design
- Design a sample app that stress-tests every layer of VectorVolt.
- Must handle billions of records and thousands of req/sec.
- Must exercise: primary lookups, secondary indexes, range scans, filtered queries, cross-shard sagas, failover, projection to multiple databases.
- Specifics to be decided after Phases 1-6 (user will provide research input).

### 7.3 Load Testing
- BenchmarkDotNet microbenchmarks for every hot-path function.
- End-to-end load test: sustained thousands of req/sec.
- Measure: p50, p99, p99.9 latencies.
- Verify zero GC pauses under load (.NET diagnostics).
- Verify all data projected correctly to external databases.

### 7.4 Chaos Testing (Production Simulation)
- Kill leader mid-transaction → verify failover and zero data loss.
- Corrupt WAL segment → verify detection and clean recovery.
- Network partition → verify epoch fencing prevents split-brain.
- Fill memory budget → verify eviction to warm tier.
- Kill projection dispatcher → verify cursor-based resume with no duplicates.
- Add/remove nodes → verify auto-rebalance and shard migration.

### 7.5 Legacy Migration Toolkit

*Goal: Prove that existing .NET applications can migrate to VectorVolt without a big-bang rewrite.*

#### Concept 7.5.1: Command Gateway (Anti-Corruption Layer)
- **Learn:** The Strangler Fig Pattern — how Amazon, Uber, and every successful database migration works. Why legacy code (JSON, strings, Entity Framework) must never touch the nanosecond kernel. How an edge gateway translates between the "dirty zone" (GC-permitted, ~5-50 μs) and the "clean zone" (zero-alloc CommandFrames).
- **Build:** CommandGateway — accepts JSON/HTTP, deserializes, packs into MemoryArena, constructs CommandFrame, pushes to Ring Buffer. The gateway IS the GC boundary.
- **Test:** Send 10,000 JSON requests through Gateway. Verify: all arrive as valid CommandFrames, zero GC triggered inside the engine, gateway latency < 50 μs p99.
- **Prove:** Benchmark: legacy app → SQL Server vs legacy app → Gateway → VectorVolt → Projection → SQL Server. Show total latency is still dramatically lower.

#### Concept 7.5.2: Assembly Static Analyzer (Migration Assistant)
- **Learn:** How Mono.Cecil / System.Reflection.Metadata can read IL opcodes from compiled .NET assemblies. How to classify methods by their side-effect profile (pure math vs I/O-bound vs mixed). How call graph analysis reveals which methods are leaf functions (convertible) vs which depend on DbContext/HttpClient (not convertible).
- **Build:** MigrationAnalyzer — scans a .NET DLL and produces a report:
  - ✅ Auto-convertible: pure logic methods → generate YAML mutations or struct function stubs.
  - ⚠️ Mixed: logic + I/O tangled → flag for manual separation, show exact lines.
  - ❌ Gateway-only: pure I/O → leave in Gateway or DLL Sandbox.
- **Test:** Feed a sample legacy DLL with known method types. Verify report correctly classifies each method. Verify auto-generated YAML matches expected output.
- **Prove:** Run against a realistic 50-class legacy assembly. Measure: % auto-convertible, % needing manual work.

#### Concept 7.5.3: eBPF Shadow Mode (Zero-Touch Migration)
- **Learn:** How eBPF injects programs into the Linux kernel. How XDP runs at NIC driver level (Ring 0) before kernel memory allocation — filtering 20-30M packets/sec. Why TLS-encrypted packets are useless at the NIC level. How `uretprobe` hooks the return of `SSL_read()` inside the legacy app's OpenSSL library to capture decrypted plaintext from the app's own RAM. How the paired uprobe (entry) captures the buffer pointer from register `rsi`, stores in BPF_MAP_TYPE_HASH keyed by thread ID, and the uretprobe (return) reads it back. How eBPF Ring Buffers provide lock-free kernel→userspace data transfer.
- **Build:**
  1. XDP filter program (Restricted C): filter by TCP dest port + IP subnet, drop non-business traffic.
  2. uretprobe program (Restricted C): attach to `SSL_read` return, copy decrypted payload to eBPF ring buffer (4-16 MB, tunable, with drop counter).
  3. Compile both to eBPF bytecode via `clang -target bpf`.
  4. C# BpfRingBufferListener: P/Invoke to `libbpf.so`, `ring_buffer__new` + `ring_buffer__poll(rb, -1)` (epoll-based blocking, not polling), `[UnmanagedCallersOnly]` callback for zero-alloc interop.
  5. Shadow Mode comparator: run VectorVolt in parallel with legacy, compare outputs daily.
- **Test:** Deploy XDP filter on test server. Verify: SSH/ping packets dropped, HTTPS packets passed. Attach uretprobe to OpenSSL. Verify: decrypted JSON appears in ring buffer. Verify: legacy app performance impact < 5 μs per request.
- **Prove:** Run Shadow Mode for 7 days against simulated production traffic. Compare VectorVolt state vs legacy SQL state — verify mathematical equivalence. Measure uretprobe overhead at 1K, 10K, 100K req/sec.

#### Concept 7.5.4: CDC Reverse-Ingestion (Database-Level Migration)
- **Learn:** How Change Data Capture works — Debezium reads the database's internal WAL/transaction log at disk level. How every INSERT/UPDATE/DELETE becomes a change event. The ordering problem: cross-table transactions may arrive out of order (Debezium reads per-table WAL sequentially). How transaction-boundary markers reconstruct atomicity. The schema evolution problem: ALTER TABLE changes the CDC stream shape.
- **Build:**
  1. Debezium connector deployment (SQL Server / PostgreSQL).
  2. WalReplicator: consumes Debezium change events, translates SQL row changes → CommandFrames.
  3. CDC Schema Translator: maps SQL schema changes to VectorVolt entity updates (integrated with Schema Registry from Phase 6).
  4. Transaction boundary reconstruction: group CDC events by transaction ID, apply atomically.
- **Test:** Legacy app writes 10,000 transactions to SQL (multi-table). Verify: all arrive in VectorVolt in correct order with atomicity preserved. ALTER a table mid-stream — verify schema translator handles it. Measure CDC lag (target: < 500 ms).
- **Prove:** Build full in-memory replica of a 1M-row legacy database via CDC. Point read dashboards to VectorVolt. Verify query results match SQL exactly.

#### Concept 7.5.5: End-to-End Migration Demo
- **Learn:** How the full migration pipeline works: legacy app → Gateway → VectorVolt → Projection Dispatcher → legacy SQL (for read continuity). Understand the eventual consistency caveat (SQL reads lag by milliseconds). How all three strategies (Gateway, eBPF Shadow, CDC) compose together in a phased rollout.
- **Build:** Complete demo: take the sample app from 7.2, create a "legacy version" (EF + SQL), then migrate it step by step. Phase A: eBPF Shadow Mode (prove correctness). Phase B: CDC reverse-ingestion (build read replica). Phase C: Gateway cutover (redirect writes). Document each step as a migration playbook.
- **Prove:** Legacy screens continue working unchanged. Write latency drops from milliseconds to microseconds. Zero data loss during migration.

### 7.6 Reference Application Migration & Testbed
- **Physics:** CDC (Change Data Capture) mechanics — reading PostgreSQL/MySQL WAL, transforming SQL rows to CommandFrames; Java-to-VectorVolt communication via gRPC SDK.
- **Test:** Java app CRUD operations captured by CDC, ingested into VectorVolt, queryable via Read Fabric, visible in dashboard.
- **Build:**
  1. Select/arrange a business-heavy Java application (e-commerce, trading, banking, inventory).
  2. Must have: multiple entity types, complex business logic, existing relational DB, 10K+ records.
  3. Set up CDC pipeline: Java app → PostgreSQL → Debezium → VectorVolt WalReplicator.
  4. Configure Java SDK client for direct VectorVolt integration.
  5. Connect Live Dashboard to show real throughput from testbed.
  6. Run load test with vvctl loadgen against migrated data.
- **Prove:** Real Java application's data flowing through VectorVolt at target latencies, visible on live dashboard.
- **Deliverable:** Complete migration demo — Java app running alongside VectorVolt with live dashboard.

### 7.7 Platform Distribution & Package Management
- **Learn:** How NativeAOT compilation produces a single self-contained binary with no runtime dependency. How single binary deployment simplifies operations. How cross-platform packaging works for each target ecosystem (.deb for Debian/Ubuntu, .rpm for RHEL/Fedora, .msi for Windows, .pkg for macOS). How container images and Helm charts enable Kubernetes deployment. How package manager manifests (winget, Chocolatey, Homebrew) enable one-command installation.
- **Test:** NativeAOT binary runs on clean machines without .NET runtime installed. Package installs and uninstalls cleanly on each target platform (no leftover files, services, or registry entries). Verify binary starts and passes health check on each platform.
- **Build:** CI/CD pipeline producing: .deb package (Debian/Ubuntu), .rpm package (RHEL/Fedora), .msi installer (Windows), .pkg installer (macOS), Docker image (multi-arch), Helm chart (Kubernetes), winget manifest (Windows Package Manager), Chocolatey package (Windows), Homebrew formula (macOS/Linux).
- **Prove:** `winget install vectorvolt`, `apt install vectorvolt`, `docker pull vectorvolt`, `helm install vectorvolt` all work end-to-end on clean environments. Verify each package includes correct version metadata, license, and uninstall support.
- **Deliverable:** All platform packages published and installable.

### 7.8 Research Documentation
- Document every benchmark result with methodology.
- Compare against theoretical latency budget (15-22 ns target).
- Compare against existing systems (LMAX, TigerBeetle, FoundationDB, VoltDB).
- Prepare material suitable for PhD thesis / research paper.

---

## Phase 8 (Optional): Research Modules — PhD Differentiators

*These are independent research modules that can be built after the core engine is complete. Each one could be a chapter in a thesis or a standalone paper.*

### 8.1 Learned Indexes
- **Learn:** MIT 2018 "Learned Indexes" paper. How a linear regression model can predict memory offset from key in ~3 ns (vs B-Tree ~150 ns). When it works (stable, predictable distributions) and when it fails (highly skewed, rapidly changing data).
- **Build:** LearnedIndexModel (train on key distribution at snapshot time). Integrate as alternative to hash index.
- **Prove:** Benchmark learned index vs hash index vs B-Tree on VectorVolt data.

### 8.2 In-Network Compute (P4 / Programmable Switches)
- **Learn:** How Intel Tofino switches run P4 programs at line rate. How NOPaxos uses the switch to stamp global sequence numbers. How this eliminates consensus overhead entirely.
- **Build:** P4 program for packet sequencing (if hardware available). Update CommandFrame to accept switch-stamped SequenceNumber.
- **Prove:** Compare consensus latency: software Raft vs P4 switch-ordered.

### 8.3 Unikernel Deployment
- **Learn:** Why Protection Rings (Ring 0 vs Ring 3) cause ~1,000-2,000 ns per syscall. How a Unikernel eliminates this by running the app as the OS itself (everything in Ring 0). Why VectorVolt is an ideal candidate (zero-GC, self-contained, deterministic, external observability). The trade-offs: no SSH, no background agents, no arbitrary hardware drivers, 100% reliance on TscMetricsBuffer for observability.
- **Build:**
  1. Enable NativeAOT in .csproj (`PublishAot=true`, `OptimizationPreference=Speed`, `IlcInstructionSet=native`)
  2. Compile: `dotnet publish -c Release -r linux-x64`
  3. Use Nanos/NanoVMs (Library OS providing minimal POSIX for NativeAOT)
  4. Use `ops` CLI to bake binary into bootable image: `ops build vectorvolt_binary -c config.json`
  5. Deploy .img to hypervisor (AWS EC2 / KVM / bare metal)
  6. For Kubernetes: use KubeVirt to run Unikernel .img as a VMI (VirtualMachineInstance) alongside regular pods. KubeVirt CRDs manage lifecycle, scaling, and live migration of VectorVolt Unikernel VMs within K8s.
- **Prove:** Measure OS jitter: Linux (tuned with isolcpus/nohz_full) vs Unikernel. Boot time: Linux (~10-30 sec) vs Unikernel (~5-20 ms). Syscall overhead: eliminated entirely on Unikernel. KubeVirt: verify VM scheduling, health checks, and auto-restart work correctly.

### 8.4 WebAssembly (Wasm) Sandbox — Legacy Code Isolation
- **Learn:** How WebAssembly provides physical memory isolation via linear memory (fixed-size, no access to host heap). How Wasmtime (Bytecode Alliance) executes Wasm modules with near-native speed (~2-10x overhead). How .NET NativeAOT-LLVM compiles C# → Wasm via `dotnet-wasi-sdk`. The three-tier execution model: YAML (~1-2 ns) → DLL Sandbox (~1-10 ms) → Wasm Sandbox (~100-200 ns). Why Wasm GC garbage is physically trapped and cannot pause the host VectorVolt thread.
- **Build:**
  1. Embed Wasmtime runtime inside VectorVolt (P/Invoke to `libwasmtime.so`).
  2. Build WasmSandboxExecutor: copy CommandFrame into Wasm linear memory → execute → read result struct back.
  3. Compile a sample legacy C# module to Wasm via NativeAOT-LLVM.
  4. Integrate as third execution tier in ShardExecutor (YAML → DLL → Wasm fallback chain).
- **Prove:** Benchmark: native struct function (~2 ns) vs Wasm-sandboxed same function (~100-200 ns). Verify: Wasm module GC activity produces zero impact on host thread. Verify: Wasm module crash does not crash host. Document .NET→Wasm compilation limitations (what compiles, what doesn't).
- **Status:** Research module. Depends on .NET NativeAOT-LLVM maturing. Do not promise to enterprises until compilation path is production-ready.

### 8.5 TTL / Automatic Record Expiration
- **Learn:** How Redis, Aerospike, and ScyllaDB implement per-record TTL. How branchless TTL check works: compare entity timestamp + TTL against engine clock. How background reaper threads evict expired records without disturbing hot path. How expired records are skipped during scans (lazy deletion) and reaped by periodic compaction (eager deletion).
- **Build:** TTL field per entity (YAML-configurable). Branchless expiry check in scan loop. BackgroundReaper (shadow thread, separate core). Optional expiry event emission for audit/cleanup.
- **Prove:** Insert 100K records with 5s TTL. Verify: records accessible before TTL, invisible after. Verify: reaper reclaims memory. Verify: zero hot-path impact during reaping.

### 8.6 Probabilistic Data Structures
- **Learn:** How Bloom Filters provide O(1) membership testing with configurable false-positive rates (~1 KB per million entries). How HyperLogLog estimates cardinality in 12 KB regardless of dataset size (~0.81% error). How Count-Min Sketch estimates frequency in sub-linear memory. How all three are fixed-size, zero-alloc, MemoryArena-compatible.
- **Build:** BloomFilter\<T\> (k hash functions via CRC32 hardware intrinsic). HyperLogLog (register-based, SIMD-friendly). CountMinSketch (2D array, CRC32 hashing). All stored as fixed-size structs in MemoryArena.
- **Prove:** Benchmark insertion and query speed. Verify false-positive rates match theory. Demonstrate use cases: Gateway deduplication (Bloom), unique user counting (HLL), rate limiting (CMS).

### 8.7 Transactional Watches / Key Change Notifications
- **Learn:** How FoundationDB lets clients set watches on keys that fire when values change. How Redis keyspace notifications work. How to implement watch registration per shard without impacting hot-path latency. How the shard checks a compact watch list on entity mutation and enqueues push notifications.
- **Build:** WatchRegistry (per-shard, compact bitmap of watched entity IDs). NotificationDispatcher (push notifications to client ring buffers). YAML/SDK API for watch registration. Fire-once and continuous modes.
- **Prove:** Client watches entity A. Another client mutates A. Verify: watcher receives notification within ~μs. Verify: hot-path overhead of watch check < 2 ns (bitmap test). Benchmark: 10K active watches, verify zero degradation.

### 8.8 Write-Verify (Read-After-Write Verification)
- **Learn:** How TigerBeetle reads data back from disk after WAL writes to catch silent corruption, firmware bugs, and misdirected I/O. Why fsync alone doesn't guarantee data integrity (disk firmware can lie). How sampling (verify 1% of writes) balances safety vs performance.
- **Build:** WriteVerifier (reads WAL data back after fsync, compares against in-memory copy via CRC32). Configurable modes: disabled, sampled (1-10%), full. Runs on shadow thread.
- **Prove:** Simulate bit corruption after write. Verify: WriteVerifier detects it. Benchmark: overhead of sampled verification (target: < 1% throughput impact).

### 8.9 Built-in Load Generator (`vvctl loadgen`)
- **Learn:** How Aerospike's asbenchmark and Redis's redis-benchmark simulate real workloads. How to generate realistic entity CRUD patterns (Zipfian key distribution, configurable read/write ratio). How to measure and report p50/p99/p99.9 latency distributions.
- **Build:** LoadGenerator (configurable entity count, read/write ratio, concurrency, duration, payload size, key distribution). Integrated as `vvctl loadgen` command. Reports: ops/sec, latency percentiles, GC pauses (should be zero), ring buffer depth, WAL throughput, memory usage.
- **Prove:** Run loadgen at 1M ops/sec. Verify: metrics match vvctl metrics output. Verify: zero GC pauses reported. Compare against theoretical latency budget.

### 8.10 Warm Restart / Fast Restart
- **Learn:** How Aerospike keeps its primary index in shared memory so the process can restart without rebuilding from disk. How `shm_open` (Linux) and named shared memory (Windows) allow a MemoryArena to survive process death. How epoch + checksum validation ensures shared memory integrity on restart. The trade-off: shared memory adds complexity and requires careful cleanup on abnormal termination.
- **Build:** SharedMemoryArena (wraps MemoryArena over shared memory segment). EpochValidator (checksum + epoch tag on shared memory header). On restart: validate → attach → resume (milliseconds) vs fallback to snapshot + WAL replay (seconds).
- **Prove:** Benchmark: cold restart (snapshot + WAL replay) vs warm restart (shared memory attach). Target: warm restart < 50 ms. Verify: epoch mismatch triggers safe fallback. Verify: no data corruption after unclean shutdown.

---

## Summary

| Phase | Focus | Key Learning | Key Deliverable |
|-------|-------|-------------|-----------------|
| 1 | Memory & CPU | Cache lines, GC, NUMA, TLB, 1GB HugePages, OS jitter | MemoryArena, aligned structs, AffinityScheduler, jitter config |
| 2 | Execution | Ring buffers, wait strategies, DAGs, back-pressure, idempotency, two-phase txns, prefetch, branchless, YAML compiler | ShardExecutor, RingBuffer, WaitStrategies, ConsumerDAG, BackPressure, BloomFilter, TwoPhaseManager, YamlCommandCompiler |
| 3 | Durability | Direct I/O, group commit, hash chains, replay, projections, interceptor pipeline, bulk load | WAL writer, recovery, snapshots, ProjectionDispatcher, InterceptorPipeline, OfflineWalCompiler |
| 4 | Networking | eBPF/XDP, DPDK, RDMA, RIO, PTP, zero-copy, cloud adapters | XdpPreFilter, KernelBypassListener, RdmaReplicator, PtpClockReader |
| 5 | Reads | CQRS, columnar, SIMD/AVX2, Expression Trees, materialized views | Read Fabric, VectorScanner, QueryFilterEngine, MaterializedViewEngine |
| 6 | Control Plane | Hashing, tenancy, epoch fencing, polyglot SDKs, auto-scaling, migration, observability, QoS, versionstamps, CLI/TUI, real-time web dashboard | ShardCatalog, Gateway, Polyglot SDKs, AutoScaleManager, ShardMigrator, TscMetrics, WorkloadScheduler, VersionstampGenerator, vvctl, Web Dashboard |
| 7 | Proof | DST, load test, chaos test, migration toolkit (Gateway, eBPF Shadow, CDC), reference app testbed, platform distribution, research docs | Simulator, sample app, CommandGateway, MigrationAnalyzer, BpfRingBufferListener, WalReplicator, reference app migration demo, platform packages, benchmarks, thesis material |
| 8 | Research | Learned indexes, P4, Unikernel, Wasm, TTL, probabilistic structs, watches, write-verify, loadgen, warm restart | Optional PhD modules + advanced engine features |

---

*Version 3.0 — Competitive gap analysis complete. Added: configurable wait strategies, consumer DAGs, back-pressure policies, idempotency keys, two-phase transactions, materialized views, workload prioritization, cluster-wide versionstamps (core features); TTL expiration, probabilistic data structures, transactional watches, write-verify, load generator, warm restart (research modules). Feature parity achieved with LMAX, TigerBeetle, VoltDB, Redis, Aerospike, SingleStore, SAP HANA, FoundationDB, and ScyllaDB. Review and confirm before we begin Phase 1 development.*
