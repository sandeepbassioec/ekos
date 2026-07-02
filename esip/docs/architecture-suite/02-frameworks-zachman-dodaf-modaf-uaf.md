# ESIP Architecture Automation Suite — Volume 02: Other EA Frameworks
## Zachman · DoDAF 2.02 · MODAF/NAF · UAF · FEAF · Cross-Framework

---

**Document ID:** ESIP-REQ-VOL02  
**Version:** 1.0  
**Date:** 2026-06-23  
**Prefix:** `REQ-FWK`  
**Status:** DRAFT — Authoritative Requirements Baseline  

---

## Scope

This volume defines requirements for ESIP's support of EA frameworks beyond TOGAF/ADM (covered in Volume 01) and ArchiMate diagramming (Volume 03). It governs:

- **Zachman Framework** — 6×6 ontology cell modeling, population, linking, and consistency
- **DoDAF 2.02** — All viewpoints and their models, DM2 alignment, fit-for-purpose authoring/generation
- **MODAF / NAF v4** — Strategic, Operational, Systems, Acquisition, Service, Technical, All-Views; NAF grid
- **UAF** — Unified Architecture Framework domains × aspects, UAF↔SysML integration
- **FEAF Reference Models** — PRM, BRM, DRM, ARM, IRM
- **Cross-Framework** — Selection wizard, viewpoint libraries, model interoperability, format export

**Out of scope for this volume:**
- ArchiMate metamodel and diagram engine engine → Volume 03
- TOGAF ADM phase-by-phase requirements → Volume 01
- Deliverable document templates and generation → Volume 10
- AI model orchestration and generation pipelines → Volume 06

---

## Personas

| ID | Persona | Description |
|----|---------|-------------|
| P-EA | Enterprise Architect | Defines and governs the overall architecture program |
| P-DA | Defense/Gov Architect | Works with DoDAF/MODAF/NAF for government/defense programs |
| P-SA | Solutions Architect | Designs solutions aligned to enterprise reference frameworks |
| P-BA | Business Analyst | Captures requirements, processes, and motivations |
| P-PM | Program/Project Manager | Oversees delivery roadmaps and acquisition programs |
| P-SE | Systems Engineer | Authors SV/SysML/UAF models for systems design |
| P-CIO | CIO/Executive Sponsor | Consumes strategic views; FEAF compliance oversight |
| P-AUD | Auditor/Compliance Officer | Validates framework completeness and regulatory fit |
| P-AI | AI/Automation Engineer | Configures AI extraction and auto-generation pipelines |

---

## Requirement Categories

`Functional` | `NFR` | `Integration` | `Data` | `AI/ML` | `UX` | `Security` | `Governance` | `Reporting`

---

## 1. Zachman Framework

### 1.1 Zachman Ontology Foundation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWK-001 | Zachman 6×6 Cell Registry | The system SHALL maintain a formal registry of all 36 Zachman cells, indexed by Perspective (Row 1–6) and Interrogative (Column 1–6), with canonical names, descriptions, and intended artifact types per cell. | Data | P-EA, P-AI | Must | Zachman Framework v3 ontology; prevents misclassification of artifacts |
| REQ-FWK-002 | Perspective Definitions | The system SHALL implement the six Zachman perspectives as enumerated types: Row 1 Executive/Contextual (Scope), Row 2 Business Management/Conceptual (Business Model), Row 3 Architect/Logical (System Model), Row 4 Engineer/Physical (Technology Model), Row 5 Technician/Detailed Out-of-Context (Detailed Representations), Row 6 Enterprise/Operational (Functioning Enterprise). | Data | P-EA | Must | Zachman 2011 v3 canonical taxonomy |
| REQ-FWK-003 | Interrogative Definitions | The system SHALL implement the six Zachman interrogatives as enumerated types: Col 1 What (Inventory/Data/Things), Col 2 How (Process/Function/Transformation), Col 3 Where (Distribution/Network/Locations), Col 4 Who (Responsibility/People/Agents), Col 5 When (Timing/Events/Cycles), Col 6 Why (Motivation/Reasons/Goals). | Data | P-EA | Must | Zachman Framework canonical interrogatives |
| REQ-FWK-004 | Cell Uniqueness Enforcement | The system SHALL enforce that each artifact or model element is classified into at most one primary Zachman cell, with an optional secondary cross-reference, preventing ontological duplication. | Governance | P-EA, P-AUD | Must | Zachman ontological rule: cells are mutually exclusive |
| REQ-FWK-005 | Cell Naming Convention Enforcement | The system SHALL enforce that artifact names within a cell follow the Zachman-recommended naming primitives (e.g., Row1/Col1 = "List of Things Important to the Business") and flag deviations for review. | Governance | P-EA | Should | Consistency with Zachman canonical model names |

**Acceptance Criteria (REQ-FWK-001 to 005):**
- All 36 cells present in registry with correct row/column coordinates
- No artifact can be saved without a valid cell assignment
- Attempting to assign one artifact to two primary cells returns a validation error

---

### 1.2 Zachman Cell Modeling — What (Data/Inventory)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWK-010 | Cell (R1,C1) — Scope Inventory List | The system SHALL support authoring a Scope Inventory/List of Things artifact (Row 1, What), including a flat list of business entities important to the enterprise, with classification tags and natural-language descriptions. | Functional | P-EA, P-BA | Must | Zachman R1C1: Executive List of Things |
| REQ-FWK-011 | Cell (R2,C1) — Business Entity Model | The system SHALL support authoring a Business Conceptual Data/Entity Model (Row 2, What) as an ERD or class diagram with business-named entities, relationships, and cardinalities, independent of any physical implementation. | Functional | P-EA, P-BA | Must | Zachman R2C1: Business Model of data |
| REQ-FWK-012 | Cell (R3,C1) — Logical Data Model | The system SHALL support authoring a Logical Data Model (Row 3, What) with normalized entities, attributes, primary/foreign keys, and domains, linked to business entities in R2C1. | Functional | P-SA, P-EA | Must | Zachman R3C1: Architect's logical data model |
| REQ-FWK-013 | Cell (R4,C1) — Physical Data Model | The system SHALL support authoring a Physical Data Model (Row 4, What) with table definitions, column types, indexes, constraints, and database-specific constructs, traceable to R3C1 entities. | Functional | P-SA | Must | Zachman R4C1: Engineer's physical design |
| REQ-FWK-014 | Cell (R5,C1) — DDL/Schema Artifact | The system SHALL support ingestion and display of actual DDL scripts, JSON schemas, or Avro/Protobuf definitions (Row 5, What), automatically mapped to R4C1 physical model entries. | Functional | P-SA, P-AI | Should | Zachman R5C1: Technician's actual data definitions |
| REQ-FWK-015 | Cell (R6,C1) — Operational Data | The system SHALL link Row 6, What to operational data stores and runtime repositories, referencing the deployed schema artifacts in R5C1. | Functional | P-EA | Could | Zachman R6C1: Functioning enterprise data |

---

### 1.3 Zachman Cell Modeling — How (Function/Process)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWK-020 | Cell (R1,C2) — Business Process List | The system SHALL support authoring a Scope Process List (Row 1, How) as an enumeration of high-level business processes with plain-language descriptions. | Functional | P-EA, P-BA | Must | Zachman R1C2: Executive list of processes |
| REQ-FWK-021 | Cell (R2,C2) — Business Process Model | The system SHALL support authoring a Business Process Model (Row 2, How) using BPMN 2.0 or structured narrative, with swim lanes, actors, and business logic captured at a conceptual level. | Functional | P-BA, P-EA | Must | Zachman R2C2: Business model of processes |
| REQ-FWK-022 | Cell (R3,C2) — Application/Logical Function Model | The system SHALL support authoring a Logical Application Function model (Row 3, How), decomposing business processes into application functions, APIs, and service interfaces without specifying technology. | Functional | P-SA | Must | Zachman R3C2: Architect's system model |
| REQ-FWK-023 | Cell (R4,C2) — Physical Function/Component Design | The system SHALL support authoring a Physical Function/Component design (Row 4, How), specifying deployable components, modules, program units, and their interfaces with technology-specific details. | Functional | P-SA | Must | Zachman R4C2: Engineer's physical design |
| REQ-FWK-024 | Cell (R5,C2) — Executable Code/Program | The system SHALL support ingestion of source code repositories and mapping of code modules/classes to their corresponding R4C2 component entries, with traceability links. | Functional | P-AI, P-SA | Should | Zachman R5C2: Technician's actual code |
| REQ-FWK-025 | Cell (R6,C2) — Operational Functions | The system SHALL maintain a runtime function inventory linked to deployed components (R5C2), reflecting the actual operational state. | Functional | P-EA | Could | Zachman R6C2: Functioning enterprise |

---

### 1.4 Zachman Cell Modeling — Where (Network/Distribution)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWK-030 | Cell (R1,C3) — Location List | The system SHALL support a Scope Location List (Row 1, Where) enumerating business locations, sites, and geographies relevant to the enterprise. | Functional | P-EA, P-BA | Must | Zachman R1C3 |
| REQ-FWK-031 | Cell (R2,C3) — Business Logistics Model | The system SHALL support a Business Logistics/Distribution Network model (Row 2, Where) showing how the business operates across locations, including supply chains and communication paths at conceptual level. | Functional | P-EA, P-BA | Must | Zachman R2C3 |
| REQ-FWK-032 | Cell (R3,C3) — Distributed System Architecture | The system SHALL support a Logical Distributed Architecture model (Row 3, Where) depicting nodes, connectors, and system distribution across logical locations, independent of physical hardware. | Functional | P-SA | Must | Zachman R3C3 |
| REQ-FWK-033 | Cell (R4,C3) — Technology/Network Design | The system SHALL support a Physical Network/Technology design (Row 4, Where) with hardware nodes, network topology, IP addressing schemes, and infrastructure components. | Functional | P-SA | Must | Zachman R4C3 |
| REQ-FWK-034 | Cell (R5,C3) — Network Configurations | The system SHALL support ingestion of infrastructure-as-code (Terraform, CloudFormation, Ansible) and network config files, mapping them to R4C3 physical topology entries. | Functional | P-AI, P-SA | Should | Zachman R5C3 |
| REQ-FWK-035 | Cell (R6,C3) — Live Infrastructure | The system SHALL optionally integrate with cloud provider APIs (AWS/Azure/GCP) to reflect the live network topology in the R6C3 operational view. | Integration | P-EA | Could | Zachman R6C3 |

---

### 1.5 Zachman Cell Modeling — Who (People/Responsibility)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWK-040 | Cell (R1,C4) — Organization List | The system SHALL support a Scope Organization List (Row 1, Who) enumerating organizational units, external parties, and stakeholder groups. | Functional | P-EA, P-BA | Must | Zachman R1C4 |
| REQ-FWK-041 | Cell (R2,C4) — Org/Role Model | The system SHALL support a Business Organization/Role model (Row 2, Who) with org charts, role definitions, RACI matrices, and responsibility assignments at the business level. | Functional | P-EA, P-BA | Must | Zachman R2C4 |
| REQ-FWK-042 | Cell (R3,C4) — Human/System Interface Model | The system SHALL support a Logical Human Interface model (Row 3, Who) mapping users, user groups, and personas to application interfaces and interaction patterns. | Functional | P-SA, P-BA | Must | Zachman R3C4 |
| REQ-FWK-043 | Cell (R4,C4) — Security/Access Design | The system SHALL support a Physical Security and Access Control design (Row 4, Who) specifying authentication mechanisms, authorization rules, user accounts, and identity schemas. | Functional | P-SA | Must | Zachman R4C4 |
| REQ-FWK-044 | Cell (R5,C4) — Actual Users/Accounts | The system SHALL support ingestion of IAM policy exports or LDAP/AD schema dumps and map them to R4C4 entries, reflecting actual deployed identity structure. | Functional | P-AI | Should | Zachman R5C4 |
| REQ-FWK-045 | Cell (R6,C4) — Operational Org | The system SHALL link Row 6, Who to HR systems or directory services to reflect the functioning enterprise's actual people/org state. | Integration | P-EA | Could | Zachman R6C4 |

---

### 1.6 Zachman Cell Modeling — When (Timing/Events)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWK-050 | Cell (R1,C5) — Event List | The system SHALL support a Scope Event/Time List (Row 1, When) enumerating significant business events, cycles, and time constraints. | Functional | P-EA, P-BA | Must | Zachman R1C5 |
| REQ-FWK-051 | Cell (R2,C5) — Business Event Model | The system SHALL support a Business Event/Cycle model (Row 2, When) showing master schedules, business calendars, event sequences, and temporal constraints at the business level. | Functional | P-EA, P-BA | Must | Zachman R2C5 |
| REQ-FWK-052 | Cell (R3,C5) — System Timing/State Model | The system SHALL support a Logical System Timing model (Row 3, When) using state machines, sequence diagrams (UML), or event-driven flow diagrams to describe system-level timing logic. | Functional | P-SA | Must | Zachman R3C5 |
| REQ-FWK-053 | Cell (R4,C5) — Physical Timing/Scheduling | The system SHALL support a Physical Scheduling and Control Flow design (Row 4, When) specifying batch schedules, cron expressions, event queues, and real-time triggers. | Functional | P-SA | Should | Zachman R4C5 |
| REQ-FWK-054 | Cell (R5,C5) — Actual Schedules/Configs | The system SHALL support ingestion of scheduler configs (Kubernetes CronJobs, AWS EventBridge rules) and map them to R4C5 entries. | Functional | P-AI | Should | Zachman R5C5 |
| REQ-FWK-055 | Cell (R6,C5) — Operational Timing | The system SHALL maintain an operational event log reference linked to actual system timings, reflecting R6C5. | Functional | P-EA | Could | Zachman R6C5 |

---

### 1.7 Zachman Cell Modeling — Why (Motivation/Rationale)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWK-060 | Cell (R1,C6) — Goal/Objective List | The system SHALL support a Scope Goals/Objectives List (Row 1, Why) enumerating strategic goals, vision statements, and high-level objectives. | Functional | P-EA, P-CIO | Must | Zachman R1C6 |
| REQ-FWK-061 | Cell (R2,C6) — Business Rule/Strategy Model | The system SHALL support a Business Strategy and Rules model (Row 2, Why) capturing business rules, policies, strategies, and motivation chains at the conceptual level, compatible with BMM (Business Motivation Model) notation. | Functional | P-EA, P-BA | Must | Zachman R2C6; OMG BMM alignment |
| REQ-FWK-062 | Cell (R3,C6) — System Rules/Logic | The system SHALL support a Logical System Rules model (Row 3, Why) defining application-level business rules, decision tables, and constraint logic. | Functional | P-SA | Must | Zachman R3C6 |
| REQ-FWK-063 | Cell (R4,C6) — Condition/Action Rules | The system SHALL support a Physical Condition/Action rule set (Row 4, Why) specifying implemented validation rules, guard conditions, and rule-engine configurations. | Functional | P-SA | Should | Zachman R4C6 |
| REQ-FWK-064 | Cell (R5,C6) — Actual Rule Implementations | The system SHALL support ingestion of rule engine exports (e.g., Drools DRL, decision model DMN files) and map them to R4C6 entries. | Functional | P-AI | Should | Zachman R5C6; DMN 1.3 |
| REQ-FWK-065 | Cell (R6,C6) — Operational Motivation | The system SHALL link Row 6, Why to governance dashboards and KPI tracking systems to reflect the functioning enterprise's actual strategic alignment. | Integration | P-CIO, P-EA | Could | Zachman R6C6 |

---

### 1.8 Zachman Cell Linkage and Consistency

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWK-070 | Column Vertical Traceability | The system SHALL enforce and visualize vertical traceability within each column (interrogative), ensuring that Row 2 artifacts refine Row 1, Row 3 refines Row 2, and so on, surfacing gaps or breaks in the chain. | Governance | P-EA, P-AUD | Must | Zachman vertical coherence rule |
| REQ-FWK-071 | Row Horizontal Coherence | The system SHALL validate horizontal coherence within each row (perspective), checking that the six interrogative answers for a given perspective are mutually consistent and cross-referencing. | Governance | P-EA, P-AUD | Must | Zachman horizontal coherence rule |
| REQ-FWK-072 | Cell Population Coverage Dashboard | The system SHALL provide a Zachman completeness heatmap dashboard showing which of the 36 cells are Populated, Partially Populated, or Empty, with drill-down to cell artifacts. | Reporting | P-EA, P-CIO | Must | Zachman governance best practice |
| REQ-FWK-073 | Cross-Cell Element Linking | The system SHALL support explicit cross-cell links (e.g., a Data Entity in R2C1 linked to a Business Process in R2C2 that uses it), maintaining a link registry with type annotations. | Functional | P-EA, P-SA | Must | Zachman inter-cell relationships |
| REQ-FWK-074 | Cell Change Impact Analysis | The system SHALL detect when an artifact in one Zachman cell is modified and surface downstream impact on linked cells in other rows/columns, presenting a change impact report. | Governance | P-EA, P-AUD | Should | Change management best practice |
| REQ-FWK-075 | Cell-to-Artifact Type Mapping | The system SHALL maintain a configurable mapping table from Zachman cells to allowed artifact types (e.g., R3C1→ERD/Class Diagram, R2C4→RACI/OrgChart), enforcing or warning when artifacts are placed in cells for which they are not the canonical type. | Governance | P-EA | Should | Zachman artifact classification |
| REQ-FWK-076 | Zachman AI Auto-Classification | The system SHALL use AI to automatically suggest the most likely Zachman cell classification for uploaded artifacts (documents, diagrams, code) based on content analysis, with a confidence score and rationale. | AI/ML | P-AI, P-EA | Should | AI-assisted population; Volume 06 boundary note: orchestration in V06, cell taxonomy in this volume |
| REQ-FWK-077 | Zachman Export — Matrix View | The system SHALL generate an exportable Zachman Matrix view as HTML, PDF, and PowerPoint, rendering all 36 cells with their artifact summaries, links, and coverage status. | Reporting | P-EA, P-CIO | Must | Executive communication |

---

## 2. DoDAF 2.02

### 2.1 DoDAF Foundation and DM2 Alignment

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKD-001 | DoDAF 2.02 Viewpoint Registry | The system SHALL maintain a formal registry of all DoDAF 2.02 viewpoints — All Viewpoint (AV), Capability Viewpoint (CV), Operational Viewpoint (OV), Systems Viewpoint (SV), Services Viewpoint (SvcV), Standards Viewpoint (StdV), Data & Information Viewpoint (DIV), and Project Viewpoint (PV) — with each viewpoint's purpose, audience, and allowed model types. | Data | P-DA, P-EA | Must | DoDAF 2.02 DoDI 7041.3; DoD CIO official specification |
| REQ-FWKD-002 | DM2 Data Model Alignment | The system SHALL align its internal data model for DoDAF artifacts to the DoDAF Meta-Model (DM2) core concepts — Activity, Capability, Information Element, Location, Measure, Operational Performer, Organization, Resource, Role, Rule, Service, Standard, System, and Measure Type — using the DM2 taxonomy as the canonical vocabulary. | Data | P-DA, P-AI | Must | DoDAF 2.02 Volume 2: DM2; ensures interoperability with DoDAF-compliant tools |
| REQ-FWKD-003 | DM2 Entity Type Validation | The system SHALL validate that model elements assigned to a DoDAF view are of the correct DM2 entity type for that view (e.g., Operational Activity for OV-5a/5b, Capability for CV-2/3/4), rejecting or flagging type mismatches. | Governance | P-DA, P-AUD | Must | DM2 type system enforcement |
| REQ-FWKD-004 | Fit-for-Purpose View Guidance | The system SHALL provide fit-for-purpose guidance for each model, surfacing the DoDAF-stated purpose, typical stakeholders, and recommended level of detail when an author begins creating a new model. | UX | P-DA, P-PM | Should | DoDAF 2.02 Volume 1 fit-for-purpose principle |
| REQ-FWKD-005 | DoDAF Version Selector | The system SHALL allow projects to declare their DoDAF version compliance target (2.02 or future updates) and shall version-lock the registry and validation rules accordingly. | Governance | P-DA | Should | Version governance for long-running defense programs |

---

### 2.2 All Viewpoint (AV)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKD-010 | AV-1 Overview & Summary Information | The system SHALL support authoring an AV-1 artifact as a structured document capturing architecture name, description, scope, intended use, assumptions, constraints, environmental conditions, and version metadata, with AI-assisted population from project context. | Functional | P-DA, P-EA | Must | DoDAF 2.02 AV-1: scope and metadata view |
| REQ-FWKD-011 | AV-1 Template Enforcement | The system SHALL enforce a mandatory AV-1 template that includes: Architecture Name, Version, Authorizing Organization, Date Prepared, Scope, Viewpoint Inclusion List, Assumptions, Constraints, and Findings. | Governance | P-DA, P-AUD | Must | DoDAF 2.02 AV-1 required fields |
| REQ-FWKD-012 | AV-2 Integrated Dictionary | The system SHALL support authoring an AV-2 Integrated Dictionary as a structured glossary of all terms, acronyms, and abbreviations used across the architecture, with definitions, sources, and cross-references to model elements that use each term. | Functional | P-DA | Must | DoDAF 2.02 AV-2: terminology standardization |
| REQ-FWKD-013 | AV-2 Automated Term Extraction | The system SHALL automatically extract candidate AV-2 terms from ingested documents, diagrams, and other models, presenting them for author review, disambiguation, and approval, with AI-generated definitions. | AI/ML | P-AI, P-DA | Should | AI-assisted dictionary population |

---

### 2.3 Capability Viewpoint (CV)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKD-020 | CV-1 Vision | The system SHALL support authoring a CV-1 Vision artifact as a narrative/diagram describing the overall capability vision, strategic goals, and high-level capability needs driving the architecture. | Functional | P-DA, P-CIO | Must | DoDAF 2.02 CV-1 |
| REQ-FWKD-021 | CV-2 Capability Taxonomy | The system SHALL support authoring a CV-2 Capability Taxonomy as a hierarchical tree of capabilities with parent-child relationships, capability IDs, definitions, and status (existing/planned/divested), exportable as structured data. | Functional | P-DA, P-EA | Must | DoDAF 2.02 CV-2; hierarchy depth ≤ 8 levels |
| REQ-FWKD-022 | CV-3 Capability Phasing | The system SHALL support authoring a CV-3 Capability Phasing Plan showing the temporal deployment of capabilities across defined time periods or program increments, with Gantt-style or tabular views. | Functional | P-DA, P-PM | Must | DoDAF 2.02 CV-3 |
| REQ-FWKD-023 | CV-4 Capability Dependencies | The system SHALL support authoring a CV-4 Capability Dependencies diagram showing dependency relationships (enables, requires, supports) between capabilities, with directed graph rendering. | Functional | P-DA | Must | DoDAF 2.02 CV-4 |
| REQ-FWKD-024 | CV-5 Capability-to-Organizational Dev | The system SHALL support authoring a CV-5 Capability to Organizational Development mapping, linking capabilities to the organizational units responsible for developing or fielding them, across planning phases. | Functional | P-DA, P-PM | Must | DoDAF 2.02 CV-5 |
| REQ-FWKD-025 | CV-6 Capability-to-Operational Activities | The system SHALL support authoring a CV-6 Capability to Operational Activities mapping, cross-referencing CV-2 capabilities to OV-5 operational activities they support, with a configurable matrix view. | Functional | P-DA | Must | DoDAF 2.02 CV-6 |
| REQ-FWKD-026 | CV-7 Capability-to-Services Mapping | The system SHALL support authoring a CV-7 Capability to Services mapping, linking CV-2 capabilities to SvcV service definitions that realize them, enabling service portfolio analysis. | Functional | P-DA, P-SA | Should | DoDAF 2.02 CV-7 |
| REQ-FWKD-027 | CV AI Generation | The system SHALL provide AI-assisted generation of CV-1 through CV-7 drafts from ingested strategy documents, requirements, and program documentation, with source traceability to originating text. | AI/ML | P-AI, P-DA | Should | Automation value for complex CV hierarchies |

---

### 2.4 Operational Viewpoint (OV)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKD-030 | OV-1 High-Level Operational Concept Graphic | The system SHALL support authoring an OV-1 as a rich graphical context diagram (free-form or template-based) depicting the operational concept, environment, actors, and key interactions at a high level, with annotation overlays and export to PNG/SVG. | Functional | P-DA, P-CIO | Must | DoDAF 2.02 OV-1; typically a PowerPoint/Visio-style diagram |
| REQ-FWKD-031 | OV-2 Operational Resource Flow Description | The system SHALL support authoring an OV-2 depicting Operational Nodes/Performers and the needlines (resource flows) between them, as a labeled directed graph with flow type annotations (data, material, energy). | Functional | P-DA | Must | DoDAF 2.02 OV-2 |
| REQ-FWKD-032 | OV-3 Operational Resource Flow Matrix | The system SHALL support authoring an OV-3 as a matrix cross-referencing source performers, destination performers, needline names, media/quality attributes, and information elements exchanged, with sortable/filterable table view. | Functional | P-DA | Must | DoDAF 2.02 OV-3 |
| REQ-FWKD-033 | OV-4 Organizational Relationships Chart | The system SHALL support authoring an OV-4 showing organizational structure, command/control relationships, and collaboration relationships between organizational entities, as a hierarchical or free-form org chart. | Functional | P-DA, P-PM | Must | DoDAF 2.02 OV-4 |
| REQ-FWKD-034 | OV-5a Operational Activity Decomposition Tree | The system SHALL support authoring an OV-5a as a hierarchical decomposition tree of operational activities, with activity IDs, descriptions, and parent-child nesting to unlimited depth. | Functional | P-DA | Must | DoDAF 2.02 OV-5a |
| REQ-FWKD-035 | OV-5b Operational Activity Model | The system SHALL support authoring an OV-5b as a process flow/IDEF0-style or swim-lane diagram showing inputs, outputs, controls, mechanisms for each operational activity, linked to OV-5a decomposition nodes. | Functional | P-DA, P-BA | Must | DoDAF 2.02 OV-5b; IDEF0 notation support |
| REQ-FWKD-036 | OV-6a Operational Rules Model | The system SHALL support authoring an OV-6a as a structured list of operational constraints and governing rules, tagged by rule type (constraint, policy, assumption), linked to the activities or performers they govern. | Functional | P-DA | Must | DoDAF 2.02 OV-6a |
| REQ-FWKD-037 | OV-6b State Transition Description | The system SHALL support authoring an OV-6b as a UML State Machine or equivalent state transition diagram for operational concepts or systems-of-systems, with states, transitions, triggers, and guards. | Functional | P-DA, P-SE | Must | DoDAF 2.02 OV-6b |
| REQ-FWKD-038 | OV-6c Event-Trace Description | The system SHALL support authoring an OV-6c as a UML Sequence Diagram or equivalent event-trace showing operational events, message exchanges, and temporal ordering between performers, with lifelines and combined fragments. | Functional | P-DA, P-SE | Must | DoDAF 2.02 OV-6c |
| REQ-FWKD-039 | OV AI Generation | The system SHALL provide AI-assisted generation of OV-1 narrative summaries, OV-5a activity hierarchies, and OV-6a rule lists from ingested CONOPS documents, use-case specifications, and SOPs. | AI/ML | P-AI, P-DA | Should | Major time-saver for CONOPS-to-DoDAF translation |

---

### 2.5 Systems Viewpoint (SV)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKD-040 | SV-1 Systems Interface Description | The system SHALL support authoring an SV-1 as a block diagram showing systems/sub-systems and their interfaces (physical or logical connections), with interface type, protocol, and data flow annotations. | Functional | P-DA, P-SE | Must | DoDAF 2.02 SV-1 |
| REQ-FWKD-041 | SV-2 Systems Resource Flow Description | The system SHALL support authoring an SV-2 showing resource flows between systems, analogous to OV-2 but at the system level, with flow labels, rates, and media. | Functional | P-DA, P-SE | Must | DoDAF 2.02 SV-2 |
| REQ-FWKD-042 | SV-3 Systems-to-Systems Matrix | The system SHALL support authoring an SV-3 as a cross-reference matrix of systems-to-systems relationships, indicating interaction types (data exchange, control, feedback). | Functional | P-DA, P-SE | Must | DoDAF 2.02 SV-3 |
| REQ-FWKD-043 | SV-4 Systems Functionality Description | The system SHALL support authoring an SV-4 as a functional decomposition of system functions, linked to OV-5 activities they support, with input/output specification. | Functional | P-DA, P-SE | Must | DoDAF 2.02 SV-4 |
| REQ-FWKD-044 | SV-5a System-to-Operational Activity Traceability Matrix | The system SHALL support authoring an SV-5a as a matrix tracing system functions (SV-4) to operational activities (OV-5), with coverage analysis highlighting gaps. | Functional | P-DA, P-AUD | Must | DoDAF 2.02 SV-5a; traceability |
| REQ-FWKD-045 | SV-5b Operational Activity-to-System Function Traceability Matrix | The system SHALL support authoring an SV-5b as the inverse mapping of SV-5a, tracing operational activities to system functions, supporting completeness analysis. | Functional | P-DA, P-AUD | Should | DoDAF 2.02 SV-5b |
| REQ-FWKD-046 | SV-6 Systems Resource Flow Matrix | The system SHALL support authoring an SV-6 as a structured data matrix of system-to-system resource flows, with flow name, type, quality attributes, and frequency. | Functional | P-DA, P-SE | Must | DoDAF 2.02 SV-6 |
| REQ-FWKD-047 | SV-7 Systems Measures Matrix | The system SHALL support authoring an SV-7 as a matrix of system performance measures, KPPs (Key Performance Parameters), and MOEs (Measures of Effectiveness), linked to system elements. | Functional | P-DA, P-SE, P-PM | Must | DoDAF 2.02 SV-7; JCIDS KPP alignment |
| REQ-FWKD-048 | SV-8 Systems Evolution Description | The system SHALL support authoring an SV-8 as a phased migration roadmap for systems, showing current state, transition states, and target state with timeline, linked to CV-3. | Functional | P-DA, P-PM | Must | DoDAF 2.02 SV-8 |
| REQ-FWKD-049 | SV-9 Systems Technology & Skills Forecast | The system SHALL support authoring an SV-9 as a structured forecast of technologies and personnel skills required to support system evolution, with horizon dates. | Functional | P-DA, P-PM | Should | DoDAF 2.02 SV-9 |
| REQ-FWKD-050 | SV-10a Systems Rules Model | The system SHALL support authoring an SV-10a as a structured list of system-level constraints and rules (analogous to OV-6a), tagged by rule type and linked to system elements. | Functional | P-DA, P-SE | Must | DoDAF 2.02 SV-10a |
| REQ-FWKD-051 | SV-10b Systems State Transition Description | The system SHALL support authoring an SV-10b as a state machine diagram for individual systems or system configurations, with states, transitions, and triggering events. | Functional | P-DA, P-SE | Must | DoDAF 2.02 SV-10b |
| REQ-FWKD-052 | SV-10c Systems Event-Trace Description | The system SHALL support authoring an SV-10c as a sequence/event-trace diagram at the system level, showing message exchanges between system elements with timing annotations. | Functional | P-DA, P-SE | Must | DoDAF 2.02 SV-10c |
| REQ-FWKD-053 | SV-11 Physical Schema | The system SHALL support authoring an SV-11 as a physical data schema artifact (ERD, XML Schema, JSON Schema), aligned with DIV-3 and the DM2 data model. | Functional | P-DA, P-SE | Should | DoDAF 2.02 SV-11 |

---

### 2.6 Services Viewpoint (SvcV)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKD-060 | SvcV-1 Services Context Description | The system SHALL support authoring a SvcV-1 as a diagram showing services (as DoDAF service resources) and their interfaces/connections with consumers, providers, and data exchanges, analogous to SV-1 but for services. | Functional | P-DA, P-SA | Must | DoDAF 2.02 SvcV-1 |
| REQ-FWKD-061 | SvcV-2 Services Resource Flow Description | The system SHALL support authoring a SvcV-2 as a flow diagram of service-to-service resource flows, with service endpoint annotations. | Functional | P-DA, P-SA | Must | DoDAF 2.02 SvcV-2 |
| REQ-FWKD-062 | SvcV-3a Systems-to-Services Matrix | The system SHALL support authoring a SvcV-3a as a cross-reference matrix from systems to services they consume or provide, with interaction type. | Functional | P-DA, P-SA | Must | DoDAF 2.02 SvcV-3a |
| REQ-FWKD-063 | SvcV-3b Services-to-Services Matrix | The system SHALL support authoring a SvcV-3b as a service-to-service dependency matrix, showing orchestration, choreography, and composition relationships. | Functional | P-DA, P-SA | Must | DoDAF 2.02 SvcV-3b |
| REQ-FWKD-064 | SvcV-4 Services Functionality Description | The system SHALL support authoring a SvcV-4 as a functional decomposition of service operations, linked to OV-5 activities, with input/output/pre-condition/post-condition specification. | Functional | P-DA, P-SA | Must | DoDAF 2.02 SvcV-4 |
| REQ-FWKD-065 | SvcV-5 Operational Activity-to-Services Traceability | The system SHALL support authoring a SvcV-5 as a traceability matrix from operational activities (OV-5) to services (SvcV-4 functions), with coverage gap highlighting. | Functional | P-DA, P-AUD | Must | DoDAF 2.02 SvcV-5 |
| REQ-FWKD-066 | SvcV-6 Services Resource Flow Matrix | The system SHALL support authoring a SvcV-6 structured matrix of service-to-service data/resource flows with quality attributes (latency, bandwidth, reliability). | Functional | P-DA, P-SA | Must | DoDAF 2.02 SvcV-6 |
| REQ-FWKD-067 | SvcV-7 Services Measures Matrix | The system SHALL support authoring a SvcV-7 as a matrix of service quality measures (SLAs, KPIs), linked to SvcV-1 service elements. | Functional | P-DA, P-SA, P-PM | Must | DoDAF 2.02 SvcV-7 |
| REQ-FWKD-068 | SvcV-8 Services Evolution Description | The system SHALL support authoring a SvcV-8 as a phased migration plan for services, showing service lifecycle states and timeline, consistent with SV-8. | Functional | P-DA, P-PM | Should | DoDAF 2.02 SvcV-8 |
| REQ-FWKD-069 | SvcV-9 Services Technology & Skills Forecast | The system SHALL support authoring a SvcV-9 forecasting technologies (e.g., SOA platforms, API gateways) and skills required for service evolution. | Functional | P-DA, P-PM | Should | DoDAF 2.02 SvcV-9 |
| REQ-FWKD-070 | SvcV-10a/10b/10c Services Rules, State, Event | The system SHALL support authoring SvcV-10a (service constraints/rules), SvcV-10b (service state transitions), and SvcV-10c (service event traces), fully analogous to their SV-10 counterparts but scoped to services. | Functional | P-DA, P-SA | Must | DoDAF 2.02 SvcV-10a/b/c |

---

### 2.7 Standards Viewpoint (StdV)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKD-080 | StdV-1 Standards Profile | The system SHALL support authoring a StdV-1 as a structured profile of applicable standards, mandates, and specifications, each tagged with: standard name, issuing body, version, category (interface, data, security, etc.), applicability statement, and compliance status. | Functional | P-DA, P-AUD | Must | DoDAF 2.02 StdV-1 |
| REQ-FWKD-081 | StdV-1 Standards Library Integration | The system SHALL provide a curated library of common defense and government standards (e.g., MIL-STD-498, IEEE 12207, NIST SP 800-series, DoD CIO mandates) that can be imported into project-level StdV-1 profiles. | Data | P-DA, P-AUD | Should | Time-saving for defense architects |
| REQ-FWKD-082 | StdV-2 Standards Forecast | The system SHALL support authoring a StdV-2 as a forecast of emerging or planned standards, with anticipated adoption dates and impact on current architecture, linked to SV-9/SvcV-9. | Functional | P-DA, P-PM | Should | DoDAF 2.02 StdV-2 |
| REQ-FWKD-083 | Standards Compliance Traceability | The system SHALL provide traceability from StdV-1 standard entries to model elements (systems, services, interfaces) that must comply, with compliance status tracking (Compliant/Partial/Non-Compliant/Not Assessed). | Governance | P-DA, P-AUD | Must | Compliance governance |

---

### 2.8 Data & Information Viewpoint (DIV)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKD-090 | DIV-1 Conceptual Data Model | The system SHALL support authoring a DIV-1 as a conceptual data/information model depicting high-level information entities, relationships, and cardinalities important to operational activities, independent of system implementation, aligned to DM2 Information Element concepts. | Functional | P-DA, P-BA | Must | DoDAF 2.02 DIV-1; DM2 alignment |
| REQ-FWKD-091 | DIV-2 Logical Data Model | The system SHALL support authoring a DIV-2 as a logical data model normalizing DIV-1 entities into data attributes, keys, and associations, with ER notation, linked to OV-3 needlines and SV-6 flows. | Functional | P-DA, P-SE | Must | DoDAF 2.02 DIV-2 |
| REQ-FWKD-092 | DIV-3 Physical Data Model | The system SHALL support authoring a DIV-3 as a physical data schema linked to DIV-2, specifying database-level constructs, storage formats, and technology-specific types, consistent with SV-11. | Functional | P-DA, P-SE | Must | DoDAF 2.02 DIV-3 |
| REQ-FWKD-093 | DIV Cross-View Information Traceability | The system SHALL automatically cross-reference DIV information elements with their occurrences in OV-3 (needlines), SV-6 (resource flows), SvcV-6 (service flows), and AV-2 (dictionary), surfacing inconsistencies. | Governance | P-DA, P-AUD | Must | DM2 data consistency |

---

### 2.9 Project Viewpoint (PV)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKD-100 | PV-1 Project Portfolio Relationships | The system SHALL support authoring a PV-1 diagram showing relationships between acquisition programs/projects and their architectural contributions, including dependencies and shared resources. | Functional | P-DA, P-PM | Must | DoDAF 2.02 PV-1 |
| REQ-FWKD-101 | PV-2 Project Timelines | The system SHALL support authoring a PV-2 as a Gantt or timeline view of project/program milestones, phases, and key decision points, linked to CV-3 capability phasing. | Functional | P-DA, P-PM | Must | DoDAF 2.02 PV-2 |
| REQ-FWKD-102 | PV-3 Project-to-Capability Mapping | The system SHALL support authoring a PV-3 as a matrix mapping projects/programs to the capabilities (CV-2) and architecture products they deliver, with planned delivery dates. | Functional | P-DA, P-PM | Must | DoDAF 2.02 PV-3 |
| REQ-FWKD-103 | Program Integration with Portfolio Module | The system SHALL integrate PV models with the ESIP Portfolio module (Volume 09), enabling bidirectional data exchange for program milestones, budgets, and capability delivery status. | Integration | P-PM, P-CIO | Should | Cross-module coherence |

---

### 2.10 DoDAF Cross-View Analysis

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKD-110 | DoDAF Architecture Completeness Scoring | The system SHALL calculate a DoDAF completeness score per project, indicating which of the standard models (AV-1/2, CV-1..7, OV-1..6c, SV-1..11, SvcV-1..10c, StdV-1..2, DIV-1..3, PV-1..3) are present, partially complete, or missing. | Reporting | P-DA, P-AUD | Must | Program review preparation |
| REQ-FWKD-111 | OV-to-SV-to-SvcV Traceability Chain | The system SHALL maintain and visualize the full traceability chain from Operational Activities (OV-5) → System Functions (SV-4, SV-5) → Service Operations (SvcV-4, SvcV-5), highlighting breaks in the chain. | Governance | P-DA, P-SE, P-AUD | Must | DoDAF fit-for-purpose traceability |
| REQ-FWKD-112 | DoDAF Automated View Generation | The system SHALL support AI-assisted generation of AV-1 summaries, CV-2 hierarchies, OV-1 narratives, and OV-5a activity trees from uploaded program documentation (SOW, CONOPS, CDD, ICD), with source text citations per generated element. | AI/ML | P-AI, P-DA | Should | Core automation value |
| REQ-FWKD-113 | DoDAF Export — UPDM/XMI Format | The system SHALL support export of DoDAF models in UPDM (Unified Profile for DoDAF/MODAF) XMI format, compatible with IBM Rational Rhapsody, Sparx EA, and No Magic Cameo, with field-level mapping validation. | Integration | P-DA, P-SE | Must | Tool interoperability |
| REQ-FWKD-114 | DoDAF Export — DDMS/KML | The system SHALL support export of geospatially relevant models (OV-2 nodes, SV-1 locations) in KML and DDMS-compliant XML for integration with GIS and C2 systems. | Integration | P-DA | Could | Geospatial community requirement |

---

## 3. MODAF / NAF v4

### 3.1 MODAF Foundation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKM-001 | MODAF Viewpoint Registry | The system SHALL maintain a MODAF viewpoint registry covering all seven MODAF viewpoint families: All Views (AV), Strategic (StV), Operational (OV), Systems (SV), Acquisition (AcV), Service-Oriented (SOV), and Technical (TV), with canonical model names and descriptions per viewpoint. | Data | P-DA, P-EA | Must | UK MOD Architecture Framework (MODAF) v1.2.004 |
| REQ-FWKM-002 | MODAF-DoDAF Semantic Mapping | The system SHALL maintain a formal semantic mapping between MODAF views and their DoDAF 2.02 counterparts (e.g., MODAF StV-3 ↔ DoDAF CV-1, MODAF OV-1 ↔ DoDAF OV-1), surfacing the mapping when authors switch frameworks mid-project. | Data | P-DA, P-EA | Must | MODAF-DoDAF reconciliation; UK MOD guidance |
| REQ-FWKM-003 | MODAF Version Selector | The system SHALL support MODAF version selection (1.2, 1.2.004) and lock associated validation rules per project. | Governance | P-DA | Should | Version governance |

---

### 3.2 MODAF Strategic Viewpoint (StV)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKM-010 | StV-1 Capability Vision | The system SHALL support authoring a MODAF StV-1 Capability Vision artifact as a narrative or structured document defining strategic intent and capability goals, linked to DoDAF CV-1 if dual-framework mode is active. | Functional | P-DA, P-CIO | Must | MODAF StV-1 |
| REQ-FWKM-011 | StV-2 Capability Taxonomy (MODAF) | The system SHALL support authoring a MODAF StV-2 Capability Taxonomy as a hierarchical capability tree with UK MOD-standard capability categories, reusable across MODAF projects. | Functional | P-DA | Must | MODAF StV-2 |
| REQ-FWKM-012 | StV-3 Capability Phasing (MODAF) | The system SHALL support authoring a MODAF StV-3 as a temporal phasing plan of capabilities, analogous to DoDAF CV-3, with Through-Life Management (TLM) phase labels. | Functional | P-DA, P-PM | Must | MODAF StV-3; UK TLM alignment |
| REQ-FWKM-013 | StV-4 Capability Dependencies (MODAF) | The system SHALL support authoring a MODAF StV-4 capability dependency graph, consistent with DoDAF CV-4 semantics. | Functional | P-DA | Must | MODAF StV-4 |
| REQ-FWKM-014 | StV-5 Capability-to-Org Mapping (MODAF) | The system SHALL support MODAF StV-5 mapping capabilities to UK MOD organisational entities (TLB/Budget Holders), with UK-specific organisational hierarchy. | Functional | P-DA | Should | MODAF StV-5 |
| REQ-FWKM-015 | StV-6 Capability-to-Operational Activities (MODAF) | The system SHALL support MODAF StV-6 cross-reference from strategic capabilities to operational activities. | Functional | P-DA | Should | MODAF StV-6 |
| REQ-FWKM-016 | StV-7 Performance Parameters | The system SHALL support MODAF StV-7 as a structured table of performance parameters for each capability, including Key Performance Indicators and threshold/objective values. | Functional | P-DA, P-PM | Should | MODAF StV-7 |

---

### 3.3 MODAF Acquisition Viewpoint (AcV)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKM-020 | AcV-1 Acquisition Programme | The system SHALL support authoring a MODAF AcV-1 depicting the acquisition programme structure, showing programmes, their Inter-Dependencies (IDs), and relationship to capability delivery (StV-3). | Functional | P-DA, P-PM | Must | MODAF AcV-1; no direct DoDAF equivalent — unique to MODAF |
| REQ-FWKM-021 | AcV-2 Programme Timeline | The system SHALL support authoring a MODAF AcV-2 as a programme timeline/Gantt showing approval gates (Main Gates), assessment phases, and delivery milestones per UK MOD CADMID lifecycle. | Functional | P-DA, P-PM | Must | MODAF AcV-2; CADMID gate model |
| REQ-FWKM-022 | AcV Lifecycle Gate Validation | The system SHALL validate that AcV-2 timelines include required CADMID gates (Concept, Assessment, Demonstration, Manufacture, In-Service, Disposal) and flag missing gates. | Governance | P-DA, P-AUD | Should | UK MOD CADMID compliance |

---

### 3.4 MODAF Service-Oriented Viewpoint (SOV)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKM-030 | SOV-1 Service Taxonomy | The system SHALL support authoring a MODAF SOV-1 service taxonomy as a hierarchical classification of services, analogous to DoDAF SvcV-1 but with MODAF-specific service categories. | Functional | P-DA, P-SA | Must | MODAF SOV-1 |
| REQ-FWKM-031 | SOV-2 through SOV-5 Service Models | The system SHALL support MODAF SOV-2 (Service Deployment), SOV-3 (Service Contracts), SOV-4 (Service Orchestration), and SOV-5 (Service Functional Decomposition), each with templated authoring views. | Functional | P-DA, P-SA | Must | MODAF SOV-2..5 |
| REQ-FWKM-032 | SOV-DoDAF SvcV Mapping | The system SHALL maintain a formal mapping from MODAF SOV models to their DoDAF SvcV equivalents, enabling synchronized dual-framework projects. | Integration | P-DA | Should | Interoperability |

---

### 3.5 NAF v4 Grid

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKM-040 | NAF v4 Grid Registry | The system SHALL maintain a formal registry of the NATO Architecture Framework (NAF) v4 architecture grid, organized by viewpoint (C1–C8: Concepts; L1–L8: Logical; P1–P8: Physical; A1–A5: Architecture Overview and Summary) with canonical descriptions for each grid cell. | Data | P-DA, P-EA | Must | NATO C3 Board NAF v4 (2018); NATO STANAG 5524 |
| REQ-FWKM-041 | NAF v4 Concepts Viewpoints (C1–C8) | The system SHALL support authoring NAF v4 concepts-level architecture products C1 (Capability Taxonomy), C2 (Enterprise Vision), C3 (Capability Dependencies), C4 (Standard Processes), C5 (Capability-to-Org), C6 (Capability Phasing), C7 (Performance Parameters), C8 (Standards Constraints) as structured artifacts with validation rules per NAF v4 definition. | Functional | P-DA | Must | NAF v4 grid; NATO-specific C-column |
| REQ-FWKM-042 | NAF v4 Logical Viewpoints (L1–L8) | The system SHALL support authoring NAF v4 logical-level products L1 (Node Connectivity), L2 (Logical Scenario), L3 (Node Decomposition), L4 (Logical Activities), L5 (Logical States), L6 (Logical Sequence), L7 (Logical Data Model), L8 (Logical Constraints) with diagram and structured data editors. | Functional | P-DA, P-SE | Must | NAF v4 grid L-column |
| REQ-FWKM-043 | NAF v4 Physical Viewpoints (P1–P8) | The system SHALL support authoring NAF v4 physical-level products P1 (Resource Connectivity), P2 (Resource Deployment), P3 (Resource Composition), P4 (Resource Functions), P5 (Resource States), P6 (Resource Sequence), P7 (Physical Data Model), P8 (Physical Constraints) with structured artifact editors. | Functional | P-DA, P-SE | Must | NAF v4 grid P-column |
| REQ-FWKM-044 | NAF v4 Architecture Overview (A-Column) | The system SHALL support authoring NAF v4 architecture overview products A1 (Architecture Overview), A2 (Architecture Products), A3 (Architecture Compliance), A4 (Architectural Lineage), A5 (Architecture Status) as mandatory framing artifacts for all NAF projects. | Functional | P-DA, P-AUD | Must | NAF v4 A-column |
| REQ-FWKM-045 | NAF v4 Grid Completeness Dashboard | The system SHALL provide a NAF v4 grid completeness heatmap (analogous to the Zachman heatmap) showing populated/partial/empty cells across the C/L/P/A grid. | Reporting | P-DA, P-CIO | Must | NAF governance |
| REQ-FWKM-046 | NAF v4 / MODAF / DoDAF Reconciliation | The system SHALL provide a formal three-way reconciliation mapping (NAF v4 ↔ MODAF ↔ DoDAF 2.02) for each view/product, allowing architects to trace a NAF concept to its MODAF and DoDAF equivalents and identify gaps. | Integration | P-DA, P-EA | Must | Multi-standard defense programs |
| REQ-FWKM-047 | NAF v4 Export — SysML/UPDM | The system SHALL support export of NAF v4 models in SysML profile format and UPDM XMI, compatible with Cameo Systems Modeler NAF plugin and Sparx EA NAF toolbox. | Integration | P-DA, P-SE | Must | Tool interoperability |

---

## 4. UAF — Unified Architecture Framework

### 4.1 UAF Foundation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKU-001 | UAF Domain Registry | The system SHALL maintain a formal registry of UAF domains: Doctrine (Dr), Services (Sv), Personnel (Pr), Materiel (Mt), Facilities (Fc), Policy (Pl), Processes (Ps), and Enterprise (Er), with canonical definitions per the OMG Unified Architecture Framework v1.1 specification. | Data | P-EA, P-SE | Must | OMG UAF v1.1 (formal/2019-05-17) |
| REQ-FWKU-002 | UAF Aspect Registry | The system SHALL maintain a formal registry of UAF aspects: Strategy (St), Operational (Op), Services/Capability (Sv), Structure (Sc), Connectivity (Co), Behavior (Be), Information/Data (In), Parameters (Pa), and Roadmap (Rm), with definitions per OMG UAF v1.1. | Data | P-EA, P-SE | Must | OMG UAF v1.1 aspect taxonomy |
| REQ-FWKU-003 | UAF Grid Enforcement | The system SHALL enforce the UAF grid (Domains × Aspects), assigning each UAF viewpoint to its correct grid cell, and preventing creation of models at undefined grid intersections. | Governance | P-EA, P-AUD | Must | UAF grid topology |
| REQ-FWKU-004 | UAF Version Selector | The system SHALL support UAF version selection (1.0, 1.1) with version-specific grid cell definitions and validation rules. | Governance | P-EA | Should | OMG versioning |

---

### 4.2 UAF Viewpoints by Aspect

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKU-010 | UAF Strategy Aspect Viewpoints | The system SHALL support authoring all UAF Strategy aspect viewpoints, including Dr-St (Doctrine Strategy), Sv-St (Services Strategy), and enterprise-level strategy views, with capability goals, strategic assumptions, and constraint statements. | Functional | P-EA, P-CIO | Must | UAF v1.1 Strategy aspect |
| REQ-FWKU-011 | UAF Operational Aspect Viewpoints | The system SHALL support authoring UAF Operational aspect viewpoints across applicable domains, including operational concept models, use case scenarios, operational activities, and operational constraints, using BPMN or UAF-specific notation. | Functional | P-EA, P-BA, P-SE | Must | UAF v1.1 Operational aspect |
| REQ-FWKU-012 | UAF Structure Aspect Viewpoints | The system SHALL support authoring UAF Structure aspect viewpoints depicting organizational structure, system composition, service structure, and personnel structure views per applicable domain. | Functional | P-EA, P-SE | Must | UAF v1.1 Structure aspect |
| REQ-FWKU-013 | UAF Connectivity Aspect Viewpoints | The system SHALL support authoring UAF Connectivity aspect viewpoints showing resource flows, information exchanges, interface connections, and network topology between structural elements. | Functional | P-EA, P-SE | Must | UAF v1.1 Connectivity aspect |
| REQ-FWKU-014 | UAF Behavior Aspect Viewpoints | The system SHALL support authoring UAF Behavior aspect viewpoints depicting state machines, event traces, activity sequences, and behavioral rules for systems and services. | Functional | P-SE | Must | UAF v1.1 Behavior aspect |
| REQ-FWKU-015 | UAF Information Aspect Viewpoints | The system SHALL support authoring UAF Information/Data aspect viewpoints with conceptual, logical, and physical data models, aligned with DIV-1/2/3 and the DM2 for defense projects. | Functional | P-EA, P-SE, P-DA | Must | UAF v1.1 Information aspect |
| REQ-FWKU-016 | UAF Parameters Aspect Viewpoints | The system SHALL support authoring UAF Parameters aspect viewpoints capturing performance parameters, KPIs, measures of effectiveness, and SLA thresholds. | Functional | P-SE, P-PM | Should | UAF v1.1 Parameters aspect |
| REQ-FWKU-017 | UAF Roadmap Aspect Viewpoints | The system SHALL support authoring UAF Roadmap aspect viewpoints showing temporal migration plans, capability phasing, and resource deployment roadmaps. | Functional | P-EA, P-PM | Must | UAF v1.1 Roadmap aspect |

---

### 4.3 UAF ↔ SysML Integration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKU-020 | UAF SysML Profile Support | The system SHALL implement UAF modeled as a SysML Profile (per OMG UAF v1.1 normative SysML profile specification), enabling UAF viewpoints to be expressed as SysML stereotype-extended block definition diagrams (BDD), internal block diagrams (IBD), activity diagrams, sequence diagrams, and parametric diagrams. | Functional | P-SE, P-EA | Must | OMG UAF v1.1 SysML profile annex |
| REQ-FWKU-021 | UAF SysML XMI Export/Import | The system SHALL support export and import of UAF models as SysML XMI 2.5.1, compatible with Cameo Systems Modeler, Sparx EA with SysML plugin, and IBM Rhapsody. | Integration | P-SE | Must | Tool interoperability |
| REQ-FWKU-022 | UAF Parametric Model Integration | The system SHALL support UAF parametric diagrams (mapped to SysML par diagrams) for capturing performance constraints and executing constraint expressions via embedded or linked solvers. | Functional | P-SE | Should | Model-Based Systems Engineering (MBSE) value |
| REQ-FWKU-023 | UAF MBSE Mode | The system SHALL provide a dedicated MBSE mode for UAF projects that enforces model-element reuse across viewpoints — ensuring a structural element defined in a Structure viewpoint is referenced (not copied) by Connectivity, Behavior, and Parameters viewpoints. | Governance | P-SE, P-EA | Must | MBSE discipline; single-source-of-truth |
| REQ-FWKU-024 | UAF-DoDAF-MODAF Cross-Framework Mode | The system SHALL support a cross-framework project mode where a single model repository is simultaneously expressed in UAF, DoDAF 2.02, and MODAF/NAF v4, with automated synchronization of shared elements (e.g., capability definitions, organizational nodes, information elements). | Integration | P-DA, P-EA, P-SE | Should | Multi-standard programs (NATO + US DoD + UK MOD) |

---

## 5. FEAF Reference Models

### 5.1 FEAF Foundation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKF-001 | FEAF Reference Model Registry | The system SHALL maintain a registry of the five FEAF 2.0 Reference Models — Performance Reference Model (PRM), Business Reference Model (BRM), Service Component Reference Model (SRM/formerly ARM), Data Reference Model (DRM), and Technical Reference Model (IRM/TRM) — with canonical descriptions, sub-layer structures, and taxonomy nodes per Federal Enterprise Architecture Framework v2. | Data | P-EA, P-CIO | Must | FEA Practice Guidance (OMB, 2013); FEAF 2.0 |
| REQ-FWKF-002 | FEAF Version Lock | The system SHALL support FEAF versioning (FEAF 1.0, FEAF 2.0, Agency-specific extensions) and maintain version-specific taxonomy nodes per project. | Governance | P-CIO, P-AUD | Should | Regulatory compliance |

---

### 5.2 Performance Reference Model (PRM)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKF-010 | PRM Taxonomy Support | The system SHALL support the FEAF PRM hierarchy: Mission and Business Results → Customer Results → Processes and Activities → Technology, with each level's standard outcome categories and measurement indicators, allowing agencies to tag projects and investments to PRM nodes. | Data | P-CIO, P-EA | Must | FEAF PRM layer structure |
| REQ-FWKF-011 | PRM Measurement Linkage | The system SHALL link PRM outcome indicators to portfolio investments (Volume 09) and roadmap milestones, enabling traceability from federal spending to performance outcomes. | Integration | P-CIO, P-PM | Must | OMB Exhibit 53/300 alignment |

---

### 5.3 Business Reference Model (BRM)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKF-020 | BRM Business Functions Taxonomy | The system SHALL provide the standard FEAF BRM business function taxonomy (Business Areas → Lines of Business → Sub-Functions) as a built-in controlled vocabulary, allowing agency capabilities and processes to be mapped to BRM nodes. | Data | P-EA, P-BA, P-CIO | Must | FEAF BRM v2.0; OMB reference |
| REQ-FWKF-021 | BRM Agency Customization | The system SHALL allow agencies to extend the standard BRM taxonomy with agency-specific business functions while preserving linkage to standard BRM nodes for cross-agency comparability. | Functional | P-EA | Should | Agency-specific extensions |
| REQ-FWKF-022 | BRM Capability Cross-Reference | The system SHALL automatically cross-reference BRM business functions with enterprise capabilities (DoDAF CV-2, FEAF capabilities), surfacing potential duplications or alignment gaps. | AI/ML | P-EA, P-CIO | Should | AI-assisted analysis |

---

### 5.4 Data Reference Model (DRM)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKF-030 | DRM Data Sharing Taxonomy | The system SHALL implement the FEAF DRM three-component taxonomy — Data Context (standardized descriptions of data), Data Sharing (data exchange patterns and standards), and Data Description (metadata, data standards) — enabling agencies to classify data assets against DRM categories. | Data | P-EA, P-SA | Must | FEAF DRM v2.0 |
| REQ-FWKF-031 | DRM Data Asset Registry | The system SHALL provide a Data Asset Registry where agency data assets are tagged with DRM categories, data custodians, sharing restrictions (FISMA classification levels, PII flags), and linkage to BRM functions. | Data | P-EA, P-AUD | Must | FEAF DRM; FISMA data classification |
| REQ-FWKF-032 | DRM-DIV Alignment | The system SHALL provide automated alignment checking between FEAF DRM data assets and DoDAF DIV information elements, flagging conceptual conflicts or naming inconsistencies for dual-framework projects. | Governance | P-DA, P-EA | Should | Cross-framework data governance |

---

### 5.5 Technical/Infrastructure Reference Model (IRM/TRM)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKF-040 | IRM/TRM Standards Taxonomy | The system SHALL implement the FEAF TRM/IRM technology standards taxonomy (Service Areas → Service Categories → Standards/Specifications) as a controlled vocabulary, with each node pre-populated with applicable Federal standards (FIPS, NIST SP, IEEE). | Data | P-SA, P-AUD | Must | FEAF TRM v2.0 |
| REQ-FWKF-041 | IRM Technology Inventory | The system SHALL maintain an agency Technology Inventory mapping deployed technologies to TRM categories, with lifecycle status (Emerging/Standard/Containing/Retiring), and vendor/product details. | Data | P-SA, P-EA | Must | FEAF IRM asset inventory |
| REQ-FWKF-042 | IRM-StdV Linkage | The system SHALL provide automated linkage between FEAF IRM standards entries and DoDAF StdV-1 profiles for dual-framework projects, ensuring consistent standards management. | Integration | P-DA, P-EA | Should | Cross-framework integration |

---

### 5.6 Service Component Reference Model (SRM)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKF-050 | SRM Service Component Taxonomy | The system SHALL implement the FEAF SRM service component taxonomy (Service Domains → Service Types → Service Components), enabling agencies to map capabilities and IT services to SRM categories. | Data | P-SA, P-EA | Must | FEAF SRM v2.0 |
| REQ-FWKF-051 | SRM Shared Service Identification | The system SHALL support analysis workflows that identify SRM service components shared across multiple agency capabilities, flagging consolidation opportunities and potential shared service candidates. | Reporting | P-CIO, P-EA | Should | OMB shared services mandate |

---

### 5.7 FEAF Integrated View

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKF-060 | FEAF Reference Model Cross-Walk | The system SHALL provide a FEAF cross-walk dashboard visually linking: agency capabilities → BRM functions → PRM outcomes → SRM components → DRM data assets → TRM technology standards, with drill-down to individual artifacts. | Reporting | P-CIO, P-AUD | Must | FEAF integrated portfolio analysis |
| REQ-FWKF-061 | FEAF Compliance Report | The system SHALL generate a FEAF compliance report suitable for OMB submission, summarizing coverage across all five reference models, gaps, and roadmap items, in both HTML and PDF formats. | Reporting | P-CIO, P-AUD | Must | OMB EA compliance reporting |
| REQ-FWKF-062 | Gartner/BIAN Framework Note | The system SHALL provide an extensible framework adapter that can accommodate additional industry EA frameworks (Gartner Enterprise Architecture, BIAN for banking, HR Open Standards) via configuration, without requiring code changes, following the same viewpoint-registry pattern as FEAF. | Functional | P-EA | Could | Future extensibility; Gartner and BIAN are framework options but below MVP scope |

---

## 6. Cross-Framework Capabilities

### 6.1 Framework Selection Wizard

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKX-001 | Framework Selection Wizard | The system SHALL provide a guided Framework Selection Wizard that, given inputs about project context (industry, regulatory regime, stakeholder community, purpose: compliance/design/acquisition/strategy), recommends the most appropriate EA framework(s) with a ranked list, rationale, and a comparison matrix. | UX | P-EA, P-CIO | Must | Usability; non-expert adoption |
| REQ-FWKX-002 | Wizard Input Dimensions | The Wizard SHALL evaluate at minimum: (a) Industry sector (Government-Defense, Government-Civilian, Commercial, Academic), (b) Regulatory requirement (NDAA/DoD directive, OMB EA mandate, ISO 42010), (c) Primary objective (capability planning, system design, compliance, digital transformation), (d) Stakeholder audience (military, federal CIO, board, development team), (e) Existing tool ecosystem (Sparx EA, No Magic, Archi, IBM Rhapsody). | UX | P-EA | Must | Comprehensive recommendation logic |
| REQ-FWKX-003 | Dual-Framework Project Mode | The system SHALL support designating a project as Dual-Framework (e.g., DoDAF + FEAF, MODAF + NAF v4), activating synchronized view registries, shared element repositories, and cross-framework traceability for all applicable views. | Functional | P-DA, P-EA | Must | Joint US/UK programs; federal defense agencies |
| REQ-FWKX-004 | Multi-Framework Project Mode | The system SHALL support Multi-Framework mode for projects requiring simultaneous compliance with three or more frameworks, with a shared element pool and per-framework view filters. | Functional | P-EA | Should | NATO/multilateral programs |

---

### 6.2 Viewpoint Libraries

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKX-010 | Unified Viewpoint Library | The system SHALL maintain a Unified Viewpoint Library aggregating all viewpoints from Zachman, DoDAF 2.02, MODAF, NAF v4, UAF, TOGAF (V01), and FEAF, with each entry tagged by framework, purpose, audience, DM2 entity types, and maturity level. | Data | P-EA, P-AI | Must | Central governance of viewpoint catalog |
| REQ-FWKX-011 | Viewpoint Search and Filter | The system SHALL provide viewpoint search and filter capabilities allowing architects to find relevant viewpoints by keyword, framework, audience, or purpose, with a preview of the viewpoint template and example output. | UX | P-EA, P-DA | Must | Discoverability |
| REQ-FWKX-012 | Custom Viewpoint Definition | The system SHALL allow organizations to define custom viewpoints by specifying: name, parent framework (or "Custom"), applicable DM2/ISO 42010 element types, template structure, and validation rules, storing them in the Unified Viewpoint Library as custom entries. | Functional | P-EA | Should | Enterprise-specific view types |
| REQ-FWKX-013 | Viewpoint Template Versioning | The system SHALL version all viewpoint templates, preserving historical snapshots when templates are updated, and alerting architects with open models that a new template version is available. | Governance | P-EA, P-AUD | Should | Template governance |

---

### 6.3 Model Interoperability and Mapping

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKX-020 | ArchiMate ↔ Zachman Cell Mapping | The system SHALL maintain a formal mapping from ArchiMate 3.1 element types to Zachman cells (e.g., ArchiMate Business Actor → R2/R3 × C4; ArchiMate Application Component → R3/R4 × C2; ArchiMate Technology Node → R4 × C3), enabling automatic Zachman cell assignment for ArchiMate-modeled elements. | Integration | P-EA | Must | ArchiMate 3.1 / Zachman cross-reference; V03 boundary: diagram engine in V03, mapping table in this volume |
| REQ-FWKX-021 | ArchiMate ↔ TOGAF ADM Phase Mapping | The system SHALL maintain the mapping from ArchiMate element types and relationships to the TOGAF ADM phase in which they are primarily created/used (V01 boundary: ADM phase guidance in V01, mapping table in this volume). | Integration | P-EA | Must | TOGAF 10 / ArchiMate 3.1 white paper |
| REQ-FWKX-022 | ArchiMate ↔ DoDAF View Mapping | The system SHALL maintain a formal mapping from ArchiMate element types and viewpoints to DoDAF views (e.g., ArchiMate Organization viewpoint → OV-4; ArchiMate Application Cooperation viewpoint → SV-1; ArchiMate Data Flow viewpoint → OV-3/DIV), enabling automatic DoDAF view population from ArchiMate models. | Integration | P-EA, P-DA | Must | ArchiMate-DoDAF cross-mapping; The Open Group guidance |
| REQ-FWKX-023 | TOGAF ↔ Zachman Cell Mapping | The system SHALL maintain a mapping from TOGAF ADM architecture domains (Business, Data, Application, Technology) and deliverables to Zachman cells, surfacing the mapping in the UI and in export metadata. | Integration | P-EA | Must | Zachman-TOGAF cross-reference table |
| REQ-FWKX-024 | DoDAF ↔ UAF Domain/Aspect Mapping | The system SHALL maintain a formal mapping from DoDAF 2.02 models to UAF domain/aspect grid cells (e.g., OV-1 → Operational domain, Strategy aspect; SV-1 → Materiel domain, Connectivity aspect), enabling automatic UAF grid population from DoDAF models. | Integration | P-SE, P-DA | Must | OMG UAF to DoDAF crosswalk |
| REQ-FWKX-025 | Element-Level Cross-Framework Synchronization | The system SHALL support element-level synchronization across frameworks — when a canonical element (e.g., a Capability definition) is updated in one framework view, the change SHALL propagate to all linked elements in other frameworks, with a conflict resolution workflow for manual overrides. | Functional | P-EA, P-DA | Must | Single source of truth across frameworks |
| REQ-FWKX-026 | Cross-Framework Consistency Checker | The system SHALL provide a Cross-Framework Consistency Checker that identifies semantic conflicts between equivalent elements in different framework representations (e.g., mismatched capability names in CV-2 vs. StV-2 vs. UAF C1), presenting a reconciliation workbench. | Governance | P-EA, P-AUD | Must | Governance for multi-framework programs |
| REQ-FWKX-027 | ISO 42010 Alignment | The system SHALL represent all framework viewpoints and views in conformance with ISO/IEC/IEEE 42010:2022 (Systems and software engineering — Architecture description) concepts — Architecture Description, Viewpoint, View, Model, Model Kind — enabling any framework's content to be expressed as an ISO 42010-conformant Architecture Description. | Governance | P-EA, P-AUD | Should | ISO 42010:2022 compliance; internationally recognized standard |

---

### 6.4 Export to Framework-Specific Formats

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKX-030 | Zachman Export — HTML/PDF/PPTX Matrix | The system SHALL export a fully populated 36-cell Zachman matrix in HTML (interactive), PDF (print-ready), and PPTX (presentation) formats, with artifact summaries, coverage status, and cross-cell link counts per cell. | Reporting | P-EA, P-CIO | Must | Executive deliverable |
| REQ-FWKX-031 | DoDAF Export — UPDM XMI | The system SHALL export DoDAF models as UPDM 2.1.1 XMI, validated against the UPDM schema, for import into IBM Rational Rhapsody, Sparx Enterprise Architect, and No Magic Cameo. | Integration | P-DA, P-SE | Must | DoDAF tool ecosystem |
| REQ-FWKX-032 | DoDAF Export — FACE/C2SIL | The system SHALL support export of DoDAF SV/SvcV models as FACE (Future Airborne Capability Environment) conformant JSON and C2SIL XML for tactical/command-and-control integration. | Integration | P-DA | Could | Specialized defense integrations |
| REQ-FWKX-033 | MODAF Export — UPDM XMI | The system SHALL export MODAF models as UPDM XMI with MODAF-specific stereotypes applied, consistent with UK MOD toolchain requirements. | Integration | P-DA | Must | UK MOD toolchain |
| REQ-FWKX-034 | NAF v4 Export — SysML/NAF Plugin | The system SHALL export NAF v4 grid artifacts in SysML XMI with NAF v4 profile stereotypes for import into Cameo Enterprise Architecture NAF Plugin. | Integration | P-DA, P-SE | Must | NATO toolchain |
| REQ-FWKX-035 | UAF Export — SysML/XMI | The system SHALL export UAF models as SysML XMI 2.5.1 with OMG UAF v1.1 profile stereotypes, validated against the OMG UAF normative SysML profile schema. | Integration | P-SE | Must | MBSE toolchain |
| REQ-FWKX-036 | FEAF Export — JSON/XML | The system SHALL export FEAF reference model data as structured JSON and XML conformant to OMB FEA data standards, suitable for submission to agency EA repositories or ITAR-level systems. | Reporting | P-CIO, P-AUD | Must | Federal compliance |
| REQ-FWKX-037 | Universal ArchiMate XML Export | The system SHALL export any cross-framework project as an ArchiMate Exchange Format XML (archimate-model.org exchange schema), with framework-specific metadata preserved as custom property extensions. | Integration | P-EA | Must | Open standard; V03 boundary: diagram rendering in V03, exchange format in this volume |
| REQ-FWKX-038 | Batch Export with Change Filtering | The system SHALL support batch export of all models in a project, with filtering options to export only changed artifacts since a specified date, for incremental delivery workflows. | Functional | P-DA, P-PM | Should | Program delivery efficiency |

---

### 6.5 Performance, Security, and Governance NFRs

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKX-050 | Framework Model Load Performance | The system SHALL load any single framework view with up to 1,000 model elements in under 3 seconds on a standard browser client (Chrome/Edge, 16GB RAM, 10Mbps network). | NFR | P-DA, P-SE | Must | Usability threshold |
| REQ-FWKX-051 | Concurrent Multi-Framework Project Support | The system SHALL support at least 50 concurrent users working within the same multi-framework project without degradation of consistency check latency beyond 5 seconds. | NFR | P-DA, P-EA | Must | Enterprise scale |
| REQ-FWKX-052 | Framework Classification Data Security | The system SHALL apply the project's security classification label (Unclassified, CUI, Secret if hosted on authorized infrastructure) to all framework models and enforce access control based on classification level and user clearance attributes. | Security | P-DA, P-AUD | Must | Defense data security; NIST SP 800-53 AC controls |
| REQ-FWKX-053 | Model Audit Trail | The system SHALL maintain an immutable audit trail for all create, update, and delete operations on framework models, including user ID, timestamp, changed fields, before/after values, and framework/viewpoint context, retained for a minimum of 7 years. | Security | P-AUD | Must | Regulatory audit; NIST SP 800-53 AU controls |
| REQ-FWKX-054 | Framework Access Control | The system SHALL enforce role-based access control at the framework-viewpoint level — allowing organizations to grant different users read, draft, approve, or admin access to individual viewpoints within a project. | Security | P-AUD, P-EA | Must | Need-to-know control for sensitive architectural views |
| REQ-FWKX-055 | Framework Model Versioning | The system SHALL maintain full version history for all framework models, with branch-and-merge support for parallel development of architecture variants, and three-way merge conflict detection. | Functional | P-EA, P-DA | Must | Architecture baseline management |
| REQ-FWKX-056 | Framework Baseline Locking | The system SHALL support locking a framework architecture baseline (snapshot) for governance review, preventing modifications to locked artifacts without an approved change request workflow. | Governance | P-AUD, P-EA | Must | Configuration management |
| REQ-FWKX-057 | AI Model Generation Attribution | The system SHALL tag all AI-generated or AI-suggested model elements with a provenance attribute indicating generation method (AI-suggested, AI-generated, Human-authored, AI-assisted-human-approved), source documents cited, and AI model version used, visible in audit trail and exports. | Governance | P-AUD, P-AI | Must | AI accountability; V06 boundary: AI generation engine in V06, attribution data model in this volume |

---

## 7. AI/ML Capabilities Specific to This Volume

*Note: Detailed AI orchestration, model selection, prompt engineering, and pipeline configuration requirements are governed by Volume 06. This section records requirements where the data model, framework taxonomy, or output format is specific to this volume.*

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKA-001 | Framework-Aware Document Ingestion | The system SHALL ingest uploaded documents (PDF, DOCX, PPTX, HTML, Confluence export, SharePoint export) and use AI to detect which EA framework(s) the document is relevant to, extracting candidate model elements tagged to framework-specific entity types (e.g., DM2 Operational Activity, Zachman R2C1 entity, FEAF BRM sub-function). | AI/ML | P-AI, P-DA | Must | Core automation value |
| REQ-FWKA-002 | AI-Assisted Zachman Cell Population | The system SHALL provide an AI workflow that, given a set of ingested documents, drafts content for each Zachman cell with confidence scores, allowing architects to review, edit, accept, or reject each cell draft in a structured triage UI. | AI/ML | P-AI, P-EA | Should | Zachman population acceleration |
| REQ-FWKA-003 | AI Cross-Framework Gap Detection | The system SHALL use AI to analyze the current state of a multi-framework project and generate a Gap Report identifying: (a) required framework models not yet created, (b) mapped elements in one framework with no counterpart in the mapped framework, (c) semantic inconsistencies detected by NLP comparison. | AI/ML | P-AI, P-EA | Should | AI governance value |
| REQ-FWKA-004 | AI Framework Selection Recommendation | The system SHALL integrate AI-based recommendations into the Framework Selection Wizard (REQ-FWKX-001), using project context embeddings to rank frameworks and generate rationale text, with the reasoning visible to users. | AI/ML | P-EA | Should | Intelligent wizard |
| REQ-FWKA-005 | Source Traceability for AI Outputs | The system SHALL ensure every AI-generated model element includes a citation to the originating document, section, and paragraph from which it was inferred, enabling human review and audit. | AI/ML | P-AUD, P-AI | Must | AI auditability |

---

## 8. Reporting Requirements

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-FWKR-001 | Framework Coverage Dashboard | The system SHALL provide a unified Framework Coverage Dashboard showing, for each active framework in a project, the percentage of required models that are: not started, in progress, completed, and approved. | Reporting | P-EA, P-CIO | Must | Program governance |
| REQ-FWKR-002 | Traceability Report | The system SHALL generate an end-to-end Traceability Report for a multi-framework project showing the traceability chain from strategic goals (Zachman R1C6 / CV-1 / FEAF PRM) through capabilities (CV-2 / StV-2 / UAF) through operational activities (OV-5 / UAF Op) through systems and services (SV/SvcV) to technical standards (StdV / TRM), with coverage percentages and gap indicators. | Reporting | P-EA, P-AUD | Must | End-to-end traceability |
| REQ-FWKR-003 | Framework Compliance Report | The system SHALL generate per-framework compliance reports (DoDAF completeness per program, FEAF coverage per agency, NAF v4 grid completeness per NATO project) formatted for submission to relevant governance authorities. | Reporting | P-AUD, P-CIO | Must | Regulatory submission |
| REQ-FWKR-004 | Cross-Framework Consistency Report | The system SHALL generate a Cross-Framework Consistency Report listing all detected semantic inconsistencies, duplicate concepts, and unmapped equivalents across active frameworks, with severity ratings and recommended remediation. | Reporting | P-EA, P-AUD | Must | Governance health |
| REQ-FWKR-005 | Executive Summary Auto-Generation | The system SHALL auto-generate an Executive Summary report for each framework, combining AI-extracted highlights from populated views into a structured narrative (Purpose → Scope → Key Findings → Gaps → Recommendations), exportable as PDF and PPTX. | Reporting | P-CIO, P-EA | Should | Executive communication |

---

## Appendix A: Framework-View-to-Requirement Index

| Framework | View/Product | Primary Requirement(s) |
|-----------|-------------|------------------------|
| Zachman | R1–R6 × C1 (What) | REQ-FWK-010..015 |
| Zachman | R1–R6 × C2 (How) | REQ-FWK-020..025 |
| Zachman | R1–R6 × C3 (Where) | REQ-FWK-030..035 |
| Zachman | R1–R6 × C4 (Who) | REQ-FWK-040..045 |
| Zachman | R1–R6 × C5 (When) | REQ-FWK-050..055 |
| Zachman | R1–R6 × C6 (Why) | REQ-FWK-060..065 |
| Zachman | Cell linkage & consistency | REQ-FWK-070..077 |
| DoDAF | AV-1, AV-2 | REQ-FWKD-010..013 |
| DoDAF | CV-1..7 | REQ-FWKD-020..027 |
| DoDAF | OV-1..6c | REQ-FWKD-030..039 |
| DoDAF | SV-1..11 | REQ-FWKD-040..053 |
| DoDAF | SvcV-1..10c | REQ-FWKD-060..070 |
| DoDAF | StdV-1..2 | REQ-FWKD-080..083 |
| DoDAF | DIV-1..3 | REQ-FWKD-090..093 |
| DoDAF | PV-1..3 | REQ-FWKD-100..103 |
| DoDAF | Cross-view analysis | REQ-FWKD-110..114 |
| MODAF | StV-1..7 | REQ-FWKM-010..016 |
| MODAF | AcV-1..2 | REQ-FWKM-020..022 |
| MODAF | SOV-1..5 | REQ-FWKM-030..032 |
| NAF v4 | C1–C8, L1–L8, P1–P8, A1–A5 | REQ-FWKM-040..047 |
| UAF | Domain/Aspect grid | REQ-FWKU-001..004 |
| UAF | Viewpoints by aspect | REQ-FWKU-010..017 |
| UAF | SysML integration | REQ-FWKU-020..024 |
| FEAF | PRM | REQ-FWKF-010..011 |
| FEAF | BRM | REQ-FWKF-020..022 |
| FEAF | DRM | REQ-FWKF-030..032 |
| FEAF | IRM/TRM | REQ-FWKF-040..042 |
| FEAF | SRM | REQ-FWKF-050..051 |
| FEAF | Integrated view | REQ-FWKF-060..062 |
| Cross-Framework | Selection wizard | REQ-FWKX-001..004 |
| Cross-Framework | Viewpoint libraries | REQ-FWKX-010..013 |
| Cross-Framework | Model interoperability | REQ-FWKX-020..027 |
| Cross-Framework | Format exports | REQ-FWKX-030..038 |
| Cross-Framework | NFR/Security/Governance | REQ-FWKX-050..057 |
| AI/ML | Framework-specific AI | REQ-FWKA-001..005 |
| Reporting | Dashboards and reports | REQ-FWKR-001..005 |

---

## Appendix B: Boundary Notes and Assumptions

1. **Volume 03 (ArchiMate + Diagramming):** The diagram rendering engine, ArchiMate metamodel, viewpoint diagram templates, and graphical editor are governed by V03. This volume governs only the mapping tables from ArchiMate element types to Zachman cells and DoDAF views (REQ-FWKX-020, -022, -037).

2. **Volume 01 (TOGAF/ADM):** TOGAF ADM phase-by-phase deliverable requirements and Architecture Development Method are in V01. This volume governs only the TOGAF↔Zachman cell mapping (REQ-FWKX-023) and TOGAF↔ArchiMate→DoDAF bridging.

3. **Volume 06 (AI):** AI model selection, prompt engineering, RAG pipeline, LLM orchestration, and inference budgets are governed by V06. This volume governs the framework taxonomy, entity type registry, and attribution data model that AI pipelines must use.

4. **Volume 10 (Deliverables):** Formal deliverable document templates (Word, PDF, slide decks) for DoDAF products, MODAF views, and FEAF reports are governed by V10. This volume governs the data and export format requirements.

5. **DM2 Assumption:** Requirements referencing the DoDAF Meta-Model (DM2) assume conformance to the published DM2 UML class model (DoDAF 2.02, Volume 2). Any DM2 extensions for UAF alignment follow the OMG UAF 1.1 normative specification.

6. **MODAF/NAF Distinction:** MODAF v1.2.004 and NAF v4 are treated as distinct frameworks. MODAF is the UK MOD bilateral framework; NAF v4 is the multilateral NATO successor. The reconciliation requirements (REQ-FWKM-046) bridge them explicitly.

7. **FEAF Gartner Note:** The Gartner EA framework and BIAN framework are noted at REQ-FWKF-062 as extensibility targets via the framework adapter pattern, not as first-class implementations in this version.

8. **Security Classification:** Requirements referencing SECRET-level data (REQ-FWKX-052) are conditional on deployment on IL4/IL5/IL6-authorized infrastructure and are labeled "Must" only for such deployments; standard commercial deployments treat this as "Should".

---

*End of Volume 02 — ESIP Architecture Automation Suite Requirements Catalog*  
*Total requirements this volume: 214*  
*Prefix: REQ-FWK (base), REQ-FWKD (DoDAF), REQ-FWKM (MODAF/NAF), REQ-FWKU (UAF), REQ-FWKF (FEAF), REQ-FWKX (Cross-Framework), REQ-FWKA (AI), REQ-FWKR (Reporting)*
