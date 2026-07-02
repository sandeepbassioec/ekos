# ESIP Architecture Automation Suite — Volume 11: Platform, NFRs, Governance, Collaboration, Extensible Metamodel & APIs

**Prefix:** PLT
**Status:** V1.0 — authored 2026-06-23
**Scope:** Extensible Metamodel · Multi-Tenancy & Workspaces · Identity & Access · Security & Compliance · Collaboration · Versioning & Lifecycle · Integration Platform · Admin & Operations · Platform-Wide NFRs
**Boundary notes:**
- AI guardrails, prompt library, AI provider/model management → Volume 06 (here: AI key proxy infra, secrets management, AI cost metering hooks)
- **Autopilot:** autopilot **definitions/catalog** (what autopilots exist, their steps/policies) → **Volume 16**; the autopilot **executor runtime** (the AI orchestration engine that runs them) → **Volume 06**. V11 owns only the platform infra those depend on (event bus, secrets, metering, and the AI-pipeline SLO/capacity NFRs in §9.9).
- Role-specific artifact types and field schemas are *declared* by each role volume (V01–V09) and *governed/enforced* here via the metamodel engine
- Connector-specific ingestion logic (Jira, Confluence, SharePoint) → Volume 08 (here: the platform API surface and event bus that connectors call)
- Deliverable template *content and formatting* → Volume 10 (here: template governance, branding infra, and lifecycle management)
- Diagram renderer internals → Volume 03 (here: versioning of diagram definitions and export governance)
- Portfolio, code-scan details → Volume 09 (here: data residency, retention, and lifecycle of portfolio data)

---

## Table of Contents

1. [Extensible Metamodel](#1-extensible-metamodel)
2. [Multi-Tenancy & Workspaces](#2-multi-tenancy--workspaces)
3. [Identity & Access Management](#3-identity--access-management)
4. [Security & Compliance](#4-security--compliance)
5. [Collaboration](#5-collaboration)
6. [Versioning & Lifecycle Management](#6-versioning--lifecycle-management)
7. [Integration Platform](#7-integration-platform)
8. [Admin & Operations](#8-admin--operations)
9. [Platform-Wide NFRs](#9-platform-wide-nfrs)

---

## 1. Extensible Metamodel

> **Design principle:** Every framework, persona, artifact type, field, relationship, diagram type, viewpoint, deliverable template, AI prompt pack, intake questionnaire, complexity rule, and estimation rule in ESIP SHALL be representable as **governed configuration** — not hard-coded logic. A tenant admin or marketplace publisher configures a new architecture discipline without a software deployment. The metamodel itself is versioned, validated, packaged, imported/exported, and migrated.

### 1.1 Framework Registry

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-FMK-001 | Framework Definition | The platform SHALL provide a Framework Registry where each architecture framework (TOGAF, Zachman, ArchiMate, C4, DoDAF, etc.) is defined as a named configuration record containing: identifier, display name, version, description, maturity status (Draft/Active/Deprecated), owning standards body reference, and linked metamodel package. | Functional | All | Must | Enables "configure, not code" extensibility; covers all V01–V05 frameworks |
| REQ-PLT-FMK-002 | Framework Activation per Tenant | The platform SHALL allow tenant administrators to activate or deactivate specific frameworks for their workspace, so that users only see relevant framework tooling in the UI. | Functional | All | Must | Prevents cognitive overload; supports specialised deployments |
| REQ-PLT-FMK-003 | Framework Versioning | The platform SHALL version each framework definition independently and SHALL maintain a full history of framework configuration changes with author, timestamp, and change description. | Governance | All | Must | Standards bodies release new versions (e.g., TOGAF 9.2 → 10); backward compatibility required |
| REQ-PLT-FMK-004 | Framework Interoperability Mapping | The platform SHALL support declaring cross-framework element mappings (e.g., TOGAF Application Component ↔ ArchiMate ApplicationComponent ↔ C4 Container) as first-class metamodel relationships, enabling cross-framework traceability and views. | Functional | P1, P2, All | Should | Volume 02 framework interop requirements; avoids element duplication |
| REQ-PLT-FMK-005 | Custom Framework Registration | The platform SHALL allow a tenant administrator to register a fully custom framework (e.g., an internal "Enterprise Reference Architecture" methodology) by defining its phases, artifact types, diagram types, viewpoints, and deliverable templates without any code change or platform redeployment. | Functional | P12, All | Must | P12 "Any Architect" extensibility; core differentiator |

---

### 1.2 Persona & Role Configuration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-PRC-001 | Persona Definition | The platform SHALL represent each architecture persona (P1–P13 and custom) as a configuration record containing: identifier, display name, description, associated frameworks, primary artifact types, diagram types, deliverable templates, AI copilot prompt packs, and intake questionnaire references. | Functional | All | Must | Foundation for "any architect" extensibility (V05) |
| REQ-PLT-PRC-002 | Persona Assignment to Users | The platform SHALL allow assigning one or more personas to a platform user, controlling which tooling, artifact types, diagram types, and deliverable templates are exposed in the user's workspace. | Functional | All | Must | Reduces UI noise; enforces appropriate tooling per role |
| REQ-PLT-PRC-003 | Custom Persona Creation | The platform SHALL allow a tenant administrator to create a custom persona by selecting from the library of artifact types, diagram types, deliverable templates, and AI prompt packs, without platform code changes. | Functional | P12, All | Must | P12 extensibility mandate from V05 |
| REQ-PLT-PRC-004 | Persona Journey Configuration | The platform SHALL allow configuring a structured "journey" (ordered phases/steps) for each persona, defining which activities are available in each phase, what the recommended sequence is, and what constitutes completion of each phase. | Functional | All | Should | Guides architects through unfamiliar frameworks; supports onboarding |
| REQ-PLT-PRC-005 | Persona Inheritance | The platform SHALL support persona inheritance, where a custom persona inherits all configuration from a base persona and the administrator overrides only differing elements. | Functional | P12 | Should | Reduces duplication when creating specialised roles (e.g., "Domain Architect" inherits from P2 SA) |

---

### 1.3 Artifact Type Schema

> **Boundary note (global search ownership split):** The **search/index infrastructure** — the full-text index, the **vector store/embeddings**, per-artifact-type index configuration (REQ-PLT-ART-008), and the index-query latency NFR (REQ-PLT-NFR-005) — is **owned here in V11**. The **search experience** — query UX, relevance **ranking/scoring**, facets/filters presentation, and result-surfacing — is owned by **V17**. V11 provides the indexable substrate; V17 builds the search UX on top of it.

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-ART-001 | Artifact Type Registry | The platform SHALL maintain an Artifact Type Registry where each artifact type (e.g., ADR, Capability Map, Component Diagram, Data Model, API Contract) is defined as a configuration record with: type ID, display name, description, associated persona(s), associated framework(s), field schema reference, relationship schema reference, diagram type(s), deliverable template(s), and lifecycle states. | Functional | All | Must | Core metamodel object; enables "configure, not code" |
| REQ-PLT-ART-002 | Field Schema Definition | The platform SHALL allow defining a structured field schema for each artifact type, where each field is configured with: field ID, display label, data type (text, number, date, boolean, enum, multi-select, reference, rich-text, URL, file-attachment), required/optional flag, validation rules (regex, min/max, enum values), default value, display order, and visibility conditions. | Functional | All | Must | Enables arbitrary artifact structure without code; ESIP today has seed field definitions |
| REQ-PLT-ART-003 | Conditional Field Logic | The platform SHALL support conditional field visibility and validation rules, so that a field is shown, hidden, or made required based on the value of another field in the same artifact instance. | Functional | All | Should | Common in enterprise forms (e.g., show "Cloud Region" only when "Deployment Target" = Cloud) |
| REQ-PLT-ART-004 | Artifact Type Lifecycle States | The platform SHALL allow defining a custom lifecycle state machine per artifact type (e.g., Draft → In Review → Approved → Baselined → Retired), including allowed state transitions and the role required to execute each transition. | Governance | All | Must | Required for architecture governance gates (V01 ADM phases, V04 SA gates) |
| REQ-PLT-ART-005 | Cross-Artifact Relationship Schema | The platform SHALL allow defining typed relationships between artifact types (e.g., "Requirement realises Capability", "ADR governs Component", "Capability is supported by Application"), including cardinality constraints (1:1, 1:N, M:N), directionality, and bidirectional display names. | Functional | All | Must | Traceability backbone; underpins RTM (V07) and impact analysis |
| REQ-PLT-ART-006 | Artifact Type Inheritance | The platform SHALL support artifact type inheritance so that a child artifact type inherits all field schema and relationship schema from a parent type and the configuration adds or overrides specific elements. | Functional | P12 | Should | Reduces duplication when creating specialised artifact variants |
| REQ-PLT-ART-007 | Artifact Validation Engine | The platform SHALL execute field validation rules at artifact save time and SHALL surface validation errors per field with actionable messages; SHALL also execute cross-field validation rules (e.g., "If Priority = Critical then Risk Rating is required"). | Functional | All | Must | Data quality; prerequisite for automated deliverable generation |
| REQ-PLT-ART-008 | Artifact Search Index Configuration | The platform SHALL allow specifying which fields of each artifact type are full-text indexed, facet-filterable, and sortable in the platform search engine, configurable per artifact type. | Data | All | Should | Efficient search across thousands of artifacts across a large tenant |

---

### 1.4 Relationship Types & Graph Model

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-REL-001 | Relationship Type Registry | The platform SHALL maintain a Relationship Type Registry defining all typed edges in the platform's knowledge graph, each with: type ID, name, inverse name, source artifact types, target artifact types, cardinality, and display style (line style, colour, label). | Functional | All | Must | Underpins cross-artifact traceability, impact analysis, and portfolio views |
| REQ-PLT-REL-002 | Configurable Relationship Properties | The platform SHALL allow attaching properties to relationships (e.g., weight, confidence, source reference, rationale) where property schemas are configurable per relationship type. | Functional | All | Should | Richer traceability; used in RTM confidence scoring (V07) |
| REQ-PLT-REL-003 | Graph Traversal API | The platform SHALL expose a graph traversal API that allows querying the knowledge graph by: starting artifact, relationship type(s), depth, direction (incoming/outgoing/both), and filter predicates, returning typed result sets. | Integration | All | Must | Enables impact analysis, traceability views, and AI context assembly |
| REQ-PLT-REL-004 | Computed Relationship Inference | The platform SHALL support declaring computed relationship rules (e.g., "Application transitively depends on all technologies used by its components") that are materialised asynchronously and refreshed on upstream change. | Functional | P1, P3, P9 | Should | Portfolio-level impact analysis; tech debt roll-up |

---

### 1.5 Diagram Type & Viewpoint Configuration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-DGM-001 | Diagram Type Registry | The platform SHALL maintain a Diagram Type Registry where each diagram type (ArchiMate view, C4 level, UML diagram type, BPMN process, ERD, Network topology, etc.) is defined with: type ID, name, notation standard reference, allowed element types, allowed relationship types, auto-layout algorithm, validation rules, and export formats. | Functional | All | Must | Enables diagram engine (V03) to be driven by metamodel; allows adding new notation types without code |
| REQ-PLT-DGM-002 | Viewpoint Library | The platform SHALL provide a configurable Viewpoint Library per framework where a viewpoint is defined as: a named diagram filter (which element types and relationship types are shown), a default layout, a purpose statement, and the stakeholder role it serves; new viewpoints SHALL be addable without code changes. | Functional | P1, P2, All | Must | TOGAF ACF viewpoints; ArchiMate viewpoints; DoDAF view products |
| REQ-PLT-DGM-003 | Custom Diagram Type | The platform SHALL allow a tenant administrator to define a custom diagram type by specifying its palette (element types and relationships drawn from the artifact type registry) and layout/render rules, enabling domain-specific notations. | Functional | P12 | Should | "Any Architect" extensibility; e.g., a Telco-specific network diagram type |
| REQ-PLT-DGM-004 | Diagram Element Style Configuration | The platform SHALL allow configuring the default visual style (shape, colour, icon, font, border) for each element type within a diagram type, with tenant-level and project-level overrides. | UX | All | Should | Brand consistency; framework notation compliance |
| REQ-PLT-DGM-005 | Diagram Validation Rule Configuration | The platform SHALL allow defining structural validation rules per diagram type (e.g., "An ArchiMate Business Process must have at least one triggering relationship") that are checked on save and reported as warnings or errors. | Governance | All | Should | Notation compliance; prevents invalid diagrams entering architecture repository |

---

### 1.6 Deliverable Template Governance

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-TPL-001 | Template Registry | The platform SHALL maintain a Deliverable Template Registry where each template (Word, PPT, Excel, PDF report, HTML portal page) is defined with: template ID, name, format, associated framework(s), associated persona(s), content section definitions, data binding expressions, version, and lifecycle status. | Functional | All | Must | Governs template inventory; links with V10 template content |
| REQ-PLT-TPL-002 | Template Versioning | The platform SHALL version each deliverable template and maintain full history; when a template version changes, existing generated deliverables SHALL reference the template version used to generate them; regeneration SHALL use the current template version. | Governance | All | Must | Audit-grade provenance; required for SOC2 change management |
| REQ-PLT-TPL-003 | Tenant Template Override | The platform SHALL allow a tenant administrator to create a tenant-specific override of a platform template (applying corporate branding: logo, colour palette, font, header/footer, cover page) without altering the base template. | Functional | All | Must | Enterprise branding requirement; common enterprise need |
| REQ-PLT-TPL-004 | Template Packaging & Import/Export | The platform SHALL allow exporting a complete template bundle (template file + data binding definitions + metadata) as a portable package and importing it into another tenant or environment. | Integration | All | Should | Template marketplace; cross-tenant template sharing |
| REQ-PLT-TPL-005 | Template Approval Workflow | The platform SHALL support a configurable approval workflow for promoting a template from Draft to Active, requiring sign-off from designated roles, ensuring only approved templates are used in production deliverable generation. | Governance | All | Should | Change governance for approved outputs; aligns with ISO/IEC 27001 A.12.1 change management |

---

### 1.7 AI Prompt Pack Configuration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-PRM-001 | Prompt Pack Registry | The platform SHALL maintain a Prompt Pack Registry where each prompt pack is a versioned, named collection of AI prompt templates associated with a persona, framework, artifact type, and task type (extraction, generation, analysis, classification, summarisation). | AI/ML | All | Must | Decouples prompt engineering from code; enables no-code AI customisation per tenant |
| REQ-PLT-PRM-002 | Prompt Template Field Binding | The platform SHALL support prompt templates with typed placeholder variables that are automatically populated from the current artifact's field values, the project's knowledge hub context, and the active framework configuration at runtime. | AI/ML | All | Must | Enables context-aware prompting; reduces manual prompt crafting by architects |
| REQ-PLT-PRM-003 | Prompt Pack Versioning | The platform SHALL version each prompt pack independently and SHALL maintain a full change history; rollback to a prior prompt pack version SHALL be possible within a tenant without platform redeployment. | Governance | All | Must | AI output reproducibility; audit trail for AI-driven governance decisions |
| REQ-PLT-PRM-004 | Prompt Pack Tenant Override | The platform SHALL allow a tenant to override specific prompt templates in a platform-provided prompt pack with tenant-specific instructions while inheriting all non-overridden prompts from the base pack. | AI/ML | All | Should | Enterprise customisation; competitive differentiation for large customers |
| REQ-PLT-PRM-005 | Prompt Pack Access Control | The platform SHALL restrict which user roles can create, edit, activate, or delete prompt packs, preventing non-authorised users from altering AI behaviour. | Security | All | Must | Governance over AI outputs; misuse prevention |

---

### 1.8 Intake Questionnaire Configuration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-QST-001 | Questionnaire Definition | The platform SHALL allow defining structured intake questionnaires as configuration, where each questionnaire contains: sections, questions (with type: text, number, enum, multi-select, date, file-upload, rating scale), conditional display logic, help text, and validation rules. | Functional | All | Must | Structured ingestion for AI extraction; replaces ad-hoc brief documents |
| REQ-PLT-QST-002 | Questionnaire Association | The platform SHALL allow associating a questionnaire with a project type, a framework, or a persona journey phase, so that the relevant questionnaire is presented automatically at the appropriate workflow stage. | Functional | All | Must | Contextual data collection without user needing to know which form to fill |
| REQ-PLT-QST-003 | Questionnaire Response to Knowledge Hub | The platform SHALL store completed questionnaire responses as structured artifacts in the project Knowledge Hub, making them available as source material for AI extraction and RAG queries. | Data | All | Must | Traceability from intake to architecture decisions; feeds V08 RAG |
| REQ-PLT-QST-004 | Questionnaire Versioning | The platform SHALL version questionnaires; responses SHALL reference the questionnaire version they were answered against; changing a questionnaire SHALL not retroactively alter existing responses. | Data | All | Should | Audit integrity; safe questionnaire evolution |

---

### 1.9 Complexity & Estimation Rule Configuration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-EST-001 | Complexity Rule Definition | The platform SHALL allow defining complexity scoring rules as configuration, where a rule specifies: input fields (from artifact type schemas), a scoring expression (weighted formula or decision table), output score range, and a complexity tier mapping (Simple/Medium/Complex/Very Complex). | Functional | P2, P3, All | Must | Complements SA estimation wizard (V04 SA-10/SA-11); enables role-specific complexity models |
| REQ-PLT-EST-002 | Estimation Template Configuration | The platform SHALL allow defining effort estimation templates as configuration, specifying: effort-hour ranges per complexity tier, activity breakdown ratios (design/build/test/review), role assumptions (architect days, developer days), and adjustment factors. | Functional | P2, P3, All | Must | Standardised, configurable estimation; reduces per-project manual calibration |
| REQ-PLT-EST-003 | Complexity Rule Association | The platform SHALL allow associating complexity rules and estimation templates with specific artifact types, persona journeys, and project types, so that the correct model is applied automatically per context. | Functional | All | Should | Context-aware estimation; prevents applying a cloud migration model to a greenfield app design |
| REQ-PLT-EST-004 | Historical Calibration | The platform SHALL allow feeding actual effort data from completed projects back into the estimation configuration as calibration data, with the ability to adjust estimation templates based on organisation-specific actuals. | Data | All | Could | Continuous improvement of estimation accuracy; aligns with PMBOK earned-value practices |

---

### 1.10 Metamodel Versioning, Validation & Migration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-MMV-001 | Metamodel Version Control | The platform SHALL maintain a versioned history of every metamodel configuration change (framework definitions, artifact types, field schemas, relationship types, diagram types, viewpoints, templates, prompt packs, questionnaires, estimation rules), recording: version number, change timestamp, author identity, and change description. | Governance | All | Must | Audit trail; rollback capability; SOC2 change management control |
| REQ-PLT-MMV-002 | Metamodel Schema Validation | The platform SHALL validate all metamodel configuration records against a platform meta-schema at save time, preventing invalid configuration from entering the active metamodel (e.g., a field referencing a non-existent artifact type, a diagram type referencing an undefined element type). | Governance | All | Must | Configuration integrity; prevents runtime errors from bad metamodel config |
| REQ-PLT-MMV-003 | Metamodel Packaging | The platform SHALL allow packaging a complete or partial metamodel configuration set (e.g., "TOGAF 10 Pack", "Data Architect Pack") into a named, versioned package (JSON/YAML bundle) that can be exported, shared, and imported into another tenant or environment. | Integration | All | Must | Marketplace enablement; cross-organisation sharing; faster new-tenant onboarding |
| REQ-PLT-MMV-004 | Metamodel Marketplace | The platform SHALL provide a curated Metamodel Marketplace where certified packages (frameworks, persona packs, template bundles, prompt packs) can be browsed, rated, and one-click installed into a tenant. | Functional | All | Should | Ecosystem growth; reduces time-to-value for new architecture disciplines |
| REQ-PLT-MMV-005 | Metamodel Import/Export | The platform SHALL support exporting the active metamodel configuration (full or filtered by framework/persona) to a structured file (JSON/YAML) and importing a metamodel file into a tenant, with validation, conflict detection, and a preview of changes before applying. | Integration | All | Must | Portability; disaster recovery of configuration; cross-environment promotion |
| REQ-PLT-MMV-006 | Tenant-Level Metamodel Override | The platform SHALL support a layered metamodel model: a platform-level baseline, a tenant-level override layer, and an optional project-level override layer; each layer can extend or restrict the layer below without modifying the base; changes at lower layers do not affect the base or sibling tenants. | Functional | All | Must | Multi-tenancy isolation; safe customisation without forking the platform baseline |
| REQ-PLT-MMV-007 | Backward-Compatible Schema Migration | The platform SHALL automatically migrate existing artifact instance data when a field schema is updated (field renamed, type widened, new optional field added) using configurable migration rules, with the option to dry-run migration and review impact before committing. | Data | All | Must | Safe metamodel evolution without data loss; large tenants can have millions of artifact instances |
| REQ-PLT-MMV-008 | Breaking Change Detection | The platform SHALL detect and flag breaking metamodel changes (field type narrowing, required field addition, relationship removal, artifact type deletion) that would render existing data invalid, and SHALL require explicit administrator confirmation and a migration plan before applying. | Governance | All | Must | Data integrity; prevents silent data corruption on metamodel updates |
| REQ-PLT-MMV-009 | Metamodel Rollback | The platform SHALL allow a tenant administrator to roll back the active metamodel to any prior version within a configurable retention window (default: 90 days), with automatic migration of artifact instance data to the rolled-back schema. | Governance | All | Should | Recovery from erroneous configuration changes; aligns with DR requirements |
| REQ-PLT-MMV-010 | Metamodel Diff & Change Preview | The platform SHALL provide a visual diff view comparing two metamodel versions, highlighting added, removed, and modified configuration elements, before any import or version promotion is applied. | UX | All | Should | Informed decision-making; reduces risk of unintended metamodel changes in production |

---

## 2. Multi-Tenancy & Workspaces

### 2.1 Tenancy Model

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-TEN-001 | Hard Tenant Isolation | The platform SHALL enforce strict data isolation between tenants at the data storage layer, ensuring no query, API call, or background process can access data belonging to a different tenant; isolation SHALL be verified by automated tests on every release. | Security | All | Must | SOC2 CC6.3; multi-tenancy safety; foundational trust requirement |
| REQ-PLT-TEN-002 | Tenant Provisioning | The platform SHALL support automated tenant provisioning via an admin API that creates a new isolated tenant (organisation), applies the default metamodel configuration, creates the root administrator account, and sends an onboarding email within 60 seconds of request. | Functional | All | Must | Operational efficiency; SaaS onboarding expectation |
| REQ-PLT-TEN-003 | Tenant Offboarding & Data Export | The platform SHALL allow a tenant to export all their data (artifacts, diagrams, documents, audit logs, metamodel configuration) in a standard portable format (JSON + files) before tenant deletion; data SHALL be fully deleted from all storage systems within 30 days of confirmed offboarding request. | Data | All | Must | GDPR Article 17 (right to erasure); contractual right to data portability |
| REQ-PLT-TEN-004 | Tenant Configuration Isolation | The platform SHALL ensure that metamodel customisations, integrations, SSO settings, branding, feature flags, and admin settings made by one tenant have no effect on any other tenant. | Security | All | Must | Configuration isolation; prevents cross-tenant side effects |
| REQ-PLT-TEN-005 | Tenant Suspension | The platform SHALL allow platform administrators to suspend a tenant (disabling all user logins and API access) without deleting tenant data, and to reinstate a suspended tenant, for billing enforcement or security response purposes. | Functional | All | Must | Billing enforcement; incident containment |

---

### 2.2 Organisation / Workspace / Project Hierarchy

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-WRK-001 | Three-Level Hierarchy | The platform SHALL support a three-level hierarchy: Organisation (tenant root) → Workspace (logical boundary for a business unit, programme, or team) → Project (a scoped architecture engagement); each level SHALL have independent access control, settings, and metadata. | Functional | All | Must | Matches real-world enterprise org structures; enables delegated administration |
| REQ-PLT-WRK-002 | Workspace Templates | The platform SHALL allow defining Workspace Templates that pre-configure a workspace with a specific metamodel subset, default artifact types, project structure templates, and role assignments, so new workspaces can be created consistently at scale. | Functional | All | Should | Enterprise onboarding efficiency; programme-level standardisation |
| REQ-PLT-WRK-003 | Cross-Workspace Views | The platform SHALL support read-only cross-workspace views (e.g., an Enterprise Architecture portfolio view spanning all workspaces) available to users with cross-workspace read permission, without collapsing workspace data boundaries. | Functional | P1, All | Should | EA portfolio views require cross-workspace aggregation (V01, V09) |
| REQ-PLT-WRK-004 | Project Archiving | The platform SHALL allow archiving a project, which removes it from active project lists but preserves all data and audit history; archived projects SHALL be searchable and restorable by authorised users. | Data | All | Must | Project lifecycle management; storage management |
| REQ-PLT-WRK-005 | Project Metadata Schema | The platform SHALL allow tenant administrators to define a custom metadata schema for projects (e.g., programme name, business unit, investment category, regulatory scope, criticality tier) used for filtering, reporting, and governance. | Functional | P1, All | Should | Portfolio-level reporting (V09); organisational classification |
| REQ-PLT-WRK-006 | Environment Separation | The platform SHALL support named environment tiers within a project (e.g., Development, Test, Production-Architecture), where each environment tier holds independent artifact baselines and promotion between environments requires authorised approval. | Governance | All | Must | Architecture governance; prevents unreviewed architecture entering production scope |

---

## 3. Identity & Access Management

> **Phase note (MVP vs. Phase 4):** Full **SSO/SAML/SCIM**, **RBAC/ABAC at scale**, and **full multi-tenancy** (§2 hard isolation, tenant provisioning at scale) are **Phase 4** capabilities. The **MVP** uses **simple role checks** (extending ESIP's existing admin/manager/developer/architect/viewer roles) plus a **minimal license scaffold**; the requirements in §3 (and §2) below describe the Phase-4 target, not the MVP build. Priority (MoSCoW) here reflects eventual product scope, not MVP timing — see 00-MASTER-INDEX §6 (Priority≠Phase).

### 3.1 Authentication

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-IAM-001 | SAML 2.0 SSO | The platform SHALL support SAML 2.0 Identity Provider integration, allowing enterprise customers to authenticate users via their corporate identity provider (Okta, Azure AD, ADFS, Ping, etc.) with SP-initiated and IdP-initiated flows. | Security | All | Must | Enterprise procurement gate; SAML 2.0 standard; most enterprise IdPs |
| REQ-PLT-IAM-002 | OIDC / OAuth 2.0 SSO | The platform SHALL support OpenID Connect (OIDC) and OAuth 2.0 for identity federation, including PKCE flow for SPA clients and authorization code flow for server-side clients. | Security | All | Must | Modern IdP standard; Azure AD, Google Workspace, Okta OIDC |
| REQ-PLT-IAM-003 | Multi-Factor Authentication | The platform SHALL enforce MFA for all user accounts accessing the platform, supporting TOTP (RFC 6238), push notification (via OIDC IdP), and hardware security keys (FIDO2/WebAuthn). | Security | All | Must | SOC2 CC6.1; NIST SP 800-63B Level 2; enterprise security posture |
| REQ-PLT-IAM-004 | MFA Enforcement Policy | The platform SHALL allow tenant administrators to configure MFA enforcement policies: require MFA for all users, require MFA only for users with elevated roles (Admin, Approver), or enforce MFA based on network location (off-corp-network). | Security | All | Must | Flexible MFA policy for diverse enterprise security postures |
| REQ-PLT-IAM-005 | Session Management | The platform SHALL enforce configurable session timeout (idle and absolute), session revocation on password change or admin request, and single active session enforcement (optional). | Security | All | Must | SOC2 CC6.1; session security; privilege revocation |
| REQ-PLT-IAM-006 | Existing Firebase Auth Migration | The platform SHALL maintain compatibility with the existing Firebase Authentication implementation during a planned migration to enterprise SSO capabilities, preserving existing user accounts and role assignments through the migration. | Integration | All | Must | ESIP today uses Firebase auth; migration must be non-breaking |

---

### 3.2 Directory & SCIM Provisioning

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-SCM-001 | SCIM 2.0 User Provisioning | The platform SHALL implement SCIM 2.0 (System for Cross-domain Identity Management) to support automated user provisioning and deprovisioning from enterprise identity systems, including create, update, and delete of user accounts and group memberships. | Security | All | Must | SCIM 2.0 (RFC 7642–7644); enterprise IT automation; Okta, Azure AD SCIM support |
| REQ-PLT-SCM-002 | SCIM Group-to-Role Mapping | The platform SHALL allow mapping SCIM groups (or IdP attribute values) to platform roles, so that users are automatically assigned the correct platform roles based on their directory group membership. | Security | All | Must | Just-in-time role assignment; avoids manual role administration at scale |
| REQ-PLT-SCM-003 | Just-in-Time (JIT) Provisioning | The platform SHALL support JIT provisioning via SAML/OIDC assertions, automatically creating a user account on first successful SSO login if the user does not yet exist, mapping identity attributes to user profile fields and initial role assignments. | Security | All | Should | Common enterprise SSO pattern; reduces onboarding friction |
| REQ-PLT-SCM-004 | Deprovisioning & Access Revocation | The platform SHALL immediately revoke platform access when a user is deprovisioned via SCIM or when their IdP account is disabled, invalidating all active sessions and API tokens associated with the user within 60 seconds of deprovisioning event. | Security | All | Must | SOC2 CC6.2; leaver access removal; critical security control |
| REQ-PLT-SCM-005 | User Directory Search | The platform SHALL provide an administrator-accessible user directory within each tenant, searchable by name, email, role, workspace membership, and last-active date. | Functional | All | Must | Administrative oversight; licence management |

---

### 3.3 RBAC & ABAC

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-ACC-001 | Role-Based Access Control | The platform SHALL implement RBAC with a predefined set of platform roles (Platform Admin, Tenant Admin, Workspace Admin, Project Owner, Architect, Reviewer, Viewer, Guest) each with a fixed set of permissions, enforced consistently across UI, API, and background jobs. | Security | All | Must | Industry standard RBAC; SOC2 CC6.3; foundational access control |
| REQ-PLT-ACC-002 | Custom Role Definition | The platform SHALL allow tenant administrators to define custom roles by composing permissions from the platform permission catalogue, enabling fine-grained role tailoring (e.g., "API Governance Reviewer" who can approve API contracts but not edit component diagrams). | Security | All | Should | Enterprise need for role precision; reduces over-privileged access |
| REQ-PLT-ACC-003 | Role Hierarchy | The platform SHALL implement a role hierarchy where higher-level roles inherit all permissions of lower-level roles in the hierarchy; the hierarchy SHALL be configurable by platform administrators. | Security | All | Must | Reduces permission maintenance; standard RBAC best practice |
| REQ-PLT-ACC-004 | Attribute-Based Access Control (ABAC) | The platform SHALL support ABAC policy rules that grant or restrict access based on artifact attributes (e.g., classification: "Confidential", owner: "same-team", framework: "DoDAF") and user attributes (e.g., clearance level, department, location), evaluated at access-decision time. | Security | All | Should | NIST SP 800-162; fine-grained access for regulated data (government, healthcare, financial) |
| REQ-PLT-ACC-005 | Per-Artifact Permissions | The platform SHALL allow setting per-artifact access overrides that restrict read, write, or share permissions to a named set of users or roles, independently of project-level RBAC, for sensitive artifacts. | Security | All | Should | Fine-grained control for commercially sensitive or classified architecture artifacts |
| REQ-PLT-ACC-006 | Per-Field Permissions | The platform SHALL support field-level visibility rules on artifact type schemas, allowing specific fields to be hidden from users without the required role, so sensitive data (e.g., cost estimates, risk ratings) is only visible to authorised roles. | Security | All | Should | Data minimisation; need-to-know access for sensitive fields |
| REQ-PLT-ACC-007 | Delegated Administration | The platform SHALL allow tenant administrators to delegate Workspace Admin rights to workspace-level users, who can then manage membership, roles, and settings within their workspace without tenant-level administrator access. | Functional | All | Must | Operational scalability; avoids central admin bottleneck in large organisations |
| REQ-PLT-ACC-008 | Least Privilege Enforcement | The platform SHALL enforce least-privilege by defaulting new users to the Viewer role in any new workspace they are added to, requiring explicit elevation to higher-privilege roles by an authorised administrator. | Security | All | Must | SOC2 CC6.3; NIST least-privilege principle |
| REQ-PLT-ACC-009 | Privileged Access Review | The platform SHALL generate a periodic (configurable: weekly/monthly/quarterly) Privileged Access Review report listing all users with Admin or Owner roles, for tenant administrators to review and approve or revoke. | Governance | All | Should | SOC2 CC6.3 access review control; ISO/IEC 27001 A.9.2.5 |

---

### 3.4 External Sharing & API Access

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-EXT-001 | Guest / External User Sharing | The platform SHALL allow authenticated users to invite external (non-tenant) users as Guests to specific projects or artifacts, with configurable expiry, limited permissions (view only or comment only), and without requiring the guest to have a full platform account. | Functional | All | Should | Stakeholder review cycles; external consultant access; common SaaS collaboration feature |
| REQ-PLT-EXT-002 | Public Read-Only Share Links | The platform SHALL allow generating time-limited, token-authenticated public read-only share links for specific diagrams or deliverables, accessible to recipients without a platform account. | Functional | All | Could | Executive and stakeholder review without platform access; shareable architecture views |
| REQ-PLT-EXT-003 | API Keys | The platform SHALL allow creating named, scoped API keys per tenant or per service account, with configurable permission scope (read-only, read-write, specific resource types), expiry date, and the ability to revoke at any time. | Security | All | Must | Programmatic integration; CI/CD pipeline integration; connector authentication |
| REQ-PLT-EXT-004 | Service Accounts | The platform SHALL support service accounts (non-human identities) distinct from user accounts, used for automated integrations, background jobs, and CI/CD pipelines, with audit logs attributing actions to the service account identity. | Security | All | Must | Non-human access control; full audit attribution |
| REQ-PLT-EXT-005 | OAuth 2.0 Client Credentials | The platform SHALL support OAuth 2.0 Client Credentials flow for server-to-server API access, allowing external systems to obtain access tokens without user interaction. | Security | All | Should | Standard machine-to-machine auth pattern; enterprise integration requirement |

---

## 4. Security & Compliance

### 4.1 Encryption & Data Protection

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-SEC-001 | Encryption at Rest | All tenant data stored by the platform (databases, file storage, backups, caches, logs) SHALL be encrypted at rest using AES-256 or equivalent, with encryption keys managed in a dedicated key management service (KMS) separate from the data store. | Security | All | Must | SOC2 CC6.1; ISO/IEC 27001 A.10.1.1; GDPR data protection by design |
| REQ-PLT-SEC-002 | Encryption in Transit | All data transmitted between clients and the platform, and between internal platform services, SHALL use TLS 1.2 or higher; TLS 1.0 and 1.1 SHALL be disabled; certificate management SHALL use automated rotation. | Security | All | Must | SOC2 CC6.7; NIST SP 800-52; standard transport security |
| REQ-PLT-SEC-003 | Customer-Managed Encryption Keys (CMEK) | The platform SHALL support Customer-Managed Encryption Keys (CMEK) for tenants requiring key sovereignty, where the tenant provides their own KMS key (AWS KMS, Azure Key Vault, GCP KMS) and the platform encrypts tenant data with that key. | Security | All | Should | HIPAA; FedRAMP; financial sector requirements; data sovereignty |
| REQ-PLT-SEC-004 | Secrets Management (AI Key Proxy) | The platform SHALL manage all AI provider API keys (OpenAI, Anthropic, Azure OpenAI, etc.) via a server-side secrets management service (e.g., HashiCorp Vault, AWS Secrets Manager), never exposing keys to client-side code; ESIP's existing server-side AI proxy architecture SHALL be formalised under this requirement. | Security | All | Must | ESIP current state: AI keys moved to server-side proxy; this formalises and extends that pattern |
| REQ-PLT-SEC-005 | Secret Rotation | The platform SHALL support automated rotation of all platform-managed secrets (database credentials, API keys, signing keys, encryption keys) on a configurable schedule (default: 90 days for API keys, 365 days for encryption keys) with zero-downtime rotation. | Security | All | Must | SOC2 CC6.1; credential hygiene; reduces exposure window |
| REQ-PLT-SEC-006 | Data Masking for Sensitive Fields | The platform SHALL support configuring specific artifact fields as "sensitive" (e.g., cost data, personal data fields), which are masked in logs, audit trails, and AI prompt contexts, unless the user has explicit permission to view them. | Security | All | Should | GDPR data minimisation; prevents sensitive data leakage into AI model contexts |

---

### 4.2 Audit Logging & Tamper-Evidence

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-AUD-001 | Comprehensive Audit Log | The platform SHALL maintain an immutable audit log recording every security-relevant and data-change event, including: user authentication (login, logout, failed login, MFA events), user provisioning/deprovisioning, role assignments, artifact create/read/update/delete, metamodel changes, admin configuration changes, API key creation/use/revocation, and AI task invocations. | Security | All | Must | SOC2 CC7.2; ISO/IEC 27001 A.12.4.1; GDPR Article 30 records of processing |
| REQ-PLT-AUD-002 | Audit Log Tamper-Evidence | The platform SHALL implement cryptographic chaining of audit log entries (each entry includes a hash of the prior entry) to provide tamper-evident audit trails; any tampering SHALL be detectable by an integrity check. | Security | All | Must | SOC2 CC7.3; tamper detection; regulatory requirement for financial/government tenants |
| REQ-PLT-AUD-003 | Audit Log Retention | The platform SHALL retain audit logs for a configurable minimum retention period (default: 2 years; configurable up to 7 years) and SHALL enforce retention even if the tenant requests early deletion, where required by applicable law or contract. | Governance | All | Must | SOC2 CC7.2; GDPR record-keeping; financial regulation (7 years); government (varies) |
| REQ-PLT-AUD-004 | Audit Log Export | The platform SHALL allow authorised administrators to export audit logs in SIEM-compatible formats (JSON, CEF, Splunk HEC) for ingestion into enterprise security monitoring systems. | Security | All | Must | SOC2 CC7.2; enterprise SIEM integration; Splunk, Microsoft Sentinel, Sumo Logic |
| REQ-PLT-AUD-005 | Audit Log Search & Query | The platform SHALL provide a real-time searchable audit log interface within the admin console, supporting filtering by: event type, user, tenant, resource type, resource ID, date range, and outcome (success/failure). | Security | All | Must | Incident investigation; access review; compliance evidence gathering |
| REQ-PLT-AUD-006 | AI Interaction Audit | The platform SHALL log every AI task invocation in the audit trail, recording: user, timestamp, AI task type, artifact/project context, model used, input token count, output token count, and outcome status; AI prompt content SHALL be optionally included based on tenant configuration. | AI/ML | All | Must | AI governance; cost attribution; regulatory audit of AI-assisted architecture decisions |

---

### 4.3 Data Residency & Sovereignty

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-RES-001 | Regional Data Residency | The platform SHALL allow a tenant to select a data residency region (e.g., US, EU, UK, APAC, Australia) at tenant provisioning time; all tenant data SHALL be stored and processed exclusively within the selected region with no cross-region transfer of tenant content. | Security | All | Must | GDPR Chapter V; UK GDPR; data sovereignty requirements; financial/government sector |
| REQ-PLT-RES-002 | Data Transfer Agreements | The platform SHALL implement and document Standard Contractual Clauses (SCCs) and data processing agreements (DPAs) for any cross-border data transfers required for platform operations, with tenant-specific DPAs available on request. | Governance | All | Must | GDPR Article 46; UK adequacy decisions; CCPA data transfer obligations |
| REQ-PLT-RES-003 | AI Processing Location Control | The platform SHALL allow tenants to configure which AI inference regions are permitted for their tenant; for tenants with EU data residency, AI processing SHALL default to EU-region endpoints; US-only AI endpoints SHALL be configurable as an opt-in for EU tenants where legally permissible. | Security | All | Should | GDPR data processing location; AI model provider region constraints |
| REQ-PLT-RES-004 | Data Classification Labelling | The platform SHALL support tenant-configurable data classification labels (e.g., Public, Internal, Confidential, Restricted, Secret) applicable at project, workspace, or artifact level, driving enforcement of access controls, sharing restrictions, and audit policies. | Security | All | Should | ISO/IEC 27001 A.8.2; NIST SP 800-53 RA-2; government classification requirements |

---

### 4.4 Retention, Legal Hold & GDPR Controls

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-RET-001 | Data Retention Policies | The platform SHALL allow tenant administrators to define data retention policies per data category (artifacts, documents, audit logs, AI outputs, deleted items) specifying a retention period; data exceeding the retention period SHALL be automatically purged from all storage systems. | Data | All | Must | GDPR Article 5(1)(e); data minimisation; storage governance |
| REQ-PLT-RET-002 | Legal Hold | The platform SHALL support placing a Legal Hold on specific projects, workspaces, or data categories that suspends all automated deletion and purging of affected data until the hold is lifted by an authorised administrator, with audit trail of hold creation, modification, and release. | Governance | All | Must | Legal e-discovery; litigation hold obligations; FRCP Rule 37(e) |
| REQ-PLT-RET-003 | Right to Erasure (GDPR Art. 17) | The platform SHALL support a Right to Erasure workflow where a user's personal data (name, email, profile) is deleted or anonymised on verified request, preserving audit records by replacing identifying fields with anonymised tokens while retaining event records for legal compliance. | Security | All | Must | GDPR Article 17; privacy by design; reachable via DPA |
| REQ-PLT-RET-004 | Right to Access (GDPR Art. 15) | The platform SHALL support export of all personal data held about a specific user on verified request, in a machine-readable format (JSON), within 30 days. | Security | All | Must | GDPR Article 15; data subject rights; standard enterprise DPA commitment |
| REQ-PLT-RET-005 | Consent Management | The platform SHALL track and record user consent for any optional data processing activities (e.g., usage analytics, AI model improvement), allow users to withdraw consent at any time, and enforce the cessation of the relevant processing within 48 hours of withdrawal. | Security | All | Must | GDPR Article 7; ePrivacy Directive; privacy compliance |

---

### 4.5 Compliance Framework Posture

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-CMP-001 | SOC 2 Type II | The platform SHALL maintain SOC 2 Type II certification (Security, Availability, Confidentiality trust service criteria), conducting annual audits and making the report available to prospective enterprise customers under NDA. | Governance | All | Must | SOC2 AICPA TSC; enterprise procurement gate; B2B SaaS baseline |
| REQ-PLT-CMP-002 | ISO/IEC 27001 Certification | The platform SHALL achieve and maintain ISO/IEC 27001 Information Security Management System (ISMS) certification, conducting annual surveillance audits; the ISMS SHALL cover all platform infrastructure, development, and operations. | Governance | All | Must | ISO/IEC 27001:2022; global enterprise procurement standard; EU/UK requirement |
| REQ-PLT-CMP-003 | GDPR Compliance Programme | The platform SHALL implement and document a GDPR compliance programme including Records of Processing Activities (Art. 30 ROPA), Data Protection Impact Assessments (DPIA) for high-risk processing, DPO appointment (or documented exemption), and breach notification within 72 hours of confirmed breach. | Governance | All | Must | GDPR (EU) 2016/679; UK GDPR; applies to all EU/EEA customer data |
| REQ-PLT-CMP-004 | HIPAA Controls (opt-in) | The platform SHALL provide a HIPAA-eligible configuration tier for tenants processing Protected Health Information (PHI), implementing: BAA execution, additional access controls, minimum necessary data access, PHI audit logging, and breach notification procedures aligned with HIPAA Breach Notification Rule. | Security | All | Should | HIPAA 45 CFR Part 164; healthcare sector tenants |
| REQ-PLT-CMP-005 | Vulnerability Management | The platform SHALL conduct automated vulnerability scanning of platform code and dependencies on every CI pipeline run, conduct external penetration testing annually by a certified third party, and remediate Critical (CVSS ≥9.0) vulnerabilities within 24 hours and High (CVSS 7.0–8.9) within 7 days. | Security | All | Must | SOC2 CC7.1; ISO/IEC 27001 A.12.6.1; responsible disclosure programme |
| REQ-PLT-CMP-006 | Dependency & Supply Chain Security | The platform SHALL track all third-party software dependencies in a Software Bill of Materials (SBOM) per release, monitor dependencies for known CVEs via automated tooling, and maintain a documented policy for third-party vendor security assessment. | Security | All | Must | NIST SSDF; Executive Order 14028; SOC2 supply chain risk |
| REQ-PLT-CMP-007 | Penetration Test Reporting | The platform SHALL make a summary penetration test report and remediation status available to enterprise customers on request (under NDA), covering scope, methodology, findings, and closure evidence. | Governance | All | Should | Enterprise procurement security questionnaire requirement |

---

### 4.6 Backup, Restore & Disaster Recovery

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-DR-001 | Automated Backup | The platform SHALL perform automated backups of all tenant data (databases, file storage, metamodel configuration) with a minimum backup frequency of daily, and retain backups for a configurable period (default: 30 days). | Security | All | Must | SOC2 CC9.1; ISO/IEC 27001 A.12.3.1; operational resilience |
| REQ-PLT-DR-002 | Point-in-Time Recovery | The platform SHALL support point-in-time recovery (PITR) for tenant data, allowing restoration to any point within the last 7 days with RPO ≤ 1 hour, supporting accidental deletion or corruption recovery. | Security | All | Must | Enterprise RPO expectation; BCDR requirement |
| REQ-PLT-DR-003 | Disaster Recovery RTO/RPO | The platform SHALL maintain a Recovery Time Objective (RTO) of ≤ 4 hours and a Recovery Point Objective (RPO) of ≤ 1 hour for a full primary-region failure, with DR runbooks tested in a scheduled drill at minimum semi-annually. | Security | All | Must | SOC2 A1.2 availability; enterprise SLA requirement |
| REQ-PLT-DR-004 | Cross-Region Backup Replication | The platform SHALL replicate backups to a secondary region in the same geopolitical zone as the tenant's selected data residency region, maintaining geographic separation of at least 100 km between primary and backup regions. | Security | All | Should | BCDR best practice; ISO/IEC 22301 business continuity |
| REQ-PLT-DR-005 | Tenant Self-Service Restore | The platform SHALL allow tenant administrators to initiate a self-service project-level restore from the last N backups (N configurable, default 5) without requiring platform-level support intervention. | Functional | All | Should | Reduces support load; tenant autonomy for accidental deletion scenarios |

---

## 5. Collaboration

### 5.1 Real-Time Co-Editing

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-COL-001 | Real-Time Collaborative Editing | The platform SHALL support real-time concurrent editing of architecture diagrams and structured artifact fields by multiple users simultaneously, with changes propagated to all active editors within 500ms under normal network conditions. | Functional | All | Must | Google Docs/Figma UX expectation; distributed team collaboration; Operational Transform or CRDT |
| REQ-PLT-COL-002 | Conflict Resolution | The platform SHALL implement conflict resolution for concurrent edits using operational transformation (OT) or Conflict-free Replicated Data Types (CRDTs), ensuring convergence to a consistent state without data loss when two users edit the same field or diagram element simultaneously. | Functional | All | Must | CRDT (e.g., Yjs, Automerge); correctness requirement for collaborative editing |
| REQ-PLT-COL-003 | User Presence Indicators | The platform SHALL display real-time presence indicators showing which users are currently viewing or editing an artifact or diagram, with named cursor positions for diagram editing and highlighted field focus for form-based artifacts. | UX | All | Should | Awareness of collaborators; reduces collision and confusion |
| REQ-PLT-COL-004 | Edit Attribution | The platform SHALL attribute every change in a collaborative editing session to the originating user, storing attribution in the version history so that each field change or diagram element change can be traced back to its author. | Governance | All | Must | Audit trail completeness; accountability for architecture decisions |

---

### 5.2 Comments, Mentions & Notifications

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-CMT-001 | Contextual Comments | The platform SHALL support threaded comments anchored to specific artifacts, diagram elements, document sections, or deliverable sections, allowing users to leave contextual feedback, questions, and decisions inline. | Functional | All | Must | Confluence-style inline comments; design review workflow |
| REQ-PLT-CMT-002 | @Mention Notifications | The platform SHALL support @mention of any project team member in a comment, triggering an in-platform notification and optional email notification to the mentioned user, with a direct link to the commented item. | Functional | All | Must | Standard SaaS collaboration feature; focus attention on required actions |
| REQ-PLT-CMT-003 | Comment Resolution | The platform SHALL allow marking comments as Resolved, with the ability to reopen a resolved comment; comment resolution SHALL be recorded in the activity feed with attribution. | Functional | All | Must | Design review workflow; prevents review feedback being silently dismissed |
| REQ-PLT-CMT-004 | Notification Preferences | The platform SHALL allow each user to configure their notification preferences per notification type (mention, comment on my artifact, status change, assignment, approval request, system alert), selecting delivery channel (in-platform, email, webhook to Slack/Teams). | UX | All | Should | Notification fatigue management; personalised workflow |
| REQ-PLT-CMT-005 | Comment Export | The platform SHALL support exporting all comments on an artifact or project as a structured report (JSON/Word/PDF), for offline review and governance evidence collection. | Governance | All | Could | External stakeholder review packages; audit evidence |

---

### 5.3 Review & Approval Workflows

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-REV-001 | Configurable Review Workflow | The platform SHALL support configurable multi-stage review and approval workflows for architecture artifacts and deliverables, where stages, approver roles, approval conditions (all-must-approve vs. any-one), and escalation paths are defined as configuration. | Governance | All | Must | Architecture governance gate (V01 ADM phase reviews; V04 SA governance); TOGAF ACF |
| REQ-PLT-REV-002 | Review Assignment | The platform SHALL allow the initiator of a review to assign specific reviewers by name or role, with optional due date, and send automated notifications to all assigned reviewers on review start and reminder notifications N days before due date. | Functional | All | Must | Governance process; prevents reviews being overlooked |
| REQ-PLT-REV-003 | Approval with Digital Signature | The platform SHALL record each approval action as a digitally signed event (signing the artifact version hash + approver identity + timestamp + decision) to provide non-repudiable evidence of approval. | Governance | All | Should | Regulated industries; legal evidence of architecture sign-off; ISO/IEC 27001 change management |
| REQ-PLT-REV-004 | Review Dashboard | The platform SHALL provide a Review Dashboard for each user showing all artifacts and deliverables awaiting their review, sorted by urgency, with one-click access to the review item. | UX | All | Must | Productivity; governance compliance; prevents lost reviews |
| REQ-PLT-REV-005 | Baseline Lock on Approval | The platform SHALL automatically create an immutable baseline of an artifact at the point of final approval, preventing further edits without re-initiating the review workflow for the modified version. | Governance | All | Must | Approved architecture = stable baseline; change control for approved designs |

---

### 5.4 Assignments, Tasks & Activity Feed

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-TSK-001 | Architecture Task Assignments | The platform SHALL allow creating and assigning tasks related to architecture work items (e.g., "Complete capability map", "Review ADR-012", "Update data model for User domain") to named users, with due dates, priority, and status tracking. | Functional | All | Should | Project management lite; action tracking without leaving the architecture tool |
| REQ-PLT-TSK-002 | Activity Feed | The platform SHALL maintain a chronological activity feed per project and per workspace showing all significant events: artifact changes, comment threads, status changes, review starts/completions, assignments, AI task completions, and integration sync events. | UX | All | Must | Team awareness; audit-grade event timeline; async collaboration context |
| REQ-PLT-TSK-003 | Digest Notifications | The platform SHALL allow users to subscribe to a daily or weekly digest notification summarising all activity in their projects and workspaces, reducing real-time notification noise for users in asynchronous workflows. | UX | All | Should | Notification hygiene; async-first collaboration |

---

## 6. Versioning & Lifecycle Management

### 6.1 Artifact & Model Versioning

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-VER-001 | Automatic Version History | The platform SHALL automatically record a version snapshot of every artifact on each save, retaining the full version history for the configurable retention period (default: unlimited within project lifetime), attributing each version to its author. | Data | All | Must | Architecture evolution audit trail; rollback capability; Git-style history |
| REQ-PLT-VER-002 | Named Baselines | The platform SHALL allow users with appropriate permissions to create a named baseline (e.g., "Phase 1 Architecture — Board-approved 2026-06-01") from the current state of any project or subset of artifacts, marking it as immutable. | Governance | All | Must | TOGAF Architecture Repository baselines; governance gates; signed-off architecture |
| REQ-PLT-VER-003 | Branching | The platform SHALL support branching of a project or workspace content set, creating an independent copy of the current artifact state on a named branch for parallel exploration, what-if analysis, or phased planning without affecting the main (trunk) content. | Functional | P1, P2, All | Should | Architecture option analysis; multi-scenario roadmapping; parallel team workstreams |
| REQ-PLT-VER-004 | Merge & Conflict Resolution | The platform SHALL support merging a branch back to the main content set, detecting field-level and relationship-level conflicts and presenting a conflict resolution UI for the user to choose between competing versions or manually compose a merged value. | Functional | P1, P2, All | Should | Branch lifecycle completion; required if branching is implemented |
| REQ-PLT-VER-005 | Diff View | The platform SHALL provide a visual diff view between any two versions of an artifact or diagram, clearly highlighting added fields, removed fields, changed values, and structural diagram changes (elements added/removed/moved). | UX | All | Must | Architecture change review; impact analysis; audit evidence |
| REQ-PLT-VER-006 | Version Rollback | The platform SHALL allow rolling back a specific artifact to any prior version in its history, requiring a justification comment and creating an explicit rollback event in the audit log. | Data | All | Must | Error recovery; governance-mandated rollback scenario |
| REQ-PLT-VER-007 | Deliverable Version History | The platform SHALL version all generated deliverables (Word documents, PPT decks, PDFs, diagrams exports), retaining each generated artefact with the template version and data snapshot used to generate it, accessible from the artifact's version history. | Data | All | Must | Deliverable audit trail; reproduces prior output on demand |

---

### 6.2 Change Governance & Content Promotion

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-CHG-001 | Change Request Workflow | The platform SHALL support a Change Request workflow for baselined/approved artifacts, where any modification request passes through a configurable approval gate before the change is applied to the approved baseline. | Governance | All | Must | Architecture change management; TOGAF Phase H; ITIL Change Management |
| REQ-PLT-CHG-002 | Environment Content Promotion | The platform SHALL support a structured promotion workflow for progressing architecture content from a Development environment to Test and then to Production-Architecture, where each promotion requires an authorised approval and creates an immutable baseline in the target environment. | Governance | All | Must | Architecture lifecycle management; separation of in-progress from approved architecture |
| REQ-PLT-CHG-003 | Architecture Change Impact Report | The platform SHALL generate an Architecture Change Impact Report for any proposed change to a baselined artifact, listing all dependent artifacts, diagrams, deliverables, requirements, and downstream relationships that may be affected by the change. | Governance | P1, P2, All | Should | Change risk assessment; informed decision-making before approving changes |
| REQ-PLT-CHG-004 | Change Freeze Periods | The platform SHALL allow tenant administrators to declare a change freeze period (start/end date, scope: workspace or project) during which write operations are blocked for non-Admin users on affected content, preventing accidental changes during critical periods (e.g., audit windows, programme milestones). | Governance | All | Could | Programme management; audit preparation; regulatory submission windows |

---

## 7. Integration Platform

### 7.1 REST API

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-API-001 | Public REST API | The platform SHALL expose a versioned, documented public REST API covering all core data entities (organizations, workspaces, projects, artifacts, relationships, diagrams, deliverables, users, roles, metamodel elements, audit events), with full CRUD and query operations, authenticated via API keys or OAuth 2.0 bearer tokens. | Integration | All | Must | Programmatic access; CI/CD integration; custom tooling; ecosystem enablement |
| REQ-PLT-API-002 | API Versioning | The platform SHALL version the public API using URL path versioning (e.g., `/v1/`, `/v2/`), maintain prior major API versions for a minimum of 12 months after deprecation notice, and publish a migration guide before each major version deprecation. | Integration | All | Must | Enterprise integration stability; prevents breaking partner integrations |
| REQ-PLT-API-003 | OpenAPI Specification | The platform SHALL publish a complete OpenAPI 3.x specification for the public REST API, kept in sync with the live API on each release, and accessible from the platform developer portal without authentication. | Integration | All | Must | Developer experience; SDK generation; integration testing |
| REQ-PLT-API-004 | Pagination & Filtering | The platform SHALL implement cursor-based pagination for all list endpoints, and support structured server-side filtering, sorting, and sparse fieldsets (field projection) per resource type to support efficient bulk data access patterns. | Integration | All | Must | Performance at scale; large tenant catalogs; efficient synchronisation |
| REQ-PLT-API-005 | Rate Limiting | The platform SHALL enforce per-tenant and per-API-key rate limits on the public API (configurable defaults; minimum 1,000 requests/minute per tenant), returning standardised 429 responses with Retry-After headers. | Security | All | Must | DoS protection; fair use enforcement; SLA protection |
| REQ-PLT-API-006 | API Idempotency | The platform SHALL support idempotency keys on all state-mutating API operations (POST, PUT, PATCH, DELETE), allowing clients to safely retry requests without risk of duplicate processing. | Integration | All | Should | Reliable integrations; network retry safety; event-driven systems |

---

### 7.2 GraphQL API

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-GQL-001 | GraphQL API | The platform SHALL provide a GraphQL API exposing the same data graph as the REST API, allowing clients to query exactly the fields and relationships they need in a single request, with schema introspection enabled. | Integration | All | Should | Efficient frontend data loading; reduces over/under-fetching; modern SPA pattern |
| REQ-PLT-GQL-002 | GraphQL Subscriptions | The platform SHALL support GraphQL Subscriptions (over WebSocket) for real-time event streaming of artifact changes, comment events, review status changes, and activity feed events, for clients that need push-based updates. | Integration | All | Should | Real-time collaboration clients; dashboard live updates |
| REQ-PLT-GQL-003 | GraphQL Depth Limiting | The platform SHALL enforce query depth and complexity limits on all GraphQL queries to prevent denial-of-service via deeply nested queries, returning a structured error if limits are exceeded. | Security | All | Must | GraphQL-specific security; DoS prevention |

---

### 7.3 Webhooks & Event Bus

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-EVT-001 | Outbound Webhooks | The platform SHALL support configuring outbound webhooks per tenant, triggered by platform events (artifact created/updated/deleted, review approved/rejected, deliverable generated, AI task completed, metamodel changed), delivering a signed JSON payload to a configured HTTPS endpoint. | Integration | All | Must | Event-driven integration; Jira/ServiceNow/Slack notification; CI/CD trigger |
| REQ-PLT-EVT-002 | Webhook Payload Signing | The platform SHALL sign all webhook payloads using HMAC-SHA256 with a tenant-specific secret, allowing the receiving endpoint to verify payload authenticity and detect tampering. | Security | All | Must | Webhook security; prevents spoofed events; GitHub-style signing |
| REQ-PLT-EVT-003 | Webhook Retry & Dead-Letter | The platform SHALL retry failed webhook deliveries using exponential backoff (up to configurable max retries, default: 5), and move undeliverable events to a per-tenant dead-letter queue accessible for reprocessing via the admin console. | Integration | All | Must | Reliability; prevents lost integration events; operational visibility |
| REQ-PLT-EVT-004 | Event Bus (Internal) | The platform SHALL implement an internal event bus (Kafka, Pub/Sub, or equivalent) for decoupling platform services, publishing all significant domain events (artifact lifecycle, metamodel changes, AI task results, user actions); consumers SHALL be independently scalable. | Integration | All | Must | Platform scalability; service decoupling; enables connector integrations (V08) |
| REQ-PLT-EVT-005 | Webhook Event Catalogue | The platform SHALL publish a documented Webhook Event Catalogue listing all supported event types, payload schemas, and example payloads, as part of the developer portal. | Integration | All | Must | Developer experience; integration documentation |

---

### 7.4 SDKs & Developer Portal

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-SDK-001 | Official Client SDKs | The platform SHALL provide official client SDKs for TypeScript/JavaScript and Python, generated from the OpenAPI specification, with typed models, authentication helpers, pagination utilities, and retry logic. | Integration | All | Should | Developer onboarding velocity; reduces integration effort; ecosystem growth |
| REQ-PLT-SDK-002 | Developer Portal | The platform SHALL maintain a public developer portal containing: API reference (OpenAPI rendered), Webhook Event Catalogue, SDK documentation, authentication guide, rate limit documentation, changelog, and interactive API explorer (e.g., Swagger UI or Redoc). | Integration | All | Must | Developer experience; self-service integration; enterprise evaluation |
| REQ-PLT-SDK-003 | SDK Changelog & Versioning | The platform SHALL publish a changelog for each SDK release and follow semantic versioning (semver), with at least 3 months advance notice before any breaking SDK change. | Integration | All | Should | Integration stability; enterprise lifecycle management |

---

### 7.5 Import/Export

> **Boundary note (ALM three-way split):** **One-time ALM import** (Jira / Azure DevOps → ESIP, e.g. REQ-PLT-IMP-004) lives **here** in V11. **Read-only ALM evidence** ingestion → **V08**. **Ongoing bidirectional ALM sync** (continuous two-way reconciliation of requirements/stories/status with Jira/ADO) is owned by **V19** — V11 does *not* provide bidirectional sync. (Provider names are illustrative — see 00-MASTER-INDEX §6.)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-IMP-001 | Project Data Export | The platform SHALL support exporting a complete project (all artifacts, diagrams, documents, version history, relationships, metamodel configuration subset, and audit logs) as a portable, self-describing archive file (ZIP containing JSON + binary files), importable into any ESIP instance. | Data | All | Must | Data portability; disaster recovery; cross-instance migration |
| REQ-PLT-IMP-002 | Selective Export | The platform SHALL support selective export of specific artifact types, date ranges, or workspaces, for partial backup, data migration, or regulatory disclosure purposes. | Data | All | Should | Operational flexibility; GDPR subject access requests |
| REQ-PLT-IMP-003 | Industry Format Import/Export | The platform SHALL support import and export of diagrams in standard interchange formats: ArchiMate Model Exchange Format (AMEF/XML), Visio (.vsdx), draw.io XML, PlantUML, Mermaid, and PNG/SVG. | Integration | All | Must | Interoperability with existing tools; import from incumbent tools; V03 requirement alignment |
| REQ-PLT-IMP-004 | ALM System Import | The platform SHALL support importing requirements, user stories, and epics from Jira (via REST API), Azure DevOps (via REST API), and CSV export, creating corresponding Requirement artifacts in ESIP with provenance metadata linking back to the source system. | Integration | All | Must | V07 requirements ingestion; V08 Knowledge Hub connectors; complements V08 detailed connector logic |
| REQ-PLT-IMP-005 | ITSM Integration Hooks | The platform SHALL expose standard integration hooks (API endpoints + webhook events) enabling ITSM platforms (ServiceNow, Jira Service Management) to create architecture review requests, track approval status, and receive completion callbacks from the ESIP review workflow. | Integration | P1, P2, All | Should | Enterprise change management; TOGAF ADM ↔ ITSM change lifecycle |

---

## 8. Admin & Operations

### 8.1 Tenant Admin Console

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-ADM-001 | Tenant Admin Console | The platform SHALL provide a dedicated Tenant Admin Console accessible to users with Tenant Admin role, covering: user management, role management, SSO/SCIM configuration, metamodel management, feature flag management, usage dashboards, billing/quota management, integration settings, and audit log access. | Functional | All | Must | Operational autonomy; reduces support dependency; standard SaaS admin expectation |
| REQ-PLT-ADM-002 | User Lifecycle Management | The tenant admin console SHALL provide full user lifecycle management: invite users, assign roles, reset passwords (or trigger IdP password reset), deactivate/reactivate users, view user activity summary, and bulk-import users via CSV or SCIM. | Functional | All | Must | Day-to-day user administration; onboarding/offboarding efficiency |
| REQ-PLT-ADM-003 | Workspace & Project Management | The tenant admin console SHALL allow creating, configuring, archiving, and deleting workspaces and projects, assigning workspace owners, and viewing aggregate usage metrics (artifact count, storage used, active users) per workspace. | Functional | All | Must | Operational oversight; capacity management |
| REQ-PLT-ADM-004 | SSO Configuration | The tenant admin console SHALL provide a step-by-step SSO configuration wizard for SAML 2.0 and OIDC, with connection test, attribute mapping configuration, and fallback authentication settings. | Security | All | Must | Self-service SSO setup; reduces support tickets; standard enterprise SaaS feature |
| REQ-PLT-ADM-005 | Branding Configuration | The tenant admin console SHALL allow uploading a tenant logo, configuring primary/secondary brand colours, and selecting font families, which are applied to the platform UI, generated deliverables, and email notifications for that tenant. | UX | All | Should | Enterprise brand consistency; white-label readiness |

---

### 8.2 Feature Flags & Configuration Management

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-FLG-001 | Feature Flag Management | The platform SHALL implement a feature flag system allowing platform operators to enable or disable specific features (e.g., "GraphQL API", "Real-time collaboration", "AI copilots") at the platform level, tenant level, or user level, without code deployment. | Functional | All | Must | Safe feature rollout; canary releases; tenant-specific feature gating for beta programmes |
| REQ-PLT-FLG-002 | Gradual Feature Rollout | The platform SHALL support gradual rollout configurations (e.g., enable feature for 10% of tenants, increasing to 100% over 30 days), with automatic rollout progression and the ability to halt/revert at any stage. | Functional | All | Should | Risk mitigation on feature releases; A/B testing of UX changes |
| REQ-PLT-FLG-003 | Configuration as Code | The platform SHALL support exporting and importing all platform and tenant configuration (feature flags, metamodel, SSO settings, integration settings) as version-controlled configuration files (YAML/JSON) compatible with GitOps deployment workflows. | Integration | All | Should | DevOps-grade configuration management; reproducible environments; Infrastructure as Code alignment |

---

### 8.3 Usage Metering, Quotas & Billing

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-MET-001 | Usage Metering | The platform SHALL meter and record all billable usage dimensions per tenant: active user-seats, AI token consumption (by model and task type), storage consumed (artifacts, documents, generated deliverables), API call volume, and number of active integrations. | Data | All | Must | SaaS billing foundation; cost attribution; AI cost governance (complements V06 §13) |
| REQ-PLT-MET-002 | Quota Enforcement | The platform SHALL enforce configurable quotas per tenant for each metered dimension, rejecting or throttling requests that exceed quota limits, and notifying tenant administrators when usage reaches 80% and 100% of any quota threshold. | Functional | All | Must | Fair use; resource protection; billing enforcement |
| REQ-PLT-MET-003 | Usage Dashboard | The platform SHALL provide a usage dashboard visible to tenant administrators showing current consumption vs. quota for all metered dimensions, with time-series charts (daily/weekly/monthly), and per-workspace/per-user breakdown where available. | Reporting | All | Must | Transparency; cost control; informed licence renewal decisions |
| REQ-PLT-MET-004 | Billing Integration | The platform SHALL integrate with at least one billing platform (Stripe or equivalent) to automate subscription management, invoice generation, overage billing, and plan upgrades/downgrades, with tenant-visible invoice history. | Integration | All | Should | SaaS commercial operations; B2B billing automation |
| REQ-PLT-MET-005 | AI Cost Alerts | The platform SHALL allow configuring per-tenant AI cost budget alerts that trigger notifications to tenant administrators when projected monthly AI token spend exceeds configurable thresholds, enabling proactive cost management. | Data | All | Must | AI cost governance; budgetary control; complements V06 §13 AI cost metering |

---

### 8.4 Observability, Health & Rate Limiting

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-OBS-001 | Platform Health Dashboard | The platform SHALL maintain a public status page (e.g., status.esip.io) and an internal health dashboard showing real-time service health indicators for all platform components, with historical uptime metrics for the last 90 days. | Functional | All | Must | Trust; incident communication; SOC2 availability control |
| REQ-PLT-OBS-002 | Distributed Tracing | The platform SHALL implement distributed tracing (OpenTelemetry) across all platform services, allowing platform engineers to trace a single user request end-to-end across services, with traces retained for minimum 7 days. | Functional | All | Must | Performance debugging; latency attribution; SRE practice |
| REQ-PLT-OBS-003 | Structured Logging | The platform SHALL emit structured JSON logs from all services, including: request ID, tenant ID (pseudonymised where required), service name, severity level, timestamp (UTC), and relevant context fields; logs SHALL be forwarded to a centralised log aggregation platform. | Functional | All | Must | Operational debugging; security monitoring; SIEM integration |
| REQ-PLT-OBS-004 | Metrics & Alerting | The platform SHALL expose key platform metrics (request latency p50/p95/p99, error rate, queue depth, DB connection pool utilisation, AI API call latency) via a metrics system (Prometheus or equivalent), with alerting rules that trigger on-call paging for SLA-threatening conditions. | Functional | All | Must | SRE; SLA monitoring; incident detection |
| REQ-PLT-OBS-005 | API Rate Limit Monitoring | The platform SHALL provide a real-time rate limit monitoring view for platform operators and tenant admins, showing current API usage rate vs. quota per tenant and per API key, with historical rate charts. | Functional | All | Should | Capacity management; proactive throttling; abuse detection |

---

## 9. Platform-Wide NFRs

### 9.1 Performance

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-NFR-001 | UI Interaction Latency | The platform UI SHALL render responses to user interactions (navigation, artifact open, field save, diagram element add) within 200ms at p95 for payloads up to 1MB, measured under normal load (up to 1,000 concurrent users per tenant). | NFR | All | Must | Google RAIL model; 200ms perceptual responsiveness threshold; enterprise UX expectation |
| REQ-PLT-NFR-002 | API Response Time | The public REST and GraphQL APIs SHALL respond within 500ms at p95 for single-resource read operations and within 2s at p95 for complex queries involving cross-artifact graph traversal with up to 5 hops, measured under normal load. | NFR | All | Must | API consumer SLA; integration system timeout expectations |
| REQ-PLT-NFR-003 | Diagram Render Performance | The platform SHALL render a complex architecture diagram containing up to 500 elements and 1,000 relationships within 3 seconds of initial load, and incremental updates (single element added/moved) SHALL be visible within 100ms. | NFR | All | Must | Usable at enterprise diagram scale; large TOGAF/ArchiMate views |
| REQ-PLT-NFR-004 | Deliverable Generation Performance | The platform SHALL generate a standard architecture deliverable document (Word/PPT/PDF, up to 100 pages) within 30 seconds of user-initiated generation request, providing a progress indicator and notifying the user on completion. | NFR | All | Must | Practical usability; acceptable wait time for document generation |
| REQ-PLT-NFR-005 | Search Response Time | The platform full-text search over a project's artifact and document corpus SHALL return results within 1 second at p95 for corpora up to 100,000 artifacts and 10GB of document content. | NFR | All | Must | Knowledge Hub usability; V08 search requirements |
| REQ-PLT-NFR-006 | Collaborative Edit Latency | Changes made by one collaborator in a shared editing session SHALL be visible to all other co-editors within 500ms at p95 under normal network conditions (< 100ms RTT). | NFR | All | Must | Real-time collaboration feel; CRDT/OT propagation target |

---

### 9.2 Scalability

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-NFR-007 | Horizontal Scalability | All stateless platform services SHALL scale horizontally by adding instances without manual reconfiguration, supporting auto-scaling based on CPU utilisation, request queue depth, or custom metrics. | NFR | All | Must | Cloud-native scalability; SLA under load spikes |
| REQ-PLT-NFR-008 | Tenant Scale | The platform SHALL support a minimum of 10,000 active tenants on shared infrastructure, with no per-tenant performance degradation as tenant count grows (verified by load test simulation). | NFR | All | Must | SaaS business scale; multi-tenancy efficiency |
| REQ-PLT-NFR-009 | Artifact Volume per Tenant | The platform SHALL maintain full functionality (search, navigation, reporting, AI, versioning) for a tenant with up to 1,000,000 artifacts, 500,000 relationships, and 10,000 concurrent projects without degradation below NFR thresholds. | NFR | All | Should | Large enterprise architecture repository scale; Gartner/LeanIX-class volume |
| REQ-PLT-NFR-010 | Ingestion Throughput | The platform ingestion pipeline SHALL process a batch upload of 1,000 documents (mixed Word/PDF/Excel, average 2MB each) within 60 minutes, with per-document progress visible in the UI. | NFR | All | Should | Bulk onboarding of existing architecture assets; V07 ingestion volumes |

---

### 9.3 Availability & SLA

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-NFR-011 | Availability SLA | The platform SHALL provide a contracted SLA of ≥ 99.9% monthly uptime for the SaaS multi-tenant tier (Standard) and ≥ 99.95% for Enterprise tier, excluding scheduled maintenance windows with ≥ 72 hours advance notice. | NFR | All | Must | Enterprise SaaS baseline SLA; 99.9% = ~8.7 hours downtime/year |
| REQ-PLT-NFR-012 | Graceful Degradation | The platform SHALL implement graceful degradation so that AI service unavailability does not prevent users from performing non-AI tasks (diagram editing, artifact management, deliverable download); AI features SHALL display a clear unavailability notice. | NFR | All | Must | Resilience; prevents total outage due to AI provider downtime; user experience |
| REQ-PLT-NFR-013 | Scheduled Maintenance Windows | The platform SHALL conduct scheduled maintenance in pre-announced windows (default: Sundays 02:00–06:00 UTC) and SHALL avoid breaking changes during maintenance; maintenance SHALL be communicated ≥ 72 hours in advance via status page and email. | NFR | All | Should | Enterprise expectation; no-surprise downtime |

---

### 9.4 Reliability & Maintainability

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-NFR-014 | Automated Test Coverage | The platform SHALL maintain automated test coverage of ≥ 80% for server-side business logic and ≥ 70% for UI components, with test suites executed in CI on every code commit. | NFR | All | Must | Reliability; regression prevention; engineering maturity |
| REQ-PLT-NFR-015 | Zero-Downtime Deployments | The platform SHALL deploy new releases using zero-downtime deployment strategies (blue/green or canary), ensuring no user-visible interruption during routine release deployment. | NFR | All | Must | SaaS operational excellence; continuous delivery |
| REQ-PLT-NFR-016 | Dependency Currency | The platform SHALL maintain a policy of updating third-party dependencies with known Critical or High CVEs within 24 hours (Critical) or 7 days (High), with automated dependency scanning on CI and weekly dependency freshness reports. | NFR | All | Must | Security; supply chain hygiene; SOC2 CC7.1 |
| REQ-PLT-NFR-017 | MTTR Target | The platform SHALL target a Mean Time to Restore (MTTR) of ≤ 30 minutes for P1 (service outage) incidents, supported by on-call rotation, automated alerting, runbook availability, and post-incident review within 48 hours. | NFR | All | Should | SRE operational maturity; enterprise SLA alignment |

---

### 9.5 Accessibility

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-NFR-018 | WCAG 2.1 AA Compliance | The platform UI SHALL conform to Web Content Accessibility Guidelines (WCAG) 2.1 Level AA across all user-facing pages and interactions, verified by automated accessibility scanning (axe-core or equivalent) on every UI release and annual expert manual audit. | NFR | All | Must | ADA / Section 508 (US); EN 301 549 (EU); legal requirement in public sector procurement |
| REQ-PLT-NFR-019 | Keyboard Navigation | All platform UI interactions SHALL be fully operable by keyboard alone, with logical focus order, visible focus indicators, skip-navigation links, and no keyboard traps. | NFR | All | Must | WCAG 2.1 SC 2.1.1; assistive technology users |
| REQ-PLT-NFR-020 | Screen Reader Compatibility | The platform UI SHALL be compatible with leading screen readers (NVDA, JAWS, VoiceOver) for all primary workflows, with ARIA roles and labels on all interactive elements, form fields, error messages, and dynamic content updates. | NFR | All | Should | WCAG 2.1 SC 4.1.2; users with visual impairments |

---

### 9.6 Internationalisation & Localisation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-NFR-021 | UI Internationalisation (i18n) | The platform UI SHALL be fully internationalised, with all user-visible text externalised into resource files, supporting addition of new UI locales without code changes. | NFR | All | Must | Global enterprise deployment; EU/APAC/LATAM customers |
| REQ-PLT-NFR-022 | Initial Locale Support | The platform SHALL ship with English (en-US, en-GB) and SHALL add further locale support (French, German, Spanish, Japanese, Portuguese) on a roadmap basis, with locale coverage tracked publicly. | NFR | All | Should | Global market; enterprise deal requirements in EU and APAC |
| REQ-PLT-NFR-023 | Date, Number & Currency Formatting | The platform SHALL format dates, numbers, currency values, and time zones according to the user's selected locale and time zone preference, stored in their user profile. | NFR | All | Should | Global usability; prevents misinterpretation of dates/numbers across locales |
| REQ-PLT-NFR-024 | Right-to-Left (RTL) Layout | The platform UI SHALL support right-to-left text layout for Arabic and Hebrew locales, with mirrored layout where required. | NFR | All | Could | MENA market; enterprise government customers in Arabic-speaking regions |

---

### 9.7 Browser, Device & Deployment Support

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-NFR-025 | Browser Support | The platform SHALL support the latest two major releases of Google Chrome, Mozilla Firefox, Apple Safari, and Microsoft Edge (Chromium), and SHALL function correctly for the identified primary user workflows in all supported browsers. | NFR | All | Must | Majority of enterprise desktop browser coverage; covers >95% of users |
| REQ-PLT-NFR-026 | Responsive Design | The platform UI SHALL provide a functional, readable experience on screens from 1280px wide (desktop) to 768px wide (tablet), with diagram editing constrained to ≥ 1024px wide screens and a read-only diagram view available at tablet width. | NFR | All | Should | Multi-device enterprise use; executive review on tablet |
| REQ-PLT-NFR-027 | Progressive Web App (PWA) | The platform SHALL be deliverable as a Progressive Web App installable to the OS desktop, with offline read-only access to recently viewed artifacts and diagrams cached locally. | NFR | All | Could | Enterprise desktop experience; offline access in low-connectivity environments |

---

### 9.8 Deployment Models

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-NFR-028 | SaaS Multi-Tenant | The platform SHALL be deployable as a shared SaaS multi-tenant offering on a public cloud provider (AWS, Azure, or GCP), with tenant isolation as defined in §2 and all SLA/compliance controls active. | NFR | All | Must | Primary go-to-market model; broadest reach |
| REQ-PLT-NFR-029 | Single-Tenant SaaS (Dedicated) | The platform SHALL be deployable as a single-tenant dedicated cloud instance (same cloud, isolated infrastructure) for enterprise customers requiring stronger isolation, with all multi-tenancy controls inherited plus dedicated compute and storage. | NFR | All | Should | Enterprise tier offering; customers with strict data isolation requirements |
| REQ-PLT-NFR-030 | Customer VPC Deployment | The platform SHALL be packageable as a deployable stack into a customer-managed VPC on AWS, Azure, or GCP (using Helm charts for Kubernetes or equivalent IaC), with all platform capabilities available and the customer controlling the underlying infrastructure. | NFR | All | Should | Financial sector; regulated industries; enterprise procurement requirement |
| REQ-PLT-NFR-031 | On-Premises / Air-Gapped Deployment | The platform SHALL provide an on-premises deployment package (container images + orchestration manifests + offline metamodel packages) operable in an air-gapped environment with no internet connectivity, substituting cloud-managed services with self-hosted equivalents (local KMS, object storage, identity). | NFR | All | Could | Government; defence (DoDAF/MODAF persona); classified environments; FEDRAMP High |
| REQ-PLT-NFR-032 | Deployment Configuration Parity | All deployment models (SaaS, dedicated, VPC, on-prem) SHALL support the same feature set and metamodel capabilities; any feature unavailable in a given deployment model SHALL be documented and justified. | NFR | All | Must | Consistent product experience across deployment tiers; avoids feature fragmentation |

---

### 9.9 AI-Pipeline SLO Monitoring & Capacity

> **Reconciliation Additions (GAP G-5, platform side).** These platform NFRs make the **AI-pipeline SLOs observable and capacity-managed** at the platform layer. The **per-task SLO targets themselves** (e.g., extraction < 30s/100pp, diagram < 10s, copilot turn < 3s) are **defined in V06** (cross-ref **V06 REQ-AI-SLO**); V11 owns the **monitoring, alerting, and capacity** infrastructure that measures and protects them. Builds on REQ-PLT-OBS-004 (metrics/alerting) and REQ-PLT-NFR-012 (graceful AI degradation).

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-PLT-NFR-033 | AI-Pipeline SLO Monitoring | The platform SHALL measure and report, per AI task type (extraction, diagram generation, copilot turn, analysis, classification, summarisation), the end-to-end latency (p50/p95/p99), throughput, queue depth, and success/error/timeout rates, and SHALL raise alerts when an AI task type breaches its V06-defined SLO target (cross-ref V06 REQ-AI-SLO) over a configurable rolling window. | NFR | All | Must | GAP G-5; makes per-task AI SLOs testable and observable at the platform layer; extends REQ-PLT-OBS-004 to AI-pipeline-specific metrics |
| REQ-PLT-NFR-034 | AI-Pipeline Capacity & Backpressure | The platform SHALL provision and auto-scale AI-pipeline capacity (worker concurrency, queue capacity, provider-rate headroom) to sustain the V06-defined SLO targets under normal and peak load, and SHALL apply backpressure (fair queuing, per-tenant concurrency caps, graceful shed/defer with user-visible status) rather than failing silently when AI-provider or compute capacity is saturated. | NFR | All | Should | GAP G-5; prevents SLO collapse and noisy-neighbour starvation under load; complements REQ-PLT-NFR-012 graceful degradation and REQ-PLT-MET-001 AI metering |

---

## Appendix A: Requirement Count Summary

| Capability Area | Sub-Area | Requirement Count |
|----------------|----------|-------------------|
| 1. Extensible Metamodel | Framework Registry | 5 |
| | Persona & Role Configuration | 5 |
| | Artifact Type Schema | 8 |
| | Relationship Types & Graph Model | 4 |
| | Diagram Type & Viewpoint Configuration | 5 |
| | Deliverable Template Governance | 5 |
| | AI Prompt Pack Configuration | 5 |
| | Intake Questionnaire Configuration | 4 |
| | Complexity & Estimation Rule Configuration | 4 |
| | Metamodel Versioning, Validation & Migration | 10 |
| **Subtotal §1** | | **55** |
| 2. Multi-Tenancy & Workspaces | Tenancy Model | 5 |
| | Organisation/Workspace/Project Hierarchy | 6 |
| **Subtotal §2** | | **11** |
| 3. Identity & Access Management | Authentication | 6 |
| | Directory & SCIM Provisioning | 5 |
| | RBAC & ABAC | 9 |
| | External Sharing & API Access | 5 |
| **Subtotal §3** | | **25** |
| 4. Security & Compliance | Encryption & Data Protection | 6 |
| | Audit Logging & Tamper-Evidence | 6 |
| | Data Residency & Sovereignty | 4 |
| | Retention, Legal Hold & GDPR | 5 |
| | Compliance Framework Posture | 7 |
| | Backup, Restore & Disaster Recovery | 5 |
| **Subtotal §4** | | **33** |
| 5. Collaboration | Real-Time Co-Editing | 4 |
| | Comments, Mentions & Notifications | 5 |
| | Review & Approval Workflows | 5 |
| | Assignments, Tasks & Activity Feed | 3 |
| **Subtotal §5** | | **17** |
| 6. Versioning & Lifecycle Management | Artifact & Model Versioning | 7 |
| | Change Governance & Content Promotion | 4 |
| **Subtotal §6** | | **11** |
| 7. Integration Platform | REST API | 6 |
| | GraphQL API | 3 |
| | Webhooks & Event Bus | 5 |
| | SDKs & Developer Portal | 3 |
| | Import/Export | 5 |
| **Subtotal §7** | | **22** |
| 8. Admin & Operations | Tenant Admin Console | 5 |
| | Feature Flags & Configuration Management | 3 |
| | Usage Metering, Quotas & Billing | 5 |
| | Observability, Health & Rate Limiting | 5 |
| **Subtotal §8** | | **18** |
| 9. Platform-Wide NFRs | Performance | 6 |
| | Scalability | 4 |
| | Availability & SLA | 3 |
| | Reliability & Maintainability | 4 |
| | Accessibility | 3 |
| | Internationalisation & Localisation | 4 |
| | Browser, Device & Deployment Support | 3 |
| | Deployment Models | 5 |
| **Subtotal §9** | | **32** |
| | | |
| **VOLUME TOTAL** | | **224** |

---

## Appendix B: Standards & Frameworks Referenced

| Standard / Framework | Requirement Area | Relevant Requirements |
|---------------------|------------------|-----------------------|
| SOC 2 Type II (AICPA TSC) | Security, Availability, Confidentiality | REQ-PLT-TEN-001, REQ-PLT-IAM-003, REQ-PLT-ACC-001, REQ-PLT-SEC-001–005, REQ-PLT-AUD-001–005, REQ-PLT-CMP-001, REQ-PLT-DR-001–003 |
| ISO/IEC 27001:2022 | ISMS, access control, cryptography, ops security | REQ-PLT-IAM-003, REQ-PLT-SEC-001–005, REQ-PLT-AUD-001–003, REQ-PLT-CMP-002, REQ-PLT-REV-003, REQ-PLT-TPL-005 |
| GDPR (EU) 2016/679 | Data protection, residency, subject rights | REQ-PLT-TEN-003, REQ-PLT-RES-001–004, REQ-PLT-RET-001–005, REQ-PLT-AUD-003, REQ-PLT-SEC-006 |
| UK GDPR | Same scope as EU GDPR for UK customers | REQ-PLT-RES-001, REQ-PLT-RES-002 |
| HIPAA 45 CFR Part 164 | Healthcare PHI controls | REQ-PLT-SEC-003, REQ-PLT-CMP-004 |
| NIST SP 800-63B | Digital identity / MFA levels | REQ-PLT-IAM-003 |
| NIST SP 800-162 | ABAC | REQ-PLT-ACC-004 |
| NIST SP 800-53 | Security controls catalogue | REQ-PLT-RES-004, REQ-PLT-ACC-008 |
| NIST SSDF / EO 14028 | Software supply chain security | REQ-PLT-CMP-006 |
| SAML 2.0 | Federation | REQ-PLT-IAM-001, REQ-PLT-ADM-004 |
| OIDC / OAuth 2.0 | Authentication, API auth | REQ-PLT-IAM-002, REQ-PLT-EXT-003–005 |
| SCIM 2.0 (RFC 7642–7644) | Identity provisioning | REQ-PLT-SCM-001–004 |
| FIDO2 / WebAuthn | Hardware MFA | REQ-PLT-IAM-003 |
| OpenAPI 3.x | API specification | REQ-PLT-API-003, REQ-PLT-SDK-001 |
| WCAG 2.1 AA | Accessibility | REQ-PLT-NFR-018–020 |
| ISO/IEC 22301 | Business continuity | REQ-PLT-DR-004 |
| ADA / Section 508 | US accessibility law | REQ-PLT-NFR-018 |
| EN 301 549 | EU accessibility standard | REQ-PLT-NFR-018 |
| OpenTelemetry | Observability standard | REQ-PLT-OBS-002 |
| TOGAF ADM Phase H | Architecture change management | REQ-PLT-CHG-001 |
| ITIL Change Management | ITSM integration | REQ-PLT-CHG-001, REQ-PLT-IMP-005 |
| ArchiMate Model Exchange Format (AMEF) | Diagram interoperability | REQ-PLT-IMP-003 |

---

## Appendix C: Cross-Volume Dependencies

| This Volume (V11) Requires | From Volume | Specific Dependency |
|---------------------------|-------------|---------------------|
| Artifact type definitions (field schemas) | V01–V09 | Each role volume declares its artifact types; this volume governs and enforces them via metamodel engine |
| AI key proxy / secrets management | V06 | V06 defines AI provider model; V11 REQ-PLT-SEC-004 formalises server-side proxy as platform-security control |
| AI cost metering hooks | V06 §13 | V11 REQ-PLT-MET-001/005 meters AI usage; V06 §13 defines AI cost governance logic |
| Connector integration hooks | V08 | V08 defines connector-specific logic; V11 REQ-PLT-EVT-004 provides the event bus connectors publish to |
| Deliverable template content | V10 | V10 defines template content; V11 REQ-PLT-TPL-001–005 governs their lifecycle and branding |
| Diagram renderer (versioning, export) | V03 | V03 defines renderer; V11 REQ-PLT-VER-007 governs versioning of generated diagram exports |
| Portfolio data lifecycle | V09 | V09 defines portfolio data; V11 §4.4 governs retention/legal hold of that data |
| TOGAF Phase H change management | V01 | V01 defines ADM change flow; V11 REQ-PLT-CHG-001 provides the platform-level workflow mechanism |
| Role-extensibility surface | V05 | V05 defines what extensibility must expose; V11 §1.1–§1.10 define the metamodel engine that realises it |

---

*End of Volume 11 — Platform, NFRs, Governance, Collaboration, Extensible Metamodel & APIs*
*Prefix: PLT | Requirements: 224 | Authored: 2026-06-23*
