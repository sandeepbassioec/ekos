# VectorVolt

Ultra-low-latency OLTP data engine with kernel-bypass networking, sub-microsecond
reads, and zero garbage collection.

```
INSERT 1 1 30 99           →  < 1 µs    (buffered, no flush)
GET    1                   →  < 500 ns  (CuckooMap probe)
SCAN col=1 lo=0 hi=99      →  zone-map prune + SIMD filter
HISTOGRAM col=1 lo=0 hi=99 buckets=10  →  O(matching rows)
CAS 1 expected 1 30 99 new 1 31 100    →  serialised under cas_lock
```

---

## Architecture

```
┌────────────────────────────────────────────────────────────────────────┐
│  vvctl CLI (serve / connect / http / bench / hal / keygen / store / wal)│
├────────────────────────────────────────────────────────────────────────┤
│         REST HTTP (axum 0.8)          │  TCP wire (122 opcodes)        │
│  analytics · window · group · TTL    │  auth · CRUD · aggregates      │
│  scan_where · scan_project · CAS     │  batch · CAS · MVCC txns       │
├────────────────────────────────────────────────────────────────────────┤
│               vectorvolt-query  DSL  (lexer / parser / executor)       │
├────────────────────────────────────────────────────────────────────────┤
│                     TableEngine  /  TableRegistry                      │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐  ┌──────────┐  ┌───────┐  │
│  │WriteBuffer│  │   WAL   │  │ Partitions │  │ Indexes  │  │  TTL  │  │
│  │CuckooMap  │  │(fsync)  │  │  (mmap)    │  │(BTreeMap)│  │ index │  │
│  └──────────┘  └──────────┘  └───────────┘  └──────────┘  └───────┘  │
├────────────────────────────────────────────────────────────────────────┤
│         vectorvolt-replication  (Leader / Follower, crossbeam)        │
└────────────────────────────────────────────────────────────────────────┘
```

| Layer | Crate | Responsibility |
|---|---|---|
| Storage | `vectorvolt-engine` | Column-store partitions, WAL, compaction, all analytics |
| Network | `vectorvolt-net` | 122-opcode wire protocol, TLS, HMAC-SHA3 auth |
| HTTP | `vectorvolt-http` | REST gateway (axum 0.8), Prometheus metrics |
| Query | `vectorvolt-query` | Text DSL — lexer, parser, executor |
| Replication | `vectorvolt-replication` | Leader/follower streaming |
| CLI | `vvctl` | `serve`, `connect`, `http`, `keygen`, `bench`, `hal` |

---

## Quick Start

### Build

```bash
cd vectorvolt
cargo build --release
```

The `vvctl` binary is at `target/release/vvctl`.

### Serve

```bash
# Single-table server (TCP + HTTP)
vvctl serve ./data --columns pk age score \
    --http-addr 127.0.0.1:7879

# Multi-table server
vvctl serve ./data --multi \
    --addr 127.0.0.1:5995 \
    --http-addr 127.0.0.1:7879

# With auth
vvctl keygen my-key            # prints a hex secret
vvctl serve ./data --multi \
    --secret <hex>             \
    --http-addr 127.0.0.1:7879 \
    --http-token mytoken
```

### Connect (interactive REPL — TCP)

```bash
vvctl connect 127.0.0.1:5995
> INSERT 1 1 30 99
> GET 1
> SCAN col=1 lo=0 hi=100
> COUNT col=1 lo=0 hi=50
> SCAN SORTED col=1 lo=0 hi=100 sort=2 asc=1 limit=10
> SCAN PAGE col=0 lo=0 hi=1000 after=0 limit=25
> quit
```

### HTTP CLI (`vvctl http`)

All HTTP-gateway analytics, window functions, TTL, CAS, and batch ops are
accessible directly from the CLI without writing curl commands:

```bash
BASE="--addr http://127.0.0.1:7879 --table orders"

# Analytics
vvctl http $BASE histogram --col 1 --lo 0 --hi 999 --buckets 10
vvctl http $BASE percentile --col 1 --lo 0 --hi 999 --p 95

# Group-by
vvctl http $BASE group-count --key-col 1
vvctl http $BASE group-sum   --key-col 1 --val-col 2

# Window functions
vvctl http $BASE window-running-sum   --col 2
vvctl http $BASE window-rank          --col 2

# Distinct / value counts
vvctl http $BASE distinct     --col 1
vvctl http $BASE value-counts --col 1

# Conditional writes
vvctl http $BASE if-absent --pk 42 --values "42 5 999"
vvctl http $BASE cas --pk 1 --expected "1 5 999" --new-values "1 6 1099"

# Batch insert
vvctl http $BASE batch-insert --rows "1 1 10 99" "2 2 20 199"

# TTL
vvctl http $BASE insert-ttl --pk 10 --values "10 1 100" --expire-at 9999999999
vvctl http $BASE get-ttl  --pk 10 --now 0
vvctl http $BASE purge-expired --now 5000

# Raw DSL pass-through
vvctl http $BASE query "HISTOGRAM col=1 lo=0 hi=999 buckets=10"
```

### REST API (curl)

```bash
BASE=http://127.0.0.1:7879

# DDL
curl -X POST $BASE/tables/orders -H 'Content-Type: application/json' \
     -d '{"columns":["pk","qty","price"]}'

# Insert / get / upsert / update field
curl -X POST  $BASE/tables/orders/rows/1 -H 'Content-Type: application/json' -d '{"values":[1,10,999]}'
curl          $BASE/tables/orders/rows/1
curl -X PUT   $BASE/tables/orders/rows/1 -H 'Content-Type: application/json' -d '{"values":[1,5,499]}'
curl -X PATCH $BASE/tables/orders/rows/1/field -H 'Content-Type: application/json' -d '{"col":1,"val":7}'

# Aggregates
curl "$BASE/tables/orders/count?col=1&lo=5&hi=20"
curl "$BASE/tables/orders/sum?col=2&lo=0&hi=9999"
curl "$BASE/tables/orders/avg?col=2&lo=0&hi=9999"

# Analytics (Layers 48-53)
curl "$BASE/tables/orders/histogram?col=2&lo=0&hi=9999&buckets=10"
curl "$BASE/tables/orders/percentile?col=2&lo=0&hi=9999&p=95"
curl "$BASE/tables/orders/group/count?filter_col=1&lo=0&hi=99&group_col=1"
curl "$BASE/tables/orders/group/sum?filter_col=1&lo=0&hi=99&group_col=1&agg_col=2"
curl "$BASE/tables/orders/window/running_sum?filter_col=1&lo=0&hi=99&agg_col=2"
curl "$BASE/tables/orders/window/rank?filter_col=1&lo=0&hi=99&sort_col=2&asc=1"
curl "$BASE/tables/orders/distinct?col=1&lo=0&hi=99"
curl "$BASE/tables/orders/value_counts?col=1&lo=0&hi=99"

# Bulk field updates (Layer 54)
curl "$BASE/tables/orders/update_where?filter_col=1&lo=5&hi=10&target_col=2&val=0"
curl "$BASE/tables/orders/increment_where?filter_col=1&lo=5&hi=10&target_col=2&delta=1"
curl "$BASE/tables/orders/decrement_where?filter_col=1&lo=5&hi=10&target_col=2&delta=1"

# Scan where + projection (Layers 56-57)
curl "$BASE/tables/orders/scan_where?col0=1&lo0=5&hi0=10"
curl "$BASE/tables/orders/scan/project?filter_col=1&lo=0&hi=99&proj=0,2"

# Conditional writes (Layer 58)
curl -X POST $BASE/tables/orders/rows/42/if_absent \
     -H 'Content-Type: application/json' -d '{"values":[42,3,299]}'
curl -X POST $BASE/tables/orders/rows/1/cas \
     -H 'Content-Type: application/json' \
     -d '{"expected":[1,5,499],"new_values":[1,6,599]}'

# Batch insert (Layer 59)
curl -X POST $BASE/tables/orders/batch/insert \
     -H 'Content-Type: application/json' \
     -d '{"rows":[{"pk":2,"values":[2,3,299]},{"pk":3,"values":[3,1,99]}]}'

# TTL operations (Layer 60)
curl -X POST $BASE/tables/orders/rows/99/ttl \
     -H 'Content-Type: application/json' -d '{"values":[99,1,49],"expire_at":9999999999}'
curl "$BASE/tables/orders/get_with_ttl/99?now=0"
curl "$BASE/tables/orders/scan_with_ttl?now=0"
curl -X POST "$BASE/tables/orders/purge_expired?now=5000"

# Sorted scan / cursor pagination
curl "$BASE/tables/orders/scan/sorted?col=0&lo=1&hi=100&sort=2&asc=1&limit=10"
curl "$BASE/tables/orders/scan/page?col=0&lo=0&hi=999&after=0&limit=25"

# Observability
curl $BASE/health
curl $BASE/metrics     # Prometheus text format
```

---

## REST API Reference

### DDL

| Method | Path | Description |
|---|---|---|
| `GET` | `/tables` | List all tables |
| `POST` | `/tables/{t}` | Create table — body: `{"columns":[...]}` |
| `DELETE` | `/tables/{t}` | Drop table |
| `GET` | `/tables/{t}/schema` | Describe table columns |

### Core CRUD

| Method | Path | Description |
|---|---|---|
| `POST` | `/tables/{t}/rows/{pk}` | Insert row |
| `GET` | `/tables/{t}/rows/{pk}` | Get row |
| `PUT` | `/tables/{t}/rows/{pk}` | Upsert row |
| `DELETE` | `/tables/{t}/rows/{pk}` | Delete row |
| `PATCH` | `/tables/{t}/rows/{pk}/field` | Update one column |
| `DELETE` | `/tables/{t}/rows/range` | Delete range (`?col=&lo=&hi=`) |
| `GET` | `/tables/{t}/scan` | Range scan (`?col=&lo=&hi=`) |
| `GET` | `/tables/{t}/secondary` | Secondary-index lookup (`?col=&val=`) |

### Aggregates

| Method | Path | Description |
|---|---|---|
| `GET` | `/tables/{t}/count` | Count matching rows |
| `GET` | `/tables/{t}/sum` | Sum of column in range |
| `GET` | `/tables/{t}/min` | Min of column in range |
| `GET` | `/tables/{t}/max` | Max of column in range |
| `GET` | `/tables/{t}/avg` | Avg (rounded) of column in range |
| `GET` | `/tables/{t}/scan/sorted` | Sorted scan with limit |
| `GET` | `/tables/{t}/scan/page` | Cursor-based pagination |

### Analytics (Layers 48-53)

| Method | Path | Description |
|---|---|---|
| `GET` | `/tables/{t}/histogram` | Bucket histogram (`?col=&lo=&hi=&buckets=`) |
| `GET` | `/tables/{t}/percentile` | Nearest-rank percentile (`?col=&lo=&hi=&p=`) |
| `GET` | `/tables/{t}/group/count` | Group-by count (`?filter_col=&lo=&hi=&group_col=`) |
| `GET` | `/tables/{t}/group/sum` | Group-by sum (`+&agg_col=`) |
| `GET` | `/tables/{t}/group/min` | Group-by min (`+&agg_col=`) |
| `GET` | `/tables/{t}/group/max` | Group-by max (`+&agg_col=`) |
| `GET` | `/tables/{t}/window/running_sum` | Cumulative sum (`?filter_col=&lo=&hi=&agg_col=`) |
| `GET` | `/tables/{t}/window/running_count` | Row number (`?filter_col=&lo=&hi=`) |
| `GET` | `/tables/{t}/window/rank` | Standard rank (`?filter_col=&lo=&hi=&sort_col=&asc=`) |
| `GET` | `/tables/{t}/window/dense_rank` | Dense rank (same params) |
| `GET` | `/tables/{t}/distinct` | Distinct values in range (`?col=&lo=&hi=`) |
| `GET` | `/tables/{t}/value_counts` | Value frequency map (same params) |

### Bulk Mutations (Layer 54)

| Method | Path | Description |
|---|---|---|
| `GET` | `/tables/{t}/update_where` | Set field for matching rows (`?filter_col=&lo=&hi=&target_col=&val=`) |
| `GET` | `/tables/{t}/increment_where` | Add delta for matching rows (`+&delta=`) |
| `GET` | `/tables/{t}/decrement_where` | Subtract delta for matching rows (`+&delta=`) |

### Scan + Project (Layers 56-57)

| Method | Path | Description |
|---|---|---|
| `GET` | `/tables/{t}/scan_where` | Multi-predicate AND scan (`?col0=&lo0=&hi0=[&col1=...]`) |
| `GET` | `/tables/{t}/scan/project` | Column projection (`?filter_col=&lo=&hi=&proj=csv`) |

### Conditional Writes + Batch (Layers 58-59)

| Method | Path | Description |
|---|---|---|
| `POST` | `/tables/{t}/rows/{pk}/if_absent` | Insert only if PK absent — body: `{"values":[...]}` |
| `POST` | `/tables/{t}/rows/{pk}/cas` | Compare-and-swap — body: `{"expected":[...],"new_values":[...]}` |
| `POST` | `/tables/{t}/batch/insert` | Insert multiple rows — body: `{"rows":[{"pk":n,"values":[...]},...]}` |
| `POST` | `/tables/{t}/batch/get` | Get multiple rows by PK — body: `{"pks":[...]}` |
| `POST` | `/tables/{t}/batch/delete` | Delete multiple rows by PK |

### TTL / Expiry (Layer 60)

| Method | Path | Description |
|---|---|---|
| `POST` | `/tables/{t}/rows/{pk}/ttl` | Insert with expiry — body: `{"values":[...],"expire_at":<epoch>}` |
| `GET` | `/tables/{t}/get_with_ttl/{pk}` | Get row if not expired (`?now=<epoch>`) |
| `GET` | `/tables/{t}/scan_with_ttl` | Scan non-expired rows (`?now=<epoch>`) |
| `POST` | `/tables/{t}/purge_expired` | Delete expired rows (`?now=<epoch>`) |

### Query DSL + Observability

| Method | Path | Description |
|---|---|---|
| `POST` | `/tables/{t}/query` | Text DSL query — body: plain-text statement |
| `GET` | `/health` | Health check (always public) |
| `GET` | `/metrics` | Prometheus metrics (always public) |

---

## DSL Reference

### Core CRUD

```
GET <pk>
INSERT <pk> <v1> [v2 ...]
UPSERT <pk> <v1> [v2 ...]
DELETE <pk>
UPDATE FIELD <pk> col=<n> val=<v>
```

### Scans + Aggregates

```
SCAN col=<n> lo=<lo> hi=<hi>
SCAN SORTED col=<n> lo=<lo> hi=<hi> sort=<n> asc=<0|1> limit=<n>
SCAN PAGE col=<n> lo=<lo> hi=<hi> after=<pk> limit=<n>
SECONDARY col=<n> val=<v>
COUNT col=<n> lo=<lo> hi=<hi>
SUM   col=<n> lo=<lo> hi=<hi>
MIN   col=<n> lo=<lo> hi=<hi>
MAX   col=<n> lo=<lo> hi=<hi>
AVG   col=<n> lo=<lo> hi=<hi>
DELETE RANGE col=<n> lo=<lo> hi=<hi>
```

### Analytics (Layers 50-53)

```
HISTOGRAM  col=<n> lo=<lo> hi=<hi> buckets=<n>
PERCENTILE col=<n> lo=<lo> hi=<hi> p=<0-100>

GROUP COUNT    filter_col=<n> lo=<lo> hi=<hi> group_col=<n>
GROUP SUM      filter_col=<n> lo=<lo> hi=<hi> group_col=<n> agg_col=<n>
GROUP MIN      filter_col=<n> lo=<lo> hi=<hi> group_col=<n> agg_col=<n>
GROUP MAX      filter_col=<n> lo=<lo> hi=<hi> group_col=<n> agg_col=<n>

WINDOW RUNNING_SUM   filter_col=<n> lo=<lo> hi=<hi> agg_col=<n>
WINDOW RUNNING_COUNT filter_col=<n> lo=<lo> hi=<hi>
WINDOW RANK          filter_col=<n> lo=<lo> hi=<hi> sort_col=<n> asc=<0|1>
WINDOW DENSE_RANK    filter_col=<n> lo=<lo> hi=<hi> sort_col=<n> asc=<0|1>

DISTINCT    col=<n> lo=<lo> hi=<hi>
VALUE_COUNTS col=<n> lo=<lo> hi=<hi>
```

### Bulk Mutations (Layer 54)

```
UPDATE    WHERE filter_col=<n> lo=<lo> hi=<hi> target_col=<n> val=<v>
INCREMENT WHERE filter_col=<n> lo=<lo> hi=<hi> target_col=<n> delta=<d>
DECREMENT WHERE filter_col=<n> lo=<lo> hi=<hi> target_col=<n> delta=<d>
```

### Scan + Project (Layers 56-57)

```
SCAN WHERE col=<n> lo=<lo> hi=<hi> [AND col=<n> lo=<lo> hi=<hi> ...]
SCAN PROJECT col=<n> lo=<lo> hi=<hi> cols <c1> [c2 ...]
```

### Conditional Writes + Batch (Layers 58-59)

```
INSERT IF_ABSENT <pk> <v1> [v2 ...]
CAS <pk> expected <e1> [e2 ...] new <n1> [n2 ...]
BATCH INSERT row <pk> <v1> [v2 ...] [row <pk> <v1> ...]
```

### TTL / Expiry (Layer 60)

```
INSERT TTL <pk> <v1> [v2 ...] expire=<epoch>
GET TTL <pk> now=<epoch>
SCAN TTL now=<epoch>
PURGE EXPIRED now=<epoch>
```

### MVCC Transactions

```
BEGIN TXN
INSERT TXN <txn_id> <pk> <v1> [v2 ...]
DELETE TXN <txn_id> <pk>
GET TXN <txn_id> <pk>
COMMIT TXN <txn_id>
ROLLBACK TXN <txn_id>
```

### Introspection

```
DESCRIBE
STATS
```

---

## Python SDK

```python
from vectorvolt import VectorVoltClient

c = VectorVoltClient("http://127.0.0.1:7879")
c.create_table(["pk", "qty", "price"], table="orders")
c.insert(1, [1, 10, 999], table="orders")
c.use_table("orders")

# Core CRUD
row   = c.get(1)                          # Row(pk=1, values=[1, 10, 999])
rows  = c.scan(1, 5, 20)                  # range scan on col 1
count = c.count(1, 5, 20)
page  = c.scan_page(0, 0, 100, limit=10)  # PageResult(rows, next_cursor)

# Analytics (v0.2.0)
buckets = c.histogram(col=2, lo=0, hi=9999, buckets=10)
p95     = c.percentile(col=2, lo=0, hi=9999, p=95)
groups  = c.group_count(key_col=1)         # list[GroupEntry]
ranks   = c.window_rank(col=2)             # list[WindowRow]
vals    = c.distinct_values(col=1, lo=0, hi=99)
freq    = c.value_counts(col=1, lo=0, hi=99)

# Bulk mutations
c.update_field_where(col=1, lo=5, hi=10, target_col=2, new_val=0)
c.increment_field_where(col=1, lo=5, hi=10, target_col=2, delta=1)

# Scan + project
rows = c.scan_where(predicates=[(1, 5, 10)], limit=50)
proj = c.scan_project(filter_col=1, lo=0, hi=99, proj=[0, 2])

# Conditional writes + batch
inserted = c.insert_if_absent(pk=42, values=[42, 3, 299])
swapped  = c.compare_and_swap(pk=1, expected=[1, 5, 499], new_values=[1, 6, 599])
n = c.batch_insert(rows=[(2, [2, 3, 299]), (3, [3, 1, 99])])

# TTL
c.insert_with_ttl(pk=99, values=[99, 1, 49], expire_at=9_999_999_999)
row  = c.get_with_ttl(pk=99, now=0)
live = c.scan_with_ttl(now=0)
gone = c.purge_expired(now=5000)

# Raw DSL
result = c.query("HISTOGRAM col=2 lo=0 hi=9999 buckets=10")
```

Install (dev):
```bash
pip install -e sdk/python
```

---

## C# SDK

**TCP client (full 122-opcode parity):**
```csharp
using var client = VectorVoltClient.Connect("127.0.0.1", 5995, "my-key", secret);
client.UseTable("orders");
client.Insert(1, new ulong[] { 1, 10, 999 });
var row   = client.Get(1);
var count = client.CountRange(1, 5, 20);
```

**HTTP client (Layers 44-60 REST gateway):**
```csharp
using var http = new VectorVoltHttpClient("http://127.0.0.1:7879");
await http.UpsertAsync("orders", 1, new ulong[] { 1, 10, 999 });

// Analytics
var hist    = await http.HistogramAsync("orders", col: 2, lo: 0, hi: 9999, buckets: 10);
var p95     = await http.PercentileAsync("orders", col: 2, lo: 0, hi: 9999, p: 95);
var groups  = await http.GroupCountAsync("orders", keyCol: 1);
var ranks   = await http.WindowRankAsync("orders", col: 2);
var vals    = await http.DistinctValuesAsync("orders", col: 1, lo: 0, hi: 99);

// Scan + project
var rows = await http.ScanWhereAsync("orders", new[] { (1u, 5ul, 10ul) });
var proj = await http.ScanProjectAsync("orders", filterCol: 1, lo: 0, hi: 99,
                                        projCols: new uint[] { 0, 2 });

// Conditional writes + batch
bool ok = await http.InsertIfAbsentAsync("orders", 42, new ulong[] { 42, 3, 299 });
bool sw = await http.CompareAndSwapAsync("orders", 1,
                expected: new ulong[] { 1, 5, 499 },
                newValues: new ulong[] { 1, 6, 599 });
ulong n = await http.BatchInsertAsync("orders",
                new[] { (2ul, (IReadOnlyList<ulong>)new ulong[] { 2, 3, 299 }) });

// TTL
await http.InsertWithTtlAsync("orders", 99, new ulong[] { 99, 1, 49 }, expireAt: 9_999_999_999);
var row  = await http.GetWithTtlAsync("orders", 99, now: 0);
var live = await http.ScanWithTtlAsync("orders", now: 0);
ulong gc = await http.PurgeExpiredAsync("orders", now: 5000);
```

---

## Benchmarks

```bash
# Engine hot path (all groups including Layer 50-60 new ops)
cargo bench -p vectorvolt-engine

# Query DSL (all groups including Layer 50-60 statements)
cargo bench -p vectorvolt-query

# Save a baseline and compare
cargo bench -p vectorvolt-engine -- --save-baseline main
# ... make changes ...
cargo bench -p vectorvolt-engine -- --baseline main
```

Representative numbers on a 12-core dev machine (AMD Ryzen 9, NVMe):

| Operation | Latency (p50) |
|---|---|
| `insert/buffered/1` | ~180 ns |
| `get/buffered` | ~85 ns |
| `get/flushed` | ~220 ns |
| `scan/flushed_1pct` (10k rows, 1% sel.) | ~28 µs |
| `aggregate/count_flushed` | ~18 µs |
| `histogram/10_buckets` (10k rows) | ~35 µs |
| `group_by/count` (10k rows, 100 groups) | ~120 µs |
| `window/running_sum` (1k matching rows) | ~18 µs |
| `cond_write/cas_mismatch` | ~95 ns |
| `dsl/read/get` | ~310 ns |
| `dsl/analytics/histogram` | ~40 µs |

---

## Engine Feature Matrix

| Feature | Method | Notes |
|---|---|---|
| Insert / get / delete | `insert`, `get`, `delete` | WAL-durable |
| WAL crash recovery | auto on `open()` | replay on reopen |
| Range scan | `scan` | zone-map prune + SIMD filter |
| MVCC transactions | `begin_txn` / `commit_txn` / `rollback_txn` | read-your-writes isolation |
| Aggregates | `count_range`, `sum_range`, `min_range`, `max_range`, `avg_range` | column-store pushdown |
| Histogram | `histogram_range` | configurable bucket count |
| Percentile | `percentile_range` | nearest-rank algorithm |
| Column projection | `scan_project` | decode only requested columns |
| Conditional writes | `insert_if_absent`, `compare_and_swap`, `upsert` | serialised under `cas_lock` |
| Batch ops | `batch_get`, `batch_insert`, `batch_delete` | single-call multi-row |
| Sorted scan | `scan_sorted` | cross-partition sort + limit |
| Group-by | `group_by_count`, `group_by_sum`, `group_by_min`, `group_by_max` | BTreeMap accumulation |
| Window functions | `running_sum`, `running_count`, `rank_by`, `dense_rank_by` | order by pk |
| TTL / expiry | `insert_with_ttl`, `get_with_ttl`, `scan_with_ttl`, `purge_expired` | opaque u64 epoch |
| Range delete | `delete_range` | zone-prune + lock-inversion-safe |
| Secondary indexes | `build_secondary_index`, `secondary_lookup` | auto-maintained on write |
| Field-level updates | `update_field`, `increment_field`, `decrement_field` | wrapping/saturating |
| Bulk predicate updates | `update_field_where`, `increment_field_where`, `decrement_field_where` | multi-row under lock |
| Compound scan (AND) | `scan_where` | up to N ANDed predicates |
| Distinct / value counts | `distinct_values`, `value_counts` | sorted output |
| Cursor pagination | `scan_page` | exclusive cursor over pk |
| Auto-compaction | `EngineOptions::with_auto_compact` | partition count trigger |
| Persisted indexes | rebuilt on open | binary catalog file |

---

## Building for Release

```bash
cargo build --release --workspace
```

The `vvctl` binary is fully statically linked on Linux (musl target):
```bash
cargo build --release --target x86_64-unknown-linux-musl
```

---

## Running Tests

```bash
cargo test --workspace
```

Python SDK tests (requires `vvctl` release build):
```bash
pip install pytest
cargo build --release
pytest sdk/python/tests/
```

C# SDK tests (requires `vvctl` release build):
```bash
dotnet test sdk/csharp/VectorVolt.Client.Tests/
```

---

## License

Apache-2.0 — see [LICENSE](LICENSE).
