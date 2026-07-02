import React, { useState } from 'react';
import { FileSpreadsheet, Presentation, Download, CheckCircle2, AlertCircle, FileText } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Badge } from '../ui/badge';
import { generateRequirementsExcelFile } from '../../utils/requirementsExcelGenerator';
import { generateRequirementsPptxFile } from '../../utils/requirementsPptxGenerator';
import { toast } from 'sonner';

interface DeliverablesTabProps {
  roadmapItemTitle: string;
  roadmapType: string;
  projectName: string;
  hub: any;
}

interface DeliverableCard {
  id: string;
  title: string;
  description: string;
  icon: React.ReactNode;
  format: string;
  sheets?: string[];
  slides?: string[];
  color: string;
  onGenerate: () => Promise<void>;
  isReady: boolean;
  missingData?: string;
}

export function DeliverablesTab({ roadmapItemTitle, roadmapType, projectName, hub }: DeliverablesTabProps) {
  const [generating, setGenerating] = useState<string | null>(null);

  const buildExportData = () => ({
    projectName,
    roadmapItemTitle,
    roadmapType,
    generatedDate: new Date().toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' }),
    formalRequirements: hub?.formalRequirements ?? [],
    gaps:               hub?.gaps ?? [],
    confusions:         hub?.confusions ?? [],
    suggestions:        hub?.suggestions ?? [],
    securityRequirements: hub?.securityRequirements ?? [],
    threats:            hub?.threats ?? [],
    nfrs:               hub?.nfrs ?? [],
    cloudGovernance:    hub?.cloudGovernance ?? [],
  });

  const hasSomeData =
    (hub?.formalRequirements?.length ?? 0) > 0 ||
    (hub?.gaps?.length ?? 0) > 0 ||
    (hub?.nfrs?.length ?? 0) > 0 ||
    (hub?.securityRequirements?.length ?? 0) > 0;

  const handleGenerate = async (id: string, fn: () => void) => {
    setGenerating(id);
    try {
      fn();
      toast.success('Deliverable generated — check your downloads');
    } catch (err) {
      console.error(err);
      toast.error('Failed to generate deliverable');
    } finally {
      setGenerating(null);
    }
  };

  const deliverables: DeliverableCard[] = [
    {
      id: 'excel',
      title: 'Requirements Specification',
      description: 'Complete requirements register in Excel — all formal requirements, NFRs, security, gaps, confusions, suggestions and cloud governance across 6 worksheets.',
      icon: <FileSpreadsheet className="w-8 h-8 text-green-600" />,
      format: 'Excel (.xlsx)',
      sheets: [
        'Summary Dashboard',
        'Functional Requirements',
        'Non-Functional Requirements',
        'Security & Threats (STRIDE)',
        'Gaps & Confusions',
        'Suggestions & Cloud Governance',
      ],
      color: 'border-green-200 bg-green-50',
      isReady: hasSomeData,
      missingData: !hasSomeData ? 'Add requirements or run AI analysis first' : undefined,
      onGenerate: async () => handleGenerate('excel', () => generateRequirementsExcelFile(buildExportData())),
    },
    {
      id: 'pptx',
      title: 'Executive Presentation',
      description: 'Professional slide deck for stakeholder presentations — executive summary, requirements overview, security analysis, gaps and next steps across 8 slides.',
      icon: <Presentation className="w-8 h-8 text-blue-600" />,
      format: 'PowerPoint (.pptx)',
      slides: [
        'Title Slide',
        'Executive Summary (stats)',
        'Functional Requirements',
        'Non-Functional Requirements',
        'Security & Threat Analysis',
        'Gaps & Open Items',
        'Recommendations',
        'Next Steps',
      ],
      color: 'border-blue-200 bg-blue-50',
      isReady: hasSomeData,
      missingData: !hasSomeData ? 'Add requirements or run AI analysis first' : undefined,
      onGenerate: async () => handleGenerate('pptx', () => generateRequirementsPptxFile(buildExportData())),
    },
  ];

  const stats = [
    { label: 'Formal Requirements', value: hub?.formalRequirements?.length ?? 0 },
    { label: 'Open Gaps', value: (hub?.gaps ?? []).filter((g: any) => g.status === 'Open').length },
    { label: 'Security Reqs', value: hub?.securityRequirements?.length ?? 0 },
    { label: 'Threats', value: hub?.threats?.length ?? 0 },
    { label: 'NFRs (AI)', value: hub?.nfrs?.length ?? 0 },
    { label: 'Suggestions', value: hub?.suggestions?.length ?? 0 },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h3 className="text-lg font-semibold">Generate Deliverables</h3>
        <p className="text-sm text-muted-foreground mt-1">
          Export professional documents from this roadmap item's requirements and AI analysis.
        </p>
      </div>

      {/* Data readiness summary */}
      <Card className="border-dashed">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground uppercase tracking-wide">
            Data Available for Export
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-6 gap-3">
            {stats.map(s => (
              <div key={s.label} className="text-center">
                <div className={`text-2xl font-bold ${s.value > 0 ? 'text-primary' : 'text-muted-foreground/40'}`}>
                  {s.value}
                </div>
                <div className="text-xs text-muted-foreground leading-tight">{s.label}</div>
                {s.value > 0
                  ? <CheckCircle2 className="w-3 h-3 text-green-500 mx-auto mt-0.5" />
                  : <AlertCircle className="w-3 h-3 text-muted-foreground/30 mx-auto mt-0.5" />}
              </div>
            ))}
          </div>
          {!hasSomeData && (
            <p className="text-sm text-muted-foreground text-center mt-3">
              Add requirements manually or run <strong>Analyze with AI</strong> to populate data for export.
            </p>
          )}
        </CardContent>
      </Card>

      {/* Deliverable cards */}
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        {deliverables.map(d => (
          <Card key={d.id} className={`border-2 ${d.color}`}>
            <CardContent className="pt-5">
              <div className="flex items-start gap-4">
                <div className="shrink-0">{d.icon}</div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <h4 className="font-semibold">{d.title}</h4>
                    <Badge variant="outline" className="text-xs">{d.format}</Badge>
                  </div>
                  <p className="text-sm text-muted-foreground mb-3">{d.description}</p>

                  {/* Sheet/Slide list */}
                  <div className="space-y-1 mb-4">
                    {(d.sheets ?? d.slides ?? []).map((item, i) => (
                      <div key={i} className="flex items-center gap-2 text-xs text-muted-foreground">
                        <FileText className="w-3 h-3 shrink-0" />
                        <span>{d.sheets ? `Sheet ${i + 1}:` : `Slide ${i + 1}:`} {item}</span>
                      </div>
                    ))}
                  </div>

                  {d.missingData && (
                    <p className="text-xs text-orange-600 mb-3 flex items-center gap-1">
                      <AlertCircle className="w-3 h-3 shrink-0" />
                      {d.missingData}
                    </p>
                  )}

                  <Button
                    onClick={d.onGenerate}
                    disabled={generating !== null || !d.isReady}
                    className="w-full"
                    variant={d.isReady ? 'default' : 'outline'}
                  >
                    {generating === d.id ? (
                      <>Generating...</>
                    ) : (
                      <>
                        <Download className="w-4 h-4 mr-2" />
                        Download {d.format.split(' ')[0]}
                      </>
                    )}
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Both at once */}
      {hasSomeData && (
        <Card>
          <CardContent className="pt-5">
            <div className="flex items-center justify-between gap-4">
              <div>
                <h4 className="font-semibold">Download Both</h4>
                <p className="text-sm text-muted-foreground">Generate Excel + PowerPoint in one click</p>
              </div>
              <Button
                size="lg"
                disabled={generating !== null}
                onClick={async () => {
                  setGenerating('both');
                  try {
                    const d = buildExportData();
                    generateRequirementsExcelFile(d);
                    await new Promise(r => setTimeout(r, 500));
                    generateRequirementsPptxFile(d);
                    toast.success('Both deliverables generated — check your downloads');
                  } catch (err) {
                    toast.error('Failed to generate deliverables');
                  } finally {
                    setGenerating(null);
                  }
                }}
              >
                <Download className="w-4 h-4 mr-2" />
                {generating === 'both' ? 'Generating...' : 'Download Both'}
              </Button>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
