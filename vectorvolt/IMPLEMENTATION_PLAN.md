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
| **S3** | Type System + Config + Txn read/write path + Partition routing | 🔄 IN PROGRESS (L105 next) |
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
| **L105** | **VV-CTL-001** | **vvctl serve wired to TxnManager + catalog** | **#183** | **⬜ NEXT** |

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

## Pre-commit Gate (run from `C:\VectorVolt\vectorvolt\`)

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
```

All three must pass before every commit. CI checks fmt first.
