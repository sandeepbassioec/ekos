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

## Reserved / Pending decisions (to become ADRs when decided)
- Bitemporal storage strategy (OQ-01-B) · Query language openCypher/GQL (leaning yes, OQ-01-C) · K-IR representation (OQ-19-A) · IDL/binding toolchain (OQ-22-A) · ABI deprecation window length (OQ-18-D/22-B) · fintech licensing per region (OQ-13-A) · concrete small-model family (OQ-03-D).

> **Next decisions to register:** A02 (logical boundaries), A03 (the full technology-
> selection ADR set, scored via the Trade-off methodology), A04–A11.
