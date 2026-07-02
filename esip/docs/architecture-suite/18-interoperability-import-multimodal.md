# Volume 18 — Interoperability, Import & Multimodal Capture

**Document ID:** ESIP-VOL-18  
**Version:** 1.0.0  
**Date:** 2026-06-23  
**Status:** Draft — Authoritative  
**Scope:** Comprehensive import/interoperability mechanics, computer-vision whiteboard/sketch capture pipeline, and mobile companion app for the ESIP Architecture Automation Suite.

---

## Scope Statement

This volume owns the **mechanics** of bringing external content into ESIP: file-based import, API-based migration, visual capture from photos, and the mobile app that acts as the primary capture surface for field/meeting use. It does **not** own: notation rendering or canvas drawing (V03 — maxGraph engine); CV/OCR/LLM model hosting and training infrastructure (V06); live enterprise connector protocols (V08); identity, MDM, and cloud sync infrastructure (V11); mobile review UX and approval workflows (V15); or competitive migration accelerator intelligence/analytics (V16). Cross-references to those volumes are noted inline.

---

## Cross-Volume Reference Map

| Concern | This Volume Owns | Delegated To |
|---|---|---|
| maxGraph canvas rendering of imported shapes | Import shape→metamodel mapping | V03 |
| CV/OCR model hosting, GPU infra, model versioning | CV pipeline API contract, confidence scoring | V06 |
| OAuth/OIDC token broker, SSO federation | Consuming auth tokens in mobile | V11 |
| Live Jira/Confluence/ADO connector sync | Import from exported CSV/XMI dumps | V08 |
| Competitive migration accelerator rules | Import mechanics for each source format | V16 |
| Mobile review workflow, comment/approval UX | Photo capture, voice capture, sync handoff | V15 |

---

## Personas

| ID | Persona | Description |
|---|---|---|
| P-EA | Enterprise Architect | Imports existing EA models; maps to ESIP metamodel |
| P-SA | Solution Architect | Imports Visio/draw.io diagrams; migrates team wikis |
| P-BA | Business Analyst | Imports BPMN processes; maps requirements from Excel |
| P-PM | Product Manager | Captures whiteboard sessions; quick-captures meeting notes |
| P-TA | Tenant Admin | Configures permitted import sources; monitors import jobs |
| P-DEV | Developer | Uses import APIs programmatically; embeds in CI/CD |
| P-MOB | Mobile User | Any of above using the mobile companion app |
| P-FIELD | Field Architect | Captures photos of physical whiteboards at client sites |

---

## Requirement Priority Legend

| Symbol | MoSCoW |
|---|---|
| M | Must Have |
| S | Should Have |
| C | Could Have |
| W | Won't Have (this release) |

---

## REQ-IMP — Comprehensive Import / Interoperability

> **Reconciliation note (ownership — diagram/model import):** **V18 is CANONICAL for ALL diagram and model import** across every source format (draw.io/Visio/Sparx EA/AMEFF/Mermaid/PlantUML/Lucidchart/Gliffy/Miro/BPMN/XMI + LeanIX/Ardoq/MEGA/Avolution/BiZZdesign/erwin), including the import wizard, format detection, metamodel-mapping engine, conflict resolution, provenance, and round-trip export of imported content. **V03 does NOT implement its own import path** — V03's former import requirements (REQ-MOD-320–327) are superseded and **defer to this V18 pipeline**, retaining only "accept parsed elements handed off from the V18 pipeline" plus V03's own export-format rendering. This is the **reciprocal of the note in V03**, which points its import requirements here. V03 still owns maxGraph canvas rendering/layout of the imported elements (see the per-requirement V03 cross-references below). Resolves the FINALIZATION-REPORT "MERGE diagram-import" action.

### REQ-IMP-001 — Import Subsystem Architecture

| Field | Value |
|---|---|
| **ID** | REQ-IMP-001 |
| **Title** | Pluggable Import Subsystem Architecture |
| **Category** | Functional |
| **Persona(s)** | P-EA, P-TA, P-DEV |
| **Priority** | M |
| **Rationale** | ESIP must accept content from dozens of source formats without tightly coupling import logic to the core metamodel. A plugin-per-format architecture ensures maintainability and extensibility. |

**SHALL Description:** The system SHALL implement import as a pluggable pipeline of discrete, independently-deployable format adapters. Each adapter SHALL: (a) declare the MIME types and file extensions it handles; (b) expose a standardized `ImportAdapter` interface (`detect`, `parse`, `mapToMetamodel`, `validate`, `report`); (c) be versioned independently; (d) be togglable per-tenant by the Tenant Admin without a platform deployment. The pipeline orchestrator SHALL chain: Upload → Format Detection → Adapter Selection → Parse → Metamodel Mapping → Conflict Resolution → Validation → Commit/Rollback → Provenance Record.

**Acceptance Criteria:**
- New adapter can be added without modifying core engine code
- Adapter toggle per-tenant reflected within 30 s of config change
- Pipeline stages independently observable via structured logs
- Adapter health check endpoint returns format coverage matrix

---

### REQ-IMP-002 — Import Wizard UX Shell

| Field | Value |
|---|---|
| **ID** | REQ-IMP-002 |
| **Title** | Multi-Step Import Wizard |
| **Category** | UX |
| **Persona(s)** | P-EA, P-SA, P-BA, P-PM |
| **Priority** | M |
| **Rationale** | Import is a high-risk operation (overwriting existing model content). A guided, reviewable wizard with preview and explicit mapping confirmation prevents data loss and builds trust. |

**SHALL Description:** The system SHALL provide a multi-step import wizard accessible from any workspace/project context. Steps SHALL be: (1) **Source Selection** — upload file(s) OR paste URL/API key for SaaS sources; (2) **Format Detection** — auto-detect format with confidence indicator; allow manual override; (3) **Preview** — render a read-only preview of parsed elements (list + visual sample) before any write; (4) **Metamodel Mapping** — display the source-element→ESIP-metamodel mapping with ability to remap, exclude, or add custom attribute mappings; (5) **Merge Strategy** — select New Workspace / Merge Into Existing / Overlay Only New Elements; (6) **Conflict Review** — show element-level conflicts (duplicate IDs, name clashes, type mismatches) with resolution choices; (7) **Validation** — run pre-commit validation rules; show errors/warnings inline; (8) **Import Report** — summary of created/updated/skipped/failed elements; downloadable as PDF/CSV. Wizard state SHALL be persisted so the user can pause and resume within 24 hours.

**Acceptance Criteria:**
- All 8 steps render correctly with React/Radix/Tailwind UI components
- Navigation allows backward traversal without data loss
- Wizard state persisted in backend (not just browser storage)
- Import Report downloadable within 5 s of commit

---

### REQ-IMP-003 — Automatic Format Detection

| Field | Value |
|---|---|
| **ID** | REQ-IMP-003 |
| **Title** | File Format Auto-Detection |
| **Category** | Functional |
| **Persona(s)** | P-EA, P-SA, P-BA |
| **Priority** | M |
| **Rationale** | Users often don't know or don't state file formats precisely. Reliable auto-detection reduces friction. |

**SHALL Description:** The system SHALL detect import file format by: (a) file extension match; (b) magic-byte / MIME sniffing for ZIP-based formats (.vsdx, .eapx, .drawio); (c) XML root-element namespace detection (BPMN, ArchiMate AMEFF, XMI/UML, PlantUML, draw.io); (d) content-signature heuristics for Mermaid (leading `graph`/`flowchart`/`sequenceDiagram` keywords). Detection SHALL return a ranked list of candidate formats with confidence scores. When confidence < 0.8, the wizard SHALL prompt the user to confirm format before proceeding. Detection SHALL complete within 2 s for files ≤ 50 MB.

**Acceptance Criteria:**
- Correct format detected for all formats listed in REQ-IMP-010 through REQ-IMP-030 on a test corpus of ≥ 10 samples per format
- Confidence score displayed in UI
- Manual override always available regardless of confidence

---

### REQ-IMP-004 — Large File Handling

| Field | Value |
|---|---|
| **ID** | REQ-IMP-004 |
| **Title** | Large File & Batch Import Handling |
| **Category** | NFR |
| **Persona(s)** | P-EA, P-TA, P-DEV |
| **Priority** | M |
| **Rationale** | Enterprise EA repositories can be hundreds of megabytes. The import pipeline must not block the UI or time out on large files. |

**SHALL Description:** The system SHALL: (a) support file upload up to 500 MB per file via multipart/chunked upload with resumable upload protocol (TUS or equivalent); (b) process import jobs asynchronously — the wizard moves to a "Processing" state with a live progress indicator once the file is submitted; (c) notify the user via in-app notification and optionally email when the async job completes; (d) support batch import of up to 50 files in a single import session, processed in parallel up to a configurable concurrency limit (default 5); (e) enforce per-tenant quotas on import file size and monthly import volume (configurable by Tenant Admin); (f) stream preview data progressively — show first 500 elements within 10 s even for large files by parsing incrementally.

**Acceptance Criteria:**
- 200 MB .vsdx import completes within 120 s on reference hardware
- Progress indicator updates at least every 5 s during async processing
- Failed large-file upload can be resumed from last chunk
- Batch of 10 × 5 MB files imports without UI freeze

**Dependencies:** REQ-IMP-001 (pluggable pipeline); V11 (cloud storage for chunked uploads)

---

### REQ-IMP-005 — Metamodel Mapping Engine

| Field | Value |
|---|---|
| **ID** | REQ-IMP-005 |
| **Title** | Source-Element-to-ESIP-Metamodel Mapping |
| **Category** | Functional |
| **Persona(s)** | P-EA, P-SA |
| **Priority** | M |
| **Rationale** | Every source format has its own element vocabulary. The mapping engine bridges source semantics to ESIP's canonical metamodel (ArchiMate-aligned) so imported elements are first-class model citizens. |

**SHALL Description:** The system SHALL maintain a **Mapping Registry** — a versioned library of default source→metamodel mappings, one per adapter. Each mapping entry SHALL declare: source element type (string), target ESIP metamodel type (enum), attribute mappings (source attribute → target attribute with optional transform function), and confidence (auto-mapped / suggested / user-confirmed). The wizard's **Metamodel Mapping** step (REQ-IMP-002 step 4) SHALL render this registry as an editable table. Users SHALL be able to: (a) accept all auto-mapped entries; (b) remap individual entries via dropdown of valid ESIP types; (c) mark elements as "ignore" (exclude from import); (d) save the customized mapping as a reusable **Migration Template** scoped to the source format + tenant. Migration templates SHALL be versioned, shareable, and importable/exportable as JSON.

**Acceptance Criteria:**
- Default mapping covers ≥ 90% of elements for each supported format
- User-customized mapping persisted and re-used on next import of same format
- Migration template JSON round-trips (export → import → same behavior)

**Cross-reference:** V16 owns competitive migration accelerator intelligence that uses these templates.

---

### REQ-IMP-006 — Conflict Detection and Deduplication

| Field | Value |
|---|---|
| **ID** | REQ-IMP-006 |
| **Title** | Merge Conflict Detection and Deduplication |
| **Category** | Functional |
| **Persona(s)** | P-EA, P-SA, P-TA |
| **Priority** | M |
| **Rationale** | When importing into an existing workspace, element collisions must be surfaced and resolved, not silently overwritten. |

**SHALL Description:** The system SHALL detect merge conflicts by comparing incoming elements against existing workspace elements on: (a) stable ID match (source tool's internal ID preserved as provenance attribute); (b) name + type match within same parent container; (c) configurable fuzzy-name match (Levenshtein distance threshold, default ≤ 2). For each conflict, the system SHALL present three resolution options: **Keep Existing**, **Replace with Imported**, **Create as New (rename)**. The system SHALL also deduplicate structurally identical relationships (same source ID + target ID + relationship type). Bulk-resolve SHALL allow applying a single choice to all conflicts of the same type. Conflict resolution decisions SHALL be logged in the Import Report.

**Acceptance Criteria:**
- No silent overwrite of existing elements without user confirmation
- Bulk-resolve reduces conflict clicks by ≥ 80% on a 50-conflict scenario
- Import Report records resolution choice for each conflict

---

### REQ-IMP-007 — Provenance Tracking

| Field | Value |
|---|---|
| **ID** | REQ-IMP-007 |
| **Title** | Import Provenance and Traceability |
| **Category** | Data |
| **Persona(s)** | P-EA, P-TA |
| **Priority** | M |
| **Rationale** | Architects must know which elements came from which external source and when, for audit, rollback, and lineage queries. |

**SHALL Description:** Every element created or updated via import SHALL carry a `provenance` block recording: source format, source file name, source file hash (SHA-256), source tool element ID, import job ID, import timestamp (UTC ISO-8601), importing user ID, and mapping template version used. The provenance block SHALL be queryable via ESIP's search/filter APIs. The system SHALL provide an **Import History** view per workspace listing all past import jobs with status, element counts, and a rollback option (within configurable retention window, default 30 days). Rollback SHALL revert all elements created or modified by that import job in a single atomic transaction.

**Acceptance Criteria:**
- Provenance fields visible in element detail panel
- Import History lists last 100 jobs per workspace
- Rollback of a 500-element import completes within 30 s

---

### REQ-IMP-008 — Import REST/GraphQL API

| Field | Value |
|---|---|
| **ID** | REQ-IMP-008 |
| **Title** | Programmatic Import API |
| **Category** | Integration |
| **Persona(s)** | P-DEV, P-TA |
| **Priority** | M |
| **Rationale** | CI/CD pipelines and migration tooling need headless import without the wizard UI. |

**SHALL Description:** The system SHALL expose a REST API (and optionally GraphQL mutation) for headless import: `POST /api/v1/import` accepting multipart file upload + JSON body with: target workspace ID, format hint (optional), merge strategy, mapping template ID (optional), dry-run flag. Response SHALL include job ID. `GET /api/v1/import/{jobId}/status` SHALL return live status + partial progress metrics. `GET /api/v1/import/{jobId}/report` SHALL return the full import report as JSON. The API SHALL require authentication (Bearer token, API key, or service account). Dry-run mode SHALL return the full conflict + mapping preview without committing any changes.

**Acceptance Criteria:**
- Dry-run returns mapping + conflict report without writing to DB
- API documented in OpenAPI 3.x spec
- SDK wrappers generated for TypeScript and Python

---

### REQ-IMP-009 — Error Handling and Recovery

| Field | Value |
|---|---|
| **ID** | REQ-IMP-009 |
| **Title** | Import Error Handling and Partial Recovery |
| **Category** | NFR |
| **Persona(s)** | P-EA, P-SA |
| **Priority** | M |
| **Rationale** | Malformed source files or partially corrupt exports are common. The import should not fail entirely on one bad element. |

**SHALL Description:** The import pipeline SHALL use **partial import with error isolation**: a parse or mapping error on one element SHALL not abort the entire import. Failed elements SHALL be flagged in the Import Report with error type, source element reference, and suggested remediation. The system SHALL distinguish: (a) **Warnings** — imported with degraded fidelity (e.g., unsupported shape type mapped to generic); (b) **Errors** — element skipped, workspace unchanged for that element; (c) **Fatal** — entire file unparseable (corrupt ZIP, invalid XML namespace). Fatal errors SHALL surface a user-friendly message with a diagnostic code and suggestion (e.g., "Try exporting as XMI 2.1 from your EA tool"). All error detail SHALL be available in the downloadable Import Report.

**Acceptance Criteria:**
- Import of a file with 5% corrupt elements imports remaining 95% successfully
- Error report categorizes all failure modes
- No unhandled exceptions surface to the UI (all caught, logged, reported)

---

### REQ-IMP-010 — draw.io / diagrams.net Import

| Field | Value |
|---|---|
| **ID** | REQ-IMP-010 |
| **Title** | draw.io / diagrams.net (.drawio / .xml) Import |
| **Category** | Functional |
| **Persona(s)** | P-SA, P-BA, P-PM |
| **Priority** | M |
| **Rationale** | draw.io is the most widely used free diagramming tool; teams maintain large repositories of architecture and process diagrams in this format. |

**SHALL Description:** The system SHALL parse draw.io XML (both `.drawio` and the `.xml` variant) and extract: all `mxCell` nodes (vertices and edges), labels, shapes (`style` attribute parsed for shape type), connector routing points, groups/layers, embedded images. The adapter SHALL map draw.io shape styles to ESIP/ArchiMate element types using the default mapping registry (REQ-IMP-005) and the [draw.io shape library taxonomy](https://www.diagrams.net/doc/faq/). Page/tab structure SHALL map to ESIP diagram tabs. Position and size (`mxGeometry`) SHALL be preserved for layout fidelity. Multi-page `.drawio` files SHALL import each page as a separate diagram linked under a parent container. The adapter SHALL handle compressed draw.io XML (base64-deflate encoded `value` attributes).

**Acceptance Criteria:**
- All shape types in draw.io general/network/flowchart libraries parsed without error
- Multi-page file imports all pages
- Connector routing preserved within ±5 px of source geometry
- Compressed XML decoded correctly

**Cross-reference:** V03 owns rendering of the imported shapes on the maxGraph canvas.

---

### REQ-IMP-011 — Microsoft Visio Import (.vsdx / .vdx)

| Field | Value |
|---|---|
| **ID** | REQ-IMP-011 |
| **Title** | Microsoft Visio (.vsdx / .vdx) Import |
| **Category** | Functional |
| **Persona(s)** | P-EA, P-SA |
| **Priority** | M |
| **Rationale** | Visio is the dominant enterprise diagramming tool; every large organization has thousands of .vsdx files representing existing architecture artifacts. |

**SHALL Description:** The system SHALL parse `.vsdx` (Open XML ZIP) by extracting `visio/pages/*.xml`, `visio/masters/*.xml`, and `visio/media/*`. It SHALL extract: all Shape elements (ID, name, ShapeSheet `Prop.*` custom properties, master reference), connector elements (Begin/End cell references), text labels, layers, groups, page size/scale metadata, and hyperlinks. Master shape → ESIP element type mapping SHALL be configurable via the mapping registry. The adapter SHALL also accept legacy `.vdx` (Visio XML) by parsing its flat XML schema. Shape geometry (PinX, PinY, Width, Height, Angle) SHALL be preserved. Custom shape properties SHALL be mapped to ESIP element custom attributes. The adapter SHALL handle Visio stencil references (masters) gracefully when master files are embedded vs. linked externally.

**Acceptance Criteria:**
- .vsdx imported without requiring Visio license on the server
- Custom Prop.* fields appear as custom attributes on imported elements
- Multi-page document: each page → separate diagram
- Connector connectivity (source/target references) preserved

---

### REQ-IMP-012 — Sparx Enterprise Architect Import (.eap / .eapx / .qea + XMI)

| Field | Value |
|---|---|
| **ID** | REQ-IMP-012 |
| **Title** | Sparx Enterprise Architect Import |
| **Category** | Functional |
| **Persona(s)** | P-EA |
| **Priority** | M |
| **Rationale** | Sparx EA is the dominant enterprise modeling platform; many organizations have years of UML/ArchiMate work in .eap or .qea repositories that they want to migrate or federate into ESIP. |

**SHALL Description:** The system SHALL support three EA import paths: (a) **XMI 2.x export** — accept EA-exported XMI (UML:Model root) including Sparx EA's proprietary XMI extensions (`EAStub`, `EAPackage`, `EA:Diagram`); extract packages, classes, interfaces, components, associations, dependencies, use cases, activities, sequence diagram lifelines/messages, and all tagged values; (b) **QEAX/EAPX file** — these are SQLite databases; the adapter SHALL open them as SQLite, query the canonical EA tables (`t_object`, `t_connector`, `t_attribute`, `t_operation`, `t_diagram`, `t_diagramobjects`) and reconstruct the model tree; (c) **EA CSV export** — accept element/connector CSV reports exported from EA. Tagged values SHALL map to ESIP custom attributes. Package hierarchy SHALL map to ESIP project/folder structure. UML stereotypes SHALL map to ESIP element types via the mapping registry (e.g., `<<ApplicationComponent>>` → ArchiMate Application Component). Diagrams within EA SHALL reconstruct as ESIP diagrams with best-effort layout from `t_diagramobjects` X/Y coordinates.

**Acceptance Criteria:**
- XMI import preserves package hierarchy depth ≥ 10 levels
- Tagged values appear as custom attributes
- SQLite path does not require Sparx EA to be installed on the server
- Sparx proprietary XMI extensions parsed without fatal error

---

### REQ-IMP-013 — ArchiMate Open Exchange File (AMEFF) Import

| Field | Value |
|---|---|
| **ID** | REQ-IMP-013 |
| **Title** | ArchiMate Open Exchange File Format (AMEFF) Import |
| **Category** | Functional |
| **Persona(s)** | P-EA |
| **Priority** | M |
| **Rationale** | AMEFF is the standard interchange format for ArchiMate tools; any compliant tool can export it, making it the highest-fidelity migration path between EA tools. |

**SHALL Description:** The system SHALL parse AMEFF files (XML conforming to the ArchiMate Exchange File Format schema v3.x, namespace `http://www.opengroup.org/xsd/archimate/3.0/`). The adapter SHALL extract: all `element` nodes (with `xsi:type` → ArchiMate element type), all `relationship` nodes, `view` nodes (diagrams) including `node`/`connection` layout data, property definitions, and `organization` (folder structure). ESIP's native metamodel IS ArchiMate-aligned, so AMEFF→ESIP mapping SHALL be 1:1 for all standard ArchiMate 3.x element and relationship types. The system SHALL validate the imported file against the official AMEFF XSD before parsing. AMEFF export from ESIP SHALL also be supported (see REQ-IMP-050). The adapter SHALL flag any elements using non-standard xsi:type values and map them to the closest ESIP type with a warning.

**Acceptance Criteria:**
- All 54 ArchiMate 3.1 element types imported correctly
- All 16 ArchiMate 3.1 relationship types imported correctly
- View layout (node x/y/w/h) preserved
- Round-trip: ESIP AMEFF export → AMEFF import → element count unchanged

---

### REQ-IMP-014 — Mermaid Import

| Field | Value |
|---|---|
| **ID** | REQ-IMP-014 |
| **Title** | Mermaid Diagram Import |
| **Category** | Functional |
| **Persona(s)** | P-SA, P-DEV, P-BA |
| **Priority** | M |
| **Rationale** | Mermaid is ubiquitous in developer documentation (README files, Confluence, Notion). Teams have significant diagram libraries in this text-based format. |

**SHALL Description:** The system SHALL accept Mermaid text (`.mmd` or `.md` files, or pasted text in the wizard). The adapter SHALL parse the following Mermaid diagram types: `flowchart`/`graph` (LR/TD/RL/BT), `sequenceDiagram`, `classDiagram`, `erDiagram`, `stateDiagram`, `gantt`, `C4Context`/`C4Container`/`C4Component`. For each type, nodes/participants/entities/states SHALL map to ESIP element types via the mapping registry. Relationships/arrows/messages SHALL map to ESIP relationship types. `gantt` charts SHALL import as ESIP roadmap milestones/phases. Labels SHALL be preserved. The adapter SHALL handle Mermaid 10.x syntax. Subgraphs SHALL map to ESIP groups/containers. Notes and `%%` comments SHALL be preserved as ESIP element notes/annotations.

**Acceptance Criteria:**
- All 8 listed diagram types parsed without fatal error
- C4Context elements mapped to ArchiMate Person/System/Container types
- gantt import creates corresponding roadmap items
- `%%` comments preserved as annotations

---

### REQ-IMP-015 — PlantUML Import

| Field | Value |
|---|---|
| **ID** | REQ-IMP-015 |
| **Title** | PlantUML Diagram Import |
| **Category** | Functional |
| **Persona(s)** | P-SA, P-DEV, P-BA |
| **Priority** | S |
| **Rationale** | PlantUML is widely used in enterprise Java/Spring ecosystems for UML diagrams embedded in code comments and wikis. |

**SHALL Description:** The system SHALL parse PlantUML `.puml`/`.plantuml` files. Supported diagram types: `@startuml`/`@enduml` blocks for: class diagrams, sequence diagrams, component diagrams, deployment diagrams, activity diagrams (new syntax), use-case diagrams, and C4-PlantUML extensions. Elements (classes, components, nodes, actors) SHALL map to ESIP/ArchiMate types. Relationships (extends, implements, uses, associations) SHALL map to ESIP relationship types. Stereotypes SHALL map to ESIP element types or custom attributes. `skinparam` and style directives SHALL be captured as metadata but need not control rendering (V03 owns rendering). The adapter SHALL handle `!include` directives by resolving included files if present in the same upload batch.

**Acceptance Criteria:**
- Class and sequence diagram import without fatal error
- C4-PlantUML elements map to ArchiMate equivalents
- `!include` resolved when referenced file included in batch upload

---

### REQ-IMP-016 — Lucidchart Import

| Field | Value |
|---|---|
| **ID** | REQ-IMP-016 |
| **Title** | Lucidchart Import |
| **Category** | Functional |
| **Persona(s)** | P-SA, P-BA, P-PM |
| **Priority** | S |
| **Rationale** | Lucidchart is widely used in product and BA teams; export formats include Lucidchart's native JSON and Visio-compatible .vsdx. |

**SHALL Description:** The system SHALL import Lucidchart via two paths: (a) **Lucidchart JSON export** (`.lucid` or `.json`) — parse shapes, connectors, pages, groups, and custom shape libraries; map to ESIP metamodel; (b) **Visio .vsdx export from Lucidchart** — handled by REQ-IMP-011. For the native JSON path, the adapter SHALL extract: shape type, label, position, size, connector endpoints, page/layer structure, and custom data fields. Lucidchart shape type identifiers SHALL be mapped via the registry to ESIP element types. The wizard SHALL present Lucidchart-specific migration guidance noting common shape-type mapping caveats.

**Acceptance Criteria:**
- Native JSON import extracts all shapes and connectors
- Custom data fields appear as ESIP custom attributes
- Lucidchart-to-ESIP mapping table shown in wizard

---

### REQ-IMP-017 — Gliffy Import

| Field | Value |
|---|---|
| **ID** | REQ-IMP-017 |
| **Title** | Gliffy Diagram Import |
| **Category** | Functional |
| **Persona(s)** | P-SA, P-BA |
| **Priority** | C |
| **Rationale** | Gliffy is embedded in Confluence; many Atlassian-heavy organizations have diagram libraries in Gliffy format. |

**SHALL Description:** The system SHALL parse Gliffy `.gliffy` files (JSON format). The adapter SHALL extract: objects (uid, type, graphic, geometry, text), constraints (connector source/target linkage), groups, and layers. Gliffy `graphic.type` values SHALL map to ESIP element types via the registry. `text.html` labels SHALL be sanitized (strip HTML tags, preserve text). The adapter SHALL handle Gliffy v1.x and v2.x JSON schemas. Multi-page Gliffy files SHALL import each page as a separate diagram.

**Acceptance Criteria:**
- Gliffy flowchart and network diagram types parsed correctly
- HTML-formatted labels converted to plain text
- Multi-page import creates multiple diagrams

---

### REQ-IMP-018 — Miro Board Import

| Field | Value |
|---|---|
| **ID** | REQ-IMP-018 |
| **Title** | Miro Board Import |
| **Category** | Functional |
| **Persona(s)** | P-PM, P-BA, P-EA |
| **Priority** | S |
| **Rationale** | Miro is the dominant collaborative whiteboard for architecture workshops; converting Miro boards to structured ESIP models closes the workshop→model gap. |

**SHALL Description:** The system SHALL support Miro import via two paths: (a) **Miro REST API** — user authenticates via OAuth 2.0, selects a board, and the system fetches it via `GET /v2/boards/{boardId}/items`; (b) **Miro CSV/JSON export** — parse offline exports. From either path, the adapter SHALL extract: sticky notes (→ ESIP requirements or annotations), shapes (rectangles, circles, diamonds → element types via mapping), connectors (→ relationships), frames (→ containers/groups), text items (→ annotations), and images. Sticky note colors MAY be used as type hints in the mapping (configurable). The adapter SHALL cluster spatial proximity groups as suggested groupings, presenting them to the user for confirmation. Long sticky-note text SHALL be parsed for embedded structured data patterns (e.g., "REQ: ...", "AC: ...") to auto-extract requirement fields.

**Acceptance Criteria:**
- OAuth 2.0 Miro authorization flow completes in wizard
- All sticky notes on a board appear as importable elements
- Sticky-note color mapping configurable per-tenant
- Spatial clustering suggestion presented with visual preview

**Dependencies:** V08 for the Miro OAuth connector infrastructure

---

### REQ-IMP-019 — BPMN 2.0 XML Import

| Field | Value |
|---|---|
| **ID** | REQ-IMP-019 |
| **Title** | BPMN 2.0 XML Process Import |
| **Category** | Functional |
| **Persona(s)** | P-BA, P-SA |
| **Priority** | M |
| **Rationale** | BPMN is the ISO/OMG standard for business process modeling; BA teams produce BPMN 2.0 XML from Camunda, Signavio, Bizagi, and similar tools. |

**SHALL Description:** The system SHALL parse BPMN 2.0 XML (namespace `http://www.omg.org/spec/BPMN/20100524/MODEL`). The adapter SHALL extract: Processes, SubProcesses, Tasks (all types: UserTask, ServiceTask, ScriptTask, BusinessRuleTask, etc.), Gateways (Exclusive, Inclusive, Parallel, EventBased, Complex), Events (Start, Intermediate, End, all trigger types), SequenceFlows, MessageFlows, Pools, Lanes, Data Objects, Data Stores, and text annotations. Layout SHALL be extracted from the `BPMNDiagram`/`BPMNShape`/`BPMNEdge` section. Imported BPMN processes SHALL be linkable to ESIP ArchiMate Business Process elements. Conditional expressions on SequenceFlows SHALL be preserved as relationship labels/attributes. The system SHALL validate imported XML against the BPMN 2.0 XSD.

**Acceptance Criteria:**
- All BPMN 2.0 element types (task, gateway, event subtypes) imported
- Pool/Lane structure preserved as ESIP grouping
- BPMN diagram layout preserved from BPMNDiagram section
- Validation against XSD with errors reported in Import Report

---

### REQ-IMP-020 — Generic XMI / UML Import

| Field | Value |
|---|---|
| **ID** | REQ-IMP-020 |
| **Title** | Generic XMI 2.x / UML Import |
| **Category** | Functional |
| **Persona(s)** | P-EA, P-SA |
| **Priority** | M |
| **Rationale** | XMI is the OMG standard for UML model interchange and is emitted by virtually every UML tool (IBM Rational, MagicDraw, Enterprise Architect, Modelio, StarUML, Visual Paradigm). |

**SHALL Description:** The system SHALL parse XMI 2.1 and 2.5 files with `uml:Model` root elements. The adapter SHALL extract: Packages, Classes, Interfaces, Components, Nodes, Artifacts, Associations, Dependencies, Realizations, Usages, all UML diagram types found in the `xmi:Extension` section, and all tagged values / stereotypes. Tool-specific XMI extensions SHALL be handled gracefully: Sparx EA extensions (REQ-IMP-012), IBM RSA/RD extensions, MagicDraw/Cameo extensions, and generic `xmi:Extension` blocks. UML stereotypes SHALL map to ESIP element types; unrecognized stereotypes SHALL be preserved as custom attributes. The adapter SHALL handle XMI files that omit diagram layout (model-only) by auto-generating a hierarchical layout via V03's layout engine on import completion.

**Acceptance Criteria:**
- UML class model imported from 4 different XMI-emitting tools correctly
- Unknown XMI extensions do not cause fatal error
- Layout auto-generated when BPMNDiagram section absent

---

### REQ-IMP-021 — CSV / Excel Catalog Import

| Field | Value |
|---|---|
| **ID** | REQ-IMP-021 |
| **Title** | CSV / Excel Element Catalog Import |
| **Category** | Functional |
| **Persona(s)** | P-EA, P-BA, P-PM |
| **Priority** | M |
| **Rationale** | Many organizations maintain architecture catalogs, application inventories, and requirement lists in Excel; CSV/XLSX import is the universal onramp for structured data migration. |

**SHALL Description:** The system SHALL accept `.csv`, `.xlsx`, and `.ods` files. The import wizard SHALL: (a) parse the first row as column headers (with option to skip N header rows); (b) display a **column-mapping UI** where the user maps each column to an ESIP metamodel attribute (element type, name, description, custom attribute, or ignore); (c) infer common column patterns automatically (columns named "Name", "Type", "Description", "ID", "Owner" SHALL be auto-mapped); (d) support multi-sheet `.xlsx` — each sheet maps to a separate import batch or element type, user-configurable; (e) validate data types per mapped attribute (e.g., date columns, enum columns); (f) support a **bulk requirements import** mode that maps rows to ESIP Requirement elements with ID, title, description, priority, acceptance criteria columns. The adapter SHALL handle files with up to 50,000 rows without timeout. Encoding detection (UTF-8, UTF-16, Windows-1252) SHALL be automatic.

**Acceptance Criteria:**
- 50,000-row CSV imported within 60 s
- Auto-mapping detects standard column headers correctly
- Multi-sheet XLSX: user can select which sheets to import
- Encoding detected correctly for Windows-1252 encoded file

---

### REQ-IMP-022 — LeanIX Import

| Field | Value |
|---|---|
| **ID** | REQ-IMP-022 |
| **Title** | LeanIX EA Repository Import |
| **Category** | Integration |
| **Persona(s)** | P-EA, P-TA |
| **Priority** | S |
| **Rationale** | LeanIX is a leading SaaS EA platform; enterprises migrating to or federating with ESIP need a structured migration path. |

**SHALL Description:** The system SHALL support LeanIX import via: (a) **LeanIX REST API** — using the LeanIX API v1 (`/api/v1/factSheets`, `/api/v1/relations`, `/api/v1/subscriptions`) with a user-supplied API token; (b) **LeanIX CSV/Excel export** — from the LeanIX Pathfinder reporting. For API path: import FactSheets (Application, BusinessCapability, DataObject, Interface, ITComponent, Process, UserGroup, Project) with all fields, relations, and subscription (ownership) data. FactSheet types SHALL map to ESIP element/metamodel types via the registry. Relation types SHALL map to ESIP relationship types. LeanIX Quality Seals SHALL be captured as custom attributes. Pagination SHALL be handled automatically. Rate limiting SHALL be respected with configurable retry/backoff.

**Acceptance Criteria:**
- Full LeanIX tenant import (up to 5,000 FactSheets) completes within 10 min
- All FactSheet types have a default mapping in the registry
- Rate limit 429 responses retried with exponential backoff
- API token stored encrypted, never logged

---

### REQ-IMP-023 — Ardoq Import

| Field | Value |
|---|---|
| **ID** | REQ-IMP-023 |
| **Title** | Ardoq EA Repository Import |
| **Category** | Integration |
| **Persona(s)** | P-EA |
| **Priority** | C |
| **Rationale** | Ardoq is a growing EA SaaS platform; ESIP needs a migration path for Ardoq customers. |

**SHALL Description:** The system SHALL support Ardoq import via: (a) **Ardoq REST API** (`/api/component`, `/api/reference`, `/api/workspace`) using a user-supplied API token; (b) **Ardoq Excel export**. Components SHALL map to ESIP element types via the registry. References SHALL map to ESIP relationship types. Ardoq workspaces SHALL map to ESIP workspaces or sub-areas. Field values SHALL map to ESIP custom attributes. The adapter SHALL handle Ardoq's flexible "any component type" model by prompting the user to confirm type mappings for unrecognized component types.

**Acceptance Criteria:**
- Component and reference import functional for a sample Ardoq workspace
- User prompted for type confirmation on unrecognized component types
- Ardoq field values visible as custom attributes post-import

---

### REQ-IMP-024 — MEGA HOPEX Import

| Field | Value |
|---|---|
| **ID** | REQ-IMP-024 |
| **Title** | MEGA HOPEX Import |
| **Category** | Integration |
| **Persona(s)** | P-EA |
| **Priority** | C |
| **Rationale** | MEGA HOPEX is prevalent in French and European enterprise markets; large organizations with HOPEX repositories need a migration path. |

**SHALL Description:** The system SHALL support MEGA HOPEX import via: (a) **HOPEX GraphQL API** (HOPEX V5+) — query objects and relations using user-supplied API token/session; (b) **HOPEX XMI/XML export** — parse HOPEX's proprietary XML export format. The adapter SHALL extract: all MetaClass instances (Business Process, Application, Technology, Organization, etc.), all MetaAssociation instances, and all diagram/view layouts. HOPEX MetaClass names SHALL map to ESIP/ArchiMate types via the registry. The adapter SHALL document limitations where HOPEX proprietary modeling extensions have no direct ESIP equivalent, presenting these as unmapped custom attributes.

**Acceptance Criteria:**
- HOPEX GraphQL query executes and returns object list
- MetaClass instances import with all standard attributes
- Unknown MetaClass types fall back to generic ESIP element with warning

---

### REQ-IMP-025 — Avolution ABACUS Import

| Field | Value |
|---|---|
| **ID** | REQ-IMP-025 |
| **Title** | Avolution ABACUS Import |
| **Category** | Integration |
| **Persona(s)** | P-EA |
| **Priority** | C |
| **Rationale** | ABACUS is used in financial services and defense EA programs; clients migrating to ESIP need structured migration support. |

**SHALL Description:** The system SHALL support ABACUS import via: (a) **ABACUS XML/XMI export** — parse ABACUS repository exports; (b) **ABACUS Excel/CSV reports**. The adapter SHALL map ABACUS archetype constructs (Asset, Actor, Interface, System, Process, Location) to ESIP/ArchiMate equivalents. ABACUS analysis views SHALL import as ESIP diagrams with best-effort layout. The adapter SHALL handle ABACUS version differences (v5.x, v6.x) by auto-detecting schema version from export metadata.

**Acceptance Criteria:**
- ABACUS XML export parsed without fatal error
- Core archetype constructs mapped to ESIP metamodel
- Schema version detected from export metadata

---

### REQ-IMP-026 — BiZZdesign Import

| Field | Value |
|---|---|
| **ID** | REQ-IMP-026 |
| **Title** | BiZZdesign Architect/Enterprise Studio Import |
| **Category** | Integration |
| **Persona(s)** | P-EA |
| **Priority** | C |
| **Rationale** | BiZZdesign is a leading ArchiMate-native EA tool; migration from BiZZdesign to ESIP should be high-fidelity given shared ArchiMate foundation. |

**SHALL Description:** The system SHALL support BiZZdesign import via: (a) **AMEFF export** — BiZZdesign's primary exchange format; handled by REQ-IMP-013 which provides near-native fidelity; (b) **BiZZdesign XML repository export** — parse proprietary XML for elements not captured in AMEFF (risk items, roadmap items, matrices). The AMEFF path SHALL be the default recommended path. BiZZdesign-specific element extensions (Risk, Capability, Motivation extensions beyond ArchiMate standard) SHALL be mapped to ESIP's motivation/risk element types.

**Acceptance Criteria:**
- BiZZdesign AMEFF export imports at full fidelity via REQ-IMP-013
- BiZZdesign-specific risk/capability extensions recognized and mapped

---

### REQ-IMP-027 — erwin Data Modeler Import

| Field | Value |
|---|---|
| **ID** | REQ-IMP-027 |
| **Title** | erwin Data Modeler Import |
| **Category** | Integration |
| **Persona(s)** | P-EA, P-SA |
| **Priority** | C |
| **Rationale** | erwin is used for enterprise data modeling; data architects need to import ERDs and data dictionaries into ESIP's data layer. |

**SHALL Description:** The system SHALL support erwin import via: (a) **erwin XML export** (`.erwin` files) — parse entity definitions, attributes, primary/foreign keys, relationships, and domain definitions; (b) **erwin report CSV exports** — subject and entity lists. Entities SHALL map to ESIP Data Object / Application Data elements. Relationships (identifying, non-identifying, many-to-many) SHALL map to ESIP relationship types with appropriate stereotype labels. Primary key / foreign key metadata SHALL be preserved as custom attributes. Physical model details (table names, column types) SHALL be captured as custom attributes alongside logical names.

**Acceptance Criteria:**
- erwin XML parsed and entities/relationships imported
- PK/FK metadata preserved as custom attributes
- Logical and physical names both preserved

---

### REQ-IMP-028 — Round-Trip Export Support

| Field | Value |
|---|---|
| **ID** | REQ-IMP-028 |
| **Title** | Round-Trip Export (ESIP → Source Formats) |
| **Category** | Functional |
| **Persona(s)** | P-EA, P-SA |
| **Priority** | S |
| **Rationale** | Teams co-maintain models in ESIP and source tools; round-trip export allows changes made in ESIP to be pushed back to source formats. |

**SHALL Description:** The system SHALL support export to the following formats: (a) **AMEFF** (required, M) — full model export; (b) **draw.io XML** (S) — export ESIP diagrams as draw.io XML; (c) **Mermaid text** (S) — export supported diagram types as Mermaid syntax; (d) **PlantUML** (C) — export selected diagram types; (e) **BPMN 2.0 XML** (S) — export Business Process elements and diagrams; (f) **XMI 2.1** (S) — export UML-compatible model; (g) **CSV/Excel** (M) — export element catalogs as tabular data. Exports SHALL be accessible from the Import History view (re-export same content) and from any workspace/diagram via Export menu. Export SHALL respect access permissions (user cannot export elements they cannot read).

**Acceptance Criteria:**
- AMEFF export passes AMEFF XSD validation
- draw.io XML opens correctly in diagrams.net application
- CSV export includes all element attributes as columns

---

### REQ-IMP-029 — Per-Source Migration Accelerators

| Field | Value |
|---|---|
| **ID** | REQ-IMP-029 |
| **Title** | Per-Source Migration Accelerator Templates |
| **Category** | Functional |
| **Persona(s)** | P-EA, P-TA |
| **Priority** | S |
| **Rationale** | Source-specific migration patterns (e.g., "Sparx EA to ESIP", "LeanIX to ESIP") recur across clients; pre-built templates reduce migration effort significantly. |

**SHALL Description:** The system SHALL ship a **Migration Accelerator Template Library** — a curated, versioned set of mapping templates for each supported source format. Each template SHALL include: (a) default element type mappings; (b) default attribute mappings; (c) common post-import cleanup rules (e.g., trim whitespace from names, normalize casing); (d) a migration guide document (Markdown, linked from wizard) covering known caveats, recommended export settings in the source tool, and expected fidelity. Templates SHALL be maintained in a Git repository and versioned with the adapter. Tenant Admins SHALL be able to clone and customize templates without modifying the system library. The wizard SHALL display the applicable template and its version at the mapping step.

**Acceptance Criteria:**
- Template library ships with templates for all M-priority source formats
- Tenant Admin can clone, customize, and save a private template
- Migration guide document linked from wizard step 4
- Template version displayed in wizard and recorded in provenance

**Cross-reference:** V16 owns competitive intelligence and accelerator analytics.

---

### REQ-IMP-030 — Import Audit and Governance

| Field | Value |
|---|---|
| **ID** | REQ-IMP-030 |
| **Title** | Import Audit Logging and Governance Controls |
| **Category** | Governance |
| **Persona(s)** | P-TA |
| **Priority** | M |
| **Rationale** | Enterprise governance requires immutable audit trails for all data ingestion events. |

**SHALL Description:** The system SHALL write an immutable audit log entry for every import operation, recording: user ID, workspace ID, source format, file hash, import job ID, element counts (created/updated/skipped/failed), merge strategy chosen, mapping template ID, and outcome (success/partial/failed). Audit logs SHALL be queryable by Tenant Admin (by user, workspace, date range, format). Tenant Admins SHALL be able to configure: (a) approved source formats (whitelist); (b) maximum file size per format; (c) require-approval workflow for large imports (> N elements, configurable); (d) data classification tagging automatically applied to imported elements. Import attempts using blocked formats SHALL be rejected at upload with a clear policy message.

**Acceptance Criteria:**
- Audit log entry written for every import attempt (including failed/rejected)
- Tenant Admin can whitelist/blacklist formats in < 5 clicks
- Require-approval workflow routes to designated approver before commit
- Blocked format rejection returns 403 with policy code

---

### REQ-IMP-031 — Layout Fidelity Preservation

| Field | Value |
|---|---|
| **ID** | REQ-IMP-031 |
| **Title** | Diagram Layout Fidelity on Import |
| **Category** | Functional |
| **Persona(s)** | P-SA, P-EA |
| **Priority** | S |
| **Rationale** | Architects have invested significant time in laying out diagrams; preserving layout reduces post-import rework. |

**SHALL Description:** For source formats that include spatial geometry (draw.io, Visio .vsdx, AMEFF view nodes, Sparx EA t_diagramobjects, BPMN BPMNShape), the import adapter SHALL: (a) extract absolute x/y coordinates and width/height; (b) normalize to ESIP's coordinate system (origin top-left, units pixels at 96 DPI); (c) pass the coordinate data to V03's maxGraph canvas for initial placement; (d) preserve relative z-order (layering) where available. When source coordinates are absent, the system SHALL auto-apply a hierarchical layout algorithm. Layout fidelity SHALL be measured as: percentage of connector crossings vs. source; mean positional drift of nodes. The user SHALL be able to trigger a "re-layout" from the canvas without losing metamodel data.

**Acceptance Criteria:**
- draw.io diagram position preserved within ±5 px after import
- AMEFF view layout preserved within ±5 px
- Auto-layout applied when source has no geometry data
- Re-layout available without data loss

**Cross-reference:** V03 owns the maxGraph rendering; this requirement specifies the data contract passed to V03.

---

### REQ-IMP-032 — Import Performance and Scalability

| Field | Value |
|---|---|
| **ID** | REQ-IMP-032 |
| **Title** | Import Throughput and Scalability NFRs |
| **Category** | NFR |
| **Persona(s)** | P-TA |
| **Priority** | M |
| **Rationale** | Enterprise migrations may involve tens of thousands of elements; the system must scale without degrading other tenants. |

**SHALL Description:** Import processing SHALL be tenant-isolated (separate async worker queue per tenant). The system SHALL support: (a) ≥ 10,000 elements per import job without exceeding 5-minute processing time on reference hardware (8-core, 16 GB); (b) ≥ 100 concurrent import jobs across all tenants; (c) per-tenant concurrency limit (default 3 concurrent jobs, configurable). Large import jobs SHALL not degrade API response times for other operations in the same tenant by > 10%. Import workers SHALL be horizontally scalable (stateless; state in shared DB/cache). Import job queue SHALL have a configurable priority (Tenant Admin can mark a job as high-priority).

**Acceptance Criteria:**
- 10,000-element XMI import completes in ≤ 5 min
- 100 concurrent small imports (100 elements each) complete without errors
- API P95 latency for read operations unchanged during concurrent imports

---

### REQ-IMP-033 — Import Notifications

| Field | Value |
|---|---|
| **ID** | REQ-IMP-033 |
| **Title** | Import Job Notifications |
| **Category** | UX |
| **Persona(s)** | P-EA, P-SA |
| **Priority** | M |
| **Rationale** | Async import jobs complete after the user has navigated away; they need to be notified without polling. |

**SHALL Description:** On completion of any import job (success, partial, or failed), the system SHALL: (a) send an in-app notification (toast + notification bell) with job summary and link to Import Report; (b) optionally send an email notification (user-configurable); (c) optionally send a webhook notification (Tenant Admin configurable, for CI/CD integration). Notifications SHALL include: job ID, workspace name, element counts, status, and deep link to the workspace.

**Acceptance Criteria:**
- In-app notification appears within 5 s of job completion
- Email notification arrives within 2 min of job completion
- Webhook payload documented in API spec and includes all required fields

---

## REQ-CV — Computer-Vision Whiteboard & Sketch Capture

### REQ-CV-001 — CV Capture Pipeline Architecture

| Field | Value |
|---|---|
| **ID** | REQ-CV-001 |
| **Title** | Computer-Vision Capture Pipeline Architecture |
| **Category** | Functional |
| **Persona(s)** | P-EA, P-SA, P-PM, P-FIELD |
| **Priority** | M |
| **Rationale** | Whiteboards are the primary artifact of architecture workshops; converting them to structured models eliminates manual re-entry and accelerates model creation. |

**SHALL Description:** The system SHALL implement a CV capture pipeline as a first-class feature. The pipeline stages SHALL be: (1) **Image Upload** — accept JPEG/PNG/HEIC/HEIF/WebP/TIFF, single or multi-image batch, via web upload or mobile app; (2) **Pre-processing** — perspective correction, skew/tilt correction, glare/reflection suppression, contrast normalization, resolution upscaling if < 1 MP; (3) **Shape Detection** — detect geometric primitives (rectangles, rounded rectangles, circles, ellipses, diamonds, triangles, cylinders, parallelograms) and freehand enclosed regions; (4) **Connector Detection** — detect lines, arrows (single/double headed), curved connectors, dashed connectors; (5) **Text/Label OCR** — extract text labels inside or adjacent to shapes and on connectors; (6) **Sticky Note Detection** — detect post-it sticky notes by color and rectangular outline; (7) **Semantic Classification** — classify detected elements into diagram type (flowchart, ER, sequence, ArchiMate, etc.) and element types; (8) **Model Reconstruction** — reconstruct as ESIP elements and relationships with confidence scores; (9) **User Review** — present annotated image overlay showing detected elements + ESIP model preview for confirmation; (10) **Commit** — on user approval, write to designated target area/artifact.

**Cross-reference:** V06 owns the CV/OCR model infrastructure and GPU hosting. This volume owns the pipeline API contract, UX flow, and integration into the ESIP model.

---

### REQ-CV-002 — Perspective and Image Pre-Processing

| Field | Value |
|---|---|
| **ID** | REQ-CV-002 |
| **Title** | Perspective Correction and Image Enhancement |
| **Category** | AI/ML |
| **Persona(s)** | P-FIELD, P-PM |
| **Priority** | M |
| **Rationale** | Phone photos of whiteboards are almost always taken at an angle with glare; automatic correction is essential for accurate downstream detection. |

**SHALL Description:** The pre-processing stage SHALL: (a) **perspective correction** — detect the whiteboard boundary (edge detection, corner finding) and apply a projective transformation to produce a front-facing, rectangular crop; fall back to the full image if boundary detection confidence < 0.6; (b) **skew correction** — detect and correct rotation up to ±45 degrees; (c) **glare suppression** — identify specular highlight regions and apply inpainting/masking to reduce interference with text/line detection; (d) **contrast enhancement** — adaptive histogram equalization on the luminance channel to improve ink/marker visibility against whiteboard surface; (e) **noise reduction** — Gaussian/bilateral filter for low-light photos; (f) **resolution normalization** — upsample to ≥ 2 MP before passing to shape/text detection if input is smaller. Each pre-processing step SHALL be logged with before/after quality metrics. Users SHALL be able to see the pre-processed image before OCR results and request re-processing with different settings.

**Acceptance Criteria:**
- Perspective correction applied to ≥ 90% of whiteboard photos with detectable boundary
- Text OCR accuracy improves by ≥ 20 percentage points after pre-processing vs. raw image on test corpus
- User can view pre-processed image in review step
- Pre-processing completes within 5 s for a 12 MP image

**Cross-reference:** V06 owns model hosting; this requirement specifies the behavioral contract.

---

### REQ-CV-003 — Shape and Connector Detection

| Field | Value |
|---|---|
| **ID** | REQ-CV-003 |
| **Title** | Diagram Shape and Connector Detection |
| **Category** | AI/ML |
| **Persona(s)** | P-EA, P-SA, P-PM |
| **Priority** | M |
| **Rationale** | Accurate shape detection is the foundation of whiteboard-to-model conversion. |

**SHALL Description:** The shape detection stage SHALL detect and classify: (a) **Geometric shapes** — rectangle (solid/dashed border), rounded rectangle, square, circle, ellipse, diamond, triangle, cylinder (3D box), parallelogram, actor (stick figure), cloud shape, note shape; (b) **Freehand enclosed regions** — hand-drawn closed polygons interpreted as shapes; (c) **Connectors** — straight lines, arrows (single/double headed), curved/bent lines, dashed lines, orthogonal connectors; (d) **Arrow direction** — detected arrowheads indicate directionality; (e) **Connector source/target** — associate each connector endpoint with the nearest shape within a configurable proximity threshold. Each detected element SHALL carry: bounding box coordinates, shape type (with confidence score 0.0–1.0), and a unique detection ID. The system SHALL distinguish between connector lines and shape borders. Overlapping shapes SHALL be handled by reporting both with overlap metadata.

**Acceptance Criteria:**
- Precision ≥ 0.85 on rectangles, diamonds, circles on reference test corpus
- Arrow direction correctly detected in ≥ 90% of cases with clear arrowheads
- Connector source/target association correct for ≥ 85% of connectors
- Freehand closed regions detected with precision ≥ 0.75

---

### REQ-CV-004 — Handwriting OCR for Labels

| Field | Value |
|---|---|
| **ID** | REQ-CV-004 |
| **Title** | Handwriting OCR and Label Extraction |
| **Category** | AI/ML |
| **Persona(s)** | P-EA, P-SA, P-PM, P-FIELD |
| **Priority** | M |
| **Rationale** | Shape labels are the primary semantic content of whiteboard diagrams; accurate handwriting recognition directly determines model usefulness. |

**SHALL Description:** The OCR stage SHALL: (a) extract text from within and adjacent to each detected shape, and text on connector lines; (b) support **handwritten and printed text** in a single model; (c) support Latin script (priority), with planned extension to CJK characters; (d) return per-word confidence scores; (e) handle multi-line labels within shapes; (f) handle angled/rotated text (common on connector labels); (g) handle colored marker text on white/light backgrounds AND dark-background whiteboards; (h) associate extracted text with the nearest bounding shape (or with the connector if the text lies along a connector path); (i) flag standalone text blocks not associated with any shape as **free annotations**; (j) apply a post-OCR language model correction pass to fix common substitution errors (O/0, l/1, etc.) in the context of known metamodel terms.

**Acceptance Criteria:**
- Character error rate ≤ 10% on handwritten text test corpus (12-point equivalent marker)
- Character error rate ≤ 5% on printed/typed text in photos
- Multi-line labels extracted as multi-line strings
- Angled connector labels extracted with CER ≤ 15%
- Post-OCR correction reduces known metamodel-term errors by ≥ 50%

**Cross-reference:** V06 owns OCR model selection and training.

---

### REQ-CV-005 — Sticky Note Detection and Extraction

| Field | Value |
|---|---|
| **ID** | REQ-CV-005 |
| **Title** | Sticky Note Detection and Content Extraction |
| **Category** | AI/ML |
| **Persona(s)** | P-PM, P-BA, P-EA |
| **Priority** | S |
| **Rationale** | Sticky notes are the most common physical artifact in agile and EA workshops; extracting them with their color context unlocks automatic requirement/story capture. |

**SHALL Description:** The sticky note detection stage SHALL: (a) detect rectangular regions matching common sticky note aspect ratios (3:3, 3:5, 4:6 inches) with solid or near-solid background colors; (b) identify sticky note color (yellow, blue, green, pink, orange, purple) and return the dominant HSV value; (c) extract the text content of each sticky note via OCR (REQ-CV-004); (d) detect sticky notes on both whiteboards and physical table surfaces; (e) group spatially proximate sticky notes into clusters (swimlane/affinity groups) with suggested group label; (f) allow color-to-element-type mapping (e.g., yellow = Requirement, pink = Risk, green = Action Item) configurable per-user or per-capture session; (g) present detected sticky notes as a list alongside the image overlay for review.

**Acceptance Criteria:**
- Sticky notes detected with precision ≥ 0.88 on reference corpus
- Color classification correct for 6 standard colors in ≥ 95% of cases
- Text extracted from sticky notes with CER ≤ 12%
- Color-to-type mapping persisted per user preference

---

### REQ-CV-006 — Semantic Element Classification

| Field | Value |
|---|---|
| **ID** | REQ-CV-006 |
| **Title** | Semantic Diagram Type and Element Classification |
| **Category** | AI/ML |
| **Persona(s)** | P-EA, P-SA, P-BA |
| **Priority** | M |
| **Rationale** | The same rectangle on a whiteboard could mean an Application in an ArchiMate diagram or a Process in a BPMN diagram; semantic classification determines the correct metamodel type. |

**SHALL Description:** The classification stage SHALL: (a) **diagram type inference** — infer the overall diagram type (flowchart, ER, sequence/swim-lane, ArchiMate Layer, UML class, UML sequence, use-case, mind map, generic box/arrow) from the global arrangement of detected shapes, connector patterns, and OCR text hints; (b) **element type mapping** — given the inferred diagram type, map each detected shape geometry + label to a specific ESIP element type (e.g., in ArchiMate context: rectangle → Application Component; cylinder → Data Store; stick figure → Actor); (c) when multiple diagram type interpretations are plausible (confidence < 0.75), present the top 3 candidates to the user for selection; (d) allow the user to **override** the diagram type inference at the start of the review step, which re-runs the element classification with the user-selected type; (e) use label text as a strong signal — if OCR returns a label matching a known ArchiMate type name (e.g., "Application Service"), classify accordingly.

**Acceptance Criteria:**
- Diagram type correctly inferred in ≥ 80% of test cases across all supported types
- Element type mapping achieves ≥ 75% precision when diagram type is correct
- User override of diagram type triggers re-classification within 3 s
- Label-driven classification overrides geometry-based when confidence > 0.9

---

### REQ-CV-007 — Model Reconstruction and ESIP Element Creation

| Field | Value |
|---|---|
| **ID** | REQ-CV-007 |
| **Title** | CV Model Reconstruction — ESIP Element and Relationship Creation |
| **Category** | Functional |
| **Persona(s)** | P-EA, P-SA |
| **Priority** | M |
| **Rationale** | The CV pipeline output must produce first-class ESIP model elements, not just a flat image annotation. |

**SHALL Description:** On user approval of the CV review step (REQ-CV-009), the system SHALL create ESIP elements and relationships as follows: (a) each detected shape → ESIP element of the classified type, with name = OCR label text, description = any adjacent annotation text, provenance.source = "cv-capture", provenance.captureJobId = job ID, provenance.confidence = detection confidence; (b) each detected connector → ESIP relationship of the classified type, source = source element, target = target element, label = connector OCR text; (c) detected groups/clusters → ESIP grouping containers; (d) free annotations → ESIP notes; (e) sticky notes → ESIP elements of the color-mapped type. Elements SHALL be placed on the V03 maxGraph canvas using the spatial coordinates (normalized from pixel coordinates to canvas coordinates). Provenance SHALL link back to the source image and capture job.

**Acceptance Criteria:**
- Created elements appear in workspace model immediately after commit
- Provenance block references source image file hash and capture job ID
- Connector source/target references correct ESIP elements
- Elements positioned on canvas reflecting spatial layout of original whiteboard

---

### REQ-CV-008 — Confidence Scoring and Review Interface

| Field | Value |
|---|---|
| **ID** | REQ-CV-008 |
| **Title** | Confidence Scores and Interactive Review Overlay |
| **Category** | UX |
| **Persona(s)** | P-EA, P-SA, P-PM |
| **Priority** | M |
| **Rationale** | CV is imperfect; users must be able to review and correct before committing, with clear indication of which detections are uncertain. |

**SHALL Description:** The review interface SHALL display: (a) the pre-processed image with a **detection overlay** — bounding boxes color-coded by confidence (green ≥ 0.85, yellow 0.60–0.84, red < 0.60); (b) a side-by-side panel showing the detected element list with type, label, and confidence score for each; (c) **in-place editing** — clicking on a bounding box in the overlay opens an edit popover to: change the element type, edit the OCR text, delete the detection, or merge two detections; (d) ability to **draw a new box** on the image to add a missed element; (e) ability to **reassign connector endpoints** by dragging; (f) a **diagram type selector** at the top of the review step with current inferred type and alternatives; (g) a **confidence threshold filter** slider — elements below threshold are highlighted for mandatory review; (h) bulk-accept all high-confidence (≥ 0.85) elements in one click. The review interface SHALL be responsive and usable on tablet screens (min 768 px wide).

**Acceptance Criteria:**
- Overlay renders within 3 s of CV pipeline completion
- Color-coded confidence bounding boxes rendered for all detections
- In-place edit changes reflected in the element list immediately
- Bulk-accept correctly excludes items below threshold
- Interface usable on 768 px wide tablet screen

---

### REQ-CV-009 — Target Workspace Designation

| Field | Value |
|---|---|
| **ID** | REQ-CV-009 |
| **Title** | User-Designated Target Area and Artifact |
| **Category** | UX |
| **Persona(s)** | P-EA, P-SA, P-PM |
| **Priority** | M |
| **Rationale** | CV capture must be directed to a specific area of the model; the user must be able to specify exactly where captured content lands. |

**SHALL Description:** Before initiating a CV capture (or as the first step of the CV review workflow), the user SHALL designate: (a) **Target Workspace** — the ESIP workspace/project to add elements to; (b) **Target Area/Container** — optionally, a specific folder, view, layer, or existing diagram within that workspace; (c) **Target Diagram** — optionally, an existing diagram to add elements to (in which case imported elements are positioned relative to existing canvas content); (d) **New Diagram** — option to create a new diagram from the capture result. The mobile app (REQ-MOB) SHALL allow pre-setting the default target workspace + area so that quick captures go to the right place without re-navigation. The target designation SHALL be persisted as user preference per device and updated when changed.

**Acceptance Criteria:**
- Target workspace/area selector displayed before CV review step
- Elements placed in designated target on commit
- Mobile default target pre-selected from saved preference
- Changing target on mobile takes ≤ 3 taps

---

### REQ-CV-010 — Multi-Photo Stitching

| Field | Value |
|---|---|
| **ID** | REQ-CV-010 |
| **Title** | Multi-Photo Whiteboard Stitching |
| **Category** | AI/ML |
| **Persona(s)** | P-FIELD, P-PM |
| **Priority** | S |
| **Rationale** | Large whiteboards require multiple photos; stitching them into a single panoramic view before detection dramatically improves coverage and spatial relationships. |

**SHALL Description:** When the user uploads ≥ 2 photos designated as a multi-photo capture of the same whiteboard, the system SHALL: (a) detect overlapping regions between adjacent photos using feature matching (SIFT/ORB-style keypoint matching); (b) compute homography transforms and warp images to a common plane; (c) blend the overlap zones to produce a seamless panoramic composite; (d) run the full CV pipeline (REQ-CV-001) on the composite image; (e) present the stitched panoramic image in the review interface with detection overlay; (f) handle up to 6 photos per stitch session; (g) detect and report when photos appear to not overlap (likely different whiteboards) and prompt the user. The stitching SHALL preserve the aspect ratio of the final composite without excessive distortion.

**Acceptance Criteria:**
- 3-photo stitch of a 120-inch whiteboard produces a usable composite without visible seams
- Stitch completes within 20 s for 3 × 12 MP photos
- Non-overlapping photos flagged with user prompt
- Detection on stitched image maintains same quality as single-photo detection

---

### REQ-CV-011 — Printed Diagram Photo Import

| Field | Value |
|---|---|
| **ID** | REQ-CV-011 |
| **Title** | Printed / Projected Diagram Photo Capture |
| **Category** | AI/ML |
| **Persona(s)** | P-EA, P-SA |
| **Priority** | S |
| **Rationale** | Architects frequently photograph printouts of existing diagrams at meetings; the CV pipeline should work on printed content as well as hand-drawn content. |

**SHALL Description:** The CV pipeline SHALL detect and handle printed diagram photos: (a) distinguish between printed/digital-rendered shapes (clean edges, uniform line weight) and hand-drawn shapes (variable line weight, imprecise angles); (b) apply a sharper detection model path for printed diagrams with tighter geometric fitting; (c) apply Tesseract-class OCR (not handwriting-specific) for printed text, achieving CER ≤ 3%; (d) detect and extract legend/key sections as metadata; (e) detect and OCR title blocks; (f) for printed Visio or draw.io outputs, attempt to match detected shapes to known stencil shapes for higher-confidence type classification; (g) handle screenshots of diagramming tools shown on screens (monitor/projector photo captures).

**Acceptance Criteria:**
- Printed text OCR achieves CER ≤ 3%
- Printed shapes detected with precision ≥ 0.90
- Title block text extracted as diagram metadata

---

### REQ-CV-012 — Review-Before-Commit Workflow

| Field | Value |
|---|---|
| **ID** | REQ-CV-012 |
| **Title** | Mandatory Review-Before-Commit Gate |
| **Category** | Governance |
| **Persona(s)** | P-EA, P-SA, P-PM |
| **Priority** | M |
| **Rationale** | CV is probabilistic; committing unreviewed CV output directly to the model would introduce noise. A mandatory review gate is a governance control. |

**SHALL Description:** The system SHALL enforce a review-before-commit gate for all CV capture operations: (a) CV pipeline results SHALL NEVER be auto-committed to the model without explicit user confirmation; (b) the review interface (REQ-CV-008) SHALL present all detected elements before any model write; (c) the commit button SHALL be disabled until the user has acknowledged the review (scrolled through all red-confidence items or explicitly dismissed review); (d) partial commit SHALL be supported — user can deselect individual detections from the commit; (e) the system SHALL record: which detections were committed vs. skipped; which labels were manually edited; which element types were overridden; this metadata SHALL be stored in the capture job provenance record.

**Acceptance Criteria:**
- No CV element appears in the model without explicit user commit action
- Partial commit correctly creates only selected elements
- Commit button disabled until review acknowledgement
- Provenance records all manual edits made during review

---

### REQ-CV-013 — CV Capture Job History and Rollback

| Field | Value |
|---|---|
| **ID** | REQ-CV-013 |
| **Title** | CV Capture Job History and Rollback |
| **Category** | Functional |
| **Persona(s)** | P-EA, P-SA |
| **Priority** | S |
| **Rationale** | Post-commit review may reveal that CV misclassified elements; users need to be able to undo a capture batch. |

**SHALL Description:** The system SHALL maintain a **Capture Job History** per workspace, listing all CV capture jobs with status, source image thumbnails, element counts, and timestamp. Each committed capture job SHALL support atomic rollback within the configurable retention window (default 30 days). Rollback SHALL remove all elements created by that capture job that have not been subsequently modified by other operations (modified elements SHALL be flagged and left in place). The capture job history SHALL be accessible from the workspace sidebar.

**Acceptance Criteria:**
- Capture job history lists last 50 jobs per workspace
- Rollback of a 50-element capture completes within 10 s
- Modified elements correctly excluded from rollback with warning message

---

### REQ-CV-014 — Voice Annotation During Capture

| Field | Value |
|---|---|
| **ID** | REQ-CV-014 |
| **Title** | Voice Annotation Linked to CV Capture |
| **Category** | Functional |
| **Persona(s)** | P-PM, P-FIELD |
| **Priority** | C |
| **Rationale** | Architects often narrate context while photographing whiteboards; capturing that audio and associating it with the capture session adds valuable context. |

**SHALL Description:** During a mobile CV capture session (REQ-MOB), the user SHALL be able to record a voice annotation (up to 5 minutes) attached to the capture batch. The system SHALL: (a) transcribe the voice annotation using the speech-to-text pipeline (see REQ-MOB-010); (b) attach the transcript as a capture session note visible in the review interface; (c) allow the user to associate sections of the transcript with specific detected elements during review; (d) store the audio file linked to the capture job provenance. The transcription SHALL complete before the review interface is presented (or be shown as "transcribing..." with the review proceeding without it if transcription exceeds 30 s).

**Acceptance Criteria:**
- Voice annotation recorded and transcript displayed in review interface
- Transcript association with specific elements possible during review
- Audio file stored and accessible from capture job history

---

## REQ-MOB — Mobile Whiteboarding & Companion App

### REQ-MOB-001 — Mobile App Architecture

| Field | Value |
|---|---|
| **ID** | REQ-MOB-001 |
| **Title** | Mobile Companion App Architecture |
| **Category** | Functional |
| **Persona(s)** | P-MOB, P-FIELD |
| **Priority** | M |
| **Rationale** | A mobile-first capture surface dramatically reduces the friction between physical workshop activities and the digital model, enabling real-time model updates from the field. |

**SHALL Description:** The system SHALL provide a mobile companion app available as: (a) **Native iOS app** (iOS 16+, Swift/SwiftUI or React Native); (b) **Native Android app** (Android 12+, Kotlin/Jetpack Compose or React Native); (c) **Progressive Web App (PWA)** as a fallback (installable, offline-capable, camera/microphone access via Web APIs). The recommended implementation SHALL be **React Native (Expo)** to maximize code reuse with the ESIP React web frontend and to meet both native platforms simultaneously. The app SHALL: link to the user's ESIP account and project workspace; present a streamlined, mobile-optimized UI distinct from the desktop web app; be distributed via Apple App Store, Google Play Store, and as a PWA URL. The app SHALL target a ≤ 30 MB download size.

**Acceptance Criteria:**
- App installable on iOS 16+ and Android 12+ devices
- PWA installs from mobile browser without App Store
- App size ≤ 30 MB on first install
- React Native codebase shares ≥ 60% of logic with web frontend

---

### REQ-MOB-002 — Authentication and SSO

| Field | Value |
|---|---|
| **ID** | REQ-MOB-002 |
| **Title** | Mobile Authentication, SSO, and MFA |
| **Category** | Security |
| **Persona(s)** | P-MOB |
| **Priority** | M |
| **Rationale** | Enterprise mobile apps must support the same identity providers as the web app; separate authentication would create security gaps. |

**SHALL Description:** The mobile app SHALL authenticate users via: (a) **OAuth 2.0 / OIDC** with the same identity providers as the web app (Azure AD, Okta, Google Workspace, SAML-bridged IdPs); (b) PKCE flow for native/mobile OAuth; (c) **Biometric authentication** (Face ID, Touch ID, fingerprint) for re-authentication after the initial login, using the device secure enclave to store the refresh token; (d) **MFA** — support TOTP and push-based MFA as required by the IdP; (e) **Session management** — refresh tokens rotated on each use, stored in iOS Keychain / Android Keystore; (f) automatic re-authentication prompt on token expiry without data loss (in-progress capture preserved). Device management: Tenant Admin SHALL be able to remotely revoke a device's access.

**Acceptance Criteria:**
- OAuth 2.0 PKCE flow completes on iOS 16+ and Android 12+
- Biometric re-auth works without re-entering password
- Remote device revocation takes effect within 60 s
- In-progress capture not lost on token expiry re-auth

**Cross-reference:** V11 owns identity federation infrastructure; this requirement specifies the mobile app's consumption of it.

---

### REQ-MOB-003 — Workspace Linking and Project Navigation

| Field | Value |
|---|---|
| **ID** | REQ-MOB-003 |
| **Title** | Mobile Workspace Linking and Navigation |
| **Category** | UX |
| **Persona(s)** | P-MOB |
| **Priority** | M |
| **Rationale** | Captured content must be directed to the correct workspace/project; the mobile UI must make workspace selection fast and discoverable. |

**SHALL Description:** The mobile app home screen SHALL display: (a) **Recent workspaces** — last 5 accessed workspaces; (b) **Pinned workspaces** — user-pinned for quick access; (c) **Search** — full-text search across accessible workspaces; (d) **Quick-capture button** — large primary CTA on home screen, using the last-used workspace as default target. Workspace selection SHALL support **deep linking** from QR codes (scanning a workspace QR code navigates directly to that workspace's capture context). The app SHALL cache workspace metadata for offline access. Navigation depth SHALL be limited to Workspace → Area → (optional diagram/artifact) for mobile simplicity.

**Acceptance Criteria:**
- Recent and pinned workspaces displayed within 1 s of app open (cached)
- QR code scan navigates to workspace within 2 s
- Workspace search returns results within 2 s
- Quick-capture accessible in ≤ 2 taps from home screen

---

### REQ-MOB-004 — Whiteboard Photo Capture

| Field | Value |
|---|---|
| **ID** | REQ-MOB-004 |
| **Title** | Mobile Whiteboard Photo Capture |
| **Category** | Functional |
| **Persona(s)** | P-MOB, P-FIELD |
| **Priority** | M |
| **Rationale** | This is the primary field use case; the capture UX must be fast, robust, and guide the user to take a good-quality photo. |

**SHALL Description:** The mobile app SHALL provide a dedicated **Whiteboard Capture** screen with: (a) **Live camera viewfinder** with real-time whiteboard boundary detection overlay (highlighting the detected whiteboard outline in green when well-framed); (b) **Auto-capture** — option to automatically trigger capture when a stable, well-framed whiteboard is detected (holding steady for 1 s); (c) **Manual capture** button; (d) **Multi-photo mode** — for large whiteboards, sequential capture of multiple photos with visual stitching guidance (shows overlap indicator with previous photo); (e) **Review thumbnail** — after capture, shows the perspective-corrected thumbnail before submitting to the CV pipeline; (f) **Retake** option; (g) **Flash/torch** toggle; (h) **Grid overlay** option for alignment; (i) the capture screen SHALL request camera permission on first launch and provide clear rationale. Capture SHALL work with the rear camera only (front camera not appropriate for whiteboard capture).

**Acceptance Criteria:**
- Whiteboard boundary overlay appears within 500 ms of pointing at whiteboard
- Auto-capture triggers within 1 s of stable framing
- Multi-photo overlap guide shows correct overlap with previous frame
- Retake discards previous capture cleanly

---

### REQ-MOB-005 — Mobile CV Review and Commit

| Field | Value |
|---|---|
| **ID** | REQ-MOB-005 |
| **Title** | Mobile CV Result Review and Commit |
| **Category** | UX |
| **Persona(s)** | P-MOB |
| **Priority** | M |
| **Rationale** | The review-before-commit gate (REQ-CV-012) applies on mobile; the UX must be adapted for small screens without sacrificing review quality. |

**SHALL Description:** On the mobile app, the CV review interface SHALL be a mobile-optimized version of REQ-CV-008: (a) **Scrollable annotated image** — pinch-to-zoom on the detection overlay; (b) **Swipeable element cards** — horizontal card stack showing each detected element for review/edit/delete; (c) **Quick-approve** — swipe right on a card to approve, swipe left to reject; (d) **Tap to edit** — tapping an element card opens an edit sheet (element type picker, label edit); (e) **Low-confidence filter** — option to show only below-threshold items; (f) **Commit button** — large, accessible, at the bottom of the screen; (g) **Defer to desktop** option — save the capture job without committing, to complete review on the desktop web app.

**Acceptance Criteria:**
- Annotated image renders within 3 s on a mid-range Android device
- Swipe-to-approve gesture works reliably
- "Defer to desktop" saves job accessible from desktop within 10 s
- Edit sheet opens within 300 ms of tap

---

### REQ-MOB-006 — Mobile Sketch / Drawing

| Field | Value |
|---|---|
| **ID** | REQ-MOB-006 |
| **Title** | Mobile Freehand Sketch and Drawing |
| **Category** | Functional |
| **Persona(s)** | P-MOB, P-PM |
| **Priority** | S |
| **Rationale** | When no physical whiteboard is available, the mobile app should allow freehand sketching directly on the device, feeding the same CV pipeline. |

**SHALL Description:** The mobile app SHALL include a **sketch canvas** mode where the user can: (a) draw freehand using finger or stylus (Apple Pencil / S-Pen supported); (b) select from pen types: marker (thick), pen (thin), highlighter (transparent); (c) select colors from a limited palette (black, blue, red, green, orange); (d) eraser tool; (e) text input (keyboard-typed labels as an alternative to handwritten); (f) clear canvas; (g) submit sketch to the CV pipeline (same pipeline as photo capture — freehand drawing detection uses the "printed diagram" path for cleaner input); (h) undo/redo (≥ 20 steps). The sketch canvas SHALL support a minimum 1920×1080 virtual canvas with infinite-canvas panning.

**Acceptance Criteria:**
- Freehand drawing latency ≤ 16 ms (60 fps) on current-generation devices
- Apple Pencil pressure sensitivity supported on compatible iPads
- Sketch submitted to CV pipeline identical to photo submission path
- Undo/redo supports ≥ 20 steps

---

### REQ-MOB-007 — Voice-to-Text Note Dictation

| Field | Value |
|---|---|
| **ID** | REQ-MOB-007 |
| **Title** | Voice-to-Text Requirement and Note Dictation |
| **Category** | Functional |
| **Persona(s)** | P-MOB, P-PM, P-BA |
| **Priority** | M |
| **Rationale** | Quick dictation of requirements, decisions, and notes during a meeting is faster than typing on mobile; speech-to-text lowers the capture barrier. |

**SHALL Description:** The mobile app SHALL provide a **Voice Capture** mode allowing: (a) tap-to-speak recording of a note/requirement; (b) real-time speech-to-text transcription displayed while speaking; (c) automatic punctuation insertion; (d) support for English (default) and configurable target language; (e) on completion, display the transcript for user confirmation/edit; (f) map the note to an ESIP element type (default: Requirement or Decision, user-selectable); (g) assign to the designated target workspace/area (REQ-CV-009); (h) commit with one tap. The voice note SHALL also be stored as an audio attachment linked to the created element. The system SHALL use the device's on-device speech recognition engine where available (Apple Speech Framework, Android SpeechRecognizer) with cloud fallback.

**Acceptance Criteria:**
- Transcription begins within 1 s of tapping record
- Transcript accuracy ≥ 85% WER for clear speech in a quiet environment
- Element created in designated workspace within 3 s of commit tap
- On-device recognition used when available (no cloud call for basic dictation)

---

### REQ-MOB-008 — Meeting Audio Recording

| Field | Value |
|---|---|
| **ID** | REQ-MOB-008 |
| **Title** | Meeting Audio Recording and Transcription |
| **Category** | Functional |
| **Persona(s)** | P-PM, P-BA |
| **Priority** | S |
| **Rationale** | Capturing meeting audio and transcribing it allows requirements/decisions mentioned verbally to be surfaced for import post-meeting. |

**SHALL Description:** The mobile app SHALL provide a **Meeting Recording** mode: (a) record up to 4-hour audio sessions; (b) upload recording to the ESIP backend upon completion; (c) trigger asynchronous transcription via V06's speech-to-text service; (d) present the transcript (with speaker diarization if available) for review; (e) allow the user to highlight transcript segments and link them to ESIP element types (Requirement, Decision, Risk, Action Item); (f) linked segments are committed to the designated workspace as elements with the audio timestamp as provenance; (g) the recording SHALL be stored securely linked to the capture job with tenant-level access control. The app SHALL warn the user of legal recording consent requirements in applicable jurisdictions (static configurable disclaimer, not legal advice).

**Acceptance Criteria:**
- 60-minute recording uploads and transcription completes within 15 min
- Speaker diarization (if available) labels speakers A/B/C
- Linked transcript segments create ESIP elements on commit
- Recording stored with tenant-level access control (not globally accessible)

**Cross-reference:** V06 owns speech-to-text model infrastructure.

---

### REQ-MOB-009 — Offline Capture and Sync

| Field | Value |
|---|---|
| **ID** | REQ-MOB-009 |
| **Title** | Offline Capture with Automatic Sync |
| **Category** | Functional |
| **Persona(s)** | P-FIELD, P-MOB |
| **Priority** | M |
| **Rationale** | Client sites often have poor connectivity; the app must not block capture when offline. |

**SHALL Description:** The mobile app SHALL support offline capture mode: (a) all capture operations (photo, sketch, voice note, text requirement) SHALL be saveable to a local on-device queue when the device is offline or the network is unavailable; (b) the offline queue SHALL survive app restarts and device reboots (persisted to on-device SQLite/encrypted storage); (c) when network connectivity is restored, the queue SHALL sync automatically in the background; (d) sync SHALL be conflict-aware — if an offline capture targets an element that was modified online since the capture, the conflict SHALL be flagged in the sync report; (e) the user SHALL be able to see the pending sync queue and manually trigger sync; (f) offline mode indicator SHALL be clearly visible in the app UI; (g) workspace metadata (project list, recent elements) SHALL be cached for offline browsing (read-only).

**Acceptance Criteria:**
- 20 photos captured offline sync successfully on reconnection without data loss
- Offline indicator displayed within 2 s of network loss
- Sync queue visible and manually triggerable
- Sync conflict flagged with resolution options

---

### REQ-MOB-010 — Push Notifications

| Field | Value |
|---|---|
| **ID** | REQ-MOB-010 |
| **Title** | Mobile Push Notifications |
| **Category** | Functional |
| **Persona(s)** | P-MOB |
| **Priority** | M |
| **Rationale** | Timely notifications for import completions, CV job results, review requests, and approvals keep mobile users engaged and responsive. |

**SHALL Description:** The mobile app SHALL receive push notifications for: (a) CV capture job completed (with element count); (b) Import job completed (REQ-IMP-033); (c) Review request assigned to the user (V15 cross-reference); (d) Approval request awaiting action (V15); (e) Mention of the user in a workspace comment; (f) Sync conflict detected after offline reconnection. Notifications SHALL deep-link to the relevant workspace/job/review on tap. Notification preferences SHALL be configurable per category (iOS notification settings and in-app settings). Notifications SHALL use APNs (iOS) and FCM (Android). The system SHALL NOT send push notifications for routine system events (audit logs, batch processing unrelated to the user).

**Acceptance Criteria:**
- CV job completion notification arrives within 10 s of job completion
- Deep link opens correct workspace/job within 2 s
- Per-category notification toggle works and is respected
- Notification badge count matches unread notification count in-app

---

### REQ-MOB-011 — Real-Time Collaborative Mobile Whiteboard

| Field | Value |
|---|---|
| **ID** | REQ-MOB-011 |
| **Title** | Real-Time Collaborative Multi-User Mobile Whiteboard |
| **Category** | Functional |
| **Persona(s)** | P-MOB, P-PM, P-EA |
| **Priority** | S |
| **Rationale** | Architecture workshops involve multiple participants who should all be able to capture and sketch simultaneously, with changes visible to all in real time. |

**SHALL Description:** The mobile app SHALL support a **Collaborative Whiteboard Session** mode: (a) one user creates a session and shares an invite code or QR; (b) other participants join the session; (c) up to 20 concurrent participants; (d) all freehand drawing, added stickies, and typed elements are visible to all participants in real time via WebSocket/WebRTC; (e) each participant's cursor/pointer is visible with their name badge; (f) the collaborative whiteboard persists in the ESIP backend as a live canvas linked to a workspace; (g) at the end of the session, the facilitator triggers "Commit to model" which runs the full CV pipeline on the final composite canvas; (h) session replay: the backend records the sequence of drawing events for replay. Collaborative session SHALL gracefully handle participant joins/leaves and network interruptions (CRDT-based conflict-free data structure for canvas state).

**Acceptance Criteria:**
- Stroke from participant A appears on participant B's screen within 300 ms (LAN conditions)
- Up to 20 participants join without session degradation
- Session persists in backend on facilitator disconnect for 24 hours
- Commit-to-model triggers CV pipeline on final canvas state

---

### REQ-MOB-012 — Quick-Capture Requirements and Decisions

| Field | Value |
|---|---|
| **ID** | REQ-MOB-012 |
| **Title** | Mobile Quick-Capture for Requirements and Decisions |
| **Category** | Functional |
| **Persona(s)** | P-PM, P-BA, P-EA |
| **Priority** | M |
| **Rationale** | Requirements and architecture decisions are frequently captured ad-hoc in meetings; the mobile app must make structured capture as fast as possible. |

**SHALL Description:** The mobile app SHALL provide a **Quick-Capture** screen (accessible in ≤ 2 taps from home) with: (a) **Element type selector** — Requirement, Architecture Decision Record (ADR), Risk, Action Item, Note (default-remembers last used); (b) **Title field** — short text (autofocus, keyboard opens immediately); (c) **Description field** — optional longer text or voice dictation toggle; (d) **Priority picker** — MoSCoW or High/Medium/Low (configurable per workspace); (e) **Tag/label field** with autocomplete from workspace taxonomy; (f) **Target workspace/area** — shown and tappable to change; (g) **Submit** — single tap, creates element immediately (optimistic UI, sync in background); (h) **Batch mode** — after submitting, quick-capture screen resets for immediate next entry without returning to home. The quick-capture SHALL complete in ≤ 10 s end-to-end for a titled requirement.

**Acceptance Criteria:**
- Quick-capture accessible in ≤ 2 taps from home
- Title field autofocuses and keyboard opens within 500 ms
- Element created in designated workspace within 3 s of submit (online)
- Batch mode resets within 500 ms of previous submit

---

### REQ-MOB-013 — Synced Real-Time State with Desktop

| Field | Value |
|---|---|
| **ID** | REQ-MOB-013 |
| **Title** | Real-Time Sync Between Mobile App and Desktop |
| **Category** | Functional |
| **Persona(s)** | P-MOB |
| **Priority** | M |
| **Rationale** | A capture made on mobile must appear immediately in the desktop web app without requiring a manual refresh; the two surfaces must stay in sync. |

**SHALL Description:** The system SHALL maintain real-time bidirectional sync between the mobile app and the desktop web app for the same workspace: (a) elements created/modified/deleted on mobile SHALL appear in the desktop app within 2 s (via WebSocket push); (b) elements created/modified/deleted on the desktop SHALL appear in the mobile app within 2 s; (c) sync SHALL be selective — only workspaces currently open on the device receive push updates (other workspaces sync lazily on next open); (d) the mobile app SHALL display a "last synced" timestamp; (e) sync SHALL survive brief network interruptions (< 60 s) with automatic re-connection and state reconciliation.

**Acceptance Criteria:**
- Element created on mobile appears in desktop web app within 2 s
- Element created on desktop appears in mobile app within 2 s
- Network interruption of < 60 s recovered without data loss
- Last-synced timestamp updates in real time

**Cross-reference:** V11 owns the sync infrastructure (WebSocket hub, connection management, conflict resolution CRDT).

---

### REQ-MOB-014 — Mobile Security and Device Management

| Field | Value |
|---|---|
| **ID** | REQ-MOB-014 |
| **Title** | Mobile App Security and Device Management |
| **Category** | Security |
| **Persona(s)** | P-TA |
| **Priority** | M |
| **Rationale** | Enterprise mobile apps handling architecture IP must meet enterprise MDM and data protection standards. |

**SHALL Description:** The mobile app SHALL implement: (a) **Data at rest encryption** — all locally cached workspace data and offline queue encrypted using AES-256 with device-derived key (iOS Data Protection, Android EncryptedSharedPreferences / SQLCipher); (b) **Screen capture prevention** — FLAG_SECURE (Android) / allowScreenCapture(false) (iOS) applied to sensitive screens; (c) **App-level lock** — biometric or PIN lock after configurable idle timeout (default 5 min); (d) **Jailbreak/root detection** — warn user and optionally block access on jailbroken/rooted devices (Tenant Admin configurable); (e) **MDM policy compliance** — support Intune, VMware Workspace ONE, and Jamf MDM policies including remote wipe of app data; (f) **Certificate pinning** — pin the ESIP backend TLS certificate to prevent MITM; (g) **No data in clipboard** — sensitive model data not automatically copied to clipboard.

**Acceptance Criteria:**
- Local cached data encrypted: verified by inspecting SQLite file directly (opaque)
- Biometric lock activates after 5-min idle
- Remote wipe via Intune clears all app data within 60 s
- Certificate pinning fails connection on certificate mismatch

**Cross-reference:** V11 owns MDM integration and remote wipe infrastructure.

---

### REQ-MOB-015 — Mobile Review and Approval (Light)

| Field | Value |
|---|---|
| **ID** | REQ-MOB-015 |
| **Title** | Mobile Light Review and Approval Actions |
| **Category** | Functional |
| **Persona(s)** | P-MOB |
| **Priority** | S |
| **Rationale** | Reviewers and approvers need to be able to respond to review requests while away from their desk; basic approve/reject/comment on mobile reduces bottlenecks. |

**SHALL Description:** The mobile app SHALL support light review actions: (a) receive push notification for a review/approval request (REQ-MOB-010); (b) view the artifact under review (diagram thumbnail, requirement text, ADR summary) within the mobile app; (c) perform actions: Approve, Reject, Request Changes (with comment); (d) add inline comments on specific elements; (e) deep-link from email notification to the review screen in the app. Full review workflow (multi-reviewer, review boards, structured feedback templates) is owned by V15; this requirement covers the mobile action surface only.

**Acceptance Criteria:**
- Approve/Reject/RequestChanges actions available within 3 taps of opening notification
- Comment added and synced to desktop within 2 s
- Email deep-link opens correct review in mobile app

**Cross-reference:** V15 owns full review workflow; this is the mobile action surface only.

---

### REQ-MOB-016 — Accessibility on Mobile

| Field | Value |
|---|---|
| **ID** | REQ-MOB-016 |
| **Title** | Mobile App Accessibility |
| **Category** | UX |
| **Persona(s)** | P-MOB |
| **Priority** | S |
| **Rationale** | Enterprise apps must meet WCAG 2.1 AA and platform accessibility guidelines to ensure inclusive access. |

**SHALL Description:** The mobile app SHALL meet: (a) **iOS VoiceOver** — all interactive elements have accessible labels and traits; (b) **Android TalkBack** — all interactive elements have contentDescription; (c) **Dynamic text scaling** — UI scales correctly for all iOS text size settings and Android font scale settings up to 200%; (d) **Sufficient contrast** — all text ≥ 4.5:1 contrast ratio (WCAG 2.1 AA); (e) **Touch target size** — all tappable elements ≥ 44×44 pt; (f) **Alternative input** — keyboard/switch control navigation functional; (g) **No information conveyed by color alone** (confidence levels use both color AND icon). The camera/capture screen is exempt from VoiceOver navigation (inherently visual) but SHALL provide audio feedback (shutter sound, boundary detection beep).

**Acceptance Criteria:**
- Automated accessibility scan (axe-mobile or equivalent) passes with 0 critical violations
- VoiceOver navigates all primary flows on iOS
- Font scale 200% does not cause layout overflow

---

### REQ-MOB-017 — App Performance NFRs

| Field | Value |
|---|---|
| **ID** | REQ-MOB-017 |
| **Title** | Mobile App Performance Non-Functional Requirements |
| **Category** | NFR |
| **Persona(s)** | P-MOB |
| **Priority** | M |
| **Rationale** | Slow performance on mobile destroys adoption; the app must meet platform performance expectations. |

**SHALL Description:** The mobile app SHALL meet: (a) **Cold launch** — app fully interactive within 3 s on a mid-range device (2021-era CPU, 3 GB RAM); (b) **Warm launch** — app interactive within 1 s; (c) **Navigation transitions** — screen transitions complete within 300 ms; (d) **Camera viewfinder** — live camera preview at ≥ 30 fps; (e) **Drawing latency** — freehand stroke latency ≤ 16 ms (see REQ-MOB-006); (f) **Offline cache size** — workspace cache ≤ 100 MB on device (evict LRU beyond limit); (g) **Battery** — 30-min active capture session drains ≤ 15% battery on reference device; (h) **Memory** — app does not exceed 200 MB resident memory during normal use; (i) **Network efficiency** — sync delta updates only (not full model re-download).

**Acceptance Criteria:**
- Cold launch measured at ≤ 3 s on Samsung Galaxy A52 (reference mid-range)
- Camera viewfinder measured at ≥ 30 fps in viewfinder mode
- Battery drain measured at ≤ 15% over 30-min capture session

---

### REQ-MOB-018 — Cross-Platform PWA Fallback

| Field | Value |
|---|---|
| **ID** | REQ-MOB-018 |
| **Title** | Progressive Web App (PWA) Fallback |
| **Category** | Functional |
| **Persona(s)** | P-MOB |
| **Priority** | S |
| **Rationale** | Some enterprise MDM policies block third-party app installs; a PWA provides mobile capability without an App Store dependency. |

**SHALL Description:** The ESIP web application SHALL expose a PWA manifest and service worker meeting: (a) installable on mobile Chrome/Safari/Edge; (b) offline service worker caches app shell, workspace metadata, and last-viewed diagrams; (c) Web Camera API access for whiteboard capture (REQ-MOB-004 subset); (d) Web Speech API for voice dictation (REQ-MOB-007 subset) where supported; (e) push notifications via Web Push API; (f) PWA SHALL support all quick-capture features (REQ-MOB-012) and light review actions (REQ-MOB-015); (g) features not available in PWA (native biometrics, camera auto-detection, collaborative whiteboard at full fidelity) SHALL degrade gracefully with a "Get the native app" nudge.

**Acceptance Criteria:**
- PWA installable on Chrome for Android and Safari for iOS 16.4+
- Offline shell cached and readable without network
- Web Camera API capture functional in Chrome for Android
- Feature degradation messages clear and actionable

---

## Supplementary Requirements

### REQ-IMP-034 — Import API Rate Limiting and Throttling

| Field | Value |
|---|---|
| **ID** | REQ-IMP-034 |
| **Title** | Import API Rate Limiting |
| **Category** | NFR |
| **Persona(s)** | P-TA, P-DEV |
| **Priority** | M |

**SHALL Description:** Import API endpoints SHALL enforce per-tenant rate limits (default: 10 import jobs/hour, 100 MB/hour upload bandwidth). Limits SHALL be configurable per tenant. Rate limit exceeded SHALL return HTTP 429 with `Retry-After` header. Internal service-account API keys SHALL have elevated limits.

---

### REQ-IMP-035 — Tenant-Scoped Import Isolation

| Field | Value |
|---|---|
| **ID** | REQ-IMP-035 |
| **Title** | Multi-Tenant Import Isolation |
| **Category** | Security |
| **Persona(s)** | P-TA |
| **Priority** | M |

**SHALL Description:** Import jobs SHALL be fully isolated per tenant: uploaded files stored in tenant-scoped cloud storage paths (never cross-tenant accessible), import workers process jobs in tenant-isolated contexts, and no element from one tenant's import SHALL ever appear in another tenant's workspace. Uploaded files SHALL be deleted from intermediate storage within 24 hours of import job completion.

---

### REQ-IMP-036 — Import Configuration Export

| Field | Value |
|---|---|
| **ID** | REQ-IMP-036 |
| **Title** | Import Configuration Portability |
| **Category** | Governance |
| **Persona(s)** | P-TA |
| **Priority** | C |

**SHALL Description:** Tenant Admins SHALL be able to export all import configuration (format whitelist, mapping templates, migration accelerator customizations) as a JSON bundle for backup, audit, and migration between ESIP tenants. The bundle SHALL be importable to a new tenant without manual re-configuration.

---

### REQ-CV-015 — CV Pipeline Observability

| Field | Value |
|---|---|
| **ID** | REQ-CV-015 |
| **Title** | CV Pipeline Observability and Quality Metrics |
| **Category** | Reporting |
| **Persona(s)** | P-TA |
| **Priority** | S |

**SHALL Description:** The system SHALL expose CV pipeline quality metrics per tenant: mean detection precision, mean OCR confidence, acceptance rate (elements committed vs. detected), user edit rate (how often users correct CV output). These metrics SHALL be available in the Tenant Admin dashboard and exportable as CSV.

**Cross-reference:** V06 owns model performance monitoring.

---

### REQ-MOB-019 — Mobile Analytics and Crash Reporting

| Field | Value |
|---|---|
| **ID** | REQ-MOB-019 |
| **Title** | Mobile App Analytics and Crash Reporting |
| **Category** | NFR |
| **Persona(s)** | P-TA |
| **Priority** | S |

**SHALL Description:** The mobile app SHALL integrate: (a) crash reporting (Sentry or equivalent) for automatic crash report collection; (b) anonymized usage analytics (screen views, feature usage frequencies) for product improvement; (c) opt-out mechanism for analytics collection respecting GDPR/CCPA; (d) no capture of model content in analytics events (PII/IP-sensitive).

---

### REQ-MOB-020 — Mobile OS Update Policy

| Field | Value |
|---|---|
| **ID** | REQ-MOB-020 |
| **Title** | Mobile App Version and OS Update Policy |
| **Category** | Governance |
| **Persona(s)** | P-TA |
| **Priority** | C |

**SHALL Description:** The mobile app SHALL enforce a minimum version policy: Tenant Admin configures a minimum app version; users running older versions SHALL see a mandatory update prompt on launch and SHALL NOT be able to capture and sync until updated. The minimum supported OS versions (iOS 16+, Android 12+) SHALL be re-evaluated annually and updated with ≥ 6-month deprecation notice.

---

## Requirement Summary Matrix

| Sub-Area | Prefix | Count | M | S | C |
|---|---|---|---|---|---|
| Import / Interoperability | REQ-IMP | 36 | 21 | 9 | 6 |
| Computer-Vision Capture | REQ-CV | 15 | 9 | 5 | 1 |
| Mobile App | REQ-MOB | 20 | 10 | 8 | 2 |
| **Total** | | **71** | **40** | **22** | **9** |

---

## Boundary and Assumption Notes

1. **V03 (maxGraph)** receives normalized coordinate data from import adapters (x, y, w, h in canvas units) and renders the canvas. This volume specifies the data contract; V03 specifies the rendering behavior.
2. **V06 (AI)** hosts the CV/OCR models and speech-to-text models, manages GPU infrastructure, and owns model versioning. This volume specifies the API contract (inputs, outputs, confidence format) and the UX pipeline; V06 owns the model implementation.
3. **V08 (Connectors)** owns live bidirectional connectors (e.g., live Miro sync). This volume owns the import mechanic from Miro exports and one-time API-fetch imports.
4. **V11 (Platform)** owns the identity federation, MDM integration, WebSocket hub, cloud storage infrastructure, and sync conflict resolution algorithms. This volume owns the mobile app's consumption of those services.
5. **V15 (Review)** owns the full review workflow, approval boards, and structured feedback. This volume owns the mobile action surface (light approve/reject/comment).
6. **V16 (Intelligence)** owns competitive migration accelerator intelligence, migration analytics, and recommendation engines. This volume owns the import mechanics and the template library that V16 consumes.
7. **Sparx EA XMI** — the `.eap` format (pre-2007 Access MDB) is explicitly out of scope; only `.eapx`/`.qea` (SQLite) and XMI exports are supported.
8. **LeanIX, Ardoq, MEGA HOPEX, ABACUS, BiZZdesign** API import paths require user-supplied API credentials; ESIP does not store third-party SaaS admin credentials beyond the duration of the import session (OAuth tokens managed per V11).
9. **Legal / Jurisdiction** — REQ-MOB-008 (meeting recording) includes only a configurable consent disclaimer. ESIP does not provide legal advice; enterprise customers are responsible for compliance with local recording consent laws.
