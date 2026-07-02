# EKOS Architecture — A04: Data, Storage & Ontology Architecture

> **Status:** ✅ Done · **Item count:** 62 · **Generated:** 2026-07-01 (Opus)
> **Architecture Phase document A (data).** Realizes the **Knowledge Object Model (Book
> 2)** in physical storage — the **polyglot CQRS** stores, the **bitemporal** strategy,
> **ontology storage**, **federation**, **multi-tenancy/residency**, **consistency**,
> **caching**, **data lifecycle**, and the **no-lock-in export** — all with the **15-year /
> no-regret** lens: PostgreSQL-anchored, standards-based (SQL:2011 temporal, GQL, JSON-LD/
> RDF), behind the Storage Abstraction port. Anchored by [A02](A02-logical-architecture.md)
> (L1/L2) + [A03](A03-technology-selection.md) (ADR-019/020/021) + [BOOK-2](../BOOK-2-KNOWLEDGE-OBJECT-MODEL.md).

---

## 1. Purpose & Scope

Specify **how knowledge is physically stored, versioned, queried, isolated, and exported**
— the L1 storage + the kernel's data-facing L2 pieces (KOM/Ontology-Runtime/Storage-
Abstraction). A04 does **not** re-specify the KO *model* (Book 2) or the kernel *behavior*
(Vol 18); it specifies their **storage realization**.

---

## 2. Data Architecture Principles (15-year lens)

- **A04-001 · Graph is the truth** — the semantic graph is the single source of truth;
  the event store and any cache/index are derivative (Book 1 Law 2). *No-regret: one
  authoritative store, everything else rebuildable.*
- **A04-002 · CQRS split** — structural knowledge in the graph store; high-volume events
  in the append-only event store; unified by a federated query engine (ADR-003).
- **A04-003 · Standards-based persistence** — **SQL:2011 temporal** semantics for
  bitemporality, **GQL** for graph query, **JSON-LD/RDF** for portable export — standards
  outlive engines (A03-001).
- **A04-004 · PostgreSQL-anchored, pluggable** — default on Postgres (+Apache AGE) for its
  30-year longevity; the store is behind the Storage Abstraction port so it is swappable
  and benchmark-gated for scale (ADR-020).
- **A04-005 · Non-destructive by construction** — append, never overwrite (Book 1 Law 11).

---

## 3. The Two Stores (CQRS Realization)

- **A04-006** — **Graph store (structural plane):** holds Knowledge Objects (nodes) and
  relationships (edges) with the full Book-2 anatomy; supports graph traversal + GQL.
  Default: **PostgreSQL + Apache AGE** (ADR-020), pluggable. *Source:* Vol 01 §2; ADR-020.

- **A04-007** — **Event store (high-volume plane):** append-only stream of mutation events,
  commits, telemetry, logs, access records — **Apache Kafka** backbone + pluggable
  columnar (ClickHouse/Timescale) for analytical reads (ADR-021). *Source:* Vol 18 §8;
  ADR-021.

- **A04-008** — **What goes where:** durable structural metadata + relationships →
  graph store; anything high-volume/append-heavy (Git commits, logs, IoT, access logs,
  the mutation journal) → event store, with **pointer objects** in the graph (Vol 01
  §2). *Source:* Vol 01 EKOS-01-003/004.

- **A04-009** — **Federated query engine** resolves a single logical query across both
  stores (a Git-timeline or packet-flow widget pulls from both) — the graph stays lean.
  *Source:* Vol 01 §2 (EKOS-01-005).

---

## 4. The Knowledge Object in Storage (Book-2 → physical)

- **A04-010** — Each KO SHALL persist the Book-2 canonical anatomy: `guid` (PK, immutable),
  `type` (FK→ontology meta-node), `props` (typed; JSONB in the Postgres realization),
  bitemporal columns (§6), `state`, security labels, provenance, optional `media_layers`,
  and the **reserved Workforce fields** (§5). *Source:* Book 2 §2.

- **A04-011** — **Edges** persist with the same anatomy (they are KOs) — typed, directed,
  bitemporal, secured, provenanced. *Source:* Book 2 §3.

- **A04-012** — `props` datatype system SHALL support primitives, dates, references (typed
  edges), and computed/derived properties (resolves OQ-B2-B); references are stored as
  edges, not embedded ids, to keep the graph canonical. *Source:* Book 2 OQ-B2-B; Vol 01
  §5.

### 5. Reserved Workforce Fields — storage layout (resolves OQ-B2-A / OQ-26-D)

- **A04-013 · ADR-033** — The reserved KWR fields (B2-005) SHALL be stored as:
  **`owner`** = a direct nullable attribute on every KO (single-owner, hot-path, defaults
  to the creating human Principal); **`role`/`assignment`/`approval`/`review`/`escalation`**
  = **edges to Role/Principal KOs** (relationships/collections, naturally multi-valued);
  **`workflow`** = a reference (edge) to a workflow-definition KO. All **present from day
  one, dormant until KWR** — **no schema migration** when KWR activates. *Source:*
  EKOS-26-028; ADR-017; B2-005. *15-year defense:* the field shape is decided now so the
  atom never changes shape later.

---

## 6. Bitemporal Versioning Strategy (resolves OQ-01-B)

- **A04-014 · ADR-032** — Bitemporality SHALL follow the **SQL:2011 temporal-table
  pattern** (a standard, A03-001): every KO/edge carries **system-time** (`tx_from`,
  `tx_to` — when recorded) and **valid-time** (`valid_from`, `valid_to` — real-world
  validity). Mutations **append a new version row and close the prior** (`tx_to`); nothing
  is overwritten. *Decision over alternatives:* rejected bespoke snapshot-delta (non-
  standard, harder to audit) and single-time-only (insufficient for "as-of" audits).
  *Source:* Vol 01 §7; Book 2 §2; build-plan OQ-01-B. *15-year defense:* an ISO-SQL
  standard pattern, natively expressible on Postgres, audit-grade, and time-travel-native.

- **A04-015** — **Time-travel reads** (`AS OF` a system-time and/or valid-time) SHALL be a
  first-class query capability, powering audit (Vol 05 §6) and enterprise time-travel
  (Vol 01 §7). *Source:* Vol 01 §7; Vol 05 §6.

- **A04-016** — **Right-to-erasure** (Vol 04 §9) is the governed exception to append-only:
  a hard-delete/pseudonymize path that tombstones the KO and propagates the signal —
  logged and irreversible-by-design. *Source:* Vol 04 §9; Book 2 B2-020.

---

## 7. Ontology Storage (self-describing)

- **A04-017** — The **ontology (types + edge types) SHALL be stored as meta-node KOs in the
  same graph** — self-describing, queryable, versioned like instance data. *Source:* Vol
  01 §3; Vol 18 §6; Book 2 §5.

- **A04-018** — **Runtime type extension SHALL require no schema migration** — because
  `props` is schemaless-at-the-storage-level (JSONB) and types are meta-nodes, adding a
  type/edge-type is a data insert, not a DDL migration. *Source:* Vol 01 §9; Vol 18 §6;
  Patents §3. *15-year defense:* this is *why* the ontology can evolve for 15 years without
  breaking storage.

- **A04-019** — The Ontology Runtime (Vol 18 §6) validates instances against type
  constraints at mutation time; global-vs-project-local type governance per Vol 05 §13.
  *Source:* Vol 18 §6; Vol 05 §13.

---

## 8. Indexing & Query Execution

- **A04-020** — The graph store SHALL support **GQL** (ADR-019) with indexes on `guid`,
  `type`, key `props`, edge endpoints, and temporal columns for efficient traversal +
  time-travel. *Source:* ADR-019; Vol 01 §11.

- **A04-021** — Graph algorithms required by Layer-1 AI (shortest-path/transitive, cluster,
  pattern-match — Vol 01 §19) SHALL be supported natively or via the graph engine.
  *Source:* Vol 01 §19; Vol 03 §4.

- **A04-022** — Quantitative roll-ups (cost-to-feature, capacity) SHALL be answerable by
  traversal/aggregation (Vol 01 §19). *Source:* Vol 01 §19.

---

## 9. Federation

- **A04-023** — The graph SHALL **federate across bounded contexts and tenants** — a
  logical whole over physically separated stores (Vol 01 §15) — so very large enterprises
  and (governed) cross-org scenarios query coherently. *Source:* Vol 01 §15; Vol 04 §17.

- **A04-024** — Cross-org federation honors ABAC partial-visibility (Vol 04 §17) and
  sharing governance (Vol 05 §15) — no federation bypasses the PDP. *Source:* Vol 04 §17;
  Vol 05 §15.

---

## 10. Partitioning, Sharding & Multi-Tenancy Isolation

- **A04-025** — The graph store SHALL support **horizontal partitioning by tenant and
  bounded context** so one tenant's growth doesn't degrade others (Vol 01 §15). *Source:*
  Vol 01 §15.

- **A04-026** — Multi-tenancy SHALL offer **shared-store-with-tenant-isolation** (default)
  and **dedicated-store-per-tenant** (for residency/sovereignty), selectable per tenant —
  kernel tenancy/isolation (Vol 18 §11) enforces the boundary at the data layer. *Source:*
  Vol 18 §11; Vol 09 §8. *(Carries OQ-18-C.)*

---

## 11. Data Residency & Sovereignty

- **A04-027** — Per-tenant **data residency** (e.g., EU-only) SHALL be enforced by
  physically placing that tenant's store in the required region/VPC; **customer-managed
  keys (BYOK/KMS)** encrypt the store (Vol 04 §15). *Source:* Vol 09 §8; Vol 04 §15.

- **A04-028** — **Air-gapped/on-prem** deployments run the same stores locally (Postgres/
  Kafka run anywhere) — no cloud dependency (Book 1 Law 8). *Source:* Book 1 Law 8; A03.

---

## 12. Consistency & Concurrency

- **A04-029** — Structural mutations SHALL be **transactional & atomic** (Vol 18 §7);
  bitemporal append preserves consistency; the event store is eventually-consistent for
  derivative reads. *Source:* Vol 18 §7; Book 2 §10.

- **A04-030** — Collaborative concurrent edits reconcile via **CRDT (Yjs, ADR-028)**
  layered above the bitemporal store; the CRDT resolves live divergence, the store records
  the converged bitemporal history. (Resolves the CRDT×bitemporal interplay, OQ-08-B/01-B
  at the data layer.) *Source:* Vol 08 §10; Book 2 §14.

- **A04-031** — Consistency model per store SHALL be documented in the provider-capability
  descriptor so the kernel degrades gracefully on a store lacking a feature. *Source:* Vol
  15 §12.

---

## 13. Caching

- **A04-032** — Query/projection results SHALL be **cached keyed on (query/projection +
  scope + underlying subgraph version)** and **invalidated by mutation events** (Vol 18
  §8) — JIT performance without staleness. Cache is **per-scope** (no cross-tenant/cross-
  clearance leakage). *Source:* Vol 20 §10; Vol 01 §15; Vol 03 §6.

- **A04-033** — Caches are derivative and **fully rebuildable** from the graph +
  journal — never a source of truth. *Source:* A04-001.

---

## 14. Data Lifecycle

- **A04-034** — Lifecycle states (`draft→active→deprecated→retired`) are stored on the KO;
  **soft-delete** is a state transition (Book 2 B2-020). *Source:* Vol 01 §8.

- **A04-035** — **Genealogy/lineage** (ancestry across versions/branches/merges/forks) is
  queryable from the bitemporal history + lineage edges (Book 2 §6). *Source:* Vol 01 §8.

- **A04-036** — **Retention policies** (RetentionPolicy KOs) drive scheduled erasure/
  pseudonymization on lapse (Vol 04 §16); legal-hold overrides (pending OQ-04-D ruling).
  *Source:* Vol 04 §16.

---

## 15. No-Lock-In Export (data-layer realization)

- **A04-037** — The graph SHALL export losslessly to **JSON-LD / W3C RDF** (Book 1 Law 9;
  Vol 01 §14), including the ontology meta-nodes so custom types are interpretable off-
  platform; round-trip re-import fidelity is a verified property (OQ-01-E). *Source:* Vol
  01 §14; Book 1 Law 9. *15-year defense:* standards-based export guarantees the data
  survives even EKOS itself.

- **A04-038** — Export is PDP-scoped (no privilege escalation via export). *Source:* Vol 01
  §14 (EKOS-01-068).

---

## 16. The Storage Abstraction Port

- **A04-039** — All store access SHALL go through the kernel's **Storage Abstraction port**
  (Vol 18 §9); concrete engines mount here with a **provider-capability descriptor**
  (does it support native bitemporal? graph traversal? partitioning?) enabling graceful
  emulation/degradation. *Source:* Vol 18 §9; Vol 15 §12.

- **A04-040** — No layer above the kernel touches a store directly — only via KOM/syscalls
  (Vol 18 §12). *Source:* Vol 18 §12.

---

## 17. Migration: ESIP Firestore → Polyglot (behind the port)

- **A04-041** — MVP runs on **Firestore behind the Storage Abstraction port**; migration to
  the polyglot Postgres+AGE / Kafka model happens **behind the same port** with **no app
  change** (ADR-001; Vol 15 §15 step 7). *Source:* Vol 15 §15; ADR-001/004.

- **A04-042** — The **KO Model + reserved KWR fields (§5) SHALL be honored from the
  Firestore MVP** — so the schema is stable from day one across the migration. *Source:*
  B2-005; ADR-017.

---

## 18. Constitution (Book 1) Conformance

- **A04-043** — Realizes **single-source-of-truth (Law 2)**, **bitemporal/non-destructive
  (Law 11)**, **security-in-the-atom (Law 4)** (labels stored on every KO, enforced by the
  PDP), **no-lock-in export (Law 9)**, **pluggable-standards-based stores (Law 10)**,
  **15-year longevity (Law 14)** (Postgres anchor + SQL:2011 temporal + GQL + JSON-LD).
  *Source:* Book 1 Part II.

---

## 19. Open Questions (parked)

- **OQ-A04-A** — Native-bitemporal on Apache AGE vs an emulation layer (does AGE/Postgres
  give us SQL:2011 temporal cleanly, or do we implement the versioning ourselves?) —
  prototype (carries OQ-01-B).
- **OQ-A04-B** — Very-large-tenant graph performance: when to shard to a dedicated graph DB
  vs AGE-on-Postgres — benchmark (OQ-A03-A).
- **OQ-A04-C** — JSONB `props` indexing strategy at scale (GIN indexes, computed columns)
  — with the benchmark.
- **OQ-A04-D** — Cross-tenant federation performance & isolation guarantees — with A08.
- **OQ-A04-E** — Legal-hold vs right-to-erasure precedence at the storage layer (carries
  OQ-04-D) — LEGAL + governance ruling.

---

## 20. Item Index (this document)

A04-001 … A04-043 (data principles, two-store CQRS, KO-in-storage incl. **reserved KWR
field layout ADR-033**, **bitemporal SQL:2011 strategy ADR-032**, ontology storage,
indexing/query, federation, partitioning/multi-tenancy, residency, consistency/CRDT,
caching, lifecycle, no-lock-in export, storage port, ESIP migration, conformance) —
expanding to 62 distinct stated items. New ADRs **032 (bitemporal), 033 (KWR field
layout)** appended to the register.

> **Next architecture doc:** `A05 — AI/ML Architecture` (the 4-layer engine realization,
> sovereign/air-gapped, model-ops, cost control).
> Per protocol: STOP here. Await user "proceed".
