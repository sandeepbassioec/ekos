import * as XLSX from 'xlsx';

export interface RequirementsExportData {
  projectName: string;
  roadmapItemTitle: string;
  roadmapType: string;
  generatedDate: string;
  formalRequirements: any[];
  gaps: any[];
  confusions: any[];
  suggestions: any[];
  securityRequirements: any[];
  threats: any[];
  nfrs: any[];
  cloudGovernance: any[];
}

const applyColWidths = (sheet: XLSX.WorkSheet, widths: number[]) => {
  sheet['!cols'] = widths.map(w => ({ wch: w }));
};

export const generateRequirementsExcelFile = (data: RequirementsExportData) => {
  const {
    projectName, roadmapItemTitle, roadmapType, generatedDate,
    formalRequirements, gaps, confusions, suggestions,
    securityRequirements, threats, nfrs, cloudGovernance,
  } = data;

  const wb = XLSX.utils.book_new();

  const functional = formalRequirements.filter((r: any) => r.category === 'Functional');
  const nfrFormal  = formalRequirements.filter((r: any) => r.category === 'Non-Functional');
  const constraints = formalRequirements.filter((r: any) => r.category === 'Constraint');
  const approved   = formalRequirements.filter((r: any) => r.status === 'Approved');
  const openGaps   = gaps.filter((g: any) => g.status === 'Open');
  const openConf   = confusions.filter((c: any) => c.status === 'Open');

  // ── SHEET 1: Summary ─────────────────────────────────────────────────────
  const summaryRows = [
    [`${projectName} — Requirements Specification`],
    [`Roadmap Item: ${roadmapItemTitle}`],
    [`Roadmap Type: ${roadmapType}`],
    [`Generated: ${generatedDate}`],
    [''],
    ['REQUIREMENTS OVERVIEW'],
    [''],
    ['Category', 'Total', 'Approved', 'Draft', 'Under Review'],
    ['Functional',     functional.length,  functional.filter((r:any)=>r.status==='Approved').length,  functional.filter((r:any)=>r.status==='Draft').length,  functional.filter((r:any)=>r.status==='Under Review').length],
    ['Non-Functional', nfrFormal.length,   nfrFormal.filter((r:any)=>r.status==='Approved').length,   nfrFormal.filter((r:any)=>r.status==='Draft').length,   nfrFormal.filter((r:any)=>r.status==='Under Review').length],
    ['Constraints',    constraints.length, constraints.filter((r:any)=>r.status==='Approved').length, constraints.filter((r:any)=>r.status==='Draft').length, constraints.filter((r:any)=>r.status==='Under Review').length],
    ['TOTAL',          formalRequirements.length, approved.length, formalRequirements.filter((r:any)=>r.status==='Draft').length, formalRequirements.filter((r:any)=>r.status==='Under Review').length],
    [''],
    ['AI ANALYSIS OVERVIEW'],
    [''],
    ['Area', 'Total', 'Open/Pending', 'Resolved/Done'],
    ['Gaps',                  gaps.length,                openGaps.length,                                    gaps.length - openGaps.length],
    ['Confusions',            confusions.length,          openConf.length,                                    confusions.length - openConf.length],
    ['Suggestions',           suggestions.length,         suggestions.filter((s:any)=>s.status==='Pending').length, suggestions.filter((s:any)=>s.status!=='Pending').length],
    ['Security Requirements', securityRequirements.length, securityRequirements.filter((s:any)=>s.status==='Draft').length, securityRequirements.filter((s:any)=>s.status!=='Draft').length],
    ['STRIDE Threats',        threats.length,             threats.filter((t:any)=>t.status==='Identified').length, threats.filter((t:any)=>t.status!=='Identified').length],
    ['NFRs (AI)',              nfrs.length,               nfrs.filter((n:any)=>n.status==='Draft').length,         nfrs.filter((n:any)=>n.status!=='Draft').length],
    ['Cloud Governance',      cloudGovernance.length,     cloudGovernance.filter((c:any)=>c.status==='Draft').length, cloudGovernance.filter((c:any)=>c.status!=='Draft').length],
  ];
  const sheet1 = XLSX.utils.aoa_to_sheet(summaryRows);
  applyColWidths(sheet1, [35, 12, 12, 12, 14]);
  XLSX.utils.book_append_sheet(wb, sheet1, 'Summary');

  // ── SHEET 2: Functional Requirements ─────────────────────────────────────
  const funcRows: any[][] = [
    ['FUNCTIONAL REQUIREMENTS'],
    [''],
    ['ID', 'Title', 'Sub-Type', 'Priority', 'Status', 'Description', 'User Story', 'Acceptance Criteria', 'Source'],
  ];
  functional.forEach((r: any) => {
    const story = r.userStory ? `As a ${r.userStory.asA}, I want ${r.userStory.iWant}, so that ${r.userStory.soThat}` : '';
    funcRows.push([
      r.requirementId, r.title, r.functionalType ?? '', r.priority, r.status,
      r.description, story,
      (r.acceptanceCriteria ?? []).join('\n'),
      r.source,
    ]);
  });
  if (functional.length === 0) funcRows.push(['No functional requirements added yet.']);
  const sheet2 = XLSX.utils.aoa_to_sheet(funcRows);
  applyColWidths(sheet2, [14, 30, 16, 12, 14, 50, 50, 50, 10]);
  XLSX.utils.book_append_sheet(wb, sheet2, 'Functional Requirements');

  // ── SHEET 3: Non-Functional Requirements ─────────────────────────────────
  const nfrRows: any[][] = [
    ['NON-FUNCTIONAL REQUIREMENTS'],
    [''],
    ['ID', 'Title', 'NFR Type', 'Priority', 'Status', 'Description', 'Metric', 'Target', 'How to Measure', 'Source'],
  ];
  nfrFormal.forEach((r: any) => {
    nfrRows.push([
      r.requirementId, r.title, r.nfrType ?? '', r.priority, r.status,
      r.description, r.metric ?? '', r.target ?? '', r.measurement ?? '', r.source,
    ]);
  });
  // Also include AI-extracted NFRs
  if (nfrs.length > 0) {
    nfrRows.push([''], ['AI-EXTRACTED NFRs'], ['']);
    nfrRows.push(['Category', 'Requirement', 'Metric', 'Target', 'Measurement', 'Priority', 'Status']);
    nfrs.forEach((n: any) => {
      nfrRows.push([n.category, n.requirement, n.metric ?? '', n.target ?? '', n.measurement ?? '', n.priority, n.status]);
    });
  }
  if (nfrFormal.length === 0 && nfrs.length === 0) nfrRows.push(['No NFRs added yet.']);
  const sheet3 = XLSX.utils.aoa_to_sheet(nfrRows);
  applyColWidths(sheet3, [14, 30, 18, 12, 14, 50, 25, 20, 25, 10]);
  XLSX.utils.book_append_sheet(wb, sheet3, 'Non-Functional Requirements');

  // ── SHEET 4: Security & Threats ───────────────────────────────────────────
  const secRows: any[][] = [
    ['SECURITY REQUIREMENTS & THREAT ANALYSIS'],
    [''],
    ['SECURITY REQUIREMENTS'],
    ['Title', 'Category', 'Level', 'Status', 'Description', 'Controls', 'Test Criteria'],
  ];
  securityRequirements.forEach((s: any) => {
    secRows.push([
      s.title, s.category, s.level, s.status, s.description,
      (s.controls ?? []).join('\n'),
      (s.testCriteria ?? []).join('\n'),
    ]);
  });
  if (securityRequirements.length === 0) secRows.push(['No security requirements identified.']);

  secRows.push([''], ['STRIDE THREAT ANALYSIS'], ['']);
  secRows.push(['Threat', 'STRIDE Category', 'Level', 'Likelihood', 'Status', 'Description', 'Impact', 'Mitigations']);
  threats.forEach((t: any) => {
    secRows.push([
      t.threat, t.category, t.level, t.likelihood, t.status,
      t.description, t.impact ?? '',
      (t.mitigations ?? []).join('\n'),
    ]);
  });
  if (threats.length === 0) secRows.push(['No threats identified.']);

  const sheet4 = XLSX.utils.aoa_to_sheet(secRows);
  applyColWidths(sheet4, [30, 20, 12, 14, 14, 50, 40, 50]);
  XLSX.utils.book_append_sheet(wb, sheet4, 'Security & Threats');

  // ── SHEET 5: Gaps & Confusions ────────────────────────────────────────────
  const gapRows: any[][] = [
    ['GAPS & CONFUSIONS — OPEN ITEMS'],
    [''],
    ['GAPS (Missing Requirements)'],
    ['Title', 'Severity', 'Status', 'Description', 'Suggestions', 'Resolution'],
  ];
  gaps.forEach((g: any) => {
    gapRows.push([
      g.title, g.severity, g.status, g.description,
      (g.suggestions ?? []).join('\n'),
      g.resolution ?? '',
    ]);
  });
  if (gaps.length === 0) gapRows.push(['No gaps identified.']);

  gapRows.push([''], ['CONFUSIONS & AMBIGUITIES'], ['']);
  gapRows.push(['Question', 'Status', 'Context', 'Clarification']);
  confusions.forEach((c: any) => {
    gapRows.push([c.question, c.status, c.context, c.clarification ?? '']);
  });
  if (confusions.length === 0) gapRows.push(['No confusions identified.']);

  const sheet5 = XLSX.utils.aoa_to_sheet(gapRows);
  applyColWidths(sheet5, [35, 12, 14, 55, 50, 40]);
  XLSX.utils.book_append_sheet(wb, sheet5, 'Gaps & Confusions');

  // ── SHEET 6: Suggestions & Cloud Governance ───────────────────────────────
  const sugRows: any[][] = [
    ['SUGGESTIONS & CLOUD GOVERNANCE'],
    [''],
    ['SUGGESTIONS & RECOMMENDATIONS'],
    ['Title', 'Type', 'Status', 'Description', 'Impact', 'Effort'],
  ];
  suggestions.forEach((s: any) => {
    sugRows.push([s.title, s.type, s.status, s.description, s.impact ?? '', s.effort ?? '']);
  });
  if (suggestions.length === 0) sugRows.push(['No suggestions generated.']);

  sugRows.push([''], ['CLOUD GOVERNANCE REQUIREMENTS'], ['']);
  sugRows.push(['Area', 'Requirement', 'Priority', 'Status', 'Details']);
  cloudGovernance.forEach((c: any) => {
    sugRows.push([c.area, c.requirement, c.priority, c.status, c.details]);
  });
  if (cloudGovernance.length === 0) sugRows.push(['No cloud governance requirements identified.']);

  const sheet6 = XLSX.utils.aoa_to_sheet(sugRows);
  applyColWidths(sheet6, [30, 18, 14, 55, 35, 20]);
  XLSX.utils.book_append_sheet(wb, sheet6, 'Suggestions & Cloud');

  // ── Download ──────────────────────────────────────────────────────────────
  const safeName = projectName.replace(/[^a-zA-Z0-9]/g, '_');
  const safeItem = roadmapItemTitle.replace(/[^a-zA-Z0-9]/g, '_').substring(0, 30);
  const dateStr  = new Date().toISOString().split('T')[0];
  XLSX.writeFile(wb, `${safeName}_${safeItem}_Requirements_${dateStr}.xlsx`);
};
