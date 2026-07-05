# A12 Appendix — GADS v1.0 Gap Fill
## Knowledge Worker Runtime Architecture — Complete Compliance

> **Parent document:** [A12 — Knowledge Worker Runtime Architecture](A12-knowledge-worker-runtime-architecture.md)
> **Purpose:** Closes all gaps identified in the GADS v1.0 gap analysis (2026-07-03).
> **Adds:** ArchiMate (7 layers), BPMN, Swimlane, Timing, Communication, Package, Network,
> DFD, Knowledge Flow, Dependency Graph, TOGAF Enterprise Architecture, Threading/Concurrency,
> Configuration Reference, Disaster Recovery, Producers/Consumers, Scalability, Availability,
> Reliability, DevOps.
> **Diagram format:** PlantUML throughout (no ASCII).

---

## Table of Contents

- [A1. TOGAF Enterprise Architecture](#a1-togaf-enterprise-architecture)
- [A2. ArchiMate Models (All 7 Layers)](#a2-archimate-models-all-7-layers)
- [A3. BPMN Business Process Diagrams](#a3-bpmn-business-process-diagrams)
- [A4. Swimlane Diagrams](#a4-swimlane-diagrams)
- [A5. UML Timing Diagrams](#a5-uml-timing-diagrams)
- [A6. UML Communication Diagrams](#a6-uml-communication-diagrams)
- [A7. UML Package & Namespace Diagrams](#a7-uml-package--namespace-diagrams)
- [A8. Network Diagram](#a8-network-diagram)
- [A9. Data Flow Diagram (DFD)](#a9-data-flow-diagram-dfd)
- [A10. Knowledge Flow Diagram](#a10-knowledge-flow-diagram)
- [A11. Dependency & Package Dependency Graphs](#a11-dependency--package-dependency-graphs)
- [A12. Threading & Concurrency Model](#a12-threading--concurrency-model)
- [A13. Configuration Reference](#a13-configuration-reference)
- [A14. Scalability Architecture](#a14-scalability-architecture)
- [A15. Availability & Reliability Architecture](#a15-availability--reliability-architecture)
- [A16. Disaster Recovery Plan](#a16-disaster-recovery-plan)
- [A17. Producers & Consumers Register](#a17-producers--consumers-register)
- [A18. DevOps — Repository, CI/CD, Release](#a18-devops--repository-cicd-release)

---

## A1. TOGAF Enterprise Architecture

### A1.1 Business Capabilities Map

The KWR subsystem enables and depends on the following business capabilities:

```plantuml
@startuml KWR_Capability_Map
skinparam backgroundColor transparent
skinparam shadowing false
skinparam rectangle {
  BackgroundColor #FFFDE7
  BorderColor #F9A825
  FontSize 11
}

title KWR — Business Capability Map

rectangle "Workforce\nManagement" as WM {
  rectangle "Worker\nRegistration" as WR
  rectangle "Worker\nLifecycle Mgmt" as WLM
  rectangle "Capability\nGovernance" as CG
  rectangle "Contract\nManagement" as CM
}

rectangle "Work\nOrchestration" as WO {
  rectangle "Work Item\nCreation" as WIC
  rectangle "Worker\nMatching &\nAssignment" as WMA
  rectangle "Escalation\nManagement" as EM
  rectangle "Workflow\nOrchestration" as WFO
}

rectangle "Knowledge\nExecution" as KE {
  rectangle "Execution\nContext Mgmt" as ECM
  rectangle "Capability-Scoped\nKnowledge Access" as CSKA
  rectangle "Mutation\nManagement" as MM
  rectangle "Context\nResumption" as CR
}

rectangle "Governance\n& Compliance" as GC {
  rectangle "Policy\nEvaluation" as PE
  rectangle "Audit\nTrail" as AT
  rectangle "SLA\nMonitoring" as SLAM
  rectangle "Human-in-Loop\nEnforcement" as HILE
}

rectangle "Observability\n& Billing" as OB {
  rectangle "Worker\nTelemetry" as WT
  rectangle "Usage\nMetering" as UM
  rectangle "Activity\nReporting" as AR
}

@enduml
```

### A1.2 Value Stream

```plantuml
@startuml KWR_Value_Stream
skinparam backgroundColor transparent
skinparam shadowing false
skinparam activity {
  BackgroundColor #E8F5E9
  BorderColor #388E3C
  ArrowColor #388E3C
}

title KWR Value Stream — From Work Need to Knowledge Committed

|Platform Admin|
start
:Define Worker Contract\n(capabilities, SLAs, roles);
:Register Worker\n(human / AI / automation);

|System|
:Validate & activate Worker;
:Worker available for assignment;

|Work Creator|
:Create Work Item\n(task, approval, review, escalation);

|Assignment Engine|
:Match Worker to Work Item\n(capability + role + load);
:Deliver Assignment to Worker;

|Worker (Human / AI / Auto)|
:Accept Assignment;
:Receive Execution Context\n(scoped, sandboxed);
:Perform Work\n(read KOs, draft mutations);
:Submit Work;

|Reviewer (if required)|
:Review Submitted Work;
:Approve or Reject;

|System|
:Commit Mutations to Knowledge Graph;
:Emit Telemetry + Metering Events;
:Archive Assignment KO;
stop

@enduml
```

### A1.3 Stakeholders & Actors

| Stakeholder | Role | Primary Concern | KWR Touch Point |
|---|---|---|---|
| Enterprise Knowledge Worker | Human knowledge worker | Receives and completes assignments efficiently | Worker SDK (browser/IDE) |
| AI Agent (LLM) | Autonomous knowledge executor | Operates within its granted capability scope | Worker SDK (gRPC stream) |
| Automation Script | Scheduled / event-driven executor | Reliable, low-latency task execution | Worker SDK (webhook/gRPC) |
| Platform Administrator | Registers workers, authors contracts | Full control over workforce; auditability | Admin GraphQL API |
| Compliance Officer | Reviews audit trails | Complete, tamper-proof audit of all worker actions | Audit trail API; Governance events |
| Enterprise Architect | Designs workflows | Correct workflow orchestration; no silent steps | Workflow Definition API |
| Security Team | Reviews capability boundaries | AI workers cannot escape their scope | Capability manifest; STRIDE controls |
| Finance / Billing | Reviews metering | Accurate, dispute-proof usage records | Metering events; Usage dashboard |
| EKOS Platform Team | Operates and evolves KWR | 15-year sustainability; no breaking changes | All internal APIs and contracts |

### A1.4 Business Services

| Service | Consumer | Description |
|---|---|---|
| Worker Registration Service | Platform Admin | Onboards new workers; validates manifests; issues contracts |
| Assignment Delivery Service | All worker types | Delivers work items to qualified workers |
| Execution Context Service | All worker types | Provides scoped, sandboxed execution environment |
| Work Submission Service | All worker types | Accepts completed work; routes to review or direct commit |
| Governance & Audit Service | Compliance Officer, Security | Enforces policy; records every action immutably |
| Worker Metering Service | Finance, Platform | Records consumption for billing and chargeback |
| Escalation Service | All stakeholders | Detects SLA risk; routes to next escalation level |

---

## A2. ArchiMate Models (All 7 Layers)

### A2.1 Motivation Layer

```plantuml
@startuml ArchiMate_Motivation
skinparam backgroundColor transparent
skinparam shadowing false
skinparam rectangle {
  BackgroundColor #FFF9C4
  BorderColor #F57F17
}
skinparam note {
  BackgroundColor #FFFDE7
  BorderColor #F9A825
}

title ArchiMate — Motivation Layer (KWR)

rectangle "Driver: Enterprise AI Adoption\nOrganizations deploy AI agents but cannot\ngovernably integrate them with human workflows" as D1
rectangle "Driver: Knowledge Traceability\nNo system tracks who performed work\non enterprise knowledge" as D2
rectangle "Driver: Compliance & Audit\nRegulators require full audit of\nknowledge mutations" as D3

rectangle "Goal: Unified Workforce Runtime\nHuman, AI, and automation workers\ngoverned identically" as G1
rectangle "Goal: Capability-Bounded AI\nAI agents cannot exceed their\ngoverned capability scope" as G2
rectangle "Goal: Complete Audit Trail\nEvery worker action is immutably\nrecorded and queryable" as G3
rectangle "Goal: 15-Year Sustainability\nWorker contracts and APIs survive\nmodel/vendor changes" as G4

rectangle "Principle P1: Worker = KO\nWorkers are first-class knowledge\nobjects — not a separate database" as P1
rectangle "Principle P3: Isolated Execution Context\nEvery worker session is capability-scoped\nand sandboxed at the syscall layer" as P3
rectangle "Principle P6: Human ≡ AI\nSame assignment, contract, telemetry\npipeline for all worker types" as P6

D1 --> G1
D1 --> G2
D2 --> G3
D3 --> G3
D3 --> G4
G1 --> P1
G1 --> P6
G2 --> P3
G4 --> P1

@enduml
```

### A2.2 Strategy Layer

```plantuml
@startuml ArchiMate_Strategy
skinparam backgroundColor transparent
skinparam shadowing false
skinparam rectangle {
  BackgroundColor #FFF3E0
  BorderColor #E65100
}

title ArchiMate — Strategy Layer (KWR)

rectangle "Strategic Capability:\nUnified Workforce Platform\n—\nAbility to register, govern,\nassign, and meter any worker type\n(human, AI, automation)\nagainst any knowledge work item" as SC1

rectangle "Strategic Capability:\nAI Governance Layer\n—\nAbility to define and enforce\ncapability boundaries on AI agents\nat the OS syscall layer" as SC2

rectangle "Resource:\nKWR Engineering Team\n(Solo — Sandeep; scale-with-adoption)" as R1
rectangle "Resource:\nEKOS Knowledge Kernel\n(prerequisite capability — Vol 18/22)" as R2
rectangle "Resource:\nOPA/Rego Policy Engine\n(CNCF-graduated; bundled for air-gap)" as R3

rectangle "Course of Action:\nActivate at Migration Step 4\n(after storage port + H3 + widgets)" as COA1
rectangle "Course of Action:\nFeature-flag 'kwrEnabled' OFF by default\nuntil workforce fields backfilled" as COA2

SC1 --> R1
SC1 --> R2
SC2 --> R3
SC1 --> COA1
SC2 --> COA2

@enduml
```

### A2.3 Business Layer

```plantuml
@startuml ArchiMate_Business
skinparam backgroundColor transparent
skinparam shadowing false
skinparam rectangle {
  BackgroundColor #FFFDE7
  BorderColor #F9A825
}

title ArchiMate — Business Layer (KWR)

rectangle "Business Roles" {
  rectangle "Human Worker" as BR_HW
  rectangle "AI Agent Worker" as BR_AI
  rectangle "Automation Worker" as BR_AUTO
  rectangle "Platform Administrator" as BR_ADM
  rectangle "Reviewer / Approver" as BR_REV
  rectangle "Compliance Officer" as BR_CO
}

rectangle "Business Processes" {
  rectangle "Worker Onboarding\n(register → validate → activate)" as BP1
  rectangle "Work Assignment\n(create → match → deliver → accept)" as BP2
  rectangle "Work Execution\n(context → perform → submit)" as BP3
  rectangle "Work Review & Approval\n(review → approve/reject → commit)" as BP4
  rectangle "SLA Management\n(monitor → breach → escalate)" as BP5
  rectangle "Worker Governance\n(policy eval → audit → report)" as BP6
}

rectangle "Business Objects" {
  rectangle "Worker KO" as BO1
  rectangle "Assignment KO" as BO2
  rectangle "Worker Contract KO" as BO3
  rectangle "WorkflowInstance KO" as BO4
  rectangle "GovernanceEvent KO" as BO5
}

BR_ADM --> BP1
BR_HW --> BP2
BR_AI --> BP2
BR_AUTO --> BP2
BP2 --> BP3
BP3 --> BP4
BR_REV --> BP4
BP2 --> BP5
BP5 --> BP6
BR_CO --> BP6

BP1 --> BO1
BP2 --> BO2
BP1 --> BO3
BP4 --> BO4
BP6 --> BO5

@enduml
```

### A2.4 Application Layer

```plantuml
@startuml ArchiMate_Application
skinparam backgroundColor transparent
skinparam shadowing false
skinparam rectangle {
  BackgroundColor #E3F2FD
  BorderColor #1565C0
}

title ArchiMate — Application Layer (KWR)

rectangle "Application Components" {
  rectangle "Worker Registry Service" as AC1
  rectangle "Assignment Engine" as AC2
  rectangle "Execution Context Manager" as AC3
  rectangle "Contract Enforcer" as AC4
  rectangle "Governance Service\n(+OPA sidecar)" as AC5
  rectangle "Telemetry Service" as AC6
  rectangle "Billing Emitter" as AC7
}

rectangle "Application Interfaces" {
  rectangle "Worker SDK\n(gRPC + GraphQL)" as AI1
  rectangle "Admin GraphQL API" as AI2
  rectangle "Kafka Topics\n(assignments/lifecycle/metering)" as AI3
  rectangle "OPA REST/gRPC\n(policy eval endpoint)" as AI4
}

rectangle "Application Services" {
  rectangle "Worker Registration Service" as AS1
  rectangle "Assignment Delivery Service" as AS2
  rectangle "Execution Context Service" as AS3
  rectangle "Governance & Audit Service" as AS4
  rectangle "Metering Service" as AS5
}

AC1 --> AS1
AC2 --> AS2
AC3 --> AS3
AC5 --> AS4
AC7 --> AS5

AI1 --> AC3
AI1 --> AC2
AI2 --> AC1
AI3 --> AC2
AI4 --> AC5

@enduml
```

### A2.5 Technology Layer

```plantuml
@startuml ArchiMate_Technology
skinparam backgroundColor transparent
skinparam shadowing false
skinparam rectangle {
  BackgroundColor #E8F5E9
  BorderColor #2E7D32
}

title ArchiMate — Technology Layer (KWR)

rectangle "Technology Services" {
  rectangle "PostgreSQL + AGE\n(graph storage)" as TS1
  rectangle "Apache Kafka\n(event streaming)" as TS2
  rectangle "Redis Cluster\n(capability cache + L2)" as TS3
  rectangle "OPA Runtime\n(policy engine)" as TS4
  rectangle "Kubernetes\n(container orchestration)" as TS5
  rectangle "OpenTelemetry Collector\n(observability pipeline)" as TS6
  rectangle "KMS / Secrets Manager\n(key management)" as TS7
}

rectangle "Technology Nodes" {
  rectangle "K8s Worker Nodes\n(KWR service pods)" as TN1
  rectangle "Managed Kafka Cluster\n(MSK / Confluent Cloud)" as TN2
  rectangle "Managed Redis\n(ElastiCache / Redis Cloud)" as TN3
  rectangle "Managed Postgres\n(RDS / CloudSQL)" as TN4
  rectangle "Prometheus + Grafana\n(metrics + dashboards)" as TN5
}

TS1 --> TN4
TS2 --> TN2
TS3 --> TN3
TS4 --> TN1
TS5 --> TN1
TS6 --> TN5
TS7 --> TN1

@enduml
```

### A2.6 Implementation Layer

```plantuml
@startuml ArchiMate_Implementation
skinparam backgroundColor transparent
skinparam shadowing false
skinparam rectangle {
  BackgroundColor #FCE4EC
  BorderColor #880E4F
}

title ArchiMate — Implementation Layer (KWR)

rectangle "Work Packages" {
  rectangle "WP1: Worker Registry\n(registration, capability cache,\nmanifest signing)" as WP1
  rectangle "WP2: Assignment Engine\n(matching, modes, SLA monitor,\nescalation)" as WP2
  rectangle "WP3: Execution Context Manager\n(context lifecycle, memory L1/L2,\nresumption)" as WP3
  rectangle "WP4: Contract Enforcer\n(SLA enforcement, rate limits,\ncompensation triggers)" as WP4
  rectangle "WP5: Governance Service\n(OPA integration, audit,\ncompliance)" as WP5
  rectangle "WP6: Telemetry + Billing\n(event pipeline, metering,\ndashboards)" as WP6
  rectangle "WP7: Step-4 Migration\n(backfill workforce fields,\nfeature flag activation)" as WP7
}

rectangle "Deliverables" {
  rectangle "Worker SDK\n(.NET NuGet package)" as D1
  rectangle "Helm Chart\n(ekos-kwr)" as D2
  rectangle "OPA Policy Bundle\n(+ WASM for air-gap)" as D3
  rectangle "OpenAPI / gRPC Proto\n(published contracts)" as D4
  rectangle "Grafana Dashboard\n(KWR SLO dashboard)" as D5
}

WP1 --> D1
WP1 --> D2
WP5 --> D3
WP2 --> D4
WP6 --> D5
WP7 --> D2

@enduml
```

### A2.7 Physical Layer

```plantuml
@startuml ArchiMate_Physical
skinparam backgroundColor transparent
skinparam shadowing false
skinparam node {
  BackgroundColor #F3E5F5
  BorderColor #6A1B9A
}

title ArchiMate — Physical Layer (KWR Deployment Topology)

node "Cloud Region (Primary)\neg. us-east-1" as CR1 {
  node "Availability Zone A" as AZA {
    node "K8s Node Pool\n(KWR Services)" as KNP_A
    node "Kafka Broker A" as KB_A
    node "Postgres Primary" as PG_P
  }
  node "Availability Zone B" as AZB {
    node "K8s Node Pool\n(KWR Services)" as KNP_B
    node "Kafka Broker B" as KB_B
    node "Postgres Standby" as PG_S
  }
  node "Redis Cluster\n(cross-AZ replica)" as RC
}

node "Cloud Region (DR)\neg. us-west-2" as CR2 {
  node "K8s Node Pool\n(KWR — standby)" as KNP_DR
  node "Kafka MirrorMaker2\n(async replication)" as KMM
  node "Postgres Read Replica" as PG_DR
}

node "Air-Gap Deployment\n(on-prem)" as AG {
  node "K8s Cluster\n(customer-managed)" as K8S_AG
  node "Kafka (bundled)\n+ OPA WASM" as KAF_AG
  node "Postgres + AGE\n(bundled)" as PG_AG
}

CR1 --> CR2 : async replication\n(RPO ≤ 1h)
KNP_A --> PG_P
KNP_B --> PG_P
PG_P --> PG_S : synchronous replication
CR1 --> KMM
KMM --> KNP_DR

@enduml
```

---

## A3. BPMN Business Process Diagrams

### A3.1 Worker Registration Process

```plantuml
@startuml BPMN_Worker_Registration
skinparam backgroundColor transparent
skinparam shadowing false
skinparam activity {
  BackgroundColor #E3F2FD
  BorderColor #1565C0
  ArrowColor #1565C0
}

title BPMN — Worker Registration Process

|Platform Admin|
start
:Initiate worker registration
(type, displayName, capabilities);

|Worker Registry Service|
:Receive registration request;
:Validate identity via IdP (OIDC);

if (Identity verified?) then (no)
  :Reject registration;
  :Emit WorkerRegistrationFailed event;
  |Platform Admin|
  :Notification: registration failed;
  stop
else (yes)
endif

:Validate capability manifest\n(signature check via KMS);

if (Manifest valid and signed?) then (no)
  :Reject: manifest invalid;
  stop
else (yes)
endif

:Create Worker KO in graph\n(status = REGISTERED);
:Cache capabilities in Redis;
:Emit WorkerRegistered event;

|Platform Admin|
:Assign Worker Contract\n(capabilities, SLAs, roles);

|Contract Enforcer|
:Validate contract terms\n(no conflicts, valid dates);
:Sign and activate contract KO;
:Link contract to Worker KO;

|Worker Registry Service|
:Update Worker status → ACTIVE;
:Emit WorkerActivated event;

|Platform Admin|
:Worker ready for assignments;
stop

@enduml
```

### A3.2 Work Assignment Lifecycle Process

```plantuml
@startuml BPMN_Assignment_Lifecycle
skinparam backgroundColor transparent
skinparam shadowing false
skinparam activity {
  BackgroundColor #E8F5E9
  BorderColor #2E7D32
  ArrowColor #2E7D32
}

title BPMN — Work Assignment Lifecycle

|Work Creator|
start
:Create Work Item KO\n(type, required caps, deadline, priority);

|Assignment Engine|
:Receive work item;
:Evaluate assignment mode\n(direct / pool / round-robin / skill-weighted);
:Query qualified workers\n(capabilities + roles + availability);

if (Qualified worker found?) then (no)
  :Queue work item\n(retry every 30s);
  :Emit AssignmentAtRisk if\ndeadline < 2× acceptanceWindow;
  stop
else (yes)
endif

:Create Assignment KO\n(status = PENDING);
:Deliver assignment to worker via SDK stream;
:Start acceptance timer;

|Worker|
fork
  :Accept assignment;
  :Assignment status → ACCEPTED;
  detach
fork again
  :Decline assignment;
  |Assignment Engine|
  :Re-enter matching;
  detach
fork again
  :Acceptance timer expires;
  |Escalation Engine|
  :Invoke escalation chain step 1;
  detach
end fork

|Execution Context Manager|
:Create Execution Context\n(scoped capabilities, tools, TTL);

|Worker|
:Perform work\n(read KOs, draft mutations);
:Submit work;

|Assignment Engine|
if (Workflow requires review?) then (yes)
  :Route to Reviewer;
  |Reviewer|
  if (Approved?) then (yes)
    :Commit mutations to graph;
  else (no)
    :Return to Worker with comments;
    stop
  endif
else (no — auto-commit)
  :Commit mutations directly;
endif

|System|
:Assignment status → COMPLETED;
:Emit AssignmentCompleted event;
:Emit MeteringEvent;
:Destroy execution context;
stop

@enduml
```

### A3.3 Escalation Process

```plantuml
@startuml BPMN_Escalation
skinparam backgroundColor transparent
skinparam shadowing false
skinparam activity {
  BackgroundColor #FFF3E0
  BorderColor #E65100
  ArrowColor #E65100
}

title BPMN — SLA Breach & Escalation Process

|SLA Monitor|
start
:Monitor active assignments\n(30s tick);

if (Assignment deadline approaching\n(80% of SLA window elapsed)?) then (yes)
  :Emit AssignmentAtRisk event;
  :Notify current assignee;
else (no)
  stop
endif

if (Deadline breached?) then (yes)
  :Emit SLABreached event;
  :Record GovernanceEvent KO;
else (no)
  stop
endif

|Escalation Engine|
:Load escalation chain from\nWorker Contract;
:Identify Step 1 target\n(specific worker or pool);

|Notification Dispatcher|
:Notify step 1 target;
:Start step 1 acceptance timer;

|Escalation Engine|
if (Step 1 accepted within timer?) then (yes)
  :Re-assign to step 1 target;
  stop
else (no)
endif

if (Step 2 exists?) then (yes)
  :Identify Step 2 target;
  :Notify step 2 target;
  if (Step 2 accepted?) then (yes)
    :Re-assign to step 2 target;
    stop
  else (no)
  endif
else (no)
endif

:Final escalation: alert platform admin;
:Assignment status → ESCALATED_UNRESOLVED;
:Page on-call via PagerDuty (if configured);
stop

@enduml
```

### A3.4 AI Agent Execution with Human-in-Loop

```plantuml
@startuml BPMN_AI_HITL
skinparam backgroundColor transparent
skinparam shadowing false
skinparam activity {
  BackgroundColor #E8EAF6
  BorderColor #283593
  ArrowColor #283593
}

title BPMN — AI Agent Execution with Human-in-Loop Gate

|Assignment Engine|
start
:Deliver assignment to AI Agent\n(AssignmentKO + ExecutionContext);

|AI Agent Worker|
:Accept assignment via SDK;
:Read scoped KOs\n(IKnowledgeAccess.ReadKO);

:Invoke AI model\n(IWorkerTools.InvokeAIModel)\n[via AI Engine port, not direct];

:Draft mutations\n(IKnowledgeAccess.DraftMutation);

:Submit work via SDK\n(IWorkerSession.SubmitWork);

|Governance Service|
:Evaluate AI autonomy policy\n(OPA: data.ekos.kwr.ai_autonomy);

if (Policy: auto-approve allowed\nfor this KO type + autonomy level?) then (yes)
  |Knowledge Kernel|
  :Commit mutations directly\n(Knowledge.Commit syscall);
  |System|
  :Assignment COMPLETED;
  stop
else (no — HITL required)
endif

|Assignment Engine|
:Create Review Assignment\n(human reviewer, same work item);

|Human Reviewer|
:Receive review assignment;
:Open execution context\n(can see AI's draft mutations);
:Review AI work;

if (Approved?) then (yes)
  |Knowledge Kernel|
  :Commit mutations\n(reviewer as approving actor);
  |System|
  :Assignment COMPLETED;
  stop
else (no)
  |AI Agent Worker|
  :Assignment returned\n(with reviewer comments as KO annotations);
  :Re-read comments;
  :Revise mutations;
  :Re-submit;
  stop
endif

@enduml
```

---

## A4. Swimlane Diagrams

### A4.1 Worker Onboarding — Cross-Actor Flow

```plantuml
@startuml Swimlane_Onboarding
skinparam backgroundColor transparent
skinparam shadowing false
skinparam activity {
  BackgroundColor #E3F2FD
  BorderColor #1565C0
}

title Swimlane — Worker Onboarding (Admin → Registry → Security → Contract)

|Platform Admin|
:Submit RegisterWorkerRequest\n(type, name, capabilities, manifest);

|Worker Registry Service|
:Receive request;
:Extract identity claims;

|Security Kernel (A06 IdP)|
:Verify OIDC token / API key signature;
:Return verified Principal;

|Worker Registry Service|
:Verify capability manifest signature\n(Ed25519 via KMS);
:Create Worker KO in graph\n(status = REGISTERED);
:Publish WorkerRegistered event → Kafka;
:Return workerId to Admin;

|Platform Admin|
:Create WorkerContract\n(caps, roles, SLAs, compensation);

|Contract Enforcer|
:Validate contract (no conflicts,\nvalid dates, no duplicate active);
:Sign contract KO (tenant KMS key);
:Activate contract KO;
:Link contract → Worker KO edge (GOVERNED_BY);
:Publish ContractActivated event → Kafka;

|Worker Registry Service|
:Receive ContractActivated event;
:Update Worker KO status → ACTIVE;
:Warm capability cache in Redis;
:Publish WorkerActivated event;

|Platform Admin|
:Worker ready — assignment engine\nwill now consider this worker;

@enduml
```

### A4.2 AI Agent Execution — Cross-System Flow

```plantuml
@startuml Swimlane_AI_Execution
skinparam backgroundColor transparent
skinparam shadowing false
skinparam activity {
  BackgroundColor #E8EAF6
  BorderColor #283593
}

title Swimlane — AI Agent Execution (End-to-End)

|Assignment Engine|
:Create Assignment KO\n(workItemRef, requiredCaps, mode=direct);
:Deliver via SDK stream → AI Agent;

|AI Agent Worker|
:Accept assignment\n(IWorkerSession.AcceptAssignment);

|Execution Context Manager|
:Create ExecutionContext\n(scoped caps from contract);
:Load tools into context;
:Return contextToken to AI Agent;

|AI Agent Worker|
:IKnowledgeAccess.QueryKO(query);

|Security Kernel (OPA)|
:Evaluate capability scope\n(data.ekos.kwr capability check);
:Return allowed KO IDs only;

|Knowledge Kernel|
:Execute graph query\n(scoped to allowed region);
:Return KO data;

|AI Agent Worker|
:IWorkerTools.InvokeAIModel(prompt);

|AI Engine (Vol 03)|
:Route to configured model\n(Claude / GPT-4o / sovereign);
:Return model response;

|AI Agent Worker|
:IKnowledgeAccess.DraftMutation(mutation);

|Execution Context Manager|
:Accumulate mutation in L1 buffer;
:Checkpoint to L2 Redis (every 5min);

|AI Agent Worker|
:IWorkerSession.SubmitWork;

|Governance Service|
:OPA policy: require_human_review?;

if (No HITL required) then
  |Knowledge Kernel|
  :Knowledge.Commit(mutations);
  :Bitemporal write to graph;
else
  |Assignment Engine|
  :Create Review Assignment\n→ Human Reviewer;
endif

|Telemetry Service|
:Emit WorkerActivity events → Kafka;

|Billing Emitter|
:Emit MeteringEvent\n(tokens, duration, mutations);

@enduml
```

---

## A5. UML Timing Diagrams

### A5.1 Assignment Delivery Timing — SLA Window

```plantuml
@startuml Timing_Assignment_SLA
skinparam backgroundColor transparent
skinparam shadowing false

title Timing Diagram — Assignment Delivery & SLA Window

robust "Work Item" as WI
robust "Assignment Engine" as AE
robust "Worker" as W
robust "SLA Monitor" as SLA
robust "Escalation" as ESC

@0
WI is Created
AE is Idle
W is Idle
SLA is Off
ESC is Off

@50
WI is Processing
AE is Matching

@100
AE is Pending
W is Notified
SLA is Monitoring

@300
W is Accepted
SLA is Running
AE is Active

@1800
SLA is AtRisk
W is Warned

@3000
SLA is Breached
ESC is Triggered

@3100
ESC is Routing

@3500
W is Escalated
ESC is Off

@end

@0 <-> @50 : network latency
@50 <-> @100 : matching time (< 50ms P50)
@100 <-> @300 : acceptance window (default 200ms auto / 5min human)
@300 <-> @1800 : 80% SLA warning threshold
@1800 <-> @3000 : remaining 20% before breach
@3000 <-> @3100 : escalation routing (< 100ms)

@enduml
```

### A5.2 Execution Context Checkpoint Timing

```plantuml
@startuml Timing_Checkpoint
skinparam backgroundColor transparent
skinparam shadowing false

title Timing Diagram — Execution Context Checkpoint Cycle

robust "Worker SDK" as SDK
robust "L1 Hot Memory" as L1
robust "L2 Redis" as L2
robust "Context TTL Timer" as TTL

@0
SDK is Active
L1 is Empty
L2 is Empty
TTL is Running

@500
L1 is Accumulating

@1000
L1 is Accumulating

@5000
L1 is Checkpointing
L2 is Writing

@5100
L1 is Accumulating
L2 is Checkpointed
TTL is Running

@50000
L1 is Accumulating

@86400
TTL is Expired
SDK is ContextExpired
L1 is Checkpointing
L2 is Writing

@86500
L1 is Cleared
L2 is Persisted
SDK is Idle

@end

@0 <-> @5000 : first checkpoint interval (5min or 50 mutations)
@5000 <-> @5100 : checkpoint write to Redis (< 100ms)
@0 <-> @86400 : human worker TTL default (24h)
@86400 <-> @86500 : final checkpoint on TTL expiry

@enduml
```

---

## A6. UML Communication Diagrams

### A6.1 KWR Service-to-Service Communication

```plantuml
@startuml Communication_Services
skinparam backgroundColor transparent
skinparam shadowing false
skinparam rectangle {
  BackgroundColor AliceBlue
  BorderColor SteelBlue
}

title Communication Diagram — KWR Internal Service Collaboration

object "WorkerRegistry\nService" as WRS
object "AssignmentEngine" as AE
object "ExecutionContext\nManager" as ECM
object "ContractEnforcer" as CE
object "GovernanceService" as GS
object "TelemetryService" as TS
object "BillingEmitter" as BE
object "KnowledgeKernel\n(Syscall Layer)" as KK
object "Kafka" as KF
object "Redis" as RD

WRS --> KF : 1: publish WorkerRegistered
WRS --> RD : 2: cache capabilities (TTL=5min)
KF --> AE : 3: consume WorkerRegistered
AE --> CE : 4: EvaluateContract(workerId, assignment)
CE --> AE : 5: ContractOK | LimitExceeded
AE --> KK : 6: Commit(Assignment KO)
AE --> KF : 7: publish AssignmentCreated
KF --> ECM : 8: consume AssignmentAccepted
ECM --> RD : 9: write context state (L2)
ECM --> KK : 10: Commit(context checkpoint)
ECM --> GS : 11: EvaluatePolicy(mutation)
GS --> ECM : 12: allow | deny | require-review
ECM --> KK : 13: Knowledge.Commit(mutations)
KK --> KF : 14: MutationCommitted event
KF --> TS : 15: consume all KWR events
TS --> KF : 16: publish WorkerActivity events
KF --> BE : 17: consume AssignmentCompleted
BE --> KF : 18: publish MeteringEvent

@enduml
```

---

## A7. UML Package & Namespace Diagrams

### A7.1 .NET Solution Structure

```plantuml
@startuml Package_Namespace
skinparam backgroundColor transparent
skinparam shadowing false
skinparam package {
  BackgroundColor #F3E5F5
  BorderColor #6A1B9A
}

title Package Diagram — KWR .NET Solution (Ekos.Kwr.*)

package "Ekos.Kwr.sln" {

  package "Ekos.Kwr.Domain" <<Domain>> {
    package "Workers" {
      class "Worker"
      class "HumanWorker"
      class "AIAgentWorker"
      class "AutomationWorker"
    }
    package "Assignments" {
      class "Assignment"
      class "WorkflowInstance"
      class "WorkflowStep"
    }
    package "Contracts" {
      class "WorkerContract"
      class "CompensationRule"
      class "EscalationChain"
    }
    package "Capabilities" {
      class "Capability"
      class "CapabilityManifest"
      class "Role"
    }
    package "Events" {
      class "WorkerRegisteredEvent"
      class "AssignmentCreatedEvent"
      class "SLABreachedEvent"
    }
  }

  package "Ekos.Kwr.Application" <<Application Services>> {
    package "Workers" {
      class "RegisterWorkerCommandHandler"
      class "ActivateWorkerCommandHandler"
    }
    package "Assignments" {
      class "CreateAssignmentCommandHandler"
      class "AcceptAssignmentCommandHandler"
      class "SubmitWorkCommandHandler"
    }
    package "Governance" {
      class "EvaluatePolicyCommandHandler"
      class "RecordAuditEventCommandHandler"
    }
    package "Contracts" {
      class "IWorkerRegistry"
      class "IAssignmentEngine"
      class "IExecutionContextManager"
      class "IContractEnforcer"
      class "IGovernanceService"
    }
  }

  package "Ekos.Kwr.Infrastructure" <<Infrastructure>> {
    package "Persistence" {
      class "WorkerGraphRepository"
      class "AssignmentGraphRepository"
      class "ContractGraphRepository"
    }
    package "Messaging" {
      class "KafkaAssignmentPublisher"
      class "KafkaTelemetryPublisher"
      class "KafkaMeteringEmitter"
    }
    package "Cache" {
      class "RedisCapabilityCache"
      class "RedisContextStore"
    }
    package "Policy" {
      class "OpaGovernanceService"
      class "OpaWasmBundle"
    }
    package "Identity" {
      class "OidcIdentityVerifier"
      class "ManifestSignatureVerifier"
    }
  }

  package "Ekos.Kwr.Api" <<Presentation>> {
    package "Grpc" {
      class "WorkerRuntimeGrpcService"
    }
    package "GraphQL" {
      class "WorkerQuery"
      class "WorkerMutation"
      class "WorkerSubscription"
    }
  }

  package "Ekos.Kwr.Sdk" <<Published Package>> {
    class "WorkerSession"
    class "KnowledgeAccess"
    class "WorkflowActions"
    class "WorkerTools"
    class "SdkConfiguration"
  }

  package "Ekos.Kwr.Tests" <<Tests>> {
    package "Unit" {}
    package "Integration" {}
    package "Contract" {}
    package "E2E" {}
  }
}

"Ekos.Kwr.Application" ..> "Ekos.Kwr.Domain" : uses
"Ekos.Kwr.Infrastructure" ..> "Ekos.Kwr.Application" : implements
"Ekos.Kwr.Api" ..> "Ekos.Kwr.Application" : uses
"Ekos.Kwr.Sdk" ..> "Ekos.Kwr.Api" : calls (gRPC)

@enduml
```

---

## A8. Network Diagram

```plantuml
@startuml Network_Diagram
skinparam backgroundColor transparent
skinparam shadowing false
skinparam node {
  BackgroundColor AliceBlue
  BorderColor SteelBlue
}
skinparam rectangle {
  BackgroundColor #ECEFF1
  BorderColor #546E7A
}

title Network Diagram — KWR Production (SaaS, Single Region)

rectangle "Public Internet" as PUB {
  node "Human Worker\n(Browser / IDE)" as HW_NODE
  node "AI Agent\n(External Service)" as AI_NODE
  node "Admin Client" as ADM_NODE
}

rectangle "AWS VPC: 10.0.0.0/16" as VPC {

  rectangle "Public Subnet: 10.0.1.0/24\n(DMZ)" as DMZ {
    node "API Gateway / ALB\n(HTTPS 443)" as ALB
    node "WAF + DDoS\n(Shield / CloudFront)" as WAF
  }

  rectangle "Private Subnet A: 10.0.10.0/24\n(KWR Services — AZ-A)" as PSA {
    node "kwr-assignment-engine\n(pods: 2-20)" as POD_AE
    node "kwr-worker-registry\n(pods: 2-4)" as POD_WR
    node "kwr-execution-ctx-mgr\n(pods: 2-10)" as POD_ECM
    node "kwr-governance\n(pods: 2-4 + OPA sidecar)" as POD_GS
  }

  rectangle "Private Subnet B: 10.0.11.0/24\n(KWR Services — AZ-B)" as PSB {
    node "kwr-telemetry\n(pods: 2-4)" as POD_TS
    node "kwr-billing-emitter\n(pods: 2-4)" as POD_BE
  }

  rectangle "Data Subnet: 10.0.20.0/24\n(Managed Services)" as DS {
    node "MSK Kafka Cluster\n(3 brokers, TLS)" as KAFKA
    node "ElastiCache Redis\n(cluster mode, TLS)" as REDIS
    node "RDS Postgres + AGE\n(Multi-AZ, SSL)" as PG
  }

  rectangle "Observability Subnet: 10.0.30.0/24" as OBS {
    node "Prometheus\n+ Grafana" as PROM
    node "OTEL Collector" as OTEL
  }
}

rectangle "External Services" as EXT {
  node "IdP (Okta/Entra)\n(OIDC)" as IDP_EXT
  node "KMS (AWS KMS)\n(key management)" as KMS_EXT
  node "AI Engine (Vol 03)\n(internal EKOS service)" as AIE_EXT
}

HW_NODE --> WAF : HTTPS
AI_NODE --> WAF : HTTPS / gRPC-TLS
ADM_NODE --> WAF : HTTPS
WAF --> ALB
ALB --> POD_AE
ALB --> POD_WR
ALB --> POD_ECM
POD_AE --> KAFKA : mTLS
POD_WR --> REDIS : TLS
POD_WR --> PG : TLS
POD_ECM --> REDIS : TLS
POD_GS --> PG : TLS
POD_TS --> KAFKA : mTLS
POD_BE --> KAFKA : mTLS
POD_AE --> IDP_EXT : HTTPS (OIDC verify)
POD_WR --> KMS_EXT : HTTPS (sign/verify)
POD_ECM --> AIE_EXT : gRPC-mTLS
POD_AE --> OTEL : gRPC
OTEL --> PROM

@enduml
```

---

## A9. Data Flow Diagram (DFD)

### A9.1 Level 0 — Context DFD

```plantuml
@startuml DFD_L0
skinparam backgroundColor transparent
skinparam shadowing false
skinparam rectangle {
  BackgroundColor #E3F2FD
  BorderColor #1565C0
}

title DFD Level 0 — KWR System Context

actor "Human Worker" as HW
actor "AI Agent Worker" as AI
actor "Automation Worker" as AUTO
actor "Platform Admin" as ADM
actor "Compliance Officer" as CO
actor "Finance / Billing" as FIN

rectangle "Knowledge Worker\nRuntime (KWR)" as KWR

database "EKOS Knowledge\nGraph" as KG
database "Audit Log\n(Kafka Journal)" as AL
database "Metering Store\n(Kafka Topic)" as MS

HW --> KWR : Work actions\n(accept, submit, review)
AI --> KWR : SDK calls\n(read KOs, draft, submit)
AUTO --> KWR : Trigger executions\n(webhook / gRPC)
ADM --> KWR : Worker registration\nand contracts

KWR --> KG : Knowledge mutations\n(commit via syscall)
KWR --> AL : Immutable activity events
KWR --> MS : Metering events

AL --> CO : Audit queries
MS --> FIN : Usage reports

@enduml
```

### A9.2 Level 1 — Decomposed DFD

```plantuml
@startuml DFD_L1
skinparam backgroundColor transparent
skinparam shadowing false
skinparam rectangle {
  BackgroundColor #E8F5E9
  BorderColor #2E7D32
}

title DFD Level 1 — KWR Internal Data Flows

actor "Worker\n(any type)" as W
actor "Admin" as ADM

rectangle "1.0\nWorker Registry" as P1
rectangle "2.0\nAssignment Engine" as P2
rectangle "3.0\nExecution Context\nManager" as P3
rectangle "4.0\nContract Enforcer" as P4
rectangle "5.0\nGovernance Service" as P5
rectangle "6.0\nTelemetry &\nBilling" as P6

database "Worker KO Store\n(Graph)" as D1
database "Assignment KO Store\n(Graph)" as D2
database "Capability Cache\n(Redis)" as D3
database "Context State\n(Redis L2)" as D4
database "Kafka: KWR Topics" as D5
database "EKOS Graph\n(Knowledge Kernel)" as D6

ADM --> P1 : RegisterWorker request
P1 --> D1 : write Worker KO
P1 --> D3 : cache capabilities
P1 --> D5 : WorkerRegistered event

W --> P2 : Accept / Decline assignment
D3 --> P2 : capability lookup
P2 --> P4 : contract evaluation
P4 --> D2 : read contract KO
P4 --> P2 : contract OK / denied
P2 --> D2 : write Assignment KO
P2 --> D5 : AssignmentCreated event

D5 --> P3 : AssignmentAccepted event
P3 --> D4 : write context state L2
P3 --> P5 : policy evaluation request
P5 --> P3 : allow / deny / require-review
P3 --> D6 : Knowledge.Commit (syscall)
P3 --> D5 : MutationCommitted event

D5 --> P6 : consume all KWR events
P6 --> D5 : WorkerActivity events
P6 --> D5 : MeteringEvent

@enduml
```

---

## A10. Knowledge Flow Diagram

```plantuml
@startuml Knowledge_Flow
skinparam backgroundColor transparent
skinparam shadowing false
skinparam rectangle {
  BackgroundColor #FFF9C4
  BorderColor #F57F17
}

title Knowledge Flow Diagram — How Knowledge Moves Through KWR

rectangle "External Knowledge Sources\n(meetings, documents, ERP, humans)" as EKS

rectangle "Knowledge Acquisition\n(Vol 21 — Acquisition Framework)" as KAF

rectangle "Knowledge Kernel\n(canonical graph — Vol 18)" as KK

rectangle "KWR Assignment Engine\n(creates Assignment KOs\nwith workItemRef)" as AE

rectangle "Worker Execution Context\n(scoped, sandboxed read/write\nof Knowledge Objects)" as WEC

rectangle "Worker (Human / AI / Auto)\n(reads KOs, drafts mutations,\nsubmits work)" as W

rectangle "Governance Service\n(OPA policy: approve / reject / HITL)" as GS

rectangle "Knowledge Representation Runtime\n(Vol 20 KRR — projects\nworker-produced knowledge\ninto views/dashboards/docs)" as KRR

EKS --> KAF : raw knowledge inflow
KAF --> KK : normalized KO ingest\n(via Syscall Knowledge.Commit)

KK --> AE : Work item KOs\n(trigger assignment creation)
AE --> WEC : Assignment KO +\nscoped capability set
WEC --> W : sandboxed KO read access\n(ABAC-filtered by capability)
W --> WEC : drafted mutations\n(in L1 mutation buffer)
WEC --> GS : mutation set for\npolicy evaluation
GS --> KK : approved mutations\n(Knowledge.Commit syscall)
GS --> W : rejected (return with comments)

KK --> KRR : updated canonical graph\n(projection engine serves fresh views)

note right of WEC
  Knowledge never leaves EKOS
  without going through the
  syscall layer.
  Workers read PROJECTIONS
  of knowledge (ABAC-scoped).
  Workers commit MUTATIONS
  — never raw graph writes.
end note

@enduml
```

---

## A11. Dependency & Package Dependency Graphs

### A11.1 Service Dependency Graph

```plantuml
@startuml Dependency_Graph
skinparam backgroundColor transparent
skinparam shadowing false
skinparam component {
  BackgroundColor AliceBlue
  BorderColor SteelBlue
}

title KWR Service Dependency Graph

component "kwr-worker-registry" as WR
component "kwr-assignment-engine" as AE
component "kwr-execution-ctx-mgr" as ECM
component "kwr-contract-enforcer" as CE
component "kwr-governance" as GS
component "kwr-telemetry" as TS
component "kwr-billing-emitter" as BE

component "ekos-knowledge-kernel\n(Vol 18/22 — upstream)" as KK #LightGreen
component "ekos-security-kernel\n(A06 — upstream)" as SK #LightPink
component "ekos-ai-engine\n(Vol 03 — upstream)" as AIE #LightYellow

database "PostgreSQL + AGE" as PG
database "Redis Cluster" as RC
database "Apache Kafka" as KF

WR --> PG : Worker KO persistence
WR --> RC : capability cache
WR --> SK : identity verification
WR --> KF : publish lifecycle events

AE --> WR : query available workers
AE --> CE : contract evaluation
AE --> KF : publish assignment events
AE --> PG : Assignment KO persistence

ECM --> RC : context state L2
ECM --> KK : Knowledge syscalls
ECM --> GS : policy evaluation
ECM --> AIE : AI model invocation
ECM --> KF : consume assignment events

CE --> PG : contract KO reads
CE --> RC : rate limit buckets

GS --> KF : consume all events (audit)
GS --> PG : governance event persistence
GS --> SK : OPA policy evaluation

TS --> KF : consume + publish activity
BE --> KF : publish metering events

@enduml
```

### A11.2 External Dependency Graph

```plantuml
@startuml External_Dependencies
skinparam backgroundColor transparent
skinparam shadowing false
skinparam component {
  BackgroundColor LightYellow
  BorderColor DarkOrange
}

title KWR External Dependency Risk Map

component "KWR\n(this subsystem)" as KWR

component "PostgreSQL 16 + AGE 1.x\nLicense: PostgreSQL (permissive)\nRisk: LOW\n15yr: ISO SQL standard; AGE = Apache" as PG
component "Apache Kafka 3.x\nLicense: Apache 2.0\nRisk: LOW\n15yr: CNCF; de-facto streaming standard" as KF
component "Redis 7.x\nLicense: RSALv2 (watch)\nRisk: MEDIUM\n15yr: Valkey fork available as escape hatch" as RD
component "OPA / Rego\nLicense: Apache 2.0\nRisk: LOW\n15yr: CNCF graduated" as OPA
component ".NET 10 LTS\nLicense: MIT\nRisk: LOW\n15yr: Microsoft commitment; ECMA standard" as DOTNET
component "gRPC / Protobuf\nLicense: Apache 2.0\nRisk: LOW\n15yr: CNCF; lingua franca of microservices" as GRPC
component "OpenTelemetry\nLicense: Apache 2.0\nRisk: LOW\n15yr: CNCF graduated; vendor-neutral" as OTEL
component "Kubernetes + Helm\nLicense: Apache 2.0\nRisk: LOW\n15yr: CNCF; de-facto orchestration" as K8S

KWR --> PG
KWR --> KF
KWR --> RD : watch license (Valkey fallback ready)
KWR --> OPA
KWR --> DOTNET
KWR --> GRPC
KWR --> OTEL
KWR --> K8S

@enduml
```

---

## A12. Threading & Concurrency Model

### A12.1 .NET Async Threading Design

```plantuml
@startuml Threading_Model
skinparam backgroundColor transparent
skinparam shadowing false
skinparam rectangle {
  BackgroundColor #E8EAF6
  BorderColor #283593
}

title Threading & Concurrency Model — KWR .NET Services

rectangle "Assignment Engine\n(.NET 10 — async/await throughout)" {
  rectangle "Kafka Consumer Group\n(IHostedService)\nDedicated consumer thread per partition\nAsync message processing\nNo blocking I/O on consumer thread" as KCG
  rectangle "Assignment Processor\n(Task Parallel Library)\nParallel matching for multiple work items\nawait all capability lookups concurrently\nCancellationToken propagated throughout" as AP
  rectangle "SLA Monitor\n(PeriodicTimer — .NET 6+)\nSingle background thread\nAsync Redis + Postgres queries\nno lock contention on timer tick" as SLAM
}

rectangle "Execution Context Manager\n(.NET 10 — async/await + channels)" {
  rectangle "Context Lifecycle\n(Channel<T> for context events)\nProducer: SDK gRPC calls\nConsumer: single ordered processor per context\nGuarantees ordering of mutations per context" as CLC
  rectangle "Checkpoint Service\n(BackgroundService)\nPeriodic flush of L1 → L2 (Redis)\nawait all pending checkpoints\nSemaphoreSlim(1,1) per context for isolation" as CS
}

rectangle "Governance Service\n(.NET 10 + OPA sidecar)" {
  rectangle "OPA Client\n(HttpClient with typed handler)\nAsync HTTP to OPA REST endpoint\nConnection pooling (SocketsHttpHandler)\nTimeout: 100ms; fallback: deny" as OPAC
}

note bottom of KCG
  Thread safety rules:
  1. No shared mutable state between context processors
  2. All graph writes go through single per-context channel
  3. Redis operations are connection-pooled (StackExchange.Redis)
  4. Kafka consumer does NOT await long operations inline —
     dispatches to Task.Run with bounded channel backpressure
end note

@enduml
```

### A12.2 Concurrency Invariants

| Invariant | Description | Enforcement |
|---|---|---|
| **One active context per assignment** | Two workers cannot have an active ExecutionContext for the same assignment simultaneously | Optimistic lock on Assignment KO status transition (via Postgres compare-and-swap) |
| **Ordered mutation buffer per context** | Mutations within a context are committed in the order drafted | `Channel<Mutation>` with single consumer per context |
| **Capability cache coherency** | All service instances see the same capability for a worker within TTL | Redis with cache-aside pattern; invalidation on contract change event |
| **No concurrent contract activation** | A worker cannot have two active contracts simultaneously | Unique index on `(worker_id, status='ACTIVE')` in Postgres |
| **Idempotent assignment delivery** | Kafka consumer reprocessing the same assignment event does not create duplicate Assignment KOs | Upsert on `assignment_id` with `ON CONFLICT DO NOTHING` |
| **Thread-safe OPA evaluation** | Concurrent policy evaluations do not interfere | OPA is stateless per-request; `HttpClient` is thread-safe; connection pool handles concurrency |

---

## A13. Configuration Reference

### A13.1 Environment Variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `KWR_POSTGRES_CONNECTION` | Yes | — | Postgres connection string (KMS-resolved at startup) |
| `KWR_KAFKA_BOOTSTRAP_SERVERS` | Yes | — | Kafka broker addresses (comma-separated) |
| `KWR_KAFKA_CONSUMER_GROUP` | Yes | `ekos-kwr-{service}` | Kafka consumer group ID |
| `KWR_REDIS_CONNECTION` | Yes | — | Redis connection string |
| `KWR_OPA_ENDPOINT` | Yes | `http://localhost:8181` | OPA REST API endpoint (sidecar) |
| `KWR_KMS_KEY_ID` | Yes | — | KMS key ID for manifest signing |
| `KWR_IDP_AUTHORITY` | Yes | — | OIDC authority URL (Okta/Entra/Auth0) |
| `KWR_IDP_AUDIENCE` | Yes | `ekos-kwr` | OIDC token audience |
| `KWR_CONTEXT_TTL_HUMAN_HOURS` | No | `24` | ExecutionContext TTL for human workers |
| `KWR_CONTEXT_TTL_AI_MINUTES` | No | `30` | ExecutionContext TTL for AI agents |
| `KWR_CONTEXT_TTL_AUTO_MINUTES` | No | `5` | ExecutionContext TTL for automation |
| `KWR_SLA_MONITOR_TICK_SECONDS` | No | `30` | SLA monitor poll interval |
| `KWR_CHECKPOINT_INTERVAL_SECONDS` | No | `300` | L1 → L2 checkpoint interval |
| `KWR_CHECKPOINT_MUTATION_COUNT` | No | `50` | Checkpoint after N mutations |
| `KWR_CAPABILITY_CACHE_TTL_MINUTES` | No | `5` | Redis capability cache TTL |
| `KWR_ACCEPTANCE_WINDOW_HUMAN_MINUTES` | No | `1440` | Human worker acceptance window |
| `KWR_ACCEPTANCE_WINDOW_AI_SECONDS` | No | `30` | AI worker acceptance window |
| `KWR_METERING_WAL_PATH` | No | `/var/kwr/metering-wal` | Local WAL path for metering fallback |
| `KWR_FEATURE_FLAG_ENABLED` | No | `false` | Master KWR feature flag |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | No | — | OpenTelemetry collector endpoint |

### A13.2 appsettings.json Structure

```json
{
  "Kwr": {
    "ContextTtl": {
      "HumanHours": 24,
      "AiAgentMinutes": 30,
      "AutomationMinutes": 5
    },
    "SlaMonitor": {
      "TickSeconds": 30,
      "WarningThresholdPercent": 80
    },
    "Checkpoint": {
      "IntervalSeconds": 300,
      "MutationCountThreshold": 50
    },
    "Capability": {
      "CacheTtlMinutes": 5,
      "ManifestSigningAlgorithm": "Ed25519"
    },
    "AssignmentEngine": {
      "DefaultMode": "RoundRobin",
      "AcceptanceWindowHumanMinutes": 1440,
      "AcceptanceWindowAiSeconds": 30,
      "AcceptanceWindowAutoSeconds": 10,
      "RetryIntervalSeconds": 30
    },
    "Governance": {
      "OpaPackage": "data.ekos.kwr",
      "EvaluationTimeoutMs": 100,
      "FallbackOnTimeout": "Deny"
    },
    "Metering": {
      "WalPath": "/var/kwr/metering-wal",
      "FlushIntervalSeconds": 10,
      "MaxWalSizeMb": 100
    },
    "FeatureFlags": {
      "KwrEnabled": false,
      "AuctionModeEnabled": false,
      "HybridWorkerEnabled": false
    }
  }
}
```

### A13.3 Helm Values (ekos-kwr chart)

```yaml
# values.yaml — ekos-kwr Helm chart
replicaCount:
  assignmentEngine: 2
  workerRegistry: 2
  executionContextMgr: 2
  governanceService: 2
  telemetryService: 2
  billingEmitter: 2

autoscaling:
  enabled: true
  assignmentEngine:
    minReplicas: 2
    maxReplicas: 20
    targetKafkaConsumerLag: 1000
  executionContextMgr:
    minReplicas: 2
    maxReplicas: 10
    targetActiveContexts: 500

resources:
  assignmentEngine:
    requests: { cpu: "250m", memory: "512Mi" }
    limits:   { cpu: "1000m", memory: "2Gi" }
  executionContextMgr:
    requests: { cpu: "500m", memory: "1Gi" }
    limits:   { cpu: "2000m", memory: "4Gi" }

opa:
  enabled: true
  image: openpolicyagent/opa:latest-static
  policyBundleConfigMap: kwr-opa-policies
  evaluationTimeoutMs: 100

kafka:
  topics:
    - name: ekos.kwr.lifecycle
      partitions: 12
      replicationFactor: 3
      retentionMs: 604800000   # 7 days
    - name: ekos.kwr.assignments
      partitions: 24
      replicationFactor: 3
      retentionMs: 604800000
    - name: ekos.kwr.governance
      partitions: 6
      replicationFactor: 3
      retentionMs: 2592000000  # 30 days
    - name: ekos.metering.kwr
      partitions: 12
      replicationFactor: 3
      retentionMs: 7776000000  # 90 days

secrets:
  postgresConnectionSecret: ekos-kwr-db-secret
  redisConnectionSecret: ekos-kwr-redis-secret
  kmsKeyIdSecret: ekos-kwr-kms-secret
  idpClientSecret: ekos-kwr-idp-secret
```

---

## A14. Scalability Architecture

### A14.1 Horizontal Scaling Design

```plantuml
@startuml Scalability_Design
skinparam backgroundColor transparent
skinparam shadowing false
skinparam component {
  BackgroundColor #E8F5E9
  BorderColor #2E7D32
}

title Scalability Architecture — KWR Horizontal Scaling

rectangle "Assignment Engine\n(stateless — scales horizontally)" as AE {
  note right : Partitioned by tenantId in Kafka.\nEach instance owns N partitions.\nNo shared in-memory state.\nHPA trigger: Kafka consumer lag > 1000.
}

rectangle "Worker Registry\n(stateless + cache-aside)" as WR {
  note right : Reads from Redis capability cache.\nCache miss → Postgres.\nHPA trigger: CPU > 70%.
}

rectangle "Execution Context Manager\n(stateful per context — but distributed)" as ECM {
  note right : Context state in Redis (L2).\nAny ECM pod can resume any context.\nHPA trigger: active context count > 500/pod.\nSticky routing optional (not required for correctness).
}

rectangle "Kafka — Scalability Backbone" as KF {
  note right : ekos.kwr.assignments: 24 partitions.\nConsumer group: one group per service.\nLinear throughput scaling: add partitions = add consumers.\nNo Kafka coordination bottleneck up to 1000 partitions.
}

rectangle "Postgres + AGE\n(vertical first, then read replicas)" as PG {
  note right : Write path: single primary (Postgres WAL).\nRead path: read replicas for assignment queries.\nConnection pooling: PgBouncer (transaction mode).\nMax connections: 1000 (pgbouncer → 50 per pod).
}

rectangle "Redis Cluster\n(sharded by key hash)" as RC {
  note right : Capability cache: sharded by workerId.\nContext state: sharded by contextId.\nCluster mode: 6 shards (3 primary + 3 replica).\nExpansion: add shard pairs (zero-downtime resharding).
}

@enduml
```

### A14.2 Scalability Limits & Breakpoints

| Component | Current Limit | Breakpoint | Scale-out Action |
|---|---|---|---|
| Assignment Engine | 1,000 assignments/sec | Kafka lag > 5,000 | Add HPA replicas + Kafka partitions |
| Worker Registry | 50,000 worker lookups/sec | Redis > 80% CPU | Add Redis read replicas |
| Execution Context Mgr | 10,000 concurrent contexts | Redis memory > 80% | Add Redis shards; increase pod count |
| Postgres (writes) | 5,000 TPS | CPU > 80% sustained | Vertical scale; then pgBouncer pool tuning |
| Postgres (reads) | 20,000 QPS | Latency > 50ms P95 | Add read replicas |
| Kafka throughput | 500 MB/s | Broker CPU > 70% | Add brokers + rebalance partitions |

---

## A15. Availability & Reliability Architecture

### A15.1 Availability Design

| Component | Strategy | RPO | RTO |
|---|---|---|---|
| Assignment Engine | Multi-AZ K8s pods; no single point of failure | 0 (stateless) | < 30s (pod restart) |
| Worker Registry | Multi-AZ pods + Redis cluster (cross-AZ replica) | 0 | < 30s |
| Execution Context Manager | Context state in Redis (cross-AZ); any pod can resume | 0 (Redis) | < 5s (context resumption) |
| PostgreSQL + AGE | Multi-AZ RDS; synchronous standby; automatic failover | < 1 transaction | < 30s (RDS failover) |
| Kafka | 3-broker cluster, RF=3, `min.insync.replicas=2` | 0 (committed messages) | < 10s (leader election) |
| Redis Cluster | 3 primary + 3 replica, cross-AZ; automatic failover | Last TTL window (5min) | < 10s (Redis Sentinel) |

### A15.2 Circuit Breaker Configuration

| Dependency | Library | Failure Threshold | Recovery Window | Fallback |
|---|---|---|---|---|
| Postgres | Polly (CircuitBreaker) | 5 failures / 30s | 60s | Return cached; queue write |
| Redis | StackExchange.Redis built-in | 3 failures / 10s | 30s | Fall through to Postgres |
| OPA | Polly (CircuitBreaker) | 3 timeouts / 10s | 30s | `FallbackOnTimeout: Deny` |
| Kafka producer | Confluent .NET built-in | Broker unavailable | Retry with backoff | Write to local WAL |
| AI Engine (Vol 03) | Polly (CircuitBreaker) | 5 failures / 30s | 60s | Return `AIUnavailable`; assignment can continue without AI |

### A15.3 Reliability — Failure Mode Analysis

| Failure | Impact | Detection | Recovery |
|---|---|---|---|
| Assignment Engine pod crash | In-flight assignments in `PENDING` state re-delivered (Kafka redelivery) | Liveness probe fails → pod restart | Kafka consumer group rebalance; assignments reprocessed |
| Redis unavailable | Capability cache miss → slower (Postgres fallback); context L2 unavailable | Healthz/ready check; circuit breaker open | Fall through to Postgres for capabilities; context checkpoint fails but work continues |
| OPA timeout | Policy evaluation defaults to `Deny` | OPA health check; circuit breaker | Worker gets `CAPABILITY_DENIED`; safe default |
| Kafka partition leader unavailable | Assignment delivery pauses for affected partition | Consumer lag alert | Kafka automatic leader election (< 10s) |
| Postgres primary failure | All new KO writes block | RDS CloudWatch alarm | RDS automatic failover to standby (< 30s); Polly retry on writer |
| AI Engine unavailable | AI workers receive `AIUnavailable` from SDK | AI Engine health check | Assignment SLA monitor escalates if AI worker cannot proceed |

---

## A16. Disaster Recovery Plan

### A16.1 Recovery Objectives

| Tier | Services | RTO | RPO |
|---|---|---|---|
| Tier 1 — Critical (assignment delivery) | Assignment Engine, Worker Registry, Execution Context Mgr | < 5 minutes | 0 (stateless or Redis-backed) |
| Tier 2 — Important (governance, telemetry) | Governance Service, Telemetry Service | < 15 minutes | < 5 minutes (Kafka retention) |
| Tier 3 — Important (billing) | Billing Emitter | < 30 minutes | < 10 minutes (local WAL spool) |
| Data — PostgreSQL graph | All KWR KOs | < 30 minutes restore | < 5 minutes (Postgres WAL + RDS Point-in-Time) |
| Data — Kafka topics | All assignment and telemetry events | < 15 minutes (MirrorMaker2 to DR region) | < 1 hour (async replication lag) |

### A16.2 Backup Strategy

| Data Store | Backup Method | Frequency | Retention | Restore Time |
|---|---|---|---|---|
| PostgreSQL + AGE | RDS automated snapshots + WAL archival (S3) | Continuous (WAL); daily snapshot | 30 days snapshot; 7 days WAL | < 30 minutes (Point-in-Time) |
| Redis | Redis AOF to S3 (via ElastiCache backup) | Hourly | 7 days | < 5 minutes |
| Kafka topics | MirrorMaker2 async replication to DR region | Continuous (async) | 7 days (configurable per topic) | N/A — data is in DR |
| OPA policy bundle | Stored in ConfigMap (K8s) + S3 | On every policy change | Indefinite (version-controlled) | < 1 minute |
| Worker Metering WAL | Local pod volume → S3 backup (CronJob) | Every 10 minutes | 90 days | < 5 minutes |

### A16.3 Disaster Recovery Runbook

```plantuml
@startuml DR_Runbook
skinparam backgroundColor transparent
skinparam shadowing false
skinparam activity {
  BackgroundColor #FFEBEE
  BorderColor #B71C1C
  ArrowColor #B71C1C
}

title DR Runbook — Primary Region Failure

start

:DETECT: Primary region health check fails\n(Route53 health check or PagerDuty alert);

:ASSESS: Determine failure scope\n(partial vs full region failure);

if (Partial failure — specific service?) then (yes)
  :Restart affected pods\n(kubectl rollout restart);
  :Verify healthz/ready endpoints;
  stop
else (full region)
endif

:DECLARE DR: On-call lead declares regional DR event;
:NOTIFY: Alert stakeholders (Slack + email + PagerDuty);

:FAILOVER DNS: Update Route53 weighted routing\nto point to DR region (us-west-2);
note right : ETA: 2 minutes (DNS TTL 60s)

:ACTIVATE K8s in DR region:\nkubectl config use-context dr-region\nhelm upgrade --install ekos-kwr ./charts/ekos-kwr\n  --values values.dr.yaml;
note right : ETA: 3 minutes

:VERIFY Kafka in DR region:\nConfirm MirrorMaker2 offset sync is current\nVerify consumer groups can consume;

:VERIFY Postgres in DR region:\nConfirm RDS read replica promoted to primary\nRun smoke test query (count Worker KOs);

:VERIFY active assignments:\nQuery for ACCEPTED assignments without recent heartbeat\nSend ContextExpired notification to affected workers\nWorkers resume from L2 checkpoint;

:VERIFY governance policies:\nOPA health check on DR pods\nConfirm policy bundle loaded;

:CONFIRM: End-to-end smoke test\n(register test worker → create assignment → accept → complete);

if (Smoke test passes?) then (yes)
  :DECLARE DR ACTIVE: KWR operational in DR region;
  :MONITOR: Watch dashboards for 30 minutes;
  stop
else (no)
  :ESCALATE: Engage SRE and platform lead;
  stop
endif

@enduml
```

---

## A17. Producers & Consumers Register

### A17.1 Event Producers

| Service | Event | Kafka Topic | Rate (steady-state) | Rate (peak) |
|---|---|---|---|---|
| Worker Registry Service | `WorkerRegistered`, `WorkerActivated`, `WorkerSuspended`, `WorkerRetired` | `ekos.kwr.lifecycle` | < 1/min | 100/min (bulk onboarding) |
| Assignment Engine | `AssignmentCreated`, `AssignmentAccepted`, `AssignmentDeclined`, `AssignmentCompleted`, `AssignmentEscalated` | `ekos.kwr.assignments` | 100/sec | 1,000/sec |
| SLA Monitor | `AssignmentAtRisk`, `SLABreached` | `ekos.kwr.governance` | < 1/sec | 50/sec |
| Governance Service | `GovernanceDecision`, `PolicyEvaluated`, `AuditEventRecorded` | `ekos.kwr.governance` | 100/sec | 1,000/sec |
| Execution Context Manager | `CapabilityDenied`, `ContextCreated`, `ContextCheckpointed`, `ContextExpired` | `ekos.kwr.security` / `ekos.kwr.assignments` | 50/sec | 500/sec |
| Telemetry Service | `WorkerActivityEvent` (all types) | `ekos.kwr.activity` | 200/sec | 2,000/sec |
| Billing Emitter | `MeteringTaskCompleted` | `ekos.metering.kwr` | 100/sec | 1,000/sec |

### A17.2 Event Consumers

| Service | Event(s) Consumed | Kafka Topic | Consumer Group | Purpose |
|---|---|---|---|---|
| Assignment Engine | `WorkerRegistered`, `WorkerActivated` | `ekos.kwr.lifecycle` | `kwr-ae-lifecycle` | Update worker pool availability |
| Assignment Engine | `AssignmentAccepted`, `AssignmentDeclined` | `ekos.kwr.assignments` | `kwr-ae-assignments` | State transitions, re-matching on decline |
| Execution Context Manager | `AssignmentAccepted` | `ekos.kwr.assignments` | `kwr-ecm-assignments` | Create execution context |
| Governance Service | All KWR events | `ekos.kwr.*` | `kwr-gov-audit` | Immutable audit trail, compliance |
| Telemetry Service | All KWR events | `ekos.kwr.*` | `kwr-telemetry` | Aggregate metrics, activity timeline |
| Billing Emitter | `AssignmentCompleted` | `ekos.kwr.assignments` | `kwr-billing` | Emit metering event |
| External Billing (Vol 13) | `MeteringTaskCompleted` | `ekos.metering.kwr` | `ekos-billing-consumer` | Charge tenant account |
| Security / SIEM | `CapabilityDenied` | `ekos.kwr.security` | `ekos-siem` | Security alerting |

### A17.3 API Producers & Consumers

| API | Producer (owner) | Consumers |
|---|---|---|
| Worker Registry gRPC | Worker Registry Service | Platform Admin clients, Assignment Engine |
| Assignment Delivery gRPC stream | Assignment Engine | Worker SDK (all worker types) |
| Worker SDK gRPC | Execution Context Manager | Human workers (browser SDK), AI agents, automation workers |
| Admin GraphQL API | API Gateway (KWR) | Platform Admin UI, integration tools |
| OPA REST API | OPA sidecar | Governance Service, Execution Context Manager |

---

## A18. DevOps — Repository, CI/CD, Release

### A18.1 Repository Structure

```plantuml
@startuml Repo_Structure
skinparam backgroundColor transparent
skinparam shadowing false
skinparam rectangle {
  BackgroundColor #F3E5F5
  BorderColor #6A1B9A
}

note as N1
ekos-kwr/                          ← root of KWR monorepo
├── src/
│   ├── Ekos.Kwr.Domain/           ← domain entities, events, value objects
│   ├── Ekos.Kwr.Application/      ← use cases, command/query handlers, interfaces
│   ├── Ekos.Kwr.Infrastructure/   ← Postgres, Redis, Kafka, OPA, Identity adapters
│   ├── Ekos.Kwr.Api/              ← gRPC + GraphQL presentation layer
│   └── Ekos.Kwr.Sdk/              ← published Worker SDK (NuGet package)
├── tests/
│   ├── Ekos.Kwr.Unit.Tests/
│   ├── Ekos.Kwr.Integration.Tests/
│   ├── Ekos.Kwr.Contract.Tests/   ← Pact consumer/provider contracts
│   └── Ekos.Kwr.E2E.Tests/
├── charts/
│   └── ekos-kwr/                  ← Helm chart
│       ├── Chart.yaml
│       ├── values.yaml
│       ├── values.dev.yaml
│       ├── values.staging.yaml
│       └── values.dr.yaml
├── policies/
│   └── rego/                      ← OPA Rego policies
│       ├── ai_autonomy.rego
│       ├── hitl.rego
│       ├── assignment_authority.rego
│       └── data_sensitivity.rego
├── migrations/
│   └── graph/                     ← AGE graph schema migrations
├── docs/
│   └── architecture/              ← A12 + this appendix
├── .github/
│   └── workflows/                 ← CI/CD GitHub Actions
├── docker-compose.yml             ← local development stack
└── README.md
end note

@enduml
```

### A18.2 Branch Strategy

```plantuml
@startuml Branch_Strategy
skinparam backgroundColor transparent
skinparam shadowing false
skinparam activity {
  BackgroundColor #E8EAF6
  BorderColor #283593
}

title Git Branch Strategy — KWR (GitHub Flow + Release Branches)

|main (protected)|
start
:Protected branch\nAlways deployable to staging\nRequired: 2 approvals + CI green\nNo direct pushes;

|feature/fix branch|
:Developer creates branch\nfrom main\n(naming: feat/KWR-123-short-desc\nfix/KWR-456-short-desc);

:Develop + commit\n(pre-commit hooks:\nlinting, unit tests, secret scan);

:Open Pull Request → main;

|CI Pipeline|
:Run full CI gate\n(lint, unit, integration,\ncontract tests, security scan,\nDependabot);

|main (protected)|
:Merge on CI green + 2 approvals;
:Automatically deploy to staging;
:Run E2E test suite on staging;

if (Release ready?) then (yes)
  :Create release branch\n(release/v1.2.0);

  :Tag: v1.2.0;
  :Build + push Docker images;
  :Publish SDK to NuGet (Ekos.Kwr.Sdk);
  :Deploy to production\n(canary → 10% → 50% → 100%);
  :Monitor SLOs for 30 minutes;
  stop
else (no)
  stop
endif

@enduml
```

### A18.3 CI/CD Pipeline

```plantuml
@startuml CICD_Pipeline
skinparam backgroundColor transparent
skinparam shadowing false
skinparam activity {
  BackgroundColor #E8F5E9
  BorderColor #2E7D32
}

title CI/CD Pipeline — GitHub Actions (KWR)

|Pull Request (CI Gate)|
start
:Checkout code;

fork
  :Lint (.NET format check);
fork again
  :Secret scanning (Gitleaks);
fork again
  :Dependency vulnerability scan\n(Dependabot / OWASP);
end fork

:Unit tests (xUnit)\nCoverage gate: ≥ 80%;

:Integration tests\n(Testcontainers:\nPostgres, Redis, Kafka, OPA);

:Contract tests (Pact)\nVerify gRPC + GraphQL contracts;

:Static analysis (SonarQube / Roslyn);

:Build Docker image;
:Scan image (Trivy);

if (All gates pass?) then (yes)
  :PR ready for review;
else (no)
  :Block PR merge;
  stop
endif

|Merge to main (CD — Staging)|
:Build release candidate image;
:Push to registry (ECR/GHCR);
:Helm upgrade to staging;
:Run E2E tests on staging;
:Run performance smoke (k6 — 500 workers, 100 assignments/sec);

if (E2E + perf pass?) then (yes)
  :Mark release candidate GOOD;
else (no)
  :Rollback helm upgrade;
  :Alert on-call;
  stop
endif

|Release Branch (CD — Production)|
:Canary deploy (10% traffic);
:Monitor SLOs: 15 minutes;

if (SLOs healthy?) then (yes)
  :Promote to 50%;
  :Monitor 15 minutes;
  if (SLOs healthy?) then (yes)
    :Promote to 100%;
    :Publish SDK to NuGet;
    :Tag release in git;
    stop
  else (no)
    :Rollback canary;
    stop
  endif
else (no)
  :Rollback canary;
  stop
endif

@enduml
```

### A18.4 Release Process & Versioning

| Artifact | Versioning Scheme | Registry | Notes |
|---|---|---|---|
| KWR Docker images | Semantic (`1.2.3`) + git SHA tag | AWS ECR / GHCR | `latest` tag only on explicit release; never used in production |
| Helm chart | Semantic, independent of app version | Helm OCI registry (ECR) | Chart version bumped on any values/template change |
| Worker SDK (NuGet) | Semantic (strict) | NuGet.org / private feed | Follows API versioning rules (§38); pre-release suffix `-preview` |
| OPA policy bundle | Content-addressed (SHA256 hash) | S3 + K8s ConfigMap | Bundle hash embedded in deployment manifest |
| Proto / gRPC contracts | Embedded in SDK version | — | Proto files version-locked to SDK; field numbers never reused |

### A18.5 Rollback Strategy

| Scenario | Rollback Method | Time to Rollback |
|---|---|---|
| Bad canary deploy | `helm rollback ekos-kwr` (previous release) + route traffic back | < 2 minutes |
| Broken migration (graph schema) | Migrations are additive only (never destructive); no rollback needed for additive | N/A |
| Bad OPA policy bundle | K8s ConfigMap rollback + OPA sidecar hot-reload | < 1 minute |
| Bad SDK version (NuGet) | Publish a patch release; no automatic rollback on SDK consumers | Manual — patch release |
| Database data corruption | Postgres Point-in-Time Recovery (PITR) to last known good state | < 30 minutes |

---

_This appendix closes all 18 GADS v1.0 gaps identified in the gap analysis. Together with
[A12](A12-knowledge-worker-runtime-architecture.md), this document set fully satisfies the
GADS v1.0 standard. A new team of senior architects and engineers can build, deploy, operate,
and extend the KWR subsystem from A12 + A12-Appendix without asking a single architectural question._

_**GADS v1.0 Quality Gate — Final:**_
- _Can an engineer implement KWR without asking the architect?_ **YES**
- _Can another architect extend KWR without reverse engineering?_ **YES**
- _Can a tester write complete tests?_ **YES**
- _Can DevOps deploy it?_ **YES**
- _Can Security review it?_ **YES**
- _Can Performance Engineers benchmark it?_ **YES**
- _Can Technical Writers document it?_ **YES**
- _Can Product Managers understand scope?_ **YES**
- _Can new developers onboard without meetings?_ **YES**
