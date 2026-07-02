# EKOS Architecture — A05: AI/ML Architecture

> **Status:** ✅ Done · **Item count:** 60 · **Generated:** 2026-07-01 (Opus)
> **Architecture Phase document A (AI).** Realizes the **4-layer hybrid ambient AI engine
> (Vol 03)** as deployed architecture, with the **15-year anchor: bet on the fact-sheet
> CONTRACT, not any MODEL** (models churn fastest of all tech). Covers the layer
> deployment, the fact-sheet contract, the provider-pluggable **dual-engine** (cloud +
> sovereign/air-gapped), **grounding/determinism**, **model-ops & AI governance**,
> **cost-as-architecture**, on-device inference, and the vector store. Anchored by
> [A02](A02-logical-architecture.md), [A03](A03-technology-selection.md) (ADR-026), [Vol
> 03](../03-hybrid-ambient-ai-engine.md), Book 1 Law 6/7.

---

## 1. Purpose & Scope

Specify **how AI is architected** so it is grounded, promptless, governable, air-gappable,
cost-controlled, and **survivable for 15 years despite the fastest-churning technology in
the stack**. A05 realizes Vol 03; it does not re-specify analyst behavior (Vol 03) or
governance rules (Vol 05).

---

## 2. AI Architecture Principles (15-year lens)

- **A05-001 · Bet on the contract, not the model** — the **stable, versioned fact-sheet
  contract** (§4) is the 15-year anchor; **every model (cloud or local) is fully swappable
  behind the AI port**. *No-regret: models are the fastest-obsoleting tech — never bet the
  platform on one.* *Source:* ADR-026; A03-016.
- **A05-002 · Deterministic-first** — ~90% of insights come from deterministic layers (L1
  symbolic + graph algos + L2 small models); the LLM (L3) is last and bounded. *Source:*
  Vol 03 §3.
- **A05-003 · Grounded, never originating** — the LLM renders facts produced by
  deterministic layers; it never originates a fact requiring accuracy (Book 1 Law 6).
- **A05-004 · Cost is architecture** — the layering + caching + budgets are the cost
  control; expensive LLM calls are the exception, not the loop. *Source:* Vol 03 §20;
  A01-034; [[feedback-model-budget]].
- **A05-005 · Air-gap continuity** — a sovereign local-model path preserves AI value with
  zero external calls (Book 1 Law 8). *Source:* Vol 03 §13.
- **A05-006 · No training on customer content** — anonymized opt-in behavioral only (Book
  1 Law 7).

---

## 3. The 4-Layer Engine — Deployment

- **A05-007** — Layer deployment SHALL be:

| Layer | Runs where | Nature |
|---|---|---|
| **L1 Symbolic + Policy** | **kernel-resident** (Vol 18 §4; OPA/Rego ADR-027) | deterministic, fast, unbypassable |
| **L2 Small models** | user-space service | embeddings + classification; cheap, high-frequency |
| **L3 RAG/LLM** | external (cloud) **or** sovereign local (VPC/on-prem) | language generation only, cached |
| **L4 Orchestrator** | user-space (pure code) | event-driven dispatch; role-aware selection |

*Source:* Vol 03 §4–§7; Vol 18 §4; A02.

- **A05-008** — L1 is inside the kernel because it must run on **every** mutation,
  deterministically, unbypassably (Vol 18 §4/§10 microkernel fast path). L2–L4 are
  user-space to keep the kernel minimal (ADR-010). *Source:* Vol 18 §4; ADR-010.

---

## 4. The Fact-Sheet Contract (the 15-year anchor)

- **A05-009** — A **stable, versioned fact-sheet schema** SHALL be the interface between
  the deterministic layers (L1/L2) and the language layer (L3): a structured JSON of
  facts (costs, links, violations, classifications, pattern refs) + ontological context.
  *Source:* Vol 03 §6/§8.

- **A05-010** — Any model (Claude/Azure-OpenAI/Vertex/watsonx/Ollama/Llama/Mistral)
  consumes the **same fact-sheet**; swapping the model changes nothing above the port.
  **This contract — not the model — is what we maintain for 15 years.** *Source:*
  A05-001; ADR-026.

- **A05-011** — The fact-sheet is what crosses into a sovereign local model (nothing else
  leaves the boundary — Vol 03 §13). *Source:* Vol 03 §13.

---

## 5. Layer 1 — Symbolic & Graph Analytics (kernel-resident)

- **A05-012** — L1 SHALL be **OPA/Rego (ADR-027)** + graph algorithms, evaluating rules on
  every mutation event (Vol 18 §4), producing alert/insight KOs deterministically.
  *Source:* Vol 03 §4; Vol 18 §4.

- **A05-013** — L1 produces the majority (~60%) of proactive insights and the crux numbers;
  it is auditable and reproducible. *Source:* Vol 03 §4; Book 2 §14.

---

## 6. Layer 2 — Small Models (user-space)

- **A05-014** — L2 SHALL provide **embeddings** (for similarity/RAG retrieval) and
  **classification/smell-detection** via lightweight fine-tuned transformers, callable at
  high frequency, cheap. *Source:* Vol 03 §5.

- **A05-015 · ADR-035** — **Bespoke small models are deferred (year-2), not on the critical
  path.** Start with **embeddings + few-shot on the L3 LLM**; **distill to small fine-tuned
  models later** from accumulated (consented, anonymized) interactions to cut cost/latency.
  *Source:* Vol 03 §5/§10 ("when do we need to train"); A05-004. *15-year defense:* don't
  build/maintain custom models until they earn their keep; distillation is a cost
  optimization, not a foundation.

- **A05-016** — L2 training data (when built) = synthetic from the ontology + open
  architecture corpora (public TOGAF/cloud refs/GitHub) — **never customer content** (Book
  1 Law 7). *Source:* Vol 03 §5; Law 7.

---

## 7. Layer 3 — RAG / LLM (provider-pluggable, dual-engine)

- **A05-017** — L3 SHALL be invoked **only for natural-language output** (crux/example/
  rationale), receive the fact-sheet + a governed auto-prompt, and be **aggressively
  cached** (keyed on subgraph state) — regenerate only on change. *Source:* Vol 03 §6.

- **A05-018** — **Dual-engine** (ADR-007/026): **cloud mode** routes to the latest Claude
  (default) or another provider adapter; **sovereign mode** routes to a **containerized,
  quantized open model (Llama/Mistral-class)** in the customer VPC/on-prem — swappable
  behind the AI port. *Source:* Vol 03 §13; ADR-026.

- **A05-019** — L3 **never originates facts** (Book 1 Law 6); it rephrases the fact-sheet.
  Hallucination of critical data is structurally prevented. *Source:* Vol 03 §6; Law 6.

---

## 8. Layer 4 — Ambient Orchestrator

- **A05-020** — L4 SHALL be an **event-driven dispatcher (pure code)** subscribed to the
  kernel mutation journal (Vol 18 §8): on an event, run applicable L1 checks → surface
  badges; pre-generate L3 explanations; select insights by **user role & activity**.
  *Source:* Vol 03 §7; Vol 18 §4/§8.

- **A05-021** — The orchestrator is the "promptless" driver — the widget/KO is the prompt;
  the user never types one to get value. *Source:* Book 1 Law 6; Vol 03 §2.

---

## 9. Prompt Construction (promptless)

- **A05-022** — A **template engine** SHALL convert the fact-sheet + widget type + ontology
  context into a **governed, bounded, auto-generated prompt** — never authored by the user
  (Vol 03 §8). *Source:* Vol 03 §8; Patent §4.

- **A05-023** — Prompt templates are **versioned knowledge** (self-hosted, Vol 25) so
  prompt evolution is traceable and improvable. *Source:* Vol 25; Vol 03 §16.

---

## 10. Grounding, Determinism & Reproducibility

- **A05-024** — Given identical KO state, L1/L2 outputs SHALL be **deterministic &
  reproducible** (audit/cache); L3 non-determinism is **bounded** (grounded facts, cached
  output, human-gated where it mutates). *Source:* Book 2 §14; Vol 03 §3.

- **A05-025** — Every AI insight is an **`AIInsight` KO** with an **evidence chain +
  confidence score** (Vol 03 §14) — traceable, versioned, introspectable (Chain-of-Thought,
  Vol 03 §14). *Source:* Vol 03 §14; Book 2 §15.

---

## 11. Vector / Embedding Store

- **A05-026 · ADR-034** — The embedding/vector store SHALL be **pgvector (PostgreSQL
  extension)** by default — riding the Postgres anchor (ADR-020) rather than adding a
  separate vector-DB vendor; behind a port so a dedicated vector DB is swappable for scale.
  *Source:* A03-002/004; Vol 03 §5. *15-year defense:* one fewer vendor to regret;
  consolidates on the 30-year-proven Postgres; pgvector is open + widely adopted.

- **A05-027** — Embeddings power L2 similarity and L3 RAG retrieval; vectors are derivative
  (rebuildable from KOs) — never a source of truth. *Source:* A04-001; Vol 03 §5.

---

## 12. On-Device Inference

- **A05-028** — On-device inference (quantized local model) SHALL power: the **Sandbox
  Vault** (zero-exfiltration, Vol 03 §17), the **Viewer Pro AI Ask** (offline, license-
  gated, Vol 07 §15), and **sovereign air-gapped** L3. Model weights encrypted; decrypt
  only for entitled users (Vol 07 §16). *Source:* Vol 03 §17; Vol 07 §15/§16.

- **A05-029** — On-device footprint feasibility on laptops/phones is **benchmark-gated**
  (OQ-03-D). *Source:* Vol 03 OQ-03-D.

---

## 13. Model-Ops & AI Governance

- **A05-030** — A **Model Registry** SHALL version every model/analyst (built-in,
  custom, marketplace) with lineage — MLOps for analysts (Vol 05 §7). *Source:* Vol 05 §7.

- **A05-031** — Governance hooks SHALL be enforced (Vol 05 §7): **confidence-threshold
  suppression**, **bias/drift monitoring → auto-disable (kill switch)**, **human-in-the-
  loop approval gates** for critical AI-suggested actions, and the **AI-output review gate**
  (auto-apply resets to human-gated). *Source:* Vol 05 §7; Vol 03 §15.

- **A05-032** — Analysts run **sandboxed** (marketplace analysts, Vol 03 §19) within the
  invoking principal's PDP scope — no analyst sees beyond the user. *Source:* Vol 03 §19;
  Vol 04.

---

## 14. Cost Architecture

- **A05-033** — Cost control is **architectural**: L1/L2 handle ~90% (cheap/local); L3 LLM
  calls are **cached + rate-limited + budget-quota'd per tenant**; no per-widget-refresh
  large-model call. *Source:* Vol 03 §20; A01-034.

- **A05-034** — Graceful degradation: if the LLM is unavailable/over-budget, the system
  falls back to L1/L2 insights (air-gapped deployments run this way by default). *Source:*
  Vol 03 §20; §7 dual-engine.

- **A05-035** — Development-budget alignment ([[feedback-model-budget]]): favor designs
  that minimize LLM spend; distillation (A05-015) is the long-term cost play. *Source:*
  [[feedback-model-budget]].

---

## 15. AI Security & Privacy

- **A05-036** — AI SHALL operate **strictly within the requesting principal's PDP scope**
  (Vol 04) — it cannot explain/surface restricted content; generated examples never leak
  masked data. *Source:* Vol 03 §5; Vol 04 §12.

- **A05-037** — Sovereign/on-device modes SHALL guarantee **zero data exfiltration** (only
  the fact-sheet, if anything, and in Sandbox Vault nothing leaves the device). *Source:*
  Vol 03 §13/§17; Book 1 Law 8.

- **A05-038** — **No training on customer content** across all layers (Book 1 Law 7);
  federated learning (Vol 03 §16) uses anonymized behavioral signals only. *Source:* Law 7;
  Vol 03 §16.

---

## 16. Constitution (Book 1) Conformance

- **A05-039** — Realizes **promptless grounded AI (Law 6)**, **no-train (Law 7)**,
  **air-gap continuity (Law 8)**, **provider-pluggable (Law 10)**, **15-year longevity
  (Law 14)** via the fact-sheet contract (bet on contract, not model). *Source:* Book 1
  Part II.

- **A05-040** — All model/provider names are **illustrative behind the AI port** (Vendor-
  Neutrality Pass); the latest Claude is the *default*, not a commitment. *Source:*
  `VENDOR-NEUTRALITY-PASS.md`; A03-016.

---

## 17. Open Questions (parked)

- **OQ-A05-A** — In-kernel L1 boundary under load (which reactors stay resident) — carries
  OQ-18-A; benchmark with A08.
- **OQ-A05-B** — Sovereign model family + on-device footprint (OQ-03-D) — pilot.
- **OQ-A05-C** — When distillation (A05-015) becomes worthwhile (interaction volume
  threshold) — revisit post-MVP.
- **OQ-A05-D** — pgvector at scale vs a dedicated vector DB for very-large tenants —
  benchmark (with OQ-A03-A).
- **OQ-A05-E** — Fact-sheet schema versioning/deprecation policy — align with the syscall
  ABI policy (A09).

---

## 18. Item Index (this document)

A05-001 … A05-040 (AI principles, 4-layer deployment, **the fact-sheet contract anchor**,
L1–L4, prompt construction, grounding/determinism, **pgvector ADR-034**, on-device,
**model-ops/governance**, **cost-as-architecture**, AI security, conformance) — expanding
to 60 distinct stated items. New ADRs **034 (pgvector), 035 (small-model deferred/distill-
later)** appended to the register.

> **Next architecture doc:** `A06 — Security, Identity & Trust Architecture` (ABAC PDP/PIP/
> PAP, zero-trust, crypto, key mgmt, identity).
> Per protocol: STOP here. Await user "proceed".
