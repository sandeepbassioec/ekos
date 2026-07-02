# Volume 14 — Personalization & Adaptive Intelligence

**Catalog Prefix:** `PERS`
**Status:** Draft v1.0 — 2026-06-23
**Owner:** ESIP Product — Architecture Automation Suite
**Depends On:** V06 (AI/RAG/Agent Infrastructure), V11 (Platform NFR, Identity, Telemetry, Privacy), V12 (Cross-Role Workflow Engine), V13 (Tenant/License/Admin/Onboarding)

---

## Scope

This volume defines requirements for ESIP's per-user adaptive intelligence and personalization engine. It covers:

- First-run experience after activation (arriving from V13 activation link)
- Explicit preference capture (dashboard, theme, density, working mode)
- Personal profile and preference model (explicit + inferred)
- Adaptive dashboards and home surfaces
- Behavioral telemetry and working-pattern model
- Repetition detection and personal automation
- Skill/coverage gap detection and guided growth
- Proactive next-best-action suggestions
- Personal AI assistant memory and style adaptation
- Process-mode engine (design / estimation / requirements / review + custom)
- Team/peer intelligence (anonymized, tenant-scoped)
- Feedback and learning loop
- Transparency, user control, and ethical guardrails
- NFRs: latency, cold-start, data minimization, tenant isolation

**Out of scope (boundary):**
- AI model hosting, RAG pipelines, agent orchestration → V06
- Identity/SSO, raw telemetry pipeline, privacy infrastructure → V11
- Cross-role request/approval workflows → V12
- Tenant provisioning, license management, admin portals, activation email → V13
- Persona definitions and metamodel → V05/V11

---

## Personas

| Code | Role |
|------|------|
| P1 | Enterprise Architect |
| P2 | Solution Architect |
| P3 | Application Architect |
| P4 | Integration Architect |
| P5 | API Architect |
| P6 | CRM Architect |
| P7 | Data Architect |
| P8 | Security Architect |
| P9 | Cloud Architect |
| P10 | Infrastructure Architect |
| P11 | Microservices Architect |
| P12 | Domain/Emerging role (extensible) |
| All | All of the above |

---

## 1. First-Run Wizard & Working-Mode Selection

### 1.1 Activation Entry & Wizard Launch

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-001 | Wizard Trigger on First Login | The system SHALL automatically launch the first-run personalization wizard when a user authenticates for the first time after completing the V13 activation flow; the wizard SHALL NOT re-launch on subsequent logins unless explicitly reset by the user. | Functional | All | Must | Ensures setup is skipped for returning users; ties to V13 activation token. |
| REQ-PERS-002 | Wizard Skip + Resume | The system SHALL allow the user to skip the wizard at any step, preserving already-captured answers, and SHALL present a "Complete your profile" prompt on the next three logins until fully completed. | UX | All | Should | Reducing friction while capturing preferences asynchronously. |
| REQ-PERS-003 | Wizard Progress Indicator | The wizard SHALL display a linear step indicator showing current step, total steps (≤7), and an estimated time-to-complete (≤3 minutes). | UX | All | Should | ESIP users are busy architects; sub-3-minute setup reduces drop-off. |
| REQ-PERS-004 | Wizard Resumability | The system SHALL persist partially completed wizard state server-side so that if the user closes the browser mid-wizard, their answers are restored on re-open. | Functional | All | Must | Prevents re-entry frustration for users interrupted during setup. |

### 1.2 Visual & Layout Preferences (Wizard Step 1)

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-010 | Theme Selection | The wizard SHALL present theme options (Light, Dark, System-default, High-Contrast) and persist the selection as the user's default theme; the user SHALL be able to change this at any time via preferences. | UX | All | Must | Accessibility and working environment comfort. |
| REQ-PERS-011 | Information Density | The wizard SHALL offer density presets (Compact, Balanced, Spacious) affecting padding, font size, sidebar width, and table row height across the entire application; tenant admins SHALL be able to lock density. | UX | All | Must | Architect tooling is data-dense; density affects cognitive load. |
| REQ-PERS-012 | Default Landing Surface | The wizard SHALL ask which surface the user wants to see on login: Dashboard (default), Last-Visited Workspace, Requirements Pillar, Roadmap Pillar, or Knowledge Hub; the system SHALL honor this on every subsequent login. | UX | All | Must | Different architect roles have different primary workflows. |
| REQ-PERS-013 | Diagram Canvas Defaults | The wizard SHALL capture preferred default diagramming notation (ArchiMate 3, UML, C4, BPMN, Freeform) and default canvas orientation (left-right, top-bottom, free); these SHALL pre-fill the "new diagram" dialog. | UX | P1–P11 | Should | Reduces clicks per diagram creation; a P8 always opens threat models, a P4 always opens sequence diagrams. |
| REQ-PERS-014 | Language & Locale | The wizard SHALL capture the user's preferred display language and date/number locale; this SHALL override tenant-level locale for that user's session. | UX | All | Must | Multi-national enterprise teams. |

### 1.3 Working Process Preference (Wizard Step 2) — Core Innovation

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-020 | Working Mode Selection | The wizard SHALL present four built-in working modes and the option to define a custom mode; the selected mode SHALL determine the default navigation order, foregrounded pillars, and step prompts when the user creates or opens an architecture workspace. Built-in modes: **Design-First** (diagram → requirements → NFRs → decisions), **Requirements-First** (requirements → design → NFRs → roadmap), **Estimation-First** (scope → cost model → dependencies → design), **Review-First** (compare → gap analysis → findings → decisions). | Functional | All | Must | Different architect personas and project phases require radically different workflow entry points. |
| REQ-PERS-021 | Mode Description Preview | Each mode option in the wizard SHALL display an animated flow preview (3–5 step visual) showing what the workspace navigation will look like, so the user can make an informed choice without already knowing ESIP's features. | UX | All | Should | Self-explanatory modes increase correct selection; architects are visual thinkers. |
| REQ-PERS-022 | Custom Mode Builder (Wizard Entry) | The wizard SHALL allow the user to drag-and-drop up to 8 named steps from a palette of all available ESIP pillars/tools to construct a personal custom workflow mode; this custom mode SHALL be saved as a named template. | Functional | All | Could | Power users will want bespoke flows for client/domain-specific methodologies. |
| REQ-PERS-023 | Mode Inheritance | If the user skips mode selection, the system SHALL assign a default mode based on the persona role set in V13 provisioning (e.g., P8 Security Architect defaults to Review-First; P7 Data Architect defaults to Requirements-First). | Functional | All | Must | Cold-start quality; persona-based defaults outperform generic defaults. |

### 1.4 Role & Skill Self-Assessment (Wizard Step 3)

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-030 | Skill Self-Assessment | The wizard SHALL present a concise 8-question self-assessment covering: years of architecture experience, primary domain, frameworks used (TOGAF, Zachman, ArchiMate, etc.), typical deliverable types, primary stakeholders, and current toolchain; responses SHALL seed the user's competency model. | AI/ML | All | Should | Seeds the gap-detection model and contextual tip engine without requiring behavioral data upfront. |
| REQ-PERS-031 | Persona Multi-Select | The wizard SHALL allow users to select multiple active personas (e.g., P2 + P4); the system SHALL weight recommendations and navigation defaults to the primary persona but keep secondary personas enabled. | Functional | All | Must | Architects commonly wear multiple hats; rigid single-persona assignment is a known pain point. |
| REQ-PERS-032 | Assessment Skip → Behavioral Inference | If the user skips the self-assessment, the system SHALL mark the skill model as "inferred-only" and build it exclusively from behavioral telemetry (REQ-PERS-100 series); guidance nudges SHALL be throttled until the model reaches 70% confidence. | AI/ML | All | Should | Graceful degradation for reluctant self-assessors; avoids mis-targeted tips. |

---

## 2. Personal Profile & Preference Model

### 2.1 Preference Hierarchy & Precedence

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-040 | Three-Tier Preference Stack | The system SHALL resolve preferences using a three-tier hierarchy: **Tenant Policy** (locked, set by tenant admin) overrides **Role/Persona Default** (set by V05 persona config) overrides **User Explicit Setting**; user settings SHALL only override the tiers below tenant policy. | Governance | All | Must | Enterprise tenants need guardrails (branding, security) without denying user customization. |
| REQ-PERS-041 | Per-Workspace Overrides | The system SHALL allow users to override theme, density, and working mode at the individual workspace level; workspace-level overrides SHALL persist with the workspace and restore when it is reopened. | Functional | All | Should | An architect may switch to Review-First mode for a specific audit workspace while using Design-First as their global default. |
| REQ-PERS-042 | Preference Versioning | The system SHALL maintain a timestamped log of at least the last 10 changes to any user preference; users SHALL be able to view and revert any prior preference state from the settings UI. | Functional | All | Should | Accidental preference changes are a user support hot-spot; self-service revert reduces tickets. |
| REQ-PERS-043 | Preference Export/Import | The system SHALL allow users to export their full preference profile as a portable JSON document and re-import it on another account or tenant instance; the import SHALL validate against the current schema and warn on unknown fields. | Functional | All | Could | Enterprise teams share standardized setups; consultants carry settings across client tenants. |
| REQ-PERS-044 | Tenant-Level Role Defaults | Tenant admins SHALL be able to define persona-specific default preference sets that apply to all new users provisioned with that persona; user-level explicit settings SHALL override these once set. | Governance | All | Must | Onboarding at scale requires sensible per-persona defaults without manual configuration of each user. |

### 2.2 Preference Categories

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-050 | Notification Preferences | The system SHALL allow users to configure, per notification type (suggestion, automation, peer pattern, system alert): delivery channel (in-app, email, none), frequency (real-time, digest, off), and quiet hours with timezone. | UX | All | Must | Notification fatigue is a leading reason users disable personalization features. |
| REQ-PERS-051 | AI Interaction Style | The system SHALL store the user's preferred AI response style: verbosity (Concise / Balanced / Detailed), tone (formal / collaborative / terse), output format preference (bullet-first, narrative-first, table-first), and terminology profile (see REQ-PERS-220). | AI/ML | All | Should | Architects have highly individual preferences for AI output; one-size prompting reduces adoption. |
| REQ-PERS-052 | Shortcut & Keybinding Profile | The system SHALL persist per-user keyboard shortcut overrides for at least 30 named actions; import/export of shortcut profiles SHALL be supported. | UX | All | Should | Power users invest in muscle memory; custom shortcuts are a retention driver. |
| REQ-PERS-053 | Sidebar & Navigation Pin Config | The system SHALL persist which nav items the user has pinned, hidden, or reordered in the sidebar; these SHALL override persona defaults and survive browser/device changes. | UX | All | Must | Navigation personalization is a basic usability expectation. |

---

## 3. Adaptive Dashboards & Home Surface

### 3.1 Role- and Behavior-Aware Home

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-060 | Adaptive Home Layout | The system SHALL render a home dashboard composed of configurable widgets; the initial widget set and layout SHALL be determined by the user's primary persona and working mode; the layout SHALL evolve over time based on usage signals (dwell time, click frequency, task completions). | AI/ML | All | Must | A static dashboard becomes stale; adaptive layouts surface the work that actually matters. |
| REQ-PERS-061 | Continue Where You Left Off | The home dashboard SHALL display a "Resume" card surfacing the last 3 open workspaces with activity timestamps, completion percentage of the current phase, and the next recommended step per working mode. | UX | All | Must | Architect sessions are frequently interrupted; reducing re-orientation time is a top productivity gain. |
| REQ-PERS-062 | Widget Usage Tracking | The system SHALL track per-widget interaction events (click, hover, expand, dismiss, ignore) and surface a weekly widget relevance score; widgets scoring below threshold for 4 consecutive weeks SHALL be flagged for removal with user consent. | AI/ML | All | Should | Dashboards accumulate dead widgets; proactive cleanup improves signal-to-noise. |
| REQ-PERS-063 | Widget Catalog | The system SHALL maintain a catalog of at least 20 named widget types (recent work, my requirements, my diagrams, team activity, suggestion inbox, automation status, roadmap burndown, knowledge hub bookmarks, peer patterns, compliance checklist, AI chat entry, quick-create, calendar, search, metrics, decision log, gap alerts, upcoming reviews, published deliverables, AI usage quota); users SHALL be able to add/remove/resize/reorder any widget. | UX | All | Must | Rich widget catalog enables role-appropriate customization without requiring custom development. |
| REQ-PERS-064 | Saved Dashboard Views | Users SHALL be able to save named dashboard layouts ("Monday Review", "Deep Work", "Client Presentation") and switch between them from a quick-picker; up to 10 named views per user. | UX | All | Should | Context-switching between project roles or meeting modes is common; named views eliminate repeated manual rearrangement. |
| REQ-PERS-065 | Time-of-Day Dashboard Shift | The system SHALL optionally apply a time-of-day rule (configurable) that automatically switches the active dashboard view at specified times (e.g., "mornings → pipeline review; afternoons → deep-work mode"); users SHALL be able to enable/disable/configure this rule. | UX | All | Could | Circadian workflow patterns are measurable; automatic mode switching reduces decision fatigue. |

### 3.2 Widget Intelligence

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-070 | Smart Recent Work | The "Recent Work" widget SHALL rank items not just by recency but by a composite score of recency, frequency, completion-proximity, and pending actions; items with an overdue step SHALL be visually flagged. | AI/ML | All | Should | Pure recency ranking is inferior to intent-weighted ranking for architects managing multiple concurrent engagements. |
| REQ-PERS-071 | Deadline & Milestone Proximity Widget | The system SHALL surface a widget that aggregates user-owned milestones and roadmap items with approaching due dates, sorted by urgency; days-to-deadline SHALL be computed from roadmap metadata. | Functional | P1, P2, P9 | Should | Architects own deliverables with deadlines; passive deadline visibility reduces missed commitments. |
| REQ-PERS-072 | My Pending Actions Widget | The system SHALL provide a widget aggregating all items requiring the user's action across all pillars (requirements awaiting approval, diagrams flagged for review, automation suggestions, open peer comments); action items SHALL be completable inline from the widget. | UX | All | Must | Reduces context-switching to complete small actions. |

---

## 4. Behavioral Telemetry & Working-Pattern Model

> **Training-data boundary (C1 — cross-ref 00-MASTER-INDEX §6 ruling #2: "no training on customer CONTENT; anonymized opt-in behavioral only").** The behavioral telemetry and pattern-mining capabilities in Sections 4–7 and 11 use **anonymized structural signals to drive the user's OWN personalization** (their dashboards, suggestions, automations). Dismissals, accept/reject feedback, and other behavioral signals are **NOT sent to fine-tune or train any shared/global model without explicit per-tenant opt-in/consent** (gated via REQ-PERS-100 and the V11 consent system of record, REQ-PERS-324). **Customer content is never training data** — captured telemetry is restricted to structural metadata only (REQ-PERS-103); requirement text, diagram labels, and chat messages are excluded by design. Any model-improvement use of behavioral signals beyond the individual user's own personalization is consent-gated and PII-scrubbed.

### 4.1 Event Capture

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-100 | Opt-In Behavioral Telemetry | The system SHALL collect per-user behavioral events ONLY after the user provides explicit opt-in consent during or after the first-run wizard; consent SHALL be granular (telemetry-for-automation, telemetry-for-suggestions, telemetry-for-team-patterns) and reversible at any time. | Security | All | Must | GDPR/CCPA compliance; consent is per-purpose not blanket. |
| REQ-PERS-101 | Event Taxonomy | The system SHALL capture the following event categories: navigation events (page/view transitions), creation events (new artifact, new workspace, new diagram), edit events (field changes, diagram edits), search/query events (queries, filters, result selections), AI interaction events (prompts, accepted/rejected suggestions), export/publish events, and abandonment events (started but not completed within session). Each event SHALL carry: event_type, resource_type, resource_id (hashed), timestamp (UTC), session_id, working_mode, persona_active, and tenant_id. | Data | All | Must | Comprehensive taxonomy is required for pattern detection; event schema must be stable for the downstream model. |
| REQ-PERS-102 | No Cross-Tenant Leakage | The behavioral telemetry pipeline SHALL enforce tenant-scoped storage; no telemetry event from tenant A SHALL ever be queryable, retrievable, or inferable from tenant B's session; isolation SHALL be enforced at both storage and inference layers. | Security | All | Must | Enterprise tenants have contractual data-isolation obligations. |
| REQ-PERS-103 | Telemetry Data Minimization | The system SHALL NOT capture the content of user-entered text (requirement descriptions, diagram labels, chat messages) in telemetry events; it SHALL capture only structural metadata (artifact type, field name, character-count range, AI-feature-used). | Security | All | Must | Data minimization principle; textual content belongs to the user/tenant, not the platform telemetry store. |
| REQ-PERS-104 | Session Replay Opt-In (Separate) | The system SHALL support an optional separate consent for full session replay (for support purposes only); session replay data SHALL be stored encrypted, scoped to a 30-day retention window, and accessible only to authorized support staff, never to the personalization engine. | Security | All | Could | Session replay is a powerful support tool but must be governed separately from behavioral personalization. |
| REQ-PERS-105 | Telemetry Retention Policy | The system SHALL enforce a configurable per-tenant behavioral telemetry retention window (default 180 days, max 730 days); on expiry, raw events SHALL be deleted; aggregated pattern summaries SHALL persist unless the user requests full erasure under GDPR right-to-erasure. | Data | All | Must | Balances model quality (needs history) with regulatory retention limits. |

### 4.2 Working-Pattern Model

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-110 | Per-User Working-Pattern Model | The system SHALL maintain a per-user working-pattern model updated incrementally after each session; the model SHALL encode: most-frequent task types, most-used features, most-visited resource types, median session length, time-of-day activity distribution (morning/afternoon/evening buckets), and most-common navigation sequences (top-10 sequential pairs). | AI/ML | All | Must | This model is the foundational input to all adaptive, suggestion, and automation features. |
| REQ-PERS-111 | Frequency-Weighted Feature Usage Profile | The system SHALL compute a per-user feature-usage profile as a vector of features → normalized usage frequency; this profile SHALL be used for widget relevance scoring (REQ-PERS-062), navigation customization, and gap detection (REQ-PERS-150 series). | AI/ML | All | Must | Feature-usage vectors enable cosine-similarity comparisons for peer pattern matching. |
| REQ-PERS-112 | Session Phase Pattern Detection | The system SHALL detect the per-user distribution of session "phases" (exploration, creation, review, export) by analyzing action sequences within sessions; users who rarely enter review phases SHALL be flagged in the gap model. | AI/ML | All | Should | Phase distribution reveals blind spots in working practice invisible to the user themselves. |
| REQ-PERS-113 | Dwell & Abandonment Signals | The system SHALL record dwell time per page/view and detect abandonment patterns (navigated away without completing a wizard, form, or dialog); abandonment on the same step ≥3 times SHALL trigger a contextual help nudge for that step. | AI/ML | All | Should | Abandonment patterns identify UX friction and skill gaps simultaneously. |
| REQ-PERS-114 | Time-Envelope Modeling | The system SHALL model each user's productive hours as a time-of-day envelope derived from telemetry; this envelope SHALL be used to time suggestion delivery (suggestions sent during inactive hours are deferred to the next active window). | AI/ML | All | Could | Suggestions delivered at the wrong time of day are dismissed at 3× the rate of well-timed ones (industry research in recommender systems). |
| REQ-PERS-115 | Cold-Start Bootstrapping | For users with fewer than 5 sessions of telemetry data, the system SHALL substitute a persona-role prior (derived from anonymized aggregate patterns across the tenant's users of the same persona) until sufficient personal data exists; the switch SHALL be transparent to the user. | AI/ML | All | Must | Cold-start is the hardest problem in personalization; persona priors provide immediately useful defaults. |

---

## 5. Repetition Detection & Personal Automation

### 5.1 Pattern Recognition

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-120 | Action-Sequence Mining | The system SHALL apply a sequential-pattern mining algorithm (minimum support threshold: 3 occurrences within 14 days; minimum sequence length: 3 steps) to each user's event log; detected sequences SHALL be stored as candidate automation patterns with occurrence count and recency. | AI/ML | All | Must | Process mining on individual event logs is the core innovation enabling personal automation; without it, automations are manual-only. |
| REQ-PERS-121 | Pattern Deduplication | The system SHALL deduplicate candidate patterns that are sub-sequences of a longer already-detected pattern, retaining only the longest matching sequence; users SHALL NOT be notified of both a 3-step and a 5-step pattern when the 3-step is a strict prefix of the 5-step. | AI/ML | All | Should | Prevents notification flooding from overlapping patterns. |
| REQ-PERS-122 | Cross-Resource Pattern Detection | The system SHALL detect patterns that span multiple resource types (e.g., "create requirement → attach diagram → add NFR tag → export PDF") in addition to single-resource repetitions; cross-resource patterns SHALL be ranked higher in the automation suggestion queue. | AI/ML | All | Should | The most valuable automatable patterns span multiple tools; single-resource repetitions are often too coarse to be worth automating. |

### 5.2 Automation Suggestion & Creation

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-130 | Automation Suggestion Notification | When a candidate pattern meets the threshold (REQ-PERS-120), the system SHALL surface a non-intrusive banner notification: "You do [A → B → C] frequently — automate it in one click?" with Accept, Remind Later, and Never options. | UX | All | Must | The suggestion must be frictionless; one wrong touch-target layout will kill adoption. |
| REQ-PERS-131 | One-Click Personal Macro Creation | On accepting an automation suggestion, the system SHALL present a confirmation dialog showing the detected steps, allow the user to name the macro and optionally parameterize variable steps (e.g., replace a hardcoded workspace name with a prompt variable), and save it as a personal macro runnable from the command palette or quick-action bar. | Functional | All | Must | One-click creation must remain one click; parameterization is offered but optional. |
| REQ-PERS-132 | Macro Editor | The system SHALL provide a visual macro editor (step list with drag-to-reorder, per-step parameter binding, add/remove steps, rename) accessible after macro creation; macros SHALL be versioned and revertable. | Functional | All | Should | Power users will want to refine auto-generated macros; an editor prevents "delete and recreate" workflows. |
| REQ-PERS-133 | Parameterized Templates from Past Work | The system SHALL allow users to select any previously completed workspace and generate a parameterized template from it by replacing detected variable values (project names, dates, technology choices) with named placeholder variables; the template SHALL be saveable to the user's personal template library. | Functional | All | Should | Architects reuse past work constantly; template extraction from history is a high-value productivity multiplier. |
| REQ-PERS-134 | Scheduled Personal Automations | Personal macros SHALL be schedulable via a cron-like simple UI (daily, weekly, on-event triggers such as "when a new workspace is assigned to me") without requiring knowledge of cron syntax; the scheduler SHALL validate the schedule against the user's active workspace list before execution. | Functional | All | Could | Scheduled automations (e.g., weekly compliance check) require no-code scheduling accessible to non-developers. |
| REQ-PERS-135 | Promote Automation to Team/Tenant | Users SHALL be able to submit a personal macro or template to a "team automation" approval queue visible to the tenant admin; upon admin approval, the automation becomes available to all users of the same persona within the tenant. | Governance | All | Could | Bottom-up automation discovery is a proven model in RPA platforms; personal macros become organizational assets. |
| REQ-PERS-136 | Macro Execution History | The system SHALL maintain a per-user macro execution log (timestamp, macro name, outcome: success/partial/failed, steps completed, parameters used); users SHALL be able to replay failed macros from the last successful step. | Functional | All | Should | Execution history is essential for debugging and auditing automated actions on architecture artifacts. |
| REQ-PERS-137 | Automation Dry-Run | Before executing any macro or scheduled automation, the system SHALL offer a dry-run mode that simulates the steps and displays a preview of changes without committing them; dry-run output SHALL list created/modified/deleted artifacts. | Functional | All | Should | Destructive automation steps (delete, overwrite) require safe rehearsal capability. |

---

## 6. "What They Lack" — Guided Assistance & Growth

### 6.1 Skill & Coverage Gap Detection

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-150 | Coverage Gap Model | The system SHALL compute a per-user coverage gap model by comparing the user's feature-usage profile (REQ-PERS-111) against a persona-specific "expected coverage baseline" derived from aggregate peer data; gaps SHALL be ranked by frequency-weighted importance and recency of last use. | AI/ML | All | Must | Feature coverage gaps directly correlate with deliverable quality gaps; detection enables targeted guidance. |
| REQ-PERS-151 | NFR Step Gap Detection | The system SHALL specifically detect whether the user consistently creates requirements without adding NFR attributes; if the NFR completion rate is below 50% of created requirements for 3+ workspaces, the system SHALL surface a persistent contextual nudge when the user creates a new requirement. | AI/ML | All | Must | NFR omission is the single most common quality gap in architecture deliverables across all personas. |
| REQ-PERS-152 | Diagram Notation Gap | The system SHALL detect which ArchiMate layers, UML diagram types, or C4 levels the user has never or rarely used; if the user's persona baseline includes these, the system SHALL surface a "Try this: [notation type]" contextual tip with a one-click pre-populated example. | AI/ML | P1, P2, P3, P4 | Should | Notation gaps (e.g., a P1 who has never drawn a motivation diagram) indicate methodology blind spots. |
| REQ-PERS-153 | Decision Log Gap | The system SHALL detect workspaces where the user created multiple architecture artifacts but never opened the decision log (ADR); after two such workspaces, the system SHALL surface an ADR creation prompt with context auto-filled from the most recent change. | AI/ML | All | Should | Decision log omission leads to architectural drift; early habit formation is more effective than retrospective correction. |
| REQ-PERS-154 | Competency Maturity Score | The system SHALL compute a per-user competency maturity score (1–5) per dimension (requirements depth, diagram coverage, NFR completeness, knowledge-hub contribution, roadmap usage, decision logging, AI tool usage) and display it in a spider/radar chart on the user's profile page; dimensions SHALL be explainable with specific gap items. | AI/ML | All | Should | Quantified maturity scores give users a concrete growth target and make gaps concrete rather than vague. |
| REQ-PERS-155 | Peer-Relative Gap Framing | Gap notifications SHALL frame gaps relative to anonymized peers ("Architects with your profile typically also complete [X]; you haven't yet") rather than as failures; language SHALL be constructive and opt-out-able. | UX | All | Must | Shame-based gap communication causes feature avoidance; social-norm framing (Cialdini) is empirically more effective. |

### 6.2 Contextual Guidance & Learning Nudges

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-160 | Just-in-Time Contextual Tips | The system SHALL surface just-in-time tips in the UI at the point of context (e.g., a tooltip when hovering an unused feature, a side-panel tip when entering a view for the first time); tips SHALL be dismissible permanently ("don't show again") and SHALL NOT re-appear after 2 dismissals. | UX | All | Must | Just-in-time delivery is 2–3× more effective than upfront documentation (learning research). |
| REQ-PERS-161 | Guided Checklists | The system SHALL provide per-workspace guided checklists (configurable by working mode) showing recommended steps; checklist items SHALL auto-check when the corresponding action is detected in telemetry; unchecked items after workspace export SHALL be highlighted. | Functional | All | Should | Checklists externalize working-memory load, increasing completeness for complex multi-step deliverables. |
| REQ-PERS-162 | Persona-Specific Learning Path | The system SHALL surface a recommended learning path per persona (P1–P13) drawn from the Knowledge Hub (V08); path items SHALL be ordered by detected gap priority; completed items SHALL be tracked and removed from the path. | Functional | All | Could | Learning paths tailored to detected gaps are markedly more effective than generic feature tours. |
| REQ-PERS-163 | Tip Frequency Control | Users SHALL be able to control tip/nudge frequency with a slider (Frequent / Moderate / Minimal / Off-for-30-days); the system SHALL automatically adjust frequency based on session duration (new short-session users get fewer interruptions). | UX | All | Must | Tip fatigue is a top contributor to feature avoidance; user-controlled frequency is a basic usability requirement. |

---

## 7. Proactive Suggestions / Next-Best-Action Engine

### 7.1 Suggestion Engine

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-170 | Next-Best-Action Model | The system SHALL maintain a per-user next-best-action model that ranks candidate suggestions at each context point (workspace open, artifact create, export, idle period > 5 minutes); suggestions SHALL be generated by combining: working-mode step sequence, historical pattern model, coverage gap model, and AI semantic context (via V06). | AI/ML | All | Must | Context-aware next-best-action is the primary mechanism for surfacing value without user-initiated search. |
| REQ-PERS-171 | Suggestion Types | The suggestion engine SHALL support at least the following named suggestion types: **Continue** (resume in-progress step), **Attach** ("you usually attach a threat model here"), **Classify** ("this requirement looks like ones you marked Must"), **Reuse** ("you built similar pattern in [workspace]"), **Peer** ("architects like you also add [X]"), **Gap** ("this workspace is missing NFRs"), **Automate** ("you've done this 4 times — automate?"), **Review** ("this diagram hasn't been reviewed in 14 days"), **Export** ("milestone due in 2 days"). | AI/ML | All | Must | Typed suggestions enable per-type accept/reject feedback, per-type opt-out, and per-type analytics. |
| REQ-PERS-172 | Suggestion Ranking & Confidence | Each suggestion SHALL carry a confidence score (0–1) derived from the underlying model; only suggestions above a configurable threshold (default 0.60) SHALL be shown; threshold SHALL be adjustable per suggestion type by tenant admin and per the user's sensitivity preference. | AI/ML | All | Must | Unfiltered low-confidence suggestions rapidly erode trust; threshold controls are essential for tuning signal-to-noise. |
| REQ-PERS-173 | Suggestion Inbox | The system SHALL maintain a persistent "Suggestion Inbox" accessible from the home dashboard and navigation sidebar, showing all unactioned suggestions ranked by relevance score, filterable by type and resource; the inbox badge count SHALL update in real time. | UX | All | Must | Surfacing suggestions only in-context misses users working at a distance from the current artifact; an inbox provides a persistent safety net. |
| REQ-PERS-174 | Do-It-For-Me Actions | High-confidence suggestions SHALL include a "Do it for me" action button that executes the suggestion with a preview-before-confirm dialog; the action SHALL be recorded in the user's action log with an "AI-assisted" flag; all AI-assisted actions SHALL be individually undoable. | Functional | All | Should | "Do it for me" converts passive suggestions into completed work; preview-before-confirm prevents unintended changes. |
| REQ-PERS-175 | Suggestion Dismissal & Snooze | Users SHALL be able to dismiss a suggestion permanently (never show this type for this resource), snooze it (resurface in N hours/days), or mark it as irrelevant (feeds the rejection training signal). | UX | All | Must | Granular dismissal options preserve user agency and generate high-quality negative training examples. |
| REQ-PERS-176 | Suggestion Explanation | Each suggestion SHALL display a one-sentence natural-language explanation of why it was generated ("because you added a threat model to 4 of your last 5 P8 workspaces"); users SHALL be able to expand the explanation to see the supporting evidence. | UX | All | Must | Explainability is a core ethical requirement; unexplained suggestions feel intrusive and reduce trust. |
| REQ-PERS-177 | Suggestion Delivery Timing | Suggestions SHALL be delivered at contextually appropriate moments: on workspace open, on artifact save, on navigation away from a view without completing the expected next step; suggestions SHALL NOT appear during active text entry or diagram drag operations. | UX | All | Must | Poorly timed suggestions break flow; timing rules prevent interruption at high-focus moments. |
| REQ-PERS-178 | Suggestion A/B Framework | The system SHALL support A/B testing of suggestion algorithms at the tenant level (configured by tenant admin); users in experiment arms SHALL be randomly assigned and SHALL NOT be aware of assignment; uplift SHALL be measured against accept-rate, completion-rate, and session-duration metrics. | AI/ML | All | Could | A/B testing enables empirical improvement of suggestion quality; tenant-scoped assignment prevents cross-tenant data contamination. |

---

## 8. Personal AI Assistant Memory

### 8.1 Per-User Memory Store

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-200 | Cross-Session AI Memory | The personal AI assistant (integrated via V06) SHALL maintain a per-user memory store persisted across sessions, containing: prior decisions, stated preferences, terminology choices, architecture patterns the user has approved or rejected, and frequently referenced standards or policies. | AI/ML | All | Must | Stateless AI assistants that "forget" prior context are a top user complaint; persistent memory is a fundamental expectation for a professional tool. |
| REQ-PERS-201 | Memory Store Schema | The memory store SHALL be structured as a set of named memory categories: **Decisions** (ADR summaries), **Preferences** (stated style/format preferences), **Terminology** (user-defined term mappings), **Patterns** (reusable architecture patterns approved by user), **Context** (active project summaries), and **Feedback** (what the user liked/disliked about prior AI outputs); each entry SHALL carry a timestamp and a confidence/source tag (user-explicit vs. inferred). | Data | All | Must | Structured memory categories enable targeted retrieval (RAG, V06) rather than brute-force context injection. |
| REQ-PERS-202 | Memory Store Capacity & Pruning | The memory store SHALL have a configurable per-user capacity limit (default 512 KB of serialized memory entries); on reaching capacity, the oldest/lowest-confidence entries SHALL be pruned automatically; users SHALL receive a notification before automatic pruning. | Data | All | Should | Unbounded memory growth is both a storage and a context-window cost problem; user-visible pruning maintains trust. |
| REQ-PERS-203 | Memory Injection into AI Context | On each AI assistant invocation, the system SHALL inject the top-K (default K=10) most relevant memory entries (ranked by semantic similarity to the current query via V06 RAG) into the AI prompt context; injection SHALL occur transparently without requiring user action. | AI/ML | All | Must | Relevant memory injection is the mechanism that makes persistent memory operationally useful. |
| REQ-PERS-204 | Explicit Memory Addition | Users SHALL be able to explicitly add entries to their memory store by selecting text in any ESIP view and choosing "Remember this" from the context menu; the added entry SHALL be categorized automatically and confirmable by the user. | UX | All | Should | Explicit memory gives users control over what the AI remembers, supplementing inferred entries. |
| REQ-PERS-205 | Memory View & Edit UI | The system SHALL provide a dedicated Memory page (accessible from user profile) listing all memory entries by category, with per-entry edit, delete, and category-change controls; users SHALL be able to download their full memory store as JSON. | UX | All | Must | Visibility and control over AI memory is a transparency and GDPR requirement. |
| REQ-PERS-206 | Memory Isolation | The memory store SHALL be scoped to the user × tenant combination; a user with access to multiple tenants SHALL have a separate memory store per tenant; cross-tenant memory leakage SHALL be architecturally impossible. | Security | All | Must | Multi-tenant memory leakage would expose confidential architecture decisions. |

### 8.2 Personalization of AI Tone & Output

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-210 | Dynamic Tone Adaptation | The AI assistant SHALL adapt its response tone and verbosity to the user's stated preferences (REQ-PERS-051) and SHALL update these preferences when the user provides explicit feedback ("be more concise", "use bullet points") mid-conversation; updates SHALL persist to the preference store. | AI/ML | All | Should | Mid-conversation style feedback is the most natural way users express AI style preferences; real-time adaptation increases satisfaction. |
| REQ-PERS-211 | Terminology Profile | The system SHALL maintain a per-user terminology mapping (e.g., "I call them 'building blocks', not 'components'") derived from memory entries and explicit additions; the AI assistant SHALL substitute user-preferred terms in its outputs. | AI/ML | All | Should | Terminology consistency between user and AI increases comprehension speed and reduces re-reading. |
| REQ-PERS-212 | Output Format Learning | The system SHALL track which AI response formats the user consistently copies, expands, or acts on (Markdown table, bullet list, numbered steps, prose) and bias future responses toward the most-accepted format; users SHALL be able to override the learned format at any time. | AI/ML | All | Could | Format preference learning reduces the need for explicit format instructions in every prompt, measurably reducing per-interaction latency. |
| REQ-PERS-213 | Prior Decision Awareness | When the AI assistant generates a suggestion or diagram, it SHALL check the user's Decision memory store for prior decisions relevant to the current context and SHALL NOT contradict them without explicitly flagging the contradiction with reasoning. | AI/ML | All | Must | AI assistants that contradict prior user decisions without acknowledgment rapidly destroy trust. |

---

## 9. Process-Mode Engine

### 9.1 Mode Definition & Navigation Reshaping

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-220 | Process Mode Registry | The system SHALL maintain a per-tenant process mode registry containing built-in modes (Design-First, Requirements-First, Estimation-First, Review-First) and user/team-defined custom modes; each mode SHALL be a named, ordered sequence of named steps where each step maps to one or more ESIP views, pillars, or AI-assisted actions. | Functional | All | Must | The mode registry is the configuration backbone of the process-mode engine; without it, modes are hard-coded and inextensible. |
| REQ-PERS-221 | Mode-Driven Navigation | When a mode is active, the system SHALL reorder the workspace sidebar navigation to place the current mode's steps at the top in order; completed steps SHALL display a checkmark; the current step SHALL be highlighted; users SHALL be able to skip ahead or return to any step. | UX | All | Must | Non-linear navigation within a mode must remain possible; architects frequently revisit earlier steps. |
| REQ-PERS-222 | Mode-Aware Default Content | When a user creates a new artifact in a mode context, the system SHALL pre-populate fields, templates, and AI prompts appropriate for the current mode step (e.g., in Estimation-First mode, a new requirements workspace pre-populates with a cost-estimation attribute group). | Functional | All | Should | Mode-appropriate defaults reduce manual configuration per artifact by an estimated 40% for frequent patterns. |
| REQ-PERS-223 | Mode Switch Prompt | The system SHALL detect when the user's actions within a session are inconsistent with their active mode (e.g., a Requirements-First user spending 80% of time in the diagram editor) and offer a mode-switch suggestion; the user SHALL be able to accept, decline, or set a new mode as default. | AI/ML | All | Could | Mode drift without user awareness leads to inconsistent deliverables; proactive re-alignment is a quality control mechanism. |
| REQ-PERS-224 | Custom Mode Builder (Full) | The system SHALL provide a full custom mode builder (accessible outside the first-run wizard) with: step name editor, step-to-view mapping (multi-select), per-step AI prompt template, per-step required/optional artifact list, completion criteria definition, and estimated time-per-step; custom modes SHALL be importable/exportable as JSON. | Functional | All | Should | Custom modes enable methodology-specific workflows (TOGAF ADM phases, SAFe ART planning) without platform code changes. |
| REQ-PERS-225 | Team Mode Sharing | Users SHALL be able to publish a custom mode to the tenant's shared mode library (subject to admin approval); shared modes SHALL be forkable by other users for personal customization. | Governance | All | Could | Shared modes codify team methodology standards; forkability allows individual adaptation without breaking the shared definition. |
| REQ-PERS-226 | Mode Binding to Workspace Type | Tenant admins SHALL be able to bind a specific process mode to a workspace template type (e.g., "Security Review workspaces always start in Review-First mode"), overriding user default for that workspace type; users SHALL be informed of the binding and may not override it unless permitted. | Governance | P1, P8 | Should | Standardizing methodology for regulated workspace types (security reviews, compliance audits) requires admin-enforced mode binding. |

### 9.2 Mode Metrics & Evolution

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-230 | Mode Completion Tracking | The system SHALL track per-mode, per-workspace step completion rates and time-per-step for each user; these metrics SHALL be visible on the user's profile (self-only) and summarized on the tenant admin dashboard (aggregated, anonymized). | Reporting | All | Should | Mode completion metrics identify bottleneck steps within a methodology, informing both user coaching and platform improvement. |
| REQ-PERS-231 | Mode Recommendation for New Workspaces | When a user creates a new workspace, the system SHALL recommend a working mode based on: workspace name/description semantic classification, past workspace patterns, and active project context; the recommendation SHALL be dismissible. | AI/ML | All | Could | Intelligent mode pre-selection reduces the "which mode?" decision for users creating workspaces under time pressure. |

---

## 10. Team & Peer Intelligence

> **Cold-start prior constraint (C7).** Any cold-start persona prior used to bootstrap a new user (REQ-PERS-023, REQ-PERS-115) or any peer-derived signal MUST be either (a) **tenant-scoped**, OR (b) **drawn from globally aggregated, anonymized statistics protected by differential privacy AND k-anonymity ≥ 50**. **An individual tenant's data is NEVER usable as a prior for another tenant** — there is no per-tenant-identifiable cross-tenant transfer. This strengthens the tenant-scoped peer rules below (REQ-PERS-243 cohort ≥5, REQ-PERS-244 no cross-tenant patterns) for the specific case of cold-start priors. See REQ-PERS-244b.

### 10.1 Anonymized Aggregate Patterns

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-240 | Peer Pattern Aggregation | The system SHALL compute anonymized aggregate behavioral patterns across users of the same persona within a tenant (minimum cohort size: 5 users to prevent re-identification); patterns SHALL represent: most-common feature sequences, most-used templates, most-common NFR types added, median time-per-workspace-phase, and most-common deliverable export types. | AI/ML | All | Should | Peer patterns are the most actionable form of social proof for professional tool adoption. |
| REQ-PERS-241 | "Architects Like You Also…" Suggestions | The suggestion engine SHALL include a peer-pattern suggestion type ("Architects in your role also use [X] at this stage") with an accept-rate tracked separately; peer suggestions SHALL link to an example or template (never to another user's work). | AI/ML | All | Should | Social proof suggestions have 20–40% higher accept rates than system-generated suggestions in recommender system research. |
| REQ-PERS-242 | Team Template Library | The system SHALL surface a tenant-scoped shared template library populated from: admin-published templates, promoted personal macros (REQ-PERS-135), and auto-extracted templates from high-quality (peer-rated) completed workspaces; users SHALL be able to search, preview, and fork any template. | Functional | All | Must | A common template library accelerates onboarding and codifies institutional architecture patterns. |
| REQ-PERS-243 | Cohort Anonymization Enforcement | Before any peer pattern data is used in a suggestion or displayed in the UI, the system SHALL verify that the underlying cohort has ≥5 distinct users; if the cohort shrinks below 5 (e.g., due to offboarding), the pattern SHALL be suppressed until cohort is restored. | Security | All | Must | k-anonymity threshold for pattern data; prevents reverse-engineering individual behavior from small teams. |
| REQ-PERS-244 | No Cross-Tenant Peer Patterns | Peer patterns SHALL be strictly scoped to the user's tenant; the system SHALL NOT produce cross-tenant aggregate patterns even when different tenants use the same industry vertical or persona mix. | Security | All | Must | Enterprise tenants compete with each other; cross-tenant intelligence sharing would be a severe contractual breach. |

### 10.2 Reconciliation Additions

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-244b | Cold-Start Persona Prior — Scope & Privacy Guarantee | Cold-start persona priors (consumed by REQ-PERS-023 mode inheritance and REQ-PERS-115 telemetry bootstrapping) SHALL be sourced **only** from: (a) **tenant-scoped** aggregates of the same-persona cohort within the user's own tenant (subject to the REQ-PERS-243 cohort ≥ 5 floor), OR (b) **globally aggregated, anonymized statistics protected by differential privacy AND k-anonymity ≥ 50**. The system SHALL NEVER use one tenant's data, in identifiable or reconstructable form, as a prior for a user in a different tenant. Where a tenant-scoped cohort is too small to meet the floor, the system SHALL fall back to the DP/k-anon ≥ 50 global prior or to a static persona default — never to another single tenant's data. | Security | All | Must | C7 reconciliation — cold-start priors must be useful without leaking competing tenants' behavior; tenant-scoping or DP + k-anonymity ≥ 50 are the only permitted sources. |

---

## 11. Feedback & Learning Loop

### 11.1 Signal Collection

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-250 | Explicit Suggestion Feedback | Every suggestion surface SHALL capture explicit feedback signals: Accept (acted on), Dismiss-Irrelevant, Dismiss-Already-Done, Snooze, and optionally a free-text "why not" field (max 140 chars); ALL signals SHALL be stored against the suggestion type and model version. | AI/ML | All | Must | Labeled feedback is the training signal for model improvement; without it, the model cannot learn from rejections. |
| REQ-PERS-251 | Implicit Feedback Capture | The system SHALL capture implicit feedback signals: whether a suggested artifact was subsequently created (positive), whether the session ended without acting on a suggestion (weak negative), and whether a "Do it for me" result was immediately undone (strong negative). | AI/ML | All | Must | Implicit signals are higher-volume than explicit; combining both increases learning-loop throughput. |
| REQ-PERS-252 | Feedback Attribution Window | Implicit acceptance attribution SHALL use a configurable time window (default: 30 minutes from suggestion display); actions occurring outside the window SHALL NOT be attributed to the suggestion. | AI/ML | All | Should | Attribution windows prevent false positives from coincidental actions unrelated to the suggestion. |
| REQ-PERS-253 | Model Retraining Trigger | The personalization models SHALL be retrained on a rolling basis: incremental updates after each session (lightweight), full model refresh weekly (per-tenant batch); the retraining schedule SHALL be configurable by platform operators. | AI/ML | All | Should | Incremental updates maintain recency; weekly batch refresh corrects accumulated drift. |

### 11.2 Quality Metrics

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-260 | Suggestion Accept Rate Tracking | The system SHALL track suggestion accept rate per suggestion type per user per week; rate trend SHALL be displayed on the tenant admin personalization dashboard; a rate below 10% for any type for 4 consecutive weeks SHALL trigger an alert. | Reporting | All | Must | Accept rate is the primary leading indicator of personalization quality. |
| REQ-PERS-261 | Personalization Quality Score | The system SHALL compute a per-tenant personalization quality score (PQS) as a composite of: suggestion accept rate, automation adoption rate, gap nudge-to-fix rate, and AI memory hit rate; PQS SHALL be displayed on the admin dashboard and benchmarked against anonymized platform-wide percentiles. | Reporting | All | Should | PQS gives tenant admins a single actionable metric for the value of personalization; percentile benchmarking provides external context. |
| REQ-PERS-262 | User Satisfaction Signal | The system SHALL present a 5-star micro-rating prompt after the first "Do it for me" completion in each session (max once per session); ratings SHALL be correlated with model version for regression analysis. | AI/ML | All | Could | Session-level satisfaction rating is a compact proxy for overall personalization satisfaction, usable in A/B analysis. |

---

## 12. Transparency, User Control & Ethics

### 12.1 User Visibility & Control

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-270 | "What ESIP Knows About Me" Page | The system SHALL provide a dedicated page in user settings titled "My Personalization Profile" showing: behavioral telemetry consent status per category, current working-pattern summary (top tasks, active hours, most-used features), competency maturity scores, memory store contents, active automations, and suggestion type opt-in status; all displayed in plain language. | UX | All | Must | GDPR Article 15 (right of access) requires users to be able to see data held about them; proactive UI is a best-practice beyond compliance. |
| REQ-PERS-271 | Per-Capability Opt-In/Out | The system SHALL allow users to independently enable or disable each personalization capability: behavioral telemetry, working-pattern modeling, automation suggestions, gap detection, next-best-action suggestions, AI memory, and peer intelligence; disabling a capability SHALL immediately halt data collection and hide related UI surfaces. | Security | All | Must | Granular consent is a GDPR/CCPA requirement and a user trust requirement; all-or-nothing opt-out is insufficient. |
| REQ-PERS-272 | Edit Learned Preferences | Users SHALL be able to directly edit any learned preference or behavioral model attribute displayed on the personalization profile page; edits SHALL be recorded as user-explicit corrections and SHALL override inferred values. | UX | All | Must | Inferred models contain errors; user correction is both a quality improvement and an ethical obligation. |
| REQ-PERS-273 | Right to Erasure (GDPR) | Users SHALL be able to trigger a full personalization data erasure from the settings page; erasure SHALL delete: all raw telemetry events, working-pattern model, memory store, automation history, and suggestion feedback signals within 30 days; the user SHALL receive email confirmation; erasure SHALL be irreversible with a double-confirm UI. | Security | All | Must | GDPR Article 17 obligation; 30-day SLA is a common industry standard for right-to-erasure. |
| REQ-PERS-274 | Audit Log of Automated Actions | The system SHALL maintain a full audit log of every AI-assisted or macro-executed action on architecture artifacts, with: timestamp, action type, artifact affected, macro/suggestion name, and outcome; the log SHALL be accessible to the user and exportable as CSV for 12 months. | Security | All | Must | Audit logs are required for accountability when automated actions modify regulated architecture artifacts. |
| REQ-PERS-275 | No Dark Patterns | The personalization opt-in/opt-out UI SHALL NOT use dark patterns: no pre-checked consent for optional telemetry, no confirmshaming language ("No thanks, I don't want to improve"), no buried opt-out paths (opt-out SHALL be reachable in ≤2 clicks from any personalization notification). | UX | All | Must | Dark patterns in consent UIs are specifically prohibited under GDPR and increasing regulatory scrutiny globally. |

### 12.2 Admin Policy Controls

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-280 | Tenant Admin Personalization Policy | Tenant admins SHALL be able to configure a tenant-wide personalization policy specifying: which telemetry categories are permitted, minimum opt-in requirements for organizational roles, data retention limits (overridable below platform max), and which personalization features are enabled for the tenant. | Governance | All | Must | Enterprise tenants have their own data governance policies that may be more restrictive than platform defaults. |
| REQ-PERS-281 | Admin View of Personalization Adoption | Tenant admins SHALL have a dashboard showing aggregate personalization adoption metrics: percentage of users with telemetry enabled per category, suggestion accept rates by persona, automation usage rates, top automations, average competency scores by persona; individual user data SHALL NOT be exposed in the admin dashboard. | Reporting | All | Should | Admins need aggregate visibility to measure personalization ROI without violating individual user privacy. |
| REQ-PERS-282 | Platform-Level Ethics Review Workflow | The platform operator SHALL maintain an ethics review checklist for any new suggestion type or automation capability added to the engine; the checklist SHALL cover: potential for manipulation, autonomy preservation, transparency compliance, and data minimization; checklist outcomes SHALL be logged. | Governance | All | Should | Proactive ethics review prevents inadvertent dark patterns as the suggestion engine expands over time. |
| REQ-PERS-283 | Explainability Audit Trail | For each AI-generated suggestion surfaced to a user, the system SHALL log the top-3 contributing signals (from the model's attribution layer) and retain this log for 90 days; the log SHALL be accessible to platform operators for model auditing and SHALL NOT contain user-identifiable content beyond user_id. | Security | All | Should | Explainability logging enables retrospective audits of model behavior, supporting GDPR Article 22 (automated decision-making transparency). |

---

## 13. Non-Functional Requirements

### 13.1 Performance & Responsiveness

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-300 | Suggestion Latency | In-context next-best-action suggestions SHALL be computed and rendered within 200 ms of the triggering event (workspace open, artifact save) for P95 of requests under normal load; background async suggestions (inbox, scheduled) SHALL update within 30 seconds. | NFR | All | Must | Suggestions that arrive after the user has moved on are worthless; 200 ms is the threshold below which latency is imperceptible. |
| REQ-PERS-301 | Dashboard Personalization Latency | The adaptive home dashboard SHALL render with personalized widget layout within 1 second of page load (LCP) for P95; the initial render SHALL use the last-known layout from cache and update in place if the model produces a new layout. | NFR | All | Must | Dashboard latency directly impacts first impression; progressive enhancement from cache prevents blank-screen waits. |
| REQ-PERS-302 | On-Device vs Server Inference | Lightweight inference tasks (suggestion ranking, widget reordering) SHALL be executable client-side using a pre-fetched ONNX/WASM model bundle (≤500 KB) to eliminate round-trip latency; heavy inference (pattern mining, AI memory retrieval) SHALL execute server-side; the split SHALL be configurable. | NFR | All | Should | Client-side inference for real-time UI decisions avoids network latency; server-side for expensive operations avoids client-side resource exhaustion. |
| REQ-PERS-303 | Cold-Start Handling | For users with no behavioral data (first session), all personalization features SHALL degrade gracefully to persona-role defaults within 500 ms; the user SHALL NOT see blank or empty personalization surfaces. | NFR | All | Must | Cold-start graceful degradation is a quality-of-life requirement; empty suggestion inboxes discourage engagement with the feature. |
| REQ-PERS-304 | Personalization Scale | The personalization service SHALL support at least 10,000 concurrent users per tenant instance without degradation; the working-pattern model computation SHALL be horizontally scalable via event-sourced architecture. | NFR | All | Must | Enterprise tenants may have large user populations; vertical scaling limits must not cap the personalization service. |

### 13.2 Data, Isolation & Security

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-310 | Per-User Model Isolation | Each user's working-pattern model, memory store, and automation library SHALL be stored in a tenant-isolated datastore with row-level security enforced at the storage layer (not application layer alone); cross-user queries SHALL be architecturally impossible except through the authorized aggregate pipeline. | Security | All | Must | Application-layer isolation is insufficient; storage-layer enforcement is required for enterprise security posture. |
| REQ-PERS-311 | Memory Store Encryption at Rest | The per-user memory store SHALL be encrypted at rest using AES-256; encryption keys SHALL be tenant-specific and managed via the platform key management service (V11); tenant key rotation SHALL re-encrypt all memory stores within 48 hours. | Security | All | Must | Memory stores contain confidential architecture decisions; encryption at rest is a baseline enterprise security requirement. |
| REQ-PERS-312 | Telemetry Pipeline Integrity | The behavioral telemetry pipeline SHALL be authenticated with per-session signed tokens; telemetry events SHALL include a HMAC integrity tag verifiable at ingestion; events failing integrity verification SHALL be rejected and logged. | Security | All | Should | Unsigned telemetry can be spoofed to manipulate personalization models; integrity verification prevents adversarial event injection. |
| REQ-PERS-313 | Anonymization of Peer Aggregates | Before peer pattern data is written to any aggregate store, the system SHALL apply differential privacy noise (epsilon ≤ 1.0) to count-based features; exact frequencies SHALL NOT be stored in the aggregate store. | Security | All | Should | Differential privacy provides mathematical guarantees against re-identification even from aggregate queries. |
| REQ-PERS-314 | Personalization Feature Flags | All personalization capabilities SHALL be individually deployable behind feature flags; tenant admins and platform operators SHALL be able to enable/disable any capability without a code deployment; flag changes SHALL take effect within 60 seconds. | NFR | All | Must | Feature flags enable safe progressive rollout, tenant-specific capability sets, and emergency kill-switches for misbehaving personalization components. |

---

## 14. Integration & Cross-Volume Dependencies

| ID | Title | Description | Category | Persona | Priority | Rationale |
|----|-------|-------------|----------|---------|----------|-----------|
| REQ-PERS-320 | V06 AI Infrastructure Integration | The personalization engine SHALL invoke V06 AI/RAG services for: memory store retrieval (semantic similarity ranking), AI memory context injection, AI tone adaptation, and semantic classification of workspace content for suggestion generation; calls SHALL use V06's defined agent/tool interfaces. | Integration | All | Must | Personalization is a consumer of V06 capabilities; no duplication of AI/RAG infrastructure. |
| REQ-PERS-321 | V11 Telemetry Pipeline Integration | The behavioral event stream (REQ-PERS-101) SHALL be published to the V11 telemetry pipeline via a defined event schema contract; personalization SHALL consume the processed event stream from V11's event store, not raw HTTP events directly; schema versioning SHALL be managed jointly between V11 and V14. | Integration | All | Must | V11 owns the telemetry infrastructure; V14 is a consumer; coupling at the schema boundary prevents breaking changes. |
| REQ-PERS-322 | V12 Workflow Engine Integration | When a personal automation (REQ-PERS-131) involves a cross-role action (e.g., triggering a review request, submitting a deliverable for approval), the automation SHALL invoke the V12 workflow engine APIs; personal macros SHALL NOT bypass V12 workflow rules. | Integration | All | Must | Automation that bypasses governance workflows would undermine the platform's collaboration and compliance model. |
| REQ-PERS-323 | V13 Activation Handoff | V13 SHALL pass the activation context (user_id, tenant_id, persona, locale) to V14's first-run wizard initialization endpoint via a signed JWT upon first login; V14 SHALL validate the token signature and expiry before launching the wizard. | Integration | All | Must | Secure handoff from V13 activation prevents wizard spoofing and ensures preference capture is tied to verified identity. |
| REQ-PERS-324 | V11 Identity for Consent Management | Consent records (REQ-PERS-100) SHALL be stored in the V11 identity/privacy service as the system of record for user consent; V14's personalization engine SHALL query V11 consent status before collecting any behavioral event, on every session start. | Integration | All | Must | Centralizing consent in V11 prevents consent-state inconsistency across services; V14 is a consent consumer, not a consent store. |
| REQ-PERS-325 | V08 Knowledge Hub for Learning Paths | Persona-specific learning path content (REQ-PERS-162) SHALL be sourced from V08 Knowledge Hub articles via the V08 search API; V14 SHALL NOT maintain a separate content store; learning path ordering SHALL be computed by V14's gap model applied to V08 content metadata. | Integration | All | Could | V08 is the content system of record; V14 adds the "which content, in what order, for this user" intelligence layer. |

---

## Appendix A — Summary Metrics

| Dimension | Count |
|-----------|-------|
| Total requirements | 106 |
| Must | 62 |
| Should | 33 |
| Could | 11 |
| Functional | 35 |
| AI/ML | 28 |
| UX | 22 |
| Security | 14 |
| NFR | 7 |
| Governance | 6 |
| Data | 5 |
| Integration | 5 |
| Reporting | 4 |

---

## Appendix B — Capability Area Index

| # | Capability Area | REQ Range | Count |
|---|----------------|-----------|-------|
| 1 | First-Run Wizard & Working-Mode Selection | REQ-PERS-001 – 032 | 16 |
| 2 | Personal Profile & Preference Model | REQ-PERS-040 – 053 | 9 |
| 3 | Adaptive Dashboards & Home Surface | REQ-PERS-060 – 072 | 10 |
| 4 | Behavioral Telemetry & Working-Pattern Model | REQ-PERS-100 – 115 | 16 |
| 5 | Repetition Detection & Personal Automation | REQ-PERS-120 – 137 | 18 |
| 6 | "What They Lack" — Guided Assistance & Growth | REQ-PERS-150 – 163 | 14 |
| 7 | Proactive Suggestions / Next-Best-Action Engine | REQ-PERS-170 – 178 | 9 |
| 8 | Personal AI Assistant Memory | REQ-PERS-200 – 213 | 14 |
| 9 | Process-Mode Engine | REQ-PERS-220 – 231 | 12 |
| 10 | Team & Peer Intelligence | REQ-PERS-240 – 244 | 5 |
| 11 | Feedback & Learning Loop | REQ-PERS-250 – 262 | 13 |
| 12 | Transparency, User Control & Ethics | REQ-PERS-270 – 283 | 14 |
| 13 | Non-Functional Requirements | REQ-PERS-300 – 314 | 15 |
| 14 | Integration & Cross-Volume Dependencies | REQ-PERS-320 – 325 | 6 |

---

## Appendix C — Boundary Decisions

| Topic | Decision |
|-------|----------|
| AI model hosting, RAG pipelines, vector store | V06 — V14 is a consumer only |
| Identity, SSO, session management | V11 — V14 uses V11 identity APIs |
| Raw telemetry event pipeline, Kafka/queue infra | V11 — V14 publishes events; V11 routes and stores |
| Privacy infrastructure (DPA, consent backend) | V11 — V14 queries V11 consent status |
| Cross-role request/approval workflow engine | V12 — personal automations invoke V12 for cross-role actions |
| Tenant provisioning, license management | V13 — V14 wizard receives signed handoff from V13 |
| Activation email / link | V13 — V14 wizard launches post-link-click |
| Persona definitions, metamodel | V05/V11 — V14 reads persona metadata, does not define it |
| Team-level deliverable templates (non-personal) | V08 Knowledge Hub — V14 references but does not host |
