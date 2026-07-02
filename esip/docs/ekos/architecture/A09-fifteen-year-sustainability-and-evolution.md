# EKOS Architecture — A09: 15-Year Sustainability & Evolution Architecture

> **Status:** ✅ Done · **Item count:** 66 · **Generated:** 2026-07-01 (Opus)
> **Architecture Phase document E — THE longevity document.** Operationalizes the user's
> core mandate: *"this platform must sustain 15 years by regularly maintaining
> applications on latest versions."* Turns **Longevity (A01-011, Book 1 Law 14)** into
> concrete, automated policy: **dependency currency, upgrade automation, ABI deprecation
> windows, tech-radar cadence, fitness functions, EOL handling, and explicit 15-year
> guarantees with *how each is kept*.** Anchored by [A01](A01-architecture-vision-and-quality-attributes.md),
> [A03](A03-technology-selection.md) (choices), [A08](A08-deployment-infrastructure-architecture.md),
> Book 1 Law 14; ADR discipline via [ADR-REGISTER](../ADR-REGISTER.md) + [FEATURE-FREEZE](../FEATURE-FREEZE.md).

---

## 1. Purpose & Scope

Make **longevity a running, automated discipline**, not an aspiration. Specify how EKOS
stays current, upgradable, and coherent for 15+ years — and honestly, how a **solo builder
sustains it** via heavy automation. This is where "no regret later" becomes an operating
procedure.

---

## 2. Sustainability Principles

- **A09-001 · Longevity is a top-tier quality attribute** (A01-011), verified — not hoped —
  via fitness functions (§8).
- **A09-002 · Current-by-default** — dependencies are kept on supported/latest versions by
  an **automated cadence**, not a big-bang rewrite every few years (§4/§5).
- **A09-003 · Stable contracts, evolving internals** — the **syscall ABI, fact-sheet
  contract, public API, and KO model** are stable with governed deprecation; everything
  behind them modernizes freely (§6).
- **A09-004 · Standards-anchored** — standards (GQL/JSON-LD/OCI/CloudEvents/WebGPU/WASM/
  X.509/OpenTelemetry/SAML-OIDC-SCIM) outlive vendors; they are the durable bedrock (A03).
- **A09-005 · Reversibility** — every dependency behind a port; a dying/relicensed tech is
  a **swap, not a crisis** (§9).
- **A09-006 · Non-destructive evolution** — bitemporal history + self-hosting + no-lock-in
  export mean the **knowledge outlives every tool** (§11).
- **A09-007 · Solo-sustainable** — automation-first so one person can keep it current for
  15 years (§13).

---

## 3. The Three Longevity Layers (what makes each last)

- **A09-008** — Longevity is defended at three layers:

| Layer | What lasts | Mechanism |
|---|---|---|
| **Knowledge** (the data) | forever | bitemporal + self-hosting + JSON-LD/RDF export (Law 9/11/15) |
| **Contracts** (ABI/API/fact-sheet/KO model) | 15+ yrs, versioned | stable + governed deprecation windows (§6) |
| **Implementations** (tech choices) | continuously refreshed | ports + dependency currency + tech-radar (§4/§7/§9) |

*The genius of the design: the deeper the layer, the longer it lasts. Data is permanent;
contracts are stable; implementations churn safely behind them.* *Source:* Book 1 §4/§Laws.

---

## 4. Dependency-Currency Policy (ADR-044)

- **A09-009 · ADR-044** — Dependencies SHALL be kept **current by policy**:
  - **Security patches:** applied within **days** (automated, high-priority).
  - **Minor/patch versions:** **automated PRs, merged continuously** after CI + fitness
    functions pass.
  - **Major versions:** evaluated on a **scheduled cadence** (e.g., quarterly), upgraded
    behind ports with fitness-function verification.
  - **LTS discipline:** prefer LTS releases (.NET LTS, K8s stable, Node LTS) to reduce
    churn (A03).
  *Source:* Book 1 Law 14; A03. *15-year defense: continuous small upgrades beat periodic
  rewrites; the platform never falls dangerously behind.*

- **A09-010** — An **SBOM (Software Bill of Materials)** SHALL be maintained (e.g.,
  CycloneDX/SPDX — standards) with **continuous vulnerability scanning** (e.g., OSV/
  Dependabot/Trivy-class) feeding the currency pipeline. *Source:* supply-chain security
  norm; A06.

- **A09-011** — Dependency choices are constrained by the **A03 selection principles** —
  foundation-backed, permissive-licensed, standards-based — precisely so they *can* be kept
  current for 15 years. *Source:* A03-001…007.

---

## 5. Automated Upgrade Pipeline

- **A09-012** — An **automated dependency-update pipeline** (Renovate/Dependabot-class)
  SHALL open PRs for updates; CI runs tests + **fitness functions (§8)**; passing updates
  **auto-merge** (patch/minor) or queue for review (major). *Source:* A09-009; A01-045.

- **A09-013** — Every upgrade PR SHALL be **verified against the fitness functions** —
  longevity/security/perf/contract-stability are *proven still green* before merge, so
  upgrades don't silently break the architecture. *Source:* A01-045; §8.

- **A09-014** — Upgrades happen **behind ports** (A03/A07) — swapping a store/AI/IdP
  version doesn't touch the kernel/apps. *Source:* Book 1 Law 10.

- **A09-015** — For the **solo builder**, this pipeline is the force-multiplier: most
  maintenance is automated PRs + green checks, not manual toil. *Source:* A09-007;
  [[user-profile]].

---

## 6. Stable Contracts & Deprecation Windows (ADR-045)

- **A09-016 · ADR-045** — The stable contracts and their deprecation policy:

| Contract | Stability | Deprecation window |
|---|---|---|
| **Syscall ABI** (Vol 22) | versioned, additive-by-default | old ABI supported **≥ 2 major versions / ≥ 24 months** before removal, with warnings |
| **Fact-sheet contract** (A05) | versioned | same window as ABI |
| **Public API** (GraphQL, A07) | additive-evolution + versioned | deprecations flagged ≥ 12 months before removal |
| **KO Model** (Book 2) | append-only fields; reserved-fields policy | **never break**; new fields additive; reserved KWR fields already present |
| **Ontology types** | non-destructive | types deprecated (soft), never hard-removed (bitemporal) |

*Source:* Vol 22 §4; Book 2; A07; resolves OQ-18-D/22-B. *15-year defense: apps and
integrations built today keep working; nothing breaks silently; change is announced and
windowed.*

- **A09-017** — Deprecations SHALL be **announced, warned (runtime + docs), and windowed** —
  no surprise removals (Book 1 Law 12 no-silent-caps). *Source:* Book 1 Law 12.

---

## 7. Tech-Radar Cadence (ADR-046)

- **A09-018 · ADR-046** — A **Tech-Radar review** SHALL run on a **scheduled cadence (e.g.,
  every 6–12 months)** classifying every significant technology as **Adopt / Trial / Hold /
  Retire** — a deliberate, logged re-evaluation of choices against the A03 principles and
  the current landscape. *Source:* ThoughtWorks tech-radar practice; A03. *15-year defense:
  choices are re-examined regularly, not frozen-and-forgotten; regret is caught early.*

- **A09-019** — A "Retire" classification triggers a **planned migration** behind the
  relevant port (an ADR records the swap; the old is superseded, non-destructively).
  *Source:* §9; ADR discipline.

- **A09-020** — The radar explicitly watches **license changes** (the Terraform→OpenTofu
  lesson, ADR-029) — a relicense of a load-bearing dependency is a Retire trigger.
  *Source:* A03-004; ADR-029.

---

## 8. Fitness Functions (longevity is verified, not hoped)

- **A09-021** — **Automated architecture fitness functions** SHALL continuously verify the
  quality attributes (A01-045): **layering rules** (no illegal cross-layer calls),
  **contract stability** (ABI/API compatibility tests), **security invariants** (no
  bypass of the PDP), **performance budgets** (projection/render latency), **dependency
  currency** (nothing past EOL), and **vendor-neutrality** (no vendor name leaking as a
  requirement). *Source:* A01-045; evolutionary-architecture practice; Vol 05 §5.

- **A09-022** — Fitness functions run in **CI on every change + on every dependency
  upgrade** — a violation fails the build. This makes longevity/security **executable
  guarantees**, not documents. *Source:* A01-045; §5.

- **A09-023** — The **Continuous Architecture Fitness** governance capability (Vol 05 §5,
  200+ rules, drift detection) is the in-platform counterpart; the CI fitness functions are
  the build-time counterpart. Together: drift is caught at build and at runtime. *Source:*
  Vol 05 §5.

---

## 9. Reversibility & End-of-Life Handling

- **A09-024** — When a dependency is **abandoned, relicensed (BSL/SSPL), or EOL'd**, the
  response is a **port-adapter swap**, not a rewrite: implement a new adapter behind the
  existing port, verify with fitness functions, cut over. *Source:* Book 1 Law 10; A03-006.

- **A09-025** — The **reversibility map (A03 §5)** documents each choice's swap point and
  effort — so any EOL is a *known, bounded* migration, not an existential event. *Source:*
  A03 §5.

- **A09-026** — The **worst case (a foundational store dies)** is bounded by: the storage
  abstraction port + the **JSON-LD/RDF no-lock-in export** — data can be exported and
  re-imported into a new store. *Source:* A04 §15; Book 1 Law 9.

---

## 10. Breaking-Change Absorption

- **A09-027** — Breaking changes in a dependency SHALL be **absorbed inside its adapter** —
  the port's interface stays stable, so the kernel/apps never see the break. *Source:*
  Book 1 Law 10.

- **A09-028** — Because **apps depend only on the stable syscall ABI (A09-016)**, platform-
  internal modernization (even major store/AI/framework swaps) **does not break apps** —
  the central 15-year promise for the app ecosystem. *Source:* Vol 22 §4; Vol 23 §12.

---

## 11. Knowledge & Specification Longevity

- **A09-029** — The **knowledge itself outlives every tool**: bitemporal history (never
  destroyed), **self-hosting** (EKOS describes itself, Vol 25), and **one-click JSON-LD/RDF
  export + offline player** (Book 1 Law 9) guarantee the data survives even EKOS's own
  demise. *Source:* Book 1 Laws 9/11/15; Vol 25; A04 §15.

- **A09-030** — The **specification evolves as versioned, time-travellable knowledge**
  (self-hosted, Vol 25 §10) — the *why* (ADRs, Book 1/2, methodology) is preserved so
  future maintainers understand every decision. *Source:* Vol 25 §10; ADR-REGISTER.

---

## 12. Talent & Maintainability

- **A09-031** — Standards + mainstream tech (A03) = a **durable hiring pool** — .NET/
  TypeScript/Rust/K8s/Postgres/GraphQL talent will exist for decades. *Source:* A03-005.

- **A09-032** — **Maintainability by design**: minimal microkernel, clean bounded contexts,
  boundary-discipline docs, and the **self-hosted spec as onboarding** (Vol 25 §11) — a new
  maintainer learns EKOS by exploring EKOS. *Source:* Vol 18 §2; Vol 25 §11.

---

## 13. Solo-Sustainability

- **A09-033** — The whole sustainability model is tuned for a **solo/after-hours builder**:
  automated upgrades (§5), fitness functions catching regressions (§8), managed services at
  MVP (A08-037), and scale-with-adoption (ADR-043) — so **one person can keep EKOS current
  and running for 15 years** without a large team. *Source:* [[user-profile]]; A08-037;
  A09-007.

- **A09-034** — Where automation isn't enough, the **tech-radar cadence (§7)** is a small,
  scheduled, deliberate human review — bounded effort, not constant firefighting. *Source:*
  §7.

---

## 14. The Modernization Loop (the recurring cadence)

- **A09-035** — The sustainability process is a **recurring loop**:

```
   [continuous]  Vulnerability scan + automated dependency PRs → CI + fitness functions → auto-merge
        │
   [monthly]     Review flagged major-version PRs; run fitness suite
        │
   [6–12 months] Tech-Radar review (Adopt/Trial/Hold/Retire) → log ADRs → plan any Retire migrations
        │
   [as needed]   EOL/relicense event → port-adapter swap → verify → cut over → supersede ADR
        │
   [always]      Deprecation windows honored; knowledge exportable; spec self-hosted
        └──────► loop
```

*Source:* §4–§10 synthesis. *This loop is "regularly maintaining on latest versions" made
operational.*

---

## 15. The 15-Year Guarantees (explicit — with HOW each is kept)

- **A09-036** — EKOS makes these longevity guarantees:

| # | Guarantee | How it is kept |
|---|---|---|
| G1 | **Dependencies stay current** | automated update pipeline + SBOM + vuln scanning (§4/§5) |
| G2 | **Apps built today keep working** | stable syscall ABI + deprecation windows (§6) |
| G3 | **A dying/relicensed tech is not a crisis** | ports + reversibility map + adapter swap (§9) |
| G4 | **Knowledge survives even EKOS** | bitemporal + self-hosting + JSON-LD/RDF export (§11) |
| G5 | **No silent breakage** | announced/windowed deprecations + fitness functions in CI (§6/§8) |
| G6 | **Choices are re-examined, not frozen** | tech-radar cadence + ADR discipline (§7) |
| G7 | **One person can sustain it** | automation-first + managed MVP + scale-with-adoption (§13) |
| G8 | **The "why" is never lost** | ADR Register + self-hosted spec + Books 1/2 (§11) |
| G9 | **No vendor lock-in ever** | pluggable ports + standards + no-lock-in export (A03/A07) |
| G10 | **Regret is caught early** | fitness functions + tech-radar + license-watch (§7/§8) |

*Source:* the whole document; Book 1 Law 14.

---

## 16. Governance of Evolution

- **A09-037** — Evolution SHALL be governed by: the **ADR Register** (every decision + why
  + superseded-by), the **Feature Freeze** (no new pillars unless category-changing), and
  **Book 1 amendment discipline** (laws change only by logged amendment). *Source:*
  ADR-REGISTER; FEATURE-FREEZE; Book 1 Part III.

- **A09-038** — New capabilities pass the **Design Methodology** (capability-vs-feature, 5
  lenses) before entering — protecting simplicity/consistency/implementability over 15
  years. *Source:* DESIGN-METHODOLOGY.

---

## 17. Constitution (Book 1) Conformance

- **A09-039** — This document **is** the operationalization of **Law 14 (designed for 15
  years)**, supported by **Law 9 (no-lock-in)**, **Law 10 (pluggable/standards)**, **Law 11
  (non-destructive)**, **Law 15 (self-hosting)**. *Source:* Book 1 Part II.

---

## 18. Open Questions (parked)

- **OQ-A09-A** — Exact major-version review cadence + tech-radar frequency (quarterly vs
  biannual) — tune with real maintenance load.
- **OQ-A09-B** — ABI/API deprecation window lengths (the §6 numbers are initial) — confirm
  with early adopters (carries OQ-18-D/22-B).
- **OQ-A09-C** — Which fitness functions ship in Phase 1 vs later (A01 OQ-A01-B) — with A11.
- **OQ-A09-D** — SBOM/scanning toolchain pick (CycloneDX vs SPDX; OSV/Trivy/Dependabot) —
  A03-style choice, standards-based.
- **OQ-A09-E** — Solo-capacity realism: how much maintenance load is sustainable after-
  hours, and what that implies for scope pacing (A11). *Honest planning input.*

---

## 19. Item Index (this document)

A09-001 … A09-039 (sustainability principles, three longevity layers, **dependency-currency
ADR-044**, automated upgrade pipeline, **deprecation-windows ADR-045**, **tech-radar ADR-046**,
fitness functions, reversibility/EOL, breaking-change absorption, knowledge/spec longevity,
talent/maintainability, solo-sustainability, the modernization loop, **the 15-year
guarantees table**, evolution governance, conformance) — expanding to 66 distinct stated
items. New ADRs **044 (currency), 045 (deprecation windows), 046 (tech-radar)** appended.

> **Next architecture doc:** `A10 — ESIP → EKOS Migration Architecture` (the existing app
> as an EKOS application; strangler-fig path).
> Per protocol: STOP here. Await user "proceed".
