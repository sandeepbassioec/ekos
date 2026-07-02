import pptxgen from 'pptxgenjs';

export const generatePowerPointFile = () => {
  const pptx = new pptxgen();

  pptx.author = 'RepairLink Estimation Team';
  pptx.company = 'RepairLink';
  pptx.title = 'RepairLink Development Estimates - Complete';
  pptx.subject = 'Wide Band Delphi + PERT Analysis - Screens 1-3';

  const colors = {
    primary: '0066CC',
    secondary: '4F46E5',
    success: '10B981',
    warning: 'F59E0B',
    danger: 'EF4444',
    dark: '1F2937',
    light: 'F3F4F6',
    white: 'FFFFFF'
  };

  // ===== SLIDE 1: Title Slide =====
  const slide1 = pptx.addSlide();
  slide1.background = { color: colors.primary };
  
  slide1.addText('RepairLink Application', {
    x: 0.5, y: 1.5, w: 9, h: 1,
    fontSize: 44, bold: true, color: colors.white, align: 'center'
  });
  
  slide1.addText('Development Estimates - Complete', {
    x: 0.5, y: 2.5, w: 9, h: 0.8,
    fontSize: 36, color: colors.white, align: 'center'
  });
  
  slide1.addText('Wide Band Delphi + PERT Analysis', {
    x: 0.5, y: 3.5, w: 9, h: 0.5,
    fontSize: 20, color: colors.light, align: 'center'
  });
  
  slide1.addText('Screens 1-3: Dashboard + Transaction Overview + Transaction Details\nMarch 10, 2026 | Version 3.0', {
    x: 0.5, y: 4.5, w: 9, h: 0.8,
    fontSize: 16, color: colors.light, align: 'center'
  });

  // ===== SLIDE 2: Executive Summary =====
  const slide2 = pptx.addSlide();
  slide2.addText('Executive Summary', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 28, bold: true, color: colors.dark
  });
  
  const summaryData = [
    { title: 'Development', hours: '400 hrs', days: '50 days', color: colors.success },
    { title: 'QA Testing', hours: '315 hrs', days: '39.4 days', color: colors.warning },
    { title: 'Hardening', hours: '305 hrs', days: '38.1 days', color: colors.secondary }
  ];
  
  summaryData.forEach((item, idx) => {
    slide2.addShape(pptx.ShapeType.rect, {
      x: 0.5 + (idx * 3.2), y: 1.0, w: 2.8, h: 1.3,
      fill: { color: item.color }, line: { type: 'none' }
    });
    
    slide2.addText(item.title, {
      x: 0.5 + (idx * 3.2), y: 1.1, w: 2.8, h: 0.3,
      fontSize: 14, bold: true, color: colors.white, align: 'center'
    });
    
    slide2.addText(item.hours, {
      x: 0.5 + (idx * 3.2), y: 1.6, w: 2.8, h: 0.4,
      fontSize: 24, bold: true, color: colors.white, align: 'center'
    });
    
    slide2.addText(item.days, {
      x: 0.5 + (idx * 3.2), y: 2.0, w: 2.8, h: 0.25,
      fontSize: 12, color: colors.white, align: 'center'
    });
  });
  
  slide2.addShape(pptx.ShapeType.rect, {
    x: 1.5, y: 2.7, w: 6.5, h: 1.0,
    fill: { color: colors.primary }, line: { type: 'none' }
  });
  
  slide2.addText('Total: 1,020 hours ≈ 127.5 days (25-26 weeks)', {
    x: 1.5, y: 2.9, w: 6.5, h: 0.3,
    fontSize: 18, bold: true, color: colors.white, align: 'center'
  });
  
  slide2.addText('Confidence: 84% (Conservative) | Timeline: 6-7 months', {
    x: 1.5, y: 3.3, w: 6.5, h: 0.25,
    fontSize: 14, color: colors.white, align: 'center'
  });
  
  slide2.addText('Key Highlights:', {
    x: 0.5, y: 4.1, w: 9, h: 0.3,
    fontSize: 16, bold: true, color: colors.dark
  });
  
  const highlights = [
    '✓ 3 complete screens with full functionality',
    '✓ OEC OUI library provides 40% development time savings',
    '✓ Includes comprehensive testing and hardening',
    '✓ Fully responsive (Mobile, Tablet, Desktop)'
  ];
  
  highlights.forEach((text, idx) => {
    slide2.addText(text, {
      x: 0.8, y: 4.5 + (idx * 0.3), w: 8.5, h: 0.25,
      fontSize: 12, color: colors.dark
    });
  });

  // ===== SLIDE 3: Scope Breakdown =====
  const slide3 = pptx.addSlide();
  slide3.addText('Scope: All 3 Screens', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 28, bold: true, color: colors.dark
  });
  
  const scopeData = [
    ['Screen', 'Description', 'Dev (hrs)', 'QA (hrs)', 'Total (hrs)'],
    ['Screen 1', 'Overview Page - Activity + Analytics', '90', '72', '247'],
    ['Screen 2', 'Transaction Overview - Table + Filters', '125', '98', '318'],
    ['Screen 3', 'Transaction Details - Form + Modals', '185', '145', '455'],
    ['TOTAL', '', '400', '315', '1,020']
  ];
  
  slide3.addTable(scopeData, {
    x: 0.5, y: 1.0, w: 9, h: 2.0,
    fontSize: 11,
    border: { pt: 1, color: 'CCCCCC' },
    fill: { color: colors.light },
    color: colors.dark,
    align: 'center',
    valign: 'middle',
    colW: [1.5, 4.0, 1.5, 1.2, 1.3]
  });
  
  slide3.addText('Screen 1: Overview Page', {
    x: 0.5, y: 3.3, w: 4.5, h: 0.3,
    fontSize: 14, bold: true, color: colors.primary
  });
  
  const screen1Features = [
    '• Activity Dashboard (3 status cards)',
    '• Analytics Dashboard (12 KPI metrics)',
    '• Search & Filter functionality',
    '• Responsive grid layout'
  ];
  
  screen1Features.forEach((text, idx) => {
    slide3.addText(text, {
      x: 0.7, y: 3.7 + (idx * 0.25), w: 4.0, h: 0.2,
      fontSize: 10, color: colors.dark
    });
  });
  
  slide3.addText('Screen 2: Transaction Overview', {
    x: 5.2, y: 3.3, w: 4.5, h: 0.3,
    fontSize: 14, bold: true, color: colors.primary
  });
  
  const screen2Features = [
    '• Data Table (11+ columns with sorting)',
    '• Drag & Drop column customization',
    '• Multi-criteria filters',
    '• Tab navigation (All/Open/Closed)'
  ];
  
  screen2Features.forEach((text, idx) => {
    slide3.addText(text, {
      x: 5.4, y: 3.7 + (idx * 0.25), w: 4.0, h: 0.2,
      fontSize: 10, color: colors.dark
    });
  });
  
  slide3.addText('Screen 3: Transaction Details', {
    x: 0.5, y: 4.9, w: 9, h: 0.3,
    fontSize: 14, bold: true, color: colors.primary
  });
  
  const screen3Features = [
    '• Complex order form (20+ fields)',
    '• Parts table (15+ columns with inline editing)',
    '• 5 modal popups (VIN Details, Order Info, Photos & Notes, Event Log, Date/Time Picker)',
    '• Real-time calculations and validations'
  ];
  
  screen3Features.forEach((text, idx) => {
    slide3.addText(text, {
      x: 0.7, y: 5.3 + (idx * 0.25), w: 8.5, h: 0.2,
      fontSize: 10, color: colors.dark
    });
  });

  // ===== SLIDE 4: Timeline & Resource Plan =====
  const slide4 = pptx.addSlide();
  slide4.addText('Timeline & Resource Requirements', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 28, bold: true, color: colors.dark
  });
  
  const timelineData = [
    ['Phase', 'Duration', 'Start', 'End', 'Resources'],
    ['Development', '50 days', 'Week 1', 'Week 10', '2-3 Developers'],
    ['QA Testing', '39 days', 'Week 8', 'Week 16', '1-2 QA Engineers'],
    ['Hardening', '38 days', 'Week 14', 'Week 22', '1 DevOps + Specialists'],
    ['Buffer', '12 days', 'Week 22', 'Week 24', 'Full Team'],
    ['TOTAL', '~127 days', 'Week 1', 'Week 26', '4-5 FTE']
  ];
  
  slide4.addTable(timelineData, {
    x: 0.5, y: 1.0, w: 9, h: 2.5,
    fontSize: 10,
    border: { pt: 1, color: 'CCCCCC' },
    fill: { color: colors.light },
    color: colors.dark,
    align: 'center',
    valign: 'middle',
    colW: [1.8, 1.5, 1.3, 1.3, 3.1]
  });
  
  slide4.addText('Recommended Team Composition:', {
    x: 0.5, y: 3.8, w: 9, h: 0.3,
    fontSize: 16, bold: true, color: colors.dark
  });
  
  const teamData = [
    ['Role', 'FTE', 'Key Responsibilities'],
    ['Senior Frontend Dev', '1.0', 'Architecture, complex features, code review'],
    ['Mid-Level Frontend Dev', '1.0', 'Component development, integration'],
    ['QA Engineer', '1.0', 'Manual testing, test case design'],
    ['QA Automation Engineer', '0.5', 'Automated test development'],
    ['DevOps/Performance', '0.3', 'Performance tuning, deployment'],
    ['Security Specialist', '0.2', 'Security audit, vulnerability fixes'],
    ['Accessibility Specialist', '0.3', 'WCAG compliance, a11y testing'],
    ['TOTAL', '4.3 FTE', '≈ 4-5 resources recommended']
  ];
  
  slide4.addTable(teamData, {
    x: 0.5, y: 4.2, w: 9, h: 2.5,
    fontSize: 9,
    border: { pt: 1, color: 'CCCCCC' },
    fill: { color: colors.light },
    color: colors.dark,
    align: 'left',
    valign: 'middle',
    colW: [2.2, 1.0, 5.8]
  });

  // ===== SLIDE 5: Risk Assessment =====
  const slide5 = pptx.addSlide();
  slide5.addText('Risk Assessment & Mitigation', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 28, bold: true, color: colors.dark
  });
  
  const riskData = [
    ['Risk', 'Impact', 'Prob', 'Mitigation'],
    ['Scope Creep', 'High', 'High', 'Formal change process'],
    ['API Integration Delays', 'High', 'Med', 'Mock data, parallel dev'],
    ['OEC OUI Component Gaps', 'Med', 'Med', 'Custom dev budget'],
    ['Performance Issues', 'Med', 'High', 'Virtual scrolling'],
    ['Complex State Management', 'High', 'Med', 'Redux/Zustand'],
    ['Browser Compatibility', 'Low', 'Low', 'Early testing']
  ];
  
  slide5.addTable(riskData, {
    x: 0.5, y: 1.0, w: 9, h: 2.5,
    fontSize: 10,
    border: { pt: 1, color: 'CCCCCC' },
    fill: { color: colors.light },
    color: colors.dark,
    align: 'left',
    valign: 'middle',
    colW: [3.0, 1.2, 1.0, 3.8]
  });
  
  slide5.addText('Confidence Intervals:', {
    x: 0.5, y: 3.8, w: 9, h: 0.3,
    fontSize: 16, bold: true, color: colors.dark
  });
  
  const confidenceData = [
    ['Scenario', 'Timeline', 'Hours', 'Confidence'],
    ['Best Case', '94 days', '750 hrs', '1% (Unlikely)'],
    ['Expected Case', '128 days', '1,020 hrs', '50% (Realistic)'],
    ['Conservative', '137 days', '1,095 hrs', '84% (Recommended)'],
    ['Worst Case', '169 days', '1,350 hrs', '99% (High certainty)']
  ];
  
  slide5.addTable(confidenceData, {
    x: 1.0, y: 4.2, w: 8, h: 2.0,
    fontSize: 11,
    border: { pt: 1, color: 'CCCCCC' },
    fill: { color: colors.light },
    color: colors.dark,
    align: 'center',
    valign: 'middle',
    colW: [2.0, 1.8, 1.8, 2.4]
  });

  // ===== SLIDE 6: Quality Standards =====
  const slide6 = pptx.addSlide();
  slide6.addText('Quality Standards & Deliverables', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 28, bold: true, color: colors.dark
  });
  
  const qualityData = [
    ['Metric', 'Target', 'Industry Benchmark'],
    ['Code Coverage', '>80%', '70-80%'],
    ['Performance (Lighthouse)', '>90', '85+'],
    ['Accessibility (WCAG AA)', '100', '95+'],
    ['Bug Density', '<0.5/100 LOC', '1-2/100 LOC'],
    ['Security Vulnerabilities', '0 Critical/High', '<5']
  ];
  
  slide6.addTable(qualityData, {
    x: 0.5, y: 1.0, w: 9, h: 2.3,
    fontSize: 11,
    border: { pt: 1, color: 'CCCCCC' },
    fill: { color: colors.light },
    color: colors.dark,
    align: 'center',
    valign: 'middle',
    colW: [3.5, 2.5, 3.0]
  });
  
  slide6.addText('Deliverables Checklist:', {
    x: 0.5, y: 3.6, w: 9, h: 0.3,
    fontSize: 16, bold: true, color: colors.dark
  });
  
  const deliverables = [
    '✓ Fully responsive (Mobile, Tablet, Desktop)',
    '✓ Cross-browser compatible (Chrome, Firefox, Safari, Edge)',
    '✓ WCAG 2.1 AA accessible',
    '✓ Production-ready performance',
    '✓ Secure (XSS, CSRF protection)',
    '✓ Comprehensive test coverage (>80%)',
    '✓ Documented code (TypeScript + JSDoc)',
    '✓ Component library (Storybook)'
  ];
  
  deliverables.forEach((text, idx) => {
    const col = idx % 2;
    const row = Math.floor(idx / 2);
    slide6.addText(text, {
      x: 0.8 + (col * 4.5), y: 4.0 + (row * 0.3), w: 4.0, h: 0.25,
      fontSize: 11, color: colors.dark
    });
  });

  // ===== SLIDE 7: Recommendations =====
  const slide7 = pptx.addSlide();
  slide7.addText('Recommendations', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 28, bold: true, color: colors.dark
  });
  
  slide7.addShape(pptx.ShapeType.rect, {
    x: 0.5, y: 1.0, w: 4.2, h: 2.5,
    fill: { color: 'D1FAE5' }, line: { pt: 2, color: colors.success }
  });
  
  slide7.addText('For Faster Delivery:', {
    x: 0.7, y: 1.2, w: 3.8, h: 0.3,
    fontSize: 14, bold: true, color: colors.success
  });
  
  const deliveryRecs = [
    '1. Invest in Automation',
    '   • Add 93 hrs upfront',
    '   • Save 60% on regression',
    '',
    '2. Parallel Development',
    '   • Dev, QA, Hardening overlap',
    '   • Reduce timeline by 30-40%',
    '',
    '3. Early Prototyping',
    '   • Spike complex features',
    '   • Reduce risk & rework'
  ];
  
  deliveryRecs.forEach((text, idx) => {
    slide7.addText(text, {
      x: 0.8, y: 1.6 + (idx * 0.2), w: 3.6, h: 0.18,
      fontSize: text.startsWith(' ') ? 9 : 10,
      bold: !text.startsWith(' ') && text !== '',
      color: colors.dark
    });
  });
  
  slide7.addShape(pptx.ShapeType.rect, {
    x: 5.2, y: 1.0, w: 4.2, h: 2.5,
    fill: { color: 'DBEAFE' }, line: { pt: 2, color: colors.primary }
  });
  
  slide7.addText('For Cost Optimization:', {
    x: 5.4, y: 1.2, w: 3.8, h: 0.3,
    fontSize: 14, bold: true, color: colors.primary
  });
  
  const costRecs = [
    '1. Manual QA Initially',
    '   • Save 93 hrs upfront',
    '   • Add automation later',
    '',
    '2. Staged Hardening',
    '   • Core hardening first',
    '   • Advanced features post-MVP',
    '',
    '3. Leverage OEC OUI',
    '   • Already saving 40%',
    '   • Train team on library'
  ];
  
  costRecs.forEach((text, idx) => {
    slide7.addText(text, {
      x: 5.5, y: 1.6 + (idx * 0.2), w: 3.6, h: 0.18,
      fontSize: text.startsWith(' ') ? 9 : 10,
      bold: !text.startsWith(' ') && text !== '',
      color: colors.dark
    });
  });
  
  slide7.addShape(pptx.ShapeType.rect, {
    x: 1.0, y: 4.0, w: 8.0, h: 0.9,
    fill: { color: colors.primary }, line: { type: 'none' }
  });
  
  slide7.addText('💡 Recommended Approach', {
    x: 1.0, y: 4.1, w: 8.0, h: 0.3,
    fontSize: 15, bold: true, color: colors.white, align: 'center'
  });
  
  slide7.addText('Plan for 28 weeks with parallel execution and automation investment.\nThis provides 84% confidence with optimal quality.', {
    x: 1.0, y: 4.5, w: 8.0, h: 0.4,
    fontSize: 12, color: colors.white, align: 'center'
  });
  
  slide7.addText('Next Steps:', {
    x: 0.5, y: 5.2, w: 9, h: 0.3,
    fontSize: 14, bold: true, color: colors.dark
  });
  
  const nextSteps = [
    '1. Review & approve estimates',
    '2. Finalize team assignments',
    '3. Project kickoff meeting',
    '4. Sprint planning (2-week sprints)'
  ];
  
  nextSteps.forEach((text, idx) => {
    slide7.addText(text, {
      x: 0.8, y: 5.6 + (idx * 0.25), w: 8.5, h: 0.2,
      fontSize: 11, color: colors.dark
    });
  });

  // ===== SLIDE 8: Summary =====
  const slide8 = pptx.addSlide();
  slide8.background = { color: colors.secondary };
  
  slide8.addText('Summary', {
    x: 0.5, y: 1.0, w: 9, h: 0.6,
    fontSize: 32, bold: true, color: colors.white, align: 'center'
  });
  
  const summaryPoints = [
    { label: 'Timeline:', value: '25-28 weeks (6-7 months)' },
    { label: 'Team Size:', value: '4-5 full-time resources' },
    { label: 'Total Effort:', value: '1,020 hours (Manual QA)' },
    { label: 'Risk Level:', value: 'Medium (well-mitigated)' },
    { label: 'Confidence:', value: '84% (Conservative estimate)' }
  ];
  
  summaryPoints.forEach((point, idx) => {
    slide8.addText(`${point.label} ${point.value}`, {
      x: 1.5, y: 2.2 + (idx * 0.5), w: 7, h: 0.4,
      fontSize: 18, color: colors.white, align: 'center'
    });
  });
  
  slide8.addShape(pptx.ShapeType.rect, {
    x: 2.5, y: 4.8, w: 5, h: 0.7,
    fill: { color: colors.white }, line: { type: 'none' }
  });
  
  slide8.addText('✅ Ready for Approval', {
    x: 2.5, y: 4.95, w: 5, h: 0.4,
    fontSize: 22, bold: true, color: colors.secondary, align: 'center', valign: 'middle'
  });
  
  slide8.addText('Estimation Method: Wide Band Delphi + PERT\nFormula: E = (O + 4×ML + P) / 6', {
    x: 0.5, y: 6.0, w: 9, h: 0.5,
    fontSize: 11, color: colors.light, align: 'center'
  });

  // Save the presentation
  pptx.writeFile({ fileName: 'RepairLink_Complete_Estimates_Screens_1-3.pptx' });
};