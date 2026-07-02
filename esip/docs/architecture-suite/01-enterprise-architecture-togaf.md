# Volume 01 — Enterprise Architecture & TOGAF Automation
## ESIP Architecture Automation Suite — Requirements Catalog

---

### Document Control

| Field | Value |
|---|---|
| Volume | 01 |
| Domain | Enterprise Architecture & TOGAF |
| ID Prefix | REQ-EA |
| Version | 1.0 |
| Date | 2026-06-23 |
| Status | Baseline |
| Frameworks Covered | TOGAF 10 / ADM, Architecture Content Framework, Enterprise Continuum, Architecture Repository, Architecture Capability Framework |
| Primary Personas | Enterprise Architect, Architecture Practice Lead, Chief Architect, Governance Board Member |
| Out-of-Scope (see other volumes) | ArchiMate notation (V03), Zachman/DoDAF/UAF (V02), Portfolio/CodeScan/Sunset mechanics (V09), Document rendering/export (V10), AI engine internals (V06), Ingestion pipeline (V07), Knowledge Hub connectors (V08), Role-based architects (V05), Solution Architect (V04) |

---

### Scope Statement

This volume specifies all functional, non-functional, integration, data, AI/ML, UX, security, governance, and reporting requirements for the ESIP suite's automation and assistance of Enterprise Architecture practice as defined by The Open Group TOGAF Standard, 10th Edition. Requirements cover the full ADM cycle, all ADM techniques, the Architecture Content Framework, Enterprise Continuum, Architecture Repository, and Architecture Capability Framework. ArchiMate diagram generation is referenced as AI-assist intent only; notation specifics are deferred to V03. AI model/inference specifics are deferred to V06.

> **Build phase:** Enterprise Architect / TOGAF is **Phase 2** per `BUILD-PLAN-MVP-AND-PHASING.md`. Per the master-index ruling (§6 ruling #1), **`Must` means required in the product, not required in the MVP** — these requirements are not built in Phase 1. Priority (MoSCoW) is decoupled from build sequence.

---

## 1. Preliminary Phase Automation

### 1.1 Architecture Principles Management

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-001 | Principles Catalog CRUD | The suite SHALL provide a structured catalog for creating, reading, updating, and retiring architecture principles with fields for name, statement, rationale, implications, and maturity state. | Functional | Enterprise Architect | Must | TOGAF 10 §3.6; principles are foundational inputs to every ADM phase |
| REQ-EA-002 | Principle Template Library | The suite SHALL ship a library of at least 30 pre-built principle templates spanning Business, Data, Application, and Technology domains aligned to TOGAF standard principle examples. | Functional | Enterprise Architect | Must | Reduces time-to-value; mirrors LeanIX starter content |
| REQ-EA-003 | Principle Conflict Detection | The suite SHALL detect and flag logical conflicts between registered principles using AI analysis and present them to the architect for resolution. | AI/ML | Enterprise Architect | Should | Prevents contradictory governance signals downstream |
| REQ-EA-004 | Principle Traceability | The suite SHALL maintain bi-directional traceability links from each principle to the ADM phases, decisions, and architecture building blocks (ABBs) that reference or are governed by it. | Data | Enterprise Architect | Must | Enables impact analysis when principles change |
| REQ-EA-005 | Principle Version History | The suite SHALL retain a full version history of every principle, recording who changed what, when, and why, with the ability to diff any two versions. | Governance | Enterprise Architect | Must | Audit trail required for governance compliance |
| REQ-EA-006 | Principle Approval Workflow | The suite SHALL enforce a configurable multi-stage approval workflow (Draft → Review → Approved → Retired) for principles, with email/Teams notifications at each state transition. | Governance | Enterprise Architect, Governance Board Member | Must | Formalizes adoption; prevents unapproved principles entering live use |
| REQ-EA-007 | Principle Export | The suite SHALL export the principles catalog to Word, PDF, and Excel formats with configurable cover page and branding. | Functional | Enterprise Architect | Must | Required for board packs and stakeholder distribution |
| REQ-EA-008 | AI Principle Drafting | The suite SHALL offer an AI-assisted drafting mode that generates draft principle statements and rationale from ingested artifacts (strategy documents, policies, regulations) in < 30 seconds per principle. | AI/ML | Enterprise Architect | Must | Core differentiator; reduces days of drafting to minutes |
| REQ-EA-009 | Principle Relevance Scoring | The suite SHALL score each principle for relevance against active architecture work items and surface low-relevance or orphaned principles for review. | AI/ML | Enterprise Architect | Should | Keeps principles catalog lean and actionable |
| REQ-EA-010 | Principle Tagging & Search | The suite SHALL support free-form tagging and full-text search of principles with filter by domain, maturity, business unit, and framework source. | Functional | Enterprise Architect | Must | Usability at scale |

### 1.2 Framework Tailoring

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-011 | TOGAF Tailoring Configurator | The suite SHALL provide a visual configurator enabling architects to tailor the TOGAF ADM by enabling/disabling phases, renaming deliverables, and mapping organization-specific methods to ADM activities. | Functional | Enterprise Architect | Must | TOGAF 10 §5.1 mandates tailoring; no tool currently automates this (gap vs. BiZZdesign) |
| REQ-EA-012 | Tailoring Snapshot & Clone | The suite SHALL allow a tailoring configuration to be saved as a named snapshot and cloned for reuse across programs or business units. | Functional | Enterprise Architect | Should | Multi-BU enterprises need variant configurations |
| REQ-EA-013 | Hybrid Framework Mapping | The suite SHALL support overlay mappings between TOGAF ADM and SAFe, Scrum, ITIL 4, and COBIT 2019 so that ADM activities can be linked to ceremonies or processes in the overlaid framework. | Integration | Enterprise Architect | Should | Modern enterprises run hybrid operating models |
| REQ-EA-014 | Custom Phase Injection | The suite SHALL allow injection of organization-specific phases or sub-phases into the ADM lifecycle without breaking standard TOGAF traceability. | Functional | Enterprise Architect | Could | Large regulated enterprises (FSI, Defence) often extend the ADM |
| REQ-EA-015 | Tailoring Rationale Capture | The suite SHALL require architects to record a rationale when deviating from a TOGAF standard deliverable or skipping a phase, stored in the Governance Log. | Governance | Enterprise Architect | Must | Demonstrates TOGAF compliance despite tailoring |

### 1.3 Governance Setup

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-016 | Governance Board Configuration | The suite SHALL allow configuration of one or more Architecture Governance Boards with defined membership, meeting cadence, quorum rules, and decision scope. | Functional | Architecture Practice Lead | Must | TOGAF Architecture Capability Framework §44 |
| REQ-EA-017 | Governance Log | The suite SHALL maintain a tamper-evident Governance Log recording all board decisions, waivers, escalations, and compliance outcomes with timestamps and actor IDs. | Governance | Architecture Practice Lead | Must | Foundational audit artifact; referenced in ADM phases G and H |
| REQ-EA-018 | Governance Metrics Dashboard | The suite SHALL present a real-time dashboard of governance KPIs including open compliance requests, average decision cycle time, waiver rate, and principle violation count. | Reporting | Architecture Practice Lead | Must | Enables continuous improvement of governance practice |
| REQ-EA-019 | Governance Notification Engine | The suite SHALL send configurable push/email/Teams notifications for governance events including new requests, overdue reviews, and board decisions. | Integration | Enterprise Architect | Must | Keeps stakeholders informed without manual chasing |
| REQ-EA-020 | Role-Based Governance Access | The suite SHALL enforce role-based access control (RBAC) on governance objects such that only authorized roles can approve, reject, or escalate governance requests. | Security | Architecture Practice Lead | Must | Prevents unauthorized changes to governance decisions |
| REQ-EA-021 | Architecture Contract Template | The suite SHALL provide at least five parameterized Architecture Contract templates (System, Migration, Cloud, Data, Integration) that can be instantiated and linked to a project or initiative. | Functional | Enterprise Architect | Must | TOGAF Architecture Capability Framework §44.4 |
| REQ-EA-022 | Governance SLA Tracking | The suite SHALL track SLA compliance for each governance activity type (review in 5 days, decision in 10 days, etc.) and flag breaches. | Governance | Architecture Practice Lead | Should | Demonstrates practice maturity (CMF Level 3+) |

---

## 2. ADM Phase A — Architecture Vision

### 2.1 Vision Scope & Mandate

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-023 | Vision Scope Capture | The suite SHALL provide a structured form for capturing Architecture Vision scope including business context, constraints, assumptions, drivers, and success measures. | Functional | Enterprise Architect | Must | Phase A entry point per TOGAF 10 §15 |
| REQ-EA-024 | AI Vision Statement Generation | The suite SHALL generate a draft Architecture Vision statement from ingested strategy documents, OKRs, and business cases using AI, requiring < 60 seconds generation time. | AI/ML | Enterprise Architect | Must | Core productivity differentiator |
| REQ-EA-025 | Statement of Architecture Work | The suite SHALL auto-populate a Statement of Architecture Work (SoAW) template with project metadata, sponsor, scope boundaries, schedule, and resource estimates, linking it to the active ADM cycle. | Functional | Enterprise Architect | Must | Required TOGAF output of Phase A |
| REQ-EA-026 | SoAW Approval Workflow | The suite SHALL route the Statement of Architecture Work through a configurable approval workflow terminating in a digital sign-off from the Architecture Sponsor. | Governance | Enterprise Architect | Must | Provides formal authorization to proceed |
| REQ-EA-027 | Vision Diagram Auto-Draft | The suite SHALL produce a draft Solution Concept Diagram (high-level) from the Vision scope and available baseline artifacts using AI (notation specifics per V03). | AI/ML | Enterprise Architect | Must | Saves 2-4 hours of diagramming per engagement |
| REQ-EA-028 | Vision to Phase B Handoff | The suite SHALL propagate approved Vision scope, assumptions, drivers, and stakeholder list automatically to Phase B work items, eliminating manual re-entry. | Functional | Enterprise Architect | Must | Reduces rework; closes the ADM loop |
| REQ-EA-029 | Vision Version Control | The suite SHALL maintain a versioned history of Architecture Vision documents, allowing comparison between drafts and a rollback to any prior version. | Data | Enterprise Architect | Must | Supports iterative refinement across ADM iterations |

### 2.2 Stakeholder Management

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-030 | Stakeholder Register | The suite SHALL maintain a Stakeholder Register with fields for name, role, organization, concerns, influence level, interest level, and communication preferences. | Functional | Enterprise Architect | Must | TOGAF ADM Technique — Stakeholder Management §24 |
| REQ-EA-031 | Stakeholder Map Visualization | The suite SHALL render an interactive Power/Interest Grid and RACI matrix from the Stakeholder Register that updates in real time as register entries are modified. | Functional | Enterprise Architect | Must | Visual technique prescribed in TOGAF §24.4 |
| REQ-EA-032 | AI Stakeholder Identification | The suite SHALL analyze ingested project artifacts and org charts to propose candidate stakeholders with suggested role, concern, and influence classification. | AI/ML | Enterprise Architect | Should | Reduces missed stakeholders during Phase A |
| REQ-EA-033 | Concern Capture & Traceability | The suite SHALL capture individual stakeholder concerns and trace them to architecture requirements, risks, and ADM deliverables. | Data | Enterprise Architect | Must | Required for requirements completeness per TOGAF |
| REQ-EA-034 | Communication Plan Generator | The suite SHALL generate a Stakeholder Communication Plan template pre-populated with stakeholder groups, preferred channels, and suggested cadences from the Stakeholder Register. | Functional | Enterprise Architect | Should | Reduces manual planning effort |
| REQ-EA-035 | Stakeholder Import | The suite SHALL import stakeholders from Active Directory / Entra ID, Jira project roles, or CSV files to seed the Stakeholder Register. | Integration | Enterprise Architect | Should | Large programs have 50+ stakeholders — manual entry is impractical |

### 2.3 Business Scenarios

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-036 | Business Scenario Authoring | The suite SHALL provide a structured authoring interface for Business Scenarios capturing: problem description, business and technical environment, desired outcome, human actors, computer actors, and success criteria. | Functional | Enterprise Architect | Must | TOGAF ADM Technique §26 |
| REQ-EA-037 | Business Scenario AI Extraction | The suite SHALL extract candidate business scenarios from ingested artifacts (interviews, workshop transcripts, strategy docs) using AI, presenting a ranked shortlist for architect confirmation. | AI/ML | Enterprise Architect | Must | Core ingestion-to-scenario automation capability |
| REQ-EA-038 | Scenario-to-Requirement Tracing | The suite SHALL automatically generate draft architecture requirements from confirmed business scenarios and maintain bi-directional trace links. | Functional | Enterprise Architect | Must | Closes the scenario → requirement loop per TOGAF Requirements Management |
| REQ-EA-039 | Scenario Priority Scoring | The suite SHALL score business scenarios by strategic impact, urgency, and feasibility using configurable weighting, enabling prioritized architecture focus. | AI/ML | Enterprise Architect | Should | Guides resource allocation across competing scenarios |
| REQ-EA-040 | Scenario Reuse Library | The suite SHALL maintain a reuse library of past Business Scenarios searchable by industry, capability domain, and keyword, enabling cross-engagement reuse. | Data | Enterprise Architect | Should | Builds institutional knowledge over time |

---

## 3. ADM Phase B — Business Architecture

### 3.1 Business Capability Modeling

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-041 | Business Capability Map Generation | The suite SHALL generate a multi-level Business Capability Map (Levels 1–3) from ingested org charts, process documents, and strategy artifacts using AI, within 90 seconds of ingestion. | AI/ML | Enterprise Architect | Must | BCM is the primary deliverable anchor of Phase B |
| REQ-EA-042 | Capability Heat Mapping | The suite SHALL overlay configurable heat-map dimensions (investment level, strategic importance, current performance, risk, technical debt) on the Business Capability Map with per-capability color coding. | Functional | Enterprise Architect | Must | Standard EA deliverable requested by boards and C-suite |
| REQ-EA-043 | Capability CRUD | The suite SHALL allow architects to create, edit, delete, and re-parent business capabilities in a drag-and-drop tree view with instant diagram refresh. | Functional | Enterprise Architect | Must | Capability maps evolve; manual editing must be smooth |
| REQ-EA-044 | Capability to Application Mapping | The suite SHALL maintain an N:M capability-to-application mapping and surface it as an Application Portfolio by Capability view, filterable by lifecycle state and business unit. | Data | Enterprise Architect | Must | Cross-domain traceability Phase B→C |
| REQ-EA-045 | Capability to Value Stream Mapping | The suite SHALL link business capabilities to value streams, enabling visualization of which capabilities participate in which end-to-end value delivery chains. | Functional | Enterprise Architect | Must | TOGAF 10 Capability-Based Planning §30 |
| REQ-EA-046 | Capability Maturity Assessment | The suite SHALL support capability maturity scoring (1–5) per capability instance using configurable maturity models, with automated gap identification against target maturity. | Functional | Enterprise Architect | Must | Gap analysis requires maturity baseline |
| REQ-EA-047 | Capability Catalog Export | The suite SHALL export the capability catalog and heat maps to Excel, PowerPoint, and PDF with configurable column selection and heat-map legend. | Functional | Enterprise Architect | Must | Boards consume capability data in PowerPoint/Excel |
| REQ-EA-048 | AI Capability Taxonomy Alignment | The suite SHALL compare the organization's capability taxonomy against industry reference models (eTOM, BIAN, ARTS, PCF) using AI and suggest alignment or gap annotations. | AI/ML | Enterprise Architect | Should | Accelerates benchmarking; unique vs. LeanIX |
| REQ-EA-049 | Capability Duplicate Detection | The suite SHALL detect duplicate or near-duplicate capabilities in the map using semantic similarity and flag them for architect review. | AI/ML | Enterprise Architect | Should | Keeps maps clean in large enterprises |
| REQ-EA-050 | Capability Ownership Assignment | The suite SHALL assign business owners to capabilities with ownership visible in heat maps and filterable in catalog views. | Data | Enterprise Architect | Must | Accountability is required for governance |

### 3.2 Value Stream Modeling

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-051 | Value Stream Definition | The suite SHALL support definition of value streams with fields for name, triggering event, value proposition, participating stakeholders, and enabling capabilities. | Functional | Enterprise Architect | Must | TOGAF 10 §29 — Value Streams |
| REQ-EA-052 | Value Stream Stage Modeling | The suite SHALL allow decomposition of a value stream into ordered stages, each with entry/exit criteria, enabling capabilities, and performance metrics. | Functional | Enterprise Architect | Must | Stage-level modeling required for gap analysis |
| REQ-EA-053 | Value Stream Diagram | The suite SHALL render value stream diagrams showing stages, triggering events, stakeholder swim lanes, and enabling capability annotations, exportable as SVG/PNG/PDF. | Functional | Enterprise Architect | Must | Visual communication to business stakeholders |
| REQ-EA-054 | AI Value Stream Extraction | The suite SHALL extract candidate value streams from ingested process documents and customer journey maps using AI. | AI/ML | Enterprise Architect | Should | Accelerates Phase B kickoff |
| REQ-EA-055 | Value Stream to Process Linkage | The suite SHALL link value stream stages to BPMN business process models where available, maintaining navigable cross-references. | Integration | Enterprise Architect | Should | Bridges EA value stream and process modeling |

### 3.3 Business Model Canvas

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-056 | Business Model Canvas Authoring | The suite SHALL provide an interactive Business Model Canvas (Osterwalder) authoring view with the nine standard blocks: Key Partners, Activities, Resources, Value Propositions, Customer Relationships, Channels, Segments, Cost Structure, Revenue Streams. | Functional | Enterprise Architect | Must | TOGAF 10 references BMC as a business architecture input |
| REQ-EA-057 | AI BMC Population | The suite SHALL populate draft Business Model Canvas blocks from ingested annual reports, strategy decks, and investor materials using AI. | AI/ML | Enterprise Architect | Must | Dramatic time savings in Phase B initiation |
| REQ-EA-058 | BMC to Capability Traceability | The suite SHALL generate traceability links from BMC blocks to the business capability map (e.g., Key Activities → enabling capabilities). | Data | Enterprise Architect | Should | Connects business model thinking to architecture artifacts |
| REQ-EA-059 | BMC Version Comparison | The suite SHALL support side-by-side comparison of current vs. target BMC to surface business model transformation scope. | Functional | Enterprise Architect | Should | Useful in digital transformation programs |

### 3.4 Organization & Process Architecture

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-060 | Organization Unit Catalog | The suite SHALL maintain a catalog of organization units with hierarchy, parent relationships, associated capabilities, and geographic footprint. | Data | Enterprise Architect | Must | Phase B baseline for org architecture |
| REQ-EA-061 | Process Catalog | The suite SHALL maintain a catalog of business processes linked to capabilities, org units, applications, and data objects. | Data | Enterprise Architect | Must | Core Phase B artifact — Business Process/Function Catalog |
| REQ-EA-062 | BPMN Process Import | The suite SHALL import BPMN 2.0 files (.bpmn) and register processes, tasks, roles, and data objects into the Architecture Repository. | Integration | Enterprise Architect | Should | Many enterprises already have BPMN models |
| REQ-EA-063 | Process AI Extraction | The suite SHALL extract processes and tasks from ingested Word/PDF procedure documents and populate the process catalog with confidence scores. | AI/ML | Enterprise Architect | Should | Reduces manual catalog building |
| REQ-EA-064 | Function Hierarchy Tree | The suite SHALL render the organizational function hierarchy as a navigable tree with drill-down to sub-functions, linked processes, and owning org units. | Functional | Enterprise Architect | Must | Business Function Decomposition Diagram per TOGAF ACF |

### 3.5 Phase B Deliverables

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-065 | Phase B Architecture Definition Document | The suite SHALL auto-assemble a Phase B Architecture Definition Document (ADD) section comprising baseline and target business architecture descriptions with all linked artifacts, ready for review in one click. | Functional | Enterprise Architect | Must | TOGAF required output |
| REQ-EA-066 | Business Architecture Roadmap Draft | The suite SHALL generate a draft Business Architecture Roadmap of work packages derived from gap analysis, sortable by priority and effort. | Functional | Enterprise Architect | Must | Phase B → Phase E handoff artifact |
| REQ-EA-067 | Baseline vs. Target Comparison Table | The suite SHALL present a structured comparison table of baseline vs. target business architecture elements per capability/process domain for all Phase B artifacts. | Functional | Enterprise Architect | Must | Gap analysis input |

---

## 4. ADM Phase C — Information Systems Architecture

### 4.1 Data Architecture

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-068 | Data Entity Catalog | The suite SHALL maintain a Data Entity Catalog with fields for entity name, definition, owning business unit, classification, sensitivity level, lifecycle state, and related capabilities. | Data | Enterprise Architect | Must | TOGAF Phase C Data Architecture — Data Entity/Data Component Catalog |
| REQ-EA-069 | AI Data Entity Extraction | The suite SHALL extract data entities and attributes from ingested ERDs, database schemas (.sql, .ddl), Excel data dictionaries, and API specifications using AI. | AI/ML | Enterprise Architect | Must | Large enterprises have hundreds of data entities — manual entry is infeasible |
| REQ-EA-070 | Data Entity Relationship Diagram | The suite SHALL auto-generate an Entity Relationship Diagram (ERD) from the Data Entity Catalog, with configurable scope filters by domain, sensitivity, or owning unit. | Functional | Enterprise Architect | Must | ERD is a required Phase C diagram |
| REQ-EA-071 | Data Sensitivity Classification | The suite SHALL enforce data sensitivity classification (Public, Internal, Confidential, Restricted) on every data entity, with AI-suggested classifications based on entity name and definition. | Security | Enterprise Architect | Must | Data governance and regulatory compliance |
| REQ-EA-072 | Data Flow Catalog | The suite SHALL maintain a Data Flow Catalog capturing source entity, target entity, transformation, transport mechanism, frequency, and data classification per flow. | Data | Enterprise Architect | Must | Required for Data Flow Diagram generation |
| REQ-EA-073 | Data Flow Diagram Generation | The suite SHALL generate Level 0 and Level 1 Data Flow Diagrams (DFDs) from the Data Flow Catalog within 30 seconds of request. | Functional | Enterprise Architect | Must | Phase C required diagram |
| REQ-EA-074 | Data Standards Catalog | The suite SHALL maintain a Data Standards Catalog with reference data standards (ISO 3166, ISO 4217, HL7 FHIR, XBRL, etc.) linkable to data entities for compliance tracking. | Data | Enterprise Architect | Should | Supports Standards Information Base |
| REQ-EA-075 | Logical Data Model | The suite SHALL support authoring of a Logical Data Model (LDM) linked to the data entity catalog, with AI-assisted normalization suggestions. | Functional | Enterprise Architect | Should | Phase C deliverable for data-intensive programs |
| REQ-EA-076 | Data Lifecycle Modeling | The suite SHALL allow modeling of data entity lifecycles (Create, Read, Update, Delete, Archive, Purge) and render them as lifecycle diagrams. | Data | Enterprise Architect | Should | TOGAF data lifecycle artifact |
| REQ-EA-077 | Data Architecture Definition Document | The suite SHALL auto-assemble the Data Architecture section of the Architecture Definition Document from catalog entries, diagrams, and gap analysis results. | Functional | Enterprise Architect | Must | Required TOGAF Phase C output |
| REQ-EA-078 | GDPR / Data Regulation Tagging | The suite SHALL allow tagging of data entities with applicable regulations (GDPR Article, CCPA, HIPAA, SOX) and generate a regulation-to-entity traceability matrix. | Security | Enterprise Architect | Must | Regulatory compliance is a top driver for EA practice |
| REQ-EA-079 | Data Master Index | The suite SHALL identify and flag candidate master data entities across the catalog using AI analysis and present a proposed Master Data Ownership map. | AI/ML | Enterprise Architect | Should | Master data management is a frequent Phase C outcome |

### 4.2 Application Architecture

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-080 | Application Portfolio Catalog | The suite SHALL maintain an Application Portfolio Catalog with fields for application name, owner, vendor, version, technology stack, lifecycle state (Active, Sunset, Retire, Invest), business capabilities supported, and cost data. | Data | Enterprise Architect | Must | Core Phase C Application Architecture artifact |
| REQ-EA-081 | AI Application Discovery | The suite SHALL discover and ingest application metadata from ServiceNow CMDB, AWS Service Catalog, Azure Resource Manager, and CSV exports to seed the Application Portfolio Catalog. | Integration | Enterprise Architect | Must | Manual catalog population is the #1 time sink in EA |
| REQ-EA-082 | Application Lifecycle State Tracking | The suite SHALL track lifecycle state transitions of each application with dates, owner sign-off, and linked roadmap work packages. | Functional | Enterprise Architect | Must | Lifecycle states drive retirement and investment decisions |
| REQ-EA-083 | Application Heat Map | The suite SHALL render an Application Portfolio Heat Map with configurable axes (e.g., Business Value vs. Technical Health, Strategic Fit vs. Cost) with per-application color and size coding. | Functional | Enterprise Architect | Must | Standard EA board deliverable; core LeanIX/Alfabet feature |
| REQ-EA-084 | Application Interaction Matrix | The suite SHALL generate an Application Interaction Matrix (N×N grid) showing integration relationships, data flows, and dependency types between applications. | Functional | Enterprise Architect | Must | Phase C diagram — Application Communication Diagram source |
| REQ-EA-085 | Application Communication Diagram | The suite SHALL render an Application Communication Diagram from the Application Interaction Matrix with configurable layout and filterable by capability domain or technology. | Functional | Enterprise Architect | Must | Required TOGAF Phase C diagram |
| REQ-EA-086 | Application to Data Entity Matrix | The suite SHALL generate a CRUD matrix mapping each application against the data entities it creates, reads, updates, or deletes. | Data | Enterprise Architect | Must | Data ownership and governance artifact |
| REQ-EA-087 | Duplicate Application Detection | The suite SHALL analyze the Application Portfolio using AI to identify functionally duplicate or overlapping applications and present a rationalization shortlist. | AI/ML | Enterprise Architect | Must | Application rationalization is a primary Phase C outcome |
| REQ-EA-088 | Application Architecture ADD Section | The suite SHALL auto-assemble the Application Architecture section of the Architecture Definition Document. | Functional | Enterprise Architect | Must | Required TOGAF Phase C output |
| REQ-EA-089 | SaaS / Cloud Application Enrichment | The suite SHALL auto-enrich SaaS application metadata (vendor, category, compliance certifications) from a maintained reference database covering the top 500 enterprise SaaS products. | Data | Enterprise Architect | Should | Saves manual research; differentiator vs. pure-modeling tools |
| REQ-EA-090 | Application Vendor Risk Scoring | The suite SHALL score application vendor risk (financial health, support lifecycle, concentration risk) using AI-analyzed public data and flag high-risk vendors. | AI/ML | Enterprise Architect | Should | Risk management is a TOGAF Phase C consideration |
| REQ-EA-091 | Application Cost Attribution | The suite SHALL capture and allocate application total cost of ownership (TCO) by business unit and capability, enabling cost-per-capability analysis. | Reporting | Enterprise Architect | Should | Required for portfolio investment decisions |

---

## 5. ADM Phase D — Technology Architecture

### 5.1 Technology Catalog & Standards

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-092 | Technology Catalog | The suite SHALL maintain a Technology Catalog with fields for technology name, type, vendor, version, license model, lifecycle state, compliance certifications, and supported standards. | Data | Enterprise Architect | Must | Foundation of Phase D — Technology Standards Catalog |
| REQ-EA-093 | Standards Information Base | The suite SHALL maintain a Standards Information Base (SIB) organized by standard type (technical standards, product standards, de-facto standards) with applicability scope and review dates. | Data | Enterprise Architect | Must | Foundational Architecture Repository element — TOGAF §37.2.4 |
| REQ-EA-094 | Technology Lifecycle Radar | The suite SHALL render a Technology Radar (Adopt/Trial/Assess/Hold) view of the Technology Catalog with configurable quadrants and ring thresholds. | Functional | Enterprise Architect | Must | Provides a visual technology positioning tool popular with CTOs |
| REQ-EA-095 | Technology to Application Matrix | The suite SHALL generate a Technology/Application matrix showing which technologies underpin which applications, filterable by lifecycle state. | Data | Enterprise Architect | Must | Phase D cross-domain traceability |
| REQ-EA-096 | Technology Debt Identification | The suite SHALL flag technologies in the catalog that are End-of-Life (EoL), End-of-Support (EoS), or approaching sunset within 12/24 months, sourced from vendor lifecycle data maintained in the SIB. | Functional | Enterprise Architect | Must | Technology debt is a key Phase D concern |
| REQ-EA-097 | AI Technology Gap Analysis | The suite SHALL compare the baseline technology catalog against target technology standards using AI and generate a gap analysis table with recommended remediation actions. | AI/ML | Enterprise Architect | Must | Automates Phase D gap analysis |
| REQ-EA-098 | Technology Architecture Diagram | The suite SHALL generate a Technology Architecture Diagram (environments, platforms, nodes, infrastructure components) from catalog data, with ArchiMate notation (see V03). | Functional | Enterprise Architect | Must | Required Phase D diagram |
| REQ-EA-099 | Cloud Platform Mapping | The suite SHALL support mapping of technology components to cloud platforms (AWS, Azure, GCP) with service-tier annotations (IaaS/PaaS/SaaS) and region/availability zone metadata. | Data | Enterprise Architect | Should | Cloud architectures dominate modern Phase D |
| REQ-EA-100 | Technology Architecture ADD Section | The suite SHALL auto-assemble the Technology Architecture section of the Architecture Definition Document. | Functional | Enterprise Architect | Must | Required TOGAF Phase D output |

### 5.2 Environment & Infrastructure

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-101 | Environments Catalog | The suite SHALL catalog environments (Development, Test, Staging, Production, DR) with hosting model, location, and linked applications. | Data | Enterprise Architect | Must | Environments are a standard Phase D node |
| REQ-EA-102 | Infrastructure Component Import | The suite SHALL import infrastructure inventory from Terraform state files, CloudFormation stacks, and CMDB exports to populate Phase D catalogs. | Integration | Enterprise Architect | Should | Infrastructure-as-code is the dominant deployment model |
| REQ-EA-103 | Network Zone Modeling | The suite SHALL support modeling of network zones (DMZ, Internal, Restricted, Management) and their trust boundaries within the Technology Architecture. | Functional | Enterprise Architect | Should | Security architecture requires zone modeling |

---

## 6. ADM Phase E — Opportunities & Solutions

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-104 | Gap Analysis Automation | The suite SHALL automatically compute gap analysis tables for each architecture domain (Business, Data, Application, Technology) by comparing baseline and target architectures stored in the repository. | Functional | Enterprise Architect | Must | Phase E primary input — gap analysis per TOGAF §19 |
| REQ-EA-105 | Work Package Catalog | The suite SHALL maintain a Work Package Catalog with fields for name, description, business value, effort estimate, cost estimate, risk, dependencies, and linked architecture gaps. | Data | Enterprise Architect | Must | Core Phase E deliverable |
| REQ-EA-106 | AI Work Package Generation | The suite SHALL generate draft work packages from gap analysis results using AI, proposing scope, benefit, and dependency groupings. | AI/ML | Enterprise Architect | Must | Dramatically accelerates Phase E transition planning |
| REQ-EA-107 | Dependency Mapping | The suite SHALL detect and visualize dependencies between work packages, highlighting critical path and sequencing constraints. | Functional | Enterprise Architect | Must | Dependency analysis is a Phase E/F input |
| REQ-EA-108 | Implementation Options Analysis | The suite SHALL support authoring of implementation options (Buy/Build/Reuse) for each work package with pros/cons, cost/risk scoring, and a recommended option. | Functional | Enterprise Architect | Must | Phase E decision artifact |
| REQ-EA-109 | Project Portfolio Alignment | The suite SHALL map work packages to existing projects in connected PPM systems (Jira, ADO, ServiceNow PPM) to identify redundancies and integration points. | Integration | Enterprise Architect | Should | Avoids duplicated investment |
| REQ-EA-110 | Architecture Roadmap Draft | The suite SHALL generate a draft Architecture Roadmap timeline from work packages, sorted by priority and dependency order, in Gantt and swim-lane views. | Functional | Enterprise Architect | Must | Phase E primary output; detailed mechanics in V09 |
| REQ-EA-111 | Transition Architecture Definition | The suite SHALL support definition of up to N Transition Architectures between baseline and target, each with a defined scope, timeline, and success measures. | Functional | Enterprise Architect | Must | TOGAF Phase E Transition Architecture concept |
| REQ-EA-112 | Consolidated Gaps Matrix | The suite SHALL present a consolidated cross-domain Gaps matrix showing all identified gaps organized by domain, severity, and owning work package. | Reporting | Enterprise Architect | Must | Executive communication artifact |

---

## 7. ADM Phase F — Migration Planning

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-113 | Migration Plan Assembly | The suite SHALL assemble a Migration Plan document from work packages, transition architectures, resource assignments, and timeline, formatted per organization template. | Functional | Enterprise Architect | Must | Primary Phase F output |
| REQ-EA-114 | Migration Prioritization Matrix | The suite SHALL generate a Migration Prioritization Matrix scoring work packages on business value, cost, risk, and urgency, supporting portfolio-level sequencing decisions. | Functional | Enterprise Architect | Must | Systematic prioritization per TOGAF Phase F |
| REQ-EA-115 | Cost-Benefit Model | The suite SHALL support per-work-package cost-benefit modeling with NPV, payback period, and ROI calculations using configurable financial parameters. | Functional | Enterprise Architect | Should | Migration planning requires financial justification |
| REQ-EA-116 | Risk Register — Migration | The suite SHALL maintain a Migration Risk Register linked to work packages, with likelihood/impact scoring, risk owners, and mitigation actions. | Governance | Enterprise Architect | Must | Risk management is a Phase F requirement |
| REQ-EA-117 | Migration Dependency Timeline | The suite SHALL render a Migration Timeline diagram showing work packages, their dependencies, transition architecture milestones, and critical path highlighting. | Functional | Enterprise Architect | Must | Visual communication of migration plan |
| REQ-EA-118 | Resource Demand Forecast | The suite SHALL compute a resource demand forecast (FTEs by skill type, months) across the migration timeline from work package effort estimates. | Reporting | Enterprise Architect | Should | Enables capacity planning for the change program |
| REQ-EA-119 | PPM Export | The suite SHALL export the Migration Plan as a project import file compatible with Jira (CSV/JSON), Microsoft Project (.xml), and ServiceNow PPM. | Integration | Enterprise Architect | Should | Bridges EA planning with project execution |

---

## 8. ADM Phase G — Implementation Governance

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-120 | Architecture Compliance Review | The suite SHALL support structured Architecture Compliance Reviews for in-flight projects, with a scored checklist aligned to the active Architecture Contract and Architecture Principles. | Governance | Architecture Practice Lead | Must | Phase G core activity per TOGAF §22 |
| REQ-EA-121 | Compliance Review Workflow | The suite SHALL route Compliance Review requests through a defined workflow (Request → Assessment → Review Meeting → Decision → Publish) with SLA tracking and automated reminders. | Governance | Architecture Practice Lead | Must | Formalizes Phase G process |
| REQ-EA-122 | Compliance Scorecard | The suite SHALL generate a quantified Compliance Scorecard (0–100) for each project review, with individual scores per Architecture Contract clause and principle. | Reporting | Architecture Practice Lead | Must | Objective governance metric |
| REQ-EA-123 | Waiver Management | The suite SHALL support Architecture Waiver requests with fields for justification, risk acknowledgement, compensating controls, and expiry date, routed through the Governance Board. | Governance | Architecture Practice Lead | Must | Waivers are a real-world necessity; must be tracked formally |
| REQ-EA-124 | Architecture Contract Generation | The suite SHALL generate an Architecture Contract from a template, pre-populated with the relevant Architecture Principles, standards, and deliverable obligations applicable to the requesting project. | Functional | Enterprise Architect | Must | Phase G deliverable per TOGAF Architecture Capability Framework |
| REQ-EA-125 | Implementation Governance Log | The suite SHALL record all Phase G decisions, compliance outcomes, waivers, and escalations in the Governance Log with full audit trail. | Governance | Architecture Practice Lead | Must | Tamper-evident record of governance activity |
| REQ-EA-126 | Project Architecture Review Dashboard | The suite SHALL present a dashboard of all active projects under architecture governance, showing review status, compliance score, open exceptions, and assigned architect. | Reporting | Architecture Practice Lead | Must | Practice-wide visibility of governance portfolio |
| REQ-EA-127 | Non-Compliance Escalation | The suite SHALL automatically escalate unresolved compliance exceptions beyond a configurable threshold to the Architecture Board with context and recommended action. | Governance | Architecture Practice Lead | Must | Prevents governance decisions stalling at project level |
| REQ-EA-128 | Architecture Decision Record | The suite SHALL generate Architecture Decision Records (ADRs) from governance decisions, capturing context, options considered, decision, and consequences. | Data | Enterprise Architect | Must | ADRs are increasingly required by modern EA governance |

---

## 9. ADM Phase H — Architecture Change Management

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-129 | Change Request Management | The suite SHALL provide an Architecture Change Request (ACR) form capturing change description, driver (technology event, business event, lessons learned), impact scope, and urgency classification. | Functional | Enterprise Architect | Must | Phase H entry point per TOGAF §23 |
| REQ-EA-130 | Change Impact Analysis | The suite SHALL perform automated impact analysis on a proposed change by traversing the Architecture Repository graph to identify affected capabilities, applications, data entities, technologies, and deliverables. | AI/ML | Enterprise Architect | Must | Core Phase H AI capability; comparable to BiZZdesign impact analysis |
| REQ-EA-131 | Change Classification | The suite SHALL classify incoming changes as Simplex (informal, minor adjustment), Duplex (formal, ADM cycle re-entry), or Dispensation (agreed deviation) per TOGAF Phase H taxonomy. | Functional | Enterprise Architect | Must | Proper classification routes changes to the correct response |
| REQ-EA-132 | Change Trigger Monitoring | The suite SHALL monitor connected external sources (technology news feeds, regulatory update services, vendor EOL databases) for events that may trigger architecture changes and surface them as proposed ACRs. | Integration | Enterprise Architect | Should | Proactive phase H trigger management |
| REQ-EA-133 | ADM Re-Entry Recommendation | The suite SHALL recommend the appropriate ADM phase re-entry point for each Duplex change request based on the scope of architectural impact determined by change impact analysis. | AI/ML | Enterprise Architect | Should | Guides architects to the right starting point for re-iteration |
| REQ-EA-134 | Change Log | The suite SHALL maintain a Change Log recording all ACRs, their classification, impact analysis results, approval decision, and outcomes. | Governance | Architecture Practice Lead | Must | Required for Architecture Repository currency |
| REQ-EA-135 | Architecture Repository Currency Alert | The suite SHALL alert architects when an approved change has not been reflected in the Architecture Repository within a configurable time window (default: 30 days). | Governance | Architecture Practice Lead | Must | Repository currency is a key maturity indicator |

---

## 10. Requirements Management (Central Process)

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-136 | Architecture Requirements Repository | The suite SHALL maintain a central Architecture Requirements Repository storing all requirements derived from ADM phases, stakeholder concerns, business scenarios, and compliance obligations, with unique IDs. | Data | Enterprise Architect | Must | TOGAF Requirements Management is a continuous process across all phases |
| REQ-EA-137 | Requirements Classification | The suite SHALL classify each requirement by type (Functional, NFR, Constraint, Assumption, Principle-derived, Regulatory) and by ADM phase of origin. | Data | Enterprise Architect | Must | Enables targeted filtering during each ADM phase |
| REQ-EA-138 | Requirements Prioritization | The suite SHALL support MoSCoW prioritization of architecture requirements with rationale capture and linkage to business drivers. | Functional | Enterprise Architect | Must | Phase prioritization requires prioritized requirements |
| REQ-EA-139 | Requirements Traceability Matrix | The suite SHALL generate a Requirements Traceability Matrix (RTM) spanning from business scenarios through architecture requirements to deliverables, work packages, and projects. | Data | Enterprise Architect | Must | End-to-end traceability is a TOGAF and audit requirement |
| REQ-EA-140 | Requirements Gap Identification | The suite SHALL flag requirements that have no linked architecture response (orphaned requirements) or no linked business driver (gold-plating requirements) for architect review. | AI/ML | Enterprise Architect | Must | Completeness and relevance assurance |
| REQ-EA-141 | Requirements Change Control | The suite SHALL enforce a change control process for approved requirements, requiring a linked Change Request before a requirement can be modified or retired. | Governance | Enterprise Architect | Must | Prevents requirement drift undermining the architecture |
| REQ-EA-142 | Requirements AI Extraction | The suite SHALL extract architecture requirements from ingested artifacts (RFPs, project charters, compliance mandates, interview transcripts) and stage them for architect review and approval before entry into the repository. | AI/ML | Enterprise Architect | Must | Core ingestion → requirements loop |
| REQ-EA-143 | Requirements Export | The suite SHALL export the Requirements Repository to Word, Excel, and ReqIF format for use with downstream requirements management tools. | Integration | Enterprise Architect | Should | Interoperability with Jama Connect, DOORS, Azure DevOps |
| REQ-EA-144 | Requirements Completeness Score | The suite SHALL score requirements completeness per ADM phase using configurable heuristics and surface a phase-readiness indicator to architects. | Reporting | Enterprise Architect | Should | Helps architects assess phase completion confidence |

---

## 11. ADM Techniques

### 11.1 Gap Analysis

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-145 | Gap Analysis Engine | The suite SHALL implement a gap analysis engine that compares baseline and target architecture elements per domain, classifying gaps as Add, Remove, Retain, or Modify with severity scoring. | Functional | Enterprise Architect | Must | Core ADM technique — used in Phases B, C, D, E |
| REQ-EA-146 | Gap Analysis Table Generation | The suite SHALL generate formatted Gap Analysis Tables per domain from engine output, suitable for inclusion in Architecture Definition Documents. | Functional | Enterprise Architect | Must | Standard TOGAF deliverable format |
| REQ-EA-147 | Cross-Domain Gap Consolidation | The suite SHALL consolidate gaps across all four architecture domains into a single consolidated gap view with cross-domain impact flags. | Functional | Enterprise Architect | Must | Phase E requires cross-domain view |

### 11.2 Interoperability Analysis

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-148 | Interoperability Requirements Matrix | The suite SHALL generate an Interoperability Requirements Matrix identifying integration points between applications, data entities, and systems, with interface type (API, file, messaging, event) and protocol annotations. | Functional | Enterprise Architect | Must | TOGAF interoperability analysis technique §27 |
| REQ-EA-149 | Interface Catalog | The suite SHALL maintain an Interface Catalog with fields for interface ID, producer, consumer, protocol, data entities exchanged, SLA, and ownership. | Data | Enterprise Architect | Must | Phase C Application Architecture artifact |
| REQ-EA-150 | Interoperability Gap Detection | The suite SHALL identify missing or poorly defined interfaces between systems that have data dependencies, flagging them as interoperability gaps. | AI/ML | Enterprise Architect | Should | Prevents integration surprises in Phase E/F |

### 11.3 Risk Management

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-151 | Architecture Risk Register | The suite SHALL maintain an Architecture Risk Register with risk ID, description, category, likelihood (1–5), impact (1–5), risk score, owner, mitigation strategy, and residual risk. | Governance | Enterprise Architect | Must | Risk management is an explicit TOGAF technique §28 |
| REQ-EA-152 | Risk Heat Map | The suite SHALL render a 5×5 Risk Heat Map from the Architecture Risk Register with color-coded cells (Red/Amber/Green) and drillable risk entries. | Reporting | Enterprise Architect | Must | Standard board-level risk communication tool |
| REQ-EA-153 | AI Risk Identification | The suite SHALL analyze architecture decisions, gap analysis results, and work packages using AI to propose candidate architecture risks for review. | AI/ML | Enterprise Architect | Should | Catches risks architects might overlook |
| REQ-EA-154 | Risk-to-Mitigation Traceability | The suite SHALL maintain bi-directional links between risks and their mitigation actions in the work package catalog. | Data | Enterprise Architect | Must | Ensures risks are actioned |
| REQ-EA-155 | Initial Risk Assessment | The suite SHALL support the TOGAF Initial Risk Assessment (Classification + Assessment) workflow before ADM iteration proceeds. | Governance | Enterprise Architect | Must | TOGAF risk technique Phase A entry |

### 11.4 Capability-Based Planning

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-156 | Capability Increment Planning | The suite SHALL support planning of capability increments (e.g., Increment 1: basic, Increment 2: enhanced) with linkage to work packages and Transition Architectures. | Functional | Enterprise Architect | Must | TOGAF §30 Capability-Based Planning technique |
| REQ-EA-157 | Capability Roadmap View | The suite SHALL render a Capability Roadmap showing capability maturity progression over time across planned increments. | Functional | Enterprise Architect | Must | Communicates capability evolution to business stakeholders |
| REQ-EA-158 | Capability Investment Portfolio | The suite SHALL aggregate work package costs by target capability to generate a Capability Investment Portfolio view. | Reporting | Enterprise Architect | Should | Links investment to capability outcomes |

---

## 12. Architecture Content Framework

### 12.1 Content Metamodel

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-159 | TOGAF Content Metamodel Support | The suite SHALL implement the TOGAF 10 Architecture Content Metamodel entity types (Actor, Role, Business Service, Function, Process, Application Component, Data Entity, Technology Component, etc.) as first-class objects in the repository. | Data | Enterprise Architect | Must | ACF content metamodel is the data backbone of TOGAF conformance |
| REQ-EA-160 | Metamodel Extension | The suite SHALL allow architects to extend the content metamodel with custom entity types and relationship types without modifying the core metamodel schema. | Data | Enterprise Architect | Should | Enterprises routinely extend TOGAF for domain specifics |
| REQ-EA-161 | Metamodel Conformance Checker | The suite SHALL validate repository content against the TOGAF content metamodel rules and report conformance violations with remediation suggestions. | Governance | Enterprise Architect | Must | Ensures TOGAF-conformant repository content |
| REQ-EA-162 | Relationship Type Catalog | The suite SHALL maintain a complete catalog of TOGAF relationship types (Association, Aggregation, Composition, Realization, Influence, Triggering, etc.) used in all cross-domain linking. | Data | Enterprise Architect | Must | Precise relationship semantics are required for traceability |

### 12.2 Catalogs

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-163 | Catalog Generation — Phase B | The suite SHALL generate all Phase B standard catalogs: Organization/Actor, Driver/Goal/Objective, Role, Business Service/Function, Location, Process/Event/Control/Product. | Functional | Enterprise Architect | Must | TOGAF ACF Phase B catalog list |
| REQ-EA-164 | Catalog Generation — Phase C | The suite SHALL generate all Phase C standard catalogs: Data Entity/Data Component (Data), Application Portfolio, Interface (Application). | Functional | Enterprise Architect | Must | TOGAF ACF Phase C catalog list |
| REQ-EA-165 | Catalog Generation — Phase D | The suite SHALL generate all Phase D standard catalogs: Technology Standards, Technology Portfolio. | Functional | Enterprise Architect | Must | TOGAF ACF Phase D catalog list |
| REQ-EA-166 | Catalog Refresh Automation | The suite SHALL support scheduled refresh of catalogs from connected source systems (CMDB, ITSM, PPM) with configurable reconciliation rules for conflict resolution. | Integration | Enterprise Architect | Should | Keeps catalogs current without manual maintenance |
| REQ-EA-167 | Universal Catalog Search | The suite SHALL provide a universal search interface across all catalogs supporting full-text, filter, and faceted search returning results grouped by entity type. | UX | Enterprise Architect | Must | Discoverability across large repositories |

### 12.3 Matrices

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-168 | Stakeholder/Concern Matrix | The suite SHALL generate the TOGAF Stakeholder/Concern Matrix from the Stakeholder Register and Concern Catalog. | Functional | Enterprise Architect | Must | Phase A ACF matrix |
| REQ-EA-169 | Business Interaction Matrix | The suite SHALL generate the TOGAF Business Interaction Matrix mapping org units, business functions, and their interactions. | Functional | Enterprise Architect | Must | Phase B ACF matrix |
| REQ-EA-170 | Data Entity/Business Function Matrix | The suite SHALL generate the Data Entity/Business Function Matrix showing data entity CRUD by business function. | Functional | Enterprise Architect | Must | Phase C (Data) ACF matrix |
| REQ-EA-171 | Application/Function Matrix | The suite SHALL generate the Application/Function Matrix mapping applications to the business functions they support. | Functional | Enterprise Architect | Must | Phase C (Application) ACF matrix |
| REQ-EA-172 | Application/Organization Matrix | The suite SHALL generate the Application/Organization Matrix mapping applications to owning or consuming organization units. | Functional | Enterprise Architect | Must | Phase C (Application) ACF matrix |
| REQ-EA-173 | Application Interaction Matrix | The suite SHALL generate the Application Interaction Matrix showing communication pathways between applications. | Functional | Enterprise Architect | Must | Phase C (Application) ACF matrix |
| REQ-EA-174 | Application/Technology Matrix | The suite SHALL generate the Application/Technology Matrix mapping each application to its underlying technology components. | Functional | Enterprise Architect | Must | Phase D ACF matrix |

### 12.4 Diagrams (intent only — notation in V03)

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-175 | Phase B Diagram Set | The suite SHALL generate the full TOGAF Phase B diagram set (Business Footprint, Business Service/Information, Functional Decomposition, Goal/Objective/Service, Business Use Case, Organization Decomposition, Process Flow, Event Diagram) via AI-assist from catalog and matrix data. | AI/ML | Enterprise Architect | Must | ACF Phase B diagrams — notation per V03 |
| REQ-EA-176 | Phase C Diagram Set | The suite SHALL generate the full TOGAF Phase C diagram set for Data (Conceptual Data, Logical Data, Data Dissemination, Data Security, Data Migration, Data Lifecycle) and Application (Application Communication, Application/User Location, Application Use Case, Enterprise Manageability, Process/Application Realization, Software Engineering, Application Migration, Software Distribution). | AI/ML | Enterprise Architect | Must | ACF Phase C diagrams |
| REQ-EA-177 | Phase D Diagram Set | The suite SHALL generate the full TOGAF Phase D diagram set (Environments/Locations, Platform Decomposition, Processing, Networked Computing/Hardware, Communications Engineering). | AI/ML | Enterprise Architect | Must | ACF Phase D diagrams |
| REQ-EA-178 | Phase E/F Diagram Set | The suite SHALL generate the Phase E/F diagram set (Project Context, Benefits Diagram) and Phase G/H diagrams from repository data. | AI/ML | Enterprise Architect | Must | ACF Phase E–H diagrams |
| REQ-EA-179 | Diagram Auto-Layout | The suite SHALL apply automatic graph layout algorithms (hierarchical, force-directed, orthogonal) to generated diagrams with architect override capability. | UX | Enterprise Architect | Must | Generated diagrams must be readable without manual rearrangement |
| REQ-EA-180 | Diagram Export | The suite SHALL export all diagrams to PNG, SVG, PDF, PowerPoint, and ArchiMate Exchange Format. | Functional | Enterprise Architect | Must | Export detail in V10 |

---

## 13. Enterprise Continuum & Architecture Repository

### 13.1 Enterprise Continuum

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-181 | Enterprise Continuum Navigation | The suite SHALL provide a navigable Enterprise Continuum view showing Foundation, Common Systems, Industry, and Organization-Specific architecture levels as a hierarchy. | Functional | Enterprise Architect | Must | TOGAF §39 Enterprise Continuum — critical conceptual structure |
| REQ-EA-182 | Foundation Architecture Library | The suite SHALL include a Foundation Architecture library containing TOGAF Foundation Architecture patterns (TRM, III-RM) usable as starting-point building blocks. | Data | Enterprise Architect | Must | TOGAF Technical Reference Model and III-RM seeded content |
| REQ-EA-183 | Industry Architecture Reference Models | The suite SHALL include reference architecture content for at least five industry verticals (FSI, Healthcare, Retail, Telecommunications, Public Sector) as Common Systems / Industry Architecture entries in the Continuum. | Data | Enterprise Architect | Must | Differentiator — speeds architecture development vs. blank-sheet |
| REQ-EA-184 | Organization-Specific Architecture | The suite SHALL support promotion of architecture artifacts from Foundation/Common/Industry levels to Organization-Specific level with version control and approved-by metadata. | Functional | Enterprise Architect | Must | Organizations specialize reusable patterns into their own context |
| REQ-EA-185 | Continuum Search & Browse | The suite SHALL allow architects to browse and search the Enterprise Continuum by level, domain, industry, and keyword to discover reusable building blocks. | UX | Enterprise Architect | Must | Discoverability drives reuse |

### 13.2 Architecture Repository

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-186 | Architecture Landscape | The suite SHALL maintain the Architecture Landscape with Strategic, Segment, and Capability architecture views supporting drill-down from strategic intent to capability detail. | Data | Enterprise Architect | Must | TOGAF Architecture Repository §37.2.1 |
| REQ-EA-187 | Architecture Building Blocks (ABBs) | The suite SHALL maintain an ABB library with fields for name, type, description, standard compliance, domain, and reuse scope, supporting creation and discovery of vendor-neutral building blocks. | Data | Enterprise Architect | Must | TOGAF §37.2.2 — ABBs |
| REQ-EA-188 | Solution Building Blocks (SBBs) | The suite SHALL maintain an SBB library linked to ABBs, capturing vendor-specific product instances that realize each ABB. | Data | Enterprise Architect | Must | TOGAF §37.2.3 — SBBs |
| REQ-EA-189 | ABB-to-SBB Traceability | The suite SHALL maintain and visualize bi-directional traceability between ABBs and their SBB realizations. | Data | Enterprise Architect | Must | Supports architecture compliance assessment |
| REQ-EA-190 | Reference Library | The suite SHALL maintain a Reference Library for external standards, white papers, vendor documentation, and industry models with metadata tagging and full-text search. | Data | Enterprise Architect | Must | TOGAF §37.2.5 Reference Library |
| REQ-EA-191 | Governance Log (Repository) | The suite SHALL maintain a Governance Log within the Architecture Repository as a permanent record of all governance decisions, signed-off contracts, and compliance outcomes, with immutable entries. | Governance | Architecture Practice Lead | Must | TOGAF §37.2.6 — Governance Log |
| REQ-EA-192 | Architecture Instance Registry | The suite SHALL maintain an Architecture Instance Registry linking each deployed system or solution to its governing Architecture Definition, Architecture Contract, and compliance status. | Data | Architecture Practice Lead | Must | Enables live portfolio compliance tracking |
| REQ-EA-193 | Repository Graph View | The suite SHALL render the Architecture Repository as a navigable property graph where nodes are entities and edges are relationships, supporting visual exploration and path queries. | UX | Enterprise Architect | Should | Graph visualization is the most powerful way to explore cross-domain traceability; differentiator vs. MEGA HOPEX |
| REQ-EA-194 | Repository API | The suite SHALL expose a REST/GraphQL API for the Architecture Repository supporting CRUD operations on all entity types, secured by OAuth 2.0/OIDC. | Integration | Enterprise Architect | Must | Enables integration with external tools and event-driven updates |
| REQ-EA-195 | Repository Import/Export | The suite SHALL support bulk import and export of repository content in TOGAF-conformant XML, JSON-LD, and ArchiMate Exchange Format. | Integration | Enterprise Architect | Must | Tool interoperability and backup |

---

## 14. Architecture Capability Framework

### 14.1 Architecture Governance

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-196 | Architecture Board Support | The suite SHALL provide a dedicated Architecture Board workspace with agenda management, decision tracking, minutes generation, and action item follow-up. | Functional | Architecture Practice Lead | Must | Architecture Capability Framework §44 |
| REQ-EA-197 | Architecture Contract Lifecycle | The suite SHALL support the full lifecycle of Architecture Contracts: draft → negotiation → sign-off → active monitoring → closure, with configurable approval roles. | Governance | Architecture Practice Lead | Must | ACF Architecture Contracts §44.4 |
| REQ-EA-198 | Architecture Review Meeting Support | The suite SHALL provide a meeting support view surfacing all relevant architecture artifacts, open issues, and compliance data for Architecture Review meetings with single-click access. | UX | Architecture Practice Lead | Should | Reduces meeting preparation overhead |
| REQ-EA-199 | Architecture Dispensation Register | The suite SHALL maintain a Dispensation Register for agreed deviations from the target architecture, with owner, justification, review date, and linkage to the Governance Log. | Governance | Architecture Practice Lead | Must | Formal record of permitted non-conformance |
| REQ-EA-200 | Architecture Escalation Path | The suite SHALL enforce a configured escalation path for unresolved architecture issues, automatically routing to the next governance level after a configurable timeout. | Governance | Architecture Practice Lead | Must | Prevents governance deadlock |

### 14.2 Compliance Reviews

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-201 | Compliance Checklist Builder | The suite SHALL allow the construction of compliance checklists from Architecture Principles, Standards Information Base entries, and Architecture Contract clauses, tailored per project type. | Functional | Architecture Practice Lead | Must | Checklists are the primary Phase G review instrument |
| REQ-EA-202 | Automated Pre-Compliance Check | The suite SHALL perform an automated pre-compliance check against the Architecture Repository on demand for any project, surfacing potential non-conformances before a formal review. | AI/ML | Enterprise Architect | Should | Shifts compliance left; reduces formal review failures |
| REQ-EA-203 | Compliance Review History | The suite SHALL retain a full history of all compliance reviews per project with versions, scores, exceptions, and decisions. | Governance | Architecture Practice Lead | Must | Required for audit and trend analysis |
| REQ-EA-204 | Multi-Domain Compliance Scoring | The suite SHALL score compliance across Business, Data, Application, and Technology domains independently and produce a combined weighted compliance score. | Reporting | Architecture Practice Lead | Should | Domain-level scores identify which domain is weakest |

### 14.3 Maturity Models

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-205 | EA Maturity Assessment | The suite SHALL support assessment of the Architecture organization's maturity against a configurable maturity model (default: TOGAF Capability Maturity levels 0–5) across dimensions of governance, people, process, tools, and content. | Functional | Architecture Practice Lead | Must | ACF Architecture Maturity Models §43 |
| REQ-EA-206 | Maturity Trend Reporting | The suite SHALL track maturity assessment scores over time and render trend charts showing maturity progression per dimension and overall. | Reporting | Architecture Practice Lead | Must | Demonstrates practice improvement to leadership |
| REQ-EA-207 | Maturity Benchmark Comparison | The suite SHALL benchmark an organization's maturity scores against industry-average anonymized benchmarks where available. | Reporting | Architecture Practice Lead | Could | Unique differentiator — not present in BiZZdesign or LeanIX |
| REQ-EA-208 | Improvement Roadmap Generation | The suite SHALL generate an AI-assisted EA Practice Improvement Roadmap from maturity assessment gaps, with prioritized actions and estimated effort. | AI/ML | Architecture Practice Lead | Should | Closes the maturity improvement loop |

### 14.4 Architecture Skills Framework

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-209 | Skills Catalog | The suite SHALL maintain an Architecture Skills Catalog aligned to the TOGAF Architecture Skills Framework with skills mapped to proficiency levels (Awareness, Practitioner, Expert). | Data | Architecture Practice Lead | Should | ACF Architecture Skills Framework §47 |
| REQ-EA-210 | Team Skills Assessment | The suite SHALL support self-assessment and manager-assessment of architect team members against the Skills Catalog, producing a team skills heat map. | Functional | Architecture Practice Lead | Could | Enables capability gap identification in the EA team |
| REQ-EA-211 | Training Recommendation | The suite SHALL recommend training resources (linked to TOGAF courses, The Open Group certifications, vendor training) for skills gaps identified in the team assessment. | AI/ML | Architecture Practice Lead | Could | Supports team development |

---

## 15. Architecture Deliverables — Per-Phase Production

### 15.1 Architecture Vision Document (Phase A)

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-212 | Architecture Vision Document Auto-Assembly | The suite SHALL auto-assemble a complete Architecture Vision document from Phase A repository inputs, including: executive summary, business context, scope, constraints, Architecture Vision narrative, Solution Concept Diagram, and stakeholder sign-off page. | Functional | Enterprise Architect | Must | Required TOGAF Phase A deliverable |
| REQ-EA-213 | Vision Document Template Variants | The suite SHALL provide at least three Vision Document templates (Executive Brief, Full Formal, Agile one-pager) selectable at assembly time. | UX | Enterprise Architect | Should | Different stakeholders need different document depths |

### 15.2 Architecture Definition Document

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-214 | ADD Full Assembly | The suite SHALL auto-assemble a single Architecture Definition Document incorporating all domain sections (B, C-Data, C-App, D) with consistent formatting, table of contents, cross-references, and diagrams embedded. | Functional | Enterprise Architect | Must | ADD is the primary TOGAF architecture documentation output |
| REQ-EA-215 | ADD Incremental Update | The suite SHALL support incremental update of the ADD when a sub-section changes, regenerating only affected sections and version-stamping the document. | Functional | Enterprise Architect | Must | Large ADDs must not require full regeneration on every change |
| REQ-EA-216 | ADD Review Annotation | The suite SHALL allow stakeholder review comments to be added to the ADD in-tool with threaded discussion, resolved/unresolved tracking, and change request generation. | UX | Enterprise Architect | Must | Review process must be native to avoid losing comments in email chains |

### 15.3 Architecture Requirements Specification

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-217 | Requirements Spec Auto-Assembly | The suite SHALL auto-assemble an Architecture Requirements Specification document from the Requirements Repository, organized by domain and prioritized by MoSCoW. | Functional | Enterprise Architect | Must | Required TOGAF output across multiple phases |
| REQ-EA-218 | Requirements Spec Versioning | The suite SHALL version each Requirements Specification snapshot at defined phase gate milestones with rollback capability. | Data | Enterprise Architect | Must | Requirements evolve; baselines are needed for governance |

### 15.4 Architecture Roadmap

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-219 | Architecture Roadmap Document | The suite SHALL produce an Architecture Roadmap document from work packages, migration plan, and transition architectures, with timeline charts, investment summary, and benefits tracking. | Functional | Enterprise Architect | Must | Required TOGAF output; detailed mechanics in V09 |
| REQ-EA-220 | Roadmap Executive Summary | The suite SHALL generate a one-page Executive Roadmap Summary from the detailed roadmap, suitable for board-level communication. | Functional | Enterprise Architect | Must | C-suite demands concise visual roadmaps |

### 15.5 Implementation and Migration Plan (IGSM)

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-221 | Implementation Governance and Support Model | The suite SHALL produce an Implementation Governance and Support Model (IGSM) document specifying governance arrangements, support structures, and roles applicable during implementation, assembled from Governance Board configuration and Phase G inputs. | Functional | Enterprise Architect | Must | Required TOGAF deliverable spanning Phase F/G |
| REQ-EA-222 | Migration Plan Document | The suite SHALL auto-assemble a Migration Plan document from Phase F repository inputs including work packages, risk register, cost-benefit models, and resource forecast. | Functional | Enterprise Architect | Must | Required TOGAF Phase F deliverable |

---

## 16. Non-Functional Requirements

### 16.1 Performance

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-223 | Repository Query Performance | The suite SHALL return catalog, matrix, and list views with up to 10,000 repository entities within 2 seconds under standard load. | NFR | Enterprise Architect | Must | Large enterprises have very large repositories; performance is table stakes |
| REQ-EA-224 | AI Generation SLA | The suite SHALL complete AI-driven deliverable generation tasks (Vision, ADD sections, diagrams) within 120 seconds for inputs up to 500 pages of ingested content. | NFR | Enterprise Architect | Must | User patience threshold for AI generation |
| REQ-EA-225 | Concurrent User Support | The suite SHALL support at least 200 concurrent active users without degradation in page load or generation latency. | NFR | Architecture Practice Lead | Must | Enterprise accounts span large teams |
| REQ-EA-226 | Diagram Render Performance | The suite SHALL render diagrams with up to 200 nodes within 5 seconds in the browser. | NFR | Enterprise Architect | Must | Complex EA diagrams have many elements |

### 16.2 Scalability

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-227 | Repository Scalability | The suite SHALL scale the Architecture Repository to at least 1 million entity instances and 5 million relationships without architectural change. | NFR | Architecture Practice Lead | Must | Global enterprise repositories can be very large |
| REQ-EA-228 | Multi-Tenancy | The suite SHALL enforce strict multi-tenant data isolation at the database level, ensuring no cross-tenant data leakage under any circumstance. | Security | Architecture Practice Lead | Must | SaaS deployment; enterprise security requirement |

### 16.3 Availability & Resilience

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-229 | Availability SLA | The suite SHALL maintain 99.9% monthly uptime SLA for all production services excluding planned maintenance windows. | NFR | Architecture Practice Lead | Must | Enterprise tools need high availability |
| REQ-EA-230 | Disaster Recovery RTO/RPO | The suite SHALL achieve a Recovery Time Objective (RTO) of ≤ 4 hours and Recovery Point Objective (RPO) of ≤ 1 hour for all repository data. | NFR | Architecture Practice Lead | Must | Business continuity requirement |

### 16.4 Security

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-231 | Authentication — SSO | The suite SHALL support Single Sign-On (SSO) via SAML 2.0 and OIDC, integrating with enterprise identity providers (Entra ID, Okta, Ping Identity). | Security | Architecture Practice Lead | Must | Enterprise authentication standard |
| REQ-EA-232 | MFA Enforcement | The suite SHALL enforce Multi-Factor Authentication for all users accessing the governance and repository functions. | Security | Architecture Practice Lead | Must | Protects sensitive architecture IP |
| REQ-EA-233 | Data Encryption at Rest and Transit | The suite SHALL encrypt all data at rest (AES-256) and in transit (TLS 1.3), with tenant-managed key options available. | Security | Architecture Practice Lead | Must | Data protection baseline |
| REQ-EA-234 | Audit Log | The suite SHALL maintain an immutable audit log of all user actions on repository objects (create, update, delete, export, share) with timestamp, user ID, and before/after state. | Security | Architecture Practice Lead | Must | SOC 2 Type II compliance requirement |
| REQ-EA-235 | RBAC Fine-Grained | The suite SHALL support fine-grained RBAC with permissions configurable at entity type, domain, and individual object level. | Security | Architecture Practice Lead | Must | Different architects must see different repository scopes |

### 16.5 Usability

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-236 | Guided ADM Workflow | The suite SHALL provide a guided ADM Workflow UI that walks architects through required activities and deliverables phase by phase, with progress indicators and gating checks. | UX | Enterprise Architect | Must | Reduces learning curve for new TOGAF practitioners |
| REQ-EA-237 | Contextual Help & TOGAF References | The suite SHALL provide in-context help links to the relevant TOGAF 10 standard sections for every ADM activity, deliverable, and technique. | UX | Enterprise Architect | Should | Reduces dependency on external TOGAF documentation |
| REQ-EA-238 | Accessibility — WCAG 2.1 AA | The suite SHALL conform to WCAG 2.1 Level AA accessibility standards across all UI surfaces. | UX | Enterprise Architect | Must | Legal requirement in many jurisdictions |
| REQ-EA-239 | Responsive UI | The suite SHALL render correctly on desktop screens (1280×768 and above) and support read-only viewing on tablet form factors (768×1024 and above). | UX | Enterprise Architect | Should | Architects access tools on multiple devices |

### 16.6 Integration

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-240 | Jira Integration | The suite SHALL integrate with Jira Cloud and Jira Data Center to synchronize work packages, projects, and issues with architecture records. | Integration | Enterprise Architect | Must | Jira is the dominant project tracking tool in enterprise |
| REQ-EA-241 | Azure DevOps Integration | The suite SHALL integrate with Azure DevOps to synchronize work items, pipelines, and repositories with architecture artifacts. | Integration | Enterprise Architect | Must | ADO is the primary tool in Microsoft-stack enterprises |
| REQ-EA-242 | Confluence Integration | The suite SHALL publish architecture artifacts and deliverables to Confluence pages with configurable page templates and auto-update on content change. | Integration | Enterprise Architect | Should | Confluence is the dominant enterprise wiki |
| REQ-EA-243 | ServiceNow Integration | The suite SHALL integrate with ServiceNow to read CMDB configuration items and application records into the Application Portfolio Catalog. | Integration | Enterprise Architect | Should | ServiceNow CMDB is the authoritative source in many enterprises |
| REQ-EA-244 | Webhook Support | The suite SHALL support configurable outbound webhooks triggered by repository events (new entity, status change, governance decision) for integration with custom workflows. | Integration | Enterprise Architect | Should | Enables lightweight integration with arbitrary enterprise tools |

---

## 17. Governance & Reporting

### 17.1 EA Governance Reporting

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-245 | EA Practice Report | The suite SHALL generate a monthly EA Practice Report summarizing: active ADM cycles, deliverables produced, governance decisions, compliance scores, and maturity trend. | Reporting | Architecture Practice Lead | Must | Leadership visibility of EA practice value |
| REQ-EA-246 | Architecture Debt Report | The suite SHALL generate an Architecture Debt Report quantifying technical debt, lifecycle gaps, unresolved risks, and deferred work packages, with trend-over-time charts. | Reporting | Enterprise Architect | Must | Architecture debt is a key board-level concern |
| REQ-EA-247 | Stakeholder Communication Report | The suite SHALL generate a Stakeholder Communication Report listing planned and completed communications, open concerns, and satisfaction indicators per stakeholder group. | Reporting | Enterprise Architect | Should | Governance of stakeholder engagement |
| REQ-EA-248 | Custom Report Builder | The suite SHALL provide a report builder enabling architects to define custom reports from any combination of repository entities, metrics, and filters, with scheduled delivery via email. | Reporting | Architecture Practice Lead | Should | Enterprise reporting needs vary widely |
| REQ-EA-249 | Executive Dashboard | The suite SHALL provide a configurable executive dashboard with KPIs drawn from all repository domains: capability coverage, application health, technology currency, compliance rate, and roadmap progress. | Reporting | Architecture Practice Lead | Must | C-suite and board communication requires a single-pane view |

---

## 18. AI Assist Intent (cross-cutting — mechanics deferred to V06)

| ID | Title | Description | Category | Persona | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-EA-250 | AI Feedback Loop | The suite SHALL allow architects to accept, reject, or correct every AI-generated suggestion, with rejections fed back to improve future suggestions within the tenant's AI model context. | AI/ML | Enterprise Architect | Must | Human-in-the-loop is non-negotiable for governance-grade content |
| REQ-EA-251 | AI Generation Transparency | The suite SHALL mark every AI-generated artifact section with a visual indicator and expose the source evidence (ingested document, page, passage) that supported the generation. | AI/ML | Enterprise Architect | Must | Architects must be able to audit AI reasoning |
| REQ-EA-252 | AI Confidence Scoring | The suite SHALL display a confidence score for each AI-generated recommendation or extraction, calibrated to reflect model certainty. | AI/ML | Enterprise Architect | Should | Helps architects prioritize review effort |
| REQ-EA-253 | Multi-Model AI Routing | The suite SHALL route AI tasks to appropriate model tiers (large model for complex generation, small model for classification/extraction) per a configurable cost/quality policy. | AI/ML | Architecture Practice Lead | Should | Cost management of AI inference at scale |
| REQ-EA-254 | AI Audit Trail | The suite SHALL log every AI inference request — input, model used, output, and user action — in a tamper-evident AI Audit Trail retained for 90 days minimum. | AI/ML | Architecture Practice Lead | Must | AI governance and compliance |

---

## Appendix A — Requirement Count Summary

| Capability Area | Section | REQ Count |
|---|---|---|
| Preliminary Phase | 1.1–1.3 | 22 |
| Phase A — Architecture Vision | 2.1–2.3 | 18 |
| Phase B — Business Architecture | 3.1–3.5 | 27 |
| Phase C — Information Systems | 4.1–4.2 | 24 |
| Phase D — Technology Architecture | 5.1–5.2 | 12 |
| Phase E — Opportunities & Solutions | 6 | 9 |
| Phase F — Migration Planning | 7 | 7 |
| Phase G — Implementation Governance | 8 | 9 |
| Phase H — Architecture Change Mgmt | 9 | 7 |
| Requirements Management | 10 | 9 |
| ADM Techniques | 11.1–11.4 | 14 |
| Architecture Content Framework | 12.1–12.4 | 22 |
| Enterprise Continuum & Repository | 13.1–13.2 | 15 |
| Architecture Capability Framework | 14.1–14.4 | 16 |
| Per-Phase Deliverables | 15.1–15.5 | 11 |
| Non-Functional Requirements | 16.1–16.6 | 22 |
| Governance & Reporting | 17 | 5 |
| AI Assist Intent | 18 | 5 |
| **TOTAL** | | **254** |

---

## Appendix B — Standards & Competitor References

| Standard / Product | Relevance |
|---|---|
| TOGAF 10 (The Open Group, 2022) | Primary framework; all ADM phase, technique, deliverable, and repository requirements sourced directly |
| TOGAF ACF Phase B–D Catalog/Matrix/Diagram lists | Drives §12.2–12.4 exhaustive list requirements |
| TOGAF Architecture Capability Framework §43–47 | Drives §14 maturity, contracts, governance board, skills |
| TOGAF Requirements Management (§17) | Drives §10 |
| TOGAF ADM Techniques (§24–30) | Drives §11 |
| LeanIX Enterprise Architecture Management | Informs Application Heat Map, SaaS enrichment, Integration catalog automation gap |
| BiZZdesign Enterprise Studio | Informs impact analysis depth, ArchiMate + TOGAF combined tooling expectations |
| MEGA HOPEX | Informs repository graph view, metamodel extensibility, multi-framework depth |
| Avolution ABACUS | Informs capability heat mapping, dashboard richness |
| Software AG Alfabet | Informs lifecycle tracking, technology radar, cost attribution |

---

## Appendix C — Boundary Notes & Assumptions

1. **ArchiMate notation** for all diagram requirements (REQ-EA-175 to 180, REQ-EA-098) is intentional by reference only — full notation specification is in Volume 03.
2. **AI engine internals** (model selection, vector storage, RAG pipeline, embedding strategy) are deferred to Volume 06; this volume specifies the AI behavior observable by the architect persona.
3. **Portfolio management mechanics** (roadmap timeline engine, financial portfolio views, code scanning for sunset) are deferred to Volume 09; this volume covers roadmap as a Phase E/F output artifact only.
4. **Document rendering and export** (Word/PDF/PPT templates, mail-merge, brand tokens) are deferred to Volume 10.
5. **Ingestion pipeline** (parsing Word, PDF, Excel, Visio, email, Teams, code) is deferred to Volume 07.
6. **Knowledge Hub connectors** (Confluence, SharePoint, Jira, ADO live sync) are specified in Volume 08; this volume states integration intent only.
7. All requirement IDs in this volume use the `REQ-EA-NNN` prefix; sub-volumes for specialized domains use sub-prefixed series per their respective prefix definition.
8. "AI-assist" in this volume presupposes the AI engine specified in Volume 06; no specific LLM or vector store is mandated here.
9. Competitor analysis (LeanIX, BiZZdesign, MEGA HOPEX, Avolution, Alfabet) was used to calibrate feature completeness expectations, not to copy design — all requirements are independently justified against TOGAF 10.
