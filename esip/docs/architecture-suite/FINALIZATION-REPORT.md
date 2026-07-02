# ESIP Requirements Catalog — Finalization Report

**Date:** 2026-06-23 · **Scope:** final cross-volume check of all 20 volumes (≈3,841 requirements) before development.
**Method:** 4 parallel review passes — (1) overlap/ownership, (2) gaps/completeness, (3) consistency/contradictions, (4) prioritization/MVP-readiness.

> **✅ APPLIED 2026-06-23 — this punch-list is now CLOSED.** Every P0/P1/P2 item below was applied across the 20 volumes via a file-partitioned reconciliation workflow (12 agents, edit→verify, all PASS, no content removed). This report is retained as the historical record + traceability. Catalog status: **V-FINAL & RECONCILED — GO for Phase 1.**

## Verdict
**The catalog is comprehensive and build-ready. No showstoppers.** The check surfaced a *bounded, well-understood* punch-list: a handful of design-defining **contradictions** (all reconcilable, and all in **post-MVP** areas), ~5 worthwhile **gaps**, a set of **overlap/boundary** clean-ups, and one systemic issue — **MoSCoW "Must" is over-used** (~68%), conflating "in the product eventually" with "in the MVP." Crucially, **none of these block Phase 1**; the design-defining items live in features the MVP doesn't build yet.

---

## P0 — Resolve before the relevant build phase (design-defining)

| # | Issue | Volumes | Resolution | Blocks MVP? |
|---|-------|---------|------------|-------------|
| C1 | **"We don't train on customer data" (V19) vs. a fine-tuning pipeline that stores edit-pairs/feedback as training data (V06/V14/V15)** — a direct trust contradiction | 19 vs 06,14,15 | Scope the guarantee to **customer *content*** (artifacts/docs/diagrams/queries); allow only **anonymized, PII-scrubbed, per-tenant-opt-in** behavioral signals for improvement; gate all fine-tuning behind explicit consent. Make this a catalog-wide ruling. | No (fine-tuning is post-MVP) |
| C4 | **AI auto-apply of review feedback (V15) can mutate an artifact that's under an approval gate (V12)** | 15 vs 12 | AI-apply on a gated artifact must **block** or **create a new revision + reset the V12 approval** (per WFL-055). Add the dependency. | Partially (Epic D) — handle in MVP workflow |
| C5 | **Air-gapped "identical features" promise (V19) vs. features that need the internet** (real-time Teams/Zoom connectors, marketplace, live regulatory feeds, cross-tenant benchmarking) | 19 vs 16,17,20 | Enumerate explicit **air-gap exclusions** in REQ-ENT-DEP-010; tag those features "SaaS-only / offline-fallback." | No (post-MVP) |
| PRIO | **"Must" ≠ "MVP"** — ~68% of reqs are Must; Vols 01/03/11/13/16/19 mark Phase-2/3/4 capabilities (TOGAF, maxGraph canvas, SSO/SCIM, billing, predictive ML, air-gapped) as Must | catalog-wide | **Decouple priority from phase**: add a `Phase` tag (or MVP badge); add a BUILD-PLAN cross-ref banner to the heavy volumes. (Catalog-level ruling applied to the index.) | No, but essential for planning |
| H3 | **Requirement-model convergence** (localStorage + DAL `requirements` + KH `formalRequirements`) is sized as a "flag" but is MVP-critical — the SA persona generates deliverables *from* requirements | 07 (Epic H3) | Promote P1-H3 from "flag for migration" to a **full convergence** task in Phase 1. | **Yes — MVP** |

## P1 — High value (before the relevant phase)

**Gaps worth adding (top 5 of 18):**
- **G-1 Legal/IP ownership & indemnity of AI-generated content** — top procurement blocker; needs an explicit position (who owns output, infringement indemnity, ToS language). → Vol 19/10.
- **G-2 Interactive-canvas accessibility** — WCAG/keyboard/screen-reader for the **maxGraph editor itself** (not just exported HTML); the most-failed enterprise a11y area. → Vol 03.
- **G-5 AI-pipeline performance SLOs** — testable latency/throughput targets per AI task (extraction <30s/100pp, diagram <10s, copilot turn <3s). → Vol 06/11.
- **G-3 In-app support/help surface** (ticketing, contextual help, chat). → Vol 19.
- **G-4 Partner/reseller billing** (deal-reg, margins, OEM consolidated billing). → Vol 19/13.

**Overlap/boundary clean-ups (highest dev-risk):**
- **MERGE** diagram-import: V03 (REQ-MOD-320–327) duplicates V18 (REQ-IMP-010–018) → V18 is canonical; V03 keeps only "accept elements from V18 pipeline" + export formats.
- **SPLIT** license tiers: V13 owns billing/seat-enforcement; **V17 owns Viewer/Reviewer/PM tier definitions** (stub V13's rows to cross-ref).
- **Jira/ADO three-way**: V08 = read-only evidence; V11 = one-time ALM import; **V19 = ongoing bidirectional sync** (name it in each).
- **Teams capture boundary**: V08 = post-meeting transcript pull; **V17 = real-time/live capture** (prevent two Graph integrations).
- **Code-scan boundary**: V09 = scan *outputs/analytics*; **V17 = agent deployment/lifecycle**.
- **AI-finding model**: define one shared "AI Finding" record consumed by V15; both V15 inline-AI and V20 AIRB write to it.

**Contradictions (reconcilable):** C3 anonymous-Viewer vs entitlement check (add public-share-token grant); C6 **persona numbering** (P13 listed before P12; "P1–P12" used in V11/12/14 — update to P1–P13); C7 peer-intelligence cold-start must be tenant-scoped or DP/k-anon protected.

## P2 — Polish / backlog
Remaining gaps G-6…G-18 (PLG funnel, multi-language deliverables, API deprecation policy, sandbox/demo data, notification center + mobile push, end-user DSR self-service, community/marketplace content moderation, offline conflict UX, error/empty/loading-state standards, AaC schema stability, pricing edge cases, prompt-library i18n, end-to-end pipeline QA). Terminology glossary (deliverable/artifact, persona/role, connector/integration, workspace/project, edition/tier/license). Format unification (V12/V15/V06 table layouts vs the §6 schema; V15's card layout breaks machine-import).

---

## Overlap & Ownership — canonical owners (summary)
Review lifecycle→**V15**; approval gates→**V12**; AI pre-review→**V20 (writes V15 finding records)**; connector framework→**V08**; real-time meeting capture→**V17**; diagram import→**V18**; scan analytics→**V09** / agent ops→**V17**; global-search UX→**V17** / index infra→**V11**; license tiers→**V17** / billing→**V13**; per-user adaptive→**V14** / portfolio-level→**V16**; bidirectional ALM sync→**V19**; compliance analysis→**V16** (consumes V09, fed by V20 horizon-scan); M&A analytics→reuse **V16/V09** with M&A-specific wrapper in **V20**. Most are already correctly bounded; the actions above are mostly **add cross-reference / boundary note**, plus the two genuine merges (import; license-tier definitions).

## Prioritization — recommended remedy
1. Add a **Phase** field/badge to requirements (Must↔Phase decoupled).
2. BUILD-PLAN cross-ref banner on Vols 01, 03, 11, 13, 16, 19.
3. The Phase-1 **Blended MVP (Epics A–H) is coherent and ~6–10 weeks** on the existing codebase. In-scope = partial slices of Vols 04,05,06,07,08,09,10,11,12,13,14. Deferred = Vols 01,02,03(canvas),15,16,17,18,19,20 + heavy parts of 08/09/11/13. (Full in/deferred table from the prioritization pass retained in the agents' findings.)

---

## Sign-off
- **Catalog coverage: FINAL** (20 volumes, ≈3,841 reqs).
- **Build-readiness: GO for Phase 1** — no MVP-blocking issue except **H3 (requirement convergence)**, which is promoted to a full Phase-1 task and the **C4** auto-apply/gate interaction to handle in Epic D.
- **Reconciliation backlog:** the P0–P2 items above. P0/P1 design-defining items are in **post-MVP** areas; resolve each before its phase. Recommended: apply the **catalog-level rulings** (done in the index) now; apply **volume-level cross-refs/merges/gap-adds** as a tracked reconciliation pass (can run as a workflow) — not a blocker for starting Phase 1.
