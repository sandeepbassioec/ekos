# ESIP Knowledge Hub — Test Data Guide
## Scenario: FieldOps Mobile 2.0

This guide tells you exactly what to upload and when, and what the system SHOULD detect at each step.
Use it to compare AI output against expected results and identify prompt improvement opportunities.

---

## Files in this folder

| File | Upload As | Document Type |
|------|-----------|---------------|
| `FieldOps_Mobile_2.0_PRD.docx` | Document upload | Requirements |
| `FieldOps_2.0_Technical_Architecture.pdf` | Document upload | Technical |
| `FieldOps_2.0_Supporting_Data.xlsx` | Document upload | Supporting Data |
| `EMAIL_01_initial_scope_concern.txt` | Email (paste body) | Stakeholder Concern |
| `EMAIL_02_pm_response_and_clarification.txt` | Email (paste body) | PM Clarification |
| `EMAIL_03_gdpr_legal_blocker.txt` | Email (paste body) | Legal / Compliance |
| `CHAT_01_dev_team_api_design.txt` | Teams Chat (paste) | Engineering Discussion |
| `CHAT_02_offline_mode_conflict.txt` | Teams Chat (paste) | Scope Conflict |
| `MEETING_01_sprint1_planning.txt` | Meeting Transcript | Sprint Planning |
| `MEETING_02_conflict_resolution.txt` | Meeting Transcript | Conflict Resolution |

---

## Step-by-Step Upload Sequence

### STEP 1 — Upload the DOCX (PRD)
**File:** `FieldOps_Mobile_2.0_PRD.docx`
**Document Type:** Requirements Document

**Expected AI output:**
- **Requirements:** 20–30 requirements covering:
  - FR-AD-001 to FR-AD-005 (dispatch engine)
  - FR-LS-001 to FR-LS-005 (location service)
  - FR-DD-001 to FR-DD-005 (dashboard)
  - NFR-PERF, NFR-AVAIL, NFR-SEC, NFR-COST, NFR-SCALE, NFR-DATA
  - Constraints (6–8 items)
- **Gaps detected:**
  - No requirement for what happens when Kafka is down (single point of failure mentioned but no SLA)
  - dispatch_rules rule-builder UI is mentioned as out-of-scope without a replacement plan
  - No requirement for technician consent/opt-out of location tracking
  - Pre-emption customer notification (OQ-004) flagged as open — no formal requirement yet
- **Confusions detected:**
  - "SLA preemption requires remaining_time_estimate — this field does not exist on work_orders" (OQ-005 is open in PRD)
  - "Google Maps API rate limit analysis shows potential burst problem during high-concurrency dispatch"
  - "offline mode is 'under evaluation' — no clear decision criteria stated"
- **Suggestions:**
  - Add circuit-breaker timeout configuration as an NFR
  - Consider PostGIS for spatial queries (mentioned in tech PDF)
  - Define a formal decision gate for offline mode

---

### STEP 2 — Upload the PDF (Technical Architecture)
**File:** `FieldOps_2.0_Technical_Architecture.pdf`
**Document Type:** Technical Specification

**Expected AI output:**
- **New Requirements:** 5–10 additional technical requirements:
  - ETA source transparency (etaSource field) — appears in PDF but not PRD
  - GDPR erasure endpoint (Article 17) — first mention in PDF
  - JWT scope claims per endpoint
  - PostGIS evaluation question (TQ-002)
- **Gaps resolved:** None yet (no emails/chats added)
- **Gaps updated:**
  - "Kafka topic retention is mentioned as 7-day default in TQ-004 — conflicts with GDPR minimisation" → should flag as CONFLICT between PRD Section 7 (Kafka) and technical spec TQ-004
- **New Confusions:**
  - "WebSocket vs SSE for Dashboard (TQ-003) — decision not made"
  - "remaining_time_estimate problem (TQ-005) — two options presented but no decision"
  - "Scoring algorithm uses weighted sum — TQ-001 questions whether ML would be better; no decision criteria"
- **Conflict Detected:**
  - PRD says 7-day Kafka retention is acceptable default (Section 6 constraints don't mention Kafka retention). PDF says TQ-004 recommends 24h for GDPR. → CONFLICT: Kafka retention (PRD implicit vs PDF explicit)

---

### STEP 3 — Upload the XLSX (Supporting Data)
**File:** `FieldOps_2.0_Supporting_Data.xlsx`
**Document Type:** Supporting Data

**Expected AI output:**
- **New Requirements:**
  - API cost cap at $3,000/month confirmed with specific formula (Sheet 1)
  - At 2,000 technicians, current model = ~$2,700/month (within budget — confirms PRD)
  - New gap: model does NOT cover >2,000 technicians — no requirement exists for fleet scaling cost governance
- **Gaps detected:**
  - Sheet 2 (DB Schema Delta) shows `remaining_time_estimate` column is NOT in the schema delta — confirms TQ-005 gap
  - Sheet 2 shows `job_type_durations` table is NOT in schema delta — will be needed for TQ-005 resolution
  - Sheet 3: `skill_tags` backfill from skills join table is documented but no migration timeline / rollback plan stated
- **Confusions clarified:**
  - SLA breach history (Sheet 4) provides baseline data for MTTD = 22–25 min and FTFR = 59–62% — confirms PRD business goals are grounded in real data
- **No new conflicts expected at this step**

---

### STEP 4 — Add Email 1 (Scope Concern)
**File:** `EMAIL_01_initial_scope_concern.txt`
**Paste as:** Outlook Email
**From:** Daniel Frost | Subject: RE: FieldOps 2.0 PRD Review - Google Maps Cost Concern

**Expected AI output:**
- **NEW CONFLICT — HIGH severity:**
  - Title: "Google Maps cost cap breached at 3,200 technicians"
  - Description: PRD budget cap is $3,000/month, modelled at 2,000 technicians. Regional expansion (RegionEast contract) would bring fleet to 3,200, pushing estimated cost to ~$4,300/month — 43% over cap
  - Conflicting requirement: NFR-COST-001
  - Trigger source: Email from Daniel Frost
- **Gap updated:**
  - "HERE Maps formally rejected — documented in Confluence PROJ-FOPS-EVAL-001" (resolves confusion about whether alternative maps were considered)
- **New Confusion:**
  - "Batch PM route clustering disable at $2,500 alarm — PRD says auto-disable but this affects 40% of job routing; no degradation plan stated"

---

### STEP 5 — Add Email 2 (PM Response)
**File:** `EMAIL_02_pm_response_and_clarification.txt`
**Paste as:** Outlook Email
**From:** Priya Nair | Subject: RE: RE: FieldOps 2.0 PRD Review

**Expected AI output:**
- **Conflict resolved:** "Google Maps cost cap at 3,200 technicians" — PM proposes configurable MAX_TRACKED_TECHNICIANS cap + degraded batch frequency. Status should move toward Resolved or note that a formal decision is pending.
- **Gap resolved:**
  - "No degradation plan for batch job at cost alarm" — Email provides: reduce frequency daily→every 3 days + region priority
- **New Requirements identified:**
  - MAX_TRACKED_TECHNICIANS runtime config (new NFR / constraint)
  - Untracked technicians shown as greyed-out on map (new FR for Dashboard)
- **Confusion resolved:**
  - HERE Maps evaluation — confirmed rejected, reasons documented

---

### STEP 6 — Add Email 3 (GDPR Legal Blocker)
**File:** `EMAIL_03_gdpr_legal_blocker.txt`
**Paste as:** Outlook Email
**From:** Natasha Orlova (Legal/DPO)

**Expected AI output:**
- **NEW CONFLICT — CRITICAL severity:**
  - Title: "Kafka topic retention 7-day default violates GDPR data minimisation (Article 5)"
  - Conflicting requirement: NFR-DATA-001 (30-day DB retention) — PRD addresses DB retention but NOT Kafka retention. Email states max 24h for Kafka.
  - Note: this conflict was hinted in PDF TQ-004 — AI should link these
- **NEW REQUIREMENT (not in PRD):**
  - FR-MOB-001: Persistent location tracking banner on mobile home screen (mandatory compliance)
  - Legal constraint: no real technician data in UAT before DPA amendment signed
- **Gap resolved:**
  - TQ-005 partially: remaining_time_estimate is needed. Email confirms this as a blocker for pre-emption logic.
- **New Confusion:**
  - "DPA amendment expected 2026-08-21 per Legal's revised estimate (per Meeting 2) vs. no date in this email — what is the current committed date?"

---

### STEP 7 — Add Chat 1 (Dev Team API Design)
**File:** `CHAT_01_dev_team_api_design.txt`
**Paste as:** Teams Chat

**Expected AI output:**
- **Gap resolved:**
  - "Ping interval not adaptive" → Chat confirms team decision: 30s Travelling, 300s In Progress, server-directed. FR-LS-001 should be updated.
- **Gap resolved:**
  - "Auto-assign endpoint concurrency problem under high load" — Chat documents the ETA cache design (Option B)
- **New Requirements:**
  - ETA cache with SLA-tiered TTL (90s Normal/High, 20s Critical)
  - `etaSource` field in auto-assign API response
  - JSONLogic validator for dispatch_rules with 4 specific behaviours
- **Confusion resolved:**
  - "dispatch_rules JSONLogic validator not assigned to anyone" (OQ-003 engineering concern) — Ravi assigned
- **Suggestion:**
  - The Haversine fallback during cache miss should be transparent to dispatchers — `etaSource` field suggestion

---

### STEP 8 — Add Chat 2 (Offline Mode Conflict)
**File:** `CHAT_02_offline_mode_conflict.txt`
**Paste as:** Teams Chat

**Expected AI output:**
- **NEW CONFLICT — HIGH severity:**
  - Title: "Offline mode out-of-scope conflicts with real-world connectivity gaps causing false SLA alerts"
  - Description: PRD Section 3.2 explicitly marks offline mode as out of scope. Chat shows that 57% of Region East pilot technicians lose connectivity for 10-15 minutes regularly. With v2.0 live ETA tracking, missed pings trigger false SLA At Risk alerts, causing incorrect job reassignments.
  - Conflicting requirement: FR-LS-001 (location ping every 30s) vs. real-world constraint that connectivity cannot be guaranteed
- **New Gap:**
  - "No definition of 'location unknown' threshold — how many missed pings before a technician is flagged? Dashboard behaviour is undocumented."
- **New Suggestion:**
  - Offline ping buffer (AsyncStorage, 50 pings, flush on reconnect) — 4-day effort per Ahmed's assessment
- **Confusion:**
  - "Connectivity dead zone problem documented but no requirement for graceful degradation during connectivity loss"

---

### STEP 9 — Add Meeting 1 (Sprint Planning)
**File:** `MEETING_01_sprint1_planning.txt`
**Paste as:** Meeting Transcript

**Expected AI output:**
- **Multiple gaps resolved:**
  - Kafka retention: TQ-004 closed — 24h confirmed
  - Adaptive ping intervals: FR-LS-001 updated — 30s/300s confirmed
  - ETA cache design: confirmed with etaSource transparency
  - TQ-005 (remaining_time_estimate): Option B confirmed — computed from job_type_durations table
  - New table: `job_type_durations` reference table
- **Multiple confusions resolved:**
  - "Pre-emption logic needs remaining_time_estimate field" — solved via job_type_durations
  - "HERE Maps evaluation" — confirmed rejected, decision documented
- **New Requirements confirmed:**
  - FR-MOB-001: mobile tracking banner — 2-3h effort, Legal copy by end of week
  - Untracked technicians: greyed-out pin on Dashboard
  - GPS location simulator for UAT (synthetic data, snapped to roads)
  - `gdpr_erasure_log` table (mentioned briefly — see Meeting 2 for full spec)
- **Conflict resolution (partial):**
  - Offline mode: spike authorised but formal scope decision deferred to steering committee → conflict status = "In Progress" not yet resolved
- **Suggestion:**
  - Ahmed's spike cap of 2 days is a good smell test — if effort is >2 days offline mode needs a separate roadmap item

---

### STEP 10 — Add Meeting 2 (Conflict Resolution)
**File:** `MEETING_02_conflict_resolution.txt`
**Paste as:** Meeting Transcript

**Expected AI output (MAJOR — all major conflicts resolved):**
- **CONFLICT RESOLVED — Offline mode:**
  - Decision: IN SCOPE for v2.0. Offline ping buffer (50-ping AsyncStorage buffer, flush on reconnect, clientTimestamp replay). 4 days effort, Sprint 2.
  - PRD Section 3.2 updated from "out of scope" to in-scope.
- **CONFLICT RESOLVED — GDPR DPA amendment (OQ-005):**
  - DPA signed by 2026-08-21. UAT with real data from 2026-08-21. Go-live 2026-10-15 clear.
- **OPEN QUESTION CLOSED — OQ-003:**
  - dispatch_rules UI: NOT in v2.0. CLI tool only. v2.1 roadmap.
- **NEW REQUIREMENT — FR-NOTIF-001:**
  - When SLA preemption fires, send automated SMS to interrupted job customer via Twilio within 60 seconds.
  - New Kafka consumer on fieldops.dispatch-events for PreemptionEvent.
- **NEW REQUIREMENT:**
  - `gdpr_erasure_log` table: id, technician_id, requested_by_user_id, requested_at, rows_deleted_count, completed_at.
  - Sprint 2.
- **NEW REQUIREMENT:**
  - clientTimestamp field on POST /api/v2/location/ping — accepts historical timestamps for replay. Server stores both recorded_at and clientTimestamp.
- **All major open questions closed:**
  - OQ-001: battery death → last known location persists. Dispatcher must manually complete.
  - OQ-002: Google Maps rate limit — ETA cache solves burst problem; batch staggered.
  - OQ-003: no rule-builder UI in v2.0.
  - OQ-004: customer preemption SMS — FR-NOTIF-001.
  - OQ-005: GDPR DPA — resolved, date confirmed.
  - OQ-006: offline mode — resolved, in scope.

---

## What to Look for When Comparing AI Output

### 1. Requirement Extraction Quality
- Does AI extract all 8 scope areas from the DOCX (backend logic, new API, new endpoint, 3rd party, screen changes, new screen, new tables, existing table changes)?
- Does it create distinct requirements for each, or lump them?
- Does it catch the NFRs (cost, security, scaling, GDPR retention)?

### 2. Conflict Detection Accuracy
| Expected Conflict | Introduced By | Resolved By |
|---|---|---|
| Google Maps cost at 3,200 technicians | Email 1 | Email 2 |
| Kafka retention 7-day vs GDPR 24h | PDF (TQ-004) + Email 3 | Meeting 1 (TQ-004 closed) |
| Offline mode out-of-scope vs pilot data | Chat 2 | Meeting 2 |
| GDPR DPA amendment block on location tracking | Email 3 | Meeting 2 |

### 3. Gap Detection — PRD Blind Spots
The PRD intentionally omits these — AI should catch them:
- No requirement for technician consent display (found in Email 3 / Legal)
- No requirement for "untracked technician" visual indicator on map (found in Meeting 1)
- No `job_type_durations` table in schema delta (found in Meeting 1)
- No `clientTimestamp` field on ping endpoint (found in Meeting 2)
- No `gdpr_erasure_log` table design (found in PDF briefly, fully in Meeting 2)

### 4. Suggestion Quality
AI should suggest:
- ETA cache TTL strategy (hinted in Chat 1, confirmed Meeting 1)
- PostGIS vs DECIMAL for lat/lng (TQ-002 from PDF)
- Adaptive ping intervals (hinted PRD, confirmed Chat 1)
- SMS notification approach (found Meeting 2 — Twilio already integrated)

### 5. Traceability
Every requirement change introduced by email/chat/meeting should show:
- Which source triggered the update
- What changed (e.g. FR-LS-001 ping interval updated)
- If it resolved a gap or confusion

---

## Scoring Rubric (rough)

| Category | Excellent | Good | Needs Improvement |
|---|---|---|---|
| Req extraction from DOCX | 25+ requirements, all 8 scope areas | 15–25 reqs, 6+ scope areas | <15 reqs, misses scope areas |
| Conflict detection | All 4 conflicts detected | 2–3 conflicts | 0–1 conflicts |
| Gap detection | 8+ gaps, includes PRD blind spots | 4–7 gaps | <4 gaps |
| Confusion detection | 6+ confusions with clear questions | 3–5 confusions | <3 confusions |
| Differential re-analysis | Gaps/confusions updated when emails added | Some updates | No differential updates |
| Conflict resolution tracking | Conflicts marked resolved when meeting added | Some resolution tracking | No resolution tracking |
