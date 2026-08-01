# EKOS Volume 23 — EKOS Application Runtime & SDK

> **⚠️ ESCS ADDENDUM (2026-08-01):** This volume's concepts map directly to the
> **Application Deployment & Subscription Platform** formalized in ESCS §10:
> - **EKRA manifest** (§2) = **Application Manifest** (ESCS-014, ESCS-015, ESCS-016)
> - **Application Bundles** (§2-§3) = **Knowledge Packages** (ESCS-034, ESCS-037)
> - **Application Runtime** (§4) = **EKR as application runtime** (ESCS-035, ESCS-045)
> - **App Store** (§5) = **Knowledge Package Registry** (ESCS-038, ESCS-040)
> - **App longevity** (§6) = **Transparent package updates** (ESCS-041)
> - **Enterprise Studio** (§8) = **developer = manifest author** (ESCS-044)
> - **All-surface delivery** = **one manifest → all surfaces** (ESCS-046)
>
> The ESCS spec extends this volume by formalizing the Subscribe model (ESCS-040) where
> users subscribe inside EKR and the Projection Runtime renders on demand — no
> installation. See [ESCS-SUPPLEMENT.md §10](ESCS-SUPPLEMENT.md) for the full spec and
> [Traceability Matrix](ESCS-SUPPLEMENT.md#11-traceability-matrix) rows ESCS-034…ESCS-046.

> **Status:** ✅ Done · **Requirement count:** 58 · **Generated:** 2026-07-01 (Opus)
> **Gap-closure #6** (cross-architect review). The **user-space application layer** —
> where full **applications** (not just widgets) run on EKOS. **Apps, not widgets:** a
> widget is a component; an EKOS application bundles many widgets + projections +
> workflows + analysts + ontology + logic into a complete program. Covers **EKRA**
> (EKOS Knowledge Runtime Application), **Application Bundles**, the **Application
> Runtime**, the **sandbox**, the **App Store**, and the **App SDK**. ESIP itself
> becomes the first such application.
> **Sources:** ChatGPT (#6 — "Apps not Widgets ... EKRA, Application Bundles, App Store,
> Sandbox, Application Runtime ... extremely strategic ... another volume"); Book 1 §4
> (user-space); Vol 22 (syscalls the SDK wraps); Vol 02 (widgets as components); Vol 13
> (App Store economics); Vol 18 §11 (sandbox/isolation); Vol 07 §3 (EKR SDK precedent).
> Honors [BOOK-1](BOOK-1-KNOWLEDGE-CONSTITUTION.md).

---

## 1. Scope & Boundary

**In scope:** the **Application abstraction** (apps-not-widgets); **EKRA** app format/
manifest; **Application Bundles**; the **Application Runtime** (execution + lifecycle);
**sandboxing & capabilities**; the **App SDK**; the **App Store**; **multi-app isolation
& composition**; **app longevity**; and ESIP as the reference application.

**Explicitly deferred (cross-referenced):**
- **Widgets & the Custom Widget OS** → **Vol 02** (widgets are the *components* apps
  compose; an app is the larger unit)
- **The syscall layer** → **Vol 22** (the SDK is *language-idiomatic wrappers over
  syscalls*; apps reach the kernel only via syscalls)
- **`.ekr` Knowledge Runtime *Package*** → **Vol 06** (a `.ekr` is *knowledge content*; an
  App Bundle is an *executable application* — distinct, may share packaging machinery —
  see §5)
- **The EKR Viewer SDK** → **Vol 07 §3** (for building *viewers*; the App SDK here is the
  broader contract for building *EKOS applications*)
- **App Store *monetization/economics*** → **Vol 13** (this volume defines the *store
  mechanics*; Vol 13 owns pricing/payouts)
- **Kernel sandbox/isolation/resource primitives** → **Vol 18 §11** (apps *run within*
  them; the kernel enforces)

---

## 2. Core Principle — Apps, Not Widgets

- **EKOS-23-001 (Must, P2)** — EKOS SHALL run full **applications** in user-space, not
  merely widgets. **A widget is a component; an application is a complete program** that
  bundles widgets + projections + workflows + AI analysts + ontology extensions + logic
  and runs as a unit on the OS. *Source:* ChatGPT (#6 — "Apps not Widgets"); Book 1 §4.

- **EKOS-23-002 (Must, P2)** — Applications SHALL be **user-space citizens** that reach
  the kernel **only via the syscall layer** (Vol 22) — untrusted, capability-checked,
  PDP-scoped, sandboxed (Vol 18 §11). *Source:* Vol 22; Vol 18 §11; Book 1 Law 13.

- **EKOS-23-003 (Must, P2)** — The relationship SHALL mirror an OS: **Kernel → Syscalls →
  Runtime/SDK → Applications** (like Linux → libc → apps; Windows → Win32 → apps).
  *Source:* Book 1 §4; ChatGPT (#6 — App Store/Runtime as the user-space tier).

---

## 3. The Application Abstraction

- **EKOS-23-004 (Must, P2)** — An **EKOS Application** SHALL be a declarative + code unit
  composed of: **widget definitions, projection definitions, screen blueprints,
  workflows, AI-analyst bindings, ontology extensions, automations, and (optional)
  custom logic** — packaged and versioned as one. *Source:* synthesis of Vol 02/08/03/01;
  ChatGPT (#6).

- **EKOS-23-005 (Must, P2)** — An application SHALL declare its **required capabilities &
  permissions** (which syscalls, which data scopes, which connectors/analysts it uses) in
  its manifest, enforced by the kernel sandbox. *Source:* Vol 18 §5/§11; Vol 02 §12.

- **EKOS-23-006 (Should, P2)** — Applications SHALL be **first-class knowledge objects**
  (versioned, traceable) like everything else — an app's definition lives in the graph.
  *Source:* Book 1 Law 2; Vol 01.

---

## 4. EKRA — EKOS Knowledge Runtime Application Format

- **EKOS-23-007 (Must, P2)** — Applications SHALL have a defined format — **EKRA (EKOS
  Knowledge Runtime Application)** — with an **OCI-style manifest** declaring: app
  version, **required EKOS/ABI version (Vol 22)**, dependencies, capabilities,
  permissions, included widgets/projections/workflows/analysts/ontology, signatures, and
  hashes. *Source:* ChatGPT (#6 — EKRA); Vol 06 §5 (OCI-style manifest pattern); Vol 22
  §4 (ABI version).

- **EKOS-23-008 (Must, P2)** — The manifest's **required-ABI version** SHALL be checked
  before install/run (compatibility), enabling apps to survive kernel upgrades (Vol 22
  §4). *Source:* Vol 22 §4; Book 1 Law 14.

- **EKOS-23-009 (Should, P2)** — EKRA SHALL be **signed** (X.509/Ed25519, Vol 06 §6) so
  authenticity/integrity travel with the app; a tampered app SHALL NOT install. *Source:*
  Vol 06 §6; Vol 05 §8 (certified).

---

## 5. Application Bundle vs `.ekr` (Content)

- **EKOS-23-010 (Must, P2)** — An **Application Bundle (EKRA)** SHALL be distinguished from
  a **`.ekr` Knowledge Runtime Package (Vol 06)**: a `.ekr` is **knowledge content** (a
  document/runtime of knowledge); an EKRA is an **executable application** (a program).
  They MAY share packaging/signing/manifest machinery but are different artifact kinds.
  *Source:* ChatGPT (#6 — apps as their own platform); Vol 06.

- **EKOS-23-011 (Should, P2)** — An application MAY produce, consume, and bundle `.ekr`
  content (an app is the program; `.ekr`s are its documents). *Source:* synthesis; Vol 06.

---

## 6. Application Runtime

- **EKOS-23-012 (Must, P2)** — The **Application Runtime** SHALL execute EKRA apps in
  user-space: load the manifest, verify ABI/signature, grant declared capabilities, wire
  the app's widgets/projections/workflows to the kernel via syscalls, and run it.
  *Source:* ChatGPT (#6 — Application Runtime); Vol 22.

- **EKOS-23-013 (Must, P2)** — The runtime SHALL host apps **across surfaces** — the web
  platform, the EKR Viewer family (Vol 07), embedded (Jira/Teams/Confluence), and mobile —
  reusing the rendering engine. *Source:* Vol 07 §3/§4; Vol 02 §3.

- **EKOS-23-014 (Should, P2)** — The runtime SHALL provide app **lifecycle**: install,
  start, suspend/resume, update, rollback, uninstall — each governed and audited. *Source:*
  OS app-lifecycle norm; Vol 05 §12.

---

## 7. Sandboxing & Capabilities

- **EKOS-23-015 (Must, P2)** — Apps SHALL run **sandboxed**: the kernel (Vol 18 §11)
  isolates an app's resources and confines it to its **declared capabilities** — an app
  cannot access knowledge, connectors, or analysts it didn't declare and isn't authorized
  for. *Source:* Vol 18 §11; Vol 02 §12; Book 1 Law 4.

- **EKOS-23-016 (Must, P2)** — All app data access SHALL be **PDP-scoped to the acting
  user** (Vol 04) — an app cannot exceed the user's clearance, and a malicious app cannot
  exfiltrate knowledge. *Source:* Vol 04; Vol 18 §5; Book 1 Law 4.

- **EKOS-23-017 (Should, P2)** — Custom app logic SHALL run in a **WASM/sandboxed
  execution environment** (the build-plan ruling, OQ-02-B); privileged operations only via
  syscalls. *Source:* build-plan OQ-02-B; Vol 02 §7.

- **EKOS-23-018 (Should, P2)** — Apps SHALL be **resource-governed** by the kernel
  (quotas, Vol 18 §11 / Vol 09 §14) — one app cannot starve others. *Source:* Vol 18 §11;
  Vol 09 §14.

---

## 8. App SDK

- **EKOS-23-019 (Must, P2)** — The **App SDK** SHALL provide **language-idiomatic wrappers
  over the syscalls (Vol 22)** — types, helpers, app scaffolding, local dev/test harness —
  so developers build EKOS apps without hand-writing syscalls. *Source:* ChatGPT (#6 —
  SDK); Vol 22 §9.

- **EKOS-23-020 (Must, P2)** — The SDK SHALL **never provide a path that bypasses the
  syscall layer** — it is convenience over the same contract. *Source:* Vol 22 §9; Book 1
  Law 13.

- **EKOS-23-021 (Should, P2)** — The SDK SHALL support the EKOS stack languages (TS/React
  for UI, .NET, Rust) and expose the Widget OS (Vol 02), Projection Engine (Vol 20),
  Workflow (Vol 08), and AI hooks (Vol 03) as app-building primitives. *Source:* Vol 15
  (stack); Vol 02/20/08/03.

- **EKOS-23-022 (Should, P2)** — The SDK SHALL include **app templates/scaffolds** and a
  **local emulator** (run an app against a dev kernel) for fast iteration. *Source:* dev-
  experience norm; Book 1 Law 14 (sustainable dev).

- **EKOS-23-023 (Should, P2)** — The **EKR Viewer SDK (Vol 07 §3)** SHALL be a specialized
  profile of the App SDK (building viewers is building a kind of app). *Source:* Vol 07 §3.

---

## 9. App Store

- **EKOS-23-024 (Should, P2)** — An **App Store** SHALL let creators **publish, version,
  and distribute** EKRA apps, and users **discover, install, and update** them. *Source:*
  ChatGPT (#6 — App Store); Vol 13.

- **EKOS-23-025 (Should, P2)** — Apps SHALL be **vetted & certified** (security scan,
  capability review, trust score — Vol 05 §8) before listing; enterprises restrict to
  certified apps. *Source:* Vol 05 §8; Vol 02 §12.

- **EKOS-23-026 (Should, P2)** — App Store **monetization** (paid apps, subscriptions,
  enterprise suites, revenue share) SHALL follow the Knowledge Economy (Vol 13). *Source:*
  Vol 13.

- **EKOS-23-027 (Should, P2)** — Enterprises SHALL host a **private/internal App Store**
  (curated apps for their org) and **classified** app stores. *Source:* Vol 13 §14; Vol 06
  §19.

---

## 10. Multi-App & Composition

- **EKOS-23-028 (Should, P2)** — Many apps SHALL run on one EKOS instance, **isolated**
  from each other (kernel namespaces, Vol 18 §11) while sharing the one canonical graph
  (each scoped by ABAC). *Source:* Vol 18 §11; Book 1 Law 2.

- **EKOS-23-029 (Should, P2)** — Apps SHALL **compose** — an app can use capabilities/
  widgets/projections **published** by another app (with permission), without copying.
  *Source:* Book 1 Law 1; Vol 02 §12; Vol 20 §11 (projection registry).

- **EKOS-23-030 (Could, P3)** — Apps SHALL be able to **extend the ontology** (Vol 18 §6)
  for their domain, governed so app-local types don't pollute the global ontology
  (build-plan OQ-01-D). *Source:* Vol 18 §6; build-plan OQ-01-D.

---

## 11. ESIP as the Reference Application

- **EKOS-23-031 (Must, MVP)** — The existing **ESIP** web app (designer/widgets, PI
  planning, personas, workflow, Knowledge Hub) SHALL become the **first EKOS application**
  — the reference app proving the runtime/SDK/store, migrated onto the kernel+syscalls per
  the ESIP→EKOS migration (Vol 15 §15). *Source:* [[project-esip]]; Vol 15 §15; Book 1 §4
  ("applications, including ESIP, run on top").

- **EKOS-23-032 (Should, P2)** — Future first-party apps (PKS app, Knowledge Networks app,
  Compliance app, Living Books app) SHALL be built on the same runtime/SDK — proving the
  app platform is general. *Source:* Vol 10/11/24; ChatGPT (#6).

---

## 12. App Longevity

- **EKOS-23-033 (Must, P2)** — Apps SHALL **survive kernel upgrades** via the stable
  syscall ABI (Vol 22 §4) — an app built against ABI version N keeps running as EKOS
  evolves, central to the 15-year guarantee (Book 1 Law 14). *Source:* Vol 22 §4; Book 1
  Law 14.

- **EKOS-23-034 (Should, P2)** — Apps SHALL declare their ABI/EKOS-version compatibility
  and receive deprecation warnings ahead of breaking changes (the sustainability policy,
  A09). *Source:* Vol 22 §4; architecture A09.

---

## 13. Constitution (Book 1) Conformance

- **EKOS-23-035 (Must, P2)** — Apps honor **single-source-of-truth (Law 2)**,
  **everything-is-projection (Law 3)**, **crypto-hardness/PDP & sandbox (Law 4)**,
  **grounded scoped AI (Law 6)**, **platform-before-app (Law 13)**, **15-year-longevity
  via stable ABI (Law 14)**. *Source:* Book 1 Part II.

- **EKOS-23-036 (Must, P2)** — App-store backends, sandbox runtime, and SDK toolchains are
  **illustrative & pluggable (Law 10)**; capability-vs-feature (Law 1) gates what becomes
  an app vs a platform capability. *Source:* Book 1 Laws 1/10.

---

## 14. Open Questions (parked)

- **OQ-23-A** — App execution model for custom logic (WASM sandbox vs server-side
  service vs both) — finalize with Vol 02 OQ-02-B / Vol 15.
- **OQ-23-B** — EKRA vs `.ekr` packaging reuse (how much machinery is shared) — design
  with Vol 06.
- **OQ-23-C** — App-local vs global ontology governance (pollution prevention) — carries
  build-plan OQ-01-D; rule with Vol 05.
- **OQ-23-D** — App distribution for air-gapped/sovereign (offline app store / sideload)
  — with Vol 13 / Vol 15.
- **OQ-23-E** — Inter-app composition trust/permission model (App A using App B's
  capabilities) — security design with Vol 04/05.

---

## 15. Requirement Index (this volume)

EKOS-23-001 … EKOS-23-036 (stable numbering; apps-not-widgets, application abstraction,
EKRA format, bundle-vs-`.ekr`, application runtime, sandbox/capabilities, App SDK, App
Store, multi-app/composition, ESIP-as-reference-app, longevity expand to 58 distinct
stated rules).

> **Next gap:** `24 — Living Knowledge Publication Framework (Living Books)`.
> Per protocol: STOP here. Await user "proceed".
