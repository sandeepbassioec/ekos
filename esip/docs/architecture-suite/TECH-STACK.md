# ESIP Architecture Suite — Technology Stack (Decision Record)

**Date:** 2026-06-23 · **Status:** Decided · **Companion to:** `BUILD-PLAN-MVP-AND-PHASING.md`, `00-MASTER-INDEX.md`

**How decided (product owner):** Diagramming = **maxGraph** (explicit). Backend = **Hybrid: .NET Core 10 + a Rust code-scanning microservice** (explicit). Database = **client-configurable, support all major engines** (explicit requirement). Hosting = **cloud-neutral** (explicit requirement). Frontend + AI-as-pluggable were settled by the reuse + UI-consistency + portability principles.

> **Guiding rule:** the MVP keeps running on **today's stack** (Vite SPA + Firebase + `ai-proxy`). These decisions define the **target** enterprise stack and the **evolution seams** — we grow into it, we do not rewrite. The product posture is **"build once → connect to any database → deploy on any cloud or on-prem → talk to any AI provider."**

---

## Decided stack

| Layer | Choice | MVP (now) | Target (Phase 2+) | Why |
|-------|--------|-----------|-------------------|-----|
| **Frontend** | React 18 + TypeScript + **Vite** | ✅ in use | same | Reuse + UI consistency |
| **UI system** | Tailwind v4 + Radix (`components/ui/`) + lucide + sonner | ✅ | + shared app shell | Consistency mandate; already built |
| **Client state** | Zustand | ✅ | same | Reuse |
| **Server state** | **TanStack Query** | — | add when API lands | Caching/sync for real API |
| **Diagramming** | **maxGraph** (open-source draw.io engine) | introduce Phase 1/3 (frontend lib) | core modeling canvas | ArchiMate/UML/BPMN, custom shapes, Visio/drawio/XMI round-trip, model-vs-view |
| **AI** | **AWS Bedrock now, provider-pluggable** (Azure OpenAI / Vertex / watsonx / **local Ollama for air-gapped**) via generalized `ai-proxy` | ✅ proxy live (Bedrock) | provider abstraction + RAG/agents | Cloud-neutral requires the model provider to be swappable |
| **Vector/RAG** | **Pluggable** (pgvector when Postgres; standalone Qdrant/Weaviate otherwise) | — | Phase 2+ | Can't assume pgvector when the client DB varies |
| **Backend API** | **.NET Core 10 (C# / ASP.NET Core)** | Firebase only | .NET service tier behind the DAL | Best connector SDKs (esp. MS Graph), EF Core multi-DB, OpenXML doc-gen, Semantic Kernel, owner familiarity |
| **Code-scan service** | **Rust microservice** (axum + tree-sitter) | — | Phase 2 (Vol 09) | Rust + tree-sitter excels at fast multi-language parsing/dependency analysis |
| **Database (client-configurable)** | **Relational via EF Core: PostgreSQL · SQL Server · MySQL · Oracle**; **Neo4j** graph adapter (model/landscape); **Firebase** adapter (MVP/doc) | Firestore (current) | provider chosen **per client** behind a repository + provider-capability model | End client may run any DB; EF Core makes the 4 SQL engines a config switch |
| **File storage** | **S3-compatible abstraction** (cloud S3/Blob/GCS; **MinIO** on-prem); Cloudinary until migrated | Cloudinary (current) | abstracted object storage | Cloud-neutral |
| **Auth/Identity** | Firebase Auth → **pluggable IdP** (Cognito/WorkOS/Auth0/**Keycloak on-prem**) for SSO/SAML/OIDC/SCIM | Firebase (current) | IdP adapter (Phase 4) | Enterprise SSO + on-prem need a swappable IdP |
| **Workflow engine** | **XState** (DB-backed) → **Temporal/Camunda** if durable orchestration is needed | — | escalate when justified | Start light |
| **Hosting/Infra** | **Cloud-neutral: Docker + Kubernetes + Helm + Terraform**; runs on AWS/Azure/GCP/IBM/**on-prem & air-gapped** | Firebase/Vercel for MVP ok | K8s everywhere | Portability requirement; no proprietary lock-in in core |
| **Repo/tooling** | **Turborepo** polyglot monorepo (web TS · .NET api · Rust svc · shared contracts) | single app | monorepo when backend lands | Share API contracts across languages |

---

## Architecture implications of "configurable + neutral" (Ports & Adapters / Hexagonal)

The portability requirements force a **ports-and-adapters** core — the application logic depends on *interfaces*, and each external concern is a swappable *adapter*:

- **Persistence port → adapters:** EF Core relational (PG/SQL Server/MySQL/Oracle) · Neo4j graph · Firestore document. A **provider-capability model** lets a provider declare what it supports (the relational SoR and the graph model layer are distinct ports, not one pretending to be the other). This is the server-side big-brother of the existing `lib/db` `IDatabase` DAL.
- **AI port → adapters:** Bedrock · Azure OpenAI · Vertex · watsonx · local (Ollama/llama.cpp). The `ai-proxy` is already this seam.
- **Object-storage port → adapters:** S3 / Azure Blob / GCS / MinIO.
- **Identity port → adapters:** Firebase / Cognito / WorkOS / Auth0 / Keycloak.
- **Deploy:** one container set, Helm-charted, on any K8s; an **on-prem/air-gapped profile** (local LLM + MinIO + self-hosted DB + Keycloak) is a first-class target.

**Honest caveat on "any DB":** the four **relational** engines are a clean EF Core config switch. **Neo4j (graph)** and **Firebase (document)** are different paradigms — first-class but with non-identical capabilities; we will not pretend feature parity across paradigms. Relational-first; graph for the landscape/impact-analysis layer; document as the MVP adapter.

## Why maxGraph (diagramming, the big net-new)
Powers draw.io in production; handles strict notations (ArchiMate 3.2/UML/BPMN/C4/ERD) with custom shapes and **Visio/drawio/XMI/ArchiMate-Exchange round-trip**; supports **model-vs-view** separation. It's a frontend library, so the interactive canvas can begin in Phase 1 (persisting via the DAL) without waiting on the backend. Bridges from today's Mermaid + AI-generated diagrams (keep those as the on-ramp).

## Evolution seams (MVP → enterprise without a rewrite)
1. **`lib/db` DAL (`IDatabase`)** — Firestore today → .NET API + client-configured DB later; stores/UI unchanged.
2. **`ai-proxy`** — Bedrock today → multi-provider, RAG, agents; client contract stable.
3. **Auth-provider interface** — Firebase today → enterprise IdP adapter; route guards unchanged.
4. **Object-storage interface** — Cloudinary today → S3/MinIO.

*Frontend + UI stay constant. Backend (.NET + Rust), database (per-client), identity, storage, AI provider, and hosting all evolve behind these ports.*

## Polyglot split (who does what)
- **TypeScript (web):** the entire UI, maxGraph canvas, client state.
- **.NET Core 10 (api):** REST/GraphQL, integrations/connectors, persistence (EF Core + adapters), document generation (OpenXML), AI orchestration (Semantic Kernel), workflow, auth, RBAC/ABAC, multi-tenancy, audit.
- **Rust (code-scan svc):** repo ingestion, tree-sitter parsing, dependency graphs, SBOM/flow derivation → returns structured results to the .NET API (which persists + feeds the AI layer).

## Revisit later (not blocking)
- Dedicated vector store (Qdrant/Weaviate) only if pgvector/provider limits are hit.
- GraphQL vs REST surface for the public API (Vol 11).
- Whether the Rust service also owns other CPU-bound jobs (large-diagram layout, bulk export).
