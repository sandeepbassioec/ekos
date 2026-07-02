# EKOS Vendor-Neutrality Pass (P-A)

> **Gap-closure P-A** (cross-architect review #10). ChatGPT: "Too many implementation
> hints — Redis, Kafka, ClickHouse, etc. Those should be examples, never requirements.
> Do a cleanup pass to ensure no vendor name accidentally becomes an architectural
> commitment." This document records the audit + the global ruling.
> **Done:** 2026-07-01 (Opus). Enforces [BOOK-1](BOOK-1-KNOWLEDGE-CONSTITUTION.md) Law 10.

---

## 1. The Ruling (binding)

- **Book 1 Law 10** is authoritative: *every external dependency sits behind a port;*
  **all vendor/product names anywhere in the EKOS specification are illustrative
  examples, never requirements.** Where longevity depends on it, **open standards** are
  preferred over proprietary formats. *(openCypher/GQL, JSON-LD/RDF, OCI, W3C VC/DID,
  CloudEvents, OpenTelemetry, X.509, RFC 3161, glTF.)*

- A vendor name in any volume MUST be read as: *"e.g., / candidate / reference
  implementation — pluggable."* No vendor is an architectural commitment.

---

## 2. Audit Result

A scan for the named vendors (Redis, Kafka, ClickHouse, Neo4j, PostgreSQL, Stripe,
Bedrock, Okta, Llama, Mistral, etc.) found **54 occurrences across 13 files**. Each was
reviewed for framing:

| File group | Vendors | Framing — confirmed | Verdict |
|---|---|---|---|
| Vol 01 / 18 (graph, kernel) | Neo4j, Kafka, ClickHouse | "candidates", "illustrative", behind the Storage Abstraction port (EKOS-01-006, 18-031) | ✅ neutral |
| Vol 03 / 21 (AI) | Bedrock, Claude, Llama, Mistral, Ollama | "provider-pluggable", "default + swappable", "sovereign open models" (EKOS-03-028/057) | ✅ neutral |
| Vol 09 (identity) | Okta, Entra, Auth0, Keycloak | "pluggable IdP" (EKOS-09-011) | ✅ neutral |
| Vol 13 (economy) — **highest density (18)** | Stripe (Connect/Treasury) | most load-bearing; explicitly "**illustrative & pluggable; Stripe is the reference, not a hard binding**" (EKOS-13-044) | ✅ neutral (flagged) |
| Vol 15 (tech-stack) — **9** | PostgreSQL/SQL Server/MySQL/Oracle, Neo4j, Kafka/ClickHouse, Redis | the *decision* volume; all behind repository/provider-capability ports; "pluggable + benchmark", "named only as illustrative candidates" (EKOS-15-013/032) | ✅ neutral (flagged) |
| Vol 07/10/11/12, BUILD-PLAN, INDEX | scattered | "e.g.", "candidates", "reference" | ✅ neutral |

**Conclusion:** the specification is **already vendor-neutral** — every vendor name is
framed as illustrative and sits behind a port. No vendor name functions as a requirement.
The two highest-density / most load-bearing spots (**Stripe in Vol 13**, **storage/DB
engines in Vol 15**) carry explicit "not a hard binding / pluggable" statements.

---

## 3. Residual Care Items (not defects — watch during build)

- **Vol 13 (Stripe):** the *fintech-compliant escrow* design leans on Stripe Connect+
  Treasury concepts. The **capability** (compliant escrow sweep) is the requirement;
  Stripe is the reference adapter. A second payment/escrow provider SHOULD be evaluated
  before lock to prove pluggability (build-plan OQ-13-A).
- **Vol 15 (DB/event-store):** the concrete engine pick is deliberately deferred to a
  **benchmark** (OQ-15-A), keeping the choice open. No premature commitment exists.
- **Open standards** chosen where longevity matters are **requirements** (not vendors):
  openCypher/GQL (query), JSON-LD/RDF (no-lock-in export), OCI (manifests), W3C VC/DID
  (credentials), CloudEvents (events), X.509/RFC-3161 (signing). These are *standards*,
  not products — correctly load-bearing.

---

## 4. Standing Enforcement

- Any **new** spec text introducing a vendor name MUST frame it as illustrative + behind
  a port (Law 10). The Design Methodology's five lenses (lens 3, 20-year survivability)
  catch vendor lock-in attempts.
- The **pluggable-infra ports register** (Vol 15 §12) is the single source for which
  capabilities are ports; every port has ≥1 reference adapter (illustrative) + a
  provider-capability descriptor for graceful degradation.

> **Verdict: PASS.** EKOS is vendor-neutral by construction. No cleanup edits required;
> this audit + the standing rule satisfy P-A.
