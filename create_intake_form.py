from openpyxl import Workbook
from openpyxl.styles import Font, PatternFill, Alignment, Border, Side

wb = Workbook()
ws = wb.active
ws.title = "Intake Form"

DARK = "0D1B2A"
MID_BLUE = "1F4E79"
ACCENT = "2E75B6"
LIGHT_BLUE = "DEEAF1"
LIGHT_GREY = "F5F5F5"
WHITE = "FFFFFF"
GOLD = "C9A634"

thin = Border(
    left=Side(style='thin', color='CCCCCC'),
    right=Side(style='thin', color='CCCCCC'),
    top=Side(style='thin', color='CCCCCC'),
    bottom=Side(style='thin', color='CCCCCC')
)

ws.column_dimensions['A'].width = 38
ws.column_dimensions['B'].width = 19
ws.column_dimensions['C'].width = 17
ws.column_dimensions['D'].width = 22
ws.column_dimensions['E'].width = 24
ws.column_dimensions['F'].width = 18

def merge_style(ws, row, text, bg, font_color=WHITE, size=11, bold=True, height=28):
    ws.merge_cells(f'A{row}:F{row}')
    c = ws[f'A{row}']
    c.value = text
    c.font = Font(name='Arial', bold=bold, size=size, color=font_color)
    c.fill = PatternFill('solid', start_color=bg)
    c.alignment = Alignment(horizontal='left', vertical='center', wrap_text=True, indent=1)
    ws.row_dimensions[row].height = height
    c.border = thin

def qa(ws, row, q, a, alt=False):
    bg = LIGHT_BLUE if alt else WHITE
    cq = ws[f'A{row}']
    cq.value = q
    cq.font = Font(name='Arial', size=9, bold=True)
    cq.fill = PatternFill('solid', start_color=bg)
    cq.alignment = Alignment(horizontal='left', vertical='top', wrap_text=True)
    cq.border = thin

    ws.merge_cells(f'B{row}:F{row}')
    ca = ws[f'B{row}']
    ca.value = a
    ca.font = Font(name='Arial', size=9)
    ca.fill = PatternFill('solid', start_color=bg)
    ca.alignment = Alignment(horizontal='left', vertical='top', wrap_text=True)
    ca.border = thin
    ws.row_dimensions[row].height = max(32, min(130, len(str(a)) // 2))

r = 1
# ── Title ──────────────────────────────────────────────────────────────
ws.merge_cells(f'A{r}:F{r}')
t = ws[f'A{r}']
t.value = "ARCHITECT INTAKE FORM  —  Getting to Know You"
t.font = Font(name='Arial', bold=True, size=15, color=WHITE)
t.fill = PatternFill('solid', start_color=DARK)
t.alignment = Alignment(horizontal='center', vertical='center')
ws.row_dimensions[r].height = 36; t.border = thin; r += 1

ws.merge_cells(f'A{r}:F{r}')
sub = ws[f'A{r}']
sub.value = "Please fill this out before your first 1:1 with the new Director of Engineering. Your answers help them understand your world and how to best support you."
sub.font = Font(name='Arial', italic=True, size=9, color='555555')
sub.fill = PatternFill('solid', start_color='EEF2F7')
sub.alignment = Alignment(horizontal='left', vertical='center', wrap_text=True, indent=1)
ws.row_dimensions[r].height = 22; sub.border = thin; r += 2

# ── SECTION A ──────────────────────────────────────────────────────────
merge_style(ws, r, "SECTION A — ABOUT YOU", MID_BLUE); r += 1
data_a = [
    ("Your Full Name", "Sandeep Bassi"),
    ("Your Title / Level", "Architect"),
    ("Your Team / Domain", "ReparLink"),
    ("Who Do You Report To?", "Ravi Ivaturi"),
    ("How Long Have You Been at the Company?", "5 months"),
    ("How Long Have You Been in an Architecture Role?", "5 months"),
    ("Your Location / Time Zone", "IST (India Standard Time)"),
    ("Your GitHub or Confluence Handle", "sandeep.bassi@oeconnection.com"),
    ("Date You Are Filling This Out", "10/05/2026"),
]
for i, (q, a) in enumerate(data_a): qa(ws, r, q, a, i%2==0); r += 1
r += 1

# ── SECTION B ──────────────────────────────────────────────────────────
merge_style(ws, r, "SECTION B — YOUR TECHNICAL STRATEGY & ARCHITECTURE VISION", MID_BLUE); r += 1
data_b = [
    ("What is your current architecture vision for the next 12–18 months?\n💡 What are you trying to build toward?",
     "The current architecture vision is to migrate DEV, QA, and PROD environments to AWS, and thereafter focus all development on modernisation of the legacy ReparLink applications — moving from tightly coupled, stored-procedure-heavy code to a CLEAN architecture using .NET Core, Entity Framework, and cloud-native AWS services."),
    ("What are the 3 biggest architectural risks in the system today?\n💡 Be specific — what keeps you up at night?",
     "1. Tight Coupling — business logic is deeply embedded in stored procedures and service layers, making change dangerous and slow.\n2. Old Technology — .NET Remoting, asmx, and ServiceMix-era patterns not compatible with modern .NET Core runtimes.\n3. No Disaster Recovery — the system has no formalised DR strategy despite repeated recommendations. A production failure today would mean significant unplanned recovery time."),
    ("Where is the most significant technical debt, and what is your plan to address it?\n💡 Which areas are most fragile or slowing the team down?",
     "Business logic exists in stored procedures. We have initiated deprecation and are migrating logic to C# using LINQ and Entity Framework. Large volumes of code remain. The plan is to use AWS Transform Custom with a dedicated team allocated specifically to this backlog."),
    ("How do you decide when to refactor vs. rewrite vs. tolerate technical debt?\n💡 Walk us through your decision framework.",
     "Most decisions default to refactor, and even new product features are being built on the new stack. We tolerate technical debt for minor fixes or small feature requests with no architectural impact.\n\nFor RL TOS, the decision was made to rewrite entirely — ServiceMix was causing noisy-neighbour effects, availability failures, and resilience issues. We rewrote in .NET Core 8 on AWS. That call was validated by the significant improvement in reliability post-launch."),
    ("What architectural decisions are currently in flight (open ADRs)?\n💡 What decisions are you actively working through right now?",
     "Current ADRs are primarily focused on the modernisation journey. I am creating sub-pages under each design document on Confluence, with each sub-page corresponding to one ADR, so decisions are traceable, individually linkable, and communicable to engineering teams."),
    ("What is your view on the current microservices / monolith / platform architecture?\n💡 What is working well? What needs to change?",
     "The current codebase is a mixed blend of legacy services (asmx, .NET Remoting) and modernised CLEAN architecture components. Some platform services are shared between ReparLink and CollisionLink and are owned by the Enterprise team. The direction is toward decoupled, cloud-native microservices, but we are in a managed transition rather than a clean-slate rewrite."),
    ("How are architectural decisions documented and communicated to engineering teams?\n💡 e.g. ADRs, Confluence, RFCs, architecture review boards",
     "Architectural decisions are covered in design documents on Confluence. Sessions were planned to fully hand over context to developers. I am now creating sub-pages per design document — each mapping to one ADR — to make decisions searchable and version-traceable."),
    ("What is your view on the current Internal Developer Platform (IDP) maturity?\n💡 What is the gap between where it is and where it needs to be?",
     "The IDP is at an early stage. The gap between current state and the desired state (self-service provisioning, standardised CI/CD pipelines, developer autonomy) is significant. As the AWS migration matures, there will be an opportunity to formalise platform capabilities — but this requires investment in platform engineering that is not yet prioritised."),
    ("What is your AI / ML infrastructure strategy?\n💡 Where is it today, and where does it need to be in 12 months?",
     "Today, teams are using GitHub Copilot and Claude Code for coding productivity. I am leading POC work as part of the Chennai AI initiative: (1) a Parts Recommendation Engine, (2) a Parts Procurement Engine, and (3) an InApp Chat feature using LLM-backed RAG to handle shop and dealer queries. A RAG pipeline is being prototyped over a 43-table Honda parts catalog. In 12 months, the goal is to move at least one initiative from POC to a production-ready architecture."),
    ("How do you evaluate and adopt new technologies?\n💡 Walk us through a recent technology decision you made.",
     "At OEC so far, I have been focused on delivering against the product roadmap. Technology evaluations are based on fit with the AWS ecosystem, .NET compatibility, community maturity, and security posture. A current example is the evaluation of pgvector vs. OpenSearch k-NN for the AI RAG pipeline in CollisionLink Impact."),
    ("What is your approach to cloud cost optimisation and FinOps?",
     "The approach is to minimise unnecessary database load — for example, by incorporating caching layers — which reduces compute and I/O costs without compromising availability. I want to be direct: DR has been recommended and rejected multiple times. Cost efficiency must not be achieved at the expense of resilience, and I would like this conversation to be revisited."),
    ("What is your security architecture posture? Where are the gaps?",
     "My focus is on OWASP Top 10 compliance and the STRIDE threat model when designing new components or reviewing existing ones. The biggest current gaps are: (1) no formalised DR/BCP strategy; (2) no security review gate in the development lifecycle; (3) limited penetration testing cadence. These are known risks."),
    ("What is the biggest architectural decision you wish had been made differently?\n💡 What would you do differently with the benefit of hindsight?",
     "Within my 5 months at OEC, no decision stands out as one I would reverse. The RL TOS rewrite was the right call and the modernisation direction is sound. The main frustration is the rejected DR recommendation — not a decision I made, but one I wish had been accepted."),
]
for i, (q, a) in enumerate(data_b): qa(ws, r, q, a, i%2==0); r += 1
r += 1

# ── SECTION C ──────────────────────────────────────────────────────────
merge_style(ws, r, "SECTION C — SYSTEMS & SERVICES YOU OWN OR INFLUENCE", MID_BLUE); r += 1

col_headers_c = ["System / Service Name", "Your Ownership Level", "Tier / Criticality", "Biggest Current Risk", "Notes / Context"]
for col_idx, (col_letter, hdr) in enumerate(zip(['A','B','C','D','E'], col_headers_c)):
    c = ws[f'{col_letter}{r}']
    c.value = hdr
    c.font = Font(name='Arial', bold=True, size=9, color=WHITE)
    c.fill = PatternFill('solid', start_color=ACCENT)
    c.alignment = Alignment(horizontal='center', vertical='center', wrap_text=True)
    c.border = thin
ws[f'F{r}'].fill = PatternFill('solid', start_color=ACCENT); ws[f'F{r}'].border = thin
ws.row_dimensions[r].height = 22; r += 1

systems = [
    ("ReparLink Core Application", "Primary Owner", "Tier 1 — Critical",
     "Business logic locked in stored procedures; tight coupling between layers",
     "Legacy .NET with asmx and .NET Remoting; active modernisation to .NET Core + EF Core + CLEAN architecture"),
    ("RL TOS (Transaction Ordering Service)", "Primary Owner", "Tier 1 — Critical",
     "Ongoing operational monitoring post-rewrite",
     "Rewritten from ServiceMix to .NET Core 8 on AWS; resolved noisy-neighbour, availability, and resilience issues"),
    ("CollisionLink Impact", "Co-Owner / Lead Architect", "Tier 2 — High",
     "RAG pipeline accuracy and parts data freshness",
     "RN mobile (Expo Go) + .NET 10 + AWS; AI-powered parts procurement over Honda 43-table catalog; active POC"),
    ("Enterprise Platform Services", "Architectural Influence", "Tier 1 — Critical",
     "Cross-team coordination overhead",
     "Shared services consumed by both ReparLink and CollisionLink; owned by Enterprise team"),
    ("ReparLink Modernisation Pipeline", "Primary Owner", "Tier 2 — High",
     "Dependency on team allocation for AWS Transform Custom",
     "Ongoing migration effort; requires at least 1 dedicated developer; primary vehicle for technical debt reduction"),
]
for i, row_data in enumerate(systems):
    bg = LIGHT_GREY if i%2==0 else WHITE
    for col, val in zip(['A','B','C','D','E'], row_data):
        c = ws[f'{col}{r}']
        c.value = val; c.font = Font(name='Arial', size=9)
        c.fill = PatternFill('solid', start_color=bg)
        c.alignment = Alignment(horizontal='left', vertical='top', wrap_text=True)
        c.border = thin
    ws[f'F{r}'].fill = PatternFill('solid', start_color=bg); ws[f'F{r}'].border = thin
    ws.row_dimensions[r].height = 42; r += 1
r += 1

# ── SECTION D ──────────────────────────────────────────────────────────
merge_style(ws, r, "SECTION D — CROSS-TEAM DEPENDENCIES & STAKEHOLDERS", MID_BLUE); r += 1
data_d = [
    ("Which teams depend most on your architectural decisions?",
     "Greasemonks\nMechdroITs"),
    ("Where are the most critical cross-team dependencies today?\n💡 What would break if those dependencies were not managed well?",
     "In the last 5 months, work has been largely independent. The most significant potential dependency point is the Enterprise Platform Services team, whose shared services underpin both RL and CL. Misalignment there would break features in both products."),
    ("Which stakeholders (product, security, legal, finance) do you work with most closely?",
     "Product is the primary stakeholder. Security reviews are applied during design, but there is no formal security review gate in the process today."),
    ("Where do you experience the most friction with other teams or stakeholders?\n💡 Be candid — friction points are important to understand early.",
     "No significant friction to report at this stage. Product timelines can be slow, which creates lead-time pressure on architectural planning, but this has been managed without escalation."),
    ("How do you handle architectural disagreements with engineering managers or tech leads?\n💡 Walk us through how you typically navigate technical disagreements.",
     "We discuss, share each other's concerns, and arrive at a conclusion backed by facts and data. I prefer structured technical conversations with documented trade-offs over informal alignment."),
    ("How do you communicate architectural changes to teams who will be impacted?",
     "By engaging them in meetings, communicating with the Engineering Manager, and allocating relevant work in their respective Sprints. Confluence sub-pages per ADR are being built to formalise this communication channel."),
    ("What is your working relationship with the Product organisation like?\n💡 What works well? What could be better?",
     "Generally positive. Product is engaged and aligned on the modernisation direction. The main friction is pace — product decisions and prioritisation can be slow, which creates scheduling uncertainty for architectural work."),
]
for i, (q, a) in enumerate(data_d): qa(ws, r, q, a, i%2==0); r += 1
r += 1

# ── SECTION E ──────────────────────────────────────────────────────────
merge_style(ws, r, "SECTION E — YOUR TOP ARCHITECTURE RISKS", MID_BLUE); r += 1

col_headers_e = ["Risk Description", "Severity", "Systems Impacted", "Your Mitigation Plan", "What Support Do You Need?"]
for col_letter, hdr in zip(['A','B','C','D','E'], col_headers_e):
    c = ws[f'{col_letter}{r}']
    c.value = hdr; c.font = Font(name='Arial', bold=True, size=9, color=WHITE)
    c.fill = PatternFill('solid', start_color=ACCENT)
    c.alignment = Alignment(horizontal='center', vertical='center', wrap_text=True)
    c.border = thin
ws[f'F{r}'].fill = PatternFill('solid', start_color=ACCENT); ws[f'F{r}'].border = thin
ws.row_dimensions[r].height = 22; r += 1

risks = [
    ("Business Logic in Stored Procedures", "High", "All Systems",
     "Migrate using AWS Transform Custom; incrementally rewrite stored procedure logic to C# / EF Core",
     "Team allocation — at least 1 dedicated developer"),
    ("Old Technology Stack (.NET Remoting, asmx)", "High", "ReparLink Core",
     ".NET Core cannot support .NET Remoting natively; prioritise rewrite of affected components",
     "Same as above — dedicated developer resource"),
    ("No Disaster Recovery Strategy", "High", "All Production Systems",
     "DR strategy documented and presented to leadership; pending approval. Request re-evaluation.",
     "Management escalation and budget approval for DR infrastructure"),
    ("Knowledge Concentration Risk", "Medium", "ReparLink Core",
     "Sub-page ADRs on Confluence being created; team knowledge-transfer sessions already initiated",
     "Time allocation for documentation sprints"),
    ("AI/ML Pipeline Maturity", "Medium", "CollisionLink Impact / RL AI Features",
     "Structured POC delivery with defined success criteria; production path defined post-POC",
     "Product alignment on AI roadmap and dedicated sprint capacity"),
]
for i, row_data in enumerate(risks):
    bg = LIGHT_GREY if i%2==0 else WHITE
    for col, val in zip(['A','B','C','D','E'], row_data):
        c = ws[f'{col}{r}']
        c.value = val; c.font = Font(name='Arial', size=9)
        c.fill = PatternFill('solid', start_color=bg)
        c.alignment = Alignment(horizontal='left', vertical='top', wrap_text=True)
        c.border = thin
    ws[f'F{r}'].fill = PatternFill('solid', start_color=bg); ws[f'F{r}'].border = thin
    ws.row_dimensions[r].height = 44; r += 1
r += 1

# ── SECTION F ──────────────────────────────────────────────────────────
merge_style(ws, r, "SECTION F — AI ADOPTION & INNOVATION", MID_BLUE); r += 1
data_f = [
    ("What AI/ML capabilities are currently embedded in the product architecture?",
     "Currently at POC stage. Three initiatives are underway:\n1. Parts Recommendation Engine — suggests relevant parts based on repair history and vehicle data.\n2. Parts Procurement Engine — streamlines sourcing workflows.\n3. InApp Chat — LLM-backed RAG feature to handle shop and dealer queries conversationally.\nA RAG pipeline has been prototyped over a 43-table Honda parts catalog as part of CollisionLink Impact."),
    ("What AI developer tools are teams using? Are you tracking the productivity impact?",
     "GitHub Copilot and Claude Code are actively used across the team for day-to-day coding. Productivity impact is not formally tracked yet, but anecdotal feedback is positive. I am leading AI capability POCs as part of the Chennai AI initiative."),
    ("Where do you see the biggest AI-driven architectural opportunity in the next 12 months?",
     "The highest-value opportunity is the InApp Chat feature — enabling shops and dealers to resolve queries through a conversational interface backed by structured parts data and repair knowledge. This reduces support overhead and improves time-to-repair. Second priority is the Parts Recommendation Engine, which can drive measurable improvements in procurement accuracy and revenue per transaction."),
    ("What is your view on LLM integration in the product stack? Where are the risks?",
     "LLM integration is a strategic priority but must be approached carefully. Key risks:\n1. Hallucination in parts recommendations — incorrect part suggestions have direct safety and cost implications.\n2. Data privacy — repair records and dealer data are sensitive and must not be logged to third-party LLM providers without proper data governance.\n3. Latency — LLM inference latency must be managed through caching and async patterns to avoid degrading existing UX.\nMitigation strategy: RAG over controlled, versioned data stores rather than fine-tuning on live production data."),
    ("What is your data architecture strategy for AI/ML feature stores and pipelines?",
     "Short-term: lightweight RAG layer over existing relational data (starting with the Honda parts catalog in PostgreSQL/RDS) using structured prompt pipelines.\nMedium-term (12 months): introduce a vector store (pgvector or OpenSearch k-NN) for semantic search over repair notes and parts descriptions.\nFeature store formalisation will follow once POC results justify the investment. Data freshness and lineage are first-class concerns given the safety-critical nature of repair parts data."),
]
for i, (q, a) in enumerate(data_f): qa(ws, r, q, a, i%2==0); r += 1
r += 1

# ── PERSONAL ──────────────────────────────────────────────────────────
merge_style(ws, r, "PERSONAL WORKING PREFERENCES  (Confidential — shared only with your Director)", DARK, height=30); r += 1

def sub_sec(ws, r, text):
    merge_style(ws, r, text, ACCENT, size=9, height=20)

sub_sec(ws, r, "How You Like to Work"); r += 1
data_pw = [
    ("What is your preferred format for a 1:1 with your manager?",
     "A mix of both — I like to come prepared with 2–3 talking points but leave room for open conversation when needed."),
    ("What day and time works best for your regular 1:1?",
     "Tuesday or Wednesday mornings IST. Please avoid Mondays. This aligns with US afternoon (previous day) for async preparation."),
    ("How long do you prefer your 1:1s to be?",
     "30 minutes weekly, with flexibility to extend to 45 minutes for design reviews or career conversations."),
    ("What is your preferred communication channel for day-to-day messages?",
     "Teams for async day-to-day; email for formal decisions and cross-team alignment. Async is fine for most things — I do not expect real-time responses outside working hours."),
    ("How quickly do you typically respond to messages, and what do you expect in return?",
     "Same working day for non-urgent items. Within 2 hours for urgent or blocking issues during IST business hours. I extend the same expectation in return."),
    ("What are your deep-work hours when you prefer not to be interrupted?",
     "10am–1pm IST. I prefer to protect this block for design, architecture, and complex problem-solving."),
    ("What is your remote / in-office preference?",
     "Primarily remote. Available for in-office collaboration for important team sessions or architecture reviews when needed."),
    ("How many meetings per day feels like too many for you?",
     "More than 3–4 back-to-back meetings significantly reduces my effectiveness. I prefer mornings clear for deep focus work and afternoons for collaborative discussions."),
]
for i, (q, a) in enumerate(data_pw): qa(ws, r, q, a, i%2==0); r += 1
r += 1

sub_sec(ws, r, "How You Like to Receive and Give Feedback"); r += 1
data_fb = [
    ("How do you prefer to receive feedback from your manager?",
     "Direct and specific, ideally in our 1:1 first. I prefer written context before a conversation so I have time to reflect. Public criticism is demotivating; private, candid feedback is always welcome."),
    ("How do you prefer to give feedback to your manager?",
     "I will raise concerns openly in our 1:1 or async via Teams. I prefer to bring data and specific examples rather than generalisations."),
    ("What kind of recognition means the most to you?",
     "Peer and manager acknowledgment for technically complex decisions means a lot. Public visibility in team forums is appreciated, especially for cross-team architectural work."),
    ("How do you typically handle disagreements with colleagues or your manager?",
     "I address disagreements directly but constructively — I prefer to bring data and alternatives to the table rather than just objections. I will escalate only when alignment cannot be reached at the working level."),
]
for i, (q, a) in enumerate(data_fb): qa(ws, r, q, a, i%2==0); r += 1
r += 1

sub_sec(ws, r, "What Motivates and Energises You"); r += 1
data_mo = [
    ("What energises you most at work?",
     "Solving hard architectural problems from first principles. Building systems that handle scale and complexity elegantly. Seeing modernisation work pay off in measurable reliability or performance improvements."),
    ("What drains your energy or frustrates you most at work?",
     "Unclear requirements that require rework. Decisions made without architectural input that create downstream technical debt. Slow approvals on critical infrastructure changes (e.g. DR strategy rejected despite repeated recommendations)."),
    ("Describe what a great week at work looks like for you.",
     "Uninterrupted architecture design time, meaningful progress on modernisation, a well-structured 1:1, and seeing a feature or POC reach a milestone. Leaving Friday with something shipped or unblocked."),
    ("Describe what a terrible week at work looks like for you.",
     "Multiple high-priority context switches, requirements changing mid-sprint, and recommendations — like DR — being rejected without a clear rationale or a credible alternative plan."),
    ("What motivates you beyond the work itself?",
     "Career growth toward Principal/Staff Architect. Building systems with real user impact. Continuous learning — especially in distributed systems and AI/ML architecture."),
    ("What is your proudest professional achievement so far?",
     "Leading the complete rewrite of RL TOS — migrating from a fragile ServiceMix-based service to a resilient, cloud-native .NET Core 8 deployment on AWS. This resolved long-standing availability, noisy-neighbour, and resilience issues and is now a reference architecture for future modernisation work at OEC."),
    ("What are you most excited about in your current role right now?",
     "The AI POC initiatives — particularly the InApp Chat and Parts Recommendation features. These have the potential to fundamentally change how shops and dealers interact with the platform, and I am well-positioned to drive the architectural direction."),
    ("Is there anything about your current role or team that concerns you?",
     "DR remains unaddressed despite repeated recommendations. If a production incident occurs before DR is in place, recovery time will be significant. I would like this to be re-evaluated with the new Director as an early agenda item."),
]
for i, (q, a) in enumerate(data_mo): qa(ws, r, q, a, i%2==0); r += 1
r += 1

sub_sec(ws, r, "Your Career Goals and Growth"); r += 1
data_cg = [
    ("Where would you like to be in your career in 2 years?",
     "Staff Architect or Principal Architect, with broader influence across the platform architecture and ownership of the AI/ML architectural strategy at OEC."),
    ("Where would you like to be in 5 years?",
     "Principal Architect or Distinguished Engineer, with a track record of large-scale modernisation and AI-native platform design. Open to a technical leadership track if the opportunity arises."),
    ("What skills are you most focused on developing right now?",
     "AI/ML architecture and LLM integration patterns; distributed systems design at scale; cloud-native architecture on AWS; architectural communication and ADR practices."),
    ("What are you strongest at today?",
     "Systems thinking, legacy modernisation strategy, cloud migration architecture, security architecture (STRIDE / OWASP), and translating complex technical decisions into clear engineering guidance for development teams."),
    ("What do you feel least confident in?",
     "Formal stakeholder communication and executive-level narrative building. I am more comfortable in deep technical forums than in influencing business stakeholders on prioritisation decisions."),
    ("How do you prefer to learn?",
     "Learning by doing — hands-on POCs and side projects. Supplemented by deep reading and structured experimentation. Pair design sessions with senior peers are highly effective."),
    ("Have you had a great mentor or manager in the past? What did they do that worked?",
     "[Please personalise — confidential for your Director]"),
    ("What has a past manager done that you did NOT like or found unhelpful?",
     "[Please personalise — confidential for your Director]"),
    ("What does a great manager do for you?",
     "Shields the team from context-switching. Advocates for technical recommendations (like DR) at the leadership level. Provides growth opportunities through stretch assignments. Gives direct and honest feedback. Treats architecture input as a meaningful voice in product decisions."),
    ("How would you rate your promotion readiness right now?",
     "6–12 months away, depending on scope expansion and visibility of architectural impact across the platform."),
]
for i, (q, a) in enumerate(data_cg): qa(ws, r, q, a, i%2==0); r += 1
r += 1

sub_sec(ws, r, "Psychological Safety and Team Culture"); r += 1
data_ps = [
    ("On a scale of 1–5, how safe do you feel raising bad news or mistakes early? (1=not safe, 5=very safe)",
     "4 / 5"),
    ("On a scale of 1–5, do you feel your ideas and opinions are heard and valued? (1=rarely, 5=always)",
     "3 / 5 — Technical recommendations are generally heard, but not always acted upon (see: DR strategy repeatedly rejected)."),
    ("What does psychological safety mean to you personally?",
     "It means I can raise a concern — like the DR gap — without it being dismissed, and that technical recommendations are treated as genuine inputs to decisions rather than noise. It also means I can admit uncertainty without it being perceived as incompetence."),
    ("What one change would most improve the team culture?",
     "Create a lightweight architecture review process so that significant technical decisions — especially cross-team ones — go through a structured forum rather than ad hoc chat threads. This would improve decision quality, team alignment, and architectural accountability."),
]
for i, (q, a) in enumerate(data_ps): qa(ws, r, q, a, i%2==0); r += 1
r += 1

sub_sec(ws, r, "Life Context and Wellbeing  (Share only what you are comfortable with)"); r += 1
data_wb = [
    ("Are there any personal commitments or constraints that affect your work schedule?",
     "[Please personalise — share only what you're comfortable with]"),
    ("What work-life balance boundaries are important to you?",
     "I maintain clear boundaries during non-IST business hours. Generally not available for messages after 7pm IST or on weekends unless there is a production incident."),
    ("How would you rate your current stress level? (1=very low, 5=very high)",
     "[Please fill in: 1–5]"),
    ("Is there anything else you would like your Director to know about you?",
     "I am deeply invested in the modernisation roadmap and the AI initiatives. I work best when given clear goals and autonomy on the 'how'. I am happy to go above and beyond when I feel the work has real architectural impact."),
    ("How do you prefer your manager to check in on your wellbeing?",
     "Ask me directly in our 1:1. I will be honest if something is off."),
]
for i, (q, a) in enumerate(data_wb): qa(ws, r, q, a, i%2==0); r += 1

# Footer
r += 1
ws.merge_cells(f'A{r}:F{r}')
ft = ws[f'A{r}']
ft.value = "Prepared by Sandeep Bassi  |  ReparLink Architecture  |  May 2026"
ft.font = Font(name='Arial', italic=True, size=8, color='777777')
ft.fill = PatternFill('solid', start_color='EEF2F7')
ft.alignment = Alignment(horizontal='center', vertical='center')
ws.row_dimensions[r].height = 18

# Freeze top rows
ws.freeze_panes = 'A3'

out = r'C:\VectorVolt\Sandeep_Bassi_Architect_Intake_Form.xlsx'
wb.save(out)
print(f"Saved: {out}")
