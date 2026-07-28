# EAIP — Live Status Tracker
**Tool:** EAIP (Enterprise Architecture Intelligence Platform — also called EAVIP)  
**Repository:** `C:\eaip\` (standalone project, Rust + React + graph engine)  
**Relationship:** Direct predecessor to EKOS — the architecture intelligence substrate  
**Last Updated:** 2026-07-09

> **Rule:** This file is updated after every session.

---

## What EAIP Is

EAIP (also called EAVIP — Enterprise Architecture Visibility & Intelligence Platform) is a
dedicated platform for mapping, governing, and intelligencing enterprise architecture.

It answers the question: **"What does our enterprise technology landscape actually look like,
and is it healthy?"**

EAIP is the **architecture intelligence substrate** that EKOS builds upon. Where EKOS is the
full Knowledge Operating System for the enterprise, EAIP is the targeted tool focused purely
on architecture visibility, drift detection, and governance automation.

### Value Proposition

```
Before EAIP:  Enterprise architecture lives in Visio diagrams, Confluence pages, and
              spreadsheets — all immediately stale after the last edit.

After EAIP:   Architecture graph auto-ingested from Terraform, K8s, CMDB, CI/CD.
              Drift detected automatically (policy-as-code).
              Time-travel queries: "What did this system look like 6 months ago?"
              Impact analysis: "If I retire Service X, what breaks?"
              C4/ArchiMate/Mermaid diagrams auto-generated — always current.
```

---

## Architecture: Three-Tier Design

```
┌──────────────────────────────────────────────────────────────────┐
│ Tier 1: Ingestion Layer                                          │
│   Terraform parser → K8s manifest parser → CloudFormation parser │
│   CMDB export reader → API agents (REST, GraphQL)                │
│   CI/CD pipeline connectors (GitHub Actions, Azure DevOps)       │
│   Push model (webhooks) + Pull model (scheduled scans)           │
├──────────────────────────────────────────────────────────────────┤
│ Tier 2: Enterprise Graph Engine                                  │
│   Property graph database (bitemporal: validFrom + validTo)      │
│   9 node types, 7 edge types                                     │
│   Time-travel queries: as-is, to-be, historical snapshots        │
│   Policy-as-code drift detection (OPA / Rego)                    │
│   Audit trail: every change logged with who, what, when          │
├──────────────────────────────────────────────────────────────────┤
│ Tier 3: Intelligence & Presentation                              │
│   Auto-generated diagrams: C4, dependency maps, impact analysis  │
│   Governance dashboard (drift scoring, health metrics)           │
│   Export formats: Mermaid, ArchiMate, SVG, PNG, PDF              │
│   API: REST + GraphQL + gRPC                                     │
│   Plugin marketplace (connectors + visualizations)               │
└──────────────────────────────────────────────────────────────────┘
```

---

## Graph Model

### Node Types (9)

| Node Type | What It Represents |
|---|---|
| `Application` | Software application or system (e.g., "Inventory Service") |
| `Service` | Microservice or API endpoint within an application |
| `DataStore` | Database, cache, message queue, object storage |
| `Infrastructure` | Server, VM, container cluster, cloud region, VPC |
| `BusinessCapability` | Business function (e.g., "Order Fulfillment") |
| `Risk` | Risk item with severity, status, mitigation link |
| `Technology` | Technology/framework in use (e.g., "React 18", "PostgreSQL 15") |
| `Person` | Individual contributor, architect, or owner |
| `Team` | Organizational team owning one or more applications |
| `ExternalSystem` | Third-party or partner system outside org boundary |

All node types carry `validFrom` and `validTo` timestamps for bitemporal storage.

### Edge Types (7)

| Edge Type | Meaning |
|---|---|
| `DEPENDS_ON` | A depends on B (runtime dependency) |
| `HOSTS` | Infrastructure hosts Application/Service |
| `MANAGES` | Person/Team manages Application/Service |
| `USES_TECHNOLOGY` | Application/Service uses a Technology |
| `OWNS` | Team/Person owns a resource |
| `CONTRIBUTES_TO` | Application/Service contributes to a BusinessCapability |
| `RISKS_TO` | Risk applies to a node |

---

## Formal Requirements (FR-001 to FR-010)

| FR | Requirement | Status |
|---|---|---|
| FR-001 | Ingest architecture data from Terraform, CloudFormation, K8s, CMDB | 🔵 Documented |
| FR-002 | Build and maintain a graph of architecture components and relationships | 🔵 Documented |
| FR-003 | Detect drift between declared architecture and actual deployed state | 🔵 Documented |
| FR-004 | Support time-travel queries (as-is at any historical date, to-be planning) | 🔵 Documented |
| FR-005 | Auto-generate C4 diagrams, dependency maps, impact analysis | 🔵 Documented |
| FR-006 | Enforce architecture policies via policy-as-code (OPA/Rego) | 🔵 Documented |
| FR-007 | Expose REST + GraphQL + gRPC APIs for all graph operations | 🔵 Documented |
| FR-008 | Maintain full audit trail of all architecture changes | 🔵 Documented |
| FR-009 | Support plugin marketplace for custom connectors and visualizations | 🔵 Documented |
| FR-010 | Multi-tenant support with organization/workspace isolation | 🔵 Documented |

---

## Non-Functional Requirements (NFRs)

| NFR | Target | Status |
|---|---|---|
| Graph query p95 latency | < 200ms | 🔵 Defined |
| Availability | 99.9% | 🔵 Defined |
| GDPR compliance | Full compliance | 🔵 Defined |
| SOC 2 Type II | Required for Enterprise tier | 🔵 Defined |
| Ingestion throughput | 100K nodes/min during initial scan | 🔵 Defined |
| Time-travel query performance | < 500ms for historical graphs | 🔵 Defined |

---

## Commercial Model

| Tier | Pricing | Features |
|---|---|---|
| Community | Free, OSS | Core graph, basic ingestion, community connectors |
| Professional | Per-seat, monthly | Full ingestion, governance, diagram export, API access |
| Enterprise | Unlimited seats, SLA, SSO | Multi-tenant, SCIM, SSO, RDMA/high-perf graph, dedicated support |

**Plugin Marketplace:** connectors and visualizations sold/free through marketplace.
Revenue share: 70% to plugin publisher, 30% to EAIP platform.

---

## Governance and Drift Detection

### Policy-as-Code

Architecture policies written in Rego (OPA):

```rego
# Example: Every service must have an owner Team
deny[msg] {
  service := input.graph.nodes[_]
  service.type == "Service"
  not graph.has_edge(service.id, "MANAGED_BY", "Team")
  msg := sprintf("Service '%s' has no owning team", [service.id])
}
```

Policies run automatically after every ingestion cycle. Violations are:
1. Logged to the audit trail
2. Surfaced on the governance dashboard with severity score
3. Available via API for CI/CD gating

### Drift Scoring

Each drift violation carries a severity weight. The governance dashboard computes:
```
DriftScore = Σ(violation_severity × violation_weight) / total_architecture_elements

Healthy:   DriftScore < 5
Warning:   DriftScore 5–20
Critical:  DriftScore > 20
```

---

## Ingestion Connectors

| Connector | Format/Protocol | Status |
|---|---|---|
| Terraform | HCL state files | 🔵 Planned |
| CloudFormation | JSON/YAML templates | 🔵 Planned |
| Kubernetes | `kubectl` API, YAML manifests | 🔵 Planned |
| CMDB | CSV/Excel export, ServiceNow API | 🔵 Planned |
| GitHub Actions | Workflow YAML, deploy events | 🔵 Planned |
| Azure DevOps | Pipelines API, release events | 🔵 Planned |
| AWS Config | AWS Config rules export | 🔵 Planned |
| Generic REST | Webhook push, polling schedule | 🔵 Planned |

---

## Relationship to Other Projects

| Project | Relationship |
|---|---|
| **VectorVolt** | EAIP will run on VectorVolt for its graph storage and query engine (Phase 3+) |
| **EKOS Engine** | EAIP's architecture intelligence layer is a subset of EKOS's broader capability |
| **EKOS Knowledge OS (EKR)** | EAIP becomes a specialized "EA module" within EKOS Knowledge OS |
| **ESIP** | ESIP already ships a subset of EAIP features (draw.io import, C4 diagrams, Traceability Graph) |

**Migration plan:** EAIP features are gradually absorbed into EKOS as ESIP→EKOS migration
completes (M1-M4). EAIP as a standalone product is the **enterprise-grade deep-architecture
variant** — ESIP has the features, EAIP has the infrastructure depth (auto-ingestion, drift detection, governance).

---

## Current State

| Artifact | Status |
|---|---|
| Architecture docs (`C:\eaip\eaip\docs\`) | ✅ Complete (discovered 2026-07-09) |
| FR-001 through FR-010 | ✅ Documented |
| Graph model (9 node types, 7 edge types) | ✅ Documented |
| Commercial model | ✅ Documented |
| Governance/drift design | ✅ Documented |
| Ingestion connector specs | ✅ Documented |
| Implementation (Rust + React) | ⬜ Not started |
| Integration with VectorVolt | 🔵 Phase 3 (after VectorVolt Phase 2 storage engine) |

---

## Next Steps (When Prioritized)

1. Formalize EAIP requirements in the same 27-volume format as EKR (or merge into EKOS volumes)
2. Decision: standalone EAIP product vs. absorbed into EKOS as "EA Intelligence Module"
3. Design graph storage schema on top of VectorVolt
4. Build ingestion connectors (Terraform + K8s first — highest adoption)
5. Build governance dashboard as ESIP widget (reuse Radix+Tailwind design system)

---

*Updated: 2026-07-09 | Source: `C:\eaip\eaip\docs\` scan completed 2026-07-09*
*Discovered this session — was previously unknown*
