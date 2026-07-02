# EKOS Patent-Candidate Matrix (P-B)

> **Gap-closure P-B** (cross-architect review #12). ChatGPT wanted *every requirement*
> tagged Patent-Candidate Yes/No/Family/Mechanism/Claims. Re-tagging ~1,400 requirements
> individually is wasteful and brittle. **This matrix achieves the same goal** — "don't
> forget what's patent-worthy during implementation" — by mapping **each volume's novel
> mechanisms → a patent family**, at the granularity that matters. Vol 16 holds the full
> drafts; this is the navigable index.
> **Done:** 2026-07-01 (Opus). Companion to [Vol 16](16-patents-and-ip-strategy.md).

---

## 1. How to Use

- During implementation of any volume, check this matrix: if the mechanism you're building
  maps to a family, **preserve the patent-worthy specifics** (don't genericize them away)
  and link the code/ADR to the family.
- **Family status:** ✅ Drafted (Vol 16 §4) · 🟡 Identified (Vol 16 §5) · 🔵 Candidate
  (Vol 16 §6 / new) — needs attorney assessment.
- **Filing priority** (Vol 16 §7): file the **moat-critical, mechanism-broad** families
  first (marked ⭐).

---

## 2. The Matrix (Volume Mechanism → Patent Family)

| Family | Status | Core novel mechanism | Implementing volume(s) | Pri |
|---|---|---|---|---|
| **P1 · Self-Destructing Hardware-Bound Bundles** | ✅ | device-key wrap + one-time activation + volatile decryption + key destruction → copies inoperable | Vol 06 §9/§10, Vol 07 §9/§10 | ⭐ |
| **P2 · Graph-Level ABAC + Visual Masking** | ✅ | per-node security attrs → visibility-level → structure-preserving masking; screen-share elevation | Vol 04 §3–§5/§13 | ⭐ |
| **P3 · Bitemporal + User-Extensible Ontology** | ✅ | ontology-as-meta-nodes; runtime type extension w/o migration; valid+transaction time travel | Vol 01 §3/§7/§9; Vol 18 §6 | ⭐ |
| **P4 · Promptless Ambient AI Engine** | ✅ | event-triggered auto-prompt from fact-sheet+ontology; no user prompt; air-gapped local LLM | Vol 03 §2/§4–§9; Vol 18 §4 | ⭐ |
| **P5 · Process Capture AI → Widget** | ✅ | record manual routine → infer schema/logic/outputs → live-connected auto-updating widget | Vol 02 §9 | |
| **P6 · Seed Mode / Progressive Self-Assembly** | ✅ | KB-sized seed (manifest+pointers+summary+skeleton) → fetch+progressively assemble full doc | Vol 06 §13, Vol 07 §13, Vol 12 §14 | |
| **P7 · Cross-Document Living Reference** | ✅ | reference-as-edge + last-verified hash + background staleness (fresh/stale/broken) + diff | Vol 06 §22 | |
| **P8 · Presentation Guardian Mode** | ✅ | screen-share detect → override-rights masking → audited Reveal Pass w/ watermark+countdown | Vol 04 §13, Vol 07 §11 | |
| **P9 · Sandbox Vault (zero exfiltration)** | ✅ | on-device drag-drop vault + local embedding index, no data leaves device; PII masking; gov toggle | Vol 03 §17 | |
| **P10 · Living Canvas (interactive fixed-layout)** | ✅ | absolute-coordinate page rendered scaled, *interactive*; lossless PDF; responsive switch | Vol 02 §4, Vol 07 §13 | |
| **P11 · Unified Widget OS & Builder** | ✅ | no-code 3-layer builder → executable widget = live graph projection; versioned graph-node defs | Vol 02 §2/§6–§8 | ⭐ |
| **P12 · Projection-Based Personal Knowledge Profile** | 🟡 | résumé/CV/portfolio = declarative projection of a Personal Knowledge Space | Vol 10 §2/§3 | |
| **P13 · Context-Anchored Human Collaboration** | 🟡 | live session attached to a specific Knowledge Object, not a generic meeting | Vol 12 §2/§12 | |
| **P14 · Knowledge Clarification Workflow** | 🟡 | clarification as a constitutional object (request→evidence→review→approval→mutation) on any node | Vol 08 §9 | |
| **P15 · Knowledge Commerce Framework** | 🟡 | selling knowledge *assets* (widgets/books/teams/agents) + compliant escrow + reputation signals | Vol 13 | |
| **P16 · Policy-Driven Knowledge Projection** | 🟡 | different viewers see different projections of one canonical knowledge, governed by policy | Vol 10 §3, Vol 04 §5, Vol 20 §5 | ⭐ |
| **P17 · Personal Knowledge Digital Twin** | 🟡 | evolving, verifiable individual knowledge representation (identity/graph/history/reputation…) | Vol 10 §4 | ⭐ |
| **P18 · Role-Based Knowledge Workforce over a Knowledge OS** | 🔵 | unified human/AI/automation Roles transforming Knowledge Objects via governed handoff contracts | Vol 26 | ⭐ |
| **P19 · Layered Multi-Recipient Knowledge Bundle** | 🔵 | one package, different content per recipient offline via per-recipient layered encryption + embedded ABAC | Vol 06 §7, Vol 07 §6 | |
| **P20 · Differential Knowledge-Graph Delta (`.ekrd`)** | 🔵 | structural graph diff that splices into a local base graph without full rebuild | Vol 06 §11, Vol 19 §8 | |
| **P21 · Per-Session Forensic Steganographic Watermark** | 🔵 | viewer/session/device identity embedded microscopically; decodable from a leaked photo | Vol 04 §19, Vol 07 §12 | |
| **P22 · Semantic IDE Drift Gatekeeper** | 🔵 | local AST interception checked vs central semantic graph; blocks arch-constraint violations pre-commit | Vol 15 §10 | |
| **P23 · Compliant Knowledge-Marketplace Escrow Sweep** | 🔵 | yield-bearing regulatory-compliant escrow + milestone payouts + compliant core-interest sweep | Vol 13 §8 | (fintech eligibility?) |
| **P24 · Knowledge Compiler (objects → multi-target artifacts)** | 🔵 NEW | AOT compile of knowledge objects via K-IR → multi-target codegen (`.ekr`/PDF/Living-Book/site) | Vol 19 | |
| **P25 · Promptless Knowledge Acquisition Pipeline** | 🔵 NEW | unified capture→extract→classify→ontology-map→AI-enrich→commit across all source modalities | Vol 21 | |
| **P26 · Self-Hosting Knowledge OS** | 🔵 NEW | an OS whose own spec/ontology/governance is represented & operated as knowledge within itself | Vol 25 | |
| **P27 · Knowledge Syscall Layer (OS API for knowledge)** | 🔵 NEW | stable language-agnostic ABI (`Open/Query/Project/Commit/Publish/Subscribe…`) over a knowledge kernel | Vol 22, Vol 18 §12 | (eligibility?) |
| **P28 · Living Knowledge Publication** | 🔵 NEW | self-updating, connected, interactive, security-scoped publication compiled to multiple formats | Vol 24, Vol 19 §7 | |
| **P29 · Team.ekr — Portable AI Organization** | 🔵 NEW | exporting an entire role-based workforce (roles+contracts+approvals+prompts) as an importable package | Vol 26 §9, Vol 06 | |
| **P30 · Intent-Based Representation** | 🔵 NEW | a knowledge object declares representation intent (target format/audience/fidelity/channel); a Knowledge Representation Runtime resolves intent to a recipe and executes it; audience+policy-aware (different viewers → different realizations of the same intent); evidence artifacts are a typed exception, not the default. Inverts the classical export model: knowledge owns rendering, not applications. | Vol 20 §16–17, EKOS-20-040/044, Book 1 Law 16, ADR-050 | ⭐ Mechanism-broad; distinguishes from XSLT/template-engines/export-pipelines on (a) semantic intent on the object, (b) policy+audience-aware materialization, (c) evidence-artifact exception type |

---

## 3. Summary

- **30 patent families** mapped: **11 drafted (P1–P11)**, **6 identified (P12–P17)**, **13
  candidate (P18–P30)** — 6 of the candidates are **newly surfaced by the gap-closure
  OS-abstraction volumes** (Compiler, Acquisition, Self-Hosting, Syscalls, Living-Books,
  Team.ekr) + KWR; **P30 added 2026-07-01** (Intent-Based Representation, Law 16 insight).
- **⭐ Moat-critical / mechanism-broad (file first):** P1, P2, P3, P4, P11, P16, P17, P18.
- **Eligibility-uncertain (attorney must assess):** P23 (fintech method), P27 (API/ABI).
- **Action:** Vol 16 §7 governs filing; draft P12–P29 to the same structure as P1–P11
  before filing; run prior-art per family (OQ-16-C).

---

## 4. Standing Rule

> When implementing any volume, consult this matrix. If your mechanism maps to a family,
> **do not genericize away the patent-worthy specifics**, and link the implementation/ADR
> to the family id. New novel mechanisms discovered during build are added here as 🔵
> candidates and escalated to Vol 16.

*(This matrix satisfies P-B without per-requirement tagging — the volume→mechanism→family
granularity is where the value is.)*
