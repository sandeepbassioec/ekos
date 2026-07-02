# ESIP Architecture Automation Suite — Volume 21: Role-Conditioned AI Expertise & Trade-off Analysis Engine

**Prefixes:** `REQ-AIX-` (Role-Conditioned AI Expertise) · `REQ-TRD-` (Trade-off Analysis Engine)
**Status:** Draft v1.0 — 2026-06-23
**Scope:** AI expert-persona prompt packs for every architect role (P1–P14 + custom); prompt-pack governance and versioning; auto-adoption by active persona; proactive role-specific nudges; evaluation of role-expertise quality; trade-off elicitation, matrices, ATAM analysis, weighted scoring, ADR linkage, library/reuse, visualization, collaboration, and proactive detection of decision points.

**Boundary notes:**

| Concern | Owner |
|---|---|
| Base LLM / RAG / embeddings / agent infra / provider management | V06 |
| Prompt library infrastructure (storage, versioning, deployment pipeline, A/B test harness) | V06 §12 |
| Persona definitions, metamodel, role configuration | V05 / V11 |
| Per-user preference and adaptive AI-memory | V14 |
| ADR lifecycle, options generation, HLD | V04 |
| Deliverable template content and formatting | V23 (planned) |
| Cross-role review and approval workflows | V15 |
| Prescriptive next-best-action engine (portfolio level) | V16 |
| Role-specific artifact generation (diagrams, documents, code analysis) | V06, V03, V10 |
| **Expert-persona prompt-pack specification, governance & role-expertise evaluation** | **This volume (AIX)** |
| **Trade-off elicitation, matrices, ATAM, weighted scoring, visualization, proactive detection** | **This volume (TRD)** |

> **Phase banner:** Per the master-index ruling (§6 ruling #1), `Must` means "required in the product," not "in the Phase-1 MVP." Build sequence is governed by `BUILD-PLAN-MVP-AND-PHASING.md`. The expert-persona prompt-pack core (AIX §1–§3) and trade-off matrix core (TRD §1–§2) are foundational Phase 2; advanced evaluation, library, and visualization capabilities are Phase 3.

---

## Personas Reference

| Code | Persona | Volume Detail |
|------|---------|---------------|
| P1 | Enterprise Architect | V01 |
| P2 | Solution Architect | V04 |
| P3 | Application Architect | V05 |
| P4 | Integration Architect | V05 |
| P5 | API Architect | V05 |
| P6 | CRM / Platform Architect | V05 |
| P7 | Data Architect | V05 |
| P8 | Security Architect | V05 |
| P9 | Cloud Architect | V05 |
| P10 | Infrastructure / Network Architect | V05 |
| P11 | Microservices / Platform / DevOps Architect | V05 |
| P12 | Any Architect (extensible) | V05 §12 |
| P13 | Product Manager | V17 |
| P14 | Presales / Bid Architect | This volume (new persona — see AIX §2.14) |
| ALL | All personas | — |

---

## Table of Contents

### REQ-AIX — Role-Conditioned AI Expertise

1. [AIX §1 — Prompt-Pack Catalog & Structure](#aix-1)
2. [AIX §2 — Per-Role Expert Persona Packs (P1–P14 + Custom)](#aix-2)
3. [AIX §3 — Prompt-Pack Governance, Versioning & Administration](#aix-3)
4. [AIX §4 — Active-Persona Auto-Adoption](#aix-4)
5. [AIX §5 — Proactive Role-Specific Suggestions & Nudges](#aix-5)
6. [AIX §6 — Grounding, Citations & Evidence Linking](#aix-6)
7. [AIX §7 — Role-Expertise Evaluation & Quality Assurance](#aix-7)
8. [AIX §8 — Safe Fallback & Escalation](#aix-8)

### REQ-TRD — Trade-off Analysis Engine

9. [TRD §1 — Elicitation Interview Mode](#trd-1)
10. [TRD §2 — Trade-off Matrix Generation](#trd-2)
11. [TRD §3 — ATAM-Style Quality-Attribute Analysis](#trd-3)
12. [TRD §4 — Weighted Scoring & Recommendation Engine](#trd-4)
13. [TRD §5 — What-If Sensitivity Analysis](#trd-5)
14. [TRD §6 — ADR & Options Analysis Linkage](#trd-6)
15. [TRD §7 — Trade-off Library & Reuse](#trd-7)
16. [TRD §8 — Visualization (Radar / Heatmap / Pareto)](#trd-8)
17. [TRD §9 — Collaboration & Review of Trade-offs](#trd-9)
18. [TRD §10 — Proactive Decision-Point Detection](#trd-10)

---

## REQ-AIX — Role-Conditioned AI Expertise

---

### AIX §1 — Prompt-Pack Catalog & Structure {#aix-1}

#### AIX §1.1 Prompt-Pack Concept & Registry

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-001 | Prompt-Pack Registry | The suite SHALL maintain a central Prompt-Pack Registry where each entry is a named, versioned configuration record containing: pack ID, target persona code(s), pack version (semver), status (Draft/Active/Deprecated), author, description, linked persona definition (→ V11 metamodel), applicable contexts/triggers, and a list of constituent prompt-component references. | Governance | ALL | Must | Central registry enables discoverability, lifecycle management, and cross-volume integration without hard-coding prompt strings. |
| REQ-AIX-002 | Prompt-Pack Composition Model | Each prompt pack SHALL be composed of four typed constituent components: (1) **System Persona Prompt** — role identity, voice, seniority, and epistemic stance; (2) **Role Heuristics & Frameworks Block** — the key mental models, methodologies, standards, and rules-of-thumb the AI SHALL embody; (3) **Few-Shot Exemplars** — a minimum of three input/output pairs demonstrating role-appropriate responses; (4) **Do/Don't Guardrails Block** — explicit do/don't rules governing tone, assumptions, depth, and what the role should never say or skip. | AI/ML | ALL | Must | Decomposing packs into typed components allows targeted governance updates (e.g., update guardrails without touching exemplars) and enables component-level reuse across packs. |
| REQ-AIX-003 | Prompt-Pack Schema Validation | The suite SHALL validate every prompt pack against a defined schema (field types, minimum component counts, max token budget per component) before activation; packs that fail validation SHALL be blocked from activation with a structured validation report. | Governance | ALL | Must | Prevents malformed or undersized packs from degrading AI output quality in production. |
| REQ-AIX-004 | Token-Budget Enforcement per Component | The suite SHALL enforce a configurable maximum token budget per prompt-component type (system persona, heuristics, exemplars, guardrails) to prevent context-window exhaustion; the admin SHALL be able to set global defaults and per-pack overrides. | NFR | ALL | Must | Context-window overflow silently truncates prompt components, degrading AI role-expertise quality. V06 owns the provider context-window limits; this volume owns the budgeting policy. |
| REQ-AIX-005 | Pack-Level Localization | The suite SHALL support per-pack locale variants (at minimum: en-US, en-GB, fr-FR, de-DE, ja-JP) so that role-specific exemplars and do/don't rules can be expressed in the architect's working language; locale inheritance SHALL cascade: pack-locale → tenant-locale → platform default. | UX | ALL | Should | Architectural vocabulary (e.g., "solution brief," "operating model") has locale-specific nuance. Cross-ref V11 i18n and V19 (i18n/l10n). |
| REQ-AIX-006 | Persona-to-Pack Default Mapping | The suite SHALL maintain a default mapping from each persona code (P1–P14 + custom) to its primary prompt pack; a single persona MAY map to multiple ordered packs (context-switching packs) and the active pack SHALL be deterministic given the current user context (active persona + current capability area + current artifact type). | Functional | ALL | Must | Without a deterministic mapping, the AI defaults to a generic assistant mode rather than expert-persona mode. Cross-ref AIX §4 (auto-adoption). |
| REQ-AIX-007 | Multi-Pack Composition at Runtime | The suite SHALL support assembling a runtime prompt from multiple packs at inference time (e.g., base Enterprise-Architect pack + Security overlay pack + tenant custom pack) with a defined merge order and conflict-resolution policy (later pack overrides earlier for the same component type, additive for heuristics). | AI/ML | ALL | Should | Real-world architects work across discipline boundaries; pure single-pack mode misses cross-cutting expertise (e.g., a Solution Architect applying security-architecture heuristics). |

---

#### AIX §1.2 Exemplar Design Requirements

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-008 | Exemplar Coverage Mandate | Each role's few-shot exemplar set SHALL include at minimum one exemplar each for: (a) drafting a role-signature artifact; (b) critiquing/reviewing an existing artifact with role-appropriate depth; (c) answering a "what would you recommend?" advisory question with role-appropriate frameworks and caveats. | AI/ML | ALL | Must | Exemplars that cover only drafting leave the AI weak at the critique and advisory modes that practitioners rely on most. |
| REQ-AIX-009 | Exemplar Realism Standard | Exemplars SHALL be written at senior-practitioner quality — using real framework terminology, real tool/standard names, real architectural heuristics — not generic AI completions; a content-review step (human-in-the-loop or LLM-judge) SHALL be required before any exemplar is promoted to Active status. | AI/ML | ALL | Must | Low-quality exemplars produce low-quality AI outputs; exemplar quality is the single highest-leverage factor in few-shot prompt engineering. |
| REQ-AIX-010 | Exemplar Diversity Requirement | Exemplars within a pack SHALL cover diverse complexity levels (simple / medium / complex scenario), at least two industry verticals (e.g., financial services + manufacturing), and both greenfield and brownfield contexts, to prevent the AI from anchoring on a single scenario type. | AI/ML | ALL | Should | Narrow exemplars cause the AI to over-index on one scenario archetype and produce poorly calibrated responses for edge cases. |
| REQ-AIX-011 | Negative Exemplar Inclusion | Each pack SHALL include at least two negative exemplars (input → poor response + critique of why it is poor) to steer the AI away from common failure modes for that role (e.g., EA giving implementation-level detail, Security Architect skipping threat actors). | AI/ML | ALL | Should | Negative exemplars are a proven technique for reducing systematic blind spots; they also serve as the gold-standard evaluation cases for AIX §7. |

---

### AIX §2 — Per-Role Expert Persona Packs (P1–P14 + Custom) {#aix-2}

#### AIX §2.1 Enterprise Architect Pack (P1)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-012 | EA System Persona Prompt | The EA pack system-persona prompt SHALL instruct the AI to embody a senior Enterprise Architect with 15+ years experience, fluent in TOGAF 10 ADM, Zachman, DoDAF, ArchiMate 3.2, and FEAF; adopting a strategic, business-outcome-first voice; always grounding responses in capability maps and value streams before discussing technology. | AI/ML | P1 | Must | EA role requires business-capability framing first; technology without strategic context is an anti-pattern for this persona. |
| REQ-AIX-013 | EA Heuristics & Frameworks Block | The EA heuristics block SHALL embed: TOGAF ADM phase-gate discipline; Zachman row/column discipline (never conflate perspectives); DoDAF viewpoint selection rationale; business/app/data/tech architecture layering; portfolio lifecycle (TIME/Gartner); the five EA value propositions (alignment, integration, agility, efficiency, risk). | AI/ML | P1 | Must | These are the defining mental models that distinguish EA-quality output from generic architecture output. |
| REQ-AIX-014 | EA Do/Don't Guardrails | The EA guardrails SHALL explicitly prohibit: jumping to technology solutions before defining capability gaps; using implementation-level terminology in strategic deliverables; omitting stakeholder-impact analysis; ignoring regulatory and compliance context; and SHALL require: every recommendation to trace to a business capability or strategic objective; flagging orphaned technology investments. | AI/ML | P1 | Must | EA anti-patterns (technology-first, scope-blind) are the most common quality failure in EA output. |
| REQ-AIX-015 | EA Proactive Checklists | The EA pack SHALL include role-proactive checklist prompts: "what a great EA would ask" (e.g., "Which capabilities are differentiating vs. commodity?", "What is the transition architecture between current and target state?", "How does this align to the strategic plan horizon?"). | AI/ML | P1 | Should | Checklists surface the questions EAs forget to ask under time pressure, raising output quality consistently. |

---

#### AIX §2.2 Solution Architect Pack (P2)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-016 | SA System Persona Prompt | The SA pack SHALL instruct the AI to embody a senior Solution Architect: pragmatic, trade-off-oriented, experienced in translating business problems into architecturally sound designs across cloud, integration, data, and security domains; fluent in C4, ArchiMate, ADRs, HLD/LLD, and NFR specification. | AI/ML | P2 | Must | SA persona is the bridge between strategy and implementation; the AI must reflect that balance and not default to EA abstraction or developer implementation. |
| REQ-AIX-017 | SA Heuristics Block | The SA heuristics block SHALL embed: options-first thinking (never present a single solution without alternatives); YAGNI for scope control; Conway's Law as a design constraint; NFR-as-first-class-citizen; fitness-for-purpose over elegance; total cost of ownership vs. initial cost; "what problem does this solve?" challenge. | AI/ML | P2 | Must | SA quality failures centre on single-option proposals, missing NFRs, and ignoring Conway's Law — these heuristics directly counter those failure modes. |
| REQ-AIX-018 | SA Trade-off Heuristics | The SA pack SHALL embed trade-off-specific heuristics: always name the quality attribute being sacrificed for each design choice; every constraint is a potential trade-off trigger; size before you design (rough-order-of-magnitude capacity). Cross-ref TRD §1–§4 for the trade-off engine that the SA pack activates. | AI/ML | P2 | Must | Trade-off analysis is a signature SA skill; the AI must default to raising trade-offs rather than suppressing them. |
| REQ-AIX-019 | SA Do/Don't Guardrails | SA guardrails SHALL prohibit: presenting a single option as "the solution"; omitting risk or assumption documentation; skipping NFR definition; recommending build over buy without justification; and SHALL require: every ADR to capture alternatives considered; every HLD to include a deployment view. | AI/ML | P2 | Must | Common SA anti-patterns documented in TOGAF and practitioner literature. |

---

#### AIX §2.3 Application Architect Pack (P3)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-020 | AA System Persona Prompt | The AA pack SHALL instruct the AI to embody a senior Application Architect: deep in design patterns (GoF, POSA, EIP), C4 and UML modelling, component decomposition, code-quality heuristics, and the SOLID/DRY/KISS principles; comfortable at both the whiteboard-design and code-review levels. | AI/ML | P3 | Must | Application Architects operate at the design-pattern and component level; generic advice at this level is least valuable because practitioners already know the basics. |
| REQ-AIX-021 | AA Heuristics Block | The AA heuristics block SHALL embed: separation of concerns discipline; dependency inversion as a design default; "strangle the monolith" phased migration heuristic; coupling/cohesion trade-off analysis; ADR as the record of every non-trivial design decision; fitness functions for detecting architectural drift. | AI/ML | P3 | Must | AA quality failures manifest as over-coupling, missing ADRs, and ignoring evolutionary architecture principles. |
| REQ-AIX-022 | AA Do/Don't Guardrails | AA guardrails SHALL require: all component recommendations to include rationale for the pattern chosen; all technology choices to be flagged as ADR candidates; SHALL prohibit: recommending a pattern without naming its trade-offs; generating class diagrams for strategic discussions. | AI/ML | P3 | Must | Pattern-without-trade-off advice is an anti-pattern at the AA level. |

---

#### AIX §2.4 Integration Architect Pack (P4)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-023 | IA System Persona Prompt | The IA pack SHALL instruct the AI to embody a senior Integration Architect: expert in Enterprise Integration Patterns (Hohpe/Woolf), ESB/iPaaS selection, message semantics, idempotency, exactly-once delivery, canonical data models, and event-driven architectures; adopts a "what happens at failure" first mindset. | AI/ML | P4 | Must | Integration Architects must reason from failure modes first; generic assistant mode optimises for the happy path. |
| REQ-AIX-024 | IA Heuristics Block | The IA heuristics block SHALL embed: EIP pattern taxonomy (channel/router/translator/endpoint/system-management); canonical model discipline; choreography vs. orchestration trade-off; consumer-driven contract testing; idempotency and at-least-once vs. exactly-once trade-offs; ESB vs. event-mesh vs. point-to-point decision heuristics. | AI/ML | P4 | Must | EIP vocabulary and failure-mode reasoning are the definitive heuristics for Integration Architect quality. |
| REQ-AIX-025 | IA Do/Don't Guardrails | IA guardrails SHALL require: every integration design to identify the data owner and canonical model; every async pattern to name the delivery guarantee; SHALL prohibit: recommending synchronous HTTP for high-volume integrations without latency/failure analysis; ignoring idempotency in update flows. | AI/ML | P4 | Must | Synchronous-by-default and missing idempotency are the top integration architecture anti-patterns. |

---

#### AIX §2.5 API Architect Pack (P5)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-026 | APIA System Persona Prompt | The APIA pack SHALL instruct the AI to embody a senior API Architect: expert in REST constraints (Fielding), OpenAPI 3.x / AsyncAPI 3.x, GraphQL schema design, gRPC and Protobuf, API lifecycle governance, versioning strategies, and API gateway/policy design. | AI/ML | P5 | Must | API Architect requires both protocol depth and governance breadth; a generic assistant defaults to simplistic REST advice. |
| REQ-AIX-027 | APIA Heuristics Block | The APIA heuristics block SHALL embed: API-first design; resource vs. action-based URI discipline; versioning strategies (URL, header, media-type) and their trade-offs; HATEOAS applicability decision tree; GraphQL vs. REST vs. gRPC selection heuristic; rate-limiting, quotas, and SLA tiers; consumer-driven contract testing. | AI/ML | P5 | Must | Versioning strategy and protocol-selection heuristics are the highest-value, most commonly misapplied API Architect skills. |
| REQ-AIX-028 | APIA Do/Don't Guardrails | APIA guardrails SHALL require: every API design to include a versioning strategy; every breaking-change to be flagged; SHALL prohibit: recommending URL-based versioning as universal default without trade-off discussion; designing APIs without an identified consumer. | AI/ML | P5 | Must | "Version in URL always" and consumer-less APIs are the two top API design anti-patterns in enterprise settings. |

---

#### AIX §2.6 CRM / Platform Architect Pack (P6)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-029 | CRM System Persona Prompt | The CRM pack SHALL instruct the AI to embody a senior CRM/Platform Architect: expert in Salesforce (Sales/Service/Experience Cloud, Apex, LWC, Platform Events, Flows) and/or Microsoft Dynamics 365 CE; platform-extensibility boundaries; integration architecture for CRM ecosystems; and data model governance within vendor-constrained platforms. | AI/ML | P6 | Must | CRM Architect is highly platform-specific; generic advice ignores vendor governor limits, extension-point constraints, and data-model lock-in. |
| REQ-AIX-030 | CRM Heuristics Block | The CRM heuristics block SHALL embed: configuration-before-customisation discipline; governor-limit awareness (SOQL limits, CPU time, heap); event-driven integration via Platform Events over direct callouts; data model normalisation within the platform's multi-tenant constraints; licence-cost implication of customisation choices. | AI/ML | P6 | Must | Governor limits and configuration-vs-customisation are the primary quality dimensions for CRM architecture. |
| REQ-AIX-031 | CRM Do/Don't Guardrails | CRM guardrails SHALL require: every customisation recommendation to be weighed against the equivalent declarative option; every integration pattern to address governor limits; SHALL prohibit: recommending Apex triggers as a first choice over Flow; ignoring upgrade compatibility when recommending customisations. | AI/ML | P6 | Must | Apex-first (over declarative) is the most common expensive CRM anti-pattern. |

---

#### AIX §2.7 Data Architect Pack (P7)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-032 | DA System Persona Prompt | The DA pack SHALL instruct the AI to embody a senior Data Architect: expert in DAMA-DMBOK2 knowledge areas, conceptual/logical/physical data modelling, data mesh and data fabric principles, data lineage, master data management (MDM), data governance frameworks, and modern lakehouse/warehouse architecture (Medallion architecture, Delta Lake, dbt). | AI/ML | P7 | Must | Data Architect requires both governance depth and modern platform breadth; generic advice defaults to relational-first, ignoring data-mesh and governance dimensions. |
| REQ-AIX-033 | DA Heuristics Block | The DA heuristics block SHALL embed: DAMA knowledge area prioritisation per project context; "data as a product" discipline (data mesh); ubiquitous language in data models (DDD alignment); third-normal-form vs. dimensional model selection heuristics; data lineage as a first-class design output; MDM golden-record patterns; data classification (PII/PCI/PHI) as an up-front design constraint. | AI/ML | P7 | Must | DAMA, data mesh, and data classification are the three highest-value DA heuristics missing from generic AI output. |
| REQ-AIX-034 | DA Do/Don't Guardrails | DA guardrails SHALL require: every data model to include a lineage diagram; every data source to have a classification (PII/PCI/PHI/open); SHALL prohibit: recommending a data warehouse schema without addressing ETL/ELT latency requirements; designing data products without named consumers and SLAs. | AI/ML | P7 | Must | Missing lineage and data classification are the most common DA audit failures. |

---

#### AIX §2.8 Security Architect Pack (P8)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-035 | SecA System Persona Prompt | The SecA pack SHALL instruct the AI to embody a senior Security Architect: expert in TOGAF/SABSA, STRIDE threat modelling, NIST CSF/SP 800-53, Zero-Trust architecture, MITRE ATT&CK, OWASP Top-10 and ASVS, cloud-security (CIS benchmarks, CSPM), and IAM (OAuth2/OIDC/SCIM); adopts an adversary-first mindset. | AI/ML | P8 | Must | Security Architect must reason from attacker perspective first; generic assistants default to control-first which misses novel attack paths. |
| REQ-AIX-036 | SecA Heuristics Block | The SecA heuristics block SHALL embed: STRIDE-per-element threat modelling workflow; Zero-Trust "never trust, always verify" as a design default; defence-in-depth layering (prevent/detect/respond); "assume breach" posture for detection design; NIST CSF Identify→Protect→Detect→Respond→Recover scaffolding; security-by-design vs. security-bolt-on cost ratio (1:10:100 rule). | AI/ML | P8 | Must | These are the canonical heuristics that distinguish Security Architect quality output from generic security advice. |
| REQ-AIX-037 | SecA Do/Don't Guardrails | SecA guardrails SHALL require: every architecture recommendation to include a threat model entry; every sensitive data flow to be identified; every authentication design to name the protocol and grant type; SHALL prohibit: recommending security controls without naming the threat they mitigate; endorsing "VPN as perimeter" without Zero-Trust trade-off discussion; omitting security NFRs from any design review. | AI/ML | P8 | Must | Security-control-without-threat and VPN-as-sufficient-perimeter are the top two SecA anti-patterns. |

---

#### AIX §2.9 Cloud Architect Pack (P9)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-038 | CA System Persona Prompt | The CA pack SHALL instruct the AI to embody a senior Cloud Architect: expert in AWS/Azure/GCP Well-Architected Frameworks (all six pillars), landing-zone design, cloud-native service selection, FinOps, IaC (Terraform/Bicep/CDK), multi-cloud and hybrid topology, and cloud governance frameworks (Azure Policy, AWS Control Tower, GCP Organization Policy). | AI/ML | P9 | Must | Cloud Architect spans three hyperscalers plus FinOps; generic advice defaults to a single cloud and ignores FinOps and governance. |
| REQ-AIX-039 | CA Heuristics Block | The CA heuristics block SHALL embed: Well-Architected Pillars scoring as a design gate; landing-zone-first discipline; immutable infrastructure principle; FinOps Unit-Economics framing (cost per unit of value); blast-radius minimisation via account/subscription isolation; managed-services-before-self-managed heuristic; multi-region RTO/RPO as NFR-drivers. | AI/ML | P9 | Must | Well-Architected and FinOps framing are the highest-value Cloud Architect heuristics. |
| REQ-AIX-040 | CA Do/Don't Guardrails | CA guardrails SHALL require: every cloud design to include an estimated monthly cost and cost-optimization path; every stateful workload to have a named backup/DR strategy; SHALL prohibit: recommending a managed service without noting the vendor lock-in trade-off; presenting a single cloud design without multi-cloud or exit-strategy consideration. | AI/ML | P9 | Must | Missing cost estimates and lock-in analysis are the top Cloud Architect anti-patterns. |

---

#### AIX §2.10 Infrastructure / Network Architect Pack (P10)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-041 | Infra System Persona Prompt | The Infra pack SHALL instruct the AI to embody a senior Infrastructure/Network Architect: expert in data-centre topology, SD-WAN, BGP routing, VLAN/VXLAN segmentation, HA/DR patterns (active-active, active-passive), ITIL service design, capacity planning, and hybrid-cloud network design (ExpressRoute/Direct Connect/Interconnect). | AI/ML | P10 | Must | Infrastructure and network design requires topology and protocol depth that generic AI flattens to high-level diagrams. |
| REQ-AIX-042 | Infra Heuristics Block | The Infra heuristics block SHALL embed: HA/DR topology selection heuristic (RTO/RPO drives active-active vs. active-passive); network segmentation as security boundary (not just performance); infrastructure-as-code as the implementation target for all designs; capacity-headroom 20% rule; single-point-of-failure (SPOF) identification checklist; out-of-band management requirement for all production tiers. | AI/ML | P10 | Must | SPOF identification and RTO/RPO-driven topology are the highest-value Infra Architect heuristics. |
| REQ-AIX-043 | Infra Do/Don't Guardrails | Infra guardrails SHALL require: every topology to identify all SPOFs; every DR design to specify RTO/RPO targets; SHALL prohibit: designing a network topology without security-zone definitions; recommending capacity without utilisation headroom analysis. | AI/ML | P10 | Must | Missing SPOF and DR specifications are the top Infra Architect design failures. |

---

#### AIX §2.11 Microservices / Platform / DevOps Architect Pack (P11)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-044 | MSA System Persona Prompt | The MSA pack SHALL instruct the AI to embody a senior Microservices/Platform/DevOps Architect: expert in DDD bounded contexts, service-mesh (Istio/Linkerd/Consul), Kubernetes (operators, HPA, multi-cluster), GitOps (Flux/ArgoCD), CI/CD pipeline design, observability (OpenTelemetry, Prometheus/Grafana/Loki), SRE practices, and platform engineering. | AI/ML | P11 | Must | Microservices/DevOps Architect operates at the platform-engineering level; generic advice defaults to "just use Kubernetes" without DDD, observability, or SRE grounding. |
| REQ-AIX-045 | MSA Heuristics Block | The MSA heuristics block SHALL embed: DDD bounded-context-first service decomposition (anti-pattern: decompose by technical layer); "you build it, you run it" platform discipline; four golden signals (latency/traffic/errors/saturation) as observability baseline; GitOps single-source-of-truth discipline; "toil reduction" as the primary platform engineering metric; SLO-as-a-social-contract heuristic. | AI/ML | P11 | Must | DDD decomposition and SRE golden signals are the highest-value MSA heuristics missing from generic advice. |
| REQ-AIX-046 | MSA Do/Don't Guardrails | MSA guardrails SHALL require: every service boundary to be justified by DDD bounded context; every platform design to include observability requirements; SHALL prohibit: decomposing by technical layer (UI/BL/DB) as a microservices pattern; designing a CI/CD pipeline without a rollback strategy. | AI/ML | P11 | Must | Technical-layer decomposition is the single most common microservices anti-pattern. |

---

#### AIX §2.12 Product Manager Pack (P13)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-047 | PM System Persona Prompt | The PM pack SHALL instruct the AI to embody a senior Product Manager: expert in outcome-based product strategy, RICE/Kano/WSJF prioritisation, BRD/PRD authoring, OKR alignment, Jobs-to-be-Done (JTBD) framework, stakeholder management, and the product-to-delivery handoff to architecture. | AI/ML | P13 | Must | PM persona requires business-outcome and prioritisation-framework depth that generic AI does not provide without explicit framing. |
| REQ-AIX-048 | PM Heuristics Block | The PM heuristics block SHALL embed: "outcome over output" discipline; WSJF cost-of-delay framing for backlog prioritisation; JTBD for requirements framing (jobs, pains, gains); MVP scoping heuristic (smallest testable hypothesis); "who is this for?" as the first question for every requirement; OKR alignment check before roadmap commitment. | AI/ML | P13 | Must | Outcome-over-output and WSJF are the highest-value PM heuristics that generic AI consistently underweights. |
| REQ-AIX-049 | PM Do/Don't Guardrails | PM guardrails SHALL require: every requirement to trace to a customer problem or business outcome; every roadmap item to have a success metric; SHALL prohibit: writing solution-specification requirements (how) instead of problem requirements (what/why); accepting a feature request without asking "what job does this help the user do?". | AI/ML | P13 | Must | Solution-specification requirements are the most expensive PM anti-pattern (they lock architects into one design approach). |

---

#### AIX §2.13 "Any Architect" Extensibility Pack (P12 Custom)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-050 | Custom Pack Creation Wizard | The suite SHALL provide a guided wizard for creating a custom persona prompt pack, prompting the admin to specify: the target role name and description, up to 10 key frameworks/standards the role uses, 5 core heuristics, 5 guardrail rules (3 must-do + 2 must-not), and 3 exemplar input/output pairs; the wizard SHALL generate a valid baseline pack. | Functional | P12 | Must | Without a guided creation path, custom packs will be created inconsistently, degrading AI output quality for custom personas. Cross-ref V11 REQ-PLT-PRC-003. |
| REQ-AIX-051 | Inherited Pack Composition | A custom pack SHALL be able to inherit all components from a base persona pack (P1–P11) and override only the components that differ; inheritance SHALL be version-locked to the parent pack version at inheritance time, with an upgrade-check notification when the parent pack is updated. | Functional | P12 | Must | Inheritance reduces custom-pack creation effort from scratch to a delta, lowering the barrier for role extension. Cross-ref V11 REQ-PLT-PRC-005. |
| REQ-AIX-052 | Custom Pack Testing Sandbox | The suite SHALL provide a sandbox environment where an admin can test a custom pack by submitting test prompts and reviewing the AI's response before activating the pack; the sandbox SHALL display the assembled prompt (system persona + heuristics + exemplars + guardrails) for transparency. | Functional | P12 | Should | Admins need to validate custom packs before exposing them to users; prompt visibility prevents debugging-in-production. |

---

#### AIX §2.14 Presales / Bid Architect Pack (P14)

> **New persona introduced in V21.** P14 Presales/Bid Architect covers the pre-contract phase: qualifying opportunities, crafting technical proposals and RFP responses, designing proof-of-concept architectures, and translating architecture into win-themes for commercial bids. Cross-ref V11 metamodel for persona registration; V13 for licence type definition (future sprint).

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-053 | Presales System Persona Prompt | The Presales pack SHALL instruct the AI to embody a senior Presales/Bid Architect: expert in solution-brief-to-proposal translation, RFP/ITT/RFI response structure, win-theme articulation, proof-of-concept (PoC) architecture design, TCO/ROI framing for commercial audiences, and the Shipley/APMP bid-management lifecycle. | AI/ML | P14 | Must | Presales/Bid Architect combines deep technical credibility with commercial narrative skill; no existing persona covers the bid-writing and PoC-architecture intersection. |
| REQ-AIX-054 | Presales Heuristics Block | The Presales heuristics block SHALL embed: ghost-writing the customer's evaluation criteria before writing any proposal section; win-theme identification (3–5 differentiated themes per bid); PoC scope minimisation (smallest architecture that proves the win-theme); Shipley colour-review discipline (Pink/Red/Gold); TCO framing over 3/5-year horizon as a competitive differentiator; risk-section discipline (acknowledge risk proactively). | AI/ML | P14 | Must | Presales Architects who lose bids most often fail on win-theme articulation and scope-inappropriate PoC designs; these heuristics directly address those failure modes. |
| REQ-AIX-055 | Presales Do/Don't Guardrails | Presales guardrails SHALL require: every proposal section to explicitly address a stated customer evaluation criterion; every PoC to have a defined success criterion linked to the win-theme; SHALL prohibit: writing architecture detail beyond what is needed to establish technical credibility in the proposal stage; presenting a PoC design as a production-ready architecture. | AI/ML | P14 | Must | Over-engineering proposal architecture and misrepresenting PoC as production architecture are the most common Presales Architect quality failures. |
| REQ-AIX-056 | Presales Win-Theme Generator | The suite SHALL provide a Presales-specific AI capability that, given an uploaded RFP/ITT and a solution brief, generates 3–5 candidate win-themes with supporting evidence from the knowledge base, each linked to a customer evaluation criterion and a differentiating capability. Cross-ref V08 (Knowledge Hub RAG). | AI/ML | P14 | Should | Win-theme generation from RFP analysis is the highest-ROI Presales AI capability; manual analysis takes 4–8 hours per bid. |

---

### AIX §3 — Prompt-Pack Governance, Versioning & Administration {#aix-3}

#### AIX §3.1 Lifecycle & Versioning

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-057 | Semantic Versioning for Packs | Every prompt pack SHALL carry a semantic version (MAJOR.MINOR.PATCH) where MAJOR increments indicate breaking changes to exemplars or guardrails, MINOR indicate additive improvements, and PATCH indicate typo/wording corrections; the suite SHALL display the active pack version to users in the AI panel. | Governance | ALL | Must | Users and admins need to understand what changed when AI behaviour shifts; semver enables meaningful change communication. |
| REQ-AIX-058 | Pack Change Log | Every pack version SHALL include a structured change log entry: changed components, nature of change, author, date, rationale, and a link to the evaluation run that validated the change. | Governance | ALL | Must | Change log without validation link creates an untraceable audit trail; evaluation linkage closes the governance loop. |
| REQ-AIX-059 | Pack Review and Approval Workflow | Promoting a pack from Draft to Active SHALL require at least one reviewer (configurable: human reviewer, LLM-judge, or both) to approve; the suite SHALL record reviewer identity, review date, and approval decision; automatic promotion SHALL be blocked without approval. | Governance | ALL | Must | Unapproved pack promotion is a prompt-injection risk and a quality risk; the review gate is non-negotiable. |
| REQ-AIX-060 | Rollback to Previous Pack Version | The suite SHALL allow a platform admin or tenant admin to roll back the active prompt pack to any previous version within 60 seconds, with automatic notification to affected users; rollback SHALL not require a platform redeployment. | Governance | ALL | Must | Prompt changes can cause sudden AI quality regressions; a fast rollback path is the safety net. |
| REQ-AIX-061 | Pack Deprecation and Sunset | Marking a pack as Deprecated SHALL: (a) prevent new pack-persona associations; (b) trigger a notification to all users currently using the pack; (c) allow existing users a configurable grace period (default 30 days) before forced migration to the successor pack. | Governance | ALL | Must | Hard deprecation without grace periods causes user disruption; soft deprecation with migration paths is standard platform practice. |

---

#### AIX §3.2 Administrative Configuration Hierarchy

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-062 | Three-Tier Pack Override Hierarchy | The suite SHALL support a three-tier prompt-pack configuration hierarchy: (1) **Platform default** (ESIP-authored, updated by product team); (2) **Tenant override** (tenant admin creates or imports a custom pack that overrides the platform default for that tenant); (3) **Project/user override** (a project owner or user selects a non-default pack for a specific project or session). Lower tiers override higher tiers; the platform SHALL display which tier's pack is active at all times. | Governance | ALL | Must | Enterprise customers need to customise AI behaviour for their domain/brand/compliance constraints without modifying platform defaults for other tenants. Cross-ref V11 metamodel and V14 personalization. |
| REQ-AIX-063 | Tenant Admin Pack Import/Export | Tenant admins SHALL be able to import and export prompt packs as structured JSON/YAML artefacts, enabling version-controlled storage in the tenant's own repository (architecture-as-code, cross-ref V19 REQ-ENT-XXX). | Governance | ALL | Should | Architecture-as-code discipline requires that prompt packs are treated as versioned configuration, not opaque platform state. |
| REQ-AIX-064 | Platform Pack Marketplace | The suite SHALL support a curated Pack Marketplace where ESIP, partners, and tenant admins can publish and subscribe to prompt packs; each marketplace pack SHALL carry: author, version, trust-level (ESIP-verified / Partner / Community), download count, and user rating. Cross-ref V16 marketplace and V19 partner program. | Functional | ALL | Could | A pack marketplace accelerates the quality of role-specific AI for niche roles (e.g., TOGAF-specific EA, SABSA-specific SecA) by crowdsourcing expert curation. |
| REQ-AIX-065 | Tenant-Specific Vocabulary Injection | The suite SHALL allow a tenant admin to define a tenant-specific vocabulary glossary (domain terms, acronyms, product names) that is injected into every active prompt pack for that tenant, so the AI uses the tenant's language rather than generic terms. | Functional | ALL | Should | Enterprises have proprietary vocabulary; AI that uses generic terms in proposals or architecture reviews appears uncalibrated. |
| REQ-AIX-066 | Pack A/B Testing | The suite SHALL support A/B testing of prompt packs (split by user cohort or project type), with configurable metrics (user feedback score, AI-evaluation score, task completion rate); results SHALL be reportable to the admin dashboard. Cross-ref V06 §14 (feedback loop) for the evaluation pipeline. | Governance | ALL | Could | A/B testing is the principled method for validating prompt improvements before full rollout; without it, pack quality improvements are based on intuition. |

---

### AIX §4 — Active-Persona Auto-Adoption {#aix-4}

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-067 | Persona-Driven AI Auto-Adoption | When a user switches the active persona (P1–P14 or custom) in the ESIP UI, the suite SHALL automatically activate the corresponding expert prompt pack for all subsequent AI interactions in that session without requiring the user to reconfigure anything. Cross-ref V14 PERS (active persona state) and V11 PLT (persona assignment). | Functional | ALL | Must | Manual pack selection breaks the "AI as co-architect" UX promise; auto-adoption must be seamless. |
| REQ-AIX-068 | Context-Aware Pack Activation | Within an active persona, the suite SHALL further select the most contextually appropriate sub-pack or pack overlay based on: the current capability area (e.g., EA–strategy vs. EA–migration), the current artifact type being edited (e.g., threat model vs. HLD), and the current workflow step. | AI/ML | ALL | Should | A flat persona-to-pack mapping misses intra-role context variation (an EA doing a migration planning session needs different heuristics than one doing a capability strategy). |
| REQ-AIX-069 | Active Pack Transparency Display | The suite SHALL display the name, version, and tier (platform/tenant/project) of the currently active prompt pack to the user in the AI panel, with a one-click path to view its components (read-only for non-admins). | UX | ALL | Must | Transparency about which pack is active is required for user trust and for debugging unexpected AI behaviour. |
| REQ-AIX-070 | Persona Switch AI Context Flush | When the active persona changes, the suite SHALL flush the in-session AI context window (excluding persisted user memories from V14) to prevent the previous persona's context from contaminating the new persona's expert mode. | AI/ML | ALL | Must | Context contamination from a previous persona produces hybridised advice that is neither-role-specific, degrading output quality. |
| REQ-AIX-071 | Role-Conditioned Greeting & Orientation | On first AI interaction in a new persona context, the AI SHALL generate a brief orientation statement (≤ 60 words) indicating its active role-persona mode and the 2–3 highest-priority concerns it will flag proactively in that mode. | UX | ALL | Should | An orientation statement sets user expectations for AI behaviour and signals that expert-persona mode is active, differentiating from generic assistant mode. |

---

### AIX §5 — Proactive Role-Specific Suggestions & Nudges {#aix-5}

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-072 | Proactive Role-Specific Suggestions | The AI SHALL proactively surface role-appropriate suggestions in the current work context without being explicitly asked: e.g., an EA working on a capability map is nudged to check for orphaned applications; an SA drafting an HLD is nudged to add NFRs; a SecA reviewing an integration diagram is nudged to create a STRIDE threat entry. Cross-ref V16 INT (prescriptive next-best-action at portfolio level). | AI/ML | ALL | Must | Reactive-only AI requires the architect to know all the right questions; proactive nudges replicate the "senior colleague looking over your shoulder" experience. |
| REQ-AIX-073 | Role-Specific Checklist Triggers | The suite SHALL trigger role-appropriate checklists at defined workflow points (e.g., before ADR finalisation, the SA checklist fires; before solution brief approval, the PM checklist fires; before threat model completion, the SecA checklist fires); each checklist SHALL be editable and versioned as part of the prompt pack. | AI/ML | ALL | Must | Workflow-point checklists are the highest-reliability mechanism for ensuring role-specific quality gates are applied consistently. |
| REQ-AIX-074 | Anti-Pattern Detection Nudge | The AI SHALL detect role-specific anti-patterns in the active artifact (e.g., CRM Architect uses Apex where Flow is available; EA describes solution-level detail in a business architecture; MSA decomposes by technical layer) and SHALL surface a nudge with the identified anti-pattern, its consequence, and the recommended alternative. | AI/ML | ALL | Must | Anti-pattern nudges are the most concrete, actionable form of role-expertise the AI can deliver; they prevent expensive rework. |
| REQ-AIX-075 | Missing Artifact Nudge | When the AI detects that a role-signature artifact is absent from the project (e.g., a P7 Data Architect project has no data lineage diagram; a P4 Integration Architect project has no canonical data model; a P8 Security Architect project has no threat model), it SHALL surface a missing-artifact nudge with a one-click "create this artifact" action. | AI/ML | ALL | Should | Missing artifacts are systematic quality gaps; surfacing them at the right moment (not in a bulk report) maximises the chance of remediation. |
| REQ-AIX-076 | Nudge Frequency & Dismiss Control | Users SHALL be able to control the frequency of proactive nudges (Low / Medium / High) and to dismiss individual nudge types permanently or for the current session; dismissed nudges SHALL be recordable as V14 personalization preferences. | UX | ALL | Must | Excessive nudges cause habituation and dismissal fatigue; user control is required for sustained engagement. Cross-ref V14 §13. |
| REQ-AIX-077 | Cross-Persona Suggestion | When an artifact is being reviewed by a persona different from its author's persona (e.g., a Security Architect reviewing a Solution Architect's HLD), the AI SHALL adopt a cross-role expert lens, surfacing suggestions that a SecA would make when reviewing an SA deliverable, not generic review comments. | AI/ML | ALL | Should | Cross-persona review is one of the highest-value use cases; the AI must shift its expert lens to the reviewer persona, not the author persona. |

---

### AIX §6 — Grounding, Citations & Evidence Linking {#aix-6}

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-078 | Standard / Framework Citation | When the AI invokes a heuristic or guideline that originates from a published standard or framework (TOGAF, NIST, DAMA-DMBOK, EIP, SABSA, CIS, ISO/IEC, etc.), it SHALL cite the source (framework name + section/reference) inline with the recommendation. | AI/ML | ALL | Must | Architects need to know whether a recommendation is an AI opinion or a recognised standard; citations enable verification and increase trust. |
| REQ-AIX-079 | Knowledge-Base Grounding | AI recommendations SHALL be grounded in the project's knowledge base (V08 RAG) wherever relevant context exists: the AI SHALL reference the specific document, meeting, or requirement that informed the recommendation. Cross-ref V06 §9 (RAG over Knowledge Hub). | AI/ML | ALL | Must | Recommendations ungrounded in project context are generic; grounded recommendations demonstrate that the AI has "read the brief." |
| REQ-AIX-080 | Confidence and Assumption Flagging | When the AI lacks sufficient context to ground a recommendation fully, it SHALL explicitly flag the recommendation as based on an assumption and name the assumption; it SHALL NOT present assumed context as verified fact. | AI/ML | ALL | Must | Presenting assumptions as facts is the most dangerous AI failure mode for architects who may rely on outputs without verification. |
| REQ-AIX-081 | External Evidence Linking | For trade-off recommendations and technology comparisons, the AI SHALL, where available, link to publicly accessible reference documents (vendor documentation, RFC, framework specification) to support its reasoning; links SHALL be stored as provenance metadata on the recommendation, not embedded in prose (to avoid link rot impacting audit trails). | AI/ML | ALL | Should | External evidence raises recommendation credibility and reduces the research burden on the architect. |

---

### AIX §7 — Role-Expertise Evaluation & Quality Assurance {#aix-7}

#### AIX §7.1 Automated Evaluation Pipeline

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-082 | Role-Expertise Evaluation Harness | The suite SHALL include an automated evaluation harness that tests every prompt pack against a set of role-specific evaluation cases before the pack is promoted to Active status; evaluation cases SHALL be stored, versioned, and runnable on demand. Cross-ref V06 §11 (guardrails & quality) for the general AI eval infrastructure. | AI/ML | ALL | Must | Untested packs are a quality and trust risk; automated pre-promotion testing is the minimum viable quality gate. |
| REQ-AIX-083 | Role-Expertise Scoring Rubric | The evaluation harness SHALL score AI responses against a role-expertise rubric with at minimum four dimensions: (1) **Framework Accuracy** — does the AI correctly apply the role's frameworks and standards? (2) **Practitioner Voice** — does the response sound like a senior practitioner of the role? (3) **Anti-Pattern Avoidance** — does the response avoid the role's documented anti-patterns? (4) **Completeness** — does the response cover the checklist items a great practitioner would cover? | AI/ML | ALL | Must | A rubric with named dimensions produces actionable diagnostic feedback; a single aggregate score does not. |
| REQ-AIX-084 | LLM-as-Judge Evaluation | The evaluation harness SHALL support using a separate LLM-as-judge (configurable model, cross-ref V06 §11) to score AI responses against the rubric for each evaluation case; the judge model SHALL be different from the model being evaluated to prevent self-scoring bias. | AI/ML | ALL | Must | LLM-as-judge is the state-of-the-art scalable evaluation method; human-only review does not scale to frequent pack updates. |
| REQ-AIX-085 | Human Expert Calibration Loop | For each role persona, the suite SHALL support a human expert calibration loop where a designated subject-matter expert (configurable per tenant) reviews a sample of AI outputs against the rubric and their scores are used to calibrate the LLM-judge's rubric interpretation. | AI/ML | ALL | Should | LLM-judge calibration against human expert scores is required to prevent the judge from systematically rewarding fluency over correctness. |
| REQ-AIX-086 | Regression Detection on Pack Update | When a new pack version is promoted, the evaluation harness SHALL compare scores against the previous version's baseline; a regression of more than 5% on any rubric dimension SHALL block promotion and notify the pack author. | Governance | ALL | Must | Without regression detection, pack improvements can inadvertently degrade specific role-expertise dimensions. |
| REQ-AIX-087 | Evaluation Dashboard | The suite SHALL provide an admin-accessible Evaluation Dashboard showing: current pack versions by persona, latest evaluation scores per rubric dimension, trend over time, regression alerts, and pending human-review items. | Reporting | ALL | Should | Visibility into prompt-pack quality over time is required for a principled quality governance practice. |

---

#### AIX §7.2 User Feedback Collection

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-088 | In-Line AI Response Feedback | Every AI response in expert-persona mode SHALL include a lightweight feedback control (thumbs up/down + optional tag from a predefined list: "not role-appropriate", "missing framework reference", "too generic", "incorrect", "hallucination") and the feedback SHALL be stored with the active pack version and persona context. | UX | ALL | Must | In-line feedback at point-of-response has 3–5× higher response rate than end-of-session surveys; pack-version tagging makes the data actionable. |
| REQ-AIX-089 | Feedback Aggregation per Pack Version | The suite SHALL aggregate feedback signals per pack version and surface a quality score (positive / total) per pack per persona on the admin Evaluation Dashboard; trends that fall below a configurable threshold SHALL trigger an automatic quality alert to the pack owner. | Governance | ALL | Must | Aggregated feedback is the production signal for pack quality; threshold alerts prevent silent quality degradation. |
| REQ-AIX-090 | Feedback-Driven Improvement Queue | Negative feedback items with a tag SHALL automatically populate a Pack Improvement Queue visible to the pack author; each queue item SHALL include: the original prompt, the AI response, the feedback tag, and a link to the relevant rubric dimension. | Governance | ALL | Should | A structured improvement queue closes the feedback loop; without it, negative feedback is collected but not acted on. |

---

### AIX §8 — Safe Fallback & Escalation {#aix-8}

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-AIX-091 | Graceful Fallback to Base Model | If no active prompt pack is found for the current persona (e.g., pack load failure, tenant pack misconfiguration), the AI SHALL fall back to a general-purpose architecture assistant mode and SHALL display a visible warning banner indicating that expert-persona mode is unavailable. | NFR | ALL | Must | Silent fallback to generic mode without user awareness is a trust violation; the user must know the AI is not operating in expert mode. |
| REQ-AIX-092 | Out-of-Scope Escalation | When the AI detects that the user's request is outside the expertise domain of the active persona pack (e.g., an EA persona is asked to write a database migration script), the AI SHALL acknowledge the out-of-scope request, explain it is operating as a [persona name] expert, and offer to either answer in general-purpose mode or suggest the appropriate persona to switch to. | UX | ALL | Must | Answering out-of-scope questions in persona mode without flagging out-of-scope degrades trust by creating the impression the AI is a universal expert. |
| REQ-AIX-093 | Uncertainty Acknowledgement | When the AI cannot ground a recommendation in known frameworks or project context with sufficient confidence, it SHALL explicitly state the uncertainty, name the information it would need to be more confident, and avoid presenting speculative responses as definitive guidance. | AI/ML | ALL | Must | Overconfident AI responses in architecture contexts can drive expensive incorrect decisions; calibrated uncertainty is more valuable than false certainty. |
| REQ-AIX-094 | Pack Load Failure Recovery | If a prompt pack fails to load (configuration error, token-budget exceeded, provider error), the suite SHALL log the failure with full diagnostic detail, alert the tenant admin, and activate the fallback mode (AIX-091) without interrupting the user's session. | NFR | ALL | Must | Silent pack failures produce degraded AI output without any diagnostic trail, making them the hardest class of quality bug to detect and fix. |

---

## REQ-TRD — Trade-off Analysis Engine

---

### TRD §1 — Elicitation Interview Mode {#trd-1}

#### TRD §1.1 Trade-off Subject Definition

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-TRD-001 | Trade-off Subject Types | The trade-off engine SHALL support the following subject types: (a) Technology Choice (e.g., "PostgreSQL vs. MongoDB vs. DynamoDB"); (b) Architecture Style / Pattern (e.g., "event-driven vs. request-reply"); (c) Vendor / Build-vs-Buy (e.g., "build custom integration vs. adopt iPaaS"); (d) Deployment Model (e.g., "SaaS vs. VPC-deploy vs. on-prem"); (e) ADR Subject (a decision already being recorded in the ADR workflow); (f) NFR Tension (e.g., "consistency vs. availability under the CAP theorem"); (g) Free-text subject (any decision defined by the user). | Functional | ALL | Must | A fixed subject taxonomy ensures the elicitation interview asks the right questions per subject type and enables trade-off library indexing. |
| REQ-TRD-002 | AI-Guided Elicitation Interview | For any new trade-off, the AI SHALL conduct a structured elicitation interview that asks a minimum of six context-framing questions covering: (1) what decision must be made and by when; (2) the principal stakeholders and their top concerns; (3) the constraints that are non-negotiable; (4) the quality attributes that matter most in priority order; (5) what has already been ruled out and why; (6) what a "good enough" outcome looks like. The interview SHALL be completable in under 5 minutes for experienced users (estimated answer length per question ≤ 30 words). | AI/ML | ALL | Must | Elicitation quality determines matrix quality; a 5-question frame captures the minimum context needed to generate a non-trivial trade-off matrix. ATAM elicitation practice. |
| REQ-TRD-003 | Adaptive Question Branching | The elicitation interview SHALL use adaptive branching: based on the subject type and prior answers, the AI SHALL generate follow-up questions tailored to the specific context (e.g., if the subject is a vendor choice, ask about total cost of ownership horizon; if it is an architecture style, ask about team skill set). | AI/ML | ALL | Should | Adaptive branching produces more contextually relevant matrices than fixed question sets; it also reduces unnecessary questions for experienced practitioners. |
| REQ-TRD-004 | Elicitation Skip / Prefill | Users SHALL be able to skip the elicitation interview and prefill the context manually via a structured form (subject, options, criteria, constraints, quality-attribute priority weights); the AI SHALL validate completeness of the prefill before generating the matrix. | UX | ALL | Should | Experienced practitioners often already have the context structured in their head; forcing an interview is friction. Prefill caters to power users. |
| REQ-TRD-005 | Context Import from Project | The elicitation SHALL offer to pre-populate context fields from the current project's knowledge base: requirements, NFRs, constraints, stakeholder register, and existing ADRs; the user SHALL be able to accept or override each pre-populated field. | AI/ML | ALL | Must | Populating from project context reduces elicitation time and ensures the trade-off is grounded in the project's stated constraints, not hypothetical ones. Cross-ref V07 (requirements) and V08 (knowledge hub). |

---

#### TRD §1.2 Options Elicitation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-TRD-006 | Minimum Options Enforcement | The trade-off engine SHALL require at least two options before generating a matrix; it SHALL recommend three to five options as best practice and warn if more than seven options are specified (diminishing marginal value of additional options in weighted scoring). | Functional | ALL | Must | A trade-off matrix over a single option is not a trade-off; two options is the minimum for a meaningful comparison. |
| REQ-TRD-007 | AI-Suggested Options | After the elicitation interview, the AI SHALL suggest additional options that the user may not have considered, drawn from: (a) the trade-off library (TRD §7); (b) technology pattern knowledge; (c) analogous projects in the knowledge base. Each suggestion SHALL be annotated with "why this option is worth considering." | AI/ML | ALL | Must | Anchoring bias causes architects to consider only the options they already know; AI-suggested options counteract this bias. |
| REQ-TRD-008 | Option Description Standard | Each option SHALL capture: name, short description (≤ 100 words), key characteristics, known constraints or prerequisites, estimated implementation complexity (Low/Medium/High), and any known real-world precedents or case study references. | Functional | ALL | Must | Structured option descriptions ensure the scoring phase is based on consistent information per option, not impression. |

---

#### TRD §1.3 Quality-Attribute Weight Elicitation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-TRD-009 | Standard Quality-Attribute Taxonomy | The trade-off engine SHALL provide a standard quality-attribute (criteria) taxonomy covering at minimum: Performance / Latency; Scalability / Elasticity; Availability / Reliability; Security; Maintainability / Evolvability; Operational Complexity; Time-to-Market / Implementation Speed; Cost (CapEx + OpEx + TCO); Vendor Risk / Lock-in; Developer / Operator Experience; Compliance / Regulatory Fit; Interoperability. The taxonomy SHALL be extensible by tenant admin. | Functional | ALL | Must | A standard taxonomy ensures comparable matrices across projects; extensibility covers domain-specific criteria not in the base set. Cross-ref ISO/IEC 25010. |
| REQ-TRD-010 | Weight Assignment Interface | The suite SHALL provide a weight assignment interface where the user assigns a relative importance weight (1–10 or percentage allocation summing to 100) to each applicable quality attribute; the AI SHALL pre-suggest weights based on the elicitation interview answers and the active persona's typical priorities. | UX | ALL | Must | Weights encode the decision-making context; pre-suggested weights reduce cognitive load while remaining overrideable. |
| REQ-TRD-011 | Stakeholder-Differentiated Weights | The suite SHALL support recording multiple stakeholder weight profiles for the same trade-off (e.g., CTO profile weights performance first; CFO profile weights cost first); the weighted score SHALL be computable for each stakeholder profile independently. | Functional | ALL | Should | Different stakeholders optimise for different quality attributes; showing the winning option changes by stakeholder profile is one of the most persuasive artefacts in a trade-off presentation. |

---

### TRD §2 — Trade-off Matrix Generation {#trd-2}

#### TRD §2.1 Matrix Structure & Content

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-TRD-012 | Trade-off Matrix Generation | The AI SHALL generate a structured trade-off matrix (options as rows, quality attributes / criteria as columns) where each cell contains: (a) a score (1–5 scale); (b) a one-sentence rationale for the score; (c) evidence sources (project knowledge base reference, standard, or AI knowledge); and (d) a flag if the score is based on an assumption rather than verified fact. | AI/ML | ALL | Must | Cell-level rationale and evidence are what distinguish a trade-off matrix from an opinion table; without them, the matrix cannot be reviewed or challenged. |
| REQ-TRD-013 | Score Calibration Guidance | The suite SHALL provide a score calibration guide visible during matrix generation, defining what score 1, 3, and 5 mean for each quality attribute (e.g., for Performance: 1 = high latency, unsuitable for SLA; 3 = adequate for most workloads; 5 = sub-millisecond, industry-leading). | AI/ML | ALL | Must | Without calibration guidance, scores are subjective and non-comparable across matrices or reviewers. |
| REQ-TRD-014 | Constraint-Based Option Disqualification | The trade-off engine SHALL, before scoring, check each option against the non-negotiable constraints identified in elicitation; options that fail a hard constraint SHALL be marked as "Disqualified" with the constraint cited, and SHALL remain visible in the matrix with a visual indicator so the decision trail is preserved. | Functional | ALL | Must | Removing disqualified options deletes the audit trail; marking them preserves the reasoning for why they were not further considered. |
| REQ-TRD-015 | Matrix Completeness Validation | The suite SHALL validate that the generated matrix has: all option × criterion cells populated; at least one evidence source per option; no disqualification without a cited constraint; and SHALL surface a completeness report with actionable gaps before the matrix is saved. | Governance | ALL | Must | Incomplete matrices (cells without scores or rationale) undermine the credibility of the trade-off artefact. |
| REQ-TRD-016 | AI Score Challenge Mode | The suite SHALL provide a "challenge this score" mechanism where a user can question a specific cell score; the AI SHALL respond with additional supporting evidence or revise the score with a documented rationale for the revision, and the revision history SHALL be preserved. | AI/ML | ALL | Should | Score challenge mode transforms the matrix from a static AI output into an auditable, reviewed artefact; it also surfaces AI errors before they become ADR inputs. |
| REQ-TRD-017 | Matrix Versioning | The trade-off matrix SHALL be versioned; every cell edit, score revision, or criteria change SHALL create a new version with author, timestamp, and a diff of changed cells. | Governance | ALL | Must | Matrices are frequently revised during architecture reviews; version history is required for audit and for understanding how the recommendation evolved. |

---

#### TRD §2.2 Qualitative Trade-off Narratives

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-TRD-018 | Per-Option Trade-off Narrative | For each option in the matrix, the AI SHALL generate a structured narrative (≤ 150 words) describing: (a) the option's principal strength relative to alternatives; (b) its principal weakness or risk; (c) the conditions under which it is the best choice; (d) the conditions under which it would be the wrong choice. | AI/ML | ALL | Must | Decision-makers need the narrative context; raw scores alone do not communicate the "so what" of a trade-off matrix. |
| REQ-TRD-019 | Cross-Option Trade-off Highlights | The AI SHALL generate a "key trade-offs" section summarising: the 2–3 most significant capability differences between options; any options that are Pareto-optimal (no other option dominates them on all criteria); and any options that are dominated (at least one other option scores better on all criteria). | AI/ML | ALL | Must | Pareto and domination analysis surfaces the real decision axes; without it, architects may spend time comparing options that are objectively worse on all criteria. |

---

### TRD §3 — ATAM-Style Quality-Attribute Analysis {#trd-3}

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-TRD-020 | Quality-Attribute Scenario Elicitation | The suite SHALL support eliciting quality-attribute scenarios for each option in the ATAM style: stimulus, stimulus source, environment, artefact, response, and response measure; the AI SHALL suggest candidate scenarios based on the subject type and active persona's role-specific quality attribute priorities. | AI/ML | P1, P2, P8, P9 | Must | ATAM quality-attribute scenarios convert abstract quality attributes (e.g., "high availability") into testable, concrete claims — the foundation of rigorous trade-off analysis. |
| REQ-TRD-021 | Sensitivity Point Identification | For each quality-attribute scenario, the AI SHALL identify sensitivity points: the specific architectural decision within an option that most affects the scenario's response measure (e.g., "the choice of replication factor is the sensitivity point for availability under network partition"). | AI/ML | P1, P2 | Must | Sensitivity point identification is the ATAM core technique for understanding where architecture decisions have highest leverage; it guides engineers to the right decisions first. |
| REQ-TRD-022 | Trade-off Point Identification | The AI SHALL identify trade-off points: decisions that are sensitivity points for multiple quality attributes simultaneously (e.g., "synchronous replication improves consistency but is a sensitivity point for latency"); each trade-off point SHALL be displayed prominently in the matrix as a cross-criteria tension indicator. | AI/ML | P1, P2 | Must | Trade-off points are the most important output of ATAM analysis; they identify where optimising one quality attribute necessarily degrades another. |
| REQ-TRD-023 | Risk Theme Identification | The AI SHALL identify risk themes for each option: combinations of quality-attribute scenarios that, if a sensitivity point is misconfigured or missing, produce architectural risk (e.g., "absence of circuit breaker in synchronous integration pattern creates a cascading failure risk under high load"); risk themes SHALL be linked to the Risk section of the RAID register (V04). | AI/ML | P2, P8 | Should | Risk themes bridge ATAM analysis and RAID management; without linkage, ATAM analysis remains a design-time exercise disconnected from delivery risk management. |
| REQ-TRD-024 | ATAM Report Generation | The suite SHALL generate an ATAM-style trade-off analysis report (exportable to Word/PDF via V10) that includes: quality-attribute utility tree, scenarios per option, identified sensitivity points, trade-off points, risk themes, and the recommended option with rationale; the report SHALL be cross-referenceable with the matrix and the ADR (TRD §6). | Reporting | P1, P2 | Should | ATAM reports are a recognised governance artefact; generating them reduces the manual documentation burden for architecture review boards. |

---

### TRD §4 — Weighted Scoring & Recommendation Engine {#trd-4}

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-TRD-025 | Weighted Score Calculation | The suite SHALL calculate a weighted score for each option as the sum of (criterion weight × option score) across all criteria; the calculation SHALL be transparent (expandable cell-by-cell breakdown) and SHALL recalculate immediately on weight or score change. | Functional | ALL | Must | Transparent, real-time recalculation is the foundational feature of a weighted decision matrix; without it, the matrix is just a formatting tool. |
| REQ-TRD-026 | Recommended Option with Rationale | The suite SHALL identify the highest-scoring option as the AI recommendation and SHALL generate a recommendation rationale (≤ 200 words) citing: the winning criteria, the margin over the second-placed option, any concerns or conditions on the recommendation (e.g., "recommended if team has Kubernetes expertise"), and the disqualifying constraints that eliminated other options. | AI/ML | ALL | Must | A matrix without an explicit recommendation transfers the decision burden back to the user; the AI must take a position and explain it. |
| REQ-TRD-027 | Second-Place Option Delta Analysis | The AI SHALL generate a delta analysis comparing the recommended option to the second-place option, identifying: the criteria on which the second-place option is equal or better, the criteria gap that drives the overall recommendation, and the conditions under which the second-place option would become the top recommendation. | AI/ML | ALL | Should | Delta analysis is the most common stakeholder question after a trade-off presentation; generating it proactively reduces back-and-forth. |
| REQ-TRD-028 | Minimum-Score Threshold Disqualification | The suite SHALL support a configurable minimum-score threshold per criterion: any option that scores below the threshold on a Must-Have criterion SHALL be disqualified from recommendation regardless of its overall weighted score. | Functional | ALL | Must | Weighted averages can recommend options that score catastrophically low on a critical dimension (e.g., a low-security score averaged away by a high-performance score); threshold disqualification prevents this. |
| REQ-TRD-029 | Confidence Level on Recommendation | The recommendation SHALL carry an AI confidence level (High / Medium / Low) based on: the fraction of cell scores that are evidence-backed vs. assumption-based; the score margin between top and second option; and the completeness of the elicitation context. Low-confidence recommendations SHALL display a prominent caveat. | AI/ML | ALL | Must | Architects must know how much to trust the recommendation; a recommendation with 40% assumption-based cells requires more validation than one with 90% evidence-based cells. |

---

### TRD §5 — What-If Sensitivity Analysis {#trd-5}

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-TRD-030 | Weight Sensitivity Slider | The suite SHALL provide an interactive what-if interface where the user adjusts the weight of any criterion via a slider and the weighted scores and recommendation update in real time; the interface SHALL highlight when the top recommendation changes as a result of a weight adjustment. | UX | ALL | Must | Real-time weight sensitivity is the most impactful interactive feature for trade-off review sessions; seeing when the recommendation flips demonstrates the decision's sensitivity to stakeholder priorities. |
| REQ-TRD-031 | Weight Sensitivity Report | The suite SHALL generate a weight sensitivity report that identifies: for each criterion, the minimum weight increase needed to change the current recommendation (the "tipping point"); and the criteria where a small weight change (≤ 20%) flips the recommendation (high-sensitivity criteria). | Reporting | P1, P2 | Must | Tipping-point analysis reveals the most consequential assumptions in the weighting model and guides the architecture review board's deliberation to the right questions. |
| REQ-TRD-032 | Score Sensitivity Analysis | The suite SHALL, on demand, compute score sensitivity: for each option × criterion cell, the minimum score change needed to change the overall recommendation; cells with high sensitivity SHALL be visually highlighted as requiring additional evidence or validation. | AI/ML | P2 | Should | Score sensitivity analysis identifies the matrix's most vulnerable cells — those where the AI's assessment most influences the outcome — enabling targeted expert review. |
| REQ-TRD-033 | Scenario What-If | The suite SHALL support scenario what-if analysis: the user defines a named scenario (e.g., "if regulatory compliance becomes a hard constraint") with a set of criteria weight overrides and/or constraint additions; the engine SHALL compute the recommendation under each named scenario and display all scenarios side-by-side. | AI/ML | ALL | Should | Scenario what-if captures the "it depends" nature of most architecture trade-offs; presenting multiple stakeholder scenarios in one view replaces repeated matrix runs. |

---

### TRD §6 — ADR & Options Analysis Linkage {#trd-6}

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-TRD-034 | Trade-off to ADR Auto-Population | From a completed trade-off analysis, the suite SHALL offer a one-click action to create or update an ADR (V04 SA-05) pre-populated with: the decision subject, the options considered (from the matrix), the decision (the recommended option), the consequences (from the per-option narrative), and the ATAM trade-off points as the "context" section. | Integration | P2, P3 | Must | Manual re-entry of trade-off matrix content into an ADR is error-prone and time-consuming; pre-population closes the loop between the analysis and its governance record. Cross-ref V04 SA-05. |
| REQ-TRD-035 | ADR-Triggered Trade-off | When a new ADR is created in the ADR workflow (V04), the suite SHALL offer to initiate a trade-off analysis for the ADR's decision subject, pre-populating elicitation fields from the ADR context section. | Integration | P2, P3 | Must | ADR authoring and trade-off analysis are co-dependent; supporting bidirectional initiation prevents one from being authored without the other. |
| REQ-TRD-036 | Options Analysis Linkage | The trade-off matrix SHALL be linkable as the supporting evidence for a Solution Options Analysis artefact (V04 SA-03); the link SHALL be bidirectional: the options analysis displays the matrix summary, and the matrix displays the options analysis it supports. | Integration | P2 | Must | Options analyses without supporting trade-off matrices are opinion documents; matrix linkage elevates them to evidenced recommendations. |
| REQ-TRD-037 | Trade-off Change Impact on ADR | When a trade-off matrix is revised after its linked ADR is in Accepted status, the suite SHALL notify the ADR owner and trigger an ADR review workflow (V04 SA-05); the ADR SHALL not auto-update without explicit owner approval. | Governance | P2, P3 | Must | Silently updating an accepted ADR's evidence without governance review undermines the ADR as a stable decision record. |

---

### TRD §7 — Trade-off Library & Reuse {#trd-7}

#### TRD §7.1 Library Structure

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-TRD-038 | Trade-off Library | The suite SHALL maintain a searchable Trade-off Library where completed trade-off analyses can be saved as reusable templates, each tagged with: subject type, technology/pattern/vendor domain, personas used, quality-attribute emphasis, industry vertical, and outcome. | Functional | ALL | Must | Trade-off analyses for common decisions (monolith vs. microservices, SQL vs. NoSQL, sync vs. async integration) should not be reinvented on every project; a library provides a curated starting point. |
| REQ-TRD-039 | Library-Derived Trade-off Initialisation | When starting a new trade-off analysis, the AI SHALL search the library for semantically similar past analyses and offer to initialise the new matrix from a library template; the user SHALL be able to accept, partially inherit, or start from scratch. | AI/ML | ALL | Must | Library initialisation reduces cold-start time for trade-off analysis from 30–60 minutes to 5–10 minutes for well-covered decision types. |
| REQ-TRD-040 | Platform Library vs. Tenant Library | The trade-off library SHALL have two tiers: (a) **Platform Library** — ESIP-curated, read-only for tenants, covering common architectural decisions; (b) **Tenant Library** — tenant-editable, inherits from platform but can override or extend; a tenant's project team can promote a completed analysis to the tenant library. | Functional | ALL | Must | Platform library provides immediate value at onboarding; tenant library captures institutional knowledge specific to the tenant's domain and technology landscape. |
| REQ-TRD-041 | Library Entry Quality Gate | Promoting a trade-off analysis to the tenant or platform library SHALL require: matrix completeness (TRD-015), at least one peer review (TRD §9), an outcome record (was the recommended option actually adopted? what was the result?), and admin approval. | Governance | ALL | Should | Without a quality gate, the library fills with incomplete or low-quality analyses that undermine its value as a trusted resource. |
| REQ-TRD-042 | Library Search & Similarity Ranking | The trade-off library search SHALL support: keyword search, tag filter, persona filter, quality-attribute emphasis filter, and AI-semantic similarity ranking (cross-ref V06 RAG); results SHALL display a similarity score to the current analysis subject. | Functional | ALL | Must | Faceted search with semantic similarity enables practitioners to find relevant precedents quickly, even when the terminology differs from their own framing. |
| REQ-TRD-043 | Outcome Tracking | The suite SHALL support recording the actual outcome of a trade-off decision: which option was implemented, whether the implementation matched the matrix recommendation, and a post-implementation assessment of whether the quality-attribute predictions were accurate. | Governance | ALL | Should | Outcome tracking creates a feedback loop from decision to result, enabling the library to surface which past analyses were well-calibrated vs. poorly-calibrated, improving future trade-off quality. |

---

### TRD §8 — Visualization {#trd-8}

#### TRD §8.1 Radar Chart

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-TRD-044 | Multi-Option Radar Chart | The suite SHALL render a multi-option radar (spider) chart with quality attributes on the axes and each option as an overlaid polygon; the chart SHALL update in real time as scores or weights change. | UX | ALL | Must | Radar charts are the standard visual for multi-criteria option comparison; they communicate option shape (where an option is strong vs. weak) instantly. |
| REQ-TRD-045 | Weighted vs. Unweighted Radar Toggle | The radar chart SHALL support toggling between unweighted (raw scores) and weighted (weight × score) axes so users can see both the inherent option characteristics and the priority-adjusted view. | UX | ALL | Should | Architects frequently need to communicate both the raw capabilities and the weighted priority view to different stakeholder audiences. |
| REQ-TRD-046 | Radar Export | The radar chart SHALL be exportable as SVG/PNG and SHALL be embeddable in deliverable documents (V10) and in ADRs (V04) with one click. | UX | ALL | Must | Radar charts are the most commonly inserted trade-off visualisation in architecture decision documents; export friction is a common complaint in competing tools. |

---

#### TRD §8.2 Heatmap

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-TRD-047 | Trade-off Heatmap | The suite SHALL render the trade-off matrix as a colour-coded heatmap (green=high/5, amber=medium/3, red=low/1) with cells sized proportionally to the criterion weight; disqualified options SHALL be visually dimmed. | UX | ALL | Must | Heatmaps communicate the full matrix state at a glance and make outlier scores (unexpectedly red or green cells) immediately visible. |
| REQ-TRD-048 | Sensitivity Highlight Overlay | The suite SHALL provide a sensitivity overlay mode for the heatmap that highlights cells identified in TRD §5 as high-sensitivity (small score change changes the recommendation), enabling focused review of the most consequential cells. | UX | P2 | Should | Combining heatmap and sensitivity analysis in a single view directs review effort to where it matters most. |

---

#### TRD §8.3 Pareto & Score Distribution

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-TRD-049 | Pareto Frontier Visualization | For trade-offs between two specified quality attributes (user-selectable), the suite SHALL render a 2D Pareto frontier scatter plot with options as data points; Pareto-optimal options SHALL be connected by the frontier curve; dominated options SHALL be visually distinguished. | AI/ML | P1, P2 | Should | Pareto frontier visualization is the rigorous method for identifying the efficient frontier of options in a two-attribute trade-off; it directly corresponds to ATAM trade-off point identification. |
| REQ-TRD-050 | Weighted Score Bar Chart | The suite SHALL render a bar chart of weighted total scores per option, with each bar segmented by criterion contribution; the chart SHALL update in real time with weight changes and SHALL be exportable for inclusion in presentations. | UX | ALL | Must | Segmented bar charts communicate both the ranking and the composition of the score, making the drivers of the recommendation visible to non-technical stakeholders. |

---

### TRD §9 — Collaboration & Review of Trade-offs {#trd-9}

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-TRD-051 | Matrix Sharing & Permissions | A trade-off matrix SHALL be shareable with configurable permissions (Owner / Editor / Commenter / Viewer) independently of the broader project permissions; a matrix SHALL be shareable with stakeholders outside the project as read-only with a time-limited link. | Functional | ALL | Must | Trade-off analyses are frequently shared for stakeholder review before an ADR is finalised; separate sharing permissions enable selective disclosure without opening full project access. |
| REQ-TRD-052 | Cell-Level Commenting | Reviewers SHALL be able to add comments on individual matrix cells (challenging a score or its rationale) with @mention, thread reply, and resolve capabilities; cell-level comments SHALL be displayed inline with the cell in a hover panel and SHALL be tracked in the review history. | UX | ALL | Must | Cell-level comments enable reviewers to challenge specific scores without requiring a full re-analysis; they are the most granular review mechanism for trade-off matrices. Cross-ref V15 REV. |
| REQ-TRD-053 | Matrix Review Workflow | The suite SHALL support a lightweight matrix review workflow (Draft → Under Review → Reviewed → Approved) with configurable mandatory reviewers; the ADR linkage (TRD §6) SHALL block ADR promotion to Accepted until the linked matrix is Approved. | Governance | ALL | Must | Linking matrix approval to ADR promotion enforces the principle that architecture decisions are not accepted without reviewed trade-off analysis. |
| REQ-TRD-054 | AI-Assisted Review Summary | On completion of matrix review, the AI SHALL generate a review summary (≤ 250 words) capturing: unresolved comments, score revisions made during review, change to the recommendation (if any), and the final recommendation with reviewer sign-off names. | AI/ML | ALL | Should | A structured review summary transforms the review artefact into an auditable document of record; it reduces the effort of writing governance minutes for architecture review boards. |
| REQ-TRD-055 | Reviewer Persona Weighting | The suite SHALL allow the review facilitator to associate each reviewer with a stakeholder persona (e.g., "CISO reviews the security criteria"; "FinOps reviews the cost criteria") so that reviewer comments on criteria outside their designated persona are flagged as advisory rather than blocking. | Governance | P1, P2 | Should | Reviewer-persona association prevents a single reviewer from blocking progress on criteria outside their expertise, while ensuring the right subject-matter experts review the right criteria. |

---

### TRD §10 — Proactive Decision-Point Detection {#trd-10}

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|-----------|----------|-----------|
| REQ-TRD-056 | Decision-Point Detection in Artifacts | The AI SHALL monitor the current artifact being authored (HLD, solution brief, NFR register, requirements document) and detect candidate decision points: phrases or structural patterns that indicate a choice between alternatives is implicit (e.g., "we could use X or Y", "the team is considering", "TBD — need to decide", "options include"). | AI/ML | ALL | Must | Decision points are most reliably detected at the moment they are articulated in a document; delayed detection results in undocumented decisions being implemented without trade-off analysis. |
| REQ-TRD-057 | Decision-Point Nudge | When a decision-point is detected, the AI SHALL surface a contextual nudge: "This looks like a trade-off decision — would you like to create a trade-off analysis for '[detected subject]'?"; the nudge SHALL offer a one-click action to initiate the elicitation interview with the detected subject and options pre-populated. | UX | ALL | Must | The nudge-to-initiate mechanism is the key activation pathway for proactive trade-off analysis; it must be low-friction (single click) to convert detection into analysis. |
| REQ-TRD-058 | NFR Tension Detection | The AI SHALL detect NFR tensions in the NFR register (V04 SA-09): pairs of NFRs that are in inherent quality-attribute tension (e.g., "sub-100ms latency" + "strong consistency" in a distributed system) and SHALL suggest a trade-off analysis for each detected tension, pre-seeding the ATAM scenarios. | AI/ML | P2, P8 | Must | NFR tensions are the most frequently overlooked class of implicit trade-off; detecting them proactively prevents "we didn't realise these were in conflict" late-stage surprises. |
| REQ-TRD-059 | Meeting & Discussion Decision-Point Capture | The AI SHALL scan real-time meeting transcripts (V17 real-time capture connector) and discussion threads (V12 collaboration) for decision-point patterns and surface them for trade-off initiation; detected decision points SHALL be presented in the project's trade-off queue. | AI/ML | ALL | Should | Architecture decisions are frequently made verbally in meetings and never formally documented; real-time transcript scanning creates a systematic capture mechanism. Cross-ref V17 (RTC connectors). |
| REQ-TRD-060 | Trade-off Queue & Backlog | The suite SHALL maintain a Trade-off Queue per project: a structured backlog of detected decision points and explicitly created trade-off analyses, each with status (Detected / Elicitation-in-Progress / Analysis-Complete / Linked-to-ADR / Closed), priority, and assigned owner. | Functional | ALL | Must | A queue makes the trade-off backlog visible and manageable; without it, detected decision points are ephemeral nudges that are dismissed and forgotten. |
| REQ-TRD-061 | Prescriptive Trade-off Initiation | The suite SHALL, based on the current project's architecture phase and persona context, proactively suggest trade-off analyses that a practitioner of that role would typically perform at this stage (e.g., "At Solution Architecture HLD stage, a great SA would perform a trade-off analysis for the deployment model and the data consistency strategy"); each suggestion SHALL include the default elicitation context pre-populated for the project. Cross-ref V16 INT (prescriptive next-best-action) and AIX §5 (proactive nudges). | AI/ML | ALL | Should | Phase-contextual trade-off suggestions are the prescriptive tier of the trade-off engine; they shift the AI from reactive (detecting explicit decisions) to proactive (anticipating necessary decisions). |
| REQ-TRD-062 | Trade-off Coverage Metric | The suite SHALL compute a Trade-off Coverage Metric per project: the ratio of detected decision points that have an associated approved trade-off analysis; projects with low coverage SHALL surface a governance alert; the metric SHALL be reportable at portfolio level. Cross-ref V16 INT (conformance and governance analytics). | Reporting | P1, P2 | Should | Trade-off coverage is a leading indicator of architecture decision quality; low coverage predicts future RAID entries and rework; portfolio-level visibility enables governance intervention at the right moment. |

---

## Appendix: Requirement Counts by Prefix

| Prefix | Sub-area | Count | Range |
|--------|----------|-------|-------|
| REQ-AIX | §1 Prompt-Pack Catalog & Structure | 11 | 001–011 |
| REQ-AIX | §2 Per-Role Expert Persona Packs | 45 | 012–056 |
| REQ-AIX | §3 Governance, Versioning & Admin | 10 | 057–066 |
| REQ-AIX | §4 Active-Persona Auto-Adoption | 5 | 067–071 |
| REQ-AIX | §5 Proactive Suggestions & Nudges | 6 | 072–077 |
| REQ-AIX | §6 Grounding & Citations | 4 | 078–081 |
| REQ-AIX | §7 Evaluation & Quality Assurance | 9 | 082–090 |
| REQ-AIX | §8 Safe Fallback & Escalation | 4 | 091–094 |
| **REQ-AIX subtotal** | | **94** | 001–094 |
| REQ-TRD | §1 Elicitation Interview Mode | 8 | 001–008 + weights |
| REQ-TRD | §1.3 Quality-Attribute Weights | 3 | 009–011 |
| REQ-TRD | §2 Matrix Generation | 8 | 012–019 |
| REQ-TRD | §3 ATAM-Style Analysis | 5 | 020–024 |
| REQ-TRD | §4 Weighted Scoring | 5 | 025–029 |
| REQ-TRD | §5 What-If Sensitivity | 4 | 030–033 |
| REQ-TRD | §6 ADR & Options Linkage | 4 | 034–037 |
| REQ-TRD | §7 Trade-off Library & Reuse | 6 | 038–043 |
| REQ-TRD | §8 Visualization | 7 | 044–050 |
| REQ-TRD | §9 Collaboration & Review | 5 | 051–055 |
| REQ-TRD | §10 Proactive Detection | 7 | 056–062 |
| **REQ-TRD subtotal** | | **62** | 001–062 |
| **VOLUME 21 TOTAL** | | **156** | — |

---

## Cross-Reference Summary

| Cross-ref | Topic |
|-----------|-------|
| V04 SA-03, SA-04, SA-05 | Options analysis, trade-off, ADR lifecycle |
| V05 ROL | Per-role persona definitions, role-extensibility |
| V06 §9, §11, §12, §14 | RAG, guardrails/eval, prompt library infra, feedback loop |
| V11 PLT | Persona metamodel, persona assignment, custom persona creation |
| V14 PERS | Active persona state, per-user adaptation, personalization preferences |
| V15 REV | Review designer, inline comments/threads/resolve on artefacts |
| V16 INT | Prescriptive next-best-action, conformance/governance analytics, marketplace |
| V17 RTC | Real-time meeting transcript connectors for decision-point capture |
| V19 ENT | Architecture-as-code (prompt packs in git); partner/pack marketplace |
| V23 (planned) | Deliverable template library (system prompt for template rendering referenced) |
