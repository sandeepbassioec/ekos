# EKOS Volume 26 — Knowledge Workforce Runtime (KWR)

> **Status:** ✅ Done · **Requirement count:** 56 · **Generated:** 2026-07-01 (Opus)
> **Gap-closure #9** (cross-architect review). **KWR is NOT part of EKOS — it runs ON
> TOP of EKOS**, exactly as GitHub Actions runs on Linux and .NET runs on Windows. It is
> the **role-based orchestration of knowledge work** performed by **humans + AI +
> automation + bots — unified** — over EKOS knowledge objects. It is a **separate product
> and a separate discipline** (Knowledge Workforce Engineering), **Phase-deferred** but
> **architecturally reserved now** so EKOS needs no schema change to support it later.
> **Sources:** ChatGPT (the KWR review — "AI Workforce Orchestration → Knowledge Workforce
> Runtime; runs on EKOS not in it; workers manipulate Knowledge Objects not files; the
> workforce is a graph; Team.ekr; sell AI Teams; reserve the architecture; the discipline
> split"); Book 1 §4 (layering) + Law 13; Vol 18 §10 (Principal model = the foundation);
> Vol 08 (cross-role workflow = precursor); Vol 03 (AI analysts = one worker kind); Vol 06
> (`.ekr`), Vol 13 (marketplace). Honors [BOOK-1](BOOK-1-KNOWLEDGE-CONSTITUTION.md).

---

## 1. Scope & Boundary

**In scope:** the **KWR layering** (on EKOS, not in it); **workforce vs agent**; the
**Role model** (humans/AI/automation unified); knowledge-work **primitives** (Assignment,
Ownership, Handoff, Approval, Review, Escalation); the **workforce-as-a-graph**; **workers
manipulate Knowledge Objects not files**; **Knowledge Handoffs replace meetings**;
**Team.ekr** export/import; **selling AI Teams**; the **architectural reservation** EKOS
must honor now; **Phase deferral**; the **discipline split**; and the patent.

**Explicitly deferred / cross-referenced:**
- **EKOS itself** (kernel, knowledge, identity, events, policies, runtime, packages,
  security, marketplace) → **Vols 00–25** (KWR *consumes* EKOS via syscalls; it does not
  reimplement it)
- **The Principal model** (human/AI/connector/app principals) → **Vol 18 §10** (the
  *foundation* KWR Roles build on)
- **Cross-role workflow engine, ClarificationRequest, approval gates** → **Vol 08** (the
  *precursor*; KWR generalizes single-workflow into a full workforce)
- **AI analysts / 4-layer engine** → **Vol 03** (an AI analyst is *one kind of worker*;
  KWR orchestrates many roles)
- **`.ekr` packaging** → **Vol 06** (Team.ekr is an `.ekr`/EKRA variant); **App Runtime &
  SDK** → **Vol 23** (KWR is itself a layer/app-platform on EKOS); **marketplace
  economics** → **Vol 13**

---

## 2. Core Principle — KWR Runs ON EKOS, Not IN It

- **EKOS-26-001 (Should, P3)** — The **Knowledge Workforce Runtime (KWR)** SHALL be a
  **separate layer/product that runs on top of EKOS**, not a subsystem inside it — the
  same separation as **GitHub Actions on Linux** or **.NET on Windows**. EKOS provides
  knowledge/identity/relationships/events/policies/runtime/packages/security/marketplace;
  KWR provides workforce orchestration on those primitives. *Source:* ChatGPT (KWR review
  — "EKOS is the Knowledge OS; KWR is the Knowledge Worker Runtime; those are different");
  Book 1 §4/§Law 13.

- **EKOS-26-002 (Should, P3)** — The layering SHALL be **EKOS → KWR → Applications** —
  exactly like **Windows → .NET → Applications**. *Source:* ChatGPT (KWR review).

- **EKOS-26-003 (Should, P3)** — KWR SHALL reach EKOS **only via the syscall layer**
  (Vol 22) and run as a user-space platform (Vol 23) — it is privileged like any other
  EKOS client, no more. *Source:* Vol 22; Vol 23; Book 1 Law 13.

---

## 3. Workforce, Not Agent

- **EKOS-26-004 (Should, P3)** — KWR SHALL orchestrate a **workforce**, not merely
  **agents**. An agent has a single responsibility ("write code"); a **workforce** is
  multiple specialists with **ownership, responsibilities, approvals, reviews,
  dependencies, and governance** — i.e., an **organization**. *Source:* ChatGPT (KWR
  review — "Everyone talks about agents. Nobody talks about workforces").

- **EKOS-26-005 (Should, P3)** — KWR SHALL be deliberately distinct from existing **agent
  frameworks** (CrewAI / AutoGen / LangGraph / Semantic Kernel / OpenAI Agents) — those
  orchestrate *agents*; KWR orchestrates a *governed knowledge workforce* over a knowledge
  OS. *Source:* ChatGPT (KWR review — the differentiation).

---

## 4. The Role Model (Humans + AI + Automation, Unified)

- **EKOS-26-006 (Should, P3)** — The central abstraction SHALL be the **Role**, not the
  person/agent. A Role defines: **Responsibilities, Capabilities, Inputs, Outputs,
  Policies, Permissions, and Escalation Rules.** One person/AI may hold many roles; many
  may hold one role. *Source:* ChatGPT (KWR review — Role definition; "roles, not people").

- **EKOS-26-007 (Should, P3)** — **Humans, AI, automation, bots, scripts, and future
  autonomous systems SHALL all be represented using the same Role model** — humans are
  **not special**. The runtime cares about *what role is responsible*, not *who/what*
  performs it. *Source:* ChatGPT (KWR review — "Knowledge Workforce ... Humans. AI.
  Automation. Bots ... Same model"). This builds directly on the EKOS **Principal kinds**
  (Vol 18 §10). *Depends:* Vol 18 §10.

- **EKOS-26-008 (Should, P3)** — Roles SHALL map to the engineering-org role set as a
  starting catalog (Product Manager, Architect, Business Analyst, Tech Lead, Developer,
  QA, Security, DevOps, Release, Support) and be **fully extensible** to any domain.
  *Source:* ChatGPT (KWR review — the role chain); Book 1 Law 1.

---

## 5. Knowledge-Work Primitives

- **EKOS-26-009 (Should, P3)** — **Assignment:** a Knowledge Object MAY be assigned to one
  or more Roles. *Source:* ChatGPT (AI Assignment).

- **EKOS-26-010 (Should, P3)** — **Ownership:** **exactly one Role SHALL own a Knowledge
  Object at any point in time** (single-owner invariant). *Source:* ChatGPT (AI Ownership —
  "Exactly one Role SHALL own ... at any point in time. Very important").

- **EKOS-26-011 (Should, P3)** — **Handoff:** Roles SHALL communicate **only through
  Knowledge Objects — never through chat.** Handoffs are knowledge contracts on the edges
  between roles. *Source:* ChatGPT (AI Handoff — "only through Knowledge Objects. Never
  through chat. Huge").

- **EKOS-26-012 (Should, P3)** — **Approval:** Roles MAY require approval before publishing
  mutations. *Source:* ChatGPT (AI Approval).

- **EKOS-26-013 (Should, P3)** — **Review:** every mutation MAY require one or more review
  Roles. *Source:* ChatGPT (AI Review). *Ties:* Vol 05 §7 (AI-output review gate), Vol 08.

- **EKOS-26-014 (Should, P3)** — **Escalation:** if a worker's confidence falls below a
  threshold, the work SHALL **escalate** (to another role / a human). *Source:* ChatGPT
  (AI Escalation); Vol 03 §14 (confidence).

- **EKOS-26-015 (Should, P3)** — **Human participation:** humans SHALL be represented using
  the **same Role model** as AI/automation — no special-case path. *Source:* ChatGPT
  (Human Participation — "Not special. Brilliant").

---

## 6. The Workforce Is a Graph

- **EKOS-26-016 (Should, P3)** — A workforce SHALL be modeled **as a graph**: **Roles are
  nodes; handoffs are edges carrying handoff/knowledge contracts** — exactly like
  microservices with interface contracts. *Source:* ChatGPT (KWR review — "the workforce
  itself becomes a graph ... Every edge contains handoff contracts. Exactly like
  microservices").

- **EKOS-26-017 (Should, P3)** — Handoff contracts SHALL specify the **inputs/outputs and
  knowledge-object types** exchanged between roles (a typed contract, validated like an
  API). *Source:* ChatGPT (handoff contracts); Vol 18 §6 (validation).

- **EKOS-26-018 (Should, P3)** — Because it is a graph, a workforce SHALL be **traceable,
  versioned, projectable, and analyzable** like any EKOS knowledge (bottleneck detection,
  Vol 03 §16). *Source:* Book 1 Laws 2/3/11; Vol 03 §16.

---

## 7. Workers Manipulate Knowledge Objects, Not Files

- **EKOS-26-019 (Should, P3)** — Workers SHALL operate on **Knowledge Objects, never
  files**: Requirement AI asks EKOS for the architecture of Requirement 183 → EKOS returns
  knowledge objects → Architecture AI creates **Architecture Objects** → Review AI creates
  **Review Objects** → Security AI creates **Threat Objects** → QA AI creates **Test
  Objects**. **Everything produced is a Knowledge Object.** *Source:* ChatGPT (KWR review —
  "AI Workforce doesn't manipulate files. It manipulates Knowledge Objects").

- **EKOS-26-020 (Should, P3)** — All worker output SHALL be **committed via the syscall
  layer** (Vol 22 `Commit`), stamped with the producing Role/Principal as provenance
  (Vol 01 §6, Vol 18 §10), and PDP-scoped. *Source:* Vol 22; Vol 18 §10; Vol 01 §6.

---

## 8. Meetings Disappear → Knowledge Handoffs

- **EKOS-26-021 (Should, P3)** — Coordination meetings SHALL be **replaced by Knowledge
  Handoffs** — asynchronous, contract-typed transfers of knowledge between roles, like
  message queues. *Source:* ChatGPT (KWR review — "Meetings disappear ... Knowledge
  Handoffs. Exactly like message queues").

- **EKOS-26-022 (Could, Frontier)** — Where a live human conversation is still needed, it
  SHALL be a **Knowledge Synchronization Session** (Vol 12) anchored to the knowledge
  object in question — not an unanchored meeting. *Source:* Vol 12; synthesis.

---

## 9. Team Bundles — `Team.ekr`

- **EKOS-26-023 (Should, P3)** — A defined workforce/team SHALL be **exportable as a
  `Team.ekr`** (an `.ekr`/EKRA variant, Vol 06/Vol 23) containing: the **Roles/Agents,
  prompts, policies, responsibilities, handoffs, knowledge contracts, and approval
  rules.** *Source:* ChatGPT (KWR review — "Architecture Team.ekr ... Contains Architect
  Agent / Review Agent / ... Handoffs / Knowledge Contracts / Approval Rules").

- **EKOS-26-024 (Should, P3)** — Another organization SHALL **import a `Team.ekr` to
  instantiate an instant workforce** — e.g., `Healthcare Team.ekr`, `Legal Team.ekr`,
  `Finance Team.ekr`, `Manufacturing Team.ekr` — **entire AI organizations, not prompts.**
  *Source:* ChatGPT (KWR review — "another company imports ArchitectureTeam.ekr → Instant
  architecture team ... entire AI organizations").

- **EKOS-26-025 (Should, P3)** — Imported teams SHALL run **sandboxed and capability-
  scoped** (Vol 18 §11, Vol 23 §7) — an imported workforce cannot exceed its declared/
  authorized scope. *Source:* Vol 18 §11; Vol 23 §7.

---

## 10. Marketplace — Sell AI Teams

- **EKOS-26-026 (Should, P3)** — The marketplace (Vol 13) SHALL sell **entire AI Teams**,
  not just widgets — e.g., **SAP Modernization Team, AWS Migration Team, Security Audit
  Team, TOGAF Review Team, .NET Architecture Team.** A customer imports one EKR → instant
  workforce. *Source:* ChatGPT (KWR review — "Sell AI Teams ... Customer imports one EKR.
  Boom. Instant workforce"); Vol 13.

- **EKOS-26-027 (Should, P3)** — AI Teams SHALL be **certified, governed, and revenue-
  shared** like other marketplace assets (Vol 05 §8, Vol 13). *Source:* Vol 05 §8; Vol 13.

---

## 11. THE ARCHITECTURAL RESERVATION (binding on EKOS now)

> The most important actionable: reserve the architecture so KWR needs **no schema change**
> later — even though KWR is Phase-deferred.

- **EKOS-26-028 (Must, P2)** — **Every Knowledge Object in EKOS SHALL reserve, from day
  one, the fields/relationships:** `Owner`, `Role`, `Assignment`, `Workflow`, `Approval`,
  `Review`, `Escalation` — even if, initially, the **Owner is always a Human** and the
  others are unused. This SHALL be reflected in the **Vol 01 common attribute set** and the
  **Vol 18 KOM object model**, so adopting KWR later requires **no schema migration**.
  *Source:* ChatGPT (KWR review — "I would absolutely reserve the architecture ... every
  Knowledge Object already supports Owner/Role/Assignment/Workflow/Approval/Review/
  Escalation. Even if initially the owner is Human. Later AI. No schema changes"). *Binding
  cross-ref:* **Vol 01 §6, Vol 18 §3/§10** — this reservation is mandated there.

- **EKOS-26-029 (Must, P2)** — The **Principal model (Vol 18 §10)** already distinguishes
  human/AI/connector/app principals; KWR's Role model SHALL be the **higher-level layer
  built on that principal foundation** — so the kernel needs no change, only KWR adds the
  Role/handoff orchestration above. *Source:* Vol 18 §10; ChatGPT (KWR review — unified
  model). *Confirms the reservation is already half-present.*

- **EKOS-26-030 (Should, P2)** — The reservation SHALL be **inert until KWR is enabled** —
  Phase-1 EKOS behaves as today (Human owners, no workforce orchestration), with the fields
  present but dormant. *Source:* ChatGPT (KWR review — reserve, don't implement, in
  Phase 1).

---

## 12. Phase Deferral

- **EKOS-26-031 (Must, P2)** — KWR SHALL **NOT be implemented in Phase 1.** Phase 1 ships
  EKOS with the **reserved architecture** (§11) only. KWR is a later phase / separate
  product line. *Source:* ChatGPT (KWR review — "I would not implement this in Phase 1.
  But I would absolutely reserve the architecture").

- **EKOS-26-032 (Should, P3)** — KWR's own build phasing (when the workforce orchestration,
  Team.ekr, and AI-team marketplace land) SHALL be planned separately, after EKOS's core
  platform phases. *Source:* synthesis; build-plan (Priority≠Phase).

---

## 13. The Discipline Split

- **EKOS-26-033 (Should, P3)** — EKOS work SHALL be recognized as splitting into **two
  complementary disciplines**: **(1) Enterprise Knowledge Systems** — how knowledge is
  represented/stored/versioned/projected/secured/evolved (**EKOS itself**); and **(2)
  Knowledge Workforce Engineering** — how humans/AI/automation collaborate over that
  knowledge using roles/contracts/approvals/workflows (**KWR**). *Source:* ChatGPT (KWR
  review — the discipline split).

- **EKOS-26-034 (Should, P3)** — This split SHALL keep EKOS's core model clean — EKOS
  supports not just AI agents but **entire human-AI organizations without changing its
  core**, a stronger long-term foundation than an "agent orchestration feature." *Source:*
  ChatGPT (KWR review — closing argument).

---

## 14. Patent Opportunity

- **EKOS-26-035 (Should, P3)** — A patent family SHALL be captured (Vol 16): **"Role-based
  Knowledge Workforce executing over a Knowledge Operating System"** — distinct from
  "AI agents", protecting the mechanism of unified human/AI/automation roles transforming
  knowledge objects via governed handoff contracts. *Source:* ChatGPT (KWR review — "This
  is definitely patent-worthy ... Nobody phrases it that way today"); Vol 16. *Depends:*
  Vol 16 (to be added as a family).

---

## 15. Relationship to Existing EKOS Volumes

- **EKOS-26-036 (Should, P2)** — KWR SHALL build on: **Vol 18 §10 (Principal model =
  foundation)**, **Vol 08 (cross-role workflow + ClarificationRequest = precursor;
  generalized into a full workforce)**, **Vol 03 (AI analysts = one worker kind)**, **Vol
  05 (governance/approval/review gates)**, **Vol 06/23 (`Team.ekr`/EKRA + App Runtime)**,
  **Vol 13 (sell AI Teams)**, **Vol 22 (syscalls)**. *Source:* cross-volume synthesis.

- **EKOS-26-037 (Should, P3)** — The **Process Capture AI** (Vol 02 §9) and **ambient AI
  orchestrator** (Vol 03 §7) SHALL be recognized as primitive building blocks a KWR role
  can use — but KWR is the org-level layer above them. *Source:* Vol 02 §9; Vol 03 §7.

---

## 16. Constitution (Book 1) Conformance

- **EKOS-26-038 (Should, P2)** — KWR honors: **platform-before-application & layering
  (Law 13)** — it runs on EKOS; **everything is a Knowledge Object (Law 2/3)** — workers
  produce knowledge, not files; **grounded scoped work (Laws 4/6)** — roles act within PDP
  scope; **capability-not-feature (Law 1)** — "Role" is the timeless primitive; **15-year
  longevity (Law 14)** — the reservation means no future schema break. *Source:* Book 1
  Part II.

- **EKOS-26-039 (Should, P3)** — Agent/automation backends a Role wraps are **illustrative
  & pluggable (Law 10)** — KWR is not bound to any agent framework. *Source:* Book 1 Law 10.

---

## 17. Open Questions (parked)

- **OQ-26-A** — KWR as a separate product/repo vs an EKOS-hosted app (Vol 23) — strategic/
  GTM decision; affects branding and the discipline split.
- **OQ-26-B** — Handoff-contract language (typed knowledge-object contracts between roles)
  — design with Vol 18 §6 validation when KWR phase begins.
- **OQ-26-C** — Confidence-based escalation thresholds & human-in-the-loop policy — with
  Vol 03 §14 / Vol 05 §7.
- **OQ-26-D** — Exactly which reserved fields (§11) are first-class node attributes vs
  edges/sub-objects — finalize the reservation shape in Vol 01/18 before Phase-1 build.
- **OQ-26-E** — Team.ekr vs EKRA app-bundle overlap (a team is a kind of app?) — design
  with Vol 23 / Vol 06.

---

## 18. Requirement Index (this volume)

EKOS-26-001 … EKOS-26-039 (stable numbering; KWR-on-EKOS layering, workforce-vs-agent, the
Role model, knowledge-work primitives, workforce-as-graph, knowledge-objects-not-files,
handoffs-replace-meetings, Team.ekr, sell-AI-teams, **the architectural reservation**,
phase deferral, discipline split, patent, relationships expand to 56 distinct stated
rules).

> **NOTE:** The single most important, *actionable-now* item here is **EKOS-26-028 — the
> architectural reservation** (every Knowledge Object reserves Owner/Role/Assignment/
> Workflow/Approval/Review/Escalation from day one). Everything else is Phase-deferred.

> **Gap-closure remaining before A01:** `DESIGN-METHODOLOGY.md` (METHOD), vendor-name
> cleanup (P-A), Patent-Candidate Matrix (P-B). Then the Architecture Phase begins.
> Per protocol: STOP here. Await user direction.
