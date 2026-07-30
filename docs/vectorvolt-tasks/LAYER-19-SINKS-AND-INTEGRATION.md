# Layer 19 — Sinks & External Integration (`VV-SNK-*`)

**Added 2026-07-30.** Publishing VectorVolt's WAL outward: to relational, NoSQL, and graph databases,
to event buses, and to analytics stores. 39 features / 231 tasks.

Status: ✅ done · 🟡 partial · ⬜ pending · 🧊 env-gated · 🔬 research

## Rollup

| Group | Features | Tasks | ⬜ |
|---|---|---|---|
| Sink framework | 12 | 78 | 78 |
| Relational sinks | 4 | 22 | 22 |
| NoSQL sinks | 5 | 27 | 27 |
| Graph sinks | 2 | 13 | 13 |
| Analytics / object store | 3 | 18 | 18 |
| Event bus | 9 | 48 | 48 |
| Configuration & operations | 4 | 25 | 25 |
| **Total** | **39** | **231** | **231** |

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

## What this layer does NOT do

- **It does not make VectorVolt depend on anything.** Delete every sink and the database is
  unaffected. LD-KAFKA stands.
- **It does not put an external system on the commit path.** No sink is synchronous with a write, ever.
  Two-phase commit is offered only where a target genuinely supports it, never emulated.
- **It is not bidirectional.** This layer publishes outward. Ingesting *from* an external system is a
  separate concern (a future `VV-SRC-*` source layer) and is deliberately out of scope here.
- **It does not guarantee global ordering.** Per-key ordering is guaranteed; cross-key total order is
  not, once batching and partitioned targets are involved. Promising otherwise would be a lie.
