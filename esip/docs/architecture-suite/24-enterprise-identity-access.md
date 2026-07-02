# ESIP Architecture Automation Suite — Volume 24: Enterprise Identity & Access (Pluggable SSO / IdP)

**Prefix:** IAM  
**Status:** V1.0 — authored 2026-06-23  
**Scope:** Pluggable IdP / SSO architecture · Identity-provider port/adapter model · App-gallery & tile integration · SAML 2.0 & OIDC/OAuth 2.0 · SCIM 2.0 provisioning · Multi-IdP & home-realm discovery · MFA / step-up / passwordless / FIDO2 / passkeys · Session management & SLO · RBAC + ABAC authorization model · API authentication · Migration from Firebase Auth · Identity audit

**This volume is the authoritative deep identity specification.** It supersedes and expands the high-level IAM sub-area (§3) in Volume 11, which SHALL be treated as a summary cross-reference. Requirements here use prefix `REQ-IAM-NNN` (with sub-area suffixes where noted).

---

## Boundary Notes

| Concern | Owner |
|---------|-------|
| Secrets vault implementation, server-side authz enforcement at the API layer (JWT validation middleware, policy engine runtime) | **Volume 25** (planned; secrets vault + authz enforcement) |
| Licensing seat enforcement, edition feature gating, per-persona license assignment on first login / JIT | **Volume 13** (ONB-LR; cross-ref §5 below) |
| Free Viewer & Reviewer tier authentication/login flows | **Volume 17** (LIC sub-area; cross-ref §6.4) |
| Multi-tenancy data isolation kernel, workspace/project hierarchy ownership | **Volume 11** (PLT-TEN; §2) |
| Enterprise deployment topologies (on-prem, air-gapped Keycloak) | **Volume 19** (ENT-DEP) |
| Audit log storage, retention, tamper-evidence, SIEM export infrastructure | **Volume 11** (PLT-AUD; §4.2) — V24 defines _what identity events_ are logged; V11 owns the log infrastructure |
| Connector org-level OAuth / service auth for Knowledge Hub connectors (Jira, Confluence, ADO) | **Volume 08** (KH connectors) |

---

## Table of Contents

1. [Identity-Provider Port / Adapter Architecture](#1-identity-provider-port--adapter-architecture)
2. [Supported IdP Catalogue](#2-supported-idp-catalogue)
3. [SAML 2.0 Integration](#3-saml-20-integration)
4. [OIDC / OAuth 2.0 Integration](#4-oidc--oauth-20-integration)
5. [App-Gallery & Tile Integration](#5-app-gallery--tile-integration)
6. [SCIM 2.0 Provisioning & Deprovisioning](#6-scim-20-provisioning--deprovisioning)
7. [Multi-IdP, Realms & Home-Realm Discovery](#7-multi-idp-realms--home-realm-discovery)
8. [Authentication Strength: MFA, Passkeys & Conditional Access](#8-authentication-strength-mfa-passkeys--conditional-access)
9. [Sessions, SLO & Impersonation](#9-sessions-slo--impersonation)
10. [Authorization Model: RBAC + ABAC + Delegated Admin](#10-authorization-model-rbac--abac--delegated-admin)
11. [API Authentication: Client-Credentials, API Keys, mTLS](#11-api-authentication-client-credentials-api-keys-mtls)
12. [Migration & Fallback (Firebase Auth → Enterprise IdP)](#12-migration--fallback-firebase-auth--enterprise-idp)
13. [Identity Audit & Observability](#13-identity-audit--observability)

---

## Requirement Conventions

- **ID format:** `REQ-IAM-<Sub>-NNN` where Sub is the two-letter sub-area code (PA, IdP, SA, OI, AG, SC, MR, MF, SS, AZ, AP, MIG, AU).
- **Description:** single SHALL statement, testable and atomic.
- **Categories:** Functional | NFR | Integration | Data | AI/ML | UX | Security | Governance | Reporting.
- **Personas:** Tenant Admin, IT Admin, Security Admin, All (authenticated end-users), Platform Ops (internal), API Client (non-human).
- **Priority (MoSCoW):** Must | Should | Could | Won't-now. Priority ≠ Phase — see 00-MASTER-INDEX §6.

---

## 1. Identity-Provider Port / Adapter Architecture

> **Design principle:** ESIP's identity subsystem is a **ports-and-adapters (hexagonal)** seam. All authentication and token-exchange logic is hidden behind an `IIdentityProvider` port. Each named IdP (Firebase, Okta, Entra ID, Auth0, Keycloak, etc.) is a swappable adapter implementing that port. The front-end route guards and the .NET API token-validation middleware depend only on the port contract — not on any specific IdP SDK. A tenant migrating from Firebase to Okta to Keycloak requires only a new adapter wiring; no UI change, no API contract change.

### 1.1 Identity-Provider Port Contract

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-PA-001 | IIdentityProvider Port Interface | The platform SHALL define a versioned `IIdentityProvider` interface (port) that all identity adapters must implement, exposing: `AuthenticateAsync` (validate bearer token → ClaimsPrincipal), `GetDiscoveryMetadata` (SP/RP metadata), `HandleCallbackAsync` (SAML ACS / OIDC redirect), `InitiateSSOAsync` (SP-initiated redirect), `HandleSLOAsync` (single log-out), `ProvisionUserAsync` (JIT / SCIM), `RevokeSessionAsync`, and `HealthCheckAsync`. | Functional | Platform Ops | Must | Hexagonal architecture mandate (TECH-STACK.md); enables IdP swap without UI/API changes |
| REQ-IAM-PA-002 | Per-Tenant Adapter Selection | The platform SHALL allow each tenant to select exactly one primary IdP adapter from the registered adapter catalogue at any time, and to configure secondary adapters for B2B partner access or multi-IdP scenarios (see §7), without any platform redeployment. | Functional | Tenant Admin, IT Admin | Must | Per-tenant choice; cloud-neutral principle |
| REQ-IAM-PA-003 | Adapter Hot-Swap Without Downtime | The platform SHALL support switching a tenant from one IdP adapter to another (e.g., Firebase → Okta) with zero application downtime, using a configurable overlap window during which both adapters accept tokens, until all sessions and SCIM links are migrated. | NFR | Tenant Admin, IT Admin | Must | Firebase-to-enterprise migration path (cross-ref §12) |
| REQ-IAM-PA-004 | Adapter Versioning | The platform SHALL version each adapter implementation independently; a tenant SHALL be able to pin to a specific adapter version and upgrade deliberately, with a changelog and automated compatibility test report before upgrade. | Governance | Platform Ops | Should | Prevents surprise breakage when an IdP SDK updates or an IdP changes its metadata format |
| REQ-IAM-PA-005 | Adapter Health Monitoring | The platform SHALL continuously monitor the health of each active IdP adapter (connectivity to IdP metadata endpoint, token validation latency, SCIM endpoint availability), surfacing degradation alerts in the admin console and triggering fallback authentication if the primary adapter is unreachable (see §12). | NFR | IT Admin, Security Admin | Must | Availability SLA; fallback trigger |
| REQ-IAM-PA-006 | Custom Adapter Registration | The platform SHALL provide a documented adapter SDK (Rust trait + .NET interface) allowing a customer or implementation partner to register a custom identity adapter for a proprietary or uncommon IdP, without modifying the core platform. | Integration | Platform Ops, IT Admin | Should | Long-tail IdP support (proprietary government directories, bespoke enterprise SSO gateways) |

---

### 1.2 Adapter Catalogue & Configuration Store

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-PA-007 | Adapter Configuration Store | The platform SHALL maintain a per-tenant IdP configuration store (encrypted at rest via V25 secrets vault) holding: adapter type, display name, entityID / client-ID, metadata URL or uploaded XML, client secret reference (vault key), attribute/claims mapping, group mapping rules, license assignment rules, JIT provisioning settings, and enabled/disabled status. | Data | Tenant Admin, IT Admin | Must | Persistent, auditable IdP config per tenant |
| REQ-IAM-PA-008 | Configuration Audit Trail | Every change to the IdP configuration store SHALL be recorded in the identity audit log (§13) with the actor identity, timestamp, before/after diff (secrets redacted), and rationale field. | Governance | Security Admin | Must | SOC2 CC6.1 change management; regulatory audit |
| REQ-IAM-PA-009 | Test-Connection Dry Run | The configuration wizard SHALL provide a "Test Connection" action that performs a dry-run authentication against the configured IdP (using an admin-supplied test credential or a background service account), reports success or structured error (metadata parse failure, certificate mismatch, attribute not found), and does NOT create any user session. | UX | IT Admin | Must | Reduces misconfiguration incidents at go-live |

---

## 2. Supported IdP Catalogue

### 2.1 Named IdP Adapters

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-IdP-001 | Okta Adapter | The platform SHALL provide a first-class Okta adapter supporting: SAML 2.0 and OIDC authentication, Okta Integration Network (OIN) tile publishing (see §5.1), SCIM 2.0 provisioning via the Okta SCIM connector, Okta group→ESIP role mapping, MFA policy delegation, and Okta Universal Prompt. | Integration | IT Admin | Must | Okta is the dominant enterprise IdP; OIN tile is a procurement accelerator |
| REQ-IAM-IdP-002 | Microsoft Entra ID (Azure AD) Adapter | The platform SHALL provide a first-class Microsoft Entra ID adapter supporting: SAML 2.0 and OIDC (Microsoft identity platform v2.0), Azure AD app gallery registration (see §5.2), SCIM 2.0 via Entra automatic provisioning, Entra group→ESIP role mapping, Conditional Access policy integration (device compliance signal), MFA via Microsoft Authenticator, and M365 cross-tenant guest access (B2B collaboration). | Integration | IT Admin | Must | Microsoft Entra ID is the most widely deployed enterprise IdP; Entra app gallery shortens enterprise procurement |
| REQ-IAM-IdP-003 | Azure AD B2C Adapter | The platform SHALL provide a Microsoft Azure AD B2C adapter supporting: external / consumer user flows (sign-up, sign-in, password reset, profile editing), custom policies (Identity Experience Framework), social IdP federation within B2C (Google, Facebook, Apple at B2C level), token issuance to ESIP, and mapping B2C user attributes to ESIP Viewer/Reviewer tiers (cross-ref V17). | Integration | IT Admin | Should | B2C / external-reviewer flows for free-tier (Reviewer) users who are not corporate employees |
| REQ-IAM-IdP-004 | Auth0 / Okta Customer Identity Adapter | The platform SHALL provide an Auth0 (Okta Customer Identity Cloud) adapter supporting OIDC, Universal Login, Auth0 organizations (for multi-tenant B2B), social connections, SCIM bridge, and Actions/Rules for claims enrichment. | Integration | IT Admin | Should | Auth0 widely used for product-led growth / B2B SaaS; Auth0 organizations maps directly to ESIP tenants |
| REQ-IAM-IdP-005 | Ping Identity (PingFederate / PingOne) Adapter | The platform SHALL provide a Ping Identity adapter supporting SAML 2.0 and OIDC for PingFederate on-prem and PingOne cloud, Ping SCIM, and Ping group→ESIP role mapping. | Integration | IT Admin | Should | Ping is common in large financial-services, insurance, and public-sector enterprises |
| REQ-IAM-IdP-006 | OneLogin Adapter | The platform SHALL provide a OneLogin adapter supporting SAML 2.0 and OIDC, OneLogin provisioning (SCIM), and OneLogin SmartFactor Authentication delegation. | Integration | IT Admin | Should | OneLogin common in mid-market enterprises |
| REQ-IAM-IdP-007 | Google Workspace Adapter | The platform SHALL provide a Google Workspace adapter supporting SAML 2.0 (via Google Workspace SAML app) and OIDC (Google Identity Services / Sign in with Google), Google Directory group→ESIP role sync, and SCIM via Google Cloud Directory Sync or direct SCIM. | Integration | IT Admin | Should | Google Workspace common in technology companies and startups; required for consumer-tier Reviewer flows |
| REQ-IAM-IdP-008 | Microsoft ADFS Adapter | The platform SHALL provide an Active Directory Federation Services (ADFS) adapter supporting SAML 2.0 and WS-Federation-originated tokens normalised to SAML 2.0 claims, for on-premises enterprise deployments where Entra ID cloud is not available. | Integration | IT Admin | Should | ADFS common in regulated on-prem environments (government, defence); required for air-gapped deployments cross-ref V19 |
| REQ-IAM-IdP-009 | Keycloak Adapter | The platform SHALL provide a first-class Keycloak adapter supporting SAML 2.0 and OIDC/OAuth 2.0 (Keycloak as Authorization Server), Keycloak group/role→ESIP role mapping, SCIM via Keycloak SCIM extension, and Keycloak Realm Federation (for multi-IdP, §7). This adapter SHALL be the primary identity solution for the **on-prem/air-gapped** deployment profile (cross-ref V19 ENT-DEP). | Integration | IT Admin | Must | Keycloak is the open-source IdP for on-prem/air-gapped; TECH-STACK.md names it explicitly |
| REQ-IAM-IdP-010 | Generic SAML 2.0 Adapter | The platform SHALL provide a generic SAML 2.0 adapter supporting any standards-compliant SAML 2.0 IdP (SP-metadata upload, IdP-metadata upload/URL, ACS configuration, custom attribute mapping, NameID format selection: email/persistent/transient). | Integration | IT Admin | Must | Covers long-tail IdPs not listed above |
| REQ-IAM-IdP-011 | Generic OIDC / OAuth 2.0 Adapter | The platform SHALL provide a generic OIDC adapter supporting any standards-compliant OIDC 1.0 provider (discovery document URL, client-ID/secret, scope configuration, claims mapping, PKCE for SPA flow, authorization code flow for API). | Integration | IT Admin | Must | Covers long-tail OIDC providers |
| REQ-IAM-IdP-012 | LDAP / Active Directory Adapter | The platform SHALL provide an LDAP / Active Directory (AD) adapter supporting: LDAP bind for user authentication, AD over LDAPS (TLS), group-membership query for role mapping, and periodic directory sync (configurable interval), for enterprises that cannot federate via SAML/OIDC but have an accessible AD/LDAP endpoint. | Integration | IT Admin | Should | Legacy on-prem AD environments where ADFS / Entra ID Connect is not deployed; common in manufacturing, utilities |
| REQ-IAM-IdP-013 | Firebase Auth Adapter (Current) | The platform SHALL maintain the Firebase Auth adapter as a fully supported adapter (not deprecated), implementing `IIdentityProvider`, so that tenants that do not require enterprise SSO continue to work without change; this adapter is the default for new SaaS tenants until they configure an enterprise IdP. | Integration | Platform Ops | Must | ESIP current state; non-breaking evolution; TECH-STACK.md §3 Auth/Identity |
| REQ-IAM-IdP-014 | Custom / BYOI Adapter | The platform SHALL support a "Custom / BYOI (Bring Your Own IdP)" registration that accepts a custom adapter package (see REQ-IAM-PA-006) and enables it for a specific tenant. | Integration | IT Admin | Could | Bespoke government / defence IdPs |

---

## 3. SAML 2.0 Integration

### 3.1 Service-Provider (SP) Metadata

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-SA-001 | SP Metadata Endpoint | The platform SHALL expose a per-tenant SAML 2.0 Service Provider metadata document at a stable, publicly reachable URL (`https://<tenant>.esip.io/auth/saml/metadata`) containing: entityID, AssertionConsumerService (ACS) URL(s), NameID format(s), SP signing certificate, SP encryption certificate (if used), and service name. | Integration | IT Admin | Must | SAML 2.0 §4 SP metadata; IdP admin needs this to configure the ESIP application in their IdP |
| REQ-IAM-SA-002 | Metadata Auto-Refresh | The platform SHALL support pointing to an IdP metadata URL and auto-refreshing the IdP metadata on a configurable interval (default: 24 hours), detecting key rollover and certificate changes without requiring manual re-upload. | NFR | IT Admin | Must | Prevents SSO outages on IdP certificate rotation |
| REQ-IAM-SA-003 | EntityID Configuration | The platform SHALL allow the tenant to configure a custom SAML entityID for the SP (default: `https://<tenant>.esip.io/auth/saml`), for organisations with strict naming conventions. | Functional | IT Admin | Should | Enterprise IdP registration policies often mandate specific entityID formats |
| REQ-IAM-SA-004 | ACS URL Allowlist | The platform SHALL validate that the SAML Response destination and the Recipient attributes match the registered ACS URL(s), rejecting responses with unexpected destinations, to prevent SAML redirect attacks. | Security | All | Must | SAML 2.0 binding security; OWASP SAML Security Cheat Sheet |
| REQ-IAM-SA-005 | Assertion Signing Validation | The platform SHALL validate the digital signature on every SAML Assertion or Response using the registered IdP signing certificate, rejecting unsigned or invalidly signed assertions. | Security | All | Must | SAML 2.0 §4.4.4.1; signature validation is mandatory; unsigned assertions enable impersonation |
| REQ-IAM-SA-006 | SP Signing of AuthnRequests | The platform SHALL support signing SP-initiated AuthnRequests with the SP private key (RSA-SHA256 minimum) and SHALL expose the SP signing certificate in SP metadata so the IdP can validate the request. | Security | IT Admin | Should | Prevents AuthnRequest tampering; required by some IdPs (Entra ID, ADFS) in stricter configurations |
| REQ-IAM-SA-007 | Assertion Encryption | The platform SHALL support receiving encrypted SAML Assertions (EncryptedAssertion with AES-256-GCM or AES-128-CBC + RSA-OAEP key transport) and SHALL expose the SP encryption certificate in metadata. | Security | IT Admin | Should | Defense-in-depth for PII in assertions; required by some regulated IdPs |
| REQ-IAM-SA-008 | NameID Format Support | The platform SHALL support all common SAML NameID formats: `emailAddress`, `persistent`, `transient`, `unspecified`, and `X509SubjectName`; the tenant SHALL be able to configure the preferred NameID format for user linkage. | Functional | IT Admin | Must | Different IdPs emit different NameID formats; persistent NameID is preferred for stable user linkage |
| REQ-IAM-SA-009 | Replay Attack Prevention | The platform SHALL enforce SAML assertion replay prevention by tracking assertion IDs in a distributed cache for at least the assertion validity window, rejecting any assertion whose ID has already been consumed. | Security | All | Must | SAML 2.0 §4.1.4.2; replay prevention is a baseline security control |
| REQ-IAM-SA-010 | Assertion Clock Skew Tolerance | The platform SHALL accept SAML assertions whose `NotBefore` / `NotOnOrAfter` timestamps are within a configurable clock-skew tolerance (default: ±3 minutes) to accommodate IdP/SP time synchronisation drift. | NFR | IT Admin | Must | Prevents legitimate assertions being rejected due to minor clock drift between IdP and SP |
| REQ-IAM-SA-011 | SP-Initiated & IdP-Initiated Flows | The platform SHALL support both SP-initiated SSO (user lands on ESIP, redirected to IdP for auth, returned to original URL via RelayState) and IdP-initiated SSO (user clicks tile in IdP portal, asserted into ESIP). | Functional | All | Must | Both flows are required for Okta OIN and Entra app gallery tile behaviour |
| REQ-IAM-SA-012 | RelayState Deep-Link Preservation | The platform SHALL preserve the intended deep-link URL in the RelayState parameter during SP-initiated SSO and restore the user to that URL after successful authentication. | UX | All | Must | Users emailed a direct project link should land there after SSO, not the home page |

---

### 3.2 Certificate Management

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-SA-013 | Certificate Expiry Alerting | The platform SHALL alert the Tenant Admin and Security Admin via in-app notification and email at 90, 60, 30, 14, and 7 days before the expiry of any SAML signing or encryption certificate (both SP certificates and registered IdP certificates). | Security | Security Admin, IT Admin | Must | Expired certificates cause SSO outages; advance notice is critical for enterprise change-control processes |
| REQ-IAM-SA-014 | SP Certificate Rotation | The platform SHALL support zero-downtime SP certificate rotation by: (1) generating a new certificate, (2) exporting updated SP metadata (containing both old and new certificates), (3) providing a rollover window during which both certificates are accepted, and (4) retiring the old certificate after a configurable rollover window (default: 72 hours). | Security | IT Admin | Must | Prevents SSO outage during certificate rotation; enterprise change-freeze periods can last days |
| REQ-IAM-SA-015 | Multi-Certificate IdP Support | The platform SHALL support registering multiple simultaneous IdP signing certificates (e.g., during IdP certificate rotation), accepting assertions signed by any registered valid IdP certificate. | Security | IT Admin | Must | IdP certificate rotation requires accepting both old and new certificates in parallel |

---

## 4. OIDC / OAuth 2.0 Integration

### 4.1 OpenID Connect Configuration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-OI-001 | OIDC Discovery Document | The platform SHALL expose a per-tenant OIDC discovery document at `https://<tenant>.esip.io/auth/oidc/.well-known/openid-configuration` containing: issuer, authorization_endpoint, token_endpoint, userinfo_endpoint, jwks_uri, scopes_supported, response_types_supported, claims_supported, and subject_types_supported. | Integration | IT Admin | Must | OIDC Core §4; allows RP/client auto-configuration; required for Entra ID and Okta OIDC federation |
| REQ-IAM-OI-002 | PKCE for SPA / Mobile Clients | The platform SHALL enforce PKCE (RFC 7636, S256 code-challenge method) for all public clients (the ESIP React SPA and future mobile app, cross-ref V18), rejecting authorization code requests from public clients that omit the code_challenge. | Security | All | Must | RFC 7636; prevents authorization code interception; PKCE is mandatory for public clients per OAuth 2.0 Security BCP |
| REQ-IAM-OI-003 | Authorization Code Flow for Confidential Clients | The platform SHALL support OAuth 2.0 Authorization Code flow with client_secret_basic or client_secret_post authentication for confidential clients (.NET API to upstream IdP). | Security | IT Admin | Must | Standard server-side OAuth2; required for Entra ID, Okta, Auth0 authorization code flow |
| REQ-IAM-OI-004 | ID Token Validation | The platform SHALL validate every OIDC ID token by: checking the `iss` (issuer) against the registered IdP discovery document, validating the `aud` (audience) matches the registered client_id, verifying the `exp` (expiry), checking `iat` (issued-at) is recent (within clock-skew tolerance), and verifying the token signature against the IdP JWKS. | Security | All | Must | OIDC Core §3.1.3.7; token validation is non-negotiable |
| REQ-IAM-OI-005 | JWKS Caching & Refresh | The platform SHALL cache the IdP JWKS (JSON Web Key Set) and refresh it on a configurable interval (default: 1 hour) and on signature validation failure (to handle key rotation), applying exponential backoff on repeated failures. | NFR | IT Admin | Must | Avoids JWKS round-trip on every request while handling IdP key rotation gracefully |
| REQ-IAM-OI-006 | Nonce Replay Prevention | The platform SHALL generate a cryptographically random nonce for every OIDC authorization request, validate the nonce claim in the returned ID token, and reject tokens with a missing or replayed nonce. | Security | All | Must | OIDC Core §3.1.2.1; prevents ID token replay |
| REQ-IAM-OI-007 | OIDC Claims Mapping | The platform SHALL support configuring a per-tenant claims mapping (IdP claim name → ESIP user attribute) for standard claims (`sub`, `email`, `name`, `given_name`, `family_name`, `email_verified`, `picture`) and for custom/extra claims added by the IdP (e.g., department, costCenter, groups). | Functional | IT Admin | Must | Standard claims differ in name/format across IdPs; custom claims carry group/role information |
| REQ-IAM-OI-008 | Refresh Token Handling | The platform SHALL request the `offline_access` scope (where supported by the IdP) to obtain refresh tokens for the SPA session, store them server-side (never in browser storage), and use them to silently extend sessions within the platform idle/absolute timeout policy (§9). | Security | All | Must | Prevents forced re-authentication on short-lived access tokens; refresh tokens must not be exposed client-side |

---

### 4.2 Token Issuance & Internal JWT

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-OI-009 | Internal Platform JWT | After successful authentication via any IdP adapter, the platform SHALL issue a signed internal JWT (RS256 minimum) containing: `sub` (platform user GUID), `tenant_id`, `roles[]`, `personas[]`, `license_type`, `session_id`, `iat`, `exp` (configurable: default 15 minutes for access tokens), and `jti` (JWT ID for revocation tracking). | Security | All | Must | Decouples internal authz decisions from IdP-specific token formats; the .NET API validates only this internal token |
| REQ-IAM-OI-010 | Token Signing Key Rotation | The platform SHALL support rotation of the internal JWT signing keys with zero downtime, using a published JWKS endpoint, a rollover window during which both old and new keys are valid, and automatic revocation of the old key after the window. | Security | Platform Ops | Must | Key hygiene; zero-downtime rotation |
| REQ-IAM-OI-011 | Token Claims Enrichment | The platform SHALL enrich the internal JWT with claims derived from the RBAC/ABAC policy model at issuance time (e.g., `effective_permissions[]`, `data_residency_region`, `mfa_verified`, `step_up_required`), so that downstream API services do not need to re-query the user directory on every request. | Security | All | Should | Performance; reduces authz latency on each API call by embedding pre-computed claims |

---

## 5. App-Gallery & Tile Integration

### 5.1 Okta Integration Network (OIN) Tile

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-AG-001 | Okta OIN Application Submission | The platform SHALL publish ESIP as an application in the Okta Integration Network (OIN), supporting both SAML 2.0 and OIDC authentication methods, so that Okta customers can find and configure ESIP as a pre-built integration without reading ESIP documentation. | Integration | IT Admin | Must | OIN membership shortens enterprise procurement cycles; security teams trust pre-vetted OIN apps |
| REQ-IAM-AG-002 | OIN SCIM Connector | The platform SHALL include a SCIM 2.0 connector in the OIN tile that allows Okta admins to enable automatic provisioning/deprovisioning directly from the Okta Lifecycle Management UI. | Integration | IT Admin | Must | Eliminates manual user management; OIN SCIM is a deal-qualifying requirement for large Okta shops |
| REQ-IAM-AG-003 | OIN Attribute Statements | The ESIP OIN tile configuration SHALL document the required SAML attribute statements / OIDC claims (email, firstName, lastName, groups/roles) and expose these as the default attribute mapping within the OIN tile, minimising custom configuration by the customer's Okta admin. | UX | IT Admin | Must | Reduces configuration burden; reduces support tickets during onboarding |

---

### 5.2 Microsoft Entra ID App Gallery

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-AG-004 | Entra App Gallery Registration | The platform SHALL submit ESIP to the Microsoft Entra ID application gallery, supporting both SAML 2.0 and OIDC (Microsoft identity platform), with pre-configured attribute mapping, so that Entra admins can find and integrate ESIP from the "Enterprise Applications" blade without bespoke configuration. | Integration | IT Admin | Must | Entra app gallery is the primary discovery path for Microsoft-shop enterprises; Entra certification accelerates large deals |
| REQ-IAM-AG-005 | Entra Automatic Provisioning | The ESIP Entra app gallery entry SHALL declare a SCIM 2.0 endpoint so that Entra admins can enable automatic provisioning, with documented attribute mapping for userPrincipalName, displayName, mail, department, jobTitle, and group memberships. | Integration | IT Admin | Must | Entra automatic provisioning is a must-have for Microsoft Enterprise Agreement customers |
| REQ-IAM-AG-006 | Entra Conditional Access Signal | The platform SHALL consume the `cnf` (confirmation) claim or Continuous Access Evaluation (CAE) revocation signals from Entra, immediately revoking the ESIP session when Entra emits a token revocation event (e.g., user account disabled, high-risk sign-in detected). | Security | IT Admin | Should | Near-real-time account revocation for Entra-managed tenants; Microsoft CAE is increasingly mandated in enterprise security reviews |

---

### 5.3 Self-Service Configuration Wizard

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-AG-007 | Self-Service SSO Configuration Wizard | The platform SHALL provide a guided, step-by-step SSO configuration wizard in the admin console (Protocol selection → IdP metadata → Attribute mapping → Certificate → Test connection → Activate) that an IT Admin can complete without ESIP support involvement, including inline help text and IdP-specific step-by-step guidance panels for Okta, Entra ID, Auth0, and Keycloak. | UX | IT Admin | Must | Self-service SSO is a procurement-qualifying feature; reduces ESIP support burden |
| REQ-IAM-AG-008 | SP Metadata Download & Copy | The wizard SHALL provide a one-click download of the SP SAML metadata XML, a copy-to-clipboard of the ACS URL and entityID, and a QR-code / link for the OIDC redirect URI, so the IT Admin can easily paste values into their IdP configuration form. | UX | IT Admin | Must | Reduces copy-paste errors during IdP setup |
| REQ-IAM-AG-009 | Claims / Attribute Debugger | The platform SHALL provide a Claims Debugger tool (available to IT Admins) that, after a test authentication, displays the raw claims/attributes received from the IdP, the mapped ESIP user attributes, the resolved roles, and the assigned persona/license, highlighting any missing or unexpected claims. | UX | IT Admin | Must | Dramatically reduces SSO misconfiguration debugging time; standard in enterprise SSO tooling (Okta previewer, Entra SAML Toolkit) |
| REQ-IAM-AG-010 | Certificate Expiry Dashboard | The admin console SHALL display a certificate health dashboard showing all registered SP and IdP certificates across all configured IdPs, with expiry dates, days remaining, RAG status, and a "Rotate now" action per certificate. | UX | Security Admin, IT Admin | Must | Single pane of glass for cert hygiene; prevents surprise SSO outages |
| REQ-IAM-AG-011 | Staged Rollout & Shadow Mode | The SSO configuration wizard SHALL support a "shadow mode" phase where the new IdP adapter is configured and test-validated but logins continue via the existing method; the IT Admin activates the new IdP for a pilot group (by email domain or group membership) before full cutover. | UX | IT Admin | Should | Reduces risk of enterprise-wide SSO misconfiguration; standard enterprise change-management practice |

---

## 6. SCIM 2.0 Provisioning & Deprovisioning

### 6.1 SCIM Endpoint

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-SC-001 | SCIM 2.0 API Endpoint | The platform SHALL implement a SCIM 2.0 compliant REST API at `https://<tenant>.esip.io/scim/v2/` supporting: `/Users` (create, read, update, delete, list/filter), `/Groups` (create, read, update, delete, list/filter), `/Schemas`, `/ResourceTypes`, and `/ServiceProviderConfig`, in accordance with RFC 7643 and RFC 7644. | Integration | IT Admin | Must | RFC 7643/7644; Okta, Entra ID, and Ping all support SCIM 2.0 for automatic provisioning |
| REQ-IAM-SC-002 | SCIM Bearer Token Authentication | The SCIM endpoint SHALL authenticate inbound requests using a long-lived SCIM Bearer token (generated in the admin console, stored in V25 secrets vault, rotatable with zero-downtime via overlap window) and SHALL reject all unauthenticated SCIM requests with HTTP 401. | Security | IT Admin | Must | SCIM RFC 7644 §2; token-based auth is the standard SCIM auth mechanism used by Okta, Entra, Ping |
| REQ-IAM-SC-003 | User Attribute Mapping | The platform SHALL support configuring a per-tenant SCIM attribute mapping for User resources, mapping SCIM standard attributes (`userName`, `name.givenName`, `name.familyName`, `emails[primary]`, `displayName`, `active`, `title`, `department`, `externalId`) and custom Enterprise User extension attributes to ESIP user profile fields. | Integration | IT Admin | Must | Each IdP uses slightly different attribute names; configurable mapping prevents hard-coded attribute assumptions |
| REQ-IAM-SC-004 | Group Provisioning & Role Mapping | The platform SHALL support SCIM Group provisioning, automatically creating or updating ESIP groups from IdP groups, and SHALL apply group→ESIP role and persona mapping rules configured by the IT Admin (e.g., IdP group "ESIP-Enterprise-Architects" → ESIP persona P1 + role Architect, license type EA). | Integration | IT Admin | Must | Eliminates manual role assignment at scale; cross-ref V13 ONB-LT license assignment |
| REQ-IAM-SC-005 | Immediate Deprovisioning | The platform SHALL process a SCIM DELETE /Users/{id} or a PATCH `active=false` event within 60 seconds, immediately invalidating all active sessions, API tokens, and refresh tokens for the deprovisioned user, and marking the user account as suspended (not deleted, to preserve audit history). | Security | IT Admin | Must | SOC2 CC6.2; leaver access removal SLA; suspended (not deleted) to preserve audit trail |
| REQ-IAM-SC-006 | License Reclaim on Deprovision | The platform SHALL automatically return the deprovisioned user's consumed license seat(s) to the tenant's available license pool within the same 60-second deprovisioning window (cross-ref V13 ONB-LR). | Functional | Tenant Admin | Must | License pool management; prevents ghost-user license waste |
| REQ-IAM-SC-007 | SCIM Audit Log | Every inbound SCIM operation (create, update, delete, group membership change) SHALL be recorded in the identity audit log (§13) with: operation type, affected resource, IdP-provided externalId, requesting SCIM client IP, timestamp, and outcome. | Governance | Security Admin | Must | SOC2 CC6.2; full provenance of automated provisioning events |

---

### 6.2 Just-In-Time (JIT) Provisioning

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-SC-008 | JIT User Creation on First SSO | The platform SHALL create a new user account on first successful SSO login when no matching account exists (matching by NameID or email, configurable), mapping IdP assertion attributes to the ESIP user profile and assigning an initial role and persona based on configured claim-mapping rules. | Functional | All | Should | JIT reduces provisioning lag for tenants without SCIM; common enterprise SSO expectation |
| REQ-IAM-SC-009 | JIT Attribute Sync on Login | The platform SHALL optionally update the ESIP user profile attributes (display name, department, title) on every SSO login from the IdP assertion, keeping the ESIP profile in sync without requiring SCIM. | Functional | All | Could | Useful for tenants with dynamic org structures but no SCIM connector |
| REQ-IAM-SC-010 | JIT License Assignment | When a user is JIT-provisioned, the platform SHALL automatically assign a license seat based on the IdP group / claim mapping rules configured for the tenant; if no matching rule exists, the user SHALL be assigned the lowest license tier (Viewer) and notified that access is pending admin approval. | Functional | Tenant Admin | Should | Cross-ref V13 ONB-LR license assignment rules; prevents unlicensed access on JIT |

---

## 7. Multi-IdP, Realms & Home-Realm Discovery

### 7.1 Multiple IdPs per Tenant

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-MR-001 | Multiple IdP Configurations per Tenant | The platform SHALL allow a tenant to register and activate more than one IdP configuration simultaneously (e.g., primary corporate Entra ID + a partner Okta + a Google Workspace for external reviewers), each with independent claims mapping, role mapping, and SCIM bindings. | Functional | IT Admin | Should | B2B partner access and free-tier external reviewers often have different IdPs from the corporate tenant |
| REQ-IAM-MR-002 | Home-Realm Discovery by Email Domain | The platform SHALL implement home-realm discovery that determines which IdP to redirect to based on the user's email domain, routing `@acme.com` to Acme's Entra ID and `@partner.org` to the partner's Okta, without the user having to select their IdP. | UX | All | Should | Eliminates "which IdP?" confusion for multi-IdP tenants; standard in enterprise federated login |
| REQ-IAM-MR-003 | Email Domain → IdP Routing Rules | The platform SHALL allow IT Admins to configure email domain → IdP routing rules per tenant, supporting exact-match domain patterns (`@acme.com`), wildcard subdomain patterns (`@*.acme.com`), and a catch-all fallback IdP. | Functional | IT Admin | Should | Flexible routing for complex enterprise domain structures |
| REQ-IAM-MR-004 | IdP Selection Screen Fallback | When home-realm discovery cannot determine the IdP (e.g., unknown email domain), the platform SHALL present an IdP selection screen listing the tenant's configured IdPs with display names and logos, before redirecting. | UX | All | Should | User experience fallback when domain routing is ambiguous |

---

### 7.2 B2B Partner Access

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-MR-005 | B2B Partner IdP Federation | The platform SHALL support federating a partner organisation's IdP (via SAML 2.0 or OIDC) into a tenant's ESIP workspace, so that partner users authenticate with their own corporate credentials and are mapped to designated ESIP roles (typically Reviewer or Guest). | Functional | IT Admin | Should | External architects, consultants, and systems integrators working in ESIP projects use their employer IdP |
| REQ-IAM-MR-006 | Partner User Namespace Isolation | Partner-federated users SHALL be scoped to designated workspaces or projects (not given tenant-wide access) and SHALL be clearly labelled in the user directory as external/partner users, with their IdP source displayed. | Security | IT Admin, Security Admin | Should | Principle of least privilege for external users; visibility into external access |
| REQ-IAM-MR-007 | Partner Session Binding | Platform sessions for B2B partner users SHALL honour the partner IdP's session lifetime and SHALL be immediately invalidated if the partner IdP issues a back-channel logout / revocation event. | Security | IT Admin | Should | Partner organisation retains control over their users' access; SLO cross-IdP |

---

### 7.3 B2C / External-Reviewer Flows

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-MR-008 | B2C Login for Free Reviewer Tier | The platform SHALL support consumer-grade login flows for the free Reviewer tier (cross-ref V17), including: social sign-in via Google and Microsoft accounts (OIDC), email/password (via Firebase Auth or Azure AD B2C), and email magic-link (passwordless), without requiring the user to have a corporate IdP account. | Functional | All | Should | Free Reviewer tier users are often external stakeholders without corporate IdP access |
| REQ-IAM-MR-009 | Reviewer Account Auto-Creation | The platform SHALL automatically create a ESIP Viewer/Reviewer account on first successful B2C login (JIT), mapping the social identity to a tenant-scoped external user record with the minimum required attributes (email, display name). | Functional | All | Should | Frictionless onboarding for free-tier users; cross-ref V17 LIC |
| REQ-IAM-MR-010 | B2C Email Verification | The platform SHALL require email address verification (via a confirmation email with time-limited link) for any B2C account created with email/password, before granting access to any ESIP workspace. | Security | All | Must | Prevents fake account creation and email-address spoofing in B2C flows |

---

## 8. Authentication Strength: MFA, Passkeys & Conditional Access

### 8.1 MFA Methods

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-MF-001 | MFA Delegation to IdP | The platform SHALL support fully delegating MFA enforcement to the enterprise IdP (SAML `AuthnContextClassRef` / OIDC `acr_values` negotiation), accepting the IdP's MFA assertion without requiring the user to re-enrol an ESIP-side second factor. | Security | All | Must | Enterprise security teams manage MFA policy centrally via the IdP; double-MFA creates friction and erodes trust |
| REQ-IAM-MF-002 | Built-in TOTP | The platform SHALL provide a built-in TOTP (Time-based One-Time Password, RFC 6238, 30-second window, HMAC-SHA1) second factor, compatible with any TOTP authenticator app (Google Authenticator, Microsoft Authenticator, Authy, 1Password), for tenants not using an enterprise IdP or as a fallback. | Security | All | Must | NIST SP 800-63B AAL2; fallback MFA for non-SSO tenants; widely understood by end users |
| REQ-IAM-MF-003 | FIDO2 / WebAuthn Hardware Security Keys | The platform SHALL support FIDO2/WebAuthn (W3C Web Authentication Level 2) authentication using roaming hardware security keys (YubiKey, Titan Key, Google Advanced Protection-compatible devices) as a second factor or as a primary passwordless factor. | Security | All | Must | NIST SP 800-63B AAL3 (hardware bound); phishing-resistant; increasingly mandated by government and financial-sector tenants |
| REQ-IAM-MF-004 | Passkeys (Synced FIDO2 Credentials) | The platform SHALL support passkeys (FIDO2 discoverable credentials synced by the platform authenticator — Apple iCloud Keychain, Google Password Manager, Windows Hello / Microsoft Authenticator) as a primary passwordless authenticator, providing both user verification and platform binding. | Security | All | Should | FIDO Alliance passkeys specification; W3C WebAuthn Level 3; passwordless replaces phishable passwords; growing industry adoption 2025–2026 |
| REQ-IAM-MF-005 | Push Notification MFA (via IdP) | The platform SHALL support consuming push-notification MFA via OIDC `acr_values` negotiation with IdPs that issue push MFA (Okta Verify, Microsoft Authenticator, Duo Security), accepting the IdP's step-up token after push approval. | Security | All | Should | Push MFA is the dominant MFA UX in enterprise; delegation model means no extra ESIP implementation |
| REQ-IAM-MF-006 | Backup / Recovery Codes | The platform SHALL generate a set of one-time backup recovery codes on TOTP or WebAuthn enrolment, encrypted at rest, displayed once, and usable to regain access if the primary second factor is lost; use of a recovery code SHALL trigger an immediate alert to the user and security admin. | Security | All | Must | MFA self-service account recovery; prevents helpdesk lockout spiral |

---

### 8.2 MFA Policies

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-MF-007 | Tenant MFA Policy | The platform SHALL allow Security Admins to configure a tenant-wide MFA policy with options: (a) require MFA for all users; (b) require MFA only for users with elevated roles (Admin, Owner, Approver); (c) require MFA for logins outside of a configured IP allowlist (§8.3); (d) defer to IdP MFA only. | Security | Security Admin | Must | Flexible MFA policy supporting diverse enterprise security postures |
| REQ-IAM-MF-008 | Step-Up Authentication | The platform SHALL enforce step-up authentication (requiring the user to re-verify their second factor) before: (a) accessing the admin console; (b) performing sensitive actions (delete project, export all artifacts, change IdP config, add admin role, impersonate user — see §9); (c) accessing artifacts marked with a tenant-defined sensitivity threshold. | Security | All | Must | Mitigates session hijacking by requiring step-up at high-value operations; NIST SP 800-63C §4.1.2 |
| REQ-IAM-MF-009 | MFA Enrolment Grace Period | The platform SHALL enforce an admin-configurable grace period (default: 24 hours) after first login during which MFA enrolment is prompted but not yet mandatory, after which the user SHALL be required to complete MFA enrolment before accessing any workspace content. | UX | All | Must | Balances security with onboarding UX; 0-hour grace is too aggressive for enterprise rollout |
| REQ-IAM-MF-010 | MFA Bypass (Allowlisted Networks) | The platform SHALL allow Security Admins to configure IP CIDR ranges as trusted networks where MFA is not required for standard-role users, for organisations with strong network-perimeter controls (internal corporate network, VPN). | Security | Security Admin | Should | Common enterprise policy; reduces MFA friction for on-network users while protecting off-network access |

---

### 8.3 Conditional Access & Device Trust

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-MF-011 | IP Allowlist / Denylist | The platform SHALL support tenant-level IP allowlists (CIDR notation, IPv4 and IPv6) that restrict login attempts or API calls to designated IP ranges, and IP denylists blocking specific ranges; policy violations SHALL produce an HTTP 403 with a user-visible "access restricted by your organisation's policy" message. | Security | Security Admin | Must | Network-perimeter access control; common enterprise requirement for regulated environments |
| REQ-IAM-MF-012 | Device Trust via IdP | The platform SHALL accept Entra ID device compliance claims (the `deviceid` / `compliantDevice` claim in Entra tokens) and Okta device assurance policy signals, treating device-compliant logins as higher-assurance and optionally relaxing MFA requirements accordingly. | Security | IT Admin | Should | Device-trust signals from the corporate IdP enable zero-trust conditional-access policies |
| REQ-IAM-MF-013 | Passwordless Email Magic Link | The platform SHALL support passwordless login via a time-limited (15-minute), single-use, signed magic link sent to the user's verified email address, as an alternative authentication method for B2C / Reviewer-tier users without SSO. | UX | All | Should | Reduces password-management burden for external/free-tier users; email magic links are widely accepted |

---

## 9. Sessions, SLO & Impersonation

### 9.1 Session Lifecycle

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-SS-001 | Session Idle Timeout | The platform SHALL enforce a configurable per-tenant idle session timeout (default: 30 minutes; range: 5 minutes–8 hours), after which the user is required to re-authenticate; the timeout SHALL reset on any user action that reaches the server. | Security | Security Admin | Must | SOC2 CC6.1; NIST SP 800-63C; reduces exposure from unattended sessions |
| REQ-IAM-SS-002 | Session Absolute Timeout | The platform SHALL enforce a configurable per-tenant absolute session lifetime (default: 12 hours; range: 1 hour–7 days), after which the session expires regardless of activity, requiring full re-authentication. | Security | Security Admin | Must | Mitigates persistent sessions on shared or compromised devices; NIST SP 800-63C |
| REQ-IAM-SS-003 | Concurrent Session Limit | The platform SHALL support a configurable per-tenant concurrent session limit per user (default: unlimited; optional: 1 or N sessions maximum), with a tenant-configurable policy of either rejecting new logins when the limit is reached or invalidating the oldest session on new login. | Security | Security Admin | Should | Prevents credential sharing; useful for compliance audits ("prove one person used the tool") |
| REQ-IAM-SS-004 | Session Revocation on Security Event | The platform SHALL immediately revoke all active sessions for a user when any of the following occur: password change (for non-SSO accounts), admin-triggered revocation, SCIM deprovisioning (`active=false`), IdP back-channel logout (SAML SLO / OIDC logout), or detection of a suspicious authentication event flagged by the identity audit log. | Security | Security Admin | Must | SOC2 CC6.1; critical security control; revocation must propagate to all platform nodes within 60 seconds |
| REQ-IAM-SS-005 | Session List for Users | The platform SHALL provide authenticated users with a self-service "Active Sessions" page listing all current sessions (device type inferred from user-agent, IP address, location estimate, last active time), with a "revoke this session" and "revoke all other sessions" action. | UX | All | Must | User self-service security; GDPR transparency; industry standard (Google, GitHub) |
| REQ-IAM-SS-006 | Admin Session View | The platform SHALL provide Security Admins with a tenant-wide session list, searchable and filterable by user, IP, device, login time, and IdP, with the ability to revoke any session or all sessions for a given user. | Security | Security Admin | Must | Incident response capability; revoke access of a compromised account immediately |

---

### 9.2 Single Log-Out (SLO)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-SS-007 | SAML 2.0 SP-Initiated SLO | The platform SHALL support SAML 2.0 Single Logout as the Service Provider initiator: on user logout from ESIP, the platform SHALL send a SAML LogoutRequest to the IdP SLO endpoint (HTTP-Redirect or HTTP-POST binding), terminate the local ESIP session, and redirect to the IdP logout page or a configured post-logout URL. | Security | All | Must | SAML 2.0 §3.7; without SLO, users who log out of ESIP remain silently authenticated at the IdP (and vice versa); enterprise security expectation |
| REQ-IAM-SS-008 | SAML 2.0 IdP-Initiated SLO | The platform SHALL accept SAML LogoutRequest messages from the IdP (IdP-initiated SLO), validate the request signature, terminate all local ESIP sessions for the identified user, and return a signed SAML LogoutResponse to the IdP. | Security | All | Must | Enables the IdP admin to force-logout a user from all federated apps simultaneously; required for Okta and Entra SLO policies |
| REQ-IAM-SS-009 | OIDC Front-Channel Logout | The platform SHALL implement the OIDC Front-Channel Logout specification (OpenID Connect Front-Channel Logout 1.0), registering a `frontchannel_logout_uri` in the IdP client configuration so the IdP can notify ESIP when the user logs out of the IdP session. | Security | All | Should | OIDC SLO for IdPs that use front-channel (browser-based) logout notification |
| REQ-IAM-SS-010 | OIDC Back-Channel Logout | The platform SHALL implement the OIDC Back-Channel Logout specification (OpenID Connect Back-Channel Logout 1.0), providing a `backchannel_logout_uri` endpoint that receives a signed `logout_token` from the IdP and immediately revokes all associated ESIP sessions. | Security | All | Must | Back-channel SLO does not depend on an active browser session; more reliable than front-channel; supported by Okta, Entra ID, Auth0, Keycloak |
| REQ-IAM-SS-011 | SLO Propagation to All Nodes | ESIP session revocation on SLO SHALL propagate to all platform nodes (API replicas, BFF layers) within 10 seconds via a distributed session store invalidation event, ensuring no stale session on any node can service requests after revocation. | NFR | All | Must | Distributed session revocation; required for multi-node K8s deployments |

---

### 9.3 Impersonation & Break-Glass

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-SS-012 | Admin Impersonation | The platform SHALL allow Platform Ops (internal ESIP staff only, never tenant admins) to impersonate any tenant user for support purposes, with: (a) a mandatory reason field; (b) a time-limited impersonation token (max: 60 minutes, non-renewable); (c) an immutable audit log entry recording the impersonating operator, impersonated user, reason, and every action taken during the session; (d) an optional real-time notification to the impersonated user. | Security | Platform Ops | Must | Support and debugging capability; must be strictly controlled and audited to prevent abuse |
| REQ-IAM-SS-013 | Impersonation Scope Restriction | Admin impersonation SHALL be restricted to read-only access by default; write access during impersonation SHALL require a second explicit confirmation and SHALL generate a separate, higher-severity audit event. | Security | Platform Ops | Must | Least privilege during impersonation; prevents accidental data changes during support sessions |
| REQ-IAM-SS-014 | Break-Glass / Emergency Access | The platform SHALL maintain a break-glass emergency access account per tenant (credentials stored offline in V25 secrets vault, sealed with tenant-admin-held key shards), accessible only after a documented emergency access request process, with mandatory post-incident audit review. | Security | Platform Ops, Security Admin | Must | Handles scenarios where the primary IdP is completely unavailable and no local admin can log in; required for SLA recovery |
| REQ-IAM-SS-015 | Break-Glass Usage Alert | Any use of a break-glass emergency access account SHALL immediately trigger real-time alerts to: the tenant's registered Security Admin email(s), ESIP Platform Ops security mailbox, and an immutable audit log entry with the highest severity classification. | Security | Security Admin, Platform Ops | Must | Break-glass use is an exceptional event that must be monitored; alert-on-use is a SOC2 CC7 requirement |

---

## 10. Authorization Model: RBAC + ABAC + Delegated Admin

> **Note:** The V11 §3.3 RBAC/ABAC sub-area (PLT-ACC) provides the foundational role and permission scaffolding. This section deepens the identity-specific authorization model — persona/license-aware entitlements, fine-grained per-artifact/field permissions, delegated admin scoping, and the policy evaluation engine. Server-side enforcement of all policies at the API layer is owned by **Volume 25**.

### 10.1 Role & Permission Model

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-AZ-001 | Platform Role Catalogue | The platform SHALL define a canonical platform role catalogue with at minimum: Platform Admin (internal), Tenant Admin, Security Admin, User Admin, Config Admin, Billing Admin, Workspace Admin, Project Owner, Architect, Reviewer, Viewer, Guest; each role SHALL have a documented, immutable permission baseline. | Security | All | Must | Canonical roles referenced throughout all other volumes; required for consistent RBAC enforcement |
| REQ-IAM-AZ-002 | Permission Catalogue | The platform SHALL maintain a machine-readable Permission Catalogue enumerating every permission in the system as `resource:action` pairs (e.g., `artifact:create`, `project:delete`, `tenant:manage-idp`, `workspace:admin`, `report:export`, `user:impersonate`), versioned and queryable via an admin API. | Functional | Platform Ops | Must | Foundation for custom roles (V11 PLT-ACC-002); enables UI to check permissions programmatically; required for authz engine in V25 |
| REQ-IAM-AZ-003 | Persona-Aware Entitlements | The platform SHALL enforce that a user can only access artifact types, diagram types, deliverable templates, and AI prompt packs associated with their assigned persona(s) and licensed tier, determined at token-issuance time and embedded in the internal JWT (cross-ref REQ-IAM-OI-009), with no client-side bypass possible. | Security | All | Must | Cross-ref V13 ONB-LR licensing-enforcement; persona restrictions must be enforced server-side |
| REQ-IAM-AZ-004 | License-Tier Entitlement Enforcement | The platform SHALL enforce that features gated behind higher license tiers (e.g., EA persona features, AI copilot, export to Word/PPT) are inaccessible to users whose assigned license type does not include those features, at the API layer, returning HTTP 403 with a structured `entitlement_required` error body. | Security | All | Must | Licensing integrity; prevents feature bypass by API-direct calls; cross-ref V13 ONB-ED feature gating |
| REQ-IAM-AZ-005 | Role Assignment at Multiple Scopes | The platform SHALL support role assignments at three scopes: (1) Tenant scope (applies to all workspaces and projects); (2) Workspace scope (overrides tenant scope within a workspace); (3) Project scope (overrides workspace scope within a project); lower-scope assignments SHALL take precedence for narrowing access, not for escalating beyond the tenant-scope ceiling. | Functional | Tenant Admin, Workspace Admin | Must | Enterprise org structures require scoped role management; project owners should not grant tenant-wide admin |
| REQ-IAM-AZ-006 | Inherited Role Resolution | The platform SHALL compute the effective role for a user at any given scope by resolving the most specific (narrowest) scope assignment, falling back to wider scopes if no specific assignment exists, with the computed effective role always visible in the admin user-details panel. | Functional | Tenant Admin | Must | Transparency of effective permissions; resolves "why can't I see X?" support questions |

---

### 10.2 ABAC Policy Engine

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-AZ-007 | ABAC Policy Rule Definition | The platform SHALL support defining ABAC policy rules as structured expressions (subject attributes AND resource attributes AND environment attributes → permit/deny), editable by Tenant Admins and Security Admins via an admin console rule builder, without code changes. | Security | Security Admin | Should | NIST SP 800-162; fine-grained access for regulated data (government, healthcare, financial); enables dynamic access decisions |
| REQ-IAM-AZ-008 | Subject Attributes for ABAC | ABAC policy SHALL support the following subject (user) attributes as inputs: `role`, `persona`, `license_type`, `department`, `cost_center`, `clearance_level` (custom, tenant-defined), `mfa_verified`, `authentication_method` (SAML/OIDC/local), `IP_in_allowlist`, `device_compliant`. | Security | Security Admin | Should | Rich subject context enables fine-grained policies (e.g., only SAML-authenticated users with clearance_level≥SECRET can read DoDAF artifacts) |
| REQ-IAM-AZ-009 | Resource Attributes for ABAC | ABAC policy SHALL support the following resource attributes as inputs: `artifact_type`, `classification` (tenant-defined labels: Public/Internal/Confidential/Restricted), `owner_id`, `workspace_id`, `project_id`, `framework`, `data_residency_region`. | Security | Security Admin | Should | Fine-grained resource-based policies (e.g., "Confidential" classified artifacts require Architect role AND SAML auth) |
| REQ-IAM-AZ-010 | ABAC Policy Simulation | The admin console SHALL provide an ABAC policy simulation tool where an admin can enter a hypothetical (user, action, resource) triple and see whether the policies permit or deny, and which specific rule caused the outcome. | UX | Security Admin | Should | Dramatically reduces ABAC misconfiguration risk; essential for complex policy sets |
| REQ-IAM-AZ-011 | Per-Artifact Classification Labels | The platform SHALL allow tenant admins to define a classification label schema (e.g., Public, Internal, Confidential, Restricted, Top Secret), allow Architects to apply labels to artifacts, and enforce ABAC policies that reference the label; default label for new artifacts SHALL be configurable per tenant. | Security | All | Should | Data classification enables ABAC policies and field-level masking; essential for government and financial tenants |

---

### 10.3 Fine-Grained Permissions

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-AZ-012 | Per-Artifact Access Overrides | The platform SHALL allow Project Owners and Architects to set per-artifact access control overrides (restrict read, write, or share to a named user list or role list), independently of project-level RBAC, for commercially sensitive or classified artifacts. | Security | All | Should | Cross-ref V11 PLT-ACC-005; named-user access control for sensitive architecture items (e.g., M&A target architecture) |
| REQ-IAM-AZ-013 | Field-Level Visibility Enforcement | The platform SHALL enforce field-level visibility rules (configured per artifact type field in the metamodel) at the API response layer, omitting hidden fields from API responses for users without the required role, regardless of query parameters. | Security | All | Should | Cross-ref V11 PLT-ACC-006; field-level enforcement must be server-side, not client-side |
| REQ-IAM-AZ-014 | Diagram Sharing Scope | The platform SHALL allow the creator of a diagram to set a sharing scope (Private, Project, Workspace, Tenant, Authenticated-External, Public-Link), and enforce that only users within the sharing scope can view or embed the diagram, including shared links. | Functional | All | Must | Controls on architecture diagram sharing are a common enterprise governance requirement |

---

### 10.4 Delegated Administration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-AZ-015 | Delegated Admin Scoping | The platform SHALL allow Tenant Admins to delegate Workspace Admin rights to specific users within designated workspaces, granting them authority to: manage workspace membership, assign project-scoped roles, configure workspace branding, and manage workspace-level connectors — but NOT to modify tenant-level IdP config, billing, or security policy. | Functional | Tenant Admin | Must | Operational scalability; large enterprises need workspace-level admin autonomy without sharing tenant-admin credentials |
| REQ-IAM-AZ-016 | Admin Role Ceiling | Delegated Workspace Admins SHALL NOT be able to grant roles higher than their own ceiling role; specifically, a Workspace Admin SHALL NOT be able to grant Tenant Admin or Security Admin to any user, preventing privilege escalation. | Security | All | Must | Prevents delegated-admin privilege escalation; standard RBAC security invariant |
| REQ-IAM-AZ-017 | Admin Actions Audit | All admin actions (user role assignment/revocation, IdP config change, policy update, session revocation, break-glass use, impersonation) SHALL be recorded in the identity audit log (§13) with the acting admin's identity, scope, target resource, and timestamp. | Governance | Security Admin | Must | SOC2 CC6.3; full audit trail of privileged actions |

---

## 11. API Authentication: Client-Credentials, API Keys, mTLS

### 11.1 OAuth 2.0 Client Credentials

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-AP-001 | OAuth 2.0 Client Credentials Flow | The platform SHALL implement the OAuth 2.0 Client Credentials grant (RFC 6749 §4.4) for non-human API clients (CI/CD pipelines, external integrations, the Rust code-scanner microservice), issuing access tokens scoped to the client's registered permissions. | Security | API Client | Must | Standard machine-to-machine auth; required for Rust scanner agent, connector services, and V17 AGT sub-area agents |
| REQ-IAM-AP-002 | Client Registration | The platform SHALL allow IT Admins and Tenant Admins to register OAuth 2.0 clients in the admin console, specifying: client name, allowed grant types (client_credentials only for non-human), scopes, IP allowlist, token expiry, and a webhook URL for token lifecycle events. | Functional | IT Admin | Must | Self-service client management; audit-grade client registry |
| REQ-IAM-AP-003 | Scoped Token Issuance | Access tokens issued via Client Credentials SHALL carry only the scopes explicitly granted to the client at registration, enforcing least-privilege for non-human callers; clients SHALL NOT be able to request scopes beyond their registered grant. | Security | API Client | Must | Prevents scope creep for automated callers; consistent with OAuth 2.0 RFC 6749 |
| REQ-IAM-AP-004 | Client Secret Rotation | The platform SHALL support rotating OAuth 2.0 client secrets with a configurable overlap window (default: 24 hours) during which both old and new secrets are valid, after which the old secret is invalidated; rotation SHALL be triggerable by the IT Admin or on a configured schedule. | Security | IT Admin | Must | Client secret hygiene; zero-downtime rotation for production integrations |

---

### 11.2 API Keys

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-AP-005 | Named API Keys | The platform SHALL allow creating named, scoped API keys per tenant or per service account, with: a configurable expiry date (default: 90 days; max: 1 year; non-expiring keys require explicit Security Admin approval), a permission scope (e.g., `artifact:read`, `project:read`), an IP allowlist, a description, and a label. | Security | IT Admin | Must | Cross-ref V11 PLT-EXT-003; programmatic access for CI/CD and integrations |
| REQ-IAM-AP-006 | API Key Hashing | The platform SHALL store only the HMAC-SHA256 hash of the API key value, never the plaintext; the plaintext key SHALL be displayed exactly once at creation and not be retrievable afterward. | Security | IT Admin | Must | Prevents API key theft from the ESIP database; industry standard (GitHub PAT model) |
| REQ-IAM-AP-007 | API Key Usage Telemetry | The platform SHALL record every API key usage event (key ID, endpoint called, IP address, timestamp, HTTP status) in the identity audit log, enabling admins to audit API access patterns and detect anomalous usage. | Security | IT Admin | Should | Audit trail; anomaly detection input; compliance evidence |
| REQ-IAM-AP-008 | API Key Expiry Alerting | The platform SHALL alert the creating user and IT Admin via email and in-app notification at 30, 14, and 7 days before an API key's expiry, with a "Rotate key" action in the alert. | Security | IT Admin | Must | Prevents integrations breaking on silent key expiry |

---

### 11.3 Service Accounts

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-AP-009 | Service Account Identity | The platform SHALL support service accounts as first-class non-human identities distinct from user accounts, with: a display name, owner (human user responsible), description, assigned role(s), and a dedicated audit log stream. | Security | IT Admin | Must | Cross-ref V11 PLT-EXT-004; non-human identities must be separately auditable and revocable |
| REQ-IAM-AP-010 | Service Account Credential Types | Service accounts SHALL support two credential types: (a) OAuth 2.0 client_credentials (preferred for new integrations), (b) API key (for legacy integrations); a service account SHALL NOT use a human user's credentials. | Security | IT Admin | Must | Prevents shared credentials; each non-human caller has its own identity |
| REQ-IAM-AP-011 | Service Account Lifecycle | Service accounts SHALL be tied to a named human owner and SHALL be automatically flagged for review when the owner leaves the organisation (SCIM deprovisioning of the owner); orphaned service accounts SHALL be suspended after 14 days without a new owner assignment. | Governance | IT Admin | Should | Prevents persistent unauthorised access via orphaned service accounts after staff departure |

---

### 11.4 mTLS for Scanner Agents

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-AP-012 | mTLS Authentication for Rust Scanner Agents | The platform SHALL support mutual TLS (mTLS, RFC 8705) authentication for the Rust code-scanner microservice agents (cross-ref V17 AGT sub-area), requiring scanner agents to present a client certificate signed by the ESIP tenant certificate authority, validating the certificate at the API gateway layer. | Security | API Client | Should | mTLS provides the strongest non-human authentication; scanner agents operate in the customer's network perimeter and may handle sensitive repository content; certificate-bound access prevents bearer-token theft |
| REQ-IAM-AP-013 | Scanner Agent Certificate Provisioning | The platform SHALL provide an automated certificate provisioning flow for scanner agents: the agent generates a CSR, submits it to the ESIP Agent Registration API (authenticated with a short-lived bootstrap token), the platform signs and returns the client certificate (valid for a configurable period, default: 1 year), and the agent stores the certificate in the OS credential store or a secure file path. | Integration | IT Admin | Should | Operationalises mTLS agent auth; bootstrap token pattern is the standard approach for zero-touch agent cert provisioning |
| REQ-IAM-AP-014 | Scanner Agent Certificate Rotation | The platform SHALL support automated scanner agent certificate rotation: the agent SHALL request a new certificate at 80% of the certificate lifetime using the existing certificate for authentication (mTLS-bootstrapped renewal), with zero downtime and no manual IT Admin intervention required. | Security | IT Admin | Should | Certificate rotation without manual intervention is required for production scanner deployments at scale |

---

## 12. Migration & Fallback (Firebase Auth → Enterprise IdP)

### 12.1 Migration Path

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-MIG-001 | Firebase Auth Adapter Preserved | The Firebase Auth adapter SHALL remain a fully supported, non-deprecated adapter indefinitely; all existing ESIP tenants using Firebase Auth SHALL continue to work without change until they explicitly choose to migrate to an enterprise IdP. | Functional | All | Must | TECH-STACK.md §3; ESIP current state; non-breaking evolution is the design mandate |
| REQ-IAM-MIG-002 | Migration Wizard | The platform SHALL provide a guided migration wizard in the admin console (Firebase → Enterprise IdP) that: (1) configures and test-validates the target IdP in shadow mode; (2) maps Firebase UID → enterprise IdP subject identifier for all existing users; (3) migrates Firebase password hashes to the new IdP (where the IdP supports hash import, e.g., Firebase hash → Keycloak); (4) runs a parallel-validation phase; (5) enables a configurable switchover date with automatic cutover. | UX | IT Admin | Must | Firebase is the current auth; enterprises upgrading to SSO need a safe, guided migration path without re-creating user accounts |
| REQ-IAM-MIG-003 | User Account Linkage | The migration wizard SHALL support linking a Firebase UID to an enterprise IdP subject identifier (NameID / `sub`) for each user, storing the mapping in the ESIP user record so that pre-migration artifacts and audit logs remain attributed to the correct user post-migration. | Data | IT Admin | Must | Preserves history attribution; prevents "who created this artifact?" ambiguity after migration |
| REQ-IAM-MIG-004 | Parallel Auth Overlap Window | During migration, the platform SHALL support an overlap window (configurable: 1–30 days) during which both the Firebase adapter and the new enterprise IdP adapter are active; users can log in via either path; overlap window logs both auth methods per login event; admin can monitor the ratio and cut over when ready. | NFR | IT Admin | Must | Zero-downtime migration; users complete migration at their own pace during the overlap window |
| REQ-IAM-MIG-005 | Migration Completion Verification | After migration, the platform SHALL provide a migration completeness report listing: users migrated, users still on Firebase, users with broken links (no matching enterprise IdP subject), and date of last Firebase login per unlinked user. | Reporting | IT Admin | Must | Confirms successful migration; identifies stragglers before Firebase adapter is disabled |

---

### 12.2 SSO Failure Fallback

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-MIG-006 | SSO Failure Fallback Path | The platform SHALL detect IdP adapter availability degradation (via REQ-IAM-PA-005 health monitoring) and, when the primary IdP is unreachable for more than a configurable threshold (default: 60 seconds), SHALL: (a) surface a clear user-facing message ("Your organisation's login provider is unavailable"); (b) offer an admin-configurable fallback option: either a local emergency login (for accounts with a pre-enrolled local second factor) or break-glass access (§9.3). | NFR | All | Must | IdP outages do cause SSO failures; users need a defined fallback rather than a blank error screen |
| REQ-IAM-MIG-007 | Cached Authentication for In-Progress Sessions | The platform SHALL honour existing valid sessions during an IdP outage (sessions are validated against the local session store, not the IdP, on every request); users with active sessions SHALL be able to continue working; only new logins are blocked during an IdP outage. | NFR | All | Must | An IdP outage should not terminate all active user sessions; only new authentications are affected |
| REQ-IAM-MIG-008 | IdP Degradation Alert | The platform SHALL send an alert to the Tenant Admin and Security Admin within 2 minutes of detecting primary IdP adapter degradation, including the error type, time of first failure, and a link to the admin console health dashboard. | NFR | IT Admin, Security Admin | Must | Early warning of SSO outage; enables fast incident response |

---

## 13. Identity Audit & Observability

### 13.1 Identity Event Log

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|----|-------|-------------------|----------|------------|----------|-----------|
| REQ-IAM-AU-001 | Comprehensive Identity Event Logging | The platform SHALL log every identity event to the identity audit log (stored in V11 PLT-AUD infrastructure), including: successful login (user, IdP, auth method, MFA verified, IP, device), failed login (user, IdP, failure reason, IP), MFA events (enrolment, verification success/failure, step-up trigger, backup code use), SCIM operations (create/update/delete, group changes), IdP config changes, role assignments, session creation/revocation, token issuance/revocation, impersonation events, break-glass access, API key / service account operations, and SSO/SLO events. | Security | Security Admin | Must | SOC2 CC7.2; ISO/IEC 27001 A.12.4.1; GDPR Article 30; complete identity event log is the foundation of security incident investigation |
| REQ-IAM-AU-002 | Structured Log Format | Every identity audit event SHALL be structured (JSON) with mandatory fields: `event_id` (UUID), `event_type` (enum), `timestamp` (ISO 8601 UTC), `tenant_id`, `actor_id` (user GUID or service account ID), `actor_ip`, `target_id` (affected user or resource), `idp_name`, `auth_method`, `outcome` (SUCCESS/FAILURE/BLOCKED), and an optional `detail` object for event-specific fields. | Data | Security Admin | Must | Structured events enable SIEM parsing, alerting, and automated analysis; ad-hoc text logs are not queryable |
| REQ-IAM-AU-003 | Login Anomaly Detection | The platform SHALL apply configurable anomaly detection rules to the identity event stream, raising alerts for: (a) login from a new country or geolocation not seen in the past 30 days; (b) login velocity exceeding a configurable threshold (e.g., >5 failed attempts in 5 minutes → soft lock); (c) login outside business hours for accounts with a configured "expected hours" policy; (d) impossible-travel events (successful login from two geographically impossible IPs within a short window). | Security | Security Admin | Should | Proactive threat detection; reduces MTTD for compromised account scenarios; aligns with Azure AD Identity Protection / Okta Behaviour Detection patterns |
| REQ-IAM-AU-004 | Account Lockout on Failed MFA | The platform SHALL lock an account after a configurable number of consecutive MFA failures (default: 5 attempts, configurable 3–10), applying an exponential backoff before allowing re-attempt, and notifying the user and Security Admin of the lockout event; lockout SHALL be resettable by an admin or via the account recovery flow. | Security | Security Admin | Must | NIST SP 800-63B §5.2.2; brute-force MFA protection |
| REQ-IAM-AU-005 | Identity Audit Report | The platform SHALL provide a pre-built Identity Audit Report (exportable to CSV/PDF) for a configurable time window, containing: total login events by IdP, failed logins by user and reason, MFA enrolment status per user, SCIM provisioning/deprovisioning events, admin privilege changes, impersonation events, and API key usage summary. | Reporting | Security Admin, Tenant Admin | Must | SOC2 evidence package; enterprise security review artefact; compliance audit support |
| REQ-IAM-AU-006 | SIEM / SOC Export of Identity Events | The identity event log SHALL support real-time streaming to external SIEM platforms (Splunk HEC, Microsoft Sentinel Syslog/CEF, Sumo Logic, Datadog Logs) via a configurable identity event webhook / log-forwarding sink, in addition to the V11 PLT-AUD-004 general audit export. | Integration | Security Admin | Should | Enterprises with a Security Operations Centre (SOC) require identity events in their central SIEM; separate from general audit export for specialised identity tooling (e.g., Okta Identity Threat Protection feed) |
| REQ-IAM-AU-007 | Privacy-Safe Logging | The identity audit log SHALL never record the plaintext value of passwords, TOTP codes, hardware-key assertions, client secrets, or API key values; these SHALL be redacted in all log entries. | Security | All | Must | GDPR data minimisation; PCI DSS §10.5; prevents credential leakage via audit log access |

---

## Capability Area Summary

| # | Sub-area | Prefix | Requirements | Key Topics |
|---|----------|--------|-------------|-----------|
| 1 | IdP Port / Adapter Architecture | IAM-PA | 001–009 | `IIdentityProvider` interface, adapter catalogue, hot-swap, health monitoring, custom adapter SDK |
| 2 | Supported IdP Catalogue | IAM-IdP | 001–014 | Okta, Entra ID, Azure AD B2C, Auth0, Ping, OneLogin, Google Workspace, ADFS, Keycloak, Generic SAML/OIDC, LDAP/AD, Firebase (current), BYOI |
| 3 | SAML 2.0 | IAM-SA | 001–015 | SP metadata, ACS, signing, encryption, NameID, replay prevention, cert rotation, SP-/IdP-initiated |
| 4 | OIDC / OAuth 2.0 | IAM-OI | 001–011 | PKCE, discovery document, ID token validation, JWKS, claims mapping, refresh tokens, internal JWT |
| 5 | App-Gallery & Tile | IAM-AG | 001–011 | Okta OIN, Entra app gallery, SCIM OIN tile, self-service wizard, claims debugger, cert dashboard, staged rollout |
| 6 | SCIM 2.0 Provisioning | IAM-SC | 001–010 | SCIM endpoint, group→role mapping, JIT, deprovisioning, license reclaim, SCIM audit |
| 7 | Multi-IdP & Realms | IAM-MR | 001–010 | Multiple IdPs per tenant, home-realm discovery, B2B partner access, B2C Reviewer flows, email magic link |
| 8 | MFA / Passkeys / Conditional Access | IAM-MF | 001–013 | MFA delegation, TOTP, FIDO2/WebAuthn, passkeys, push MFA, step-up, IP allowlist, device trust |
| 9 | Sessions, SLO & Impersonation | IAM-SS | 001–015 | Idle/absolute timeout, concurrent sessions, SAML SLO, OIDC front-/back-channel logout, propagation, impersonation, break-glass |
| 10 | RBAC + ABAC + Delegated Admin | IAM-AZ | 001–017 | Role catalogue, permission catalogue, persona/license entitlements, ABAC engine, classification labels, per-artifact/field permissions, delegated admin |
| 11 | API Auth: Client-Credentials, API Keys, mTLS | IAM-AP | 001–014 | OAuth 2.0 client credentials, API keys (hashed), service accounts, mTLS for scanner agents, cert provisioning |
| 12 | Migration & Fallback | IAM-MIG | 001–008 | Firebase → enterprise IdP migration wizard, user linkage, overlap window, SSO failure fallback, cached sessions |
| 13 | Identity Audit & Observability | IAM-AU | 001–007 | Comprehensive event logging, structured format, anomaly detection, account lockout, identity audit report, SIEM export |

**Total requirements in this volume: ≈ 225**

---

## Cross-Reference Map

| Topic | This Volume | Cross-ref Volume |
|-------|-------------|-----------------|
| Secrets vault, JWT signing key storage, client-secret storage | Referenced (V25) | **Volume 25** (planned) |
| Server-side authz enforcement (middleware, policy engine runtime) | Referenced (V25) | **Volume 25** (planned) |
| Licensing seat enforcement, edition feature gating | REQ-IAM-AZ-003/004, REQ-IAM-SC-006, REQ-IAM-SC-010 | **Volume 13** (ONB-LR, ONB-ED) |
| Free Viewer / Reviewer license tiers | REQ-IAM-MR-008/009/010 | **Volume 17** (LIC sub-area) |
| Multi-tenancy data isolation kernel | Referenced (V11) | **Volume 11** (PLT-TEN) |
| Audit log storage infrastructure, tamper-evidence, SIEM export infra | REQ-IAM-AU-001/002/006 | **Volume 11** (PLT-AUD) |
| Enterprise deployment topologies (on-prem, air-gapped Keycloak) | REQ-IAM-IdP-009 | **Volume 19** (ENT-DEP) |
| Rust scanner agent provisioning, ADO-style build agent | REQ-IAM-AP-012/013/014 | **Volume 17** (AGT sub-area) |
| V11 §3 IAM summary (superseded by this volume) | §1–§11 (authoritative) | **Volume 11** §3 (PLT-IAM, PLT-SCM, PLT-ACC, PLT-EXT — summary cross-ref only) |

---

*Volume 24 — Enterprise Identity & Access (Pluggable SSO / IdP). Authored 2026-06-23. Prefix: IAM. ≈ 225 requirements across 13 capability areas.*
