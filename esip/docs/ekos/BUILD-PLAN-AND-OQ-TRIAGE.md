# EKOS Build Plan & Open-Question Triage

> **Status:** 🟡 Living document · **Started:** 2026-06-30 (Opus)
> The post-catalog planning artifact. Turns the 18-volume requirements catalog
> ([00-MASTER-INDEX](00-MASTER-INDEX.md)) into a phased, sequenced, buildable plan.
> **Not a requirements volume** — a planning/decision log.

---

## 0. How to use this file

1. Resume planning by reading §1 (decisions log) then §2 (phases).
2. Open Questions are triaged in §4 — each has an owner, a disposition (DECIDE-NOW /
   BENCHMARK / DEFER / LEGAL), and where it lands.
3. The ESIP→EKOS migration order is §3.
4. Pre-deploy blockers are §5 — **nothing ships until these are green**.
5. Priority ≠ Phase (Vol 15 §17): a "Must" requirement is not automatically Phase 1.

---

## 1. Decisions Log (locked rulings carried from the catalog)

| # | Decision | Source |
|---|---|---|
| D1 | Build on current React+Firebase+ai-proxy stack; **evolve behind ports, NOT rewrite** | Vol 15 §2 |
| D2 | Backend = **hybrid .NET Core 10 + Rust microservice**; VectorVolt stays separate pure-Rust | Vol 15 §5 |
| D3 | Storage = **polyglot CQRS** (graph DB + event store), client-configurable DB behind repository port | Vol 15 §6 |
| D4 | AI = **provider-pluggable** (Bedrock/Claude default + sovereign Llama/Mistral); one shared symbolic engine for 3 policy kinds | Vol 15 §7, Vol 05 §14 |
| D5 | Diagramming = **maxGraph** (already shipped in ESIP) | Vol 15 §4 |
| D6 | **Résumé = projection of PKS**; no bespoke Resume Widget | Vol 10 §2 |
| D7 | **"Cryptographic Expiration"** not "self-destruct"; cryptographic-hardness-not-obscurity wording everywhere | Constitution §2/§3 |
| D8 | **No training on customer content**; anonymized opt-in behavioral only | Constitution §6 |
| D9 | EKOS is a **feeder to / two-way sync with Jira/ADO**, not a forced replacement (sprint-board lesson) | Vol 15 §9; [[esip-resume-state]] |
| D10 | Capability-vs-feature Golden Rule + 5 lenses gate every addition | Vol 17 §6 |

---

## 2. Phase Plan

> Phase 1 = the smallest coherent slice that proves the EKOS thesis on the current
> stack. Later phases add the heavy/differentiating capabilities. Frontier = research.

### Phase 1 — "EKOS MVP on the current stack" (extends the shipped ESIP Blended-MVP)
**Goal:** prove "knowledge graph + widgets + traceability + ambient AI crux" end-to-end.
- **Graph foundation (subset):** repository port + minimal ontology core (`Element`/
  `Artifact`/`Requirement`/`ArchitectureComponent`), `traces_to`, soft-delete — *Vol 01
  §3/§4/§5/§8 (subset)*. Runs on Firestore behind the DAL.
- **Widget engine (graph-backed) for the top widgets:** Traceability Graph (master),
  Requirement, C4/flow (maxGraph), Trade-off, NFR, Matrix, Mermaid — *Vol 02 §2, Vol 14
  §4/§5/§19 (ESIP widgets re-expressed as projections)*.
- **Ambient AI L1 + crux (cloud):** symbolic rules (traceability gaps, PII-without-
  encryption) + Layer-3 crux via ai-proxy → .NET-bound later — *Vol 03 §4/§6, §9 Crux+
  Traceability-Policeman*.
- **ABAC v1 + masking:** classification labels + server-side projection (basic) — *Vol 04
  §2–§6 (subset)*.
- **Cross-role workflow + clarification object + feature flags + persona/working-mode**
  (already shipped in ESIP — generalize) — *Vol 08 §8/§9, Vol 05 §11*.
- **Identity/license scaffolding** (already shipped Epic G/H) — *Vol 09 §6/§13 (subset)*.
- **Export Word/HTML/PDF + JSON-LD/RDF no-lock-in stub** — *Vol 02 §5, Vol 01 §14*.
- **PRE-DEPLOY BLOCKERS (§5) must be green before any hosted Phase-1.**

### Phase 2 — "The differentiators"
- **Polyglot CQRS storage migration** + bitemporal versioning + time-travel — *Vol 01 §2/§7*.
- **Full Hybrid AI** (Layer 2 small models, orchestrator, sovereign/air-gapped dual-engine,
  explainability/Chain-of-Thought) — *Vol 03*.
- **`.ekr` Knowledge Runtime Package + free Viewer (Reader)** + layered encryption +
  cryptographic expiration + copy-proof — *Vol 06, Vol 07*.
- **Governance Framework** (policy-as-code engine, Decision DNA, KQE, Governance
  Workbench) — *Vol 05*.
- **Custom Widget OS + Process Capture AI** — *Vol 02 §6–§9*.
- **Enterprise customization** (Screen Blueprints, Visual Workflow Designer, real-time
  collaboration generalized from Yjs) — *Vol 08*.
- **Enterprise identity** (SSO/SAML/OIDC + SCIM/JIT + pluggable IdP) + DR + residency +
  analytics + bulk/diagram import — *Vol 09*.
- **Server-side enforcement migration to .NET** — *Vol 15 §13*.
- **Full widget catalog rollout** (industry-deep + advanced viz/simulation) — *Vol 14*.

### Phase 3 — "The consumer + ecosystem editions"
- **Personal Knowledge Space & Digital Twin** (résumé projection, W3C VC/DID) — *Vol 10*.
- **Knowledge Networks** (communities) — *Vol 11*.
- **Knowledge Session Runtime** (pure P2P/mDNS/WebRTC, Identity Bridge) — *Vol 12*.
- **Knowledge Economy** (marketplace, Stripe Connect+Treasury escrow, reputation) — *Vol 13*.
- **Viewer family** (Presenter/Review/Inspector/Diff/Mobile/Embedded/SDK/CLI/Browser) +
  Pro monetization + Sandeep Genie — *Vol 07*.
- **EKR Everywhere + Gateway Module** — *Vol 07 §19/§20*.
- **Seed Mode, Delta bundles, Living References, Bundle Registry** — *Vol 06*.

### Frontier — research track (not roadmapped until a thesis-fit emerges)
- Green-IT, M&A due-diligence, regulatory-impact, voice, AI pre-review board, AR/VR,
  evolutionary fitness, generative design, chaos sim, enterprise digital twin — *Vol 17 §8*.

---

## 3. ESIP → EKOS Migration Sequence (Vol 15 §15)

1. **Graph DAL behind the repository port** — keep Firestore, add the seam. *(enables all
   graph-backed work without a DB switch)*
2. **Widgets → graph-backed projections** — re-express the shipped Tiptap widgets as
   persisted-query + visual-recipe (Vol 02 §14). Start with Requirement + Traceability.
3. **H3 requirement-model convergence** — unify FormalRequirement / DAL Requirement /
   localStorage Requirement (the known wart). *(blocks clean traceability)*
4. **Personalization/workflow → graph nodes** (Vol 08 §13).
5. **AI behind the provider port** — fold ai-proxy into the .NET API.
6. **Server-side enforcement to .NET** + retire stranded Express+Prisma `server/`.
7. **Polyglot CQRS storage** migration (Phase 2 trigger).

---

## 4. Open-Question Triage

> Disposition: **DECIDE-NOW** (ruling below) · **BENCHMARK** (needs a spike) ·
> **DEFER** (revisit at a phase) · **LEGAL** (counsel needed).

| OQ | Topic | Disposition | Note / Ruling |
|---|---|---|---|
| 01-A | Graph DB + event-store engine pick | BENCHMARK | Neo4j + Kafka/ClickHouse as candidates; spike in Phase 2 prep. Stay pluggable. |
| 01-B | Bitemporal storage strategy | BENCHMARK | Prototype append-log vs versioned-edge vs snapshot-delta; affects cost. Phase 2. |
| 01-C | Query language (openCypher/GQL vs custom DSL) | DECIDE-NOW | **openCypher/GQL** for portability + widget-author familiarity. |
| 01-D | Who adds global vs project-local ontology types | DECIDE-NOW | Global types via governed promotion pipeline (Vol 05 §13); project-local by delegated admins (Vol 05 §4). |
| 02-A / 07-A | Host shell + native rendering engine | DECIDE-NOW | **WASM rendering core in a web/Electron host**; keep ESIP React shell for Phase 1, add WASM engine when graphs get large. |
| 02-B / 08-D | Custom-code/widget sandbox model | DECIDE-NOW | **WASM sandbox** for client-side; server-side execution for privileged. Phase 2. |
| 02-C | ESIP Tiptap widgets survive vs rebuilt | DECIDE-NOW | Rebuild graph-native incrementally (migration step 2); keep authored content. |
| 03-A | AI model picks per layer | DECIDE-NOW | Layer-3 cloud = **latest Claude** (default) + pluggable; sovereign = Llama/Mistral; Layer-2 = small fine-tuned (Phase 2). |
| 03-B | Rule-engine home | DECIDE-NOW | **One shared symbolic engine**, 3 policy kinds (Vol 05 §14). |
| 03-C | Layer-2 training-data licensing | LEGAL | Public TOGAF/cloud-ref/GitHub corpora — counsel review before training. |
| 03-D / 07-D / 15-D | On-device model footprint | BENCHMARK | Quantized LLM on laptop/phone feasibility spike. Phase 2/3. |
| 04-A | Policy-engine unification | DECIDE-NOW | Resolved: shared engine (Vol 05 §14). |
| 04-B | Per-element ABAC perf | BENCHMARK | Caching + projection benchmark with Vol 01 §15. Phase 2. |
| 04-C | PII auto-classification accuracy + human review | DEFER | Pilot in Phase 2 with a human-review queue. |
| 04-D / 05-B | Legal-hold vs right-to-erasure conflict | LEGAL | Counsel ruling; affects Vol 04 §16 / Vol 05 §6. |
| 04-E | Screen-share detection reliability + analog-hole residual | DEFER | Document residual; watermarking is mitigation not prevention. Phase 3 (viewer). |
| 05-A | Default 200+ policy library content | DEFER | Author seeded TOGAF/SOC2/well-arch ruleset alongside Vol 14 compliance widgets. Phase 2. |
| 05-C | KQE scoring cost at scale | BENCHMARK | Caching/sampling with Vol 01 §15. Phase 2. |
| 05-D | Sandbox→prod promotion pipeline stages | DECIDE-NOW | review → diff → approve → promote → rollback; build with Vol 08 workflow. Phase 2. |
| 06-A | `.ekr` on-the-wire container spec | DECIDE-NOW | Author a format RFC in Phase 2 (extend a signed-archive standard; version the format). |
| 06-B | Delta (`.ekrd`) algorithm | BENCHMARK | Graph-diff representation + merge-conflict handling; with Vol 01 bitemporal. Phase 2. |
| 06-C | Destruction-attestation backing | DECIDE-NOW | Start with **activation-server records** (PKI); skip blockchain unless a customer requires it. |
| 06-D / 13-B | Registry trust model + revenue-share % | DEFER | With Vol 05 governance + Vol 13 economics. Phase 3. |
| 06-E / 14-D | Living-reference web scraping legality | LEGAL | Rate-limit + ToS review. Phase 3. |
| 07-B | App-store DRM + on-device AI policy | LEGAL | Store-policy review before viewer submission. Phase 3. |
| 07-C | Watermark robustness vs compression/crop | BENCHMARK | Detection-accuracy study. Phase 3. |
| 07-E / 15-B | Secure-erase on SSD residual | DECIDE-NOW | **Key-destruction is the guarantee** (Constitution §3); document SSD residual honestly. |
| 08-A | WorkflowRequest single canonical model | DECIDE-NOW | Vol 08 owns the object; Vol 05 governs it. One model. |
| 08-B / 01-B / 09-B | CRDT × bitemporal reconciliation | BENCHMARK | Design with the storage spike. Phase 2. |
| 08-C | Blueprint precedence order | DECIDE-NOW | **tenant → persona → project → user** (most-specific wins; mandatory locks override). |
| 08-E | Spatial ghost-lock UX | DEFER | Prototype with the rendering engine. Phase 2. |
| 09-A | Certification sequencing | DEFER | GTM decision; SOC2 first when hosted Phase 2/3. |
| 09-C / 14-B | Diagram-import fidelity matrix | DEFER | ESIP already covers several; document round-trip vs one-way. |
| 09-D | Metering granularity | BENCHMARK | With Vol 01 §15 + Vol 13. Phase 2/3. |
| 09-E / 13 | White-label depth | DEFER | Phase 3 / GTM. |
| 10-A | VC/DID standards surface | DEFER | Standards review. Phase 3. |
| 10-B / 13-C | Reputation algorithm + token model | DEFER + LEGAL | Gaming-resistance design + currency-regulation check. Phase 3. |
| 10-C | PKS consumer vs enterprise edition | DEFER | GTM decision. Phase 3. |
| 10-D / 11-B | Default privacy posture for public PKS/profiles | DECIDE-NOW | **Private by default**; explicit opt-in to publish. |
| 10-E | One-PKS-per-human (KYC?) | DEFER | Trust-model decision with Vol 09 identity. Phase 3. |
| 11-A | Network discovery at scale | BENCHMARK | With Vol 01 §15. Phase 3. |
| 11-C | Public-network moderation/abuse | DEFER | Governance design with Vol 05. Phase 3. |
| 11-D | Light governance for Family/Friends nets | DEFER | Product decision. Phase 3. |
| 12-A | Transport-negotiation order/timeouts | DECIDE-NOW | **LAN(mDNS) → P2P(STUN) → TURN-relay**; clear UX when all fail. |
| 12-B | Invitation-Token size/format | BENCHMARK | Compressed text vs QR capacity spike. Phase 3. |
| 12-C | Presence without directory | DEFER | Lean on Vol 11 network presence + opt-in minimal relay. Phase 3. |
| 12-D | Identity-Bridge token refresh offline | DECIDE-NOW | X.509-style validity window; re-auth online on expiry. |
| 12-E | CRDT-over-lossy-P2P resync | BENCHMARK | With storage spike. Phase 3. |
| 13-A | Per-region fintech licensing | LEGAL | **Gates float-interest**; counsel before any payout feature. Phase 3. |
| 13-D | Chargeback/refund across escrow+Connect | LEGAL | Fintech design. Phase 3. |
| 13-E | Cross-jurisdiction tax | LEGAL | Stripe Tax vs custom. Phase 3. |
| 14-A | MVP widget subset sequencing | DECIDE-NOW | Phase-1 widgets listed in §2 above. |
| 14-C | Split catalog 14a/14b | DECIDE-NOW | Keep one file; organizational only. |
| 15-A…E | Engine/stack benchmarks | BENCHMARK | Consolidated in Phase 2 prep spikes. |
| 16-A…E | Patent jurisdiction/eligibility/prior-art | LEGAL | Patent attorney engagement (file P1–P11 first). |
| 17-A | Frontier prioritization | DEFER | Research track; no roadmap until thesis-fit. |
| 17-B | Federated-learning privacy proof | LEGAL + DEFER | With Vol 05 + counsel. Phase 3+. |
| 17-C | AI autonomy on ontology changes | DECIDE-NOW | Human-gated (Vol 05 review gate); no silent ontology mutation. |
| 17-D | Cross-volume OQ→phase mapping | DONE | This table. |

**Triage summary:** ~20 DECIDE-NOW (ruled above), ~14 BENCHMARK (Phase 2 spikes),
~16 DEFER (phase-tied), ~11 LEGAL (counsel — fintech, patents, scraping, training data,
legal-hold).

---

## 5. Pre-Deploy Blockers (nothing hosted ships until green)

| # | Blocker | Source | Status |
|---|---|---|---|
| B1 | **Rotate Bedrock/Groq API keys** in `esip/ai-proxy/.env` (+ git history); new keys server-side | Vol 15 §16; session-verbatim accepted risk | ⛔ open |
| B2 | Harden **dev-grade permissive Firestore rules** (incl. `users` collection) | [[project-esip]] P1 | ⛔ open |
| B3 | Resolve **server↔Firebase JWT mismatch** / retire stranded `server/` | Vol 15 §13 | ⛔ open |
| B4 | **Secrets manager / KMS** wired (no keys in client/repo) | Vol 04 §15, Vol 15 §16 | ⛔ open |
| B5 | Server-side enforcement of ABAC before exposing any public API | Vol 04 §2, Vol 09 §17 | ⛔ open |

---

## 6. Immediate Next Actions (recommended order)

1. **B1 key rotation** — cheapest, highest-risk; do first (you flagged it).
2. **Migration step 1** — graph DAL behind the repository port (no DB switch).
3. **Migration step 3 (H3 convergence)** — unify the 3 requirement models; unblocks
   clean traceability for the Phase-1 widgets.
4. **Phase-1 widget re-expression** — Requirement + Traceability Graph as graph-backed
   projections.
5. **Ambient AI L1 + crux** wired through ai-proxy (then fold into .NET in Phase 2).

> This is a planning log, not code. When you're ready, pick an action and we build it on
> the current stack per D1 (evolve behind ports, not rewrite).
