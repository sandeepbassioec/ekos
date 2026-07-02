# 🚀 REPAIRLINK ESTIMATION PLATFORM - PROJECT CONTEXT

**Last Updated:** March 11, 2026  
**Version:** 5.0 → 6.0 (Estimation Wizard MVP)  
**Status:** Active Development - Phase 2 (Universal Estimation Platform)

---

## 📋 PROJECT OVERVIEW

### What We've Built (Phase 1 - COMPLETED ✅)
A React-based application that generates comprehensive development estimates for RepairLink dashboard using:
- **Wide Band Delphi** estimation method
- **PERT (Three-Point)** formula: `E = (O + 4×ML + P) / 6`
- Generates professional Excel (.xlsx) and PowerPoint (.pptx) files
- Covers **3 screens**: Dashboard Overview, Transaction Overview, Transaction Details
- **Total Estimates**: 2,220 hours (1,020 frontend + 1,200 backend)

### What We're Building Now (Phase 2 - IN PROGRESS 🔨)
**Universal Estimation Platform** - A wizard-based app that:
- Supports **15+ estimation methods** (PERT, FPA, UCP, Story Points, COCOMO, etc.)
- Multi-step form wizard for gathering project details
- Accepts manual input OR Excel upload
- Generates professional outputs (Excel + PowerPoint + PDF)
- Saves drafts locally (no data loss)
- Method comparison (compare 2-3 methods side-by-side)

---

## 🏗️ CURRENT TECH STACK

### Core Technologies
```json
{
  "framework": "React 18.3.1",
  "language": "TypeScript",
  "styling": "Tailwind CSS v4",
  "routing": "react-router 7.13.0",
  "build": "Vite 6.3.5",
  "state": "useState (will migrate to Zustand)",
  "forms": "react-hook-form 7.55.0"
}
```

### Key Libraries
```json
{
  "excel": "xlsx 0.18.5",
  "powerpoint": "pptxgenjs 4.0.1",
  "icons": "lucide-react 0.487.0",
  "charts": "recharts 2.15.2",
  "ui": "@radix-ui/* (40+ components)",
  "validation": "zod (to be added)",
  "upload": "react-dropzone (to be added)",
  "state": "zustand (to be added)"
}
```

### File Structure
```
/src/
  /app/
    App.tsx (Main UI - Download buttons)
    ErrorBoundary.tsx (Error handling)
    /components/
      /ui/ (40+ Radix UI components)
      /figma/ (ImageWithFallback)
    /utils/
      excelGenerator.ts (Screen 1 Excel)
      pptxGenerator.ts (Screen 1 PPT)
      completeEstimatesGenerator.ts (All 3 screens Excel)
      completePptxGenerator.ts (All 3 screens PPT - 14 slides)
      estimatesData.ts (Hardcoded estimate data)
  /styles/
    index.css, tailwind.css, theme.css, fonts.css
  /imports/
    (Figma imported components - not used currently)
```

---

## 📊 ESTIMATION DATA SUMMARY

### Frontend Estimates (React + TypeScript + OEC OUI)
| Phase | Hours | Details |
|-------|-------|---------|
| Development | 400 hrs | 3 screens, API integration, search/filter |
| QA Testing | 315 hrs | Manual (225) + Automation (90) |
| Hardening | 305 hrs | Performance, security, refactoring |
| **TOTAL** | **1,020 hrs** | ~127 days (40% reduction via OUI library) |

### Backend Estimates (.NET Core + EF Core + Redis)
| Phase | Hours | Details |
|-------|-------|---------|
| API Development | 890 hrs | 25 APIs, SP→LINQ conversion, Redis caching |
| Testing & Parity | 400 hrs | Unit + Integration + SP parity testing |
| Performance & Hardening | 195 hrs | Query optimization, load testing |
| **TOTAL** | **1,200 hrs** | ~150 days (NOTE: Screen 2 API exists, saves 300 hrs!) |

### Infrastructure Sizing (Added in v5.0)
- **AWS ECS Fargate**: 6 tasks (0.5 vCPU, 1GB each) = ~$118/month
- **On-Prem IIS**: 2 servers (8 cores, 32GB) in HA cluster
- **SQL Server**: 16 cores, 64GB, Always On Availability Groups
- **Redis**: 16GB in-memory, Sentinel HA, 60% cache hit rate
- **Capacity**: Supports 4,284 concurrent users, 336 RPS

---

## 🎯 PHASE 2 GOALS (Estimation Wizard Platform)

### MVP Features (Current Sprint)
1. **7-Step Wizard**
   - Step 1: Method Selection
   - Step 2: Project Setup
   - Step 3: Scope Definition
   - Step 4: Method-Specific Questions
   - Step 5: Risk & Contingency
   - Step 6: Review & Calculate
   - Step 7: Generate Deliverables

2. **Estimation Methods** (Priority Order)
   - ✅ PERT (Already implemented)
   - 🔨 Function Point Analysis (FPA) - Next
   - 📋 Story Points (Agile)
   - 📋 Use Case Points (UCP)
   - 📋 T-Shirt Sizing
   - 📋 COCOMO II (Advanced)

3. **Input Methods**
   - Manual entry (forms)
   - Excel upload (template-based)
   - Quick mode (screen count wizard)

4. **Output Formats**
   - Excel with formulas (reuse existing generators)
   - PowerPoint CXO presentation (reuse existing)
   - PDF summary report (new)
   - JSON export (new)

---

## 🔧 RECENT FIXES

### React Refresh Error (Fixed - March 11, 2026)
**Problem:** `TypeError: Failed to fetch dynamically imported module: @react-refresh`

**Solution Applied:**
```typescript
// vite.config.ts
optimizeDeps: {
  include: ['react', 'react-dom', 'xlsx', 'pptxgenjs', 'lucide-react'],
  exclude: ['@react-refresh'], // ← Key fix
},
server: {
  hmr: { overlay: true },
},
```

### Capacity Slides Added (v5.0)
- Added 5 new slides (10-14) to complete PowerPoint
- Slides cover: Traffic assumptions, ECS Fargate sizing, IIS capacity, HA architecture, deployment timeline
- Total presentation: 14 slides

---

## 📥 HOW TO RESUME WORK (IF LAPTOP RESTARTS)

### Option 1: Return to Figma Make
1. Open your browser
2. Go to the **same Figma Make URL** you were using
3. All code is **automatically saved** in Figma Make cloud
4. The app will be **exactly as you left it**

### Option 2: Share Context with New AI Session
If you start a new chat with Claude/AI:
1. Upload this file: `/PROJECT_CONTEXT.md`
2. Say: "Read PROJECT_CONTEXT.md and continue from where we left off"
3. Mention current task: "We're building the Estimation Wizard MVP"

### What's Preserved:
✅ All code files (auto-saved)
✅ All generated Excel/PPT templates
✅ File structure
✅ Package.json dependencies

### What's NOT Preserved:
❌ Conversation history with AI
❌ In-progress thoughts/plans (that's why this file exists!)

---

## 🎨 DESIGN PATTERNS & CONVENTIONS

### Color Scheme (Tailwind)
```typescript
const colors = {
  primary: '0066CC',      // Blue (main brand)
  secondary: '4F46E5',    // Indigo
  success: '10B981',      // Green (development phase)
  warning: 'F59E0B',      // Orange (QA phase)
  danger: 'EF4444',       // Red (risks)
  dark: '1F2937',         // Dark gray (text)
  light: 'F3F4F6',        // Light gray (backgrounds)
  cyan: '06B6D4',         // Cyan (AWS/cloud)
  rose: 'F43F5E'          // Rose (backend)
}
```

### Excel Sheet Structure (Existing Pattern)
```
Sheet 1: Executive Summary (totals, pie charts)
Sheet 2: Screen 1 Breakdown (WBS with formulas)
Sheet 3: Screen 2 Breakdown
Sheet 4: Screen 3 Breakdown
Sheet 5: Backend Estimates
Sheet 6: Formula Reference (explains all calculations)
Sheet 7: Capacity Planning (NEW in v5.0)
```

### PowerPoint Slide Structure (14 slides)
```
Slide 1:  Title
Slide 2:  Executive Summary
Slide 3:  Scope Breakdown
Slide 4:  Backend Deep Dive
Slide 5:  Timeline & Resources
Slide 6:  Risk Assessment
Slide 7:  Cost Analysis
Slide 8:  Recommendations
Slide 9:  Summary
Slide 10: Capacity Planning Overview (NEW)
Slide 11: AWS ECS Sizing (NEW)
Slide 12: IIS Server Sizing (NEW)
Slide 13: HA Architecture (NEW)
Slide 14: Deployment Timeline (NEW)
```

---

## 🔢 ESTIMATION FORMULAS LIBRARY

### PERT (Currently Implemented)
```typescript
Expected Time (E) = (Optimistic + 4×MostLikely + Pessimistic) / 6
Standard Deviation = (Pessimistic - Optimistic) / 6
Variance = SD²
Confidence Interval = E ± (Z × SD)
```

### Function Point Analysis (To Implement)
```typescript
// Component Weights
EI (External Input): Low=3, Med=4, High=6
EO (External Output): Low=4, Med=5, High=7
EQ (External Inquiry): Low=3, Med=4, High=6
ILF (Internal Logical File): Low=7, Med=10, High=15
EIF (External Interface File): Low=5, Med=7, High=10

// Calculation
UFP = Σ(Component × Weight)
VAF = 0.65 + (0.01 × Σ(TCF)) // Technical Complexity Factors (14)
AFP = UFP × VAF
Effort = AFP × Productivity (10-20 hrs/FP)
```

### Use Case Points (To Implement)
```typescript
UUCW = Σ(Simple×5 + Average×10 + Complex×15)
UAW = Σ(Simple×1 + Average×2 + Complex×3)
UUCP = UUCW + UAW
TCF = 0.6 + (0.01 × Σ(TF[i]))
ECF = 1.4 + (-0.03 × Σ(EF[i]))
UCP = UUCP × TCF × ECF
Effort = UCP × 20 hrs
```

### Story Points → Hours (To Implement)
```typescript
// Fibonacci: 1, 2, 3, 5, 8, 13, 21, 34
TotalPoints = Σ(Story.points)
Velocity = HistoricalVelocity || EstimatedVelocity
Sprints = TotalPoints / Velocity
Hours = TotalPoints × HoursPerPoint (6-10 hrs typical)
```

---

## 📦 PACKAGES TO INSTALL (Phase 2)

```bash
# State Management
pnpm add zustand

# File Upload
pnpm add react-dropzone

# Validation
pnpm add zod

# PDF Generation
pnpm add jspdf jspdf-autotable

# Already Installed:
✅ react-router (navigation)
✅ react-hook-form (forms)
✅ xlsx (Excel)
✅ pptxgenjs (PowerPoint)
✅ recharts (charts)
```

---

## 🚀 NEXT IMMEDIATE TASKS

### Current Sprint (Week of March 11, 2026)
- [x] Create PROJECT_CONTEXT.md (this file)
- [ ] Create multi-step wizard shell (routes)
- [ ] Build Step 1: Method Selection screen
- [ ] Build Step 2: Project Setup form
- [ ] Build Step 3: Scope Definition (manual entry)
- [ ] Integrate existing PERT calculations
- [ ] Test end-to-end flow
- [ ] Generate Excel/PPT from wizard data

### Week 2
- [ ] Add Function Point Analysis method
- [ ] Build method comparison view
- [ ] Add Excel upload feature
- [ ] Implement save/load draft (localStorage)

---

## 💡 KEY INSIGHTS & DECISIONS

### Why This Approach?
1. **Reusability**: Existing Excel/PPT generators are already robust - just feed them dynamic data
2. **Modularity**: Each estimation method is a separate module - easy to add/remove
3. **User Choice**: Let users choose 1-3 methods and compare results
4. **No Backend Needed**: All calculations happen client-side (localStorage for drafts)

### Architecture Decisions
```typescript
// State Management: Zustand (chosen for simplicity)
// Why not Redux? Too heavy for this use case
// Why not Context? Performance issues with large forms

// Routing: React Router (already installed)
// Multi-step wizard: /estimation/step/:stepNumber

// Data Flow:
User Input → Zustand Store → Calculation Engine → Excel/PPT Generators
```

### User Journey
```
Landing Page → [Start New Estimation]
  ↓
Step 1: Choose Method(s) → PERT, FPA, UCP (multi-select)
  ↓
Step 2: Project Details → Name, type, tech stack, rates
  ↓
Step 3: Define Scope → Add screens manually OR upload Excel
  ↓
Step 4: Method Questions → Different for each method
  ↓
Step 5: Risk Factors → Contingency %, confidence level
  ↓
Step 6: Review → Live calculation preview, comparison table
  ↓
Step 7: Generate → Download Excel + PPT + PDF
```

---

## 🎯 SUCCESS METRICS

### MVP Success Criteria
- [ ] User can complete full wizard in <10 minutes
- [ ] Generated Excel has all formulas working
- [ ] Generated PPT matches existing quality
- [ ] Draft saves automatically every 30 seconds
- [ ] Works offline (no API calls)
- [ ] Mobile-responsive (at least tablet-friendly)

### Quality Bars
- Code coverage: Not applicable (client-side app)
- TypeScript strict mode: Enabled
- Accessibility: WCAG AA minimum
- Performance: <3s initial load, <500ms step transitions

---

## 📞 CONTACT & COLLABORATION

### How to Share This Project
1. **Figma Make URL**: Share the app URL directly (all code is there)
2. **Export Code**: Download all files as .zip from Figma Make
3. **Context Document**: Share this file (`PROJECT_CONTEXT.md`) with anyone who needs to understand the project

### How to Get Help
- Upload this file to any AI assistant
- Say: "Continue working on the Estimation Wizard based on PROJECT_CONTEXT.md"
- Mention specific task: "I need help with Step 4: Method-Specific Questions for FPA"

---

## 🏆 WINS & ACHIEVEMENTS

### Phase 1 Achievements ✅
1. Built complete PERT estimation system
2. Generated professional Excel with 7 sheets, all formulas working
3. Generated 14-slide PowerPoint presentation
4. Added infrastructure capacity planning (AWS + On-Prem)
5. Fixed React Refresh module loading error
6. Total estimates: 2,220 hours delivered with full breakdown

### What Makes This Special
- **Defend-ready**: Every number has a formula - no black box estimates
- **CXO-ready**: PowerPoint can go straight to executives
- **Reusable**: Templates can be adjusted for any project
- **Professional**: Industry-standard PERT + Delphi methodology

---

## 📚 REFERENCES & RESOURCES

### Estimation Methodologies
- PERT: https://en.wikipedia.org/wiki/Program_evaluation_and_review_technique
- Function Points: https://www.ifpug.org/
- Use Case Points: Karner's original paper (1993)
- COCOMO II: http://csse.usc.edu/tools/cocomoii.php

### Libraries Documentation
- pptxgenjs: https://gitbrent.github.io/PptxGenJS/
- xlsx (SheetJS): https://docs.sheetjs.com/
- react-hook-form: https://react-hook-form.com/
- zustand: https://github.com/pmndrs/zustand

---

## 🔄 VERSION HISTORY

### v5.0 (March 10, 2026)
- Added 5 capacity planning slides to PowerPoint
- Added infrastructure sizing (AWS ECS + IIS)
- Total: 14 slides in complete presentation

### v4.0 (March 9, 2026)
- Created complete Excel with all 3 screens
- Created complete PowerPoint with 9 slides
- Added backend estimates (.NET Core + EF Core)

### v3.0 (March 8, 2026)
- Added Screen 3: Transaction Details estimates
- Backend: 18 stored procedures to convert

### v2.0 (March 7, 2026)
- Added Screen 2: Transaction Overview estimates
- Discovered API already exists (300 hrs saved!)

### v1.0 (March 6, 2026)
- Initial MVP: Screen 1 estimates only
- Excel + PowerPoint generators working

### v6.0 (March 11, 2026) - IN PROGRESS
- Started Estimation Wizard Platform
- Multi-step wizard architecture
- Universal estimation tool (15+ methods planned)

---

**END OF CONTEXT DOCUMENT**

*This file should be updated whenever major decisions are made or architecture changes.*
*Last human interaction: March 11, 2026 - Request to build universal estimation platform*
