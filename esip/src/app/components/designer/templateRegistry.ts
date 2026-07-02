import type { JSONContent } from '@tiptap/react';
import type { DesignerTemplate } from './types';

// ── Text helpers ──────────────────────────────────────────────────────────────
function h(level: 1 | 2 | 3, text: string): JSONContent {
  return { type: 'heading', attrs: { level }, content: [{ type: 'text', text }] };
}
function p(text: string): JSONContent {
  return { type: 'paragraph', content: text ? [{ type: 'text', text }] : undefined };
}
function meta(pairs: [string, string][]): JSONContent {
  const content: JSONContent[] = [];
  pairs.forEach(([label, value], i) => {
    content.push({ type: 'text', marks: [{ type: 'bold' }], text: `${label}: ` });
    content.push({ type: 'text', text: value });
    if (i < pairs.length - 1) content.push({ type: 'hardBreak' });
  });
  return { type: 'paragraph', content };
}
function bullets(items: string[]): JSONContent {
  return { type: 'bulletList', content: items.map(t => ({ type: 'listItem', content: [p(t)] })) };
}

// ── Widget helpers ────────────────────────────────────────────────────────────
function callout(type: 'info' | 'warning' | 'success' | 'danger', title: string, body: string): JSONContent {
  return { type: 'callout', attrs: { type, title, body } };
}
function req(id: string, reqType: 'SHALL' | 'SHOULD' | 'MAY', priority: 'HIGH' | 'MEDIUM' | 'LOW', status: 'OPEN' | 'IN-PROGRESS' | 'VERIFIED' | 'CLOSED', description: string, tags: string): JSONContent {
  return { type: 'requirement', attrs: { reqId: id, reqType, priority, status, description, tags } };
}
function adr(id: string, title: string, status: 'PROPOSED' | 'ACCEPTED' | 'SUPERSEDED' | 'DEPRECATED', context: string, decision: string, consequences: string): JSONContent {
  return { type: 'adr', attrs: { adrId: id, title, status, context, decision, consequences } };
}
function kpi(title: string, items: { id: string; value: string; label: string; delta: string; trend: 'up' | 'down' | 'neutral' }[]): JSONContent {
  return { type: 'metrics', attrs: { title, metrics: JSON.stringify(items) } };
}
function steps(title: string, items: { id: string; name: string; status: 'done' | 'active' | 'todo' }[]): JSONContent {
  return { type: 'processSteps', attrs: { title, steps: JSON.stringify(items) } };
}
function diagram(title: string, source: string): JSONContent {
  return { type: 'diagram', attrs: { title, source } };
}

const today = new Date().toLocaleDateString('en-GB', { day: 'numeric', month: 'long', year: 'numeric' });

// ── System prompts ────────────────────────────────────────────────────────────
const BRD_SYSTEM = `You are a senior business analyst writing a Business Requirements Document.
Write clear, professional prose. Use IEEE-830 conventions (SHALL/SHOULD/MAY) for requirements.
Avoid technical jargon in stakeholder-facing sections; be precise in requirements sections.`;

const AGILE_SYSTEM = `You are a product manager writing an Agile BRD.
Use user-story format (As a … I want … So that …) and INVEST criteria for stories.
Write acceptance criteria as Given/When/Then or bullet checklists.`;

const ENTERPRISE_SYSTEM = `You are a senior enterprise business analyst writing a formal BRD.
Include executive-level summaries, business case justification, ROI analysis, and governance.
Use formal language and structured numbering throughout.`;

const LEAN_SYSTEM = `You are writing a Lean BRD — optimised for speed and clarity.
Keep every section concise (bullet-focused). Ruthlessly cut filler.
Each section should be digestible in under 2 minutes.`;

const HLD_SYSTEM = `You are a solution architect writing a High-Level Design document.
Use precise technical language for a senior engineering audience.
Follow C4 model concepts (Context → Containers → Components). Avoid decorative filler.`;

const MSA_SYSTEM = `You are a microservices architect writing a service design document.
Cover service responsibilities, APIs, data ownership, async messaging, and deployment.
Use domain-driven design terminology where appropriate.`;

const CLOUD_SYSTEM = `You are a cloud architect writing a Cloud Native Design document.
Specify cloud provider services (prefer agnostic where possible), IaC approach, scaling strategies.
Include cost estimation guidance and security posture per section.`;

const LLD_SYSTEM = `You are a senior software engineer writing a Low-Level Design document.
Include data types, field names, API signatures, error codes, and edge cases.
Use tables for data models, numbered steps for sequences.`;

const DB_SYSTEM = `You are a database architect writing a Database Design document.
Cover entity relationships, normalisation decisions, indexing strategy, and migration approach.
Be precise: include field types, constraints, foreign keys, and sample queries.`;

const API_SYSTEM = `You are an API architect writing an API Contract document.
Describe every endpoint: path, method, auth, request schema, response schema, error codes.
Follow REST conventions or note deviations. Keep it implementable without further clarification.`;

// ── BRD: Classic ──────────────────────────────────────────────────────────────
const BRD_CLASSIC: DesignerTemplate = {
  id: 'brd-classic',
  name: 'Classic BRD',
  description: 'Standard business requirements document: executive summary, scope, functional & non-functional requirements, risks, and success criteria.',
  category: 'brd',
  badge: 'BRD',
  systemPrompt: BRD_SYSTEM,
  toolboxActions: ['write-section', 'improve', 'add-requirements', 'add-risks', 'add-stakeholders', 'generate-diagram', 'summarize'],
  initialContent: {
    type: 'doc',
    content: [
      h(1, 'Business Requirements Document'),
      meta([['Project', '(enter project name)'], ['Version', '1.0'], ['Status', 'Draft'], ['Date', today]]),

      callout('info', 'How to use this template', 'Click on any widget to edit it inline. Use "/" on an empty line to insert new widgets. Use "Ask Sandeep" to generate content with AI.'),

      h(2, '1. Executive Summary'),
      p('Provide a 2–3 paragraph overview of the project purpose, key drivers, and expected business outcomes.'),

      kpi('Business Impact', [
        { id: '1', value: 'TBD', label: 'Revenue Impact',   delta: '',    trend: 'neutral' },
        { id: '2', value: 'TBD', label: 'Cost Reduction',   delta: '',    trend: 'neutral' },
        { id: '3', value: 'TBD', label: 'Time to Market',   delta: '',    trend: 'neutral' },
        { id: '4', value: 'TBD', label: 'User Adoption',    delta: '',    trend: 'neutral' },
      ]),

      h(2, '2. Problem Statement'),
      callout('warning', 'Problem Statement', 'Define the specific problem or opportunity. Quantify the cost of inaction (revenue loss, manual effort, customer churn, compliance risk).'),

      h(2, '3. Scope'),
      callout('success', 'In Scope', 'List the features, systems, and processes included in this initiative.'),
      callout('danger', 'Out of Scope', 'List explicitly excluded items to prevent scope creep. Be specific.'),

      h(2, '4. Business Requirements'),
      req('REQ-001', 'SHALL', 'HIGH',   'OPEN', 'The system SHALL [describe core functional requirement].', 'Functional, Core'),
      req('REQ-002', 'SHALL', 'HIGH',   'OPEN', 'The system SHALL [describe second functional requirement].', 'Functional, Core'),
      req('REQ-003', 'SHALL', 'MEDIUM', 'OPEN', 'The system SHALL maintain response time ≤ 2 seconds under normal load (< 1000 concurrent users).', 'Non-Functional, Performance'),
      req('REQ-004', 'SHOULD', 'MEDIUM', 'OPEN', 'The system SHOULD support [optional enhancement].', 'Functional, Enhancement'),

      h(2, '5. Stakeholders'),
      callout('info', 'Stakeholder Matrix', 'Role | Name | Interest | Influence\nProject Sponsor | TBD | High | High\nProduct Owner | TBD | High | High\nEnd Users | TBD | High | Low\nIT / Engineering | TBD | Medium | Medium'),

      h(2, '6. Risks'),
      callout('danger', 'Risk 1 — [Title]', 'Likelihood: High · Impact: High · Mitigation: Describe the mitigation strategy and owner.'),
      callout('warning', 'Risk 2 — [Title]', 'Likelihood: Medium · Impact: Medium · Mitigation: Describe the mitigation strategy and owner.'),

      h(2, '7. Timeline'),
      steps('Project Phases', [
        { id: '1', name: 'Discovery',     status: 'todo' },
        { id: '2', name: 'Requirements',  status: 'todo' },
        { id: '3', name: 'Design',        status: 'todo' },
        { id: '4', name: 'Development',   status: 'todo' },
        { id: '5', name: 'UAT',           status: 'todo' },
        { id: '6', name: 'Go Live',       status: 'todo' },
      ]),

      h(2, '8. Success Criteria'),
      kpi('Target KPIs', [
        { id: '1', value: '≥ 90%',  label: 'User Satisfaction',   delta: 'baseline: TBD', trend: 'neutral' },
        { id: '2', value: '< 2s',   label: 'Page Load Time',      delta: 'target P99',    trend: 'neutral' },
        { id: '3', value: '99.9%',  label: 'Availability SLA',    delta: 'per month',     trend: 'neutral' },
        { id: '4', value: '0',      label: 'P1 Bugs at Launch',   delta: 'hard limit',    trend: 'neutral' },
      ]),
    ],
  },
};

// ── BRD: IEEE 830 SRS ─────────────────────────────────────────────────────────
const BRD_IEEE: DesignerTemplate = {
  id: 'brd-ieee-830',
  name: 'IEEE 830 SRS',
  description: 'Formal Software Requirements Specification following IEEE-830 structure: functional, non-functional, and interface requirements with traceability.',
  category: 'brd',
  badge: 'SRS',
  systemPrompt: BRD_SYSTEM,
  toolboxActions: ['write-section', 'improve', 'add-requirements', 'add-risks', 'generate-diagram', 'summarize'],
  initialContent: {
    type: 'doc',
    content: [
      h(1, 'Software Requirements Specification'),
      meta([['System', '(enter system name)'], ['Version', '1.0'], ['Status', 'Draft'], ['Date', today]]),

      callout('info', 'IEEE 830 Standard', 'This SRS follows IEEE Std 830-1998. Each requirement is uniquely numbered and tagged with SHALL (mandatory), SHOULD (recommended), or MAY (optional).'),

      h(2, '1. Introduction'),
      h(3, '1.1 Purpose'),
      p('Describe the purpose of this SRS and its intended audience (developers, QA, project management, customer).'),
      h(3, '1.2 Scope'),
      p('Name the software product. State what it will do and, equally importantly, what it will NOT do.'),
      h(3, '1.3 Definitions & Acronyms'),
      bullets(['API — Application Programming Interface', 'SRS — Software Requirements Specification', 'SHALL — Mandatory requirement', 'SHOULD — Recommended requirement']),

      h(2, '2. Overall Description'),
      h(3, '2.1 Product Perspective'),
      callout('info', 'System Context', 'Describe how this software fits into a larger system. Is it standalone, a component, or a replacement for an existing system?'),
      h(3, '2.2 User Classes & Characteristics'),
      callout('info', 'User Classes', 'Admin User: Full system access, technical background.\nEnd User: Core feature access, non-technical.\nIntegration Client: API access only, developer persona.'),

      h(2, '3. Functional Requirements'),
      req('FR-001', 'SHALL', 'HIGH',   'OPEN', 'The system SHALL [core functional requirement — e.g. authenticate users via SSO using SAML 2.0].', 'Functional, Authentication'),
      req('FR-002', 'SHALL', 'HIGH',   'OPEN', 'The system SHALL [second functional requirement — e.g. allow users to create, read, update, and delete records].', 'Functional, CRUD'),
      req('FR-003', 'SHALL', 'HIGH',   'OPEN', 'The system SHALL [third functional requirement — e.g. send email notifications on key state transitions].', 'Functional, Notifications'),
      req('FR-004', 'SHOULD', 'MEDIUM','OPEN', 'The system SHOULD [optional enhancement — e.g. provide audit log for all data mutations].', 'Functional, Audit'),

      h(2, '4. Non-Functional Requirements'),
      h(3, '4.1 Performance'),
      req('NFR-P-001', 'SHALL', 'HIGH', 'OPEN', 'The system SHALL serve API responses with P99 latency ≤ 200ms under a load of 500 concurrent requests.', 'Performance'),
      req('NFR-P-002', 'SHALL', 'HIGH', 'OPEN', 'The system SHALL support horizontal scaling to handle a 10× traffic spike within 60 seconds.', 'Performance, Scalability'),

      h(3, '4.2 Security'),
      req('NFR-S-001', 'SHALL', 'HIGH', 'OPEN', 'The system SHALL encrypt all data in transit using TLS 1.2 or higher.', 'Security'),
      req('NFR-S-002', 'SHALL', 'HIGH', 'OPEN', 'The system SHALL enforce multi-factor authentication for all admin accounts.', 'Security, Authentication'),

      h(3, '4.3 Availability & Reliability'),
      req('NFR-A-001', 'SHALL', 'HIGH', 'OPEN', 'The system SHALL achieve ≥ 99.9% monthly uptime (measured as successful health-check responses).', 'Availability'),

      kpi('Quality Targets', [
        { id: '1', value: '99.9%', label: 'Availability',      delta: 'per month',    trend: 'neutral' },
        { id: '2', value: '200ms', label: 'P99 Latency',        delta: 'API endpoints',trend: 'neutral' },
        { id: '3', value: '≥ 80%', label: 'Test Coverage',      delta: 'new code',     trend: 'neutral' },
        { id: '4', value: '0',     label: 'Known Vulns',         delta: 'at release',   trend: 'neutral' },
      ]),

      h(2, '5. Interface Requirements'),
      callout('info', 'External Interfaces', 'User Interface: Web application (React, WCAG 2.1 AA).\nAPI: RESTful JSON over HTTPS.\nDatabase: PostgreSQL 16.\nExternal: [list third-party integrations].'),

      h(2, '6. Traceability Matrix'),
      callout('warning', 'Traceability Note', 'Map each requirement to its source: FR-001 ← User Story US-042 ← Business Goal BG-01. Update this section before review sign-off.'),
    ],
  },
};

// ── BRD: Agile ────────────────────────────────────────────────────────────────
const BRD_AGILE: DesignerTemplate = {
  id: 'brd-agile',
  name: 'Agile BRD',
  description: 'User story–driven BRD: product vision, epics, acceptance criteria, Definition of Done — ready for sprint planning.',
  category: 'brd',
  badge: 'AGILE',
  systemPrompt: AGILE_SYSTEM,
  toolboxActions: ['write-section', 'improve', 'add-requirements', 'add-risks', 'generate-diagram', 'summarize'],
  initialContent: {
    type: 'doc',
    content: [
      h(1, 'Agile Product Requirements'),
      meta([['Product', '(enter product name)'], ['Version', '1.0'], ['Status', 'Draft'], ['Date', today]]),

      callout('success', 'Product Vision', 'FOR [target customer] WHO [need or opportunity], the [product name] IS A [product category] THAT [key benefit]. UNLIKE [primary alternative], OUR PRODUCT [primary differentiation].'),

      h(2, '1. Goals & Success Metrics'),
      kpi('OKR Dashboard', [
        { id: '1', value: 'TBD', label: 'KR-01 Metric',   delta: 'baseline: ?',  trend: 'neutral' },
        { id: '2', value: 'TBD', label: 'KR-02 Metric',   delta: 'baseline: ?',  trend: 'neutral' },
        { id: '3', value: 'TBD', label: 'KR-03 Metric',   delta: 'baseline: ?',  trend: 'neutral' },
        { id: '4', value: 'TBD', label: 'KR-04 Metric',   delta: 'baseline: ?',  trend: 'neutral' },
      ]),

      h(2, '2. User Personas'),
      callout('info', 'Persona 1 — [Role Name]', 'Goals: [what they want to achieve]\nFrustrations: [current pain points]\nKey behaviour: [how they currently solve the problem]'),
      callout('info', 'Persona 2 — [Role Name]', 'Goals: [what they want to achieve]\nFrustrations: [current pain points]\nKey behaviour: [how they currently solve the problem]'),

      h(2, '3. Epics'),
      steps('Epic Delivery Sequence', [
        { id: '1', name: 'Epic 1: [Name]',  status: 'todo' },
        { id: '2', name: 'Epic 2: [Name]',  status: 'todo' },
        { id: '3', name: 'Epic 3: [Name]',  status: 'todo' },
        { id: '4', name: 'Epic 4: [Name]',  status: 'todo' },
      ]),

      h(2, '4. User Stories'),
      req('US-001', 'SHALL', 'HIGH',   'OPEN', 'As a [Persona 1], I want [action] so that [benefit]. Acceptance: Given [context] When [action] Then [outcome].', 'Epic 1'),
      req('US-002', 'SHALL', 'HIGH',   'OPEN', 'As a [Persona 1], I want [action] so that [benefit]. Acceptance: Given [context] When [action] Then [outcome].', 'Epic 1'),
      req('US-003', 'SHALL', 'MEDIUM', 'OPEN', 'As a [Persona 2], I want [action] so that [benefit]. Acceptance: Given [context] When [action] Then [outcome].', 'Epic 2'),

      h(2, '5. Definition of Done'),
      callout('success', 'Definition of Done', '✓ Code reviewed by ≥ 1 peer\n✓ Unit test coverage ≥ 80% for new code\n✓ Acceptance criteria verified by QA\n✓ No open P1/P2 bugs\n✓ Documentation updated\n✓ Deployed to staging and smoke-tested'),

      h(2, '6. Risks & Blockers'),
      callout('danger',  'Risk 1 — [Title]',   'Likelihood: High · Impact: High · Owner: [Name] · Due: [Date]'),
      callout('warning', 'Open Question 1',     'Question: [What needs to be decided?] · Owner: [Name] · Deadline: [Date]'),
    ],
  },
};

// ── BRD: Enterprise ───────────────────────────────────────────────────────────
const BRD_ENTERPRISE: DesignerTemplate = {
  id: 'brd-enterprise',
  name: 'Enterprise BRD',
  description: 'Full formal BRD with business case, ROI analysis, stakeholder matrix, governance, budget, and approvals — for large-scale initiatives.',
  category: 'brd',
  badge: 'ENT',
  systemPrompt: ENTERPRISE_SYSTEM,
  toolboxActions: ['write-section', 'improve', 'add-requirements', 'add-risks', 'add-stakeholders', 'generate-diagram', 'summarize'],
  initialContent: {
    type: 'doc',
    content: [
      h(1, 'Enterprise Business Requirements Document'),
      meta([['Project', '(name)'], ['Project ID', '(ID)'], ['Sponsor', '(name)'], ['Programme', '(name)'], ['Version', '1.0'], ['Status', 'Draft'], ['Date', today]]),

      callout('success', 'Executive Summary', 'Problem: [1 sentence quantifying the business problem]\nSolution: [1 sentence describing the proposed solution]\nROI: [expected return and timeframe]\nAsk: [budget / resources / decisions required]'),

      h(2, '1. Business Case'),
      kpi('Financial Impact', [
        { id: '1', value: '$0M',   label: 'Estimated Cost (CAPEX)', delta: '',          trend: 'neutral' },
        { id: '2', value: '$0M',   label: 'Annual OPEX',            delta: '',          trend: 'neutral' },
        { id: '3', value: '$0M',   label: 'Expected Benefit (Y1)',  delta: '',          trend: 'neutral' },
        { id: '4', value: '0 mo',  label: 'Break-Even',             delta: 'estimated', trend: 'neutral' },
      ]),

      callout('info', 'Alternatives Considered', 'Option A: [description] — rejected because [reason]\nOption B: [description] — rejected because [reason]\nSelected: [chosen option] — rationale: [why this is the best choice]'),

      h(2, '2. Stakeholder Analysis'),
      callout('info', 'Stakeholder Matrix', 'Role | Name | Organisation | Interest | Influence | Engagement\nExecutive Sponsor | TBD | TBD | High | High | Inform\nProduct Owner | TBD | TBD | High | High | Collaborate\nEnd Users | TBD | TBD | High | Low | Consult\nIT / Engineering | TBD | TBD | Medium | Medium | Collaborate\nCompliance / Legal | TBD | TBD | Low | High | Inform'),

      h(2, '3. Requirements'),
      req('FR-001',  'SHALL', 'HIGH',   'OPEN', 'The system SHALL [core functional requirement].', 'Functional, Core'),
      req('FR-002',  'SHALL', 'HIGH',   'OPEN', 'The system SHALL [second functional requirement].', 'Functional, Core'),
      req('REG-001', 'SHALL', 'HIGH',   'OPEN', 'The solution SHALL comply with [regulation / standard] including [specific clauses].', 'Compliance, Regulatory'),
      req('NFR-001', 'SHALL', 'MEDIUM', 'OPEN', 'The system SHALL maintain a maximum planned downtime of 4 hours per quarter.', 'Non-Functional, Availability'),

      adr('GOV-001', 'Programme governance model', 'PROPOSED',
        'Large-scale initiative requires formal governance. Options: steering committee, product council, or delegated authority.',
        'Establish a monthly Steering Committee with Executive Sponsor, Programme Director, and key stakeholders.',
        'Ensures executive visibility and timely decision-making. Adds meeting overhead (~2h/month per stakeholder).'),

      h(2, '4. Programme Timeline'),
      steps('Programme Phases', [
        { id: '1', name: 'Mobilisation',    status: 'todo' },
        { id: '2', name: 'Discovery & Design', status: 'todo' },
        { id: '3', name: 'Build Phase 1',   status: 'todo' },
        { id: '4', name: 'Build Phase 2',   status: 'todo' },
        { id: '5', name: 'UAT & Cutover',   status: 'todo' },
        { id: '6', name: 'Hypercare',       status: 'todo' },
      ]),

      h(2, '5. Risks'),
      callout('danger',  'Risk — Scope Creep',    'Likelihood: High · Impact: High · Mitigation: Formal change control process with Steering Committee approval for scope changes > 5 days effort.'),
      callout('warning', 'Risk — Vendor Delay',   'Likelihood: Medium · Impact: High · Mitigation: Include SLA penalties in vendor contract. Identify backup vendor.'),

      h(2, '6. Approval Sign-Off'),
      callout('warning', 'Pending Approvals', 'Executive Sponsor: [Name] — Signature: _____ — Date: _____\nProgramme Director: [Name] — Signature: _____ — Date: _____\nIT Architecture: [Name] — Signature: _____ — Date: _____'),
    ],
  },
};

// ── BRD: Lean ─────────────────────────────────────────────────────────────────
const BRD_LEAN: DesignerTemplate = {
  id: 'brd-lean',
  name: 'Lean BRD',
  description: 'Minimal, fast BRD for startups or rapid initiatives — problem, solution, requirements, and metrics on a single page.',
  category: 'brd',
  badge: 'LEAN',
  systemPrompt: LEAN_SYSTEM,
  toolboxActions: ['write-section', 'improve', 'add-requirements', 'add-risks', 'generate-diagram', 'summarize'],
  initialContent: {
    type: 'doc',
    content: [
      h(1, 'Lean Business Requirements'),
      meta([['Initiative', '(name)'], ['Owner', '(name)'], ['Date', today], ['Status', 'Draft']]),

      callout('warning', 'Problem', 'In 2–3 sentences: what is broken, who feels the pain, and what does it cost the business today?'),
      callout('success', 'Solution', 'In 2–3 sentences: what are we building and why is this the right approach over alternatives?'),

      h(2, '1. Core Requirements'),
      callout('info', 'Keep it lean', 'Maximum 8 requirements. If you have more, you need a Classic BRD, not a Lean one.'),
      req('REQ-001', 'SHALL', 'HIGH',   'OPEN', 'The system SHALL [most critical requirement].', 'Core'),
      req('REQ-002', 'SHALL', 'HIGH',   'OPEN', 'The system SHALL [second critical requirement].', 'Core'),
      req('REQ-003', 'SHOULD', 'MEDIUM','OPEN', 'The system SHOULD [desirable requirement].', 'Enhancement'),

      h(2, '2. Success Metrics'),
      kpi('Targets', [
        { id: '1', value: 'TBD', label: 'Primary Metric',    delta: 'baseline: ?', trend: 'neutral' },
        { id: '2', value: 'TBD', label: 'Secondary Metric',  delta: 'baseline: ?', trend: 'neutral' },
      ]),

      h(2, '3. Constraints'),
      callout('danger', 'Hard Constraints', 'Timeline: [date]\nBudget: [£/$ amount]\nTechnology: [must use / cannot use]\nRegulatory: [any compliance requirements]'),

      h(2, '4. Open Questions'),
      callout('warning', 'Must Resolve Before Build', '[Question 1] — Owner: [Name] · Deadline: [Date]\n[Question 2] — Owner: [Name] · Deadline: [Date]'),
    ],
  },
};

// ── HLD: System Architecture ──────────────────────────────────────────────────
const HLD_SYSTEM_ARCH: DesignerTemplate = {
  id: 'hld-system-arch',
  name: 'System Architecture',
  description: 'C4-model system context, major containers, integration points, and architectural decisions.',
  category: 'hld',
  badge: 'HLD',
  systemPrompt: HLD_SYSTEM,
  toolboxActions: ['write-section', 'improve', 'add-requirements', 'add-risks', 'generate-diagram', 'summarize'],
  initialContent: {
    type: 'doc',
    content: [
      h(1, 'High-Level Design Document'),
      meta([['System', '(name)'], ['Version', '1.0'], ['Status', 'Draft'], ['Date', today]]),

      callout('info', 'Architecture Principles', 'Key principles guiding this design:\n• Separation of concerns — each service has a single responsibility\n• Design for failure — assume every dependency will fail\n• Security by default — zero-trust networking, least-privilege access\n• Observable — every service emits structured logs, metrics, and traces'),

      h(2, '1. System Context (C4 Level 1)'),
      diagram('System Context Diagram', `flowchart TB
  User["👤 User"]
  Admin["👤 Admin"]

  subgraph System["[System Name]"]
    API["API Layer"]
    Core["Core Service"]
    DB[("Database")]
  end

  ExtAuth["🔐 Identity Provider"]
  ExtSvc["📦 External Service"]
  Notify["📧 Notification Service"]

  User  --> API
  Admin --> API
  API   --> Core
  Core  --> DB
  API   <--> ExtAuth
  Core  --> ExtSvc
  Core  --> Notify`),

      h(2, '2. Container Architecture (C4 Level 2)'),
      diagram('Container Architecture', `flowchart LR
  Client["Web / Mobile Client"]

  subgraph Backend ["Backend Services"]
    GW["API Gateway\n(Auth, Rate Limit, Routing)"]
    SVC["Core Service\n(Business Logic)"]
    WRK["Background Worker\n(Async Jobs)"]
  end

  subgraph Data ["Data Layer"]
    DB[("Primary DB\nPostgreSQL")]
    CACHE[("Cache\nRedis")]
    QUEUE["Message Queue"]
  end

  Client --> GW
  GW --> SVC
  SVC --> DB
  SVC --> CACHE
  SVC --> QUEUE
  QUEUE --> WRK
  WRK --> DB`),

      h(2, '3. Integration Points'),
      callout('warning', 'External Dependencies', 'List every external system this solution depends on. For each, document: SLA, auth method, data format, failure behaviour, and owner.\n\n• Identity Provider (SSO): SAML 2.0, 99.9% SLA\n• [External Service 1]: REST API, API Key auth\n• [External Service 2]: Webhook, HMAC signature'),

      h(2, '4. Architectural Decisions'),
      adr('ADR-001', 'Primary data store selection', 'PROPOSED',
        'Multiple services need ACID transactions. Options evaluated: PostgreSQL, MySQL, MongoDB, DynamoDB.',
        'Use PostgreSQL 16 as the primary relational store. Each bounded context gets its own schema.',
        'Pros: ACID guarantees, rich query capability, strong tooling. Cons: vertical scaling ceiling — plan for read replicas when reads exceed 10k/s.'),

      adr('ADR-002', 'API communication pattern', 'PROPOSED',
        'Services need to exchange data. Options: REST, GraphQL, gRPC, async events.',
        'Synchronous REST for user-facing operations (latency-sensitive). Async events via message queue for background operations.',
        'REST is well-understood and easy to debug. Async events decouple services and improve resilience. Added complexity of eventual consistency for async paths.'),

      h(2, '5. Non-Functional Requirements'),
      req('NFR-001', 'SHALL', 'HIGH',   'OPEN', 'The system SHALL serve API responses with P99 latency ≤ 200ms for read operations under a load of 500 concurrent requests.', 'Performance'),
      req('NFR-002', 'SHALL', 'HIGH',   'OPEN', 'The system SHALL achieve ≥ 99.9% monthly uptime, measured by external health checks at 1-minute intervals.', 'Availability'),
      req('NFR-003', 'SHALL', 'HIGH',   'OPEN', 'All data in transit SHALL be encrypted using TLS 1.2+. All data at rest SHALL be encrypted using AES-256.', 'Security'),
      req('NFR-004', 'SHOULD', 'MEDIUM','OPEN', 'The system SHOULD emit structured JSON logs to a centralised log aggregation service (e.g. Datadog / ELK).', 'Observability'),

      h(2, '6. Deployment Overview'),
      steps('Deployment Pipeline', [
        { id: '1', name: 'Code Commit',     status: 'todo' },
        { id: '2', name: 'CI Build & Test', status: 'todo' },
        { id: '3', name: 'Security Scan',   status: 'todo' },
        { id: '4', name: 'Deploy Staging',  status: 'todo' },
        { id: '5', name: 'Smoke Tests',     status: 'todo' },
        { id: '6', name: 'Deploy Prod',     status: 'todo' },
      ]),

      h(2, '7. Risks & Open Issues'),
      callout('danger',  'Risk — [Technical Risk]',  'What could go wrong technically? What is the impact? What is the mitigation or proof-of-concept needed?'),
      callout('warning', 'Open Issue — [Topic]',     'What still needs to be decided? Who owns it? What is the deadline?'),
    ],
  },
};

// ── HLD: Microservices ────────────────────────────────────────────────────────
const HLD_MICROSERVICES: DesignerTemplate = {
  id: 'hld-microservices',
  name: 'Microservices Architecture',
  description: 'Service topology, domain ownership, async messaging patterns, data isolation, and deployment strategy.',
  category: 'hld',
  badge: 'MSA',
  systemPrompt: MSA_SYSTEM,
  toolboxActions: ['write-section', 'improve', 'add-requirements', 'add-risks', 'generate-diagram', 'summarize'],
  initialContent: {
    type: 'doc',
    content: [
      h(1, 'Microservices Architecture Design'),
      meta([['System', '(name)'], ['Version', '1.0'], ['Status', 'Draft'], ['Date', today]]),

      callout('info', 'Design Principles', '• Each service owns its data — no shared databases\n• Services communicate via well-defined APIs or async events\n• Every service is independently deployable\n• Design for failure — every remote call can fail'),

      h(2, '1. Service Topology'),
      diagram('Service Map', `graph LR
  subgraph "Client Layer"
    WEB["Web App"]
    MOB["Mobile App"]
  end

  subgraph "Gateway Layer"
    GW["API Gateway\n(Auth + Routing)"]
  end

  subgraph "Domain Services"
    US["User Service"]
    OS["Order Service"]
    PS["Payment Service"]
    NS["Notification Service"]
  end

  subgraph "Data Layer"
    UDB[("Users DB")]
    ODB[("Orders DB")]
    MQ["Message Broker"]
  end

  WEB & MOB --> GW
  GW --> US & OS & PS
  US --> UDB
  OS --> ODB
  OS --event--> MQ
  MQ --> NS
  PS --event--> MQ`),

      h(2, '2. Service Responsibilities'),
      callout('info', 'User Service', 'Owns: user identity, profiles, preferences\nExposes: REST API for user CRUD, authentication token generation\nConsumes: identity provider events\nDB: PostgreSQL — users schema'),

      callout('info', 'Order Service', 'Owns: order lifecycle, order items, fulfilment status\nExposes: REST API for order management\nConsumes: payment.completed event from Payment Service\nDB: PostgreSQL — orders schema\nPublishes: order.created, order.fulfilled events'),

      callout('info', 'Payment Service', 'Owns: payment intents, transactions, refunds\nExposes: REST API for payment initiation\nConsumes: order.created event\nPublishes: payment.completed, payment.failed events\nExternal: Stripe / Adyen gateway'),

      h(2, '3. Architectural Decisions'),
      adr('ADR-001', 'Service boundary definition — Order and Inventory', 'PROPOSED',
        'Should Inventory management live inside Order Service or as a separate service? Splitting adds complexity; merging risks growing a monolith.',
        'Keep Inventory as a separate service. Order Service calls Inventory synchronously for stock checks and subscribes to inventory.reserved events.',
        'Clear domain ownership. Allows Inventory to be scaled independently during peak. Trade-off: one more service to deploy and monitor.'),

      adr('ADR-002', 'Async messaging broker selection', 'PROPOSED',
        'Need durable, ordered, replay-capable messaging. Options: RabbitMQ, Kafka, AWS SQS/SNS, Azure Service Bus.',
        'Use Apache Kafka for event streaming. Topic-per-domain-event pattern.',
        'Durable, replayable, high-throughput. Consumer groups enable fan-out. Complexity: Kafka cluster ops, schema registry needed for evolution.'),

      h(2, '4. Resilience Patterns'),
      callout('warning', 'Failure Handling', 'Circuit Breaker: wrap all synchronous downstream calls — open after 5 failures in 10s, half-open after 30s.\nRetry: exponential backoff with jitter — max 3 retries, cap at 30s.\nTimeout: all outbound HTTP calls — 2s timeout for user-facing, 10s for background.\nBulkhead: separate thread pools for each downstream service.'),

      h(2, '5. Observability'),
      kpi('SLO Targets', [
        { id: '1', value: '99.9%',  label: 'API Gateway Uptime',  delta: 'per month', trend: 'neutral' },
        { id: '2', value: '< 150ms',label: 'P99 User Service',    delta: 'read ops',  trend: 'neutral' },
        { id: '3', value: '< 500ms',label: 'P99 Order Service',   delta: 'write ops', trend: 'neutral' },
        { id: '4', value: '100%',   label: 'Distributed Tracing', delta: 'coverage',  trend: 'neutral' },
      ]),

      h(2, '6. Deployment Strategy'),
      steps('Deployment Pipeline', [
        { id: '1', name: 'Build & Unit Test',   status: 'todo' },
        { id: '2', name: 'Integration Tests',   status: 'todo' },
        { id: '3', name: 'Container Registry',  status: 'todo' },
        { id: '4', name: 'Deploy to Staging',   status: 'todo' },
        { id: '5', name: 'Contract Tests',      status: 'todo' },
        { id: '6', name: 'Canary → Production', status: 'todo' },
      ]),

      h(2, '7. Risks & Open Issues'),
      callout('danger',  'Risk — Distributed transaction complexity', 'Sagas or 2PC needed for cross-service workflows (e.g. Order + Payment + Inventory). Mitigation: implement Saga orchestrator pattern. Spike: 2 days.'),
      callout('warning', 'Open Issue — Schema evolution strategy',     'How do we handle backward-incompatible event schema changes? Decision needed before first production event. Owner: [Name]. Due: [Date].'),
    ],
  },
};

// ── HLD: Cloud Native ─────────────────────────────────────────────────────────
const HLD_CLOUD: DesignerTemplate = {
  id: 'hld-cloud-native',
  name: 'Cloud Native Design',
  description: 'Cloud services selection, infrastructure topology, scaling strategy, IaC, and security posture for cloud-first architectures.',
  category: 'hld',
  badge: 'CND',
  systemPrompt: CLOUD_SYSTEM,
  toolboxActions: ['write-section', 'improve', 'add-requirements', 'add-risks', 'generate-diagram', 'summarize'],
  initialContent: {
    type: 'doc',
    content: [
      h(1, 'Cloud Native Architecture Design'),
      meta([['System', '(name)'], ['Cloud Provider', '(AWS / Azure / GCP)'], ['Region', '(primary)'], ['Version', '1.0'], ['Status', 'Draft'], ['Date', today]]),

      callout('info', 'Cloud Strategy', 'Design principles: 12-factor app, everything-as-code, immutable infrastructure, auto-scaling, zero-trust networking.\nCloud provider: [AWS / Azure / GCP] — selected for [reason: existing enterprise agreement / specific managed service / team expertise].'),

      h(2, '1. Infrastructure Architecture'),
      diagram('Cloud Infrastructure', `flowchart TB
  subgraph "Public Internet"
    CDN["CDN / WAF"]
  end

  subgraph "Cloud Region (Primary)"
    LB["Load Balancer"]

    subgraph "AZ-1"
      APP1["App Tier\n(Container / Serverless)"]
    end
    subgraph "AZ-2"
      APP2["App Tier\n(Container / Serverless)"]
    end

    subgraph "Data Tier"
      DB_P[("Primary DB")]
      DB_R[("Read Replica")]
      CACHE[("Cache Cluster")]
      STORAGE["Object Storage"]
    end

    QUEUE["Message Queue"]
    SECRETS["Secrets Manager"]
  end

  CDN --> LB
  LB --> APP1 & APP2
  APP1 & APP2 --> DB_P & CACHE & QUEUE
  DB_P --> DB_R
  APP1 & APP2 --> STORAGE
  APP1 & APP2 --> SECRETS`),

      h(2, '2. Service Level Objectives'),
      kpi('SLO Dashboard', [
        { id: '1', value: '99.9%',  label: 'Availability SLO',     delta: 'per month',     trend: 'neutral' },
        { id: '2', value: '4h',     label: 'RTO',                   delta: 'max recovery',  trend: 'neutral' },
        { id: '3', value: '1h',     label: 'RPO',                   delta: 'max data loss',  trend: 'neutral' },
        { id: '4', value: '< 200ms',label: 'P99 API Latency',       delta: 'global avg',    trend: 'neutral' },
      ]),

      h(2, '3. Security Posture'),
      adr('ADR-001', 'Zero-trust network architecture', 'ACCEPTED',
        'Traditional perimeter-based security is insufficient for cloud. All internal traffic must be authenticated.',
        'Implement zero-trust: all service-to-service calls use mTLS. No implicit trust based on network location. Short-lived credentials via service accounts.',
        'Significantly improves security posture. Additional latency (~1ms) for mTLS handshakes. Requires certificate management infrastructure.'),

      req('SEC-001', 'SHALL', 'HIGH', 'OPEN', 'All data in transit SHALL use TLS 1.2+ (TLS 1.3 preferred). All data at rest SHALL use AES-256 encryption.', 'Security, Encryption'),
      req('SEC-002', 'SHALL', 'HIGH', 'OPEN', 'No secrets SHALL be stored in source code, environment variables, or container images. All secrets SHALL be retrieved from a managed secrets service at runtime.', 'Security, Secrets'),
      req('SEC-003', 'SHALL', 'HIGH', 'OPEN', 'All infrastructure changes SHALL be applied via IaC (Terraform / Pulumi). No manual changes to production resources.', 'Security, IaC'),

      h(2, '4. Cost Management'),
      kpi('Estimated Monthly Cost', [
        { id: '1', value: '$0',   label: 'Compute',        delta: 'estimate', trend: 'neutral' },
        { id: '2', value: '$0',   label: 'Storage & DB',   delta: 'estimate', trend: 'neutral' },
        { id: '3', value: '$0',   label: 'Network / CDN',  delta: 'estimate', trend: 'neutral' },
        { id: '4', value: '$0',   label: 'Total / Month',  delta: 'estimate', trend: 'neutral' },
      ]),

      callout('warning', 'Cost Optimisation Levers', '• Use Reserved Instances / Committed Use for baseline workloads (save ~30%)\n• Spot / Preemptible instances for batch processing\n• Auto-scaling to zero for dev/staging environments outside business hours\n• Right-size compute instances after 30 days of production data'),

      h(2, '5. Migration Phases'),
      steps('Cloud Migration', [
        { id: '1', name: 'Foundation (IaC, Networking, IAM)',   status: 'todo' },
        { id: '2', name: 'Lift & Shift (Rehost)',                status: 'todo' },
        { id: '3', name: 'Optimise (Replatform)',                status: 'todo' },
        { id: '4', name: 'Modernise (Refactor to cloud-native)',  status: 'todo' },
        { id: '5', name: 'Decommission Legacy',                  status: 'todo' },
      ]),

      h(2, '6. Risks'),
      callout('danger',  'Risk — Vendor lock-in',    'Managed cloud services (Lambda, DynamoDB, Pub/Sub) create tight coupling. Mitigation: abstract behind interfaces; document which services are cloud-specific and why.'),
      callout('warning', 'Risk — Cost overrun',      'Auto-scaling can generate unexpected costs. Mitigation: set billing alerts at 80% and 100% of monthly budget. Enable anomaly detection on cost dashboard.'),
    ],
  },
};

// ── LLD: Component Design ─────────────────────────────────────────────────────
const LLD_COMPONENT: DesignerTemplate = {
  id: 'lld-component',
  name: 'Component Design',
  description: 'Detailed component specification: responsibilities, data models, API contracts, sequence flows, and error handling.',
  category: 'lld',
  badge: 'LLD',
  systemPrompt: LLD_SYSTEM,
  toolboxActions: ['write-section', 'improve', 'add-requirements', 'generate-diagram', 'summarize'],
  initialContent: {
    type: 'doc',
    content: [
      h(1, 'Low-Level Design Document'),
      meta([['Component', '(name)'], ['Parent System', '(system name)'], ['Version', '1.0'], ['Status', 'Draft'], ['Date', today]]),

      callout('info', 'Component Responsibility', 'Single sentence: this component is responsible for [ONE thing].\nDependencies: [list upstream services]\nConsumers: [list downstream services / clients]'),

      h(2, '1. Sequence Flow — Happy Path'),
      diagram('Main Sequence Flow', `sequenceDiagram
  autonumber
  Client->>+API: POST /resource { body }
  API->>+Service: processRequest(dto)
  Service->>+Repository: findById(id)
  Repository->>+DB: SELECT ...
  DB-->>-Repository: row data
  Repository-->>-Service: Entity
  Service->>Service: validateAndTransform()
  Service->>+Repository: save(entity)
  Repository->>+DB: INSERT/UPDATE ...
  DB-->>-Repository: affected rows
  Repository-->>-Service: savedEntity
  Service-->>-API: ResponseDto
  API-->>-Client: 200 OK { data }`),

      h(2, '2. API Contracts'),
      req('API-001', 'SHALL', 'HIGH', 'OPEN', 'GET /[resource]\n→ 200: { data: [...], meta: { total, page, limit } }\n→ 400: { errors: [{ code, message, field }] }\n→ 401: Unauthorized\n→ 500: Internal Error', 'API, Read'),

      req('API-002', 'SHALL', 'HIGH', 'OPEN', 'POST /[resource]\n→ 201: { data: { id, ...fields, createdAt } }\n→ 400: Validation error\n→ 409: Conflict (duplicate)\n→ 422: Business rule violation', 'API, Write'),

      req('API-003', 'SHALL', 'HIGH', 'OPEN', 'PUT /[resource]/:id\n→ 200: { data: { id, ...updatedFields, updatedAt } }\n→ 404: Resource not found\n→ 409: Optimistic lock conflict (include expectedVersion in request)', 'API, Write'),

      h(2, '3. Data Model'),
      diagram('Entity Relationships', `erDiagram
  ENTITY_A {
    uuid      id          PK
    string    name
    string    status
    uuid      owner_id    FK
    timestamp created_at
    timestamp updated_at
  }

  ENTITY_B {
    uuid      id          PK
    uuid      entity_a_id FK
    string    type
    jsonb     metadata
    timestamp created_at
  }

  ENTITY_A ||--o{ ENTITY_B : "has many"`),

      h(2, '4. Error Handling'),
      callout('danger', 'Error Scenarios', 'Document every non-happy path:\n• Validation failure → 400 with field-level errors\n• Downstream service timeout → 503 with retry-after header\n• Optimistic lock conflict → 409 with current version\n• Unexpected exception → 500 (never expose stack trace in response)'),

      callout('warning', 'Retry & Circuit Breaker', 'Downstream calls: max 3 retries, exponential backoff (1s, 2s, 4s), jitter ±500ms.\nCircuit breaker: open after 5 failures in 10s. Half-open probe every 30s.\nDead-letter queue: failed async messages retained for 7 days with alerting.'),

      h(2, '5. Performance Requirements'),
      req('PERF-001', 'SHALL', 'HIGH',   'OPEN', 'Read endpoints SHALL return P99 response ≤ 100ms at 200 RPS. Index strategy: [list required indexes].', 'Performance'),
      req('PERF-002', 'SHALL', 'HIGH',   'OPEN', 'Write endpoints SHALL return P99 response ≤ 300ms. DB writes SHALL use connection pool (min: 5, max: 20).', 'Performance'),
      req('PERF-003', 'SHOULD', 'MEDIUM','OPEN', 'Frequently-read data SHOULD be cached (TTL: 5 minutes). Cache invalidation on write.', 'Performance, Caching'),

      kpi('Performance SLAs', [
        { id: '1', value: '100ms', label: 'P99 Read Latency',  delta: 'at 200 RPS',   trend: 'neutral' },
        { id: '2', value: '300ms', label: 'P99 Write Latency', delta: 'at 200 RPS',   trend: 'neutral' },
        { id: '3', value: '99.9%', label: 'Success Rate',      delta: 'excl. 4xx',    trend: 'neutral' },
        { id: '4', value: '< 1%',  label: 'Error Rate',        delta: 'target',        trend: 'neutral' },
      ]),
    ],
  },
};

// ── LLD: Database Design ──────────────────────────────────────────────────────
const LLD_DATABASE: DesignerTemplate = {
  id: 'lld-database',
  name: 'Database Design',
  description: 'Entity-relationship model, table definitions, indexes, constraints, query patterns, and migration plan.',
  category: 'lld',
  badge: 'DB',
  systemPrompt: DB_SYSTEM,
  toolboxActions: ['write-section', 'improve', 'generate-diagram', 'summarize'],
  initialContent: {
    type: 'doc',
    content: [
      h(1, 'Database Design Document'),
      meta([['System', '(name)'], ['Database', '(PostgreSQL 16)'], ['Schema', '(schema name)'], ['Version', '1.0'], ['Status', 'Draft'], ['Date', today]]),

      callout('info', 'Design Decisions', 'Database engine: [PostgreSQL / MySQL / MongoDB] — selected for [reasons: ACID, JSON support, existing team expertise].\nNormalisation target: 3NF except where denormalisation is explicitly justified for performance.\nMulti-tenancy: [row-level / schema-per-tenant / none]'),

      h(2, '1. Entity Relationship Diagram'),
      diagram('ER Diagram', `erDiagram
  USERS {
    uuid      id           PK
    string    email        UK
    string    name
    string    status
    timestamp created_at
    timestamp updated_at
  }

  ORGANISATIONS {
    uuid      id           PK
    string    name
    string    plan
    timestamp created_at
  }

  PROJECTS {
    uuid      id           PK
    uuid      org_id       FK
    uuid      owner_id     FK
    string    name
    string    status
    timestamp created_at
    timestamp updated_at
  }

  DOCUMENTS {
    uuid      id           PK
    uuid      project_id   FK
    uuid      created_by   FK
    string    title
    text      content
    string    type
    int       version
    timestamp created_at
    timestamp updated_at
  }

  USERS         }o--|| ORGANISATIONS : "belongs to"
  ORGANISATIONS ||--o{ PROJECTS      : "owns"
  USERS         ||--o{ PROJECTS      : "creates"
  PROJECTS      ||--o{ DOCUMENTS     : "contains"
  USERS         ||--o{ DOCUMENTS     : "authors"`),

      h(2, '2. Key Design Decisions'),
      adr('DB-ADR-001', 'UUIDs vs auto-increment IDs', 'ACCEPTED',
        'Auto-increment IDs are simpler and slightly faster. UUIDs prevent ID enumeration attacks and are better for distributed systems.',
        'Use UUID v4 for all primary keys. Use gen_random_uuid() as the default value.',
        'Eliminates sequential ID guessing. Slightly larger index size (~16 bytes vs 8 bytes for bigint). No meaningful performance impact at our scale.'),

      adr('DB-ADR-002', 'Soft delete vs hard delete', 'PROPOSED',
        'Hard delete removes data permanently (simple, clean). Soft delete retains rows with a deleted_at timestamp (audit trail, recovery).',
        'Use soft delete for all business entities (users, projects, documents). Add deleted_at nullable timestamp column.',
        'Enables audit trail and accidental-deletion recovery. All queries must include WHERE deleted_at IS NULL. Add partial index to enforce this.'),

      h(2, '3. Index Strategy'),
      callout('warning', 'Required Indexes', 'users: CREATE UNIQUE INDEX idx_users_email ON users(email);\nprojects: CREATE INDEX idx_projects_org_id ON projects(org_id);\nprojects: CREATE INDEX idx_projects_owner_id ON projects(owner_id);\ndocuments: CREATE INDEX idx_documents_project_id ON documents(project_id);\ndocuments: CREATE INDEX idx_documents_created_by ON documents(created_by);\nSoft delete: CREATE INDEX idx_documents_active ON documents(project_id) WHERE deleted_at IS NULL;'),

      h(2, '4. Migration Plan'),
      req('DB-001', 'SHALL', 'HIGH', 'OPEN', 'All schema changes SHALL be managed via versioned migration files (Flyway / Liquibase / custom). No manual DDL in production.', 'Migrations, Governance'),
      req('DB-002', 'SHALL', 'HIGH', 'OPEN', 'Every migration SHALL be backward-compatible with the currently deployed application version (blue-green deploy window).', 'Migrations, Safety'),
      req('DB-003', 'SHALL', 'HIGH', 'OPEN', 'Migrations that add NOT NULL columns to existing tables SHALL provide a default value or run as multi-step: add nullable → backfill → add constraint.', 'Migrations, Safety'),

      steps('Migration Delivery Process', [
        { id: '1', name: 'Write migration file',      status: 'todo' },
        { id: '2', name: 'Test on dev copy of prod',  status: 'todo' },
        { id: '3', name: 'Review (DBA / Senior Dev)',  status: 'todo' },
        { id: '4', name: 'Apply to staging',          status: 'todo' },
        { id: '5', name: 'Validate + rollback drill',  status: 'todo' },
        { id: '6', name: 'Apply to production',       status: 'todo' },
      ]),

      callout('danger', 'Migration Risks', 'Long-running migrations: adding indexes on large tables locks the table. Use CREATE INDEX CONCURRENTLY.\nColumn renames: never rename directly — add new column, backfill, update app, drop old column.\nRollback: every migration must have a tested rollback script.'),
    ],
  },
};

// ── LLD: API Contract ─────────────────────────────────────────────────────────
const LLD_API: DesignerTemplate = {
  id: 'lld-api-contract',
  name: 'API Contract',
  description: 'RESTful API specification: auth, versioning, endpoint definitions, request/response schemas, error codes, and rate limiting.',
  category: 'lld',
  badge: 'API',
  systemPrompt: API_SYSTEM,
  toolboxActions: ['write-section', 'improve', 'generate-diagram', 'summarize'],
  initialContent: {
    type: 'doc',
    content: [
      h(1, 'API Contract Document'),
      meta([['API Name', '(name)'], ['Base URL', 'https://api.example.com/v1'], ['Auth', 'Bearer JWT'], ['Version', '1.0'], ['Status', 'Draft'], ['Date', today]]),

      callout('info', 'API Design Principles', 'REST conventions: plural nouns for resources (/users not /getUser), HTTP verbs for actions.\nVersioning: URI path versioning (/v1/) — deprecation notice 6 months before removal.\nResponse envelope: { "data": {...}, "meta": {...}, "errors": [] }'),

      h(2, '1. Authentication & Security'),
      callout('warning', 'Auth Requirements', 'All endpoints require Bearer JWT in Authorization header.\nToken issued by: [Identity Provider URL]\nToken lifetime: 1 hour (access) / 7 days (refresh)\nScopes: read:resources write:resources admin:resources'),

      req('SEC-001', 'SHALL', 'HIGH', 'OPEN', 'All endpoints SHALL return 401 Unauthorized when no token is present and 403 Forbidden when the token lacks required scope.', 'Security, Auth'),
      req('SEC-002', 'SHALL', 'HIGH', 'OPEN', 'All responses SHALL include X-Correlation-ID header matching the request header (or generated if absent). Used for distributed tracing.', 'Security, Tracing'),

      h(2, '2. Request / Response Standards'),
      callout('info', 'Request Headers', 'Required:\n  Authorization: Bearer {token}\n  Content-Type: application/json\nOptional:\n  X-Correlation-ID: {uuid}  — pass-through for tracing\n  Accept-Language: en-GB     — for localised error messages'),

      callout('info', 'Response Envelope', '{\n  "data": { ... },          // null on error\n  "meta": {\n    "requestId": "uuid",\n    "timestamp": "ISO-8601",\n    "pagination": { "total": 0, "page": 1, "limit": 20 }\n  },\n  "errors": [               // empty array on success\n    { "code": "INVALID_PARAM", "message": "...", "field": "email" }\n  ]\n}'),

      h(2, '3. Endpoints'),
      diagram('API Request Flow', `sequenceDiagram
  Client->>+Gateway: Request + Bearer JWT
  Gateway->>+AuthSvc: Validate token
  AuthSvc-->>-Gateway: Token claims
  Gateway->>+API: Forward + X-User-ID header
  API->>+DB: Query
  DB-->>-API: Data
  API-->>-Gateway: 200 { data }
  Gateway-->>-Client: 200 { data } + X-Correlation-ID`),

      req('EP-001', 'SHALL', 'HIGH', 'OPEN', 'GET /[resources]\nQuery: ?page=1&limit=20&sort=createdAt:desc&filter[status]=active\n→ 200: { data: [...], meta: { pagination } }\n→ 400: Invalid query parameter\n→ 401: Missing / invalid token', 'Endpoint, List'),

      req('EP-002', 'SHALL', 'HIGH', 'OPEN', 'POST /[resources]\nBody: { required fields... }\n→ 201: { data: { id, ...fields, createdAt } }\n→ 400: Validation error (field-level errors in response)\n→ 409: Conflict — duplicate unique field\n→ 422: Business rule violation', 'Endpoint, Create'),

      req('EP-003', 'SHALL', 'HIGH', 'OPEN', 'GET /[resources]/:id\n→ 200: { data: { id, ...fields } }\n→ 401: Unauthorized\n→ 403: Forbidden (insufficient scope)\n→ 404: Resource not found', 'Endpoint, Read'),

      req('EP-004', 'SHALL', 'HIGH', 'OPEN', 'PATCH /[resources]/:id\nBody: { partial fields to update }\n→ 200: { data: { id, ...updatedFields, updatedAt } }\n→ 400: Validation error\n→ 404: Not found\n→ 409: Optimistic lock conflict', 'Endpoint, Update'),

      h(2, '4. Error Code Reference'),
      callout('danger', 'Error Codes', 'VALIDATION_ERROR  — 400 — One or more fields failed validation. See "errors" array for field details.\nUNAUTHORIZED      — 401 — No token or token expired.\nFORBIDDEN         — 403 — Valid token but insufficient scope.\nNOT_FOUND         — 404 — Resource with given ID does not exist.\nCONFLICT          — 409 — Unique constraint violation or optimistic lock failure.\nRATE_LIMITED      — 429 — Too many requests. Retry after X-RateLimit-Reset.\nINTERNAL_ERROR    — 500 — Unexpected server error. Never expose stack trace.'),

      h(2, '5. Rate Limiting'),
      kpi('Rate Limits', [
        { id: '1', value: '100/min',  label: 'Standard Tier',   delta: 'per API key',   trend: 'neutral' },
        { id: '2', value: '1000/min', label: 'Premium Tier',    delta: 'per API key',   trend: 'neutral' },
        { id: '3', value: '10/min',   label: 'Auth Endpoints',  delta: 'per IP',        trend: 'neutral' },
        { id: '4', value: '429',      label: 'Exceeded Code',   delta: 'HTTP status',   trend: 'neutral' },
      ]),

      callout('info', 'Rate Limit Headers', 'Every response includes:\n  X-RateLimit-Limit: 100\n  X-RateLimit-Remaining: 87\n  X-RateLimit-Reset: 1719230400 (Unix timestamp)'),
    ],
  },
};

// ── Registry ──────────────────────────────────────────────────────────────────
const TEMPLATES: DesignerTemplate[] = [
  BRD_CLASSIC, BRD_IEEE, BRD_AGILE, BRD_ENTERPRISE, BRD_LEAN,
  HLD_SYSTEM_ARCH, HLD_MICROSERVICES, HLD_CLOUD,
  LLD_COMPONENT, LLD_DATABASE, LLD_API,
];

export const templateRegistry = {
  all(): DesignerTemplate[] { return TEMPLATES; },
  getById(id: string): DesignerTemplate | undefined { return TEMPLATES.find(t => t.id === id); },
  getByCategory(category: string): DesignerTemplate[] { return TEMPLATES.filter(t => t.category === category); },
  register(template: DesignerTemplate): void { TEMPLATES.push(template); },
};
