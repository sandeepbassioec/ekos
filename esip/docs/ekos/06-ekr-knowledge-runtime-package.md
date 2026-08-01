# EKOS Volume 06 — `.ekr` Knowledge Runtime Package & Format

> **⚠️ ESCS ADDENDUM (2026-08-01):** The `.ekr` bundle format defined in this volume is
> the **underlying packaging machinery** for the **Knowledge Package** concept formalized
> by ESCS-034. The "Bundle Store / Registry" (§4.11) and "Publishing Pipeline" (§3) map
> directly to the **Knowledge Package Registry** (ESCS-038) and **Application Manifest →
> Knowledge Package compilation** (ESCS-037). The no-source-code-ships constraint
> (ESCS-042) is enforced by the format-secrecy / anti-RE requirements in §25. See
> [ESCS-SUPPLEMENT.md §10](ESCS-SUPPLEMENT.md) for the full Application Deployment &
> Subscription Platform spec; see [Traceability Matrix](ESCS-SUPPLEMENT.md#11-traceability-matrix)
> rows ESCS-034, ESCS-037, ESCS-038, ESCS-041, ESCS-042 for the precise mapping.

> **Status:** ✅ Done · **Requirement count:** 104 · **Generated:** 2026-06-30 (Opus)
> **Sources mined:** `EKR_Research.txt` (format, container, AES-256-GCM/DRM, layered
> multi-recipient PII encryption, self-destruct, copy-proofing, format secrecy),
> `blueprint §8` (EKR Studio/Viewer arch) + §9 (media types) + §10 (Seed Mode/Nano
> Bot) + §11 (Digital Signatures) + §13 (Event-Driven) + §16 (Monetization), `ChatGPT`
> reframing (**Knowledge Runtime Package not "encrypted file"**; bundle runtime,
> OCI-style manifest, package manager, registry, composition, live-vs-snapshot, trust
> chain, lineage, provenance, bundle policy engine, VM-image-for-knowledge),
> `Gemini critique` (`.ekrd` delta bundles), `Patents.txt` §1 (Self-Destructing
> Hardware-Bound Bundles), §6 (Seed Mode / Progressive Self-Assembly), §7 (Cross-
> Document Living References). Honors [00-MASTER-INDEX](00-MASTER-INDEX.md) §2.

---

## 1. Scope & Boundary

**In scope:** the **`.ekr` package format & runtime contract** — the reframing from
"encrypted file" to **Knowledge Runtime Package**; the bundle-as-executable-runtime
model; container structure & manifest; encryption/DRM; **layered multi-recipient PII
encryption**; the publishing/generation pipeline; **cryptographic expiration**
(format side) & **copy-proofing**; **`.ekrd` delta** bundles; lifecycle modes
(Snapshot / Live / Seed); **Seed Mode / progressive self-assembly**; **trust chain**,
**lineage**, **provenance**; **composition**; **package manager** (dependencies);
**Bundle Store / Registry**; **Bundle Policy Engine**; **digital signatures / legal
sealing**; **cross-document living references**; **media types** in bundles;
**event-driven** within a bundle; **format secrecy / anti-RE**; and the no-lock-in
relationship.

**Explicitly deferred (cross-referenced, not duplicated):**
- The **Viewer apps** that open `.ekr` (Reader/Presenter/Review/Inspector/Diff/Mobile/
  Embedded/SDK/CLI/Browser), in-app AI monetization, Sandeep Genie in viewer, the
  offline ABAC evaluator's *runtime behavior*, **cryptographic-expiration enforcement
  & key-wipe**, forensic-watermark *rendering*, EKR Everywhere/Gateway → **Vol 07**
- The graph, ontology, bitemporal substrate, **semantic JSON-LD/RDF export & offline
  player** (the open no-lock-in escrow) → **Vol 01** (distinct from `.ekr`; see §27)
- Widget rendering engine, Living Canvas, Word/PDF/HTML export → **Vol 02**
- Pre-computed AI insights packaged in the bundle are produced by → **Vol 03**
- Live-platform ABAC/masking & e-signature *of graph nodes* → **Vol 04** (this volume
  is the *bundle-side* packaging of the same model)
- Marketplace **economics** (pricing, revenue share, payouts) → **Vol 13** (this
  volume = the registry/format/governance side); ecosystem **governance** (vetting,
  licensing enforcement, monetization audit) → **Vol 05**
- P2P live-session ephemeral handshake / transport → **Vol 12**

---

## 2. Core Reframing — Knowledge Runtime Package (NOT "Encrypted File")

> ChatGPT's ruling: stop thinking of `.ekr` as a secure file format; define it as a
> Knowledge Runtime Package. This is a category-defining decision.

- **EKOS-06-001 (Must, MVP)** — `.ekr` SHALL be defined and described as a **Knowledge
  Runtime Package** (a portable, executable enterprise-knowledge runtime), **NOT** as
  "Encrypted Knowledge Representation" or a mere encrypted document. The name reserves
  flexibility for streaming/incremental/partial/embedded/AI/offline variants. *Source:*
  ChatGPT(reframing — "EKR becomes runtime package instead of merely encrypted file").

- **EKOS-06-002 (Must, MVP)** — The package SHALL be positioned as a **distribution
  standard** — the goal is that people say "send me the EKR" the way they say "send me
  the PDF". *Source:* ChatGPT("if this succeeds, people won't say 'can you send me the
  PDF', they'll say 'can you send me the EKR'"); EKR_Research(de-facto standard).

- **EKOS-06-003 (Must, MVP)** — `.ekr` SHALL register a custom MIME type
  (`application/ekr`) and file association so it behaves like a first-class document
  type (full handler behavior in Vol 07). *Source:* blueprint §12 (EKR Everywhere —
  custom MIME). *Depends:* Vol 07.

---

## 3. The Bundle as an Executable Runtime (Mini-EKOS / VM-for-Knowledge)

- **EKOS-06-004 (Must, P2)** — An `.ekr` SHALL be an **executable knowledge runtime**,
  not a static payload: opening it gives the recipient a **mini-EKOS** offering
  widgets, **local runtime, AI, simulation, search, navigation, timeline, playback,
  diff, annotations, comments, and offline analytics** — without a server, platform,
  license, or internet. *Source:* ChatGPT(bundle runtime — "the bundle itself becomes
  mini EKOS"; "equivalent of a VM image for enterprise knowledge"). *Depends:* Vol 07
  (the runtime host).

- **EKOS-06-005 (Must, MVP)** — A bundle SHALL be **self-contained**: when fully packed,
  the viewer renders the interactive experience with **zero network calls**. *Source:*
  EKR_Research(viewer — "fully offline if bundle self-contained").

- **EKOS-06-006 (Should, P2)** — The runtime semantics inside a bundle SHALL reuse the
  same widget definitions, graph subset, and (pre-computed) AI insights as the live
  platform — authored once, consumed identically. *Source:* EKR_Research(bundle
  contents); blueprint §8 (100% widget reuse). *Depends:* Vol 02, Vol 03.

---

## 4. Container Format & Structure

- **EKOS-06-007 (Must, MVP)** — The `.ekr` container SHALL be a **ZIP-like structure
  with an unencrypted file table but AES-256-GCM-encrypted data blobs**. *Source:*
  EKR_Research(format §1 — container).

- **EKOS-06-008 (Must, MVP)** — A bundle SHALL contain: `graph.json.enc` (the subgraph —
  nodes, edges, properties, visual-layout metadata), `widgets/` (widget definitions +
  visual recipes), `assets/` (images, fonts, small shader programs for animations),
  `ai_insights/` (pre-generated crux summaries, trade-off analyses, example stories so
  the viewer can show them offline), and `manifest.json.enc` (bundle descriptor).
  *Source:* EKR_Research(format §1 — contents).

- **EKOS-06-009 (Must, P2)** — For multi-clearance bundles, the structure SHALL add:
  `public_layer` (no-key-required readable metadata — manifest, widget defs,
  non-sensitive graph structure with masked nodes as placeholders),
  `encrypted_data_layers` (per-recipient encrypted blobs of sensitive nodes + their
  visual positions + AI insights), `access_policy` (signed policy block stating which
  recipient may see which layer), and `viewer_manifest` (auth instructions: password/
  biometric/keystore). *Source:* EKR_Research(PII §3 — bundle structure).

- **EKOS-06-010 (Should, P2)** — Bundle size SHALL be kept **small** (metadata + text;
  no heavy binaries by default) so distribution by email/secure-share/USB is trivial.
  *Source:* EKR_Research(publishing §3 — "kaafi chhota").

---

## 5. Bundle Manifest (OCI-Style)

> ChatGPT: model it like OCI container images.

- **EKOS-06-011 (Must, MVP)** — Every bundle SHALL carry a **manifest** declaring at
  least: bundle version, **runtime version, SDK version, AI version, widget versions,
  schema (ontology) versions**, **dependencies**, **capabilities**, **permissions**,
  **signatures**, and **content hashes** — exactly like an OCI image manifest. *Source:*
  ChatGPT(Bundle Manifest — manifest.yaml fields).

- **EKOS-06-012 (Must, P2)** — The viewer SHALL use the manifest's **required-viewer/
  runtime version** to verify compatibility before opening; incompatible versions
  prompt a clear upgrade message. *Source:* EKR_Research(manifest — "required viewer
  version"); ChatGPT. *Depends:* Vol 07.

- **EKOS-06-013 (Must, P2)** — The manifest SHALL declare the **capabilities &
  permissions** a bundle exercises (which connectors/resources/AI it may use) so the
  viewer can sandbox accordingly. *Source:* ChatGPT(manifest — capabilities/permissions);
  Vol 02 EKOS-02-054.

- **EKOS-06-014 (Must, MVP)** — The manifest SHALL include **content hashes** for
  integrity verification and **signatures** for authenticity (see §21). *Source:*
  ChatGPT(manifest — signatures/hashes); EKR_Research(format §1 — manifest, signature).

---

## 6. Encryption & DRM (Format Side)

> Cryptographic hardness, not obscurity (Constitution §2). Standard FIPS-validated
> libraries; the key is the secret, not the algorithm.

- **EKOS-06-015 (Must, MVP)** — Bundle data SHALL be encrypted with **AES-256-GCM** under
  a **per-bundle random data-encryption key (DEK)**. *Source:* EKR_Research(format §1;
  format-security §1).

- **EKOS-06-016 (Must, MVP)** — The DEK SHALL be wrapped using **ECIES (X25519/secp256r1)
  with the recipient's public key**, or, for password-protected bundles, derived via
  **Argon2id** (memory-hard KDF). *Source:* EKR_Research(format §1 — encryption & DRM;
  format-security §1).

- **EKOS-06-017 (Should, P2)** — For device-locked bundles, the DEK SHALL be wrapped to
  the recipient **device's hardware-backed keystore public key** (Android StrongBox,
  iOS Secure Enclave, TPM 2.0) such that the private key never leaves the chip.
  *Source:* EKR_Research(format §1; format-security §1 — device-bound keys).

- **EKOS-06-018 (Must, MVP)** — Every bundle SHALL be **digitally signed (Ed25519 or
  X.509/PKI)** by the creator's platform so the viewer can verify integrity; a tampered
  bundle SHALL NOT open. *Source:* EKR_Research(format §1 — Ed25519 signature; PII §6
  anti-tamper); §21.

- **EKOS-06-019 (Should, P2)** — The bundle MAY embed an **access control list**
  (recipient identities, expiry date). *Source:* EKR_Research(format §1 — optional ACL).

- **EKOS-06-020 (Must, MVP)** — Only **standard, open, FIPS-140-2-validated crypto
  libraries** SHALL be used — there is **no proprietary encryption**; security rests on
  the key, which is held in a chip / the user's memory (password) / a one-time
  activation token (Constitution §2). *Source:* EKR_Research(format-security §3 — "no
  proprietary encryption ... algorithm is not secret; key is secret").

- **EKOS-06-021 (Could, P3)** — A **post-quantum option** (Kyber/Dilithium) SHALL be on
  the roadmap; AES-256 treated as quantum-safe-enough for now. *Source:* EKR_Research
  (format-security §5 — quantum); Vol 04 EKOS-04-053.

---

## 7. Layered Multi-Recipient PII Encryption (Format Side of Vol 04 ABAC)

> One `.ekr` shows different things to different people, fully offline, server-free.

- **EKOS-06-022 (Must, P2)** — A single bundle SHALL support **multi-clearance via
  layered encryption**: at creation, the platform reads each node's `SecurityClassification`
  / `PersonalDataCategory` (Vol 01 §16) and encrypts **public nodes with a common key**
  (openable without auth) and **restricted nodes only with the DEKs of authorized
  recipients**. *Source:* EKR_Research(PII §1, §2).

- **EKOS-06-023 (Must, P2)** — Restricted-node encryption SHALL use **hybrid encryption**:
  node data encrypted with AES-GCM, the session key wrapped per-recipient with their
  public key (RSA/ECIES). A node may be encrypted into multiple recipient layers.
  *Source:* EKR_Research(PII §2).

- **EKOS-06-024 (Should, P2)** — The bundle SHALL offer two packaging strategies:
  **"Single Bundle, Multi-Clearance"** (one file, layered encryption — power users) and
  **"Per-Role Bundle"** (separate `.ekr` per audience — simple sharing). Both open in
  the same viewer with their own keys. *Source:* EKR_Research(PII alternative §).

- **EKOS-06-025 (Must, P2)** — The packaged `access_policy` block SHALL be **signed** and
  state which recipient may decrypt which layer (transparency). *Source:* EKR_Research
  (PII §3 — access_policy).

- **EKOS-06-026 (Must, P2)** — The bundle SHALL embed a **stripped-down ABAC evaluator's
  policy data** so the offline viewer (Vol 07) can replicate platform masking — the
  *evaluator behavior* is Vol 07; the *packaged policy* is here. *Source:* EKR_Research
  (PII §5 — local ABAC engine); Vol 04 EKOS-04-090.

- **EKOS-06-027 (Must, P2)** — On consent withdrawal or retention expiry, bundle
  **regeneration SHALL omit the node**, or the node's key SHALL **expire** after the
  retention date (GDPR/DPDP right-to-erasure, bundle side). *Source:* EKR_Research(PII
  §GDPR); Vol 04 EKOS-04-031.

---

## 8. Publishing / Generation Pipeline (Platform → `.ekr`)

- **EKOS-06-028 (Must, P2)** — A **"Publish Bundle"** action SHALL let a creator select
  one or more widget instances and options (recipients/identities, password, expiry,
  self-destruct, depth). *Source:* EKR_Research(publishing §3; self-destruct §1).
  *Note:* the authoring UX lives in Studio/Vol 07; the bundle-generation spec is here.

- **EKOS-06-029 (Must, P2)** — Generation SHALL run a background job that: (1) extracts
  the relevant **subgraph with depth control** (e.g., "include all linked components up
  to 2 hops"), (2) **AI-precomputes** required insights (crux, examples) if not cached,
  (3) **packs widget definitions**, (4) **encrypts** per the layering rules, (5)
  **signs**, (6) emits the `.ekr`. *Source:* EKR_Research(publishing §3 — workflow).

- **EKOS-06-030 (Should, P2)** — Precomputed AI insights SHALL be packaged so the viewer
  shows them offline **without re-running the AI engine** (the viewer's optional live
  AI is a Pro feature, Vol 07). *Source:* EKR_Research(format §1 — ai_insights; viewer
  limitations). *Depends:* Vol 03, Vol 07.

---

## 9. Cryptographic Expiration (Format Side — "Self-Destruct")

> Constitution §3: renamed from "self-destruct" to **Cryptographic Expiration** — we
> guarantee KEY/license/token destruction & future-access denial, not physical
> bit-destruction. Format carries the controls; the viewer ENFORCES (Vol 07).

- **EKOS-06-031 (Should, P2)** — A bundle MAY be created in **Cryptographic-Expiration
  mode**, wrapping the encrypted data in a **volatile DRM layer** and carrying a
  **time-to-live (TTL)** and/or **view-count limit** (e.g., "5 minutes after first
  open", "exactly 2 views"). *Source:* EKR_Research(self-destruct §1); Patent §1 (claim
  1, 2). *Note:* viewer-side wipe/enforcement = Vol 07.

- **EKOS-06-032 (Should, P2)** — A Cryptographic-Expiration bundle MAY require a
  **destruction attestation** — a signed proof returned to the platform (blockchain/PKI)
  once the viewer confirms key destruction (sync when online). *Source:* EKR_Research
  (self-destruct §1 — destruction attestation); Patent §1 (detailed).

- **EKOS-06-033 (Must, P2)** — Naming & wording SHALL be **"Cryptographic Expiration"**,
  and claims SHALL state **future-access denial via key/license/token destruction** —
  never "guaranteed bit destruction on someone else's machine" (Constitution §2/§3;
  ChatGPT's correction). *Source:* Constitution §3; ChatGPT(self-destruct refinement).

---

## 10. Copy-Proofing (One-Time Activation + Hardware Binding)

> Solves the "copy before opening" attack. Patent §1.

- **EKOS-06-034 (Should, P2)** — A Cryptographic-Expiration bundle SHALL be made
  **single-use, device-locked**: each copy can be consumed only **once, on one specific
  device** — unlimited copies are useless. *Source:* EKR_Research(copy-proof §);
  Patent §1.

- **EKOS-06-035 (Should, P2)** — At creation the platform SHALL generate a **unique
  Bundle ID** and a **one-time activation nonce** (in the signed manifest). *Source:*
  EKR_Research(copy-proof §1); Patent §1 (claim 3).

- **EKOS-06-036 (Should, P2)** — **First open** SHALL require **one-time activation**:
  the viewer sends a **hardware fingerprint / attestation** (Secure-Enclave pubkey, TPM
  attestation, Play Integrity / DeviceCheck) + the nonce to an **activation server**,
  which converts the nonce into a **device-bound, time-limited activation token** and
  records "Bundle ID activated for Device XYZ"; a second device requesting the same
  Bundle ID is **rejected ("already consumed")**. *Source:* EKR_Research(copy-proof §2);
  Patent §1 (claim 3). *Depends:* Vol 07 (viewer flow).

- **EKOS-06-037 (Should, P2)** — The activation server SHALL be deployable **on-prem /
  air-gapped** (e.g., government internal PKI) so activation works with no public
  internet. *Source:* EKR_Research(copy-proof §5; government-grade); Patent §1 (claim 4).

- **EKOS-06-038 (Should, P2)** — For zero-network air-gapped use, the format SHALL
  support **device pre-provisioning** (creator encrypts to a pre-registered device's
  public key), so the bundle opens only on that device with no runtime activation.
  *Source:* EKR_Research(copy-proof §5; pure-offline §); Patent §1 (claim 4).

- **EKOS-06-039 (Must, P2)** — The format/design SHALL acknowledge that **pure offline,
  unlimited-copy prevention is impossible** without some hardware-tied state; the
  practical guarantee is **one-time online activation OR pre-provisioned device
  whitelist**, both rendering copies useless. *Source:* EKR_Research(pure-offline
  scenario — honest framing).

---

## 11. Delta Bundles (`.ekrd`)

> Gemini: never regenerate a whole bundle because one node changed. Git-like.

- **EKOS-06-040 (Should, P2)** — A secondary **`.ekrd` (delta patch) format** SHALL
  exist: the platform computes the **structural graph difference** between a previously
  sent bundle version and the current state and outputs a lightweight patch containing
  only the changes. *Source:* Gemini critique (Differential Graph Delta Bundling);
  ChatGPT(Delta Bundles — "Base + Delta + Merge").

- **EKOS-06-041 (Should, P2)** — Dropping a `.ekrd` into the viewer SHALL **splice the
  new nodes/relationship modifications into the existing local base graph** without a
  full asset rebuild. *Source:* Gemini critique; ChatGPT.

- **EKOS-06-042 (Should, P2)** — Delta application SHALL preserve the **base + delta +
  merge** model (git-like), keeping bandwidth low for remote/weak-network stakeholders
  (ships at sea, field centers). *Source:* Gemini critique; ChatGPT.

---

## 12. Bundle Lifecycle Modes

- **EKOS-06-043 (Must, P2)** — The format SHALL support **Snapshot Bundle** (static —
  never changes after creation) and **Live Bundle** (receives updates when online) as
  distinct modes. *Source:* ChatGPT(Live Bundle — "Offline / Snapshot / Live").

- **EKOS-06-044 (Should, P2)** — A **Live Bundle** SHALL fetch authorized updates when
  connectivity is available, always reflecting the latest data; a **Snapshot** is
  immutable and reproducible. *Source:* ChatGPT(Live Bundle).

- **EKOS-06-045 (Should, P2)** — **Seed Bundle** (§13) SHALL be the third mode — a tiny
  payload that self-assembles on open. *Source:* blueprint §10; Patent §6.

---

## 13. Seed Mode / Nano Bot (Progressive Self-Assembly)

> Patent §6 + blueprint §10. KB-sized payload that expands into the full document.

- **EKOS-06-046 (Should, P2)** — A **Seed Bundle** SHALL be a tiny (KB-sized) encrypted
  payload containing: a signed **manifest**, **resource pointers** (APIs, cloud
  storage) with access scopes, a **pre-generated AI summary**, and a **skeleton
  (high-level) graph/layout**. *Source:* Patent §6 (claim 1); blueprint §10.

- **EKOS-06-047 (Should, P2)** — On open, the viewer SHALL **authenticate, verify the
  manifest, fetch the referenced content** (using stored EKR-SSO credentials, §22/
  Vol 12), and **progressively populate** the skeleton — instant access to the core
  while details load. *Source:* Patent §6 (claim 1, 3); blueprint §10. *Depends:* Vol 07.

- **EKOS-06-048 (Should, P2)** — The viewer SHALL display the **pre-generated AI summary
  immediately** while content streams in; skeleton renders as outline/wireframe,
  updates animate as they load. *Source:* Patent §6 (claim 2, 4).

- **EKOS-06-049 (Should, P2)** — A Seed Bundle SHALL **always reflect the most current
  data** (resources fetched live) and fall back to **last-cached state offline**.
  *Source:* Patent §6 (detailed); blueprint §10.

- **EKOS-06-050 (Could, Frontier)** — Future seed concepts: **multi-seed swarms**,
  **self-destruct seeds**, and **AI-generated bundles from an intent prompt**. *Source:*
  blueprint §10 (future).

---

## 14. Bundle Trust Chain

- **EKOS-06-051 (Should, P2)** — Each bundle SHALL carry a **trust chain** of states —
  Created By, Reviewed By, Approved By, Certified By, Expired, Revoked, Superseded —
  modeled like certificates. *Source:* ChatGPT(Bundle Trust Chain).

- **EKOS-06-052 (Should, P2)** — The viewer SHALL surface the trust state (e.g., a
  "Revoked"/"Superseded" banner) so consumers know a bundle's standing. *Source:*
  ChatGPT(Trust Chain). *Depends:* Vol 07.

---

## 15. Bundle Lineage (Git History for Bundles)

- **EKOS-06-053 (Should, P2)** — Bundles SHALL maintain **lineage**: bundle1 → bundle2 →
  bundle3 → patch → merge → fork — a queryable history of how a bundle evolved.
  *Source:* ChatGPT(Bundle Lineage — "git history for bundles"); Vol 01 §8 (genealogy).

---

## 16. Bundle Provenance

- **EKOS-06-054 (Should, P2)** — Every bundle SHALL record **provenance**: from which
  Requirement → Architecture → who generated it → from what data → when → which AI
  version → which SDK version. *Source:* ChatGPT(Bundle Provenance — "auditors will
  love this"); Vol 01 EKOS-01-034.

---

## 17. Bundle Composition

- **EKOS-06-055 (Should, P2)** — Bundles SHALL support **composition operations**: merge,
  split, extract, overlay, reference, compose — e.g., **Architecture Bundle + Security
  Bundle + Compliance Bundle = Executive Bundle**. *Source:* ChatGPT(Bundle Composition);
  Vol 02 EKOS-02-059.

- **EKOS-06-056 (Could, P3)** — Composition SHALL respect each source bundle's security
  layering — a composed bundle never elevates access beyond the composer's rights.
  *Source:* derived from Vol 04 + ChatGPT.

---

## 18. Bundle Package Manager (Dependencies)

- **EKOS-06-057 (Should, P2)** — A **package manager** SHALL resolve **bundle
  dependencies** — e.g., Bundle A depends on a Corporate Design Pack, AI Pack, Risk
  Pack, Cloud Ontology Pack. *Source:* ChatGPT(Bundle Package Manager — dependency
  resolution).

- **EKOS-06-058 (Should, P2)** — Dependency resolution SHALL verify version
  compatibility (against manifest versions, §5) and signatures before assembling.
  *Source:* derived from ChatGPT(manifest + package manager).

---

## 19. Bundle Store / Registry

> Like Docker Hub / NuGet / npm — turns distribution into an ecosystem.

- **EKOS-06-059 (Should, P2)** — An **EKOS Package Registry** SHALL let users **publish,
  version, download, verify, sign, rate, and discover** bundles & packs. *Source:*
  ChatGPT(Bundle Store — "EKOS Package Registry like Docker Hub/NuGet/npm").

- **EKOS-06-060 (Should, P2)** — The registry SHALL support **private/enterprise
  registries** and **classified marketplaces** (e.g., a defense contractor sharing a
  FedRAMP evidence pack within a closed registry). *Source:* EKR_Research(distribution);
  research(monetization — private marketplaces). *Depends:* Vol 05 (vetting), Vol 13
  (economics).

- **EKOS-06-061 (Should, P2)** — Registry entries SHALL carry trust state, provenance,
  lineage, and signatures (§14–16, §21) so consumers verify before download. *Source:*
  ChatGPT(registry — verify/sign).

---

## 20. Bundle Policy Engine

> ChatGPT: organizations control exports, NOT individual users.

- **EKOS-06-062 (Must, P2)** — A **Bundle Policy Engine** SHALL let the **organization**
  (not the individual user) set policies governing bundle creation/sharing: **Export
  Policy, Classification Policy, Retention Policy, Sharing Policy, Watermark Policy, AI
  Policy, Regional Policy**. *Source:* ChatGPT(Bundle Policy Engine).

- **EKOS-06-063 (Must, P2)** — Bundle creation SHALL be **blocked or constrained** by
  these org policies (e.g., a Restricted-classified subgraph cannot be exported to an
  unmanaged recipient). *Source:* ChatGPT(Bundle Policy Engine — "organization controls
  exports, not users"); Vol 05 (governance).

- **EKOS-06-064 (Should, P2)** — **Watermark Policy** SHALL determine whether forensic
  watermarking (Vol 04 §19 / Vol 07) is mandatory for a given classification. *Source:*
  ChatGPT(Watermark Policy); Gemini(forensic watermark).

---

## 21. Digital Signatures & Legal Sealing (Bundle Side)

> blueprint §11. Node-level signing is Vol 04; bundle-level sealing is here.

- **EKOS-06-065 (Should, P2)** — A bundle SHALL be sealable as a **legally-valid signed
  bundle** with a full **X.509/PKI certificate chain**, **RFC 3161 timestamping**, and
  **long-term validation (LTV)**. *Source:* blueprint §11 (EKR Bundle Signing).

- **EKOS-06-066 (Should, P2)** — **Multi-sign workflows** SHALL be supported on a bundle
  (chain of approval, countersign) with auditable evidence per signature event.
  *Source:* blueprint §11; Vol 04 EKOS-04-047.

- **EKOS-06-067 (Could, P3)** — Hybrid integration with **DocuSign/Adobe Sign** SHALL be
  available for bundle-level signing workflows. *Source:* blueprint §11; Vol 04
  EKOS-04-048.

---

## 22. Cross-Document Living References (Patent §7)

> References that never silently break or go stale.

- **EKOS-06-068 (Should, P2)** — A cross-reference from a source artifact to a target
  document SHALL be stored as a **graph edge** carrying the target identifier and the
  **last-verified content hash**. *Source:* Patent §7 (claim 1).

- **EKOS-06-069 (Should, P2)** — A background service SHALL **periodically re-verify the
  target** (fetch current content, compare hash); on change, update the edge's status
  (**fresh / stale / broken**) and **generate a diff**. *Source:* Patent §7 (claim 1).

- **EKOS-06-070 (Should, P2)** — Reference targets MAY be: another `.ekr` bundle (verify
  signature + summary), a web page (scrape + hash), or an authenticated API (via the
  Identity Bridge, Vol 12). *Source:* Patent §7 (claim 2; detailed).

- **EKOS-06-071 (Should, P2)** — The source widget SHALL render a **color-coded health
  badge** for each reference and offer a **one-click "Update"** that, on acceptance,
  updates the source artifact from the target's change. *Source:* Patent §7 (claim 3, 4).

- **EKOS-06-072 (Should, P2)** — If a target is inaccessible, status SHALL become
  **"broken"** with a warning. *Source:* Patent §7 (claim 4).

---

## 23. Media Types in Bundles

> blueprint §9 — every media type is a first-class semantic graph node, packageable.

- **EKOS-06-073 (Should, P2)** — Bundles SHALL package **all media types as first-class
  nodes**: static images, GIFs, videos (with AI transcription), audio, 3D models
  (glTF), interactive prototypes (Figma/XD), live dashboards, code snippets/live
  editors, social-media embeds, AR/VR content, and traditional documents (Word/PDF) —
  all **importable, traceable, and PII-maskable**. *Source:* blueprint §9 (Media Types
  Strategy). *Relates:* Vol 02 EKOS-02-051 (media-layer capability); Vol 10/14.

- **EKOS-06-074 (Should, P2)** — Embedded media SHALL inherit the bundle's security
  layering (a restricted video masks like any restricted node). *Source:* blueprint §9 +
  Vol 04.

---

## 24. Event-Driven Within a Bundle

> blueprint §13.

- **EKOS-06-075 (Should, P2)** — Every action on a bundle (open, review, sign, comment)
  SHALL emit an **immutable `Event` node** (Vol 01 §13) into the bundle's local event
  log. *Source:* blueprint §13 (event-driven).

- **EKOS-06-076 (Should, P2)** — The viewer's **local event queue** SHALL sync these
  events back to the platform when online (e.g., access logs, comments, signatures).
  *Source:* blueprint §13; EKR_Research(viewer audit logs sync). *Depends:* Vol 07.

- **EKOS-06-077 (Could, P3)** — A webhook dispatcher SHALL emit **CloudEvents-formatted**
  JSON for bundle events to external systems (full event/workflow engine in a later
  volume). *Source:* blueprint §13 (Webhook Dispatcher).

---

## 25. Format Secrecy / Anti-Reverse-Engineering

- **EKOS-06-078 (Should, P2)** — A bundle's **header and metadata SHALL also be
  encrypted** so the file reads as random noise to a hex editor — no JSON structure or
  pattern leaks; an optional magic number SHALL be HMAC-obfuscated, not plaintext.
  *Source:* EKR_Research(format-security §3 — random-looking file).

- **EKOS-06-079 (Should, P2)** — Anti-RE for the format SHALL be an **extra layer only**;
  core security is cryptographic (the key), and the viewer binary's tamper-checks
  (Vol 07) protect key handling — never security-by-obscurity. *Source:* EKR_Research
  (format-security §4); Constitution §2.

- **EKOS-06-080 (Must, MVP)** — Claims about the format SHALL use accurate engineering
  language: *"Reverse-engineering the format does not reveal protected content without
  authorized cryptographic keys"* and *"the security model assumes attackers may
  possess the encrypted file; without authorized keys, contents remain computationally
  infeasible to recover."* Forbidden: "impossible to reverse engineer", "Microsoft/
  Google can't read it". *Source:* ChatGPT(wording corrections); Constitution §2.

---

## 26. Bundle ↔ Live-Platform Consistency

- **EKOS-06-081 (Must, P2)** — A bundle SHALL be a **faithful, security-correct
  projection** of the live graph at generation time — the same ABAC model, masking, and
  widget semantics apply offline as online. *Source:* EKR_Research(PII §); Vol 04
  EKOS-04-090.

- **EKOS-06-082 (Should, P2)** — Live Bundles SHALL reconcile with the live graph via
  delta application (§11) so an offline consumer can converge to the latest authorized
  state when online. *Source:* synthesis of ChatGPT(Live Bundle) + Gemini(delta).

---

## 27. Relationship to No-Lock-In Export (Vol 01)

- **EKOS-06-083 (Must, MVP)** — `.ekr` (sealed, interactive runtime) and the **open
  JSON-LD/RDF semantic export + offline HTML/JS player** (Vol 01 §14) are **distinct and
  both required**: `.ekr` is the rich, secured distribution runtime; JSON-LD/RDF is the
  open, vendor-neutral escrow that guarantees no-lock-in even if EKOS disappears.
  *Source:* EKR_Research(no-lock-in double assurance); Gemini(no-lock-in); Vol 01
  EKOS-01-066/067.

---

## 28. Constitution Conformance (format-specific)

- **EKOS-06-084 (Must, MVP)** — Cryptographic hardness, not obscurity; accurate claims
  wording (Constitution §2) — see §6, §25. *Source:* Constitution §2.

- **EKOS-06-085 (Must, MVP)** — "Cryptographic Expiration", not "self-destruct"
  (Constitution §3) — see §9. *Source:* Constitution §3.

- **EKOS-06-086 (Must, MVP)** — Activation server, keystores, signing, registry backends
  are **illustrative & pluggable** (Constitution §6). *Source:* Constitution §6.

- **EKOS-06-087 (Must, P2)** — Air-gap support excludes only internet-only features
  (e.g., Live Bundle fetch, online activation has an on-prem equivalent) — air-gapped
  bundles use pre-provisioning + on-prem activation. *Source:* Constitution §6;
  EKR_Research(government-grade).

---

## 29. Open Questions (parked)

- **OQ-06-A** — Exact on-the-wire container spec (custom vs. extend an existing
  signed-archive standard) + versioning of the format itself — Vol 15 + a format RFC.
- **OQ-06-B** — Delta (`.ekrd`) algorithm (graph-diff representation, conflict handling
  on merge) — prototype; interplays with Vol 01 bitemporal model.
- **OQ-06-C** — Destruction-attestation backing (PKI vs. ledger/blockchain) and whether
  it's worth the complexity vs. activation-server records — design decision.
- **OQ-06-D** — Registry trust model (who certifies; cross-registry federation) — with
  Vol 05 governance + Vol 13 economics.
- **OQ-06-E** — Living-reference scraping legality/rate-limits for web targets — legal
  review; align with Vol 04 anti-exfiltration.

---

## 30. Requirement Index (this volume)

EKOS-06-001 … EKOS-06-087 (stable numbering; container/manifest/crypto/layered-
encryption/expiration/copy-proof/delta/seed/trust-chain/lineage/provenance/composition/
package-manager/registry/policy-engine/signing/living-refs/media/events/secrecy expand
to 104 distinct stated rules).

> **Next volume:** `07 — EKR Viewer Family + Cryptographic-Expiration Enforcement +
> Viewer Monetization`.
> Per protocol: STOP here. Await user "proceed".
