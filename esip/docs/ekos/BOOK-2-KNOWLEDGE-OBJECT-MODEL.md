# EKOS Book 2 — The Knowledge Object Model & Computational Model

> **Book 2. The canonical specification of EKOS's atom and how it computes.** Where
> Book 1 carries the *laws*, Book 2 carries the *object model* — the equivalent of
> Linux's `process/thread/file/socket/memory`, the **CLR Object Model**, or **LLVM IR**.
> Every subsystem references it; everything inherits from it.
> **Created:** 2026-07-01 (Opus). Gap-closure (2nd cross-architect review, concerns #1 +
> #2). **Formalization of the existing model — NOT a new pillar** (Feature Freeze /
> ADR-018). Consolidates the KO anatomy scattered across **Vol 01 §6/§7/§8, Vol 18 §3,
> Vol 26 §11** into one authoritative spec, and makes the **computational model**
> explicit. Subordinate to [BOOK-1](BOOK-1-KNOWLEDGE-CONSTITUTION.md); self-hosted per
> Vol 25.

---

# PART A — The Knowledge Object Model

## 1. The Universal Atom

- **B2-001** — The **Knowledge Object (KO)** SHALL be the **single, universal atom** of
  EKOS. **Everything is a Knowledge Object**: requirements, decisions, code, costs,
  risks, widgets, projections, policies, applications, workforce roles, AI insights,
  media, the ontology's own types, and the kernel's own configuration. *Source:* Book 1
  Law 2; Vol 01; Vol 25 §3.

- **B2-002** — There SHALL be exactly **one canonical KO specification** (this Book) from
  which all subsystems and object kinds derive — no subsystem invents a parallel object
  model. *Source:* 2nd review #1 ("one authoritative Knowledge Object Specification").

- **B2-003** — Both **nodes and edges (relationships) are Knowledge Objects** — an edge
  has the same anatomy (§3) as a node, is typed, versioned, secured, and traceable.
  *Source:* Vol 01 §5/§6 (edges carry the common attribute set).

---

## 2. Anatomy of a Knowledge Object (Canonical Fields)

- **B2-004** — Every KO SHALL have these canonical components (the inheritance root):

| # | Component | Specification |
|---|---|---|
| **Identity** | `guid` | Globally unique, immutable, assigned at allocation (Vol 18 §3). |
| **Type** | `type` | Reference to an ontology meta-node (which is itself a KO, §5). |
| **Properties** | `props{}` | Open, typed key-value map; datatypes per the type's schema. |
| **Temporal** | `valid_from, valid_until, tx_time` | **Bitemporal**: valid-time (real-world) + transaction-time (recorded). Never overwritten; appended (Vol 01 §7). |
| **Lifecycle** | `state` | Controlled vocabulary (`draft→active→deprecated→retired`); soft-delete (Vol 01 §8). |
| **Security** | `classification, pii_category[], access_rules[]` | Labels driving the in-kernel PDP (Vol 04; Vol 18 §5). Inheritable down the hierarchy. |
| **Provenance** | `created_by, source, generation_method, modified_by` | Who/what/whence: principal (human/AI/connector/app), source-system, manual/AI/imported/process-captured (Vol 01 §6; Vol 18 §10). |
| **Media (optional)** | `media_layers[]` | Attached first-class media (image/audio/video/3D/AR-VR), PII-maskable (Vol 06 §23; Book 1 Law 3). |
| **Workforce (reserved)** | `owner, role, assignment, workflow, approval, review, escalation` | **Reserved from day one** (EKOS-26-028) — dormant until KWR; Owner defaults Human. **No schema change later.** |

*Source:* canonical consolidation of Vol 01 §6/§7/§8, Vol 04, Vol 18 §3/§10, Vol 26 §11.

- **B2-005** — The **Workforce fields (B2-004 reserved row)** SHALL be present on every KO
  from the first implementation, inert until KWR (Vol 26) is enabled — guaranteeing zero
  schema migration. *Source:* ADR-017; EKOS-26-028.

- **B2-006** — No layer above the kernel SHALL create a KO lacking its stamped components
  (identity/type/temporal/security/provenance) — the kernel's KOM enforces this (Vol 18
  §3). *Source:* Vol 18 §3; Book 1 Law 12.

---

## 3. Relationships (Edges) as First-Class Objects

- **B2-007** — An **edge** SHALL be a typed, directed KO connecting a source KO to a
  target KO, carrying the full anatomy (§2) — including its own security, temporal, and
  provenance. *Source:* Vol 01 §5; B2-003.

- **B2-008** — Edge types SHALL be ontology meta-nodes (extensible at runtime), e.g.
  `traces_to`, `implements`, `depends_on`, `deployed_on`, `mitigated_by`, `costs`,
  `tested_by`, `owned_by`, `triggers`, `contained_in`, `changes`. *Source:* Vol 01 §5.

- **B2-009** — `traces_to` SHALL be transitively queryable — the backbone of end-to-end
  traceability. *Source:* Vol 01 §5 (EKOS-01-029).

---

## 4. Object Kinds (Taxonomy)

- **B2-010** — All KO kinds SHALL derive from the core meta-model root **`Element`**, with
  first-level `Artifact` / `Persona` / `Property`, and `Artifact` specializing into the
  full domain catalog (Vol 01 §4) + the **reflective kinds**: `WidgetDefinition`,
  `Projection`, `GovernancePolicy`, `AccessControlRule`, `Application` (EKRA), `Role`
  (KWR), `AIInsight`, `Event`. **These reflective kinds are themselves KOs** — the
  platform is self-describing. *Source:* Vol 01 §3/§4; Vol 02/05/20/23/26; Book 1 Law 15.

- **B2-011** — New kinds SHALL be addable at runtime by defining a new ontology meta-node
  (Vol 18 §6), inheriting the full KO anatomy and behavior — no schema migration.
  *Source:* Vol 01 §9; Vol 18 §6.

---

## 5. The Self-Describing Ontology

- **B2-012** — A KO's **type is itself a KO** (an ontology meta-node) — the ontology is
  stored in the same graph and is queryable/versioned like instance data. *Source:* Vol 01
  §3; Vol 18 §6; Patents §3.

- **B2-013** — Type **inheritance** SHALL propagate traceability, temporal, and security
  applicability from parent to child kind. *Source:* Vol 01 §3 (EKOS-01-010).

---

## 6. Identity, Equivalence & Lineage

- **B2-014** — **Identity** SHALL be by `guid` — two KOs are the same iff same GUID.
  *Source:* Vol 01 §6; B2-004.

- **B2-015** — **Equivalence** (two KOs representing the same fact from different sources)
  SHALL be a *derived* relation resolved by deduplication/reconciliation (Vol 21 §14) —
  distinct from identity; equivalent KOs may merge or link, never silently collide.
  *Source:* Vol 21 §14; 2nd review #3 (Knowledge Equivalence).

- **B2-016** — **Lineage/genealogy** SHALL be first-class: every KO's ancestry across
  edits, versions, branches, merges, forks, and supersession is traceable (non-
  destructive). *Source:* Vol 01 §8; 2nd review #3 (Knowledge Lineage).

- **B2-017** — A **Mutation** (§Part B) of a KO produces a new bitemporal state, not an
  overwrite — the prior state remains queryable via time-travel. *Source:* Vol 01 §7.

---

## 7. Invariants (Every KO Must Satisfy)

- **B2-018** — **Stamped:** every KO carries identity/type/temporal/security/provenance.
- **B2-019** — **Referential integrity:** an edge references only existing KOs; type
  constraints (allowed edges/cardinality) hold.
- **B2-020** — **Bitemporal monotonicity:** state is appended with timestamps; history is
  never destroyed (except governed right-to-erasure, Vol 04 §9).
- **B2-021** — **Single-owner (when KWR active):** at most one Role owns a KO at any time
  (Vol 26 §5). Dormant until KWR.
- **B2-022** — **Scoped:** a KO is only ever returned through a PDP-scoped result (Vol 18
  §5) — never raw.

*Source:* Vol 01 §12; Vol 18; Vol 04; Vol 26.

---

# PART B — The Computational Model

## 8. The Computational Principle

- **B2-023** — EKOS computation is **not code execution over data** — it is **governed
  transformation and projection of Knowledge Objects**. Computation = **Mutation
  (write) + Dispatch (react) + Projection (read/compute)**, all over the KO graph.
  *Source:* 2nd review #2 ("how does computation happen?"); Vol 18; Vol 20.

- **B2-024** — The canonical computation flow SHALL be:

```
   Knowledge Object
        │  (a change is requested)
        ▼
   Mutation ──────────► Transaction (atomic, bitemporal, validated)
        │  (committed)
        ▼
   Event  (immutable journal entry)
        │
        ▼
   Scheduler / Dispatch  ──► reactors:
        ├─ Symbolic rules (deterministic, in-kernel)
        ├─ Policy / governance gates
        ├─ AI analysts (ambient, grounded)
        ├─ Projections (reactive re-evaluation)
        └─ Connectors / webhooks
        │  (reactors may produce new Knowledge Objects)
        ▼
   new Knowledge Objects ──► (loop)

   Projection (JIT, PDP-scoped) reads KO state → a live view, any time.
```

*Source:* 2nd review #2 (the named flow); Vol 18 §3/§4/§8; Vol 20.

---

## 9. Mutation (The Write Primitive)

- **B2-025** — A **Mutation** SHALL be the atomic creation/update/state-transition/soft-
  delete of one or more KOs, performed by the kernel's KOM (Vol 18 §3): validated
  (ontology + policy), bitemporally versioned, PDP-checked, and journaled — **no quiet
  writes**. *Source:* Vol 18 §3; Book 1 Law 12.

- **B2-026** — A mutation SHALL be attributed to a **Principal** (human/AI/connector/app,
  Vol 18 §10), recorded as provenance. *Source:* Vol 18 §10; B2-004.

---

## 10. Transaction

- **B2-027** — A **Transaction** SHALL provide all-or-nothing atomicity over a multi-KO
  mutation, with bitemporal consistency (Vol 18 §7). *Source:* Vol 18 §7.

- **B2-028** — Concurrent mutations SHALL reconcile via the concurrency model — CRDT for
  collaborative edits (Vol 08 §10), optimistic for spatial, bitemporal append underneath.
  *Source:* Vol 01 §12; Vol 08 §10.

---

## 11. Event & Dispatch

- **B2-029** — Every committed mutation SHALL emit an immutable **`Event` KO** onto the
  kernel **Mutation Journal** (Vol 18 §8) — the propagation mechanism. Events are
  themselves KOs (self-describing computation). *Source:* Vol 18 §8.

- **B2-030** — The **Scheduler/Dispatch** (Vol 18 §4) SHALL, per event, fire the
  registered reactors: kernel-resident **symbolic rules + policy** (deterministic fast
  path), then user-space **AI analysts**, **reactive projections**, and **connectors** —
  fairly and resource-governed. *Source:* Vol 18 §4; Vol 03 §7.

- **B2-031** — Reactors MAY produce **new KOs** (e.g., an `AIInsight`, a derived
  requirement, a violation node) — closing the computation loop. *Source:* Vol 03; Vol 05.

---

## 12. Projection (The Read/Compute)

- **B2-032** — A **Projection** SHALL be the JIT computation that reads KO state +
  viewer scope → a live view (Vol 20). Given the same KO state + scope, a projection is
  **deterministic and cacheable**. *Source:* Vol 20; Book 1 Law 3.

- **B2-033** — Projection is the universal "read/compute" — requirements views, resumes,
  dashboards, books, compliance reports are all projections of KO state. *Source:* Vol 20
  §4; Book 1 Law 3.

---

## 13. AI as Computation

- **B2-034** — Ambient **AI is a scheduled reactor**, not a separate runtime: triggered by
  events, grounded in deterministic fact-sheets, producing `AIInsight` KOs — never
  originating unverified facts (Book 1 Law 6). *Source:* Vol 03; Vol 18 §4.

- **B2-035** — The deterministic layers (symbolic rules, graph algorithms, projections)
  are the **reproducible core** of computation; the LLM layer renders language only and
  is cached. *Source:* Vol 03 §3; Book 1 Law 6.

---

## 14. Determinism, Reproducibility & Concurrency

- **B2-036** — Given identical KO state, **deterministic computation** (rules, graph
  algorithms, projections, compilation) SHALL produce identical results — enabling audit,
  caching, reproducible builds (Vol 19 §10). *Source:* Vol 19 §10; Vol 03.

- **B2-037** — Non-determinism is confined to the LLM language layer and is **bounded**
  (grounded facts, cached output, human-gated where it mutates). *Source:* Book 1 Law 6;
  Vol 05 §7.

- **B2-038** — The concurrency model SHALL guarantee convergence (CRDT) and consistency
  (bitemporal append) under concurrent multi-principal computation. *Source:* Vol 01 §12.

---

## 15. Computation Is Itself Knowledge

- **B2-039** — Mutations, events, insights, projections-run, and compile jobs SHALL all be
  representable as KOs — **the computation is self-describing and introspectable** (via
  Observe/Query syscalls), enabling self-hosting (Vol 25) and the Chain-of-Thought
  visualizer (Vol 03 §14). *Source:* Vol 25; Vol 22 §11; Vol 03 §14.

---

# PART C — Relationship & Positioning

## 16. How Subsystems Reference the KO Model

- **B2-040** — All subsystems operate on the one KO Model: the **Kernel** allocates/
  mutates/journals KOs; the **Compiler** reads KOs → artifacts; the **Projection Engine**
  projects KOs → views; **Acquisition** produces KOs; **Apps & KWR Roles** transform KOs;
  the **Syscall layer** is the KO-operation contract. *Source:* Vols 18/19/20/21/22/23/26.

## 17. Positioning vs Known Object Models

- **B2-041** — The KO Model SHALL be positioned as the knowledge-domain analogue of:
  **Linux kernel objects** (process/file/socket — fundamental, everything builds on
  them), the **CLR Object Model** (a unified type system all languages share), **LLVM IR**
  (a target-agnostic intermediate everything compiles through), and **RDF triples** (but
  bitemporal, secured, governed, and executable). *Source:* 2nd review #1 ("CLR Object
  Model / LLVM IR"); Book 1.

## 18. Constitution (Book 1) Conformance

- **B2-042** — The KO Model realizes **single-source-of-truth (Law 2)**, **everything-is-
  projection (Law 3)** — projections compute over KOs, **security-in-the-atom (Law 4)** —
  KOs carry classification/access labels, **grounded AI (Law 6)**, **bitemporal/non-
  destructive (Law 11)**, **self-hosting (Law 15)**. *Source:* Book 1 Part II.

- **B2-043** — The **reserved Workforce fields (B2-005)** realize **15-year longevity
  (Law 14)** — no future schema break for KWR. *Source:* Book 1 Law 14; ADR-017.

---

## 19. Reserved Formalization (post-freeze, audience-gated)

- **B2-044** — 🟡 **RESERVED — Formal Semantics:** mathematical definitions of
  Projection, Knowledge Identity, Mutation, Lineage, and Equivalence SHALL be authored
  **when a standards body / certification partner requires them** (2nd review #3). This
  Book gives the *operational* semantics; the *mathematical* formalization is deferred,
  not abandoned. *Source:* 2nd review #3; Feature Freeze (formalization, audience-gated).

---

## 20. Open Questions (parked)

- **OQ-B2-A** — Exact storage shape of the reserved Workforce fields (node attributes vs a
  sub-object/edge) — finalize before Phase-1 build (carries OQ-26-D); affects A04.
- **OQ-B2-B** — Property datatype system depth (primitive set, references, computed
  properties) — with A04 data architecture.
- **OQ-B2-C** — Equivalence/dedup algorithm (how aggressively equivalent KOs merge vs
  link) — with Vol 21 OQ-21-C.
- **OQ-B2-D** — Formal-semantics trigger: which standardization milestone unlocks B2-044.

---

## 21. Item Index (this Book)

B2-001 … B2-044 (the universal atom, canonical anatomy incl. reserved workforce fields,
edges-as-objects, taxonomy, self-describing ontology, identity/equivalence/lineage,
invariants; and the computational model — principle, flow, mutation, transaction, event/
dispatch, projection, AI-as-computation, determinism/concurrency, computation-as-
knowledge; positioning; reserved formal semantics — totaling ~58 distinct stated items).

> **Next:** `WHY-NOT-EXISTING-TOOLS.md` (the category-definition paper), then continue the
> Architecture Phase (A02).
> Per protocol: STOP here. Await user "proceed".
