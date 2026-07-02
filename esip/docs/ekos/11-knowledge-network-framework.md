# EKOS Volume 11 — Knowledge Network Framework

> **Status:** ✅ Done · **Requirement count:** 58 · **Generated:** 2026-06-30 (Opus)
> **Sources mined:** `ResumeCommunityBuilderPaymentsPrompt.txt` (Community & Instant
> Live Help, Community Building, social-network-for-knowledge, network-effects
> flywheel), **`ChatGPT` reframe** (Community → Knowledge Network platform capability;
> "Communities are only one type"; Enterprise/University/Research-Lab/NGO/Government/
> Family/Startup/Friends/Conference). Honors [00-MASTER-INDEX](00-MASTER-INDEX.md) §2.

---

## 1. Scope & Boundary

**In scope:** the **Knowledge Network** platform capability (of which "community" is
one type); network types & visibility; network-as-subgraph; **network creation**;
**membership/roles/expert-mode**; **expert directory & discovery** ("Find a Live
Expert"); **Q&A forums & shared knowledge**; **group sessions scheduling**;
**document-anchored help** entry; the **feedback loop** to the Knowledge Quality
Engine; **network effects**; **network governance**; **network privacy/ABAC**; and
monetization/cross-org **hooks**.

**Explicitly deferred (cross-referenced, not duplicated):**
- **One-on-one live sessions, WebRTC/P2P, live-call-anchored-to-widget, shared
  whiteboard overlay, mDNS discovery** → **Vol 12** (this volume finds the expert &
  starts the request; Vol 12 runs the connection)
- **Expert/community monetization — Stripe Connect/Treasury, escrow, payouts, creator
  cut, transaction fees, expert-Pro subscription** → **Vol 13**
- **Reputation/badges/Knowledge-Score *data*** → **Vol 10** (the Digital Twin owns the
  data; the network *surfaces* it)
- **ClarificationRequest object & "Clarified" → KQE feed** → **Vol 08/05** (this volume
  references the loop)
- **ABAC/masking, cross-org partial visibility primitive** → **Vol 04**; **network
  governance policy engine** → **Vol 05**
- The graph/ontology (`Stakeholder`, member/community nodes, edges) → **Vol 01**

---

## 2. Core Reframe — Community → Knowledge Network

> ChatGPT: "DeepSeek thinks Community. I think Knowledge Network. Community is one
> implementation; Knowledge Network is the platform capability."

- **EKOS-11-001 (Must, P2)** — The platform capability SHALL be a **Knowledge Network**,
  not merely "communities"; a community is **one type** of Knowledge Network. *Source:*
  ChatGPT(reframe — "Community is one implementation; Knowledge Network is the platform
  capability").

- **EKOS-11-002 (Must, P2)** — The framework SHALL support these network types as
  configurations of one capability: **Enterprise, Community, University, Research Lab,
  NGO, Government, Family, Startup, Friends, Conference** — and be extensible to others.
  *Source:* ChatGPT(reframe — the network-types list).

- **EKOS-11-003 (Must, P2)** — Networks SHALL be **public, private, or enterprise-gated**.
  *Source:* ResumeCommunity(Community — public/private; enterprise-gated).

---

## 3. Network as Graph

- **EKOS-11-004 (Must, P2)** — Each Knowledge Network SHALL be a **subgraph** (Vol 01)
  holding member profiles, skill badges, scheduled sessions, and a shared Q&A/knowledge
  set — fully traceable and governed like everything else. *Source:* ResumeCommunity
  (Community — "own graph ... member profiles, skill badges, scheduled group sessions,
  Q&A forum"); Vol 01.

- **EKOS-11-005 (Should, P2)** — Member profiles in a network SHALL **link to each
  member's PKS** (Vol 10) rather than duplicating data. *Source:* synthesis of
  ResumeCommunity(member profiles) + Vol 10 (PKS single source). *Depends:* Vol 10.

---

## 4. Network Creation

- **EKOS-11-006 (Must, P2)** — **Any user SHALL create a Knowledge Network** around a
  knowledge domain (e.g., "Python for Data Science", "Stanford CS 101", "Acme Corp
  Internal Experts"). *Source:* ResumeCommunity(Community Building — "any user can
  create a community around a knowledge domain").

- **EKOS-11-007 (Should, P2)** — A network SHALL be configurable as **free, paid
  (subscription), or enterprise-gated** at creation; the *payment mechanics* are Vol 13.
  *Source:* ResumeCommunity(Community Building — free/paid/enterprise-gated). *Depends:*
  Vol 13.

---

## 5. Membership, Roles & Expert Mode

- **EKOS-11-008 (Should, P2)** — Users SHALL **join networks** and hold roles within
  them (member, expert, moderator, owner). *Source:* ResumeCommunity(Community/payments).

- **EKOS-11-009 (Should, P2)** — Any user SHALL enable **"Expert Mode"** in their profile
  — declaring expertise areas, availability, and (if monetized) rates; this creates an
  expert profile node linked to the community graph. *Source:* ResumeCommunity(payments
  — Expert Onboarding). *Depends:* Vol 13 (rates).

- **EKOS-11-010 (Should, P2)** — Member skill badges SHALL be **earned/verifiable** and
  reflect the member's PKS reputation (Vol 10 §9). *Source:* ResumeCommunity(skill
  badges); Vol 10.

---

## 6. Expert Directory & Discovery

- **EKOS-11-011 (Must, P2)** — While viewing any widget or EKR book, a user SHALL click
  **"Find a Live Expert"**, and the system SHALL search the relevant **Community
  Knowledge Graph** — a public community, a private community, or the document's own
  **verified contributors**. *Source:* ResumeCommunity(Community & Instant Live Help —
  "Find a Live Expert").

- **EKOS-11-012 (Must, P2)** — The user SHALL see a list of **available mentors/experts**
  with their **ratings, expertise badges, and availability status**. *Source:*
  ResumeCommunity(— "list of available mentors/experts with ratings, badges,
  availability").

- **EKOS-11-013 (Should, P2)** — From the directory the user SHALL **initiate a live
  session** with an expert — the connection itself (WebRTC/P2P, widget-anchored,
  whiteboard) is **Vol 12**. *Source:* ResumeCommunity(initiate voice/video call within
  Viewer); Vol 12. *Depends:* Vol 12.

---

## 7. Q&A Forum & Shared Knowledge

- **EKOS-11-014 (Should, P2)** — Each network SHALL have a **shared Q&A forum** and
  shared knowledge set accessible inside the EKR Viewer / platform. *Source:*
  ResumeCommunity(Community — Q&A forum; "accessible directly inside the EKR Viewer").

- **EKOS-11-015 (Should, P2)** — Q&A items and shared knowledge SHALL be **graph nodes**
  (traceable, searchable, reputation-affecting). *Source:* synthesis; Vol 01, Vol 10.

---

## 8. Group Sessions

- **EKOS-11-016 (Should, P2)** — Networks SHALL support **scheduled group sessions**
  (the live runtime is Vol 12). *Source:* ResumeCommunity(Community — scheduled group
  sessions). *Depends:* Vol 12.

---

## 9. Document-Anchored Help & Feedback Loop

- **EKOS-11-017 (Must, P2)** — Help SHALL be **anchored to the exact widget/book** the
  user was viewing, so the expert/session opens at that spot (the live anchoring is
  Vol 12). *Source:* ResumeCommunity(— "session is anchored to the exact widget");
  Vol 12.

- **EKOS-11-018 (Should, P2)** — After help, both parties SHALL leave a **review**, and
  the learner SHALL mark the confusing part **"Clarified"** — feeding the **Knowledge
  Quality Engine** (Vol 05) so the document author learns which sections need
  improvement. *Source:* ResumeCommunity(— review + "Clarified" → KQE); Vol 05
  EKOS-05-012; Vol 08 EKOS-08-034. *Depends:* Vol 05, Vol 08.

---

## 10. Network Effects — Social Network for Knowledge

- **EKOS-11-019 (Should, P2)** — The framework SHALL be positioned as a **"social
  network for knowledge"** — not just a document viewer — solving the human factor that
  AI alone can't. *Source:* ResumeCommunity(— "social network for knowledge").

- **EKOS-11-020 (Should, P2)** — The design SHALL drive a **flywheel**: more people →
  more available experts → more learners → more content creators. *Source:*
  ResumeCommunity(— network-effects flywheel).

---

## 11. Network Governance

- **EKOS-11-021 (Should, P2)** — A **community owner** SHALL control which experts are
  allowed in their community and (if monetized) the **community cut** on sessions; the
  *governance enforcement* is Vol 05 and the *money* is Vol 13. *Source:*
  ResumeCommunity(payments — "community owner decides which experts ... community cut").

- **EKOS-11-022 (Should, P2)** — Network membership, expert allow-lists, and content
  visibility SHALL be governed by policy (Vol 05) and ABAC (Vol 04). *Source:* Vol 04,
  Vol 05.

---

## 12. Network Privacy & ABAC

- **EKOS-11-023 (Must, P2)** — During a session/interaction, the requester's personal
  details (real email, phone) SHALL be **automatically masked** until they choose to
  reveal them (ABAC, Vol 04). *Source:* ResumeCommunity(payments — "student's personal
  details automatically masked"); Vol 04.

- **EKOS-11-024 (Should, P2)** — Private/enterprise-gated networks SHALL enforce
  membership-scoped visibility; public networks default to privacy-preserving exposure.
  *Source:* Vol 04; ResumeCommunity(public/private).

---

## 13. Monetization & Cross-Org Hooks (Owned Elsewhere)

- **EKOS-11-025 (Should, P2)** — Expert sessions, community subscriptions, and creator
  cuts SHALL be monetizable; **all payment mechanics (Stripe Connect/Treasury, escrow,
  payouts, fees, expert-Pro) are Vol 13**. *Source:* ResumeCommunity(payments); Vol 13.
  *Depends:* Vol 13.

- **EKOS-11-026 (Could, P3)** — Knowledge Networks SHALL support **cross-organization
  membership** (a prime + suppliers in one governed network) using the cross-org
  partial-visibility primitive (Vol 04 §17). *Source:* synthesis; Vol 04 EKOS-04-057,
  Vol 05 EKOS-05-049.

---

## 14. Constitution Conformance (network-specific)

- **EKOS-11-027 (Must, P2)** — Networks are **subgraphs** of the single source of truth;
  member data references PKS, not duplicated (Constitution §4). *Source:* Constitution
  §4; Vol 10.

- **EKOS-11-028 (Must, P2)** — Network capability is a **platform capability, not a
  feature** (Constitution §1) — "community" is one configuration. *Source:* Constitution
  §1; ChatGPT.

- **EKOS-11-029 (Should, P2)** — Interaction/discovery backends are illustrative &
  pluggable (Constitution §6). *Source:* Constitution §6.

---

## 15. Open Questions (parked)

- **OQ-11-A** — Discovery/search at scale across many public networks (indexing,
  ranking) — interplays with Vol 01 §15.
- **OQ-11-B** — Default privacy posture for public PKS-linked member profiles —
  resolve with Vol 10 OQ-10-D.
- **OQ-11-C** — Moderation/abuse handling in public networks — governance design with
  Vol 05.
- **OQ-11-D** — Family/Friends network types vs. enterprise governance — do they need a
  lighter governance profile? GTM/product decision.

---

## 16. Requirement Index (this volume)

EKOS-11-001 … EKOS-11-029 (stable numbering; network reframe/types, creation,
membership/expert-mode, directory/discovery, Q&A, group sessions, document-anchored
help + feedback loop, network effects, governance, privacy, monetization/cross-org
hooks expand to 58 distinct stated rules).

> **Next volume:** `12 — Knowledge Session Runtime (P2P / mDNS / WebRTC, transport-
> pluggable)`.
> Per protocol: STOP here. Await user "proceed".
