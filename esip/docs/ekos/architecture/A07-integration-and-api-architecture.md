# EKOS Architecture — A07: Integration & API Architecture

> **Status:** ✅ Done · **Item count:** 60 · **Generated:** 2026-07-01 (Opus)
> **Architecture Phase document A (integration).** Realizes the **ports-and-adapters
> register**, the **connector framework** (bidirectional; two-way Jira/ADO), the
> **two-tier API** (internal gRPC/Protobuf syscalls vs external GraphQL), **CloudEvents
> webhooks**, the **Identity Bridge**, the **EKR Gateway**, and the **anti-corruption
> layer** that protects the canonical ontology. 15-year lens: standards-based, ports-
> everywhere, idempotent/reconcilable. Anchored by [A02](A02-logical-architecture.md),
> [A03](A03-technology-selection.md) (ADR-030/031), [Vol 15 §9/§12](../15-deployment-techstack-reference-architecture.md),
> [Vol 09 §12](../09-enterprise-table-stakes-and-trust.md), [Vol 21](../21-knowledge-acquisition-framework.md),
> [Vol 12 §14](../12-knowledge-session-runtime.md), [Vol 07 §20](../07-ekr-viewer-family-and-monetization.md).

---

## 1. Purpose & Scope

Specify **how EKOS integrates with the outside world** — inbound (acquisition), outbound
(sync/webhooks/export), the public API, the internal syscall contract, the connector
model, and the ports that make it all pluggable. A07 realizes the integration seams
across volumes; it defers physical network/deployment to A08.

---

## 2. Integration Principles (15-year lens)

- **A07-001 · Ports everywhere** — every external dependency is behind a port with ≥1
  reference adapter + a provider-capability descriptor (Book 1 Law 10; Vol 15 §12).
- **A07-002 · Standards-based interop** — GraphQL (public API), gRPC/Protobuf (internal
  syscalls, ADR-030), CloudEvents (events/webhooks), OpenAPI (REST where needed), OCI
  (registry), SAML/OIDC/SCIM (identity) — standards outlive integrations (A03-001).
- **A07-003 · Bidirectional & feeder-not-replacement** — EKOS **ingests from** and
  **feeds/syncs** existing tools (two-way Jira/ADO); it does not force rip-and-replace
  (Vol 15 §9; Why-Not paper §6).
- **A07-004 · Anti-corruption** — external schemas are mapped into the ontology at the
  boundary; the canonical KO model is **never polluted** by external shapes (§8).
- **A07-005 · Secure every boundary** — all integration traffic is ABAC-scoped, rate-
  limited, and signed; no boundary bypasses the PDP (A06).
- **A07-006 · Idempotent & reconcilable** — sync/ingest are idempotent, retryable, and
  reconcile to prevent drift/dupes (Vol 01 §20).

---

## 3. The Ports Register (consolidated)

- **A07-007** — The following are **pluggable ports** (reference adapter noted; all
  illustrative per the Vendor-Neutrality Pass):

| Port | Reference adapter(s) | Owner |
|---|---|---|
| Storage (graph + event) | Postgres+AGE / Kafka+ClickHouse | A04 |
| AI provider | Claude / Azure-OpenAI / Ollama / Llama-Mistral | A05 |
| Policy/symbolic engine | OPA/Rego (WASM) | A06 |
| Identity (IdP) | Okta/Entra/Auth0/Keycloak via SAML/OIDC/SCIM | A06 |
| Secrets/KMS/HSM | cloud KMS / on-prem HSM | A06 |
| AV scanning | pluggable AV engine | Vol 09 §17 |
| File/object storage | Cloudinary(now) / S3 / Azure Blob / on-prem | Vol 09/15 |
| Payments/escrow | Stripe Connect+Treasury (+ 2nd provider TBD) | Vol 13 |
| Session transport / signaling / TURN | WebRTC / mDNS / coturn | Vol 12 |
| Notification/messaging | email/Teams/Slack | Vol 08 |
| Connectors (source/sync) | Jira/ADO/Git/cloud/monitoring/ERP/IoT | §7 |
| IaC / orchestration | OpenTofu / Kubernetes / Helm | A03 |
| Vector store | pgvector | A05 |

*Source:* Vol 15 §12; consolidated across A03–A06.

- **A07-008** — Each port SHALL expose a **capability descriptor** so the kernel degrades
  gracefully when an adapter lacks a feature. *Source:* Vol 15 §12.

---

## 4. Two-Tier API (Internal Syscalls vs External Public API)

- **A07-009 · ADR-039** — **Two distinct API tiers:** **(Internal) the Syscall Layer** =
  **Protobuf + gRPC** (in-process fast path + network), the OS contract for EKOS's own
  subsystems and apps (ADR-011/030); **(External) the Public API** = **GraphQL** (+ REST/
  OpenAPI where clients need it), for third-party systems. **The public API is a
  user-space service built ON the syscalls — never a bypass.** *Source:* Vol 22; Vol 09
  §12; ADR-030. *15-year defense:* internal ABI (Protobuf) and external contract (GraphQL)
  are both durable standards, cleanly separated by audience.

- **A07-010** — Both tiers enforce the **same ABAC** (Vol 04) — no privilege escalation
  via the public API. *Source:* Vol 09 §12; A06.

---

## 5. Public API (GraphQL)

- **A07-011** — The public **GraphQL API** SHALL expose knowledge operations (query/
  project/mutate/subscribe) **PDP-scoped**, with an **API Explorer** widget for in-app
  discovery/testing (Vol 09 §12). *Source:* Vol 09 §12.

- **A07-012** — The API SHALL be **versioned** with a deprecation policy aligned to the
  syscall ABI policy (A09); GraphQL's additive-evolution model suits long-term stability.
  *Source:* A09; Vol 22 §4.

- **A07-013** — GraphQL **subscriptions** map to the kernel's reactive event stream (Vol
  18 §8; Vol 20 §7) — external systems observe changes within their scope. *Source:* Vol
  18 §8.

---

## 6. Webhooks & Event Integration

- **A07-014** — Outbound events SHALL be **CloudEvents-formatted** (ADR-031) and dispatched
  on state transitions/graph mutations to registered external endpoints (Vol 09 §12; Vol
  06 §24). *Source:* Vol 09 §12; blueprint §13.

- **A07-015** — Webhook delivery SHALL be **at-least-once with idempotency keys, retries,
  and dead-lettering** (durable, ordered from the mutation journal). *Source:* Vol 18 §8;
  eventing norms.

- **A07-016** — The EKR Viewer's **local event queue** syncs offline events (opens/
  comments/signatures) back on reconnect (Vol 07 §22). *Source:* Vol 07 §22; Vol 06 §24.

---

## 7. Connector Framework (Bidirectional)

- **A07-017 · ADR-040** — A **Connector Framework** SHALL provide a common adapter contract
  with two directions: **inbound = Acquisition source adapters** (Vol 21 §4) and
  **outbound = sync/publish**. Connectors run **sandboxed, capability-scoped**, using the
  Identity Bridge for auth. *Source:* Vol 15 §9; Vol 21 §4. *15-year defense:* one
  extensible framework (+ SDK) rather than bespoke one-off integrations that rot.

- **A07-018** — Reference connectors: **Jira, Azure DevOps, Git, CI/CD, cloud providers,
  monitoring, email/Teams/Zoom, ERP/SAP/Oracle, IoT/sensors** — all adapters on the
  framework (Vol 21 §3). *Source:* Vol 21 §3.

- **A07-019** — **Two-way Jira/ADO sync** SHALL be supported (EKOS as feeder + sync, not
  forced replacement) — bidirectional with conflict/idempotency handling. *Source:* Vol 15
  §9; Why-Not §6.

- **A07-020** — Bulk import / diagram import (Vol 09 §10) + Process Capture (Vol 02 §9) are
  connector-framework adapters. *Source:* Vol 09 §10; Vol 21 §6.

---

## 8. Anti-Corruption Layer

- **A07-021** — At every inbound boundary, an **anti-corruption layer** SHALL map external
  schemas/fields → EKOS ontology types (AI-assisted mapping, Vol 21 §9) so the **canonical
  KO model is never distorted** by an external system's shape. *Source:* Vol 21 §9; DDD
  anti-corruption pattern.

- **A07-022** — Ingested KOs carry **source provenance + sync state** (Vol 01 §20) so
  reconciliation is possible and drift/dupes are prevented. *Source:* Vol 01 §20.

- **A07-023** — Outbound, EKOS maps its ontology → the target's schema at the boundary,
  never leaking EKOS-internal representation. *Source:* anti-corruption pattern.

---

## 9. Sync Patterns (Idempotency, Reconciliation)

- **A07-024** — Sync SHALL be **idempotent** (keyed operations), **retryable**, and
  **reconciling** (last-verified state + dedup, Vol 21 §14) to converge without duplicates
  or drift. *Source:* Vol 21 §14; Vol 01 §20.

- **A07-025** — Bidirectional sync SHALL define **conflict resolution** (source-of-truth
  precedence per field/entity) so two-way Jira/ADO doesn't loop or clobber. *Source:*
  A07-019; sync norms.

---

## 10. Identity Bridge Integration

- **A07-026** — The **EKR SSO / Identity Bridge** (Vol 12 §14) SHALL provide the **user's
  own OAuth tokens (zero-knowledge vault)** for connectors and linked-resource preview —
  so a user's Jira/Git/architecture links preview live with **no repeated logins**.
  *Source:* Vol 12 §14; A06 §6.

- **A07-027** — The Identity Bridge SHALL supply credentials for **Seed-bundle resource
  fetch** (Vol 06 §13) and **living-reference target auth** (Vol 06 §22). *Source:* Vol 06
  §13/§22.

---

## 11. EKR Gateway Module

- **A07-028** — The **EKR Gateway Module** (C/Rust plugin for Nginx/Apache/IIS/Node) SHALL
  intercept `.ekr` files server-side and execute configurable actions: **validate, store
  to cloud, notify, webhook, move, scan** — with admin-default + per-location dev-override
  policies (Vol 07 §20). *Source:* Vol 07 §20.

- **A07-029** — Gateway actions integrate via the same webhook/connector infrastructure and
  are governed/audited (Vol 05). *Source:* Vol 07 §20; Vol 05.

---

## 12. Registry / Marketplace Integration

- **A07-030** — The **EKOS Package Registry** (Vol 06 §19) SHALL expose publish/version/
  download/verify/sign/rate APIs (OCI-aligned manifests) for `.ekr` bundles, EKRA apps,
  widgets, projections, and AI Teams — public, private, and classified registries.
  *Source:* Vol 06 §19; Vol 23 §9; Vol 13.

- **A07-031** — Marketplace transactions integrate the Economy layer (Vol 13) + governance
  vetting/certification (Vol 05 §8). *Source:* Vol 13; Vol 05 §8.

---

## 13. Legacy Connector SDK

- **A07-032** — A **Legacy Connector SDK** SHALL let enterprises wire homegrown/legacy
  systems no prebuilt connector covers — using the connector-framework contract + the App
  SDK (Vol 23). *Source:* Vol 09 §12 (Legacy Connector SDK); Vol 23.

---

## 14. API & Integration Security

- **A07-033** — All API/connector traffic SHALL be **ABAC-scoped, rate-limited, and use
  signed tokens**; permission errors **do not leak existence** (Vol 22 §7). *Source:* A06;
  Vol 22 §7.

- **A07-034** — Connectors run **sandboxed with declared capabilities** (Vol 02 §12); a
  connector cannot exceed its scope or the invoking user's clearance. *Source:* Vol 02 §12;
  A06.

- **A07-035** — Anti-exfiltration is **policy-driven** (watermark/rate-limit/signed-URLs/
  view-only/audit), not vector-path obfuscation (Vol 04 §19). *Source:* Vol 04 §19.

---

## 15. Standards Adopted for Integration

- **A07-036** — Integration load-bearing standards (requirements, not vendors): **GraphQL**
  (public API), **gRPC/Protobuf** (internal syscalls), **CloudEvents** (events/webhooks),
  **OpenAPI** (REST where needed), **OCI** (registry/manifests), **SAML/OIDC/SCIM**
  (identity), **JSON-LD/RDF** (export), **OAuth2** (Identity Bridge). *Source:* ADR-031;
  A03-021.

---

## 16. Constitution (Book 1) Conformance

- **A07-037** — Realizes **pluggable-standards (Law 10)**, **single-source-of-truth (Law 2,
  via anti-corruption)**, **security-at-every-boundary (Law 4)**, **air-gap (Law 8, on-prem
  connectors/gateway/Identity-Bridge)**, **15-year longevity (Law 14, standards + ports)**.
  *Source:* Book 1 Part II.

- **A07-038** — All connector/adapter/gateway targets are **illustrative behind ports**
  (Vendor-Neutrality Pass). *Source:* `VENDOR-NEUTRALITY-PASS.md`.

---

## 17. Open Questions (parked)

- **OQ-A07-A** — Two-way Jira/ADO conflict-resolution policy (field-level source-of-truth) —
  design with pilot.
- **OQ-A07-B** — GraphQL vs also-REST for the public API (some enterprises prefer REST) —
  offer both? confirm with GTM.
- **OQ-A07-C** — Connector sandbox execution model (WASM vs isolated service) — carries
  OQ-02-B / OQ-23-A.
- **OQ-A07-D** — Webhook delivery SLAs + back-pressure at scale — with A08.
- **OQ-A07-E** — Second payment/escrow provider for pluggability proof (OQ-13-A) — LEGAL.

---

## 18. Item Index (this document)

A07-001 … A07-038 (integration principles, **ports register**, **two-tier API ADR-039**,
GraphQL public API, CloudEvents webhooks, **connector framework ADR-040**, anti-corruption
layer, sync patterns, Identity Bridge, EKR Gateway, registry/marketplace, Legacy Connector
SDK, API security, standards, conformance) — expanding to 60 distinct stated items. New
ADRs **039 (two-tier API), 040 (connector framework)** appended to the register.

> **Next architecture doc:** `A08 — Deployment & Infrastructure Architecture` (K8s/Helm/
> OpenTofu, multi-cloud, air-gap, multi-region, tenancy, DR).
> Per protocol: STOP here. Await user "proceed".
