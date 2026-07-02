# Volume 13 — Client Onboarding, Licensing & Tenant Administration

**Prefix:** ONB
**Status:** V1 — authored 2026-06-23
**Scope:** Guided tenant/client onboarding; subscription editions and per-persona licensing; licensing rules and enforcement; admin users and delegated administration; user management and provisioning; invite and activation flow; admin configuration zone (database, cloud, BYOK AI keys, Git targets, Knowledge/SaaS connectors, branding, email, security policy, data residency); billing and metering.

**Out of scope (handled in other volumes):**
- Deep IAM/SSO/SCIM/ABAC internals, secrets vault implementation, multi-tenancy isolation kernel → **Volume 11**
- Connector sync internals, crawl schedules, content pipelines → **Volume 08**
- AI provider routing, model routing logic, guardrails, prompt library → **Volume 06**
- Artifact-to-Git push formatting, PR templates, deliverable rendering → **Volume 10**
- Per-user dashboard preferences, layout personalization, adaptive learning → **Volume 14**

---

> **Phase banner (Priority ≠ Phase — see 00-MASTER-INDEX §6).** Full billing/metering, the 31 licensing-enforcement rules (ONB-LR), SCIM auto-provisioning, and delegated-admin are **Phase 4**. The **MVP** scope for this volume is limited to: a minimal license-type model + user assignment + a single entitlement gate. MoSCoW "Must" markings here denote "in the product eventually," not "in the MVP" — do not read this volume's Must-count as the MVP build list.

---

## Personas & Admin Roles (this volume)

| Role | Description |
|------|-------------|
| **Tenant Admin / Owner** | First super-admin created during onboarding; owns the tenant; all config permissions |
| **User Admin** | Manages users, licenses, invites, teams — delegated by Tenant Admin |
| **Billing Admin** | Manages subscriptions, invoices, seat purchases, true-ups |
| **Config Admin** | Manages integration config: DB, cloud, AI keys, Git targets, connectors, branding |
| **Security Admin** | Manages SSO, password policy, session policy, data residency, audit |
| **P1–P12 architects** | End-users; subject to licensing rules; limited self-service visibility |
| **Platform Ops** (internal) | ESIP platform team; cross-tenant super-admin; not exposed to clients |

---

## Requirement Conventions

- **ID format:** `REQ-ONB-NNN` (general) or `REQ-ONB<Sub>-NNN` where Sub = two-letter sub-area code
- **Description:** single SHALL statement, testable, atomic
- **Categories:** Functional | NFR | Integration | Data | AI/ML | UX | Security | Governance | Reporting

---

## Capability Areas

1. [ONB-OB] Tenant Onboarding Flow
2. [ONB-ED] Editions, Subscriptions & Feature Gating
3. [ONB-LT] Per-Persona License Types & Pools
4. [ONB-LR] Licensing Rules & Enforcement
5. [ONB-AD] Admin Users & Delegated Administration
6. [ONB-UM] User Management & Provisioning
7. [ONB-IV] Invite & Activation Flow
8. [ONB-DB] Admin Config — Database Provider
9. [ONB-CL] Admin Config — Cloud Account & Region
10. [ONB-AK] Admin Config — BYOK AI Keys
11. [ONB-GT] Admin Config — Git Targets
12. [ONB-KC] Admin Config — Knowledge/SaaS Connectors (org-level auth)
13. [ONB-BX] Admin Config — Branding, Email/SMTP & Theme
14. [ONB-SP] Admin Config — Security Policy
15. [ONB-DR] Admin Config — Data Residency & Retention
16. [ONB-BL] Billing, Metering & Seat Management
17. [ONB-RP] License Compliance & Reporting
18. [ONB-AU] Onboarding & Admin Audit Log

---

## ## ONB-OB — Tenant Onboarding Flow

### ONB-OB — Overview
The guided onboarding flow takes a net-new client from "account requested" to "tenant provisioned and first admin logged in." It captures org profile, industry, deployment preferences, database provider, optional project imports, and contract acceptance. It must support both **self-serve** (no human from ESIP) and **assisted** (ESIP CSM drives) paths.

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBOB-001 | Self-Serve Onboarding Path | The system SHALL provide a fully self-serve onboarding path that allows a prospective tenant admin to sign up, complete all onboarding steps, and activate the tenant without requiring manual intervention from ESIP staff. | Functional | Tenant Admin | Must | Scalable SaaS growth; reduces COGS |
| REQ-ONBOB-002 | Assisted Onboarding Path | The system SHALL support an assisted onboarding mode where an ESIP CSM can initiate a tenant, pre-fill configuration, and co-complete the onboarding wizard alongside the client. | Functional | Tenant Admin, Platform Ops | Should | Enterprise sales motion; CSM-led deployment |
| REQ-ONBOB-003 | Onboarding Wizard — Stepwise Flow | The system SHALL present onboarding as a numbered, resumable, multi-step wizard (minimum steps: Account, Org Profile, Deployment, Licensing, Admin User, Terms, Provision) with progress indicator and ability to navigate back. | UX | Tenant Admin | Must | UX standard for enterprise SaaS onboarding |
| REQ-ONBOB-004 | Capture Organisation Profile | The system SHALL capture organisation name, legal entity name, primary country/jurisdiction, industry vertical (selectable from a standard taxonomy), estimated employee count range, and primary use-case during onboarding. | Data | Tenant Admin | Must | Drives persona defaults, compliance region, feature recommendations |
| REQ-ONBOB-005 | Industry Vertical Taxonomy | The system SHALL provide a configurable industry taxonomy (minimum: Financial Services, Insurance, Healthcare, Retail, Manufacturing, Public Sector, Telco, Energy/Utilities, Technology, Professional Services, Other) for selection at onboarding. | Data | Tenant Admin | Must | Persona recommendations vary by industry |
| REQ-ONBOB-006 | Subdomain / Tenant Slug | The system SHALL allow the Tenant Admin to choose a unique subdomain slug (e.g., `acme.esip.io`) during onboarding, validate availability in real time, and reserve it on tenant provisioning. | Functional | Tenant Admin | Must | Tenant identity; branded URL |
| REQ-ONBOB-007 | Choose Database Provider | The system SHALL present a database provider selection step during onboarding (PostgreSQL, SQL Server, MySQL, Oracle), with a BYOD (Bring Your Own Database) connection-string input or a managed/hosted option, before tenant provisioning begins. | Functional | Tenant Admin | Must | Client-configurable DB is a core architectural requirement |
| REQ-ONBOB-008 | Choose Cloud Deployment Region | The system SHALL allow the tenant to choose the cloud provider (AWS, Azure, GCP, IBM Cloud, on-premises) and target region for their tenant workloads and data storage during onboarding. | Functional | Tenant Admin | Must | Cloud-neutral + data residency requirements |
| REQ-ONBOB-009 | Optional Import — Existing Projects | The system SHALL offer an optional step to import existing project data (JSON/ZIP export from another ESIP instance or supported format) during onboarding, deferring the import job to post-provisioning. | Functional | Tenant Admin | Should | Migration from prior ESIP/prototype instances; reduces friction |
| REQ-ONBOB-010 | Optional Import — Existing Designs & Artifacts | The system SHALL allow upload of existing architecture artifacts (Visio, drawio, ArchiMate XML, Word/PDF documents) during onboarding for background ingestion into the tenant's Knowledge Hub after provisioning. | Functional | Tenant Admin | Should | Carry-forward of prior architecture work |
| REQ-ONBOB-011 | Accept Terms of Service & DPA | The system SHALL require explicit checkbox acceptance of the Terms of Service, Privacy Policy, and Data Processing Agreement (DPA) before tenant provisioning, recording the accepting user's identity, IP address, timestamp, and document version. | Governance | Tenant Admin | Must | Legal/compliance; GDPR DPA obligations |
| REQ-ONBOB-012 | Contract / MSA Upload | The system SHALL allow the Tenant Admin or ESIP CSM to upload a signed MSA/contract document during onboarding (PDF), stored immutably against the tenant record. | Governance | Tenant Admin, Platform Ops | Should | Enterprise procurement requirement |
| REQ-ONBOB-013 | Tenant Provisioning Job | Upon completion of onboarding steps, the system SHALL asynchronously provision the tenant (create schema/database, configure cloud resources, seed default roles/permissions, generate tenant encryption keys) and display real-time provisioning status to the waiting admin. | Functional | Tenant Admin | Must | Core provisioning; async avoids timeouts |
| REQ-ONBOB-014 | Provisioning Failure Handling | The system SHALL detect provisioning failures at each step, display a human-readable error message indicating the failed component, allow retry without re-entering already-validated data, and alert ESIP Platform Ops automatically. | Functional | Tenant Admin, Platform Ops | Must | Production reliability; support escalation |
| REQ-ONBOB-015 | Onboarding Progress Checklist | The system SHALL present a persistent onboarding checklist (viewable post-provisioning in the Admin dashboard) tracking completion status of: Admin profile set, first users invited, database configured, AI provider configured, Git target configured, first project created, first deliverable generated. | UX | Tenant Admin | Must | Drives activation; reduces churn |
| REQ-ONBOB-016 | Onboarding Checklist — Dismiss / Complete | The system SHALL allow the Tenant Admin to mark individual checklist items as skipped (with reason) or complete, and to dismiss the checklist once all items are resolved. | UX | Tenant Admin | Should | Avoids persistent nag for mature tenants |
| REQ-ONBOB-017 | Onboarding Email Sequence | The system SHALL trigger an onboarding email sequence to the Tenant Admin: (a) welcome + activation link immediately on provisioning, (b) Day 2 checklist reminder if checklist <50% complete, (c) Day 7 help offer if no project created. | Functional | Tenant Admin | Should | Standard SaaS activation; drives onboarding completion |
| REQ-ONBOB-018 | Resumable Onboarding | The system SHALL persist onboarding wizard state so that a Tenant Admin who abandons mid-flow can return (via the original signup link or re-login) and resume from the last completed step, with all prior entries retained. | Functional | Tenant Admin | Must | Enterprise buyers often complete over multiple sessions |
| REQ-ONBOB-019 | Onboarding Session Expiry | The system SHALL invalidate an incomplete onboarding session after 72 hours of inactivity and notify the initiating email, while preserving all entered data for re-authentication and resume. | Security | Tenant Admin | Should | Security hygiene; abandoned provisioning cleanup |
| REQ-ONBOB-020 | Onboarding Sandbox / Trial Mode | The system SHALL support a trial-mode onboarding path that provisions a tenant with a sandbox database (ESIP-managed), a 30-day expiry, and a capped feature set aligned to the Trial edition, without requiring database or cloud config. | Functional | Tenant Admin | Must | Frictionless trial; no infrastructure commitment |
| REQ-ONBOB-021 | Trial-to-Paid Conversion | The system SHALL provide a one-click upgrade path from Trial to a paid edition within the Admin dashboard, preserving all trial data, migrating the tenant to the customer-configured database, and applying the purchased license immediately. | Functional | Tenant Admin, Billing Admin | Must | Monetisation; retain trial work |
| REQ-ONBOB-022 | Onboarding Audit Trail | The system SHALL record every onboarding step completion event (step name, timestamp, user identity, IP, outcome) in the tenant audit log. | Governance | Platform Ops | Must | Compliance; dispute resolution |
| REQ-ONBOB-023 | Multi-Subsidiary Onboarding | The system SHALL allow a single parent tenant to spawn subsidiary tenants (child tenants) via the Admin dashboard, reusing the parent's MSA/contract, with inherited default settings and a separate license pool per subsidiary. | Functional | Tenant Admin | Should | Enterprise multi-entity deployments |
| REQ-ONBOB-024 | Onboarding Persona Recommendation | Based on the org profile (industry, size, use-case), the system SHALL recommend a starter set of architect personas (P1–P12) with suggested license quantities, which the Tenant Admin can accept, modify, or override during onboarding. | AI/ML | Tenant Admin | Could | Accelerates correct licensing; reduces support calls |

---

## ## ONB-ED — Editions, Subscriptions & Feature Gating

### ONB-ED — Overview
ESIP offers edition tiers (e.g., Trial, Starter, Professional, Enterprise, Custom) that gate features, persona availability, storage quotas, AI credit allotments, and SLA levels. Feature gating is enforced at every access point.

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBED-001 | Edition Tiers Definition | The system SHALL support at minimum four named editions — Trial, Starter, Professional, Enterprise — each with a defined feature matrix governing which capabilities, personas, storage quotas, AI credits, and integrations are available. | Functional | All | Must | Core commercial model |
| REQ-ONBED-002 | Custom Edition | The system SHALL support a Custom edition whose feature matrix, persona availability, seat limits, and pricing are configured per-tenant by ESIP Platform Ops for large/strategic accounts. | Functional | Platform Ops | Should | Enterprise deals; negotiated terms |
| REQ-ONBED-003 | Feature Flag Enforcement | The system SHALL enforce edition-based feature flags at the API gateway and UI layer, preventing access to any capability or persona not included in the tenant's current edition, and displaying a contextual upgrade prompt instead. | Functional | All | Must | Revenue protection; gating |
| REQ-ONBED-004 | Feature Gate — Upgrade Prompt | When a user attempts to access a gated feature, the system SHALL display a clear upgrade prompt explaining the feature, which edition unlocks it, and a CTA to contact Billing Admin or upgrade. | UX | All | Must | Conversion; reduces support tickets |
| REQ-ONBED-005 | Edition-Level Storage Quota | The system SHALL enforce per-tenant storage quotas (documents, diagrams, artifacts) by edition, alert at 80% utilisation, and prevent new uploads at 100% with a quota-exceeded message and upgrade CTA. | Functional | Tenant Admin | Must | Cost control; capacity planning |
| REQ-ONBED-006 | Edition-Level AI Credit Allotment | The system SHALL allocate a monthly AI credit budget per tenant by edition tier, track consumption in real time, soft-warn at 80%, and hard-block AI features at 100% unless overage is enabled. | Functional | Tenant Admin | Must | AI cost management; see Vol 06 for routing details |
| REQ-ONBED-007 | Add-On Modules | The system SHALL support purchasable add-on modules (e.g., Advanced Code Scanning, Additional AI Credits, Extra Storage, Additional Personas) that activate incrementally on top of the base edition without a full tier upgrade. | Functional | Billing Admin | Should | Upsell flexibility |
| REQ-ONBED-008 | Edition Upgrade Path | The system SHALL allow Billing Admins to upgrade the tenant edition (e.g., Starter → Professional) from the Admin dashboard; the upgrade SHALL take effect immediately, unlock gated features, and adjust billing from the effective date. | Functional | Billing Admin | Must | Self-serve upgrade; commercial agility |
| REQ-ONBED-009 | Edition Downgrade Path | The system SHALL allow Billing Admins to schedule a downgrade to a lower edition (effective at next billing cycle), displaying a list of features and data that will become inaccessible or read-only post-downgrade for admin review before confirmation. | Functional | Billing Admin | Should | Churn management; customer control |
| REQ-ONBED-010 | Free / Freemium Tier | The system SHALL support a permanent Free tier (distinct from Trial) with minimal quotas and persona limits, enabling a low-friction entry point without a time expiry. | Functional | Tenant Admin | Could | Growth/PLG; awareness |
| REQ-ONBED-011 | Edition History | The system SHALL maintain an immutable edition change log (prior edition, new edition, effective date, changed-by user, billing impact) per tenant accessible to Tenant Admin and Platform Ops. | Governance | Tenant Admin | Should | Audit; billing dispute resolution |
| REQ-ONBED-012 | Trial Expiry Notification | The system SHALL send email notifications to the Tenant Admin at 14 days, 7 days, 3 days, and 1 day before trial expiry, each including the current checklist completion status and a conversion CTA. | Functional | Tenant Admin | Must | Reduce trial abandonment |
| REQ-ONBED-013 | Post-Trial Lockout | Upon trial expiry without conversion, the system SHALL transition the tenant to a read-only locked state (no edits, no new AI runs, no exports), retain data for 30 days, and display a conversion prompt on every page load until upgraded or data deleted. | Functional | Tenant Admin | Must | Data retention; recovery window; commercial pressure |
| REQ-ONBED-014 | Edition Feature Matrix UI | The system SHALL display a current feature comparison matrix (all editions × all capabilities/personas) within the Admin dashboard Billing section, always reflecting the live edition configuration. | UX | Billing Admin | Should | Informed purchase decision |

---

## ## ONB-LT — Per-Persona License Types & Pools

### ONB-LT — Overview
Each of the 12 architect personas (P1–P12 plus extensible custom roles) is a **distinct license type**. A tenant purchases a quantity of each persona license. Users are assigned one or more persona licenses from the tenant pool. This section covers the license type model; enforcement rules are in ONB-LR.

> **Boundary note (license-tier SPLIT — V13 ↔ V17).** **V13 owns billing, seat-pool metering & entitlement ENFORCEMENT for all tiers; V17 owns the tier capability definitions.** The non-architect access tiers — **Viewer, Reviewer, and PM (Project Manager)** — have their *capability definitions* owned by V17 (`REQ-LIC-*`). Within V13 these tiers exist only as license-type pool entries that the enforcement engine (ONB-LR) meters and gates; their feature/entitlement scope is normative in V17. The Viewer row below (REQ-ONBLT-010) and the Reviewer/PM tiers are therefore **cross-ref stubs** to V17 — do not redefine their capabilities here.

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBLT-001 | Per-Persona License Type Catalogue | The system SHALL define a license type for each architect persona: EA (Enterprise Architect), SA (Solution Architect), AppA (Application Architect), IntA (Integration Architect), APIA (API Architect), CRMA (CRM Architect), DA (Data Architect), SecA (Security Architect), CloudA (Cloud Architect), InfraA (Infrastructure Architect), MSA (Microservices Architect), plus CustomA (extensible). | Data | Tenant Admin, Billing Admin | Must | Core commercial model; per-persona monetisation |
| REQ-ONBLT-002 | License Type — Feature Entitlement Mapping | Each license type SHALL map to a defined set of feature entitlements (accessible personas, diagram types, deliverable templates, AI capabilities, framework support) that are enforced at runtime. | Functional | All | Must | Entitlement enforcement basis |
| REQ-ONBLT-003 | Purchased Seat Quantity per License Type | The system SHALL allow a tenant to purchase a specific seat quantity for each license type independently (e.g., 5 EA seats, 10 SA seats, 3 DA seats), stored per-tenant in the license pool. | Data | Billing Admin | Must | Granular commercial control |
| REQ-ONBLT-004 | Named User Licensing | The system SHALL support named-user licensing, where each purchased seat is assigned to a specific user identity and cannot simultaneously be used by another user. | Functional | Tenant Admin | Must | Standard enterprise named-user model |
| REQ-ONBLT-005 | Concurrent User Licensing | The system SHALL support concurrent/floating licensing as an alternative model, where a purchased quantity represents simultaneous active sessions rather than named users, and the system enforces the concurrent limit in real time. | Functional | Tenant Admin | Should | Common in tooling (e.g., Sparx EA); large teams rotating |
| REQ-ONBLT-006 | Mixed Licensing Models per Pool | The system SHALL allow a tenant to operate named licensing for some persona types and concurrent licensing for others within the same tenant, configured per license-type pool. | Functional | Tenant Admin | Could | Flexible enterprise commercial structures |
| REQ-ONBLT-007 | Multi-Persona Assignment per User | The system SHALL allow a single user to be assigned licenses for multiple persona types (e.g., a user with both SA and SecA licenses), with entitlements being the union of all assigned persona entitlements. | Functional | User Admin | Must | Many practitioners hold multiple architecture roles |
| REQ-ONBLT-008 | Tenant License Pool Isolation | Each tenant's license pool SHALL be strictly isolated; a license purchased for Tenant A cannot be consumed by users of Tenant B even if both are subsidiaries under the same parent account, unless explicitly pooled via a parent-subsidiary license agreement. | Security | Platform Ops | Must | Multi-tenant isolation; compliance |
| REQ-ONBLT-009 | Custom Persona License Type | The system SHALL allow Platform Ops (or Tenant Admin on Enterprise edition) to define a custom persona license type with a configured name, feature entitlement set, and purchased quantity, extending the standard 12-persona catalogue. | Functional | Tenant Admin, Platform Ops | Should | Extensibility; custom architecture roles |
| REQ-ONBLT-010 | Viewer License (pool stub → V17) | **Cross-ref stub.** V13 SHALL maintain a Viewer license type as a meterable, gateable entry in the tenant license pool (billing, seat counting, entitlement enforcement); the Viewer tier's *capability definition* (read-only access to projects, diagrams, and deliverables, not consuming an architect persona seat) is **owned by V17 — see `REQ-LIC-*`**. This row exists for pool/enforcement purposes only and SHALL NOT redefine Viewer capabilities. | Functional | Tenant Admin | Should | Stakeholder access; wider adoption. Capability scope normative in V17; V13 owns enforcement only. |
| REQ-ONBLT-013 | Reviewer & PM Licenses (pool stub → V17) | **Cross-ref stub.** V13 SHALL maintain Reviewer and PM (Project Manager) license types as meterable, gateable entries in the tenant license pool; the *capability definitions* for the Reviewer and PM tiers are **owned by V17 — see `REQ-LIC-*`**. V13 provides billing, seat-pool metering, and entitlement enforcement for these tiers; it SHALL NOT define their feature scope. | Functional | Tenant Admin | Should | Reviewer/PM are non-architect access tiers; definitions consolidated in V17 per the license-tier SPLIT, enforcement in V13. |
| REQ-ONBLT-011 | License Pool Dashboard | The system SHALL provide a Tenant Admin / Billing Admin dashboard panel showing, for each license type: purchased quantity, assigned quantity, available quantity, active concurrent sessions (if concurrent model), and expiry date. | Reporting | Tenant Admin, Billing Admin | Must | Operational visibility; licence management |
| REQ-ONBLT-012 | License Type Entitlement Version | Each license type entitlement mapping SHALL be versioned; changes to entitlements SHALL apply to future seat purchases but not retroactively reduce entitlements of existing assigned seats until their renewal. | Governance | Platform Ops | Should | Contractual predictability; avoid breaking users at renewal |

---

## ## ONB-LR — Licensing Rules & Enforcement

### ONB-LR — Overview
This is the most critical enforcement subsystem. All rules below are mandatory for a commercially viable, legally sound SaaS product. Rules span allocation, over-allocation prevention, reclamation, role changes, concurrent-session limits, hard/soft caps, overage, expiry, compliance, and entitlement checks.

> **Entitlement-check exception (C3 — anonymous Viewer via public-share token).** The synchronous per-action entitlement check (REQ-ONBLR-018/019) assumes an authenticated session identity. There is one sanctioned exception: **anonymous Viewer access via a public-share token.** When a resource is opened through a valid public-share token, the request carries the **Viewer grant intrinsically from the token itself — without a session-identity lookup or a named-seat allocation**. The token's scope (read-only, resource-bounded, time-bounded) IS the entitlement; no per-user license is consumed. See REQ-ONBLR-032 below. (Viewer tier capability definition is owned by V17 — see ONB-LT boundary note.)

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBLR-001 | Prevent Over-Allocation Beyond Purchased Seats | The system SHALL prevent a User Admin from assigning a persona license type to a new user when zero seats remain in that license type's pool, returning a clear error message stating the type, current allocation, and purchased quantity. | Functional | User Admin | Must | Core licensing integrity |
| REQ-ONBLR-002 | Real-Time Seat Availability Check | Before every license assignment action (individual or bulk), the system SHALL perform a real-time check of available seats in the target license pool and reject the assignment atomically if insufficient seats exist. | Functional | User Admin | Must | Race-condition prevention in concurrent admin operations |
| REQ-ONBLR-003 | Allocate License to User | The system SHALL record a license allocation event (user ID, license type, assigned-by, timestamp) when a seat is assigned, immediately decrementing the available count in the pool. | Data | User Admin | Must | Allocation tracking |
| REQ-ONBLR-004 | Deallocate License from User | The system SHALL support manual deallocation of a license from a user by a User Admin, returning the seat to the available pool immediately and revoking the user's entitlements for that persona type on next authentication token refresh (max 5 minutes). | Functional | User Admin | Must | Seat recovery; offboarding |
| REQ-ONBLR-005 | Automatic Seat Reclamation on Inactivity | The system SHALL reclaim a named-user persona seat automatically if the assigned user has not logged in or performed any licensed action for a configurable inactivity period (default: 90 days, admin-configurable 30–365 days), notifying the user by email 7 days and 1 day before reclamation. | Functional | User Admin | Must | Prevent perpetual licence hoarding; industry standard (c.f. GitHub Enterprise seat reclaim) |
| REQ-ONBLR-006 | Reclamation Candidate Report | The system SHALL provide a User Admin report of all users approaching the inactivity reclamation threshold (within 14 days), with one-click manual reclaim or extension per user. | Reporting | User Admin | Must | Admin visibility before auto-reclaim |
| REQ-ONBLR-007 | Role-Change Re-Licensing | When a user's assigned persona type is changed (e.g., from AppA to SA), the system SHALL atomically deallocate the prior persona seat, allocate the new persona seat (checking availability), and apply the new entitlement set in a single transaction; rollback if new seat unavailable. | Functional | User Admin | Must | Atomicity; no orphaned allocations |
| REQ-ONBLR-008 | License Upgrade for User | The system SHALL allow a User Admin to add an additional persona license type to a user (upgrade) without removing existing ones, consuming one seat from each type's pool. | Functional | User Admin | Must | Multi-role users; incremental upgrade |
| REQ-ONBLR-009 | License Downgrade for User | The system SHALL allow a User Admin to remove one or more persona license types from a user (downgrade), returning each seat to the pool and revoking the corresponding entitlements within the token refresh window. | Functional | User Admin | Must | License cost optimisation |
| REQ-ONBLR-010 | Seat Transfer Between Users | The system SHALL support a seat transfer operation that atomically deallocates a persona license from User A and allocates it to User B in a single admin action, with a single audit record. | Functional | User Admin | Should | Org restructuring; personnel change |
| REQ-ONBLR-011 | Concurrent Session Limit Enforcement | For concurrent-model licenses, the system SHALL enforce the purchased concurrent session count by rejecting new logins or session activations for that persona type once the limit is reached, displaying a "No concurrent sessions available" message with an estimated wait or contact-admin CTA. | Functional | All | Must | Concurrent licence model integrity |
| REQ-ONBLR-012 | Concurrent Session Release on Logout/Expiry | The system SHALL release a concurrent session slot immediately upon explicit user logout or session token expiry, making it available to queued users or new logins without admin intervention. | Functional | All | Must | UX; prevent stuck sessions |
| REQ-ONBLR-013 | Concurrent Session Stale Recovery | The system SHALL detect stale concurrent sessions (no heartbeat for > configurable idle timeout, default 30 min) and release the slot automatically, with a configurable grace period before forced release. | Functional | User Admin | Must | Prevent session hogging from crashed clients |
| REQ-ONBLR-014 | Hard Cap Enforcement | The system SHALL enforce hard seat caps (default behaviour): no user can be assigned a license when zero seats remain regardless of admin privilege level; the only exception is a Billing Admin purchasing additional seats. | Functional | Tenant Admin, Billing Admin | Must | Revenue protection; legal compliance |
| REQ-ONBLR-015 | Soft Cap with Overage Option | The system SHALL optionally allow a Billing Admin to enable a soft-cap overage mode per license type, permitting seat assignments up to a configurable overage limit (e.g., 110% of purchased), flagging overage use in billing/metering for true-up. | Functional | Billing Admin | Should | Enterprise flexibility; true-up model common in large contracts |
| REQ-ONBLR-016 | Overage Notification | When soft-cap overage is triggered, the system SHALL immediately notify the Billing Admin and Tenant Admin via email and in-app notification, stating the license type, overage count, and projected additional cost. | Functional | Billing Admin, Tenant Admin | Must | Cost visibility; prevents bill shock |
| REQ-ONBLR-017 | True-Up Period | The system SHALL support a configurable true-up period (default: annual, coinciding with contract anniversary) during which accumulated overage usage is reconciled, invoiced, and the purchased seat count updated to reflect actual usage. | Functional | Billing Admin | Should | Enterprise contract model |
| REQ-ONBLR-018 | Feature / Persona Gating by License Type | At every API endpoint and UI element that is persona-gated, the system SHALL verify the requesting user holds an active, valid license of the required persona type; deny with HTTP 403 and a UI-level "Upgrade required" message if not. | Security | All | Must | Entitlement enforcement |
| REQ-ONBLR-019 | Entitlement Check on Every Gated Action | The system SHALL perform a synchronous entitlement check (from a low-latency license cache, TTL ≤ 60 seconds) on every gated action (UI navigation, API call, AI invocation, diagram creation, deliverable generation, export), not only at login. | Security | All | Must | Prevent cached-entitlement bypass |
| REQ-ONBLR-020 | Entitlement Cache Invalidation | The system SHALL invalidate a user's entitlement cache within 60 seconds of any license change event (allocation, deallocation, type change, expiry) affecting that user. | Security | User Admin | Must | Ensures near-real-time enforcement after changes |
| REQ-ONBLR-021 | License Expiry — Warning Notifications | The system SHALL send automated email notifications to Billing Admin and Tenant Admin at 60, 30, 14, 7, and 1 day before any license type's term expiry date. | Functional | Billing Admin, Tenant Admin | Must | Prevent accidental expiry; renewal prompt |
| REQ-ONBLR-022 | License Expiry — Grace Period | The system SHALL apply a configurable grace period (default: 14 days) after license expiry during which users retain full access, after which the system transitions to expiry-lockout behaviour. | Functional | Billing Admin | Must | Accounts-payable lag; operational continuity |
| REQ-ONBLR-023 | Expiry Lockout — Read-Only Mode | After the grace period expires, users holding expired license types SHALL be locked to read-only mode: they can view diagrams, documents, and deliverables, but cannot create, edit, run AI operations, or export. A renewal CTA is displayed on every page. | Functional | All | Must | Data preservation; recovery window; commercial pressure |
| REQ-ONBLR-024 | Expiry Lockout — Full Block | If the tenant edition itself expires (not individual licenses), the system SHALL fully block all non-admin user logins and display a platform-level renewal page; the Tenant Admin retains read-only admin access for 30 days to export data. | Functional | Tenant Admin | Must | Last-resort data preservation |
| REQ-ONBLR-025 | Auto-Renewal | The system SHALL support auto-renewal of license terms and subscriptions via stored payment method; auto-renewal events SHALL be logged and a receipt emailed to Billing Admin. | Functional | Billing Admin | Should | Reduce churn from lapsed renewals; SaaS standard |
| REQ-ONBLR-026 | Renewal Opt-Out | The system SHALL allow a Billing Admin to disable auto-renewal for any license pool or subscription, requiring manual confirmation before the next billing cycle, with a non-renewal confirmation email sent immediately. | Functional | Billing Admin | Should | Customer control |
| REQ-ONBLR-027 | License Compliance Audit | The system SHALL produce an on-demand license compliance report (license type, purchased, assigned, active, inactive, overage, expiry date) exportable as CSV/PDF, suitable for internal compliance review or external audit. | Reporting | Tenant Admin, Billing Admin | Must | Enterprise compliance; software asset management |
| REQ-ONBLR-028 | Usage Metering per Seat | The system SHALL record per-seat usage metrics (login events, AI calls, diagram operations, deliverable exports) per license type per billing period, available in the Admin dashboard and billing export. | Reporting | Billing Admin | Must | True-up evidence; ROI reporting |
| REQ-ONBLR-029 | License Pool Multi-Tenant Isolation | The system SHALL enforce that license pool counts, seat assignments, and entitlement checks are strictly scoped to a single tenant, with no cross-tenant license leakage, validated by automated integration tests on every release. | Security | Platform Ops | Must | Multi-tenancy correctness |
| REQ-ONBLR-030 | Bulk Seat Purchase | The system SHALL allow Billing Admin to purchase additional seats for any license type in bulk from the Admin dashboard, immediately increasing the available pool count and triggering prorated billing. | Functional | Billing Admin | Must | Organic growth; self-serve commercial |
| REQ-ONBLR-031 | License Pool History | The system SHALL maintain an immutable history of all license pool changes (purchase, reclaim, expiry, overage, true-up) per license type per tenant, accessible to Billing Admin and Platform Ops. | Governance | Billing Admin | Should | Audit; billing dispute evidence |

### ONB-LR — Reconciliation Additions

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBLR-032 | Public-Share-Token Viewer Grant (entitlement-check exception) | The system SHALL accept a valid, unexpired **public-share token** as a self-contained Viewer entitlement: a request bearing such a token SHALL be granted **read-only Viewer access to the token-scoped resource(s) WITHOUT a session-identity lookup and WITHOUT consuming a named or concurrent license seat**. The token's embedded scope (resource set, read-only, expiry) IS the entitlement evaluated at REQ-ONBLR-018/019; the system SHALL reject the request if the token is revoked, expired, or out of scope, and SHALL record anonymous-Viewer access in the audit log with the token reference (not a user identity). | Security | All | Must | C3 reconciliation — anonymous public-share Viewer access must work without an identity (no seat consumption) while still being gated, scoped, revocable, and audited. Viewer capability scope owned by V17. |

---

## ## ONB-AD — Admin Users & Delegated Administration

### ONB-AD — Overview
The first Tenant Admin / Owner is created during provisioning. They can delegate administration to named users via admin roles. All admin roles follow least-privilege. Admin actions are fully audited.

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBAD-001 | First Tenant Admin Creation | The system SHALL automatically create the first Tenant Admin / Owner account using the identity provided during onboarding (email + set-password or SSO), granting full administrative privileges within the tenant scope. | Functional | Tenant Admin | Must | Bootstrapping; entry point |
| REQ-ONBAD-002 | Tenant Admin — Full Tenant Privileges | The Tenant Admin / Owner SHALL have full privileges within their tenant: manage all users, all licenses, all configuration, all billing (or delegate to Billing Admin), and all security settings, but SHALL NOT have privileges on other tenants. | Security | Tenant Admin | Must | Scope containment; multi-tenancy |
| REQ-ONBAD-003 | Delegated Admin Role — User Admin | The system SHALL support a User Admin delegated admin role with permissions to: invite users, assign/deallocate licenses, activate/deactivate/suspend users, manage teams — but not access billing, security policy, or infrastructure config. | Functional | Tenant Admin | Must | Least-privilege; separation of duties |
| REQ-ONBAD-004 | Delegated Admin Role — Billing Admin | The system SHALL support a Billing Admin role with permissions to: view/manage subscriptions, purchase seats, view invoices/usage, manage payment methods, configure overage — but not manage users, security policy, or infra config. | Functional | Tenant Admin | Must | Finance/procurement separation |
| REQ-ONBAD-005 | Delegated Admin Role — Config Admin | The system SHALL support a Config Admin role with permissions to: configure database, cloud, AI keys, Git targets, connectors, branding, SMTP — but not manage users or billing. | Functional | Tenant Admin | Must | IT/DevOps separation from HR/billing |
| REQ-ONBAD-006 | Delegated Admin Role — Security Admin | The system SHALL support a Security Admin role with permissions to: configure SSO/SAML/OIDC, password policy, session policy, MFA policy, data residency, view security audit log — but not manage end-users directly or billing. | Security | Tenant Admin | Must | Security-team separation; SoD |
| REQ-ONBAD-007 | Multiple Admins per Role | The system SHALL allow multiple users to hold each delegated admin role simultaneously (no single-admin-per-role restriction), supporting redundancy and team-based administration. | Functional | Tenant Admin | Must | Operational resilience; avoid single points of failure |
| REQ-ONBAD-008 | Admin Role Assignment / Revocation | The system SHALL allow the Tenant Admin to grant or revoke any admin role from any user at any time, with the change taking effect within the next token refresh cycle (≤5 minutes). | Functional | Tenant Admin | Must | Admin lifecycle management |
| REQ-ONBAD-009 | Admin Privileges Do Not Bypass Entitlement | Holding an admin role SHALL NOT grant the admin any architect persona entitlements beyond those conferred by their explicitly assigned persona licenses; admin is a permission overlay, not a license bypass. | Security | All | Must | License integrity; prevent admin privilege escalation |
| REQ-ONBAD-010 | Tenant Admin Transfer / Owner Change | The system SHALL provide a Tenant Admin transfer workflow allowing the current Owner to designate another Tenant Admin as the new Owner, with dual confirmation (both parties confirm) and an immutable audit record. | Functional | Tenant Admin | Should | Personnel change; M&A; offboarding |
| REQ-ONBAD-011 | Admin Audit Log | All admin actions (user invite, user deactivation, license assign/revoke, config change, role grant/revoke, billing action) SHALL be recorded in an immutable admin audit log with: actor identity, action type, target entity, timestamp, IP address, before/after state. | Governance | Tenant Admin, Security Admin | Must | Compliance; SOC2/ISO27001 |
| REQ-ONBAD-012 | Admin Audit Log Retention | The admin audit log SHALL be retained for a minimum of 3 years (configurable up to 7 years), protected from deletion by any tenant admin, and exportable as signed JSON or CSV by Platform Ops. | Governance | Security Admin | Must | Regulatory compliance (SOX, GDPR, HIPAA) |
| REQ-ONBAD-013 | Admin Activity Anomaly Alerting | The system SHALL alert the Tenant Admin and Security Admin by email and in-app notification when unusual admin activity is detected, including: bulk user deactivation (>10 in 5 minutes), mass license revocation, or admin role self-assignment. | Security | Security Admin | Should | Insider threat detection |
| REQ-ONBAD-014 | Platform Ops Super-Admin | The system SHALL reserve a Platform Ops super-admin role (internal to ESIP) with cross-tenant read access for support/debugging, never granting write access without a time-limited explicit tenant-authorised break-glass session recorded in audit. | Security | Platform Ops | Must | SaaS operations; customer data governance |
| REQ-ONBAD-015 | Admin Session Timeout | Admin sessions (any admin role) SHALL have a configurable inactivity timeout (default: 30 minutes) independent of end-user session timeout, enforced by re-authentication challenge. | Security | Security Admin | Should | Privileged session hygiene |

---

## ## ONB-UM — User Management & Provisioning

### ONB-UM — Overview
User Admin creates users, assigns persona licenses, manages the full user lifecycle. Users can be provisioned manually (individual + bulk CSV) or automatically via SCIM. Teams/groups allow license and project access organisation.

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBUM-001 | Manual User Creation | The system SHALL allow a User Admin to create individual user accounts by entering first name, last name, email address, one or more persona license types (from available pool), and optional team assignment, triggering an invitation email. | Functional | User Admin | Must | Day-1 user provisioning |
| REQ-ONBUM-002 | Bulk User Import via CSV | The system SHALL accept a CSV file upload for bulk user creation, with a defined header schema (email, first_name, last_name, persona_license_types, teams), validate every row before import, report per-row errors without failing the entire batch, and display a pre-import summary (count, seat availability check). | Functional | User Admin | Must | Enterprise bulk onboarding |
| REQ-ONBUM-003 | CSV Import — Seat Availability Validation | The CSV bulk import SHALL validate seat availability for each license type in the batch as a pre-flight check, reporting exactly which rows would over-allocate which license types, and allowing partial import of rows that can be satisfied. | Functional | User Admin | Must | Prevent partial over-allocation |
| REQ-ONBUM-004 | SCIM Auto-Provisioning | The system SHALL implement the SCIM 2.0 protocol (RFC 7644) for automated user provisioning and deprovisioning from enterprise identity providers (Okta, Azure AD/Entra, Ping, etc.), including user create, update, deactivate, and group sync. | Integration | User Admin | Must | Enterprise IdP integration; eliminates manual provisioning |
| REQ-ONBUM-005 | SCIM License Mapping | The system SHALL support configurable SCIM group-to-persona-license-type mapping, so that membership in an IdP group (e.g., "ESIP-SA") automatically assigns the corresponding persona license type on user provisioning via SCIM. | Integration | User Admin | Must | Automated license assignment at scale |
| REQ-ONBUM-006 | SCIM Deprovisioning — License Reclaim | When SCIM deactivates a user, the system SHALL automatically deallocate all persona licenses held by that user, returning seats to the pool, within 60 seconds of the SCIM PATCH/DELETE event. | Integration | User Admin | Must | Automated offboarding; seat recovery |
| REQ-ONBUM-007 | User Profile — Core Fields | Each user record SHALL contain: user ID (immutable UUID), email (unique per tenant), display name, first/last name, job title, department, language/locale preference, time zone, MFA status, account status, last login, and assigned persona license types. | Data | User Admin | Must | Identity and directory record |
| REQ-ONBUM-008 | User Activate / Deactivate | The system SHALL allow a User Admin to deactivate a user account (preventing login, not deleting data or reassigning work), and reactivate it later, with deactivation taking effect within the next token refresh cycle. | Functional | User Admin | Must | HR lifecycle; offboarding |
| REQ-ONBUM-009 | User Suspend | The system SHALL support a suspended user state (stronger than deactivated) that additionally invalidates all existing session tokens immediately, preventing any cached-token access, used for security incidents. | Security | Security Admin | Must | Incident response; immediate access revocation |
| REQ-ONBUM-010 | User Deletion (GDPR Right to Erasure) | The system SHALL support permanent user deletion upon admin request, anonymising PII in audit logs (replacing with a pseudonymous identifier), archiving or transferring owned work to a designated user, and recording the deletion event. | Governance | User Admin | Must | GDPR Article 17; data subject rights |
| REQ-ONBUM-011 | License Reassignment at Deactivation | When a user is deactivated, the system SHALL prompt the User Admin to optionally immediately deallocate the user's persona licenses (returning seats to the pool) or retain assignment for a configurable hold period (default: 30 days) before automatic reclamation. | Functional | User Admin | Must | Seat management; balance between operational continuity and cost |
| REQ-ONBUM-012 | Teams / Groups | The system SHALL support creation of named teams/groups within a tenant, allowing users to be assigned to one or more teams; teams control project access visibility and can be used as units for license pool sub-allocation reporting. | Functional | User Admin | Should | Org structure; project access management |
| REQ-ONBUM-013 | Team Membership Management | The system SHALL allow a User Admin to add/remove members from a team, with changes reflected immediately in project access permissions. | Functional | User Admin | Should | Team lifecycle |
| REQ-ONBUM-014 | User Directory Search & Filter | The system SHALL provide a User Admin interface to search users by name/email, filter by status (active/deactivated/suspended/pending-invite), license type, team, last-login date, and sort/paginate results for tenants with thousands of users. | UX | User Admin | Must | Operational usability at scale |
| REQ-ONBUM-015 | User Profile Edit — Self Service | The system SHALL allow end users to edit their own profile fields (display name, job title, language, time zone, password) but NOT their own license type, admin roles, or status (admin-only operations). | Functional | All | Should | User autonomy; accurate profile data |
| REQ-ONBUM-016 | Password Reset — Admin Initiated | The system SHALL allow a User Admin to trigger a password reset email for any user, invalidating the current password and requiring the user to set a new one on next login. | Functional | User Admin | Must | Support workflow; locked-out user recovery |
| REQ-ONBUM-017 | MFA Enforcement by Admin | The system SHALL allow the Security Admin to require MFA for all users, specific admin roles, or specific user groups, with enforcement applied on next login after the policy is set. | Security | Security Admin | Must | Account security; compliance requirement |
| REQ-ONBUM-018 | MFA Reset — Admin Initiated | The system SHALL allow a Security Admin to reset a user's MFA enrollment (e.g., lost device), forcing re-enrollment on next login, with an audit record of the reset. | Security | Security Admin | Must | Operational support |
| REQ-ONBUM-019 | User Import — Idempotency | The system SHALL ensure bulk CSV and SCIM import operations are idempotent for user creation: re-importing a user with the same email does not create a duplicate but updates the existing record's non-immutable fields. | Functional | User Admin | Must | Data quality; safe re-runs |
| REQ-ONBUM-020 | User Last-Activity Tracking | The system SHALL record and display each user's last login timestamp and last active timestamp (last API or UI action) in the User Admin panel, used for inactivity detection and reclamation. | Data | User Admin | Must | Inactivity reclamation basis (REQ-ONBLR-005) |

---

## ## ONB-IV — Invite & Activation Flow

### ONB-IV — Overview
User Admins send invite emails that clearly state the assigned license type and contain a secure, time-limited activation link. The recipient completes account setup (password or SSO), activates their account, and is taken to their persona-appropriate starting view (per Vol 14 for preferences).

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBIV-001 | Invite Email Generation | The system SHALL generate an invite email for each new user containing: tenant name, inviting admin's name, the user's assigned persona license type(s), a secure single-use activation link, link expiry duration, and a brief description of ESIP. | Functional | User Admin | Must | First touchpoint; sets expectations |
| REQ-ONBIV-002 | Invite Email — License Type Stated | The invite email SHALL explicitly name each assigned persona license type (e.g., "Solution Architect") so the recipient understands their access level before accepting. | Functional | User Admin | Must | Transparency; reduces support calls |
| REQ-ONBIV-003 | Activation Link — Secure Token | The activation link SHALL be based on a cryptographically random, single-use token (minimum 256 bits entropy), transmitted over HTTPS, and valid for a configurable duration (default: 72 hours). | Security | User Admin | Must | Security; prevent link reuse |
| REQ-ONBIV-004 | Activation Link — Expiry Enforcement | The system SHALL reject an activation link attempt after its expiry duration has elapsed, display an "Activation link expired" message, and prompt the user to contact their admin for a resend. | Security | All | Must | Security hygiene |
| REQ-ONBIV-005 | Resend Invite | The system SHALL allow a User Admin to resend the activation email to a pending-invite user, generating a new token and invalidating all prior pending tokens for that user. | Functional | User Admin | Must | Operational recovery; missed email |
| REQ-ONBIV-006 | Revoke Invite | The system SHALL allow a User Admin to revoke a pending invite before activation, immediately invalidating the activation token, and setting the user record to a "revoked" status without consuming a license seat. | Functional | User Admin | Must | Change of plan before activation |
| REQ-ONBIV-007 | First-Login — Set Password | For non-SSO tenants, activating the invite link SHALL direct the user to a set-password screen enforcing the tenant's current password policy (length, complexity, history); on success, the account is fully activated and the user lands in the application. | Functional | All | Must | Account activation |
| REQ-ONBIV-008 | First-Login — SSO Handoff | For SSO-enabled tenants, the activation link SHALL redirect the user to the tenant's configured SSO provider for authentication; on successful SSO response, the account is activated and the user lands in the application. | Integration | All | Must | Enterprise SSO onboarding |
| REQ-ONBIV-009 | Email Verification | The system SHALL verify the user's email address either via the activation link click (implicitly confirming email ownership) or via a separate OTP if an alternative email is set post-activation. | Security | All | Must | Email ownership; anti-abuse |
| REQ-ONBIV-010 | Pending Invite Visibility | The system SHALL display all pending (not-yet-activated) invites in the User Admin panel with: invitee email, assigned license types, sent date, expiry date, and actions (resend, revoke). | UX | User Admin | Must | Admin visibility; invite management |
| REQ-ONBIV-011 | Invite Batch | The system SHALL support batch invite dispatch from the CSV import flow, sending individual invite emails to all successfully imported users in a single admin action. | Functional | User Admin | Must | Efficiency for bulk onboarding |
| REQ-ONBIV-012 | Post-Activation Landing | After account activation, the system SHALL land the user on an appropriate starting screen for their primary persona (not a blank screen), consistent with the persona-specific onboarding experience defined in Vol 14. | UX | All | Must | Activation → first value; reduces drop-off |
| REQ-ONBIV-013 | Self-Registration with Domain Restriction | The system SHALL optionally allow users with an email matching the tenant's verified domain(s) to self-register without an explicit invite, subject to Tenant Admin enabling this option, with auto-assignment of a default license type configured by the admin. | Functional | Tenant Admin | Should | Low-friction adoption for internal rollouts |
| REQ-ONBIV-014 | Invite Audit Record | Each invite sent, resent, revoked, and activated SHALL generate an immutable audit record (actor, target email, license types, token hash, event type, timestamp). | Governance | User Admin | Must | Compliance |

---

## ## ONB-DB — Admin Config — Database Provider

### ONB-DB — Overview
Each tenant's database configuration is set during onboarding (REQ-ONBOB-007) and managed post-provisioning via the Admin Configuration Zone. The system uses a ports-and-adapters pattern (EF Core with provider plug-in) to support PostgreSQL, SQL Server, MySQL, and Oracle, per the decided stack.

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBDB-001 | Database Provider Selection | The system SHALL allow Tenant Admin / Config Admin to select the tenant's database provider from: PostgreSQL, SQL Server, MySQL 8+, Oracle 19c+, and record this selection immutably for use by the EF Core provider plug-in. | Functional | Config Admin | Must | Client-configurable DB requirement |
| REQ-ONBDB-002 | BYOD Connection String | The system SHALL accept a full connection string (or individual host/port/database/username/password/SSL-mode fields) for the tenant's database, encrypt it at rest using the tenant's encryption key, and use it for all tenant data operations. | Functional | Config Admin | Must | BYOD; customer controls their data location |
| REQ-ONBDB-003 | Managed Database Option | The system SHALL offer a ESIP-managed hosted database option (per cloud region) as an alternative to BYOD, provisioned automatically, with connection details stored internally without being visible to the Tenant Admin. | Functional | Config Admin | Should | Ease of use for cloud-native deployments |
| REQ-ONBDB-004 | Connection Test | The system SHALL provide a "Test Connection" action that validates the provided connection string against the live database (connectivity, authentication, required-permission check) and returns a pass/fail result with error detail before saving. | Functional | Config Admin | Must | Reduce misconfiguration-related support tickets |
| REQ-ONBDB-005 | Connection String Encryption | The database connection string SHALL be stored encrypted using an AES-256 key derived from the tenant's master encryption key (see Vol 11); decrypted only in-memory at the time of database access, never logged in plaintext. | Security | Security Admin | Must | Credential security; SOC2 CC6.1 |
| REQ-ONBDB-006 | Connection String Rotation | The system SHALL allow a Config Admin to update the database connection string (e.g., on password rotation) with zero-downtime connection pool drain-and-reconnect, without service interruption. | Functional | Config Admin | Must | Operational: DB credential rotation is a standard security practice |
| REQ-ONBDB-007 | Database Schema Migration | When a new ESIP release requires schema changes, the system SHALL apply schema migrations to the tenant's database using EF Core migrations on service startup, with pre-flight dry-run option and rollback plan recorded. | Functional | Platform Ops | Must | Multi-tenant schema lifecycle |
| REQ-ONBDB-008 | Database Config Audit | Every change to the database configuration (provider change, connection string update, migration applied) SHALL be recorded in the admin audit log with actor, timestamp, and change summary (not including plaintext credentials). | Governance | Config Admin | Must | Audit; change management |
| REQ-ONBDB-009 | Database Health Monitoring | The system SHALL continuously monitor database connectivity health for each tenant, alert Config Admin and Tenant Admin if the database becomes unreachable, and surface the status in the Admin dashboard. | NFR | Config Admin | Should | Operational visibility; early warning |
| REQ-ONBDB-010 | Multi-Region Read Replica Config | The system SHALL allow configuration of optional read replicas (host/port) per tenant for read-heavy workloads, with the application routing read queries to replicas when available. | Functional | Config Admin | Could | Performance; advanced deployments |

---

## ## ONB-CL — Admin Config — Cloud Account & Region

### ONB-CL — Overview
Tenant workloads, file storage, and AI requests are cloud-neutral. The Admin Configuration Zone lets each tenant register cloud account credentials and choose deployment region, which governs where tenant data at rest and compute operate.

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBCL-001 | Cloud Provider Selection | The system SHALL allow Config Admin to designate the cloud provider for the tenant's deployment: AWS, Azure, GCP, IBM Cloud, or on-premises, recorded at tenant provisioning and governing all subsequent cloud resource allocation for that tenant. | Functional | Config Admin | Must | Cloud-neutral architecture requirement |
| REQ-ONBCL-002 | Cloud Region Selection | The system SHALL allow Config Admin to select the cloud region (from a provider-specific list of available ESIP-supported regions) for tenant data storage and compute, enforcing data residency if configured. | Functional | Config Admin | Must | Data residency; latency; compliance |
| REQ-ONBCL-003 | Cloud Account / Credentials Config | The system SHALL allow Config Admin to provide cloud account credentials (AWS IAM Role ARN for assume-role, Azure Service Principal, GCP Service Account key, IBM API key) for ESIP's use in the tenant's cloud account (e.g., file storage, AI service proxying), stored encrypted. | Functional | Config Admin | Should | BYOC (Bring Your Own Cloud) model; tenant controls billing |
| REQ-ONBCL-004 | ESIP-Managed Cloud Option | The system SHALL support an ESIP-managed cloud option where ESIP provisions cloud resources on behalf of the tenant within ESIP's own cloud accounts, with the tenant selecting region only, as the default for lower-tier editions. | Functional | Config Admin | Must | Lower-tier simplicity; managed SaaS |
| REQ-ONBCL-005 | Cloud Credential Validation | The system SHALL validate provided cloud credentials (test IAM/RBAC permissions against required resource types) before saving, reporting missing permissions with actionable remediation instructions. | Functional | Config Admin | Must | Reduce config errors; clear onboarding experience |
| REQ-ONBCL-006 | Cloud Storage Bucket / Container Config | The system SHALL allow Config Admin to specify the cloud storage bucket (S3/Azure Blob/GCS bucket name and access path) for tenant file storage, with a connectivity and permission test. | Functional | Config Admin | Must | BYOC file storage |
| REQ-ONBCL-007 | Cloud Config Audit | All changes to cloud provider selection, region, credentials, and storage config SHALL be recorded in the admin audit log. | Governance | Config Admin | Must | Change management; compliance |
| REQ-ONBCL-008 | Cloud Config Credential Rotation | The system SHALL allow Config Admin to rotate cloud credentials with zero-downtime (completing in-flight requests before switching to new credentials). | Functional | Config Admin | Should | Security hygiene; credential rotation policy |
| REQ-ONBCL-009 | On-Premises Deployment Mode | The system SHALL support an on-premises deployment configuration mode where cloud provider is "none", all file storage and compute are local/self-hosted, and cloud-specific features are gracefully disabled with appropriate messaging. | Functional | Config Admin | Should | Air-gapped enterprise; private-cloud deployments |
| REQ-ONBCL-010 | Multi-Cloud Config (advanced) | The system SHALL support configuring separate cloud providers for different tenant workloads (e.g., storage on Azure, AI compute on AWS), with routing rules defined in the Admin Configuration Zone. | Functional | Config Admin | Could | Multi-cloud enterprise architectures |

---

## ## ONB-AK — Admin Config — BYOK AI Keys

### ONB-AK — Overview
A tenant may supply their own API keys for AI providers (AWS Bedrock, Azure OpenAI, Google Vertex AI, IBM watsonx, or local/self-hosted models). These keys are stored as tenant secrets (encrypted, not readable after save), consumed by the AI proxy layer (Vol 06) for all AI operations on behalf of that tenant. Per-key budget limits enforce cost control.

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBAK-001 | BYOK AI Provider — Supported Providers | The system SHALL support BYOK (Bring Your Own Key) configuration for the following AI providers: AWS Bedrock (IAM Role or Access Key), Azure OpenAI (API key + endpoint), Google Vertex AI (Service Account), IBM watsonx (API key + project ID), and a generic OpenAI-compatible endpoint (URL + key) for local/self-hosted models. | Integration | Config Admin | Must | Provider-pluggable AI; client controls AI cost & data policy |
| REQ-ONBAK-002 | Multiple AI Provider Configs | The system SHALL allow Config Admin to configure multiple AI provider connections simultaneously, enabling the AI routing layer (Vol 06) to select or fallback between providers per tenant policy. | Integration | Config Admin | Should | Resilience; cost optimisation; multi-provider routing |
| REQ-ONBAK-003 | AI Key Secure Storage | Each AI provider key/credential SHALL be stored as a tenant secret, encrypted with the tenant's master encryption key, never returned in plaintext via any API response after save (only a masked preview, e.g., last 4 characters). | Security | Security Admin | Must | Credential security; SOC2 |
| REQ-ONBAK-004 | AI Key — Test Connectivity | The system SHALL provide a "Test API Key" action for each configured AI provider, performing a minimal API call to validate authentication and model accessibility, and reporting success or a provider-specific error before saving. | Functional | Config Admin | Must | Reduce misconfiguration; fast feedback |
| REQ-ONBAK-005 | Per-Key Monthly Budget Limit | The system SHALL allow Config Admin to set a monthly spend/credit budget limit per configured AI key (in USD or token count), with the AI proxy tracking usage against this limit and enforcing a hard stop or soft-warn at configurable thresholds (80%, 100%). | Functional | Config Admin | Must | Cost control; prevent runaway AI spend |
| REQ-ONBAK-006 | Per-Key Budget Notifications | When AI key usage reaches 80% and 100% of the configured budget, the system SHALL send an email notification to Config Admin and Tenant Admin, and display an in-app alert in the Admin dashboard. | Functional | Config Admin | Must | Cost awareness; avoids surprise bills |
| REQ-ONBAK-007 | AI Key Rotation | The system SHALL allow Config Admin to rotate an AI provider key (replace with a new key value) without service interruption, with the old key invalidated in the secret store immediately after successful validation of the new key. | Security | Config Admin | Should | Key rotation hygiene |
| REQ-ONBAK-008 | AI Key Deletion | The system SHALL allow Config Admin to delete an AI provider configuration, removing the key from the secret store and disabling that provider for the tenant's AI operations; the system SHALL alert if the deleted provider is the sole configured provider. | Functional | Config Admin | Must | Key offboarding |
| REQ-ONBAK-009 | ESIP Default AI Keys (Fallback) | The system SHALL support a fallback mode where, if no BYOK keys are configured, the tenant's AI operations use ESIP's platform-level AI keys subject to the edition's AI credit quota, transparently billed through the subscription. | Functional | Config Admin | Must | Out-of-box experience for new tenants; graduated adoption |
| REQ-ONBAK-010 | AI Key Usage Report | The system SHALL provide a per-AI-provider, per-key usage report (token counts, estimated cost, requests, error rate) per billing period, accessible to Config Admin and Billing Admin. | Reporting | Config Admin, Billing Admin | Should | Cost visibility; FinOps |
| REQ-ONBAK-011 | AI Key Audit Log | All AI key configuration events (add, test, update, rotate, delete, budget change) SHALL be recorded in the admin audit log with actor, timestamp, provider name, and action (not key value). | Governance | Config Admin | Must | Audit; SOC2 |
| REQ-ONBAK-012 | Local / Self-Hosted AI Endpoint | The system SHALL accept an OpenAI-compatible base URL (e.g., Ollama, vLLM, Azure AI Studio endpoint) as a BYOK target, allowing fully air-gapped / on-premises AI for tenants with data-sovereignty requirements. | Integration | Config Admin | Should | Air-gapped enterprise; data sovereignty |

---

## ## ONB-GT — Admin Config — Git Targets

### ONB-GT — Overview
A key ESIP differentiator: generated deliverables, diagrams, ADRs, and code can be committed to the client's own source control system as part of the architecture-as-code workflow. Tenant admins register Git targets (ADO, GitHub, GitLab, Bitbucket) that project owners can map to their projects. The output formatting and PR templating is handled in Vol 10; this volume covers registration and credential management only.

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBGT-001 | Git Target — Supported Providers | The system SHALL support registration of Git targets for: Azure DevOps (org/project/repo), GitHub (org/repo), GitLab (group/project), and Bitbucket (workspace/repo), with OAuth or PAT authentication options for each. | Integration | Config Admin | Must | Client's own repos; architecture-as-code |
| REQ-ONBGT-002 | Multiple Git Targets | The system SHALL allow Config Admin to register multiple Git targets of the same or different providers (e.g., two GitHub repos, one ADO project), each with a unique tenant-scoped name and config. | Integration | Config Admin | Should | Multiple product lines / teams |
| REQ-ONBGT-003 | OAuth Authentication for Git | For GitHub, GitLab, and Bitbucket, the system SHALL support OAuth 2.0 app authorization flow, storing the resulting access/refresh tokens as tenant secrets and refreshing them automatically before expiry. | Security | Config Admin | Must | Secure, no-password-sharing auth |
| REQ-ONBGT-004 | PAT Authentication for Git | The system SHALL accept a Personal Access Token (PAT) as an alternative authentication method for all four Git providers, storing the PAT as a tenant secret and validating it with a test API call before saving. | Security | Config Admin | Must | Service account / automation use case; ADO preference |
| REQ-ONBGT-005 | Git Target — Connection Test | The system SHALL provide a "Test Connection" action per registered Git target that verifies authentication and required permissions (read repo, create branch, create PR, push commits), reporting each permission check result. | Functional | Config Admin | Must | Reduce misconfiguration at registration time |
| REQ-ONBGT-006 | Default Branch Configuration | Each Git target registration SHALL include a configurable default target branch (e.g., `main`, `develop`) for commits/PRs from ESIP; overridable at project-level mapping. | Functional | Config Admin | Must | Branch strategy control |
| REQ-ONBGT-007 | PR Strategy Configuration | Each Git target SHALL have a configurable PR strategy: "create PR" (default), "direct commit to branch", or "create draft PR"; overridable per project. | Functional | Config Admin | Should | Fits varied GitFlow / trunk-based strategies |
| REQ-ONBGT-008 | Per-Project Git Target Mapping | The system SHALL allow project owners (or User Admins) to map a specific registered Git target to a project, designating the target org/repo and path prefix for ESIP-generated artifacts committed from that project. | Functional | Config Admin, All | Must | Project-level control; avoids org-wide blanket target |
| REQ-ONBGT-009 | Git Credential Secure Storage | All Git authentication credentials (PAT, OAuth tokens) SHALL be stored as tenant secrets, encrypted at rest, never returned in any API response, and never logged. | Security | Security Admin | Must | Credential security |
| REQ-ONBGT-010 | Git Credential Rotation | The system SHALL allow Config Admin to rotate a Git target's PAT (replace + re-validate) without removing the target configuration or affecting project mappings. | Security | Config Admin | Should | Credential hygiene; PAT expiry cycles |
| REQ-ONBGT-011 | Git Target Delete | The system SHALL allow Config Admin to delete a Git target, alerting if any projects are currently mapped to it and requiring confirmation, with project mappings cleared on delete. | Functional | Config Admin | Must | Lifecycle management |
| REQ-ONBGT-012 | Git Target Audit | All Git target config events (add, test, update, delete, credential rotation) SHALL be recorded in the admin audit log. | Governance | Config Admin | Must | Audit |
| REQ-ONBGT-013 | Azure DevOps — Work Item Integration | For ADO Git targets, the system SHALL optionally accept an ADO Work Item ID or area path prefix to link ESIP-generated PRs to ADO work items, surfaced in the project-level mapping config. | Integration | Config Admin | Could | ADO traceability; enterprise integration |
| REQ-ONBGT-014 | Webhook Registration | The system SHALL optionally register a webhook on the configured repository (where the Git provider API supports it) to receive PR status events (merged, closed, declined), updating the artifact's publication status in ESIP. | Integration | Config Admin | Could | Round-trip status visibility |

---

## ## ONB-KC — Admin Config — Knowledge/SaaS Connectors (org-level auth)

### ONB-KC — Overview
Org-wide connector authorizations are managed by Config Admin, establishing the tenant-level OAuth/API-key credentials for enterprise connectors (Jira, Confluence, SharePoint, ADO, ServiceNow, etc.). Project-level connector enablement and crawl configuration are handled in Vol 08. This section covers registration and credential management only.

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBKC-001 | Connector Registry — List of Supported Connectors | The system SHALL display a connector registry in the Admin Configuration Zone listing all supported connectors (Jira, Confluence, SharePoint/M365, Azure DevOps, ServiceNow, GitHub, GitLab, Bitbucket, Salesforce, Google Drive, Slack, Teams, Miro, LucidChart, Figma, and others defined in Vol 08), with add/configure/status actions per connector. | Functional | Config Admin | Must | Discoverability; managed connector lifecycle |
| REQ-ONBKC-002 | OAuth Org-Level Authorization | For OAuth-capable connectors (Jira/Atlassian, M365/SharePoint, Salesforce, Slack, Google Drive, Miro, Figma), the system SHALL support an org-level OAuth 2.0 authorization flow, storing resulting tokens as tenant secrets at the org level, usable by all projects within the tenant. | Integration | Config Admin | Must | Single-auth for all projects; reduce per-user re-auth |
| REQ-ONBKC-003 | API Key / Service Account Config | For connectors using API keys or service accounts (ServiceNow, GitHub PAT, ADO PAT), the system SHALL accept and securely store the org-level credential. | Integration | Config Admin | Must | Service-account pattern for automation |
| REQ-ONBKC-004 | Connector Connection Test | Each registered connector SHALL provide a "Test Connection" action validating authentication and required read/write permissions, with per-permission pass/fail detail. | Functional | Config Admin | Must | Reduce misconfiguration |
| REQ-ONBKC-005 | Connector Credential Rotation | The system SHALL allow Config Admin to rotate connector credentials (OAuth re-authorize or new API key), with the old credential invalidated after successful validation of the new one. | Security | Config Admin | Should | Credential lifecycle |
| REQ-ONBKC-006 | Connector Enable/Disable | The system SHALL allow Config Admin to enable or disable individual connectors at the org level, with disabled connectors unavailable for project-level activation. | Functional | Config Admin | Must | Governance; scope control |
| REQ-ONBKC-007 | Connector Credential Secure Storage | All connector credentials (OAuth tokens, API keys) SHALL be stored as tenant secrets, encrypted at rest, never returned in plaintext after save. | Security | Security Admin | Must | Credential security |
| REQ-ONBKC-008 | Connector Audit | All connector config events (add, authorize, test, rotate, enable, disable, delete) SHALL be recorded in the admin audit log. | Governance | Config Admin | Must | Audit |
| REQ-ONBKC-009 | Connector Token Expiry Monitoring | The system SHALL monitor OAuth token expiry for all org-level connector authorizations, proactively refresh tokens where the provider supports refresh tokens, and alert Config Admin by email and in-app notification when a token cannot be refreshed and requires re-authorization. | Functional | Config Admin | Must | Prevent silent connector failures |

---

## ## ONB-BX — Admin Config — Branding, Email/SMTP & Theme

### ONB-BX — Overview
Tenants can brand the ESIP interface with their org's logo, colours, and name. Custom SMTP settings allow invite/notification emails to originate from the client's own domain. Theme selection affects all end users in the tenant.

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBBX-001 | Tenant Logo Upload | The system SHALL allow Config Admin to upload a tenant logo (PNG/SVG, max 2 MB) displayed in the application header, login page, and email templates for all users of the tenant. | Functional | Config Admin | Should | Brand identity; enterprise whitelabelling |
| REQ-ONBBX-002 | Tenant Name Display | The system SHALL display the tenant's configured organisation name (from onboarding profile) in the application header, login page, email subject lines, and generated document headers. | Functional | Config Admin | Should | Brand identity |
| REQ-ONBBX-003 | Primary Brand Colour | The system SHALL allow Config Admin to configure a primary brand hex colour applied to key UI accent elements (buttons, highlights, active states) using the Tailwind v4 CSS variable theming system. | UX | Config Admin | Could | Visual branding; enterprise customisation |
| REQ-ONBBX-004 | Theme Selection (Light/Dark/System) | The system SHALL support Light, Dark, and System (follows OS) themes at the tenant level (default for all users), while allowing individual users to override in their own preferences (Vol 14). | UX | Config Admin | Should | Accessibility; developer audience prefers dark |
| REQ-ONBBX-005 | Custom SMTP Configuration | The system SHALL allow Config Admin to configure a custom SMTP server (host, port, TLS mode, username, password) so that all system-generated emails (invites, notifications, password reset, reports) originate from the client's own email domain. | Integration | Config Admin | Should | Brand trust; enterprise mail policy compliance |
| REQ-ONBBX-006 | Custom SMTP — Test Send | The system SHALL provide a "Send Test Email" action for the custom SMTP config that sends a test message to the Config Admin's email address and confirms delivery, before saving the SMTP configuration. | Functional | Config Admin | Should | Reduce misconfiguration |
| REQ-ONBBX-007 | Custom Email From-Name and Address | The system SHALL allow Config Admin to set the "From" display name and email address used in all system emails (e.g., "ACME Architecture Suite <noreply@acme.com>"). | Functional | Config Admin | Should | Brand identity; email trust |
| REQ-ONBBX-008 | Fallback to ESIP SMTP | If custom SMTP is not configured or fails, the system SHALL silently fall back to ESIP's own SMTP service for email delivery, logging the fallback event in the admin audit log. | Functional | Platform Ops | Must | Operational continuity; ensure invites are delivered |
| REQ-ONBBX-009 | Branding Audit | All branding and email config changes SHALL be recorded in the admin audit log. | Governance | Config Admin | Should | Change management |

---

## ## ONB-SP — Admin Config — Security Policy

### ONB-SP — Overview
Security Admin configures tenant-level security policies: password rules, session timeout, MFA requirements, SSO settings, allowed IP ranges. Internals (SAML/OIDC implementation, session store, token signing) are in Vol 11.

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBSP-001 | Password Policy Configuration | The system SHALL allow Security Admin to configure the tenant password policy: minimum length (8–32), required character classes (uppercase, lowercase, digit, special), maximum age (days, 0 = no expiry), password history (prevent reuse of last N passwords). | Security | Security Admin | Must | Defence-in-depth; compliance baseline |
| REQ-ONBSP-002 | Session Idle Timeout | The system SHALL allow Security Admin to configure the session idle timeout (15 min to 24 h, default 60 min) after which inactive sessions are invalidated and the user must re-authenticate. | Security | Security Admin | Must | Session security; PCI-DSS, ISO 27001 |
| REQ-ONBSP-003 | Absolute Session Lifetime | The system SHALL allow Security Admin to configure an absolute session lifetime (1 h to 30 days, default 12 h) after which any session must be re-authenticated regardless of activity. | Security | Security Admin | Should | Long-lived session risk mitigation |
| REQ-ONBSP-004 | MFA Policy | The system SHALL allow Security Admin to enforce MFA (mandatory, optional, or disabled) tenant-wide or per user group, supporting TOTP (RFC 6238) and WebAuthn/FIDO2 as MFA factors; SSO-based MFA can be accepted in lieu of in-app MFA when SSO is configured. | Security | Security Admin | Must | Account security; compliance |
| REQ-ONBSP-005 | SSO / SAML 2.0 Configuration | The system SHALL allow Security Admin to configure SAML 2.0 SSO by providing IdP metadata URL or XML, attribute mapping (email, name, groups), and selecting Just-In-Time (JIT) provisioning option; implementation details in Vol 11. | Integration | Security Admin | Must | Enterprise SSO; AD/Okta/Ping integration |
| REQ-ONBSP-006 | SSO / OIDC Configuration | The system SHALL allow Security Admin to configure OpenID Connect SSO by providing issuer URL, client ID, client secret (stored encrypted), and scopes; implementation details in Vol 11. | Integration | Security Admin | Must | Modern SSO; Entra ID, Google Workspace, Auth0 |
| REQ-ONBSP-007 | SSO Enforcement Mode | The system SHALL allow Security Admin to enforce SSO-only login (disabling email/password login for all non-admin users), or to allow both SSO and local passwords, switchable per policy. | Security | Security Admin | Must | Enterprise compliance; policy control |
| REQ-ONBSP-008 | Allowed IP Ranges (IP Allowlisting) | The system SHALL allow Security Admin to configure a list of allowed IP CIDR ranges; if configured, login and API requests from IPs outside these ranges SHALL be rejected with a clear message (referencing Vol 11 for enforcement details). | Security | Security Admin | Should | VPN enforcement; corporate network policy |
| REQ-ONBSP-009 | Account Lockout Policy | The system SHALL allow Security Admin to configure account lockout: number of failed login attempts before lockout (3–10), lockout duration (5 min to permanent), and unlock mechanism (automatic timeout or admin unlock). | Security | Security Admin | Must | Brute-force defence |
| REQ-ONBSP-010 | Concurrent Login Policy | The system SHALL allow Security Admin to configure whether a user may have multiple simultaneous active sessions (default: allowed) or is restricted to a single session (new login terminates prior session). | Security | Security Admin | Should | Session control; shared-credential detection |
| REQ-ONBSP-011 | Security Policy Audit | All changes to security policy settings SHALL be recorded in the admin audit log with before/after values, actor, and timestamp. | Governance | Security Admin | Must | Compliance; change control |
| REQ-ONBSP-012 | Security Policy Effective Notification | When a security policy change (MFA enforcement, password policy tightening, SSO enforcement) is saved, the system SHALL notify all active Tenant Admins by email of the change and its effective time. | Functional | Tenant Admin | Should | Operational awareness; prevent user lockouts |

---

## ## ONB-DR — Admin Config — Data Residency & Retention

### ONB-DR — Overview
Enterprise tenants require control over where data is stored and how long it is retained. This section governs the admin-facing configuration for data residency commitments and retention policies. Enforcement internals are in Vol 11.

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBDR-001 | Data Residency Region Commitment | The system SHALL allow Config Admin to configure and display the tenant's committed data residency region (matching the cloud region chosen at onboarding), with the system enforcing that no tenant data (files, DB records, AI request/response logs) is stored outside this region. | Governance | Config Admin, Security Admin | Must | GDPR; data localisation laws; enterprise procurement requirement |
| REQ-ONBDR-002 | Data Residency Certification | The system SHALL display a data residency certificate (region, provider, effective date) in the Admin Configuration Zone, downloadable as PDF for inclusion in vendor security questionnaires. | Governance | Security Admin | Should | Customer trust; procurement support |
| REQ-ONBDR-003 | Document Retention Policy | The system SHALL allow Config Admin to configure retention periods for different data classes (uploaded documents: min 1 year, max 10 years; AI generation logs: min 90 days, max 5 years; audit logs: 3–7 years), with automated purge of data exceeding the retention period. | Governance | Config Admin, Security Admin | Must | GDPR data minimisation; legal hold; corporate policy |
| REQ-ONBDR-004 | Legal Hold | The system SHALL support placing a legal hold on a tenant or specific project, suspending all automated retention purges until the hold is lifted, accessible only to Platform Ops and (optionally) Tenant Admin. | Governance | Security Admin | Should | Litigation support; e-discovery |
| REQ-ONBDR-005 | Data Export (Tenant Portability) | The system SHALL allow Tenant Admin to request a full tenant data export (all projects, diagrams, documents, requirements, deliverables, audit logs) as a structured ZIP/archive, fulfilling GDPR data portability rights, completed within 72 hours. | Governance | Tenant Admin | Must | GDPR Article 20; vendor lock-in mitigation |
| REQ-ONBDR-006 | Tenant Deletion / Right to Erasure | Upon a verified tenant deletion request, the system SHALL purge all tenant data from all stores (DB, file storage, caches, backups within retention window), generating a deletion certificate with completion timestamp. | Governance | Tenant Admin, Platform Ops | Must | GDPR Article 17; contractual obligation |
| REQ-ONBDR-007 | AI Data Logging Policy | The system SHALL allow Config Admin to configure whether AI request payloads and responses are logged for debugging/audit (default: on, 30-day retention) or disabled (no AI payload logging), respecting data sensitivity requirements. | Governance | Config Admin | Should | Data privacy; AI data governance |
| REQ-ONBDR-008 | Retention Policy Audit | All data residency and retention policy changes SHALL be recorded in the admin audit log. | Governance | Config Admin | Must | Compliance |

---

## ## ONB-BL — Billing, Metering & Seat Management

### ONB-BL — Overview
Light billing layer (detailed billing engine is an integration concern). Covers usage dashboards, invoice management, plan upgrades, seat purchasing, true-up, and integration with external billing systems (Stripe, etc.).

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBBL-001 | Billing Dashboard | The system SHALL provide a Billing Admin dashboard showing: current edition, active subscription term, next renewal date, per-license-type purchased/assigned/available counts, current period AI credit usage, storage usage, and estimated next invoice. | Reporting | Billing Admin | Must | Central billing visibility |
| REQ-ONBBL-002 | Invoice History | The system SHALL display a paginated invoice history (date, invoice number, line items, amount, status: paid/unpaid/overdue) with downloadable PDF per invoice for the last 36 months. | Reporting | Billing Admin | Must | Finance record-keeping; accounts payable |
| REQ-ONBBL-003 | Payment Method Management | The system SHALL allow Billing Admin to add, remove, and set-default payment methods (credit card via PCI-compliant tokenisation, or wire/invoice for Enterprise), and update billing address. | Functional | Billing Admin | Must | Self-serve payment management |
| REQ-ONBBL-004 | External Billing Integration — Stripe | The system SHALL support integration with Stripe as the default payment processing and subscription management backend, using Stripe's webhooks for payment events (success, failure, chargeback) to update tenant subscription status in near-real-time. | Integration | Platform Ops | Must | Standard SaaS billing; PCI compliance via Stripe |
| REQ-ONBBL-005 | External Billing Integration — Enterprise Invoicing | The system SHALL support an enterprise invoicing mode (PO number, net-30/60 payment terms) for tenants on Enterprise plans, integrated with ESIP's accounts receivable workflow, with overdue invoice escalation logic. | Integration | Billing Admin, Platform Ops | Should | Enterprise procurement; no credit-card-only constraint |
| REQ-ONBBL-006 | Seat Purchase Self-Serve | The system SHALL allow Billing Admin to purchase additional seats for any persona license type directly from the Billing dashboard, with prorated billing calculation displayed before confirmation, and immediate seat pool increase on payment. | Functional | Billing Admin | Must | Organic growth; self-serve commercial |
| REQ-ONBBL-007 | Usage Export — Billing Period | The system SHALL allow Billing Admin to export a detailed usage report for any billing period as CSV, covering: per-user login counts, AI calls by type, storage delta, license days active per seat, and overage events. | Reporting | Billing Admin | Must | True-up evidence; ROI reporting |
| REQ-ONBBL-008 | Overage True-Up Invoice | When a true-up period closes, the system SHALL generate an overage true-up line item in the next invoice (or a standalone invoice), itemised by license type and overage quantity × unit price, visible in the invoice detail view. | Functional | Billing Admin | Should | True-up model integrity |
| REQ-ONBBL-009 | Cost Allocation Tags | The system SHALL allow Billing Admin to assign cost allocation tags (e.g., business unit, cost centre) to license pools or teams, included in billing exports for internal chargeback/showback reporting. | Reporting | Billing Admin | Could | FinOps; enterprise cost allocation |
| REQ-ONBBL-010 | Payment Failure Handling | The system SHALL send automated email notifications to Billing Admin on payment failure, retry payment per the configured schedule (e.g., Day 1, Day 3, Day 7), enter a grace period if retries are exhausted, and escalate to Tenant Admin if not resolved within the grace period. | Functional | Billing Admin | Must | Revenue recovery; SaaS billing standard |
| REQ-ONBBL-011 | Plan Change — Proration | When a Billing Admin upgrades or downgrades the plan mid-cycle, the system SHALL calculate and display a prorated credit/charge amount before confirmation, applying it to the current invoice or next invoice per configuration. | Functional | Billing Admin | Should | Customer fairness; billing accuracy |
| REQ-ONBBL-012 | Spend Limit Alert | The system SHALL allow Billing Admin to configure a monthly spend alert threshold; when estimated total charges exceed the threshold, an email alert is sent to Billing Admin. | Functional | Billing Admin | Could | Budget governance |

---

## ## ONB-RP — License Compliance & Reporting

### ONB-RP — Overview
Admin-facing reports specifically focused on license compliance, seat utilisation, and audit-readiness. Complements the licensing rules in ONB-LR and the billing in ONB-BL.

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBRP-001 | License Utilisation Report | The system SHALL provide a real-time license utilisation report per license type (purchased, assigned, active last 30d, inactive, pending activation, reclaimed, expiring within 30d) accessible to Tenant Admin, User Admin, and Billing Admin. | Reporting | Tenant Admin, Billing Admin | Must | Operational visibility; cost management |
| REQ-ONBRP-002 | Seat Efficiency Score | The system SHALL compute and display a per-license-type seat efficiency score (active/assigned ratio, based on configurable active-usage threshold), with recommendations (reduce seats if <60% active, add seats if >90% assigned). | Reporting | Billing Admin | Could | FinOps optimisation |
| REQ-ONBRP-003 | Compliance Snapshot Export | The system SHALL allow on-demand export of a license compliance snapshot as PDF/CSV (point-in-time: licence type, purchased, assigned, active, overage, expiry), suitable for software asset management (SAM) audit. | Reporting | Tenant Admin | Must | SAM; enterprise procurement; audit evidence |
| REQ-ONBRP-004 | Per-User License Activity Report | The system SHALL provide a per-user activity report (last login, licensed actions count by type, AI calls, deliverables generated, last active project) per billing period, filterable by license type and team. | Reporting | User Admin | Should | Usage evidence; HR / project review |
| REQ-ONBRP-005 | Scheduled License Report | The system SHALL allow Tenant Admin to schedule automated license utilisation reports (monthly, quarterly) for email delivery to designated recipients (Billing Admin, finance contacts). | Reporting | Tenant Admin | Should | Passive visibility; no manual pulls required |
| REQ-ONBRP-006 | Expiry Forecast Report | The system SHALL generate an expiry forecast showing, for each license type, the term expiry date, assigned users, and risk level (red: expiry < 30 days, amber: 30–90 days, green: > 90 days). | Reporting | Billing Admin | Must | Renewal planning |
| REQ-ONBRP-007 | Cross-Tenant License Comparison (Platform Ops) | Platform Ops SHALL have access to a cross-tenant license summary report (anonymised or named, per ESIP policy) for identifying at-risk renewals, underutilised seats, or overage exposure across the fleet. | Reporting | Platform Ops | Should | ESIP commercial operations |

---

## ## ONB-AU — Onboarding & Admin Audit Log

### ONB-AU — Overview
Audit logging is foundational across all admin areas. This section consolidates the universal audit log requirements for the onboarding and admin domain, referencing the platform-level audit storage and retention specified in Vol 11.

---

| ID | Title | Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------|----------|------------|----------|-----------|
| REQ-ONBAU-001 | Unified Admin Audit Log | The system SHALL maintain a unified, append-only audit log for all admin domain events (onboarding steps, user lifecycle, license operations, config changes, billing events, invite events, policy changes) at the tenant scope, conforming to the platform audit log schema defined in Vol 11. | Governance | Tenant Admin, Security Admin | Must | SOC2; ISO 27001; GDPR |
| REQ-ONBAU-002 | Audit Record Schema | Every audit record SHALL contain: event_id (UUID), tenant_id, event_type (namespaced: `onb.user.invite_sent`, `onb.license.assigned`, etc.), actor_id, actor_ip, target_entity_type, target_entity_id, timestamp (UTC ISO 8601), outcome (success/failure), before_state (JSON), after_state (JSON), correlation_id. | Data | Security Admin | Must | Structured, queryable, importable audit data |
| REQ-ONBAU-003 | Audit Log Search & Filter | The system SHALL provide a Tenant Admin / Security Admin UI to search the audit log by: date range, event type, actor, target entity, outcome — with pagination for large result sets. | UX | Security Admin | Must | Incident investigation |
| REQ-ONBAU-004 | Audit Log Export | The system SHALL allow Security Admin to export audit log entries for a date range as signed JSON-Lines or CSV, with a cryptographic signature verifiable against ESIP's signing certificate, for use as tamper-evident evidence. | Governance | Security Admin | Must | Regulatory submission; external audit |
| REQ-ONBAU-005 | Audit Log Immutability | The admin audit log SHALL be immutable: no admin role at any level (including Tenant Admin and Platform Ops) shall be able to delete or modify audit records; deletion is only possible by automated retention purge after the configured retention period. | Security | Platform Ops | Must | Tamper-evidence; compliance requirement |
| REQ-ONBAU-006 | Audit Log SIEM Integration | The system SHALL support streaming the admin audit log to an external SIEM/logging platform (Splunk, Azure Sentinel, AWS CloudTrail, Datadog) via configurable webhook or cloud-native log export, configured by Security Admin. | Integration | Security Admin | Should | Enterprise security operations; centralised monitoring |
| REQ-ONBAU-007 | Audit Log Retention | Admin audit logs SHALL be retained per the data retention policy (ONB-DR-003), minimum 3 years, with legal hold support (ONB-DR-004). | Governance | Security Admin | Must | Regulatory compliance |
| REQ-ONBAU-008 | Real-Time Audit Notification | The system SHALL allow Security Admin to configure real-time email/webhook notifications for specific high-sensitivity audit event types (e.g., Tenant Admin transfer, mass license revocation, SSO config change, data export initiated). | Security | Security Admin | Should | Insider threat; real-time awareness |

---

## Summary Statistics

| Capability Area | Sub-prefix | Requirement Count |
|----------------|-----------|-------------------|
| Tenant Onboarding Flow | ONB-OB | 24 |
| Editions, Subscriptions & Feature Gating | ONB-ED | 14 |
| Per-Persona License Types & Pools | ONB-LT | 12 |
| Licensing Rules & Enforcement | ONB-LR | 31 |
| Admin Users & Delegated Administration | ONB-AD | 15 |
| User Management & Provisioning | ONB-UM | 20 |
| Invite & Activation Flow | ONB-IV | 14 |
| Admin Config — Database Provider | ONB-DB | 10 |
| Admin Config — Cloud Account & Region | ONB-CL | 10 |
| Admin Config — BYOK AI Keys | ONB-AK | 12 |
| Admin Config — Git Targets | ONB-GT | 14 |
| Admin Config — Knowledge/SaaS Connectors | ONB-KC | 9 |
| Admin Config — Branding, Email/SMTP & Theme | ONB-BX | 9 |
| Admin Config — Security Policy | ONB-SP | 12 |
| Admin Config — Data Residency & Retention | ONB-DR | 8 |
| Billing, Metering & Seat Management | ONB-BL | 12 |
| License Compliance & Reporting | ONB-RP | 7 |
| Onboarding & Admin Audit Log | ONB-AU | 8 |
| **TOTAL** | **ONB** | **241** |

---

*Volume 13 — End. Authored 2026-06-23. Next: update 00-MASTER-INDEX.md §9 Generation Log to add Vol 13 ONB row.*
