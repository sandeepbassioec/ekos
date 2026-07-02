# ESIP Architecture Suite — MVP & Phased Build Plan

**Date:** 2026-06-23 · **Companion to:** `00-MASTER-INDEX.md` (the ~2,794-requirement catalog).
**Purpose:** turn the catalog into a buildable sequence **on top of today's ESIP code**, not a greenfield rewrite.

## Governing principles (non-negotiable)
1. **Reuse first.** Every phase extends existing assets (stores, DAL, Knowledge Hub, roadmaps, metamodel, AI proxy, generators). Changes are expected; rip-and-replace is not.
2. **UI consistency.** All new screens compose from the existing design system — Radix primitives in `src/app/components/ui/`, Tailwind v4 tokens (`src/styles/`), `lucide-react` icons, `sonner` toasts, and the current page/layout patterns. No new visual language.
3. **Workflow-native.** Cross-role collaboration (Vol 12) is wired in from the MVP — because a multi-persona product inherently needs clarification/approval/hand-off between roles.
4. **Metamodel-driven.** Personas, artifacts, diagrams, deliverables, prompts, and workflows are *configuration* (Vol 11), so new roles are added without code.

---

## 1. What we already have (reuse inventory)

| Existing asset (in repo) | Reuse as → | Catalog vol |
|--------------------------|-----------|-------------|
| React 18 + TS + Vite + **Tailwind v4** + **Radix `components/ui/`** (~50) | The design system for the whole suite | (UI) |
| **Zustand stores** (auth, projects, roadmap, knowledgeHub, requirements, complexity, estimation) | Per-domain state; extend, don't replace | all |
| **Firebase DAL** (`lib/db` interface + firebase-impl; now incl. pis/sprints/requirements) + **Cloudinary** + **ai-proxy** | The swappable persistence + AI backbone | 06, 07, 11 |
| **Auth + roles** (Firebase, persisted `users/{uid}` role, ProtectedRoute gates) | Identity for workflow participants & approvals | 11, 12 |
| **Knowledge Hub** (per-roadmap-item; 14 tabs; Bedrock `analyzeAllSources`; AI search/commands) | The Knowledge Hub pillar — generalize per persona | 06, 08 |
| **Confusions / Gaps / Clarifications / ManagerApproval** | **Seeds of the cross-role workflow engine** | 12 |
| **Requirements** (CaptureBoard, CreateRequirement, ComprehensiveWizard, formalRequirements, `documentParser`, BRDImportModal) | Requirements pillar — multi-format ingestion already partly works | 07 |
| **Roadmaps** (3 types, year selector, Quarterly/Monthly/Gantt, react-dnd available) | Roadmap pillar — extend to transition states/portfolio | 09 |
| **Architecture generators** (BFA/DFA/HLD/LLD, **MermaidDiagram**, ArchitectureWizard) | Diagramming/deliverables seed — upgrade Mermaid→interactive later | 03, 10 |
| **Metamodel** (AdminMetaModelPage, `metamodel-store`, ArtifactType/FieldDefinition/templates, industry-templates, questionnaires) | **The persona/artifact/extensibility engine** — central to the suite | 11, 05 |
| **Deliverable generators** (SheetJS xlsx, pptxgenjs, jsPDF, requirements Excel/PPT) | Deliverables factory seed | 10 |
| **Estimation wizard** + complexity/WBS (Groq via proxy) | Solution-Architect sizing/estimation | 04 |

**Read:** the current-state truth is in `../../ESIP_CODEBASE_CONTEXT.md`. ~70% of the *foundations* the suite needs already exist in some form — the work is generalizing them to multi-persona and adding the missing engines (interactive modeling, code-scan, connectors, workflow, governance-at-scale).

---

## 2. North-star shape (how today's pieces generalize)

```
            ┌──────────────── Persona layer (metamodel-driven) ────────────────┐
            │  EA · SA · AppArch · Integration/API/CRM/Data/Sec/Cloud/… · custom │
            └───────────────────────────────┬───────────────────────────────────┘
   3 pillars (persona-aware):   Requirements ── Roadmaps ── Knowledge Hub
            │                       │             │              │
            ▼                       ▼             ▼              ▼
   Cross-role Workflow engine (clarify · approve · hand-off)  ◄─ ties roles together
            │
   AI engine (Bedrock via ai-proxy; per-persona prompt packs; extract → generate → analyze)
            │
   Modeling/Diagramming  +  Deliverables factory  (Mermaid today → ArchiMate canvas + doc gen)
            │
   Persistence: Firebase DAL (swappable) · Cloudinary · metamodel config
```
The existing app is effectively the **Application-Architect persona** with these pillars. The suite = make the persona layer explicit + add the workflow, modeling, code-scan, connector, and governance engines around the same spine.

---

## 3. MVP — "Multi-persona suite, proven on the existing base"

**Goal:** one coherent slice that demonstrates the vision end-to-end for **two personas** (Application Architect = today; **Solution Architect** = nearest new value), with **cross-role workflow** and **per-persona AI**, all on the current stack.

**MVP Must-haves (MoSCoW = Must):**
- **M1 — Persona model & switcher** (metamodel-driven). Reuse `metamodel-store` + `AdminMetaModelPage`; add a `persona` concept (artifacts/tabs/deliverables/prompts per persona). Workspace adapts to the selected persona. *(Vols 05, 11)*
- **M2 — Persona-aware pillars.** Reuse Requirements + Knowledge Hub + Roadmap; scope/label them per persona. No schema rewrite — add `personaId` context. *(07, 08, 09)*
- **M3 — Cross-role workflow v1.** Reuse Confusions/Gaps + roles + ManagerApproval + sonner. Deliver: **Clarification request**, **Missing-requirement request**, **Approval-before-rollout gate**, **hand-off**; a **work inbox** (tabs: assigned / awaiting-me / raised-by-me); in-app + email notify. *(Vol 12: WFL-2/3/4/5/6/9)*
- **M4 — AI per-persona.** Reuse `aiPrompts` + Bedrock proxy; add per-persona prompt packs; keep extraction (already working) + add "generate first-cut deliverable" for SA (HLD/options). *(06)*
- **M5 — Solution Architect persona.** Reuse estimation/complexity + BFA/DFA/HLD/LLD generators + MermaidDiagram; add Options/Trade-off + ADR capture. *(04)*
- **M6 — Deliverable generation per persona.** Reuse SheetJS/pptxgenjs/jsPDF generators behind a persona-template selector. *(10)*
- **M7 — UI consistency pass + shared app shell.** Introduce one consistent nav/layout shell (persona switcher, workspace nav, inbox badge) reusing `sidebar`/`navigation-menu`/`breadcrumb` from `components/ui/`; retrofit existing pages to it. *(UI)*

**Should (MVP+):** AI "suggest target role" for clarifications (WFL-5); approval e-sign stub; basic SLA/reminders (WFL-8).
**Won't (defer):** full ArchiMate canvas, code-scanning, enterprise connectors, SSO/multi-tenancy, EA frameworks.

**MVP proves:** pick a persona → capture requirements (ingest docs, AI-extract) → AI generates a first-cut deliverable/diagram → route a clarification + an approval to another role → publish after sign-off. That is the whole product in miniature, on reused code.

---

## 4. Phased roadmap

| Phase | Theme | Headline scope | Primary reuse | Vols |
|-------|-------|----------------|---------------|------|
| **0 — Foundations** *(largely done / in-flight)* | Stabilize spine | Doc-text extraction✓, requirements via DAL✓, AI proxy✓, role persistence✓; **design-system tokens audit**; metamodel→persona groundwork | existing fixes | 06,07,11 |
| **1 — MVP (above)** | Multi-persona core + workflow | M1–M7 | metamodel, pillars, KH, generators, roles | 04,05,06,07,08,09,10,11,12 |
| **2 — Enterprise Architect & flagship analysis** | EA journey + code→portfolio→sunset | TOGAF ADM workspace; capability/value-stream maps; **APM portfolio**; **code-scanning → app/data/business-flow derivation → sunset/migration (6 R's/TIME)**; roadmap transition states/plateaus | roadmap, metamodel, AI, diagram seeds | 01,02,09 |
| **3 — Modeling & deliverables at depth** | Real modeling + doc factory | **Interactive ArchiMate 3.2 canvas** (upgrade from Mermaid) + UML/BPMN/C4/ERD; model↔view; validation; **per-framework deliverable factory + publishing portal** | MermaidDiagram, generators | 03,10 |
| **4 — Enterprise platform** | Scale & governance | SSO/SCIM, **RBAC/ABAC**, multi-tenancy, audit, **enterprise connectors** (Jira/Confluence/SharePoint/ADO/ServiceNow), no-code workflow builder, metamodel marketplace | auth, DAL, KH connectors, Vol 12 | 08,11,12 |
| **5 — Role packs & "any architect"** | Breadth | Integration/API/CRM/Data/Security/Cloud/Infra/µservices packs as **metamodel config**; agentic AI copilots per persona; GA extensibility | metamodel, AI | 05,06 |

**Sequencing logic:** Phase 1 leverages the most existing code for the fastest credible "suite" demo; Phase 2 delivers your flagship EA/code-scan/sunset differentiator; Phase 3 makes diagrams/deliverables enterprise-grade; Phases 4–5 scale and broaden. Cross-role workflow (Vol 12) and the metamodel (Vol 11) thread through every phase.

---

## 5. UI consistency mandate (how, concretely)
- **Single source of components:** build only from `src/app/components/ui/*` (Radix wrappers). New primitives get added there, not inline.
- **Tokens, not hardcoded styles:** colors/spacing/typography from Tailwind v4 theme in `src/styles/`. Reuse the existing palette (the brand blue/indigo/etc.).
- **One app shell:** a shared layout (header with persona switcher + workspace nav + inbox badge + user menu) applied to all authenticated routes — replacing today's per-page inline back-buttons. Use `sidebar`/`navigation-menu`/`breadcrumb`.
- **Patterns:** lists as `table`/`card`; create/edit in `dialog`/`drawer`+`form`; status via `badge`; feedback via `sonner`; charts via `recharts`. New features must match these.
- **Definition of done includes** a consistency check against this list.

---

## 6. Risks & notes
- **Requirement-model sprawl** (catalog has overlapping requirement notions across V05/07/12 + today's 4 capture paths) — Phase 1 should converge on one canonical requirement model (the DAL `requirements` + Knowledge-Hub `formalRequirements`), retiring the localStorage paths.
- **Catalog is V1, un-deduplicated** — a cross-volume reconciliation pass is recommended before committing Phase 2+ scope (tracked as follow-up in the index §9).
- **Modeling engine is the biggest net-new build** (Phase 3) — keep Mermaid + AI-generation as the bridge until the interactive canvas lands.
- **Don't regress today's working flows** while generalizing to personas — wrap, feature-flag, and migrate.

---

*Next: on approval, Phase 1 can be broken into concrete tickets mapped to specific files/components, or we can run a cross-volume dedup/reconciliation pass first.*
