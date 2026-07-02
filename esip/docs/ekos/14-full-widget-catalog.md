# EKOS Volume 14 — Full Widget Catalog (All Industries)

> **Status:** ✅ Done · **Requirement count:** 110 · **Generated:** 2026-06-30 (Opus)
> **Sources mined:** `research_across_llms.txt` (full phase-by-phase widget catalog,
> industry-deep widgets, missing-95% artifacts, advanced visualization/simulation
> widgets, architecture-domain widgets TOGAF/Zachman/cloud/packet-animation), `blueprint
> §2` (Living Widget Ecosystem) + `§17` (Executive Mission Control), `ConfusionClarity.txt`
> (final-spec §3 catalog), ESIP existing widget set. Honors
> [00-MASTER-INDEX](00-MASTER-INDEX.md) §2.

---

## 1. Scope & Boundary

**In scope:** the **concrete, seeded widget library** — every widget named across the
research, organized by SDLC phase, industry vertical, artifact category, and advanced
visualization/simulation. Each entry states what it **replaces** and its **visual
superpower**.

**Explicitly deferred (cross-referenced, not duplicated):**
- The widget **engine, anatomy, rendering, Living Canvas, export, Custom Widget OS,
  Process Capture AI** → **Vol 02** (each catalog widget is an instance of the engine)
- The graph/ontology/projection-query → **Vol 01**
- **AI Crux/Explain/Example/analyst behavior** on each widget → **Vol 03** (this volume
  notes which analysts attach; it does not re-specify them)
- **ABAC masking, Privacy Dashboard *data*, Regulatory Evidence *data*** → **Vol 04**;
  **Governance Workbench *engine*, compliance gates, Green policy** → **Vol 05**
- **Matrix/Trade-off/NFR widget *mechanics* already shipped in ESIP** → noted as early
  impls (§22); their EKOS-native re-expression follows Vol 02 migration

---

## 2. Catalog Principle

- **EKOS-14-001 (Must, MVP)** — Every widget SHALL be a **graph projection** (Vol 01) +
  visual recipe (Vol 02) that **replaces a text-heavy artifact**, is a **first-class,
  ID-bearing, traceable object**, and is **AI-explainable** (Vol 03). *Source:* research
  (widget intro — "each widget is a first-class object ... traced from requirement →
  code").

- **EKOS-14-002 (Must, MVP)** — This catalog is the **seeded baseline**, not a closed
  set; the Custom Widget OS (Vol 02 §6–8) lets users build the rest. *Source:*
  research(custom widget framework — "we'll never cover every niche"); Vol 02.

- **EKOS-14-003 (Must, MVP)** — Every catalog widget SHALL support **AI "Explain"** and a
  **role-adaptive example** (Vol 03 §10) and SHALL expose its declared `ai_hooks`.
  *Source:* research(AI Example Engine); Vol 02 EKOS-02-034/035, Vol 03.

---

## 3. Requirements-Phase Widgets

*(Replace: BRDs, FRDs, flat user-story lists, use-case docs.)*

- **EKOS-14-004 (Must, MVP)** — **Story Map Canvas** — 2D grid of user activities
  (backbone) × releases (slices); cards color-coded by status; drag to reprioritize.
- **EKOS-14-005 (Must, MVP)** — **Requirement Tree / Mind Map** — collapsible hierarchy;
  nodes tagged functional/non-functional; links to story map & architecture.
- **EKOS-14-006 (Must, MVP)** — **Use-Case Flow Widget** — per-actor swimlane flowchart;
  steps expand to pre/post conditions; alternate flows branch visually.
- **EKOS-14-007 (Must, MVP)** — **Acceptance Criteria Canvas** — visual Given-When-Then
  scenario cards as small state-flows; criteria link to test cases.
- **EKOS-14-008 (Must, MVP)** — **Business Process Model (BPMN-lite)** — pools/lanes/
  tasks/events; tasks link to requirements/rules/roles. *(ESIP ships a BPMN widget.)*
- **EKOS-14-009 (Should, P2)** — **Impact Mapping Widget** — Goal → Actors → Impacts →
  Deliverables mind-map; deliverables spawn feature/epic canvases.
- **EKOS-14-010 (Should, P2)** — **Persona / Stakeholder Card** — visual card (photo,
  goals, pain points, cared-about features); hover highlights related requirements.

*Source (4–10):* research(Requirements Phase table); blueprint §2; ConfusionClarity §3.1.

---

## 4. Architecture-Phase Widgets

*(Replace: ADRs, design docs, interface specs, data-model docs.)*

- **EKOS-14-011 (Must, MVP)** — **C4 Model Drill-down** — Context → Container → Component
  → Code; each box live-linked to requirements/stories/repos. *(ESIP flow/maxGraph is a
  precursor.)*
- **EKOS-14-012 (Must, MVP)** — **Sequence Diagram Widget** — lifelines/messages; arrows
  expand payload schema; linked to a story.
- **EKOS-14-013 (Should, P2)** — **Entity-Relationship Explorer** — tables/columns/
  cardinality; click table → where used; can generate migration scripts.
- **EKOS-14-014 (Should, P2)** — **API Resource Graph** — resources as nodes, hypermedia
  edges; methods + request/response trees; traceable to the requiring story.
- **EKOS-14-015 (Should, P2)** — **Event Storming Canvas** — DDD stickies (events/
  commands/aggregates) on a timeline; link to code & process.
- **EKOS-14-016 (Should, P2)** — **Deployment / Infrastructure Map** — topology (regions/
  VPCs/clusters/pods) with health; trace requirement → running pod.
- **EKOS-14-017 (Should, P2)** — **Dependency / Risk Matrix** — heatmap of components ×
  NFRs; color = compliance status; click → evidence/decisions.

*Source (11–17):* research(Architecture Phase table); blueprint §2; ConfusionClarity §3.2.

---

## 5. Development-Phase Widgets

*(Replace: code comments, API docs, test plans, traceability matrices, READMEs.)*

- **EKOS-14-018 (Should, P2)** — **Codebase Sunburst / Treemap** — zoomable; sized by
  LOC/complexity; colored by coverage/churn; drill to class + linked requirements.
- **EKOS-14-019 (Should, P2)** — **Live Class / Component Diagram** — auto-generated from
  code; each class shows which requirement(s) it implements.
- **EKOS-14-020 (Should, P2)** — **API Sandbox Widget** — interactive endpoint card,
  "Try It", linked to code; schema auto-updates from code.
- **EKOS-14-021 (Should, P2)** — **Test Coverage Landscape** — treemap/heatmap over code;
  test suites linked to acceptance criteria; pass/fail visible.
- **EKOS-14-022 (Should, P2)** — **Branch & Merge Timeline** — Git-flow visual; commits/
  PRs linked to story & architecture component. *(Uses CQRS event store, Vol 01 §2.)*
- **EKOS-14-023 (Must, MVP)** — **Traceability Graph (master widget)** — network of all
  artifacts with `traces_to` edges; filter by type; surface coverage gaps; navigate
  requirement → running code → test status. *The single pane that replaces alignment
  meetings.*
- **EKOS-14-024 (Should, P2)** — **Runbook / Operation Widget** — visual procedure with
  embedded monitoring charts; highlights steps on component alert.

*Source (18–24):* research(Development Phase table); blueprint §2; ConfusionClarity §3.3.

---

## 6. Industry-Deep — Product Companies

- **EKOS-14-025 (Should, P2)** — **Experiment & Feature-Flag Board** — active A/B tests,
  linked features, rollout %; idea → flag → code → outcome traceability.
- **EKOS-14-026 (Should, P2)** — **User Feedback & Sentiment Graph** — feedback clusters
  (Intercom/surveys) auto-linked to story map; hotspots on painful journeys.
- **EKOS-14-027 (Should, P2)** — **Roadmap Timeline vs. Actual Delivery** — features on a
  timeline overlaid with live deployment/adoption; promised vs reality.

*Source (25–27):* research(Product Companies); ConfusionClarity §3.4.

---

## 7. Industry-Deep — Services / Consulting

- **EKOS-14-028 (Should, P2)** — **SOW & Deliverable Compliance / Health Dashboard** —
  hierarchy of milestones/deliverables/acceptance; compliance bar fills on "done";
  client one-click Acceptance View with digital signature.
- **EKOS-14-029 (Should, P2)** — **RFP Response Mapper** — AI extracts client requirement
  nodes, maps to reusable assets; proposal becomes visual widgets; cuts presales cycle.
- **EKOS-14-030 (Should, P2)** — **Timesheet & Effort Traceability Matrix** — heatmap of
  billable hours over architecture/story map; spot gold-plating.
- **EKOS-14-031 (Should, P2)** — **Client Dependency Tracker** — flowchart of client-side
  dependencies blocking stories; red chains for breached SLAs; auto-draft reminder.
- **EKOS-14-032 (Should, P2)** — **Contract Change Request (CR) Visualizer** — shows
  impact of scope change (affected requirements/components/effort), traced to the SOW
  paragraph; client approves inline.
- **EKOS-14-033 (Should, P2)** — **Staffing & Skills Matrix** — team × skills grid +
  availability heatmap; drag person → story to assign; auto-updates capacity.

*Source (28–33):* research(Services/Consulting); ConfusionClarity §3.4/§2.5.

---

## 8. Industry-Deep — Security / DevSecOps

- **EKOS-14-034 (Should, P2)** — **Threat Model Canvas (STRIDE / LINDDUN)** — each
  architecture component carries a threat list; data-flow edges annotated with threats;
  each threat traces to a security requirement + code mitigation; auditor "Verify".
- **EKOS-14-035 (Should, P2)** — **Compliance Control Mapping (SOC2 / ISO 27001 /
  FedRAMP)** — grid mapping framework controls → components/tests/runbooks; flags impacted
  control on a firewall-rule change; continuous compliance. *(Data ties to Vol 04/05.)*
- **EKOS-14-036 (Should, P2)** — **Vulnerability-to-Source Traceability** — CVE published
  → scans SBOM → highlights affected components → trace back to features; instant impact.

*Source (34–36):* research(Security/DevSecOps); ConfusionClarity §3.4.

---

## 9. Industry-Deep — Government / Defense

- **EKOS-14-037 (Should, P2)** — **Acquisition & Procurement Milestone Widget** —
  Gantt of govt acquisition phases (RFI/RFP/Award/CDRL); milestones tied to SDLC
  requirements/tests.
- **EKOS-14-038 (Should, P2)** — **Section 508 / Accessibility Overlay** — paints UI
  stories/components green/red per WCAG check; links to exact criteria + violation
  screenshot.
- **EKOS-14-039 (Should, P2)** — **Change Control Board (CCB) Workflow Visualizer** —
  flowchart routing changes through approval gates (security/ops/business) with visual
  status; full audit trail.
- **EKOS-14-040 (Should, P2)** — **Classified Data Flow & Boundary Marker** — mark
  Unclassified/Secret/Top-Secret boundaries; verify flows don't cross without cross-
  domain guards; red flash on mismatch. *(Ties to Vol 04 clearance.)*

*Source (37–40):* research(Government/Defense); ConfusionClarity §3.4.

---

## 10. Industry-Deep — Life Sciences

- **EKOS-14-041 (Should, P2)** — **GxP Validation & Audit Trail Widget** — every
  requirement/test/release stamped with e-signature (Vol 04 §14) + linked to regulatory
  submission; FDA-navigable predicate-rule → code → test. *(21 CFR Part 11.)*

*Source:* research(Life Sciences); ConfusionClarity §3.4.

---

## 11. Industry-Deep — Embedded / Automotive / Aerospace

- **EKOS-14-042 (Should, P2)** — **BOM Traceability Widget** — tree of hardware components/
  firmware/software features; click a sensor → dependent requirements + reading code.
- **EKOS-14-043 (Should, P2)** — **Safety Case & Hazard Analysis (ISO 26262 / DO-178C)** —
  fault-tree of hazards → safety requirements → safety mechanisms → validating test; AI
  drafts a safety-case summary for certification.
- **EKOS-14-044 (Could, P3)** — **Vehicle Bus Signal Map** — CAN/LIN signals × ECUs matrix;
  signal linked to producing/consuming feature; trace warning light → code.

*Source (42–44):* research(Hardware/Embedded); ConfusionClarity §2.5.

---

## 12. Project & Governance Artifacts

- **EKOS-14-045 (Should, P2)** — **Project Charter Canvas** — tiles (Vision/Scope/Out-of-
  scope/Stakeholders/High-level Risks/Success Metrics); AI auto-drafts from a transcript.
- **EKOS-14-046 (Should, P2)** — **RAID Log Dynamic Radar** — 4-quadrant (Risks/Assumptions/
  Issues/Decisions); risks on probability-impact matrix; cards linked to requirements/
  commits/components; AI suggests mitigations.
- **EKOS-14-047 (Should, P2)** — **Stakeholder Communication Plan** — timeline of each
  stakeholder's touchpoints; linked to milestones/stories.
- **EKOS-14-048 (Should, P2)** — **RACI Matrix Visualizer** — tasks × roles colored chips;
  click → story/component; AI flags missing accountability. *(ESIP ships a RACI matrix.)*
- **EKOS-14-049 (Should, P2)** — **Meeting Outcomes & Decision Log** — tree of meetings →
  decisions → impacted artifacts; AI summarizes notes into decision records.

*Source (45–49):* research(Project & Governance Artifacts); ConfusionClarity §2.5/§3.5.

---

## 13. QA & Testing

- **EKOS-14-050 (Should, P2)** — **Test Strategy Navigator** — mind-map of test levels
  (unit/integration/E2E) per feature; leaves = suites with pass/fail; perf test → NFR.
- **EKOS-14-051 (Should, P2)** — **Defect Clustering Map** — heatmap over codebase/feature
  map of bug concentration; AI correlates with recent changes/complexity → refactor
  stories.
- **EKOS-14-052 (Should, P2)** — **Accessibility Audit Widget** — UI replica with hotspots
  for WCAG violations, each linked to the component code + responsible story.

*Source (50–52):* research(QA & Testing); ConfusionClarity §2.5/§3.5.

---

## 14. Business & Product Management

- **EKOS-14-053 (Should, P2)** — **Pricing & Packaging Modeler** — plan tiers/features/
  limits; each feature toggle links to feature-flag + requirement; shows revenue impact.
- **EKOS-14-054 (Could, P3)** — **Competitive Intelligence Battlecard** — your features
  (live from story map) vs competitors; AI scrapes public data; win/loss gaps highlighted.
- **EKOS-14-055 (Should, P2)** — **Customer Onboarding Funnel Widget** — Sankey of first-
  week drop-off overlaid with the stories meant to address them.

*Source (53–55):* research(Business & Product Management); ConfusionClarity §3.5.

---

## 15. IT Operations & DevOps

- **EKOS-14-056 (Should, P2)** — **Runbook Automation Visualizer** — flowchart of automated
  remediation with live status; steps linked to the architecture component fixed.
- **EKOS-14-057 (Should, P2)** — **Cost-to-Feature Map** — architecture diagram where each
  cloud resource is sized by cost, colored by owning feature team; per-feature monthly
  cost, traced requirement → cloud resource.

*Source (56–57):* research(IT Operations & DevOps); ConfusionClarity §3.5.

---

## 16. Training & Change Management

- **EKOS-14-058 (Should, P2)** — **Learning Path Widget** — story map of training modules
  linked to product features; tracks which features a new joiner understands.
- **EKOS-14-059 (Should, P2)** — **Release Communication Canvas** — auto-generated change
  log from "done" customer-facing stories; AI writes plain-language "what's new".

*Source (58–59):* research(Training & Change Management); ConfusionClarity §3.5.

---

## 17. Privacy, Audit & Compliance Widgets

- **EKOS-14-060 (Should, P2)** — **Privacy & Data Flow Widget (GDPR/CCPA)** — data flow
  over architecture; each store tagged PII type/retention/lawful-basis; AI flags a flow
  lacking a processing agreement. *(Data: Vol 04.)*
- **EKOS-14-061 (Should, P2)** — **Audit Evidence Map** — tree from a control (e.g. SOC2
  CC6.1) → policies → technical controls → code configs → test results; live evidence.
- **EKOS-14-062 (Should, P2)** — **Incident Postmortem Timeline** — timeline + fishbone;
  events from monitoring; causes link to the code change/requirement/missed-story; AI
  drafts postmortem + preventive stories.
- **EKOS-14-063 (Should, P2)** — **Privacy Dashboard** — live map of all PII (purpose/
  consent/retention) across the graph. *(Vol 04 §8.)*
- **EKOS-14-064 (Should, P2)** — **Regulatory Evidence Widget** — pre-compiled evidence
  pack for ISO 27001/GDPR audits citing graph nodes. *(Vol 04 §18 / Vol 05 §6.)*

*Source (60–64):* research(Security/Audit/Compliance); ConfusionClarity(Pillar 4; §2.5).

---

## 18. Advanced Visualization & Simulation Widgets

- **EKOS-14-065 (Should, P2)** — **Multi-Cloud Topology Designer** — drag real cloud
  services; live cost estimates beside it (pricing-API via connectors); Trade-off Advisor
  suggests cheaper/faster alternatives with *why* (NFR links).
- **EKOS-14-066 (Should, P2)** — **Kubernetes Cluster Painter** — drag Pods/Services/
  Ingress; AI warns on unschedulable resource requests; suggests node pools.
- **EKOS-14-067 (Could, P3)** — **Helm Chart Diff** — visual tree of a Helm release,
  changes highlighted, linked to the requesting feature.
- **EKOS-14-068 (Should, P2)** — **Integration Flow Animator** — drop systems/APIs/queues;
  animated packets flow along edges (real trace or simulated); click packet → payload
  schema → requirement.
- **EKOS-14-069 (Should, P2)** — **Pattern Library Widget** — canonical integration
  patterns (Scatter-Gather, Content Enricher…); drag one → auto-creates nodes/links.
- **EKOS-14-070 (Should, P2)** — **Security Topology & Attack Simulator** — security zones
  (DMZ/internal); simulate attacker path (red packet); firewalls/IDS light up; AI explains
  kill chain + controls (NIST).
- **EKOS-14-071 (Should, P2)** — **Packet Tracer Widget** — subnets/routers/ACLs; inject a
  packet (src/dst IP); glowing dot traverses with allow/block decisions; AI explains the
  blocking ACL + linked compliance requirement (e.g. PCI-DSS 1.2).
- **EKOS-14-072 (Should, P2)** — **Resilience Map** — component redundancy across AZs;
  What-If slider simulates zone failure; affected components grey out; cost impact flashes;
  AI proposes well-architected improvements.
- **EKOS-14-073 (Should, P2)** — **DR Plan Timeline** — Gantt-like recovery sequence; AI
  auto-drafts a runbook from the architecture graph. *(Relates DR dashboard, Vol 09.)*
- **EKOS-14-074 (Should, P2)** — **FinOps Lens Widget** — treemap of cloud resources sized
  by cost, colored by team/feature; drill from a feature requirement; AI cost-attribution.
- **EKOS-14-075 (Should, P2)** — **UML Family Widgets** — Class/Sequence/State/Activity as
  C4-style drill-downs; auto-generated from code, manually enrichable; AI explains the
  pattern.
- **EKOS-14-076 (Should, P2)** — **Change Impact Simulator** — on a requirement change,
  propagate "affected" tags to dependent services/tests/deployments; animated ripples; AI
  drafts a change request. *(Substrate: Vol 01 §19.)*
- **EKOS-14-077 (Should, P2)** — **What-If Analysis Dashboard** — drag sliders (load,
  users, region failure); propagate impact across cost/perf/architecture; animated result.
- **EKOS-14-078 (Should, P2)** — **Uncertainty-Driven Roadmap (meta-widget)** — map
  `Uncertainty` nodes (Vol 01 §10) onto a timeline; adjusts as real events arrive; the
  Horizon Scanner (Vol 03) feeds it.

*Source (65–78):* research(architecture-domain widgets; advanced viz/sim); blueprint;
ConfusionClarity §3.6.

---

## 19. Enterprise / Architecture-Domain Widgets

- **EKOS-14-079 (Should, P2)** — **TOGAF ADM Navigator Widget** — journey map of ADM
  phases (Prelim, A, B, C…); each contains clickable artifact widgets; AI auto-generates a
  draft Architecture Definition Document (Word/PDF).
- **EKOS-14-080 (Should, P2)** — **Zachman Matrix** — 6×6 grid, each cell a widget
  container; click cell → filtered widget; AI validates row/column consistency.
- **EKOS-14-081 (Should, P2)** — **Cloud Architecture Designers (AWS / Azure / GCP / IBM
  Watson / cloud-neutral)** — provider service nodes with pre-built compliance links;
  drag a requirement → cloud node → traceability; live cost. *(Provider-pluggable.)*
- **EKOS-14-082 (Should, P2)** — **Governance Workbench Widget** — Policy Canvas /
  Violation Dashboard / Audit Explorer / Exception Management. *(Engine: Vol 05 §9.)*
- **EKOS-14-083 (Should, P2)** — **Chain-of-Thought Visualizer** — renders the AI's
  reasoning tree linked to graph evidence. *(Data: Vol 03 §14.)*
- **EKOS-14-084 (Could, P3)** — **Executive Mission Control** — portfolio-level health/
  risk/adoption/cost overview across the enterprise graph. *(Vol 09 §18.)*
- **EKOS-14-085 (Could, P3)** — **Green Lens / Carbon Accounting Widget** — scores features/
  architectures for carbon impact (unused resources, polling vs eventing). *(Policy: Vol
  05 §15; ESG procurement criterion.)*
- **EKOS-14-086 (Should, P2)** — **Data Residency Map** — where each tenant's data resides.
  *(Vol 09 §8.)*
- **EKOS-14-087 (Should, P2)** — **API Explorer Widget** — discover/test the public GraphQL
  API in-app. *(Vol 09 §12.)*
- **EKOS-14-088 (Should, P2)** — **Usage Analytics / Adoption Widget** — active users,
  widget views, AI insights consumed, workflow completion; ROI report. *(Vol 09 §9.)*
- **EKOS-14-089 (Should, P2)** — **Solution Design / Trade-off Analysis Widget** — weighted
  options scoring, recommendation, decision rationale. *(ESIP ships this; Vol 03 Trade-off
  Advisor feeds it.)*
- **EKOS-14-090 (Should, P2)** — **NFR & Quality Attributes Widget** — structured ISO-25010
  NFR capture (Performance/Availability/Security/Maintainability/Compliance/Usability).
  *(ESIP ships this.)*
- **EKOS-14-091 (Could, P3)** — **PI Planning / Sprint / Velocity Widgets** — SAFe PI board,
  sprint Kanban, burn-up/velocity. *(ESIP ships PI board + flag-gated sprint board; SAFe
  Step 7 metrics pending — [[esip-resume-state]]; note: feeds-Jira-not-replaces stance.)*

*Source (79–91):* research(architecture domains as widgets); blueprint §17; ConfusionClarity
§3.6; [[project-esip]]/[[esip-resume-state]].

---

## 20. Cross-Cutting Widget Capabilities

- **EKOS-14-092 (Must, MVP)** — Every widget SHALL maintain **bidirectional traceability**
  — hover/select highlights related items across all other widgets/phases. *Source:*
  research(glue §); Vol 02 EKOS-02-006.

- **EKOS-14-093 (Must, MVP)** — Every widget SHALL be **exportable** (Word/HTML/PDF, and
  packageable into `.ekr`) under the security projection. *Source:* Vol 02 §5, Vol 06.

- **EKOS-14-094 (Should, P2)** — Media types (image/GIF/video/audio/3D/prototype/AR-VR) MAY
  be embedded in any widget as first-class nodes. *Source:* blueprint §9; Vol 06 §23.

---

## 21. Constitution Conformance (catalog-specific)

- **EKOS-14-095 (Must, MVP)** — Every widget is a **graph projection** (Constitution §4),
  not a standalone datastore. *Source:* Constitution §4; Vol 01.

- **EKOS-14-096 (Must, MVP)** — Cloud-provider widgets and external data sources are
  **illustrative & pluggable** (Constitution §6); provider names are examples. *Source:*
  Constitution §6.

- **EKOS-14-097 (Must, MVP)** — Catalog widgets respect ABAC masking (Vol 04) and AI
  scoping (Vol 03) — no widget bypasses the security projection. *Source:* Vol 04, Vol 01
  EKOS-01-056.

---

## 22. Relationship to Existing ESIP Widget Set

- **EKOS-14-098 (Must, MVP)** — ESIP's shipped widgets — **Mermaid diagram, maxGraph flow,
  BPMN, Matrix (RACI/CRUD/SIPOC/BMC), Trade-off, NFR, Callout, Requirement, ADR, Metrics,
  Process-Steps, Status-Badge, Divider**, plus draw.io/Visio/Excalidraw/SVG/Kroki(PlantUML)
  imports — SHALL be recognized as **early implementations** of catalog entries, to be
  re-expressed as graph-backed projections (Vol 02 §14). *Source:* [[project-esip]];
  [[esip-resume-state]].

---

## 23. Open Questions (parked)

- **OQ-14-A** — MVP widget subset vs. full catalog sequencing (which widgets ship Phase 1
  vs later) — build-plan decision; the master-index Priority≠Phase rule applies.
- **OQ-14-B** — Auto-generation fidelity (Live Class Diagram, UML from code; ER → migration
  scripts) — depends on connector/AST depth (Vol 01 §20, Vol 09 import).
- **OQ-14-C** — Whether to split this catalog into 14a (core SDLC) / 14b (industry +
  advanced) for build tracking — organizational, not scope.
- **OQ-14-D** — Live external data widgets (Competitive Intelligence scraping, cloud
  pricing APIs) legality/rate-limits — align with Vol 04 anti-exfiltration + legal.

---

## 24. Requirement Index (this volume)

EKOS-14-001 … EKOS-14-098 (stable numbering; ~88 distinct catalog widgets across
requirements/architecture/development phases, 6 industry verticals, project/QA/business/
ops/training/privacy categories, advanced visualization/simulation, and enterprise/
architecture-domain widgets expand to 110 distinct stated rules incl. cross-cutting
capabilities & ESIP mapping).

> **Next volume:** `15 — Deployment, Tech-Stack & Reference Architecture`.
> Per protocol: STOP here. Await user "proceed".
