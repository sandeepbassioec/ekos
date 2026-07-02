# EKOS Design Methodology

> **A standalone chapter of Book 1's foundations** (gap-closure METHOD; cross-architect
> review #11). The methodology is "probably the most valuable thing we discovered" — the
> repeatable process by which EKOS capabilities are *discovered, evaluated, and locked*.
> It must be preserved as first-class knowledge, not lost in conversations.
> **Created:** 2026-07-01 (Opus). Companion to [BOOK-1](BOOK-1-KNOWLEDGE-CONSTITUTION.md)
> (which carries the *laws*; this carries the *method*). Self-hosted per Vol 25 §9.

---

## 1. Why a Methodology (Not Just Requirements)

The requirements capture **what EKOS does**. This chapter captures **how EKOS is
designed** — and that is just as valuable, because it guides every architectural
decision long after v1 ships. The pattern observed across the entire EKOS journey:

> Every time an idea arrived looking like a *feature*, enough questioning revealed a
> deeper *platform capability* beneath it.

This is not a style. It is **the mechanism that turns a product into an operating
system** (Book 1 Manifesto §3) and the reason EKOS can survive 15+ years.

---

## 2. The Core Question

Whenever any need, idea, or feature request appears, the **first and governing question**
is:

> ### "Is this a feature, or what platform capability does it reveal?"

Worked examples (the precedents that built EKOS):

| Arrived as (feature) | Revealed capability |
|---|---|
| Resume builder | **Personal Knowledge Space** (résumé = one projection) |
| Video call | **Knowledge Synchronization Session** (video = one stream) |
| Community | **Knowledge Network** (community = one type) |
| Payments | **Knowledge Economy** (a fee = one transaction) |
| Encrypted export | **Knowledge Runtime Package** (encryption = one property) |
| Agent orchestration | **Knowledge Workforce Runtime** (an agent = one worker kind) |
| PDF/Word export | **Knowledge Compiler + Projection Engine** |
| Document | **Knowledge** (the primary abstraction itself) |

---

## 3. The Discovery Cascade

When the core question says "not just a feature," descend through these questions until
the true primitive is found:

```
Is this a feature?
   → No.
What capability does it reveal?
   → ...
What abstraction exists below that?
   → ...
Does it belong in the Kernel, the Runtime, or User-space?   (Book 1 §4)
   → ...
Can it survive twenty years architecturally?
   → ...
Can it become a patent (a novel mechanism)?
   → ...
Can it become a constitutional Law (a non-negotiable)?
   → ...
```

The output of the cascade is one of: a **kernel primitive**, a **runtime subsystem**, a
**user-space capability/app**, a **projection**, a **new Law**, and/or a **patent family**.

---

## 4. The Five Lenses (the Lock Gate)

An idea is **locked into the catalog only after passing all five** (the gate referenced
in Vol 02 §11, Vol 17 §6):

1. **Capability vs feature** — is it a reusable platform capability, not a one-off screen?
2. **New category vs copy** — does it create/extend a category, or merely copy an existing tool?
3. **20-year survivability** — can the abstraction survive two decades of change architecturally?
4. **Simplify vs complicate** — does it simplify the platform, or add net complexity?
5. **CTO test** — can a Fortune-100 CTO justify building their engineering org around it?

Fail any lens → the idea is reshaped (usually one abstraction-level deeper) or rejected.

---

## 5. The Anti-Feature-Creep Discipline

With 4,000+ possible requirements, the methodology is also the guard against bloat:

- **Prefer primitives to features.** A primitive serves a thousand features; a feature
  serves one. Build the primitive.
- **No bespoke artifact where a projection suffices** (Book 1 Law 3). The clearest
  precedent: **no Resume Widget** — a résumé is a projection of PKS.
- **Defer, don't drop.** A valid idea that isn't a Phase-1 capability is *reserved*
  (architecturally) and *phase-tagged* — never silently lost (e.g., KWR's reservation,
  Vol 26 §11).
- **One layer at a time.** When unsure if something is kernel/runtime/user-space, find the
  lowest layer that *must* own it; everything else sits above.

---

## 6. Boundary Discipline (How Volumes Stay Clean)

Every volume/spec document follows the same structure, which is itself part of the method:

- **Scope** — what this document owns.
- **Boundary** — what is explicitly *deferred*, with a "→ Vol X" pointer (never
  duplicated; never silently assumed).
- **Cross-references** — `traces_to` links to dependent volumes.
- **Constitution conformance** — which Book 1 Laws apply.
- **Open Questions** — parked, owned, dispositioned (not blocking).

This discipline is *why* the catalog has clean ownership and no duplication — and it is
reusable for every future spec.

---

## 7. The Cross-AI Verification Loop (a process the user invented)

A meta-method proven over this project: **synthesize on one model, challenge on others.**
Research flowed DeepSeek → Gemini → ChatGPT → Claude, each adversarially reviewing the
last. This **fool-proofing loop** surfaced the deepest gaps (the OS abstractions, KWR)
that a single pass missed. It SHALL remain a standing practice: *take important
architecture to multiple independent reviewers before locking it.*

---

## 8. Outputs the Methodology Produces

Applying this method yields, for any accepted idea:
- A **placement** (kernel / runtime / user-space / projection) per Book 1 §4.
- A **capability statement** (phrased as a platform capability, not a feature).
- **Phase & priority** (Priority ≠ Phase — Vol 15 §17).
- An **architectural reservation** if Phase-deferred (no future schema break).
- A **patent-candidate flag** (→ Patent-Candidate Matrix, `PATENT-CANDIDATE-MATRIX.md`).
- Possibly a **new Law** (→ Book 1, by logged amendment).

---

## 9. Standing Rule

> **Every future addition to EKOS — feature request, idea, or pivot — SHALL be run
> through this methodology (the core question → the cascade → the five lenses) before it
> is built.** This is how EKOS stays an operating system, stays coherent, and stays alive
> for fifteen years.

*(This methodology is itself preserved as self-hosted knowledge — Vol 25 §9 — so it
guides decisions long after v1.)*
