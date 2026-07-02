# VectorVolt — Complete Requirements Specification
*Version: 3.0.0 | Date: 2026-04-08 | Status: DRAFT FOR REVIEW*

---

## 0. Vision

VectorVolt is a **sub-microsecond OLTP engine** ("Data Operating System") built on **.NET 10.0**. It is a research-grade, learning-first platform that processes billions of records and millions of requests per second. Business logic executes entirely in memory. Disk exists only for durability. The OS kernel is bypassed for networking. The .NET Garbage Collector never touches the hot path.

Application developers interact with VectorVolt through **YAML configuration** and **YAML-defined business logic**. They never write code that runs inside the engine's hot path. VectorVolt compiles their YAML into branchless, zero-allocation machine code at startup.

---

## 1. Core Philosophy

Traditional databases spend 90% of their time on overhead (locking, logging, buffer management) and 10% on actual data manipulation. VectorVolt inverts this by eliminating every layer of overhead:

- **In-Memory First:** Disk is only for WAL durability, not active processing.
- **Single-Writer per Partition:** One thread owns one shard. No locks, no contention.
- **Zero-Copy Networking:** DPDK/io_uring bypass the kernel; data moves from NIC directly to user space.
- **Hardware Sympathy:** Code is written for the CPU, not against it — cache-aligned, branchless, SIMD-accelerated.

---

## 2. Architectural Goals

| Goal | Target | How |
|------|--------|-----|
| Command execution latency | ~20-50 ns (engine internals) | Zero-GC, cache-aligned structs, single-writer |
| End-to-end latency (with network) | < 1 μs (RDMA), 1-2 μs (DPDK) | Kernel bypass networking |
| Throughput per shard | Millions of ops/sec | Lock-free ring buffer, branchless execution |
| Total system throughput | Billions of records, thousands of req/sec | Horizontal sharding across cores and nodes |
| GC pauses on hot path | Zero | NativeMemory.AlignedAlloc, MemoryArena, no `new` |
| Recovery time | Seconds | Snapshot + deterministic WAL replay at millions of ops/sec |
| Data loss on crash | Zero | Log-first durability (fsync before ACK) |

---

## 3. Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Runtime | .NET 10.0 (LTS) | Managed runtime with unsafe/intrinsics access |
| Memory | NativeMemory.AlignedAlloc, Span\<T\>, ref struct | Zero-GC, cache-line-aligned allocation |
| Concurrency | Thread-per-core, LMAX-style ring buffers | Lock-free, wait-free inter-thread communication |
| Acceleration | AVX2/AVX-512 via System.Runtime.Intrinsics | Vectorized index scanning, checksum, filtering |
| CPU Dispatch | Dynamic Intel/AMD detection at boot | Wire AVX-512 (Intel) vs AVX2 (AMD) paths per chip |
| Hashing | Sse42.Crc32 / AES-NI hardware circuits | Hardware-accelerated hashing for routing and WAL checksums |
| Bit Ops | BitOperations.PopCount (POPCNT) | Single-cycle bit counting for ring buffer slot checking |
| Timing | X86Base.ReadTimeStampCounter (RDTSC) | Sub-nanosecond telemetry without OS overhead |
| I/O (Linux) | DPDK 26.03, io_uring (Kernel 6.19+) | Kernel bypass packet receive/send |
| I/O (Windows) | Registered I/O (RIO) | High-performance Windows networking |
| Replication | RDMA (RoCE v2) via libibverbs P/Invoke | Sub-microsecond cross-node memory transfer |
| Pre-filter | eBPF/XDP (Linux) | Packet validation at NIC driver level in ~10-15 ns |
| Cloud | AWS EFA, Azure InfiniBand, Oracle RDMA, GCP gVNIC | Cloud-adapted kernel bypass |
| Protocol | FlatBuffers (binary, zero-copy) | No parsing overhead on hot path |
| Durability | Append-only WAL with O_DIRECT / io_uring | Sequential NVMe writes, group commit |

---

## 4. Inviolable Rules (Non-Negotiable)

These rules cannot be compromised at any stage. Any code violating them is rejected.

1. **No Managed Allocations on Hot Path:** The `new` keyword for classes is forbidden during command execution. All data lives in unmanaged memory (MemoryArena).
2. **Cache-Line Alignment:** All hot-path structs must be exactly 64 bytes (or multiples). Prevents false sharing and cache misses.
3. **Branchless Logic:** Use SIMD masks and bitwise operations. Avoid `if/else` that cause CPU pipeline stalls (~15ns per misprediction).
4. **NUMA Affinity:** Every execution thread is pinned to a physical core and accesses only its local NUMA node's memory.
5. **Log-First Durability:** Every command is appended to the WAL (with fsync) before in-memory state mutation and acknowledgment.
6. **Memory-First Execution:** Domain logic runs in memory. SQL/DB is never in the hot path.
7. **Deterministic Processing:** Single-writer per shard. No locks, mutexes, or semaphores inside the execution engine.
8. **SQL Is Not the Source of Truth:** WAL + snapshots define correctness. SQL is a projection/audit/analytics store only.
9. **Recovery Is WAL-Only:** No DB reads during recovery. Snapshot → WAL replay → Ready.
10. **No Reflection, No LINQ on Hot Path:** Only compiled delegates, Span\<T\>, and struct-based pipelines.

---

## 5. Functional Requirements

### 5.1 Developer Interaction Model

**Developers do not write code that runs inside the engine.**

- **YAML Configuration:** Developers define topology, shards, entities, routing, retention, and scaling via YAML files.
- **YAML Business Logic (90% of use cases):** Simple state mutations, validations, and event emissions are defined declaratively in YAML. VectorVolt compiles these into branchless, zero-allocation machine code at startup.
- **DLL Sandbox (10% of use cases):** Complex business logic that cannot be expressed in YAML is provided as a compiled DLL. This code runs in a **sandboxed secondary path** (post-commit, event-driven), never on the hot path. It submits new commands back to the engine.

```yaml
# Example: YAML-defined command handler (compiled by engine)
commands:
  PlaceOrder:
    entity: Order
    validations:
      - require: command.Amount > 0
      - require: state.Status != "Cancelled"
    mutations:
      - set: Status = "Placed"
      - set: Amount = command.Amount
      - set: CreatedAt = engine.Timestamp
    emit: OrderPlaced

# Example: Complex logic via sandboxed DLL (post-commit)
post_commit:
  OrderPlaced:
    handler_dll: "MyApp.Workflows.dll"
    class: "DiscountCalculator"
    # This runs OUTSIDE the hot path, asynchronously
```

### 5.2 Command Processing Pipeline

All data mutation follows this strict pipeline:

```
Command arrives → Validation → WAL Append (fsync) → In-Memory Mutation → ACK → Async Event Dispatch
```

- Commands are intent-based (Create, Update, Delete) — not direct CRUD.
- Engine generates all IDs before WAL append (LD-ID-01).
- Per-shard monotonic sequencing guarantees total order.
- Idempotency keys ensure exactly-once execution within a shard.
- Transaction lifecycle: Prepare → Execute → Commit | Compensate.

### 5.3 Read Model (85% of Operations)

Reads are first-class. The engine supports two read modes, chosen by the developer per entity:

**Document Mode (by-ID lookups):**
- Pre-joined, denormalized entities stored as contiguous memory blocks.
- One memory fetch returns the full entity with all relationships.
- Optimal for: "Get order #12345 with all details."

**Columnar Mode (range scans, filters):**
- Each field stored as a contiguous array (all Amounts together, all Statuses together).
- SIMD/AVX2 scans 8 values per instruction.
- Optimal for: "All orders where Amount > 500 AND Status = Placed."

```yaml
read_models:
  - name: "TransactionFull"
    type: "document"
    root: "Transaction"
    include: ["TransactionDetails", "TransactionType", "TransactionAttributes"]
    access_pattern: "by_id"

  - name: "TransactionScan"
    type: "columnar"
    columns: ["TransactionId", "Amount", "Status", "Date"]
    access_pattern: "range_scan"
```

### 5.4 Time-Tiered Data Management

Not all data lives in memory. Developer configures retention per entity:

```yaml
shards:
  - name: "OrderShard"
    aggregate: "Order"
    hot_window: "30_days"           # In-memory, nanosecond reads
    warm_window: "2_years"          # NVMe SSD, microsecond reads
    cold_storage: "s3://archive"    # Beyond 2 years, millisecond reads
    memory_budget: "4GB"            # Max RAM for hot data
    eviction: "oldest_first"        # When full, push to warm tier
```

| Query Target | Tier | Latency | Storage |
|-------------|------|---------|---------|
| Recent/active data | Hot (in-memory) | ~50 ns | MemoryArena |
| Historical (months-years) | Warm (NVMe) | ~10-50 μs | SSD index scan |
| Archive (years+) | Cold (object store) | ~50-200 ms | S3/Azure Blob |

Routing is transparent — developer sends one query, engine routes to the correct tier.

### 5.5 Legacy Migration & Bulk Data Loading

**Online migration (streaming):**
```yaml
migration:
  source: "postgresql://legacy-db/orders"
  strategy: "backfill"
  # Step 1: Load hot window into memory
  # Step 2: Build warm indexes on NVMe
  # Step 3: Stream new WAL events going forward
  # Step 4: Cold data stays in legacy DB (queried on demand)
```

**Offline WAL Compiler (bulk loading billions of records):**
For massive migrations (e.g., 10 billion historical transactions), sending records through the network gateway would take days. Instead:

1. **OfflineWalCompiler** — A standalone tool that reads CSV/SQL dumps and compiles them directly into binary WalRecord format at NVMe write speed (~5 GB/sec).
2. **Direct-to-disk:** Produces a ready-to-replay WAL file (e.g., `bootstrap_tenant.wal`).
3. **Instant boot:** VectorVolt replays this WAL at startup using the DeterministicReplayer. 10 billion records loaded in seconds, not days.

```yaml
bootstrap:
  source: "csv:///data/historical_orders.csv"
  target_shard: "OrderShard"
  output: "/wal/bootstrap_orders.wal"
  format: "vectorvolt_wal_v1"
```

### 5.6 Write-Ahead Log (WAL)

- Append-only segmented log per shard (256MB segments, configurable).
- WalRecord: 64-byte cache-line-aligned struct with hash chain (PrevChecksum for tamper/corruption detection).
- fsync policies: sync (safest), group-commit (batched fsync for throughput), async (fastest, risk of last-batch loss).
- Zero-copy replication to followers via RDMA.
- Compaction: key-based (retain latest per AggregateId) and time-based retention.
- Tiered storage: Hot (NVMe) → Warm (SQL projections) → Cold (S3/Azure Blob).

### 5.7 Snapshots & Recovery

- Snapshots: time-based, count-based, or adaptive scheduling.
- Formats: binary (fastest), with compression and encryption.
- Recovery: Snapshot → WAL replay → Ready. Never queries SQL.
- WAL stores serialized CommandEnvelopes, not state mutations.
- Recovery replays commands through ShardExecutor (deterministic).
- Point-in-time recovery to any WAL LSN.

### 5.8 Sharding & Routing

- Consistent hashing (Rendezvous) for deterministic shard placement.
- Single aggregate root owned by exactly one shard at a time.
- Composite shards: multiple entities co-located for single-shard ACID (no saga needed).
- Shard Catalog: authoritative metadata for all shards.
- Policy Engine: enforces partitioning, consistency, throughput limits, rebalancing.
- Per-command topology override: force single-shard execution (policy-governed, auditable).

### 5.9 Cross-Shard Operations

- Saga / orchestration for operations spanning multiple shards (reserve → confirm → commit).
- If topology resolver selects a single composite shard, saga is bypassed — full ACID within shard.
- Idempotent steps; no distributed SQL transactions.

### 5.10 Multi-Tenancy (Engine-Level)

- Every command, event, WAL record, snapshot, and projection scoped by TenantId.
- Isolation modes (configurable per tenant via YAML):
  - **Shared Shard:** Multiple tenants, logical isolation. Cost-efficient.
  - **Dedicated Shard:** One tenant, physical isolation. For regulated/high-performance tenants.
  - **Hybrid:** Critical tenants dedicated, long-tail tenants shared.
- Per-tenant: throughput quotas, memory limits, encryption keys, RBAC, retention policies.
- Tenant boundaries preserved during WAL replay, snapshot restore, and failover.

### 5.11 Event-Driven Orchestration (Post-Commit)

- Orchestrations run strictly after transaction commit, consuming domain events.
- Fully isolated: separate processes/containers, no shared memory/threads with engine.
- Interaction only via: reading from Read Shards, submitting new Commands to Gateway.
- Replay-safe, idempotent, auditable.
- Optional engine-native DSL (Expression Trees → cached delegates) for close-to-data workflows.

#### 5.11.4 Event Interceptor Pipeline

A middleware pipeline that lets users inject custom code (PII masking, enrichment, filtering, compliance tagging) between WAL read and projection target write. Interceptors run in the **projection milli-path only** — they never touch the nano-path core engine.

- **Interface:** `IProjectionInterceptor` — ties to the projection layer, not generic event handling.
- **Execution context:** Runs inside the projector (milli-path). The core engine continues at ~20 ns regardless of interceptor behavior.
- **Registration:** User implements `IProjectionInterceptor`, registers at startup via YAML configuration.
- **Input:** Receives fully hydrated event (deserialized from WAL). Can modify, enrich, or drop the event before it reaches the projection target.
- **Action model:** Explicit `InterceptAction` enum (`Continue` / `Drop`). Null-based dropping is forbidden — silent data loss risk.
- **Error policies (per-interceptor):** `Skip` | `Retry` | `Dead-letter` | `Circuit-breaker`. Configured per interceptor in YAML.
- **Fault isolation:** If user code crashes or leaks memory, only the projector is affected. The core engine keeps running unimpaired.
- **Pipeline ordering:** Configurable via YAML. Interceptors execute in declared order.

```yaml
projections:
  - name: "OrdersToSqlServer"
    source: "OrderShard"
    target: "sqlserver://prod-db/Orders"
    interceptors:
      - class: "MyApp.Interceptors.PiiMasker"
        error_policy: "dead-letter"
      - class: "MyApp.Interceptors.ComplianceTagger"
        error_policy: "circuit-breaker"
      - class: "MyApp.Interceptors.EnrichWithGeo"
        error_policy: "retry"
        retry_max: 3
```

**Locked Decision — LD-INTC-01:** Interceptors run in the projection milli-path only; never on the nano-path. User code never executes inside the core engine.

### 5.12 Query Filter Engine (Shared Core Capability)

- Filtering, sorting, grouping, projection via compiled Expression Trees.
- Evaluated inside Read Shards using columnar in-memory layouts and Span\<T\>.
- Delegate caching for reuse.
- Zero-GC, deterministic execution.
- Accessible from SDK, ReadModel, and Read Shards.

### 5.13 Projection Dispatcher (Async Database Sync)

The engine is in-memory, but external systems need the data in traditional databases. The Projection Dispatcher asynchronously consumes committed WAL events and writes them to external databases. **This never runs on the hot path.**

- **Cursor-based:** Each projection tracks its WAL offset. On crash, resumes from last committed cursor. No duplicates (idempotent writes).
- **Bulk/batch:** No row-by-row inserts. Uses TVP (SQL Server), COPY (PostgreSQL), bulk insert (MongoDB) for millions of rows/sec.
- **Multi-target:** One WAL stream feeds multiple databases simultaneously, each with its own cursor.
- **Rebuildable:** Add a new target 6 months later — replay the WAL from the beginning to rebuild.
- **IProjectionTarget interface:** Pluggable adapters for SQL Server, PostgreSQL, Oracle, MySQL, MongoDB, Elasticsearch, or any custom store.

```yaml
projections:
  - name: "OrdersToSqlServer"
    source: "OrderShard"
    target: "sqlserver://prod-db/Orders"
    mode: "bulk_tvp"
    batch_size: 5000
    flush_interval_ms: 100

  - name: "OrdersToPostgres"
    source: "OrderShard"
    target: "postgresql://analytics/Orders"
    mode: "copy"
    batch_size: 10000

  - name: "OrdersToMongo"
    source: "OrderShard"
    target: "mongodb://nosql-cluster/Orders"
    mode: "bulk_insert"

  - name: "OrdersToElastic"
    source: "OrderShard"
    target: "elasticsearch://search-cluster/orders"
    mode: "bulk_index"
```

**Data consistency model:**
- Engine (in-memory): real-time, nanosecond reads
- Projected database: eventually consistent, typically 100-500ms behind
- If consumers need real-time: query engine's Read Fabric directly
- If consumers need historical/analytical: query projected database

### 5.14 Auto-Scaling & Shard Lifecycle

**Auto-scale triggers:**
- CPU utilization threshold per shard core
- Memory budget threshold per shard
- Ring buffer queue depth threshold (back-pressure signal)

**Shard splitting:** When a shard is too hot, the Policy Engine splits it — re-hashes a subset of keys to a new shard on another core/node.

**Shard migration protocol (live, zero-downtime):**
1. Pause writes momentarily on source shard
2. Take snapshot
3. Transfer snapshot to destination node
4. Resume writes on source; WAL streams to destination for catch-up
5. Destination catches up to latest WAL offset
6. Switchover: destination becomes leader, source stops

**Drain protocol:** Gracefully removes a node by migrating all its shards elsewhere before shutdown.

**Health monitoring:**
- Heartbeat protocol between all nodes
- Configurable failure detection timeout
- Missed heartbeats trigger automatic failover

```yaml
scaling:
  auto_rebalance: true
  max_shards_per_node: 8

  auto_scale_triggers:
    cpu_threshold: 80
    memory_threshold: 90
    queue_depth_threshold: 10000

  health:
    heartbeat_interval_ms: 100
    failure_detection_ms: 500

  migration:
    strategy: "snapshot_and_catchup"
    max_concurrent_migrations: 2
```

### 5.15 Polyglot Wire Protocol & Client SDKs

**VectorVolt's kernel speaks one language: 64-byte binary CommandFrames.** It cannot tell which programming language sent the bytes. All polyglot support is handled at the Gateway/SDK layer — YAML business logic is completely unaffected.

**Two-layer separation:**
- **YAML** = defines **what** the engine does (entities, mutations, policies). Compiled at **boot time**.
- **SDKs/gRPC** = defines **how** clients talk to the engine (wire protocol, auth, connection). Used at **request time**.

**Wire Protocol (Protocol Buffers):**

```protobuf
// vectorvolt.proto — ONE file generates ALL language SDKs
syntax = "proto3";

message VectorVoltCommand {
    uint64 sequence_number = 1;
    uint32 tenant_id = 2;
    uint32 shard_id = 3;
    bytes  aggregate_id = 4;  // 16 bytes (GUID)
    uint32 command_type = 5;
    bytes  payload = 6;
}

message VectorVoltResponse {
    uint64 sequence_number = 1;
    uint32 status_code = 2;
    bytes  payload = 3;
}

service VectorVoltService {
    rpc Execute(VectorVoltCommand) returns (VectorVoltResponse);
    rpc ExecuteStream(stream VectorVoltCommand) returns (stream VectorVoltResponse);
}
```

**Transport Options (3 tiers):**

| Protocol | Use Case | Overhead |
|----------|----------|----------|
| Raw TCP/UDP | Ultra-low-latency clients (C++, Rust) packing CommandFrames directly | ~1-5 μs |
| gRPC (binary protobuf) | Standard clients (Java, C#, Go, Python, Node.js) | ~10-50 μs |
| HTTP/REST (JSON) | Teams that refuse gRPC, browser-based clients | ~50-500 μs |

**Polyglot Client SDKs (thin wrappers, ~200-500 lines each):**

Each SDK wraps the generated gRPC/protobuf client and handles:
- Connection pooling + automatic reconnect
- Tenant authentication (JWT / mTLS)
- CommandFrame packing (language-native types → binary protobuf)
- Response unpacking (binary → language-native types)
- Retry logic with exponential backoff
- Client-side shard routing (optional, for latency-sensitive clients)

**Supported Languages:**
- **Tier 1 (built by us):** C#, Java, Python, Node.js/TypeScript, Go, Rust, C++
- **Tier 2 (community / auto-generated):** Ruby, PHP, Kotlin, Swift, Dart

**Multi-tenant + Multi-language flow:**
```
Tenant A (Java app)   → Java SDK   → gRPC → Gateway → CommandFrame → Kernel (Shard 1)
Tenant B (Python app) → Python SDK → gRPC → Gateway → CommandFrame → Kernel (Shard 5)
Tenant C (Rust app)   → Rust SDK   → Raw TCP → Gateway → CommandFrame → Kernel (Shard 3)
```

The Gateway is the GC/translation boundary. The kernel never knows which language or tenant originated the command — it only sees a 64-byte CommandFrame with a TenantId field.

**Performance Tiers by Client Language:**

| Client Language | Recommended Protocol | SDK Overhead | Total Client→Gateway |
|----------------|---------------------|-------------|---------------------|
| C++ / Rust | Raw TCP (direct CommandFrame packing) | ~1-5 μs | ~5-20 μs |
| C# / Java / Go | gRPC (binary protobuf) | ~10-50 μs | ~20-100 μs |
| Python / Node.js / Ruby | gRPC or HTTP/REST | ~50-500 μs | ~100 μs - 1 ms |

The kernel is still ~20 ns regardless of client language. Client language only affects gateway ingestion cost.

### 5.16 VectorVolt CLI & Interactive Dashboard (`vvctl`)

A unified command-line tool for configuring, managing, inspecting, and monitoring the entire VectorVolt ecosystem. Two modes: **CLI** (scriptable terminal commands) and **TUI** (live interactive dashboard like `htop`/`k9s`).

**5.16.1 Cluster Configuration & Setup**
- `vvctl init` — scaffold a new VectorVolt project (YAML templates, folder structure, sample entities)
- `vvctl apply <topology.yaml>` — deploy or update cluster topology (nodes, shards, tenants)
- `vvctl validate` — validate all YAML definitions (entities, mutations, policies, scaling rules) before deploy
- `vvctl tenant create|list|update|remove` — manage tenants and their quotas/policies
- `vvctl schema list|diff|rollback` — schema registry operations from terminal

**5.16.2 Cluster Health & Status**
- `vvctl status` — full cluster overview: nodes, shards, leaders, replicas, current epoch, quorum state
- `vvctl nodes` — per-node detail: CPU cores assigned, NUMA node, memory budget used/available, shard count, heartbeat status, uptime
- `vvctl shards` — per-shard detail: entity type, assigned core, leader/follower role, WAL offset, ring buffer depth, record count
- `vvctl health` — HA/DR readiness: quorum status, replication lag per shard, failover readiness, epoch fencing state, split-brain risk assessment

**5.16.3 Entity & Data Inspection**
- `vvctl entities` — list all registered entity types: shard assignments, record counts, memory usage, index types
- `vvctl get <EntityType> <AggregateId>` — fetch a single entity by primary key (Document mode)
- `vvctl query "<filter expression>"` — execute a query against the Read Fabric directly from terminal (e.g., `vvctl query "Order.Status == Placed AND Order.Amount > 500"`)
- `vvctl wal tail --shard <id>` — live-tail the WAL stream for a specific shard
- `vvctl wal inspect --segment <id>` — inspect WAL segment: record count, hash chain integrity check, timestamp range, checksum verification

**5.16.4 Performance & Metrics**
- `vvctl metrics` — live latency dashboard: p50/p99/p99.9 per shard, ring buffer utilization, WAL write throughput, commands/sec
- `vvctl bench` — run built-in hardware microbenchmarks (cache line timing, SIMD throughput, WAL write speed, memory bandwidth, TLB miss rate)
- `vvctl profile --shard <id> --duration <seconds>` — capture TSC-based latency profile for a specific shard, output as flamegraph-compatible format
- `vvctl cores` — CPU core map: core ID → shard → entity type → NUMA node → frequency → temperature → utilization

**5.16.5 Projection & External Database Status**
- `vvctl projections` — list all projection targets (SQL Server, PostgreSQL, ElasticSearch, etc.): cursor position, lag behind WAL head, error count, throughput
- `vvctl projection pause|resume <target>` — control projection flow (for maintenance, debugging)
- `vvctl projection replay --target <name> --from <wal-offset>` — replay projections from a specific WAL offset (for recovery or re-sync)

**5.16.6 Operations & Disaster Recovery**
- `vvctl snapshot create|list|restore` — manage point-in-time snapshots
- `vvctl drain <node>` — gracefully migrate all shards off a node before maintenance
- `vvctl failover --shard <id> --to <node>` — manual leader failover
- `vvctl rebalance` — trigger shard rebalancing across cluster based on load
- `vvctl logs --shard <id> --level <debug|info|warn|error>` — view engine logs (emitted by shadow thread, never from hot path)

**5.16.7 Migration Toolkit**
- `vvctl migrate analyze --dll <path>` — run Assembly Static Analyzer on a legacy .NET DLL, produce migration report
- `vvctl migrate analyze --connection <connstring>` — analyze legacy SQL database schema for CDC migration
- `vvctl migrate import --source <csv|sql-dump> --entity <type>` — bulk import via Offline WAL Compiler
- `vvctl migrate shadow --status` — view eBPF Shadow Mode comparison results
- `vvctl migrate cdc --status` — view CDC ingestion status, lag, schema translator events

**5.16.8 TUI Interactive Dashboard (`vvctl dashboard`)**
- Full-screen terminal UI built with `Spectre.Console` or `Terminal.Gui` (.NET TUI libraries)
- Live-updating panels: cluster map, shard topology, latency histograms, WAL throughput graph, ring buffer depth bars, core assignment grid
- Keyboard navigation: arrow keys to select a node → Enter to drill into its shards → Enter to view shard's entities/WAL/metrics
- Color-coded health: green (healthy), yellow (warning/degraded), red (critical/down)
- Searchable: type to filter entities, shards, nodes
- Auto-refresh interval configurable (default: 1 second)

```
┌─ vvctl dashboard ──────────────────────────────────────────────────┐
│ Cluster: prod-east │ Nodes: 5/5 ✓ │ Epoch: 42 │ Uptime: 14d 3h  │
├────────────────────┬───────────────────────┬───────────────────────┤
│ NODE STATUS        │ SHARD MAP             │ LATENCY (p99)         │
│ node-1 ✓ 4 shards │ Shard-0 [Order]  L    │ ██████░░ 28 ns        │
│ node-2 ✓ 4 shards │ Shard-1 [Order]  F    │ █████░░░ 22 ns        │
│ node-3 ✓ 3 shards │ Shard-2 [Payment] L   │ ███████░ 35 ns        │
│ node-4 ✓ 3 shards │ Shard-3 [Inventory] L │ █████░░░ 24 ns        │
│ node-5 ✓ 2 shards │ Shard-4 [User]  L     │ ████░░░░ 18 ns        │
├────────────────────┴───────────────────────┴───────────────────────┤
│ CORE MAP                                                           │
│ Core 0: Shard-0 (Order)     NUMA-0  3.8GHz  42°C  ████░ 78%      │
│ Core 1: Shard-2 (Payment)   NUMA-0  3.9GHz  44°C  ██░░░ 45%      │
│ Core 2: Shard-3 (Inventory) NUMA-1  3.7GHz  39°C  ███░░ 62%      │
│ Core 3: Shadow/Metrics      NUMA-1  2.1GHz  32°C  █░░░░ 12%      │
├────────────────────────────────────────────────────────────────────┤
│ PROJECTIONS          │ WAL THROUGHPUT                               │
│ SQL Server   lag: 3ms│ ████████████████████ 1.2M ops/sec           │
│ ElasticSearch lag:12ms│ Ring Buffer: 2% full                        │
│ PostgreSQL   lag: 5ms│ WAL Segments: 1,247                         │
└──────────────────────┴─────────────────────────────────────────────┘
```

### 5.17 Configurable Ring Buffer Wait Strategies

Different consumers on the ring buffer have different latency/CPU trade-off needs. The engine supports configurable wait strategies per consumer, inspired by LMAX Disruptor.

| Strategy | Behavior | CPU Cost | Latency | Use Case |
|----------|----------|----------|---------|----------|
| BusySpin | Tight spin loop, never yields | Highest (burns core) | Lowest (~ns) | ShardExecutor hot path |
| Yielding | Spin + Thread.Yield() | High | Low (~μs) | WAL writer, replicator |
| Sleeping | Spin briefly, then sleep(1ms) | Low | Medium (~ms) | Projection Dispatcher |
| Blocking | Wait on kernel event (epoll/WaitHandle) | Lowest | Highest (~ms) | Background analytics, CDC |

```yaml
ring_buffer:
  size: 1048576  # Power of two (bitwise mod)
  consumers:
    shard_executor:
      wait_strategy: "busy_spin"
    wal_writer:
      wait_strategy: "yielding"
    projection_dispatcher:
      wait_strategy: "sleeping"
```

**Pre-allocated slots:** All ring buffer entries are pre-allocated at startup via EventFactory pattern. Zero allocation at runtime. Two-phase publish: claim sequence → copy data → publish.

### 5.18 Consumer Dependency DAGs (Pipeline Orchestration)

Ring buffer consumers can be arranged in a directed acyclic graph (DAG) with gating dependencies, inspired by LMAX Disruptor.

**Example pipeline:**
```
Command arrives on Ring Buffer
        │
   ┌────┴────┐
   ▼         ▼
Journaler  Replicator   ← Run in PARALLEL (both must complete)
   └────┬────┘
        ▼
  ShardExecutor          ← Runs AFTER both complete (gated)
        │
   ┌────┴────┐
   ▼         ▼
Projector  EventEmitter  ← Run in PARALLEL after execution
```

- Each consumer maintains its own independent sequence counter.
- A gated consumer reads upstream consumers' counters — won't proceed until all upstream consumers have processed that sequence.
- DAG topology defined in YAML:

```yaml
ring_buffer:
  pipeline:
    - stage: [journaler, replicator]    # Parallel
    - stage: [shard_executor]           # Gated on previous stage
    - stage: [projector, event_emitter] # Parallel after execution
```

### 5.19 Back-Pressure Policies

When the ring buffer is full (producer has lapped the slowest consumer), the engine enforces a configurable back-pressure policy:

| Policy | Behavior | Use Case |
|--------|----------|----------|
| Block | Producer blocks until slot available (LMAX default) | Financial systems — never lose data |
| RejectWithError | Return error to client immediately | Low-latency systems that prefer fast failure |
| DropOldest | Overwrite oldest unprocessed slot | Telemetry/metrics where latest data matters most |

```yaml
ring_buffer:
  back_pressure:
    policy: "block"           # block | reject | drop_oldest
    queue_depth_warning: 8192 # Emit metric when queue exceeds this depth
```

Per-tenant back-pressure policies are supported — a noisy tenant hitting their quota gets `reject` while critical tenants get `block`.

### 5.20 Idempotency Keys (Exactly-Once Execution)

Every command carries a client-generated 128-bit idempotency key (in the existing `CorrelationId` field of CommandFrame). The engine enforces exactly-once semantics at the shard level, inspired by TigerBeetle.

- On command arrival, the shard checks a **bloom filter** (O(1), ~1ns) for probable duplicate detection.
- On bloom filter hit, verifies against an **idempotency log** (last N processed keys per shard, stored in MemoryArena).
- If duplicate confirmed: return the original result (cached), do NOT re-execute.
- If not duplicate: execute normally, add key to bloom filter + idempotency log.
- Idempotency window is configurable per entity (default: 24 hours, then keys are evicted).

```yaml
idempotency:
  window: "24h"
  bloom_filter_size: 1048576  # 1M entries, ~1.2 MB
  false_positive_rate: 0.001  # 0.1%
```

**Client retry safety:** If a network timeout occurs and the client retries the same command with the same idempotency key, VectorVolt returns the cached result instead of double-processing. No duplicate orders, no double charges.

### 5.21 Two-Phase Transactions (Pending → Post/Void/Timeout)

For operations requiring reservation semantics (payments, escrow, inventory holds, booking), VectorVolt supports two-phase transactions inspired by TigerBeetle.

**Phase 1 — Pending (Reserve):**
- Command creates a pending state: funds reserved, inventory held, seat locked.
- Entity records `PendingAmount`, `PendingState`, and a `TimeoutSeconds` value.
- The reserved amount is immediately reflected in balance checks (prevents overdraft).

**Phase 2 — Resolution (one of three):**
- **Post (Confirm):** Pending amount moves to committed. Partial post supported (post less than reserved).
- **Void (Cancel):** Pending amount released back. Entity returns to pre-reservation state.
- **Timeout (Auto-expire):** If neither post nor void arrives within `TimeoutSeconds`, the engine automatically voids the pending transaction.

```yaml
commands:
  ReserveFunds:
    entity: Account
    type: "two_phase"
    phase: "pending"
    mutations:
      - set: PendingDebit = command.Amount
    timeout_seconds: 300  # Auto-void after 5 minutes

  ConfirmPayment:
    entity: Account
    type: "two_phase"
    phase: "post"
    pending_command: "ReserveFunds"
    mutations:
      - set: CommittedDebit += pending.Amount
      - set: PendingDebit -= pending.Amount

  CancelPayment:
    entity: Account
    type: "two_phase"
    phase: "void"
    pending_command: "ReserveFunds"
    mutations:
      - set: PendingDebit -= pending.Amount
```

**Engine-level guarantees:**
- A posted transaction can never violate balance constraints (validated at pending time).
- Timeout expiry is driven by the engine's internal clock (deterministic, not wall-clock).
- All pending/post/void transitions are WAL-logged for durability and replay.

### 5.22 Materialized Views / Continuous Aggregations

Pre-computed aggregates that update in real-time as transactions execute, inspired by VoltDB materialized views. These run inside the engine — no external database needed.

**Incremental update:** When a command mutates an entity, the engine updates all dependent materialized views in the same atomic operation. No batch recomputation.

```yaml
materialized_views:
  - name: "OrdersByStatus"
    source: "Order"
    group_by: ["Status"]
    aggregates:
      - count: "TotalOrders"
      - sum: "Amount → TotalAmount"
      - avg: "Amount → AvgAmount"
      - min: "Amount → MinAmount"
      - max: "Amount → MaxAmount"

  - name: "RevenueByTenant"
    source: "Payment"
    group_by: ["TenantId", "Currency"]
    aggregates:
      - sum: "Amount → TotalRevenue"
      - count: "TransactionCount"
```

- Views are stored in the MemoryArena as compact structs (one struct per group key).
- Queryable via Read Fabric: `vvctl query "OrdersByStatus.Status == Placed"` returns the pre-computed count/sum/avg instantly (~5 ns lookup, no scan required).
- Views survive restart via WAL replay (deterministic recomputation).
- YAML-defined — developers don't write aggregation code.

### 5.23 Workload Prioritization / QoS Tiers

When real-time OLTP transactions and heavier read queries run on the same cluster, the engine enforces priority scheduling, inspired by ScyllaDB service levels.

| Tier | Shares | Behavior | Use Case |
|------|--------|----------|----------|
| Critical | 1000 | Always runs first, never preempted | Financial transactions, payments |
| Interactive | 500 | High priority, low latency target | Real-time API responses |
| Standard | 100 | Normal priority | Dashboard queries, projections |
| Batch | 10 | Lowest priority, runs in gaps | Bulk exports, analytics scans, CDC |

```yaml
workload_tiers:
  - name: "critical"
    shares: 1000
    max_latency_target_ns: 50

  - name: "interactive"
    shares: 500
    max_latency_target_ns: 500

  - name: "batch"
    shares: 10
    # No latency target — throughput optimized

tenants:
  - id: "bank_alpha"
    workload_tier: "critical"
  - id: "analytics_team"
    workload_tier: "batch"
```

- Shares are proportional: critical gets 1000/(1000+500+100+10) = ~62% of resources when all tiers are active.
- When only one tier is active, it gets 100% — no wasted capacity.
- Per-shard CPU scheduler enforces shares at the ring buffer consumer level.
- Read Fabric scans from batch-tier tenants yield to interactive-tier queries.

### 5.24 Cluster-Wide Versionstamps (Global Ordering Tokens)

For cross-shard event ordering, audit trails, and event sourcing, VectorVolt generates cluster-wide monotonic versionstamps, inspired by FoundationDB.

- **Format:** 10 bytes = 8-byte cluster-wide monotonic version + 2-byte intra-version sequence.
- **Generation:** The cluster leader assigns monotonic versions at commit time. Each committed transaction receives a unique versionstamp.
- **Embedding:** Versionstamps can be embedded into entity fields or WAL records, enabling:
  - Total ordering of all mutations across all shards.
  - Cross-shard change feeds with guaranteed ordering.
  - Audit trails with provable "event A happened before event B" semantics.
- **Per-shard sequence numbers remain** for intra-shard ordering (cheaper, no leader round-trip). Versionstamps are for cross-shard scenarios only.

```yaml
entities:
  Order:
    fields:
      - name: "LastModifiedVersion"
        type: "versionstamp"
        auto: true  # Engine assigns at commit time
```

### 5.25 Platform Distribution & Package Management

VectorVolt is distributed as a **NativeAOT single self-contained binary** per platform — no .NET runtime dependency required on the target machine. The CI/CD pipeline builds `.deb`, `.rpm`, `.msi`, `.pkg`, Docker image, and Helm chart on every release.

**Supported package managers:**

| Platform | Manager | Install Command |
|----------|---------|----------------|
| Windows | winget | `winget install VectorVolt.Engine` |
| Windows | Chocolatey | `choco install vectorvolt` |
| Ubuntu/Debian | apt | `apt install vectorvolt` |
| RHEL/CentOS | yum/dnf | `dnf install vectorvolt` |
| macOS | Homebrew | `brew install vectorvolt` |
| Docker | Docker Hub | `docker pull vectorvolt/engine:latest` |
| .NET Tool | dotnet CLI | `dotnet tool install -g vvctl` (CLI only) |
| Kubernetes | Helm | `helm install vv vectorvolt/engine` |

- **NativeAOT compilation:** Each release produces a single, statically-linked binary per OS/architecture. No JIT, no runtime install, no framework dependency.
- **CI/CD pipeline:** Every tagged release automatically builds and publishes packages to all supported registries (winget, Chocolatey, apt repo, yum repo, Homebrew tap, Docker Hub, Helm repo, NuGet for `vvctl`).
- **`vvctl` as .NET tool:** The CLI-only tool (`vvctl`) is additionally available as a .NET global tool for developers who already have the .NET SDK installed.

**Kubernetes / Helm Chart — HugePage & Resource Configuration:**

- Helm chart must include configurable HugePage resource requests.
- Pod spec must request `hugepages-2Mi` resource (configurable amount, default 1Gi).
- Pod spec must mount `hugetlbfs` volume at `/dev/hugepages`.
- Pod spec must request dedicated CPU cores (Kubernetes CPU Manager `static` policy).
- Pod spec must set `resources.requests.cpu == resources.limits.cpu` (triggers CPU pinning in Kubernetes).
- **Node prerequisite:** `vm.nr_hugepages` must be configured on worker nodes before scheduling VectorVolt pods.
- Helm `values.yaml` must expose the following tunables:

| Key | Default | Description |
|-----|---------|-------------|
| `hugepages.enabled` | `true` | Enable HugePage resource requests |
| `hugepages.size` | `"1Gi"` | Amount of 2Mi HugePages to request |
| `cpu.cores` | `4` | Number of dedicated CPU cores |
| `memory.size` | `"8Gi"` | Memory request/limit |

**Locked Decisions:**
- **LD-DIST-01:** NativeAOT single binary, no runtime dependency. Every release artifact is fully self-contained.
- **LD-DIST-02:** All major OS package managers supported. Users install VectorVolt through their platform's native package manager.
- **LD-K8S-01:** Helm chart must configure HugePage resources and CPU pinning; engine auto-detects Kubernetes cgroup constraints for core affinity.

---

### 5.26 Real-Time Monitoring Dashboard

A web-based live dashboard accessible via browser (HTML/JS/CSS served by built-in HTTP endpoint) that shows an animated deployment architecture of the entire system with live data flowing through it. Visualizes like a metro train live status board — data flowing through the pipeline in real-time.

**Key Panels:**

- **Client SDK traffic:** connections per SDK (C#, Java, Go, Python, Rust, etc.)
- **eBPF/XDP pre-filter:** packets filtered per second
- **Ring Buffer:** fill level, commands/sec entering each shard
- **Per-Shard Core utilization:** which CPU core, ops/sec, breakdown by Create/Update/Delete
- **WAL Engine:** active segment file, records/sec, bytes/sec, hash chain status, fsync mode
- **Replication:** RDMA/follower sync status, lag in bytes and time
- **Projection Dispatcher:** per-target throughput (SQL Server, Kafka, Elastic), cursor lag
- **Read Fabric:** document lookups/sec, columnar scans/sec, SIMD filter ops/sec, avg latency
- **Interceptor Pipeline:** events processed, dropped, errored per interceptor
- **Cluster health:** node status, shard distribution, epoch numbers
- **Container/Kubernetes:** HugePage allocation status (granted vs fallback), CPU cores assigned by cgroup, memory limits

**Zero-overhead metrics collection on hot path:**

- Lock-free counters using `Interlocked.Increment` (~1 ns per counter update)
- Counters are plain `long` fields in a shared `MetricsRegistry` struct
- Dashboard thread reads counters from a SEPARATE thread every 100ms
- NO locks, NO allocations, NO GC pressure on the nano-path
- WebSocket endpoint pushes metrics to browser

**Modes:** "live" (connected to running engine) — no simulation mode.

**Serving:** Built-in lightweight HTTP server (Kestrel minimal API) on configurable port. Accessible at `http://node-ip:port/dashboard`. Also available through: `vvctl dashboard --web` (launches browser).

**Locked Decisions:**
- **LD-DASH-01:** Metrics collection uses only Interlocked atomics; zero locks on hot path.
- **LD-DASH-02:** Dashboard reads metrics from separate thread; never touches nano-path.

---

### 5.27 Reference Application Testbed

A real-world business-heavy application used to prove VectorVolt works end-to-end.

- **Primary candidate:** Java application (user is arranging one) to be migrated to VectorVolt.
- **Fallback:** Open-source business-heavy Java application from GitHub (e-commerce, trading, banking, inventory management) that will be ported.

**Purpose:**

- Prove the Legacy Migration Toolkit works (Assembly Analyzer, Command Gateway, CDC, eBPF Shadow Mode)
- Demonstrate real CRUD operations at nanosecond speed
- Feed REAL data to the Live Monitoring Dashboard
- Serve as the demo application for PhD thesis and presentations
- Validate polyglot SDK (Java SDK specifically, since testbed is Java)

**Requirements for the testbed app:**

- Must have complex business logic (not just hello world)
- Must have multiple entity types (orders, customers, products, etc.)
- Must have CRUD operations with relationships
- Must have existing database (PostgreSQL/MySQL/SQL Server) for CDC testing
- Must have at least 10K records for meaningful load testing

**Migration approach:** Run Java app → CDC captures DB changes → VectorVolt ingests → dashboard shows live throughput.

**Locked Decisions:**
- **LD-TEST-01:** At least one real-world Java application must be migrated before declaring Phase 7 complete.

---

## 6. Non-Functional Requirements

### 6.1 Performance

- Engine-internal command execution: ~20-50 ns.
- End-to-end with kernel bypass networking: < 1-2 μs.
- Bounded tail latency via single-writer, back-pressure, preallocation.
- No GC pauses: all hot-path memory via MemoryArena (NativeMemory.AlignedAlloc).
- Arena reset after every command (O(1) — just zero the offset pointer).

### 6.2 Correctness

- Deterministic ordering per shard (monotonic sequence numbers).
- Idempotent command processing (correlation IDs, idempotency keys).
- ACID within shard: Atomicity (log-first), Consistency (domain invariants), Isolation (single-writer = serializable), Durability (fsync before ACK).
- Immutable audit trail via WAL hash chain.

### 6.3 Reliability & Availability

- Active-Passive or Active-Active log replication per shard.
- Automatic failover: follower promotes via epoch fencing (prevents split-brain).
- Configurable replication factor (e.g., 1 leader + 2 followers).
- No single point of failure: Gateway, Router, Projection Dispatchers are stateless and horizontally scalable.
- Self-healing: shard rebalancing, automatic restart, replay continuation.

### 6.4 Scalability

- Vertical: more shards per node (1 shard = 1 core).
- Horizontal: add nodes, rebalance shards via consistent hashing.
- Auto-rebalance configurable via YAML.
- Read scaling via Read Shards, replicas, and caches independently of write path.

```yaml
cluster:
  replication_factor: 3
  failover_mode: "automatic"
  failover_timeout_ms: 500

nodes:
  - host: "node-1"
    cores: [0, 1, 2, 3]
  - host: "node-2"
    cores: [0, 1, 2, 3]

scaling:
  auto_rebalance: true
  max_shards_per_node: 8
```

### 6.5 Security & Compliance

- TLS 1.3 for all external communication.
- AES-256-GCM encryption at rest (logs, snapshots). Per-tenant keys optional.
- RBAC for admin operations, scoped by tenant.
- Tamper-evident WAL (hash chaining).
- Immutable audit trail.

### 6.6 Operability & Zero-Overhead Observability

Standard logging (Serilog, NLog, Console.WriteLine) triggers string allocations, GC pauses, and disk I/O — destroying the nanosecond budget. VectorVolt uses a completely different approach:

**Telemetry Ring Buffers (hot path):**
- ShardExecutor writes a single 8-byte CPU Time Stamp Counter (TSC) value into a dedicated lock-free telemetry ring buffer on command completion.
- Zero allocations, zero formatting, zero I/O. Just one integer write (~1 ns).

**Shadow Thread (cold path):**
- A separate low-priority thread pinned to a different core (e.g., Core 0) reads the telemetry ring buffer.
- Formats metrics and exports to Prometheus/Grafana/OpenTelemetry.
- The hot path is completely unaware this is happening.

**External profiling:**
- Linux: eBPF for kernel-level tracing without modifying engine code.
- Windows: ETW (Event Tracing for Windows) for external profiling.
- Both allow watching the memory allocator, thread scheduler, and cache behavior from outside.

**Metrics exposed:**
- Per-shard latency percentiles (p50, p99, p99.9)
- Queue depth, replay rate, replication lag
- GC pauses (should always be zero on hot path)
- Arena usage, TLB misses, cache hit rates
- Per-tenant throughput and quota utilization

**Admin operations:** pause/resume shard, force snapshot, rebuild projections, drain queues.
**Structured logging (JSON)** with correlation IDs for end-to-end tracing (cold path only).
**Rolling upgrades** with log format/schema versioning.

### 6.7 OS Jitter Eradication

The OS is the enemy of nanosecond latency. A single kernel interrupt costs 1,000-5,000 ns. We must silence the OS on VectorVolt's cores.

**Linux (production target):**
- `nohz_full=<cores>` — Disable scheduler tick on isolated cores (normally interrupts 1,000x/sec)
- `isolcpus=<cores>` — Remove cores from OS scheduler entirely
- `intel_idle.max_cstate=0` — Disable CPU power-saving sleep states (C6 wake = ~50,000 ns)
- `mlockall()` via P/Invoke — Wire physical RAM immediately, prevent demand paging (page fault = ~2,000 ns)
- `transparent_hugepage=never` — Prevent THP compaction jitter

**Windows:**
- MMCSS (Multimedia Class Scheduler Service) — Register engine threads as highest priority, above mouse/keyboard
- `SetProcessPriorityBoost(false)` — Disable priority boost randomness
- RIO for networking (bypass Winsock kernel copy)

**macOS (development only):**
- No production-grade jitter tuning available
- Falls back to standard Sockets and managed memory
- Code compiles and runs correctly; just slower

### 6.8 Cross-Platform & Cloud Neutrality

- Single engine binary runs on Windows, Linux, and macOS (dev).
- Hardware Abstraction Layer (HAL) with platform adapters:
  - Linux: DPDK, io_uring, eBPF/XDP, sched_setaffinity, mlockall
  - Windows: RIO, SetThreadAffinityMask, MMCSS
  - macOS: kqueue (dev only, not production)
- Cloud adapters per provider:

| Provider | Recommended Instance | Networking | RDMA | Thread Pinning | NUMA | Expected Latency |
|----------|---------------------|------------|------|---------------|------|-------------------|
| On-Prem (bare metal) | Any Xeon 6 / EPYC 400 | DPDK + RDMA (Mellanox) | Full | Full (isolcpus) | Full | ~20-50 ns engine, ~1-2 μs with network |
| AWS | c5n.metal, c6i.metal | EFA | Yes | Full | Full | ~2-5 μs |
| Azure | HBv3, NDv5, Fsv2 | InfiniBand | Yes | Yes | Yes | ~2-5 μs |
| GCP | c3-highcpu | gVNIC Tier-1 | Limited | Limited | Limited | ~5-10 μs |
| Oracle Cloud | Bare Metal | Native RDMA | Full | Full | Full | ~1-3 μs |
| IBM | PowerVS | Limited | Partial | Partial | Partial | ~5-10 μs |

- No cloud-vendor lock-in in execution, durability, governance, or control-plane.
- Feature detection at startup selects optimal path per environment.
- Network transport abstraction:

```
INetworkTransport
├── DpdkTransport           (on-prem Linux)
├── RdmaTransport           (on-prem RDMA)
├── EfaTransport            (AWS)
├── InfiniBandTransport     (Azure)
├── RioTransport            (Windows)
└── StandardSocketTransport (fallback — slow but works everywhere)
```

### 6.9 Kubernetes Deployment

- K8s permitted only in Infrastructure & Control Plane.
- K8s must NOT participate in: transaction ordering, WAL replication, commit protocols, replay.
- Custom CRDs/Operators for: shard lifecycle, leader promotion, rebalancing, snapshot orchestration, DR.
- Required K8s configuration for latency-sensitive deployment:
  - **CPU Manager Static Policy** — pins pods to physical cores (not shared CPU slices)
  - **HugePages** — pre-allocated at node level (2MB or 1GB)
  - **HostNetworking mode** — bypass overlay network (Flannel/Calico adds ~50,000 ns)
  - **SR-IOV Device Plugin** — splits physical NIC into virtual hardware slices, one per pod
  - **Multus CNI** — attaches raw SR-IOV hardware NIC directly into pod, bypassing K8s network stack entirely
  - **Topology Manager** — ensures CPU, memory, and NIC are on the same NUMA node within the pod
- Without these, K8s overlay networking and pod migration will destroy latency.

### 6.10 Precision Time Protocol (PTP / IEEE 1588)

Standard clocks (NTP) drift by milliseconds between nodes. At nanosecond scale, this makes cross-node ordering impossible.

- **PTP hardware timestamps:** Modern NICs (Mellanox ConnectX, Intel E810) have onboard hardware clocks synchronized to <100 ns precision across the cluster.
- **NIC stamps the packet** the instant it arrives — not when the OS processes it.
- **CommandFrame.Timestamp** is sourced from the NIC's hardware clock (PHC) via P/Invoke, not `DateTime.UtcNow`.
- **Result:** Globally ordered transaction serialization across the cluster without a central coordinator.
- **Fallback:** On dev machines or cloud VMs without PTP hardware, fall back to `DateTime.UtcNow` (sufficient for correctness, not for nanosecond ordering).

---

## 7. Hardware Technologies Used

| Technology | Using? | Layer | How |
|-----------|--------|-------|-----|
| SIMD/AVX2 | Yes | Index scanning, checksum, filtering | System.Runtime.Intrinsics (native C#) |
| NUMA | Yes | Memory/thread affinity | P/Invoke (sched_setaffinity / SetThreadAffinityMask) |
| DPDK | Yes | Client → Engine networking (Linux) | P/Invoke to C library |
| RDMA (RoCE v2) | Yes | Engine → Engine replication | P/Invoke to libibverbs |
| io_uring | Yes | Disk I/O (Linux) | P/Invoke |
| RIO | Yes | Networking (Windows) | P/Invoke |
| eBPF/XDP | Yes | Pre-filter at NIC driver level (Linux) | Packet validation in ~10-15 ns before DPDK poll loop |
| HugePages (2MB) | Yes | TLB miss reduction | NativeMemory.AlignedAlloc |
| HugePages (1GB) | Yes (configurable) | Near-zero TLB misses | Linux boot-time reservation; 4 TLB entries covers 4GB |
| Prefetching | Yes (critical) | Predictive cache warming | Sse.Prefetch0(); ring buffer lookahead prefetches next entity |
| PTP (IEEE 1588) | Yes | Nanosecond-precision cross-node timestamps | NIC hardware clock (PHC) via P/Invoke; replaces DateTime.UtcNow |
| CRC32/AES-NI | Yes | Hardware-accelerated hashing | Sse42.Crc32 for WAL checksums and rendezvous routing (~1 cycle vs thousands for software hash) |
| POPCNT (BMI1) | Yes | Lock-free bit manipulation | BitOperations.PopCount for ring buffer slot checking (1 cycle vs 64-iteration loop) |
| RDTSC | Yes | Sub-nanosecond timing | X86Base.ReadTimeStampCounter for TscMetricsBuffer (~1 ns vs Stopwatch ~20-30 ns) |
| Dynamic CPU Dispatch | Yes | Intel vs AMD optimization | Boot-time detection wires AVX-512 (Intel Xeon) vs AVX2 (AMD EPYC) code paths |
| mlockall | Yes (Linux) | Prevent demand paging / page faults | P/Invoke; wire physical RAM at startup |
| MMCSS | Yes (Windows) | Thread priority above all OS tasks | P/Invoke; register engine threads as highest priority |
| AVX-512 | Deferred (Tier 4) | Extended vectorization | Risk of CPU thermal throttling |
| CUDA/GPU | No | Not applicable | OLTP is random-access, single-record; GPUs are bulk-parallel |
| SmartNIC/FPGA | No (design-compatible) | Not in scope | Wire format (64-byte binary) is FPGA-parseable if needed later |
| Intel TSX | Dropped | Dead technology | Disabled via microcode (security vulns); single-writer eliminates need |
| CXL | Deferred (interface-ready) | Memory disaggregation | IMemoryArena supports pluggable backing stores for future CXL memory pools |
| In-Network Compute (P4) | Deferred (design-compatible) | Switch-level sequencing | CommandFrame.SequenceNumber ready to receive P4-stamped global order |
| UINTR | Deferred | User-mode interrupts | Experimental kernel support; INetworkTransport ready for future UintrTransport |
| PMem/Optane | Dropped | Intel killed Optane in 2022 | Dead technology |

---

## 8. Latency Budget Breakdown

**The Physics of Time:**
- 1 nanosecond = light travels 30 cm through fiber
- 1 ns ≈ 1 clock cycle at 1 GHz (0.2 ns at 5 GHz)
- L1 cache hit: 1-2 ns | L2: 4-7 ns | L3: 15-20 ns | RAM: 60-100 ns
- **If data hits L3, the 20 ns budget is gone. If it hits RAM, we've failed by 500%.**
- Therefore: all hot data MUST reside in L1/L2 cache. Entire business logic must execute in ~30-60 CPU clock cycles.

Every nanosecond is accounted for:

| Operation | Cost | Mitigation | Savings |
|-----------|------|------------|---------|
| GC pause | 10,000,000 ns | MemoryArena (zero-GC) | Entire milliseconds |
| Kernel network syscall | 50,000-100,000 ns | DPDK / kernel bypass | ~98,000 ns |
| Invalid packet processing | 50,000+ ns | eBPF/XDP pre-filter at NIC driver | Rejected in ~10-15 ns before reaching engine |
| Context switch | 5,000-10,000 ns | Thread pinning (isolcpus) | ~9,000 ns |
| Lock contention | 1,000-10,000 ns | Single-writer per shard | ~9,000 ns |
| L3 cache miss → RAM | 60-100 ns | NUMA-local memory + prefetching | ~60 ns |
| TLB miss (page walk) | 10-30 ns | 1GB HugePages (4 TLB entries = 4GB) | ~25 ns |
| Branch misprediction | 12-15 ns | Branchless SIMD code | ~12 ns |
| L2 cache miss → L3 | 10-15 ns | 64-byte struct alignment | ~10 ns |
| Scalar scan of 8 keys | 8-16 ns | AVX2 (1 instruction) | ~14 ns |
| Memory copy | 5-20 ns | Zero-copy (ref, Span\<T\>) | ~15 ns |
| Function call overhead | 2-5 ns | AggressiveInlining | ~4 ns |

**Theoretical floor for one CRUD operation (engine internals):**

```
Ring buffer read:     ~3 ns  (prefetched via lookahead, cache-line-padded head/tail)
Entity prefetch:      ~0 ns  (issued during previous command, data already in L1)
Hash index lookup:    ~5 ns  (SIMD-accelerated, entity in L1/L2 from prefetch)
State mutation:       ~2 ns  (compiled YAML handler, ref struct, branchless)
WAL batch append:     ~5 ns  (amortized group commit, 1GB HugePages = no TLB miss)
────────────────────────────────
Total:                ~15 ns per operation (best case, all L1/L2 hits)
Total:                ~22 ns per operation (typical, some L2 hits)
```

**Critical technique for hitting 15-22 ns: Ring Buffer Lookahead Prefetching.**
The ring buffer lets us peek at the next 2-3 commands. While processing command N, we issue `Sse.Prefetch0()` for command N+1's target entity. By the time we finish N, entity N+1 is already in L1 cache (1 ns access instead of 100 ns from RAM).

---

## 9. Component Architecture (~128 Core Classes)

### Component 1: Hardware Abstraction Layer (HAL) — ~12 classes
Directly interfaces with CPU, RAM, and NIC hardware.
- MemoryArena (with 2MB and 1GB HugePage support), AffinityScheduler, CacheLineConstants
- CommandFrame, WalRecord (64-byte aligned structs)
- HugePageAllocator, NumaAllocator
- PlatformDetector (feature detection at startup)
- PrefetchHelper (ring buffer lookahead prefetching via Sse.Prefetch0)

### Component 2: Command Execution Kernel (Hot Path) — ~25 classes
Single-writer state machine per shard.
- ShardExecutor (infinite loop processing commands)
- RingBuffer (LMAX-style wait-free queue, power-of-2, pre-allocated slots)
- IWaitStrategy: BusySpinWaitStrategy, YieldingWaitStrategy, SleepingWaitStrategy, BlockingWaitStrategy
- SequenceBarrier, ConsumerDagBuilder (pipeline orchestration with gating)
- IBackPressurePolicy: BlockPolicy, RejectPolicy, DropOldestPolicy
- BloomFilter, IdempotencyLog (exactly-once execution)
- TwoPhaseTransactionManager, PendingTimeoutScheduler (pending → post/void/timeout)
- TransactionContext (Prepare → Execute → Commit)
- YamlCommandCompiler (compiles YAML mutations to delegates)
- CommandFrame routing and validation

### Component 3: Durability Engine (WAL) & Projections — ~23 classes
Append-only sequential NVMe writes + async database sync + bulk loading.
- WalSegmentWriter (O_DIRECT / io_uring)
- GroupCommitBatcher (coalesce multiple commands per fsync)
- DeterministicReplayer (crash recovery)
- SnapshotManager
- WalCompactor (key-based and time-based)
- ProjectionDispatcher (async WAL consumer, cursor-based)
- IProjectionTarget interface + adapters: SqlServerTarget (TVP), PostgresTarget (COPY), OracleTarget, MySqlTarget, MongoTarget (bulk), ElasticTarget
- ProjectionCursorStore (tracks WAL offset per projection)
- OfflineWalCompiler (standalone tool: CSV/SQL dump → binary WAL at NVMe speed)
- InterceptorPipeline (ordered middleware chain between WAL read and projection write)
- IProjectionInterceptor (user-implemented interface for event modification/filtering)
- InterceptResult (carries InterceptAction enum: Continue/Drop + modified event payload)

### Component 4: Read Fabric — ~22 classes
Handles queries without disturbing the write path.
- DocumentStore (denormalized, pre-joined entities)
- ColumnarStore (SIMD-scannable column arrays)
- QueryFilterEngine (Expression Trees → cached delegates)
- VectorScanner (AVX2 hardware-accelerated scanning)
- TieredRouter (hot → warm → cold transparent routing)
- MaterializedViewEngine (incremental aggregation on write)
- ViewStore (compact structs per group key, queryable via Read Fabric)

### Component 5: Kernel Bypass Networking — ~14 classes
DPDK/RDMA/RIO/eBPF layer.
- XdpPreFilter (eBPF/XDP program for ~10-15 ns packet validation at NIC driver level, Linux)
- KernelBypassListener (polls NIC hardware queues via DPDK)
- ZeroCopyDeserializer (packet → CommandFrame without allocation)
- RdmaReplicator (cross-node WAL sync)
- INetworkTransport (abstraction interface)
- DpdkTransport, EfaTransport (AWS), InfiniBandTransport (Azure), RioTransport (Windows), StandardSocketTransport (fallback)
- FlowSteeringConfigurator (RSS/FDIR NIC programming)

### Component 6: Control Plane ("Data OS" Brain) — ~28 classes
YAML-driven configuration, orchestration, scaling, health, and observability.
- YamlTopologyParser
- ShardCatalog, PolicyEngine
- CommandGateway (unified entry point, GC boundary for all client protocols)
- ProtobufTranslator (protobuf → CommandFrame zero-copy packing)
- GrpcEndpoint, RawTcpEndpoint, HttpRestEndpoint (transport listeners)
- TenantManager (isolation, quotas, encryption)
- SchemaRegistry (versioning, compatibility)
- MigrationEngine (legacy backfill)
- AutoScaleManager (threshold-based shard splitting and node scaling)
- ShardMigrator (live migration: snapshot → transfer → WAL catchup → switchover)
- DrainCoordinator (graceful node removal)
- HealthMonitor (heartbeat protocol, failure detection)
- EpochFencingManager (split-brain prevention)
- TscMetricsBuffer (zero-GC telemetry ring buffer, 8-byte TSC writes)
- ShadowMetricsExporter (low-priority thread → Prometheus/Grafana/OpenTelemetry)
- WorkloadScheduler (per-shard share-based QoS prioritization)
- VersionstampGenerator (cluster-wide monotonic ordering tokens)

### Component 7: CLI & Interactive Dashboard (`vvctl`) — ~15 classes
Unified operational tool for configuration, monitoring, inspection, and management.
- VvctlApp (entry point, command routing via System.CommandLine)
- ClusterStatusCommand, NodeListCommand, ShardListCommand
- EntityListCommand, EntityGetCommand, QueryCommand
- WalTailCommand, WalInspectCommand
- MetricsCommand, ProfileCommand, CoreMapCommand
- ProjectionStatusCommand, ProjectionControlCommand
- SnapshotCommand, DrainCommand, FailoverCommand, RebalanceCommand
- MigrateAnalyzeCommand, MigrateImportCommand, MigrateShadowCommand
- DashboardRenderer (TUI mode via Spectre.Console / Terminal.Gui)
- VvctlApiClient (connects to VectorVolt cluster via gRPC for all operations)

### Component 8: Real-Time Web Monitoring Dashboard — ~5 classes
Web-based live dashboard with zero-overhead metrics collection.
- MetricsRegistry (shared struct with lock-free `long` counters updated via `Interlocked.Increment`)
- MetricsCollector (separate thread that reads MetricsRegistry counters every 100ms)
- MetricsSnapshot (immutable point-in-time copy of all counters for serialization)
- MetricsHttpServer (Kestrel minimal API serving HTML/JS/CSS dashboard and REST endpoints)
- DashboardWebSocketHandler (pushes MetricsSnapshot to connected browsers via WebSocket)

---

## 10. ACID Guarantees

### Atomicity
- Log-first: durable intent logged before in-memory mutation.
- Composite shards: all state transitions in a single atomic log record across co-located entities.
- Cross-shard: saga with idempotent, compensatable steps.

### Consistency
- Domain invariants enforced by compiled YAML validations.
- Versioned state; no side-effects outside engine during execution.

### Isolation
- Single-writer per shard = serializable execution (strongest isolation level).
- Cross-shard flows coordinated via messages/sagas.

### Durability
- WAL fsync (or replicated commit) before ACK.
- Snapshots + deterministic replay for recovery.
- SQL projections are derivable; never the source of truth.

---

## 11. Locked Decisions (from Vega LLD Standards — Adopted)

All LD-* decisions from vega_lld_standards.md are adopted and incorporated:

- LD-XX: Shard Governance & Topology Control
- LD-K8S-01: Helm chart must configure HugePage resources and CPU pinning; engine auto-detects Kubernetes cgroup constraints for core affinity
- LD-K8S-02: Kubernetes boundary enforcement
- LD-DEP-01: Cloud-neutral mandate
- LD-INF-01: Sidecar isolation
- LD-READ-01/02: Read Fabric mandate, CQRS enforced
- LD-REF-01/02: Replicated reference data, embedded sub-shards
- LD-SCALE-01: Deterministic write scaling
- LD-AA-01: Active-active by shard (no dual leaders)
- LD-LB-01: Load balancing rules (writes deterministic, reads balanced)
- LD-CFG-01: Declarative auto-scaling & topology
- LD-BOOT-01: Read Fabric bootstrap (backfill then WAL streaming)
- LD-RUNTIME-01: Role-based runtime model (single binary, multiple roles)
- LD-SDK-01: Polyglot client SDKs (gRPC/Raw TCP/HTTP — protobuf wire protocol)
- LD-SDK-02: YAML defines engine behavior (boot time), SDKs define client communication (request time) — completely independent layers
- LD-SDK-03: Tier 1 SDKs (C#, Java, Python, Node.js, Go, Rust, C++) built in-house; Tier 2 auto-generated
- LD-MT-01 through 08: Native multi-tenancy
- LD-FUNC-01 through 04: Orchestration isolation
- LD-DSL-01: Engine-native DSL (optional)
- LD-ID-01: Engine-generated identifiers
- LD-PERF-01: Execution & memory discipline
- LD-READ-DSL-02: Query filter engine as shared capability
- LD-OPS-01: Workflow isolation
- LD-REC-01: Recovery execution rule (WAL stores commands, not state)

---

## 12. Testing Strategy

- **Test-First:** Every component gets tests before implementation code.
- **Unit Tests (70%):** < 100ms each. Cover all hot-path logic, struct alignment, arena allocation, ring buffer operations.
- **Integration Tests (20%):** 1-2 min. WAL write/replay, shard failover, cross-shard saga.
- **E2E Tests (10%):** 10-20 min. Full pipeline from client command to read query with sample app.
- **Performance Tests:** BenchmarkDotNet microbenchmarks for every hot-path function. VTune/perf profiling for cache misses and branch mispredictions.
- **Deterministic Replay Tests:** Verify recovery produces identical state from WAL.
- **Chaos Tests:** Adapter outages, replica lag, disk faults, power failure simulation.
- **Deterministic Simulation Testing (DST):** FoundationDB/TigerBeetle-proven methodology. Replace physical clock, network, and disk with mock hardware. Run entire cluster in a single thread. Simulate millions of failure scenarios (power loss, disk corruption, network partition, split-brain) per hour. Because VectorVolt enforces strict determinism (no DateTime.UtcNow, no Guid.NewGuid, no random I/O on hot path), DST is a natural fit. Can simulate 10 years of cluster uptime and failures in a 1-hour test run.

---

## 13. Success Criteria

- Sub-microsecond end-to-end latency with kernel bypass networking.
- ~20-50 ns engine-internal command execution.
- Zero GC pauses on hot path (verifiable via .NET diagnostics).
- Zero data loss on crash (WAL-first durability).
- Recovery in seconds via snapshot + replay.
- Billions of historical records handled via tiered storage.
- Developer onboarding via YAML only (no engine internals knowledge required).
- All concepts understood and documented as research material.

---

## 14. Research Modules (Tier 4 — PhD Differentiators)

These are not core requirements but potential research contributions that can be built as optional modules:

**Learned Indexes (AI-Indexed Memory):**
- MIT 2018 paper: replace B-Tree with a tiny ML model (linear regression) that predicts memory offset from key.
- B-Tree: ~3 cache misses (~150 ns). Learned index: ~3 ns (SIMD-accelerated math formula).
- Feasibility: works when data distribution is predictable and stable. Our hash index gives ~5-10 ns already, but learned indexes could push to ~3 ns for sorted/sequential keys.
- Implementation: train model on key distribution at snapshot time; retrain periodically.

**In-Network Compute (P4 / Programmable Switches):**
- Program network switch ASICs (Intel Tofino) to stamp global sequence numbers on packets at line rate.
- Eliminates consensus bottleneck entirely — packets arrive pre-ordered.
- Academic reference: NOPaxos (Network-Ordered Paxos).
- Our CommandFrame.SequenceNumber is already designed to receive switch-stamped order.

**Unikernel Deployment (OSv + NativeAOT):**
- Eliminate the OS entirely. Compile VectorVolt via .NET NativeAOT, link with minimal hardware drivers (OSv/NanoVMs).
- No kernel, no user-space boundary, no context switches, no bash shell. The C# app IS the operating system.
- .NET NativeAOT makes this more feasible than ever.
- Eliminates all OS jitter by definition.
- K8s orchestration via KubeVirt: wraps .img in container for transport, KubeVirt boots it as a hardware VM (KVM/QEMU) not a container. Must combine with SR-IOV + Multus to bypass K8s networking.
- Observability via embedded OpenTelemetry push (UDP fire-and-forget from shadow thread) + hypervisor-level monitoring (CPU/memory/network from outside the VM).

**Coroutine Latency Hiding:**
- MosaicDB (Microsoft Research) pattern. When Transaction A hits a cache miss, yield to Transaction B while RAM fetches A's data.
- Guarantees CPU ALU is doing math 100% of the time, never waiting.
- .NET lacks suitable stackless coroutines today; our prefetching approach covers ~80% of the benefit.

**TTL / Automatic Record Expiration (inspired by Redis, Aerospike, ScyllaDB):**
- Per-record or per-entity time-to-live. When TTL expires, the engine automatically evicts/deletes the record.
- Use cases: session stores, temporary holds, rate limiting windows, cache entries.
- TTL check is branchless: compare entity timestamp + TTL against current engine clock. Dead records skipped during scans, reaped by background thread.
- Configurable per entity type in YAML: `ttl: "30m"`, `ttl: "24h"`, `ttl: "never"`.
- Expired records can optionally trigger an event (for audit/cleanup workflows) before deletion.

**Atomic Multi-Operation Commands (inspired by Redis MULTI/EXEC, Aerospike Operations API):**
- A single CommandFrame can carry multiple field-level mutations executed atomically on one entity.
- Example: increment balance, append to transaction list, set last-modified timestamp — all in one atomic command, one WAL record, one network round-trip.
- Reduces client round-trips for compound mutations from N to 1.
- Defined in YAML as a command with multiple mutation steps (already partially supported by our mutation list syntax).

**Probabilistic Data Structures (inspired by Redis):**
- **Bloom Filter:** O(1) membership test ("have I seen this key before?"). ~1 KB per million entries. Configurable false-positive rate.
- **HyperLogLog:** Cardinality estimation ("how many unique users today?"). 12 KB per counter regardless of cardinality. ~0.81% error.
- **Count-Min Sketch:** Frequency estimation ("how many times has this IP been seen?"). Sub-linear memory.
- Use cases: deduplication at Gateway, rate limiting, analytics, anomaly detection.
- Stored in MemoryArena as fixed-size structs — zero-GC compatible.

**Transactional Watches / Key Change Notifications (inspired by FoundationDB, Redis):**
- Client sets a watch on an entity/key. Engine fires an async push notification when the entity changes.
- Eliminates polling for real-time UI updates, live dashboards, and event-driven integrations.
- Watch registration stored per-shard. On entity mutation, shard checks watch list and enqueues notification to the client's ring buffer.
- Configurable: fire-once (re-register to continue) or continuous (fire on every change until unsubscribed).

**Write-Verify (Read-After-Write Verification) (inspired by TigerBeetle):**
- After WAL write + fsync, optionally read the data back from disk and compare against the in-memory copy.
- Catches silent disk corruption, firmware bugs, misdirected I/O, and bit rot.
- Configurable: disabled (fastest), sample (verify 1% of writes), full (verify every write).
- Verification runs on shadow thread — zero hot-path impact.

**Built-in Load Generator (`vvctl loadgen`) (inspired by Aerospike asbenchmark):**
- Beyond hardware microbenchmarks (`vvctl bench`), a full entity-level load generator that simulates real CRUD workloads.
- Configurable: entity count, read/write ratio, concurrency, duration, payload size.
- Reports: ops/sec, p50/p99/p99.9 latency, GC pauses (should be zero), ring buffer depth, WAL throughput.

**Warm Restart / Fast Restart (inspired by Aerospike):**
- On process restart, keep the MemoryArena in shared memory (`shm_open` on Linux, named shared memory on Windows).
- On restart, the engine validates the shared memory region (checksum + epoch check) and resumes without WAL replay.
- Recovery time: seconds (WAL replay) → milliseconds (shared memory attach).
- Fallback: if shared memory is corrupted or epoch mismatch, fall back to normal snapshot + WAL replay.

---

## 15. Legacy Application Migration (Brownfield Strategy)

Migrating existing .NET applications (Entity Framework, LINQ, SQL Server, string-heavy code) into VectorVolt's zero-allocation ecosystem requires a phased approach. Direct copy-paste of legacy code would trigger GC, flush L1 cache, and degrade the engine from 20 ns to 200 ms.

### 15.1 Command Gateway (Anti-Corruption Layer)

- Legacy applications send standard JSON/HTTP requests to an **Edge Gateway**.
- The Gateway is a normal C# process — GC is permitted, strings are permitted. Expected latency: ~5-50 μs (still 1000x faster than legacy SQL round-trips).
- The Gateway's job: deserialize JSON → allocate into MemoryArena → construct 64-byte CommandFrame → push onto Ring Buffer.
- The VectorVolt kernel never sees JSON, strings, or HTTP. Complete isolation.

### 15.2 Polyglot Projections (Legacy Read Continuity)

- Legacy applications stop writing to SQL directly. Writes go through VectorVolt Gateway.
- VectorVolt processes the command in nanoseconds, then the **Projection Dispatcher** (Section 5.13) asynchronously updates the legacy SQL Server / PostgreSQL / MongoDB tables.
- Legacy read screens, dashboards, and APIs continue reading from SQL unchanged.
- **Caveat:** This introduces eventual consistency. Legacy SQL reads may lag VectorVolt writes by milliseconds. For real-time balance/position screens, clients must read from VectorVolt's hot path directly (new read endpoints required).

### 15.3 Business Logic Extraction

**Two-tier migration path for rewriting business logic:**

1. **Simple Logic (90%) → YAML Mutations:** Define entities and state transitions in YAML. The engine compiles these into branchless delegates at startup. Legacy developers don't write C# — they write YAML.
2. **Complex Logic (10%) → DLL Sandbox:** Complex rules (47 if/else branches, external API calls) run post-commit in a separate process. Never on the hot path.

**Pure Math Function Rule:** Native VectorVolt functions take a struct in, return a struct out. No database calls, no I/O, no strings, no allocations.

### 15.4 Assembly Disassembly & Static Analysis (Migration Assistant)

- Use IL decompilation (ILSpy, Mono.Cecil) to scan existing .NET assemblies.
- Produce an automated **Migration Report** classifying every method:
  - ✅ **Auto-convertible:** Pure arithmetic/logic methods → auto-generate YAML mutations or struct functions.
  - ⚠️ **Mixed:** Methods combining logic + I/O (DbContext, HttpClient, File.IO) → need manual separation of pure logic from side effects.
  - ❌ **Gateway-only:** Pure I/O methods → stay in Gateway layer or DLL Sandbox.
- **Convertible patterns:** if/else, switch, math, enum values, constants, state machines, pure filters.
- **Non-convertible patterns:** DbContext calls, HttpClient calls, string manipulations (must determine: business-meaningful strings → enum mapping, display strings → Gateway layer), LINQ chains hiding database calls.
- This is a **migration assistant**, not a migration automator. Enterprise selling point: "plug in your DLL, we tell you exactly what moves to the hot path."

### 15.5 Data Migration (Offline WAL Compiler)

- Existing billion-row SQL databases are migrated via the **Offline WAL Compiler** (Section 3.6 in Implementation Plan).
- Bulk-export data from SQL → CSV/SQL dump → compile directly into WAL segment format → replay into VectorVolt.
- No live engine required during bulk import. WAL segments are pre-built offline and loaded at startup.

### 15.6 eBPF Network Mirroring — Shadow Mode (Strategy 1: Zero-Touch Migration)

For tier-one financial institutions where the legacy system is too fragile to modify, VectorVolt can intercept production traffic at the OS level without the legacy application knowing.

**Architecture: XDP Filter + uretprobe (two-stage pipeline)**

1. **XDP Packet Filter (Ring 0, NIC driver level):**
   - eBPF program written in Restricted C, compiled to eBPF bytecode via clang/LLVM.
   - Runs before the Linux kernel allocates memory for the packet.
   - Filters by TCP destination port (e.g., only port 443) and IP subnet — drops SSH, health checks, pings at 20-30M packets/sec with near-zero CPU.
   - Prevents "bombardment" — only business traffic reaches the mirror pipeline.

2. **uretprobe TLS Decryption Intercept (User-Space Return Probe):**
   - XDP sees encrypted TLS packets (random garbage). Cannot decrypt without the private key, and TLS decryption math would destroy the nanosecond budget.
   - Instead, attach a `uretprobe` to the `SSL_read()` return point inside the legacy app's TLS library (OpenSSL `libssl.so` on Linux, SChannel on Windows).
   - **Entry uprobe** captures the buffer pointer (register `rsi` on x86-64), stores in BPF_MAP_TYPE_HASH keyed by thread ID.
   - **Return uretprobe** fires after decryption completes, copies the plaintext payload from the legacy app's RAM into an eBPF Ring Buffer.
   - VectorVolt reads the decrypted JSON from the ring buffer via P/Invoke to `libbpf.so`, zero-copy.

3. **Shadow Mode Operation:**
   - VectorVolt processes identical production traffic in parallel with the legacy system.
   - Run for weeks/months, comparing VectorVolt output vs legacy SQL at end of each day.
   - Proves mathematical correctness before any cutover.
   - Legacy developers change zero lines of code.

**Constraints & Caveats:**
- eBPF probes must be written in Restricted C (kernel verifier rejects loops, dynamic allocation, uninitialized variables). Cannot be written in C#.
- uretprobe overhead: ~1-5 μs per probe fire (int3 breakpoint trap). Acceptable for 10K req/sec, measurable at 1M req/sec.
- TLS library varies: OpenSSL (`SSL_read`), BoringSSL (`SSL_read`), .NET SslStream (wraps OpenSSL on Linux). Probe must be tuned per target.
- Ring buffer size must be tunable (4-16 MB for production, not 256 KB) with drop counter for burst overflow detection.
- `ring_buffer__poll` should use epoll-based blocking (`timeout_ms: -1`) not polling loops, to avoid CPU waste.
- Linux only. Windows `ebpf-for-windows` (Microsoft) supports XDP but not uprobes yet.

### 15.7 Reverse-Ingestion via CDC — Change Data Capture (Strategy 2: Database-Level Migration)

For enterprises where the legacy code is too deeply coupled to SQL to insert a Gateway, VectorVolt can consume the database's own transaction log.

**Architecture: Debezium + WalReplicator**

1. Deploy Debezium CDC connector alongside legacy SQL Server / PostgreSQL.
2. Debezium reads the database's internal WAL/transaction log at the disk level.
3. Every INSERT/UPDATE/DELETE is streamed as a change event to VectorVolt's **WalReplicator**.
4. WalReplicator translates SQL row changes into CommandFrames and executes them.
5. VectorVolt builds a nanosecond-fast in-memory replica of the legacy database.
6. Gradual cutover: point read-heavy dashboards to VectorVolt first, migrate writes later.

**Constraints & Caveats:**
- **Schema evolution:** When legacy team ALTERs a SQL table, the CDC stream shape changes. Requires a **CDC Schema Translator** integrated with the Schema Registry (Phase 6).
- **Cross-table ordering:** Debezium reads WAL sequentially per table, but cross-table transaction atomicity is not guaranteed. VectorVolt needs transaction-boundary markers to reconstruct atomicity.
- **Latency:** Debezium delivers changes in ~10-500 ms. Suitable for read replicas and dashboards, not for real-time trading positions.
- **Most practical strategy for 90% of enterprises.** Battle-tested at Netflix, Airbnb, Uber scale.

### 15.8 WebAssembly (Wasm) Sandbox (Strategy 3: Legacy Code Isolation — Research)

For enterprises that refuse to rewrite legacy C# business logic but want it running inside VectorVolt, Wasm provides physical GC isolation.

**Architecture: NativeAOT-LLVM → Wasm → Wasmtime**

1. Compile legacy C# code to WebAssembly via .NET NativeAOT-LLVM (`dotnet-wasi-sdk`).
2. Embed Wasmtime runtime inside ShardExecutor.
3. Copy 64-byte CommandFrame into Wasm module's isolated linear memory.
4. Wasm module executes legacy logic — any GC garbage is trapped inside the Wasm memory sandbox.
5. Read result struct back from linear memory.

**Three-Tier Execution Model:**
| Tier | Mechanism | Latency | Use Case |
|------|-----------|---------|----------|
| 1 | YAML compiled mutations | ~1-2 ns | Simple state transitions (90%) |
| 2 | DLL Sandbox (separate process) | ~1-10 ms | Complex post-commit logic (10%) |
| 3 | Wasm Sandbox (inline, isolated) | ~100-200 ns | Legacy code that can't be rewritten |

**Constraints & Caveats:**
- **.NET NativeAOT-LLVM → Wasm is NOT production-ready today.** Entity Framework, HttpClient, and most legacy surface area won't compile to Wasm without rework. This is a Phase 8 research module.
- **Performance:** Wasm execution is ~2-10x slower than native. 20 ns native → 40-200 ns Wasm. Still faster than SQL, but breaks sub-microsecond promise. Position as compatibility tier, not performance tier.
- **Memory copy overhead:** ~50-200 ns per call (memcpy + boundary checks into/out of linear memory).
- **Wasmtime is production-quality** (Fastly, Cloudflare Workers). The runtime is ready; the .NET→Wasm compilation path is the bottleneck.

### Locked Decisions: Migration
- **LD-MIG-01:** Legacy applications NEVER touch the VectorVolt kernel directly. All legacy interaction goes through the Command Gateway.
- **LD-MIG-02:** Gateway latency (~5-50 μs) is acceptable; it is not on the nanosecond hot path.
- **LD-MIG-03:** eBPF probes are written in Restricted C, not C#. This is a kernel law, not a choice.
- **LD-MIG-04:** CDC (Debezium) is the recommended default migration strategy for most enterprises.
- **LD-MIG-05:** Wasm Sandbox is a research module (Phase 8). Do not promise to enterprises until .NET→Wasm matures.
- **LD-CLI-01:** `vvctl` is the single operational tool for the entire VectorVolt ecosystem — no separate admin portals.
- **LD-CLI-02:** `vvctl` communicates with the cluster via gRPC (same polyglot protocol). It never touches engine internals directly.
- **LD-CLI-03:** TUI dashboard reads from TscMetricsBuffer/ShadowMetricsExporter — zero impact on hot path.
- **LD-RB-01:** Ring buffer wait strategies are configurable per consumer via YAML. Hot path = BusySpin, background = Sleeping/Blocking.
- **LD-RB-02:** Consumer DAG topology is YAML-defined. Gating enforces pipeline ordering.
- **LD-RB-03:** Back-pressure policy is configurable per tenant. Default: block (never lose data).
- **LD-IDEM-01:** Idempotency keys are mandatory for all commands. Engine enforces exactly-once within configurable window.
- **LD-2PC-01:** Two-phase transactions are YAML-defined. Timeout-based auto-void is deterministic (engine clock, not wall clock).
- **LD-MV-01:** Materialized views update atomically with the source entity mutation. No separate aggregation job.
- **LD-QOS-01:** Workload tiers are per-tenant. Critical tier always preempts batch tier under contention.
- **LD-VS-01:** Versionstamps are for cross-shard ordering only. Intra-shard uses SequenceNumber (zero overhead).
- **LD-INTC-01:** Interceptors run in the projection milli-path only; never on the nano-path. User code never executes inside the core engine.
- **LD-DIST-01:** NativeAOT single binary, no runtime dependency. Every release artifact is fully self-contained.
- **LD-DIST-02:** All major OS package managers supported. Users install VectorVolt through their platform's native package manager.

---

## 16. Out of Scope (Permanently)

- Full distributed consensus (Raft/Paxos) beyond log replication.
- Global secondary indexes across shards.
- Real-time analytics inside the execution engine.
- CUDA/GPU acceleration (wrong tool for OLTP).
- PMem/Optane (dead technology).
- SIMD JSON shredding (binary protocol only, no JSON on hot path).
- Custom TCP stack (DPDK handles this).
- Intel TSX (dead — disabled via microcode, security vulnerabilities).

---

*Version 3.0 — Competitive gap analysis complete. Added: configurable wait strategies, consumer DAGs, back-pressure policies, idempotency keys, two-phase transactions, materialized views, workload prioritization (QoS), cluster-wide versionstamps (core features); TTL expiration, atomic multi-ops, probabilistic data structures, transactional watches, write-verify, load generator, warm restart (research modules). Feature parity achieved with LMAX, TigerBeetle, VoltDB, Redis, Aerospike, SingleStore, SAP HANA, FoundationDB, and ScyllaDB. Review and confirm before implementation begins.*
