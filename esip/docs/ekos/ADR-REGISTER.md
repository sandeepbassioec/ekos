# EKOS Architecture Decision Register (ADR)

> **Governance artifact** (cross-architect review — immediate add). Every major
> architectural decision is recorded here with **context, decision, alternatives,
> consequences, and status** — so that *five years from now we remember WHY*. This is a
> **living register**; A03 (Technology ADRs) and every later architecture doc append to
> it. Format follows the standard ADR pattern (Nygard).
> **Started:** 2026-07-01 (Opus). Seeded with decisions already made (build-plan D1–D10,
> gap-closure structural decisions, A01 quality-attribute rulings, resolved OQs).
>
> **Status legend:** ✅ Accepted · 🔵 Proposed · 🟡 Reserved (decide later) · ⛔ Superseded.

---

## How to use
- One ADR per significant decision. Number monotonically (ADR-001, ADR-002, …).
- When a decision changes, **don't delete** — add a new ADR and mark the old "Superseded by ADR-NNN" (non-destructive, like the platform itself).
- Reference ADRs from volumes/architecture docs (e.g., "per ADR-007").
- Each ADR: **Context · Decision · Alternatives · Consequences · Status · Links.**

---

## ADR-001 — Evolve ESIP behind ports; do NOT rewrite
- **Context:** ESIP (React+Firebase+ai-proxy) is the shipped MVP; EKOS is the OS it must become.
- **Decision:** Build the platform by **evolving ESIP behind ports-and-adapters seams**, not a greenfield rewrite. ESIP becomes the first EKOS application.
- **Alternatives:** (a) Greenfield rewrite — rejected (waste, risk, solo-infeasible). (b) Keep ESIP as-is — rejected (it's a web app, not an OS).
- **Consequences:** Incremental migration (A10); stable seams required; some interim duality during migration.
- **Status:** ✅ Accepted. **Links:** Book 1 Law 13; Vol 15 §2; build-plan D1.

## ADR-002 — Hybrid .NET Core 10 + Rust backend; VectorVolt stays separate
- **Context:** Need enterprise backend + performance-critical paths; VectorVolt is a separate pure-Rust OLTP engine.
- **Decision:** Backend = **.NET Core 10** (API/integration/DB/doc-gen) + **Rust** microservice (code-scanning, rendering core, `.ekr` crypto, Gateway). **VectorVolt remains a separate pure-Rust product.**
- **Alternatives:** All-Rust (slower enterprise dev), all-.NET (weaker perf paths), Node (weaker for this).
- **Consequences:** Two-language toolchain; clear service boundaries; don't conflate with VectorVolt.
- **Status:** ✅ Accepted. **Links:** Vol 15 §5; build-plan D2.

## ADR-003 — Polyglot CQRS storage (graph DB + event store), pluggable
- **Context:** A single monolithic graph collapses at 100K-user / 10⁹-event scale.
- **Decision:** **CQRS for knowledge** — property-graph DB for structural metadata + append-only event store for high-volume events, unified by a federated query engine; all behind the Storage Abstraction port.
- **Alternatives:** Single graph DB (scale risk), single RDBMS (no graph traversal), document store (no relationships).
- **Consequences:** Federated query complexity; concrete engines deferred to benchmark (ADR-009).
- **Status:** ✅ Accepted. **Links:** Vol 01 §2; Vol 18 §9; Vol 15 §6; Gemini critique.

## ADR-004 — Client-configurable relational DB via EF Core + graph + Firebase adapters
- **Context:** Enterprises mandate their own DB; MVP runs on Firestore.
- **Decision:** Relational behind **EF Core** (PostgreSQL/SQL Server/MySQL/Oracle) + a graph adapter + a Firebase adapter, all behind the repository/provider-capability port. MVP on Firestore.
- **Alternatives:** Single fixed DB — rejected (enterprise lock-in).
- **Consequences:** Provider-capability descriptors + graceful degradation needed.
- **Status:** ✅ Accepted. **Links:** Vol 15 §6; build-plan D3.

## ADR-005 — maxGraph for diagramming
- **Context:** Need a draw.io-grade diagram engine; already integrated in ESIP (@maxgraph/core).
- **Decision:** **maxGraph** is the diagramming engine.
- **Alternatives:** xyflow (replaced), bespoke (cost), other libs (less draw.io-compatible).
- **Consequences:** maxGraph constraints accepted; rendering at scale uses native WASM/WebGPU core (ADR-006).
- **Status:** ✅ Accepted. **Links:** Vol 15 §4; build-plan D5; [[esip-resume-state]].

## ADR-006 — Native WASM/WebGPU rendering core hosted by the UI shell
- **Context:** The host UI framework cannot render 100K-node graphs at 60fps.
- **Decision:** Heavy graph layout/draw runs in a **native Rust/C++ → WebGPU/WASM** core, mounted as one surface inside the host (React) shell; host hosts, doesn't lay out.
- **Alternatives:** Pure-DOM/SVG (too slow), pure-canvas-JS (limited).
- **Consequences:** Cross-language rendering boundary; reused by the EKR Viewer.
- **Status:** ✅ Accepted (resolves OQ-02-A/07-A). **Links:** Vol 02 §3; Vol 07 §4; Vol 15 §4.

## ADR-007 — AI is provider-pluggable; latest Claude default; sovereign open models for air-gap
- **Context:** Public LLMs illegal in air-gapped/sovereign environments; vendor lock-in risk.
- **Decision:** AI behind an AI port — **latest Claude (cloud default)** + Azure-OpenAI/Vertex/watsonx/Ollama adapters; **sovereign = containerized Llama/Mistral-class** in the customer VPC; only the structured fact-sheet leaves to the model.
- **Alternatives:** Single cloud LLM (lock-in, no air-gap), train a bespoke foundation model (cost, not critical path).
- **Consequences:** Dual-engine layer; layered AI keeps LLM cost/latency controlled.
- **Status:** ✅ Accepted (resolves OQ-03-A). **Links:** Vol 03; Vol 15 §7; build-plan D4.

## ADR-008 — One shared symbolic engine for three policy kinds
- **Context:** Risk of three separate engines for ABAC rules, governance policies, and Layer-1 validation.
- **Decision:** **One kernel-resident symbolic evaluation engine** executes three *distinct, non-conflated* policy kinds: `AccessControlRule` (Vol 04), `GovernancePolicy` (Vol 05), Layer-1 validation rules (Vol 03).
- **Alternatives:** Separate engines (duplication, drift), LLM-based policy (non-deterministic — rejected).
- **Consequences:** Unified authoring surfaces; clear logical separation maintained.
- **Status:** ✅ Accepted (resolves OQ-03-B/04-A). **Links:** Vol 05 §14; Vol 18 §4.

## ADR-009 — Defer concrete graph-DB + event-store engine selection to a benchmark
- **Context:** Engine choice is high-stakes and longevity-critical; premature lock-in is risky.
- **Decision:** **Keep the choice open**; select via a load/feature **benchmark** in Phase-2 prep. Candidates (Neo4j; Kafka+ClickHouse) are *illustrative*, behind ports.
- **Alternatives:** Commit now (risk), never decide (paralysis).
- **Consequences:** Storage Abstraction port must be honored from day one so the pick is swappable.
- **Status:** 🟡 Reserved (benchmark pending). **Links:** Vol 15 §6 OQ-15-A; Vol 01 OQ-01-A.

## ADR-010 — Microkernel posture for the Knowledge Kernel
- **Context:** A large privileged core is unauditable and won't last 15 years.
- **Decision:** **Microkernel** — kernel holds only object lifecycle, ontology runtime, scheduler, resident PDP+symbolic engine, transaction/versioning, mutation journal, storage abstraction, principal/capability, tenancy/isolation. Everything else (compiler, projection, widgets, analysts, apps) is user-space.
- **Alternatives:** Monolithic (auditability/longevity risk).
- **Consequences:** Syscall boundary is performance-critical; benchmark which reactors are in-kernel (OQ-18-A).
- **Status:** ✅ Accepted. **Links:** Vol 18 §2.

## ADR-011 — Stable, versioned syscall ABI as the kernel↔user-space contract
- **Context:** Apps must survive 15 years of kernel evolution.
- **Decision:** A **stable, versioned, language-agnostic syscall ABI** (`Open/Query/Project/Commit/Validate/Package/Publish/Subscribe/Observe/Acquire/Authorize/Extend/Transact`) is the only path across the boundary; backward-compatible additive changes; governed deprecation window.
- **Alternatives:** Unstable internal APIs (breaks apps), GraphQL-only (external-only, not an OS contract).
- **Consequences:** Deprecation policy required (A09); IDL/binding toolchain (OQ-22-A).
- **Status:** ✅ Accepted. **Links:** Vol 22; Vol 18 §12.

## ADR-012 — Everything is a projection; JIT Projection Engine vs AOT Compiler
- **Context:** Views (requirements/resume/architecture/books/dashboards/compliance) were spread across widget/viewer/export.
- **Decision:** **Everything is a projection.** A **JIT Projection Engine** serves live per-viewer views; an **AOT Knowledge Compiler** produces durable multi-target artifacts (`.ekr`/PDF/Living-Book/site). They share machinery, differ in lifecycle.
- **Alternatives:** One conflated rendering layer (unclear lifecycle).
- **Consequences:** Shared-machinery boundary to design (OQ-20-A).
- **Status:** ✅ Accepted. **Links:** Vol 19; Vol 20; Book 1 Law 3.

## ADR-013 — In-kernel, server-side, zero-trust ABAC enforcement
- **Context:** Client-side or bypassable security is unacceptable for enterprise/gov.
- **Decision:** The **PDP is kernel-resident**; **no query/projection/export/AI-fact-sheet** is produced except from a server-side, PDP-scoped, masked result. Client is untrusted.
- **Alternatives:** App-layer enforcement (bypassable — rejected).
- **Consequences:** Per-element ABAC perf needs caching (OQ-04-B); enforcement logic migrates off React/Firebase to .NET.
- **Status:** ✅ Accepted. **Links:** Vol 04; Vol 18 §5; Vol 15 §13.

## ADR-014 — "Cryptographic Expiration" + one-time activation + hardware binding (not "self-destruct")
- **Context:** True bit-destruction on others' machines is impossible; honesty + accurate claims required.
- **Decision:** Term it **Cryptographic Expiration**; guarantee **key/license/token destruction & future-access denial**; copy-resistance via **one-time activation + hardware binding**; acknowledge pure-offline unlimited-copy prevention is impossible.
- **Alternatives:** Claim "self-destruct"/"unbreakable" — rejected (false, Law 4/5).
- **Consequences:** Activation server (cloud or on-prem) for the strongest mode.
- **Status:** ✅ Accepted. **Links:** Book 1 Laws 4/5; Vol 06 §9/§10; Vol 07 §9.

## ADR-015 — Cloud-neutral deployment (Docker/K8s/Helm/Terraform); air-gap first-class
- **Context:** Must serve AWS/Azure/GCP/IBM/on-prem/air-gapped without re-architecture.
- **Decision:** **Cloud-neutral** containerized deployment; multi-region active-active; per-tenant VPC option; **air-gap is first-class** (every capability has a local equivalent).
- **Alternatives:** Single-cloud (lock-in, no gov), PaaS-bound (portability loss).
- **Consequences:** Higher ops complexity; pays off in reach + longevity.
- **Status:** ✅ Accepted. **Links:** Vol 15 §11; Book 1 Law 8.

## ADR-016 — No-lock-in open export coexists with the IP moat
- **Context:** Enterprises fear vendor hostage; EKOS also wants a moat.
- **Decision:** Ship a **one-click JSON-LD/RDF export + open-source offline player** (no-lock-in guarantee) that **coexists** with the proprietary `.ekr`/ontology moat.
- **Alternatives:** Lock-in only (loses enterprise trust), open only (no moat).
- **Consequences:** Maintain two output paths; the moat is value, not hostage-taking.
- **Status:** ✅ Accepted. **Links:** Book 1 Law 9; Vol 01 §14.

## ADR-017 — KWR runs ON EKOS; reserve its data fields now
- **Context:** Knowledge Workforce Runtime is strategic but Phase-deferred.
- **Decision:** KWR is a **layer on top of EKOS** (not in it); but **every Knowledge Object reserves Owner/Role/Assignment/Workflow/Approval/Review/Escalation from day one** (even if Owner=Human, others dormant) → no schema change when KWR lands.
- **Alternatives:** Build KWR now (premature), ignore it (costly schema break later).
- **Consequences:** The reserved fields are part of the Knowledge Object Model (Book 2).
- **Status:** ✅ Accepted. **Links:** Vol 26 §11 (EKOS-26-028); A04.

## ADR-018 — Feature Freeze on new pillars (see FEATURE-FREEZE)
- **Context:** Architecture risks never shipping if pillars keep being added.
- **Decision:** **Architecture is feature-frozen** — no new subsystem/pillar unless it changes the entire category; only fixes, clarifications, formalization, and governance artifacts proceed.
- **Alternatives:** Keep expanding (never ships — rejected).
- **Consequences:** Remaining work = formalize (Book 2, comparison paper), reserve (formal semantics, conformance), then implement.
- **Status:** ✅ Accepted. **Links:** `FEATURE-FREEZE.md`; cross-architect review.

---

## ADR-019…031 — Technology Selections (full detail + scoring in `architecture/A03-technology-selection.md`)
- **ADR-019 — Query language = GQL (ISO/IEC 39075:2024)** + openCypher bridge; SPARQL at RDF export. ✅ *(15-yr: ISO standard outlives any vendor dialect.)*
- **ADR-020 — Graph store = PostgreSQL-anchored (Apache AGE default), graph engine pluggable; final pick benchmark-gated.** ✅ *(15-yr: Postgres = the no-regret database bet.)*
- **ADR-021 — Event store = Apache Kafka** + pluggable columnar (ClickHouse/Timescale). ✅ *(15-yr: Apache-governed; rejected single-vendor Redpanda.)*
- **ADR-022 — Backend = .NET (LTS) + Rust** (confirms ADR-002). ✅
- **ADR-023 — Rendering core = Rust → WebGPU/WASM** (W3C standards). ✅ (confirms ADR-006)
- **ADR-024 — Frontend shell = React + TypeScript** — *acknowledged medium 15-yr risk; mitigated as the most-replaceable layer behind the durable rendering core + syscall boundary.* ✅
- **ADR-025 — Diagramming = maxGraph** (confirms ADR-005). ✅
- **ADR-026 — AI = provider-pluggable, Claude default + sovereign open models** (confirms ADR-007) — *bet on the fact-sheet contract, not any model.* ✅
- **ADR-027 — Policy/symbolic engine = Open Policy Agent (OPA/Rego)**, embedded in kernel + WASM for offline viewer (implements ADR-008). ✅ *(CNCF-graduated standard.)*
- **ADR-028 — Real-time collaboration = Yjs (CRDT, MIT)**. ✅
- **ADR-029 — Containers/Orchestration/IaC = OCI + Kubernetes + Helm + OpenTofu** — *chose the open fork over BSL-relicensed Terraform (active license-longevity vigilance).* ✅
- **ADR-030 — Syscall IDL/binding = Protobuf + gRPC** (in-process fast path + network); public API stays GraphQL. ✅ (resolves OQ-22-A)
- **ADR-031 — Adopt open standards as load-bearing requirements:** GQL, JSON-LD/RDF, OCI, W3C VC/DID, CloudEvents, OpenTelemetry, WebGPU/WASM, X.509/Ed25519/RFC-3161, glTF, WCAG 2.2, SAML/OIDC/SCIM. ✅

## ADR-032…033 — Data architecture (full detail in `architecture/A04-data-storage-ontology-architecture.md`)
- **ADR-032 — Bitemporal strategy = SQL:2011 temporal-table pattern** (system-time + valid-time; append-new-version, close-prior; never overwrite). ✅ *(15-yr: ISO-SQL standard, audit-grade, Postgres-native. Resolves OQ-01-B.)*
- **ADR-033 — Reserved KWR field layout:** `owner` = nullable attribute (defaults human); `role/assignment/approval/review/escalation` = edges to Role/Principal KOs; `workflow` = reference edge. Present day-one, dormant until KWR → no schema migration. ✅ *(Resolves OQ-B2-A/OQ-26-D.)*

## ADR-034…035 — AI architecture (full detail in `architecture/A05-ai-ml-architecture.md`)
- **ADR-034 — Vector/embedding store = pgvector (PostgreSQL extension)** by default, behind a port. ✅ *(15-yr: one fewer vendor; rides the Postgres anchor.)*
- **ADR-035 — Bespoke small models deferred (year-2); start with embeddings + few-shot LLM, distill later** from consented anonymized data. ✅ *(Cost-responsible; not on critical path.)*

## ADR-036…038 — Security architecture (full detail in `architecture/A06-security-identity-trust-architecture.md`)
- **ADR-036 — Crypto-agility interface + reserved post-quantum path (Kyber/Dilithium)**; AES-256 quantum-safe-enough now. ✅ *(15-yr: must be able to rotate algorithms; reserving PQC prevents a forced rewrite.)*
- **ADR-037 — Identity = pluggable IdP via SAML+OIDC + SCIM/JIT + app-gallery**, feeding the PIP → kernel Principal model. ✅ *(15-yr: entrenched standards, no IdP lock-in.)*
- **ADR-038 — Secrets via secrets-manager/KMS; none in client or repo.** Pre-deploy blockers B1–B5 must be green before hosting (rotate ai-proxy keys, harden Firestore rules, fix JWT mismatch, wire KMS, server-side ABAC). ✅

## ADR-039…040 — Integration architecture (full detail in `architecture/A07-integration-and-api-architecture.md`)
- **ADR-039 — Two-tier API:** internal = Protobuf+gRPC syscalls (OS contract); external = GraphQL (+REST where needed) as a user-space service ON the syscalls, same ABAC. ✅
- **ADR-040 — Connector Framework** (bidirectional; inbound=acquisition adapters, outbound=sync; two-way Jira/ADO; sandboxed; anti-corruption layer maps external schemas→ontology). ✅

## ADR-041…043 — Deployment architecture (full detail in `architecture/A08-deployment-infrastructure-architecture.md`)
- **ADR-041 — Four deployment profiles, one artifact set:** SaaS multi-tenant · dedicated single-tenant · on-prem · air-gapped. ✅ *(No divergent codebases for 15 years.)*
- **ADR-042 — Active-active multi-region + zero-downtime upgrades + self-service DR** (enabled per tier; relies on stable syscall ABI). ✅
- **ADR-043 — Deployment scales with adoption, not ambition:** MVP = one managed region; multi-region/air-gap/dedicated stood up only when a tenant needs it (a config change via IaC/profiles, not a re-architecture). ✅ *(Solo/budget-honest.)*

## ADR-044…046 — 15-year sustainability (full detail in `architecture/A09-fifteen-year-sustainability-and-evolution.md`)
- **ADR-044 — Dependency-currency policy:** security patches in days; minor/patch auto-PR+auto-merge on green; major on scheduled cadence; LTS-preferred; SBOM + continuous vuln scanning. ✅
- **ADR-045 — Stable-contract deprecation windows:** syscall ABI & fact-sheet supported ≥2 major / ≥24 months; public API deprecations ≥12 months; KO model never breaks; ontology types soft-deprecated only. ✅ *(Resolves OQ-18-D/22-B.)*
- **ADR-046 — Tech-Radar cadence (6–12 months):** classify every significant tech Adopt/Trial/Hold/Retire; watch license changes; Retire → planned port-adapter migration. ✅

## ADR-047…049 — ESIP→EKOS migration (full detail in `architecture/A10-esip-to-ekos-migration-architecture.md`)
- **ADR-047 — Strangler-fig migration:** new EKOS seams progressively strangle ESIP internals; every step shippable/reversible/verified; ESIP becomes the first EKOS app; no big-bang rewrite. ✅
- **ADR-048 — H3 requirement-model convergence:** unify FormalRequirement + DAL Requirement + localStorage Requirement into one `Requirement` KO; remove the capture-board localStorage bypass; preserve authored content. ✅
- **ADR-049 — Data migration:** Firestore→graph behind the port via anti-corruption mapping + provenance stamping + bitemporal backfill; idempotent/reconciling/verified; reserved KWR fields populated (Owner=human). ✅

## ADR-050 — Representation Independence + KRR Scope Expansion (2026-07-01)
- **Context:** Vol 20 was named "Projection Engine" — one operation (selection+transform+
  visual recipe). A first-principles challenge identified that "Projection" names only the
  *first* capability inside a much broader subsystem, and that no Law explicitly required
  knowledge to be stored only as knowledge (not as representations). Enterprises today
  store 100K PDFs, 50K PowerPoints, 20K Word docs — duplicating knowledge in frozen
  representational form, causing drift, bloat, and traceability collapse. The insight:
  every document, diagram, book, dashboard, and report is a runtime manifestation of
  knowledge, not knowledge itself.
- **Decision:** (1) Enact **Book 1 Law 16 — Representation Independence**: representations
  are ephemeral by default; only canonical knowledge and representation rules persist; the
  sole exception is the **immutable evidence artifact** (crossed a legal/regulatory/
  contractual boundary — filed, signed, sent). (2) Rename Vol 20's conceptual scope to
  **Knowledge Representation Runtime (KRR)**: projection is the first capability; the KRR
  also covers rendering, interaction, navigation, animation, streaming, responsive layout,
  localization, themes, branding, accessibility. (3) Define the **Knowledge Representation
  Framework (KRF)** as the contract/semantics layer inside KRR. (4) Formalize **Intent-
  Based Representation** (EKOS-20-040): a KO declares representation intent; the KRR
  fulfills it — knowledge owns rendering, applications do not. All prior EKOS-20-*
  identifiers remain stable; "Projection Engine" is a backward-compatible alias.
- **Alternatives:** (a) Keep "Projection Engine" name and narrow scope — rejected: it
  undersells the subsystem and leaves rendering/interaction out of the spec. (b) Create a
  *separate* KRR volume — rejected: over-fragmentation; one coherent volume is better for
  a solo maintainer; the KRF/KRR conceptual split is inside Vol 20. (c) Persist
  representations for performance — rejected: violates Law 2 (single source of truth);
  correct solution is caching (§10), not storage.
- **Consequences:** Law 16 is binding; every volume that discusses "stored documents" must
  mean evidence artifacts only; all new KO types default to ephemeral representation.
  Storage estimates in deployment architecture (A08) reduce significantly. AI outputs
  (Vol 03) are representation intents, not file edits (Law 6 alignment). New patent
  candidate: Intent-Based Representation (EKOS-20-044).
- **Status:** ✅ Accepted. **Links:** Book 1 Law 16; Vol 20 §16–§17; EKOS-20-036–044;
  Vol 16 (patent family).

> **ADR-047…049** (migration) recorded above. **Architecture Phase A01–A11 COMPLETE
> (2026-07-01)** — 49 ADRs logged. **ADR-050** enacted 2026-07-01 (post-architecture
> insight: Law 16 + KRR). Total: **50 ADRs**.

## ADR-051 — Constitutional Compliance Runtime (CCR)
- **Context:** Book 1 declares 16 Laws that are binding on every caller — UI, SDK, EKR
  Viewer, KWR Worker, third-party extension, autonomous AI agent. Until 2026-07-03, these
  Laws existed as a *document* (design-time intent) but had no runtime enforcement layer.
  No architecture document specified *where* the Laws are checked, *who* checks them,
  *what error type* a violation produces, *how* violations are observed, or *how*
  third-party extensions declare and prove constitutional adherence. The Constitution was
  philosophy without a court. This gap was identified by the user: "whosoever calls it —
  whether UI or SDK or EKR reader or EKOS or anybody extending it — should follow the same
  Constitution and it should give the centralized observability, logging and metrics and
  error mechanism."
- **Decision:** Introduce the **Constitutional Compliance Runtime (CCR)** as a
  cross-cutting enforcement layer at the syscall boundary (Vol 22). The CCR:
  (1) Intercepts every `Knowledge.*` syscall on every caller path — no opt-out,
  no trusted-caller bypass, no internal escape hatch.
  (2) Enforces Laws in three tiers: Structural (architecture-enforced), Behavioral
  (CCR runtime check per syscall), Policy (OPA/Rego delegation).
  (3) Produces a typed `ConstitutionalViolation` error hierarchy — callers handle
  specific law violations; violations are never suppressible.
  (4) Emits every evaluation (pass or violation) to a **Constitutional Telemetry Stream
  (CTS)** — a single, immutable, caller-agnostic observability pipeline covering all
  callers across all deployment tiers (SaaS, air-gap, EKR Viewer, Edge).
  (5) Requires a signed **Constitutional Compliance Manifest (CCM)** from every
  third-party extension — declaring syscalls used, law engagements, and evidence-artifact
  types — checked at registration and on every syscall.
  The Constitution is enforced by the runtime, not trusted to callers.
- **Alternatives considered:**
  (a) *Trust internal callers; only check third-party extensions* — rejected. Internal
  callers are principals with grants, not trusted actors. A compromised internal service
  should be governed identically to any other caller. "Trusted internal" is the most
  common source of compliance failures in real systems.
  (b) *Enforce laws in each subsystem independently* — rejected. This produces N
  independent enforcement implementations, N independent observability streams, and
  inevitable gaps where a subsystem forgets to enforce a law. Centralization at the
  syscall boundary is the only model that guarantees uniformity.
  (c) *Enforcement via code review / architecture linting only* — rejected. Design-time
  gates catch known violations in first-party code; they do not cover third-party
  extensions, AI agents, or runtime edge cases. Both are needed; neither replaces the other.
  (d) *Separate CCR service (sidecar)* — rejected for the in-process fast path. CCR must
  be < 1ms P99; a network hop to a sidecar adds unjustifiable latency for every syscall.
  The enforcement logic runs in-process; the *telemetry* is emitted to Kafka (asynchronous,
  non-blocking).
- **Consequences:**
  Positive: The Constitution becomes a runtime contract, not just a document. Every
  caller sees identical enforcement. A single CTS stream provides audit, compliance
  reporting, anomaly detection, and constitutional health metrics for the entire platform.
  Extension ecosystem is safe by default (CCM required). `ConstitutionalViolation` type
  hierarchy makes law enforcement introspectable in code.
  Negative: CCR adds in-process overhead to every syscall (target: < 1ms P99 for
  behavioral-tier checks; OPA policy checks are async where possible). CCM registration
  adds friction for extension authors (mitigated by SDK tooling that generates CCMs).
  The typed violation hierarchy must be kept stable across ABI versions (breaking a
  violation type is a breaking change — governs by the same deprecation policy as the
  syscall ABI, ADR-019 / EKOS-22-019).
- **Status:** ✅ Accepted. **Links:** Book 1 Part III (CCR specification); Vol 22 §§14–16
  (CCR requirements, constitutional telemetry, extension manifest); ADR-050 (Law 16 /
  KRR — the law CCR enforces); Vol 04 §7 (no-existence-leak rule, honored by
  ConstitutionalViolation responses); Vol 05 (governance / OPA — Tier 3 enforcement);
  A06 (security architecture — CCR is a security-critical path).

---

### ADR-052 — Storage Provider Abstraction Model (Multi-Database Portability)

- **Date:** 2026-07-03
- **Status:** ✅ Accepted
- **Deciders:** Sandeep Bassi (architect), Claude (design partner)
- **Context:**
  A03 ADR-020 declared that the graph store is "behind the Storage Abstraction port (Vol 18 §9) —
  swappable," and A02 established hexagonal ports everywhere. However, neither document formalizes the
  *adapter contract* — what a storage driver must implement to plug into EKOS. Without a formal
  interface specification, "swappable" is an intention, not a deployable capability.
  The user identified the critical risk during pre-ADR review: *"We would like to support multiple
  graph databases, if it needs relational database then multiple and same way for NoSQL databases.
  Tomorrow if we distribute it for on-prem, client's cloud infrastructure (they may want cheaper
  services) then this will be another factor which goes against us."* This surfaces four concrete
  failure modes: (1) an enterprise client who already has SQL Server or MongoDB refuses to provision
  a new database just to run EKOS; (2) a government/air-gap client cannot use any managed cloud DB;
  (3) a cost-sensitive SME rejects PostgreSQL-specific operational overhead when they already run
  MySQL; (4) a client on AWS with Amazon Neptune or Azure with Cosmos DB cannot bring their existing
  investment. Any hard dependency on a specific database loses those deals or forces unwanted
  infrastructure. EKOS must be database-agnostic from the first production deployment.
- **Decision:**
  Adopt a formal **Storage Provider Model** with four interface families, each independently
  pluggable at deploy time. EKOS core code calls only interfaces — never database drivers directly.
  Clients configure which providers to use via Helm values / environment configuration.
  VectorVolt (per `VECTORVOLT-EKOS-INTEGRATION-PLAN.md`) is the long-term unified provider that
  eventually implements all four interfaces; until VectorVolt matures to each gate, the below
  reference implementations are used.

  **Interface 1 — `IGraphStoreProvider`** (knowledge graph: nodes, edges, traversal, Cypher/GQL)
  | Provider | License | When to Use |
  |---|---|---|
  | `PostgreSQLAGEProvider` | Apache 2.0 | Default — lowest operational overhead, same Postgres instance as relational |
  | `Neo4jProvider` | GPL/Commercial | Clients with existing Neo4j enterprise license |
  | `JanusGraphProvider` | Apache 2.0 | Very-large-scale graph (>10B edges) on HBase/Cassandra backend |
  | `AmazonNeptuneProvider` | Proprietary (AWS) | AWS-native clients; behind port so no lock-in |
  | `VectorVoltGraphProvider` | Open-source | Phase 3 gate — VectorVolt graph extension complete |

  **Interface 2 — `IRelationalStoreProvider`** (structured data, EF Core projections, governance tables)
  | Provider | License | When to Use |
  |---|---|---|
  | `PostgreSQLProvider` | PostgreSQL License | Default |
  | `SQLServerProvider` | Commercial | Enterprise on-prem clients with existing SQL Server licenses |
  | `MySQLProvider` | GPL / Commercial | Cost-sensitive deployments; MySQL-native cloud offerings |
  | `SQLiteProvider` | Public Domain | Embedded / developer / air-gap single-node deployments |

  **Interface 3 — `IDocumentStoreProvider`** (hot cache, KO identity reads, CCR capability cache)
  | Provider | License | When to Use |
  |---|---|---|
  | `RedisProvider` | SSPL / Redis Source | Default hot cache |
  | `MongoDBProvider` | SSPL / Commercial | Clients on MongoDB Atlas |
  | `CosmosDBProvider` | Proprietary (Azure) | Azure-native clients (Mongo API) |
  | `CassandraProvider` | Apache 2.0 | Large-scale write-heavy NoSQL tenants |
  | `VectorVoltDocumentProvider` | Open-source | Phase 2 gate — VectorVolt Phase 5 DocumentStore |

  **Interface 4 — `IColumnarStoreProvider`** (analytics, aggregates, widget dashboards, reporting)
  | Provider | License | When to Use |
  |---|---|---|
  | `ClickHouseProvider` | Apache 2.0 | Default analytics |
  | `TimescaleProvider` | Timescale License | PostgreSQL-embedded option (same instance as relational) |
  | `VectorVoltColumnarProvider` | Open-source | Phase 2 gate — VectorVolt Phase 5 ColumnarStore + VectorScanner |

  **Adapter Contract Requirements** (all four interfaces must satisfy):
  (a) Implements the EKOS storage interface (defined in `EKOS.Storage.Abstractions` package).
  (b) Respects the bitemporal data model — valid-time and transaction-time columns are mandatory
  schema requirements; the adapter translates them to native constructs (temporal tables, timestamp
  columns, etc.).
  (c) Is configured entirely via the EKOS Helm values / environment config — no code changes needed
  to switch providers.
  (d) Supports the EKOS tenancy model — tenant isolation is enforced at the provider level (row-level
  security, namespace separation, or separate schema per tenant depending on provider capability).
  (e) Emits health metrics to the Constitutional Telemetry Stream (CTS) — provider health is
  observable alongside constitutional compliance, not via a separate channel.

- **Rejected Alternatives:**
  (a) *Single-provider mandate (PostgreSQL + AGE only)* — rejected. Law 10 requires providers to be
  pluggable; mandating a single provider violates it. Enterprise on-prem and client-cloud deployments
  make this commercially untenable.
  (b) *Thin abstraction layer with hard-coded Postgres fallback* — rejected. A thin wrapper that
  falls back to Postgres for all unhandled cases means third-party providers can never fully replace
  the default. The interface must be fully abstract.
  (c) *ORM-only abstraction (EF Core providers)* — rejected for graph and columnar tiers. EF Core
  covers the relational interface adequately, but has no first-class graph query or columnar SIMD
  model. The four-interface model extends beyond what any single ORM provides.
  (d) *Wait for VectorVolt to implement all four interfaces before launching EKOS* — rejected.
  VectorVolt Phases 1–3 are prerequisites for EKOS Phase 1, but VectorVolt Phase 5 (read fabric) and
  the graph extension are years away. EKOS cannot wait. The provider model lets EKOS launch with
  reference providers, then transparently upgrade to VectorVolt as each phase completes.

- **Consequences:**
  Positive: EKOS is database-agnostic from first deployment — any client infrastructure is
  compatible. VectorVolt is positioned as the performance upgrade path, not a hard dependency.
  Enterprise, SME, government, and air-gap deployments all supported without custom builds.
  Provider switching (e.g., AGE → VectorVolt graph) requires only configuration changes.
  Negative: Four interfaces multiply integration test surface (each reference provider needs its own
  test suite). Community-contributed providers may be lower quality than reference implementations
  (mitigated by conformance test kit — see below). Bitemporal requirement adds schema complexity for
  providers that don't natively support temporal tables (they must implement it in application code).
  Provider health diversity means ops teams need familiarity with whichever providers they deploy.
  
- **Implementation Note:**
  Ship a **Storage Provider Conformance Test Kit** (EKOS.Storage.ConformanceTests) — a shared test
  suite that any provider implementation must pass before being considered EKOS-compatible. Community
  providers run this kit as CI. This is analogous to the TCK (Technology Compatibility Kit) used by
  Jakarta EE. Makes "EKOS-compatible" a verifiable claim, not a self-declaration.

- **Links:** ADR-020 (graph store port origin); ADR-003 (hexagonal architecture); ADR-009
  (benchmark gates for final picks); Book 1 Law 10 (provider pluggability); Vol 18 §9 (Storage
  Abstraction port); `architecture/VECTORVOLT-EKOS-INTEGRATION-PLAN.md` (phase-by-phase provider
  fulfillment schedule); `00-MASTER-INDEX.md` OQ-A03-A (graph-store benchmark gate).

> **ADR-052** enacted 2026-07-03. Total: **52 ADRs**.

---

### ADR-053 — VectorVolt as EKOS Storage Engine (Phased Integration)

- **Date:** 2026-07-03
- **Status:** ✅ Accepted
- **Deciders:** Sandeep Bassi (architect), Claude (design partner)
- **Context:**
  ADR-052 established four pluggable storage provider interfaces (`IGraphStoreProvider`,
  `IRelationalStoreProvider`, `IDocumentStoreProvider`, `IColumnarStoreProvider`) with
  reference implementations (PostgreSQL+AGE, Redis, ClickHouse) as the launch defaults.
  ADR-052 explicitly reserved VectorVolt as the long-term unified provider across all four
  interfaces, referencing `VECTORVOLT-EKOS-INTEGRATION-PLAN.md` for the phased schedule.
  This ADR formally records that decision: VectorVolt — the zero-GC, kernel-bypass,
  sub-microsecond OLTP engine being built by the same team — is the designated long-term
  storage engine for EKOS. This is an unusual architectural choice (first-party storage
  engine rather than adopting an existing database) that requires explicit justification.
  The motivation is performance: EKOS's constitutional constraints demand sub-microsecond
  CCR capability lookups (EKOS-22-041: CCR < 1ms P99 per syscall), immutable append-only
  CTS writes at high throughput (metering + telemetry), and SIMD-accelerated Knowledge
  Object scans with ABAC filters. No existing open-source database simultaneously provides
  zero-GC memory management, kernel-bypass networking, SIMD columnar reads, hardware-clock
  bitemporal timestamps (PTP IEEE 1588), and a WAL that serves as the constitutional
  audit spine. VectorVolt is being built to provide all of these, engineered specifically
  for the EKOS workload profile.
- **Decision:**
  VectorVolt is the designated long-term EKOS storage engine, integrated in three
  phases across six gates (full specification in `VECTORVOLT-EKOS-INTEGRATION-PLAN.md`):

  **Integration Phase 1 — Gate 1 (VectorVolt Phase 1+2+3 complete)**
  VectorVolt provides: (a) WAL for the Constitutional Telemetry Stream — immutable,
  hash-chained, tamper-evident (satisfies EKOS-22-046); (b) GroupCommitBatcher for
  high-throughput metering events (Knowledge Economy, Vol 13); (c) ProjectionDispatcher
  syncing WAL → PostgreSQL+AGE (keeping graph store current); (d) InterceptorPipeline
  as the CCR enforcement hook before events reach storage. All other storage (graph,
  document, columnar) remains with reference providers until Gate 3.

  **Integration Phase 2 — Gate 3 (VectorVolt Phase 5 complete)**
  VectorVolt's Read Fabric replaces Redis (`IDocumentStoreProvider`) and ClickHouse
  (`IColumnarStoreProvider`): DocumentStore for sub-10ns CCR capability cache reads;
  VectorScanner (AVX2/AVX-512) for ABAC-filtered Knowledge Object scans;
  MaterializedViewEngine for widget dashboard aggregates. PostgreSQL+AGE continues as
  `IGraphStoreProvider` for all graph traversals until Gate 6.

  **Integration Phase 3 — Gate 6 (VectorVolt Graph Extension complete)**
  VectorVolt's graph extension (property graph model + Cypher/GQL + bitemporal edges)
  replaces PostgreSQL+AGE as `IGraphStoreProvider`. VectorVolt becomes the single engine
  for graph, document, and columnar tiers. PostgreSQL remains permanently for relational
  projections (EF Core). Full specification of graph requirements: Vol 01 (ontology),
  A04 (bitemporal), `VECTORVOLT-EKOS-INTEGRATION-PLAN.md` §6.

  **What VectorVolt never replaces:** `IRelationalStoreProvider` (PostgreSQL) — EKOS uses
  PostgreSQL for EF Core–based projection targets, control-plane tables, and governance
  data. VectorVolt is not a relational database and makes no claim to be one.

- **Rejected Alternatives:**
  (a) *Use an existing database (TigerBeetle / FoundationDB / ScyllaDB)* — rejected.
  TigerBeetle covers financial double-entry accounting (not a general OLTP engine);
  FoundationDB has strong consistency but no SIMD read fabric or graph model; ScyllaDB
  is C++ and provides no bitemporal semantics. None provide the specific combination of
  zero-GC + kernel-bypass + SIMD columnar + bitemporal edges + WAL-as-CTS-spine that
  EKOS requires. Building VectorVolt also yields a standalone PhD research artifact and
  industry recognition (user goal), which purchasing an existing database does not.
  (b) *Use PostgreSQL+AGE permanently (never build VectorVolt for EKOS)* — rejected.
  PostgreSQL round-trips are ~1–3ms; CCR must be <1ms P99 per syscall (EKOS-22-041).
  Columnar ABAC scans over millions of KOs in PostgreSQL require full row deserialization;
  VectorVolt VectorScanner processes 8–16 values per CPU cycle. At EKOS scale (10⁹
  events/day, per A08-017), PostgreSQL-only architecture fails the performance contract.
  (c) *Build EKOS's storage layer as a separate internal library (not VectorVolt)* —
  rejected. VectorVolt and EKOS share the same development team. Splitting into two
  separate storage projects doubles maintenance burden with no architectural benefit.
  VectorVolt's phased gate model means EKOS is never blocked waiting for VectorVolt
  capabilities that don't exist yet — reference providers bridge every gap.
  (d) *Use Redis for all hot-path reads permanently* — rejected. Redis is single-threaded,
  GC-adjacent (Lua garbage collection), and has no SIMD scan capability. Redis serves as
  the `IDocumentStoreProvider` until Gate 3 precisely because it is adequate for the
  interim — not because it meets EKOS's long-term performance contract.

- **Consequences:**
  Positive: EKOS's performance contract (sub-microsecond CCR, SIMD KO scans, WAL-backed
  CTS) is achievable without compromising on existing database limitations. VectorVolt and
  EKOS co-evolve — each VectorVolt phase immediately maps to an EKOS capability. The gate
  model means EKOS is never blocked: reference providers run until VectorVolt is ready,
  then swap transparently via ADR-052 configuration. VectorVolt becomes a standalone
  research + commercial product in parallel with EKOS.
  Negative: EKOS has a strategic dependency on VectorVolt completing its phases on
  schedule. If VectorVolt development stalls, EKOS remains on reference providers
  indefinitely (which is functional but not optimal). The graph extension (Gate 6) is
  unscoped in VectorVolt's current 8-phase plan — it requires additional phase planning
  before Gate 6 can open. First-party storage engines carry higher operational risk than
  battle-tested databases; mitigated by VectorVolt's Phase 7 deterministic simulation
  testing (10M failure scenarios) and chaos testing before any Gate promotion.

- **Invariant:** VectorVolt provider promotion at each gate is a **configuration-only
  change** (Helm values swap) — no EKOS application code changes, no data migration, no
  downtime. The ADR-052 provider interface guarantees this. Any gate promotion that
  requires application code changes is a defect in the provider interface, not a planned
  migration step.

- **Status:** ✅ Accepted. **Links:** ADR-052 (provider interfaces); ADR-020 (graph store
  original decision); `architecture/VECTORVOLT-EKOS-INTEGRATION-PLAN.md` (6-gate schedule,
  gate status tracker); `C:\VectorVolt\IMPLEMENTATION_PLAN.md` (VectorVolt 8-phase plan);
  EKOS-22-041 (CCR <1ms P99 requirement); Vol 01 (graph ontology — Gate 6 requirements);
  A04 (bitemporal — Gate 6 edge model); A16 (VectorVolt node pool in distributed topology).

> **ADR-053** enacted 2026-07-03. Total: **53 ADRs**.

## Reserved / Pending decisions (to become ADRs when decided)
- Bitemporal storage strategy (OQ-01-B) · Query language openCypher/GQL (leaning yes, OQ-01-C) · K-IR representation (OQ-19-A) · IDL/binding toolchain (OQ-22-A) · ABI deprecation window length (OQ-18-D/22-B) · fintech licensing per region (OQ-13-A) · concrete small-model family (OQ-03-D) · VectorVolt graph extension scope (OQ-A21-A relates; formal ADR when Phase 8 graph extension is scoped).

> **Next decisions to register:** A02 (logical boundaries), A03 (the full technology-
> selection ADR set, scored via the Trade-off methodology), A04–A11.
