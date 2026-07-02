# EKOS Architecture — Feature Freeze Declaration

> **Adopted: 2026-07-01 (Opus).** On the cross-architect review (9.8/10) recommendation
> and the user's agreement. Recorded as **ADR-018**. This protects the three qualities
> that now matter most: **simplicity, internal consistency, implementability.**

---

## The Freeze

> **The EKOS architecture is FEATURE-FROZEN.**
>
> **No new subsystem or pillar SHALL be added unless it changes the entire category.**

The foundation has stabilized. Twelve gaps from the prior review are closed; the catalog
reads as an early **platform standard**, not a product spec. Further pillar-hunting now
*risks never shipping* — the failure mode an ambitious platform must avoid.

---

## What is ALLOWED under the freeze

- **Bug fixes** to requirements/architecture (errors, contradictions).
- **Clarifications** and requirement improvements.
- **Formalization** of the *existing* model (e.g., the Knowledge Object Model / Book 2,
  the computational model, formal semantics) — these document what already exists; they
  are **not new pillars**.
- **Governance artifacts** (the ADR Register, the Patent Matrix, conformance reservation,
  the "why-not-existing-tools" category paper).
- **The Architecture Phase (A01–A11)** — designing *how* to build the frozen *what*.
- **Implementation** — building it.

## What is FROZEN (not allowed without category-change justification)

- New top-level subsystems / pillars (we have: Kernel, Compiler, Projection, Acquisition,
  Syscalls, App-Runtime, Living-Books, Self-Hosting, KWR, + the 18 requirement volumes).
- New volumes that introduce a new capability category.
- Scope expansions that grow the surface area rather than deepen the existing model.

## The test for any proposed new pillar

> *Does this change the entire category of EKOS, or is it a capability that fits within
> the existing model?* — If it fits within the model, it is **not** a new pillar; express
> it as a projection, a widget, an app, a workforce role, or a configuration of an
> existing subsystem (per the Design Methodology's five lenses).

---

## Remaining pre-implementation work (formalization, not pillars)

Tracked in `00-MASTER-INDEX.md` and the architecture index:

1. **Book 2 — Knowledge Object Model & Computational Model** (the canonical "atom" + how
   computation flows) — the #1 remaining concern. *Formalization.*
2. **Why-Not-Existing-Tools** category-definition paper (vs DMS/Wikis/Graph-DB/RAG/Agent-
   frameworks/BPM/PLM-ALM/ERP). *Positioning.*
3. **Formal Semantics** — 🟡 RESERVED (math defs of Projection/Identity/Mutation/Lineage/
   Equivalence; write when standardization matters).
4. **Conformance Specification** — 🟡 RESERVED (EKOS Core / Compatible / Certified, like
   POSIX/OCI/K8s; reserve the tiers now, specify later).
5. **The Architecture Phase A01–A11** — proceed.

After Book 2 + the comparison paper, the specification is **implementation-ready**; the
two reserved items (semantics, conformance) are written when their audience (standards
bodies, certification partners) actually exists.

---

## Why this matters (the user's own principle)

> *"Kaam chodna hi kyun, khatam karke aage chalte hain"* — finish, don't abandon. The
> freeze is the discipline that turns an endlessly-expanding vision into a **shippable
> platform**. The vision is captured (1,560+ requirements, 27 volumes, Book 1 + Book 2).
> Now we protect it from feature creep and **build**.
