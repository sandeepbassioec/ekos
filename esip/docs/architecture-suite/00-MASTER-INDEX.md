# ESIP Architecture Automation Suite — Master Requirements Catalog

**Status:** Living document — V0 (index + taxonomy) authored 2026-06-23. Detail volumes generated in parallel (see §8 Volume Map).
**Vision owner:** Sandeep Bassi. **Compiled by:** Claude.

> **Note on scale:** This catalog targets *exhaustive, deduplicated coverage* — every capability area, for every persona and framework — as thousands of atomic, testable requirements organized by a strict taxonomy. That is deliberately more useful than a raw "lakh" of near-duplicate rows. Completeness is measured by *coverage of the taxonomy*, not row count.

---

## 1. Product Vision

ESIP becomes an **enterprise-grade, multi-persona Architecture Automation Suite** — the system of record and the "co-architect" for every architecture role in a software organization. It ingests the messy reality of a project (documents, meetings, emails, chats, spreadsheets, decks, source code, IT landscape data), uses AI to **extract the core business requirements and everything else needed**, and **auto-generates the architecture deliverables** — models, diagrams, documents, roadmaps, portfolios — for whichever framework and persona the user works in.

It is built so that **no architect role is generalized away**: Enterprise and Solution Architects get their full, distinct journeys; Application/Integration/API/CRM/Data/Security/Cloud and "any other" architects get theirs; and the platform is **extensible** so a new architecture role can be configured, not coded.

**Positioning / "nobody can afford to lose it":** a single suite that does what LeanIX + Ardoq + BiZZdesign + MEGA HOPEX + Sparx EA + Avolution + erwin + ServiceNow APM + Jira/Confluence-for-architecture do — but unified, AI-native, and requirements-driven end to end.

---

## 2. Personas & Journeys

Three pillars (Requirements · Roadmaps · Knowledge Hub) are **universal**, but each persona's journey, artifacts, and deliverables differ.

| # | Persona | Primary frameworks/standards | Signature outputs |
|---|---------|------------------------------|-------------------|
| P1 | **Enterprise Architect** | TOGAF (ADM), Zachman, DoDAF, MODAF, UAF, FEAF | Capability maps, business/data/app/tech architectures, roadmaps, portfolios, transition states |
| P2 | **Solution Architect** | TOGAF (solution), ArchiMate, C4, ADRs | Solution designs, options/trade-offs, HLD/LLD, NFRs, sizing/cost, reference architectures |
| P3 | **Application Architect** *(current ESIP)* | UML, C4, ArchiMate (app layer) | App designs, component/sequence/class diagrams, API/data contracts, WBS/estimates |
| P4 | **Integration Architect** | EIP, ArchiMate, BPMN | Integration patterns, message flows, ESB/iPaaS designs, canonical models |
| P5 | **API Architect** | OpenAPI, AsyncAPI, REST/GraphQL/gRPC | API catalogs, contracts, governance, gateway/policy designs |
| P6 | **CRM Architect** | Salesforce/Dynamics/etc. blueprints | CRM domain models, object/flow designs, integration maps |
| P7 | **Data Architect** | DAMA-DMBOK, ER, data mesh/lakehouse | Conceptual/logical/physical data models, lineage, governance |
| P8 | **Security Architect** | TOGAF/SABSA, STRIDE, NIST, Zero-Trust | Threat models, control catalogs, security architectures |
| P9 | **Cloud Architect** | AWS/Azure/GCP Well-Architected | Landing zones, cloud topologies, cost/governance designs |
| P10 | **Infrastructure / Network Architect** | ITIL, network refs | Network/infra topologies, capacity, HA/DR |
| P11 | **Microservices / Platform / DevOps Architect** | C4, DDD, 12-factor | Service maps, platform blueprints, CI/CD/observability designs |
| P13 | **Product Manager** | BRD/PRD, RICE/Kano/WSJF, product backlog | BRDs/PRDs, product roadmap, release notes; rolls each roadmap item → BRD, hands off to architects (Vol 17) |
| P14 | **Presales / Bid / Proposal Architect** | Shipley, APMP, PCR 2015, FAR/DFARS, ITT/RFP practice | Bid/no-bid decision, compliance matrix, proposal document, answer bank, pricing model, win/loss analytics (Vol 22) |
| P12 | **+ Any Architect (extensible)** | Configurable | Configurable artifacts/diagrams/deliverables via the metamodel |

> **Access tiers (not personas):** **Viewer** (free, read-only) and **Reviewer** (free, comment/review after login) are licensing tiers, detailed in Vols 13 & 17.

> **Journey divergence:** EA (P1) and SA (P2) journeys are substantially broader (portfolio/landscape/multi-year transformation; options/trade-off/decision) than the role-based architects (P3–P11), which are more design- and delivery-centric. The platform must model both without forcing one into the other.

---

## 3. The Three Pillars (per persona)

- **Requirements pillar** — multi-format ingestion → AI extraction of business/functional/non-functional/constraint requirements → classification, dedup, conflict detection, traceability (RTM), baselining, sign-off. (Volume 07)
- **Knowledge Hub pillar** — the evidence base: documents, meetings, emails, chats + enterprise connectors (Jira, Confluence, SharePoint, ADO, ServiceNow, CMDB); AI search/Q&A/summarization over it. (Volume 08)
- **Roadmaps pillar** — multi-year, multi-type roadmaps; transition architectures; tied to portfolio, capabilities, and migration/sunset plans. (Volume 09)

These feed the **modeling/diagramming engine** (Volume 03), the **AI engine** (Volume 06), and the **deliverable generators** (Volume 10).

---

## 4. Framework Coverage Matrix

| Framework | Scope in suite | Volume |
|-----------|----------------|--------|
| **TOGAF 10 / ADM** | Full ADM (Preliminary → H + Requirements Mgmt), ACF, Enterprise Continuum, Architecture Repository, content metamodel | 01 |
| **Zachman** | 6 perspectives × 6 interrogatives (36 cells), each with artifacts | 02 |
| **DoDAF 2.02** | All viewpoints (AV, CV, OV, SV, SvcV, StdV, DIV, PV) | 02 |
| **MODAF / NAF** | Viewpoints (StV, OV, SV, AcV, SOV, TV) | 02 |
| **UAF** | Unified Architecture Framework grid | 02 |
| **ArchiMate 3.2** | Full metamodel: Strategy, Business, Application, Technology, Physical, Motivation, Implementation & Migration; relationships; viewpoints | 03 |
| **FEAF / Gartner / SABSA / DAMA / C4 / UML / BPMN** | Reference + role-specific support | 02, 03, 05, 07 |

---

## 5. Cross-cutting Capability Domains

- **AI engine** (Volume 06): extraction, classification, gap/confusion/threat/NFR analysis (existing), AI diagram generation, AI architecture generation per framework, code analysis, portfolio analysis, per-persona copilots, agentic workflows, RAG, guardrails/eval, prompt library, traceability AI.
- **Modeling & Diagramming engine** (Volume 03): ArchiMate + UML, BPMN, C4, ERD, DFD, sequence, network, infra, cloud, mind-map, flowchart; canvas, auto-layout, validation, round-trip, versioned views, export.
- **Ingestion & Extraction** (Volume 07): Word/PDF/Excel/PPT/Visio/images/email/chat/issue-tracker → structured content.
- **Code-scanning, Portfolio & Sunset/Migration** (Volume 09): repo scan → application portfolio (APM) → application/business/data flow derivation → tech-debt, TIME model, 6 R's migration, decommission/sunset planning.
- **Deliverables & Export** (Volume 10): per-framework documents (Word/PPT/Excel/PDF), diagram export (PNG/SVG/Visio/drawio), portals/publishing.
- **Platform, NFRs, Governance, Metamodel** (Volume 11): multi-tenancy, RBAC/ABAC, SSO, audit, the **extensible metamodel** (frameworks/artifact-types/diagram-types as configuration), collaboration, APIs, compliance, scale.

---

## 6. Requirement Taxonomy & Conventions

Every detail volume follows this schema so the catalog is uniform and machine-importable.

**Requirement record fields:**
- **ID** — `REQ-<PREFIX>-NNN` (zero-padded; sub-areas may use `REQ-<PREFIX><Sub>-NNN`).
- **Title** — short imperative.
- **Description** — a single SHALL-statement (testable, atomic).
- **Category** — `Functional | NFR | Integration | Data | AI/ML | UX | Security | Governance | Reporting`.
- **Persona(s)** — which of P1–P13 it serves (or "All"). *(P13 Product Manager was added after P1–P12; numbering is not sequential. Where a volume still says "P1–P12", read "P1–P13" if P13 applies.)*
- **Priority** — MoSCoW (`Must | Should | Could | Won't-now`).
- **Rationale/Source** — why; standard/competitor reference if applicable.
- **Acceptance Criteria** *(where useful)* and **Dependencies** *(where useful)*.

**Volume ID prefixes:** EA (01), FWK (02), MOD (03), SA (04), ROL (05), AI (06), REQM (07), KH (08), PF (09), DLV (10), PLT (11), WFL (12), ONB (13), PERS (14), REV (15), INT (16), PM·LIC·AGT·RTC·SRCH (17), IMP·CV·MOB (18), ENT (19), FRO (20). A single volume MAY register multiple sub-area prefixes (e.g. 17, 18).

**Catalog-wide rulings (from `FINALIZATION-REPORT.md` — apply across all volumes):**
1. **Priority ≠ Phase.** `Must` means "required in the product," NOT "in the MVP." Build sequence is governed by `BUILD-PLAN-MVP-AND-PHASING.md`, not by MoSCoW. (A `Phase` tag will be added in the reconciliation pass.)
2. **AI & customer data.** "No training on customer data" (REQ-ENT-TRU-010) applies to customer **content** (artifacts/documents/diagrams/queries). Model improvement uses only **anonymized, PII-scrubbed, per-tenant opt-in behavioral signals**; all fine-tuning is consent-gated. Overrides conflicting wording in V06/V14/V15.
3. **Air-gapped mode** excludes internet-dependent features (real-time meeting connectors, marketplace, live regulatory feeds, cross-tenant benchmarking) — SaaS-only with documented offline fallbacks.
4. **Provider names** in requirements (Bedrock/OpenAI/AWS/Azure/…) are **illustrative, never exclusive** — the suite stays provider-pluggable and cloud-neutral.

---

## 7. Build Phasing (suggested)

1. **Foundations** — extensible metamodel, modeling/diagramming engine, ingestion+extraction, AI engine core. (Vols 03, 06, 07, 11)
2. **Application + Solution Architect** — extend current ESIP (Vols 04, 05) — fastest path to value on today's base.
3. **Enterprise Architect** — frameworks, portfolio, code-scan/sunset, deliverables. (Vols 01, 02, 09, 10)
4. **Connectors + role-based architects + governance at scale.** (Vols 05, 08, 11)

---

## 8. Volume Map

| Vol | File | Scope | Prefix |
|-----|------|-------|--------|
| 00 | `00-MASTER-INDEX.md` | This index, taxonomy, conventions | — |
| 01 | `01-enterprise-architecture-togaf.md` | Enterprise Architecture + TOGAF ADM, capability/value-stream, repository | EA |
| 02 | `02-frameworks-zachman-dodaf-modaf-uaf.md` | Zachman, DoDAF, MODAF/NAF, UAF, FEAF; framework interop | FWK |
| 03 | `03-archimate-modeling-diagramming.md` | ArchiMate 3.2 metamodel + full diagramming/modeling engine | MOD |
| 04 | `04-solution-architecture.md` | Solution Architect journey, options/ADR/HLD/LLD/NFR/sizing | SA |
| 05 | `05-role-based-architects.md` | Application/Integration/API/CRM/Data/Security/Cloud/Infra/Microservices + extensibility | ROL |
| 06 | `06-ai-capabilities.md` | Full AI engine: extraction→generation→analysis→copilots→agents | AI |
| 07 | `07-requirements-pillar.md` | Multi-format ingestion + extraction + RTM/traceability/quality | REQM |
| 08 | `08-knowledge-hub-connectors.md` | Knowledge Hub + enterprise connectors + search/Q&A | KH |
| 09 | `09-roadmap-portfolio-codescan-sunset.md` | Roadmaps, portfolio (APM), code-scan, flow derivation, sunset/migration | PF |
| 10 | `10-deliverables-generation.md` | Per-framework document + diagram generation/export/publishing | DLV |
| 11 | `11-platform-nfr-governance-metamodel.md` | Platform, NFRs, RBAC, audit, extensible metamodel, collaboration, APIs | PLT |
| 12 | `12-cross-role-collaboration-workflow.md` | Cross-role requests (clarification/missing-req/approval/hand-off), approval gates, workflow engine, inbox, SLAs | WFL |
| 13 | `13-client-onboarding-licensing-tenant-admin.md` | Client onboarding flow, editions, per-persona licensing, licensing rules, delegated admin, user management, invite/activation, admin config zone (DB, cloud, AI keys, Git targets, connectors, branding, security policy, data residency), billing, audit | ONB |
| 14 | `14-personalization-adaptive-intelligence.md` | Per-user personalization & adaptive intelligence: first-run/working-mode wizard, preference model, adaptive dashboards, behavioral pattern mining, repetition→automation, gap-based guidance, next-best-action, personal AI memory, peer intelligence, transparency/ethics | PERS |
| 15 | `15-review-collaboration-designer.md` | Native PR-style review designer (docs + maxGraph models): inline comments/threads/resolve, lifecycle/diff, AI-apply, AI-augmented review (security/NFR/functional topic-nav + suggested diagrams), free Reviewer license, cross-team scoped dependency sharing, audit | REV |
| 16 | `16-differentiating-intelligence-analytics.md` | Adaptive/predictive/prescriptive analysis, architecture drift/conformance, impact/blast-radius, auto-traceability, compliance/audit-readiness, FinOps, digital twin + what-if, knowledge graph + NL query, agentic autopilots, live docs, benchmarking, repo→portfolio, marketplace | INT |
| 17 | `17-pm-agents-connectors-search.md` | Product Manager persona (BRD/PRD), free Viewer/Reviewer license tiers, Rust scanner **agent provisioning** (ADO-style), real-time **Teams/Outlook/Zoom** auto-capture, system-wide **global search** | PM/LIC/AGT/RTC/SRCH |
| 18 | `18-interoperability-import-multimodal.md` | Comprehensive import (draw.io/Visio/Sparx EA/ArchiMate-Exchange/Mermaid/PlantUML/Lucidchart/XMI + LeanIX/Ardoq/MEGA/erwin), **computer-vision** whiteboard-photo→editable model, **mobile whiteboarding app** (capture→straight into workspace, offline sync, real-time collab) | IMP/CV/MOB |
| 19 | `19-enterprise-adoption-trust-gtm.md` | Enterprise deal-closers: deployment flexibility (SaaS/VPC/on-prem/**air-gapped+local LLM**), trust/security/AI governance (SOC2/ISO27001/**ISO42001**/GDPR/HIPAA/FedRAMP-path, BYOK, no-training guarantee, CAIQ pack), ROI/TCO calculator + business-case generator + POC program, **two-way Jira & ADO sync** + Slack/Teams app + browser extension, **guided playbooks per persona/framework** + in-app tours + **academy/certification/community**, **architecture-as-code (git-native)** + full data portability + open formats, partner/SI program + **white-label/OEM** + i18n/l10n + WCAG 2.1 AA + SLAs/status page + reference-customer program | ENT |
| 20 | `20-frontier-emerging-capabilities.md` | Frontier: sustainability/Green-IT carbon analysis, M&A architecture due-diligence, regulatory-change→impact, voice-driven architecture, AI pre-review board, immersive AR/VR/3D, evolutionary fitness functions, generative design-space exploration, resilience/chaos simulation | FRO |
| 21 | `21-ai-role-expertise-tradeoffs.md` | Role-conditioned AI expertise (AI drafts/suggests as a seasoned [persona] — governed, versioned prompt packs, admin→project→user override) + Trade-off Analysis engine (AI question-driven elicitation, quality-attribute matrices/ATAM, weighted scoring + recommendation, what-if, ADR linkage, library) | AIX/TRD |
| 22 | `22-presales-bid-proposal-architect.md` | Presales/Bid/Proposal Architect (P14): bid/no-bid decision, opportunity workspace + pipeline, RFP/RFI/RFQ/RFR intake + AI parsing, response & compliance matrix, proposal authoring (full section suite), win-theme engine, answer bank (CV/references/certifications), pricing & commercials inputs, clarification Q&A, compliance & eligibility, colour-team reviews + deal-desk governance, win/loss capture + bid analytics, deliverables export (Word/PDF/PPT/Excel/ZIP bundle), AI copilot + Red Team AI review | BID |
| 23 | `23-template-presentation-designer.md` | Template & Presentation Designer: WYSIWYG designer w/ placeholders/merge-fields/conditional sections, admin→project→user override hierarchy, templates governing the whole draft→create→review→present→deliver lifecycle (not just output), adaptive/prescriptive suggestions (user config wins), brand & audience-mode governance, template versioning/marketplace | TPL |
| 24 | `24-enterprise-identity-access.md` | **Enterprise Identity & Access (Pluggable SSO / IdP) — authoritative deep IAM spec:** `IIdentityProvider` port/adapter model (Firebase → any IdP, hot-swap); 14 named IdP adapters (Okta/Entra ID/Azure AD B2C/Auth0/Ping/OneLogin/Google Workspace/ADFS/Keycloak/Generic SAML/OIDC/LDAP-AD/Firebase/BYOI); SAML 2.0 (SP metadata, cert rotation, SLO); OIDC/OAuth 2.0 (PKCE, JWKS, internal JWT); Okta OIN + Entra app gallery tiles; self-service SSO wizard + claims debugger; SCIM 2.0 (provisioning/deprovisioning/group→role/JIT/license-reclaim); multi-IdP + home-realm discovery; B2B partner + B2C Reviewer flows; MFA (TOTP/FIDO2/WebAuthn/passkeys/push/step-up); sessions (idle/absolute timeout, concurrent, SAML/OIDC SLO front+back-channel, SLO propagation); impersonation (audited, read-only) + break-glass; RBAC+ABAC (role/permission catalogue, persona/license entitlements, classification labels, per-artifact/field permissions, delegated admin); API auth (OAuth2 client-credentials, hashed API keys, service accounts, mTLS for scanner agents); Firebase→enterprise migration wizard + overlap window + fallback; identity event log + anomaly detection + SIEM export | IAM |
| 25 | `25-security-compliance-trust.md` | **Security, Compliance & Trust Architecture — authoritative control catalog:** server-side enforcement mandate (client-embedded Firebase logic → .NET API migration plan); API & application security (OWASP API Top 10/ASVS L2 baseline — BOLA/IDOR, SSRF, XXE, template injection, CSRF, XSS, security headers, rate limiting, brute-force protection); encryption in transit (TLS 1.2+/1.3, mTLS) + at rest (AES-256, TDE, field-level, 3-tier key hierarchy) + BYOK + rotation; secrets vault (no secrets in code/container/bundle, vault, rotation, AI-proxy key isolation); **document upload AV/malware scanning (pluggable: ClamAV / ICAP / MS Defender / BYO — quarantine, CDR, file-type allowlisting, archive-bomb protection)**; **configurable storage targets (S3/Azure Blob/GCS/MinIO/local disk/SMB/NFS/BYO-bucket, per-tenant, IFileStorage port)**; tenant isolation (logical/physical, cross-tenant pen-test, context propagation); data residency + data classification (5 levels, auto-classify AI, export labeling) + DLP engine + retention/legal-hold/DSAR/right-to-erasure; secure SDLC (SAST/DAST/SCA/secret-scan/IaC-scan in CI, SBOM CycloneDX/SPDX, threat modeling STRIDE, release security gate, patch SLAs, annual pen test, bug bounty, red team); tamper-evident audit log (append-only, WORM/hash-chain, structured schema, 1-year online + 6-year archive); SIEM export (Splunk/Sentinel/Chronicle/syslog); built-in anomaly detection; backup/restore/DR (RTO ≤ 4h, RPO ≤ 1h); IRP (classification, playbooks, 72-hour breach notification, tabletop exercises); AI-specific security (prompt injection / indirect RAG injection / jailbreak defense, cross-tenant RAG isolation, output data-exfil detection, no-training-on-customer-content architectural enforcement, HITL for high-impact agentic actions); compliance control catalog (SOC 2 CC/A/C/P, ISO 27001:2022 Annex A, ISO 42001:2023, GDPR, HIPAA, FedRAMP/NIST 800-53, CCPA, DORA); CAIQ/SIG auto-population; posture dashboard; change management + vendor security review | SEC |
| 26 | `26-globalization-localization-theming.md` | Globalization, Localization & Theming: full UI i18n (ICU MessageFormat, CLDR plural/gender, **RTL/bidi** incl. canvas, locale date/number/currency/timezone/collation), content **localization** (AI-translate deliverables/proposals/diagram-labels, translation memory TMX, per-locale glossary TBX, **framework-terminology localization** TOGAF/Zachman/etc.), **AI output in the user's language** (localized prompt-packs, multilingual ingestion + cross-lingual search), g11n (multi-currency, regional content/compliance, calendar systems), **theming & white-label** (light/dark/high-contrast/dyslexia themes, tenant brand themes w/ OKLCH tokens, per-user theme, theme designer, theming across generated outputs) | I18N/THM |

---

*Detail volumes are generated by domain. Counts and completion notes are appended to §9 as each lands.*

## 9. Generation Log

**Status: 26 detail volumes, ≈4,845 atomic requirements, 14 personas (2026-06-23). Original 20 volumes are V-FINAL & RECONCILED (12-agent workflow, all PASS). v3+v4 additions — Vol 21 (AI role-expertise + trade-offs), 22 (Presales/Bid Architect P14), 23 (Template Designer), 24 (deep Enterprise Identity/SSO — supersedes V11 §3 IAM), 25 (Security/Compliance control catalog — supersedes V11 §4), 26 (i18n/l10n/g11n + theming) — are mini-reconciled via the consolidated boundary rulings in §9 below. **Catalog LOCKED & build-ready → GO for Phase 1.** See `FINALIZATION-REPORT.md`.**

**Governing principles (apply to all build work):** (1) **Reuse** the existing ESIP codebase — extend, don't rip-and-replace. (2) **UI consistency** — every feature uses the existing Radix + Tailwind v4 design system (`src/app/components/ui/`). (3) **Cross-role workflow** (Vol 12) is foundational, not an add-on. (4) **Stack** is decided in `TECH-STACK.md`. See also `BUILD-PLAN-MVP-AND-PHASING.md`.

| Vol | Prefix | Reqs | Scope summary |
|-----|--------|------|---------------|
| 01 | EA | 254 | TOGAF 10: Preliminary + ADM A–H + Requirements Mgmt, ADM techniques, ACF metamodel, Enterprise Continuum, Repository, Capability Framework, per-phase deliverables |
| 02 | FWK | 214 | Zachman 36 cells, DoDAF 2.02 (all viewpoints/models), MODAF/NAF v4, UAF, FEAF; framework selection + interop mapping |
| 03 | MOD | 276 | ArchiMate 3.2 full metamodel + relationships + viewpoints; diagramming engine (UML/BPMN/C4/ERD/DFD/cloud/etc.), auto-layout, validation, import/export, co-edit |
| 04 | SA | 183 | Solution Architect lifecycle: intake→options→trade-off/ADR→HLD/LLD→NFR/sizing/cost→vendor select→RAID→governance/handover |
| 05 | ROL | 210 | Application/Integration/API/CRM/Data/Security/Cloud/Infra/Microservices (+Mobile/AI-ML) + "any architect" extensibility |
| 06 | AI | 273 | AI engine: extraction/quality, diagram gen, architecture gen, code analysis, deliverable drafting, copilots, agents, RAG, provider mgmt, guardrails/eval, prompts, cost, traceability |
| 07 | REQM | 270 | Multi-format ingestion + extraction handling + requirement model/quality/RTM/baselining/change/reporting/tool-sync |
| 08 | KH | 207 | Knowledge Hub + 23 enterprise connectors, storage/lifecycle, AI search/Q&A, provenance/governance, ops |
| 09 | PF | 228 | Roadmaps, APM (TIME/6R), code-scanning (AST/SBOM/SAST), flow derivation, landscape mapping, sunset/decommission, migration planning |
| 10 | DLV | 360 | Per-framework deliverable templates, doc engines (docx/pptx/xlsx/pdf/md/html), template mgmt, diagram export, portal/publishing, reporting, quality/a11y |
| 11 | PLT | 224 | Extensible metamodel, multi-tenancy, IAM (SSO/SCIM/RBAC/ABAC), security/compliance, collaboration, versioning, integration platform/APIs, admin/ops, platform NFRs |
| 12 | WFL | 95 | Cross-role collaboration & workflow engine: typed requests (clarification/missing-req/review/approval/decision/change/hand-off/escalation), state machines, approval gates before rollout, work inbox, routing, SLAs/escalation, no-code builder, audit/analytics, pillar integration |
| 13 | ONB | 241 | Client onboarding (self-serve + assisted, wizard, org profile, DB/cloud choice, import, terms, provisioning), editions & feature gating, per-persona license types (EA/SA/AppA/IntA/APIA/CRMA/DA/SecA/CloudA/InfraA/MSA + custom, named + concurrent), 31 licensing-enforcement rules, delegated admin roles (User/Billing/Config/Security Admin), user lifecycle, SCIM 2.0, invite/activation, admin config zone (DB provider+BYOD, cloud+BYOC, BYOK AI keys, Git targets, connector org-auth, branding/SMTP, security policy, data residency/retention), billing/metering, license compliance reporting, admin audit log |
| 14 | PERS | 106 | Personalization & adaptive intelligence: first-run/working-mode wizard, preference model, adaptive dashboards, behavioral telemetry & sequential-pattern mining, repetition→one-click personal automation, "what they lack" guided growth, next-best-action suggestions, per-user AI memory, process-mode engine, peer intelligence (k-anon/differential-privacy), feedback loop, transparency/control/ethics |
| 15 | REV | 67 | Native review & collaboration designer: PR-style inline comments/threads/resolve on docs + maxGraph models, lifecycle/diff, AI-apply feedback, AI-augmented review (auto-surface security/NFR/functional + topic navigator + suggested diagrams), free Reviewer license, cross-team scoped dependency sharing, immutable audit |
| 16 | INT | 178 | Differentiating intelligence: adaptive/predictive/prescriptive analysis, architecture conformance & drift, impact/blast-radius, auto-traceability, compliance/audit-readiness, FinOps, enterprise digital twin + what-if, knowledge graph + NL query, anomaly detection, agentic autopilots, live auto-docs, estimation intelligence, portfolio intelligence, benchmarking, exec narratives, time-travel, repo→portfolio in 15 min, competitive migration + marketplace |
| 17 | PM/LIC/AGT/RTC/SRCH | 170 | Product Manager persona (P13) + BRD/PRD lifecycle; free Viewer/Reviewer + PM license tiers; Rust code-scanner agent provisioning (ADO-build-agent style, air-gapped, admin data-boundary); real-time Teams/Outlook/Zoom auto-capture connectors; system-wide hybrid semantic+keyword permission-aware global search |
| 18 | IMP/CV/MOB | 71 | Interoperability, import & multimodal: comprehensive import from draw.io/Visio(.vsdx)/Sparx EA(XMI/.qea)/ArchiMate-Exchange/Mermaid/PlantUML/Lucidchart/Miro/BPMN/XMI + competitor EA tools, import wizard w/ metamodel mapping & round-trip; computer-vision whiteboard/sketch photo→editable diagram (shape+arrow detect, handwriting OCR, review-before-commit); mobile app (React Native/PWA) linked to workspace — capture→straight into system, offline+CRDT sync, real-time collab whiteboard, voice capture, on-the-go review |
| 19 | ENT | 125 | Enterprise deal-closers: 7 deployment topologies incl. air-gapped+local LLM (D1); SOC2/ISO27001/ISO42001/GDPR/HIPAA/FedRAMP posture; BYOK; no-training-on-customer-data guarantee; CAIQ/SIG pre-fill; ROI/TCO calculator; business-case generator; POC sandbox; two-way Jira+ADO sync (D3); Slack+Teams app; browser extension; guided playbooks per persona/framework (D2); academy+certification+community (D5); architecture-as-code git-native (D4); full data portability; open API; white-label/OEM; i18n/WCAG 2.1 AA; SLA tiers; reference-customer program |
| 20 | FRO | 89 | Frontier/emerging: sustainability & Green-IT (carbon-aware design, cost×carbon×perf Pareto), M&A architecture due-diligence + PMI roadmap, regulatory-change horizon-scan→impact simulation, voice-driven/conversational architecture, AI Pre-Review Board (autonomous reviewer-personas before humans), immersive AR/VR/3D, evolutionary architecture fitness functions + CI gates, generative multi-candidate design-space exploration, resilience/chaos game-day simulation |
| 21 | AIX/TRD | 156 | Role-conditioned AI expertise (per-persona expert prompt packs — AI drafts/critiques as a seasoned EA/SA/PM/Presales/etc.; governed, versioned, admin→project→user override; proactive role nudges; eval harness) + Trade-off Analysis engine (AI question-driven elicitation; quality-attribute matrices + ATAM sensitivity/trade-off points; weighted scoring + recommendation + confidence; what-if on weights; ADR linkage; trade-off library; proactive decision-point detection) |
| 22 | BID | ≈181 | Presales/Bid/Proposal Architect (P14): bid/no-bid scorecard + qualification, opportunity workspace + pipeline, RFP/RFI/RFQ/RFR multi-doc intake + AI parsing (requirements/questions/deadlines/compliance items), response & compliance matrix (auto-generated, gap-flagged), full proposal authoring (executive summary, technical, delivery, team/CVs, references, assumptions, compliance statement), win-theme library + heat-map, answer bank (semantic search, AI synthesis, CV library, references, certifications), pricing & commercials model (BOM, rate cards, margin, multi-scenario, deal-desk gate), customer clarification Q&A management, compliance & eligibility pre-check + scorecard, colour-team reviews (Pink/Red/Gold) + deal-desk + legal + final sign-off gates, win/loss capture + bid analytics (pipeline KPIs, cycle time, answer bank analytics, win driver analysis), branded deliverables (Word/PDF/PPT/Excel/ZIP bundle + portal upload), AI copilot + Red Team AI + win probability + proposal strength score, bid workspace data security |
| 23 | TPL | 163 | Template & Presentation Designer / content lifecycle: WYSIWYG designer (placeholders/merge-fields, conditional sections, repeaters, styling, live preview, validation); template types across the lifecycle (deliverables, proposals, ADR/trade-off, BRD/PRD, diagrams, dashboards, decks, review checklists, emails, drafting-prompt templates, delivery packaging); admin→project→user override hierarchy + effective-template resolution; template-as-lifecycle-binding (draft→create→review→present→deliver); placeholder/merge engine; brand & audience-mode governance (exec/technical/board/customer-external); adaptive/prescriptive suggestions (config always wins, explainable); template versioning/approval/marketplace |
| 24 | IAM | ≈225 | Enterprise Identity & Access: IIdentityProvider port/adapter (14 named adapters incl. Okta/Entra/Keycloak/Firebase/BYOI); SAML 2.0 + OIDC/OAuth2; Okta OIN + Entra app gallery tiles; self-service wizard + claims debugger + cert dashboard; SCIM 2.0 (group→role/JIT/license-reclaim); multi-IdP + home-realm discovery; B2B/B2C; TOTP/FIDO2/WebAuthn/passkeys/step-up; sessions/SLO (SAML+OIDC front+back-channel); impersonation + break-glass; RBAC+ABAC (permission catalogue, persona/license entitlements, classification labels); OAuth2 CC + API keys + mTLS scanner agents; Firebase migration wizard; identity audit + anomaly detection + SIEM |
| 25 | SEC | 141 + 62 ctrl | Security, Compliance & Trust: server-side enforcement mandate (Firebase→API migration plan); OWASP API Top 10 / ASVS (BOLA, SSRF, XXE, template injection, rate limiting); TLS 1.2+/1.3 + mTLS; AES-256 at rest (TDE + field-level); 3-tier key hierarchy + BYOK + rotation; secrets vault; pluggable AV scanning (ClamAV/ICAP/Defender/BYO) + quarantine + CDR; configurable storage targets (S3/Blob/GCS/MinIO/disk/SMB/NFS/BYO-bucket); tenant isolation (logical/physical); data residency + classification + DLP + retention/legal-hold/DSAR/erasure; secure SDLC (SAST/DAST/SCA/secret-scan/IaC/SBOM/threat-model/release-gate/pen-test/bug-bounty); tamper-evident audit log + SIEM export + anomaly detection; backup/DR (RTO ≤ 4h/RPO ≤ 1h); IRP + breach notification + exercises; AI security (prompt injection, indirect RAG, jailbreak, no-training enforcement, HITL for agentic actions); control catalog mapping SOC2/ISO27001/ISO42001/GDPR/HIPAA/FedRAMP/CCPA/DORA; CAIQ/SIG auto-population; posture dashboard |
| 26 | I18N/THM | 109 | Globalization, Localization & Theming: UI i18n (ICU/CLDR, RTL/bidi incl. canvas, locale formats/timezone/collation), content localization (AI translation of deliverables/proposals/diagram labels, translation memory, per-locale glossary, framework-terminology localization), AI output in user's language (localized prompt-packs, multilingual ingestion + cross-lingual search), g11n (multi-currency, regional content/compliance, calendars), theming & white-label (light/dark/high-contrast/dyslexia themes, tenant brand tokens via OKLCH, per-user theme, theme designer, consistent theming across generated outputs) |
| **Total** | — | **≈4,845** | 26 volumes · 14 personas · ~475 capability areas. Original 20 V-FINAL & RECONCILED; v3+v4 additions — Vol 21 (AIX/TRD), 22 (BID/P14), 23 (TPL), 24 (IAM deep-identity), 25 (SEC control-catalog), 26 (I18N/THM) — pending a focused mini-reconciliation. |

**Reconciliation (2026-06-23):** all `FINALIZATION-REPORT.md` punch-list items applied across the volumes via a file-partitioned workflow (12 agents, edit→verify, all PASS, no content removed): overlap/boundary notes + 2 merges (diagram-import → V18; license-tier definitions → V17), 5 gaps added (IP/indemnity, canvas accessibility, AI-pipeline SLOs, in-app support, partner billing), contradictions C1–C7 resolved in-place (no-train scope, air-gap exclusions, auto-apply↔approval-gate, anonymous-viewer, P1–P13, peer-intel tenant scope), and Phase banners on Vols 01/03/06/11/13/16/19.

**v3/v4 Reconciliation Rulings (2026-06-23) — canonical ownership for Vols 21–26 (per-volume cross-ref notes already written by the generating agents; these rulings are authoritative):**
- **Identity:** V24 owns all IdP/SSO/SCIM/session/auth — V11 §3 IAM is now a cross-ref.
- **Security/Compliance:** V25 owns the security control catalog + server-side-enforcement mandate + AV scanning + configurable storage — V11 §4 and V16/V19 posture claims cross-ref it; V08/V18 uploads use V25 for AV + `IFileStorage`.
- **AI role-expertise:** V21 owns the per-persona expert prompt-pack *spec & governance*; V06 owns model/prompt-library infra; V14 owns per-user adaptation.
- **Trade-offs:** V21 owns the trade-off engine; V04 owns ADRs (trade-off → ADR linkage).
- **Presales (V22):** owns bid/proposal; reuses V04 (solution/sizing), V10 (docs), V23 (templates), V21 (trade-offs), V12/V15 (workflow/review) — no re-implementation.
- **Templates & presentation:** V23 owns template design + content-lifecycle governance; V10 owns the file-rendering engines.
- **Theming & i18n:** V26 owns app-level UI theming/white-label tokens + i18n/l10n/g11n; V23 owns brand governance *inside deliverables*; V06 = translation infra, V08 = multilingual ingestion, V13 = multi-currency (cross-refs). Theming is consistent across app **and** generated outputs.
- **Personas:** P13 Product Manager, P14 Presales/Bid Architect are first-class; "All / P1–P14" includes them.

**Suggested follow-ups:** (a) cross-volume dedup/reconciliation pass; (b) MoSCoW-driven MVP slice + phased build plan; (c) export the catalog to a machine-readable format (CSV/JSON) the app itself can import; (d) deepen any volume with an adversarial completeness-critic workflow.
