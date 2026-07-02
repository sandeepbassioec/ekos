# Volume 08 — Knowledge Hub & Enterprise Connectors
## ESIP Architecture Automation Suite

| Attribute | Value |
|---|---|
| Volume | 08 |
| Prefix | KH |
| Status | Baseline Draft |
| Date | 2026-06-23 |
| Scope | Knowledge Hub pillar (evidence base, ingestion, storage, lifecycle), enterprise connectors (Jira, Confluence, SharePoint/OneDrive, Microsoft Graph, Azure DevOps, ServiceNow+CMDB, GitHub/GitLab/Bitbucket, Slack, Google Workspace, Box/Dropbox, webhooks, inbound email, RSS/web), AI capabilities over the hub (search, Q&A, summarization, tagging, extraction, discovery), provenance and governance, and operational/UX capabilities |
| Out of Scope | AI model/RAG internals (V06); extraction of evidence into managed requirements (V07); identity/SSO/secrets platform infrastructure (V11); code repositories as portfolio-scan source for APM/tech-debt (V09) — GitHub/GitLab/Bitbucket are in scope here only as *document and knowledge* connectors (README, wikis, issues, PRs, release notes) |

---

## Personas Reference

| Code | Persona |
|---|---|
| EA | Enterprise Architect |
| SA | Solution Architect |
| AA | Application Architect |
| IA | Integration/API Architect |
| DA | Data Architect |
| CA | Cloud/Infrastructure Architect |
| SecA | Security Architect |
| MA | Microservices Architect |
| CRM | CRM/Platform Architect |
| ALL | All personas |

---

## Table of Contents

1. [Knowledge Hub — Core Model](#1-knowledge-hub--core-model)
2. [Evidence Types & Manual Entry](#2-evidence-types--manual-entry)
3. [Meetings — Transcripts & Recordings](#3-meetings--transcripts--recordings)
4. [Email Evidence (Outlook/IMAP/Inbound)](#4-email-evidence-outlookimapibound)
5. [Chat Evidence (Teams/Slack)](#5-chat-evidence-teamsslack)
6. [File Upload & Document Ingestion](#6-file-upload--document-ingestion)
7. [Workspace & Project Scoping](#7-workspace--project-scoping)
8. [Folders, Collections & Tagging](#8-folders-collections--tagging)
9. [Connector Framework — Common Capabilities](#9-connector-framework--common-capabilities)
10. [Jira Connector](#10-jira-connector)
11. [Confluence Connector](#11-confluence-connector)
12. [SharePoint / OneDrive Connector](#12-sharepoint--onedrive-connector)
13. [Microsoft Graph — Mail / Calendar / Teams](#13-microsoft-graph--mail--calendar--teams)
14. [Azure DevOps Connector](#14-azure-devops-connector)
15. [ServiceNow + CMDB Connector](#15-servicenow--cmdb-connector)
16. [GitHub / GitLab / Bitbucket Connector (Document/Knowledge)](#16-github--gitlab--bitbucket-connector-documentknowledge)
17. [Slack Connector](#17-slack-connector)
18. [Google Workspace — Drive / Gmail / Meet](#18-google-workspace--drive--gmail--meet)
19. [Box / Dropbox Connector](#19-box--dropbox-connector)
20. [Generic Webhook / REST Connector](#20-generic-webhook--rest-connector)
21. [Inbound Email (IMAP / Forwarding)](#21-inbound-email-imap--forwarding)
22. [RSS / Web Page Connector](#22-rss--web-page-connector)
23. [Storage & Artifact Lifecycle](#23-storage--artifact-lifecycle)
24. [AI Over the Knowledge Hub](#24-ai-over-the-knowledge-hub)
25. [Provenance & Governance](#25-provenance--governance)
26. [Operational & UX Capabilities](#26-operational--ux-capabilities)

---

## 1. Knowledge Hub — Core Model

### 1.1 Hub Architecture

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KH-001 | Unified Evidence Repository | The system SHALL maintain a unified Knowledge Hub as the canonical evidence store for all architecture work — the single authoritative collection of documents, meeting records, email threads, chat excerpts, connector-synced artifacts, and manually entered notes that underpin requirements, decisions, and deliverables | Functional | ALL | Must | Evidence traceability without a unified store produces orphaned citations |
| REQ-KH-002 | Dual Scoping — Item and Workspace | The system SHALL support evidence items scoped either to a specific roadmap item/architecture deliverable (per-item Knowledge Hub, the existing ESIP feature) or to an entire project/workspace (cross-item discovery), and SHALL allow the same source artifact to be referenced in both scopes without duplication of the underlying storage object | Functional | ALL | Must | Per-item hub exists; enterprise vision requires workspace-level cross-cutting evidence |
| REQ-KH-003 | Artifact Identity & Deduplication | The system SHALL assign a globally unique, stable internal identifier (KH artifact ID) to every ingested artifact and SHALL detect duplicate content arriving from multiple connectors or upload paths using content hash + source URI matching, merging metadata while retaining a single canonical storage object | Data | ALL | Must | Multi-connector environments inevitably push the same file/email from multiple sources |
| REQ-KH-004 | Artifact Status Lifecycle | The system SHALL track each Knowledge Hub artifact through a defined status lifecycle: `Pending` → `Indexed` → `Stale` (when source is modified) → `Archived` → `Deleted`, and SHALL surface current status to users in every view | Data | ALL | Must | Stale evidence silently misdirects AI and architects |
| REQ-KH-005 | Metamodel Integration | The system SHALL integrate the Knowledge Hub artifact model into the platform extensible metamodel (V11) so that artifact types, connector types, tag taxonomies, and retention policies are configurable without code changes | Functional | ALL | Should | Supports extensibility goal across personas |
| REQ-KH-006 | Cross-Project Search Boundary | The system SHALL enforce workspace/project access boundaries in all Knowledge Hub operations — search, Q&A, and AI analysis results SHALL never surface artifacts from projects the requesting user is not authorized to access | Security | ALL | Must | Multi-tenant confidentiality requirement |
| REQ-KH-007 | Offline / Degraded Mode | The system SHALL continue to serve previously indexed artifact content to authorized users in read-only mode when external connector systems are temporarily unavailable, clearly indicating the last-synced timestamp for each artifact | NFR | ALL | Should | Connector outages must not block ongoing architecture work |

### 1.2 Hub Navigation & Views

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KH-008 | Hub Landing Dashboard | The system SHALL provide a Knowledge Hub landing view per project/workspace showing: total artifact count by type, connector sync health, recently added/updated items, items flagged for review, and quick-access search | UX | ALL | Must | Orientation view required for enterprise-scale hubs |
| REQ-KH-009 | Evidence Timeline View | The system SHALL provide a chronological timeline view of all evidence items with filtering by type, connector, author, tag, and date range, enabling architects to reconstruct the information journey of a decision | UX | EA, SA | Should | Decision retrospective capability |
| REQ-KH-010 | Related Items Panel | The system SHALL surface a "related evidence" panel alongside any open artifact, showing AI-derived semantic relationships to other artifacts in the workspace hub, with confidence scores and relationship type labels | UX | ALL | Should | Reduces duplicate research effort |

---

## 2. Evidence Types & Manual Entry

### 2.1 Supported Evidence Categories

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KH-011 | Document Evidence Type | The system SHALL support document evidence artifacts: Word (.docx/.doc), PDF, PowerPoint (.pptx/.ppt), Excel (.xlsx/.xls/.csv), plain text (.txt/.md/.rst), RTF, and OpenDocument formats, with type-specific metadata extraction | Functional | ALL | Must | Primary document types used by architects |
| REQ-KH-012 | Meeting Evidence Type | The system SHALL support meeting evidence artifacts comprising: transcript text, recording reference (URI or stored blob), meeting metadata (title, date, attendees, platform), AI-generated summary, and extracted action items and decisions | Functional | ALL | Must | Architecture decisions are heavily made in meetings |
| REQ-KH-013 | Email Evidence Type | The system SHALL support email evidence artifacts comprising: sender, recipients (To/CC/BCC), date, subject, body text, inline images (stored), attachments (linked as child document artifacts), thread-ID for threading, and conversation-level grouping | Functional | ALL | Must | Email threads capture stakeholder intent and approvals |
| REQ-KH-014 | Chat Message Evidence Type | The system SHALL support chat message evidence artifacts comprising: message text, author, timestamp, channel/conversation context, platform origin (Teams/Slack/etc.), thread/reply chain, and linked attachments or shared files | Functional | ALL | Must | Chat is a primary async decision channel |
| REQ-KH-015 | Issue / Ticket Evidence Type | The system SHALL support issue/ticket evidence artifacts from Jira, Azure DevOps, ServiceNow, and GitHub Issues comprising: issue ID, title, description, comments, status, assignee, priority, labels/components, and linked artifacts | Functional | ALL | Must | Requirements and decisions are often captured as issues |
| REQ-KH-016 | Web Page / Feed Evidence Type | The system SHALL support web page and RSS feed evidence artifacts comprising: URL, title, snapshot timestamp, scraped body text, and extracted metadata (author, publish date where available) | Functional | EA, SA | Should | External standards, vendor docs, and industry articles are evidence |
| REQ-KH-017 | Manual Note Evidence Type | The system SHALL support manually authored evidence notes — rich text notes created directly in the platform with title, body (Markdown), author, date, and workspace/item scope — as first-class Knowledge Hub artifacts with the same tagging, search, and AI capabilities as ingested evidence | Functional | ALL | Must | Allows capturing institutional knowledge not in any external system |
| REQ-KH-018 | CMDB / Configuration Item Evidence Type | The system SHALL support CMDB/configuration item evidence artifacts from ServiceNow or similar, comprising: CI name, CI type, business service linkages, attribute set, and change history references, for use in technology landscape and impact analysis | Functional | EA, SA, CA | Should | CMDB data underpins technology portfolio and impact analysis |

### 2.2 Manual Entry Capabilities

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KH-019 | Rich-Text Note Editor | The system SHALL provide an in-platform rich-text editor (Markdown with preview, supporting headings, tables, code blocks, embedded images, and inline citations to other KH artifacts) for manual evidence note creation and editing | UX | ALL | Must | Architects need low-friction note capture |
| REQ-KH-020 | Paste-to-Ingest | The system SHALL allow users to paste raw text or email body content into a creation dialog that automatically classifies the pasted content as a note, email excerpt, or meeting excerpt based on structure heuristics, pre-filling metadata fields accordingly | UX | ALL | Should | Reduces friction for quick capture |
| REQ-KH-021 | Template-Based Note Creation | The system SHALL provide configurable note templates (e.g., Decision Record, Meeting Note, Stakeholder Interview Summary, Architecture Review Feedback) selectable at creation time, pre-populating section headings and metadata fields | UX | ALL | Should | Structured notes improve AI extraction yield |
| REQ-KH-022 | Manual Metadata Override | The system SHALL allow any automatically extracted metadata field (source date, author, classification, tags) on any artifact to be manually overridden by an authorized user, with the override recorded in audit log and displayed as "manually set" | Functional | ALL | Must | Auto-extracted metadata is sometimes incorrect |

---

## 3. Meetings — Transcripts & Recordings

> **Boundary note (reconciliation).** The Teams / Outlook / Zoom meeting integrations in this volume (incl. §13 Microsoft Graph transcript/recording pull) are **POST-meeting transcript and recording ingestion** only — evidence is fetched *after* a meeting ends. **Real-time / live capture** (in-meeting bot, live transcription/notes, live attendance) is owned by **V17 (REQ-RTC)**. This prevents two competing Microsoft Graph meeting integrations; see 00-MASTER-INDEX §6.

### 3.1 Transcript Ingestion

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHMT-001 | Transcript File Upload | The system SHALL accept meeting transcript file uploads in VTT, SRT, DOCX, TXT, and JSON formats (Teams/Zoom/Webex native export formats) and SHALL parse speaker labels, timestamps, and utterances into a structured transcript model | Functional | ALL | Must | All major platforms export transcripts in these formats |
| REQ-KHMT-002 | Teams Meeting Transcript Pull | The system SHALL retrieve Teams meeting transcripts automatically via Microsoft Graph API (callRecords and onlineMeetings endpoints) for meetings linked to the workspace calendar integration, storing the transcript as a Knowledge Hub artifact with meeting metadata | Integration | ALL | Must | Microsoft Graph `/communications/callRecords` + `/onlineMeetings/{id}/transcripts` |
| REQ-KHMT-003 | Zoom Transcript Pull | The system SHALL retrieve Zoom meeting transcripts via Zoom API (OAuth 2.0 server-to-server) for configured Zoom accounts, fetching VTT transcript files and storing them with linked recording metadata | Integration | ALL | Should | Zoom Recording API: GET `/recordings/{meetingId}` |
| REQ-KHMT-004 | Speaker Diarization Linking | The system SHALL attempt to resolve speaker labels in transcripts to known workspace user profiles (by name matching and calendar attendee cross-reference), flagging unresolved speakers for manual assignment | Functional | ALL | Should | Speaker resolution enables per-person decision attribution |
| REQ-KHMT-005 | Transcript Search & Navigation | The system SHALL provide full-text search within transcripts with highlighted match context, speaker filter, time-range jump, and keyword occurrence timeline so architects can navigate long transcripts efficiently | UX | ALL | Must | Transcripts of 2–4-hour architecture reviews require targeted navigation |
| REQ-KHMT-006 | Meeting Segment Bookmarking | The system SHALL allow users to bookmark named segments of a transcript (by timestamp range) as independent evidence excerpts linked to the parent transcript, enabling precise citation of a specific discussion moment | Functional | ALL | Should | Architectural decisions are often a 3-minute segment in a 2-hour meeting |

### 3.2 Recording Management

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHMT-007 | Recording Reference Storage | The system SHALL store meeting recording references as URIs pointing to the source platform (Teams Stream, Zoom Cloud, SharePoint) rather than re-storing recording binaries by default, but SHALL support optional local storage of recordings up to a configurable size limit for compliance/legal-hold scenarios | Data | ALL | Should | Recording binaries are large; re-storage doubles cost |
| REQ-KHMT-008 | Recording Timestamp Deep-Link | The system SHALL store deep-link URLs per transcript bookmark that open the source platform recording at the correct timestamp, enabling one-click verification of cited content | UX | ALL | Should | Evidence verification requires hearing original context |
| REQ-KHMT-009 | Transcript-Recording Synchronization | The system SHALL visually synchronize transcript text navigation with recording playback (where recording is embedded or linked) so scrolling the transcript highlights the current playback position and vice versa | UX | ALL | Could | High usability value for evidence review; technically complex |

---

## 4. Email Evidence (Outlook/IMAP/Inbound)

### 4.1 Outlook / Graph Email Integration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHEM-001 | Outlook Folder Watch | The system SHALL allow users to designate Outlook mail folders (via Microsoft Graph API with delegated `Mail.ReadWrite` scope) as watched sources; all emails in those folders SHALL be automatically ingested as evidence artifacts on a configurable polling interval (default: 15 minutes) | Integration | ALL | Must | Architecture-related emails are curated by architects into project folders |
| REQ-KHEM-002 | Email Thread Grouping | The system SHALL group ingested emails by conversation thread (using `conversationId` from Microsoft Graph) and present the full thread as a single evidence unit with individual message drill-down, ensuring architects see the complete decision thread | Functional | ALL | Must | Individual emails without thread context lose meaning |
| REQ-KHEM-003 | Email Attachment Extraction | The system SHALL automatically extract email attachments as child document artifacts, inheriting the parent email thread's metadata tags and security permissions, and cross-link them to the parent email artifact | Functional | ALL | Must | Key architecture documents are often sent as email attachments |
| REQ-KHEM-004 | Selective Ingestion via Label/Tag | The system SHALL support selective email ingestion triggered by application of a configurable Outlook category or label (e.g., "ESIP-Evidence") to an email, allowing architects to curate evidence without moving emails to a watched folder | Functional | ALL | Should | Reduces noise from broad folder watches |

### 4.2 IMAP / Forwarding

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHEM-005 | IMAP Mailbox Connector | The system SHALL provide an IMAP connector capable of connecting to any IMAP-compatible mailbox using IMAP4/SSL credentials (stored as platform secrets per V11), polling for new messages in designated folders, and ingesting them as email evidence artifacts | Integration | ALL | Should | Supports non-Microsoft mail environments (Gmail, generic corporate IMAP) |
| REQ-KHEM-006 | Inbound Email Forwarding Address | The system SHALL provision per-workspace and per-item unique inbound email addresses (e.g., `{workspace-id}@evidence.esip.internal`) so that users can forward any email from any client to the Knowledge Hub, which SHALL parse and store it as an email artifact scoped to the target workspace/item | Integration | ALL | Must | Zero-friction capture: forward email = evidence stored |
| REQ-KHEM-007 | Email PII Detection on Ingest | The system SHALL scan email bodies and subject lines for PII patterns (names, email addresses, phone numbers, national ID formats) during ingestion, tagging detected PII fields and applying configured redaction or masking policies before storage | Security | ALL | Must | Emails frequently contain personal data subject to GDPR/privacy regulations |

---

## 5. Chat Evidence (Teams/Slack)

### 5.1 Microsoft Teams Chat

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHCH-001 | Teams Channel Watch | The system SHALL allow configuration of specific Microsoft Teams channels as watched sources (via Graph API with `ChannelMessage.Read.All` application permission), ingesting messages and replies on a configurable incremental sync schedule (default: real-time via Change Notifications, fallback: 5-minute poll) | Integration | ALL | Must | Architecture decisions frequently occur in dedicated Teams channels |
| REQ-KHCH-002 | Teams Chat (1:1 and Group) Watch | The system SHALL support user-initiated capture of specific 1:1 and group chat conversations (via Graph API with delegated `Chat.ReadBasic` scope + explicit user consent per conversation) as evidence, respecting user privacy by requiring per-conversation opt-in | Integration | ALL | Should | Some key decisions happen in ad-hoc chats; capture must be consent-based |
| REQ-KHCH-003 | Teams Message Thread Preservation | The system SHALL ingest Teams message threads (parent message + all replies) as a single evidence unit, preserving reply indentation, reaction counts, and @mention resolution, presenting the full thread in a readable format | Functional | ALL | Must | Threads are the unit of deliberation, not individual messages |
| REQ-KHCH-004 | Teams Adaptive Card & File Share Capture | The system SHALL detect and record Teams messages that contain file shares (SharePoint links) or Adaptive Card attachments, creating linked references to the shared files in the Knowledge Hub even when the message body text is minimal | Functional | ALL | Should | Key documents are shared via Teams file links |

### 5.2 Slack Chat

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHCH-005 | Slack Channel Watch | The system SHALL provide a Slack connector (OAuth 2.0 with `channels:history`, `channels:read`, `users:read` scopes) enabling configuration of specific public or private channels as watched sources with incremental sync using the Slack Conversations API | Integration | ALL | Must | Slack is used by engineering/architecture teams at many enterprises |
| REQ-KHCH-006 | Slack Thread Ingestion | The system SHALL ingest Slack message threads (parent + all thread replies) as atomic evidence units, resolving user IDs to display names, handling Slack mrkdwn formatting, and preserving emoji reactions as metadata | Functional | ALL | Must | Slack threads are the primary deliberation unit |
| REQ-KHCH-007 | Slack File Attachment Linking | The system SHALL detect Slack messages containing file uploads, download the file content via Slack Files API (respecting file access permissions), store it as a child document artifact, and cross-link it to the parent message evidence | Functional | ALL | Should | Architecture diagrams and specs are routinely shared in Slack |
| REQ-KHCH-008 | Slack Emoji-Trigger Capture | The system SHALL support a configurable emoji reaction (e.g., `:evidence:` or `:esip:`) that, when added to any Slack message by an authorized user, triggers immediate ingestion of that message thread as a Knowledge Hub artifact in the designated workspace | Functional | ALL | Should | Allows curated capture without watching entire channels |

---

## 6. File Upload & Document Ingestion

### 6.1 Direct Upload

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KH-023 | Multi-File Drag-and-Drop Upload | The system SHALL provide a drag-and-drop upload interface accepting multiple files simultaneously (up to 50 files per batch), with real-time per-file progress indicators, virus scanning, format validation, and error reporting per file | UX | ALL | Must | Bulk document ingestion is a common first-step onboarding action |
| REQ-KH-024 | Large File Upload (Resumable) | The system SHALL support resumable multipart uploads for files up to 5 GB using a chunked upload protocol (TUS or equivalent), so that large recordings, Visio models, or archive files can be uploaded reliably over slow or interrupted connections | Functional | ALL | Should | Architecture repositories often contain large Visio or EA tool exports |
| REQ-KH-025 | Archive Expansion | The system SHALL automatically expand uploaded ZIP, TAR, and 7Z archives, ingesting each contained file as an individual artifact while retaining the archive structure as folder metadata | Functional | ALL | Should | Architecture deliverable packages are routinely zipped |
| REQ-KH-026 | Visio / Draw.io / ArchiMate Tool File Ingest | The system SHALL accept Visio (.vsdx/.vsd), draw.io (.drawio/.xml), and ArchiMate Exchange Format (.archimate/.xml) file uploads, parsing diagram metadata (element names, relationship types, layer annotations) into structured artifact metadata for Knowledge Hub indexing | Functional | EA, SA, AA | Must | Architects produce large Visio/ArchiMate libraries as primary deliverables |
| REQ-KH-027 | Duplicate Upload Detection | The system SHALL detect when a newly uploaded file is a byte-for-byte or near-duplicate (≥95% content similarity using fuzzy hash) of an existing artifact in the workspace hub and SHALL prompt the user to confirm whether to update the existing artifact, create a new version, or discard | Functional | ALL | Must | Prevents hub pollution with identical or near-identical uploads |

### 6.2 Format-Specific Processing

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KH-028 | OCR for Scanned PDFs and Images | The system SHALL apply OCR (Optical Character Recognition) to scanned PDFs and image files (PNG, JPG, TIFF, BMP) during ingestion, producing searchable text content stored alongside the original file, with OCR confidence score tracked per artifact | Functional | ALL | Must | Many architecture deliverables from legacy projects exist only as scanned PDFs |
| REQ-KH-029 | Email File (.eml / .msg) Upload | The system SHALL accept uploaded .eml and .msg files, parse them as email evidence artifacts (extracting sender, recipients, date, body, attachments), and treat them with the same capabilities as connector-ingested emails | Functional | ALL | Must | Architects frequently export and share email evidence as .eml/.msg files |
| REQ-KH-030 | Audio/Video Transcription (Uploaded Files) | The system SHALL accept uploaded audio (MP3, WAV, M4A) and video (MP4, MOV, WebM) files, submit them to a configured speech-to-text service (Azure Cognitive Services / OpenAI Whisper), and store the resulting transcript as a meeting evidence artifact linked to the original media file | Functional | ALL | Should | Architecture review recordings are often shared outside of Teams/Zoom |

---

## 7. Workspace & Project Scoping

### 7.1 Scope Hierarchy

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KH-031 | Four-Level Scope Hierarchy | The system SHALL support a four-level Knowledge Hub scope hierarchy: Organization → Program/Portfolio → Project/Workspace → Architecture Item, with each level having its own artifact collections, connectors, and access controls, and with artifacts optionally promoted upward in scope | Functional | EA, SA | Must | Enterprise EA programs span multiple projects; evidence must be organized hierarchically |
| REQ-KH-032 | Per-Item Knowledge Hub | The system SHALL maintain a per-roadmap-item knowledge hub (the existing ESIP capability) containing evidence directly relevant to that item, with the item hub being a filtered view derived from the workspace hub rather than a separate storage silo | Functional | ALL | Must | Preserves existing per-item hub feature while extending to workspace level |
| REQ-KH-033 | Cross-Workspace Artifact Referencing | The system SHALL allow authorized users to reference artifacts from other workspaces within the same organization as evidence in the current workspace hub, with read access controlled by the source workspace permissions, without copying the artifact | Functional | EA | Should | EA programs reference artifacts from multiple domain architecture workspaces |
| REQ-KH-034 | Global Organization Hub | The system SHALL provide an organization-wide Knowledge Hub view aggregating all artifacts across all projects the requesting user can access, with unified search, tag browsing, and analytics, functioning as the organization's architecture knowledge repository | Functional | EA | Should | Enterprise architecture repositories require organization-wide knowledge management |

---

## 8. Folders, Collections & Tagging

### 8.1 Organization Structure

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KH-035 | Virtual Folder Hierarchy | The system SHALL provide a virtual folder hierarchy for organizing Knowledge Hub artifacts, where folders are metadata groupings (not storage paths), supporting unlimited nesting, multi-parent (artifact in multiple folders), and drag-and-drop reorganization | UX | ALL | Must | Architects use folder structures to organize evidence by domain, phase, or deliverable |
| REQ-KH-036 | Smart Collections | The system SHALL support smart collections — saved search queries that dynamically populate with all artifacts matching the defined criteria (tags, type, connector, date range, keyword) — updating automatically when new artifacts are ingested | Functional | ALL | Should | Dynamic grouping reduces manual curation overhead |
| REQ-KH-037 | Named Collections (Manual) | The system SHALL support named manual collections — curated sets of specific artifacts assembled by a user, shareable with workspace members — functioning as a "reading list" or "evidence package" for a specific architecture decision or review | Functional | ALL | Must | Architects need to package and share specific evidence sets for reviews |

### 8.2 Tagging & Classification

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KH-038 | Manual Tag Assignment | The system SHALL allow users to assign one or more free-text and/or controlled-vocabulary tags to any artifact, with workspace-level and organization-level tag taxonomies managed by administrators, supporting bulk tag operations on selected artifacts | Functional | ALL | Must | Tagging is the primary evidence organization mechanism |
| REQ-KH-039 | AI Auto-Tagging on Ingest | The system SHALL automatically suggest up to 10 tags for each newly ingested artifact using AI content analysis, presenting suggestions for user acceptance/rejection; accepted tags shall be applied immediately, and the acceptance signal used to improve future suggestions | AI/ML | ALL | Must | Manual tagging of hundreds of connector-synced artifacts is not scalable |
| REQ-KH-040 | Architecture Domain Classification | The system SHALL automatically classify each artifact into one or more architecture domains (Business, Application, Data, Technology, Security, Integration, Cloud, Infrastructure) using AI content analysis, displayed as a classification badge and usable as a filter | AI/ML | ALL | Must | Domain classification enables persona-scoped searches |
| REQ-KH-041 | Framework Relevance Tagging | The system SHALL automatically detect references to architecture frameworks (TOGAF, ArchiMate, Zachman, C4, DoDAF, MODAF, ITIL, COBIT, etc.) within artifact content and tag artifacts with the detected frameworks, enabling framework-scoped evidence filtering | AI/ML | EA, SA | Should | Framework-linked evidence aids structured ADM phase work |
| REQ-KH-042 | Tag Hierarchy & Synonyms | The system SHALL support hierarchical tag relationships (parent/child tags) and synonym mappings within the tag taxonomy, so that searching for a parent tag returns artifacts tagged with any descendant, and searching for a synonym returns artifacts tagged with the canonical form | Functional | ALL | Should | Flat tag vocabularies degrade at enterprise scale |
| REQ-KH-043 | Bulk Tag Operations | The system SHALL support bulk tag operations (add, remove, replace tags) on user-selected sets of artifacts via a multi-select action panel, with undo capability within a 60-second window | UX | ALL | Must | Initial tagging of imported archives requires bulk operations |

---

## 9. Connector Framework — Common Capabilities

### 9.1 Authentication & Authorization

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHCF-001 | OAuth 2.0 Authorization Code Flow | The system SHALL implement OAuth 2.0 Authorization Code Flow with PKCE for all connectors that support it (Jira Cloud, Confluence Cloud, Microsoft Graph, GitHub, Slack, Google Workspace, Box, Dropbox, ServiceNow OAuth), storing access and refresh tokens encrypted in the platform secrets vault (V11) | Security | ALL | Must | OAuth 2.0 PKCE is the current secure standard for delegated API access |
| REQ-KHCF-002 | Service Account / App Credential Auth | The system SHALL support service account authentication (API key, Basic Auth, OAuth 2.0 Client Credentials) for connectors used in server-to-server integration contexts (Jira Server, Confluence Server, ServiceNow, ADO), with credentials stored in the platform secrets vault | Security | ALL | Must | Enterprise on-prem connectors do not support user OAuth flows |
| REQ-KHCF-003 | Token Refresh Automation | The system SHALL automatically refresh OAuth access tokens before expiry using stored refresh tokens, retrying with exponential backoff on failure, and alerting workspace administrators when a connector's token cannot be refreshed (e.g., revocation) before the connector is paused | Functional | ALL | Must | Expired tokens silently break sync; proactive refresh prevents data gaps |
| REQ-KHCF-004 | Connector Credential Isolation | The system SHALL isolate connector credentials at workspace level by default (credentials set by workspace admins, usable only within that workspace), with an option for organization-level shared connectors managed by organization admins | Security | ALL | Must | Prevents cross-workspace credential exposure |
| REQ-KHCF-005 | Scoped Permission Request (Least Privilege) | The system SHALL request only the minimum OAuth scopes required for the connector's configured capabilities; if optional capabilities (e.g., calendar read) are not configured, their scopes SHALL NOT be requested | Security | ALL | Must | Least-privilege principle; reduces customer security review surface |

### 9.2 Sync Modes & Scheduling

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHCF-006 | Full Sync | The system SHALL support a configurable full synchronization mode for each connector that crawls all accessible content within the configured scope, rebuilds the artifact index, and resolves deletions (artifacts present in ESIP but absent from source are marked `Deleted`) | Functional | ALL | Must | Required for initial setup and periodic consistency checks |
| REQ-KHCF-007 | Incremental Sync | The system SHALL support incremental synchronization using each source platform's change-tracking mechanism (Jira changelog, Confluence `lastModified`, Graph delta queries, GitHub Events API, etc.) so that only new or changed artifacts are fetched per sync cycle, minimizing API calls | Functional | ALL | Must | Full sync of large Confluence instances takes hours; incremental is essential |
| REQ-KHCF-008 | Real-Time Sync via Webhooks | The system SHALL support real-time event-driven sync for connectors that offer webhooks or push notifications (Jira webhooks, GitHub webhooks, Slack Events API, Microsoft Graph change notifications, Google Pub/Sub), processing events within 30 seconds of source change | Functional | ALL | Must | Near-real-time evidence freshness is required for active project phases |
| REQ-KHCF-009 | Sync Schedule Configuration | The system SHALL allow administrators to configure per-connector sync schedules using cron expressions (for incremental sync) and a separate schedule for periodic full-sync baseline runs, with timezone support | Functional | ALL | Must | Different connectors warrant different cadences (email: every 15 min; web pages: daily) |
| REQ-KHCF-010 | On-Demand Sync Trigger | The system SHALL provide a "Sync Now" action on each connector configuration panel that triggers an immediate incremental sync, with real-time progress feedback and completion notification | UX | ALL | Must | Architects need to pull specific new content immediately without waiting for schedule |

### 9.3 Rate Limiting & Resilience

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHCF-011 | Per-Connector Rate Limit Compliance | The system SHALL comply with each source platform's published API rate limits; when a rate limit response (HTTP 429) is received, the connector SHALL pause, respect the `Retry-After` header (or apply a default backoff), and resume automatically without data loss | NFR | ALL | Must | API over-consumption results in connector suspension; mandatory for enterprise connectors |
| REQ-KHCF-012 | Exponential Backoff with Jitter | The system SHALL implement exponential backoff with full jitter for all transient connector errors (5xx responses, network timeouts), with a maximum retry count and maximum backoff cap configurable per connector | NFR | ALL | Must | Jitter prevents thundering-herd reconnects after source platform restarts |
| REQ-KHCF-013 | Connector Circuit Breaker | The system SHALL implement a circuit breaker pattern per connector: after a configurable number of consecutive failures (default: 5), the connector enters `Open` state and stops attempting sync; it transitions to `Half-Open` after a configurable cooldown period and resumes only on a successful probe | NFR | ALL | Must | Prevents resource exhaustion from failed connectors cascading to ESIP platform |
| REQ-KHCF-014 | Sync Error Queue & Replay | The system SHALL maintain a per-connector error queue recording each failed artifact fetch with error type, timestamp, and source reference; administrators SHALL be able to inspect the queue, dismiss individual errors, or trigger a replay of failed fetches after the root cause is resolved | Functional | ALL | Must | Connector errors during large syncs must not silently lose evidence |
| REQ-KHCF-015 | Connector Health Score | The system SHALL compute and display a real-time health score (0–100) for each connector based on: % of scheduled syncs completing without error, average sync latency vs. baseline, error rate, and token validity, updated after each sync cycle | Reporting | ALL | Must | Administrators need an at-a-glance view of connector fleet health |

### 9.4 Field Mapping & Data Normalization

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHCF-016 | Configurable Field Mapping | The system SHALL provide a field-mapping configuration UI for each connector, allowing administrators to map source-system fields (e.g., Jira `customfield_10001`) to ESIP Knowledge Hub artifact properties, with default mappings provided for standard fields and custom mapping for non-standard fields | Functional | ALL | Must | Enterprise instances have custom fields with business-critical data |
| REQ-KHCF-017 | Normalization of Common Fields | The system SHALL normalize common fields across all connectors to a unified artifact schema: `source_system`, `source_id`, `source_url`, `author`, `created_at`, `updated_at`, `title`, `body_text`, `labels/tags`, `status`, ensuring that cross-connector search operates on consistent field names | Data | ALL | Must | Unified schema enables cross-connector faceting and filtering |
| REQ-KHCF-018 | Source URL Canonical Resolution | The system SHALL store a canonical source URL (direct hyperlink to the source artifact in its native system) for every connector-synced artifact, displayed in the ESIP UI as a "View in [Source System]" link, enabling one-click context switching | UX | ALL | Must | Architects need to navigate from evidence to source for full context |
| REQ-KHCF-019 | Conflict Resolution Policy | The system SHALL apply a configurable conflict resolution policy when the same logical artifact is updated both in ESIP (e.g., manual annotation) and at the source during a sync cycle — options SHALL include: `Source Wins`, `ESIP Wins`, `Merge (source body + ESIP metadata)`, and `Flag for Manual Review` | Data | ALL | Must | Sync conflicts are inevitable in bidirectional or annotation-heavy workflows |

---

## 10. Jira Connector

> **Boundary note (reconciliation).** The Jira / Azure DevOps connectors in this volume (§10, §14) are **READ-ONLY evidence ingestion** — issues, comments, and attachments are pulled *into* the Knowledge Hub as evidence. They do **not** write back to the source. The three-way split is: **V08 = read-only evidence ingestion (this volume)**; **V11 = one-time ALM import** (initial migration of work items into ESIP requirements/records); **V19 = ongoing bidirectional sync** (status/field write-back, two-way reconciliation). Any write-back or continuous sync requirement belongs to V19, not here.

### 10.1 Jira Cloud & Server

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHJI-001 | Jira OAuth 2.0 (Cloud) | The system SHALL authenticate to Jira Cloud using OAuth 2.0 Authorization Code Flow with PKCE (Atlassian `read:jira-work`, `read:jira-user` scopes), storing tokens in the platform secrets vault with automatic refresh | Integration | ALL | Must | Jira Cloud REST API v3 OAuth 2.0 (3LO) |
| REQ-KHJI-002 | Jira PAT / API Token (Server/DC) | The system SHALL authenticate to Jira Server and Jira Data Center using Personal Access Token (Jira Server PAT) or API Token (Jira Cloud fallback) via HTTP Basic Auth header (`Authorization: Basic base64(email:token)`), stored in the secrets vault | Integration | ALL | Must | Jira Server does not support OAuth 2.0 PKCE |
| REQ-KHJI-003 | JQL-Based Scope Configuration | The system SHALL allow administrators to configure the sync scope using a JQL (Jira Query Language) expression, enabling fine-grained selection of issues to ingest (e.g., project = ARCH AND type in (Story, Epic, Bug, Task) AND updated >= -30d) | Functional | ALL | Must | Jira instances contain millions of issues; JQL scoping is essential to avoid ingesting irrelevant content |
| REQ-KHJI-004 | Issue Full Ingestion | The system SHALL ingest Jira issues as Knowledge Hub artifacts, capturing: key, summary, description (Jira Document Format → plain text + structured content), issue type, status, priority, assignee, reporter, created/updated dates, fix version, components, labels, custom fields per the field mapping configuration | Functional | ALL | Must | Full issue record is required for requirements extraction (V07) |
| REQ-KHJI-005 | Issue Comment Ingestion | The system SHALL ingest Jira issue comments as part of the parent issue artifact, preserving author, timestamp, and formatted body, with comments searchable independently but displayed as a timeline within the issue view | Functional | ALL | Must | Comments capture stakeholder decisions and clarifications on requirements |
| REQ-KHJI-006 | Issue Attachment Handling | The system SHALL detect Jira issue attachments, download their content (within configured size and type limits), store them as child document artifacts linked to the parent issue, and index their text content for search | Functional | ALL | Should | Architecture specifications are routinely attached to Jira epics |
| REQ-KHJI-007 | Epic / Story Hierarchy Preservation | The system SHALL preserve the Jira issue hierarchy (Epic → Story → Sub-task, and Epic Link / Parent Link relationships) in the Knowledge Hub artifact model, enabling filtering and navigation by hierarchy level | Functional | SA, AA | Must | Hierarchy is essential for requirements traceability from epic to story |
| REQ-KHJI-008 | Sprint and Board Context | The system SHALL capture sprint assignment, board name, and sprint dates for ingested issues, enabling temporal filtering of evidence by sprint/delivery cycle | Functional | AA, MA | Should | Sprint context aids architectural impact analysis by delivery increment |
| REQ-KHJI-009 | Jira Webhook for Real-Time Sync | The system SHALL register a Jira webhook (via Jira REST API `POST /rest/webhooks/1.0/webhook`) to receive real-time events for issue created, updated, commented, and deleted events within the configured scope, processing them within 30 seconds | Integration | ALL | Must | Jira is often the primary real-time requirements event source |
| REQ-KHJI-010 | Jira Incremental Sync via Changelog | The system SHALL implement incremental sync using Jira's `updated` field in JQL (e.g., `updated >= "-15m"`) for scheduled polls, processing only changed issues since the last successful sync timestamp | Integration | ALL | Must | Avoids full re-crawl of potentially millions of issues per sync cycle |
| REQ-KHJI-011 | Jira Issue Link Mapping | The system SHALL ingest Jira issue links (blocks, is-blocked-by, relates-to, duplicates, clones) and represent them as Knowledge Hub artifact relationships, enabling cross-issue dependency navigation and AI impact analysis | Functional | SA, AA | Should | Issue links represent architectural dependencies and constraints |

---

## 11. Confluence Connector

> **Boundary note (reconciliation).** The Confluence and SharePoint/OneDrive connectors in this volume (§11, §12) provide **INBOUND ingestion** only — pages, files, and metadata are pulled *into* the Knowledge Hub. **Outbound publishing** (writing ESIP deliverables/diagrams back out to a Confluence space or SharePoint library) is owned by **V19**. Any publish-to-source requirement belongs to V19, not here.

### 11.1 Confluence Cloud & Server

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHCO-001 | Confluence OAuth 2.0 (Cloud) | The system SHALL authenticate to Confluence Cloud using OAuth 2.0 Authorization Code Flow with PKCE (Atlassian `read:confluence-content.all`, `read:confluence-space.summary`, `read:confluence-user` scopes) | Integration | ALL | Must | Confluence Cloud REST API v2 uses same Atlassian OAuth as Jira |
| REQ-KHCO-002 | Confluence PAT (Server/DC) | The system SHALL authenticate to Confluence Server and Data Center using Personal Access Token via `Authorization: Bearer {token}` header, stored in the secrets vault | Integration | ALL | Must | Confluence Server PAT is the current non-deprecated auth method |
| REQ-KHCO-003 | Space / Page-Tree Scope Configuration | The system SHALL allow administrators to configure sync scope by selecting specific Confluence spaces, and optionally limiting to specific page trees (root page + all descendants) or label-based page sets, preventing ingestion of irrelevant spaces | Functional | ALL | Must | Enterprise Confluence instances contain thousands of spaces |
| REQ-KHCO-004 | Page Content Ingestion | The system SHALL ingest Confluence pages as document artifacts, converting Confluence Storage Format (XHTML) to clean plain text plus structured content (headings, tables, code blocks, embedded macros summary), capturing page title, space key, author, created/modified dates, labels, and version number | Functional | ALL | Must | Confluence is the primary enterprise architecture wiki and documentation platform |
| REQ-KHCO-005 | Page Version History | The system SHALL ingest Confluence page version history (up to a configurable depth, default: last 10 versions), storing previous versions as archived artifact versions with diff metadata, enabling architects to compare how a decision document evolved | Functional | EA, SA | Should | Architecture decision documents evolve significantly; version history matters |
| REQ-KHCO-006 | Blog Post Ingestion | The system SHALL ingest Confluence blog posts using the same pipeline as pages, treating them as document artifacts with a `Blog` sub-type classification | Functional | ALL | Should | Architecture teams publish decision summaries and RFC blogs |
| REQ-KHCO-007 | Embedded Image & Attachment Extraction | The system SHALL extract inline images from Confluence pages (stored as embedded binary in storage format) and file attachments, applying OCR to images and storing both as child artifacts with parent-page cross-links | Functional | ALL | Should | Architecture diagrams are routinely embedded in Confluence pages |
| REQ-KHCO-008 | Comment Ingestion | The system SHALL ingest page-level and inline Confluence comments as part of the parent page artifact, preserving author, timestamp, and text, with inline comments linked to the specific text they annotate | Functional | ALL | Should | Review comments on architecture documents capture stakeholder feedback |
| REQ-KHCO-009 | Confluence Change Notification (Webhook) | The system SHALL register Confluence webhooks for `page_created`, `page_updated`, `page_deleted`, and `comment_created` events to enable real-time incremental sync | Integration | ALL | Must | Confluence REST API webhook: `POST /rest/webhooks/1.0/webhook` |
| REQ-KHCO-010 | CQL-Based Sync Filtering | The system SHALL support CQL (Confluence Query Language) expressions for sync scope refinement (e.g., `space = "ARCH" AND label = "architecture-decision" AND lastModified >= "2024-01-01"`) | Functional | ALL | Should | CQL provides flexible, familiar filtering for Confluence administrators |

---

## 12. SharePoint / OneDrive Connector

> **Boundary note (reconciliation).** SharePoint/OneDrive here is **INBOUND ingestion** only (see §11 note). **Outbound publishing** of ESIP-generated deliverables/diagrams to a SharePoint library → **V19**.

### 12.1 SharePoint Online & OneDrive

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHSP-001 | SharePoint OAuth via Graph API | The system SHALL authenticate to SharePoint Online and OneDrive for Business using Microsoft Graph API OAuth 2.0 (delegated `Sites.Read.All`, `Files.Read.All` scopes or application permissions for background sync), using the same tenant app registration as the Graph Mail/Teams connector where possible | Integration | ALL | Must | Microsoft Graph is the unified API for M365 including SharePoint/OneDrive |
| REQ-KHSP-002 | Site and Library Scope Configuration | The system SHALL allow administrators to configure sync scope at SharePoint site collection, sub-site, document library, or folder level, with a recursive option to include all nested folders and a library allowlist/denylist | Functional | ALL | Must | SharePoint deployments span hundreds of site collections; selective sync is mandatory |
| REQ-KHSP-003 | File Ingestion with Metadata | The system SHALL ingest SharePoint/OneDrive files as document artifacts, extracting SharePoint column metadata (custom metadata columns configured on the library), version number, modification dates, and the sharing/permissions state (public/restricted/specific users) | Functional | ALL | Must | SharePoint column metadata contains business classification not in file content |
| REQ-KHSP-004 | SharePoint List Item Ingestion | The system SHALL ingest SharePoint List items (including custom lists used as lightweight requirements or decision registers) as structured data artifacts, mapping list columns to artifact properties per the connector field mapping | Functional | EA, SA | Should | SharePoint Lists are widely used as lightweight decision and risk registers |
| REQ-KHSP-005 | OneDrive Personal Folder Watch | The system SHALL support watching specific folders in a user's OneDrive for Business (with delegated access and user consent), ingesting files placed in those folders as personal evidence scoped to the user's workspace | Functional | ALL | Should | Architects frequently store working documents in personal OneDrive before sharing |
| REQ-KHSP-006 | SharePoint Delta Query Sync | The system SHALL implement incremental sync using Microsoft Graph delta queries (`GET /sites/{id}/drive/root/delta`) to detect file creates, updates, moves, and deletes since the last sync token, minimizing API calls | Integration | ALL | Must | Graph delta queries are the correct incremental sync mechanism for SharePoint/OneDrive |
| REQ-KHSP-007 | SharePoint Permission Inheritance | The system SHALL read and store SharePoint file permission metadata (inherited from library vs. uniquely set, share link type) and use this to enforce hub-level access control: artifacts from restricted SharePoint files SHALL only be visible in ESIP to users who also have access in SharePoint | Security | ALL | Must | SharePoint permissions represent data classification decisions that must be honored |

---

## 13. Microsoft Graph — Mail / Calendar / Teams

### 13.1 Graph API Integration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHGR-001 | Graph App Registration | The system SHALL use a single Microsoft Entra ID (Azure AD) app registration per ESIP tenant as the identity for all Microsoft connector interactions (SharePoint, Exchange/Mail, Teams, Calendar), supporting both delegated and application permission modes with admin consent for application permissions | Integration | ALL | Must | Single app registration minimizes IT admin overhead and simplifies permission management |
| REQ-KHGR-002 | Exchange Mail Sync via Graph | The system SHALL sync email via Microsoft Graph `GET /users/{id}/mailFolders/{id}/messages` (with `$filter` on `receivedDateTime` for incremental sync), capturing full message metadata and body for watched folders per REQ-KHEM-001 | Integration | ALL | Must | Graph mail endpoint with incremental timestamp filter |
| REQ-KHGR-003 | Calendar Event Ingestion | The system SHALL sync calendar events from configured user or shared calendars via Microsoft Graph `GET /users/{id}/events` (or `/groups/{id}/calendar/events` for shared/group calendars), ingesting event metadata (title, organizer, attendees, time, description, online meeting link) as meeting evidence artifacts | Functional | ALL | Should | Calendar events are the precursor to meeting transcripts; linking them provides meeting context |
| REQ-KHGR-004 | Teams Message Sync via Graph | The system SHALL sync Teams channel messages via Microsoft Graph `GET /teams/{teamId}/channels/{channelId}/messages` with delta query support for incremental sync per REQ-KHCH-001 | Integration | ALL | Must | Graph Teams messaging endpoint with delta token support |
| REQ-KHGR-005 | Graph Subscription (Webhook) Management | The system SHALL create, renew, and delete Microsoft Graph change notification subscriptions for all configured resources (mail, calendar, channels), handling subscription expiry (max 4320 min for mail; 60 min for messages) via automated renewal before expiry | Integration | ALL | Must | Graph subscriptions expire; renewal is mandatory for continuous real-time sync |
| REQ-KHGR-006 | Graph Throttling Compliance | The system SHALL handle Microsoft Graph throttling responses (HTTP 429 with `Retry-After`) for all Graph connectors using a shared Graph API request queue with per-resource throttling state, ensuring that throttling on one resource (e.g., mail) does not block processing of another (e.g., Teams) | NFR | ALL | Must | Graph throttling limits differ per API surface; a shared queue prevents cascading delays |
| REQ-KHGR-007 | Teams Meeting Transcript & Recording Pull | The system SHALL retrieve Teams meeting transcripts and recording metadata via Graph `GET /users/{userId}/onlineMeetings/{meetingId}/transcripts` and `/recordings`, linking them to the parent calendar event artifact | Integration | ALL | Must | Complements REQ-KHMT-002 with calendar event linkage |

---

## 14. Azure DevOps Connector

### 14.1 ADO Integration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHAD-001 | ADO PAT Authentication | The system SHALL authenticate to Azure DevOps Services and Azure DevOps Server using Personal Access Tokens (PAT) with minimum required scopes (`Work Items: Read`, `Code: Read` for wiki, `Analytics: Read`), stored in the platform secrets vault | Integration | ALL | Must | ADO supports OAuth 2.0 but PAT is more widely used for server integrations |
| REQ-KHAD-002 | ADO OAuth 2.0 (Cloud) | The system SHALL support OAuth 2.0 Authorization Code Flow for Azure DevOps Services using the Azure DevOps OAuth provider (`vso.work`, `vso.wiki`, `vso.code_read` scopes) as an alternative to PAT for user-delegated access | Integration | ALL | Should | OAuth preferred over PAT for user-context access |
| REQ-KHAD-003 | Work Item Ingestion (WIQL Scope) | The system SHALL ingest ADO work items (Epics, Features, User Stories, Tasks, Bugs, Test Cases) as issue artifacts using WIQL (Work Item Query Language) expressions for scope configuration, capturing all standard and custom fields per field mapping | Functional | ALL | Must | ADO is the primary ALM tool for Microsoft-stack engineering teams |
| REQ-KHAD-004 | ADO Wiki Page Ingestion | The system SHALL ingest Azure DevOps Wiki pages (both project wikis and code wikis) as document artifacts, converting wiki Markdown to structured text and capturing page hierarchy | Functional | ALL | Must | ADO Wikis contain architecture documentation and design notes |
| REQ-KHAD-005 | ADO Pull Request & Review Ingestion | The system SHALL ingest ADO Pull Requests as document artifacts including: title, description, reviewers, status, linked work items, and PR comment threads, enabling architecture review evidence capture | Functional | AA, MA | Should | PR descriptions and reviews capture architecture decision reasoning |
| REQ-KHAD-006 | ADO Service Hooks (Webhooks) | The system SHALL configure ADO Service Hooks for `workitem.created`, `workitem.updated`, `workitem.commented`, and `git.push` (for wiki changes) events to trigger real-time incremental sync | Integration | ALL | Must | ADO Service Hooks: `POST /{organization}/_apis/hooks/subscriptions` |
| REQ-KHAD-007 | ADO Incremental Sync via Rev/Watermark | The system SHALL implement incremental work item sync using ADO's revision watermark approach (tracking the maximum `rev` value seen per project) to detect only updated items since last sync | Integration | ALL | Must | ADO does not support `updatedAfter` query natively; revision watermark is the correct approach |

---

## 15. ServiceNow + CMDB Connector

### 15.1 ServiceNow Integration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHSN-001 | ServiceNow OAuth 2.0 | The system SHALL authenticate to ServiceNow using OAuth 2.0 Authorization Code Flow (ServiceNow OAuth provider, `useraccount` + `itil` scopes as minimum), with PAT/Basic Auth fallback for ServiceNow instances with OAuth disabled | Integration | ALL | Must | ServiceNow Table API supports OAuth 2.0 |
| REQ-KHSN-002 | Incident / Change / Problem Ingestion | The system SHALL ingest ServiceNow Incidents, Change Requests, Problems, and Service Requests as issue artifacts, capturing number, short description, description, state, priority, assignment group, category, CMDB CI references, and activity log (journal entries) | Functional | EA, SA, CA, SecA | Must | Architecture impact analysis requires ITSM record context |
| REQ-KHSN-003 | CMDB CI Ingestion | The system SHALL ingest ServiceNow CMDB Configuration Items (selected CI classes: `cmdb_ci_appl`, `cmdb_ci_service`, `cmdb_ci_server`, `cmdb_ci_database`, `cmdb_ci_network_adapter`, and others per configuration) as CMDB evidence artifacts, capturing CI attributes and relationship data (upstream/downstream CIs via `cmdb_rel_ci`) | Functional | EA, CA | Must | CMDB is the authoritative source for technology landscape facts in most enterprises |
| REQ-KHSN-004 | CMDB Relationship Graph | The system SHALL construct a CMDB CI relationship graph from ingested `cmdb_rel_ci` records and present it in the Knowledge Hub as a navigable topology view, enabling architects to understand application and infrastructure dependencies | Functional | EA, CA | Should | CMDB relationship data is the primary input to application portfolio impact analysis |
| REQ-KHSN-005 | ServiceNow Knowledge Article Ingestion | The system SHALL ingest ServiceNow Knowledge Base articles from configured knowledge bases as document artifacts, capturing number, short description, text body, workflow state, and category | Functional | ALL | Should | Knowledge Base articles document operational procedures and architecture constraints |
| REQ-KHSN-006 | ServiceNow Scoped Table Filtering | The system SHALL support configuration of ServiceNow ingestion scope using encoded queries (ServiceNow sysparm_query syntax) per table, allowing selective ingestion based on category, assignment group, date range, or any indexed field | Functional | ALL | Must | ServiceNow tables can contain millions of records; scope restriction is mandatory |
| REQ-KHSN-007 | ServiceNow Incremental Sync (sys_updated_on) | The system SHALL implement incremental sync using the `sys_updated_on` timestamp field available on all ServiceNow tables, fetching only records updated since the last successful sync watermark | Integration | ALL | Must | Standard ServiceNow incremental sync pattern |
| REQ-KHSN-008 | ServiceNow Outbound REST Message (Webhook) | The system SHALL support configuration of ServiceNow Outbound REST Messages and Business Rules to push record changes in real-time to the ESIP webhook receiver endpoint, enabling near-real-time ITSM event sync for high-priority CMDB and Change Management changes | Integration | EA, CA | Should | Push-based sync reduces polling overhead for change management events |

---

## 16. GitHub / GitLab / Bitbucket Connector (Document/Knowledge)

### 16.1 GitHub Connector

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHGH-001 | GitHub OAuth 2.0 App | The system SHALL authenticate to GitHub using a GitHub OAuth App (Authorization Code Flow, `repo:read`, `read:org` scopes) or GitHub App installation token for organization-wide access, stored in the platform secrets vault | Integration | ALL | Must | GitHub OAuth App or GitHub App are the correct auth mechanisms for org-level access |
| REQ-KHGH-002 | GitHub Wiki Ingestion | The system SHALL ingest GitHub repository wikis (cloned from the wiki Git repository) as document artifacts, parsing Markdown pages and preserving the page hierarchy and cross-links as artifact metadata | Functional | AA, MA | Must | GitHub wikis contain architecture decision records, API documentation, and design notes |
| REQ-KHGH-003 | README and Docs Folder Ingestion | The system SHALL ingest README files and files within configured documentation directories (e.g., `/docs`, `/architecture`, `/adr`) from configured repositories, treating each Markdown/AsciiDoc file as a document artifact | Functional | AA, MA | Must | README and docs folders are primary sources of architecture documentation in code repos |
| REQ-KHGH-004 | GitHub Issues & PR Ingestion | The system SHALL ingest GitHub Issues and Pull Requests as issue artifacts, capturing title, body, labels, milestone, assignees, comments, and linked PRs/issues, scoped by label, milestone, or date range configuration | Functional | AA, MA | Should | GitHub Issues capture architecture requirements and design decisions in open-source/inner-source contexts |
| REQ-KHGH-005 | GitHub Releases / Changelog Ingestion | The system SHALL ingest GitHub Releases (tag, title, release notes body, assets) and CHANGELOG files as document artifacts, providing architecture change history for portfolio analysis | Functional | EA, SA | Should | Release notes document what architecture decisions were implemented per version |
| REQ-KHGH-006 | GitHub Webhook for Real-Time Sync | The system SHALL configure GitHub organization or repository webhooks for `issues`, `issue_comment`, `pull_request`, `gollum` (wiki), and `push` (for docs folder) events to trigger real-time incremental sync | Integration | ALL | Should | GitHub webhooks provide immediate notification of documentation changes |

### 16.2 GitLab Connector

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHGL-001 | GitLab OAuth 2.0 | The system SHALL authenticate to GitLab.com and GitLab Self-Managed using OAuth 2.0 Authorization Code Flow (`read_api`, `read_repository` scopes) | Integration | ALL | Must | GitLab OAuth 2.0 is the current recommended auth method |
| REQ-KHGL-002 | GitLab Wiki & Docs Ingestion | The system SHALL ingest GitLab project and group wikis and documentation files from configured repositories using the GitLab REST API, applying the same processing pipeline as GitHub wiki/docs ingestion | Functional | AA, MA | Must | GitLab is the primary code hosting platform at many enterprises |
| REQ-KHGL-003 | GitLab Issues & MR Ingestion | The system SHALL ingest GitLab Issues and Merge Requests as issue artifacts using the same pipeline as GitHub, with GitLab-specific fields (weight, due date, epic linkage) captured as extended metadata | Functional | AA, MA | Should | GitLab is often used with issues for architecture requirements tracking |
| REQ-KHGL-004 | GitLab Webhook | The system SHALL configure GitLab project or group webhooks for issue, note (comment), wiki_page, and merge_request events | Integration | ALL | Should | GitLab system hooks: project/group-level webhook configuration |

### 16.3 Bitbucket Connector

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHBB-001 | Bitbucket OAuth 2.0 | The system SHALL authenticate to Bitbucket Cloud using OAuth 2.0 Authorization Code Flow (`repository:read`, `issue:read`, `wiki:read` scopes) and to Bitbucket Data Center using HTTP Access Tokens (Bearer auth) | Integration | ALL | Should | Bitbucket remains in use at large enterprise accounts |
| REQ-KHBB-002 | Bitbucket Wiki & Docs Ingestion | The system SHALL ingest Bitbucket repository wikis and documentation files, applying the same processing pipeline as GitHub/GitLab | Functional | AA, MA | Should | Bitbucket wiki/docs are document evidence, not code artifacts |
| REQ-KHBB-003 | Bitbucket Issue & PR Ingestion | The system SHALL ingest Bitbucket Issues and Pull Requests as issue artifacts using the Bitbucket Cloud REST API v2 | Functional | AA, MA | Could | Lower market share than GitHub/GitLab in net-new enterprise deployments |

---

## 17. Slack Connector

### 17.1 Slack Integration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHSL-001 | Slack OAuth 2.0 App | The system SHALL implement a Slack OAuth 2.0 app (user token flow with `channels:history`, `channels:read`, `groups:history`, `groups:read`, `files:read`, `users:read`, `team:read` scopes) to access Slack workspace content | Integration | ALL | Must | Slack OAuth 2.0 v2 is the current auth mechanism; Bot token insufficient for full history access |
| REQ-KHSL-002 | Slack User Identity Mapping | The system SHALL map Slack user IDs to ESIP workspace user profiles using Slack's `users.info` API method, resolving display names and email addresses for author attribution | Functional | ALL | Must | Slack user IDs are not human-readable without resolution |
| REQ-KHSL-003 | Slack Channel Metadata Sync | The system SHALL sync Slack channel metadata (channel name, purpose, topic, creation date, member count) and store it as context for all message artifacts ingested from that channel | Functional | ALL | Should | Channel context aids classification of message evidence |
| REQ-KHSL-004 | Slack Enterprise Grid Support | The system SHALL support Slack Enterprise Grid organizations via org-level OAuth tokens, enabling configuration of channels from multiple workspaces within a Grid organization | Integration | ALL | Should | Large enterprise Slack deployments use Enterprise Grid; org-level access is required |
| REQ-KHSL-005 | Slack Events API (Real-Time) | The system SHALL use the Slack Events API (socket mode or HTTP event endpoint) to receive real-time events for `message`, `message.channels`, `message.groups`, `file_shared`, and `reaction_added` events in watched channels | Integration | ALL | Must | Slack Events API provides real-time message delivery without polling |
| REQ-KHSL-006 | Slack Rate Limit Compliance | The system SHALL comply with Slack API Tier rate limits (Tier 1: 1 req/min; Tier 2: 20 req/min; Tier 3: 50 req/min; Tier 4: 100 req/min) for all methods, using per-tier request queues with burst control | NFR | ALL | Must | Slack rate limits differ by method tier; undifferentiated throttling causes unnecessary slowdown |

---

## 18. Google Workspace — Drive / Gmail / Meet

### 18.1 Google Drive Connector

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHGW-001 | Google Workspace OAuth 2.0 | The system SHALL authenticate to Google Workspace APIs using OAuth 2.0 Authorization Code Flow (Drive: `drive.readonly`; Gmail: `gmail.readonly`; Calendar/Meet: `calendar.readonly` scopes), using Google's OAuth 2.0 endpoints and storing tokens in the secrets vault | Integration | ALL | Must | Google Identity Platform OAuth 2.0 is the only supported auth method for Workspace |
| REQ-KHGW-002 | Google Drive File Ingestion | The system SHALL ingest files from configured Google Drive folders (including Team Drives / Shared Drives), converting Google Docs (`.gdoc`) to structured text, Google Sheets (`.gsheet`) to tabular text, and Google Slides (`.gslides`) to slide-structured text using the Drive Export API | Functional | ALL | Should | Google Workspace is used by a significant portion of enterprise customers alongside M365 |
| REQ-KHGW-003 | Google Drive Incremental Sync | The system SHALL implement incremental sync using Google Drive Changes API (`drive.changes.list` with page token) to detect file creates, updates, and deletions since the last sync checkpoint | Integration | ALL | Should | Drive Changes API is the correct incremental sync mechanism |
| REQ-KHGW-004 | Google Drive Push Notifications | The system SHALL use Google Drive push notifications (`files.watch` endpoint) to receive real-time updates for changes in watched folders, renewing the watch channel before its expiry (max 7 days) | Integration | ALL | Should | Push notifications are preferred over polling for latency |

### 18.2 Gmail Connector

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHGW-005 | Gmail Label Watch | The system SHALL allow users to designate Gmail labels as watched sources, ingesting all labeled emails as evidence artifacts using Gmail API `users.messages.list` with label filter | Integration | ALL | Should | Gmail is the primary email client for Google Workspace customers |
| REQ-KHGW-006 | Gmail Push via Pub/Sub | The system SHALL use Gmail push notifications via Google Cloud Pub/Sub (`users.watch` endpoint) for real-time ingestion of newly labeled or folder-received emails | Integration | ALL | Should | Gmail Cloud Pub/Sub push is more efficient than polling for real-time email capture |
| REQ-KHGW-007 | Gmail Thread Grouping | The system SHALL group Gmail messages by thread (using Gmail `threadId`) and present them as a single evidence unit, consistent with Microsoft Graph email thread handling | Functional | ALL | Should | Consistent thread-based email experience regardless of email provider |

### 18.3 Google Meet / Calendar

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHGW-008 | Google Calendar Event Ingestion | The system SHALL ingest Google Calendar events from configured user or shared calendars via Google Calendar API, capturing event metadata and Google Meet links, linking them to meeting transcript artifacts when available | Functional | ALL | Should | Calendar events provide meeting context for transcript artifacts |
| REQ-KHGW-009 | Google Meet Transcript Ingestion | The system SHALL ingest Google Meet transcripts via Google Workspace Admin SDK / Vault API (when Meet transcript capture is enabled by the Workspace admin), storing them as meeting evidence artifacts linked to the parent calendar event | Integration | ALL | Should | Google Meet transcript availability depends on Workspace tier and admin configuration |

---

## 19. Box / Dropbox Connector

### 19.1 Box Connector

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHBX-001 | Box OAuth 2.0 | The system SHALL authenticate to Box using OAuth 2.0 Authorization Code Flow (Box application with `root_readwrite` scope limited to configured folder scope) | Integration | ALL | Should | Box OAuth 2.0 is the standard auth; JWT app auth also supported for service accounts |
| REQ-KHBX-002 | Box Folder Ingestion | The system SHALL ingest files from configured Box folders (including Box for Business collaborative folders), extracting metadata attributes (Box custom metadata templates) alongside file content | Functional | ALL | Should | Box is used at regulated-industry enterprises (financial services, healthcare) for document management |
| REQ-KHBX-003 | Box Webhook for Real-Time Sync | The system SHALL configure Box webhooks for `FILE.UPLOADED`, `FILE.UPDATED`, and `FILE.DELETED` events on watched folders | Integration | ALL | Should | Box webhooks: `POST /webhooks` for V2 webhook subscription |
| REQ-KHBX-004 | Box Incremental Sync via Events API | The system SHALL implement Box incremental sync using the Box Events API (`GET /events?stream_type=changes&stream_position={position}`) to detect changes since last sync | Integration | ALL | Should | Box Events API supports continuous change stream position tracking |

### 19.2 Dropbox Connector

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHDB-001 | Dropbox OAuth 2.0 | The system SHALL authenticate to Dropbox Business using OAuth 2.0 Authorization Code Flow (`files.metadata.read`, `files.content.read` scopes) | Integration | ALL | Could | Dropbox Business has lower enterprise adoption than Box/SharePoint/GDrive |
| REQ-KHDB-002 | Dropbox Folder Ingestion & Webhook | The system SHALL ingest files from configured Dropbox folders and register Dropbox webhooks for real-time change notification | Integration | ALL | Could | Dropbox webhooks: `POST /2/files/list_folder/longpoll` or webhook endpoint |

---

## 20. Generic Webhook / REST Connector

### 20.1 Inbound Webhook Receiver

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHWH-001 | Webhook Endpoint Provisioning | The system SHALL provision unique, authenticated webhook receiver endpoints (HTTPS URLs with HMAC signature verification and optional Bearer token auth) per workspace and per connector instance, enabling any external system that supports outbound webhooks to push evidence to ESIP | Functional | ALL | Must | Generic webhook receiver provides extensibility beyond named connectors |
| REQ-KHWH-002 | Webhook Payload Schema Configuration | The system SHALL allow administrators to configure a JSONPath-based field mapping from the incoming webhook payload to the ESIP artifact schema (title, body, author, date, type, tags), with a payload preview and test-fire capability in the UI | Functional | ALL | Must | Payload formats vary widely across webhook sources; configurable mapping is essential |
| REQ-KHWH-003 | Webhook Signature Verification | The system SHALL verify incoming webhook payloads against configurable HMAC-SHA256 signatures (compatible with GitHub, GitLab, Jira, Stripe, and custom signature schemes) and reject payloads with invalid or missing signatures with an HTTP 401/403 response | Security | ALL | Must | Unsigned webhooks are an injection vector |
| REQ-KHWH-004 | Webhook Delivery Log | The system SHALL maintain a delivery log for each webhook endpoint recording: payload receipt timestamp, payload size, signature verification result, processing status, and extracted artifact ID, retaining logs for 30 days | Reporting | ALL | Must | Delivery logs are essential for debugging webhook integration issues |

### 20.2 Outbound REST Connector

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHWH-005 | Generic REST Connector | The system SHALL provide a generic REST connector enabling administrators to configure a polling-based integration with any JSON or XML REST API by specifying: base URL, authentication (Bearer, API Key, Basic, OAuth 2.0 Client Credentials), list endpoint with pagination params, item endpoint, field mapping, and incremental filter parameter | Functional | ALL | Should | Covers custom ITSM, in-house wikis, and any system not covered by named connectors |
| REQ-KHWH-006 | REST Connector Pagination Support | The system SHALL support configurable pagination modes in the generic REST connector: offset/limit (`?page=N&limit=100`), cursor-based (`?cursor={next_cursor}`), and Link header (`rel="next"` RFC 5988) | Functional | ALL | Should | Pagination is universal but non-standard across REST APIs |
| REQ-KHWH-007 | REST Connector Request Throttling | The system SHALL apply a configurable request rate (requests per second or requests per minute) to the generic REST connector to respect unknown source system rate limits | NFR | ALL | Should | Generic connector must not inadvertently DoS internal systems |

---

## 21. Inbound Email (IMAP / Forwarding)

### 21.1 IMAP Connector Details

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHIM-001 | IMAP IDLE Support | The system SHALL use IMAP IDLE command (RFC 2177) for real-time push-style notification of new messages when the IMAP server supports it, falling back to polling on the configured schedule when IDLE is not supported | Integration | ALL | Should | IMAP IDLE eliminates polling latency for mail servers that support it |
| REQ-KHIM-002 | S/MIME and PGP Decryption | The system SHALL support decryption of S/MIME and PGP-encrypted emails when the workspace has been configured with the appropriate decryption certificate/key (stored in the secrets vault), storing the decrypted body as the artifact content | Security | ALL | Could | Encrypted email is uncommon in practice but required by some regulated industries |
| REQ-KHIM-003 | Email Bounce Handling | The system SHALL detect MAILER-DAEMON and bounce emails delivered to the inbound address, discard them without creating evidence artifacts, and log the bounce event for operational monitoring | Functional | ALL | Should | Bounce emails pollute the evidence hub if ingested as artifacts |

### 21.2 Inbound Forwarding Address

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHIM-004 | Forwarding Address Routing | The system SHALL route inbound forwarded emails to the correct workspace/item scope based on the email address used (one address per workspace, one per item), and SHALL reply to the sender with a confirmation email containing a link to the created evidence artifact | Functional | ALL | Must | Feedback confirms successful evidence capture and provides the artifact link |
| REQ-KHIM-005 | Forwarding Address Allowlist | The system SHALL enforce a configurable sender allowlist on each inbound forwarding address, rejecting (or quarantining for review) emails from non-allowlisted senders to prevent spam injection into the evidence hub | Security | ALL | Must | Inbound addresses without sender restriction are a spam vector |

---

## 22. RSS / Web Page Connector

### 22.1 RSS / Atom Feed Connector

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHRSS-001 | RSS/Atom Feed Polling | The system SHALL poll configured RSS 2.0 and Atom 1.0 feed URLs on a configurable schedule (default: daily), ingesting new feed items as web page evidence artifacts with title, link, description, author, and published date extracted from the feed item elements | Functional | ALL | Should | Standards bodies (OMG, TOGAF, NIST), vendor blogs, and IT news publish architecture-relevant content via RSS |
| REQ-KHRSS-002 | Full-Page Fetch on Feed Item | The system SHALL optionally fetch the full HTML page content for each new RSS feed item and extract the article body text using a configurable CSS selector or readability algorithm, storing the full text alongside the feed excerpt | Functional | EA, SA | Should | RSS excerpt often truncates content; full-page fetch enables complete AI analysis |
| REQ-KHRSS-003 | Feed Change Detection (ETag/Last-Modified) | The system SHALL use HTTP `ETag` and `Last-Modified` headers for conditional RSS feed polling (`If-None-Match` / `If-Modified-Since`), avoiding re-processing feeds that have not changed since last poll | NFR | ALL | Should | Conditional HTTP reduces unnecessary traffic to RSS feed servers |

### 22.2 Web Page Watcher

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHRSS-004 | Web Page Watch & Snapshot | The system SHALL allow configuration of specific web page URLs for periodic monitoring (schedule: configurable, default: weekly); on each poll it SHALL fetch the page, extract text content via readability/CSS selector, and create a new artifact version if content has changed from the previous snapshot | Functional | EA, SA | Should | Standards pages (TOGAF.org, NIST, OMG) update periodically; change detection keeps evidence fresh |
| REQ-KHRSS-005 | Web Page Content Change Alert | The system SHALL detect significant content changes in monitored web pages (>5% change in body text by character count) and send a notification to the workspace with a diff summary and link to the new artifact version | Functional | EA, SA | Could | Page change alerts are valuable for regulatory and standards compliance tracking |
| REQ-KHRSS-006 | Robots.txt Compliance | The system SHALL respect `robots.txt` directives for all web crawling and page fetching operations, skipping URLs disallowed by the site's robots.txt and logging the exclusion | NFR | ALL | Must | Robots.txt compliance is a legal and ethical requirement |

---

## 23. Storage & Artifact Lifecycle

### 23.1 Artifact Storage

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KH-044 | Tiered Blob Storage | The system SHALL store artifact binaries (original files, recordings, images) in object storage with a two-tier lifecycle: hot tier (≤90 days since last access, SSD-backed) and warm/archive tier (>90 days, lower-cost storage), with automatic tier migration and transparent retrieval | Data | ALL | Must | Knowledge hubs accumulate significant binary storage; tiered storage controls cost |
| REQ-KH-045 | Full-Text Search Index | The system SHALL maintain a full-text search index (Elasticsearch or equivalent) over all extracted artifact text content, updated within 60 seconds of artifact ingest or update, supporting BM25 ranking and field-boosting | Data | ALL | Must | Full-text search is the primary navigation mechanism for large hubs |
| REQ-KH-046 | Vector Embedding Storage | The system SHALL store dense vector embeddings for all artifact chunks (generated by the AI engine per V06) in a vector database, with per-artifact chunk provenance (artifact ID, byte-range, chunk index) for citation tracing | Data | ALL | Must | Vector embeddings enable semantic search and RAG Q&A per V06 |
| REQ-KH-047 | Structured Metadata Store | The system SHALL store artifact metadata (all non-binary fields: ID, type, dates, tags, status, source info, permissions) in a relational or document database with appropriate indexing for high-performance filtered search | Data | ALL | Must | Structured metadata store enables faceted filtering at sub-100ms latency |
| REQ-KH-048 | Artifact Chunk Model | The system SHALL split large artifacts into chunks (maximum 512 tokens each, with 64-token overlap) during indexing, storing chunk boundaries with character offsets in the original text, enabling exact span citation when AI returns evidence references | Data | ALL | Must | Chunking is required for vector search and citation accuracy |

### 23.2 Versioning

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KH-049 | Immutable Artifact Versioning | The system SHALL create an immutable new version record for every update to an artifact's content (binary or text body), retaining all previous versions indefinitely unless an explicit retention policy or legal hold triggers archival or deletion | Data | ALL | Must | Architecture decisions may be challenged; original evidence must be preserved |
| REQ-KH-050 | Version Diff View | The system SHALL provide a text diff view (character-level diff with semantic paragraph-level context) between any two versions of a document artifact, enabling architects to understand how a source document evolved | UX | ALL | Should | Evidence evolution view is critical for architecture audit trails |
| REQ-KH-051 | Current Version Promotion | The system SHALL allow authorized users to designate any historical version as the "current" version (e.g., to revert after an erroneous sync), with the action logged in the artifact audit trail | Functional | ALL | Should | Connector sync errors can push bad content; rollback capability is essential |

### 23.3 Large File Handling

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KH-052 | Streaming Ingestion Pipeline | The system SHALL process artifact ingestion as a streaming pipeline (read → extract → chunk → index) without loading the full file into memory, supporting files up to 5 GB without OOM conditions | NFR | ALL | Must | Large architecture repositories (Visio export ZIPs, recording archives) can exceed 1 GB |
| REQ-KH-053 | Long Text Truncation with Summary | The system SHALL handle documents whose full text exceeds the AI context window by: (a) processing the document in sliding windows for full indexing, (b) generating a representative AI summary of the full document, and (c) storing the summary as the primary AI-query target while preserving full-text for keyword search | Functional | ALL | Must | Documents such as RFP responses, compliance frameworks, and architecture repositories can be 500+ pages |
| REQ-KH-054 | Media File Size Limits & Warnings | The system SHALL enforce configurable per-artifact and per-connector maximum file size limits, warning users when an uploaded or synced file approaches the limit and rejecting files that exceed it with a clear error message identifying the file and limit exceeded | NFR | ALL | Must | Prevents hub storage exhaustion from runaway media ingestion |

### 23.4 Retention, Archival & Legal Hold

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KH-055 | Retention Policy Engine | The system SHALL provide a retention policy engine allowing organization admins to define retention rules per artifact type, connector, classification, or tag: `Retain Indefinitely`, `Retain N Years`, `Delete After N Days Inactive`, `Archive After N Years` | Governance | ALL | Must | Retention compliance is a mandatory enterprise governance requirement (GDPR Art. 5, SOX, HIPAA) |
| REQ-KH-056 | Legal Hold | The system SHALL support legal hold designation on individual artifacts, collections, or connector sync scopes: held artifacts SHALL be immune to retention-policy deletion or archival and SHALL be flagged with a legal hold indicator in all views | Governance | ALL | Must | Legal hold is mandatory for organizations under litigation or regulatory investigation |
| REQ-KH-057 | Archival to Cold Storage | The system SHALL automatically move artifacts meeting archival criteria to a cold storage tier (write-once, read-rarely), maintaining searchable metadata records pointing to the archived binary, with a configurable retrieval SLA (e.g., restore within 4 hours) | Data | ALL | Should | Cold storage dramatically reduces retention costs for multi-year evidence archives |
| REQ-KH-058 | Data Residency Configuration | The system SHALL allow configuration of artifact storage regions per workspace (e.g., EU-only, US-only, AU-only), enforcing that all artifact binaries and metadata are stored and processed only within the configured region(s) | Governance | ALL | Must | GDPR and sovereign cloud requirements mandate data residency controls |
| REQ-KH-059 | Deletion Audit Trail | The system SHALL record a non-deletable audit event for every artifact deletion (manual or retention-policy-driven), capturing: artifact ID, title, deletion timestamp, deletion actor (user or policy name), and deletion reason | Governance | ALL | Must | Audit trails for deletions are required by information governance standards |

### 23.5 Source Freshness & Conflict Handling

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KH-060 | Freshness Score | The system SHALL compute a freshness score for each connector-synced artifact based on: time since last successful sync, source system `updated_at` vs. ESIP `synced_at`, and whether the source artifact has been deleted; displaying the score and staleness indicator in artifact detail views | Functional | ALL | Must | Stale evidence must be visually distinguished from current evidence |
| REQ-KH-061 | Staleness Alert Threshold | The system SHALL allow configuration of per-connector staleness alert thresholds (e.g., "alert if an artifact has not been synced in >24 hours for real-time connectors, >7 days for document connectors"), generating workspace notifications when breached | Functional | ALL | Should | Proactive staleness alerting prevents silent evidence gaps |
| REQ-KH-062 | Source Deletion Handling | The system SHALL detect when a source artifact has been deleted (via webhook event, absence during incremental sync, or 404 during fetch) and transition the ESIP artifact to `Source-Deleted` status, retaining the content per retention policy, notifying workspace members, and preventing the artifact from appearing in default search results | Functional | ALL | Must | Source deletion should not silently purge evidence; architects must be notified |
| REQ-KH-063 | Sync Conflict Resolution Log | The system SHALL log every sync conflict (artifact updated both in ESIP annotations and at source between sync cycles) to a per-workspace conflict log, accessible to workspace admins, showing the conflicting fields, both values, and the resolution outcome per the configured conflict policy (REQ-KHCF-019) | Data | ALL | Must | Conflict log provides an audit trail for data integrity decisions |

---

## 24. AI Over the Knowledge Hub

*Scope note: AI model/RAG internals (chunking strategies, embedding model selection, LLM provider routing, agentic orchestration) are specified in V06. Requirements here govern Knowledge Hub-facing AI capabilities — what architects experience from the hub side.*

### 24.1 Semantic & Keyword Search

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHAI-001 | Unified Hybrid Search | The system SHALL provide a single search interface combining BM25 keyword search and dense vector semantic search (cosine similarity), with configurable re-ranking (RRF or cross-encoder) to return the most relevant artifacts, ranked by a blended score | AI/ML | ALL | Must | Hybrid search consistently outperforms either method alone for architecture knowledge retrieval |
| REQ-KHAI-002 | Faceted Filter Search | The system SHALL support faceted filtering of search results by: artifact type, connector/source, date range, author, tag, architecture domain, framework relevance, and PII sensitivity flag, with facet counts updated dynamically as filters are applied | Functional | ALL | Must | Large hubs require faceted navigation to surface relevant results |
| REQ-KHAI-003 | Cross-Project / Cross-Workspace Search | The system SHALL provide an optional cross-workspace search mode that searches across all workspaces the user is authorized to access within the organization, clearly labeling results by workspace origin | Functional | EA | Should | Enterprise architects need to discover evidence from across multiple domain programs |
| REQ-KHAI-004 | Search Result Snippet with Highlights | The system SHALL return search results with highlighted matching text snippets (keyword matches highlighted in yellow, semantic match context in the surrounding paragraph), truncated to 300 characters with a "show more" expansion | UX | ALL | Must | Context snippets allow architects to assess relevance without opening each artifact |
| REQ-KHAI-005 | Search History & Saved Searches | The system SHALL persist each user's search history (last 50 queries per workspace) and allow saving of named searches with optional notification settings (alert when new artifacts match a saved search) | UX | ALL | Should | Architects run the same evidence searches repeatedly during active design phases |

### 24.2 Natural-Language Q&A with Citations

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHAI-006 | Evidence Q&A Chatbot | The system SHALL provide a natural-language Q&A interface over the Knowledge Hub, accepting free-text questions from architects and returning AI-generated answers grounded exclusively in retrieved artifact content (RAG pattern), with inline citations for every factual claim | AI/ML | ALL | Must | The Q&A chatbot is the primary AI interface for evidence-based architecture work |
| REQ-KHAI-007 | Citation Traceability to Span | The system SHALL make every citation in a Q&A answer clickable, opening the source artifact at the specific paragraph or sentence that the answer is grounded in, using stored chunk provenance (artifact ID + character offset) | AI/ML | ALL | Must | Span-level citation is the standard for enterprise-grade RAG; claim-level without span is insufficient |
| REQ-KHAI-008 | Scope-Bounded Q&A | The system SHALL allow the user to scope a Q&A query to: all hub artifacts, a specific folder/collection, a specific connector source, a specific tag, or a specific artifact type, with the scope displayed alongside the answer | Functional | ALL | Must | Scoping prevents answers that mix evidence from unrelated project areas |
| REQ-KHAI-009 | Confidence & Grounding Score | The system SHALL display a grounding score (0–100%) with each Q&A answer, reflecting the degree to which the answer is supported by retrieved evidence vs. LLM parametric knowledge, and a warning when the score is below a configurable threshold (default: 60%) | AI/ML | ALL | Must | Low-grounding answers may contain hallucinations; architects must be warned |
| REQ-KHAI-010 | Multi-Turn Q&A Context | The system SHALL maintain conversational context across up to 20 turns in a Q&A session, allowing architects to ask follow-up questions that reference previous answers ("What else does that document say about...") without repeating full context | AI/ML | ALL | Should | Multi-turn dialogue is essential for deep evidence investigation |
| REQ-KHAI-011 | Q&A Answer Export | The system SHALL allow architects to export a Q&A session (questions, answers, and citations) as a Markdown or PDF report, suitable for inclusion in architecture documentation or review packages | UX | ALL | Should | Q&A sessions represent research work product that needs to be preserved and shared |
| REQ-KHAI-012 | Conflicting Evidence Detection | The system SHALL detect when retrieved artifacts contain contradictory information relevant to a Q&A query (e.g., different source documents stating conflicting NFR values) and surface the contradiction in the answer with references to both conflicting sources | AI/ML | EA, SA | Should | Contradictory evidence is a critical signal for architecture decision-making |

### 24.3 Summarization

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHAI-013 | Single Artifact Summarization | The system SHALL provide an AI-generated summary for every artifact in the Knowledge Hub, generated on ingest (or on demand for pre-existing artifacts), covering: main topic, key decisions/conclusions, and action items if present | AI/ML | ALL | Must | Artifact summaries enable rapid triage of large evidence sets |
| REQ-KHAI-014 | Collection Summarization | The system SHALL allow architects to request an AI summary of a folder, collection, or tag-filtered artifact set, generating a synthesized narrative covering the main themes, convergences, and divergences across all included artifacts | AI/ML | ALL | Should | Collection summaries enable rapid onboarding to a project's evidence base |
| REQ-KHAI-015 | Meeting Summary with Agenda Sections | The system SHALL generate structured meeting summaries for meeting transcript artifacts, organized by agenda section (detected from transcript structure) with: discussion points, decisions made, action items (with owner and due date if mentioned), and open questions | AI/ML | ALL | Must | Structured meeting summaries are the highest-value immediate output from transcript ingestion |
| REQ-KHAI-016 | Progressive Summarization | The system SHALL support progressive summarization at three levels (Brief: 3 sentences; Standard: 3 paragraphs; Detailed: full section-by-section) selectable per artifact view and per collection summary request | UX | ALL | Should | Different architect roles and contexts require different levels of summary depth |

### 24.4 Auto-Tagging & Classification

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHAI-017 | Sensitivity Classification | The system SHALL automatically classify each artifact into a sensitivity tier (`Public`, `Internal`, `Confidential`, `Restricted`) based on content analysis (detecting classification markings, PII presence, business-sensitive keywords), presenting the classification as a badge and using it to enforce access control | AI/ML | ALL | Must | Sensitivity classification is a prerequisite for enterprise information governance |
| REQ-KHAI-018 | Content Type Classification | The system SHALL automatically classify artifacts by content type beyond file format: `Requirements`, `Decision Record`, `Meeting Notes`, `Design Document`, `Specification`, `Policy`, `Research`, `Correspondence`, enabling content-type-scoped searches | AI/ML | ALL | Must | Content type classification enables domain-specific evidence retrieval patterns |
| REQ-KHAI-019 | Architecture Phase Tagging | The system SHALL tag artifacts with the TOGAF ADM phase(s) most relevant to their content (Preliminary, A–H, Requirements Management), enabling EA-workflow-scoped evidence filtering | AI/ML | EA | Should | ADM-phase tagging enables architects to filter evidence by current ADM phase |

### 24.5 Entity & Decision Extraction

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHAI-020 | Decision Extraction | The system SHALL extract explicit and implicit architecture decisions from artifact content (meeting transcripts, email threads, design documents), presenting them as structured decision records with: decision text, context, date, participants, and source artifact citation — supplementing the manual ADR process | AI/ML | EA, SA | Must | Decisions buried in meeting transcripts and emails are routinely missed; automated extraction recovers them |
| REQ-KHAI-021 | Action Item Extraction | The system SHALL extract action items from meeting transcripts, email threads, and chat messages, presenting them as structured records with: action text, assignee (resolved to workspace user if possible), due date (if mentioned), and source artifact citation | AI/ML | ALL | Must | Action items from architecture reviews drive follow-up work |
| REQ-KHAI-022 | Requirement Signal Extraction | The system SHALL extract explicit and implicit requirement signals (SHALL, MUST, SHOULD, WILL, NEED TO, "it is required that") from artifact content, flagging them as candidate requirements for review in the Requirements Pillar (V07) — with source citation and confidence score | AI/ML | ALL | Must | Requirement signals in evidence feed V07 extraction pipeline |
| REQ-KHAI-023 | Named Entity Recognition | The system SHALL extract named entities from artifact content: organization names, person names, application/system names, technology names, vendor names, and acronyms, building a per-workspace entity glossary used for enhanced search and relationship discovery | AI/ML | ALL | Should | NER-derived entity glossary improves search recall and cross-artifact navigation |
| REQ-KHAI-024 | Risk & Concern Extraction | The system SHALL extract explicit and implicit risk signals (risk, concern, issue, blocker, problem, constraint, dependency, assumption) from artifact content, presenting them as a structured risk/assumption/issue/dependency (RAID) evidence list with source citations | AI/ML | SA, EA | Should | RAID register population from evidence reduces manual risk tracking effort |

### 24.6 Related Source Discovery

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHAI-025 | Semantic Related-Article Discovery | The system SHALL automatically compute semantic similarity between all artifacts in the workspace hub (using vector cosine similarity) and surface the top-5 semantically related artifacts in the artifact detail view's "Related Evidence" panel | AI/ML | ALL | Must | Cross-reference discovery reduces the risk of architects working with incomplete evidence |
| REQ-KHAI-026 | Contradiction & Consistency Detection | The system SHALL run a periodic consistency analysis across the workspace hub, identifying pairs of artifacts that contain factually contradictory statements (e.g., different throughput requirements, conflicting technology choices) and surfacing them in a hub-level consistency report | AI/ML | EA, SA | Should | Contradictory evidence sets are a leading indicator of misaligned stakeholder understanding |
| REQ-KHAI-027 | Evidence Coverage Gap Detection | The system SHALL analyze the workspace hub against the configured architecture framework phase checklist (e.g., TOGAF ADM Phase A inputs) and flag evidence gaps — phases or viewpoints with no supporting artifacts — enabling targeted evidence gathering | AI/ML | EA | Should | Gap detection prevents architectures built on incomplete evidence |

---

## 25. Provenance & Governance

### 25.1 Source Attribution

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHPG-001 | Immutable Provenance Record | The system SHALL create an immutable provenance record for every artifact at ingest time, capturing: source system, source ID, source URL, ingestion timestamp, ingestion method (connector name + version, manual upload, inbound email, webhook), and the user or service account identity responsible for ingestion | Governance | ALL | Must | Provenance is the foundation of evidence trustworthiness |
| REQ-KHPG-002 | Span-Level Citation | The system SHALL track and expose chunk-level provenance (artifact ID, version, character start/end offset) for all AI-generated outputs (Q&A answers, summaries, extractions) so that every AI-generated claim can be traced to the specific text span in the source artifact | Governance | ALL | Must | Span-level citation enables evidence verification and satisfies audit requirements |
| REQ-KHPG-003 | Derivation Chain Tracking | The system SHALL record the derivation chain when one artifact is generated from another (e.g., a requirement extracted from a document, a summary generated from a transcript), maintaining a derivation graph navigable from any derived artifact back to its source evidence | Governance | ALL | Must | Requirements traceability (V07) depends on derivation chain from evidence to requirement |
| REQ-KHPG-004 | Source System Change Audit | The system SHALL record an audit event whenever a connector-synced artifact is updated, capturing: previous version reference, new version reference, fields changed, sync timestamp, and the connector identity that triggered the update | Governance | ALL | Must | Change audit supports compliance reviews and evidence integrity checks |

### 25.2 Access Control on Sources

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHPG-005 | Source Permission Inheritance | The system SHALL inherit access permissions from source systems for connector-synced artifacts where the source system exposes permission metadata (SharePoint, GitHub, Confluence space permissions), applying them as the default ESIP access policy for each artifact | Security | ALL | Must | Users should not gain access to content in ESIP that they cannot access in the source system |
| REQ-KHPG-006 | ESIP-Layer Access Override | The system SHALL allow workspace administrators to apply additional ESIP-layer access restrictions on artifacts (further restricting beyond source permissions), but SHALL NOT allow ESIP-layer policies to grant access to users who lack source-system access | Security | ALL | Must | ESIP can only narrow, never expand, source-system access rights |
| REQ-KHPG-007 | Per-Artifact Access Control | The system SHALL support per-artifact access control lists (ACLs) specifying individual users or ESIP roles permitted to view, annotate, or manage the artifact, enforced in all access paths: UI, API, search, Q&A, and export | Security | ALL | Must | Fine-grained per-artifact control is required for mixed-classification hubs |
| REQ-KHPG-008 | Derived Artifact Permission Propagation | The system SHALL propagate the most restrictive access policy of all source artifacts to any derived artifact (summaries, extractions, Q&A answer sessions that cite multiple sources), ensuring that derived outputs are never more permissive than any of their source inputs | Security | ALL | Must | A summary of a restricted document must inherit that document's access restriction |
| REQ-KHPG-009 | Row-Level Security in Search | The system SHALL apply access control filters at the search-index query layer (pre-query, not post-filter), ensuring that unauthorized artifacts never appear in search results, counts, or facets for users who lack access | Security | ALL | Must | Post-filtering leaks metadata (counts, facets) of restricted artifacts; pre-query filtering is the correct pattern |

### 25.3 PII & Sensitivity

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHPG-010 | Automated PII Detection | The system SHALL apply PII detection (using pattern matching + NER models) to all ingested artifact text content, identifying: email addresses, phone numbers, national ID/passport numbers, credit card numbers (PAN), IP addresses, physical addresses, and personal names in non-byline positions | Security | ALL | Must | GDPR Art. 5 (data minimization), CCPA, and HIPAA require PII awareness |
| REQ-KHPG-011 | PII Tagging & Redaction Modes | The system SHALL support three configurable PII handling modes per connector or artifact type: `Tag Only` (identify and flag but store plaintext), `Mask in UI` (display masked in UI, store plaintext for authorized roles), and `Redact on Ingest` (replace PII spans with `[REDACTED-{type}]` before storage) | Security | ALL | Must | Different regulatory contexts require different PII handling strictness |
| REQ-KHPG-012 | Sensitive Data Classifier | The system SHALL detect and classify artifact content against configurable sensitive data patterns beyond PII: financial figures, pricing, M&A-related terminology, personally identifiable HR data (salary, performance), legal-privileged content indicators, and export-controlled technology markers | Security | EA, SecA | Should | Enterprise architecture content frequently contains commercially sensitive information |
| REQ-KHPG-013 | PII Subject Access & Right-to-Erasure | The system SHALL support a data subject access request (DSAR) workflow: on request, identify all artifacts containing text matches for a specified personal data subject (by name/email), generate a report of located instances, and support bulk redaction or deletion of the identified spans | Governance | ALL | Must | GDPR Art. 15 (access) and Art. 17 (erasure) are legal requirements for EU data subjects |
| REQ-KHPG-014 | Consent & Acceptable-Use Policy | The system SHALL display and require acceptance of an acceptable-use policy for accessing Knowledge Hub content involving meeting transcripts of identified individuals (attendee-consent notice), tracking acceptance per user per workspace | Governance | ALL | Should | Meeting transcript ingestion involves third-party attendee data; consent notice is a GDPR transparency requirement |

### 25.4 Access Audit

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHPG-015 | Artifact Access Audit Log | The system SHALL record an immutable audit event for every artifact access (view, download, search result impression, Q&A citation retrieval), capturing: actor identity, artifact ID, access type, timestamp, and access context (search query, Q&A session ID, direct navigation) | Governance | ALL | Must | Access audit is a mandatory information governance requirement |
| REQ-KHPG-016 | Bulk Export Audit | The system SHALL detect and record bulk evidence export operations (≥10 artifacts in a single export, or a collection export exceeding 50 MB), triggering an elevated audit event and optionally requiring additional administrator approval for large exports | Security | ALL | Should | Bulk exports represent a data exfiltration risk requiring enhanced monitoring |
| REQ-KHPG-017 | Audit Log Export & SIEM Integration | The system SHALL provide an audit log export API and support streaming audit events to SIEM systems (Splunk, Microsoft Sentinel, Sumo Logic) via configurable webhook or syslog forwarding, with events formatted in CEF or JSON | Governance | ALL | Must | SIEM integration is required for enterprise security operations compliance |

---

## 26. Operational & UX Capabilities

### 26.1 Ingestion Dashboard

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHOP-001 | Ingestion Status Dashboard | The system SHALL provide a workspace-level ingestion dashboard showing: total artifacts by connector and type, ingest rate over time (charts: last 24h, 7 days, 30 days), pending queue depth, failed ingestions count, and estimated time to complete any running bulk sync | UX | ALL | Must | Administrators need real-time visibility into hub population progress |
| REQ-KHOP-002 | Per-Artifact Ingestion Status | The system SHALL display the ingestion processing status on each artifact: `Queued`, `Text-Extracting`, `OCR-Processing`, `Embedding`, `Indexed`, `Error` (with error detail), `Stale`, so users understand why a recently uploaded file may not yet be searchable | UX | ALL | Must | Architects frequently ask "why can't I find the file I just uploaded?" |
| REQ-KHOP-003 | Ingestion Error Report | The system SHALL generate a downloadable ingestion error report listing all artifacts that failed to index in a selected time window, with error type, source, file name, and recommended remediation action | Reporting | ALL | Must | Systematic ingestion errors indicate format or permission issues that need bulk remediation |

### 26.2 Connector Health & Monitoring

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHOP-004 | Connector Health Panel | The system SHALL provide a connector management panel showing all configured connectors with: status (`Active`, `Paused`, `Error`, `Expired Token`), last sync timestamp, next scheduled sync, items synced (total and this cycle), error count, and health score (REQ-KHCF-015) | UX | ALL | Must | Connector health visibility is essential for administrators managing enterprise connector fleets |
| REQ-KHOP-005 | Connector Sync Log | The system SHALL maintain a per-connector sync log with entries for each sync cycle: start time, end time, items processed, items created/updated/deleted, errors encountered, and API calls made (for rate-limit budget monitoring) | Reporting | ALL | Must | Sync logs are the primary diagnostic tool for connector issues |
| REQ-KHOP-006 | Token Expiry Alerts | The system SHALL send notifications (in-app + email) to workspace administrators 7 days, 3 days, and 1 day before an OAuth connector token is predicted to expire (based on refresh token expiry), prompting re-authorization before sync interruption | UX | ALL | Must | Token expiry notifications prevent silent sync failures |
| REQ-KHOP-007 | Connector Configuration Change Audit | The system SHALL record an audit event for every connector configuration change (scope change, credential change, schedule change, enable/disable), capturing the actor, previous configuration, and new configuration | Governance | ALL | Must | Connector scope changes can silently add or remove evidence; audit is required |

### 26.3 Notifications on New Evidence

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHOP-008 | New Evidence Notification | The system SHALL support per-user notification subscriptions on: new artifacts matching a saved search, new artifacts added to a specific folder/collection, new artifacts from a specific connector, and new artifacts tagged with a specific tag — delivered via in-app notification, email, or Teams/Slack message | UX | ALL | Must | Proactive evidence notification keeps architects informed without polling the hub |
| REQ-KHOP-009 | Evidence Digest | The system SHALL generate a configurable periodic evidence digest (daily or weekly) per workspace, summarizing: new artifacts ingested, connector sync issues, artifacts approaching expiry/staleness, and AI-detected decisions and action items from new content — delivered to workspace members | UX | ALL | Should | Digest format is more manageable than per-artifact notifications for high-volume hubs |
| REQ-KHOP-010 | Anomaly Alert | The system SHALL detect and alert on evidence anomalies: sudden drop in ingestion rate from an active connector (possible authentication failure), bulk deletion of artifacts (possible misconfigured retention policy), or spike in PII detection events (possible sensitive data ingestion) | Functional | ALL | Should | Anomaly alerts enable proactive governance intervention |

### 26.4 Deduplication Across Connectors

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHOP-011 | Cross-Connector Duplicate Detection | The system SHALL detect when the same document is ingested via multiple connectors (e.g., a document shared via both SharePoint and email attachment) using content hash comparison, presenting a duplicate candidate for user-directed merge | Functional | ALL | Must | Multi-connector deployments inevitably create duplicates without cross-connector dedup |
| REQ-KHOP-012 | Duplicate Merge Workflow | The system SHALL provide a merge workflow for confirmed duplicate artifacts: selecting a canonical artifact, transferring all annotations, tags, and derivation references from the duplicates to the canonical, and replacing duplicate references throughout the hub with the canonical artifact ID | Functional | ALL | Must | Merge must be reference-complete; dangling references to merged artifacts would break citations |
| REQ-KHOP-013 | Deduplication Report | The system SHALL generate a workspace deduplication report (scheduled or on-demand) listing all detected duplicate candidate pairs with similarity scores, source systems, and recommended merge actions, enabling bulk resolution | Reporting | ALL | Should | Manual review of duplicates at scale requires a structured report |

### 26.5 Bulk Operations

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale/Source |
|---|---|---|---|---|---|---|
| REQ-KHOP-014 | Bulk Select and Action | The system SHALL support multi-select of artifacts (by checkbox, Shift-click range, or "select all in current filter") with bulk actions: add tags, remove tags, move to folder, set retention policy, apply sensitivity classification, delete, export | UX | ALL | Must | Individual operations on hundreds of artifacts is unusable at enterprise scale |
| REQ-KHOP-015 | Bulk Export Package | The system SHALL allow bulk export of selected artifacts as a ZIP archive containing original files, a metadata manifest (JSON), and an AI-generated evidence package summary document, with export size and count limits enforced per the access audit requirements (REQ-KHPG-016) | Functional | ALL | Should | Bulk export enables evidence package preparation for external reviews and audits |
| REQ-KHOP-016 | Bulk Connector Scope Edit | The system SHALL allow administrators to edit connector scope configuration (e.g., add or remove watched folders, update JQL filter) in bulk across multiple connector instances of the same type via a batch configuration UI | UX | ALL | Could | Multi-project deployments with many connector instances benefit from bulk scope management |

---

## Appendix A — Boundary Notes & Assumptions

| Boundary | Decision |
|---|---|
| AI model/RAG internals | Embedding model selection, LLM provider routing, chunking strategies, agentic orchestration → V06 |
| Extraction of evidence into requirements | Requirement signal extraction AI capability is in scope here (REQ-KHAI-022 flags candidates); the full extraction pipeline with conflict detection, classification, and RTM is → V07 |
| Identity, SSO, secrets vault | Platform-level auth infrastructure → V11; this volume depends on secrets vault existing per V11 |
| GitHub/GitLab/Bitbucket — code analysis | Code scanning for APM, tech-debt, dependency extraction → V09; this volume covers only document/knowledge artifacts (wikis, README, issues, PRs) |
| CMDB as portfolio source | CMDB CI data for technology portfolio/APM analysis → V09; this volume covers CMDB as a Knowledge Hub evidence source |
| Diagramming engine | ArchiMate/Visio file rendering → V03; this volume handles Visio/ArchiMate files as indexed document artifacts only |
| Deliverable generation | Publishing Knowledge Hub evidence to Word/PPT reports → V10 |
| Multi-tenancy, RBAC model definition | RBAC roles and permission model definition → V11; this volume references them as "authorized user" and "workspace admin" |

---

## Appendix B — Connector Coverage Summary

| Connector | Auth Method | Real-Time Sync | Incremental Sync | Priority |
|---|---|---|---|---|
| Jira Cloud | OAuth 2.0 PKCE | Webhook | JQL `updated` filter | Must |
| Jira Server/DC | PAT / API Token | Webhook | JQL `updated` filter | Must |
| Confluence Cloud | OAuth 2.0 PKCE | Webhook | CQL `lastModified` | Must |
| Confluence Server/DC | PAT | Webhook | CQL `lastModified` | Must |
| SharePoint Online / OneDrive | Graph OAuth 2.0 | Graph Change Notifications | Graph delta query | Must |
| Microsoft Graph (Mail/Calendar/Teams) | Graph OAuth 2.0 | Graph Subscriptions | Timestamp filter / delta | Must |
| Azure DevOps | PAT / OAuth 2.0 | Service Hooks | Revision watermark | Must |
| ServiceNow + CMDB | OAuth 2.0 / Basic | Outbound REST Message | `sys_updated_on` | Must |
| GitHub | OAuth App / GitHub App | Webhooks | Events API / timestamp | Must |
| GitLab | OAuth 2.0 | Webhooks | Timestamp filter | Must |
| Bitbucket | OAuth 2.0 / HTTP Access Token | Webhooks | Timestamp filter | Should |
| Slack | OAuth 2.0 v2 | Events API | Conversations History | Must |
| Google Drive | OAuth 2.0 | Drive Push Notifications | Changes API delta | Should |
| Gmail | OAuth 2.0 | Cloud Pub/Sub | Timestamp filter | Should |
| Google Meet / Calendar | OAuth 2.0 | Calendar API push | Timestamp filter | Should |
| Box | OAuth 2.0 | Webhooks | Events API position | Should |
| Dropbox Business | OAuth 2.0 | Webhooks | `list_folder/longpoll` | Could |
| Generic Webhook (inbound) | HMAC-SHA256 / Bearer | Push (event-driven) | N/A | Must |
| Generic REST (outbound) | Configurable | N/A | Configurable timestamp | Should |
| IMAP | Credentials / IMAP IDLE | IMAP IDLE | Timestamp/UID watermark | Should |
| Inbound Email Forwarding | Sender allowlist | Push (SMTP delivery) | N/A | Must |
| RSS / Atom Feeds | None / API Key | N/A | ETag/Last-Modified | Should |
| Web Page Watcher | None / Basic Auth | N/A | Content hash change | Should |

---

*End of Volume 08 — Knowledge Hub & Enterprise Connectors*
