# Layer 19 — Sinks & External Integration (`VV-SNK-*`)

**Added 2026-07-30.** Publishing VectorVolt's WAL outward: to relational, NoSQL, and graph databases,
to event buses, and to analytics stores — plus the adapter SDK that lets anyone add a target we did
not write. **52 features / 209 tasks.**

Status: ✅ done · 🟡 partial · ⬜ pending · 🧊 env-gated · 🔬 research

## Rollup

| Group | Features | Tasks |
|---|---|---|
| 1. Sink framework | 12 | 54 |
| 2. Relational sinks | 4 | 15 |
| 3. NoSQL sinks | 5 | 13 |
| 4. Graph sinks | 2 | 7 |
| 5. Analytics / object store | 3 | 9 |
| 6. Event bus | 9 | 25 |
| 7. Configuration & operations | 4 | 18 |
| 8. **Adapter SDK & extensibility** | **11** | **58** |
| 9. Additional named targets | 2 | 10 |
| **Total** | **52** | **209** |

All ⬜ pending — nothing in this layer is built yet.

Counts are measured from this file (unique `VV-SNK-NNN` ids and `VV-SNK-NNN.N` task rows), not
asserted. An earlier revision of this header claimed 231 tasks, which was an estimate rather than a
count; corrected here.

---

## ⚠️ Reconciling this with LD-KAFKA and LD-PROJ

The existing docs state, verbatim:

> **Why not actual Kafka:** VectorVolt's event stream is internal to the cluster.
> **No external dependencies at runtime:** No ZooKeeper, no etcd, no Kafka.

Read literally, that forbids this layer. It does not, and the distinction must be explicit or a
future reader will "enforce" LD-KAFKA and delete the sink subsystem:

| | Meaning | Verdict |
|---|---|---|
| **Inbound dependency** | VectorVolt *requires* Kafka/etcd/ZooKeeper to function | **Forbidden — LD-KAFKA stands unchanged** |
| **Outbound integration** ← *this layer* | VectorVolt *optionally feeds* external systems | **Permitted, and strategically important** |

The test: **delete every sink and VectorVolt must be completely unaffected.** Sinks are consumers of
an internal log that already exists for projections. They add no runtime dependency, no quorum
participant, and nothing on the commit path.

Similarly, `STORAGE_ENGINE_ADVANCED.md` frames VectorVolt as *replacing* the "PostgreSQL + Neo4j means
dual ingestion, dual consistency" problem. That remains the long-term position. Sinks serve the
*migration and coexistence* reality: nobody replaces their warehouse, search index, or reporting stack
to trial a new database. Sinks let VectorVolt become the system of record while existing downstream
consumers keep working — which is how a database gets adopted rather than merely admired.

### The one inviolable constraint

**A slow or failing sink must never stall, slow, or fail the write path.**

This extends LD-PROJ ("projections are eventually consistent, never synchronous on the command path")
to external systems, where it matters far more — an external network hop can hang for minutes.
If adding a Kafka sink makes VectorVolt as slow and as available as Kafka, the feature is a net
negative. Every task in the framework group exists to enforce this.

---

# Group 1 — Sink framework

### VV-SNK-001 — `Sink` trait and registry  ⬜
**Blocked by:** VV-PRJ-003 (WAL-as-event-log with LSN offsets)

| Task | Status | Detail |
|---|---|---|
| VV-SNK-001.1 | ⬜ | Define `Sink` trait: `open`, `write_batch(&[ChangeEvent])`, `flush`, `close`, `health` |
| VV-SNK-001.2 | ⬜ | Define `ChangeEvent { lsn, table, op: Insert/Update/Delete, before, after, txn_id, commit_ts }` |
| VV-SNK-001.3 | ⬜ | Registry mapping a config-declared sink `type` to an implementation |
| VV-SNK-001.4 | ⬜ | Sinks are **batch-oriented** — per-row network round-trips would make any sink useless |
| VV-SNK-001.5 | ⬜ | `SinkError` distinguishing retryable (network) from permanent (schema mismatch) — they need opposite responses |
| VV-SNK-001.6 | ⬜ | In-memory test sink, so framework behaviour is testable with no external service |

### VV-SNK-002 — WAL-tailing runner  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-SNK-002.1 | ⬜ | Tail committed WAL frames, decode to `ChangeEvent`, reusing the projection worker's cursor machinery rather than a parallel implementation |
| VV-SNK-002.2 | ⬜ | One independent task per sink — one slow sink must not delay another |
| VV-SNK-002.3 | ⬜ | Run on background cores only (VV-CFG-008 role map); never on a writer core |
| VV-SNK-002.4 | ⬜ | Batch by size *and* time (whichever first) so idle systems still deliver promptly |
| VV-SNK-002.5 | ⬜ | Only read frames below the durable commit watermark — never publish an uncommitted write |
| VV-SNK-002.6 | ⬜ | Test: a sink that blocks forever leaves write latency statistically unchanged |

### VV-SNK-003 — At-least-once delivery with offset checkpointing  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-SNK-003.1 | ⬜ | Persist per-sink committed LSN durably |
| VV-SNK-003.2 | ⬜ | Checkpoint *after* the sink acknowledges, never before — the ordering that makes it at-least-once rather than at-most-once |
| VV-SNK-003.3 | ⬜ | Resume from the checkpoint on restart |
| VV-SNK-003.4 | ⬜ | Hold WAL segments needed by the slowest sink; surface that they are pinned (interacts with VV-WAL-013 log compaction) |
| VV-SNK-003.5 | ⬜ | Bound that hold — a dead sink must not grow the WAL without limit. Configurable max lag, then drop the sink with a loud alarm |
| VV-SNK-003.6 | ⬜ | Test: kill mid-batch, restart, assert no event lost (duplicates permitted) |

### VV-SNK-004 — Dead letter queue and pause-on-failure  ⬜
### VV-SNK-005 — Backpressure isolation  ⬜
### VV-SNK-006 — Schema mapping and transforms  ⬜
### VV-SNK-007 — Table/column filtering and routing  ⬜
### VV-SNK-008 — Lag and throughput metrics  ⬜
### VV-SNK-009 — Ordering guarantees  ⬜
### VV-SNK-010 — Replay and backfill  ⬜
### VV-SNK-011 — Health checks and circuit breaker  ⬜
### VV-SNK-012 — Exactly-once for idempotent sinks  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-SNK-004.1 | ⬜ | Permanent failures go to a per-sink DLQ with the event and the error |
| VV-SNK-004.2 | ⬜ | Pause the sink after N consecutive failures — never silently skip, which turns a bug into invisible data loss |
| VV-SNK-004.3 | ⬜ | `vvctl sink replay-dlq` after the operator fixes the cause |
| VV-SNK-004.4 | ⬜ | Alert on pause; a paused sink is an incident, not a log line |
| VV-SNK-005.1 | ⬜ | Bounded per-sink queue; when full, the *sink* falls behind — the write path never blocks |
| VV-SNK-005.2 | ⬜ | Overflow policy per sink: `lag` (default, keep reading WAL from disk) or `drop-oldest` (explicit opt-in for lossy telemetry-style sinks) |
| VV-SNK-005.3 | ⬜ | Never `drop-newest`, and never block — both silently corrupt the downstream view |
| VV-SNK-005.4 | ⬜ | Test: sink queue saturated → p99 write latency unchanged within noise |
| VV-SNK-006.1 | ⬜ | Declarative column→field mapping, including rename and omit |
| VV-SNK-006.2 | ⬜ | Type coercion per target (u64 → BIGINT / NUMBER / Long), with an explicit error on a lossy conversion rather than silent truncation |
| VV-SNK-006.3 | ⬜ | Optional computed/derived fields |
| VV-SNK-006.4 | ⬜ | Redaction/masking of columns per sink — a reporting sink should not receive PII the primary holds |
| VV-SNK-006.5 | ⬜ | Envelope choice: raw row, or before/after change record |
| VV-SNK-007.1 | ⬜ | Include/exclude by schema, table, and column |
| VV-SNK-007.2 | ⬜ | Route by predicate (e.g. only `region = 'EU'` rows to the EU sink) |
| VV-SNK-007.3 | ⬜ | Route by operation (e.g. deletes only to the audit sink) |
| VV-SNK-007.4 | ⬜ | Fan-out: one table to many sinks; fan-in: many tables to one sink |
| VV-SNK-008.1 | ⬜ | `vv_sink_lag_lsn`, `vv_sink_lag_seconds`, `vv_sink_events_total`, `vv_sink_errors_total` per sink |
| VV-SNK-008.2 | ⬜ | Batch-size and write-latency histograms per sink |
| VV-SNK-008.3 | ⬜ | Surface in `vvctl sink status` and the admin panel |
| VV-SNK-009.1 | ⬜ | Guarantee per-primary-key ordering; document that cross-key global order is **not** guaranteed once batching and partitioned targets are involved |
| VV-SNK-009.2 | ⬜ | Partition-key selection for ordered targets (Kafka key = PK by default) |
| VV-SNK-009.3 | ⬜ | Preserve transaction boundaries where the sink can express them; document where it cannot |
| VV-SNK-009.4 | ⬜ | Test: rapid update sequence on one PK arrives in order |
| VV-SNK-010.1 | ⬜ | `vvctl sink replay --from-lsn` / `--from-timestamp` |
| VV-SNK-010.2 | ⬜ | Full backfill from a snapshot plus WAL, for a newly added sink that must see history |
| VV-SNK-010.3 | ⬜ | Replay must not disturb live sinks |
| VV-SNK-010.4 | ⬜ | Warn plainly that replay produces duplicates at a non-idempotent sink |
| VV-SNK-011.1 | ⬜ | Periodic health probe per sink |
| VV-SNK-011.2 | ⬜ | Circuit breaker: exponential backoff on repeated failure, rather than hammering a downed system |
| VV-SNK-011.3 | ⬜ | Automatic resume when health recovers |
| VV-SNK-011.4 | ⬜ | Distinguish "sink down" from "sink slow" — they warrant different alerts |
| VV-SNK-012.1 | ⬜ | Idempotency key derived from `(table, pk, lsn)` |
| VV-SNK-012.2 | ⬜ | Upsert-by-key where the target supports it, making replay safe |
| VV-SNK-012.3 | ⬜ | Two-phase commit only where the target genuinely supports it; **never** emulate it, which would put an external system on our commit path |
| VV-SNK-012.4 | ⬜ | Document per sink which guarantee it actually achieves — at-least-once or effectively-once |

---

# Group 2 — Relational sinks

### VV-SNK-013 PostgreSQL · VV-SNK-014 MySQL/MariaDB · VV-SNK-015 SQL Server · VV-SNK-016 Generic SQL

| Task | Status | Detail |
|---|---|---|
| VV-SNK-013.1 | ⬜ | `tokio-postgres` sink with batched multi-row `INSERT ... ON CONFLICT DO UPDATE` |
| VV-SNK-013.2 | ⬜ | `COPY` protocol for bulk backfill — orders of magnitude faster than INSERT |
| VV-SNK-013.3 | ⬜ | DDL bootstrap: optionally create the target table from our schema |
| VV-SNK-013.4 | ⬜ | Schema-drift detection: fail loudly, never silently drop a column |
| VV-SNK-013.5 | ⬜ | TLS + connection pooling |
| VV-SNK-013.6 | ⬜ | Test against a real PostgreSQL in a container |
| VV-SNK-014.1 | ⬜ | MySQL sink via `sqlx`, `INSERT ... ON DUPLICATE KEY UPDATE` |
| VV-SNK-014.2 | ⬜ | Handle MySQL's looser type coercion explicitly rather than inheriting its surprises |
| VV-SNK-014.3 | ⬜ | Test against MariaDB as well as MySQL |
| VV-SNK-015.1 | ⬜ | SQL Server sink via `tiberius`, `MERGE` for upsert |
| VV-SNK-015.2 | ⬜ | Bulk-copy path for backfill |
| VV-SNK-015.3 | ⬜ | Test against `mssql` container |
| VV-SNK-016.1 | ⬜ | Generic SQL sink: configurable dialect, statement template, batch size |
| VV-SNK-016.2 | ⬜ | Escape/parameterize correctly per dialect — **never** string-concatenate values into SQL |
| VV-SNK-016.3 | ⬜ | Document it as best-effort; the named sinks are the supported path |

---

# Group 3 — NoSQL sinks

### VV-SNK-017 MongoDB · VV-SNK-018 Redis · VV-SNK-019 DynamoDB · VV-SNK-020 Elasticsearch/OpenSearch · VV-SNK-021 Cassandra/ScyllaDB

| Task | Status | Detail |
|---|---|---|
| VV-SNK-017.1 | ⬜ | Mongo sink: `bulkWrite` with upsert on `_id` mapped from PK |
| VV-SNK-017.2 | ⬜ | Nested-document shaping from flat rows (config-driven) |
| VV-SNK-017.3 | ⬜ | Deletes as real deletes or soft-delete flags, configurable |
| VV-SNK-018.1 | ⬜ | Redis sink: pipelined `SET`/`HSET`, key template from config |
| VV-SNK-018.2 | ⬜ | Optional TTL per sink — Redis as a cache tier fed by the WAL |
| VV-SNK-018.3 | ⬜ | Cluster-mode support with hash-tag-aware batching |
| VV-SNK-019.1 | ⬜ | DynamoDB `BatchWriteItem` (25-item cap) with partition/sort key mapping |
| VV-SNK-019.2 | ⬜ | Handle throttling as retryable, with respect for the provisioned rate |
| VV-SNK-020.1 | ⬜ | Bulk API with per-document `_id` from PK |
| VV-SNK-020.2 | ⬜ | Index-template bootstrap and mapping from schema |
| VV-SNK-020.3 | ⬜ | Full-text search over VectorVolt data without VectorVolt implementing search |
| VV-SNK-021.1 | ⬜ | CQL batched upsert with partition-key mapping |
| VV-SNK-021.2 | ⬜ | Respect Cassandra's batch-size guidance rather than sending unbounded batches |

---

# Group 4 — Graph sinks

### VV-SNK-022 Neo4j · VV-SNK-023 Generic property graph

| Task | Status | Detail |
|---|---|---|
| VV-SNK-022.1 | ⬜ | Bolt-protocol sink emitting parameterized Cypher `MERGE` |
| VV-SNK-022.2 | ⬜ | Config-declared node/edge mapping: which table is a node, which FK is an edge |
| VV-SNK-022.3 | ⬜ | Reuse VV-QRY-015's FK-derived graph model so node/edge mapping is not defined twice |
| VV-SNK-022.4 | ⬜ | Batched transactions, since per-edge round-trips are unusable |
| VV-SNK-022.5 | ⬜ | Test against a `neo4j` container |
| VV-SNK-023.1 | ⬜ | Gremlin/TinkerPop sink for JanusGraph, Neptune, Cosmos DB Gremlin API |
| VV-SNK-023.2 | ⬜ | Shared node/edge mapping vocabulary with VV-SNK-022 |

---

# Group 5 — Analytics and object storage

### VV-SNK-024 Parquet→object store · VV-SNK-025 ClickHouse · VV-SNK-026 Warehouse (Snowflake/BigQuery)

| Task | Status | Detail |
|---|---|---|
| VV-SNK-024.1 | ⬜ | Buffer to Parquet and upload to S3/Azure Blob/GCS on size or time trigger |
| VV-SNK-024.2 | ⬜ | Hive-style partitioned paths (`dt=/hour=`) for query engines |
| VV-SNK-024.3 | ⬜ | Our columnar layout maps to Parquet almost directly — exploit that rather than round-tripping through rows |
| VV-SNK-024.4 | ⬜ | Optional Iceberg/Delta manifest emission |
| VV-SNK-025.1 | ⬜ | ClickHouse native-protocol batch insert |
| VV-SNK-025.2 | ⬜ | `ReplacingMergeTree` mapping so updates and deletes behave |
| VV-SNK-026.1 | ⬜ | Snowflake sink via staged files plus `COPY INTO` |
| VV-SNK-026.2 | ⬜ | BigQuery via storage-write API |
| VV-SNK-026.3 | ⬜ | Document cost implications — a naive per-row warehouse sink is financially hostile |

---

# Group 6 — Event bus

### VV-SNK-027 Kafka · VV-SNK-028 NATS · VV-SNK-029 RabbitMQ/AMQP · VV-SNK-030 Pulsar · VV-SNK-031 AWS (EventBridge/SNS/SQS) · VV-SNK-032 Azure/GCP · VV-SNK-033 HTTP webhook · VV-SNK-034 CloudEvents envelope · VV-SNK-035 Outbox pattern

| Task | Status | Detail |
|---|---|---|
| VV-SNK-027.1 | ⬜ | Kafka producer (`rdkafka`), configurable topic template per table |
| VV-SNK-027.2 | ⬜ | Message key = PK by default, preserving per-key ordering |
| VV-SNK-027.3 | ⬜ | `acks`, compression, linger, idempotent-producer settings exposed in config |
| VV-SNK-027.4 | ⬜ | Schema Registry (Avro/Protobuf/JSON Schema) as an option |
| VV-SNK-027.5 | ⬜ | SASL/SSL authentication |
| VV-SNK-027.6 | ⬜ | **Optional Cargo feature** — `rdkafka` links a C library; the default build must not require it |
| VV-SNK-027.7 | ⬜ | Test against a `kafka` container |
| VV-SNK-028.1 | ⬜ | NATS / JetStream publish, subject template per table |
| VV-SNK-028.2 | ⬜ | Pure-Rust client, so this can be a default-on sink unlike Kafka |
| VV-SNK-029.1 | ⬜ | AMQP 0-9-1 publish with exchange/routing-key templates |
| VV-SNK-029.2 | ⬜ | Publisher confirms for at-least-once |
| VV-SNK-030.1 | ⬜ | Pulsar producer with topic template |
| VV-SNK-031.1 | ⬜ | EventBridge `PutEvents` with detail-type mapping |
| VV-SNK-031.2 | ⬜ | SNS and SQS variants |
| VV-SNK-031.3 | ⬜ | IAM role-based auth (no static keys in config — see VV-SNK-038) |
| VV-SNK-032.1 | ⬜ | Azure Event Hubs sink |
| VV-SNK-032.2 | ⬜ | GCP Pub/Sub sink |
| VV-SNK-033.1 | ⬜ | Batched HTTP POST to a configured URL |
| VV-SNK-033.2 | ⬜ | HMAC request signing so receivers can verify authenticity |
| VV-SNK-033.3 | ⬜ | Retry with backoff, honouring `Retry-After` |
| VV-SNK-033.4 | ⬜ | Per-request timeout — an unresponsive endpoint must not accumulate tasks |
| VV-SNK-034.1 | ⬜ | CloudEvents 1.0 envelope as a selectable format across every event sink |
| VV-SNK-034.2 | ⬜ | Stable `source` and `id` so consumers can deduplicate |
| VV-SNK-035.1 | ⬜ | Transactional outbox: a designated table whose rows are published then marked, for applications that need publish-with-transaction semantics |
| VV-SNK-035.2 | ⬜ | Document that this gives the application atomicity without VectorVolt joining a distributed transaction |

---

# Group 7 — Configuration and operations

### VV-SNK-036 — Declarative YAML sink configuration  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-SNK-036.1 | ⬜ | `sinks:` block: list of named sinks, each with `type`, `enabled`, filters, mapping, batching, target settings |
| VV-SNK-036.2 | ⬜ | Validate at load — unknown sink type, bad filter, or unmapped required field fails with a specific message |
| VV-SNK-036.3 | ⬜ | Validate connectivity on `vvctl config validate --check-sinks`, without starting the server |
| VV-SNK-036.4 | ⬜ | A disabled sink is inert but retains its checkpoint, so re-enabling resumes rather than replays |
| VV-SNK-036.5 | ⬜ | Worked examples per sink type in the config reference (VV-CFG-038) |

### VV-SNK-037 — Hot add/remove without restart  ⬜
### VV-SNK-038 — Credential handling  ⬜
### VV-SNK-039 — `vvctl sink` commands  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-SNK-037.1 | ⬜ | Add a sink at runtime on config reload; start it at the current LSN (or backfill if asked) |
| VV-SNK-037.2 | ⬜ | Remove a sink cleanly: stop, flush, release pinned WAL segments |
| VV-SNK-037.3 | ⬜ | Change a sink's settings without losing its checkpoint |
| VV-SNK-037.4 | ⬜ | Test: add, remove, and reconfigure under sustained write load |
| VV-SNK-038.1 | ⬜ | **Credentials must never be written in the YAML.** Reference `${env:VAR}` or a secret store |
| VV-SNK-038.2 | ⬜ | Support AWS/Azure/GCP managed identity so no secret exists to leak |
| VV-SNK-038.3 | ⬜ | Redact credentials from logs, `vvctl` output, and support bundles |
| VV-SNK-038.4 | ⬜ | Refuse to start on a literal-looking secret in config, with a message explaining the alternative |
| VV-SNK-039.1 | ⬜ | `vvctl sink list` — name, type, state, lag, error count |
| VV-SNK-039.2 | ⬜ | `vvctl sink pause` / `resume` |
| VV-SNK-039.3 | ⬜ | `vvctl sink replay --from-lsn` / `--from-timestamp` |
| VV-SNK-039.4 | ⬜ | `vvctl sink replay-dlq` and `vvctl sink dlq list` |
| VV-SNK-039.5 | ⬜ | `vvctl sink test <name>` — send a synthetic event end-to-end |

---

---

# Group 8 — Adapter SDK and extensibility

**Added 2026-07-30.** The reframe that makes this layer viable commercially.

## The problem this group solves

Customers will name targets we have not written: Oracle, CosmosDB, SAP HANA, DB2, Informix, an
internal message bus. Shipping 25 native adapters does not fix it — the 26th request still loses the
deal. And 25 native adapters means maintaining 25 integrations against 25 upstream projects' version
churn indefinitely, which is how integration-heavy projects collapse under their own weight.

So the load-bearing features here are **not the adapters**. They are the *contract*, the *generic
adapters*, and the *plugin protocol*.

### Three tiers, by effort and by who maintains them

| Tier | Mechanism | Covers | Maintainer |
|---|---|---|---|
| **1. Generic, config-only** | YAML: connection string + statement template + type mapping | **Oracle, DB2, HANA, Informix, Snowflake** — anything with SQL. Plus any REST API — **CosmosDB**, cloud services | Us: 3 adapters, not 25 |
| **2. Native Rust** | Implements `Sink`, in-tree or as an external crate | The 5–6 worth hand-optimising | Us for core, community beyond |
| **3. Out-of-process** | Subprocess speaking gRPC over a Unix socket | **Any language.** JDBC in a Java sidecar, Python, Go | The customer |

Tier 1 alone likely answers ~70% of "can you support X?" with **zero new code** — just YAML. That is
the difference between "we'll add it next quarter" and "here is the config."

**Why out-of-process is not optional:** there is no mature pure-Rust Oracle driver (the `oracle` crate
needs OCI client libraries), so Oracle realistically arrives via ODBC or a JDBC sidecar. Designing for
that now beats discovering it during a deal. It also buys crash isolation — **a buggy
customer-written adapter must not be able to take down the database** — and the IPC cost is
acceptable precisely because sinks are async and batched, so it amortises across a whole batch rather
than per event.

**Why not WASM,** despite `wasmtime` already being planned for CIP (VV-AUT-004): a sink's entire job
is to talk to the network, and WASI's socket story is not mature enough to connect to Oracle or
Kafka. gRPC-over-UDS to a native subprocess is the right mechanism for *this* problem.

---

### VV-SNK-040 — Stable, versioned public `Sink` trait  ⬜
**The contract everything else depends on.**

| Task | Status | Detail |
|---|---|---|
| VV-SNK-040.1 | ⬜ | Publish the trait in its own minimal crate (`vectorvolt-sink-api`) so an adapter author depends on the contract, not on the engine |
| VV-SNK-040.2 | ⬜ | Semver discipline: adding a *required* trait method is a breaking change. New capabilities arrive as defaulted methods |
| VV-SNK-040.3 | ⬜ | `ChangeEvent` is part of the public contract; version its wire form explicitly |
| VV-SNK-040.4 | ⬜ | Declare and honour an MSRV, so adapter authors know what toolchain they need |
| VV-SNK-040.5 | ⬜ | Compatibility policy documented: which engine versions accept which API versions |
| VV-SNK-040.6 | ⬜ | CI check: any change to the trait's required surface fails without a major-version bump |

### VV-SNK-041 — Adapter capability declaration  ⬜
**The framework adapts to the adapter, rather than assuming every target behaves like PostgreSQL.**

| Task | Status | Detail |
|---|---|---|
| VV-SNK-041.1 | ⬜ | `SinkCapabilities { upsert, delete, transactions, idempotent_write, max_batch, ordered_per_key, ddl_bootstrap, .. }` |
| VV-SNK-041.2 | ⬜ | Framework consults capabilities: no upsert → insert-then-update fallback; no idempotency → warn plainly that replay duplicates |
| VV-SNK-041.3 | ⬜ | Refuse a config that requests a guarantee the adapter has not declared, rather than silently degrading it |
| VV-SNK-041.4 | ⬜ | `vvctl sink capabilities <name>` prints what a target actually guarantees |
| VV-SNK-041.5 | ⬜ | Every shipped adapter documents its real guarantee — at-least-once or effectively-once — with no hand-waving |

### VV-SNK-042 — Out-of-tree native adapters  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-SNK-042.1 | ⬜ | Registration so an external crate can supply a sink type name |
| VV-SNK-042.2 | ⬜ | Decide and document the linkage model: recompile-with-feature (simple, safe) vs `dlopen` (flexible, ABI-fragile). Recommend the former; Rust has no stable ABI |
| VV-SNK-042.3 | ⬜ | A `vectorvolt-server` build that includes third-party adapters must report them in `vvctl version` |
| VV-SNK-042.4 | ⬜ | Example out-of-tree adapter crate in the repo, kept compiling in CI |

### VV-SNK-043 — Generic SQL adapter (config-only)  ⬜
**Tier 1. The single highest-leverage feature in this layer.**

| Task | Status | Detail |
|---|---|---|
| VV-SNK-043.1 | ⬜ | Driver selection by URL scheme (`postgres://`, `mysql://`, `mssql://`, `odbc://`) |
| VV-SNK-043.2 | ⬜ | **ODBC backend** — the escape hatch that reaches Oracle, DB2, HANA, Informix, Teradata, and anything else with an ODBC driver |
| VV-SNK-043.3 | ⬜ | Config-declared statement templates for insert / upsert / delete, with named parameter binding |
| VV-SNK-043.4 | ⬜ | **Always parameterised, never string-concatenated** — a template engine that interpolates values into SQL is an injection vector by construction |
| VV-SNK-043.5 | ⬜ | Config-declared type mapping per target, with an explicit error on lossy conversion |
| VV-SNK-043.6 | ⬜ | Declared dialect quirks: identifier quoting, upsert syntax, max parameters per statement |
| VV-SNK-043.7 | ⬜ | Batch-size negotiation against the driver's parameter limit |
| VV-SNK-043.8 | ⬜ | Ship worked YAML examples for **Oracle** and **DB2** as proof the escape hatch works |
| VV-SNK-043.9 | ⬜ | Document the tradeoff honestly: slower than a native adapter, available immediately for anything |

### VV-SNK-044 — Generic HTTP/REST adapter (config-only)  ⬜
**Tier 1. Reaches CosmosDB and essentially every cloud service.**

| Task | Status | Detail |
|---|---|---|
| VV-SNK-044.1 | ⬜ | Config-declared method, URL template, headers, and body template per operation |
| VV-SNK-044.2 | ⬜ | Body templating to JSON with field mapping from the change event |
| VV-SNK-044.3 | ⬜ | Auth modes: bearer, basic, API key header, HMAC signing, AWS SigV4, Azure AD, GCP OIDC |
| VV-SNK-044.4 | ⬜ | Batch semantics: N events per request where the API supports it, else pipelined singles |
| VV-SNK-044.5 | ⬜ | Response handling: which status codes are success, retryable, or permanent |
| VV-SNK-044.6 | ⬜ | Honour `Retry-After`; per-request timeout; bounded concurrency |
| VV-SNK-044.7 | ⬜ | Worked YAML example for **CosmosDB** (SQL API) and a generic webhook |

### VV-SNK-045 — Generic gRPC adapter  ⬜
### VV-SNK-046 — Out-of-process adapter protocol  ⬜
**Tier 3. Any language, with crash isolation.**

| Task | Status | Detail |
|---|---|---|
| VV-SNK-045.1 | ⬜ | Config-declared proto descriptor + method; map change events to request messages |
| VV-SNK-045.2 | ⬜ | Reflection or a supplied descriptor set, so no recompile is needed per target |
| VV-SNK-046.1 | ⬜ | Define the adapter protocol as protobuf: `Open`, `WriteBatch`, `Flush`, `Health`, `Capabilities`, `Close` |
| VV-SNK-046.2 | ⬜ | Transport over Unix domain socket (named pipe on Windows) — never TCP by default, which would expose the plane |
| VV-SNK-046.3 | ⬜ | Supervise the subprocess: spawn, health-check, restart with backoff, surface state |
| VV-SNK-046.4 | ⬜ | **Crash isolation:** an adapter that panics, hangs, or leaks must not affect the engine. Enforce a timeout and kill; treat it as sink-down, not engine-down |
| VV-SNK-046.5 | ⬜ | Checkpoint only on acknowledgement, so a crash mid-batch replays rather than loses |
| VV-SNK-046.6 | ⬜ | Resource limits per adapter process (memory, CPU share) so it cannot starve the engine |
| VV-SNK-046.7 | ⬜ | Reference adapters in **Python** and **Java** (the Java one demonstrating JDBC → Oracle) |
| VV-SNK-046.8 | ⬜ | Document the latency cost measured, not assumed, so users can choose tier deliberately |

### VV-SNK-047 — Adapter conformance test suite  ⬜
**How an author proves an adapter is correct without reading our internals.**

| Task | Status | Detail |
|---|---|---|
| VV-SNK-047.1 | ⬜ | Publishable test harness an adapter author runs against their own implementation |
| VV-SNK-047.2 | ⬜ | Asserts: per-key ordering, at-least-once under kill-restart, DLQ on permanent error, backpressure without blocking, batch-boundary correctness |
| VV-SNK-047.3 | ⬜ | Asserts declared capabilities match observed behaviour — an adapter claiming idempotency is tested for it |
| VV-SNK-047.4 | ⬜ | Fault injection: target down, target slow, target returning partial success |
| VV-SNK-047.5 | ⬜ | Machine-readable conformance report, so certification is evidence-based rather than a claim |
| VV-SNK-047.6 | ⬜ | Every in-tree adapter passes it — the suite is not just for outsiders |

### VV-SNK-048 — Certification tiers  ⬜
### VV-SNK-049 — Adapter registry and version negotiation  ⬜
### VV-SNK-050 — Adapter developer kit  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-SNK-048.1 | ⬜ | Three tiers: **Core** (we maintain, we test, we support), **Certified** (third-party, passes conformance, listed), **Community** (unverified, clearly labelled) |
| VV-SNK-048.2 | ⬜ | `vvctl sink list --available` shows each adapter's tier, so nobody deploys an experimental adapter believing it supported |
| VV-SNK-048.3 | ⬜ | Log a warning at startup for a non-Core adapter, naming its tier |
| VV-SNK-048.4 | ⬜ | Publish the support policy: what Core guarantees and what it does not |
| VV-SNK-049.1 | ⬜ | Manifest per adapter: name, version, API version, capabilities, tier, maintainer |
| VV-SNK-049.2 | ⬜ | Reject an adapter built against an incompatible API version, with a message stating both versions |
| VV-SNK-049.3 | ⬜ | Discoverable index of known adapters, resolvable offline for air-gapped installs |
| VV-SNK-050.1 | ⬜ | `cargo generate` template producing a compiling skeleton adapter with conformance tests wired up |
| VV-SNK-050.2 | ⬜ | Local harness: run an adapter against a synthetic change stream with no engine required |
| VV-SNK-050.3 | ⬜ | Authoring guide covering batching, idempotency, and error classification — the three things adapter authors get wrong |
| VV-SNK-050.4 | ⬜ | Worked walkthrough: build a working adapter for an unsupported target, start to finish |

---

# Group 9 — Additional named targets

Explicitly requested and not covered by the original Layer 19 list.

### VV-SNK-051 — Oracle  ⬜
### VV-SNK-052 — Azure CosmosDB  ⬜

| Task | Status | Detail |
|---|---|---|
| VV-SNK-051.1 | ⬜ | **Path A (available immediately):** generic SQL adapter over ODBC — worked YAML config, no new code |
| VV-SNK-051.2 | ⬜ | **Path B:** out-of-process JDBC adapter, the pragmatic route given Oracle's Rust driver situation |
| VV-SNK-051.3 | ⬜ | **Path C (only on real demand):** native adapter via the `oracle` crate, which requires OCI client libraries on the host — a deployment burden worth avoiding until a customer justifies it |
| VV-SNK-051.4 | ⬜ | `MERGE INTO` upsert template and Oracle type mapping (`NUMBER` scale/precision is the trap) |
| VV-SNK-051.5 | ⬜ | Document the three paths with their tradeoffs so the choice is informed |
| VV-SNK-052.1 | ⬜ | SQL API via the generic HTTP adapter — worked YAML, no new code |
| VV-SNK-052.2 | ⬜ | MongoDB API via the existing Mongo adapter (VV-SNK-017) — note this is often the simplest route |
| VV-SNK-052.3 | ⬜ | Gremlin API via VV-SNK-023 for graph workloads |
| VV-SNK-052.4 | ⬜ | Partition-key mapping, since Cosmos requires an explicit partition key |
| VV-SNK-052.5 | ⬜ | Request-Unit awareness: treat 429 as retryable and respect the returned backoff, or the bill and the throttling both bite |

---

## What this layer does NOT do

- **It does not make VectorVolt depend on anything.** Delete every sink and the database is
  unaffected. LD-KAFKA stands.
- **It does not put an external system on the commit path.** No sink is synchronous with a write, ever.
  Two-phase commit is offered only where a target genuinely supports it, never emulated.
- **It is not bidirectional.** This layer publishes outward. Ingesting *from* an external system is a
  separate concern (a future `VV-SRC-*` source layer) and is deliberately out of scope here.
- **It does not guarantee global ordering.** Per-key ordering is guaranteed; cross-key total order is
  not, once batching and partitioned targets are involved. Promising otherwise would be a lie.
