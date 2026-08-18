# Changelog

All notable changes to VectorVolt are documented here.

---

## [Unreleased]

### Layer 66 — Materialized Views over CDC
- `views` module in `vectorvolt-engine`: `ViewKind` (Count / Sum / Min / Max),
  `ViewDefinition`, `ViewEntry`, `ViewRegistry`
- Each view runs a background tokio task subscribed to the source table's CDC
  broadcast channel; events are applied to an in-memory group-by aggregate
- Row mirror (`HashMap<pk, Vec<u64>>`) enables correct delete/upsert undo;
  multiset (`BTreeMap<agg_val, count>`) for correct min/max recomputation on delete
- `AppState` gains `Arc<ViewRegistry>`; routes wired into `build_router`
- HTTP DDL: `POST /views`, `GET /views`, `GET /views/{name}`, `DELETE /views/{name}`
- `vvctl http create-view / list-views / get-view / drop-view`
- 19 integration tests

### Layer 65 — CDC + SSE Streaming Subscriptions
- `cdc` module in `vectorvolt-engine`: `ChangeEvent` { kind, pk, values } + `ChangeKind` enum
- `TableEngine::subscribe_changes()` → `tokio::sync::broadcast::Receiver<ChangeEvent>`
- `insert()` and `delete()` publish CDC events; zero-cost when no active subscribers
- All mutation paths (upsert, CAS, insert_if_absent, field_update, batch_insert, TTL, purge_expired)
  propagate automatically through the `insert`/`delete` publication points
- HTTP: `GET /tables/{t}/subscribe[?col=&lo=&hi=]` — Server-Sent Events endpoint
- Optional column-range filter: events whose `values[col] ∉ [lo, hi]` are suppressed;
  delete events (no values) are suppressed when a filter is active
- `vvctl http subscribe [--col N --lo N --hi N]` — streams events to stdout
- 15 integration tests

### Layer 64 — Packaging & Docs Refresh
- `README.md`: updated architecture diagram, REST API reference (all 50+ endpoints),
  DSL reference (all Layer 50-60 statements), Python/C# SDK v0.2.0 examples,
  `vvctl http` quick-start, feature matrix refresh
- `CHANGELOG.md`: entries for Layers 47-64
- `docs/ARCHITECTURE.md`: HTTP gateway endpoint group table, DSL statement coverage table

### Layer 63 — Benchmarking Pass (Layers 50-60)
- `engine_bench.rs`: 11 new Criterion groups — `histogram` (2 cases), `percentile` (p50/p90/p99),
  `group_by` (count/sum/min/max), `window` (running_sum/count/rank/dense_rank),
  `distinct` (distinct_values + value_counts), `update_where` (update/increment/decrement),
  `scan_where` (1 + 2 predicates), `scan_project` (1/2 cols), `cond_write` (if_absent ×2 + cas),
  `batch_insert` (10/100/1000 rows), `ttl` (insert/get/scan/purge_none/purge_all)
- `query_bench.rs`: 4 new groups — `dsl/analytics` (13 cases), `dsl/predicate` (6 cases),
  `dsl/cond_batch_ttl` (8 cases), `dsl/parse_layer50` (19 parse-only cases)

### Layer 62 — Python + C# SDK Expansion
- **Python** (`sdk/python/`): `__version__ = "0.2.0"`; 4 new dataclasses
  (`HistogramBucket`, `GroupEntry`, `WindowRow`, `ValueCountEntry`); 25 new methods
  covering histogram, percentile, group-by ×4, window ×4, distinct, value_counts,
  update/increment/decrement_where, batch_get/delete, scan_where, scan_project,
  insert_if_absent, compare_and_swap, batch_insert, TTL ×4, raw query; 29 integration tests
- **C#** (`sdk/csharp/VectorVolt.Client/`): 25 new async methods with matching coverage;
  4 new record types; 29 xUnit integration tests in `HttpClientTests2.cs`

### Layer 61 — vvctl HTTP Integration
- `vvctl http` top-level subcommand with 24 dedicated subcommands + raw `query` pass-through
- Covers all HTTP-gateway-only ops from Layers 44-60: histogram, percentile, group-by ×4,
  window ×4, distinct, value-counts, update/increment/decrement-where, scan-where,
  scan-project, if-absent, CAS, batch-insert, insert-ttl, get-ttl, scan-ttl, purge-expired
- Uses `reqwest::blocking`; `reqwest` added to vvctl regular `[dependencies]`

### Layer 60 — TTL Operations (HTTP + DSL)
- 4 HTTP endpoints: `POST /rows/{pk}/ttl`, `GET /get_with_ttl/{pk}?now=`,
  `GET /scan_with_ttl?now=`, `POST /purge_expired?now=`
- DSL: `INSERT TTL pk v... expire=n`, `GET TTL pk now=n`, `SCAN TTL now=n`,
  `PURGE EXPIRED now=n`; 5 new keywords (`Ttl`, `Expire`, `Purge`, `Expired`, `Now`)
- 13 integration tests

### Layer 59 — Batch Insert (HTTP + DSL)
- 1 HTTP endpoint: `POST /batch/insert` — body `{"rows":[{"pk":n,"values":[...]},...] }`
- DSL: `BATCH INSERT row pk v... [row pk v...]`; 1 new keyword (`Row`)
- 11 integration tests

### Layer 58 — Conditional Writes (HTTP + DSL)
- 2 HTTP endpoints: `POST /rows/{pk}/if_absent`, `POST /rows/{pk}/cas`
- DSL: `INSERT IF_ABSENT pk v...`, `CAS pk expected v... new v...`
- 4 new keywords (`IfAbsent`, `Cas`, `Expected`, `New`); 10 integration tests

### Layer 57 — Column Projection Scan (HTTP + DSL)
- 1 HTTP endpoint: `GET /scan/project?filter_col=&lo=&hi=&proj=<csv>`
- DSL: `SCAN PROJECT col=n lo=lo hi=hi cols c1 [c2...]`
- 2 new keywords (`Project`, `Cols`); 15 integration tests

### Layer 56 — Multi-Predicate Scan (HTTP + DSL)
- 1 HTTP endpoint: `GET /scan_where?col0=&lo0=&hi0=[&col1=...]` (dynamic indexed params)
- DSL: `SCAN WHERE col=n lo=lo hi=hi [AND col=n lo=lo hi=hi ...]`
- 15 integration tests

### Layer 55 — Batch HTTP Endpoints
- 2 HTTP endpoints: `POST /batch/get`, `POST /batch/delete`
- Consistent with engine Layer 26 batch ops
- 11 integration tests

### Layer 54 — Bulk Field Updates (HTTP + DSL)
- 3 HTTP endpoints: `GET /update_where`, `GET /increment_where`, `GET /decrement_where`
- DSL: `UPDATE WHERE`, `INCREMENT WHERE`, `DECREMENT WHERE`
- 15 integration tests

### Layer 53 — Distinct + Value Counts (HTTP + DSL)
- 2 HTTP endpoints: `GET /distinct?col=&lo=&hi=`, `GET /value_counts?col=&lo=&hi=`
- DSL: `DISTINCT col=n lo=lo hi=hi`, `VALUE_COUNTS col=n lo=lo hi=hi`
- 15 integration tests

### Layer 52 — Window Functions (HTTP + DSL)
- 4 HTTP endpoints: `GET /window/running_sum`, `/window/running_count`,
  `/window/rank`, `/window/dense_rank`
- DSL: `WINDOW RUNNING_SUM/RUNNING_COUNT/RANK/DENSE_RANK filter_col=n lo=lo hi=hi [...]`
- 17 integration tests

### Layer 51 — Group-By Aggregates (HTTP + DSL)
- 4 HTTP endpoints: `GET /group/count`, `/group/sum`, `/group/min`, `/group/max`
- DSL: `GROUP COUNT/SUM/MIN/MAX filter_col=n lo=lo hi=hi group_col=n [agg_col=n]`
- 17 integration tests

### Layer 50 — Histogram + Percentile (HTTP + DSL)
- 2 HTTP endpoints: `GET /histogram?col=&lo=&hi=&buckets=`, `GET /percentile?col=&lo=&hi=&p=`
- DSL: `HISTOGRAM col=n lo=lo hi=hi buckets=n`, `PERCENTILE col=n lo=lo hi=hi p=0-100`
- 14 integration tests

### Layer 47 — Packaging & Docs (initial)
- `README.md`: project overview, architecture, quick-start, REST API reference (Layers 1-46),
  DSL reference, SDK examples, benchmark numbers, feature matrix
- `CHANGELOG.md`: full release history through Layer 46
- `docs/ARCHITECTURE.md`: deep-dive on storage, WAL, compaction, and networking

### Layer 46 — Benchmark Suite Expansion
- `engine_bench.rs`: 9 new Criterion cases — `aggregate` (count/sum/min/max
  buffered+flushed), `sorted_scan` (buffered+flushed), `scan_page` (buffered+flushed)
- `crates/vectorvolt-query/benches/query_bench.rs` (new): 16 DSL cases across
  `dsl/read`, `dsl/write`, and `dsl/parse` groups

### Layer 45 — SDK Expansion
- **Python** (`sdk/python/vectorvolt/client.py`): 10 new HTTP methods —
  `upsert`, `update_field`, `delete_range`, `count`, `sum_range`, `min_range`,
  `max_range`, `avg`, `scan_sorted`, `scan_page`; `PageResult` dataclass; 15 tests
- **C#** (`sdk/csharp/VectorVolt.Client/VectorVoltHttpClient.cs`): new
  `VectorVoltHttpClient` class (HTTP-based) with the same 10 methods; `HttpRow`
  and `HttpPageResult` result types; `ServerHarness.StartWithHttp()`; 15 xUnit tests

### Layer 44 — HTTP Gateway Expansion
- 10 new REST endpoints: `GET /count`, `GET /sum`, `GET /min`, `GET /max`,
  `GET /avg`, `GET /scan/sorted`, `GET /scan/page`, `PUT /rows/{pk}` (upsert),
  `PATCH /rows/{pk}/field`, `DELETE /rows/range`
- New handler functions, query-param structs (`RangeQuery`, `ScanSortedQuery`,
  `ScanPageQuery`, `UpdateFieldBody`), and response types (`CountResp`, `AggResp`,
  `PageResp`)
- 15 integration tests

### Layer 43 — DSL Expansion
- 11 new `Query` variants: `Count`, `Sum`, `Min`, `Max`, `Avg`, `ScanSorted`,
  `ScanPage`, `Upsert`, `UpdateField`, `DeleteRange`
- 3 new `QueryResponse` variants: `Count(u64)`, `Aggregate(Option<u64>)`,
  `Page(PageResult)`
- HTTP bridge updated to handle new response types
- 15 integration tests

### Layer 42 — Bearer-Token Auth for HTTP Gateway
- `HttpServerConfig::with_token(t)` + Axum `auth_middleware`
- `/health` and `/metrics` always public; all data/DDL routes require
  `Authorization: Bearer <token>` when configured
- `--http-token` flag on `vvctl serve`
- 15 integration tests

### Layer 41 — Persist Secondary Indexes
- `index_catalog` module: binary catalog file records live column indexes
- Indexes rebuilt via `scan_deduped` on `open()` — excludes tombstoned rows
- `build_secondary_index` / `drop_secondary_index` atomically update catalog
- Fixed `build_secondary_index` to use `scan_deduped` (was including tombstones)
- 15 integration tests

### Layer 40 — Auto-Compaction Trigger
- `EngineOptions::with_auto_compact(threshold)`: compact after any flush
  reaching the partition count threshold
- `EngineStats::auto_compact_threshold`
- 15 integration tests

### Layer 39 — Scan Dedup Correctness
- `scan_deduped()` helper (scan + `get()`-based dedup)
- All read-path methods routed through it: `scan_page`, `scan_sorted`,
  `scan_where`, window functions, group-by, aggregates, `scan_project`,
  distinct, TTL scan
- 15 integration tests

### Layer 38 — Distinct Values + Value Counts
- `distinct_values(col, lo, hi)` — unique values in range
- `value_counts(col, lo, hi)` — frequency map
- 15 integration tests

### Layer 37 — Cursor-Based Pagination
- `scan_page(filter_col, lo, hi, after_pk, limit) -> PageResult`
- Deduplicates by pk across partition files; exclusive cursor
- 15 integration tests

### Layer 36 — Bulk Predicate-Filtered Field Updates
- `update_field_where`, `increment_field_where`, `decrement_field_where`
- 15 integration tests

### Layer 35 — Window Functions
- `running_sum`, `running_count`, `rank_by`, `dense_rank_by`
- `WindowRow { pk, window_val }`
- 18 integration tests

### Layer 34 — Multi-Predicate Compound Scan
- `scan_where(&[(col, lo, hi)])` — AND semantics; early-exit per predicate
- 17 integration tests

### Layer 33 — Auto-Maintained Secondary Indexes
- Insert / delete / upsert / batch / field-update operations all patch live indexes
- Fast-path when no indexes registered
- 15 integration tests

### Layer 32 — Field-Level Atomic Updates
- `update_field`, `increment_field` (wrapping), `decrement_field` (saturating)
- All under `cas_lock`; `Ok(None)` on missing PK
- 22 integration tests

### Layer 31 — Secondary Index
- `build_secondary_index`, `secondary_lookup`, `secondary_range_lookup`,
  `secondary_get`, `drop_secondary_index`, `has_secondary_index`, `entry_count`
- `BTreeMap<value, BTreeSet<pk>>`; on-demand, not auto-maintained at this layer
- 21 integration tests

### Layer 30 — Range Delete
- `delete_range(filter_col, lo, hi) -> usize`
- Zone-prune + 2-col decode; lock-inversion-safe; silent skip on missing PKs
- 14 integration tests

### Layer 29 — TTL / Row Expiry
- `insert_with_ttl`, `get_with_ttl`, `scan_with_ttl`, `purge_expired`
- BTreeMap TTL index; opaque u64 epoch
- 19 integration tests

### Layer 28 — Group-By Aggregates
- `group_by_count`, `group_by_sum`, `group_by_min`, `group_by_max`
- BTreeMap accumulation; sorted output
- 17 integration tests

### Layer 27 — Sorted Scan + Top-N
- `scan_sorted(filter_col, lo, hi, sort_col, ascending, limit)`
- Zone-prune + 2-col decode; single cross-partition sort
- 16 integration tests

### Layer 26 — Batch Operations
- `batch_get`, `batch_insert`, `batch_delete`
- Silent skip on missing PKs
- 15 integration tests

### Layer 25 — Conditional Writes
- `insert_if_absent`, `compare_and_swap`, `upsert -> UpsertResult`
- `cas_lock: Mutex<()>` serialises read-check-write
- 13 integration tests (including 2 concurrent barrier tests)

### Layer 24 — Column Projection for Scan
- `scan_project(filter_col, lo, hi, projection: &[usize])`
- Decodes only col 0 + projected cols (up to 98% decode-work reduction)
- 12 integration tests

### Layer 23 — Histogram + Percentile Queries
- `histogram_range(col, lo, hi, n_buckets) -> Vec<HistogramBucket>`
- `percentile_range(col, lo, hi, p) -> Option<u64>` (nearest-rank)
- 18 integration tests

### Layer 22 — Query DSL → HTTP Bridge
- `POST /tables/{table}/query` — accepts plain-text DSL; returns tagged JSON
- All 14 query types supported including MVCC transactions
- 11 integration tests (29 total in `http_integration`)

### Layer 21 — Aggregate Pushdown / Column Store
- `count_range`, `sum_range`, `min_range`, `max_range`, `avg_range`
- Zone pruning; only target column decoded per partition
- 16 integration tests

### Layer 20 — Distributed Replication
- `vectorvolt-replication` crate: `Leader` + `Follower`
- Crossbeam broadcast; auto-prune disconnected followers
- 12 integration tests

### Layer 19 — Query Language / DSL
- `vectorvolt-query` crate: lexer / parser / executor
- 14 statement types including MVCC transaction variants
- `run(engine, text)` one-shot API
- 13 integration tests

### Layer 18 — MVCC Transactions
- `MvccStore` (atomic TxnId + per-txn mutation HashMap)
- `begin_txn`, `insert_txn`, `delete_txn`, `get_txn`, `commit_txn`, `rollback_txn`
- Read-your-own-writes; isolation between concurrent transactions
- 12 integration tests

### Layer 17 — Observability
- `Metrics` struct (8 atomic counters)
- `GET /health` — JSON: version, table count, request counters
- `GET /metrics` — Prometheus text: 8 gateway counters + 3 per-table gauges
- 18 integration tests

### Layer 15 — Python Client SDK
- `sdk/python/vectorvolt/` package — zero runtime deps (stdlib `urllib`)
- Full API: CRUD / scan / secondary / DDL / `use_table`
- 14 pytest integration tests against live `vvctl` process
- PyPI-ready via `pyproject.toml`

### Layer 14 — HTTP / REST Gateway
- `vectorvolt-http` crate (axum 0.8); 9 REST endpoints
- `--http-addr` flag on `vvctl serve`
- 12 async reqwest integration tests

### Layer 13 — Engine Benchmark Suite
- 13 Criterion cases across 5 groups: insert, get, scan, secondary, flush

### Layer 12 — Control-Plane C# Tests
- 7 tests: `CreateTable`, `DropTable`, `ListTables`, `TableExists`,
  `UseTable` isolation, `DescribeTable`
- `StartMultiTable()` in `ServerHarness`

### Layer 11 — Query Reads C# Tests
- `SecondaryLookup` + `FkExists` opcodes; 11 C# integration tests
- Fixed `RecvRows()` to set `Row.Pk = values[0]`

### Layer 10 — C# Integration Tests
- 53 xUnit tests against live `vvctl serve` process
- `ServerHarness` spawns real server per test class (OS-free port, temp dir)

### Layer 9 — C# Client SDK Full 122 Opcodes
- All 122 opcodes; correct wire encoding (u32 string prefix, u64 column values)
- 19 unit tests; full parity with the Rust `client.rs`

### Layer 8 — C# Client SDK (Structured Core)
- `IVectorVoltTransport` abstraction; `SocketTransport` (raw Socket, TCP_NODELAY)
- `WireProtocol` codec (24-byte LE header, CRC32)
- `VectorVoltClient` (DDL + DML + aggregates + batch)
- 13 wire protocol unit tests; NuGet-ready

### Layers 1–7 — Rust Engine Foundation
- **Layer 1** — `TableEngine`: open, insert, get, delete, scan, tombstone
- **Layer 2** — WAL (write-ahead log): 14 tests, crash-recovery
- **Layer 3** — Network protocol: 122 opcodes encoded/decoded/tested
- **Layer 4** — CLI: `vvctl serve` (TLS/auth/multi-table/Ctrl-C) + `vvctl connect` (REPL)
- **Layer 5** — Auth: challenge-response (SHA3/nonce), 3 scopes, 19 tests, `vvctl keygen`
- **Layer 6** — Storage hardening: 6 tests (concurrent compaction, 3-cycle reopen,
  delete-all+compact+reopen, partial WAL crash, 32-col wide schema, multi-gen compaction)
- **Layer 7** — Control plane: create/drop/list tables, `use_table`, isolation, admin API

---

## Version Policy

VectorVolt follows [Semantic Versioning](https://semver.org/). Until `v1.0.0` the
public API is unstable and minor versions may contain breaking changes.

[Unreleased]: https://github.com/workerruntime/vectorvolt/compare/HEAD
