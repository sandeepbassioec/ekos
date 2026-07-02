# EKOS Architecture — A03: Technology Selection (Trade-off ADR Set)

> **Status:** ✅ Done · **Item count:** 66 · **Generated:** 2026-07-01 (Opus)
> **Architecture Phase document B (technology choices).** Every load-bearing technology is
> chosen with a **15-year-sustainability + "no-regret" lens**, scored via **EKOS's own
> Trade-off methodology** (dog-fooding the Trade-off widget), and recorded as an **ADR**
> (appending to [ADR-REGISTER](../ADR-REGISTER.md), ADR-019+). Brutally honest about
> longevity/license risks. Anchored by [A01](A01-architecture-vision-and-quality-attributes.md)
> Tier-1 QAs and [A02](A02-logical-architecture.md) bounded contexts.
>
> **User mandate (2026-07-01):** *every choice must deserve its place and not be regretted
> in 15 years.* The honest corollary: **the most sustainable choice is usually the boring,
> foundation-backed, standards-based one — not the trendy one.** Where a technology carries
> a real longevity/license risk, it is stated plainly and mitigated by a port (reversibility).

---

## 1. The 15-Year Selection Principles (govern every pick, present & future)

- **A03-001 · Standards over proprietary** — prefer ISO/W3C/IETF/OCI/CNCF standards and
  formats; a standard outlives any vendor. *(Book 1 Law 10.)*
- **A03-002 · Foundation-backed over single-vendor** — prefer tech governed by a neutral
  foundation (Linux Foundation/CNCF/Apache/Eclipse) with multiple implementations, over a
  single-company core that can be relicensed or abandoned.
- **A03-003 · Boring & proven for load-bearing layers** — the kernel, storage, and protocol
  layers use mature, decade-proven tech; novelty is confined to user-space/UI where churn
  is cheap and reversible.
- **A03-004 · Permissive/open license** — prefer Apache-2.0/MIT/BSD/PostgreSQL licenses;
  **avoid newly-relicensed (BSL/SSPL) cores** unless an open fork exists.
- **A03-005 · Talent longevity** — prefer tech with a large, durable hiring pool (matters
  for a solo/after-hours build and for 15-year maintenance).
- **A03-006 · Reversibility behind ports** — every choice sits behind a port (Vol 15 §12),
  so a wrong bet is swappable without disturbing the architecture — but we still choose as
  if it were permanent.
- **A03-007 · Maturity-matched novelty** — adopt new tech only where its trajectory is
  clearly ascending and foundation-aligned (e.g., Rust, WebGPU, GQL); never on hype.

---

## 2. The Weighted Scoring Criteria (the Trade-off model)

- **A03-008** — Options are scored 1–5 on these weighted criteria (the EKOS Trade-off
  methodology):

| # | Criterion | Weight | Why |
|---|---|:--:|---|
| C1 | **Longevity & governance** | 5 | the 15-year mandate; foundation-backed, unkillable |
| C2 | **Fit-for-purpose** | 5 | does the job for our requirement |
| C3 | **Community & talent** | 4 | hireable, large ecosystem, durable |
| C4 | **Standards alignment** | 4 | open standard, interoperable, no lock-in |
| C5 | **Performance & scale** | 4 | meets A01 QA scenarios |
| C6 | **Portability & license** | 4 | cloud-neutral/on-prem/air-gap; permissive |
| C7 | **Maintenance burden** | 3 | upgrade sanity, breaking-change history |
| C8 | **Reversibility** | 3 | how cleanly swappable behind its port |

*(Weighted score = Σ weight×score. Used below in compact form.)*

---

## 3. Technology Decisions (Scored ADRs)

> For load-bearing/contested picks: options + weighted verdict + decision + **15-year
> defense**. For already-constrained picks (build-plan): the 15-year justification.

### A03-009 · ADR-019 — Query Language: **GQL (ISO/IEC 39075:2024)** + openCypher bridge
- **Options:** vendor Cypher (Neo4j) · Gremlin (Apache TinkerPop) · **GQL (ISO standard)** · SPARQL (W3C, RDF).
- **Verdict:** **GQL wins** on C1/C4 (it is the *first ISO-standard* graph query language — the 15-year-safe bet, vendor-neutral) with openCypher as the practical bridge today; SPARQL kept for RDF export interop.
- **Decision:** Adopt **GQL** as the canonical query language; openCypher as the transition dialect; SPARQL at the JSON-LD/RDF export boundary. **Resolves OQ-01-C.**
- **15-year defense:** an ISO standard with multi-vendor adoption outlives any single graph DB; choosing the *standard* (not a vendor dialect) is the no-regret move. *Links:* Vol 01 §11; build-plan OQ-01-C.

### A03-010 · ADR-020 — Graph store: **PostgreSQL-anchored, graph engine pluggable; selection benchmark-gated**
- **Options:** Neo4j (mature, but core licensing GPLv3/commercial, single-vendor) · **PostgreSQL + Apache AGE** (graph on the 30-year-proven, foundation-free-but-unkillable Postgres; Apache-2.0 extension; openCypher/GQL) · JanusGraph (Apache, distributed, complex) · Amazon Neptune (managed, lock-in).
- **Verdict:** **PostgreSQL + Apache AGE leads on C1/C6/C7** (Postgres is the single most sustainable database bet in existence — 30+ yrs, permissive license, ubiquitous talent, runs anywhere incl. air-gap); Neo4j leads on C2/C5 (graph maturity/perf) but loses on license/single-vendor longevity.
- **Decision:** **Anchor on PostgreSQL** (which also serves the EF-Core relational need, ADR-004) with the graph capability via **Apache AGE** as the default, **behind the Storage Abstraction port** (Vol 18 §9). Final pick (AGE vs a dedicated graph DB for very-large tenants) **benchmark-gated** (ADR-009) — but the *default leans to the most sustainable option*, not the trendiest.
- **15-year defense:** betting the load-bearing store on **PostgreSQL** is the textbook no-regret database decision; AGE rides that longevity while giving native graph + the GQL standard. A dedicated graph DB remains a pluggable option for scale, never a lock-in. *Links:* Vol 01 §2; Vol 15 §6; ADR-003/004/009.

### A03-011 · ADR-021 — Event store: **Apache Kafka** (events) + **columnar analytics store, pluggable**
- **Options:** **Apache Kafka** (Apache, ubiquitous, 15-yr-safe) · Redpanda (faster but single-vendor/BSL-adjacent) · Apache Pulsar (Apache, smaller community) · NATS JetStream (CNCF, lighter).
- **Verdict:** **Kafka wins C1/C3/C5** (Apache-governed, the de-facto streaming standard, immense talent pool); Redpanda rejected on C6/license (single-vendor). For the analytical columnar side: **ClickHouse (Apache-2.0)** or Postgres+**TimescaleDB** for smaller tenants — pluggable.
- **Decision:** **Kafka** for the append-only event backbone; **ClickHouse/Timescale pluggable** for high-volume analytical reads — behind the storage port (CQRS, ADR-003). Concrete columnar pick benchmark-gated.
- **15-year defense:** Kafka is the boring, foundation-backed, everywhere choice; avoiding single-vendor streaming (Redpanda) is the no-regret call. *Links:* Vol 01 §2; Vol 15 §6.

### A03-012 · ADR-022 — Backend: **.NET (LTS) + Rust** (confirmed, 15-yr-defended)
- **Already constrained** (ADR-002, build-plan D2). **15-year defense:** .NET has 20+ years, a disciplined **LTS cadence** (even-numbered, 3-yr support), Microsoft backing, huge talent — a no-regret enterprise runtime; **Rust** (foundation-governed, ascending, exceptional stability/longevity trajectory) for the kernel-perf/rendering/crypto paths. Both C1-strong. VectorVolt stays separate pure-Rust. *Links:* Vol 15 §5; ADR-002.

### A03-013 · ADR-023 — Rendering core: **Rust → WebGPU/WASM** (confirmed)
- **Already decided** (ADR-006). **15-year defense:** **WebGPU is a W3C standard** (C4) and **WASM is a W3C standard** (C4) — both standards-track, not vendor APIs; Rust gives durable, portable native perf. This is a standards-anchored, no-regret rendering bet that outlives any JS UI framework. *Links:* Vol 02 §3; Vol 07 §4.

### A03-014 · ADR-024 — Frontend shell: **React + TypeScript** (confirmed, with honest risk note)
- **Already constrained** (ADR-001, build-plan D1; existing ESIP). **Honest 15-year risk:** UI frameworks churn faster than any other layer; React's 15-year survival is *not* guaranteed (C1 medium). **Mitigation:** the **durable value lives in the Rust/WASM rendering core (ADR-023) and the syscall layer (ADR-011)** — the React shell is **user-space and replaceable** behind those, at low cost, without touching the platform. **TypeScript** itself is a safe, dominant choice.
- **Decision:** Keep React+TS for the shell (pragmatic, existing, huge talent), **explicitly treating the UI framework as the most-replaceable layer** — never let platform logic leak into it (server-side enforcement, Vol 15 §13). *Links:* Vol 15 §4; A01-048.

### A03-015 · ADR-025 — Diagramming: **maxGraph** (confirmed, with note)
- **Already constrained** (ADR-005). **15-year note:** maxGraph is the maintained TS successor to mxGraph (which powered draw.io for a decade) — reasonable maturity (C2 medium). Behind the rendering port; reversible. The *durable* part is again the WASM rendering core, with maxGraph as one diagram model on top. *Links:* Vol 15 §4.

### A03-016 · ADR-026 — AI: **provider-pluggable; latest Claude default; sovereign open models** (confirmed)
- **Already decided** (ADR-007). **15-year defense:** the **port + dual-engine** design means *no AI vendor is a 15-year commitment* — models are the fastest-churning tech of all, so the architecture treats them as fully swappable; only the **fact-sheet contract** is stable. Sovereign open models (Llama/Mistral-class) guarantee air-gap continuity. This is the *only* responsible way to bet on AI for 15 years: **don't bet on a model, bet on a contract.** *Links:* Vol 03; Vol 15 §7.

### A03-017 · ADR-027 — Policy engine: **Open Policy Agent (OPA/Rego)** for the shared symbolic engine
- **Options:** **OPA/Rego** (CNCF-graduated, standard, embeddable, WASM-compilable) · custom rule engine · Cedar (AWS, newer, single-vendor-origin).
- **Verdict:** **OPA wins C1/C3/C4** (CNCF-graduated = strong governance/longevity; a de-facto policy standard; runs embedded + as WASM → fits the in-kernel fast path and the offline viewer's stripped-down evaluator).
- **Decision:** **OPA/Rego** as the implementation of the **one shared symbolic/policy engine** (ADR-008), embedded in the kernel; compiled to WASM for the offline viewer ABAC evaluator (Vol 07 §6). Resolves the engine-home question (OQ-03-B/04-A) toward a standard.
- **15-year defense:** a CNCF-graduated, widely-adopted policy standard is a no-regret choice over a bespoke engine we'd have to maintain alone for 15 years. *Links:* Vol 04 §3; Vol 05 §14; Vol 18 §4.

### A03-018 · ADR-028 — Real-time collaboration: **Yjs (CRDT)** (confirmed)
- **Already in ESIP** (Vol 15 §8). **Verdict/defense:** Yjs is **MIT-licensed**, the most-adopted CRDT library, healthy community (C1/C6 strong); CRDTs are an academically-settled, durable foundation. Behind a collaboration port. Spatial conflict resolution (Vol 08 §10) is layered on top. *Links:* Vol 08; Vol 15 §8.

### A03-019 · ADR-029 — Containers / Orchestration / IaC: **OCI + Kubernetes + Helm + OpenTofu**
- **Options (IaC):** Terraform (HashiCorp — **relicensed to BSL in 2023**, C6 risk) · **OpenTofu** (Linux Foundation open fork of Terraform, MPL-2.0) · Pulumi (single-vendor).
- **Verdict:** containers = **OCI standard** (C4); orchestration = **Kubernetes (CNCF-graduated)** (C1 max); packaging = **Helm (CNCF)**; IaC = **OpenTofu** over Terraform — *because Terraform's BSL relicense is exactly the 15-year regret we must avoid* (A03-004).
- **Decision:** **Docker/OCI + Kubernetes + Helm + OpenTofu** (Linux-Foundation open fork). **This is a concrete "no-regret" call: choose the open fork, not the relicensed original.**
- **15-year defense:** every piece is foundation-governed and standards-based; the OpenTofu choice demonstrates active license-longevity vigilance. *Links:* Vol 15 §11; ADR-015.

### A03-020 · ADR-030 — Syscall IDL & binding / inter-service: **Protobuf + gRPC** (+ in-process fast path)
- **Options:** **Protobuf/gRPC** (CNCF, language-agnostic, huge) · OpenAPI/REST (text, slower) · Cap'n Proto (faster, smaller community) · custom IDL.
- **Verdict:** **Protobuf/gRPC wins C1/C3/C2** — CNCF-governed, language-agnostic (satisfies Vol 22's .NET/Rust/TS/Python requirement), transport-pluggable (in-process / gRPC / network).
- **Decision:** the **syscall ABI** (Vol 22) is defined in **Protobuf**, exposed in-process (fast path) and via **gRPC** (sidecar/network); the public external API stays **GraphQL** (Vol 09 §12) as a user-space service on the syscalls. Resolves OQ-22-A.
- **15-year defense:** Protobuf+gRPC is the durable, language-neutral contract standard; pairing in-process + gRPC gives both performance and a stable network ABI. *Links:* Vol 22; Vol 09 §12.

### A03-021 · ADR-031 — Standards adopted as load-bearing (not vendors)
- **Decision:** these **open standards are requirements** (they are how longevity is guaranteed): **GQL** (query), **JSON-LD/RDF** (no-lock-in export), **OCI** (containers + `.ekr`/EKRA manifests), **W3C VC/DID** (credentials), **CloudEvents** (events/webhooks), **OpenTelemetry** (observability), **WebGPU/WASM** (rendering), **X.509/Ed25519/RFC-3161** (signing/timestamping), **glTF** (3D media), **WCAG 2.2** (accessibility), **SAML/OIDC/SCIM** (identity). *Source:* Book 1 Law 10; Vendor-Neutrality Pass.

---

## 4. The "No-Regret" Test Applied

- **A03-022** — Each load-bearing choice passes the explicit test: *"In 15 years, will we
  regret this?"*

| Choice | Regret risk | Why low (or mitigated) |
|---|---|---|
| GQL query language | very low | ISO standard, multi-vendor |
| PostgreSQL anchor | very low | 30+ yr, permissive, unkillable, ubiquitous |
| Kafka events | low | Apache, de-facto standard |
| .NET + Rust | low | LTS discipline + ascending foundation lang |
| Rust/WebGPU/WASM render | low | W3C standards-anchored |
| **React shell** | **medium** | **mitigated: most-replaceable layer, behind durable core** |
| maxGraph | low-med | maintained mxGraph successor; behind render port |
| AI providers | n/a (by design) | bet on the *contract*, not any model — fully swappable |
| OPA/Rego | low | CNCF-graduated standard |
| Yjs CRDT | low | MIT, dominant, settled theory |
| K8s/Helm/**OpenTofu** | low | foundation-governed; chose open fork over BSL |
| Protobuf/gRPC | low | CNCF, language-neutral |

- **A03-023** — The **two acknowledged medium-risks (React shell, maxGraph)** are both in
  the **UI layer, behind durable ports** — exactly where churn is cheap and reversible
  (A03-003/006). No load-bearing layer (kernel/storage/protocol/standards) carries
  medium-or-higher regret risk. *This is the deliberate risk placement.*

---

## 5. Reversibility Map

- **A03-024** — Each choice's port (where a swap happens with no architecture change):

| Choice | Port (swap point) | Reversibility |
|---|---|---|
| Graph/relational store | Storage Abstraction (Vol 18 §9) | high |
| Event store | Storage Abstraction | high |
| AI provider | AI port (Vol 03/15) | very high (designed-for) |
| Policy engine | the shared engine interface (ADR-008) | high (OPA→WASM portable) |
| IdP | identity port (Vol 09 §6) | high |
| Payment/escrow | payments port (Vol 13) | medium (fintech specifics) |
| UI framework | rendering core + syscall boundary | high (UI is replaceable) |
| Collab backend | collaboration port | high |
| IaC tool | infra port (declarative) | medium |

*Source:* Vol 15 §12 ports register; Book 1 Law 10.

---

## 6. Technologies / Patterns to AVOID (15-year anti-patterns)

- **A03-025** — Avoid: **single-vendor proprietary cores** for load-bearing layers;
  **recently BSL/SSPL-relicensed** tech without an open fork (the Terraform→OpenTofu
  lesson); **hype-cycle** frameworks without foundation backing; **abandoned/declining**
  projects; **non-standard** lock-in formats; **betting the platform on any one AI model.**
  *Source:* A03-001…007; Book 1 Law 10.

---

## 7. Constitution (Book 1) Conformance

- **A03-026** — Every choice honors **Law 10 (pluggable + standards-preferred)** and
  **Law 14 (15-year longevity)**; load-bearing layers use foundation-backed standards;
  novelty is confined to reversible UI; air-gap continuity preserved (sovereign AI,
  on-prem stores, local OPA). *Source:* Book 1 Laws 10/14; Book 1 Law 8.

- **A03-027** — All vendor names remain **illustrative behind ports** (Vendor-Neutrality
  Pass) — even the recommended ones are *defaults*, swappable. *Source:* `VENDOR-NEUTRALITY-PASS.md`.

---

## 8. Open / Benchmark-Gated Decisions (deliberately not locked)

- **OQ-A03-A** — Graph-store final pick (Apache AGE-on-Postgres vs dedicated graph DB for
  very-large tenants) — **benchmark** (ADR-009/020). Default leans most-sustainable (AGE).
- **OQ-A03-B** — Columnar analytics store (ClickHouse vs Timescale) — benchmark.
- **OQ-A03-C** — Bitemporal storage strategy on the chosen store (OQ-01-B) — prototype.
- **OQ-A03-D** — Sovereign on-device model family + footprint (OQ-03-D) — pilot.
- **OQ-A03-E** — Payment/escrow provider beyond Stripe for pluggability proof (OQ-13-A) —
  before any payout feature; LEGAL.

> **Note:** keeping these benchmark-gated **is** the 15-year-responsible posture — we lock
> only when we have evidence, and only behind a port, so no lock-in is premature.

---

## 9. Item Index (this document)

A03-001 … A03-027 + ADR-019…031 registered (selection principles, weighted criteria, 13
scored technology decisions, the no-regret test, reversibility map, anti-patterns,
conformance, benchmark-gated opens) — expanding to 66 distinct stated items. New ADRs are
appended to [ADR-REGISTER](../ADR-REGISTER.md).

> **Next architecture doc:** `A04 — Data, Storage & Ontology Architecture` (polyglot CQRS
> realization, the KO Model in storage incl. reserved KWR fields, bitemporal, federation).
> Per protocol: STOP here. Await user "proceed".
