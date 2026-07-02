# Volume 20 — Frontier & Emerging Capabilities

**Prefix:** `REQ-FRO-`
**Status:** Requirements Catalog — Final Volume (Frontier / Forward-Looking Differentiators)
**Date:** 2026-06-23
**Audience:** Product Owner, Principal Architect, Roadmap Committee

---

## Scope & Boundary

This volume covers capabilities that keep ESIP ahead of the market for 3–7 years. They span nine distinct frontier areas. Requirements are predominantly **Could / Should / Won't-now-but-roadmapped**; a small set are **Should** (near-term strategic).

**Boundary — do NOT duplicate:**
| Topic | Covered In |
|---|---|
| maxGraph 3D canvas engine, rendering, shape libraries | V03 |
| Base AI/voice/CV/LLM infrastructure, provider management, RAG, per-persona copilots | V06 |
| Human review workflow, review designer, review lifecycle | V15 |
| Predictive/prescriptive analytics, digital twin, what-if simulation, agentic autopilots, enterprise knowledge graph | V16 |
| Multimodal import, computer vision, mobile capture | V18 |

---

## Table of Contents

1. [GREEN — Sustainability & Green-IT Architecture](#1-green--sustainability--green-it-architecture)
2. [MNA — M&A Architecture Due-Diligence](#2-mna--ma-architecture-due-diligence)
3. [REG — Regulatory-Change Impact Analysis](#3-reg--regulatory-change-impact-analysis)
4. [VOICE — Voice-Driven & Conversational Architecture](#4-voice--voice-driven--conversational-architecture)
5. [AIRB — AI Pre-Review Board](#5-airb--ai-pre-review-board)
6. [IMM — Immersive Visualization (AR/VR/3D)](#6-imm--immersive-visualization-arvr3d)
7. [EVO — Evolutionary Architecture & Fitness Functions](#7-evo--evolutionary-architecture--fitness-functions)
8. [GEN — Generative Architecture Design (Design-Space Exploration)](#8-gen--generative-architecture-design-design-space-exploration)
9. [RES — Resilience & Chaos Simulation](#9-res--resilience--chaos-simulation)

---

## 1. GREEN — Sustainability & Green-IT Architecture

### Overview
Green-IT is a rising enterprise mandate (EU Corporate Sustainability Reporting Directive, SEC climate disclosure, EcoVadis, ISO 14001). ESIP becomes the system-of-record for architectural carbon and energy, linking design decisions to sustainability outcomes before a line of code is written.

### Requirements

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-FRO-GREEN-001 | Carbon Footprint Estimator | ESIP SHALL estimate the carbon footprint of an architecture or portfolio based on infrastructure topology (cloud regions, on-prem DC, edge), workload profiles, and published carbon-intensity coefficients (e.g., electricityMaps, AWS/Azure/GCP sustainability APIs). | AI/ML | Enterprise Architect, CTO | Should | CSRD / SEC climate disclosure mandate growing |
| REQ-FRO-GREEN-002 | Energy Consumption Modelling | ESIP SHALL model energy consumption (kWh/year) per architectural tier — compute, storage, network, cooling — using component-level PUE and TDP parameters, surfacing top consumers as a heat-map overlay on the architecture diagram. | Functional | Enterprise Architect, FinOps Architect | Should | Direct cost + sustainability KPI |
| REQ-FRO-GREEN-003 | Carbon-Aware Design Recommendations | ESIP SHALL generate carbon-aware refactoring suggestions: region migration to lower-carbon grids, right-sizing over-provisioned compute, serverless migration candidates, scheduling workloads to off-peak/renewable windows. | AI/ML | Enterprise Architect, Solution Architect | Could | Differentiator vs. pure FinOps tooling |
| REQ-FRO-GREEN-004 | Multi-Objective Optimization (Cost + Carbon + Performance) | ESIP SHALL offer a Pareto-front optimizer that simultaneously models trade-offs between monthly cost ($), carbon (kg CO₂e/year), and latency/performance SLA, presenting a set of non-dominated candidate designs for human selection. | AI/ML | Enterprise Architect, CTO, FinOps Architect | Could | Novel: no EA tool does this today |
| REQ-FRO-GREEN-005 | ESG / Sustainability Reporting | ESIP SHALL generate ESG-ready sustainability reports (PDF/DOCX/dashboard) showing architecture-level carbon baseline, reduction initiatives, progress vs. target, and methodology narrative suitable for board/regulator consumption. | Reporting | CTO, Chief Sustainability Officer | Should | Board-level KPI; feeds CSRD mandatory disclosures |
| REQ-FRO-GREEN-006 | Green Migration Suggestion Engine | ESIP SHALL score migration candidates (on-prem → cloud, region swap, hardware refresh) on a composite Green-Index combining carbon delta, cost delta, and migration complexity, and surface a prioritized green-migration backlog. | AI/ML | Portfolio Architect, FinOps Architect | Could | Closes loop between sustainability and migration planning |
| REQ-FRO-GREEN-007 | Carbon Intensity Data Connector | ESIP SHALL support integration with real-time carbon-intensity feeds (electricityMaps API, Watttime, cloud-provider sustainability dashboards) as pluggable data sources; air-gapped deployments SHALL support static annual-average coefficient tables (IEA grid factors). | Integration | Platform Admin | Should | Air-gap compatibility required |
| REQ-FRO-GREEN-008 | Carbon Budget & Threshold Governance | ESIP SHALL allow tenants to define architecture-level carbon budgets (kg CO₂e/year) and SHALL trigger policy-gate warnings or blocks on design approvals when a proposed architecture exceeds the budget threshold. | Governance | Enterprise Architect, Platform Admin | Could | Policy enforcement before deployment |
| REQ-FRO-GREEN-009 | Sustainability Score on Architecture Card | Every architecture artifact/component in ESIP SHALL display a sustainability score (A–E or 0–100) as a first-class metadata field, searchable and filterable in the catalog. | UX | All Architects | Could | Normalizes green as design criterion |

**Acceptance Criteria (GREEN):** Carbon estimates are within ±20% of cloud-provider sustainability dashboards for equivalent workloads in controlled tests; reports export cleanly to PDF with GHG Protocol methodology footnotes; air-gapped mode produces coherent estimates using bundled static coefficients.

---

## 2. MNA — M&A Architecture Due-Diligence

> **Reconciliation note (reuse / boundary — M&A analytics):** The M&A requirements here **REUSE existing analytics rather than re-implement them**. Specifically: estate comparison, overlap/redundancy detection, rationalisation costing, and synergy analysis (REQ-FRO-MNA-002/-003/-005/-009/-010) **reuse V16 portfolio intelligence** (portfolio analysis, impact/blast-radius, benchmarking); the technical-debt register for valuation (REQ-FRO-MNA-006) **reuses the V09 code-scan tech-debt / TIME model** output. **V20 adds ONLY the M&A-specific wrapper**: the dual-estate workspace with source-org labelling and isolation (REQ-FRO-MNA-001), the data-room confidentiality/need-to-know access controls and time-bounded auto-archive (REQ-FRO-MNA-008), and the PMI-roadmap framing (REQ-FRO-MNA-007). Implement these as a thin M&A layer that calls V16/V09 engines — do not duplicate the portfolio or tech-debt logic. Aligns with the FINALIZATION-REPORT "M&A analytics → reuse V16/V09 with M&A-specific wrapper" ruling.

### Overview
M&A deals fail 50–70% of the time partly due to IT integration surprises (Gartner). ESIP's M&A module makes architecture due-diligence a first-class workflow: ingest two estates, compare automatically, surface risk/synergy, and generate integration roadmaps.

### Requirements

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-FRO-MNA-001 | Dual-Estate Ingestion | ESIP SHALL allow creation of an M&A workspace that holds two (or more) separate organization estates; each estate ingests its architecture artifacts via the standard import pipeline (V18) with source-org labelling and strict data isolation until authorized. | Functional | Enterprise Architect, CTO | Should | Foundation for all M&A features |
| REQ-FRO-MNA-002 | Automated Estate Comparison | ESIP SHALL auto-compare two estates across capability maps, technology stacks, application portfolios, data domains, and integration patterns, producing a side-by-side comparison matrix with match/gap/overlap classification per element. | AI/ML | Enterprise Architect, Portfolio Architect | Should | Core M&A value prop |
| REQ-FRO-MNA-003 | Overlap & Redundancy Detection | ESIP SHALL identify overlapping capabilities, duplicate applications, and redundant data stores across the two estates, quantifying the rationalisation opportunity (estimated licence cost saving, headcount, hosting cost) per overlap cluster. | AI/ML | Enterprise Architect, FinOps Architect | Should | Directly feeds valuation models |
| REQ-FRO-MNA-004 | Integration Risk Scoring | ESIP SHALL score each detected integration point between the two estates on a risk scale (1–10) using heuristics: technology heterogeneity, data coupling depth, protocol mismatch, security zone boundary, and estimated integration effort (story points / days). | AI/ML | Solution Architect, Risk Officer | Should | Reduces post-merger surprises |
| REQ-FRO-MNA-005 | Synergy Analysis Report | ESIP SHALL generate a synergy analysis report identifying capability-level revenue synergies (shared platform, cross-sell enablement, shared data asset) and cost synergies (decommission candidates, shared services consolidation), with confidence bands. | Reporting | CTO, CFO Persona | Could | Board-level deliverable |
| REQ-FRO-MNA-006 | Technical Debt & Risk Surface for Valuation | ESIP SHALL generate a technical debt register for the target estate, scoring each debt item on severity, remediation cost estimate, and valuation impact, exportable as a structured annex to the due-diligence data room. | Reporting | Enterprise Architect, Risk Officer | Could | Bridges EA to M&A deal team |
| REQ-FRO-MNA-007 | Post-Merger Integration Roadmap Generator | ESIP SHALL auto-generate a phased post-merger integration (PMI) roadmap covering: wave 0 (day-1 coexistence), wave 1 (quick wins/decommissions), wave 2 (platform consolidation), wave 3 (target-state architecture). Each wave has dependency-ordered work items, effort, and risk rating. | Functional | Enterprise Architect, Portfolio Architect | Could | Accelerates deal value realisation |
| REQ-FRO-MNA-008 | Confidentiality & Data-Room Access Controls | ESIP SHALL enforce strict need-to-know access within an M&A workspace: estate data is siloed by role until a merge event is authorised; all M&A workspace activity SHALL be fully audited; workspaces SHALL be time-bounded with auto-archive. | Security | Platform Admin, CTO | Should | Legal / NDR / regulatory requirement |
| REQ-FRO-MNA-009 | Consolidation Scenario Modeller | ESIP SHALL allow architects to model consolidation scenarios (keep A, keep B, build new, hybrid) on the combined estate, scoring each scenario on cost, risk, time-to-realise, and strategic fit, with side-by-side comparison view. | Functional | Enterprise Architect | Could | Interactive what-if for M&A |
| REQ-FRO-MNA-010 | M&A Architecture Health Dashboard | ESIP SHALL provide a PMI health dashboard tracking integration milestones, decommission progress, open risk items, and synergy-realisation percentage post-close, updated from architecture artifact status. | Reporting | CTO, Portfolio Architect | Could | Sustains value tracking post-merger |

**Dependencies:** REQ-FRO-MNA-001 gates all others. Data isolation (MNA-008) must pass security review before any estate data is merged.

---

## 3. REG — Regulatory-Change Impact Analysis

> **Reconciliation note (C5 — air-gap exclusion / offline fallback):** The **Regulation Horizon Scanner (REQ-FRO-REG-001)** depends on live external regulatory feeds (EUR-Lex, FCA/PRA, NIST, sector bodies) and is therefore **SaaS-only / air-gap-EXCLUDED**. Its documented offline fallback is the **bundled, periodically-side-loaded offline regulation library (REQ-FRO-REG-010)**; the Regulation Change Feed / delta alerts (REQ-FRO-REG-008) degrade to the offline library's update cadence in air-gapped mode. This is the V20 entry behind the catalog-wide air-gap exclusion list in REQ-ENT-DEP-010 (V19) and §6 ruling 3. **Confirmed:** the other external-feed frontier features likewise carry offline fallbacks — GREEN carbon-intensity feeds → static annual coefficient tables (REQ-FRO-GREEN-007), and VOICE ASR → local engine (REQ-FRO-VOICE-010); IMM WebXR is a client/runtime capability, not an internet feed (no external-data dependency). See Assumption 6.

### Overview
Regulations such as GDPR, DORA, EU AI Act, NIS2, HIPAA, PCI-DSS, and emerging AI/data sovereignty laws create cascading architecture changes. ESIP's regulatory module maps incoming regulation to the live architecture, surfaces gaps, and plans remediation — turning a reactive scramble into a proactive design activity.

### Requirements

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-FRO-REG-001 | Regulation Horizon Scanner | ESIP SHALL subscribe to regulatory horizon feeds (EUR-Lex, FCA/PRA feeds, NIST, sector-specific bodies) and surface upcoming/enacted regulations relevant to the tenant's industry, jurisdiction, and technology profile on a Regulation Radar dashboard. | Integration | Enterprise Architect, Risk Officer | Could | Proactive rather than reactive compliance |
| REQ-FRO-REG-002 | Regulation → Architecture Capability Mapping | ESIP SHALL maintain a curated library of regulations (GDPR, DORA, EU AI Act, NIS2, HIPAA, PCI-DSS, SOX, CCPA, etc.) with machine-readable requirement clauses; for each clause, ESIP SHALL map affected architecture capability domains and component types. | Data | Enterprise Architect, Risk Officer | Should | Foundation for all downstream REG features |
| REQ-FRO-REG-003 | Regulation Impact Analysis | Given a regulation (or specific clause set), ESIP SHALL traverse the live architecture graph and identify every application, data asset, integration, and infrastructure component affected by that regulation's requirements, with impact classification (direct, transitional, indirect). | AI/ML | Enterprise Architect, Risk Officer | Should | Core use-case: "what does DORA change for us?" |
| REQ-FRO-REG-004 | Gap & Remediation Plan Generator | ESIP SHALL compare the identified regulation requirements against the current architecture's documented controls and capabilities, compute a gap list, and generate a prioritised remediation plan with work items, effort estimates, and RACI assignment. | Functional | Enterprise Architect, Risk Officer | Should | Converts analysis to actionable plan |
| REQ-FRO-REG-005 | "What-If This Regulation Passes" Simulation | ESIP SHALL allow architects to load a draft/proposed regulation text and simulate its impact on the architecture before it is enacted, producing a pre-impact report and an early-action recommendation set. | AI/ML | Enterprise Architect, Risk Officer | Could | Competitive differentiator — proactive compliance |
| REQ-FRO-REG-006 | Compliance Roadmap & Calendar | ESIP SHALL generate a compliance roadmap overlaying regulation enforcement dates, architecture change delivery milestones, and resource requirements on a Gantt/timeline view, exportable to project planning tools (Jira, Azure DevOps). | Reporting | Enterprise Architect, Portfolio Architect | Should | Bridges EA and compliance programme |
| REQ-FRO-REG-007 | Multi-Jurisdiction Overlay | ESIP SHALL support multi-jurisdiction profiles (e.g., EU + UK + US federal) and reconcile conflicting or overlapping requirements, flagging conflicts and identifying the strictest-applicable requirement per capability domain. | Functional | Enterprise Architect, Risk Officer | Could | Critical for multinationals |
| REQ-FRO-REG-008 | Regulation Change Feed & Delta Alert | ESIP SHALL monitor configured regulations for amendments and SHALL alert relevant personas when a regulatory change may alter a previously closed gap or invalidate a prior remediation, triggering a re-assessment workflow. | Integration | Enterprise Architect, Risk Officer | Could | Keeps compliance live, not point-in-time |
| REQ-FRO-REG-009 | Evidence Traceability for Audit | ESIP SHALL maintain traceable links from each regulation clause → architecture control → design decision → artefact version, producing an evidence package suitable for regulatory audit submission. | Governance | Risk Officer, Platform Admin | Should | Reduces audit preparation from weeks to hours |
| REQ-FRO-REG-010 | Air-Gap / Offline Regulation Library | For air-gapped deployments, ESIP SHALL ship a bundled, periodically-updated offline regulation library (snapshotted from authoritative sources), with a documented update procedure for controlled environments. | Integration | Platform Admin | Should | Air-gap requirement is first-class |

**Acceptance Criteria (REG):** GDPR Article 17 ("right to erasure") impact analysis on a reference architecture produces ≥90% precision (validated by human reviewer) in identifying data stores and pipelines requiring change; remediation plan exports to structured JSON consumable by Jira importer.

---

## 4. VOICE — Voice-Driven & Conversational Architecture

### Overview
Voice input removes friction for architects in meetings, on-site walkdowns, and accessibility scenarios. ESIP's voice layer translates spoken commands and dictation into architecture model mutations, query results, and requirement captures — building on the V06 AI/voice infrastructure (ASR, NLU, TTS) without re-specifying it here.

*Boundary: ASR engine, language models, TTS infrastructure → V06. This volume specifies the architecture-domain voice UX and command set.*

### Requirements

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-FRO-VOICE-001 | Voice Command Vocabulary for Architecture Modelling | ESIP SHALL support a spoken command vocabulary covering: create/delete/move/connect architecture elements; assign properties; open/close views; run analyses; trigger diagram layout — mapped to the same internal commands as keyboard/mouse interactions. | UX | All Architects | Could | Core voice modelling capability |
| REQ-FRO-VOICE-002 | Voice-to-Requirement Dictation | ESIP SHALL allow architects to dictate free-form requirements text; the system SHALL parse the dictated text, extract structured requirement fields (ID, title, description, category, persona, priority) using NLU, and present a draft requirement for confirmation before saving. | AI/ML | Business Architect, Requirements Analyst | Could | Captures requirements in meetings without friction |
| REQ-FRO-VOICE-003 | Voice Architecture Query | ESIP SHALL answer spoken natural-language questions about the architecture ("How many systems process personal data?", "What are the dependencies of the payment service?") and respond with a spoken summary AND highlight the relevant elements on the active diagram. | AI/ML | Enterprise Architect, CTO | Could | Hands-free review and exploration |
| REQ-FRO-VOICE-004 | Meeting / Workshop Transcription → Live Model Update | ESIP SHALL optionally ingest a live audio stream from a meeting (via browser/OS microphone) or a transcription feed, extract architecture decisions and requirement statements using event-detection NLU, and queue them as pending model updates for architect confirmation. | AI/ML | Enterprise Architect, Facilitator Persona | Won't-now (roadmap) | Highest-value but technically complex |
| REQ-FRO-VOICE-005 | Voice Review Annotation | During a review session (V15), ESIP SHALL allow reviewers to dictate review comments verbally; speech-to-text SHALL produce a draft comment attached to the selected element, with the reviewer confirming/editing before submission. | UX | Reviewer, Enterprise Architect | Could | Speeds review; accessibility benefit |
| REQ-FRO-VOICE-006 | Hands-Free Navigation Mode | ESIP SHALL support a hands-free navigation mode where all diagram navigation (pan, zoom, select, focus) and view switching are controllable by voice command, with no pointer/keyboard required — accessibility WCAG 2.1 AA aligned. | UX | All Architects (Accessibility) | Should | Accessibility mandate; motor-impaired users |
| REQ-FRO-VOICE-007 | Wake-Word / Push-to-Talk Toggle | ESIP SHALL support both a configurable wake-word ("Hey ESIP") and a push-to-talk keyboard shortcut for activating voice input, with a clear visual indicator of listening state; voice can be globally disabled by tenant admin. | UX | All Architects | Should | Privacy / enterprise control |
| REQ-FRO-VOICE-008 | Voice Command Confirmation & Undo | All destructive or structural voice commands (delete, rename, restructure) SHALL require spoken or clicked confirmation before execution; a voice-triggered undo ("undo that") SHALL revert the last voice-issued change. | UX | All Architects | Should | Prevents accidental voice mutations |
| REQ-FRO-VOICE-009 | Multilingual Voice Support | ESIP SHALL support voice input in at least English, French, German, and Spanish (matching the written UI language setting); voice commands SHALL be language-matched to the active UI locale. | UX | All Architects | Could | Global enterprise customer base |
| REQ-FRO-VOICE-010 | On-Premise / Local ASR for Air-Gap | In air-gapped deployments, ESIP SHALL support a locally-hosted ASR engine (e.g., Whisper.cpp or equivalent) so voice features function without external network calls; model files shall be bundled or delivered via the offline update channel. | Integration | Platform Admin | Could | Air-gap + privacy requirement |

---

## 5. AIRB — AI Pre-Review Board

### Overview
Before human reviewers engage, an autonomous panel of AI reviewer-personas scrutinises every architecture deliverable across multiple lenses. This dramatically reduces human reviewer load, surfaces issues earlier in the design lifecycle, and establishes a consistent quality floor. It feeds — but does not replace — the human Review Designer (V15).

*Boundary: Human review workflow, review lifecycle, human feedback incorporation → V15. This volume covers the autonomous AI review panel only.*

> **Reconciliation note (shared "AI Finding" record):** Every AIRB finding (REQ-FRO-AIRB-002/-004/-006/-008) SHALL be **written as a V15 "AI Finding" record** — the single shared finding model that V15 also consumes from its own inline AI review. AIRB runs *before* human review and **populates** these finding records, which then drive V15 human-review routing (REQ-FRO-AIRB-006) and appear in the review designer alongside V15-originated findings; AIRB does not maintain a parallel finding store. This implements the FINALIZATION-REPORT "AI-finding model" action (one shared record consumed by V15; both V15 inline-AI and V20 AIRB write to it). The autonomous-panel mechanics remain V20; the finding record schema and its review surface are V15.

### Requirements

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-FRO-AIRB-001 | AI Reviewer Persona Panel | ESIP SHALL provide a configurable panel of AI reviewer personas; default personas SHALL include: Security Reviewer, Performance Reviewer, Cost/FinOps Reviewer, Standards & Compliance Reviewer, Data Governance Reviewer, Resilience Reviewer, and Sustainability Reviewer. | AI/ML | Enterprise Architect, Platform Admin | Should | Core AIRB capability |
| REQ-FRO-AIRB-002 | Pre-Review Report Generation | ESIP SHALL automatically trigger the AI Pre-Review Board when an architect submits a deliverable for human review; the panel SHALL produce a Pre-Review Report listing severity-ranked findings (Critical / High / Medium / Low / Informational) per reviewer persona before any human reviewer is notified. | AI/ML | Enterprise Architect, Reviewer | Should | Reduces human load; raises quality floor |
| REQ-FRO-AIRB-003 | Lens-Based Configurable Analysis | Each AI reviewer persona SHALL operate from a configurable lens — a structured prompt + rule set + reference knowledge base — that platform admins and tenant architects can extend or override; lenses SHALL support tenant-specific standards and patterns. | Functional | Platform Admin, Enterprise Architect | Should | Tenant customisation is essential |
| REQ-FRO-AIRB-004 | Cross-Persona Finding Correlation | ESIP SHALL detect and surface cross-persona conflicts or compounding risks in findings (e.g., "Security finding X and Performance finding Y both point to the same component Z — combined risk is elevated") and present correlated findings as a unified insight. | AI/ML | Enterprise Architect, Reviewer | Could | Novel: most tools present siloed findings |
| REQ-FRO-AIRB-005 | Severity Calibration & Suppression | Architects SHALL be able to accept, suppress (with justification), or dispute individual AIRB findings; suppressed findings SHALL carry an audit trail; over time, the system SHALL learn from accept/suppress patterns to calibrate future finding severity (feedback loop, not raw LLM). | AI/ML | Enterprise Architect | Could | Reduces noise; builds trust |
| REQ-FRO-AIRB-006 | AIRB Finding → Human Review Routing | ESIP SHALL use AIRB findings to influence human review routing (V15): Critical/High findings SHALL auto-assign the corresponding domain expert reviewer; deliverables with zero Critical findings MAY follow an expedited review track. | Functional | Reviewer, Enterprise Architect | Should | Closes loop between AIRB and V15 |
| REQ-FRO-AIRB-007 | Architecture Pattern Library Cross-Check | Each AI reviewer SHALL cross-check the submitted architecture against the tenant's pattern library and known anti-patterns (V08 knowledge hub); deviations from approved patterns SHALL be flagged as findings with a reference to the canonical pattern. | AI/ML | Enterprise Architect | Should | Enforces house standards automatically |
| REQ-FRO-AIRB-008 | AIRB Finding Traceability to Requirements | ESIP SHALL link each AIRB finding to the specific requirement(s) or quality attribute(s) it relates to, providing a traceable rationale; this enables architects to understand the root cause rather than just the symptom. | Governance | Enterprise Architect | Could | Raises review quality and learning |
| REQ-FRO-AIRB-009 | Async & Batch Pre-Review Mode | ESIP SHALL support an async batch AIRB run across a portfolio (e.g., "run the Security Reviewer persona across all Solution Architecture deliverables modified in the last 30 days"), surfacing a portfolio-level pre-review heat-map. | Functional | Enterprise Architect, Portfolio Architect | Could | Portfolio-level quality governance |
| REQ-FRO-AIRB-010 | AIRB Metrics Dashboard | ESIP SHALL provide a dashboard showing AIRB effectiveness metrics: findings raised/accepted/suppressed per persona, average findings per deliverable, trends over time, human review time saved (estimated), and top recurring finding types. | Reporting | Platform Admin, CTO | Could | Demonstrates ROI of AIRB |
| REQ-FRO-AIRB-011 | Custom AI Reviewer Persona Creation | Platform admins SHALL be able to create custom AI reviewer personas with a defined name, icon, lens rules, reference documents, and severity thresholds, enabling domain-specific review (e.g., "OT/SCADA Security Reviewer", "FHIR Compliance Reviewer"). | Functional | Platform Admin | Could | Extensibility for vertical markets |

**Acceptance Criteria (AIRB):** On a reference architecture with 10 seeded issues (3 Critical, 4 High, 3 Medium), the default panel surfaces ≥80% of seeded issues at correct or higher severity; false-positive rate ≤25% in an independent blind evaluation.

---

## 6. IMM — Immersive Visualization (AR/VR/3D)

### Overview
Large enterprise landscapes (500+ applications, 10,000+ components) are cognitively overwhelming in 2D. Immersive 3D/AR/VR lets architects navigate spatial representations, improving comprehension, pattern recognition, and stakeholder communication.

*Boundary: The 3D rendering engine extension to maxGraph canvas is specified in V03. This volume covers the immersive UX layer, VR/AR export/runtime, and the architectural design affordances unique to immersive mode.*

### Requirements

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-FRO-IMM-001 | 3D Landscape View | ESIP SHALL render an architecture landscape as an interactive 3D scene (WebGL/WebGPU, browser-native) where capability domains are 3D spatial zones, applications are positioned within them by dependency proximity (force-directed 3D), and dependency flows are animated arcs. | UX | Enterprise Architect, CTO | Could | Comprehension at scale |
| REQ-FRO-IMM-002 | Spatial Navigation Controls | In 3D view, users SHALL navigate via orbit, pan, zoom, and fly-through camera; double-click SHALL drill into a component to surface its 2D detail view; breadcrumb trail SHALL maintain spatial context. | UX | All Architects | Could | Table stakes for usable 3D |
| REQ-FRO-IMM-003 | VR Headset Export / Runtime | ESIP SHALL export an architecture scene to a WebXR-compatible format so it can be loaded on VR headsets (Quest, Vision Pro class) via a WebXR browser session; six-degrees-of-freedom navigation and controller-based element selection SHALL be supported. | UX | Enterprise Architect, CTO | Won't-now (roadmap) | Aspirational; hardware penetration still low |
| REQ-FRO-IMM-004 | AR Overlay Mode | ESIP SHALL support an AR mode (browser-based WebXR AR or native iOS/Android ARKit/ARCore) that overlays architecture elements onto a physical whiteboard or meeting-room surface via device camera, enabling mixed-reality collaborative design sessions. | UX | Enterprise Architect, Facilitator | Won't-now (roadmap) | Emerging; roadmap for 2027+ |
| REQ-FRO-IMM-005 | Spatial Dependency Graph Navigation | ESIP SHALL render dependency graphs spatially in 3D, applying graph-layout algorithms (force-directed, hierarchical, radial) adapted for three dimensions, with automatic clustering of tightly-coupled component groups into spatial "galaxies". | AI/ML | Enterprise Architect | Could | Reveals structural patterns invisible in 2D |
| REQ-FRO-IMM-006 | Immersive Heat-Map Overlays | All analysis results (health scores, carbon scores, risk scores, compliance gaps) SHALL be applicable as colour/size/opacity overlays in 3D view — allowing architects to "fly through" the risk landscape and spot hotspots spatially. | UX | Enterprise Architect, Risk Officer | Could | High-impact at C-suite presentations |
| REQ-FRO-IMM-007 | Multi-User Shared 3D Session | ESIP SHALL support a shared 3D session where multiple users (identified by avatar presence indicators) navigate the same scene simultaneously, with real-time cursor/camera position sharing and synchronised element selection for collaborative walkthrough. | UX | Enterprise Architect, CTO, Stakeholder | Won't-now (roadmap) | Technically complex; high value for exec reviews |
| REQ-FRO-IMM-008 | 3D → 2D Clip-Plane Slice | From a 3D view, architects SHALL be able to define a clip-plane (horizontal or vertical cut) that produces a 2D cross-section of the architecture at that layer — automatically generating a standard 2D ArchiMate view for documentation. | Functional | Enterprise Architect | Could | Bridges immersive exploration and standard documentation |
| REQ-FRO-IMM-009 | Accessibility & Motion Safety Controls | ESIP SHALL provide motion-reduction settings for 3D/VR views (disable fly animations, reduce parallax) compliant with WCAG 2.2 "prefers-reduced-motion"; the 2D view SHALL always remain the primary accessible mode. | UX | All Architects | Should | Vestibular disorder / accessibility |

---

## 7. EVO — Evolutionary Architecture & Fitness Functions

### Overview
Evolutionary architecture (Building Evolutionary Architectures, Ford/Parsons/Kua) holds that architectural fitness must be mechanically verified, not just designed. ESIP implements fitness functions as first-class artifacts evaluated continuously against the living architecture model and in CI/CD pipelines.

*Boundary: Drift detection against documented standards → V16 (Architecture Conformance & Drift Detection). Fitness functions here are architect-defined, domain-specific, and continuously tracked — distinct from standards conformance.*

### Requirements

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-FRO-EVO-001 | Fitness Function Definition | ESIP SHALL allow architects to define fitness functions as named, versioned artifacts with: a natural-language description, a dimension (coupling, deployability, performance, security, scalability, data integrity, etc.), a measurement specification (query expression, metric threshold, or rule), and a target threshold with tolerance band. | Functional | Enterprise Architect | Should | Foundation for evolutionary arch in ESIP |
| REQ-FRO-EVO-002 | Architecture-Model Evaluation | ESIP SHALL evaluate all defined fitness functions against the current architecture model on demand or on schedule, computing a fitness score per function and an overall architectural fitness index, surfacing pass/warn/fail status per function. | Functional | Enterprise Architect | Should | Core EVO evaluation loop |
| REQ-FRO-EVO-003 | Fitness Function Evolution Timeline | ESIP SHALL maintain a time-series history of fitness function scores, enabling architects to view the evolution of architectural fitness over time (weeks/months/quarters) as trend charts — detecting degradation or improvement trajectories. | Data | Enterprise Architect | Could | "How is our architecture aging?" |
| REQ-FRO-EVO-004 | CI/CD Fitness Gate | ESIP SHALL expose fitness function evaluation via a REST/webhook API so that CI/CD pipelines (GitHub Actions, Azure DevOps, GitLab CI) can invoke ESIP fitness checks as a pipeline gate; the API SHALL return a structured JSON result with pass/fail and a machine-readable finding list. | Integration | Enterprise Architect, DevOps Engineer Persona | Could | Embeds architectural governance in delivery pipelines |
| REQ-FRO-EVO-005 | Fitness Function Library | ESIP SHALL ship a curated library of pre-built fitness functions covering: cyclic dependency detection, service coupling index, data sovereignty compliance, blast-radius score, API versioning hygiene, documentation completeness — each customisable. | Functional | Enterprise Architect | Should | Accelerates adoption; avoids blank-slate problem |
| REQ-FRO-EVO-006 | Guided Fitness Function Authoring | ESIP SHALL provide a guided authoring wizard for fitness functions, using NL-to-rule AI assistance to translate a natural-language fitness goal ("No service should depend on more than 3 other services") into a formal rule expression, with preview evaluation on the current model. | AI/ML | Enterprise Architect | Could | Lowers barrier to adoption |
| REQ-FRO-EVO-007 | Fitness vs. Architectural Decision Correlation | ESIP SHALL correlate fitness function score changes with Architecture Decision Records (ADRs) in the knowledge base, surfacing which decisions improved or degraded fitness over time. | AI/ML | Enterprise Architect | Could | Closes feedback loop between decisions and outcomes |
| REQ-FRO-EVO-008 | Fitness Budget (Acceptable Degradation Range) | ESIP SHALL allow architects to define a "fitness budget" — a maximum acceptable degradation delta per function per quarter — and SHALL alert when accumulated degradation approaches or exceeds the budget, analogous to a technical-debt budget. | Governance | Enterprise Architect | Could | Manages architectural entropy proactively |
| REQ-FRO-EVO-009 | Multi-Dimensional Radar View | ESIP SHALL render architectural fitness as a multi-axis radar/spider chart covering the principal fitness dimensions (coupling, deployability, security, scalability, observability, sustainability), enabling architects to see overall architectural health at a glance. | UX | Enterprise Architect, CTO | Should | Executive-readable architectural health summary |

---

## 8. GEN — Generative Architecture Design (Design-Space Exploration)

### Overview
Rather than a single generated design (V06 AI Architecture Generation), this area automates the exploration of a *design space*: generating multiple candidate architectures from a requirement/constraint set, scoring them systematically, and supporting iterative refinement — analogous to parametric design in engineering.

*Boundary: V06 covers single-candidate AI architecture generation and per-persona copilots. This volume covers multi-candidate generative design-space exploration, scoring, and Pareto comparison.*

### Requirements

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-FRO-GEN-001 | Multi-Candidate Architecture Generation | ESIP SHALL generate N (configurable, default 3–5) structurally distinct candidate architectures from a seed input of: functional requirements, quality attributes (NFRs), constraints (budget, tech stack, geography, regulatory), and strategic drivers — each candidate representing a different design strategy (e.g., monolith-first, microservices, serverless-native, event-driven). | AI/ML | Enterprise Architect, Solution Architect | Could | Core GEN capability; design-space exploration |
| REQ-FRO-GEN-002 | Candidate Architecture Scoring | Each generated candidate SHALL be automatically scored on a configurable attribute set (cost, performance, security, scalability, resilience, carbon, maintainability, time-to-market) producing a normalised score matrix; scores SHALL reference the scoring rationale. | AI/ML | Enterprise Architect, Solution Architect | Could | Makes comparison objective |
| REQ-FRO-GEN-003 | Pareto Comparison View | ESIP SHALL render candidates on a Pareto comparison UI: a scatter plot (or parallel coordinates view) showing the trade-off frontier across two or more selected attributes, enabling architects to identify non-dominated candidates and understand trade-offs visually. | UX | Enterprise Architect, CTO | Could | Reduces cognitive load of multi-attribute comparison |
| REQ-FRO-GEN-004 | Constraint Propagation Engine | ESIP SHALL enforce hard constraints (e.g., "must be GDPR-compliant", "max $50k/month cloud spend", "no AWS services") during generation, pruning candidates that violate hard constraints before scoring, and flagging soft-constraint violations. | Functional | Enterprise Architect | Could | Prevents generating infeasible candidates |
| REQ-FRO-GEN-005 | Iterative Refinement Cycle | Architects SHALL be able to select a candidate and apply refinement instructions ("increase resilience", "reduce cloud dependency", "simplify the integration layer") that trigger a targeted regeneration of that candidate's affected sub-domains, producing an updated candidate within the same session. | AI/ML | Enterprise Architect, Solution Architect | Could | Brings human judgment into design-space exploration |
| REQ-FRO-GEN-006 | Candidate → Full Architecture Promotion | An architect SHALL be able to promote a selected candidate to a full ESIP architecture artifact, triggering a structured elaboration workflow that populates the standard architecture metadata, opens the diagram for editing, and creates linked ADRs explaining the design choices. | Functional | Enterprise Architect | Could | Connects GEN to the ESIP artifact lifecycle |
| REQ-FRO-GEN-007 | Design Pattern Injection | During generation, ESIP SHALL optionally inject candidate-appropriate design patterns from the tenant pattern library (V08) — e.g., CQRS for high-read patterns, saga for distributed transactions — with pattern provenance tracked in the architecture rationale. | AI/ML | Enterprise Architect | Could | Grounds generated designs in approved patterns |
| REQ-FRO-GEN-008 | Generation Run History & Reproducibility | ESIP SHALL store each generation run (seed inputs, constraint set, model version, generated candidates, scores) as an immutable versioned record so that runs can be reproduced, compared across time, and audited. | Data | Enterprise Architect | Should | Reproducibility and auditability |
| REQ-FRO-GEN-009 | Collaborative Candidate Voting | In a team context, multiple architects SHALL be able to score/vote on generated candidates (thumbs up/down, weighted preference), with ESIP aggregating votes alongside AI scores to surface a team-preferred candidate with a transparent justification. | UX | Enterprise Architect, Solution Architect | Could | Sociotechnical: gets human wisdom into AI-generated options |
| REQ-FRO-GEN-010 | Domain-Specific Generation Templates | Platform admins SHALL be able to define domain-specific generation templates (e.g., "retail e-commerce", "healthcare data platform", "financial trading system") that pre-seed the constraint/quality-attribute defaults and pattern libraries for common architecture scenarios. | Functional | Platform Admin | Could | Accelerates vertical-market adoption |

---

## 9. RES — Resilience & Chaos Simulation

### Overview
Architecture resilience must be designed, not tested only in production. ESIP's simulation layer allows architects to inject failures, disasters, and degradation scenarios into the *architecture model* before build, predict blast radius, validate HA/DR designs, and generate a structured resilience score — bringing chaos engineering principles to the design phase.

*Boundary: V16 covers blast-radius impact analysis as a predictive/prescriptive analytics feature from the perspective of change propagation. This volume covers active failure simulation, scenario game-day planning, and resilience scoring from an architecture-model basis.*

### Requirements

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-FRO-RES-001 | Failure Scenario Library | ESIP SHALL provide a library of pre-built failure scenarios covering: single component failure, AZ failure, region failure, network partition, cascading failure cascade, database primary failure, third-party SaaS outage, DDoS impact, ransomware lateral movement, and power/cooling failure. | Functional | Enterprise Architect, Solution Architect | Should | Foundation for simulation |
| REQ-FRO-RES-002 | Architectural Failure Simulation Engine | ESIP SHALL simulate a selected failure scenario on the architecture model by: (1) marking affected components as failed/degraded; (2) propagating the failure along dependency paths using configurable propagation rules (hard-dependency: cascades; soft-dependency: degrades); (3) computing the set of affected services, data flows, and user-facing capabilities. | AI/ML | Enterprise Architect, Solution Architect | Should | Core RES simulation engine |
| REQ-FRO-RES-003 | Blast Radius Visualisation | ESIP SHALL render the simulated failure's blast radius as an animated overlay on the architecture diagram: failed components in red, degraded in amber, unaffected in green — with a summary panel listing affected capabilities and estimated user/business impact. | UX | Enterprise Architect, Risk Officer | Should | Visual communication of failure impact |
| REQ-FRO-RES-004 | Resilience Pattern Validation | ESIP SHALL check the architecture against a set of resilience patterns (circuit breaker, bulkhead, retry with back-off, active-active/active-passive HA, multi-region DR, data replication strategy) and report which patterns are present, absent, or incompletely implemented per component. | Functional | Enterprise Architect, Solution Architect | Should | Validates design intent against resilience standards |
| REQ-FRO-RES-005 | Recovery Path Analysis | For each simulated failure, ESIP SHALL identify the expected recovery path (manual failover steps, automated recovery triggers, RTO/RPO implied by the design) and SHALL flag gaps where the recovery path is undefined, exceeds SLA targets, or has unvalidated assumptions. | AI/ML | Enterprise Architect, Risk Officer | Should | RTO/RPO design validation before build |
| REQ-FRO-RES-006 | Resilience Score | ESIP SHALL compute a Resilience Score (0–100) for an architecture based on: coverage of resilience patterns, single-points-of-failure count, worst-case blast radius severity, redundancy levels by tier, and DR coverage — with sub-scores per resilience dimension. | Functional | Enterprise Architect, CTO | Could | Single KPI for executive communication |
| REQ-FRO-RES-007 | Game-Day Scenario Planner | ESIP SHALL support a "Game-Day" planning workflow where an architect designs a multi-step failure sequence (e.g., AZ-1 goes down, then DB failover is delayed, then CDN is also degraded), runs the compound simulation, and exports a game-day runbook with expected outcomes and validation checkpoints. | Functional | Enterprise Architect, SRE Persona | Could | Bridges design-time and operational chaos engineering |
| REQ-FRO-RES-008 | DR Design Validator | ESIP SHALL validate a documented DR design (RPO, RTO, replication topology, failover trigger) against the architecture model, flagging components with: no DR coverage, RPO/RTO that cannot be achieved given the documented replication strategy, and manual steps with no automation fallback. | Functional | Enterprise Architect, Solution Architect | Could | De-risks DR designs before incident |
| REQ-FRO-RES-009 | Custom Failure Scenario Authoring | Architects SHALL be able to define custom failure scenarios beyond the library (e.g., "vendor X's API returns 503 for >60 seconds"), specifying: trigger, affected component set, propagation mode, and expected business impact descriptor — storable and shareable within the tenant. | Functional | Enterprise Architect | Could | Enables domain-specific resilience testing |
| REQ-FRO-RES-010 | Resilience Benchmark (Industry Comparison) | ESIP SHALL optionally benchmark an architecture's Resilience Score against anonymised industry-sector averages (sourced from aggregated ESIP telemetry, opt-in only), giving architects a percentile position and peer comparison. | Data | CTO, Enterprise Architect | Won't-now (roadmap) | Requires critical telemetry mass; roadmap |
| REQ-FRO-RES-011 | Resilience Findings → AIRB Integration | Resilience simulation findings (single points of failure, missing patterns) SHALL be made available as inputs to the AIRB Resilience Reviewer persona (REQ-FRO-AIRB-001), enabling the pre-review board to include simulation-grounded resilience findings rather than purely heuristic ones. | Integration | Enterprise Architect | Could | Synergy: RES enriches AIRB quality |

---

## Summary Statistics

| Area | Prefix | Req Count | Priority Split |
|---|---|---|---|
| GREEN — Sustainability | REQ-FRO-GREEN | 9 | 4× Should, 5× Could |
| MNA — M&A Due-Diligence | REQ-FRO-MNA | 10 | 4× Should, 5× Could, 1× Should (MNA-008 security) |
| REG — Regulatory Impact | REQ-FRO-REG | 10 | 5× Should, 4× Could, 1× Should (REG-010 air-gap) |
| VOICE — Conversational Arch | REQ-FRO-VOICE | 10 | 3× Should, 5× Could, 2× Won't-now |
| AIRB — AI Pre-Review Board | REQ-FRO-AIRB | 11 | 4× Should, 6× Could, 1× Could (AIRB-011) |
| IMM — Immersive Viz | REQ-FRO-IMM | 9 | 1× Should, 5× Could, 3× Won't-now |
| EVO — Evolutionary Arch | REQ-FRO-EVO | 9 | 3× Should, 6× Could |
| GEN — Generative Design | REQ-FRO-GEN | 10 | 1× Should, 9× Could |
| RES — Resilience Simulation | REQ-FRO-RES | 11 | 5× Should, 5× Could, 1× Won't-now |
| **TOTAL** | | **89** | **30× Should, 50× Could, 9× Won't-now** |

---

## Cross-Volume References

| This Volume | References |
|---|---|
| 3D canvas engine & rendering pipeline | V03 (ArchiMate Diagramming, maxGraph) |
| ASR, LLM, TTS, provider management, RAG | V06 (AI Capabilities) |
| Human review workflow, routing, lifecycle | V15 (Review & Collaboration Designer) |
| Predictive analytics, digital twin, what-if, drift detection, blast-radius (change-propagation), agentic autopilots | V16 (Differentiating Intelligence & Analytics) |
| Multimodal import, computer vision | V18 (Interoperability & Import) |

---

## Assumptions & Boundary Notes

1. **Voice (VOICE)**: The ASR/NLU/TTS *infrastructure* is not specified here — it relies on the provider-agnostic model gateway in V06. VOICE requirements specify only the architecture-domain command vocabulary and UX affordances.
2. **AIRB vs. V15**: AIRB is *pre*-human-review automation only. The moment a human reviewer receives and acts on a review, that workflow is V15. AIRB findings *feed* V15 routing — they do not replace it.
3. **IMM 3D canvas extension**: The WebGL/WebGPU rendering extension to maxGraph is a V03 dependency. IMM builds *on top of* that canvas capability and does not re-specify the rendering engine.
4. **GEN vs. V16 what-if**: V16's what-if simulation explores impact variations of an *existing* architecture. GEN generates architectures *from scratch* from requirements — fundamentally different (design-space generation vs. perturbation analysis).
5. **RES vs. V16 blast-radius**: V16 blast-radius is about change-propagation from a proposed architecture change. RES blast-radius is failure-propagation under active failure simulation — a different computational model.
6. **Air-gap**: All areas with external data feeds (GREEN carbon intensity, REG regulation feeds, IMM WebXR) include explicit air-gap/offline fallback requirements.
7. **LLM dependency**: GEN, AIRB, REG, and VOICE-004 are among the most LLM-intensive features. In air-gapped / local-LLM deployments, capability quality will depend on the locally-deployed model's capability tier.
