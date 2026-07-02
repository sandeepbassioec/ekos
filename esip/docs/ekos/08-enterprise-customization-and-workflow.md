# EKOS Volume 08 — Enterprise Customization & Workflow Orchestration

> **Status:** ✅ Done · **Requirement count:** 80 · **Generated:** 2026-06-30 (Opus)
> **Sources mined:** `ConfusionClarity.txt` (customization deep-dive — Salesforce/
> ServiceNow/Jira/dbt lessons, Org-Wide Experience Config, End-User Personalization,
> Visual Workflow Engine, Layout/Theme; final-spec §8), `blueprint §14` (Guaranteed
> Comment Resolution / ClarificationRequest), `ChatGPT` (Clarification as a
> constitutional Knowledge Object), `research_across_llms.txt` / `Gemini critique`
> (CRDT real-time collaboration + Visual Spatial Optimistic Locking / ghost-lock),
> `[[project-esip]]` Epics B (persona), D (cross-role workflow inbox), F (working-mode/
> adaptive home) as early implementations. Honors [00-MASTER-INDEX](00-MASTER-INDEX.md)
> §2.

---

## 1. Scope & Boundary

**In scope:** org-wide **experience configuration** (Screen Blueprints, placement
rules, conditional visibility); **end-user personalization** (My Workspace);
**visual workflow designer** (states/transitions/triggers/conditions/parallel
approvals, graph-stored & versioned); org-level **triggers & automations**;
**layout/theme/branding**; the **cross-role workflow engine** (WorkflowRequest /
inbox); **Clarification as a constitutional Knowledge Object**; **real-time
collaboration** (CRDT sync, presence, spatial conflict resolution / ghost-lock);
**working modes & adaptive home**; and sandbox/staging of customization changes.

**Explicitly deferred (cross-referenced, not duplicated):**
- The node TYPES (`ExperienceDefinition`, `ScreenBlueprint`, `UserWorkspace`,
  `BPMNProcess/Task/Gateway`, `ApprovalGate`, `ChangeRequest`, workflow versioning,
  CRDT-able mutation model) → **Vol 01**
- Widget composition into screens, per-widget interactivity/automation surface, the
  rendering engine → **Vol 02** (this volume *arranges & governs* widgets)
- **ABAC conditional-visibility evaluation & masking** → **Vol 04** (this volume
  *uses* it to drive layout)
- AI-suggested workflow improvements, bottleneck detection → **Vol 03**
- **Sandbox/staging & governed promotion pipeline, AI-output review gate, feature
  flags** → **Vol 05** (this volume *consumes* the promotion pipeline)
- **P2P live sessions, WebRTC, mDNS discovery, BYOI live sync, live-session-anchored
  clarification calls** → **Vol 12** (this volume owns the *workflow object*; Vol 12
  owns the *live connection*)

---

## 2. Core Principle — Customization Is Non-Negotiable for Enterprise Adoption

- **EKOS-08-001 (Must, MVP)** — EKOS SHALL provide deep org-wide and user-level
  customization; a rigid, fixed application **will be rejected by enterprises**. An OS
  must provide the kernel and let organizations build their computing environment on
  top. *Source:* ConfusionClarity(customization — "mandatory for any platform claiming
  to be an operating system").

- **EKOS-08-002 (Must, MVP)** — The customization model SHALL deliver the **"My
  Salesforce" factor** (one instance behaves completely differently per business unit
  while sharing one data model), **ServiceNow-class configurable workflows**, and
  **Jira-style role-driven visibility**. *Source:* ConfusionClarity(customization —
  Salesforce/ServiceNow/Jira analysis).

- **EKOS-08-003 (Should, P2)** — Deep customization SHALL create a **lock-in moat**:
  once an org tailors its screens, workflows, and analysts, switching tools means
  rebuilding institutional logic. (Balanced by the no-lock-in *data* export of Vol 01.)
  *Source:* ConfusionClarity(customization — lock-in moat); Vol 01 §14.

- **EKOS-08-004 (Must, MVP)** — Customization SHALL build on the existing foundation —
  **semantic graph + Widget OS + Policy-as-Code** — not a new subsystem. *Source:*
  ConfusionClarity(customization — "we already have the foundation").

---

## 3. Org-Wide Experience Configuration (Admin Layer)

- **EKOS-08-005 (Must, P2)** — An admin SHALL define an **`ExperienceDefinition` /
  `ScreenBlueprint`** (a graph node, Vol 01) composing a **layout grid of widget
  instances** with layout metadata (e.g., 2-column, dashboard, full-screen). *Source:*
  ConfusionClarity(customization A — Screen Templates; final-spec §8).

- **EKOS-08-006 (Must, P2)** — Blueprints SHALL support **Widget Placement Rules** —
  e.g., "on the Project Overview screen, show the Story Map Canvas top and Risk Radar
  in the sidebar, **but only if project phase = Execution**". *Source:* ConfusionClarity
  (customization A — Widget Placement Rules).

- **EKOS-08-007 (Must, P2)** — Blueprints SHALL support **Conditional Visibility** driven
  by the ABAC engine — e.g., "show the Cost-to-Feature Map only to `FinOps` or
  `ProjectManager`". *Source:* ConfusionClarity(customization A — Conditional
  Visibility); Vol 04. *Depends:* Vol 04.

- **EKOS-08-008 (Must, P2)** — The rendering engine SHALL **assemble each user's screen
  at runtime** from the applicable blueprint based on their identity/role/phase.
  *Source:* ConfusionClarity(customization A — "interprets at runtime"). *Depends:*
  Vol 02.

- **EKOS-08-009 (Should, P2)** — `ScreenBlueprint`s SHALL be **versioned & traceable**
  (Vol 01) so the experience configuration's evolution is auditable. *Source:* derived
  from ConfusionClarity + Vol 01 §7.

---

## 4. End-User Personalization (My Workspace)

- **EKOS-08-010 (Must, P2)** — Each user SHALL customize their **own view without
  affecting the org blueprint**: add/remove widgets, **resize, reorder, minimize**.
  *Source:* ConfusionClarity(customization B — My Workspace).

- **EKOS-08-011 (Should, P2)** — Users SHALL **save multiple workspace layouts** (e.g.,
  "Morning Standup" vs "Deep Work"). *Source:* ConfusionClarity(customization B).

- **EKOS-08-012 (Should, P2)** — Users SHALL choose **personal AI Analyst subscriptions**
  ("always surface the Security Auditor; hide the Cost Trade-off Advisor"). *Source:*
  ConfusionClarity(customization B); Vol 03.

- **EKOS-08-013 (Must, P2)** — Personalization SHALL be stored as a **`UserWorkspace`
  node** linked to the user identity; the engine SHALL **merge the org blueprint with
  the user delta at load**, resolving conflicts gracefully. *Source:* ConfusionClarity
  (customization B — implementation).

- **EKOS-08-014 (Must, P2)** — If a user hides a widget the admin later marks
  **mandatory**, the system SHALL show it with a **lock icon + tooltip "Required by
  admin"** rather than honoring the hide. *Source:* ConfusionClarity(customization B —
  conflict resolution).

---

## 5. Visual Workflow Designer

- **EKOS-08-015 (Must, P2)** — A **visual, BPMN-like workflow designer** SHALL let
  non-technical admins model artifact lifecycles directly in the browser. *Source:*
  ConfusionClarity(customization C; final-spec §8 — Visual Workflow Designer).

- **EKOS-08-016 (Must, P2)** — A workflow SHALL define **states and transitions** for
  any artifact type (e.g., a `Requirement`: Draft → Reviewed → Approved → Implemented).
  *Source:* ConfusionClarity(customization C).

- **EKOS-08-017 (Must, P2)** — On entering a state, a workflow SHALL fire **triggers**:
  call a webhook, send an email/notification, create a task, or **execute a custom
  script**. *Source:* ConfusionClarity(customization C — triggers).

- **EKOS-08-018 (Must, P2)** — Transitions SHALL support **conditions** — e.g., "allow
  transition to Approved only if linked `TestCase` coverage > 80%" (compliance gates,
  Vol 05). *Source:* ConfusionClarity(customization C — conditions); Vol 05 EKOS-05-014.

- **EKOS-08-019 (Must, P2)** — Workflows SHALL support **parallel approvals** — multi-
  person sign-off modeled as nodes in the workflow graph. *Source:* ConfusionClarity
  (customization C — parallel approvals).

- **EKOS-08-020 (Must, P2)** — Workflow definitions SHALL be **stored as graph nodes**
  (not an external engine) with **full versioning**, so it's traceable exactly which
  workflow version was used when a requirement was approved. *Source:* ConfusionClarity
  (customization C — "workflow becomes part of the model ... full traceability");
  final-spec §8.

- **EKOS-08-021 (Should, P2)** — The AI SHALL **suggest workflow improvements** from
  bottleneck analysis (Vol 03 EKOS-03-073). *Source:* ConfusionClarity(customization C —
  AI integration). *Depends:* Vol 03.

---

## 6. Org-Level Triggers & Automations

- **EKOS-08-022 (Must, P2)** — Admins SHALL define **rule-based automations** on state
  changes — e.g., "when a `Requirement` moves to Approved, create an architecture-review
  task and notify the Security Architect". *Source:* ConfusionClarity(customization
  intro — dbt/Airbyte lesson; final-spec §8 — Triggers & Automations).

- **EKOS-08-023 (Should, P2)** — Org-level automations SHALL compose with the per-widget
  automation surface of Vol 02 (no duplication — widget-scoped vs org-scoped).
  *Source:* synthesis; Vol 02 EKOS-02-033.

- **EKOS-08-024 (Could, P3)** — Custom code injection into triggers SHALL run sandboxed
  (same sandbox model as custom widgets, Vol 02/Vol 05). *Source:* ConfusionClarity
  (customization C — "inject custom code if needed"); Vol 02.

---

## 7. Layout, Theme & Branding

- **EKOS-08-025 (Should, P2)** — Orgs SHALL apply **branding**: upload logos, choose
  color schemes. *Source:* ConfusionClarity(customization D — Branding); final-spec §8.

- **EKOS-08-026 (Should, P2)** — **Conditional styling** SHALL be supported (e.g., "high
  risk → red pulsating border") — the visual-behavior layer of Vol 02. *Source:*
  ConfusionClarity(customization D); Vol 02 EKOS-02-029.

- **EKOS-08-027 (Could, P3)** — Advanced enterprises SHALL apply **custom CSS/JSON
  themes**; full white-label is Vol 09/Vol 26-equivalent. *Source:* ConfusionClarity
  (customization D — custom CSS/JSON). *Depends:* Vol 09.

---

## 8. Cross-Role Workflow Engine (Inbox)

> ESIP Epic D is the early implementation; generalize it here.

- **EKOS-08-028 (Must, MVP)** — A **cross-role workflow engine** SHALL let any user
  **raise a request** to a role or named user and the recipient **respond / approve /
  reject / resolve / withdraw**; an **Inbox** SHALL aggregate "assigned to me",
  "awaiting my approval", and "raised by me". *Source:* [[project-esip]] Epic D
  (workflowRequests, raise/respond/decide/resolve/withdraw, InboxPage).

- **EKOS-08-029 (Must, P2)** — Workflow requests SHALL route by **role and named user**
  (and, generalized, by **persona scope**) with a live count badge surfaced in the
  shell. *Source:* [[project-esip]] Epic D (v1 routes by auth role; persona-scoped
  routing was deferred — generalize here).

- **EKOS-08-030 (Should, P2)** — Workflow requests SHALL be first-class **graph nodes**
  (single-field-queryable, no composite-index dependency in the early Firebase impl),
  evolving behind the Vol 01 DAL. *Source:* [[project-esip]] Epic D; Vol 01.

---

## 9. Clarification as a Constitutional Knowledge Object

> ChatGPT's ruling: Clarification should be a constitutional object that works
> everywhere — requirements, résumé, architecture, law, research.

- **EKOS-08-031 (Must, P2)** — **Clarification SHALL be a constitutional Knowledge
  Object** with a uniform lifecycle: **Knowledge Object → ClarificationRequest →
  Evidence → Review → Approval → Mutation** — usable against *any* node type, not just
  requirements. *Source:* ChatGPT(Clarification as constitutional object).

- **EKOS-08-032 (Must, P2)** — A `ClarificationRequest` SHALL carry an **SLA, escalation
  path, and AI-drafted suggestion**, guaranteeing resolution (no silently-dropped
  comments). *Source:* blueprint §14 (Guaranteed Comment Resolution).

- **EKOS-08-033 (Should, P2)** — Clarifications raised in a viewer SHALL queue **offline**
  and sync when online. *Source:* blueprint §14 (offline queue in viewer); Vol 07 §22.

- **EKOS-08-034 (Should, P2)** — Resolving a clarification SHALL feed the **Knowledge
  Quality Engine** (Vol 05) so authors learn which sections need improvement. *Source:*
  research(ResumeCommunity — "marks confusing part Clarified ... feeds Knowledge
  Quality Engine"); Vol 05. *Note:* the *live-call* anchored to a widget is Vol 12.

- **EKOS-08-035 (Should, P2)** — A recruiter/consumer "Request Additional Info" flow
  (e.g., on a résumé/PKS projection) SHALL be the same `ClarificationRequest` object,
  with the response (text/audio/video) shown only to that requester via ABAC. *Source:*
  research(ResumeCommunity — recruiter clarification loop); Vol 04, Vol 10.

---

## 10. Real-Time Collaboration

> The data model is CRDT-able (Vol 01 §12); the collaboration UX lives here.

- **EKOS-08-036 (Must, P2)** — Multiple users SHALL **co-edit in real time** via
  **CRDT-based sync** with **presence awareness** (who's here, where their cursor is).
  *Source:* research(architecture — CRDTs); ConfusionClarity(final-spec §Collaboration);
  [[project-esip]] (Yjs Mermaid collab early impl).

- **EKOS-08-037 (Must, P2)** — For **spatial/diagram co-editing**, plain text CRDTs are
  insufficient; the system SHALL implement **Visual Spatial Optimistic Locking**: while
  User A drags a node cluster, it enters a **"ghost lock"** state for User B with a
  subtle real-time indicator. *Source:* Gemini critique (Visual Spatial Conflict
  Resolution); research(spatial conflict).

- **EKOS-08-038 (Must, P2)** — On simultaneous conflicting spatial placements (network
  lag), a **geometric vector layout algorithm** SHALL place nodes adjacently so nothing
  is stacked or lost off-screen. *Source:* Gemini critique.

- **EKOS-08-039 (Should, P2)** — Real-time collaboration SHALL operate within each
  user's **ABAC projection** (no co-editor sees beyond their clearance). *Source:*
  Vol 04; synthesis.

- **EKOS-08-040 (Should, P2)** — The existing ESIP **Yjs-based real-time Mermaid
  collaboration** SHALL be recognized as the early implementation and generalized to
  graph-backed spatial collaboration. *Source:* [[project-esip]]/[[esip-resume-state]]
  (Yjs collab task #5).

---

## 11. Working Modes & Adaptive Home

> ESIP Epic F is the early implementation.

- **EKOS-08-041 (Should, P2)** — Each persona SHALL have a **working-mode switcher**
  (e.g., design / requirements / estimation / review) whose choice is **remembered per
  persona**, reshaping the home and the work nav. *Source:* [[project-esip]] Epic F
  (workingModeStore, mode-by-persona memory).

- **EKOS-08-042 (Should, P2)** — An **adaptive home** SHALL present mode-tailored,
  persona-aware "start here" actions and recent items. *Source:* [[project-esip]] Epic F
  (WorkspaceHomePage).

- **EKOS-08-043 (Should, P2)** — The **persona model** (config-driven `enabledNav`,
  `defaultWorkingMode`, `requirementTypes`, `roadmapType`, `knowledgeHubTabs`) SHALL be
  the basis on which screens/workflows are shaped; switching persona reshapes the
  workspace. *Source:* [[project-esip]] Epic B/C (personaStore). *Note:* full persona
  catalog (14 personas) belongs to a persona-registry concern; the *customization
  mechanism* is here.

---

## 12. Sandbox / Staging of Customization Changes

- **EKOS-08-044 (Should, P2)** — New Screen Blueprints, workflows, triggers, themes, and
  ontology extensions SHALL be **testable in a sandbox** (workspace clone) and
  **promoted to production via the governed pipeline** (Vol 05 §13) — never edited
  directly in production. *Source:* ConfusionClarity(#4 — Sandbox & Staging); Vol 05
  EKOS-05-043/044. *Depends:* Vol 05.

---

## 13. Relationship to Existing ESIP

- **EKOS-08-045 (Must, MVP)** — ESIP's shipped **AppShell + persona switcher + working-
  mode engine + adaptive home + cross-role workflow inbox + feature-flag-gated nav**
  (Epics A/B/C/D/F/H) SHALL be recognized as the **early, partial implementation** of
  this customization & workflow layer, to be generalized onto the EKOS graph backbone.
  *Source:* [[project-esip]] (Epics A–H complete); [[esip-resume-state]].

- **EKOS-08-046 (Should, P2)** — Migration path: persona/workspace/workflow state moves
  from the current Firebase/Zustand stores to graph-backed `ExperienceDefinition` /
  `UserWorkspace` / workflow nodes behind the Vol 01 DAL. *Source:* [[project-esip]]
  (evolve behind seams); Vol 01.

---

## 14. Constitution Conformance (customization-specific)

- **EKOS-08-047 (Must, MVP)** — All customization artifacts (blueprints, workspaces,
  workflows, themes) are **graph nodes** — single source of truth, versioned, traceable
  (Constitution §4). *Source:* Constitution §4.

- **EKOS-08-048 (Must, MVP)** — Conditional visibility & co-editing run within the **ABAC
  projection** (Constitution security spirit). *Source:* Vol 04.

- **EKOS-08-049 (Must, MVP)** — **No silent overrides:** when personalization conflicts
  with a mandatory org rule, the user is shown the lock/tooltip — never silently
  altered. *Source:* EKOS-08-014; transparency principle.

- **EKOS-08-050 (Should, P2)** — Workflow/automation engines and CRDT/collaboration
  backends are **illustrative & pluggable** (Constitution §6). *Source:* Constitution §6.

---

## 15. Open Questions (parked)

- **OQ-08-A** — Workflow-object ownership boundary: EKOS-05-024 AI-output review gate &
  EKOS-08-028 cross-role workflow share the `WorkflowRequest` object — confirm one
  canonical model (this volume owns it; Vol 05 governs it).
- **OQ-08-B** — CRDT + bitemporal-versioning interaction (how concurrent edits reconcile
  with append-only history) — design with Vol 01 OQ-01-B.
- **OQ-08-C** — Blueprint precedence rules (tenant → persona → project → user override
  order) — needs a definitive resolution ruling (relates to Vol 01 ESIP context
  "admin → project → user override").
- **OQ-08-D** — Custom-script trigger sandbox & resource limits — shared with Vol 02
  OQ-02-B / Vol 05.
- **OQ-08-E** — Spatial-conflict UX details (ghost-lock granularity, lock timeout) —
  prototype with the rendering engine (Vol 02).

---

## 16. Requirement Index (this volume)

EKOS-08-001 … EKOS-08-050 (stable numbering; org config, personalization, workflow
designer, cross-role engine, clarification object, real-time collaboration, working
modes, sandbox expand to 80 distinct stated rules).

> **Next volume:** `09 — Enterprise Table-Stakes & Trust (i18n/WCAG/SSO/SCIM/sandbox/
> DR/residency/analytics/bulk-import/e-sign/feature-flags/admin-audit/offline/APIs/
> licensing/SLAs)`.
> Per protocol: STOP here. Await user "proceed".
