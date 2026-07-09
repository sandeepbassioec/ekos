# VectorVolt — Locked Decisions Register
## Architecture Decisions That Are NOT Open For Debate

**Status:** Authoritative — these decisions were locked after deep analysis in the
C# prototype (Vega, ultra-fast-oltp) and carried forward to the Rust implementation.
**Sources:** `C:\vega\vega_lld_standards.md` (55 LDs), `C:\ultra-fast-oltp\docs\state-continuation.md` (48 LDs)  
**Captured:** 2026-07-09

---

> A "Locked Decision" (LD) is a design choice that was debated, analyzed, and closed.
> It cannot be reopened without restarting the system design from scratch.
> Future contributors must implement these decisions, not relitigate them.

---

## Group 1: Correctness and ACID

### LD-ACID: Full ACID at Shard Level

VectorVolt provides full ACID guarantees within a single shard. Cross-shard transactions
use Saga (compensating transactions) or 2PC (two-phase commit) — never distributed locking.

**Rationale:** Distributed locking kills latency. Saga is the correct pattern for
cross-shard correctness at scale. Every transaction that can be designed to fit within one
shard should be — schema design and shard key selection matter.

### LD-SNAP: Copy-on-Write Snapshots

Snapshots use Copy-on-Write semantics. WAL segments are compacted after a successful snapshot.
Snapshot does not block command processing — it captures the state at the snapshot start
LSN and serializes it asynchronously while the engine continues.

**Rationale:** Blocking snapshots create latency spikes that violate the sub-millisecond SLA.

### LD-REPLAY: Deterministic Replay Guarantee

Given the same commands from the same WAL log, the engine produces identical state.
This is a hard invariant. Any non-determinism (time, random, external calls) is injected
via the `DeterminismGuard` interface — handlers never call `Instant::now()` or `rand::random()` directly.

**Rationale:** Log replay is the foundation of crash recovery, replication, and testing.
Non-determinism breaks all three simultaneously.

### LD-CHAOS: Built-in Chaos Testing

The engine ships with a built-in fault injection framework. Any test can trigger:
network partitions, disk I/O failures, process crashes, memory pressure, clock skew.
No external chaos tool required.

**Rationale:** A database that cannot be tested for failure modes under controlled conditions
is a database that will fail under uncontrolled conditions.

---

## Group 2: Storage Tiering

### LD-COLD: Five-Level Storage Hierarchy

VectorVolt defines 5 storage tiers, not 3:

| Level | Name | Storage | Latency |
|---|---|---|---|
| L0 | CPU Cache | Registers + L1/L2/L3 cache | < 1 ns – 40 ns |
| L1 | DRAM / Arena | mlock'd + MAP_HUGETLB | < 100 ns |
| L2 | NVMe Local SSD | io_uring, direct I/O | < 10 µs |
| L3 | Network SSD | NFS/EBS/network block | < 1 ms |
| L4 | Object Storage | S3, Azure Blob, GCS | seconds |

Promotion/demotion between tiers is automatic. The pre-fetch engine manages L0–L2.
L3–L4 transitions are policy-driven (age-based, size-based, or compliance-based).

**Rationale:** The distinction between L2 (local NVMe) and L3 (network SSD) is critical
for cloud deployments — these are fundamentally different latency classes and require
different I/O strategies.

### LD-PITR: Point-In-Time Recovery to Any Millisecond

VectorVolt supports PITR (Point-In-Time Recovery) — the ability to restore to any
millisecond in the past, not just to the last snapshot. This is implemented by:
1. Maintaining WAL segments back to the PITR retention horizon
2. On recovery: restore last snapshot before target time, then replay WAL to the exact LSN
   corresponding to the target timestamp

**Retention policy:** Configurable per-shard. Financial compliance may require 7 years.
Standard operational recovery requires 30 days.

**Rationale:** "Restore to last backup" is insufficient for production OLTP. Losing
hours of transactions is not acceptable. PITR is a non-negotiable for financial-grade systems.

### LD-RETAIN: Configurable Retention Policies

Three retention policy types per shard:
- **Time-based:** Keep WAL and snapshots for N days
- **Size-based:** Keep WAL and snapshots up to X GB
- **Compliance-based:** Keep for legally mandated duration (financial: 7 years; healthcare: varies)

WAL segments beyond retention horizon are automatically archived to L4 (object storage)
before deletion — they are never discarded without archiving.

---

## Group 3: Replication and Clustering

### LD-SWIM: SWIM Protocol for Cluster Membership

Cluster membership (which nodes are alive) is managed by **SWIM (Scalable Weakly-consistent
Infection-style Membership)** protocol — a gossip-based protocol, NOT Raft.

Raft is used for **leader election and log replication within a shard group**.
SWIM is used for **cluster-wide membership and failure detection**.

```
SWIM characteristics:
  - O(log n) propagation of membership changes
  - Scales to 800+ node clusters
  - Configurable: suspicion period before marking a node dead
  - No single point of failure for membership state
```

**Rationale:** Raft does not scale to large cluster sizes for membership management.
SWIM provides the eventual consistency needed for membership at the scale of hundreds of nodes.

### LD-VNODE: 4096 Virtual Nodes in Consistent Hash Ring

Data is distributed across shards via a **consistent hash ring with 4096 virtual nodes**.
Each physical shard owns a contiguous arc of the virtual node range.

```
Physical shard count: 1 to 65,536
Virtual nodes: always 4096 (fixed)
Virtual nodes per physical shard: 4096 / physical_shard_count

For 16 shards: 256 virtual nodes per shard
For 1024 shards: 4 virtual nodes per shard
```

**Why 4096 virtual nodes:** Provides even distribution without requiring shard rebalancing
for small cluster changes. When a shard is added or removed, only 1/4096 of the data moves.

### LD-EPOCH: Epoch Fencing for Split-Brain Prevention

The cluster maintains a monotonically incrementing epoch number. Any shard leader from
a previous epoch is rejected by quorum. This prevents split-brain: a partitioned leader
cannot commit writes after the cluster has elected a new leader.

**Epoch increment triggers:** topology change, leader election, shard split/merge, failover.

### LD-FAILOVER: < 2 Second Failover Target

Leader election and failover must complete in < 2 seconds end-to-end:
1. Heartbeat timeout detection: 500ms default
2. Raft election: typically completes in 1–2 RTTs
3. Epoch fence: all followers reject old leader immediately on new epoch
4. Clients detect new leader via ShardCatalog refresh

### LD-QUORUM: Quorum Write Semantics

All writes require acknowledgment from a **majority of replicas** before commit.
Configurable: synchronous (all replicas, stronger durability) or async (majority, lower latency).

Default: majority quorum. Financial-grade deployments use full synchronous replication.

---

## Group 4: Memory and Performance

### LD-GC: Zero GC on Hot Path

No heap allocation occurs on the command processing hot path. All hot-path objects
are allocated from per-shard arenas (bump allocators). Arena compaction is triggered
by snapshot completion, not by GC.

**Two arenas per shard:**
- "Live" arena: active commands being processed
- "Stable" arena: committed state waiting for snapshot
After snapshot: stable arena is serialized and reclaimed via bump-reset (no GC pause).

### LD-ARENA: Arena Size Configuration

Each shard gets a dedicated `ArenaAllocator`. Size is configurable at shard creation:
- Edge nodes: 64 MB per shard
- Standard cloud: 4–16 GB per shard
- Large cloud instances: up to 64 GB per shard

Arena overflow (shard exceeds arena size without snapshot) triggers an emergency snapshot.

### LD-BATCH: Adaptive Batching

Command batch sizes adapt to workload:
- **Under load:** grow batch size to maximize throughput (up to 10,000 commands/fsync)
- **Idle:** shrink batch size to minimize latency (down to 1 command/fsync for interactive workloads)
- **Backpressure:** when queue depth exceeds configurable threshold, reject new commands with a retriable error

**Rationale:** Fixed batch sizes optimize for either throughput or latency, never both.
Adaptive batching achieves both across different workload phases.

### LD-BACK: Token Bucket Backpressure

Inbound commands are rate-limited via a **token bucket algorithm** per shard.
When the shard's command queue fills, the token bucket is drained and new commands
receive a `429 TooManyRequests` with a `Retry-After` header.

**Client behavior:** all VectorVolt SDKs implement exponential backoff with jitter
on 429 responses. Clients do not busy-spin.

---

## Group 5: CQRS and Binary Architecture

### LD-READ-01: Read Fabric is a Separate Binary

The read path (queries, projections, analytics) runs as a **separate binary process**
from the write engine. This is CQRS at the binary level, not just the code level.

```
vectorvolt-server   — write engine binary (WAL, MVCC, shard execution)
vectorvolt-reader   — read fabric binary (SQL queries, SIMD scans, projections)
vectorvolt-control  — control plane binary (ShardCatalog, policy engine, admin)
```

**Rationale:** Read and write workloads have fundamentally different resource profiles.
Running them in separate processes enables independent scaling, separate CPU affinity
policies, and failure isolation.

### LD-READ-02: Stale Reads by Default, Consistent Reads Opt-In

The read fabric serves **stale reads** by default (reads from a slightly behind snapshot).
Consistent reads (guaranteed to see all committed writes as of the query time) are opt-in
and more expensive.

```
Default query:           may be stale by up to 50ms (configurable)
Consistent query:        SELECT /*+ consistent */ ...
                         routes to the shard leader, not a follower
```

**Rationale:** 99% of OLTP reads do not require strict consistency. Stale reads from
followers eliminate cross-shard coordination and reduce leader load.

### LD-RUNTIME-01: Role-Based Binary Deployment

Three distinct binary roles, each independently deployable and scalable:
- **Write Node:** runs `vectorvolt-server`, owns WAL + arena + shard execution
- **Read Node:** runs `vectorvolt-reader`, serves SQL queries, SIMD scans
- **Control Node:** runs `vectorvolt-control`, manages ShardCatalog, policy engine

A single-node deployment runs all three binaries on the same machine. A distributed
cluster separates them onto different node pools.

---

## Group 6: Projection Architecture

### LD-PROJ: Projections are Eventually Consistent

Projections (denormalized SQL views built from domain events) are **eventually consistent**.
They are never updated synchronously on the command path.

```
Command path:     CommandFrame → WAL → arena state update (sync, fast)
                  ↓
                  Domain event emitted to event router (async, fire-and-forget)
                  ↓
Projection path:  Projection worker reads event → bulk SQL insert (async, slow is fine)
```

**Rationale:** Synchronous projection writes on the command path would multiply write
latency by the number of projections. A busy system with 20 projections cannot afford
to wait for all 20 SQL inserts on every command.

### LD-KAFKA: Kafka-Inspired Event Log for Projections

The domain event stream uses **Kafka-inspired semantics**:
- Sequential, append-only log (exactly like a Kafka topic/partition)
- Each projection worker maintains its own **offset pointer** into the event log
- Log compaction removes superseded events while preserving ordering
- **Consumer lag** (how far behind each projection worker is) is a first-class metric

**Why not actual Kafka:** VectorVolt's event stream is internal to the cluster.
An external Kafka dependency adds operational complexity, network latency, and a
failure point for an internal concern. The WAL IS the event log — projections read
directly from WAL segments.

### LD-DLQ: Dead Letter Queue for Failed Projections

Projection writes that fail after max retries go to a **Dead Letter Queue (DLQ)**
with the full event context. The DLQ is monitored. Failed projection workers pause and
alert — they do not silently fall further behind.

---

## Group 7: SDK and External API

### LD-SDK-01: Polyglot SDKs in 5 Languages

The VectorVolt SDK is available in: **C#, Java, TypeScript/JavaScript, Python, Go**.
Rust is the primary SDK (the engine itself). All non-Rust SDKs are generated from the
same protobuf definitions.

**SDK safety guarantees built into the SDK (not just the engine):**
- Commands must implement the `Command` interface (compile-time check in typed languages)
- Handler determinism: DeterminismGuard wraps every handler call
- Command versioning: schema changes require explicit migration handler registration
- Backward compatibility: engine supports N-2 command schema versions simultaneously

### LD-PG-WIRE: PostgreSQL Wire Protocol Compatibility

VectorVolt speaks the **PostgreSQL wire protocol** on its SQL query port.
This means any PostgreSQL client library (psql, pgAdmin, JDBC, Npgsql, psycopg2,
the Go `pgx` driver) can connect to VectorVolt without modification.

```
Port 5432 (configurable): PostgreSQL wire protocol (libpq compatible)
Port 50051 (configurable): gRPC (native high-performance interface)

The PostgreSQL wire protocol port serves:
  - SQL SELECT queries (read fabric)
  - Limited DDL (CREATE TABLE, CREATE INDEX — translated to VectorVolt schema operations)
  - NOT: direct WAL access, command submission (use gRPC for those)
```

**Rationale:** Ecosystem compatibility. Every database tool, ORM, and BI tool already
speaks PostgreSQL wire protocol. This gives VectorVolt instant tooling support.

---

## Group 8: Multi-Tenancy

### LD-MT-01 through LD-MT-08: Eight Multi-Tenancy Decisions

| LD | Decision |
|---|---|
| LD-MT-01 | Tenant isolation at shard group level — each tenant gets dedicated shard groups |
| LD-MT-02 | Cross-tenant queries are prohibited at the engine level (not just policy) |
| LD-MT-03 | Tenant onboarding: control plane API, < 100ms activation |
| LD-MT-04 | Tenant metering: command count + storage GB + projection lag — all per-tenant |
| LD-MT-05 | Data residency: each tenant's shard group can be placed in a designated cloud region |
| LD-MT-06 | Shared infrastructure mode: multiple tenants on same nodes (logical isolation) |
| LD-MT-07 | Dedicated infrastructure mode: hardware-level isolation (dedicated node pool per tenant) |
| LD-MT-08 | Tenant SLA: each tenant's QoS policy is independently configurable |

---

## Group 9: Observability and Performance Engineering

### LD-OBS: Built-in Profiling

Every production binary includes built-in profiling via:
- Linux `perf` (CPU profiling, PMU events)
- eBPF-based flame graph capture (no kernel recompile)
- Always-on latency histograms per operation type

**No external APM agent required.** Profiling is embedded.

### LD-GATE: Performance Gates in CI

Every pull request runs performance benchmarks. Benchmarks are **gates** — a PR
that regresses p99 latency by > 5% is rejected by CI automatically. No human review
bypasses a performance gate.

**Rationale:** Latency regressions are nearly invisible in code review. They appear
only under load. Automated gates catch them before merge.

### LD-ADAPT: Adaptive Self-Tuning

The engine continuously measures:
- Observed batch fsync latency
- Queue depth per shard
- Memory pressure (arena fill rate)

And adjusts:
- Batch commit window (grow when queue is deep, shrink when idle)
- Pre-fetch budget (reduce when memory pressure is high)
- Backpressure threshold (adjust to maintain target queue depth)

This is **closed-loop adaptive tuning** — the engine optimizes itself without operator intervention.

---

## Group 10: Reference Domain Implementations

VectorVolt ships with three reference domain implementations that serve as both
integration tests and adoption blueprints:

### RD-1: Financial Ledger
Double-entry bookkeeping: accounts, debits, credits, balance queries.
Validates: ACID correctness, sub-millisecond write latency, concurrent balance queries.

### RD-2: Workflow Engine
State machine execution: task creation, approval chains, audit trail.
Validates: event sourcing model, projection consistency, multi-step transactions.

### RD-3: Order Management
Order lifecycle: creation, inventory reservation, fulfillment, cancellation.
Validates: FK constraint enforcement, cross-shard saga, projection rebuild.

---

## Performance Targets (Locked)

| Metric | Target | Source |
|---|---|---|
| p99 command latency (memory path) | < 10 µs | `vega_requirements.md` |
| p99 command latency (including WAL fsync) | < 1 ms | `state-continuation.md` |
| p99.9 command latency (memory path) | < 10 µs | `vega_requirements.md` |
| Single-shard throughput | 500K+ commands/sec | `state-continuation.md` |
| Cluster throughput | 10M+ commands/sec | `state-continuation.md` |
| Shard recovery time (100GB shard) | < 60 seconds | `state-continuation.md` |
| Max shards per cluster | 65,536 | `vega_requirements.md` |
| Max cluster nodes | 800+ | `vega_requirements.md` |
| Availability target | 99.999% (five nines) | `state-continuation.md` |
| Failover time | < 2 seconds | `state-continuation.md` |
| Tenant onboarding time | < 100ms | `state-continuation.md` |
| Read latency (stale default) | < 120ns (L1 hit) | `CPU_CACHE_NUMA_ISOLATION.md` |

---

*These decisions cannot be changed without restarting the system design.*
*If you disagree with a decision, write a formal counter-proposal with benchmarks.*
*Opinion without data does not reopen a locked decision.*
