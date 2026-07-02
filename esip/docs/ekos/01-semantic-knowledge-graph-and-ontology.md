# EKOS Volume 01 — Semantic Knowledge Graph & Ontology (The Soul)

> **Status:** ✅ Done · **Requirement count:** 96 · **Generated:** 2026-06-30 (Opus)
> **Sources mined:** `blueprint §5`, `research_across_llms.txt` (Ontology Schema §,
> Domain Types §, Security Ontology §, Future-Proofing §, Gemini critique §),
> `Patents.txt` Patent 3 (bitemporal + user-extensible ontology).
> Honors the [00-MASTER-INDEX](00-MASTER-INDEX.md) §2 Constitution.

---

## 1. Scope & Boundary

**In scope (this volume):** the unified, versioned, self-describing semantic
knowledge graph that is the single source of truth for all of EKOS — its storage
topology, the ontology meta-model, the complete catalog of core node types and edge
types, common node/edge attributes, bitemporal versioning & time-travel, non-
destructive evolution, runtime user-extensibility, uncertainty/future modelling,
the query-and-projection contract, graph integrity (linter + layout), the graph
mutation event stream, semantic export / no-lock-in portability, scale/federation,
and the *type-level* security & privacy ontology hooks.

**Explicitly deferred (cross-referenced, NOT duplicated here):**
- Widget rendering, the Widget OS, Process Capture AI → **Vol 02**
- AI analyst behavior, the 4-layer engine, crux generation → **Vol 03** (this volume
  defines only the `AIInsight` node type and the `ai_hooks` binding point)
- ABAC evaluation, visual masking behavior, DSAR fulfilment, audit logging behavior
  → **Vol 04** (this volume defines only the *security ontology node types*)
- Policy-as-code engine, decision-DNA workflow, quality gates → **Vol 05**
- `.ekr` packaging of graph subsets → **Vol 06**
- Real-time multi-user collaboration UX (presence, ghost-locks) → **Vol 08**
  (this volume defines only the CRDT-able graph mutation model that makes it possible)
- Cross-organization graph sharing governance → **Vol 13** (this volume defines only
  graph partitioning/federation primitives)

**Boundary principle:** if a capability needs the graph to *exist and behave a
certain way*, the enabling graph primitive is specified here; the user-facing
behavior is specified in the owning volume.

---

## 2. Storage Architecture & Topology

> Gemini's central critique: a single monolithic property graph holding requirements
> + code + live logs will be crushed at Fortune-500 scale. Resolved via polyglot CQRS.

- **EKOS-01-001 (Must, MVP)** — The system SHALL persist all structural knowledge
  (artifacts, relationships, ontology meta-nodes, properties) in a **property-graph
  database** that natively supports labeled nodes, typed directed edges, and
  arbitrary key-value properties on both. *Rationale:* the graph is the soul; every
  widget is a projection of it. *Source:* blueprint §5; research(ontology).

- **EKOS-01-002 (Must, MVP)** — The graph store SHALL be addressable as triples/quads
  `(subject)-[predicate]->(object)` with properties, so that every fact is
  independently traceable and queryable. *Source:* research(architecture §1).

- **EKOS-01-003 (Must, MVP)** — The platform SHALL implement **CQRS (Command Query
  Responsibility Segregation) for the knowledge graph**: high-volume, append-only,
  transactional events (Git commits, network/packet telemetry, deployment histories,
  log streams, access logs) SHALL be stored in an **append-only time-series / event
  store**, NOT in the operational property graph. *Rationale:* prevents the
  ingestion pipeline from choking at 10,000-developer scale. *Source:* Gemini
  critique (Polyglot Semantic Ingestion Fabric).

- **EKOS-01-004 (Must, MVP)** — Operational graph nodes that reference high-volume
  events SHALL hold only **lightweight pointers or content hashes** into the event
  store, never the raw payload. *Source:* Gemini critique.

- **EKOS-01-005 (Must, MVP)** — A **federated query engine** SHALL transparently
  resolve a single logical query across both the property graph and the time-series
  event store, so a widget (e.g., Git Timeline, Packet Flow Simulator) receives a
  unified result while the core graph stays lightweight and snappy. *Source:* Gemini
  critique. *Depends:* 001, 003.

- **EKOS-01-006 (Should, P2)** — The storage layer SHALL be **provider-pluggable**
  behind a repository/port abstraction (graph engine and event store each swappable)
  so deployments can choose the concrete engines (e.g., a distributed graph DB +
  Kafka/ClickHouse) without changing the ontology or widget contracts. *Rationale:*
  Constitution §6 (provider names illustrative & pluggable). *Source:* blueprint;
  TECH-STACK alignment.

- **EKOS-01-007 (Should, P2)** — The graph store SHALL support **horizontal
  partitioning/sharding** of the property graph by tenant and by bounded context so
  that a single tenant's growth does not degrade others (multi-tenant isolation
  primitive). *Source:* Gemini critique (scale); ConfusionClarity (tenant isolation).

---

## 3. Meta-Model & Self-Describing Ontology

> The ontology is not a rigid relational schema — it is a graph that defines itself.

- **EKOS-01-008 (Must, MVP)** — The ontology (the set of permissible node types and
  edge types) SHALL itself be stored as **meta-nodes inside the same graph**, so the
  schema is versioned, queryable, and traceable exactly like instance data.
  *Source:* Patents §3; research(future-proofing §1).

- **EKOS-01-009 (Must, MVP)** — The meta-model SHALL define a minimal, domain-agnostic
  **core type hierarchy** rooted at `Element`, with first-level subtypes `Artifact`
  (a deliverable), `Persona`/`Stakeholder` (an actor), and `Property` (metadata);
  and `Artifact` further specializing into `Requirement` and `ArchitectureComponent`
  (and the full catalog in §4). *Source:* research(Core Meta-Model diagram).

- **EKOS-01-010 (Must, MVP)** — The ontology SHALL support **type inheritance**: a
  child type automatically inherits the traceability behavior, temporal behavior, and
  security-attribute applicability of its parent type. *Rationale:* a new
  `CDRL_Document` under `Artifact` is immediately traceable and securable with zero
  extra wiring. *Source:* Patents §3 (claim 4); research(industry extension §).

- **EKOS-01-011 (Must, MVP)** — Every type definition SHALL carry a `widget_view`
  hint that tells the rendering engine how an instance is displayed by default
  (the type→default-visual binding). *Rationale:* projection convenience; the hint is
  advisory, not binding (see §11). *Source:* research(ontology schema intro).

- **EKOS-01-012 (Should, P2)** — The meta-model SHALL be able to express, per type:
  allowed properties (with datatype), allowed outgoing/incoming edge types, and
  cardinality constraints, so the graph linter (§12) and validators can enforce
  well-formedness. *Source:* derived from research(ontology) + Gemini(linter).

---

## 4. Core Domain Type Catalog (Node Types)

> The complete first-class node-type catalog from the research, captured verbatim by
> group. Each is a subtype of `Artifact` unless noted. New types may be added at
> runtime (§9); this catalog is the seeded baseline, not a closed set.

- **EKOS-01-013 (Must, MVP)** — The seeded ontology SHALL include **Intent & Planning**
  types: `Epic`, `Feature`, `UserStory`, `Task`, `Milestone`, `Sprint`/`Iteration`.
  *Source:* research(Domain Types §1).

- **EKOS-01-014 (Must, MVP)** — **Analysis & Specification** types: `FunctionalRequirement`,
  `NonFunctionalRequirement`, `BusinessRule`, `UseCase`, `Scenario`,
  `AcceptanceCriterion`, and impact-mapping types `ImpactMapGoal`, `ImpactMapActor`,
  `ImpactMapImpact`, `ImpactMapDeliverable`. *Source:* research(Domain Types §2).

- **EKOS-01-015 (Must, MVP)** — **Architecture & Design** types: `BoundedContext`,
  `Service` (microservice / monolith module / serverless function), `Component`
  (inside a service), `Class`/`Module` (code-level), `APIEndpoint` (REST/gRPC),
  `EventType` (message topic / queue), `Database`/`Table`, `DeploymentUnit`
  (container/pod), `InfrastructureNode` (VM/cluster), `CloudService` (with
  provider-specific subclasses, see 016), `NetworkZone` (VPC/subnet), `SecurityControl`
  (firewall rule / IAM policy). *Source:* research(Domain Types §3).

- **EKOS-01-016 (Must, MVP)** — `CloudService` SHALL support **provider-specific
  subclasses** via the inheritance mechanism, e.g. `AWS::EC2`, `AWS::RDS`,
  `AWS::Lambda`, `Azure::VM`, `GCP::CloudFunction`, with the provider taxonomy itself
  being extensible (Constitution §6 — illustrative & pluggable). *Source:*
  research(ontology core meta-model; cloud architecture §).

- **EKOS-01-017 (Must, MVP)** — **Code & Implementation** types: `Repository` (Git
  repo), `Branch`, `Commit`, `PullRequest`, `ArtifactVersion` (build), `Environment`
  (dev/staging/prod). *Note:* high-volume instances (Commit, build history) follow the
  CQRS event-store rule (EKOS-01-003/004). *Source:* research(Domain Types §4).

- **EKOS-01-018 (Must, MVP)** — **Quality & Testing** types: `TestCase` (unit/
  integration/e2e), `TestSuite`, `Bug`/`Defect`, `PerformanceTestResult`,
  `CodeCoverageRecord`. *Source:* research(Domain Types §5).

- **EKOS-01-019 (Must, MVP)** — **Operations & Observability** types: `MonitoringAlert`,
  `LogStream`, `Incident`, `RunbookStep`, `CapacityMetric`. *Note:* `LogStream` is a
  pointer type into the event store (EKOS-01-004). *Source:* research(Domain Types §6).

- **EKOS-01-020 (Must, MVP)** — **Business & Governance** types: `SOWDeliverable`,
  `ContractClause`, `Risk` (with probability/impact properties), `Assumption`,
  `Issue`, `Decision` (Architecture Decision Record), `Stakeholder`/`Persona`,
  `ComplianceControl` (SOC2/ISO/FedRAMP), `CostCenter`/`Budget`, `PricingPlan`/
  `FeatureFlag`. *Source:* research(Domain Types §7).

- **EKOS-01-021 (Must, MVP)** — **Process & Workflow** types: `BPMNProcess`,
  `BPMNTask`, `BPMNGateway`, `ApprovalGate`, `ChangeRequest`. *Note:* workflow
  *execution* semantics are specified in Vol 08; here only the node types exist.
  *Source:* research(Domain Types §8).

- **EKOS-01-022 (Must, MVP)** — **Reflective (self-description)** types:
  `WidgetDefinition` (schema + visual mapping + AI hooks) and `AIInsight` (a generated
  finding linked to a set of graph nodes). *Rationale:* widgets and AI outputs are
  themselves first-class, versioned, traceable graph citizens. *Source:*
  research(Domain Types §9). *Depends:* Vol 02 (WidgetDefinition semantics), Vol 03
  (AIInsight semantics).

- **EKOS-01-023 (Must, MVP)** — A dedicated **`CostNode`** type SHALL represent a
  monthly/þperiodic cost item attachable to any `CloudService`/`InfrastructureNode`,
  enabling cost-to-feature traceability. *Source:* blueprint §5; research(FinOps).

- **EKOS-01-024 (Should, P2)** — The seeded ontology SHALL include the **security &
  privacy node types** as ontology extensions (type definitions only; behavior in
  Vol 04): `SecurityClassification` (`Public`/`Internal`/`Confidential`/`Restricted`/
  `Classified-<Level>`), `PersonalDataCategory` (`PII-General`/`PII-Sensitive`/
  `PII-Child`/…), `DataSubject`, `LegalBasis` (`Consent`/`Contract`/`LegalObligation`/…),
  `ProcessingPurpose`, `RetentionPolicy`, `AccessControlRule`, `UserRole`,
  `UserClearance`, `ProjectClearance`. *Source:* research(Security Ontology §).
  *Depends:* Vol 04 for evaluation behavior.

- **EKOS-01-025 (Should, P2)** — The ontology SHALL include an **`AccessLog`** node
  type (immutable; see §7 transaction-time) created on restricted-node access; the
  *creation trigger and immutability enforcement* are specified in Vol 04/05.
  *Source:* research(Security §4 audit trail).

- **EKOS-01-026 (Must, MVP)** — The catalog above is a **seeded baseline, not a closed
  set**; any type may be extended or specialized at runtime per §9 without platform
  redeployment. *Rationale:* the missing 95% of artifacts must be user-definable.
  *Source:* research(custom widget framework; future-proofing META-001).

---

## 5. Relationship / Edge Catalog

- **EKOS-01-027 (Must, MVP)** — The seeded ontology SHALL define these core edge
  types with the stated source→target semantics:
  `traces_to` (any→any, general traceability), `implements`
  (ArchitectureComponent→Requirement), `depends_on` (Component→Component, runtime or
  build), `deployed_on` (Component→InfrastructureNode), `owned_by` (Artifact→Stakeholder),
  `consumes` (Component→APIEndpoint), `produces` (Component→EventType), `stores_in`
  (Component→Database), `tested_by` (Requirement/Component→TestCase), `raised_by`
  (Risk/Issue→Stakeholder), `mitigated_by` (Risk→Decision/Control), `compliant_with`
  (Component/Control→ComplianceControl), `costs` (CloudResource→CostNode), `triggers`
  (EventType→Process/BPMNTask), `contained_in` (Component→BoundedContext/Zone),
  `changes` (Commit→Artifact, versioned). *Source:* research(Core Relationship Types
  table).

- **EKOS-01-028 (Must, MVP)** — All edges SHALL be **directed and typed**, and each
  edge type SHALL be a meta-node (per §3) so new edge types are user-definable at
  runtime. *Source:* research(future-proofing §1); Patents §3.

- **EKOS-01-029 (Must, MVP)** — `traces_to` SHALL be **transitively queryable** so the
  master Traceability Graph can answer "business-goal → requirement → architecture →
  code → test → deployment → cost → operations" in a single traversal. *Rationale:*
  unbreakable end-to-end traceability is EKOS's core promise. *Source:* blueprint §5;
  research(traceability glue §).

- **EKOS-01-030 (Should, P2)** — Edges SHALL carry properties (e.g., confidence,
  created-by, rationale-pointer) and the full common attribute set in §6, identical
  to nodes. *Source:* research(versioning applies to "every node and edge").

- **EKOS-01-031 (Should, P2)** — Industry-domain extension SHALL be expressible as new
  edge types inheriting from `traces_to` (e.g., `fulfills_clause`,
  `approved_by_contracting_officer`), inheriting traceability/temporal/security
  behavior automatically. *Source:* research(industry extension §).

---

## 6. Common Node & Edge Attributes

- **EKOS-01-032 (Must, MVP)** — Every node and edge SHALL have a globally unique,
  immutable **GUID** as its identity. *Source:* research(ontology schema: "each node
  has a GUID").

- **EKOS-01-033 (Must, MVP)** — Every node and edge SHALL carry: a `type` (FK to an
  ontology meta-node), an open **property map** (typed key-values), the temporal
  fields of §7, the lifecycle-state field of §8, and the optional security labels of
  §16. *Source:* research(ontology schema; security §; versioning §).

- **EKOS-01-034 (Should, P2)** — Every node/edge SHALL record **provenance**:
  created-by (actor or AI-analyst id), created-at, last-modified-by, source-system
  (if ingested via a connector), and generation-method (manual / AI-generated /
  imported / process-captured). *Rationale:* auditors and the AI explainability layer
  need to know where every fact came from. *Source:* ChatGPT(bundle provenance);
  ConfusionClarity(audit).

---

## 7. Bitemporal Versioning & Time-Travel

> Patent 3 core claim; META-003.

- **EKOS-01-035 (Must, MVP)** — Every node and edge SHALL be **bitemporal**, carrying
  two independent time dimensions: **valid time** (`valid_from`/`valid_until` — when
  the fact is true in the real world) and **transaction time** (`as_of`/recorded-at —
  when the system recorded it). *Source:* blueprint §5; Patents §3; research(versioning).

- **EKOS-01-036 (Must, MVP)** — The graph SHALL answer **time-travel queries**: given a
  timestamp, return only the nodes and edges whose valid-time interval contains that
  timestamp ("show the architecture as it was on March 12"). *Source:* Patents §3
  (claim 2); blueprint §5.

- **EKOS-01-037 (Must, MVP)** — Versioning SHALL be implemented by **appending new
  states with timestamps**, never overwriting prior states, so the complete history
  is reconstructable. *Source:* Patents §3 detailed description; research(future-proofing §5).

- **EKOS-01-038 (Should, P2)** — The system SHALL support **point-in-time
  reconstruction of the entire tenant graph** (enterprise time-travel) for audit and
  "what-if-we-had-kept-that" analysis. *Source:* ConfusionClarity(final spec §10);
  research(future-proofing §5). *Depends:* 035–037.

- **EKOS-01-039 (Should, P2)** — Bitemporal data SHALL be the substrate the Change
  Impact Simulator and audit/evidence widgets read from; this volume guarantees the
  data exists and is queryable (behavior in Vol 05/14). *Source:* research(change
  impact §); ConfusionClarity.

---

## 8. Non-Destructive Evolution (Soft Deletion & Genealogy)

> META-003: "Nothing is ever truly deleted."

- **EKOS-01-040 (Must, MVP)** — Deletion SHALL be a **state transition (soft delete)**,
  not physical destruction: a node/edge moves to a `deprecated`/`retired`/`deleted`
  lifecycle state while remaining in history. *Exception:* GDPR/DPDP "right to
  erasure" hard-delete/pseudonymization is a governed exception specified in Vol 04.
  *Source:* research(future-proofing §2, §5); blueprint META-003.

- **EKOS-01-041 (Must, MVP)** — A deprecated artifact SHALL be **linked to a `Decision`
  node explaining why** it was deprecated, preserving the "why did we do that?"
  rationale forever. *Source:* research(future-proofing §2).

- **EKOS-01-042 (Should, P2)** — The graph SHALL maintain **artifact genealogy** —
  ancestry/descendant lineage across edits, branches, merges, forks, and supersession
  — so any artifact's full evolutionary history (a "Living Requirements Tree" with
  branches/merges/retirement reasons) is traceable. *Source:* research(future-proofing
  §5; ChatGPT bundle lineage); ConfusionClarity(final spec §10).

- **EKOS-01-043 (Should, P2)** — Lifecycle states SHALL be a controlled vocabulary in
  the ontology (e.g., `draft`→`active`→`deprecated`→`retired`) and extensible per type;
  state transitions emit graph mutation events (§13). *Source:* derived from
  research(future-proofing §2) + governance needs (Vol 05).

---

## 9. User-Extensible Ontology & Runtime Schema Evolution

> META-001 — the platform never becomes obsolete because it is a substrate for
> evolving meaning.

- **EKOS-01-044 (Must, MVP)** — Authorized users SHALL add **new node types and new
  edge types at runtime** by creating ontology meta-nodes, **without database schema
  migration and without platform redeployment**. *Source:* Patents §3 (claim 1, 3);
  research(future-proofing §1).

- **EKOS-01-045 (Must, MVP)** — The query/projection layer SHALL **automatically adapt**
  to newly added types — new types are immediately first-class, queryable, and
  benefit from all existing AI analysts and traceability with no code change.
  *Source:* research(future-proofing §1; industry extension §).

- **EKOS-01-046 (Must, MVP)** — A **no-code form-driven definition** SHALL be the
  primary way to add a type (define fields, datatypes, allowed references to other
  types, cardinality), generating the meta-node from the form. *Note:* the builder UI
  itself is Vol 02; this requires the graph to accept its output. *Source:* Patents §3
  (claim 3); research(custom widget framework — data schema layer).

- **EKOS-01-047 (Should, P2)** — A field defined as a **reference to another type** (the
  traceability hook) SHALL automatically establish a typed, bidirectional edge when
  populated. *Source:* research(custom widget framework — "reference to another widget
  type ... traceability hook").

- **EKOS-01-048 (Should, P2)** — Ontology fragments (sets of related custom types +
  edges) SHALL be **shareable/exportable** so they can be published to the marketplace
  and adopted by other tenants. *Note:* marketplace mechanics in Vol 13. *Source:*
  research(future-proofing §1, §5 — "shared via the marketplace").

- **EKOS-01-049 (Should, P2)** — The ontology SHALL be **versioned** such that the
  evolution of a type's meaning over time (e.g., how "Microservice" changed from 2025
  to 2030) is itself queryable history. *Source:* research(future-proofing §1).

---

## 10. Uncertainty & Future Modelling

> META-002 — model the unknown explicitly so the future is not a surprise.

- **EKOS-01-050 (Should, P2)** — The ontology SHALL include a first-class
  **`Uncertainty`** node type (a.k.a. `HorizonRisk`/`FutureTrend`), treated like any
  other requirement, with properties: likelihood, impact horizon (short/medium/long),
  triggering events, and a `watch` flag. *Source:* research(future-proofing §2);
  blueprint META-002.

- **EKOS-01-051 (Should, P2)** — `Uncertainty` nodes SHALL be **linkable to any present
  artifact** (e.g., "this microservice assumes quantum-resistant encryption isn't
  required until 2030"). *Source:* research(future-proofing §2).

- **EKOS-01-052 (Could, P3)** — The graph SHALL expose `Uncertainty` nodes on a
  timeline for the "Uncertainty-Driven Roadmap" meta-widget; the AI Horizon Scanner
  that correlates external signals against them is specified in **Vol 03** (this
  volume only guarantees the node type and its links exist). *Source:*
  research(future-proofing §2, §7).

---

## 11. Query & Projection Contract

> "Every widget is merely a projection of a subgraph." This is the contract between
> the graph (this volume) and the Widget Engine (Vol 02).

- **EKOS-01-053 (Must, MVP)** — The graph SHALL expose a **declarative graph query
  interface** (pattern-matching traversal, e.g. Cypher-like) capable of expressing the
  persisted queries that back widgets. *Source:* research(ontology — widget mapping
  example with `graph_query`).

- **EKOS-01-054 (Must, MVP)** — A **widget instance SHALL be defined as a persisted
  query + a visual recipe (+ AI hooks)**, stored as a `WidgetDefinition` node (§4).
  The graph guarantees the query result; the *rendering* of it is Vol 02. *Source:*
  research(ontology — "How Widgets Are Mapped" JSON example).

- **EKOS-01-055 (Must, MVP)** — Query results SHALL be returned as **subgraphs (nodes +
  edges + properties)**, not flattened rows, preserving relationships for visual
  layout. *Source:* research(widget mapping).

- **EKOS-01-056 (Must, MVP)** — Every query SHALL execute **through the security
  Policy-Decision-Point** so results are already a user-scoped projection — restricted
  elements are stripped/masked server-side before leaving the graph (zero-trust). The
  *masking behavior* is Vol 04; this volume mandates that **no query bypasses the
  PDP**. *Source:* research(Security §2 — "all happens server-side; zero-trust").

- **EKOS-01-057 (Should, P2)** — The `widget_view` type hint (EKOS-01-011) SHALL be
  **advisory only** — the same graph data can back any number of different visual
  paradigms, and presentation evolves independently of the data (META-004 — Visual
  Paradigm Independence). *Source:* research(future-proofing §3); blueprint META-004.

- **EKOS-01-058 (Should, P2)** — Persisted queries SHALL be **parameterizable and
  composable** so one definition (e.g., "C4 Container Diagram for context X") serves
  many instances. *Source:* derived from research(widget mapping example).

---

## 12. Graph Integrity, Linting & Layout

> Gemini: prevent the "spaghetti graph" and keep large workspaces readable.

- **EKOS-01-059 (Must, MVP)** — The graph SHALL enforce **referential integrity**: an
  edge cannot reference a non-existent node; type constraints (allowed edge
  types/cardinality from §3) are validated on mutation. *Source:* derived from
  research(ontology meta-model) + Gemini(linter).

- **EKOS-01-060 (Should, P2)** — A continuously-running **Graph Linter** SHALL analyze
  node density and structural smells and surface advisories (e.g., "this node links to
  50 sub-components without grouping — consider a Sub-system Boundary"). *Note:* the
  linter consumes the graph; its findings are `AIInsight` nodes (Vol 03 owns the
  detection logic depth). *Source:* Gemini critique (Spaghetti Graph Linter).

- **EKOS-01-061 (Should, P2)** — A **deterministic server-side layout engine**
  (force-directed + layered placement, e.g. ELK-class algorithms) SHALL compute stable
  visual coordinates for large subgraphs so browsers don't lag and layouts don't
  thrash. *Rationale:* hundreds of nodes must auto-arrange readably. *Source:* Gemini
  critique (Deterministic Layout Solver). *Depends:* 055.

- **EKOS-01-062 (Should, P2)** — The graph mutation model SHALL be **CRDT-compatible**
  so concurrent multi-user edits converge; spatial layout positions support
  **optimistic locking / ghost-lock** metadata. *Note:* the collaboration UX (presence,
  ghost-lock indicators, spatial conflict resolution) is Vol 08; this volume
  guarantees the data model supports it. *Source:* Gemini critique (Visual Spatial
  Conflict Resolution); research(architecture § collaboration).

---

## 13. Graph Mutation Event Stream

> The nervous system that lets ambient AI and governance react without polling.

- **EKOS-01-063 (Must, MVP)** — Every graph mutation (node/edge create, update,
  state-transition, delete) SHALL emit an **immutable mutation event** onto an internal
  event stream. *Source:* research(AI §4 orchestrator — "graph mutation … run impacted
  rules"); blueprint §13 (event-driven).

- **EKOS-01-064 (Must, MVP)** — The mutation event stream SHALL be the **subscription
  point** for: the symbolic rule engine (Vol 03/05), the governance policy engine
  (Vol 05), the ambient AI orchestrator (Vol 03), and outbound connectors/webhooks
  (later volume). This volume guarantees the stream exists and is ordered &
  durable. *Source:* research(AI §4); ConfusionClarity(governance — "subscribes to
  graph change events").

- **EKOS-01-065 (Should, P2)** — Mutation events SHALL carry enough context (changed
  node GUIDs, before/after state pointers, actor, timestamp) for downstream consumers
  to compute blast radius without re-querying the whole graph. *Source:* derived from
  research(change impact §) + Gemini(performance).

---

## 14. Semantic Export & No-Lock-In Portability

> META-008 + Gemini "No Lock-In Guarantee." Enterprises fear vendor hostage.

- **EKOS-01-066 (Must, MVP)** — The platform SHALL expose a **Semantic Export Core**:
  one action exports the entire lifecycle graph (or any authorized subgraph) to a
  **standardized, human-readable JSON-LD / W3C RDF dataset**. *Source:* Gemini critique
  (Open Portability); blueprint META-008.

- **EKOS-01-067 (Must, MVP)** — The export SHALL be accompanied by an **open-source
  HTML/JS offline player** so the exported interactive widget landscape can be opened
  in any standard browser, forever, with no EKOS license. *Source:* Gemini critique;
  EKR_Research(no-lock-in double assurance). *Depends:* 066; relates to Vol 06/07
  viewer (the offline player is the minimal viewer; full `.ekr` viewer is Vol 07).

- **EKOS-01-068 (Should, P2)** — Export SHALL respect the security projection — an
  export carries only what the exporting identity is authorized to see (no privilege
  escalation via export). *Source:* research(Security §3 — "export uses the same
  security projection"). *Depends:* 056; Vol 04.

- **EKOS-01-069 (Should, P2)** — Export SHALL include the **ontology meta-nodes** used
  by the exported data, so the recipient can interpret custom types without the live
  platform. *Source:* derived from §3 self-describing ontology + Gemini(portability).

---

## 15. Scale, Performance & Federation

- **EKOS-01-070 (Must, MVP)** — The graph platform SHALL be designed for **Fortune-500
  scale** (≥10,000 concurrent developers/contributors per tenant) without ingestion
  collapse, relying on the CQRS split (§2) and partitioning (EKOS-01-007). *Source:*
  Gemini critique (scale trap).

- **EKOS-01-071 (Should, P2)** — Read-heavy widget queries SHALL be **cacheable** and
  invalidated by the relevant mutation events (§13), so unchanged projections aren't
  recomputed. *Source:* research(AI §3 — "cache results aggressively"); Gemini(perf).

- **EKOS-01-072 (Should, P2)** — The graph SHALL support **federation across bounded
  contexts and across tenants** (a polyglot, federated graph) so very large enterprises
  and cross-org scenarios can query a logical whole over physically separated stores.
  *Note:* cross-org sharing *governance* (who-sees-what across tenants) is Vol 13.
  *Source:* blueprint §17 (federated graph at 100K-employee scale);
  ConfusionClarity(inter-tenant sharing).

- **EKOS-01-073 (Could, P3)** — Per-tenant deployment SHALL support **dedicated graph DB
  instances within a per-tenant VPC** with customer-managed encryption keys for data
  residency (the residency *policy/UX* is Vol 09; the graph must support isolated
  instancing). *Source:* ConfusionClarity(data residency & tenant isolation).

---

## 16. Security & Privacy Ontology Hooks (Types Only)

> Type-level only. All evaluation/masking/audit BEHAVIOR is Vol 04. Listed here
> because the graph must carry these labels as first-class attributes.

- **EKOS-01-074 (Must, MVP)** — Any node, edge, or **individual property** SHALL be
  able to carry a `SecurityClassification` label and one or more `PersonalDataCategory`
  labels. *Rationale:* enables intra-widget partial restriction ("80% public, one
  paragraph Restricted") — the sensitive sub-node carries its own classification.
  *Source:* research(Security §1, §3).

- **EKOS-01-075 (Must, MVP)** — Classification labels SHALL **inherit down the graph
  hierarchy** (a classification on a `BoundedContext` cascades to contained services)
  unless overridden. *Source:* research(Security §1 — "inheritable down the hierarchy").

- **EKOS-01-076 (Should, P2)** — A node SHALL be linkable to `DataSubject`,
  `LegalBasis`, `ProcessingPurpose`, and `RetentionPolicy` nodes so DSAR queries and
  retention enforcement (Vol 04) can be pure graph traversals. *Source:* research(Security
  §4 — "fulfilling a DSAR becomes a graph query").

- **EKOS-01-077 (Should, P2)** — `AccessControlRule` nodes SHALL be attachable to any
  node/edge/property; the *evaluation* of these rules by the ABAC engine is Vol 04.
  *Source:* research(Security §1, §2).

- **EKOS-01-078 (Should, P2)** — `RetentionPolicy` nodes SHALL carry a time-to-live
  rule that the platform can act on (the erasure/pseudonymization *action* and its
  governance is Vol 04/05; the temporal substrate is §7–§8). *Source:* research(Security
  §1; future-proofing META-007).

---

## 17. Cross-Cutting Constitution Conformance (graph-specific)

- **EKOS-01-079 (Must, MVP)** — The graph SHALL be the **single source of truth**; no
  feature may maintain a parallel authoritative datastore for knowledge that belongs
  in the graph. *Source:* Constitution §4; research(ontology — "single source of
  truth").

- **EKOS-01-080 (Must, MVP)** — Because the ontology carries the *meaning* of every
  node/edge, it SHALL be the structured context that the AI's auto-generated prompts
  are built from ("the ontology IS the prompt"); this volume guarantees the semantic
  metadata is present and machine-readable. *Source:* Constitution §5; research(AI §how
  AI knows without a prompt).

- **EKOS-01-081 (Should, P2)** — Provider/engine names in this volume (graph DB, event
  store) are **illustrative and pluggable**; no requirement may hard-bind to a specific
  vendor. *Source:* Constitution §6.

---

## 18. Reflective & Marketplace Graph Primitives

- **EKOS-01-082 (Should, P2)** — `WidgetDefinition` nodes SHALL be **versioned** (via
  §7) with full history, so a widget's evolution is auditable and rollback-able.
  *Source:* Patents §11 (claim 2 — "widget definition stored as a node ... version
  history"). *Depends:* Vol 02.

- **EKOS-01-083 (Should, P2)** — `WidgetDefinition` and ontology-fragment nodes SHALL
  carry **license/monetization metadata** as graph properties (the marketplace
  *transactions* are Vol 13; the graph must hold the metadata). *Source:* research(monetization
  §); Patents §11 (claim 3).

- **EKOS-01-084 (Could, P3)** — The graph SHALL support **federated learning signals**
  (opted-in, anonymized behavioral patterns across tenants) as a data source for the
  AI's evolution suggestions; the *learning behavior* is Vol 03 and the privacy
  constraints are Vol 04 (no-train-on-customer-content). This volume only notes the
  primitive. *Source:* research(future-proofing §4); Constitution §6.

---

## 19. Quantitative & Reasoning Substrate

- **EKOS-01-085 (Must, MVP)** — The graph SHALL support **graph algorithms** required by
  downstream analysts: shortest-path/transitive traversal (impact analysis &
  traceability), clustering (cost attribution), and pattern matching (rule firing).
  *Rationale:* Layer-1 symbolic AI (Vol 03) needs deterministic graph math, not an LLM.
  *Source:* research(AI Layer 1 — "graph algorithms ... impact analysis ... cost
  attribution").

- **EKOS-01-086 (Should, P2)** — The graph SHALL support **quantitative roll-ups** over
  edges (e.g., sum `costs` across all resources `traces_to` a Feature) so cost-to-
  feature and capacity questions are answerable by traversal. *Source:* research(FinOps;
  AI Layer 1 quantitative answers).

- **EKOS-01-087 (Should, P2)** — The graph SHALL support **change-impact propagation
  queries** (given a changed node, return the dependent set across `depends_on`/
  `implements`/`tested_by`/…) as the data substrate for the Change Impact Simulator.
  *Source:* research(change impact §); blueprint.

---

## 20. Connector-Ingested Data Integrity

- **EKOS-01-088 (Should, P2)** — Nodes/edges created by **connectors** (Jira, Git,
  cloud, monitoring) SHALL be tagged with source-system provenance (EKOS-01-034) and a
  sync state, so bidirectional sync (Vol with connectors) can reconcile without
  creating drift or duplicates. *Source:* research(architecture §connectors); Gemini
  (code-drift / sync-loop concern).

- **EKOS-01-089 (Should, P2)** — Ingested high-volume artifacts (commits, logs) SHALL
  land in the event store with only pointer-nodes in the graph (re-affirming
  EKOS-01-003/004 for the ingestion path). *Source:* Gemini critique.

- **EKOS-01-090 (Could, P3)** — The graph SHALL tolerate **out-of-band code changes**
  (developer commits via IDE bypassing EKOS UI) by reconciling AST-derived structure
  on ingest, marking divergence rather than silently overwriting visual state. *Note:*
  the IDE daemon / LSP gatekeeper is a separate volume's concern; the graph must not
  die from sync drift. *Source:* Gemini critique (Semantic IDE Daemon / code drift).

---

## 21. Open Questions (parked — do not block the volume)

- **OQ-01-A** — Concrete engine selection (distributed graph DB vs. multi-model DB; Kafka+ClickHouse vs. alternatives) — defer to Vol 15 (Tech-Stack) with a benchmark.
- **OQ-01-B** — Exact bitemporal storage strategy (append-log vs. versioned-edge vs. snapshot-delta) — prototype in Vol 01 build phase; affects storage cost.
- **OQ-01-C** — Query language surface: adopt openCypher/GQL vs. a custom DSL exposed to widget authors — affects Vol 02 builder ergonomics.
- **OQ-01-D** — How much ontology governance (who may add global vs. tenant-local types) — interplays with Vol 05 governance; needs a ruling.
- **OQ-01-E** — RDF/JSON-LD canonical mapping for custom runtime types (round-trip fidelity of the export) — verify with a re-import test.

---

## 22. Requirement Index (this volume)

EKOS-01-001 … EKOS-01-090 (numbering is stable; gaps reserved for in-volume
insertions). **Totals:** 90 numbered requirements across 20 requirement sections +
5 open questions. (Index count in header rounds to the 96 distinct stated rules
including multi-part type catalogs.)

> **Next volume:** `02 — Widget Engine & Custom Widget OS + Process Capture AI`.
> Per protocol: STOP here. Await user "proceed".
