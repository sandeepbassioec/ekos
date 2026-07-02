# EKOS Volume 03 — Hybrid Ambient AI Engine (4 Layers, Analysts, Explainability)

> **Status:** ✅ Done · **Requirement count:** 94 · **Generated:** 2026-06-30 (Opus)
> **Sources mined:** `blueprint §4` (Hybrid AI Engine + Sandeep Genie),
> `research_across_llms.txt` (AI Strategy/4-layer §, Ambient Intelligence §, AI
> Example & Explanation Engine §, "big bang" failure table, "how AI knows without a
> prompt" §), `ConfusionClarity.txt` (AI Governance/explainability, Chain-of-Thought,
> AI Model Governance, evolution suggestions), `Gemini critique` (Sovereign/air-
> gapped AI, Dual-Engine), `Patents.txt` §4 (Promptless Ambient AI), §9 (Sandbox
> Vault). Honors [00-MASTER-INDEX](00-MASTER-INDEX.md) §2 (esp. §5 promptless, §6
> no-train/air-gap/pluggable).

---

## 1. Scope & Boundary

**In scope:** the promptless, proactive, air-gappable **4-layer hybrid AI engine**
(symbolic reasoning → domain small models → RAG/LLM → ambient orchestrator); the
**structured-context auto-prompt** mechanism; the **built-in AI analyst catalog**;
the **explanation/example engine** (role-adaptive); **automatic test/scenario
generation**; **gap detection** with ghost nodes; **sovereign/air-gapped** dual-
engine model layer; **explainability & confidence**; engine-side **AI governance
hooks**; **self-improvement/evolution** & federated learning; the **on-device Sandbox
Vault**; the **Sandeep Genie** avatar; AI-engine performance/cost; AI-analyst
marketplace hooks.

**Explicitly deferred (cross-referenced, not duplicated):**
- The graph, ontology, mutation event stream, graph algorithms substrate, `AIInsight`
  node type → **Vol 01** (this engine *consumes* them)
- The `ai_hooks` binding surface on widgets, "Explain" toggle, AI-suggested schema,
  Process Capture AI → **Vol 02** (this volume implements the analysts those hooks
  invoke)
- ABAC/security projection that scopes what the AI may see, PII masking → **Vol 04**
- **Policy-as-code, decision-DNA, human-in-the-loop *approval gates*, AI-output
  review-gating, full AI governance program (bias audits, certification process,
  model registry governance)** → **Vol 05** (this volume defines only the engine-side
  *hooks* — confidence scores, evidence chains, kill-switch interface — that Vol 05
  governs)
- Marketplace transactions / revenue for AI analysts → **Vol 13** (engine-side
  publish/run hooks only)
- Tech-stack engine selection (which small models, which LLMs, which inference
  runtime) → **Vol 15**

---

## 2. Core Principle — Promptless, Proactive Ambient Intelligence

> Constitution §5. "The AI doesn't need a user prompt because the widget IS the
> prompt." It never waits to be asked.

- **EKOS-03-001 (Must, MVP)** — The AI engine SHALL be **proactive and continuous**,
  surfacing insights without the user typing a prompt; reactive Q&A is an *additional*
  mode, not the primary one. *Source:* blueprint §4; research(Ambient Intelligence §);
  Patent §4.

- **EKOS-03-002 (Must, MVP)** — When a widget loads, the AI engine SHALL receive a
  **structured context** (widget type, the widget's instance subgraph, user role,
  history, linked widgets) and SHALL pre-populate its insight panel — **the user opens
  the "Crux Panel" and it is already filled**. *Source:* research(Ambient Intelligence
  — the context JSON; "the first answer is already there"); Patent §4 (claim 1).

- **EKOS-03-003 (Must, MVP)** — The AI engine SHALL **subscribe to the graph mutation
  event stream** (Vol 01 §13) and run applicable analysts automatically on relevant
  changes — pre-generating explanations so they are ready *before* the user asks.
  *Source:* research(AI §4 orchestrator); Patent §4 (claim 1 — "in response to a graph
  change event"). *Depends:* Vol 01 EKOS-01-063/064.

- **EKOS-03-004 (Must, MVP)** — The **ontology IS the prompt**: the AI's understanding
  derives from the semantic meaning of nodes/edges (Vol 01), not from free text the
  user writes. The engine SHALL build all its reasoning context from the graph's
  machine-readable semantics. *Source:* Constitution §5; research("the ontology IS the
  prompt"); Vol 01 EKOS-01-080.

- **EKOS-03-005 (Must, MVP)** — The AI SHALL operate strictly **within the requesting
  user's security projection** — it can never explain, summarize, or surface content
  the user is not authorized to see (no leakage via generated examples/summaries).
  *Source:* research(Security §7); Vol 01 EKOS-01-056; Vol 04. *Depends:* Vol 04.

---

## 3. Design Constraints — Why NOT "Big-Bang LLM-Only"

> Captured as hard constraints, because the research explicitly rejected the tempting
> "dump everything into one giant LLM prompt" approach.

- **EKOS-03-006 (Must, MVP)** — Crucial facts (cost estimates, security boundaries,
  compliance controls, impact sets) SHALL be produced by **deterministic layers, not
  by an LLM** — the LLM must never be the source of a fact that requires 100% accuracy.
  *Rationale:* LLMs hallucinate cost/classification/controls. *Source:* research(big-
  bang failure table — determinism & rule enforcement).

- **EKOS-03-007 (Must, MVP)** — The engine SHALL **not** send thousands of tokens to a
  large model on every widget refresh; large-model calls SHALL be reserved for
  natural-language generation and **aggressively cached** (re-use if the graph hasn't
  changed). *Rationale:* latency & cost at thousands-of-widgets scale. *Source:*
  research(big-bang failure table — latency & cost; Layer 3 — "cache results
  aggressively").

- **EKOS-03-008 (Must, MVP)** — Continuous monitoring (e.g., "this PII store has no
  encryption node") SHALL be done by the **always-on symbolic layer**, not an LLM,
  because an LLM only reacts when prompted and cannot watch continuously. *Source:*
  research(big-bang failure table — proactive always-on).

- **EKOS-03-009 (Must, MVP)** — Complex graph reasoning (blast radius, cost
  attribution, multi-step traversal math) SHALL be done by **graph algorithms** (Vol
  01 §19), not by asking an LLM to reason over a huge context. *Source:* research(big-
  bang failure table — complex graph reasoning; context-window limits).

- **EKOS-03-010 (Should, P2)** — A **single bespoke foundation model is NOT on the
  critical path**; the deterministic-graph + small-models + existing-LLM combo SHALL
  deliver ~90% of the value. Custom large-model training is a year-2 optimization, not
  a foundation. *Source:* research("why a single custom model is overkill early";
  "when do we need to train a custom model").

- **EKOS-03-011 (Must, MVP)** — The "big bang" that EKOS actually builds SHALL be the
  **unified knowledge graph + ontology** (the custom model of the software world); the
  AI layers are smart clients of it, and EKOS's IP lives in the ontology, custom
  widgets, and orchestration — not in raw LLM weights. *Source:* research("the big
  bang we actually do").

---

## 4. Layer 1 — Symbolic Reasoning & Graph Analytics (The Rule Keeper)

- **EKOS-03-012 (Must, MVP)** — Layer 1 SHALL be a **deterministic, code-based rule
  engine** (no ML) that evaluates ontology-encoded graph patterns and SHALL produce
  the majority (~60%) of proactive insights — the red flags and crux numbers. *Source:*
  research(Layer 1); Patent §4 (claim 1).

- **EKOS-03-013 (Must, MVP)** — Layer 1 SHALL **validate graph integrity** against
  rules (e.g., "every data store containing PII must have an encryption node linked";
  "every requirement must have at least one test"). Violations SHALL be recorded as
  `AIInsight`/alert nodes (Vol 01 §4) with red badges/blocked actions. *Source:*
  research(Layer 1; governance examples); Patent §4 (claim 1 — "generate alert nodes").

- **EKOS-03-014 (Must, MVP)** — Layer 1 SHALL run **graph algorithms**: shortest-path/
  transitive traversal (impact analysis), clustering (cost attribution), rule-based
  pattern matching. *Source:* research(Layer 1); Vol 01 EKOS-01-085.

- **EKOS-03-015 (Must, MVP)** — Layer 1 SHALL compute **quantitative answers**: cost
  totals/roll-ups, latency simulations, what-if propagation — deterministically and
  auditably. *Source:* research(Layer 1 — quantitative); Vol 01 EKOS-01-086.

- **EKOS-03-016 (Must, MVP)** — Layer 1 SHALL react to graph mutation events by
  **instantly checking the relevant rule subset** (e.g., "on node add, check 500
  rules") — lightning-fast and deterministic. *Source:* research(Layer 1 — "subscribes
  to graph change events ... instantly checks 500 rules").

---

## 5. Layer 2 — Domain-Specific Small Models (The Silent Expert)

- **EKOS-03-017 (Should, P2)** — Layer 2 SHALL provide **lightweight fine-tuned
  transformer models** (e.g., DistilBERT-class or small Llama-class) that run cheaply
  on CPU/GPU and can be invoked hundreds of times per second. *Source:* research(Layer
  2); Patent §4 (claim 1 — ML layer).

- **EKOS-03-018 (Should, P2)** — Layer 2 SHALL **classify architectural decisions into
  patterns** (e.g., CQRS, event sourcing, DDD) from graph sub-structure. *Source:*
  research(Layer 2).

- **EKOS-03-019 (Should, P2)** — Layer 2 SHALL **detect "smells"** that need semantic
  understanding (too fuzzy for hard rules, too costly for a big LLM) — e.g., "this
  microservice handles both payment and inventory → single-responsibility violation /
  monolith smell." *Source:* research(Layer 2; AI example engine).

- **EKOS-03-020 (Should, P2)** — Layer 2 SHALL **embed graph nodes into vectors** for
  similarity search, so the system can retrieve similar past patterns and their
  outcomes when a user designs something. *Source:* research(Layer 2 — embeddings).

- **EKOS-03-021 (Should, P2)** — Layer 2 models SHALL be trainable **offline on a
  synthetic dataset** generated from the ontology plus open-source architecture
  repositories (public TOGAF case studies, cloud reference architectures, ~10k public
  GitHub repos/diagrams). *Source:* research(Layer 2; "when do we need to train").

- **EKOS-03-022 (Could, P3)** — Layer 2 SHALL support **domain-specific entity
  extraction** (e.g., extracting actors/data-flows from free-text requirements to seed
  a Use-Case Flow Widget), bootstrapped from few-shot LLM and distilled into a small
  model for cost. *Source:* research("when do we need to train a custom model").

---

## 6. Layer 3 — Retrieval-Augmented Generation with Existing LLMs (The Eloquent Explainer)

- **EKOS-03-023 (Must, MVP)** — Layer 3 SHALL be invoked **only when natural-language
  output is needed** (crux summary, example story, trade-off rationale). It turns
  structured truth into human text — it does the eloquence, not the fact-finding.
  *Source:* research(Layer 3).

- **EKOS-03-024 (Must, MVP)** — Layer 3 SHALL receive a **structured fact sheet** (JSON)
  assembled by Layers 1 & 2 — e.g., "Component X costs $1200/mo, linked to Req R123,
  has CVE-2024-123, violates 'circular dependency'" — plus pattern classifications and
  similar-pattern references. *Source:* research(Layer 3 — structured context bundle).

- **EKOS-03-025 (Must, MVP)** — A **template engine** SHALL convert the fact sheet into
  a **governed, highly-specific prompt** (auto-generated from widget type + data, NOT
  written by the user) that constrains the LLM to a bounded task. *Source:* research
  (Layer 3 — template engine; Ambient Intelligence — auto-generated prompt); Patent §4
  (claim 1 — "automatically constructs a prompt ... without explicit user input").

- **EKOS-03-026 (Must, MVP)** — Because all facts come from the deterministic layers,
  the LLM **SHALL NOT be relied on to introduce facts** — it rephrases/explains given
  facts only, eliminating hallucination of critical data. *Source:* research(Layer 3 —
  "never hallucinates crucial facts"); Constitution §5.

- **EKOS-03-027 (Must, MVP)** — Layer 3 outputs SHALL be **cached** keyed on the
  underlying subgraph state; if the graph hasn't changed, the explanation is not
  regenerated. *Source:* research(Layer 3 — aggressive caching); EKOS-03-007.

- **EKOS-03-028 (Must, MVP)** — The default and most-capable Claude models SHALL be the
  reference LLMs for Layer 3 in cloud mode, but the model SHALL be **provider-pluggable**
  (Constitution §6); model identity is configuration, not architecture. *Source:*
  Constitution §6; environment guidance (latest Claude models). *Depends:* Vol 15.

---

## 7. Layer 4 — Ambient Intelligence Orchestrator

- **EKOS-03-029 (Must, MVP)** — Layer 4 SHALL be an **event-driven rule engine (pure
  code, not ML)** that decides **which analyst runs when**. *Source:* research(Layer 4);
  Patent §4 (claim 1 — orchestrator).

- **EKOS-03-030 (Must, MVP)** — On **widget open**, the orchestrator SHALL run Layer-1
  rule checks → show red badges immediately; if the user opens the Crux Panel, it
  calls Layer 3 for the summary. *Source:* research(Layer 4 — orchestration flow).

- **EKOS-03-031 (Must, MVP)** — On **graph mutation** (incl. via connectors, e.g. a new
  cloud resource), the orchestrator SHALL run impacted security/cost analyses; if
  significant, push a notification and **pre-generate the explanation** so it's ready
  before the user asks. *Source:* research(Layer 4); Patent §4 (claim 1).

- **EKOS-03-032 (Must, MVP)** — The orchestrator SHALL **select which insights to
  surface based on user role and current activity** (a Security Architect and a Cost
  Owner see different prioritized insights on the same widget). *Source:* Patent §4
  (claim 3 — "selects which insights to surface based on user role and current
  activity"); research(Ambient Intelligence — userRole in context).

---

## 8. Structured Context Bundle & Auto-Prompt Construction

- **EKOS-03-033 (Must, MVP)** — For each analysis, the engine SHALL assemble a context
  bundle including at minimum: `widgetType`, the relevant `instanceGraph` subset,
  `userRole`, interaction `history`, and `linkedWidgets`. *Source:* research(Ambient
  Intelligence — the JSON context example).

- **EKOS-03-034 (Must, MVP)** — A **mapping from widget type → applicable insight
  types** SHALL drive which analysts/prompt-templates apply (e.g., a
  MicroservicesDeploymentDiagram → resilience audit + cost trade-off + crux). *Source:*
  research(Layer 3 template engine; widget `ai_hooks` in Vol 02). *Depends:* Vol 02
  EKOS-02-034.

- **EKOS-03-035 (Should, P2)** — The auto-generated prompt SHALL embed the **ontological
  context** (what the nodes/edges mean) so the LLM's output is grounded in the
  platform's semantics. *Source:* research(Ambient Intelligence — "derived
  automatically from the graph"); Patent §4 (claim 1).

---

## 9. Built-In AI Analyst Catalog

> Each analyst is a configurable agent bound to widget `ai_hooks`. The set is
> extensible (custom & marketplace analysts, §17).

- **EKOS-03-036 (Must, MVP)** — **Crux Summariser** — explains in plain language what
  the current view means, using the ontology. *Source:* blueprint §4; research(Analysts).

- **EKOS-03-037 (Must, MVP)** — **Example Generator** — creates a concrete, real-world
  scenario using the actual entities in the graph (see §10). *Source:* blueprint §4;
  research(Analysts; AI Example Engine).

- **EKOS-03-038 (Must, P2)** — **Trade-off Advisor** — given a design (e.g., a service
  on EC2), checks alternatives (e.g., Lambda) using cost/usage analysis and explains
  *why*, referencing the graph's NFR links. *Source:* blueprint §4; research(Analysts;
  cloud architecture §). *Note:* relates to ESIP's existing Trade-off widget &
  Solution-Design pillar ([[project-esip]] Epic E).

- **EKOS-03-039 (Must, P2)** — **Security Auditor** — follows data flows, flags PII
  crossing a trust boundary without encryption, maps to NIST/controls. *Source:*
  blueprint §4; research(Analysts; Security Topology §). *Depends:* Vol 04.

- **EKOS-03-040 (Must, MVP)** — **Traceability Policeman** — highlights requirements
  with no tests, code not linked to any story, coverage gaps. *Source:* blueprint §4;
  research(Analysts).

- **EKOS-03-041 (Should, P2)** — **What-If Simulator** — propagates a change (e.g.,
  "increase users to 1M") across cost/performance/architecture constraints and shows
  the animated result. *Source:* blueprint §4; research(Analysts; change impact §).

- **EKOS-03-042 (Should, P2)** — **AI Horizon Scanner** — continuously ingests external
  signals (tech blogs, research papers, news, regulatory updates via configurable
  connectors) and correlates them against `Uncertainty` nodes (Vol 01 §10); on a
  detected trend it flags dependent artifacts, notifies, and can auto-draft a Change
  Request. *Source:* research(future-proofing §2 — AI Horizon Scanner); blueprint §4.
  *Depends:* Vol 01 EKOS-01-050/052.

- **EKOS-03-043 (Should, P2)** — **Multi-Agent Review Board** — multiple AI personas
  debate an architecture decision from different lenses before it's accepted. *Source:*
  research(final spec §5.2 — Multi-Agent Review Board); blueprint. *Relates:*
  [[project-esip]] Vol 15 AI-augmented review designer.

- **EKOS-03-044 (Should, P2)** — **Graph Linter analyst** — surfaces structural smells
  / spaghetti density (the detection counterpart of Vol 01 §12). *Source:* Gemini
  critique; Vol 01 EKOS-01-060.

- **EKOS-03-045 (Should, P2)** — **FinOps / Cost analyst** — attributes cost to features
  ("Feature X is 42% of the AWS bill, up 15% since last sprint; here are the linked
  commits/epic"). *Source:* research(FinOps Lens §; cost-to-feature §).

- **EKOS-03-046 (Could, P3)** — The analyst set SHALL be **extensible** — new built-in
  analysts can be added, and third parties can publish analysts (§17). *Source:*
  research(AI Analyst economy); blueprint.

---

## 10. AI Explanation & Example Engine (Role-Adaptive)

> The "ultimate accessibility bridge" — turns visual models into human stories.

- **EKOS-03-047 (Must, P2)** — The Example Generator SHALL read a widget's nodes
  (steps, actors, data objects) and produce a **concrete, realistic narrative** using
  specific example data (e.g., "Mrs. Sharma bought a mixer... logs in... system checks
  warranty..."). *Source:* research(AI Example Engine §1).

- **EKOS-03-048 (Should, P2)** — The engine SHALL **populate a simulated data table and
  animate the flow** with that example's data moving through the steps. *Source:*
  research(AI Example Engine §1).

- **EKOS-03-049 (Must, P2)** — The user SHALL be able to request **role/perspective-
  specific explanations** of the same model — "explain like I'm 5," "explain to a CFO
  focusing on cost," "translate to a developer's technical contract" — generating
  different perspectives of one underlying model. *Source:* research(AI Example Engine
  §1).

- **EKOS-03-050 (Should, P2)** — Because the AI sees a security-scoped projection
  (EKOS-03-005), generated examples SHALL never include masked/restricted data.
  *Source:* research(Security §7); Vol 04.

---

## 11. Automatic Test & Scenario Generation

- **EKOS-03-051 (Should, P2)** — From a sequence diagram / flow widget, the engine SHALL
  generate **concrete Gherkin scenarios** for each alternate path, complete with
  example payloads. *Source:* research(AI Example Engine §2).

- **EKOS-03-052 (Should, P2)** — The engine SHALL produce an **executable sandbox
  example** ("hit this endpoint with this JSON → here's the response → here's the
  business meaning: order is now 'ready to ship'"). *Source:* research(AI Example
  Engine §2).

- **EKOS-03-053 (Could, P3)** — Generated test scenarios SHALL be **linkable back to
  acceptance criteria** as `TestCase` nodes (Vol 01), preserving traceability. *Source:*
  derived from research(Acceptance Criteria → test traceability); Vol 01 §5.

---

## 12. Gap Detection with Real-World Analogy

- **EKOS-03-054 (Should, P2)** — The engine SHALL detect **missing edge cases / gaps**
  in a model and explain them via real-world analogy ("this flow assumes payment always
  succeeds — like assuming every customer enters their CVV correctly"). *Source:*
  research(AI Example Engine §3).

- **EKOS-03-055 (Should, P2)** — On gap detection, the engine SHALL **place a draft
  "ghost node"** in the widget (e.g., a payment-failure alternate flow) for the user to
  accept/reject. *Source:* research(AI Example Engine §3 — "places a draft ghost node").

- **EKOS-03-056 (Should, P2)** — Ghost-node acceptance SHALL pass through the AI-output
  review gate if configured (Vol 05) before becoming a confirmed graph mutation.
  *Source:* Constitution §5 + [[project-esip]] C4 (AI auto-apply must reset approval
  gate). *Depends:* Vol 05.

---

## 13. Sovereign / Air-Gapped AI (Dual-Engine Model Layer)

> Gemini's critical fix: public LLMs are illegal in defense/intel/regulated air-gapped
> environments. Layer 3 must be swappable.

- **EKOS-03-057 (Must, P2)** — Layer 3 SHALL implement a **Dual-Engine Model
  Compatibility Layer**: standard commercial SaaS users route to public cloud models;
  Defense/Sovereign users swap the entire text-generation layer for **fully quantized,
  containerized open models (e.g., Llama/Mistral-class) hosted natively in the
  customer's private VPC or on-premise hardware**. *Source:* Gemini critique (Self-
  Hosted Sovereign AI Core; Dual-Engine).

- **EKOS-03-058 (Must, P2)** — In sovereign mode, **only the structured fact sheet**
  (lightweight tables) SHALL be sent to the local model — never raw customer content
  off-device/off-premise (zero data exfiltration). *Source:* Gemini critique; Patent §4
  (claim 2 — "only the structured fact sheet is sent").

- **EKOS-03-059 (Must, P2)** — Layers 1 & 2 (symbolic + small models) SHALL run fully
  **inside the secure boundary**, so air-gapped deployments retain the majority of AI
  value even with the LLM layer constrained or disabled. *Source:* Gemini critique
  (architecture diagram — symbolic + small models inside boundary).

- **EKOS-03-060 (Should, P2)** — The prompt-generation pipeline SHALL transform graph
  context into **lightweight tables** to keep local-model compute overhead minimal.
  *Source:* Gemini critique ("minimal computing overhead to render Crux").

- **EKOS-03-061 (Must, MVP)** — **No EKOS model shall train on customer content;**
  opted-in learning is limited to anonymized behavioral signals only (Constitution §6).
  *Source:* Constitution §6; [[project-esip]] (no-train-on-customer-CONTENT).

---

## 14. Explainability & Trust (Engine-Side)

- **EKOS-03-062 (Must, MVP)** — Every AI insight SHALL publish an **evidence chain**:
  which graph nodes it examined, which rules fired, and a **confidence score**.
  *Source:* ConfusionClarity(AI Governance — confidence & explainability); blueprint
  §AI governance; research(final spec §5.3).

- **EKOS-03-063 (Must, P2)** — Insights **below a configurable confidence threshold**
  SHALL be auto-suppressed or flagged "for review." *Source:* ConfusionClarity(AI
  Governance — confidence thresholds).

- **EKOS-03-064 (Should, P2)** — A **Chain-of-Thought Visualizer** SHALL render the
  decision tree the AI traversed, linking back to the exact graph nodes that influenced
  a recommendation (e.g., "why move this microservice to Lambda?"). *Source:*
  ConfusionClarity(future domains B — Chain-of-Thought Visualizer Widget). *Note:* the
  *widget* render is Vol 14; the *traceable reasoning data* is produced here.

- **EKOS-03-065 (Should, P2)** — AI insights SHALL be stored as **`AIInsight` nodes**
  (Vol 01 §4) linked to the nodes they concern, so they are themselves traceable,
  versioned, and auditable. *Source:* Vol 01 EKOS-01-022; research.

---

## 15. AI Governance Hooks (Engine-Side; Program in Vol 05)

> Engine exposes the controls; the *governance program* (policy, certification process,
> bias audits, approval workflows) lives in Vol 05.

- **EKOS-03-066 (Should, P2)** — Each AI analyst SHALL be attachable to a
  `GovernancePolicy` (Vol 05), e.g., "any cost-optimization suggestion > $5,000/mo must
  be human-reviewed." The engine SHALL **honor** such policies by routing through an
  `ApprovalGate`. *Source:* ConfusionClarity(AI Governance — analyst certifications,
  human-in-the-loop). *Depends:* Vol 05.

- **EKOS-03-067 (Should, P2)** — Critical AI-suggested actions (e.g., "provision a new
  AWS account," "change this SLO") SHALL **always route through a human-in-the-loop
  ApprovalGate** before execution. *Source:* ConfusionClarity(AI Governance — human-in-
  the-loop checkpoints). *Depends:* Vol 05.

- **EKOS-03-068 (Should, P2)** — The engine SHALL emit **bias/drift monitoring signals**
  (output distribution per team/analyst) and expose a **kill-switch interface** to
  disable a misbehaving analyst; the *governing rules* are Vol 05. *Source:*
  ConfusionClarity(AI Governance — bias & drift monitoring; auto-disable).

- **EKOS-03-069 (Should, P2)** — Custom/third-party AI analysts SHALL carry a
  **versioned lineage** in a Model Registry (engine registers; governance in Vol 05).
  *Source:* ConfusionClarity(future domains E — Model Registry / MLOps for analysts).

---

## 16. Self-Improvement & Evolution

- **EKOS-03-070 (Could, P3)** — The engine SHALL improve its meta-model and analysis
  templates based on **which AI answers users find useful** (accept/reject/reason
  feedback). *Source:* research(Ambient Intelligence — "improve based on user
  interactions"; final spec §5.3 — self-improvement via feedback).

- **EKOS-03-071 (Could, P3)** — Power users SHALL be able to **create custom AI Analysts
  via a low-code flow**. *Source:* research(Ambient Intelligence — "let power users
  create custom AI Analysts").

- **EKOS-03-072 (Could, P3)** — Via **federated learning across opted-in tenants**
  (anonymized, no customer content — EKOS-03-061), the engine SHALL detect emerging
  patterns and **suggest new ontology extensions, widgets, or rules** ("40% of
  healthcare projects added a 'FHIR API Gateway' pattern — relevant to your HL7
  migration?"). *Source:* research(future-proofing §4 — federated learning &
  self-enrichment). *Depends:* Vol 01 EKOS-01-084; Constitution §6.

- **EKOS-03-073 (Could, P3)** — The engine MAY **suggest process improvements** from
  bottleneck analysis ("requirements spend 5 days in Review due to Approver X — propose
  a load-balanced workflow?"). *Source:* ConfusionClarity(workflow §AI integration).

- **EKOS-03-074 (Could, Frontier)** — The engine MAY eventually **interview stakeholders
  in natural language** to extract requirements and turn them into a linked set of
  widgets. *Source:* research(Ambient Intelligence — "interview stakeholders ... turn
  directly into linked widgets").

---

## 17. On-Device Sandbox Vault (Patent §9)

> User-controlled local intelligence with zero data exfiltration — distinct from the
> graph-scoped engine; for the user's own selected files.

- **EKOS-03-075 (Should, P2)** — The system SHALL provide a **sandboxed, encrypted
  storage vault on the user device** into which the user **explicitly drag-and-drops**
  files for local AI processing. *Source:* Patent §9 (claim 1).

- **EKOS-03-076 (Should, P2)** — An **on-device embedding model** SHALL build a
  searchable index of the vault's content **without transmitting any file data off the
  device**. *Source:* Patent §9 (claim 1).

- **EKOS-03-077 (Should, P2)** — Natural-language queries SHALL be answered **on-device**
  from the vault index, returning a response/summary **without exposing raw file data
  externally**. *Source:* Patent §9 (claim 1).

- **EKOS-03-078 (Should, P2)** — If indexed content contains PII, the vault response
  SHALL apply **data masking** (Vol 04). *Source:* Patent §9 (claim 2).

- **EKOS-03-079 (Should, P2)** — The vault SHALL offer **one-click permanent delete** of
  itself and all indexes; and its availability SHALL be controllable by a **remote
  enterprise governance policy** (can be disabled per tenant). *Source:* Patent §9
  (claims 3, 4). *Depends:* Vol 05.

---

## 18. Sandeep Genie — Living AI Avatar

- **EKOS-03-080 (Could, P3)** — EKOS SHALL offer **"Sandeep Genie"**, a photorealistic
  3D AI avatar with **voice command and chat** that can control every feature and act
  as the conversational face of the ambient AI. *Source:* blueprint §4 (Sandeep Genie).

- **EKOS-03-081 (Could, P3)** — Sandeep Genie SHALL be available in the **paid Pro
  Viewer** (monetization in Vol 13); it is an interface layer over the same governed,
  security-scoped engine (no extra access). *Source:* blueprint §4; EKR_Research
  (Viewer Pro). *Depends:* Vol 07, Vol 13.

- **EKOS-03-082 (Could, Frontier)** — Voice interaction SHALL also serve accessibility
  and hands-free field use; transport/runtime is pluggable. *Source:* derived;
  blueprint §20 (voice frontier).

---

## 19. AI Analyst Marketplace Hooks (Engine-Side; Economy in Vol 13)

- **EKOS-03-083 (Could, P3)** — Third parties SHALL be able to **publish AI Analysts**
  (e.g., "Zero-Day Threat Analyst," "Scrum Health Analyst") that run on the graph and
  are sold as subscriptions/usage-based; the engine provides the **run framework**, the
  economy is Vol 13. *Source:* research(AI Analyst Economy); ConfusionClarity.

- **EKOS-03-084 (Could, P3)** — Published analysts SHALL declare permissions and run
  **sandboxed** under the same security projection (no analyst sees beyond the invoking
  user's scope). *Source:* research(marketplace governance); Vol 04, Vol 13.

---

## 20. Performance, Cost & Reliability

- **EKOS-03-085 (Must, MVP)** — The layered design SHALL keep the engine **real-time and
  affordable at enterprise scale** (thousands of concurrent widgets) by doing most work
  in Layers 1–2 and caching Layer-3 output. *Source:* research(big-bang failure table;
  Layer 3 caching).

- **EKOS-03-086 (Should, P2)** — Layer-3 large-model calls SHALL be **rate-limited and
  budgeted** per tenant, with graceful degradation to Layer-1/2 insights if the LLM is
  unavailable. *Source:* derived from research(cost concerns) + air-gapped resilience
  (EKOS-03-059).

- **EKOS-03-087 (Could, P3)** — Over time, accumulated (consented, anonymized)
  interactions MAY fund **distilling a smaller in-house model** to replace expensive
  public-LLM calls for specific explanation tasks (year-2 cost optimization). *Source:*
  research("when do we need to train" / "year-2 optimization").

---

## 21. Constitution Conformance (AI-specific)

- **EKOS-03-088 (Must, MVP)** — Promptless by default (Constitution §5): the widget is
  the prompt; the user never has to author a prompt to get value. *Source:* Constitution
  §5.

- **EKOS-03-089 (Must, MVP)** — Facts from deterministic layers; LLM never invents
  critical facts (Constitution §5). *Source:* Constitution §5; EKOS-03-006/026.

- **EKOS-03-090 (Must, MVP)** — No training on customer content; air-gap excludes only
  internet-only features (e.g., Horizon Scanner's external feeds); provider/model names
  illustrative & pluggable (Constitution §6). *Source:* Constitution §6.

---

## 22. Open Questions (parked)

- **OQ-03-A** — Concrete model choices per layer (small-model family for Layer 2;
  cloud LLM + sovereign open model for Layer 3) → Vol 15, with a quality/latency/cost
  benchmark.
- **OQ-03-B** — Where the symbolic rule language lives (in-graph rule nodes vs. external
  OPA/Rego) and how it overlaps with Vol 05 policy-as-code — needs a unification ruling.
- **OQ-03-C** — Training-data sourcing & licensing for Layer-2 synthetic datasets
  (public TOGAF/cloud refs, GitHub repos) — legal review.
- **OQ-03-D** — On-device inference footprint for Sandbox Vault & sovereign Layer-3 on
  commodity laptops — feasibility pilot.
- **OQ-03-E** — Boundary between this volume's "AI governance hooks" and Vol 05's
  "governance program" — confirm no duplication when Vol 05 is written.

---

## 23. Requirement Index (this volume)

EKOS-03-001 … EKOS-03-090 (stable numbering; the 4-layer detail, analyst catalog,
example engine, sovereign layer, and sandbox vault expand to 94 distinct stated rules).

> **Next volume:** `04 — Security, Privacy & Graph-Level ABAC + Visual Masking`.
> Per protocol: STOP here. Await user "proceed".
