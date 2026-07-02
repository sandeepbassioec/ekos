# Phase 1 — Blended MVP — Ticket Breakdown

**Date:** 2026-06-23 · **Companion to:** `BUILD-PLAN-MVP-AND-PHASING.md`, `TECH-STACK.md`, `00-MASTER-INDEX.md`

**Scope (chosen):** the **Blended MVP** — architect value core (personas + 3 pillars + cross-role workflow + Solution Architect persona) **+** day-1 personalization (working-mode wizard, adaptive home) **+** minimal tenant/admin/license scaffolding. **On today's stack** (React + Vite + Firebase + `ai-proxy`), **built against the ports** so the .NET/DB/cloud/maxGraph swaps land cleanly later, and **UI-consistent** (everything from `src/app/components/ui/`).

**Explicitly OUT of Phase 1** (later phases): full licensing enforcement/billing, deep adaptive learning (pattern mining/automation), EA frameworks (TOGAF/Zachman/DoDAF), code-scanning, enterprise connectors/SSO/multi-tenancy at scale, the **maxGraph** interactive canvas (MVP keeps **Mermaid + AI-generated** diagrams), and the .NET/Rust backend (MVP stays on Firebase behind the DAL).

**Conventions:** ticket IDs `P1-<Epic><n>`. Size = S (≤1d) / M (2–4d) / L (1–2wk). "Reuse" lists existing files to extend; "New" lists net-new. Every ticket's DoD includes a **UI-consistency check** (uses `components/ui/`, Tailwind tokens) and **no regression** to existing flows (feature-flagged).

---

## EPIC A — App Shell & Navigation *(UI-consistency foundation; do first)*

| ID | Ticket | Reuse & New (files) | Size |
|----|--------|---------------------|------|
| P1-A1 | Shared `AppShell` (header + collapsible sidebar + content) wrapping all authenticated routes; remove per-page inline back-buttons | Reuse `components/ui/sidebar.tsx`, `navigation-menu.tsx`, `breadcrumb.tsx`; edit `app/routes.tsx`, `App.tsx`. New `components/layout/AppShell.tsx` | M |
| P1-A2 | Persona switcher in header (active persona, persists) | Reuse `dropdown-menu.tsx`; depends P1-B3 | S |
| P1-A3 | Workspace nav driven by active persona + working mode (which pillars/tools show) | New `components/layout/WorkspaceNav.tsx`; depends P1-B1, P1-F3 | M |
| P1-A4 | Global inbox badge (pending workflow count) in header | Reuse `badge.tsx`; depends P1-D6 | S |
| P1-A5 | User menu (profile, preferences, logout) | Reuse `dropdown-menu.tsx`, `authStore`; links to P1-F5 | S |
| P1-A6 | `UI-CONVENTIONS.md` + tokens audit; lint rule "compose from `ui/` only" | New doc; `styles/` review | S |

## EPIC B — Persona Model *(metamodel-driven; core)*

| ID | Ticket | Reuse & New | Size |
|----|--------|-------------|------|
| P1-B1 | Add `Persona` type (id, label, icon, enabledPillars, artifactTypes, diagramTypes, deliverableTemplates, defaultWorkingMode, promptPackId) | Edit `types/metamodel.ts` | S |
| P1-B2 | Seed personas: **Application Architect** (= today) + **Solution Architect** | Edit `lib/metamodel-store.ts` (`initializeMetaModel`) | M |
| P1-B3 | `personaStore` (activePersona, list, switch) | New `store/personaStore.ts`; reads metamodel | M |
| P1-B4 | Persona admin in `AdminMetaModelPage` (view/edit personas — config not code) | Extend `pages/AdminMetaModelPage.tsx`, `CreateArtifactTypePage.tsx` | M |
| P1-B5 | Persist user's assigned personas + last-active persona | Extend `lib/db` (users profile via `firebase-impl` `users/{uid}`) | S |

## EPIC C — Persona-aware Pillars *(reuse the 3 pillars, scope by persona)*

| ID | Ticket | Reuse & New | Size |
|----|--------|-------------|------|
| P1-C1 | Requirements pillar reads active persona (labels/scope/required types per persona) | Reuse `requirementsStore`, `RequirementsCaptureBoard`, `ComprehensiveRequirementsWizard` | M |
| P1-C2 | Knowledge Hub tab set configurable per persona | Reuse `KnowledgeHubPage.tsx`, `knowledgeHubStore` | M |
| P1-C3 | Roadmap types/views per persona | Reuse `RoadmapPage.tsx`, `roadmapStore`, `types/roadmap.ts` | S |
| P1-C4 | Pillar/tab visibility from persona config (single source) | Driven by P1-B1; wire into A3 | S |

## EPIC D — Cross-role Workflow v1 *(Vol 12 slice; reuse Confusions/Gaps/Approval)*

| ID | Ticket | Reuse & New | Size |
|----|--------|-------------|------|
| P1-D1 | Workflow **port** + `Request` types (clarification, missing-req, approval, hand-off): interface + Firestore adapter | New `lib/db/interface.ts` additions + `firebase-impl.ts`; new `types/workflow.ts` | M |
| P1-D2 | `workflowStore` (raise / route / respond / approve / reject / hand-off / withdraw) | New `store/workflowStore.ts` | M |
| P1-D3 | Convert Knowledge-Hub **Confusion/Gap → routed clarification/missing-req request** (one click) | Reuse `knowledgeHubStore` (confusions/gaps), `ConfusionsTab`, `GapsTab` | M |
| P1-D4 | **Approval gate before publish** (deliverable/roadmap/diagram) — block until approved | Reuse `ManagerApproval.tsx` patterns; gate in `DeliverablesTab`/publish actions | M |
| P1-D5 | Hand-off request (EA→SA→role) with context + acceptance | New in workflowStore + UI dialog | S |
| P1-D6 | **Work Inbox** page (tabs: Assigned / Awaiting me / Raised by me) with deep-links | Reuse `tabs.tsx`, `table.tsx`, `badge.tsx`; new `pages/InboxPage.tsx` + route | M |
| P1-D7 | Notifications: in-app `sonner` + email on assign/decision | Reuse `sonner`; email via `ai-proxy`/server endpoint or Firebase ext | M |
| P1-D8 | Routing v1 (to role or named user) + permission check | In workflowStore; reuse role from `authStore`/`users` | S |

## EPIC E — Solution Architect Persona *(reuse estimation + arch generators; Mermaid diagrams)*

| ID | Ticket | Reuse & New | Size |
|----|--------|-------------|------|
| P1-E1 | SA workspace (persona config + nav + pillars) | Persona from P1-B2; reuses pillars | S |
| P1-E2 | Options & trade-off capture (weighted scoring matrix) | Reuse `form.tsx`/`table.tsx`/`card.tsx`; new `components/solution/OptionsMatrix.tsx` | M |
| P1-E3 | ADR capture + decision log (status/context/decision/consequences/supersedes) | New `store/adrStore.ts` via DAL; new `pages/ADRsPage.tsx` | M |
| P1-E4 | Wire HLD/LLD generation into SA persona (keep Mermaid for MVP) | Reuse `components/architecture/{HLD,LLD}Generator.tsx`, `MermaidDiagram.tsx` | M |
| P1-E5 | Surface sizing/estimation in SA | Reuse `estimationStore`, `complexityStore`, `EstimationWizard` | S |
| P1-E6 | SA deliverable generation (solution overview, options paper, HLD) | Reuse `utils/dynamic{Excel,Pptx}Generator.ts`, `jspdf` | M |

## EPIC F — Day-1 Personalization *(Vol 14 part 1)*

| ID | Ticket | Reuse & New | Size |
|----|--------|-------------|------|
| P1-F1 | Preferences **port** + `preferencesStore` (dashboard layout, theme/density, working mode) | New `store/preferencesStore.ts` via DAL | M |
| P1-F2 | First-run **working-mode/preference wizard** (on activation/first login): dashboard + mode (design / estimation / requirements-first / custom) | Reuse `StepIndicator.tsx`, `dialog.tsx`, `form.tsx`; new `pages/OnboardingWizard.tsx` | M |
| P1-F3 | **Working-mode engine** — mode reshapes nav defaults + foregrounded pillars/steps | Drives P1-A3; reads preferencesStore | M |
| P1-F4 | Adaptive **home** (recent items, persona/role widgets, "resume where you left off") | Reuse `card.tsx`, `recharts`; new `pages/HomePage.tsx` (replace stub) | M |
| P1-F5 | Preferences settings page (change anytime) | Reuse `form.tsx`, `tabs.tsx`; new `pages/PreferencesPage.tsx` | S |

## EPIC G — Minimal Tenant/Admin/License Scaffolding *(Vol 13 slice; single-tenant MVP)*

| ID | Ticket | Reuse & New | Size |
|----|--------|-------------|------|
| P1-G1 | License-type model per persona + per-org quantity (single-tenant assumption) | New `types/license.ts`; config in metamodel/DAL | S |
| P1-G2 | Admin user-management (create user, assign persona license) | Extend `pages/TeamManagement.tsx`; reuse `table`/`dialog`/`form` | M |
| P1-G3 | **Entitlement gate** — user must hold the persona license to use it | Extend `components/ProtectedRoute.tsx` + persona gate | M |
| P1-G4 | Invite + activation (admin-created user → email w/ license type + activation link → first login → P1-F2) | Reuse `Register.tsx`, `authStore`, `firebase-impl` auth; email via D7 channel | M |
| P1-G5 | Config-zone stub page (DB/AI-keys/Git targets placeholders; wire **per-org BYOK AI key** into `ai-proxy` only) | New `pages/admin/ConfigZonePage.tsx`; light `ai-proxy` per-tenant key | M |

## EPIC H — Ports & Foundations Hygiene *(de-risk the later migration)*

| ID | Ticket | Reuse & New | Size |
|----|--------|-------------|------|
| P1-H1 | Make the client-side **ports** explicit: DAL (exists) + workflow + preferences + persona/entitlement + notifications | `lib/db/interface.ts` + new port interfaces in `lib/` | M |
| P1-H2 | Generalize `ai-proxy` with a `provider` field (Bedrock default) — prep pluggable AI | Edit `ai-proxy/index.js`, `lib/ai/bedrock.ts` | S |
| P1-H3 | Converge requirement model on DAL `requirements` + KH `formalRequirements`; flag localStorage capture paths for migration | `requirementsStore`, `RequirementsCaptureBoard`, `CreateRequirement` | M |
| P1-H4 | Feature flags for persona/workflow/personalization (safe rollout, no regression) | New `lib/featureFlags.ts` | S |
| P1-H5 | Back-compat: existing projects/roadmaps/KH default to **Application Architect** persona | Migration shim in stores/DAL | S |

---

## Suggested build order
1. **A (shell) + H1/H4/H5 (ports, flags, back-compat)** — the scaffold everything hangs on; zero regression.
2. **B (persona model)** — unlocks persona-aware nav/pillars.
3. **F (personalization v1)** — working-mode wizard + adaptive home (depends B).
4. **C (persona-aware pillars)** — light, high-visibility.
5. **D (cross-role workflow v1)** — the differentiator; reuses Confusions/Gaps/Approval.
6. **E (Solution Architect persona)** — proves "second persona" on the model.
7. **G (admin/license scaffolding)** — wraps it for multi-user onboarding.

Rough size: ~38 tickets; ≈ a focused MVP. A and B are the critical path; D and F are the headline differentiators.

## Definition of Done (every ticket)
- Built only from `components/ui/` + Tailwind tokens (UI-consistency check).
- Talks to a **port/store**, never a backend directly (keeps the .NET/DB swap clean).
- Feature-flagged; existing flows unaffected.
- Verified in the running app (Vite dev server) with no new console errors.

---

*Next: implement in build order (start Epic A + H foundations), or adjust scope first. maxGraph and the .NET/DB migration begin in later phases per `TECH-STACK.md`.*
