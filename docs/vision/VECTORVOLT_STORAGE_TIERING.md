# VectorVolt — Storage Tiering, PITR, and Projection Workers
## L0–L4 Five-Level Stack, Point-in-Time Recovery, Kafka-Inspired Event Projection

**Status:** Architecture reference — locked decisions implemented across Phases 2-4  
**Related:** `VECTORVOLT_LOCKED_DECISIONS.md` (LD-COLD, LD-PITR, LD-RETAIN, LD-PROJ, LD-KAFKA, LD-DLQ, LD-SNAP)  
**Captured:** 2026-07-09

---

## Part 1: Five-Level Storage Hierarchy

### The L0–L4 Stack

```
┌──────────────────────────────────────────────────────────────────┐
│ L0: CPU Registers + L1/L2/L3 Cache                               │
│     Size: KB–MB range                                            │
│     Latency: 0.3 ns (register) → 40 ns (L3 miss)                │
│     What lives here: CommandFrame being processed, hot counters  │
│     Managed by: CPU hardware (automatic)                         │
├──────────────────────────────────────────────────────────────────┤
│ L1: DRAM — mlock'd Arena                                         │
│     Size: 64 MB → 64 GB per shard (configurable)                 │
│     Latency: 60–100 ns                                           │
│     What lives here: Arena state, Cuckoo Hash index, WAL buffer  │
│     Managed by: ArenaAllocator (VectorVolt, per-shard)           │
│     Allocation: MAP_HUGETLB + mlock (no swap, no TLB misses)     │
├──────────────────────────────────────────────────────────────────┤
│ L2: NVMe Local SSD — io_uring + Direct I/O                       │
│     Size: TB range                                               │
│     Latency: 10–50 µs                                            │
│     What lives here: WAL segments, snapshots, cold arena pages   │
│     Managed by: WalManager + SnapshotEngine (VectorVolt)         │
│     I/O path: io_uring submission queue (zero-copy, async)       │
├──────────────────────────────────────────────────────────────────┤
│ L3: Network SSD — EBS, NFS, network block storage                │
│     Size: TB range                                               │
│     Latency: ~1 ms                                               │
│     What lives here: WAL archive, old snapshots, audit trail     │
│     Managed by: RetentionPolicy + ArchiveEngine (VectorVolt)     │
│     Transition: policy-driven (age, size, compliance)            │
├──────────────────────────────────────────────────────────────────┤
│ L4: Object Storage — S3, Azure Blob, GCS                         │
│     Size: unlimited                                              │
│     Latency: seconds                                             │
│     What lives here: long-term WAL archive, compliance snapshots │
│     Managed by: ObjectArchiveWorker (VectorVolt)                 │
│     Transition: before any L2 deletion                           │
└──────────────────────────────────────────────────────────────────┘
```

### L0 → L1 Management: Pre-Fetch Engine

The Pre-Fetch Engine uses a **Markov Chain Transaction Dependency Graph** to predict
which arena pages will be needed next and promotes them to L0 (CPU cache) before they
are needed:

```rust
pub struct TransactionDependencyGraph {
    pub transition_matrix: HashMap<(PageId, PageId), f64>,  // transition probability
    pub access_history:    RingBuffer<(PageId, Instant)>,   // recent access log
    pub temporal_decay:    f64,                             // older observations weight less
}

pub struct PrefetchEngine {
    pub dependency_graph: TransactionDependencyGraph,
    pub prefetch_budget:  usize,   // max pages to pre-fetch per tick
    pub tier_1_cache:     BTreeSet<PageId>,  // currently in L1
}
```

Pre-fetch issues `PREFETCHT1` x86 hints (or equivalent ARM `PRFM` instructions) to
bring pages into L2/L3 CPU cache ahead of the arena access.

### L1 → L2 Transition: Snapshot + WAL Compaction

Arena pages are persisted to L2 (NVMe) via the snapshot mechanism:

```
Trigger: elapsed since last snapshot > snapshot_interval
         OR arena fill rate > 80% (emergency snapshot)
         OR operator manual trigger

Process:
  1. Capture snapshot_start_lsn (atomic read, no pause to execution)
  2. Serialize arena pages to NVMe via io_uring (async, zero-copy)
  3. Write snapshot manifest (snapshot_id, lsn, size, crc)
  4. WAL compaction: truncate WAL segments before snapshot_start_lsn
     (they are archived to L3/L4 first — LD-RETAIN)
  5. Update PITR catalog with new snapshot record
```

### L2 → L3 → L4 Transition: Retention Policy

Three retention policy types control L2→L3→L4 movement:

```toml
# Example shard retention policy (vectorvolt.toml)
[shard.retention]
  wal_retention_days   = 30        # Keep WAL on L2 for 30 days
  snapshot_count_keep  = 5         # Keep last 5 snapshots on L2
  archive_to_l3_after  = "30d"     # Move to L3 after 30 days
  archive_to_l4_after  = "90d"     # Move to L4 after 90 days
  compliance_lock      = "7y"      # Financial: 7-year WORM on L4
  delete_after         = never     # Never delete if compliance_lock set
```

**Rule:** Nothing is ever deleted from L2 before being archived to L4.
The `ObjectArchiveWorker` writes to L4 (object storage), then verifies the write
with a checksum read, then deletes from L2. This guarantees PITR data is never lost.

---

## Part 2: Point-in-Time Recovery (PITR)

### PITR Catalog

The PITR Catalog tracks every snapshot and WAL segment with its time range:

```rust
pub struct PitrCatalog {
    pub snapshots: Vec<SnapshotRecord>,
    pub wal_segments: BTreeMap<Lsn, WalSegmentRecord>,
}

pub struct SnapshotRecord {
    pub snapshot_id:   Uuid,
    pub shard_id:      ShardId,
    pub start_lsn:     Lsn,
    pub wall_time_ns:  u64,   // nanosecond wall clock at snapshot start
    pub storage_tier:  StorageTier,  // L2, L3, or L4
    pub location:      String,  // path or object storage key
    pub size_bytes:    u64,
    pub crc32:         u32,
}

pub struct WalSegmentRecord {
    pub segment_id:    u64,
    pub shard_id:      ShardId,
    pub first_lsn:     Lsn,
    pub last_lsn:      Lsn,
    pub first_ts_ns:   u64,
    pub last_ts_ns:    u64,
    pub storage_tier:  StorageTier,
    pub location:      String,
}
```

### PITR Recovery Algorithm

```
Operator input: restore shard S to timestamp T

Step 1: Find the latest snapshot BEFORE T
  pitr_catalog.snapshots
    .filter(|s| s.shard_id == S && s.wall_time_ns <= T)
    .max_by_key(|s| s.wall_time_ns)
  → base_snapshot (may be on L2, L3, or L4 — fetch from wherever it is)

Step 2: Find all WAL segments BETWEEN base_snapshot.start_lsn AND T
  pitr_catalog.wal_segments
    .range(base_snapshot.start_lsn..)
    .filter(|seg| seg.first_ts_ns <= T)

Step 3: Restore
  a. Fetch base_snapshot from storage, verify CRC
  b. Load snapshot into fresh arena
  c. Replay WAL segments in LSN order
  d. Stop replay at the first WAL entry where entry.timestamp_ns > T
     → This is the "target LSN"

Step 4: Start shard at target LSN
  The recovered shard has exactly the state it had at timestamp T.
  p99 restoration time for 100GB shard: < 60 seconds
```

### PITR Monitoring

PITR health metrics (always exported to Prometheus):

| Metric | Description |
|---|---|
| `vv_pitr_oldest_recovery_point_age_seconds` | How far back PITR can restore |
| `vv_wal_segment_count_l2` | WAL segments currently on local NVMe |
| `vv_snapshot_age_seconds` | Age of most recent snapshot |
| `vv_pitr_archive_lag_bytes` | Bytes not yet archived to L4 |

Alert thresholds:
- `snapshot_age_seconds > 3600`: WARN (no snapshot in 1 hour)
- `pitr_archive_lag_bytes > 10GB`: WARN (archiver falling behind)
- `wal_segment_count_l2 > 10000`: WARN (retention policy may not be running)

---

## Part 3: Copy-on-Write Snapshots

### CoW Snapshot Internals

VectorVolt snapshots use Copy-on-Write semantics at the arena page level.
A snapshot "view" is a set of page references — not a copy of the data:

```rust
pub struct ArenaSnapshot {
    pub snapshot_id:  Uuid,
    pub lsn:          Lsn,
    pub page_refs:    Vec<(PageId, ArenaOffset)>,  // references into arena pages
    pub ref_counts:   Arc<PageRefCounts>,          // shared with live arena
}
```

When a snapshot is taken:
1. The snapshot holds references to current arena pages
2. The live arena continues processing commands
3. When a command mutates a page that a snapshot references: CoW triggers
   - Allocate new page from live arena
   - Copy current page content to new page
   - Update live arena pointer to new page
   - Snapshot still holds reference to original page
4. Snapshot serializes original pages to NVMe at its own pace
5. After serialization complete: snapshot releases references → pages reclaimed

**No blocking.** The live arena is never paused during snapshot creation.
The only overhead is the CoW copy when a mutated page is still referenced by an
in-progress snapshot.

### Incremental Snapshots

After the first full snapshot, VectorVolt takes incremental snapshots:

```
Full snapshot:        Serialize all arena pages → L2 NVMe
                      Typical size: full arena size (e.g., 16 GB)

Incremental snapshot: Serialize only pages modified since last snapshot
                      Typical size: 1–5% of arena (depending on write rate)

Restoration:          Full snapshot + apply incremental snapshots in order
                      This is exactly what PITR does with WAL (same pattern)
```

---

## Part 4: Projection Worker Architecture (Kafka-Inspired)

### What Projections Are

Projections are denormalized read models built from domain events. They are the
CQRS "read side" — maintained asynchronously after every committed command.

**Example: OrderSummaryProjection**
- Command: `PlaceOrderCommand { customer_id, items, total }`
- Domain event emitted: `OrderPlacedEvent { order_id, customer_id, total, ts }`
- Projection worker: inserts row into `order_summary` table
- Read query hits `order_summary` (fast, indexed)
- Not the primary domain store (slow, event-sourced)

### Event Log Architecture

The domain event stream is implemented on top of the WAL — the WAL IS the event log:

```
WAL LSN: monotonically increasing sequence number
         same as a Kafka partition offset

Domain event payload in WAL:
  [lsn: u64][timestamp_ns: u64][event_type: u16][event_payload: bytes]

Projection worker keeps its offset:
  projection_offsets table:
    worker_id | shard_id | last_processed_lsn
    ──────────┼──────────┼────────────────────
    "orders"  | 0        | 1,847,293
    "orders"  | 1        | 1,847,290  (3 behind — normal)
    "metrics" | 0        | 1,847,100  (193 behind — investigate)
```

### Projection Worker Execution Model

```rust
pub struct ProjectionWorker {
    pub worker_id:       String,
    pub shard_id:        ShardId,
    pub current_offset:  Lsn,
    pub batch_size:      usize,  // read this many events per tick
    pub sql_writer:      Arc<dyn SqlWriter>,
}

impl ProjectionWorker {
    pub async fn tick(&mut self) -> Result<usize, ProjectionError> {
        // Read a batch of events from WAL starting at current_offset
        let events = self.wal_reader
            .read_events(self.shard_id, self.current_offset, self.batch_size)
            .await?;

        if events.is_empty() { return Ok(0); }

        // Write batch to SQL projection store (bulk INSERT)
        self.sql_writer.bulk_insert(&events).await?;

        // Update offset (durable, survives restart)
        self.offset_store.update(self.worker_id, events.last_lsn()).await?;
        self.current_offset = events.last_lsn() + 1;

        Ok(events.len())
    }
}
```

### Consumer Lag as First-Class Metric

```
vv_projection_consumer_lag{worker="orders", shard="0"}    = 3      # healthy
vv_projection_consumer_lag{worker="metrics", shard="0"}   = 193    # borderline
vv_projection_consumer_lag{worker="audit", shard="0"}     = 50000  # ALERT

Alert: consumer_lag > 10000 for more than 5 minutes → page on-call
```

### Log Compaction

WAL segments are compacted after projection workers have consumed them:

```
Compaction eligibility check:
  - All projection workers for this shard have advanced past segment_last_lsn?
  - Snapshot has been taken that includes this segment's LSN range?
  - Retention policy allows deletion?
  If all three: segment is eligible for compaction (archive to L4, delete from L2)
```

Compaction does NOT lose data — segments are archived to L4 before deletion.
PITR can still restore to any point using L4-archived segments.

---

## Part 5: Dead Letter Queue

### Why DLQ

Projection workers can fail:
- SQL connection lost temporarily
- Projection SQL schema out of sync with event schema (deployment issue)
- Event payload malformed (should not happen, but guard for it)
- Downstream system temporarily unavailable

Without a DLQ, a failed projection worker would retry infinitely, blocking progress.

### DLQ Behavior

```
Projection worker fails event E after N retries:
  1. Write E to DLQ (persistent, separate table: `projection_dlq`)
  2. Log: event_id, worker_id, error_message, failure_count, last_attempt_ts
  3. Worker PAUSES (stops consuming new events)
  4. Alert fires: "ProjectionWorker 'orders' paused — events in DLQ"
  5. Operator investigates: fix schema mismatch, replay DLQ, resume worker

DLQ schema:
  worker_id | shard_id | event_lsn | event_payload | error | failed_at
```

**Workers do not silently skip failed events.** A silent skip would cause the
projection to diverge from the domain state — a data integrity violation.
The DLQ makes failures visible and forces operator action.

### DLQ Replay

After fixing the root cause:
```
vvctl projection replay-dlq --worker orders --shard 0
```
This replays all DLQ entries for the worker in LSN order, then resumes normal consumption.

---

## Part 6: Retention Policy Reference

### Policy Types

| Policy | Use Case | Example |
|---|---|---|
| Time-based | Standard operational | Keep WAL for 30 days |
| Size-based | Storage-constrained | Keep WAL up to 500 GB |
| Compliance-based | Financial/healthcare | Keep WAL for 7 years, WORM |
| Snapshot-count | Simple operational | Keep last 10 snapshots |

### Compliance Lock (WORM Semantics)

When `compliance_lock` is set, archived segments on L4 (object storage) are written
with WORM (Write Once Read Many) semantics:
- S3: Object Lock with Governance mode (admin can override) or Compliance mode (immutable)
- Azure Blob: Immutable Blob Storage policy
- GCS: Object holds (retention policy)

**VectorVolt enforcement:** Before issuing any L4 delete operation, the engine checks
the compliance clock. If `now() < archived_at + compliance_lock_duration`, the delete
is rejected with `ComplianceLockViolation` error — even if the operator requests it.

---

*Source decisions: LD-COLD, LD-PITR, LD-RETAIN, LD-PROJ, LD-KAFKA, LD-DLQ, LD-SNAP, LD-GC*  
*From: `C:\vega\vega_requirements.md`, `C:\ultra-fast-oltp\docs\state-continuation.md`, `C:\ultra-fast-oltp\docs\hld-lld-architecture.md`*
