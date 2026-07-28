# Storage Engine Advanced Capabilities
## VectorVolt — QoS, Durability, Observability, Testing, and Index Intelligence

**Status:** Architecture Capture (Phase 3–5)
**Captured:** 2026-07-09

---

## 1. Transaction Prioritization (QoS Isolation)

Not all transactions are equal. A high-value financial settlement and a metrics logging call
must not compete on the same scheduler queue.

### Priority Model

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransactionPriority {
    Critical = 1,   // financial settlement, payment commit, SLA-bound operation
    High     = 2,   // user-facing write, order placement
    Normal   = 3,   // background analytics, secondary writes
    Low      = 4,   // metrics logging, audit trails, telemetry writes
}
```

Priority is assigned per-connection by EKOS based on **business context scores**:

```toml
# ekosd.toml
[qos]
critical_patterns = ["payment_", "settlement_", "order_confirm"]  # table name prefix
high_patterns     = ["orders", "invoices", "claims"]
low_patterns      = ["metrics", "logs", "telemetry", "audit"]
```

EKOS inspects the target table name at parse time and assigns priority in ~10 ns.

### Strict Priority Queue

The per-core transaction scheduler uses a **strict priority queue** with a CPU slice budget:

```
Critical → dedicated CPU slice, guaranteed < 200 ns queue wait
High     → next slot after any pending Critical
Normal   → scheduled in FIFO within Normal band
Low      → scheduled only when no Critical/High is pending
```

Under load spike, Low transactions back-pressure (queue fills, client gets backpressure signal)
while Critical transactions maintain their SLA. No starvation guard is needed in the initial
design — Critical/High workloads are expected to be a small fraction of total volume.

### Admin Panel Exposure

Admin panel shows per-priority throughput and queue depth in real time. DBA can override
a connection's priority mid-session via ekosctl:

```bash
ekosctl qos override --connection=<id> --priority=critical --reason="incident response"
```

Override stored as immutable audit KO.

**Phase placement:** Phase 3 (implement after WAL and core scheduler are stable).

---

## 2. HITL Index Advisor — Human-in-the-Loop Index Tuning

VectorVolt should not blindly create or drop indexes. The system proposes, the DBA decides,
and the system learns from the decision.

### Advisor Pipeline

```
Execution Plan Analysis (per query, background):
  Capture: table scan instead of index scan (missed index opportunity)
  Capture: index scan with high filter ratio (stale/wrong index)
  Capture: join with no index on FK column

        ↓

Index Recommendation Engine (EKOS, runs nightly):
  For each missed opportunity: compute estimated benefit (rows/sec improvement)
  Rank by expected impact
  Deduplicate against existing pending recommendations

        ↓

Admin Panel: "Index Advisor" section
  Shows: recommended index, query that triggered it, estimated speedup, table size
  DBA clicks: Accept / Override / Dismiss

        ↓

If Accept:
  CREATE INDEX runs in Shadowbox("qa") first
  Test suite verifies no regression
  PromotionEngine evaluates
  Auto-promote or await UAT approval

If Override (DBA says "no, this index is wrong"):
  Decision stored as training data
  Advisor learns team's risk preference

If Dismiss:
  Suppressed for 30 days
```

### Learning Over Time

The advisor builds a **risk tolerance model** from DBA accept/override history:

```rust
pub struct DbaPreference {
    pub accepts_covering_indexes:  bool,
    pub accepts_partial_indexes:   bool,
    pub max_table_size_mb_for_auto: u64,   // auto-propose only if table < this
    pub preferred_index_types:     Vec<IndexType>,
}
```

After 3-6 months, the advisor stops proposing index types the DBA consistently rejects.
Accepted patterns (e.g., "always index FK columns on tables > 100K rows") become automatic
recommendations without human review after enough confirmations.

**Phase placement:** Phase 4 (after behavioral fingerprinting provides execution plan metrics).

---

## 3. Synthetic Data Generator

For load testing, chaos engineering, and staging environment setup — without touching
production data.

### How It Works

The generator reads:
1. VectorVolt schema (column types, FK relationships, CHECK constraints, NOT NULL)
2. Column histograms from the statistics engine (value distribution, cardinality, null rate)
3. FK graph (which tables reference which)

It then generates rows that are **statistically identical** to production data but completely
fabricated — no PII, no real customer records.

```rust
pub struct SyntheticDataConfig {
    pub target_tables:    Vec<String>,    // which tables to generate
    pub row_count:        HashMap<String, u64>,  // rows per table
    pub respect_fk:       bool,           // maintain referential integrity in generated data
    pub scale_factor:     f64,            // 1.0 = match production row count
    pub seed:             Option<u64>,    // for reproducible generation
}
```

**FK integrity:** Generator uses a topological sort of the FK graph to insert parent records
before child records. Generated IDs are tracked to ensure child FKs always point to valid
generated parents.

**Histogram fidelity:** For numeric columns, the generator samples from the production histogram
(stored in EKOS KOs). For string columns, dictionary encoding gives us real cardinality to
match. The result: generated data causes the same index selectivity, join cardinality, and
query plan behavior as production.

### Use Cases

```
Load testing:    Generate 10B rows matching production schema → benchmark VectorVolt TPS
Chaos testing:   Generate malformed edge cases to stress ARP's capture phase
Staging env:     Populate staging with realistic data for pre-prod testing
Benchmark:       Compare two algorithm implementations on identical datasets
```

**CLI:**

```bash
vvctl synthetic generate \
  --table orders --table customers --table line_items \
  --scale 10 \        # 10x production volume
  --output-db staging-vectorvolt:50051
```

**Phase placement:** Phase 3 (needed for ARP's PayloadFuzzer and staging env before Phase 4 load testing).

---

## 4. Transaction Fingerprint — Zero-Overhead Post-Mortem

Every VectorVolt response carries a **128-bit transaction fingerprint** in its metadata.
Debugging uses the fingerprint to retrieve full execution context — no tracing overhead
during normal operation.

### The Fingerprint

```rust
// Computed at commit time: ~5 ns (bit mixing)
pub struct TransactionFingerprint(u128);

impl TransactionFingerprint {
    pub fn compute(txn_id: u64, table_id: u32, timestamp_ns: u64) -> Self {
        // FNV-1a or xxHash — deterministic, non-cryptographic
        let mixed = txn_id ^ ((table_id as u64) << 32) ^ timestamp_ns;
        TransactionFingerprint(mixed as u128)
    }
}
```

Embedded in gRPC response metadata:

```
grpc-metadata:
  x-vv-txn: 7F3A9B2C11D4E5F68A0B1C2D3E4F5067
```

### The Context Store

At commit time, VectorVolt writes the **full execution context** asynchronously to a
cold-tier context store (separate from the hot KoStore):

```rust
pub struct TransactionContext {
    pub fingerprint:     TransactionFingerprint,
    pub query_ast:       String,            // the original query
    pub execution_plan:  ExecutionPlan,     // which indexes were used
    pub latency_ns:      u64,
    pub latch_waits:     Vec<LatchWait>,    // any contention during execution
    pub buffer_reads:    u32,               // pages touched
    pub delta_log_reads: u32,               // deltas applied on read
    pub tier_source:     TierSource,        // which memory tier served the data
    pub commit_time_ns:  u64,
}
```

The async write adds **zero latency to the transaction**. If the context store is under
pressure, context is simply not written (fingerprint still valid, just no stored context).

### Post-Mortem Debugging

When a client reports a slow or failed transaction, they provide the fingerprint:

```bash
vvctl debug txn 7F3A9B2C11D4E5F68A0B1C2D3E4F5067
```

VectorVolt retrieves the full context and shows:
- Which query ran
- Which plan was chosen (and why a different plan might be better)
- Which tier served the data (Tier 3 NVMe miss → pre-fetch gap → add to warm list)
- Any latch contention
- Exact timing breakdown

**Phase placement:** Phase 2 (lightweight — fingerprint computation is trivial; async context store
is a background writer on existing WAL infrastructure).

---

## 5. Cross-Partition Checksum — Paranoid Durability

Silent data corruption (bit-rot, memory drift, SSD firmware bugs) is real and dangerous.
We detect it in the background without blocking the write path.

### How It Works

Every Micro-Partition has a **CRC-32C checksum** computed at write time and stored in
the Partition Catalog (a KO in the KoStore):

```rust
pub struct PartitionCatalogEntry {
    pub partition_id:    u32,
    pub row_count:       u64,
    pub checksum_crc32c: u32,   // CRC of the raw partition bytes
    pub written_at:      u64,
    pub last_verified:   u64,
}
```

### Verification Workflow

A background task (`ChecksumVerifyWorkflow`) runs nightly:

```
For each Micro-Partition:
  Read partition bytes from storage
  Compute CRC-32C
  Compare against stored checksum
  If mismatch:
    → Emit alert KO: kind="partition_corruption"
    → Cross-check against hot-standby (WAL streaming replica)
    → If standby matches: source partition corrupted → restore from standby
    → If standby differs too: both corrupted → restore from cold backup + WAL replay
    → Page admin immediately (not just an alert — escalate to Critical)
```

### Quorum Witness (Distributed Mode)

In multi-node mode, each partition's checksum is also stored on two other nodes
(quorum witnesses). Verification compares the local checksum against both witnesses.

If local checksum matches witnesses: local is correct.
If local checksum differs from both witnesses: local is corrupted, witnesses serve as
the recovery source.

This detects **silent partial writes** (write succeeded but data is wrong) that would
otherwise only appear when the corrupted row is read.

**Phase placement:** Phase 3 single-node (local CRC only), Phase 5 distributed quorum witnesses.

---

## Summary of Phase Placement

| Capability | Phase |
|---|---|
| Transaction Prioritization (QoS) | Phase 3 |
| Transaction Fingerprint | Phase 2 |
| Synthetic Data Generator | Phase 3 |
| Cross-Partition Checksum | Phase 3 |
| HITL Index Advisor | Phase 4 |
| Quorum Witness Verification | Phase 5 |
