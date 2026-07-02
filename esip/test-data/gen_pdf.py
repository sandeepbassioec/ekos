"""Generate FieldOps 2.0 Architecture & Technical Spec as PDF"""
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.units import mm
from reportlab.lib import colors
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, HRFlowable
from reportlab.lib.enums import TA_LEFT, TA_CENTER

doc = SimpleDocTemplate(
    r'C:\VectorVolt\esip\test-data\FieldOps_2.0_Technical_Architecture.pdf',
    pagesize=A4,
    leftMargin=20*mm, rightMargin=20*mm,
    topMargin=20*mm, bottomMargin=20*mm
)

styles = getSampleStyleSheet()
H1 = ParagraphStyle('H1', parent=styles['Heading1'], fontSize=16, textColor=colors.HexColor('#1F3864'), spaceAfter=6)
H2 = ParagraphStyle('H2', parent=styles['Heading2'], fontSize=13, textColor=colors.HexColor('#2E5090'), spaceAfter=4)
H3 = ParagraphStyle('H3', parent=styles['Heading3'], fontSize=11, textColor=colors.HexColor('#375623'), spaceAfter=3)
BODY = ParagraphStyle('BODY', parent=styles['Normal'], fontSize=9, leading=14, spaceAfter=4)
CODE = ParagraphStyle('CODE', parent=styles['Normal'], fontSize=8, fontName='Courier', backColor=colors.HexColor('#F5F5F5'), leftIndent=10, spaceAfter=4)
BULLET = ParagraphStyle('BULLET', parent=styles['Normal'], fontSize=9, leading=13, leftIndent=15, bulletIndent=5, spaceAfter=2)
CAPTION = ParagraphStyle('CAPTION', parent=styles['Normal'], fontSize=8, textColor=colors.grey, alignment=TA_CENTER, spaceAfter=6)

def h1(t): return Paragraph(t, H1)
def h2(t): return Paragraph(t, H2)
def h3(t): return Paragraph(t, H3)
def p(t): return Paragraph(t, BODY)
def code(t): return Paragraph(t.replace('\n','<br/>').replace(' ','&nbsp;'), CODE)
def b(t): return Paragraph(f'• {t}', BULLET)
def sp(n=6): return Spacer(1, n*mm)
def hr(): return HRFlowable(width="100%", thickness=0.5, color=colors.HexColor('#CCCCCC'), spaceAfter=4)

tbl_style = TableStyle([
    ('BACKGROUND', (0,0), (-1,0), colors.HexColor('#1F3864')),
    ('TEXTCOLOR', (0,0), (-1,0), colors.white),
    ('FONTNAME', (0,0), (-1,0), 'Helvetica-Bold'),
    ('FONTSIZE', (0,0), (-1,-1), 8),
    ('ROWBACKGROUNDS', (0,1), (-1,-1), [colors.white, colors.HexColor('#EEF2F7')]),
    ('GRID', (0,0), (-1,-1), 0.3, colors.HexColor('#AAAAAA')),
    ('ALIGN', (0,0), (-1,-1), 'LEFT'),
    ('VALIGN', (0,0), (-1,-1), 'TOP'),
    ('PADDING', (0,0), (-1,-1), 4),
    ('WORDWRAP', (0,0), (-1,-1), True),
])

story = []

story += [
    Paragraph('<b>FieldOps Mobile 2.0</b>', ParagraphStyle('TITLE', parent=styles['Title'], fontSize=22, textColor=colors.HexColor('#1F3864'), alignment=TA_CENTER)),
    Paragraph('Technical Architecture &amp; API Specification', ParagraphStyle('SUB', parent=styles['Normal'], fontSize=14, textColor=colors.HexColor('#555555'), alignment=TA_CENTER, spaceAfter=2)),
    Paragraph('Version 0.9 DRAFT | Engineering Review | 2026-06-12', CAPTION),
    hr(), sp(2),
]

story += [h1('1. System Architecture Overview'), p(
    'FieldOps Mobile 2.0 adopts a microservices architecture deployed on AWS EKS (Kubernetes). '
    'The two new services — Technician Location Service (TLS) and Enhanced Dispatch Engine (EDE) — '
    'are isolated deployments communicating via Kafka for async flows and gRPC for synchronous scoring queries. '
    'The existing WorkOrderService is extended with new gRPC endpoints rather than replaced, '
    'preserving backward compatibility with v1 API consumers during the sunset period.'
), sp(2)]

story += [h2('1.1 Service Topology'), p(
    'Mobile App → HTTPS → API Gateway (Kong) → TLS / EDE / WorkOrderService<br/>'
    'TLS → Kafka topic: fieldops.technician-location<br/>'
    'EDE consumes: fieldops.technician-location<br/>'
    'EDE → gRPC → WorkOrderService (assignment update)<br/>'
    'EDE → Google Maps Routes API (HTTPS outbound)<br/>'
    'Dashboard (React Web) → HTTPS/WebSocket → API Gateway → EDE / WorkOrderService'
), sp(2)]

story += [h2('1.2 Technology Stack'), ]
stack = [
    ['Component', 'Technology', 'Version', 'Hosting'],
    ['Technician Location Service', 'Node.js + Express', '20 LTS', 'EKS (2 replicas min)'],
    ['Enhanced Dispatch Engine', 'Java Spring Boot', '3.2', 'EKS (3 replicas min)'],
    ['Existing WorkOrderService', 'Java Spring Boot', '2.7 (upgrade to 3.x in v2.1)', 'EKS'],
    ['Live Dispatch Dashboard', 'React 18 + TypeScript', 'Vite 5', 'S3 + CloudFront'],
    ['Database', 'PostgreSQL 15', 'RDS Multi-AZ', 'AWS RDS'],
    ['Message Bus', 'Apache Kafka', '3.6', 'AWS MSK'],
    ['API Gateway', 'Kong 3.4', 'DB-less mode', 'EKS'],
    ['Maps', 'Google Maps Platform', 'Routes API v2', 'Google Cloud'],
]
story += [Table(stack, colWidths=[50*mm, 45*mm, 35*mm, 35*mm], style=tbl_style), sp(4)]

story += [h1('2. Technician Location Service — API Specification')]
story += [h2('2.1 POST /api/v2/location/ping'), p('Accepts a GPS fix from a field technician\'s mobile device. Must return within 100ms (p95).')]
story += [h3('Request Body (JSON)')]
story += [code(
    '{\n'
    '  "technicianId": "uuid",\n'
    '  "jobId": "uuid | null",\n'
    '  "lat": 51.5074,\n'
    '  "lng": -0.1278,\n'
    '  "accuracyMetres": 8,\n'
    '  "headingDegrees": 180,\n'
    '  "speedKmh": 42.5,\n'
    '  "batteryPct": 78,\n'
    '  "clientTimestamp": "2026-06-12T09:14:32.000Z"\n'
    '}'
)]
story += [h3('Response Body (JSON)')]
story += [code(
    '{\n'
    '  "accepted": true,\n'
    '  "accuracyFlag": false,\n'
    '  "etaMinutesToActiveJob": 12,\n'
    '  "nextPingIntervalSecs": 30\n'
    '}'
)]
story += [p('<b>Error cases:</b>'), b('401 Unauthorized — missing/invalid JWT'), b('422 Unprocessable — lat/lng out of range, batteryPct > 100'), b('429 Too Many Requests — ping rate > 2/min without active job'), sp(2)]

story += [h2('2.2 GET /api/v2/location/fleet'), p(
    'Returns all active technicians in a geographic region. Used by the Live Dispatch Dashboard '
    'map layer. Response is cached for 15 seconds server-side.'
)]
story += [h3('Query Parameters')]
fleet_params = [
    ['Param', 'Type', 'Required', 'Description'],
    ['regionId', 'string', 'Yes', 'Region identifier from the regions reference table'],
    ['includeOffline', 'boolean', 'No (default: false)', 'Include technicians with availability_status=offline'],
    ['maxAgeSeconds', 'integer', 'No (default: 120)', 'Exclude technicians with location older than N seconds'],
]
story += [Table(fleet_params, colWidths=[35*mm, 25*mm, 40*mm, 65*mm], style=tbl_style), sp(2)]

story += [h1('3. Enhanced Dispatch Engine — Scoring Algorithm')]
story += [p(
    'The scoring formula assigns a composite score (0–1000) to each available technician for a given job. '
    'Higher score = better match. The technician with the highest score is auto-assigned.'
)]
story += [h2('3.1 Score Formula')]
story += [code(
    'score = (\n'
    '  skill_match_score     * 0.40  +   # 0-100, from skill_tags vs job.required_skills\n'
    '  proximity_score       * 0.30  +   # 100 - (eta_minutes / max_eta * 100), min 0\n'
    '  queue_capacity_score  * 0.20  +   # (max_queue - current_queue) / max_queue * 100\n'
    '  sla_urgency_bonus     * 0.10      # 100 if critical, 60 if high, 20 if normal\n'
    ') * 10  # scale to 0-1000\n'
    '\n'
    '# Disqualification conditions (score forced to 0):\n'
    '# - availability_status IN (offline, break)\n'
    '# - skill_match_score < 30 AND overrideSkillCheck = false\n'
    '# - current_queue >= max_queue_cap\n'
    '# - active dispatch_rule with type=region_lock blocks technician'
)]
story += [sp(2)]

story += [h2('3.2 SLA Pre-emption Logic')]
story += [p(
    'When a Critical-SLA job arrives, the engine checks all In Progress jobs. '
    'If a technician\'s current job meets ALL of the following, pre-emption is triggered:'
)]
for c in [
    'Current job SLA tier = normal',
    'Current job remaining_time_estimate > 60 minutes',
    'Technician ETA to the Critical job site ≤ 15 minutes',
    'Technician skill_match_score for Critical job ≥ 50',
    'No active dispatch_rule with type=sla_preemption blocking this technician',
]:
    story += [b(c)]
story += [p(
    'Pre-emption action: (1) pause current job (status → Paused), (2) assign Critical job, '
    '(3) re-queue paused job to top of technician queue, (4) emit PreemptionEvent to Kafka '
    '(topic: fieldops.dispatch-events) for customer notification workflow.'
), sp(2)]

story += [h1('4. Security Architecture')]
story += [h2('4.1 Authentication & Authorisation')]
story += [p(
    'All API endpoints are protected by JWT Bearer tokens issued by the existing Auth Service (Auth0). '
    'Scope claims required:'
)]
auth_table = [
    ['Endpoint Group', 'Required Scope', 'Notes'],
    ['POST /api/v2/location/ping', 'fieldops:location:write', 'Mobile app service account'],
    ['GET /api/v2/location/*', 'fieldops:location:read', 'Dashboard + dispatch engine'],
    ['POST /api/v2/fieldops/dispatch/auto-assign', 'fieldops:dispatch:write', 'Dashboard + EDE internal'],
    ['GET /api/v2/fieldops/dispatch/*', 'fieldops:dispatch:read', 'Manager + Dispatcher roles'],
    ['admin/dispatch_rules', 'fieldops:admin', 'Manager role only'],
]
story += [Table(auth_table, colWidths=[50*mm, 45*mm, 70*mm], style=tbl_style), sp(2)]

story += [h2('4.2 Data Privacy')]
story += [p(
    'Location data is classified as Personal Data under GDPR. Controls in place:'
)]
for c in [
    'Encryption at rest: AES-256 (RDS + MSK encryption enabled)',
    'Encryption in transit: TLS 1.3 enforced at Kong gateway (TLS 1.2 rejected)',
    'Data minimisation: lat/lng precision truncated to 5 decimal places (~1m accuracy) in API responses visible to non-technician roles',
    'Retention enforcement: pg_cron job drops monthly partitions of technician_locations older than 30 days — runs 01:00 UTC on 1st of each month',
    'GDPR Article 17 (right to erasure): DELETE endpoint at DELETE /api/v2/location/technician/{techId}/all-history — logs erasure event to compliance audit table',
    'DPA amendment required before go-live — see PRD OQ-005',
]:
    story += [b(c)]
story += [sp(2)]

story += [h1('5. Performance & Scalability Targets')]
perf = [
    ['Metric', 'Target', 'Measurement Method'],
    ['Location ping API p95 latency', '< 100ms', 'CloudWatch custom metric, 1-min resolution'],
    ['Auto-assign API p99 latency', '< 500ms', 'CloudWatch custom metric'],
    ['Scoring engine throughput', '200 concurrent requests', 'k6 load test, CI gate'],
    ['Kafka consumer lag (location topic)', '< 5 seconds at peak', 'MSK consumer group lag alarm'],
    ['Dashboard map refresh latency', '< 2 seconds end-to-end', 'Synthetic monitor (CloudWatch Synthetics)'],
    ['DB query p99 (auto-assign scoring)', '< 50ms', 'pg_stat_statements + RDS Performance Insights'],
    ['Google Maps API error rate', '< 1% over 5 min', 'CloudWatch alarm + circuit breaker trigger'],
]
story += [Table(perf, colWidths=[55*mm, 40*mm, 70*mm], style=tbl_style), sp(2)]

story += [h1('6. Open Technical Questions')]
oqs = [
    ('TQ-001', 'OPEN', 'The scoring algorithm uses a simple weighted sum. Engineering recommends evaluating a lightweight ML model (pre-trained, offline) for better FTFR prediction. Decision needed before Sprint 4.'),
    ('TQ-002', 'OPEN', 'technician_locations table uses DECIMAL for lat/lng. PostGIS GEOGRAPHY type would enable native spatial queries (ST_Distance). Migration effort: ~2 days. Decision: adopt PostGIS or keep DECIMAL?'),
    ('TQ-003', 'OPEN', 'WebSocket vs SSE for Dashboard real-time updates: WebSocket preferred for bi-directional (drag-drop ACK), but Kong 3.4 requires websocket plugin configuration. SSE is simpler but one-way only.'),
    ('TQ-004', 'OPEN', 'Kafka topic retention for fieldops.technician-location: currently defaulting to 7 days. Should be reduced to 24h to limit PII retention. Confirm with Data Privacy team.'),
    ('TQ-005', 'OPEN', 'The EDE pre-emption logic requires knowing remaining_time_estimate for in-progress jobs. This field does not currently exist on work_orders. Does the mobile tech manually update it, or do we estimate via job type average duration?'),
]
for oid, status, text in oqs:
    story += [p(f'<b>[{oid}] [{status}]</b> {text}')]

doc.build(story)
print('PDF saved.')
