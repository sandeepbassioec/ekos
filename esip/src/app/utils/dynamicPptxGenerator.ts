import pptxgen from 'pptxgenjs';
import { EstimationData } from './dynamicExcelGenerator';

export const generateDynamicPowerPointFile = (data: EstimationData) => {
  const { project, screens, results, riskFactors, contingency, selectedMethods } = data;
  const pptx = new pptxgen();

  pptx.author = 'Estimation Wizard';
  pptx.company = project.name;
  pptx.title = `${project.name} - Development Estimates`;
  pptx.subject = `${selectedMethods.join(', ')} Analysis`;

  const colors = {
    primary: '1E3A8A',    // Blue 900
    secondary: '475569',  // Gray 600
    success: '059669',    // Green 600
    warning: 'D97706',    // Amber 600
    danger: 'DC2626',     // Red 600
    white: 'FFFFFF',
    light: 'F3F4F6',      // Gray 100
  };

  // ===== SLIDE 1: Title Slide =====
  const slide1 = pptx.addSlide();
  slide1.background = { color: colors.primary };
  
  slide1.addText(project.name, {
    x: 0.5, y: 1.5, w: 9, h: 1.0,
    fontSize: 44, bold: true, color: colors.white, align: 'center'
  });
  
  slide1.addText('Development Estimation Report', {
    x: 0.5, y: 2.7, w: 9, h: 0.6,
    fontSize: 28, color: colors.white, align: 'center'
  });
  
  const currentDate = new Date().toLocaleDateString('en-US', { 
    year: 'numeric', 
    month: 'long', 
    day: 'numeric' 
  });
  
  slide1.addText([
    { text: `Project Type: ${project.type}\n`, options: { fontSize: 16, color: colors.white } },
    { text: `Estimation Methods: ${selectedMethods.join(', ')}\n`, options: { fontSize: 16, color: colors.white } },
    { text: `Date: ${currentDate}`, options: { fontSize: 16, color: colors.white } }
  ], {
    x: 0.5, y: 3.8, w: 9, h: 1.5,
    align: 'center'
  });

  // ===== SLIDE 2: Executive Summary =====
  const slide2 = pptx.addSlide();
  slide2.addText('Executive Summary', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 32, bold: true, color: colors.primary
  });

  const summaryData = [
    { label: 'Total Effort', value: `${results.totalHours} hours`, color: colors.primary },
    { label: 'Total Cost', value: `$${results.totalCost.toLocaleString()}`, color: colors.success },
    { label: 'Timeline', value: `${results.timelineWeeks} weeks`, color: colors.warning }
  ];
  
  summaryData.forEach((item, idx) => {
    slide2.addShape(pptx.ShapeType.rect, {
      x: 0.5 + (idx * 3.2), y: 1.0, w: 2.8, h: 1.3,
      fill: { color: item.color }, line: { type: 'none' }
    });
    
    slide2.addText(item.label, {
      x: 0.5 + (idx * 3.2), y: 1.1, w: 2.8, h: 0.4,
      fontSize: 14, color: colors.white, align: 'center', bold: true
    });
    
    slide2.addText(item.value, {
      x: 0.5 + (idx * 3.2), y: 1.6, w: 2.8, h: 0.6,
      fontSize: 24, color: colors.white, align: 'center', bold: true
    });
  });
  
  // Key Metrics
  slide2.addText('Key Metrics', {
    x: 0.5, y: 2.6, w: 9, h: 0.4,
    fontSize: 18, bold: true, color: colors.primary
  });
  
  const metricsTable = [
    ['Metric', 'Value'],
    ['Team Size', `${project.teamSize} developers`],
    ['Hourly Rate', `$${project.hourlyRate}/hour`],
    ['Screens/Features', `${screens.length} items`],
    ['Contingency Buffer', `${contingency}% (${riskFactors.length} risks)`],
    ['Confidence Level', `${results.confidence}%`],
    ['Tech Stack', project.techStack.join(', ') || 'Not specified'],
  ];
  
  slide2.addTable(metricsTable, {
    x: 1.5, y: 3.1, w: 7.0, h: 2.0,
    fontSize: 12,
    colW: [2.5, 4.5],
    border: { pt: 1, color: colors.secondary },
    fill: { color: colors.light },
    color: colors.secondary,
    rowH: 0.28,
    valign: 'middle',
  });

  // ===== SLIDE 3: Effort Breakdown =====
  const slide3 = pptx.addSlide();
  slide3.addText('Effort Breakdown', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 32, bold: true, color: colors.primary
  });

  const breakdownData = [
    { label: 'Development', hours: results.breakdown.development, percent: 50, color: colors.primary },
    { label: 'Testing (QA)', hours: results.breakdown.testing, percent: 30, color: colors.success },
    { label: 'Hardening', hours: results.breakdown.hardening, percent: 20, color: colors.warning }
  ];

  breakdownData.forEach((phase, idx) => {
    const y = 1.2 + (idx * 1.2);
    
    slide3.addText(phase.label, {
      x: 0.5, y: y, w: 2.5, h: 0.5,
      fontSize: 16, bold: true, color: colors.secondary
    });
    
    slide3.addShape(pptx.ShapeType.rect, {
      x: 3.0, y: y + 0.05, w: (phase.percent / 100) * 6.0, h: 0.4,
      fill: { color: phase.color }, line: { type: 'none' }
    });
    
    slide3.addText(`${phase.hours} hours (${phase.percent}%)`, {
      x: 3.0, y: y + 0.05, w: 6.0, h: 0.4,
      fontSize: 14, color: colors.white, align: 'center', bold: true
    });
  });

  // Timeline calculation
  slide3.addText('Timeline Calculation', {
    x: 0.5, y: 4.3, w: 9, h: 0.4,
    fontSize: 18, bold: true, color: colors.primary
  });

  const timelineTable = [
    ['Component', 'Value'],
    ['Total Hours', `${results.totalHours} hours`],
    ['Team Size', `${project.teamSize} developers`],
    ['Weekly Capacity', `${project.teamSize * 40} hours/week`],
    ['Estimated Duration', `${results.timelineWeeks} weeks`],
    ['Calendar Months', `~${Math.ceil(results.timelineWeeks / 4)} months`],
  ];

  slide3.addTable(timelineTable, {
    x: 2.0, y: 4.8, w: 6.0, h: 1.5,
    fontSize: 12,
    colW: [3.0, 3.0],
    border: { pt: 1, color: colors.secondary },
    fill: { color: colors.light },
    color: colors.secondary,
    rowH: 0.25,
    valign: 'middle',
  });

  // ===== SLIDE 4: Scope Overview =====
  const slide4 = pptx.addSlide();
  slide4.addText('Scope Overview', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 32, bold: true, color: colors.primary
  });

  slide4.addText(`Total: ${screens.length} Features/Screens`, {
    x: 0.5, y: 0.9, w: 9, h: 0.4,
    fontSize: 16, color: colors.secondary
  });

  // Scope summary table (first 10 items)
  const scopeTableData = [
    ['#', 'Name', 'Type', 'Complexity', 'Level']
  ];

  screens.slice(0, 10).forEach((screen, idx) => {
    const indent = '  '.repeat(screen.level || 0);
    scopeTableData.push([
      `${idx + 1}`,
      indent + screen.name,
      screen.type,
      screen.complexity,
      `${screen.level || 0}`
    ]);
  });

  if (screens.length > 10) {
    scopeTableData.push(['...', `+ ${screens.length - 10} more items`, '', '', '']);
  }

  slide4.addTable(scopeTableData, {
    x: 0.5, y: 1.4, w: 9.0, h: 3.5,
    fontSize: 11,
    colW: [0.5, 5.0, 1.5, 1.2, 0.8],
    border: { pt: 1, color: colors.secondary },
    fill: { color: colors.light },
    color: colors.secondary,
    rowH: 0.35,
    valign: 'middle',
  });

  // Summary by type
  const typeCount: Record<string, number> = {};
  screens.forEach(s => {
    typeCount[s.type] = (typeCount[s.type] || 0) + 1;
  });

  let summaryText = 'Summary by Type: ';
  Object.entries(typeCount).forEach(([type, count], idx) => {
    summaryText += `${type}: ${count}${idx < Object.entries(typeCount).length - 1 ? ' | ' : ''}`;
  });

  slide4.addText(summaryText, {
    x: 0.5, y: 5.2, w: 9, h: 0.4,
    fontSize: 12, color: colors.secondary, italic: true
  });

  // ===== SLIDE 5: Risk Assessment =====
  const slide5 = pptx.addSlide();
  slide5.addText('Risk Assessment & Contingency', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 32, bold: true, color: colors.primary
  });

  slide5.addText(`Contingency Buffer: ${contingency}% | ${riskFactors.length} Risks Identified`, {
    x: 0.5, y: 0.9, w: 9, h: 0.4,
    fontSize: 16, color: colors.secondary
  });

  // Risk table (first 8 risks)
  const riskTableData = [
    ['Risk Name', 'Impact', 'Probability', 'Score']
  ];

  const impactWeights: Record<string, number> = { LOW: 1, MEDIUM: 2, HIGH: 3 };

  riskFactors.slice(0, 8).forEach(risk => {
    const riskScore = impactWeights[risk.impact] * (risk.probability / 100);
    riskTableData.push([
      risk.name,
      risk.impact,
      `${risk.probability}%`,
      riskScore.toFixed(2)
    ]);
  });

  if (riskFactors.length > 8) {
    riskTableData.push([`+ ${riskFactors.length - 8} more risks`, '', '', '']);
  }

  slide5.addTable(riskTableData, {
    x: 0.5, y: 1.4, w: 9.0, h: 3.0,
    fontSize: 11,
    colW: [5.0, 1.5, 1.5, 1.0],
    border: { pt: 1, color: colors.secondary },
    fill: { color: colors.light },
    color: colors.secondary,
    rowH: 0.33,
    valign: 'middle',
  });

  // Contingency calculation
  const totalRiskScore = riskFactors.reduce(
    (sum, r) => sum + impactWeights[r.impact] * (r.probability / 100),
    0
  );
  const baseHours = Math.round(results.totalHours / (1 + contingency / 100));
  const contingencyHours = results.totalHours - baseHours;

  slide5.addShape(pptx.ShapeType.rect, {
    x: 1.5, y: 4.7, w: 7.0, h: 1.0,
    fill: { color: colors.warning }, line: { pt: 2, color: colors.warning }
  });

  slide5.addText([
    { text: `Total Risk Score: ${totalRiskScore.toFixed(2)} | `, options: { fontSize: 14, bold: true, color: colors.white } },
    { text: `Base: ${baseHours}h + Contingency: ${contingencyHours}h = Total: ${results.totalHours}h`, options: { fontSize: 14, color: colors.white } }
  ], {
    x: 1.5, y: 4.9, w: 7.0, h: 0.6,
    align: 'center'
  });

  // ===== SLIDE 6: Estimation Methods =====
  const slide6 = pptx.addSlide();
  slide6.addText('Estimation Methodology', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 32, bold: true, color: colors.primary
  });

  slide6.addText(`Methods Used: ${selectedMethods.join(', ')}`, {
    x: 0.5, y: 0.9, w: 9, h: 0.4,
    fontSize: 16, color: colors.secondary
  });

  const methodDescriptions: Record<EstimationMethod, string> = {
    PERT: 'Three-Point Estimation using Optimistic, Most Likely, and Pessimistic estimates. Formula: (O + 4M + P) / 6',
    STORY_POINTS: 'Agile estimation using Fibonacci sequence. Default conversion: 1 point = 8 hours',
    TSHIRT: 'Relative sizing from XS (8h) to XXL (200h). Simple and quick estimation for early stages',
    FPA: 'Function Point Analysis - standardized method for software measurement',
    UCP: 'Use Case Points - measures application from user perspective',
  };

  let y = 1.5;
  selectedMethods.forEach(method => {
    slide6.addText(`${method}:`, {
      x: 0.5, y: y, w: 9, h: 0.3,
      fontSize: 14, bold: true, color: colors.primary
    });

    slide6.addText(methodDescriptions[method], {
      x: 0.8, y: y + 0.35, w: 8.4, h: 0.6,
      fontSize: 12, color: colors.secondary
    });

    y += 1.0;
  });

  // Confidence level explanation
  slide6.addShape(pptx.ShapeType.rect, {
    x: 1.0, y: 4.5, w: 8.0, h: 1.2,
    fill: { color: colors.light }, line: { pt: 2, color: colors.primary }
  });

  slide6.addText([
    { text: `Confidence Level: ${results.confidence}%\n`, options: { fontSize: 16, bold: true, color: colors.primary } },
    { text: 'Based on estimation method accuracy, scope clarity, and team experience', options: { fontSize: 12, color: colors.secondary } }
  ], {
    x: 1.2, y: 4.7, w: 7.6, h: 0.8,
    valign: 'middle'
  });

  // ===== SLIDE 7: Recommendations =====
  const slide7 = pptx.addSlide();
  slide7.addText('Recommendations', {
    x: 0.5, y: 0.3, w: 9, h: 0.5,
    fontSize: 32, bold: true, color: colors.primary
  });

  const recommendations = [
    '• Begin with detailed requirements gathering and design',
    `• Allocate ${project.teamSize} developers for ${results.timelineWeeks} weeks`,
    `• Budget: $${results.totalCost.toLocaleString()} including ${contingency}% contingency`,
    '• Conduct regular sprint reviews and retrospectives',
    '• Maintain continuous integration and automated testing',
    '• Schedule weekly stakeholder check-ins',
    '• Plan for 2-week sprints with clear deliverables',
    `• Consider ${riskFactors.length} identified risks in project planning`,
  ];

  let recY = 1.2;
  recommendations.forEach(rec => {
    slide7.addText(rec, {
      x: 0.5, y: recY, w: 9, h: 0.4,
      fontSize: 14, color: colors.secondary
    });
    recY += 0.5;
  });

  slide7.addShape(pptx.ShapeType.rect, {
    x: 1.0, y: 4.8, w: 8.0, h: 0.9,
    fill: { color: colors.success }, line: { type: 'none' }
  });

  slide7.addText([
    { text: '✓ Ready to Proceed\n', options: { fontSize: 18, bold: true, color: colors.white } },
    { text: 'This estimate provides a solid foundation for project planning and resource allocation', options: { fontSize: 12, color: colors.white } }
  ], {
    x: 1.0, y: 4.9, w: 8.0, h: 0.7,
    align: 'center', valign: 'middle'
  });

  // ===== SLIDE 8: Next Steps =====
  const slide8 = pptx.addSlide();
  slide8.background = { color: colors.primary };
  
  slide8.addText('Next Steps', {
    x: 0.5, y: 1.5, w: 9, h: 0.8,
    fontSize: 36, bold: true, color: colors.white, align: 'center'
  });

  const nextSteps = [
    '1. Review and approve this estimation',
    '2. Finalize project requirements and scope',
    '3. Set up development environment',
    '4. Assign team members and roles',
    '5. Create project timeline and milestones',
    '6. Begin development sprint planning',
  ];

  let nsY = 2.8;
  nextSteps.forEach(step => {
    slide8.addText(step, {
      x: 1.5, y: nsY, w: 7.0, h: 0.4,
      fontSize: 16, color: colors.white
    });
    nsY += 0.5;
  });

  slide8.addText('Questions? Contact the estimation team for details', {
    x: 0.5, y: 5.5, w: 9, h: 0.5,
    fontSize: 14, color: colors.white, align: 'center', italic: true
  });

  // Save the presentation
  const fileName = `${project.name.replace(/\s+/g, '_')}_Estimation_${new Date().toISOString().split('T')[0]}.pptx`;
  pptx.writeFile({ fileName });
};
