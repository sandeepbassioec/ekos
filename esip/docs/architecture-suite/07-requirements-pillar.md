# Volume 07 — Requirements Pillar
## ESIP Architecture Automation Suite

| Attribute | Value |
|---|---|
| Volume | 07 |
| Prefix | REQM |
| Status | Baseline Draft |
| Date | 2026-06-23 |
| Scope | Multi-format document ingestion, parsing & normalization; extraction-candidate lifecycle management; requirements data model & schema; quality & hygiene checks; traceability matrix; baselining, versioning & change control; collaboration, governance & reuse; dashboards & reporting exports |
| Out of Scope | AI extraction reasoning & prompt logic (V06); source artifact storage, connectors & version control (V08); deliverable document rendering (V10); diagram rendering engine (V03); platform infrastructure, secrets, identity (V11); portfolio & roadmap items (V09) |

---

## Personas Reference

| Code | Persona |
|---|---|
| EA | Enterprise Architect |
| SA | Solution Architect |
| AA | Application Architect |
| BA | Business Analyst / Requirements Engineer |
| PM | Product Manager |
| DA | Data Architect |
| IA | Integration/API Architect |
| SecA | Security Architect |
| QA | Quality Assurance Engineer |
| GVN | Governance / Compliance Officer |
| STK | Business Stakeholder (Approver/Reviewer) |
| ALL | All personas |

---

## Table of Contents

1. [Multi-Format Ingestion — File Types & Upload](#1-multi-format-ingestion--file-types--upload)
2. [Ingestion Pipeline — Parsing, Normalization & Metadata](#2-ingestion-pipeline--parsing-normalization--metadata)
3. [Format-Specific Extractors](#3-format-specific-extractors)
4. [Large-File, Batch & Bulk Handling](#4-large-file-batch--bulk-handling)
5. [Deduplication & Source Management](#5-deduplication--source-management)
6. [Ingestion Status, Progress & Error Handling](#6-ingestion-status-progress--error-handling)
7. [Extraction-Candidate Lifecycle](#7-extraction-candidate-lifecycle)
8. [Requirement Data Model & Schema](#8-requirement-data-model--schema)
9. [Requirement Types & Hierarchy](#9-requirement-types--hierarchy)
10. [User Stories & Acceptance Criteria](#10-user-stories--acceptance-criteria)
11. [MoSCoW, Priority & Status Lifecycle](#11-moscow-priority--status-lifecycle)
12. [Custom Fields, Tags & Categories](#12-custom-fields-tags--categories)
13. [Quality & Hygiene Checks](#13-quality--hygiene-checks)
14. [Deduplication, Merge & Conflict Detection](#14-deduplication-merge--conflict-detection)
15. [Completeness & Coverage Scoring](#15-completeness--coverage-scoring)
16. [Requirements Traceability Matrix](#16-requirements-traceability-matrix)
17. [Forward & Backward Traceability](#17-forward--backward-traceability)
18. [Coverage, Orphan & Gap Analysis](#18-coverage-orphan--gap-analysis)
19. [Impact Analysis on Change](#19-impact-analysis-on-change)
20. [Baselining & Versioning](#20-baselining--versioning)
21. [Change Requests & Approval Workflows](#21-change-requests--approval-workflows)
22. [Change History, Diff & Audit](#22-change-history-diff--audit)
23. [E-Sign & Formal Sign-Off](#23-e-sign--formal-sign-off)
24. [Collaboration & Review](#24-collaboration--review)
25. [Stakeholder Management & RACI](#25-stakeholder-management--raci)
26. [BABOK Alignment & Business Glossary](#26-babok-alignment--business-glossary)
27. [Requirement Reuse Library](#27-requirement-reuse-library)
28. [Prioritization Frameworks](#28-prioritization-frameworks)
29. [Dashboards & Reporting](#29-dashboards--reporting)
30. [Exports & Tool Integration](#30-exports--tool-integration)

---

## 1. Multi-Format Ingestion — File Types & Upload

### 1.1 Document & Office Format Ingestion

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-001 | Word Document Ingestion (.docx) | The system SHALL ingest Microsoft Word documents in .docx format, preserving heading hierarchy (H1–H6), paragraph body, inline styles, tables, numbered/bulleted lists, footnotes, endnotes, tracked-change blocks, comments, and embedded images | Functional | BA, SA, EA | Must | Primary source format for requirement specs; ISO 29148 §5.2 |
| REQ-REQM-002 | Legacy Word Document Ingestion (.doc) | The system SHALL ingest Microsoft Word 97-2003 documents in binary .doc format via server-side conversion to OOXML prior to parsing, handling compound binary format quirks and maintaining structural fidelity | Functional | BA, SA | Must | Legacy enterprise archives commonly in .doc |
| REQ-REQM-003 | PDF Ingestion — Native Text | The system SHALL ingest PDF files that contain native (selectable) text by extracting text content with positional metadata, preserving reading order, column detection, table boundaries, headers/footers, and bookmarks/outline structure | Functional | BA, SA, EA | Must | PDFs are primary deliverable format for RFPs, SLAs, contracts |
| REQ-REQM-004 | PDF Ingestion — Scanned / Image-Only (OCR) | The system SHALL ingest scanned PDF files and image-only PDFs by applying OCR (optical character recognition) achieving ≥95% character-level accuracy on standard business documents at 300 DPI or higher; deskew, despeckle, and orientation correction SHALL be applied automatically | AI/ML | BA, EA | Must | Many legacy requirements exist only as scanned documents |
| REQ-REQM-005 | Mixed PDF Ingestion | The system SHALL detect and handle mixed PDFs containing both native text pages and scanned image pages, applying OCR selectively per page and merging output into a unified document representation | Functional | BA | Must | Common in real-world document archives |
| REQ-REQM-006 | Excel Workbook Ingestion (.xlsx) | The system SHALL ingest Microsoft Excel .xlsx workbooks extracting all worksheets, named ranges, tables, cell values, formulas (as text), merged cells, and sheet metadata; multi-sheet workbooks SHALL be treated as logically related sub-documents | Functional | BA, DA, PM | Must | Requirements often tracked in Excel matrices (DOORS-light patterns) |
| REQ-REQM-007 | CSV File Ingestion | The system SHALL ingest CSV files with auto-detection of delimiter (comma, semicolon, tab, pipe), encoding (UTF-8, UTF-16, ISO-8859-1), and header row presence; malformed rows SHALL be flagged with row number and error description | Functional | BA, DA | Must | Common export from issue trackers and legacy RM tools |
| REQ-REQM-008 | PowerPoint Ingestion (.pptx) | The system SHALL ingest Microsoft PowerPoint .pptx files extracting per-slide: title, body text, speaker notes, text boxes, table content, and embedded images; slide sequence SHALL be preserved as narrative ordering signal | Functional | BA, SA, PM | Must | Architecture and project kick-off decks frequently contain embedded requirements |
| REQ-REQM-009 | Visio Diagram Ingestion (.vsdx) | The system SHALL ingest Microsoft Visio .vsdx files, extracting shape labels, connector labels, shape types (via master shape names), groupings, page metadata, and shape data/property rows; the output SHALL be a labelled graph structure | Functional | EA, SA, IA | Should | Process and data-flow diagrams contain implicit requirements |
| REQ-REQM-010 | Image File Ingestion | The system SHALL ingest raster images (PNG, JPEG, TIFF, BMP, GIF) and apply OCR + object detection to extract embedded text, table structures visible in the image, and diagram labels; minimum supported resolution 150 DPI | AI/ML | BA, EA | Should | Whiteboard photos, UI mockups, and scanned diagrams are common requirement sources |

**Acceptance Criteria (REQ-REQM-001–010):**
- AC1: A 200-page Word document with 15 heading levels is ingested and produces a hierarchical section tree with zero missing headings
- AC2: OCR on a standard 300 DPI A4 scanned page achieves ≥95% character accuracy vs. ground truth
- AC3: An Excel workbook with 10 sheets and 500 rows is fully ingested with all sheet names and row counts preserved

---

### 1.2 Communication & Collaboration Format Ingestion

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-011 | Email Ingestion (.eml Format) | The system SHALL ingest RFC-5322 .eml email files, extracting From/To/CC/BCC, subject, date, plain-text and HTML body, inline images, and attachment metadata; threaded conversations SHALL be reconstructed by In-Reply-To and References headers | Functional | BA, STK | Should | Requirement decisions are frequently communicated via email |
| REQ-REQM-012 | Outlook Message Ingestion (.msg Format) | The system SHALL ingest Microsoft Outlook .msg (MAPI compound document) files with equivalent field extraction to REQ-REQM-011, handling OLE storage format via server-side conversion | Functional | BA, STK | Should | Enterprise users primarily use .msg attachments |
| REQ-REQM-013 | Microsoft Graph Email Ingestion | The system SHALL ingest emails directly from Microsoft Exchange/M365 mailboxes via the Microsoft Graph API (GET /me/mailFolders/{id}/messages), using OAuth 2.0 delegated or application permission flows, with folder selection, date-range filtering, and unread/flagged filters | Integration | BA, STK | Should | Eliminates manual export step; aligns with M365 enterprise deployments |
| REQ-REQM-014 | Microsoft Teams Transcript Ingestion | The system SHALL ingest Teams meeting transcripts (VTT and JSON formats exported from Teams or retrieved via Graph API), attributing each utterance to speaker display name and timestamp, and segmenting transcript by topic-change heuristic | Integration | BA, STK | Should | Post-meeting transcripts are a primary source of decision capture |
| REQ-REQM-015 | Zoom Transcript Ingestion | The system SHALL ingest Zoom meeting transcripts in VTT format, extracting speaker segments with timestamps and reconstructing conversation turns; transcripts downloaded manually via file upload SHALL be supported | Functional | BA, STK | Could | Zoom is common in cross-org collaboration |
| REQ-REQM-016 | Teams Chat Export Ingestion | The system SHALL ingest Microsoft Teams channel and chat message exports (JSON format from Teams Compliance Export or eDiscovery), extracting message body, sender, timestamp, thread ID, reaction summary, and attached file references | Integration | BA, STK | Should | Channel discussions contain informal requirements and decisions |
| REQ-REQM-017 | Slack Export Ingestion | The system SHALL ingest Slack workspace export archives (ZIP containing per-channel JSON message files), reconstructing thread hierarchies, extracting message text, sender, timestamp, emoji reactions, and file attachment references | Functional | BA, PM | Should | Many product teams use Slack for requirements discussion |

**Dependencies:** REQ-REQM-013 depends on V11 OAuth/Graph connector; REQ-REQM-014 depends on V08 connector framework.

---

### 1.3 Issue Tracker & Wiki Ingestion

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-018 | Jira Issue Ingestion (API) | The system SHALL ingest Jira issues via the Jira REST API v3, extracting issue key, summary, description (Atlassian Document Format), issue type, status, priority, labels, components, fix versions, acceptance criteria (custom field), linked issues, and comments; pagination SHALL handle projects with >10,000 issues | Integration | BA, PM | Must | Jira is the dominant issue tracker; requirements often originate there |
| REQ-REQM-019 | Jira Export Ingestion (JSON/XML) | The system SHALL ingest Jira bulk export files in JSON and XML formats as a fallback for environments without API access, with equivalent field extraction to REQ-REQM-018 | Functional | BA, PM | Must | Air-gapped or restricted environments require file-based import |
| REQ-REQM-020 | Azure DevOps Work Item Ingestion (API) | The system SHALL ingest Azure DevOps (ADO) work items via the ADO REST API, extracting ID, title, description, work item type, state, iteration path, area path, acceptance criteria, tags, relations, and attachments; supports ADO Cloud and ADO Server ≥2019 | Integration | BA, PM | Must | ADO is the primary ALM tool in Microsoft-centric enterprises |
| REQ-REQM-021 | ADO Work Item Export Ingestion | The system SHALL ingest ADO bulk exports in JSON, CSV, and XLSX formats as a fallback, with equivalent field extraction to REQ-REQM-020 | Functional | BA, PM | Should | Fallback for restricted API environments |
| REQ-REQM-022 | Confluence Page Ingestion (API) | The system SHALL ingest Confluence pages via the Confluence REST API v2, extracting page title, body (Storage Format HTML), space key, labels, version, author, created/modified dates, and child page hierarchy | Integration | BA, EA | Should | Confluence is the primary wiki for requirements documentation in Atlassian shops |
| REQ-REQM-023 | Confluence Export Ingestion (HTML/PDF) | The system SHALL ingest Confluence space/page exports in HTML and PDF formats, reconstructing page hierarchy from navigation structure and extracting equivalent content to REQ-REQM-022 | Functional | BA | Should | Backup ingestion path |
| REQ-REQM-024 | SharePoint Page Ingestion (Graph API) | The system SHALL ingest SharePoint Online pages and document library files via the Microsoft Graph API, extracting page content, document content, metadata, version, author, and list item properties | Integration | BA, EA | Should | Many enterprises use SharePoint as requirements wiki |
| REQ-REQM-025 | ServiceNow Story/Requirement Ingestion | The system SHALL ingest ServiceNow Agile Development stories and requirements via the ServiceNow Table API (REST), extracting short description, description, acceptance criteria, state, priority, and related stories | Integration | BA, PM | Could | ServiceNow ITSM shops may track requirements there |

---

### 1.4 Drag-Drop & Bulk Upload UX

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-026 | Drag-and-Drop Single File Upload | The system SHALL support drag-and-drop upload of a single file from the user's local filesystem onto a designated drop zone in the web UI, supporting all formats listed in §1.1–§1.2; file type validation SHALL occur client-side before upload begins | UX | ALL | Must | Core productivity requirement; industry-standard UX pattern |
| REQ-REQM-027 | Drag-and-Drop Multi-File Upload | The system SHALL allow dragging and dropping multiple files simultaneously (up to 50 files per drop operation) with each file processed as an independent ingestion job and a per-file status indicator rendered in a queue panel | UX | BA, PM | Must | Bulk uploads are the common pattern when onboarding a new project |
| REQ-REQM-028 | File Browser Upload Dialog | The system SHALL provide a "Browse Files" button that opens the OS file picker, supporting multi-select, alongside the drag-and-drop zone | UX | ALL | Must | Accessibility fallback for keyboard/screen-reader users |
| REQ-REQM-029 | Folder Upload | The system SHALL support upload of an entire folder (via the HTML File System Access API or equivalent), recursively discovering files of supported types, presenting a preview tree of discovered files before confirming ingestion | UX | BA, PM | Should | Project archives are typically organized in folders |
| REQ-REQM-030 | Bulk Upload via URL List | The system SHALL accept a plain-text list of URLs (one per line, up to 500 URLs) and download-then-ingest each URL, with per-URL status tracking; supported protocols: HTTPS, SharePoint direct links, Confluence page URLs | Functional | BA | Should | Enables scripted or pipeline-driven bulk onboarding |
| REQ-REQM-031 | Upload Progress Indicator | The system SHALL display a real-time upload progress bar per file showing bytes transferred / total bytes, elapsed time, and estimated remaining time; progress SHALL be computed server-side for chunked uploads and pushed to the client via WebSocket or SSE | UX | ALL | Must | Standard UX expectation; reduces abandonment |
| REQ-REQM-032 | Upload Cancellation | The system SHALL allow the user to cancel an in-progress upload at any time; partial uploads SHALL be discarded server-side and no ingestion job SHALL be created for a cancelled upload | UX | ALL | Must | Data hygiene; avoids ghost ingestion jobs |

---

## 2. Ingestion Pipeline — Parsing, Normalization & Metadata

### 2.1 Parsing & Structural Normalization

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-033 | Universal Canonical Document Model | The system SHALL normalize all ingested artifacts into a canonical internal document model (CDM) comprising: document header (title, author, date, source URL, file hash), hierarchical section tree (level, heading text, body blocks), and an ordered list of content blocks (paragraph, table, list, image, code) | Data | ALL | Must | Enables format-agnostic downstream processing (AI, RTM, search) |
| REQ-REQM-034 | Heading Hierarchy Extraction | The system SHALL extract heading hierarchy to a minimum of 6 levels from all office formats (Word styles H1–H6, PDF bookmarks, HTML h1–h6, Confluence headings), mapping each heading to a CDM section node with level, text, and character offset | Data | BA, EA | Must | Structural context is essential for traceability source spans |
| REQ-REQM-035 | Table Extraction & Normalization | The system SHALL extract tables from all supported formats and represent each as a CDM table block with: column headers (first-row or explicit header detection), row data as typed cell values (text, number, date, boolean), row count, column count, and source position | Data | BA, DA | Must | Requirements matrices, acceptance criteria grids, and RTMs are table-heavy |
| REQ-REQM-036 | List & Enumeration Extraction | The system SHALL extract numbered and bulleted lists (nested to ≥4 levels) and represent them as CDM list blocks preserving nesting depth, bullet/numbering style, and parent heading association | Data | BA | Must | Requirements are frequently embedded in lists |
| REQ-REQM-037 | Metadata Extraction | The system SHALL extract document-level metadata from file format properties including: title, author, organization, creation date, last-modified date, revision number, subject, keywords, and application name; missing fields SHALL be marked null rather than inferred | Data | ALL | Must | Provenance and governance require authoritative metadata |
| REQ-REQM-038 | Language Detection | The system SHALL detect the primary written language of each ingested document (ISO 639-1 code) using character n-gram statistics; multi-language documents SHALL be detected at section level with the dominant language flagged at document level | Data | ALL | Should | Global enterprises operate in multiple languages; AI prompting must match language |
| REQ-REQM-039 | Encoding Normalization | The system SHALL normalize all extracted text to UTF-8 before storing in the CDM, converting from detected source encodings (ISO-8859-1, Windows-1252, UTF-16, etc.) with lossy-character substitution logged as a warning | Data | ALL | Must | Prevents downstream character-set corruption |
| REQ-REQM-040 | Section-Level Provenance Annotation | The system SHALL annotate every CDM content block with source provenance: file name, page number (or slide/sheet index), character start/end offset within the page, and bounding-box coordinates (pixels) where available from PDF/image sources | Data | ALL | Must | Provenance is the foundation of REQ-REQM traceability (§16) |

---

### 2.2 Content Enrichment & Metadata Augmentation

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-041 | File Hash & Fingerprint | The system SHALL compute a SHA-256 hash of every uploaded file at ingest time and store it alongside the source record; the hash SHALL be used for deduplication (§5) and integrity verification | Security | ALL | Must | Ensures no duplicate ingestion of the same file; integrity baseline |
| REQ-REQM-042 | MIME-Type Detection | The system SHALL detect the actual MIME type of uploaded files using magic-byte inspection (not solely relying on file extension), rejecting files whose detected MIME type does not match any supported type with a clear error message | Security | ALL | Must | Prevents malicious file disguised as a supported type |
| REQ-REQM-043 | Virus / Malware Scanning | The system SHALL pass every uploaded file through an integrated antivirus/anti-malware scanner before parsing; infected files SHALL be quarantined, not parsed, and the uploader SHALL be notified with a quarantine reference ID | Security | ALL | Must | Enterprise security baseline; protects shared knowledge store |
| REQ-REQM-044 | CDM Schema Versioning | The canonical document model schema SHALL be versioned (major.minor); stored CDM records SHALL carry the schema version at which they were produced; a migration service SHALL re-normalize CDM records when the schema is upgraded | Data | ALL | Should | Long-lived repositories require schema evolution without data loss |
| REQ-REQM-045 | Thumbnail Generation | The system SHALL generate a 200×200 px thumbnail image for the first page/slide of each ingested document and store it as a display asset for the ingestion source list UI | UX | ALL | Could | Improves source identification in large libraries |

---

## 3. Format-Specific Extractors

### 3.1 Word / OOXML Extractor

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM3-001 | Word Tracked Changes Handling | The Word extractor SHALL detect tracked changes (insertions, deletions, format changes) and offer three ingestion modes: (a) Accept All — use final text, (b) Reject All — use original text, (c) Preserve Both — emit dual CDM blocks with change type annotations; default SHALL be Accept All | Functional | BA, GVN | Must | Requirements documents frequently arrive with tracked changes from review cycles |
| REQ-REQM3-002 | Word Comment Extraction | The Word extractor SHALL extract all comments (author, date, anchor text, comment body) as CDM annotation blocks associated with the parent paragraph or table cell | Functional | BA, STK | Must | Comments often contain reviewer requirements or objections |
| REQ-REQM3-003 | Word Styles Mapping | The Word extractor SHALL map custom named styles to semantic categories using a configurable style-map (e.g., "Requirement" style → CDM requirement-hint block; "Assumption" style → CDM assumption-hint block); built-in heading styles SHALL always be recognized | Functional | BA | Should | Organizations use custom Word templates with requirement-specific styles |
| REQ-REQM3-004 | Word Table of Contents Extraction | The Word extractor SHALL parse a Word Table of Contents field and cross-reference it with extracted headings to validate structural completeness; discrepancies SHALL be flagged as parsing warnings | Functional | BA | Could | TOC validation confirms extraction fidelity |
| REQ-REQM3-005 | Word Embedded Objects | The Word extractor SHALL detect and catalogue embedded OLE objects (Excel workbook, Visio diagram, PDF) within a .docx file, trigger the appropriate sub-extractor per object type, and link the sub-extracted CDM to the parent document's CDM via an embed reference | Functional | BA, DA | Should | Complex requirement documents embed spreadsheets or diagrams |

---

### 3.2 PDF Extractor

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM3-006 | PDF Structural Hierarchy Recovery | The PDF extractor SHALL recover logical document structure from visual layout heuristics (font size, bold, position) when the PDF lacks a Tagged PDF structure tree, constructing a best-effort heading hierarchy with confidence scores | Functional | BA | Must | Most scanned or print-to-PDF documents lack structural tags |
| REQ-REQM3-007 | PDF Table Detection | The PDF extractor SHALL detect tabular structures from PDF page layout using whitespace analysis and/or ML-based table detection, extracting cell text with column/row coordinates; overlapping or merged-cell tables SHALL be flagged | Functional | BA, DA | Must | Requirement matrices and RTMs are often PDF tables |
| REQ-REQM3-008 | PDF Annotation Extraction | The PDF extractor SHALL extract PDF annotations (sticky notes, highlights, free text, ink annotations) and represent them as CDM annotation blocks with anchor coordinates and author metadata | Functional | BA, STK | Should | Reviewers annotate PDFs with requirement comments |
| REQ-REQM3-009 | PDF Form Field Extraction | The PDF extractor SHALL detect and extract AcroForm and XFA form fields (field name, field type, value, required flag) from interactive PDF forms, representing them as structured CDM fields | Functional | BA | Could | Some organizations use PDF forms as requirements templates |
| REQ-REQM3-010 | Multi-Column Layout Detection | The PDF extractor SHALL detect multi-column page layouts (2- and 3-column) and reconstruct correct reading order across columns before emitting CDM paragraph blocks | Functional | BA | Must | Academic-style requirements documents and RFPs use multi-column layouts |

---

### 3.3 Excel / CSV Extractor

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM3-011 | Excel Requirement Matrix Detection | The Excel extractor SHALL heuristically identify worksheets that represent requirements matrices by pattern-matching column headers against a configurable vocabulary (e.g., "ID", "Requirement", "Priority", "Status", "Category", "Acceptance Criteria", "Source", "Owner") and auto-map detected columns to CDM requirement fields | Functional | BA, PM | Must | Excel-based requirement matrices are extremely common in practice |
| REQ-REQM3-012 | Excel Formula Preservation | The Excel extractor SHALL store formula strings (as text) alongside computed cell values so that business-rule formulas (e.g., score calculations, priority formulas) are visible to downstream AI extraction | Data | DA | Should | Formulas encode business logic that informs NFR and constraint extraction |
| REQ-REQM3-013 | Excel Named Range Extraction | The Excel extractor SHALL extract all named ranges with their name, sheet reference, and resolved cell values, storing them as CDM metadata entities | Data | DA | Could | Named ranges often define domain enumeration lists |
| REQ-REQM3-014 | CSV Encoding & Delimiter Auto-Detection | The CSV extractor SHALL auto-detect encoding (BOM-based then heuristic) and delimiter (scoring comma, semicolon, tab, pipe against row consistency) before parsing; the detected encoding and delimiter SHALL be reported in ingestion metadata and overridable by the user | Functional | BA, DA | Must | Incorrect delimiter detection is the most common CSV ingestion failure mode |

---

### 3.4 PowerPoint Extractor

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM3-015 | Slide Speaker Notes as Requirement Candidates | The PowerPoint extractor SHALL treat slide speaker notes as high-priority requirement candidate text, annotating them with slide number and presenting them distinctly from slide body text in the CDM | Functional | BA, PM | Must | Product managers embed detailed requirements in speaker notes |
| REQ-REQM3-016 | Slide Layout Classification | The PowerPoint extractor SHALL classify each slide by layout type (title slide, section header, content, two-content, comparison, blank) and emit layout type as a CDM attribute to assist AI extraction in weighting content blocks | Functional | BA | Should | Layout type signals whether content is introductory vs. substantive |
| REQ-REQM3-017 | Embedded Video/Audio Transcript Ingestion | The PowerPoint extractor SHALL detect embedded video and audio files; if a transcript file (VTT/SRT) is embedded or co-located, it SHALL be extracted and associated with the slide; direct audio transcription is deferred to V06 | Functional | BA | Could | Training and demo decks may contain recorded narration |

---

### 3.5 Visio Extractor

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM3-018 | Visio Shape Data Row Extraction | The Visio extractor SHALL extract Shape Data (custom property rows) from each shape in addition to label text, mapping property names and values into CDM node attributes; this enables extraction of requirements embedded in shape metadata | Functional | EA, IA | Should | Visio shapes in architecture diagrams often carry metadata (e.g., "Requirement ID") |
| REQ-REQM3-019 | Visio Page-Level Extraction | The Visio extractor SHALL extract each Visio page as a separate CDM sub-document, preserving page name, page index, and background layer assignments | Functional | EA | Should | Multi-page Visio files represent different architecture views |
| REQ-REQM3-020 | Visio Hyperlink Extraction | The Visio extractor SHALL extract hyperlinks attached to shapes and emit them as CDM cross-reference attributes, enabling downstream linking to related source artifacts | Functional | EA | Could | Visio diagrams may hyperlink to requirement documents |

---

### 3.6 Communication Format Extractors (Email / Chat / Transcript)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM3-021 | Email Thread Reconstruction | The email extractor SHALL reconstruct email conversation threads from In-Reply-To and References headers, ordering messages chronologically within threads and representing the thread as a CDM conversation with child message nodes | Functional | BA, STK | Should | Decision context requires full thread visibility |
| REQ-REQM3-022 | Email Attachment Association | The email extractor SHALL inventory file attachments in each email message, extract their names/MIME types/sizes, and enqueue each supported-type attachment as an independent ingestion job linked to the parent email message | Functional | BA | Should | Requirements documents are frequently email attachments |
| REQ-REQM3-023 | Transcript Speaker Diarization Normalization | The transcript extractor SHALL normalize speaker labels from VTT/JSON transcripts (which may be "Speaker 1", "unknown", or raw display names) by resolving them against a project member roster (if available) and presenting unresolved speakers for manual disambiguation | Functional | BA, STK | Should | Correct attribution is required for governance and RACI alignment |
| REQ-REQM3-024 | Chat Message Reaction Extraction | The Teams and Slack chat extractors SHALL extract emoji reactions on messages and surface aggregate reaction counts (thumbs-up, thumbs-down, etc.) as CDM sentiment metadata, surfaced in the extraction review UI as weak approval/rejection signals | Functional | BA, PM | Could | Reactions signal stakeholder consensus or disagreement on proposed requirements |

---

## 4. Large-File, Batch & Bulk Handling

### 4.1 Large-File Handling

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-046 | Chunked Upload Protocol | The system SHALL implement a chunked (resumable) upload protocol for files exceeding 10 MB, splitting files into 5 MB chunks, and supporting resume from the last successfully uploaded chunk on connection interruption; the protocol SHALL be compatible with TUS open specification or equivalent | Functional | ALL | Must | Large PDFs and Excel workbooks frequently exceed 50–100 MB |
| REQ-REQM-047 | Maximum Single-File Size | The system SHALL support individual file ingestion up to 2 GB without server-side memory-mapping the entire file; streaming parse pipelines SHALL be used for PDF, Office formats, and ZIP archives | NFR | ALL | Must | Large CAD-embedded Visio or multi-hundred-page PDF documents exist |
| REQ-REQM-048 | Large Document Pagination in Processing | The system SHALL process large documents (>500 pages or >200,000 words) in streaming page-by-page or section-by-section windows to avoid memory exhaustion; partial CDM results SHALL be committed incrementally so that a parse failure on page N does not lose pages 1–N-1 | NFR | ALL | Must | Resilience for very large requirements specifications |
| REQ-REQM-049 | Large Excel Workbook Streaming | The system SHALL stream-read .xlsx files using the Open XML streaming API (SAX-mode), avoiding full DOM load, enabling processing of workbooks with >1 million rows without OOM errors | NFR | DA | Must | Requirement trace matrices on large programs can exceed 100k rows |

---

### 4.2 Batch & Queue Management

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-050 | Ingestion Job Queue | The system SHALL maintain a persistent, durable ingestion job queue (backed by a message broker) where each uploaded file or API-fetched artifact creates an independent job; jobs SHALL survive server restarts | NFR | ALL | Must | Reliability requirement for enterprise batch ingestion |
| REQ-REQM-051 | Parallel Ingestion Workers | The system SHALL support configurable parallel ingestion worker pools (default: 4 workers per tenant), processing queue jobs concurrently to minimize total batch-import wall-clock time | NFR | ALL | Must | Batch onboarding of 500-file project archives must complete in reasonable time |
| REQ-REQM-052 | Batch Import Scheduling | The system SHALL allow administrators to schedule recurring batch imports from connected sources (Jira, Confluence, SharePoint, ADO) on a cron-style schedule (minimum frequency: hourly), with last-run time and next-run time visible in the admin UI | Functional | GVN, BA | Should | Keeps the knowledge base current with upstream changes |
| REQ-REQM-053 | Batch Import Dry-Run | The system SHALL offer a dry-run mode for batch imports that reports how many files would be ingested, how many skipped (duplicates), and estimated processing time, without persisting any data | Functional | BA, GVN | Should | Risk-reduction before large imports |
| REQ-REQM-054 | Priority Queue Lanes | The system SHALL provide high-priority and standard-priority queue lanes; files uploaded interactively by a user SHALL be placed in the high-priority lane and processed before background batch-import jobs | NFR | ALL | Should | Interactive users should not wait behind batch jobs |

---

## 5. Deduplication & Source Management

### 5.1 Source Deduplication

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-055 | Exact-File Deduplication | The system SHALL compare the SHA-256 hash of each uploaded file against all previously ingested source files in the same project; an exact hash match SHALL be flagged as a duplicate and the user SHALL be prompted to (a) skip, (b) replace the existing version, or (c) keep both as separate versions | Data | BA | Must | Duplicate ingestion inflates extraction counts and pollutes the requirements catalog |
| REQ-REQM-056 | Near-Duplicate Detection | The system SHALL detect near-duplicate documents (same logical document, minor edits) using a document-level MinHash/SimHash similarity score; documents with similarity ≥ 0.90 SHALL be surfaced to the user with a side-by-side diff preview before ingestion | Data | BA, GVN | Should | Version confusion is common when teams share multiple drafts |
| REQ-REQM-057 | Cross-Source Deduplication | The system SHALL deduplicate source records originating from different connectors that represent the same artifact (e.g., a Word document uploaded manually AND referenced via SharePoint link); canonical source SHALL be designated by the user | Data | BA | Should | Prevents double-extraction of the same requirements from two ingestion paths |
| REQ-REQM-058 | Source Version Lineage | The system SHALL maintain a version lineage chain for each logical source document, linking each ingested version (v1, v2, …) to its predecessor with the delta (number of added/changed/deleted CDM blocks) | Data | BA, GVN | Must | Change management requires knowing which document version was current at baseline |
| REQ-REQM-059 | Source Retirement | The system SHALL allow a BA to mark a source document as "Retired", preventing it from surfacing new extraction candidates while retaining all existing traceability links for audit purposes | Functional | BA, GVN | Should | Superseded documents should not generate new requirements but must remain auditable |

---

## 6. Ingestion Status, Progress & Error Handling

### 6.1 Status & Progress

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-060 | Per-Job Status Lifecycle | Every ingestion job SHALL transition through a defined status lifecycle: Queued → Uploading → Scanning → Parsing → Extracting → Reviewing → Complete (or Failed / Cancelled); the current status SHALL be visible in real-time in the UI | Functional | ALL | Must | Users need to know when documents are ready for extraction review |
| REQ-REQM-061 | Real-Time Progress Push | The system SHALL push ingestion progress updates to the client via WebSocket or Server-Sent Events at ≥1 Hz update frequency, reporting: current phase, percentage complete within phase, and a human-readable status message | UX | ALL | Must | Real-time feedback is required for acceptable UX on large uploads |
| REQ-REQM-062 | Estimated Completion Time | The system SHALL compute and display an estimated time-to-completion for each ingestion job based on file size, format, and current worker throughput; the estimate SHALL be updated at each phase transition | UX | ALL | Should | Helps users plan their work session around processing wait times |
| REQ-REQM-063 | Ingestion Job History | The system SHALL maintain a persistent ingestion job history per project, retaining job status, start/end timestamps, file metadata, phase durations, and outcome (success/fail/cancelled) for a minimum of 90 days | Data | BA, GVN | Must | Audit trail and troubleshooting |
| REQ-REQM-064 | Batch Import Summary Report | Upon completion of a batch import, the system SHALL generate a summary report listing: files attempted, files succeeded, files failed (with error codes), files skipped (duplicate), total extraction candidates produced, and processing time | Reporting | BA | Must | Operational visibility for large-scale ingestion |

---

### 6.2 Error Handling & Recovery

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-065 | Structured Parse Error Reporting | When a file fails to parse, the system SHALL capture and store a structured error record containing: error type (enum: FORMAT_UNSUPPORTED, CORRUPT_FILE, OCR_FAILURE, TIMEOUT, VIRUS_DETECTED, SIZE_EXCEEDED, ENCODING_ERROR, NETWORK_ERROR), error message, page/row number where failure occurred, and a support reference ID | Functional | ALL | Must | Structured errors enable automated retry logic and user-guided remediation |
| REQ-REQM-066 | Partial-Success Handling | If parsing fails after partial completion (e.g., failure on page 150 of a 200-page document), the system SHALL persist the successfully parsed portion, mark the job as PARTIAL_SUCCESS, and report the boundary page/section so the user can decide to re-attempt or accept partial results | Functional | BA | Must | Prevents total loss of work on large documents with a single bad section |
| REQ-REQM-067 | Automatic Retry with Back-Off | The system SHALL automatically retry failed ingestion jobs up to 3 times with exponential back-off (1 min, 4 min, 16 min) for transient failure types (TIMEOUT, NETWORK_ERROR); permanent failure types (CORRUPT_FILE, FORMAT_UNSUPPORTED) SHALL NOT be retried automatically | Functional | ALL | Must | Transient infrastructure failures should not require manual intervention |
| REQ-REQM-068 | Manual Retry | The system SHALL allow a BA to manually trigger a retry of a failed ingestion job from the UI at any time, optionally with a different ingestion configuration (e.g., different OCR mode) | Functional | BA | Must | Gives user control after diagnosing an error |
| REQ-REQM-069 | Error Notification | The system SHALL send an in-app notification (and optionally email/Teams notification if configured) to the user who initiated a job when that job fails or completes with warnings; notifications SHALL include the job reference and a direct link to the error details | UX | ALL | Should | Users need to be informed without polling the UI |

---

## 7. Extraction-Candidate Lifecycle

### 7.1 Candidate Creation & Review Queue

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-070 | AI Extraction Candidate Record | For each requirement candidate produced by V06 AI extraction, the system SHALL create an extraction-candidate record containing: candidate text, candidate type (business/functional/NFR/constraint/assumption/unknown), AI confidence score (0.0–1.0), source provenance reference (CDM block ID, source document, page, character offset), extraction model version, and initial candidate status (PENDING_REVIEW) | Data | BA | Must | Candidates are the bridge between raw documents and managed requirements |
| REQ-REQM-071 | Bulk Candidate Review Queue | The system SHALL present extraction candidates in a paginated review queue, sortable and filterable by: source document, candidate type, AI confidence score, status (PENDING / ACCEPTED / REJECTED / DEFERRED), assigned reviewer, date extracted, and project area | UX | BA, PM | Must | Core review workflow for turning AI output into managed requirements |
| REQ-REQM-072 | Candidate Accept Action | A reviewer SHALL be able to accept an extraction candidate via a single click or keyboard shortcut, promoting it to a managed requirement; on acceptance the system SHALL pre-populate the new requirement's fields from the candidate record (type, text, source span) | UX | BA | Must | Accept is the highest-frequency review action; must be low-friction |
| REQ-REQM-073 | Candidate Reject Action | A reviewer SHALL be able to reject an extraction candidate with a mandatory or optional rejection reason (free text or from a configurable picklist: DUPLICATE, OUT_OF_SCOPE, NOISE, ALREADY_CAPTURED, OTHER); rejected candidates SHALL be retained in the audit log | UX | BA | Must | Rejection data feeds AI quality improvement loops (V06) |
| REQ-REQM-074 | Candidate Defer Action | A reviewer SHALL be able to defer a candidate to a named future review milestone, removing it from the active queue but retaining it for recall; deferred candidates SHALL be visible in a filtered view and appear automatically at the configured milestone date | UX | BA, PM | Should | Some candidates are valid but belong to a future release scope |
| REQ-REQM-075 | Candidate Edit Before Accept | Before accepting a candidate, a reviewer SHALL be able to inline-edit the candidate text, type, and AI-assigned fields without leaving the review queue; edits SHALL be tracked with editor identity and timestamp | UX | BA | Must | AI-extracted text often requires minor wording corrections before promotion |
| REQ-REQM-076 | Bulk Accept / Reject | The system SHALL allow a reviewer to select multiple candidates (up to 200 per operation) and perform a bulk accept or bulk reject action; bulk rejection SHALL require a single reason applied to all selected candidates | UX | BA, PM | Must | High-volume extraction runs (1000+ candidates) require bulk actions to be tractable |
| REQ-REQM-077 | Candidate Merge | The system SHALL allow a reviewer to select two or more candidates and merge them into a single requirement, with the system prompting for which candidate's text to use as the primary or offering a merged composite; all source spans of all merged candidates SHALL be retained as multi-provenance links | Functional | BA | Must | AI may extract fragments of the same requirement from multiple locations |
| REQ-REQM-078 | Candidate Split | The system SHALL allow a reviewer to split a single candidate into two or more requirements by selecting split points in the candidate text; each resulting requirement SHALL inherit the original source span with narrowed character offsets | Functional | BA | Should | AI may bundle two distinct requirements into a single extracted block |
| REQ-REQM-079 | Confidence Threshold Filtering | The system SHALL allow administrators to configure a confidence threshold (default 0.7); candidates below the threshold SHALL be marked LOW_CONFIDENCE and visually distinguished in the review queue; a bulk-reject-all-low-confidence action SHALL be available | Functional | BA, GVN | Should | Reduces review noise on large-document runs |
| REQ-REQM-080 | Re-Extraction Trigger | The system SHALL allow a BA to trigger re-extraction of a source document (using an updated extraction prompt or model version from V06) without re-uploading the document; the new candidate batch SHALL be presented as a delta relative to the prior extraction run | Functional | BA | Should | Model improvements and refined prompts should be applicable to existing sources |

---

### 7.2 Provenance & Source Linking

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-081 | Source Span Highlight | For any requirement in the catalog, the system SHALL provide a "View Source" action that opens the source document viewer, scrolls to the originating passage, and highlights the exact character span from which the requirement was extracted | UX | BA, STK | Must | Reviewers must be able to verify AI extraction against the original text |
| REQ-REQM-082 | Multi-Provenance Linking | A single requirement SHALL be linkable to multiple source spans across multiple documents (supporting merged candidates and cross-document consolidation); the provenance list SHALL show all contributing source references with page/section context | Data | BA | Must | Requirements often appear in multiple source documents (spec + email + transcript) |
| REQ-REQM-083 | Provenance Immutability | Source provenance references (CDM block ID, file hash, character offset) SHALL be immutable once created; if the source document is re-ingested as a new version, new provenance references SHALL be created pointing to the new version while old references remain intact | Data | GVN | Must | Audit integrity: historical provenance must not change retroactively |
| REQ-REQM-084 | Source Document Viewer | The system SHALL provide an inline source document viewer rendering the original file (PDF viewer, Word-to-HTML render, spreadsheet grid) with highlight overlay support for source spans; the viewer SHALL be accessible from both the candidate review queue and the requirement detail page | UX | BA, STK | Must | In-tool document review eliminates context-switching to external apps |

---

## 8. Requirement Data Model & Schema

### 8.1 Core Requirement Schema

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-085 | Unique Requirement Identifier | Every requirement SHALL be assigned a system-generated, immutable, globally unique identifier (UUID) at creation time; additionally a human-readable project-scoped ID (configurable prefix + sequential number, e.g., "REQ-0042") SHALL be assigned and SHALL NOT be reused upon deletion | Data | ALL | Must | IEEE 830 §3.2.1; Jira/DOORS best practice |
| REQ-REQM-086 | Requirement Title Field | Every requirement SHALL have a short title (max 255 characters), mandatory, used as the primary display label in lists, matrices, and trace views | Data | ALL | Must | ISO 29148 §5.2.5 |
| REQ-REQM-087 | Requirement Statement Field | Every requirement SHALL have a full statement (shall/should/must phrasing), free-text, rendered as rich text (Markdown or WYSIWYG), with a recommended length of 20–500 words; length violations SHALL trigger a quality warning | Data | ALL | Must | ISO 29148 §5.4.1 requirement text |
| REQ-REQM-088 | Requirement Type Field | Every requirement SHALL have a type drawn from the taxonomy defined in §9 (Business, Functional, NFR, Constraint, Assumption, Transition, Regulatory, Interface); the type SHALL influence quality check rules applied (§13) | Data | ALL | Must | BABOK v3 §4.5; ISO 29148 requirement classification |
| REQ-REQM-089 | Requirement Category / Domain Field | Requirements SHALL be classifiable by one or more domain categories (configurable per project; examples: Performance, Security, Usability, Data, Integration, Reporting) using a multi-select tag field | Data | BA, SA | Must | Enables filtering and dashboard slicing by architecture domain |
| REQ-REQM-090 | Requirement Status Field | Requirements SHALL have a status from a configurable lifecycle (see §11); default lifecycle: Draft → In Review → Approved → Baselined → Deprecated → Rejected | Data | ALL | Must | ISO 29148 §5.2.3 lifecycle |
| REQ-REQM-091 | Requirement Priority Field | Requirements SHALL have a priority assignment following one or more prioritization frameworks (§28); minimum: MoSCoW (Must/Should/Could/Won't); WSJF score SHALL be storable as a numeric custom field | Data | BA, PM | Must | MoSCoW is the baseline; WSJF needed for SAFe-aligned orgs |
| REQ-REQM-092 | Rationale Field | Requirements SHALL have an optional rationale / business justification field (free-text rich text) describing why the requirement exists; absence SHALL trigger a completeness warning | Data | BA, STK | Must | BABOK v3 §4.5 — rationale is a required requirement attribute |
| REQ-REQM-093 | Owner / Author Field | Requirements SHALL carry a mandatory owner field (linked to a system user or stakeholder record) and an optional author field (who first created the record); both SHALL be displayed in list views and used in RACI | Data | BA, GVN | Must | Governance: every requirement must have an identified owner |
| REQ-REQM-094 | Source Reference Field | Requirements SHALL carry one or more source provenance references (§7.2) pointing to CDM blocks in ingested source documents; requirements created manually SHALL allow free-text source description if no document is ingested | Data | BA | Must | IEEE 830 §3.2.1 traceability |
| REQ-REQM-095 | Created / Modified Timestamps | Requirements SHALL automatically record: created_at (UTC), created_by (user), last_modified_at (UTC), last_modified_by (user); these fields SHALL be system-managed and not user-editable | Data | ALL | Must | Audit trail baseline |
| REQ-REQM-096 | Version Field | Requirements SHALL carry a version counter (major.minor, starting 1.0) incremented at configurable events (minor: text edits; major: status promotion to Approved or Baselined); version history SHALL be navigable (§22) | Data | ALL | Must | Required for baseline and change management (§20) |

---

### 8.2 Extended Schema Attributes

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-097 | Constraint & Assumption Linkage | Requirements of type Constraint and Assumption SHALL be linkable to one or more regular requirements they constrain or underpin; the linkage SHALL be rendered as a dependency graph in the requirement detail view | Data | BA, EA | Must | Constraints and assumptions affect a requirement's validity context |
| REQ-REQM-098 | Regulatory Reference Field | Requirements SHALL carry an optional regulatory reference field (free text + configurable picklist of regulation names: GDPR, HIPAA, SOC2, PCI-DSS, ISO 27001, etc.) to flag compliance-driven requirements | Data | SecA, GVN | Must | Compliance tracing is a first-class concern in enterprise architecture |
| REQ-REQM-099 | Effort Estimate Fields | Requirements SHALL support optional effort estimate fields: story points (integer), T-shirt size (XS/S/M/L/XL), and hours (decimal); these fields feed the WSJF calculation and roadmap planning integrations | Data | PM, BA | Should | Required for WSJF scoring and sprint-capacity planning |
| REQ-REQM-100 | Risk Level Field | Requirements SHALL carry an optional risk level (Low/Medium/High/Critical) with a risk description field; high-risk requirements SHALL be visually flagged in lists and dashboards | Data | BA, SecA | Should | Risk visibility is a BABOK v3 and ISO 29148 quality attribute |
| REQ-REQM-101 | Stability / Volatility Field | Requirements SHALL carry a stability indicator (Stable / Likely to Change / Volatile) to support volatility reporting and change-impact analysis weighting | Data | BA, PM | Should | High-volatility requirements indicate rework risk |
| REQ-REQM-102 | Linked Test Cases | Requirements SHALL support linking to external test cases via URL or structured reference (Jira test issue key, ADO test case ID, Zephyr/Xray test key); linked test case coverage SHALL be reflected in traceability dashboards | Data | QA, BA | Must | Test coverage traceability is a core RTM capability |
| REQ-REQM-103 | Rich Text with Embedded Media | Requirement statement and rationale fields SHALL support inline images, hyperlinks, inline code blocks, and embedded Mermaid diagrams as rich-text content; media SHALL be stored in the platform's asset store with provenance linkage | Data | BA, SA | Should | Complex requirements often need supporting diagrams or screenshots inline |

---

## 9. Requirement Types & Hierarchy

### 9.1 Requirement Type Taxonomy

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-104 | Business Requirement Type | The system SHALL support Business Requirement as a first-class type representing high-level organizational goals, outcomes, or business problems (BABOK v3 "Business Requirements"); these SHALL be linkable to functional requirements and roadmap objectives | Functional | BA, STK | Must | BABOK v3 §1.3; ISO 29148 §5.1 |
| REQ-REQM-105 | Functional Requirement Type | The system SHALL support Functional Requirement type representing system behaviors, features, or capabilities the solution must provide (IEEE 830 §3.1); quality checks SHALL enforce "shall" phrasing and single-behavior rule | Functional | BA, SA | Must | IEEE 830; ISO 29148 §5.2 |
| REQ-REQM-106 | Non-Functional Requirement (NFR) Type | The system SHALL support NFR type with a mandatory NFR sub-type picklist: Performance, Scalability, Availability, Reliability, Maintainability, Security, Usability, Portability, Interoperability, Compliance, Capacity, Recoverability; NFRs SHALL support measurable acceptance thresholds (value + unit + condition) | Functional | SA, SecA, QA | Must | ISO 25010 quality model; ISO 29148 §5.2.5 |
| REQ-REQM-107 | Constraint Type | The system SHALL support Constraint type representing non-negotiable restrictions (technology stack, legal, regulatory, time, budget); constraints SHALL have a constraint source field (Legal/Business/Technology/Regulatory/Financial) | Functional | BA, GVN | Must | BABOK v3 §6.1; ISO 29148 §5.5 |
| REQ-REQM-108 | Assumption Type | The system SHALL support Assumption type representing factors assumed true for the project; assumptions SHALL have a validation status (Unvalidated / Validated / Invalidated) and a validation owner | Functional | BA, PM | Must | BABOK v3 §6.2; unvalidated assumptions are project risk |
| REQ-REQM-109 | Transition Requirement Type | The system SHALL support Transition Requirement type for requirements that exist only during system migration/cutover (e.g., data migration, parallel-run, rollback criteria); these SHALL be auto-deprecated upon transition milestone completion | Functional | SA, BA | Should | BABOK v3 §6.3 transition requirements |
| REQ-REQM-110 | Regulatory / Compliance Requirement Type | The system SHALL support Regulatory Requirement type linked to one or more regulations (REQ-REQM-098); these SHALL be treated as Must-have by default and require GVN-role approval in change workflows | Functional | GVN, SecA | Must | Compliance requirements require elevated governance |
| REQ-REQM-111 | Interface Requirement Type | The system SHALL support Interface Requirement type describing integration contracts, API contracts, UI interaction contracts, and data exchange requirements; interface requirements SHALL linkable to integration components in the architecture model | Functional | IA, SA | Must | IEEE 830 §3.2 interface requirements |

---

### 9.2 Requirement Hierarchy & Decomposition

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-112 | Parent-Child Decomposition | The system SHALL support parent-child decomposition of requirements to unlimited depth, allowing a business requirement to decompose into functional requirements which decompose into detailed sub-requirements; the hierarchy SHALL be displayed as a collapsible tree | Data | BA, SA | Must | BABOK v3 hierarchical decomposition; required for large programs |
| REQ-REQM-113 | Horizontal Grouping (Epics / Features / Stories) | The system SHALL support grouping requirements into configurable hierarchy levels named by the organization (defaults: Epic → Feature → Story → Task); level names SHALL be configurable per project | Functional | BA, PM | Must | Aligns with SAFe/Scrum; common PM grouping model |
| REQ-REQM-114 | Cross-Hierarchy Linking | The system SHALL allow requirements to link across hierarchy branches (e.g., a business requirement linked to NFRs at a different branch level); cross-hierarchy links SHALL be visually distinct from parent-child links in tree views | Data | BA, EA | Should | Real requirements have complex many-to-many relationships |
| REQ-REQM-115 | Requirement Package / Module | The system SHALL support organizing requirements into named packages/modules (analogous to DOORS modules) with independent ID namespaces, access controls, and baseline lifecycles; packages can be nested | Data | BA, GVN | Should | Large programs partition requirements by subsystem/stream |

---

## 10. User Stories & Acceptance Criteria

### 10.1 User Story Model

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-116 | User Story Template | Requirements of type User Story SHALL support the structured template: "As a [persona], I want [goal], so that [benefit]"; the system SHALL provide labeled input fields for persona, goal, and benefit, with a preview of the assembled story text | UX | BA, PM | Must | Standard Agile user story format (Connextra template) |
| REQ-REQM-117 | AI Story Generation from Business Requirement | The system SHALL provide an AI-assisted action to generate user story candidates from a selected business requirement; AI story generation is orchestrated by V06, but the resulting stories SHALL be created as draft requirements in this pillar and queued for review | Functional | BA, PM | Must | Core AI-productivity feature in the Requirements Pillar |
| REQ-REQM-118 | Persona Library | The system SHALL maintain a project-level persona library (name, role description, sample goals) reusable across user stories; personas SHALL be linkable to RACI stakeholder records | Data | BA, PM | Should | Consistent persona vocabulary improves story quality |
| REQ-REQM-119 | Story Map View | The system SHALL provide a story-map visualization showing user stories organized by user activity (columns) and release/sprint (rows), drag-and-drop reordering, and inline story creation; this view SHALL be sortable by priority | UX | PM, BA | Should | Story mapping is a standard BA/PM planning technique |

---

### 10.2 Acceptance Criteria Model

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-120 | Acceptance Criteria Field | Every requirement SHALL support one or more acceptance criteria (ACs), each stored as a discrete record with: AC text (rich text), AC type (Given-When-Then / Checklist / Metric / Rule), status (Draft / Agreed / Verified / Failed), and a linked test case reference | Data | BA, QA | Must | ISO 29148 §5.4.2; BABOK v3 §4.5 verifiability attribute |
| REQ-REQM-121 | Given-When-Then (BDD) Template | ACs of type Given-When-Then SHALL support structured fields for Given context, When action, and Then expected outcome, with a preview of the assembled BDD sentence; BDD scenarios SHALL be exportable as Gherkin (.feature files) | Functional | BA, QA | Must | BDD is the dominant AC format for Agile teams; Gherkin export enables test automation |
| REQ-REQM-122 | AI AC Generation | The system SHALL provide an AI-assisted action to generate acceptance criteria candidates for a selected requirement; candidates SHALL be presented in the AC review panel for accept/edit/reject before being saved to the requirement | AI/ML | BA, QA | Must | Core AI-productivity feature; AC generation is delegated to V06 but managed here |
| REQ-REQM-123 | AC Metric Template | ACs of type Metric SHALL support structured fields: measure (e.g., "response time"), condition (e.g., "under 95th percentile load"), threshold (numeric), unit, and measurement method (e.g., "JMeter load test at 1,000 concurrent users") | Data | SA, QA | Must | Measurable ACs are required by ISO 29148 §5.4.2 |
| REQ-REQM-124 | AC Sign-Off | Each AC SHALL support individual sign-off by a designated verifier (QA, BA, or STK role), recording verifier identity, sign-off date, and optional comment; a requirement SHALL be eligible for Approved status only when all ACs are in Draft/Agreed state (not Failed) | Governance | GVN, QA | Must | Verifiability and sign-off are governance requirements |

---

## 11. MoSCoW, Priority & Status Lifecycle

### 11.1 MoSCoW Prioritization

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-125 | MoSCoW Priority Assignment | The system SHALL provide a MoSCoW priority field (Must Have / Should Have / Could Have / Won't Have This Time) for every requirement; Must Have requirements SHALL be visually distinguished in all list and dashboard views | Data | BA, PM | Must | MoSCoW is the baseline prioritization standard; DSDM/Agile alignment |
| REQ-REQM-126 | MoSCoW Budget Rule | The system SHALL compute and display the percentage of Must Have requirements within a release scope and warn when Must Have count exceeds 60% of total requirements in scope, as a DSDM guideline check | Functional | PM | Should | DSDM guideline: >60% Must Have signals over-commitment |
| REQ-REQM-127 | Priority Change History | All priority changes SHALL be tracked with previous value, new value, changed-by, changed-at, and a mandatory change justification note; the priority history SHALL be visible on the requirement detail page | Data | GVN | Must | Priority inflation is a common governance risk |

---

### 11.2 Requirement Status Lifecycle

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-128 | Configurable Status Lifecycle | The system SHALL provide a configurable status workflow per project with: configurable status names, allowed transitions, required fields per transition, and role permissions per transition; a default lifecycle SHALL be provided (Draft → In Review → Approved → Baselined → Deprecated → Rejected) | Functional | GVN, BA | Must | Different organizations have different maturity workflows |
| REQ-REQM-129 | Status Transition Guards | The system SHALL enforce transition guards: (a) a requirement cannot move to Approved without all mandatory fields populated; (b) a Baselined requirement cannot be directly edited — only via a change request (§21); (c) a Rejected requirement cannot be transitioned to Approved without first reverting to Draft | Functional | GVN | Must | Governance guardrails prevent accidental status inflation |
| REQ-REQM-130 | Bulk Status Transition | The system SHALL allow authorized users to bulk-transition requirements (up to 500 per operation) through a permitted status transition, with a confirmation dialog listing affected requirements and an optional bulk justification note | Functional | BA, PM | Should | Sprint ceremonies involve batch status changes |
| REQ-REQM-131 | Status Dashboard Swimlane | The system SHALL provide a Kanban-style swimlane view of requirements organized by status column, with drag-and-drop status transitions (subject to the same guards as §REQ-REQM-129) and WIP limits configurable per column | UX | BA, PM | Should | Visual workflow management improves throughput visibility |

---

## 12. Custom Fields, Tags & Categories

### 12.1 Custom Fields

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-132 | Custom Field Definition | Project administrators SHALL be able to define custom fields on requirements with types: Single-line text, Multi-line text, Number (integer/decimal), Date, Date-time, Boolean, Single-select dropdown, Multi-select dropdown, URL, User reference, Requirement reference, and Attachment | Functional | GVN, BA | Must | Organizations have organization-specific attributes (e.g., "System Owner", "Compliance Reference", "Sprint Target") |
| REQ-REQM-133 | Custom Field Required Flag | Custom fields SHALL be configurable as mandatory or optional; mandatory custom fields SHALL participate in completeness scoring (§15) and status transition guards (§11.2) | Functional | GVN | Must | Ensures critical metadata is captured before sign-off |
| REQ-REQM-134 | Custom Field Visibility & Section Grouping | Custom fields SHALL be groupable into named sections on the requirement detail form; sections SHALL be collapsible and orderable by drag-and-drop in the field definition UI | UX | GVN | Should | Long field lists must be organized for usability |
| REQ-REQM-135 | Custom Field Computed Formula | The system SHALL support a computed custom field type whose value is a formula over other numeric fields (e.g., WSJF = Business Value / Job Size); formula syntax SHALL use a safe expression language (not full script execution) | Functional | PM | Should | WSJF requires computed scoring across 3–4 input fields |
| REQ-REQM-136 | Custom Field Inheritance | When a requirement is created as a child of a parent, configurable custom fields SHALL inherit the parent's value as a default; inheritance SHALL be overridable at the child level | Functional | BA | Could | Reduces data-entry repetition in hierarchically structured projects |

---

### 12.2 Tags & Categories

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-137 | Free-Form Tag Assignment | Users SHALL be able to assign free-form text tags (labels) to requirements; tags SHALL be auto-suggested based on the project's existing tag vocabulary as the user types; tags SHALL be multi-valued per requirement | UX | ALL | Must | Tags are the lightest-weight categorization mechanism |
| REQ-REQM-138 | Controlled Category Taxonomy | Administrators SHALL be able to define a controlled category taxonomy (hierarchical or flat) for the project; requirements can be assigned to one or more categories from the taxonomy; categories SHALL be used in dashboard filters, RTM column groupings, and reporting | Functional | GVN, BA | Must | Controlled vocabulary enables consistent reporting across projects |
| REQ-REQM-139 | AI-Suggested Tags | The system SHALL offer AI-suggested tags when a requirement is saved; suggestions SHALL be drawn from the project's existing tag vocabulary using embedding similarity; users may accept, reject, or add additional tags | AI/ML | BA | Should | Reduces manual tagging effort; consistency improvement |
| REQ-REQM-140 | Tag-Based Bulk Operations | Users SHALL be able to select requirements by tag filter and perform bulk operations: bulk assign tags, bulk change status, bulk assign owner, or bulk export | Functional | BA | Should | Tag-based operations enable theme-based batch processing |

---

## 13. Quality & Hygiene Checks

### 13.1 INVEST / IEEE-830 / ISO-29148 Quality Checks

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-141 | Atomic Requirement Check | The system SHALL flag requirements whose text contains compound "and/or" structures linking two or more distinct behaviors as potentially non-atomic, using clause-boundary NLP analysis; the flag SHALL cite the detected conjunction and suggest splitting | AI/ML | BA | Must | IEEE 830 §4.2 "each requirement should state one thing"; ISO 29148 §5.4.1 |
| REQ-REQM-142 | Verifiability Check | The system SHALL flag requirements that lack a measurable or observable acceptance condition — detected by absence of numeric thresholds, comparative terms, or verifiable actions — as non-verifiable; NFRs without a metric SHALL always trigger this flag | AI/ML | BA, QA | Must | ISO 29148 §5.4.2 — verifiability is a mandatory requirement quality attribute |
| REQ-REQM-143 | Ambiguity Flag | The system SHALL flag requirements containing ambiguous language tokens from a configurable vocabulary (e.g., "appropriate", "adequate", "as required", "easy to use", "efficient", "minimal", "fast", "flexible", "robust", "user-friendly") and link to the exact token in the requirement text | AI/ML | BA | Must | IEEE 830 §4.3 — ambiguity is the primary requirement quality defect |
| REQ-REQM-144 | Feasibility & Contradiction Check | The system SHALL apply AI-driven cross-requirement analysis (using V06) to flag direct logical contradictions between requirements in the same project (e.g., "response time SHALL be ≤100 ms" conflicting with "response time SHALL be ≥500 ms for audit logging") | AI/ML | SA, BA | Must | ISO 29148 §5.4.1 consistency attribute |
| REQ-REQM-145 | Mandatory-Field Completeness Check | The system SHALL compute a completeness score per requirement (0–100%) based on the ratio of populated mandatory fields to total mandatory fields; requirements with score <70% SHALL be flagged INCOMPLETE with a list of missing fields | Functional | BA, GVN | Must | BABOK v3 §4.5 completeness attribute |
| REQ-REQM-146 | Passive Voice Check | The system SHALL flag requirements using passive voice without identifying the responsible party (e.g., "data shall be encrypted" without specifying by what component) as potentially incomplete; the flag SHALL underline the passive construction | AI/ML | BA | Should | IEEE 830 — passive voice obscures system responsibility |
| REQ-REQM-147 | Subjective Term Check | Extend REQ-REQM-143 to include subjective quality adjectives ("high quality", "good performance", "modern UI") with a configurable additional vocabulary per project | AI/ML | BA | Should | Project-specific jargon requires configurable detection |
| REQ-REQM-148 | INVEST Check for User Stories | For requirements of type User Story, the system SHALL apply INVEST quality checks: Independent (no implicit dependency on sequential story), Negotiable (no locked implementation mandate detected), Valuable (benefit clause present), Estimable (scope bounded), Small (no multi-sprint scope detected), Testable (AC present); each failing dimension SHALL be flagged individually | AI/ML | BA, PM | Must | INVEST is the canonical user story quality framework |
| REQ-REQM-149 | Orphan Requirement Flag | Requirements with no source provenance link, no parent requirement, no child requirement, and no trace to any deliverable or roadmap item SHALL be flagged as ORPHAN; an orphan dashboard SHALL list all such requirements by project | Functional | BA | Should | Orphan requirements indicate scope that is untracked and at risk |
| REQ-REQM-150 | Quality Check Override | Users SHALL be able to suppress individual quality flags on a requirement with a mandatory justification and suppressor identity; suppressed flags SHALL be visually distinct from resolved flags in audit reports | Functional | BA | Must | Legitimate exceptions exist; suppression must be auditable |
| REQ-REQM-151 | Bulk Quality Check Run | The system SHALL provide a bulk quality check action over a selected requirement set (or entire project), generating a quality report (§29) listing all flagged requirements by rule, with link-to-requirement | Reporting | BA, GVN | Must | Sprint quality gates require batch quality validation |

---

## 14. Deduplication, Merge & Conflict Detection

### 14.1 Requirement Deduplication

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-152 | Semantic Duplicate Detection | The system SHALL compute embedding similarity between all requirement statements within a project and surface pairs with cosine similarity ≥ 0.85 as potential duplicates in a deduplication dashboard; similarity computation SHALL run incrementally (not full re-scan) on each new requirement creation | AI/ML | BA | Must | Semantic duplicates are common when multiple teams contribute requirements |
| REQ-REQM-153 | Duplicate Merge Wizard | For each detected duplicate pair, the system SHALL provide a merge wizard showing both requirements side-by-side with field-level diff highlighting; the user SHALL be able to designate a canonical requirement, selectively merge field values, and transfer all trace links and source spans from the merged-away requirement to the canonical one before deletion | UX | BA | Must | Merge must be lossless with respect to traceability |
| REQ-REQM-154 | Duplicate Flag History | All duplicate detections, merge actions, and dismiss-as-not-duplicate actions SHALL be logged in the audit trail with user and timestamp; dismissed pairs SHALL not be re-surfaced unless the text of either requirement changes by ≥10% | Data | GVN | Should | Prevents repeated surfacing of intentionally distinct requirements |

---

### 14.2 Conflict Detection

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-155 | NFR Threshold Conflict Detection | The system SHALL detect conflicting NFR thresholds on the same quality attribute (e.g., two performance requirements targeting different response-time SLAs for the same endpoint pattern) and flag them as CONFLICT with a side-by-side view | AI/ML | SA, BA | Must | NFR conflicts are frequently introduced when teams work independently |
| REQ-REQM-156 | Cross-Package Conflict Detection | When requirements from different packages/modules share an interface or entity, the system SHALL check for behavioral conflicts (e.g., two requirements specifying different default values for the same field) using AI cross-analysis; flagged conflicts SHALL be assigned to both package owners | AI/ML | SA, IA | Should | Interface requirements between teams are the highest-conflict zone |
| REQ-REQM-157 | Conflict Resolution Workflow | Detected conflicts SHALL be convertible into conflict-resolution tasks assigned to designated owners, with a target resolution date, discussion thread, and final resolution record (which requirement was amended or removed) | Functional | BA, GVN | Should | Unresolved conflicts block baseline; workflow ensures closure |

---

## 15. Completeness & Coverage Scoring

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-158 | Project Completeness Score | The system SHALL compute a project-level requirements completeness score (0–100%) as a weighted average of: field completeness (§13), AC coverage (% of requirements with ≥1 AC), traceability coverage (§18), quality check pass rate (§13), and source coverage (% of requirements traced to a source); weights SHALL be configurable | Reporting | BA, GVN | Must | Single-number health indicator for stakeholder reporting |
| REQ-REQM-159 | Completeness Trend | The system SHALL track completeness score over time (daily snapshot) and display a trend line chart; regression (score decrease) SHALL trigger an in-app notification to the project BA | Reporting | BA | Should | Trend reveals if quality is improving or degrading during elicitation |
| REQ-REQM-160 | Domain Coverage Matrix | The system SHALL display a domain/category × requirement-type matrix (heat map) showing count and completeness score per cell, enabling identification of gaps (e.g., Security NFRs missing, Interface requirements incomplete) | Reporting | EA, BA | Must | Cross-domain coverage is a critical architecture review artifact |
| REQ-REQM-161 | BABOK Knowledge Area Coverage | The system SHALL map requirements to the six BABOK v3 knowledge area categories (Business Analysis Planning, Elicitation, Requirements Lifecycle, Strategy Analysis, Requirements Analysis, Solution Evaluation) based on type and metadata, and display coverage per area | Reporting | BA, GVN | Could | For organizations that formally align to BABOK v3 practices |

---

## 16. Requirements Traceability Matrix

### 16.1 RTM Core Model

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-162 | RTM Data Model | The system SHALL maintain a Requirements Traceability Matrix (RTM) as a directed graph where nodes are: Source Document, Requirement, Architecture Model Element, Deliverable Section, Test Case, and Roadmap Item; edges are typed relationships with: source node ID, target node ID, relationship type (traces-to / satisfies / implements / verified-by / allocated-to / derived-from / refines), created-by, and created-at | Data | ALL | Must | IEEE 29148 §5.2.9 traceability; BABOK v3 §3.7 |
| REQ-REQM-163 | RTM Grid View | The system SHALL render the RTM as a 2D grid (rows = requirements; columns = configurable: Sources / Model Elements / Deliverables / Test Cases / Roadmap Items); each cell SHALL show the presence/absence of a link and SHALL be clickable to view link details or create a link | UX | BA, EA, QA | Must | Grid view is the traditional RTM format expected by auditors |
| REQ-REQM-164 | RTM Graph View | The system SHALL render the RTM as an interactive force-directed graph where nodes are coloured by type and edges by relationship type; pan, zoom, and click-to-detail SHALL be supported; graph SHALL handle ≥2,000 nodes without performance degradation | UX | EA, SA | Should | Graph view reveals complex multi-hop traceability patterns |
| REQ-REQM-165 | RTM Link Creation | Users SHALL be able to create RTM links via: (a) drag-and-drop in the graph view, (b) a "Link To" action from the requirement detail page with a search-and-select dialog for the target artifact, (c) bulk import of a link CSV (source ID, target ID, relationship type), (d) AI-suggested links (V06) requiring user confirmation | Functional | BA, EA | Must | Multiple link creation modalities for different workflow contexts |
| REQ-REQM-166 | RTM Link Justification | Each RTM link SHALL carry an optional justification/note field; for Regulatory requirement → model element links the justification SHALL be mandatory | Data | GVN | Should | Compliance audits require rationale for traceability decisions |
| REQ-REQM-167 | RTM Link Health | The system SHALL continuously evaluate RTM link health: (a) STALE — the target artifact has been modified since the link was created; (b) BROKEN — the target artifact has been deleted; (c) UNCONFIRMED — AI-suggested link not yet reviewed by a human; each health status SHALL be surfaced in the RTM grid and graph views | Functional | BA, EA | Must | Stale and broken links are the primary RTM maintenance problem |

---

### 16.2 Trace Link Types & Semantics

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-168 | Source-to-Requirement Trace | The system SHALL support a "derived-from" trace link from a requirement to one or more CDM source spans (§7.2), capturing which text passage in which document gave rise to the requirement; this trace is the primary provenance chain | Data | BA | Must | Foundation of all downstream traceability |
| REQ-REQM-169 | Requirement-to-Requirement Trace | The system SHALL support requirement-to-requirement links of types: refines (child elaborates parent), satisfies (lower-level fulfills higher-level), conflicts-with (§14.2), depends-on (ordering dependency), and supersedes (replacement relationship) | Data | BA, SA | Must | Inter-requirement relationships are required for impact analysis |
| REQ-REQM-170 | Requirement-to-Architecture Element Trace | The system SHALL support "allocated-to" and "implements" link types from a requirement to architecture model elements (component, service, interface, data entity) managed in the platform's architecture model store (V04/V08) | Integration | EA, SA | Must | Allocation traceability is a core TOGAF / IEEE 42010 concern |
| REQ-REQM-171 | Requirement-to-Deliverable Trace | The system SHALL support "satisfied-by" links from requirements to sections or paragraphs within platform-generated deliverables (V10), so that a generated Architecture Decision Record or design document can be traced back to the requirement it addresses | Integration | EA, SA | Must | Closes the loop from requirement to architecture output |
| REQ-REQM-172 | Requirement-to-Test Trace | The system SHALL support "verified-by" links from requirements to external test cases (Jira/ADO/Zephyr test keys or URLs); test execution status SHALL be pulled via connector and reflected in the RTM as a coverage metric | Integration | QA, BA | Must | Test coverage traceability is a fundamental QA governance requirement |
| REQ-REQM-173 | Requirement-to-Roadmap Item Trace | The system SHALL support "planned-for" links from requirements to roadmap items / releases (V09); these links feed the portfolio-level coverage and release-readiness dashboards | Integration | PM, BA | Must | Ensures every requirement is allocated to a delivery vehicle |

---

## 17. Forward & Backward Traceability

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-174 | Forward Traceability View | The system SHALL provide a forward traceability view for a selected requirement showing the full downstream chain: requirement → architecture elements → deliverables → test cases → roadmap items, rendered as a collapsible tree with health indicators at each link | UX | EA, SA, BA | Must | Forward trace answers "is this requirement fully addressed?" |
| REQ-REQM-175 | Backward Traceability View | The system SHALL provide a backward traceability view starting from any linked artifact (test case, deliverable, model element) traversing upstream to source requirements and source documents, showing the full lineage chain | UX | QA, BA, GVN | Must | Backward trace answers "why does this artifact exist?" — required for audit |
| REQ-REQM-176 | Multi-Hop Trace Path | The system SHALL resolve multi-hop trace paths (e.g., source doc → requirement → sub-requirement → architecture element → test case) and present the full path as a sequence of nodes with link types annotated between each node | UX | EA | Should | Complex systems have deep trace chains; full-path visibility is critical |
| REQ-REQM-177 | Trace Path Export | The system SHALL export any trace path or the full RTM to Excel/CSV with configurable column selection (node IDs, names, types, statuses, link types, health); an optionally filtered export (by requirement type, status, or category) SHALL be supported | Reporting | GVN, BA | Must | Auditors and compliance reviewers require offline RTM artifacts |

---

## 18. Coverage, Orphan & Gap Analysis

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-178 | Test Coverage Percentage | The system SHALL compute per requirement (and aggregated to package/type/category): test coverage percentage = (requirements with ≥1 verified-by link to a test case) / (total requirements in scope); this metric SHALL be displayed on the quality dashboard and exportable | Reporting | QA, BA | Must | Test coverage is a primary release-readiness gate metric |
| REQ-REQM-179 | Architecture Coverage Percentage | The system SHALL compute architecture allocation coverage = (requirements with ≥1 allocated-to architecture element link) / (total Must/Should requirements); unallocated Must requirements SHALL trigger a CRITICAL coverage gap alert | Reporting | EA, SA | Must | Unallocated requirements signal architecture scope gaps |
| REQ-REQM-180 | Orphan Requirement Report | The system SHALL generate an orphan requirements report listing all requirements with no downstream trace links (no test, no architecture element, no deliverable, no roadmap item), grouped by type and priority; Must-priority orphans SHALL be highlighted as critical | Reporting | BA, GVN | Must | Orphan detection is a standard RTM audit procedure |
| REQ-REQM-181 | Orphan Source Document Report | The system SHALL report source documents that have been fully ingested but have zero requirements traced back to them (no extraction candidates accepted), flagging them as "unused source" for user review | Reporting | BA | Should | Ensures no ingested document is silently ignored |
| REQ-REQM-182 | Gap Heatmap | The system SHALL display a configurable gap heatmap (axes: requirement type × architecture layer, or requirement category × delivery release) where each cell is coloured by coverage %, enabling rapid visual identification of under-traced areas | Reporting | EA, BA | Should | Heatmap format is the preferred executive summary for traceability gaps |
| REQ-REQM-183 | Coverage Threshold Alerts | The system SHALL allow per-project configuration of coverage threshold alerts (e.g., alert when test coverage of Must requirements drops below 80%); alerts SHALL be surfaced as in-app notifications and optionally as email / Teams webhooks | Functional | GVN, BA | Should | Proactive governance rather than reactive reporting |

---

## 19. Impact Analysis on Change

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-184 | Change Impact Scope Computation | When a requirement is modified, the system SHALL traverse the RTM forward from the changed requirement to identify all directly and transitively linked artifacts (architecture elements, deliverables, test cases, roadmap items) and present an impact scope list with artifact count and estimated rework effort (Low/Medium/High heuristic) | Functional | EA, SA, BA | Must | Impact analysis is the primary value of a maintained RTM |
| REQ-REQM-185 | Impact Analysis Report | The system SHALL generate a printable/exportable Impact Analysis Report for a proposed change to one or more requirements, listing: changed requirements, directly impacted artifacts (by type), transitively impacted artifacts (by type), impacted team owners, and recommended review actions | Reporting | EA, SA, BA | Must | Required artifact for change advisory board (CAB) presentations |
| REQ-REQM-186 | Cascade Change Suggestion | After impact analysis, the system SHALL suggest cascade status changes: e.g., when a Baselined requirement is amended, the system SHALL flag all linked test cases as RETEST_REQUIRED and all linked deliverable sections as REVIEW_REQUIRED; users SHALL confirm or dismiss each cascade action individually | Functional | BA, QA | Should | Prevents stale downstream artifacts after requirement changes |
| REQ-REQM-187 | "What-If" Change Simulation | The system SHALL support a "what-if" simulation mode where a user can draft a change to a requirement and view the projected impact scope before committing the change; the draft SHALL not create a real change request until explicitly submitted | Functional | SA, BA | Should | De-risks change decisions by previewing blast radius |

---

## 20. Baselining & Versioning

### 20.1 Requirement Versioning

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-188 | Automatic Requirement Versioning | The system SHALL automatically create a new requirement version record on every field modification (including text, status, priority, custom fields, AC changes), storing the full field snapshot of the previous version and the diff; versions SHALL be navigable from the requirement detail page | Data | ALL | Must | ISO 29148 §5.2.3 requirement lifecycle; BABOK v3 traceability |
| REQ-REQM-189 | Version Diff View | The system SHALL provide a side-by-side or unified diff view between any two versions of a requirement, highlighting added, removed, and changed field values; diff SHALL cover all fields including rich-text statements | UX | BA, GVN | Must | Change review requires clear before/after visualization |
| REQ-REQM-190 | Version Restore | Authorized users SHALL be able to restore a requirement to any prior version; restoration SHALL create a new version record (not overwrite history) with a restoration note identifying which version was restored | Functional | BA | Must | Accidental edits must be reversible without losing history |
| REQ-REQM-191 | Version Pruning Policy | Administrators SHALL be able to configure a version pruning policy: (a) retain all versions indefinitely, (b) retain last N versions per requirement, (c) retain versions ≤ X days old, with Baselined versions always exempted from pruning | Data | GVN | Could | Storage management for long-running programs |

---

### 20.2 Baselines

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-192 | Baseline Creation | Authorized users SHALL be able to create a named project baseline capturing a point-in-time snapshot of all requirements (and their current versions) in scope; a baseline SHALL be immutable after creation | Data | GVN, BA | Must | IEEE 829 / ISO 29148 baseline concept; BABOK v3 §3.6 |
| REQ-REQM-193 | Baseline Scope Configuration | Baselines SHALL be configurable by scope filter (all requirements / by package / by status / by release / by tag), enabling partial baselines for subsystem or release-level governance | Functional | GVN | Must | Large programs baseline by release train, not all-at-once |
| REQ-REQM-194 | Baseline Comparison | The system SHALL provide a baseline comparison view showing differences between two named baselines (or between a baseline and the current state): added requirements, removed requirements, and field-level changes per requirement | UX | GVN, BA | Must | Baseline diff is the standard change-set review artifact |
| REQ-REQM-195 | Baseline Export | Each baseline SHALL be exportable as a self-contained artifact: Excel/CSV, JSON, or Word report with a full snapshot of all requirements in scope including all fields, ACs, and RTM links at the time of baseline creation | Reporting | GVN | Must | Baseline archives are required for regulatory audits and contractual obligations |
| REQ-REQM-196 | Baselined Requirement Edit Guard | Any attempt to directly edit a field of a requirement that is part of a Baselined set SHALL be blocked by the system; editing SHALL only be possible by initiating a Change Request (§21) which, when approved, creates a new version and updates the current baseline pointer | Functional | GVN | Must | Baseline integrity is a hard governance requirement |

---

## 21. Change Requests & Approval Workflows

### 21.1 Change Request Model

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-197 | Change Request Record | The system SHALL support a Change Request (CR) record with: CR ID, title, description of change, change type (Add / Modify / Delete / Defer), target requirement(s), requester, submission date, business justification, impact assessment reference, priority, status lifecycle (Draft → Submitted → Under Review → Approved → Rejected → Withdrawn → Implemented) | Data | BA, GVN | Must | ISO 29148 §5.2.3 change management |
| REQ-REQM-198 | CR Impact Analysis Embedding | A CR SHALL automatically trigger REQ-REQM-184 impact analysis on its target requirements; the impact scope SHALL be embedded in the CR record and visible to approvers before they vote | Functional | EA, GVN | Must | Approvers need full impact context to make informed decisions |
| REQ-REQM-199 | CR Approval Workflow Configuration | Administrators SHALL be able to configure CR approval workflows: single-approver, sequential multi-approver, parallel multi-approver (all must approve), and committee-vote (majority rule); different workflows SHALL be configurable per requirement type or project area | Functional | GVN | Must | Different organizations have different change governance models |
| REQ-REQM-200 | CR Approval Notifications | The system SHALL send notifications to each approver in sequence (or in parallel) when a CR reaches their approval step; reminders SHALL be sent at configurable intervals (default: 48 hours, 96 hours) if no response is received; escalation to the approver's manager SHALL be configurable | Functional | GVN | Must | Approval workflows stall without automated reminders |
| REQ-REQM-201 | CR Implementation | When a CR is approved, the system SHALL enable the designated implementer to apply the approved changes to the target requirements; for Modify CRs, the system SHALL open the requirement editor pre-populated with the proposed changes from the CR description; upon save, the requirement version is incremented and the CR is marked Implemented | Functional | BA | Must | CR closure must be coupled to actual requirement change |
| REQ-REQM-202 | CR Rollback | If an implemented CR is subsequently found to be incorrect, authorized users SHALL be able to rollback the CR implementation, reverting the affected requirements to their pre-CR version records and reopening the CR as Draft | Functional | GVN | Should | Rollback safety net for post-implementation errors |

---

## 22. Change History, Diff & Audit

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-203 | Full Audit Log | The system SHALL maintain an immutable, append-only audit log for every action performed on any requirement or RTM link: create, view, update (field name + old + new value), delete, status change, priority change, link add/remove, baseline creation, CR submit/approve/reject/implement, sign-off, and export; every log entry SHALL include: action type, actor user ID, actor display name, timestamp (UTC), requirement ID, session ID, and source IP | Security | GVN | Must | GDPR Art. 5(1)(f); SOC 2 CC7; ISO 27001 A.12.4.1 — audit trail is a mandatory control |
| REQ-REQM-204 | Audit Log Export | The full audit log SHALL be exportable to CSV with configurable date-range and action-type filters; exports SHALL themselves be logged in the audit trail | Security | GVN | Must | Auditors require offline log analysis |
| REQ-REQM-205 | Per-Requirement Change History Panel | The requirement detail page SHALL include a chronological change history panel showing all audited events on that requirement with actor, timestamp, and changed fields; this panel SHALL be accessible to all project members, not only admins | UX | BA, GVN | Must | Transparency builds stakeholder trust in the requirements process |
| REQ-REQM-206 | Audit Log Tamper Detection | The audit log storage SHALL use a hash-chained or cryptographic integrity mechanism (e.g., SHA-256 chained log entries or Write-Once storage) such that any tampering with historical log entries is detectable; tamper alerts SHALL notify the system administrator | Security | GVN | Should | High-compliance environments (SOX, FDA) require tamper-evident audit logs |

---

## 23. E-Sign & Formal Sign-Off

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-207 | Sign-Off Workflow Initiation | Authorized users SHALL be able to initiate a formal sign-off workflow for a set of requirements (or a baseline), selecting one or more signatories from the stakeholder roster, setting a due date, and attaching a cover memo | Functional | GVN, BA | Must | Formal approval is a governance milestone for regulated industries |
| REQ-REQM-208 | In-App E-Signature | The system SHALL capture in-app e-signatures: a signatory confirms their identity (by re-authenticating or entering a PIN), reviews the requirements set, and clicks "I Approve" or "I Reject with Comments"; the approval SHALL be stored with: signatory user ID, display name, email, timestamp, and IP address | Security | GVN, STK | Must | Digital signature evidence for audit; equivalent to 21 CFR Part 11 simple e-signature |
| REQ-REQM-209 | External E-Signature Integration | The system SHALL optionally integrate with DocuSign or Adobe Sign for qualified electronic signatures on exported sign-off documents; the signed document SHALL be linked back to the relevant baseline/requirement set | Integration | GVN | Should | Regulated industries (pharma, finance) require qualified e-signatures |
| REQ-REQM-210 | Sign-Off Certificate | Upon completion of a sign-off workflow, the system SHALL generate a PDF sign-off certificate recording: project name, scope description, signatory list (name, title, email, signature timestamp), and a hash of the baselined requirement set; the certificate SHALL be archived and immutable | Reporting | GVN | Must | Contractual and regulatory compliance artifact |
| REQ-REQM-211 | Sign-Off Status Visibility | The sign-off status (Pending / In Progress / Approved / Rejected) SHALL be visible on every requirement in the signed set via a badge in list and detail views; requirements in a rejected sign-off SHALL be automatically returned to Draft status with the rejection comment appended to the change history | UX | GVN, BA | Must | Stakeholders and the team need to see approval state without opening the workflow |

---

## 24. Collaboration & Review

### 24.1 Inline Comments & Discussions

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-212 | Inline Requirement Comments | Any project member SHALL be able to add a threaded comment to any requirement; comments SHALL support rich text (Markdown), @-mention of project members (triggering notification), and attachment of screenshots or files | UX | ALL | Must | Inline discussion reduces email threads and keeps context with the requirement |
| REQ-REQM-213 | Comment Resolution | Comments SHALL support a "Resolved" toggle; open comments SHALL prevent a requirement from being transitioned to Approved (configurable guard); resolved comments SHALL be collapsible but retained for audit | Functional | BA, GVN | Must | Unresolved comments block sign-off; resolution is a governance gate |
| REQ-REQM-214 | Review Cycle Management | The system SHALL support named review cycles for a requirement set (e.g., "Iteration 2 Technical Review"), assigning specific reviewers, a review period (start/end date), and a checklist of review tasks; review progress (comments submitted, requirements marked reviewed) SHALL be tracked | Functional | BA, GVN | Must | Structured review cycles are a BABOK v3 elicitation and validation best practice |
| REQ-REQM-215 | Reviewer Assignment | Project leads SHALL be able to assign specific requirements to specific reviewers (named individual or role); assigned reviewers SHALL receive notification and the requirements SHALL appear in their personal review queue | Functional | BA, GVN | Should | Large review sets need workload distribution |
| REQ-REQM-216 | Review Completion Tracking | The system SHALL track review completion: % of assigned requirements reviewed (comment or explicit "reviewed" click) per reviewer, per review cycle; a completion summary SHALL be visible to the project lead | Reporting | GVN | Should | Review governance requires completion evidence |
| REQ-REQM-217 | Stakeholder Notification Preferences | Project members SHALL be able to configure per-requirement and per-event notification preferences (in-app / email / Teams webhook) for: new comments, status changes, priority changes, assignment changes, sign-off requests, and CR submissions | UX | ALL | Must | Notification overload is a leading cause of disengagement |

---

## 25. Stakeholder Management & RACI

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-218 | Stakeholder Registry | The system SHALL maintain a project-level stakeholder registry with: display name, email, organization, role (from a configurable role list), phone, timezone, and a free-text notes field; stakeholders may be system users or external contacts (without system login) | Data | BA, GVN | Must | BABOK v3 §3.2 stakeholder management |
| REQ-REQM-219 | RACI Matrix | The system SHALL maintain a RACI matrix at project and requirement-package level with Responsible, Accountable, Consulted, and Informed roles per stakeholder per activity type (Elicitation, Review, Approval, Implementation, Testing); the RACI SHALL be exportable to Excel | Functional | GVN, BA | Must | BABOK v3 §3.2; RACI is a standard governance artifact |
| REQ-REQM-220 | Stakeholder Engagement Scoring | The system SHALL score each stakeholder's engagement level (High/Medium/Low) based on: comment activity, review completion rate, and sign-off timeliness; low-engagement stakeholders on critical requirements SHALL be flagged for the project lead | AI/ML | BA, PM | Could | Proactive engagement management reduces requirements sign-off delays |
| REQ-REQM-221 | Stakeholder Communication Log | The system SHALL maintain a communication log per stakeholder, allowing BAs to record meeting notes, email summaries, and decisions; log entries SHALL be linkable to specific requirements or source documents | Functional | BA | Should | BABOK v3 elicitation documentation; decision audit trail |

---

## 26. BABOK Alignment & Business Glossary

### 26.1 BABOK v3 Alignment

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-222 | Elicitation Technique Logging | The system SHALL allow BAs to log the elicitation technique used to discover each requirement or source document (configurable from BABOK v3 technique list: Brainstorming, Document Analysis, Focus Group, Interface Analysis, Interview, Observation, Prototyping, Survey/Questionnaire, Workshops, etc.) | Functional | BA | Should | BABOK v3 §4.1 elicitation techniques; creates a methodological audit trail |
| REQ-REQM-223 | Solution Scope Statement | The system SHALL provide a project-level solution scope statement form capturing: business need, desired business outcomes, solution scope boundaries (in-scope/out-of-scope), key stakeholders, constraints, and assumptions; this document SHALL be linked to business requirements | Functional | BA, EA | Must | BABOK v3 §6.4 solution scope; ISO 29148 §5.1 |
| REQ-REQM-224 | Business Rules Repository | The system SHALL maintain a business rules repository within the Requirements Pillar, storing rules as discrete records (ID, name, rule statement, rule type: definition/fact/constraint/derivation, related requirements, status); rules SHALL be linkable to requirements as a special trace link type | Data | BA, DA | Must | Business rules are distinct from but tightly linked to functional requirements; BABOK v3 §9.2 |

---

### 26.2 Business Glossary

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-225 | Business Glossary Terms | The system SHALL maintain a business glossary with: term ID, preferred term, synonyms, definition (rich text), domain/category, status (Approved / Draft / Deprecated), owner, and related terms; terms SHALL be linkable to requirements, business rules, and data entities | Data | BA, DA | Must | Consistent terminology is prerequisite for unambiguous requirements; BABOK v3 §9.6 |
| REQ-REQM-226 | Automated Glossary Term Detection | The system SHALL scan requirement text and automatically detect words or phrases matching glossary terms, rendering them as hyperlinks in the requirement view that open a definition popover; unrecognized domain terms that appear ≥3 times across the project SHALL be suggested as new glossary candidates | AI/ML | BA | Must | Reduces ambiguity from undefined terms; enforces controlled vocabulary |
| REQ-REQM-227 | Glossary Conflict Detection | When two glossary terms have overlapping definitions or synonyms, the system SHALL flag the conflict and suggest merge or disambiguation; synonym collisions (same synonym mapped to two different preferred terms) SHALL be flagged as errors | Functional | BA, GVN | Should | Glossary conflicts produce requirement ambiguity |
| REQ-REQM-228 | Glossary Import / Export | The business glossary SHALL be importable from and exportable to: SKOS RDF/XML, Excel (term, definition, synonyms columns), and plain CSV; import SHALL support upsert (add new, update existing, skip unchanged) with a dry-run preview | Functional | DA, GVN | Should | Enterprises may have an existing enterprise glossary to import |

---

## 27. Requirement Reuse Library

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-229 | Reuse Library Structure | The system SHALL maintain a cross-project Requirement Reuse Library (RRL) containing approved, reusable requirement templates and patterns; RRL entries SHALL be organized by domain (Security, Performance, Usability, Integration, Data, etc.) and type | Data | BA, GVN | Must | Reuse reduces elicitation effort and enforces organizational standards |
| REQ-REQM-230 | Promote to Reuse Library | BAs SHALL be able to promote any approved requirement to the RRL; the promoted entry SHALL capture: the requirement text as a template (with placeholder variables in double-braces, e.g., `{{system_name}}`), domain, type, quality score, originating project reference, and approval date | Functional | BA, GVN | Must | Bidirectional flow between project requirements and library |
| REQ-REQM-231 | Instantiate from Reuse Library | BAs SHALL be able to search the RRL by keyword, domain, or type and instantiate one or more library entries into their project as new requirements; placeholder variables SHALL be prompted for substitution at instantiation time | UX | BA | Must | Core reuse productivity feature |
| REQ-REQM-232 | RRL Search & Semantic Matching | The system SHALL support full-text and semantic (embedding-based) search of the RRL; search results SHALL be ranked by semantic similarity to the current project context (active requirements as query context) | AI/ML | BA | Should | Embedding search surfaces relevant reusable requirements that keyword search misses |
| REQ-REQM-233 | RRL Usage Tracking | The system SHALL track how many times each RRL entry has been instantiated, across how many projects, and by whom; highly-reused entries SHALL be flagged as "Canonical Standards" | Reporting | GVN | Should | Usage data validates ROI of standardization effort |
| REQ-REQM-234 | RRL Governance | RRL entries SHALL have a governance lifecycle: Proposed → Under Review → Approved → Deprecated; only entries in Approved status SHALL be searchable by default; a GVN-role owner SHALL be required for each RRL entry | Governance | GVN | Must | Quality control for the reuse library |
| REQ-REQM-235 | Industry Template Packs | The system SHALL ship pre-loaded RRL template packs for common domains: GDPR/Privacy, Cybersecurity (NIST CSF, ISO 27001), API/Integration, Performance SLAs, Accessibility (WCAG 2.1), Healthcare (HIPAA); template packs SHALL be activatable per project by administrators | Data | GVN, BA | Should | Reduces time-to-first-requirement for regulated industries |

---

## 28. Prioritization Frameworks

### 28.1 WSJF (Weighted Shortest Job First)

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-236 | WSJF Score Fields | The system SHALL provide per-requirement numeric fields for the four SAFe WSJF input components: Business Value (1–10), Time Criticality (1–10), Risk Reduction / Opportunity Enablement (RR/OE) (1–10), and Job Size (1–10, Fibonacci scale); a computed WSJF score SHALL be calculated as (BV + TC + RR/OE) / Job Size | Data | PM | Must | SAFe PI Planning requires WSJF for backlog prioritization |
| REQ-REQM-237 | WSJF Ranking View | The system SHALL provide a sortable WSJF ranking table showing all requirements (or a filtered set) ordered by WSJF score descending, with the four input scores and computed WSJF visible as columns | UX | PM | Must | PI planning uses WSJF ranking to sequence the program backlog |
| REQ-REQM-238 | WSJF AI Score Suggestion | The system SHALL offer an AI-assisted WSJF scoring suggestion for Business Value, Time Criticality, and RR/OE based on requirement text and type; suggestions SHALL be presented as editable defaults, not auto-applied | AI/ML | PM, BA | Should | Manual WSJF scoring of 100+ requirements is time-consuming |

---

### 28.2 Kano Model

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-239 | Kano Classification | The system SHALL support Kano model classification for requirements: Must-Be (Basic), One-Dimensional (Performance), Attractive (Delighter), Indifferent, and Reverse; Kano class SHALL be an optional field on each requirement and used in dashboard filtering | Data | PM, BA | Should | Kano model is used in product strategy to balance delight vs. basics |
| REQ-REQM-240 | Kano Survey Workflow | The system SHALL support a lightweight Kano survey workflow: generate a survey from selected requirements with functional/dysfunctional question pairs, send to selected stakeholders via in-app or email link, collect responses, and auto-classify each requirement by Kano logic | Functional | PM | Could | Quantitative Kano classification requires stakeholder survey data |

---

### 28.3 Additional Prioritization Tools

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-241 | Priority Stack Ranking | The system SHALL allow PMs to drag-and-drop requirements into a forced stack rank order within a release scope; stack rank order SHALL be stored as a numeric rank field and exportable | UX | PM | Should | Explicit ordering resolves ties in MoSCoW priority |
| REQ-REQM-242 | Relative Prioritization (Pairwise Comparison) | The system SHALL offer a pairwise comparison mode for ≤30 requirements, presenting each pair and asking the user to select the higher-priority item; the system SHALL compute a derived priority ranking from the comparison matrix using a configurable algorithm (Elo/Bradley-Terry) | UX | PM, BA | Could | Pairwise comparison is more precise than absolute scoring for small sets |
| REQ-REQM-243 | Priority Override Justification | Any change to a requirement's MoSCoW, WSJF, Kano, or stack-rank priority SHALL require the user to enter a justification note (min 20 characters); the justification SHALL be appended to the change history | Governance | GVN | Should | Priority changes without justification are a governance red flag |

---

## 29. Dashboards & Reporting

### 29.1 Requirement Dashboards

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-244 | Project Health Dashboard | The system SHALL provide a project-level requirements health dashboard displaying: total requirements by status, completeness score trend, quality check pass/fail counts by rule, RTM coverage %, open CRs count, open comments count, and days since last baseline; dashboard SHALL refresh on demand and be embeddable as a widget | Reporting | BA, PM, GVN | Must | Single-pane-of-glass health view for project governance |
| REQ-REQM-245 | Elicitation Progress Dashboard | The system SHALL provide an elicitation-focused dashboard showing: ingested sources count, extraction candidates processed (accepted/rejected/pending), requirements by type and category, sources with zero accepted requirements, and elicitation activity timeline | Reporting | BA | Must | Tracks the ingestion-to-requirement conversion funnel |
| REQ-REQM-246 | Quality Metrics Dashboard | The system SHALL display quality metrics including: ambiguity flag rate, verifiability flag rate, atomic failure rate, INVEST failure rate per dimension, AC coverage %, and orphan requirement count; each metric SHALL be drillable to the list of flagged requirements | Reporting | BA, GVN | Must | Quality governance requires quantitative visibility |
| REQ-REQM-247 | Volatility Dashboard | The system SHALL compute and display requirement volatility: number of versions per requirement (heatmap), requirements changed per week (trend), requirements changed within 30 days of baseline, and top 10 most-changed requirements; high-volatility requirements SHALL be highlighted | Reporting | PM, GVN | Should | Volatility is a key project risk indicator |
| REQ-REQM-248 | Coverage Dashboard | The system SHALL display a multi-dimensional coverage dashboard: test coverage by type and priority, architecture allocation coverage by package, roadmap allocation coverage by release, and sign-off coverage by stakeholder; each dimension drills to uncovered items | Reporting | QA, EA, PM, GVN | Must | Coverage is the primary release-readiness gate |
| REQ-REQM-249 | Stakeholder Engagement Dashboard | The system SHALL display stakeholder engagement metrics per review cycle: review completion rate by stakeholder, comment activity by stakeholder, open action items by owner, and average sign-off turnaround time | Reporting | BA, GVN | Should | Low-engagement stakeholders are a governance risk |

---

### 29.2 Coverage & Quality Metrics

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-250 | Requirements Volatility Index | The system SHALL compute a Requirements Volatility Index (RVI) = (number of requirements changed in last 30 days) / (total requirements in scope); an RVI >15% SHALL trigger a quality alert visible on the project dashboard | Reporting | PM, GVN | Should | Industry benchmark: >15% volatility post-baseline indicates scope instability |
| REQ-REQM-251 | Requirement Churn Rate | The system SHALL report the requirement churn rate (requirements added + removed + re-stated per sprint or calendar week) as a time-series chart, enabling PMs to identify scope-creep episodes | Reporting | PM | Should | Churn visibility enables proactive scope management |
| REQ-REQM-252 | Source-to-Requirement Funnel | The system SHALL provide a funnel visualization: documents ingested → extraction candidates produced → candidates reviewed → candidates accepted → requirements approved → requirements baselined; conversion rates at each funnel step SHALL be displayed | Reporting | BA, PM | Should | Funnel reveals where requirements are being lost or delayed |
| REQ-REQM-253 | Custom Dashboard Builder | The system SHALL provide a drag-and-drop dashboard builder allowing users to compose personal or shared project dashboards from a widget library (metric cards, charts, tables, RTM mini-view, activity feed); dashboards SHALL be shareable via URL with read-only access | UX | ALL | Should | Power users need tailored views beyond the built-in dashboards |

---

## 30. Exports & Tool Integration

### 30.1 Export Formats

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-254 | Excel Export | The system SHALL export any filtered requirement set to Excel (.xlsx) with configurable column selection covering all schema fields including custom fields; the export SHALL include a summary sheet and per-category sub-sheets; formatting SHALL use alternating row colors for readability | Reporting | BA, PM | Must | Excel is the universal downstream tool for stakeholder distribution |
| REQ-REQM-255 | CSV Export | The system SHALL export requirements to CSV (UTF-8 with BOM, comma delimiter, RFC 4180 compliant) with header row; all fields except rich text SHALL be included; rich-text fields SHALL be exported as plain text with basic Markdown stripped | Reporting | BA | Must | CSV feeds downstream tools (DOORS import, custom scripts) |
| REQ-REQM-256 | JSON Export | The system SHALL export requirements to a structured JSON format following a documented schema (version-stamped), including all fields, ACs, custom fields, and RTM links; this export SHALL be machine-readable by downstream integration tools | Integration | DA | Must | API-grade export for automation pipelines |
| REQ-REQM-257 | Word Document Export | The system SHALL export a selected requirement set to a formatted Word (.docx) document using a configurable template, rendering: cover page, table of contents, sections by type/category, requirement tables with all fields, and an RTM appendix | Reporting | BA, GVN | Must | Word export is the standard format for requirements specification documents shared with external stakeholders |
| REQ-REQM-258 | PDF Export | The system SHALL export any requirement view (dashboard, list, detail, RTM grid) to PDF using a configurable template with organization logo and header/footer; PDF SHALL be print-optimized (A4/Letter) | Reporting | GVN, STK | Must | PDF is the standard format for formal submission to regulators or clients |
| REQ-REQM-259 | Gherkin / BDD Export | The system SHALL export all Given-When-Then acceptance criteria for a selected requirement set as Gherkin .feature files, one file per requirement (Feature), with Scenarios for each AC; the export SHALL be directly usable by Cucumber / Behave / SpecFlow | Functional | QA | Should | Gherkin export closes the loop between requirements and test automation |
| REQ-REQM-260 | ReqIF Export | The system SHALL export requirements in Requirements Interchange Format (ReqIF v1.0.1, OSLC compliant), enabling import into DOORS Next, Polarion, Jama, and other ReqIF-compliant tools | Integration | GVN, BA | Should | ReqIF is the ISO standard for requirements exchange; required for aerospace/automotive/defense programs |

---

### 30.2 Tool Imports & Integration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale / Source |
|---|---|---|---|---|---|---|
| REQ-REQM-261 | DOORS Classic Import | The system SHALL import requirement exports from IBM DOORS Classic (text-based module export format .csv / .xls) mapping standard DOORS attributes (Object Identifier, Object Heading, Object Text, Object Short Text) to the REQM schema; a field-mapping wizard SHALL be provided | Integration | BA | Should | Many regulated-industry customers have legacy DOORS repositories |
| REQ-REQM-262 | DOORS Next / DNG Import | The system SHALL import requirements from IBM DOORS Next Generation via the OSLC RM API v2.1, mapping artifact types, attributes, and link types to the REQM schema | Integration | BA | Should | DNG is the modern IBM RM tool |
| REQ-REQM-263 | Jama Connect Import | The system SHALL import requirements from Jama Connect via the Jama REST API v1, mapping items, categories, and relationships to the REQM schema | Integration | BA | Could | Jama is used in medical device and defense programs |
| REQ-REQM-264 | ReqIF Import | The system SHALL import ReqIF v1.0.1 documents, mapping ReqIF attribute definitions to REQM schema fields via a configurable mapping wizard; unmapped attributes SHALL be created as custom fields | Integration | BA | Should | Reciprocal to REQ-REQM-260; enables round-trip with RM tools |
| REQ-REQM-265 | Excel Template Import | The system SHALL accept Excel files structured to a downloadable ESIP import template, mapping column headers to requirement schema fields and bulk-creating requirements from rows; a validation pre-flight SHALL detect missing mandatory fields before import | Functional | BA | Must | Excel import is the most common migration path from informal tracking |
| REQ-REQM-266 | Jira Two-Way Sync | The system SHALL support configurable two-way synchronization with Jira: (a) ESIP → Jira: push requirement changes to linked Jira issues; (b) Jira → ESIP: pull issue updates back to linked requirements; conflict resolution policy (ESIP wins / Jira wins / prompt user) SHALL be configurable per field | Integration | BA, PM | Must | Teams that manage delivery in Jira need bidirectional sync to avoid split management |
| REQ-REQM-267 | ADO Two-Way Sync | The system SHALL support configurable two-way synchronization with Azure DevOps work items equivalent to REQ-REQM-266, supporting ADO Cloud and Server ≥2019 | Integration | BA, PM | Must | ADO is the primary ALM for Microsoft-centric enterprises |
| REQ-REQM-268 | Webhook Outbound Events | The system SHALL support configurable outbound webhooks (HTTP POST, JSON payload) for events: requirement.created, requirement.updated, requirement.status_changed, baseline.created, cr.approved, sign_off.completed; webhooks SHALL support HMAC-SHA256 request signing | Integration | DA | Should | Enables custom downstream automation (Slack notifications, CI triggers, data pipelines) |
| REQ-REQM-269 | REST API for Requirements | The system SHALL expose a versioned REST API (OpenAPI 3.1 documented) for all REQM operations: CRUD requirements, manage ACs, manage RTM links, list baselines, submit CRs, list ingestion jobs; the API SHALL use OAuth 2.0 Bearer token authentication and support pagination, filtering, and sorting | Integration | DA, ALL | Must | Programmatic access is required for enterprise integration and automation |
| REQ-REQM-270 | Import Conflict Resolution | When importing requirements that conflict with existing records (same human-readable ID, same text hash), the system SHALL present a conflict resolution UI: skip, overwrite, or create as new version; bulk-resolution policies SHALL be configurable | Functional | BA | Must | Import without conflict handling corrupts the requirements catalog |

---

*End of Volume 07 — Requirements Pillar*

---

## Appendix A — Standards Referenced

| Standard | Relevance to This Volume |
|---|---|
| ISO/IEC/IEEE 29148:2018 | Requirements engineering process, requirement attributes, quality characteristics |
| IEEE Std 830-1998 | Classic SRS structure; quality attributes (unambiguous, complete, verifiable, consistent, modifiable, traceable, correct) |
| BABOK v3 (IIBA) | Requirements lifecycle, elicitation techniques, stakeholder analysis, RACI, business rules, glossary |
| SAFe 6.0 | WSJF scoring, PI planning, epics/features/stories hierarchy |
| ReqIF v1.0.1 (OMG) | Requirements interchange format for tool integration |
| OSLC RM v2.1 | Open Services for Lifecycle Collaboration — Requirements Management |
| DSDM Agile | MoSCoW prioritization framework |
| ISO 25010:2011 | NFR quality model (sub-types for REQ-REQM-106) |
| GDPR Article 5 | Audit trail integrity requirements |
| SOC 2 CC7 | Change management and audit logging controls |
| ISO 27001 A.12.4.1 | Event logging control |
| 21 CFR Part 11 | Electronic signature requirements for FDA-regulated industries |
