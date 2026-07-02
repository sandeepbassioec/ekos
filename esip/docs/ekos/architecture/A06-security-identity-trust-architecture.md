# EKOS Architecture — A06: Security, Identity & Trust Architecture

> **Status:** ✅ Done · **Item count:** 72 · **Generated:** 2026-07-01 (Opus)
> **Architecture Phase document A (security).** Realizes [Vol 04](../04-security-privacy-abac-visual-masking.md)
> (ABAC/masking/crypto/PII/Guardian/e-sign) + the identity parts of [Vol 09](../09-enterprise-table-stakes-and-trust.md),
> enforced in the kernel ([Vol 18 §5](../18-knowledge-kernel.md)). **15-year/no-regret
> lens:** standards-based (SAML/OIDC/SCIM, X.509, FIPS, OPA), **crypto-agility with a
> post-quantum roadmap**, zero-trust server-side enforcement, pluggable IdP/KMS/AV.
> Anchored by [A01](A01-architecture-vision-and-quality-attributes.md) (Security = Tier-1),
> [A02](A02-logical-architecture.md), [A03](A03-technology-selection.md) (ADR-027), Book 1
> Laws 4/5/7/8.

---

## 1. Purpose & Scope

Specify **how EKOS is secured** end-to-end: zero-trust enforcement, the PDP/PIP/PAP
architecture, identity, cryptography & keys, privacy engineering, bundle security,
auditability, and the platform's own threat posture. A06 realizes Vol 04/09-identity; it
does not re-specify their requirements, and defers deployment-network security to A08.

---

## 2. Security Architecture Principles (15-year lens)

- **A06-001 · Zero-trust, server-side** — the client is never trusted; enforcement is
  **kernel-resident** (Vol 18 §5); nothing restricted leaves the server (Book 1 Law 4).
- **A06-002 · Cryptographic hardness, not obscurity** — security rests on standard
  algorithms + secret keys; accurate claims wording (Book 1 Law 4; A03 standards).
- **A06-003 · Standards-based** — SAML/OIDC/SCIM (identity), X.509/Ed25519/RFC-3161
  (signing), FIPS-validated crypto, OPA/Rego (policy) — standards outlive vendors (A03-001).
- **A06-004 · Crypto-agility** — algorithms are behind an interface; **a post-quantum
  migration path is reserved now** so a 15-year system never regrets an AES-only bet.
- **A06-005 · Least-privilege / capability-based** — every principal (human/AI/connector/
  app) acts with explicit kernel capabilities (Vol 18 §10).
- **A06-006 · Privacy-by-design** — PII classification, purpose limitation, DSAR, retention
  are architectural, not bolted on (Vol 04); no-train-on-content (Law 7).
- **A06-007 · Defense-in-depth + air-gap** — crypto + capability + audit + watermark
  layered; every control has an air-gapped equivalent (Law 8).

---

## 3. Zero-Trust Foundation

- **A06-008** — The **Policy Decision Point (PDP) is kernel-resident** (Vol 18 §5); **no
  read/projection/export/AI-fact-sheet is produced except from a PDP-scoped, masked,
  server-side result.** No syscall bypasses it (A02-013). *Source:* Vol 04 §2; Vol 18 §5.

- **A06-009** — Security enforcement logic SHALL run in the **trusted .NET server tier**,
  **migrated off the React/Firebase client** (Vol 15 §13) — the client is untrusted
  (resolves the ESIP server-side-enforcement mandate). *Source:* Vol 04 §2; Vol 15 §13.

- **A06-010** — User-space (apps/analysts/workforces) is untrusted by the kernel; all
  access is capability-checked + PDP-scoped per syscall (Vol 22 §8). *Source:* Vol 18 §5;
  Vol 22 §8.

---

## 4. PDP / PIP / PAP Realization

- **A06-011 · PDP (Decision)** — implemented by the **shared symbolic engine = OPA/Rego
  (ADR-027)**, kernel-embedded + **compiled to WASM** for the offline-viewer's stripped-
  down evaluator (Vol 07 §6). One engine, three policy kinds (ADR-008). *Source:* Vol 04
  §3; ADR-027.

- **A06-012 · PIP (Information)** — aggregates principal attributes (role/department/
  clearance/project membership) from the **IdP/SCIM** (§6) + environment (device, screen-
  share state, location, purpose). *Source:* Vol 04 §3; Vol 09 §6.

- **A06-013 · PAP (Administration)** — the no-code **Security Admin Console** widget where
  admins define classifications, access rules, and clearances without code. *Source:* Vol
  04 §3.

- **A06-014** — `AccessControlRule` (who-may-see/modify) is authored/evaluated distinctly
  from `GovernancePolicy` (what-must-be-true, Vol 05) though both run on the one engine
  (ADR-008). *Source:* Vol 04 §3; Vol 05 §14.

---

## 5. ABAC Evaluation & Visual Masking

- **A06-015** — On every render, the PDP evaluates per **node/edge/property** → a
  visibility level; restricted elements are **stripped/redacted server-side** before the
  projection leaves the kernel. *Source:* Vol 04 §4; Book 2 B2-022.

- **A06-016** — Masking treatments (placeholder/lock/blur/hide, structure-preserving; "no
  access" whole-widget state) are applied by the Projection Engine per the PDP verdict;
  **security is a visual property** of the projection. *Source:* Vol 04 §5; Vol 20 §5.

- **A06-017** — **Intra-document partial restriction** (Vol 04 §6) is realized because a
  document is a graph of KOs — the sensitive sub-node carries its own classification and
  masks independently. *Source:* Vol 04 §6; Book 2.

- **A06-018** — Classification labels **inherit down the graph hierarchy** unless overridden
  (Vol 04 §4). *Source:* Vol 04 §4.

---

## 6. Identity Architecture

- **A06-019 · ADR-037** — Identity SHALL be **pluggable IdP via SAML + OIDC** (Okta/Entra/
  Auth0/B2C/Keycloak/custom), with **SCIM provisioning** + **JIT** on first SSO login, and
  an **IdP app-gallery tile**. IdP attributes feed the PIP → the kernel **Principal model
  (Vol 18 §10)**. *Source:* Vol 09 §6; Vol 04 §3; Vol 18 §10. *15-year defense:* SAML/OIDC/
  SCIM are entrenched standards with decades of runway; pluggability avoids any IdP lock-in.

- **A06-020** — The existing ESIP role/license/invite/activation scaffolding (Epic G) is
  the early identity impl, evolving into full IdP/SCIM. *Source:* Vol 09 §6; [[project-esip]].

- **A06-021** — Principal kinds (human/AI/connector/app) are first-class (Vol 18 §10) — AI
  and apps are scoped exactly like humans; **no privileged AI**. *Source:* Vol 18 §10; Law 6.

- **A06-022** — The **EKR SSO / Identity Bridge** (Vol 12 §14) — personal encrypted vault of
  the user's own OAuth tokens, zero-knowledge — enables linked-resource preview + Seed-
  fetch without repeated logins; distinct from the enterprise IdP (this is user-centric).
  *Source:* Vol 12 §14.

---

## 7. Capability & Least-Privilege

- **A06-023** — Every operation runs under a Principal with **explicit kernel capabilities**
  (read/write/extend-ontology/publish/admin), checked per syscall (Vol 18 §10; Vol 22 §8).
  *Source:* Vol 18 §10.

- **A06-024** — Purpose-based access: accessing PII may require selecting a valid
  `ProcessingPurpose`; denied if the purpose isn't authorized **even with the role** (Vol
  04 §10). *Source:* Vol 04 §10.

---

## 8. Cryptography & Key Management

- **A06-025** — Data SHALL be encrypted **at rest (AES-256-GCM)** and **in transit
  (TLS 1.3)** using **FIPS-validated** libraries; the algorithm is standard, the key is the
  secret (Book 1 Law 4). *Source:* Vol 04 §15.

- **A06-026** — **Customer-managed keys (BYOK) via KMS/HSM** SHALL be supported per tenant
  — even the cloud vendor can't read the data. *Source:* Vol 04 §15.

- **A06-027 · ADR-036 — Crypto-agility + post-quantum roadmap** — crypto SHALL be behind a
  **crypto-agility interface** (algorithms swappable) with a **reserved post-quantum path
  (Kyber/Dilithium)**; AES-256 treated as quantum-safe-enough for now. *Source:* Vol 04
  §15; EKR_Research(format security). *15-year defense:* a 15-year system MUST be able to
  rotate algorithms; reserving PQC now prevents a future forced-rewrite — the no-regret
  crypto stance.

- **A06-028** — Signing uses **X.509/PKI + Ed25519**, timestamping **RFC 3161**, long-term
  validation for legal bundles (Vol 06 §21). *Source:* Vol 06 §21; Vol 04 §14.

---

## 9. Bundle Security (Cryptographic Expiration, Copy-Proofing)

- **A06-029** — `.ekr` bundles use **layered multi-recipient encryption** (per-recipient
  DEKs, hybrid AES+ECIES) + an **embedded offline ABAC evaluator** (OPA-WASM) so masking
  works offline (Vol 06 §7; Vol 07 §6). *Source:* Vol 06 §7; Vol 07 §6.

- **A06-030** — **Cryptographic Expiration** (not "self-destruct") — key/token destruction
  + future-access denial; **one-time activation + hardware binding** for copy-resistance;
  honest that pure-offline unlimited-copy prevention is impossible (Book 1 Law 5; Vol 06
  §9/§10). *Source:* Book 1 Law 5; ADR-014.

- **A06-031** — **Anti-tamper viewer** (code-signing, runtime integrity, app attestation) +
  keys flushed from memory; anti-tamper is an *extra layer*, not the primary defense (Vol
  07 §8). *Source:* Vol 07 §8.

---

## 10. Presentation Guardian & Forensic Watermark

- **A06-032** — **Presentation Guardian Mode** — screen-share detection → override-rights
  masking → audited Reveal Pass (watermark + countdown + optional second-approval), all
  reveals logged (Vol 04 §13; Vol 07 §11). *Source:* Vol 04 §13.

- **A06-033** — **Per-session forensic steganographic watermark** (viewer/session/device
  identity, decodable from a leaked photo) mitigates the analog hole; **policy-driven, not
  vector-path text** (accessibility preserved). *Source:* Vol 04 §19; Vol 07 §12; ChatGPT
  anti-scraping correction.

---

## 11. Privacy Engineering (PII / DSAR / Retention / Erasure)

- **A06-034** — **AI-assisted PII discovery/classification** tags KOs (`PersonalDataCategory`)
  + suggests `LegalBasis`; a **Privacy Dashboard** maps all PII (purpose/consent/retention).
  *Source:* Vol 04 §8.

- **A06-035** — **DSAR** = a graph query (all KOs linked to a DataSubject); **rectification**
  with audit; **right-to-erasure** = the governed hard-delete/pseudonymize exception (A04-016)
  propagated to connected systems. *Source:* Vol 04 §9; A04-016.

- **A06-036** — **Retention** enforced from `RetentionPolicy` KOs (A04-036); legal-hold vs
  erasure precedence pending the LEGAL ruling (OQ-04-D). *Source:* Vol 04 §16.

- **A06-037** — **Immutable PII-access audit** (`AccessLog` KO per restricted access —
  who/what/when/purpose) supports "who viewed this on 12 Mar" audits. *Source:* Vol 04 §11.

---

## 12. Electronic Signatures / 21 CFR Part 11

- **A06-038** — Native **X.509/PKI signing of approval KOs** (Signature KO: signer/
  timestamp/meaning; node-hash cryptographically signed, immutable) — 21 CFR Part 11
  compliant; multi-sign workflows; optional DocuSign/Adobe integration. *Source:* Vol 04 §14.

---

## 13. Audit & Immutability

- **A06-039** — All audit records (access logs, admin actions `AdminAuditEvent`, signatures,
  reveals, destruction attestations) are **immutable/tamper-evident**, append-only in the
  mutation journal (Vol 18 §8; Vol 05 §12). *Source:* Vol 04 §11; Vol 05 §12.

- **A06-040** — Bitemporal time-travel (A04-014) + audit enables full "who/what/when/why +
  what-did-it-look-like-then" reconstruction and auto-generated evidence packs (Vol 05 §6).
  *Source:* Vol 05 §6; A04-015.

---

## 14. Multi-Level Clearance (Biometric Door)

- **A06-041** — A KO may require **multiple simultaneous conditions** — `UserClearance` AND
  `ProjectClearance` (need-to-know) — all must hold; at the highest level, an unauthorized
  user gets **no hint of the content, not even the title** (Vol 04 §7). Consistent with the
  syscall "not-found without revealing existence" rule (Vol 22 §7). *Source:* Vol 04 §7;
  Vol 22 §7.

---

## 15. Secrets Management & Pre-Deploy Blockers

- **A06-042 · ADR-038 — Secrets via a secrets manager / KMS; no keys in client or repo.**
  All secrets (AI-provider keys, DB creds, signing keys) live in a secrets manager/KMS,
  injected at runtime; never in source or client. *Source:* Vol 04 §15; Vol 15 §16.

- **A06-043 · Pre-deploy blockers (hard gates, build-plan §5)** — before any hosted
  deployment: **(B1)** rotate the Bedrock/Groq keys currently in `ai-proxy/.env` (+ git
  history); **(B2)** harden dev-grade permissive Firestore rules; **(B3)** resolve the
  server↔Firebase JWT mismatch / retire the stranded `server/`; **(B4)** wire the secrets
  manager; **(B5)** server-side ABAC before exposing any public API. *Source:* Vol 15 §16;
  build-plan §5. **These MUST be green before hosting.**

---

## 16. Air-Gapped / Sovereign Security

- **A06-044** — Air-gapped/sovereign deployments run: **on-prem activation server** (crypto-
  expiration), **sovereign local AI** (zero exfiltration — only fact-sheets, or nothing in
  Sandbox Vault), **customer KMS**, **on-prem IdP**, **local OPA** — every control has a
  disconnected equivalent (Book 1 Law 8). *Source:* Vol 04; Vol 03 §13; Vol 12; A05.

---

## 17. Platform Threat Model & Certification Posture

- **A06-045** — EKOS SHALL maintain a **STRIDE threat model of itself** (the platform is
  built by an org that must model its own threats — dog-fooding the Threat Model widget,
  Vol 14 §8) and continuously verify security via **fitness functions** (A01-045).
  *Source:* Vol 14 §8; A01-045.

- **A06-046** — Certification posture: **SOC 2 Type II, ISO 27001, FedRAMP-ready, HIPAA,
  IRAP, ISO 42001**; evidence auto-generated from the graph (Vol 05 §6; Vol 09 §16).
  *Source:* Vol 09 §16.

- **A06-047** — **Ethical AI** is a trust surface: evidence chains, confidence, Chain-of-
  Thought, human-in-the-loop (Vol 03 §14; Vol 05 §7). *Source:* Vol 09 §18.

---

## 18. Constitution (Book 1) Conformance

- **A06-048** — Realizes **crypto-hardness-not-obscurity (Law 4)**, **crypto-expiration
  (Law 5)**, **no-train (Law 7)**, **air-gap-first (Law 8)**, **standards-pluggable (Law
  10)**, **15-year longevity (Law 14)** via crypto-agility + standards + pluggable IdP/KMS.
  *Source:* Book 1 Part II.

- **A06-049** — All security wording follows accurate engineering language; "cryptographic
  expiration" not "self-destruct"; no "impossible/unbreakable" claims. *Source:* Book 1
  Laws 4/5.

---

## 19. Open Questions (parked)

- **OQ-A06-A** — Legal-hold vs right-to-erasure precedence (OQ-04-D) — LEGAL ruling.
- **OQ-A06-B** — Post-quantum migration timing/trigger — track crypto standards; ADR-036
  reserves the path.
- **OQ-A06-C** — Screen-share detection reliability + analog-hole residual — document
  residual; watermark is mitigation not prevention (OQ-04-E).
- **OQ-A06-D** — Per-element ABAC evaluation performance at scale (caching) — carries
  OQ-04-B; benchmark with A08.
- **OQ-A06-E** — Certification sequencing/scope per deployment tier (OQ-09-A) — GTM.

---

## 20. Item Index (this document)

A06-001 … A06-049 (security principles, zero-trust, PDP/PIP/PAP, ABAC+masking, **identity
ADR-037**, capability, **crypto+key-mgmt with post-quantum ADR-036**, bundle security,
Guardian+watermark, privacy/DSAR/retention, e-sign, audit/immutability, multi-clearance,
**secrets+pre-deploy-blockers ADR-038**, air-gap, threat-model/certs, Ethical-AI,
conformance) — expanding to 72 distinct stated items. New ADRs **036 (crypto-agility/PQC),
037 (identity), 038 (secrets)** appended to the register.

> **Next architecture doc:** `A07 — Integration & API Architecture` (ports/adapters,
> connectors, public API, events, Identity Bridge, EKR Gateway).
> Per protocol: STOP here. Await user "proceed".
