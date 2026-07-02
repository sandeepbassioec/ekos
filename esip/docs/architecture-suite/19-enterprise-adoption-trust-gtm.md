# ESIP Architecture Automation Suite — Volume 19: Enterprise Adoption, Trust & Go-to-Market Differentiators

**Product:** ESIP — Enterprise Architecture Automation Suite  
**Volume:** 19  
**Prefix:** ENT  
**Status:** Draft v1.0  
**Date:** 2026-06-23  
**Owner:** Product — Enterprise Go-to-Market & Trust Domain

---

> **Phase banner:** **Enterprise GTM / deployment / trust = Phase 3–4.** Per the catalog-wide ruling (00-MASTER-INDEX §6.1, Priority ≠ Phase), the `Must` priorities throughout this volume mean "required in the product," NOT "in the MVP." The enterprise deployment topologies, trust/compliance attestations, GTM programs, academy/certification, and partner/OEM ecosystem here are sequenced to **Phase 3–4** in `BUILD-PLAN-MVP-AND-PHASING.md`; the Phase-1 Blended MVP does not build this volume. Build sequence is governed by the build plan, not by MoSCoW.

## Scope & Boundary

This volume owns the **deal-closing enterprise layer** — everything that makes a procurement committee say "we need this" rather than "interesting demo." It covers deployment flexibility, trust/security/AI governance, procurement value, delivery ecosystem integration, adoption enablement, openness/no-lock-in, and go-to-market capabilities.

| Concern | Owner |
|---|---|
| AI guardrails, model provider management, prompt library, AI eval | V06 |
| Connector ingestion mechanics (Jira, Confluence, SharePoint, ADO) | V08 |
| Multi-tenancy internals, RBAC/ABAC, SCIM, SSO wiring, API infra | V11 |
| Licensing editions, billing metering, feature gating, delegated admin | V13 |
| Marketplace, competitive migration intelligence, repo→portfolio | V16 |
| Import mechanics (bulk data import, migration tooling) | V18 |
| **Deployment topology options & feature parity across tiers** | **This volume** |
| **SOC 2/ISO 27001/ISO 42001/FedRAMP/HIPAA/GDPR attestations & posture** | **This volume** |
| **ROI/TCO calculator, business-case generator, POC program** | **This volume** |
| **Two-way Jira/ADO sync, Slack/Teams app, browser extension** | **This volume** |
| **Guided playbooks, academy, certification, community** | **This volume** |
| **Architecture-as-code, open formats, full data portability** | **This volume** |
| **Partner/SI program, white-label/OEM, i18n/l10n, accessibility, SLAs** | **This volume** |

---

## Personas Referenced

| Code | Persona |
|---|---|
| Buyer | Economic Buyer (CTO / VP Eng / Chief Architect / CFO) |
| IT | IT / Infrastructure / Platform Team |
| Security | CISO / Security Reviewer / Compliance Officer |
| Admin | Tenant Admin / Workspace Admin |
| EA | Enterprise Architect |
| SA | Solution Architect |
| Dev | Developer / Tech Lead |
| PM | Product Manager |
| CS | Customer Success / Implementation Consultant |
| All | All authenticated personas |

---

## Flagged Differentiators

| # | Differentiator | Sub-area |
|---|---|---|
| D1 | Air-gapped deployment with self-hosted/local LLM (unlocks DoDAF/MODAF offline — defense, gov, banking) | ENT-DEP |
| D2 | Guided playbooks per persona/framework (walk non-experts through TOGAF/Zachman end-to-end) | ENT-ADOPT |
| D3 | Two-way Jira & Azure DevOps sync (architecture stays connected to delivery, not a dead artifact) | ENT-INT |
| D4 | Architecture-as-code — text-based, git-stored, versioned, diffable (openness as a selling point) | ENT-LOCK |
| D5 | Academy + certification + community (ecosystem moat; stickiness beyond feature parity) | ENT-ADOPT |

---

## Table of Contents

1. [ENT-DEP — Deployment Flexibility](#1-ent-dep--deployment-flexibility)
2. [ENT-TRU — Trust, Security & AI Governance](#2-ent-tru--trust-security--ai-governance)
3. [ENT-VAL — Value & Procurement](#3-ent-val--value--procurement)
4. [ENT-INT — Delivery & Ecosystem Integration](#4-ent-int--delivery--ecosystem-integration)
5. [ENT-ADOPT — Adoption & Enablement](#5-ent-adopt--adoption--enablement)
6. [ENT-LOCK — Openness / No Lock-In](#6-ent-lock--openness--no-lock-in)
7. [ENT-ECO — Ecosystem & Go-to-Market](#7-ent-eco--ecosystem--go-to-market)

---

## 1. ENT-DEP — Deployment Flexibility

> **Design principle:** ESIP SHALL be deployable in every topology an enterprise buyer requires — from fully managed SaaS to fully air-gapped on-premises — with identical feature parity and a single unified codebase. No buyer should be forced to compromise on features because of their deployment model.

### 1.1 Deployment Topology Options

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-DEP-001 | SaaS Multi-Tenant | The platform SHALL offer a fully managed SaaS multi-tenant deployment tier where Anthropic/vendor operates all infrastructure, updates, and scaling; tenants share compute with strict data isolation enforced at the application and storage layers. | NFR | Buyer, IT | Must | Default entry point for SMB and mid-market; lowest friction to start |
| REQ-ENT-DEP-002 | Dedicated Single-Tenant SaaS | The platform SHALL offer a dedicated single-tenant SaaS tier where the vendor provisions an isolated environment (dedicated compute, dedicated database, dedicated object storage) per customer, managed by the vendor but logically and physically separate. | NFR | Buyer, IT, Security | Must | Enterprise requirement for isolation; justifies premium pricing |
| REQ-ENT-DEP-003 | Customer VPC Deployment | The platform SHALL support deployment into a customer-owned VPC (AWS, Azure, or GCP) via a published Helm chart or equivalent IaC package, where the customer controls the network boundary, egress, and cloud account but the vendor may provide managed update channels. | NFR | IT, Security | Must | Common in FSI, healthcare — data never leaves customer cloud account |
| REQ-ENT-DEP-004 | Full On-Premises Deployment | The platform SHALL support full on-premises deployment onto customer-owned infrastructure (bare metal or private cloud / VMware / OpenShift / k3s), with no mandatory outbound internet connection required for runtime operation. | NFR | IT, Security | Must | Required for large banks, government agencies, regulated industries |
| REQ-ENT-DEP-005 | **★ D1 — Air-Gapped Deployment with Local LLM** | The platform SHALL support a fully air-gapped deployment mode with zero outbound internet at runtime: all AI capabilities SHALL be powered by a self-hosted or locally running LLM (e.g., Ollama, vLLM, LM Studio, llama.cpp, Hugging Face Inference Server) configured by the administrator; all model weights, embeddings, and vector index SHALL reside on-premises; no telemetry, no phone-home, no cloud AI API calls. | NFR | IT, Security, Buyer | Must | **Differentiator #1 — unlocks DoDAF/MODAF-offline users in defense, classified-government, central banking; competitors cannot match this without significant rearchitecting** |
| REQ-ENT-DEP-006 | Bring-Your-Own-Cloud (BYOC) | The platform SHALL support BYOC across AWS, Azure, GCP, and IBM Cloud, where the customer provides the cloud account/subscription credentials and ESIP deploys and operates within that account; cloud costs appear on the customer's own cloud bill. | Integration | Buyer, IT | Should | Reduces vendor-lock perception; enables enterprise cloud consolidation commitments (AWS/Azure EAs) |
| REQ-ENT-DEP-007 | One-Click / Helm Install | The platform SHALL provide a single-command install path for on-prem and customer-VPC deployments via a published Helm chart (Kubernetes) and an equivalent Docker Compose manifest for smaller footprints, with pre-flight validation and post-install health checks included. | NFR | IT | Must | Reduces deployment friction; IT teams expect Helm-native operations |
| REQ-ENT-DEP-008 | Unified Upgrade Path | The platform SHALL provide a unified upgrade mechanism that works identically across all deployment topologies; upgrades SHALL be delivered as Helm chart version bumps (or equivalent) with schema migration scripts bundled and validated before apply. | NFR | IT | Must | Prevents version drift across deployment models |
| REQ-ENT-DEP-009 | Deployment Configuration Manifest | The platform SHALL provide a deployment configuration manifest (YAML/HCL) that captures all deployment-time choices (topology, DB provider, object storage, AI provider, region, domain, TLS, SMTP, SSO) so deployments are reproducible and version-controlled. | NFR | IT | Must | Infrastructure-as-code compliance; supports GitOps workflows |

### 1.2 Feature Parity Across Tiers

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-DEP-010 | Functional Feature Parity (with enumerated air-gap exclusions) | The platform SHALL deliver identical functional capabilities (all personas, all frameworks, all AI features) across all deployment topologies; no feature SHALL be exclusively available only in SaaS multi-tenant unless technically impossible in air-gapped mode (in which case the limitation SHALL be documented and an offline substitute offered). The following internet-dependent features are the **explicit, exhaustive air-gap EXCLUSIONS** (per 00-MASTER-INDEX §6 ruling 3), each with its documented offline fallback: **(a) Real-time meeting/chat connectors** — Teams/Outlook/Zoom auto-capture (V17 REQ-RTC-*); offline fallback: manual transcript/notes upload into the Knowledge Hub (V08) + the same AI essence/decision extraction. **(b) Integration & ISV Marketplace** (REQ-ENT-ECO-019; V16); offline fallback: side-loading of vendor-signed connector/playbook/template bundles via the admin import path. **(c) Live regulatory horizon feeds** (V20 REQ-FRO-REG-001); offline fallback: the periodically side-loaded offline regulatory library (V20 REQ-FRO-REG-010). **(d) Cross-tenant benchmarking / peer intelligence** (V14/V16); offline fallback: single-tenant-only benchmarks against the tenant's own historical baselines. All four exclusions SHALL be surfaced in the Admin > Deployment panel when air-gapped mode is active, each labelled "SaaS-only / offline-fallback." | NFR | Buyer, Security, IT | Must | Enterprises in regulated industries must not be penalized with fewer features for choosing secure deployment; resolves contradiction C5 by making the air-gap exclusion list explicit rather than implied by a blanket "identical features" promise |
| REQ-ENT-DEP-011 | Air-Gapped AI Feature Parity | In air-gapped mode, AI features that rely on cloud LLMs SHALL have an equivalent local-LLM-powered counterpart; the admin UI SHALL display model capability warnings where a local model's output quality may differ from a cloud model, with guidance on recommended local models. | UX | IT, Admin | Must | Manages expectations; supports model selection decisions |
| REQ-ENT-DEP-012 | Deployment Tier Upgrade Path | The platform SHALL support in-place migration between deployment topologies (e.g., from SaaS multi-tenant to customer VPC or on-prem) via an export-and-restore mechanism with documented runbook, preserving all tenant data, users, configurations, and artifacts. | NFR | IT | Must | Enterprises grow into more secure topologies; blocking migration creates churn |

### 1.3 Sovereign & Region Pinning

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-DEP-013 | Data Residency Region Selection | The platform SHALL allow a tenant to select its primary data residency region at provisioning time (e.g., US-East, EU-West, AP-Southeast) and SHALL guarantee that all tenant data at rest and in transit stays within the selected region; cross-region replication SHALL only occur with explicit tenant consent. | Security | Buyer, Security | Must | GDPR Art. 44–49; EU AI Act; data sovereignty laws in Germany (DSGVO), India (DPDPA), Australia (Privacy Act) |
| REQ-ENT-DEP-014 | Sovereign Cloud Support | The platform SHALL support deployment on sovereign cloud platforms (e.g., AWS GovCloud, Azure Government, Google Workspace for Government, OVHcloud Sovereign) with documentation for each sovereign cloud's compliance posture. | NFR | IT, Security | Should | Required for US federal and EU government buyers |
| REQ-ENT-DEP-015 | Region Pinning for AI Calls | When using cloud AI providers, the platform SHALL allow the admin to restrict AI API calls to a specified cloud region (e.g., Azure OpenAI in EU regions only) so that no inference traffic crosses into non-approved jurisdictions. | Security | Security, Admin | Must | GDPR; regulated industries require data-processing geography control |

---

## 2. ENT-TRU — Trust, Security & AI Governance

> **Design principle:** Trust is a product feature. Every security, compliance, and AI-governance capability in this section SHALL be customer-facing, documentable, and verifiable — not just internal controls.

### 2.1 Security Certifications & Compliance Posture

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-TRU-001 | SOC 2 Type II | The platform SHALL maintain a current (annual) SOC 2 Type II report covering the Trust Services Criteria (Security, Availability, Confidentiality); the report SHALL be made available to enterprise customers under NDA via the trust center. | Security | Buyer, Security | Must | De-facto minimum for B2B SaaS enterprise sales; required by most enterprise procurement checklists |
| REQ-ENT-TRU-002 | ISO 27001 Certification | The platform SHALL obtain and maintain ISO/IEC 27001:2022 certification for its information security management system (ISMS); the certificate SHALL be publicly viewable on the trust center. | Security | Buyer, Security | Must | Required in EU, UK, and APAC enterprise procurement; complements SOC 2 |
| REQ-ENT-TRU-003 | **ISO 42001 AI Management** | The platform SHALL obtain ISO/IEC 42001:2023 certification (AI Management System) — the first international standard for responsible AI — demonstrating documented AI risk assessment, AI impact assessment, AI policy, AI objective monitoring, and AI system lifecycle controls; the certificate SHALL be publicly viewable on the trust center. | Governance | Buyer, Security | Must | **Emerging requirement in EU AI Act-governed procurement; differentiated from almost all current architecture tool competitors; signals AI responsibility at C-suite level** |
| REQ-ENT-TRU-004 | GDPR Compliance | The platform SHALL implement and document GDPR compliance controls: lawful basis for processing, data subject rights (access, erasure, portability, rectification), DPA template, sub-processor list, privacy notice, and data retention automation; a GDPR-readiness checklist SHALL be downloadable from the trust center. | Governance | Security, Buyer | Must | Mandatory for EU/UK customers; increasingly expected globally |
| REQ-ENT-TRU-005 | HIPAA Readiness | The platform SHALL provide a HIPAA-compliant configuration profile for US healthcare customers, including: Business Associate Agreement (BAA) template, PHI boundary controls, audit log hardening, encryption-at-rest and in-transit requirements for PHI, and documented HIPAA Technical Safeguard controls. | Security | Buyer, Security | Should | Healthcare systems and insurers are large architecture tool buyers; BAA is contractual gate |
| REQ-ENT-TRU-006 | FedRAMP Roadmap | The platform SHALL publish a documented FedRAMP Moderate authorization roadmap with target milestone dates; the platform architecture SHALL be designed from the outset to be FedRAMP-compatible (NIST SP 800-53 control families, GovCloud deployment, FIPS 140-2 cryptography, continuous monitoring). | Governance | Buyer, Security | Should | Required for US federal civilian agency contracts; a public roadmap unblocks pre-FedRAMP pilots |
| REQ-ENT-TRU-007 | Penetration Testing | The platform SHALL conduct at least annual third-party penetration tests against the SaaS environment; executive summary results SHALL be available to enterprise customers under NDA; a public disclosure of test dates and no-critical-finding status SHALL appear on the trust center. | Security | Security | Must | Required in enterprise security questionnaires (VSA, SIG, CAIQ) |
| REQ-ENT-TRU-008 | Bug Bounty Program | The platform SHALL operate a public bug bounty program (e.g., via HackerOne or Bugcrowd) with defined scope, severity SLAs, and researcher acknowledgments; the program URL SHALL be linked from the trust center. | Security | Security | Should | Demonstrates proactive vulnerability management; reduces questionnaire friction |
| REQ-ENT-TRU-009 | Vulnerability Disclosure Policy | The platform SHALL publish a Coordinated Vulnerability Disclosure (CVD) policy at a well-known URL (e.g., /security or security.txt per RFC 9116), including contact method, PGP key, and response SLAs. | Security | Security | Must | RFC 9116 compliance; expected by security-conscious enterprise buyers |

### 2.2 Data Privacy & Customer Data Policy

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-TRU-010 | **No Training on Customer Data** | The platform SHALL explicitly guarantee — in the Terms of Service, Privacy Policy, and DPA — that no customer data (artifacts, diagrams, documents, queries, AI outputs, or any other tenant content) is used to train, fine-tune, or improve any AI model, whether vendor-operated or third-party; this commitment SHALL be displayed prominently in the UI in the Admin > Trust & Privacy settings screen. | Governance | Buyer, Security | Must | **Top-of-list concern for enterprise AI tool procurement; failure to state this clearly is a deal-killer in financial services, legal, and defense** |
| REQ-ENT-TRU-011 | Customer Data Isolation | The platform SHALL enforce strict data isolation between tenants at the storage, caching, and AI context layers: no customer's data, embeddings, or vector indices SHALL be accessible from another customer's session or queries; isolation SHALL be enforced regardless of deployment topology. | Security | Security | Must | Foundational multi-tenancy security; required in all enterprise security questionnaires |
| REQ-ENT-TRU-012 | Sub-Processor Transparency | The platform SHALL maintain and publish a current sub-processor list (name, location, processing purpose, data categories) on the trust center; customers SHALL receive 30-day advance notice before adding new sub-processors, with opt-out rights as required by GDPR Art. 28. | Governance | Security, Buyer | Must | GDPR Art. 28; standard DPA obligation |
| REQ-ENT-TRU-013 | Data Retention & Deletion | The platform SHALL allow tenant admins to configure data retention periods per content category (artifacts, documents, AI interaction logs, audit logs); upon tenant off-boarding, the platform SHALL execute a documented secure-deletion process and provide a deletion certificate within 30 days. | Governance | Admin, Security | Must | GDPR Art. 17 (Right to Erasure); procurement requirement |
| REQ-ENT-TRU-014 | Data Processing Addendum | The platform SHALL provide a ready-to-sign Data Processing Addendum (DPA) compliant with GDPR Art. 28, SCCs (where applicable), and UK IDTA; the DPA SHALL be available for self-serve electronic signature from the admin billing/legal panel. | Governance | Buyer | Must | Unblocks procurement in EU/UK without legal back-and-forth |

### 2.3 Encryption & Key Management

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-TRU-015 | Encryption at Rest | All tenant data at rest (database records, object storage, backups, vector indices) SHALL be encrypted using AES-256; encryption SHALL be enforced at the platform layer, not dependent on underlying cloud defaults alone. | Security | Security | Must | Baseline security control; required in all security questionnaires |
| REQ-ENT-TRU-016 | Encryption in Transit | All data in transit (browser↔API, service↔service, platform↔cloud storage, platform↔AI provider) SHALL be encrypted using TLS 1.2 minimum (TLS 1.3 preferred); weak cipher suites SHALL be disabled; certificates SHALL be auto-renewed. | Security | Security | Must | Baseline security control |
| REQ-ENT-TRU-017 | Bring-Your-Own-Key (BYOK) | The platform SHALL support Bring-Your-Own-Key (BYOK) encryption where enterprise tenants supply their own customer-managed encryption keys (CMEK) stored in their own KMS (AWS KMS, Azure Key Vault, GCP Cloud KMS, HashiCorp Vault); the platform SHALL use envelope encryption so that revoking the tenant key immediately renders all tenant data unreadable. | Security | Security, Admin | Must | Required by FSI, legal, and defense buyers; BYOK is a hard gate in many enterprise procurement processes |
| REQ-ENT-TRU-018 | Customer-Managed Key Rotation | The platform SHALL support customer-initiated key rotation; upon key rotation, the platform SHALL re-encrypt all affected data envelopes in the background without service interruption and log rotation events to the immutable audit trail. | Security | Admin, Security | Should | Operational key hygiene; required in FedRAMP and ISO 27001 controls |
| REQ-ENT-TRU-019 | FIPS 140-2 Cryptography Mode | The platform SHALL provide a FIPS 140-2 validated cryptography mode (using FIPS-validated libraries and modules) activatable for FedRAMP and US DoD deployments; this mode SHALL apply to all cryptographic operations platform-wide. | Security | Security, IT | Should | FedRAMP Moderate; US federal procurement; DoD STIG baseline |

### 2.4 AI Governance & Explainability

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-TRU-020 | AI Output Citations | Every AI-generated architecture artifact, recommendation, diagram, or analysis output SHALL include source citations linking back to the specific evidence items (documents, requirements, connector data) used to generate the output; citations SHALL be browsable inline in the UI. | AI/ML | All | Must | EU AI Act Art. 13 (transparency); ISO 42001 §6.1 (AI risk assessment); prevents "hallucination laundering" |
| REQ-ENT-TRU-021 | AI Confidence Scoring | The platform SHALL display a calibrated confidence indicator on every AI-generated content block; confidence SHALL reflect retrieval relevance, model certainty, and source freshness; low-confidence outputs SHALL be visually flagged and require human review before promotion to Approved state. | AI/ML | All | Must | ISO 42001; EU AI Act; enterprise architects need to know when to trust AI |
| REQ-ENT-TRU-022 | AI Interaction Audit Log | Every AI prompt and response (including model ID, version, temperature, token counts, latency, user ID, tenant ID, and a SHA-256 hash of the prompt/response) SHALL be written to an immutable, tamper-evident audit log; the log SHALL be queryable by tenant admins and exportable for compliance review. | Governance | Admin, Security | Must | ISO 42001; SOC 2 CC7; enterprise compliance requirement |
| REQ-ENT-TRU-023 | AI Guardrails Reference | All AI guardrails (content filtering, output validation, bias detection, harmful-content blocking, scope enforcement, hallucination mitigation) are defined in Volume 06; this volume requires only that: (a) guardrail status per tenant is visible in the Admin > AI Governance panel, and (b) guardrail violations are surfaced in the trust center's operational health feed. | AI/ML | Admin, Security | Must | Boundary: V06 owns guardrail logic; this volume owns the trust-center-visible surface |
| REQ-ENT-TRU-024 | AI Model Provenance | For every AI inference, the platform SHALL record and display: model provider, model name, model version/snapshot date, deployment region, and whether the call was cloud or local/air-gapped; this provenance SHALL be part of the artifact's full provenance record. | Governance | Security, Admin | Should | ISO 42001 §8 (AI system lifecycle documentation); EU AI Act supply chain transparency |
| REQ-ENT-TRU-025 | AI Purpose Limitation | The platform SHALL enforce AI purpose limitation: AI models SHALL only be invoked for documented, user-visible purposes (stated in the Privacy Policy and in-app AI-use disclosures); no hidden background AI analysis of user content SHALL occur without explicit user awareness and consent. | Governance | Security, Buyer | Must | GDPR Art. 5(1)(b); ISO 42001; EU AI Act |

### 2.5 Trust Center & Security Questionnaire Automation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-TRU-026 | Public Trust Center | The platform SHALL operate a public-facing trust center (e.g., trust.esip.io) listing: certifications with certificate links, SOC 2 report request process, penetration test summary, sub-processor list, privacy policy, DPA, CVD policy, uptime/status page link, and security contact; the trust center SHALL be kept current within 30 days of any status change. | Governance | Buyer, Security | Must | Reduces sales-cycle friction; procurement teams check trust centers before scheduling demos |
| REQ-ENT-TRU-027 | Pre-Filled Security Questionnaire Pack (CAIQ/SIG/VSA) | The platform SHALL provide a downloadable, pre-filled Consensus Assessments Initiative Questionnaire (CAIQ v4), Shared Assessments SIG, and Vendor Security Assessment (VSA) response pack; the pack SHALL be updated with each major release and available from the trust center without requiring an NDA. | Governance | Security, Buyer | Must | CAIQ/SIG completion is a gate in 80%+ enterprise security reviews; providing pre-filled responses eliminates weeks of back-and-forth |
| REQ-ENT-TRU-028 | AI-Assisted Security Questionnaire Response | The platform SHALL provide an internal tool that uses AI to auto-draft responses to customer-specific security questionnaires (uploaded as Word/Excel/PDF) by matching questions to the platform's documented controls database; a human reviewer approves before sending. | AI/ML | CS, Security | Should | Scales enterprise sales enablement; reduces security team burden |

### 2.6 Reconciliation Additions — IP Ownership & Indemnity of AI-Generated Output (GAP G-1)

> These requirements close **GAP G-1** from the FINALIZATION-REPORT (legal/IP ownership & indemnity of AI-generated content — a top procurement blocker). They establish an explicit position on who owns AI output, third-party-infringement indemnity, and the supporting Terms-of-Service liability language. Prefix continues the ENT-TRU sequence.

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-TRU-029 | Customer Ownership of AI-Generated Output | The platform's Terms of Service and DPA SHALL explicitly state that, as between the parties, the **customer owns all right, title, and interest in the architecture artifacts, diagrams, documents, and other output generated by the AI from the customer's inputs** (subject only to the customer's own licenses in any third-party material they supplied); the vendor SHALL assign or waive any claim to ownership of customer AI output and SHALL NOT assert IP rights over it. This ownership position SHALL be displayed in the Admin > Trust & Privacy screen alongside the no-training guarantee (REQ-ENT-TRU-010). | Governance | Buyer, Security | Must | GAP G-1; output-ownership ambiguity is a top procurement blocker for AI tools in legal/FSI/defense — buyers will not adopt a tool that claims rights over their generated architecture |
| REQ-ENT-TRU-030 | Third-Party IP Infringement Indemnity | The platform's commercial agreement SHALL include a **vendor IP-infringement indemnity** under which the vendor defends and indemnifies the customer against third-party claims that the platform itself, or AI output generated by the platform's own models in their as-delivered state, infringes that third party's intellectual-property rights; the indemnity SHALL state its scope, the customer's notice obligations, and standard carve-outs (e.g., customer-supplied inputs, customer modifications, use in combination with non-ESIP products). Enterprise-tier agreements SHALL offer an uncapped or elevated-cap indemnity option for this clause. | Governance | Buyer, Security | Must | GAP G-1; an AI-output infringement indemnity is increasingly a hard contractual gate in enterprise AI procurement; its absence stalls legal review indefinitely |
| REQ-ENT-TRU-031 | AI Output Liability & Acceptable-Use Terms | The platform's Terms of Service SHALL include clear AI-specific liability language covering: (a) the "human-in-the-loop" expectation that AI output is a draft requiring professional review before reliance (cross-ref AI confidence scoring REQ-ENT-TRU-021); (b) allocation of liability and disclaimers for decisions made on AI output; (c) an acceptable-use policy for the AI features; and (d) how the no-training guarantee (REQ-ENT-TRU-010), output ownership (REQ-ENT-TRU-029), and infringement indemnity (REQ-ENT-TRU-030) interlock. These terms SHALL be surfaced in-product at first AI use and versioned. | Governance | Buyer, Security | Must | GAP G-1; procurement and legal teams require an explicit, coherent ToS liability position on AI before sign-off; piecemeal or missing terms are a deal-killer |

---

## 3. ENT-VAL — Value & Procurement

> **Design principle:** Enterprise buyers need to justify ESIP to a CFO or procurement committee. ESIP SHALL provide the business-case collateral, calculator tools, and procurement artifacts that turn a technical champion's enthusiasm into a signed contract.

### 3.1 ROI & TCO Calculator

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-VAL-001 | ROI / TCO Calculator | The platform SHALL provide an interactive, embeddable ROI/TCO calculator (available on the marketing site and within the product) that accepts inputs: number of architects by persona, current tools and licenses (LeanIX, Sparx, Visio, Confluence, Miro, Lucidchart, Archimate tool, etc.), time spent per workflow step, and hourly rates; and produces: total current-state cost, projected ESIP cost, net savings, payback period, and 3-year NPV. | Functional | Buyer | Must | CFO/procurement gate; deals stall without a numbers story |
| REQ-ENT-VAL-002 | Tool Consolidation Comparison | The platform SHALL provide a side-by-side capability comparison matrix showing ESIP covering the combined function set of: LeanIX (enterprise portfolio), Sparx EA (modeling), Visio/Lucidchart/draw.io (diagramming), Confluence (architecture wiki), Ardoq (dependency mapping), BiZZdesign/Hopex (frameworks), ServiceNow APM (application portfolio), and Jira (delivery tracking) — with evidence-mapped capability coverage scores; this SHALL be available as a downloadable PDF leave-behind. | Reporting | Buyer | Must | "Consolidate N tools into 1" is the anchor message; visual proof points close deals |
| REQ-ENT-VAL-003 | Business Case Generator | The platform SHALL provide an AI-assisted business case document generator that takes ROI calculator outputs, selected use cases, and customer-provided context (industry, headcount, transformation program name) and produces a formatted Word/PDF business case document suitable for executive sponsorship approval, including: executive summary, current pain points, ESIP solution fit, financial justification, risk mitigation, and recommended roadmap. | AI/ML | Buyer, CS | Must | Reduces time-to-LOI; technical champions need this to present to CFO |
| REQ-ENT-VAL-004 | Time-to-Value Metric | The platform SHALL measure and display, within the admin dashboard, the time elapsed from tenant provisioning to first architecture artifact published; this metric SHALL be used in success benchmarking and SHALL be referenced in marketing materials (e.g., "repo-to-portfolio in under 15 minutes"). | Reporting | Buyer, Admin | Should | Rapid time-to-value is a differentiator in competitive bake-offs (ref V16) |
| REQ-ENT-VAL-005 | Per-Persona License Cost Breakdown | The ROI/TCO calculator SHALL break down cost and savings at the per-persona license level (EA, SA, AppA, etc.) so a buyer can model a phased adoption (start with EAs, add SAs in Q3) and see the incremental cost/savings at each step. | Functional | Buyer | Should | Reduces risk perception; allows partial commitment as initial ask |

### 3.2 Free POC / Sandbox Program

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-VAL-006 | Guided POC / Sandbox Environment | The platform SHALL offer a time-boxed (30-day default, extendable), fully featured, guided POC/sandbox environment for enterprise prospects, pre-loaded with synthetic reference data (a realistic sample enterprise architecture for a fictional organization) covering all personas and frameworks. | UX | Buyer, EA, SA | Must | Enterprise buyers cannot commit without hands-on validation; a guided sandbox de-risks the evaluation |
| REQ-ENT-VAL-007 | POC Success Checklist | The POC environment SHALL include a built-in "POC Success Checklist" with a curated set of scenario-based tasks (per persona) that demonstrate the highest-value capabilities; completion of the checklist SHALL generate a downloadable POC summary report. | UX | Buyer, CS | Must | Structures the evaluation; ensures buyers experience differentiating features rather than random clicking |
| REQ-ENT-VAL-008 | POC-to-Production Data Carry-Over | Any data created in the POC/sandbox environment SHALL be exportable and importable into a production tenant without loss; the platform SHALL provide a documented migration path from POC to production. | Functional | IT, Buyer | Must | Prevents "start from scratch" objection at close; protects POC investment |
| REQ-ENT-VAL-009 | Shared POC Evaluation Environment for Partners | The platform SHALL allow certified SI partners and resellers to provision POC environments on behalf of their prospects, with partner branding, without requiring the prospect to create an account first. | Functional | CS | Should | Scales partner-led selling motion; reduces friction in partner-delivered evaluations |

### 3.3 Transparent Pricing & Analyst Readiness

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-VAL-010 | Transparent, Flexible Licensing | The platform SHALL publish pricing tiers publicly (or provide a pricing calculator on the sales page) covering SaaS multi-tenant, dedicated SaaS, and on-prem/BYOC options; per-persona and enterprise-flat licensing models SHALL both be available (ref V13). | Governance | Buyer | Must | Price transparency reduces sales-cycle length; analyst firms (Gartner/Forrester) penalize opaque pricing |
| REQ-ENT-VAL-011 | Analyst Briefing & RFI Readiness | The platform SHALL maintain an analyst relations pack (Gartner MQ/Critical Capabilities, Forrester Wave, G2/Capterra profile materials) updated quarterly; the pack SHALL include: product overview, differentiation narrative, reference customer list (with permission), win/loss data, and roadmap briefing. | Reporting | Buyer | Should | Gartner MQ inclusion drives 20–40% of enterprise inbound; Forrester Wave inclusion is a deal qualifier |
| REQ-ENT-VAL-012 | G2 / Peer Review Enablement | The platform SHALL include an in-app prompt (suppressible per tenant) inviting satisfied users to leave a review on G2, TrustRadius, or Capterra after completing their first major artifact; the request SHALL be sent once per user, 14 days after first meaningful use. | UX | All | Should | Peer reviews are researched by 80%+ of enterprise buyers pre-shortlist |
| REQ-ENT-VAL-013 | Enterprise Procurement Artifact Pack | The platform SHALL provide a downloadable enterprise procurement artifact pack including: Company overview, ISO 27001 cert, SOC 2 Type II summary, DPA, Insurance certificate (Cyber, E&O, GL), Financial stability attestation, SLA/support terms, and Accessibility conformance statement (VPAT); all documents SHALL be accessible from the trust center. | Governance | Buyer, Security | Must | Procurement teams request these items within 48 hours of verbal commitment; having them ready removes weeks of delay |

---

## 4. ENT-INT — Delivery & Ecosystem Integration

> **Design principle:** Architecture that lives only in ESIP is a dead artifact. ESIP SHALL be the connective tissue that keeps architecture decisions alive inside the delivery tools, communication platforms, and enterprise systems where work actually happens.

### 4.1 Two-Way Jira & Azure DevOps Sync ★ D3

> **Reconciliation note (ownership — ALM sync, three-way split):** **V19 is the canonical owner of *ongoing bidirectional* Jira & Azure DevOps sync** (REQ-ENT-INT-001–006) — continuous two-way push/pull, field-mapping, conflict resolution, and delivery traceability. This is distinct from the other two ALM touchpoints: **V08 = read-only evidence ingest** (Jira/ADO issues pulled into the Knowledge Hub as context, no write-back); **V11 = one-time ALM *import*** (bulk migration of an existing backlog at onboarding). The connector transport/OAuth/credential plumbing is V08/V11; the live bidirectional-sync behaviour and its admin surface are owned here. (V17 REQ-PM-066 PM-backlog↔Jira/ADO sync rides on this same V19 sync engine.) Resolves the FINALIZATION-REPORT "Jira/ADO three-way" action.

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-INT-001 | **★ D3 — Jira Two-Way Sync** | The platform SHALL provide a bi-directional sync with Jira (Cloud and Data Center) where: ESIP requirements/stories/decisions can be pushed to Jira as issues; Jira issues can be pulled into ESIP as requirements or knowledge items; status changes in Jira are reflected in ESIP (and vice versa); traceability links between ESIP artifacts and Jira issues are maintained as hyperlinks on both sides. | Integration | EA, SA, Dev, PM | Must | **Differentiator #3 — keeps architecture connected to delivery; without this, ESIP becomes shelfware after the initial model is built** |
| REQ-ENT-INT-002 | Jira Sync Field Mapping Configuration | Admins SHALL be able to configure per-project field mappings between ESIP artifact fields and Jira issue fields (type, priority, assignee, component, label, custom fields) via a visual mapping UI; mappings SHALL be saveable as reusable profiles. | Integration | Admin | Must | Enterprises have diverse Jira schemas; rigid mapping blocks adoption |
| REQ-ENT-INT-003 | Jira Sync Conflict Resolution | The platform SHALL detect and surface sync conflicts (same field updated in both systems since last sync) and present a conflict resolution UI that shows both values and allows the user to choose which wins; resolved conflicts SHALL be logged. | Integration | Admin, EA | Must | Bi-directional sync without conflict resolution causes data corruption |
| REQ-ENT-INT-004 | **★ D3 — Azure DevOps Two-Way Sync** | The platform SHALL provide a bi-directional sync with Azure DevOps (ADO) Services and ADO Server, equivalent in scope to the Jira sync (REQ-ENT-INT-001), supporting: Work Items (User Stories, Tasks, Epics, Features, Bugs), Iterations, Boards, and ADO Queries as sync filters. | Integration | EA, SA, Dev, PM | Must | **Differentiator #3 counterpart for Microsoft-stack enterprises (a large segment of the EA tool market)** |
| REQ-ENT-INT-005 | ADO Pipeline Trigger on Architecture Change | The platform SHALL support triggering an Azure DevOps pipeline (or GitHub Actions workflow) when a specified ESIP artifact reaches a defined lifecycle state (e.g., "Solution Design Approved" triggers a downstream ADO release pipeline); the integration SHALL use ADO service hooks or webhook. | Integration | EA, SA, DevOps | Should | Closes the loop between architecture approval gates and delivery automation |
| REQ-ENT-INT-006 | Delivery Traceability Dashboard | The platform SHALL provide a cross-system traceability dashboard showing: ESIP requirement → ESIP artifact → Jira/ADO issue → sprint → delivery status; this SHALL be filterable by initiative, team, and time period. | Reporting | EA, PM, Buyer | Must | This is the proof that architecture decisions flow into delivery; the "architecture is not shelfware" evidence |

### 4.2 ServiceNow & Confluence Integration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-INT-007 | ServiceNow CMDB Sync | The platform SHALL provide a sync connector to ServiceNow CMDB that: imports CI records as ESIP technology/application components; exports ESIP application portfolio records back to CMDB; maintains a bidirectional identity link; and triggers re-sync on CMDB change events via ServiceNow event management. | Integration | EA, IT | Must | ServiceNow is the system of record for IT assets in most large enterprises; ESIP needs to read from and write to it |
| REQ-ENT-INT-008 | ServiceNow Change Advisory Board (CAB) Integration | The platform SHALL allow promoting an ESIP Architecture Decision Record (ADR) or Solution Design to a ServiceNow Change Request, pre-populating change request fields from the ESIP artifact; the approval status in ServiceNow SHALL flow back to ESIP. | Integration | EA, SA, IT | Should | Closes architecture → change governance loop in ITIL-mature enterprises |
| REQ-ENT-INT-009 | Confluence Page Publish & Sync | The platform SHALL allow publishing any ESIP deliverable (architecture document, diagram, roadmap) as a Confluence page (Cloud or Data Center) under a configurable space/page hierarchy; subsequent updates in ESIP SHALL trigger an update to the Confluence page with a diff-based change note. | Integration | EA, SA, All | Must | Confluence is the enterprise documentation standard; architects need to meet the audience where they are |
| REQ-ENT-INT-010 | Confluence Ingest to Knowledge Hub | The platform SHALL allow ingesting Confluence spaces and pages into the ESIP Knowledge Hub (ref V08) to build context for AI queries and architecture analysis; ingest SHALL be incremental (pull only changed pages since last sync). | Integration | EA, SA | Must | Boundary: V08 owns ingestion mechanics; this requirement is for the ENT-INT surface (admin-visible connector tile, usage reporting) |

### 4.3 Slack & Microsoft Teams App

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-INT-011 | Slack App — Architecture Query | The platform SHALL provide a Slack app that allows any workspace member with an ESIP account to query the architecture from Slack using natural language (e.g., `/esip ask what are the dependencies of the payments service?`) and receive an AI-generated, cited response in the thread. | Integration | All | Must | Architecture knowledge is buried in ESIP; surfacing it in Slack drives daily active use and democratises access |
| REQ-ENT-INT-012 | Slack App — Notification & Approval Actions | The ESIP Slack app SHALL deliver proactive notifications into configurable Slack channels for: artifact state changes, review requests, approval gates, and AI-detected risks; notification messages SHALL include inline action buttons (Approve / Request Changes / View in ESIP) that call back to ESIP without leaving Slack. | Integration | EA, SA, All | Must | Approval friction is a top adoption blocker; meeting architects in Slack removes the "go check ESIP" step |
| REQ-ENT-INT-013 | Microsoft Teams App — Architecture Query | The platform SHALL provide a Microsoft Teams app equivalent in scope to REQ-ENT-INT-011, installable from the Teams App Store; the Teams app SHALL use adaptive cards for richer in-chat rendering of architecture diagrams and traceability tables. | Integration | All | Must | Microsoft Teams is the dominant collaboration platform in enterprise (Fortune 500); Teams app is a procurement expectation |
| REQ-ENT-INT-014 | Microsoft Teams App — Meeting Capture | The Teams app SHALL integrate with Teams meeting transcripts (Teams Premium or Graph API) to auto-capture architecture decisions discussed in meetings and surface them as ESIP draft ADRs for review; ref real-time capture in V17. | Integration | EA, SA | Should | Boundary: V17 owns meeting-capture agent mechanics; this requirement is the Teams-app surface |
| REQ-ENT-INT-015 | Chat App Authenticated Access | Both Slack and Teams apps SHALL authenticate users via OAuth against the ESIP SSO provider; all queries and actions SHALL be scoped to the user's ESIP RBAC permissions, so that a user cannot access artifacts they cannot see in the ESIP UI. | Security | Admin, Security | Must | Slack/Teams must not bypass ESIP access controls |

### 4.4 Browser Extension & Embeds

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-INT-016 | Browser Extension — Architecture Capture | The platform SHALL provide a browser extension (Chrome, Edge, Firefox) that allows architects to: highlight text or images on any web page and send them to the ESIP Knowledge Hub as a captured source; annotate the capture with tags, persona, and framework context; and link the capture to an existing ESIP project. | Integration | EA, SA, All | Should | Captures ad-hoc research (vendor docs, industry reports, competitor sites) that would otherwise be lost |
| REQ-ENT-INT-017 | Browser Extension — Inline Architecture Query | The browser extension SHALL allow the user to invoke an ESIP AI query panel from any webpage, sending the page content as additional context to the query; responses SHALL cite both the current page and the user's ESIP knowledge base. | AI/ML | EA, SA | Could | Enriches research workflows; differentiates from point-in-time searches |
| REQ-ENT-INT-018 | Embeddable Diagram & View Widget | The platform SHALL provide an embeddable read-only diagram/view widget (iFrame + JavaScript SDK) that renders live ESIP diagrams on external portals (intranet, Confluence, SharePoint, internal developer portals); the widget SHALL authenticate via a scoped embed token and respect the source artifact's access controls. | Integration | EA, Admin | Should | Brings architecture visibility into portals where stakeholders already work |
| REQ-ENT-INT-019 | Public API, Webhooks & SDK | All ESIP data and actions SHALL be accessible via a versioned REST/GraphQL public API; webhooks SHALL fire on configurable events (artifact created/updated/approved, review requested, etc.); a TypeScript/JavaScript client SDK SHALL be published; ref V11 for API platform details. | Integration | IT, Dev | Must | Boundary: V11 owns API infra; this requirement is the GTM/enterprise-buyer-facing surface (API docs, SDK, webhook docs in developer portal) |

---

## 5. ENT-ADOPT — Adoption & Enablement

> **Design principle:** An architecture tool is only valuable if architects actually use it and produce better outputs. ESIP SHALL lower the expertise barrier through guided playbooks, in-product coaching, and a learning ecosystem that keeps users engaged long-term.

### 5.1 Guided Playbooks Per Persona & Framework ★ D2

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-ADOPT-001 | **★ D2 — Persona + Framework Playbooks** | The platform SHALL provide structured, step-by-step interactive playbooks for each major persona × framework combination (e.g., "Enterprise Architect — TOGAF ADM Phase A to B", "Solution Architect — C4 HLD", "Data Architect — DAMA Conceptual Model"); each playbook SHALL guide the user through the recommended sequence of activities, point to the right ESIP features, and surface AI assistance at each step without requiring prior framework expertise. | UX | EA, SA, All | Must | **Differentiator #2 — most enterprise architects are not TOGAF experts; playbooks collapse the expertise gap that makes competitors unusable without training; this is the adoption flywheel** |
| REQ-ENT-ADOPT-002 | Playbook Progress Tracking | The platform SHALL track each user's progress through active playbooks, persist progress across sessions, and display a completion percentage and next-step prompt on the user's dashboard; incomplete playbooks SHALL surface in the user's activity feed. | UX | All | Must | Prevents "I don't know what to do next" abandonment |
| REQ-ENT-ADOPT-003 | Playbook Branching & Context-Awareness | Playbooks SHALL support branching logic so that the path adapts to the user's context (e.g., "Does your organisation use ArchiMate? Yes → branch to ArchiMate-flavoured steps; No → branch to generic notation steps"); branching logic SHALL be configurable by tenant admins for custom playbooks. | UX | All | Should | One-size-fits-all playbooks fail diverse enterprise contexts; branching ensures relevance |
| REQ-ENT-ADOPT-004 | Custom Playbook Authoring | Tenant admins and certified SI partners SHALL be able to create custom playbooks using a no-code playbook builder (drag-and-drop step sequencer, branch editor, AI-assist toggle per step, resource link editor); custom playbooks SHALL be shareable within the tenant and optionally publishable to the marketplace. | Functional | Admin, CS | Should | Enterprises have internal methodology variants; ISPs want to package their IP as playbooks |
| REQ-ENT-ADOPT-005 | Starter Kits & Sample Projects | The platform SHALL provide downloadable starter kits (pre-populated sample projects) for each major persona × framework, including: realistic synthetic architecture artifacts, pre-linked diagrams, sample roadmaps, and pre-configured AI prompts; starter kits SHALL be importable with one click from the onboarding screen. | UX | All | Must | Reduces "blank canvas paralysis"; gives new users a working model to deconstruct and adapt |
| REQ-ENT-ADOPT-006 | Reusable Architecture Patterns Library | The platform SHALL include a curated architecture patterns library (cloud reference architectures, microservices patterns, data platform patterns, security patterns, integration patterns) searchable and importable into any project; patterns SHALL be tagged by framework, persona, technology, and industry. | Functional | SA, EA, All | Must | Pattern libraries are a top-requested feature in architecture tool surveys; reduces repetitive work |

### 5.2 In-App Tours & Coachmarks

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-ADOPT-007 | Onboarding Tours | The platform SHALL provide contextual onboarding tours (modal walkthroughs with coachmarks) triggered on first use of each major feature area; tours SHALL be persona-specific (an EA's first-use tour covers different features than an AppA's); users SHALL be able to replay any tour from Help > Tours. | UX | All | Must | First-session activation rate is the strongest predictor of 90-day retention; tours increase activation |
| REQ-ENT-ADOPT-008 | Contextual Coachmarks & Tooltips | Complex UI areas (diagramming canvas, framework configuration, AI panel, playbook builder) SHALL have contextual coachmarks (hover tooltips, "?" icons, and step-indicator overlays) that explain purpose, keyboard shortcuts, and AI assistance availability; coachmarks SHALL be suppressible once dismissed. | UX | All | Must | Reduces support tickets; increases feature discovery |
| REQ-ENT-ADOPT-009 | In-App What's New & Changelog | The platform SHALL surface a "What's New" panel (accessible from the top navigation) highlighting the latest feature releases with brief explanations, demo GIFs, and links to documentation; the panel SHALL be dismissible and SHALL track which users have seen each announcement. | UX | All | Should | Feature adoption rate increases when users are proactively informed; reduces "I didn't know that existed" churn |

### 5.3 Academy, Certification & Community ★ D5

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-ADOPT-010 | **★ D5 — ESIP Academy** | The platform SHALL operate an ESIP Academy (accessible at academy.esip.io or an equivalent domain) providing self-paced learning paths per persona covering: ESIP product proficiency, architecture framework fundamentals (TOGAF, ArchiMate, C4, DoDAF), and AI-assisted architecture techniques; course content SHALL include video lessons, interactive labs using ESIP sandbox environments, quizzes, and assessments. | UX | All | Must | **Differentiator #5 — creates an ecosystem moat; users who complete Academy courses have 3-5× higher retention rates than non-Academy users (industry benchmark); competitors with thin training content lose to those with structured academies** |
| REQ-ENT-ADOPT-011 | **★ D5 — ESIP Certification Program** | The platform SHALL offer proctored certification exams for: "ESIP Certified Architect" (product proficiency), "ESIP Certified Enterprise Architect" (TOGAF + ESIP), and "ESIP Certified Partner" (SI/consulting firm qualification); certificates SHALL be verifiable via a public credential registry (Credly or equivalent) and shareable on LinkedIn. | UX | All | Must | **Differentiator #5 — certifications drive demand-side pull (architects put them on resumes) and supply-side stickiness (orgs that have certified staff are highly unlikely to switch tools)** |
| REQ-ENT-ADOPT-012 | **★ D5 — Community Forum & User Groups** | The platform SHALL host a community forum (community.esip.io) with sections for: Q&A per persona/framework, template/playbook sharing, feature requests and voting, announcements, and regional/industry user groups; active community members SHALL earn reputation points and badges visible in the product; the community SHALL be moderated by vendor community managers. | UX | All | Must | **Differentiator #5 — vibrant communities are the most cost-effective growth engine (Atlassian, Salesforce, Figma all attribute 30%+ of growth to community); community is a structural moat competitors cannot copy** |
| REQ-ENT-ADOPT-013 | Academy Integration with Product | Users who complete Academy learning paths SHALL automatically unlock in-product recognition (badges, profile certifications, unlocked advanced features); the platform SHALL surface Academy course recommendations contextually when a user first attempts a feature they have not been trained on. | UX | All | Should | Reinforces Academy investment with product-level recognition; closes the loop between training and product |
| REQ-ENT-ADOPT-014 | Partner Enablement Learning Path | The Academy SHALL include a dedicated SI/consulting partner learning path covering: platform administration, connector configuration, customer onboarding, playbook customisation, and migration from competitor tools; completion SHALL qualify the partner for "ESIP Certified Partner" status. | UX | CS | Must | Scales partner-delivered implementations; trained partners reduce vendor PS costs |

### 5.4 Customer Success & Health Scoring

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-ADOPT-015 | Customer Health Score | The platform SHALL compute and display a per-tenant Customer Health Score (0–100) combining: monthly active user rate, feature adoption breadth, artifact creation velocity, playbook completion rate, Academy enrollment rate, and support ticket volume; the score SHALL be visible to the customer's admin and to the vendor's CS team. | Reporting | Admin, CS | Must | Health scores are the leading indicator for churn; CS teams need objective data to prioritise interventions |
| REQ-ENT-ADOPT-016 | In-App Customer Success Prompts | The platform SHALL surface proactive in-app prompts when health indicators drop (e.g., "Your team hasn't published a new architecture artifact in 30 days — here are 3 quick wins to get back on track") linked to relevant Academy courses, playbooks, or CS contact options. | UX | Admin, All | Should | Reduces time-to-churn-signal detection; proactive nudges extend engagement |
| REQ-ENT-ADOPT-017 | Change Management Toolkit | The platform SHALL provide a downloadable change management toolkit for enterprise rollouts, including: stakeholder communication templates, adoption metrics dashboard template, architect manager guide, executive briefing deck, and a 90-day onboarding plan template; the toolkit SHALL be available from the admin Help & Resources section. | UX | Admin, CS | Should | Large enterprises struggle to drive internal tool adoption; a vendor-provided change-management pack accelerates rollout |
| REQ-ENT-ADOPT-018 | Dedicated CSM for Enterprise Tier | Enterprise-tier tenants SHALL be assigned a named Customer Success Manager (CSM) as part of the support agreement; the CSM SHALL be accessible directly from within the product via a "Contact my CSM" button in the admin help panel. | UX | Buyer, Admin | Must | Dedicated CSM is the #1 factor enterprises cite for renewal in CS benchmarks; it signals long-term partnership |

### 5.5 Reconciliation Additions — In-App Support & Help Desk (GAP G-3)

> These requirements close **GAP G-3** from the FINALIZATION-REPORT (in-app support/help surface: contextual help, ticketing, live chat, escalation). They complement the enablement content above (playbooks, tours, academy) with a reactive support surface for when a user is stuck. Prefix continues the ENT-ADOPT sequence.

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-ADOPT-019 | Contextual In-App Help & Help Center | The platform SHALL provide an always-available in-app Help panel that surfaces **context-aware** help articles, short how-to videos, and relevant playbook/academy links based on the user's current screen and persona; the panel SHALL include a searchable knowledge base (help center) and SHALL deep-link from coachmarks/tooltips (REQ-ENT-ADOPT-008) into the relevant article. In air-gapped deployments the help center SHALL be bundled and served locally. | UX | All | Must | GAP G-3; self-serve contextual help deflects the majority of support volume and is a baseline enterprise SaaS expectation; air-gapped users still need it offline |
| REQ-ENT-ADOPT-020 | In-App Support Ticketing | The platform SHALL allow any user to raise a support ticket from within the product (Help > Contact Support) capturing: category, description, severity, the current screen/artifact context, and optional diagnostic attachments (with explicit consent); tickets SHALL be tracked with a status the user can view in-app, and SHALL integrate with the vendor's helpdesk system (e.g., Zendesk/Jira Service Management). For on-prem/air-gapped tenants, ticket submission SHALL fall back to the email path per REQ-ENT-ECO-016. | Functional | All, Admin | Must | GAP G-3; integrated ticketing with captured context resolves issues faster and is required for the SLA response commitments in REQ-ENT-ECO-013 |
| REQ-ENT-ADOPT-021 | Live Chat / In-App Messaging Support | The platform SHALL provide an in-app live-chat support widget (vendor-staffed during published support hours, with an AI assistant for off-hours triage that can answer from the help center and create a ticket when it cannot) available to Professional and Enterprise support tiers; chat availability and tier eligibility SHALL be configurable and clearly indicated. Live chat is **SaaS-only** (requires vendor connectivity); air-gapped tenants use ticketing/email fallback. | Functional | All | Should | GAP G-3; live chat materially improves activation and satisfaction for higher support tiers; AI off-hours triage scales coverage |
| REQ-ENT-ADOPT-022 | Support Escalation & SLA Linkage | The platform SHALL implement a defined support escalation path: ticket severity drives routing and response/resolution targets aligned to the tenant's contracted SLA tier (REQ-ENT-ECO-013); a user or admin SHALL be able to escalate an open ticket, and severity-1 tickets SHALL auto-notify the assigned CSM (REQ-ENT-ADOPT-018) and trigger the incident-communication protocol where a platform incident is implicated (REQ-ENT-ECO-015). | Governance | Admin, CS | Must | GAP G-3; escalation tied to SLA tiers is what makes the published SLA real rather than aspirational; closes the loop with CSM and incident comms |

---

## 6. ENT-LOCK — Openness / No Lock-In

> **Design principle:** "Lock-in" is a trust-killer in enterprise sales. ESIP SHALL make openness a positive selling point — not a reluctant concession. Architecture-as-code, open formats, and data portability SHALL be first-class features, not afterthoughts.

### 6.1 Architecture-as-Code ★ D4

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-LOCK-001 | **★ D4 — Architecture-as-Code (AaC) Format** | The platform SHALL provide a human-readable, text-based Architecture-as-Code (AaC) format (YAML or structured Markdown with a published schema) for all ESIP artifacts, diagrams, models, relationships, and configurations; the AaC format SHALL be the canonical source of truth for storage, enabling all artifacts to be stored in any Git repository. | Data | EA, SA, All | Must | **Differentiator #4 — architecture that lives in a Git repo is versioned, diffable, reviewable via PR, and fully portable; no competitor has made AaC the primary storage model; this is a "leave anytime" guarantee built into the architecture** |
| REQ-ENT-LOCK-002 | **★ D4 — Git-Native Storage Backend** | The platform SHALL support Git (GitHub, GitLab, Bitbucket, Azure Repos, Gitea for air-gapped) as a first-class storage backend where all project artifacts are committed to a Git repository chosen by the tenant; ESIP acts as an editor over Git, not a proprietary data silo. | Data | IT, EA, Admin | Must | **Differentiator #4 — engineers trust Git; GitOps-native storage removes the "data hostage" objection that blocks adoption in engineering-led organisations** |
| REQ-ENT-LOCK-003 | Git Commit, Branch & PR Workflow | Every artifact save SHALL result in a Git commit (with author, timestamp, and change message); artifact versions SHALL correspond to Git commits; branch-and-merge workflows SHALL be supported so an architect can work on a feature branch and raise a PR for peer review before merging to main. | Data | EA, SA, Dev | Must | Enables familiar "code review for architecture" workflows; integrates with existing CI/CD pipelines |
| REQ-ENT-LOCK-004 | Architecture Diff in PR | Pull requests on architecture branches SHALL display a visual diff of changed artifacts: added/removed/modified fields, diagram element changes, relationship changes; the diff SHALL be human-readable in the Git host's PR UI (as structured text) and rendered as a visual diff inside ESIP. | UX | EA, SA, Dev | Must | Without meaningful diffs, architecture PRs are unreviewed; this is the feature that makes AaC reviewable |
| REQ-ENT-LOCK-005 | Architecture Schema Published as Open Standard | The AaC YAML/Markdown schema SHALL be published under an open license (CC BY 4.0 or Apache 2.0) at a stable public URL with versioning; third-party tools SHALL be able to read and write ESIP artifacts without a commercial license. | Governance | IT, Buyer | Should | Open schema prevents "walled garden" perception; enables community tooling; analyst-friendly |

### 6.2 Export & Data Portability

> **Reconciliation note (cross-ref — export formats):** The diagram- and model-export *formats* referenced in this section (SVG/PNG/PDF/draw.io XML/Visio VDX·VSDX/PlantUML/Mermaid in REQ-ENT-LOCK-007; ArchiMate Exchange/C4 DSL/JSON-LD in REQ-ENT-LOCK-008) are **owned and implemented by the diagram/deliverable export engines — V03 (diagram canvas export) and V10 (deliverable/document export)**. ENT-LOCK requires only that these exports are *exposed as a no-lock-in, no-premium-gate guarantee* at the enterprise/GTM surface; it does not re-specify the renderers. The reciprocal round-trip *import* of the same formats is V18 (see V18 import-ownership note).

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-LOCK-006 | Full Tenant Data Export | Tenant admins SHALL be able to export all tenant data at any time (not only at off-boarding) in a structured, documented format: artifacts as AaC YAML, diagrams as SVG + AaC, documents as Word/PDF/Markdown, relationships as a JSON graph, user and permission data as CSV; the export SHALL complete within 24 hours for tenants up to 100,000 artifacts. | Data | Admin, Buyer | Must | GDPR Art. 20 (data portability); "leave anytime" guarantee; removes procurement objection |
| REQ-ENT-LOCK-007 | Diagram Export in Open Formats | Every diagram created in ESIP SHALL be exportable individually as: SVG, PNG, PDF, draw.io XML, Visio VDX/VSDX, and PlantUML/Mermaid (where diagram type permits); export SHALL be available without any premium tier gate. | Data | All | Must | Diagrams must be sharable with stakeholders who do not have ESIP accounts |
| REQ-ENT-LOCK-008 | Architecture Model Export | Full architecture models (ArchiMate, C4, Zachman matrices, DoDAF viewpoint sets) SHALL be exportable in their standard exchange formats: ArchiMate Exchange Format (XML), C4 DSL, JSON-LD for Zachman/DoDAF; published schemas SHALL accompany exports. | Data | EA, SA | Must | Interoperability with other architecture tools (Sparx, BiZZdesign, Archi); M&A data transfer scenarios |
| REQ-ENT-LOCK-009 | Scheduled Automatic Export | Admins SHALL be able to configure a scheduled automatic export (daily/weekly/monthly) to a customer-controlled destination: Git repository, S3-compatible storage, Azure Blob, or SFTP; this acts as a continuous off-site backup independent of the vendor. | Data | Admin, IT | Should | Eliminates "what if ESIP goes down or shuts down?" objection; enterprise business continuity |
| REQ-ENT-LOCK-010 | "Leave Anytime" Guarantee | The platform's Terms of Service SHALL include a documented "Leave Anytime" guarantee stating: (a) data export is available at any time without restriction, (b) data is deleted within 30 days of off-boarding upon request, (c) all exported data is in open, documented formats, (d) no contractual lock-in exceeds 12 months for any tier; the guarantee SHALL be prominently displayed on the pricing page. | Governance | Buyer | Must | Procurement committees flag long-term lock-in risk as a veto; a vendor-stated guarantee removes this objection |

### 6.3 Open API & Documented Schemas

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-LOCK-011 | Public OpenAPI Specification | The full ESIP REST API SHALL be described by a published, versioned OpenAPI 3.1 specification available at a stable public URL; the spec SHALL be importable into Postman, Insomnia, and other API clients; all endpoints used by the ESIP UI SHALL be available in the public spec (no hidden internal API). | Integration | Dev, IT | Must | "Secret internal API" is a lock-in red flag; open spec enables integration, automation, and ecosystem tooling |
| REQ-ENT-LOCK-012 | Documented Storage Schemas | All database schemas and storage formats used by ESIP (relational tables, document structures, vector index formats, event log structures) SHALL be documented in the public developer documentation; on-prem customers SHALL be able to run direct SQL/API queries against their own database. | Data | IT | Should | On-prem operators need to build their own backup, reporting, and integration pipelines |
| REQ-ENT-LOCK-013 | Self-Hostable Local Tools | The platform SHALL publish Docker images for all ESIP services on a public container registry (Docker Hub, GitHub Container Registry, or equivalent) tagged by version; on-prem operators SHALL be able to run ESIP from these images without a separate installer. | NFR | IT | Must | Standard enterprise expectation for on-prem software; simplifies air-gapped deployment |

---

## 7. ENT-ECO — Ecosystem & Go-to-Market

> **Design principle:** ESIP's long-term market position depends not just on product features but on the ecosystem around it — partners who implement it, customers who champion it, and a brand that signals trustworthiness and openness.

### 7.1 Partner & SI Program

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-ECO-001 | Partner Portal | The platform SHALL operate a partner portal (partners.esip.io) providing SIs, resellers, and ISVs with: NFR agreements, deal registration, co-marketing materials, technical documentation, sandbox access, and community forums; the portal SHALL be accessible to registered partners without sales-team gating. | Functional | CS | Must | Partners close 40–60% of enterprise deals in architecture tools (Gartner data); self-serve partner portal scales the partner program without headcount |
| REQ-ENT-ECO-002 | Partner Tier Structure | The platform SHALL implement a tiered partner program (e.g., Registered / Silver / Gold / Platinum) with defined criteria (certified staff count, revenue threshold, customer references) and associated benefits (deal registration protection, margin %, MDF, CSM support, roadmap preview); tier status SHALL be visible in the partner portal. | Governance | CS | Should | Tiers incentivize partner investment; Platinum-level protections prevent channel conflict |
| REQ-ENT-ECO-003 | Partner-Led Deployment & Configuration | Partners SHALL be able to provision, configure, and hand over ESIP tenants on behalf of their customers using a delegated admin model; partner admins SHALL have scoped access that allows tenant setup without accessing tenant data. | Functional | CS, Admin | Must | SI-delivered implementations require delegated provisioning; without it, partners cannot operate at scale |
| REQ-ENT-ECO-004 | SI Implementation Accelerator Kit | The platform SHALL provide an SI implementation accelerator kit: discovery questionnaire templates, architecture assessment templates, workshop facilitation guides, migration-from-competitor runbooks, and a project plan template for 4-week, 8-week, and 12-week enterprise deployments. | Functional | CS | Should | Reduces SI time-to-first-value; standardises delivery quality; increases partner NPS |

### 7.2 White-Label / OEM / Embeddable

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-ECO-005 | White-Label / OEM Mode | The platform SHALL support a white-label/OEM deployment mode where a commercial partner (ISV, consulting firm, or enterprise customer) can: replace all ESIP branding (logo, product name, colour scheme, domain, email templates, PDF footers) with their own brand via admin configuration; the vendor brand SHALL be removable entirely in OEM mode. | Functional | Admin, CS | Must | OEM/embed is a separate revenue stream; ISPs selling "powered by ESIP" under their own brand represent a high-margin distribution channel |
| REQ-ENT-ECO-006 | Embeddable Module Mode | The platform SHALL support an embeddable module mode where specific ESIP capabilities (e.g., the diagram editor, the AI architecture generator, the requirements extractor) can be embedded within a third-party application using an SDK + iFrame + event API; the embedded module SHALL authenticate via OAuth and respect the host application's user session. | Integration | Dev, CS | Could | Enables ISVs to embed ESIP capabilities into their own products (e.g., a GRC tool embedding ESIP's architecture mapping) |
| REQ-ENT-ECO-007 | OEM Usage Reporting | In OEM deployments, the platform SHALL provide the OEM partner with aggregated usage analytics (active users, artifact counts, feature adoption) accessible via a partner-scoped API; individual end-customer data SHALL not be exposed to the OEM partner beyond their own tenants. | Reporting | CS | Should | OEM partners need usage data for their own product decisions and customer QBRs |

### 7.3 Internationalisation & Accessibility

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-ECO-008 | Internationalisation (i18n) Framework | The platform SHALL implement a complete internationalisation (i18n) framework with all UI strings, validation messages, email templates, and in-app notifications stored in locale resource files; the framework SHALL support right-to-left (RTL) languages (Arabic, Hebrew) and double-byte character sets (Japanese, Chinese, Korean). | UX | All | Must | Required for EU, APAC, and Middle East enterprise sales; i18n must be baked in — retrofitting is expensive |
| REQ-ENT-ECO-009 | Localisation (l10n) — Tier 1 Languages | At GA, the platform SHALL ship localised UIs in: English (US/UK), French, German, Spanish, Portuguese (BR), Japanese; subsequent releases SHALL add: Dutch, Italian, Polish, Korean, Simplified Chinese, Traditional Chinese; all diagram text and AI-generated content SHALL be generation-language-aware (generate in the user's UI language). | UX | All | Should | European and APAC enterprise deals stall when the product is English-only; French/German are hard gates in public sector |
| REQ-ENT-ECO-010 | WCAG 2.1 AA Accessibility | The platform's web UI SHALL conform to WCAG 2.1 Level AA across all pages and interactive components; conformance SHALL be validated with automated testing (axe-core or equivalent) on every build and with periodic manual testing by users with disabilities; a VPAT (Voluntary Product Accessibility Template) SHALL be published and kept current. | UX | All | Must | Section 508 (US federal), EN 301 549 (EU public procurement), Accessibility Canada Act; WCAG AA is a hard gate in government and education procurement |
| REQ-ENT-ECO-011 | Section 508 Conformance Statement | The platform SHALL publish a Section 508 conformance statement (ACR) covering all WCAG 2.1 AA criteria, available from the trust center; the statement SHALL be updated with each major release. | Governance | Buyer, Security | Must | US federal procurement gate; education sector requirement |
| REQ-ENT-ECO-012 | Keyboard Navigation & Screen Reader Support | All interactive UI elements (modals, diagrams, tables, forms, menus, AI panels) SHALL be fully operable via keyboard without a mouse; ARIA attributes SHALL be applied to all custom components; the diagramming canvas SHALL provide a keyboard-accessible alternative representation (list view of elements and relationships). | UX | All | Must | WCAG 2.1 SC 2.1.1 (Keyboard) and SC 4.1.2 (Name, Role, Value); essential for visually impaired architects |

### 7.4 SLAs, Support Tiers & Status Page

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-ECO-013 | Published SLA Tiers | The platform SHALL offer at least three support/SLA tiers: Standard (community forum + email, 99.5% uptime SLA, 2-business-day response), Professional (email + chat, 99.9% uptime SLA, 4-hour critical response), Enterprise (dedicated CSM, 99.95% uptime SLA, 1-hour critical response, 24×7 severity-1 coverage); SLA terms SHALL be publicly documented. | Governance | Buyer | Must | SLA tiers are a procurement requirement; without published uptime commitments, enterprise deals do not close |
| REQ-ENT-ECO-014 | Status Page | The platform SHALL operate a public status page (status.esip.io) showing real-time and historical uptime for all platform components, scheduled maintenance windows, incident history with post-mortem links, and an RSS/Atom feed; uptime metrics SHALL be computed from independent third-party monitoring. | NFR | IT, Buyer | Must | Status pages are expected by DevOps and IT teams; independence from vendor monitoring prevents "all green during an outage" trust failures |
| REQ-ENT-ECO-015 | Incident Communication Protocol | For severity-1 incidents (full outage or data integrity risk), the platform SHALL notify all affected enterprise tenant admins via email within 15 minutes of incident declaration and provide updates at no longer than 30-minute intervals until resolution; a post-mortem SHALL be published within 5 business days. | Governance | IT, Admin | Must | Incident communication SLAs are increasingly part of enterprise DPAs and MSAs |
| REQ-ENT-ECO-016 | On-Premises Support Path | For on-prem and air-gapped deployments, the platform SHALL provide a supported update delivery mechanism that does not require internet access: signed release bundles downloadable from a vendor portal (before going air-gapped) or deliverable on physical media; support tickets SHALL be submittable via email in air-gapped environments. | NFR | IT | Must | Air-gapped operators cannot use a cloud-native support ticketing system; this is a table-stakes requirement for defense/gov deployments |

### 7.5 Reference Customer & GTM Programs

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-ECO-017 | Reference Customer Program | The platform SHALL operate a formal reference customer program where design partners and early enterprise customers are invited to participate in: beta previews, advisory board sessions, co-authored case studies, joint press releases, and conference speaking slots; reference customers SHALL receive dedicated CSM access and early-access feature flags. | Governance | Buyer, CS | Must | Reference customers are the #1 sales tool in enterprise B2B; buyers want to speak to peers, not sales |
| REQ-ENT-ECO-018 | Case Study & ROI Proof Point Library | The platform SHALL maintain a public case study library (minimum 3 case studies at GA, growing to 20+ within 12 months) segmented by industry, company size, and persona; each case study SHALL include: customer challenge, ESIP solution, quantified outcomes (time saved, tools consolidated, cost avoided), and a customer quote. | Reporting | Buyer | Must | Case studies are accessed by 73% of enterprise buyers before a purchase decision (Gartner) |
| REQ-ENT-ECO-019 | Integration & ISV Marketplace | The platform SHALL operate an integration marketplace (ref V16 for competitive intelligence on marketplace content) where third-party connectors, playbooks, templates, and AI prompt packs are published by partners; each listing SHALL display: publisher name, rating, install count, compatible tiers, and support contact. | Functional | Admin, All | Should | Marketplace creates network effects; ISVs building on ESIP increase switching cost and expand addressable use cases |

### 7.6 Reconciliation Additions — Partner / Reseller Billing (GAP G-4)

> These requirements close **GAP G-4** from the FINALIZATION-REPORT (partner/reseller billing: deal registration, margin/discount tiers, OEM consolidated billing, commission). The partner *program* shell already exists (deal registration REQ-ENT-ECO-001; tier structure with margin %/MDF REQ-ENT-ECO-002; OEM mode REQ-ENT-ECO-005/-007); these additions specify the **billing mechanics** that sit on top of it. **Boundary:** invoicing, metering, tax, and payment *enforcement internals* → V13; these requirements own the partner-facing billing surface and the data the channel needs. Prefix continues the ENT-ECO sequence.

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|----|-------|-------------------|----------|------------|----------|-----------------|
| REQ-ENT-ECO-020 | Deal Registration → Billing Linkage | The platform SHALL link a partner's registered deal (REQ-ENT-ECO-001) to the resulting customer subscription so that, on close, the registered partner is durably recorded as the deal owner of record; deal-registration approval SHALL gate the partner discount and protection window, and the linkage SHALL be visible in both the partner portal and the V13 billing record. | Functional | CS, Admin | Must | GAP G-4; without a registration→subscription link the channel cannot reconcile who earns the margin, causing channel-conflict disputes |
| REQ-ENT-ECO-021 | Margin & Discount Tier Application | The platform SHALL apply partner margin/discount automatically based on the partner's program tier (REQ-ENT-ECO-002) and the registered deal: reseller transactions SHALL compute partner cost = list price × (1 − tier discount %), and the partner-facing quote/order SHALL show list price, tier discount, partner cost, and suggested end-customer price; discount schedules SHALL be configurable per tier and per region and enforced through V13 billing. | Functional | CS, Admin | Must | GAP G-4; margin tiers must be applied consistently and transparently or partners cannot quote reliably; manual discounting does not scale |
| REQ-ENT-ECO-022 | OEM / Reseller Consolidated Billing | The platform SHALL support **consolidated billing** where a single OEM partner or reseller is invoiced for multiple downstream end-customer tenants on one consolidated statement (with per-tenant line-item breakdown for usage, seats, and overages), enabling the partner — not the end customer — to be the billing party of record; consolidated-billing relationships SHALL be configurable per partner and reconcilable against per-tenant metering. (Complements OEM usage reporting REQ-ENT-ECO-007.) | Functional | CS, Admin | Must | GAP G-4; OEM and reseller channels require the partner to own the customer billing relationship; per-tenant-only invoicing blocks the entire indirect motion |
| REQ-ENT-ECO-023 | Partner Commission & Payout Tracking | The platform SHALL track partner commissions/referral fees per closed and renewed deal — computing commission from the program tier, deal type (resell vs. referral vs. influence), and recognized revenue — and SHALL expose a partner-facing earnings statement (accrued, pending, paid) plus an export for the vendor's finance/payout process; commission rules SHALL be configurable per tier and auditable. | Reporting | CS, Admin | Should | GAP G-4; visible, predictable commission/payout tracking is what sustains partner investment; opacity here is a top partner-program complaint |

---

## Appendix A — Requirements Summary

| Sub-Area | Code | Count |
|---|---|---|
| Deployment Flexibility | ENT-DEP | 15 |
| Trust, Security & AI Governance | ENT-TRU | 28 |
| Value & Procurement | ENT-VAL | 13 |
| Delivery & Ecosystem Integration | ENT-INT | 19 |
| Adoption & Enablement | ENT-ADOPT | 18 |
| Openness / No Lock-In | ENT-LOCK | 13 |
| Ecosystem & Go-to-Market | ENT-ECO | 19 |
| **Total** | **ENT** | **125** |

---

## Appendix B — Flagged Differentiators Cross-Reference

| Differentiator | Req IDs | Why it wins deals |
|---|---|---|
| D1 — Air-gapped + local LLM | REQ-ENT-DEP-005, -011 | Unlocks defense, classified gov, central banking — segments competitors cannot serve without full rearchitecting |
| D2 — Guided playbooks per persona/framework | REQ-ENT-ADOPT-001–004 | Eliminates the expertise barrier; makes TOGAF/ArchiMate accessible to architects who are not framework-certified |
| D3 — Two-way Jira & ADO sync | REQ-ENT-INT-001–006 | Keeps architecture connected to delivery; without it, ESIP is a modeling tool that gets updated once and abandoned |
| D4 — Architecture-as-code (git-native) | REQ-ENT-LOCK-001–005 | Removes the "data hostage" objection; engineers adopt tools that behave like code; GitOps-native architecture is a structural differentiator |
| D5 — Academy + certification + community | REQ-ENT-ADOPT-010–013 | Creates demand-side pull (certifications on resumes), supply-side stickiness (certified orgs don't churn), and an ecosystem moat (communities take years to build) |

---

## Appendix C — Boundary Notes

| Concern | Owner Volume | Boundary |
|---|---|---|
| AI guardrails, model provider config, AI eval pipelines | V06 | ENT-TRU-023 references V06; this volume owns only the trust-center-visible surface |
| Connector ingestion mechanics (Jira ingest, Confluence ingest) | V08 | ENT-INT covers the Jira/ADO/Confluence sync surface and GTM-visible admin UX; V08 owns the ingestion pipeline internals |
| Multi-tenancy implementation, RBAC/ABAC, SCIM, SSO wiring, API infra | V11 | ENT-LOCK covers the open-API developer-portal surface; V11 owns infrastructure |
| Licensing editions, billing metering, feature gating | V13 | ENT-VAL references V13 for flexible licensing; V13 owns enforcement rules |
| Marketplace content, competitive migration intelligence | V16 | ENT-ECO-019 defines the marketplace surface; V16 owns intelligence and competitive migration |
| Import mechanics (bulk data import, legacy migration) | V18 | ENT-LOCK POC-to-production carry-over and data export are here; V18 owns bulk import pipelines |

---

*Volume 19 — Enterprise Adoption, Trust & Go-to-Market Differentiators. Prefix: ENT. 125 requirements across 7 capability areas. Authored 2026-06-23.*
