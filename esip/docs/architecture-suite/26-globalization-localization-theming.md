# Volume 26 — Globalization, Localization & Theming

**ESIP Architecture Automation Suite**
**Volume:** 26
**Prefix-A:** `REQ-I18N-` — Internationalization, Localization & Globalization
**Prefix-B:** `REQ-THM-` — Theming & White-Label
**Status:** Authoritative
**Cross-references:** V03 Diagramming · V06 AI Capabilities · V08 Knowledge Hub · V10 Deliverables · V11 Platform · V13 Onboarding/Billing · V14 Personalization · V19 Enterprise Adoption · V21 AI Prompt-Packs · V23 Templates

---

## Scope

This volume owns all requirements for making ESIP fully internationalizable, localizable, and globalizable at the UI, AI, and content layers, and for providing a comprehensive theming and white-label system. It does **not** own:

| Concern | Owned by |
|---|---|
| Brand governance inside generated deliverables (slide/doc templates) | V23 |
| Per-user preference *storage* (locale, theme choices) | V14 |
| File rendering engines (PDF/DOCX/PPTX output) | V10 |
| AI translation model infra, multilingual model selection | V06 |
| Multi-tenancy config storage, tenant provisioning | V11 |
| Multi-currency billing and invoicing | V13 |
| White-label domain/branding at the enterprise tier | V19 |
| Multilingual ingestion and semantic search | V08 |

---

## Area I — Internationalization (UI)

### UI String Externalization

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-I18N-001 | Zero Hard-Coded UI Strings | The application SHALL contain no hard-coded user-visible strings anywhere in the React component tree, AI-generated UI snippets, error messages, toast notifications, modal text, ARIA labels, or placeholder text. All strings SHALL be referenced exclusively through a locale key. | Functional | All | Must | Prevents per-locale code forks; enables runtime locale switching without rebuild. |
| REQ-I18N-002 | ICU MessageFormat Compliance | All localizable strings SHALL be expressed using ICU MessageFormat syntax (select, plural, ordinal, number, date, time, duration skeletons). The runtime SHALL use an ICU-compliant parser (e.g., `@formatjs/intl`, `intl-messageformat`). | Functional | All | Must | ICU is the CLDR-backed industry standard; handles plural rules for all 200+ CLDR locales. |
| REQ-I18N-003 | Plural & Gender Agreement | String catalog entries SHALL support plural categories (zero/one/two/few/many/other) per CLDR plural rules for each target locale, and grammatical gender agreement (select male/female/neuter) where required by the target language. | Functional | All | Must | Without CLDR plural rules, English-style "1 item / N items" breaks for e.g. Russian (4 forms), Arabic (6 forms), Polish (4 forms). |
| REQ-I18N-004 | Ordinal Number Formatting | The system SHALL format ordinal numbers (1st, 2nd, 1er, 1ère, …) using ICU `selectordinal` rules per locale rather than concatenating English suffixes. | Functional | All | Should | Used in roadmap step labels, phase numbering, ranked lists. |
| REQ-I18N-005 | String Catalog Format | String catalogs SHALL be stored as structured JSON or YAML per locale (e.g., `en-US.json`, `ar-SA.json`) under a versioned namespace hierarchy, importable at build time and patchable at runtime via a locale bundle endpoint. | Functional | Tenant Admin | Must | Structured catalogs allow partial bundle loading, CI linting, and translation tooling integration. |
| REQ-I18N-006 | Missing-Key Fallback Chain | When a key is absent in the active locale, the system SHALL fall back in order: (1) tenant-supplied override, (2) locale region variant (e.g., `fr-CA` → `fr`), (3) `en-US` baseline. Missing keys SHALL never render raw key identifiers to end users in production. | Functional | All | Must | Prevents broken UI during incremental translation rollouts. |
| REQ-I18N-007 | CI String Lint | A CI step SHALL fail if any new component commit introduces a hard-coded user-visible string or references an undefined locale key. | Governance | DevOps | Must | Enforces zero-hard-coded-string invariant continuously. |

### Runtime Locale Switching

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-I18N-010 | Per-User Locale Preference | The system SHALL respect a locale setting stored in the user profile (see V14) and apply it at session start without page reload where technically feasible. | Functional | All | Must | Multilingual teams share one ESIP tenant; each user works in their own language. |
| REQ-I18N-011 | Per-Tenant Default Locale | Tenant administrators SHALL be able to configure a default locale for their organization. New users who have not set a personal locale SHALL inherit the tenant default. | Functional | Tenant Admin | Must | Ensures newly onboarded users in a French-speaking org see French immediately. |
| REQ-I18N-012 | Browser Language Detection | On first visit with no stored preference, the system SHALL detect the user's browser `Accept-Language` header, match it to the nearest available locale, and apply it. The user SHALL be able to confirm or override via a locale selector. | Functional | All | Should | Reduces friction for new users who have not yet configured a preference. |
| REQ-I18N-013 | In-Session Locale Switch | Authenticated users SHALL be able to switch locale mid-session from a persistent locale selector (header or user menu) without losing unsaved work; the UI SHALL re-render in the new locale within 500 ms after bundle load. | UX | All | Should | Power users who manage multilingual content need to verify both languages in one session. |
| REQ-I18N-014 | Locale Bundle Lazy Loading | Locale bundles SHALL be code-split and loaded on demand; the baseline bundle SHALL include only the active locale. Bundle size for any single locale SHALL not exceed 150 KB gzipped. | NFR | All | Must | Prevents all-locale payload bloat; keeps initial load time within performance budget. |

### RTL / Bidirectional Layout

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-I18N-020 | Full RTL Layout for RTL Locales | When the active locale is RTL (Arabic, Hebrew, Persian, Urdu, and any future RTL locale), the entire application layout SHALL flip to right-to-left: flex direction, margin/padding logical properties (`margin-inline-start/end`), icon mirroring, and scroll anchoring. No component SHALL use physical CSS properties (`margin-left`, `padding-right`) that hard-code directionality. | Functional | All | Must | Arabic and Hebrew are Tier-1 enterprise markets; layout breakage under RTL is a blocker. |
| REQ-I18N-021 | `dir` Attribute Propagation | The `<html>` and `<body>` elements SHALL have their `dir` attribute set to `"rtl"` or `"ltr"` dynamically based on the active locale. All Radix UI primitives and Tailwind utilities SHALL use CSS logical properties throughout. | Functional | All | Must | `dir` on root triggers browser bidi algorithm; logical CSS properties respect it automatically. |
| REQ-I18N-022 | Diagram Canvas RTL | The architecture diagram canvas (see V03) SHALL support RTL layout: text within shape labels SHALL be right-aligned for RTL locales; connector routing SHALL be mirrored; the sidebar panel SHALL appear on the left (which is the reading-start side in RTL). | Functional | Enterprise Architect | Must | A diagram with LTR labels in an otherwise RTL UI creates a disorienting cognitive clash. |
| REQ-I18N-023 | Bidi Text Isolation | Inline strings that mix LTR and RTL content (e.g., an English identifier inside an Arabic sentence) SHALL be wrapped with Unicode bidi isolation markers (`U+2068`/`U+2069`, or the CSS `unicode-bidi: isolate` property) to prevent bidi algorithm contamination. | Functional | All | Must | Without isolation, punctuation and numbers adjacent to RTL text can visually reorder. |
| REQ-I18N-024 | Bidirectional Icon Mirroring | Directional icons (arrows, chevrons, breadcrumb separators, progress indicators, back/forward navigation) SHALL be mirrored horizontally in RTL mode. Icons that are symmetric or non-directional (e.g., warning triangle, avatar) SHALL NOT be mirrored. | UX | All | Must | Following Material Design and Apple HIG RTL icon guidance; directional cues must match reading direction. |
| REQ-I18N-025 | RTL Input Fields | Text input fields, text areas, and search boxes SHALL have `dir="auto"` so that user-typed RTL content right-aligns automatically regardless of the UI layout direction. | UX | All | Must | Ensures Arabic/Hebrew users can type naturally even when the form label is in a mixed context. |

### Locale-Aware Formatting

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-I18N-030 | Date & Time Formatting | All displayed dates and times SHALL be formatted using the `Intl.DateTimeFormat` API (or ICU-equivalent) with locale-specific patterns. The system SHALL never format dates by hand (e.g., string concatenation of month/day/year). | Functional | All | Must | Date format ambiguity (MM/DD vs. DD/MM) causes data misreading; locale-native patterns eliminate it. |
| REQ-I18N-031 | Relative Time | Relative timestamps ("3 minutes ago", "in 2 days") SHALL use `Intl.RelativeTimeFormat` with the active locale so the output phrase is grammatically correct in that language. | UX | All | Should | "Il y a 3 minutes" vs. "3 minutes ago"; important for audit log readability. |
| REQ-I18N-032 | Number & Decimal Formatting | Numbers SHALL be formatted with `Intl.NumberFormat` using locale-appropriate decimal separator (`.` or `,`), grouping separator, and digit grouping size (e.g., South Asian lakh system `1,00,000`). | Functional | All | Must | European locales use `.` as thousands separator and `,` as decimal; hard-coded dots cause misreading. |
| REQ-I18N-033 | Currency Display | Monetary values SHALL display the correct ISO 4217 currency symbol and position (prefix/suffix per locale) and use locale-aware number formatting. Currency code is derived from the tenant or user's regional config (see V13). | Functional | All | Must | "$1,234.56" vs. "1.234,56 €" vs. "¥1,234"; wrong format erodes trust in financial data. |
| REQ-I18N-034 | Unit Formatting (Measurement) | Measurement values (file sizes, durations, distances) SHALL use `Intl.NumberFormat` with `unit` option and respect locale preference for metric vs. imperial units. | UX | All | Should | Relevant for performance metrics, diagram scale, and roadmap duration displays. |
| REQ-I18N-035 | List Formatting | Conjunctive and disjunctive lists ("A, B, and C" vs. "A, B et C") SHALL use `Intl.ListFormat` with the active locale rather than hard-coded connectors. | Functional | All | Should | Used in AI-generated prose summaries, validation messages, and filter descriptions. |
| REQ-I18N-036 | Collation & Sorting | All user-visible sorted lists (artifact lists, dropdown options, table columns) SHALL sort using `Intl.Collator` with the active locale to handle locale-specific ordering (e.g., ä after z in Swedish, before b in German, ñ after n in Spanish). | Functional | All | Should | ASCII-byte sort produces wrong ordering in non-English locales; user-visible alphabetical sorts must be locale-correct. |

### Timezone Handling

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-I18N-040 | Per-User Timezone | All datetime values SHALL be stored as UTC internally and converted to the user's configured timezone (see V14) for display. The user SHALL be able to set their timezone independently of their locale. | Functional | All | Must | Architects in different timezones on the same tenant must see correct local times for meetings, deadlines. |
| REQ-I18N-041 | Timezone-Aware Scheduling UI | Any scheduling or calendar UI element (e.g., roadmap milestones with deadlines) SHALL display a timezone indicator and, where relevant, offer conversion hover-tooltips for other configured timezones. | UX | Enterprise Architect, PM Architect | Should | Prevents scheduling ambiguity in globally distributed teams. |
| REQ-I18N-042 | DST Handling | The system SHALL correctly handle Daylight Saving Time transitions using the IANA timezone database (via `Temporal` API or a maintained polyfill). DST edge cases SHALL not cause off-by-one-hour errors in displayed dates. | Functional | All | Must | Hardcoded UTC offsets (e.g., `UTC+1`) are wrong during DST periods; IANA database is the authoritative source. |

### Locale-Aware Input

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-I18N-050 | Input Method Editor (IME) Support | All text input fields SHALL correctly support IME composition for CJK languages (Japanese, Chinese, Korean) and other composition-based input. Inline validation SHALL NOT trigger during IME composition sequences (before `compositionend`). | Functional | All | Must | Premature validation during IME composition produces spurious errors and breaks CJK typing. |
| REQ-I18N-051 | Locale-Aware Date Input | Date input components SHALL accept dates in the locale's conventional format (DD/MM/YYYY vs. YYYY-MM-DD vs. MM/DD/YYYY) and parse them unambiguously, converting internally to ISO 8601. | UX | All | Must | Requiring ISO 8601 from European users who expect DD/MM/YYYY creates data-entry errors. |
| REQ-I18N-052 | Locale-Aware Number Input | Numeric input fields SHALL accept the locale's decimal and grouping separators (e.g., a German user typing `1.234,56`) and parse to a normalized float internally. | UX | All | Should | Pasting locale-formatted numbers into a form should not require manual reformatting. |
| REQ-I18N-053 | Keyboard Shortcut Localization | Keyboard shortcuts that conflict with OS-level input method keys (e.g., Alt+key combinations used by Windows for special characters in European keyboards) SHALL provide a locale-safe fallback or be user-remappable. | UX | All | Should | A shortcut that works on a US keyboard may be a dead key or compose sequence on a French AZERTY keyboard. |

---

## Area II — Localization (Content)

### AI-Assisted Translation of Deliverables

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-I18N-060 | Deliverable Translation Workflow | The system SHALL provide a one-click "Translate Deliverable" action on any generated artifact (proposal, architecture document, diagram export) that invokes an AI translation service (see V06) to produce a locale-specific version, with the output stored as a sibling artifact in the same project. | Functional | Enterprise Architect, Presales Architect | Must | Global RFP responses require deliverables in the client's language; manual translation is a bottleneck. |
| REQ-I18N-061 | Diagram Label Translation | The diagramming engine (see V03) SHALL support per-locale label overlays: shape labels and connector annotations SHALL be replaceable with translated equivalents without modifying the underlying diagram structure. The base diagram remains the source of truth. | Functional | Enterprise Architect | Must | Diagram structure is language-neutral; labels are the only locale-variant data. |
| REQ-I18N-062 | Proposal Section Translation | The AI proposal generator (see V22) SHALL support generating proposal sections directly in a specified target locale, as an alternative to post-hoc translation, to produce more idiomatic output. | AI/ML | Presales Architect | Should | Generating in the target language is idiomatically superior to translating from English for many languages. |
| REQ-I18N-063 | Translation Quality Scoring | For each AI-translated deliverable, the system SHALL compute and display a translation quality confidence score (e.g., based on COMET or BLEU proxy) to indicate review urgency. | AI/ML | Tenant Admin | Should | Enables triage: high-confidence translations can go straight to light review; low-confidence ones need a human. |
| REQ-I18N-064 | Human Review / Approve Workflow | Translated deliverables SHALL have a review state machine: Draft Translation → Pending Review → Approved → Published. Designated reviewers SHALL be able to inline-edit translation segments and approve. Approvals SHALL be timestamped and attributed. | Functional | Tenant Admin, Enterprise Architect | Must | Regulated industries and client-facing documents require human approval before delivery. |
| REQ-I18N-065 | Source-Translation Version Traceability | Each translated artifact SHALL record the source artifact version it was translated from. If the source is updated, the system SHALL flag the translation as "stale" and prompt for re-translation or delta-review of changed sections only. | Governance | All | Must | Without version linkage, translated documents silently diverge from the source when the source is revised. |

### Translation Memory

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-I18N-070 | Tenant-Scoped Translation Memory (TM) | Each tenant SHALL have a translation memory (TM) database that stores confirmed source-segment → target-segment pairs per language pair. When translating new content, the system SHALL query the TM and surface exact matches (100%) and fuzzy matches (≥75%) for human confirmation. | Functional | Tenant Admin | Must | TM reuse eliminates redundant translation cost and ensures consistency across documents. |
| REQ-I18N-071 | TM Fuzzy Matching | The TM engine SHALL support configurable fuzzy-match thresholds (default 75%). Fuzzy matches SHALL be highlighted in the review UI, showing the diff between the stored and current source segments. | Functional | Tenant Admin | Should | Fuzzy matching handles minor edits to previously translated segments (e.g., a version number change). |
| REQ-I18N-072 | TM Population from Approved Translations | Every segment approved in the human review workflow (REQ-I18N-064) SHALL be automatically added to the TM. TM entries SHALL be versioned and attributed to the approver. | Functional | Tenant Admin | Must | TM value compounds with use; the approval workflow is the authoritative quality gate for TM population. |
| REQ-I18N-073 | Cross-Tenant TM Isolation | TM data SHALL be strictly tenant-scoped. No segment from Tenant A's TM SHALL be visible to or suggested for Tenant B. | Governance | All | Must | TMs contain confidential client-facing language; cross-tenant leakage is a data breach. |
| REQ-I18N-074 | TM Export/Import (TMX) | Tenants SHALL be able to export their TM in TMX 1.4 format and import a pre-existing TMX file to seed the TM from external CAT tools (e.g., SDL Trados, memoQ, Phrase). | Integration | Tenant Admin | Should | Enterprises with existing translation workflows can port their TM into ESIP rather than starting from zero. |

### Per-Locale Glossary & Terminology

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-I18N-080 | Tenant Glossary per Language Pair | Each tenant SHALL be able to maintain a glossary of preferred translations for domain-specific terms (e.g., product names, technical jargon, brand terms) per source→target language pair. Glossary terms SHALL be enforced in AI translation and flagged for human review if violated. | Functional | Tenant Admin | Must | A product named "Blueprint" must not be translated as the common noun "blueprint" in German ("Entwurf"); controlled terms enforce branding. |
| REQ-I18N-081 | Do-Not-Translate (DNT) Terms | Certain terms (brand names, acronyms, proper nouns) SHALL be marked as Do-Not-Translate in the glossary. The translation engine SHALL pass these through verbatim and the review UI SHALL highlight any instance where they were incorrectly translated. | Functional | Tenant Admin | Must | ESIP, TOGAF, ArchiMate, and customer product names must remain untranslated. |
| REQ-I18N-082 | Glossary Import/Export (TBX) | Glossaries SHALL be exportable and importable in TBX (TermBase eXchange) 2.0 format to interoperate with industry terminology management tools. | Integration | Tenant Admin | Should | Enterprises typically maintain terminology in specialized tools; TBX is the ISO standard interchange format. |

### Framework Terminology Localization

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-I18N-090 | TOGAF Term Translations | The system SHALL ship a baseline glossary of standard TOGAF 10 terms (Architecture Domain, Building Block, Capability, Phase, etc.) pre-translated into all supported locales, sourced from the Open Group's official translations where available. | Functional | Enterprise Architect | Must | Architects in non-English organizations use TOGAF terms in their own language; incorrect translations undermine credibility. |
| REQ-I18N-091 | Zachman Framework Term Translations | Zachman Framework cell names and interrogatives (What/How/Where/Who/When/Why → Data/Function/Network/People/Time/Motivation) SHALL be pre-translated and available in the locale glossary. | Functional | Enterprise Architect | Should | V02 supports Zachman; localized labels must be accurate for non-English deployments. |
| REQ-I18N-092 | DoDAF / MODAF / UAF Term Translations | DoDAF 2.02, MODAF, and UAF viewpoint names and stereotype labels SHALL have pre-translated baseline entries in supported locales, with tenant override capability. | Functional | Defense/Government Architect | Should | DoDAF deployments in NATO non-Anglophone contexts require accurate local-language viewpoint terminology. |
| REQ-I18N-093 | ArchiMate Term Translations | ArchiMate 3.2 element, relationship, and aspect names SHALL be pre-translated per The Open Group ArchiMate specifications, where official translations exist, and machine-translated + reviewed otherwise. | Functional | Enterprise Architect | Must | ArchiMate diagrams generated by V03 with locale labels must use standardized terms. |
| REQ-I18N-094 | Framework Term Override | Tenants SHALL be able to override any pre-shipped framework term translation with their own preferred rendering. Overrides are scoped to the tenant and versioned. | Functional | Tenant Admin | Should | Some organizations have established in-house translations that differ from official ones; tenant override respects this. |
| REQ-I18N-095 | Localized Framework Guidance Tooltips | Contextual help tooltips and inline definitions for framework constructs SHALL be displayed in the user's locale, drawing from the translated framework glossary and the AI knowledge base (see V08). | UX | Enterprise Architect | Should | Reduces friction for non-English-native architects who need definitions in their own language. |

---

## Area III — AI in the User's Language

### AI Prompt Pack Localization

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-I18N-100 | Locale-Aware AI Prompt Packs | AI prompt packs (see V21) SHALL have locale variants: system prompts, role personas, and instruction scaffolds SHALL be translated and culturally adapted so that the AI produces drafts, critiques, and structured outputs in the user's active locale without requiring the user to specify the language. | AI/ML | All | Must | A German architect who has set locale `de-DE` expects AI-generated content in German without prompting; locale should be implicit context. |
| REQ-I18N-101 | Locale Injection into AI Context | Before invoking any AI capability, the system SHALL inject the user's locale code, preferred language name, and any tenant-specific terminology glossary into the AI system prompt context so that AI output language and terminology are consistent. | AI/ML | All | Must | Without explicit locale injection, AI defaults to English; locale must be a first-class context variable, not an afterthought. |
| REQ-I18N-102 | AI Output Language Consistency | When a user has set locale `fr-FR`, all AI-generated text (proposals, architecture descriptions, review comments, summaries, diagram labels) SHALL be in French, unless the source material is in a different language and the user requests a translation. | AI/ML | All | Must | Mixed-language AI output (English paragraphs in a French-locale session) is a regression; language consistency is a quality invariant. |
| REQ-I18N-103 | Locale-Specific AI Critique Style | AI critique and review prompts SHALL be adapted for locale-specific professional register and communication norms (e.g., German technical directness vs. Japanese indirect politeness registers) through locale-specific prompt variants in V21. | AI/ML | Enterprise Architect | Should | Communication style expectations differ strongly by culture; a one-size-fits-all critique tone can feel inappropriate. |
| REQ-I18N-104 | Prompt Pack Translation Pipeline | New AI prompt packs SHALL include a translation step in the authoring workflow: prompt authors create the canonical English version, and the system generates locale variants via AI translation + human review before the pack is published for non-English locales. | Governance | Platform Admin | Must | Ensures locale coverage grows in step with new prompt packs rather than lagging. |

### Multilingual Ingestion & Semantic Search

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-I18N-110 | Language Detection on Ingest | When a document is ingested into the Knowledge Hub (see V08), the system SHALL automatically detect the document's language using a language identification model (e.g., langdetect, fastText langid) and tag it with the detected language code. | AI/ML | Knowledge Manager | Must | Downstream semantic search and translation depend on knowing the source language; untagged multilingual corpora produce poor retrieval. |
| REQ-I18N-111 | Multilingual Embedding Models | The vector embedding pipeline (see V08) SHALL use multilingual embedding models (e.g., `multilingual-e5-large`, `paraphrase-multilingual-mpnet-base-v2`) capable of representing semantically equivalent content across languages in a shared embedding space. | AI/ML | Knowledge Manager | Must | Monolingual English embeddings cannot retrieve French documents using German queries; a multilingual space enables cross-lingual semantic search. |
| REQ-I18N-112 | Cross-Lingual Semantic Search | The knowledge hub SHALL support cross-lingual Q&A: a user querying in French SHALL retrieve relevant documents whether those documents are in French, English, German, or any other supported language, ranked by semantic relevance. | AI/ML | All | Must | Global organizations have heterogeneous-language document corpora; language of the document must not limit discoverability. |
| REQ-I18N-113 | Answer Synthesis in User Locale | When the AI synthesizes an answer from retrieved multilingual sources (see V08), the synthesized answer SHALL be rendered in the user's active locale, regardless of the source document languages. | AI/ML | All | Must | Users expect answers in their own language even when the underlying sources are English. |
| REQ-I18N-114 | Per-Language Chunking Strategy | Document chunking for indexing SHALL respect language-specific sentence and paragraph boundaries (using locale-aware NLP tokenizers, not byte-count splits) to avoid chunk-boundary artifacts that degrade retrieval quality for non-English text. | AI/ML | Knowledge Manager | Should | CJK languages lack whitespace word delimiters; chunking by byte count or whitespace token produces garbage chunks for Japanese, Chinese, Arabic. |

### AI Output Quality for Non-English Locales

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-I18N-120 | Non-English AI Output Quality Monitoring | The system SHALL track AI output language accuracy (correct locale used, no English leak-through, correct terminology) as a monitored metric. Alerts SHALL fire if the measured English-leak rate across non-English sessions exceeds 5%. | Governance | Platform Admin | Must | Silent language regression is hard to detect without automated monitoring; a metric + alert makes it visible. |
| REQ-I18N-121 | AI Locale Self-Check | AI generation flows SHALL include a post-generation step that verifies the output language matches the requested locale before delivery. If mismatch is detected, the output SHALL be re-generated or flagged for review before display. | AI/ML | All | Should | Model hallucinations occasionally produce English output despite non-English system prompts; a language-detect gate catches this. |
| REQ-I18N-122 | Locale Fidelity in Structured Outputs | AI-generated structured outputs (JSON schemas, YAML configuration, data tables) SHALL keep structural keys in their canonical (English) form but populate human-visible string values in the user's locale. | AI/ML | All | Must | Mixing localized keys into structured data breaks downstream parsing; only display strings should be localized. |

---

## Area IV — Globalization (g11n)

### Locale & Region Configuration

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-I18N-130 | Supported Locale Registry | The platform SHALL maintain a registry of supported locales (BCP 47 language tags) with metadata: LTR/RTL, plural form count, calendar system, first day of week, and paper size. Adding a new locale SHALL follow a defined on-ramp process (translate, QA, ship). | Functional | Platform Admin | Must | A registry makes locale support explicit and managed rather than ad-hoc. |
| REQ-I18N-131 | Tenant Default Region & Locale | Tenant configuration (see V11) SHALL include: default locale, default timezone, default currency, default paper size (A4 vs. Letter), default date format, and default units system (metric/imperial). These SHALL be applied as defaults for new users under that tenant. | Functional | Tenant Admin | Must | Ensures a coherent regional experience from first login for all users under a tenant. |
| REQ-I18N-132 | Regional Content Variants | The system SHALL support tagging content templates and AI prompt packs with regional applicability (e.g., a US-specific compliance checklist vs. an EU-specific one) so that regionalized content is surfaced preferentially for the user's region. | Functional | All | Should | Compliance requirements differ sharply by region; irrelevant regional content creates noise. |
| REQ-I18N-133 | Calendar System Support | The date picker and calendar UI components SHALL support Gregorian, Hijri (Islamic), Hebrew, Persian (Solar Hijri), and Japanese imperial calendar systems, switchable per locale or explicit user setting. | Functional | All | Should | Saudi Arabia, Israel, Iran, and Japan use non-Gregorian calendars in official business contexts. |
| REQ-I18N-134 | First Day of Week | Calendar and week-based views SHALL use the locale-appropriate first day of the week (Sunday in US/Canada, Monday in most of Europe, Saturday in parts of the Middle East) per CLDR data. | UX | All | Should | A Saudi user expects Saturday as the start of the week; defaulting to Sunday/Monday is jarring. |
| REQ-I18N-135 | Address Format per Region | Where the application collects or displays postal addresses (billing, org profile), it SHALL use region-specific address formats (CLDR address format data): field order, required fields, postal code placement, and province/state label. | Functional | Tenant Admin | Should | US "State/ZIP" vs. UK "Town/Postcode" vs. Japan "Postal code → Prefecture → City → Street" are structurally different; a generic form produces invalid addresses. |
| REQ-I18N-136 | Name Format per Region | Person name display and input SHALL respect locale-specific conventions: Western "Given Family" vs. East Asian "Family Given" vs. Arabic honorific-first ordering. | UX | All | Should | Displaying "Smith John" to a Western user or "John Smith" to a Japanese user where given name should come last is disrespectful. |

### Regional Compliance

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-I18N-140 | Right to Be Forgotten per Region | The platform SHALL implement GDPR Article 17 (EU), UK GDPR, and equivalent regional right-to-erasure obligations. A data erasure request SHALL purge all PII including name, email, locale preferences, and any user-authored content from all stores (primary DB, vector indices, TMs, audit logs where legally permitted). | Governance | Tenant Admin | Must | GDPR right-to-erasure is a legal obligation for EU deployments; failure to implement is a regulatory risk. |
| REQ-I18N-141 | Data Residency per Region | The platform SHALL support data residency configuration: tenant data (documents, TMs, glossaries, user profiles) SHALL be stored in a cloud region matching the tenant's regulatory jurisdiction (EU, US, APAC). AI inference calls SHALL respect data residency constraints where model endpoints are available in-region. | Governance | Tenant Admin | Must | EU GDPR and data sovereignty laws in Germany, France, and Saudi Arabia require local data storage. |
| REQ-I18N-142 | Locale-Specific Legal Notices | Terms of service, privacy notice, and cookie consent notices SHALL be presented in the user's locale and jurisdiction-appropriate legal language. The system SHALL store locale-specific versions of legal documents with effective dates and version history. | Governance | All | Must | Presenting English-only legal notices to EU users may be unenforceable; GDPR implies clear-language notices. |

---

## Area V — Theming & White-Label

### Theme System Architecture

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-THM-001 | Design Token Foundation | The entire UI SHALL be styled via a structured design token system (W3C Design Token Community Group format or compatible) with three tiers: (1) primitive tokens (raw values: `color.blue.500 = #3B82F6`), (2) semantic tokens (`color.interactive.primary = {color.blue.500}`), (3) component tokens (`button.primary.background = {color.interactive.primary}`). No hardcoded hex values or pixel values SHALL appear outside the token definitions. | Functional | Platform Admin | Must | Tiered tokens allow tenant brand changes to propagate automatically through the component tree without per-component overrides. |
| REQ-THM-002 | CSS Custom Property Delivery | Design tokens SHALL be compiled to CSS custom properties (`--esip-color-interactive-primary`) and injected at the `:root` or a tenant-scoped scope. The React component library SHALL reference only custom properties, never literal values. | Functional | All | Must | CSS custom properties are runtime-overridable without recompile, enabling per-tenant theming via a single scope override. |
| REQ-THM-003 | Tailwind v4 Token Integration | The Tailwind v4 configuration SHALL map to the design token system: Tailwind utility classes SHALL reference CSS custom properties so that theme changes automatically propagate to utility-class-based layouts. | Functional | Platform Admin | Must | ESIP uses Tailwind v4; decoupling Tailwind from hardcoded values ensures the theme system is complete and not bypassed. |
| REQ-THM-004 | Token Schema Validation | Token files SHALL be validated against a JSON Schema on every build and theme publish. Invalid token references (cyclic, missing primitives) SHALL fail CI. | Governance | DevOps | Must | Prevents broken themes from reaching production; enforces token graph integrity. |

### Built-In Themes

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-THM-010 | Light Theme | The system SHALL ship a complete light theme as the default, with color contrast meeting WCAG 2.2 AA (4.5:1 for normal text, 3:1 for large text and UI components) across all interactive and informational states. | Functional | All | Must | Light theme is the universal baseline; contrast compliance is a legal accessibility requirement in many jurisdictions. |
| REQ-THM-011 | Dark Theme | The system SHALL ship a complete dark theme where background and foreground tokens are inverted with perceptual-luminance adjustments (not naïve hex inversion), all component states (hover, focus, disabled, error) are visually distinct, and WCAG 2.2 AA contrast is maintained. | Functional | All | Must | Dark mode is a strong user expectation in developer-facing and design tools; many enterprise architects work late; dark mode reduces eye strain. |
| REQ-THM-012 | High-Contrast Accessibility Theme | The system SHALL ship a high-contrast theme (light-on-dark and dark-on-light variants) meeting WCAG 2.2 AAA contrast ratios (7:1 for normal text). This theme SHALL also override border radii and decoration to maximize edge clarity. | Functional | All | Must | WCAG AAA high-contrast is required for accessibility compliance in EU (EN 301 549) and US government (Section 508) contexts. |
| REQ-THM-013 | System Theme Sync | The active theme SHALL default to the OS-level preference (`prefers-color-scheme: dark/light`) unless the user has set an explicit preference (see V14). The system SHALL re-apply the OS preference if the user switches OS theme mid-session and has no stored override. | UX | All | Must | Users expect apps to respect their OS dark/light choice automatically. |
| REQ-THM-014 | Reduced-Motion Theme Variant | All animations, transitions, and scroll behaviors SHALL be disabled or replaced with instant transitions when `prefers-reduced-motion: reduce` is active. The system SHALL provide a manual toggle in addition to the media query. | UX | All | Must | Required for users with vestibular disorders; CSS motion queries alone are insufficient without a manual override for users on shared machines. |
| REQ-THM-015 | Dyslexia-Friendly Font Option | The system SHALL provide an opt-in font theme that substitutes the default typeface with a dyslexia-friendly font (e.g., OpenDyslexic or Lexie Readable) and applies wider letter spacing and increased line height. | UX | All | Should | Dyslexia affects approximately 10% of the population; a dedicated font option materially reduces reading difficulty without impacting non-dyslexic users. |

### Tenant Brand Themes

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-THM-020 | Tenant Color Palette | Tenant administrators SHALL be able to configure a brand color palette: primary, secondary, accent, and neutral color scales. The system SHALL derive the full semantic token tree from these brand colors using a perceptual color algorithm (e.g., OKLCH-based shade generation) to maintain contrast compliance. | Functional | Tenant Admin | Must | Enterprises expect the tool to carry their brand colors; raw hex entry with automatic shade derivation prevents contrast failures from manual misconfiguration. |
| REQ-THM-021 | Tenant Logo | Tenant administrators SHALL be able to upload a logo (SVG preferred, PNG fallback, max 500 KB) that replaces the ESIP wordmark in the navigation header, login screen, and generated deliverable headers. Light and dark logo variants SHALL be supported. | Functional | Tenant Admin | Must | Brand presence requires logo replacement; SVG ensures crisp rendering at all DPI levels. |
| REQ-THM-022 | Tenant Typography | Tenants SHALL be able to specify up to three brand typefaces (heading, body, monospace) via Google Fonts URL, or by uploading WOFF2 font files. The design token system SHALL apply these typefaces globally. Font fallback chains SHALL be automatically appended to ensure legibility if font loading fails. | Functional | Tenant Admin | Should | Brand typography is a core brand identity element; default system fonts break brand alignment. |
| REQ-THM-023 | Tenant Custom CSS Override | As a power-user escape hatch, Tenant administrators SHALL be able to inject a scoped CSS custom property block (max 10 KB) that overrides any token, reviewed and sandboxed to prevent layout breakage outside the tenant scope. Injected CSS SHALL be sanitized to disallow `position: fixed`, z-index escalation, and cross-tenant selectors. | Functional | Tenant Admin | Could | Token system cannot cover every edge case; a scoped escape hatch with sanitization is safer than no override path. |
| REQ-THM-024 | White-Label Branding (Cross-ref V19) | At the white-label tier, tenants SHALL be able to: remove all ESIP branding (wordmark, "powered by" footer, documentation links), configure a custom application title, and deploy under their own subdomain or custom domain. This requirement coordinates with V19 (Enterprise Adoption) and V11 (Platform). | Functional | Tenant Admin | Must | White-label is a key enterprise differentiator; see V19 for white-label domain/SSO details; this volume owns the *visual* de-branding layer. |
| REQ-THM-025 | Theme Preview Before Publish | The theme/token designer SHALL provide a real-time preview panel showing the active tenant theme applied to representative app screens (navigation, a diagram canvas, a deliverable header, a modal, a form) before the theme is committed and published to tenant users. | UX | Tenant Admin | Must | Without preview, a misconfigured color token can make buttons invisible; preview is the primary safeguard against live-environment regressions. |
| REQ-THM-026 | Theme Versioning & Rollback | Every published tenant theme SHALL be stored as a versioned snapshot with a timestamp and author. Tenant administrators SHALL be able to roll back to any previous theme version within 90 days. | Governance | Tenant Admin | Must | A brand refresh gone wrong (e.g., an illegible color combination) needs an instant rollback path; version history also satisfies audit requirements. |
| REQ-THM-027 | Theme Change Audit Log | All tenant theme changes (token edits, logo uploads, CSS overrides, publishes, rollbacks) SHALL be recorded in the tenant audit log with actor, timestamp, and a diff of changed tokens. | Governance | Tenant Admin | Should | Audit trails for visual configuration changes are required in regulated industries (financial services, healthcare). |

### Per-User Theme Preference

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-THM-030 | User Light/Dark/System Override | Individual users SHALL be able to override the tenant default theme mode (light/dark/system) from their profile settings (see V14). The per-user preference SHALL persist across sessions and devices. | UX | All | Must | Tenant-default dark theme should not override a user's personal light preference; individual autonomy must be respected. |
| REQ-THM-031 | User Font Size Scaling | Users SHALL be able to set a global font size scale factor (80%, 100%, 115%, 130%) that scales the base font size token, causing all relative-unit (`rem`/`em`) typography and spacing to scale proportionally. | UX | All | Must | Accessibility: users with low vision need larger text without relying on browser zoom which breaks fixed-pixel layouts. |
| REQ-THM-032 | User Cannot Override Tenant Brand Colors | Per-user theme preferences SHALL be limited to mode (light/dark), font scale, motion, and font choice. Users SHALL NOT be able to override tenant brand colors or logo, preserving brand consistency within a tenant deployment. | Governance | All | Must | White-label tenants require visual consistency; brand color overrides at the user level would break this. |

### Accessibility Themes

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-THM-040 | WCAG 2.2 AA Baseline for All Themes | Every built-in theme and every published tenant brand theme SHALL pass automated WCAG 2.2 Level AA contrast checks. The theme publication pipeline SHALL include an automated axe-core or equivalent audit; themes failing AA checks SHALL be blocked from publishing with specific token violations reported. | Functional | All | Must | Accessibility compliance is a legal baseline (ADA, EN 301 549, CVAA); automated enforcement prevents non-compliant themes from shipping. |
| REQ-THM-041 | Focus Indicator Visibility | All themes SHALL include a clearly visible keyboard focus indicator (minimum 3 px outline, contrasting against both light and dark adjacent surfaces) on all interactive elements. The focus indicator style SHALL not be removed or reduced below WCAG 2.2 Focus Appearance (Minimum) SC 2.4.11 thresholds. | Functional | All | Must | Focus indicators are a critical accessibility requirement for keyboard-only and switch-access users. |
| REQ-THM-042 | Color-Blind Safe Palettes | The built-in and tenant theme designer SHALL include a color-blind simulation tool (Deuteranopia, Protanopia, Tritanopia, Monochromacy) so designers can verify that information conveyed by color also uses shape/pattern/label distinctions. Status tokens (success, warning, error, info) SHALL include a non-color differentiator. | UX | Tenant Admin | Should | Approximately 8% of males have color vision deficiency; status colors alone (red/green) create critical accessibility failures. |
| REQ-THM-043 | Print / Export Theme | The system SHALL provide a print-optimized theme (white background, dark text, hidden navigation chrome, expanded abbreviations) applied via CSS `@media print` that is triggered for browser print and for PDF export. The print theme SHALL be customizable at the tenant level. | Functional | All | Should | Screen-optimized dark themes with thin fonts are illegible when printed; a print theme ensures professional hard-copy output. |

### Theme/Token Designer

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-THM-050 | Visual Token Editor | The platform SHALL provide a visual design token editor (within the Tenant Admin console) where administrators can edit semantic tokens via color pickers, typography controls, and spacing sliders. Changes SHALL reflect live in the preview panel (REQ-THM-025). | Functional | Tenant Admin | Must | Non-developer brand managers must be able to configure brand tokens without editing JSON files. |
| REQ-THM-051 | AI Brand Color Suggestion | The token editor SHALL offer an AI-assisted mode where the user uploads a brand logo or pastes a brand hex color, and the system generates a full harmonious token set (primary scale, semantic tokens, surface and text tokens) with WCAG compliance guaranteed. | AI/ML | Tenant Admin | Should | Most brand managers have one or two brand colors, not a full design system; AI generation bridges this gap while enforcing accessibility. |
| REQ-THM-052 | Token Documentation | Each design token SHALL have a machine-readable description field displayed in the editor. The editor SHALL export a living token documentation page (HTML) that tenant design teams can reference when building external content that must match the ESIP deployment. | Functional | Tenant Admin | Could | Enterprises that build collateral outside ESIP need a reference document to match ESIP's brand configuration. |
| REQ-THM-053 | Shared Theme Library | Platform administrators SHALL be able to publish optional theme presets to a shared theme library accessible to all tenants as a starting point. Tenant-specific modifications do not affect the shared library entry. | Functional | Platform Admin | Could | Accelerates tenant onboarding; industry-standard color themes (e.g., government blue, professional neutral) reduce the effort to produce a compliant theme from scratch. |

### Theming Across Generated Outputs

| ID | Title | SHALL Description | Category | Persona(s) | Priority | Rationale |
|---|---|---|---|---|---|---|
| REQ-THM-060 | Tenant Theme in Generated Deliverables | Generated documents, presentations, and reports (see V10, V23) SHALL use the tenant brand color palette, typography, and logo configured in the theme system. The deliverable rendering engine SHALL receive a theme context object at generation time. | Functional | All | Must | Brand consistency requires that documents produced by ESIP carry the same visual identity as the ESIP UI. |
| REQ-THM-061 | Diagram Canvas Theme Sync | The architecture diagram canvas (see V03) SHALL respect the active UI theme: shape fill colors, connector colors, background, grid color, and text color SHALL all reference the design token system so that switching from light to dark theme produces a coherent diagram appearance. | Functional | Enterprise Architect | Must | A diagram with a white background embedded in a dark-themed UI creates a jarring visual inconsistency. |
| REQ-THM-062 | Print / Presentation Theme in Export | When a user exports a diagram or deliverable to PDF, PNG, or PowerPoint (see V10), the export SHALL apply the configured print/presentation theme tokens rather than the screen-display tokens, ensuring output is optimized for the target medium. | Functional | All | Must | Dark-mode canvas diagrams exported to PNG produce unreadable slides when embedded in a white PowerPoint deck. |
| REQ-THM-063 | White-Label in Exported Deliverables | When white-label mode is active (REQ-THM-024), generated deliverables SHALL carry no ESIP brand marks, footers, or watermarks. All headers, footers, and cover pages SHALL reflect only the tenant brand. | Functional | Tenant Admin | Must | White-label clients explicitly need ESIP's tool identity hidden from their end customers; deliverable headers are the most visible exposure point. |
| REQ-THM-064 | Theming in Presentation Mode | The full-screen presentation mode for architecture reviews (if provided) SHALL apply a dedicated presentation theme optimized for large-screen projection: larger base font size, high-contrast backgrounds, and simplified chrome. | UX | Enterprise Architect | Could | Presentation mode on a projector requires different density and contrast than desktop review mode. |

---

## Requirement Count Summary

| Sub-Prefix | Area | Count |
|---|---|---|
| REQ-I18N-001–007 | UI String Externalization | 7 |
| REQ-I18N-010–014 | Runtime Locale Switching | 5 |
| REQ-I18N-020–025 | RTL / Bidirectional Layout | 6 |
| REQ-I18N-030–036 | Locale-Aware Formatting | 7 |
| REQ-I18N-040–042 | Timezone Handling | 3 |
| REQ-I18N-050–053 | Locale-Aware Input | 4 |
| REQ-I18N-060–065 | AI-Assisted Deliverable Translation | 6 |
| REQ-I18N-070–074 | Translation Memory | 5 |
| REQ-I18N-080–082 | Per-Locale Glossary & Terminology | 3 |
| REQ-I18N-090–095 | Framework Terminology Localization | 6 |
| REQ-I18N-100–104 | AI Prompt Pack Localization | 5 |
| REQ-I18N-110–114 | Multilingual Ingestion & Search | 5 |
| REQ-I18N-120–122 | AI Output Quality — Non-English | 3 |
| REQ-I18N-130–136 | Locale & Region Configuration | 7 |
| REQ-I18N-140–142 | Regional Compliance | 3 |
| **REQ-I18N subtotal** | | **75** |
| REQ-THM-001–004 | Theme System Architecture | 4 |
| REQ-THM-010–015 | Built-In Themes | 6 |
| REQ-THM-020–027 | Tenant Brand Themes | 8 |
| REQ-THM-030–032 | Per-User Theme Preference | 3 |
| REQ-THM-040–043 | Accessibility Themes | 4 |
| REQ-THM-050–053 | Theme / Token Designer | 4 |
| REQ-THM-060–064 | Theming Across Generated Outputs | 5 |
| **REQ-THM subtotal** | | **34** |
| **TOTAL** | | **109** |

---

## Boundary Notes

| Concern | This Volume (V26) | Delegated Volume |
|---|---|---|
| Translation model selection, multilingual model infra | References V06; does not specify model choices | V06 AI Capabilities |
| Per-user locale and theme *storage* | References the stored preference; does not define storage schema | V14 Personalization |
| File rendering engines for DOCX/PPTX/PDF output | Specifies that theme context is passed to rendering; not the rendering itself | V10 Deliverables |
| Brand governance inside deliverable *templates* | Specifies deliverable output carries tenant theme; not template authoring rules | V23 Templates |
| Multi-tenancy config storage, tenant provisioning | References tenant config; not the config store schema | V11 Platform |
| Multi-currency billing and invoicing | Specifies currency *display* formatting; not billing logic | V13 Onboarding/Billing |
| White-label domain, SSO, enterprise tier entitlement | Specifies visual de-branding; not domain/SSO setup | V19 Enterprise Adoption |
| Multilingual ingestion pipeline architecture | Specifies language detection requirement; not ingestion pipeline design | V08 Knowledge Hub |
| Diagram canvas rendering engine | Specifies RTL and theme requirements *on* the canvas; not canvas engine internals | V03 Diagramming |
