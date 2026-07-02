"""Generate FieldOps Mobile 2.0 supporting data XLSX — 4 sheets"""
import openpyxl
from openpyxl.styles import Font, PatternFill, Alignment, Border, Side
from openpyxl.utils import get_column_letter

wb = openpyxl.Workbook()

HEADER_FILL = PatternFill("solid", fgColor="1F3864")
HEADER_FONT = Font(color="FFFFFF", bold=True, size=10)
ALT_FILL = PatternFill("solid", fgColor="EEF2F7")

def style_header(ws, row=1):
    for cell in ws[row]:
        cell.fill = HEADER_FILL
        cell.font = HEADER_FONT
        cell.alignment = Alignment(horizontal="center", wrap_text=True)

def auto_width(ws):
    for col in ws.columns:
        max_len = max((len(str(c.value or '')) for c in col), default=8)
        ws.column_dimensions[get_column_letter(col[0].column)].width = min(max_len + 4, 40)

# ── Sheet 1: API Cost Model ──────────────────────────────────────────────────
ws1 = wb.active
ws1.title = "API Cost Model"
ws1.append(["Use Case", "API", "Calls / Technician / Day", "Active Technicians", "Total Calls / Day", "Cost per 1000 Calls (USD)", "Daily Cost (USD)", "Monthly Cost (USD)"])
style_header(ws1)

rows1 = [
    ("Real-time ETA (90-sec cycle, 8h shift)", "Routes API", 320, 2000, "=C3*D3", 0.01, "=E3*F3/1000", "=G3*22"),
    ("Route optimise on new job assign", "Routes API", 12, 2000, "=C4*D4", 0.01, "=E4*F4/1000", "=G4*22"),
    ("Daily PM batch clustering (06:00 UTC)", "Distance Matrix", 1, 2000, "=C5*D5", 0.005, "=E5*F5/1000", "=G5*22"),
    ("On-demand auto-assign calls", "Routes API", 5, 2000, "=C6*D6", 0.01, "=E6*F6/1000", "=G6*22"),
]
for i, r in enumerate(rows1, start=3):
    ws1.append(r)

ws1.append([])
ws1.append(["", "", "", "", "", "TOTAL MONTHLY (USD)", "", "=H3+H4+H5+H6"])
ws1["A9"].font = Font(bold=True)
ws1["F9"].font = Font(bold=True, color="C00000")
ws1["H9"].font = Font(bold=True, color="C00000")
ws1.append(["", "", "", "", "", "BUDGET CAP (USD)", "", 3000])
ws1["F10"].font = Font(bold=True)
ws1["H10"].font = Font(bold=True)
ws1.append(["", "", "", "", "", "HEADROOM (USD)", "", "=H10-H9"])
ws1["F11"].font = Font(bold=True, color="375623")
ws1["H11"].font = Font(bold=True, color="375623")
auto_width(ws1)

# ── Sheet 2: Database Schema Delta ──────────────────────────────────────────
ws2 = wb.create_sheet("DB Schema Delta")
ws2.append(["Table", "Column", "Type", "Nullable", "Default", "Change Type", "Migration Script", "Notes"])
style_header(ws2)
schema = [
    ("work_orders", "assigned_method", "ENUM('manual','auto_engine','forced')", "NO", "manual", "ADD COLUMN", "ALTER TABLE work_orders ADD COLUMN assigned_method ...", "Backfill: UPDATE SET assigned_method='manual'"),
    ("work_orders", "sla_tier", "ENUM('critical','high','normal')", "NO", "normal", "ADD COLUMN", "ALTER TABLE work_orders ADD COLUMN sla_tier ...", "Default covers legacy rows"),
    ("work_orders", "eta_minutes", "INT", "YES", "NULL", "ADD COLUMN", "ALTER TABLE work_orders ADD COLUMN eta_minutes INT", "Populated by location service"),
    ("work_orders", "last_eta_computed_at", "TIMESTAMPTZ", "YES", "NULL", "ADD COLUMN", "ALTER TABLE work_orders ADD COLUMN last_eta_computed_at TIMESTAMPTZ", ""),
    ("work_orders", "dispatch_score_breakdown", "JSONB", "YES", "NULL", "ADD COLUMN", "ALTER TABLE work_orders ADD COLUMN dispatch_score_breakdown JSONB", "Stores per-tech score details"),
    ("technicians", "availability_status", "ENUM('available','on_job','break','offline')", "NO", "offline", "ADD COLUMN", "ALTER TABLE technicians ADD COLUMN availability_status ...", ""),
    ("technicians", "current_location_lat", "DECIMAL(10,8)", "YES", "NULL", "ADD COLUMN", "ALTER TABLE technicians ADD COLUMN current_location_lat DECIMAL(10,8)", "Denorm from location service"),
    ("technicians", "current_location_lng", "DECIMAL(11,8)", "YES", "NULL", "ADD COLUMN", "ALTER TABLE technicians ADD COLUMN current_location_lng DECIMAL(11,8)", ""),
    ("technicians", "location_updated_at", "TIMESTAMPTZ", "YES", "NULL", "ADD COLUMN", "ALTER TABLE technicians ADD COLUMN location_updated_at TIMESTAMPTZ", ""),
    ("technicians", "skill_tags", "TEXT[]", "NO", "ARRAY[]::TEXT[]", "ADD COLUMN", "ALTER TABLE technicians ADD COLUMN skill_tags TEXT[] DEFAULT ARRAY[]::TEXT[]", "Backfill from skills join table"),
    ("technician_locations", "(new table)", "—", "—", "—", "CREATE TABLE", "See PRD Section 3.1.7", "Monthly partitioned"),
    ("route_optimisations", "(new table)", "—", "—", "—", "CREATE TABLE", "See PRD Section 3.1.7", "Max 25 job_ids per row"),
    ("dispatch_rules", "(new table)", "—", "—", "—", "CREATE TABLE", "See PRD Section 3.1.7", "JSONLogic conditions/actions"),
]
for r in schema:
    ws2.append(r)
for i, row in enumerate(ws2.iter_rows(min_row=2), start=2):
    if i % 2 == 0:
        for cell in row:
            cell.fill = ALT_FILL
auto_width(ws2)

# ── Sheet 3: Technician Skill Matrix ────────────────────────────────────────
ws3 = wb.create_sheet("Technician Skill Matrix")
ws3.append(["Technician ID", "Name", "Region", "HVAC", "Electrical", "Plumbing", "Network", "Solar", "Skill Score (0-100)", "Avg FTFR (%)", "Active Jobs Cap", "Current Availability"])
style_header(ws3)
technicians = [
    ("TECH-001", "Alice Mbeki", "North", 90, 70, 40, 20, 85, 81, 78, 4, "Available"),
    ("TECH-002", "Rajan Patel", "North", 60, 95, 30, 88, 10, 79, 82, 3, "On Job"),
    ("TECH-003", "Wei Zhang", "South", 75, 80, 85, 40, 0, 72, 71, 4, "Available"),
    ("TECH-004", "Maria Santos", "South", 40, 60, 95, 30, 0, 65, 69, 3, "Break"),
    ("TECH-005", "John Okafor", "East", 85, 85, 60, 70, 90, 90, 88, 5, "Available"),
    ("TECH-006", "Sara Lindqvist", "East", 30, 40, 20, 95, 50, 59, 74, 2, "On Job"),
    ("TECH-007", "Carlos Rivera", "West", 70, 70, 70, 70, 70, 70, 77, 4, "Available"),
    ("TECH-008", "Amara Diallo", "West", 95, 50, 45, 10, 95, 79, 85, 4, "Offline"),
    ("TECH-009", "Tom Nguyen", "North", 50, 88, 30, 92, 0, 74, 79, 3, "Available"),
    ("TECH-010", "Priya Singh", "South", 80, 65, 80, 55, 80, 78, 81, 4, "Available"),
]
for r in technicians:
    ws3.append(r)

skill_cols = ["D","E","F","G","H"]
green_fill = PatternFill("solid", fgColor="C6EFCE")
red_fill = PatternFill("solid", fgColor="FFC7CE")
for row in ws3.iter_rows(min_row=2, max_row=11, min_col=4, max_col=8):
    for cell in row:
        if cell.value and cell.value >= 75:
            cell.fill = green_fill
        elif cell.value and cell.value < 50:
            cell.fill = red_fill
auto_width(ws3)

# ── Sheet 4: SLA Breach History ─────────────────────────────────────────────
ws4 = wb.create_sheet("SLA Breach History (6mo)")
ws4.append(["Month", "Total Jobs", "Critical SLA Jobs", "Critical Breaches", "High SLA Jobs", "High Breaches", "Normal Jobs", "Normal Breaches", "Overall Breach Rate (%)", "MTTD (min)", "FTFR (%)"])
style_header(ws4)
history = [
    ("Jan 2026", 4821, 312, 28, 1103, 47, 3406, 12, "=(D3+F3+H3)/B3*100", 24, 60),
    ("Feb 2026", 4310, 298, 21, 987, 38, 3025, 9, "=(D4+F4+H4)/B4*100", 23, 61),
    ("Mar 2026", 5102, 341, 31, 1204, 52, 3557, 14, "=(D5+F5+H5)/B5*100", 25, 59),
    ("Apr 2026", 4987, 329, 25, 1178, 41, 3480, 11, "=(D6+F6+H6)/B6*100", 22, 62),
    ("May 2026", 5341, 355, 33, 1265, 55, 3721, 15, "=(D7+F7+H7)/B7*100", 23, 61),
    ("Jun 2026", 5589, 378, 38, 1321, 61, 3890, 18, "=(D8+F8+H8)/B8*100", 22, 60),
]
for r in history:
    ws4.append(r)
ws4.append(["AVERAGE", "=AVERAGE(B3:B8)", "=AVERAGE(C3:C8)", "=AVERAGE(D3:D8)",
            "=AVERAGE(E3:E8)", "=AVERAGE(F3:F8)", "=AVERAGE(G3:G8)", "=AVERAGE(H3:H8)",
            "=AVERAGE(I3:I8)", "=AVERAGE(J3:J8)", "=AVERAGE(K3:K8)"])
ws4[f"A{ws4.max_row}"].font = Font(bold=True)
auto_width(ws4)

wb.save(r'C:\VectorVolt\esip\test-data\FieldOps_2.0_Supporting_Data.xlsx')
print('XLSX saved.')
