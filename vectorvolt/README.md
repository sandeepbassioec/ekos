# VectorVolt

Ultra-low-latency OLTP data engine with kernel-bypass networking, sub-microsecond
reads, and zero garbage collection.

```
INSERT 1 1 30 99   →   < 1 µs   (buffered, no flush)
GET    1           →   < 500 ns (CuckooMap probe)
SCAN col=1 lo=0 hi=99 → zone-map prune + SIMD filter
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  vvctl CLI  ┃  REST HTTP (axum)  ┃  TCP wire (122 opcodes)  │
├─────────────────────────────────────────────────────────────┤
│              vectorvolt-query  DSL  (lexer/parser/executor)  │
├─────────────────────────────────────────────────────────────┤
│                  TableEngine  /  TableRegistry               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────┐  │
│  │ WriteBuffer│  │  WAL    │  │ Partitions│  │ Indexes   │  │
│  │ CuckooMap │  │ (fsync) │  │ (mmap)   │  │ (BTreeMap)│  │
│  └──────────┘  └──────────┘  └──────────┘  └───────────┘  │
├─────────────────────────────────────────────────────────────┤
│  vectorvolt-replication  (Leader / Follower, crossbeam)     │
└─────────────────────────────────────────────────────────────┘
```

| Layer | Crate | Responsibility |
|---|---|---|
| Storage | `vectorvolt-engine` | Column-store partitions, WAL, compaction, aggregates |
| Network | `vectorvolt-net` | 122-opcode wire protocol, TLS, HMAC-SHA3 auth |
| HTTP | `vectorvolt-http` | REST gateway (axum), Prometheus metrics |
| Query | `vectorvolt-query` | Text DSL — lexer, parser, executor |
| Replication | `vectorvolt-replication` | Leader/follower streaming |
| CLI | `vvctl` | `serve`, `connect`, `keygen`, `bench` |

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

### Connect (interactive REPL)

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

### REST API (curl)

```bash
BASE=http://127.0.0.1:7879

# DDL
curl -X POST $BASE/tables/orders -H 'Content-Type: application/json' \
     -d '{"columns":["pk","qty","price"]}'

# Insert
curl -X POST $BASE/tables/orders/rows/1 \
     -H 'Content-Type: application/json' \
     -d '{"values":[1,10,999]}'

# Get
curl $BASE/tables/orders/rows/1

# Aggregates
curl "$BASE/tables/orders/count?col=1&lo=5&hi=20"
curl "$BASE/tables/orders/sum?col=2&lo=0&hi=9999"
curl "$BASE/tables/orders/avg?col=2&lo=0&hi=9999"

# Sorted scan
curl "$BASE/tables/orders/scan/sorted?col=0&lo=1&hi=100&sort=2&asc=1&limit=10"

# Cursor-based pagination
curl "$BASE/tables/orders/scan/page?col=0&lo=0&hi=999&after=0&limit=25"

# Upsert / update field / delete range
curl -X PUT $BASE/tables/orders/rows/1 \
     -H 'Content-Type: application/json' -d '{"values":[1,5,499]}'
curl -X PATCH $BASE/tables/orders/rows/1/field \
     -H 'Content-Type: application/json' -d '{"col":1,"val":7}'
curl -X DELETE "$BASE/tables/orders/rows/range?col=1&lo=0&hi=3"

# Observability
curl $BASE/health
curl $BASE/metrics     # Prometheus text format
```

---

## REST API Reference

| Method | Path | Description |
|---|---|---|
| `GET` | `/tables` | List all tables |
| `POST` | `/tables/{t}` | Create table — body: `{"columns":[...]}` |
| `DELETE` | `/tables/{t}` | Drop table |
| `GET` | `/tables/{t}/schema` | Describe table columns |
| `POST` | `/tables/{t}/rows/{pk}` | Insert row |
| `GET` | `/tables/{t}/rows/{pk}` | Get row |
| `PUT` | `/tables/{t}/rows/{pk}` | Upsert row |
| `DELETE` | `/tables/{t}/rows/{pk}` | Delete row |
| `PATCH` | `/tables/{t}/rows/{pk}/field` | Update one column |
| `DELETE` | `/tables/{t}/rows/range` | Delete range (`?col=&lo=&hi=`) |
| `GET` | `/tables/{t}/scan` | Range scan |
| `GET` | `/tables/{t}/secondary` | Secondary-index lookup |
| `GET` | `/tables/{t}/count` | Count matching rows |
| `GET` | `/tables/{t}/sum` | Sum of column in range |
| `GET` | `/tables/{t}/min` | Min of column in range |
| `GET` | `/tables/{t}/max` | Max of column in range |
| `GET` | `/tables/{t}/avg` | Avg (rounded) of column in range |
| `GET` | `/tables/{t}/scan/sorted` | Sorted scan with limit |
| `GET` | `/tables/{t}/scan/page` | Cursor-based pagination |
| `POST` | `/tables/{t}/query` | Text DSL query |
| `GET` | `/health` | Health check (always public) |
| `GET` | `/metrics` | Prometheus metrics (always public) |

---

## DSL Reference

```
GET <pk>
INSERT <pk> <v1> [v2 ...]
UPSERT <pk> <v1> [v2 ...]
DELETE <pk>
UPDATE FIELD <pk> col=<n> val=<v>
SCAN col=<n> lo=<lo> hi=<hi>
SCAN SORTED col=<n> lo=<lo> hi=<hi> sort=<n> asc=<0|1> limit=<n>
SCAN PAGE col=<n> lo=<lo> hi=<hi> after=<pk> limit=<n>
SECONDARY col=<n> val=<v>
COUNT col=<n> lo=<lo> hi=<hi>
SUM col=<n> lo=<lo> hi=<hi>
MIN col=<n> lo=<lo> hi=<hi>
MAX col=<n> lo=<lo> hi=<hi>
AVG col=<n> lo=<lo> hi=<hi>
DELETE RANGE col=<n> lo=<lo> hi=<hi>
BEGIN TXN
INSERT TXN <txn_id> <pk> <v1> [v2 ...]
DELETE TXN <txn_id> <pk>
GET TXN <txn_id> <pk>
COMMIT TXN <txn_id>
ROLLBACK TXN <txn_id>
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

row = c.get(1)                          # Row(pk=1, values=[1, 10, 999])
rows = c.scan(1, 5, 20)                 # Range scan on col 1
count = c.count(1, 5, 20)              # 3
total = c.sum_range(2, 0, 9999)        # sum of col 2
page = c.scan_page(0, 0, 100, limit=10)  # PageResult(rows, next_cursor)
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
var row = client.Get(1);              // ulong[]?
var count = client.CountRange(1, 5, 20);
```

**HTTP client (Layer 44 analytical endpoints):**
```csharp
using var http = new VectorVoltHttpClient("http://127.0.0.1:7879");
await http.UpsertAsync("orders", 1, new ulong[] { 1, 10, 999 });
var page = await http.ScanPageAsync("orders", col: 0, lo: 0, hi: 100, limit: 25);
```

---

## Benchmarks

```bash
# Engine hot path
cargo bench -p vectorvolt-engine

# Query DSL
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
| `dsl/read/get` | ~310 ns |
| `dsl/write/insert` | ~620 ns |

---

## Engine Feature Matrix

| Feature | Method | Layer |
|---|---|---|
| Insert / get / delete | `insert`, `get`, `delete` | 1 |
| WAL crash recovery | `flush` | 2 |
| Range scan | `scan` | 1 |
| MVCC transactions | `begin_txn`, `commit_txn`, `rollback_txn` | 18 |
| Aggregates | `count_range`, `sum_range`, `min_range`, `max_range`, `avg_range` | 21 |
| Histogram / percentile | `histogram_range`, `percentile_range` | 23 |
| Column projection | `scan_project` | 24 |
| Conditional writes | `insert_if_absent`, `compare_and_swap`, `upsert` | 25 |
| Batch ops | `batch_get`, `batch_insert`, `batch_delete` | 26 |
| Sorted scan | `scan_sorted` | 27 |
| Group-by | `group_by_count`, `group_by_sum`, etc. | 28 |
| TTL / expiry | `insert_with_ttl`, `purge_expired` | 29 |
| Range delete | `delete_range` | 30 |
| Secondary indexes | `build_secondary_index`, `secondary_lookup` | 31 |
| Field-level updates | `update_field`, `increment_field` | 32 |
| Auto-maintained indexes | automatic on insert/delete | 33 |
| Compound scan (AND) | `scan_where` | 34 |
| Window functions | `running_sum`, `rank_by`, `dense_rank_by` | 35 |
| Bulk field updates | `update_field_where` | 36 |
| Cursor pagination | `scan_page` | 37 |
| Distinct / value counts | `distinct_values`, `value_counts` | 38 |
| Auto-compaction | `EngineOptions::with_auto_compact` | 40 |
| Persisted indexes | rebuilt automatically on open | 41 |

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
