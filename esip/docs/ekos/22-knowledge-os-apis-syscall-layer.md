# EKOS Volume 22 — Knowledge OS APIs / Syscall Layer

> **Status:** ✅ Done · **Requirement count:** 58 · **Generated:** 2026-07-01 (Opus)
> **Gap-closure #5** (cross-architect review). The **stable contract across the kernel↔
> user-space boundary** — EKOS's equivalent of **Win32 / POSIX / Linux syscalls**:
> `Knowledge.Open / Query / Project / Validate / Commit / Publish / Package / Subscribe /
> Observe / Acquire / Authorize / Extend / Transact`. Every user-space subsystem
> (Compiler, Projection Engine, Widget OS, Acquisition, Viewer, Apps) reaches the kernel
> **only** through this layer. It is the foundation the **App SDK (Vol 23)** and the
> **public API (Vol 09 §12)** are built on.
> **Sources:** ChatGPT (#5 — "where is the Operating System API? ... your equivalent of
> Win32/POSIX/Linux Syscalls. Huge missing abstraction"); Vol 18 §12 (the boundary);
> POSIX/Win32 syscall-ABI pattern; Book 1 Law 13/14. Honors
> [BOOK-1](BOOK-1-KNOWLEDGE-CONSTITUTION.md).

---

## 1. Scope & Boundary

**In scope:** the **syscall surface** (the complete call catalog), each call family's
contract, the **stable ABI & versioning** policy, **language-agnostic binding**, **sync/
async/streaming** semantics, the **error model**, **per-call capability & PDP
enforcement**, and the relationship to the SDK and public API.

**Explicitly deferred (cross-referenced):**
- **The kernel itself** (subsystems behind the calls) → **Vol 18** (this volume *is the
  boundary's interface*; Vol 18 implements behind it)
- **The App SDK** (language-idiomatic wrappers, app dev ergonomics) → **Vol 23** (built
  *on* these syscalls)
- **The public GraphQL API & webhooks** (external third-party integration) → **Vol 09
  §12** (a user-space service built *on* syscalls; different audience — external systems
  vs internal OS clients)
- **What each call *does* downstream** — query semantics (Vol 01), projection (Vol 20),
  compilation (Vol 19), acquisition (Vol 21), ABAC (Vol 04), governance (Vol 05) — owned
  by those volumes; this volume defines the **call signatures, guarantees, and ABI**.

---

## 2. Core Principle — The Stable OS Contract

- **EKOS-22-001 (Must, MVP)** — The syscall layer SHALL be EKOS's **operating-system API**
  — the single, stable, language-agnostic contract through which all user-space code
  reaches the kernel, analogous to Win32 / POSIX / Linux syscalls. *Source:* ChatGPT
  (#5); Vol 18 §12.

- **EKOS-22-002 (Must, MVP)** — It SHALL be the **only** path across the kernel↔user-
  space boundary (Vol 18 §12) — no subsystem or app bypasses it to touch knowledge
  directly. *Source:* Vol 18 §12; Book 1 Law 2.

- **EKOS-22-003 (Must, MVP)** — Every syscall SHALL be **capability-checked and PDP-
  scoped** by the kernel (Vol 18 §5/§10) — user-space is untrusted; there is no
  privileged escape via the API. *Source:* Vol 18 §5/§10; Book 1 Law 4.

- **EKOS-22-004 (Must, MVP)** — The syscall layer SHALL be the **foundation** on which the
  App SDK (Vol 23) and the public integration API (Vol 09 §12) are built — both are
  higher-level wrappers, never alternatives that bypass it. *Source:* ChatGPT (#5); Vol 23;
  Vol 09 §12.

---

## 3. The Syscall Surface (Call Catalog)

> The complete OS API, grouped. ChatGPT's named calls are the core; the set is rounded
> out to a coherent OS surface. Names are illustrative of the *contract*, not a final
> language binding.

- **EKOS-22-005 (Must, MVP)** — **Lifecycle / context:** `Open()` (open a knowledge
  handle/session/context, establishing principal + scope), `Close()` (release). *Source:*
  ChatGPT (#5 — Open); POSIX open/close.

- **EKOS-22-006 (Must, MVP)** — **Read:** `Query()` (declarative graph query/traversal,
  PDP-scoped; returns a subgraph — Vol 01 §11). *Source:* ChatGPT (#5 — Query); Vol 01.

- **EKOS-22-007 (Must, MVP)** — **Projection:** `Project()` (request a live projection /
  view via the Projection Engine — Vol 20). *Source:* ChatGPT (#5 — Project); Vol 20.

- **EKOS-22-008 (Must, MVP)** — **Write:** `Commit()` / `Mutate()` (create/update/state-
  transition/soft-delete knowledge atomically via the KOM — Vol 18 §3; bitemporal).
  *Source:* Vol 18 §3; Book 1 Law 11. *(The write syscall ChatGPT implied alongside the
  read/project calls.)*

- **EKOS-22-009 (Must, MVP)** — **Validation:** `Validate()` (check objects/changes
  against ontology + policy/compliance gates — Vol 18 §6, Vol 05). *Source:* ChatGPT (#5
  — Validate); Vol 18 §6; Vol 05.

- **EKOS-22-010 (Must, P2)** — **Packaging/Build:** `Package()` / `Compile()` (invoke the
  Compiler to produce an artifact — `.ekr`/PDF/Living-Book/etc. — Vol 19). *Source:*
  ChatGPT (#5 — Package); Vol 19.

- **EKOS-22-011 (Must, P2)** — **Publish/Share:** `Publish()` (publish/share to the
  registry, marketplace, or a knowledge network — Vol 06/11/13). *Source:* ChatGPT (#5 —
  Publish); Vol 06/11/13.

- **EKOS-22-012 (Must, MVP)** — **Reactive:** `Subscribe()` (subscribe to mutation/events
  within scope) and `Observe()` (observe live state/metrics within scope) — backed by the
  kernel mutation journal (Vol 18 §8). *Source:* ChatGPT (#5 — Subscribe/Observe); Vol 18
  §8; Vol 20 §7.

- **EKOS-22-013 (Should, P2)** — **Acquisition:** `Acquire()` (feed knowledge in via the
  Acquisition Framework — Vol 21). *Source:* Vol 21; Book 1 §4.

- **EKOS-22-014 (Should, P2)** — **Authorization/Capability:** `Authorize()` / capability
  queries (check/request capabilities; the PDP decision surface — Vol 04/Vol 18 §5).
  *Source:* Vol 04; Vol 18 §5.

- **EKOS-22-015 (Should, P2)** — **Ontology:** `Extend()` (define/extend ontology types
  at runtime, governed — Vol 18 §6, Vol 01 §9). *Source:* Vol 18 §6; Vol 01 §9.

- **EKOS-22-016 (Should, P2)** — **Transaction:** `Transact()` / begin-commit-rollback
  (atomic multi-object transaction boundaries — Vol 18 §7). *Source:* Vol 18 §7.

- **EKOS-22-017 (Should, P2)** — The surface SHALL be **complete enough that every higher
  subsystem (compiler/projection/widgets/acquisition/viewer/apps) needs nothing outside
  it** — if a subsystem needs a new kernel capability, it is added as a governed syscall,
  not a backdoor. *Source:* Book 1 Law 13; POSIX-completeness principle.

---

## 4. Stable ABI & Versioning (15-Year Longevity)

- **EKOS-22-018 (Must, MVP)** — The syscall layer SHALL be a **stable, versioned ABI-like
  contract**; apps and subsystems compiled against version N keep working as the kernel
  evolves — central to 15-year longevity (Book 1 Law 14). *Source:* Vol 18 §12; Book 1
  Law 14.

- **EKOS-22-019 (Must, P2)** — Breaking changes SHALL follow a **deprecation policy**: a
  new ABI version is added; the old is supported for a defined window with warnings before
  removal (the policy detailed in the 15-year sustainability architecture, A09). *Source:*
  Book 1 Law 14; architecture A09 (forward ref); Vol 18 OQ-18-D.

- **EKOS-22-020 (Must, P2)** — The ABI SHALL be **explicitly versioned and negotiated** at
  `Open()` (client states the ABI version it targets; kernel honors or rejects with a
  clear error). *Source:* POSIX/Win32 versioning norm; analogy: ABI negotiation.

- **EKOS-22-021 (Should, P2)** — Additive changes (new optional syscalls/params) SHALL be
  **backward-compatible by default**; only removals/semantic-changes are "breaking."
  *Source:* semver/ABI norms.

---

## 5. Language-Agnostic Binding

- **EKOS-22-022 (Must, P2)** — The syscall contract SHALL be **language-agnostic** —
  callable from .NET, Rust, TypeScript/JS, Python, etc. — like POSIX is language-neutral.
  *Source:* ChatGPT (#5 — POSIX/Win32 analogy); Vol 15 (hybrid .NET+Rust+React stack).

- **EKOS-22-023 (Should, P2)** — The contract SHALL be defined in a **neutral interface
  description** (e.g., a schema/IDL — illustrative, not vendor-bound) from which language
  bindings are generated. *Source:* IDL/ABI pattern; Book 1 Law 10.

- **EKOS-22-024 (Should, P2)** — Transport SHALL be **pluggable** (in-process call,
  IPC/gRPC, network) so the same syscalls work whether the caller is in-process, a
  sidecar, or remote — without changing the contract. *Source:* Book 1 Law 10; Vol 12
  (transport-pluggable precedent).

---

## 6. Sync / Async / Streaming Semantics

- **EKOS-22-025 (Must, P2)** — Calls SHALL support **synchronous** (request/response),
  **asynchronous** (long-running compile/acquire/project), and **streaming** (Subscribe/
  Observe, streaming acquisition) modes with consistent semantics. *Source:* synthesis;
  Vol 21 (streaming), Vol 20 §7 (reactive).

- **EKOS-22-026 (Should, P2)** — Long-running calls (`Package`/`Compile`, bulk `Acquire`)
  SHALL be **cancellable** and **progress-observable** via `Observe()`. *Source:* Vol 19;
  Vol 21; UX.

---

## 7. Error Model

- **EKOS-22-027 (Must, P2)** — The layer SHALL have a **consistent, documented error model**
  (an `errno`-like taxonomy): permission-denied (PDP/capability), not-found, validation-
  failed, conflict (concurrency), quota-exceeded, ABI-version-mismatch, etc. *Source:*
  POSIX errno pattern; Vol 18 §5/§6/§11.

- **EKOS-22-028 (Must, P2)** — Permission errors SHALL **not leak existence** of
  unauthorized knowledge (a restricted object returns "not found / not authorized" without
  revealing it exists — consistent with Vol 04 §7 "no hint of content"). *Source:* Vol 04
  §7; Book 1 Law 4.

- **EKOS-22-029 (Should, P2)** — Errors SHALL be **actionable & localized** (i18n, Vol 09
  §3) and carry correlation IDs for tracing. *Source:* Vol 09 §3; observability.

---

## 8. Per-Call Capability & PDP Enforcement

- **EKOS-22-030 (Must, MVP)** — **Every** syscall SHALL be mediated by the kernel's
  enforcement module (Vol 18 §5): capability check + PDP scoping, on every call, with no
  fast-path that bypasses it. *Source:* Vol 18 §5; Book 1 Law 4.

- **EKOS-22-031 (Must, P2)** — Syscalls SHALL execute under the calling **Principal** (Vol
  18 §10) — human/AI/connector/app — so AI and apps are scoped exactly like humans (no
  privileged AI/app). *Source:* Vol 18 §10; Book 1 Law 6.

- **EKOS-22-032 (Must, P2)** — Security-relevant syscalls (`Commit`, `Publish`, `Extend`,
  `Authorize`) SHALL emit immutable audit events (Vol 18 §5, Vol 05 §12). *Source:* Vol 18
  §5; Vol 05 §12.

---

## 9. Relationship to SDK & Public API

- **EKOS-22-033 (Must, P2)** — The **App SDK (Vol 23)** SHALL be language-idiomatic
  wrappers over these syscalls (ergonomics, types, helpers) — never a parallel path to
  the kernel. *Source:* Vol 23; ChatGPT (#6 SDK).

- **EKOS-22-034 (Should, P2)** — The **public GraphQL API & webhooks (Vol 09 §12)** SHALL
  be a **user-space service built on the syscalls** — same ABAC, exposed to external
  systems; the syscall layer is internal-OS, the public API is external-integration.
  *Source:* Vol 09 §12; ChatGPT (#5).

- **EKOS-22-035 (Should, P2)** — All EKOS subsystems (Compiler/Projection/Widget-OS/
  Acquisition/Viewer) SHALL use the syscalls — proving the surface is complete and
  dog-fooded. *Source:* §3; Book 1 Law 13.

---

## 10. Observability

- **EKOS-22-036 (Should, P2)** — `Observe()`/`Subscribe()` SHALL power user-space
  monitoring, analytics (Vol 09 §9), the Chain-of-Thought visualizer (Vol 03 §14), and
  reactive projections (Vol 20 §7) — all **within the caller's PDP scope**. *Source:*
  Vol 09 §9; Vol 03 §14; Vol 20 §7.

- **EKOS-22-037 (Should, P2)** — Syscall telemetry SHALL feed standard observability
  (e.g., OpenTelemetry — illustrative) for latency/error/quota monitoring. *Source:*
  Vol 09 §14; Book 1 Law 10.

---

## 11. Self-Hosting & Constitution Conformance

- **EKOS-22-038 (Could, P3)** — The syscall surface SHALL itself be **described as
  knowledge** (the OS API is a knowledge object), introspectable via `Query()` — EKOS
  describes its own API (Vol 25). *Source:* Book 1 Law 15; Vol 25.

- **EKOS-22-039 (Must, MVP)** — The layer honors **single-source-of-truth (Law 2),
  crypto-hardness/PDP (Law 4), grounded scoped AI (Law 6), platform-before-app (Law 13),
  and stable-ABI-for-longevity (Law 14)**; the IDL/transport are **illustrative &
  pluggable (Law 10)**. *Source:* Book 1 Part II.

---

## 12. Constitutional Compliance Runtime (CCR) Gate

> **Added 2026-07-03 (ADR-051).** The CCR is the cross-cutting enforcement layer that
> makes Book 1 Laws binding at runtime. It intercepts every syscall on every caller path.
> Specification: Book 1 Part III. This section states the syscall-layer requirements.

- **EKOS-22-040 (Must, MVP)** — A **Constitutional Compliance Runtime (CCR)** gate SHALL
  be executed as part of every syscall invocation, in-process, before the syscall body
  executes. No caller type, no principal level, and no deployment tier bypasses the gate.
  *Source:* Book 1 Part III §1; ADR-051.

- **EKOS-22-041 (Must, MVP)** — The CCR SHALL enforce Laws in three tiers:
  **Tier 1 — Structural** (architecture port boundaries; no runtime check needed);
  **Tier 2 — Behavioral** (CCR runtime check per qualifying syscall — Law 4, 6, 11, 16);
  **Tier 3 — Policy** (CCR delegates to the OPA/Rego policy engine — Law 5, 7, 8, 9, 10,
  12, 14, 15). Tier 2 checks SHALL complete in < 1ms P99 (in-process). Tier 3 policy
  evaluation SHALL be asynchronous where the calling syscall permits; synchronous where the
  result gates the commit (e.g., `Commit()` with a governance-required approval).
  *Source:* Book 1 Part III §2; ADR-051.

- **EKOS-22-042 (Must, P2)** — When the CCR detects a violation, it SHALL:
  (a) halt the syscall before any mutation or data access occurs,
  (b) return a typed `ConstitutionalViolation` subtype to the caller (matching the
  violated law — see Book 1 Part III §3 hierarchy),
  (c) emit a `ConstitutionalEvent` to the Constitutional Telemetry Stream before
  returning the error (see EKOS-22-045),
  (d) never reveal the existence of unauthorized knowledge objects in the violation
  message (consistent with EKOS-22-028 / Vol 04 §7).
  *Source:* Book 1 Part III §3; ADR-051.

- **EKOS-22-043 (Must, P2)** — The `ConstitutionalViolation` type hierarchy SHALL be
  **stable across ABI versions** — adding a new subtype is additive (not breaking);
  removing or renaming a subtype is breaking and follows the deprecation policy
  (EKOS-22-019). *Source:* ADR-051; Book 1 Law 14.

- **EKOS-22-044 (Must, P2)** — The CCR SHALL enforce the following behavioral-tier laws
  at the named syscalls:
  - `Knowledge.Commit()` → Law 11: reject if bitemporal fields (valid-time, transaction-
    time) are absent or if the write attempts a destructive overwrite.
  - `Knowledge.Project()` → Law 16: reject if the caller attempts to persist a
    projected representation to canonical knowledge without a declared evidence-artifact
    exception type.
  - `Knowledge.Query()` with an AI-bound context → Law 6: reject if the AI invocation
    lacks a `groundingSourceRef` pointing to a `Query()` result; reject if the AI
    operation executes outside the calling principal's PDP scope.
  - `Knowledge.Commit()` with a training flag → Law 7: reject if the commit would
    enqueue customer knowledge for model training without a verified opt-in record.
  - Any syscall using a cryptographic parameter → Law 4: reject null-key, test-bypass,
    or weakened-algorithm configurations in non-development deployment tiers.
  *Source:* Book 1 Part III §2; ADR-051.

---

## 13. Constitutional Telemetry Stream (CTS)

- **EKOS-22-045 (Must, P2)** — Every CCR evaluation — pass or violation — SHALL emit a
  **ConstitutionalEvent** to the **Constitutional Telemetry Stream (CTS)**. The event
  SHALL be emitted asynchronously (non-blocking to the caller) via an in-process buffer
  flushed to the Kafka journal (Vol 18 §8, topic `ekos.ccr.constitutional`). If the
  buffer is full, events are written to a local WAL before the Kafka flush completes.
  *Source:* Book 1 Part III §4; ADR-051.

- **EKOS-22-046 (Must, P2)** — The CTS SHALL be **immutable and append-only** — Law 11
  applies to the telemetry stream itself. No caller, tenant, or platform operator may
  delete or modify CTS events. *Source:* Book 1 Law 11; ADR-051.

- **EKOS-22-047 (Must, P2)** — The CTS SHALL be **caller-agnostic and
  deployment-tier-agnostic** — identical events are emitted whether the caller is a
  browser, SDK, EKR Viewer (local kernel instance), KWR Worker, third-party extension,
  or autonomous AI agent, and whether the deployment is SaaS, air-gap, or Edge.
  Air-gap deployments SHALL buffer CTS events locally and replicate to the central CTS
  upon connectivity. *Source:* Book 1 Part III §4; ADR-051; Book 1 Law 8.

- **EKOS-22-048 (Must, P2)** — The CTS SHALL emit the following mandatory fields per
  event (schema stable across ABI versions): `eventId`, `timestamp`, `lawsEvaluated[]`,
  `outcome` (PASS | VIOLATION | POLICY_DENY), `violatedLaw` (if outcome ≠ PASS),
  `violationType`, `syscall`, `callerType`, `callerId`, `tenantId`,
  `knowledgeObjectRef` (masked if PDP denies existence), `policyRef` (if Tier 3),
  `correlationId` (OpenTelemetry trace ID), `deploymentTier`.
  *Source:* Book 1 Part III §4; ADR-051.

- **EKOS-22-049 (Must, P2)** — The following **mandatory CTS metrics** SHALL be computed
  in real-time and exposed via `Observe()` within the calling tenant's PDP scope:
  `ekos.ccr.evaluations.total`, `ekos.ccr.evaluations.by_law`,
  `ekos.ccr.violations.total`, `ekos.ccr.violations.by_law`,
  `ekos.ccr.violations.by_caller_type`, `ekos.ccr.policy_denies.total`,
  `ekos.ccr.latency_p99_ms`.
  Platform operators (cross-tenant) MAY access aggregate metrics via their governance
  plane. *Source:* Book 1 Part III §4; ADR-051.

- **EKOS-22-050 (Should, P2)** — A **Constitutional Compliance Dashboard** SHALL be
  provided as a first-class platform governance view (a widget projection of the CTS —
  Law 3), showing per-tenant constitutional health, violation trends by law, caller-type
  breakdown, and SLO compliance for CCR latency. *Source:* Book 1 Part III §4; Vol 02
  (widget projection); ADR-051.

---

## 14. Extension Compliance Manifest (CCM)

- **EKOS-22-051 (Must, P2)** — Every third-party extension (widget, connector, app, SDK
  client, AI agent integration) MUST submit a signed **Constitutional Compliance Manifest
  (CCM)** at registration time. A CCM-less or CCM-invalid extension SHALL be rejected at
  registration; the syscall layer SHALL reject calls from an unregistered extension.
  *Source:* Book 1 Part III §5; ADR-051.

- **EKOS-22-052 (Must, P2)** — The CCM SHALL declare at minimum:
  (a) `extension_id` (globally unique),
  (b) `extension_type` (App | Widget | Connector | AI_Agent | SDK_Client),
  (c) `constitution_version` (pinned to the Book 1 version at registration),
  (d) `law_engagements[]` (which Laws the extension consciously interacts with and how),
  (e) `syscalls_used[]` (complete list; CCR checks against this list at every call),
  (f) `violation_surface` (how the extension surfaces ConstitutionalViolations to its
  own users — SDK-exception | UI-error-boundary | gRPC-status),
  (g) a cryptographic signature (Ed25519, signed with the author's tenant KMS key).
  *Source:* Book 1 Part III §5; ADR-051.

- **EKOS-22-053 (Must, P2)** — If a caller's `extension_id` invokes a syscall not
  declared in `syscalls_used`, the CCR SHALL reject the call with a
  `Law13Violation` (unexpected kernel access) and emit a CTS event.
  *Source:* Book 1 Part III §5; ADR-051.

- **EKOS-22-054 (Must, P2)** — When the Constitution is amended (new Law or Law update),
  all active CCMs are flagged as **pending re-compliance**. An extension that has not
  submitted an updated CCM against the new Constitution version within the declared
  re-compliance window SHALL have its syscall access suspended.
  *Source:* Book 1 Part III §5; ADR-051; Book 1 Law 14.

- **EKOS-22-055 (Should, P2)** — The **EKOS SDK** (Vol 23) SHALL include a CCM generator
  tool that scaffolds a valid CCM from the extension's declared syscall usage, reducing
  the compliance burden on extension authors to a review-and-sign step rather than manual
  authoring. *Source:* Vol 23; ADR-051.

- **EKOS-22-056 (Should, P3)** — CCMs SHALL be knowledge objects in the graph
  (introspectable via `Knowledge.Query()` — Law 15 self-hosting), allowing platform
  operators and compliance officers to audit the full extension landscape, search for
  extensions engaging with a specific law, or query the set of extensions with pending
  re-compliance obligations. *Source:* Book 1 Law 15; Vol 25; ADR-051.

---

## 16. Open Questions (parked)

- **OQ-22-A** — Concrete IDL & binding-generation toolchain (illustrative candidates;
  decide in Vol 15/A03) — must satisfy language-agnostic + transport-pluggable.
- **OQ-22-B** — ABI deprecation window length (how many years/versions old ABIs are
  supported) — define in 15-year sustainability arch (A09); carries Vol 18 OQ-18-D.
- **OQ-22-C** — In-process vs IPC/network call performance at scale (when each transport)
  — benchmark with Vol 15.
- **OQ-22-D** — Exact split: which capabilities are syscalls vs SDK conveniences vs
  public-API only — finalize with Vol 23 / Vol 09.
- **OQ-22-E** — Streaming back-pressure & cancellation semantics for high-volume
  `Subscribe`/`Acquire` — with Vol 21 / Vol 20.

---

## 13. Requirement Index (this volume)

EKOS-22-001 … EKOS-22-039 — original 58 requirements (OS-contract principle, full syscall
catalog, stable ABI/versioning, language-agnostic binding, sync/async/streaming, error
model, per-call enforcement, SDK/public-API relationship, observability, self-hosting).

EKOS-22-040 … EKOS-22-056 — **CCR amendment (2026-07-03, ADR-051)**: Constitutional
Compliance Runtime gate (040–044), Constitutional Telemetry Stream (045–050), Extension
Compliance Manifest (051–056). **Total: 74 requirements.**

> **Next gap:** `23 — EKOS Application Runtime & SDK`.
> Per protocol: STOP here. Await user "proceed".
