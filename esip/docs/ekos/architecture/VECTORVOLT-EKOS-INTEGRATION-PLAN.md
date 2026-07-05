# VectorVolt–EKOS Integration Plan
**Formal Phase-Gate Dependency Map | Created: 2026-07-03 | Status: ACTIVE**

> **How to use this document:**
> Before beginning any EKOS implementation phase, open this file. The gate table in §3
> tells you exactly which VectorVolt phases must be complete first. Before beginning any
> VectorVolt phase, check §4 to see which EKOS capabilities unlock as a result. This
> document is the authoritative sequencing contract between the two projects.
>
> **Source of truth for VectorVolt phases:** `C:\VectorVolt\IMPLEMENTATION_PLAN.md`
> **Source of truth for EKOS architecture:** `C:\VectorVolt\esip\docs\ekos\architecture\`
> **Storage provider interface spec:** `ADR-REGISTER.md` → ADR-052

---

## 1. Purpose

VectorVolt and EKOS are two distinct projects that will progressively integrate:

- **VectorVolt** is a zero-GC, kernel-bypass, sub-microsecond OLTP "Data OS" — written in
  Rust (engine core) with a C# client SDK. Its 8-phase implementation plan builds from
  silicon foundations up to a full distributed control plane.
- **EKOS** is an Enterprise Knowledge Operating System — a semantic knowledge graph with
  ambient AI, governed by the Knowledge Constitution. It requires a storage layer that is
  fast, durable, graph-capable, and portable across client infrastructure.

VectorVolt is the long-term storage engine for EKOS — but not from day one. The
integration is phased: EKOS launches with pluggable reference storage providers (ADR-052),
and VectorVolt providers replace them as each VectorVolt phase completes.

**What this plan is not:** it does not replicate VectorVolt's own implementation plan.
It maps VectorVolt's deliverables to EKOS's needs and defines the gates that govern
when EKOS can begin each integration phase.

---

## 2. VectorVolt Phase Summary (Quick Reference)

| VV Phase | Name | Key Deliverables Relevant to EKOS |
|---|---|---|
| **1** | Silicon Foundation | MemoryArena (zero-GC), AffinityScheduler (core pinning), PlatformDetector (AVX2/SIMD dispatch), OS jitter eradication |
| **2** | Single-Writer Engine | ShardExecutor (single-writer loop), RingBuffer (LMAX-style), ConsumerDagBuilder (pipeline), BackPressurePolicy, IdempotencyLog, YamlCommandCompiler |
| **3** | WAL & Durability | WalSegmentWriter (append-only, O_DIRECT), GroupCommitBatcher, DeterministicReplayer, SnapshotManager, **ProjectionDispatcher** (IProjectionTarget adapters for PostgreSQL + SQL Server), InterceptorPipeline, OfflineWalCompiler |
| **4** | Kernel Bypass Networking | XdpPreFilter (eBPF/XDP), KernelBypassListener (DPDK), RioListener (Windows RIO), ZeroCopyDeserializer (FlatBuffers), RdmaReplicator, PtpClockReader, INetworkTransport (cloud adapters) |
| **5** | Read Fabric & Vectorization | **DocumentStore** (by-ID, zero-parse), **ColumnarStore** (SIMD-friendly), **VectorScanner** (AVX2/AVX-512), QueryFilterEngine (Expression Trees), TieredRouter (hot/warm/cold), **MaterializedViewEngine** (continuous aggregates) |
| **6** | Control Plane | TenantManager, ConsistentHashRouter, EpochFencingManager, **CommandGateway** (polyglot SDKs: C#/Java/Python/Go/Rust), SchemaRegistry, AutoScaleManager, TscMetricsBuffer, WorkloadScheduler (QoS), VersionstampGenerator, vvctl CLI |
| **7** | Testing & Distribution | DeterministicSimulator (DST), ChaosTestRunner, MigrationToolkit (Gateway/eBPF shadow/CDC), platform packages (deb/rpm/msi/Docker/Helm), research documentation |
| **8** | Research Modules | LearnedIndexes, Unikernel, Wasm sandbox, probabilistic structs, write-verify, warm restart — **plus VectorVolt Graph Extension (planned, see §6)** |

---

## 3. EKOS Integration Gates (THE SEQUENCING CONTRACT)

### Gate 0 — Before EKOS implementation begins (no VectorVolt dependency)

EKOS architecture, requirements, and planning are VectorVolt-independent. All architecture
documents (A00–A12), requirements volumes (00–26), the Knowledge Constitution, ADR register,
and this integration plan are written and finalized before VectorVolt Phase 1 is complete.
EKOS implementation uses reference storage providers (ADR-052) until each gate opens.

### Gate 1 — MANDATORY: VectorVolt Phase 1 + 2 + 3 complete → EKOS Phase 1 begins

**All three phases must be fully complete, tested, and benchmarked before EKOS Phase 1
implementation starts. There are no exceptions to this gate.**

| VectorVolt Phase | Minimum Deliverable Required for EKOS Phase 1 |
|---|---|
| Phase 1 | MemoryArena with HugePage support; PlatformDetector (SIMD dispatch); AffinityScheduler; OS jitter configuration |
| Phase 2 | ShardExecutor; RingBuffer; BackPressurePolicy (Block mode); IdempotencyLog; YamlCommandCompiler; ConsumerDagBuilder |
| Phase 3 | WalSegmentWriter; GroupCommitBatcher; DeterministicReplayer; SnapshotManager; **ProjectionDispatcher** with PostgreSQLTarget + IProjectionTarget interface; InterceptorPipeline |

**Why Phase 1+2+3 together?** EKOS Phase 1 uses VectorVolt for two purposes: (a) the
Constitutional Telemetry Stream (CTS) requires an immutable, append-only, hash-chained WAL
(Phase 3's WalSegmentWriter); (b) the metering pipeline for the Knowledge Economy requires
GroupCommitBatcher for high-throughput metering event recording. Neither is possible without
the execution engine (Phase 2) or the memory foundation (Phase 1). Phase 3's
ProjectionDispatcher also provides the sync path from VectorVolt's WAL to PostgreSQL+AGE,
which is EKOS's graph store until Gate 3 opens.

**What EKOS Phase 1 gains from VectorVolt:**
- CTS writes go to VectorVolt WAL (immutable, hash-chained, tamper-evident) — fulfills
  EKOS-22-046 (immutable append-only CTS).
- Metering events (Knowledge Economy, Vol 13) are written via GroupCommitBatcher — high
  throughput without per-event fsync latency.
- ProjectionDispatcher syncs VectorVolt WAL → PostgreSQL+AGE, so the Knowledge Kernel graph
  store stays current.
- InterceptorPipeline provides the CCR enforcement hook before events reach storage —
  Law 4, 6, 11, 16 checks run in the interceptor chain.
- EKOS still uses reference providers for all other storage (PostgreSQL+AGE for graph,
  Redis for document cache, ClickHouse for columnar analytics).

### Gate 2 — VectorVolt Phase 4 complete → EKOS distributed networking unlocked

| VectorVolt Deliverable | EKOS Capability Unlocked |
|---|---|
| KernelBypassListener (DPDK) | Multi-kernel EKOS topology (A15) — Knowledge.Open() across nodes at kernel-bypass latency |
| RioListener (Windows RIO) | Windows-native EKOS node support with kernel-bypass networking |
| ZeroCopyDeserializer (FlatBuffers) | Knowledge syscall wire protocol goes zero-copy — no GC on hot path |
| RdmaReplicator | WAL replication between EKOS nodes via RDMA (sub-microsecond replication lag) |
| PtpClockReader | Bitemporal transaction-time stamps use hardware clock (PTP IEEE 1588) — <100ns drift |
| INetworkTransport (EFA/InfiniBand/gVNIC) | AWS/Azure/GCP cloud deployments use provider-native high-speed networking |

Gate 2 is **not a hard prerequisite for EKOS Phase 1** — EKOS Phase 1 runs on standard
networking (INetworkTransport falls back to StandardSocketTransport). Gate 2 unlocks
the multi-kernel / edge-deployment architecture described in A15.

### Gate 3 — VectorVolt Phase 5 complete → EKOS Phase 2 begins (Read Fabric integration)

**VectorVolt Phase 5 must be fully complete before EKOS Phase 2 starts.**

| VectorVolt Deliverable | EKOS Provider Fulfilled | EKOS Capability |
|---|---|---|
| DocumentStore | `VectorVoltDocumentProvider` replaces `RedisProvider` as `IDocumentStoreProvider` | CCR capability cache — per-worker capability lookup at nanosecond latency (previously ~1ms Redis RTT, now ~10ns VectorVolt lookup) |
| ColumnarStore + VectorScanner (AVX2) | `VectorVoltColumnarProvider` replaces `ClickHouseProvider` as `IColumnarStoreProvider` | Knowledge.Query() with ABAC filters — SIMD-accelerated scan over millions of KOs; 8 values compared per CPU cycle |
| QueryFilterEngine (Expression Trees) | Part of `VectorVoltColumnarProvider` | ABAC predicate compiled to delegate — zero-alloc filter evaluation on every KO scan |
| TieredRouter (hot/warm/cold) | Embedded in both VectorVolt providers | EKOS knowledge access pattern (active projects = hot, historical = warm, archived = cold) maps directly — no separate tiering logic needed |
| MaterializedViewEngine | Part of `VectorVoltColumnarProvider` | Widget dashboards (Vol 02/14) — pre-computed aggregates update atomically with each Knowledge mutation; no batch scan on widget render |

**What VectorVolt Phase 5 does NOT yet provide:**
- Graph traversal (e.g., "traverse 4 hops from ADR-012 through all connected decisions").
  `IGraphStoreProvider` is still fulfilled by `PostgreSQLAGEProvider`. Graph stays with
  PostgreSQL+AGE until Gate 4.
- Full multi-tenant isolation at storage level — TenantManager comes in Phase 6 (Gate 4).

**EKOS Phase 2 architecture:** VectorVolt Phase 5 sits in front of PostgreSQL+AGE as the
read acceleration layer. Hot-path reads (CCR, widget rendering, KO scans) go to VectorVolt.
Graph traversals go to PostgreSQL+AGE. ProjectionDispatcher (Gate 1) keeps VectorVolt's
read store current by projecting WAL → VectorVolt stores in near-real-time.

### Gate 4 — VectorVolt Phase 6 complete → EKOS multi-tenancy + polyglot SDKs

| VectorVolt Deliverable | EKOS Capability |
|---|---|
| TenantManager | EKOS multi-tenant isolation at storage level — per-tenant VectorVolt shards; VectorVolt enforces tenant boundaries independent of EKOS application logic |
| ConsistentHashRouter | EKOS shard routing — `Knowledge.Open(koId)` routes to the correct VectorVolt shard transparently |
| CommandGateway (polyglot SDKs) | EKOS SDK (Vol 23) uses VectorVolt's gRPC/protobuf SDK as the transport layer — all languages (C#, Java, Python, Go, Rust) get EKOS access via VectorVolt's generated SDKs |
| SchemaRegistry | EKOS Knowledge Object schema versioning enforced at VectorVolt level — incompatible schema changes rejected before they reach storage |
| TscMetricsBuffer + ShadowMetricsExporter | VectorVolt's zero-overhead telemetry feeds EKOS observability — CTS latency metrics, CCR check timing, projection lag all visible in Prometheus/Grafana |
| WorkloadScheduler (QoS tiers) | EKOS interactive syscalls (Knowledge.Open/Query) vs batch analytics — different QoS tiers; interactive-tier latency protected even under batch-scan load |
| VersionstampGenerator | Cross-shard bitemporal ordering — global versionstamps enable correct bitemporal ordering of KO mutations across shards |
| vvctl CLI + TUI Dashboard | EKOS operators can inspect VectorVolt from the terminal — shard health, WAL throughput, projection lag, per-KO access patterns |

### Gate 5 — VectorVolt Phase 7 complete → EKOS production GA readiness

**VectorVolt must pass Phase 7 validation before EKOS goes to production with VectorVolt
as a storage provider.**

| Phase 7 Deliverable | EKOS Requirement |
|---|---|
| DeterministicSimulator (DST) — 10M failure scenarios, zero data loss | VectorVolt is a load-bearing storage provider for EKOS; unproven failure behavior = unacceptable production risk |
| ChaosTestRunner (power loss, corruption, partition, split-brain) | EKOS Law 11 (non-destructive) requires guaranteed durability; chaos validation proves it |
| Platform packages (deb/rpm/msi/Docker/Helm) | EKOS on-prem deployments include VectorVolt packages; they must be production-grade single-command installs |
| Research documentation (benchmarks vs LMAX/TigerBeetle/FoundationDB) | EKOS marketing and PhD materials reference VectorVolt's validated performance numbers |

### Gate 6 — VectorVolt Graph Extension complete → EKOS Phase 3 (Full Unification)

This gate opens when VectorVolt adds native graph primitives — property graph model,
edge traversal, Cypher/GQL query support. This is currently a **planned research extension**
in VectorVolt Phase 8 (see §6). When complete:

| Provider Transition | From | To |
|---|---|---|
| `IGraphStoreProvider` | `PostgreSQLAGEProvider` | `VectorVoltGraphProvider` |
| `IDocumentStoreProvider` | `VectorVoltDocumentProvider` | (already VectorVolt — no change) |
| `IColumnarStoreProvider` | `VectorVoltColumnarProvider` | (already VectorVolt — no change) |
| `IRelationalStoreProvider` | `PostgreSQLProvider` | Kept (VectorVolt is not a relational DB; Postgres stays for EF Core projections) |

After Gate 6: VectorVolt is the single storage engine for graph, document, and columnar
tiers. PostgreSQL remains for relational projections only. Knowledge graph traversals
(bitemporal, 4-hop, ABAC-filtered) run at VectorVolt's kernel-bypass latency — no
PostgreSQL round-trip.

---

## 4. Integration Phase Summary

| EKOS Integration Phase | Prerequisite VectorVolt Gates | What VectorVolt Provides | What Stays with Reference Providers |
|---|---|---|---|
| **Phase 1** — CTS + Metering WAL | **Gates 1 (mandatory)** | WAL (CTS), GroupCommitBatcher (metering), ProjectionDispatcher (graph sync), InterceptorPipeline (CCR hook) | Graph: PostgreSQL+AGE · Document: Redis · Columnar: ClickHouse |
| **Phase 2** — Read Fabric | **Gates 1+3** (Gate 2 optional, recommended) | DocumentStore (CCR cache), VectorScanner (KO scans), MaterializedViewEngine (widgets), TieredRouter | Graph: PostgreSQL+AGE · Relational: PostgreSQL |
| **Phase 3** — Multi-Tenant + SDK | **Gates 1+3+4** | TenantManager (isolation), ConsistentHashRouter, Polyglot SDKs (transport), SchemaRegistry, QoS, Versionstamps | Graph: PostgreSQL+AGE · Relational: PostgreSQL |
| **Phase 4** — Full Unification | **All Gates (1–6)** | All four storage provider interfaces (except relational) | Relational: PostgreSQL (permanent — EF Core projections) |

---

## 5. VectorVolt Build Order for EKOS Teams

When EKOS implementation is ready to begin (Gate 0 closed), initiate VectorVolt development
in strict phase order. Do not skip phases; each phase's hardware abstractions are the
foundation for the next.

```
START VectorVolt Phase 1 (Silicon Foundation)
  → MemoryArena, AffinityScheduler, PlatformDetector, jitter config
  → All Phase 1 benchmarks documented and passing

START VectorVolt Phase 2 (Single-Writer Engine)
  → ShardExecutor, RingBuffer, ConsumerDagBuilder, BackPressurePolicy
  → IdempotencyLog, YamlCommandCompiler
  → All Phase 2 tests passing

START VectorVolt Phase 3 (WAL & Durability)
  → WalSegmentWriter, GroupCommitBatcher, DeterministicReplayer
  → ProjectionDispatcher + PostgreSQLTarget + IProjectionTarget
  → InterceptorPipeline + IProjectionInterceptor
  → All Phase 3 tests passing, recovery verified

══════════════════════════════════════
GATE 1 OPENS → BEGIN EKOS PHASE 1
══════════════════════════════════════

VectorVolt Phase 4 (Kernel Bypass Networking) — run in parallel with EKOS Phase 1
  → XdpPreFilter, KernelBypassListener, RdmaReplicator, PtpClockReader
  → INetworkTransport cloud adapters
  [Gate 2 opens → EKOS A15 multi-kernel architecture unlocked]

VectorVolt Phase 5 (Read Fabric & Vectorization)
  → DocumentStore, ColumnarStore, VectorScanner (AVX2)
  → QueryFilterEngine, TieredRouter, MaterializedViewEngine
  → All benchmarks vs scalar baseline documented

══════════════════════════════════════
GATE 3 OPENS → BEGIN EKOS PHASE 2
══════════════════════════════════════

VectorVolt Phase 6 (Control Plane)
  → TenantManager, ConsistentHashRouter, EpochFencingManager
  → CommandGateway, Polyglot SDKs (C#/Java/Python/Go/Rust)
  → SchemaRegistry, AutoScaleManager, WorkloadScheduler
  → TscMetricsBuffer, VersionstampGenerator, vvctl CLI

══════════════════════════════════════
GATE 4 OPENS → EKOS MULTI-TENANT + SDK
══════════════════════════════════════

VectorVolt Phase 7 (Testing & Distribution)
  → DeterministicSimulator (10M failure scenarios)
  → ChaosTestRunner, platform packages (deb/rpm/msi/Docker/Helm)
  → Research documentation

══════════════════════════════════════
GATE 5 OPENS → EKOS PRODUCTION GA
══════════════════════════════════════

VectorVolt Phase 8 + Graph Extension (Research)
  → Graph primitives (property graph model, Cypher/GQL)
  → Bitemporal edge support
  → VectorVoltGraphProvider implements IGraphStoreProvider

══════════════════════════════════════
GATE 6 OPENS → EKOS FULL UNIFICATION
══════════════════════════════════════
```

---

## 6. The Graph Primitive Gap (VectorVolt Phase 8 Extension)

**Current state:** VectorVolt's 8-phase implementation plan (v3.0, 2026-04-05) does not
include a native graph engine. Phases 1–7 cover memory, execution, WAL, networking, read
fabric, control plane, and testing. Phase 8 research modules cover learned indexes, P4,
Unikernel, Wasm sandbox, TTL, probabilistic structures, watches, write-verify, and warm
restart — but not a property graph engine.

**What EKOS needs from a graph engine:**
- Property graph model (nodes with typed labels + typed edges with properties)
- Bitemporal edges — each edge carries valid-time and transaction-time intervals
- Cypher/GQL traversal — multi-hop path queries, pattern matching
- ABAC-aware traversal — visibility filter applied per-hop (Law 4, Vol 04)
- Sub-millisecond P99 for common traversals (1–3 hops)

**Plan:** VectorVolt Phase 8 will be extended with a **Graph Extension** module. This is
a new research deliverable, not yet scoped in detail. When scoping begins:
1. Read this section first.
2. Read Vol 01 (Semantic Knowledge Graph — the full ontology and traversal requirements).
3. Read A04 (Data, Storage & Ontology Architecture — bitemporal storage strategy).
4. Design the graph primitive to satisfy the bitemporal edge model (OQ-A03-C / OQ-01-B).
5. Implement as a Phase 8 module: `VectorVoltGraph` (property graph on top of VectorVolt's
   MemoryArena + WAL + Read Fabric — bitemporal natively, no separate temporal layer needed).

Until this extension is complete, PostgreSQL+AGE (via `PostgreSQLAGEProvider`) handles all
graph traversals. This is not a blocker for EKOS Phases 1–3; it is only required for Gate 6.

---

## 7. Reference: EKOS Syscalls by Storage Provider

The table below maps EKOS Knowledge OS syscalls (Vol 22) to the storage provider they hit
at each integration phase. Use this when debugging or tracing a specific syscall's storage
path.

| Syscall | Phase 1 (reference providers) | Phase 2 (VV Phase 5) | Phase 3 (VV Phase 6) | Phase 4 (VV graph) |
|---|---|---|---|---|
| `Knowledge.Open(koId)` | Redis (by-ID lookup) | **VectorVolt DocumentStore** | VV + TenantRouter | VV (same) |
| `Knowledge.Query(filter)` | PostgreSQL+AGE (graph scan) | **VV VectorScanner** (columnar) + AGE (graph hops) | VV + shard-aware | VV (graph native) |
| `Knowledge.Write(mutation)` | VectorVolt WAL → projected to AGE | VV WAL → VV stores + AGE | VV WAL + TenantIsolation | VV WAL (no AGE) |
| `Knowledge.Project(koId, target)` | VV ProjectionDispatcher → AGE | VV → AGE (graph) + VV stores | VV → VV (all tiers) | VV (all tiers) |
| `Knowledge.Publish(koId)` | VV WAL (CTS) + AGE write | VV WAL + VV document | VV (all) | VV (all) |
| `CCR.CheckCapability(workerId)` | Redis (cache) | **VV DocumentStore** (~10ns) | VV + tenant-scoped | VV (same) |
| `CTS.Emit(event)` | **VectorVolt WAL** (from Phase 1) | VV WAL (same) | VV WAL (same) | VV WAL (same) |
| `Meter.Record(event)` | **VV GroupCommitBatcher** (Phase 1) | VV (same) | VV (same) | VV (same) |
| `Widget.Render(widgetId)` | ClickHouse (aggregates) | **VV MaterializedViewEngine** | VV + QoS tier | VV (same) |

---

## 8. Decision Log

| Decision | Date | Rationale |
|---|---|---|
| VectorVolt Rust pivot (engine core) | 2026-06-23 | Zero-GC guarantees require systems language; C# SDK remains for client tooling |
| VectorVolt Phases 1+2+3 mandatory before EKOS Phase 1 | 2026-07-03 | CTS WAL + metering require WAL writer + execution engine + memory foundation |
| Phase 4 (networking) not a hard prerequisite for EKOS Phase 1 | 2026-07-03 | StandardSocketTransport fallback sufficient; kernel bypass is a performance upgrade, not a functional gate |
| PostgreSQL+AGE as `IGraphStoreProvider` until Gate 6 | 2026-07-03 | VectorVolt has no graph primitives in Phases 1–7; AGE is the sustainable, Apache-licensed bridge |
| PostgreSQL stays for relational tier permanently (not replaced by VectorVolt) | 2026-07-03 | VectorVolt is not a relational DB; EF Core projection targets require SQL |
| VectorVolt graph extension scoped as Phase 8 addition | 2026-07-03 | Not in original Phase 8 plan; must be formally scoped when Phase 7 completes |

---

## 9. Document Maintenance

This document must be updated when:
- VectorVolt's `IMPLEMENTATION_PLAN.md` adds, removes, or resequences phases
- A gate condition is satisfied (flip to ✅ in §3 and record the date)
- A new EKOS requirement creates a new dependency on VectorVolt
- The VectorVolt graph extension is scoped (update §6 with the design)

**Gates status (update here as each opens):**

| Gate | Condition | Status | Date Opened |
|---|---|---|---|
| Gate 1 | VV Phase 1+2+3 complete | ⬜ Not yet | — |
| Gate 2 | VV Phase 4 complete | ⬜ Not yet | — |
| Gate 3 | VV Phase 5 complete | ⬜ Not yet | — |
| Gate 4 | VV Phase 6 complete | ⬜ Not yet | — |
| Gate 5 | VV Phase 7 complete | ⬜ Not yet | — |
| Gate 6 | VV Graph Extension complete | ⬜ Not yet | — |

---

_Created 2026-07-03. The authoritative phase-gate contract between VectorVolt and EKOS.
Read this before starting any EKOS or VectorVolt implementation phase._
