# EKOS Volume 07 — EKR Viewer Family + Cryptographic-Expiration Enforcement + Viewer Monetization

> **Status:** ✅ Done · **Requirement count:** 99 · **Generated:** 2026-06-30 (Opus)
> **Sources mined:** `EKR_Research.txt` (cross-platform free viewer, capabilities/
> limitations, authentication, offline ABAC engine, self-destruct viewer execution,
> in-app monetization, store integration, on-device inference, govt+monetization),
> `blueprint §8` (Viewer/Studio arch, Capacitor+Electron, offline-first, SQLCipher,
> PWA/WASM) + §11 (signature verification) + §12 (EKR Everywhere, Gateway Module) +
> §16 (Viewer Pro/tiers), `ChatGPT` (Viewer FAMILY + Viewer/Viewer+/Viewer AI/Viewer
> Enterprise tiers; runtime = Rust/C++/WebGPU/WASM, host only hosts), `Gemini critique`
> (native rendering bypass; per-session forensic steganographic watermark),
> `Patents.txt` §1 (self-destruct viewer execution), §8 (Presentation Guardian), §10
> (Living Canvas dual-mode), §6 (seed assembly), §2 (offline masked projection),
> Sandeep Genie (Vol 03 §18). Honors [00-MASTER-INDEX](00-MASTER-INDEX.md) §2.

---

## 1. Scope & Boundary

**In scope:** the **EKR Viewer family** (the apps that open `.ekr`); cross-platform
architecture & rendering reuse; viewer capabilities/limitations; **authentication &
identity**; the **embedded offline ABAC evaluator** (runtime behavior); **signature
verification & anti-tamper**; **Cryptographic-Expiration enforcement** (key-wipe,
secure-erase, screen-capture prevention); **copy-proof activation flow** (viewer
side); **Presentation Guardian Mode** runtime; **forensic watermark rendering**;
**Living Canvas / Seed-mode** rendering; **in-app monetization** (free baseline +
Pro tiers + store integration + on-device model unlock); **Sandeep Genie** in Pro;
**government+monetization** offline licensing; **EKR Everywhere** (browser-native,
Web Viewer SDK/WASM, extensions, PWA file handlers, MIME); the **EKR Gateway Module**;
**distribution**; **offline-first & sync**.

**Explicitly deferred (cross-referenced, not duplicated):**
- The `.ekr`/`.ekrd` **format, container, manifest, crypto packaging, layered
  encryption, expiration controls in the bundle, activation nonce, registry,
  composition, signing** → **Vol 06** (this volume *consumes/enforces* the format)
- The **ABAC model & masking policy**, Presentation Guardian *policy*, e-signatures
  *semantics*, forensic-watermark *policy* → **Vol 04** (this volume is where they
  *run* offline)
- The **rendering engine core** (native GPU/WASM), Living Canvas *definition*, Word/
  PDF/HTML export → **Vol 02** (the viewer *reuses* the engine)
- **AI analyst behavior, on-device small/LLM models, Sandeep Genie engine** → **Vol 03**
  (this volume defines viewer packaging/monetization of them)
- **Marketplace economics, Stripe payouts, revenue share, platform subscription
  tiers** → **Vol 13** (this volume covers only *viewer in-app purchase* mechanics);
  ecosystem governance/vetting → **Vol 05**
- **P2P live sessions, EKR SSO / Identity Bridge token sourcing** → **Vol 12** (the
  viewer *uses* stored credentials; their lifecycle is Vol 12)

---

## 2. Core Principle — The Free, Cross-Platform, Secure Viewer

- **EKOS-07-001 (Must, MVP)** — EKOS SHALL ship a **free, cross-platform Viewer** that
  opens `.ekr` bundles and reproduces the interactive visual experience (read-only) —
  the **"Adobe Reader of enterprise knowledge"** — so any stakeholder, client,
  auditor, or vendor can consume bundles **without a platform license**. *Source:*
  EKR_Research(viewer §2; impact §); blueprint §8.

- **EKOS-07-002 (Must, MVP)** — The viewer SHALL run on **Windows, macOS, Linux, iOS,
  Android** (phones & tablets) and the **web**, from a single codebase where practical.
  *Source:* EKR_Research(viewer §2 — cross-platform); blueprint §8 (Capacitor+Electron,
  100% widget reuse).

- **EKOS-07-003 (Must, MVP)** — Viewer security SHALL be **cryptographic, not trust-
  based**: the viewer is NOT trusted; if it lacks the key, it cannot read the data
  (Constitution §2). *Source:* EKR_Research(PII §"Is Viewer Valid" — "security boundary
  is cryptography, not trust boundary").

---

## 3. The Viewer Family

> ChatGPT: don't stop at one viewer — build a family; this becomes a knowledge
> distribution ecosystem.

- **EKOS-07-004 (Must, MVP)** — **EKR Reader** — the baseline free read/consume viewer.
  *Source:* ChatGPT(viewer family — Read).

- **EKOS-07-005 (Should, P2)** — **EKR Presenter** — presentation mode (full-screen,
  guided navigation, Guardian-Mode-aware). *Source:* ChatGPT(EKR Presenter).

- **EKOS-07-006 (Should, P2)** — **EKR Review** — comment + approvals mode (clarification
  workflow, sign-off). *Source:* ChatGPT(EKR Review); relates [[project-esip]] Vol 15
  review designer.

- **EKOS-07-007 (Should, P2)** — **EKR Inspector** — audit mode (trust chain, lineage,
  provenance, evidence, access log inspection). *Source:* ChatGPT(EKR Inspector);
  Vol 06 §14–16.

- **EKOS-07-008 (Should, P2)** — **EKR Diff** — compare bundles side-by-side with
  highlighted differences and time-travel view (Pro feature, §16). *Source:* ChatGPT
  (EKR Diff); EKR_Research(monetization — Bundle Compare & Merge).

- **EKOS-07-009 (Should, P2)** — **EKR Mobile** — field-engineer mobile viewer (offline,
  touch-first). *Source:* ChatGPT(EKR Mobile).

- **EKOS-07-010 (Should, P2)** — **EKR Embedded** — an embeddable viewer for Jira,
  Teams, Confluence, etc. *Source:* ChatGPT(EKR Embedded); blueprint §12.

- **EKOS-07-011 (Should, P2)** — **EKR SDK** — to build custom viewers / integrate `.ekr`
  rendering into third-party apps. *Source:* ChatGPT(EKR SDK); blueprint §8 (Web Viewer
  SDK).

- **EKOS-07-012 (Should, P2)** — **EKR CLI** — inspect, verify, diff, sign bundles from
  the terminal (CI/automation). *Source:* ChatGPT(EKR CLI).

- **EKOS-07-013 (Should, P2)** — **EKR Browser Extension** — open `.ekr` directly in the
  browser. *Source:* ChatGPT(EKR Browser Extension); blueprint §12.

- **EKOS-07-014 (Must, MVP)** — All family members SHALL share the **same rendering
  engine core and security model** so a bundle looks and behaves identically across
  them. *Source:* EKR_Research(viewer §2 — reuse engine); Vol 02 EKOS-02-012.

---

## 4. Cross-Platform & Rendering Architecture

- **EKOS-07-015 (Must, MVP)** — The viewer SHALL reuse the **native, GPU-accelerated
  rendering engine** of Vol 02 (Rust/C++/WebGPU/WASM core), with the host UI framework
  (Flutter/Capacitor+Electron/web) **only hosting** the engine — never laying out large
  graphs node-by-node. *Source:* Gemini critique; ChatGPT(rendering = Rust/C++/WebGPU/
  WASM, host only hosts); Vol 02 EKOS-02-009.

- **EKOS-07-016 (Must, P2)** — A **Progressive Web Viewer (WebAssembly-based EKR Web
  Viewer SDK)** SHALL allow browser-native opening of `.ekr` without install. *Source:*
  blueprint §8 (PWA WASM); §12.

- **EKOS-07-017 (Must, MVP)** — The viewer SHALL be **offline-first**: a local
  **encrypted vault (SQLCipher-class)** stores opened bundles; full feature parity
  offline; background sync when online. *Source:* blueprint §8 (offline-first, SQLCipher,
  background sync, full parity).

- **EKOS-07-018 (Should, P2)** — Heavy core processing (encryption, graph traversal) MAY
  be migrated to a Rust core exposed as native plugins for performance. *Source:*
  blueprint §8 ("core heavy processing ... migrated to Rust ... Capacitor plugins").

---

## 5. Viewer Capabilities & Limitations

- **EKOS-07-019 (Must, MVP)** — The free viewer SHALL: open/decrypt bundles
  (password/PIN/biometric), **render the graph interactively** (zoom, pan, drill-down
  C4 levels, click nodes, play packet animations, scrub timelines), show the
  **precomputed AI Insights panel** offline, and provide **basic search & filter**.
  *Source:* EKR_Research(viewer §2 capabilities; monetization free baseline).

- **EKOS-07-020 (Must, MVP)** — The free viewer SHALL apply **PII visual masking** with
  placeholders (the offline ABAC evaluator, §6). *Source:* EKR_Research(monetization —
  free baseline incl. PII masking).

- **EKOS-07-021 (Must, MVP)** — Free-viewer **limitations** SHALL be: no edit, no graph
  modification, no live connectors (Git/Jira/cloud), and no *new* AI questions (only
  prepackaged insights) — editing/live-AI are platform/Pro features. *Source:*
  EKR_Research(viewer §2 limitations).

- **EKOS-07-022 (Must, MVP)** — A fully self-contained bundle SHALL be viewable with
  **zero network calls**. *Source:* EKR_Research(viewer §2); Vol 06 EKOS-06-005.

---

## 6. Embedded Offline ABAC Evaluator (Runtime Behavior)

- **EKOS-07-023 (Must, P2)** — The viewer SHALL embed a **stripped-down ABAC policy
  evaluator** (e.g., an OPA-Wasm module or compiled rule engine) replicating the
  platform's ABAC rules from the bundle's packaged policy (Vol 06 §7). *Source:*
  EKR_Research(PII §5 — local ABAC engine).

- **EKOS-07-024 (Must, P2)** — On successful authentication, the viewer SHALL derive the
  decryption keys for the layers the identity is authorized for, then: render
  `public_layer` nodes fully, render authorized encrypted layers fully, and render
  unauthorized layers as **placeholder masks (grey box / lock icon)** — exactly as the
  platform does. *Source:* EKR_Research(PII §5); Patent §2; Vol 04 EKOS-04-090.

- **EKOS-07-025 (Must, P2)** — Property-level restriction SHALL be honored offline
  (e.g., blur only the `email` field). *Source:* EKR_Research(PII §5); Vol 04
  EKOS-04-014.

- **EKOS-07-026 (Must, P2)** — All decryption SHALL occur in the **viewer's sandboxed
  environment**; keys SHALL be **flushed from memory immediately** after use. *Source:*
  EKR_Research(PII §6 — zero-trust & anti-tamper).

---

## 7. Authentication & Identity

- **EKOS-07-027 (Must, MVP)** — The viewer SHALL authenticate the opener via **password,
  PIN, or biometric**, and SHALL use the device's **hardware-backed keystore** (Secure
  Enclave / StrongBox / TPM) for key operations. *Source:* EKR_Research(PII §4; format
  §1 — device keystore).

- **EKOS-07-028 (Should, P2)** — For recipient-specific bundles, the viewer SHALL use a
  **device-local private key/certificate** provisioned via a secure enrolment process.
  *Source:* EKR_Research(PII §4 — recipient-specific).

- **EKOS-07-029 (Should, P2)** — A user MAY **import an identity token (JWT/SAML
  assertion)** verified once online, after which it works **offline** within its
  validity period (X.509-style expiry). *Source:* EKR_Research(PII §4; "offline
  authorisation"). *Depends:* Vol 12 (Identity Bridge).

---

## 8. Signature Verification & Anti-Tamper

- **EKOS-07-030 (Must, MVP)** — The viewer SHALL **verify the bundle's digital signature
  (Ed25519/X.509)** before opening; a tampered bundle SHALL NOT open. *Source:*
  EKR_Research(format §1; PII §6); blueprint §11; Vol 06 EKOS-06-018.

- **EKOS-07-031 (Should, P2)** — The viewer binary SHALL be **code-signed** and perform
  **runtime integrity checks** (checksums, anti-debugger); if patched/tampered (to
  steal keys), it SHALL **refuse to decrypt**. *Source:* EKR_Research(format-security §4
  — anti-RE layer); Patent §1 (anti-tamper watchdog).

- **EKOS-07-032 (Should, P2)** — On mobile, the viewer SHALL use **app attestation**
  (Play Integrity / DeviceCheck) to confirm it is the genuine, unmodified app before
  unlocking keys. *Source:* EKR_Research(format-security §2/§4); Vol 06 EKOS-06-036.

- **EKOS-07-033 (Should, P2)** — Anti-tamper SHALL be an **extra layer**, never the
  primary defense — core security is the key (Constitution §2). *Source:* EKR_Research
  (format-security §4); Constitution §2.

---

## 9. Cryptographic-Expiration Enforcement (Viewer Side)

> The runtime side of Vol 06 §9. Constitution §3 naming.

- **EKOS-07-034 (Should, P2)** — On opening a Cryptographic-Expiration bundle, the viewer
  SHALL show a **clear warning** ("Content will be permanently inaccessible after
  viewing") and decrypt content **into RAM only — never writing plaintext to disk**.
  *Source:* EKR_Research(self-destruct §2); Patent §1 (claim 1, detailed).

- **EKOS-07-035 (Should, P2)** — The viewer SHALL enable **screen-capture prevention**
  (Android `FLAG_SECURE`, macOS window-level, equivalents) for such content. *Source:*
  EKR_Research(self-destruct §2); Patent §1; Patent §8.

- **EKOS-07-036 (Should, P2)** — On TTL expiry or view-count limit, the viewer SHALL:
  **zero-out decryption keys**, **flush non-swappable memory pages**, **secure-erase**
  the bundle file (overwrite with random data, then delete; TRIM on SSD), and **write an
  encrypted local audit-log entry** ("Bundle XYZ destroyed at [timestamp]"). *Source:*
  EKR_Research(self-destruct §2); Patent §1 (claim 1).

- **EKOS-07-037 (Should, P2)** — If the user force-quits mid-view, the viewer SHALL
  **complete pending destruction on next launch**, using a **secure monotonic clock /
  watchdog** for offline TTL enforcement. *Source:* EKR_Research(self-destruct §2 —
  force-quit handling; offline monotonic clock).

- **EKOS-07-038 (Should, P2)** — An **anti-tamper watchdog** SHALL **instantly wipe keys**
  if the viewer is forcibly debugged or its memory is dumped. *Source:* EKR_Research
  (self-destruct §1 — watchdog); Patent §1.

- **EKOS-07-039 (Could, P3)** — The viewer SHALL optionally return a **destruction
  attestation** (signed proof) to the platform when next online. *Source:* EKR_Research
  (self-destruct §1); Vol 06 EKOS-06-032.

---

## 10. Copy-Proof Activation Flow (Viewer Side)

- **EKOS-07-040 (Should, P2)** — On **first open** of an activation-bound bundle, the
  viewer SHALL warn "one-time activation required; internet required", send its
  **hardware fingerprint/attestation + nonce** to the activation server (public or
  on-prem), receive a **device-bound, time-limited token**, and store it in the
  **Secure Enclave**. *Source:* EKR_Research(copy-proof §2); Vol 06 EKOS-06-036.

- **EKOS-07-041 (Should, P2)** — Thereafter the viewer SHALL operate **fully offline**,
  deriving the decryption key from the **server-signed token + device-specific key**
  (DH/key-derivation); the key never persists in plaintext. *Source:* EKR_Research
  (copy-proof §3).

- **EKOS-07-042 (Should, P2)** — A copy opened on a **different device** SHALL fail (no
  token; key cannot be derived) and a second activation request for the same Bundle ID
  SHALL be rejected by the server ("already consumed"). *Source:* EKR_Research(copy-proof
  §4); Vol 06 EKOS-06-036.

- **EKOS-07-043 (Should, P2)** — For air-gapped/pre-provisioned bundles, the viewer SHALL
  open only on the **pre-registered device** with no runtime activation. *Source:*
  EKR_Research(copy-proof §5); Vol 06 EKOS-06-038.

---

## 11. Presentation Guardian Mode (Viewer Runtime)

> Policy defined in Vol 04 §13; this is the viewer-side execution.

- **EKOS-07-044 (Should, P2)** — The viewer SHALL **detect screen sharing/capture** (OS
  signals), activate **Guardian Mode** (stricter masking on all open widgets regardless
  of personal rights), and support a **time-limited, audited Reveal Pass** with visible
  countdown + identity watermark + optional second-user approval. *Source:* Patent §8;
  blueprint §6; Vol 04 EKOS-04-039…044.

---

## 12. Forensic Watermark Rendering

> Gemini's analog-hole fix (photograph of screen).

- **EKOS-07-045 (Should, P2)** — When content is decrypted for display, the viewer's
  layout engine SHALL embed a **microscopic, per-session steganographic pattern** across
  the canvas background encoding the **viewer's public-key hash, session ID, and device
  ID** (frequency-shift steganography). *Source:* Gemini critique (Dynamic Per-Session
  Visual Forensic Watermarking).

- **EKOS-07-046 (Should, P2)** — A leaked photograph (even angled/cropped) SHALL be
  decodable by a security analysis script to **pinpoint the leaking device/identity**.
  *Source:* Gemini critique.

- **EKOS-07-047 (Should, P2)** — Watermarking SHALL be governed by the bundle's
  **Watermark Policy** (Vol 06 §20) per classification, and SHALL NOT degrade
  accessibility/searchability (policy-driven, not vector-path text). *Source:* Vol 06
  EKOS-06-064; Vol 04 EKOS-04-060.

---

## 13. Living Canvas & Seed-Mode Rendering

- **EKOS-07-048 (Should, P2)** — The viewer SHALL render **Living Canvas** (fixed-layout,
  PDF-grade fidelity) and **responsive** modes, switchable without altering the bundle
  (Patent §10; Vol 02 §4). *Source:* Patent §10; Vol 02 EKOS-02-013…017.

- **EKOS-07-049 (Should, P2)** — For **Seed Bundles**, the viewer SHALL render the
  skeleton + AI summary immediately and **progressively fetch & splice** referenced
  content (Patent §6; Vol 06 §13), animating updates as they load. *Source:* Patent §6;
  Vol 06 EKOS-06-047/048.

- **EKOS-07-050 (Should, P2)** — For **Delta (`.ekrd`)** drops, the viewer SHALL splice
  changes into the local base graph without a full rebuild (Vol 06 §11). *Source:*
  Gemini; Vol 06 EKOS-06-041.

---

## 14. In-App Monetization — Free Baseline

- **EKOS-07-051 (Must, MVP)** — The following SHALL always be **free** (drives adoption):
  open bundles, interactive graph viewing (zoom/drill/animations), **prepackaged AI
  insights**, basic search, PII masking, and **Cryptographic-Expiration** (a security
  feature — never monetized). *Source:* EKR_Research(monetization — free baseline;
  "self-destruct ... monetize nahi karna chahiye").

- **EKOS-07-052 (Must, MVP)** — The free viewer SHALL carry **no ads** and be
  **signature-verified**. *Source:* EKR_Research(viewer §2 — distribution, free no ads).

---

## 15. EKR Reader Pro & Viewer Tiers

> ChatGPT: expand Free/Pro into a tier ladder. EKR_Research: per-feature monetization.

- **EKOS-07-053 (Should, P2)** — Viewer monetization SHALL use a **tier ladder**:
  **Viewer** (free), **Viewer+**, **Viewer AI**, **Viewer Enterprise** — more scalable
  than a flat Free/Pro split. *Source:* ChatGPT(tiers — "Viewer / Viewer+ / Viewer AI /
  Viewer Enterprise"); EKR_Research(monetization).

- **EKOS-07-054 (Should, P2)** — **AI Ask & Chat** (Pro): the user can ask live questions
  of the bundle; an **on-device small model** (quantized Llama/Mistral-class) answers
  in the bundle's context with **no cloud call**; sold as subscription / Pro unlock
  (not per-token). *Source:* EKR_Research(monetization — AI Ask & Chat). *Depends:*
  Vol 03 (on-device model).

- **EKOS-07-055 (Should, P2)** — **Advanced AI Analysts** (in-app purchase, per pack):
  e.g., Cloud Cost Optimizer, Security Auditor, Architecture Smell Detector run on the
  bundle graph and produce reports. *Source:* EKR_Research(monetization — Advanced AI
  Analysts); Vol 03 §19.

- **EKOS-07-056 (Should, P2)** — **Simulations & What-If Engine** (one-time per module):
  sandboxed local simulations ("if users 10×..."). *Source:* EKR_Research(monetization —
  Simulations). *Depends:* Vol 03 (What-If Simulator).

- **EKOS-07-057 (Should, P2)** — **Bundle Compare & Merge** (Pro subscription) — the EKR
  Diff family member (§8). *Source:* EKR_Research(monetization — Bundle Compare & Merge).

- **EKOS-07-058 (Should, P2)** — **Watermark & Redaction tools** (Pro/Enterprise) — apply
  custom watermark, or redact fields to generate a new read-only bundle. *Source:*
  EKR_Research(monetization — Watermark & Redaction).

- **EKOS-07-059 (Should, P2)** — **Multi-Bundle Workspace** (Pro subscription) — open
  many bundles and cross-reference. *Source:* EKR_Research(monetization — Multi-Bundle
  Workspace).

---

## 16. Monetization — Technical Implementation

- **EKOS-07-060 (Should, P2)** — An **In-App Purchase Manager** SHALL integrate the
  platform-appropriate store: **Apple StoreKit, Google Play Billing, Microsoft Store,
  Mac App Store**, and **Stripe** (Linux/direct) for license-key activation. *Source:*
  EKR_Research(monetization — technical implementation).

- **EKOS-07-061 (Should, P2)** — Pro AI capabilities SHALL run via **on-device inference**
  (offline, privacy-preserving); model weights SHALL be **encrypted and decrypted only
  for a paid user via a license-derived key** (so the viewer ships larger but premium
  features unlock only after purchase). *Source:* EKR_Research(monetization — technical;
  on-device, encrypted weights).

- **EKOS-07-062 (Should, P2)** — A **"Try Pro" trial** (e.g., 7 days) SHALL be offered,
  then a paywall. *Source:* EKR_Research(monetization — "Try Pro" trial).

- **EKOS-07-063 (Should, P2)** — Viewer in-app revenue is **distinct** from platform
  subscriptions and marketplace revenue (the full economy is Vol 13). *Source:*
  EKR_Research(monetization — "platform license + marketplace cut + Viewer Pro
  subscriptions"); Vol 13.

---

## 17. Sandeep Genie in Pro Viewer

- **EKOS-07-064 (Could, P3)** — **Sandeep Genie** (Vol 03 §18) SHALL be available in the
  paid Pro Viewer as a photorealistic 3D avatar with voice/chat that controls viewer
  features — an interface over the same governed, security-scoped engine (no extra
  access). *Source:* blueprint §4; Vol 03 EKOS-03-080/081.

---

## 18. Government + Monetization Combined

- **EKOS-07-065 (Should, P2)** — Government/air-gapped deployments SHALL get **offline
  license activation** for Pro features (volume licensing, pre-installed license key,
  **no cloud check**), so air-gapped users can use paid on-device AI. *Source:*
  EKR_Research(combined edge — govt + monetization).

- **EKOS-07-066 (Must, P2)** — Cryptographic-Expiration + AI Ask MAY coexist in a govt
  bundle, with all activation/licensing happening via the internal server. *Source:*
  EKR_Research(combined edge); Vol 06 EKOS-06-037.

---

## 19. EKR Everywhere (Browser-Native & Embedding)

- **EKOS-07-067 (Should, P2)** — A custom MIME type (`application/ekr`), the **EKR Web
  Viewer SDK**, **browser extensions**, and **PWA File Handlers** SHALL make `.ekr`
  open directly in the browser **like a PDF**. *Source:* blueprint §12 (EKR Everywhere);
  Vol 06 EKOS-06-003.

- **EKOS-07-068 (Should, P2)** — **EKR Embedded** SHALL embed the viewer inside Jira,
  Teams, Confluence, etc. (§3). *Source:* blueprint §12; ChatGPT(EKR Embedded).

---

## 20. EKR Gateway Module

> blueprint §12 — server-side interception & policy actions.

- **EKOS-07-069 (Could, P3)** — A lightweight native **EKR Gateway Module** (C/Rust
  plugin) SHALL be available for **Nginx, Apache, IIS, and Node.js** that **intercepts
  `.ekr` files** and executes configurable actions: **validate, store to cloud, notify,
  webhook, move, scan**. *Source:* blueprint §12 (EKR Gateway Module).

- **EKOS-07-070 (Could, P3)** — The Gateway SHALL support **admin-set global default
  policies that developers can override per location**. *Source:* blueprint §12 (default
  & overridable policies).

---

## 21. Distribution

- **EKOS-07-071 (Must, MVP)** — The viewer SHALL be distributed via **Apple App Store,
  Google Play, Microsoft Store, Flathub, and direct download**, **signature-verified**,
  free, no ads. *Source:* EKR_Research(viewer §2 — distribution).

---

## 22. Offline-First & Sync

- **EKOS-07-072 (Must, P2)** — The viewer SHALL maintain a **local event queue** (opens,
  comments, signatures, access logs, destruction attestations) and **sync it to the
  platform when online** (Vol 06 §24). *Source:* blueprint §13; EKR_Research(viewer
  audit logs sync); Vol 06 EKOS-06-076.

- **EKOS-07-073 (Must, P2)** — Offline access SHALL maintain **full feature parity** for
  free capabilities; Pro features that need on-device models work offline once unlocked.
  *Source:* blueprint §8 (full parity offline); EKR_Research(on-device AI offline).

---

## 23. Constitution Conformance (viewer-specific)

- **EKOS-07-074 (Must, MVP)** — Viewer security is cryptographic-hardness, not obscurity;
  accurate claims wording (Constitution §2). *Source:* Constitution §2; §8.

- **EKOS-07-075 (Must, MVP)** — "Cryptographic Expiration" terminology (Constitution §3);
  the security feature is never monetized. *Source:* Constitution §3; EKOS-07-051.

- **EKOS-07-076 (Must, MVP)** — Host frameworks, stores, on-device model families, and
  the activation server are **illustrative & pluggable** (Constitution §6); air-gap
  excludes only internet-only paths, each with an on-prem equivalent. *Source:*
  Constitution §6.

- **EKOS-07-077 (Must, MVP)** — The viewer SHALL choose the **most privacy-preserving
  defaults** and make **zero network calls** for self-contained bundles. *Source:*
  Constitution (privacy); EKR_Research(viewer offline).

---

## 24. Open Questions (parked)

- **OQ-07-A** — Host-shell strategy per platform (Flutter+plugin vs Capacitor+Electron
  vs pure-web WASM) reconciling EKOS-07-015/016 with Vol 02 OQ-02-A — Vol 15 decision.
- **OQ-07-B** — App-store policy compliance for crypto/DRM + on-device AI model
  shipping (size, store rules) — legal/store review.
- **OQ-07-C** — Steganographic watermark robustness vs. compression/cropping/re-photo
  — needs a detection-accuracy study.
- **OQ-07-D** — On-device model footprint (quantized LLM) on low-end phones for AI Ask —
  feasibility pilot (shared with Vol 03 OQ-03-D).
- **OQ-07-E** — Secure-erase guarantees on modern SSDs/flash (wear-leveling defeats
  overwrite) — document residual; key-destruction is the real guarantee (Constitution
  §3).

---

## 25. Requirement Index (this volume)

EKOS-07-001 … EKOS-07-077 (stable numbering; viewer family, cross-platform, offline
ABAC, signature/anti-tamper, crypto-expiration enforcement, copy-proof flow, Guardian
Mode, watermark, monetization tiers, Genie, govt licensing, EKR Everywhere, Gateway,
distribution, sync expand to 99 distinct stated rules).

> **Next volume:** `08 — Enterprise Customization & Workflow Orchestration`.
> Per protocol: STOP here. Await user "proceed".
