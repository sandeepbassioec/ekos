# ESIP Selling Strategy

_Last updated: 25 June 2026_

---

## The Product Positioning

ESIP is not a "better diagram tool." It is the **first AI-native architecture documentation platform** — the only product where structured content (requirements, ADRs, metrics, process steps), any diagram type, and contextual AI generation coexist in the same document with the same data model and the same save cycle.

No current product owns this intersection. Lucidchart is great at diagrams but knows nothing about the document around them. Confluence knows documents but outsources diagrams to plugins that don't communicate with each other. Notion/Linear have no diagrams at all. Eraser.io has code-to-diagram but no structured widgets and no BPMN/ArchiMate.

---

## Target Customer Segments

### Tier 1 — Enterprise Architecture Teams (Large Enterprises)
- **Profile**: Fortune 500, banks, insurance, telco, government. 5–50 architects per org.
- **Current spend**: $50–150k/yr on Sparx EA, BiZZdesign, ADOIT, Lucidchart.
- **Pain**: Tools are draw-only (no structured content), no AI, desktop installs, diagrams disconnected from requirements.
- **WTP**: $500–2000/user/yr — they already have EA tool budgets.
- **Sales motion**: Enterprise sales, 6–18 month cycle. Needs SSO, audit logs, security review.

### Tier 2 — Solution/Application Architects at Product Companies _(primary beachhead)_
- **Profile**: SaaS companies, tech firms, consulting. Writing HLDs, LLDs, ADRs, C4 diagrams.
- **Current stack**: Confluence + Lucidchart plugins, or Miro + Google Docs. Context-switching hell.
- **Pain**: AI can't understand their architecture context. ADRs and diagrams live in different places.
- **WTP**: $100–500/user/yr. Team budgets $5–20k/yr.
- **Sales motion**: Bottom-up — individual architect adopts, team follows, IT approves.
- **Volume**: ~500,000 solution architects globally. **Start here.**

### Tier 3 — Business Analysts and Process Designers
- **Profile**: BAs at consulting firms, banks, insurers. Currently Visio + Confluence.
- **Pain**: BPMN tools are complex, no AI assistance, diagrams not in the same doc as requirements.
- **WTP**: $100–300/user/yr.
- **Sales motion**: Bottom-up or bundled into enterprise deal.

### Tier 4 — Consulting Firms (Accenture, Deloitte, Capgemini, boutique EA)
- **Profile**: Firms delivering architecture documentation as client deliverables.
- **Pain**: Every engagement rebuilds diagrams from scratch. Client deliverables are Word + Visio.
- **WTP**: $2000–5000/user/yr. ROI is immediate — tool saves 20% of documentation time.
- **Sales motion**: Firm-level partnership, not individual sale. One deal = 5,000 users.
- **Note**: White-label option is critical for this segment. Frame as "IP accelerator."

---

## Licensing Model

### Free / Open-Source Core (MIT or Apache 2.0)
Release the Tiptap widget framework, basic diagram types (Mermaid, React Flow), and template system as open-source.

**Rationale**: draw.io built $100M+ on this model. EA architects are deeply skeptical of vendor lock-in — open-source removes that objection, attracts contributors who build diagram types we haven't built yet, and establishes ESIP as a standard rather than just another SaaS vendor.

### Professional — $20–50/user/month
For teams. Adds:
- AI (Ask Sandeep) with Claude/Bedrock backend
- Real-time collaboration
- Version history
- PDF/print export at A3/A1
- SSO (SAML/OIDC)

### Enterprise — $100–200/user/month (min $2k/month)
For large orgs. Adds:
- Self-hosted deployment (critical for banks, government, defense)
- ArchiMate-compatible views
- Visio/draw.io import
- RBAC and audit logs
- On-premise AI option (Bedrock private endpoint or local model)
- SLA + dedicated support

### Consulting Firm Partnership — Revenue Share
- White-label option
- Per-engagement licensing (block of document seats per project)
- Revenue share on client seats provisioned
- Firm gets co-marketing rights ("Powered by ESIP")

---

## Competitive Moats

### 1. The Data Model
Once a company stores architecture documentation in ESIP's structured format (requirements linked to diagrams, ADRs, metrics, all in one doc), migrating away is painful. This is inherently sticky in a way that Lucidchart diagrams are not — because Lucidchart has no structured content to migrate.

### 2. AI Context Compounding
Ask Sandeep gets more valuable the more documents a company has in ESIP. A company with 200 architecture documents has an AI that deeply understands their tech stack, naming conventions, and architectural patterns. This isn't replicable by a competitor starting fresh.

### 3. Why Incumbents Can't Replicate
- **Notion**: Deliberately avoids technical/structured content. Wrong brand, wrong buyer.
- **Atlassian/Confluence**: Tried with Gliffy acquisition, failed to integrate deeply. Architecture too legacy.
- **Microsoft**: Has Visio + Loop + Copilot but has never integrated them — and Loop is still missing basic features.
- **Lucidchart**: Canvas-first architecture. Adding structured documents requires a rewrite.

---

## Diagram Coverage (Current vs. Roadmap)

### Currently Supported (~55–60 of ~180 EA diagram types)
- Full UML suite (14 types via Mermaid)
- Full C4 Model (6 views via Mermaid)
- ER Diagrams / Data Models (Crow's Foot via Mermaid erDiagram)
- Gantt / Roadmaps / Timelines (Mermaid)
- State Machines, Sequence Diagrams
- Customer Journey, Mind Maps, Pie/Quadrant charts
- Flowcharts + drag-and-drop canvas (React Flow widget)
- Git graphs, XY charts

### Roadmap Additions (prioritised by business value)

#### Phase 1 — Land Tier 2 (Solution Architects) [Next 3 months]
| Feature | Value | Effort |
|---|---|---|
| **BPMN widget** (`bpmn-js`) | Unlocks all process modeling, BAs, consulting | 6–8 weeks |
| **Real-time collab (Mermaid first)** | Miro-parity for diagram co-editing | 2 weeks |
| **draw.io XML import** | Architects bring existing diagrams in | 3–4 weeks |
| **Client-side A3/A1 PDF export** | Formal EA deliverable requirement | 1–2 weeks |
| **Matrix/grid widgets** (RACI, CRUD, SIPOC, BMC) | Covers ~20 diagram types with one widget | 3–4 weeks |

#### Phase 2 — Move Up to Enterprise EA [Months 4–9]
| Feature | Value | Effort |
|---|---|---|
| **ArchiMate-compatible views** (Archi file format) | TOGAF/ArchiMate enterprise deals | 3–4 months |
| **Cloud icon node packs** (AWS, Azure, GCP) | Cloud architecture diagrams | 3–4 weeks |
| **React Flow CRDT collaboration** | Full real-time collab on drag-and-drop | 4–6 weeks |
| **Self-hosted deployment** | Banks, government, defense deals | Infra work |
| **Server-side PDF (Puppeteer)** | Guaranteed print fidelity | 2–3 weeks |

#### Phase 3 — Scale via Consulting Partnerships [Year 2]
| Feature | Value | Effort |
|---|---|---|
| **White-label / theming** | Consulting firm partnerships | 2–3 weeks |
| **Visio (.vsdx) import** | Full enterprise migration story | 6–12 weeks |
| **ArchiMate certified** (The Open Group) | Government/defense mandated procurement | 9–12 months + cert fees |
| **tldraw freehand widget** | Wireframes, event storming, service blueprints | 3–4 weeks |
| **BPMN collaboration** | Real-time BPMN co-editing | 6–10 weeks |

---

## The ArchiMate Certification Decision

**Do not pursue The Open Group certification speculatively.**

Build Archi-compatible first (targets the same Archi Exchange Format XML). Most enterprise architects care about file compatibility with Archi and correct-looking notation — they don't check certification badges. The $5–10k fee and 6-month process is better spent on features unlocking the next 50 customers.

**Exception**: If a real enterprise prospect requires certified tooling as a procurement condition, initiate certification at that point — it becomes a funded engineering activity tied to a closed deal.

---

## Go-to-Market Sequence

```
Phase 1 (Months 1–6)
  └─ Open-source core → GitHub community
  └─ Professional tier launch → target Tier 2 (solution architects, product companies)
  └─ Bottom-up adoption → individual → team → IT approval

Phase 2 (Months 6–18)
  └─ Enterprise tier launch → self-hosted option unlocks banks, gov
  └─ First 5–10 enterprise reference customers
  └─ Case studies proving AI-assisted documentation speed (target: 50% time saving claim)

Phase 3 (Year 2+)
  └─ Consulting firm partnership talks (Accenture EA, Deloitte, Capgemini)
  └─ White-label option
  └─ One large firm deal = thousands of seats
```

---

## The Pitch (One Paragraph)

> "Enterprise architects spend 40% of their time in documentation — writing in one tool, drawing in another, asking AI in a third, none of them connected. ESIP is the first platform where requirements, ADRs, BPMN processes, C4 architecture diagrams, and AI generation all live in the same document. Your architect types 'draw me the OAuth sequence for this service' and a rendered diagram appears inline, saved with the document, visible in read-only preview. No context switching. No copy-paste. No diagrams orphaned from the decisions that created them."
