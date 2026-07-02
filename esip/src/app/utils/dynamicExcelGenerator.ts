import * as XLSX from 'xlsx';
import { ProjectSetup, Screen, EstimationResults, RiskFactor, EstimationMethod } from '../store/estimationStore';

export interface EstimationData {
  project: ProjectSetup;
  screens: Screen[];
  results: EstimationResults;
  riskFactors: RiskFactor[];
  contingency: number;
  selectedMethods: EstimationMethod[];
}

export const generateDynamicExcelFile = (data: EstimationData) => {
  const { project, screens, results, riskFactors, contingency, selectedMethods } = data;
  const workbook = XLSX.utils.book_new();

  // ===== SHEET 1: Executive Summary =====
  const currentDate = new Date().toLocaleDateString('en-US', { 
    year: 'numeric', 
    month: 'long', 
    day: 'numeric' 
  });

  const summaryData = [
    [`${project.name} - Development Estimates`],
    [`Estimation Methods: ${selectedMethods.join(', ')}`],
    [''],
    ['Project Information'],
    ['Project Name:', project.name],
    ['Project Type:', project.type],
    ['Technology Stack:', project.techStack.join(', ')],
    ['Date Generated:', currentDate],
    ['Team Size:', project.teamSize],
    ['Hourly Rate:', `$${project.hourlyRate}`],
    [''],
    ['EXECUTIVE SUMMARY'],
    [''],
    ['Metric', 'Value', 'Details'],
    ['Total Effort', `${results.totalHours} hours`, `${Math.round(results.totalHours / 8)} days`],
    ['Total Cost', `$${results.totalCost.toLocaleString()}`, `At $${project.hourlyRate}/hour`],
    ['Timeline', `${results.timelineWeeks} weeks`, `With team of ${project.teamSize}`],
    ['Confidence Level', `${results.confidence}%`, 'Based on method and scope'],
    ['Contingency Buffer', `${contingency}%`, `${riskFactors.length} risks identified`],
    ['Screens/Features', screens.length, `Across ${screens.filter(s => !s.parentId).length} modules`],
    [''],
    ['EFFORT BREAKDOWN'],
    [''],
    ['Phase', 'Hours', 'Percentage', 'Days (8h)'],
    ['Development', results.breakdown.development, '50%', Math.round(results.breakdown.development / 8)],
    ['Testing (QA)', results.breakdown.testing, '30%', Math.round(results.breakdown.testing / 8)],
    ['Hardening', results.breakdown.hardening, '20%', Math.round(results.breakdown.hardening / 8)],
    ['TOTAL', results.totalHours, '100%', Math.round(results.totalHours / 8)],
    [''],
    ['TIMELINE CALCULATION'],
    [''],
    ['Total Hours', results.totalHours],
    ['Hours per Week (per person)', 40],
    ['Team Members', project.teamSize],
    ['Weekly Capacity', project.teamSize * 40],
    ['Estimated Duration', `${results.timelineWeeks} weeks`],
  ];

  const summarySheet = XLSX.utils.aoa_to_sheet(summaryData);
  summarySheet['!cols'] = [
    { wch: 35 },
    { wch: 25 },
    { wch: 30 },
  ];

  XLSX.utils.book_append_sheet(workbook, summarySheet, 'Executive Summary');

  // ===== SHEET 2: Scope Breakdown (All Screens/Features) =====
  const scopeHeaders = ['ID', 'Name', 'Type', 'Complexity', 'Level', 'Parent', 'Description'];
  const scopeData = [
    ['SCOPE DEFINITION - ALL FEATURES'],
    [''],
    scopeHeaders,
  ];

  screens.forEach((screen, idx) => {
    const indent = '  '.repeat(screen.level || 0);
    scopeData.push([
      idx + 1,
      indent + screen.name,
      screen.type,
      screen.complexity,
      screen.level || 0,
      screen.parentId ? screens.find(s => s.id === screen.parentId)?.name || 'N/A' : 'Root',
      screen.description || '',
    ]);
  });

  scopeData.push(['']);
  scopeData.push(['SUMMARY BY TYPE']);
  scopeData.push(['Type', 'Count']);
  
  const typeCount: Record<string, number> = {};
  screens.forEach(s => {
    typeCount[s.type] = (typeCount[s.type] || 0) + 1;
  });
  Object.entries(typeCount).forEach(([type, count]) => {
    scopeData.push([type, count]);
  });

  scopeData.push(['']);
  scopeData.push(['SUMMARY BY COMPLEXITY']);
  scopeData.push(['Complexity', 'Count']);
  
  const complexityCount: Record<string, number> = {};
  screens.forEach(s => {
    complexityCount[s.complexity] = (complexityCount[s.complexity] || 0) + 1;
  });
  Object.entries(complexityCount).forEach(([complexity, count]) => {
    scopeData.push([complexity, count]);
  });

  const scopeSheet = XLSX.utils.aoa_to_sheet(scopeData);
  scopeSheet['!cols'] = [
    { wch: 6 },
    { wch: 45 },
    { wch: 15 },
    { wch: 12 },
    { wch: 8 },
    { wch: 20 },
    { wch: 40 },
  ];

  XLSX.utils.book_append_sheet(workbook, scopeSheet, 'Scope Definition');

  // ===== SHEET 3: Estimation Details (Per Screen) =====
  const estimationHeaders = ['Screen', 'Method', 'Details', 'Estimated Hours'];
  const estimationData = [
    ['ESTIMATION DETAILS BY SCREEN'],
    [''],
    ['Using methods:', selectedMethods.join(', ')],
    [''],
    estimationHeaders,
  ];

  screens.forEach(screen => {
    selectedMethods.forEach(method => {
      let details = '';
      let hours = 0;

      if (method === 'PERT' && screen.pertData) {
        const { optimistic, mostLikely, pessimistic } = screen.pertData;
        const expected = (optimistic + 4 * mostLikely + pessimistic) / 6;
        details = `O: ${optimistic}h, M: ${mostLikely}h, P: ${pessimistic}h`;
        hours = expected;
      } else if (method === 'STORY_POINTS' && screen.storyPoints) {
        details = `${screen.storyPoints} points × 8 hrs/point`;
        hours = screen.storyPoints * 8;
      } else if (method === 'TSHIRT' && screen.tshirtSize) {
        const sizeMap: Record<string, number> = { 
          XS: 8, S: 20, M: 40, L: 80, XL: 120, XXL: 200 
        };
        hours = sizeMap[screen.tshirtSize] || 40;
        details = `Size: ${screen.tshirtSize} (${hours} hours)`;
      } else {
        details = 'Not estimated with this method';
        hours = 0;
      }

      estimationData.push([
        screen.name,
        method,
        details,
        hours > 0 ? parseFloat(hours.toFixed(1)) : 'N/A',
      ]);
    });
  });

  estimationData.push(['']);
  estimationData.push(['TOTALS BY METHOD']);
  estimationData.push(['Method', 'Total Hours']);

  selectedMethods.forEach(method => {
    let total = 0;
    screens.forEach(screen => {
      if (method === 'PERT' && screen.pertData) {
        const { optimistic, mostLikely, pessimistic } = screen.pertData;
        total += (optimistic + 4 * mostLikely + pessimistic) / 6;
      } else if (method === 'STORY_POINTS' && screen.storyPoints) {
        total += screen.storyPoints * 8;
      } else if (method === 'TSHIRT' && screen.tshirtSize) {
        const sizeMap: Record<string, number> = { 
          XS: 8, S: 20, M: 40, L: 80, XL: 120, XXL: 200 
        };
        total += sizeMap[screen.tshirtSize] || 40;
      }
    });
    estimationData.push([method, parseFloat(total.toFixed(1))]);
  });

  const estimationSheet = XLSX.utils.aoa_to_sheet(estimationData);
  estimationSheet['!cols'] = [
    { wch: 40 },
    { wch: 18 },
    { wch: 35 },
    { wch: 18 },
  ];

  XLSX.utils.book_append_sheet(workbook, estimationSheet, 'Estimation Details');

  // ===== SHEET 4: Risk Assessment =====
  const riskHeaders = ['Risk Name', 'Impact', 'Probability', 'Risk Score'];
  const riskData = [
    ['RISK ASSESSMENT'],
    [''],
    ['Contingency Buffer:', `${contingency}%`],
    ['Total Risks Identified:', riskFactors.length],
    [''],
    riskHeaders,
  ];

  const impactWeights: Record<string, number> = { LOW: 1, MEDIUM: 2, HIGH: 3 };

  riskFactors.forEach(risk => {
    const riskScore = impactWeights[risk.impact] * (risk.probability / 100);
    
    riskData.push([
      risk.name,
      risk.impact,
      `${risk.probability}%`,
      parseFloat(riskScore.toFixed(2)),
    ]);
  });

  riskData.push(['']);
  riskData.push(['CONTINGENCY CALCULATION']);
  riskData.push(['Metric', 'Value']);
  
  const totalRiskScore = riskFactors.reduce(
    (sum, r) => sum + impactWeights[r.impact] * (r.probability / 100), 
    0
  );
  
  const baseHours = Math.round(results.totalHours / (1 + contingency / 100));
  const contingencyHours = results.totalHours - baseHours;

  riskData.push(['Total Risk Score:', parseFloat(totalRiskScore.toFixed(2))]);
  riskData.push(['Recommended Contingency:', `${contingency}%`]);
  riskData.push(['Base Hours (before contingency):', baseHours]);
  riskData.push(['Contingency Hours Added:', contingencyHours]);
  riskData.push(['Total with Contingency:', results.totalHours]);

  riskData.push(['']);
  riskData.push(['RISK GUIDELINES']);
  riskData.push(['Risk Score Range', 'Recommended Contingency']);
  riskData.push(['0.0 - 1.0 (Low)', '5-10%']);
  riskData.push(['1.0 - 2.0 (Medium)', '10-20%']);
  riskData.push(['2.0 - 3.0 (High)', '20-30%']);
  riskData.push(['3.0+ (Very High)', '30-50%']);

  const riskSheet = XLSX.utils.aoa_to_sheet(riskData);
  riskSheet['!cols'] = [
    { wch: 50 },
    { wch: 15 },
    { wch: 15 },
    { wch: 12 },
  ];

  XLSX.utils.book_append_sheet(workbook, riskSheet, 'Risk Assessment');

  // ===== SHEET 5: Work Breakdown Structure =====
  const wbsHeaders = ['Level', 'ID', 'Item Name', 'Type', 'Complexity', 'Dev Hours', 'QA Hours', 'Hardening Hours', 'Total Hours'];
  const wbsData = [
    ['WORK BREAKDOWN STRUCTURE'],
    [''],
    ['Phase Distribution:', `Development: 50% | QA: 30% | Hardening: 20%`],
    [''],
    wbsHeaders,
  ];

  screens.forEach((screen, idx) => {
    // Calculate hours for this screen (simplified - using proportion of total)
    const screenPortion = 1 / screens.length;
    const devHours = Math.round(results.breakdown.development * screenPortion);
    const qaHours = Math.round(results.breakdown.testing * screenPortion);
    const hardeningHours = Math.round(results.breakdown.hardening * screenPortion);
    const totalHours = devHours + qaHours + hardeningHours;

    const indent = '  '.repeat(screen.level || 0);
    
    wbsData.push([
      screen.level || 0,
      idx + 1,
      indent + screen.name,
      screen.type,
      screen.complexity,
      devHours,
      qaHours,
      hardeningHours,
      totalHours,
    ]);
  });

  wbsData.push(['']);
  wbsData.push(['', '', 'TOTAL', '', '', results.breakdown.development, results.breakdown.testing, results.breakdown.hardening, results.totalHours]);

  wbsData.push(['']);
  wbsData.push(['NOTES']);
  wbsData.push(['• Hours are distributed evenly across all screens/features']);
  wbsData.push(['• Actual hours may vary based on complexity and dependencies']);
  wbsData.push(['• Hierarchy levels: 0 = Root, 1 = Child, 2 = Grandchild']);

  const wbsSheet = XLSX.utils.aoa_to_sheet(wbsData);
  wbsSheet['!cols'] = [
    { wch: 8 },
    { wch: 6 },
    { wch: 50 },
    { wch: 15 },
    { wch: 12 },
    { wch: 12 },
    { wch: 12 },
    { wch: 15 },
    { wch: 12 },
  ];

  XLSX.utils.book_append_sheet(workbook, wbsSheet, 'Work Breakdown');

  // ===== SHEET 6: Assumptions & Exclusions =====
  const assumptionsData = [
    ['PROJECT ASSUMPTIONS & EXCLUSIONS'],
    [''],
    ['PROJECT: ' + project.name],
    ['TYPE: ' + project.type],
    [''],
    ['INCLUSIONS'],
    ['✓ Frontend development'],
    ['✓ Responsive design (Desktop, Tablet, Mobile)'],
    ['✓ Component development and integration'],
    ['✓ State management implementation'],
    ['✓ API integration (frontend only)'],
    ['✓ Unit testing and integration testing'],
    ['✓ Code review and quality assurance'],
    ['✓ Bug fixing during development'],
    ['✓ Documentation (inline code comments)'],
    [''],
    ['EXCLUSIONS'],
    ['✗ Backend/Server development'],
    ['✗ Database design and implementation'],
    ['✗ DevOps/Infrastructure setup'],
    ['✗ Third-party API development'],
    ['✗ Content creation (copywriting, images)'],
    ['✗ SEO optimization'],
    ['✗ Performance testing (load, stress)'],
    ['✗ Security penetration testing'],
    ['✗ Training and documentation (user manuals)'],
    [''],
    ['ASSUMPTIONS'],
    ['• Requirements are well-defined and stable'],
    ['• Design assets will be provided on time'],
    ['• Development environment is set up'],
    ['• Access to necessary APIs and services'],
    [`• Team of ${project.teamSize} developers available`],
    ['• Standard working hours (40 hrs/week)'],
    ['• No major scope changes during development'],
    ['• Timely feedback and approvals from stakeholders'],
    project.techStack.length > 0 ? `• Technology stack: ${project.techStack.join(', ')}` : '',
    ['• Team has experience with selected technologies'],
    [''],
    ['METHODOLOGY'],
    [`• Estimation Methods: ${selectedMethods.join(', ')}`],
    [`• Contingency Buffer: ${contingency}% (based on ${riskFactors.length} identified risks)`],
    ['• Confidence Level: ' + results.confidence + '%'],
    ['• Phase Distribution: Development 50%, QA 30%, Hardening 20%'],
  ].filter(row => row.length > 0 && row[0] !== '');

  const assumptionsSheet = XLSX.utils.aoa_to_sheet(assumptionsData);
  assumptionsSheet['!cols'] = [{ wch: 70 }];

  XLSX.utils.book_append_sheet(workbook, assumptionsSheet, 'Assumptions');

  // Generate and download
  const excelBuffer = XLSX.write(workbook, { bookType: 'xlsx', type: 'array' });
  const blob = new Blob([excelBuffer], { type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' });
  const url = window.URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  const fileName = `${project.name.replace(/\s+/g, '_')}_Estimation_${new Date().toISOString().split('T')[0]}.xlsx`;
  link.download = fileName;
  link.click();
  window.URL.revokeObjectURL(url);
};
