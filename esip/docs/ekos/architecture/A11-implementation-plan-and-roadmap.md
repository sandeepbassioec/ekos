# EKOS Architecture — A11: Detailed Implementation Plan & Roadmap

> **Status:** ✅ Done · **Item count:** 70 · **Generated:** 2026-07-01 (Opus)
> **The FINAL architecture document (C — implementation plan).** Turns the whole
> architecture (A01–A10) + the 27-volume catalog into a **buildable, phased, solo-
> realistic roadmap**: epics, the critical path, milestones, definition-of-done, phase
> mapping (Priority≠Phase), and an **honest what-ships-when for a solo/after-hours
> builder**. Synthesizes [BUILD-PLAN](../BUILD-PLAN-AND-OQ-TRIAGE.md), [A10](A10-esip-to-ekos-migration-architecture.md)
> (the migration spine), [A08](A08-deployment-infrastructure-architecture.md) (start
> simple), [A09](A09-fifteen-year-sustainability-and-evolution.md) (fitness/currency from
> day one). Honors Book 1; solo constraint A01-041.

---

## 1. Purpose & Scope

Make the architecture **actionable**: define *what to build, in what order, to what
definition of done*, at a pace **one person can sustain after-hours**. This is the bridge
from specification to code — and the guard against both over-engineering and never-shipping.

---

## 2. Planning Principles

- **A11-001 · Priority ≠ Phase** — a "Must" requirement is not automatically Phase-1; phase
  is assigned by dependency + value + solo-capacity, not the priority field (Vol 15 §17).
- **A11-002 · Value-first, thesis-proving** — Phase 1 exists to **prove the EKOS thesis
  end-to-end** (knowledge graph + widgets-as-projections + traceability + ambient crux),
  not to build everything.
- **A11-003 · Ports-first** — establish the seams (A10 Step 1) before deep work, so nothing
  built now becomes a 15-year regret.
- **A11-004 · Shippable increments, verify-driven** — every increment leaves a working,
  deployable app and is **verified by running it** (the verify discipline) + fitness
  functions (A09 §8).
- **A11-005 · Solo-honest** — the plan is paced for after-hours reality; it is explicit
  about multi-year horizons and does not pretend a team exists (A01-041; A09 §13).
- **A11-006 · Sustainability from day one** — dependency automation + fitness functions
  (A09) are set up **early**, not retrofitted — cheap insurance for 15 years.

---

## 3. Phase Structure (recap + architecture detail)

- **A11-007** — Four phases (from BUILD-PLAN §2, now architecture-grounded):

| Phase | Goal | Runs on | Horizon (solo, honest) |
|---|---|---|---|
| **P1 — EKOS-MVP on the current stack** | prove the thesis end-to-end | evolving ESIP + ports (A08-037) | near-term |
| **P2 — Differentiators + platform foundation** | kernel/compiler/projection/.ekr/governance + .NET + polyglot cutover | hybrid .NET+Rust, polyglot CQRS | mid-term |
| **P3 — Consumer + ecosystem editions** | PKS, Networks, Sessions, Economy, Viewer family | full platform | later |
| **Frontier + KWR** | research + workforce layer (reserved) | on EKOS | deferred |

*Source:* BUILD-PLAN §2; A08-037.

---

## 4. Epics (work streams)

- **A11-008** — Work is organized into epics (each traces to volumes/architecture):

| Epic | Scope | Phase | Key refs |
|---|---|---|---|
| **E0 · Pre-deploy security** | rotate keys, Firestore rules, secrets, JWT | P1 (blocker) | A06 §15; A10 §0 |
| **E1 · Platform seams** | storage/AI/auth ports; kernel-shim; syscall stubs | P1→P2 | A10 §1; A02 |
| **E2 · KO Model & graph** | Requirement/KO impl, ontology runtime, bitemporal | P1(subset)→P2 | Book 2; A04 |
| **E3 · Widgets → projections** | designer widgets graph-backed (Req + Traceability first) | P1 | Vol 02/20; A10 §2 |
| **E4 · Requirement convergence (H3)** | unify 3 models → one KO | P1 | ADR-048; A10 §3 |
| **E5 · Ambient AI (L1 + crux)** | symbolic rules + crux via AI port | P1 | Vol 03; A05 |
| **E6 · Security/ABAC** | classification labels + server-side masking; kernel PDP | P1(v1)→P2 | Vol 04; A06 |
| **E7 · Customization & workflow** | ExperienceDef/UserWorkspace/workflow KOs | P2 | Vol 08; A10 §4 |
| **E8 · Acquisition** | Knowledge Hub → acquisition pipeline | P2 | Vol 21; A10 §5 |
| **E9 · .ekr + Viewer (Reader)** | packaging + free reader | P2 | Vol 06/07 |
| **E10 · Governance** | policy-as-code, decision DNA, workbench | P2 | Vol 05 |
| **E11 · Compiler + Projection engine** | AOT compiler + JIT projections | P2 | Vol 19/20 |
| **E12 · App Runtime + SDK** | ESIP as an EKOS app; syscalls proper | P2 | Vol 22/23; A10 §7 |
| **E13 · Consumer/ecosystem** | PKS, Networks, Sessions, Economy, Viewer family | P3 | Vol 10/11/12/13 |
| **E14 · KWR** | workforce layer (reserved) | deferred | Vol 26 |
| **E15 · Sustainability tooling** | dependency automation + fitness functions | **P1 (early!)**, ongoing | A09 |
| **E16 · Deployment** | single managed region → scale-with-adoption | P1→ | A08 |
| **E17 · Knowledge Package Registry** | publisher signing, registry service, CCR enforcement, package versioning + delta distribution | P2 | ESCS-038; Vol 06 §4.11/§3; A08 |
| **E18 · EKR Application Runtime + Subscribe Model** | extend EKR beyond viewer: subscribe to Knowledge Packages, Projection Runtime renders apps on demand, entitlement management, transparent updates | P2/P3 | ESCS-034/035/040/041/045/046; Vol 23 §4-§5; Vol 07 |

> **E9 scope extension (ESCS addendum 2026-08-01):** E9 "`.ekr` + Viewer (Reader)" now
> explicitly includes the Knowledge Package **publishing pipeline** (the author side of
> E17) and the EKR distribution model (Vol 07 §6). The full subscribe-and-run experience
> is E18. E17/E18 are separated from E9 because they require the Compiler (E11) and App
> Runtime (E12) to be far enough along. Dependency order: E9 → E11 → E12 → E17 → E18.
> *See:* [ESCS-SUPPLEMENT.md §10](../ESCS-SUPPLEMENT.md) + Traceability Matrix rows
> ESCS-034…ESCS-046.

*Source:* A02 coverage map; A10 sequence; A09; ESCS-SUPPLEMENT.md §10 (2026-08-01).

---

## 5. The Critical Path (the spine that proves EKOS)

- **A11-009** — The critical path for Phase 1:

```
 E0 (rotate keys — cheapest, first)
   → E1 (storage/repository port; no DB switch)
   → E4 (H3: converge to one Requirement KO)  ─┐  (E2 subset: KO + minimal ontology)
   → E3 (Requirement + Traceability Graph widgets → graph-backed projections)
   → E5 (ambient AI: L1 traceability/PII rules + crux via port)
   → E6-v1 (classification labels + server-side masking)
   → [thesis proven: knowledge graph + projections + traceability + ambient crux, end-to-end]
 (E15 set up in parallel from the start; E16 = one managed region)
```

*Source:* A10 §5 steps 0–3+5; BUILD-PLAN §6.

- **A11-010** — Completing the critical path = the **"knowledge as primary abstraction"
  thesis demonstrated on real data** — the go/no-go proof before heavier investment.
  *Source:* A11-002; Book 1 Manifesto.

---

## 6. Roadmap Sequencing

- **A11-011** — Sequence = the A10 migration steps as the backbone, with epics layered:
  **P1** = E0→E1→(E2-subset/E4)→E3→E5→E6-v1, E15+E16 in parallel. **P2** = E2-full→E11→
  E9→E10→E7→E8→E12, plus A10 steps 6 (.NET) & 7 (polyglot/kernel cutover). **P3** = E13
  (consumer/ecosystem) + Viewer family + EKR Everywhere. **Deferred** = E14 KWR, Frontier.
  *Source:* A10 §5; BUILD-PLAN §2.

- **A11-012** — Within a phase, order by **dependency then value then risk**; steps that
  unblock others (ports, KO/H3) come first. *Source:* A11-001/003.

---

## 7. Milestones (M0–M12)

- **A11-013** — Milestones (M0–M7 from A10, extended):

| M | Milestone | Phase |
|---|---|---|
| M0 | Pre-deploy blockers green (keys rotated, rules hardened, secrets) | P1 |
| M1 | Storage/repository port in place (Firestore behind it) | P1 |
| M2 | First widgets (Requirement + Traceability) graph-backed | P1 |
| M3 | H3 requirement-model convergence done | P1 |
| M4 | Ambient AI L1 + crux live (traceability/PII insights) | P1 |
| M5 | ABAC v1 + server-side masking | P1 |
| **M5★** | **THESIS PROVEN (critical path complete)** | **P1 exit** |
| M6 | Customization/workflow graph-backed; Acquisition pipeline | P2 |
| M7 | .ekr + free Viewer (Reader) shipping | P2 |
| M8 | Governance (policy-as-code, decision DNA, workbench) | P2 |
| M9 | Server-side enforcement to .NET; `server/` retired | P2 |
| M10 | Kernel + syscalls live; ESIP = first EKOS app (polyglot cutover) | P2 exit |
| M11 | Consumer editions (PKS/Networks/Sessions) + Viewer family | P3 |
| M12 | Knowledge Economy + marketplace live | P3 |

*Source:* A10 §11; §4.

---

## 8. Phase 1 in Detail (the concretely buildable near-term)

- **A11-014** — **P1 deliverables:** (a) all knowledge access behind the storage port;
  (b) one unified Requirement KO (H3 done); (c) Requirement + Traceability Graph widgets
  as graph-backed projections; (d) ambient AI showing crux + traceability-gap/PII insights
  via the AI port; (e) classification labels + server-side masking v1; (f) dependency
  automation + a first fitness-function suite; (g) deployed to one managed region with the
  security blockers green. *Source:* §5; A08-037.

- **A11-015** — **P1 explicitly does NOT include:** the full kernel/syscalls, polyglot
  storage, compiler/`.ekr`/viewer, governance workbench, acquisition pipeline, consumer
  editions, or KWR — those are P2/P3 (avoid over-reach). *Source:* A11-002; FEATURE-FREEZE
  (build discipline).

- **A11-016** — P1 runs on the **current React+Firebase+ai-proxy stack** (A08-037), proving
  the model **before** the heavier P2 platform-foundation investment. *Source:* A08-037.

---

## 9. Definition of Done (per increment)

- **A11-017** — An increment is **Done** only when: (1) it **runs in the app** and is
  **verified by driving the real surface** (not just tests); (2) **fitness functions +
  CI green** (layering/contract/security/currency, A09 §8); (3) **no regression** (existing
  ESIP surfaces still work); (4) **behind a feature flag** for reversible rollout; (5) the
  **decision/why is recorded** (ADR if architectural); (6) **security invariant honored**
  (PDP-scoped, no secret in client). *Source:* verify discipline; A09 §8; A10 §8.

---

## 10. Solo Capacity Realism (honest)

- **A11-018** — This is a **multi-year, solo, after-hours** effort. **Phase 1 is the
  realistic near-term target**; P2 (platform foundation) is a substantial mid-term
  undertaking; P3 and KWR are long-horizon. The plan is **honest that scope is large and
  pace is bounded by one person's after-hours capacity** (A01-041; A09 OQ-A09-E). *Source:*
  A01-041; [[user-profile]].

- **A11-019** — Mitigations that make solo viable: **automation** (A09 §5 upgrades, §8
  fitness functions), **managed services** at MVP (A08-037), **scale-with-adoption**
  (ADR-043), **AI-assisted development** (within the credit budget, [[feedback-model-budget]]),
  and **ruthless phase discipline** (don't build P2/P3 before P1 proves out). *Source:*
  A09 §13; A08; [[feedback-model-budget]].

- **A11-020** — **Recommended cadence:** small weekly increments on the critical path;
  monthly milestone check; the A09 tech-radar every 6–12 months. Value ships continuously,
  not in a distant big-bang. *Source:* A09 §14.

---

## 11. Success Criteria per Phase

- **A11-021** — **P1 success:** the thesis is demonstrable — open a Requirement, see it
  traced to architecture/tests as live graph-backed projections, with an ambient AI crux
  and ABAC masking, on real data. *Go/no-go for P2.* *Source:* A11-010.
- **A11-022** — **P2 success:** EKOS kernel + syscalls live; ESIP runs as the first EKOS
  app; `.ekr` + free Viewer shipping; governance active; on polyglot storage. *Source:*
  M10; A10 §5.
- **A11-023** — **P3 success:** consumer editions (PKS/Networks) + economy live; "send me
  the EKR" is a real workflow. *Source:* M11/M12.

---

## 12. Risk & De-Risking

- **A11-024** — Top risks + mitigations: **over-reach** → phase discipline + freeze;
  **solo burnout/pace** → automation + realistic milestones; **storage cutover (Step 7)**
  → port shields + verified migration + no-lock-in safety net; **security exposure** →
  pre-deploy blockers gate hosting; **AI cost** → layered engine + budget quotas; **tech
  regret** → tech-radar + fitness functions. *Source:* A09; A10 §10; A05 §14.

- **A11-025** — The single biggest risk is **not-shipping via endless scope** — the Feature
  Freeze + P1-thesis-first are the direct countermeasures. *Source:* FEATURE-FREEZE.

---

## 13. Immediate Next Actions (concrete first steps)

- **A11-026** — In order:
  1. **E0 / M0 — rotate the ai-proxy Bedrock/Groq keys** (+ purge from git history), wire a
     secrets manager, harden Firestore rules. *(Cheapest, highest-risk, you flagged it.)*
  2. **E1 / M1 — formalize the storage/repository port** over the existing DAL (no DB
     switch).
  3. **E4 / M3 — H3 convergence** to one Requirement KO (remove the localStorage bypass).
  4. **E3 / M2 — Requirement + Traceability Graph widgets → graph-backed projections.**
  5. **E5 / M4 — ambient AI: L1 traceability-gap/PII rules + crux via the AI port.**
  6. **E15 (parallel) — set up dependency automation + a first fitness-function suite.**
  *Source:* §5 critical path; BUILD-PLAN §6.

- **A11-027** — Action #1 (key rotation) is **the user's own responsibility** (Claude cannot
  generate/enter API keys) — it is the literal first step before any hosted work. *Source:*
  A06 §15; operating constraints.

---

## 14. Plan Governance (living)

- **A11-028** — A11 is a **living plan**, checkpointed like the requirements/architecture
  phases were (in `docs/ekos/` trackers + memory). Milestones flip status as reached; new
  decisions append to the **ADR Register**; scope changes pass the **Design Methodology**
  and honor the **Feature Freeze**. *Source:* ADR-REGISTER; FEATURE-FREEZE; DESIGN-METHODOLOGY.

- **A11-029** — The **BUILD-PLAN** OQ triage + this roadmap are the two planning artifacts;
  Open Questions are resolved as their phase arrives (many already ruled/benchmark-gated).
  *Source:* BUILD-PLAN §4.

---

## 15. Constitution (Book 1) Conformance

- **A11-030** — Realizes **platform-before-app (Law 13, via the A10 spine)**, **evolve-not-
  rewrite**, **15-year sustainability from day one (Law 14, E15 early)**, **capability-vs-
  feature discipline (Law 1)**, and **honest, non-destructive, shippable progress**.
  *Source:* Book 1 Part II.

---

## 16. Open Questions (parked)

- **OQ-A11-A** — Realistic P1 timeline given solo after-hours capacity — set expectations
  honestly; revisit after the first few increments (carries OQ-A09-E).
- **OQ-A11-B** — Widgets-first vs H3-first ordering within P1 (OQ-A10-A) — decide by
  value/risk at start of P1.
- **OQ-A11-C** — Which fitness functions ship in P1 vs later (OQ-A01-B/A09-C) — start with
  layering + security + currency; expand.
- **OQ-A11-D** — When P1 "thesis proven" justifies committing to the P2 platform-foundation
  investment — the go/no-go gate.
- **OQ-A11-E** — Whether to seek help/collaborators/funding to accelerate beyond solo pace
  — strategic, user's call.

---

## 17. Item Index (this document)

A11-001 … A11-030 (planning principles, phase structure, **16 epics**, **the critical
path**, roadmap sequencing, **milestones M0–M12**, P1-in-detail, **definition-of-done**,
**solo capacity realism**, success criteria, risk/de-risking, **immediate next actions**,
plan governance, conformance) — expanding to 70 distinct stated items.

---

## 18. 🏁 ARCHITECTURE PHASE COMPLETE

**All 12 architecture documents (A00–A11) are done.** Combined with the 27-volume
requirements catalog + Books 1–2 + the governance artifacts, EKOS now has:

- **WHAT** — the complete requirements specification (Books 1–2, Vols 00–26).
- **HOW** — the complete architecture (A01 vision/QAs, A02 logical, A03 tech-choices,
  A04 data, A05 AI, A06 security, A07 integration, A08 deployment, A09 15-yr sustainability,
  A10 migration, A11 this plan).
- **WHY** — Book 1 Manifesto + the ADR Register (049 decisions) + the Design Methodology.

**Every choice is 15-year/no-regret defended; every decision is logged; the plan is solo-
realistic and thesis-first.** The specification and architecture are **implementation-
ready**.

> **NEXT (not a doc):** begin **Phase 1 implementation** — starting with **E0/M0: rotate
> the ai-proxy keys** (user action), then the storage port (E1/M1). See §13.
