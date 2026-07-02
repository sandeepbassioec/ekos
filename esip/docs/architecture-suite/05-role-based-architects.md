# ESIP Architecture Automation Suite — Volume 05: Role-Based Architects + Extensibility

**Prefix:** ROL  
**Status:** V1.0 — authored 2026-06-23  
**Scope:** Application · Integration · API · CRM · Data · Security · Cloud · Infrastructure/Network · Microservices/Platform/DevOps · Mobile (brief) · AI/ML Architect (brief) · Extensibility ("Any Architect")  
**Boundary notes:**
- Metamodel engine internals → Volume 11 (here: state what role-extensibility must expose)
- AI copilot capabilities (prompt libraries, extraction models, generation models) → Volume 06 (here: requirements on *what* AI must do per role)
- Shared diagram engine canvas, renderers, auto-layout → Volume 03 (here: which diagram types each role needs)
- Deliverable document/PPT/Excel generators → Volume 10 (here: which deliverable templates each role requires)
- Enterprise Architecture (TOGAF/ADM/capability maps) → Volume 01
- Solution Architecture (options, ADR, HLD/LLD) → Volume 04
- Requirements pillar (ingestion, RTM, quality) → Volume 07
- Knowledge Hub (connectors, search) → Volume 08
- Portfolio, code-scan, sunset → Volume 09
- Platform NFRs, RBAC, audit, multi-tenancy → Volume 11

---

## Table of Contents

1. [Application Architect (P3)](#1-application-architect-p3)
2. [Integration Architect (P4)](#2-integration-architect-p4)
3. [API Architect (P5)](#3-api-architect-p5)
4. [CRM Architect (P6)](#4-crm-architect-p6)
5. [Data Architect (P7)](#5-data-architect-p7)
6. [Security Architect (P8)](#6-security-architect-p8)
7. [Cloud Architect (P9)](#7-cloud-architect-p9)
8. [Infrastructure / Network Architect (P10)](#8-infrastructure--network-architect-p10)
9. [Microservices / Platform / DevOps Architect (P11)](#9-microservices--platform--devops-architect-p11)
10. [Mobile Architect (brief)](#10-mobile-architect-brief)
11. [AI/ML Architect (brief)](#11-aiml-architect-brief)
12. [Role Extensibility — "Any Architect"](#12-role-extensibility--any-architect)

---

## 1. Application Architect (P3)

**Role summary:** Defines application structure, component decomposition, technology selection, API/data contracts, and application rationalization decisions. Primary consumer of the existing ESIP baseline.

**Key activities:** Component design, pattern selection (layered, hexagonal, event-driven, CQRS/ES), technology stack decisions, UML/C4 modeling, application portfolio entry management, API contract authoring, architecture decision records (ADRs), NFR definition, estimation, code review for architectural compliance.

**Signature artifacts:** Component diagrams (C4 L1–L4, UML), sequence/interaction diagrams, class/object diagrams, ADRs, HLD/LLD documents, API/data contracts, technology inventory, application profile, NFR specification, WBS/estimation sheet.

**Diagram types needed (→ V03):** C4 Context, Container, Component, Code; UML Sequence, Class, State, Activity, Package, Deployment; ERD (logical); Flowchart; Mind-map (pattern brainstorm).

**Deliverable templates needed (→ V10):** Application Design Document, ADR log, Component Catalog, Technology Radar, NFR Register.

---

### 1.1 Application Structure & Component Modeling

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-APP-001 | Component Catalog | The suite SHALL provide a structured Application Component Catalog where an Application Architect can define, describe, and version-control named components with attributes: name, description, technology, stereotype (UI/Service/Repository/Gateway/etc.), owning team, lifecycle status. | Functional | P3 | Must | Core ESIP baseline; aligns with ArchiMate Application Component |
| REQ-ROL-APP-002 | Component Relationship Mapping | The suite SHALL allow an Application Architect to define typed relationships between components (calls, depends-on, publishes-to, subscribes-from, exposes, realises, composes) and render them on a component diagram. | Functional | P3 | Must | ArchiMate 3.2 Application Layer relationships |
| REQ-ROL-APP-003 | Pattern Library | The suite SHALL include a curated Pattern Library of architectural patterns (Layered, Hexagonal/Ports-and-Adapters, CQRS/ES, Saga, Outbox, BFF, Strangler Fig, Sidecar, Circuit Breaker, Bulkhead, Retry/Backoff) that an Application Architect can attach to a component or subsystem. | Functional | P3 | Must | GoF, POSA, EIP, Gregor Hohpe; enables consistent terminology |
| REQ-ROL-APP-004 | AI Pattern Suggestion | The suite SHALL invoke the AI engine to suggest applicable architectural patterns based on the problem statement, NFRs, and existing component topology described in the project knowledge base. | AI/ML | P3 | Should | V06 AI engine; productivity multiplier |
| REQ-ROL-APP-005 | C4 Model Authoring | The suite SHALL support authoring and auto-layout of all four C4 levels (Context, Container, Component, Code) with correct semantic constraints (e.g., Code-level shows classes/files, not systems). | Functional | P3 | Must | C4 Model (Simon Brown); widely adopted lightweight notation |
| REQ-ROL-APP-006 | UML Diagram Suite | The suite SHALL support authoring of UML 2.5 Class, Sequence, State Machine, Activity, Package, Deployment, Use Case, and Component diagrams with syntactic validation. | Functional | P3 | Must | OMG UML 2.5; baseline ESIP capability |
| REQ-ROL-APP-007 | Technology Stack Inventory | The suite SHALL maintain a Technology Stack Inventory per application (programming languages, frameworks, databases, message brokers, caches, auth libraries, observability tools) with version and lifecycle status (current/strategic/sunset). | Functional | P3 | Must | Technology Radar; APM foundation → feeds V09 portfolio |
| REQ-ROL-APP-008 | Application Rationalization Tagging | The suite SHALL allow tagging each application in the portfolio with a TIME disposition (Tolerate/Invest/Migrate/Eliminate) and a 6-R cloud migration flag (Retain/Rehost/Replatform/Refactor/Repurchase/Retire) with justification text. | Functional | P3, P1 | Must | Gartner TIME; AWS 6 R's; feeds V09 portfolio |
| REQ-ROL-APP-009 | Architecture Decision Record (ADR) Management | The suite SHALL provide a structured ADR management capability where an Application Architect can create, link (to requirements and components), status-track (Proposed/Accepted/Deprecated/Superseded), and query ADRs; each ADR SHALL capture context, decision, consequences, and alternatives considered. | Functional | P3, P2 | Must | Michael Nygard ADR format; traceability to V07 RTM |
| REQ-ROL-APP-010 | NFR Specification per Application | The suite SHALL allow an Application Architect to specify, classify (ISO/IEC 25010: Performance, Reliability, Usability, Security, Maintainability, Portability, Scalability, etc.), and prioritise Non-Functional Requirements per application, with measurable acceptance criteria. | Functional | P3, P2 | Must | ISO/IEC 25010; NFR-to-component traceability |
| REQ-ROL-APP-011 | Estimation & WBS Integration | The suite SHALL support a Work Breakdown Structure (WBS) template per application component, allowing size/complexity tagging and effort-hour estimation, with roll-up to project-level estimates. | Functional | P3 | Should | Standard delivery planning |
| REQ-ROL-APP-012 | AI-Assisted Component Generation | The suite SHALL allow an Application Architect to describe a use case in natural language and invoke the AI engine to generate a draft component diagram, suggest component names, stereotypes, and relationships, which the architect can accept, edit, or reject. | AI/ML | P3 | Should | V06 AI engine; reduces cold-start time |
| REQ-ROL-APP-013 | Code-to-Component Reverse Engineering | The suite SHALL integrate with the code-scanning engine (V09) to infer an application's component structure from source repositories and populate the Component Catalog with detected layers, modules, and dependencies. | Integration | P3 | Should | V09 code scan; automated baseline derivation |
| REQ-ROL-APP-014 | Cross-Application Dependency Map | The suite SHALL render a cross-application dependency map (aggregated from component relationships) showing which applications call, share data with, or depend on other applications in the portfolio. | Functional | P3, P1 | Must | Portfolio intelligence; impact analysis |
| REQ-ROL-APP-015 | Architecture Compliance Check | The suite SHALL allow defining architectural fitness functions (e.g., "no UI layer shall directly call the database layer") and SHALL flag violations detected via code-scan or manual declaration. | Governance | P3 | Should | Evolutionary Architecture (Ford/Parsons); prevents drift |

---

### 1.2 API & Data Contracts (Application Layer)

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-APP-016 | Internal API Contract Authoring | The suite SHALL support authoring OpenAPI 3.x contracts for internal service APIs from the Application Architect's component view, with round-trip synchronisation between the component model and the API contract. | Functional | P3, P5 | Must | OpenAPI Spec; shared with API Architect role |
| REQ-ROL-APP-017 | Data Contract per Component | The suite SHALL allow an Application Architect to define data contracts (schema, validation rules, nullability, example payloads) for each component's input and output, stored and version-controlled as artefacts in the project. | Functional | P3, P7 | Should | Data-Mesh data-contract pattern; quality gate |
| REQ-ROL-APP-018 | Schema Conflict Detection | The suite SHALL detect naming, type, or cardinality conflicts between data contracts defined by different components or teams and alert the Application Architect. | Governance | P3 | Should | Prevents silent incompatibilities at integration seams |

---

### 1.3 Application Architecture Deliverables

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-APP-019 | Auto-Generate Application Design Document | The suite SHALL auto-generate an Application Design Document (Word/PDF) from the component catalog, pattern annotations, ADRs, NFRs, and diagrams with configurable section template. | Functional | P3 | Must | V10 deliverable engine; architect's primary artefact |
| REQ-ROL-APP-020 | Diagram Export for Design Doc | The suite SHALL embed or link exported diagram images (PNG/SVG) of C4 and UML diagrams into the generated Application Design Document automatically. | Functional | P3 | Must | V10 / V03 integration |
| REQ-ROL-APP-021 | Technology Radar Export | The suite SHALL export the Technology Stack Inventory as a Thoughtworks-style Technology Radar (Rings: Adopt/Trial/Assess/Hold) in SVG and HTML formats. | Reporting | P3, P1 | Could | Thoughtworks Tech Radar format; communication artefact |

---

## 2. Integration Architect (P4)

**Role summary:** Designs and governs the integration layer across applications — messaging patterns, ESB/iPaaS topologies, event-driven architectures, B2B/EDI, canonical data models, error handling, and idempotency strategies.

**Key activities:** Integration pattern selection from EIP catalog, message/event flow design, ESB/iPaaS topology design (MuleSoft, Azure Integration Services, AWS EventBridge/SQS/SNS, Kafka, IBM MQ, TIBCO, WSO2), canonical/common data model definition, B2B/EDI partner onboarding, error-handling & retry design, choreography vs. orchestration decisions, integration testing strategy, monitoring/alerting design.

**Signature artifacts:** Integration pattern catalog, message flow diagrams, ESB/iPaaS topology diagrams, canonical data model (CDM), B2B partner map, error-handling matrix, idempotency design, integration ADRs, API-to-system mapping.

**Diagram types needed (→ V03):** Integration flow diagrams, sequence diagrams, event flow/topology diagrams, BPMN process flows, ERD (CDM), network diagrams (integration topology).

**Deliverable templates needed (→ V10):** Integration Architecture Document, Canonical Data Model specification, B2B/EDI partner onboarding guide, Error-handling matrix.

---

### 2.1 Enterprise Integration Patterns (EIP) Catalog

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-INT-001 | EIP Pattern Library | The suite SHALL include a built-in EIP library covering all 65 patterns from Hohpe & Woolf (Message Channel, Message Router, Translator, Aggregator, Splitter, Filter, Enricher, Normaliser, Scatter-Gather, Saga, etc.) with descriptions, use-case guidance, and diagram primitives. | Functional | P4 | Must | Hohpe & Woolf "Enterprise Integration Patterns"; Camel/MuleSoft reference |
| REQ-ROL-INT-002 | EIP Annotation on Diagrams | The suite SHALL allow each node in an integration flow diagram to be annotated with one or more EIP pattern names, rendered with the standard Hohpe/Woolf pipe-and-filter iconography. | Functional | P4 | Must | EIP visual language; team legibility |
| REQ-ROL-INT-003 | AI EIP Recommendation | The suite SHALL invoke the AI engine to recommend applicable EIP patterns for a described integration scenario (source, transformation, target, constraints) extracted from the project knowledge base. | AI/ML | P4 | Should | V06 AI engine; accelerates design |
| REQ-ROL-INT-004 | Idempotency Design Checklist | The suite SHALL provide a built-in idempotency design checklist (message deduplication key design, at-least-once vs exactly-once, idempotent consumer pattern, outbox/inbox table approach) that the Integration Architect can complete and attach to each integration flow. | Functional | P4 | Must | Critical correctness concern; EIP outbox/inbox pattern |
| REQ-ROL-INT-005 | Error Handling Matrix | The suite SHALL support authoring an Error Handling Matrix per integration flow, capturing error category (transient/permanent/poison-message), retry policy (max attempts, backoff, jitter), dead-letter destination, alerting threshold, and remediation SLA. | Functional | P4 | Must | Production resilience; EIP dead-letter channel |

---

### 2.2 Message & Event Flow Design

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-INT-006 | Integration Flow Diagram Authoring | The suite SHALL support authoring integration flow diagrams that show source system, transformations, routing logic, target systems, queues/topics, and protocols, with EIP annotation. | Functional | P4 | Must | Core artefact for integration design |
| REQ-ROL-INT-007 | Event-Driven Architecture (EDA) Design | The suite SHALL support modelling event-driven architectures including event producers, consumers, event types (with schema), topics/partitions, choreography flows, and event-sourcing patterns on a dedicated EDA canvas. | Functional | P4 | Must | AsyncAPI 3.x; CNCF Serverless; Kafka/EventBridge patterns |
| REQ-ROL-INT-008 | AsyncAPI Contract Authoring | The suite SHALL support authoring and validating AsyncAPI 3.x contracts for asynchronous/event-based APIs (Kafka, AMQP, MQTT, WebSocket) with the same contract-management workflow as OpenAPI contracts. | Functional | P4, P5 | Must | AsyncAPI 3.x specification |
| REQ-ROL-INT-009 | Message Schema Registry | The suite SHALL maintain a message/event schema registry (Avro, JSON Schema, Protobuf) per integration project, with versioning, compatibility mode (backward/forward/full), and impact analysis on consumers for schema changes. | Data | P4 | Must | Confluent Schema Registry pattern; prevents breaking changes |
| REQ-ROL-INT-010 | Orchestration vs Choreography Decision | The suite SHALL provide a structured decision aid (trade-off table, criteria checklist) for choosing between orchestration (Saga orchestrator, BPM engine) and choreography (event-driven, Saga choreography) and persist the decision as an ADR. | Functional | P4 | Should | Common design decision; ADR traceability |
| REQ-ROL-INT-011 | Protocol Matrix | The suite SHALL maintain a Protocol Matrix per integration, documenting transport protocol (HTTP/REST, AMQP, MQTT, SFTP, AS2, SOAP/WS, gRPC, JMS), serialization format (JSON, XML, Avro, Protobuf, CSV, EDI X12, EDIFACT), and encoding (UTF-8, ISO-8859, binary). | Data | P4 | Must | Foundational integration metadata |

---

### 2.3 ESB / iPaaS Design

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-INT-012 | ESB/iPaaS Topology Diagram | The suite SHALL support designing ESB/iPaaS topology diagrams showing the integration platform (MuleSoft, Azure Logic Apps/Service Bus, AWS EventBridge/SQS/SNS/Step Functions, IBM MQ, TIBCO, WSO2, Boomi, Informatica IICS) with connectors to source and target systems. | Functional | P4 | Must | Multi-platform; reflects real enterprise diversity |
| REQ-ROL-INT-013 | Integration Platform Capability Model | The suite SHALL include a capability model for each supported iPaaS platform listing available connectors, trigger types, transformation capabilities, governance features, and pricing tier constraints, to aid platform selection. | Functional | P4 | Should | Platform selection guidance |
| REQ-ROL-INT-014 | API-Led Connectivity Model | The suite SHALL support modelling MuleSoft-style API-led connectivity (System APIs, Process APIs, Experience APIs) as a three-tier diagram with data flow annotations. | Functional | P4 | Must | MuleSoft API-led connectivity; widely adopted in enterprise |
| REQ-ROL-INT-015 | Mediator / Adapter Pattern Library | The suite SHALL provide a library of integration adapter patterns (File Adapter, DB Adapter, FTP Adapter, HTTP Adapter, JMS Adapter, SAP BAPI Adapter, Salesforce Adapter) to be placed on integration topology diagrams. | Functional | P4 | Should | Standard integration taxonomy |

---

### 2.4 Canonical / Common Data Model

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-INT-016 | Canonical Data Model Authoring | The suite SHALL provide a dedicated Canonical Data Model (CDM) editor where the Integration Architect can define canonical entity types, attributes, and relationships that are the authoritative "hub" schemas for the enterprise integration layer. | Data | P4, P7 | Must | CDM/CIM pattern; prevents point-to-point schema proliferation |
| REQ-ROL-INT-017 | CDM Mapping Tables | The suite SHALL support authoring mapping tables between each source/target system schema and the CDM, capturing field name, type, transformation expression, mandatory/optional, and any lookup/code translation. | Data | P4 | Must | ETL/ELT mapping artefact; traceability |
| REQ-ROL-INT-018 | CDM Versioning & Governance | The suite SHALL version-control CDM schemas, require a change rationale for any breaking change, notify the owning Integration Architect, and list impacted integration flows. | Governance | P4 | Must | Change management; prevents silent breaking changes |
| REQ-ROL-INT-019 | CDM–to–System Schema Consistency Check | The suite SHALL detect when a source system's actual schema (imported via code-scan or manual entry) deviates from its CDM mapping and alert the Integration Architect. | Governance | P4 | Should | Automated drift detection |

---

### 2.5 B2B / EDI

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-INT-020 | B2B Partner Registry | The suite SHALL maintain a B2B Partner Registry listing each external trading partner with: name, type (customer/supplier/carrier/marketplace), protocol (AS2, SFTP, VAN, API), EDI standard (X12, EDIFACT, RosettaNet, OAGIS), ISA/GS identifiers, and contact. | Functional | P4 | Must | B2B integration governance; onboarding repeatability |
| REQ-ROL-INT-021 | EDI Transaction Set Catalog | The suite SHALL include a reference catalog of commonly used EDI transaction sets (X12 850 PO, 855 PO Ack, 856 ASN, 810 Invoice, 997 FA; EDIFACT ORDERS, ORDRSP, DESADV, INVOIC) with field descriptions. | Functional | P4 | Should | EDI reference knowledge; reduces lookup friction |
| REQ-ROL-INT-022 | B2B Partner Onboarding Checklist | The suite SHALL auto-generate a B2B partner onboarding checklist from a template (protocol setup, certificate exchange, test transaction, acknowledgement loop, cutover) when a new partner is added to the registry. | Functional | P4 | Should | Operational repeatability; reduces onboarding errors |

---

### 2.6 Integration Governance & Deliverables

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-INT-023 | Integration Inventory | The suite SHALL maintain an Integration Inventory listing every integration in the portfolio (source, target, pattern, platform, protocol, SLA, owner, status) as a searchable, filterable registry. | Functional | P4, P1 | Must | Portfolio governance; impact analysis |
| REQ-ROL-INT-024 | Integration Health Dashboard | The suite SHALL provide an Integration Health Dashboard showing integration count by status (active/deprecated/failed), pattern distribution, platform distribution, and SLA breach risk. | Reporting | P4 | Should | Governance visibility |
| REQ-ROL-INT-025 | Auto-Generate Integration Architecture Document | The suite SHALL auto-generate an Integration Architecture Document from the integration inventory, flow diagrams, CDM, error-handling matrix, and protocol matrix. | Functional | P4 | Must | V10 deliverable engine |

---

## 3. API Architect (P5)

**Role summary:** Defines and governs the organisation's API strategy, design standards, contract lifecycle, gateway/policy configuration, developer experience, monetisation model, and API governance across REST, GraphQL, gRPC, and asynchronous APIs.

**Key activities:** API strategy definition, API catalog maintenance, OpenAPI/AsyncAPI/GraphQL/gRPC contract authoring and review, versioning policy, gateway policy design (auth, rate-limit, quota, throttle, caching, transformation), developer portal management, API security (OAuth 2.x, OIDC, JWT, mTLS, API keys), monetisation model, linting and standards enforcement, deprecation planning.

**Signature artifacts:** API catalog, OpenAPI contracts, AsyncAPI contracts, GraphQL schema, gRPC .proto files, gateway policy configurations, developer portal, API versioning policy, API monetisation model, API governance standards doc, deprecation register.

**Diagram types needed (→ V03):** API topology (gateway/backend), sequence (API call flows), event flow (async APIs), ER diagram (resource models).

**Deliverable templates needed (→ V10):** API Catalog report, API Governance Standards document, API Design Guide, Gateway Policy Specification.

---

### 3.1 API Strategy & Catalog

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-API-001 | API Catalog | The suite SHALL provide an API Catalog where every API (internal, partner, public) is registered with: name, version, type (REST/GraphQL/gRPC/AsyncAPI), status (design/beta/stable/deprecated/retired), owner team, SLA tier, and contract link. | Functional | P5 | Must | API-first strategy; Gartner API management reference |
| REQ-ROL-API-002 | API Strategy Document Generator | The suite SHALL auto-generate an API Strategy Document capturing API vision, target architecture (internal/partner/open), governance model, tooling choices, and KPIs, from structured intake answers. | Functional | P5 | Should | Executive communication artefact |
| REQ-ROL-API-003 | API Lifecycle State Machine | The suite SHALL enforce a configurable API lifecycle state machine (Design → Review → Beta → Stable → Deprecated → Retired) with role-based transition approvals and automated notifications to consumer teams on deprecation. | Governance | P5 | Must | Lifecycle governance; prevents orphaned APIs |
| REQ-ROL-API-004 | API Deprecation Register | The suite SHALL maintain an API Deprecation Register listing all deprecated APIs with sunset date, migration target API, consumer list, and communication history. | Governance | P5 | Must | Consumer communication; technical-debt management |
| REQ-ROL-API-005 | API Dependency Map | The suite SHALL render an API Dependency Map showing which applications/services consume which APIs, enabling impact analysis for versioning and deprecation decisions. | Functional | P5, P3 | Must | Impact analysis; deprecation safety |

---

### 3.2 Contract Design — OpenAPI / AsyncAPI / GraphQL / gRPC

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-API-006 | OpenAPI 3.x Contract Editor | The suite SHALL provide a visual and YAML/JSON editor for OpenAPI 3.x contracts with syntax validation, schema $ref resolution, example generation, and render of interactive documentation. | Functional | P5, P3 | Must | OpenAPI Initiative; de-facto REST contract standard |
| REQ-ROL-API-007 | AsyncAPI 3.x Contract Editor | The suite SHALL provide an editor for AsyncAPI 3.x contracts for event/message-driven APIs, with schema validation, channel/operation modelling, and render of documentation. | Functional | P5, P4 | Must | AsyncAPI Initiative |
| REQ-ROL-API-008 | GraphQL Schema Editor | The suite SHALL provide a GraphQL SDL editor with schema validation, type relationship visualisation, and documentation generation from schema comments. | Functional | P5 | Should | GraphQL Foundation; growing adoption |
| REQ-ROL-API-009 | gRPC / Protobuf Editor | The suite SHALL provide a .proto file editor with syntax validation, service/message visualisation, and documentation generation, with stub generation capability invocation. | Functional | P5 | Should | gRPC / Protocol Buffers; dominant in internal microservices |
| REQ-ROL-API-010 | Contract Linting & Standards Enforcement | The suite SHALL lint every API contract against a configurable standards ruleset (Spectral-compatible rules) covering naming conventions, HTTP verb usage, pagination, error response shapes, and security scheme requirements. | Governance | P5 | Must | Stoplight Spectral; API style guide enforcement |
| REQ-ROL-API-011 | Contract Version Diff | The suite SHALL diff two versions of an API contract and classify each change as non-breaking (additive) or breaking (removal, type change, constraint tightening), and block promotion to Stable if breaking changes are present without a major version bump. | Governance | P5 | Must | Semantic versioning; consumer protection |
| REQ-ROL-API-012 | Mock Server Generation | The suite SHALL auto-generate a mock server (Prism-style) from an OpenAPI or AsyncAPI contract so that consumer teams can develop against the API before the provider is implemented. | Functional | P5 | Should | Design-first API development; reduces blocking |
| REQ-ROL-API-013 | AI API Contract Generation | The suite SHALL invoke the AI engine to generate a first-draft OpenAPI or AsyncAPI contract from a natural-language description of the API's purpose, resources, operations, and error cases. | AI/ML | P5 | Should | V06 AI engine; reduces contract cold-start time |

---

### 3.3 API Versioning & Lifecycle

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-API-014 | Versioning Strategy Configuration | The suite SHALL allow the API Architect to configure and enforce a versioning strategy per API domain: URI path versioning (v1/, v2/), header versioning (Accept: application/vnd.x.v1+json), or query-parameter versioning, with consistent application across the catalog. | Governance | P5 | Must | API versioning best practices (Fielding, Richardson) |
| REQ-ROL-API-015 | Consumer Notification on Breaking Change | The suite SHALL automatically notify registered consumer teams (via in-app alert and email) when a breaking API change is detected or a deprecation notice is set. | Governance | P5 | Must | Consumer protection SLA |
| REQ-ROL-API-016 | Multi-Version Coexistence Tracking | The suite SHALL track which API versions are simultaneously live, when each was introduced, how many consumers are on each version, and the planned retirement date for older versions. | Governance | P5 | Must | Transition period management |

---

### 3.4 Gateway, Policy & Security

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-API-017 | Gateway Policy Design | The suite SHALL support designing and documenting API gateway policies per API (or API product) including: authentication scheme, authorisation scopes, rate-limit (requests/min/hour/day), quota (monthly), throttling, request/response transformation, caching TTL, and CORS configuration. | Functional | P5 | Must | Kong, Apigee, AWS API GW, Azure APIM, Mulesoft patterns |
| REQ-ROL-API-018 | OAuth 2.x / OIDC Flow Design | The suite SHALL provide a visual OAuth 2.x / OIDC flow designer showing authorization_code + PKCE, client_credentials, device_code, and implicit (legacy) flows, with correct token exchange steps and scope definitions. | Functional | P5, P8 | Must | RFC 6749, 7636, 8252; OIDC Core; security correctness |
| REQ-ROL-API-019 | JWT Claim Specification | The suite SHALL allow the API Architect to define required JWT claims (iss, sub, aud, exp, iat, custom scopes) per API and generate a claim validation specification for the gateway and back-end services. | Functional | P5, P8 | Must | RFC 7519 JWT; security contract |
| REQ-ROL-API-020 | mTLS Certificate Management Design | The suite SHALL support documenting mTLS requirements per API (client cert CA, server cert SAN, pinning policy, rotation schedule) as a security design artefact. | Security | P5, P8 | Should | High-security internal/partner API pattern |
| REQ-ROL-API-021 | API Security Threat Model Integration | The suite SHALL link each API in the catalog to a threat model (V08 Security Architect) and surface threat findings as API design review items. | Security | P5, P8 | Should | OWASP API Security Top 10; cross-role traceability |
| REQ-ROL-API-022 | OWASP API Security Top 10 Checklist | The suite SHALL provide an OWASP API Security Top 10 checklist (2023) per API contract, allowing the API Architect to assess and record mitigation status for each risk. | Security | P5, P8 | Must | OWASP API Security Project 2023 |

---

### 3.5 Developer Portal & Monetisation

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-API-023 | Developer Portal Design Blueprint | The suite SHALL support authoring a Developer Portal Design Blueprint specifying portal information architecture (API catalog browse, search, onboarding, sandbox, support) and content requirements per API. | Functional | P5 | Should | Developer experience (DX) standard practice |
| REQ-ROL-API-024 | API Product Bundling | The suite SHALL allow the API Architect to define API Products (bundles of one or more APIs with a SLA tier and access control policy) as the unit exposed to consumers, mirroring Apigee/APIM product model. | Functional | P5 | Should | Apigee, Azure APIM product model; monetisation foundation |
| REQ-ROL-API-025 | Monetisation Model Design | The suite SHALL support documenting a monetisation model per API Product (freemium tiers, pay-per-call, subscription, revenue-share) as a structured artefact linked to the gateway quota policy. | Functional | P5 | Could | API economy; revenue stream design |

---

### 3.6 API Governance & Reporting

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-API-026 | API Governance Dashboard | The suite SHALL provide an API Governance Dashboard showing: total APIs by type/status, contract linting pass/fail rate, versioning policy compliance %, OWASP checklist completion %, deprecation countdown, and consumer count per API. | Reporting | P5 | Must | Governance visibility; executive reporting |
| REQ-ROL-API-027 | API Design Review Workflow | The suite SHALL support a structured API Design Review workflow where a contract is submitted for peer review, reviewers add comments at field level, and approval gates (lint pass + security checklist + reviewer sign-off) are enforced before promotion to Beta. | Governance | P5 | Must | Design-first API maturity |
| REQ-ROL-API-028 | Auto-Generate API Design Guide | The suite SHALL auto-generate an organisation-specific API Design Guide document from the configured linting ruleset, versioning strategy, security schemes, and naming conventions. | Functional | P5 | Should | V10 deliverable engine; living style guide |

---

## 4. CRM Architect (P6)

**Role summary:** Designs CRM platform architecture (Salesforce, Microsoft Dynamics 365, HubSpot, or others) including object/data model, automation/flows, org topology, integration strategy, data migration, and security model.

**Key activities:** CRM platform selection, object/data model design (standard + custom objects), automation design (workflow rules, flows, process builder/Power Automate), org strategy (single-org vs. multi-org, sandboxes), integration with ERP/backend, data migration plan (ETL, delta loads, cutover), security model (profiles, permission sets, record-level sharing rules, field-level security), AppExchange/extension evaluation.

**Signature artifacts:** CRM object model diagram, automation flow diagrams, org topology diagram, integration map (CRM ↔ ERP/backend), data migration plan, security model matrix, CRM architecture document.

**Diagram types needed (→ V03):** ERD (CRM object model), BPMN (automation flows), topology diagram (org structure), integration flow diagram.

**Deliverable templates needed (→ V10):** CRM Architecture Document, Data Migration Plan, Security Model Matrix, Integration Map.

---

### 4.1 CRM Object & Data Modeling

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-CRM-001 | CRM Platform Profile | The suite SHALL maintain a CRM Platform Profile per project capturing platform (Salesforce Sales/Service/Marketing/Experience Cloud, Dynamics 365 Sales/CE/F&O, HubSpot, SAP CRM, custom), edition/tier, region, and org URL. | Functional | P6 | Must | Multi-platform; foundation for all CRM artefacts |
| REQ-ROL-CRM-002 | Standard Object Catalog | The suite SHALL include a built-in Standard Object Catalog for Salesforce (Account, Contact, Lead, Opportunity, Case, Product2, Pricebook2, Order, etc.) and Dynamics 365 (Account, Contact, Lead, Opportunity, Incident, etc.) with field lists, data types, and relationship maps. | Functional | P6 | Must | Platform knowledge base; accelerates design |
| REQ-ROL-CRM-003 | Custom Object Designer | The suite SHALL provide a Custom Object Designer where the CRM Architect can define custom objects with label, API name, fields (data type, length, formula, lookup/master-detail relationship, external ID), and record type configuration. | Functional | P6 | Must | Core CRM design activity |
| REQ-ROL-CRM-004 | Object Relationship Diagram | The suite SHALL render an Object Relationship Diagram (ERD-style) from the CRM object model (standard + custom) showing lookup, master-detail, and many-to-many (junction object) relationships with cardinality. | Functional | P6 | Must | Visual artefact; team communication |
| REQ-ROL-CRM-005 | Field-Level Data Dictionary | The suite SHALL auto-generate a Field-Level Data Dictionary from the object model listing every field with API name, label, data type, length/precision, formula, required flag, unique flag, external ID flag, and description. | Data | P6 | Must | Governance; migration foundation |
| REQ-ROL-CRM-006 | Governor Limit Awareness | The suite SHALL surface relevant platform governor limits (Salesforce: SOQL query rows, DML statements/transaction, heap size, CPU time; Dynamics: API call limits per day) as contextual warnings when the object model or automation design approaches a limit. | Functional | P6 | Should | Salesforce Governor Limits reference; prevents production failures |
| REQ-ROL-CRM-007 | AI Object Model Suggestion | The suite SHALL invoke the AI engine to suggest a CRM object model (standard + custom objects, fields, relationships) from a business domain description and requirements extracted from the project knowledge base. | AI/ML | P6 | Should | V06 AI engine; cold-start acceleration |

---

### 4.2 Automation, Flows & Business Process

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-CRM-008 | Automation Inventory | The suite SHALL maintain an Automation Inventory listing every automation artefact (Salesforce: Flows, Workflow Rules, Process Builder, Apex Triggers, Scheduled Jobs; Dynamics: Power Automate flows, Business Rules, plugins) with trigger object, trigger event, active status, and owner. | Functional | P6 | Must | Automation governance; prevents conflicts/order-of-execution issues |
| REQ-ROL-CRM-009 | Automation Flow Diagram | The suite SHALL support designing automation logic as BPMN-style flow diagrams (trigger, decision, action, record update, email alert, subflow call) linked to the automation inventory entry. | Functional | P6 | Should | Visual design before declarative/code implementation |
| REQ-ROL-CRM-010 | Order-of-Execution Checker | The suite SHALL document and validate Salesforce order-of-execution (before triggers, system validations, before save flows, after triggers, assignment rules, auto-response rules, workflow rules, escalation rules, after save flows, roll-up summaries) and flag automation conflicts. | Governance | P6 | Must | Salesforce Order of Execution; critical correctness |
| REQ-ROL-CRM-011 | Record Type Strategy | The suite SHALL allow the CRM Architect to define Record Types per object with page layout assignments, business process (for Leads/Opportunities/Cases/Contracts), profile visibility, and use-case rationale. | Functional | P6 | Should | CRM configuration pattern; differentiated UX |

---

### 4.3 Org Strategy & Topology

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-CRM-012 | Org Topology Diagram | The suite SHALL support authoring an Org Topology Diagram showing production org(s), sandbox strategy (Full/Partial/Developer/Scratch), environment promotion path (Scratch → Dev → SIT → UAT → Prod), and deployment tooling (SFDX, Copado, Gearset, Azure DevOps). | Functional | P6 | Must | Multi-org governance; release management |
| REQ-ROL-CRM-013 | Single-org vs Multi-org Decision Aid | The suite SHALL provide a structured decision aid (criteria matrix: data isolation, team autonomy, licensing, integration complexity, governance overhead) for single-org vs. multi-org strategy with ADR capture. | Functional | P6 | Should | Common architectural decision; costly to reverse |
| REQ-ROL-CRM-014 | AppExchange / Extension Register | The suite SHALL allow registration of third-party managed packages (AppExchange ISVs) and custom-developed packages with: namespace, version, license cost, objects/fields/apex added, upgrade risk, and support status. | Functional | P6 | Should | Dependency tracking; upgrade governance |

---

### 4.4 CRM Integration & Data Migration

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-CRM-015 | CRM Integration Map | The suite SHALL render a CRM Integration Map showing all systems integrated with the CRM platform, the integration direction (bidirectional/inbound/outbound), mechanism (MuleSoft/Dell Boomi/custom API/ETL), data objects exchanged, and frequency (real-time/batch). | Functional | P6, P4 | Must | Cross-role integration; impact analysis |
| REQ-ROL-CRM-016 | Data Migration Plan Template | The suite SHALL auto-generate a Data Migration Plan template covering: data scope (objects/fields), source system, extraction method, transformation rules, load order (parent-before-child), tool (Dataloader, DemandTools, Jitterbit, MuleSoft), delta strategy, cutover approach, rollback plan, and success criteria. | Functional | P6 | Must | Critical CRM programme artefact; frequently mishandled |
| REQ-ROL-CRM-017 | Data Migration Mapping Workbook | The suite SHALL generate a Data Migration Mapping Workbook (Excel) from source field → transformation expression → target CRM field mappings defined in the CDM mapping tool. | Data | P6, P4 | Should | V10 deliverable; migration team handoff artefact |

---

### 4.5 CRM Security Model

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-CRM-018 | Security Model Matrix | The suite SHALL support authoring a CRM Security Model Matrix covering: object-level access (CRUD per profile/permission set), field-level security (read/edit per profile/permission set), record-level sharing (OWD, sharing rules, manual shares, teams, territories), and page-layout assignments. | Security | P6, P8 | Must | Salesforce Security Model; Dynamics Security Roles; data protection |
| REQ-ROL-CRM-019 | Principle-of-Least-Privilege Audit | The suite SHALL perform a principle-of-least-privilege audit on the CRM security model, flagging profiles/permission sets that grant unnecessary Create/Delete access, Modify All Data, View All Data, or access to sensitive fields (SSN, credit card, health data). | Security | P6, P8 | Must | SOX, HIPAA, GDPR; security compliance |
| REQ-ROL-CRM-020 | Shield / Data Encryption Design | The suite SHALL support documenting Salesforce Shield Platform Encryption or Dynamics field-level encryption design: which fields are encrypted, encryption scheme (AES-256), key management approach, and impact on search/formula fields. | Security | P6, P8 | Should | Salesforce Shield; data-at-rest protection |

---

## 5. Data Architect (P7)

**Role summary:** Designs the enterprise data landscape — conceptual/logical/physical data models, data warehouse/lake/lakehouse/mesh architecture, MDM, data lineage, data catalog, governance, and quality — aligned to DAMA-DMBOK knowledge areas.

**Key activities:** Conceptual data modeling (CDM), logical data modeling (LDM), physical data modeling (PDM, DDL), data warehouse/star-schema/data vault design, data lake/lakehouse architecture (medallion/lambda/kappa), data mesh domain ownership design, MDM hub/spoke design, data lineage tracking, data catalog governance, data quality rules, DAMA-DMBOK alignment, data classification and sensitivity labelling.

**Signature artifacts:** Conceptual/Logical/Physical data models (ERD), Data Vault 2.0 or star-schema diagrams, Data Flow Diagrams (DFD), Data Architecture document, Data Governance framework, MDM design, Data Catalog, Data Lineage diagrams, Data Quality rules register, Data Classification matrix.

**Diagram types needed (→ V03):** ERD (conceptual/logical/physical), Data Flow Diagram, Data Vault diagram, Star Schema / Snowflake diagram, Data Mesh topology, Data Lineage DAG.

**Deliverable templates needed (→ V10):** Data Architecture Document, Data Governance Framework, MDM Design, Data Dictionary, Data Quality Rules Register.

---

### 5.1 Conceptual, Logical & Physical Data Modeling

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-DATA-001 | Conceptual Data Model Editor | The suite SHALL provide a Conceptual Data Model (CDM) editor supporting entity definition (name, description, business synonym), relationship types (one-to-one, one-to-many, many-to-many), and business rules, rendered as a clean ERD without implementation detail. | Functional | P7 | Must | DAMA-DMBOK Chapter 5 (Data Modeling); business communication |
| REQ-ROL-DATA-002 | Logical Data Model Editor | The suite SHALL provide a Logical Data Model (LDM) editor extending CDM with attribute definitions (name, data type, optionality, business key, candidate keys), normalisation to 3NF/BCNF option, and relationship resolution (junction tables for M:M). | Functional | P7 | Must | DAMA-DMBOK; normalisation standards |
| REQ-ROL-DATA-003 | Physical Data Model Editor | The suite SHALL provide a Physical Data Model (PDM) editor adding platform-specific details (column type/length/precision, null/not-null, default, index type, partitioning, tablespace) and generate DDL (CREATE TABLE/INDEX scripts) for target databases (PostgreSQL, SQL Server, Oracle, MySQL, Snowflake, BigQuery, Redshift). | Functional | P7 | Must | Database design practice; DBA handoff artefact |
| REQ-ROL-DATA-004 | CDM→LDM→PDM Propagation | The suite SHALL propagate entity/attribute changes from CDM to LDM and LDM to PDM with conflict-detection (e.g., attribute removed from LDM but columns exist in PDM) and require explicit reconciliation. | Governance | P7 | Must | Model consistency; change management |
| REQ-ROL-DATA-005 | AI Entity & Attribute Suggestion | The suite SHALL invoke the AI engine to suggest entities, attributes, and relationships for a given business domain (e.g., "automotive parts ordering") from business requirements extracted from the knowledge base. | AI/ML | P7 | Should | V06 AI engine; model cold-start |
| REQ-ROL-DATA-006 | Reverse-Engineer Data Model from DDL | The suite SHALL accept SQL DDL scripts or connect to live database metadata to reverse-engineer a Physical Data Model and optionally produce a draft Logical Data Model by inferring entity groupings. | Functional | P7 | Must | Legacy documentation; code-scan integration |
| REQ-ROL-DATA-007 | Data Dictionary Auto-Generation | The suite SHALL auto-generate a Data Dictionary (Word/Excel) from the LDM or PDM listing each table/entity, column/attribute, data type, nullable, PK/FK flags, description, and data classification. | Reporting | P7 | Must | V10 deliverable engine; governance artefact |

---

### 5.2 Data Warehouse / Lake / Lakehouse / Mesh

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-DATA-008 | Star Schema Designer | The suite SHALL provide a Star Schema designer for fact tables (measures, grain statement, surrogate key) and dimension tables (SCD Type 0/1/2/3 designation, natural key, surrogate key, attributes), rendering the classic star/snowflake pattern. | Functional | P7 | Must | Kimball DW/BI method; OLAP design |
| REQ-ROL-DATA-009 | Data Vault 2.0 Designer | The suite SHALL provide a Data Vault 2.0 designer supporting Hubs (business key, load-date, record-source), Links (relationship between hubs, composite keys), Satellites (descriptive data, hash diff, sequence key, CDC columns), and same-as links. | Functional | P7 | Should | Dan Linstedt Data Vault 2.0; enterprise DW scalability |
| REQ-ROL-DATA-010 | Medallion Architecture Design | The suite SHALL support designing a Medallion (Bronze/Silver/Gold) lakehouse architecture diagram showing ingestion, cleansing, and serving layers, with technology annotations (Databricks/Delta Lake, Azure Synapse, Apache Iceberg, AWS Glue). | Functional | P7 | Must | Delta Lake / Databricks Medallion; widely adopted lakehouse pattern |
| REQ-ROL-DATA-011 | Lambda vs Kappa Architecture Decision | The suite SHALL provide a structured Lambda vs Kappa architecture decision aid (trade-offs: batch vs streaming latency, complexity, tooling) with ADR capture. | Functional | P7 | Should | Big data architecture pattern |
| REQ-ROL-DATA-012 | Data Mesh Domain Design | The suite SHALL support modelling a data mesh architecture showing data domains (bounded contexts), data products per domain (with schema, SLA, owner team), data product marketplace, and federated governance model. | Functional | P7 | Should | Zhamak Dehghani Data Mesh; decentralised data ownership |
| REQ-ROL-DATA-013 | Data Flow Diagram (DFD) | The suite SHALL support Level-0 and Level-1 DFDs showing external entities, processes, data stores, and data flows with Yourdon/DeMarco notation. | Functional | P7, P3 | Should | Classic DFD notation; system-to-system data movement |

---

### 5.3 MDM, Data Lineage & Catalog

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-DATA-014 | MDM Design Canvas | The suite SHALL provide an MDM Design Canvas where the Data Architect can define master entities (Party, Product, Location, Account, Asset), golden record design, survivorship rules (source precedence, most-recent, most-complete), matching/dedup strategy, and hub topology (registry/central/coexistence/consolidation). | Functional | P7 | Must | DAMA MDM; Gartner MDM Reference; prevents data silos |
| REQ-ROL-DATA-015 | Data Lineage Diagram | The suite SHALL render a Data Lineage Diagram (DAG) showing each data asset's upstream sources, transformations (ETL/ELT steps with logic), and downstream consumers, traceable to table/column level. | Functional | P7 | Must | DAMA Data Lineage; GDPR impact analysis; audit |
| REQ-ROL-DATA-016 | Data Catalog Integration | The suite SHALL integrate with enterprise data catalog tools (Apache Atlas, Collibra, Alation, Microsoft Purview, AWS Glue Data Catalog) to import/export metadata and maintain bidirectional synchronisation of the data model. | Integration | P7 | Should | DAMA Data Governance; catalog-as-system-of-record |
| REQ-ROL-DATA-017 | Column-Level Lineage | The suite SHALL support column-level lineage tracing (showing which source column feeds which target column through which transformation) for critical data elements (CDEs). | Functional | P7 | Should | Regulatory compliance; BCBS 239; GDPR Right-to-Erasure |

---

### 5.4 Data Governance & Quality

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-DATA-018 | DAMA-DMBOK Alignment Matrix | The suite SHALL provide a DAMA-DMBOK 11-knowledge-area alignment matrix (Data Governance, Data Architecture, Data Modeling, Data Storage & Ops, Data Security, Data Integration, Document & Content, Reference & MDM, Data Warehousing, Metadata Management, Data Quality) showing which ESIP capabilities address each area. | Governance | P7 | Should | DAMA-DMBOK v2; customer credibility for data practices |
| REQ-ROL-DATA-019 | Data Classification Matrix | The suite SHALL support classifying each data entity/attribute by sensitivity tier (Public / Internal / Confidential / Restricted / Regulated) and regulatory category (PII, PHI, PCI-DSS, ITAR, GDPR, CCPA) with owner assignment. | Security | P7, P8 | Must | DAMA Data Security; GDPR Art.30 records of processing |
| REQ-ROL-DATA-020 | Data Quality Rules Register | The suite SHALL maintain a Data Quality Rules Register per dataset/entity: dimension (completeness, accuracy, consistency, timeliness, uniqueness, validity), rule expression, measurement frequency, threshold, owner, and current score if available. | Governance | P7 | Must | DAMA Data Quality; ISO 8000 |
| REQ-ROL-DATA-021 | Reference Data Management | The suite SHALL support managing reference data domains (country codes, currency codes, product categories, status codes, UOM) with version history, governance owner, and linkage to the entities that consume each code set. | Data | P7 | Should | DAMA Reference Data; consistency across systems |
| REQ-ROL-DATA-022 | Data Retention & Purge Policy Design | The suite SHALL support documenting a Data Retention & Purge Policy per entity: retention period, legal hold triggers, purge method (hard-delete/anonymise/pseudonymise), and regulatory basis (GDPR Art.17, CCPA §1798.105, HIPAA). | Governance | P7, P8 | Must | GDPR / CCPA / HIPAA; right-to-erasure compliance |

---

## 6. Security Architect (P8)

**Role summary:** Designs the enterprise security architecture using SABSA, NIST frameworks, and Zero-Trust principles; performs threat modelling; defines control catalogs; governs IAM; maps compliance requirements; and ensures security is a first-class concern across all other architecture roles.

**Key activities:** Security strategy (SABSA business attributes → security architecture), threat modelling (STRIDE, PASTA, LINDDUN), control catalog definition (NIST SP 800-53, ISO 27001 Annex A, CIS Controls v8), zero-trust architecture design, IAM architecture (identity providers, MFA, PAM, RBAC/ABAC), data protection design (encryption at rest/in transit, DLP, tokenisation), compliance mapping (NIST CSF, ISO 27001, SOC 2, PCI-DSS, HIPAA, GDPR), security pattern library, security architecture review, penetration test findings triage.

**Signature artifacts:** Security architecture document, SABSA attribute profiles, threat model (STRIDE/PASTA), control catalog, IAM design, zero-trust design, compliance mapping matrix, security pattern library, security risk register.

**Diagram types needed (→ V03):** Trust zone / network security diagram, IAM flow diagram, Zero-Trust topology, Data flow diagram (for GDPR/threat modelling), Sequence (auth flows).

**Deliverable templates needed (→ V10):** Security Architecture Document, Threat Model Report, Control Catalog (Excel), Compliance Mapping Matrix, IAM Design Document.

---

### 6.1 SABSA & Security Strategy

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-SEC-001 | SABSA Business Attribute Profiling | The suite SHALL support authoring a SABSA Business Attribute Profile defining security-driving business attributes (Confidentiality, Integrity, Availability, Authenticity, Authorised-Use, Non-repudiation, Privacy, Resilience) from business goals, mapped to security services. | Functional | P8 | Must | SABSA Institute; business-driven security architecture |
| REQ-ROL-SEC-002 | SABSA Six-Layer Architecture | The suite SHALL support modelling security architecture at all six SABSA layers (Contextual, Conceptual, Logical, Physical, Component, Operational) with artefact templates at each layer. | Functional | P8 | Should | SABSA Institute; comprehensive coverage |
| REQ-ROL-SEC-003 | Security Strategy Document | The suite SHALL auto-generate a Security Strategy Document from SABSA attribute profiles, selected frameworks, zero-trust posture decision, and top risks. | Functional | P8 | Should | V10 deliverable engine; executive communication |

---

### 6.2 Threat Modeling

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-SEC-004 | STRIDE Threat Model | The suite SHALL support authoring STRIDE threat models (Spoofing, Tampering, Repudiation, Information Disclosure, Denial of Service, Elevation of Privilege) against a Data Flow Diagram, generating a structured threat list with affected component, threat agent, impact, likelihood, and mitigation. | Security | P8, P5 | Must | Microsoft STRIDE; OWASP Threat Dragon; SDL practice |
| REQ-ROL-SEC-005 | PASTA Threat Model | The suite SHALL support the PASTA (Process for Attack Simulation and Threat Analysis) seven-stage methodology: objectives, technical scope, application decomposition, threat analysis, vulnerability analysis, attack modelling, risk/impact analysis. | Security | P8 | Should | PASTA framework; risk-centric threat modelling |
| REQ-ROL-SEC-006 | LINDDUN Privacy Threat Analysis | The suite SHALL support LINDDUN privacy threat analysis (Linkability, Identifiability, Non-repudiation, Detectability, Disclosure of information, Unawareness, Non-compliance) against DFDs for GDPR/privacy-by-design compliance. | Security | P8 | Should | LINDDUN; GDPR Art.25 Privacy by Design |
| REQ-ROL-SEC-007 | Threat Model DFD Editor | The suite SHALL provide a Data Flow Diagram editor with trust zone boundaries, processes, external entities, and data stores following Microsoft Threat Modelling Tool / OWASP Threat Dragon notation, usable as input to STRIDE/PASTA/LINDDUN analyses. | Functional | P8 | Must | Standard threat modelling tooling |
| REQ-ROL-SEC-008 | AI Threat Generation | The suite SHALL invoke the AI engine to automatically generate a candidate STRIDE threat list from a system DFD or architecture description, which the Security Architect can review, triage, and supplement. | AI/ML | P8 | Must | V06 AI engine; dramatically reduces manual effort |
| REQ-ROL-SEC-009 | Threat-to-Control Traceability | The suite SHALL link each threat to one or more mitigating controls in the control catalog, and flag threats with no mitigating control as open risks requiring a risk acceptance or remediation decision. | Governance | P8 | Must | Risk closure assurance |

---

### 6.3 Control Catalog

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-SEC-010 | NIST SP 800-53 Control Catalog | The suite SHALL include the NIST SP 800-53 Rev 5 control catalog (20 control families, ~1000 controls) as a built-in reference, searchable by control ID, family, and keyword. | Security | P8 | Must | NIST SP 800-53 Rev 5; US federal and broadly adopted |
| REQ-ROL-SEC-011 | ISO 27001:2022 Annex A Controls | The suite SHALL include the ISO/IEC 27001:2022 Annex A control set (93 controls, 4 themes: Organisational/People/Physical/Technological) as a built-in reference. | Security | P8 | Must | ISO/IEC 27001:2022 |
| REQ-ROL-SEC-012 | CIS Controls v8 | The suite SHALL include CIS Controls v8 (18 controls, 153 safeguards, IG1/IG2/IG3 implementation groups) as a built-in reference. | Security | P8 | Must | CIS Controls v8; practical implementation roadmap |
| REQ-ROL-SEC-013 | Custom Control Catalog | The suite SHALL allow the Security Architect to define organisation-specific controls, mapped to one or more standard control frameworks, forming the enterprise control catalog. | Security | P8 | Must | Enterprise customisation |
| REQ-ROL-SEC-014 | Control Implementation Status Tracking | The suite SHALL track implementation status (Not Started / In Progress / Implemented / Partially Implemented / Not Applicable) per control per project/system, with evidence links and responsible team. | Governance | P8 | Must | Audit readiness; compliance assurance |
| REQ-ROL-SEC-015 | Compliance Framework Mapping Matrix | The suite SHALL generate a Compliance Mapping Matrix showing coverage of NIST CSF 2.0, ISO 27001, SOC 2 (CC series), PCI-DSS v4, HIPAA Security Rule, GDPR Art.32, and CIS Controls, highlighting gaps. | Governance | P8 | Must | Multi-framework compliance; audit preparation |

---

### 6.4 Zero-Trust & IAM Architecture

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-SEC-016 | Zero-Trust Architecture Design | The suite SHALL support designing a Zero-Trust Architecture (ZTA) based on NIST SP 800-207 principles (never trust / always verify, micro-segmentation, least privilege, continuous validation), with a topology diagram showing policy engine, policy administrator, policy enforcement points, and trust algorithm. | Security | P8 | Must | NIST SP 800-207; CISA ZTA strategic plan |
| REQ-ROL-SEC-017 | Network Micro-Segmentation Design | The suite SHALL support designing network micro-segmentation zones (trust zones, DMZ, management zone, data zone, user zone) with allowed/denied traffic flows between zones rendered as a zone-based security diagram. | Security | P8 | Must | Zero-Trust; NIST SP 800-125B |
| REQ-ROL-SEC-018 | IAM Architecture Design | The suite SHALL support designing an IAM architecture covering: Identity Providers (Entra ID / Okta / Ping / Auth0 / Cognito), MFA/passwordless, Federation (SAML 2.0, OIDC, WS-Federation), PAM (CyberArk/BeyondTrust), RBAC/ABAC policy design, and Service Account governance. | Security | P8 | Must | IAM core security control; cloud-native essential |
| REQ-ROL-SEC-019 | PAM Design | The suite SHALL support designing Privileged Access Management (PAM) covering: vault architecture, just-in-time (JIT) access workflows, session recording, credential rotation, and break-glass procedures. | Security | P8 | Should | PAM as key Zero-Trust control; CyberArk/BeyondTrust reference |
| REQ-ROL-SEC-020 | Secrets Management Design | The suite SHALL support documenting secrets management design: vault platform (HashiCorp Vault, AWS Secrets Manager, Azure Key Vault), secret types (DB credentials, API keys, certificates, SSH keys), rotation policy, access policy, and audit trail requirements. | Security | P8 | Must | NIST SP 800-57; production secrets hygiene |

---

### 6.5 Data Protection & Security Patterns

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-SEC-021 | Encryption Design | The suite SHALL support authoring an Encryption Design document specifying: algorithms (AES-256-GCM, RSA-2048/4096, ECDSA P-256, ChaCha20-Poly1305), key hierarchy (KEK/DEK), KMS platform, encryption scope (data at rest, in transit [TLS 1.2+], in processing), and HSM requirements. | Security | P8 | Must | NIST SP 800-175B; FIPS 140-3 |
| REQ-ROL-SEC-022 | DLP Pattern Design | The suite SHALL support documenting a Data Loss Prevention (DLP) design: classification tiers triggering DLP, inspection modes (endpoint/network/cloud), response actions (block/quarantine/alert), and integration with M365 Purview / Symantec DLP / Forcepoint. | Security | P8 | Should | Data protection; GDPR Art.32 |
| REQ-ROL-SEC-023 | Security Pattern Library | The suite SHALL include a Security Pattern Library covering: Authentication patterns (API Key, OAuth/OIDC, Client Certificate, SPIFFE/SPIRE), Authorisation patterns (RBAC, ABAC, PBAC, ReBAC), Secure Communication patterns (TLS termination, mutual TLS, service mesh), and Data patterns (field-level encryption, tokenisation, pseudonymisation). | Security | P8 | Must | Security patterns reference; design consistency |
| REQ-ROL-SEC-024 | Security Architecture Review Checklist | The suite SHALL auto-generate a Security Architecture Review (SAR) checklist from the system's architecture description, covering authentication, authorisation, data protection, network security, logging/monitoring, supply chain, and secret management dimensions. | Security | P8 | Must | SAR process; V10 deliverable |

---

## 7. Cloud Architect (P9)

**Role summary:** Designs cloud architecture aligned to AWS/Azure/GCP Well-Architected frameworks; defines landing zones; selects cloud service topologies; governs IaC; optimises cost (FinOps); designs resilience/DR; and manages multi-cloud or hybrid-cloud strategies.

**Key activities:** Well-Architected Framework (WAF) reviews (all pillars), landing zone design (account/subscription structure, VPC/VNET topology, guardrails, identity federation), cloud service topology design (compute, storage, networking, messaging, data, ML services), IaC standards (Terraform/Bicep/CloudFormation/Pulumi), FinOps cost optimisation, resilience and DR design (RPO/RTO, multi-AZ, multi-region, chaos engineering), multi-cloud and hybrid-cloud design.

**Signature artifacts:** WAF review report, landing zone design, cloud topology diagram, IaC reference architecture, FinOps cost model, DR design, cloud migration plan.

**Diagram types needed (→ V03):** Cloud topology diagram (provider-specific icons), network diagram (VPC/VNET/subnet/peering), HA/DR diagram, landing zone diagram, hybrid-connectivity diagram.

**Deliverable templates needed (→ V10):** WAF Review Report, Landing Zone Design, Cloud Architecture Document, FinOps Cost Model, DR Design Document.

---

### 7.1 Well-Architected Framework Reviews

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-CLD-001 | AWS Well-Architected Review | The suite SHALL provide a structured AWS Well-Architected Framework review covering all six pillars (Operational Excellence, Security, Reliability, Performance Efficiency, Cost Optimisation, Sustainability) with question sets aligned to the AWS WA Tool question library and risk rating (High/Medium/Low). | Functional | P9 | Must | AWS Well-Architected Framework; official question library |
| REQ-ROL-CLD-002 | Azure Well-Architected Review | The suite SHALL provide a structured Azure Well-Architected Framework review covering all five pillars (Reliability, Security, Cost Optimisation, Operational Excellence, Performance Efficiency) aligned to the Azure Advisor recommendation categories. | Functional | P9 | Must | Microsoft Azure Well-Architected Framework |
| REQ-ROL-CLD-003 | GCP Architecture Framework Review | The suite SHALL provide a structured Google Cloud Architecture Framework review covering all six areas (Operational Excellence, Security/Privacy/Compliance, Reliability, Cost Optimisation, Performance, Infrastructure). | Functional | P9 | Should | Google Cloud Architecture Framework |
| REQ-ROL-CLD-004 | WAF Finding Risk Register | The suite SHALL aggregate WAF review findings into a risk register with pillar, question, risk level, recommendation, owner, and remediation status, and auto-generate a WAF Review Report. | Reporting | P9 | Must | V10 deliverable; governance |
| REQ-ROL-CLD-005 | AI WAF Gap Analysis | The suite SHALL invoke the AI engine to analyse a cloud architecture description (IaC code, architecture diagram, system context) and generate a candidate WAF gap list for each pillar, which the Cloud Architect reviews and validates. | AI/ML | P9 | Should | V06 AI engine; automated pre-screening |

---

### 7.2 Landing Zones

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-CLD-006 | Landing Zone Design | The suite SHALL support authoring a Landing Zone Design covering: account/subscription vending model, OU/Management Group hierarchy, identity federation (Entra ID / AWS SSO/IAM Identity Center / Google Workspace), network topology (hub-spoke/mesh), DNS strategy, logging/monitoring baseline, and policy/guardrail set. | Functional | P9 | Must | AWS Control Tower, Azure Landing Zones (ALZ), GCP Landing Zone |
| REQ-ROL-CLD-007 | Hub-and-Spoke vs Mesh Topology Decision | The suite SHALL provide a structured Hub-and-Spoke vs Mesh topology decision aid with trade-off criteria (blast radius, cost, latency, complexity) and ADR capture. | Functional | P9 | Should | Cloud networking pattern decision |
| REQ-ROL-CLD-008 | VPC/VNET Design | The suite SHALL support designing VPC (AWS), VNET (Azure), or VPC Network (GCP) topology including CIDR allocation, subnet segmentation (public/private/data/management tiers), peering/transit-gateway/hub, security group/NSG rules matrix, and NAT/IGW placement. | Functional | P9 | Must | Core cloud networking; overlaps with P10 |
| REQ-ROL-CLD-009 | Guardrail Catalog | The suite SHALL include a Guardrail Catalog for AWS SCPs, Azure Policy, and GCP Organization Policies covering preventive, detective, and responsive categories, aligned to CIS Benchmarks and cloud-provider security baselines. | Security | P9, P8 | Must | CIS Cloud Benchmarks; landing zone compliance |

---

### 7.3 IaC Standards & FinOps

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-CLD-010 | IaC Reference Architecture | The suite SHALL support documenting an IaC Reference Architecture specifying: tooling (Terraform / OpenTofu / Bicep / Pulumi / CloudFormation), module structure, state management (S3+DynamoDB / Azure Blob / GCS), secrets injection strategy, CI/CD pipeline integration, and drift detection approach. | Functional | P9 | Must | IaC best practices; HashiCorp Terraform; CNCF |
| REQ-ROL-CLD-011 | IaC Code Review for Architecture Compliance | The suite SHALL scan IaC (Terraform .tf files, Bicep, CloudFormation) and flag deviations from the documented architecture (e.g., open security groups, unencrypted S3 buckets, missing tags) using tfsec/checkov-style rules. | Governance | P9, P8 | Should | tfsec, checkov, Snyk IaC; architecture-as-code quality |
| REQ-ROL-CLD-012 | FinOps Cost Model | The suite SHALL support authoring a FinOps Cost Model documenting: cloud cost allocation tags/showback strategy, reserved instance/savings plan coverage targets, rightsizing recommendations, idle resource policies, unit-economics metric (cost per transaction/user/GB), and FinOps maturity level (Crawl/Walk/Run). | Functional | P9 | Must | FinOps Foundation; Well-Architected Cost Optimisation pillar |
| REQ-ROL-CLD-013 | Cost Forecast Dashboard | The suite SHALL provide a Cost Forecast Dashboard showing monthly projected spend per workload/account/tag, coverage % of reserved capacity, and top five cost-optimisation recommendations by savings potential. | Reporting | P9 | Should | FinOps Foundation; cost governance |

---

### 7.4 Resilience, DR & Multi-Cloud

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-CLD-014 | Resilience Design | The suite SHALL support authoring a Resilience Design per workload documenting: availability target (e.g., 99.99%), RPO/RTO targets, failure mode analysis (AZ failure, region failure, dependency failure), multi-AZ/multi-region deployment pattern, auto-scaling policy, circuit breaker/retry config, and chaos engineering test scenarios. | Functional | P9 | Must | AWS Reliability Pillar; Azure Reliability; Google SRE |
| REQ-ROL-CLD-015 | DR Strategy Matrix | The suite SHALL support authoring a DR Strategy Matrix per workload classifying the DR strategy as Backup-and-Restore, Pilot Light, Warm Standby, or Multi-Site Active-Active, with cost and RTO/RPO trade-off rationale. | Functional | P9 | Must | AWS DR Strategies; Azure BCDR; workload criticality tiers |
| REQ-ROL-CLD-016 | Multi-Cloud / Hybrid-Cloud Topology | The suite SHALL support designing multi-cloud and hybrid-cloud topologies showing on-premise data centres, cloud regions, interconnect (ExpressRoute / AWS Direct Connect / Cloud Interconnect), data sovereignty zones, and workload placement rationale. | Functional | P9 | Should | Enterprise multi-cloud reality; hybrid integration |
| REQ-ROL-CLD-017 | Cloud-Native Service Map | The suite SHALL maintain a Cloud-Native Service Map per project listing every cloud service used (by provider + service name + tier/SKU), its purpose, availability SLA, and associated WAF pillar risks. | Data | P9 | Should | Inventory control; FinOps and resilience analysis |

---

## 8. Infrastructure / Network Architect (P10)

**Role summary:** Designs physical and virtual infrastructure topology, network architecture, capacity planning, high-availability and disaster-recovery designs, and ITIL-aligned service design, covering data-centre and edge environments.

**Key activities:** Network topology design (core/distribution/access, campus, WAN, SD-WAN), data-centre architecture (cabling, power/cooling, rack design, fabric), server/virtualisation design (VMware vSphere, Hyper-V, KVM), storage architecture (SAN/NAS/object), capacity planning, HA/DR design, ITIL service design (availability management, capacity management, IT service continuity), edge/CDN design, DNS/DHCP/IPAM design.

**Signature artifacts:** Network topology diagrams (L1/L2/L3), data-centre floor plan, rack diagrams, server/storage inventory, IPAM plan, capacity plan, HA/DR design document, ITIL service design package.

**Diagram types needed (→ V03):** L1/L2/L3 network diagram, data-centre topology, rack diagram, IPAM/subnetting diagram, HA/DR topology.

**Deliverable templates needed (→ V10):** Network Design Document, Capacity Plan, HA/DR Design, ITIL Service Design Package.

---

### 8.1 Network Topology Design

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-INF-001 | L1/L2/L3 Network Diagram | The suite SHALL support authoring Layer 1 (physical cabling/port), Layer 2 (VLAN/spanning tree/trunk), and Layer 3 (IP routing/BGP/OSPF/EIGRP) network diagrams with Cisco-style iconography and selectable notation sets (Cisco, AWS, Azure, generic). | Functional | P10 | Must | Standard network documentation |
| REQ-ROL-INF-002 | IPAM / Subnet Design | The suite SHALL support designing an IPAM plan with CIDR block allocation tree (supernets, subnets, host ranges), VLAN-to-subnet mapping, reserved ranges, and DHCP scope definitions. | Functional | P10 | Must | IP address management; routing prerequisite |
| REQ-ROL-INF-003 | WAN / SD-WAN Design | The suite SHALL support WAN and SD-WAN topology design (sites, links, MPLS/Internet/LTE overlay, underlay protocols, SD-WAN policy: QoS, application steering, failover). | Functional | P10 | Should | Enterprise WAN evolution; SD-WAN adoption |
| REQ-ROL-INF-004 | DNS / DHCP Architecture | The suite SHALL support documenting DNS architecture (internal zones, external zones, split-horizon, forwarders, DNSSEC policy) and DHCP architecture (scopes, failover pairs, option sets, superscopes). | Functional | P10 | Should | Core network services |
| REQ-ROL-INF-005 | Firewall Rule Matrix | The suite SHALL support authoring a Firewall Rule Matrix (source zone, destination zone, protocol, port, action, justification) with deduplication detection and unused-rule identification. | Security | P10, P8 | Must | Network security; audit readiness |

---

### 8.2 Data Centre & Infrastructure

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-INF-006 | Data Centre Topology Diagram | The suite SHALL support authoring a data-centre topology diagram showing compute, storage, network fabric (spine-leaf / three-tier), OOB management, power distribution, and cooling zones. | Functional | P10 | Must | Data-centre design; colocation/migration planning |
| REQ-ROL-INF-007 | Rack Diagram | The suite SHALL support authoring rack diagrams (42U/48U standard) showing equipment placement (servers, switches, patch panels, PDUs, KVM), power draw per rack, and cable management notation. | Functional | P10 | Should | Physical infrastructure documentation |
| REQ-ROL-INF-008 | Virtualisation Platform Design | The suite SHALL support designing virtualisation platform topology (VMware vSphere cluster, Hyper-V cluster, KVM) showing host nodes, vCPU/RAM allocations, vSAN/shared storage, vSwitch/DVS network, HA admission control policy, and DRS rules. | Functional | P10 | Should | Virtualisation is dominant on-prem pattern |
| REQ-ROL-INF-009 | Storage Architecture Design | The suite SHALL support designing storage architecture (SAN fabric topology, NAS filer, S3-compatible object storage) with LUN/volume allocation, multipath design, replication topology, and performance tier (NVMe/SSD/HDD) assignment. | Functional | P10 | Should | Storage design; performance-sensitive workloads |

---

### 8.3 Capacity Planning, HA & ITIL

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-INF-010 | Capacity Plan | The suite SHALL support authoring a Capacity Plan capturing: current utilisation baseline (CPU, RAM, storage, network bandwidth per component), growth projections (linear/seasonal/step), headroom targets, and capacity expansion triggers and options. | Functional | P10 | Must | ITIL Capacity Management; prevents brownout failures |
| REQ-ROL-INF-011 | HA Design Document | The suite SHALL support authoring an HA Design Document per service: availability target, single-points-of-failure (SPOF) analysis, redundancy approach (N+1/2N/2N+1), failover mechanism (automatic/manual), failover time objective, and test schedule. | Functional | P10 | Must | ITIL Availability Management; business continuity |
| REQ-ROL-INF-012 | ITIL Service Design Package | The suite SHALL auto-generate an ITIL Service Design Package for a new or changed infrastructure service covering: service description, SLA, OLA, UC, capacity plan, availability plan, IT service continuity plan, information security plan, and service acceptance criteria. | Functional | P10 | Should | ITIL 4 Service Design; formal transition artefact |
| REQ-ROL-INF-013 | Edge / CDN Architecture | The suite SHALL support designing edge and CDN architecture (PoP locations, origin-shield topology, cache-key strategy, WAF placement at edge, Anycast routing, BGP anycast prefix) for geographically distributed workloads. | Functional | P10, P9 | Should | CDN architecture; latency-sensitive global apps |

---

## 9. Microservices / Platform / DevOps Architect (P11)

**Role summary:** Designs microservice-based systems using DDD/bounded-context decomposition; defines the internal developer platform, golden paths, and paved roads; architects CI/CD pipelines; establishes observability standards; and owns SRE/SLO practices.

**Key activities:** DDD event storming and bounded context mapping, service decomposition (choreography/orchestration, API contracts, service mesh), platform engineering (IDP design, golden paths, scaffolding, portal), CI/CD pipeline design (GitOps, ArgoCD, Tekton, GitHub Actions, Azure DevOps), observability design (metrics/logs/traces — OTel, Prometheus, Grafana, Jaeger), SRE practices (SLI/SLO/SLA/error budget, toil reduction), 12-factor + 15-factor app design.

**Signature artifacts:** DDD context map, service decomposition diagram, service mesh topology, platform engineering blueprint, CI/CD pipeline diagram, observability architecture, SLO catalog, runbook templates.

**Diagram types needed (→ V03):** DDD context map, C4 (Container/Component), service mesh topology, CI/CD pipeline diagram, observability topology, sequence (inter-service calls).

**Deliverable templates needed (→ V10):** Microservices Architecture Document, Platform Engineering Blueprint, SLO Catalog, CI/CD Design Document, Observability Design Document.

---

### 9.1 DDD & Bounded Context Design

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-MS-001 | Event Storming Board | The suite SHALL provide a virtual Event Storming board supporting Domain Events (orange), Commands (blue), Aggregates (yellow), Policies (purple), External Systems (pink), Read Models (green), Users/Actors (yellow sticky), and Hotspot markers with spatial grouping into bounded contexts. | Functional | P11 | Must | Alberto Brandolini Event Storming; DDD community standard |
| REQ-ROL-MS-002 | Bounded Context Map | The suite SHALL support authoring a DDD Bounded Context Map with named contexts and context relationship patterns: Shared Kernel, Customer-Supplier, Conformist, Anticorruption Layer (ACL), Open Host Service (OHS), Published Language (PL), Partnership, and Big Ball of Mud. | Functional | P11 | Must | Eric Evans DDD; Vaughn Vernon IDDD |
| REQ-ROL-MS-003 | Aggregate Design Canvas | The suite SHALL provide an Aggregate Design Canvas per bounded context listing: aggregate roots, entities, value objects, domain events emitted, commands handled, invariants enforced, and concurrency strategy (optimistic/pessimistic). | Functional | P11 | Should | DDD blue book; aggregate correctness |
| REQ-ROL-MS-004 | Ubiquitous Language Glossary | The suite SHALL maintain a per-bounded-context Ubiquitous Language Glossary where domain terms are defined, distinguishing meanings when the same word has different semantics in different contexts. | Data | P11 | Must | DDD foundational concept; communication clarity |
| REQ-ROL-MS-005 | Service Decomposition Validation | The suite SHALL validate that proposed services respect bounded-context boundaries (no cross-context data coupling, no direct DB sharing) and flag violations detected from context map and component definitions. | Governance | P11 | Should | Microservices decomposition principle; Fowler |

---

### 9.2 Service Mesh & Communication

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-MS-006 | Service Mesh Topology Design | The suite SHALL support designing a service mesh topology (Istio, Linkerd, Consul Connect, AWS App Mesh) showing data-plane sidecar/ambient injection, control-plane components, traffic management (canary, blue/green, weighted routing), mTLS mesh policy, and observability tap points. | Functional | P11 | Must | CNCF service mesh landscape; Istio/Linkerd documentation |
| REQ-ROL-MS-007 | Inter-Service Communication Pattern | The suite SHALL maintain a per-service communication pattern declaration: sync (REST/gRPC, circuit-breaker config, timeout, retry) or async (topic, consumer group, ordering guarantee, idempotency key), with rationale for the choice. | Functional | P11 | Must | Microservices patterns (Richardson); communication contract |
| REQ-ROL-MS-008 | API Gateway vs. Service Mesh Decision | The suite SHALL provide a structured decision aid (North-South vs East-West traffic, L7 routing needs, developer portal need, service-to-service auth) for API gateway vs service mesh vs both, with ADR capture. | Functional | P11, P5 | Should | Common architectural decision |

---

### 9.3 Platform Engineering & Golden Paths

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-MS-009 | Internal Developer Platform (IDP) Blueprint | The suite SHALL support authoring an IDP Blueprint covering: self-service portal (Backstage/Port/Cortex), service scaffolding (template repository, cookiecutter/Yeoman), golden path definitions per tech stack (Java/Spring Boot, Node/Express, .NET), environment provisioning (IaC vending), and secret injection. | Functional | P11 | Must | CNCF CNOE; Gartner Platform Engineering; Team Topologies |
| REQ-ROL-MS-010 | Golden Path Template Register | The suite SHALL maintain a Golden Path Template Register listing each approved technology golden path (name, tech stack, scaffolding source, CI/CD template, included libraries, compliance controls pre-wired) with adoption metrics. | Functional | P11 | Must | Platform engineering; cognitive load reduction (Team Topologies) |
| REQ-ROL-MS-011 | Team Topologies Mapping | The suite SHALL support modelling team topologies (Stream-aligned, Platform, Enabling, Complicated-subsystem teams) per the Skelton/Pais model, with interaction modes (Collaboration, X-as-a-Service, Facilitating) between teams, and cognitive load assessment per team. | Functional | P11 | Should | Team Topologies (Skelton & Pais); org design input |

---

### 9.4 CI/CD & GitOps

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-MS-012 | CI/CD Pipeline Design | The suite SHALL support authoring CI/CD pipeline designs (GitHub Actions, GitLab CI, Azure DevOps Pipelines, Tekton, Jenkins) showing pipeline stages (checkout, build, SAST/SCA scan, unit test, container build, push, integration test, deploy, smoke test, DAST scan, promote) with environment promotion gates. | Functional | P11 | Must | CI/CD standard practice; DevSecOps pipeline |
| REQ-ROL-MS-013 | GitOps Design | The suite SHALL support designing a GitOps deployment model (ArgoCD, Flux) showing: app-of-apps structure, environment branches/repos, sync policy (automated/manual), drift detection, rollback mechanism, and secret management integration. | Functional | P11 | Should | Weaveworks GitOps principles; ArgoCD/Flux CNCF projects |
| REQ-ROL-MS-014 | Container Image Supply Chain Design | The suite SHALL support documenting container image supply chain security design: base image policy, SBOM generation (Syft/Trivy), image signing (Cosign/Notary), admission controller (Kyverno/OPA Gatekeeper), and registry policy (immutable tags, scan-on-push). | Security | P11, P8 | Must | SLSA framework; CNCF supply chain security; Executive Order 14028 |
| REQ-ROL-MS-015 | DORA Metric Baseline | The suite SHALL support capturing DORA metric baselines (Deployment Frequency, Lead Time for Changes, Change Failure Rate, Mean Time to Recovery) per team/service as design targets and track actual vs target. | Governance | P11 | Should | DORA research; DevOps performance benchmarking |

---

### 9.5 Observability & SRE

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-MS-016 | Observability Architecture Design | The suite SHALL support authoring an Observability Architecture Design covering the three pillars: Metrics (OTel Collector → Prometheus / Thanos → Grafana), Logs (Fluent Bit/Fluentd → OpenSearch/Loki/Elasticsearch → Grafana/Kibana), and Traces (OTel SDK → Tempo / Jaeger / Zipkin → Grafana/Jaeger UI), with retention policies and cardinality budgets. | Functional | P11 | Must | OpenTelemetry; CNCF observability landscape |
| REQ-ROL-MS-017 | SLI/SLO Catalog | The suite SHALL support defining SLI/SLO per service: SLI type (availability/latency-p99/error-rate/throughput), measurement window (rolling-28d/calendar-month), target (e.g., 99.9%), error budget (minutes/month), burn-rate alert thresholds (1h/6h/72h multiwindow), and exhaustion policy. | Functional | P11 | Must | Google SRE; SLO/error-budget alerting (Prometheus Recording Rules) |
| REQ-ROL-MS-018 | Toil Register | The suite SHALL support maintaining a Toil Register per service listing manual, repetitive operational tasks with estimated toil hours/week and automation roadmap items. | Governance | P11 | Should | Google SRE Chapter 5 (Eliminating Toil) |
| REQ-ROL-MS-019 | Runbook Template | The suite SHALL auto-generate a Runbook Template per service covering: service overview, on-call escalation path, key dashboards, alert catalog (alert name → symptom → diagnosis steps → remediation), known failure modes, and rollback procedure. | Functional | P11 | Should | SRE operations; incident response readiness |
| REQ-ROL-MS-020 | 12-Factor Compliance Assessment | The suite SHALL provide a 12-Factor App (extended to 15-factor) compliance assessment per service covering: codebase, dependencies, config, backing services, build/release/run, processes, port binding, concurrency, disposability, dev/prod parity, logs, admin processes, API first, telemetry, authentication. | Governance | P11 | Should | 12-factor app; Heroku; cloud-native best practices |

---

## 10. Mobile Architect (brief)

**Role summary:** Designs mobile application architecture for native (iOS/Android), cross-platform (React Native, Flutter, .NET MAUI), and PWA solutions including offline-first strategy, device capability access, push notification architecture, and app store governance.

**Key activities:** Platform choice (native vs cross-platform vs PWA), component architecture (MVVM, MVI, Clean Architecture on mobile), offline-first/sync strategy, push notification pipeline, biometric/auth integration, mobile CI/CD (Fastlane, App Center, Bitrise, EAS Build), app store submission and governance, MDM integration, performance profiling targets.

**Diagram types needed (→ V03):** Component diagram (MVVM layers), sequence (offline sync, auth flow, push flow), topology (backend-for-frontend/BFF).

---

### 10.1 Mobile Architecture Core Requirements

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-APP-022 | Mobile Platform Strategy | The suite SHALL support authoring a Mobile Platform Strategy document comparing native (Swift/Kotlin), cross-platform (React Native, Flutter, .NET MAUI), and PWA approaches against criteria (performance, platform feature access, team skills, TTM, cost) and capturing the decision as an ADR. | Functional | Mobile Architect | Should | Mobile architecture trade-off; common first decision |
| REQ-ROL-APP-023 | Offline-First Sync Design | The suite SHALL support documenting an offline-first sync design: local store (SQLite/Realm/Core Data/Room), sync protocol (CRDTs, operational transform, last-write-wins), conflict resolution strategy, sync trigger (foreground/background, on-connect), and delta-sync approach. | Functional | Mobile Architect | Should | Offline-first pattern; WatermelonDB, PouchDB, Realm Sync |
| REQ-ROL-APP-024 | Mobile CI/CD Pipeline | The suite SHALL support designing mobile CI/CD pipelines (code signing, build number increment, lane configuration for Fastlane/EAS Build/App Center, beta distribution, store submission automation) as a pipeline diagram. | Functional | Mobile Architect | Should | Mobile DevOps practice |
| REQ-ROL-APP-025 | Push Notification Architecture | The suite SHALL support designing a push notification architecture showing client registration, device token management, notification routing (APNs / FCM), delivery receipts, deep-link payload schema, and opt-in/opt-out governance. | Functional | Mobile Architect | Should | Mobile UX; APNs/FCM technical design |

---

## 11. AI/ML Architect (brief)

**Role summary:** Designs AI/ML system architecture including model selection, training/fine-tuning pipelines, serving infrastructure, MLOps, RAG/embedding pipelines, LLM integration patterns, responsible AI controls, and model governance.

**Key activities:** ML platform selection, feature store design, training pipeline design (ETL→feature→train→eval→register), model serving (batch/online inference, shadow mode, A/B testing), MLOps (MLflow, Kubeflow, SageMaker Pipelines, Vertex AI), RAG architecture (chunking, embedding, vector store, retrieval pipeline), LLM integration (prompt engineering, function calling, multi-agent), responsible AI (fairness, explainability, bias testing, guardrails), model card authoring, model governance.

**Diagram types needed (→ V03):** ML pipeline diagram, RAG topology diagram, data flow (features → training → inference), LLM agent graph.

---

### 11.1 AI/ML Architecture Core Requirements

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-APP-026 | ML Platform Design | The suite SHALL support authoring an ML Platform Design covering: data ingestion (feature pipeline), feature store (Feast/Tecton/Hopsworks), experiment tracking (MLflow/W&B/Comet), model registry, training infrastructure (distributed training, GPU cluster, Spot/Preemptible), evaluation harness, and serving platform (TF Serving/TorchServe/Triton/BentoML). | Functional | AI/ML Architect | Should | MLOps reference architecture; Databricks/Google MLOps |
| REQ-ROL-APP-027 | RAG Pipeline Architecture | The suite SHALL support designing RAG pipeline architectures: document ingestion → chunking strategy → embedding model selection → vector store (Pinecone/Weaviate/Qdrant/pgvector) → retrieval strategy (semantic/hybrid/keyword) → re-ranking → LLM generation → guardrails; with latency budget per stage. | Functional | AI/ML Architect | Should | LLM application pattern; V06 AI engine |
| REQ-ROL-APP-028 | Model Card | The suite SHALL provide a Model Card template (per Mitchell et al. 2019) capturing: model details, intended use, factors (demographic/environmental), metrics, evaluation data, training data, ethical considerations, caveats, and recommendations. | Governance | AI/ML Architect | Should | Mitchell et al. Model Cards; responsible AI |
| REQ-ROL-APP-029 | Responsible AI Controls Checklist | The suite SHALL provide a Responsible AI Controls Checklist covering: fairness (bias testing per protected attribute), explainability (SHAP/LIME/saliency), privacy (differential privacy, federated learning consideration), security (adversarial robustness, prompt injection), and regulatory compliance (EU AI Act risk tier classification). | Security | AI/ML Architect | Should | EU AI Act 2024; NIST AI RMF; responsible AI governance |

---

## 12. Role Extensibility — "Any Architect"

**Vision:** A new architect role (e.g., IoT Architect, Sustainability Architect, Quantum Architect, Domain-Specific Solution Architect) SHALL be fully operational in the ESIP suite by configuration alone — no code changes, no deployments, no engineering work — through the extensible role configuration system described in this section.

**Boundary note:** The metamodel engine that *stores and processes* these configurations lives in Volume 11. This section defines the *requirements on what that engine must support* from the role-extensibility perspective. AI prompt templates that power each role's copilot are defined in Volume 06.

---

### 12.1 Role Definition & Registration

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-EXT-001 | Role Configuration Record | The suite SHALL allow an administrator to define a new architect role via a structured configuration record containing: role name, short description, persona ID, icon, colour, associated frameworks/standards (free-text list), primary activities (list), and enabled feature flags (which pillar modules apply). | Functional | All / Admin | Must | "Any architect" promise; zero-code extensibility |
| REQ-ROL-EXT-002 | Role-Specific Terminology Glossary | The suite SHALL allow each role configuration to include a role-specific terminology glossary (term, definition, synonyms) that is surfaced in the UI contextually and used by the AI engine to interpret role-specific language. | Functional | All / Admin | Must | Role terminology varies significantly; prevents AI hallucination of wrong domain |
| REQ-ROL-EXT-003 | Role Inheritance | The suite SHALL allow a new role configuration to inherit from an existing role (e.g., "IoT Architect extends Infrastructure Architect") and override or extend specific sections (artifact types, diagram types, questionnaires, AI prompts) without duplicating the entire configuration. | Functional | All / Admin | Should | Reduces configuration effort; promotes reuse |
| REQ-ROL-EXT-004 | Role Activation Per Project | The suite SHALL allow individual projects to activate a subset of the configured roles, so that only the relevant architect roles (and their UI, AI, and deliverable features) are visible to project team members. | Functional | All | Must | Role-scoped UX; avoids cognitive overload |

---

### 12.2 Artifact Type Configuration

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-EXT-005 | Custom Artifact Type Definition | The suite SHALL allow an administrator to define a new artifact type for a role via configuration, specifying: artifact name, description, schema (list of fields with name, type, validation, optionality), display layout (form sections, field order), and version-control behaviour. | Functional | All / Admin | Must | Role-specific artefacts (e.g., IoT Device Catalog, Sustainability Carbon Budget) |
| REQ-ROL-EXT-006 | Artifact Relationship Configuration | The suite SHALL allow configuration of typed relationships between artifact types (e.g., "IoT Device links to Network Zone", "Carbon Budget references Cloud Service"), rendered as diagram edges and traceable in the RTM. | Functional | All / Admin | Must | Cross-artifact traceability; role-specific data model |
| REQ-ROL-EXT-007 | Artifact Lifecycle States | The suite SHALL allow each custom artifact type to define its own lifecycle states (e.g., Draft / Under Review / Approved / Superseded / Archived) with role-based transition permissions. | Governance | All / Admin | Should | Role-tailored governance; lifecycle management |
| REQ-ROL-EXT-008 | Artifact Validation Rules | The suite SHALL allow administrators to define validation rules per artifact type (required fields, cross-field rules, referential integrity checks) that are enforced at save time. | Governance | All / Admin | Should | Data quality; consistency enforcement |

---

### 12.3 Diagram Type Configuration

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-EXT-009 | Custom Diagram Type Definition | The suite SHALL allow configuration of a custom diagram type for a role, specifying: diagram name, description, allowed node types (with shape, icon, colour, attribute schema), allowed edge types (with label, style, cardinality), and layout algorithm preference. | Functional | All / Admin | Must | Role-specific diagram notations (e.g., IoT device-network topology, Sustainability flow) |
| REQ-ROL-EXT-010 | Node Shape & Icon Library Extension | The suite SHALL allow administrators to upload custom SVG icons for new node types in a custom diagram type, and assign them to node stereotypes. | Functional | All / Admin | Should | Visual notation customisation |
| REQ-ROL-EXT-011 | Diagram Validation Rules | The suite SHALL allow configuration of diagram validation rules (e.g., "every IoT Device node must be connected to a Network Zone", "every process node must have at least one data flow out") per custom diagram type. | Governance | All / Admin | Should | Diagram correctness enforcement; notation compliance |
| REQ-ROL-EXT-012 | Diagram-to-Artifact Binding | The suite SHALL allow a custom diagram type to be bound to one or more artifact types so that placing a node of a given type on the diagram auto-creates or links to the corresponding artifact record. | Functional | All / Admin | Must | Round-trip model/diagram consistency; reduces duplicate data entry |

---

### 12.4 Deliverable Template Configuration

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-EXT-013 | Custom Deliverable Template | The suite SHALL allow an administrator to define a custom deliverable template (Word/Excel/PPT/PDF) for a role via a template configuration: document structure (sections, sub-sections), data bindings (which artifact fields/queries populate which sections/tables), diagram placements, and conditional sections (include if certain conditions met). | Functional | All / Admin | Must | Role-specific deliverable generation; V10 integration |
| REQ-ROL-EXT-014 | Template Variable System | The suite SHALL provide a template variable system (using a documented expression language, e.g., Mustache/Handlebars-style {{artifact.field}} or JEXL) that maps artifact data to template placeholders, allowing non-engineers to create deliverable templates. | Functional | All / Admin | Must | No-code template authoring; ESIP extensibility promise |
| REQ-ROL-EXT-015 | Template Version Control | The suite SHALL version-control deliverable templates with author, change note, and the ability to generate a deliverable from any historical template version. | Governance | All / Admin | Should | Audit trail; reproducible deliverables |
| REQ-ROL-EXT-016 | Template Export & Import | The suite SHALL allow exporting a deliverable template as a portable bundle (JSON/YAML + assets) and importing it into another ESIP tenant, enabling a marketplace or community template-sharing model. | Functional | All / Admin | Should | Template ecosystem; cross-tenant reuse |

---

### 12.5 AI Prompt Configuration per Role

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-EXT-017 | Role-Specific AI Prompt Library | The suite SHALL allow each role configuration to include a set of named AI prompt templates (with intent tag, system prompt, user prompt template, output format) covering key role activities (e.g., "Generate a draft {artifact_type} from this requirement set"). | AI/ML | All / Admin | Must | V06 AI engine; role-specific copilot capability |
| REQ-ROL-EXT-018 | AI Prompt Parameter Binding | The suite SHALL allow AI prompt templates to reference live artifact data as parameters (e.g., "{{project.name}}", "{{requirements.list}}", "{{component.technology_stack}}") so prompts are contextualised to the active project and artifacts. | AI/ML | All / Admin | Must | Context-aware AI generation; reduces hallucination |
| REQ-ROL-EXT-019 | AI Prompt Override per Project | The suite SHALL allow project-level override of role AI prompt templates, enabling experimentation or domain-specific tuning without modifying the global role configuration. | AI/ML | All / Admin | Should | Flexibility; localised customisation |
| REQ-ROL-EXT-020 | Prompt Library Versioning | The suite SHALL version each AI prompt in the role prompt library with author, change note, and the ability to rollback to a prior version if outputs degrade. | AI/ML | All / Admin | Should | Prompt governance; quality control |

---

### 12.6 Intake Questionnaire Configuration

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-EXT-021 | Custom Intake Questionnaire | The suite SHALL allow each role configuration to define one or more intake questionnaires (wizard-style multi-step forms) that guide a new project through the role-specific information gathering, with question types: free-text, single-select, multi-select, scale, file-upload, and nested conditional questions. | Functional | All / Admin | Must | Role-specific project on-boarding; AI extraction seed |
| REQ-ROL-EXT-022 | Questionnaire Response–to–Artifact Binding | The suite SHALL automatically populate artifact fields from questionnaire responses when the mapping is configured, eliminating re-entry. | Functional | All / Admin | Must | Data consistency; reduced friction |
| REQ-ROL-EXT-023 | Questionnaire AI Suggestion | The suite SHALL invoke the AI engine to pre-populate questionnaire answers from the project's knowledge base content, presenting suggestions to the architect for acceptance or revision rather than starting from a blank form. | AI/ML | All / Admin | Should | V06 AI engine; productivity |
| REQ-ROL-EXT-024 | Questionnaire Export | The suite SHALL allow exporting and importing questionnaire definitions as portable JSON/YAML bundles for sharing across tenants or role-configuration packages. | Functional | All / Admin | Should | Template marketplace; reuse |

---

### 12.7 Role Governance & Discoverability

| ID | Title | SHALL Description | Category | Persona | Priority | Rationale/Source |
|----|-------|-------------------|----------|---------|----------|-----------------|
| REQ-ROL-EXT-025 | Role Configuration Marketplace | The suite SHALL provide (or support publishing to) a Role Configuration Marketplace where pre-built role configurations (artifact types, diagrams, templates, prompt libraries, questionnaires) can be browsed, installed, rated, and updated, similar to a plugin/extension store. | Functional | All / Admin | Could | Community ecosystem; viral adoption |
| REQ-ROL-EXT-026 | Role Configuration Validation | The suite SHALL validate a role configuration bundle on import for: schema compliance, referential integrity (diagram types reference only declared node types), template variable resolution, and AI prompt syntax; and report validation errors with actionable guidance. | Governance | All / Admin | Must | Configuration quality gate; prevents broken roles |
| REQ-ROL-EXT-027 | Role Usage Analytics | The suite SHALL track role usage analytics per tenant: number of projects per role, artifact creation rate, AI prompt invocation rate, deliverable generation count, and questionnaire completion rate, to inform platform investment decisions. | Reporting | All / Admin | Should | Product analytics; roadmap prioritisation |
| REQ-ROL-EXT-028 | Cross-Role Traceability Matrix | The suite SHALL maintain a cross-role traceability matrix linking artifacts from different roles (e.g., Security Architect threat linked to API Architect OWASP checklist item linked to Integration Architect error-handling control linked to Data Architect data-classification entry) based on configured inter-artifact relationships. | Governance | All | Must | End-to-end traceability; regulatory evidence |

---

## Requirement Summary

| Role / Area | Sub-prefix | Requirements | ID Range |
|-------------|------------|-------------|----------|
| Application Architect | ROL-APP | 25 | REQ-ROL-APP-001 – 029 |
| Integration Architect | ROL-INT | 25 | REQ-ROL-INT-001 – 025 |
| API Architect | ROL-API | 28 | REQ-ROL-API-001 – 028 |
| CRM Architect | ROL-CRM | 20 | REQ-ROL-CRM-001 – 020 |
| Data Architect | ROL-DATA | 22 | REQ-ROL-DATA-001 – 022 |
| Security Architect | ROL-SEC | 24 | REQ-ROL-SEC-001 – 024 |
| Cloud Architect | ROL-CLD | 17 | REQ-ROL-CLD-001 – 017 |
| Infrastructure/Network Architect | ROL-INF | 13 | REQ-ROL-INF-001 – 013 |
| Microservices/Platform/DevOps Architect | ROL-MS | 20 | REQ-ROL-MS-001 – 020 |
| Mobile Architect (brief) | ROL-APP (continued) | 4 | REQ-ROL-APP-022 – 025 |
| AI/ML Architect (brief) | ROL-APP (continued) | 4 | REQ-ROL-APP-026 – 029 |
| Role Extensibility | ROL-EXT | 28 | REQ-ROL-EXT-001 – 028 |
| **Total** | | **~210** | |

---

## Boundary Decisions & Assumptions

1. **V11 (Platform/Metamodel)** owns the runtime execution of role configuration records, artifact storage, schema enforcement, RBAC per role, and the extensibility engine. This volume states *what* must be configurable; V11 states *how* it is stored and executed.
2. **V06 (AI)** owns the AI model selection, RAG infrastructure, prompt execution, guardrails, and copilot orchestration. This volume states *which AI capabilities each role needs*; V06 delivers them.
3. **V03 (Modeling/Diagramming)** owns the diagram canvas, renderer, auto-layout, and export. This volume states *which diagram types* each role requires; V03 must support them all.
4. **V10 (Deliverables)** owns the document/PPT/Excel generation engine and export pipeline. This volume states *which deliverable templates* each role needs.
5. **Application Architect (P3)** is the current ESIP baseline; requirements here extend and formalise it. Existing ESIP component/sequence/class diagrams, UML tooling, and basic component catalog are assumed to exist and are refined here.
6. **Mobile and AI/ML Architect** sections are intentionally brief (4 requirements each) per the volume brief instruction; they are scoped to register their key concerns and enable extensibility — a dedicated volume could follow.
7. **CRM Architect** covers Salesforce primarily with Dynamics 365 as secondary; other CRM platforms (HubSpot, SAP C4C) are referenced for completeness but detailed Salesforce-specific governor limits and order-of-execution are included because Salesforce dominates enterprise CRM market share.
8. **ROL-EXT** requirements are stated from the *consumer perspective* (what an admin configurator can do); the *technical mechanisms* (DSL, metamodel schema, plugin API) are V11 territory.
9. All security-related requirements across roles are cross-referenced to the Security Architect role (P8) rather than duplicated; role-specific security (e.g., OWASP API Top 10 for P5, data classification for P7, IaC scanning for P9) is captured in the relevant role section with a cross-reference tag.

---

*Volume 05 complete. Next: Volume 06 (AI Capabilities) or Volume 07 (Requirements Pillar) per generation schedule.*
