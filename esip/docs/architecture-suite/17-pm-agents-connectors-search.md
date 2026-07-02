# Volume 17 — Product Management, License Tiers, Scanner Agents, Real-time Connectors & Global Search
## ESIP Architecture Automation Suite

| Attribute | Value |
|---|---|
| Volume | 17 |
| Sub-prefixes | `REQ-PM-NNN` (Product Manager persona) · `REQ-LIC-NNN` (License tiers) · `REQ-AGT-NNN` (Code-scanner agent provisioning) · `REQ-RTC-NNN` (Real-time meeting/chat connectors) · `REQ-SRCH-NNN` (Global search) |
| Status | V1.0 — authored 2026-06-23 |
| Scope | P13 Product Manager persona (BRD/PRD authoring, roadmap ownership, backlog management, prioritisation, PM dashboards/metrics, handoff to architects); license tier additions (Viewer free, Reviewer free, PM license, alongside architect licenses); code-scanner agent provisioning (Azure DevOps self-hosted–style: download, install, register, pool, health, auto-update, EA-configured scan targets, air-gap support, data-boundary controls, continuous portfolio enrichment); real-time meeting and chat connectors (Microsoft Teams auto-transcripts + channel chat essence, Outlook email+calendar, Zoom transcripts/recordings — extending V08's base connector framework); system-wide global search (semantic + keyword hybrid, permission-aware, deep-link, NLP, saved searches, analytics) |
| **Out of Scope** | AI reasoning/extraction/RAG internals → **V06**; base connector framework/OAuth/credential vault → **V08**; code-scan parse/flow-derivation/portfolio model → **V09**; persona metamodel engine/RBAC/audit/multi-tenancy → **V05/V11**; licensing admin enforcement internals/billing/seat metering → **V13**; review designer → **V15**; search indexing infrastructure (inverted index, vector store, ANN) → **V11** |

---

## Table of Contents

1. [REQ-PM — Product Manager Persona (P13)](#1-req-pm--product-manager-persona-p13)
   - 1.1 Persona Foundation & Workspace
   - 1.2 Business Requirements Document (BRD) Authoring
   - 1.3 Product Requirements Document (PRD) Authoring
   - 1.4 AI-Assisted BRD/PRD Authoring & Completeness Nudging
   - 1.5 Product-Centric Requirement Capture
   - 1.6 Product Roadmap (PM-owned)
   - 1.7 Product Backlog — Epics, Features & Stories
   - 1.8 Prioritisation Frameworks (RICE / Kano / WSJF / MoSCoW)
   - 1.9 BRD → Roadmap Item → Architecture Handoff
   - 1.10 PM Dashboards & Metrics
   - 1.11 PM Deliverables Library
2. [REQ-LIC — License Tiers](#2-req-lic--license-tiers)
   - 2.1 Tier Definitions & Scope
   - 2.2 Viewer Tier (Free)
   - 2.3 Reviewer Tier (Free)
   - 2.4 Product Manager License
   - 2.5 Tier Interactions, Upgrade Paths & Rules
   - 2.6 Free-Tier Governance & Limits
3. [REQ-AGT — Code-Scanner Agent Provisioning](#3-req-agt--code-scanner-agent-provisioning)
   - 3.1 Agent Model & Architecture
   - 3.2 Agent Download, Install & Registration
   - 3.3 Agent Pools & Capacity
   - 3.4 Health, Heartbeat & Logs
   - 3.5 Auto-Update & Version Management
   - 3.6 EA-Configured Scan Targets
   - 3.7 Clone, Scan Execution & Local Runner
   - 3.8 Derived Results Streaming & Data Boundary Controls
   - 3.9 Continuous Portfolio Enrichment
   - 3.10 Air-Gapped Deployment
   - 3.11 Multi-Agent Scaling
   - 3.12 Security & Isolation
4. [REQ-RTC — Real-Time Meeting & Chat Connectors](#4-req-rtc--real-time-meeting--chat-connectors)
   - 4.1 Connector Registration & Consent Framework
   - 4.2 Microsoft Teams — Meeting Auto-Capture
   - 4.3 Microsoft Teams — Channel & Group Chat Essence
   - 4.4 Outlook — Email & Calendar
   - 4.5 Zoom — Meeting Transcripts & Recordings
   - 4.6 Speaker Attribution & Identity Resolution
   - 4.7 AI Summarisation & Essence Extraction
   - 4.8 Decision & Action-Item Extraction (→ V06)
   - 4.9 Real-Time vs. Scheduled Sync
   - 4.10 Project/Roadmap-Item Mapping
   - 4.11 Compliance, Consent & Retention
5. [REQ-SRCH — System-Wide Global Search](#5-req-srch--system-wide-global-search)
   - 5.1 Search Scope & Index Coverage
   - 5.2 Hybrid Semantic + Keyword Engine
   - 5.3 Filters, Facets & Scoped Search
   - 5.4 Permission-Aware Results
   - 5.5 Instant / Typeahead Search
   - 5.6 Natural-Language Search
   - 5.7 Deep-Link & Result Navigation
   - 5.8 Saved Searches & Alerts
   - 5.9 Cross-Project / Cross-Workspace Search
   - 5.10 Search Analytics & Insights

---

## 1. REQ-PM — Product Manager Persona (P13)

### Overview
P13 is the **Product Manager** persona. PMs own product roadmaps, author BRDs and PRDs, maintain product backlogs, run prioritisation exercises, and hand off to architects (P1–P12) for design execution. ESIP must serve both **rigorous PMs** who meticulously capture functional, NFR, security, and performance requirements *and* **lightweight PMs** who skip categories — the system proactively nudges and AI-assists to fill gaps in both cases. P13 operates alongside architect personas; a user may hold both a PM license and an architect persona license.

---

### 1.1 Persona Foundation & Workspace

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PM-001 | P13 Persona Record | The system SHALL define a Product Manager persona (P13) as a first-class persona configuration record in the metamodel (V11), with associated artifact types (BRD, PRD, product roadmap, product backlog, release notes), diagram types (roadmap timeline, user story map, product vision board), deliverable templates, and AI copilot prompt packs, activatable per tenant independently of architect personas. | Functional | P13 | Must | Extends V05/V11 persona pattern; PM is a distinct discipline from architects |
| REQ-PM-002 | PM Workspace Home | The system SHALL provide a dedicated PM Workspace Home dashboard surfacing: owned products/programs, active BRDs and PRDs, roadmap health indicators, upcoming milestones, backlog items awaiting prioritisation, pending handoffs to architects, and AI-generated nudges for incomplete requirement categories. | UX | P13 | Must | Reduces context-switching; single start point for all PM activity |
| REQ-PM-003 | Product Registry | The system SHALL maintain a Product Registry where each product/program is a named entity with: owner (P13), description, business domain, stakeholder list, linked roadmap, linked BRDs/PRDs, lifecycle status (Concept/Active/Sunset), and creation/modification audit. | Data | P13 | Must | Anchors BRDs, roadmaps and backlogs to a product entity; prevents orphaned documents |
| REQ-PM-004 | Multi-Product Support | The system SHALL allow a single P13 user to own and manage multiple concurrent products/programs, switching context via a product selector, with no mixing of data between products unless explicitly linked. | Functional | P13 | Must | PMs frequently manage product portfolios, not single products |
| REQ-PM-005 | Cross-Persona Visibility | The system SHALL allow architect personas (P1–P12) to view (read-only) any BRD, PRD, product roadmap, and backlog item for which they are listed as a stakeholder or assignee, without requiring a PM license. | Functional | P1–P12, P13 | Must | Handoff requires architects to read PM deliverables without licence friction |

---

### 1.2 Business Requirements Document (BRD) Authoring

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PM-010 | BRD Entity Model | The system SHALL maintain each BRD as a structured document entity with: unique ID, title, version, status (Draft/In-Review/Approved/Superseded), product reference, author, stakeholder sign-off list, creation/modified timestamps, and a structured body comprising at minimum: Executive Summary, Business Objectives, Scope (In/Out), Assumptions & Constraints, Stakeholder Catalogue, Current-State Problem, Proposed Solution Overview, High-Level Requirements (Functional, NFR, Security, Performance, Compliance), Business Rules, Data Requirements, Integration Touch-points, Success Metrics, Dependencies, Risks, and Glossary. | Data | P13 | Must | BRD industry standard (IEEE 830 lineage; enterprise PM practice) |
| REQ-PM-011 | BRD Versioning | The system SHALL version each BRD automatically on every save (draft auto-version) and on explicit baseline actions (named release version), preserving full history and supporting diff view between any two versions. | Functional | P13 | Must | Governance: requirements change history is an audit obligation |
| REQ-PM-012 | BRD Status Workflow | The system SHALL enforce a configurable BRD status workflow (Draft → PM Review → Stakeholder Review → Approved → Locked) with role-based transition permissions, mandatory reviewer assignments per stage, and notifications. | Governance | P13 | Must | Formal approval gates are required before architect handoff |
| REQ-PM-013 | Structured Section Editor | The system SHALL provide a rich structured editor for each BRD section supporting: formatted text, inline tables, numbered/bulleted lists, inline diagrams (embed), embedded requirement cards (link to requirement records), acceptance criteria blocks, and embedded links to Knowledge Hub artifacts. | UX | P13 | Must | Document fidelity comparable to Word/Confluence; structured (not free-form) |
| REQ-PM-014 | BRD Requirement Cards | The system SHALL allow a PM to create formal Requirement Cards inline within a BRD section, each card containing: ID (auto-assigned per BRD), title, type (Functional/NFR/Security/Performance/Compliance/Business Rule/Data), priority, description, acceptance criteria, and source evidence link; these cards SHALL be linkable to the platform Requirements Pillar (V07). | Data | P13 | Must | Bridges informal narrative to traceable, structured requirements (V07 RTM) |
| REQ-PM-015 | BRD Templates | The system SHALL provide at least three configurable BRD templates: Standard Enterprise BRD, Lightweight BRD (simplified for smaller products), and Regulatory/Compliance BRD (expanded security/compliance sections); templates SHALL be customisable and saveable per tenant. | UX | P13 | Should | Accelerates authoring; different contexts need different depths |
| REQ-PM-016 | Roadmap Item Linkage in BRD | The system SHALL allow each BRD to be linked to one or more roadmap items (from the product roadmap or architecture roadmap), propagating the BRD ID to the roadmap item's evidence set and updating the roadmap item status when the BRD is approved. | Functional | P13, EA | Must | BRD → roadmap → architecture traceability (core V17 design principle) |

---

### 1.3 Product Requirements Document (PRD) Authoring

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PM-020 | PRD Entity Model | The system SHALL maintain each PRD as a structured document entity derived from (or linked to) an approved BRD, with: unique ID, version, status, target release, associated product, author, structured body comprising at minimum: Product Vision, Goals & Metrics (OKRs/KPIs), User Personas & Jobs-to-Be-Done, Problem Statements, Proposed Features (linked to epics/features in backlog), Detailed Functional Requirements, User Stories (linked to backlog), NFRs, UX/Design References, Out-of-Scope Items, Dependencies, Timeline, and Open Questions log. | Data | P13 | Must | PRD is the standard PM artifact bridging business intent to engineering; aligns with industry PM practice |
| REQ-PM-021 | BRD → PRD Derivation | The system SHALL support one-click derivation of a new PRD scaffold from an approved BRD, pre-filling the PRD structure with mapped content from the BRD (objectives, scope, stakeholders, high-level requirements) and flagging sections that require PM enrichment. | Functional | P13 | Must | Prevents re-work and ensures BRD→PRD traceability |
| REQ-PM-022 | PRD Versioning & Approval | The system SHALL version PRDs with the same scheme as BRDs (auto-version on save, named baseline) and enforce a PRD approval workflow configurable per product/tenant. | Governance | P13 | Must | Governance parity with BRD |
| REQ-PM-023 | Feature Table | The system SHALL provide a structured Feature Table within a PRD where each feature row contains: Feature ID, Name, Description, User Value, Linked Epic, Priority (RICE/Kano/WSJF/MoSCoW score), Status, Target Release, and Owner; the table SHALL be sortable, filterable, and exportable. | Functional | P13 | Must | Feature list is the core of a PRD; must be structured not narrative |
| REQ-PM-024 | User Story Embed | The system SHALL allow a PM to author or link User Stories (As a/I want/So that format + acceptance criteria) directly within the PRD and have them synchronise bidirectionally with the Product Backlog. | Functional | P13 | Must | PRD and backlog must not diverge; bidirectional link prevents duplication |
| REQ-PM-025 | NFR Section with Category Completeness | The system SHALL include a dedicated NFR section in the PRD with sub-categories: Performance, Availability/Reliability, Security, Scalability, Maintainability, Compliance/Regulatory, Accessibility, Internationalisation; the system SHALL indicate completeness status per sub-category and prompt the PM when any sub-category is empty. | Functional | P13 | Must | NF categories are systematically skipped in practice; structured completeness is required |

---

### 1.4 AI-Assisted BRD/PRD Authoring & Completeness Nudging

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PM-030 | AI BRD Draft Generation | The system SHALL invoke the AI engine (V06) to generate a complete BRD draft from PM-supplied inputs: product description, business objectives, known constraints, stakeholder notes, and linked Knowledge Hub artifacts; the draft SHALL populate all standard sections with content explicitly flagged as AI-generated and editable. | AI/ML | P13 | Must | AI-native authoring is a flagship ESIP differentiator; eliminates blank-page problem |
| REQ-PM-031 | AI PRD Draft from BRD | The system SHALL invoke the AI engine to generate a PRD draft from an approved (or draft) BRD, mapping BRD sections to PRD sections, inferring user personas, drafting feature statements, and proposing NFR stubs; output is fully editable and AI-labelled. | AI/ML | P13 | Must | BRD → PRD is the natural progression; AI acceleration of this step is high value |
| REQ-PM-032 | Section-Level AI Assist | The system SHALL provide per-section AI assist actions within the BRD/PRD editor: "Expand this section," "Improve clarity," "Suggest acceptance criteria," "Extract requirements from this text," "Identify risks," and "Summarise for executive audience"; each action references the document context and Knowledge Hub evidence. | AI/ML | P13 | Must | Inline AI is more useful than a separate AI chat panel for document authoring |
| REQ-PM-033 | Completeness Score & Nudging — All PMs | The system SHALL compute a real-time BRD/PRD completeness score across all defined requirement categories (Functional, NFR, Security, Performance, Compliance, Business Rules, Data, Integration) and SHALL display the score as a progress indicator, highlighting missing or thin categories in the navigation sidebar regardless of whether the PM entered them or not. | AI/ML | P13 | Must | Both rigorous and lightweight PMs benefit; AI makes the gap visible before handoff |
| REQ-PM-034 | Proactive Category Nudge (Lightweight PMs) | The system SHALL proactively surface an inline nudge/suggestion card when a PM has not addressed a requirement category (e.g., Security, Performance) after completing 50% of other sections, offering: a brief explanation of why the category matters, AI-generated starter questions for that category, and a one-click "Ask AI to draft this section" action. | AI/ML | P13 | Must | Lightweight PMs skip non-functional categories by default; nudging prevents costly late-stage architecture rework |
| REQ-PM-035 | AI-Suggested Acceptance Criteria | The system SHALL invoke the AI engine to suggest acceptance criteria for any requirement or user story in a BRD/PRD based on the description text, domain, and related Knowledge Hub context; suggestions SHALL be reviewable and individually accept/reject/editable. | AI/ML | P13 | Should | Acceptance criteria quality is the most common gap in PM-authored requirements |
| REQ-PM-036 | AI Consistency Check | The system SHALL run an AI-powered consistency check across the full BRD or PRD on demand, reporting: conflicting requirements, requirements that contradict stated scope/constraints, duplicate requirements, and requirements without acceptance criteria; results displayed as an inline issue list with navigation links. | AI/ML | P13 | Should | Catches internal contradictions before architect handoff; reduces rework |
| REQ-PM-037 | AI Risk Identification | The system SHALL invoke the AI engine to identify and narrate business, delivery, and technical risks implicit in the BRD/PRD content, appending AI-suggested risks to the Risks section for PM review and acceptance. | AI/ML | P13 | Should | Risk identification is consistently under-done in BRDs; AI can surface patterns from cross-project knowledge |
| REQ-PM-038 | Voice/Dictation Input to BRD | The system SHALL support browser-native speech-to-text input in any BRD/PRD editor field, enabling PMs to dictate requirements and have AI clean and structure the transcribed text into proper requirement syntax. | UX | P13 | Could | Accessibility and mobile PM workflow; reduces time-to-capture |
| REQ-PM-039 | AI NFR Inference from Functional Requirements | The system SHALL invoke the AI engine to infer likely NFRs (performance targets, availability tiers, security categories, scalability assumptions) from the functional requirements written by the PM, and present these as suggested NFR additions in the NFR section. | AI/ML | P13 | Must | NFRs are the most frequently omitted category; AI inference from functional text is a practical workaround for lightweight PMs |

---

### 1.5 Product-Centric Requirement Capture

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PM-040 | Requirement Category Coverage Enforcement | The system SHALL enforce coverage of all requirement categories (Functional, NFR, Security, Performance, Compliance, Data, Integration, Business Rules) during BRD/PRD validation before status progression to "Approved," displaying a category completeness gate that blocks progression if any Must-have category is empty and offers AI-fill as an alternative to manual authoring. | Governance | P13 | Must | Gate prevents incomplete BRDs from reaching architecture handoff |
| REQ-PM-041 | Functional Requirement Capture | The system SHALL provide a structured Functional Requirement form within BRDs/PRDs capturing: ID, title, description (SHALL form), actor/user, trigger, expected system behaviour, exceptions, linked acceptance criteria, and source evidence; requirements SHALL be extractable to the Requirements Pillar (V07). | Data | P13 | Must | Foundation of any PM document |
| REQ-PM-042 | NFR Capture with Standard Taxonomy | The system SHALL provide an NFR capture form offering a selectable taxonomy of NFR types (ISO 25010 quality characteristics: performance efficiency, security, reliability, maintainability, portability, usability, compatibility, functional suitability) and SHALL allow PMs to enter measurable targets (e.g., "P95 API response < 200 ms at 1000 rps"). | Data | P13 | Must | ISO 25010 is the standard NFR taxonomy; measurable targets enable testability |
| REQ-PM-043 | Security Requirement Capture | The system SHALL provide a Security Requirements section in BRDs/PRDs with sub-categories aligned to OWASP Top 10 and ISO 27001 control categories (Authentication, Authorisation, Data Protection, Audit/Logging, Vulnerability Management, Incident Response, Encryption, Third-party Risk), with AI-suggested starters per sub-category. | Security | P13 | Must | Security requirements are consistently missing from PM-authored BRDs; alignment to OWASP/ISO 27001 makes them actionable for Security Architects |
| REQ-PM-044 | Performance Requirement Capture | The system SHALL provide a Performance Requirements section with structured fields for: throughput targets, latency/response-time SLAs (P50/P95/P99), concurrency/user load targets, data volume projections, and scalability expectations; AI SHALL suggest performance targets based on the product domain and comparable historical entries. | Data | P13 | Must | Performance requirements are typically missing or vague; structured capture enables testable SLAs |
| REQ-PM-045 | Compliance Requirement Capture | The system SHALL provide a Compliance/Regulatory Requirements section with AI-assisted identification of applicable regulations based on the product's industry vertical, geography, and data types (e.g., GDPR, HIPAA, PCI-DSS, SOX, ISO 27001, DORA, NIS2), pre-populating relevant regulatory stubs for PM confirmation. | Security | P13 | Must | Compliance gaps are costly; AI can infer regulatory scope from product profile |
| REQ-PM-046 | Requirement Import from Knowledge Hub | The system SHALL allow a PM to extract requirement candidates from Knowledge Hub artifacts (meeting transcripts, email threads, uploaded documents) using the AI engine (V06) and insert them as draft Requirement Cards into the active BRD/PRD with source citation preserved. | AI/ML | P13 | Must | Meeting transcripts and emails contain latent requirements; PM should not re-transcribe them manually |
| REQ-PM-047 | Requirement Traceability to V07 RTM | The system SHALL export all finalised Requirement Cards from a BRD/PRD into the Requirements Pillar (V07) Requirement Traceability Matrix (RTM), preserving the source BRD/PRD reference, section, and version as origin metadata. | Data | P13, EA | Must | V07 RTM is the cross-persona traceability system; PM requirements must feed it |

---

### 1.6 Product Roadmap (PM-owned)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PM-050 | PM-Owned Product Roadmap | The system SHALL support a PM-owned Product Roadmap entity (distinct from the EA-owned Architecture Roadmap in V09) as an instance of the platform roadmap model (V09), with PM-specific swimlane types: Product Theme, Epic, Feature, Release; owned and edited exclusively by P13 (with configured sharing to other personas). | Functional | P13 | Must | PM roadmap is a business-facing planning artefact; distinct from tech/architecture roadmap |
| REQ-PM-051 | Roadmap Item as BRD Carrier | The system SHALL allow each Product Roadmap item (theme/epic/feature/release) to carry one or more linked BRDs; the roadmap item status SHALL reflect the aggregate BRD approval status (e.g., "BRD pending" surfaced on roadmap item). | Functional | P13 | Must | BRD is the justification artefact for a roadmap item; linkage is the V17 core design principle |
| REQ-PM-052 | Release Plan View | The system SHALL provide a Release Plan view of the product roadmap showing: releases on a timeline, features assigned to each release, feature status, owner, BRD/PRD linkage, and completion percentage; filterable by theme, epic, and team. | Functional | P13 | Must | Release planning is a primary PM activity |
| REQ-PM-053 | Now–Next–Later View for PM | The system SHALL provide a Now–Next–Later horizon view of the product roadmap for high-level stakeholder communication, showing items in three horizons with configurable horizon window lengths and exportable as a shareable deck slide. | UX | P13 | Should | Standard executive-audience roadmap format |
| REQ-PM-054 | AI Roadmap Reshaping | The system SHALL invoke the AI engine to suggest roadmap reshaping actions given new BRD inputs, stakeholder feedback, changing priorities, or newly identified dependencies, presenting "consider moving X before Y because…" and "merge items A and B because they share scope" type recommendations. | AI/ML | P13 | Should | V09 roadmap AI applied to PM-persona context |
| REQ-PM-055 | Roadmap Share & Publish | The system SHALL allow a PM to publish a product roadmap as a shareable, read-only view accessible to: (a) Viewer-tier users (free, no login required for public roadmaps), (b) authenticated users with Reviewer or higher license, with configurable detail level (full vs. summary). | Functional | P13 | Must | Stakeholder transparency; aligns with Viewer/Reviewer tier definitions |

---

### 1.7 Product Backlog — Epics, Features & Stories

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PM-060 | Backlog Entity Hierarchy | The system SHALL maintain a Product Backlog as a hierarchical structure: Product → Epic → Feature → User Story → Sub-task, each with: unique ID, title, description, acceptance criteria, status, owner, sprint assignment (optional), story-point estimate, priority score, labels/tags, and linked BRD/PRD section. | Data | P13 | Must | Standard agile product backlog hierarchy; enables ESIP to serve as a lightweight PDM |
| REQ-PM-061 | Backlog Board View | The system SHALL provide a Kanban-style Backlog Board view with configurable columns (e.g., Backlog/Ready/In-Progress/Done or custom states) showing Epic, Feature, and Story cards with drag-and-drop prioritisation. | UX | P13 | Must | Visual backlog management is table-stakes for a PM tool |
| REQ-PM-062 | Backlog List View with Bulk Edit | The system SHALL provide a flat list/table view of backlog items with multi-select, bulk priority assignment, bulk status update, bulk label assignment, and column-level sort/filter. | UX | P13 | Must | Bulk operations are required for backlog refinement sessions |
| REQ-PM-063 | AI Story Generation from BRD/PRD | The system SHALL invoke the AI engine to generate a set of draft User Stories from a BRD section or PRD feature entry on demand, producing As a/I want/So that statements with acceptance criteria, inserting them into the backlog as Draft stories linked to the source BRD/PRD section. | AI/ML | P13 | Must | Story generation from requirements is a high-value AI automation; prevents manual transcription |
| REQ-PM-064 | Acceptance Criteria Validation | The system SHALL evaluate acceptance criteria on any backlog item using the AI engine and surface a completeness/quality score with suggestions (Given/When/Then format, testability, measurability) inline in the backlog item editor. | AI/ML | P13 | Should | AC quality gates prevent ambiguous stories reaching engineering |
| REQ-PM-065 | Sprint / Iteration Planning | The system SHALL support sprint/iteration planning by allowing a PM (or Scrum Master) to assign backlog stories to named sprints with capacity limits, drag-and-drop story assignment, burn-down velocity projection, and a sprint goal field. | Functional | P13 | Should | Lightweight sprint planning removes the need for a separate Jira-like tool for smaller teams |
| REQ-PM-066 | External Jira/ADO Sync | The system SHALL bidirectionally synchronise Product Backlog items with linked Jira projects or Azure DevOps Boards (via V08 connectors), mapping Epic↔Epic, Feature↔Feature/User Story↔Story, preserving ESIP fields not present in the target system as labels/custom fields. | Integration | P13 | Should | Most engineering teams use Jira/ADO; PM should not maintain two backlogs |

---

### 1.8 Prioritisation Frameworks (RICE / Kano / WSJF / MoSCoW)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PM-070 | Multi-Framework Prioritisation Engine | The system SHALL provide a Prioritisation Engine that supports four frameworks concurrently for any product backlog: RICE (Reach × Impact × Confidence / Effort), Kano (Must-Have/Performance/Delighter/Indifferent/Reverse), WSJF (Cost of Delay / Job Duration), and MoSCoW (Must/Should/Could/Won't); each framework produces its own score column on the backlog list view. | Functional | P13 | Must | Each framework serves a different prioritisation context; enterprise PMs use multiple simultaneously |
| REQ-PM-071 | RICE Scoring Form | The system SHALL provide an inline RICE scoring form per backlog item with: Reach (numeric), Impact (1–3 scale with labels), Confidence (%, slider), Effort (person-months or story-points), computing RICE score automatically and updating the backlog sort order in real time. | Functional | P13 | Must | RICE is the de-facto PM scoring standard (Intercom methodology) |
| REQ-PM-072 | Kano Categorisation | The system SHALL allow a PM to assign a Kano category (Must-Have, Performance, Delighter, Indifferent, Reverse) to each feature/story, display the Kano model quadrant visualisation, and filter/sort backlog by Kano category. | Functional | P13 | Must | Kano is essential for feature investment decisions in product-led growth contexts |
| REQ-PM-073 | WSJF Calculation | The system SHALL compute WSJF = Cost of Delay / Job Duration for each backlog item; Cost of Delay SHALL decompose into: User/Business Value, Time Criticality, and Risk Reduction/Opportunity Enablement, each scored on a Fibonacci-relative scale, consistent with SAFe WSJF practice. | Functional | P13 | Must | WSJF is the standard prioritisation method in SAFe; required for enterprise PI planning integration |
| REQ-PM-074 | AI Prioritisation Suggestions | The system SHALL invoke the AI engine to suggest prioritisation adjustments to a backlog, reasoning over: strategic alignment of each item to stated product objectives, completeness of acceptance criteria (items with poor AC deprioritised), market/customer signals from Knowledge Hub evidence, and dependency ordering; suggestions presented as a ranked list with rationale. | AI/ML | P13 | Should | AI prioritisation recommendation is a differentiator over pure spreadsheet scoring |
| REQ-PM-075 | Prioritisation Comparison View | The system SHALL provide a side-by-side comparison table showing all four framework scores per item simultaneously, with configurable weighting to produce a composite priority rank, exportable to Excel/CSV. | Functional | P13 | Should | PMs want a single synthesised view after running multiple frameworks |
| REQ-PM-076 | Priority History & Audit | The system SHALL log every change to a backlog item's priority scores (who changed, from what to what, when) as an immutable audit record, viewable per item and exportable for governance review. | Governance | P13 | Must | Priority decisions must be auditable in regulated enterprises |

---

### 1.9 BRD → Roadmap Item → Architecture Handoff

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PM-080 | Handoff Package Creation | The system SHALL allow a PM to create a formal Architecture Handoff Package from an approved BRD/PRD linked roadmap item, comprising: the approved BRD (frozen at approval version), the PRD (if exists), all associated Requirement Cards, the relevant backlog epics/features, and AI-generated architecture guidance starters (key NFRs, suggested architecture questions, recommended architect personas to engage). | Functional | P13 | Must | Formalises the transition from PM to architecture phase; prevents verbal/email handoffs |
| REQ-PM-081 | Architect Assignment to Handoff | The system SHALL allow a PM to assign one or more architect personas (P1–P12) to a Handoff Package, triggering notifications to those users, creating a linked architecture work item in the architecture personas' queues, and tracking architect acceptance. | Functional | P13, EA | Must | Handoff is incomplete without formal architect assignment and acknowledgement |
| REQ-PM-082 | Handoff Status Tracking | The system SHALL track the Handoff Package through statuses: Pending Assignment → Accepted by Architect(s) → Architecture In Progress → Architecture Complete → PM Review → Closed; PM SHALL see this status on the roadmap item and receive notifications on status changes. | Functional | P13 | Must | Visibility into architecture progress is a core PM need |
| REQ-PM-083 | Bidirectional Linkage Integrity | The system SHALL maintain bidirectional referential integrity between: BRD → PRD → Roadmap Item → Handoff Package → Architecture Artefacts (V01–V12 deliverables); a broken link (e.g., deleted BRD) SHALL surface as an integrity warning on all downstream items. | Data | P13, EA | Must | Traceability chain must not silently break |
| REQ-PM-084 | Architecture Feedback to PM | The system SHALL allow architect personas to post structured feedback on a Handoff Package (requirement gap identified, clarification needed, conflicting requirements, NFR target infeasible), triggering PM notification and creating a tracked resolution thread. | Functional | P13, P1–P12 | Must | Architects regularly discover gaps; formal feedback channel prevents ad-hoc communication loss |
| REQ-PM-085 | Impact Change Notification | The system SHALL notify assigned architect(s) when a PM changes a BRD/PRD that is part of an active or accepted Handoff Package, summarising the diff and requesting architect acknowledgement of the change, preventing architects from working from stale requirements. | Functional | P13, P1–P12 | Must | Requirement changes during architecture design are a major source of rework |

---

### 1.10 PM Dashboards & Metrics

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PM-090 | PM Executive Dashboard | The system SHALL provide a PM Executive Dashboard displaying: portfolio summary (total products, active BRDs/PRDs, approved vs. pending), roadmap milestone health (on-track/at-risk/delayed counts), backlog health (unscored items, items without AC, coverage completeness across categories), and handoff status summary. | Reporting | P13 | Must | PMs need a single glass-pane view of portfolio health |
| REQ-PM-091 | Requirement Coverage Metrics | The system SHALL display per-product and per-BRD/PRD metrics: % of backlog items with acceptance criteria, % of requirement categories with at least one requirement, % of backlog items with RICE/Kano/WSJF scores, and NFR coverage score (sub-categories present vs. total). | Reporting | P13 | Must | Makes completeness gaps actionable and comparable across products |
| REQ-PM-092 | Velocity & Throughput Tracking | The system SHALL track and display: BRD authoring-to-approval cycle time (avg/P50/P95), handoff-to-architect-accepted time, architecture completion time by product/epic, and backlog delivery throughput (stories per sprint); trends over time displayed as line charts. | Reporting | P13 | Should | Process metrics enable PM and organisation to improve practice over time |
| REQ-PM-093 | Stakeholder Sign-off Tracker | The system SHALL provide a Stakeholder Sign-off Tracker per BRD/PRD showing: each required sign-off party, their role, status (Pending/Approved/Changes-Requested), timestamp, and comments; escalation trigger when any sign-off is outstanding beyond a configurable SLA. | Functional | P13 | Must | Sign-off tracking is a governance requirement in enterprise BRD practice |
| REQ-PM-094 | AI Insight Cards on Dashboard | The system SHALL invoke the AI engine to generate weekly/on-demand Insight Cards on the PM dashboard, surfacing: stalled BRDs, roadmap items with no BRD, features missing NFRs, backlogs with high WSJF items not in next sprint, and Knowledge Hub artifacts that should be incorporated into open BRDs. | AI/ML | P13 | Should | Proactive AI insights replace manual weekly PM status review |

---

### 1.11 PM Deliverables Library

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PM-100 | BRD Export | The system SHALL export any BRD as: formatted Word (.docx), PDF, and structured JSON (for API/integration consumption), preserving all sections, requirement cards, tables, embedded diagrams, and version/approval metadata in the export. | Functional | P13 | Must | BRDs are shared with stakeholders who do not use ESIP |
| REQ-PM-101 | PRD Export | The system SHALL export any PRD as: formatted Word (.docx), PDF, and JSON, with embedded feature table, user stories, and NFR sections rendered faithfully including all acceptance criteria. | Functional | P13 | Must | Parity with BRD export |
| REQ-PM-102 | Product Roadmap Export | The system SHALL export a product roadmap as: PNG/SVG image, PDF, PowerPoint slide (single slide with roadmap visual), and JSON; the PPT export SHALL use the tenant brand theme (logo, colours, fonts). | Functional | P13 | Must | Roadmap is a stakeholder presentation artefact |
| REQ-PM-103 | Release Notes Generation | The system SHALL generate Release Notes for any release from the linked backlog stories/features (status=Done), AI-summarising each item in user-friendly language, grouping by feature/epic, and exporting as Markdown, HTML, or Word. | AI/ML | P13 | Should | Automates a time-consuming PM task; reduces copy-paste from Jira |
| REQ-PM-104 | Executive Summary Slide Deck | The system SHALL generate an AI-authored Executive Summary slide deck (PowerPoint, 5–10 slides) from a BRD/PRD on demand, covering: product vision, top 5 objectives, key features, timeline, risks, and success metrics; slides use the tenant brand theme. | AI/ML | P13 | Should | PMs spend significant time converting BRD content to executive slides; AI automation is high-value |

---

## 2. REQ-LIC — License Tiers

> **Reconciliation note (ownership — license tiers):** **V17 is CANONICAL for the Viewer, Reviewer, and Product Manager tier *capability definitions*** (what each tier can see/do, capability gates, tier-interaction rules). **Billing and enforcement → V13** (seat metering, subscription enforcement internals, payment, seat-pool true-up). V13's corresponding edition/license-type rows are stubs that **cross-reference these REQ-LIC-* definitions**; do not duplicate the capability semantics in V13. Resolves the FINALIZATION-REPORT "SPLIT license tiers" action.

### Overview
This sub-area extends Volume 13's edition/licensing model by defining two new **free** tiers (Viewer, Reviewer) and the **Product Manager** license, alongside the existing architect-persona licenses. Free tiers are designed to maximise stakeholder reach without requiring paid seats.

**Boundary:** Billing mechanics, seat metering, subscription enforcement internals, and payment processing → V13. The tier definitions, capability gates, and interaction rules are specified here (canonical).

---

### 2.1 Tier Definitions & Scope

| Tier | Cost | Target User | Core Access |
|------|------|-------------|-------------|
| **Viewer** | Free, no login required (for public shares) or with login (for internal shares) | External stakeholders, executives, read-only consumers | View published/shared end results only — roadmaps, deliverables, dashboards |
| **Reviewer** | Free, login required | Reviewers, approvers, SMEs, stakeholders who need to comment | View + comment/annotate/vote on review cycles (V15); cannot author |
| **Product Manager** | Paid named seat | P13 users | Full PM persona capabilities (all REQ-PM-*) |
| **Architect (P1–P12)** | Paid named seat (existing V13 model) | Architecture personas | Per-persona capabilities as defined in V05 |
| **Enterprise Architect** | Paid (typically higher tier) | P1 EA, senior | EA persona including portfolio, roadmap, multi-workspace |
| **Admin** | Bundled with paid seats | Tenant/User/Config Admins | All admin config capabilities per V13 |

---

### 2.2 Viewer Tier (Free)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-LIC-001 | Viewer Tier Definition | The system SHALL define a Viewer license tier that grants read-only access to explicitly published or shared ESIP artefacts (product roadmaps, architecture deliverables, requirement summaries, dashboards); no authoring, no commenting, no review actions are permitted under this tier. | Functional | Viewer | Must | Free tier broadens reach of ESIP outputs to non-technical stakeholders without licence cost |
| REQ-LIC-002 | Anonymous Viewer Access | The system SHALL support anonymous (no-login) Viewer access to artefacts the PM or architect has published with a public share link, subject to tenant-level configuration enabling/disabling anonymous access; tenant admins MAY restrict all Viewer access to authenticated-only. | Security | Viewer | Must | External stakeholders (clients, partners) should be able to view public roadmaps without ESIP accounts |
| REQ-LIC-003 | Authenticated Viewer | The system SHALL support authenticated Viewer access (login required) for internal stakeholders who need to view internal-only shared artefacts; authenticated Viewers SHALL receive a free seat that does not count against paid seat quotas. | Functional | Viewer | Must | Internal executives/stakeholders should not need a paid seat to view a roadmap |
| REQ-LIC-004 | Viewer Scope Restriction | The system SHALL ensure a Viewer can only access artefacts that have been explicitly published/shared to them or to the public; no Viewer SHALL be able to navigate to unpublished projects, workspace listings, or any authoring interface. | Security | Viewer | Must | Viewer must never see work-in-progress or confidential unpublished content |
| REQ-LIC-005 | Viewer Read-Only Guarantee | The system SHALL enforce at the API level that any authenticated or anonymous request from a Viewer session cannot invoke any state-changing operation (create, update, delete, comment, approve, react); attempts SHALL return HTTP 403 with a clear upgrade prompt. | Security | Viewer | Must | API-level enforcement prevents UI bypass |
| REQ-LIC-006 | Viewer Watermark | The system SHALL display a non-removable "Powered by ESIP" watermark or branding badge on all artefacts viewed by anonymous Viewers unless the tenant is on an Enterprise plan with white-label rights. | Functional | Viewer | Should | Virality/brand awareness; standard for free tiers in SaaS |

---

### 2.3 Reviewer Tier (Free)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-LIC-010 | Reviewer Tier Definition | The system SHALL define a Reviewer license tier (login required, free seat) that grants: view access to all artefacts a Viewer can see PLUS the ability to post comments, annotations, reactions, and review responses on artefacts via the Review Designer (V15); Reviewers cannot author artefacts, manage backlogs, run AI actions, or access admin functions. | Functional | Reviewer | Must | Extends Viewer with review participation; enables SME/stakeholder input without paid seats |
| REQ-LIC-011 | Reviewer Seat Count | The system SHALL NOT count Reviewer seats against any paid seat quota; Reviewer seats SHALL be unlimited per tenant but subject to tenant admin configuration of a maximum cap if desired. | Functional | Admin | Must | Unlimited free Reviewer seats incentivises stakeholder engagement; paid seats are for creators/authors only |
| REQ-LIC-012 | Reviewer Comment Capabilities | The system SHALL allow a Reviewer to: post threaded comments on any shared artefact section, annotate diagrams with sticky-note markers, react with emoji/approval signals, respond to review requests, mark their own comments as resolved, and @mention any authenticated user in their comment. | Functional | Reviewer | Must | Core review participation capability; aligns with V15 Review Designer |
| REQ-LIC-013 | Reviewer Access to Review Cycles | The system SHALL allow a Reviewer to participate fully in any review cycle (V15) to which they have been explicitly invited (by a PM or architect with a paid seat), including casting approve/reject/abstain votes and completing assigned review tasks. | Functional | Reviewer | Must | Formal review participation is the primary use case for this tier |
| REQ-LIC-014 | Reviewer Cannot Author | The system SHALL prevent Reviewer-tier users from: creating any artefact (BRD, PRD, diagram, requirement, roadmap item, backlog item, Knowledge Hub artifact); running any AI action; accessing any workspace settings; or modifying any artefact except their own comment text. | Security | Reviewer | Must | Clear capability boundary prevents tier confusion |
| REQ-LIC-015 | Reviewer Upgrade Path | The system SHALL display an in-app upgrade prompt to Reviewer-tier users when they attempt an action that requires a paid seat (authoring, AI actions, etc.), linking to the tenant's billing page or a contact-sales page depending on tenant configuration. | UX | Reviewer | Must | Reviewers who want to author must be guided to upgrade |
| REQ-LIC-016 | Reviewer Activity Feed | The system SHALL provide each Reviewer with a personalised Activity Feed showing: new artefacts shared with them, review cycle invitations, replies to their comments, and artefact status changes on things they have commented on. | UX | Reviewer | Should | Keeps Reviewers engaged and aware of progress without requiring them to poll |

---

### 2.4 Product Manager License

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-LIC-020 | PM License Definition | The system SHALL define a Product Manager (P13) license as a paid named seat granting access to all capabilities defined under REQ-PM-* in this volume, including: BRD/PRD authoring, product roadmap ownership, backlog management, prioritisation engines, AI assist actions, PM dashboards, and the Architecture Handoff module; it SHALL NOT grant access to architect-persona tooling (V01–V12 capabilities) unless the user additionally holds an architect persona license. | Functional | Admin, P13 | Must | PM is a distinct paid tier from architects; clear capability boundary |
| REQ-LIC-021 | PM + Architect Combo Seat | The system SHALL support assigning both a PM license and one or more architect-persona licenses to a single user, enabling "player-coach" roles where one person authors BRDs AND performs architecture design; combined licensing is additive in capability and billed at the sum of the component seat costs (or a configured bundle discount). | Functional | Admin | Must | Senior architects frequently also perform PM functions; a single user identity must not require two accounts |
| REQ-LIC-022 | PM Seat Pool | The system SHALL manage PM licenses as a configurable named-seat pool (e.g., "10 PM seats"), enforcing that no more than the pooled number of users may hold an active PM license simultaneously; the pool size is set during tenant onboarding or via billing admin (V13). | Functional | Admin | Must | Seat pool enforcement is a standard SaaS licensing mechanism |
| REQ-LIC-023 | PM License Capability Gating | The system SHALL gate all PM-specific API endpoints and UI actions behind PM license validation at the platform middleware layer, returning HTTP 403 with upgrade messaging for non-PM-licensed users who attempt PM actions. | Security | All | Must | API-level enforcement prevents UI bypass; consistent with V13 licensing enforcement |

---

### 2.5 Tier Interactions, Upgrade Paths & Rules

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-LIC-030 | Tier Hierarchy | The system SHALL enforce a capability hierarchy: Viewer (lowest) < Reviewer < PM < Architect-Persona < Admin (highest), where each tier is a strict superset of all lower-tier capabilities with respect to access to shared/published artefacts. | Functional | All | Must | Prevents confusion about what a higher tier can always do what a lower one can |
| REQ-LIC-031 | Self-Service Upgrade | The system SHALL allow a Viewer or Reviewer to initiate a seat upgrade request from any in-app upgrade prompt; the request SHALL: (a) in self-serve tenants — redirect to the billing portal for immediate purchase, (b) in enterprise tenants — submit an email notification to the tenant's User Admin for manual license assignment. | Functional | Viewer, Reviewer | Must | Smooth upgrade path increases conversion |
| REQ-LIC-032 | License Revocation & Downgrade | The system SHALL allow a User Admin to revoke a PM or architect license from a user, downgrading them to Reviewer tier (retaining all their comments and review history) or Viewer tier (stripping comment/review rights), without deleting any content they authored. | Functional | Admin | Must | Licence management must not destroy content when seats are reassigned |
| REQ-LIC-033 | Free Tier Interaction with Published Artefacts | The system SHALL ensure that Viewer and Reviewer users can only interact with artefacts the PM/architect has explicitly published or shared; sharing SHALL be configurable as: (a) public link, (b) specific email addresses, (c) all authenticated users in the tenant, (d) all Reviewer+ users in the tenant. | Security | Admin, P13 | Must | Sharing model must be granular; default MUST be non-public |
| REQ-LIC-034 | Licence Usage Reporting | The system SHALL provide real-time licence usage reports to Billing Admins and User Admins showing: total allocated seats per tier, active seats per tier, Reviewer and Viewer monthly-active-user counts (even though free, for capacity planning), and peak concurrent usage per paid tier. | Reporting | Admin | Must | Capacity planning and true-up billing require usage visibility |

---

### 2.6 Free-Tier Governance & Limits

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-LIC-040 | Viewer Storage Exemption | The system SHALL NOT bill or meter storage consumed by artefacts viewed by Viewer-tier users; Viewer access to a shared artefact SHALL NOT create a copy or incur storage charges for the tenant. | Functional | Admin | Must | Free tier must not create hidden cost surprise |
| REQ-LIC-041 | Reviewer Comment Retention | The system SHALL retain comments posted by Reviewer-tier users for the lifetime of the review cycle or artefact version to which they are attached, and SHALL NOT delete Reviewer comments if the Reviewer's account is downgraded to Viewer. | Data | Admin | Must | Comments are evidence artefacts; their loss would break review traceability |
| REQ-LIC-042 | Rate Limiting for Free Tiers | The system SHALL apply per-session API rate limits to anonymous Viewer sessions (e.g., 60 requests/minute) to prevent abuse of the free tier for data scraping; authenticated Viewer and Reviewer tiers SHALL have higher rate limits (configurable). | Security | Admin | Must | Free tiers must be protected against scraping/abuse |
| REQ-LIC-043 | SSO Applicability to Free Tiers | The system SHALL enforce tenant-configured SSO policies for authenticated Viewer and Reviewer logins (if the tenant requires SSO for all authenticated access); anonymous Viewers are exempt from SSO by definition. | Security | Admin | Must | Enterprise tenants may require SSO even for free-tier users to maintain identity governance |

---

## 3. REQ-AGT — Code-Scanner Agent Provisioning

> **Reconciliation note (boundary — scanner agents):** **V17 owns the scanner-agent *lifecycle*** — provisioning, install/registration, pools, health/heartbeat, auto-update, scan-target configuration, data-boundary controls, and the operational/security envelope (all REQ-AGT-*). The **scan *analytics*, flow-derivation, SBOM/SAST outputs, and the application-portfolio (APM) model the scan feeds → V09**. V17 streams derived results into V09; it does not own the portfolio data model or scan reasoning. Resolves the FINALIZATION-REPORT "Code-scan boundary" action.

### Overview
This sub-area specifies the **agent provisioning model** for the Rust code-scanner, following an Azure DevOps self-hosted build-agent pattern. The Enterprise Architect (or designated admin) provisions agents in the client's environment, configures scan targets, and controls what data crosses the client boundary. The agent runs the Rust scanner locally, enriches the portfolio (V09) with derived architectural intelligence, and operates in air-gapped environments. Code-scan parsing/reasoning/portfolio model → V09; agent provisioning and operational model → this volume.

---

### 3.1 Agent Model & Architecture

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-AGT-001 | Agent Entity Model | The system SHALL maintain an Agent entity per installed scanner agent, with: unique agent ID (UUID), display name, description, tenant ID, agent pool assignment, registration token hash (not plaintext), OS/platform (Linux/Windows/macOS), agent version, Rust scanner version, current status (Idle/Running/Offline/Degraded/Updating), last heartbeat timestamp, install path, and capabilities list. | Data | EA, Admin | Must | Agent registry is the source of truth for all installed agents |
| REQ-AGT-002 | Agent Architecture Pattern | The system SHALL implement the code-scanner agent using a **pull-based job queue model** (agent polls or maintains long-poll/websocket connection to the ESIP control plane for scan jobs), avoiding the need for inbound network connectivity from ESIP to the client environment; all communication SHALL be agent-initiated outbound only. | NFR | EA | Must | Pull-model is the Azure DevOps self-hosted agent pattern; works behind NAT/firewall without firewall rule changes; mandatory for air-gapped compatibility |
| REQ-AGT-003 | Agent Binary Packaging | The system SHALL package the code-scanner agent as a single self-contained executable per supported platform (Linux x86_64, Linux aarch64, Windows x86_64, macOS x86_64, macOS arm64), including the Rust scanner runtime, without requiring any language runtime (JVM, .NET, Python) pre-installed on the host. | NFR | Admin | Must | Zero-dependency install is mandatory for enterprise environments with locked-down OS configurations |
| REQ-AGT-004 | Agent as OS Service | The system SHALL support installing the agent as an OS-level service (Linux: systemd unit; Windows: Windows Service) via an included install script, enabling automatic startup on host reboot without manual intervention. | Functional | Admin | Must | Production agents must survive host restarts; manual restart is not acceptable |

---

### 3.2 Agent Download, Install & Registration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-AGT-010 | Agent Download from Admin Portal | The system SHALL provide a per-platform agent binary download in the ESIP admin portal (Tenant Admin or EA with delegated agent-admin permission), showing available versions, release notes, SHA-256 checksum, and a signed binary certificate for integrity verification. | Functional | Admin, EA | Must | Signed binaries are a security requirement for enterprise software distribution |
| REQ-AGT-011 | One-Time Registration Token | The system SHALL generate a one-time-use, time-limited (configurable, default 1 hour) registration token (minimum 256-bit entropy) that the admin copies from the portal and passes to the agent installer; the token authenticates the initial agent registration and is invalidated immediately upon use. | Security | Admin | Must | One-time token prevents token reuse or interception replay; aligns with Azure DevOps PAT-based agent registration model |
| REQ-AGT-012 | Mutual TLS Agent Authentication | The system SHALL issue a unique TLS client certificate to each agent during registration (client certificate provisioned by the ESIP CA or a tenant-supplied CA), used for all subsequent authenticated communication between the agent and the ESIP control plane after the initial registration handshake. | Security | Admin | Must | mTLS provides cryptographically strong per-agent authentication, replacing shared secrets post-registration |
| REQ-AGT-013 | Registration Confirmation | The system SHALL confirm successful agent registration in the admin portal in real time, displaying: agent ID, display name, assigned pool, OS platform, agent version, and first-heartbeat timestamp within 60 seconds of registration completion. | UX | Admin | Must | Immediate confirmation reduces admin uncertainty about whether registration succeeded |
| REQ-AGT-014 | Air-Gap Registration via Bundle | The system SHALL support an offline registration bundle mode: admin generates a registration bundle (JSON + certificates) in the portal, transfers it to the air-gapped environment via removable media, and the agent installer consumes the bundle to register without outbound internet access during installation; subsequent communication uses a pre-configured internal proxy or relay endpoint. | Functional | Admin | Must | Air-gapped environments cannot reach the internet; offline registration is mandatory for compliance with classified/isolated network requirements |

---

### 3.3 Agent Pools & Capacity

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-AGT-020 | Agent Pool Model | The system SHALL support creating named Agent Pools grouping one or more agents, with: pool name, description, pool type (Default/Private/Shared), assigned tenant/workspaces, maximum concurrent jobs, and queue-depth watermark alerts. | Functional | Admin | Must | Pool model (Azure DevOps pattern) enables routing scan jobs to appropriate agent groups |
| REQ-AGT-021 | Pool-to-Scan-Target Assignment | The system SHALL allow an Enterprise Architect to assign scan target configurations to a named agent pool, ensuring scan jobs for those targets are routed only to agents in that pool; this enables environment segmentation (e.g., a pool for "Production Repos" vs. "Dev Repos"). | Functional | EA, Admin | Must | Security isolation: production repo scans should not run on dev-environment agents |
| REQ-AGT-022 | Default Pool | The system SHALL provision a Default agent pool per tenant automatically; all newly registered agents are assigned to the Default pool unless explicitly re-assigned. | Functional | Admin | Must | Default pool ensures agents are usable immediately post-registration without manual pool setup |
| REQ-AGT-023 | Agent Capacity Scaling | The system SHALL support horizontal scaling of agent capacity within a pool by adding agents; the ESIP scheduler SHALL distribute queued scan jobs across idle agents in the pool using a fair-work-stealing algorithm, with configurable max-concurrent-jobs-per-agent. | NFR | Admin | Should | Multi-repo portfolios generate large scan job queues; parallelism across agents is required |

---

### 3.4 Health, Heartbeat & Logs

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-AGT-030 | Agent Heartbeat | The system SHALL require each registered agent to emit a heartbeat to the ESIP control plane at a configurable interval (default: every 30 seconds); if a heartbeat is not received within 3× the configured interval, the agent status SHALL transition to Degraded; after 5× the interval, to Offline, and an alert SHALL be raised to the Admin. | NFR | Admin | Must | Heartbeat is the standard availability signal; offline detection is mandatory for agent-based architectures |
| REQ-AGT-031 | Heartbeat Payload | The system SHALL include in each agent heartbeat: agent version, Rust scanner version, CPU utilisation %, available disk space (GB), current job ID (if running), job queue depth, last successful scan timestamp, and accumulated error count since last restart. | Data | Admin | Must | Rich heartbeat enables proactive capacity and health management without a separate APM agent |
| REQ-AGT-032 | Agent Log Streaming | The system SHALL allow authorised admins to request a live log stream from any agent (last N lines or tailing mode) via the admin portal, with log content transmitted over the existing mTLS channel; logs SHALL include: scan job start/end events, errors, warnings, and agent lifecycle events. | Functional | Admin | Must | Remote log access is critical for diagnosing scan failures in client environments where admins lack SSH access |
| REQ-AGT-033 | Log Retention at Agent | The system SHALL retain agent-side logs for a configurable rolling window (default: 14 days, max: 90 days), rotating to prevent unbounded disk growth; log verbosity (ERROR/WARN/INFO/DEBUG/TRACE) SHALL be configurable via the admin portal and applied remotely without agent restart. | NFR | Admin | Should | Log retention enables post-mortem analysis of past scan failures |
| REQ-AGT-034 | Health Dashboard | The system SHALL provide an Agent Health Dashboard in the admin portal displaying all agents across all pools with: status indicator, last heartbeat, current job, agent/scanner version, disk/CPU gauges, and alert badges; refreshing in real time at 10-second intervals. | UX | Admin, EA | Must | Single-pane health visibility is table-stakes for agent fleet management |

---

### 3.5 Auto-Update & Version Management

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-AGT-040 | Auto-Update Capability | The system SHALL support automatic agent and Rust scanner binary updates via the control plane: the ESIP platform publishes a new version, agents check for updates at a configurable schedule, download the new binary over the mTLS channel, verify the SHA-256 + signature, swap the binary using atomic replace, and restart as an OS service; the update SHALL occur only when the agent is idle (no active scan job). | Functional | Admin | Must | Manual binary updates across client fleets are impractical; auto-update is an operational requirement |
| REQ-AGT-041 | Update Policy per Pool | The system SHALL allow admins to configure an update policy per agent pool: (a) Auto — update immediately when idle after a new version is published, (b) Scheduled — update during a configured maintenance window, (c) Manual — admin triggers update per agent from the portal; default is Scheduled. | Functional | Admin | Must | Enterprise clients require control over update timing to avoid disruption to scheduled scans |
| REQ-AGT-042 | Rollback Capability | The system SHALL support rolling back an agent to the previous version from the admin portal (triggering an update-to-prior-version job on the agent) if a new version is found to cause issues; rollback SHALL complete within 5 minutes. | Functional | Admin | Must | Rollback prevents fleet-wide failures from a bad update |
| REQ-AGT-043 | Air-Gap Update Bundle | The system SHALL support an offline update mode for air-gapped agents: admin downloads an update bundle (new binary + signature + metadata) from the ESIP portal on an internet-connected machine, transfers it to the air-gapped host, and the agent applies the update locally after verifying signature. | Functional | Admin | Should | Air-gapped agents cannot auto-update over the internet; offline update bundle is the only path |

---

### 3.6 EA-Configured Scan Targets

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-AGT-050 | Scan Target Entity Model | The system SHALL allow an Enterprise Architect (or delegated user with scan-admin permission) to define Scan Target records, each containing: unique ID, display name, target type (Git Repository URL / Local Folder Path / Git Folder Sub-path within monorepo), authentication method (SSH key / HTTPS with token / service account credential / none for local folder), clone/access location on the agent host, target branch/tag/ref, file-type include/exclude patterns, and assignment to an agent pool. | Data | EA | Must | Flexible target model supports cloud repos, self-hosted Git, and air-gapped local folder paths |
| REQ-AGT-051 | Repository URL Target | The system SHALL support scan targets that are Git repository URLs (HTTPS or SSH), with stored credentials resolved from the ESIP secrets vault (V11), used by the agent to clone or pull the repository to a configured local clone path before scanning. | Functional | EA | Must | Standard code-scan source for cloud-hosted and self-hosted Git (GitHub, GitLab, Bitbucket, Azure DevOps) |
| REQ-AGT-052 | Local Folder Target | The system SHALL support scan targets that are local folder paths (absolute path on the agent host, e.g., `/data/repos/my-service` or `C:\repos\my-service`), where the agent scans the folder in-place without cloning; this enables scanning of file-system-mounted repositories or locally checked-out code in air-gapped environments. | Functional | EA | Must | Air-gapped and on-premises environments often access code via file-system mounts, not remote Git |
| REQ-AGT-053 | Monorepo Sub-path Target | The system SHALL support defining a scan target as a sub-path within a cloned monorepo, allowing the EA to create multiple scan targets (e.g., `services/billing`, `services/auth`) from a single cloned repository, each scanned independently and producing separate portfolio entries. | Functional | EA | Must | Monorepos are prevalent; service-level granularity in portfolio enrichment requires sub-path targeting |
| REQ-AGT-054 | Scan Schedule Configuration | The system SHALL allow the EA to configure a scan schedule per target: (a) Manual/On-Demand trigger only, (b) Scheduled — cron expression (e.g., `0 2 * * *` for nightly at 2 AM), (c) On-Push Webhook — triggered by a webhook event from the Git host; the system SHALL display the next scheduled scan time and last scan time per target. | Functional | EA | Must | Continuous freshness requires scheduled scans; on-push enables near-real-time portfolio enrichment |
| REQ-AGT-055 | Webhook Registration | The system SHALL provide per-scan-target webhook endpoint URLs (tenant-isolated, authenticated) that the EA can register in the Git hosting system (GitHub/GitLab/Azure DevOps webhook settings) to trigger on-push scan jobs; the webhook payload SHALL be validated with an HMAC-SHA256 signature before dispatching a scan job. | Security | EA | Must | Signed webhooks prevent spoofed scan triggers |
| REQ-AGT-056 | Target Enable/Disable | The system SHALL allow the EA to enable or disable any scan target without deleting it; disabled targets do not generate scan jobs but retain all historical scan results and portfolio data. | Functional | EA | Must | Temporary disable (e.g., during repo migration) must not destroy scan history |

---

### 3.7 Clone, Scan Execution & Local Runner

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-AGT-060 | Clone/Pull on Scan Job | The system SHALL, for repository URL targets, perform a shallow clone (configurable depth, default: `--depth 1` of the target branch) or an incremental pull (if prior clone exists) to the configured clone location before invoking the Rust scanner; clone credentials SHALL be retrieved from the on-agent credential store (populated at registration or via secure remote config push). | Functional | Admin | Must | Shallow clone minimises storage and time; incremental pull maintains efficiency on repeat scans |
| REQ-AGT-061 | Rust Scanner Invocation | The system SHALL invoke the Rust code-scanner binary as a subprocess, passing: target path (repo root or sub-path), language hints, output format (NDJSON), include/exclude patterns, and a job correlation ID; the scanner SHALL produce structured output streamed to stdout which the agent consumes and processes. | Functional | Admin | Must | Scanner is a separate Rust binary; agent is the orchestrator/envelope not the scanner itself |
| REQ-AGT-062 | Scan Job Lifecycle | The system SHALL track each scan job through statuses: Queued → Assigned → Cloning → Scanning → Processing → Streaming → Completed | Failed, with start/end timestamps and error detail for each transition; status SHALL be visible in real time in the admin portal. | Data | Admin, EA | Must | Job lifecycle traceability is required for debugging and SLA measurement |
| REQ-AGT-063 | Incremental Scan | The system SHALL support incremental scanning: on subsequent scans of the same target, the agent SHALL compute the git diff since the last successful scan commit SHA, pass only changed files to the scanner, and merge incremental results with the baseline in the portfolio; a full rescan SHALL be triggerable on demand. | NFR | EA | Should | Full rescans of large repos are slow; incremental scanning enables frequent refresh without proportional cost |
| REQ-AGT-064 | Scan Concurrency Limit | The system SHALL enforce a configurable maximum number of concurrent scan jobs per agent (default: 2) to prevent resource starvation on the agent host; excess jobs SHALL queue in the ESIP job queue. | NFR | Admin | Must | Agent hosts have finite CPU/RAM; unconstrained concurrency would degrade scan quality |

---

### 3.8 Derived Results Streaming & Data Boundary Controls

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-AGT-070 | Data Boundary Configuration | The system SHALL provide a per-agent (or per-scan-target) configurable data boundary policy with two modes: (a) **Metadata-Only** — send only derived structural metadata (component names, dependency graph, API surface, tech stack, file counts, complexity metrics, vulnerability IDs) to ESIP without any source code content; (b) **Full Derived Results** — send metadata plus AI-ready excerpts (function signatures, data model schemas, API contracts) sufficient for AI reasoning in V09; raw source code lines SHALL NEVER be transmitted regardless of mode. | Security | Admin, EA | Must | Client enterprises will not tolerate raw source code leaving their boundary; metadata-only mode is a non-negotiable enterprise security requirement |
| REQ-AGT-071 | Streaming Protocol | The system SHALL stream derived scan results from the agent to the ESIP control plane as NDJSON over the mTLS-authenticated WebSocket connection in real time as the scanner produces output, rather than batching and sending at job completion; the control plane SHALL apply results incrementally to the portfolio. | Functional | Admin | Must | Streaming enables partial portfolio updates during long scans; failure mid-scan retains partial results |
| REQ-AGT-072 | Result Integrity Signature | The system SHALL sign each streamed result batch with an HMAC keyed to the agent's mTLS certificate private key material, enabling the control plane to verify that results were produced by a registered agent and not tampered with in transit. | Security | Admin | Must | Result integrity is required to trust AI-derived portfolio data; tampered results could corrupt architecture models |
| REQ-AGT-073 | Redaction Rules | The system SHALL allow tenant admins to configure redaction rules applied on-agent before transmission: path patterns to exclude (e.g., `**/secrets/**`, `**/.env`), symbol name patterns to mask, and file extension blocklists; redaction SHALL be applied to all streamed content before the mTLS transmission. | Security | Admin | Must | Clients may have files within scanned repos containing credentials or sensitive data even in non-source paths |
| REQ-AGT-074 | Transmission Audit Log | The system SHALL maintain an immutable per-job transmission audit record showing: job ID, target ID, agent ID, bytes transmitted, result record count, data boundary mode used, redaction rule version applied, start/end timestamp, and checksum of full transmitted payload; accessible to Admin and EA in the portal. | Governance | Admin | Must | Transmission audit provides evidence for client compliance teams that the data boundary policy was enforced |

---

### 3.9 Continuous Portfolio Enrichment

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-AGT-080 | Portfolio Enrichment from Scan | The system SHALL continuously enrich the Application Portfolio Management (V09) model with scan-derived artefacts on each successful scan completion: (a) system/application entity record (if new), (b) technology stack inventory, (c) dependency graph (internal + external), (d) API surface map, (e) data model schema fragments, (f) complexity and quality metrics, (g) candidate system flow diagram data, (h) application architecture component map. | Functional | EA | Must | Agent's primary purpose is continuous portfolio enrichment; each scan enrichment layer is traceable to the V09 portfolio model |
| REQ-AGT-081 | EA Foundation Artefacts | The system SHALL generate or update, for each scanned system, EA Foundation artefacts aligned to the TOGAF ADM (V01) and ArchiMate (V03) models: (a) Application Component record in the ArchiMate Application Layer, (b) Data Object references for inferred data entities, (c) Technology Component references for tech-stack entries, (d) candidate Application Cooperation diagram showing the system and its integration points. | Functional | EA | Must | EA Foundation alignment is what elevates the scanner from a code tool to an architecture intelligence source |
| REQ-AGT-082 | Conflict Detection on Enrichment | The system SHALL detect conflicts between scanner-derived data and manually maintained portfolio entries (e.g., EA recorded Java, scanner detected Kotlin; EA recorded microservices, scanner detected monolith), and SHALL surface these as Portfolio Conflict alerts for EA review rather than silently overwriting manual data. | Functional | EA | Must | Manual EA knowledge takes precedence; scanner enriches, does not overwrite without review |
| REQ-AGT-083 | Enrichment History | The system SHALL retain a full time-series history of all scanner-derived values per portfolio item (e.g., dependency count over time, complexity trend, tech-debt score trend), enabling the EA to view how portfolio properties evolved across scans. | Data | EA | Should | Portfolio evolution visibility is a core EA intelligence feature |

---

### 3.10 Air-Gapped Deployment

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-AGT-090 | Air-Gap Mode | The system SHALL support a fully air-gapped agent deployment mode where the agent has no internet connectivity and communicates exclusively with an on-premises ESIP control plane instance (deployed in the client's private network); all features (registration, heartbeat, scanning, streaming, auto-update via bundle) SHALL function in air-gap mode. | Functional | Admin | Must | Government, defence, and regulated industries operate in air-gapped environments; loss of any feature in air-gap mode is unacceptable |
| REQ-AGT-091 | On-Premises Relay | The system SHALL support deployment of an on-premises ESIP Agent Relay service (a lightweight .NET Core 10 service) that acts as a TLS-terminating reverse proxy between the air-gapped agent and the ESIP control plane hosted elsewhere; the relay SHALL require no persistent state and SHALL be deployable as a Docker container or systemd service. | Functional | Admin | Should | Some air-gapped scenarios require a relay for network segmentation compliance rather than direct agent-to-ESIP connectivity |
| REQ-AGT-092 | Air-Gap Bundle Distribution | The system SHALL document and support a bundle distribution workflow for air-gapped environments: all required artefacts (agent binary, CA certificate, registration bundle, AI model files if applicable) packaged into a single versioned .tar.gz or .zip archive downloadable from the internet-facing ESIP portal, ready for transfer via removable media. | Functional | Admin | Should | Standardised bundle reduces errors during air-gap deployment |

---

### 3.11 Multi-Agent Scaling

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-AGT-100 | Multi-Agent Job Distribution | The system SHALL distribute queued scan jobs across multiple agents in a pool using a priority-weighted queue (configurable job priority: Low/Normal/High/Critical) with fair-scheduling ensuring no single agent is starved; job affinity (prefer agents that previously cloned a given repo) SHALL be optionally configurable to leverage local clone cache. | NFR | Admin | Must | Large portfolios with hundreds of repos require parallel agent execution to maintain scan currency |
| REQ-AGT-101 | Agent-Level Job Timeout | The system SHALL enforce a configurable per-job timeout (default: 4 hours, configurable 15 minutes–24 hours); if a job exceeds the timeout, the agent SHALL mark it as timed-out, release the job lock, and the control plane SHALL re-queue it to another agent (up to a configurable retry count, default: 2). | NFR | Admin | Must | Stuck scans (e.g., infinite loops in scanner on malformed code) must not block the agent pool indefinitely |
| REQ-AGT-102 | Agent Fleet Metrics | The system SHALL expose aggregate fleet metrics (total agents, by-status counts, total jobs completed/failed/queued today, average scan duration by repo size bucket, P95 scan time) on the admin portal and as a metrics API endpoint (Prometheus-compatible) for integration with client monitoring systems. | Reporting | Admin | Should | Enterprise clients require ESIP agent metrics to feed into their existing Datadog/Grafana/Prometheus monitoring stacks |

---

### 3.12 Security & Isolation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-AGT-110 | Agent Process Isolation | The system SHALL run the Rust scanner binary as a subprocess in a separate OS process (not a thread), with configurable OS-level resource limits (CPU %, max RAM, max open files) via OS mechanisms (Linux cgroups/ulimit, Windows Job Objects), preventing a runaway scan from impacting the agent process or host OS. | Security | Admin | Must | Malformed code or adversarial repo content could trigger resource exhaustion in the scanner; OS-level isolation is the only safe boundary |
| REQ-AGT-111 | Credential Isolation | The system SHALL store all scan-target credentials (SSH keys, HTTPS tokens, service account credentials) in an on-agent encrypted credential store (key derived from the agent's mTLS private key + a hardware/TPM-bound secret if available), accessible only to the agent process user account; credentials SHALL never be logged or included in streamed results. | Security | Admin | Must | Credentials must not leak via logs, streamed results, or agent heap dumps |
| REQ-AGT-112 | Principle of Least Privilege | The system SHALL document and enforce that the agent OS service user account is granted only the minimum OS permissions required (read access to clone target paths, write access to clone scratch directory, network outbound only); the install script SHALL offer to create a dedicated low-privilege OS service account. | Security | Admin | Must | Least-privilege execution limits blast radius if the agent process is compromised |
| REQ-AGT-113 | Agent Revocation | The system SHALL allow an Admin to revoke any agent's mTLS certificate from the admin portal, immediately invalidating the agent's ability to communicate with the control plane; the revocation SHALL be propagated via the ESIP Certificate Revocation List (CRL) or OCSP within 60 seconds. | Security | Admin | Must | Revocation is required when an agent host is compromised or decommissioned |

---

## 4. REQ-RTC — Real-Time Meeting & Chat Connectors

> **Reconciliation note (boundary — Teams capture):** REQ-RTC **real-time / near-real-time live capture** (bot auto-join, in-meeting transcript streaming, webhook-triggered ingestion within minutes of meeting end) is **distinct from V08's post-meeting transcript pull**. V08 owns the base connector framework and scheduled post-meeting retrieval of completed transcripts; **V17 owns the live/real-time capture path** (REQ-RTC-010/011, REQ-RTC-042). The two MUST share a single Microsoft Graph / Zoom app registration and credential vault (V08/V11) — do not stand up a second Graph integration. Resolves the FINALIZATION-REPORT "Teams capture boundary" action.
>
> **Reconciliation note (C5 — air-gap exclusion):** The real-time meeting/chat connectors in this section — **Microsoft Teams, Outlook, and Zoom (all REQ-RTC-*)** — are **SaaS-only and air-gap-EXCLUDED** (they depend on Microsoft 365 / Zoom cloud APIs and inbound webhooks reachable from the public internet). In air-gapped deployments these connectors are unavailable; the documented offline fallback is **manual transcript/notes upload into the Knowledge Hub (V08)** followed by the same AI essence/decision/action-item extraction. This enumeration feeds the catalog-wide air-gap exclusion list in REQ-ENT-DEP-010 (V19) per §6 ruling 3.

### Overview
This sub-area adds **real-time and near-real-time auto-capture** of meeting transcripts, channel/group chat, email, and calendar data to the Knowledge Hub (V08), extending V08's base connector framework with auto-join/post-meeting processing, bot registration, and AI-powered essence extraction. Base connector framework, credential vault, OAuth flows, and connector job scheduling → V08/V11. AI summarisation/extraction reasoning → V06. This volume specifies the connector-specific capabilities, consent model, and Knowledge Hub integration.

---

### 4.1 Connector Registration & Consent Framework

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-RTC-001 | App Registration Workflow | The system SHALL guide tenant admins through a step-by-step App Registration Wizard for each real-time connector (Teams, Outlook, Zoom), covering: OAuth app creation in the target platform (deep-linked to the provider's developer portal), required permission scopes (displayed with plain-English explanations), admin consent grant, and connection test; the wizard SHALL validate all permissions before marking the connector active. | Functional | Admin | Must | App registration complexity is the primary barrier to enterprise connector adoption; a guided wizard removes this friction |
| REQ-RTC-002 | Per-User Consent Enforcement | The system SHALL require per-user consent for accessing individual user's meeting recordings, transcripts, private channel messages, or personal email via the real-time connectors; admin consent alone SHALL NOT be sufficient for individual user data; per-user consent SHALL be collected via an OAuth consent flow the first time the connector accesses a user's data. | Security | Admin, All | Must | GDPR Article 6/7 and enterprise HR policy require individual consent for capturing individual meeting data; admin consent covers organisational data, not personal data |
| REQ-RTC-003 | Consent Registry | The system SHALL maintain a Consent Registry recording: user identity, connector type, data scope consented to, consent timestamp, consent method (explicit/admin-policy), and revocation timestamp; accessible to Security Admins and Tenant Admins. | Governance | Admin | Must | Consent evidence is a GDPR audit obligation |
| REQ-RTC-004 | User Consent Revocation | The system SHALL allow any user to revoke their connector consent at any time from their profile settings, immediately stopping capture of their personal data and preventing the system from ingesting any further meeting/chat/email content attributable to that user; existing captured content SHALL be subject to the tenant's data retention policy. | Governance | All | Must | GDPR right to withdraw consent; revocation must be immediate not deferred |

---

### 4.2 Microsoft Teams — Meeting Auto-Capture

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-RTC-010 | Teams Meeting Transcript Auto-Capture | The system SHALL integrate with the Microsoft Teams meeting transcript API (Graph API: `/communications/callRecords` and `/onlineMeetings/{id}/transcripts`) to automatically retrieve meeting transcripts for meetings in configured calendar groups/teams within a configurable time window after meeting end (default: 15 minutes post-meeting). | Integration | ALL | Must | Teams transcripts are a primary source of architecture decision discussions, stakeholder interviews, and requirements discovery sessions |
| REQ-RTC-011 | Teams Bot Auto-Join | The system SHALL support an optional Teams Bot deployment mode (requiring admin-deployed Teams App from the ESIP manifest) where the ESIP Bot joins scheduled meetings as a participant to enable real-time transcript streaming during the meeting, not requiring Teams Premium transcript API licensing from the client. | Integration | Admin | Should | Teams transcript API requires Teams Premium; bot-join is the fallback path for non-Premium tenants |
| REQ-RTC-012 | Meeting Metadata Capture | The system SHALL capture, alongside the transcript, the following meeting metadata from Teams: meeting subject, organiser identity (name + email), attendee list (consented users only), meeting start/end time, meeting duration, associated Teams channel (if any), and meeting ID; metadata SHALL be stored as structured fields in the Knowledge Hub artifact record. | Data | ALL | Must | Metadata enables project/roadmap-item matching and speaker attribution |
| REQ-RTC-013 | Recording Link Capture | The system SHALL capture the OneDrive/SharePoint link to the meeting recording (if recording consent was given and the recording is accessible via MS Graph) and store it as a reference in the Knowledge Hub artifact, enabling playback from within ESIP without re-hosting the recording. | Functional | ALL | Should | Recording links add value for reviewing ambiguous transcript segments; re-hosting is avoided for data residency and storage cost reasons |
| REQ-RTC-014 | Teams Meeting Ingestion Scope | The system SHALL allow the tenant admin to configure the meeting ingestion scope per connector: (a) All meetings in a specified set of Teams groups/channels, (b) Meetings matching a subject keyword filter (e.g., contains "architecture review"), (c) Meetings organised by users in a specified group (e.g., all meetings by the Architecture Practice team), (d) Meetings where the ESIP Bot is explicitly invited. | Functional | Admin | Must | Unscoped capture of all meetings would violate employee privacy; granular scope configuration is a compliance requirement |

---

### 4.3 Microsoft Teams — Channel & Group Chat Essence

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-RTC-020 | Teams Channel Chat Ingestion | The system SHALL integrate with the Microsoft Teams Channel Messages API (Graph API: `/teams/{teamId}/channels/{channelId}/messages`) to periodically fetch channel messages from configured Teams channels, processing only messages since the last sync timestamp, within the scope of admin-configured channel lists. | Integration | ALL | Must | Architecture decisions, requirement clarifications, and stakeholder approvals frequently happen in Teams channels; capturing them prevents knowledge loss |
| REQ-RTC-021 | Chat Essence Extraction | The system SHALL invoke the AI engine (V06) to process a batch of Teams channel messages and produce a structured **Chat Essence** artifact in the Knowledge Hub, containing: key topics discussed, decisions made, action items identified, questions raised, and a 200-word narrative summary; the essence is stored as a KH artifact linked to the channel and time window, not as a raw message dump. | AI/ML | ALL | Must | Raw channel message dumps are noisy and voluminous; AI essence extraction captures the signal without the noise |
| REQ-RTC-022 | Chat Essence Frequency | The system SHALL generate a new Chat Essence artifact per configured channel at a configurable cadence: daily (default), weekly, or triggered on message volume threshold (e.g., every 50 new messages); each essence covers only messages since the prior essence generation, forming a linked chain. | Functional | Admin | Must | Configurable cadence balances freshness vs. AI cost |
| REQ-RTC-023 | Group Chat (1:N) Essence | The system SHALL support capturing and generating essence from Microsoft Teams group chats (not channel-based) where the ESIP bot has been added as a participant or where consented users have linked their group chats for capture; with per-user consent enforced per REQ-RTC-002. | Integration | ALL | Should | Architecture-critical discussions happen in ad-hoc group chats as well as formal channels |
| REQ-RTC-024 | Threaded Reply Preservation | The system SHALL preserve Teams message threading (parent message → replies chain) in the raw message store used as input to the AI essence engine, ensuring replies are contextually linked to their parent message during essence generation; the output essence SHALL reference thread topics, not individual messages. | Data | ALL | Should | Threads carry context that changes the meaning of individual messages; threading must be preserved for accurate AI interpretation |

---

### 4.4 Outlook — Email & Calendar

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-RTC-030 | Outlook Email Capture | The system SHALL integrate with Microsoft Graph Mail API to capture emails from configured mailboxes or shared mailboxes (e.g., `architecture@company.com`), with per-user consent for individual mailboxes; capture scope configurable by: folder name/path, subject keyword filter, sender/recipient domain filter, and date range. | Integration | ALL | Must | Email is still the primary channel for formal stakeholder communications, RFIs, and requirement clarifications in enterprises |
| REQ-RTC-031 | Email Thread Grouping | The system SHALL group captured emails by conversation thread (using the Graph API conversation ID) and store each thread as a single Knowledge Hub artifact, appending new messages as they arrive; AI essence SHALL be generated per thread on a configurable trigger (every new message or on thread inactivity > N hours). | Functional | ALL | Must | Thread grouping is essential for understanding the evolution of a topic across multiple email exchanges |
| REQ-RTC-032 | Email AI Essence | The system SHALL invoke the AI engine to generate an Email Thread Essence for each captured email thread: summary of the discussion, key decisions, open questions, and action items; stored in the Knowledge Hub with the raw thread retained for citation. | AI/ML | ALL | Must | Email thread summarisation is a high-value AI action; reduces reading time for architects reviewing large inboxes |
| REQ-RTC-033 | Calendar Event Capture | The system SHALL capture calendar events from configured Microsoft 365 accounts (with per-user consent), storing: event subject, organiser, attendee list, start/end time, location, recurrence, description, and links to associated files in the Knowledge Hub; calendar events SHALL be associated with linked meeting transcript artifacts if the event corresponds to a captured Teams meeting. | Integration | ALL | Should | Calendar data provides the organisational context (recurring review cycles, milestone dates) that enriches meeting transcript artifacts |
| REQ-RTC-034 | Shared Architecture Mailbox | The system SHALL support configuring a shared mailbox or distribution list email address as a Knowledge Hub ingest target without per-user consent (as shared mailboxes are org-owned), enabling a team practice of forwarding architecture-relevant emails to a shared inbox for automatic KH ingestion. | Functional | Admin | Must | Shared mailboxes avoid the privacy complexity of individual mailbox capture while enabling a practical "forward to capture" workflow |

---

### 4.5 Zoom — Meeting Transcripts & Recordings

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-RTC-040 | Zoom OAuth App Registration | The system SHALL guide tenant admins through registering a Zoom Server-to-Server OAuth app with the required scopes (cloud_recording:read:admin, meeting_token:read:admin:local_recording, meeting:read:admin) in the Zoom Marketplace; the ESIP wizard SHALL display exact scope names and link to the Zoom App Marketplace registration page. | Functional | Admin | Must | Zoom OAuth setup is error-prone; a guided wizard with exact scope names is required for successful setup |
| REQ-RTC-041 | Zoom Meeting Transcript Auto-Retrieval | The system SHALL use the Zoom Recording API (`GET /users/{userId}/recordings`) to retrieve audio transcripts (VTT format) for completed cloud recordings in configured Zoom accounts, polling at a configurable interval (default: every 30 minutes) or on receipt of a Zoom webhook event for `recording.completed`. | Integration | ALL | Must | Zoom VTT transcripts are the primary Zoom knowledge artifact; polling + webhook covers both immediate and delayed availability |
| REQ-RTC-042 | Zoom Webhook for Real-Time Trigger | The system SHALL support a Zoom webhook endpoint (URL configured in the Zoom App event subscription) receiving `meeting.ended` and `recording.completed` events, triggering immediate transcript retrieval for the ended meeting; webhook payload SHALL be validated with Zoom's HMAC-SHA256 signature. | Security | Admin | Must | Webhook trigger enables near-real-time ingestion; HMAC validation is a Zoom requirement |
| REQ-RTC-043 | Zoom Recording Reference | The system SHALL store a reference to the Zoom cloud recording URL (using the `play_url` field from the Recording API) in the Knowledge Hub artifact, enabling playback from the ESIP interface without re-hosting the recording; the reference SHALL include expiry metadata and SHALL surface a "recording expired" status when the Zoom auto-delete policy removes the recording. | Functional | ALL | Should | Same principle as Teams recording links: reference, don't re-host |
| REQ-RTC-044 | Zoom Ingestion Scope | The system SHALL allow admins to configure Zoom ingestion scope: (a) all cloud recordings for specified host email accounts, (b) recordings where the meeting topic matches a keyword filter, (c) recordings for meetings within a configured Zoom group; individual user consent SHALL be required for non-admin-owned personal meeting recordings. | Functional | Admin | Must | Consent and scope parity with Teams ingestion scope (REQ-RTC-014) |

---

### 4.6 Speaker Attribution & Identity Resolution

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-RTC-050 | Speaker Attribution in Transcripts | The system SHALL preserve speaker labels from meeting transcripts (Teams: speaker names from Graph transcript API; Zoom: speaker IDs from VTT) in the Knowledge Hub artifact, associating each transcript segment with the identified speaker name/ID. | Data | ALL | Must | Speaker attribution enables AI to reason about "what did the architect say vs. what did the PM say" and to extract decisions with attribution |
| REQ-RTC-051 | Identity Resolution | The system SHALL attempt to resolve speaker names/IDs from transcripts to ESIP user records (by matching display name or email from the meeting participant list), creating a `speaker_resolved_to_user` link in the Knowledge Hub artifact; unresolved speakers SHALL be stored as raw speaker labels without failure. | Data | ALL | Should | Resolved identities enable permission-aware filtering (user sees their own attributed statements) and richer AI analysis |
| REQ-RTC-052 | Unresolved Speaker Handling | The system SHALL store transcript segments with unresolved speaker labels as-is without discarding them; a tenant admin or authorised user MAY manually resolve speaker labels to ESIP user records from the artifact detail view. | UX | Admin | Should | Partial resolution must not degrade transcript quality |

---

### 4.7 AI Summarisation & Essence Extraction

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-RTC-060 | Meeting Transcript Summarisation | The system SHALL invoke the AI engine (V06) to generate a structured Meeting Summary artifact for each ingested meeting transcript, containing: meeting title/date, attendees, executive summary (200 words max), key discussion topics with per-topic summaries, decisions made (numbered list), action items (owner + due-date where spoken), open questions, and referenced documents/systems. | AI/ML | ALL | Must | Structured meeting summaries are immediately actionable; raw transcripts are not — architects should not read a 60-minute transcript to find one decision |
| REQ-RTC-061 | Customisable Summary Template | The system SHALL allow tenant admins to configure the meeting summary template (which sections to include, word limits per section, output language) via the Knowledge Hub connector settings, with the default template as a starting point. | Functional | Admin | Should | Different organisations have different meeting note standards; the default must be overridable |
| REQ-RTC-062 | Summary Linking to KH Artifact | The system SHALL store the AI-generated summary as a child artifact of the source transcript in the Knowledge Hub, preserving the raw transcript for full-text search and citation while surfacing the summary as the primary view; clicking any summary point SHALL navigate to the corresponding transcript segment with timestamp. | UX | ALL | Must | Navigability from summary to source is essential for trustworthiness of AI summaries |
| REQ-RTC-063 | Multi-Meeting Theme Synthesis | The system SHALL invoke the AI engine on demand to synthesise themes and decisions across a configurable set of meeting summaries (e.g., "all architecture review meetings in Q1 2026"), producing a cross-meeting insight report highlighting recurring topics, evolving decisions, and unresolved issues. | AI/ML | EA, SA, P13 | Should | Cross-meeting synthesis is a high-value EA activity that is practically impossible to do manually |

---

### 4.8 Decision & Action-Item Extraction (→ V06)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-RTC-070 | Decision Extraction | The system SHALL invoke the AI engine (V06 decision-extraction capability) on each meeting summary to extract formal Architecture Decisions mentioned in the meeting, proposing them as ADR (Architecture Decision Record) draft entries with: title, context (from transcript), decision statement, rationale (from discussion), and participants; presented for EA/SA review before materialising as ADR artefacts. | AI/ML | EA, SA | Must | Meeting transcripts are where architecture decisions are made verbally; automatic ADR candidate extraction dramatically reduces documentation lag |
| REQ-RTC-071 | Action Item Extraction | The system SHALL invoke the AI engine to extract action items from each meeting summary (phrases matching "X will do Y by Z", "action on A to B", "follow up on C"), structuring them as: assignee (resolved to ESIP user if possible), description, due date (inferred or extracted), source meeting ID + timestamp; action items are surfaced in the assignee's PM/EA workspace todo. | AI/ML | ALL | Must | Action item tracking from meetings is a universally needed capability; AI extraction from transcripts enables automatic task creation |
| REQ-RTC-072 | Requirement Candidate Extraction | The system SHALL invoke the AI engine (V06 requirement-extraction capability) on meeting summaries and chat essences to identify requirement candidate statements, proposing them as draft Requirement Cards in the BRD/PRD authoring workspace (REQ-PM-046 counterpart for RTC-sourced requirements) for PM/EA review. | AI/ML | P13, EA | Must | Meeting and chat discussions contain latent requirements; auto-surfacing them closes the capture gap |

---

### 4.9 Real-Time vs. Scheduled Sync

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-RTC-080 | Sync Mode Configuration | The system SHALL support two sync modes per connector: (a) **Near-real-time** — webhook/event-driven (transcript retrieved within 15 minutes of meeting end); (b) **Scheduled** — polling at a configurable interval (minimum 15 minutes, maximum 24 hours); the admin SHALL be able to configure the mode and fallback (if webhook fails, fallback to scheduled poll). | Functional | Admin | Must | Different organisations have different freshness requirements and different firewall configurations that may block webhook delivery |
| REQ-RTC-081 | Sync Status Dashboard | The system SHALL provide a Connector Sync Dashboard per RTC connector showing: last sync timestamp, items ingested in last 24h, error count, current sync mode, next scheduled sync, and a connector health indicator (Green/Amber/Red); accessible to tenant admins and KH managers. | Reporting | Admin | Must | Visibility into sync health prevents stale KH data going unnoticed |
| REQ-RTC-082 | Failed Ingestion Retry | The system SHALL automatically retry failed ingestion attempts (API failures, temporary permission errors) with exponential back-off (initial delay 1 minute, max delay 1 hour, max retries configurable, default 5); persistent failures after max retries SHALL surface as a connector alert to the admin. | NFR | Admin | Must | Transient API failures are common in enterprise API integrations; automatic retry with back-off is standard practice |

---

### 4.10 Project/Roadmap-Item Mapping

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-RTC-090 | Auto-Mapping to Project | The system SHALL attempt to automatically map each ingested meeting/chat/email artifact to the correct ESIP project/workspace using: (a) calendar invite subject keyword match against project name, (b) attendee list intersection with project team, (c) Teams channel-to-project mapping configured by admin, (d) email recipient/sender domain match to project; unmatched artifacts land in an Inbox for manual assignment. | AI/ML | Admin, P13, EA | Must | Unmapped artifacts are orphaned; auto-mapping ensures captured content immediately enriches the right project |
| REQ-RTC-091 | Manual Mapping & Re-mapping | The system SHALL allow any authorised user to manually assign or re-assign a KH artifact to a project/roadmap item from the artifact detail view or from the Inbox queue; re-assignment SHALL update all derived summaries and decision/action-item links. | Functional | ALL | Must | Auto-mapping is imperfect; manual override must always be possible |
| REQ-RTC-092 | Roadmap Item Tagging | The system SHALL allow a PM or EA to tag any captured meeting/chat/email artifact directly to a specific roadmap item (product or architecture), creating a traceability link from "what was discussed in this meeting" to "the roadmap item being worked on," visible in the roadmap item's evidence panel. | Functional | P13, EA | Must | Meeting traceability to roadmap items is the core value driver for RTC connectors in a PM/EA context |

---

### 4.11 Compliance, Consent & Retention

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-RTC-100 | Recording/Transcript Consent Notice | The system SHALL surface a configurable compliance notice template (editable by admin) that the ESIP Bot or meeting organiser can deliver at the start of any auto-joined meeting, informing participants that the meeting is being captured and transcribed for architecture documentation purposes; delivery method is configurable (Bot chat message, calendar invite body, or both). | Governance | Admin | Must | Informing meeting participants of recording/capture is a legal obligation in many jurisdictions; automated delivery removes the burden from the meeting organiser |
| REQ-RTC-101 | Jurisdiction-Specific Consent Controls | The system SHALL allow the admin to configure jurisdiction-based consent controls, enforcing stricter consent requirements (e.g., opt-in rather than opt-out recording notices) for users whose profile indicates residence in jurisdictions requiring explicit consent (GDPR, CCPA, Canadian PIPEDA); the system SHALL block capture for non-consented users in strict-consent jurisdictions. | Governance | Admin | Must | GDPR and equivalent laws create different consent obligations by location; a single consent model is not legally compliant globally |
| REQ-RTC-102 | Retention Policy for Captured Content | The system SHALL apply the tenant-configured Knowledge Hub retention policy to all RTC-captured artifacts: (a) automatic deletion after a configurable retention period, (b) legal hold override that prevents deletion during active litigation/regulatory review, (c) user-requested deletion within consent-revocation flow (REQ-RTC-004). | Governance | Admin | Must | GDPR data minimisation principle; captured content must not be retained indefinitely |
| REQ-RTC-103 | Audit Log for Capture Events | The system SHALL create an immutable audit log entry for every capture event: connector type, artifact ID, source meeting/chat/email reference, ingestion timestamp, consenting user IDs included, redaction actions applied, and project assignment; accessible to Security Admins. | Governance | Admin | Must | Capture audit is both a GDPR accountability obligation and an internal governance requirement |

---

## 5. REQ-SRCH — System-Wide Global Search

> **Reconciliation note (boundary — global search):** **V17 owns the global-search *experience*** — scope/coverage, hybrid query behaviour, facets, permission-aware result presentation, NLP/conversational search, deep-linking, saved searches/alerts, and search analytics (all REQ-SRCH-*). The underlying **index *infrastructure* → V11**: the inverted (BM25) index engine, the dense-vector store and ANN index, the embedding pipeline, and tenant index partitioning. REQ-SRCH-* requirements specify the contract V11 must satisfy; they do not own the index implementation. Resolves the FINALIZATION-REPORT "global-search UX vs index infra" action.

### Overview
ESIP global search provides a **"perfect search"** experience — a single search box finds everything across the platform: requirements, diagrams, deliverables, Knowledge Hub artifacts, portfolio items, comments, roadmap entries, users, and projects/workspaces. Results are **hybrid semantic + keyword**, **permission-aware**, and deep-link to the exact item. Search indexing infrastructure (inverted index engine, vector store, ANN index) → V11. This volume specifies the search experience, scope, NLP, and analytics.

---

### 5.1 Search Scope & Index Coverage

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-SRCH-001 | Universal Index Coverage | The system SHALL index all of the following content types for global search: Requirements (Functional, NFR, Security, User Stories, acceptance criteria text); Architecture Diagrams (name, description, node labels, edge labels, diagram metadata); Deliverables (BRD, PRD, HLD, LLD, ADR, all V10 document types — full text); Knowledge Hub Artifacts (meeting transcripts, summaries, chat essences, email threads, uploaded documents, connector-synced content); Portfolio Entries (application names, descriptions, tech stack, tags); Comments and Review Annotations; Product/Architecture Roadmap items (titles, descriptions, linked artefact summaries); Project/Workspace names and descriptions; User display names, emails, and role labels; Backlog Epics/Features/Stories (titles, AC, descriptions). | Functional | ALL | Must | Universal coverage is the definition of "perfect search"; gaps force users back to per-section search silos |
| REQ-SRCH-002 | Real-Time Index Updates | The system SHALL update the search index within 30 seconds of any content creation, modification, or deletion for all indexed content types; stale search results older than 30 seconds from the event SHALL be considered a defect. | NFR | ALL | Must | Stale search results erode trust in the search system; near-real-time indexing is required |
| REQ-SRCH-003 | Index Freshness Indicator | The system SHALL display a last-indexed timestamp per search result item in the result detail view; if any result's index entry is older than the configured staleness threshold (default: 2 minutes), the system SHALL surface a "results may not reflect recent changes" indicator. | UX | ALL | Should | Index transparency helps users understand why a recently created item might not appear immediately |

---

### 5.2 Hybrid Semantic + Keyword Engine

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-SRCH-010 | Hybrid Search Architecture | The system SHALL implement global search as a hybrid of: (a) **BM25 keyword search** (full-text inverted index — fast, exact matches, phrase matching, wildcard) over all indexed content, and (b) **dense vector semantic search** (embedding-based ANN search — meaning-level matching for paraphrases, synonyms, conceptual queries); results from both channels SHALL be merged using a Reciprocal Rank Fusion (RRF) or configurable score-blending algorithm, with relevance weights tunable by platform admins. | Functional | ALL | Must | Keyword-only search misses semantic queries; semantic-only misses exact identifier matches; hybrid is the state of the art for enterprise search (Elasticsearch 8 ELSER, Azure AI Search hybrid, Weaviate hybrid) |
| REQ-SRCH-011 | Embedding Model Configuration | The system SHALL allow platform admins to configure the embedding model used for vector search (options: ESIP-managed default model hosted in V11 AI infra; customer BYOK embedding via Azure OpenAI / AWS Bedrock / Anthropic API); embeddings are generated at index time and stored in the V11 vector store. | Functional | Admin | Must | Embedding model choice affects search quality and data residency; BYOK embedding supports air-gapped/private model requirements |
| REQ-SRCH-012 | Exact Match Boost | The system SHALL boost exact-match results (where the query string appears verbatim in the indexed content) to the top of hybrid results when a field-specific exact match is found for high-specificity queries (e.g., searching for a Requirement ID `REQ-PM-033` must return that exact item first). | Functional | ALL | Must | Architects frequently search by exact ID; ID lookups must always surface the exact item as result #1 |
| REQ-SRCH-013 | Fuzzy Matching & Spell Correction | The system SHALL apply configurable fuzzy matching (Levenshtein distance ≤ 2) and spell correction to keyword queries, suggesting corrected queries when no exact matches are found; fuzzy results SHALL be clearly labelled as "Did you mean X?" | UX | ALL | Should | Typos in requirement IDs, system names, and domain terms are common; fuzzy matching prevents zero-result frustration |

---

### 5.3 Filters, Facets & Scoped Search

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-SRCH-020 | Dynamic Facets | The system SHALL derive and display dynamic search facets from search results, including: Content Type (Requirement/Diagram/Deliverable/KH Artifact/Portfolio/Roadmap/Comment/etc.), Project/Workspace, Author, Creation Date (date range picker), Last Modified Date, Status, Priority, and Persona/Persona Tag; facets SHALL update in real time as filters are applied. | UX | ALL | Must | Dynamic facets are the industry standard for enterprise search (Elastic, Algolia, SharePoint); essential for navigating large result sets |
| REQ-SRCH-021 | Scoped Search | The system SHALL allow users to restrict global search to a specific scope without losing the cross-type results within that scope: (a) Workspace/Project scope — only content within the selected project; (b) Content-type scope — only requirements, or only diagrams, etc.; (c) Time scope — created/modified within a date range; scopes SHALL be combinable. | UX | ALL | Must | Users frequently want "all requirements in Project X" or "all diagrams modified this week" — scoped search serves these without leaving global search |
| REQ-SRCH-022 | Saved Filter Presets | The system SHALL allow any user to save a search query + filter combination as a named preset (e.g., "All open NFRs in my projects"), accessible from the search bar with a single click; presets are personal by default but shareable with named users or workspace members. | UX | ALL | Should | Repeated search patterns benefit from saved presets; reduces friction for frequent power-user queries |
| REQ-SRCH-023 | Cross-Content-Type Result Grouping | The system SHALL group search results by content type in a tabbed view (All / Requirements / Diagrams / Deliverables / KH Artifacts / Portfolio / Roadmap / Comments) with per-tab result counts, enabling the user to navigate to the exact content type of interest; the "All" tab shows a blended top-N from all types. | UX | ALL | Must | Mixed content-type results in a flat list are disorienting; grouping is the enterprise search UX standard |

---

### 5.4 Permission-Aware Results

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-SRCH-030 | Permission Filtering at Query Time | The system SHALL filter all search results at query time against the requesting user's access permissions (workspace/project membership, artifact visibility rules, free-tier Viewer/Reviewer scope restrictions), ensuring that results never include artifacts the user is not authorised to view; permission filtering SHALL occur at the search engine layer, not as a post-processing step, to prevent information disclosure through result counts. | Security | ALL | Must | Post-processing permission filtering reveals the existence of restricted content through result counts; query-time filtering is the only secure approach |
| REQ-SRCH-031 | No Orphaned Reference Leakage | The system SHALL ensure that result snippets, metadata, or excerpts from restricted content are never surfaced to unauthorised users even if the content type or workspace name matches the query; a zero-result response is preferable to partial leakage. | Security | ALL | Must | Information disclosure via snippets or metadata is a common search security vulnerability |
| REQ-SRCH-032 | Cross-Tenant Isolation | The system SHALL guarantee strict tenant isolation in the search index: no query from a user in Tenant A can produce results from Tenant B; this isolation SHALL be enforced by index-level tenant partitioning (separate index shards or index-level tenant key filtering), not solely by application-layer filtering. | Security | Admin | Must | Multi-tenant SaaS; cross-tenant data leakage via search is a critical security vulnerability |

---

### 5.5 Instant / Typeahead Search

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-SRCH-040 | Typeahead Suggestions | The system SHALL provide typeahead search suggestions appearing within 100ms of keystroke, suggesting: (a) exact-match item titles from the search index, (b) recent searches by this user, (c) popular searches in the tenant (configurable: show/hide), (d) known entity names (project names, application names, user names) matching the partial query; rendered in a dropdown below the search bar. | UX | ALL | Must | 100ms typeahead is the industry threshold for perceived instantaneity (Google, Elastic search UX research) |
| REQ-SRCH-041 | Instant Results Preview | The system SHALL display an instant results preview panel (top 5 results per major content type) upon query submission before full results load, enabling the user to navigate directly to a result while background results continue loading; each preview card shows: item type icon, title, project/workspace context, and a 120-character excerpt. | UX | ALL | Must | Instant preview gives the user something to interact with immediately; reduces perceived latency |
| REQ-SRCH-042 | Keyboard Navigation | The system SHALL support full keyboard navigation of typeahead suggestions and search results (arrow keys to navigate, Enter to open, Escape to dismiss, Tab to cycle facets), meeting WCAG 2.1 AA keyboard accessibility requirements. | UX | ALL | Must | Keyboard accessibility is required for enterprise compliance and power user efficiency |

---

### 5.6 Natural-Language Search

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-SRCH-050 | NLP Query Understanding | The system SHALL support natural-language queries (e.g., "show me all security requirements for the Payments product," "who created the API gateway diagram last month," "find meetings where the data retention decision was discussed") by invoking the AI engine (V06) to parse the query intent, extract entities (content type, project, user, date range, topic), and translate them to a structured search query executed against the hybrid index. | AI/ML | ALL | Must | Natural language is the most accessible query interface; NLP query understanding enables "Google-like" search behaviour for architecture artefacts |
| REQ-SRCH-051 | Entity Extraction from Query | The system SHALL extract named entities from NL queries: project/product names, persona names (EA, PM, SA), requirement category names, date expressions (last month, Q1 2026, before 2026-01-01), and content type references (BRD, ADR, diagram, roadmap); extracted entities SHALL be displayed as "chips" on the search bar for user confirmation/correction before result rendering. | AI/ML | ALL | Should | Displaying extracted entities as chips makes NLP interpretation transparent and correctable — a best practice from Slack, Notion, and Linear NLP search implementations |
| REQ-SRCH-052 | NL Search Answer Box | The system SHALL invoke the AI engine (V06 RAG capability) to generate a direct answer card at the top of NL search results when the query is answerable from indexed content (e.g., "What was decided about the authentication approach?"), citing the source artifacts and providing deep-links to the cited sections; the answer card is clearly labelled as AI-generated. | AI/ML | ALL | Should | Direct answer cards reduce result-scanning time for factual questions; citation transparency maintains trust |
| REQ-SRCH-053 | Conversational Search Refinement | The system SHALL support multi-turn conversational search refinement in the search context: after receiving an NL search result, the user can type a follow-up query ("show only the ones from last quarter", "filter to requirements owned by P13") and the system SHALL interpret it in the context of the prior query, refining results incrementally. | AI/ML | ALL | Could | Conversational refinement is an emerging enterprise search UX pattern; useful but not must-have at V1 |

---

### 5.7 Deep-Link & Result Navigation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-SRCH-060 | Deep-Link to Exact Item | The system SHALL ensure every search result carries a deep-link URL that navigates the user directly to the exact item within the ESIP UI: the specific requirement card in a BRD, the specific diagram, the specific section of a deliverable, the specific meeting transcript segment, the specific roadmap item; deep-links SHALL be stable (survive renames and moves). | Functional | ALL | Must | Deep-links are the most impactful UX feature of enterprise search; the ability to jump from search result to exact item location eliminates navigation time |
| REQ-SRCH-061 | Highlight in Context | The system SHALL highlight the matched query terms within the navigated item view (scroll to and highlight the matching text within the document/diagram/transcript) when arriving via a search deep-link, giving the user immediate visual confirmation they have landed on the relevant section. | UX | ALL | Must | Context highlighting after deep-link navigation is the browser "Ctrl+F" equivalent; without it users must re-search within the item |
| REQ-SRCH-062 | Breadcrumb in Result Card | The system SHALL display a breadcrumb path in each search result card (e.g., Project Alpha > BRD-004 > Security Requirements > REQ-BRD-004-SEC-007) giving the user workspace and document context before clicking, enabling them to disambiguate results with similar titles. | UX | ALL | Must | Context breadcrumb prevents clicking the wrong result in large multi-project tenants |

---

### 5.8 Saved Searches & Alerts

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-SRCH-070 | Saved Searches | The system SHALL allow any authenticated user to save a search query (including filters and scope) with a user-chosen name, accessible from the search bar's "Saved Searches" panel; saved searches SHALL be editable and deletable; max 50 saved searches per user. | Functional | ALL | Should | Saved searches are a standard enterprise search feature; recurring research queries benefit from one-click re-execution |
| REQ-SRCH-071 | Search Alerts | The system SHALL allow a user to set an alert on any saved search; when new content matching the saved search is indexed, the system SHALL notify the user via in-app notification and/or email (configurable) with a summary of new results; alert frequency configurable (real-time / daily digest / weekly digest). | Functional | ALL | Should | Search alerts are the "subscribe to a topic" mechanism; PMs and EAs can subscribe to "any new security requirement" or "any new meeting mentioning Project X" |
| REQ-SRCH-072 | Shared Saved Searches | The system SHALL allow a user to share a saved search with named workspace members or all members of a project; shared searches appear in recipients' "Team Searches" section of the search panel and can be copied (not edited) by recipients. | Functional | ALL | Could | Team searches enable practice leads to distribute canonical search queries (e.g., "EA's standard portfolio health search") across the team |

---

### 5.9 Cross-Project / Cross-Workspace Search

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-SRCH-080 | Cross-Project Search by Default | The system SHALL search across all projects and workspaces the requesting user has access to by default (global scope), with a project/workspace filter to narrow; users with access to N projects see results from all N projects in a single result set. | Functional | ALL | Must | Enterprise architects work across projects; cross-project search is more valuable than scoped search for portfolio-level queries |
| REQ-SRCH-081 | Cross-Project Collision Detection via Search | The system SHALL support a search-powered cross-project duplicate/overlap detection feature: a user can run a query and see semantically similar items across all accessible projects (e.g., "show me all requirements similar to REQ-PM-033 across all projects"), helping identify duplicate requirements or re-usable decisions across the portfolio. | AI/ML | EA, SA | Should | Cross-project knowledge reuse is a recognised EA productivity goal; search is the natural discovery mechanism |
| REQ-SRCH-082 | Workspace Isolation Override for EAs | The system SHALL allow Enterprise Architects with cross-workspace permissions (configurable in V11 RBAC) to search across all workspaces in a tenant, even workspaces they are not explicitly a member of, subject to admin-granted "global read" permission; non-EA users are strictly limited to their own workspace memberships. | Security | EA, Admin | Should | Enterprise-level EAs need portfolio-wide visibility; but this must be an explicit, audited permission grant, not default |

---

### 5.10 Search Analytics & Insights

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-SRCH-090 | Search Query Analytics | The system SHALL capture and store anonymised search query analytics: query text (with PII stripped per GDPR), timestamp, user tier (not identity), result count, top-3 clicked results (content type + item ID), time-to-first-click, and zero-result indicator; retained for configurable rolling window (default: 90 days). | Data | Admin | Must | Search analytics drive search quality improvement; zero-result queries identify content gaps or index deficiencies |
| REQ-SRCH-091 | Search Analytics Dashboard | The system SHALL provide a Search Analytics Dashboard to platform/tenant admins showing: top 20 queries by volume, zero-result query list with counts, average results per query, click-through rate by content type, trending queries over time (7d/30d/90d), and "no-click" queries (results shown but nothing clicked). | Reporting | Admin | Should | Search quality management requires analytics visibility; zero-result and no-click queries are primary improvement signals |
| REQ-SRCH-092 | Index Coverage Report | The system SHALL provide an Index Coverage Report showing: total indexed items by content type, last-indexed timestamp per content type, estimated index lag (average time between content creation and index availability), and items excluded from indexing (due to permission configuration or content type exclusion rules). | Reporting | Admin | Should | Index coverage transparency is required for admins to verify that the "universal index" is actually universal |
| REQ-SRCH-093 | Search-Driven Content Gaps | The system SHALL periodically (configurable: weekly default) generate an AI-driven Content Gap report from search analytics: queries with zero or low-quality results that indicate missing Knowledge Hub content, missing documentation, or un-indexed content types; presented to KH managers and tenant admins as actionable recommendations. | AI/ML | Admin | Could | Search analytics as a content strategy feedback loop; surfacing gaps the organisation is actively searching for enables targeted KH enrichment |

---

## Boundary Notes & Cross-Volume Dependencies

| Boundary | Responsible Volume |
|---|---|
| AI reasoning, RAG, extraction, prompt management, AI provider config | **V06** |
| Base connector framework (OAuth, credential vault, connector job engine, sync scheduler) | **V08** |
| Knowledge Hub artifact model, storage lifecycle, provenance | **V08** |
| Code-scan parsing, flow derivation, SBOM, portfolio APM data model | **V09** |
| Persona metamodel engine, RBAC/ABAC, audit log, multi-tenancy isolation kernel | **V11** |
| Search indexing infrastructure (inverted index, vector store, ANN engine, embedding pipeline) | **V11** |
| Billing, seat metering, subscription enforcement internals, payment processing | **V13** |
| Review designer, review cycle management, vote/approval engine | **V15** |
| Deliverable document/PPT/Excel generation templates and rendering | **V10** |
| Diagram canvas, auto-layout, ArchiMate/C4 renderers | **V03** |
| Requirements Pillar RTM, requirement quality engine, ingestion | **V07** |

---

## Assumptions

1. P13 (Product Manager) is an additive persona, not a replacement for any P1–P12 architect persona; a user may hold both.
2. Viewer and Reviewer free tiers are unlimited in seat count but scoped to shared/published artifacts only; they do not provide access to the authoring workspace.
3. The Rust code-scanner binary is a pre-existing V09 artefact; this volume specifies the agent provisioning/operational envelope only, not the scanner's parsing logic.
4. Teams meeting transcript capture assumes the client has Microsoft 365 E3/E5 or Teams Premium for transcript API access; bot-join mode (REQ-RTC-011) is the fallback for non-Premium licenses.
5. Global search result permissions are enforced at query time via the V11 ABAC layer; this volume specifies the search UX and coverage requirements, not the enforcement implementation.
6. All AI-assisted authoring (BRD/PRD drafting, NLP search, meeting summarisation) routes through the V06 AI engine; no AI model is directly integrated in this volume.
