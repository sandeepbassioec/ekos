# EKOS Platform Architecture — Master Index & Tracker

> **The platform comes before the application.** EKOS is an Operating System; ESIP (the
> web app built so far) becomes one of the applications that runs *on top of* it. This
> phase architects the OS **before** any further coding.
>
> **Rule of this phase (user directive, 2026-07-01):** *"We should not run into
> development too easily — this will be a mistake. We need to plan, we need to
> architect, we need technology choices, and we need this platform to sustain for the
> next 15 years by regularly maintaining applications on latest versions."*
>
> Resume by reading this file first → find the first ⬜/🟡 row → that's the next
> architecture document. One document per session, checkpointed here. Built on Opus.

---

## 0. Inputs

- **Requirements catalog:** `../00-MASTER-INDEX.md` (18 volumes, ~1,150 reqs) — the WHAT.
  This phase defines the HOW.
- **Build plan / OQ triage:** `../BUILD-PLAN-AND-OQ-TRIAGE.md` — phases, locked
  decisions (D1–D10), triaged Open Questions, pre-deploy blockers.
- **Existing tech-stack decision:** `../architecture-suite/TECH-STACK.md` (2026-06-23) —
  to be re-validated, not blindly inherited.

---

## 1. The Sequence (user's explicit order)

```
A. Platform/OS architecture (vision, quality attributes, logical & physical design)
        ↓
B. Technology choices (each as a Trade-off ADR — options, scoring, decision, why)
        ↓
C. Detailed implementation plan (epics, milestones, sequencing)
        ↓
D. ESIP-on-EKOS migration architecture (how the existing app becomes an EKOS app)
        ↓
E. 15-year sustainability (dependency currency, automated upgrades, deprecation,
   tech-radar, longevity guarantees)
```

---

## 2. Architecture Document Plan & Status

Status: ⬜ Not started · 🟡 In progress · ✅ Done

| # | Document | Maps to | Status |
|---|---|---|---|
| A00 | This index + Architecture Principles | — | ✅ |
| A01 | Architecture Vision, Drivers & Quality Attributes (the "-ilities", incl. 15-yr longevity as a top driver) | A | ✅ (64 items) |
| A02 | Logical Architecture (bounded contexts, layers, hexagonal ports map, C4 Context+Container) | A | ✅ (70 items; incl. complete volume→arch coverage map) |
| A03 | Technology Selection — ADR set (graph DB, event store, backend, frontend, rendering, AI, collab, infra — each a scored Trade-off ADR) | B | ✅ (66 items; ADR-019…031; 15-yr/no-regret lens) |
| A04 | Data, Storage & Ontology Architecture (polyglot CQRS, bitemporal, federation) | A | ✅ (62 items; ADR-032/033) |
| A05 | AI/ML Architecture (4-layer engine, sovereign/air-gapped, model-ops) | A | ✅ (60 items; ADR-034/035) |
| A06 | Security, Identity & Trust Architecture (ABAC PDP/PIP/PAP, zero-trust, crypto, key mgmt) | A | ✅ (72 items; ADR-036/037/038) |
| A07 | Integration & API Architecture (ports/adapters, connectors, public API, events) | A | ✅ (60 items; ADR-039/040) |
| A08 | Deployment & Infrastructure Architecture (K8s/Helm/OpenTofu, multi-cloud, air-gap, multi-region, tenancy) | A | ✅ (60 items; ADR-041/042/043) |
| A09 | **15-Year Sustainability & Evolution Architecture** (dependency-currency policy, automated upgrades, deprecation, tech-radar cadence, longevity guarantees) | E | ✅ (66 items; ADR-044/045/046; the 10 guarantees) |
| A10 | ESIP-on-EKOS Migration Architecture (the existing app as an EKOS application; strangler-fig path) | D | ✅ (58 items; ADR-047/048/049; 7-step M0–M7) |
| A11 | Detailed Implementation Plan & Roadmap (epics, milestones, team-less solo sequencing, definition-of-done) | C | ✅ (70 items; 16 epics, M0–M12, critical path) |

---

## 🏁 ARCHITECTURE PHASE COMPLETE (2026-07-01)

All 12 docs (A00–A11) done. EKOS has WHAT (27-vol catalog + Books 1–2), HOW (A01–A11), and
WHY (Book 1 Manifesto + ADR Register with 49 decisions + Design Methodology). Every choice
is **15-year / no-regret defended**; every decision **logged as an ADR**; the plan is
**solo-realistic, thesis-first**. **Specification + architecture are implementation-ready.**

**NEXT (not a doc): Phase 1 implementation** — E0/M0 rotate ai-proxy keys (user action) →
E1/M1 storage port → E4/M3 H3 convergence → E3/M2 widgets→projections → E5/M4 ambient crux.
See `A11 §13`.

> A03 deliberately uses **EKOS's own Trade-off methodology** (the catalog's Solution-
> Design/Trade-off widget thinking) to pick technology — eating our own dog food.

---

## 3. Architecture Principles (locked; refined in A01)

1. **Platform before application** — the OS is architected first; apps (incl. ESIP) run on it.
2. **Ports-and-adapters / hexagonal everywhere** — every external dependency swappable.
3. **15-year longevity is a top-tier quality attribute**, not an afterthought — designed
   for dependency currency and continuous modernization.
4. **The semantic graph is the single source of truth**; everything else is a client/projection.
5. **Cryptographic hardness, not obscurity** (Constitution §2); **no training on customer content** (§6).
6. **Air-gap is first-class**; only internet-only features are excluded, each with a local equivalent.
7. **Evolve behind seams, not rewrite** — but the *platform foundation* is built deliberately, not bolted onto the web app.
8. **Capability-vs-feature** (the EKOS Golden Rule) governs every architectural addition.
9. **Standards over proprietary** where longevity depends on it (openCypher/GQL, JSON-LD/RDF, OCI, W3C VC/DID, CloudEvents).

---

## 4. Resume Protocol

1. Open this file → first ⬜/🟡 row in §2 = next document.
2. Pull relevant requirements from `../NN-*.md` + decisions from `../BUILD-PLAN-AND-OQ-TRIAGE.md`.
3. Write `architecture/ANN-<slug>.md` → flip status to ✅ here.
4. STOP. Ask before the next.

_Created 2026-07-01 (Opus). The platform-first architecture phase for a 15-year EKOS._
