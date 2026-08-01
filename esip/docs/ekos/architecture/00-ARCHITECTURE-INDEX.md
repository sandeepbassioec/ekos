# EKOS Platform Architecture — Master Index & Tracker

> **The platform comes before the application.** EKOS is an Operating System; ESIP (the
> web app built so far) becomes one of the applications that runs *on top of* it. This
> phase architects the OS **before** any further coding.
>
> **Rule of this phase (user directive, 2026-07-01):** *"We should not run into
> development too easily — this will be a mistake. We need to plan, we need to
> architect, we need technology choices, and we need this platform to sustain for the
> next 15 years by regularly maintaining applications on latest versions."*
>
> Resume by reading this file first → find the first ⬜/🟡 row → that's the next
> architecture document. One document per session, checkpointed here. Built on Opus.

---

## 0. Inputs

- **Requirements catalog:** `../00-MASTER-INDEX.md` (18 volumes, ~1,150 reqs) — the WHAT.
  This phase defines the HOW.
- **Build plan / OQ triage:** `../BUILD-PLAN-AND-OQ-TRIAGE.md` — phases, locked
  decisions (D1–D10), triaged Open Questions, pre-deploy blockers.
- **Existing tech-stack decision:** `../architecture-suite/TECH-STACK.md` (2026-06-23) —
  to be re-validated, not blindly inherited.

---

## 1. The Sequence (user's explicit order)

```
A. Platform/OS architecture (vision, quality attributes, logical & physical design)
        ↓
B. Technology choices (each as a Trade-off ADR — options, scoring, decision, why)
        ↓
C. Detailed implementation plan (epics, milestones, sequencing)
        ↓
D. ESIP-on-EKOS migration architecture (how the existing app becomes an EKOS app)
        ↓
E. 15-year sustainability (dependency currency, automated upgrades, deprecation,
   tech-radar, longevity guarantees)
```

---

## 2. Architecture Document Plan & Status

Status: ⬜ Not started · 🟡 In progress · ✅ Done

| # | Document | Maps to | Status |
|---|---|---|---|
| A00 | This index + Architecture Principles | — | ✅ |
| A01 | Architecture Vision, Drivers & Quality Attributes (the "-ilities", incl. 15-yr longevity as a top driver) | A | ✅ (64 items) |
| A02 | Logical Architecture (bounded contexts, layers, hexagonal ports map, C4 Context+Container) | A | ✅ (70 items; incl. complete volume→arch coverage map) |
| A03 | Technology Selection — ADR set (graph DB, event store, backend, frontend, rendering, AI, collab, infra — each a scored Trade-off ADR) | B | ✅ (66 items; ADR-019…031; 15-yr/no-regret lens) |
| A04 | Data, Storage & Ontology Architecture (polyglot CQRS, bitemporal, federation) | A | ✅ (62 items; ADR-032/033) |
| A05 | AI/ML Architecture (4-layer engine, sovereign/air-gapped, model-ops) | A | ✅ (60 items; ADR-034/035) |
| A06 | Security, Identity & Trust Architecture (ABAC PDP/PIP/PAP, zero-trust, crypto, key mgmt) | A | ✅ (72 items; ADR-036/037/038) |
| A07 | Integration & API Architecture (ports/adapters, connectors, public API, events) | A | ✅ (60 items; ADR-039/040) |
| A08 | Deployment & Infrastructure Architecture (K8s/Helm/OpenTofu, multi-cloud, air-gap, multi-region, tenancy) | A | ✅ (60 items; ADR-041/042/043) |
| A09 | **15-Year Sustainability & Evolution Architecture** (dependency-currency policy, automated upgrades, deprecation, tech-radar cadence, longevity guarantees) | E | ✅ (66 items; ADR-044/045/046; the 10 guarantees) |
| A10 | ESIP-on-EKOS Migration Architecture (the existing app as an EKOS application; strangler-fig path) | D | ✅ (58 items; ADR-047/048/049; 7-step M0–M7) |
| A11 | Detailed Implementation Plan & Roadmap (epics, milestones, team-less solo sequencing, definition-of-done) | C | ✅ (70 items; 16 epics, M0–M12, critical path) |
| A12 | **Knowledge Worker Runtime Architecture (KWR)** — GADS v1.0 first-class doc: 44 sections, C4 L1–L4, DDD context map, state machines, memory model, scheduler, execution pipeline, contract system, SDK, telemetry, billing, governance, security (STRIDE), all PlantUML diagrams, 5 patent families, implementation checklist | ChatGPT GAP-1 + GADS v1.0 | ✅ (44 sections) |
| A12-APP | **A12 Appendix — GADS v1.0 Gap Fill** — closes all 18 remaining GADS gaps: TOGAF (capabilities/value-stream/stakeholders/services), ArchiMate 7-layer (Motivation/Strategy/Business/Application/Technology/Implementation/Physical), BPMN (registration/assignment/escalation/AI-HITL), swimlanes, UML timing + communication + package diagrams, network diagram, DFD (L0+L1), Knowledge Flow diagram, service + external dependency graphs, threading/concurrency model, configuration reference (env vars/appsettings/Helm), scalability architecture, availability + reliability (circuit breakers + failure modes), DR plan (RPO/RTO/runbook), producers + consumers register, DevOps (repo structure/branch strategy/CI-CD/release/rollback) | GADS v1.0 compliance | ✅ (18 sections, 100% GADS) |

---

| VECTORVOLT-INT | **VectorVolt–EKOS Integration Plan** — formal phase-gate dependency map; 6 gates (VV Phase 1+2+3 mandatory before EKOS Phase 1); provider fulfillment schedule across all 4 `IStorageProvider` interfaces (ADR-052); VectorVolt graph extension gap analysis; syscall-to-provider map; gate status tracker | ADR-052 · VV IMPLEMENTATION_PLAN.md | ✅ |

## 🏁 ARCHITECTURE PHASE COMPLETE (2026-07-01) — Extended 2026-07-03 with GADS v1.0

A00–A11: EKOS strategic architecture (WHAT + HOW + WHY). Every choice 15-year / no-regret
defended; 52 ADRs logged. **A12** (2026-07-03): first GADS v1.0 implementation-ready document
— a senior engineer can build KWR from A12 alone without asking a single architectural question.

**Standard going forward (A12+):** GADS v1.0 — all 44+ sections, PlantUML diagrams (no ASCII),
C4 all four levels, DDD context maps, state machines, full security/observability/testing/
deployment/patent analysis, implementation checklist.

### Deployment Architecture Suite (2026-07-03)

| # | Document | Maps to | Status |
|---|---|---|---|
| A16 | **Full Distributed Deployment Architecture** — C4 Context + Container + Infrastructure diagrams; 6 network zones (Edge/DMZ/Application/Kernel/Data/Management); 4 Kubernetes node pools (kernel-bypass/GPU/stateless/stateful); 23-component inventory table with replica counts; VectorVolt placement + Gate 1/3/4 activation sequence; CCR in-process architecture (<1ms P99 via compiled Go module, no network hop); CTS pipeline sequence diagram (WAL → ProjectionDispatcher → Kafka → S3); ADR-052 Helm provider snippet; multi-AZ distribution table; scaling dimensions; GPU node pool (Ollama sovereign AI); K8s namespace layout + resource quotas | A08-001…043; ADR-041/042/052; VECTORVOLT-INT | ✅ |
| A17 | **Disaster Recovery Deployment Architecture** — 4-tier DR model (Free: Pilot Light RPO 24h/RTO 24h → Dedicated: Active-Active custom); DR topology PlantUML (primary + DR region, labeled replication paths); VectorVolt WAL as recovery spine (PostgreSQL+AGE/Redis/ClickHouse = disposable projections, rebuilt via ProjectionDispatcher replay); KMS-first failover constraint (Vault cross-region Raft — unconditional first step); CTS chain gap = constitutional hard stop + mandatory escalation; 12-step WAL recovery sequence diagram; 12-step failover runbook activity diagram; failback procedure; backup architecture (retention/encryption/test-restore cadence); game-day testing protocol; DR dashboard spec | A08-020/021/022; ADR-042; Book 1 Law 11; VECTORVOLT-INT | ✅ (82 items) |
| A18 | **On-Premises Deployment Architecture** — 4 deployment personas (gov/defense air-gap, regulated enterprise, cost-optimized, data-sovereignty); 9-row hardware BOM table (Control Plane → Redis) with exact CPU/RAM/NVMe/NIC specs; 3 sizing tiers (Small 11 nodes/50 users → Large 45 nodes/5000 users); 4-VLAN network topology diagram (Management/Data/Storage/InfiniBand RDMA); 20-component software stack with exact versions (RKE2 1.30, PostgreSQL 16.3, Kafka 3.7, Ollama 0.3); VectorVolt bare-metal config (exact GRUB cmdline: hugepages/isolcpus/nohz_full/iommu/C-state, DPDK driver binding, HugePage mount, InfiniBand OFED); sovereign AI model table (5 models with VRAM requirements); air-gap bundle spec (directory tree, cosign verification, Harbor image load, delta update format); installation sequence activity diagram; Dex LDAP/SAML config; FIPS mode + HSM PKCS#11 auto-unseal; complete `ekos-values-onprem.yaml` Helm values | A08-003/033/034; ADR-041; Book 1 Law 8; VECTORVOLT-INT | ✅ |
| A19 | **Client Cloud Deployment Architecture (BYOC)** — 8 BYOC principles (no data egress, client-controlled CMK, zero static credentials, bring-your-own managed services, license-only external call); 3-cloud coverage (AWS/Azure/GCP) each with: PlantUML infra diagram, actual instance SKUs (AWS `c6gn.16xlarge` EFA, Azure `Standard_HB120rs_v3` InfiniBand, GCP `c3-highcpu-176` gVNIC), IAM zero-credential flow (IRSA/Workload Identity/WIF); ADR-052 Helm values per cloud (`values-aws.yaml`/`values-azure.yaml`/`values-gcp.yaml`); VectorVolt cloud transport config (EfaTransport/InfiniBandTransport/gVNICTransport); private endpoints for all managed services; AI adapter config (Bedrock/Azure OpenAI/Vertex AI + sovereign fallback); license telemetry JSON payload (no client data); 3-tier cost reference table per cloud; OpenTofu + Helm deployment commands | A08-001/002/009/012; ADR-041/052; VECTORVOLT-INT | ✅ |
| A20 | **SaaS Platform Architecture** — 6-tier business model (Viewer/Free/Pro $49/Team $79/Enterprise $299/Dedicated custom); control plane (Tenant Registry, Provisioner <30s idempotent Helm+ArgoCD, Billing Engine, Stripe subscriptions+usage+Treasury, License Manager RSA-4096 JWT, Customer Portal, Support Tools, Health/SLO Monitor); data plane isolation (Pro/Team: namespace+shared pool, Enterprise: dedicated node pool+RDS, Dedicated: isolated cluster+customer KMS); VectorVolt SaaS config (ConsistentHashRouter, TenantManager, metering WAL events: KO_MUTATION/AI_CALL/SESSION_MINUTE/STORAGE_BYTE_HOUR); billing metering (Kafka → Stripe every 15min, dunning Day 0→67); Knowledge Economy (Stripe Treasury escrow, 80/20 revenue split, weekly payouts, 14-day dispute+CTS evidence); CDN+EKR Viewer (Cloudflare E2E encrypted, 1MB CI gate); CTS→compliance (12-event × 3-framework mapping: SOC2/GDPR/ISO27001, S3 WORM 7yr); multi-region active-active (us-east-1/eu-west-1/ap-southeast-1, 90s geo-failover); security ops (WAF/DDoS/DAST/SAST/bug bounty/JIT K8s/Sigstore); SLO table; phased rollout (Phase 1 solo→Phase 3 scale); cost model (10 tenants $935/mo → 1,000 tenants 85% margin) | A08-009/014/039; ADR-041/042/052 | ✅ (155 items) |
| A21 | **Customer Onboarding Architecture** — self-service wizard flow (Free/Pro/Team), assisted onboarding (Enterprise/Dedicated), control plane automation; domain + data residency selection; admin account + SCIM provisioning; SSO/SAML configuration; knowledge domain + ontology setup; compliance profile selection (GDPR/HIPAA/SOC2/FedRAMP); AI provider + DPA configuration; security baseline policy; integration connectors; governance structure setup; knowledge store import (Confluence/SharePoint/Jira/GitHub/ERP); branding + custom domain; Knowledge Economy opt-in; trial → paid conversion; support tier; demo seed wizard | A20; Vol 08/09; ESIP Demo Wizard | ✅ |

| A13 | **Knowledge Runtime Architecture** — 4-layer stack (Kernel/Syscall/Runtime/User-Space); full mutation lifecycle (Mutation→Dispatch→Projection); CCR 3-tier in-process enforcement (Tier 1 Structural / Tier 2 Behavioral <1ms P99 / Tier 3 OPA async); Scheduler two-tier dispatch (kernel-resident synchronous vs user-space async fair-share); KO anatomy with bitemporal stamps + reserved KWR fields; 14-syscall ABI catalogue; EKRA WASM sandbox isolation; VectorVolt gate map (CTS WAL / DocumentStore CCR cache / VectorScanner / MaterializedViewEngine / Graph Extension); determinism boundaries (LLM confined to AIInsight KO production); JIT PDP-scoped projections + cache key; bitemporal time-travel query sequence; observability hooks (OpenTelemetry spans + Observe syscall + Subscribe stream); 6 open questions | Book 2 · Vol 18 · Vol 22 · Vol 23 · ADR-051/053 · VECTORVOLT-INT | ✅ (41 items) |

| A15 | **Runtime Distribution Architecture** — 4-tier runtime taxonomy (Tier 0 Central / Tier 1 Regional / Tier 2 Edge / Tier 3 Embedded); Knowledge Domain Boundary (KDB) as federation unit; FederationGateway WAL-delta streaming with CTS chain proof; FEDERATED mutation type (CCR Tier 2 skipped, Tier 3 runs in receiver); conflict resolution policies (valid-time-wins / source-kernel-wins / merge-properties / manual-escalation); Edge Kernel architecture (same binary, SQLite WAL, no VV, no GPU, CCR Tier 1+2 only, ontology + ABAC snapshots); offline-first operation (PENDING_UPSTREAM KO tag, SYNC_REQUIRED mode, RejectionKO on upstream rejection); WASM Micro-Kernel (browser/mobile, no Journal, no Scheduler, mutations forwarded live); EKR Viewer (≤1MB, offline `.ekr` bundle with signed projection KOs); VectorVolt federation integration (FederationGateway reads VV WAL via IColumnarStoreProvider, edge mutations replayed through upstream VV write path); constitutional continuity across kernels (per-kernel CTS chains, FederationFrame proofs, edge CTS sub-stream in upstream WAL); EdgeProvisionBundle provisioning (one-time Ed25519 keypair, TPM binding, vvctl edge commands); Edge Gate model (E0–E3); 5 open questions | Book 2 · Vol 18 · Vol 22 · A13 · ADR-052/053 · VECTORVOLT-INT | ✅ (45 items) |

**Architecture documentation complete. Remaining work:**
- **A14** — Covered by Book 2 (EKOS Computational Model); A14 slot = ✅ by reference, no separate doc needed
- ESIP Security widget (pending from previous session)
- ESIP SAFe Step 7 metrics (pending from previous session)
- **VectorVolt Phase 1 (Rust)** ← NEXT MAJOR MILESTONE

---

## 📝 ESCS Formalization Note (2026-08-01)

New research in `C:\VectorVolt\docs\ekos-application-platform.txt` introduces the formal ESCS spec. Key architectural impacts:
- **EKR expanded**: now formally the "universal interaction shell" (not just file format + viewer) — this reinforces A15 (Runtime Distribution) and A13 (Knowledge Runtime) but extends EKR's scope.
- **Enterprise Meaning layer**: sub-atomic layer below Knowledge Kernel — refines A02 (Logical Architecture) layer model.
- **Persistence Engine formalization**: Persistence Contracts + Storage Planner formalize what A04 describes as polyglot CQRS.
- See `../ESCS-SUPPLEMENT.md` for full net-new concepts.

> A03 deliberately uses **EKOS's own Trade-off methodology** (the catalog's Solution-
> Design/Trade-off widget thinking) to pick technology — eating our own dog food.

---

## 3. Architecture Principles (locked; refined in A01)

1. **Platform before application** — the OS is architected first; apps (incl. ESIP) run on it.
2. **Ports-and-adapters / hexagonal everywhere** — every external dependency swappable.
3. **15-year longevity is a top-tier quality attribute**, not an afterthought — designed
   for dependency currency and continuous modernization.
4. **The semantic graph is the single source of truth**; everything else is a client/projection.
5. **Cryptographic hardness, not obscurity** (Constitution §2); **no training on customer content** (§6).
6. **Air-gap is first-class**; only internet-only features are excluded, each with a local equivalent.
7. **Evolve behind seams, not rewrite** — but the *platform foundation* is built deliberately, not bolted onto the web app.
8. **Capability-vs-feature** (the EKOS Golden Rule) governs every architectural addition.
9. **Standards over proprietary** where longevity depends on it (openCypher/GQL, JSON-LD/RDF, OCI, W3C VC/DID, CloudEvents).

---

## 4. Resume Protocol

1. Open this file → first ⬜/🟡 row in §2 = next document.
2. Pull relevant requirements from `../NN-*.md` + decisions from `../BUILD-PLAN-AND-OQ-TRIAGE.md`.
3. Write `architecture/ANN-<slug>.md` → flip status to ✅ here.
4. STOP. Ask before the next.

_Created 2026-07-01 (Opus). The platform-first architecture phase for a 15-year EKOS._
