# ESIP — Codebase Context & Current-State Map

**Authoritative as of:** 2026-06-23
**Author:** Claude (full source + docs sweep)
**Status of this document:** This is the **current source of truth** for the architecture and state of the app. It **supersedes** `PROJECT_CONTEXT.md`, `HOW_TO_RESUME.md`, and the original `README.md`, which describe an older "estimation tool" phase and are now stale (see §13 Doc Corpus Guide).

> **What ESIP is:** A React single-page app that automates the daily workflow of an **Application Architect**. You work inside a **project workspace**; each project has multi-year **roadmaps** (three types: product / technical / architectural) viewable as quarterly, monthly, or Gantt timelines. Each **roadmap item** has a **Knowledge Hub** that ingests artifacts (uploaded documents, Teams meeting transcripts, team chats, Outlook emails) and runs **AWS Bedrock (Claude Sonnet 4.6)** over them to extract requirements, gaps, confusions/clarifications, suggestions, security requirements, STRIDE threats, NFRs, and cloud-governance items. The app grew out of an earlier "RepairLink development-estimation" tool, which still exists as a self-contained **Estimation Wizard** module.

---

## 1. Executive Summary

| Aspect | Reality today |
|---|---|
| **Origin** | Started as a hardcoded RepairLink PERT/Delphi estimate generator (Excel + PPT). Pivoted to a "Universal Estimation Wizard," then pivoted again into the current **Application Architect platform** with cloud backend. |
| **Frontend** | React 18.3 + TypeScript + Vite 6.3 + Tailwind v4 + Radix UI, React Router 7, Zustand state. Origin: Figma Make export. |
| **Primary datastore** | **Firebase Firestore** (project `esip-d6104`). `DB_PROVIDER` is hardcoded to `'firebase'`; the Supabase branch throws. |
| **File storage** | **Cloudinary** (cloud `dawxoue2h`, unsigned preset `esip_documents`). Firebase Storage is initialized but unused. |
| **Main AI** | **AWS Bedrock — `us.anthropic.claude-sonnet-4-6`**, called directly from the browser via `fetch` with a Bedrock API key (bearer token). |
| **Secondary AI** | Groq (`llama-3.3-70b-versatile`) for BRD import + estimation WBS generation; Gemini / Anthropic-direct as alternative BRD providers. |
| **Auth** | Firebase Auth (email/password). Roles persist in a Firestore `users/{uid}` profile doc (fixed 2026-06-23); the `allowedRoles` route gates enforce real roles. |
| **A 3rd backend exists** | A separate **Express + Prisma + JWT** server under `server/` is a parallel data silo used by only 3 pages — and its calls currently fail auth (see §11). |
| **Maturity** | The architect "golden path" (Projects → Roadmap → Knowledge Hub → AI analysis → AI insight tabs) works end-to-end *modulo* the document-text-extraction gap (§6). Many adjacent features are partial, stubbed, or orphaned (§9–§10). |

---

## 2. The Architect "Golden Path" (intended primary workflow)

```
/login
  → (auth) → redirect to /projects/list
      → /projects/create            (name, description, estimationTechnique)
      → /projects/:projectId         (ProjectDetailPage — dashboard + quick-nav tiles)
          → /projects/:projectId/roadmap
                (pick year + roadmap type; Quarterly / Monthly / Gantt views)
                → click a roadmap item's "Knowledge Hub" (BookOpen) icon
                    → /roadmap-items/:roadmapItemId/knowledge-hub
                          • add sources: Documents / Meetings / Emails / Chats
                          • click "Analyze All Sources" → AWS Bedrock
                          • review AI output tabs: Requirements / Gaps / Confusions /
                            Suggestions / Security / Threats / NFRs / Cloud Governance
                          • promote to Formal Requirements; generate Excel/PPTX deliverables
```

There is **no persistent sidebar/breadcrumb**; each page has its own inline back button. `HomePage` (`/`) is reachable but the default authenticated redirect goes straight to `/projects/list`. Note `HomePage` links to `/projects` (the **legacy** estimation dashboard) in one place and `/projects/list` (the **current** project manager) in another — a split-brain nav bug (§9).

---

## 3. Tech Stack

- **UI:** React 18.3.1, TypeScript, Tailwind CSS v4 (`@tailwindcss/vite`), Radix UI primitives (~40 components in `components/ui/`), `lucide-react`, `motion`, `recharts`, `sonner` (toasts), `mermaid` (diagrams).
- **Routing:** `react-router` 7.13 (`createBrowserRouter` in `src/app/routes.tsx`).
- **State:** `zustand` 5 (7 stores in `src/app/store/`), plus heavy direct `localStorage` use.
- **Forms/validation:** `react-hook-form`, `zod`.
- **Backends/SDKs:** `firebase` 12 (Auth + Firestore + Storage), `@supabase/supabase-js` (legacy, disabled), `cloudinary-core` + raw upload fetch.
- **AI SDKs:** `@google/generative-ai` (Gemini); AWS Bedrock and Groq and Anthropic are called via raw `fetch` (no AWS SDK).
- **Document parsing:** `pdfjs-dist` (PDF), `mammoth` (DOCX), `xlsx` (XLS/XLSX).
- **Deliverable generation:** `xlsx`, `pptxgenjs`, `jspdf` (+ `react-to-print`).
- **Build/run:** Vite. Scripts: `npm run dev`, `npm run build`. (Package manager: npm or pnpm; `pnpm.overrides` pins vite.)
- **Backend server (`server/`):** Express + Prisma + JWT (separate `package.json`).

---

## 4. Source Structure Map (`src/app/`)

```
main.tsx                 → initFirebase(); mounts <App/>
App.tsx                  → <ErrorBoundary><RouterProvider/><Toaster/>; imports migrate-to-supabase (console util)
routes.tsx               → ALL routes (createBrowserRouter)

pages/                   → route components
  Login, Register, Unauthorized, HomePage
  ProjectsListPage         (CURRENT projects manager → Firestore)
  ProjectsDashboard        (LEGACY estimation projects → localStorage)
  CreateProject, ProjectDetailPage
  RoadmapPage
  KnowledgeHubPage         (full-page Knowledge Hub: 3 rows of tabs)
  RequirementsCaptureBoard, CreateRequirement      (localStorage path)
  BusinessRequirementsWizard                        (industry/questionnaire-driven)
  ComprehensiveRequirementsPage                     (→ ComprehensiveRequirementsWizard)
  ArchitecturePage, ArchitectureGeneratePage
  AdminMetaModelPage, CreateArtifactTypePage        (admin metamodel)
  TeamManagement, ManagerApproval, DeveloperDashboard   (→ Express server; currently failing)
  EstimationWizard + steps/ (step1..9)              (LEGACY estimation module)

components/
  ProtectedRoute, ErrorBoundary, StepIndicator, MethodComparison
  BRDImportModal           (Groq/Gemini/Claude — separate AI pipeline)
  ComprehensiveRequirementsWizard
  ui/                      (~50 Radix wrappers)
  knowledge-hub/           (14 tab components — see §6)
  roadmap/                 (11 components — selectors, 3 views, item card/form/modal, RequirementsPanel)
  architecture/            (Wizard, Config, MermaidDiagram, BFA/DFA/HLD/LLD generators, steps/)
  wizard-steps/            (12 comprehensive-requirements step components)

store/                   → zustand: auth, projects, roadmap, knowledgeHub, requirements, complexity, estimation
lib/
  db/                      → DAL: interface.ts, firebase-impl.ts, firebase-config.ts, index.ts, types.ts
  supabase.ts              → legacy client (disabled in DAL; only RoadmapOwnerSelector still imports it)
  migrate-to-supabase.ts   → console-only migration util (has stale API calls)
  cloudinary.ts            → uploadToCloudinary (delete is a no-op stub)
  api.ts, mockApi.ts       → ApiClient for the Express server / mock
  ai/bedrock.ts            → MAIN AI (Bedrock Claude Sonnet 4.6)
  metamodel-store.ts, industry-templates.ts, questionnaires.ts
services/
  aiService.ts             → BRD import AI (Groq/Gemini/Claude + mock)
  documentParser.ts        → PDF/DOCX/XLSX → text (now wired into KH upload; also used by BRD import)
config/aiPrompts.ts        → 4 large extraction prompts (docs/meetings/emails/chats)
types/                     → project, roadmap, knowledge-hub, ai-requirements, requirements,
                              business-requirements, comprehensive-requirements, metamodel
utils/                     → estimation Excel/PPT/WBS generators + stepValidation (see §8)

server/                    → Express + Prisma + JWT (auth, projects, wbs, teams, architecture, requirements)
```

---

## 5. State Management — the 7 Zustand stores

| Store | Backs onto | Notes |
|---|---|---|
| `authStore` | Firebase Auth (via DAL) | Persisted to `localStorage['auth-storage']`; token reused by `ApiClient`. |
| `projectsStore` | **Firestore** `projects` + `localStorage['estimation-projects-storage']` | Dual: cloud fetch + local snapshot of last 50. |
| `roadmapStore` | **Firestore** `roadmaps` + `roadmapItems` | In-memory only (no local persist). |
| `knowledgeHubStore` | **Firestore** (13 collections) + **Cloudinary** (file binaries) | In-memory; orchestrates the Bedrock analysis. |
| `requirementsStore` | **Firestore** `pis` + `sprints` + `requirements` (via DAL) | Fixed 2026-06-23: now routes through `db.*` (was throwing on the removed Supabase flat API). Sole consumer: Comprehensive Wizard. |
| `complexityStore` | `localStorage['complexity-storage']` | Pure local computation. |
| `estimationStore` | `localStorage['estimation-wizard-storage']` | Pure local; has QuotaExceeded handling. |

> Additionally, `RequirementsCaptureBoard` and `CreateRequirement` bypass all stores and read/write `localStorage['requirements_<projectId>']` directly. This is **disconnected** from both `requirementsStore` (Zustand) and the Knowledge Hub's Firestore requirements.

---

## 6. Knowledge Hub + AI Pipeline (the core feature)

### Two surfaces, same engine
- **`KnowledgeHubPage`** (`/roadmap-items/:id/knowledge-hub`) — full page, **3 rows of tabs**.
- **`RequirementsPanel`** (`components/roadmap/RequirementsPanel.tsx`, ~2200 lines) — the **same Knowledge Hub rendered as an inline modal**, opened by clicking a **Gantt bar**.

### Tabs (`KnowledgeHubPage`)
- **Row 1 — Input Sources:** Documents · Meetings · Emails · Chats · AI Requirements
- **Row 2 — Requirements & Deliverables:** Formal Requirements · Generate Deliverables (Excel/PPTX, local)
- **Row 3 — AI Insights:** Gaps · Confusions · Suggestions · Security · Threats · NFRs · Cloud Governance
- **Always-on AI search bar:** natural-language questions (`searchKnowledgeHub`) + slash commands `/summary` `/openitems` `/risks` (Bedrock) and `/excel` `/pptx` `/export` `/analyze` (local/trigger).

### Artifact ingestion
| Source | How captured | Stored fields |
|---|---|---|
| Documents | File upload → Cloudinary binary + Firestore metadata | name, type, version, storagePath (Cloudinary publicId), fileUrl, `extractedText` *(see gap)* |
| Meetings | **Manual paste** | title, date, transcript, participants, keyDecisions, notes |
| Emails | **Manual paste** | subject, from, to, cc, date, body |
| Chats | **Manual paste** | platform (Teams/Slack/Other), title, participants, messages |

> There is **no live connector** to Teams or Outlook — all secondary sources are manual copy-paste. The UI labels describe *intent*.

### AI extraction flow (`bedrock.ts` → `analyzeRequirements`)
1. **Phase 1 (documents):** each document → `REQUIREMENTS_ANALYST_PROMPT` → Bedrock (parallel) → merged **baseline**.
2. **Phase 2 (meetings/emails/chats):** each source → its specialist prompt with the baseline JSON injected → Bedrock (parallel). Results are **additively merged** (refinements add to, don't replace, baseline).
3. Output is a single `AIAnalysisResult` with 8 arrays: `coreRequirements, gaps, confusions, suggestions, securityRequirements, threats, nfrs, cloudGovernance`.
4. `knowledgeHubStore.analyzeAllSources` saves each category to its own Firestore collection, then refetches.

### Prompt design (`config/aiPrompts.ts`)
Four large prompts, all returning the **same 8-key JSON**:
- **Document analyst:** IEEE-830 IDs + MoSCoW, INVEST/SMART quality flags, gap analysis, conflict/duplicate detection, rewrite + risk register.
- **Teams meeting analyzer:** classify meeting; cross-reference each statement vs baseline (VALIDATE/REJECT/REFINE/CONTRADICT/NEW/ASSUMPTION); mine decisions, "said-not-written" reqs, sentiment, action items.
- **Outlook analyzer:** map participants→roles; CONFIRM/CONTRADICT/CLARIFY/SCOPE_CHANGE/NEW; decisions, risk signals, action items.
- **Teams chat analyzer:** signal-vs-noise; informal decisions (emoji/“yeah do it”); informal additions; conflicts; accountability gaps.

### AI model & auth (verified in `bedrock.ts`)
- Model: **`us.anthropic.claude-sonnet-4-6`** (line 28).
- Endpoint: `https://bedrock-runtime.<region>.amazonaws.com/model/<model>/invoke`, body `anthropic_version: bedrock-2023-05-31`, `max_tokens: 4096`, `temperature: 0.3`; response read as `data.content[0].text`.
- Auth: `Authorization: Bearer <key>` + `x-api-key` (line 52-53). The key prefix `ABSK…` is an **AWS Bedrock API key** (valid bearer-token auth).
- **Risks:** (a) the key is **hardcoded as a fallback in source** (line 26-27) — a committed secret; (b) the call is made **directly from the browser**, which both **exposes the key in the bundle** and will hit **CORS** (Bedrock does not return browser CORS headers) — so in a real browser this likely needs a backend proxy.

### Known functional gaps in the AI path
1. **Document text is never extracted on upload.** `knowledgeHubStore.uploadDocument` → `db.documents.create` does **not** call `documentParser`, so `extractedText` is empty and Phase 1 analyzes only the **filename** (`doc.extractedText || 'Document: <name>'`). `documentParser` is only used by `BRDImportModal`. **This is the single highest-impact defect** — the headline feature analyzes near-empty document content.
2. **`coreRequirements` are discarded.** `analyzeAllSources` saves the other 7 categories but never writes `coreRequirements` to `db.aiRequirements`, so the **AI Requirements tab is always empty** after an analysis run. (The tab shows a legacy `aiRequirements` collection / manual entries.)
3. Minor mapping bugs in `bedrock.ts`: email `{{EMAIL_FROM}}` is filled with the subject (line 163); meeting content uses `notes` while the type's primary field is `transcript` (verify the store maps `transcript→notes`).

---

## 7. Other feature subsystems (status)

### Requirements capture — **four disconnected entry points**
| Entry point | Produces | Storage |
|---|---|---|
| `RequirementsCaptureBoard` (Kanban by PI/Sprint, hardcoded PI-1..3 / Sprint-1..6) | `Requirement` (10-type union) | `localStorage['requirements_<projectId>']` |
| `CreateRequirement` (Screen/Bug/API subtypes) | same `Requirement` | same localStorage |
| `BusinessRequirementsWizard` (5 steps, industry/questionnaire-driven) | `AutoSuggestion[]` | `localStorage['business_requirements_context']` → `/requirements/generate` |
| `ComprehensiveRequirementsWizard` (see below) | `ComprehensiveRequirements` | `requirementsStore.bulkCreateRequirements` (→ **broken** path) |

These use **three non-interoperable requirement types** and don't share storage. The Knowledge Hub's "Formal Requirements" are a **fifth**, Firestore-backed model.

### Comprehensive Requirements Wizard
- **11 registered steps** (CloudGovernanceStep is fully built but **never registered** — orphaned).
- Only **NFR** (step 1) and **Security** (step 2) are fully wired. Integration/Data are partial. **Steps 5–11 are uncontrolled — their data is silently discarded.** `completionPercentage` counts *visited* steps, not steps with data.

### Architecture generation (`/architecture`, `/architecture/generate`)
- **Template-only, zero AI.** The "Analyzing Requirements…" spinner is a cosmetic `setTimeout(2000)`; analysis is regex/keyword extraction.
- 4 generators produce Mermaid diagrams + downloadable docs/DDL: **BFA** (journey/process/use-case), **DFA** (ER, DFD L0/L1, sequence, class + SQL DDL), **HLD** (4 arch diagrams + ~550-line Markdown), **LLD** (class, sequence + REST spec + PostgreSQL DDL).
- `ArchitectureWizard` steps 3–5 are "Coming soon." `ArchitectureConfig` (feeds HLD) and `ArchitectureWizard` are **two disconnected entry points** — the wizard's output isn't read by the generators. Outputs hardcode AWS/JWT/TLS/SLA assumptions. DOCX/PNG/PDF exports are toast stubs.

### Metamodel / Admin (`/admin/metamodel`)
- A pluggable **artifact-type registry** (ArtifactType + FieldDefinition + ComplexityRule + WBS/Story/Architecture/Capacity templates), all `localStorage`-backed; seeds 7 default artifact types.
- `CreateArtifactTypePage` steps 1–2 work; steps 3–4 (Complexity Rules, Templates) are "Coming soon." `WBSTemplate/StoryTemplate/ArchitectureTemplate/CapacityTemplate` types exist but **no UI consumes them** — the metamodel is currently an **admin island**.

### Estimation Wizard (legacy, `/wizard/step/1..9`)
- Self-contained 9-step flow: method select → setup → scope (Module→Screen→Component) → method details → complexity weighting (19 factors) → risk (20 risks) → review → capacity planning → generate.
- Methods: PERT, Story Points, T-Shirt, FPA, UCP (formulas in `estimationStore`). Method comparison + variance.
- **Only real AI here:** `MethodSelectionWrapper` → `utils/wbsGenerator.ts` calls **Groq** (`llama-3.3-70b-versatile`) to auto-generate WBS screens — with a **hardcoded `gsk_…` Groq key in source** (security issue).
- Generators (`utils/`): `dynamicExcelGenerator`/`dynamicPptxGenerator` are wired to the wizard; the hardcoded RepairLink generators (`excelGenerator`, `pptxGenerator`, `completeEstimatesGenerator`, `completePptxGenerator`) are **dead** (not wired to any button). PDF export button is disabled.

---

## 8. Data Model (Firestore-backed, `lib/db/types.ts`)

```
Project (userId, name, status, estimationTechnique, teamId, …)
 └─ Roadmap (projectId, type: product|technology|architectural, year, ownerId)
     └─ RoadmapItem (roadmapId, title, quarter?, month?, startDate?, endDate?,
                      status, priority, assignedTo?, dependencies[])
         └─ Knowledge Hub (all keyed by roadmapItemId):
              Document · Meeting · Email · Chat · AIRequirement(legacy)
              Gap · Confusion · Suggestion · SecurityRequirement
              Threat(STRIDE) · NFR · CloudGovernance · FormalRequirement
```
> The `Project` type (`types/project.ts`) defines 20+ fields (clientName, industry, modules, teamMembers, hasWBS/hasHLD/…), but `CreateProject` only collects **name, description, estimationTechnique** — the rest are never populated. `RoadmapItem` has **no `progress` field**, yet `ProjectDetailPage` renders `item.progress` (always undefined). The roadmap form has **no `dependencies` UI** (always saved `[]`).
> The Express/Prisma server has an **entirely separate** schema (User, Team, TeamMember, Project, WbsTask, ProjectCollaborator, ArchitectureArtifact, Invitation, AuditLog, PI, Sprint, Requirement) that does not share data with Firestore.

---

## 9. AI Integrations (all three)

| Use case | Provider / model | Where | Key handling | Status |
|---|---|---|---|---|
| Knowledge Hub extraction, search, summaries | **AWS Bedrock — Claude Sonnet 4.6** (`us.anthropic.claude-sonnet-4-6`) | `lib/ai/bedrock.ts` | `VITE_BEDROCK_API_KEY` **+ hardcoded fallback in source**; browser-direct fetch | Wired & central; CORS/secret risks (§6) |
| BRD import (→ estimation screens) | **Groq** `llama-3.3-70b-versatile` (default), **Gemini** `gemini-1.5-flash`, **Anthropic** `claude-3-5-sonnet-20241022`, or **mock** | `services/aiService.ts` via `BRDImportModal` | User pastes key in UI | Works; results **not** persisted to Firebase (isolated) |
| Estimation WBS auto-gen | **Groq** `llama-3.3-70b-versatile` | `utils/wbsGenerator.ts` | **Hardcoded `gsk_…` in source** | Works; secret leak |

---

## 10. Critical issues & tech debt (prioritized)

**P0 — breaks the headline feature or core flows** — ✅ all three FIXED IN CODE 2026-06-23 (pending runtime verification; see §15 Changelog)
1. ~~**KH documents are not text-extracted on upload** → Bedrock analyzes filenames only.~~ **FIXED:** `uploadDocument` now calls `parseDocument` and persists `extractedText`. (§6)
2. ~~**`requirementsStore` is broken at runtime** — imports `db` from removed `lib/supabase` and calls non-existent flat methods.~~ **FIXED:** Supabase import removed; routed through the swappable DAL (added `pis`/`sprints`/`requirements` to `IDatabase` + Firebase impl). (§5)
3. ~~**`coreRequirements` from AI analysis are never saved** → AI Requirements tab always empty post-analysis.~~ **FIXED:** `analyzeAllSources` now maps `coreRequirements` → DAL `AIRequirement` and saves them. (§6)

**P1 — security**
4. **Hardcoded credentials committed to source:** ~~Bedrock API key (`bedrock.ts:27`), Groq key (`wbsGenerator.ts`)~~ **FIXED** — both removed from client and moved to the `ai-proxy/` server env (⚠️ **still must be rotated** — they're in git history). Remaining (lower risk): Supabase anon JWT (`lib/supabase.ts`, disabled client) + Firebase web config (`firebase-config.ts`, not secret but guard with Firestore rules). (§15)
5. ~~**Browser-direct Bedrock call** → CORS failure risk + key exposure.~~ **FIXED** — Bedrock (and Groq) now go through the `ai-proxy/` service; the browser never calls AWS directly. (§15)
6. **Express server JWT fallback** (`server/src/lib/jwt.ts`) defaults to `'default-secret'` if env unset.
7. **Firestore security rules are dev-grade** (`allow read,write: if request.auth != null`) — not production-safe.

**P1 — correctness**
8. ~~**Role system is inert:** `firebase-impl.ts` `toAuthUser()` hardcodes `role: 'manager'`…~~ **FIXED** — roles persist in a Firestore `users/{uid}` profile doc (written at signup, read on sign-in/session; legacy users self-heal). The `allowedRoles` gates now enforce real roles. (§15)
9. **Express-backed pages fail auth:** `ManagerApproval`, `TeamManagement`, `DeveloperDashboard` send the Firebase JWT to the Express server, which verifies with its own HS256 secret → 401. The server is effectively orphaned.
10. **Comprehensive Wizard drops data** for steps 5–11 (uncontrolled inputs); `CloudGovernanceStep` orphaned. (§7)

**P2 — cleanup / dead code**
11. Cloudinary **delete is a no-op** → orphaned binaries accumulate.
12. **Firestore `orderBy` clauses commented out** "to avoid composite index requirement" → unsorted results; or create the 8 documented indexes.
13. Dead/duplicate code: legacy RepairLink generators (4 files), `ProjectsDashboard` vs `ProjectsListPage` split, `RoadmapOwnerSelector` (uses browser-illegal `supabase.auth.admin`), unused Firebase Storage paths, `migrate-to-supabase.ts` stale calls, duplicate `ErrorBoundary`.
14. `BusinessRequirementsWizard` type mismatch (`processTemplates` string[] vs ProcessTemplate[]).

---

## 11. The Express server (`server/`) — what it is and why it's stranded
- Express + Prisma (DB via `DATABASE_URL`, intended Supabase Postgres) + custom HS256 JWT. Routes: `auth`, `projects`, `wbs`, `teams`, `architecture`, `requirements`.
- Frontend calls it via `lib/api.ts` (`ApiClient`, base `http://localhost:3001/api`) from exactly 3 pages.
- **Mismatch:** `ApiClient` sends the **Firebase** ID token; the server's `authenticate` middleware verifies with **its own** `JWT_SECRET` → every call 401s. Its data also never reaches Firestore. It is a **parallel silo**. Decision needed: repurpose it as the Bedrock proxy + real auth backend, or retire it.

---

## 12. How to run

**Frontend**
```bash
cd C:\VectorVolt\esip
npm install        # (or pnpm install)
npm run dev        # Vite dev server → http://localhost:5173
```
Needs a `.env` with `VITE_FIREBASE_*` (project `esip-d6104`), `VITE_CLOUDINARY_*`, `VITE_AWS_REGION`, `VITE_BEDROCK_API_KEY`. (Today several of these have hardcoded fallbacks in source — see §10.)

**Manual Firebase console setup** (per the FIREBASE-*.md docs): enable Email/Password auth; set Firestore + Storage rules; create composite indexes (or keep `orderBy` disabled).

**AI proxy (required for any AI feature — Knowledge Hub analysis, search, WBS gen)**
```bash
cd C:\VectorVolt\esip\ai-proxy
npm install
cp .env.example .env   # add ROTATED BEDROCK_API_KEY + GROQ_API_KEY
npm start              # → http://localhost:8787  (GET /health to check)
```
Frontend talks to it via `VITE_AI_PROXY_URL` (defaults to `http://localhost:8787`, so local dev needs no extra frontend config). See `ai-proxy/README.md`.

**Express server (optional/legacy)**
```bash
cd C:\VectorVolt\esip\server
npm install
npm run db:push    # push Prisma schema
npm run dev        # → http://localhost:3001
```
Needs `server/.env` with `DATABASE_URL`, `JWT_SECRET`, `JWT_REFRESH_SECRET`.

---

## 13. Doc Corpus Guide (what to trust)

**Current / useful**
- **THIS FILE** — authoritative current state.
- `KNOWLEDGE-HUB-*.md`, `AI-REQUIREMENTS-*.md` — feature intent for the Knowledge Hub & AI (note: they reference an older OpenAI-via-Supabase-Edge path that has since been replaced by Bedrock; treat provider details as historical, architecture intent as useful).
- `FIREBASE-*.md`, `CLOUDINARY-*.md` — accurate setup steps for the live backends.
- `server/README.md`, `server/QUICKSTART.md` — accurate for the Express server (which is itself stranded).

**Stale / historical (do not use as current architecture)**
- `README.md` (Figma Make stub), `PROJECT_CONTEXT.md`, `HOW_TO_RESUME.md` — describe the estimation-tool phase only.
- `ANALYSIS_SUMMARY.md`, `GAP_ANALYSIS_MATRIX.md`, `IMPLEMENTATION_PLAN.md`, `REVISED_PRIORITY_PLAN.md`, the `QUICK_WIN_*`/`*_COMPLETE.md` set — estimation-era sprint logs; completion %s are internally contradictory.
- `SUPABASE-*.md` — Supabase is disabled in the DAL; three different "Supabase schemas" appear across docs.
- `RepairLink_*.md`, `src/imports/pasted_text/*.md` — legacy RepairLink domain content.
- `guidelines/Guidelines.md` — empty Figma placeholder.

---

## 14. Candidate next steps (to discuss — not yet decided)

These fall into clear tracks; pick based on the goal (demo polish vs. production-hardening vs. feature breadth):

- **A. Make the headline feature truly work** — wire `documentParser` into KH upload + persist `extractedText`; persist `coreRequirements`; fix `requirementsStore`. (Unblocks the core value prop.)
- **B. Security + deployability** — stand up a minimal Bedrock proxy (reuse `server/`), move all keys to env, rotate the committed Bedrock/Groq keys, fix role storage, tighten Firestore rules.
- **C. Consolidate the data story** — choose one persistence path; bridge or retire the Express server, the legacy localStorage requirement paths, and the disabled Supabase client.
- **D. Finish partial features** — register/clean the Comprehensive Wizard steps; connect Architecture wizard→generators; complete metamodel steps 3–4.
- **E. Real connectors** — replace manual paste with Teams/Outlook ingestion.

---

## 15. Changelog

### 2026-06-23 — Track A: fix the AI core (code complete, runtime-unverified)
Three P0 fixes landed in two files:
- `store/knowledgeHubStore.ts`
  - `uploadDocument`: now runs `parseDocument(file)` and stores `extractedText` on the document (non-fatal on parse failure). **Effect:** Bedrock now analyzes real document content instead of just the filename.
  - `analyzeAllSources`: added a block that maps each AI `coreRequirement` → the DAL `AIRequirement` shape and saves it; updated the success toast to include the requirements count. **Effect:** the AI Requirements tab populates after analysis.
- `store/requirementsStore.ts` — **routed through the swappable DAL** (not localStorage), per the design directive that Firebase is primary today but any DB must be hookable tomorrow.
  - Removed the dead `import { db } from '../lib/supabase'`. All PI/Sprint/Requirement actions now call `db.pis.*` / `db.sprints.*` / `db.requirements.*` via `lib/db`. `bulkCreateRequirements` normalizes a single object **or** an array (the Comprehensive Wizard passes a single object). **Effect:** no more `db.getPIs is not a function`; the wizard persists through the same seam as every other store.
- DAL extended to make those entities provider-agnostic:
  - `lib/db/types.ts` — added `PI`, `Sprint`, `Requirement` types.
  - `lib/db/interface.ts` — added `pis`, `sprints`, `requirements` namespaces to `IDatabase` (the contract a new DB implements).
  - `lib/db/firebase-impl.ts` — implemented all three against Firestore (`pis`/`sprints`/`requirements` collections, projectId-scoped, via the existing generic helpers).
  - `lib/db/index.ts` — exposed the three namespaces on the `db` proxy + exported the new types.
- `lib/db/firebase-config.ts` — Firestore now initialized with `initializeFirestore(app, { ignoreUndefinedProperties: true })` (try/catch fallback to `getFirestore` for HMR). **Effect:** writes no longer throw on `undefined` fields — fixes a latent fragility in the existing `analyzeAllSources` saves (e.g. `sourceId: gap.source?.id`) and lets the Comprehensive Wizard's partial object persist.

**Verification status:** ✅ booted. `npm install` + Vite dev server ran clean (ready in ~1s, no build errors). The app — using a persisted Firebase session — reloaded into a live **Knowledge Hub**, ran `fetchKnowledgeHub` through the rewritten DAL against Firestore, and rendered all tabs with **no errors** (only a pre-existing benign `forwardRef` warning on the search `Input`). No `is not a function`, no Firebase errors. Not yet exercised: an actual upload→Bedrock analyze run (needs the proxy running with rotated keys + Cloudinary). Bedrock/CORS + key exposure are addressed by **Track B** below; the inert role system remains open.

### 2026-06-23 — Track B: AI proxy + key hardening (code complete; ⚠️ keys must be rotated)
- New **`ai-proxy/`** service (minimal Express; own `package.json`, `.env.example`, README) — holds the Bedrock + Groq keys server-side and exposes `POST /api/ai/bedrock` (`{prompt}`→`{text}`), `POST /api/ai/groq` (`{messages,…}`→`{content}`), and `GET /health`. CORS restricted to the Vite origin. **Verified:** boots and `/health` returns `{ok:true, providers:{bedrock,groq}}`.
- `lib/ai/bedrock.ts` — `callBedrock` now POSTs to the proxy (`VITE_AI_PROXY_URL`, default `http://localhost:8787`). **Removed** the hardcoded Bedrock API key and the direct `bedrock-runtime.amazonaws.com` browser call → fixes CORS + client key exposure.
- `utils/wbsGenerator.ts` — Groq WBS generation now goes through the proxy. **Removed** the hardcoded `gsk_…` key.
- Confirmed via grep: **no `ABSK…`/`gsk_…` keys remain anywhere under `src/`.**

**Action required (you):** revoke/rotate the old Bedrock + Groq keys (they're in git history), put the new ones in `ai-proxy/.env`, then `cd ai-proxy && npm install && npm start`. After that the full upload→analyze path can run end-to-end.

**Still open (P1, not in this change):** dev-grade Firestore rules; the Express server↔Firebase JWT mismatch; Firebase web config + disabled-Supabase anon key still in client source.

### 2026-06-23 — Role system (now functional)
- `lib/db/firebase-impl.ts` — `toAuthUser()` no longer hardcodes `role: 'manager'`. Roles live in a Firestore **`users/{uid}`** profile doc:
  - `signUp` writes `{ email, fullName, role }` — the role chosen on the Register form (it already flowed `authStore.register` → `auth.signUp`, but was being discarded).
  - `signIn` reads the profile and **self-heals** legacy users (creates a doc with the default role if missing).
  - `getSession` / `getUser` read the stored role; `DEFAULT_ROLE = 'manager'` is the fallback only when no doc exists yet.
- **Effect:** the `allowedRoles` gates in `routes.tsx` (`/teams` + `/manager/approvals` → manager/admin; `/developer/tasks` → developer; `/admin/*` → admin) now enforce the user's real role.
- **Grant yourself admin:** set `role: "admin"` on your `users/{uid}` doc in the Firestore console (uid is in `localStorage['auth-storage']`), or register a new admin account.
- Note: the new `users` collection is covered by the existing dev-grade Firestore rule; for production scope it to `request.auth.uid == uid`.
- Verified: app boots clean after the change (HMR applied, no new console errors). Exercise full RBAC (e.g. register as `developer` → blocked from `/admin/*`) with a test account.

---

*End of context document. Update this file whenever architecture or major status changes.*
