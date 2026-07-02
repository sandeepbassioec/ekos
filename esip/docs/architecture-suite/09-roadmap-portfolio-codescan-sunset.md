# Volume 09 — Roadmaps, Portfolio Management, Code Scanning, Flow Derivation & Sunset/Migration
## ESIP Architecture Automation Suite

| Attribute | Value |
|---|---|
| Volume | 09 |
| Prefix | PF |
| Sub-prefixes | PF-RM (Roadmaps) · PF-APM (Application Portfolio Management) · PF-CS (Code Scanning) · PF-FD (Flow Derivation) · PF-LD (Landscape & Dependency) · PF-SD (Sunset/Decommission) · PF-MIG (Migration Planning) · PF-LNK (Cross-Volume Linkage) |
| Status | Baseline Draft |
| Date | 2026-06-23 |
| Scope | Multi-year roadmaps; portfolio/APM; code-scanning & SBOM; flow derivation from code; landscape & dependency mapping; sunset/decommission; migration planning (6 R's); cross-pillar linkage |
| Out of Scope | AI/LLM reasoning engine that powers analysis → V06 (this volume states *required outputs*, V06 owns the model); diagram rendering/canvas → V03 (this volume generates the data model that V03 renders); repo authentication as a KH connector → V08 (here repos are a *scan source*); deliverable document generation → V10; platform NFRs/RBAC/audit → V11 |

---

## Personas Reference

| Code | Persona |
|---|---|
| EA | Enterprise Architect |
| SA | Solution Architect |
| AA | Application Architect |
| IA | Integration / API Architect |
| DA | Data Architect |
| CA | Cloud / Infrastructure Architect |
| SecA | Security Architect |
| MA | Microservices / Platform / DevOps Architect |
| CRM | CRM / Platform Architect |
| ALL | All personas |

---

## Table of Contents

1. [Roadmaps — Foundation & Multi-Year Views](#1-roadmaps--foundation--multi-year-views)
2. [Roadmaps — Types & Rendering Variants](#2-roadmaps--types--rendering-variants)
3. [Roadmaps — Transition Architectures & Plateaus](#3-roadmaps--transition-architectures--plateaus)
4. [Roadmaps — Dependencies, Critical Path & Scenario Planning](#4-roadmaps--dependencies-critical-path--scenario-planning)
5. [Roadmaps — Themes, Initiatives, Epics & Milestones](#5-roadmaps--themes-initiatives-epics--milestones)
6. [Roadmaps — Capacity-Aware Planning](#6-roadmaps--capacity-aware-planning)
7. [Roadmaps — Publishing & Collaboration](#7-roadmaps--publishing--collaboration)
8. [Application Portfolio Management — Inventory & Registry](#8-application-portfolio-management--inventory--registry)
9. [Application Portfolio Management — Capability & Process Mapping](#9-application-portfolio-management--capability--process-mapping)
10. [Application Portfolio Management — Scoring, Health & Heat Maps](#10-application-portfolio-management--scoring-health--heat-maps)
11. [Application Portfolio Management — Lifecycle, TIME Model & 6 R's](#11-application-portfolio-management--lifecycle-time-model--6-rs)
12. [Application Portfolio Management — Rationalization & Redundancy](#12-application-portfolio-management--rationalization--redundancy)
13. [Application Portfolio Management — Cost, Ownership & Vendor](#13-application-portfolio-management--cost-ownership--vendor)
14. [Code Scanning — Repository Ingestion & Connectivity](#14-code-scanning--repository-ingestion--connectivity)
15. [Code Scanning — Parse & Structure Derivation](#15-code-scanning--parse--structure-derivation)
16. [Code Scanning — Dependency Graphs & SBOM](#16-code-scanning--dependency-graphs--sbom)
17. [Code Scanning — API Surface & Data Access Detection](#17-code-scanning--api-surface--data-access-detection)
18. [Code Scanning — Tech-Debt, Quality & Vulnerability Surfacing](#18-code-scanning--tech-debt-quality--vulnerability-surfacing)
19. [Code Scanning — Metrics & Complexity](#19-code-scanning--metrics--complexity)
20. [Code Scanning — Monorepo & Multi-Repo Support](#20-code-scanning--monorepo--multi-repo-support)
21. [Flow Derivation — Application & Call-Graph Flows](#21-flow-derivation--application--call-graph-flows)
22. [Flow Derivation — Business Process & Data Flow Derivation](#22-flow-derivation--business-process--data-flow-derivation)
23. [Flow Derivation — Diagram Candidate Generation](#23-flow-derivation--diagram-candidate-generation)
24. [Landscape & Dependency Mapping](#24-landscape--dependency-mapping)
25. [Sunset / Decommission Planning](#25-sunset--decommission-planning)
26. [Migration Planning — Disposition & Wave Design](#26-migration-planning--disposition--wave-design)
27. [Migration Planning — Patterns & Target-State Mapping](#27-migration-planning--patterns--target-state-mapping)
28. [Cross-Volume Linkage](#28-cross-volume-linkage)

---

## 1. Roadmaps — Foundation & Multi-Year Views

### 1.1 Roadmap Record Model

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-RM-001 | Roadmap Entity Model | The system SHALL maintain a roadmap entity with: unique ID, title, type (product/technology/capability/architecture/migration/custom), horizon (1-, 3-, 5-year), owner, status, baseline date, last-updated, and a structured time-axis (fiscal quarters or calendar months configurable per roadmap) | Data | EA, SA | Must | Common to all APM/EA tooling (LeanIX, Ardoq, MEGA) |
| REQ-PF-RM-002 | Multiple Concurrent Roadmaps | The system SHALL support an unlimited number of concurrent roadmaps per tenant/workspace, each independently owned, versionable, and combinable into a portfolio-level "master" view | Functional | EA, SA | Must | Enterprise EA practice |
| REQ-PF-RM-003 | Multi-Year Time Axis | The system SHALL support roadmap time axes spanning 1 to 10 years with configurable granularity: annual, semi-annual, quarterly, monthly, sprint-level; the granularity SHALL be switchable per view without losing underlying data | Functional | EA | Must | Industry standard |
| REQ-PF-RM-004 | Roadmap Versioning & Baselining | The system SHALL allow a roadmap to be baselined (snapshot), preserving a named, immutable version; subsequent edits create a new working version; any two versions SHALL be diff-comparable | Functional | EA, SA | Must | Governance requirement |
| REQ-PF-RM-005 | Roadmap Templates | The system SHALL provide starter templates for each roadmap type (product, technology, capability, architecture, migration) with pre-populated time bands and sample swimlanes; templates SHALL be customizable and saveable as new templates | UX | EA, SA | Should | Accelerates adoption |
| REQ-PF-RM-006 | AI Roadmap Draft from Inputs | The system SHALL use AI to draft an initial roadmap from ingested inputs (requirements documents, strategy decks, portfolio assessments, migration plans), proposing items, priorities, and time placement; output SHALL be editable and traceable to source | AI/ML | EA, SA | Must | Flagship automation capability |

### 1.2 Now–Next–Later & Horizon Views

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-RM-007 | Now–Next–Later View | The system SHALL render any roadmap in a "Now / Next / Later" (Kanban-style horizon) view grouping items by configurable horizon bands (e.g., 0–3 months / 3–12 months / 12+ months) alongside the time-axis view | UX | EA, SA, MA | Must | Product roadmapping standard (ProductBoard/Aha! pattern) |
| REQ-PF-RM-008 | Quarterly Summary View | The system SHALL render a collapsible quarterly summary view showing initiative/epic counts, budget allocations, and completion status per quarter across the roadmap | UX | EA | Should | Executive communication |
| REQ-PF-RM-009 | Milestone Timeline View | The system SHALL render a milestone-only timeline overlaying the full roadmap, highlighting key decision gates, release dates, regulatory dates, and transition-state transitions | UX | EA, SA | Must | |

---

## 2. Roadmaps — Types & Rendering Variants

### 2.1 Roadmap Types

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-RM-010 | Product Roadmap Type | The system SHALL support a **Product Roadmap** type with: themes, epics, features, release trains, and customer segments as first-class items; items SHALL link to requirements (V07) and deliverables (V10) | Functional | EA, SA, MA | Must | |
| REQ-PF-RM-011 | Technology Roadmap Type | The system SHALL support a **Technology Roadmap** type with: technology components, tech-stack items, vendor/product versions, upgrade cycles, end-of-support dates, and replacement candidates as items | Functional | EA, CA | Must | Standard EA practice |
| REQ-PF-RM-012 | Capability Roadmap Type | The system SHALL support a **Capability Roadmap** type linked to a business capability map (TOGAF/BIZBOK), showing when each capability reaches target maturity (1–5 Gartner scale or configurable) per planning period | Functional | EA | Must | TOGAF ADM Phase B; links to V01 |
| REQ-PF-RM-013 | Architecture Roadmap Type | The system SHALL support an **Architecture Roadmap** type based on TOGAF ADM Phase F, with: transition architectures, architecture milestones, ADM phase gates, and linkage to the Architecture Repository; each milestone SHALL optionally attach an architecture state snapshot | Functional | EA | Must | TOGAF 10 ADM Phase F |
| REQ-PF-RM-014 | Migration Roadmap Type | The system SHALL support a **Migration Roadmap** type derived from migration planning (§26–§27), showing wave/move-group sequencing, application dispositions (6 R's), dependencies, and readiness gates over time | Functional | EA, CA | Must | Cloud/datacenter migration practice |
| REQ-PF-RM-015 | Custom Roadmap Type | The system SHALL support user-defined roadmap types by specifying item entity name, swimlane dimensions, and custom fields, without requiring code changes | Functional | ALL | Should | Extensibility |

### 2.2 Rendering Variants

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-RM-016 | Gantt View | The system SHALL render any roadmap as a Gantt chart with: bar-per-item, start/end dates, dependencies as arrows, critical path highlight, baseline vs. actual overlay, and percent-complete shading; Gantt SHALL support drag-to-reschedule and dependency drag | UX | EA, SA | Must | Universal project/architecture planning |
| REQ-PF-RM-017 | Swimlane View | The system SHALL render any roadmap as a swimlane chart, with swimlane dimensions configurable from any roadmap item attribute (e.g., business domain, team, application, technology layer); rows and columns SHALL be collapsible | UX | EA, SA | Must | |
| REQ-PF-RM-018 | Timeline / Bar Chart View | The system SHALL provide a classic horizontal timeline view with color-coded item bars, time-band markers (quarters/years), and legend; items SHALL be filterable and groupable by any attribute | UX | EA | Must | |
| REQ-PF-RM-019 | Heat-Map Overlay on Roadmap | The system SHALL allow a portfolio heat-map dimension (business fit, technical fit, cost, risk) to be overlaid on roadmap items as color coding, surfacing priority for action | UX | EA | Should | |
| REQ-PF-RM-020 | Master / Portfolio Roadmap View | The system SHALL render a master view that aggregates multiple roadmap types side-by-side on a shared time axis with cross-roadmap dependency lines, allowing an EA to see product, technology, and architecture timelines in a single canvas | UX | EA | Must | BiZZdesign/MEGA pattern |

---

## 3. Roadmaps — Transition Architectures & Plateaus

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-RM-021 | Transition Architecture Record | The system SHALL support a **Transition Architecture** entity (ArchiMate I&M aspect) with: ID, name, description, target plateau date, predecessor/successor transition architectures, and linked architecture state snapshot (ArchiMate or C4 model) | Data | EA | Must | ArchiMate 3.2 Implementation & Migration; TOGAF ADM Phase E/F |
| REQ-PF-RM-022 | Plateau Sequencing | The system SHALL allow transition architectures to be sequenced as an ordered chain of plateaus (Baseline → TA1 → TA2 → Target), each representing a stable intermediate state; the chain SHALL be visualizable as a timeline and as an ArchiMate Implementation & Migration viewpoint | Functional | EA | Must | ArchiMate Implementation & Migration viewpoint |
| REQ-PF-RM-023 | Architecture State Snapshot per Plateau | The system SHALL link each transition architecture (plateau) to a specific version of the architecture model (from the modeling engine, V03), so the evolution of architecture states over time is captured and traversable | Functional | EA, SA | Must | TOGAF Architecture Repository |
| REQ-PF-RM-024 | Gap Analysis Between Plateaus | The system SHALL compute and display the architectural delta (gap) between any two consecutive transition architectures — elements added, removed, or changed — with AI summarization of the transformation effort; this feeds the migration roadmap | Functional | EA | Must | TOGAF ADM Phase E gap analysis |
| REQ-PF-RM-025 | ArchiMate I&M Viewpoint Generation | The system SHALL produce an ArchiMate Implementation & Migration viewpoint for the plateau chain — showing Work Packages, Deliverables, Plateaus, and Gap elements — rendering via V03 | Functional | EA | Must | ArchiMate 3.2 §13 |
| REQ-PF-RM-026 | Transition Risk per Plateau | The system SHALL capture and surface architectural risk per transition: dependency conflicts, technology readiness gaps, resource conflicts, regulatory blockers; risks SHALL link to the RAID register (V04/V11) | Functional | EA, SA | Should | |

---

## 4. Roadmaps — Dependencies, Critical Path & Scenario Planning

### 4.1 Dependencies

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-RM-027 | Item-to-Item Dependencies | The system SHALL support explicit finish-to-start, start-to-start, finish-to-finish, and start-to-finish dependency relationships between any roadmap items, with configurable lag/lead time; dependencies SHALL be visualized as arrows on Gantt and timeline views | Functional | EA, SA | Must | |
| REQ-PF-RM-028 | Cross-Roadmap Dependencies | The system SHALL support dependencies between items on *different* roadmaps (e.g., a product roadmap item depending on a technology roadmap item); cross-roadmap dependencies SHALL appear on the master portfolio view | Functional | EA | Must | |
| REQ-PF-RM-029 | AI Dependency Suggestion | The system SHALL use AI to suggest likely inter-item dependencies based on semantic similarity, shared application/capability ownership, and portfolio data; suggestions SHALL require human confirmation before persisting | AI/ML | EA | Should | |
| REQ-PF-RM-030 | Critical Path Calculation & Highlight | The system SHALL calculate and highlight the critical path across all dependencies in the roadmap, showing items where a delay would extend the overall roadmap end date; recalculation SHALL trigger automatically on any change | Functional | EA, SA | Must | Standard CPM |
| REQ-PF-RM-031 | Dependency Impact Propagation | The system SHALL, when a roadmap item is rescheduled, automatically propagate the change to all dependents (with user confirmation for cascades affecting milestones) and flag items that cannot be satisfied within the current horizon | Functional | EA | Should | |

### 4.2 What-If Scenarios

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-RM-032 | Scenario Branch | The system SHALL allow any roadmap to be branched into a named **Scenario** (e.g., "Optimistic / Base / Pessimistic"; "Cloud-first vs. On-prem-extend"), where items can be moved, added, or removed without affecting the baseline | Functional | EA | Must | |
| REQ-PF-RM-033 | Scenario Comparison | The system SHALL allow side-by-side visual comparison of up to four scenarios on a shared time axis, highlighting the differences in scheduling, cost, risk, and resource load | UX | EA | Must | |
| REQ-PF-RM-034 | AI Scenario Generation | The system SHALL use AI to generate alternative scenario branches based on user-defined constraints (e.g., "reduce budget by 20%", "accelerate cloud by 6 months", "remove dependency on vendor X"), producing a proposed item rescheduling with rationale | AI/ML | EA | Should | |
| REQ-PF-RM-035 | Scenario Promotion to Baseline | The system SHALL allow any approved scenario to be promoted to the official baseline, with the previous baseline automatically archived as a named version | Functional | EA | Must | |

---

## 5. Roadmaps — Themes, Initiatives, Epics & Milestones

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-RM-036 | Strategic Theme Registry | The system SHALL maintain a **Strategic Theme** registry (e.g., "Digital Transformation", "Cost Optimisation", "Cloud Migration") that can be assigned to roadmap items, requirements, capabilities, and portfolio applications; themes SHALL be used as a filter/group dimension on all roadmap views | Data | EA | Must | SAFe / TOGAF strategy |
| REQ-PF-RM-037 | Initiative-to-Epic Hierarchy | The system SHALL support a configurable work-breakdown hierarchy for roadmap items (e.g., Theme → Initiative → Epic → Feature → Story); items at each level SHALL be independently schedulable and aggregatable on the roadmap timeline | Functional | EA, SA, MA | Must | SAFe / Agile roadmap |
| REQ-PF-RM-038 | Milestone Entity | The system SHALL support **Milestone** entities with: date, type (architecture gate, release, regulatory, board review, tech-freeze), description, owner, and linked deliverables; milestones SHALL be visually distinct on all roadmap views | Data | EA, SA | Must | |
| REQ-PF-RM-039 | Milestone Dependency | The system SHALL enforce that a milestone's date cannot precede the latest end-date of any predecessor item, with warnings surfaced in the UI and a dependency diagram | Functional | EA | Must | |
| REQ-PF-RM-040 | Requirements Linkage on Items | The system SHALL allow any roadmap item (initiative/epic/feature) to link to one or more requirements (V07 records), so that the percentage of requirements covered by the roadmap is computable and gaps are surfaced | Integration | EA, SA | Must | Traceability pillar |
| REQ-PF-RM-041 | Capability Linkage on Items | The system SHALL allow roadmap items to be linked to business capability map nodes (V01/V09 §9), enabling a capability maturity progression view: current maturity → target maturity per planning horizon | Integration | EA | Must | |
| REQ-PF-RM-042 | OKR / KPI Linkage | The system SHALL allow roadmap items to be linked to OKRs or KPIs defined in the system, and SHALL display predicted impact on each OKR/KPI when the item completes, using AI estimation or user-supplied values | Functional | EA, SA | Should | Strategy-to-execution traceability |

---

## 6. Roadmaps — Capacity-Aware Planning

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-RM-043 | Team / Resource Pool Model | The system SHALL maintain a lightweight model of teams or resource pools (name, capacity-unit, FTE-count or story-point velocity) usable for roadmap capacity planning; this model SHALL NOT replace a project management tool but SHALL surface overloads | Data | EA, SA | Should | |
| REQ-PF-RM-044 | Item Effort Estimation | The system SHALL allow roadmap items to carry an estimated effort (in T-shirt sizes, story points, or FTE-months) and associated owning team; AI SHALL suggest effort estimates based on item description, historical similar items, and portfolio complexity scores | AI/ML | EA, SA | Should | |
| REQ-PF-RM-045 | Capacity Overload Detection | The system SHALL aggregate team workload across all roadmap items per period and flag periods where capacity exceeds team availability, rendering an overload heat-map per swimlane row (team) over the time axis | Functional | EA | Should | |
| REQ-PF-RM-046 | Capacity-Constrained Scheduling | The system SHALL offer an AI-assisted capacity-constrained schedule optimiser that reschedules items to resolve overloads while respecting dependencies and milestones, producing a proposed revised roadmap for human approval | AI/ML | EA | Could | |

---

## 7. Roadmaps — Publishing & Collaboration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-RM-047 | Read-Only Shareable Portal | The system SHALL publish any roadmap as a read-only interactive web portal (password-protected or token-linked) renderable in a browser without ESIP login, suitable for sharing with stakeholders, board, or vendors | Functional | EA, SA | Must | |
| REQ-PF-RM-048 | Export to PowerPoint / PDF | The system SHALL export any roadmap view (Gantt, timeline, swimlane, now-next-later) to PowerPoint (PPTX) and PDF with configurable page layout and theme, for inclusion in board decks and architecture reviews; output SHALL route to V10 deliverable generation | Integration | EA, SA | Must | |
| REQ-PF-RM-049 | Export to Excel / CSV | The system SHALL export roadmap item data (all fields) to Excel (.xlsx) and CSV, including all linked requirements, capabilities, and milestones | Integration | EA | Must | |
| REQ-PF-RM-050 | Roadmap Comment & Review | The system SHALL support inline comments on roadmap items and milestone markers, with @mention notification, threaded replies, and resolution tracking; comments SHALL be reviewable in a consolidated view (V11 collaboration) | Functional | ALL | Should | |
| REQ-PF-RM-051 | Change Notification | The system SHALL notify roadmap stakeholders (configurable) when a baseline is changed: items added/removed/rescheduled, milestone moved, scenario promoted; notifications SHALL include a human-readable change summary and a diff link | Functional | EA | Should | |
| REQ-PF-RM-052 | Jira / ADO Sync (roadmap items) | The system SHALL optionally synchronise roadmap items (initiatives/epics) with Jira Epics or Azure DevOps Features bidirectionally, so that completion status flows back to the roadmap timeline; sync SHALL be configurable per field | Integration | EA, SA, MA | Should | |

---

## 8. Application Portfolio Management — Inventory & Registry

### 8.1 Application Registry

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-APM-001 | Application Entity Model | The system SHALL maintain an Application entity with: unique ID, canonical name, aliases, description, version, status (plan/invest/maintain/retire/retired), lifecycle phase, hosting model (on-premises/IaaS/PaaS/SaaS/hybrid), primary business owner, IT owner, primary vendor, and procurement source | Data | EA | Must | LeanIX/Ardoq APM standard |
| REQ-PF-APM-002 | Application Registry CRUD | The system SHALL provide full Create/Read/Update/Delete for the application registry, with field-level validation, duplicate detection (fuzzy-name match + alias lookup), and mandatory fields enforcement | Functional | EA | Must | |
| REQ-PF-APM-003 | Bulk Import from CMDB / ServiceNow | The system SHALL import application records in bulk from ServiceNow CMDB, Excel upload (.xlsx), and CSV; field mapping SHALL be configurable; import SHALL detect duplicates and surface conflicts for human resolution | Integration | EA | Must | ServiceNow APM is the dominant enterprise CMDB |
| REQ-PF-APM-004 | Application Relationship Types | The system SHALL support typed relationships between applications: **Uses**, **Integrates-With**, **Depends-On**, **Runs-On** (infrastructure), **Hosted-By** (platform), **Replaces**, **Is-Replaced-By**, **Is-Subset-Of**, **Shares-Data-With** | Data | EA, IA | Must | ArchiMate application-layer relationships |
| REQ-PF-APM-005 | Application Version Tracking | The system SHALL track multiple versions of an application (current production version, versions in-flight, planned versions) with separate lifecycle states per version, supporting phased migrations | Data | EA, CA | Should | |
| REQ-PF-APM-006 | AI-Assisted Registry Population | The system SHALL use AI to extract application names, owners, and relationships from ingested documents (project charters, IT strategy decks, architecture diagrams, inventory spreadsheets) and propose them as draft records for human confirmation | AI/ML | EA | Must | Flagship ingestion capability |
| REQ-PF-APM-007 | Code-Scan to Registry Population | The system SHALL auto-populate the application registry with applications discovered during code scanning (§14–§20), including tech-stack fingerprint, primary language, framework, and repository URL; discovered applications SHALL be flagged as "discovered" pending human review | Functional | EA, AA | Must | Flagship capability: scan → portfolio |
| REQ-PF-APM-008 | Application Tags & Classification | The system SHALL support free-form tags and a configurable classification taxonomy (e.g., LOB / domain / tier / criticality / data-sensitivity) on each application record, with bulk tag operations | Data | EA | Should | |

### 8.2 Supporting Entities

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-APM-009 | Technology Component Registry | The system SHALL maintain a Technology Component registry (databases, middleware, runtime platforms, cloud services, hardware) with version, vendor, end-of-support date, and relationships to applications that use them | Data | EA, CA | Must | |
| REQ-PF-APM-010 | Vendor / Product Catalog | The system SHALL maintain a Vendor and Product catalog with: contract expiry, support level, commercial model (license/subscription/consumption), and strategic status (strategic/preferred/tactical/phase-out) | Data | EA | Should | |
| REQ-PF-APM-011 | Business Process Catalog | The system SHALL maintain a Business Process catalog (L1/L2/L3 hierarchy) usable for linking applications to the business processes they support | Data | EA, SA | Must | |

---

## 9. Application Portfolio Management — Capability & Process Mapping

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-APM-012 | Business Capability Map Link | The system SHALL link each application to one or more business capability map nodes (from V01 capability map), with a support-level rating (primary/supporting/absent) per capability; this produces a capability-to-application matrix | Functional | EA | Must | TOGAF Business Architecture; LeanIX |
| REQ-PF-APM-013 | Capability Coverage Heat Map | The system SHALL render a business capability map heat-mapped by application support coverage: capabilities with no application support, single-application support, or multiple-application support (redundancy risk) SHALL use distinct colors | Reporting | EA | Must | |
| REQ-PF-APM-014 | Process-to-Application Matrix | The system SHALL render a Business Process × Application matrix (heatmap table) showing which applications support each process, with redundancy (>1 app per process) and gap (0 apps per process) highlighting | Reporting | EA | Must | |
| REQ-PF-APM-015 | Application-to-Capability Traceability | The system SHALL generate a traceability report: for any selected application, list all business capabilities and processes it supports, and for any selected capability, list all supporting applications with their lifecycle states | Reporting | EA | Must | |
| REQ-PF-APM-016 | Capability Maturity per Application | The system SHALL allow scoring each application-to-capability link with a current maturity level (1–5) and target maturity level, supporting a maturity gap roadmap linked to §1 | Functional | EA | Should | |

---

## 10. Application Portfolio Management — Scoring, Health & Heat Maps

### 10.1 Fitness Scoring

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-APM-017 | Business Fit Score | The system SHALL compute a **Business Fit** score (0–100) per application from weighted dimensions: strategic alignment, business process coverage, user satisfaction (survey input), regulatory compliance status, and business owner rating; weights SHALL be configurable | Functional | EA | Must | Gartner TIME / LeanIX standard scoring |
| REQ-PF-APM-018 | Technical Fit Score | The system SHALL compute a **Technical Fit** score (0–100) per application from weighted dimensions: architecture standard alignment, tech-stack currency (from code scan or manual), security posture, availability/reliability SLA, scalability rating, and supportability; scores from code scanning (§18) SHALL auto-populate applicable dimensions | Functional | EA, AA | Must | Gartner TIME model |
| REQ-PF-APM-019 | Score Dimension Configuration | The system SHALL allow EA administrators to add, remove, rename, and reweight scoring dimensions for both Business Fit and Technical Fit without code changes; per-tenant configurations SHALL be versioned | Functional | EA | Must | |
| REQ-PF-APM-020 | Automated Score Re-computation | The system SHALL re-compute application fitness scores automatically when underlying inputs change (new code scan result, updated survey response, updated tech-component EOL date), and SHALL flag score changes exceeding a configurable threshold for human review | Functional | EA | Should | |
| REQ-PF-APM-021 | AI Score Justification | The system SHALL use AI to generate a natural-language justification for each application's business and technical fit scores, citing the top contributing factors and suggested improvement actions | AI/ML | EA | Should | |

### 10.2 Heat Maps & Portfolio Views

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-APM-022 | 2×2 Portfolio Heat Map | The system SHALL render a 2×2 scatter-plot heat map of the application portfolio with Business Fit on X-axis and Technical Fit on Y-axis (configurable), with quadrant overlays (TIME or custom labels), bubble size configurable (cost/users/complexity), and drill-down to application record on click | Reporting | EA | Must | Gartner TIME; LeanIX standard view |
| REQ-PF-APM-023 | Multi-Dimension Heat Map Table | The system SHALL render a tabular heat map with applications as rows and scoring dimensions as columns, cell-colored by score, with sort, filter, and export to Excel | Reporting | EA | Must | |
| REQ-PF-APM-024 | Lifecycle State Heat Map | The system SHALL render the portfolio heat-mapped by lifecycle phase (Plan/Invest/Maintain/Retire/Retired) over time (x = years), showing volume of applications in each phase per planning period | Reporting | EA | Must | |
| REQ-PF-APM-025 | Obsolescence Risk Heatmap | The system SHALL render an obsolescence/risk heat map showing applications with components nearing end-of-support (technology EOL data from §9 or external feed), coloring by months-to-EOL and linking to migration recommendations | Reporting | EA, CA | Must | |
| REQ-PF-APM-026 | Portfolio Summary Dashboard | The system SHALL render a portfolio summary dashboard with: total application count by lifecycle phase, average business/technical fit, total annual cost, count by hosting model, top-N by risk/cost, and trend sparklines over time | Reporting | EA | Must | |

---

## 11. Application Portfolio Management — Lifecycle, TIME Model & 6 R's

### 11.1 Application Lifecycle Management

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-APM-027 | Lifecycle Phase Model | The system SHALL model application lifecycle as: **Plan** (approved, not yet deployed) → **Invest** (active development/enhancement investment) → **Maintain** (operational, minimal investment) → **Retire** (decommission planned) → **Retired** (decommissioned); transitions SHALL be date-stamped and audited | Data | EA | Must | LeanIX / BiZZdesign lifecycle |
| REQ-PF-APM-028 | Planned Lifecycle Timeline | The system SHALL allow an EA to set planned phase-transition dates per application, producing a per-application lifecycle timeline renderable on the roadmap (§2) and the portfolio timeline view | Functional | EA | Must | |
| REQ-PF-APM-029 | AI Lifecycle Recommendation | The system SHALL use AI to recommend a lifecycle phase for each application based on fitness scores, cost, redundancy analysis, and strategic alignment, with confidence rating and rationale; recommendation SHALL reference comparable TIME/6R patterns | AI/ML | EA | Must | |

### 11.2 TIME Model

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-APM-030 | TIME Model Classification | The system SHALL assign each application a **TIME** classification — **Tolerate** (low fit, maintain for now), **Invest** (high fit, grow), **Migrate** (migrate to a better solution), **Eliminate** (decommission) — based on fitness scores and strategic input; classification SHALL be manually overridable with justification | Functional | EA | Must | Gartner TIME model (Tolerate/Invest/Migrate/Eliminate) |
| REQ-PF-APM-031 | TIME Quadrant Visualisation | The system SHALL render the TIME model as a 2×2 quadrant map (Business Fit × Technical Fit axes) with TIME labels per quadrant, with each application plotted as a labeled point and color-coded by TIME classification; consistent with §22 heat map | Reporting | EA | Must | |
| REQ-PF-APM-032 | TIME Bulk Assignment | The system SHALL allow batch TIME classification of multiple applications, with filter/sort by score range, domain, or owner, and bulk-apply of a classification with a shared justification | Functional | EA | Should | |
| REQ-PF-APM-033 | TIME Action Roadmap | The system SHALL generate a recommended action roadmap from TIME classifications: Tolerate items get a monitoring timeline; Invest items get investment prioritisation; Migrate items get 6R disposition and wave assignment (§26); Eliminate items get decommission runbooks (§25) | Functional | EA | Must | |

### 11.3 6 R's Disposition

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-APM-034 | 6 R's Disposition Assignment | The system SHALL support the 6 R's cloud/modernisation disposition model per application: **Rehost** (lift-and-shift), **Replatform** (lift-tinker-shift), **Repurchase** (replace with SaaS), **Refactor/Re-architect** (redesign for cloud-native), **Retire** (decommission), **Retain** (keep on-premises); a 7th "Replace" (custom-to-COTS) and 8th "Relocate" (VMware to cloud) MAY be supported | Data | EA, CA | Must | AWS 6 R's / Gartner 7 R's migration strategies |
| REQ-PF-APM-035 | 6 R's Decision Wizard | The system SHALL provide a guided wizard that asks structured questions (business criticality, modernisation appetite, cloud readiness, vendor viability, integration complexity) and recommends a 6R disposition with confidence and rationale; the wizard output SHALL be overridable | UX | EA, CA | Must | |
| REQ-PF-APM-036 | 6 R's Portfolio View | The system SHALL render the portfolio segmented by 6R disposition: count and cost per disposition category, with drill-down to individual applications | Reporting | EA | Must | |

---

## 12. Application Portfolio Management — Rationalization & Redundancy

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-APM-037 | Redundancy / Overlap Detection | The system SHALL automatically detect application redundancy: two or more applications supporting the same business capability or process at the same lifecycle phase, flagging them as potential consolidation candidates with a similarity score | AI/ML | EA | Must | Core APM rationalization |
| REQ-PF-APM-038 | Feature-Parity Comparison | The system SHALL provide a feature-parity comparison view for any set of applications flagged as redundant, showing overlapping and unique functional areas, data domains, and integrations, as a structured table; AI SHALL summarize the gaps and recommend a consolidation candidate | AI/ML | EA, AA | Must | Needed before sunset (§25) |
| REQ-PF-APM-039 | Shadow IT Detection | The system SHALL flag applications discovered via code scanning (§14) or connector sync (V08) that are not registered in the application registry, classifying them as potential Shadow IT and routing them for review by the EA | Functional | EA, SecA | Must | |
| REQ-PF-APM-040 | Rationalization Scenario Modelling | The system SHALL support a rationalization scenario: the EA selects a set of applications to consolidate, maps their capabilities to a surviving application, computes the capability coverage gap, and generates a recommended migration/transition plan | Functional | EA | Must | |
| REQ-PF-APM-041 | Rationalization Business Case | The system SHALL generate a rationalization business case document for a consolidation scenario: current-state cost, target-state cost, migration investment estimate, NPV/payback period, risk summary, and transition timeline; output routed to V10 | Reporting | EA | Should | |

---

## 13. Application Portfolio Management — Cost, Ownership & Vendor

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-APM-042 | Application Cost Model | The system SHALL capture annual total cost of ownership (TCO) per application, decomposed into: license/subscription, hosting/infrastructure, internal labour (FTE-months × rate), vendor support, and change/project spend; currency SHALL be configurable | Data | EA | Must | |
| REQ-PF-APM-043 | Cost Aggregation & Reporting | The system SHALL aggregate costs across the portfolio by: domain, lifecycle phase, hosting model, TIME classification, 6R disposition, and owner; report SHALL be exportable to Excel and rendered as a treemap and bar chart | Reporting | EA | Must | |
| REQ-PF-APM-044 | AI Cost Estimation | The system SHALL use AI to estimate missing TCO values for applications based on: application size (from code scan LOC/complexity), hosting model, tech-stack, and comparable applications in the portfolio; estimates SHALL be flagged as AI-derived | AI/ML | EA | Should | |
| REQ-PF-APM-045 | Vendor Contract Tracking | The system SHALL track vendor contracts per application: contract start/end, auto-renewal flag, notice period, annual value, and renewal decision status (renew/replace/terminate); contracts expiring within a configurable warning horizon SHALL surface on the portfolio dashboard | Data | EA | Should | |
| REQ-PF-APM-046 | Ownership Model | The system SHALL capture per-application: business owner (organisational unit), IT owner (team), product manager, and architect-of-record; owner lookup SHALL connect to directory services (V11 SSO/directory) | Data | EA | Must | |

---

## 14. Code Scanning — Repository Ingestion & Connectivity

> **Boundary note (reconciliation).** This volume's code-scanning capability (§14–§20) owns the **scan OUTPUTS / analytics** (parsed structure, dependency graphs, SBOM, API/data-access detection, metrics) and the **portfolio model** they feed (§8–§13). It does **not** own the **agent runtime**: deployment, registration, agent pools, and agent lifecycle for any on-prem/CI scanning agent are owned by **V17 (REQ-AGT)**. ESIP's scan engine is the *consumer* of agent-collected results; repo authentication remains a V08 connector concern (see §14.1 / REQ-PF-CS-001). See 00-MASTER-INDEX §6.

### 14.1 Repository Connectivity

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-CS-001 | Git Repository Connection | The system SHALL connect to Git repositories hosted on GitHub (cloud & Enterprise Server), GitLab (cloud & self-managed), Azure DevOps Repos, and Bitbucket (cloud & Data Center) via OAuth2 or PAT authentication; connections SHALL be managed in the KH connector framework (V08) but ESIP's code-scan engine is the consumer | Integration | EA, AA, MA | Must | Core scan-source connectivity |
| REQ-PF-CS-002 | Repository Listing & Selection | The system SHALL, after authentication, list all accessible repositories in an organisation/project, allowing the user to select individual repos, groups/projects, or all repos for scanning; selection SHALL persist as a scan configuration | Functional | EA, AA | Must | |
| REQ-PF-CS-003 | Branch & Tag Selection | The system SHALL allow the scan to target a specific branch, tag, or commit SHA; default SHALL be the repository's default branch; multiple branches MAY be scanned in parallel and their results compared | Functional | AA, MA | Should | |
| REQ-PF-CS-004 | Incremental / Delta Scanning | The system SHALL support incremental scanning: only files changed since the last scan (based on commit delta) are re-analyzed, with the full model updated accordingly; full re-scan SHALL also be triggerable on demand | Functional | EA, AA | Must | Performance requirement for large repos |
| REQ-PF-CS-005 | Local File System / Archive Upload | The system SHALL accept a zipped source archive (.zip, .tar.gz) uploaded directly when repository connectivity is not available (air-gapped, regulated environments), scanning the archive as a one-time operation | Functional | EA, AA | Should | |
| REQ-PF-CS-006 | Scan Job Management | The system SHALL track scan jobs with: status (queued/running/completed/failed), start/end timestamp, repository, branch, file counts, error log; jobs SHALL be cancellable and re-runnable | Functional | EA, AA | Must | |
| REQ-PF-CS-007 | Webhook-Triggered Scan | The system SHALL support webhook registration so that a push or merge-request event in GitHub/GitLab/ADO triggers an automatic incremental scan, keeping the application model current | Integration | AA, MA | Should | CI/CD integration |

### 14.2 Language & Framework Support

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-CS-008 | Multi-Language Parse Engine | The system SHALL parse source code in at minimum: Java, C#/.NET, Python, TypeScript/JavaScript, Go, Kotlin, Scala, Ruby, PHP, C/C++, and Swift; each language SHALL use a grammar-based AST parser (not regex), with language auto-detection | Functional | EA, AA, MA | Must | Enterprise polyglot reality |
| REQ-PF-CS-009 | Framework Recognition | The system SHALL recognize and classify frameworks/runtimes including: Spring Boot, .NET Core/ASP.NET, Django/FastAPI/Flask, React/Angular/Vue/Next.js, Node.js/Express, Rails, Laravel, Micronaut, Quarkus, gRPC, GraphQL, AKKA, and cloud-native runtimes (AWS Lambda, Azure Functions, GCP Cloud Run) | Functional | EA, AA, CA | Must | Framework detection drives stack analysis |
| REQ-PF-CS-010 | Infrastructure-as-Code Parsing | The system SHALL parse Terraform (.tf), AWS CloudFormation (YAML/JSON), Azure Bicep/ARM, Kubernetes manifests (YAML), Helm charts, and Docker/Compose files as first-class scan inputs to derive infrastructure topology and environment configuration | Functional | CA, MA | Must | |
| REQ-PF-CS-011 | Configuration File Parsing | The system SHALL parse application configuration files (application.properties, application.yml, appsettings.json, .env, web.config, pom.xml, build.gradle, package.json, Cargo.toml, go.mod) to extract database connections, service endpoints, feature flags, and dependency declarations | Functional | AA, MA | Must | |

---

## 15. Code Scanning — Parse & Structure Derivation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-CS-012 | Module / Package Boundary Detection | The system SHALL derive module and package boundaries from project structure, build descriptors, and namespace conventions, producing a hierarchical module tree per repository with LOC, file count, and language distribution per module | Functional | AA, MA | Must | CAST/Structure101 equivalent |
| REQ-PF-CS-013 | Component / Service Detection | The system SHALL detect logical components and microservices within a repository: Maven/Gradle multi-module, .NET solution projects, npm workspaces, Go modules, Kubernetes Deployments/Services, and explicitly annotated service boundaries (e.g., Spring @Service, .NET Controller) | Functional | AA, MA | Must | |
| REQ-PF-CS-014 | Class / Type Hierarchy Derivation | The system SHALL derive class/type hierarchies (inheritance, implementation, composition) from source code and represent them as a structured graph, exportable as a UML class diagram candidate (render via V03) | Functional | AA | Should | |
| REQ-PF-CS-015 | Public Interface / Contract Extraction | The system SHALL extract the public interface of each module/component: public classes, public methods with signatures (parameters, return types, exceptions), and interfaces/traits, forming the module's contract surface | Functional | AA, IA | Must | |
| REQ-PF-CS-016 | Configuration-Driven Architecture Detection | The system SHALL detect architecture patterns declared in config: service mesh sidecars (Istio/Envoy), API gateway routing rules, message broker topic/queue declarations (Kafka topics, RabbitMQ exchanges, SQS queues), and feature-flag service integrations | Functional | MA, IA | Should | |
| REQ-PF-CS-017 | Application Identity Matching | The system SHALL match derived repository components to existing application registry records using: repository URL, application name similarity, and technology fingerprint; unmatched components SHALL prompt registry population (linking to §8, REQ-PF-APM-007) | Functional | EA, AA | Must | Core scan → portfolio linkage |

---

## 16. Code Scanning — Dependency Graphs & SBOM

### 16.1 Internal Dependencies

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-CS-018 | Intra-Repo Dependency Graph | The system SHALL construct a directed intra-repository dependency graph at module/component granularity, detecting and resolving: package imports, project references (.csproj, pom.xml modules), Go module internal references, and npm workspace internal links | Data | AA, MA | Must | |
| REQ-PF-CS-019 | Cross-Repo Dependency Graph | The system SHALL construct a cross-repository dependency graph across all scanned repositories, detecting: published internal library references (by artifact ID / npm package name / PyPI package / NuGet package), shared API client usages, and IaC cross-stack references | Data | EA, AA, MA | Must | Critical for landscape mapping |
| REQ-PF-CS-020 | Circular Dependency Detection | The system SHALL detect circular dependencies (direct and transitive) in intra-repo and cross-repo dependency graphs and flag them as architectural issues with affected module list and a suggested resolution approach | Functional | AA, MA | Must | |

### 16.2 External Dependencies & SBOM

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-CS-021 | Third-Party Library Inventory | The system SHALL enumerate all third-party library dependencies per application/component with: package name, version declared, version resolved, license, and ecosystem (Maven/NPM/PyPI/NuGet/Cargo/Go modules/RubyGems/Composer) | Data | EA, AA, SecA | Must | SBOM / supply chain security |
| REQ-PF-CS-022 | SBOM Generation (CycloneDX / SPDX) | The system SHALL generate a Software Bill of Materials (SBOM) per application in CycloneDX (JSON/XML) and SPDX (JSON/Tag-Value) formats, compliant with NTIA minimum elements, exportable on demand and automatically on each scan | Functional | EA, SecA | Must | Executive Order 14028 / NIS2 SBOM requirements |
| REQ-PF-CS-023 | License Compliance Analysis | The system SHALL analyse library licenses against a configurable policy (allowed / restricted / prohibited SPDX identifiers), flagging non-compliant dependencies with severity (copyleft risk, commercial restriction), and producing a license compliance report | Functional | EA, SecA | Must | |
| REQ-PF-CS-024 | Dependency Version Currency | The system SHALL compare each declared dependency version against the current released version (via public registry APIs: Maven Central, NPM, PyPI, NuGet.org, crates.io), computing a "version lag" score and flagging critically out-of-date dependencies | Functional | EA, AA | Should | Tech-debt signal |
| REQ-PF-CS-025 | Transitive Dependency Resolution | The system SHALL resolve and display transitive (indirect) dependencies for each application to configurable depth (default: full tree), rendering them as a collapsible tree or flat table with depth indicator | Functional | AA, SecA | Should | |

---

## 17. Code Scanning — API Surface & Data Access Detection

### 17.1 API Surface

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-CS-026 | REST Endpoint Extraction | The system SHALL extract all REST API endpoints from source code annotations (Spring @RequestMapping/@GetMapping/@PostMapping, ASP.NET [HttpGet]/[HttpPost]/[Route], Express router definitions, FastAPI path operations, Rails routes.rb) with: HTTP method, path template, controller/handler, and detected request/response types | Functional | AA, IA | Must | |
| REQ-PF-CS-027 | GraphQL Schema Extraction | The system SHALL extract GraphQL schemas (schema SDL files, code-first schema definitions) and derive types, queries, mutations, and subscriptions, producing a GraphQL schema summary | Functional | AA, IA | Should | |
| REQ-PF-CS-028 | gRPC / Protobuf Service Extraction | The system SHALL parse .proto files to extract gRPC service definitions, RPC methods, request/response message types, and produce a service interface summary | Functional | AA, IA | Should | |
| REQ-PF-CS-029 | Async / Messaging Surface Extraction | The system SHALL detect message producer and consumer declarations: Kafka @KafkaListener/@KafkaProducer, Spring AMQP, AWS SQS/SNS send/receive, Azure Service Bus, NServiceBus — extracting topic/queue names, message types, and consumer group IDs | Functional | IA, MA | Must | |
| REQ-PF-CS-030 | API Surface to Interface Catalog | The system SHALL populate the Interface Catalog (§24) from extracted API surfaces, creating interface records per endpoint/topic with application linkage, replacing manual entry | Integration | EA, IA | Must | |
| REQ-PF-CS-031 | OpenAPI / AsyncAPI Document Generation | The system SHALL generate draft OpenAPI 3.x (for REST) and AsyncAPI 2.x (for messaging) documents from extracted endpoint/topic definitions, saving as artefacts linked to the application record; output routed to V10 | Functional | IA | Must | |

### 17.2 Data Access

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-CS-032 | Database Connection String Detection | The system SHALL detect database connection strings, connection pool configurations, and connection factory declarations in source and configuration files, extracting: database type (PostgreSQL, MySQL, SQL Server, Oracle, MongoDB, Redis, Cassandra, DynamoDB, CosmosDB), host pattern, and schema/database name | Functional | AA, DA | Must | |
| REQ-PF-CS-033 | ORM Schema Usage Extraction | The system SHALL parse ORM entity definitions (JPA/Hibernate @Entity, EF Core DbContext, Django models, ActiveRecord, SQLAlchemy models, Prisma schema) to derive schema entities (tables/collections), their fields, relationships, and inferred data domain | Functional | AA, DA | Must | |
| REQ-PF-CS-034 | Raw SQL Query Extraction | The system SHALL detect raw SQL queries embedded in code (string literals, query builder calls, Dapper, MyBatis XML), extract table/view references, and link them to schema entities in the data model | Functional | AA, DA | Should | |
| REQ-PF-CS-035 | Data Store to Application Mapping | The system SHALL produce a Data Store × Application matrix: for each detected data store, the applications that read or write it, with access mode (read/write/both) and access pattern (ORM/raw/REST/event) | Reporting | DA, EA | Must | |
| REQ-PF-CS-036 | Sensitive Data Pattern Detection | The system SHALL detect code patterns indicative of PII/sensitive data handling: fields named SSN, credit_card, password, auth_token, dob, health_record (configurable pattern set), flagging them for data-privacy architectural review; results surface in the security dashboard | Security | SecA, DA | Must | GDPR / data privacy architecture |

---

## 18. Code Scanning — Tech-Debt, Quality & Vulnerability Surfacing

### 18.1 Tech Debt & Quality

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-CS-037 | Tech-Debt Detection Engine | The system SHALL detect and quantify technical debt indicators per codebase: deprecated API usage, outdated framework version usage, code smells (duplicated code blocks, excessive method length, deep nesting via AST), missing test coverage, unused/dead code, and TODO/FIXME/HACK comment density | Functional | AA, EA | Must | CAST AIP / SonarQube equivalence |
| REQ-PF-CS-038 | Code Quality Score | The system SHALL compute a composite **Code Quality Score** (0–100) per application/component, derived from: cyclomatic complexity, duplication ratio, test coverage (if test reports provided), documented public APIs ratio, and tech-debt hours (SQALE-inspired); this score SHALL feed Technical Fit in APM (REQ-PF-APM-018) | Functional | AA, EA | Must | SonarQube SQALE methodology |
| REQ-PF-CS-039 | Deprecated API / EOL Framework Flags | The system SHALL flag usage of: runtime/framework versions past vendor-declared end-of-support dates (from a configurable EOL data feed or manual registry), deprecated library APIs (detected from deprecation annotations @Deprecated, [Obsolete], #[deprecated]), and known EOL language versions | Functional | EA, AA | Must | |
| REQ-PF-CS-040 | AI Tech-Debt Summary | The system SHALL use AI to produce a human-readable tech-debt summary per application: top-3 debt areas, estimated remediation effort (S/M/L/XL), recommended priority, and linkage to the architecture quality improvement roadmap | AI/ML | EA, AA | Must | |
| REQ-PF-CS-041 | Code Quality Trend Over Time | The system SHALL track code quality and tech-debt scores across scan history and render a trend chart (quality score, debt count, complexity) per application over time; regressions SHALL be flagged | Reporting | AA, MA | Should | |

### 18.2 Vulnerability Surface

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-CS-042 | Known Vulnerability Detection (CVE) | The system SHALL match third-party library versions (from SBOM, §16) against the NIST NVD / OSV.dev / GitHub Advisory Database to identify known CVEs, reporting: CVE ID, CVSS score, affected component, version range, and fixed version | Security | SecA, EA | Must | Supply chain security; compliance |
| REQ-PF-CS-043 | Vulnerability Severity Triage | The system SHALL classify detected CVEs by: CVSS severity (Critical/High/Medium/Low/Informational), reachability (whether the vulnerable code path is reachable from application entry points — where statically determinable), and exploitability context; Critical/High CVEs SHALL generate portfolio-level alerts | Security | SecA | Must | |
| REQ-PF-CS-044 | SAST Pattern Detection | The system SHALL perform static application security testing (SAST) pattern matching for OWASP Top-10 code patterns: SQL injection (string concatenation in queries), hardcoded credentials, insecure deserialization, path traversal, and XXE patterns; findings SHALL be classified by CWE | Security | SecA, AA | Must | OWASP / application security |
| REQ-PF-CS-045 | Secret / Credential Scan | The system SHALL detect hardcoded secrets, API keys, and credentials in source and config files using entropy analysis and pattern libraries (aligned with tools like truffleHog/gitleaks patterns); findings SHALL never be stored in plaintext — only offsets and sanitized context are retained | Security | SecA | Must | |
| REQ-PF-CS-046 | Security Risk Score | The system SHALL compute a **Security Risk Score** per application (0–100, higher = more risk) aggregating: CVE severity distribution, SAST finding counts, secret scan hits, and EOL framework flags; this score SHALL feed Technical Fit in APM | Security | SecA, EA | Must | |

---

## 19. Code Scanning — Metrics & Complexity

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-CS-047 | Lines of Code (LOC) | The system SHALL compute LOC (total, code-only, comment, blank) per file, module, component, and application, with language breakdown; LOC SHALL be used as a sizing signal in cost estimation (§13) and migration effort scoring (§26) | Functional | EA, AA | Must | |
| REQ-PF-CS-048 | Cyclomatic Complexity | The system SHALL compute cyclomatic complexity per method/function and aggregate to module/component level (average, max, P90), flagging methods above a configurable threshold as high-complexity hotspots | Functional | AA | Must | |
| REQ-PF-CS-049 | Coupling & Cohesion Metrics | The system SHALL compute afferent coupling (Ca), efferent coupling (Ce), and instability index (Ce/(Ca+Ce)) per module/package (Robert C. Martin metrics); high-instability core modules SHALL be flagged as architectural risk | Functional | AA, MA | Must | Clean Architecture / Robert C. Martin |
| REQ-PF-CS-050 | Cognitive Complexity | The system SHALL compute cognitive complexity (as defined by SonarSource) per method, surfacing the top-N most cognitively complex functions per application as refactoring candidates | Functional | AA | Should | SonarQube cognitive complexity |
| REQ-PF-CS-051 | Test Coverage Integration | The system SHALL ingest JaCoCo (Java), .NET coverage (Cobertura/lcov), pytest-cov, Istanbul/NYC (JS), and GoTest coverage reports (uploaded or CI-pipeline pushed), storing line/branch coverage per module and surfacing applications with coverage below a configurable threshold | Integration | AA, MA | Should | |
| REQ-PF-CS-052 | Code Hotspot Detection | The system SHALL combine complexity, coupling, CVE presence, and tech-debt density to produce a **Code Hotspot** ranking — the top-N most risky files/modules in the portfolio — prioritised for review and refactoring | Reporting | EA, AA | Must | |

---

## 20. Code Scanning — Monorepo & Multi-Repo Support

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-CS-053 | Monorepo Component Decomposition | The system SHALL decompose a monorepo into logical components based on: directory structure conventions (apps/, services/, packages/, libs/), build descriptor presence (package.json per package, build.gradle per subproject, .csproj), and Nx/Turborepo/Bazel workspace configuration | Functional | MA, AA | Must | Large-org monorepo reality |
| REQ-PF-CS-054 | Monorepo Ownership Map | The system SHALL detect per-component ownership in a monorepo from CODEOWNERS files, package.json maintainers, or configurable path → team mappings, producing a component ownership matrix | Functional | MA, EA | Should | |
| REQ-PF-CS-055 | Multi-Repo Portfolio Scan | The system SHALL scan multiple repositories in a single portfolio scan operation, producing a unified application portfolio view with cross-repo dependency graph, aggregated metrics, and a combined SBOM | Functional | EA | Must | Portfolio-level scan |
| REQ-PF-CS-056 | Scan Results Federation | The system SHALL federate scan results from external tools — SonarQube, CAST Highlight, Snyk, Checkmarx, Veracode — by ingesting their exported JSON/XML reports and merging findings into ESIP's quality/security model, avoiding duplication of analysis effort | Integration | SecA, EA | Should | Leverages existing enterprise tooling investment |
| REQ-PF-CS-057 | Scan Exclusion Rules | The system SHALL allow configuration of scan exclusion rules by path pattern (e.g., */test/**, */vendor/**, */generated/**), file extension, and language, applied per repository; exclusions SHALL be logged for audit | Functional | AA | Must | |

---

## 21. Flow Derivation — Application & Call-Graph Flows

### 21.1 Call Graph & Sequence

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-FD-001 | Inter-Module Call Graph | The system SHALL derive an inter-module call graph from parsed ASTs, recording which modules/components call which other modules/components (static call analysis), and render it as a directed graph; edges SHALL be labeled with method/function names | Functional | AA, MA | Must | Structure101 / CAST equivalent |
| REQ-PF-FD-002 | Entry-Point to Sequence Derivation | The system SHALL derive, for selected entry points (API endpoints, message consumers, scheduled jobs), a call sequence graph tracing: entry point → controller/handler → service → repository → external API calls; depth SHALL be configurable | Functional | AA | Must | |
| REQ-PF-FD-003 | Sequence Diagram Candidate Generation | The system SHALL convert derived call sequences to sequence diagram data models (participant list, message list, return types, async indicators), passable to V03 for rendering as UML Sequence Diagrams or ArchiMate Application Interaction viewpoints | Functional | AA, SA | Must | Boundary: rendering → V03 |
| REQ-PF-FD-004 | AI Sequence Simplification | The system SHALL use AI to simplify deep/wide call graphs to architecturally significant abstractions (eliding internal utility calls), producing a "meaningful sequence" suitable for architecture diagrams rather than code-level traces | AI/ML | AA, SA | Must | V06 AI reasoning |
| REQ-PF-FD-005 | Runtime Trace Import | The system SHALL accept distributed trace data (OpenTelemetry OTLP, Jaeger, Zipkin export formats) as supplemental input to validate and augment statically derived call graphs with actual runtime call paths | Integration | MA, AA | Could | Production trace validation |

### 21.2 Event & Async Flow

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-FD-006 | Event Flow Derivation | The system SHALL derive event/message flows from detected producer/consumer declarations (§17, REQ-PF-CS-029): producer application → topic/queue → consumer application(s), producing an event-flow graph with message types | Functional | IA, MA | Must | |
| REQ-PF-FD-007 | Choreography vs. Orchestration Detection | The system SHALL detect and label async communication patterns: choreography (event-driven, no central orchestrator) vs. orchestration (saga orchestrator, BPM engine calls, workflow engine references), surfacing the dominant pattern per bounded context | AI/ML | MA, IA | Should | DDD/microservices pattern analysis |

---

## 22. Flow Derivation — Business Process & Data Flow Derivation

### 22.1 Business Process Flow

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-FD-008 | BPMN Candidate from Code | The system SHALL derive a Business Process flow candidate from: workflow engine configurations (Camunda BPMN files, Temporal workflows, AWS Step Functions ASL, Azure Durable Functions definitions), annotated saga orchestrators, and documented state machine transitions, producing a BPMN 2.0 data model passable to V03 for rendering | Functional | AA, IA | Must | Boundary: rendering → V03 |
| REQ-PF-FD-009 | AI Business Process Label Inference | The system SHALL use AI to infer human-readable business process names and activity labels from: method/class names, URL path patterns, message topic names, and variable names, converting technical identifiers to business-meaningful flow labels | AI/ML | AA, SA | Must | V06 AI reasoning |
| REQ-PF-FD-010 | Business Process to Application Linkage | The system SHALL link derived business process flows to the Application Portfolio (§8), mapping each BPMN activity to the application component that executes it, producing an Application × Process participation map | Integration | EA, AA | Must | |

### 22.2 Data Flow Derivation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-FD-011 | Data Flow Diagram (DFD) Candidate | The system SHALL derive a Level-0 DFD candidate from: detected external data stores (§17, REQ-PF-CS-032), external API calls (HTTP client usages, REST client configurations, gRPC stubs), user-facing entry points, and data transformation classes; produce a DFD data model passable to V03 | Functional | DA, SecA, SA | Must | Boundary: rendering → V03 |
| REQ-PF-FD-012 | Data Lineage Derivation | The system SHALL trace data lineage: from ingestion endpoint → transformation component → data store → outgoing API, across multiple applications in the portfolio, producing a cross-application data lineage graph for DA and EA review | Functional | DA, EA | Must | |
| REQ-PF-FD-013 | PII Data Flow Highlighting | The system SHALL overlay detected PII/sensitive field patterns (§17, REQ-PF-CS-036) on the data flow diagram, highlighting flows that carry sensitive data so security and data architects can identify exposure points | Security | SecA, DA | Must | GDPR / Privacy-by-Design |
| REQ-PF-FD-014 | DFD to Threat Model Feed | The system SHALL export the derived DFD as input to a STRIDE threat model, identifying trust boundaries (internal/external system boundaries detected from API surfaces) and data stores accessible to multiple applications; output feeds V06 AI security analysis | Security | SecA | Should | STRIDE methodology |

---

## 23. Flow Derivation — Diagram Candidate Generation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-FD-015 | C4 Model Candidate Generation | The system SHALL auto-generate C4 model candidates (Context, Container, Component levels) from code-scan results: applications/components become C4 Containers; modules become C4 Components; detected API calls across applications become C4 relationships; these are data models passable to V03 for rendering | Functional | AA, SA | Must | C4 Model (Simon Brown); Boundary: rendering → V03 |
| REQ-PF-FD-016 | ArchiMate Application Layer Candidate | The system SHALL generate an ArchiMate Application Layer viewpoint candidate from the portfolio scan: Application Components, Application Functions (from services), Application Interfaces (from API surfaces), and Application Interactions (from call/event graphs); passable to V03 | Functional | EA, AA | Must | ArchiMate 3.2 Application Layer; Boundary → V03 |
| REQ-PF-FD-017 | ArchiMate Technology Layer Candidate | The system SHALL generate an ArchiMate Technology Layer viewpoint candidate: System Software (runtime/OS/container), Technology Services (database services, messaging services), Nodes (from IaC scans), and Artifacts (deployables); passable to V03 | Functional | EA, CA | Must | ArchiMate 3.2 Technology Layer; Boundary → V03 |
| REQ-PF-FD-018 | Human Review & Edit Before Render | The system SHALL present all derived diagram candidates as a structured data model in a review pane before passing to V03 for rendering, allowing the architect to: accept, reject, or edit individual elements/relationships; approved candidates SHALL be versioned as "AI-derived, human-reviewed" | UX | ALL | Must | Human-in-the-loop requirement |
| REQ-PF-FD-019 | Diagram Candidate Confidence Score | The system SHALL attach a confidence score to each derived diagram element, indicating whether it was detected with high confidence (from explicit annotations/declarations) or inferred (from heuristics/AI); low-confidence elements SHALL be highlighted in the review pane | AI/ML | ALL | Should | Transparency / trust |
| REQ-PF-FD-020 | Derivation Audit Trail | The system SHALL maintain an audit trail for each derived diagram element: source file(s) and line number(s), detection rule or AI inference that produced it, and the architect who reviewed it; this trail SHALL be viewable from the diagram element | Governance | EA, AA | Must | Traceability requirement |

---

## 24. Landscape & Dependency Mapping

### 24.1 Integration / Interface Catalog

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-LD-001 | Interface Catalog | The system SHALL maintain an Interface Catalog with one record per integration point: source application, target application (or external party), protocol (REST/SOAP/gRPC/messaging/file/DB-link), data exchanged, frequency, criticality, and owner; populated automatically from code scan (§17) and manually | Data | EA, IA | Must | EA integration landscape standard |
| REQ-PF-LD-002 | Integration Pattern Classification | The system SHALL classify each interface by integration pattern: request/response, event/async, batch/file, database-link, API gateway-mediated, ESB-mediated, point-to-point; classification SHALL use AI where pattern is ambiguous | AI/ML | IA, EA | Must | EIP patterns |
| REQ-PF-LD-003 | Integration Landscape Map | The system SHALL render a visual Integration Landscape Map (spaghetti / dependency diagram) of all applications and their interfaces, with: edge weight proportional to interface count, filter by protocol/pattern/domain, cluster by business domain, and highlight circular / high-fan-out nodes | Reporting | EA, IA | Must | |
| REQ-PF-LD-004 | Interface Impact Analysis | The system SHALL, for any selected application, compute and display the **Blast Radius**: which upstream applications call it, which downstream applications it calls, and the transitive impact chain to n hops; used before decommission (§25) or migration (§26) | Functional | EA, SA | Must | |
| REQ-PF-LD-005 | External System / Third-Party Registry | The system SHALL maintain a registry of external systems (third-party APIs, partner integrations, SaaS providers), linking them to the interface catalog and the application portfolio | Data | EA, IA | Should | |

### 24.2 Cross-Portfolio Dependency Intelligence

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-LD-006 | Shared-Library Dependency Map | The system SHALL identify shared internal libraries used by multiple applications (from cross-repo dependency graph, §16) and render a shared-library fan-out map: for each library, which applications depend on it, enabling centralised upgrade/retirement planning | Reporting | EA, AA | Must | |
| REQ-PF-LD-007 | Technology Sprawl Analysis | The system SHALL analyse the portfolio for technology sprawl: the number of distinct technology choices per technology category (databases, languages, frameworks, messaging systems), flagging domains where standardisation could reduce complexity; AI SHALL suggest consolidation candidates | AI/ML | EA | Must | |
| REQ-PF-LD-008 | Data Store Sharing Detection | The system SHALL detect shared data stores: a single database instance or schema accessed by more than one application, flagging it as a shared-data-architecture risk and suggesting: separate schemas, API boundary, or event-based decoupling | Functional | DA, EA, MA | Must | Anti-pattern detection |
| REQ-PF-LD-009 | Dependency Graph Export | The system SHALL export the full cross-application dependency graph in GraphML, JSON (Cytoscape.js/D3), and ArchiMate Model Exchange Format (AMEF) for use in external graph analytics or rendering tools | Integration | EA, AA | Should | |

---

## 25. Sunset / Decommission Planning

### 25.1 Legacy & Sunset Identification

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-SD-001 | Legacy Application Identification | The system SHALL identify legacy and sunset candidates by combining: TIME = Eliminate, 6R = Retire, lifecycle phase = Maintain/Retire, technical fit score below configurable threshold, EOL framework flags from code scan, and zero active development (commit frequency), producing a prioritized legacy list | Functional | EA | Must | |
| REQ-PF-SD-002 | Sunset Candidate List | The system SHALL present a Sunset Candidate List with: application name, reason for flagging (rule + supporting evidence), business criticality, dependency count (from landscape map), estimated annual cost saving from retirement, and recommended action (immediate retire / migrate first / feature-parity check required) | Reporting | EA | Must | |
| REQ-PF-SD-003 | AI Sunset Recommendation | The system SHALL use AI to analyze the portfolio and proactively identify sunset candidates not caught by rule-based filters, using signals: low business/technical fit, duplicate capability coverage, diminishing user base (if usage data available), and tech-debt accumulation trend | AI/ML | EA | Must | Flagship capability |
| REQ-PF-SD-004 | Redundancy-Driven Sunset | The system SHALL integrate redundancy detection (§12, REQ-PF-APM-037) with sunset planning: applications flagged as redundant and lower-scored than their counterpart SHALL be automatically elevated to the sunset candidate list | Functional | EA | Must | |

### 25.2 Feature-Parity Gap Analysis

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-SD-005 | Feature-Parity Gap Analysis | The system SHALL perform a feature-parity gap analysis before a sunset decision: compare the retiring application's functional scope (from requirements V07, API surface §17, and capability mappings §9) against the surviving/replacement application; gaps SHALL be classified as: covered, partially covered (mapping required), or missing (new development required) | Functional | EA, AA | Must | Pre-sunset safety check |
| REQ-PF-SD-006 | Gap-to-Requirement Traceability | The system SHALL link each identified parity gap to a requirement record (V07) and to a roadmap item (§1) that will close the gap before the sunset date, ensuring no business function is lost at retirement | Integration | EA | Must | Traceability |
| REQ-PF-SD-007 | Feature-Parity Report | The system SHALL generate a Feature-Parity Report document: retiring application summary, replacement application summary, gap table (feature, status, resolution action, owner, target date), and an overall readiness score (% features covered); output routed to V10 | Reporting | EA | Must | |
| REQ-PF-SD-008 | User-Base Impact Assessment | The system SHALL estimate the user-base impacted by a sunset (from usage data if available, or stakeholder mapping), and surface the communication and change-management actions required in the decommission runbook | Functional | EA | Should | |

### 25.3 Decommission Runbook

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-SD-009 | Decommission Runbook Generation | The system SHALL auto-generate a **Decommission Runbook** for a sunset application: phase-by-phase checklist (preparation, migration/cutover, data archival, integration cutover, user communication, final shutdown, post-decommission validation), with configurable templates per application type | Functional | EA, AA | Must | |
| REQ-PF-SD-010 | Integration Cutover Plan | The system SHALL derive an integration cutover plan from the interface catalog (§24): for each interface to the retiring application, identify the replacement target, required changes in the calling application, and proposed cutover sequence | Functional | IA, EA | Must | |
| REQ-PF-SD-011 | Data Archival Plan | The system SHALL generate a data archival plan section in the runbook: data stores owned by the retiring application, retention requirements (from regulatory tags on data entities), archival destination, and deletion schedule | Functional | DA, EA | Must | |
| REQ-PF-SD-012 | Decommission Risk Register | The system SHALL maintain a decommission-specific risk register per retiring application: blast-radius risk (§24 REQ-PF-LD-004), data loss risk, user disruption risk, regulatory compliance risk; each risk SHALL have a mitigation and contingency | Functional | EA | Must | |
| REQ-PF-SD-013 | Decommission Timeline on Roadmap | The system SHALL place the decommission plan (phases and milestones from the runbook) on the Migration Roadmap type (§2, REQ-PF-RM-014), so it is visible in the context of all other transformation work | Integration | EA | Must | |
| REQ-PF-SD-014 | Post-Decommission Validation | The system SHALL provide a post-decommission validation checklist: verify no remaining network traffic to old endpoints, confirm data archival completion, validate replacement application user acceptance, and update the application registry to status = Retired | Functional | EA, AA | Should | |

---

## 26. Migration Planning — Disposition & Wave Design

### 26.1 Migration Assessment

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-MIG-001 | Migration Scope Definition | The system SHALL allow an EA to define a migration program scope: a named migration program, selected source environment (on-premises DC, legacy platform, vendor), target environment (cloud provider/region, new platform, modernised platform), and in-scope application set from the portfolio | Data | EA, CA | Must | |
| REQ-PF-MIG-002 | Migration Readiness Assessment | The system SHALL compute a **Migration Readiness Score** per application: cloud/platform readiness indicators from code scan (containerisability — Dockerfile present, statelessness indicators, config externalisation, hardcoded IPs), dependency complexity, and manual readiness questionnaire; score SHALL feed wave planning | Functional | EA, CA | Must | |
| REQ-PF-MIG-003 | 6 R's Auto-Recommendation | The system SHALL use AI to recommend a 6R disposition for each application in the migration scope, combining: migration readiness score, TCO analysis, vendor availability (SaaS replacement detection), tech-stack cloud compatibility, and time-to-value; recommendation SHALL be justifiable and overridable | AI/ML | EA, CA | Must | AWS 6 R's framework |
| REQ-PF-MIG-004 | Migration Cost Estimation | The system SHALL estimate per-application migration cost by disposition: Rehost cost (lift-shift infrastructure delta), Replatform cost (containerisation + config changes estimate from code scan), Refactor cost (LOC-based complexity × estimated hourly rate × a configurable refactoring coefficient), Repurchase cost (SaaS licensing estimate), Retire cost (decommission effort), Retain cost (zero migration, ongoing TCO delta) | Functional | EA, CA | Should | |
| REQ-PF-MIG-005 | Migration Business Case | The system SHALL generate a Migration Program Business Case: total migration investment estimate, annual run-cost delta (current vs. target), NPV at 3/5 years, strategic benefit narrative (AI-authored), top-5 risks, and a recommended program timeline; output routed to V10 | Reporting | EA | Must | |

### 26.2 Wave & Move-Group Planning

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-MIG-006 | Move Group Definition | The system SHALL support **Move Groups** (also called migration waves): a named set of applications to be migrated together in a coordinated move, with: group ID, migration window, target state, and owning migration team | Data | EA, CA | Must | Cloud migration wave planning standard |
| REQ-PF-MIG-007 | AI Move Group Suggestion | The system SHALL use AI to suggest optimal move group composition based on: application dependency graph (tightly coupled applications in the same group), team ownership, readiness scores, business criticality (lower risk first), and desired wave cadence (configurable) | AI/ML | EA, CA | Must | Flagship AI-assisted migration planning |
| REQ-PF-MIG-008 | Dependency-Aware Wave Sequencing | The system SHALL sequence migration waves respecting the dependency graph: an application that calls another SHALL be in the same wave or a later wave than its dependency; circular dependencies SHALL be flagged with a recommended resolution (anti-corruption layer, phased strangler fig) | Functional | EA, CA | Must | Core migration planning constraint |
| REQ-PF-MIG-009 | Wave Conflict Detection | The system SHALL detect move-group conflicts: two mutually dependent applications in different waves with no interim adapter strategy, missing readiness gate satisfaction, over-capacity waves (too many applications for team bandwidth), and conflicting change-freeze windows | Functional | EA | Must | |
| REQ-PF-MIG-010 | Wave Progress Tracking | The system SHALL track migration wave progress: per-application migration status (not-started / in-analysis / in-progress / validated / completed / rolled-back), milestone completion, and wave-level percent-complete; dashboard SHALL show overall program health | Functional | EA, CA | Must | |
| REQ-PF-MIG-011 | Migration Roadmap View | The system SHALL render the migration program as a Migration Roadmap (§2, REQ-PF-RM-014): waves on the time axis, swimlanes by domain or team, applications as bars colored by 6R disposition, dependency arrows between waves, and milestone markers for readiness gates | Reporting | EA | Must | |

---

## 27. Migration Planning — Patterns & Target-State Mapping

### 27.1 Modernisation Patterns

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-MIG-012 | Strangler Fig Pattern Support | The system SHALL support the **Strangler Fig** modernisation pattern: the EA defines a legacy application to strangle, a target application to grow, a set of capabilities to migrate incrementally, and a routing proxy configuration; the system SHALL track capability migration progress and generate a phase-by-phase transition architecture (§3) per capability tranche | Functional | EA, MA | Must | Martin Fowler Strangler Fig pattern |
| REQ-PF-MIG-013 | Anti-Corruption Layer Design Aid | The system SHALL assist in designing an **Anti-Corruption Layer (ACL)**: given a source API surface (from code scan, §17) and a target API specification, AI SHALL identify translation mappings, flag semantic mismatches, and generate a mapping specification passable to V03 for diagram rendering | AI/ML | SA, IA | Must | DDD ACL pattern |
| REQ-PF-MIG-014 | Branch-by-Abstraction Detection | The system SHALL detect codebases using Branch-by-Abstraction (feature-flag-driven or interface-based): abstract interfaces with multiple implementations including a "legacy" and "new" implementation, flagging them as candidates for completing the migration | Functional | AA | Should | Branch-by-Abstraction (Paul Hammant) |
| REQ-PF-MIG-015 | Containerisation Readiness | The system SHALL assess containerisation readiness per application: stateless/stateful analysis (session state in-process vs. externalised, file-system writes), config externalisation (hardcoded config vs. environment-variable usage), and port binding compliance (12-factor), producing a per-app containerisation checklist | Functional | CA, MA | Must | 12-Factor App / cloud-native readiness |
| REQ-PF-MIG-016 | Cloud-Native Pattern Recommendations | The system SHALL recommend cloud-native patterns appropriate to each application post-migration: managed database (DB modernisation), serverless function decomposition (stateless functions with low coupling), event-driven decoupling (replace synchronous chains with events), service-mesh adoption; recommendations SHALL be linked to the migration roadmap | AI/ML | CA, MA | Should | |

### 27.2 Target-State Mapping

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-MIG-017 | Target Architecture State | The system SHALL allow the EA to define a **Target Architecture State** for the migration program: target application portfolio (post-migration registry), target technology stack standards, target hosting model (cloud provider/region/landing zone), and target integration patterns; the target state SHALL be linked as the endpoint plateau in the transition architecture chain (§3) | Data | EA, CA | Must | TOGAF ADM Phase E |
| REQ-PF-MIG-018 | Current-to-Target Gap Map | The system SHALL compute the Current-to-Target Gap Map: applications present in current portfolio but absent in target (retire candidates), applications present in target but absent in current (new builds/purchases), and applications mapped 1:1 with a 6R disposition transformation; gap map SHALL feed the migration roadmap | Functional | EA | Must | |
| REQ-PF-MIG-019 | Landing Zone Mapping | The system SHALL allow each application to be assigned to a target landing zone (cloud account/subscription/project, VPC/VNet, region), producing a landing zone population view overlaid on the target architecture; IaC templates (Terraform/Bicep) for landing zones MAY be generated or linked | Functional | CA | Should | Cloud landing zone design |
| REQ-PF-MIG-020 | Migration Risk Heat Map | The system SHALL render a migration risk heat map: x-axis = migration complexity (from readiness score), y-axis = business criticality, bubble size = annual cost, color = 6R disposition; quadrant labels (e.g., "Quick Wins", "Strategic Projects", "Complex/Critical", "Retire First") SHALL be configurable | Reporting | EA, CA | Must | |

---

## 28. Cross-Volume Linkage

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-PF-LNK-001 | Portfolio ↔ Roadmap Linkage | The system SHALL maintain bidirectional links between application portfolio records and roadmap items: a roadmap item (initiative/epic) SHALL list the applications it affects; an application record SHALL display its associated roadmap items, lifecycle milestones, and migration wave | Integration | EA | Must | Cross-pillar traceability |
| REQ-PF-LNK-002 | Portfolio ↔ Requirements Linkage | The system SHALL link application portfolio records to requirements (V07): each application in the portfolio SHALL display the requirements it fulfils; each requirement SHALL show which applications implement it; unimplemented requirements (no application) and unsupported applications (no requirement) SHALL surface as gaps | Integration | EA, SA | Must | TOGAF Requirements Management |
| REQ-PF-LNK-003 | Portfolio ↔ Target Architecture Linkage | The system SHALL link each application to the architecture model elements (from V03 modeling engine): ArchiMate Application Components, C4 Containers, or UML Components; changes in the application registry SHALL propagate to model element attributes (name, lifecycle) | Integration | EA | Must | |
| REQ-PF-LNK-004 | Portfolio ↔ Deliverables Linkage | The system SHALL link any generated deliverable (V10) — roadmap deck, portfolio report, decommission runbook, migration business case — to the portfolio records and roadmap items that sourced it, so the deliverable can be regenerated with updated data on demand | Integration | EA | Must | V10 deliverables |
| REQ-PF-LNK-005 | Code Scan ↔ Portfolio Feedback Loop | The system SHALL continuously feed code-scan results back into the portfolio: updated technical fit scores (from quality/security scan), updated SBOM and EOL flags, updated LOC/complexity (cost proxy), and updated API surface (interface catalog), keeping the portfolio current without manual re-entry | Functional | EA, AA | Must | Core automated feedback loop |
| REQ-PF-LNK-006 | Roadmap ↔ Capability Map Linkage | The system SHALL link roadmap items to business capability map nodes (V01/V09 §9), so the capability roadmap type (REQ-PF-RM-012) can visualise capability maturity progression; selecting a capability on the map SHALL show all roadmap items that target it | Integration | EA | Must | TOGAF capability roadmap |
| REQ-PF-LNK-007 | Migration Plan ↔ Transition Architecture Linkage | The system SHALL link each migration wave to a transition architecture plateau (§3), so the wave completion produces a named intermediate architecture state, and the full sequence of waves traces the path from baseline to target through defined plateaus | Integration | EA | Must | TOGAF ADM Phase F / ArchiMate I&M |
| REQ-PF-LNK-008 | AI Analysis Output ↔ Portfolio Records | The system SHALL persist all AI-generated outputs (TIME recommendations, 6R recommendations, tech-debt summaries, redundancy analyses, sunset recommendations) as structured data attached to portfolio records, with: AI model version, timestamp, confidence, and review status (pending / accepted / overridden); these SHALL feed V06 analysis provenance | AI/ML | EA | Must | AI provenance / traceability (V06) |
| REQ-PF-LNK-009 | V06 AI Reasoning Contract | The system SHALL expose the following structured data payloads to the V06 AI engine for reasoning tasks: application portfolio graph (registry + relationships + scores), dependency graph (cross-repo), interface catalog, roadmap items, transition architectures, and code-scan findings; V06 SHALL return structured recommendation objects (not free text only) conformant to a defined schema | Integration | EA | Must | Boundary: AI reasoning → V06 |
| REQ-PF-LNK-010 | V03 Diagram Candidate Contract | The system SHALL expose diagram candidate data models (C4, ArchiMate Application/Technology layers, DFD, BPMN, Sequence) as structured JSON conformant to V03's ingestion schema (defined in V03), such that V03 can render them without custom per-V09 code | Integration | EA, AA | Must | Boundary: rendering → V03 |

---

## Requirement Count Summary

| Section | Sub-prefix | Count |
|---|---|---|
| §1–§7 Roadmaps | REQ-PF-RM | 52 |
| §8–§13 Application Portfolio Management | REQ-PF-APM | 46 |
| §14–§20 Code Scanning | REQ-PF-CS | 57 |
| §21–§23 Flow Derivation | REQ-PF-FD | 20 |
| §24 Landscape & Dependency | REQ-PF-LD | 9 |
| §25 Sunset / Decommission | REQ-PF-SD | 14 |
| §26–§27 Migration Planning | REQ-PF-MIG | 20 |
| §28 Cross-Volume Linkage | REQ-PF-LNK | 10 |
| **Total** | **PF** | **228** |

---

## Models, Frameworks & Tooling References Incorporated

| Domain | Reference |
|---|---|
| Roadmap types | TOGAF ADM Phase F; ArchiMate I&M viewpoint; SAFe PI/Release Train; ProductBoard/Aha! Now-Next-Later; BiZZdesign Enterprise Architecture |
| APM lifecycle | LeanIX Application Lifecycle; Ardoq APM; MEGA HOPEX |
| TIME model | Gartner TIME (Tolerate / Invest / Migrate / Eliminate) |
| 6 R's / 7 R's | AWS 6 R's Migration Strategies; Gartner 7 R's; Forrester FAST Cloud framework |
| SBOM standards | CycloneDX 1.5; SPDX 2.3; NTIA Minimum Elements; CISA SBOM guidance |
| Code quality | SonarQube SQALE / Cognitive Complexity; CAST AIP; Structure101; Robert C. Martin coupling metrics |
| Security scanning | OWASP Top-10; NIST NVD; OSV.dev; GitHub Advisory Database; CWE; CVSSv3.1; SAST patterns |
| Dependency mgmt | Maven Central; NPM; PyPI; NuGet.org; crates.io; Go Module Proxy |
| Flow / diagram notation | C4 Model (Simon Brown); ArchiMate 3.2 (The Open Group); BPMN 2.0 (OMG); UML 2.5; DFD Level 0/1 |
| Modernisation patterns | Strangler Fig (Fowler); Anti-Corruption Layer (Evans DDD); Branch-by-Abstraction (Hammant); 12-Factor App; BFF; Saga |
| Tracing | OpenTelemetry OTLP; Jaeger; Zipkin |
| IaC | Terraform HCL; AWS CloudFormation; Azure Bicep/ARM; Kubernetes YAML; Helm; Docker Compose |
| Enterprise connectors | ServiceNow CMDB; Jira; Azure DevOps; GitHub; GitLab; Bitbucket |

---

## Boundary Notes & Assumptions

1. **V06 (AI engine) boundary:** All AI/LLM inference calls (sequence simplification, TIME/6R recommendations, tech-debt narrative, redundancy detection, scenario generation) are *invoked via V06*. This volume specifies the required input payloads and output schemas; V06 owns model selection, prompt management, and guardrails.
2. **V03 (Diagramming) boundary:** All diagram rendering (ArchiMate viewpoints, C4 diagrams, BPMN, DFD, Sequence, Gantt, heat maps) is *delegated to V03*. This volume produces structured data model objects; V03 renders them on its canvas.
3. **V08 (Knowledge Hub / connectors) boundary:** Repository OAuth connections and CMDB API connectors are *registered and authenticated in V08*. V09 consumes the connection credentials/tokens from V08 to perform code scanning; it does not manage connector lifecycle.
4. **V10 (Deliverables) boundary:** All generated documents (migration business case, feature-parity report, decommission runbook, portfolio reports, roadmap decks) are *rendered and exported by V10*. V09 provides the structured content payloads.
5. **V07 (Requirements) boundary:** Requirements traceability (portfolio ↔ requirements) assumes V07 has persisted requirement records with stable IDs; V09 links to those IDs.
6. **V11 (Platform) boundary:** RBAC, audit logging, multi-tenancy, SSO/directory integration, and collaboration (comments, notifications) for all V09 features are governed by V11.
7. **Runtime trace data (REQ-PF-FD-005)** is listed as "Could" — it requires infrastructure access beyond source code; it is included for completeness of the vision.
8. **Cost estimation formulas** (REQ-PF-APM-044, REQ-PF-MIG-004) are AI-estimated and heuristic; they are architectural signals, not financial accounting.
9. **External EOL feeds** for technology components (REQ-PF-APM-025, REQ-PF-CS-039) are sourced from endoflife.date API or equivalent; the connector to that feed is a V11 platform concern.
