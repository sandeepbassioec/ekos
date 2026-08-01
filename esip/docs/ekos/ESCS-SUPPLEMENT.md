# EKOS — Enterprise Semantic Computing Specification (ESCS) Supplement

> **Status:** Formalization (2026-08-01). Feature-frozen; this document refines and
> extends existing terminology — it does NOT add new pillars or break the Feature Freeze
> (ADR-018). Source: `C:\VectorVolt\docs\ekos-application-platform.txt` (636KB).
>
> Requirement IDs follow the pattern `EKOS-ESCS-NNN`.

---

## 1. Purpose

This document captures the **Enterprise Semantic Computing Specification (ESCS)** — a
formal refinement of EKOS terminology introduced by new research. It formalizes the
computing paradigm category name, introduces a sub-atomic meaning layer below the
Knowledge Kernel (Vol 18), expands EKR's role from file-format family to universal
interaction shell, specifies the Application Manifest, formalizes the Capability
Structure, and defines the Persistence Engine's contracts and storage planning
responsibilities. Nothing here contradicts the existing 27-volume catalog or the
Architecture Phase; every item is a formalization or precision-sharpening of a concept
already present.

---

## 2. Enterprise Semantic Computing (ESC)

**EKOS-ESCS-001** — The formal category name for the computing paradigm EKOS embodies
SHALL be **Enterprise Semantic Computing (ESC)**. ESC is the successor to procedural,
object-oriented, and service-oriented computing at the enterprise scale. _(Must · MVP)_

**EKOS-ESCS-002** — The ESC computing model chain SHALL be:

```
Business Intent
      ↓
Enterprise Knowledge
      ↓
Semantic Compilation          ← Knowledge Compiler (Vol 19)
      ↓
Knowledge Runtime             ← Knowledge Runtime Architecture (A13)
      ↓
Semantic Runtime              ← Knowledge Representation Runtime (Vol 20 / KRR)
      ↓
Projection Runtime            ← KRR Projection layer
      ↓
Living Experiences            ← Living Applications + Living Documents
```

**EKOS-ESCS-003** — The platform SHALL expose this chain as a deterministic pipeline:
given the same Enterprise Knowledge and Business Intent, the resulting Living Experience
MUST be reproducible. Non-determinism is confined to the AIInsight KO production layer
(Book 2 §6). _(Must · MVP)_

**EKOS-ESCS-004** — All marketing, documentation, and standards submissions from EKOS
SHOULD use "Enterprise Semantic Computing" as the category descriptor to establish
category ownership and patent-filing coherence. _(Should · P2)_

---

## 3. Enterprise Meaning — Sub-Atomic Semantic Layer

**EKOS-ESCS-005** — The platform SHALL recognize **Enterprise Meaning** as the most
primitive, indivisible semantic truth in the system — the atom BELOW the Knowledge Atom
(Vol 18 §3). Enterprise Knowledge is composed FROM Enterprise Meanings. _(Must · MVP)_

**EKOS-ESCS-006** — The semantic hierarchy SHALL be:

```
Enterprise Meaning            ← simplest indivisible semantic truth
      ↓ (Composition)
Enterprise Knowledge          ← Knowledge Objects assembled from Meanings
      ↓
Semantic Runtime              ← compiled, validated, executable knowledge
      ↓
Projection Runtime            ← representation-independent output
      ↓
Living Experiences            ← Living Applications + Living Documents
```

**EKOS-ESCS-007** — Every Enterprise Meaning SHALL answer exactly five questions. No
Meaning that cannot answer all five is valid:

| Question | Dimension |
|---|---|
| What is it? | Identity |
| Why does it exist? | Purpose |
| What does it relate to? | Relationships |
| What constrains it? | Rules / Invariants |
| What does it enable? | Capabilities unlocked |

**EKOS-ESCS-008** — The Knowledge Kernel (Vol 18) SHALL be extended to maintain
Enterprise Meanings as first-class primitives, with their own stable IDs, versioning,
and bitemporal lifecycle, separate from — but composable into — Knowledge Objects.
_(Must · P2)_

**EKOS-ESCS-009** — Enterprise Meanings SHALL be immutable once published to a Semantic
Runtime. Evolution occurs through versioned replacement and composition revision, never
in-place mutation. _(Must · MVP)_

---

## 4. EKR as Universal Interaction Shell

**EKOS-ESCS-010** — The **Enterprise Knowledge Repository (EKR)** SHALL be formally
redefined as the **universal interaction shell** for EKOS. It presents ANY projection —
structured documents, interactive applications, dashboards, REST/GraphQL APIs, voice
interfaces, and conversational AI — from a single knowledge source. _(Must · MVP)_

> **Relationship to Vol 06/07:** Vol 06 defines the `.ekr` format and cryptographic
> packaging; Vol 07 defines the Viewer family and Crypto-Expiration. ESCS-010 does NOT
> replace those specifications. It expands EKR's _scope_: EKR is no longer "the file
> format + viewer family"; it is the shell that can also host live applications, APIs,
> and voice projections — each still sealed within the same cryptographic envelope.

**EKOS-ESCS-011** — The division of responsibilities SHALL be:

| Layer | Responsibility |
|---|---|
| **EKOS Knowledge Kernel** | Owns meaning — the authoritative semantic graph |
| **EKR (Universal Shell)** | Owns presentation — all projection surfaces |
| **Projection Runtime (KRR)** | Translates Knowledge → Projection on demand |

**EKOS-ESCS-012** — EKR SHALL support projection targets including (non-exhaustive):
structured document, reactive web application, mobile-native screen, REST API endpoint,
GraphQL schema, voice response (TTS + NLU), conversational AI context packet, and
embedded widget. Each target is a **projection**, not a copy of the data. _(Must · P2)_

**EKOS-ESCS-013** — Every EKR interaction surface — regardless of projection target —
SHALL enforce the same ABAC policy, constitutional compliance (CCR), and audit trail as
the Knowledge Kernel. There is no "lightweight" shell that bypasses governance.
_(Must · MVP)_

---

## 5. Application Manifest Specification

**EKOS-ESCS-014** — Applications on EKOS SHALL be described by a **semantic manifest**
rather than source code, database schema, or UI implementation. The manifest is the
application. The Projection Runtime generates the executable experience from it.
_(Must · MVP)_

**EKOS-ESCS-015** — An Application Manifest SHALL contain exactly these five top-level
sections:

| Section | Content |
|---|---|
| **Purpose** | What the application exists to accomplish (semantic, not technical) |
| **Audience** | The personas and roles the application serves |
| **Capability Assembly** | The ordered set of Capabilities that compose the application |
| **Experience Organization** | Navigation, grouping, and sequencing of Capabilities |
| **Projection Targets** | Which EKR surfaces this manifest targets (web / mobile / API / voice / …) |

**EKOS-ESCS-016** — An Application Manifest SHALL contain NO source code, NO SQL
schema, NO CSS layout rules, and NO implementation-layer directives. All of those are
the Projection Runtime's responsibility. _(Must · MVP)_

**EKOS-ESCS-017** — Application Manifests SHALL be stored as Knowledge Objects in the
Knowledge Kernel, subject to the same versioning, ABAC, and bitemporal lifecycle as any
other KO. _(Must · MVP)_

**EKOS-ESCS-018** — The five design goals for the Application Manifest are:

1. **Semantic completeness** — the manifest fully describes WHAT, not HOW.
2. **Projection-target independence** — one manifest, many surfaces.
3. **Composability** — Capabilities are reusable across manifests.
4. **Evolvability** — when Knowledge changes, the manifest adapts without re-authoring.
5. **Governance inheritance** — policy, audit, and compliance come from the Kernel, not the manifest.

---

## 6. Capability Structure Formalization

**EKOS-ESCS-019** — A Capability (Vol 02, Vol 23) SHALL have the following formal
section structure. All sections marked **mandatory** MUST be present; others SHOULD be
present when applicable:

| Section | Mandatory? | Description |
|---|---|---|
| **Intent** | ✅ | WHY the Capability exists — its semantic purpose |
| **Interaction Contract** | ✅ | Required fields, optional fields, conditional visibility, validation sequence |
| **Projection Hints** | ✅ | Presentation guidance (labels, grouping, ordering) — guidance only; Projection Runtime decides implementation |
| **Persistence Contracts** | ✅ | What state the Capability produces and how it must be persisted (see §7) |
| **Composition Rules** | When composed | Which other Capabilities this one requires or precludes |
| **Policy References** | When governed | Which policy-as-code rules apply |
| **AI Integration Points** | When AI-augmented | Where AI layer may inject; determinism boundaries |
| **Audit Requirements** | When regulated | Which interactions produce audit events |

**EKOS-ESCS-020** — The **Intent** section SHALL be a single, unambiguous semantic
statement that any domain expert (non-technical) can validate. A Capability whose Intent
cannot be explained to a domain expert in one sentence is incorrectly scoped. _(Must · MVP)_

**EKOS-ESCS-021** — The **Interaction Contract** SHALL specify: (a) required fields with
their semantic type (not data type), (b) optional fields, (c) conditional visibility
rules expressed as semantic predicates (not JavaScript), and (d) the validation sequence
— in the order validations run, not the order fields appear on screen. _(Must · MVP)_

**EKOS-ESCS-022** — **Projection Hints** SHALL be declarative guidance, never
imperative layout instructions. Examples of valid hints: `display-priority: high`,
`grouping: financial-summary`, `ordering: chronological`. Examples of invalid hints:
`margin-top: 16px`, `col-span: 3`. _(Must · MVP)_

---

## 7. Persistence Engine Formalization

### 7.1 Persistence Contracts

**EKOS-ESCS-023** — Capabilities SHALL express their persistence needs as **Persistence
Contracts**, never as SQL statements, ORM mappings, or physical storage directives. The
Persistence Engine translates Contracts to physical storage. _(Must · MVP)_

**EKOS-ESCS-024** — A Persistence Contract SHALL specify:

| Field | Description |
|---|---|
| **Semantic Identity** | What this state represents in the domain |
| **State Transitions** | Valid lifecycle transitions (state machine, not triggers) |
| **Persistence Intent** | `append-only`, `mutable`, `immutable`, or `ephemeral` |
| **Consistency Requirements** | `strong`, `eventual`, or `causal` |
| **Audit Requirements** | Which state changes must appear in the CTS audit chain |
| **Retention Requirements** | How long state must be retained and under what policy |
| **Storage Hints** | Advisory hint to Storage Planner (e.g., `prefers-graph`, `high-read-frequency`) — not a mandate |

**EKOS-ESCS-025** — Capabilities MUST NOT generate SQL. Any requirement in prior volumes
that implies direct SQL generation by a Capability is superseded by ESCS-023/024.
Physical storage is entirely the Persistence Engine's concern. _(Must · MVP)_

### 7.2 Enterprise State vs. Enterprise Knowledge

**EKOS-ESCS-026** — The platform SHALL formally distinguish:

| Concept | Definition | Changes how often | Governed by |
|---|---|---|---|
| **Enterprise Knowledge** | The semantic model — concepts, relationships, policies, ontology | Infrequently (deliberate evolution) | Knowledge Kernel (Vol 18) |
| **Enterprise State** | Operational conditions — `CustomerRegistered`, `InvoiceApproved`, `ShipmentDispatched` | Continuously (runtime events) | Persistence Engine |

Both are represented as Knowledge Objects with bitemporal stamps, but they live in
different logical partitions and are governed by different lifecycle rules. _(Must · MVP)_

### 7.3 Storage Planner

**EKOS-ESCS-027** — The Persistence Engine SHALL contain a **Storage Planner**
component that selects the optimal physical storage strategy for each Persistence
Contract. The Storage Planner's decision SHALL be automatic and auditable. _(Must · P2)_

**EKOS-ESCS-028** — The Storage Planner SHALL evaluate the following signals when
selecting a storage strategy:

| Signal | Examples |
|---|---|
| Relationship density | sparse → relational; dense → graph |
| Read/write frequency | high-read → cache+OLAP; high-write → append-only event store |
| Latency requirements | sub-ms → VectorVolt columnar; relaxed → object storage |
| Retention period | short → in-memory / Redis; long + regulatory → WORM / object |
| Regulatory compliance | PII → encrypted column store; HIPAA → isolated partition |
| Consistency model | strong → RDBMS; eventual → document / event store |

**EKOS-ESCS-029** — The Storage Planner SHALL route Persistence Contracts to the
following Storage Provider types. All providers are pluggable (ADR-052 `IStorageProvider`
pattern):

| Provider Type | Primary use-case |
|---|---|
| Relational | Structured state, referential integrity, ACID transactions |
| Document | Flexible schema, hierarchical state |
| Graph | High-density relationship traversal (VectorVolt extension) |
| Object | Large binary assets, EKR packages, audit archives |
| Event Store | Append-only state transitions, event sourcing |
| Vector | Semantic similarity, AI embedding retrieval |
| Time Series | Sensor / telemetry / metrics state |
| Key-Value | Session state, ephemeral lookups, cache |

---

## 8. Living Application

**EKOS-ESCS-030** — **Living Application** SHALL be a first-class concept on equal
standing with Living Document (Vol 24). Both are projections of Enterprise Knowledge
through the EKR universal shell. _(Must · P2)_

> **Relationship to Vol 24:** Vol 24 defines the Living Knowledge Publication Framework
> for Living Books. Living Applications extend that same projection model: when
> Enterprise Knowledge changes, both Living Documents and Living Applications adapt
> automatically — navigation adapts, capabilities appear or retire, validations evolve
> — without manual re-authoring.

**EKOS-ESCS-031** — A Living Application SHALL automatically adapt when its underlying
Enterprise Knowledge changes: (a) new Capabilities appear if their preconditions are
met, (b) deprecated Capabilities retire gracefully with migration paths, (c) validation
rules evolve without a code deployment, and (d) navigation reorganizes to reflect the
updated Capability Assembly. _(Must · P2)_

---

## 9. Enterprise Studio

**EKOS-ESCS-032** — The EKOS authoring IDE SHALL be formally named **Enterprise Studio**
(previously discussed under the ESIP trajectory). Enterprise Studio is an EKOS
application that runs on the platform it authors — eating its own dog food (Vol 25).
_(Must · P2)_

**EKOS-ESCS-033** — Enterprise Studio SHALL contain the following named workspaces:

| Workspace | Purpose |
|---|---|
| **Concept Designer** | Author Enterprise Meanings and Knowledge Objects |
| **Policy Designer** | Define policy-as-code rules and governance constraints |
| **Capability Composer** | Build and validate Capabilities (all 8 Capability sections) |
| **Interaction Designer** | Author Interaction Contracts and Projection Hints |
| **Experience Composer** | Assemble Application Manifests from Capabilities |
| **Projection Designer** | Configure per-target projection hints and EKR surfaces |
| **Deployment Center** | Package, sign, and publish `.ekr` bundles |
| **Validation Center** | Run constitutional compliance checks before publication |
| **Marketplace** | Browse and compose from the Knowledge Economy (Vol 13) |
| **AI Assistant** | Ambient AI (Vol 03) integrated into every Studio workspace |

---

## 10. EKR as Application Deployment & Subscription Platform

> **This is the paradigm shift.** The preceding sections describe WHAT EKOS/EKR is.
> This section describes the one thing that changes the entire application development
> landscape: **applications are no longer installed. They are subscribed to.**

### 10.1 The New Application Distribution Model

**EKOS-ESCS-034** — **Knowledge Packages are the deployable unit of enterprise
computing**, replacing installed applications, bespoke SaaS deployments, and
per-organization codebases. A Knowledge Package bundles an Application Manifest,
its referenced Capabilities, Experiences, Projection Definitions, governance policies,
and cryptographic metadata into a single distributable artifact. _(Must · MVP)_

> Traditional model: developer writes code → builds binary → deploys to server →
> user installs app or opens URL → org maintains indefinitely.
>
> EKOS model: developer authors Application Manifest + Capabilities → compiles to
> Knowledge Package → publishes to EKR registry → user subscribes inside EKR →
> Projection Runtime renders the application on demand. **No code ships to the user.
> No installation occurs. The Knowledge Package is the app.**

**EKOS-ESCS-035** — EKR SHALL function as an **application runtime platform** — not
only a document viewer. A user who opens EKR and subscribes to a Knowledge Package
SHALL receive a fully functional application experience rendered by the Projection
Runtime, indistinguishable in capability from a traditionally-built application.
_(Must · MVP)_

**EKOS-ESCS-036** — The "Viewer" label in Vol 07 does NOT fully describe EKR's scope.
Vol 07 specifies the cross-platform distribution and rendering surface (Electron,
WASM, mobile), the cryptographic enforcement, and the tier model. Those specifications
remain authoritative and unchanged. ESCS-035 extends the _role_ of that same surface:
it is simultaneously a document viewer (Vol 07), a live-application runtime (this
section), and an API gateway (Vol 06 EKR Gateway Module). _(Must · MVP)_

### 10.2 Publishing a Knowledge Package

**EKOS-ESCS-037** — An Application Manifest (§5) SHALL be compiled by the Knowledge
Compiler (Vol 19) into a **Knowledge Package** — a cryptographically signed,
versioned, self-describing deployable artifact. The package is complete: receiving the
package is sufficient to run the application; no supplementary build step or runtime
configuration is required. _(Must · MVP)_

**EKOS-ESCS-038** — Knowledge Packages SHALL be published to a **Knowledge Package
Registry** — an EKOS-governed distribution endpoint. The registry enforces:
(a) cryptographic signature verification before admission, (b) constitutional compliance
gate (CCR Tier 1 + Tier 2 pre-flight), (c) semantic version management, and (d)
dependency resolution across packages. _(Must · MVP)_

**EKOS-ESCS-039** — The Knowledge Economy (Vol 13) serves as the commercial layer
over the Knowledge Package Registry: publishers monetize packages via subscription,
per-use, or one-time license; the platform manages escrow, revenue share, and
entitlement. An EKOS-provided free tier SHALL allow OSS packages to be published and
subscribed to without cost. _(Should · P2)_

### 10.3 Subscribing to a Knowledge Package

**EKOS-ESCS-040** — A user or organization subscribes to a Knowledge Package inside
EKR. Upon subscription activation: (a) EKR resolves the package from the registry,
(b) the Projection Runtime compiles the applicable projections for the subscriber's
context (role, device, locale, permissions), (c) the application becomes immediately
available — no download, no installation wizard, no restart required. _(Must · P2)_

**EKOS-ESCS-041** — Application updates are absorbed transparently. When a publisher
publishes a new version of a Knowledge Package, subscribers receive the update through
the Projection Runtime without any manual action. If the Knowledge change is breaking
(semantic major version bump), the subscriber is notified and given a migration path
before the update is applied. _(Must · P2)_

**EKOS-ESCS-042** — An organization subscribing to a Knowledge Package does NOT
receive source code, database schemas, or implementation artifacts. The Knowledge
Package IS the application. Reverse-engineering, forking, or rehosting a subscribed
package without publisher authorization is prohibited by cryptographic means (Vol 06
copy-proofing + activation nonce) and ABAC policy. _(Must · MVP)_

### 10.4 What This Changes About Application Development

**EKOS-ESCS-043** — The application development lifecycle CHANGES fundamentally:

| Traditional SDLC | EKOS ESC Model |
|---|---|
| Define requirements (documents) | Author Application Manifest (semantic artifact) |
| Design database schema | Define Capabilities with Persistence Contracts |
| Write backend code | Capabilities are the executable artifact — no code |
| Write frontend code | Experience Designer + Projection Runtime handle rendering |
| Deploy to servers | Publish Knowledge Package to registry |
| Users install/access app | Users subscribe to package inside EKR |
| Push update → re-deploy → notify users | Publish new package version → subscribers updated automatically |
| Maintain separate docs | Living Documents generated as projections of the same Knowledge |

**EKOS-ESCS-044** — A developer building on EKOS SHALL author Capabilities and an
Application Manifest using Enterprise Studio. The developer DOES NOT write UI code,
SQL migrations, REST endpoint handlers, or deployment scripts. Those concerns are
handled by: Projection Runtime (UI), Persistence Engine (storage), Semantic Runtime
(APIs), and the EKR distribution infrastructure (deployment). _(Must · P3)_

### 10.5 EKR as the Common Runtime Environment

**EKOS-ESCS-045** — EKR is the **runtime common ground** across the enterprise — the
equivalent of what the browser is to the web, with three critical differences:
(1) applications run deterministically from semantic definitions, not interpreted
JavaScript; (2) governance, ABAC, and constitutional compliance are enforced at the
runtime level, not the application level; (3) the same Knowledge Package runs
identically on desktop EKR, mobile EKR, embedded EKR, and WASM EKR — the projection
surface changes, the semantic model does not. _(Must · P2)_

**EKOS-ESCS-046** — Because the Projection Runtime inside EKR is the universal
renderer, **the same published Knowledge Package produces**: a desktop enterprise
application (Electron EKR), a mobile experience (React Native EKR), a REST API
(EKR Gateway Module), a Living Document (EKR Document Renderer), and a
conversational AI grounding package — all from a single manifest, with no additional
developer work per target. _(Must · P3)_

---

## 11. Traceability Matrix

Every ESCS requirement maps forward to the existing volumes (WHAT), architecture docs (HOW), and implementation epics (WHEN). A requirement with no row here is a gap.

| ESCS Req | Title (short) | Existing Volume(s) | Architecture Doc(s) | Implementation Epic |
|---|---|---|---|---|
| ESCS-001 | ESC category name | — (new terminology) | A01 (quality attributes) | E0 (pre-deploy docs) |
| ESCS-002 | ESC computing chain | Vol 18 (Kernel), Vol 19 (Compiler), Vol 20 (KRR) | A13 (Knowledge Runtime), A02 (Logical Arch) | E11 (Compiler+Projection) |
| ESCS-003 | Deterministic pipeline | Vol 18 §6, Book 2 §6 | A13 §4 (determinism boundaries) | E11 |
| ESCS-004 | Marketing category name | Vol 16 (patents) | A01 | — |
| ESCS-005 | Enterprise Meaning primitive | Vol 18 §3 (Knowledge Atom — extended) | A02 (Logical Arch layer model) | E2 (KO Model & graph) |
| ESCS-006 | Semantic hierarchy | Vol 18, Vol 20 | A13 | E2, E11 |
| ESCS-007 | Five-question Meaning validation | Vol 01 (ontology), Vol 18 | A04 (ontology architecture) | E2 |
| ESCS-008 | Meaning as first-class KO | Vol 18 §3, Vol 01 | A04 | E2 |
| ESCS-009 | Meanings immutable | Vol 01 §4 (non-destructive), Book 2 §3 | A04 | E2 |
| ESCS-010 | EKR = universal shell | Vol 07 (surface), Vol 06 (format) | A13, A15 | E9 (extended) |
| ESCS-011 | EKOS/EKR/KRR responsibilities | Vol 06, Vol 07, Vol 20 | A02, A13 | E9, E11, E12 |
| ESCS-012 | EKR projection targets | Vol 07 §4 (EKR Everywhere), Vol 20 | A15 (Runtime Distribution) | E9 |
| ESCS-013 | EKR enforces full governance | Vol 04 (ABAC), Vol 05 (CCR), Vol 07 | A06, A13 §3 | E6, E9 |
| ESCS-014 | Application = semantic manifest | **Vol 23 §2 (EKRA/Application Bundle)** | A11 E12 | E12 |
| ESCS-015 | Manifest five sections | **Vol 23 §2 (EKRA manifest)** | A11 E12 | E12 |
| ESCS-016 | Manifest has no code/SQL | Vol 23, Vol 06 | A11 | E12 |
| ESCS-017 | Manifests stored as KOs | Vol 23 §3, Vol 01 | A04 | E2, E12 |
| ESCS-018 | Manifest design goals | Vol 23, Vol 20 | A11 | E12 |
| ESCS-019 | Capability formal structure | Vol 02 §3 (capability anatomy) | A02 | E3 (Widgets→projections) |
| ESCS-020 | Intent mandatory | Vol 02 §3 | A02 | E3 |
| ESCS-021 | Interaction Contract | Vol 02 §4, Vol 22 (syscalls) | A02, A13 | E3 |
| ESCS-022 | Projection Hints declarative | Vol 20 §3 (KRR), Vol 02 | A02 | E11 |
| ESCS-023 | Persistence Contracts not SQL | Vol 18 §9 (persistence policy), Vol 15 §8 | A04 (polyglot CQRS) | E2, E11 |
| ESCS-024 | Persistence Contract fields | Vol 18, Vol 15 §8 | A04 | E2 |
| ESCS-025 | Capabilities MUST NOT generate SQL | Vol 18, Vol 22 | A04 | E2, E11 |
| ESCS-026 | Enterprise State vs Knowledge | Vol 01 §6, Book 2 §2 | A04 | E2 |
| ESCS-027 | Storage Planner component | Vol 15 §8 (polyglot) | A04 | E2 (P2) |
| ESCS-028 | Storage Planner signals | Vol 15 §8 | A04 | E2 (P2) |
| ESCS-029 | Storage Provider types | Vol 15 §8, ADR-032/033 | A04 | E2 (P2) |
| ESCS-030 | Living Application = first-class | **Vol 24 (Living Pub Framework)**, Vol 02 | A13, A15 | E11, E13 |
| ESCS-031 | Living App adapts automatically | Vol 24, Vol 20 §5 | A13 | E11 |
| ESCS-032 | Enterprise Studio formal name | Vol 23 §8, ESIP (current impl) | A10 (migration) | E12 |
| ESCS-033 | Studio workspaces | Vol 23 §8, Vol 02 (Widget OS) | A10, A11 E12 | E12 |
| **ESCS-034** | **Knowledge Packages = deployable unit** | **Vol 06 (format/registry), Vol 23 (App Bundle)** | **A11 E9+E9b** | **E9 (P2), E9b (P2/P3)** |
| **ESCS-035** | **EKR = application runtime** | **Vol 07 (surface), Vol 23 (App Runtime)** | **A13, A15** | **E9 (extended), E12** |
| ESCS-036 | Vol 07 scope addendum | Vol 07 §1 addendum | — | — |
| **ESCS-037** | **App Manifest → Knowledge Package** | **Vol 06 §3 (publishing), Vol 19 (Compiler), Vol 23** | **A11 E9+E11** | **E9b (P2)** |
| **ESCS-038** | **Knowledge Package Registry** | **Vol 06 §4.11 (Bundle Store), Vol 23 §5 (App Store)** | **A08 (deployment), A20 (SaaS)** | **E9b (P2/P3)** |
| **ESCS-039** | **Commercial layer over registry** | **Vol 13 (Knowledge Economy)** | **A20 (SaaS), A08** | **E13 (P3)** |
| **ESCS-040** | **Subscribe → immediate app** | **Vol 23 §5 (App Store), Vol 07 §6 (distribution)** | **A15, A20** | **E9b (P3)** |
| **ESCS-041** | **Transparent package updates** | **Vol 23 §6 (app longevity), Vol 06 §4.12 (delta)** | **A09 (sustainability)** | **E9b (P3)** |
| **ESCS-042** | **No source code ships to user** | **Vol 06 §4.1 (format secrecy), Vol 04 (ABAC)** | **A06** | **E6, E9** |
| **ESCS-043** | **SDLC change table** | Cross-cutting all volumes | A11 (all epics) | All phases |
| **ESCS-044** | **Developer = manifest author** | **Vol 23, Vol 19, Enterprise Studio** | **A11 E12** | **E12 (P2)** |
| **ESCS-045** | **EKR = common runtime environment** | **Vol 07, Vol 23, Vol 22** | **A13, A15** | **E9, E12** |
| **ESCS-046** | **One package → all surfaces** | **Vol 20 (Projection Engine), Vol 07** | **A15** | **E11, E9** |

> Bold rows = the Application Deployment & Subscription Platform (ESCS-034…046) — the
> paradigm shift. These map primarily to **Vol 06, Vol 23, Vol 13, E9, E9b, E12, E13**.

---

## 12. Source File Reference

| File | Size | Role |
|---|---|---|
| `C:\VectorVolt\docs\ekos-application-platform.txt` | 636 KB | Primary source for this supplement — the formal ESCS specification |

> **Do NOT re-read this file in full each session.** Grep for specific concepts.
> This supplement captures all net-new material; the existing 27-volume catalog remains
> authoritative for everything it already covers.

---

_Created 2026-08-01. ESCS formalization pass — Feature Freeze maintained (ADR-018)._
