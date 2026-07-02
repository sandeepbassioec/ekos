# EKOS Volume 20 — Knowledge Representation Runtime (KRR)
### *formerly "Knowledge Projection Engine" — backward-compatible alias retained for all EKOS-20-* citations*

> **Status:** ✅ Done · **Requirement count:** ~72 · **Updated:** 2026-07-01 (Opus)
> **Gap-closure #3** (cross-architect review) + **Law 16 expansion** (2026-07-01).
> Projection — "everything is a projection" (Book 1 Law 3) — is one of EKOS's biggest
> inventions, but "Projection Engine" named only one operation inside a much larger
> subsystem. **Book 1 Law 16 (Representation Independence)** crystallizes the full insight:
> representations are ephemeral; only canonical knowledge persists. This volume now covers
> the complete **Knowledge Representation Runtime (KRR)**: the JIT engine plus rendering,
> interaction, navigation, animation, streaming, accessibility, localization, themes, and
> **Intent-Based Representation** — a knowledge object declares how it wants to be
> rendered; applications no longer own rendering. Requirements, résumé, architecture,
> books, dashboards, compliance — **all are runtime manifestations of one canonical
> knowledge graph.** "Projection Engine" is a backward-compatible alias.
> **Sources:** ChatGPT (#3 — "Projection ... deserves its own subsystem ... Everything is
> a projection"); Book 1 Law 3; Vol 01 §11 (the projection contract); Vol 10 §3 (Policy-
> Driven Projection); Vol 02 (widget rendering primitive); Vol 19 (AOT compiler boundary);
> Vol 04 (PDP scoping). Honors [BOOK-1](BOOK-1-KNOWLEDGE-CONSTITUTION.md).

---

## 1. Scope & Boundary

**In scope:** the **Projection** abstraction (selection + transform + visual-recipe +
viewer-scope); the **catalog of projection types**; **Policy-Driven Knowledge
Projection**; the **JIT view-generation pipeline**; **reactive/live** projections;
**projection composition**; **multi-modal** projection; **caching & invalidation**; the
**Projection Registry**; and the engine's relationship to the Widget Engine, Compiler,
and AI.

**Explicitly deferred (cross-referenced):**
- **The projection *contract*** (a widget = persisted query + visual recipe) → **Vol 01
  §11** (this engine *executes* that contract live, per viewer)
- **Single-widget *rendering primitive*** → **Vol 02** (the engine *orchestrates*
  widgets; it does not re-implement rendering)
- **AOT, durable artifact production** → **Vol 19 Compiler** (this engine is JIT/live;
  see §13)
- **The viewer apps that *display* projections** → **Vol 07**
- **ABAC policy/masking *rules*** → **Vol 04** (the engine *applies* the PDP)
- **PKS as the canonical space behind résumé/CV projections** → **Vol 10** (the engine
  *serves* its projections)
- **AI analyst *behavior*** → **Vol 03** (crux/explain are themselves projections served
  here; Vol 03 implements the analysts)
- **Kernel/syscalls** → **Vol 18/22** (the engine is a user-space kernel client)

---

## 2. Core Principle — Everything Is a Projection

- **EKOS-20-001 (Must, MVP)** — **Everything a user sees SHALL be a projection** of the
  canonical knowledge graph — a requirement view, a résumé, an architecture diagram, a
  dashboard, a compliance report, a book — never an independent artifact (Book 1 Laws
  2/3). *Source:* ChatGPT (#3); Book 1 Law 3.

- **EKOS-20-002 (Must, MVP)** — The Projection Engine SHALL be the **single runtime
  subsystem** that turns "knowledge + a projection definition + a viewer's context" into
  a **live view**, on demand. *Source:* ChatGPT (#3 — "deserves its own subsystem").

- **EKOS-20-003 (Must, MVP)** — The Projection Engine is **JIT (just-in-time, runtime)**;
  the Compiler (Vol 19) is **AOT (build-time)**. They are distinct subsystems that share
  machinery (selection, scoping, layout) but differ in lifecycle: live views vs durable
  artifacts. *Source:* Vol 19 §9; ChatGPT (#2/#3).

- **EKOS-20-004 (Must, MVP)** — The engine SHALL be a **user-space client of the kernel**
  (Vol 18), reading via PDP-scoped query syscalls (Vol 22). *Source:* Vol 18 §12; Vol 22.

---

## 3. The Projection Abstraction

- **EKOS-20-005 (Must, MVP)** — A **Projection** SHALL be a first-class, named, versioned
  object (Vol 01) composed of: a **selection** (declarative graph query), a **transform**
  (shaping/aggregation), a **visual/output recipe** (how it renders), and a **scope**
  (per-viewer/policy). *Source:* Vol 01 §11 (contract); Book 1 Law 3.

- **EKOS-20-006 (Must, MVP)** — A Projection SHALL be **parameterizable & reusable** —
  one definition ("C4 Container view", "hiring résumé view") serves many instances/
  subjects. *Source:* Vol 01 EKOS-01-058; Vol 02 EKOS-02-007.

- **EKOS-20-007 (Should, P2)** — A Projection definition SHALL be **versioned & traceable**
  (Vol 01 §7), so a view's evolution is auditable and a past view is reconstructable via
  time-travel. *Source:* Vol 01 §7.

---

## 4. Catalog of Projection Types

- **EKOS-20-008 (Must, MVP)** — The engine SHALL serve, as projections (not bespoke
  features): **requirement views, story maps, architecture views (C4/sequence/ER),
  dashboards, compliance reports, audit-evidence views, résumé/CV/PKS projections (Vol
  10), Living Books (Vol 24), executive overviews, and widget screens**. *Source:*
  ChatGPT (#3 — "Requirements. Resume. Architecture. Books. Dashboards. Compliance.");
  Vol 10; Vol 14.

- **EKOS-20-009 (Must, MVP)** — Adding a new "view type" SHALL be **defining a new
  projection**, never building a new bespoke artifact (Book 1 Law 1). *Source:* Book 1
  Law 1; Vol 02 §11.

---

## 5. Policy-Driven Knowledge Projection

- **EKOS-20-010 (Must, P2)** — **Different viewers SHALL see different projections of the
  same canonical knowledge**, decided by ABAC/policy — a recruiter sees a hiring view, a
  university an academic view, an auditor a compliance view, all from one source. *Source:*
  ChatGPT (Policy-Driven Knowledge Projection patent family); Vol 10 §3; Vol 16 §P16.

- **EKOS-20-011 (Must, P2)** — Every projection SHALL be **PDP-scoped & masked** (Vol 04)
  — restricted elements are stripped/masked before the view leaves the engine (server-
  side, zero-trust). *Source:* Vol 04 §2/§5; Vol 01 EKOS-01-056; Book 1 Law 4.

- **EKOS-20-012 (Should, P2)** — Policy-Driven Projection SHALL be the runtime basis for
  intra-document partial restriction (Vol 04 §6) — the same view masks different
  sub-nodes for different viewers. *Source:* Vol 04 §6.

---

## 6. JIT View-Generation Pipeline

- **EKOS-20-013 (Must, MVP)** — On a view request, the engine SHALL: **(1) resolve the
  projection definition → (2) PDP-scope to the viewer → (3) query the graph → (4)
  transform/shape → (5) bind to widgets/layout → (6) emit the live view**. *Source:*
  synthesis of Vol 01 §11 + Vol 02 + Vol 04; ChatGPT (#3).

- **EKOS-20-014 (Must, MVP)** — Step (5) SHALL **orchestrate the Widget Engine (Vol 02)**
  as the rendering primitive and the **deterministic layout engine (Vol 01 §12)** — the
  Projection Engine composes; the Widget Engine renders. *Source:* Vol 02; Vol 01 §12.

- **EKOS-20-015 (Should, P2)** — The pipeline SHALL support **multiple output channels**
  for a live view: interactive web, the EKR Viewer (Vol 07), or a streamed/embedded
  surface — same projection, different live channel. *Source:* Vol 07; Book 1 Law 3.

---

## 7. Reactive / Live Projections

- **EKOS-20-016 (Must, P2)** — Projections SHALL be **reactive**: subscribed to the
  kernel mutation journal (Vol 18 §8), they **re-evaluate and update the live view** when
  underlying knowledge changes — the "self-updating widget" property. *Source:* Vol 02
  EKOS-02-005; Vol 18 §8; Book 1 Law 3.

- **EKOS-20-017 (Should, P2)** — Reactive updates SHALL respect the viewer's PDP scope on
  every re-evaluation (a newly-restricted node masks live). *Source:* Vol 04; §5.

- **EKOS-20-018 (Should, P2)** — Live collaboration (Vol 08 §10) SHALL operate over the
  reactive projection — co-viewers see synchronized, individually-scoped projections.
  *Source:* Vol 08 §10; Vol 12.

---

## 8. Projection Composition

- **EKOS-20-019 (Must, P2)** — Projections SHALL **compose** — a screen/dashboard/document
  is a composition of projections (Screen Blueprints, Vol 08 §3, are compositions).
  *Source:* Vol 08 §3; Vol 02 §13.

- **EKOS-20-020 (Should, P2)** — Composed projections SHALL support **cross-projection
  interactivity** (selection in one updates another) since all read one graph. *Source:*
  Vol 02 §13; research(glue).

- **EKOS-20-021 (Should, P2)** — Composition SHALL never elevate access — a composed view
  shows only the intersection the viewer is authorized for. *Source:* Vol 04; Vol 06 §17.

---

## 9. Multi-Modal Projection

- **EKOS-20-022 (Should, P2)** — Modality SHALL be a **projection choice**: the same
  knowledge can project as **visual (graph/diagram), textual (prose/document), tabular
  (data), audio (narration), or 3D/AR-VR** — chosen by the recipe/context. *Source:*
  Book 1 Law 3; blueprint §9 (media); Vol 03 §10 (role-adaptive). *Relates:* Vol 06 §23.

- **EKOS-20-023 (Should, P2)** — Accessibility renderings (screen-reader/textual) SHALL be
  first-class projection modalities, never an afterthought (Vol 09 §5). *Source:* Vol 09
  §5; Book 1.

---

## 10. Caching & Invalidation

- **EKOS-20-024 (Must, P2)** — Projection results SHALL be **cached keyed on (projection
  def + scope + underlying subgraph state)** and **invalidated by relevant mutation
  events** (Vol 18 §8) — JIT performance without staleness. *Source:* Vol 01 §15; Vol 03
  §6 (caching); Vol 18 §8.

- **EKOS-20-025 (Should, P2)** — Caching SHALL be **per-scope** so one viewer's authorized
  cache is never served to an unauthorized viewer. *Source:* Vol 04; security.

---

## 11. Projection Registry

- **EKOS-20-026 (Should, P2)** — Named projection definitions SHALL live in a **Projection
  Registry** — discoverable, reusable, versioned, shareable, and **marketplace-publishable**
  (a "compliance report projection" or "hiring résumé projection" as a product). *Source:*
  Vol 13 (marketplace); Vol 02 §12; Book 1 Law 1.

- **EKOS-20-027 (Should, P2)** — Registry projections SHALL carry license/governance
  metadata (Vol 05/13) and be sandboxed if they contain custom transform/render code
  (Vol 02 §7). *Source:* Vol 05; Vol 13; Vol 02 §7.

---

## 12. Projection & The Widget Engine

- **EKOS-20-028 (Must, MVP)** — Clear division: **Widget Engine (Vol 02) = the rendering
  primitive** for a single widget; **Projection Engine = the orchestrator** that selects,
  scopes, composes, and binds widgets into a complete, per-viewer view. *Source:*
  synthesis; ChatGPT (#3 — projection vs widget engine).

- **EKOS-20-029 (Should, P2)** — A single widget *is* the simplest projection (one
  selection + one recipe); a dashboard is a composed projection — same abstraction, scaled.
  *Source:* §3/§8.

---

## 13. Projection & The Compiler

- **EKOS-20-030 (Must, P2)** — The **Compiler (Vol 19) materializes a projection into a
  durable artifact** (PDF/`.ekr`/Living Book) at build time; the **Projection Engine
  serves it live** at runtime. A **Live Bundle** (Vol 06 §12) is the bridge — a compiled
  artifact whose projections re-evaluate at runtime. *Source:* Vol 19 §9; Vol 06 §12.

- **EKOS-20-031 (Should, P2)** — The two subsystems SHALL **share projection definitions,
  scoping, and layout machinery** to guarantee a compiled artifact and a live view of the
  same projection look identical. *Source:* Vol 19 §9; consistency principle.

---

## 14. Projection & AI

- **EKOS-20-032 (Should, P2)** — AI outputs are themselves projections: the **crux panel,
  example, Chain-of-Thought visualizer** (Vol 03) are projections of the AI's reasoning +
  the underlying graph, served by this engine within the viewer's scope. *Source:* Vol 03
  §10/§14; Book 1 Law 3/6.

- **EKOS-20-033 (Should, P2)** — **Role-adaptive projection** (Vol 03 §10 — "explain to a
  CFO / a 5-year-old / a developer") SHALL be implemented as alternate projections of the
  same knowledge. *Source:* Vol 03 §10; ChatGPT (#3).

---

## 15. Self-Hosting & Constitution Conformance

- **EKOS-20-034 (Could, P3)** — The EKOS spec itself SHALL be viewable as projections
  (the requirements as a navigable view, the architecture as a diagram) — EKOS projects
  EKOS (Vol 25). *Source:* Book 1 Law 15; Vol 25.

- **EKOS-20-035 (Must, MVP)** — Projections honor **single-source-of-truth (Law 2),
  everything-is-projection (Law 3), crypto-hardness/PDP scoping (Law 4), grounded AI
  (Law 6)**; recipe/render libraries are **illustrative & pluggable (Law 10)**. *Source:*
  Book 1 Part II.

---

## 16. Representation Independence & The Knowledge Representation Runtime (KRR)

> *Law 16 (added 2026-07-01) crystallizes a first-principles insight: "Projection Engine"
> names one operation; the real subsystem is a full **Representation Runtime** — of which
> projection is the first capability. This section expands the scope accordingly.*

- **EKOS-20-036 (Must, MVP)** — **Representations are ephemeral by default** (Book 1
  Law 16): no book, diagram, dashboard, architecture view, presentation, résumé, or
  compliance report SHALL be stored as a frozen artifact. The only things persisted are
  the canonical knowledge objects plus the projection/representation rules. Storage is
  *radically* smaller; consistency is *structurally* guaranteed. *Source:* Book 1 Law 16;
  ChatGPT first-principles analysis; Law 2/3.

- **EKOS-20-037 (Must, P2)** — The sole persistence exception is the **immutable evidence
  artifact**: a representation that has crossed a legal, regulatory, contractual, archival,
  or interoperability boundary (filed PDF, signed contract, sent invoice, patent submission,
  government form, audit trail). Evidence artifacts SHALL be stored once, immutably, with:
  (a) provenance link to the source KOs, (b) creation timestamp + reason, (c) creating
  actor identity, (d) cryptographic hash for tamper evidence. They are never edited; only
  superseded by a new evidence artifact. *Source:* Book 1 Law 16; Law 11 (bitemporal).

- **EKOS-20-038 (Should, P2)** — The runtime layer that executes representations SHALL be
  logically split into two concerns: the **Knowledge Representation Framework (KRF)** and
  the **Knowledge Representation Runtime (KRR)**. KRF defines the *contracts* (what a
  representation is, its semantics, accessibility contract, policies, fidelity guarantees,
  metadata schema). KRR *executes* those contracts at runtime. Projection is the first
  capability inside KRF/KRR, not the whole of it. *Source:* ChatGPT KRF/KRR analysis.

- **EKOS-20-039 (Should, P2)** — The KRR SHALL support the full representation lifecycle
  beyond projection: **rendering, interaction, navigation, animation, streaming, responsive
  layout, localization, theme/branding application, accessibility realization, and
  incremental/on-demand updates**. Each is a distinct capability inside the runtime, not
  a separate subsystem. *Source:* ChatGPT KRR responsibilities; Vol 09 §5 (a11y); Vol 08
  (branding/themes); Vol 07 (viewer family as consumers of KRR).

- **EKOS-20-040 (Must, P2)** — **Intent-Based Representation** (patent candidate): a KO
  SHALL be able to declare *representation intent* — "render me as a C4 diagram", "render
  me as a 10-minute executive presentation", "render me as a Living Book" — and the KRR
  fulfills the intent by selecting and executing the appropriate projection/rendering
  recipe. This inverts the classical export model: **knowledge requests its own
  representation**; applications do not own rendering. *Source:* ChatGPT "knowledge owns
  rendering" insight; Book 1 Law 3. *Patent candidate:* Vol 16 §P-NEW (see §18 below).

- **EKOS-20-041 (Should, P3)** — When AI creates a presentation, report, or book, it SHALL
  express the output as a **Representation Intent** ("create a Presentation Projection for
  Q4 board review") rather than editing a file format. The KRR executes the intent; the AI
  never directly edits a stored artifact. *Source:* Book 1 Law 6 (AI grounds on facts, not
  files); EKOS-20-040.

- **EKOS-20-042 (Should, P2)** — The Projection Registry (§11) SHALL evolve into a
  **Representation Contract Registry** — a catalog of named, versioned representation
  contracts (KRF definitions) that any KO can reference. Contracts are composable,
  marketplace-publishable, and governance-tracked. *Source:* §11; EKOS-20-038.

- **EKOS-20-043 (Must, P2)** — Vol 20's subsystem SHALL be published and referenced
  throughout EKOS documentation as **"the Knowledge Representation Runtime (KRR)"**, with
  "Projection Engine" as a backward-compatible alias. Volume name preserved for stability
  of existing citation identifiers (EKOS-20-*). *Source:* Book 1 Law 14 (stable; not
  rewrite).

---

## 17. Patent Candidate — Intent-Based Representation

- **EKOS-20-044 (Should, P3)** — A new patent family SHALL be drafted: **"Intent-Based
  Knowledge Representation"** — a system and method in which a knowledge object declares
  a representation intent (target format, audience, fidelity, channel), and a
  Representation Runtime resolves the intent to a recipe and executes it, without the
  knowledge object storing or knowing the representation format. Distinguishes from prior
  art (export pipelines, XSLT transforms, template engines) by: (a) the intent is
  semantically declared on the *object*, not the *application*; (b) the runtime is
  audience- and policy-aware (different viewers get different realizations of the same
  intent); (c) evidence artifacts are a typed exception, not the default. *Source:*
  EKOS-20-040/041; Vol 16 (patent catalog).

---

## 18. Open Questions (parked)

- **OQ-20-A** — Shared-machinery boundary between Projection Engine / KRR (JIT) and
  Compiler (AOT) — exactly what is shared vs duplicated; design with Vol 19.
- **OQ-20-B** — Reactive-projection update transport at scale (push vs poll over the
  mutation journal) — with Vol 18 §8 / Vol 15 edge-signaling.
- **OQ-20-C** — Projection caching invalidation precision (sub-graph granularity) vs cost
  — with Vol 01 §15 benchmark.
- **OQ-20-D** — Multi-modal (audio/AR-VR) projection scope for MVP vs frontier — phase
  decision (Vol 17 §8 frontier).
- **OQ-20-E** — Projection-definition language / Representation Contract language
  (openCypher/GQL + recipe DSL) — with Vol 01 OQ-01-C / Vol 02 OQ-02-A.
- **OQ-20-F** — KRF/KRR split as separate sub-volumes vs single-volume distinction —
  decide when implementation begins (P2); interim: single volume, clear conceptual split.

---

## 19. Requirement Index (this volume)

EKOS-20-001 … EKOS-20-044 (stable numbering; the projection abstraction, type catalog,
policy-driven projection, JIT pipeline, reactive/live, composition, multi-modal, caching,
registry, widget/compiler/AI relationships, self-hosting, **representation independence,
KRF/KRR split, intent-based representation, evidence-artifact exception** — expand to
~72 distinct stated rules).

> **Volume 20 updated 2026-07-01:** Law 16 (Representation Independence) enacted in Book 1;
> KRR scope expanded; Intent-Based Representation patent candidate added (EKOS-20-044);
> ADR-050 in ADR Register. All prior EKOS-20-* identifiers stable.
