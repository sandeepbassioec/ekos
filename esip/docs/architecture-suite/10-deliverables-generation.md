# Volume 10 — Deliverables Generation, Export & Publishing

**Suite:** ESIP Architecture Automation Suite  
**Volume:** 10 of 11  
**Prefix:** DLV  
**Status:** Draft v1.0  
**Date:** 2026-06-23  
**Boundary volumes:** V01 (TOGAF deliverable definitions) · V02 (framework view products) · V03 (diagramming/rendering engine) · V06 (AI content drafting) · V11 (platform templates, branding, governance)

---

## Table of Contents

1. [Scope & Purpose](#1-scope--purpose)
2. [Capability Areas Overview](#2-capability-areas-overview)
3. [Per-Framework Deliverable Templates](#3-per-framework-deliverable-templates)
   - 3.1 TOGAF Deliverables
   - 3.2 Zachman Cell Artifacts
   - 3.3 DoDAF / MODAF / NAF / UAF View Products
   - 3.4 ArchiMate View Sets
   - 3.5 Solution Architecture (SA) Deliverables
   - 3.6 Per-Role Deliverables
4. [Document Generation Engines](#4-document-generation-engines)
   - 4.1 Word / .docx Engine
   - 4.2 PowerPoint / .pptx Engine
   - 4.3 Excel / .xlsx Engine
   - 4.4 PDF Engine
   - 4.5 Markdown Engine
   - 4.6 HTML Engine
   - 4.7 Cross-Engine Shared Capabilities
5. [Template Management](#5-template-management)
6. [Diagram Export](#6-diagram-export)
7. [Architecture Portal & Publishing](#7-architecture-portal--publishing)
8. [Presentation & Reporting](#8-presentation--reporting)
9. [Quality, Compliance & Accessibility](#9-quality-compliance--accessibility)
10. [Integration & Platform Requirements](#10-integration--platform-requirements)
11. [Requirements Summary Table](#11-requirements-summary-table)

---

## 1. Scope & Purpose

Volume 10 specifies all requirements for the **Deliverable Factory** layer of the ESIP Architecture Automation Suite. This layer transforms structured architecture content (models, catalogs, matrices, AI-generated narratives, imported diagrams) into polished, publishable output artefacts consumable by enterprise stakeholders at every level — from hands-on architects to C-suite boards.

The factory extends ESIP's existing generators (SheetJS for `.xlsx`, pptxgenjs for `.pptx`, jsPDF for `.pdf`) into a unified, template-driven, multi-format, multi-framework pipeline.

**In scope:**
- Template-driven document generation across all supported formats
- Per-framework deliverable composition (TOGAF, Zachman, DoDAF/MODAF/NAF/UAF, ArchiMate, SA)
- Per-role output packages
- Template library management and versioning
- Diagram export (raster, vector, interchange formats)
- Architecture portal and stakeholder publishing
- Executive presentation and scheduled reporting
- Deliverable quality gates, accessibility, and redaction

**Out of scope (boundary):**
- Diagram rendering logic → V03
- AI narrative/content drafting → V06
- Deliverable *definitions* and catalogue entries → V01, V02
- Corporate branding assets, governance policy, and platform theming → V11
- Requirements modeling and traceability → V07
- Knowledge Hub content management → V08
- Portfolio dashboards → V09

---

## 2. Capability Areas Overview

| # | Capability Area | Req Range | Count |
|---|---|---|---|
| 3 | Per-Framework Deliverable Templates | REQ-DLV-001 – REQ-DLV-095 | 95 |
| 4 | Document Generation Engines | REQ-DLV-100 – REQ-DLV-169 | 70 |
| 5 | Template Management | REQ-DLV-170 – REQ-DLV-209 | 40 |
| 6 | Diagram Export | REQ-DLV-210 – REQ-DLV-249 | 40 |
| 7 | Architecture Portal & Publishing | REQ-DLV-250 – REQ-DLV-309 | 60 |
| 8 | Presentation & Reporting | REQ-DLV-310 – REQ-DLV-359 | 50 |
| 9 | Quality, Compliance & Accessibility | REQ-DLV-360 – REQ-DLV-399 | 40 |
| 10 | Integration & Platform | REQ-DLV-400 – REQ-DLV-419 | 20 |
| **Total** | | | **415** |

---

## 3. Per-Framework Deliverable Templates

### 3.1 TOGAF Deliverables

TOGAF ADM produces a defined set of formal deliverables per phase. The suite SHALL support template-driven auto-generation of each, populated from the repository.

---

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-001 | TOGAF Architecture Vision Document | The system SHALL generate a complete Architecture Vision document (ADM Phase A output) comprising executive summary, problem statement, capability assessment, architecture vision narrative, key stakeholder concerns mapped to drivers, and a high-level architecture diagram placeholder referencing V03. | Functional | Enterprise Architect, Architecture Sponsor | Must | TOGAF 10 §16.4; Phase A deliverable |
| REQ-DLV-002 | Architecture Vision — Stakeholder Map Table | The system SHALL auto-populate the stakeholder register table within the Architecture Vision from the project's stakeholder catalog, including name, role, concerns, and viewpoints assigned. | Functional | Enterprise Architect | Must | TOGAF stakeholder management |
| REQ-DLV-003 | Architecture Vision — Value Proposition Matrix | The system SHALL embed a value proposition matrix cross-referencing strategic drivers against proposed architecture capabilities within the Architecture Vision document. | Functional | Enterprise Architect, Business Architect | Should | TOGAF Phase A outcomes |
| REQ-DLV-004 | Architecture Definition Document (ADD) — Business Architecture Chapter | The system SHALL generate the Business Architecture chapter of the ADD, including business capability maps, organization maps, actor/role catalog tables, business service catalog, and process flow diagram placeholders. | Functional | Business Architect, Enterprise Architect | Must | TOGAF Phase B |
| REQ-DLV-005 | Architecture Definition Document — Data/Information Architecture Chapter | The system SHALL generate the Information Architecture chapter of the ADD, including data entity catalog, logical data model diagram placeholder, data governance and classification notes, and canonical data standards references. | Functional | Data Architect, Enterprise Architect | Must | TOGAF Phase C (Information) |
| REQ-DLV-006 | Architecture Definition Document — Application Architecture Chapter | The system SHALL generate the Application Architecture chapter of the ADD, including application catalog, application interaction matrix, interface catalog, and application diagram placeholders. | Functional | Solution Architect, Application Architect | Must | TOGAF Phase C (Application) |
| REQ-DLV-007 | Architecture Definition Document — Technology Architecture Chapter | The system SHALL generate the Technology Architecture chapter of the ADD, including technology standards catalog, technology portfolio matrix, infrastructure component catalog, and technology landscape diagram placeholder. | Functional | Infrastructure Architect, Enterprise Architect | Must | TOGAF Phase D |
| REQ-DLV-008 | Architecture Definition Document — Full Assembly | The system SHALL assemble all ADD chapters (Business, Information, Application, Technology) into a single navigable document with auto-generated cross-references, a unified table of contents, and consistent section numbering. | Functional | Enterprise Architect | Must | TOGAF 10 §17 |
| REQ-DLV-009 | Architecture Requirements Specification (ARS) | The system SHALL generate an Architecture Requirements Specification document containing gap analysis tables (baseline vs. target per domain), transition architecture requirements, compliance requirements, and a requirements traceability matrix (RTM) sourced from V07. | Functional | Enterprise Architect, Requirements Manager | Must | TOGAF Phase E/F |
| REQ-DLV-010 | ARS — Gap Analysis Table | The system SHALL auto-populate gap analysis tables per architecture domain (Business/Data/Application/Technology) by diffing baseline and target architecture elements stored in the repository. | Functional | Enterprise Architect | Must | TOGAF gap analysis |
| REQ-DLV-011 | Architecture Roadmap | The system SHALL generate an Architecture Roadmap document containing a work package catalog, work breakdown by transition architecture, Gantt-style timeline placeholder, and migration sequencing rationale. | Functional | Enterprise Architect, Program Manager | Must | TOGAF Phase F |
| REQ-DLV-012 | Architecture Roadmap — Work Package Table | The system SHALL auto-populate the work package table within the Architecture Roadmap from the project's work package catalog, including ID, name, description, estimated effort, dependencies, and owning architect. | Functional | Enterprise Architect | Must | TOGAF Phase F output |
| REQ-DLV-013 | Architecture Roadmap — Migration Sequencing | The system SHALL generate a migration sequencing narrative and tabular summary showing the logical ordering of transition architectures and the rationale for sequencing decisions. | Functional | Enterprise Architect, Program Manager | Should | TOGAF Phase F |
| REQ-DLV-014 | Architecture Contract | The system SHALL generate an Architecture Contract document covering architecture principles, conformance requirements, SLAs and quality criteria, governance checkpoints, and sign-off blocks. | Functional | Enterprise Architect, Architecture Board | Must | TOGAF Phase G |
| REQ-DLV-015 | Architecture Contract — Compliance Checklist | The system SHALL embed a compliance checklist within the Architecture Contract, auto-populated from the architecture standards and principles catalog, with pass/fail/waiver fields. | Functional | Architecture Board | Must | TOGAF governance |
| REQ-DLV-016 | Implementation & Governance Model (IGM) | The system SHALL generate an Implementation & Governance Model document specifying architecture governance structure, change request process, compliance review process, and architecture review board charter. | Functional | Enterprise Architect, Governance Lead | Must | TOGAF Phase G/H |
| REQ-DLV-017 | IGM — Governance Process Diagrams | The system SHALL include BPMN/flowchart diagram placeholders within the IGM for key governance processes (change request, compliance review, ADR escalation), referencing the V03 rendering engine. | Functional | Governance Lead | Should | TOGAF IGM |
| REQ-DLV-018 | Capability Assessment Report | The system SHALL generate a Capability Assessment Report listing current capabilities against a configurable maturity model (e.g., CMMI, TOGAF maturity), with heat-map table and narrative gap commentary. | Functional | Business Architect, Enterprise Architect | Must | TOGAF Phase A/B |
| REQ-DLV-019 | Capability Assessment — Maturity Heat Map Table | The system SHALL auto-render a color-coded maturity heat map table within the Capability Assessment Report, derived from maturity scores stored in the capability catalog. | Functional | Business Architect | Must | Capability planning |
| REQ-DLV-020 | Statement of Architecture Work (SoAW) | The system SHALL generate a Statement of Architecture Work document covering scope, approach, schedule, resources, constraints, and acceptance criteria, with merge fields for sponsor and architect sign-off. | Functional | Enterprise Architect, Architecture Sponsor | Must | TOGAF Phase A |
| REQ-DLV-021 | SoAW — Resource & Schedule Table | The system SHALL auto-populate the resource and schedule table in the SoAW from project metadata (team members, phases, target dates) held in the project registry. | Functional | Enterprise Architect | Should | TOGAF SoAW |
| REQ-DLV-022 | Architecture Principles Catalog Document | The system SHALL generate a standalone Architecture Principles document listing all principles with ID, name, statement, rationale, implications, and conflict-resolution notes. | Functional | Enterprise Architect | Must | TOGAF principles |
| REQ-DLV-023 | Transition Architecture Document | The system SHALL generate a Transition Architecture document per defined transition state, including delta tables showing elements added/removed/modified relative to the baseline architecture. | Functional | Enterprise Architect, Solution Architect | Should | TOGAF Phase F |
| REQ-DLV-024 | Architecture Change Request Document | The system SHALL generate a templated Architecture Change Request document, auto-filled with the change description, impact assessment, affected architecture elements, and review/approval workflow status. | Functional | Enterprise Architect, Governance Lead | Must | TOGAF Phase H |

---

### 3.2 Zachman Cell Artifacts

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-025 | Zachman Cell Artifact Export | The system SHALL generate a document or spreadsheet artifact for any individual Zachman cell (row × column intersection), populating it with all repository elements tagged to that cell. | Functional | Enterprise Architect, Business Analyst | Must | Zachman Framework |
| REQ-DLV-026 | Zachman Full Matrix Report | The system SHALL generate a full Zachman Matrix report as a structured document or formatted spreadsheet showing all 36 cells with summary content and links/references to detailed cell artifacts. | Functional | Enterprise Architect | Should | Zachman Framework |
| REQ-DLV-027 | Zachman Column Pack — What (Data) | The system SHALL generate a column artifact pack for the "What" (Data) column across all six rows (Scope, Business Concept, System Logic, Technology, Component, Instance), assembling entity lists, logical models, physical models, and runtime schemas. | Functional | Data Architect | Should | Zachman Framework |
| REQ-DLV-028 | Zachman Column Pack — How (Function) | The system SHALL generate a column artifact pack for the "How" (Function) column, assembling business processes, application functions, system modules, and deployed services. | Functional | Business Architect, Solution Architect | Should | Zachman Framework |
| REQ-DLV-029 | Zachman Row Pack — Planner/Scope Row | The system SHALL generate a Planner Row artifact pack documenting scope definitions (business vocabulary, context diagrams, scope lists) for each column. | Functional | Enterprise Architect | Could | Zachman Framework |
| REQ-DLV-030 | Zachman Row Pack — Designer/System Logic Row | The system SHALL generate a Designer Row artifact pack compiling logical models, process decompositions, entity-relationship diagrams, and network topology for the system logic perspective. | Functional | Solution Architect, Data Architect | Could | Zachman Framework |

---

### 3.3 DoDAF / MODAF / NAF / UAF View Products

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-031 | DoDAF AV-1 Overview & Summary Document | The system SHALL generate an AV-1 Overview and Summary Information document from project metadata, including description of architecture, assumptions, constraints, and key findings. | Functional | Enterprise Architect (Defence/Gov) | Must | DoDAF 2.02 AV-1 |
| REQ-DLV-032 | DoDAF AV-2 Integrated Dictionary | The system SHALL generate an AV-2 Integrated Dictionary document, auto-populated from the project glossary and term catalog, with hyperlinked cross-references. | Functional | Enterprise Architect | Must | DoDAF 2.02 AV-2 |
| REQ-DLV-033 | DoDAF OV-1 High-Level Operational Concept | The system SHALL generate an OV-1 document containing the operational concept narrative, stakeholder context, and an operational concept diagram placeholder referencing V03. | Functional | Operations Architect | Must | DoDAF OV-1 |
| REQ-DLV-034 | DoDAF OV-2 Operational Resource Flow | The system SHALL generate an OV-2 document with resource flow tables and a corresponding diagram placeholder showing nodes and needlines. | Functional | Operations Architect | Must | DoDAF OV-2 |
| REQ-DLV-035 | DoDAF SV-1 Systems Interface Description | The system SHALL generate an SV-1 document listing system nodes, systems, and system interfaces from the application/system catalog, with an interface matrix table. | Functional | Systems Architect | Must | DoDAF SV-1 |
| REQ-DLV-036 | DoDAF SV-4 Systems Functionality Description | The system SHALL generate an SV-4 document describing system functions and functional hierarchy, auto-populated from the capability/function catalog. | Functional | Systems Architect | Should | DoDAF SV-4 |
| REQ-DLV-037 | DoDAF TV-1 Technical Standards Profile | The system SHALL generate a TV-1 Technical Standards Profile document listing applicable standards, protocols, and profiles from the technology standards catalog. | Functional | Technology Architect | Must | DoDAF TV-1 |
| REQ-DLV-038 | MODAF StV-1 Strategic Capability Concept | The system SHALL generate a StV-1 document describing strategic capabilities, derived from the capability catalog and strategic driver registry. | Functional | Enterprise Architect (Defence) | Should | MODAF StV-1 |
| REQ-DLV-039 | NAF Chapter View Products | The system SHALL generate NAF v4 chapter-aligned view product documents (C1–C8, L1–L4, Lr, P1–P8, S1–S5, A1–A5, Ar, Grid) using catalog data and V03 diagram placeholders. | Functional | Enterprise Architect (NATO/Defence) | Should | NAF v4 |
| REQ-DLV-040 | UAF Domain Viewpoint Documents | The system SHALL generate UAF domain viewpoint documents (Operational, Services, Structure, Connectivity, Personnel, Resources, Security, Projects, Standards, Actual Resources, Measures, Summary/Overview) from UAF-tagged repository elements. | Functional | Enterprise Architect (SysML/UAF) | Should | UAF 1.1 |
| REQ-DLV-041 | Cross-Framework View Mapping Appendix | The system SHALL auto-generate a cross-framework mapping appendix document that maps equivalent views across DoDAF, MODAF, NAF, UAF, TOGAF, and Zachman, for inclusion in any of the above deliverables. | Functional | Enterprise Architect | Could | Multi-framework traceability |

---

### 3.4 ArchiMate View Sets

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-042 | ArchiMate Motivation View Document | The system SHALL generate a Motivation View document listing stakeholders, drivers, assessments, goals, requirements, and principles from the motivation layer catalog. | Functional | Enterprise Architect | Must | ArchiMate 3.2 |
| REQ-DLV-043 | ArchiMate Organization Map Document | The system SHALL generate an Organization Map document from the business layer actor/role/collaboration catalog with relationship tables and diagram placeholder. | Functional | Business Architect | Must | ArchiMate 3.2 |
| REQ-DLV-044 | ArchiMate Application Cooperation View Document | The system SHALL generate an Application Cooperation View document listing application components, interfaces, and interactions with a diagram placeholder. | Functional | Application Architect | Must | ArchiMate 3.2 |
| REQ-DLV-045 | ArchiMate Technology Infrastructure View Document | The system SHALL generate a Technology Infrastructure View document from the technology layer catalog including nodes, devices, system software, and communication paths. | Functional | Infrastructure Architect | Must | ArchiMate 3.2 |
| REQ-DLV-046 | ArchiMate Layered View Summary | The system SHALL generate a Layered View summary document condensing all three layers (Business, Application, Technology) with cross-layer relationship highlights. | Functional | Enterprise Architect | Should | ArchiMate 3.2 |
| REQ-DLV-047 | ArchiMate Migration View Document | The system SHALL generate a Migration View document tabulating implementation and migration elements (work packages, deliverables, plateaus, gaps) from the migration catalog. | Functional | Enterprise Architect, Program Manager | Should | ArchiMate 3.2 |
| REQ-DLV-048 | ArchiMate Physical View Document | The system SHALL generate a Physical View document for physical infrastructure elements (distribution networks, equipment, facilities, materials) when relevant to the engagement. | Functional | Infrastructure Architect | Could | ArchiMate 3.2 |

---

### 3.5 Solution Architecture (SA) Deliverables

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-049 | High-Level Design (HLD) Document | The system SHALL generate an HLD document comprising executive summary, solution overview, key architectural decisions, component architecture, integration points, non-functional requirements, and diagram placeholders for context, component, and deployment views. | Functional | Solution Architect | Must | SA practice standard |
| REQ-DLV-050 | HLD — Component Architecture Section | The system SHALL auto-populate the component architecture section of the HLD from the application/component catalog, including component descriptions, responsibilities, and interface summary tables. | Functional | Solution Architect | Must | SA HLD |
| REQ-DLV-051 | HLD — NFR Summary Table | The system SHALL embed an NFR summary table in the HLD, sourced from the requirements catalog (V07), covering performance, availability, scalability, security, and maintainability targets. | Functional | Solution Architect | Must | SA HLD |
| REQ-DLV-052 | Low-Level Design (LLD) Document | The system SHALL generate an LLD document covering detailed component specifications, sequence diagrams, data schema definitions, API contract summaries, configuration parameters, and deployment topology. | Functional | Solution Architect, Software Engineer | Must | SA practice standard |
| REQ-DLV-053 | LLD — API Contract Summary Section | The system SHALL auto-populate an API contract summary section in the LLD from the API catalog (OpenAPI/AsyncAPI references), including endpoints, methods, request/response schemas, and authentication mechanism. | Functional | Solution Architect, API Designer | Must | SA LLD |
| REQ-DLV-054 | LLD — Data Schema Section | The system SHALL auto-populate the data schema section of the LLD from the data model catalog, including entity definitions, attribute tables, relationship descriptions, and primary/foreign key annotations. | Functional | Data Architect, Solution Architect | Must | SA LLD |
| REQ-DLV-055 | Architecture Decision Record (ADR) | The system SHALL generate individual ADR documents in a standard format (Status, Context, Decision, Consequences, Alternatives Considered, Decision Drivers) from the ADR catalog entries. | Functional | Solution Architect, Enterprise Architect | Must | ADR practice (Nygard pattern) |
| REQ-DLV-056 | ADR Register / Index Document | The system SHALL generate an ADR Register document listing all ADRs for the project with status, date, decision title, related components, and links to full ADR documents. | Functional | Solution Architect | Must | SA governance |
| REQ-DLV-057 | Solution Overview / One-Pager | The system SHALL generate a concise Solution Overview document (1–3 pages) summarizing the solution's purpose, key components, primary integrations, and strategic fit, targeted at executive/stakeholder audiences. | Functional | Solution Architect, Executive Sponsor | Must | Stakeholder communication |
| REQ-DLV-058 | Options Paper / Architecture Options Analysis | The system SHALL generate an Options Paper document presenting at minimum three architectural options with pros/cons tables, evaluation criteria, scoring matrix, and recommended option rationale. | Functional | Solution Architect, Enterprise Architect | Must | SA decision-making |
| REQ-DLV-059 | Options Paper — Decision Matrix Table | The system SHALL auto-populate the decision matrix table in the Options Paper from the evaluation criteria catalog and option assessments entered by the architect. | Functional | Solution Architect | Must | Options analysis |
| REQ-DLV-060 | API Specification Document | The system SHALL generate an API Specification document from OpenAPI 3.x and/or AsyncAPI 3.x source files, rendered as a readable Word/PDF with sections for overview, authentication, endpoints/operations, models, error codes, and examples. | Functional | API Designer, Solution Architect | Must | API governance |
| REQ-DLV-061 | Data Model Document | The system SHALL generate a Data Model document from the data catalog and/or imported ER/UML source, including a logical ERD diagram placeholder, entity definitions, attribute dictionaries, and relationship descriptions. | Functional | Data Architect | Must | Data governance |
| REQ-DLV-062 | Threat Model Report | The system SHALL generate a Threat Model Report from threat catalog entries (STRIDE or PASTA methodology), including asset list, threat actor catalog, threat scenario table, DREAD/CVSS risk scores, and mitigating controls table. | Functional | Security Architect | Must | Security by design |
| REQ-DLV-063 | Cloud Landing Zone Document | The system SHALL generate a Cloud Landing Zone document covering account/subscription structure, networking topology, IAM strategy, compliance controls, tagging/naming conventions, and baseline policy catalog, populated from cloud infrastructure catalog entries. | Functional | Cloud Architect, Infrastructure Architect | Must | Cloud adoption frameworks |
| REQ-DLV-064 | Cloud Landing Zone — Policy Catalog Appendix | The system SHALL include an appendix in the Cloud Landing Zone document auto-populated with the full policy catalog (SCPs, Azure Policy, OPA policies as applicable) referenced in the main document. | Functional | Cloud Architect | Should | CIS Benchmarks, CSP best practices |
| REQ-DLV-065 | Integration Architecture Document | The system SHALL generate an Integration Architecture document covering integration patterns, message catalog, event catalog, API gateway configuration, middleware topology, and error handling strategy. | Functional | Integration Architect, Solution Architect | Must | Enterprise integration patterns |
| REQ-DLV-066 | Security Architecture Document | The system SHALL generate a Security Architecture document covering security principles, zero-trust design, identity and access management architecture, data protection strategy, network security zones, detection & response approach, and compliance mapping. | Functional | Security Architect | Must | NIST CSF, ISO 27001 |
| REQ-DLV-067 | DevOps / CI-CD Architecture Document | The system SHALL generate a DevOps Architecture document covering branching strategy, pipeline stages, environment topology, artifact management, deployment patterns, observability architecture, and runbook references. | Functional | DevOps Architect, Solution Architect | Should | DevOps practices |
| REQ-DLV-068 | Runbook Document | The system SHALL generate a templated Runbook document for each deployable system component, covering operational procedures, health checks, alerting thresholds, escalation paths, and rollback procedures. | Functional | Platform Engineer, DevOps Architect | Should | Operational readiness |

---

### 3.6 Per-Role Deliverables

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-069 | Business Architect Pack | The system SHALL assemble a Business Architect deliverable pack containing: capability map, business process catalog, organizational model, value stream map, and business motivation model, formatted for a business stakeholder audience. | Functional | Business Architect | Must | Role-based output |
| REQ-DLV-070 | Data Architect Pack | The system SHALL assemble a Data Architect deliverable pack containing: data catalog, logical and physical data models, data lineage diagram placeholder, data governance framework summary, and data quality rules register. | Functional | Data Architect | Must | Role-based output |
| REQ-DLV-071 | Application Architect Pack | The system SHALL assemble an Application Architect deliverable pack containing: application portfolio catalog, integration matrix, application lifecycle table, technology stack summary, and application service map. | Functional | Application Architect | Must | Role-based output |
| REQ-DLV-072 | Infrastructure / Cloud Architect Pack | The system SHALL assemble an Infrastructure Architect deliverable pack containing: infrastructure topology, capacity plan summary, SLA register, cost optimization findings, and resiliency design summary. | Functional | Infrastructure Architect, Cloud Architect | Must | Role-based output |
| REQ-DLV-073 | Security Architect Pack | The system SHALL assemble a Security Architect deliverable pack containing: threat model report, security control catalog, compliance mapping table, identity architecture summary, and penetration test scope document. | Functional | Security Architect | Must | Role-based output |
| REQ-DLV-074 | Domain Architect Pack | The system SHALL generate role-specific domain packs for Data, Integration, API, and Platform architects, each containing the relevant subset of deliverables from the master catalog filtered by architect role assignment. | Functional | Domain Architect (any) | Should | Role-based output |
| REQ-DLV-075 | Stakeholder-Tailored Summary Pack | The system SHALL generate a stakeholder-tailored summary pack (1–5 pages per stakeholder group) containing only the architecture views and key messages relevant to that stakeholder group's concerns, derived from the stakeholder catalog. | Functional | Enterprise Architect | Must | Stakeholder communication |
| REQ-DLV-076 | Program Manager Deliverable Pack | The system SHALL assemble a Program Manager pack containing: architecture roadmap, work package catalog, transition architecture summary, and dependency map formatted for a project/program management audience. | Functional | Program Manager, Enterprise Architect | Should | PMO alignment |
| REQ-DLV-077 | Audit & Compliance Pack | The system SHALL assemble an Audit & Compliance pack containing the architecture compliance checklist, standards conformance matrix, ADR register, and governance sign-off log, formatted for audit and assurance audiences. | Functional | Governance Lead, Auditor | Must | Governance |
| REQ-DLV-078 | Innovation & Options Briefing | The system SHALL generate an Innovation and Options Briefing document summarising evaluated architectural options, experimental technologies assessed, and recommended innovation bets, targeted at CTO/CIO audience. | Functional | Enterprise Architect, CTO/CIO | Could | Strategic planning |

---

### 3.7 Deliverable Bundles & Packs

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-079 | ADM Phase Pack Assembly | The system SHALL bundle all deliverables required for a given ADM phase into a single downloadable ZIP archive, with a manifest file listing each artefact and its status (draft/reviewed/approved). | Functional | Enterprise Architect | Must | TOGAF ADM governance |
| REQ-DLV-080 | Multi-Framework Deliverable Bundle | The system SHALL support bundling deliverables from multiple frameworks (e.g., TOGAF + ArchiMate + SA) into a single release package with a cross-framework index document. | Functional | Enterprise Architect | Should | Multi-framework engagements |
| REQ-DLV-081 | Deliverable Status Tracking | The system SHALL maintain a deliverable status register (planned, in-progress, draft, under-review, approved, superseded) per deliverable instance, queryable and reportable. | Functional | Enterprise Architect, Governance Lead | Must | Governance |
| REQ-DLV-082 | Deliverable Version History | The system SHALL version every generated deliverable, retaining prior versions with generation date, generator version, and the snapshot of source data used at generation time. | Data | Enterprise Architect | Must | Audit trail |
| REQ-DLV-083 | Deliverable Regeneration on Source Change | The system SHALL detect when underlying catalog data or template content has changed since a deliverable was last generated and present a staleness indicator with a one-click regeneration option. | Functional | Enterprise Architect | Must | Content freshness |
| REQ-DLV-084 | Deliverable Dependency Map | The system SHALL maintain a dependency graph between deliverables (e.g., ARS depends on ADD, ADD depends on capability catalog) and surface the impact of upstream changes on downstream deliverables. | Functional | Enterprise Architect | Should | Traceability |
| REQ-DLV-085 | Approval Workflow Integration | The system SHALL support configurable multi-stage approval workflows (draft → review → approved) for each deliverable, with email/notification triggers and approval log embedded in the generated document. | Functional | Governance Lead, Architecture Board | Must | Governance |
| REQ-DLV-086 | Deliverable Catalog UI | The system SHALL provide a deliverable catalog UI listing all deliverable templates available for the current framework context, with per-deliverable status, last-generated date, and actions (Generate, Preview, Download, Publish). | UX | Enterprise Architect | Must | Discoverability |
| REQ-DLV-087 | Bulk Deliverable Generation | The system SHALL support bulk generation of all deliverables for a selected ADM phase, framework, or project scope in a single operation, with parallel processing and progress reporting. | Functional | Enterprise Architect | Should | Productivity |
| REQ-DLV-088 | Deliverable Diff View | The system SHALL provide a document-level diff view comparing two versions of the same deliverable, highlighting added, removed, and modified sections. | Functional | Enterprise Architect, Governance Lead | Should | Change management |
| REQ-DLV-089 | Deliverable Checklist per Framework | The system SHALL provide a pre-configured checklist of required deliverables per framework (TOGAF ADM phase, DoDAF project, etc.) and track completion status against that checklist. | Functional | Enterprise Architect | Must | Governance |
| REQ-DLV-090 | Deliverable RTM (Requirements Traceability Matrix) | The system SHALL generate an RTM appendix for any deliverable that references requirements from V07, mapping deliverable sections to requirement IDs and showing coverage percentage. | Functional | Enterprise Architect, Requirements Manager | Must | Traceability |
| REQ-DLV-091 | Deliverable Glossary Auto-Population | The system SHALL auto-populate a Glossary appendix in every generated deliverable from the project's term catalog, including only terms referenced in the document body. | Functional | Enterprise Architect | Should | Consistency |
| REQ-DLV-092 | Deliverable Abbreviations List | The system SHALL auto-generate an Abbreviations and Acronyms list in every deliverable, populated from the project term catalog filtered to abbreviation-type entries detected in the document. | Functional | Enterprise Architect | Should | Professionalism |
| REQ-DLV-093 | Deliverable Cover Page | The system SHALL generate a branded cover page for every deliverable including: project name, document title, version, date, classification, author, reviewer, and approver fields, using the active branding theme from V11. | Functional | Enterprise Architect | Must | Corporate standards |
| REQ-DLV-094 | Deliverable Header/Footer | The system SHALL apply consistent headers and footers to every generated document page including: document title (header), page number, classification marking, and confidentiality notice (footer), configurable per template. | Functional | Enterprise Architect | Must | Corporate standards |
| REQ-DLV-095 | Deliverable Revision History Table | The system SHALL embed a Revision History table at the front of every generated deliverable, auto-populated from the deliverable version history, listing version, date, author, and change summary. | Functional | Enterprise Architect | Must | Document management |

---

## 4. Document Generation Engines

### 4.1 Word / .docx Engine

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-100 | .docx Generation via docx.js | The system SHALL use the `docx` npm library (or equivalent FOSS alternative) to generate Word-compatible `.docx` files entirely in the browser or Node.js runtime without requiring a Microsoft Office installation. | Functional | All | Must | Technology constraint |
| REQ-DLV-101 | .docx Heading Hierarchy | The system SHALL apply Word heading styles (Heading 1–5) mapped to document section levels, enabling correct navigation pane rendering and auto-TOC generation in Microsoft Word. | Functional | All | Must | Word compatibility |
| REQ-DLV-102 | .docx Automatic Table of Contents | The system SHALL embed a pre-formatted Table of Contents field in `.docx` output that auto-updates when opened in Microsoft Word, or alternatively render a static TOC with page number references pre-computed at generation time. | Functional | All | Must | Professional documents |
| REQ-DLV-103 | .docx Cross-References | The system SHALL support numbered cross-references within `.docx` output (e.g., "see Figure 3.1", "refer to Table 4.2") using bookmarks and hyperlinks, with consistent numbering maintained at generation time. | Functional | Enterprise Architect | Must | Professional documents |
| REQ-DLV-104 | .docx Figure and Table Numbering | The system SHALL apply sequential figure and table captions with chapter-section numbering (e.g., Figure 2.3, Table 5.1) to all figures and tables in `.docx` output. | Functional | All | Must | Professional documents |
| REQ-DLV-105 | .docx Embedded Diagrams | The system SHALL embed diagrams exported from V03 as inline images (PNG/SVG-to-EMF) within `.docx` documents, with captions, alt-text, and configurable width/alignment. | Functional | All | Must | Integration with V03 |
| REQ-DLV-106 | .docx Embedded Tables from Catalogs | The system SHALL render data from any architecture catalog (application catalog, capability catalog, etc.) as formatted Word tables with styled header rows, alternating row shading, and column widths proportional to content. | Functional | All | Must | Catalog integration |
| REQ-DLV-107 | .docx Style Application | The system SHALL apply a corporate Word style set (Normal, Heading 1–5, Caption, Table Text, Code Block, Quote, Note) sourced from the active V11 branding theme to all generated `.docx` documents. | Functional | All | Must | Branding (ref V11) |
| REQ-DLV-108 | .docx Cover Page | The system SHALL insert a styled cover page as the first section of every `.docx` output, with project logo (from V11), title, version, date, classification, and author fields. | Functional | All | Must | Branding (ref V11) |
| REQ-DLV-109 | .docx Multi-Column Layouts | The system SHALL support multi-column layout sections within `.docx` output for catalog listings and comparison tables where horizontal space allows. | Functional | Enterprise Architect | Could | Layout fidelity |
| REQ-DLV-110 | .docx Tracked Changes / Review Mode | The system SHALL optionally produce `.docx` output with tracked changes enabled, annotating AI-drafted sections with author "ESIP AI" to distinguish generated content from manually authored content. | Functional | Enterprise Architect | Should | Review workflow |
| REQ-DLV-111 | .docx Comments / Annotations | The system SHALL support inserting review comments and annotations at configurable locations in `.docx` output, e.g., flagging incomplete sections or AI-generated content requiring human review. | Functional | Enterprise Architect | Should | Review workflow |
| REQ-DLV-112 | .docx Watermark | The system SHALL apply a configurable watermark (e.g., "DRAFT", "CONFIDENTIAL", "RESTRICTED") to `.docx` pages based on document classification and status settings. | Functional | Governance Lead | Must | Classification |
| REQ-DLV-113 | .docx Multi-Language Support | The system SHALL support generating `.docx` output in any configured locale, applying locale-specific date formats, number formats, and right-to-left paragraph direction where required. | NFR | All | Should | Internationalization |

---

### 4.2 PowerPoint / .pptx Engine

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-114 | .pptx Generation via pptxgenjs | The system SHALL use the existing pptxgenjs library integration to generate PowerPoint `.pptx` files, extending it with template-driven layout selection and per-deliverable slide masters. | Functional | All | Must | Existing integration |
| REQ-DLV-115 | .pptx Slide Masters / Layouts | The system SHALL support multiple slide master/layout configurations (title slide, content slide, two-column, table slide, diagram slide, section divider) switchable per-slide during generation. | Functional | All | Must | Presentation quality |
| REQ-DLV-116 | .pptx Agenda / Contents Slide | The system SHALL auto-generate an agenda slide at the beginning of every presentation, listing all sections with slide numbers, updated automatically when sections are added or reordered. | Functional | All | Must | Presentation standards |
| REQ-DLV-117 | .pptx Section Divider Slides | The system SHALL insert section divider slides at the start of each major section, matching the active slide master branding from V11. | Functional | All | Must | Presentation structure |
| REQ-DLV-118 | .pptx Speaker Notes | The system SHALL populate speaker notes for each slide from template note placeholders and/or AI-drafted talking points (from V06), formatted as plain text. | Functional | Enterprise Architect | Should | Presentation delivery |
| REQ-DLV-119 | .pptx Diagram Embedding | The system SHALL embed diagrams from V03 as high-resolution PNG or scalable EMF images in designated diagram slides, with captions and source attribution. | Functional | All | Must | V03 integration |
| REQ-DLV-120 | .pptx Table Slides | The system SHALL generate table slides with catalog data rendered as formatted PowerPoint tables with branded header rows and alternating row shading. | Functional | All | Must | Data presentation |
| REQ-DLV-121 | .pptx Chart Slides | The system SHALL generate chart slides (bar, column, pie, doughnut, radar) from quantitative catalog data (maturity scores, portfolio counts, gap severities) using pptxgenjs chart types. | Functional | Enterprise Architect, Portfolio Manager | Should | Executive reporting |
| REQ-DLV-122 | .pptx Animation/Transition Control | The system SHALL allow template configuration to enable or disable slide transitions and progressive build animations, defaulting to no animation for professional/board decks and allowing animated builds for presentation decks. | Functional | All | Could | Presentation flexibility |
| REQ-DLV-123 | .pptx Executive Summary Slide | The system SHALL auto-generate an executive summary slide with key findings, decisions, and recommendations, populated from the project's decision log and AI summary (V06). | Functional | Enterprise Architect | Must | Executive communication |

---

### 4.3 Excel / .xlsx Engine

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-124 | .xlsx Generation via SheetJS | The system SHALL use the existing SheetJS integration to generate Excel `.xlsx` files, extending it with multi-sheet workbook support, named ranges, and formula embedding. | Functional | All | Must | Existing integration |
| REQ-DLV-125 | .xlsx Multi-Sheet Workbooks | The system SHALL generate multi-sheet workbooks with one sheet per catalog type (e.g., applications, capabilities, requirements, interfaces) within a single deliverable workbook. | Functional | Enterprise Architect, Data Analyst | Must | Catalog export |
| REQ-DLV-126 | .xlsx Catalog Export — All Types | The system SHALL support export of every catalog type (application, capability, component, interface, principle, requirement, risk, decision, standard, gap, work package) to a dedicated sheet with full field coverage. | Functional | Enterprise Architect | Must | Completeness |
| REQ-DLV-127 | .xlsx Styled Header Rows | The system SHALL apply branded header row styling (background color from V11 palette, bold white text, frozen header row) to all catalog export sheets. | Functional | All | Must | Branding |
| REQ-DLV-128 | .xlsx Auto-Filter | The system SHALL apply Excel auto-filter to all catalog export sheets, enabling column-level filtering without additional user configuration. | Functional | All | Must | Usability |
| REQ-DLV-129 | .xlsx Conditional Formatting | The system SHALL apply conditional formatting rules to key columns (e.g., risk severity → red/amber/green, maturity level → color gradient, status → icon set) in catalog export sheets. | Functional | Enterprise Architect | Must | Data readability |
| REQ-DLV-130 | .xlsx Formula Embedding | The system SHALL embed Excel formulas in summary sheets (count of gaps by severity, total work packages by status, capability maturity average) that auto-update when source data sheets are refreshed. | Functional | Enterprise Architect | Should | Analytics |
| REQ-DLV-131 | .xlsx RTM Workbook | The system SHALL generate a dedicated Requirements Traceability Matrix workbook with sheets for requirements, architecture elements, and a cross-reference mapping sheet, sourced from V07. | Functional | Requirements Manager | Must | Traceability |
| REQ-DLV-132 | .xlsx Data Validation | The system SHALL apply Excel data validation (dropdown lists for status, priority, severity fields) to editable catalog export sheets, supporting round-trip import after manual update. | Functional | Enterprise Architect | Should | Data quality |

---

### 4.4 PDF Engine

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-133 | PDF Generation via jsPDF + Extension | The system SHALL extend the existing jsPDF integration to support full-document PDF generation with multi-page layouts, headers/footers, page numbering, hyperlinks, and image embedding. | Functional | All | Must | Existing integration |
| REQ-DLV-134 | PDF-UA Accessibility | The system SHALL generate tagged PDF output conforming to PDF/UA-1 (ISO 14289-1), including document structure tags, alt text for images, reading order, and accessible table markup. | NFR | All | Must | Accessibility |
| REQ-DLV-135 | PDF Bookmarks / Navigation | The system SHALL generate PDF bookmarks (Outline panel) reflecting the document heading hierarchy, enabling in-reader navigation of large architecture documents. | Functional | All | Must | Usability |
| REQ-DLV-136 | PDF Hyperlinks | The system SHALL embed active hyperlinks for all internal cross-references, external URL citations, and diagram source links within PDF output. | Functional | All | Must | Navigation |
| REQ-DLV-137 | PDF Cover Page and Watermark | The system SHALL apply the same cover page and watermark (classification, draft status) as the `.docx` engine to PDF output for visual consistency. | Functional | All | Must | Consistency |
| REQ-DLV-138 | PDF Redaction | The system SHALL support selective redaction of designated classified/sensitive sections or fields within PDF output before publishing, with a visible redaction marker (black bar + "REDACTED" label). | Security | Governance Lead, Security Architect | Must | Information security |
| REQ-DLV-139 | PDF Page Size / Orientation | The system SHALL support configurable page size (A4, Letter, A3, Legal) and per-section page orientation (portrait/landscape) within a single PDF document, e.g., for wide table or diagram sections. | Functional | All | Must | Layout flexibility |
| REQ-DLV-140 | PDF Print-Ready | The system SHALL generate print-ready PDF output with embedded fonts, CMYK color space option, and crop/bleed marks for professional printing of formal architecture documents. | NFR | Governance Lead | Could | Print quality |

---

### 4.5 Markdown Engine

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-141 | Markdown Generation | The system SHALL generate well-structured Markdown (CommonMark-compatible) output for all deliverable templates, suitable for rendering in GitHub, GitLab, Confluence, and static site generators (Hugo, MkDocs, Docusaurus). | Functional | Software/Platform Architect | Must | Developer toolchain |
| REQ-DLV-142 | Markdown YAML Front Matter | The system SHALL prefix generated Markdown documents with YAML front matter containing document metadata (title, version, date, author, tags, classification) compatible with major static site generators. | Functional | Software Architect | Must | SSG compatibility |
| REQ-DLV-143 | Markdown Mermaid Diagrams | The system SHALL embed Mermaid diagram code blocks within Markdown output for flow, sequence, class, ER, and Gantt views, generated from the V03 diagram model, enabling in-platform rendering on GitHub/GitLab. | Functional | Software/Platform Architect | Must | V03 integration |
| REQ-DLV-144 | Markdown Table Generation | The system SHALL generate GitHub-Flavored Markdown (GFM) pipe tables from all catalog data, with correct column alignment markers and escaped special characters. | Functional | All | Must | GFM compatibility |
| REQ-DLV-145 | Markdown ADR Format | The system SHALL generate ADRs in the standard Markdown ADR format (Context/Decision/Consequences/Status header pattern) compatible with the `adr-tools` convention. | Functional | Solution Architect | Must | ADR toolchain |
| REQ-DLV-146 | Markdown Navigation Index | The system SHALL generate a navigable Markdown index file (e.g., `README.md` or `INDEX.md`) for every deliverable set, listing all generated files with relative links. | Functional | All | Should | Navigation |

---

### 4.6 HTML Engine

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-147 | HTML Microsite Generation | The system SHALL generate a self-contained HTML microsite for any deliverable set, with a navigation sidebar, search index, and responsive layout, deployable as a static site. | Functional | Enterprise Architect | Must | Architecture portal base |
| REQ-DLV-148 | HTML Embedded Interactive Diagrams | The system SHALL embed interactive SVG diagrams (pan/zoom/click-through) within HTML deliverable output, referencing V03 SVG exports. | Functional | Enterprise Architect | Must | V03 integration |
| REQ-DLV-149 | HTML Searchable Content | The system SHALL include a client-side full-text search index (lunr.js or equivalent) in HTML microsite output, enabling stakeholders to search architecture content without a server. | Functional | All | Must | Accessibility/usability |
| REQ-DLV-150 | HTML Print Stylesheet | The system SHALL include a print CSS stylesheet with HTML output, enabling clean A4/Letter-page printing of architecture documents from a browser. | NFR | All | Should | Printing |
| REQ-DLV-151 | HTML WCAG 2.2 AA Compliance | The system SHALL generate HTML deliverable output conforming to WCAG 2.2 Level AA, including semantic markup, skip-navigation links, ARIA labels, sufficient color contrast, and keyboard navigability. | NFR | All | Must | Accessibility |
| REQ-DLV-152 | HTML Anchor Deep-Links | The system SHALL generate stable anchor IDs for every section and named element in HTML output, enabling shareable deep-links to specific sections. | Functional | All | Must | Shareability |

---

### 4.7 Cross-Engine Shared Capabilities

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-153 | Multi-Format Simultaneous Export | The system SHALL support generating the same deliverable in multiple formats simultaneously (e.g., .docx + PDF + Markdown) in a single generation operation, sharing the same source data pass. | Functional | All | Must | Productivity |
| REQ-DLV-154 | Section Composition Engine | The system SHALL implement a section composition engine that assembles document sections from discrete section templates (cover, TOC, executive summary, chapters, appendices) in a configurable order defined by the deliverable template. | Functional | Enterprise Architect | Must | Template flexibility |
| REQ-DLV-155 | Conditional Section Rendering | The system SHALL support conditional section inclusion based on template metadata conditions (e.g., include Cloud Landing Zone section only if cloud_platform is set; include Threat Model only if security_review = true). | Functional | Enterprise Architect | Must | Context sensitivity |
| REQ-DLV-156 | Merge Field / Variable Substitution | The system SHALL support merge field substitution ({{project_name}}, {{architecture_phase}}, {{author_name}}, etc.) across all output formats, with merge field values drawn from project metadata. | Functional | All | Must | Personalization |
| REQ-DLV-157 | Cross-Reference Auto-Resolution | The system SHALL resolve cross-references between deliverables (e.g., "see Architecture Vision, Section 3.2") to hyperlinks in HTML/PDF output and bookmarks in .docx output. | Functional | Enterprise Architect | Should | Document linking |
| REQ-DLV-158 | Generation Progress Reporting | The system SHALL provide real-time progress reporting during document generation (section-level progress bar, estimated completion time, error/warning log) for long-running multi-section documents. | UX | All | Must | User experience |
| REQ-DLV-159 | Generation Error Handling | The system SHALL handle generation errors gracefully (missing data, unsupported diagram type, template parse error) by inserting a placeholder section with an error notice, continuing generation of remaining sections, and surfacing a post-generation error summary. | NFR | All | Must | Resilience |
| REQ-DLV-160 | Character Encoding | The system SHALL generate all output in UTF-8 encoding (or UTF-16 for .docx/.pptx/.xlsx OOXML compliance), correctly handling non-ASCII characters including European diacritics, CJK ideographs, and Arabic/Hebrew text. | NFR | All | Must | Internationalization |
| REQ-DLV-161 | Document Size Limits | The system SHALL handle documents up to 500 pages / 50 MB without browser memory errors, using streaming or chunked generation where browser memory constraints apply. | NFR | All | Must | Scalability |
| REQ-DLV-162 | Generation Audit Log | The system SHALL record a generation audit log entry for every generated deliverable, capturing: timestamp, user, deliverable type, template version, source data snapshot hash, output file name, and output size. | Governance | Governance Lead | Must | Audit |

---

## 5. Template Management

### 5.1 Template Library

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-170 | Built-In Template Library | The system SHALL ship with a built-in template library covering all deliverable types defined in Section 3 (TOGAF, Zachman, DoDAF/MODAF/NAF/UAF, ArchiMate, SA, per-role), for each of the supported output formats (.docx, .pptx, .xlsx, PDF, Markdown, HTML). | Functional | All | Must | Out-of-box usability |
| REQ-DLV-171 | Template Catalog UI | The system SHALL provide a template catalog UI listing all available templates with metadata: framework, deliverable type, output format, version, last updated, and a preview thumbnail. | UX | Enterprise Architect | Must | Discoverability |
| REQ-DLV-172 | Template Preview | The system SHALL render a live preview of any template populated with sample/placeholder data, showing the visual output (paged PDF-style or HTML render) before generation with real data. | UX | Enterprise Architect | Must | WYSIWYG confidence |
| REQ-DLV-173 | Template Cloning | The system SHALL allow any built-in or custom template to be cloned and saved as a new custom template, preserving all section structure and styles while enabling independent modification. | Functional | Enterprise Architect | Must | Customization |
| REQ-DLV-174 | Template Import | The system SHALL support importing custom `.docx` / `.pptx` files as master templates, extracting their style definitions and placeholder structure for use in the generation engine. | Functional | Enterprise Architect | Should | Corporate templates |
| REQ-DLV-175 | Template Export | The system SHALL allow any template (built-in or custom) to be exported as a standalone bundle (JSON definition + asset files) for backup, sharing, or migration between ESIP instances. | Functional | Enterprise Architect | Should | Portability |

---

### 5.2 Template Authoring

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-176 | Visual Template Editor (WYSIWYG) | The system SHALL provide a WYSIWYG template editor for `.docx`-style and `.pptx`-style templates, enabling non-developer architects to modify section layouts, fonts, colors, and placeholder positions without writing code. | UX | Enterprise Architect | Must | Self-service |
| REQ-DLV-177 | Placeholder / Merge Field Definition | The system SHALL allow template authors to define named merge fields ({{field_name}}) with metadata: field type (text, date, table, diagram, list), source binding (catalog field, project metadata, AI draft), and fallback value. | Functional | Enterprise Architect | Must | Template flexibility |
| REQ-DLV-178 | Conditional Section Rules | The system SHALL allow template authors to define IF/ELSE conditional rules on sections (e.g., include "Cloud Architecture" section IF cloud_deployment = true), using a simple rule builder UI. | Functional | Enterprise Architect | Must | Context sensitivity |
| REQ-DLV-179 | Repeating Section / Loop Constructs | The system SHALL support loop constructs in templates that iterate over catalog collections (e.g., one sub-section per application, one table row per requirement) to auto-expand content at generation time. | Functional | Enterprise Architect | Must | Data-driven content |
| REQ-DLV-180 | Section Ordering UI | The system SHALL allow template authors to drag-and-drop reorder sections within a template and set each section as required, optional, or conditional. | UX | Enterprise Architect | Must | Template authoring |
| REQ-DLV-181 | Template Validation | The system SHALL validate templates on save, checking for undefined merge fields, circular section references, invalid conditional expressions, and missing required sections, surfacing errors with contextual guidance. | Functional | Enterprise Architect | Must | Quality |
| REQ-DLV-182 | Code-Level Template Editing | The system SHALL expose a code editor view for each template's underlying JSON/YAML definition, enabling developer-level customization alongside the visual editor with round-trip sync. | Functional | Platform Engineer | Should | Power user access |
| REQ-DLV-183 | Template Section Reuse (Partials) | The system SHALL support shared section partials (e.g., standard cover page, revision history, abbreviations list) reusable across multiple templates, updating centrally when the partial is modified. | Functional | Enterprise Architect | Must | DRY principle |

---

### 5.3 Template Versioning & Governance

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-184 | Template Versioning | The system SHALL version every template with semantic version numbers (MAJOR.MINOR.PATCH), retaining the full version history and allowing rollback to any prior version. | Governance | Governance Lead | Must | Template governance |
| REQ-DLV-185 | Template Approval Workflow | The system SHALL support a configurable approval workflow for new or modified templates (draft → review → approved), with email/notification triggers and an approval audit log. | Governance | Governance Lead | Should | Template governance |
| REQ-DLV-186 | Template Deprecation | The system SHALL support marking templates as deprecated with a migration notice pointing to the replacement template, preventing new deliverable generation from deprecated templates while retaining access to prior deliverables. | Governance | Governance Lead | Must | Template lifecycle |
| REQ-DLV-187 | Default Template Assignment | The system SHALL allow administrators to assign default templates per deliverable type and framework combination, applied automatically when a new deliverable is generated without explicit template selection. | Functional | Administrator | Must | Usability |
| REQ-DLV-188 | Branding Theme Binding | The system SHALL bind templates to a corporate branding theme (colors, fonts, logos, header/footer assets) sourced from V11, with theme updates automatically propagating to all templates bound to that theme at next generation. | Functional | Administrator | Must | Branding (ref V11) |
| REQ-DLV-189 | Template Usage Analytics | The system SHALL track template usage analytics (generation count, average document size, error rate, user) per template, surfacing this data in an admin dashboard. | Reporting | Administrator | Should | Adoption monitoring |
| REQ-DLV-190 | Template Sharing (Org-Wide) | The system SHALL support publishing custom templates org-wide so all users and projects can access them, with role-based write-protection preventing unauthorized modification. | Functional | Administrator | Should | Reuse |

---

### 5.4 Branding and Theming

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-191 | Corporate Branding Themes | The system SHALL support multiple named corporate branding themes, each defining: primary/secondary/accent colors, font family and sizes per style, logo (header/cover), page margins, and header/footer layout. | Functional | Administrator | Must | Branding (ref V11) |
| REQ-DLV-192 | Per-Project Branding Override | The system SHALL allow per-project overrides of the active branding theme (e.g., applying a client-specific theme for consulting deliverables) without modifying the org-level theme. | Functional | Enterprise Architect | Must | Client branding |
| REQ-DLV-193 | Dark / Light Theme for HTML Output | The system SHALL generate HTML deliverable output supporting both light and dark themes with a user-togglable theme switch, applying appropriate color schemes in CSS custom properties. | UX | All | Should | Modern UX |
| REQ-DLV-194 | High-Contrast Theme | The system SHALL provide a high-contrast branding theme option for HTML output meeting WCAG 2.2 Level AA contrast ratios for all text and interactive elements. | NFR | All | Must | Accessibility |
| REQ-DLV-195 | Logo Placement Options | The system SHALL support configurable logo placement on cover pages (top-left, top-center, top-right) and in headers (left or right), with adjustable size and padding. | Functional | Administrator | Should | Branding flexibility |

---

### 5.5 Multi-Language

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-196 | Template Locale Configuration | The system SHALL associate each template with a locale (language + region), applying locale-specific formatting for dates, numbers, currency, and list ordering. | NFR | All | Must | Internationalization |
| REQ-DLV-197 | Translated Template Variants | The system SHALL support storing translated variants of any template (same structure, different language strings for section headings, boilerplate, and labels), selectable at generation time. | Functional | Enterprise Architect | Should | Multi-language enterprises |
| REQ-DLV-198 | RTL Layout Support | The system SHALL support right-to-left (RTL) document layout for Arabic and Hebrew locales in `.docx` and HTML output, including bidirectional text handling and mirrored page margins. | NFR | All | Could | Internationalization |
| REQ-DLV-199 | Multi-Language Glossary | The system SHALL support project glossaries with terms in multiple languages, embedding the locale-appropriate glossary in each generated deliverable based on the selected locale. | Functional | Enterprise Architect | Could | Internationalization |
| REQ-DLV-200 | Output Locale Metadata | The system SHALL embed locale metadata (language tag per ISO 639-1, region per ISO 3166-1) in generated document properties (`.docx` XML, PDF XMP metadata, HTML lang attribute) for screen reader and spell-checker compatibility. | NFR | All | Must | Accessibility |

---

### 5.6 Template Variables & Advanced Composition

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-201 | Global Variables | The system SHALL support org-level and project-level global variables (e.g., `{{org_name}}`, `{{classification_default}}`, `{{fiscal_year}}`) injectable into any template across all deliverables. | Functional | Administrator | Must | Consistency |
| REQ-DLV-202 | Computed Variables | The system SHALL support computed variables in templates that calculate values at generation time (e.g., `{{gap_count}}` = count of gaps in scope, `{{current_date_formatted}}`). | Functional | Enterprise Architect | Should | Dynamic content |
| REQ-DLV-203 | Catalog Filter Expressions | The system SHALL allow template authors to define catalog filter expressions (e.g., "include only capabilities with maturity < 3") that scope data dynamically within repeating sections. | Functional | Enterprise Architect | Must | Data-driven documents |
| REQ-DLV-204 | Section-Level Metadata | The system SHALL allow template authors to annotate sections with metadata (owner persona, review required flag, AI-draft flag, mandatory/optional) visible in the ESIP UI but suppressed in the output document. | Functional | Enterprise Architect | Should | Authoring governance |
| REQ-DLV-205 | Template Test/Preview with Live Data | The system SHALL allow template authors to preview the template with live project data (not just sample data) before finalising, showing actual catalog content in the rendered preview. | UX | Enterprise Architect | Must | Template validation |
| REQ-DLV-206 | Template Inheritance | The system SHALL support template inheritance, where a child template extends a parent template (inheriting global sections and styles) and overrides or extends specific sections without duplicating shared content. | Functional | Platform Engineer | Should | Template DRY |
| REQ-DLV-207 | Template Snapshot on Generation | The system SHALL snapshot the exact template definition (version + content) used at the time of generating each deliverable, storing it with the deliverable record so the document can be exactly reproduced. | Governance | Governance Lead | Must | Reproducibility |
| REQ-DLV-208 | Template Change Impact Analysis | The system SHALL analyze the impact of a template change by listing all deliverables generated from that template that would be affected, allowing administrators to decide whether to regenerate or grandfather existing deliverables. | Governance | Governance Lead | Should | Change management |
| REQ-DLV-209 | Template Marketplace / Community | The system SHALL provide a template marketplace UI where users can browse, rate, and import community-contributed templates (with admin approval gate for org-published templates). | Functional | Enterprise Architect | Could | Community |

---

## 6. Diagram Export

> **Cross-reference note (reconciliation).** Ownership of *raw diagram export formats* (the diagram-interchange/vector/raster file exports themselves — Visio `.vsdx`, draw.io XML, XMI, ArchiMate Exchange/AMEFF, Mermaid, PlantUML) sits with **V03 (REQ-MOD-333..338)**, which is the diagramming/rendering engine. The requirements in §6 below (REQ-DLV-210..) are the **deliverable-factory consumers** of those V03 exports — embedding/packaging diagrams into generated documents and portals — not a second implementation of the exporters; treat any apparent duplication as "invoke the V03 exporter." The **V19 ENT-LOCK** offline/air-gapped export path likewise *references* the V03 export formats rather than redefining them. **V10's own scope is styled-document generation only** (`.docx` / `.pptx` / `.xlsx` / PDF — see §4). See 00-MASTER-INDEX §6.

### 6.1 Raster Export

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-210 | PNG Export | The system SHALL export any diagram from V03 as a PNG raster image at user-specified resolution (default 150 dpi screen; options: 300 dpi print, 600 dpi high-resolution), with configurable width in pixels. | Functional | All | Must | Universal format |
| REQ-DLV-211 | PNG Transparent Background | The system SHALL support exporting PNG diagrams with a transparent background, enabling overlay on branded document backgrounds without white boxing. | Functional | All | Must | Document embedding |
| REQ-DLV-212 | JPEG Export | The system SHALL support JPEG export for photographic-quality presentations where smaller file size is preferred over transparency, with configurable quality level (60–100%). | Functional | All | Should | File size |
| REQ-DLV-213 | WebP Export | The system SHALL support WebP export for HTML deliverable embedding, offering superior compression vs PNG/JPEG for web-rendered architecture diagrams. | Functional | Platform Engineer | Could | Web performance |

---

### 6.2 Vector Export

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-214 | SVG Export | The system SHALL export any diagram as clean SVG, with all shape geometry, text, and style information preserved as valid XML, importable into drawing tools and embeddable in HTML. | Functional | All | Must | Scalable format |
| REQ-DLV-215 | SVG with Embedded Fonts | The system SHALL optionally embed (or outline) fonts in SVG export to prevent font substitution when SVG files are opened outside the generating environment. | Functional | All | Should | Portability |
| REQ-DLV-216 | SVG Minification | The system SHALL offer an SVG minification option that removes editor metadata, redundant attributes, and whitespace to reduce file size by ≥40% for web publishing. | NFR | Platform Engineer | Could | Web performance |
| REQ-DLV-217 | PDF Diagram Export | The system SHALL export any diagram as a standalone PDF page, suitable for inclusion in professional print packages or standalone diagram distribution. | Functional | Enterprise Architect | Must | Printing |

---

### 6.3 Architecture Interchange Formats

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-218 | Visio / .vsdx Export | The system SHALL export diagrams to Microsoft Visio `.vsdx` format, preserving shape groups, connection topology, and shape data properties, enabling continued editing in Visio by stakeholders who do not use ESIP. | Integration | Enterprise Architect | Must | Enterprise toolchain |
| REQ-DLV-219 | draw.io / .drawio XML Export | The system SHALL export diagrams to draw.io (diagrams.net) XML format, enabling round-trip editing in draw.io and importing into Confluence/Atlassian toolchains. | Integration | Solution Architect | Must | Atlassian toolchain |
| REQ-DLV-220 | Lucidchart Export | The system SHALL support export to Lucidchart-compatible format (CSV shape import or Lucidchart JSON) for teams using Lucidchart as their primary diagramming tool. | Integration | Enterprise Architect | Could | Toolchain |
| REQ-DLV-221 | XMI Export | The system SHALL export UML-structured diagrams (class, sequence, component, deployment, use case) to XMI 2.5 format (Eclipse UML2 profile), enabling import into enterprise modeling tools (Sparx EA, MagicDraw, Rhapsody). | Integration | Solution Architect, Systems Architect | Must | EA tool interop |
| REQ-DLV-222 | ArchiMate Exchange Format Export | The system SHALL export ArchiMate diagrams and model data to the ArchiMate Exchange Format (Open Exchange, OEF) as defined by The Open Group, enabling import into Archi, Sparx EA, and other ArchiMate-compliant tools. | Integration | Enterprise Architect | Must | ArchiMate interop |
| REQ-DLV-223 | BPMN 2.0 XML Export | The system SHALL export BPMN process diagrams to BPMN 2.0 XML format, enabling import into Camunda, Signavio, Bizagi, and other BPMN-compliant process engines. | Integration | Business Architect | Must | BPMN toolchain |
| REQ-DLV-224 | PlantUML Source Export | The system SHALL export UML-type diagrams to PlantUML source text, enabling developers to include diagrams as code in documentation pipelines and CI/CD systems. | Integration | Solution Architect | Should | Docs-as-code |
| REQ-DLV-225 | Mermaid Source Export | The system SHALL export flow, sequence, Gantt, ER, and class diagrams to Mermaid source syntax for embedding in Markdown and GitHub/GitLab rendering. | Integration | Software Architect | Must | Docs-as-code |
| REQ-DLV-226 | C4 Model DSL Export | The system SHALL export C4 Model diagrams to Structurizr DSL format, enabling import into Structurizr and consistent C4 model documentation. | Integration | Solution Architect | Should | C4 toolchain |

---

### 6.4 Batch and Advanced Diagram Export

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-227 | Batch Diagram Export | The system SHALL support batch export of all diagrams within a project or selected subset to a chosen format, packaged into a ZIP archive with a manifest listing diagram IDs, names, and file names. | Functional | Enterprise Architect | Must | Bulk operations |
| REQ-DLV-228 | Selective Layer Export | The system SHALL allow export of diagrams with selected layers visible/hidden, enabling targeted stakeholder views (e.g., exporting only business layer from an ArchiMate full-stack diagram). | Functional | Enterprise Architect | Must | Stakeholder tailoring |
| REQ-DLV-229 | Diagram Export with Metadata | The system SHALL embed diagram metadata (title, version, author, creation date, project reference) in all exported raster and vector files (EXIF/XMP for PNG/JPEG, `<metadata>` element for SVG, document properties for PDF). | Data | Enterprise Architect | Must | Asset management |
| REQ-DLV-230 | Diagram Watermark on Export | The system SHALL apply an optional watermark (classification, draft status, organization name) to exported diagram images, configurable per-export or via project settings. | Security | Governance Lead | Must | Classification |
| REQ-DLV-231 | High-DPI / Retina Export | The system SHALL generate 2x and 3x resolution PNG variants for any diagram on demand, supporting high-DPI/Retina display rendering in HTML deliverables. | NFR | All | Should | Display quality |
| REQ-DLV-232 | Diagram Version Export | The system SHALL allow export of any historical version of a diagram (from V03 version history), not only the current version, for comparison or audit purposes. | Functional | Enterprise Architect | Should | Audit |
| REQ-DLV-233 | Clipping / Region Export | The system SHALL support exporting a user-defined rectangular region of a large diagram canvas as a separate image or PDF, useful for embedding partial views in slide decks. | Functional | Enterprise Architect | Could | Composition |
| REQ-DLV-234 | Diagram Asset Library Export | The system SHALL support exporting the full diagram asset library (stencil shapes, custom icons, color palettes) as a reusable pack (JSON + SVG icons) for use in external drawing tools. | Functional | Platform Engineer | Could | Toolchain |

---

### 6.5 Visio / External Tool Import

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-235 | Visio .vsdx Import | The system SHALL import existing `.vsdx` Visio files, converting shape topology and connections into V03 diagram elements, preserving shape data as element properties. | Integration | Enterprise Architect | Must | Existing assets |
| REQ-DLV-236 | draw.io XML Import | The system SHALL import `.drawio` XML files into V03 diagram elements, enabling migration of existing architecture diagrams. | Integration | Enterprise Architect | Must | Existing assets |
| REQ-DLV-237 | ArchiMate OEF Import | The system SHALL import ArchiMate Exchange Format (OEF) files from Archi and Sparx EA, populating the V03 model and repository catalogs. | Integration | Enterprise Architect | Must | Existing assets |
| REQ-DLV-238 | XMI Import | The system SHALL import XMI 2.5 files (Eclipse UML2 / Sparx EA XMI) to populate UML diagram elements and class/component catalogs. | Integration | Solution Architect, Systems Architect | Should | Existing assets |
| REQ-DLV-239 | BPMN 2.0 XML Import | The system SHALL import BPMN 2.0 XML process definitions, rendering them as editable BPMN diagrams in V03 and populating the process catalog. | Integration | Business Architect | Should | Existing assets |
| REQ-DLV-240 | PlantUML Import | The system SHALL parse PlantUML source text and render the corresponding diagram type in V03, importing sequence, class, component, and deployment diagrams. | Integration | Software Architect | Could | Docs-as-code import |

---

## 7. Architecture Portal & Publishing

### 7.1 Portal Infrastructure

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-250 | Architecture Portal | The system SHALL provide an Architecture Portal — a web-based, read-only site where published architecture deliverables, diagrams, and catalog views are accessible to authorized stakeholders without requiring an ESIP author account. | Functional | All Stakeholders | Must | Stakeholder communication |
| REQ-DLV-251 | Portal Static Site Generation | The system SHALL generate the Architecture Portal as a static HTML site deployable to any static hosting provider (GitHub Pages, Azure Static Web Apps, AWS S3/CloudFront, Netlify, internal web server) without server-side runtime dependencies. | NFR | Platform Engineer | Must | Deployment flexibility |
| REQ-DLV-252 | Portal Live (Auto-Refresh) Mode | The system SHALL support a "Live" portal mode that automatically refreshes published content whenever underlying architecture data or diagrams are updated, with a configurable refresh interval (minimum 5 minutes, default 60 minutes). | Functional | Enterprise Architect | Must | Content freshness |
| REQ-DLV-253 | Portal Snapshot Mode | The system SHALL support a "Snapshot" portal mode that publishes a point-in-time immutable view of all architecture content, tagged with a version label and publication date, for formal review or audit purposes. | Functional | Governance Lead | Must | Governance |
| REQ-DLV-254 | Portal Versioned Publications | The system SHALL maintain a version history of portal publications, enabling stakeholders to navigate to any prior snapshot and compare it to the current version. | Data | Governance Lead | Must | Audit trail |
| REQ-DLV-255 | Portal Custom Domain | The system SHALL support configuration of a custom domain (e.g., `architecture.company.com`) for the Architecture Portal, with HTTPS enforced via configurable TLS certificate. | NFR | Platform Engineer | Should | Enterprise branding |
| REQ-DLV-256 | Portal Branding | The system SHALL apply the active V11 corporate branding theme (logo, colors, fonts) to the Architecture Portal, with a configurable portal name, tagline, and hero image on the landing page. | Functional | Administrator | Must | Branding |
| REQ-DLV-257 | Portal Navigation Structure | The system SHALL generate a hierarchical navigation structure for the Architecture Portal based on the project's deliverable hierarchy (framework → phase → deliverable → section), with breadcrumb navigation and persistent left sidebar. | UX | All Stakeholders | Must | Navigation |
| REQ-DLV-258 | Portal Full-Text Search | The system SHALL provide a full-text search capability across all published portal content, with results ranked by relevance and filtered by framework, deliverable type, or date. | Functional | All Stakeholders | Must | Discoverability |
| REQ-DLV-259 | Portal Mobile Responsiveness | The system SHALL render the Architecture Portal responsively across desktop (≥1280px), tablet (768–1279px), and mobile (320–767px) screen widths, maintaining readability of all diagrams and tables. | NFR | All Stakeholders | Must | Accessibility |
| REQ-DLV-260 | Portal Offline Access | The system SHALL support progressive web app (PWA) capabilities for the Architecture Portal, enabling stakeholders to access recently visited deliverables offline via browser service worker caching. | NFR | All Stakeholders | Could | Mobile use cases |

---

### 7.2 Access Control

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-261 | Portal Authentication | The system SHALL support authentication for the Architecture Portal via configurable identity providers: SSO/SAML 2.0, OpenID Connect, Azure AD/Entra ID, Google Workspace, and optional email-link magic-link authentication. | Security | Administrator | Must | Enterprise security |
| REQ-DLV-262 | Portal Public / Private Mode | The system SHALL support configuring individual portal publications as fully public (no authentication required), private (authentication required), or restricted (specific user groups only). | Security | Administrator | Must | Access control |
| REQ-DLV-263 | Role-Based Portal Access | The system SHALL enforce role-based access control on portal content, with configurable roles (Executive, Stakeholder, Reviewer, Auditor) mapped to content visibility rules (e.g., Executives see summary views only; Auditors see all plus compliance packs). | Security | Administrator | Must | Information security |
| REQ-DLV-264 | Content-Level Classification Enforcement | The system SHALL suppress or redact content elements marked as classified above the requesting user's clearance level when rendering portal pages, with visible placeholder indicating redacted content. | Security | Governance Lead | Must | Classification |
| REQ-DLV-265 | Shareable Link with Expiry | The system SHALL generate shareable deep-links to specific portal pages or deliverables with configurable expiry (1 day, 7 days, 30 days, no-expiry) and optional password protection for external stakeholder access. | Security | Enterprise Architect | Must | Stakeholder sharing |
| REQ-DLV-266 | Portal Access Audit Log | The system SHALL log all access events on the Architecture Portal (user, timestamp, page, document downloaded, search query) to an immutable audit log accessible to administrators. | Security | Administrator | Must | Audit |

---

### 7.3 Stakeholder-Tailored Views

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-267 | Stakeholder Persona Views | The system SHALL generate stakeholder-persona-specific views of the Architecture Portal, surfacing only the deliverables, views, and catalog sections relevant to each stakeholder group's defined concerns and viewpoints. | Functional | Enterprise Architect | Must | Stakeholder management |
| REQ-DLV-268 | Executive View | The system SHALL provide a dedicated executive portal view presenting: architecture vision summary, key decisions, roadmap timeline, top risks, and capability heat map — with no deep technical content. | Functional | Executive Stakeholder | Must | C-suite communication |
| REQ-DLV-269 | Technical Deep-Dive View | The system SHALL provide a technical portal view with full access to HLD/LLD, ADRs, API specs, data models, and infrastructure diagrams, filtered to the technology stack and components relevant to the accessing architect's domain. | Functional | Solution/Domain Architect | Must | Technical stakeholders |
| REQ-DLV-270 | Compliance / Audit View | The system SHALL provide a compliance portal view listing architecture principles, governance decisions, compliance matrices, risk registers, and formal approval records for audit and assurance audiences. | Functional | Auditor, Governance Lead | Must | Audit |
| REQ-DLV-271 | Business Stakeholder View | The system SHALL provide a business stakeholder portal view focusing on capability maps, business process diagrams, organizational models, value streams, and strategic roadmaps, excluding technology implementation details. | Functional | Business Stakeholder | Must | Business communication |
| REQ-DLV-272 | Custom View Builder | The system SHALL allow architects to compose and save custom portal views by selecting a subset of deliverables, diagrams, and catalog sections, assigning the view to a named stakeholder group or individual. | Functional | Enterprise Architect | Should | Tailoring |
| REQ-DLV-273 | View Embedding | The system SHALL support embedding individual portal views or deliverable sections as iframes in external portals (SharePoint, Confluence, ServiceNow) via an embeddable snippet with scoped authentication token. | Integration | Enterprise Architect | Should | Enterprise portal integration |

---

### 7.4 Comments, Feedback & Collaboration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-274 | In-Line Comments on Published Content | The system SHALL allow authenticated portal users to add inline comments on specific paragraphs, tables, or diagram elements within published deliverables, visible to other portal users in that access group. | Functional | All Stakeholders | Must | Stakeholder review |
| REQ-DLV-275 | Comment Threading | The system SHALL support threaded comment conversations (reply, resolve, reopen) on any published content element, with timestamps, user attribution, and a resolved/open status indicator. | Functional | All Stakeholders | Must | Review workflow |
| REQ-DLV-276 | Comment Notification | The system SHALL send email/in-app notifications to document owners and tagged users when a new comment, reply, or resolution is added to a published deliverable. | Functional | Enterprise Architect | Must | Collaboration |
| REQ-DLV-277 | Comment Export | The system SHALL allow export of all comments on a deliverable as a structured review log (PDF or CSV), suitable for inclusion in formal review records. | Functional | Governance Lead | Should | Governance |
| REQ-DLV-278 | Feedback Voting | The system SHALL allow portal users to upvote/thumbs-up specific comments to surface prioritized feedback to document owners, supporting efficient stakeholder review management. | Functional | All Stakeholders | Could | Prioritized feedback |
| REQ-DLV-279 | Portal Change Notifications | The system SHALL allow stakeholders to subscribe to change notifications for specific deliverables or catalog sections, receiving email/in-app alerts when content is updated and republished. | Functional | All Stakeholders | Must | Stakeholder engagement |

---

### 7.5 Publishing Workflow

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-280 | Publish Action | The system SHALL provide an explicit "Publish to Portal" action for any deliverable or deliverable bundle, triggering portal content regeneration and notifying subscribed stakeholders. | Functional | Enterprise Architect | Must | Publishing workflow |
| REQ-DLV-281 | Staged / Draft Publishing | The system SHALL support publishing deliverables to a "staging" portal environment for review before promoting to the production portal, with a one-click promote action. | Functional | Enterprise Architect | Should | Review workflow |
| REQ-DLV-282 | Scheduled Publishing | The system SHALL allow architects to schedule portal publications for a future date/time (e.g., publish simultaneously with a board meeting), with automatic execution and a pre-publish reminder notification. | Functional | Enterprise Architect | Should | Release management |
| REQ-DLV-283 | Unpublish / Archive | The system SHALL support unpublishing deliverables from the portal (removing them from active view while retaining in the archive) and archiving entire portal snapshots for long-term retention. | Functional | Governance Lead | Must | Content lifecycle |
| REQ-DLV-284 | Publication Log | The system SHALL maintain a publication log recording every portal publication event (who published, what was published, timestamp, environment, version), accessible to administrators and governance roles. | Governance | Governance Lead | Must | Audit |
| REQ-DLV-285 | Portal Analytics | The system SHALL provide a portal analytics dashboard showing: page views, unique visitors, most-accessed deliverables, search query analysis, comment activity, and download counts, for the last 30/90/365 days. | Reporting | Administrator, Enterprise Architect | Should | Engagement monitoring |
| REQ-DLV-286 | SEO Metadata | The system SHALL embed appropriate SEO metadata (title, description, canonical URL, Open Graph tags, sitemap.xml, robots.txt) in portal HTML output for intranet and internal search indexing. | NFR | Platform Engineer | Should | Discoverability |
| REQ-DLV-287 | Portal API | The system SHALL expose a read-only REST API for the Architecture Portal, enabling external systems (ServiceNow, Confluence, ITSM tools) to retrieve published deliverable metadata and content programmatically. | Integration | Platform Engineer | Should | Ecosystem integration |
| REQ-DLV-288 | Multi-Project Portal | The system SHALL support a multi-project portal mode displaying deliverables from multiple ESIP projects in a unified portal, with clear project-level navigation and cross-project search. | Functional | Enterprise Architect | Should | Portfolio visibility |
| REQ-DLV-289 | Portal Download Centre | The system SHALL provide a Download Centre section of the portal where authenticated users can download packaged deliverable bundles (ZIP archives of related documents) with download counts tracked. | Functional | All Stakeholders | Must | Access |

---

## 8. Presentation & Reporting

### 8.1 Executive Presentations

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-310 | Executive Architecture Deck | The system SHALL generate a fully branded executive architecture deck (`.pptx`) covering: vision, problem statement, strategic options, recommended architecture, roadmap, risks, and investment summary — auto-populated from the project's repository data and AI-drafted narratives (V06). | Functional | Enterprise Architect | Must | C-suite communication |
| REQ-DLV-311 | Board Report | The system SHALL generate a Board Report document (`.docx` and PDF) presenting architecture outcomes at board level, with: investment overview, strategic alignment, risk summary, key decisions, and one-page capability heat map. | Functional | Enterprise Architect, CIO | Must | Board governance |
| REQ-DLV-312 | Architecture Review Board (ARB) Pack | The system SHALL generate an ARB pack (`.pptx` + `.docx` appendix) for formal architecture review, including: work item summary, ADR index, compliance checklist, waiver requests, and risk register. | Functional | Enterprise Architect, Architecture Board | Must | Governance |
| REQ-DLV-313 | Architecture Status Report | The system SHALL generate a weekly/monthly Architecture Status Report (`.docx` and PDF) summarising: deliverable completion progress, open decisions, risks, and roadmap milestone status. | Reporting | Enterprise Architect, Program Manager | Must | Project governance |
| REQ-DLV-314 | Architecture Status Report Auto-Fields | The system SHALL auto-populate the Architecture Status Report with live data from deliverable status, decision log, and risk register, requiring zero manual data entry by the architect for standard fields. | Functional | Enterprise Architect | Must | Automation |
| REQ-DLV-315 | Options Analysis Brief | The system SHALL generate a concise Options Analysis Brief (2–4 page `.docx`) from the Options Paper data, suitable for quick executive review, with a single-page decision matrix and recommendation. | Functional | Enterprise Architect | Must | Communication |

---

### 8.2 Scheduled Report Generation & Distribution

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-316 | Scheduled Report Definition | The system SHALL allow architects to define scheduled reports specifying: report template, output format, delivery recipients (email list), delivery schedule (cron expression or natural language: daily, weekly, monthly, on ADM phase change), and source project scope. | Functional | Enterprise Architect | Must | Automation |
| REQ-DLV-317 | Report Schedule Execution | The system SHALL execute scheduled report generation at the defined schedule, generating the report from live project data at execution time, attaching the output file to an email, and delivering it to the configured recipient list. | Functional | Enterprise Architect | Must | Automation |
| REQ-DLV-318 | Report Schedule Management UI | The system SHALL provide a Schedule Management UI listing all active report schedules with status (active/paused), last execution time, next execution time, recipient count, and controls to edit, pause, resume, or delete. | UX | Enterprise Architect | Must | Manageability |
| REQ-DLV-319 | On-Demand Report Trigger | The system SHALL support on-demand triggering of any scheduled report definition outside of its schedule, useful for ad-hoc distribution to stakeholders before a meeting. | Functional | Enterprise Architect | Must | Flexibility |
| REQ-DLV-320 | Report Delivery Audit Log | The system SHALL log every report delivery event (schedule ID, report name, execution timestamp, recipients, output file name, delivery status per recipient), accessible to administrators. | Governance | Governance Lead | Must | Audit |
| REQ-DLV-321 | Report Delivery Failure Handling | The system SHALL detect delivery failures (SMTP bounce, email undeliverable) and retry delivery up to 3 times with exponential backoff, alerting the report owner if all retries fail. | NFR | Platform Engineer | Must | Reliability |
| REQ-DLV-322 | Report Email Template | The system SHALL support configurable email body templates for scheduled report delivery, supporting merge fields (report name, generation date, portal link, summary highlights). | Functional | Enterprise Architect | Should | Communication quality |
| REQ-DLV-323 | Report Subscription Self-Service | The system SHALL allow portal users to subscribe or unsubscribe from specific report distributions via a self-service preferences page on the Architecture Portal, subject to administrator-defined subscription access rules. | Functional | All Stakeholders | Must | Self-service |

---

### 8.3 Multi-Deliverable Bundles

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-324 | Bundle Definition | The system SHALL allow architects to define named deliverable bundles specifying: a set of deliverable types, output format per deliverable, archive format (ZIP/TAR), and optional inclusion of a bundle manifest document. | Functional | Enterprise Architect | Must | Package management |
| REQ-DLV-325 | Bundle Generation | The system SHALL generate all deliverables in a bundle definition in parallel, packaging the outputs into the configured archive with a manifest listing artefact names, types, generation dates, and checksums. | Functional | Enterprise Architect | Must | Efficiency |
| REQ-DLV-326 | Bundle Download | The system SHALL make generated bundles immediately available for download from the ESIP UI and optionally from the Architecture Portal Download Centre, with a configurable retention period before bundle expiry. | Functional | Enterprise Architect | Must | Access |
| REQ-DLV-327 | Bundle Email Distribution | The system SHALL support distributing generated bundles to a recipient list via email (bundle ZIP as attachment or as a secure download link), subject to file size limits (attachment ≤20 MB; link for larger). | Functional | Enterprise Architect | Should | Distribution |
| REQ-DLV-328 | Bundle Versioning | The system SHALL version every generated bundle with an auto-incremented bundle version, retaining prior bundle archives for the configured retention period (default 12 months). | Data | Governance Lead | Must | Audit |

---

### 8.4 Traceability Appendices

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-329 | Auto-Generated RTM Appendix | The system SHALL auto-generate a Requirements Traceability Matrix (RTM) appendix in any deliverable that references requirements from V07, showing: requirement ID, description, source, mapped architecture element, deliverable section, and verification method. | Functional | Enterprise Architect, Requirements Manager | Must | Traceability |
| REQ-DLV-330 | Decision Log Appendix | The system SHALL auto-generate a Decision Log appendix in any deliverable containing architecture decisions, listing all ADRs with status, date, and decision summary. | Functional | Enterprise Architect | Must | Governance |
| REQ-DLV-331 | Risk Register Appendix | The system SHALL auto-generate a Risk Register appendix in architecture reports and governance deliverables, populated from the risk catalog with: risk ID, description, likelihood, impact, risk score, owner, and mitigation. | Functional | Enterprise Architect, Risk Manager | Must | Risk governance |
| REQ-DLV-332 | Standards Compliance Appendix | The system SHALL auto-generate a Standards Compliance appendix listing all applicable architecture and technology standards from the standards catalog, with conformance status (compliant/non-compliant/waiver) per standard. | Functional | Governance Lead | Must | Compliance |
| REQ-DLV-333 | Cross-Reference Index Appendix | The system SHALL auto-generate a Cross-Reference Index appendix for large documents listing all internal cross-references (figure numbers, table numbers, section references) alphabetically by referenced element name. | Functional | Enterprise Architect | Should | Navigation |
| REQ-DLV-334 | Source Data Snapshot Appendix | The system SHALL optionally include a Source Data Snapshot appendix in generated deliverables showing the catalog query parameters, data snapshot timestamp, and record counts used at generation time, supporting reproducibility. | Data | Governance Lead | Could | Reproducibility |

---

### 8.5 Portfolio & Metrics Reports

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-335 | Architecture Portfolio Report | The system SHALL generate an Architecture Portfolio Report listing all active architecture projects with status, phase, deliverable completion %, open risks, and key milestones, sourced from V09 portfolio data. | Reporting | Portfolio Manager, CIO | Must | Portfolio visibility |
| REQ-DLV-336 | Capability Maturity Report | The system SHALL generate a Capability Maturity Report showing maturity heat map, domain averages, top 5 capability gaps, and improvement roadmap, populated from the capability catalog. | Reporting | Enterprise Architect | Must | Capability planning |
| REQ-DLV-337 | Technology Radar Report | The system SHALL generate a Technology Radar Report (similar to Thoughtworks radar) categorising technologies into: adopt/trial/assess/hold rings, with rationale text per technology entry. | Reporting | CTO, Technology Architect | Must | Technology governance |
| REQ-DLV-338 | Architecture Debt Report | The system SHALL generate an Architecture Debt Report cataloguing identified technical/architectural debts, their business impact, estimated remediation effort, and prioritised resolution recommendations. | Reporting | Enterprise Architect | Should | Debt management |
| REQ-DLV-339 | Compliance Dashboard Report | The system SHALL generate a Compliance Dashboard Report summarising: compliance checklist completion rate, open waivers, standards conformance by domain, and outstanding governance actions, as a one-page PDF and `.pptx` slide. | Reporting | Governance Lead | Must | Compliance |
| REQ-DLV-340 | KPI / Architecture Metrics Report | The system SHALL generate a KPI/Metrics Report for configurable architecture metrics (e.g., time-to-ADR, deliverable SLA adherence, review cycle time, portal engagement), populated from ESIP platform telemetry. | Reporting | Enterprise Architect, Administrator | Should | Performance management |

---

## 9. Quality, Compliance & Accessibility

### 9.1 Deliverable Completeness Checks

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-360 | Template Completeness Check | The system SHALL validate every generated deliverable against its template definition, flagging sections where required data was absent (resulting in a placeholder or empty section), and producing a completeness score (% of mandatory sections fully populated). | Functional | Enterprise Architect | Must | Quality |
| REQ-DLV-361 | Completeness Threshold Gate | The system SHALL allow administrators to configure a minimum completeness threshold (default 80%) below which a deliverable may not be marked as "Review Ready", surfacing a warning to the author listing unfilled mandatory sections. | Functional | Governance Lead | Must | Quality gate |
| REQ-DLV-362 | Missing Catalog Data Warning | The system SHALL identify and flag catalog data referenced by a deliverable template that is missing, empty, or outdated (last updated > configurable staleness threshold), prompting architects to complete or refresh data before generation. | Functional | Enterprise Architect | Must | Data quality |
| REQ-DLV-363 | Orphaned Section Detection | The system SHALL detect sections in generated deliverables that are not connected to any catalog data or merge field (i.e., purely manual-entry sections), flagging them for human authoring review. | Functional | Enterprise Architect | Should | Content coverage |
| REQ-DLV-364 | Diagram Placeholder Detection | The system SHALL detect diagram placeholders in generated deliverables that have not been fulfilled (no diagram exported from V03 for the referenced diagram ID) and list them in the post-generation quality summary. | Functional | Enterprise Architect | Must | Completeness |
| REQ-DLV-365 | Reference Integrity Check | The system SHALL validate all internal cross-references in generated documents to ensure every referenced figure number, table number, section number, and document link resolves to an actual element in the document, reporting broken references. | Functional | Enterprise Architect | Must | Document integrity |
| REQ-DLV-366 | Post-Generation Quality Summary | The system SHALL produce a post-generation quality summary for every generated deliverable, listing: completeness score, unfilled sections, orphaned sections, broken references, diagram placeholders, AI-drafted sections flagged for review, and accessibility violations. | Functional | Enterprise Architect | Must | Quality |

---

### 9.2 Template Compliance

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-367 | Framework Compliance Check | The system SHALL check generated TOGAF, DoDAF, NAF, and UAF deliverables against the mandatory content requirements defined in the respective framework standard (sourced from V01/V02), flagging missing mandatory elements and citing the framework reference. | Functional | Enterprise Architect | Must | Standards compliance |
| REQ-DLV-368 | Corporate Style Compliance | The system SHALL verify that generated `.docx`, `.pptx`, and PDF outputs conform to the active corporate branding theme (correct font families, correct heading color, correct logo placement, correct header/footer), flagging deviations as style warnings. | Functional | Governance Lead | Must | Branding compliance |
| REQ-DLV-369 | Section Order Validation | The system SHALL validate that the section order of the generated deliverable matches the template-defined required order, flagging any out-of-order sections resulting from conditional logic or manual override. | Functional | Enterprise Architect | Should | Template compliance |
| REQ-DLV-370 | Version and Metadata Completeness | The system SHALL verify that every generated deliverable has fully populated document metadata (title, version, date, author, reviewer, classification) and report any missing metadata fields as critical quality failures. | Functional | Governance Lead | Must | Document management |

---

### 9.3 Accessibility (WCAG / PDF-UA)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-371 | WCAG 2.2 AA Compliance — HTML | The system SHALL ensure all HTML portal and HTML deliverable output passes automated WCAG 2.2 Level AA accessibility checks (using an integrated axe-core or equivalent scanner), with a detailed accessibility report produced per page. | NFR | All | Must | Legal/regulatory |
| REQ-DLV-372 | PDF-UA Compliance | The system SHALL generate PDF deliverables compliant with PDF/UA-1 (ISO 14289-1), verified by an integrated PDF accessibility checker (PAC 3 criteria), ensuring tagged structure, alt text, reading order, and table headers. | NFR | All | Must | Legal/regulatory |
| REQ-DLV-373 | Alt Text for Diagrams | The system SHALL require and validate that every embedded diagram in every output format has a meaningful alt text description, sourced from the V03 diagram metadata `description` field, blocking generation if alt text is absent for a mandatory diagram. | NFR | All | Must | Accessibility |
| REQ-DLV-374 | Colour-Blind Safe Palette | The system SHALL provide a colour-blind safe palette option for all heat maps, charts, and status indicators in generated deliverables, using patterns or symbols in addition to colour to convey meaning. | NFR | All | Must | Accessibility |
| REQ-DLV-375 | Accessible Tables | The system SHALL ensure all tables in generated output have defined header rows (TH elements in HTML, proper header row styling in .docx, and tagged table headers in PDF), never using layout tables for data presentation. | NFR | All | Must | Accessibility |
| REQ-DLV-376 | Font Minimum Size | The system SHALL enforce a minimum body font size of 10pt in `.docx`/PDF output and 16px equivalent in HTML output across all active branding themes. | NFR | All | Must | Readability |
| REQ-DLV-377 | Keyboard Navigation — Portal | The system SHALL ensure the Architecture Portal is fully keyboard-navigable, with visible focus indicators on all interactive elements, logical tab order, and no keyboard traps. | NFR | All | Must | Accessibility |
| REQ-DLV-378 | Screen Reader Compatibility | The system SHALL test and verify HTML deliverable and portal output for compatibility with at minimum NVDA (Windows), VoiceOver (macOS/iOS), and TalkBack (Android) screen readers as part of the release acceptance criteria. | NFR | All | Must | Accessibility |

---

### 9.4 Redaction & Information Security

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-379 | Content Classification Tagging | The system SHALL allow architects to tag any catalog element, section, or deliverable with an information classification level (Unclassified, Internal, Confidential, Restricted, Secret) that governs visibility and redaction rules. | Security | Governance Lead | Must | Information security |
| REQ-DLV-380 | Auto-Redaction on Export | The system SHALL automatically redact tagged classified elements from generated deliverables and portal views when the requesting user's clearance level is below the element's classification, replacing content with a visible redaction marker. | Security | Governance Lead | Must | Information security |
| REQ-DLV-381 | Manual Redaction Tool | The system SHALL provide a manual redaction tool allowing governance users to select and redact specific text passages, table cells, or diagram regions in a generated PDF before distribution, producing a sanitised version distinct from the source document. | Security | Governance Lead | Must | Controlled release |
| REQ-DLV-382 | Redaction Audit Trail | The system SHALL maintain an immutable audit trail of all redaction actions (who redacted, what, from which document version, timestamp), accessible only to administrators and governance roles. | Security | Administrator | Must | Audit |
| REQ-DLV-383 | Sensitive Field Masking | The system SHALL support configurable masking of sensitive field values (e.g., internal IP addresses, credentials references, personnel IDs) in exported deliverables when publishing to the external portal, replacing values with `[MASKED]` tokens. | Security | Governance Lead | Must | Data protection |
| REQ-DLV-384 | DLP Integration | The system SHALL support integration with enterprise Data Loss Prevention (DLP) tools via webhook, invoking a DLP policy check on generated documents before download or distribution and blocking release if a policy violation is detected. | Security | Administrator | Should | Enterprise security |

---

### 9.5 Additional Quality Requirements

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-385 | Spell Check Integration | The system SHALL integrate a server-side spell check (supporting English, French, German, Spanish, and configurable additional languages) run at generation time, flagging suspected spelling errors in AI-drafted and merge-field content within the quality summary. | NFR | Enterprise Architect | Should | Content quality |
| REQ-DLV-386 | Readability Score | The system SHALL compute and display a Flesch-Kincaid readability score for narrative sections of generated deliverables, flagging sections with a score below the configured threshold (default Grade 12 / score 30) for simplification. | NFR | Enterprise Architect | Could | Communication quality |
| REQ-DLV-387 | Broken Hyperlink Check | The system SHALL validate all external URLs embedded in generated deliverables by making HEAD requests at generation time (with configurable timeout), flagging broken or unreachable links in the quality summary. | NFR | Enterprise Architect | Should | Link integrity |
| REQ-DLV-388 | File Size Warning | The system SHALL warn the architect if a generated document exceeds configurable file size thresholds (default: .docx > 20 MB, PDF > 50 MB, .pptx > 100 MB), recommending diagram resolution reduction or content segmentation. | NFR | Enterprise Architect | Should | Usability |
| REQ-DLV-389 | Automated Regression Test for Generation | The system SHALL include an automated regression test suite that generates a representative sample of deliverables (one per template) using a canonical test dataset after each platform release, verifying output format validity, section count, and file integrity, with failures blocking deployment. | NFR | Platform Engineer | Must | Release quality |
| REQ-DLV-390 | Generation Performance SLA | The system SHALL generate a single standard-length deliverable (≤50 pages, ≤10 embedded diagrams) within 30 seconds end-to-end, and a full TOGAF ADD (all four domain chapters) within 90 seconds, measured from generation request to file-ready. | NFR | All | Must | Performance |
| REQ-DLV-391 | Concurrent Generation | The system SHALL support at least 20 concurrent deliverable generation operations without degradation below the performance SLA defined in REQ-DLV-390. | NFR | All | Must | Scalability |
| REQ-DLV-392 | Generation Queue | The system SHALL implement a generation queue for bulk and scheduled operations, processing requests with configurable concurrency and providing queue position and estimated wait time to requesting users. | NFR | Platform Engineer | Must | Resource management |
| REQ-DLV-393 | Generation Retry on Transient Failure | The system SHALL automatically retry failed generation jobs up to 3 times on transient infrastructure failures (memory limit, timeout, transient network error), with each retry logged, before marking the job as failed and alerting the user. | NFR | Platform Engineer | Must | Reliability |
| REQ-DLV-394 | Long-Term Archival Format | The system SHALL support export of deliverables in archival-grade formats (PDF/A-3 for documents, ODF for editable, PNG for diagrams) for long-term archiving per records management policy. | Governance | Governance Lead | Must | Records management |
| REQ-DLV-395 | Checksum Verification | The system SHALL compute and store a SHA-256 checksum for every generated deliverable file, enabling integrity verification before download and detecting tampering in long-term storage. | Security | Governance Lead | Must | Integrity |
| REQ-DLV-396 | Virus Scan on Download | The system SHALL invoke a configurable antivirus/malware scan (ClamAV or enterprise AV integration via webhook) on generated document files before making them available for download, quarantining files that fail the scan. | Security | Administrator | Should | Security |
| REQ-DLV-397 | Document Expiry | The system SHALL support configurable document expiry periods per deliverable type (e.g., Architecture Vision expires 12 months from approval), surfacing expiry warnings to owners 30 and 7 days before expiry and flagging expired deliverables in the portal. | Governance | Governance Lead | Must | Document lifecycle |
| REQ-DLV-398 | Copyright & Licensing Notices | The system SHALL embed configurable copyright and licensing notices (configurable per template: org name, year, license type, confidentiality notice) in the footer of every generated document page and in document metadata. | Governance | Governance Lead | Must | Legal compliance |
| REQ-DLV-399 | GDPR / PII Compliance | The system SHALL detect and warn when personally identifiable information (PII) fields (names, email addresses, phone numbers derived from stakeholder catalog) are included in published portal content, prompting the architect to confirm consent or redact before publishing. | Governance | Governance Lead | Must | GDPR |

---

## 10. Integration & Platform Requirements

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-DLV-400 | V03 Diagram Engine Integration | The system SHALL consume diagrams from the V03 rendering engine via a defined internal API, triggering diagram export (PNG/SVG/interchange format) as part of the document generation pipeline without requiring manual diagram export by the architect. | Integration | Platform Engineer | Must | V03 boundary |
| REQ-DLV-401 | V06 AI Content Integration | The system SHALL invoke V06 AI content drafting services to populate narrative merge fields in templates (executive summary, option rationale, recommendation narrative) when the template marks a field as `source: ai_draft`. | Integration | Platform Engineer | Must | V06 boundary |
| REQ-DLV-402 | V07 Requirements Catalog Integration | The system SHALL read requirements, constraints, and assumptions from the V07 requirements catalog via defined API to populate RTM appendices, ARS documents, and gap analysis tables. | Integration | Platform Engineer | Must | V07 boundary |
| REQ-DLV-403 | V08 Knowledge Hub Integration | The system SHALL read architecture knowledge artifacts (patterns, reference architectures, lessons learned) from the V08 Knowledge Hub to populate relevant sections in ADD, HLD, and best-practice appendices. | Integration | Platform Engineer | Should | V08 boundary |
| REQ-DLV-404 | V09 Portfolio Data Integration | The system SHALL read portfolio metrics and project status data from V09 to populate Architecture Portfolio Reports and roadmap timeline slides. | Integration | Platform Engineer | Should | V09 boundary |
| REQ-DLV-405 | V11 Branding/Theme Integration | The system SHALL read corporate branding theme assets (logos, color tokens, font definitions, header/footer templates) from V11 via the platform API, applying them to all generated outputs without local copies. | Integration | Platform Engineer | Must | V11 boundary |
| REQ-DLV-406 | Confluence Export Integration | The system SHALL support one-click publishing of generated Markdown and HTML deliverables to Atlassian Confluence spaces via the Confluence REST API v2, preserving heading structure as Confluence page hierarchy. | Integration | Enterprise Architect | Must | Atlassian toolchain |
| REQ-DLV-407 | SharePoint Export Integration | The system SHALL support uploading generated `.docx`, PDF, and `.pptx` deliverables to Microsoft SharePoint document libraries via Microsoft Graph API, with configurable target library and folder path per deliverable type. | Integration | Enterprise Architect | Must | Microsoft 365 toolchain |
| REQ-DLV-408 | Git Repository Export | The system SHALL support committing generated Markdown deliverables and diagram exports directly to a configured Git repository (GitHub, GitLab, Azure DevOps) via their respective APIs, enabling docs-as-code workflows with automated PR creation. | Integration | Platform Engineer | Should | Docs-as-code |
| REQ-DLV-409 | ServiceNow CMDB Integration | The system SHALL support exporting application, component, and infrastructure catalog data from generated deliverables to ServiceNow CMDB via ServiceNow Table API, mapping ESIP catalog fields to CMDB CI attributes. | Integration | Enterprise Architect | Should | ITSM integration |
| REQ-DLV-410 | SMTP Email Service | The system SHALL use a configurable SMTP or email service API (SendGrid, AWS SES, Microsoft Exchange) for all delivery of scheduled reports and portal notifications, with support for HTML email templates and inline branding. | Integration | Platform Engineer | Must | Email delivery |
| REQ-DLV-411 | Cloud Storage Integration | The system SHALL support storing generated deliverables in configurable cloud object storage (AWS S3, Azure Blob Storage, Google Cloud Storage) in addition to local filesystem storage, with configurable bucket/container, path prefix, and encryption-at-rest settings. | Integration | Platform Engineer | Should | Cloud-native deployment |
| REQ-DLV-412 | Webhook Notifications | The system SHALL emit configurable webhook events on key deliverable lifecycle events (generated, approved, published, expired, quality-failed) to registered endpoints, enabling integration with enterprise workflow tools (Power Automate, Zapier, ITSM). | Integration | Platform Engineer | Should | Ecosystem integration |
| REQ-DLV-413 | Sparx Enterprise Architect Integration | The system SHALL support import from and export to Sparx Enterprise Architect (XMI, ArchiMate OEF) for bi-directional model exchange with organizations using Sparx EA as their primary modeling tool. | Integration | Enterprise Architect | Should | EA tool interop |
| REQ-DLV-414 | iServer / Orbus Integration | The system SHALL provide an iServer/Orbus Software REST API integration for importing repository data (capability catalog, application catalog, architecture elements) from iServer-based repositories. | Integration | Enterprise Architect | Could | EA tool interop |
| REQ-DLV-415 | LeanIX Integration | The system SHALL provide a LeanIX REST API integration for importing application landscape and capability data from LeanIX, synchronising with ESIP catalogs for use in generated deliverables. | Integration | Enterprise Architect | Could | EA tool interop |
| REQ-DLV-416 | Jira Integration | The system SHALL support linking work packages and architecture gaps to Jira issues via the Jira REST API, embedding live Jira issue status in roadmap and work package tables in generated deliverables. | Integration | Program Manager, Enterprise Architect | Should | Project toolchain |
| REQ-DLV-417 | Azure DevOps Boards Integration | The system SHALL support linking work packages to Azure DevOps work items via the Azure DevOps REST API, reflecting work item status in generated roadmap deliverables. | Integration | Program Manager | Should | Project toolchain |
| REQ-DLV-418 | Audit Log Export | The system SHALL support exporting the full deliverable generation and publication audit log to SIEM systems (Splunk, Microsoft Sentinel) via syslog or webhook, including all security-relevant events. | Security | Administrator | Should | Security monitoring |
| REQ-DLV-419 | API Key / OAuth Credential Management | The system SHALL provide a secure credential vault for storing API keys, OAuth tokens, and service account credentials used by all integrations, with encryption at rest (AES-256), role-based access to credential management, and configurable rotation reminders. | Security | Administrator | Must | Security |

---

## 11. Requirements Summary Table

| Area | ID Range | Count | Key Themes |
|---|---|---|---|
| Per-Framework Deliverable Templates | REQ-DLV-001 – REQ-DLV-095 | 95 | TOGAF (24), Zachman (6), DoDAF/MODAF/NAF/UAF (11), ArchiMate (7), SA (20), Per-Role (10), Bundles/Packs (17) |
| Document Generation Engines | REQ-DLV-100 – REQ-DLV-162 | 63 | .docx (14), .pptx (10), .xlsx (9), PDF (8), Markdown (6), HTML (6), Cross-engine (10) |
| Template Management | REQ-DLV-170 – REQ-DLV-209 | 40 | Library (6), Authoring (8), Versioning/Governance (7), Branding (5), Multi-language (5), Advanced (9) |
| Diagram Export | REQ-DLV-210 – REQ-DLV-240 | 31 | Raster (4), Vector (4), Interchange formats (10), Batch/advanced (8), Import (5) |
| Architecture Portal & Publishing | REQ-DLV-250 – REQ-DLV-289 | 40 | Infrastructure (11), Access control (6), Stakeholder views (7), Comments (6), Publishing workflow (10) |
| Presentation & Reporting | REQ-DLV-310 – REQ-DLV-340 | 31 | Executive decks (6), Scheduled reports (8), Bundles (5), Traceability appendices (6), Portfolio/metrics (6) |
| Quality, Compliance & Accessibility | REQ-DLV-360 – REQ-DLV-399 | 40 | Completeness (7), Template compliance (4), WCAG/PDF-UA (8), Redaction/security (6), Additional quality (15) |
| Integration & Platform | REQ-DLV-400 – REQ-DLV-419 | 20 | Suite integration (6), Toolchain (9), Security/ops (5) |
| **TOTAL** | | **360** | |

---

## Appendix A — Format Support Matrix

| Deliverable Type | .docx | .pptx | .xlsx | PDF | Markdown | HTML |
|---|---|---|---|---|---|---|
| TOGAF ADD | ✓ | ✓ | — | ✓ | ✓ | ✓ |
| Architecture Vision | ✓ | ✓ | — | ✓ | ✓ | ✓ |
| Architecture Contract | ✓ | — | — | ✓ | ✓ | ✓ |
| ARS / RTM | ✓ | — | ✓ | ✓ | ✓ | ✓ |
| Architecture Roadmap | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| SoAW | ✓ | — | — | ✓ | ✓ | — |
| Capability Assessment | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| DoDAF View Products | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| ArchiMate View Sets | ✓ | ✓ | — | ✓ | ✓ | ✓ |
| HLD / LLD | ✓ | — | — | ✓ | ✓ | ✓ |
| ADR / ADR Register | ✓ | — | — | ✓ | ✓ | ✓ |
| Options Paper | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| API Spec Document | ✓ | — | — | ✓ | ✓ | ✓ |
| Data Model Document | ✓ | — | ✓ | ✓ | ✓ | ✓ |
| Threat Model Report | ✓ | — | ✓ | ✓ | ✓ | ✓ |
| Cloud Landing Zone | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Executive Deck | — | ✓ | — | ✓ | — | — |
| Board Report | ✓ | ✓ | — | ✓ | — | — |
| Catalog Export | — | — | ✓ | ✓ | ✓ | ✓ |
| Architecture Status Report | ✓ | ✓ | — | ✓ | ✓ | ✓ |

---

## Appendix B — Interchange Format Support Matrix

| Diagram Type | PNG | SVG | PDF | .vsdx | .drawio | XMI | ArchiMate OEF | BPMN 2.0 | PlantUML | Mermaid | C4 DSL |
|---|---|---|---|---|---|---|---|---|---|---|---|
| ArchiMate | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | — | — | — | — |
| UML (Class/Seq/Comp) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | ✓ |
| BPMN | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | — | — |
| C4 | ✓ | ✓ | ✓ | — | ✓ | — | — | — | ✓ | ✓ | ✓ |
| Flow / ER / Generic | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | ✓ | ✓ | — |
| DoDAF / NAF Views | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — |

---

## Appendix C — Boundary Decisions & Assumptions

| Decision | Rationale |
|---|---|
| Diagram rendering logic is entirely in V03; V10 calls V03 API for export only | Clean separation of concerns; V03 owns the canvas and rendering pipeline |
| AI narrative generation is entirely in V06; V10 calls V06 API for merge field population | V06 owns model selection, prompt engineering, and content drafting; V10 owns document composition |
| Framework deliverable *definitions* (what constitutes a TOGAF ADD) live in V01/V02; V10 uses them as specification inputs | Single source of truth for framework knowledge |
| Corporate branding assets (logos, color tokens, font files) are stored and governed in V11; V10 reads them at generation time | Prevents duplicate branding configuration |
| Requirements catalog and RTM data are owned by V07; V10 queries V07 for RTM appendix population | Single source of truth for requirements |
| Knowledge Hub patterns/reference architectures are owned by V08; V10 reads them as advisory content | V08 owns curated knowledge; V10 embeds it in output documents |
| Portfolio metrics sourced from V09; V10 reads and formats | V09 owns project/portfolio data model |
| SheetJS, pptxgenjs, and jsPDF are retained and extended (not replaced) | Preserves existing investment and compatibility |
| Template storage is local to V10 (not in V08 Knowledge Hub) | Templates are generation-engine constructs, not architecture knowledge artifacts |
| Portal static site generation is included in V10 scope | Publishing is an output/distribution concern owned by the deliverable factory |
| Portal authentication references enterprise IdP; V10 implements the integration but does not own identity governance | Identity governance is a platform concern |

---

*End of Volume 10 — Deliverables Generation, Export & Publishing*
