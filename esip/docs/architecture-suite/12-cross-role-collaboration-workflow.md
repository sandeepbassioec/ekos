# Volume 12 — Cross-Role Collaboration & Workflow Engine

**Prefix:** WFL · **Personas:** All (P1–P13) · **Standards/refs:** BPMN 2.0, workflow/state-machine patterns, RACI, ITIL change/approval, e-sign (links to Vol 07).

**Scope statement.** Architecture work is rarely done by one role alone. To complete a task in one role, a user routinely needs something from a user in a *different* role — **clarity** on an ambiguous requirement, a **missing requirement** filled in, an **artifact/decision**, a **review**, or an **approval/sign-off before rollout**. This volume specifies the **workflow engine** that makes all of this first-class: typed cross-role requests, routing, approvals/gates, SLAs/escalation, a per-user work inbox, notifications, a no-code workflow builder, and audit/analytics — wired into all three pillars and every persona.

**Reuse anchors (build on, don't replace):** ESIP already models **Confusions** and **Clarifications** and **Gaps** in the Knowledge Hub, **roles** (admin/manager/developer/architect/viewer) with route gates, **approval pages** (ManagerApproval), and **toast notifications** (sonner). Cross-role requests should extend these, route them to the right role/user, and track them to resolution. **UI:** use the existing Radix `dialog`, `dropdown-menu`, `tabs`, `badge`, `card`, `table`, `sonner` components and current layout patterns — no new visual language.

> **Table-schema note (intentional compact form).** This volume uses a deliberately compact requirement table — `ID | Requirement (SHALL) | Cat | Pri` — rather than the full 00-MASTER-INDEX §6 column set. This is **intentional, not an omission**: **Persona** defaults to **"All" (P1–P13)** for every WFL requirement unless the row text names a specific role, and the **Rationale/Source** is stated **inline** within the requirement text (often in **bold** lead-ins). The compact schema is semantically equivalent to the canonical schema for this workflow-engine volume.

---

## WFL-1 — Collaboration Model & Participants

| ID | Requirement (SHALL) | Cat | Pri |
|----|---------------------|-----|-----|
| REQ-WFL-001 | The platform SHALL treat every architecture role (P1–P13) as a potential workflow participant who can both raise requests to and receive requests from any other role. | Functional | Must |
| REQ-WFL-002 | The platform SHALL allow any work item (requirement, gap, confusion, roadmap item, diagram, deliverable, portfolio app, ADR, migration wave, etc.) to be the *subject* of a cross-role request. | Functional | Must |
| REQ-WFL-003 | The platform SHALL record, for every request, the raiser (role+user), the target (role and/or specific user), the subject artifact, type, status, and full timeline. | Functional | Must |
| REQ-WFL-004 | The platform SHALL support participants who are internal users, role-groups, or external/guest users (e.g., a client approver). | Functional | Should |
| REQ-WFL-005 | The platform SHALL associate each request with a project/workspace and respect that workspace's access control. | Security | Must |
| REQ-WFL-006 | The platform SHALL let a request target a *role* (any qualified holder may pick it up) or a *named user* (directed). | Functional | Must |
| REQ-WFL-007 | The platform SHALL maintain a participant directory mapping users↔roles↔skills↔availability for routing. | Functional | Should |

## WFL-2 — Request Types (the cross-role "asks")

> Each type shares a common lifecycle (raised → routed → in-progress → responded/decided → resolved/closed, with reject/rework branches) but carries type-specific fields.

| ID | Requirement (SHALL) | Cat | Pri |
|----|---------------------|-----|-----|
| REQ-WFL-010 | **Clarification Request (RFI):** SHALL let a user ask another role to clarify an ambiguous requirement/artifact; the response SHALL be linkable back to the subject (and to an existing Knowledge Hub *Confusion* where applicable). | Functional | Must |
| REQ-WFL-011 | **Missing-Requirement Request:** SHALL let a user request that another role supply a missing/implied requirement; an accepted response SHALL create or update a requirement in the Requirements pillar (Vol 07) with provenance to the request. | Functional | Must |
| REQ-WFL-012 | **Information/Artifact Request:** SHALL let a user request a document, data, diagram, or decision input from another role, attachable to the Knowledge Hub. | Functional | Must |
| REQ-WFL-013 | **Review Request:** SHALL let a user request a review of an artifact, capturing reviewer comments, findings, and a review outcome (approve/changes-requested/reject). | Functional | Must |
| REQ-WFL-014 | **Approval Request / Sign-off:** SHALL let a user request formal approval before an artifact is published/rolled out, with an auditable decision and optional e-signature (Vol 07). | Governance | Must |
| REQ-WFL-015 | **Decision Request:** SHALL let a user escalate a decision (e.g., option selection, ADR ratification) to a deciding role, recording the decision and rationale (link to ADR). | Functional | Should |
| REQ-WFL-016 | **Change Request:** SHALL let a user request a change to a baselined artifact, routing through impact analysis (Vol 07) and approval. | Governance | Must |
| REQ-WFL-017 | **Hand-off / Assignment:** SHALL let a user hand work to another role (e.g., EA → Solution Architect → Data Architect) with context, acceptance, and ownership transfer. | Functional | Must |
| REQ-WFL-018 | **Escalation:** SHALL let any request be escalated to a higher authority/role on breach, disagreement, or risk. | Functional | Must |
| REQ-WFL-019 | The platform SHALL allow new request types to be defined via configuration (fields, lifecycle, routing) without code — see metamodel (Vol 11). | Functional | Should |

## WFL-3 — Workflow Engine & State Machines

| ID | Requirement (SHALL) | Cat | Pri |
|----|---------------------|-----|-----|
| REQ-WFL-030 | The platform SHALL provide a configurable workflow engine driving each request type through a defined state machine (states, transitions, guards, actions). | Functional | Must |
| REQ-WFL-031 | The engine SHALL support sequential, parallel, and conditional steps. | Functional | Must |
| REQ-WFL-032 | The engine SHALL support multi-stage approvals, including N-level chains and parallel approvers. | Governance | Must |
| REQ-WFL-033 | The engine SHALL support quorum/threshold approval (e.g., any 2 of 3, majority, unanimous). | Governance | Should |
| REQ-WFL-034 | The engine SHALL support conditional routing based on attributes (artifact type, risk, cost, persona, value, tags). | Functional | Must |
| REQ-WFL-035 | The engine SHALL support delegation (an approver delegates to another) and reassignment, with audit. | Functional | Must |
| REQ-WFL-036 | The engine SHALL support recall/withdraw of a raised request before it is actioned. | Functional | Should |
| REQ-WFL-037 | The engine SHALL support rework/return loops (e.g., changes-requested sends the item back to the raiser). | Functional | Must |
| REQ-WFL-038 | The engine SHALL trigger workflows automatically on events (e.g., "deliverable marked ready" → approval workflow; "requirement flagged ambiguous by AI" → clarification request). | Functional | Should |
| REQ-WFL-039 | The engine SHALL persist workflow instances durably and resume safely after interruptions. | NFR | Must |
| REQ-WFL-040 | The engine SHALL prevent conflicting concurrent transitions on the same item (optimistic locking). | NFR | Must |

## WFL-4 — Approval Gates & Sign-off (before rollout)

> **Boundary/dependency note (C4 — external mutation under an active gate).** When an artifact is under an **active approval gate**, *any* external mutation of that artifact — including **AI auto-apply of review feedback (V15)**, automated edits, connector writes, or branch merges — MUST either be **blocked** while the gate is open, or **create a new revision and reset the approval** per **REQ-WFL-055** (the approval is bound to the exact version; a material change invalidates it and re-approval is required). This is the V12 side of the **V15↔V12** dependency; **V15 AI-apply MUST check for and honour an active V12 gate** before mutating, and MUST surface the reset to approvers. See REQ-WFL-057 below and 00-MASTER-INDEX (C4).

| ID | Requirement (SHALL) | Cat | Pri |
|----|---------------------|-----|-----|
| REQ-WFL-050 | The platform SHALL allow publishing/rollout of an artifact (deliverable, architecture, roadmap, portfolio decision) to be gated behind required approvals. | Governance | Must |
| REQ-WFL-051 | The platform SHALL block the gated action until all required approvals are obtained, and record who approved/when. | Governance | Must |
| REQ-WFL-052 | The platform SHALL support configurable gate policies per artifact type, persona, and risk/cost threshold. | Governance | Should |
| REQ-WFL-053 | The platform SHALL capture rejection with reason and route the item back for rework. | Governance | Must |
| REQ-WFL-054 | The platform SHALL support optional e-signature on approval (integrating Vol 07 e-sign) and produce a sign-off certificate. | Governance | Should |
| REQ-WFL-055 | The platform SHALL version the approved artifact and bind the approval to that exact version (re-approval required after material change). | Governance | Must |
| REQ-WFL-056 | The platform SHALL support an Architecture Review Board (ARB) gate, assembling the review package and recording the board decision (links to TOGAF governance, Vol 01). | Governance | Should |

### WFL-4 — Reconciliation Additions

> **GAP/contradiction C4.** Added to make the V15↔V12 interaction enforceable on the workflow-engine side. Reciprocal obligation lives in **V15** (AI-apply must detect an active gate before mutating).

| ID | Requirement (SHALL) | Cat | Pri |
|----|---------------------|-----|-----|
| REQ-WFL-057 | While an artifact is under an active approval gate, the platform SHALL intercept every **external mutation** of that artifact — explicitly including **V15 AI auto-apply of review feedback**, automated/agentic edits, connector-driven writes, and branch merges — and SHALL either (a) **block** the mutation until the gate resolves, or (b) **create a new revision and reset/re-open the approval per REQ-WFL-055**, recording the reset event (actor, source, timestamp) in the audit trail and notifying all assigned approvers. Silent mutation of a gated artifact SHALL NOT be permitted. *(Persona: All. Reciprocal cross-ref: V15 AI-apply MUST check for an active V12 gate before applying changes.)* | Governance | Must |

## WFL-5 — Clarification & Missing-Requirement Loop (Knowledge-Hub-integrated)

| ID | Requirement (SHALL) | Cat | Pri |
|----|---------------------|-----|-----|
| REQ-WFL-060 | The platform SHALL let AI-detected Confusions/Gaps/ambiguities (existing Knowledge Hub features) be converted into routed clarification/missing-requirement requests to the appropriate role with one action. | AI/ML | Must |
| REQ-WFL-061 | When a clarification is answered, the platform SHALL update the originating Confusion/Gap to Clarified/Resolved and attach the answer with author and timestamp. | Functional | Must |
| REQ-WFL-062 | The platform SHALL let AI suggest the most appropriate target role/user for a clarification based on the artifact's domain and the participant directory. | AI/ML | Should |
| REQ-WFL-063 | The platform SHALL track unresolved clarifications/missing-requirements as blockers on the dependent task and surface them in dashboards. | Functional | Must |
| REQ-WFL-064 | The platform SHALL allow a single request to reference multiple subjects and a single subject to have multiple open requests. | Functional | Should |

## WFL-6 — Work Inbox & "My Work"

| ID | Requirement (SHALL) | Cat | Pri |
|----|---------------------|-----|-----|
| REQ-WFL-070 | Each user SHALL have a unified inbox with tabs: *Assigned to me*, *Awaiting my approval/response*, *Raised by me*, *Following/CC*. | UX | Must |
| REQ-WFL-071 | The inbox SHALL show due dates, SLA status, priority, subject artifact, and requesting role, with filter/sort/search. | UX | Must |
| REQ-WFL-072 | The inbox SHALL support bulk actions (approve/reject/reassign) where policy allows. | UX | Should |
| REQ-WFL-073 | The platform SHALL provide a team/role queue view so a role-group can pick up unassigned requests. | Functional | Should |
| REQ-WFL-074 | The inbox SHALL deep-link to the subject artifact in context (e.g., open the exact requirement/diagram/deliverable). | UX | Must |
| REQ-WFL-075 | The inbox SHALL reuse the existing UI component library (tabs/table/badge/card) for visual consistency. | UX | Must |

## WFL-7 — Routing & Assignment

| ID | Requirement (SHALL) | Cat | Pri |
|----|---------------------|-----|-----|
| REQ-WFL-080 | The platform SHALL route a request to a role, a named user, or a queue, per the workflow definition. | Functional | Must |
| REQ-WFL-081 | The platform SHALL support auto-assignment strategies: round-robin, least-loaded, skill-match, and explicit. | Functional | Should |
| REQ-WFL-082 | The platform SHALL respect availability/out-of-office and reroute or delegate accordingly. | Functional | Could |
| REQ-WFL-083 | The platform SHALL allow manual (re)assignment by authorized users with audit. | Functional | Must |
| REQ-WFL-084 | The platform SHALL prevent routing to a user lacking permission on the subject artifact. | Security | Must |

## WFL-8 — SLAs, Timers, Escalation & Reminders

| ID | Requirement (SHALL) | Cat | Pri |
|----|---------------------|-----|-----|
| REQ-WFL-090 | The platform SHALL allow SLA/due-date policies per request type and priority. | Governance | Must |
| REQ-WFL-091 | The platform SHALL send reminders before due and overdue notices after breach. | Functional | Must |
| REQ-WFL-092 | The platform SHALL auto-escalate on SLA breach per policy (e.g., to manager/ARB). | Governance | Must |
| REQ-WFL-093 | The platform SHALL pause SLA timers in defined wait states (e.g., awaiting external input) and resume them. | Functional | Should |
| REQ-WFL-094 | The platform SHALL report SLA attainment and breach trends. | Reporting | Should |

## WFL-9 — Notifications & Communication

| ID | Requirement (SHALL) | Cat | Pri |
|----|---------------------|-----|-----|
| REQ-WFL-100 | The platform SHALL notify participants in-app (reusing the existing toast/notifications) and via email for new/assigned/approaching-due/decided requests. | Functional | Must |
| REQ-WFL-101 | The platform SHALL support threaded comments/discussion on each request with @mentions that notify and pull the mentioned user in. | Functional | Must |
| REQ-WFL-102 | The platform SHALL support per-user notification preferences and a digest mode. | UX | Should |
| REQ-WFL-103 | The platform SHALL optionally deliver notifications to Teams/Slack/email via connectors (Vol 08). | Integration | Could |
| REQ-WFL-104 | The platform SHALL let users follow/subscribe to a request or artifact to receive updates. | Functional | Should |

## WFL-10 — No-Code Workflow Builder

| ID | Requirement (SHALL) | Cat | Pri |
|----|---------------------|-----|-----|
| REQ-WFL-110 | The platform SHALL provide a visual, no-code workflow/state-machine builder (states, transitions, conditions, approvers, SLAs, notifications). | Functional | Should |
| REQ-WFL-111 | Workflow definitions SHALL be governed configuration in the metamodel (Vol 11): versioned, validated, import/export, tenant-overridable. | Governance | Should |
| REQ-WFL-112 | The platform SHALL ship starter workflow templates (RFI, approval gate, ARB review, change control, hand-off chain) that tenants can clone and adapt. | Functional | Should |
| REQ-WFL-113 | The builder SHALL simulate/dry-run a workflow before activation. | Functional | Could |
| REQ-WFL-114 | The platform SHALL support a BPMN-compatible representation for portability/import-export. | Integration | Could |

## WFL-11 — Audit, Status Tracking & Analytics

| ID | Requirement (SHALL) | Cat | Pri |
|----|---------------------|-----|-----|
| REQ-WFL-120 | The platform SHALL maintain an immutable, timestamped audit trail of every request/workflow action (who/what/when/why). | Governance | Must |
| REQ-WFL-121 | The platform SHALL provide a "what's blocking what" view across the project (open requests blocking requirements/roadmap items/deliverables). | Reporting | Must |
| REQ-WFL-122 | The platform SHALL report cycle time, throughput, bottleneck roles, and rework rates. | Reporting | Should |
| REQ-WFL-123 | The platform SHALL show per-artifact request history (all clarifications/approvals/changes on it). | Functional | Must |
| REQ-WFL-124 | The platform SHALL expose workflow status via API/webhooks for external dashboards (Vol 11). | Integration | Should |

## WFL-12 — Pillar & Persona Integration

| ID | Requirement (SHALL) | Cat | Pri |
|----|---------------------|-----|-----|
| REQ-WFL-130 | **Requirements pillar:** a requirement SHALL be able to carry open clarification/missing-req/change requests, and its status SHALL reflect blocking requests. | Functional | Must |
| REQ-WFL-131 | **Roadmap pillar:** a roadmap item SHALL support stakeholder sign-off and dependency-clarification requests before commitment. | Functional | Should |
| REQ-WFL-132 | **Knowledge Hub:** evidence/answers supplied via requests SHALL be filed into the Hub with provenance. | Functional | Must |
| REQ-WFL-133 | **Portfolio/sunset (Vol 09):** a sunset/migration disposition SHALL require approval from the application owner role before action. | Governance | Must |
| REQ-WFL-134 | **Deliverables (Vol 10):** publishing a deliverable SHALL be able to require review+approval gates. | Governance | Must |
| REQ-WFL-135 | **Diagrams/models (Vol 03):** a model/view SHALL support review and approval before being marked authoritative/baselined. | Governance | Should |
| REQ-WFL-136 | **EA↔SA↔role hand-offs:** the platform SHALL support a guided hand-off chain reflecting the differing EA/SA vs role-based journeys (e.g., EA target architecture → SA solution design → Data/Security/Cloud detailed design). | Functional | Must |

## WFL-13 — External / Guest Participation

| ID | Requirement (SHALL) | Cat | Pri |
|----|---------------------|-----|-----|
| REQ-WFL-140 | The platform SHALL allow an external/guest user to be invited to respond to or approve a specific request with scoped, time-bounded access. | Security | Should |
| REQ-WFL-141 | Guest approvals SHALL be auditable and bound to the artifact version, identical to internal approvals. | Governance | Should |
| REQ-WFL-142 | The platform SHALL support email-based approve/respond for low-friction external participation, with verification. | Functional | Could |

## WFL-14 — NFR & Governance for the Workflow Engine

| ID | Requirement (SHALL) | Cat | Pri |
|----|---------------------|-----|-----|
| REQ-WFL-150 | Workflow actions SHALL be authorized by RBAC/ABAC (Vol 11); only permitted roles may approve/decide. | Security | Must |
| REQ-WFL-151 | The engine SHALL scale to many concurrent workflow instances without degrading inbox/notification latency (target < 2s). | NFR | Should |
| REQ-WFL-152 | Workflow data SHALL be tenant-isolated and retained per policy. | Security | Must |
| REQ-WFL-153 | The platform SHALL degrade gracefully (queue + retry) if a notification channel is unavailable, never losing a request. | NFR | Must |

---

### Boundary notes
- Identity/RBAC/ABAC, multi-tenancy, the metamodel that stores workflow definitions → **Vol 11**.
- Requirement records, RTM, e-sign primitive, impact analysis → **Vol 07**.
- AI suggestions (target-role inference, auto-clarification drafting) → **Vol 06**.
- Connector-based notification delivery (Teams/Slack/email) → **Vol 08**.
- Existing ESIP **Confusions/Gaps/Clarifications**, **roles**, **ManagerApproval**, and **toast notifications** are the reuse seeds for this engine.

**Count:** ~95 requirements (prefix WFL) across 14 capability areas.
