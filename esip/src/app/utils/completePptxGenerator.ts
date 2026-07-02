import pptxgen from 'pptxgenjs';

export const generateCompletePowerPointFile = () => {
  const pptx = new pptxgen();

  pptx.author = 'RepairLink Estimation Team';
  pptx.company = 'RepairLink';
  pptx.title = 'RepairLink Full Stack Development Estimates';
  pptx.subject = 'Wide Band Delphi + PERT Analysis - Frontend + Backend';

  const colors = {
    primary: '0066CC',
    secondary: '4F46E5',
    success: '10B981',
    warning: 'F59E0B',
    danger: 'EF4444',
    dark: '1F2937',
    light: 'F3F4F6',
    white: 'FFFFFF',
    cyan: '06B6D4',
    rose: 'F43F5E'
  };

  // ===== SLIDE 1: Title Slide =====
  const slide1 = pptx.addSlide();
  slide1.background = { color: colors.primary };
  
  slide1.addText('RepairLink Application', {
    x: 0.5, y: 1.3, w: 9, h: 1,
    fontSize: 48, bold: true, color: colors.white, align: 'center'
  });
  
  slide1.addText('Full Stack Development Estimates', {
    x: 0.5, y: 2.4, w: 9, h: 0.7,
    fontSize: 32, color: colors.white, align: 'center'
  });
  
  slide1.addText('Frontend (React + TypeScript) + Backend (.NET Core + EF Core)', {
    x: 0.5, y: 3.3, w: 9, h: 0.5,
    fontSize: 18, color: colors.light, align: 'center'
  });
  
  slide1.addText('Wide Band Delphi + PERT Analysis\nScreens 1-3 | March 10, 2026 | Version 5.0', {
    x: 0.5, y: 4.3, w: 9, h: 0.8,
    fontSize: 16, color: colors.light, align: 'center'
  });

  // ===== SLIDE 2: Executive Summary =====
  const slide2 = pptx.addSlide();
  slide2.addText('Executive Summary: Full Stack Estimates', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 26, bold: true, color: colors.dark
  });
  
  // Frontend boxes
  slide2.addText('FRONTEND (React + TypeScript)', {
    x: 0.5, y: 1.0, w: 9, h: 0.3,
    fontSize: 14, bold: true, color: colors.dark
  });
  
  const frontendData = [
    { title: 'Development', hours: '400 hrs', days: '50 days', color: colors.success },
    { title: 'QA Testing', hours: '315 hrs', days: '39.4 days', color: colors.warning },
    { title: 'Hardening', hours: '305 hrs', days: '38.1 days', color: colors.secondary }
  ];
  
  frontendData.forEach((item, idx) => {
    slide2.addShape(pptx.ShapeType.rect, {
      x: 0.5 + (idx * 3.2), y: 1.4, w: 2.8, h: 1.1,
      fill: { color: item.color }, line: { type: 'none' }
    });
    
    slide2.addText(item.title, {
      x: 0.5 + (idx * 3.2), y: 1.5, w: 2.8, h: 0.25,
      fontSize: 12, bold: true, color: colors.white, align: 'center'
    });
    
    slide2.addText(item.hours, {
      x: 0.5 + (idx * 3.2), y: 1.85, w: 2.8, h: 0.35,
      fontSize: 20, bold: true, color: colors.white, align: 'center'
    });
    
    slide2.addText(item.days, {
      x: 0.5 + (idx * 3.2), y: 2.25, w: 2.8, h: 0.2,
      fontSize: 11, color: colors.white, align: 'center'
    });
  });
  
  // Backend boxes
  slide2.addText('BACKEND (.NET Core + EF Core + SQL Server)', {
    x: 0.5, y: 2.7, w: 9, h: 0.3,
    fontSize: 14, bold: true, color: colors.dark
  });
  
  const backendData = [
    { title: 'API Development', hours: '855 hrs', days: '107 days', color: colors.cyan },
    { title: 'Testing & Integration', hours: '250 hrs', days: '31 days', color: colors.rose },
    { title: 'Performance & Hardening', hours: '165 hrs', days: '21 days', color: '8B5CF6' }
  ];
  
  backendData.forEach((item, idx) => {
    slide2.addShape(pptx.ShapeType.rect, {
      x: 0.5 + (idx * 3.2), y: 3.1, w: 2.8, h: 1.1,
      fill: { color: item.color }, line: { type: 'none' }
    });
    
    slide2.addText(item.title, {
      x: 0.5 + (idx * 3.2), y: 3.2, w: 2.8, h: 0.25,
      fontSize: 12, bold: true, color: colors.white, align: 'center'
    });
    
    slide2.addText(item.hours, {
      x: 0.5 + (idx * 3.2), y: 3.55, w: 2.8, h: 0.35,
      fontSize: 20, bold: true, color: colors.white, align: 'center'
    });
    
    slide2.addText(item.days, {
      x: 0.5 + (idx * 3.2), y: 3.95, w: 2.8, h: 0.2,
      fontSize: 11, color: colors.white, align: 'center'
    });
  });
  
  // Total
  slide2.addShape(pptx.ShapeType.rect, {
    x: 1.5, y: 4.5, w: 6.5, h: 1.0,
    fill: { color: colors.primary }, line: { type: 'none' }
  });
  
  slide2.addText('Total Full Stack: 2,220 hours', {
    x: 1.5, y: 4.7, w: 6.5, h: 0.3,
    fontSize: 20, bold: true, color: colors.white, align: 'center'
  });
  
  slide2.addText('Frontend: 1,020 hrs | Backend: 1,200 hrs | ≈278 days (55-56 weeks / ~12 months)', {
    x: 1.5, y: 5.05, w: 6.5, h: 0.3,
    fontSize: 12, color: colors.white, align: 'center'
  });
  
  // Key highlights
  slide2.addText('⭐ Screen 2 API Already Exists - Integration Only! (Saves ~300 hrs)', {
    x: 0.8, y: 5.8, w: 8.5, h: 0.25,
    fontSize: 11, bold: true, color: colors.danger
  });
  
  slide2.addText('✓ 84% confidence level (conservative) | With 15% buffer: 2,553 hrs (64 weeks)', {
    x: 0.8, y: 6.15, w: 8.5, h: 0.25,
    fontSize: 10, color: colors.dark
  });

  // ===== SLIDE 3: Scope Breakdown =====
  const slide3 = pptx.addSlide();
  slide3.addText('Complete Scope: Frontend + Backend', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 26, bold: true, color: colors.dark
  });
  
  const scopeData = [
    ['Screen', 'Frontend (hrs)', 'Backend (hrs)', 'Total (hrs)', 'Notes'],
    ['Screen 1', '247', '225', '472', 'NEW APIs needed'],
    ['Screen 2', '318', '75', '393', '✅ API exists!'],
    ['Screen 3', '455', '900', '1,355', 'SP→EF migration'],
    ['TOTAL', '1,020', '1,200', '2,220', '']
  ];
  
  slide3.addTable(scopeData, {
    x: 0.5, y: 1.0, w: 9, h: 2.0,
    fontSize: 11,
    border: { pt: 1, color: 'CCCCCC' },
    fill: { color: colors.light },
    color: colors.dark,
    align: 'center',
    valign: 'middle',
    colW: [1.8, 1.8, 1.8, 1.8, 1.8]
  });
  
  // Frontend Features
  slide3.addText('Frontend Features (React + TypeScript + OEC OUI)', {
    x: 0.5, y: 3.3, w: 4.5, h: 0.3,
    fontSize: 13, bold: true, color: colors.primary
  });
  
  const feFeatures = [
    '• Activity Dashboard (3 status cards)',
    '• Analytics Dashboard (12 KPI metrics)',
    '• Data Table (11+ columns, sorting)',
    '• Drag & Drop column customization',
    '• Multi-criteria filters',
    '• Complex form (20+ fields)',
    '• Parts table (15+ columns)',
    '• 5 Modal popups (VIN, Order Info, etc.)'
  ];
  
  feFeatures.forEach((text, idx) => {
    slide3.addText(text, {
      x: 0.7, y: 3.7 + (idx * 0.24), w: 4.0, h: 0.2,
      fontSize: 9, color: colors.dark
    });
  });
  
  // Backend Features
  slide3.addText('Backend Features (.NET Core + EF Core)', {
    x: 5.2, y: 3.3, w: 4.5, h: 0.3,
    fontSize: 13, bold: true, color: colors.cyan
  });
  
  const beFeatures = [
    '• Screen 1: 3 NEW APIs (Dashboard)',
    '• Screen 2: Integration only (API ready)',
    '• Screen 3: 18+ SP→EF conversion',
    '• Entity Framework Core + LINQ',
    '• Redis caching layer',
    '• AutoMapper DTOs',
    '• Parity testing (SP vs EF)',
    '• GitHub Copilot (75% generation)'
  ];
  
  beFeatures.forEach((text, idx) => {
    slide3.addText(text, {
      x: 5.4, y: 3.7 + (idx * 0.24), w: 4.0, h: 0.2,
      fontSize: 9, color: colors.dark
    });
  });
  
  // Important note
  slide3.addShape(pptx.ShapeType.rect, {
    x: 0.5, y: 5.7, w: 9, h: 0.6,
    fill: { color: 'FEF3C7' }, line: { pt: 2, color: colors.warning }
  });
  
  slide3.addText('💡 COST SAVER: Transaction Overview API exists - saves ~300 hrs & 5 weeks!', {
    x: 0.7, y: 5.85, w: 8.6, h: 0.3,
    fontSize: 13, bold: true, color: '92400E', align: 'center', valign: 'middle'
  });

  // ===== SLIDE 4: Backend Deep Dive =====
  const slide4 = pptx.addSlide();
  slide4.addText('Backend Development Breakdown', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 26, bold: true, color: colors.dark
  });
  
  const backendBreakdown = [
    ['Screen', 'API Dev', 'Testing', 'Performance', 'Total', 'Status'],
    ['Screen 1\nDashboard', '135 hrs', '60 hrs', '30 hrs', '225 hrs', 'NEW APIs'],
    ['Screen 2\nTrans. Overview', '30 hrs', '25 hrs', '20 hrs', '75 hrs', '✅ API Ready'],
    ['Screen 3\nTrans. Details', '540 hrs', '245 hrs', '115 hrs', '900 hrs', 'SP Migration'],
    ['TOTAL', '705 hrs', '330 hrs', '165 hrs', '1,200 hrs', '']
  ];
  
  slide4.addTable(backendBreakdown, {
    x: 0.5, y: 1.0, w: 9, h: 2.5,
    fontSize: 10,
    border: { pt: 1, color: 'CCCCCC' },
    fill: { color: colors.light },
    color: colors.dark,
    align: 'center',
    valign: 'middle',
    colW: [2.0, 1.5, 1.3, 1.5, 1.4, 1.3]
  });
  
  slide4.addText('Screen 3: Stored Procedure Modernization (18+ SPs)', {
    x: 0.5, y: 3.8, w: 9, h: 0.3,
    fontSize: 14, bold: true, color: colors.dark
  });
  
  const screen3Details = [
    ['Category', 'Tasks', 'Hours'],
    ['SP Translation', 'Transaction, PartPool, Document, LineItem, Chains, EventLogs, etc.', '150'],
    ['Data Modeling', 'EF Core entities, DbContext, relationships, migrations', '103'],
    ['LINQ Development', 'Repository layer, query optimization, compiled queries', '119'],
    ['Copilot Work', 'AI-assisted code generation (75%) + human review', '76'],
    ['DTO & Mapping', 'AutoMapper configuration, API contracts', '52'],
    ['Parity Testing', 'Automated SP vs EF output comparison', '84'],
    ['Other Testing', 'Unit, integration, e2e, performance tests', '99'],
    ['Hardening', 'Security, performance tuning, documentation', '77']
  ];
  
  slide4.addTable(screen3Details, {
    x: 0.5, y: 4.2, w: 9, h: 2.5,
    fontSize: 9,
    border: { pt: 1, color: 'CCCCCC' },
    fill: { color: colors.light },
    color: colors.dark,
    align: 'left',
    valign: 'middle',
    colW: [2.0, 5.5, 1.5]
  });

  // ===== SLIDE 5: Timeline & Resources =====
  const slide5 = pptx.addSlide();
  slide5.addText('Timeline & Resource Plan', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 26, bold: true, color: colors.dark
  });
  
  const timelineData = [
    ['Phase', 'Duration', 'Start', 'End', 'Team'],
    ['Sprint 0 (Setup)', '2 weeks', 'Week 1', 'Week 2', 'Full Team'],
    ['FE Development', '26 weeks', 'Week 3', 'Week 28', '2-3 FE Devs'],
    ['BE Screen 1 & 2', '7 weeks', 'Week 3', 'Week 9', '2 BE Devs'],
    ['BE Screen 3 (SP Mig)', '24 weeks', 'Week 8', 'Week 31', '3 BE Devs'],
    ['Integration & Testing', '8 weeks', 'Week 28', 'Week 36', 'Full Team'],
    ['Hardening', '6 weeks', 'Week 32', 'Week 38', 'Specialists'],
    ['UAT & Fixes', '6 weeks', 'Week 36', 'Week 42', 'QA + Devs'],
    ['Deployment', '3 weeks', 'Week 42', 'Week 45', 'DevOps'],
    ['Buffer (15%)', '7 weeks', 'Week 45', 'Week 52', 'As needed'],
    ['TOTAL', '52 weeks', 'Week 1', 'Week 52', '≈ 1 year']
  ];
  
  slide5.addTable(timelineData, {
    x: 0.5, y: 1.0, w: 9, h: 3.5,
    fontSize: 10,
    border: { pt: 1, color: 'CCCCCC' },
    fill: { color: colors.light },
    color: colors.dark,
    align: 'center',
    valign: 'middle',
    colW: [2.5, 1.5, 1.3, 1.3, 2.4]
  });
  
  slide5.addText('Team Composition (6.8 FTE)', {
    x: 0.5, y: 4.8, w: 9, h: 0.3,
    fontSize: 14, bold: true, color: colors.dark
  });
  
  const teamData = [
    ['Role', 'FTE', 'Primary Focus'],
    ['Senior Full Stack Dev', '1.0', 'Architecture, code review'],
    ['Frontend Developer', '1.0', 'React components, UI/UX'],
    ['Backend Developer', '2.0', 'API dev, EF Core, SP migration'],
    ['QA Engineer', '1.0', 'Manual testing, parity validation'],
    ['QA Automation', '0.5', 'Test automation'],
    ['DevOps', '0.5', 'CI/CD, performance'],
    ['DBA', '0.3', 'DB optimization'],
    ['Others (PM, Security)', '0.5', 'Management, security']
  ];
  
  slide5.addTable(teamData, {
    x: 0.5, y: 5.2, w: 9, h: 1.6,
    fontSize: 9,
    border: { pt: 1, color: 'CCCCCC' },
    fill: { color: colors.light },
    color: colors.dark,
    align: 'left',
    valign: 'middle',
    colW: [2.5, 1.0, 5.5]
  });

  // ===== SLIDE 6: Risk Assessment =====
  const slide6 = pptx.addSlide();
  slide6.addText('Risk Assessment & Mitigation', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 26, bold: true, color: colors.dark
  });
  
  const riskData = [
    ['Risk', 'Impact', 'Prob', 'Mitigation'],
    ['Scope Creep', 'High', 'High', 'Formal change process'],
    ['EF Performance (Screen 3)', 'High', 'Med', 'Benchmarking, projections'],
    ['Copilot Errors (Screen 3)', 'High', 'Med', 'Mandatory code review'],
    ['State Management (FE)', 'High', 'Med', 'Redux/Zustand'],
    ['Parity Failures (Screen 3)', 'High', 'High', 'Automated parity tests'],
    ['OEC OUI Gaps', 'Med', 'Med', 'Custom dev buffer'],
    ['Team Knowledge Gaps', 'Med', 'Med', 'Training, pair programming']
  ];
  
  slide6.addTable(riskData, {
    x: 0.5, y: 1.0, w: 9, h: 2.8,
    fontSize: 10,
    border: { pt: 1, color: 'CCCCCC' },
    fill: { color: colors.light },
    color: colors.dark,
    align: 'left',
    valign: 'middle',
    colW: [3.5, 1.2, 1.0, 3.3]
  });
  
  slide6.addText('Confidence Intervals:', {
    x: 0.5, y: 4.1, w: 9, h: 0.3,
    fontSize: 14, bold: true, color: colors.dark
  });
  
  const confidenceData = [
    ['Scenario', 'Hours', 'Days', 'Weeks', 'Confidence'],
    ['Best Case (60%)', '1,332', '167', '33', '1% (Unlikely)'],
    ['Expected Case', '2,220', '278', '56', '50% (Realistic)'],
    ['Conservative (+15%)', '2,553', '319', '64', '84% (Recommended)'],
    ['Worst Case (+35%)', '2,997', '375', '75', '99% (High certainty)']
  ];
  
  slide6.addTable(confidenceData, {
    x: 1.0, y: 4.5, w: 8, h: 1.8,
    fontSize: 10,
    border: { pt: 1, color: 'CCCCCC' },
    fill: { color: colors.light },
    color: colors.dark,
    align: 'center',
    valign: 'middle',
    colW: [2.2, 1.5, 1.3, 1.3, 1.7]
  });

  // ===== SLIDE 7: Cost Analysis =====
  const slide7 = pptx.addSlide();
  slide7.addText('Cost Analysis & Savings', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 26, bold: true, color: colors.dark
  });
  
  // Cost breakdown
  const costData = [
    ['Resource Type', 'Hours', 'Rate/hr', 'Cost'],
    ['Frontend Team', '1,020', '$140', '$142,800'],
    ['Backend Team', '1,200', '$160', '$192,000'],
    ['QA Team', '1,205', '$120', '$144,600'],
    ['DevOps & Others', '535', '$150', '$80,250'],
    ['SUBTOTAL', '3,960', '', '$559,650'],
    ['15% Contingency', '594', '', '$83,948'],
    ['TOTAL ESTIMATED COST', '4,554 hrs', '', '$643,598']
  ];
  
  slide7.addTable(costData, {
    x: 0.5, y: 1.0, w: 9, h: 2.8,
    fontSize: 11,
    border: { pt: 1, color: 'CCCCCC' },
    fill: { color: colors.light },
    color: colors.dark,
    align: 'center',
    valign: 'middle',
    colW: [2.5, 1.8, 1.8, 2.9]
  });
  
  // Savings box
  slide7.addShape(pptx.ShapeType.rect, {
    x: 1.0, y: 4.2, w: 8.0, h: 1.3,
    fill: { color: 'D1FAE5' }, line: { pt: 2, color: colors.success }
  });
  
  slide7.addText('💰 COST SAVINGS FROM EXISTING API', {
    x: 1.0, y: 4.4, w: 8.0, h: 0.3,
    fontSize: 14, bold: true, color: '065F46', align: 'center'
  });
  
  slide7.addText('Screen 2 API Already Exists\nSaved: ~300 backend hours + 70 testing hours = 370 hrs\nCost Savings: ~$55,500 | Timeline Savings: ~5 weeks', {
    x: 1.0, y: 4.8, w: 8.0, h: 0.6,
    fontSize: 12, color: '065F46', align: 'center'
  });
  
  // ROI note
  slide7.addText('Note: Estimates assume market rates. Adjust based on location and team seniority.', {
    x: 0.5, y: 5.8, w: 9, h: 0.3,
    fontSize: 10, italic: true, color: colors.dark, align: 'center'
  });

  // ===== SLIDE 8: Recommendations =====
  const slide8 = pptx.addSlide();
  slide8.addText('Recommendations & Next Steps', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 26, bold: true, color: colors.dark
  });
  
  // For faster delivery
  slide8.addShape(pptx.ShapeType.rect, {
    x: 0.5, y: 1.0, w: 4.2, h: 2.2,
    fill: { color: 'D1FAE5' }, line: { pt: 2, color: colors.success }
  });
  
  slide8.addText('For Faster Delivery:', {
    x: 0.7, y: 1.2, w: 3.8, h: 0.3,
    fontSize: 13, bold: true, color: colors.success
  });
  
  const deliveryRecs = [
    '1. Parallel Development',
    '   • Start FE & BE simultaneously',
    '   • Use mocks for Screen 3 API',
    '',
    '2. Leverage Screen 2 API',
    '   • Already built - quick wins',
    '   • Focus on Screen 1 & 3',
    '',
    '3. Invest in Automation',
    '   • Upfront cost pays off',
    '   • Faster regression testing',
    '',
    '4. GitHub Copilot',
    '   • 75% code generation',
    '   • Speeds up SP conversion'
  ];
  
  deliveryRecs.forEach((text, idx) => {
    slide8.addText(text, {
      x: 0.8, y: 1.55 + (idx * 0.17), w: 3.6, h: 0.15,
      fontSize: text.startsWith(' ') ? 9 : 10,
      bold: !text.startsWith(' ') && text !== '',
      color: colors.dark
    });
  });
  
  // For cost optimization
  slide8.addShape(pptx.ShapeType.rect, {
    x: 5.2, y: 1.0, w: 4.2, h: 2.2,
    fill: { color: 'DBEAFE' }, line: { pt: 2, color: colors.primary }
  });
  
  slide8.addText('For Cost Optimization:', {
    x: 5.4, y: 1.2, w: 3.8, h: 0.3,
    fontSize: 13, bold: true, color: colors.primary
  });
  
  const costRecs = [
    '1. Phased Approach',
    '   • Launch Screen 2 first',
    '   • Defer Screen 3 if needed',
    '',
    '2. Manual QA Initially',
    '   • Save automation cost upfront',
    '   • Add automation later',
    '',
    '3. Offshore Team Mix',
    '   • Sr devs onshore',
    '   • Jr devs offshore for tests',
    '',
    '4. Leverage OEC OUI',
    '   • Already saving 40%',
    '   • Maximize component reuse'
  ];
  
  costRecs.forEach((text, idx) => {
    slide8.addText(text, {
      x: 5.5, y: 1.55 + (idx * 0.17), w: 3.6, h: 0.15,
      fontSize: text.startsWith(' ') ? 9 : 10,
      bold: !text.startsWith(' ') && text !== '',
      color: colors.dark
    });
  });
  
  // Recommended approach
  slide8.addShape(pptx.ShapeType.rect, {
    x: 1.0, y: 3.5, w: 8.0, h: 0.8,
    fill: { color: colors.primary }, line: { type: 'none' }
  });
  
  slide8.addText('💡 Recommended: 64-week timeline (15% buffer) with parallel execution', {
    x: 1.0, y: 3.75, w: 8.0, h: 0.3,
    fontSize: 14, bold: true, color: colors.white, align: 'center', valign: 'middle'
  });
  
  // Next steps
  slide8.addText('Next Steps:', {
    x: 0.5, y: 4.6, w: 9, h: 0.3,
    fontSize: 14, bold: true, color: colors.dark
  });
  
  const nextSteps = [
    ['#', 'Action', 'Owner', 'Timeline'],
    ['1', 'Review & approve estimates', 'Stakeholders', 'Week 0'],
    ['2', 'Assemble team (6-7 FTE)', 'HR + PM', 'Week 0-1'],
    ['3', 'Environment setup', 'DevOps', 'Week 1'],
    ['4', 'Project kickoff', 'PM + Team', 'Week 2'],
    ['5', 'Sprint planning (2-week sprints)', 'PM + Scrum Master', 'Week 2+'],
    ['6', 'Quick win: Screen 2 integration', 'FE Team', 'Week 3-7']
  ];
  
  slide8.addTable(nextSteps, {
    x: 0.5, y: 5.0, w: 9, h: 1.7,
    fontSize: 9,
    border: { pt: 1, color: 'CCCCCC' },
    fill: { color: colors.light },
    color: colors.dark,
    align: 'left',
    valign: 'middle',
    colW: [0.6, 4.0, 2.4, 2.0]
  });

  // ===== SLIDE 9: Summary =====
  const slide9 = pptx.addSlide();
  slide9.background = { color: colors.secondary };
  
  slide9.addText('Summary', {
    x: 0.5, y: 1.0, w: 9, h: 0.6,
    fontSize: 32, bold: true, color: colors.white, align: 'center'
  });
  
  const summaryPoints = [
    { label: 'Timeline:', value: '52-64 weeks (12-15 months)' },
    { label: 'Team Size:', value: '6-7 full-time resources' },
    { label: 'Total Effort:', value: '2,220 hours (FE: 1,020 | BE: 1,200)' },
    { label: 'Estimated Cost:', value: '~$644K (with contingency)' },
    { label: 'Cost Savings:', value: '~$55K (Screen 2 API exists)' },
    { label: 'Confidence:', value: '84% (Conservative estimate)' }
  ];
  
  summaryPoints.forEach((point, idx) => {
    slide9.addText(`${point.label} ${point.value}`, {
      x: 1.5, y: 2.0 + (idx * 0.5), w: 7, h: 0.35,
      fontSize: 16, color: colors.white, align: 'center'
    });
  });
  
  slide9.addShape(pptx.ShapeType.rect, {
    x: 2.5, y: 5.0, w: 5, h: 0.7,
    fill: { color: colors.white }, line: { type: 'none' }
  });
  
  slide9.addText('✅ Ready for Approval', {
    x: 2.5, y: 5.15, w: 5, h: 0.4,
    fontSize: 22, bold: true, color: colors.secondary, align: 'center', valign: 'middle'
  });
  
  slide9.addText('Estimation Method: Wide Band Delphi + PERT | Formula: E = (O + 4×ML + P) / 6', {
    x: 0.5, y: 6.2, w: 9, h: 0.3,
    fontSize: 10, color: colors.light, align: 'center'
  });

  // ===== SLIDE 10: Capacity Planning Overview =====
  const slide10 = pptx.addSlide();
  slide10.background = { color: '1E293B' };
  
  slide10.addText('Capacity Planning & Infrastructure Sizing', {
    x: 0.5, y: 0.4, w: 9, h: 0.6,
    fontSize: 36, bold: true, color: colors.white, align: 'center'
  });
  
  slide10.addText('Hybrid Cloud Architecture: AWS ECS Fargate + On-Premise IIS', {
    x: 0.5, y: 1.1, w: 9, h: 0.4,
    fontSize: 16, color: colors.light, align: 'center'
  });
  
  // Traffic Assumptions Box
  slide10.addShape(pptx.ShapeType.roundRect, {
    x: 0.5, y: 1.8, w: 4, h: 2.5,
    fill: { color: colors.white }
  });
  
  slide10.addText('📊 Traffic Assumptions', {
    x: 0.5, y: 1.9, w: 4, h: 0.4,
    fontSize: 18, bold: true, color: colors.primary, align: 'center'
  });
  
  const trafficData = [
    ['Metric', 'Value'],
    ['Concurrent Users (Peak)', '500 users'],
    ['Peak Load Multiplier', '2x (1,000 users)'],
    ['Avg RPS', '27.78 req/sec'],
    ['Peak RPS', '55.56 req/sec'],
    ['Session Duration', '15 minutes'],
    ['Daily Active Users', '4,000 users/day']
  ];
  
  slide10.addTable(trafficData, {
    x: 0.6, y: 2.4, w: 3.8, h: 1.8,
    fontSize: 10,
    border: { pt: 1, color: 'E5E7EB' },
    fill: { color: 'F9FAFB' },
    color: colors.dark,
    valign: 'middle',
    rowH: [0.3, 0.22, 0.22, 0.22, 0.22, 0.22, 0.22]
  });
  
  // Deployment Architecture Box
  slide10.addShape(pptx.ShapeType.roundRect, {
    x: 5.0, y: 1.8, w: 4.5, h: 2.5,
    fill: { color: colors.white }
  });
  
  slide10.addText('🏗️ Deployment Architecture', {
    x: 5.0, y: 1.9, w: 4.5, h: 0.4,
    fontSize: 18, bold: true, color: colors.secondary, align: 'center'
  });
  
  const archData = [
    ['Layer', 'Technology', 'Location'],
    ['Frontend', 'React App (ECS Fargate)', 'AWS Cloud'],
    ['Load Balancer', 'Application Load Balancer', 'AWS Cloud'],
    ['Backend APIs', 'IIS + .NET Core', 'On-Premise'],
    ['Database', 'SQL Server 2022 + EF Core', 'On-Premise'],
    ['Cache', 'Redis (16GB + Sentinel)', 'On-Premise'],
    ['Network', 'Direct Connect 1Gbps', 'Hybrid']
  ];
  
  slide10.addTable(archData, {
    x: 5.1, y: 2.4, w: 4.3, h: 1.8,
    fontSize: 9,
    border: { pt: 1, color: 'E5E7EB' },
    fill: { color: 'F9FAFB' },
    color: colors.dark,
    valign: 'middle',
    rowH: [0.3, 0.21, 0.21, 0.21, 0.21, 0.21, 0.21]
  });
  
  // Key Benefits
  slide10.addShape(pptx.ShapeType.roundRect, {
    x: 0.5, y: 4.5, w: 9, h: 1.8,
    fill: { color: '0EA5E9' }
  });
  
  slide10.addText('✅ Key Architecture Benefits', {
    x: 0.5, y: 4.6, w: 9, h: 0.4,
    fontSize: 18, bold: true, color: colors.white, align: 'center'
  });
  
  slide10.addText([
    { text: '🌐 Hybrid Cloud: ', options: { bold: true } },
    { text: 'Frontend on AWS (scalable) + Backend on-prem (data sovereignty)' }
  ], {
    x: 0.8, y: 5.1, w: 4, h: 0.3,
    fontSize: 11, color: colors.white, bullet: true
  });
  
  slide10.addText([
    { text: '🔒 High Availability: ', options: { bold: true } },
    { text: 'Multi-AZ frontend, clustered backend, DB Always On AG' }
  ], {
    x: 0.8, y: 5.5, w: 4, h: 0.3,
    fontSize: 11, color: colors.white, bullet: true
  });
  
  slide10.addText([
    { text: '📈 Auto-Scaling: ', options: { bold: true } },
    { text: 'ECS tasks scale 3-9 based on CPU (70% threshold)' }
  ], {
    x: 5.2, y: 5.1, w: 4, h: 0.3,
    fontSize: 11, color: colors.white, bullet: true
  });
  
  slide10.addText([
    { text: '💰 Cost-Effective: ', options: { bold: true } },
    { text: '~$300/month AWS + existing on-prem infrastructure' }
  ], {
    x: 5.2, y: 5.5, w: 4, h: 0.3,
    fontSize: 11, color: colors.white, bullet: true
  });

  // ===== SLIDE 11: AWS ECS Fargate Sizing =====
  const slide11 = pptx.addSlide();
  slide11.addText('AWS ECS Fargate - Frontend Sizing', {
    x: 0.5, y: 0.4, w: 9, h: 0.6,
    fontSize: 32, bold: true, color: colors.primary
  });
  
  slide11.addText('React Application Containerization & Auto-Scaling Configuration', {
    x: 0.5, y: 1.0, w: 9, h: 0.3,
    fontSize: 14, color: '64748B'
  });
  
  // Container Specifications
  slide11.addShape(pptx.ShapeType.roundRect, {
    x: 0.5, y: 1.5, w: 4.3, h: 1.5,
    fill: { color: 'F0F9FF' }
  });
  
  slide11.addText('Container Specifications', {
    x: 0.5, y: 1.6, w: 4.3, h: 0.4,
    fontSize: 16, bold: true, color: colors.cyan, align: 'center'
  });
  
  const containerSpecs = [
    ['Parameter', 'Value'],
    ['vCPU per Task', '0.5 vCPU'],
    ['Memory per Task', '1024 MB (1 GB)'],
    ['Requests per Task', '~100 req/sec'],
    ['OS', 'Linux (Amazon Linux 2)']
  ];
  
  slide11.addTable(containerSpecs, {
    x: 0.6, y: 2.1, w: 4.1, h: 0.8,
    fontSize: 10,
    border: { pt: 1, color: 'BAE6FD' },
    fill: { color: colors.white },
    color: colors.dark,
    valign: 'middle'
  });
  
  // Task Sizing Table
  slide11.addShape(pptx.ShapeType.roundRect, {
    x: 5.0, y: 1.5, w: 4.5, h: 1.5,
    fill: { color: 'F0FDF4' }
  });
  
  slide11.addText('Task Sizing (Formula-Based)', {
    x: 5.0, y: 1.6, w: 4.5, h: 0.4,
    fontSize: 16, bold: true, color: colors.success, align: 'center'
  });
  
  const taskSizing = [
    ['Config', 'Tasks', 'vCPU', 'Memory', 'Capacity'],
    ['Minimum (HA)', '3', '1.5', '3 GB', '2,142 users'],
    ['Recommended', '6', '3.0', '6 GB', '4,284 users'],
    ['Peak Load', '9', '4.5', '9 GB', '6,426 users']
  ];
  
  slide11.addTable(taskSizing, {
    x: 5.1, y: 2.1, w: 4.3, h: 0.8,
    fontSize: 9,
    border: { pt: 1, color: 'BBF7D0' },
    fill: { color: colors.white },
    color: colors.dark,
    valign: 'middle',
    align: 'center'
  });
  
  // Auto-Scaling Configuration
  slide11.addText('Auto-Scaling Configuration', {
    x: 0.5, y: 3.2, w: 9, h: 0.4,
    fontSize: 18, bold: true, color: colors.secondary
  });
  
  const autoScalingData = [
    ['Parameter', 'Value', 'Purpose'],
    ['Minimum Tasks', '3 tasks', 'HA across 3 AZs (us-east-1a/b/c)'],
    ['Desired Tasks', '6 tasks', 'Normal operation capacity'],
    ['Maximum Tasks', '9 tasks', 'Peak load / traffic spike handling'],
    ['Scale Out Trigger', 'CPU > 70%', 'Add task when CPU exceeds threshold'],
    ['Scale In Trigger', 'CPU < 40%', 'Remove task to save cost'],
    ['Scale Out Cooldown', '60 seconds', 'Wait before next scale out'],
    ['Scale In Cooldown', '300 seconds', 'Wait before scale in (safety)']
  ];
  
  slide11.addTable(autoScalingData, {
    x: 0.5, y: 3.7, w: 9, h: 1.4,
    fontSize: 10,
    border: { pt: 1, color: 'E5E7EB' },
    fill: { color: 'F9FAFB' },
    color: colors.dark,
    valign: 'middle',
    colW: [2.0, 1.5, 5.5]
  });
  
  // Monthly Cost Estimate
  slide11.addShape(pptx.ShapeType.roundRect, {
    x: 0.5, y: 5.3, w: 9, h: 1.2,
    fill: { color: '0EA5E9' }
  });
  
  slide11.addText('💰 Monthly Cost Estimate (AWS - Recommended Config: 6 Tasks)', {
    x: 0.5, y: 5.4, w: 9, h: 0.4,
    fontSize: 16, bold: true, color: colors.white, align: 'center'
  });
  
  slide11.addText([
    { text: 'vCPU Cost: ', options: { bold: true } },
    { text: '$60.70  |  ' },
    { text: 'Memory Cost: ', options: { bold: true } },
    { text: '$13.54  |  ' },
    { text: 'ALB: ', options: { bold: true } },
    { text: '$16.20  |  ' },
    { text: 'Data Transfer: ', options: { bold: true } },
    { text: '$18  |  ' },
    { text: 'CloudWatch: ', options: { bold: true } },
    { text: '$10' }
  ], {
    x: 0.5, y: 5.85, w: 9, h: 0.3,
    fontSize: 12, color: colors.white, align: 'center'
  });
  
  slide11.addText('TOTAL: ~$118/month (scales between $90-150 based on traffic)', {
    x: 0.5, y: 6.15, w: 9, h: 0.25,
    fontSize: 11, bold: true, color: 'FDE047', align: 'center'
  });

  // ===== SLIDE 12: On-Premise IIS Server Sizing =====
  const slide12 = pptx.addSlide();
  slide12.addText('On-Premise IIS Servers - Backend Sizing', {
    x: 0.5, y: 0.4, w: 9, h: 0.6,
    fontSize: 32, bold: true, color: colors.primary
  });
  
  slide12.addText('.NET Core Web API Capacity Planning with Entity Framework Core', {
    x: 0.5, y: 1.0, w: 9, h: 0.3,
    fontSize: 14, color: '64748B'
  });
  
  // API Inventory
  slide12.addShape(pptx.ShapeType.roundRect, {
    x: 0.5, y: 1.5, w: 4.3, h: 1.6,
    fill: { color: 'FEF3C7' }
  });
  
  slide12.addText('API Endpoint Inventory', {
    x: 0.5, y: 1.6, w: 4.3, h: 0.4,
    fontSize: 16, bold: true, color: 'D97706', align: 'center'
  });
  
  const apiInventory = [
    ['Screen', 'Endpoints', 'Complexity', 'Avg Response'],
    ['Dashboard', '4', 'Medium', '200 ms'],
    ['Transaction Overview', '3', 'Low', '150 ms'],
    ['Transaction Details', '18', 'High', '300 ms'],
    ['TOTAL', '25 APIs', '', '250 ms avg']
  ];
  
  slide12.addTable(apiInventory, {
    x: 0.6, y: 2.1, w: 4.1, h: 0.9,
    fontSize: 10,
    border: { pt: 1, color: 'FCD34D' },
    fill: { color: colors.white },
    color: colors.dark,
    valign: 'middle',
    align: 'center'
  });
  
  // IIS Server Specifications
  slide12.addShape(pptx.ShapeType.roundRect, {
    x: 5.0, y: 1.5, w: 4.5, h: 1.6,
    fill: { color: 'DBEAFE' }
  });
  
  slide12.addText('IIS Server Specifications', {
    x: 5.0, y: 1.6, w: 4.5, h: 0.4,
    fontSize: 16, bold: true, color: colors.primary, align: 'center'
  });
  
  const iisSpecs = [
    ['Parameter', 'Value'],
    ['CPU Cores', '8 cores per server'],
    ['RAM', '32 GB per server'],
    ['Application Pools', '3 pools (1 per screen)'],
    ['Worker Processes', '4 per pool (Web Garden)'],
    ['Max Concurrent Requests', '5,000 per process']
  ];
  
  slide12.addTable(iisSpecs, {
    x: 5.1, y: 2.1, w: 4.3, h: 0.9,
    fontSize: 10,
    border: { pt: 1, color: '93C5FD' },
    fill: { color: colors.white },
    color: colors.dark,
    valign: 'middle'
  });
  
  // Capacity Calculations
  slide12.addText('Capacity Calculations (Formula-Based)', {
    x: 0.5, y: 3.3, w: 9, h: 0.4,
    fontSize: 18, bold: true, color: colors.secondary
  });
  
  const capacityCalc = [
    ['Configuration', 'Servers', 'Workers', 'Max Requests', 'Effective (70%)', 'RPS Capacity', 'Status'],
    ['Single Server', '1', '12', '60,000', '42,000', '168 req/sec', '✅ Sufficient'],
    ['HA Configuration', '2', '24', '120,000', '84,000', '336 req/sec', '✅ Excellent'],
    ['Peak Load', '3', '36', '180,000', '126,000', '504 req/sec', '✅ Over-provisioned']
  ];
  
  slide12.addTable(capacityCalc, {
    x: 0.5, y: 3.8, w: 9, h: 0.8,
    fontSize: 9,
    border: { pt: 1, color: 'E5E7EB' },
    fill: { color: 'F9FAFB' },
    color: colors.dark,
    valign: 'middle',
    align: 'center',
    colW: [2.0, 1.0, 1.0, 1.5, 1.5, 1.5, 1.5]
  });
  
  // Database & Caching
  slide12.addShape(pptx.ShapeType.roundRect, {
    x: 0.5, y: 4.8, w: 4.3, h: 1.6,
    fill: { color: 'F3E8FF' }
  });
  
  slide12.addText('🗄️ Database Layer', {
    x: 0.5, y: 4.9, w: 4.3, h: 0.4,
    fontSize: 16, bold: true, color: '9333EA', align: 'center'
  });
  
  const dbLayer = [
    ['Component', 'Configuration'],
    ['SQL Server', '2019/2022 Enterprise'],
    ['CPU/RAM', '16 cores, 64 GB RAM'],
    ['EF Core', 'Version 8.0 (Latest LTS)'],
    ['HA', 'Always On Availability Groups'],
    ['Connection Pool', '100 per app pool']
  ];
  
  slide12.addTable(dbLayer, {
    x: 0.6, y: 5.4, w: 4.1, h: 0.9,
    fontSize: 10,
    border: { pt: 1, color: 'E9D5FF' },
    fill: { color: colors.white },
    color: colors.dark,
    valign: 'middle'
  });
  
  slide12.addShape(pptx.ShapeType.roundRect, {
    x: 5.0, y: 4.8, w: 4.5, h: 1.6,
    fill: { color: 'FEE2E2' }
  });
  
  slide12.addText('⚡ Redis Caching Layer', {
    x: 5.0, y: 4.9, w: 4.5, h: 0.4,
    fontSize: 16, bold: true, color: 'DC2626', align: 'center'
  });
  
  const cacheLayer = [
    ['Component', 'Configuration'],
    ['Redis Version', 'Latest stable + Sentinel HA'],
    ['Memory', '16 GB (in-memory cache)'],
    ['Cache Hit Rate', '60% (expected)'],
    ['Response Time', '20 ms (vs 150 ms DB)'],
    ['Strategy', 'Cache-Aside pattern'],
    ['TTL', '15-30 minutes']
  ];
  
  slide12.addTable(cacheLayer, {
    x: 5.1, y: 5.4, w: 4.3, h: 0.9,
    fontSize: 9,
    border: { pt: 1, color: 'FECACA' },
    fill: { color: colors.white },
    color: colors.dark,
    valign: 'middle'
  });

  // ===== SLIDE 13: High Availability & Architecture Diagram =====
  const slide13 = pptx.addSlide();
  slide13.background = { color: '1E293B' };
  
  slide13.addText('High Availability Architecture', {
    x: 0.5, y: 0.4, w: 9, h: 0.6,
    fontSize: 32, bold: true, color: colors.white, align: 'center'
  });
  
  slide13.addText('Multi-AZ Frontend + Clustered Backend + Database Replication', {
    x: 0.5, y: 1.0, w: 9, h: 0.3,
    fontSize: 14, color: colors.light, align: 'center'
  });
  
  // Architecture Diagram (Text-based)
  slide13.addShape(pptx.ShapeType.roundRect, {
    x: 0.5, y: 1.5, w: 9, h: 3.5,
    fill: { color: colors.white }
  });
  
  slide13.addText('🌐 INTERNET / USERS', {
    x: 4, y: 1.7, w: 2, h: 0.3,
    fontSize: 12, bold: true, color: colors.primary, align: 'center',
    fill: { color: 'DBEAFE' }
  });
  
  // AWS Cloud Section
  slide13.addShape(pptx.ShapeType.roundRect, {
    x: 0.7, y: 2.2, w: 8.6, h: 1.5,
    fill: { color: 'FEF3C7' },
    line: { color: 'F59E0B', width: 2 }
  });
  
  slide13.addText('☁️ AWS CLOUD (us-east-1)', {
    x: 0.8, y: 2.3, w: 2, h: 0.3,
    fontSize: 11, bold: true, color: 'D97706'
  });
  
  slide13.addText('Route 53 DNS → ALB (SSL + WAF)', {
    x: 1, y: 2.7, w: 8, h: 0.25,
    fontSize: 10, color: colors.dark, align: 'center'
  });
  
  slide13.addText('ECS Task AZ-a  |  ECS Task AZ-b  |  ECS Task AZ-c', {
    x: 1, y: 3.05, w: 8, h: 0.25,
    fontSize: 9, color: colors.dark, align: 'center',
    fill: { color: '93C5FD' }
  });
  
  slide13.addText('0.5 vCPU, 1GB     |     0.5 vCPU, 1GB     |     0.5 vCPU, 1GB', {
    x: 1, y: 3.3, w: 8, h: 0.2,
    fontSize: 8, color: '1E40AF', align: 'center'
  });
  
  slide13.addText('↓ AWS Direct Connect 1Gbps + VPN Backup ↓', {
    x: 3, y: 3.85, w: 4, h: 0.25,
    fontSize: 10, bold: true, color: colors.white, align: 'center',
    fill: { color: colors.secondary }
  });
  
  // On-Premise Section
  slide13.addShape(pptx.ShapeType.roundRect, {
    x: 0.7, y: 4.3, w: 8.6, h: 1.5,
    fill: { color: 'DBEAFE' },
    line: { color: colors.primary, width: 2 }
  });
  
  slide13.addText('🏢 ON-PREMISE DATA CENTER', {
    x: 0.8, y: 4.4, w: 2.5, h: 0.3,
    fontSize: 11, bold: true, color: colors.primary
  });
  
  slide13.addText('F5 Load Balancer / HAProxy', {
    x: 1, y: 4.8, w: 8, h: 0.25,
    fontSize: 10, color: colors.dark, align: 'center'
  });
  
  slide13.addText('IIS Server 1 (8c, 32GB)   |   IIS Server 2 (8c, 32GB)', {
    x: 1.5, y: 5.15, w: 7, h: 0.25,
    fontSize: 9, color: colors.dark, align: 'center',
    fill: { color: 'A5F3FC' }
  });
  
  slide13.addText('Redis Cache (16GB + Sentinel)  →  SQL Server 2022 (16c, 64GB + Always On AG)', {
    x: 1.5, y: 5.5, w: 7, h: 0.25,
    fontSize: 9, color: colors.dark, align: 'center',
    fill: { color: 'C4B5FD' }
  });
  
  // HA Table
  slide13.addText('High Availability Configuration', {
    x: 0.5, y: 6.1, w: 9, h: 0.3,
    fontSize: 16, bold: true, color: colors.white
  });
  
  const haConfig = [
    ['Component', 'HA Setup', 'Failover Time', 'Method'],
    ['Frontend (ECS)', '3 tasks across 3 AZs', '< 30 sec', 'ALB auto-routing'],
    ['Backend (IIS)', '2+ servers + LB', '< 1 min', 'F5/HAProxy failover'],
    ['SQL Server', 'Always On AG', '< 30 sec', 'Automatic failover'],
    ['Redis', 'Sentinel (3 nodes)', '< 10 sec', 'Sentinel auto-failover']
  ];
  
  slide13.addTable(haConfig, {
    x: 0.5, y: 6.5, w: 9, h: 0.8,
    fontSize: 9,
    border: { pt: 1, color: '475569' },
    fill: { color: '334155' },
    color: colors.white,
    valign: 'middle',
    align: 'center'
  });

  // ===== SLIDE 14: Deployment Timeline & Recommendations =====
  const slide14 = pptx.addSlide();
  slide14.addText('Deployment Timeline & Recommendations', {
    x: 0.5, y: 0.4, w: 9, h: 0.6,
    fontSize: 32, bold: true, color: colors.primary
  });
  
  slide14.addText('Infrastructure Setup to Production Rollout', {
    x: 0.5, y: 1.0, w: 9, h: 0.3,
    fontSize: 14, color: '64748B'
  });
  
  // Deployment Phases
  const deploymentPhases = [
    ['Phase', 'Duration', 'Activities', 'Dependencies'],
    ['1. Infrastructure Setup', '2 weeks', 'AWS account, VPC, ECS cluster, Direct Connect setup', 'Network team + AWS TAM'],
    ['2. Frontend Deployment', '1 week', 'Docker build, ECR push, ECS service deploy, ALB config', 'Phase 1 complete'],
    ['3. Backend API Deploy', '3 weeks', 'IIS setup, SQL migration, EF Core deploy, Redis config', 'Backend dev complete'],
    ['4. Integration Testing', '2 weeks', 'E2E testing, performance testing, security audit', 'Phase 2 + 3 complete'],
    ['5. Production Rollout', '1 week', 'Blue-green deployment, DNS cutover, monitoring setup', 'Phase 4 sign-off'],
    ['TOTAL', '9 weeks', '~2 months (excluding dev time)', 'All phases sequential']
  ];
  
  slide14.addTable(deploymentPhases, {
    x: 0.5, y: 1.5, w: 9, h: 1.5,
    fontSize: 9,
    border: { pt: 1, color: 'E5E7EB' },
    fill: { color: 'F9FAFB' },
    color: colors.dark,
    valign: 'middle',
    colW: [2.5, 1.2, 3.5, 1.8]
  });
  
  // Recommendations
  slide14.addText('🎯 Final Recommendations', {
    x: 0.5, y: 3.2, w: 9, h: 0.4,
    fontSize: 18, bold: true, color: colors.secondary
  });
  
  const recommendations = [
    ['Priority', 'Recommendation', 'Rationale'],
    ['HIGH', 'Deploy 6 ECS tasks across 3 AZs', 'Ensures no single point of failure + handles traffic'],
    ['HIGH', '2 IIS servers in HA cluster', 'Backend redundancy with F5/HAProxy load balancing'],
    ['HIGH', 'Redis caching with 60%+ hit rate', 'Reduces database load by ~60%, faster response'],
    ['MEDIUM', 'Auto-scaling: 3 min, 9 max tasks', 'Handle traffic spikes without over-provisioning'],
    ['MEDIUM', 'AWS Direct Connect 1 Gbps', 'Low-latency, reliable AWS-to-On-Prem connectivity'],
    ['LOW', 'CloudFront CDN (optional)', 'Further reduce latency for static assets']
  ];
  
  slide14.addTable(recommendations, {
    x: 0.5, y: 3.7, w: 9, h: 1.3,
    fontSize: 10,
    border: { pt: 1, color: 'E5E7EB' },
    fill: { color: 'F9FAFB' },
    color: colors.dark,
    valign: 'middle',
    colW: [1.2, 3.5, 4.3]
  });
  
  // Cost Summary
  slide14.addShape(pptx.ShapeType.roundRect, {
    x: 0.5, y: 5.2, w: 4.3, h: 1.3,
    fill: { color: '10B981' }
  });
  
  slide14.addText('💰 Monthly Cost Summary', {
    x: 0.5, y: 5.3, w: 4.3, h: 0.4,
    fontSize: 16, bold: true, color: colors.white, align: 'center'
  });
  
  slide14.addText([
    { text: 'AWS Frontend: ', options: { bold: true } },
    { text: '$118/month (recommended)\n' },
    { text: 'On-Prem Backend: ', options: { bold: true } },
    { text: 'Existing infrastructure\n' },
    { text: 'Direct Connect: ', options: { bold: true } },
    { text: '~$200/month (prorated)' }
  ], {
    x: 0.7, y: 5.8, w: 3.9, h: 0.6,
    fontSize: 11, color: colors.white
  });
  
  // Performance SLA
  slide14.addShape(pptx.ShapeType.roundRect, {
    x: 5.2, y: 5.2, w: 4.3, h: 1.3,
    fill: { color: colors.secondary }
  });
  
  slide14.addText('📊 Performance SLA Targets', {
    x: 5.2, y: 5.3, w: 4.3, h: 0.4,
    fontSize: 16, bold: true, color: colors.white, align: 'center'
  });
  
  slide14.addText([
    { text: 'Page Load Time: ', options: { bold: true } },
    { text: '< 2 seconds (P95)\n' },
    { text: 'API Response: ', options: { bold: true } },
    { text: '< 300 ms (P95)\n' },
    { text: 'Availability: ', options: { bold: true } },
    { text: '99.5% monthly uptime\n' },
    { text: 'Concurrent Users: ', options: { bold: true } },
    { text: '4,284 users supported' }
  ], {
    x: 5.4, y: 5.8, w: 3.9, h: 0.6,
    fontSize: 10, color: colors.white
  });

  // Save the presentation
  pptx.writeFile({ fileName: 'RepairLink_Complete_FullStack_Estimates.pptx' });
};