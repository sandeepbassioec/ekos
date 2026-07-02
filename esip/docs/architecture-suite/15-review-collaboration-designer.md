# Volume 15 — Native Review & Collaboration Designer

**Prefix:** REV  
**Status:** Draft v1.0  
**Date:** 2026-06-23  
**Scope:** Review request & reviewer selection; native inline comment designer (documents + diagrams); review lifecycle & versioning; feedback incorporation & AI-apply; audit & history; access & Reviewer licensing; AI-augmented review (security, NFR, topic navigator, diagram generation, summaries, checklists); cross-team scoped dependency sharing.  
**Out of scope (boundaries):** Approval gates/routing/SLA state-machine (→ V12); AI model/RAG infrastructure (→ V06); maxGraph diagram canvas rendering (→ V03); identity, RBAC, sharing permissions infrastructure (→ V11); deliverable generation (→ V10); onboarding/license provisioning flows (→ V13).

---

## Table of Contents

1. [REV-01 — Review Request & Reviewer Selection](#rev-01--review-request--reviewer-selection)
2. [REV-02 — Native Review Designer: Documents](#rev-02--native-review-designer-documents)
3. [REV-03 — Native Review Designer: Diagrams & Architecture Models](#rev-03--native-review-designer-diagrams--architecture-models)
4. [REV-04 — Review Lifecycle & Versioning](#rev-04--review-lifecycle--versioning)
5. [REV-05 — Feedback Incorporation & AI-Apply](#rev-05--feedback-incorporation--ai-apply)
6. [REV-06 — Audit & History](#rev-06--audit--history)
7. [REV-07 — Access & Reviewer Licensing](#rev-07--access--reviewer-licensing)
8. [REV-08 — AI-Augmented Review](#rev-08--ai-augmented-review)
9. [REV-09 — Cross-Team Scoped Dependency Sharing](#rev-09--cross-team-scoped-dependency-sharing)

---

## REV-01 — Review Request & Reviewer Selection

### REV-01-01 Universal Artifact Submission

| Field | Value |
|---|---|
| **ID** | REQ-REV-001 |
| **Title** | Universal Review Submission for Any Artifact Type |
| **Description** | The system SHALL allow any author to submit any ESIP-managed artifact — including BRDs, Enterprise Architecture documents, Solution Architecture documents, Integration Architecture documents, Application Architecture documents, Data Architecture documents, Security Architecture documents, HLDs, LLDs, diagrams, roadmaps, requirement sets, and composite deliverable packages — for native peer review from within the ESIP platform without requiring export to an external tool. |
| **Category** | Functional |
| **Persona(s)** | Enterprise Architect, Solution Architect, Application Architect, Integration Architect, API Architect, CRM Architect, Data Architect, Security Architect, Cloud Architect, Infra Architect, Microservices Architect, Product Manager, **Reviewer** |
| **Priority** | Must Have |
| **Rationale/Source** | Core platform differentiator; zero-friction submission drives review adoption across all delivery artifacts. |
| **Acceptance Criteria** | - "Submit for Review" action is available on every artifact type's detail view.<br>- Author can submit a single artifact or a composite package (e.g., BRD + HLD + diagrams together).<br>- Submission creates a review record linked to the artifact version at time of submission.<br>- Submission is rejected with a clear error if artifact is in a non-reviewable state (e.g., deleted). |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-002 |
| **Title** | Artifact Version Snapshot on Submission |
| **Description** | The system SHALL capture an immutable snapshot of the artifact's content (document body, diagram model, requirement set) at the exact moment of review submission, so that reviewers always see the version the author submitted regardless of subsequent author edits. |
| **Category** | Functional |
| **Persona(s)** | All Architects, Product Manager, **Reviewer** |
| **Priority** | Must Have |
| **Rationale/Source** | Prevents reviewer confusion from live edits; aligns with Azure DevOps PR "base commit" model. |
| **Acceptance Criteria** | - Snapshot ID and timestamp are stored and displayed to reviewers.<br>- Author edits post-submission do not alter the reviewers' view unless a new revision is explicitly submitted.<br>- Snapshot is linked to the review record in the audit trail. |
| **Dependencies** | V10 (deliverable versioning), V03 (diagram model snapshot) |

---

### REV-01-02 Reviewer Selection & Assignment

| Field | Value |
|---|---|
| **ID** | REQ-REV-003 |
| **Title** | Reviewer Search by Name, Role, or Team |
| **Description** | The system SHALL provide a reviewer picker that allows the author to search and select reviewers by display name, email, persona/role (e.g., Security Architect, Product Manager), or team/group, and assign each reviewer as **required** or **optional**. |
| **Category** | Functional |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Mirrors Azure DevOps PR reviewer assignment; supports compliance requirements for mandatory sign-off. |
| **Acceptance Criteria** | - Picker supports fuzzy search across name, email, and role.<br>- Author can label each reviewer Required or Optional before submitting.<br>- Author can assign one reviewer as **primary reviewer** (owns merge/approval decision, per V12 SLA).<br>- Required/Optional distinction is visually differentiated in the review UI.<br>- Author can remove or swap a reviewer before the review starts. |
| **Dependencies** | V11 (identity directory, RBAC) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-004 |
| **Title** | AI-Recommended Reviewer Suggestions |
| **Description** | The system SHALL surface AI-generated reviewer suggestions based on the artifact type, domain tags, past review participation, team ownership, and declared dependencies, so the author can add relevant stakeholders they may have overlooked. |
| **Category** | AI/ML |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Should Have |
| **Rationale/Source** | Reduces missed stakeholder reviews; improves architectural coverage across personas. |
| **Acceptance Criteria** | - Suggestions panel shows ranked candidates with reason (e.g., "Security Architect — this artifact has security zone changes").<br>- Author can accept a suggestion with one click.<br>- Suggestions are dismissible without affecting the submission. |
| **Dependencies** | V06 (AI model), V11 (identity/team graph) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-005 |
| **Title** | Review Due Date Setting |
| **Description** | The system SHALL allow the author to set a target due date for the review at submission time; this date SHALL be surfaced to all reviewers in the review header and in notification messages. |
| **Category** | Functional |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Due dates drive reviewer urgency without requiring SLA enforcement logic (which is V12). |
| **Acceptance Criteria** | - Date picker is available on the submission form.<br>- Due date is displayed prominently in the review UI for all participants.<br>- Passing of the due date does NOT auto-close the review (SLA enforcement → V12).<br>- Reviewers receive a reminder notification as due date approaches (notification delivery → V12/V11). |
| **Dependencies** | V12 (SLA engine — boundary: this requirement covers only UI display and due-date storage) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-006 |
| **Title** | Reviewer Notification on Assignment |
| **Description** | The system SHALL notify each assigned reviewer immediately upon review submission via in-app notification and (optionally) email, including a direct deep-link to the review, artifact name, author, due date, and their Required/Optional status. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer** |
| **Priority** | Must Have |
| **Rationale/Source** | Timely notification reduces review latency; deep-links reduce friction. |
| **Acceptance Criteria** | - Notification is delivered within 30 seconds of submission for in-app channel.<br>- Notification body includes: artifact name, type, author, due date, Required/Optional, and a deep-link URL.<br>- Reviewer can access the review directly from the link without landing on a home page first. |
| **Dependencies** | V11 (notification infrastructure) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-007 |
| **Title** | Review Description & Context Note |
| **Description** | The system SHALL allow the author to add a free-text description/context note when submitting a review, giving reviewers guidance on what to focus on, known risks, open questions, and scope boundaries. |
| **Category** | Functional |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Reduces reviewer onboarding time; sets review expectations upfront. |
| **Acceptance Criteria** | - Rich-text editor (bold, bullets, links, @mentions) is available on the submission form.<br>- Context note is displayed at the top of the review thread for all reviewers.<br>- @mentions in the note trigger notifications to the mentioned users. |

---

### REV-01-03 Review Cancellation & Reassignment

| Field | Value |
|---|---|
| **ID** | REQ-REV-008 |
| **Title** | Reviewer Reassignment & Addition Mid-Review |
| **Description** | The system SHALL allow the author (and platform administrators) to add new reviewers or reassign an existing reviewer to another person while the review is in progress, with all changes recorded in the audit trail. |
| **Category** | Functional |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Should Have |
| **Rationale/Source** | Reviewer availability changes; team structures shift mid-project. |
| **Acceptance Criteria** | - Add/remove reviewer action is available to the author while review is In-Review or Changes-Requested.<br>- Newly added reviewer receives notification and sees the full comment history.<br>- Removed reviewer's prior comments and decisions are preserved and attributed. |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-009 |
| **Title** | Review Cancellation by Author |
| **Description** | The system SHALL allow the author to cancel an in-progress review, providing a cancellation reason; all reviewers SHALL be notified and the review SHALL move to a Cancelled terminal state, preserving all existing comments in read-only mode. |
| **Category** | Functional |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Artifacts get superseded; reviews must be closeable without blocking the author. |
| **Acceptance Criteria** | - Cancellation requires a mandatory reason text (min 10 chars).<br>- Reviewers receive a cancellation notification with the reason.<br>- Cancelled review is visible in review history in read-only mode. |

---

## REV-02 — Native Review Designer: Documents

### REV-02-01 Inline Comment Anchoring

| Field | Value |
|---|---|
| **ID** | REQ-REV-010 |
| **Title** | Inline Comment Anchored to Section |
| **Description** | The system SHALL allow reviewers to place inline comments anchored to a named document section (heading level 1–4), so that comments are always associated with the correct part of the document even when content is reordered. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Section-anchored comments survive minor document reflow; mirrors Google Docs suggesting mode and Azure DevOps file-level comments. |
| **Acceptance Criteria** | - Clicking a paragraph or heading shows an "Add Comment" affordance (margin icon or highlight).<br>- Comment is stored with the anchor: section-ID + character offset range.<br>- If the section is removed in a subsequent revision, the comment is flagged as "Orphaned — original section no longer present" and shown in an orphan panel. |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-011 |
| **Title** | Inline Comment Anchored to Paragraph |
| **Description** | The system SHALL allow reviewers to select any contiguous text range within a paragraph and anchor a comment to that precise text span, with the selected text highlighted in the document view for all participants. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Paragraph-level precision reduces ambiguity vs. section-level only; enables pinpoint feedback on exact wording. |
| **Acceptance Criteria** | - Text selection triggers an inline comment affordance.<br>- Selected text is highlighted with a color-coded underline/background per comment state.<br>- Multiple overlapping comment spans are visually distinguishable. |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-012 |
| **Title** | Inline Comment Anchored to Table Row |
| **Description** | The system SHALL allow reviewers to anchor a comment to a specific table row (or individual cell) within document tables, including requirement tables, so that the comment is clearly associated with that row when the table is rendered. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Requirement sets and architecture matrices are table-heavy; row-level comments are essential for precise feedback. |
| **Acceptance Criteria** | - Row hover or row selector shows "Add Comment" affordance.<br>- Comment panel displays the originating table row context (e.g., row number + first-cell preview).<br>- Row comments survive row reordering in the same revision. |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-013 |
| **Title** | Inline Comment Anchored to Requirement ID |
| **Description** | The system SHALL allow reviewers to anchor comments directly to a requirement ID (e.g., REQ-BRD-042) within requirement sets, so that the comment is navigable by requirement ID independent of document position. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, Enterprise Architect, Solution Architect, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Requirement-ID anchoring enables downstream traceability from review comment to requirement change; critical for regulated delivery. |
| **Acceptance Criteria** | - Requirement-ID-anchored comments are displayed in a dedicated "Comments on REQ-XXX" filter.<br>- Comment survives the requirement being moved to a different section.<br>- Requirement-level comment count is shown inline next to each requirement ID in the designer. |

---

### REV-02-02 Threaded Replies & Comment States

| Field | Value |
|---|---|
| **ID** | REQ-REV-014 |
| **Title** | Threaded Reply Chains on Inline Comments |
| **Description** | The system SHALL support threaded replies on any inline comment, allowing multiple participants to engage in a nested discussion attached to the same anchor, with each reply attributed to its author with a timestamp. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Threaded discussions prevent comment sprawl; mirrors PR review comment threads in GitHub/Azure DevOps. |
| **Acceptance Criteria** | - Reply button is present on every comment and reply.<br>- Thread depth is unlimited but visually indented to max 3 levels before collapsing into a "View more" control.<br>- Thread is collapsible; collapsed state shows reply count and last-reply author + timestamp. |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-015 |
| **Title** | Comment State Machine: Active → Resolved → Reopened |
| **Description** | The system SHALL implement a per-comment state machine with states: **Active** (default), **Resolved** (marked by author or reviewer), and **Reopened** (by any participant); resolved comments SHALL be visually de-emphasized but remain in the document margin; state transitions SHALL be recorded in the comment audit trail. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Comment lifecycle management prevents review clutter; aligned with GitHub PR resolve/reopen model. |
| **Acceptance Criteria** | - "Resolve" button is available on any Active comment to all review participants.<br>- "Reopen" button is available on Resolved comments to any participant.<br>- Resolved comments collapse in the margin but are accessible via "Show resolved".<br>- Filter panel allows filtering by state (All / Active / Resolved / Reopened). |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-016 |
| **Title** | Comment Severity / Type Labels |
| **Description** | The system SHALL allow reviewers to tag each comment with a type/severity label: **Blocker**, **Major**, **Minor**, **Question**, **Suggestion**, **FYI**; labels SHALL be filterable in the comment panel and SHALL influence the AI review summary. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, All Architects, Product Manager |
| **Priority** | Should Have |
| **Rationale/Source** | Severity labels allow the author to triage feedback efficiently; blockers vs. suggestions have very different resolution urgency. |
| **Acceptance Criteria** | - Label dropdown is available when creating or editing a comment.<br>- Comment panel provides a filter/group-by label view.<br>- Blocker-labeled comments must be resolved before the AI completeness score reports 100%. |

---

### REV-02-03 @Mentions, Rich Content & Suggestions

| Field | Value |
|---|---|
| **ID** | REQ-REV-017 |
| **Title** | @Mention in Comments with Notification |
| **Description** | The system SHALL support @mention syntax within comment bodies and replies; typing "@" SHALL trigger an autocomplete picker of platform users; mentioned users SHALL receive an in-app notification with a deep-link to the comment, even if they are not assigned reviewers. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | @mentions route feedback to subject-matter experts on demand without pre-assigning them as reviewers. |
| **Acceptance Criteria** | - @mention picker shows display name, role, and avatar.<br>- Mentioned user receives notification within 30 seconds.<br>- @mentions render as clickable user chips in the comment body.<br>- Users mentioned in a comment gain read access to that artifact for the duration of the review (permissioned via V11). |
| **Dependencies** | V11 (identity, permissions) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-018 |
| **Title** | Rich Text & Code Snippet Support in Comments |
| **Description** | The system SHALL support rich-text comment authoring including bold, italic, bulleted/numbered lists, hyperlinks, inline code, fenced code blocks with syntax highlighting, and inline image/attachment upload; the comment composer SHALL render a live preview. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Architecture reviews routinely include code snippets, API signatures, and annotated screenshots; rich text prevents comment truncation. |
| **Acceptance Criteria** | - Code blocks support at minimum: JSON, YAML, C#, Rust, SQL, TypeScript, XML syntax highlighting.<br>- Attachments: PNG, JPEG, PDF, DOCX, XLSX, up to 25 MB per file.<br>- Live preview toggle is available in the composer. |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-019 |
| **Title** | Suggestion Edits — Inline Text Change Proposals |
| **Description** | The system SHALL allow reviewers to propose a specific text change (suggestion edit) directly within a document, showing the proposed replacement inline in a diff-style view; the author SHALL be able to **Accept** (apply the suggestion) or **Reject** (dismiss it) with one click; accepting a suggestion SHALL automatically resolve the originating comment. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Suggestion edits are the #1 efficiency driver in Google Docs review; eliminates manual copy-paste of reviewer-proposed text. |
| **Acceptance Criteria** | - "Suggest Edit" mode is a toggle in the comment composer, available for paragraph-anchored comments.<br>- Suggested text renders with deletion in red strikethrough and addition in green underline inline in the document.<br>- Accept/Reject controls are on the suggestion comment card.<br>- Accepted suggestions are tracked: the resulting document diff references the source suggestion comment ID.<br>- Rejected suggestions show a "Rejected" badge and reason (optional). |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-020 |
| **Title** | Bulk Comment Actions |
| **Description** | The system SHALL allow the author and authorized reviewers to perform bulk actions on selected comments: bulk resolve, bulk reopen, bulk assign label, so that high-volume reviews (50+ comments) remain manageable. |
| **Category** | Functional |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Should Have |
| **Rationale/Source** | Large reviews can accumulate 100+ comments; bulk actions reduce resolution time. |
| **Acceptance Criteria** | - Multi-select checkboxes appear on comments in the comment panel list view.<br>- Bulk actions toolbar appears when ≥2 comments are selected.<br>- Bulk resolve requires a single confirmation prompt and records the resolution in each comment's audit trail. |

---

## REV-03 — Native Review Designer: Diagrams & Architecture Models

### REV-03-01 Canvas-Anchored Comment on Diagram Elements

| Field | Value |
|---|---|
| **ID** | REQ-REV-021 |
| **Title** | Comment Anchored to Specific Diagram Element |
| **Description** | The system SHALL allow reviewers to place a comment anchored to a specific element (node, shape, container, swimlane) on a maxGraph-rendered diagram; the comment pin SHALL be rendered on the canvas at the element's position and SHALL follow element position if the diagram is rearranged. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, Enterprise Architect, Solution Architect, Integration Architect, Cloud Architect, Infra Architect, Microservices Architect, Security Architect |
| **Priority** | Must Have |
| **Rationale/Source** | Screenshot-based review loses element identity when diagram changes; model-anchored comments survive layout changes and are precise. |
| **Acceptance Criteria** | - Right-click on any diagram element opens "Add Comment" in the review sidebar.<br>- Comment pin (numbered balloon) is overlaid on the element in the canvas.<br>- If the element is deleted in a subsequent revision, the comment is marked Orphaned and shown in the orphan panel.<br>- Comment panel entry shows the element's ID, label, and type as context. |
| **Dependencies** | V03 (maxGraph canvas, element ID model) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-022 |
| **Title** | Comment Anchored to Diagram Relationship/Edge |
| **Description** | The system SHALL allow reviewers to anchor a comment to a specific relationship or edge between two elements on a diagram (e.g., a data flow, dependency arrow, integration link); the comment pin SHALL render near the midpoint of the edge. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, Integration Architect, API Architect, Data Architect, Microservices Architect |
| **Priority** | Must Have |
| **Rationale/Source** | Relationships in architecture diagrams carry critical semantic meaning (protocols, data contracts, trust boundaries); edge-level comments are essential for integration and API review. |
| **Acceptance Criteria** | - Edge hover shows "Add Comment" affordance on the canvas.<br>- Comment is stored with the source-element-ID + target-element-ID pair + edge-ID for precise re-anchoring.<br>- Edge comment survives edge rerouting (same source/target). |
| **Dependencies** | V03 (maxGraph edge model) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-023 |
| **Title** | Comment Anchored to Diagram View / Viewport Region |
| **Description** | The system SHALL allow reviewers to anchor a comment to a rectangular region of the diagram canvas (a free-form area selection), for cases where the concern spans multiple elements or an empty area of the diagram. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, All Architects |
| **Priority** | Should Have |
| **Rationale/Source** | Cross-element concerns (e.g., "This entire zone is missing HA design") cannot be anchored to a single element. |
| **Acceptance Criteria** | - Rubber-band selection mode is activatable from the review toolbar.<br>- Selected region is stored as canvas coordinates relative to a named diagram layer/view.<br>- Region pins are visually distinct from element pins. |
| **Dependencies** | V03 (canvas coordinate model) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-024 |
| **Title** | Element-Level Discussion Panel |
| **Description** | The system SHALL provide a dedicated element discussion panel that, when an element is selected on the canvas, shows all comments (threaded) anchored to that element, sorted by creation time, with full reply/resolve/reopen functionality inline. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, All Architects |
| **Priority** | Must Have |
| **Rationale/Source** | Contextual element panel is the UX pattern that makes canvas-level review usable — prevents context switching to a separate comment list. |
| **Acceptance Criteria** | - Panel slides in from the right on element selection (non-modal).<br>- Panel shows comment count badge on element when closed.<br>- Panel is keyboard-navigable (Esc to close, Tab to cycle comments). |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-025 |
| **Title** | Review on the Model, Not a Screenshot |
| **Description** | The system SHALL ensure that all diagram review interactions (commenting, navigating, diffing) operate directly on the live maxGraph model, not on a rasterized screenshot or PDF export, so that element IDs, relationships, and metadata are always queryable. |
| **Category** | NFR |
| **Persona(s)** | All Architects, **Reviewer** |
| **Priority** | Must Have |
| **Rationale/Source** | Screenshot-based review is a known anti-pattern; it destroys model semantics and prevents element-level traceability. |
| **Acceptance Criteria** | - No review workflow step requires exporting or screenshot-capturing the diagram.<br>- The review designer embeds the full maxGraph canvas in read-only (reviewer) or annotate mode.<br>- Model API calls during review return structured element data, not image data. |
| **Dependencies** | V03 (maxGraph embed/API) |

---

### REV-03-02 Multi-View Diagram Review

| Field | Value |
|---|---|
| **ID** | REQ-REV-026 |
| **Title** | Multi-View Navigation Within Diagram Review |
| **Description** | The system SHALL allow reviewers to navigate between multiple views/layers of a composite diagram (e.g., C4 Context → Container → Component levels; or different architecture views within a single model) within the review designer without leaving the review context. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, Enterprise Architect, Solution Architect, Cloud Architect |
| **Priority** | Must Have |
| **Rationale/Source** | Complex architecture models have multiple views; requiring separate review submissions per view fragments the review context. |
| **Acceptance Criteria** | - View selector (tab or breadcrumb) is present in the review designer when the model has multiple views.<br>- Comments are view-scoped but accessible from a unified "All Comments" panel filtered by view.<br>- Switching views does not lose the current comment selection. |
| **Dependencies** | V03 (multi-view model) |

---

## REV-04 — Review Lifecycle & Versioning

### REV-04-01 Review States

| Field | Value |
|---|---|
| **ID** | REQ-REV-027 |
| **Title** | Review State Machine: Draft → In-Review → Changes-Requested → Approved |
| **Description** | The system SHALL implement a review-level state machine with states: **Draft** (author preparing), **In-Review** (active review by reviewers), **Changes-Requested** (≥1 required reviewer requested changes), **Approved** (all required reviewers approved), and **Cancelled**; transitions SHALL be rule-governed and SHALL feed the approval gate in V12. |
| **Category** | Functional |
| **Persona(s)** | All Architects, Product Manager, **Reviewer** |
| **Priority** | Must Have |
| **Rationale/Source** | Explicit states give the author clear status awareness; states feed V12 approval gates. |
| **Acceptance Criteria** | - State is displayed prominently in the review header.<br>- Draft → In-Review triggered by author clicking "Submit for Review" (REQ-REV-001).<br>- In-Review → Changes-Requested triggered when any required reviewer submits a "Request Changes" decision.<br>- Changes-Requested → In-Review triggered when author submits a revised version.<br>- In-Review → Approved triggered when all required reviewers submit "Approve".<br>- State transition events are published to V12 for approval gate evaluation. |
| **Dependencies** | V12 (approval gate consumer) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-028 |
| **Title** | Reviewer Decision: Approve / Request Changes / Comment Only |
| **Description** | The system SHALL allow each reviewer to submit one of three decisions at the close of their review: **Approve**, **Request Changes**, or **Comment Only** (no blocking vote); the decision SHALL be accompanied by an optional summary comment and SHALL be updateable before the review is closed. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer** |
| **Priority** | Must Have |
| **Rationale/Source** | Three-way decision matches standard PR review UX (GitHub/Azure DevOps) and supports nuanced reviewer positions. |
| **Acceptance Criteria** | - Decision panel is available in the review header for each reviewer.<br>- Decision + optional summary text is submitted atomically.<br>- Reviewer can update their decision at any time while the review is In-Review or Changes-Requested.<br>- Decision history (all decisions per reviewer) is shown in the review timeline. |

---

### REV-04-02 Revision Iterations

| Field | Value |
|---|---|
| **ID** | REQ-REV-029 |
| **Title** | Author Revision Submission |
| **Description** | The system SHALL allow the author to submit a revised version of the artifact while the review is in Changes-Requested or In-Review state; the revision SHALL create a new snapshot, increment a revision counter (R1, R2, …), and notify all reviewers that a new revision is ready. |
| **Category** | Functional |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Multi-iteration reviews are the norm for complex artifacts; revision management is fundamental. |
| **Acceptance Criteria** | - "Submit Revision" action is available to the author while review is In-Review or Changes-Requested.<br>- Author can include a revision summary (what changed) in the revision submission form.<br>- Previous revisions remain accessible in the revision history panel.<br>- Comments from prior revisions carry forward but are marked with their originating revision number. |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-030 |
| **Title** | Version Compare: Document Diff Side-by-Side |
| **Description** | The system SHALL provide a side-by-side document diff view comparing any two revisions of a reviewed document, highlighting added, removed, and modified paragraphs, table rows, and requirement fields with color-coded annotations. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Reviewers on subsequent revisions need to quickly see what changed without re-reading the entire document. |
| **Acceptance Criteria** | - Diff viewer is accessible from the revision history panel (select any two revisions to compare).<br>- Additions rendered in green, deletions in red, unchanged content in neutral.<br>- Paragraph-level and table-row-level granularity minimum.<br>- Diff view includes comment count delta ("3 new comments since R1"). |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-031 |
| **Title** | Version Compare: Diagram Model Diff |
| **Description** | The system SHALL provide a diagram model diff view between any two revisions of a reviewed diagram, visually indicating added elements (green overlay), removed elements (red overlay), modified elements (yellow overlay), added/removed relationships, and changed property values on a shared canvas. |
| **Category** | Functional |
| **Persona(s)** | **Reviewer**, Enterprise Architect, Solution Architect, Integration Architect, Cloud Architect |
| **Priority** | Must Have |
| **Rationale/Source** | Diagram diffs are the hardest problem in architecture tool review; visual overlay on the model is the gold standard. |
| **Acceptance Criteria** | - Diff view compares element IDs across revisions; element moves are detected (same ID, different position).<br>- Changed properties (label, stereotype, color, metadata) are listed in a side panel when a changed element is selected.<br>- Unchanged elements are rendered at 30% opacity to visually foreground changes.<br>- Diff view is read-only; comments can still be added to diff-highlighted elements. |
| **Dependencies** | V03 (maxGraph model diff API) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-032 |
| **Title** | Re-Review Request on Revision |
| **Description** | The system SHALL allow the author to explicitly request a re-review from specific reviewers when submitting a revision, resetting those reviewers' decisions to pending while preserving comment history from prior rounds. |
| **Category** | Functional |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Authors need selective re-review when only certain areas changed; blanket re-review wastes reviewer time. |
| **Acceptance Criteria** | - "Request re-review from" picker pre-selects reviewers who had "Request Changes" decisions.<br>- Reviewer receives a re-review notification referencing the revision number and the revision summary.<br>- Prior reviewer decisions that are NOT re-requested remain as-is and are flagged "Decision from R{n}" in the review header. |

---

## REV-05 — Feedback Incorporation & AI-Apply

### REV-05-01 Manual Resolution

| Field | Value |
|---|---|
| **ID** | REQ-REV-033 |
| **Title** | Manual Comment Resolution by Author |
| **Description** | The system SHALL allow the author to resolve any Active comment manually by marking it Resolved, optionally adding a resolution note explaining what change was made in response; the resolution note SHALL appear inline in the comment thread. |
| **Category** | Functional |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Manual resolution is the baseline feedback incorporation mechanism; resolution notes provide traceability. |
| **Acceptance Criteria** | - "Resolve" action is available to the artifact author on any Active comment.<br>- Resolution note is optional but surfaced as a prompt ("What change did you make?").<br>- Resolved comment shows author name, timestamp, and resolution note in the thread. |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-034 |
| **Title** | Comment-to-Change Traceability |
| **Description** | The system SHALL track which document or diagram changes (edits, element additions/deletions, requirement updates) were made in response to which review comments, creating a bidirectional traceability link: comment → change and change → comment. |
| **Category** | Functional |
| **Persona(s)** | All Architects, Product Manager, **Reviewer** |
| **Priority** | Must Have |
| **Rationale/Source** | Audit requirements in enterprise architecture governance demand proof that feedback was acted upon; traceability is non-negotiable. |
| **Acceptance Criteria** | - When an author accepts a suggestion edit (REQ-REV-019), the change record references the comment ID automatically.<br>- For manual edits, the author can manually link a comment to the change set from the review UI.<br>- Traceability matrix is exportable as a report (comment ID, author, description, linked change, change timestamp). |

---

### REV-05-02 AI-Apply

> **Reconciliation note — Approval-gate interaction (C4; cross-ref REQ-WFL-055).** AI-apply (REQ-REV-035 / REQ-REV-036 / REQ-REV-037) MUST NOT silently mutate an artifact that is under an active V12 approval gate. When the target artifact has an active V12 approval gate, AI-apply MUST either (a) **block** the apply with a clear explanation that the artifact is gated, or (b) **create a new revision** (REQ-REV-029) and **reset the V12 approval** so the gate must be re-satisfied. This applies to all single and batch AI-apply paths. Human-in-the-loop preview (REQ-REV-038) still applies in all cases.

| Field | Value |
|---|---|
| **ID** | REQ-REV-035 |
| **Title** | AI-Apply: Auto-Suggest Change When Comment Provides Sufficient Information |
| **Description** | The system SHALL analyze Active comments on document artifacts and, when the comment contains sufficient information to derive a specific change (e.g., "Replace 'eventual consistency' with 'strong consistency' in section 3.2"), SHALL generate a suggested document edit and present it to the author as a diff preview before any modification is applied. |
| **Category** | AI/ML |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | AI-apply is the headline efficiency gain for the review loop; eliminates manual transcription of reviewer-suggested text. |
| **Acceptance Criteria** | - "AI Suggest Change" button appears on comments where the AI detects sufficient information to generate a change.<br>- AI generates a diff-style preview of the proposed document change.<br>- Author can Accept (apply), Edit (modify before applying), or Reject.<br>- No change is applied without explicit author confirmation.<br>- AI confidence indicator is shown (High / Medium / Low); Low-confidence suggestions include a disclaimer.<br>- **Approval-gate guard (C4; cross-ref REQ-WFL-055):** if the target artifact has an active V12 approval gate, AI-apply MUST either **block** the apply (with an explanation that the artifact is under an active approval gate) **or** **create a new revision (REQ-REV-029) and reset the V12 approval** so the gate is re-evaluated; AI-apply MUST NOT mutate a gated artifact in place without one of these two outcomes. |
| **Dependencies** | V06 (AI model), V10 (document edit API), **V12 (approval gate — AI-apply on a gated artifact must block or create-revision-and-reset per REQ-WFL-055)** |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-036 |
| **Title** | AI-Apply: Diagram Element Change Suggestion |
| **Description** | The system SHALL analyze Active comments anchored to diagram elements and, when the comment describes a specific change to a diagram element or relationship (e.g., "Add a firewall between Service A and the DB layer"), SHALL generate a diagram change proposal as a model diff preview for author confirmation. |
| **Category** | AI/ML |
| **Persona(s)** | All Architects |
| **Priority** | Should Have |
| **Rationale/Source** | Diagram change suggestions accelerate complex architecture iteration; manual diagram editing is the biggest time cost in architecture review cycles. |
| **Acceptance Criteria** | - Diagram change proposal is previewed on the canvas using the same diff overlay as REQ-REV-031.<br>- Author can Accept, Edit (open in full canvas editor with pre-staged change), or Reject.<br>- Accepted diagram changes auto-resolve the originating comment and create a comment-to-change traceability link. |
| **Dependencies** | V06 (AI model), V03 (diagram edit API) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-037 |
| **Title** | Batch AI-Apply Across All Actionable Comments |
| **Description** | The system SHALL provide a "Review AI Suggestions" mode that collects all AI-generated change suggestions across an entire review in a single queue, allowing the author to step through each suggestion and Accept/Reject in sequence, with a final confirmation step before batch application. |
| **Category** | AI/ML |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Should Have |
| **Rationale/Source** | Batch review of AI suggestions is significantly faster than reviewing each comment individually; critical for 50+ comment reviews. |
| **Acceptance Criteria** | - "AI Review Mode" button is available in the review toolbar when ≥3 AI suggestions exist.<br>- Step-through UI shows: comment context, proposed change diff, confidence level, Accept/Reject/Skip.<br>- Final confirmation lists all accepted changes before applying.<br>- Skipped suggestions remain available for later individual review. |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-038 |
| **Title** | AI-Apply Change Preview Before Commit |
| **Description** | The system SHALL ensure that ALL AI-generated change suggestions — single or batch — are previewed as a diff before any persistent change is written to the artifact; no AI-driven modification SHALL be applied without an explicit human confirmation action. |
| **Category** | AI/ML |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Human-in-the-loop is a non-negotiable safety principle for AI-modifying enterprise architecture documents; avoids silent corruption of authoritative artifacts. |
| **Acceptance Criteria** | - Preview is shown for 100% of AI-apply actions, no exceptions.<br>- The confirmation button is labeled with the action type ("Apply 1 change" / "Apply {n} changes").<br>- Preview is cancellable without side effects at any point before the final confirm. |

---

## REV-06 — Audit & History

### REV-06-01 Immutable Audit Trail

| Field | Value |
|---|---|
| **ID** | REQ-REV-039 |
| **Title** | Immutable Audit Trail for All Review Events |
| **Description** | The system SHALL record an immutable, append-only audit trail for every review event, including: submission, reviewer assignment/removal, comment creation/edit/deletion/resolution/reopen, suggestion accept/reject, AI-apply accept/reject, reviewer decisions, revision submissions, state transitions, and cancellation; each record SHALL include actor identity, timestamp (UTC), and event payload. |
| **Category** | Governance |
| **Persona(s)** | All Architects, Product Manager, **Reviewer** |
| **Priority** | Must Have |
| **Rationale/Source** | Enterprise governance frameworks (ISO 27001, TOGAF compliance, SOC 2) require evidence of who reviewed what and when; immutability prevents tampering. |
| **Acceptance Criteria** | - Audit trail is stored separately from the mutable artifact store; no update/delete operations are permitted on audit records.<br>- Audit records are visible to authorized users in a review timeline view.<br>- All timestamps are in UTC ISO-8601 format. |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-040 |
| **Title** | Review Timeline View |
| **Description** | The system SHALL provide a chronological review timeline view showing all events for a given review: who did what and when, with comment excerpts, decision summaries, and revision milestones displayed as a vertical activity feed. |
| **Category** | Functional |
| **Persona(s)** | All Architects, Product Manager, **Reviewer** |
| **Priority** | Must Have |
| **Rationale/Source** | Timeline gives stakeholders and audit consumers a narrative view of the entire review history without trawling individual comments. |
| **Acceptance Criteria** | - Timeline is paginated (20 events per page) with infinite scroll.<br>- Events are grouped by revision (R0, R1, R2…) with clear dividers.<br>- Clicking any timeline event deep-links to the corresponding comment or change in the designer. |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-041 |
| **Title** | Audit Trail Export |
| **Description** | The system SHALL allow authorized users to export the full audit trail for any review as a structured JSON file and as a human-readable PDF/HTML report, including all events, comment threads, decisions, and AI-apply actions. |
| **Category** | Governance |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Audit export is required for regulatory submissions, internal compliance reviews, and contractual evidence packages. |
| **Acceptance Criteria** | - Export action is available to artifact owners and platform administrators.<br>- JSON export conforms to a documented schema version.<br>- PDF/HTML report includes artifact metadata, reviewer list, decision summary, and chronological event log.<br>- Export includes all comment bodies, including deleted comments marked "[DELETED by {user} at {timestamp}]". |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-042 |
| **Title** | Per-Artifact Review History |
| **Description** | The system SHALL maintain a complete review history per artifact, listing all review submissions (past and current) with their outcome, revision count, reviewer list, and date range, accessible from the artifact's detail page. |
| **Category** | Functional |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Artifacts evolve over multiple review cycles; history gives maintainers insight into the full review genealogy. |
| **Acceptance Criteria** | - Review history panel is always visible on the artifact detail page.<br>- Each review entry links to its full review record (read-only for closed reviews).<br>- History is never deleted, even if the artifact is archived. |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-043 |
| **Title** | Comment Edit History |
| **Description** | The system SHALL retain the full edit history of every comment, showing each version of the comment body with the editor's identity and edit timestamp, accessible via an "Edited" indicator on the comment. |
| **Category** | Governance |
| **Persona(s)** | All Architects, **Reviewer** |
| **Priority** | Must Have |
| **Rationale/Source** | Editable comments without history create accountability gaps; reviewers must not be able to silently retract critical feedback after the fact. |
| **Acceptance Criteria** | - "Edited {n} times" indicator appears on modified comments.<br>- Clicking the indicator shows a diff-style history panel with all versions.<br>- Comment body edit is limited to the original commenter; edit by others is not permitted (they must reply instead). |

---

## REV-07 — Access & Reviewer Licensing

### REV-07-01 Login Requirement

| Field | Value |
|---|---|
| **ID** | REQ-REV-044 |
| **Title** | Login Required for All Review Participants |
| **Description** | The system SHALL require all review participants — authors, assigned reviewers, @mentioned users, and observers — to be authenticated (logged in to the ESIP platform) before accessing any review, comment, or artifact snapshot; unauthenticated deep-links SHALL redirect to the login page and then return the user to the original destination after login. |
| **Category** | Security |
| **Persona(s)** | All Architects, Product Manager, **Reviewer** |
| **Priority** | Must Have |
| **Rationale/Source** | Enterprise data governance prohibits anonymous access to architecture artifacts; login gates protect IP and audit integrity. |
| **Acceptance Criteria** | - All review URLs return HTTP 401 to unauthenticated requests.<br>- Post-login redirect preserves the original deep-link target (including specific comment anchor).<br>- Session timeout follows the platform-wide policy (configurable, default 8 hours). |
| **Dependencies** | V11 (identity/auth) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-045 |
| **Title** | Free Reviewer License Tier |
| **Description** | The system SHALL define a **Reviewer** license tier that is free of charge, grants the license holder the ability to: be assigned as a reviewer, read any artifact they are assigned to review (snapshot only), create/edit/resolve their own comments and replies, submit review decisions, and view the audit trail of reviews they participated in; the Reviewer license SHALL NOT grant the ability to create artifacts, initiate reviews, access the full platform workspace, or consume any other platform capability beyond the review UI. |
| **Category** | Governance |
| **Persona(s)** | **Reviewer** |
| **Priority** | Must Have |
| **Rationale/Source** | Free reviewer access maximizes stakeholder participation (business SMEs, external auditors, cross-org reviewers) without license-cost barriers; increases review coverage and quality. |
| **Acceptance Criteria** | - Reviewer license is self-service registerable by any invited user.<br>- Review-only UI surface is presented to Reviewer-licensed users (no artifact creation, no workspace navigation beyond the review).<br>- Platform administrators can provision/revoke Reviewer licenses.<br>- Reviewer license count is unlimited (no per-seat cost) in all platform license tiers. |
| **Dependencies** | V13 (license type provisioning), V11 (RBAC) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-046 |
| **Title** | Permission-Scoped Review Access |
| **Description** | The system SHALL enforce that a reviewer's access to an artifact snapshot is scoped strictly to the specific review they are assigned to; they SHALL NOT be able to browse the artifact repository, access other versions of the artifact, or access other artifacts in the project unless they hold a broader platform license. |
| **Category** | Security |
| **Persona(s)** | **Reviewer** |
| **Priority** | Must Have |
| **Rationale/Source** | Need-to-know principle; reviewers may be external stakeholders who should not have broad platform visibility. |
| **Acceptance Criteria** | - Reviewer-license API calls are scoped to a review ID allow-list at the authorization layer.<br>- Reviewer cannot navigate to "Browse artifacts" or "Project workspace" — those routes return 403.<br>- Access is revoked automatically when the review moves to Approved or Cancelled state (configurable retention window, default 30 days post-close). |
| **Dependencies** | V11 (RBAC, review-scoped permissions) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-047 |
| **Title** | Guest Reviewer Invitation via Email |
| **Description** | The system SHALL allow the review author to invite external (non-platform) users as reviewers by email address; the system SHALL send an invitation email with a one-time registration link that pre-provisions a Reviewer license and pre-assigns the recipient to the review. |
| **Category** | Functional |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Should Have |
| **Rationale/Source** | Cross-organizational reviews (vendor reviews, customer acceptance, external auditors) are common; email-based invitation removes the registration friction barrier. |
| **Acceptance Criteria** | - Invitation email contains: artifact name, author, due date, a platform registration link, and a direct review deep-link.<br>- Registration link expires after 7 days (configurable).<br>- Invited reviewer's account is created with Reviewer license only.<br>- Author sees invitation status (Invited / Registered / Active) in the reviewer list. |
| **Dependencies** | V11 (identity provisioning), V13 (license provisioning) |

---

## REV-08 — AI-Augmented Review

> **Reconciliation note — Canonical "AI Finding" record (cross-ref REQ-REV-068).** Volume 15 is the **canonical owner** of the shared **AI Finding** record. All AI findings surfaced in this volume (security — REQ-REV-048; NFR — REQ-REV-050; and the AI-finding feedback loop — REQ-REV-059) are instances of this record. **V20 AIRB (AI pre-review)** and **V06** write findings into this same record rather than defining their own finding schemas; V15 owns the schema, lifecycle, and de-duplication. See the canonical definition in **Reconciliation Additions → REQ-REV-068**.

### REV-08-01 Security Aspect Detection

| Field | Value |
|---|---|
| **ID** | REQ-REV-048 |
| **Title** | Automatic Security Concern Detection and Surface |
| **Description** | The system SHALL automatically analyze the submitted artifact at the start of each review and surface any detected security aspects, concerns, and warnings — including authentication/authorization gaps, unencrypted data flows, missing security zones, overly permissive trust boundaries, exposed secrets/credentials patterns, and compliance deviations — as a dedicated **Security Review** panel visible to all reviewers. |
| **Category** | AI/ML |
| **Persona(s)** | **Reviewer**, Security Architect, Enterprise Architect, Solution Architect |
| **Priority** | Must Have |
| **Rationale/Source** | Security review is consistently the most under-served area in architecture reviews; proactive surfacing catches issues before they become production vulnerabilities. |
| **Acceptance Criteria** | - Security panel is populated within 60 seconds of review submission.<br>- Each finding includes: severity (Critical/High/Medium/Low/Info), description, the specific artifact location (section/element/requirement ID), and a recommended action.<br>- Findings are linkable to comments (reviewer can "Create comment from finding" with one click).<br>- Security panel is always present, even if no findings are detected (shows "No security concerns detected" with confidence note). |
| **Dependencies** | V06 (AI security analysis model) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-049 |
| **Title** | Security Reviewer Suggestion Based on Security Findings |
| **Description** | The system SHALL suggest that a Security Architect persona reviewer be added to the review if security findings of severity High or Critical are detected and no Security Architect is currently assigned. |
| **Category** | AI/ML |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Should Have |
| **Rationale/Source** | Ensures high-severity security findings are reviewed by an appropriately credentialed reviewer, not just the artifact author. |
| **Acceptance Criteria** | - Suggestion banner appears in the reviewer list section if the condition is met.<br>- Author can dismiss the suggestion with a reason.<br>- If the author accepts, the suggested reviewer picker opens pre-filtered to Security Architect role. |

---

### REV-08-02 Non-Functional Aspect Detection & Topic Navigator

| Field | Value |
|---|---|
| **ID** | REQ-REV-050 |
| **Title** | Automatic NFR Concern Detection and Surface |
| **Description** | The system SHALL automatically detect and surface non-functional requirement concerns within the artifact under review, including: missing performance targets, absent scalability considerations, unaddressed availability/HA requirements, undefined recovery point/time objectives (RPO/RTO), missing observability/monitoring provisions, unconstrained resource consumption, and absent accessibility/internationalization considerations. |
| **Category** | AI/ML |
| **Persona(s)** | **Reviewer**, All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | NFRs are systematically omitted from first-draft architecture artifacts; AI-surfacing ensures coverage before formal review. |
| **Acceptance Criteria** | - NFR findings are displayed in a dedicated NFR panel alongside the Security panel.<br>- Each finding references the artifact location and the NFR category (Performance, Scalability, Availability, Recoverability, Observability, Security, Compliance, Cost, Usability).<br>- Findings auto-generate draft AI comments that reviewers can publish as formal comments with one click. |
| **Dependencies** | V06 (AI NFR analysis model) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-051 |
| **Title** | AI Topic Navigator — One-Click Jump to Any Concern Area |
| **Description** | The system SHALL provide a **Topic Navigator** panel that organizes all AI-detected concerns and reviewer comments by topic category — Security, Performance, Scalability, Availability, Functional, Data, Integration, Compliance, Cost, Usability, and any custom tags — allowing any reviewer to jump directly to any topic cluster with a single click, eliminating the need to scroll through the entire document or diagram to find relevant concerns. |
| **Category** | UX |
| **Persona(s)** | **Reviewer**, All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Long architecture documents (50+ pages) and complex diagrams (100+ elements) make linear navigation impractical; topic-clustered navigation is the #1 reviewer productivity feature for large artifacts. |
| **Acceptance Criteria** | - Topic Navigator is docked as a collapsible left-panel (or accessible via keyboard shortcut) in the review designer.<br>- Each topic node shows: badge count (comments + AI findings), severity indicator (highest severity in the group), and expand/collapse for sub-items.<br>- Clicking a topic item scrolls the document or canvas to the anchored location AND highlights the associated comment/finding.<br>- Topics with zero items are hidden by default (toggleable).<br>- Navigator updates in real-time as new comments are added during the review. |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-052 |
| **Title** | Concern Areas Proactively Presented to Reviewers |
| **Description** | The system SHALL proactively present concern areas to reviewers upon entering the review for the first time, via a **Review Briefing** modal or panel, summarizing: the top 3 security concerns, top 3 NFR gaps, and any open questions from the author's context note, so that reviewers orient their attention before beginning detailed review. |
| **Category** | UX |
| **Persona(s)** | **Reviewer** |
| **Priority** | Should Have |
| **Rationale/Source** | First-time reviewer orientation reduces time-to-first-comment and prevents superficial reviews that miss the critical issues. |
| **Acceptance Criteria** | - Briefing modal appears on first entry to a review session (dismiss persists per reviewer per review).<br>- Briefing is regeneratable from the Topic Navigator ("Re-brief me" action).<br>- Briefing links to each concern area in the Topic Navigator. |

---

### REV-08-03 Functional & Architecture Assistance Suggestions

| Field | Value |
|---|---|
| **ID** | REQ-REV-053 |
| **Title** | AI-Suggested Class Candidates per Technology Stack |
| **Description** | The system SHALL allow any reviewer to request AI-generated class/component candidates for a selected section of an application or solution architecture document, scoped to the technology stack declared in the artifact (e.g., .NET/C#, Rust, Java/Spring, Node.js/TypeScript); the suggestions SHALL be presented as a structured list with rationale, importable as a candidate diagram into the review as a generated review aid. |
| **Category** | AI/ML |
| **Persona(s)** | **Reviewer**, Application Architect, Solution Architect, Microservices Architect |
| **Priority** | Should Have |
| **Rationale/Source** | Reviewers often identify missing classes/components but cannot quickly draft alternatives; AI-suggested candidates accelerate review feedback quality. |
| **Acceptance Criteria** | - "Suggest Classes" action is available in the review tools menu when a section with component description is selected.<br>- Output includes: class/component name, responsibility, key methods/interfaces, and relationship to existing components.<br>- Suggestions are exportable as a draft UML class diagram (V03/V10). |
| **Dependencies** | V06 (AI code/architecture model), V03 (diagram generation) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-054 |
| **Title** | AI-Generated Diagram Candidates as Review Aids |
| **Description** | The system SHALL allow any reviewer to request AI-generated candidate diagrams — including use-case diagrams, sequence diagrams, logical architecture diagrams, C4 Context/Container/Component models, deployment diagrams, networking diagrams, and HA/DR topology diagrams — based on the artifact content under review; generated diagrams SHALL be attached to the review as **review aids** (not authoritative artifacts) that the reviewer or author can reference, comment on, or promote. |
| **Category** | AI/ML |
| **Persona(s)** | **Reviewer**, Enterprise Architect, Solution Architect, Application Architect, Cloud Architect, Infra Architect |
| **Priority** | Must Have |
| **Rationale/Source** | Generated candidate diagrams are the most powerful review-time tool for surfacing missing or incorrect architecture decisions; they make implicit structure explicit. |
| **Acceptance Criteria** | - "Generate Diagram" action is available in the review tools menu, with a diagram type selector.<br>- Generated diagrams are rendered in the maxGraph canvas within the review context (read-only baseline).<br>- Generated diagrams are watermarked "AI-Generated Review Aid — Not Authoritative".<br>- Reviewer or author can "Promote" a generated diagram to a full artifact (via V10), which exits the review context and creates a new artifact.<br>- Generated diagrams can be commented on with element-level comments (REQ-REV-021 through REQ-REV-024). |
| **Dependencies** | V06 (AI diagram generation model), V03 (maxGraph render), V10 (promote to artifact) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-055 |
| **Title** | AI Review Summary: Changes, Key Risks, Completeness Score |
| **Description** | The system SHALL generate an **AI Review Summary** for every review at submission time and on each revision, providing: a concise natural-language summary of what changed from the prior version; identified key risks (ranked by severity); and a **completeness score** (0–100%) reflecting how well the artifact covers its declared scope, required sections, and NFR categories. |
| **Category** | AI/ML |
| **Persona(s)** | **Reviewer**, All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Review summary reduces reviewer onboarding time from 15 minutes to under 2 minutes on large artifacts; completeness score sets a quality baseline. |
| **Acceptance Criteria** | - Summary is generated within 90 seconds of submission/revision.<br>- Summary length: 100–300 words (configurable).<br>- Completeness score is accompanied by a breakdown by category (Functional coverage, NFR coverage, Diagram coverage, Requirement ID completeness).<br>- Summary is pinned at the top of the review thread, accessible from the review header. |
| **Dependencies** | V06 (AI summarization model) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-056 |
| **Title** | Per-Deliverable AI Review Checklist |
| **Description** | The system SHALL generate a per-deliverable review checklist at the start of each review, customized to the artifact type (e.g., Enterprise Architecture checklist, Security Architecture checklist, Integration Architecture checklist, BRD checklist, HLD checklist) and aligned to the relevant framework (TOGAF ADM phases, C4 model completeness, SABSA security layers, etc.); checklist items SHALL be trackable (checked/unchecked) by each reviewer independently. |
| **Category** | AI/ML |
| **Persona(s)** | **Reviewer**, All Architects, Product Manager |
| **Priority** | Must Have |
| **Rationale/Source** | Checklists are the #1 evidence-based reviewer effectiveness tool; framework-aligned checklists ensure architectural rigor without requiring reviewers to memorize standards. |
| **Acceptance Criteria** | - Checklist is auto-populated within 60 seconds of review submission.<br>- Each reviewer has their own independent checklist state (checking an item for one reviewer does not check it for others).<br>- Checklist items that are unchecked at decision time trigger a warning ("You have {n} unchecked items — are you sure you want to submit your decision?").<br>- Checklists are extensible: platform administrators can define custom checklist templates per artifact type.<br>- Aggregate checklist completion % is shown in the review header. |
| **Dependencies** | V06 (AI checklist generation), framework templates |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-057 |
| **Title** | Reviewer Effort Metrics |
| **Description** | The system SHALL track and surface reviewer effort metrics per review: time-on-review (time between first open and decision submission), comment count authored, checklist completion rate, and number of AI suggestions reviewed; these metrics SHALL be used to compute a **review quality indicator** visible to the author and platform administrators. |
| **Category** | Reporting |
| **Persona(s)** | All Architects, Product Manager |
| **Priority** | Could Have |
| **Rationale/Source** | Effort metrics incentivize thorough reviews; low-effort reviews (30 seconds, zero comments, "Approve") undermine the review process. |
| **Acceptance Criteria** | - Metrics are collected passively without requiring any reviewer action.<br>- Review quality indicator is displayed on the reviewer's card in the review header (color-coded: Green ≥70%, Amber 40–69%, Red <40%).<br>- Metrics are includable in the audit export (REQ-REV-041). |

---

### REV-08-04 AI Review Performance & Quality

| Field | Value |
|---|---|
| **ID** | REQ-REV-058 |
| **Title** | AI Analysis Latency SLA |
| **Description** | The system SHALL complete all AI analysis tasks — security scanning, NFR scanning, review summary, and checklist generation — within 90 seconds of review submission for artifacts up to 50 pages / 500 diagram elements; larger artifacts shall complete within 5 minutes. |
| **Category** | NFR |
| **Persona(s)** | All Architects, Product Manager, **Reviewer** |
| **Priority** | Must Have |
| **Rationale/Source** | AI analysis that takes more than 2 minutes breaks the review workflow; reviewers begin reviewing before AI context is available. |
| **Acceptance Criteria** | - P95 latency for standard artifacts (≤50 pages, ≤500 elements): ≤90 seconds.<br>- P95 latency for large artifacts: ≤5 minutes.<br>- A progress indicator ("AI is analyzing this artifact…") is shown during analysis.<br>- Partial results (e.g., security scan complete, NFR scan still running) are surfaced incrementally. |
| **Dependencies** | V06 (AI infrastructure scaling) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-059 |
| **Title** | AI Finding Feedback Loop |
| **Description** | The system SHALL allow reviewers and authors to rate individual AI findings as **Useful** or **Not useful / False positive**, so that the AI model can be fine-tuned over time for the organization's specific architecture patterns and terminology. |
| **Category** | AI/ML |
| **Persona(s)** | **Reviewer**, All Architects, Product Manager |
| **Priority** | Should Have |
| **Rationale/Source** | AI finding accuracy improves with domain-specific feedback; false-positive fatigue will erode trust in the AI panel if not addressed. |
| **Acceptance Criteria** | - Thumbs up/down rating is available on every AI finding card.<br>- Ratings are stored per finding with user identity for model feedback pipeline (V06).<br>- "Not useful" rating includes an optional category: Wrong location / Incorrect assessment / Out of scope / Duplicate.<br>- Aggregate rating data is exportable by platform administrators. |
| **Dependencies** | V06 (model feedback pipeline) |

---

## REV-09 — Cross-Team Scoped Dependency Sharing

### REV-09-01 Scoped Artifact Sharing

| Field | Value |
|---|---|
| **ID** | REQ-REV-060 |
| **Title** | Share Full Artifact with Another Team for Review |
| **Description** | The system SHALL allow the artifact owner to share an artifact (or a specific approved version/snapshot) with members of another team or project, granting them Reviewer-level access for the purpose of dependency review and integration planning, without granting access to the full project workspace. |
| **Category** | Functional |
| **Persona(s)** | Enterprise Architect, Solution Architect, Integration Architect, API Architect |
| **Priority** | Must Have |
| **Rationale/Source** | Cross-team dependency review is a primary collaboration pattern in large enterprise architectures; current tooling forces screenshot email chains. |
| **Acceptance Criteria** | - "Share with Team" action is available to the artifact owner on any published artifact version.<br>- Recipient team members receive a notification and are granted Reviewer license for the shared artifact.<br>- Shared artifact appears in a "Shared with me" section of the recipient's ESIP workspace.<br>- Sharing is revokable by the owner at any time. |
| **Dependencies** | V11 (cross-team permissions) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-061 |
| **Title** | Scoped Part Sharing — Dependency Interface Only |
| **Description** | The system SHALL allow the artifact owner to share only a **scoped subset** of an artifact — specifically the dependency interface details (e.g., a specific API contract section, a data schema, a set of integration endpoints, an interface definition table, a specific diagram layer/subgraph) — with another team, so that the dependent team receives only the information relevant to their integration, without exposure to the full architecture document. |
| **Category** | Functional |
| **Persona(s)** | Integration Architect, API Architect, Enterprise Architect, Data Architect |
| **Priority** | Must Have |
| **Rationale/Source** | Information security and IP protection requirements prohibit sharing full architecture documents externally; scoped sharing enables collaboration without over-disclosure. |
| **Acceptance Criteria** | - Artifact owner can select specific sections, requirement groups, table rows, or diagram subgraphs for scoped sharing.<br>- Scoped share creates a **Shared Contract** object containing only the selected content, with a version tag.<br>- Recipients can view only the Shared Contract content; the parent artifact is not accessible.<br>- Scoped share includes a "Last updated" timestamp and a notification when the source content changes. |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-062 |
| **Title** | Controlled Visibility on Shared Contracts |
| **Description** | The system SHALL enforce that shared contracts are accessible only to explicitly named recipient users or teams; sharing SHALL NOT create public links or broadly accessible URLs; all access SHALL be authenticated (login required, REQ-REV-044) and SHALL be logged in the audit trail. |
| **Category** | Security |
| **Persona(s)** | Enterprise Architect, Solution Architect, Integration Architect, API Architect |
| **Priority** | Must Have |
| **Rationale/Source** | Architecture IP and commercial-sensitive interface details must not leak beyond intended recipients. |
| **Acceptance Criteria** | - Shared contract URLs return 403 to unauthenticated or unauthorized users.<br>- Access list is visible to the owner in the sharing management panel.<br>- All access events are logged in the artifact audit trail. |
| **Dependencies** | V11 (access control) |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-063 |
| **Title** | "Request More Detail" Workflow from Dependent Team |
| **Description** | The system SHALL allow a dependent team (recipient of a shared contract) to submit a formal "Request More Detail" request to the owning team from within the shared contract view, specifying which aspect they need clarification or additional information on; the owning team receives the request, responds in the same thread, and can optionally expand the scope of the shared contract in response. |
| **Category** | Functional |
| **Persona(s)** | Integration Architect, API Architect, Enterprise Architect, Solution Architect |
| **Priority** | Must Have |
| **Rationale/Source** | Dependency resolution always involves back-and-forth clarification; a formal in-platform request workflow prevents email fragmentation and creates an auditable dependency negotiation record. |
| **Acceptance Criteria** | - "Request More Detail" button is available on every shared contract view.<br>- Request form includes: specific section/element, question text, priority (Blocking / Important / Nice-to-have).<br>- Owning team receives in-app notification and email.<br>- Responses are threaded in the shared contract view (visible to both teams).<br>- Request and response history is included in the artifact audit trail. |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-064 |
| **Title** | Versioned Shared Contracts |
| **Description** | The system SHALL version shared contracts so that when the underlying source content changes (a new artifact revision is published), the dependent team receives a notification of the version change and can view a diff between their current shared contract version and the new version before accepting the update. |
| **Category** | Functional |
| **Persona(s)** | Integration Architect, API Architect, Enterprise Architect |
| **Priority** | Must Have |
| **Rationale/Source** | Breaking changes in shared interfaces (API contracts, data schemas) must be communicated formally with explicit version acceptance; silent updates create integration failures. |
| **Acceptance Criteria** | - Each shared contract has a semantic version (e.g., v1.0, v1.1, v2.0).<br>- Dependent team receives notification when a new version is published by the owning team.<br>- Diff view shows changes between the accepted version and the new version (using the same diff framework as REQ-REV-030/031).<br>- Dependent team can "Accept new version" or continue consuming the prior version (with a warning after configurable grace period). |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-065 |
| **Title** | Dependent Team Consumption in Their Own Design Context |
| **Description** | The system SHALL allow the dependent team to import a versioned shared contract into their own ESIP workspace as a **Referenced Dependency** artifact, which is read-only (reflects the shared contract content) and is linkable from their own diagrams, requirement sets, and architecture documents as a pinned external reference. |
| **Category** | Functional |
| **Persona(s)** | Integration Architect, API Architect, Solution Architect, Data Architect |
| **Priority** | Must Have |
| **Rationale/Source** | Dependent teams need to reference the upstream contract within their own design work to maintain traceability; copying/pasting creates maintenance debt when the contract changes. |
| **Acceptance Criteria** | - "Import as Referenced Dependency" action is available on any accepted shared contract.<br>- Referenced Dependency artifact shows: source team, artifact name, contract version, last-updated timestamp.<br>- Version drift warning appears when the source contract has a new version that has not been accepted by the dependent team.<br>- Referenced Dependency can be dragged onto a maxGraph canvas as a reference node linked to the contract details (V03). |
| **Dependencies** | V03 (diagram reference node), V10 (referenced dependency artifact type) |

---

### REV-09-02 Shared Contract Governance

| Field | Value |
|---|---|
| **ID** | REQ-REV-066 |
| **Title** | Shared Contract Audit Trail |
| **Description** | The system SHALL maintain a separate, immutable audit trail for each shared contract, recording: creation, version changes, access grants/revocations, request-more-detail threads, version acceptance/rejections by dependent teams, and contract expiry. |
| **Category** | Governance |
| **Persona(s)** | Enterprise Architect, Integration Architect |
| **Priority** | Must Have |
| **Rationale/Source** | Shared contracts represent interface agreements between teams; the audit trail is the evidence of the negotiation and agreement process for integration governance. |
| **Acceptance Criteria** | - Shared contract audit trail is accessible to the owning team owner and platform administrators.<br>- Audit trail is exportable (JSON + PDF) independently of the parent artifact audit trail.<br>- Audit records meet the same immutability requirements as REQ-REV-039. |

---

| Field | Value |
|---|---|
| **ID** | REQ-REV-067 |
| **Title** | Shared Contract Expiry and Revocation |
| **Description** | The system SHALL support setting an expiry date on any shared contract; upon expiry (or earlier revocation by the owner), dependent team access is automatically revoked, the Referenced Dependency artifact in the dependent team's workspace is marked "Contract Expired — contact the owning team", and the dependent team is notified. |
| **Category** | Governance |
| **Persona(s)** | Enterprise Architect, Integration Architect |
| **Priority** | Should Have |
| **Rationale/Source** | Architecture contracts have defined validity periods (project phases, sprint boundaries, contract durations); time-bounded sharing prevents stale dependency references. |
| **Acceptance Criteria** | - Expiry date is settable on shared contract creation and updateable by the owning team owner.<br>- 7-day and 1-day expiry warning notifications are sent to both teams.<br>- Expired contracts are accessible in read-only mode to the owning team for audit purposes. |

---

## Reconciliation Additions

> Added during the cross-volume finalization pass (2026-06-23). New requirements only; no existing requirement was modified or removed. See FINALIZATION-REPORT.md for rationale.

### RA-1 Canonical AI Finding Record

| Field | Value |
|---|---|
| **ID** | REQ-REV-068 |
| **Title** | Canonical "AI Finding" Record — V15 as System of Record |
| **Description** | The system SHALL define a single, canonical **AI Finding** record owned by Volume 15, and ALL AI-generated review findings — regardless of which engine produced them — SHALL be persisted as instances of this record. The AI Finding record SHALL include at minimum the fields: **`id`** (stable unique identifier), **`severity`** (Critical / High / Medium / Low / Info), **`lens`** (the analysis lens, e.g. Security, NFR, Functional, Architecture, Compliance), **`text`** (human-readable finding description with artifact-location reference), and **`source`** ∈ **{`AIRB-pre`, `REV08-inline`}** identifying the producing path. V15 SHALL own the record's schema, lifecycle (create / link-to-comment / resolve / rate), and cross-source de-duplication. **V20 AIRB** (AI pre-review) and **V06** SHALL write their findings into this canonical record rather than maintaining a separate finding model; downstream consumers (Security panel REQ-REV-048, NFR panel REQ-REV-050, finding feedback loop REQ-REV-059, and "Create comment from finding") SHALL read from this single record. |
| **Category** | AI/ML |
| **Persona(s)** | **Reviewer**, All Architects, Product Manager, Security Architect |
| **Priority** | Must Have |
| **Rationale/Source** | Reconciliation finding (FINALIZATION-REPORT.md): V15 inline AI and V20 AIRB both produce findings; a single shared record prevents schema drift, double-counting, and divergent severity scales, and lets V20 pre-review and V15 inline findings share one panel, one feedback loop, and one audit trail. |
| **Acceptance Criteria** | - A single AI Finding schema (versioned) is the system of record; no consumer defines its own competing finding schema.<br>- Every AI Finding carries `id`, `severity`, `lens`, `text`, and `source` ∈ {AIRB-pre, REV08-inline}.<br>- Findings written by V20 AIRB and by V06 are stored as instances of this record and are indistinguishable to consumers except via the `source` field.<br>- De-duplication: findings from different sources targeting the same artifact location + lens are merged or cross-linked rather than duplicated.<br>- The finding feedback loop (REQ-REV-059) and "Create comment from finding" operate uniformly across all sources. |
| **Dependencies** | V06 (AI infrastructure — writes findings into this record), V20 (AIRB pre-review — writes findings into this record; V15 is the record owner) |

---

## Appendix A — Requirement Summary

| Capability Area | Req Count | IDs |
|---|---|---|
| REV-01 Review Request & Reviewer Selection | 9 | REQ-REV-001 – REQ-REV-009 |
| REV-02 Native Review Designer: Documents | 11 | REQ-REV-010 – REQ-REV-020 |
| REV-03 Native Review Designer: Diagrams & Models | 5 | REQ-REV-021 – REQ-REV-025 + REQ-REV-026 |
| REV-04 Review Lifecycle & Versioning | 6 | REQ-REV-027 – REQ-REV-032 |
| REV-05 Feedback Incorporation & AI-Apply | 6 | REQ-REV-033 – REQ-REV-038 |
| REV-06 Audit & History | 5 | REQ-REV-039 – REQ-REV-043 |
| REV-07 Access & Reviewer Licensing | 4 | REQ-REV-044 – REQ-REV-047 |
| REV-08 AI-Augmented Review | 12 | REQ-REV-048 – REQ-REV-059 |
| REV-09 Cross-Team Scoped Dependency Sharing | 8 | REQ-REV-060 – REQ-REV-067 |
| **TOTAL** | **67** | **REQ-REV-001 – REQ-REV-067** |

---

## Appendix B — Boundary Assumptions

| Concern | Disposition |
|---|---|
| Approval gate routing, SLA enforcement, timer-based escalation | → V12 (Workflow). REV-01 stores due dates; V12 consumes them. |
| AI model hosting, RAG pipeline, embedding infrastructure, model selection | → V06 (AI Infrastructure). Volume 15 specifies AI behaviors and UX contracts only. |
| maxGraph canvas rendering, element model API, diagram generation primitives | → V03 (Diagramming). Volume 15 specifies the review overlay and comment anchor contracts on top of V03. |
| Identity, SSO, RBAC infrastructure, team directory, notification delivery | → V11 (Platform). Volume 15 specifies access patterns and license tier behaviors; V11 implements them. |
| Deliverable generation, artifact version storage, artifact type registry | → V10 (Deliverables). Volume 15 specifies snapshot and promote-to-artifact contracts. |
| License type definitions, license provisioning flows, billing | → V13 (Onboarding). Volume 15 defines the Reviewer license capability boundary only. |
| Intelligence layer, analytics, cross-artifact pattern mining | → V16 (Intelligence). Volume 15 generates structured events (comment data, decision data) that V16 consumes. |

---

## Appendix C — Persona Map

| Persona | Primary Review Capabilities |
|---|---|
| Enterprise Architect | Full review authoring, submission, AI-assisted review of EA artifacts, shared contract owner |
| Solution Architect | Full review authoring, solution diagram review, shared contract consumer/owner |
| Application Architect | Full review authoring, application-level diagram review, class suggestion consumer |
| Integration Architect | Full review authoring, edge-level diagram review, shared contract owner/consumer |
| API Architect | Full review authoring, API contract review, scoped share author |
| CRM Architect | Full review authoring, CRM integration pattern review |
| Data Architect | Full review authoring, data schema review, scoped data contract sharing |
| Security Architect | Full review authoring, security panel SME, security finding owner |
| Cloud Architect | Full review authoring, deployment/networking diagram review |
| Infra Architect | Full review authoring, HA/DR diagram review |
| Microservices Architect | Full review authoring, service mesh / sequence diagram review |
| Product Manager | Full review authoring, BRD/roadmap review, reviewer assignment |
| **Reviewer** | Comment, reply, resolve, reopen, decision, checklist (no artifact creation, no workspace navigation) — **free license** |

---

*End of Volume 15 — Native Review & Collaboration Designer*
