# EKOS Volume 02 — Widget Engine & Custom Widget OS (+ Process Capture AI)

> **Status:** ✅ Done · **Requirement count:** 88 · **Generated:** 2026-06-30 (Opus)
> **Sources mined:** `blueprint §2–3` (Living Widget Ecosystem, Custom Widget OS),
> `research_across_llms.txt` (Custom Widget Framework Part 2, Widget Engine
> architecture, widget-mapping JSON), `Patents.txt` §11 (Unified Widget OS &
> Builder), §5 (Process Capture AI), §10 (Living Canvas Mode), `Gemini critique`
> (native rendering engine). Honors [00-MASTER-INDEX](00-MASTER-INDEX.md) §2.

---

## 1. Scope & Boundary

**In scope:** the engine that turns graph subgraphs into interactive, traceable
visual widgets; the **anatomy/contract** of a widget; the high-performance
**rendering engine**; **Living Canvas** (fixed-layout) vs responsive modes; **export**
to Word/HTML/PDF; the **no-code Custom Widget OS** (Data Schema / Visual
Representation / Behavior & AI layers); **Process Capture AI** (record manual
Excel/PPT/email → living widget); widget-as-first-class-citizen; widget-level
accessibility & i18n the builder must enforce; and the **platform-capability >
feature** principle proven through the "resume = projection" case.

**Explicitly deferred (cross-referenced, not duplicated):**
- The graph, ontology, `WidgetDefinition` node storage & versioning, persisted-query
  semantics, server-side layout engine, security projection on query → **Vol 01**
  (this volume *consumes* those; it does not redefine them)
- AI analyst depth, the 4-layer engine, crux/explain generation → **Vol 03** (this
  volume defines only the `ai_hooks` *binding surface* on a widget)
- ABAC masking *behavior* inside a widget → **Vol 04** (this volume mandates widgets
  honor the masked projection; it does not define the masking logic)
- Marketplace transactions, pricing, revenue share, certification → **Vol 13** (this
  volume defines only publish/consume *hooks*)
- The actual **catalog** of concrete widgets (Story Map, C4, Threat Model, etc.) →
  **Vol 14** (this volume is the *engine*, not the catalog)
- **Personal Knowledge Space** (the generalized "human profile" of which a résumé is
  one projection) → **Vol 10** (this volume only states the principle that makes it
  inevitable — see §11)
- Real-time multi-user co-editing UX (presence/ghost-locks) → **Vol 08**
- Full table-stakes accessibility/i18n program → **Vol 09** (this volume covers only
  what the *widget builder* must enforce at authoring time)

---

## 2. Widget Definition & Anatomy

> "Every widget is merely a projection of a subgraph rendered with a specific visual
> behavior." A widget is NOT a document — it is a live, data-bound, version-controlled
> object.

- **EKOS-02-001 (Must, MVP)** — A widget SHALL be defined by exactly four parts: a
  **data query** (declarative graph query, per Vol 01 §11), a **visual mapping** (how
  result nodes/edges become shapes/positions/colors), an **interaction schema** (what
  happens on click/drag/hover/expand), and **AI hooks** (which proactive insights to
  surface). *Source:* research(Widget Engine §; widget-mapping JSON); Patent §11.

- **EKOS-02-002 (Must, MVP)** — Each widget definition SHALL be persisted as a
  `WidgetDefinition` graph node (Vol 01 §4/§18), making it itself **versioned,
  searchable, and traceable** — never a binary blob. *Source:* Patent §11 (claim 2);
  research(custom widget framework).

- **EKOS-02-003 (Must, MVP)** — A widget SHALL be a **first-class, ID-bearing object**
  so it can be traced from requirement → story → architecture → code → test (it
  participates in `traces_to`). *Source:* research(intro — "each widget is a
  first-class object with its own ID"); Vol 01 §5.

- **EKOS-02-004 (Must, MVP)** — Editing a widget SHALL be a **semantic change to the
  underlying graph, not a pixel change** — moving a box updates layout metadata;
  changing a label updates the node's property; the meaning stays intact and
  traceable. *Source:* research(glue § — "semantic change, not a pixel change").

- **EKOS-02-005 (Must, MVP)** — Widgets SHALL be **data-bound and self-updating**: when
  the underlying graph changes (via any path, incl. connectors), open widgets reflect
  it (driven by the Vol 01 §13 mutation event stream + cache invalidation). *Source:*
  research(living documentation §); Patent §5 ("automatically update in response to
  changes").

- **EKOS-02-006 (Must, MVP)** — Hovering/selecting any element in a widget SHALL be
  able to **highlight related items across all other widgets and phases**
  (bidirectional traceability surfaced visually). *Source:* research(glue §).

- **EKOS-02-007 (Should, P2)** — A single `WidgetDefinition` SHALL be
  **instantiable many times** with different parameters (one "C4 Container Diagram"
  definition serves every bounded context). *Source:* research(widget mapping); Vol 01
  EKOS-01-058.

---

## 3. Widget Rendering Engine (Performance)

> ChatGPT/Gemini: the host UI framework must NEVER try to render 100,000-node graphs
> itself. It hosts a native high-performance engine.

- **EKOS-02-008 (Must, MVP)** — The rendering engine SHALL render high-density
  interactive graphs (thousands of vector paths, nodes, animated edges) at **smooth
  interactive frame rates (target 60 fps)** for zoom, pan, and drill-down on
  commodity hardware including tablets. *Source:* Gemini critique (High-Performance
  Visual Compilation).

- **EKOS-02-009 (Must, P2)** — Heavy graph layout/draw SHALL run on a **native,
  GPU-accelerated canvas** (WebGL/WebGPU / native Skia/Impeller-class), compiled from
  a Rust/C++/WASM core, **mounted as a single high-performance surface** inside the
  host UI shell; the host framework only hosts the engine, it does not lay out the
  graph node-by-node. *Source:* Gemini critique; ChatGPT ("rendering engine should be
  Rust/C++ WebGPU WASM ... Flutter only hosts").

- **EKOS-02-010 (Must, MVP)** — The rendering engine SHALL consume **server-computed
  deterministic layout coordinates** (Vol 01 §12 layout engine) so large workspaces
  auto-arrange readably without thrashing the client. *Source:* Gemini(layout);
  Vol 01 EKOS-01-061.

- **EKOS-02-011 (Should, P2)** — The engine SHALL support **rich visual behaviors**:
  packet/flow animation (dots traversing edges), state pulsing/conditional styling,
  drill-down zoom (C4-style Context→Container→Component→Code), and timeline scrubbing.
  *Note:* per-widget use of these is Vol 14; the engine must offer them as primitives.
  *Source:* research(architecture §packet animation; UML drill-down §).

- **EKOS-02-012 (Should, P2)** — The same rendering engine core SHALL be **reusable in
  the EKR Viewer** (Vol 07) so authored and consumed widgets look pixel-identical with
  edit APIs simply disabled. *Source:* EKR_Research(Flutter visual reuse); ChatGPT
  (viewer family shares engine). *Depends:* Vol 07.

---

## 4. Living Canvas Mode (Fixed-Layout + Responsive)

> Patent §10 — PDF-grade visual fidelity WITHOUT sacrificing interactivity.

- **EKOS-02-013 (Should, P2)** — The viewer SHALL support a **Living Canvas (fixed-
  layout) mode**: widgets positioned on a virtual page with absolute coordinates
  (e.g., A4 @ 300 DPI), rendered scaled-to-fit, maintaining exact relative positions
  on any screen size — like a PDF, but interactive. *Source:* Patent §10 (claim 1).

- **EKOS-02-014 (Should, P2)** — In fixed-layout mode, widgets SHALL **remain fully
  interactive** (drill-down, animation, AI invocation, real-time updates of other
  widgets). *Source:* Patent §10 (claim 1, 4).

- **EKOS-02-015 (Should, P2)** — The viewer SHALL also support a **responsive mode**
  that reflows the same widgets to the viewport, and a user SHALL switch modes
  **without altering the underlying document**. *Source:* Patent §10 (claim 2).

- **EKOS-02-016 (Should, P2)** — Because the canvas is defined in absolute terms,
  **export to static PDF SHALL be lossless** (layout already in physical-page
  coordinates). *Source:* Patent §10 (claim 3).

- **EKOS-02-017 (Should, P2)** — Interaction in fixed-layout mode SHALL translate
  viewport coordinates back to the virtual coordinate space so clicks map correctly at
  any scale. *Source:* Patent §10 (detailed description).

---

## 5. Export & Interop

> User requirement: "if they want, export it to Word, HTML or PDF as well."

- **EKOS-02-018 (Must, MVP)** — Any widget or composed view SHALL be **exportable to
  Word, HTML, and PDF**, rendering the visual model as the exported artifact.
  *Source:* research(architecture domains § — "if they want export it to word, html or
  pdf"); blueprint.

- **EKOS-02-019 (Must, MVP)** — Export SHALL use the **same user-scoped security
  projection** as the live view, so restricted/masked content never leaks into an
  exported file. *Source:* research(Security §3 — "export to PDF/Word uses the same
  security projection"); Vol 01 EKOS-01-056/068; Vol 04.

- **EKOS-02-020 (Should, P2)** — Export SHALL be distinct from the open-standard
  **semantic export** (JSON-LD/RDF + offline player) of Vol 01 §14: Word/HTML/PDF are
  *human-facing renders*; JSON-LD/RDF is the *no-lock-in data escrow*. Both SHALL be
  available. *Source:* Gemini(no-lock-in) + research(export); Vol 01 EKOS-01-066/067.

- **EKOS-02-021 (Could, P3)** — HTML export SHALL be **self-contained and openable
  offline** (a lightweight viewer bundle), bridging toward the EKR Viewer family.
  *Source:* EKR_Research(offline web viewer); Vol 07.

---

## 6. Custom Widget OS — Layer A: Data Schema

> "Let the world build the missing 95%." Three layers; this is the first.

- **EKOS-02-022 (Must, MVP)** — A **no-code Data Schema builder** SHALL let any
  knowledge worker define the atomic units of a widget (cards / nodes / rows): add
  fields with datatypes (text, number, date, dropdown, …). *Source:* research(Custom
  Widget Framework — Data Schema Layer); Patent §11 (claim 1).

- **EKOS-02-023 (Must, MVP)** — A field MAY be a **reference to another widget/graph
  type** — this is the **traceability hook**: populating it auto-creates a typed
  bidirectional graph edge (Vol 01 EKOS-01-047). *Source:* research(Data Schema Layer
  — "reference to another widget type ... traceability hook").

- **EKOS-02-024 (Must, MVP)** — Defining a schema SHALL **create/extend ontology
  meta-nodes at runtime** (Vol 01 §9), so the custom type is immediately a first-class,
  queryable, AI-aware citizen with no platform redeploy. *Source:* Patent §11 (claim
  1); Patent §3; research(future-proofing §1).

- **EKOS-02-025 (Should, P2)** — The builder SHALL **AI-suggest a schema** by reading
  an uploaded Excel/CSV or by the user describing the process in natural language.
  *Source:* research(Data Schema Layer — "AI can suggest a schema by reading your
  existing Excel sheet"); Patent §11 (detailed description). *Depends:* Vol 03.

- **EKOS-02-026 (Should, P2)** — Custom types SHALL **inherit** traceability, temporal
  (bitemporal), and security-attribute applicability from their chosen parent type
  automatically (Vol 01 EKOS-01-010). *Source:* Patent §3 (claim 4).

---

## 7. Custom Widget OS — Layer B: Visual Representation

- **EKOS-02-027 (Must, MVP)** — The builder SHALL offer a **library of preset visual
  templates**: card grid, kanban, timeline, flowchart, tree, table, force-directed
  graph, map, heatmap, dashboard panels, Gantt, matrix, radar, **packet animation**,
  3D network, sunburst/treemap, etc. *Source:* research(Visual Representation Layer;
  Widget Builder OS §); blueprint §3.

- **EKOS-02-028 (Must, MVP)** — Users SHALL **drag-and-drop map data fields to visual
  properties** (color, size, position, icon) of the chosen template. *Source:*
  research(Visual Representation Layer).

- **EKOS-02-029 (Must, MVP)** — The builder SHALL support **conditional formatting**
  rules (e.g., "Risk = High → red pulsing border"). *Source:* research(Visual
  Representation Layer; future-proofing — conditional styling).

- **EKOS-02-030 (Should, P2)** — Advanced users SHALL be able to write a **custom
  rendering function** (JavaScript/Python) to produce wholly novel visuals beyond the
  preset templates, executed in a sandbox (security/sandboxing per marketplace rules,
  Vol 13). *Source:* research(Visual Representation Layer — "scriptable rendering");
  blueprint §3 (20+ templates + custom).

- **EKOS-02-031 (Should, P2)** — A **community library of visuals** SHALL be available
  so authors reuse rendering recipes without coding. *Source:* research(Visual
  Representation Layer — "vast library of community visuals").

---

## 8. Custom Widget OS — Layer C: Behavior & AI

- **EKOS-02-032 (Must, MVP)** — Authors SHALL define **interactivity** per widget:
  click actions (navigate to linked widget, show details, open a form, trigger a
  workflow), drag, hover. *Source:* research(Behavior & AI Layer); Patent §11 (claim
  1, interaction).

- **EKOS-02-033 (Must, MVP)** — Authors SHALL define **automations**: "when this field
  changes, notify this channel"; "every Monday, generate a PDF snapshot for the
  client"; trigger a webhook/workflow on a state change. *Note:* the workflow engine
  itself is Vol 08; this defines the per-widget trigger surface. *Source:*
  research(Behavior & AI Layer; Process Capture logic).

- **EKOS-02-034 (Must, MVP)** — Each widget SHALL expose **AI hooks** — a binding
  surface naming which proactive analysts run on it (e.g., `crux_summary`,
  `resilience_audit`, `cost_tradeoff`, `explain`, `detect_anomalies`). The *analyst
  implementations* are Vol 03. *Source:* research(widget-mapping `ai_hooks`; Behavior &
  AI Layer); Patent §11 (claim 4 — AI hook configurator).

- **EKOS-02-035 (Must, MVP)** — Toggling **"Explain"** on any widget (custom or built-
  in) SHALL produce a plain-language, context-aware explanation/example **without the
  user typing a prompt** — the widget IS the prompt (Constitution §5). *Source:*
  research(Behavior & AI Layer — "Toggle 'Explain' ... AI generates a plain-language
  example"); Patent §11 (claim 4). *Depends:* Vol 03.

- **EKOS-02-036 (Must, MVP)** — **Traceability autowiring:** when a field is declared
  as a reference type and a source object is dropped onto it (e.g., drag a story card
  onto a custom widget's story field), the bidirectional link is created and persists
  forever. *Source:* research(Behavior & AI Layer — "Traceability autowiring");
  EKOS-02-023.

- **EKOS-02-037 (Should, P2)** — The AI SHALL operate **within the requesting user's
  security projection** — it cannot explain or surface what the user is not authorized
  to see (no leakage via AI). *Source:* research(Security §7 — "AI operates within the
  same security context"); Vol 04.

---

## 9. Process Capture AI

> Patent §5. The killer adoption feature: absorb existing Excel/PPT/email habits and
> turn them into living widgets. "Nobody is using it" problem solved permanently.

- **EKOS-02-038 (Must, P2)** — A **screen-capture/recorder module** SHALL record a
  user performing a manual routine across applications (Excel cell edits, email sends,
  PowerPoint updates) with metadata (application, cell references, timestamps), **with
  explicit user permission**. *Source:* Patent §5 (claim 1, detailed); research(Process
  Capture AI).

- **EKOS-02-039 (Must, P2)** — A **sequence analyzer** SHALL infer, from the recording,
  the underlying process: data input fields/sources, transformation logic/formulas,
  conditional logic, and output destinations — using a combination of rule-based
  pattern matching and a transformer model. *Source:* Patent §5 (claim 1, 2); research.

- **EKOS-02-040 (Must, P2)** — Transformation operations SHALL be inferred partly by
  **comparing values before and after user edits** and applying a rule-based inference
  engine. *Source:* Patent §5 (claim 2).

- **EKOS-02-041 (Must, P2)** — The system SHALL **construct a graph schema** (data
  entities + relationships) representing the captured process, then **generate an
  executable `WidgetDefinition`** that renders the result. *Source:* Patent §5 (claim
  1); research(Process Capture AI).

- **EKOS-02-042 (Must, P2)** — The generated widget SHALL **connect to live data
  sources** (via connectors / target-application APIs, with permission), **replacing
  static cell references with live connectors**, so future updates happen
  automatically and the manual chore disappears. *Source:* Patent §5 (claim 1, detailed
  — "establish live connectors, replacing static references"); research ("zero-effort,
  always-updated widget").

- **EKOS-02-043 (Should, P2)** — The AI SHALL **suggest a name and description** for the
  generated widget from the recorded context. *Source:* Patent §5 (claim 3).

- **EKOS-02-044 (Should, P2)** — The generated widget SHALL be **adjustable in the
  no-code builder** (Layers A/B/C) after generation. *Source:* research(Process Capture
  — "user can adjust the widget in a no-code builder").

- **EKOS-02-045 (Could, P3)** — Process Capture AI MAY be packaged/monetized as an
  add-on module ("AI Studio Pro"); monetization mechanics are Vol 13. *Source:*
  research(monetization §4 — Process Capture add-on).

---

## 10. Widget Accessibility & i18n (Authoring-Time Enforcement)

> Full table-stakes program is Vol 09; but the BUILDER must bake these in at authoring
> time or every custom widget becomes non-compliant.

- **EKOS-02-046 (Must, P2)** — Every widget (built-in and custom) SHALL emit
  **WCAG-2.2-AA / ARIA-compliant markup**; the widget builder SHALL include an
  **accessibility checker** (Lighthouse-class) that scores a widget **before it can be
  published**. *Source:* ConfusionClarity(table-stakes #2 — accessibility); Patent §11
  context. *Depends:* Vol 09.

- **EKOS-02-047 (Must, P2)** — Widget labels, field names, and AI explanations SHALL be
  **translatable** (i18n), rendered per the user's locale; translations stored as graph
  data (Vol 01) linked to a `Locale` concept. *Source:* ConfusionClarity(table-stakes
  #1 — i18n). *Depends:* Vol 09.

- **EKOS-02-048 (Should, P2)** — ChatGPT's anti-scraping vector (drawing text as
  vector paths) SHALL **NOT** be the default protection, because it breaks
  accessibility/searchability/screen-readers/SEO; protection SHALL be **policy-driven**
  (watermarking, authorization, rate-limiting, signed tokens, redaction, audit) and
  the rendering engine stays independent of the protection strategy. *Source:* ChatGPT
  (disagreement on vector-path scraping fix — "rendering engine should remain
  independent from the protection strategy"). *Depends:* Vol 04 (protection), Vol 10
  (resume scraping concern).

---

## 11. Platform-Capability > Feature — the "Resume = Projection" Principle

> THE governing design ruling (Constitution §1). Captured here because the Widget OS
> is where the temptation to build bespoke widgets lives. Fully expanded in Vol 10.

- **EKOS-02-049 (Must, MVP)** — Before any new "widget" is built as a bespoke artifact,
  it SHALL be evaluated against the rule **"is this a platform capability or just an
  application feature?"** — bespoke widgets are avoided where a **projection of an
  existing graph + generic widget definition** suffices. *Source:* ChatGPT(golden rule);
  Constitution §1.

- **EKOS-02-050 (Must, MVP)** — Concretely: there SHALL be **no bespoke "Resume
  Widget."** A résumé is **one projection of a generalized Personal Knowledge Space**
  (career, education, projects, skills, certifications, media, recommendations,
  patents, publications, achievements). Academic CV, immigration profile, startup
  pitch, research portfolio, artist portfolio, security-clearance record, etc. are
  **other projections of the same space** — not separate widgets. *Source:*
  ChatGPT(explicit correction — "It shouldn't exist ... the real abstraction is
  Personal Knowledge Space"). *Depends:* **Vol 10** (full PKS spec). *Flag:* Vol 10
  MUST own this in full.

- **EKOS-02-051 (Should, P2)** — Likewise, "media" (audio/video/whiteboard/animation/
  demo/simulation) SHALL be a **media-layer capability available to every Knowledge
  Object**, not a resume-specific or widget-specific feature. *Source:* ChatGPT("every
  Knowledge Object should support Media Layers"); blueprint §9 (media as first-class
  graph nodes). *Depends:* Vol 10, Vol 14.

- **EKOS-02-052 (Should, P2)** — Each accepted platform capability SHALL pass ChatGPT's
  five lenses: (1) capability vs feature, (2) creates a new category vs copies one,
  (3) survivable 20 years architecturally, (4) simplifies vs complicates the platform,
  (5) a Fortune-100 CTO can justify building their org around it. *Source:* ChatGPT
  (five evaluation lenses). *Application:* a review gate, not a runtime feature.

---

## 12. Marketplace Hooks (Engine-Side Only)

> Full economy = Vol 13. Here only what the engine must expose.

- **EKOS-02-053 (Should, P2)** — A finished `WidgetDefinition` (schema + visual mapping
  + automations + AI hooks) SHALL be **publishable to a marketplace** as a
  self-contained, shareable unit. *Source:* research(Widget Marketplace); Patent §11
  (claim 3).

- **EKOS-02-054 (Should, P2)** — Published widgets SHALL declare **permissions** (what
  data/connectors they access) so they can be **sandboxed** and security-scanned on
  import (data-exfiltration detection); the certification program is Vol 13. *Source:*
  research(monetization — "sandboxed execution ... permissions declared in the
  specification"); Patent §11 (detailed description).

- **EKOS-02-055 (Should, P2)** — Widgets SHALL be **discoverable** (search, categories,
  ratings) inside the builder/marketplace surface. *Source:* research(Widget
  Marketplace & Community). *Depends:* Vol 13.

- **EKOS-02-056 (Could, P3)** — High-value widgets MAY protect their visual-mapping/AI
  logic via a **secure execution environment / source escrow** (runs without exposing
  source). *Source:* research(monetization §3 — IP protection & source escrow);
  Vol 13.

---

## 13. Widget Composition & Reuse

- **EKOS-02-057 (Should, P2)** — Widgets SHALL be **composable into screens/dashboards**
  (multiple widget instances arranged on one surface). *Note:* org-defined
  ScreenBlueprints and personalization are Vol 08; here, composition is the engine
  primitive. *Source:* ConfusionClarity(Screen Blueprints — "layout grid composed of
  widget instances"); research.

- **EKOS-02-058 (Should, P2)** — A widget SHALL be able to **react to selection in
  another widget** on the same screen (cross-widget interactivity), since all read from
  the same graph. *Source:* research(glue §); ConfusionClarity.

- **EKOS-02-059 (Could, P3)** — Knowledge bundles SHALL be **composable** (Architecture
  Bundle + Security Bundle + Compliance Bundle = Executive Bundle); the *packaging* is
  Vol 06, but the engine SHALL support assembling a view from multiple widget sets.
  *Source:* ChatGPT(bundle composition). *Depends:* Vol 06.

---

## 14. Relationship to the Existing ESIP Designer

- **EKOS-02-060 (Must, MVP)** — The existing ESIP React designer (Tiptap-based, with
  widgets: callout, requirement, ADR, metrics, process-steps, status-badge, divider,
  Mermaid diagram, maxGraph flow diagram, BPMN, matrix/RACI/CRUD/SIPOC/BMC, trade-off,
  NFR) SHALL be recognized as an **early partial implementation of this Widget Engine**
  — its widgets are precursors to graph-backed `WidgetDefinition`s. *Source:*
  [[project-esip]]; [[esip-resume-state]]; MASTER-INDEX §0.

- **EKOS-02-061 (Should, P2)** — Migration path: existing ESIP widgets SHALL be
  **re-expressed as graph-backed projections** (persisted query + visual mapping)
  rather than document-embedded Tiptap nodes, when EKOS's graph backbone lands —
  preserving authored content. *Source:* derived; H3 convergence note in [[project-esip]].

- **EKOS-02-062 (Should, P2)** — The ESIP designer's current document-centric storage
  (Firebase + embedded JSON) SHALL be treated as the legacy seam to evolve **behind the
  Vol 01 graph DAL**, not a rewrite. *Source:* [[project-esip]] (ports-and-adapters,
  evolve behind seams).

---

## 15. Constitution Conformance (engine-specific)

- **EKOS-02-063 (Must, MVP)** — Widgets are **projections of the graph** — no widget
  may hold authoritative state outside the graph (Constitution §4; Vol 01 EKOS-01-079).
  *Source:* Constitution §4.

- **EKOS-02-064 (Must, MVP)** — The **widget engine is decoupled from the graph**
  (META-004): new visual paradigms can be introduced as new widgets working on
  existing data with **no data migration**. *Source:* research(future-proofing §3);
  Vol 01 EKOS-01-057.

- **EKOS-02-065 (Should, P2)** — Rendering-engine and host-framework names are
  illustrative & pluggable (Constitution §6); requirements bind to capabilities
  (60 fps, GPU canvas), not vendors. *Source:* Constitution §6.

---

## 16. Open Questions (parked)

- **OQ-02-A** — Host shell + native engine combo (Web WebGPU vs. Flutter+plugin vs.
  Tauri+Rust) — decide in Vol 15; must satisfy EKOS-02-008/009 and viewer reuse
  (EKOS-02-012).
- **OQ-02-B** — Sandbox model for custom JS/Python rendering & published widgets
  (WASM sandbox vs. server-side execution) — security ruling needed with Vol 04/13.
- **OQ-02-C** — Process Capture AI capture mechanism on locked-down enterprise
  desktops (OS permissions, privacy) — pilot needed; interplays with Vol 04.
- **OQ-02-D** — How much of the ESIP Tiptap widget set survives vs. is rebuilt graph-
  native — sequencing decision for the build phase (H3 convergence).
- **OQ-02-E** — Default fixed-layout page metrics & multi-page Living Canvas (A4/Letter,
  pagination of huge graphs) — UX decision for Vol 07 viewer alignment.

---

## 17. Requirement Index (this volume)

EKOS-02-001 … EKOS-02-065 (stable numbering; the four-part anatomy, three OS layers,
Process Capture pipeline, and resume-projection ruling expand to 88 distinct stated
rules counting multi-part catalogs). **Key flag for later:** EKOS-02-050/051 — the
**Personal Knowledge Space / resume-as-projection** ruling MUST be fully owned and
expanded in **Vol 10**.

> **Next volume:** `03 — Hybrid Ambient AI Engine (4 layers, analysts, explainability)`.
> Per protocol: STOP here. Await user "proceed".
