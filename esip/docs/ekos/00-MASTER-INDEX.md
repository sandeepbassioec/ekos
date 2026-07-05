# EKOS — Enterprise Knowledge Operating System
## Master Requirements Index & Resumable State Tracker

> **This file is the single source of truth for the phased requirements generation.**
> If a session ends (credits, crash, context loss), reopen THIS file first. The
> Status column tells you exactly which volume to resume. Nothing is lost.

---

## 0. What EKOS Is (one paragraph)

EKOS is not a tool or a platform — it is a new software category: a unified,
versioned **semantic knowledge graph** where the entire software lifecycle (and
beyond) lives as interactive, traceable, AI-powered **widgets** instead of dead
documents. Knowledge is packaged into a portable, cryptographically-sealed
**`.ekr` Knowledge Runtime Package** that opens in a free, cross-platform
**Viewer** (the "Adobe Reader of enterprise knowledge"), powered by a promptless
**hybrid ambient AI**, governed by **policy-as-code**, and traded in a
**knowledge economy**. ESIP (the current React app at `C:\VectorVolt\esip`) is an
early partial implementation of ONE pillar — the Widget Engine / designer.

---

## 1. Source Research (the raw material — do NOT re-read in full each session)

Located in `C:\VectorVolt\esip\`. Research was done DeepSeek → Gemini → ChatGPT.

| File | Feeds Volumes | Notes |
|---|---|---|
| `blueprintPrompt.txt` | 0, 1–13, 16 | The definitive EKOS blueprint (19 sections). Best high-level map. |
| `research_across_llms.txt` | 1, 2, 3, 14, 15 | 360KB. Widget catalog + ontology + AI architecture + tradeoff matrices. Grep per-volume; never read whole. |
| `EKR_Research.txt` | 6, 7 | `.ekr` format, free Viewer, PII layered encryption, self-destruct, copy-proofing, format secrecy. |
| `Patents.txt` | 16 | 11 ready patent drafts. |
| `ConfusionClarityPrompt.txt` | 5, 8, 9 | Governance deep-dive + org/user customization + 15 enterprise table-stakes + 5 future domains + final spec v1.0. |
| `ResumeCommunityBuilderPaymentsPrompt.txt` | 10, 11, 12, 13 | Living resume, communities, P2P sessions, Stripe/escrow economy. |
| `PendingESIPResearchToMakeIt100Percent.txt` | (cross-check) | Earlier "pending to 100%" list — reconcile against new volumes. |
| (existing) `docs/architecture-suite/` 26-vol catalog | cross-ref | The older EA-suite catalog (~4,845 reqs). EKOS supersedes/absorbs its scope; mine it, don't duplicate. |

---

## 2. Cross-Cutting Constitution (applies to EVERY volume)

These are locked rulings from the research. Every requirement must honor them:

1. **Platform capability > feature** (ChatGPT's golden rule). Each requirement is
   phrased as a reusable platform capability, not a one-off screen. ("Knowledge
   Editor Framework," not "Requirement Editor.")
2. **Security = cryptographic hardness, NOT obscurity.** Never write "impossible
   to reverse engineer" or "Microsoft/Google can't read it." Correct wording:
   *"Without authorized cryptographic keys, contents remain computationally
   infeasible to recover."*
3. **"Self-destruct" → "Cryptographic Expiration."** We can guarantee KEY/license/
   token destruction & future-access denial, not physical bit-destruction on
   someone else's disk. Copy-proofing = one-time activation + hardware binding.
4. **Everything is a projection of the graph.** Widgets, resumes, viewers, reports
   = declarative graph query + visual mapping. No siloed data models.
5. **Promptless AI:** the widget IS the prompt. Structured fact-sheets from
   symbolic/small-model layers feed the LLM; user never types a prompt to get the
   crux. LLM never invents facts — facts come from deterministic layers.
6. **No-train-on-customer-content;** air-gap excludes internet-only features;
   provider names (AWS/Bedrock/etc.) are illustrative & pluggable.
7. **Bitemporal, non-destructive, user-extensible ontology** — soft delete, time-
   travel, runtime type extension without schema migration.

---

## 3. Volume Plan & Status (THE TRACKER)

Generation order is foundational → outward. Each volume = one phase = one session.
**Protocol: generate ONE volume → STOP → wait for user "proceed" → next.**

Status legend: ⬜ Not started · 🟡 In progress · ✅ Done · ⏭️ Skipped

| # | Volume | Status | Primary Sources |
|---|---|---|---|
| 00 | Master Index + Vision + Glossary (this file + glossary) | ✅ | blueprint |
| 01 | Semantic Knowledge Graph & Ontology (the soul) | ✅ (90 reqs) | research_across_llms (ontology), blueprint §5, Patents §3, Gemini critique |
| 02 | Widget Engine & Custom Widget OS + Process Capture AI | ✅ (65 reqs) | research_across_llms, blueprint §2–3, Patents §11/§5/§10, Gemini |
| 03 | Hybrid Ambient AI Engine (4 layers, analysts, explainability) | ✅ (90 reqs) | research_across_llms (AI), blueprint §4, ConfusionClarity, Gemini, Patents §4/§9 |
| 04 | Security, Privacy & Graph-Level ABAC + Visual Masking | ✅ (65 reqs) | blueprint §6/§11, EKR_Research (PII), ConfusionClarity, Patents §2/§8, Gemini |
| 05 | Governance Framework (policy-as-code, decision DNA, AI gov) | ✅ (53 reqs) | ConfusionClarity, blueprint §7, research final spec §7 |
| 06 | `.ekr` Knowledge Runtime Package & Format | ✅ (87 reqs) | EKR_Research, blueprint §8/9/10/11/13/16, ChatGPT(runtime/manifest/registry/lineage/provenance/composition/policy), Gemini(.ekrd), Patents §1/§6/§7 |
| 07 | EKR Viewer Family + Crypto-Expiration + Viewer Monetization | ✅ (77 reqs) | EKR_Research, blueprint §8/11/12/16, ChatGPT(viewer family+tiers), Gemini(rendering+watermark), Patents §1/§8/§10/§6/§2 |
| 08 | Enterprise Customization & Workflow Orchestration | ✅ (50 reqs) | ConfusionClarity (customization), blueprint §14, ChatGPT (clarification object), Gemini (spatial CRDT), ESIP Epics B/D/F |
| 09 | Enterprise Table-Stakes & Trust (i18n/WCAG/SSO/SCIM/DR/…) | ✅ (55 reqs) | ConfusionClarity (15 gaps + final-spec §9), blueprint §17, old-catalog Vols 24/25/26, ESIP Epics G/H |
| 10 | Personal Knowledge Space & Digital Twin (resume = projection) | ✅ (40 reqs) | ResumeCommunity, ChatGPT(PKS/Digital-Twin/3 patent families), Gemini(W3C VC/DID) — **ruling OWNED in §2 (no bespoke Resume Widget); Vol 02 flag discharged** |
| 11 | Knowledge Network Framework (communities) | ✅ (29 reqs) | ResumeCommunity, ChatGPT (Community→Knowledge Network reframe) |
| 12 | Knowledge Session Runtime (P2P/mDNS/WebRTC, transport-pluggable) | ✅ (36 reqs) | ResumeCommunity/Gemini(pure-P2P), ChatGPT(Knowledge Session/transport-pluggable), blueprint §14/§15 |
| 13 | Knowledge Economy Framework (marketplace, Stripe/escrow, reputation) | ✅ (46 reqs) | ResumeCommunity, Gemini(fintech shield/Treasury escrow), ChatGPT(Knowledge Commerce), blueprint §16, ConfusionClarity(marketplace) |
| 14 | Full Widget Catalog (all industries) | ✅ (98 reqs / ~88 widgets) | research_across_llms (widget lists), blueprint §2/§17, ConfusionClarity §3, ESIP widget set |
| 15 | Deployment, Tech-Stack & Reference Architecture | ✅ (47 reqs) | research_across_llms (architecture), TECH-STACK.md, Gemini (CQRS/IDE-daemon/edge-signaling), blueprint §17, ESIP migration |
| 16 | Patents & IP Strategy (17 families) | ✅ (36 reqs / 11 drafted + 6 identified + 5 candidate) | Patents.txt, ChatGPT(6 new families), blueprint §19 (moat) |
| 17 | Future-Proofing / Meta-Requirements / Frontier | ✅ (44 reqs) | blueprint §18/§20, research future-proofing, ConfusionClarity §12, old-catalog Vol 20, ChatGPT methodology |

---

## 🏁 CATALOG COMPLETE — all 18 volumes done (2026-06-30)

**Total: ~1,150 atomic requirements across 18 volumes**, synthesized from the 6-day
DeepSeek→Gemini→ChatGPT research, deduplicated and cross-referenced, with the §2
Constitution applied throughout. Every flagged ruling discharged (notably résumé→PKS,
Vol 10 §2). Per-volume counts in the table above.

**Next (not a volume):** build-planning pass — triage all Open Questions (OQ-01-A …
OQ-17-D), assign phases (Priority≠Phase, Vol 15 §17), sequence ESIP→EKOS migration
(Vol 15 §15), rotate the ai-proxy keys before any deploy (Vol 15 §16).

---

## 🔧 GAP-CLOSURE PASS (started 2026-07-01) — the missing OS abstractions

Triggered by a cross-architect review (ChatGPT) that judged the catalog 9.3/10 but
flagged that we modeled the **Runtime/User-space** abstractions while missing the
deeper **OS-layer** abstractions (Kernel, Compiler, Projection Engine, Acquisition,
syscall APIs, App Runtime). These are real gaps in *platform abstraction*, not missing
requirements. Closing them BEFORE the Architecture Phase (A01).

### Foundational restructure (Book 1)

| Doc | Title | Status |
|---|---|---|
| BOOK-1 | **Knowledge Constitution & Manifesto** (the *why EKOS exists* + laws/axioms; elevated from index §2 to standalone Book 1) | ✅ |
| METHOD | **Design Methodology** (capability-vs-feature, 5 lenses, the questions — standalone chapter; elevated from Vol 17 §6) | ✅ |

### New abstraction volumes

| # | Volume | Status | Why (ChatGPT gap #) |
|---|---|---|---|
| 18 | **Knowledge Kernel** (deepest primitives; Kernel ≠ Runtime ≠ User-space) | ✅ (70 reqs) | #1 |
| 19 | **Knowledge Compiler** (Knowledge Objects → compiled artifacts; ≠ widget engine) | ✅ (58 reqs) | #2 |
| 20 | **Knowledge Representation Runtime (KRR)** — fka "Projection Engine"; scope expanded 2026-07-01: KRF/KRR split, Law 16 (Representation Independence), Intent-Based Representation, evidence-artifact exception | ✅ (~72 reqs) | #3 + Law 16 |
| 21 | **Knowledge Acquisition Framework** (all knowledge inflow — meetings/ERP/IoT/voice/sensors/AI/humans) | ✅ (60 reqs) | #4 |
| 22 | **Knowledge OS APIs / Syscall Layer** (`Knowledge.Open/Query/Project/Publish…` = Win32/POSIX equiv) + CCR amendment 2026-07-03: Constitutional Compliance Runtime gate, Constitutional Telemetry Stream, Extension Compliance Manifest (ADR-051) | ✅ (74 reqs) | #5 |
| 23 | **EKOS Application Runtime & SDK** (apps-not-widgets, App Bundles, App Store, sandbox) | ✅ (58 reqs) | #6 |
| 24 | **Living Knowledge Publication Framework** (Living Books — first-class) | ✅ (52 reqs) | #7 |
| 25 | **EKOS as Self-Hosting Reference Knowledge Base** (EKOS imports EKOS; the spec is the first reference impl) | ✅ (44 reqs) | #8 |
| 26 | **Knowledge Workforce Runtime (KWR)** — runs ON EKOS (like .NET on Windows); role-based human+AI+automation workforce; Phase-deferred but **architecture reserved now** (EKOS-26-028) | ✅ (56 reqs) | #9 |

### Cross-cutting passes

| Pass | Action | Status |
|---|---|---|
| P-A | **Vendor-name cleanup** — `VENDOR-NEUTRALITY-PASS.md`: audited 54 mentions / 13 files, all illustrative + port-backed; verdict PASS | ✅ |
| P-B | **Patent-Candidate Matrix** — `PATENT-CANDIDATE-MATRIX.md`: 29 families (11 drafted + 6 identified + 12 candidate) mapped volume→mechanism→family | ✅ |

### 🏁 GAP-CLOSURE COMPLETE (2026-07-01)

All gaps from the cross-architect review are closed. The catalog is now a true **OS
specification**: **Book 1 (Constitution) + Design Methodology** on top; **Kernel (18) →
Syscalls (22) → Compiler (19) / Projection (20) / Acquisition (21) → Runtime/Apps (23) /
Living Books (24)**; **Self-Hosting (25)**; and **KWR (26)** as a layer-on-top product.
Plus the **Vendor-Neutrality Pass** and the **29-family Patent-Candidate Matrix**.

**Catalog totals: Book 1 + Methodology + 27 volumes (00–26) + Build-Plan + Vendor Pass +
Patent Matrix ≈ 1,560 atomic requirements.**

**NEXT: Architecture Phase A01** — `architecture/00-ARCHITECTURE-INDEX.md`.
The requirements (WHAT) are complete; architecture (HOW) begins.

---

## 🧊 POST-9.8-REVIEW: FEATURE FREEZE + FORMALIZATION (2026-07-01)

Second cross-architect review scored **9.8/10** ("early draft of a platform standard")
and recommended a **Feature Freeze**. Adopted. No new pillars; only formalization,
governance, fixes, and implementation proceed.

| Artifact | Purpose | Status |
|---|---|---|
| `FEATURE-FREEZE.md` | Declares the freeze (ADR-018) — protect simplicity/consistency/implementability | ✅ |
| `ADR-REGISTER.md` | Architecture Decision Register — every decision w/ rationale/alternatives/consequences; seeded ADR-001…018 (living; A03+ append) | ✅ |
| **BOOK-2** — Knowledge Object Model & Computational Model | The canonical "atom" (like CLR Object Model / LLVM IR) + how computation flows (Object→Mutation→Scheduler→Projection→Event→AI→Transaction). **#1 remaining concern.** Formalization, NOT a new pillar. | ✅ (~58 items; formal semantics RESERVED B2-044) |
| `WHY-NOT-EXISTING-TOOLS.md` | Category-definition paper: where DMS/Wikis/Graph-DB/Enterprise-Search/RAG/Agent-frameworks/BPM/PLM-ALM/ERP stop and EKOS begins | ✅ (~42 items; incl. honest caveats + capability matrix) |
| FORMAL-SEMANTICS | Math defs of Projection/Identity/Mutation/Lineage/Equivalence | 🟡 RESERVED |
| CONFORMANCE-SPEC | EKOS Core / Compatible / Certified tiers (like POSIX/OCI/K8s) | 🟡 RESERVED |

> Plan: **Book 2** (the #1 concern) → **Why-Not paper** → then continue the Architecture
> Phase (A02 onward). Formal Semantics + Conformance stay reserved until their audience
> (standards bodies / certification partners) exists.

> Volumes may be re-ordered or split on request — it's just edits to this table.

---

## 4. Per-Volume Output Convention

Each volume is written as `NN-<slug>.md` in this folder, containing:
- **Scope & boundary** (what's in, what's explicitly deferred to another volume)
- **Atomic requirements** with stable IDs: `EKOS-<VOL>-<NNN>` (e.g., `EKOS-06-014`)
- Each requirement: statement (SHALL/SHOULD/MAY) · rationale · source-trace ·
  dependencies · priority (Must/Should/Could) · phase hint (MVP/P2/P3/Frontier)
- **Open questions** parked at the end (don't block the volume)
- On completion: flip this file's Status to ✅ and add the req count.

---

## 5. Resume Protocol (read this if you're resuming cold)

1. Open this file. Find the first 🟡 or ⬜ row in §3 — that's the next volume.
2. Read ONLY that volume's Primary Sources (Grep for the relevant section; do not
   read 360KB files whole).
3. Generate the volume → write `NN-<slug>.md` → flip Status to ✅ (+ req count).
4. STOP. Ask the user to proceed to the next volume.

---

## 6. Credit-Conservation Notes

- **Model:** Volume generation is a grind — recommend **Sonnet 4.6** for the bulk
  (per [[feedback-model-budget]]); reserve **Opus** for hard architecture/ontology
  calls (Vol 01) only. User is <$50 remaining as of 2026-06-30.
- **Never** re-read full source files each session — Grep the relevant slice.
- One volume per session. Stop and checkpoint after each.

---

_Created 2026-06-30. Phased EKOS requirements generation. ESIP designer/PI-planning
pipeline is paused but NOT abandoned — see [[esip-resume-state]]._
