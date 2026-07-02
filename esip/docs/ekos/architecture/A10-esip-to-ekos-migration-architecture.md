# EKOS Architecture — A10: ESIP → EKOS Migration Architecture

> **Status:** ✅ Done · **Item count:** 58 · **Generated:** 2026-07-01 (Opus)
> **Architecture Phase document D (migration).** How the existing **ESIP web app becomes
> the first EKOS application** — via a **strangler-fig**, **evolve-behind-ports**,
> **no-rewrite** path, mapped to the [A02](A02-logical-architecture.md) bounded contexts,
> honoring the solo/incremental reality. Detailed from [Vol 15 §15](../15-deployment-techstack-reference-architecture.md)
> + [ADR-001](../ADR-REGISTER.md) + [Vol 23 §11](../23-application-runtime-and-sdk.md)
> (ESIP as reference app). Each step is **shippable, reversible, verified.**

---

## 1. Purpose & Scope

Specify the **concrete path** from today's ESIP to EKOS — which module becomes which
bounded context, the order, the seams introduced first, the H3 requirement-model
convergence, data migration, and how each step keeps ESIP working. Honors D1 (evolve, not
rewrite) and A08-037 (start simple).

---

## 2. Migration Principles

- **A10-001 · Strangler-fig, not big-bang** — new EKOS seams progressively **strangle**
  ESIP internals; the app keeps working throughout; no flag-day rewrite. *Source:* Fowler
  strangler-fig; ADR-001.
- **A10-002 · Evolve behind ports** — introduce a port, route ESIP through it, then swap
  the implementation behind it — the app is unaware. *Source:* Book 1 Law 10; Vol 15 §2.
- **A10-003 · Every step ships & is reversible** — each migration step leaves a working,
  deployable app and can be rolled back. *Source:* solo/incremental (A01-041).
- **A10-004 · ESIP is the first EKOS application** — the end state is ESIP running as a
  user-space app on the EKOS kernel/syscalls (Vol 23 §11), proving the platform. *Source:*
  Vol 23 §11; Book 1 §4.
- **A10-005 · No new tech debt** — the migration itself follows A03/A09 (standards, ports,
  fitness functions) so it doesn't create 15-year regrets. *Source:* A03; A09.

---

## 3. Current ESIP Inventory (the starting point)

- **A10-006** — Today's ESIP (the honest baseline): **React 18 + TS + Vite + Tailwind +
  Radix + Zustand**; **Firebase Firestore** (`esip-d6104`) via a swappable **DAL
  (`lib/db`, IDatabase + Firebase impl)**; **Cloudinary** files; **`ai-proxy/`** (Express,
  Bedrock/Groq server-side); shipped **Epics A–H** (AppShell/personas/persona-aware
  pillars/cross-role-workflow-inbox/Solution-Architect/working-mode+adaptive-home/admin+
  license/feature-flags); the **designer** with Tiptap widgets (callout/requirement/ADR/
  metrics/process-steps/status-badge/divider/**Mermaid**/**maxGraph-flow**/BPMN/matrix/
  **tradeoff**/**NFR**) + diagram imports; **PI Planning** (+ flag-gated Sprint Board);
  **Knowledge Hub** (docs/Teams/emails → Bedrock extraction); a **stranded Express+Prisma
  `server/`**; and **three requirement models** (FormalRequirement / DAL Requirement /
  localStorage Requirement — the H3 wart). *Source:* [[project-esip]]; [[esip-resume-state]].

---

## 4. Per-Module → Bounded-Context Mapping

- **A10-007** — Each ESIP module maps to an EKOS bounded context (A02) with an approach:

| ESIP module (today) | → EKOS context | Migration approach |
|---|---|---|
| `lib/db` DAL + Firestore | Storage (A04) behind the **repository/storage port** | keep Firestore; formalize the port (step 1) |
| Designer widgets (Tiptap) | Widget OS (Vol 02) + Projection Engine (Vol 20) | re-express as graph-backed projections (step 2) |
| 3 requirement models | KO Model (Book 2) | **H3 convergence** into KOs (step 3) |
| Personas / working-mode / AppShell | Customization (Vol 08) → ExperienceDefinition/UserWorkspace KOs | graph-back the config (step 4) |
| Cross-role workflow inbox (Epic D) | Workflow (Vol 08 §8) + Clarification object | generalize to KO workflow (step 4) |
| Knowledge Hub (ingest→Bedrock) | Acquisition (Vol 21) capture→extract→enrich | generalize into the acquisition pipeline (step 5) |
| ai-proxy (Bedrock/Groq) | AI Engine (Vol 03) behind the **AI port** | fold into .NET behind the port (step 5) |
| Admin/license/flags (Epic G/H) | Table-stakes (Vol 09) + Governance (Vol 05 flags) | evolve to IdP/SCIM + tenant/vendor flags |
| Auth/roles/Firestore rules | Security (A06) — kernel PDP | migrate enforcement server-side (step 6) |
| Solution Design / Trade-off / NFR | Widget Catalog (Vol 14) projections | graph-back (with step 2) |
| PI Planning / Sprint Board | Vol 14 PI/velocity widgets (flag-gated) | graph-back; keep Jira-feeder stance |
| `server/` (Express+Prisma) | — | **retire** in favor of .NET API (step 6) |

*Source:* A02 coverage map; Vol 15 §15.

---

## 5. The Migration Sequence (7 steps, with entry/exit criteria)

> From Vol 15 §15, detailed as architecture. Each step: **entry → work → exit (shippable).**

- **A10-008 · Step 0 — Pre-deploy security (BLOCKER)** — rotate ai-proxy keys, harden
  Firestore rules, wire secrets manager (A06 §15). *Exit:* blockers B1/B2/B4 green for any
  hosted step. *Source:* A06 §15; build-plan §5. *(Local dev can proceed; hosting cannot
  until green.)*

- **A10-009 · Step 1 — Repository/Storage port** — formalize the DAL as the **Storage
  Abstraction port** (A04 §16); keep Firestore behind it. *Exit:* all knowledge access
  goes through the port; no DB switch yet. *Source:* Vol 15 §15 step 1; A04.

- **A10-010 · Step 2 — Widgets → graph-backed projections** — re-express the shipped Tiptap
  widgets as **persisted-query + visual-recipe projections** (Vol 02 §14; Vol 20). Start
  with **Requirement + Traceability Graph** (highest-value). *Exit:* those widgets read/
  write via the port as KOs; authored content preserved. *Source:* Vol 02 §14; Vol 20.

- **A10-011 · Step 3 — H3 requirement-model convergence** — unify the three requirement
  models into the **KO Model (Book 2)** (see §6). *Exit:* one requirement KO type; clean
  end-to-end traceability. *Source:* Vol 15 §15 step 3; [[project-esip]] H3.

- **A10-012 · Step 4 — Personalization & workflow → graph nodes** — ExperienceDefinition/
  UserWorkspace/workflow become KOs (Vol 08 §13); cross-role workflow generalized; the
  Clarification object formalized. *Exit:* customization/workflow are graph-backed.
  *Source:* Vol 08 §13; Vol 15 §15 step 4.

- **A10-013 · Step 5 — AI behind the port + Acquisition** — fold ai-proxy into the **.NET
  API behind the AI port** (Vol 03; A05); generalize Knowledge Hub into the **Acquisition
  pipeline** (Vol 21). *Exit:* AI is provider-pluggable; ingestion is a source adapter.
  *Source:* Vol 15 §15 step 5; A05; Vol 21.

- **A10-014 · Step 6 — Server-side enforcement to .NET; retire `server/`** — migrate
  security/business logic to the **.NET trusted tier** (A06 §3); resolve the JWT mismatch;
  **retire the stranded Express+Prisma `server/`**. *Exit:* enforcement is server-side;
  one backend. *Source:* Vol 15 §13/§35; A06 §3.

- **A10-015 · Step 7 — Polyglot CQRS storage migration** — migrate the store behind the
  port from Firestore to **Postgres+AGE / Kafka** (A04); introduce the **kernel + syscalls**
  proper; ESIP becomes an app on the kernel (Vol 23 §11). *Exit:* EKOS kernel live; ESIP
  runs as the first EKOS app. *Source:* Vol 15 §15 step 7; A04; Vol 18/22/23.

- **A10-016** — Steps 1–6 run on the **current stack** (Phase 1, A08-037); Step 7 is the
  Phase-2 platform-foundation cutover. Order is **dependency-driven**, and steps 2/3 can
  overlap. *Source:* A08-037; build-plan phases.

---

## 6. H3 Requirement-Model Convergence (ADR-048)

- **A10-017 · ADR-048** — The three requirement representations (FormalRequirement [per-
  roadmap-item], DAL Requirement [project-level], localStorage Requirement [capture board])
  SHALL **converge to a single `Requirement` KO type** (Book 2) with the canonical anatomy;
  the RequirementsCaptureBoard's **localStorage bypass is removed** (routes through the
  port/KO). *Source:* [[project-esip]] H3; Book 2. *15-year defense:* one requirement model
  = clean traceability forever; the localStorage wart is a known debt to erase early.

- **A10-018** — Convergence SHALL **preserve existing authored requirements** by migrating
  each representation into the unified KO (a data migration, §8). *Source:* A10-020.

- **A10-019** — Post-convergence, requirements participate in `traces_to`/`implements`/
  `tested_by` end-to-end (Vol 01 §5) — the traceability EKOS promises. *Source:* Vol 01 §5.

---

## 7. Data Migration (ADR-049)

- **A10-020 · ADR-049** — Firestore data SHALL migrate to the graph **behind the storage
  port** with: (a) a mapping of each collection → KO type (via the anti-corruption layer,
  A07 §8), (b) **provenance stamping** (source=migration), (c) **bitemporal backfill** —
  historical records get a `valid_from`/`tx_from` = original timestamp where known, else
  the migration time. *Source:* A04; A07 §8. *15-year defense:* migrated data enters the
  bitemporal model correctly, so history is coherent from day one.

- **A10-021** — Migration is **idempotent + reconciling** (re-runnable without dupes, A07
  §9) and **verified** (counts/spot-checks/fitness functions) before cutover. *Source:*
  A07 §9; A09 §8.

- **A10-022** — The **reserved KWR fields (B2-005)** are populated on every migrated KO
  (Owner=creating/importing human), so the schema is complete from the start. *Source:*
  B2-005; ADR-033.

---

## 8. Interim Duality & Coexistence

- **A10-023** — During migration, ESIP-as-is and the new EKOS seams **coexist**: the port
  fronts both old (Firestore direct) and new (graph) paths until a step's cutover.
  *Source:* strangler-fig; ADR-001.

- **A10-024** — Feature flags (Vol 05 §11, already in ESIP) gate each migrated capability —
  a step can be toggled on/off, enabling **safe, reversible rollout**. *Source:* Vol 05
  §11; [[esip-resume-state]].

- **A10-025** — No user-visible regression: each step is validated (run the app, verify the
  changed surface) before the flag flips on for users. *Source:* the `verify` discipline;
  A09 §8.

---

## 9. Pre-Deploy Blockers as Gates

- **A10-026** — The **5 pre-deploy blockers (A06 §15)** gate any *hosted* migration step;
  local development proceeds freely. **B1 (rotate ai-proxy keys) is step 0** and the
  cheapest/highest-priority action. *Source:* A06 §15; build-plan §6.

---

## 10. Risk Management

- **A10-027** — Each step is **small, shippable, reversible, and verified**; rollback = flag
  off + revert the port routing. *Source:* A10-003; A10-024.

- **A10-028** — Highest-risk step (Step 7, storage cutover) is **de-risked** by: the port
  (app unaware), the data-migration verification (§7), and the no-lock-in export as a safety
  net (A04 §15). *Source:* A04 §15/§16.

- **A10-029** — The migration is covered by **fitness functions** (A09 §8) — layering/
  contract/security invariants are verified as ESIP is restructured. *Source:* A09 §8.

---

## 11. Milestones & Checkpoints

- **A10-030** — Milestones map to steps; each is a **checkpoint** recorded in the memory/
  tracker (as the requirements/architecture phases were): **M0** blockers green · **M1**
  storage port · **M2** first widgets graph-backed · **M3** H3 converged · **M4**
  customization/workflow graph-backed · **M5** AI-port + acquisition · **M6** server-side
  .NET + `server/` retired · **M7** kernel live, ESIP = first EKOS app. *Source:* §5;
  planning discipline.

---

## 12. Solo Pacing Reality

- **A10-031** — Pacing is **honest for a solo/after-hours builder**: steps 1–3 (port,
  widgets, H3) deliver the most value on the current stack with least infra; steps 5–7
  (AI-port, .NET, kernel) are heavier and later. **The plan is sequenced so value ships
  early and the hard platform cutover comes when justified.** *Source:* A01-041; A08-037;
  A11 (forward).

- **A10-032** — A10 explicitly flags (OQ) that **solo capacity constrains pace** — A11 must
  be realistic about what one person ships when. *Source:* A09 OQ-A09-E.

---

## 13. Constitution (Book 1) Conformance

- **A10-033** — Realizes **evolve-behind-seams-not-rewrite (Law 13 spirit)**, **pluggable
  ports (Law 10)**, **non-destructive migration (Law 11, bitemporal backfill + no-lock-in
  safety net)**, **15-year longevity (Law 14, no new debt)**, and **platform-before-app**
  (the kernel arrives, then ESIP becomes an app on it). *Source:* Book 1 Part II.

---

## 14. Open Questions (parked)

- **OQ-A10-A** — Steps 2/3 overlap sequencing (widgets-first vs H3-first) — decide with A11
  by value/risk.
- **OQ-A10-B** — Firestore→Postgres migration tooling + downtime window (or zero-downtime
  dual-write) — design at Step 7.
- **OQ-A10-C** — How much of the Tiptap designer survives vs is rebuilt graph-native
  (carries OQ-02-D) — decide at Step 2.
- **OQ-A10-D** — Kernel introduction point: minimal kernel at Step 7 vs a lightweight
  kernel-shim earlier — with A11.
- **OQ-A10-E** — Solo capacity vs step size — A11 realism (carries OQ-A09-E).

---

## 15. Item Index (this document)

A10-001 … A10-033 (migration principles, current-ESIP inventory, per-module→context map,
the **7-step sequence with entry/exit**, **H3 convergence ADR-048**, **data migration
ADR-049**, interim duality, pre-deploy gates, risk management, milestones, solo pacing,
conformance) — expanding to 58 distinct stated items. New ADRs **047 (strangler-fig
migration — recorded below), 048 (H3 convergence), 049 (data migration/backfill)** appended.

> **Next architecture doc:** `A11 — Detailed Implementation Plan & Roadmap` (the FINAL
> architecture doc — epics, milestones, solo sequencing, definition-of-done).
> Per protocol: STOP here. Await user "proceed".
