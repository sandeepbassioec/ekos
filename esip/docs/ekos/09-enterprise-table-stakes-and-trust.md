# EKOS Volume 09 — Enterprise Table-Stakes & Trust

> **Status:** ✅ Done · **Requirement count:** 86 · **Generated:** 2026-06-30 (Opus)
> **Sources mined:** `ConfusionClarity.txt` (15-point table-stakes table + final-spec
> §9), `blueprint §17` (Enterprise Trust & Scale — resiliency, certifications, 100K
> scale, Executive Mission Control, Legacy Connector SDK, Ethical AI), old EA-catalog
> Vols 24 (Enterprise Identity/SSO/SCIM/IdP-gallery), 25 (server-side enforcement +
> pluggable AV + configurable storage), 26 (i18n/l10n/g11n + theming/white-label) via
> [[project-esip]], ESIP Epics G (admin/license) & H (feature flags). Honors
> [00-MASTER-INDEX](00-MASTER-INDEX.md) §2.

---

## 1. Scope & Boundary

**In scope:** the "boring gatekeeper" capabilities without which enterprises reject a
platform regardless of vision — **i18n/l10n/g11n & white-label**, **accessibility
(WCAG 2.2 AA)**, **enterprise identity (SSO/SAML/OIDC + pluggable IdP + SCIM/JIT +
app-gallery)**, **backup/restore/DR (self-service)**, **data residency & tenant
isolation (UX/program)**, **usage analytics & adoption tracking**, **bulk import &
migration**, **offline mode**, **public APIs & webhooks**, **license management & cost
controls**, **performance SLAs & resource governance**, **platform resiliency &
scale**, **certifications**, **server-side enforcement & pluggable infra**, and the
**Executive Mission Control / Legacy Connector SDK / Ethical-AI** trust surfaces.

**Explicitly deferred (owned elsewhere — cross-referenced, NOT duplicated):**
- **Sandbox & staging / governed promotion** → **Vol 05/08**
- **Electronic signatures & 21 CFR Part 11** → **Vol 04 §14**
- **Platform feature flags & gradual rollout** → **Vol 05 §11**
- **Admin audit log** → **Vol 05 §12**
- **Encryption, BYOK/KMS, ABAC, PII** *mechanics* → **Vol 04** (this volume = the
  residency/identity/program layer on top)
- **Tenant-VPC / dedicated-graph-DB / federation** *primitives* → **Vol 01 §15**
- **Accessibility *authoring-time checker*** → **Vol 02 §10** (this volume = the
  platform-wide WCAG program & legal mandate)
- **Theming/branding *mechanism*** → **Vol 08 §7** (this volume = full white-label &
  custom domain)
- **License *economics*, marketplace, payouts** → **Vol 13** (this volume = seat/usage
  *management & controls*)
- **AI explainability *engine*** → **Vol 03**; **AI governance program** → **Vol 05**

---

## 2. Core Principle — Table-Stakes or Rejection

- **EKOS-09-001 (Must, MVP)** — EKOS SHALL satisfy the enterprise "table-stakes" set;
  these are not features but **prerequisites for adoption** — Gartner consistently
  ranks configuration flexibility & operational trust as top selection criteria, and
  ~70% of enterprise software projects fail on adoption. *Source:* ConfusionClarity
  (customization — "customization is not a feature; it's a requirement for adoption");
  blueprint §17 ("the boring gatekeepers").

- **EKOS-09-002 (Must, MVP)** — These capabilities SHALL be surfaced **proactively** in
  the requirements (the AI's lesson: enterprise table-stakes must be raised
  proactively, not only when asked). *Source:* ConfusionClarity(AI's apology — "table
  stakes ... should proactively surface").

---

## 3. Internationalization / Localization / Globalization

- **EKOS-09-003 (Must, P2)** — All UI labels, **ontology type names**, and **AI
  explanations** SHALL be **translatable**; the platform renders per the user's locale
  or browser language. Translations are stored in the graph linked to a `Locale` node.
  *Source:* ConfusionClarity(#1 — i18n); Vol 02 EKOS-02-047.

- **EKOS-09-004 (Should, P2)** — The platform SHALL support **right-to-left (RTL)**
  scripts, locale-specific number/date/currency formatting, and per-locale content
  variants. *Source:* derived from ConfusionClarity(#1 — "global teams German/Japanese/
  French; Quebec/Europe local-language regulators"); old-catalog Vol 26 (g11n).

- **EKOS-09-005 (Should, P2)** — Localization SHALL extend to exported artifacts and
  `.ekr` bundles (the viewer renders in the recipient's locale). *Source:* derived;
  Vol 02/07.

---

## 4. Theming & White-Label

- **EKOS-09-006 (Should, P2)** — Enterprises SHALL **white-label** the platform: custom
  logos, color schemes, **custom domain**, and dedicated branding (building on the
  Vol 08 theming mechanism). *Source:* old-catalog Vol 26 (theming/white-label);
  blueprint §16 (Enterprise — white-label branding, custom domain). *Depends:* Vol 08.

- **EKOS-09-007 (Could, P3)** — White-label SHALL extend to the **EKR Viewer** (branded
  viewer for an enterprise's external recipients). *Source:* derived; Vol 07.

---

## 5. Accessibility (WCAG 2.2 AA)

- **EKOS-09-008 (Must, P2)** — The entire platform UI and **every widget** SHALL meet
  **WCAG 2.2 AA**, emitting ARIA-compliant markup — a **legal requirement** for
  government/EU public-sector adoption. *Source:* ConfusionClarity(#2 — accessibility;
  "non-accessible tool is legally barred"); blueprint §17 (WCAG 2.2 AA).

- **EKOS-09-009 (Must, P2)** — The widget builder's **accessibility checker** (Vol 02
  §10) SHALL gate publishing; this volume mandates the **platform-wide accessibility
  program & conformance reporting**. *Source:* ConfusionClarity(#2); Vol 02 EKOS-02-046.

- **EKOS-09-010 (Should, P2)** — Accessibility (screen-reader, keyboard nav,
  searchability) SHALL NOT be sacrificed for anti-scraping (no vector-path text) —
  protection is policy-driven (Vol 04 §19). *Source:* ChatGPT(anti-scraping
  disagreement); Vol 04 EKOS-04-060.

---

## 6. Enterprise Identity — SSO / SAML / OIDC / SCIM / Pluggable IdP

> No enterprise manually manages users; automated lifecycle from their IdP is
> mandatory. (Old-catalog Vol 24 supersedes generic auth.)

- **EKOS-09-011 (Must, P2)** — The platform SHALL integrate **SSO via SAML and OIDC**
  with a **pluggable Identity Provider** model — Okta, Entra ID (Azure AD), Auth0,
  Azure AD B2C, Keycloak, and **custom IdPs**. *Source:* ConfusionClarity(#3 — SSO/
  SAML/OIDC); old-catalog Vol 24 (pluggable IdP).

- **EKOS-09-012 (Must, P2)** — **SCIM provisioning** SHALL automate user lifecycle
  (create/update/deprovision), updating the graph's `User` nodes and their roles
  automatically; **JIT provisioning** on first SSO login SHALL be supported. *Source:*
  ConfusionClarity(#3 — SCIM); old-catalog Vol 24 (SCIM/JIT).

- **EKOS-09-013 (Should, P2)** — EKOS SHALL appear as an **app-gallery tile** in major
  IdP catalogs (Okta/Entra app gallery) for one-click enterprise enablement. *Source:*
  old-catalog Vol 24 (app-gallery tile).

- **EKOS-09-014 (Must, P2)** — Identity attributes from the IdP SHALL feed the ABAC
  **PIP** (Vol 04 §3) — role, department, clearance, group membership. *Source:*
  ConfusionClarity(#3); Vol 04 EKOS-04-007.

- **EKOS-09-015 (Should, P2)** — The existing ESIP role/license/invite/activation
  scaffolding (Epic G — `licenseTier` viewer/reviewer/contributor/enterprise, status,
  invite-token `/activate/:token`, entitlements helper) SHALL be recognized as the
  **early implementation** and evolved into full IdP-driven identity. *Source:*
  [[project-esip]] Epic G.

---

## 7. Backup, Restore & Disaster Recovery (Self-Service)

- **EKOS-09-016 (Must, P2)** — Even when EKOS hosts, enterprises SHALL be able to
  **self-trigger backup/restore**: one-click snapshot of the knowledge graph and
  **point-in-time restore**. *Source:* ConfusionClarity(#5 — Backup/Restore/DR).

- **EKOS-09-017 (Should, P2)** — Immutable backups SHALL be storable in
  **customer-owned storage** (AWS S3 / Azure Blob / etc.). *Source:* ConfusionClarity
  (#5).

- **EKOS-09-018 (Should, P2)** — A **DR dashboard** widget SHALL show backup status,
  RPO/RTO posture, and restore controls. *Source:* ConfusionClarity(#5 — DR dashboard);
  relates Vol 14 (Resilience/DR widgets).

---

## 8. Data Residency & Tenant Isolation (Program/UX)

- **EKOS-09-019 (Must, P2)** — Deployment SHALL support **per-tenant data residency**
  (e.g., "EU only") and **tenant isolation** (per-tenant VPC, dedicated graph DB
  instance, customer-managed encryption keys) — a German bank's data SHALL NOT sit in a
  US data center. *Source:* ConfusionClarity(#6 — Data Residency & Tenant Isolation);
  Vol 01 EKOS-01-073, Vol 04 EKOS-04-054.

- **EKOS-09-020 (Should, P2)** — A **Data Residency Map** widget SHALL visualize where
  each tenant's data physically resides. *Source:* ConfusionClarity(#6 — Data Residency
  Map widget).

---

## 9. Usage Analytics & Adoption Tracking

- **EKOS-09-021 (Should, P2)** — A built-in **analytics** capability SHALL track active
  users, widget views, AI insights consumed, and workflow completion rates — so a
  champion can justify ROI. *Source:* ConfusionClarity(#7 — Usage Analytics).

- **EKOS-09-022 (Must, P2)** — Analytics SHALL be **anonymized with opt-out**, and SHALL
  respect the no-train-on-customer-content rule (only anonymized behavioral signals).
  *Source:* ConfusionClarity(#7 — "anonymized, with opt-out"); Constitution §6.

- **EKOS-09-023 (Should, P2)** — An **ROI / adoption report** SHALL be generable for
  buyers/champions. *Source:* ConfusionClarity(#7); relates Vol 14.

---

## 10. Bulk Import & Migration

- **EKOS-09-024 (Must, P2)** — A **connector framework** SHALL support **bulk ingestion
  with a mapping wizard** from Jira REST, CSV, and Excel; **AI assists in mapping
  legacy fields to ontology types**. *Source:* ConfusionClarity(#8 — Bulk Import/
  Migration); Vol 03. *Rationale:* a company with 10,000 Jira issues won't recreate
  them manually.

- **EKOS-09-025 (Should, P2)** — **Comprehensive diagram import** SHALL be supported:
  **draw.io, Visio (.vsdx), Sparx EA, ArchiMate Exchange, Mermaid, PlantUML,
  Lucidchart, XMI/UML**. *Source:* old-catalog Vol 18 (comprehensive import via
  [[project-esip]]); ESIP already ships draw.io/Visio/Excalidraw/SVG/Mermaid/BPMN/
  PlantUML(Kroki) widget imports. *Depends:* Vol 02/14.

- **EKOS-09-026 (Could, P3)** — **Computer-vision whiteboard-photo → model** import SHALL
  convert a photographed whiteboard into a graph-backed diagram. *Source:* old-catalog
  Vol 18 (CV whiteboard-photo). *Depends:* Vol 03.

- **EKOS-09-027 (Should, P2)** — Imported artifacts SHALL be tagged with source-system
  provenance and reconciled to avoid drift/duplicates (Vol 01 §20). *Source:* Vol 01
  EKOS-01-088.

---

## 11. Offline Mode

- **EKOS-09-028 (Must, P2)** — The platform SHALL offer a **PWA with a local graph
  cache** so users can **view and even edit offline**, syncing back via CRDTs when
  connectivity returns. *Source:* ConfusionClarity(#12 — Offline Mode). *Note:* the
  **EKR Viewer** offline-first behavior is Vol 07; this is the *platform* offline mode.
  *Depends:* Vol 07, Vol 08 (CRDT).

- **EKOS-09-029 (Should, P2)** — Offline edits SHALL reconcile with the bitemporal graph
  on reconnect (interplays with Vol 01 OQ-01-B / Vol 08 OQ-08-B). *Source:* derived.

---

## 12. Public APIs & Webhooks

- **EKOS-09-030 (Must, P2)** — The platform SHALL expose a **public GraphQL API** that
  honors the **same ABAC model** as the UI (no privilege escalation via API). *Source:*
  ConfusionClarity(#13 — Public APIs); Vol 04.

- **EKOS-09-031 (Must, P2)** — **Webhooks** SHALL fire on any state transition / graph
  mutation (CloudEvents-formatted), enabling custom integrations. *Source:*
  ConfusionClarity(#13 — webhooks); blueprint §13; Vol 01 §13.

- **EKOS-09-032 (Should, P2)** — An **API Explorer** widget SHALL let users discover and
  test the API in-app. *Source:* ConfusionClarity(#13 — API Explorer widget).

- **EKOS-09-033 (Should, P2)** — A **Legacy Connector SDK** SHALL let enterprises wire
  up homegrown/legacy systems that no prebuilt connector covers. *Source:* blueprint
  §17 (Legacy Connector SDK).

---

## 13. License Management & Cost Controls

- **EKOS-09-034 (Should, P2)** — Department heads SHALL **control AI-analyst seats and
  premium-widget usage**: usage-based metering tied to **cost center**, alerts on
  approaching limits, and a **self-service purchase flow**. *Source:* ConfusionClarity
  (#14 — License Management & Cost Controls).

- **EKOS-09-035 (Should, P2)** — License tiers (e.g., **viewer / reviewer / contributor
  / enterprise**) SHALL gate capabilities via an entitlements model; ESIP's Epic G
  entitlements helper is the early implementation. *Source:* [[project-esip]] Epic G
  (LICENSE_TIERS, getEntitlements); blueprint §16. *Depends:* Vol 13 (economics).

- **EKOS-09-036 (Should, P2)** — License *policy enforcement* (seat limits, SoD) is
  governed via Vol 05 §15; this volume owns the *management UX & metering*. *Source:*
  Vol 05 EKOS-05-050.

---

## 14. Performance SLAs & Resource Governance

- **EKOS-09-037 (Should, P2)** — The platform SHALL enforce **per-widget compute
  quotas** so a heavy simulation widget cannot degrade the whole tenant (noisy-neighbor
  containment). *Source:* ConfusionClarity(#15 — Performance SLAs & Resource
  Governance).

- **EKOS-09-038 (Should, P2)** — Backend services SHALL **auto-scale**, and a **latency
  monitoring dashboard** SHALL be visible to tenant admins. *Source:* ConfusionClarity
  (#15).

- **EKOS-09-039 (Should, P2)** — Performance SLAs SHALL be **contractually stated** for
  Enterprise tiers. *Source:* blueprint §17 (Performance at scale); ConfusionClarity(#15).

---

## 15. Platform Resiliency & Scale

- **EKOS-09-040 (Must, P2)** — The platform SHALL be **active-active multi-region** with
  **automated backups** and **zero-downtime upgrades**. *Source:* blueprint §17
  (Platform Resiliency).

- **EKOS-09-041 (Must, P2)** — The architecture SHALL deliver **linear scalability at
  100K-employee scale** via the **federated graph + polyglot persistence** (Vol 01 §2/
  §15). *Source:* blueprint §17 (Performance at 100K scale); Vol 01.

- **EKOS-09-042 (Should, P2)** — The platform SHALL support **dedicated and air-gapped
  deployment** with **customer-managed keys**. *Source:* blueprint §17 (dedicated/air-
  gapped); Vol 04 EKOS-04-051.

---

## 16. Certifications & Compliance Posture

- **EKOS-09-043 (Must, P2)** — The platform SHALL target/achieve **SOC 2 Type II,
  ISO 27001, FedRAMP-ready, HIPAA, IRAP**, and **ISO 42001 (AI management)**. *Source:*
  blueprint §17 (certifications); old-catalog Vol 25; [[project-esip]] (SOC2/ISO27001/
  ISO42001/FedRAMP).

- **EKOS-09-044 (Should, P2)** — Compliance evidence for these SHALL be **generable from
  the graph** (Regulatory Evidence Pack, Vol 05 §6 / Vol 04 §18) rather than manually
  assembled. *Source:* Vol 05 EKOS-05-019; Vol 04 EKOS-04-059.

- **EKOS-09-045 (Must, MVP)** — The platform SHALL guarantee **no training on customer
  content**; anonymized opt-in behavioral signals only (Constitution §6). *Source:*
  Constitution §6; [[project-esip]].

---

## 17. Server-Side Enforcement & Pluggable Infrastructure

> Old-catalog Vol 25 mandate. Security logic must not live in the client.

- **EKOS-09-046 (Must, P2)** — Security/business-rule enforcement SHALL run in a
  **trusted server tier** (migrate logic off the React/Firebase client), per the
  server-side-enforcement mandate. *Source:* old-catalog Vol 25; Vol 04 EKOS-04-005.
  *Depends:* Vol 15.

- **EKOS-09-047 (Should, P2)** — **Antivirus/malware scanning** of uploads SHALL be
  **pluggable** (swappable AV engine). *Source:* old-catalog Vol 25 (pluggable AV).

- **EKOS-09-048 (Should, P2)** — **Storage targets SHALL be configurable** per tenant
  (cloud object store / on-prem / customer-owned), behind a storage port. *Source:*
  old-catalog Vol 25 (configurable storage targets); [[project-esip]] (Cloudinary now,
  swappable). *Depends:* Vol 15.

---

## 18. Executive & Trust Surfaces

- **EKOS-09-049 (Could, P3)** — An **Executive Mission Control** SHALL give leadership a
  portfolio-level health/risk/adoption/cost overview across the enterprise graph.
  *Source:* blueprint §17 (Executive Mission Control); relates Vol 14.

- **EKOS-09-050 (Should, P2)** — **Ethical AI explainability** SHALL be a first-class
  trust surface (evidence chains, confidence, Chain-of-Thought) — the engine is Vol 03,
  the program is Vol 05; this volume mandates it as an enterprise-trust requirement.
  *Source:* blueprint §17 (Ethical AI explainability); Vol 03 §14, Vol 05 §7.

---

## 19. Deferred-Elsewhere Confirmations (no duplication)

- **EKOS-09-051 (Must, MVP)** — Sandbox/staging (Vol 05/08), e-signatures/21 CFR
  Part 11 (Vol 04), platform feature flags (Vol 05), and admin audit log (Vol 05) are
  **part of the table-stakes set** and are owned by those volumes — this volume lists
  them for completeness and does not re-specify them. *Source:* ConfusionClarity(#4,
  #9, #10, #11); cross-volume hygiene.

---

## 20. Relationship to Existing ESIP

- **EKOS-09-052 (Must, MVP)** — ESIP's shipped **Epic G** (admin users, invite/
  activation, license tiers, entitlements) and **Epic H** (feature flags / Capabilities
  board) SHALL be recognized as early partial implementations of the identity/license/
  flags table-stakes, to be evolved into full IdP/SCIM/metering. *Source:*
  [[project-esip]] Epics G/H; [[esip-resume-state]].

---

## 21. Constitution Conformance (table-stakes-specific)

- **EKOS-09-053 (Must, MVP)** — IdP, AV engine, storage target, KMS, hosting region, and
  analytics backend are **illustrative & pluggable** (Constitution §6). *Source:*
  Constitution §6.

- **EKOS-09-054 (Must, MVP)** — Air-gap support excludes only internet-only features;
  offline/on-prem equivalents SHALL exist for identity, backup, and licensing.
  *Source:* Constitution §6; blueprint §17.

- **EKOS-09-055 (Must, MVP)** — Analytics & adoption tracking are **anonymized, opt-out,
  no-train-on-content** (Constitution §6). *Source:* Constitution §6; EKOS-09-022.

---

## 22. Open Questions (parked)

- **OQ-09-A** — Certification sequencing & scope (which certs for which deployment
  tiers, timeline) — go-to-market decision.
- **OQ-09-B** — Offline-edit conflict model shared with Vol 01 OQ-01-B / Vol 08 OQ-08-B.
- **OQ-09-C** — Diagram-import fidelity matrix (which formats round-trip vs. one-way) —
  align with Vol 02/14; ESIP already covers several.
- **OQ-09-D** — Metering granularity vs. cost of measuring (per-AI-insight metering at
  scale) — interplays with Vol 01 §15 + Vol 13.
- **OQ-09-E** — White-label depth (per-tenant custom domain TLS, branded viewer store
  listings) — Vol 13/GTM.

---

## 23. Requirement Index (this volume)

EKOS-09-001 … EKOS-09-055 (stable numbering; i18n/white-label/accessibility/identity/
DR/residency/analytics/import/offline/APIs/licensing/SLAs/resiliency/certifications/
server-side/executive surfaces expand to 86 distinct stated rules; 4 items cross-
referenced to their owning volumes).

> **Next volume:** `10 — Personal Knowledge Space & Digital Twin (résumé = one
> projection)` — **MUST own the no-bespoke-Resume-Widget ruling flagged in Vol 02
> EKOS-02-050/051.**
> Per protocol: STOP here. Await user "proceed".
