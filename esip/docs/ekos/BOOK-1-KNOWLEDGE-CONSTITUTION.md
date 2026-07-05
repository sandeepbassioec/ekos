# EKOS Book 1 — The Knowledge Constitution & Manifesto

> **Book 1. The foundation everything else stands on.** Not an appendix, not a
> cross-cutting note — the first and highest document. The 18 requirement volumes, the
> 8 OS-abstraction volumes, the architecture phase, and every future decision derive
> from and must conform to this Book.
> **Created:** 2026-07-01 (Opus). Elevated & expanded from `00-MASTER-INDEX.md §2`,
> with the *why EKOS exists* manifesto that previously lived only in the research
> conversations (flagged by the cross-architect review as the most valuable missing
> piece). Authority: this Book overrides any conflicting statement in any volume; on
> conflict, Book 1 wins.

---

## Part I — The Manifesto (Why EKOS Exists)

### 1. The wrong abstraction

For fifty years, software has treated the **document** as the unit of knowledge. Word,
PDF, Confluence, Visio, slides, wikis, spreadsheets. We built readers, writers, editors,
and viewers for documents. But a document is **dead**: it is a frozen, disconnected,
un-traceable snapshot that rots the moment it is written and that nobody reads end to
end. Traceability collapses the instant the first ticket is created. The meaning lives
in people's heads and in meetings, not in the artifact.

**This is the wrong abstraction.**

### 2. The primary abstraction

The primary abstraction is not the document. It is **knowledge** — living, connected,
versioned, traceable, and intelligent. A requirement, an architecture decision, a line
of code, a cost, a risk, a person's expertise, a compliance control — these are
**knowledge objects** with relationships, history, and meaning. When knowledge is the
unit, a "document" becomes merely **one projection** of it, generated on demand for a
particular audience.

EKOS exists to **make knowledge the primary abstraction** of the software lifecycle and
beyond — to replace dead documents with a single, living, intelligent source of truth.

### 3. Capabilities, not features

Every time a need appears ("we need a resume builder", "we need a video call", "we need
a marketplace"), the wrong move is to build the feature. The right move is to ask **what
platform capability the feature reveals** — and to build that capability as a primitive
from which the feature (and a thousand others) emerges as a configuration or projection.

- "Resume builder" → **Personal Knowledge Space**, of which a résumé is one projection.
- "Video call" → **Knowledge Synchronization Session**, of which video is one stream.
- "Community" → **Knowledge Network**, of which a community is one type.
- "Payments" → **Knowledge Economy**, of which a fee is one transaction.
- "Encrypted export" → **Knowledge Runtime Package**, of which encryption is one property.

This is not a stylistic preference. It is the **mechanism by which EKOS becomes an
operating system rather than an application**, and the reason it can survive twenty
years: primitives endure; features churn.

### 4. Operating system, not application

EKOS is an **Operating System for knowledge**. Like any OS, it has a layered structure
that must be honored:

```
            ┌─────────────────────────────────────────────┐
  User      │  Applications (ESIP, PKS, Networks, Books…)   │  ← what users run
  Space     │  EKOS Application Runtime & SDK · App Store   │
            ├─────────────────────────────────────────────┤
            │  Knowledge Repr. Runtime (KRR) · Compiler ·    │  ← turning knowledge
  Runtime   │  Widget OS · Viewer Family · EKR Packages     │     into experiences
            ├─────────────────────────────────────────────┤
            │  OS APIs / Syscalls (Knowledge.Open/Query/…)  │  ← the stable contract
            ├─────────────────────────────────────────────┤
  Kernel    │  Knowledge Kernel: graph primitives, ontology │  ← the deepest core
            │  scheduling, ambient-AI dispatch, security    │
            └─────────────────────────────────────────────┘
              Acquisition Framework feeds knowledge IN from
              the whole world (meetings, ERP, IoT, voice, AI…)
```

Applications — including the ESIP web app built so far — are **citizens of user space**.
They run *on top of* the kernel, the syscalls, the runtime. **The platform is built
first; the application is migrated onto it.** Building the app and calling it the OS is
the cardinal mistake EKOS must never make.

### 5. The promise

A world where people say *"send me the EKR"* the way they say *"send me the PDF"* — but
the thing they send is alive: it explains itself, updates itself, proves its own
compliance, masks what you may not see, and connects every business goal to the running
byte that fulfills it. A platform that an individual, a startup, a Fortune-100, and a
defense ministry can all build their knowledge upon — and that will still be standing,
maintained on current foundations, fifteen years from now.

---

## Part II — The Laws (Non-Negotiable Axioms)

These are binding on every volume, every architecture decision, and every line of code.
Numbered for citation (e.g., "Law 4").

**Law 1 — Capability over feature.** Build platform capabilities, not bespoke features.
Every addition passes the Design Methodology (separate chapter) and the five lenses
before it is locked. No bespoke widget exists where a projection of a primitive suffices.

**Law 2 — Knowledge is the single source of truth.** All knowledge lives in the semantic
graph (the kernel). Nothing maintains an authoritative parallel datastore. Widgets,
resumes, viewers, reports, books, dashboards — all are **projections** of the graph.

**Law 3 — Everything is a projection.** Presentation is decoupled from knowledge. The
same canonical knowledge yields different projections for different viewers, governed by
policy. New visual paradigms are new projections over existing data — never a migration.

**Law 4 — Cryptographic hardness, not obscurity.** Security rests on mathematics and
keys, never on secrecy of format or algorithm. **Forbidden wording:** "impossible to
reverse engineer", "no one (incl. Microsoft/Google) can read it". **Correct wording:**
*"Without authorized cryptographic keys, the contents remain computationally infeasible
to recover; the security model assumes an attacker may possess the encrypted artifact."*

**Law 5 — Cryptographic Expiration, not self-destruct.** We guarantee destruction of
**keys/licenses/tokens** and **denial of future access** — never physical bit-destruction
on a machine we do not control. Copy-resistance = one-time activation + hardware binding,
with the honest acknowledgment that pure-offline unlimited-copy prevention is impossible.

**Law 6 — Promptless, grounded AI.** The widget/knowledge-object **is** the prompt; the
ontology is the prompt. The user never types a prompt to get value. The LLM **never
originates a fact** that requires accuracy — facts come from deterministic (symbolic +
graph) layers; the LLM only renders them into language. AI operates strictly within the
requester's security projection.

**Law 7 — No training on customer content.** EKOS never trains on customer knowledge.
Learning is limited to **anonymized, opt-in behavioral signals**. Customer knowledge is
the customer's, forever.

**Law 8 — Air-gap is first-class.** Every capability has an air-gapped/on-prem path;
only genuinely internet-only features (live external feeds) are excluded, and each has a
local equivalent (on-prem activation, sovereign models, mDNS sessions).

**Law 9 — No vendor lock-in, by design.** A one-click open export (JSON-LD/RDF + an
open-source offline player) is a permanent customer-trust guarantee that coexists with —
and deliberately counter-tensions — the IP moat. Customers are never held hostage.

**Law 10 — Providers are pluggable; standards are preferred.** Every external dependency
(DB, AI provider, storage, IdP, payments, signaling, KMS, AV) sits behind a port. **All
vendor names anywhere in the specification are illustrative examples, never
requirements.** Where longevity depends on it, open standards (openCypher/GQL, JSON-LD/
RDF, OCI, W3C VC/DID, CloudEvents, OpenTelemetry) are chosen over proprietary formats.

**Law 11 — Bitemporal & non-destructive.** Nothing is truly deleted (save the governed
right-to-erasure exception). Valid-time + transaction-time on every node/edge; full
genealogy; time-travel to any past state. "Discarded" is a state transition, not
destruction.

**Law 12 — Governance is intrinsic, not bolted on.** Policy-as-code, decision DNA,
quality gates, and AI governance are first-class graph citizens. Compliance is a
byproduct of normal work. No silent caps: any suppression/waiver/bound is visibly
logged.

**Law 13 — Platform before application; kernel before user-space.** The OS layers
(kernel → syscalls → runtime → user-space) are honored. The platform foundation is built
deliberately; applications (including ESIP) are migrated onto it. We never bolt the OS
onto the app.

**Law 14 — Designed for fifteen years.** Longevity is a top-tier quality attribute.
Dependencies are kept current by policy; upgrades are continuous and partly automated;
deprecation is planned; a tech-radar cadence governs modernization. The platform is
architected to *expect* change and *thrive* on it, not to be frozen.

**Law 15 — Self-hosting reference.** EKOS is described in EKOS. The specification itself
is the first reference knowledge base — EKOS imports EKOS. The methodology that produced
EKOS is itself a preserved, first-class artifact.

**Law 16 — Representation Independence.** Enterprise knowledge SHALL never be persisted
in its representational form unless explicitly required by legal, regulatory, contractual,
archival, or interoperability obligations. Representations — documents, diagrams, books,
dashboards, presentations, architecture views, compliance reports — are **ephemeral by
default**: generated on demand by the Knowledge Representation Runtime (Vol 20) from
canonical knowledge objects. The only thing that truly persists is the knowledge model
plus the projection/representation rules. The sole exception is the **immutable evidence
artifact**: a representation that has crossed a legal or regulatory boundary (a filed PDF,
a signed contract, a submitted government form, a sent invoice, a patent submission, an
audit trail). Evidence artifacts are stored once, immutably, with provenance, version,
reason, and the creating KO's identity — never edited, only superseded. This law is the
mechanism that eliminates storage bloat, representation drift, and the "which version is
the truth?" problem at its root.

---

## Part III — Constitutional Compliance Protocol (How Laws Are Enforced at Runtime)

> **Added 2026-07-03.** The Constitution is not a document to be read — it is a runtime
> contract to be enforced. Every caller (UI, SDK, EKR Viewer, KWR Worker, third-party
> extension, autonomous AI agent) reaches EKOS through the syscall layer (Vol 22), and
> every syscall passes through the **Constitutional Compliance Runtime (CCR)** — a
> cross-cutting enforcement layer that makes the Laws binding at execution time, not just
> at design time. **ADR-051** governs this decision.

### 1. The Enforcement Principle

Callers do not "call the Constitution." Callers call syscalls. The CCR **intercepts every
syscall** and enforces the applicable Laws on behalf of all callers — without opt-in,
without escape hatch, without trusted-caller bypass. The Constitution is enforced by the
runtime, not trusted to callers.

This mirrors how operating systems work: no user-space process can bypass the kernel's
memory protection, scheduling, or capability model regardless of how trusted it is.
EKOS's Constitution is the kernel's protection model for knowledge.

### 2. Law Enforcement Tiers

Not all Laws are enforced the same way. Laws are tiered by where and how they are checked:

| Tier | Laws | Enforcement mechanism | When checked |
|---|---|---|---|
| **Tier 1 — Structural invariants** | Law 2, 3, 13 | Architecture enforces them. The ports literally do not expose a path that bypasses the graph or the projection layer. No runtime check needed — the wrong path doesn't exist. | At design time; port boundaries |
| **Tier 2 — Behavioral laws** | Law 4, 6, 11, 16 | CCR checks on every qualifying syscall. `Commit()` checks Law 11 (bitemporal). `Project()` checks Law 3/16 (projection, no representation persistence). `Query()` checks Law 6 (AI-scoped). | At every syscall invocation |
| **Tier 3 — Policy laws** | Law 5, 7, 8, 9, 10, 12, 14, 15 | OPA/Rego policy engine (CCR delegates to the policy evaluation plane). Policies are themselves knowledge objects (Law 12). | At qualifying syscall; async for audit |
| **Tier 4 — Capability laws** | Law 1 | Design Methodology gate; ADR; architecture review. Not a runtime check — a decision-time gate. | At decision / ADR time |

### 3. The ConstitutionalViolation Error Hierarchy

Every CCR violation produces a typed error. The hierarchy is stable — callers may catch
specific law violations for structured handling:

```
ConstitutionalViolation (base)
├── Law2Violation       — caller attempted to write knowledge outside the graph
│                          (e.g., persisting to a shadow datastore via a bypassed port)
├── Law3Violation       — caller attempted to persist a representation as canonical knowledge
│                          (Law 16 subtype — representation persistence without evidence exception)
│   └── RepresentationPersistenceViolation (Law 16)
├── Law4Violation       — caller provided credentials / configuration that weakens
│                          the cryptographic model (e.g., null-key, test bypass)
├── Law6Violation       — AI syscall attempted to originate a fact without graph-sourced grounding;
│                          or AI operation executed outside the calling principal's PDP scope
├── Law7Violation       — caller attempted to trigger a training pipeline on customer content
├── Law11Violation      — caller attempted a destructive (non-bitemporal) write;
│                          valid-time or transaction-time missing from Commit()
├── Law13Violation      — app-layer code attempted to call a kernel primitive directly,
│                          bypassing the syscall layer
└── PolicyViolation     — OPA/Rego evaluation returned deny
    ├── GovernanceViolation    (Law 12)
    ├── VendorLockInViolation  (Law 9 — export path blocked)
    └── AirGapViolation        (Law 8 — feature deployed with no air-gap equivalent)
```

**Rules for all callers:**
- All `ConstitutionalViolation` subtypes are **never suppressible** by user-space callers.
- A violation **never leaks unauthorized information** about the rejected knowledge object
  (consistent with Vol 04 §7 and EKOS-22-028).
- Every violation **emits a constitutional telemetry event** (§4 below) before returning
  the error to the caller.

### 4. Constitutional Telemetry — The Unified Observability Pipeline

Every CCR evaluation — pass or violation — emits a structured event to the **Constitutional
Telemetry Stream** (CTS). This is the single, unified observability plane for all
constitutional enforcement activity across all callers and all deployment tiers (SaaS,
air-gap, EKR Viewer, SDK, KWR).

**ConstitutionalEvent schema (illustrative):**

```json
{
  "eventId": "<uuid>",
  "timestamp": "<ISO8601>",
  "lawsEvaluated": ["Law11", "Law3"],
  "outcome": "PASS | VIOLATION | POLICY_DENY",
  "violatedLaw": "Law11",
  "violationType": "Law11Violation",
  "syscall": "Knowledge.Commit",
  "callerType": "SDK | UI | EKR_Viewer | KWR_Worker | Extension | AI_Agent",
  "callerId": "<principal-id>",
  "tenantId": "<tenant-id>",
  "knowledgeObjectRef": "<KO-id>",
  "policyRef": "<OPA-package-path>",
  "correlationId": "<trace-id>",
  "deploymentTier": "SaaS | AirGap | EKR | Edge"
}
```

**CTS properties:**
- **Immutable** — events are appended, never modified (Law 11 applies to itself).
- **Caller-agnostic** — one stream regardless of whether the caller is a browser, SDK,
  EKR Viewer, autonomous AI, or third-party extension.
- **Platform-internal** — not filtered by tenant PDP scope (compliance officers can see
  all constitutional events for their tenant; the platform team sees across tenants).
- **Standard transport** — emitted as CloudEvents over the kernel's Kafka journal (Vol 18
  §8); consumed by the Governance Service (Vol 05), SIEM integrations (Vol 04 §19),
  and the Constitutional Compliance Dashboard.

**Mandatory CTS metrics (per tenant, per deployment tier):**

| Metric | Description |
|---|---|
| `ekos.ccr.evaluations.total` | Total CCR evaluations (all syscalls, all callers) |
| `ekos.ccr.evaluations.by_law` | Evaluation count broken down by law |
| `ekos.ccr.violations.total` | Total constitutional violations |
| `ekos.ccr.violations.by_law` | Violations broken down by violated law |
| `ekos.ccr.violations.by_caller_type` | Violations broken down by caller type (SDK/UI/Extension/AI) |
| `ekos.ccr.policy_denies.total` | OPA policy denials |
| `ekos.ccr.latency_p99_ms` | CCR enforcement latency at P99 (target: < 1ms in-process) |

These metrics are first-class platform health indicators — visible on the Constitutional
Compliance Dashboard alongside SLOs, and exported via `Observe()` to any monitoring tool
(Law 10 — pluggable).

### 5. Extension Compliance Manifest

Any party extending EKOS (third-party widget, connector, app, SDK client, AI agent
integration) MUST declare a **Constitutional Compliance Manifest (CCM)** — a signed
knowledge object submitted at extension registration time. This is the mechanism by which
the Constitution extends to every extension.

**CCM minimum required fields:**

```yaml
ekos_ccm_version: "1.0"
extension_id: "<globally-unique-id>"
extension_type: "App | Widget | Connector | AI_Agent | SDK_Client"
author_tenant: "<tenant-id>"
constitution_version: "Book1-v1"   # pins to a Constitution version

# Which laws the extension explicitly engages with (not opts out of —
# all laws are binding; this declares which ones the extension
# consciously interacts with and how)
law_engagements:
  - law: "Law16"
    engagement: "Produces projections on demand; no representation persistence"
    evidence_artifact_types: ["signed-contract", "filed-pdf"]  # if any
  - law: "Law6"
    engagement: "AI features are grounded via Knowledge.Query; no LLM-originated facts"
  - law: "Law8"
    engagement: "All AI model calls route through AI Engine port; local model available"

# Syscalls this extension uses (CCR uses this for pre-registration validation)
syscalls_used: ["Knowledge.Open", "Knowledge.Query", "Knowledge.Project"]

# Extension's own error mapping (how it surfaces ConstitutionalViolations to users)
violation_surface: "SDK-exception | UI-error-boundary | gRPC-status"

# Signed by the extension author's tenant key (KMS)
signature: "<Ed25519-signature-over-manifest-hash>"
```

**CCM enforcement rules:**
- An extension that uses a syscall not declared in `syscalls_used` receives a
  `ConstitutionalViolation` (Law 13 — unexpected kernel access).
- A CCM signed with an invalid or revoked key is rejected at registration time.
- CCM versions are pinned — when the Constitution is amended, extensions must declare
  re-compliance before their manifests are accepted against the new version.
- Platform operators may audit all CCMs via `Knowledge.Query` (they are knowledge objects
  in the graph — Law 15 / self-hosting).

### 6. Caller-Specific Enforcement Notes

| Caller type | CCR entry point | Notes |
|---|---|---|
| **Browser UI** (ESIP, EKOS apps) | App SDK → syscall layer | SDK wraps CCR; UI never touches kernel directly. `ConstitutionalViolation` surfaces as a typed error boundary. |
| **SDK clients** (.NET, Rust, TS, Python) | Syscall layer directly | SDK is a thin ergonomic wrapper; every call enters CCR. Language-idiomatic exception type wraps `ConstitutionalViolation`. |
| **EKR Viewer** (free viewer, offline) | Syscall layer (local kernel instance) | Viewer ships with a bundled CCR; air-gap enforcement identical to cloud. Law 4, Law 11, Law 16 enforced locally. |
| **KWR Worker** (human/AI/automation) | Worker SDK → syscall layer | Worker SDK routes every `IKnowledgeAccess` call through the syscall layer. Capability-scoped sandbox + CCR enforce together. |
| **Third-party extension** | Syscall layer (CCM required) | Must have a valid, signed CCM. `syscalls_used` checked on every call. |
| **AI Agent** (autonomous) | Syscall layer via Worker SDK | Law 6 enforced: every AI invocation is checked for grounding source. AI cannot invoke `Commit()` for facts not sourced from `Query()` results. |
| **Another EKOS service** (internal) | Syscall layer | No privileged internal bypass. Internal EKOS services are principals with their own capability grants — not trusted by default. |

---

## Part IV — How This Book Governs

- **Precedence:** Book 1 > any volume. On conflict, Book 1 wins; the volume is corrected.
- **Citation:** volumes cite laws as "Law N" (replacing the older "Constitution §N"
  references, which remain valid aliases: old §1→Law 1, §2→Law 4, §3→Law 5, §4→Law 2/3,
  §5→Law 6, §6→Law 7/8/10, §7→Law 11).
- **Amendment:** a law changes only by an explicit, logged decision recorded here and in
  the master index — never silently in a volume.
- **The Manifesto (Part I)** is the *why*; it is consulted whenever a decision feels
  ambiguous — the answer is the one that best serves "knowledge as the primary
  abstraction, capabilities from primitives, platform before application."

---

## Part IV — Companion Documents

- **Design Methodology** (`DESIGN-METHODOLOGY.md`) — the questions and five lenses by
  which capabilities are discovered and locked. (To be written — gap-closure METHOD.)
- **The OS abstraction volumes** (18 Kernel, 19 Compiler, 20 Projection Engine, 21
  Acquisition, 22 OS APIs, 23 App Runtime & SDK, 24 Living Books, 25 Self-Hosting
  Reference) — the layered realization of Part I §4.
- **The 18 requirement volumes** (00–17) — the WHAT, all subordinate to these laws.
- **The Architecture Phase** (`architecture/`) — the HOW, beginning after gap-closure.

---

_Book 1 establishes that EKOS is an operating system whose primary abstraction is
knowledge, whose capabilities emerge from primitives, and which is built to last fifteen
years. Everything else is commentary._
