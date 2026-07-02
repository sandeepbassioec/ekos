"""Generate FieldOps Mobile 2.0 PRD as DOCX"""
from docx import Document
from docx.shared import Pt, RGBColor
from docx.enum.text import WD_ALIGN_PARAGRAPH

doc = Document()

def h1(text):
    p = doc.add_heading(text, level=1)
    return p

def h2(text):
    return doc.add_heading(text, level=2)

def h3(text):
    return doc.add_heading(text, level=3)

def para(text):
    return doc.add_paragraph(text)

def bullet(text, level=0):
    p = doc.add_paragraph(text, style='List Bullet')
    p.paragraph_format.left_indent = Pt(18 * (level + 1))
    return p

doc.add_heading('FieldOps Mobile 2.0 — Product Requirements Document', 0)
para('Document Version: 1.3 | Status: In Review | Owner: Priya Nair (Head of Product)')
para('Last Updated: 2026-06-10 | Roadmap Item: Q3-2026 / Platform Modernisation')
doc.add_paragraph()

h1('1. Executive Summary')
para(
    'FieldOps Mobile 2.0 is a comprehensive modernisation of our field-service management platform. '
    'The current system (v1.4, built 2019) suffers from manual dispatch workflows, no real-time '
    'technician visibility, and a rigid work-order model that cannot accommodate SLA-tiered '
    'assignments. This initiative replaces those gaps with an intelligent auto-dispatch engine, '
    'a Live Dispatch Dashboard, native mobile enhancements, and a real-time location service — '
    'all underpinned by a Google Maps Platform integration for routing and ETA computation.'
)
para(
    'The estimated scope spans 14 weeks (Q3–Q4 2026). Primary stakeholders: Field Operations, '
    'Customer Success, Engineering, and Finance (cost governance of third-party API spend).'
)

h1('2. Business Goals & Success Metrics')
h2('2.1 Business Goals')
for g in [
    'Reduce mean-time-to-dispatch (MTTD) from 22 min to under 5 min',
    'Increase first-time fix rate (FTFR) from 61% to 75% by matching skill-set to job type',
    'Provide customers real-time ETA accuracy within ±10 minutes',
    'Enable SLA breach prevention via proactive re-routing when delays are detected',
    'Cut overtime cost 15% through optimised route clustering',
]:
    bullet(g)

h2('2.2 KPIs')
table = doc.add_table(rows=1, cols=3)
table.style = 'Light Grid Accent 1'
hdr = table.rows[0].cells
hdr[0].text = 'KPI'; hdr[1].text = 'Baseline'; hdr[2].text = 'Target'
rows = [
    ('MTTD (minutes)', '22', '< 5'),
    ('FTFR (%)', '61%', '75%'),
    ('ETA accuracy', 'No ETA shown', '±10 min, 90th pct'),
    ('Dispatcher FTE per 100 jobs', '1.8', '0.6'),
    ('Overtime hours / week', '340', '< 290'),
    ('Customer CSAT (dispatch)', '3.2 / 5', '4.2 / 5'),
]
for r in rows:
    row = table.add_row().cells
    row[0].text = r[0]; row[1].text = r[1]; row[2].text = r[2]
doc.add_paragraph()

h1('3. Scope')
h2('3.1 In-Scope')

h3('3.1.1 Backend Logic Change — Work-Order Assignment Engine')
para(
    'The existing assignment algorithm (simple FIFO queue) will be replaced with a priority-weighted, '
    'proximity-aware engine. New scoring factors: SLA tier (Critical/High/Normal), technician skill '
    'match score (0–100), current queue depth, current GPS location vs job site distance. '
    'The engine re-scores every 90 seconds while jobs remain unassigned. '
    'Business rule: a Critical SLA job always pre-empts a Normal job already in-progress if the '
    'technician is within 15 minutes of the critical site AND the in-progress job has >60 min remaining.'
)
para('Affected service: WorkOrderService (Java Spring). Target response time for scoring: <200 ms p99.')

h3('3.1.2 New API — Technician Location Service')
para(
    'A new microservice (Node.js / Express) will expose a Technician Location API. '
    'The mobile app will push GPS coordinates every 30 seconds when a job is active. '
    'The service will maintain last-known position and heading, compute ETA to next job site '
    'via Google Maps Routes API, and publish location events to an internal Kafka topic '
    '(topic: fieldops.technician-location) for consumption by the dispatch engine and dashboard.'
)
para('Base URL: /api/v2/location')
for ep in [
    'POST /api/v2/location/ping — mobile app pushes GPS fix (lat, lng, accuracy, batteryPct, jobId)',
    'GET  /api/v2/location/technician/{techId} — returns last position + ETA to active job',
    'GET  /api/v2/location/fleet?regionId={id} — returns all active technicians in a region',
    'DELETE /api/v2/location/technician/{techId}/session — ends tracking session on job completion',
]:
    bullet(ep)

h3('3.1.3 New Endpoint — Auto-Dispatch')
para(
    'POST /api/v2/fieldops/dispatch/auto-assign\n'
    'Accepts: { jobId, overrideSkillCheck: bool, forceAssignToTechId?: string }\n'
    'Returns: { assignedTechId, estimatedArrival, scoreBreakdown, alternativesConsidered }\n'
    'This endpoint triggers the assignment engine synchronously (timeout: 5 s) for on-demand dispatch. '
    'The existing POST /api/v1/dispatch/assign endpoint remains active (deprecated, sunset Q1 2027).'
)

h3('3.1.4 Third-Party Integration — Google Maps Platform')
para(
    'Integrate Google Maps Platform (Routes API + Distance Matrix API). '
    'Use case 1: real-time ETA computation for active jobs (Routes API, one call per active technician per 90-sec cycle). '
    'Use case 2: route optimisation when assigning a new job to a technician with existing queue > 1 job (Routes API, optimise_waypoints=true). '
    'Use case 3: daily route clustering for preventive maintenance jobs (Distance Matrix API, offline batch, runs at 06:00 UTC). '
    'Billing model: pay-per-use. Finance cap: $3,000/month. '
    'Circuit breaker: if Google Maps API error rate exceeds 10% over 60 seconds, the system falls back to straight-line distance (Haversine) and logs a P2 alert.'
)
para('API key stored in AWS Secrets Manager (secret: prod/fieldops/googlemaps). Rotation: 90 days.')

h3('3.1.5 Existing Screen Changes — Work Order List')
para(
    'The Work Order List screen (mobile + web) requires the following changes:\n'
    '• Add real-time status filter chips: All / Unassigned / In Progress / SLA At Risk / Completed\n'
    '• Each card now shows: technician avatar + name, ETA badge (green <30 min, amber 30–60 min, red >60 min or breached)\n'
    '• Tapping an SLA At Risk card opens a pre-populated re-assign dialog\n'
    '• Pull-to-refresh triggers a live data fetch (replaces the current 5-minute polling interval)\n'
    '• Batch action: select multiple Unassigned jobs → "Auto-Assign Selected" button calls /api/v2/fieldops/dispatch/auto-assign in parallel'
)

h3('3.1.6 New Screen — Live Dispatch Dashboard')
para(
    'A new web-only screen accessible to Dispatcher and Manager roles. Features:\n'
    '• Google Maps embed showing all active technicians (colour-coded by SLA exposure)\n'
    '• Right panel: sortable job queue by SLA deadline, unassigned count, at-risk count\n'
    '• Click technician pin → side-panel shows: current job, queue, skill score, availability window\n'
    '• Drag-and-drop job reassignment (updates via POST /api/v2/fieldops/dispatch/auto-assign with forceAssignToTechId)\n'
    '• SLA breach alert banner (WebSocket push from server; fallback: 30-second SSE poll)\n'
    '• Export to PDF: daily dispatch summary (job count, FTFR, avg ETA accuracy)'
)

h3('3.1.7 New Database Tables')
para('Three new tables introduced in the fieldops schema (PostgreSQL 15):')

h4_tables = [
    ('technician_locations',
     'id UUID PK, technician_id UUID FK(technicians.id), lat DECIMAL(10,8), lng DECIMAL(11,8), '
     'accuracy_meters INT, heading_degrees INT, speed_kmh DECIMAL(5,2), battery_pct SMALLINT, '
     'job_id UUID FK(work_orders.id) NULLABLE, recorded_at TIMESTAMPTZ, created_at TIMESTAMPTZ. '
     'Retention: 30 days (partitioned by recorded_at, monthly partitions). '
     'Index: (technician_id, recorded_at DESC).'),
    ('route_optimisations',
     'id UUID PK, technician_id UUID FK, optimisation_trigger ENUM(auto_assign, manual_reorder, batch_pm), '
     'job_ids UUID[] (ordered), total_distance_km DECIMAL(8,2), total_duration_min INT, '
     'google_maps_request_id VARCHAR(64), cost_usd DECIMAL(8,6), '
     'computed_at TIMESTAMPTZ, applied BOOLEAN DEFAULT FALSE, created_at TIMESTAMPTZ. '
     'Constraint: CHECK (array_length(job_ids,1) BETWEEN 1 AND 25).'),
    ('dispatch_rules',
     'id UUID PK, rule_name VARCHAR(128) UNIQUE, rule_type ENUM(sla_preemption, skill_filter, region_lock, time_window), '
     'priority INT, conditions JSONB, actions JSONB, is_active BOOLEAN DEFAULT TRUE, '
     'created_by UUID FK(users.id), created_at TIMESTAMPTZ, updated_at TIMESTAMPTZ. '
     'Note: conditions and actions are JSONLogic expressions evaluated at scoring time.'),
]
for name, desc in h4_tables:
    doc.add_heading(f'Table: {name}', level=4)
    para(desc)

h3('3.1.8 Existing Table Modifications')
para('Two existing tables require schema changes:')
mods = [
    ('work_orders',
     'ADD COLUMN assigned_method ENUM(manual, auto_engine, forced) DEFAULT manual\n'
     'ADD COLUMN sla_tier ENUM(critical, high, normal) DEFAULT normal\n'
     'ADD COLUMN eta_minutes INT NULLABLE\n'
     'ADD COLUMN last_eta_computed_at TIMESTAMPTZ NULLABLE\n'
     'ADD COLUMN dispatch_score_breakdown JSONB NULLABLE\n'
     'Migration: backfill assigned_method = manual for all existing rows. Zero-downtime: use ADD COLUMN with DEFAULT.'),
    ('technicians',
     'ADD COLUMN availability_status ENUM(available, on_job, break, offline) DEFAULT offline\n'
     'ADD COLUMN current_location_lat DECIMAL(10,8) NULLABLE\n'
     'ADD COLUMN current_location_lng DECIMAL(11,8) NULLABLE\n'
     'ADD COLUMN location_updated_at TIMESTAMPTZ NULLABLE\n'
     'ADD COLUMN skill_tags TEXT[] DEFAULT ARRAY[]::TEXT[]\n'
     'Migration: populate skill_tags from existing skills join table (one-time backfill script).'),
]
for name, desc in mods:
    doc.add_heading(f'Modified: {name}', level=4)
    para(desc)

h2('3.2 Out of Scope (v2.0)')
for item in [
    'iOS / Android native push notifications (planned v2.1)',
    'Customer-facing live tracking portal',
    'AI-based demand forecasting',
    'Integration with SAP Field Service Management',
    'Offline-first mobile support (cache-and-sync) — under evaluation, not committed',
]:
    bullet(item)

h1('4. Functional Requirements')

h2('4.1 Auto-Dispatch Engine')
reqs = [
    ('FR-AD-001', 'HIGH', 'The system SHALL score all available technicians against an unassigned job within 200ms (p99) using skill match, proximity, queue depth, and SLA tier.'),
    ('FR-AD-002', 'CRITICAL', 'A Critical-SLA job SHALL pre-empt a Normal-SLA in-progress job if the technician is ≤15 min ETA and the current job has >60 min remaining.'),
    ('FR-AD-003', 'HIGH', 'The engine SHALL re-score unassigned jobs every 90 seconds until assigned or manually overridden.'),
    ('FR-AD-004', 'MEDIUM', 'Dispatchers SHALL be able to override auto-assignment via the Dashboard with a mandatory reason field (audit logged).'),
    ('FR-AD-005', 'HIGH', 'The system SHALL expose scoreBreakdown in the auto-assign API response for transparency and audit.'),
]
for rid, pri, text in reqs:
    p = doc.add_paragraph()
    p.add_run(f'[{rid}] [{pri}] ').bold = True
    p.add_run(text)

h2('4.2 Location Service')
reqs2 = [
    ('FR-LS-001', 'HIGH', 'Mobile app SHALL send GPS ping every 30 seconds when a job status is In Progress or Travelling.'),
    ('FR-LS-002', 'HIGH', 'Location pings SHALL be accepted with <100ms API response time (p95) to avoid blocking the mobile app UI thread.'),
    ('FR-LS-003', 'MEDIUM', 'If GPS accuracy exceeds 50 metres, the ping SHALL be accepted but flagged; ETA computation will use the previous accurate fix.'),
    ('FR-LS-004', 'HIGH', 'All location data SHALL be encrypted in transit (TLS 1.3) and at rest (AES-256). Retention: 30 days then hard-delete.'),
    ('FR-LS-005', 'MEDIUM', 'Location tracking SHALL automatically cease when job status transitions to Completed or Cancelled.'),
]
for rid, pri, text in reqs2:
    p = doc.add_paragraph()
    p.add_run(f'[{rid}] [{pri}] ').bold = True
    p.add_run(text)

h2('4.3 Live Dispatch Dashboard')
reqs3 = [
    ('FR-DD-001', 'HIGH', 'The Dashboard SHALL display all active technicians on a Google Maps embed, updating positions every 30 seconds.'),
    ('FR-DD-002', 'HIGH', 'SLA At Risk jobs (breach within 90 min) SHALL be highlighted in amber; breached jobs in red, with an audio alert (dismissible).'),
    ('FR-DD-003', 'MEDIUM', 'Drag-and-drop reassignment SHALL complete within 3 seconds including API round-trip and map refresh.'),
    ('FR-DD-004', 'LOW', 'The Dashboard SHALL support exporting a daily dispatch summary to PDF, including job count, FTFR, avg ETA accuracy.'),
    ('FR-DD-005', 'HIGH', 'The Dashboard SHALL be accessible only to users with role Dispatcher or Manager.'),
]
for rid, pri, text in reqs3:
    p = doc.add_paragraph()
    p.add_run(f'[{rid}] [{pri}] ').bold = True
    p.add_run(text)

h1('5. Non-Functional Requirements')
nfrs = [
    ('NFR-PERF-001', 'Auto-assign API p99 latency < 500ms under 200 concurrent requests. Load test gate before production release.'),
    ('NFR-PERF-002', 'Location ping API p95 latency < 100ms. Throughput target: 5,000 pings/min at peak.'),
    ('NFR-AVAIL-001', 'Location service and dispatch engine: 99.9% monthly uptime SLA (≤44 min downtime/month).'),
    ('NFR-SEC-001', 'All new endpoints must pass OWASP Top 10 security scan. No HIGH/CRITICAL findings before go-live.'),
    ('NFR-SEC-002', 'Google Maps API key must be restricted by IP allowlist (EKS node CIDR) and API scope (Routes, Distance Matrix only).'),
    ('NFR-COST-001', 'Google Maps Platform spend must not exceed $3,000/month. CloudWatch alarm at $2,500 triggers P2 alert to Engineering and Finance.'),
    ('NFR-SCALE-001', 'System must handle 2,000 concurrent active technicians without degradation. Horizontal scaling via Kubernetes HPA.'),
    ('NFR-DATA-001', 'technician_locations table partitioned monthly. Partition older than 30 days dropped via scheduled pg_cron job.'),
]
for nid, text in nfrs:
    p = doc.add_paragraph()
    p.add_run(f'[{nid}] ').bold = True
    p.add_run(text)

h1('6. Constraints')
constraints = [
    'Technology: Backend services must remain on AWS (EKS). No new cloud providers.',
    'Database: PostgreSQL 15 only. No NoSQL stores for transactional data.',
    'Mobile: React Native (existing codebase). No platform rewrites.',
    'Third-party: Only Google Maps Platform approved by Legal/Procurement. Apple Maps and HERE rejected (licensing cost).',
    'Budget: Total project budget $480,000. Third-party API budget $3,000/month ongoing.',
    'Timeline: Go-live by 2026-10-15 for Q4 field season. Hard deadline — backed by customer contract.',
    'Compliance: GDPR Article 5(1)(e): location data minimisation. 30-day retention strictly enforced. DPA amendment required before go-live.',
    'Security: SOC 2 Type II audit scheduled 2026-11-01. All new services must be included in audit scope.',
]
for c in constraints:
    bullet(c)

h1('7. Integration Points')
table2 = doc.add_table(rows=1, cols=4)
table2.style = 'Light Grid Accent 1'
hdr2 = table2.rows[0].cells
for i, h in enumerate(['System', 'Direction', 'Protocol', 'Notes']):
    hdr2[i].text = h
integrations = [
    ('Google Maps Routes API', 'Outbound', 'HTTPS REST', 'ETA + route optimisation'),
    ('Google Maps Distance Matrix', 'Outbound', 'HTTPS REST', 'Batch PM clustering'),
    ('Kafka (fieldops cluster)', 'Publish/Subscribe', 'Kafka 3.6', 'Location events, dispatch events'),
    ('Existing WorkOrderService', 'Internal', 'gRPC', 'Score query + assignment update'),
    ('AWS Secrets Manager', 'Outbound', 'AWS SDK', 'Google Maps API key'),
    ('AWS CloudWatch', 'Outbound', 'AWS SDK', 'Cost + latency alarms'),
    ('Mobile App (React Native)', 'Inbound', 'HTTPS REST', 'Location pings'),
    ('SCIM (user directory)', 'Inbound', 'Webhook', 'Technician provisioning/deprovisioning'),
]
for row in integrations:
    r = table2.add_row().cells
    for i, v in enumerate(row):
        r[i].text = v
doc.add_paragraph()

h1('8. Data Model Summary')
para(
    'Primary key strategy: UUID v4 for all new tables. '
    'Foreign key enforcement: enabled (existing pattern). '
    'Audit columns: all new tables include created_at TIMESTAMPTZ NOT NULL DEFAULT now(). '
    'dispatch_rules.conditions and .actions columns use JSONB to store JSONLogic rule expressions — '
    'validated at insert/update via a CHECK constraint calling a PL/pgSQL validator function. '
    'route_optimisations.job_ids is a PostgreSQL UUID array; ordering is significant (index 0 = first stop).'
)

h1('9. Open Questions')
oqs = [
    ('OQ-001', 'OPEN', 'What is the fallback behaviour when a technician\'s device battery dies mid-job? Does the last known location persist for ETA until job is manually completed?'),
    ('OQ-002', 'OPEN', 'Google Maps API rate limit is 3,000 RPM for Routes API. At peak (2,000 techs × every 90 sec = ~1,333 RPM) we are within limits. But batch PM run at 06:00 UTC could spike. Confirm batch size and stagger interval.'),
    ('OQ-003', 'OPEN', 'dispatch_rules JSONLogic expressions — who authors them? Is there a UI? Product has not scoped a rule-builder UI for v2.0.'),
    ('OQ-004', 'OPEN', 'SLA preemption rule (FR-AD-002) requires notifying the customer of the in-progress job about a potential delay. Is this in scope? Customer comms team not yet consulted.'),
    ('OQ-005', 'OPEN', 'GDPR DPA amendment: Legal confirmed amendment needed but has not confirmed timeline. If DPA not signed by 2026-08-01, location tracking must be disabled at go-live.'),
    ('OQ-006', 'OPEN', 'Offline mode: out of scope for v2.0, but mobile team flagged that without offline support, technicians in low-connectivity areas cannot use the app. Risk to adoption — needs formal decision.'),
]
for oid, status, text in oqs:
    p = doc.add_paragraph()
    p.add_run(f'[{oid}] [{status}] ').bold = True
    p.add_run(text)

h1('10. Risks')
risks = [
    ('R-001', 'HIGH', 'Google Maps API cost overrun if batch PM run is not throttled. Mitigation: CloudWatch alarm at $2,500; auto-disable batch job if alarm fires.'),
    ('R-002', 'HIGH', 'GDPR DPA amendment delay blocks location tracking feature. Mitigation: parallel-track legal review; build feature-flag to disable location at deploy time.'),
    ('R-003', 'MEDIUM', 'Kafka cluster capacity: current cluster handles 50k msg/min. At 2,000 techs × 2 msg/min = 4,000 msg/min — well within limit. Risk is if fleet grows 10x. Mitigation: monitor lag, scale partitions proactively.'),
    ('R-004', 'MEDIUM', 'SLA preemption (FR-AD-002) may cause customer dissatisfaction for the interrupted job. Mitigation: limit preemption to Critical-SLA jobs only; add customer notification (OQ-004).'),
    ('R-005', 'LOW', 'Drag-and-drop on Dashboard requires modern browser. IE11 not supported. Mitigation: already dropped IE11 support in v1.4.'),
]
for rid, sev, text in risks:
    p = doc.add_paragraph()
    p.add_run(f'[{rid}] [{sev}] ').bold = True
    p.add_run(text)

doc.save(r'C:\VectorVolt\esip\test-data\FieldOps_Mobile_2.0_PRD.docx')
print('DOCX saved.')
