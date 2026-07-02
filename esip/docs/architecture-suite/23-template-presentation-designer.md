# Volume 23 — Template & Presentation Designer (Content Lifecycle)

**Suite:** ESIP Architecture Automation Suite  
**Volume:** 23  
**Prefix:** TPL  
**Status:** Draft v1.0  
**Date:** 2026-06-23  
**Owner:** Product — Content Lifecycle Domain

---

## Boundary Statement

| Concern | Owner Volume |
|---|---|
| File rendering engines (docx/pptx/xlsx/pdf/html — the byte-level generation) | **V10** |
| Metamodel config storage, multi-tenancy infrastructure, branding infra primitives | **V11** |
| AI content generation (filling placeholders, drafting sections, summarizing) | **V06** |
| Per-user adaptive preferences (learned style, density, pinned templates at user level) | **V14** |
| Adaptive/prescriptive suggestion models, template analytics intelligence | **V16** |
| Review checklists execution & review lifecycle | **V15** |
| Approval gate routing/SLA state machine | **V12** |
| Proposal/RFP content (Volume 22 when authored) — this volume owns template design for proposals | **V22 (cross-ref)** |
| ADR/trade-off record content — this volume owns template design for ADRs | **V04/V21 (cross-ref)** |

**This volume owns:** template design, template types, template lifecycle governance, multi-level hierarchy & override resolution, placeholder/merge engine specification, brand & presentation governance layer, content lifecycle binding (draft → create → review → present → deliver), adaptive suggestion surface, and template-library governance. It does NOT own the byte rendering or the AI content that fills templates.

---

## Table of Contents

1. [Visual Template Designer (WYSIWYG)](#1-visual-template-designer-wysiwyg)
2. [Template Types Across the Content Lifecycle](#2-template-types-across-the-content-lifecycle)
3. [Multi-Level Template Hierarchy & Override](#3-multi-level-template-hierarchy--override)
4. [Content Lifecycle Governance](#4-content-lifecycle-governance)
5. [Placeholder & Merge Engine](#5-placeholder--merge-engine)
6. [Brand & Presentation Governance](#6-brand--presentation-governance)
7. [Adaptive & Prescriptive Suggestion Layer](#7-adaptive--prescriptive-suggestion-layer)
8. [Template Lifecycle Management](#8-template-lifecycle-management)
9. [Template Library & Marketplace](#9-template-library--marketplace)
10. [Security, Access & Governance](#10-security-access--governance)
11. [NFRs](#11-nfrs)
12. [Requirements Summary Table](#12-requirements-summary-table)

---

## Personas Reference

| Code | Role |
|---|---|
| EA | Enterprise Architect (P1) |
| SA | Solution Architect (P2) |
| AA | Application Architect (P3) |
| IA | Integration Architect (P4) |
| DA | Data Architect (P7) |
| SecA | Security Architect (P8) |
| CA | Cloud Architect (P9) |
| PM | Product Manager (P13) |
| ALL | All architecture personas |
| Admin | Tenant/org admin |
| Proj-Lead | Project lead (any persona acting as project config owner) |

---

## 1. Visual Template Designer (WYSIWYG)

### 1.1 Canvas & Layout Designer

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-001 | WYSIWYG Template Canvas | The system SHALL provide a browser-based WYSIWYG canvas where administrators and authorised users can design templates by placing, resizing, and reordering structural elements (sections, headings, sub-headings, tables, text blocks, image placeholders, diagram frames, spacers, page-breaks) using drag-and-drop interaction without writing markup. | UX | Admin, Proj-Lead | Must | Eliminates hand-coded template maintenance; democratises template authoring |
| REQ-TPL-002 | Output-Type Aware Canvas | The canvas SHALL render in a layout mode matching the target output type (document page for DOCX/PDF; slide canvas for PPTX; grid for XLSX; responsive column for HTML/email), so authors see a faithful representation of the rendered output while designing. | UX | Admin, Proj-Lead | Must | Prevents "design-render mismatch" surprises that erode trust |
| REQ-TPL-003 | Responsive Multi-Column Layout | The designer SHALL support multi-column layouts, side-by-side section pairs, full-width vs. contained blocks, and explicit column-width configuration (pixel, percent, or grid-unit), producing layout directives that V10 rendering engines honour. | UX | Admin, Proj-Lead | Must | Executive decks and proposals require sophisticated column layouts |
| REQ-TPL-004 | Section Block Components | The designer SHALL offer a component palette of pre-built structural blocks: cover page, executive summary, section divider, numbered heading hierarchy (H1–H4), callout/note block, data table, key-value property table, decision matrix, comparison grid, SWOT quadrant, status badge row, timeline bar, and appendix block — each insertable as a template section. | UX | Admin, Proj-Lead | Must | Accelerates template construction; ensures output consistency |
| REQ-TPL-005 | Template Grid / Page Ruler | The canvas SHALL display configurable page margins, gutters, and a grid/ruler overlay (toggleable) so authors can align elements to a consistent spatial grid consistent with corporate style guides. | UX | Admin | Should | Precision alignment is required for brand-compliant output |
| REQ-TPL-006 | In-Canvas Typography Controls | The designer SHALL expose font-family, font-size, font-weight, color (from the active brand palette), line-height, letter-spacing, alignment, and list style controls for each text block, with changes previewed instantly on the canvas. | UX | Admin | Must | Typography consistency is the dominant driver of professional output quality |
| REQ-TPL-007 | Table Designer | The designer SHALL include a table-design tool supporting: column/row add/remove, cell merge/split, per-column data type hint (text, number, date, badge, hyperlink), alternating row shading, header row/column freeze, and minimum/maximum column widths. | UX | Admin, Proj-Lead | Must | RTMs, comparison tables, and capability matrices are core architecture deliverables |
| REQ-TPL-008 | Undo / Redo History | The WYSIWYG designer SHALL maintain a per-session undo/redo history of at least 50 design actions, with named action labels visible in the history panel (e.g., "Added section: Executive Summary", "Changed font to Calibri 11"). | UX | Admin, Proj-Lead | Must | Standard expectation for professional design tools |
| REQ-TPL-009 | Zoom & Pan | The canvas SHALL support zoom (25%–400%) and pan navigation, with a minimap thumbnail for large multi-page templates. | UX | Admin, Proj-Lead | Should | Multi-page templates are unwieldy without navigation aids |
| REQ-TPL-010 | Multi-Page / Multi-Slide Management | The designer SHALL provide a page/slide panel listing all pages or slides of the template, supporting: add, delete, duplicate, reorder (drag), and bulk apply of a master layout to selected pages. | UX | Admin, Proj-Lead | Must | Presentation templates and multi-section documents require multi-page management |

### 1.2 Placeholder & Merge-Field Insertion

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-011 | Placeholder Insertion Panel | The designer SHALL provide a searchable placeholder panel listing all available merge fields for the active template type, grouped by source domain (Project metadata, Requirement fields, Architecture model properties, Portfolio data, AI-generated content, User/reviewer metadata, Date/time fields, Computed fields, Custom fields), insertable into any text block or table cell by clicking or dragging. | Functional | Admin, Proj-Lead | Must | Discoverability of available fields is the #1 usability bottleneck in template tooling |
| REQ-TPL-012 | Inline Placeholder Token Rendering | Inserted placeholders SHALL render as distinctly styled tokens (coloured chip with field name) inside the canvas text flow, never as raw `{{ syntax }}`, so authors can clearly distinguish static text from dynamic fields. | UX | Admin, Proj-Lead | Must | Raw mustache syntax in the canvas is confusing and error-prone |
| REQ-TPL-013 | Required vs. Optional Placeholder Flag | Each placeholder insertion SHALL allow the author to mark it as Required or Optional, with Required placeholders producing a validation error at generation time if the bound data is null or empty. | Functional | Admin, Proj-Lead | Must | Prevents incomplete deliverables from being generated |
| REQ-TPL-014 | Fallback Value on Placeholder | Each placeholder SHALL support a configurable fallback value (static text or another field reference) that is used when the primary binding resolves to null, ensuring graceful degradation instead of blank gaps. | Functional | Admin, Proj-Lead | Must | Null field values in live data must never produce visible gaps in executive documents |
| REQ-TPL-015 | Computed Field Expressions | The designer SHALL support computed placeholder expressions combining field values, arithmetic, date arithmetic, string concatenation, conditional (IF/ELSE), and lookup (map a code to a label), evaluated at render time by the merge engine. | Functional | Admin | Must | Derived fields (e.g., "Risk Score = Likelihood × Impact", "Status label from enum code") are ubiquitous in architecture deliverables |
| REQ-TPL-016 | Diagram Frame Placeholder | The designer SHALL support a "diagram frame" placeholder block that binds to a named diagram or diagram type from the ESIP model, with configurable target dimensions and caption, so rendered output embeds the actual architecture diagram at that position. | Functional | Admin, ALL | Must | Architecture deliverables are defined by embedded diagrams — this is a primary use case |
| REQ-TPL-017 | Image / Logo Placeholder | The designer SHALL support image placeholders that bind to: tenant logo, project logo, persona-specific icon, or an image field from the data model, with configurable scaling mode (fit, fill, stretch) and alt-text field for accessibility. | Functional | Admin | Must | Branded deliverables require dynamic logo injection |

### 1.3 Conditional Sections

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-018 | Conditional Section Blocks | The designer SHALL allow wrapping any section, table, or content block in a conditional visibility rule: a boolean expression over data fields, persona, project type, framework, or custom metadata — the block renders only when the condition evaluates to true at generation time. | Functional | Admin, Proj-Lead | Must | Single template serves multiple contexts (e.g., "Show Integration Architecture section only if project type includes integration") |
| REQ-TPL-019 | Condition Builder UI | Conditional rules SHALL be authored through a visual condition builder (AND/OR groups, field selector, operator, value picker), not by writing raw expressions, with an inline preview showing which sample-data scenarios would show/hide the block. | UX | Admin, Proj-Lead | Must | Raw expression editing is a barrier to non-developer admins |
| REQ-TPL-020 | Nested Conditional Sections | The designer SHALL support up to five levels of nested conditional blocks within a template, each with independent conditions, to handle complex output personalisation such as audience-specific subsections within conditionally-included sections. | Functional | Admin | Should | Audience-differentiated output (executive vs. technical) within a single template requires nesting |
| REQ-TPL-021 | Conditional Section Preview | The designer SHALL allow previewing the template in a "condition simulator" where the author selects a persona, project type, and framework value to see which conditional blocks would appear in that scenario, highlighted on the canvas. | UX | Admin | Should | Authors need to validate conditional logic without waiting for a full generate cycle |

### 1.4 Repeaters & Loops

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-022 | Repeater Block (Collection Loop) | The designer SHALL support "repeater" blocks that bind to a data collection (e.g., list of requirements, list of risks, list of architecture decisions, list of components) and render one instance of the contained template fragment per collection item, with configurable sort, filter, and max-item-count expressions. | Functional | Admin | Must | Requirements tables, risk registers, ADR logs — all are rendered collection loops |
| REQ-TPL-023 | Nested Repeaters | The system SHALL support at least two levels of nested repeaters (e.g., for each capability area, for each requirement within that area) to generate hierarchically grouped tables and sections. | Functional | Admin | Should | Capability-grouped RTMs and multi-level breakdown structures require nested loops |
| REQ-TPL-024 | Repeater Empty-State Block | Each repeater SHALL have a configurable empty-state block rendered when the bound collection contains zero items, so the output always has meaningful content (e.g., "No risks identified at this time.") rather than a blank section. | Functional | Admin | Must | Empty sections in executive documents are unprofessional |
| REQ-TPL-025 | Repeater Group-By | The repeater SHALL support a group-by field option that inserts a configurable section-header row or divider between groups, enabling category-sorted output (e.g., requirements grouped by priority, risks grouped by domain) from a single repeater block. | Functional | Admin | Should | Grouped structure is the standard format for large requirement or risk tables |

### 1.5 Styling & Brand Binding

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-026 | Brand Theme Binding | Each template SHALL bind to a named brand theme (sourced from V11 branding infra), and all colour, font, logo, and spacing properties SHALL resolve from the bound theme at render time — the template designer stores theme references, not hard-coded values. | Functional | Admin | Must | Decoupling template from brand enables re-skinning without re-designing every template |
| REQ-TPL-027 | Style Override at Block Level | The designer SHALL allow overriding specific style properties (colour, font size, border) on individual blocks within a theme-bound template, with overrides marked visually as "custom — deviates from theme" so future theme updates prompt a reconciliation review. | UX | Admin | Should | Occasional deliberate deviations (e.g., red for critical alerts) must be supported without breaking the whole template |
| REQ-TPL-028 | Paragraph & Heading Style Library | The designer SHALL expose a style library of named paragraph styles (Body, Caption, Quote, Code, Warning) and heading styles (H1–H4) sourced from the active brand theme, applicable to text blocks with a single click. | UX | Admin | Must | Named styles enable downstream format-engine style mapping (Word named styles, PPTX layout styles) |
| REQ-TPL-029 | Table Style Presets | The designer SHALL provide table-style presets (Banded Rows, Header Bold, Minimal, Dashboard) derived from the brand theme, selectable per table block. | UX | Admin | Should | Consistent table aesthetics across all deliverables without per-block manual styling |

### 1.6 Live Preview with Sample Data

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-030 | Live Preview Panel | The designer SHALL provide a live preview panel that renders the template with sample data in near-real-time (< 2 s refresh) as the author edits, showing bound placeholder values, expanded repeaters, evaluated conditions, and brand styling applied. | UX | Admin, Proj-Lead | Must | Designers need immediate feedback to avoid trial-and-error generate cycles |
| REQ-TPL-031 | Sample Data Sets | The system SHALL maintain a library of sample data sets per template type (e.g., a sample BRD data set with 10 requirements, 3 risks, and architectural decisions) that authors select in the preview panel to simulate realistic output volume. | Functional | Admin | Must | Without realistic sample data, previews do not reveal layout issues |
| REQ-TPL-032 | Custom Preview Data | The designer SHALL allow an author to bind preview to a real project in the tenant (read-only), so the preview renders against actual architecture data from that project, enabling final validation before publishing a template. | Functional | Admin | Should | Real data surfaces edge cases (long text, null fields, large collections) that sample data misses |
| REQ-TPL-033 | Multi-Format Preview | The preview panel SHALL support switching between all target output formats (DOCX layout, PPTX slide view, PDF page view, HTML view) within the same preview session, without requiring separate template variants. | UX | Admin | Should | Authors must validate output across all delivery channels before publishing |

### 1.7 Placeholder Validation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-034 | Placeholder Validation on Save | When saving a template, the system SHALL validate all placeholder references against the current metamodel field catalogue, flag any that reference non-existent or deprecated fields, and present an actionable list of broken references with suggested replacements. | Functional | Admin | Must | Metamodel evolution breaks templates silently; proactive validation prevents runtime failures |
| REQ-TPL-035 | Required-Placeholder Coverage Check | Before publishing a template, the system SHALL check that all required output sections contain at least one placeholder or static content, and warn if any required section block is entirely empty (no content and no placeholders). | Functional | Admin | Must | Empty required sections produce unusable deliverables |
| REQ-TPL-036 | Circular Reference Detection | The merge engine SHALL detect and reject circular computed-field expressions (field A references field B which references field A) at template-save time, producing a clear error message identifying the cycle. | Functional | Admin | Must | Circular references cause infinite loops in the render pipeline |
| REQ-TPL-037 | Placeholder Type Mismatch Warning | The designer SHALL warn when a placeholder bound to a numeric or date field is placed in a context expecting text (e.g., inside a sentence), and when a text placeholder is placed in a numeric table column, allowing the author to add a format expression. | UX | Admin | Should | Type mismatches produce garbled output without early detection |

---

## 2. Template Types Across the Content Lifecycle

### 2.1 Document Deliverable Templates

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-038 | BRD/PRD Document Templates | The system SHALL provide configurable templates for Business Requirements Documents and Product Requirements Documents, with sections for executive summary, business context, stakeholders, functional requirements table, non-functional requirements, constraints, assumptions, risk register, and appendices — all placeholders bound to the Requirements Pillar (V07) and metamodel fields. | Functional | PM, ALL | Must | BRD/PRD is the primary PM (P13) deliverable cross-referenced with V13 |
| REQ-TPL-039 | Architecture Document Templates (HLD/LLD) | The system SHALL provide configurable templates for High-Level Design and Low-Level Design documents, with architecture-type-specific section sets (e.g., solution overview, component architecture, data flows, integration points, NFR analysis, deployment view, security controls, ADRs), placeholders bound to the SA/AA/IA metamodel artifacts. | Functional | SA, AA, IA | Must | Core deliverables for P2–P4 personas |
| REQ-TPL-040 | TOGAF ADM Phase Deliverable Templates | The system SHALL provide templates for each TOGAF ADM phase deliverable (Architecture Vision, Business Architecture Document, Information Systems Architecture Document, Technology Architecture Document, Architecture Definition Document, Architecture Roadmap, Architecture Contract, Migration Plan, Architecture Compliance Report, Architecture Change Request) with placeholders bound to the V01 TOGAF metamodel fields. | Functional | EA | Must | Core EA deliverables; TOGAF compliance requires specific document structures |
| REQ-TPL-041 | Capability Assessment & Roadmap Templates | The system SHALL provide templates for capability assessment documents, heat-map reports, and transformation roadmap documents, with repeater blocks over capability domains and bindings to V09 portfolio and roadmap data. | Functional | EA, PM | Must | EA roadmapping deliverables |
| REQ-TPL-042 | Data Architecture Document Templates | The system SHALL provide templates for conceptual/logical/physical data architecture documents, data dictionary, data governance charter, and data lineage report, with placeholders bound to DA (P7) metamodel fields. | Functional | DA | Must | P7 persona deliverables |
| REQ-TPL-043 | Security Architecture Document Templates | The system SHALL provide templates for threat model documents, security architecture design, zero-trust architecture blueprint, and security control catalogue, with placeholders bound to SecA (P8) metamodel fields including STRIDE categories and NIST CSF controls. | Functional | SecA | Must | P8 persona deliverables |
| REQ-TPL-044 | Cloud Architecture Document Templates | The system SHALL provide templates for landing-zone design documents, cloud architecture blueprints, Well-Architected Review documents, and FinOps reports, with placeholders bound to CA (P9) metamodel fields. | Functional | CA | Must | P9 persona deliverables |
| REQ-TPL-045 | Per-Framework Artifact Templates | The system SHALL provide at least one configurable template per artifact type defined in the V11 metamodel artifact registry (Zachman cell artifacts, DoDAF view products, ArchiMate view sets, etc.), ensuring every framework-specific output type has a starting template. | Functional | EA, ALL | Must | Consistency with V10 per-framework deliverables; no orphan artifact types |

### 2.2 Proposal & RFP Response Templates

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-046 | RFP Response Template | The system SHALL provide a configurable RFP Response template type with sections for executive summary, proposed solution overview, architecture approach, team/capability summary, methodology, delivery plan/roadmap, commercial summary, and appendices — placeholder bindings to project data, portfolio data, and AI-generated content fields (cross-ref V22 Proposal module). | Functional | SA, EA, Admin | Must | Proposal generation is a high-value use case referenced in V22 |
| REQ-TPL-047 | Proposal Variant by Audience | The system SHALL allow an RFP/Proposal template to define multiple audience variants (Executive Decision-Maker, Technical Evaluator, Procurement) each revealing different conditional sections and level of detail from the same underlying data, without requiring separate template files. | Functional | SA, Admin | Should | Single source of truth for proposal content; audience-tailored presentation |
| REQ-TPL-048 | Proposal Branding Override | The system SHALL support a per-proposal branding override that allows applying a client-specific colour scheme and logo to a proposal output without changing the tenant-default brand theme. | Functional | SA, Admin | Should | Proposals are often branded to match the client's identity |

### 2.3 ADR & Trade-Off Templates

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-049 | Architecture Decision Record Template | The system SHALL provide a configurable ADR template with canonical sections: decision title, status, context, decision, consequences, options considered, options comparison matrix, and rationale — placeholders bound to the SA ADR metamodel (cross-ref V04/V21). | Functional | SA, AA, EA | Must | ADRs are a primary SA deliverable; template consistency across the estate is essential |
| REQ-TPL-050 | Trade-Off / Options Analysis Template | The system SHALL provide a configurable options analysis template with weighted criteria table, scoring matrix, recommendation, and risk columns — placeholders bound to trade-off model fields and repeater over options. | Functional | SA, EA | Must | Structured options analysis is the core V04 SA deliverable |

### 2.4 Diagram & Diagram-Frame Templates

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-051 | Diagram Frame Template | The system SHALL support "diagram frame" templates that define the surrounding furniture of a diagram when embedded in a document: border style, caption block (title, version, author, date placeholders), legend block (optional, bound to active diagram element types), and north-star narrative block above/below the diagram frame. | Functional | Admin, ALL | Must | Consistent diagram embedding style across all deliverables |
| REQ-TPL-052 | Dashboard Layout Template | The system SHALL support dashboard templates that define a multi-panel grid layout where each panel binds to a chart type, KPI metric, or diagram — producing a branded architecture status dashboard renderable as a document page, HTML page, or presentation slide. | Functional | EA, PM, Admin | Should | Executive status dashboards are a standing deliverable in EA engagements |
| REQ-TPL-053 | Presentation Deck Template | The system SHALL support presentation deck templates defining: title slide, agenda slide, section-divider slide, content slide (with diagram + bullets variant, full-page diagram variant, two-up comparison variant, key-message slide), appendix section, and closing slide — all placeholders bound to architecture model and project data. | Functional | EA, SA, Admin | Must | Presentation decks for governance boards and steering committees are universal deliverables |

### 2.5 Review Checklist Templates

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-054 | Review Checklist Template Type | The system SHALL define a "Review Checklist" template type containing: a checklist section with repeatable line items (check item, category, PASS/FAIL/N-A options, evidence field, finding notes field), a summary section, and a sign-off block — the configured template is referenced by the V15 Review module when initiating a review of a given content type. | Functional | Admin, ALL | Must | Template-driven review checklists eliminate inconsistent ad-hoc reviews |
| REQ-TPL-055 | Checklist Item Configuration | Each review checklist template SHALL allow authors to define checklist items as: static (always present), conditional (shown based on artifact type or field value), or AI-generated (the V06 AI engine populates the item list at review time based on artifact content). | Functional | Admin | Must | AI-generated checklist items (V15/V06) need a template slot to land in |
| REQ-TPL-056 | Persona-Specific Review Checklist Templates | The system SHALL support distinct review checklist templates per persona and per content type (e.g., SA HLD review checklist, BRD review checklist, Security Architecture review checklist), selectable by the review initiator or auto-assigned by the content-lifecycle template binding (see §4). | Functional | Admin, ALL | Must | P1 EA deliverables have different review criteria than P3 AA designs |

### 2.6 Email & Customer-Reply Templates

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-057 | Email Template Type | The system SHALL support an "Email/Notification" template type with subject-line field, greeting block, body sections, signature block, and CTA button — placeholders bound to project data, recipient metadata, artifact name/link, and AI-drafted summary (cross-ref V06). | Functional | Admin | Must | Automated workflow notifications (review requests, delivery confirmations) must be brand-consistent |
| REQ-TPL-058 | Customer Reply / Stakeholder Communication Template | The system SHALL support "Customer Reply" and "Stakeholder Communication" template types for architect-to-stakeholder written communications: structured narrative with placeholders for decision summary, action items, next steps, contacts — renderable as DOCX or email HTML. | Functional | SA, EA, PM | Should | Architects regularly produce formal written communications; template-driven drafting accelerates quality output |

### 2.7 Drafting Prompt Templates

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-059 | Drafting Prompt Template Type | The system SHALL define a "Drafting Prompt" template type that encodes: the AI instruction for drafting a specific content section (system prompt), the data-field bindings to inject into the prompt as context (project name, requirements list, architecture decisions), the output target (which placeholder in a document template the AI draft should populate), and tone/audience/length constraints. | AI/ML | Admin | Must | Template-driven AI drafting (V06) requires a governed, versioned prompt template rather than ad-hoc prompt editing per generation |
| REQ-TPL-060 | Prompt Template Per Section | Each section in a document template MAY reference a distinct drafting prompt template that governs how V06 auto-drafts that section, allowing differentiated instructions for executive summary (concise/business), risk section (formal/evidence-based), and technical narrative (precise/technical). | AI/ML | Admin | Must | Section-level prompt differentiation is essential for coherent multi-section drafting |
| REQ-TPL-061 | Prompt Template Override by User | A user SHALL be able to replace the admin-configured drafting prompt template for a section with a personal prompt variant (cross-ref V14 personalisation), subject to admin-configurable allow-override flags, without affecting other users or the base template. | AI/ML | ALL | Should | Personal voice and style preferences in AI drafting must be user-controllable, not admin-locked |
| REQ-TPL-062 | Prompt Template Versioning & Rollback | Drafting prompt templates SHALL be versioned (see §8), and a user SHALL be able to regenerate a section using a prior prompt template version to compare output quality, supporting iterative prompt refinement governance. | AI/ML | Admin | Should | Prompt regression testing requires version control of prompt templates |

### 2.8 Delivery Packaging Templates

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-063 | Delivery Package Template | The system SHALL support "Delivery Package" templates that define a bundle of related output artefacts (e.g., HLD DOCX + architecture diagrams ZIP + ADR log XLSX + presentation PPTX + review checklist PDF) with a configurable cover manifest document, naming convention expressions, and delivery channel configuration (email, SharePoint, Teams, S3 — cross-ref V10/V19). | Functional | Admin, Proj-Lead | Must | Enterprise delivery requires consistent, complete, correctly named packages — not ad-hoc file exports |
| REQ-TPL-064 | Delivery Naming Convention | Each delivery package template SHALL include a naming convention expression for each artefact file (e.g., `{ProjectCode}_{ArtifactType}_{Version}_{Date}.docx`) using the same merge-field syntax as document templates, evaluated at packaging time. | Functional | Admin | Must | Consistent file naming is an enterprise governance requirement |
| REQ-TPL-065 | Package Manifest Template | The delivery package SHALL optionally include a generated manifest document (cover sheet) using a configurable template, listing all artefacts in the bundle with their titles, versions, authors, and review status. | Functional | Admin | Should | Reviewers and auditors need a single entry point for a delivery package |

---

## 3. Multi-Level Template Hierarchy & Override

### 3.1 Hierarchy Definition

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-066 | Four-Level Template Hierarchy | The system SHALL implement a four-level template hierarchy: (1) System Default — shipped by ESIP product; (2) Org/Admin — tenant-level customisation by admin; (3) Project — project-level override by project lead; (4) User — per-user personalisation (cross-ref V14). Each level inherits from the level above and MAY override specific properties, sections, or settings. | Governance | Admin, Proj-Lead, ALL | Must | Hard-coded templates cause backlash; configurable hierarchy satisfies enterprise, project, and personal diversity |
| REQ-TPL-067 | Inheritance Model (Property-Level) | Template inheritance SHALL be property-level (not whole-template replacement): a child-level template inherits all properties of its parent and only records the delta (changed or added properties/sections), so that a parent-level update propagates to all children that have not explicitly overridden that property. | Governance | Admin | Must | Whole-file inheritance means every parent update requires manual re-apply to children |
| REQ-TPL-068 | Lock / Allow-Override Flags | Each property, section, style, and placeholder in a template SHALL carry a configurable lock flag (Locked, Overridable, Forced-Inherit) settable by the owner of the current level — a Locked property cannot be modified at any lower level; an Overridable property can be changed; Forced-Inherit inherits from parent and ignores any child override. | Governance | Admin | Must | Prevents rogue project or user templates from violating brand or compliance-mandated sections |
| REQ-TPL-069 | Effective Template Computation | The system SHALL compute and display the "effective template" for any given (user, project, template type) combination, showing the merged view of all four levels with per-property source annotation (System / Org / Project / User) and lock status. | Governance | Admin, Proj-Lead | Must | Admins and leads need to audit what a user actually experiences without trial-and-error |
| REQ-TPL-070 | Org-Level Override Scope | An org/admin-level template SHALL apply to all projects in the tenant by default, with the option to scope it to: all projects, selected project types, selected framework activations, or selected persona groups. | Governance | Admin | Must | Enterprise-wide templates must apply universally; exceptions need scoping, not separate copies |
| REQ-TPL-071 | Project-Level Override Scope | A project-level template override SHALL apply only within the current project, and the project lead SHALL only be able to override properties that the org/admin level has marked as Overridable — they cannot override Locked properties. | Governance | Proj-Lead | Must | Delegation of authority without abdicating control |
| REQ-TPL-072 | User-Level Override Scope | A user-level template preference SHALL apply only to that user's own generation sessions and SHALL only cover properties the project or org level has marked as Overridable at the user level — governance layers above always win on Locked properties. | Governance | ALL | Must | User autonomy within governed guardrails |

### 3.2 Override Conflict Resolution

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-073 | Override Resolution Order | When the same property is set at multiple levels, the system SHALL apply the lowest-level non-locked setting: User > Project > Org > System, except where a higher-level Locked flag is present in which case the locked value wins unconditionally regardless of lower-level settings. | Governance | ALL | Must | Unambiguous precedence prevents unpredictable render output |
| REQ-TPL-074 | Conflict Surfacing UI | When a project or user level override is in conflict with (or shadowed by) a higher-level Locked property, the system SHALL surface a clear UI indicator ("Locked by Org — your override is not applied") so users understand why their customisation has no effect. | UX | ALL | Must | Silent lock shadowing causes user confusion and erodes trust |
| REQ-TPL-075 | Override Audit Trail | Every template property change at any level SHALL be recorded in the audit trail with: who changed it, from what value, to what value, at what time, and at which hierarchy level — queryable by admins. | Security | Admin | Must | Enterprise governance requires immutable change records |
| REQ-TPL-076 | Propagation of Parent Update | When a parent-level template property is updated, the system SHALL: (a) identify all child-level templates that inherit that property (not overriding it), (b) propagate the update automatically to those children, and (c) present a reconciliation review list for child templates that DO override the changed property, asking the child owner to review whether their override is still intentional. | Governance | Admin | Must | Without propagation, parent updates silently diverge from children |

---

## 4. Content Lifecycle Governance

### 4.1 Template as Lifecycle Binding

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-077 | Content Lifecycle Binding per Template | Each template in the system SHALL carry a content lifecycle binding that declares, for the content type this template governs: (a) which AI drafting prompt template is invoked during the Draft phase; (b) which review checklist template is applied during the Review phase; (c) which presentation layout template is used during the Present phase; (d) which delivery package template governs the Deliver phase; and (e) which approval workflow (cross-ref V12) is triggered at each phase transition. | Governance | Admin | Must | Changing a template must reshape the entire experience for that content type — this binding is the mechanism |
| REQ-TPL-078 | Phase-Gate Configuration in Template | The template lifecycle binding SHALL allow configuring required vs. optional gates at each phase transition (Draft→Create, Create→Review, Review→Present, Present→Deliver): a required gate blocks progression until the gate condition is met (e.g., at least one reviewer approved); an optional gate is suggested but skippable. | Governance | Admin, Proj-Lead | Must | Governance mandates that sensitive deliverables (security architecture, contracts) require mandatory review before delivery |
| REQ-TPL-079 | Lifecycle Stage Visibility | The system SHALL display the current lifecycle stage of any content item (which template governs it, which phase it is in, what the next gate is, and who needs to act) in the artifact detail view, driven by the template's lifecycle binding. | UX | ALL | Must | Users need to know where a document is in its lifecycle at a glance |
| REQ-TPL-080 | Template-Driven Phase Actions | The actions available at each phase in the content item's lifecycle SHALL be determined by the template lifecycle binding, not hard-coded in the UI — so an admin can add a "Legal Review" gate to any content type's lifecycle by configuring the template binding without a code change. | Governance | Admin | Must | Hard-coded phase actions are the root cause of "system doesn't fit our process" complaints |

### 4.2 Draft Phase Governance

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-081 | AI Draft Invocation from Template | When a user initiates drafting of a content item, the system SHALL automatically invoke the drafting prompt template bound in the template lifecycle binding (§4.1), passing the configured field bindings as context to V06 AI engine, and populating the result into the corresponding document template placeholders. | AI/ML | ALL | Must | Template-driven AI drafting ensures consistent, governed, persona-appropriate AI generation |
| REQ-TPL-082 | Draft Section Completion Tracker | During the Draft phase, the system SHALL display a section completion tracker showing which placeholders have been filled (AI-drafted, user-authored, or merged from data), which are empty, and which are overdue (if the template defines a draft-SLA per section). | UX | ALL | Should | Architects frequently lose track of incomplete sections in large documents |
| REQ-TPL-083 | Manual Override of AI Draft | A user SHALL be able to override any AI-drafted placeholder content with manual edits, with the system preserving both the original AI draft and the manual edit in version history, and flagging the section as "manually overridden" in the audit trail. | Functional | ALL | Must | User authority over final content must never be subordinated to AI drafting |

### 4.3 Create Phase Governance

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-084 | Template Completeness Validation at Create | When a content item transitions from Draft to Create (ready for review submission), the system SHALL execute a template completeness check: all Required placeholders filled, all required sections non-empty, conditional blocks correctly resolved — blocking the transition with a clear error list if validation fails. | Governance | ALL | Must | Incomplete documents submitted for review waste reviewer time |
| REQ-TPL-085 | Version Snapshot on Create | On entering the Create phase, the system SHALL capture an immutable snapshot of the content item as rendered by the current template (cross-ref V15 REQ-REV-002), tagged with the template version used. | Data | ALL | Must | Reviewers must see the version as submitted; post-create edits must not silently alter the review copy |

### 4.4 Review Phase Governance

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-086 | Review Checklist Template Auto-Assignment | When a content item enters the Review phase, the system SHALL auto-assign the review checklist template bound in the content type's lifecycle binding, presenting it to reviewers in the V15 review interface without requiring manual checklist selection. | Governance | ALL, Admin | Must | Manual checklist selection is skipped; template binding ensures the right checklist is always applied |
| REQ-TPL-087 | Review Presentation Layout | The Review phase SHALL render the content item using the template's review presentation layout: a reading-optimised view that shows section headings, placeholder values, inline change markers, and review comment anchors — not the full branded output layout — to focus reviewer attention on content. | UX | ALL | Should | Branded output layout distracts reviewers from content-level issues |

### 4.5 Present Phase Governance

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-088 | Audience-Specific Presentation Mode | The template lifecycle binding SHALL declare one or more audience presentation modes (Executive, Technical, Board, Customer, Regulatory) each linked to a distinct presentation layout template. When a user enters the Present phase, they select the audience mode, and the system renders the output using the corresponding layout. | UX | EA, SA, PM | Must | A single architecture review produces different presentations for the board, the engineering team, and the customer — template-driven, not manually assembled |
| REQ-TPL-089 | Present Phase Render Trigger | Entering the Present phase SHALL trigger a full render of the bound presentation template via V10 engine, using current data and the selected audience mode, and make the rendered output available for review, download, and sharing within the ESIP platform. | Functional | ALL | Must | Present phase must produce a shareable artifact, not just a preview |

### 4.6 Deliver Phase Governance

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-090 | Delivery Package Assembly | When a content item enters the Deliver phase, the system SHALL assemble the configured delivery package (§2.8) defined in the template lifecycle binding: render all constituent artefacts, apply naming conventions, generate the manifest, and deliver to the configured channel. | Functional | ALL, Admin | Must | Template-controlled delivery ensures completeness and consistency |
| REQ-TPL-091 | Delivery Confirmation Record | After delivery, the system SHALL record a delivery confirmation in the content item's audit trail: timestamp, delivery channel, recipient list, package contents, and the template version used for each artefact. | Governance | Admin | Must | Audit trail for delivered architecture documents is an enterprise compliance requirement |

---

## 5. Placeholder & Merge Engine

### 5.1 Merge Field Binding

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-092 | Model-Bound Placeholder Catalogue | The system SHALL maintain a runtime catalogue of all available merge fields, derived from the V11 metamodel (artifact type schemas, project metadata, portfolio data, user/reviewer metadata, framework fields), updated dynamically as the metamodel evolves, and exposed to the template designer and merge engine via a typed field-descriptor API. | Data | Admin | Must | The merge engine and the designer must share a single authoritative field source-of-truth |
| REQ-TPL-093 | Deep-Path Field References | The merge engine SHALL support dot-notation deep-path references (e.g., `project.stakeholders[0].name`, `requirement.priority.label`, `component.dependencies.count`) to navigate the model object graph, enabling fine-grained data binding without requiring flat denormalised views. | Functional | Admin | Must | Architecture models are deeply nested object graphs; shallow binding produces incomplete placeholders |
| REQ-TPL-094 | Aggregate Functions | The merge engine SHALL support aggregate function placeholders: COUNT, SUM, AVG, MIN, MAX, FIRST, LAST, CONCAT over a filtered collection (e.g., `COUNT(requirements WHERE priority = 'Must')`), enabling summary statistics in executive summaries and dashboards. | Functional | Admin | Must | Executive summary sections require aggregate statistics |
| REQ-TPL-095 | Format Expressions | The merge engine SHALL support format expressions on any field: date/time format patterns, number decimal/comma formatting, text truncation with ellipsis, text case transformation (UPPER, lower, Title), currency formatting, and percentage display — applied as a pipeline modifier on the placeholder token. | Functional | Admin | Must | Raw field values (ISO dates, decimal numbers, enum codes) are unacceptable in professional documents |
| REQ-TPL-096 | Null / Missing Field Handling | The merge engine SHALL apply a configurable null-handling policy per placeholder: (a) use fallback value; (b) render as blank; (c) render as a configurable placeholder label (e.g., "[Not provided]"); (d) fail the generation with a validation error. The policy is set in the template and the admin can make it tenant-wide default. | Functional | Admin | Must | Inconsistent null handling is the most common source of embarrassing document output |
| REQ-TPL-097 | Cross-Artifact Field References | The merge engine SHALL support placeholder bindings that reference fields from related artifacts (e.g., in a delivery package cover sheet, reference a field from the linked HLD document and from the linked requirements baseline), traversing artifact relationships in the metamodel graph. | Functional | Admin | Should | Delivery packages span multiple artifacts; cover sheets need cross-artifact data |

### 5.2 Conditional & Loop Evaluation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-098 | Condition Evaluation Engine | The merge engine SHALL evaluate conditional section expressions at render time using: comparison operators (=, ≠, <, >, ≤, ≥, IN, NOT IN, IS NULL, IS NOT NULL), logical operators (AND, OR, NOT), grouping, and references to any merge field or computed field — producing a boolean that controls section visibility. | Functional | Admin | Must | Conditional logic is core to multi-context templates |
| REQ-TPL-099 | Loop Iteration with Index | The merge engine SHALL expose the current iteration index (zero-based and one-based) as a special field within a repeater loop context (e.g., `_index`, `_number`) so templates can render numbered lists, alternating-row logic, and "first/last item" conditions without client-side logic. | Functional | Admin | Should | Numbered requirement lists and zebra-striped tables need iteration-aware fields |
| REQ-TPL-100 | Loop Collection Filter | The merge engine SHALL evaluate a filter expression on the repeater source collection at render time, allowing the template to render only a subset of items matching a condition (e.g., render only Must-priority requirements in the executive summary repeater). | Functional | Admin | Must | Long collections must be filterable per template context |
| REQ-TPL-101 | Loop Sort Order | The merge engine SHALL apply a configurable sort expression on the repeater collection (sort by field ASC/DESC, multi-level sort by multiple fields) before iterating, producing consistently ordered table/list output. | Functional | Admin | Must | Unsorted requirement tables are unacceptable in formal deliverables |
| REQ-TPL-102 | Loop Max-Items Truncation | The merge engine SHALL apply a configurable max-items limit on a repeater, rendering only the first N items and appending a configurable "…and N more items" summary row when the collection exceeds the limit. | Functional | Admin | Should | Large collections (1000+ requirements) must be summarisable in executive documents |

### 5.3 Merge Validation & Error Surfacing

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-103 | Pre-Render Merge Validation | Before invoking the V10 render engine, the merge engine SHALL execute a full pre-render validation pass: resolve all placeholder bindings, evaluate all conditions, expand all loops, apply all format expressions — and produce a structured validation report listing every error (broken reference, null required field, expression error, loop-with-no-items warning) with location (section name, placeholder name). | Functional | Admin, ALL | Must | Catching errors before render prevents costly full-document regeneration cycles |
| REQ-TPL-104 | Partial Render on Non-Fatal Errors | The merge engine SHALL support a "partial render" mode where non-fatal errors (null optional field, empty optional loop) use fallback values and continue, while fatal errors (broken required field, expression syntax error) halt the merge with a clear error report. | Functional | ALL | Should | Blocking on every null optional field is impractical for large documents with sparse data |
| REQ-TPL-105 | Merge Error Report UI | Merge validation errors SHALL be presented as an interactive report in the UI: grouped by section, with one-click navigation to the affected section in the template designer, and with suggested fixes (e.g., "Add fallback value" or "Mark as Optional"). | UX | Admin, ALL | Must | Raw error logs are actionable only by developers; architects need guided error resolution |

---

## 6. Brand & Presentation Governance

### 6.1 Brand Theme Layer

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-106 | Brand Theme Reference (Not Embed) | Templates SHALL reference brand themes by theme-ID (sourced from V11 branding infra) rather than embedding style values — enabling a single brand-update to propagate to all templates that reference the theme, without requiring template edits. | Governance | Admin | Must | Decoupled brand/template prevents manual re-styling of every template on rebrand |
| REQ-TPL-107 | Multi-Brand Support | The system SHALL support multiple concurrent brand themes in a tenant (e.g., primary corporate brand, acquired subsidiary brand, client-facing brand), with a given template or delivery package configured to use a specific theme or to dynamically select a theme based on project metadata (e.g., client ID). | Functional | Admin | Must | Enterprises with multiple brands or white-label needs require per-context theming |
| REQ-TPL-108 | Brand Override for External Presentations | The template lifecycle binding for the Present phase SHALL allow a brand-override policy: "Use project brand" / "Use org brand" / "Use client brand (if defined)" / "Use template-default brand" — giving admins control over which brand appears on externally delivered materials. | Governance | Admin | Should | Client-facing presentations must use the client's brand; internal reviews use the org brand |

### 6.2 Audience-Differentiated Presentation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-109 | Executive vs. Technical Mode | A single content template SHALL support at minimum two configurable audience modes (Executive, Technical), where the Executive mode hides deep technical sections, surfaces key messages, uses larger fonts, and shows diagrams without call-outs; and the Technical mode reveals full technical detail, code blocks, API specs, and data schemas. | UX | EA, SA, PM | Must | Generating two separate documents for the same content is a common pain point eliminated by audience modes |
| REQ-TPL-110 | Board / Governance Mode | Templates for architecture roadmaps, capability assessments, and delivery status SHALL include a configurable "Board/Governance" presentation mode that produces a 2–5 slide/page executive summary distillation of the full document, with RAG status indicators, key decisions, and next actions highlighted. | UX | EA, Admin | Should | Board-level audiences consume 2-pagers, not 40-page architecture documents |
| REQ-TPL-111 | Regulatory / Audit Mode | Templates for security and compliance-related content types SHALL support a "Regulatory/Audit" presentation mode that foregrounds evidence references, control identifiers (NIST, ISO 27001, etc.), gap statements, and audit-trail citations, while suppressing design rationale and background context. | UX | SecA, Admin | Should | Audit reviewers need a stripped, evidence-dense view |
| REQ-TPL-112 | Customer / External Mode | The template system SHALL support a "Customer/External" audience mode that: suppresses internal cost data, internal team names, and system hostnames; replaces them with configurable placeholder labels (e.g., [Internal System]); and applies the client brand theme — producing an externally safe version from the same document. | Security | SA, EA, Admin | Should | Accidental disclosure of internal data in client-facing documents is a significant risk |
| REQ-TPL-113 | Audience Mode Inheritance | Audience mode configuration SHALL participate in the four-level template hierarchy: the org admin defines which modes are available for a template type; the project lead MAY restrict or re-configure modes; the user selects from the available modes at generation time. | Governance | Admin, Proj-Lead | Must | Mode availability must be governable, not open-ended per user |

### 6.3 Multi-Language & Localisation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-114 | Template Locale Binding | Each template SHALL carry a locale setting (language + region) that controls: date/time format, number format, currency symbol, week-start day, and text direction (LTR/RTL). Multiple locale variants of a template SHALL be linkable as "locale alternatives" so the system selects the correct locale variant at generation time based on user or project locale. | Functional | Admin, ALL | Must | Global enterprises require localised deliverables; V19 i18n/l10n mandate |
| REQ-TPL-115 | Section Label Localisation | All static section labels, button labels, and boilerplate text within a template SHALL be externalised to a locale-string resource file (not embedded as literal text in the template structure), enabling translation without redesigning the template. | Functional | Admin | Must | Hard-coded English labels in templates cannot be translated |
| REQ-TPL-116 | RTL Layout Support | The template designer and merge engine SHALL support right-to-left layout (for Arabic, Hebrew, Farsi) by flipping column order, text direction, and margin/padding on template sections when the active locale is RTL. | Functional | Admin | Should | V19 global deployment requirement |

---

## 7. Adaptive & Prescriptive Suggestion Layer

### 7.1 Template Suggestion Engine

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-117 | Contextual Template Suggestion | The system SHALL suggest the most appropriate template(s) when a user initiates creation of a new content item, based on: the user's active persona, the current project type, the selected framework, and the content type being created — presenting a ranked suggestion list with rationale explanations, before showing the full template library. | AI/ML | ALL | Must | Architects should not have to browse a template library — the right template should surface contextually |
| REQ-TPL-118 | Suggestion Rationale Explainability | Each template suggestion SHALL include a visible, human-readable "Why suggested" explanation (e.g., "Suggested because: project type is Cloud Migration, your persona is Cloud Architect, and this template was used in 14 similar projects") — not a black-box recommendation. | UX | ALL | Must | Explainability of AI recommendations is a platform-wide principle (V16/V14) |
| REQ-TPL-119 | User Override of Suggestion | A user SHALL always be able to dismiss a suggested template and choose any other available template from the full library — suggestion is advisory, never mandatory. | UX | ALL | Must | "User configuration always wins" is the product owner's explicit intent |
| REQ-TPL-120 | Suggestion Signal Sources | Template suggestions SHALL be computed from: (a) persona + project-type + framework context (rule-based, configurable); (b) org-level usage analytics (most-used template for this context); (c) per-user usage history (cross-ref V14); (d) V16 prescriptive intelligence signals — with explicit weighting configuration by admin. | AI/ML | Admin | Must | Multi-signal suggestions are more accurate than single-heuristic approaches; weighting must be configurable |
| REQ-TPL-121 | Section Suggestion Within Template | After a template is selected, the system SHALL suggest optional sections that the user has not yet included, based on content type best-practices and peer-template analytics — displayed as an "Add recommended section" prompt in the draft interface, not auto-added. | AI/ML | ALL | Should | Users often omit sections they don't know about; suggestion-not-force respects user authority |
| REQ-TPL-122 | Layout Suggestion for Presentation Mode | When entering the Present phase, the system SHALL suggest an audience mode and presentation layout based on the scheduled meeting type, invitee roles (from calendar/email metadata if available cross-ref V17), and prior presentation history — user confirms or overrides. | AI/ML | SA, EA, PM | Should | Meeting-context-aware layout selection reduces manual selection friction |
| REQ-TPL-123 | Suggestion Opt-Out | A user SHALL be able to disable template suggestions entirely (always open the full library directly) as a personal preference (cross-ref V14), and an admin SHALL be able to disable suggestions tenant-wide for compliance environments where prescriptive AI behaviour is undesired. | UX | ALL, Admin | Must | Some regulated environments prohibit any AI recommendation behaviour without explicit human initiation |

### 7.2 Adaptive Section Reordering Suggestion

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-124 | Section Order Suggestion | Based on learned patterns from how reviewers engage with similar documents (time-on-section, comments-per-section), the system SHALL suggest an alternative section ordering that front-loads the sections reviewers most engage with — presented as a suggestion, not an automatic reorder. | AI/ML | Admin, ALL | Could | Reviewer-engagement-driven ordering improves document communication effectiveness |
| REQ-TPL-125 | Learned Template Personalisation (User Level) | Over time, if a user consistently rearranges, adds, or removes the same sections from a template, the system SHALL offer to save those modifications as a personal template variant at the User level (cross-ref V14 repetition-detection → automation suggestion). | AI/ML | ALL | Should | Detected repetition → suggest formalisation as a persistent personal variant; reduces manual effort |

---

## 8. Template Lifecycle Management

### 8.1 Versioning

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-126 | Template Versioning | Every template (of any type) SHALL be versioned using a semantic version number (Major.Minor.Patch), with an immutable version record created on every save of a published template, storing the complete template definition, the diff from the prior version, author, timestamp, and change description. | Data | Admin | Must | Template changes must not silently alter in-flight content items; version history enables rollback |
| REQ-TPL-127 | Template Version History Browser | The system SHALL provide a version history browser for each template, showing all versions with diffs (added/modified/removed sections, placeholder changes, condition changes) in a human-readable visual diff, with one-click restore to any prior version. | UX | Admin | Must | Authors need to understand what changed and why before restoring |
| REQ-TPL-128 | Version Pinning for Content Items | Each content item (BRD, HLD, ADR, etc.) SHALL record which template version was used for its most recent render, and SHALL allow re-rendering using either the pinned version or the latest published version — the user makes this choice explicitly at render time. | Data | ALL | Must | Regenerating a document with a newer template version must be a deliberate decision |
| REQ-TPL-129 | Breaking Change Detection | When a new template version introduces breaking changes (removes a required placeholder, removes a section that active content items rely on, changes the output format of a field), the system SHALL flag the version as a breaking change, require the author to acknowledge the impact, and present a list of content items that will be affected. | Governance | Admin | Must | Silent breaking changes invalidate existing documents |

### 8.2 Publish / Deprecate / Archive Lifecycle

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-130 | Template Status States | Each template SHALL have a lifecycle status: Draft (editable, not selectable for generation), Published (available for generation), Deprecated (selectable for existing content but not for new content), Archived (no longer selectable; read-only). Transitions between states SHALL be explicit admin actions with reason annotation. | Governance | Admin | Must | Production templates must be stable; decommissioning must be graceful |
| REQ-TPL-131 | Deprecation Migration Path | When deprecating a template, the system SHALL require the admin to designate a replacement template and SHALL display a migration prompt to all users/projects currently using the deprecated template, offering one-click migration of their existing content items to the replacement template version. | Governance | Admin | Must | Deprecated templates without migration paths leave stranded content |
| REQ-TPL-132 | Template Approval Workflow | For org-level or system-level templates, the system SHALL support a configurable template approval workflow (cross-ref V12/V15): a new template version in Draft status is submitted for review, reviewers use the V15 review module to inspect the template definition, and an approver publishes or rejects it. | Governance | Admin | Should | Production templates for regulated deliverables must pass an internal review before publication |

### 8.3 Import / Export

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-133 | Template Export to Portable Format | The system SHALL export any template (or a bundle of related templates forming a content type lifecycle set) to a portable JSON/YAML package including: template definition, placeholder bindings, lifecycle binding, brand theme references, and sample data sets — enabling transfer between tenants, orgs, or to the marketplace. | Integration | Admin | Must | Template portability is required for marketplace, multi-tenant deployment, and backup |
| REQ-TPL-134 | Template Import with Conflict Resolution | The system SHALL support importing a template package from an exported file, detecting conflicts with existing templates (same ID, overlapping placeholders), and presenting a conflict-resolution UI: skip, overwrite, rename-and-import, or merge. | Integration | Admin | Must | Blind import overwrites can destroy production templates |
| REQ-TPL-135 | Word / PowerPoint Template Import | The system SHALL support importing an existing Microsoft Word (.docx) or PowerPoint (.pptx) file as a starting point for a new ESIP template, parsing its headings, styles, tables, and content blocks into the WYSIWYG canvas, with wizard-guided placeholder mapping for detected static text patterns (e.g., a bracketed `[Project Name]` token auto-mapped to the project.name field). | Integration | Admin | Must | Most organisations have existing DOCX/PPTX templates; migration from them must be guided |
| REQ-TPL-136 | Template Bundle (Content Type Lifecycle Set) | The system SHALL support exporting and importing a "Template Bundle" — a named collection of all templates that together govern a complete content type lifecycle (document template + drafting prompt template + review checklist template + presentation template + delivery package template) as a single portable unit. | Integration | Admin | Must | Installing a new content type (e.g., "Cloud Migration HLD") should install all its lifecycle templates atomically |

### 8.4 Template Testing

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-137 | Template Test Mode | The system SHALL provide a "Test Template" action that runs the full merge engine pipeline (field binding, condition evaluation, loop expansion, format expressions) against a selected sample data set, produces a rendered output via V10, and presents the output alongside a validation report — without creating a real content item or consuming a generation credit. | Functional | Admin | Must | Template authors must be able to validate output before publishing to production |
| REQ-TPL-138 | Regression Test on Template Update | When a template is updated and re-published, the system SHALL optionally run a regression test: re-render a selected set of "golden" content items using the new template version, diff the output against the prior version's render, and flag sections where the output changed. | Governance | Admin | Should | Unintended output changes from template edits must be caught before they affect live users |
| REQ-TPL-139 | A/B Template Comparison | The system SHALL support an admin-initiated A/B comparison: select two template versions, render the same content item with both, and display a side-by-side visual diff of the output pages — without replacing the production template. | UX | Admin | Could | Evaluating whether a template redesign actually improves output requires visual comparison |

---

## 9. Template Library & Marketplace

### 9.1 Org Template Library

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-140 | Template Library Browse & Search | The system SHALL provide a template library browser with: keyword search, filter by content type, filter by persona, filter by framework, filter by author level (System/Org/Project), sort by usage frequency/last modified/name, and thumbnail preview per template. | UX | ALL, Admin | Must | Discoverability of the right template from a large library requires faceted search |
| REQ-TPL-141 | Template Favourites & Collections | A user SHALL be able to mark templates as personal favourites and organise them into named personal collections for quick access, persisted in their user profile (cross-ref V14). | UX | ALL | Should | Power users build personal template workflows; pinning reduces browse overhead |
| REQ-TPL-142 | Org Template Pinning | An admin SHALL be able to pin specific templates to the top of the library for a tenant or persona group (e.g., "Recommended by your organisation"), ensuring new users see the governance-approved starting points first. | Governance | Admin | Must | Enterprise governance requires steering users to approved templates without hard-locking choices |
| REQ-TPL-143 | Template Usage Analytics | The system SHALL provide an admin analytics dashboard showing: templates used per content type, generation counts per template version, most-used placeholders, most-frequent null fields (indicating data quality gaps), most-skipped optional sections, and per-persona template adoption rates. | Reporting | Admin | Should | Usage data is the primary input for template library curation and improvement (cross-ref V16) |
| REQ-TPL-144 | Template Quality Score | The system SHALL compute a template quality score based on: preview validation pass rate, generation error rate, time-to-completion for content items using this template, reviewer feedback signal, and usage trend — displayed in the library alongside each template. | Reporting | Admin | Could | Quality scoring helps admins prioritise template maintenance investment |

### 9.2 Marketplace Integration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-145 | ESIP Template Marketplace Browser | The system SHALL provide access to the ESIP template marketplace (cross-ref V16 INT marketplace) from within the template library, allowing users to browse, preview, and install community or vendor-published template bundles (content type lifecycle sets) with one click. | Integration | Admin, ALL | Should | Marketplace templates bootstrap new personas/frameworks without requiring every org to start from scratch |
| REQ-TPL-146 | Template Publish to Marketplace | An org admin SHALL be able to publish a template bundle to the marketplace (public or private-org scope), with metadata (description, tags, persona, framework), a preview screenshot, version notes, and a licence declaration. | Integration | Admin | Could | Successful orgs sharing templates builds ecosystem value (cross-ref V16 marketplace) |
| REQ-TPL-147 | Marketplace Template Provenance | Installed marketplace templates SHALL display their provenance (publisher, version, install date, source URL) and SHALL notify the org admin when an updated version is available from the publisher. | Governance | Admin | Should | Admins must know what externally-sourced templates are in production and whether they're current |
| REQ-TPL-148 | Marketplace Template Sandbox | Before installing a marketplace template to the production tenant, the system SHALL offer a "sandbox install" option that installs the template in a sandboxed workspace for testing and review, with a promotion-to-production action after admin sign-off. | Governance | Admin | Should | Unvetted marketplace templates must not land directly in production |

---

## 10. Security, Access & Governance

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-149 | Template RBAC | Access to template design, publish, override, and delete actions SHALL be governed by role-based access control (cross-ref V11 RBAC): system-level templates: ESIP product team only; org-level templates: tenant admin; project-level templates: project lead + admin; user-level templates: the user + admin. | Security | Admin | Must | Template tampering by unauthorised users is a governance risk |
| REQ-TPL-150 | Template Design Permission Delegation | An org admin SHALL be able to delegate template-design and template-publish permissions to named users (e.g., a "Template Curator" role) without granting full admin rights, enabling a centre-of-excellence model for template governance. | Security | Admin | Must | Large enterprises have dedicated template/brand teams that need design rights without full admin access |
| REQ-TPL-151 | Template Audit Log | All template-lifecycle events (created, edited, published, deprecated, archived, exported, imported, approval-submitted, approval-rejected, approval-approved) SHALL be written to the immutable audit log (cross-ref V11) with user, timestamp, template ID, version, and a diff summary. | Security | Admin | Must | Immutable audit trail is an enterprise compliance requirement |
| REQ-TPL-152 | Sensitive Field Masking in Templates | The template designer SHALL allow marking specific placeholders as "sensitive" (e.g., cost data, personal data, security credentials), which triggers automatic masking/redaction in the rendered output for audience modes designated as "External" or "Customer-facing" in the template lifecycle binding. | Security | Admin | Must | Preventing inadvertent disclosure of sensitive data in external deliverables |
| REQ-TPL-153 | DLP Integration for Template Content | When a template includes a drafting prompt template or a placeholder binding to an AI-generated section, the system SHALL apply the tenant's configured Data Loss Prevention policy (cross-ref V11/V19) before the data is passed to the AI engine, ensuring sensitive project data is not transmitted to AI providers in violation of the tenant's data policies. | Security | Admin | Must | Cross-ref V19 trust/security; enterprise deal-closer requirement |
| REQ-TPL-154 | Template Content Classification | Admins SHALL be able to assign a data-classification label (Public, Internal, Confidential, Restricted) to a template, and the system SHALL propagate that classification to all content items generated from the template as a default classification, reviewable by the author. | Governance | Admin | Should | Data-classification propagation from template reduces manual labelling overhead |

---

## 11. NFRs

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-TPL-155 | Live Preview Latency | The WYSIWYG live preview SHALL update within 2 seconds of a design change for templates with up to 50 sections and 200 placeholders on a standard broadband connection. | NFR | Admin, Proj-Lead | Must | Slow preview breaks the "design and see immediately" feedback loop |
| REQ-TPL-156 | Merge Engine Throughput | The merge engine SHALL complete the pre-render merge validation pass for a template with 500 placeholders and 50 loop-iterations (25,000 total field evaluations) in under 5 seconds on the server, excluding V10 render time. | NFR | Admin, ALL | Must | Large architecture documents (RTMs, full ADM deliverables) have high field volumes |
| REQ-TPL-157 | Template Library Scale | The template library SHALL support at least 10,000 templates per tenant without performance degradation in browse and search operations (< 1 s response for filtered library queries). | NFR | Admin | Should | Large enterprise tenants accumulate templates across hundreds of project types and personas |
| REQ-TPL-158 | Template Hierarchy Computation Latency | Effective-template computation (four-level hierarchy merge for a given user/project/type) SHALL complete in under 200 ms, cached per session with invalidation on any hierarchy-level update. | NFR | ALL | Must | Every content-creation action triggers effective-template computation; latency compounds |
| REQ-TPL-159 | Concurrent Template Editing | The template designer SHALL support optimistic-locking concurrent editing: if two admins open the same template simultaneously, the second admin is warned of the concurrent edit, and on save, the system performs a three-way merge (base, edit-A, edit-B) with conflict highlighting for manual resolution. | NFR | Admin | Should | Template curation teams work concurrently; data loss from blind overwrite is unacceptable |
| REQ-TPL-160 | Template Storage Efficiency | The versioning system SHALL store only diffs between template versions (not full copies), with full-version reconstruction on demand, to keep storage growth proportional to the number of changes rather than the number of versions times template size. | NFR | Admin | Should | Template libraries with frequent updates produce large version histories; diff-storage is essential |
| REQ-TPL-161 | Accessibility of Template Designer | The WYSIWYG template designer UI SHALL meet WCAG 2.1 Level AA accessibility requirements, including keyboard-only navigation, screen-reader labelling of all canvas elements, and sufficient colour contrast in the designer chrome. | NFR | Admin | Must | V19 WCAG 2.1 AA mandate; template authors with accessibility needs must be supported |
| REQ-TPL-162 | Air-Gapped / Offline Template Library | In air-gapped deployment mode (cross-ref V19), the full system-default and org-level template library SHALL be available offline; only marketplace browsing and cross-tenant template sharing SHALL be disabled. | NFR | Admin | Must | Air-gapped deployments must not lose core template functionality |
| REQ-TPL-163 | API Access to Template Engine | The merge engine and template lifecycle operations (fetch effective template, run merge validation, trigger render, list template versions) SHALL be exposed via the ESIP public REST/GraphQL API (cross-ref V11 Integration Platform) to enable external integrations and automation. | Integration | Admin | Should | Headless template invocation from CI/CD pipelines and external tools is a platform extensibility requirement |

---

## 12. Requirements Summary Table

| Capability Area | Req IDs | Count |
|---|---|---|
| Visual Template Designer — Canvas & Layout | REQ-TPL-001 – 010 | 10 |
| Visual Template Designer — Placeholder Insertion | REQ-TPL-011 – 017 | 7 |
| Visual Template Designer — Conditional Sections | REQ-TPL-018 – 021 | 4 |
| Visual Template Designer — Repeaters & Loops | REQ-TPL-022 – 025 | 4 |
| Visual Template Designer — Styling & Brand Binding | REQ-TPL-026 – 029 | 4 |
| Visual Template Designer — Live Preview | REQ-TPL-030 – 033 | 4 |
| Visual Template Designer — Placeholder Validation | REQ-TPL-034 – 037 | 4 |
| Template Types — Document Deliverables | REQ-TPL-038 – 045 | 8 |
| Template Types — Proposals & RFP | REQ-TPL-046 – 048 | 3 |
| Template Types — ADR & Trade-Off | REQ-TPL-049 – 050 | 2 |
| Template Types — Diagram & Diagram-Frame | REQ-TPL-051 – 053 | 3 |
| Template Types — Review Checklists | REQ-TPL-054 – 056 | 3 |
| Template Types — Email & Customer Reply | REQ-TPL-057 – 058 | 2 |
| Template Types — Drafting Prompt Templates | REQ-TPL-059 – 062 | 4 |
| Template Types — Delivery Packaging | REQ-TPL-063 – 065 | 3 |
| Multi-Level Hierarchy — Hierarchy Definition | REQ-TPL-066 – 072 | 7 |
| Multi-Level Hierarchy — Override Conflict Resolution | REQ-TPL-073 – 076 | 4 |
| Content Lifecycle — Template as Lifecycle Binding | REQ-TPL-077 – 080 | 4 |
| Content Lifecycle — Draft Phase | REQ-TPL-081 – 083 | 3 |
| Content Lifecycle — Create Phase | REQ-TPL-084 – 085 | 2 |
| Content Lifecycle — Review Phase | REQ-TPL-086 – 087 | 2 |
| Content Lifecycle — Present Phase | REQ-TPL-088 – 089 | 2 |
| Content Lifecycle — Deliver Phase | REQ-TPL-090 – 091 | 2 |
| Placeholder & Merge Engine — Field Binding | REQ-TPL-092 – 097 | 6 |
| Placeholder & Merge Engine — Conditional & Loop | REQ-TPL-098 – 102 | 5 |
| Placeholder & Merge Engine — Validation & Errors | REQ-TPL-103 – 105 | 3 |
| Brand & Presentation — Brand Theme Layer | REQ-TPL-106 – 108 | 3 |
| Brand & Presentation — Audience Differentiation | REQ-TPL-109 – 113 | 5 |
| Brand & Presentation — Multi-Language | REQ-TPL-114 – 116 | 3 |
| Adaptive & Prescriptive Suggestion | REQ-TPL-117 – 125 | 9 |
| Template Lifecycle — Versioning | REQ-TPL-126 – 129 | 4 |
| Template Lifecycle — Publish/Deprecate/Archive | REQ-TPL-130 – 132 | 3 |
| Template Lifecycle — Import/Export | REQ-TPL-133 – 136 | 4 |
| Template Lifecycle — Testing | REQ-TPL-137 – 139 | 3 |
| Template Library — Org Library | REQ-TPL-140 – 144 | 5 |
| Template Library — Marketplace | REQ-TPL-145 – 148 | 4 |
| Security, Access & Governance | REQ-TPL-149 – 154 | 6 |
| NFRs | REQ-TPL-155 – 163 | 9 |
| **TOTAL** | **REQ-TPL-001 – REQ-TPL-163** | **163** |

---

## Boundary Cross-Reference Index

| Cross-referenced Volume | What V23 hands off / depends on |
|---|---|
| **V10 Deliverables Generation** | V23 defines template design and lifecycle; V10 executes the byte-level DOCX/PPTX/XLSX/PDF/HTML render from V23 template definitions. V23 instructs V10 via a structured render-request payload (effective template + merged field data). |
| **V11 Platform / Metamodel** | V23 reads the metamodel field catalogue (REQ-TPL-092), uses V11 brand infra (REQ-TPL-106), persists template records in V11 metamodel storage, uses V11 RBAC for access control (REQ-TPL-149), and writes to V11 audit log (REQ-TPL-151). |
| **V06 AI Capabilities** | V23 owns drafting prompt templates (REQ-TPL-059–062); V06 executes the AI inference. V23 passes the prompt template + bound field context to V06; V06 returns drafted text that lands in V23-defined placeholders. |
| **V14 Personalisation** | V23 adaptive suggestions (§7) consume V14 user preference signals. V23 user-level template overrides (REQ-TPL-072, 125) are stored in the V14 user preference model. |
| **V15 Review** | V23 template lifecycle binding (REQ-TPL-086) assigns review checklist templates; V15 executes the review using the V23-defined checklist template. V23 also supports template approval via V15 review (REQ-TPL-132). |
| **V16 Intelligence** | V23 template suggestion engine (§7.1) consumes V16 prescriptive analytics signals. V23 supplies template usage analytics (REQ-TPL-143) as an input feed to V16. |
| **V12 Collaboration/Workflow** | V23 lifecycle binding configures which approval workflow is triggered at each phase gate (REQ-TPL-078, 080); V12 executes the workflow. |
| **V22 Proposal** (future) | V23 owns proposal/RFP template design (REQ-TPL-046–048); V22 owns the proposal authoring UX and workflow. |
| **V21 ADR** (future) | V23 owns ADR/trade-off template design (REQ-TPL-049–050); V21 owns the ADR decision record UX. |
| **V19 Enterprise Trust** | V23 satisfies WCAG 2.1 AA for designer UI (REQ-TPL-161), DLP integration (REQ-TPL-153), air-gapped operation (REQ-TPL-162), and i18n/RTL (REQ-TPL-114–116). |
