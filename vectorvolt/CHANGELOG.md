# Changelog

All notable changes to VectorVolt are documented here.

---

## [Unreleased]

### Layer 47 — Packaging & Docs
- `README.md`: project overview, architecture, quick-start, REST API reference,
  DSL reference, SDK examples, benchmark numbers, feature matrix
- `CHANGELOG.md`: full release history
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
