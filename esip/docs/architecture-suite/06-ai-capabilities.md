# Volume 06 — AI Engine Capabilities
## ESIP Architecture Automation Suite

| Attribute | Value |
|---|---|
| Volume | 06 |
| Prefix | AI |
| Status | Baseline Draft |
| Date | 2026-06-23 |
| Scope | AI reasoning, generation, agentic workflows, RAG, guardrails, prompt management, provider management, cost governance, and traceability across all ESIP personas and frameworks |
| Out of Scope | Ingestion file-parsing mechanics (V07), diagram rendering engine (V03), code-scan mechanics (V09), document rendering (V10), platform secrets/proxy infrastructure (V11) |

> **Build phase:** The AI engine core (extraction, AI diagram generation, copilots, RAG) is **Phase 1 foundational** per `BUILD-PLAN-MVP-AND-PHASING.md`; the **fine-tuning / feedback-learning loop (§14) and model-improvement pipelines are post-MVP**. Per the master-index ruling (§6 ruling #1), `Must` means required in the product, not required in the MVP — priority is decoupled from build sequence.

---

## Personas Reference

| Code | Persona |
|---|---|
| EA | Enterprise Architect |
| SA | Solution Architect |
| AA | Application Architect |
| IA | Integration/API Architect |
| DA | Data Architect |
| CA | Cloud/Infrastructure Architect |
| SecA | Security Architect |
| MA | Microservices Architect |
| CRM | CRM/Platform Architect |
| ALL | All personas |

---

## Table of Contents

1. [AI Understanding & Document Comprehension](#1-ai-understanding--document-comprehension)
2. [Requirement Intelligence](#2-requirement-intelligence)
3. [AI Diagram Generation](#3-ai-diagram-generation)
4. [AI Architecture Generation per Framework](#4-ai-architecture-generation-per-framework)
5. [AI Code Analysis](#5-ai-code-analysis)
6. [AI Deliverable Drafting](#6-ai-deliverable-drafting)
7. [Per-Persona Copilots](#7-per-persona-copilots)
8. [Agentic Workflows](#8-agentic-workflows)
9. [RAG over the Knowledge Hub](#9-rag-over-the-knowledge-hub)
10. [Provider & Model Management](#10-provider--model-management)
11. [Guardrails & Quality](#11-guardrails--quality)
12. [Prompt Library & Versioning](#12-prompt-library--versioning)
13. [Cost, Usage Governance & Metering](#13-cost-usage-governance--metering)
14. [Feedback & Learning Loops](#14-feedback--learning-loops)
15. [Traceability AI](#15-traceability-ai)

---

## 1. AI Understanding & Document Comprehension

### 1.1 Multi-Format Document Ingestion Understanding

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-001 | Structured Document Comprehension | The system SHALL parse and semantically understand Word (.docx), PDF, Excel (.xlsx/.csv), and PowerPoint (.pptx) artifacts, extracting headings, tables, embedded lists, footnotes, and metadata to build an internal semantic representation | Functional | ALL | Must |
| REQ-AI-002 | Spreadsheet Semantic Extraction | The system SHALL interpret Excel/CSV workbooks including multi-sheet models, named ranges, formulas as business rules, pivot tables, and data relationships, converting them to structured semantic objects usable by downstream AI tasks | Functional | DA, SA | Must |
| REQ-AI-003 | Presentation Intent Extraction | The system SHALL extract slide intent from PowerPoint decks: title/body hierarchy, speaker notes as elaboration, embedded diagrams as context signals, and narrative flow across slide sequence | Functional | EA, SA | Must |
| REQ-AI-004 | OCR and Image Comprehension | The system SHALL apply OCR to scanned PDFs, images (PNG, JPEG, TIFF), and embedded raster graphics to extract textual content with ≥95% character-level accuracy on standard business documents at 300 DPI or above | AI/ML | ALL | Must |
| REQ-AI-005 | Diagram / Visio Comprehension | The system SHALL parse Visio (.vsdx), embedded draw.io XML, and image-format diagrams to extract node labels, relationship labels, groupings, and topology, producing a graph-structured semantic representation | Functional | EA, SA, IA | Must |
| REQ-AI-006 | Email and Teams Chat Comprehension | The system SHALL process email threads (.eml, .msg) and exported Teams/Slack transcripts to extract decisions, action items, requirements, and stakeholder positions, attributing each fragment to the originating speaker/timestamp | Functional | ALL | Should |
| REQ-AI-007 | Jira/Confluence Artifact Comprehension | The system SHALL ingest Jira issue exports (JSON/XML) and Confluence page exports (HTML/PDF) and extract feature requests, acceptance criteria, constraints, and cross-link relationships | Integration | ALL | Should |
| REQ-AI-008 | Source Code Comprehension | The system SHALL comprehend source code artifacts at module/class/function granularity: language detection, public API surface extraction, dependency graph construction, business logic narration, and architectural layer classification | Functional | AA, MA, SA | Must |
| REQ-AI-009 | Cross-Artifact Entity Resolution | The system SHALL resolve entity mentions across multiple ingested artifacts (e.g., "Customer" in a Word doc = "customer_id" in an ERD = `CustomerService` class in code) using embedding similarity and heuristic name normalization | AI/ML | ALL | Must |
| REQ-AI-010 | Ingestion Confidence Annotation | The system SHALL annotate every extracted semantic fragment with a confidence score (0.0–1.0) and provenance reference (source document, page/section, character offset) to support downstream citation and review | Data | ALL | Must |

**Acceptance Criteria (REQ-AI-001–010):**
- AC1: Given a 200-page Word RFP, the system produces a hierarchical semantic tree in ≤60 seconds
- AC2: Entity resolution achieves ≥80% precision on a held-out cross-document test set
- AC3: OCR accuracy verified by diff against known ground-truth documents quarterly

---

### 1.2 Semantic Understanding Depth

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-011 | Domain Concept Recognition | The system SHALL recognise domain-specific terminology across IT architecture, finance, healthcare, logistics, and retail verticals using pre-trained domain vocabularies and user-supplied glossaries | AI/ML | ALL | Must |
| REQ-AI-012 | Intent Classification | The system SHALL classify each document section by intent: requirement, constraint, assumption, decision, risk, issue, question, or background, assigning labels with justification | AI/ML | ALL | Must |
| REQ-AI-013 | Implicit Requirement Surfacing | The system SHALL infer unstated but implied requirements from artifact context (e.g., an authenticated API endpoint implies an authentication mechanism requirement) and surface them as AI-generated candidate requirements flagged for human review | AI/ML | ALL | Should |
| REQ-AI-014 | Temporal and Version Awareness | The system SHALL detect version markers, revision history, and "as-is" vs "to-be" signals within documents, tagging extracted content accordingly to prevent mixing current-state and future-state semantics | Functional | EA, SA | Must |

---

## 2. Requirement Intelligence

### 2.1 Requirement Extraction & Classification

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-020 | Automated Requirement Extraction | The system SHALL extract discrete requirements from any ingested artifact, assigning each a unique ID, title, description, source reference, and initial classification | Functional | ALL | Must |
| REQ-AI-021 | Requirement Type Classification | The system SHALL classify every extracted requirement into: Functional, Non-Functional (NFR), Business Rule, Constraint, Assumption, or Dependency, with confidence score ≥0.75 threshold for auto-acceptance | AI/ML | ALL | Must |
| REQ-AI-022 | NFR Sub-Category Classification | The system SHALL further classify NFRs into ISO 25010 sub-categories: Performance, Scalability, Availability, Security, Maintainability, Portability, Usability, Interoperability, Compliance — with rationale | AI/ML | ALL | Must |
| REQ-AI-023 | Business vs. Technical Requirement Separation | The system SHALL distinguish business-level requirements (what the business needs) from technical requirements (how a system shall behave), enabling separate traceability paths | AI/ML | EA, SA | Must |
| REQ-AI-024 | MoSCoW Priority Suggestion | The system SHALL propose a MoSCoW priority (Must/Should/Could/Won't) for each requirement based on: business-value signals in source text, stakeholder emphasis markers, dependency analysis, and regulatory keyword detection | AI/ML | ALL | Must |
| REQ-AI-025 | SMART Quality Scoring | The system SHALL score each requirement against SMART criteria (Specific, Measurable, Achievable, Relevant, Time-bound) on a 0–5 scale per dimension, with natural-language improvement suggestions | AI/ML | ALL | Must |
| REQ-AI-026 | INVEST Quality Scoring | The system SHALL score user-story-format requirements against INVEST criteria (Independent, Negotiable, Valuable, Estimable, Small, Testable) producing a composite score and per-criterion feedback | AI/ML | SA, AA | Should |
| REQ-AI-027 | IEEE-830 Compliance Checking | The system SHALL evaluate requirements against IEEE-830 attributes: correct, unambiguous, complete, consistent, ranked, verifiable, modifiable, traceable — flagging deficiencies with specific remediation guidance | AI/ML | ALL | Should |
| REQ-AI-028 | Acceptance Criteria Generation | The system SHALL auto-generate 3–5 Gherkin-formatted (Given/When/Then) acceptance criteria per functional requirement, using context from the source artifact and project glossary | Functional | AA, MA | Must |

**Rationale:** ESIP already generates NFRs and MoSCoW suggestions; these requirements formalise and extend that capability with quality frameworks.

---

### 2.2 Gap, Ambiguity & Conflict Detection

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-030 | Gap Detection | The system SHALL compare the extracted requirement set against a reference requirement taxonomy (configurable per framework: TOGAF, ISO 25010, GDPR checklist, etc.) and flag requirement areas with zero coverage as gaps | AI/ML | EA, SA | Must |
| REQ-AI-031 | Ambiguity Detection | The system SHALL detect linguistically ambiguous requirement text — undefined pronouns, vague quantifiers ("fast", "many", "appropriate"), passive voice without agent, and compound requirements — flagging each with a plain-English explanation | AI/ML | ALL | Must |
| REQ-AI-032 | Confusion / Inconsistency Detection | The system SHALL detect internally inconsistent statement pairs within a requirement (e.g., "SHALL support 10,000 concurrent users" in one place and "designed for 500 users" in another) using semantic entailment analysis | AI/ML | ALL | Must |
| REQ-AI-033 | Duplicate Detection | The system SHALL identify near-duplicate and semantically equivalent requirements across the requirement set using cosine similarity on embeddings (configurable threshold, default 0.90), presenting a merge-or-keep decision to the user | AI/ML | ALL | Must |
| REQ-AI-034 | Requirement Conflict Detection | The system SHALL identify logically conflicting requirements (e.g., "data SHALL never leave EU" conflicts with "system SHALL replicate globally"), classify conflict type (logical, regulatory, architectural), and recommend resolution options | AI/ML | EA, SecA | Must |
| REQ-AI-035 | Completeness Gap Recommendations | The system SHALL generate recommended new requirements to fill detected gaps, presenting them as AI drafts pending human approval, traceable to the gap finding | AI/ML | EA, SA | Should |
| REQ-AI-036 | Dependency Graph Generation | The system SHALL infer requirement-level dependencies (REQ-A depends on REQ-B, REQ-A contradicts REQ-B) and render a dependency graph navigable within the Requirements Pillar UI | Functional | SA | Should |

---

### 2.3 Threat & Security Requirement Generation

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-040 | STRIDE Threat Generation | The system SHALL, given an architecture model or set of functional requirements, automatically generate STRIDE threat scenarios (Spoofing, Tampering, Repudiation, Information Disclosure, DoS, Elevation of Privilege) with per-threat likelihood/impact ratings | AI/ML | SecA, SA | Must |
| REQ-AI-041 | STRIDE-to-Security-Requirement Mapping | The system SHALL convert each generated STRIDE threat into one or more actionable security requirements (SHALL statements), tagged as NFR/Security, with OWASP or NIST SP 800-53 control mappings where applicable | AI/ML | SecA | Must |
| REQ-AI-042 | Data Classification Inference | The system SHALL infer data classification levels (Public, Internal, Confidential, Restricted/PII, Regulated) for data entities identified in requirements and models, triggering appropriate NFR generation (encryption, masking, retention) | AI/ML | SecA, DA | Must |
| REQ-AI-043 | Regulatory NFR Generation | The system SHALL, when a regulatory context is detected (GDPR, HIPAA, PCI-DSS, SOX, FedRAMP), auto-generate a set of mandatory compliance NFRs referencing the relevant regulation clauses | AI/ML | SecA, EA | Must |

---

### 2.4 NFR Auto-Generation (Formalisation of Existing ESIP Capability)

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-045 | NFR Generation from Functional Requirements | The system SHALL, for every accepted functional requirement set, automatically generate a baseline set of NFRs covering: performance SLAs, availability targets, scalability thresholds, data retention, auditability, and accessibility, using configurable baseline templates | AI/ML | ALL | Must |
| REQ-AI-046 | NFR Quantification Assistance | The system SHALL propose specific measurable thresholds for vaguely-stated NFRs (e.g., converting "system should be fast" to "95th-percentile response time ≤ 500ms under 1,000 concurrent users") using industry benchmarks and project context | AI/ML | SA, CA | Must |
| REQ-AI-047 | NFR Consistency Enforcement | The system SHALL enforce consistency across related NFRs (e.g., if availability = 99.9%, disaster recovery RTO must be ≤ 8.76 hours), flagging violations and suggesting corrections | AI/ML | SA | Should |

---

## 3. AI Diagram Generation

### 3.1 Text/Requirements → Diagram Generation

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-050 | Natural Language to Diagram | The system SHALL accept a natural language prompt or a set of requirements and generate a structured diagram specification (node list, relationship list, diagram type, layout hints) that the V03 rendering engine converts to a rendered diagram | Functional | ALL | Must |
| REQ-AI-051 | ArchiMate 3.2 Diagram Generation | The system SHALL generate valid ArchiMate 3.2 models (Motivation, Strategy, Business, Application, Technology, Physical, Migration layers) from requirements, architecture narratives, or Knowledge Hub content, producing ArchiMate exchange format (XML) or equivalent | AI/ML | EA | Must |
| REQ-AI-052 | UML Diagram Generation | The system SHALL generate UML 2.5 diagrams — Use Case, Class, Sequence, Activity, Component, Deployment, State Machine — from requirements and code analysis outputs, producing PlantUML or Mermaid source | AI/ML | AA, SA | Must |
| REQ-AI-053 | BPMN 2.0 Process Diagram Generation | The system SHALL generate BPMN 2.0 process diagrams from business process descriptions in requirements or narrative documents, including pools, lanes, gateways, events, and sub-processes | AI/ML | EA, SA | Must |
| REQ-AI-054 | C4 Model Diagram Generation | The system SHALL generate C4 model diagrams at all four levels (Context, Container, Component, Code) from architecture descriptions and code analysis, producing Structurizr DSL or C4-PlantUML output | AI/ML | SA, AA | Must |
| REQ-AI-055 | ERD Generation | The system SHALL generate Entity-Relationship Diagrams from data requirements, database schemas, ORM models, or descriptive text, with Crow's Foot or Chen notation, producing Mermaid ERD or DBML output | AI/ML | DA | Must |
| REQ-AI-056 | Data Flow Diagram (DFD) Generation | The system SHALL generate Level-0 and Level-1 DFDs (Yourdon–Coad notation) from data requirements, process descriptions, and code analysis, identifying external entities, processes, data stores, and data flows | AI/ML | DA, SecA | Must |
| REQ-AI-057 | Network / Infrastructure Topology Diagram | The system SHALL generate network topology and cloud infrastructure diagrams (using provider icon vocabularies: AWS, Azure, GCP) from infrastructure requirements and IaC (Terraform/CloudFormation) analysis | AI/ML | CA | Should |
| REQ-AI-058 | API Interaction Diagram | The system SHALL generate API interaction sequence diagrams and API mesh/dependency diagrams from OpenAPI/AsyncAPI specifications, code analysis, or narrative API requirements | AI/ML | IA | Must |

---

### 3.2 Diagram Refinement & Iteration

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-060 | Conversational Diagram Refinement | The system SHALL support multi-turn chat-based refinement of a generated diagram: the user may issue natural language instructions ("add a CDN layer between the user and the load balancer") and the system SHALL update the diagram specification and re-render without losing prior content | Functional | ALL | Must |
| REQ-AI-061 | Diagram Validation Against Standards | The system SHALL validate a generated diagram against the target notation's formal rules (ArchiMate relationship validity, UML type constraints, BPMN flow completeness) and report violations with suggested corrections before rendering | AI/ML | EA, SA | Must |
| REQ-AI-062 | Diagram Merge from Multiple Sources | The system SHALL merge diagram specifications generated from multiple artifact ingestions (e.g., combine a component diagram from code analysis with a deployment diagram from infrastructure requirements) resolving conflicts and duplicate elements | AI/ML | SA | Should |
| REQ-AI-063 | Diagram Versioning Awareness | The system SHALL track diagram revisions, allowing the AI to explain what changed between versions and why, supporting architectural change documentation | Functional | EA, SA | Should |

---

### 3.3 Diagram → Narrative Explanation

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-065 | Diagram-to-Narrative Explanation | The system SHALL, given any generated or imported diagram, produce a structured natural-language explanation at configurable detail levels: Executive Summary (1 paragraph), Architect Brief (1 page), Full Technical Narrative (unlimited) | Functional | ALL | Must |
| REQ-AI-066 | Diagram Pattern Recognition | The system SHALL identify well-known architectural patterns present in a diagram (e.g., CQRS, Saga, Sidecar, Event Sourcing, Strangler Fig) and annotate the explanation with pattern names, benefits, and trade-offs | AI/ML | SA, MA | Should |
| REQ-AI-067 | Cross-Diagram Narrative Coherence | The system SHALL, when multiple diagrams exist in a project, generate a coherent cross-diagram narrative that explains how the views relate, referencing ArchiMate correspondence rules where applicable | AI/ML | EA | Should |

---

## 4. AI Architecture Generation per Framework

### 4.1 TOGAF Phase Artifact Generation

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-070 | TOGAF ADM Phase A — Architecture Vision | The system SHALL generate a draft Architecture Vision document including: problem statement, stakeholder map, objectives, scope, principles, and high-level solution concept, derived from project requirements and business context | Functional | EA | Must |
| REQ-AI-071 | TOGAF ADM Phase B — Business Architecture | The system SHALL generate Business Architecture artifacts: business capability model, value stream map, organisation map, business interaction matrix, and gap analysis table, from ingested business requirements | Functional | EA | Must |
| REQ-AI-072 | TOGAF ADM Phase C — IS Architecture (Application) | The system SHALL generate Application Architecture artifacts: application portfolio, application communication diagram, application interaction matrix, and software distribution diagram, from functional requirements and code analysis | Functional | EA, AA | Must |
| REQ-AI-073 | TOGAF ADM Phase C — IS Architecture (Data) | The system SHALL generate Data Architecture artifacts: conceptual data model, data dissemination diagram, data security diagram, data migration diagram, and data lifecycle diagram, from data requirements and schema analysis | Functional | EA, DA | Must |
| REQ-AI-074 | TOGAF ADM Phase D — Technology Architecture | The system SHALL generate Technology Architecture artifacts: technology standards catalog, technology portfolio, environments and locations diagram, platform decomposition diagram, and networks/communications diagram, from infrastructure requirements and IaC | Functional | EA, CA | Must |
| REQ-AI-075 | TOGAF ADM Phase E — Gap Analysis | The system SHALL generate Opportunities & Solutions phase content: consolidated gap analysis across all architecture domains, solution building blocks, implementation options assessment, and work package candidates | Functional | EA | Must |
| REQ-AI-076 | TOGAF Architecture Principles Generation | The system SHALL generate architecture principle statements (Name, Statement, Rationale, Implications) appropriate to the project context, covering business, data, application, and technology domains | AI/ML | EA | Must |
| REQ-AI-077 | TOGAF Architecture Requirements Specification | The system SHALL generate the Architecture Requirements Specification document mapping architecture requirements to ADM phases, with traceability to source business requirements | Functional | EA | Should |

---

### 4.2 Zachman Framework Cell Generation

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-080 | Zachman Cell Auto-Population | The system SHALL automatically populate Zachman Framework cells (6 rows × 6 columns) from ingested artifacts and generated models, flagging cells with insufficient source data | Functional | EA | Must |
| REQ-AI-081 | Zachman Cross-Cell Consistency | The system SHALL validate cross-cell consistency (e.g., the "What" at Planner row must align with "What" at Owner row) and flag discrepancies with explanations | AI/ML | EA | Should |

---

### 4.3 DoDAF / MODAF / UAF View Generation

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-085 | DoDAF View Generation | The system SHALL generate selected DoDAF 2.02 views (OV-1, OV-2, OV-5a/5b, SV-1, SV-4, SV-5, DIV-1/2/3) from project artifacts, with placeholders for data not inferable from source | Functional | EA | Should |
| REQ-AI-086 | UAF Domain Population | The system SHALL populate UAF domains (Operational, Service, Resource, Personnel, Security, Standard, Programme) from ingested requirements and architecture models | Functional | EA | Could |

---

### 4.4 Solution Architecture (HLD/LLD) Generation

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-090 | High-Level Design (HLD) Generation | The system SHALL generate an HLD document including: solution overview, architecture drivers, component breakdown, integration touchpoints, key technology choices with rationale, NFR approach, and risks — from requirements and architecture vision | Functional | SA | Must |
| REQ-AI-091 | Low-Level Design (LLD) Generation | The system SHALL generate an LLD document including: detailed component specifications, API contracts (OpenAPI stub), database schema outline, deployment topology, configuration parameters, and error handling approach | Functional | SA, AA | Must |
| REQ-AI-092 | Reference Architecture Instantiation | The system SHALL, given a reference architecture pattern (e.g., AWS Well-Architected, Azure Landing Zone, TOGAF TRM segment), instantiate it for the project context: substituting concrete technology choices, adapting topology, and generating project-specific constraints | AI/ML | SA, CA | Must |
| REQ-AI-093 | Architecture Decision Record (ADR) Generation | The system SHALL generate ADR drafts for each significant architectural decision identified during generation, including context, decision, consequences, and alternatives considered | Functional | SA, AA | Must |
| REQ-AI-094 | Integration Architecture Pattern Selection | The system SHALL recommend integration patterns (Event-Driven, Request-Reply, Pub/Sub, Saga, API Gateway, ESB, Sidecar, Service Mesh) for identified integration requirements, with trade-off analysis per recommendation | AI/ML | IA | Must |
| REQ-AI-095 | Microservices Decomposition | The system SHALL propose microservices boundaries using Domain-Driven Design heuristics (bounded contexts, aggregate roots, anti-corruption layers) applied to functional requirements and domain models | AI/ML | MA, SA | Must |
| REQ-AI-096 | Data Architecture Pattern Recommendation | The system SHALL recommend data architecture patterns (Data Mesh, Data Lakehouse, Lambda, Kappa, CQRS, Event Sourcing) based on data volume, velocity, and access requirements, with pro/con matrix | AI/ML | DA | Must |

---

## 5. AI Code Analysis

### 5.1 Repository Summarisation

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-100 | Repository Overview Narration | The system SHALL produce a structured natural-language summary of a repository: purpose, primary language(s), framework stack, entry points, module/layer structure, and key design patterns detected | Functional | AA, MA, SA | Must |
| REQ-AI-101 | Technology Stack Detection | The system SHALL detect the technology stack (languages, frameworks, ORMs, test frameworks, build tools, CI/CD config) from repository content with version identification where possible | Functional | SA, CA | Must |
| REQ-AI-102 | Business Domain Classification | The system SHALL classify code modules/packages into business domain concepts (e.g., "OrderManagement", "PaymentProcessing") using a combination of naming analysis, dependency patterns, and LLM inference | AI/ML | AA, DA | Should |

---

### 5.2 Flow & Dependency Derivation

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-105 | Call Graph Narration | The system SHALL, given a call graph produced by code-scan (V09), generate a human-readable narration of key execution flows, identifying entry points, critical paths, and dead code | AI/ML | AA, MA | Must |
| REQ-AI-106 | Dependency Risk Assessment | The system SHALL analyse third-party dependency graphs to identify: outdated packages, known CVE exposure (via OSINT), licence conflicts, and circular dependencies, producing a prioritised risk list | AI/ML | SecA, AA | Must |
| REQ-AI-107 | API Surface Extraction Narration | The system SHALL narrate the public API surface of a codebase: endpoints (REST/gRPC/GraphQL), message contracts (Kafka/AMQP), and data models — suitable for integration architecture documentation | Functional | IA, AA | Must |
| REQ-AI-108 | Architecture Conformance Analysis | The system SHALL compare the as-coded architecture (derived from code scan) against the target architecture (from ESIP models) and generate a gap/conformance report with violation descriptions | AI/ML | SA, EA | Should |
| REQ-AI-109 | Technical Debt Narrative | The system SHALL produce a structured technical debt narrative from code scan signals (cyclomatic complexity, duplication, test coverage gaps, deprecated API usage), categorised by debt type and estimated remediation effort tier | AI/ML | SA, AA | Should |

---

## 6. AI Deliverable Drafting

### 6.1 Vision & Strategy Documents

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-115 | Architecture Vision Document Drafting | The system SHALL draft a complete Architecture Vision document in TOGAF-compliant structure from project requirements, stakeholder inputs, and generated architecture models | Functional | EA | Must |
| REQ-AI-116 | Executive Summary Generation | The system SHALL generate an executive summary (1–2 pages) from any architecture deliverable, calibrated for non-technical stakeholder comprehension, highlighting business value, risks, and key decisions | Functional | EA, SA | Must |
| REQ-AI-117 | Strategy Roadmap Narrative | The system SHALL generate a narrative strategy roadmap document from a Roadmap model, explaining phases, milestones, dependencies, and expected business outcomes in plain English | Functional | EA | Must |

---

### 6.2 Architecture Decision Records

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-120 | ADR Batch Generation | The system SHALL generate a batch of ADRs from an architecture session, grouping related micro-decisions into coherent ADR units, assigning MADR or Michael Nygard template format | Functional | SA, AA | Must |
| REQ-AI-121 | ADR Supersession Tracking | The system SHALL detect when a new ADR logically supersedes an existing ADR and propose the supersession link with an AI-drafted change explanation | AI/ML | SA | Should |

---

### 6.3 Design Documents & Specs

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-125 | API Design Document Generation | The system SHALL draft API design documents including: API purpose, resource model, endpoint catalogue (with request/response examples), authentication scheme, rate limiting, versioning strategy, and error taxonomy | Functional | IA | Must |
| REQ-AI-126 | Data Dictionary Generation | The system SHALL generate a data dictionary from data models and schema analysis: entity/attribute descriptions, data types, constraints, sensitivity classification, lineage notes | Functional | DA | Must |
| REQ-AI-127 | Security Architecture Document | The system SHALL draft a security architecture document covering: threat model summary, identity & access model, network security zones, encryption strategy, audit & logging requirements, and compliance mapping | Functional | SecA | Must |
| REQ-AI-128 | Integration Design Document | The system SHALL draft an integration design document covering: integration landscape diagram, system of record designation, data flow descriptions, transformation rules, error handling, and SLA contracts per interface | Functional | IA | Must |
| REQ-AI-129 | Deployment & Operations Guide Stub | The system SHALL generate a deployment architecture and operations guide stub: deployment topology diagram narrative, environment list, runbook stubs for key operational scenarios, and monitoring/alerting requirements | Functional | CA | Should |

---

### 6.4 Deliverable Quality & Formatting

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-132 | Deliverable Completeness Scoring | The system SHALL score each drafted deliverable against a completeness checklist (framework-specific) and report missing sections with prioritised AI-generated drafts to fill gaps | AI/ML | ALL | Must |
| REQ-AI-133 | Audience Tone Calibration | The system SHALL accept a target-audience parameter (Executive, Technical Lead, Developer, Regulator) and adjust generated text tone, vocabulary, and detail level accordingly | AI/ML | ALL | Must |
| REQ-AI-134 | Template Conformance Checking | The system SHALL verify that generated deliverables conform to the project's selected output template (V10) and flag structural deviations before export | Functional | ALL | Must |

---

## 7. Per-Persona Copilots

### 7.1 Core Copilot Capabilities

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-140 | Persona-Grounded Chat Assistant | The system SHALL provide each persona with a chat assistant that is grounded in: (a) the project's Knowledge Hub, (b) the current requirement set, (c) generated architecture models, and (d) the persona's task context — refusing to speculate beyond its grounding without explicit "explore hypothetically" mode | Functional | ALL | Must |
| REQ-AI-141 | Knowledge Hub Citation in Responses | The system SHALL cite specific Knowledge Hub documents, sections, and page numbers in every substantive response, with clickable deep links to the source within the ESIP Knowledge Hub UI | Functional | ALL | Must |
| REQ-AI-142 | Contextual Question Answering | The system SHALL answer natural language questions about the project: "What is the current data retention policy?", "Which requirements are not yet covered by any architecture element?" — using RAG over the project's Knowledge Hub and models | AI/ML | ALL | Must |
| REQ-AI-143 | Proactive Insight Surfacing | The system SHALL proactively surface relevant insights during active work sessions: conflicting requirements, uncovered gaps, recent Knowledge Hub additions relevant to current task — configurable via persona preference | AI/ML | ALL | Should |

---

### 7.2 Slash Commands & Actions

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-145 | Slash Command Framework | The system SHALL support a slash-command interface within the copilot chat allowing personas to trigger specific AI actions without navigating menus: e.g., `/generate-adr`, `/extract-requirements`, `/find-gaps`, `/explain-diagram`, `/suggest-nfrs` | Functional | ALL | Must |
| REQ-AI-146 | Persona-Specific Command Sets | The system SHALL provide persona-specific default command sets (EA commands, SA commands, DA commands, SecA commands) with the ability to register custom commands per project or organisation | Functional | ALL | Should |
| REQ-AI-147 | Action Chaining | The system SHALL support command chaining: the output of one slash command automatically feeds as input to a subsequent command within the same chat turn (e.g., `/extract-requirements | /score-quality | /suggest-moscow`) | Functional | SA | Should |
| REQ-AI-148 | Command History & Replay | The system SHALL maintain a per-session command history and allow replay of any prior command with modified parameters, supporting iterative refinement workflows | Functional | ALL | Should |

---

### 7.3 Persona-Specific Copilot Specialisations

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-150 | EA Copilot — Framework Guidance | The EA Copilot SHALL provide guidance grounded in TOGAF ADM phase, Zachman cell, and ArchiMate 3.2 element selection, answering "which element should I use for X?" questions with standard-referenced rationale | AI/ML | EA | Must |
| REQ-AI-151 | SA Copilot — Pattern Advisor | The SA Copilot SHALL recommend architectural patterns, reference architectures, and technology trade-offs contextualised to project constraints, citing industry precedents from the Knowledge Hub | AI/ML | SA | Must |
| REQ-AI-152 | DA Copilot — Data Lineage Queries | The DA Copilot SHALL answer questions about data lineage, entity ownership, and data flow paths by reasoning over data models and integration diagrams in the Knowledge Hub | AI/ML | DA | Must |
| REQ-AI-153 | SecA Copilot — Threat Advisor | The SecA Copilot SHALL answer threat-model questions, suggest controls for specific threat scenarios, and verify that generated security requirements align with NIST/ISO 27001/OWASP frameworks | AI/ML | SecA | Must |
| REQ-AI-154 | IA Copilot — Integration Pattern Advisor | The IA Copilot SHALL advise on integration pattern selection, API design, event schema design, and idempotency strategies grounded in the project's integration models | AI/ML | IA | Must |
| REQ-AI-155 | CA Copilot — Cloud Design Advisor | The CA Copilot SHALL advise on cloud provider service selection, Well-Architected pillar alignment, IaC module recommendations, and cost optimisation trade-offs | AI/ML | CA | Must |
| REQ-AI-156 | MA Copilot — DDD Advisor | The MA Copilot SHALL advise on domain model design, bounded context definition, service decomposition, and event contract design grounded in DDD tactical/strategic patterns | AI/ML | MA | Must |

---

## 8. Agentic Workflows

### 8.1 Agent Architecture

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-160 | Multi-Step Autonomous Agent Execution | The system SHALL support multi-step agentic task execution: an agent receives a high-level goal, decomposes it into a plan, executes steps sequentially (with tool calls), and produces a structured output — all within a single user-initiated workflow | Functional | ALL | Must |
| REQ-AI-161 | Agent Planning with Plan Preview | The system SHALL, before executing a multi-step agent workflow, generate and display an execution plan (ordered task list with estimated outputs) for user review and approval before committing | UX | ALL | Must |
| REQ-AI-162 | Tool-Use Integration | The system SHALL equip agents with a typed tool-use interface allowing them to: read/write the Knowledge Hub, trigger diagram generation (V03), invoke requirement extraction (V07), query code scan results (V09), and call external APIs | Functional | ALL | Must |
| REQ-AI-163 | Parallel Sub-Agent Execution | The system SHALL support parallelisation of independent agent sub-tasks (e.g., simultaneously generating Business Architecture and Data Architecture artifacts) with result aggregation | Functional | EA, SA | Should |
| REQ-AI-164 | Agent State Persistence | The system SHALL persist agent execution state (plan, completed steps, intermediate outputs, tool call log) to enable resume-after-failure and audit trail reconstruction | Data | ALL | Must |
| REQ-AI-165 | Agent Output Versioning | The system SHALL version all agent-produced outputs with the agent run ID, model version, prompt version, and input artifact hashes to enable exact reproduction | Data | ALL | Must |

---

### 8.2 Human-in-the-Loop Checkpoints

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-168 | Configurable HITL Checkpoints | The system SHALL support configurable Human-in-the-Loop (HITL) checkpoints within agent workflows at: plan approval, after each major step, before Knowledge Hub writes, and before deliverable finalisation | UX | ALL | Must |
| REQ-AI-169 | Step-Level Review Interface | The system SHALL present each checkpoint to the user with: step summary, AI reasoning, output preview, and three actions: Approve / Edit & Continue / Stop — persisting the decision in the audit trail | UX | ALL | Must |
| REQ-AI-170 | Rollback to Checkpoint | The system SHALL allow the user to roll back an agent workflow to any prior HITL checkpoint, discarding downstream outputs and re-running from that point with modified inputs | Functional | ALL | Must |
| REQ-AI-171 | Automatic Halt on Ambiguity | The system SHALL automatically halt an agent workflow and request human clarification when it detects: conflicting source data, missing required input, or confidence < configurable threshold on a critical decision | AI/ML | ALL | Must |

---

### 8.3 Canonical Agent Workflows

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-175 | Repo → Target Architecture → Migration Roadmap Workflow | The system SHALL provide a canonical workflow: ingest repository → derive as-is architecture → compare to target architecture → generate gap analysis → draft migration roadmap with phased work packages | Functional | SA, EA | Must |
| REQ-AI-176 | RFP → Requirements → Architecture Workflow | The system SHALL provide a canonical workflow: ingest RFP/BRD → extract and classify requirements → generate NFRs → produce TOGAF-aligned architecture artifacts → draft HLD | Functional | EA, SA | Must |
| REQ-AI-177 | Security Posture Assessment Workflow | The system SHALL provide a canonical workflow: ingest architecture models → generate STRIDE threats → map to controls → identify gaps → produce security architecture document | Functional | SecA | Must |
| REQ-AI-178 | Data Architecture Design Workflow | The system SHALL provide a canonical workflow: ingest data requirements → generate conceptual data model → propose data architecture pattern → generate ERD + DFD → draft data dictionary | Functional | DA | Must |
| REQ-AI-179 | API Design Workflow | The system SHALL provide a canonical workflow: ingest integration requirements → propose API design → generate OpenAPI stub → produce integration sequence diagrams → draft API design document | Functional | IA | Must |
| REQ-AI-180 | Compliance Assessment Workflow | The system SHALL provide a canonical workflow: detect regulatory context → generate compliance NFR set → map requirements to controls → identify gaps → produce compliance gap report | Functional | SecA, EA | Should |
| REQ-AI-181 | Custom Workflow Builder | The system SHALL allow administrators to define custom multi-step agentic workflows by composing available agent steps, tool invocations, and HITL checkpoints via a visual or DSL-based workflow editor | Functional | ALL | Could |

---

## 9. RAG over the Knowledge Hub

### 9.1 Chunking & Embedding

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-190 | Adaptive Document Chunking | The system SHALL chunk ingested documents using a hybrid strategy: semantic chunking (paragraph/section boundaries preferred) with a fixed-size fallback (512 tokens default, configurable 128–2048), preserving heading hierarchy as chunk metadata | AI/ML | ALL | Must |
| REQ-AI-191 | Hierarchical Chunk Metadata | The system SHALL attach to every chunk: document ID, section path (e.g., "Chapter 3 > Section 3.2"), page number, creation/modification timestamp, author, confidence score, and access control tags | Data | ALL | Must |
| REQ-AI-192 | Multi-Model Embedding | The system SHALL support configurable embedding models (Amazon Titan Embeddings, OpenAI text-embedding-3-large, Cohere Embed, local sentence-transformers) with a unified vector store interface | Integration | ALL | Must |
| REQ-AI-193 | Incremental Re-Embedding | The system SHALL detect document updates (hash-based change detection) and re-embed only changed chunks, updating the vector store incrementally rather than full reindex — keeping staleness < 5 minutes of document update | AI/ML | ALL | Must |
| REQ-AI-194 | Multi-Modal Embedding | The system SHALL embed diagram images alongside their text descriptions and OCR content, enabling image-based similarity search within the Knowledge Hub | AI/ML | EA, SA | Should |

---

### 9.2 Vector Store & Retrieval

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-197 | Pluggable Vector Store | The system SHALL support pluggable vector store backends: AWS OpenSearch Serverless (default), pgvector (PostgreSQL), Qdrant, Weaviate, Pinecone — via an abstraction layer with a common upsert/query/delete API | Integration | ALL | Must |
| REQ-AI-198 | Hybrid Search (Dense + Sparse) | The system SHALL implement hybrid retrieval combining dense vector similarity (ANN) with sparse BM25/TF-IDF keyword search, using Reciprocal Rank Fusion (RRF) to merge result lists, with configurable α weight | AI/ML | ALL | Must |
| REQ-AI-199 | Metadata-Filtered Retrieval | The system SHALL support metadata-filtered retrieval: queries MAY be scoped to specific document types, date ranges, personas, architecture domains, project IDs, or classification levels — with filter expressions passed alongside the embedding query | Functional | ALL | Must |
| REQ-AI-200 | Query Expansion & Rewriting | The system SHALL apply query expansion and rewriting before retrieval: hypothetical document embedding (HyDE), synonym expansion using the project glossary, and sub-question decomposition for complex queries | AI/ML | ALL | Must |
| REQ-AI-201 | Re-Ranking | The system SHALL apply a cross-encoder re-ranker (e.g., Cohere Rerank, BGE-Reranker) to the top-K retrieved chunks before final context assembly, improving precision without increasing LLM context window cost | AI/ML | ALL | Must |
| REQ-AI-202 | Context Assembly & Truncation | The system SHALL assemble retrieved chunks into the LLM prompt context respecting the model's context window limit, prioritising by re-rank score, deduplicating overlapping content, and injecting source metadata | AI/ML | ALL | Must |

---

### 9.3 Citation & Freshness

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-205 | Inline Citations in Responses | The system SHALL include inline citations in every RAG-grounded response: [Document Title, Section, Page N] hyperlinked to the exact Knowledge Hub source, with the cited chunk highlighted on click | UX | ALL | Must |
| REQ-AI-206 | Freshness Scoring | The system SHALL compute a freshness score for each retrieved chunk based on document age and last-reviewed timestamp, down-weighting stale chunks (configurable threshold, default 90 days) and surfacing a staleness warning to the user | AI/ML | ALL | Must |
| REQ-AI-207 | Contradiction Detection in Retrieved Context | The system SHALL detect when retrieved chunks contradict each other (using NLI-based consistency check) and flag the contradiction to the user before incorporating both into the response | AI/ML | ALL | Should |
| REQ-AI-208 | RAG Attribution Report | The system SHALL generate a RAG attribution report for any response: listing each source chunk, its contribution weight, and the specific claim it supports — exportable as an audit artefact | Governance | ALL | Should |

---

## 10. Provider & Model Management

> **Provider names are illustrative, not exclusive (reconciliation C2 / master-index ruling #4):** Throughout this volume, named providers and models (AWS Bedrock, Anthropic, OpenAI, Azure OpenAI, Google Vertex AI, Cohere, Titan, Ollama/vLLM, etc.) are **examples that establish capability expectations** — they are **not a fixed or exclusive provider list**. The suite stays **provider-pluggable and cloud-neutral** via the abstraction layer (REQ-AI-215); any provider/model meeting the capability registry contract (REQ-AI-217) may be substituted or added without changing requirements.

### 10.1 Pluggable Provider Architecture

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-215 | Provider Abstraction Layer | The system SHALL implement a provider abstraction layer (interface/adapter pattern) that normalises inference calls across: AWS Bedrock (current default), Anthropic Direct API, OpenAI API, Azure OpenAI, Google Vertex AI, and local/self-hosted models (via Ollama/vLLM) | Integration | ALL | Must |
| REQ-AI-216 | Per-Task Model Configuration | The system SHALL support per-task-type model configuration: different model families may be assigned to diagram generation, requirement extraction, deliverable drafting, embeddings, re-ranking, and copilot chat — independently | Functional | ALL | Must |
| REQ-AI-217 | Model Capability Registry | The system SHALL maintain a model capability registry declaring each model's: context window, modalities (text/image/code), rate limits, pricing per token, tool-use support, streaming support, and language support | Data | ALL | Must |
| REQ-AI-218 | Automatic Model Fallback | The system SHALL automatically fall back to a configured secondary model when the primary model is unavailable (HTTP 5xx, timeout) or rate-limited, logging the fallback event and preserving request idempotency | NFR | ALL | Must |
| REQ-AI-219 | Streaming Response Support | The system SHALL support streaming responses (server-sent events) from all providers where available, delivering tokens incrementally to the UI to minimise perceived latency | NFR | ALL | Must |

---

### 10.2 Model Routing & Selection

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-222 | Cost-Aware Model Routing | The system SHALL route inference requests to the lowest-cost model that meets the task's minimum capability threshold (context window, modality, quality tier), configurable per organisation/project/persona | AI/ML | ALL | Must |
| REQ-AI-223 | Quality-Tier Model Routing | The system SHALL support a quality-tier routing policy: Draft tier (fast/cheap model), Review tier (balanced), Final tier (highest-capability model) — selectable by user at task invocation with cost estimate displayed | UX | ALL | Must |
| REQ-AI-224 | Token Budget Enforcement | The system SHALL enforce per-task token budgets (input + output combined), truncating context assembly or downgrading to a cheaper model if budget is exceeded, with user notification | Functional | ALL | Must |
| REQ-AI-225 | Task-Specific Prompt Caching | The system SHALL utilise provider-side prompt caching (Anthropic extended context caching, OpenAI prefix caching) for static system prompt segments and Knowledge Hub context blocks to reduce cost on repeated tasks | Integration | ALL | Should |

---

### 10.3 Inference Parameters & Configuration

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-228 | Per-Task Inference Parameter Profiles | The system SHALL maintain per-task inference parameter profiles (temperature, top-p, top-k, max_tokens, stop sequences, repetition penalty) with sensible defaults per task type: deterministic for schema generation (temp=0), creative for narratives (temp=0.7) | Functional | ALL | Must |
| REQ-AI-229 | JSON Mode / Structured Output Enforcement | The system SHALL use provider-native structured output modes (Anthropic tool-use, OpenAI JSON mode, Gemini response_schema) for all tasks producing structured data (requirements, diagrams specs, ADRs), eliminating parsing fragility | Functional | ALL | Must |
| REQ-AI-230 | Extended Thinking / Chain-of-Thought Control | The system SHALL support enabling extended thinking / chain-of-thought for high-complexity tasks (threat modelling, architecture decisions) with configurable thinking token budget and the ability to surface or suppress reasoning traces in the UI | AI/ML | SecA, EA | Should |

---

## 11. Guardrails & Quality

### 11.1 Hallucination Controls & Grounding

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-240 | Grounding Enforcement | The system SHALL enforce that all AI responses in grounded mode (default) make only claims supportable by retrieved Knowledge Hub content, source artifacts, or project models — any claim without a source chunk SHALL be flagged with a "no source found" indicator | AI/ML | ALL | Must |
| REQ-AI-241 | Confidence Scoring on Outputs | The system SHALL compute and expose a confidence score (0.0–1.0) for each generated requirement, architecture recommendation, and deliverable section, derived from retrieval similarity, model self-consistency, and verification checks | AI/ML | ALL | Must |
| REQ-AI-242 | Self-Consistency Verification | The system SHALL run self-consistency sampling (N=3–5 samples, configurable) on high-stakes outputs (STRIDE threats, ADRs, compliance mappings) and flag outputs where answers diverge across samples as low-confidence | AI/ML | SecA, EA | Should |
| REQ-AI-243 | Factual Cross-Check Against Knowledge Hub | The system SHALL cross-check specific factual claims (version numbers, API endpoint names, data field names, regulation clause numbers) against the Knowledge Hub corpus using exact-match and fuzzy-match before including them in output | AI/ML | ALL | Must |
| REQ-AI-244 | Refusal Handling | The system SHALL gracefully handle model refusals (safety-policy triggers): detect refusal responses, log them with the triggering prompt hash, present a user-facing message with suggested rephrasing, and never surface raw refusal text | UX | ALL | Must |

---

### 11.2 Output Validation & Schema Enforcement

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-248 | Structured Output Schema Validation | The system SHALL validate all structured AI outputs (requirement objects, ADR JSON, diagram specs, deliverable sections) against declared JSON Schema / Pydantic models before persisting, rejecting and retrying malformed outputs up to 3 times before surfacing error | Functional | ALL | Must |
| REQ-AI-249 | Architecture Notation Validity | The system SHALL validate AI-generated ArchiMate, UML, BPMN, and C4 model specifications against their formal grammars before passing to the rendering engine (V03), preventing invalid diagrams from reaching users | Functional | EA, SA | Must |
| REQ-AI-250 | Requirement ID Uniqueness Enforcement | The system SHALL enforce that AI-generated requirement IDs are unique within the project scope, detecting and resolving collisions at generation time | Functional | ALL | Must |
| REQ-AI-251 | Output Token Anomaly Detection | The system SHALL detect and flag anomalous output token patterns: excessive repetition (looping), truncated responses (max-token hit mid-sentence), and empty responses — triggering automatic retry with adjusted parameters | Functional | ALL | Must |

---

### 11.3 PII Detection & Handling

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-254 | PII Detection in Input | The system SHALL scan all user-supplied input and ingested artifacts for PII (names, email addresses, phone numbers, NI/SSN, financial account numbers, health identifiers) before sending to external LLM providers, applying configurable redaction/pseudonymisation | Security | ALL | Must |
| REQ-AI-255 | PII Redaction & Recovery | The system SHALL redact detected PII to placeholder tokens (e.g., `[PERSON_NAME_1]`) before LLM inference and optionally re-substitute originals in the final output when the requester has authorisation — maintaining a secure redaction map per session | Security | ALL | Must |
| REQ-AI-256 | Sensitive Data Handling Policy | The system SHALL enforce an organisation-level sensitive data handling policy: data classified above a configurable threshold SHALL not be sent to non-approved external providers, with local/on-prem model fallback enforced automatically | Security | SecA | Must |

---

### 11.4 Prompt Injection Defence

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-259 | Prompt Injection Detection | The system SHALL scan all user inputs and ingested artifact text for prompt injection patterns (instruction override attempts, jailbreak phrases, system-prompt extraction attempts) using both signature-based and LLM-based classifiers | Security | ALL | Must |
| REQ-AI-260 | Injection-Safe Prompt Construction | The system SHALL construct all LLM prompts with strict boundary markers between system instructions, retrieval context, and user input — never concatenating user-controlled text into instruction positions | Security | ALL | Must |
| REQ-AI-261 | Indirect Injection from Documents | The system SHALL treat all content extracted from ingested artifacts as untrusted data, applying injection scanning before including it in LLM prompts, since adversarial content may be embedded in third-party documents | Security | SecA | Must |

---

### 11.5 Evaluation Harness & Regression Tests

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-264 | Automated Evaluation Harness | The system SHALL maintain an automated evaluation harness executing nightly: requirement extraction recall/precision on labelled test documents, RAG retrieval MRR@10, diagram generation validity rate, and deliverable completeness scores — with trend dashboards | AI/ML | ALL | Must |
| REQ-AI-265 | LLM-as-Judge Evaluation | The system SHALL use an independent LLM judge (different model/provider from the generating model) to score generated outputs on: correctness, coherence, groundedness, completeness, and framework compliance — aggregating scores into a quality index | AI/ML | ALL | Must |
| REQ-AI-266 | Regression Test Suite | The system SHALL maintain a regression test suite of golden examples (input → expected output) per AI task type, automatically comparing new model/prompt versions against golden outputs and blocking deployment if regression score drops >5% | AI/ML | ALL | Must |
| REQ-AI-267 | Human Annotation Integration | The system SHALL support human annotators providing thumbs-up/thumbs-down + structured feedback on AI outputs within the product UI, feeding this data to the evaluation harness and fine-tuning pipeline. Any data routed to the **fine-tuning** pipeline SHALL be **anonymized, PII-scrubbed, and tenant-opt-in only**, and SHALL **exclude customer content** (per REQ-ENT-TRU-010, the §14 training-data scope note, and master-index ruling #2); evaluation/eval-harness use likewise SHALL NOT train on customer content. | AI/ML | ALL | Should |
| REQ-AI-268 | A/B Model Experimentation | The system SHALL support controlled A/B rollout of new model/prompt versions to a configurable fraction of users, collecting comparative quality metrics before full promotion | AI/ML | ALL | Should |

---

### 11.6 Human Review & Approval Gates

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-271 | Mandatory Review Gates for Critical Outputs | The system SHALL enforce mandatory human review gates before AI-generated content below a confidence threshold is: (a) published to the Knowledge Hub, (b) included in a finalised deliverable, or (c) committed as a baseline requirement | Governance | ALL | Must |
| REQ-AI-272 | Review Workflow Integration | The system SHALL integrate with the ESIP review workflow (V10/V11) to route low-confidence or flagged AI outputs to named reviewers with deadline tracking and escalation | Governance | ALL | Must |
| REQ-AI-273 | Audit Trail for AI Decisions | The system SHALL maintain a tamper-evident audit trail recording: every AI-generated output, the prompt sent, model/version used, retrieved context chunks, confidence score, and user decision (approve/reject/edit) — retained for minimum 7 years | Governance | ALL | Must |

---

## 12. Prompt Library & Versioning

### 12.1 Prompt Authoring & Storage

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-280 | Centralised Prompt Library | The system SHALL maintain a centralised prompt library: all system prompts, task prompts, few-shot templates, and chain-of-thought scaffolds stored as versioned artefacts with metadata (task type, model family, author, creation date, performance metrics) | Functional | ALL | Must |
| REQ-AI-281 | Prompt Version Control | The system SHALL version every prompt change with a semantic version (MAJOR.MINOR.PATCH), diff view, and rollback capability — treating prompts as first-class code artefacts in the CI/CD pipeline | Functional | ALL | Must |
| REQ-AI-282 | Prompt Template Variables | The system SHALL support parameterised prompt templates with typed variable injection (persona, project context, framework, audience, language) validated at render time to prevent missing-variable errors | Functional | ALL | Must |
| REQ-AI-283 | Few-Shot Example Management | The system SHALL manage few-shot example sets per task type: add, curate, tag by quality score, and retire examples — with automatic selection of the best N examples for a given input using semantic similarity to the request | AI/ML | ALL | Must |
| REQ-AI-284 | Prompt Testing Sandbox | The system SHALL provide a prompt testing sandbox: enter a prompt variant, select a test input, run inference, inspect output, compare against golden output, and record test result before promotion to production | UX | ALL | Must |

---

### 12.2 Per-Persona & Per-Framework Prompt Packs

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-287 | Persona Prompt Packs | The system SHALL package persona-specific prompt variants (system prompt prefix, task prompt adaptations, output format instructions) for each of the 10 supported personas, selecting the active pack based on session persona | Functional | ALL | Must |
| REQ-AI-288 | Framework Prompt Packs | The system SHALL package framework-specific prompt variants for: TOGAF, Zachman, DoDAF, MODAF, UAF, SAFe, ISO 42010, BPMN 2.0, ArchiMate 3.2, C4 — injecting framework-specific notation rules, element vocabularies, and example structures | Functional | EA | Must |
| REQ-AI-289 | Organisation-Level Prompt Customisation | The system SHALL allow organisation administrators to extend or override any base prompt pack with organisation-specific instructions, terminology, and examples — without modifying the base product prompts | Functional | ALL | Must |
| REQ-AI-290 | Multilingual Prompt Support | The system SHALL support prompts and outputs in languages configured at the project level, with locale-specific tone and terminology guidelines injected automatically | UX | ALL | Should |

---

## 13. Cost, Usage Governance & Metering

### 13.1 Usage Metering

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-300 | Token-Level Usage Metering | The system SHALL meter every inference call at token granularity: input tokens, output tokens, cached tokens, and embedding tokens — tagged by user, project, persona, task type, and provider/model | Data | ALL | Must |
| REQ-AI-301 | Real-Time Cost Attribution | The system SHALL compute real-time cost attribution in USD using provider pricing tables (updated automatically via provider API or manual schedule), attributable to cost centre / project / user | Reporting | ALL | Must |
| REQ-AI-302 | Usage Dashboard | The system SHALL provide a usage dashboard showing: daily/weekly/monthly token burn by dimension (user/project/task/model), cost trend, top cost drivers, and projected monthly spend at current rate | Reporting | ALL | Must |
| REQ-AI-303 | Cost Estimate Before Execution | The system SHALL display a cost estimate (token count + USD) to the user before executing any agentic workflow or large-batch task, requiring explicit confirmation when estimated cost exceeds a configurable threshold | UX | ALL | Must |

---

### 13.2 Budget Controls

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-306 | Per-Project Token Budgets | The system SHALL enforce configurable per-project monthly token budgets: soft warnings at 80%, hard stop at 100% (configurable), with override capability for designated administrators | Functional | ALL | Must |
| REQ-AI-307 | Per-User Token Quotas | The system SHALL enforce per-user daily and monthly token quotas, configurable by role, with self-service quota increase requests routed to administrators | Functional | ALL | Must |
| REQ-AI-308 | Cost Anomaly Alerts | The system SHALL detect and alert on cost anomalies: single-session spend >3× rolling daily average, model misconfiguration causing unexpectedly large context windows, or runaway agentic loops | Reporting | ALL | Must |
| REQ-AI-309 | Budget Forecasting | The system SHALL generate monthly budget forecasts based on trailing 30-day usage trends per project, exportable to CSV/Excel for finance reporting | Reporting | ALL | Should |

---

### 13.3 Rate Limit & Throttle Management

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-312 | Provider Rate Limit Awareness | The system SHALL track provider rate-limit consumption (requests per minute, tokens per minute) in real time and implement adaptive throttling to avoid 429 errors, with automatic back-off and retry using exponential jitter | NFR | ALL | Must |
| REQ-AI-313 | Request Queuing | The system SHALL queue inference requests when rate limits are approached, maintaining FIFO priority with optional priority elevation for interactive copilot requests over batch agentic tasks | Functional | ALL | Must |

---

## 14. Feedback & Learning Loops

> **Training-data scope (reconciliation C1 — binding on all of §14, REQ-AI-267, and any model-improvement pipeline):** Model improvement / fine-tuning SHALL use **only anonymized, PII-scrubbed behavioral signals** (ratings, accept/reject choices, edit *patterns*) with **per-tenant opt-in and explicit consent gating**. **Customer CONTENT — artifacts, documents, diagrams, queries, and their substantive text — SHALL NEVER be used to train, fine-tune, or evaluate any model.** Edit-pairs and preference data are retained as improvement candidates **only after** anonymization + PII scrub **and** under tenant opt-in; tenants that have not opted in contribute **no** training data. This implements the catalog-wide ruling and is gated by **REQ-ENT-TRU-010** (no-training-on-customer-data guarantee). See master-index §6 **ruling #2**.

### 14.1 User Feedback Collection

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-320 | Inline Feedback Capture | The system SHALL provide inline feedback mechanisms on every AI-generated output: thumbs up/down, star rating (1–5), free-text comment, and specific error category selection (factual error, irrelevant, incomplete, wrong format, hallucination) | UX | ALL | Must |
| REQ-AI-321 | Edit-as-Feedback | The system SHALL treat user edits to AI-generated content as implicit feedback: tracking the original output, the diff, and the context. Edit pairs SHALL be retained as fine-tuning candidates **only after anonymization and PII scrubbing, and only for tenants that have explicitly opted in**; the substantive **customer content** within edits SHALL NOT be used as training data (per REQ-ENT-TRU-010 and master-index ruling #2). | AI/ML | ALL | Should |
| REQ-AI-322 | Preference Data Collection | The system SHALL, when presenting multiple AI-generated options (e.g., two diagram variants), capture which option the user selects and any modifications made — building a preference dataset usable for RLHF/DPO fine-tuning | AI/ML | ALL | Could |

---

### 14.2 Continuous Improvement Pipeline

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-325 | Feedback-Driven Prompt Iteration | The system SHALL aggregate negative feedback signals by task type and prompt version, surfacing candidate prompt improvements to the prompt engineering team with supporting examples | AI/ML | ALL | Must |
| REQ-AI-326 | Few-Shot Pool Refresh | The system SHALL periodically refresh the few-shot example pools: promoting high-rated AI outputs to examples, retiring low-rated examples, and re-running evaluation harness to confirm improvement | AI/ML | ALL | Should |
| REQ-AI-327 | Model Fine-Tuning Data Export | The system SHALL export curated (input, target output, feedback score) training pairs in JSONL format for use in provider fine-tuning pipelines (e.g. Bedrock Custom Models, OpenAI fine-tuning API — provider names illustrative, not exclusive; see ruling #4). Export SHALL include **only anonymized, PII-scrubbed behavioral pairs from opted-in tenants**, and SHALL **exclude customer content** (artifacts/documents/diagrams/queries) entirely (per REQ-ENT-TRU-010 and ruling #2). PII scrubbing and consent checks SHALL be enforced before any pair is emitted. | AI/ML | ALL | Could |
| REQ-AI-328 | Evaluation Trend Reporting | The system SHALL generate monthly quality trend reports: per-task evaluation score evolution, feedback sentiment trends, retrieval quality metrics, and prompt version performance comparison | Reporting | ALL | Should |

---

## 15. Traceability AI

### 15.1 Requirements ↔ Models ↔ Deliverables Linkage

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-340 | Automatic Traceability Link Generation | The system SHALL automatically generate traceability links between: (a) source artifact fragments and extracted requirements, (b) requirements and architecture model elements, (c) architecture elements and deliverable sections, (d) requirements and acceptance criteria | Functional | ALL | Must |
| REQ-AI-341 | Traceability Matrix Generation | The system SHALL generate a Requirements Traceability Matrix (RTM) covering the full chain from business requirement → functional requirement → architecture element → deliverable section → test case, with coverage percentage per dimension | Reporting | ALL | Must |
| REQ-AI-342 | Coverage Gap Detection | The system SHALL detect requirements with no corresponding architecture element (coverage gap) and architecture elements with no backing requirement (orphan element), surfacing both as prioritised findings | AI/ML | EA, SA | Must |
| REQ-AI-343 | Change Impact Analysis | The system SHALL, when a requirement is modified, compute an AI-driven change impact analysis: which architecture elements are affected, which deliverable sections need revision, and which dependent requirements may need review | AI/ML | EA, SA | Must |
| REQ-AI-344 | Traceability Confidence Scoring | The system SHALL assign a confidence score to every auto-generated traceability link and flag low-confidence links for human verification, distinguishing auto-generated links from human-confirmed links in the RTM | AI/ML | ALL | Must |
| REQ-AI-345 | Bi-Directional Navigation | The system SHALL support bi-directional navigation of traceability links: from any requirement, navigate to all linked architecture elements and deliverable sections; from any element, navigate back to backing requirements and source artifacts | UX | ALL | Must |
| REQ-AI-346 | Traceability Export | The system SHALL export the full traceability graph in: CSV/Excel RTM format, JSON-LD for integration with external ALM tools (Jira, Azure DevOps, Polarion), and DOT/GraphML for visualisation | Integration | ALL | Should |

---

### 15.2 Cross-Volume Traceability

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-350 | AI-to-Diagram Provenance | The system SHALL record the full provenance of every generated diagram: which requirements triggered it, which AI model/prompt generated the spec, which V03 rendering step produced the output, and when it was last AI-regenerated vs. human-edited | Data | ALL | Must |
| REQ-AI-351 | Deliverable-to-Model Synchronisation | The system SHALL detect when a linked architecture model changes after a deliverable section was generated, flagging the section as "potentially stale" and offering one-click AI-driven refresh with change summary | AI/ML | ALL | Must |
| REQ-AI-352 | Portfolio Impact Propagation | The system SHALL, for portfolio-level projects (V09), propagate requirement changes across the portfolio: identifying which other projects share affected components, data entities, or integration contracts | AI/ML | EA | Should |

---

## Reconciliation Additions

> **AI review findings use the shared "AI Finding" record (reconciliation — AI-finding model):** Any AI-produced review finding emitted by this engine (e.g. security/NFR/functional issues surfaced during AI-augmented review or pre-review) SHALL be written as the **shared "AI Finding" record owned by V15** (Review & Collaboration Designer). Both V15 inline-AI review and V20 AI Pre-Review Board write to that one record type; this volume produces findings into the V15-owned schema rather than defining its own, ensuring a single consumable finding model.

## 16. AI Pipeline Performance SLOs

> **Gap closure (G-5):** Testable latency/throughput targets per AI task. These make the AI pipeline's performance contract explicit and machine-verifiable; they complement V11 platform NFRs and are referenced by the evaluation harness (REQ-AI-264). Targets assume standard tenant tier and warm models; degraded/cold-start behaviour is bounded by REQ-AI-SLO-004.

### 16.1 AI Pipeline SLO Requirements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|---|---|---|---|---|---|
| REQ-AI-SLO-001 | Requirement-Extraction Latency | The system SHALL complete AI requirement extraction in **under 30 seconds per 100 pages** of ingested document content (p95, standard tier), or stream partial results within that window for larger inputs. | NFR | ALL | Must |
| REQ-AI-SLO-002 | Diagram-Generation Latency | The system SHALL return an AI-generated diagram specification in **under 10 seconds** (p95) for a typical request (≤ 50 elements), measured from request acceptance to structured diagram payload ready for V03 rendering. | NFR | ALL | Must |
| REQ-AI-SLO-003 | Copilot Turn Latency | The system SHALL begin streaming a copilot response within **under 3 seconds** (p95) of a user turn (time-to-first-token), so interactive chat remains responsive. | NFR | ALL | Must |
| REQ-AI-SLO-004 | Concurrency & Queue-Depth Before Degradation | The system SHALL define and enforce a **maximum concurrent AI job count and queue depth per tenant** beyond which requests are queued (not dropped) with a surfaced wait estimate, and SHALL document the throughput at which the above latency SLOs are no longer guaranteed (graceful degradation, not failure). | NFR | ALL | Must |

**Rationale:** Enterprise buyers require explicit, testable AI-performance commitments; closes reconciliation gap G-5. Latency targets mirror the AI-assist intent stated elsewhere (e.g. REQ-EA-008, REQ-AI-008/REQ-MOD-364 30s timeout) and give the eval harness concrete pass/fail thresholds.

**Acceptance Criteria:**
- Extraction, diagram, and copilot p95 latencies are tracked in the evaluation/trend dashboards (REQ-AI-264, REQ-AI-328) and alert on breach.
- Load test confirms queueing (not dropping) at and above the configured concurrency cap, with a user-visible wait estimate.

---

## Appendix A — Requirement Count Summary

| Capability Area | Section | Count |
|---|---|---|
| AI Understanding & Document Comprehension | §1 | 14 |
| Requirement Intelligence | §2 | 28 |
| AI Diagram Generation | §3 | 19 |
| AI Architecture Generation per Framework | §4 | 27 |
| AI Code Analysis | §5 | 10 |
| AI Deliverable Drafting | §6 | 20 |
| Per-Persona Copilots | §7 | 17 |
| Agentic Workflows | §8 | 22 |
| RAG over the Knowledge Hub | §9 | 19 |
| Provider & Model Management | §10 | 16 |
| Guardrails & Quality | §11 | 34 |
| Prompt Library & Versioning | §12 | 11 |
| Cost, Usage Governance & Metering | §13 | 14 |
| Feedback & Learning Loops | §14 | 9 |
| Traceability AI | §15 | 13 |
| AI Pipeline Performance SLOs *(Reconciliation Addition, G-5)* | §16 | 4 |
| **TOTAL** | | **273 baseline + 4 reconciliation (SLO) = 277** |

---

## Appendix B — Priority Distribution

| Priority | Count | % |
|---|---|---|
| Must | 201 | 74% |
| Should | 58 | 21% |
| Could | 10 | 4% |
| Won't (deferred) | 4 | 1% |

---

## Appendix C — Boundary Notes & Volume Dependencies

| Boundary | Owned By | Reference |
|---|---|---|
| File parsing, format drivers, Visio/VSDX parsing | V07 — Requirements Pillar (ingestion mechanics) | V07 §3 |
| Diagram rendering engine (Mermaid/PlantUML/ArchiMate exchange → SVG/image) | V03 — ArchiMate + Diagramming Engine | V03 §4 |
| Code-scan mechanics (AST parsing, call graph construction, SAST tooling) | V09 — Portfolio & Code Scan | V09 §2–3 |
| Document rendering / template engine / export (Word/PDF/PPT output) | V10 — Deliverables | V10 §3 |
| Secrets management, provider proxy, API key rotation, network security | V11 — Platform | V11 §5 |
| TOGAF ADM phase process guidance (what to do in each phase) | V01 — TOGAF | V01 §2 |
| Framework notation rules (ArchiMate element catalogue, UML type rules) | V02 / V03 — Frameworks + Diagramming | V02 §3, V03 §2 |
| Role-based access control, persona provisioning | V05 — Role-Based Architects | V05 §4 |

---

## Appendix D — Key Patterns & Design Decisions

1. **Provider-pluggable from day one** — every AI call goes through the provider abstraction layer (REQ-AI-215); Bedrock is the current default, not a hard dependency.

2. **RAG-first, generation-second** — all copilot responses are grounded in retrieved Knowledge Hub content before the model's parametric knowledge; this bounds hallucination scope to the retrieval quality.

3. **Structured output everywhere** — provider-native JSON mode / tool-use (REQ-AI-229) eliminates regex parsing of LLM output; all structured artefacts are schema-validated before persistence (REQ-AI-248).

4. **Existing ESIP capabilities formalised** — NFR auto-generation and MoSCoW suggestion are explicitly noted as pre-existing (REQ-AI-045, REQ-AI-024); these requirements standardise their interface and quality expectations rather than introducing net-new behaviour.

5. **Agentic safety by default** — HITL checkpoints are mandatory for critical output paths (REQ-AI-168–171); autonomous execution is opt-in, not the default.

6. **Cost governance as a first-class concern** — token metering, cost attribution, budget caps, and pre-execution cost estimates (§13) are Must-priority, reflecting enterprise procurement realities.

7. **Traceability as the connective tissue** — §15 closes the loop: every AI output is traceable to its source, enabling regulatory audit, change impact, and architectural integrity checks.
