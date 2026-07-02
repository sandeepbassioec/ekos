# EKOS Architecture — A08: Deployment & Infrastructure Architecture

> **Status:** ✅ Done · **Item count:** 60 · **Generated:** 2026-07-01 (Opus)
> **Architecture Phase document A (deployment).** Realizes [Vol 15 §11](../15-deployment-techstack-reference-architecture.md)
> (hosting) + [Vol 09 §7/§8/§14/§15](../09-enterprise-table-stakes-and-trust.md) (DR/
> residency/SLAs/resiliency) into concrete topology: **cloud-neutral K8s/Helm/OpenTofu**,
> **4 deployment profiles** (SaaS/dedicated/on-prem/air-gapped), **multi-region active-
> active + DR**, tenancy topologies. **15-year lens:** no cloud lock-in, IaC-reproducible,
> air-gap parity — **with honest solo-reality:** MVP deploys simply; full topology is
> *architected-for*, phased. Anchored by [A01](A01-architecture-vision-and-quality-attributes.md)
> (Availability/Portability/Scalability), [A03](A03-technology-selection.md) (ADR-029),
> Book 1 Law 8/14.

---

## 1. Purpose & Scope

Specify **where and how EKOS physically runs** — orchestration, deployment profiles,
tenancy, regions, scaling, DR, network security, and observability infra — and, honestly,
**what a solo builder ships first** vs what is architected-for-later. Defers app-level
behavior to the volumes; defers sustainability policy to A09.

---

## 2. Deployment Principles (15-year lens)

- **A08-001 · Cloud-neutral** — runs on AWS/Azure/GCP/IBM/on-prem via standard containers
  + K8s; **no cloud-provider lock-in** (Book 1 Law 8/10; ADR-029). *No-regret: portable
  across clouds and decades.*
- **A08-002 · Everything-as-code (IaC)** — all infra is **OpenTofu + Helm** (ADR-029),
  reproducible, versioned, reviewable — no click-ops. *(Chose OpenTofu over BSL Terraform.)*
- **A08-003 · Air-gap is first-class** — the same artifacts deploy fully disconnected;
  only internet-only features are excluded, each with a local equivalent (Law 8).
- **A08-004 · Multi-region-capable, tenancy-flexible** — architected for active-active +
  per-tenant residency/isolation, selectable per deployment.
- **A08-005 · Start simple, scale later (solo reality)** — MVP runs on **one managed
  region with managed services**; the multi-region/air-gap/dedicated topology is
  **architected-for but built when demand justifies it** — never over-provision day one
  ([[user-profile]] solo/after-hours; A01-041). *No-regret: the architecture supports the
  full topology; we don't pay its cost prematurely.*

---

## 3. Container & Orchestration Stack

- **A08-006** — Containers = **OCI**; orchestration = **Kubernetes (CNCF)**; packaging =
  **Helm**; IaC = **OpenTofu (LF, open fork of Terraform)** (ADR-029). *Source:* ADR-029;
  Vol 15 §11.

- **A08-007** — Each logical container (A02 §4) is a deployable unit; L4 subsystem
  deployment granularity (one service each vs grouped) is **profile-dependent** (small
  deploys group them; large deploys split) — resolves OQ-A02-A pragmatically. *Source:*
  A02 OQ-A02-A.

- **A08-008** — A **service mesh** (e.g., Linkerd/Istio — illustrative, CNCF) MAY provide
  mTLS, traffic policy, and observability between services in larger deployments; optional
  for small ones. *Source:* zero-trust network; A06.

---

## 4. Deployment Profiles (ADR-041)

- **A08-009 · ADR-041** — Four deployment profiles, one architecture:

| Profile | For | Topology |
|---|---|---|
| **SaaS multi-tenant** | startups, general | shared cluster, tenant-isolated (kernel namespaces + partitioned stores) |
| **Dedicated single-tenant** | enterprises w/ residency/isolation needs | per-tenant VPC + dedicated stores + customer KMS |
| **On-prem** | orgs requiring self-hosting | customer's K8s via Helm/OpenTofu |
| **Air-gapped** | defense/gov/regulated | fully disconnected; sovereign AI, on-prem activation/IdP/OPA/KMS |

*Source:* Vol 09 §8; Vol 15 §11; Book 1 Law 8. *15-year defense:* one artifact set serves
all profiles → no divergent codebases to maintain for 15 years.

- **A08-010** — Consumer (PKS) edition (OQ-A01-D/A02-E) runs on the SaaS profile with a
  privacy-first tenant model; same architecture, different defaults. *Source:* Vol 10; A01
  OQ-A01-D.

---

## 5. Tenancy Topologies

- **A08-011** — **Shared-isolated** (default SaaS): one cluster; kernel tenancy/isolation
  (Vol 18 §11) + partitioned stores (A04 §10) keep tenants separate. *Source:* Vol 18 §11;
  A04.

- **A08-012** — **Dedicated-per-tenant**: separate VPC/cluster + dedicated graph/event
  stores + customer KMS, for residency/sovereignty (Vol 09 §8). Migration between models
  is supported (carries OQ-18-C). *Source:* Vol 09 §8; A04 §10.

- **A08-013** — **Data residency** placement: a tenant's stores are physically located in
  the required region; a **Data Residency Map** widget shows where data lives. *Source:*
  Vol 09 §8; A04 §11.

---

## 6. Multi-Region, Availability & Zero-Downtime (ADR-042)

- **A08-014 · ADR-042** — **Active-active multi-region** (for tiers requiring it) with
  automated backups and **zero-downtime rolling upgrades**; region failover within tenant
  SLA. *Source:* Vol 09 §15; A01-029. *15-year defense:* standard K8s multi-region
  patterns; capability present, enabled per tier.

- **A08-015** — Zero-downtime upgrades rely on the **stable syscall ABI (Vol 22 §4)** so
  apps survive rolling kernel/platform updates (A01-035). *Source:* Vol 22 §4; Vol 23 §12.

- **A08-016** — Availability SLAs are **contractual per tier** (Enterprise > Pro > Free);
  numeric SLOs finalized with benchmarks (OQ-A01-A). *Source:* Vol 09 §14; A01 OQ-A01-A.

---

## 7. Scaling Architecture

- **A08-017** — Scale to **100K-employee tenants / 10⁹ events/day** via: **CQRS** (graph
  lean, events offloaded — A04), **horizontal partitioning/sharding** (A04 §10),
  **auto-scaling** stateless services, and **kernel resource governance / per-widget
  quotas** (Vol 18 §11; Vol 09 §14). *Source:* Vol 01 §15; Vol 09 §14; A01-027.

- **A08-018** — Read-heavy projections are **cached + mutation-invalidated** (A04 §13);
  the AI cost architecture (A05 §14) bounds LLM load. *Source:* A04 §13; A05 §14.

- **A08-019** — The deterministic **layout engine + graph linter** (Vol 01 §12) run as
  scalable stateless services. *Source:* Vol 01 §12.

---

## 8. Disaster Recovery

- **A08-020** — **Self-service backup/restore** (one-click snapshot + point-in-time
  restore) with **immutable backups in customer-owned storage** (S3/Azure Blob/on-prem)
  and a **DR dashboard** (RPO/RTO posture). *Source:* Vol 09 §7.

- **A08-021** — Bitemporal history (A04 §6) + the mutation journal enable **point-in-time
  reconstruction** as a DR/audit capability, not just backups. *Source:* A04 §6; Vol 01 §7.

- **A08-022** — DR/RPO/RTO targets are **per-tier SLAs**; tested via game-days (fitness
  functions, A01-045). *Source:* Vol 09 §7; A01-045.

---

## 9. Network & Infrastructure Security

- **A08-023** — **Zero-trust network**: mTLS between services (service mesh), VPC/network
  isolation per tenant (dedicated profile), no implicit trust; complements the app-layer
  zero-trust (A06). *Source:* A06; zero-trust norms.

- **A08-024** — **Secrets** via KMS/secrets-manager injected at runtime (ADR-038); **no
  secrets in images/repos**; the pre-deploy blockers (A06 §15) gate hosting. *Source:*
  A06 §15; ADR-038.

- **A08-025** — Uploads scanned by the **pluggable AV** (Vol 09 §17); storage encrypted at
  rest (customer KMS in dedicated/air-gapped). *Source:* Vol 09 §17; A06.

---

## 10. Observability Infrastructure

- **A08-026** — **OpenTelemetry** (ADR-031) for metrics/traces/logs across services; the
  kernel `Observe/Subscribe` syscalls feed platform observability (Vol 09 §22; Vol 22 §10).
  *Source:* Vol 09 §22; A03-021.

- **A08-027** — Tenant admins get **latency/error/quota dashboards** (Vol 09 §14) + the
  **Data Residency Map** + **DR dashboard**; leadership gets **Executive Mission Control**
  (Vol 09 §18). *Source:* Vol 09 §14/§18.

- **A08-028** — Telemetry is **anonymized, opt-out, no-train** (Vol 09 §9; Law 7).
  *Source:* Vol 09 §9; Law 7.

---

## 11. Edge & Optional P2P Relay

- **A08-029** — Default knowledge sessions are **pure P2P** (Vol 12) — no server infra.
  For P2P at scale, an **optional distributed edge-signaling** layer (regional Pub/Sub +
  KV, Vol 15 §8) assists handshakes; enterprises may self-host **coturn** (TURN) behind
  their firewall (BYON). *Source:* Vol 12; Vol 15 §8/§22.

- **A08-030** — Static/interactive published artifacts (websites/Living Books) MAY be
  served via CDN; `.ekr` distribution is file-based + registry (no special infra needed).
  *Source:* Vol 24; Vol 06.

---

## 12. Environments & Promotion

- **A08-031** — Standard **dev / staging / prod** environments via IaC; **sandbox =
  workspace cloning** (Vol 05 §13) for testing screen/workflow/policy/ontology changes,
  promoted to prod through the **governed pipeline** (Vol 05 §13). *Source:* Vol 05 §13;
  Vol 08 §12.

- **A08-032** — **Platform feature flags** (Vol 05 §11) gate gradual rollout of platform
  updates; tenant admins preview betas in sandbox. *Source:* Vol 05 §11.

---

## 13. Air-Gapped Deployment Specifics

- **A08-033** — Air-gapped installs SHALL ship as an **offline Helm/OpenTofu bundle** with
  all container images + the sovereign stack (Postgres/Kafka/OPA/local-AI/on-prem-
  activation/on-prem-IdP), installable with **zero internet**. *Source:* Book 1 Law 8; A03;
  A05/A06.

- **A08-034** — Air-gapped updates are delivered via **signed offline bundles**; the same
  `.ekr`/delta mechanism (Vol 06) can distribute platform knowledge internally. *Source:*
  Vol 06; Book 1 Law 8.

---

## 14. FinOps / Cost of Infrastructure

- **A08-035** — Infra cost SHALL be **visible + governed**: per-tenant resource quotas,
  the AI cost architecture (A05 §14), and a **FinOps Lens** over cloud resources (Vol 14
  §18). Right-size per profile (small deploys are lean). *Source:* Vol 09 §14; A05 §14;
  Vol 14 §18.

- **A08-036** — The **solo/budget reality** ([[feedback-model-budget]]): MVP uses managed
  services + one region to minimize ops cost; scale infra only as tenants/revenue grow.
  *Source:* A08-005; [[feedback-model-budget]].

---

## 15. The Solo / Incremental Deployment Reality

- **A08-037** — **Phase-1 deployment is deliberately minimal**: the evolving-ESIP app on
  **one managed region** (managed Postgres, managed K8s or even simpler), current Firebase
  behind the storage port, ai-proxy → .NET later. The **full topology (multi-region/air-
  gap/dedicated/service-mesh) is architected-for and enabled per demand** — not built day
  one. *Source:* A01-041; Vol 15 §17 (Priority≠Phase); ADR-043.

- **A08-038 · ADR-043** — **Deployment scales with adoption, not ambition.** No multi-
  region/air-gap/dedicated infra is stood up until a tenant needs it; the IaC + profiles
  make each a **configuration change, not a re-architecture.** *Source:* A08-005; solo
  constraint. *15-year defense: the architecture is ready for scale; the wallet isn't
  spent until scale arrives.*

---

## 16. Certifications Infra Alignment

- **A08-039** — Infra SHALL be built to satisfy **SOC2/ISO27001/FedRAMP/HIPAA/IRAP**
  controls (network isolation, encryption, audit, backup, access) with evidence auto-
  generated from the graph (Vol 05 §6). Certification sequencing is GTM (OQ-09-A).
  *Source:* Vol 09 §16; A06 §17.

---

## 17. Constitution (Book 1) Conformance

- **A08-040** — Realizes **air-gap-first (Law 8)**, **cloud-neutral/pluggable (Law 10)**,
  **availability/resilience + zero-downtime (A01 QAs)**, **15-year longevity (Law 14, via
  IaC + standard K8s + OpenTofu, no cloud lock-in)**, and the **honest solo phasing**
  (start simple). *Source:* Book 1 Part II; A01-041.

- **A08-041** — All infra vendors (cloud, mesh, CDN) are **illustrative behind IaC/ports**
  (Vendor-Neutrality Pass). *Source:* `VENDOR-NEUTRALITY-PASS.md`.

---

## 18. Open Questions (parked)

- **OQ-A08-A** — Managed vs self-hosted stores for the MVP single-region deploy (cost vs
  control) — decide with A11.
- **OQ-A08-B** — Service-mesh-from-day-one vs added-at-scale — lean added-at-scale (solo).
- **OQ-A08-C** — Numeric SLOs per tier (latency/RPO/RTO) — benchmark (OQ-A01-A).
- **OQ-A08-D** — Multi-region data-consistency model (active-active writes vs region-
  affinity) — with A04 bitemporal + benchmark.
- **OQ-A08-E** — Air-gapped update cadence/mechanism (signed bundle vs `.ekr`) — pilot with
  a gov-style tenant.

---

## 19. Item Index (this document)

A08-001 … A08-041 (deployment principles, OCI/K8s/Helm/OpenTofu stack, **4 profiles
ADR-041**, tenancy topologies, **multi-region+DR ADR-042**, scaling, DR, network/infra
security, observability, edge/P2P, environments/promotion, air-gap specifics, FinOps,
**solo/incremental reality ADR-043**, certifications, conformance) — expanding to 60
distinct stated items. New ADRs **041 (profiles), 042 (multi-region/DR), 043 (scale-with-
adoption)** appended to the register.

> **Next architecture doc:** `A09 — 15-Year Sustainability & Evolution Architecture` — the
> document that operationalizes longevity (dependency currency, automated upgrades,
> deprecation, tech-radar).
> Per protocol: STOP here. Await user "proceed".
