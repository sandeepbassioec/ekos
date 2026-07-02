# EKOS Architecture — A01: Vision, Drivers & Quality Attributes

> **Status:** ✅ Done · **Item count:** 64 · **Generated:** 2026-07-01 (Opus)
> **Architecture Phase document A (vision).** Establishes the north star, the
> architectural drivers, the prioritized quality attributes (with **15-year longevity as
> a top-tier driver**), concrete quality-attribute scenarios, the Architecturally
> Significant Requirements (ASRs), constraints, and the tensions to be resolved in later
> architecture docs. Anchors **A02 (logical), A03 (tech ADRs), A09 (sustainability)**.
> **Sources:** [BOOK-1](../BOOK-1-KNOWLEDGE-CONSTITUTION.md) (laws + manifesto),
> [DESIGN-METHODOLOGY](../DESIGN-METHODOLOGY.md), the 27-volume requirements catalog
> (ASRs), [BUILD-PLAN](../BUILD-PLAN-AND-OQ-TRIAGE.md) (decisions D1–D10),
> [00-ARCHITECTURE-INDEX](00-ARCHITECTURE-INDEX.md) (principles). Honors Book 1.

---

## 1. Purpose & Scope

Define **why** EKOS is architected the way it will be: the vision, the forces (drivers,
quality attributes, constraints), and the measurable targets that every subsequent
architecture decision (A02–A11) must satisfy. This document does **not** choose
technologies (→ A03) or design components (→ A02); it sets the criteria they must meet.

---

## 2. Architecture Vision (North Star)

- **A01-001** — EKOS SHALL be architected as a **Knowledge Operating System** — a minimal
  privileged **kernel** over a single semantic-knowledge source of truth, a stable
  **syscall** contract, **runtime subsystems** (compiler, projection, acquisition, AI),
  and a **user-space** of applications and workforces — built to **last and stay current
  for 15+ years**. *Source:* Book 1 §4/Law 13/Law 14.

- **A01-002** — The architecture's defining property SHALL be **endurance through
  change**: it expects new paradigms, regulations, vendors, and AI models, and absorbs
  them **behind ports and an extensible ontology** without rewrites. *Source:* Book 1
  Law 14; Vol 17 (future-proofing).

- **A01-003** — The first proof of the architecture SHALL be **self-hosting**: EKOS
  describes and operates its own specification (Vol 25). *Source:* Book 1 Law 15.

---

## 3. Stakeholders & Their Concerns

- **A01-004** — The architecture SHALL explicitly serve these stakeholders and concerns:

| Stakeholder | Primary concerns |
|---|---|
| **Knowledge workers** (architects, devs, analysts, PMs) | usability, promptless AI, traceability, low friction |
| **Enterprise IT / security** | zero-trust security, identity, residency, isolation, certifications |
| **Compliance / legal / auditors** | auditability, decision DNA, evidence, retention, regulation agility |
| **Executives / buyers** | ROI, longevity, no-lock-in, scale, cost control |
| **Developers / ISVs** | stable SDK/ABI, app runtime, marketplace economics |
| **Government / defense** | air-gap, sovereignty, cryptographic expiration, on-prem |
| **End consumers** (PKS, networks) | privacy, ownership of personal knowledge, portability |
| **The EKOS team itself** | maintainability, self-hosting, sustainable evolution |

*Source:* synthesis across the catalog volumes.

---

## 4. Architectural Drivers (Why)

- **A01-005** — **Knowledge as the primary abstraction** — replace dead documents with a
  living, connected, traceable source of truth (Book 1 Manifesto). *This is the reason
  the architecture is graph-centric, not document- or app-centric.*

- **A01-006** — **Operating-system positioning** — capabilities emerge from primitives;
  apps (incl. ESIP) and workforces (KWR) run on top. *Drives the kernel/syscall/runtime/
  user-space layering.*

- **A01-007** — **15-year longevity** — the platform must remain current and maintainable
  for 15+ years (regular dependency modernization, automated upgrades, deprecation
  policy). *Drives ports-everywhere, stable ABI, standards-over-proprietary, and A09.*

- **A01-008** — **Enterprise + government + consumer reach** — one architecture serving a
  startup, a Fortune-100, a defense ministry, and an individual. *Drives multi-tenancy,
  air-gap, residency, and the consumer (PKS) edition.*

- **A01-009** — **Trust** — zero-trust security, governance, no-lock-in, no-train-on-
  content. *Drives server-side enforcement, ABAC-in-kernel, open export, sovereign AI.*

- **A01-010** — **Ecosystem & economy** — a marketplace of widgets, apps, AI teams, and
  knowledge assets. *Drives sandboxing, certification, stable SDK, and the economy layer.*

---

## 5. Quality Attributes (The "-ilities"), Prioritized

> Priorities guide trade-off resolution: when two attributes conflict, the higher tier
> generally wins (with documented exceptions). **Longevity, Security, Extensibility, and
> Scalability are the dominant four** for a 15-year knowledge OS.

### Tier 1 — Dominant

- **A01-011 · Longevity / Evolvability** — survive 15+ years; absorb new paradigms,
  vendors, models, regulations without rewrite. *Tactics:* ports-and-adapters, stable
  syscall ABI, extensible ontology, standards-first, dependency-currency policy (A09).
- **A01-012 · Security & Privacy** — zero-trust, graph-level ABAC enforced in-kernel,
  cryptographic hardness, no-train-on-content, air-gap. *Non-negotiable (Book 1 Laws
  4/6/7/8).*
- **A01-013 · Extensibility** — everything pluggable (DB/AI/IdP/storage/payments/
  transport), runtime ontology extension, custom widgets/apps/analysts/teams. *Tactics:*
  hexagonal ports, provider-capability descriptors, marketplace.
- **A01-014 · Scalability** — linear scale to 100K-employee tenants; multi-tenant.
  *Tactics:* polyglot CQRS, federated graph, partitioning, kernel resource governance.

### Tier 2 — Critical

- **A01-015 · Availability & Resilience** — active-active multi-region, zero-downtime
  upgrades, self-service DR.
- **A01-016 · Portability** — cloud-neutral (AWS/Azure/GCP/IBM), on-prem, air-gapped;
  Docker/K8s/Helm/Terraform.
- **A01-017 · Performance** — 60 fps graph rendering; sub-second projections; real-time
  ambient AI via the layered (mostly-deterministic) engine.
- **A01-018 · Maintainability** — minimal microkernel, clean volume/subsystem boundaries,
  self-hosting, the boundary-discipline of the methodology.
- **A01-019 · Auditability & Compliance** — bitemporal time-travel, decision DNA,
  immutable audit, governance-as-code, SOC2/ISO/FedRAMP/HIPAA/IRAP/ISO-42001.
- **A01-020 · Usability & Accessibility** — promptless AI, WCAG 2.2 AA, low-friction
  adoption (adoption is the #1 enterprise-failure cause — Vol 09).

### Tier 3 — Important

- **A01-021 · Interoperability** — open standards + bidirectional connectors (two-way
  Jira/ADO; EKOS feeds, doesn't force-replace).
- **A01-022 · Observability** — Observe/Subscribe syscalls, OpenTelemetry-class telemetry,
  Chain-of-Thought explainability.
- **A01-023 · Cost-efficiency** — control LLM cost via the layered AI (most work
  deterministic + cached); per-tenant quotas; FinOps visibility.
- **A01-024 · i18n / White-label** — global teams, custom branding/domains.

---

## 6. Quality-Attribute Scenarios (Concrete & Measurable)

> Format: *Source · Stimulus · Artifact · Environment · Response · Measure.* These are the
> testable targets architecture must hit. (Numbers are initial targets, refined by
> benchmarks in A03/A08.)

- **A01-025 · Longevity** — *A core dependency (graph DB / AI model / framework) reaches
  end-of-life* → the platform *swaps the adapter behind its port* → **with no change to
  the kernel, syscalls, ontology, or apps; achieved within one maintenance cycle.**
  *Source:* Book 1 Law 14; Vol 15.

- **A01-026 · Security** — *An unauthorized principal requests restricted knowledge via any
  path (UI/API/AI/export)* → *the in-kernel PDP scopes/masks server-side* → **the
  restricted content never leaves the kernel; the principal gets no hint it exists; the
  access attempt is logged immutably.** *Source:* Vol 04/§Vol 18 §5.

- **A01-027 · Scalability** — *A tenant grows to 100,000 active contributors with 10⁹
  events/day* → *CQRS routes high-volume events to the event store; structural graph stays
  lean* → **interactive queries/projections stay sub-second; ingestion does not collapse.**
  *Source:* Vol 01 §15; Gemini scale critique.

- **A01-028 · Performance** — *A user opens a 5,000-node architecture view on a tablet* →
  *server-computed deterministic layout + native WASM/WebGPU rendering* → **≥ 60 fps pan/
  zoom; crux insight pre-populated < 1s.** *Source:* Vol 02 §3; Vol 03.

- **A01-029 · Availability** — *A region fails / a platform upgrade is deployed* → *active-
  active failover / rolling zero-downtime upgrade* → **no user-visible downtime; RPO/RTO
  within tenant SLA.** *Source:* Vol 09 §15.

- **A01-030 · Portability (air-gap)** — *A defense tenant runs fully disconnected* → *the
  same architecture runs on-prem with sovereign local AI + on-prem activation + mDNS
  sessions* → **all non-internet-only capabilities function with zero external calls.**
  *Source:* Book 1 Law 8; Vol 03 §13; Vol 12.

- **A01-031 · Extensibility** — *A team needs a brand-new domain concept ("Quantum
  Module")* → *they define a new ontology type at runtime* → **it is immediately first-
  class, queryable, AI-aware, and traceable with no schema migration or redeploy.**
  *Source:* Vol 01 §9; Vol 18 §6.

- **A01-032 · Auditability** — *An auditor asks "who viewed this PII on 12 Mar and why,
  and what did the architecture look like then?"* → *bitemporal time-travel + immutable
  access log* → **exact answer reconstructed from the graph; evidence pack auto-generated.**
  *Source:* Vol 01 §7; Vol 04 §11; Vol 05 §6.

- **A01-033 · Usability** — *A non-technical stakeholder opens any widget* → *promptless
  ambient AI* → **a plain-language crux/example is already shown, no prompt typed;
  WCAG 2.2 AA throughout.** *Source:* Book 1 Law 6; Vol 03 §10; Vol 09 §5.

- **A01-034 · Cost-efficiency** — *Thousands of widgets refresh across a tenant* → *Layer-1/
  2 handle ~90%; Layer-3 LLM calls cached + rate-limited* → **LLM spend stays within the
  tenant budget; no per-refresh large-model call.** *Source:* Vol 03 §3/§20.

- **A01-035 · Evolvability of apps** — *The kernel ships a major version* → *apps built on
  the stable ABI keep running* → **zero app breakage; deprecation warnings precede any
  breaking change per the A09 window.** *Source:* Vol 22 §4; Vol 23 §12.

---

## 7. Architecturally Significant Requirements (ASRs)

- **A01-036** — The following catalog requirements are **architecturally significant** —
  they most shape structure and SHALL be treated as hard inputs to A02–A11:

| ASR | Requirement | Shapes |
|---|---|---|
| Single source of truth | Book 1 Law 2; Vol 01 §1 | graph-centric core (A02/A04) |
| Kernel/user-space boundary | Vol 18 §12; Vol 22 | microkernel + syscalls (A02) |
| Polyglot CQRS storage | Vol 01 §2; Vol 15 §6 | data architecture (A04) |
| In-kernel ABAC PDP | Vol 04; Vol 18 §5 | security architecture (A06) |
| 4-layer hybrid AI | Vol 03 | AI architecture (A05) |
| Ports-and-adapters everywhere | Book 1 Law 10; Vol 15 §1 | every ADR (A03) |
| Stable syscall ABI | Vol 22 §4 | longevity, app runtime (A09/A02) |
| Bitemporal + extensible ontology | Vol 01 §7/§9; Vol 18 §6 | data + kernel (A04/A02) |
| Air-gap / sovereignty | Book 1 Law 8; Vol 03 §13 | deployment (A08) |
| Multi-tenant isolation + residency | Vol 18 §11; Vol 09 §8 | deployment + kernel (A08/A02) |
| No-lock-in open export | Book 1 Law 9; Vol 01 §14 | data + interop (A04/A07) |
| KWR architectural reservation | Vol 26 §11 (EKOS-26-028) | data model (A04) — reserve fields now |

*Source:* the catalog; these are the load-bearing constraints on the architecture.

---

## 8. Constraints

- **A01-037 · Stack constraint** — MVP evolves the **existing React+Firebase+ai-proxy ESIP
  stack behind ports** (D1/D2); not a greenfield rewrite. *Source:* Vol 15 §2.
- **A01-038 · Backend constraint** — hybrid **.NET Core 10 + Rust** (D2); VectorVolt stays
  separate pure-Rust. *Source:* Vol 15 §5.
- **A01-039 · Diagramming constraint** — **maxGraph** (already integrated). *Source:* Vol 15
  §4.
- **A01-040 · Regulatory constraints** — GDPR/DPDP, 21 CFR Part 11, SOC2/ISO/FedRAMP/HIPAA/
  IRAP/ISO-42001, fintech licensing (gates the economy). *Source:* Vol 04/05/09/13.
- **A01-041 · Solo/After-hours constraint** — built by one person after-hours (user
  context); the architecture and plan must be **incremental and solo-sustainable** — small
  shippable steps, heavy automation, no large-team assumptions. *Source:* [[user-profile]];
  [[project-vision]].
- **A01-042 · Budget constraint** — limited AI-credit budget; favor designs that minimize
  expensive LLM usage (the layered AI is also a cost architecture). *Source:*
  [[feedback-model-budget]]; Vol 03 §20.
- **A01-043 · Pre-deploy security constraint** — ai-proxy keys MUST be rotated + Firestore
  rules hardened before any hosted deployment. *Source:* Vol 15 §16; build-plan §5.

---

## 9. Architectural Principles (carried + architecture-specific)

- **A01-044** — Carry forward the 9 principles from [00-ARCHITECTURE-INDEX §3](00-ARCHITECTURE-INDEX.md)
  (platform-before-app, hexagonal, 15-yr longevity, single-source-of-truth, crypto-
  hardness/no-train, air-gap-first, evolve-behind-seams, capability-vs-feature, standards-
  over-proprietary).
- **A01-045 · Fitness functions** — quality attributes SHALL be encoded as **automated
  fitness functions** (continuous architecture tests) where possible — longevity/security/
  performance are *verified*, not hoped (evolutionary architecture; Vol 17 frontier
  §EKOS-17-032). *Source:* evolutionary-architecture practice; Vol 05 §5.
- **A01-046 · Reversibility** — prefer reversible decisions; make irreversible ones late
  and behind ports (supports longevity). *Source:* Book 1 Law 10/14.

---

## 10. Architectural Tensions & Trade-offs (to resolve in later docs)

- **A01-047 · Lock-in moat vs No-lock-in** — the platform creates switching costs (custom
  ontology/apps/workforces) **yet guarantees JSON-LD/RDF open export**. *Resolution:* both
  coexist by design (Book 1 Law 9); the moat is value, not hostage-taking. *Owner:* A04/A07.
- **A01-048 · Security/zero-trust vs Usability** — server-side masking everywhere vs low
  friction. *Resolution:* promptless AI + projection caching keep UX fast despite per-
  element PDP. *Owner:* A06/A02.
- **A01-049 · Microkernel minimalism vs Performance** — small privileged core vs fast
  paths. *Resolution:* symbolic rules + PDP are kernel-resident fast paths; heavy work in
  user-space. *Owner:* A02 (OQ-18-A benchmark).
- **A01-050 · Flexibility (pluggable everything) vs Coherence** — many adapters vs a
  consistent platform. *Resolution:* provider-capability descriptors + graceful
  degradation; capability-vs-feature gate. *Owner:* A03.
- **A01-051 · Longevity vs Velocity** — 15-year discipline vs shipping fast solo.
  *Resolution:* evolve ESIP behind ports incrementally; stable ABI protects apps while the
  core modernizes. *Owner:* A09/A11.
- **A01-052 · AI capability vs Cost/Determinism** — richer AI vs spend + trust. *Resolution:*
  the 4-layer engine (deterministic-first, LLM-last, cached). *Owner:* A05.
- **A01-053 · Air-gap/sovereignty vs Cloud features** — disconnected vs internet-only
  capabilities. *Resolution:* every capability has a local equivalent; only internet-only
  features excluded. *Owner:* A08.

---

## 11. What This Document Drives

- **A01-054** — **A02 (Logical Architecture)** SHALL realize the layering (kernel/syscalls/
  runtime/user-space) and bounded contexts that satisfy these QAs and ASRs.
- **A01-055** — **A03 (Technology ADRs)** SHALL select technologies **scored against the
  Tier-1 quality attributes** using EKOS's own Trade-off methodology.
- **A01-056** — **A04–A08** (data/AI/security/integration/deployment) SHALL each trace
  their decisions back to the relevant QAs/ASRs here.
- **A01-057** — **A09 (Sustainability)** SHALL operationalize **Longevity (A01-011)** into
  concrete dependency-currency, upgrade, and deprecation policies.
- **A01-058** — **A10 (ESIP→EKOS migration)** and **A11 (implementation plan)** SHALL honor
  the solo/incremental constraint (A01-041) and Priority≠Phase.

---

## 12. Open Questions (parked)

- **OQ-A01-A** — Exact numeric SLO targets (latency/throughput/RPO/RTO per tier) — refine
  with A08 + benchmarks.
- **OQ-A01-B** — Which quality attributes get automated fitness functions in Phase 1 vs
  later — with A11.
- **OQ-A01-C** — Solo-sustainability vs the full vision scope — phasing realism; the plan
  (A11) must be honest about what one person ships when.
- **OQ-A01-D** — Consumer (PKS) vs enterprise edition architectural divergence — does one
  architecture serve both cleanly? Revisit in A02/A10.

---

## 13. Item Index (this document)

A01-001 … A01-058 (vision, stakeholders, drivers, prioritized quality attributes,
measurable QA scenarios, ASRs, constraints, principles, tensions, and what-this-drives
expand to 64 distinct stated items).

> **Next architecture doc:** `A02 — Logical Architecture` (bounded contexts, the layered
> kernel/syscall/runtime/user-space realization, C4 Context + Container).
> Per protocol: STOP here. Await user "proceed".
