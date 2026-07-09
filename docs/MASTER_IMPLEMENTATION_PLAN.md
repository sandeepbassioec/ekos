# Master Implementation Plan
## VectorVolt + EKOS Engine — Every Item, Every Phase, Every Status

**Last updated:** 2026-07-09  
**Branch:** feature-ekos-phase1  
**How to read this:**
- ✅ Done and committed
- 🔄 In progress
- ⬜ Pending (scheduled for this phase)
- 🔵 Future (later phase, not yet scheduled)
- 📄 Architecture doc = the spec for that item

---

## Scope Boundary — Two Trackers, One System

**This document tracks the Rust engine layer:**

| Scope | What it covers |
|---|---|
| **VectorVolt** | Nanosecond OLTP + graph + vector query engine, WAL, MVCC, Micro-Partitions, Cuckoo indexes, SIMD joins, HNSW, gRPC server, client SDKs, vvctl CLI |
| **EKOS engine** | Operational intelligence running on top of VectorVolt: ARP self-healing, CIP capability injection, Shadowbox sandboxing, behavioral fingerprinting, in-process LLM, predictive pre-fetch, dependency self-upgrade, admin panel, 6 simulation modes |

**The EKOS Knowledge OS layer is tracked separately:**

> `esip/docs/ekos/BUILD-PLAN-AND-OQ-TRIAGE.md` — EKOS Knowledge OS build plan  
> `esip/docs/ekos/00-MASTER-INDEX.md` — 27-volume requirements catalog (~1,560 requirements)  
> `esip/docs/ekos/architecture/` — 12 architecture docs (A00–A11)
>
> That layer covers: `.ekr` Knowledge Runtime Package, EKR Viewer family, Ambient AI engine,
> semantic knowledge graph, Widget OS, Personal Knowledge Space / Digital Twin, Knowledge Economy,
> Knowledge Networks, Knowledge Session Runtime (P2P), Knowledge Workforce Runtime,
> Knowledge Kernel + Compiler + Projection Engine, patents (22+ families), ESIP→EKOS migration.

**The two layers connect at the storage port:** EKOS Knowledge OS uses VectorVolt as its storage backend via the VectorVolt client SDK (Phase 3+). VectorVolt does not depend on EKOS Knowledge OS.

---

## Phase 1 — WAL + Core Storage ✅ COMPLETE

**Goal:** Working Write-Ahead Log, KoStore, Windows MSVC build, CI pipelines, EKOS workspace.

| Status | Item | Crate | Spec Doc |
|---|---|---|---|
| ✅ | WAL — segment write, fsync, replay on restart | `vectorvolt-wal` | — |
| ✅ | KoStore — ko_put, ko_get, ko_history, ko_scan | `vectorvolt-store` | — |
| ✅ | 14 unit tests passing | both | — |
| ✅ | Windows MSVC build fix (`.cargo/config.toml`) | workspace | — |
| ✅ | EKOS workspace separation (`ekos/` independent) | `ekos-kernel`, `ekosd` | — |
| ✅ | 18 EKOS kernel tests passing | `ekos-kernel` | — |
| ✅ | VectorVolt CI pipeline | `.github/workflows/vectorvolt-ci.yml` | — |
| ✅ | EKOS CI pipeline | `.github/workflows/ekos-ci.yml` | — |

---

## Phase 2 — Storage Engine Core + Security ⬜ NEXT

**Goal:** Nanosecond read path data structures, FK validation without I/O, VECTOR column type,
MFA security mandate, observability instrumentation foundation.

### vectorvolt-index (new crate)
| Status | Item | Spec Doc |
|---|---|---|
| ⬜ | Cuckoo Hash Map primary key index | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §3 |
| ⬜ | Row Descriptor (bit-packed u64: file_id + partition_id + offset) | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §3 |
| ⬜ | Roaring Bitmap secondary index | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §4 |
| ⬜ | Zone Map index (per-partition min/max summaries) | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §5 |
| ⬜ | Cuckoo Filter (existence check, L3-resident) | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §7 |
| ⬜ | Primary-Key Hash Cache (DRAM, authoritative, 9B/row) | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §7 |

### vectorvolt-storage (new crate)
| Status | Item | Spec Doc |
|---|---|---|
| ⬜ | Micro-Partition file format (columnar, dictionary-encoded) | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §10 |
| ⬜ | `mmap` with `MAP_POPULATE` (pre-fault all pages into RAM) | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §10 |
| ⬜ | Dictionary encoding for string/enum columns | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §10 |
| ⬜ | Struct-of-Arrays layout within Micro-Partitions | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §10 |
| ⬜ | Zone Map population at Micro-Partition write time | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §5 |

### vectorvolt-query (new crate)
| Status | Item | Spec Doc |
|---|---|---|
| ⬜ | Point lookup: Cuckoo Hash → Row Descriptor → Micro-Partition read | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §3 |
| ⬜ | Secondary index lookup: Roaring Bitmap → SIMD iterate | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §4 |
| ⬜ | Range scan: Zone Map prune → binary search → SIMD stream | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §5 |
| ⬜ | FK validation: Cuckoo Filter + PK Hash Cache (zero I/O) | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §7 |
| ⬜ | Merge-on-Read Delta Log reconstruction | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §8 |
| ⬜ | Per-core DeltaLog (append-only, no read before write) | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §8 |

### vectorvolt-vector (new crate)
| Status | Item | Spec Doc |
|---|---|---|
| ⬜ | `VECTOR(n)` native column type | [IN_PROCESS_LLM_EMBEDDING](vision/IN_PROCESS_LLM_EMBEDDING.md) §4 |
| ⬜ | HNSW index construction (incremental, one INSERT at a time) | [IN_PROCESS_LLM_EMBEDDING](vision/IN_PROCESS_LLM_EMBEDDING.md) §6 |
| ⬜ | `cosine_similarity()` SQL function | [IN_PROCESS_LLM_EMBEDDING](vision/IN_PROCESS_LLM_EMBEDDING.md) §6 |
| ⬜ | VECTOR column storage layout in Micro-Partition | [IN_PROCESS_LLM_EMBEDDING](vision/IN_PROCESS_LLM_EMBEDDING.md) §4 |

### vectorvolt-server (new crate) — gRPC + network layer
| Status | Item | Spec Doc |
|---|---|---|
| ⬜ | gRPC server (tonic) — execute, query, txn begin/commit/abort endpoints | — (Phase 2 arch TBD) |
| ⬜ | Connection pool + per-core shard routing | — |
| ⬜ | `x-vv-txn` fingerprint injection into response metadata | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §4 |
| ⬜ | TLS + mTLS support (server cert, optional client cert) | — |
| ⬜ | gRPC health probe (`grpc.health.v1.Health`) | — |

### vvctl (new binary crate) — operator CLI
| Status | Item | Spec Doc |
|---|---|---|
| ⬜ | `vvctl debug txn <fingerprint>` — decode transaction context | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §4 |
| ⬜ | `vvctl synthetic generate` — schema-aware fake data generation | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §3 |
| ⬜ | `vvctl index list` / `vvctl index rebuild` — index management | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) |
| ⬜ | `vvctl backup status` — backup health check | [BACKUP_STRATEGY](security/BACKUP_STRATEGY.md) |
| ⬜ | `vvctl qos override` — runtime QoS rule injection | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §1 |

### Transaction Fingerprint
| Status | Item | Spec Doc |
|---|---|---|
| ⬜ | 128-bit fingerprint computed at commit time (~5 ns, FNV-1a) | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §4 |
| ⬜ | Fingerprint embedded in gRPC response metadata (`x-vv-txn`) | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §4 |
| ⬜ | `TransactionContext` async write to cold context store | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §4 |
| ⬜ | `vvctl debug txn <fingerprint>` CLI command | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §4 |

### ekos-security (new crate)
| Status | Item | Spec Doc |
|---|---|---|
| ⬜ | `MfaGuard::verify_startup()` — reads PAM session token | [MFA_SESSION_MANDATE](security/MFA_SESSION_MANDATE.md) §5 |
| ⬜ | HMAC-SHA256 session token verification | [MFA_SESSION_MANDATE](security/MFA_SESSION_MANDATE.md) §3 |
| ⬜ | Periodic `recheck()` every 5 min — graceful shutdown on expiry | [MFA_SESSION_MANDATE](security/MFA_SESSION_MANDATE.md) §5 |
| ⬜ | Integrate `MfaGuard` into `ekosd` main.rs (startup gate) | [MFA_SESSION_MANDATE](security/MFA_SESSION_MANDATE.md) §5 |
| ⬜ | Integrate `MfaGuard` into `vvctl` main.rs | [MFA_SESSION_MANDATE](security/MFA_SESSION_MANDATE.md) §5 |
| ⬜ | PAM exec script (`ekos-mfa-token-writer`) + install docs | [MFA_SESSION_MANDATE](security/MFA_SESSION_MANDATE.md) §4 |

### Observability Foundation (ekos-kernel addition)
| Status | Item | Spec Doc |
|---|---|---|
| ⬜ | Per-syscall latency histogram (p50, p95, p99) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §2.3 |
| ⬜ | Per-syscall error rate counter | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §2.3 |
| ⬜ | KvStore compaction duration + output size ratio metric | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §2.3 |
| ⬜ | WAL write throughput + segment count metric | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §2.3 |
| ⬜ | `QuiesceGuard` in KoStore (foundation for ARP) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §8 |
| ⬜ | Zero-overhead MVCC row header telemetry (7 extra bytes) | [RESEARCH_FRONTIER](vision/RESEARCH_FRONTIER.md) §3 |

---

## Phase 3 — Graph Engine + Embedding + CIP + QoS 🔵

**Goal:** CSR graph traversal, in-process LLM, graph-relational bridge, Capability Injection Protocol,
transaction prioritization, synthetic data generator, cross-partition checksum.

### vectorvolt-sdk-rust (new crate) — Rust client SDK
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Async gRPC client wrapper (tonic-based, connection pooling) | — |
| 🔵 | Typed query builder API (`client.query().from("table").where_eq(...)`) | — |
| 🔵 | `client.graph().from().traverse()` graph traversal API | [GRAPH_RELATIONAL_BRIDGE](vision/GRAPH_RELATIONAL_BRIDGE.md) §3 |
| 🔵 | Transaction builder (`client.txn().begin().execute().commit()`) | — |
| 🔵 | Published to crates.io as `vectorvolt-client` | — |

### vectorvolt-sdk-dotnet — C# / .NET client SDK
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | NuGet package `VectorVolt.Client` (.NET 8+) | — |
| 🔵 | Grpc.Net.Client wrapper with async/await | — |
| 🔵 | LINQ query provider surface (`context.Table<T>().Where(...)`) | — |
| 🔵 | C# was the original VectorVolt prototype — SDK is the blueprint of that work reborn | — |

### vectorvolt-sdk-go, sdk-java, sdk-python — polyglot SDKs
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Go SDK (`vectorvolt-go`) — gRPC + idiomatic Go client | — |
| 🔵 | Java/Kotlin SDK — gRPC + reactive streams | — |
| 🔵 | Python SDK — gRPC + pandas DataFrame integration | — |
| 🔵 | All SDKs generated from the same protobuf definitions | — |

---

### vectorvolt-graph (new crate)
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | CSR (Compressed Sparse Row) graph storage from FK schema | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §6 |
| 🔵 | Reverse CSR (incoming edges, for CASCADE DELETE + graph queries) | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §6 |
| 🔵 | CSR auto-derived from FK relationships at schema change | [GRAPH_RELATIONAL_BRIDGE](vision/GRAPH_RELATIONAL_BRIDGE.md) §5 |
| 🔵 | Depth-N traversal API via CSR sequential scan | [GRAPH_RELATIONAL_BRIDGE](vision/GRAPH_RELATIONAL_BRIDGE.md) §3 |
| 🔵 | CSR Rebuild Workflow (background, non-blocking, atomic pointer swap) | [GRAPH_RELATIONAL_BRIDGE](vision/GRAPH_RELATIONAL_BRIDGE.md) §5 |
| 🔵 | Graph query SDK extension (`client.graph().from().traverse()`) | [GRAPH_RELATIONAL_BRIDGE](vision/GRAPH_RELATIONAL_BRIDGE.md) §3 |

### vectorvolt-query additions
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | SIMD vectorization (AVX-512) on Roaring Bitmap iterate | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §4 |
| 🔵 | Cascade DELETE: Roaring Bitmap child discovery + bulk soft-delete | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §9 |
| 🔵 | Tombstone soft-delete + async compaction background task | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §9 |

### ekos-embedding (new crate)
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | llama.cpp or ONNX Runtime wrapper (`EmbeddingEngine`) | [IN_PROCESS_LLM_EMBEDDING](vision/IN_PROCESS_LLM_EMBEDDING.md) §3 |
| 🔵 | CPU affinity pinning to dedicated core group | [IN_PROCESS_LLM_EMBEDDING](vision/IN_PROCESS_LLM_EMBEDDING.md) §8 |
| 🔵 | cgroup memory budget enforcement | [IN_PROCESS_LLM_EMBEDDING](vision/IN_PROCESS_LLM_EMBEDDING.md) §8 |
| 🔵 | `generate_embedding()` SQL function (synchronous) | [IN_PROCESS_LLM_EMBEDDING](vision/IN_PROCESS_LLM_EMBEDDING.md) §5 |
| 🔵 | Back-pressure via max queue depth | [IN_PROCESS_LLM_EMBEDDING](vision/IN_PROCESS_LLM_EMBEDDING.md) §8 |
| 🔵 | Training pair accumulation from production corrections | [IN_PROCESS_LLM_EMBEDDING](vision/IN_PROCESS_LLM_EMBEDDING.md) §7 |

### QoS + Storage Intelligence
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | `TransactionPriority` enum (Critical / High / Normal / Low) | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §1 |
| 🔵 | Pattern-based priority assignment at parse time (~10 ns) | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §1 |
| 🔵 | Strict priority queue in per-core scheduler | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §1 |
| 🔵 | `ekosctl qos override` CLI command | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §1 |
| 🔵 | Synthetic Data Generator (schema-aware, FK-aware, histogram-matched) | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §3 |
| 🔵 | `vvctl synthetic generate` CLI command | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §3 |
| 🔵 | Cross-partition checksum (CRC-32C at write time) | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §5 |
| 🔵 | `ChecksumVerifyWorkflow` (nightly, background) | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §5 |

### ekos-cip (new crate) — Capability Injection Protocol
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Admin gRPC channel with mTLS (`InjectCapability`, `ListCapabilities`, `RevokeCapability`) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §3.2 |
| 🔵 | `CapabilityIntent` → LLM synthesis → Rust → WASM compile | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §3.2 |
| 🔵 | `wasmtime` sandbox with plugin API (`KoPlugin` trait) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §3.2 |
| 🔵 | `CapabilityRegistry` dynamic dispatch (DashMap) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §3.3 |
| 🔵 | `InvokeCapability` generic gRPC method (no proto recompile) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §3.4 |
| 🔵 | Admin panel: minimal capability list/approve UI | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §10 |

### Hardware Integrations
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Intel QAT auto-detect at startup + fallback to software AES | [HARDWARE_FRONTIER](vision/HARDWARE_FRONTIER.md) §3 |
| 🔵 | `EncryptionBackend` trait (QAT / software implementations) | [HARDWARE_FRONTIER](vision/HARDWARE_FRONTIER.md) §3 |

### Backup — Layer 1
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | `ekos-backup` crate scaffolding | [BACKUP_STRATEGY](security/BACKUP_STRATEGY.md) §8 |
| 🔵 | WAL Shipper — stream segments to hot standby after segment close | [BACKUP_STRATEGY](security/BACKUP_STRATEGY.md) §2 |
| 🔵 | Source namespace snapshot after every successful build | [BACKUP_STRATEGY](security/BACKUP_STRATEGY.md) §5 |

---

## Phase 4 — Workflow Runtime + Shadowbox + Admin + Pre-fetch 🔵

**Goal:** Full sandbox infrastructure, PromotionEngine, admin web UI, predictive memory management,
index advisor, Intel DSA, full dependency upgrade Class A.

### ekos-workflow (new crate)
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | `Shadowbox` struct — Linux namespace isolation (CLONE_NEWNS/NET/PID) | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §3 |
| 🔵 | Overlay tmpfs writable layer (discarded on drop) | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §3 |
| 🔵 | `ShadowboxMode` enum (dev/qa/uat/live) | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §3 |
| 🔵 | `ConfidenceScore` calculation (latency delta, output divergence, error delta) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §5.1 |
| 🔵 | `PromotionEngine` — RequireApproval / AutoIfConfident / FullAuto | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §5.1 |
| 🔵 | Workflow definition + trigger (Schedule / OnEvent / Manual) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §4 |
| 🔵 | Windows Job Objects sandbox (Windows equivalent of namespaces) | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §3 |

### ekos-admin (new crate) + admin-ui (Svelte SPA)
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Admin HTTP port `:8443` (axum, HTTPS, embedded in ekosd) | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §4 |
| 🔵 | Svelte SPA compiled to static assets, embedded via `include_bytes!` | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §8 |
| 🔵 | Admin login: bcrypt password + TOTP (same PAM secret) | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §4 |
| 🔵 | Session JWT (4h, signed with server key) | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §4 |
| 🔵 | Pending Approvals dashboard | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §5 |
| 🔵 | Diff view (before/after, confidence breakdown, sandbox results) | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §5 |
| 🔵 | `TrustPolicy` configuration UI (per-subject: ARP / CIP / Workflow / Named) | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §6 |
| 🔵 | Emergency Stop button + Resume Autonomous Operations | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §7 |
| 🔵 | All trust policy changes stored as immutable audit KOs | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §6 |
| 🔵 | Per-priority QoS throughput + queue depth dashboard | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §1 |

### ekos-prefetch (new crate)
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | `TransactionDependencyGraph` — Markov Chain, edge weights, temporal decay | [PREDICTIVE_PREFETCH_ENGINE](vision/PREDICTIVE_PREFETCH_ENGINE.md) §2 |
| 🔵 | Background pre-fetch loop (Tier 2 DRAM pre-loading, 50ms tick) | [PREDICTIVE_PREFETCH_ENGINE](vision/PREDICTIVE_PREFETCH_ENGINE.md) §3 |
| 🔵 | Tier 1 `mlock()` promotion for probability > 95% PKs | [PREDICTIVE_PREFETCH_ENGINE](vision/PREDICTIVE_PREFETCH_ENGINE.md) §3 |
| 🔵 | Per-query AST speculation thread (spare CPU core) | [PREDICTIVE_PREFETCH_ENGINE](vision/PREDICTIVE_PREFETCH_ENGINE.md) §4 |
| 🔵 | Hot Subgraph pre-computation for top 1% graph nodes | [PREDICTIVE_PREFETCH_ENGINE](vision/PREDICTIVE_PREFETCH_ENGINE.md) §5 |
| 🔵 | Negative caching integration (Cuckoo Filter miss prediction) | [PREDICTIVE_PREFETCH_ENGINE](vision/PREDICTIVE_PREFETCH_ENGINE.md) §6 |
| 🔵 | Hot Subgraph invalidation on INSERT/DELETE edge | [PREDICTIVE_PREFETCH_ENGINE](vision/PREDICTIVE_PREFETCH_ENGINE.md) §5 |

### HITL Index Advisor (ekos-admin addition)
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Execution plan analysis — missed index opportunity detection | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §2 |
| 🔵 | Index recommendation ranking (by estimated impact) | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §2 |
| 🔵 | Admin panel: Index Advisor section (Accept / Override / Dismiss) | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §2 |
| 🔵 | DBA risk tolerance learning (`DbaPreference` model) | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §2 |
| 🔵 | Shadowbox validation of proposed index before promotion | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §2 |

### Graph Relational Bridge additions
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | `CREATE GRAPH_EDGE` syntax (cross-table semantic links beyond FK) | [GRAPH_RELATIONAL_BRIDGE](vision/GRAPH_RELATIONAL_BRIDGE.md) §4 |
| 🔵 | `GRAPH_TRAVERSE()` SQL extension with predicate support | [GRAPH_RELATIONAL_BRIDGE](vision/GRAPH_RELATIONAL_BRIDGE.md) §4 |
| 🔵 | Weighted graph traversals (edge weight from column value) | [GRAPH_RELATIONAL_BRIDGE](vision/GRAPH_RELATIONAL_BRIDGE.md) §4 |

### Hardware Integrations
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Intel DSA auto-detect for memory copy + CRC operations | [HARDWARE_FRONTIER](vision/HARDWARE_FRONTIER.md) §3 |
| 🔵 | DSA-accelerated buffer pool fills (Shadowbox init, backup snapshots) | [HARDWARE_FRONTIER](vision/HARDWARE_FRONTIER.md) §3 |

### Backup — Layers 2+3
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Full daily KoStore snapshot (all KO kinds) | [BACKUP_STRATEGY](security/BACKUP_STRATEGY.md) §2 |
| 🔵 | `BackupVerifyWorkflow` (weekly restore in Shadowbox dev) | [BACKUP_STRATEGY](security/BACKUP_STRATEGY.md) §7 |
| 🔵 | Admin panel: backup status (last verified date + result) | [BACKUP_STRATEGY](security/BACKUP_STRATEGY.md) §7 |

### Dependency Self-Upgrade — Class A
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | `ekos-upgrade` crate scaffolding | [DEPENDENCY_SELF_UPGRADE](vision/DEPENDENCY_SELF_UPGRADE.md) §8 |
| 🔵 | `DependencyWatchWorkflow` (weekly Cargo crate version check) | [DEPENDENCY_SELF_UPGRADE](vision/DEPENDENCY_SELF_UPGRADE.md) §3 |
| 🔵 | `cargo audit` CVE advisory watcher + expedited pipeline trigger | [DEPENDENCY_SELF_UPGRADE](vision/DEPENDENCY_SELF_UPGRADE.md) §3 |
| 🔵 | Class A upgrade pipeline (dev → qa → ConfidenceScore → auto/approve) | [DEPENDENCY_SELF_UPGRADE](vision/DEPENDENCY_SELF_UPGRADE.md) §3 |

---

## Phase 5 — ARP + Behavioral Fingerprinting + Evidence Correlation + Full MVCC 🔵

**Goal:** Complete self-healing loop, behavioral security, evidence correlation, LoRA fine-tuning,
quorum checksum witnesses, RDMA WAL shipping, MVCC isolation, vectorized joins.

### ARP — Autonomous Remediation Protocol (ekos-arp, new crate)
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | `ObservabilityAgent` — classify events into 4-domain error taxonomy | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §2 |
| 🔵 | `RemediationRouter` — route classified event to correct handler | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §2.2 |
| 🔵 | **Capture Phase** — `ErrorCapture` struct with exact gRPC payload bytes | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §6.1 |
| 🔵 | Preceding 50-transaction sliding window capture | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §6.1 |
| 🔵 | Store state snapshot at time of failure | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §6.1 |
| 🔵 | **Diagnosis Phase** — LLM + source code analysis → root cause | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §6.2 |
| 🔵 | **Synthesis Phase** — LLM generates WASM patch candidate | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §6.3 |
| 🔵 | **Validation Layer 1** — original payload replay (error must not reproduce) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §6.4 |
| 🔵 | **Validation Layer 2** — `PayloadFuzzer` (14+ variant types) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §6.4 |
| 🔵 | **Validation Layer 3** — regression corpus (growing, never shrinks) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §6.4 |
| 🔵 | **Promote Phase** — PromotionEngine gates (confidence score, human approval) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §6.5 |
| 🔵 | Admin panel: ARP diff view, patch review, UAT sandbox console | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §5 |

### Evidence Correlation Engine (ekos-arp addition)
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | `Evidence` type with 5 kinds (Infra / Runtime / App / Business / Evolution) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §9.2 |
| 🔵 | Temporal clustering (30-second co-occurrence window) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §9.3 |
| 🔵 | Causal graph walk (find triggering event from KO audit trail) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §9.3 |
| 🔵 | Hypothesis scoring with confidence weights | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §9.3 |
| 🔵 | Route: deployment regression → human rollback review (NOT ARP) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §9.3 |
| 🔵 | Route: infra issue → self-heal, do NOT synthesize code patch | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §9.3 |

### Behavioral Fingerprinting (ekos-security addition)
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | `BehavioralModel` — online EMA statistical model per KO namespace | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §7.3 |
| 🔵 | `KoOperationBaseline` (payload size, version delta, tag change rate) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §7.3 |
| 🔵 | `AnomalyScore` (Normal / Suspicious / Critical) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §7.3 |
| 🔵 | Write-path integration: score every ko_put | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §7.4 |
| 🔵 | Replay attack detection, data poisoning detection | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §7.2 |

### Self-Upgrading Core Binary
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Build new ekosd binary in Shadowbox("dev") from source KOs | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §7.5 |
| 🔵 | kexec-style fd handoff (new process inherits listening sockets) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §7.5 |
| 🔵 | `RequireHumanApproval` hard constraint (no FullAuto for binary upgrade) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §7.5 |
| 🔵 | Old binary archived as KO artifact (rollback in reverse) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §7.5 |

### In-Process LLM — LoRA Fine-tuning
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | LoRA adapter computation (nightly cron, inference cores) | [IN_PROCESS_LLM_EMBEDDING](vision/IN_PROCESS_LLM_EMBEDDING.md) §7 |
| 🔵 | Shadowbox validation of new adapter (held-out validation set) | [IN_PROCESS_LLM_EMBEDDING](vision/IN_PROCESS_LLM_EMBEDDING.md) §7 |
| 🔵 | Atomic adapter swap (old archived as KO, rollback available) | [IN_PROCESS_LLM_EMBEDDING](vision/IN_PROCESS_LLM_EMBEDDING.md) §7 |
| 🔵 | Admin panel: embedding quality metrics, fine-tune history | [IN_PROCESS_LLM_EMBEDDING](vision/IN_PROCESS_LLM_EMBEDDING.md) §12 |

### Pre-fetch Engine additions
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Admin panel: working set coverage dashboard (Tier 1/2/3 %) | [PREDICTIVE_PREFETCH_ENGINE](vision/PREDICTIVE_PREFETCH_ENGINE.md) §7 |
| 🔵 | ARP integration: classify Tier 3 misses as resource issue, not code bug | [PREDICTIVE_PREFETCH_ENGINE](vision/PREDICTIVE_PREFETCH_ENGINE.md) §8 |
| 🔵 | Decay rate auto-tuning based on workload variability | [PREDICTIVE_PREFETCH_ENGINE](vision/PREDICTIVE_PREFETCH_ENGINE.md) §10 |

### Storage + Durability
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Full MVCC snapshot isolation on all read paths | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §13 |
| 🔵 | Columnar vectorized joins (hash join, sort-merge join) | [READ_PATH_ARCHITECTURE](vision/READ_PATH_ARCHITECTURE.md) §13 |
| 🔵 | Graph + relational hybrid query planner | [GRAPH_RELATIONAL_BRIDGE](vision/GRAPH_RELATIONAL_BRIDGE.md) §7 |
| 🔵 | Graph analytics (path counting, connectivity, basic centrality) | [GRAPH_RELATIONAL_BRIDGE](vision/GRAPH_RELATIONAL_BRIDGE.md) §7 |
| 🔵 | Quorum witness checksum (cross-node partition verification) | [STORAGE_ENGINE_ADVANCED](vision/STORAGE_ENGINE_ADVANCED.md) §5 |

### Dependency Self-Upgrade — Class B
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Class B pipeline (Rust toolchain upgrade, 72h UAT soak) | [DEPENDENCY_SELF_UPGRADE](vision/DEPENDENCY_SELF_UPGRADE.md) §4 |
| 🔵 | `RequireHumanApproval` for all toolchain upgrades (no FullAuto) | [DEPENDENCY_SELF_UPGRADE](vision/DEPENDENCY_SELF_UPGRADE.md) §4 |

### Source Control Strategy
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | `ekos-scm` crate (~1000 lines) on top of KoStore | [SOURCE_CONTROL_STRATEGY](vision/SOURCE_CONTROL_STRATEGY.md) |
| 🔵 | `ChangeProposal` replaces PR (stored as KO) | [SOURCE_CONTROL_STRATEGY](vision/SOURCE_CONTROL_STRATEGY.md) |
| 🔵 | `ekosctl` CLI — source control commands | [SOURCE_CONTROL_STRATEGY](vision/SOURCE_CONTROL_STRATEGY.md) |
| 🔵 | imara-diff + diffy integration for diff/merge | [SOURCE_CONTROL_STRATEGY](vision/SOURCE_CONTROL_STRATEGY.md) |

### Hardware Integrations
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | RDMA transport for WAL segment shipping to hot standby | [HARDWARE_FRONTIER](vision/HARDWARE_FRONTIER.md) §4 |
| 🔵 | TCP fallback when RDMA NIC not present | [HARDWARE_FRONTIER](vision/HARDWARE_FRONTIER.md) §4 |

---

## Phase 6 — Reflexive Encrypted Runtime + Class C Kernel 🔵

**Goal:** Source code as encrypted KOs, build-from-KO workflow, HSM ceremony, eBPF/kernel modules.

### Reflexive Encrypted Runtime
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Source files stored as AES-256-GCM encrypted KOs | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §8 |
| 🔵 | Build system as Workflow (not external CI) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §8 |
| 🔵 | Bootstrap ceremony (HSM / TEE key management) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §8 |
| 🔵 | Plaintext decryption only on-demand in isolated build sandbox | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §8 |
| 🔵 | Internal artifact registry (sparse Cargo registry) | [SOURCE_CONTROL_STRATEGY](vision/SOURCE_CONTROL_STRATEGY.md) |

### Build System
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Toolchain KO store per target triple | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §2 |
| 🔵 | `ekosctl toolchain upload` command | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §2 |
| 🔵 | `BuildWorkflow` (fetch toolchain KO → compile → store artifact KO) | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §2 |
| 🔵 | musl static builds (`x86_64-unknown-linux-musl`) | [BUILD_SANDBOX_ADMIN](vision/BUILD_SANDBOX_ADMIN.md) §2 |

### Dependency Self-Upgrade — Class C.a + C.b
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Class C.a: eBPF programs via aya-rs (XDP, KProbe, scheduler hints) | [DEPENDENCY_SELF_UPGRADE](vision/DEPENDENCY_SELF_UPGRADE.md) §5a |
| 🔵 | eBPF loaded at runtime — no kernel recompile | [DEPENDENCY_SELF_UPGRADE](vision/DEPENDENCY_SELF_UPGRADE.md) §5a |
| 🔵 | Class C.b: kernel module loading (insmod/rmmod, no reboot) | [DEPENDENCY_SELF_UPGRADE](vision/DEPENDENCY_SELF_UPGRADE.md) §5b |

### Backup
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Air-gapped cold backup support | [BACKUP_STRATEGY](security/BACKUP_STRATEGY.md) §6 |
| 🔵 | Backup key shard ceremony (separate from HSM master key) | [BACKUP_STRATEGY](security/BACKUP_STRATEGY.md) §3 |

### Hardware Research
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | CXL 3.0 hardware evaluation (when available) | [HARDWARE_FRONTIER](vision/HARDWARE_FRONTIER.md) §1 |
| 🔵 | `ConsensusBackend` trait scaffolding (`RaftConsensus` + placeholder `CxlCasConsensus`) | [HARDWARE_FRONTIER](vision/HARDWARE_FRONTIER.md) §1 |

---

## Phase 7 — Cluster + Kernel Upgrade + Simulation 🔵

**Goal:** Full distributed mode, kexec kernel upgrades, all 6 simulation modes.

### Cluster Mode
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Raft consensus for distributed KoStore | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §8 |
| 🔵 | `FederationFrame` — multi-node EKOS coordination | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §8 |
| 🔵 | All ARP/CIP/Shadowbox operations cluster-aware | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §8 |

### Simulation Capabilities (all 6 modes)
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Policy Replay (historical patch decision simulation) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §6.3 |
| 🔵 | ARP Training Simulation (BugLibrary for synthetic failure injection) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §6.3 |
| 🔵 | Time-Shifted Traffic Replay (production traffic at 100x speed) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §6.3 |
| 🔵 | Bitemporal Forward Projection (capacity planning) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §6.3 |
| 🔵 | What-If Indexing (evaluate new index without creating it) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §6.3 |
| 🔵 | Chaos Confidence (failure injection → confidence measurement) | [AUTONOMOUS_RUNTIME_VISION](vision/AUTONOMOUS_RUNTIME_VISION.md) §6.3 |

### Dependency Self-Upgrade — Class C.c
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | kexec full kernel upgrade (no hardware reboot, ~10-30s downtime) | [DEPENDENCY_SELF_UPGRADE](vision/DEPENDENCY_SELF_UPGRADE.md) §5c |
| 🔵 | 72h kernel soak in dedicated kernel test VM | [DEPENDENCY_SELF_UPGRADE](vision/DEPENDENCY_SELF_UPGRADE.md) §5c |
| 🔵 | 7-day rollback window (old kernel stays in /boot) | [DEPENDENCY_SELF_UPGRADE](vision/DEPENDENCY_SELF_UPGRADE.md) §5c |

---

## Phase 7+ — Future / Research 🔵

**Goal:** Commercialization, live kernel patching, research items.

### Evolution Engine as a Service
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | On-prem appliance packaging (Tier 3 commercial) | [EVOLUTION_ENGINE_AS_A_SERVICE](vision/EVOLUTION_ENGINE_AS_A_SERVICE.md) §8 |
| 🔵 | Evidence SDK (read from Datadog/Prometheus/Splunk) | [EVOLUTION_ENGINE_AS_A_SERVICE](vision/EVOLUTION_ENGINE_AS_A_SERVICE.md) §3.1 |
| 🔵 | Multi-language synthesis plugins (Java, C#, Go, Python) | [EVOLUTION_ENGINE_AS_A_SERVICE](vision/EVOLUTION_ENGINE_AS_A_SERVICE.md) §7 |
| 🔵 | Evolution Knowledge Graph (pattern abstraction across customers) | [EVOLUTION_ENGINE_AS_A_SERVICE](vision/EVOLUTION_ENGINE_AS_A_SERVICE.md) §6 |
| 🔵 | Digital Twin (full production state mirror for behavioral validation) | [EVOLUTION_ENGINE_AS_A_SERVICE](vision/EVOLUTION_ENGINE_AS_A_SERVICE.md) §5 |
| 🔵 | Tier 1 open-source release (evidence correlation only) | [EVOLUTION_ENGINE_AS_A_SERVICE](vision/EVOLUTION_ENGINE_AS_A_SERVICE.md) §9 |

### Dependency Self-Upgrade — Class C.d
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | livepatch / kpatch (zero-downtime CVE kernel patching) | [DEPENDENCY_SELF_UPGRADE](vision/DEPENDENCY_SELF_UPGRADE.md) §5d |

### Research Items (no phase, needs production data to validate)
| Status | Item | Spec Doc |
|---|---|---|
| 🔵 | Double-write execution (live parallel old+new path validation) | [RESEARCH_FRONTIER](vision/RESEARCH_FRONTIER.md) §1 |
| 🔵 | Retroactive rollback (Priority 4 → re-execute after Priority 1) | [RESEARCH_FRONTIER](vision/RESEARCH_FRONTIER.md) §2 |
| 🔵 | Knowledge Compiler (unified pre-fetch + index advisor + compaction) | [RESEARCH_FRONTIER](vision/RESEARCH_FRONTIER.md) §4 |
| 🔵 | CXL-based distributed commit (CxlCasConsensus implementation) | [HARDWARE_FRONTIER](vision/HARDWARE_FRONTIER.md) §1 |
| 🔵 | FPGA semantic constraints (for HFT-tier deployments) | [HARDWARE_FRONTIER](vision/HARDWARE_FRONTIER.md) §2 |

---

## Crate Registry

Every crate mentioned, its phase, and its status:

| Crate | Workspace | Phase | Status |
|---|---|---|---|
| `vectorvolt-wal` | vectorvolt | 1 | ✅ Done |
| `vectorvolt-store` | vectorvolt | 1 | ✅ Done |
| `vectorvolt-index` | vectorvolt | 2 | ⬜ Pending |
| `vectorvolt-storage` | vectorvolt | 2 | ⬜ Pending |
| `vectorvolt-query` | vectorvolt | 2 | ⬜ Pending |
| `vectorvolt-vector` | vectorvolt | 2 | ⬜ Pending |
| `vectorvolt-server` | vectorvolt | 2 | ⬜ Pending |
| `vvctl` (binary) | vectorvolt | 2 | ⬜ Pending |
| `vectorvolt-graph` | vectorvolt | 3 | 🔵 Future |
| `vectorvolt-sdk-rust` | vectorvolt | 3 | 🔵 Future |
| `vectorvolt-sdk-dotnet` | vectorvolt | 3 | 🔵 Future |
| `vectorvolt-sdk-go` | vectorvolt | 4 | 🔵 Future |
| `vectorvolt-sdk-java` | vectorvolt | 4 | 🔵 Future |
| `vectorvolt-sdk-python` | vectorvolt | 4 | 🔵 Future |
| `ekos-kernel` | ekos | 1 | ✅ Done |
| `ekosd` | ekos | 1 | ✅ Done |
| `ekos-security` | ekos | 2 | ⬜ Pending |
| `ekos-embedding` | ekos | 3 | 🔵 Future |
| `ekos-cip` | ekos | 3 | 🔵 Future |
| `ekos-upgrade` | ekos | 4 | 🔵 Future |
| `ekos-workflow` | ekos | 4 | 🔵 Future |
| `ekos-admin` | ekos | 4 | 🔵 Future |
| `ekos-prefetch` | ekos | 4 | 🔵 Future |
| `ekos-backup` | ekos | 4 | 🔵 Future |
| `ekos-arp` | ekos | 5 | 🔵 Future |
| `ekos-scm` | ekos | 5 | 🔵 Future |
| `ekos-build` | ekos | 6 | 🔵 Future |

---

## Architecture Document Index

Every spec doc and what it covers:

| Document | Phase Coverage |
|---|---|
| [READ_PATH_ARCHITECTURE.md](vision/READ_PATH_ARCHITECTURE.md) | Phase 2–5: Nanosecond read algorithms |
| [PREDICTIVE_PREFETCH_ENGINE.md](vision/PREDICTIVE_PREFETCH_ENGINE.md) | Phase 2–5: Markov chain pre-fetch |
| [IN_PROCESS_LLM_EMBEDDING.md](vision/IN_PROCESS_LLM_EMBEDDING.md) | Phase 2–5: VECTOR type, LoRA fine-tuning |
| [STORAGE_ENGINE_ADVANCED.md](vision/STORAGE_ENGINE_ADVANCED.md) | Phase 2–5: QoS, fingerprints, checksums |
| [GRAPH_RELATIONAL_BRIDGE.md](vision/GRAPH_RELATIONAL_BRIDGE.md) | Phase 3–5: Property Graph over relational |
| [AUTONOMOUS_RUNTIME_VISION.md](vision/AUTONOMOUS_RUNTIME_VISION.md) | Phase 2–7: CIP, ARP, Shadowbox, simulation |
| [BUILD_SANDBOX_ADMIN.md](vision/BUILD_SANDBOX_ADMIN.md) | Phase 3–6: Sandbox, admin UI, build system |
| [DEPENDENCY_SELF_UPGRADE.md](vision/DEPENDENCY_SELF_UPGRADE.md) | Phase 4–7+: Class A/B/C dependency upgrade |
| [SOURCE_CONTROL_STRATEGY.md](vision/SOURCE_CONTROL_STRATEGY.md) | Phase 5–6: Internal SCM on KoStore |
| [EVOLUTION_ENGINE_AS_A_SERVICE.md](vision/EVOLUTION_ENGINE_AS_A_SERVICE.md) | Phase 7+: Commercial external offering |
| [HARDWARE_FRONTIER.md](vision/HARDWARE_FRONTIER.md) | Phase 3–7+: QAT/DSA/RDMA/CXL/FPGA |
| [RESEARCH_FRONTIER.md](vision/RESEARCH_FRONTIER.md) | Phase 7+: Research items (no schedule) |
| [MFA_SESSION_MANDATE.md](security/MFA_SESSION_MANDATE.md) | Phase 2: SSH MFA enforcement |
| [BACKUP_STRATEGY.md](security/BACKUP_STRATEGY.md) | Phase 3–6: 3-2-1 backup |

---

*This document is the single source of truth for what is built, what is next, and what is future.*  
*Update status column when items are completed. Add items here before adding them to any spec doc.*
