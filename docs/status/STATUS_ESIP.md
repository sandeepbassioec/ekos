# ESIP — Live Status Tracker
**Tool:** ESIP (Enterprise Software Integration Platform → evolving into EKOS Knowledge OS)  
**Repository:** `C:\VectorVolt\esip\` (React + TypeScript + Firebase + ai-proxy)  
**Authoritative codebase map:** `C:\VectorVolt\esip\ESIP_CODEBASE_CONTEXT.md`  
**Migration plan:** `esip/docs/ekos/architecture/A10` (ESIP→EKOS strangler-fig, M0–M7)  
**Last Updated:** 2026-07-09

> **Rule:** This file is updated after every session.

---

## What ESIP Is

ESIP started as a React app automating Application Architect work (projects → roadmaps →
Knowledge Hub → Bedrock AI requirements). It is now the front-end/application layer of EKOS
Knowledge OS. The existing codebase is the Phase 1 "EKOS MVP on the current stack."

ESIP does NOT get rewritten. It evolves via strangler-fig migration (M0–M7) until it IS EKOS.

---

## Current Shipped State (Blended MVP)

| Feature | Status |
|---|---|
| Project creation + management | ✅ Shipped |
| Roadmap generation (Bedrock AI) | ✅ Shipped |
| Knowledge Hub | ✅ Shipped |
| Requirements generation (Bedrock AI) | ✅ Shipped |
| SAFe PI Planning flow | ✅ Shipped |
| Sprint Board (flag-gated OFF) | ✅ Built, flag off |
| /flow command | ✅ Shipped |
| AI button fix | ✅ Shipped |
| Trade-off Analysis widget (Radix+Tailwind) | ✅ Shipped |
| NFR (Non-Functional Requirements) widget | ✅ Shipped |
| Traceability Graph widget (maxGraph) | ✅ Shipped |
| BPMN widget (bpmn-js) | ✅ Shipped |
| Matrix/grid widget (RACI, CRUD, SIPOC, BMC presets) | ✅ Shipped |
| draw.io XML import | ✅ Shipped |
| A3/A1 PDF export (client-side SVG serialization) | ✅ Shipped |
| Real-time collaboration on Mermaid diagrams (Yjs) | ✅ Shipped |
| C4 / flow diagrams (maxGraph) | ✅ Shipped |
| Mermaid diagram widget | ✅ Shipped |

---

## Pending (Before EKOS Phase 1 Migration)

| Item | Priority | Status |
|---|---|---|
| Security widget | High | ⬜ |
| SAFe Step 7 metrics | High | ⬜ |
| Rotate ai-proxy API keys (Bedrock/Groq) — **PRE-DEPLOY BLOCKER** | Critical | ❌ USER action |

---

## ESIP→EKOS Migration Steps (M0–M7)

| Step | Name | Status |
|---|---|---|
| M0 | Rotate ai-proxy keys (pre-deploy blocker) | ❌ USER action |
| M1 | DAL storage port (abstract Firebase behind repository interface) | ⬜ |
| M2 | Graph foundation (minimal ontology: Element/Artifact/Requirement/ArchComponent) | ⬜ |
| M3 | Widget re-expression (existing widgets become graph projections) | ⬜ |
| M4 | H3 convergence (maxGraph-based diagramming aligned to Knowledge Graph) | ⬜ |
| M5 | Ambient AI crux (symbolic rules → small model) | ⬜ |
| M6 | ABAC + visual masking | ⬜ |
| M7 | .ekr packaging + Viewer MVP | ⬜ |

---

## Tech Stack

| Layer | Technology |
|---|---|
| Frontend | React 18 + TypeScript + Vite |
| UI Components | Radix UI + Tailwind CSS |
| Diagramming | maxGraph (shipped), bpmn-js (shipped) |
| State | Zustand |
| Real-time collab | Yjs |
| Backend (current) | Firebase (Firestore + Auth) |
| Backend (target) | .NET Core 10 + Rust microservice + polyglot CQRS |
| AI | Bedrock (Claude) + Groq — provider-pluggable |
| AI proxy | `esip/ai-proxy/` (Node.js) |

---

## Design System Rule

> **All new features MUST match the existing Radix+Tailwind design system.**
> Cross-role workflows (Application Architect + Business Analyst + Technical Lead) are first-class.
> Requirements catalog is living — accept field-level changes inline during build.

---

## Selling Strategy

Full strategy doc: `esip/docs/SELLING_STRATEGY.md`

| Segment | Positioning |
|---|---|
| Enterprise IT / Application Architecture teams | AI-native EA doc platform — replace Confluence + Jira ADO for architecture |
| Digital Transformation consultancies | White-label EKOS for client delivery |
| SaaS product teams | Embedded knowledge graph for product intelligence |
| PhD / Research / Think tanks | Knowledge OS for academic knowledge management |

**Licensing model:** OSS (core viewer/graph) + Pro (team features) + Enterprise (full EKOS)

---

*Updated: 2026-07-09 | Current focus: Security widget + SAFe Step 7 metrics → then M0 migration*
