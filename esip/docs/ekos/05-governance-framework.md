# EKOS Volume 05 — Governance Framework

> **Status:** ✅ Done · **Requirement count:** 84 · **Generated:** 2026-06-30 (Opus)
> **Sources mined:** `ConfusionClarity.txt` (6-pillar Governance deep-dive +
> Governance Workbench Widget; table-stakes #4 Sandbox/Staging, #10 Feature Flags,
> #11 Admin Audit; future domain E Model Registry/MLOps), `blueprint §7` (Governance
> Framework + Workbench), `research_across_llms.txt` (final spec §7), Vol 03 deferred
> AI-governance *program*, [[project-esip]] C4 (AI auto-apply resets approval gate),
> Vol 04 OQ-04-A (policy-engine unification). Honors [00-MASTER-INDEX](00-MASTER-INDEX.md)
> §2.

---

## 1. Scope & Boundary

**In scope:** governance as a first-class dimension baked into every node/edge/widget/
AI interaction — the six pillars (**Policy-as-Code**, **Federated Access Governance**,
**Quality & Compliance Gates**, **Immutable Audit & Decision DNA**, **AI Governance /
AI Constitution**, **Marketplace & Ecosystem Governance**); the **Governance
Workbench** widget (Policy Canvas, Violation Dashboard, Audit Explorer, Exception
Management); **exception/waiver** flows; **platform feature flags & gradual rollout**;
**admin audit log**; **sandbox/staging & governed change promotion**; and the
resolution of the **policy-engine unification** question.

**Explicitly deferred (cross-referenced, not duplicated):**
- `GovernancePolicy`/`Decision`/`ApprovalGate`/`ComplianceControl`/`AdminAuditEvent`
  node TYPES, bitemporal time-travel substrate, mutation event stream → **Vol 01**
- The symbolic Layer-1 engine that *evaluates* policies, AI confidence/evidence/bias
  signals, AI kill-switch *interface* → **Vol 03** (this volume *governs* via those
  hooks; it does not re-implement the engine)
- ABAC `AccessControlRule` evaluation, visual masking, purpose-based access
  *mechanics*, PII-access `AccessLog` *creation* → **Vol 04** (this volume governs the
  *who-administers / SoD / audit-explorer* layer on top)
- Marketplace transactions, pricing, revenue-share *payouts*, creator economy →
  **Vol 13** (this volume defines only **governance** of the ecosystem: vetting,
  licensing enforcement, monetization audit)
- License *purchasing/metering* UX → **Vol 09/13** (governance of license policy here)

---

## 2. Core Principle — Governance as a First-Class Dimension

> "Governance is NOT a separate module — it is a first-class dimension baked into
> every node, edge, widget, and AI interaction." It is the guardian of the single
> source of truth.

- **EKOS-05-001 (Must, MVP)** — Governance SHALL be embedded in the platform fabric
  (graph + AI + widgets), not bolted on as a separate app, so compliance becomes a
  **byproduct of normal work**. *Source:* ConfusionClarity(governance intro);
  blueprint §7.

- **EKOS-05-002 (Must, MVP)** — The framework SHALL answer the four universal
  enterprise questions: **Who can do what?** (access/authz), **Is it correct &
  compliant?** (quality/standards), **How did we get here & why?** (auditability/
  decision traceability), **Are we still aligned with the business?** (strategic
  alignment/risk). *Source:* ConfusionClarity(governance — four questions).

- **EKOS-05-003 (Must, MVP)** — Governance SHALL remain valid even as thousands of
  people and hundreds of AI agents modify the graph continuously. *Source:*
  ConfusionClarity(governance — "even as thousands ... modify it continuously").

---

## 3. Pillar 1 — Policy-as-Code (PaC) on the Knowledge Graph

- **EKOS-05-004 (Must, MVP)** — Governance SHALL be encoded as **machine-enforceable
  `GovernancePolicy` graph nodes**, not Word documents — each linking to the artifacts
  it governs. *Source:* ConfusionClarity(Pillar 1 — Policy Nodes); blueprint §7;
  research(final spec §7).

- **EKOS-05-005 (Must, MVP)** — The **policy engine** SHALL be the same symbolic
  reasoning layer (Vol 03 Layer 1) continuously evaluating policies against the graph;
  violations SHALL surface instantly as **red badges, blocked actions, or auto-
  generated remediation tasks**. *Source:* ConfusionClarity(Pillar 1 — Policy Engine);
  Vol 03 EKOS-03-013. *Depends:* Vol 03.

- **EKOS-05-006 (Must, MVP)** — Policies SHALL support **inheritance** — cascading down
  the graph hierarchy (a security policy on a `BoundedContext` automatically covers its
  `Service`s). *Source:* ConfusionClarity(Pillar 1 — Policy Inheritance); research(final
  spec §7).

- **EKOS-05-007 (Must, MVP)** — The engine SHALL support **block-on-violation** policies
  evaluated at mutation time. Canonical example: *"a `PersonalDataCategory:PII-Sensitive`
  node must not link to a `StorageService` unless an `EncryptionAtRest` node is also
  linked"* — the instant someone attempts the link, the system blocks it and **names
  the violated policy**. *Source:* ConfusionClarity(Pillar 1 — example); blueprint §7.

- **EKOS-05-008 (Should, P2)** — Policy violations SHALL be recorded as graph
  nodes/insights (Vol 01) so they are themselves traceable, time-stamped, and
  auditable. *Source:* derived from ConfusionClarity(Pillar 1) + Vol 01.

---

## 4. Pillar 2 — Federated Access Governance

> Builds on Vol 04 ABAC, adding the *purpose dimension*, *separation of duties*, and
> *delegated administration*.

- **EKOS-05-009 (Must, P2)** — **Purpose-based access**: governance SHALL validate that
  a user's selected `ProcessingPurpose` is authorized for that role and data category
  (the selection/denial *mechanics* are Vol 04; the *authorization policy* is here).
  *Source:* ConfusionClarity(Pillar 2 — purpose-based access); Vol 04 EKOS-04-032/033.

- **EKOS-05-010 (Must, P2)** — **Segregation of Duties (SoD)**: policies SHALL be able
  to enforce that the person who approves a requirement **cannot also approve the
  architecture decision it spawns**; the graph tracks all actions and prevents SoD
  conflicts. *Source:* ConfusionClarity(Pillar 2 — SoD); research(final spec §7).

- **EKOS-05-011 (Should, P2)** — **Delegated administration**: each `BoundedContext` or
  `Project` SHALL be able to have its own governance admin who defines context-specific
  policies **without affecting the entire enterprise**, scoped via the same graph ACLs.
  *Source:* ConfusionClarity(Pillar 2 — delegated administration).

---

## 5. Pillar 3 — Automated Quality & Compliance Gates

- **EKOS-05-012 (Must, P2)** — A **Knowledge Quality Engine (KQE)** SHALL assign every
  artifact (requirement, story, ADR, test case) a **dynamic quality score** based on
  completeness, recency, traceability, and review status. *Source:* ConfusionClarity
  (Pillar 3 — KQE); research(final spec §7).

- **EKOS-05-013 (Should, P2)** — The governance dashboard SHALL show **quality
  heatmaps**, and the AI SHALL **auto-flag "rotten" / stale artifacts**. *Source:*
  ConfusionClarity(Pillar 3 — KQE). *Depends:* Vol 03 (AI flagging).

- **EKOS-05-014 (Must, P2)** — **Compliance Gates in Widgets**: a state transition SHALL
  be allowed only if its gate conditions pass — e.g., marking a `SOWDeliverable` "done"
  checks: are all linked tests passing? is client acceptance recorded? has the security
  review completed? *Source:* ConfusionClarity(Pillar 3 — compliance gates);
  research(final spec §7 — "automated gates before state transitions").

- **EKOS-05-015 (Should, P2)** — **Continuous Architecture Fitness**: the platform SHALL
  continuously score architecture health against enterprise standards (TOGAF, well-
  architected frameworks) using a linting ruleset (target **200+ rules**), surface a
  **fitness dashboard**, and **raise drift alerts** when the architecture diverges from
  intended state. *Source:* ConfusionClarity(Pillar 3 — Continuous Architecture
  Fitness); research(final spec §7 — "200+ linting rules ... drift detection").

- **EKOS-05-016 (Could, P3)** — Quality hygiene MAY be **gamified** (knowledge-hygiene
  scoring/badges) to drive adoption. *Source:* research(final spec §10 — Knowledge
  Quality Engine / gamification).

---

## 6. Pillar 4 — Immutable Audit & "Why" Trail (Decision DNA)

- **EKOS-05-017 (Must, P2)** — **Decision DNA**: every `ArchitectureDecision`,
  `ChangeRequest`, and `RiskAcceptance` node SHALL **require capture of**: the business
  rationale, alternatives considered, assumptions, outcomes, lessons learned, and the
  approvers' **digital signatures** (Vol 04 §14). *Source:* ConfusionClarity(Pillar 4 —
  Decision DNA); research(final spec §7).

- **EKOS-05-018 (Must, P2)** — The graph's **bitemporal versioning** (Vol 01 §7) SHALL
  enable **replay of the entire system state at any past point**, combined with purpose
  logs and access logs ("audit nirvana"). *Source:* ConfusionClarity(Pillar 4 —
  Immutable Ledger); Vol 01 EKOS-01-038.

- **EKOS-05-019 (Must, P2)** — A **Regulatory Evidence Widget** SHALL generate a
  pre-compiled evidence pack for an ISO 27001 / GDPR audit, **directly citing the graph
  nodes** that demonstrate compliance — no manual scrambling. *Source:* ConfusionClarity
  (Pillar 4 — Regulatory Evidence Widget); Vol 04 EKOS-04-059.

- **EKOS-05-020 (Must, P2)** — The audit ledger SHALL be **immutable/tamper-evident**
  (append-only); audit records cannot be modified or deleted. *Source:* ConfusionClarity
  (Pillar 4); Vol 04 EKOS-04-036.

---

## 7. Pillar 5 — AI Governance (The AI Constitution)

> The *program* that governs the Vol 03 engine via its hooks. "If AI is constantly
> generating advice and even decisions, it must be governed."

- **EKOS-05-021 (Must, P2)** — **AI Analyst Certification**: every AI Analyst (built-in,
  custom, or marketplace) SHALL carry an attached `GovernancePolicy`. Example policy:
  *"any cost-optimization suggestion exceeding $5,000/month must be reviewed by a human
  before action."* *Source:* ConfusionClarity(Pillar 5 — AI Analyst Certifications);
  Vol 03 EKOS-03-066.

- **EKOS-05-022 (Must, P2)** — **Confidence & Explainability gate**: governance SHALL
  consume the AI's published evidence chains & confidence scores (Vol 03 §14) and
  **suppress or flag-for-review insights below a configurable confidence threshold**.
  *Source:* ConfusionClarity(Pillar 5 — confidence & explainability); Vol 03 EKOS-03-063.

- **EKOS-05-023 (Must, P2)** — **Human-in-the-Loop Checkpoints**: critical AI-suggested
  actions (e.g., "provision a new AWS account", "change this SLO") SHALL always route
  through an `ApprovalGate` linked to a policy defining the required approvers. *Source:*
  ConfusionClarity(Pillar 5 — human-in-the-loop); Vol 03 EKOS-03-067.

- **EKOS-05-024 (Must, P2)** — **AI-output review gate**: when AI auto-applies or
  extracts content (requirements, ghost nodes, classifications), a **review
  WorkflowRequest SHALL be raised so AI output is human-gated before it is treated as
  confirmed** (AI auto-apply resets the approval gate). *Source:* [[project-esip]] C4;
  Vol 03 EKOS-03-056; cross-role workflow ([[project-esip]] Epic D). *Depends:* Vol 03,
  Vol 08 (workflow).

- **EKOS-05-025 (Must, P2)** — **Bias & Drift Monitoring**: governance SHALL monitor the
  output distribution of AI recommendations (e.g., is one team getting all the
  "refactor" suggestions?); if an analyst's performance metrics drop, a governance rule
  MAY **automatically disable** it via the kill-switch interface (Vol 03 §15). *Source:*
  ConfusionClarity(Pillar 5 — bias & drift); Vol 03 EKOS-03-068.

- **EKOS-05-026 (Should, P2)** — **AI Model Governance (MLOps for analysts)**: custom
  AI Analysts (incl. fine-tuned-on-customer-data ones) SHALL have **model version
  control, a versioned lineage in a Model Registry, bias monitoring, and a kill
  switch**. *Source:* ConfusionClarity(future domain E — Model Registry); Vol 03
  EKOS-03-069.

- **EKOS-05-027 (Must, MVP)** — AI governance SHALL enforce the Constitution: **no
  training on customer content**; opted-in learning limited to anonymized behavioral
  signals. *Source:* Constitution §6; Vol 03 EKOS-03-061.

---

## 8. Pillar 6 — Marketplace & Ecosystem Governance

> Governance of the third-party widget/analyst ecosystem. (Economics → Vol 13.)

- **EKOS-05-028 (Should, P2)** — **Certified Widget Vetting**: the marketplace SHALL
  combine **automated code scanning** (detect data exfiltration / policy violations)
  with a **community trust score**; enterprises SHALL be able to restrict their private
  marketplace to only **"Platform Certified"** or **"Verified Creator"** widgets/
  analysts. *Source:* ConfusionClarity(Pillar 6 — Certified Widget Vetting);
  research(monetization — security review). *Depends:* Vol 02 EKOS-02-054, Vol 13.

- **EKOS-05-029 (Should, P2)** — **IP & Licensing Enforcement**: widgets/analysts carry
  license metadata (Vol 01 EKOS-01-083); the governance engine SHALL **prevent a widget
  licensed "Internal Use Only" from being shared externally**. *Source:* ConfusionClarity
  (Pillar 6 — IP & Licensing Enforcement).

- **EKOS-05-030 (Should, P2)** — **Monetization Audit Trail**: all revenue-share
  transactions SHALL be recorded in an **immutable ledger** for transparency to
  creators and operator alike (the payout *mechanics* are Vol 13). *Source:*
  ConfusionClarity(Pillar 6 — Monetization Audit Trail).

- **EKOS-05-031 (Could, P3)** — A **Verified Creator Program** SHALL let accredited
  integrators (e.g., large consultancies) sell "Proven" widgets/analysts with SLAs and
  support contracts. *Source:* research(monetization — Verified Creator Program);
  Vol 13.

---

## 9. The Governance Workbench Widget

> A meta-widget for governing the platform itself.

- **EKOS-05-032 (Must, P2)** — **Policy Canvas**: a drag-and-drop policy constructor
  (e.g., *"IF resource type = CloudService AND NOT linked to CostCenter THEN block
  deployment"*) producing `GovernancePolicy` nodes. *Source:* ConfusionClarity
  (Governance Workbench — Policy Canvas); blueprint §7.

- **EKOS-05-033 (Must, P2)** — **Violation Dashboard**: a living heatmap of all active
  governance violations, their severity, and who is responsible. *Source:*
  ConfusionClarity(Workbench — Violation Dashboard).

- **EKOS-05-034 (Must, P2)** — **Audit Explorer**: time-travel through the graph's
  history, filter by policy or user, and generate **tamper-proof reports**. *Source:*
  ConfusionClarity(Workbench — Audit Explorer); Vol 01 §7.

- **EKOS-05-035 (Must, P2)** — **Exception Management**: a governed flow where a user
  requests a temporary policy waiver, which itself must be approved by a designated
  authority, with **automatic expiry**. *Source:* ConfusionClarity(Workbench —
  Exception Management).

---

## 10. Exception / Waiver Management (Detail)

- **EKOS-05-036 (Should, P2)** — A waiver request SHALL be a graph object capturing:
  the policy waived, justification, requester, approver, scope, and **expiry**; on
  expiry the policy re-applies automatically. *Source:* ConfusionClarity(Exception
  Management).

- **EKOS-05-037 (Should, P2)** — Active waivers SHALL be visible on the Violation
  Dashboard so suppressed violations are never silently hidden. *Source:* derived from
  ConfusionClarity(Workbench) + no-silent-caps principle.

---

## 11. Platform Feature Flags & Gradual Rollout

> ConfusionClarity #10. Ties directly to ESIP's existing `featureFlags.ts`.

- **EKOS-05-038 (Should, P2)** — EKOS (the vendor) SHALL control feature delivery via a
  **Feature Flag console**, enabling gradual rollout of platform updates rather than
  pushing changes to all customers at once. *Source:* ConfusionClarity(#10 — Platform
  Feature Flags).

- **EKOS-05-039 (Should, P2)** — **Tenant admins** SHALL be able to **preview beta
  features in sandbox** and **roll them out gradually** to their users. *Source:*
  ConfusionClarity(#10).

- **EKOS-05-040 (Must, MVP)** — The existing ESIP feature-flag system (`lib/featureFlags.ts`
  + `store/featureFlagStore.ts` + Admin → Capabilities board, flags default-on with
  per-tenant override, including the new `sprintBoard` default-off flag) SHALL be
  recognized as the **early implementation** of this pillar and evolved into the
  tenant/vendor two-tier model. *Source:* [[project-esip]] Epic H; [[esip-resume-state]].

---

## 12. Admin Audit Log (Platform Administration)

> ConfusionClarity #11 — distinct from PII-access audit (Vol 04); this is admin actions.

- **EKOS-05-041 (Must, P2)** — Every administrative action (changing a global policy,
  adding an integration, modifying a feature flag, granting a role) SHALL be recorded
  as an immutable **`AdminAuditEvent`** node, queryable via the Governance Workbench,
  for SOC2 compliance of the platform itself. *Source:* ConfusionClarity(#11 — Admin
  Audit Log). *Depends:* Vol 01 (node type), Vol 04 (immutability pattern).

- **EKOS-05-042 (Must, P2)** — `AdminAuditEvent` records SHALL be **immutable and
  non-deletable**. *Source:* ConfusionClarity(#11).

---

## 13. Sandbox, Staging & Governed Change Promotion

> ConfusionClarity #4 — admins will never test changes in production.

- **EKOS-05-043 (Should, P2)** — The platform SHALL provide **Workspace Cloning** — a
  full copy of the tenant's graph (minus PII) where new workflows, custom widgets,
  policy changes, and ontology extensions can be tested safely. *Source:*
  ConfusionClarity(#4 — Sandbox & Staging).

- **EKOS-05-044 (Should, P2)** — Changes validated in sandbox SHALL be **promoted to
  production via a governed pipeline** (review + approval), not a raw copy-over.
  *Source:* ConfusionClarity(#4 — promotion pipeline).

- **EKOS-05-045 (Should, P2)** — Ontology-extension governance: a ruling SHALL exist for
  **who may add global (tenant-wide) types vs. project-local types** (resolving Vol 01
  OQ-01-D); global type additions route through the promotion pipeline. *Source:*
  synthesis of Vol 01 OQ-01-D + ConfusionClarity(#4). *Depends:* Vol 01 §9.

---

## 14. Policy-Engine Unification (Resolving Vol 04 OQ-04-A)

- **EKOS-05-046 (Must, MVP)** — There SHALL be **one shared symbolic evaluation engine**
  (Vol 03 Layer 1) executing three *distinct, non-conflated* policy kinds, each its own
  ontology type: **`AccessControlRule`** (Vol 04 — who may see/modify), **`GovernancePolicy`**
  (this volume — what must be true / compliance/quality/SoD), and **Layer-1 validation
  rules** (Vol 03 — graph integrity/smells). One engine, three concerns. *Source:*
  resolution of Vol 04 OQ-04-A + Vol 03 OQ-03-B; ConfusionClarity(Pillar 1 — "same
  symbolic reasoning layer"). *Depends:* Vol 03, Vol 04.

- **EKOS-05-047 (Should, P2)** — Policy authoring surfaces SHALL be unified under the
  Governance Workbench / Security Admin Console so admins reason about all three kinds
  coherently, while the engine keeps them logically separate. *Source:* derived.

---

## 15. Governance Coverage of Cross-Cutting Concerns

- **EKOS-05-048 (Should, P2)** — Governance SHALL cover **carbon/green accounting** as a
  policy surface (ESG mandates becoming procurement criteria); the Green-Lens widget is
  Vol 14, but governance policies (e.g., "flag features whose carbon impact exceeds X")
  live here. *Source:* ConfusionClarity(future domain D — Carbon Accounting). *Depends:*
  Vol 14, Vol 17.

- **EKOS-05-049 (Could, P3)** — Governance SHALL support **cross-organization /
  inter-tenant sharing governance** (who may share which subgraph with which partner,
  with what visibility) as policy; the ABAC primitive is Vol 04 §17, the sharing UX is
  Vol 13. *Source:* ConfusionClarity(future domain C). *Depends:* Vol 04, Vol 13.

- **EKOS-05-050 (Should, P2)** — **License Management policy**: governance SHALL enforce
  per-cost-center seat/usage limits and alert on approaching limits; the purchase/
  metering UX is Vol 09/13. *Source:* ConfusionClarity(#14 — License Management & Cost
  Controls). *Depends:* Vol 13.

---

## 16. Constitution Conformance (governance-specific)

- **EKOS-05-051 (Must, MVP)** — Governance artifacts (policies, decisions, waivers,
  audit events) are **graph nodes** — single source of truth, versioned, traceable
  (Constitution §4). *Source:* Constitution §4.

- **EKOS-05-052 (Must, MVP)** — **No silent caps:** when a policy/gate/waiver suppresses
  a violation or bounds coverage, it SHALL be visibly logged (Violation Dashboard /
  active-waivers), never silently hidden. *Source:* derived; transparency principle.

- **EKOS-05-053 (Must, MVP)** — Policy/quality/AI-governance evaluation runs on the
  **shared pluggable engine** (Constitution §6); engine vendor is configuration.
  *Source:* Constitution §6; EKOS-05-046.

---

## 17. Open Questions (parked)

- **OQ-05-A** — Default policy library shipped out-of-the-box (which TOGAF/well-arch/
  SOC2 rules are seeded as the 200+ baseline) — content effort; align with Vol 14
  compliance widgets.
- **OQ-05-B** — Legal-hold vs right-to-erasure conflict ruling (inherited from
  OQ-04-D) — needs legal + governance decision; affects Pillar 4 + Vol 04 §16.
- **OQ-05-C** — How heavy the KQE scoring is at scale (per-artifact continuous scoring)
  — caching/sampling strategy with Vol 01 §15.
- **OQ-05-D** — Exact promotion-pipeline stages for sandbox→prod (approvals, diffing,
  rollback) — design with Vol 08 workflow engine.
- **OQ-05-E** — Boundary confirmation: AI-output review gate (EKOS-05-024) vs Vol 08
  workflow engine ownership of the WorkflowRequest object — confirm when Vol 08 lands.

---

## 18. Requirement Index (this volume)

EKOS-05-001 … EKOS-05-053 (stable numbering; six pillars + Workbench + feature-flags +
admin-audit + sandbox + engine-unification expand to 84 distinct stated rules).

> **Next volume:** `06 — .ekr Knowledge Runtime Package & Format`.
> Per protocol: STOP here. Await user "proceed".
