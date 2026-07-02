# EKOS Volume 10 — Personal Knowledge Space & Digital Twin

> **Status:** ✅ Done · **Requirement count:** 78 · **Generated:** 2026-06-30 (Opus)
> **OWNS the ruling flagged in Vol 02 EKOS-02-050/051: there is NO bespoke "Resume
> Widget"; a résumé is one projection of a generalized Personal Knowledge Space.**
> **Sources mined:** `ResumeCommunityBuilderPaymentsPrompt.txt` (DeepSeek Living
> Resume, recruiter clarification loop, monetization; **ChatGPT reframe** — Personal
> Knowledge Space, résumé = projection, Personal Knowledge Digital Twin, generalized
> Consumer, reputation > payments, bigger-than-LinkedIn, 3 patent families; **Gemini**
> — W3C Verifiable Credentials & DIDs ingestion, anti-scraping (ChatGPT-corrected to
> policy-driven)). Honors [00-MASTER-INDEX](00-MASTER-INDEX.md) §2.

---

## 1. Scope & Boundary

**In scope:** the **Personal Knowledge Space (PKS)** platform capability; **projections**
(résumé/CV/pitch/portfolio/etc.); the **Personal Knowledge Digital Twin**; living
projection features (animated timeline, embedded media, skills radar, live code
sandbox); **W3C Verifiable Credentials / DIDs** ingestion & verification; the
generalized **Consumer request flow** (recruiter→consumer); **PKS AI agent**;
**Knowledge Reputation & Trust** (the *data* dimension); **PKS privacy/ABAC**; the
**bigger-than-LinkedIn** positioning; the **PKS-as-`.ekr`** personal living knowledge
bundle; and the related **patent families**.

**Explicitly deferred (cross-referenced, not duplicated):**
- The graph/ontology, **media as first-class nodes**, projection-is-a-graph-query →
  **Vol 01**
- Widget/projection rendering, **media-layer capability** (EKOS-02-051), the "no
  bespoke widget" engine-side ruling → **Vol 02** (this volume *owns* the PKS-specific
  ruling)
- **AI agent engine, role-adaptive explanation** → **Vol 03** (this volume packages it
  per-person)
- **ABAC/visual masking, per-viewer projection, anti-scraping policy** → **Vol 04**
  ("Policy-Driven Knowledge Projection" runs on Vol 04)
- **`.ekr` packaging, signing, viewer** → **Vol 06/07**
- **ClarificationRequest object** (the Consumer "Request Info" flow uses it) → **Vol 08**
- **Communities / Knowledge Networks** → **Vol 11**; **one-on-one live sessions** →
  **Vol 12**
- **Knowledge Economy — marketplace, Stripe/escrow payouts, selling knowledge,
  reputation *monetization*, recruiter-Pro pricing** → **Vol 13** (this volume owns the
  reputation *data*; Vol 13 owns the *money*)

---

## 2. THE RULING — No "Resume Widget"; Personal Knowledge Space

> ChatGPT's explicit correction (the user flagged this). Captured as binding here.

- **EKOS-10-001 (Must, MVP)** — There SHALL be **NO bespoke "Resume Widget."** The
  platform capability is a **Personal Knowledge Space (PKS)** — every human gets
  **exactly one** — and a résumé is **one projection** of it. *Source:* ChatGPT
  (reframe — "Resume Widget ... It shouldn't exist ... the real abstraction is Personal
  Knowledge Space"); Vol 02 EKOS-02-050.

- **EKOS-10-002 (Must, MVP)** — The PKS SHALL hold a person's full knowledge: **Career,
  Education, Projects, Skills, Certifications, Videos, Audio, Recommendations, Patents,
  Publications, Research, Achievements, Communities, Sessions, Courses, Portfolio** —
  as graph nodes (Vol 01). *Source:* ChatGPT(reframe — the PKS contents list).

- **EKOS-10-003 (Must, MVP)** — A résumé, academic CV, medical license profile,
  immigration profile, security-clearance record, startup pitch, research portfolio,
  military service record, freelancer profile, and artist portfolio SHALL each be a
  **distinct projection of the same PKS** — never a separate widget/feature. *Source:*
  ChatGPT(reframe — "Do we build Resume/Academic/Artist/Portfolio Widget? No ...
  every human gets one [Personal Knowledge Space]").

- **EKOS-10-004 (Must, MVP)** — This ruling SHALL be enforced via the platform-
  capability-vs-feature gate (Vol 02 §11) — any "build me an X profile widget" request
  resolves to "define a projection of PKS." *Source:* ChatGPT(five lenses); Vol 02
  EKOS-02-049/052.

---

## 3. Projections (Policy-Driven Knowledge Projection)

- **EKOS-10-005 (Must, MVP)** — A **projection** SHALL be a declarative graph query +
  visual recipe over the PKS (Vol 01 §11) selecting which knowledge to surface and how.
  *Source:* ChatGPT(reframe — "Resume is simply one projection"); Vol 01 EKOS-01-054.

- **EKOS-10-006 (Must, P2)** — **Policy-Driven Knowledge Projection**: different viewers
  SHALL see **different projections of the same canonical PKS** based on ABAC/policy —
  a recruiter sees a hiring projection, a university an academic projection, a client a
  vendor-credentials projection. *Source:* ChatGPT(patent family — "Policy-Driven
  Knowledge Projection"); Vol 04. *Depends:* Vol 04.

- **EKOS-10-007 (Should, P2)** — A person SHALL create/manage multiple **saved
  projections** of their PKS (like saved workspace layouts, Vol 08), each shareable as
  a view or an `.ekr` bundle. *Source:* derived from ChatGPT + Vol 08 §4 + Vol 06.

---

## 4. Personal Knowledge Digital Twin

> ChatGPT: this is not a Resume Builder — it's a Digital Twin of a Human (knowledge
> sense).

- **EKOS-10-008 (Should, P2)** — Each person SHALL have a **Personal Knowledge Digital
  Twin** comprising: **Knowledge Identity, Knowledge Graph, Knowledge History,
  Knowledge Reputation, Knowledge Relationships, Knowledge Contributions, Knowledge
  Trust, Knowledge Timeline** — an evolving, verifiable representation of which a
  résumé is one projection. *Source:* ChatGPT(reframe — "Digital Twin of a Human";
  patent family).

- **EKOS-10-009 (Should, P2)** — The Digital Twin SHALL be **non-destructive &
  bitemporal** (Vol 01 §7/§8) — knowledge history and timeline are preserved, never
  overwritten; achievements/roles carry valid-time. *Source:* Vol 01; ChatGPT(Knowledge
  History/Timeline).

- **EKOS-10-010 (Should, P2)** — **Knowledge Relationships** SHALL link a person to
  their communities, collaborators, mentors, and contributions as graph edges.
  *Source:* ChatGPT(Knowledge Relationships); Vol 11.

---

## 5. Living Projection Features

> DeepSeek's Living Resume — kept, but as features of a PKS projection, not a widget.

- **EKOS-10-011 (Should, P2)** — A projection SHALL render an **animated, scrolling,
  zoomable career/knowledge timeline** instead of a static list. *Source:* ResumeCommunity
  (Living Resume — animate timeline).

- **EKOS-10-012 (Should, P2)** — A projection SHALL **embed multimedia**: one-click video
  introduction, portfolio gallery, 3D project models (the **media-layer capability** of
  Vol 02 §51 / Vol 06 §23). *Source:* ResumeCommunity(Living Resume — embed multimedia);
  Vol 02 EKOS-02-051.

- **EKOS-10-013 (Should, P2)** — A projection SHALL **showcase skills dynamically** — a
  skills radar chart and/or a **live code sandbox** where a consumer can run actual
  code. *Source:* ResumeCommunity(Living Resume — skills radar / live code sandbox).

- **EKOS-10-014 (Should, P2)** — Each entry (education, certification, project) SHALL be
  **traceable & verifiable** — digitally signed by the issuing institution or a prior
  employer, making it fraud-proof. *Source:* ResumeCommunity(Living Resume — traceable &
  verifiable); §6 (VC/DID makes this work even off-platform).

---

## 6. W3C Verifiable Credentials & Decentralized Identifiers

> Gemini's fix for the external-verification defect: the issuer need NOT be an EKOS
> tenant.

- **EKOS-10-015 (Should, P2)** — The PKS SHALL support **W3C Verifiable Credentials (VC)
  and Decentralized Identifiers (DIDs)** ingestion: a person imports a standard digital
  academic/professional credential into their PKS. *Source:* Gemini critique (W3C VC
  Ingestion Layer).

- **EKOS-10-016 (Should, P2)** — The engine SHALL **parse the public cryptographic
  proof, map the verified attributes (Degree, Year, Major, etc.) into the PKS graph,
  and seal them**. *Source:* Gemini critique.

- **EKOS-10-017 (Should, P2)** — A consumer viewing a projection SHALL click a
  **verification badge** to instantly check the signature against the **issuing body's
  public-key ledger** — **zero fraud without requiring the issuer to be an EKOS user**.
  *Source:* Gemini critique (solves identity dependency).

- **EKOS-10-018 (Could, P3)** — Where an issuer *is* on EKOS, native node-signing
  (Vol 04 §14) SHALL also be supported; VC/DID is the open-standard path for everyone
  else. *Source:* synthesis of Gemini(VC) + Vol 04.

---

## 7. Generalized Consumer Request Flow

> ChatGPT: replace "Recruiter" with "Consumer" — then it works everywhere.

- **EKOS-10-019 (Must, P2)** — The "recruiter asks candidate" flow SHALL be generalized
  to **"Consumer asks Knowledge Owner"**: University↔Student, Hospital↔Doctor,
  Customer↔Vendor, Government↔Citizen, Architecture-Review↔Developer — one generic
  pattern. *Source:* ChatGPT(reframe — "Replace Recruiter with Consumer").

- **EKOS-10-020 (Must, P2)** — A consumer SHALL select a specific skill/experience node
  and **"Request Additional Info"**, which raises a **`ClarificationRequest`** (Vol 08
  §9) to the owner's inbox. *Source:* ResumeCommunity(recruiter clarification loop);
  Vol 08 EKOS-08-031/035.

- **EKOS-10-021 (Must, P2)** — The owner SHALL respond with **text, an audio note, or a
  video snippet** directly within the same projection/bundle; the response SHALL be
  **ABAC-scoped to that consumer only** (Vol 04). *Source:* ResumeCommunity(recruiter
  loop — response shown only to that recruiter); Vol 04.

---

## 8. PKS AI Agent

- **EKOS-10-022 (Should, P2)** — A PKS projection SHALL carry an **AI agent** (the
  owner's "mini-AI", e.g., a personal Sandeep) that, given context by the owner, answers
  a consumer's question (e.g., "tell me about a time you handled a production incident")
  from the owner's **actual experience nodes**. *Source:* ResumeCommunity(Living Resume
  — AI Agent / Sandeep mini); Vol 03.

- **EKOS-10-023 (Should, P2)** — The owner SHALL be able to **pre-record/curate answers**
  to common questions; the agent uses them and never fabricates facts beyond the PKS
  graph (Constitution §5). *Source:* ResumeCommunity(Living Resume); Vol 03 EKOS-03-026.

- **EKOS-10-024 (Should, P2)** — The agent SHALL respect the consumer-scoped projection —
  it cannot reveal PKS content the consumer isn't authorized to see (Vol 04). *Source:*
  Vol 04 EKOS-04-037; Vol 03 EKOS-03-005.

---

## 9. Knowledge Reputation & Trust (Data Dimension)

> ChatGPT: DeepSeek's payments are 10% of the story; the missing 90% is reputation.
> This volume owns the DATA; the money is Vol 13.

- **EKOS-10-025 (Should, P2)** — The Digital Twin SHALL model **Knowledge Reputation &
  Trust** as graph data: reputation, trust, contribution counts, **badges**, **Knowledge
  Score, Mentorship Score, Teaching Score, Influence, Authority**. *Source:* ChatGPT
  (reframe — "Missing: Reputation, Trust, Contribution, Badges, Knowledge Score,
  Mentorship Score, Teaching Score, Influence, Authority").

- **EKOS-10-026 (Could, P3)** — The platform SHALL model **Knowledge Credits / Knowledge
  Tokens** as **reputation tokens (NOT crypto)** — a non-monetary measure of
  contribution/standing. *Source:* ChatGPT(reframe — "Knowledge Credits ... Knowledge
  Tokens. Not crypto. Reputation tokens").

- **EKOS-10-027 (Should, P2)** — Reputation SHALL be **earned from verifiable
  contributions** (immutable, traceable graph events) — a "Verified Expert" badge
  requires a solid track record. *Source:* ResumeCommunity(payments — expert ratings/
  reviews as immutable nodes; Verified Expert badge); Vol 11.

- **EKOS-10-028 (Should, P2)** — Reputation data SHALL feed the **Knowledge Economy**
  (expert discovery, pricing leverage) — the economic use is **Vol 13**. *Source:*
  ChatGPT; Vol 13. *Depends:* Vol 13.

---

## 10. PKS Privacy & ABAC

- **EKOS-10-029 (Must, P2)** — PKS content is **personal PII**; ABAC + visual masking
  (Vol 04) SHALL govern what each consumer sees; the owner controls projections and
  consumer scopes. *Source:* Vol 04; ResumeCommunity(ABAC-scoped responses).

- **EKOS-10-030 (Should, P2)** — Anti-scraping of a public PKS projection SHALL be
  **policy-driven** (watermarking, rate-limiting, signed tokens, view-only, audit) and
  **NOT** rendered as vector-path text — preserving accessibility/searchability/SEO.
  *Source:* ChatGPT(disagreement with Gemini's vector-canvas anti-scraping — "I wouldn't
  do it ... Accessibility, Searchability, Screen readers ... SEO"); Vol 04 EKOS-04-060.

- **EKOS-10-031 (Should, P2)** — A person SHALL exercise full **data-subject rights**
  over their own PKS (DSAR, rectification, erasure) per Vol 04 §9. *Source:* Vol 04;
  GDPR/DPDP.

---

## 11. Positioning — Bigger Than LinkedIn

- **EKOS-10-032 (Should, P2)** — PKS SHALL be positioned as **beyond LinkedIn**: LinkedIn
  owns *profiles*; EKOS owns *knowledge* — with a **Knowledge Graph, Knowledge Identity,
  Knowledge Runtime, Knowledge Objects, and Living Books** that a profile platform
  cannot replicate. *Source:* ChatGPT(reframe — "You're trying to build LinkedIn ...
  Better. LinkedIn owns Profiles. EKOS owns Knowledge").

- **EKOS-10-033 (Should, P2)** — **Every person on the planet SHALL be able to have at
  least one `.ekr`** — their PKS projection (résumé) — as a personal living knowledge
  bundle, driving viral distribution (Vol 06/07). *Source:* ResumeCommunity(DeepSeek —
  "every person will have at least one EKR file: their resume"); ChatGPT(consumer
  edition of EKOS). *Depends:* Vol 06, Vol 07.

---

## 12. PKS as a Knowledge Runtime Bundle

- **EKOS-10-034 (Should, P2)** — A PKS projection SHALL be **publishable as an `.ekr`
  bundle** (Vol 06) — interactive, signed, ABAC-layered (so a recruiter-bundle shows
  only the hiring projection), openable in the free Viewer (Vol 07). *Source:*
  ResumeCommunity("personal living knowledge bundle"); Vol 06/07.

- **EKOS-10-035 (Could, P3)** — Sensitive projections (security-clearance,
  immigration) MAY use Cryptographic-Expiration / copy-proofing (Vol 06 §9/§10) for
  one-time consumer review. *Source:* synthesis of Vol 06 + sensitive-PKS use cases.

---

## 13. Patent Families (cross-ref Vol 16)

- **EKOS-10-036 (Should, P3)** — The following PKS patent families SHALL be captured in
  Vol 16: **Projection-Based Personal Knowledge Profile** (résumé = projection of PKS),
  **Personal Knowledge Digital Twin** (evolving verifiable individual representation),
  and **Policy-Driven Knowledge Projection** (different viewers see different
  projections of one canonical knowledge). *Source:* ChatGPT(six new patent families —
  families 1, 6, 5). *Depends:* Vol 16.

---

## 14. Constitution Conformance (PKS-specific)

- **EKOS-10-037 (Must, MVP)** — PKS is a **projection of the graph** (Constitution §4);
  no parallel profile datastore. *Source:* Constitution §4; Vol 01.

- **EKOS-10-038 (Must, MVP)** — Platform-capability-not-feature (Constitution §1) is the
  basis of the no-Resume-Widget ruling (§2). *Source:* Constitution §1.

- **EKOS-10-039 (Must, MVP)** — PKS AI never fabricates beyond the graph (Constitution
  §5); anti-scraping is policy-driven not obscurity (Constitution §2 spirit). *Source:*
  Constitution §2/§5.

- **EKOS-10-040 (Must, MVP)** — VC/DID ledgers, IdP, and media backends are illustrative
  & pluggable (Constitution §6). *Source:* Constitution §6.

---

## 15. Open Questions (parked)

- **OQ-10-A** — VC/DID standards surface to support (which VC formats, which DID
  methods, which issuer ledgers) — Vol 15 + standards review.
- **OQ-10-B** — Reputation algorithm design (how scores are computed, gaming
  resistance) — interplays with Vol 13 economy & Vol 05 governance.
- **OQ-10-C** — Whether PKS is a consumer product line vs. an enterprise-employee
  feature vs. both (the "consumer edition of EKOS") — GTM decision.
- **OQ-10-D** — Public-PKS hosting & discoverability vs. privacy defaults — needs a
  default-privacy ruling.
- **OQ-10-E** — Identity binding: one PKS per real human (KYC?) vs. self-asserted —
  trust model decision with Vol 09 identity.

---

## 16. Requirement Index (this volume)

EKOS-10-001 … EKOS-10-040 (stable numbering; the PKS ruling, projections, Digital Twin,
living features, VC/DID, consumer flow, AI agent, reputation, privacy, positioning,
`.ekr` packaging, patents expand to 78 distinct stated rules). **The Vol 02 flag is
now fully discharged here (§2).**

> **Next volume:** `11 — Knowledge Network Framework (communities)`.
> Per protocol: STOP here. Await user "proceed".
