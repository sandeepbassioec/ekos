import pptxgen from 'pptxgenjs';
import type { RequirementsExportData } from './requirementsExcelGenerator';

const C = {
  primary:   '1E3A8A',
  secondary: '334155',
  success:   '059669',
  warning:   'D97706',
  danger:    'DC2626',
  purple:    '7C3AED',
  white:     'FFFFFF',
  light:     'F8FAFC',
  muted:     '94A3B8',
  border:    'E2E8F0',
};

const addSectionHeader = (slide: any, title: string) => {
  slide.addShape('rect', { x: 0, y: 0, w: 10, h: 0.7, fill: { color: C.primary }, line: { type: 'none' } });
  slide.addText(title, { x: 0.4, y: 0.1, w: 9.2, h: 0.5, fontSize: 22, bold: true, color: C.white });
};

const addStatBox = (slide: any, x: number, y: number, label: string, value: string, color: string) => {
  slide.addShape('rect', { x, y, w: 2.1, h: 1.1, fill: { color }, line: { type: 'none' }, rounding: true });
  slide.addText(value, { x, y: y + 0.1, w: 2.1, h: 0.6, fontSize: 26, bold: true, color: C.white, align: 'center' });
  slide.addText(label, { x, y: y + 0.65, w: 2.1, h: 0.4, fontSize: 10, color: C.white, align: 'center' });
};

export const generateRequirementsPptxFile = (data: RequirementsExportData) => {
  const {
    projectName, roadmapItemTitle, roadmapType, generatedDate,
    formalRequirements, gaps, confusions, suggestions,
    securityRequirements, threats, nfrs, cloudGovernance,
  } = data;

  const pptx = new pptxgen();
  pptx.author  = 'ESIP Platform';
  pptx.company = projectName;
  pptx.title   = `${projectName} — Requirements Report`;
  pptx.layout  = 'LAYOUT_WIDE';

  const functional   = formalRequirements.filter((r: any) => r.category === 'Functional');
  const nfrFormal    = formalRequirements.filter((r: any) => r.category === 'Non-Functional');
  const approved     = formalRequirements.filter((r: any) => r.status === 'Approved');
  const openGaps     = gaps.filter((g: any) => g.status === 'Open');
  const criticalThreats = threats.filter((t: any) => t.level === 'Critical' || t.level === 'High');

  // ── SLIDE 1: Title ────────────────────────────────────────────────────────
  const s1 = pptx.addSlide();
  s1.background = { color: C.primary };
  s1.addShape('rect', { x: 0, y: 4.8, w: 10, h: 0.8, fill: { color: C.secondary }, line: { type: 'none' } });
  s1.addText(projectName, {
    x: 0.6, y: 0.8, w: 8.8, h: 1.0, fontSize: 40, bold: true, color: C.white, align: 'center',
  });
  s1.addText('Requirements Specification & Analysis Report', {
    x: 0.6, y: 1.9, w: 8.8, h: 0.6, fontSize: 22, color: C.muted, align: 'center',
  });
  s1.addText(roadmapItemTitle, {
    x: 0.6, y: 2.7, w: 8.8, h: 0.5, fontSize: 17, color: C.white, align: 'center', italic: true,
  });
  s1.addText([
    { text: `Roadmap Type: ${roadmapType}   |   `, options: { fontSize: 13, color: C.muted } },
    { text: `Generated: ${generatedDate}`, options: { fontSize: 13, color: C.muted } },
  ], { x: 0.6, y: 3.4, w: 8.8, h: 0.4, align: 'center' });
  s1.addText('Confidential — Internal Use Only', {
    x: 0.6, y: 4.85, w: 8.8, h: 0.5, fontSize: 12, color: C.muted, align: 'center',
  });

  // ── SLIDE 2: Executive Summary ────────────────────────────────────────────
  const s2 = pptx.addSlide();
  s2.background = { color: C.light };
  addSectionHeader(s2, 'Executive Summary');

  addStatBox(s2, 0.3,  1.0, 'Total Requirements', String(formalRequirements.length), C.primary);
  addStatBox(s2, 2.55, 1.0, 'Approved',           String(approved.length),           C.success);
  addStatBox(s2, 4.8,  1.0, 'Open Gaps',          String(openGaps.length),           C.warning);
  addStatBox(s2, 7.05, 1.0, 'Threats',            String(threats.length),            C.danger);

  const summaryTableData = [
    [{ text: 'Category',         options: { bold: true, color: C.white, fill: C.primary } },
     { text: 'Count',            options: { bold: true, color: C.white, fill: C.primary } },
     { text: 'Approved',         options: { bold: true, color: C.white, fill: C.primary } },
     { text: 'Status',           options: { bold: true, color: C.white, fill: C.primary } }],
    ['Functional Requirements',   String(functional.length),          String(functional.filter((r:any)=>r.status==='Approved').length),   functional.length === 0 ? '—' : functional.filter((r:any)=>r.status==='Approved').length === functional.length ? '✓ All Approved' : 'In Progress'],
    ['Non-Functional Requirements', String(nfrFormal.length),         String(nfrFormal.filter((r:any)=>r.status==='Approved').length),   nfrFormal.length === 0 ? '—' : 'In Progress'],
    ['AI-Extracted NFRs',         String(nfrs.length),               '—',                                                                nfrs.length > 0 ? 'AI Generated' : '—'],
    ['Security Requirements',     String(securityRequirements.length),'—',                                                                securityRequirements.length > 0 ? 'Review Required' : '—'],
    ['Open Gaps',                 String(openGaps.length),           '—',                                                                openGaps.length > 0 ? '⚠ Action Required' : '✓ None'],
    ['Confusions',                String(confusions.filter((c:any)=>c.status==='Open').length), '—',                                    confusions.filter((c:any)=>c.status==='Open').length > 0 ? '⚠ Needs Clarification' : '✓ None'],
  ];
  s2.addTable(summaryTableData as any, {
    x: 0.3, y: 2.3, w: 9.4, h: 2.2,
    fontSize: 12, border: { type: 'solid', color: C.border },
    align: 'left', valign: 'middle',
  });

  // ── SLIDE 3: Functional Requirements ─────────────────────────────────────
  const s3 = pptx.addSlide();
  s3.background = { color: C.light };
  addSectionHeader(s3, 'Functional Requirements');

  const typeCounts: Record<string, number> = {};
  functional.forEach((r: any) => {
    const t = r.functionalType ?? 'Other';
    typeCounts[t] = (typeCounts[t] ?? 0) + 1;
  });

  const typeRows = Object.entries(typeCounts).map(([t, n]) => [t, String(n)]);

  if (functional.length === 0) {
    s3.addText('No functional requirements have been captured yet.', {
      x: 0.5, y: 2.0, w: 9, h: 0.5, fontSize: 16, color: C.muted, align: 'center',
    });
  } else {
    const funcTableData = [
      [{ text: 'ID', options: { bold: true, color: C.white, fill: C.primary } },
       { text: 'Title', options: { bold: true, color: C.white, fill: C.primary } },
       { text: 'Type', options: { bold: true, color: C.white, fill: C.primary } },
       { text: 'Priority', options: { bold: true, color: C.white, fill: C.primary } },
       { text: 'Status', options: { bold: true, color: C.white, fill: C.primary } }],
      ...functional.slice(0, 12).map((r: any) => [
        r.requirementId, r.title.substring(0, 45), r.functionalType ?? '', r.priority, r.status,
      ]),
    ];
    s3.addTable(funcTableData as any, {
      x: 0.3, y: 0.9, w: 9.4, h: Math.min(4.2, 0.5 + functional.slice(0,12).length * 0.35),
      fontSize: 11, border: { type: 'solid', color: C.border },
    });
    if (functional.length > 12) {
      s3.addText(`+ ${functional.length - 12} more requirements — see Excel for full list`, {
        x: 0.3, y: 5.1, w: 9.4, h: 0.4, fontSize: 11, color: C.muted, italic: true,
      });
    }
  }

  // ── SLIDE 4: Non-Functional Requirements ─────────────────────────────────
  const s4 = pptx.addSlide();
  s4.background = { color: C.light };
  addSectionHeader(s4, 'Non-Functional Requirements');

  const allNFRs = [
    ...nfrFormal.map((r: any) => ({ category: r.nfrType ?? 'Other', requirement: r.title, metric: r.metric ?? '', target: r.target ?? '', source: 'Manual' })),
    ...nfrs.map((n: any) => ({ category: n.category, requirement: n.requirement?.substring(0,60), metric: n.metric ?? '', target: n.target ?? '', source: 'AI' })),
  ];

  if (allNFRs.length === 0) {
    s4.addText('No non-functional requirements captured yet.', {
      x: 0.5, y: 2.0, w: 9, h: 0.5, fontSize: 16, color: C.muted, align: 'center',
    });
  } else {
    const nfrTableData = [
      [{ text: 'Category', options: { bold: true, color: C.white, fill: C.purple } },
       { text: 'Requirement', options: { bold: true, color: C.white, fill: C.purple } },
       { text: 'Metric', options: { bold: true, color: C.white, fill: C.purple } },
       { text: 'Target', options: { bold: true, color: C.white, fill: C.purple } },
       { text: 'Source', options: { bold: true, color: C.white, fill: C.purple } }],
      ...allNFRs.slice(0, 12).map(n => [n.category, n.requirement, n.metric, n.target, n.source]),
    ];
    s4.addTable(nfrTableData as any, {
      x: 0.3, y: 0.9, w: 9.4, h: Math.min(4.2, 0.5 + allNFRs.slice(0,12).length * 0.38),
      fontSize: 11, border: { type: 'solid', color: C.border },
    });
    if (allNFRs.length > 12) {
      s4.addText(`+ ${allNFRs.length - 12} more — see Excel for full list`, {
        x: 0.3, y: 5.1, w: 9.4, h: 0.4, fontSize: 11, color: C.muted, italic: true,
      });
    }
  }

  // ── SLIDE 5: Security & Threat Analysis ───────────────────────────────────
  const s5 = pptx.addSlide();
  s5.background = { color: C.light };
  addSectionHeader(s5, 'Security & Threat Analysis (STRIDE)');

  addStatBox(s5, 0.3,  0.9, 'Security Reqs',       String(securityRequirements.length), C.primary);
  addStatBox(s5, 2.55, 0.9, 'Total Threats',        String(threats.length),              C.danger);
  addStatBox(s5, 4.8,  0.9, 'Critical/High',        String(criticalThreats.length),      C.warning);
  addStatBox(s5, 7.05, 0.9, 'Mitigated',            String(threats.filter((t:any)=>t.status==='Mitigated').length), C.success);

  if (threats.length > 0) {
    const threatTableData = [
      [{ text: 'Threat', options: { bold: true, color: C.white, fill: C.danger } },
       { text: 'STRIDE', options: { bold: true, color: C.white, fill: C.danger } },
       { text: 'Level', options: { bold: true, color: C.white, fill: C.danger } },
       { text: 'Likelihood', options: { bold: true, color: C.white, fill: C.danger } },
       { text: 'Status', options: { bold: true, color: C.white, fill: C.danger } }],
      ...threats.slice(0, 8).map((t: any) => [
        t.threat.substring(0, 40), t.category, t.level, t.likelihood, t.status,
      ]),
    ];
    s5.addTable(threatTableData as any, {
      x: 0.3, y: 2.2, w: 9.4, h: Math.min(3.0, 0.5 + threats.slice(0,8).length * 0.38),
      fontSize: 11, border: { type: 'solid', color: C.border },
    });
  } else {
    s5.addText('No threats identified yet. Run AI analysis to extract STRIDE threats.', {
      x: 0.3, y: 2.4, w: 9.4, h: 0.5, fontSize: 13, color: C.muted, italic: true,
    });
  }

  // ── SLIDE 6: Gaps & Open Items ────────────────────────────────────────────
  const s6 = pptx.addSlide();
  s6.background = { color: C.light };
  addSectionHeader(s6, 'Gaps & Open Items — Action Required');

  const criticalGaps = gaps.filter((g: any) => g.severity === 'Critical' || g.severity === 'High');

  addStatBox(s6, 0.3,  0.9, 'Total Gaps',     String(gaps.length),        C.warning);
  addStatBox(s6, 2.55, 0.9, 'Open',           String(openGaps.length),    C.danger);
  addStatBox(s6, 4.8,  0.9, 'Critical/High',  String(criticalGaps.length), C.danger);
  addStatBox(s6, 7.05, 0.9, 'Confusions',     String(confusions.filter((c:any)=>c.status==='Open').length), C.warning);

  if (openGaps.length > 0) {
    const gapTableData = [
      [{ text: 'Gap Title', options: { bold: true, color: C.white, fill: C.warning } },
       { text: 'Severity', options: { bold: true, color: C.white, fill: C.warning } },
       { text: 'Description', options: { bold: true, color: C.white, fill: C.warning } }],
      ...openGaps.slice(0, 8).map((g: any) => [
        g.title.substring(0, 40), g.severity, g.description.substring(0, 80),
      ]),
    ];
    s6.addTable(gapTableData as any, {
      x: 0.3, y: 2.2, w: 9.4, h: Math.min(3.0, 0.5 + openGaps.slice(0,8).length * 0.38),
      fontSize: 11, border: { type: 'solid', color: C.border },
    });
  } else {
    s6.addText('No open gaps — all gaps have been resolved.', {
      x: 0.3, y: 2.4, w: 9.4, h: 0.5, fontSize: 14, color: C.success,
    });
  }

  // ── SLIDE 7: Recommendations ──────────────────────────────────────────────
  const s7 = pptx.addSlide();
  s7.background = { color: C.light };
  addSectionHeader(s7, 'Recommendations & Suggestions');

  const acceptedSuggestions = suggestions.filter((s: any) => s.status === 'Accepted' || s.status === 'Pending');

  if (acceptedSuggestions.length === 0) {
    s7.addText('No suggestions generated yet. Run AI analysis to generate recommendations.', {
      x: 0.5, y: 2.0, w: 9, h: 0.5, fontSize: 14, color: C.muted, align: 'center',
    });
  } else {
    const sugTableData = [
      [{ text: 'Recommendation', options: { bold: true, color: C.white, fill: C.success } },
       { text: 'Type', options: { bold: true, color: C.white, fill: C.success } },
       { text: 'Impact', options: { bold: true, color: C.white, fill: C.success } },
       { text: 'Effort', options: { bold: true, color: C.white, fill: C.success } }],
      ...acceptedSuggestions.slice(0, 10).map((s: any) => [
        s.title.substring(0, 50), s.type, s.impact?.substring(0, 40) ?? '—', s.effort ?? '—',
      ]),
    ];
    s7.addTable(sugTableData as any, {
      x: 0.3, y: 0.9, w: 9.4, h: Math.min(4.2, 0.5 + acceptedSuggestions.slice(0,10).length * 0.42),
      fontSize: 11, border: { type: 'solid', color: C.border },
    });
  }

  // ── SLIDE 8: Next Steps ───────────────────────────────────────────────────
  const s8 = pptx.addSlide();
  s8.background = { color: C.primary };
  s8.addText('Next Steps', {
    x: 0.5, y: 0.4, w: 9, h: 0.6, fontSize: 32, bold: true, color: C.white, align: 'center',
  });

  const nextSteps = [
    openGaps.length > 0     ? `Resolve ${openGaps.length} open gap${openGaps.length > 1 ? 's' : ''} with stakeholders` : null,
    confusions.filter((c:any)=>c.status==='Open').length > 0 ? `Clarify ${confusions.filter((c:any)=>c.status==='Open').length} open confusion${confusions.filter((c:any)=>c.status==='Open').length>1?'s':''}` : null,
    formalRequirements.filter((r:any)=>r.status==='Draft').length > 0 ? `Review and approve ${formalRequirements.filter((r:any)=>r.status==='Draft').length} draft requirements` : null,
    criticalThreats.length > 0 ? `Address ${criticalThreats.length} critical/high STRIDE threat${criticalThreats.length>1?'s':''}` : null,
    securityRequirements.filter((s:any)=>s.status==='Draft').length > 0 ? `Review ${securityRequirements.filter((s:any)=>s.status==='Draft').length} draft security requirements` : null,
    'Complete estimation and resource planning',
    'Get formal sign-off from stakeholders on approved requirements',
  ].filter(Boolean);

  nextSteps.forEach((step, i) => {
    s8.addShape('rect', {
      x: 0.5, y: 1.2 + i * 0.58, w: 0.4, h: 0.4,
      fill: { color: C.warning }, line: { type: 'none' },
    });
    s8.addText(`${i + 1}`, {
      x: 0.5, y: 1.22 + i * 0.58, w: 0.4, h: 0.35,
      fontSize: 14, bold: true, color: C.white, align: 'center',
    });
    s8.addText(step as string, {
      x: 1.1, y: 1.22 + i * 0.58, w: 8.4, h: 0.38,
      fontSize: 14, color: C.white,
    });
  });

  s8.addText(`${projectName} — Confidential`, {
    x: 0.5, y: 5.2, w: 9, h: 0.3, fontSize: 11, color: C.muted, align: 'center',
  });

  // ── Save ──────────────────────────────────────────────────────────────────
  const safeName = projectName.replace(/[^a-zA-Z0-9]/g, '_');
  const safeItem = roadmapItemTitle.replace(/[^a-zA-Z0-9]/g, '_').substring(0, 30);
  const dateStr  = new Date().toISOString().split('T')[0];
  pptx.writeFile({ fileName: `${safeName}_${safeItem}_Requirements_${dateStr}.pptx` });
};
