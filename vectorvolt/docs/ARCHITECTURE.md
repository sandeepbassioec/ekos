# VectorVolt Architecture

## Storage Engine (`vectorvolt-engine`)

### Write Path

```
insert(pk, values)
  │
  ├─► WAL append  (WriteOnly: skip fsync; Sync: fsync)
  │
  ├─► WriteBuffer (CuckooMap<u64, Row>)
  │      holds rows until flush_threshold bytes
  │
  ├─► AutoIndex patch   (all live BTreeMap indexes updated in place)
  │
  └─► TTL index update  (if insert_with_ttl)

flush()
  │
  ├─► Encode each column as Vec<u64> → binary partition file
  ├─► Write zone-map (col min/max per partition)
  ├─► Clear write buffer
  └─► auto_compact check → compact() if threshold reached
```

### Read Path

```
get(pk)
  ├─► CuckooMap probe  O(1) average
  └─► Partition scan if not in buffer  (mmap, page-cache hit)

scan(col, lo, hi)
  ├─► For each partition:
  │     zone-map prune: skip if [min,max] ∩ [lo,hi] = ∅
  │     decode column col only (single-column decode)
  │     SIMD filter: keep rows where col ∈ [lo, hi]
  │     get() each matching pk for full row materialisation
  └─► scan_deduped(): deduplicate by pk across partition copies
```

### Compaction

```
compact()
  ├─► Read all partitions, merge into a single sorted run
  ├─► Drop tombstoned rows
  └─► Rewrite as one partition file

Strategy (Leveled):
  l0_trigger:  compact when ≥ N partition files exist at L0
  min_run:     minimum files to include in a compaction run
```

### Secondary Indexes

```
build_secondary_index(col)
  ├─► scan_deduped(col, 0, u64::MAX)
  ├─► Build BTreeMap<value, BTreeSet<pk>>
  └─► Persist to index catalog file

On insert/delete: all live indexes are patched in O(log n) per index.

On open: catalog read → each listed index rebuilt via scan_deduped.
```

---

## Network Protocol (`vectorvolt-net`)

### Wire Frame

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
┌───────────────────────────────────────────────────────────────┐
│                        magic (4 bytes)                        │
├───────────────────────────────────────────────────────────────┤
│         opcode (2 bytes)        │      flags (2 bytes)        │
├───────────────────────────────────────────────────────────────┤
│                    payload_len (4 bytes)                      │
├───────────────────────────────────────────────────────────────┤
│                      crc32 (4 bytes)                          │
├───────────────────────────────────────────────────────────────┤
│                    request_id (8 bytes)                       │
├───────────────────────────────────────────────────────────────┤
│                   payload (payload_len bytes)                  │
└───────────────────────────────────────────────────────────────┘
Total header: 24 bytes (little-endian)
```

### Auth Handshake

```
Client                           Server
  │                                │
  │──── Hello ────────────────────►│
  │◄─── Challenge (nonce) ─────────│
  │                                │
  │  digest = SHA3-256(secret ‖ nonce)
  │  payload = [key_id_len][key_id][32][digest]
  │                                │
  │──── Auth (payload) ───────────►│
  │◄─── Ok ────────────────────────│
```

### Opcode Ranges

| Range | Category |
|---|---|
| 1–20 | Core CRUD: get, insert, delete, update, scan, scan-keys, scan-page |
| 21–50 | Field ops: set-field, increment, decrement, CAS, batch |
| 51–89 | Window/ranking: running-sum, rank, percentile, z-score, histogram |
| 90–109 | Range ops: copy-range, clamp-range, sum-product, nth-value |
| 110–122 | Aggregates: count/sum/min/max/avg range, increment/decrement field, set-min/max |

---

## HTTP Gateway (`vectorvolt-http`)

Axum 0.8 router; all handlers are async and share an `Arc<AppState>`.

```
AppState {
    registry: Arc<TableRegistry>,  // multi-table store
    metrics:  Arc<Metrics>,        // 8 atomic counters
    api_token: Option<String>,     // Bearer auth
}
```

Auth middleware runs on every request. `/health` and `/metrics` are excluded.

Prometheus metrics exported as text at `GET /metrics`:
```
vv_requests_total{status="ok"}   <n>
vv_requests_total{status="err"}  <n>
vv_inserts_ok_total              <n>
vv_gets_ok_total                 <n>
vv_deletes_ok_total              <n>
vv_scans_ok_total                <n>
# per-table gauges
vv_table_rows_buffered{table="<t>"}  <n>
vv_table_rows_flushed{table="<t>"}   <n>
vv_table_partitions{table="<t>"}     <n>
```

---

## Query DSL (`vectorvolt-query`)

Three-phase pipeline:

```
text input
  │
  ├─► tokenize()   → Vec<Token>        (keywords + literals)
  ├─► parse()      → Query             (typed AST)
  └─► execute()    → QueryResponse     (dispatches to TableEngine)
```

The `run(engine, text)` function combines all three phases.

`Query` is a plain enum — all variants are statically sized. No heap
allocation in the tokenizer for inputs under 256 bytes.

---

## Replication (`vectorvolt-replication`)

```
Leader (write side)
  ├─► holds Arc<TableEngine>
  ├─► crossbeam broadcast channel  (bounded, capacity = 4096 frames)
  └─► write(): insert into engine + broadcast ReplicationFrame

Follower (read side)
  ├─► Receiver<ReplicationFrame>
  ├─► apply_pending(): drain channel, apply mutations to local engine
  ├─► catchup(): block until follower reaches a minimum sequence number
  └─► frames_applied: AtomicU64
```

Disconnected followers are pruned automatically when the broadcast
channel's lag exceeds capacity (the sender drops lagging receivers).

---

## Durability Modes

| Mode | WAL | fsync | Use case |
|---|---|---|---|
| `WriteOnly` | skip | no | benchmarks, tests |
| `Buffered` | write | no | low-latency production (data survives process crash, not power loss) |
| `Sync` | write | yes | full durability (every flush fsyncs) |

---

## Memory Layout

- **WriteBuffer**: `CuckooMap<u64, Vec<u64>>` — open-addressed hash map, no pointer chasing per probe
- **Partition files**: one file per flush, columns stored contiguously (`Vec<u64>` serialised as raw bytes)
- **Zone maps**: `(min: u64, max: u64)` per column per partition — in memory, 16 bytes × cols × partitions
- **Secondary indexes**: `BTreeMap<u64, BTreeSet<u64>>` — value → set of PKs
- **TTL index**: `BTreeMap<u64, Vec<u64>>` — expiry_epoch → list of PKs

---

## Concurrency

- `cas_lock: Mutex<()>` — serialises all conditional-write operations
  (`insert_if_absent`, `compare_and_swap`, `update_field`, range-delete)
- `partitions: RwLock<Vec<Partition>>` — readers hold shared lock; flushes
  and compactions hold exclusive lock for the write phase only
- No async runtime inside the engine — all engine methods are synchronous
- The HTTP gateway runs on Tokio; each request spawns `block_in_place` for
  the engine call if needed
