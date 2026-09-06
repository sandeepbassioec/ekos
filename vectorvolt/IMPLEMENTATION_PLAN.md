# VectorVolt Implementation Plan

> **Living document** — updated at the end of every session.
> This file is the source of truth for what is done vs what is next.
> GitHub Issues (milestone S1–S10) are linked from every commit via `Refs #N` or `Closes #N`.

---

## How to rebuild context after a gap

```bash
gh issue list --repo sandeepbassioec/ekos --milestone "S3: Type System + Config + Txn Read/Write" --state all
gh issue list --repo sandeepbassioec/ekos --state open --label "status:in-progress"
git log --oneline -20
```

That three-command sequence tells you exactly where the project stands.

---

## Sprint Roadmap

| Sprint | Theme | Status |
|--------|-------|--------|
| **S1** | HAL Foundations — SIMD, topology, NUMA, huge pages, wait strategies | ✅ COMPLETE |
| **S2** | SIMD Wire-up + CLI UX — storage/query/index SIMD, Zstd, Criterion gate | ✅ COMPLETE |
| **S3** | Type System + Config + Txn read/write path + Partition routing | 🔄 IN PROGRESS (L121 next) |
| **S4** | Network + MVCC — vectorvolt-net, TCP :50051, TLS, BEGIN/COMMIT/ABORT, Prometheus | ⬜ |
| **S5** | Storage tiering L0–L4, snapshots/PITR, Kafka/Postgres sinks | ⬜ |
| **S6** | Cluster — SWIM gossip, 4096 vnodes, Raft, <2s failover | ⬜ |
| **S7** | Control plane + admin UI, Rust+C# SDK, RBAC, package manager hooks | ⬜ |
| **S8** | SQL parser, PostgreSQL wire protocol :5432, HNSW vector index | ⬜ |
| **S9** | Graph engine — CSR, GRAPH_TRAVERSE(), cascade delete, SIMD joins | ⬜ |
| **S10** | Polyglot SDKs (Python/Go/Java/TS), chaos+DST testing, reference domains | ⬜ |

---

## S3 Layer Log

Every layer: feature branch → implementation → fmt+clippy+test → commit with `Refs #N` → push.

| Layer | VV-ID | Description | Issue | Status |
|-------|-------|-------------|-------|--------|
| L80 | VV-ENG-008 | Column type system (U64/I64/F64/Bool/Timestamp/Decimal) | #157 | ✅ |
| L81 | VV-ENG-008 | Schema wired into engine + TableRegistry | #158 | ✅ |
| L82 | VV-QRY-011..015 | DDL in query DSL — CREATE/DROP TABLE | #159 | ✅ |
| L83 | VV-ENG-017 | TOML config file loading | #160 | ✅ |
| L84 | VV-TXN-008 | LMAX Disruptor ring buffer | #161 | ✅ |
| L85 | VV-CFG-006 | vvctl tune — auto-configure host | #162 | ✅ |
| L86 | VV-TXN-009 | Per-shard writer threads | #164 | ✅ |
| L87 | VV-WAL-011 | WAL adaptive batch window | #165 | ✅ |
| L88 | VV-ENG-018 | Multi-table catalog integration | #166 | ✅ |
| L89 | VV-ENG-019 | DDL catalog WAL audit logging | #167 | ✅ |
| L90 | VV-ENG-020 | YAML entity schema loading | #168 | ✅ |
| L91 | VV-TXN-010 | Upsert WAL opcode + TxnManager::upsert() | #169 | ✅ |
| L92 | VV-ENG-009 | ALTER TABLE add/drop columns | #170 | ✅ |
| L93 | VV-TXN-019 | InsertBatch WAL opcode | #171 | ✅ |
| L94 | VV-TXN-020 | DeleteBatch WAL opcode | #172 | ✅ |
| L95 | VV-TXN-021 | UpsertBatch WAL opcode | #173 | ✅ |
| L96 | VV-TXN-022 | TxnManager::get() + scan_all_pks() | #174 | ✅ |
| L97 | VV-TXN-023 | TxnManager::scan() + RingBuffer::head() bugfix | #175 | ✅ |
| L98 | VV-ENG-010 | Fibonacci-hash partition routing | #176 | ✅ |
| L99 | VV-CFG-001 | SystemProfile host capability snapshot | #177 | ✅ |
| L100 | VV-CFG-037 | YAML-primary config + profile-aware defaults | #178 | ✅ |
| L101 | VV-ENG-023 | Aggregate Root Partitioning — AggregateRoot + AggregateRouter | #179 | ✅ |
| L102 | VV-ENG-006 | Multi-table catalog (ShardCatalog) | #180 | ✅ |
| L103 | VV-ENG-007 | Schema registry — typed column persistence | #181 | ✅ |
| L104 | VV-ENG-017 | Config: env + CLI precedence | #182 | ✅ |
| L105 | VV-CTL-001 | vvctl serve wired to TxnManager + catalog | #183 | ✅ |
| L106 | VV-CTL-002 | vvctl catalog subcommand — in-process ShardCatalog management | #184 | ✅ |
| L107 | VV-CTL-003 | SchemaRegistry integration into ShardCatalog create/drop | #185 | ✅ |
| L108 | VV-CTL-004 | vvctl catalog restore + serve --restore-catalog flag | #186 | ✅ |
| L109 | VV-CTL-005 | vvctl catalog validate — schema validation and cross-table integrity checks | #187 | ✅ |
| L110 | VV-CTL-006 | vvctl catalog export/import — schema portability (JSON round-trip) | #188 | ✅ |
| L111 | VV-CTL-007 | vvctl catalog diff — compare two catalog exports or live vs file | #209 | ✅ |
| L112 | VV-CTL-008 | vvctl catalog snapshot — point-in-time catalog backup + restore | #210 | ✅ |
| L113 | VV-CTL-009 | vvctl catalog check-health — aggregate validate + diff + snapshot freshness | #211 | ✅ |
| L114 | VV-CTL-010 | vvctl catalog gc — remove orphan dirs, ghost schema entries, stale snapshots | #212 | ✅ |
| L115 | VV-CTL-011 | vvctl catalog watch — poll catalog for changes and stream diffs | #213 | ✅ |
| L116 | VV-CTL-012 | vvctl catalog stats — per-table disk usage and schema summary | #214 | ✅ |
| L117 | VV-CTL-013 | vvctl catalog rename-table — rename a table in catalog + registry | #215 | ✅ |
| L118 | VV-CTL-014 | vvctl catalog clone — copy a table schema to a new name | #216 | ✅ |
| L119 | VV-CTL-015 | vvctl catalog tag — attach key-value metadata tags to a table | #217 | ✅ |
| L120 | VV-CTL-016 | vvctl catalog lock/unlock — advisory write-lock on a table | #218 | ✅ |
| **L121** | **VV-CTL-017** | **vvctl catalog alias — create a named alias for a table** | **#219** | **⬜ NEXT** |

---

## 🚩 Flagged Milestone — Master Config Schema (VV-CFG-010)

> **Do after S3 control-plane layers complete. Do NOT implement yet — shape is dictated by runtime APIs that must exist first.**

The current `engine.yaml` covers only `flush_threshold`, `durability`, `max_partitions`, and `columns`.
This milestone expands it into a full multi-section configuration authority consumed by `vvctl serve`,
`vvctl catalog`, and eventually `vectorvolt-server`.

### Target schema shape

```yaml
hal:
  cpu_cores: [0, 1, 2, 3]      # core affinity pinning (vectorvolt-hal API)
  numa_node: 0                  # NUMA binding
  huge_pages: 2mb               # 2mb | 1gb | disabled

threads:
  writer_count: 4               # per-shard writer thread count
  ring_buffer_size: 65536       # LMAX Disruptor ring buffer capacity
  writer_priority: normal       # normal | realtime

observability:
  prometheus_addr: "0.0.0.0:9100"
  trace_level: info             # error | warn | info | debug
  jaeger_endpoint: ~            # null = disabled

storage:
  compaction_strategy: leveled  # leveled | tiered | none
  l0_flush_threshold: 4         # SSTable L0 file count trigger
  block_size: 65536             # SSTable block size in bytes

network:
  bind_addr: "0.0.0.0:50051"
  tls_cert: ~
  tls_key: ~
  max_connections: 256
  keepalive_secs: 30

tables:
  orders:
    flush_threshold: 524288     # per-table override
    durability: fsync
    partitions: 8
    entity: Order               # logical entity name
    aggregate_root: true
    members: [order_lines]      # member tables for AggregateRouter
    columns:
      - name: pk
        type: u64
      - name: status
        type: u64

query:
  default_projection: []        # empty = all columns
  max_scan_rows: 0              # 0 = unlimited
  index_hint: auto              # auto | cuckoo | none
```

### Config sections and their runtime dependencies

| Section | Depends on | Status |
|---------|-----------|--------|
| `hal` | `vectorvolt-hal` affinity API | ✅ HAL implemented (S1) — section ready to wire |
| `threads` | `vectorvolt-txn` ring buffer + shard writers | ✅ TxnManager implemented (S3) |
| `observability` | `tracing`, Prometheus metrics | ⬜ S4 target |
| `storage` | `vectorvolt-storage` compaction | ⬜ S5 target |
| `network` | `vectorvolt-net` | ⬜ S4 target |
| `tables` | `ShardCatalog` + `AggregateRouter` | ✅ Both implemented (S3) — **ready now** |
| `query` | `vectorvolt-query` DSL executor | ✅ Partially implemented |

### Implementation approach

1. Expand `EngineConfig` into a versioned top-level struct with optional sections.
2. Each section is its own `#[derive(Deserialize)]` struct with full defaults.
3. `load_config_full` applies per-section env var overrides (e.g. `VECTORVOLT_HAL_CPU_CORES`).
4. Per-table blocks in `tables:` override the global `flush_threshold` / `durability` / `partitions`.
5. `vvctl tune --apply` writes a fully-populated template from `SystemProfile` introspection.

---

## Architecture — Locked Decisions

| Decision | Detail |
|----------|--------|
| **Single operator CLI** | `vvctl` only. Privileged tuning via `vvctl tune --apply`. |
| **Three server binaries** | `vectorvolt-server` :50051, `vectorvolt-reader` :5432, `vectorvolt-control` :8443 |
| **Build targets** | `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-pc-windows-msvc`. No GNU glibc. |
| **Config format** | YAML-primary (VV-CFG-037 ✅). TOML legacy supported. Precedence: generated < file < env < CLI. |
| **Wire protocol** | `WireFrameHeader` `#[repr(C, align(64))]`, CRC-32C per frame, `bytes::Bytes` zero-copy. |
| **Transport chain** | TCP → AF_XDP → DPDK → RDMA. Transport trait makes these drop-in. |
| **ShardCatalog** | Self-hosted via `vectorvolt-store`. No ZooKeeper/etcd. |

---

## Crate Map

| Crate | Status | Notes |
|-------|--------|-------|
| `vectorvolt-hal` | ✅ Active | SIMD, topology, affinity, memory primitives |
| `vectorvolt-wal` | ✅ Active | Adaptive group commit, checkpoint record |
| `vectorvolt-txn` | ✅ Active | Ring buffer, shard writer, all 6 mutation opcodes, get/scan |
| `vectorvolt-engine` | ✅ Active | Typed schema, DDL, YAML config, SystemProfile, partition routing |
| `vectorvolt-storage` | ✅ Active | .vvsst files, zone maps, SIMD scan |
| `vectorvolt-index` | ✅ Active | Cuckoo filter/map, SIMD batch ops |
| `vectorvolt-query` | ✅ Active | DSL lexer/parser/executor, SIMD scan ops |
| `vectorvolt-http` | ✅ Active | axum 0.8 HTTP gateway |
| `vvctl` | ✅ Active | CLI: serve, connect, tune, http, table |
| `vectorvolt-net` | ⬜ Greenfield | S4 — TCP/TLS server |
| `vectorvolt-cluster` | ⬜ Greenfield | S6 — SWIM + Raft |

---

## Known Correctness Gaps (fix before production)

| Gap | Location | VV-ID |
|-----|----------|-------|
| No false-positive re-check on Cuckoo Filter FK hits | `query/engine.rs:89-90` | VV-QRY-004.3 |
| No secondary scan batch fetch | `query/engine.rs:98-100` | VV-QRY-003.5 |
| Zone pruning doesn't save file-open cost | `engine.rs:196-223` | VV-ENG-014 |
| No binary search in partition for range scans | — | VV-QRY-002.6 |
| WAL checkpoint is ad-hoc string, not typed Mutation | `wal/` | VV-WAL-011 |
| UPDATE = delete+insert orphans storage | — | VV-TXN-004.6 |
| Compaction only on unmerged `feat/compaction` branch | — | VV-ENG-004 |
| 400x slower read path in vectorvolt-query | `query/engine.rs` | VV-QRY-001.4 |

---

---

## S22 — Semantic Partitioning + Materialized Read Models + External Cache Tier

> **Status: ⬜ PLANNED — do not implement until S21 is merged and this design is approved.**

### Problem Statement

Today VectorVolt partitions rows by write-order (physical append). There is no concept of
"all rows for customer X land on shard 2" or "all January 2025 orders are in partition 3".
Reads on date ranges or customer IDs must scan every partition. Materialized views exist
in memory only (no external cache), and projection targets are wired in Rust code with no
YAML configuration surface.

### Goals

1. Declare partition strategy per entity in YAML — the engine routes writes to the right
   partition automatically; reads skip irrelevant partitions without scanning them.
2. Give users an in-process LRU read model cache **and** an optional Redis cache tier so
   hot read models serve from RAM/Redis without touching storage at all.
3. Keep all of the above declarative: one `vectorvolt.yaml` section controls everything.

### Feature IDs (new — add to VECTORVOLT_FEATURE_MATRIX.md)

| VV-ID | Name | Description |
|-------|------|-------------|
| VV-PRJ-010 | SemanticPartitionKey | `partition_by:` YAML field on an entity; specifies which column drives physical shard placement |
| VV-PRJ-011 | DateRangePartition | `partition_by: date_column` with optional `granularity: year \| month \| day`; creates one partition file per time bucket |
| VV-PRJ-012 | CustomerPartition | `partition_by: customer_id` (or any FK column); routes all rows for a given ID to one deterministic shard via Fibonacci hash, same as AggregateRouter |
| VV-PRJ-013 | PartitionPruner | At query time, given a WHERE predicate on the partition column, skip all partitions whose bucket cannot match — extends zone-map logic to semantic buckets |
| VV-PRJ-014 | ReadModelRegistry | Runtime registry of named read models, each backed by a `Reducer` impl; replaces the ad-hoc `ProjectionStore` with a lifecycle-managed, named, restartable registry |
| VV-PRJ-015 | InProcessReadCache | LRU cache in front of `ReadModelRegistry`; configurable `max_entries` and TTL; zero-copy `Arc<ReadModelSnapshot>` returns |
| VV-PRJ-016 | RedisReadCache | Optional Redis cache tier — on cache miss, hydrate from storage and write-through to Redis; on hit, return without touching local storage; configurable TTL per read model |
| VV-PRJ-017 | YamlReadModelConfig | `read_models:` section in `vectorvolt.yaml`; declares name, source table, reducer type (`count \| sum \| min \| max \| custom`), partition key, cache strategy, and Redis URL |
| VV-CFG-040 | PartitionConfig | `partitioning:` YAML section per entity: `strategy: date \| customer \| hash \| none`, `column:`, `granularity:`, `shard_count:` |
| VV-CFG-041 | CacheConfig | `cache:` YAML section: `in_process: { max_entries, ttl_s }`, `redis: { url, ttl_s, key_prefix }` |

### YAML Shape (target)

```yaml
entities:
  - name: orders
    schema: orders.yaml
    partitioning:
      strategy: date         # routes writes by date column value
      column: created_at
      granularity: month     # one partition file per calendar month

  - name: line_items
    partitioning:
      strategy: customer     # routes by customer_id FK — same shard as parent order
      column: customer_id

read_models:
  - name: monthly_order_totals
    source: orders
    reducer: sum
    group_by: month(created_at)
    cache:
      in_process:
        max_entries: 10000
        ttl_s: 60
      redis:
        url: redis://localhost:6379
        ttl_s: 300
        key_prefix: "vv:rm:"

  - name: customer_order_count
    source: orders
    reducer: count
    group_by: customer_id
    cache:
      in_process:
        max_entries: 50000
        ttl_s: 30
```

### Implementation Layers (S22)

| Layer | VV-ID | Description |
|-------|-------|-------------|
| S22-L1 | VV-CFG-040 | `PartitionConfig` struct — parse `partitioning:` section from YAML; extend `VvConfig` |
| S22-L2 | VV-PRJ-010 | `SemanticPartitionKey` trait + `PartitionStrategy` enum (Date/Customer/Hash/None) |
| S22-L3 | VV-PRJ-011 | `DateRangeRouter` — maps `created_at` timestamp → partition bucket index by year/month/day |
| S22-L4 | VV-PRJ-012 | `CustomerRouter` — Fibonacci hash on customer FK column → shard index (reuses `AggregateRouter` internals) |
| S22-L5 | VV-PRJ-013 | `PartitionPruner` — extend zone-map query path; given a WHERE on the partition column, compute bucket set and skip non-matching partitions |
| S22-L6 | VV-PRJ-014 | `ReadModelRegistry` — named lifecycle: register, start, stop, replay-from-wal, snapshot |
| S22-L7 | VV-PRJ-015 | `InProcessReadCache` — `Arc<DashMap<K, (Instant, Arc<ReadModelSnapshot>)>>` + TTL eviction |
| S22-L8 | VV-PRJ-016 | `RedisReadCache` — `redis` crate (async), GET/SET/SETEX, write-through on miss, invalidate on mutation |
| S22-L9 | VV-CFG-041 | `CacheConfig` YAML section; wire into `VvConfig`; pass to `ReadModelRegistry` at startup |
| S22-L10 | VV-PRJ-017 | `YamlReadModelConfig` parser; `vvctl serve` reads `read_models:` block and auto-registers projections |
| S22-L11 | VV-CTL-xxx | `vvctl projection list/status/replay/invalidate-cache` subcommands |
| S22-L12 | — | Integration tests: date-partition prune (verify only 1 of 12 monthly partitions scanned), Redis cache hit/miss round-trip, YAML config end-to-end |

### Dependencies / Prerequisites

- VV-ENG-023 `AggregateRouter` ✅ (partition hash logic can be reused)
- VV-SNK-041 `WalSinkBridge` ✅ (read model replay from WAL)
- VV-ENG-020 YAML schema loading ✅
- Redis crate: `redis = "0.25"` (async feature) — new workspace dep
- `dashmap = "5"` — new workspace dep for in-process LRU map

### Acceptance Criteria

- [ ] A query `WHERE created_at BETWEEN '2025-01-01' AND '2025-01-31'` on a monthly-partitioned table reads **exactly one partition file**, verified in test via partition-access counter
- [ ] A read model query on `customer_order_count` returns from in-process cache (< 1 µs) after first hydration
- [ ] Redis cache hit is verified by checking a real Redis key via `redis-cli` in integration test (or a test double implementing the same trait)
- [ ] Removing/modifying `read_models:` from YAML and restarting drops the stale read model from the registry
- [ ] All config is YAML-only — no code changes needed to add a new read model to an existing entity

---

## S23 — Typed Domain Events + YAML Event Routing + Real Broker Integration

> **Status: ⬜ PLANNED — do not implement until S22 is complete and this design is approved.**

### Problem Statement

The existing `vectorvolt-sink` / `WalSinkBridge` emits raw storage-level CDC events
(`SinkEvent { pk: u64, values: Vec<u64> }`). These are useful for replication but not for
business domain logic: a consumer cannot tell an `OrderPlaced` from an `OrderCancelled`
without knowing the column layout. The `KafkaSink` uses an `mpsc` channel stand-in, not a
real broker. There is no path for a consumer to call back into VectorVolt CRUD after
handling an event.

### Goals

1. Let each entity declare its domain events in YAML — name, trigger (insert/update/delete
   or a conditional expression), and typed fields mapped from entity columns.
2. Route those events to real external brokers (Kafka, Redis Streams, HTTP webhook,
   in-process channel) from YAML config — no Rust code required to add a new event route.
3. Provide a typed VectorVolt client SDK call-back path so event consumers (external
   services) can perform CRUD via the existing gRPC or wire-protocol interface, closing
   the loop.

### Feature IDs (new — add to VECTORVOLT_FEATURE_MATRIX.md)

| VV-ID | Name | Description |
|-------|------|-------------|
| VV-EVT-001 | DomainEventSchema | YAML declaration of a domain event: `name`, `trigger` (insert/update/delete/condition), `fields` mapped from entity columns |
| VV-EVT-002 | DomainEventEmitter | Engine hook that fires after a committed WAL entry; maps raw `Mutation` → typed `DomainEvent` using the schema; supports filter expressions (`when: values[2] > 1000`) |
| VV-EVT-003 | DomainEventEnvelope | Wire format for domain events: `{ event_type, entity, pk, timestamp_ns, payload: { field: value } }`; serialised as JSON and MessagePack |
| VV-EVT-004 | KafkaBrokerSink | Real `rdkafka` producer sink — replaces `mpsc` stand-in; configurable topic, partition key, compression, acks |
| VV-EVT-005 | RedisStreamSink | `XADD` to a Redis Stream per event type; configurable stream name, maxlen, and consumer group name |
| VV-EVT-006 | HttpWebhookSink | POST `DomainEventEnvelope` JSON to a configurable URL; retry with exponential back-off; optional HMAC-SHA256 signature header |
| VV-EVT-007 | InProcessChannelSink | `tokio::sync::broadcast` channel — allows in-process consumers (e.g. read model updaters from S22) to subscribe to domain events without a broker |
| VV-EVT-008 | EventOutbox | Transactional outbox pattern — event is written to a WAL outbox frame in the same transaction as the mutation; a background flusher delivers to the broker; guarantees at-least-once delivery even if the broker is temporarily down |
| VV-EVT-009 | YamlEventRouteConfig | `domain_events:` section in `vectorvolt.yaml`; declares event name, trigger, field mapping, and one or more sink routes |
| VV-EVT-010 | ConsumerCrudCallback | Documentation + SDK example showing how an external consumer calls back into VectorVolt via gRPC (`WriteService`) or wire protocol to perform CRUD — no new engine code needed; this is a pattern, not a feature, but must be tested end-to-end |
| VV-EVT-011 | EventReplayApi | `vvctl events replay --entity orders --from <lsn> --to <lsn>` — re-emits stored WAL events through the domain event pipeline; useful for consumer recovery |
| VV-EVT-012 | DomainEventDlq | Dead-letter queue for failed deliveries (broker unreachable, webhook 5xx); stores failed envelopes in a local WAL segment; `vvctl events dlq list/replay/drop` |

### YAML Shape (target)

```yaml
domain_events:
  - entity: orders
    events:
      - name: OrderPlaced
        trigger: insert
        fields:
          order_id:    pk
          customer_id: values[0]
          total_cents: values[1]
          status:      values[2]
        routes:
          - sink: kafka
            topic: orders.placed
            partition_key: customer_id
          - sink: webhook
            url: https://crm.internal/hooks/order-placed
            hmac_secret: ${CRM_WEBHOOK_SECRET}

      - name: LargeOrderPlaced
        trigger: insert
        when: "values[1] > 100000"   # total_cents > $1000
        fields:
          order_id:    pk
          total_cents: values[1]
        routes:
          - sink: redis_stream
            stream: vv:events:large-orders
            maxlen: 10000

      - name: OrderCancelled
        trigger: delete
        fields:
          order_id: pk
        routes:
          - sink: kafka
            topic: orders.cancelled

sinks:
  kafka:
    brokers: ["kafka:9092"]
    compression: lz4
    acks: all
    retries: 3
  redis_stream:
    url: redis://localhost:6379
  webhook:
    timeout_s: 5
    retry_max: 3
    retry_backoff_ms: 500
```

### Implementation Layers (S23)

| Layer | VV-ID | Description |
|-------|-------|-------------|
| S23-L1 | VV-EVT-001 | `DomainEventSchema` struct; YAML parser for `domain_events:` block; extend `VvConfig` |
| S23-L2 | VV-EVT-003 | `DomainEventEnvelope` — JSON + MessagePack serialisation; `serde_json` + `rmp-serde` |
| S23-L3 | VV-EVT-002 | `DomainEventEmitter` — post-commit hook in `TxnManager`; evaluates `when:` filter; maps `Mutation` fields to envelope payload using schema |
| S23-L4 | VV-EVT-008 | `EventOutbox` — WAL frame type `OUTBOX`; background flusher loop; at-least-once delivery guarantee |
| S23-L5 | VV-EVT-007 | `InProcessChannelSink` — `tokio::sync::broadcast`; allows S22 read model updaters to consume events without broker round-trip |
| S23-L6 | VV-EVT-004 | `KafkaBrokerSink` — `rdkafka` producer; topic from route config; partition key from field mapping |
| S23-L7 | VV-EVT-005 | `RedisStreamSink` — `XADD` via `redis` crate async; maxlen trimming |
| S23-L8 | VV-EVT-006 | `HttpWebhookSink` — `reqwest` POST; HMAC-SHA256 `X-VV-Signature` header; retry with exponential back-off |
| S23-L9 | VV-EVT-009 | `YamlEventRouteConfig` parser; `vvctl serve` reads `domain_events:` block and wires emitter + sinks at startup |
| S23-L10 | VV-EVT-012 | `EventDlq` — local WAL segment for failed envelopes; `vvctl events dlq list/replay/drop` |
| S23-L11 | VV-EVT-011 | `vvctl events replay --entity --from --to` — re-emit WAL range through emitter pipeline |
| S23-L12 | VV-EVT-010 | End-to-end integration test: insert order → `OrderPlaced` fires → in-process consumer receives envelope → consumer calls `TxnManager::insert()` on a second table (simulating CRUD callback) |

### Dependencies / Prerequisites

- VV-SNK-041 `WalSinkBridge` ✅ (WAL-tail infrastructure reused)
- VV-NET-004..016 gRPC write server ✅ (consumer CRUD callback uses this)
- VV-EVT-007 `InProcessChannelSink` must land before `KafkaBrokerSink` so tests can run without a real broker
- S22 `ReadModelRegistry` recommended before S23 (InProcessChannelSink feeds read model updates)
- New workspace deps: `rdkafka = "0.36"` (kafka feature), `rmp-serde = "1"` (MessagePack)
- `rdkafka` requires `librdkafka` — feature-gate behind `kafka` feature flag so non-Kafka builds stay clean

### Acceptance Criteria

- [ ] Inserting an order row triggers `OrderPlaced` envelope delivery to the in-process channel (no broker needed in unit tests)
- [ ] `when: "values[1] > 100000"` filter correctly suppresses events for small orders
- [ ] If the broker is down, events accumulate in the WAL outbox and are delivered when the broker recovers
- [ ] DLQ captures envelopes that fail after `retry_max` attempts; `vvctl events dlq list` shows them
- [ ] An external process using the gRPC `WriteService` can insert into a second table in response to a webhook delivery (consumer CRUD callback path tested end-to-end)
- [ ] All routing config is YAML-only — adding a new Kafka topic for an existing event requires zero Rust changes

### Traceability Matrix (S22 + S23 vs existing VV-IDs)

| New VV-ID | Extends / Relates to | Relationship |
|-----------|---------------------|--------------|
| VV-PRJ-010..013 | VV-ENG-023 `AggregateRouter` | Reuses Fibonacci hash; extends to date and customer strategies |
| VV-PRJ-014..015 | VV-PRJ-003..008 `ProjectionStore` | Replaces ad-hoc store with named, lifecycle-managed, cached registry |
| VV-PRJ-016 | VV-SNK-014..035 (Redis planned sink) | Promotes Redis from write-path sink to read-cache tier |
| VV-PRJ-017 | VV-ENG-020 YAML schema | Extends same YAML config file with `read_models:` section |
| VV-CFG-040..041 | VV-CFG-037 YAML-primary config | New sections in same `vectorvolt.yaml`; same precedence chain |
| VV-EVT-001..003 | VV-SNK-040 `Sink` trait | Promotes from storage CDC to typed business domain events |
| VV-EVT-004 | VV-SNK-013 `KafkaSink` | Replaces mpsc stand-in with real `rdkafka` producer |
| VV-EVT-005 | VV-SNK-014..035 (Redis planned) | Implements Redis Streams specifically from the planned NoSQL sink list |
| VV-EVT-008 | VV-WAL-* | Adds `OUTBOX` WAL frame type; reuses WAL segment infrastructure |
| VV-EVT-011 | Layer 67 CDC event log | Extends replay API to domain-event-aware replay |

---

## Pre-commit Gate (run from `C:\VectorVolt\vectorvolt\`)

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
```

All three must pass before every commit. CI checks fmt first.
