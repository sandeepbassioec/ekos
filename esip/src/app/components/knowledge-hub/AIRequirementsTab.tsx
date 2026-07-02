import React, { useState } from 'react';
import {
  Brain, FileText, Video, Mail, MessageSquare, Sparkles, Send, Trash2,
  Check, Loader2, AlertCircle, AlertTriangle, GitMerge, Tag, Zap, Info,
  HelpCircle, Lightbulb, ChevronDown, ChevronUp, CheckSquare, TestTube,
  Users, Target, Shield, Globe, BookOpen,
} from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
import { Badge } from '../ui/badge';
import { Checkbox } from '../ui/checkbox';
import { Alert, AlertDescription } from '../ui/alert';
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription,
} from '../ui/dialog';
import { useKnowledgeHubStore } from '../../store/knowledgeHubStore';
import { useWorkflowStore } from '../../store/workflowStore';
import type { AIRequirement, RequirementPriority, RequirementCategory } from '../../types/knowledge-hub';
import { toast } from 'sonner';

interface AIRequirementsTabProps {
  roadmapItemId: string;
  projectId: string;
  requirements: AIRequirement[];
}

export function AIRequirementsTab({ roadmapItemId, projectId, requirements }: AIRequirementsTabProps) {
  const analyzeAllSources = useKnowledgeHubStore((s) => s.analyzeAllSources);
  const deleteAIRequirement = useKnowledgeHubStore((s) => s.deleteAIRequirement);
  const mergeAIRequirements = useKnowledgeHubStore((s) => s.mergeAIRequirements);
  const isAnalyzing = useKnowledgeHubStore((s) => s.isAnalyzing);
  const currentHub = useKnowledgeHubStore((s) => s.currentHub);

  const { raiseRequest } = useWorkflowStore();

  const [selectedRequirements, setSelectedRequirements] = useState<Set<string>>(new Set());
  const [expandedCards, setExpandedCards] = useState<Set<string>>(new Set());
  const [showSuccess, setShowSuccess] = useState(false);
  const [showNoDocDialog, setShowNoDocDialog] = useState(false);
  const [isMerging, setIsMerging] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const hasDocuments = (currentHub?.stats.totalDocuments ?? 0) > 0;
  const totalMeetings = currentHub?.stats.totalMeetings ?? 0;
  const totalEmails = currentHub?.stats.totalEmails ?? 0;
  const totalChats = currentHub?.stats.totalChats ?? 0;
  const secondarySourceCount = totalMeetings + totalEmails + totalChats;
  const totalSources = (currentHub?.stats.totalDocuments ?? 0) + secondarySourceCount;
  const hasAnySources = totalSources > 0;

  const runAnalysis = async (noDocumentMode = false) => {
    setShowSuccess(false);
    setShowNoDocDialog(false);
    const success = await analyzeAllSources(roadmapItemId, noDocumentMode);
    if (success) {
      setShowSuccess(true);
      setTimeout(() => setShowSuccess(false), 5000);
    }
  };

  const handleAnalyzeClick = () => {
    if (!hasDocuments && secondarySourceCount > 0) {
      setShowNoDocDialog(true);
      return;
    }
    runAnalysis(false);
  };

  const handleDelete = async (requirementId: string) => {
    await deleteAIRequirement(requirementId);
    setSelectedRequirements((prev) => {
      const next = new Set(prev);
      next.delete(requirementId);
      return next;
    });
  };

  const toggleRequirement = (requirementId: string) => {
    setSelectedRequirements((prev) => {
      const next = new Set(prev);
      if (next.has(requirementId)) next.delete(requirementId);
      else next.add(requirementId);
      return next;
    });
  };

  const toggleAll = () => {
    if (selectedRequirements.size === requirements.length) {
      setSelectedRequirements(new Set());
    } else {
      setSelectedRequirements(new Set(requirements.map(r => r.id)));
    }
  };

  const handleMerge = async () => {
    if (selectedRequirements.size < 2) return;
    setIsMerging(true);
    try {
      const ids = Array.from(selectedRequirements);
      const ok = await mergeAIRequirements(ids);
      if (ok) setSelectedRequirements(new Set());
    } finally {
      setIsMerging(false);
    }
  };

  const handleSubmitForReview = async () => {
    const selected = requirements.filter(r => selectedRequirements.has(r.id));
    if (!selected.length) return;

    const aiRequirements = selected.map(aiReq => ({
      aiRequirementId: aiReq.id,
      suggestedTitle: aiReq.requirement.substring(0, 100),
      suggestedDescription: aiReq.description || aiReq.requirement,
      suggestedType: mapCategoryToType(aiReq.category),
      suggestedPriority: (aiReq.priority ?? 'medium').toLowerCase(),
      source: aiReq.source,
      sourceType: aiReq.sourceType,
      sourceId: aiReq.sourceId,
      confidence: aiReq.confidence,
      category: aiReq.category,
    }));

    setIsSubmitting(true);
    try {
      await raiseRequest({
        type: 'review',
        title: `AI Requirements Review — ${selected.length} requirement${selected.length !== 1 ? 's' : ''} from knowledge hub`,
        description: `Please review and approve ${selected.length} AI-extracted requirement${selected.length !== 1 ? 's' : ''} before they are added to the project requirements board.`,
        targetRole: 'manager',
        projectId,
        subjectType: 'ai-requirements-batch',
        subjectId: roadmapItemId,
        metadata: { projectId, roadmapItemId, aiRequirements },
      });
      setSelectedRequirements(new Set());
      toast.success(`${selected.length} requirements submitted for manager review — check the Inbox.`);
    } catch {
      toast.error('Failed to submit for review. Please try again.');
    } finally {
      setIsSubmitting(false);
    }
  };

  const mapCategoryToType = (category?: RequirementCategory): string => {
    switch (category) {
      case 'Functional': return 'feature_change';
      case 'Non-Functional': return 'performance';
      case 'Security': return 'security';
      case 'Performance': return 'performance';
      case 'Usability': return 'ui_ux_change';
      default: return 'feature_change';
    }
  };

  const getSourceIcon = (sourceType: string) => {
    switch (sourceType) {
      case 'Document': return <FileText className="w-3.5 h-3.5" />;
      case 'Meeting': return <Video className="w-3.5 h-3.5" />;
      case 'Email': return <Mail className="w-3.5 h-3.5" />;
      case 'Chat': return <MessageSquare className="w-3.5 h-3.5" />;
      default: return <FileText className="w-3.5 h-3.5" />;
    }
  };

  const priorityStyle = (priority?: RequirementPriority) => {
    switch (priority) {
      case 'High': return 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300';
      case 'Medium': return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300';
      case 'Low': return 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300';
      default: return 'bg-gray-100 text-gray-700';
    }
  };

  const categoryStyle = (category?: RequirementCategory) => {
    switch (category) {
      case 'Functional': return 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300';
      case 'Non-Functional': return 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300';
      case 'Security': return 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300';
      case 'Performance': return 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300';
      case 'Usability': return 'bg-cyan-100 text-cyan-800 dark:bg-cyan-900/30 dark:text-cyan-300';
      case 'Scalability': return 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-300';
      default: return 'bg-gray-100 text-gray-700';
    }
  };

  const techCategoryStyle = (cat?: string) => {
    switch (cat) {
      case 'UI': return 'bg-indigo-100 text-indigo-800';
      case 'API': return 'bg-emerald-100 text-emerald-800';
      case 'Database': return 'bg-amber-100 text-amber-800';
      case 'Integration': return 'bg-teal-100 text-teal-800';
      case 'Security': return 'bg-red-100 text-red-800';
      case 'Infrastructure': return 'bg-slate-100 text-slate-800';
      default: return 'bg-gray-100 text-gray-700';
    }
  };

  const toggleExpand = (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    setExpandedCards((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const typeStyle = (type?: string) => {
    switch (type) {
      case 'Functional': return 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300';
      case 'Non-Functional': return 'bg-purple-100 text-purple-800';
      case 'Security': return 'bg-red-100 text-red-800';
      case 'Performance': return 'bg-green-100 text-green-800';
      case 'Compliance': return 'bg-amber-100 text-amber-800';
      case 'Accessibility': return 'bg-cyan-100 text-cyan-800';
      case 'Migration': return 'bg-orange-100 text-orange-800';
      case 'Training': return 'bg-pink-100 text-pink-800';
      default: return 'bg-gray-100 text-gray-700';
    }
  };

  const analyzeButtonLabel = () => {
    if (isAnalyzing) return null;
    if (requirements.length > 0) {
      if (hasDocuments && secondarySourceCount > 0) return 'Re-analyze & Refine';
      return 'Regenerate Requirements';
    }
    if (hasDocuments && secondarySourceCount > 0) return 'Analyze & Refine';
    return 'Analyze All Sources';
  };

  const selectedCount = selectedRequirements.size;

  return (
    <div className="space-y-4">

      {/* No-document confirmation dialog */}
      <Dialog open={showNoDocDialog} onOpenChange={setShowNoDocDialog}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="w-5 h-5 text-orange-500" />
              No Requirements Document Found
            </DialogTitle>
            <DialogDescription asChild>
              <div className="space-y-3 pt-1">
                <p className="text-sm text-muted-foreground">
                  You have uploaded{' '}
                  {[
                    totalMeetings > 0 && `${totalMeetings} meeting${totalMeetings > 1 ? 's' : ''}`,
                    totalEmails > 0 && `${totalEmails} email${totalEmails > 1 ? 's' : ''}`,
                    totalChats > 0 && `${totalChats} chat${totalChats > 1 ? 's' : ''}`,
                  ].filter(Boolean).join(', ')}{' '}
                  but <strong>no requirements document</strong> yet.
                </p>
                <p className="text-sm text-muted-foreground">
                  For best results, upload a BRD / requirements document first so the AI can use it as a baseline and refine it using your meetings, emails, and chats.
                </p>
                <p className="text-sm font-medium">
                  Would you like to analyze all sources together and generate requirements from scratch?
                </p>
              </div>
            </DialogDescription>
          </DialogHeader>
          <div className="flex gap-2 justify-end pt-2">
            <Button variant="outline" onClick={() => setShowNoDocDialog(false)}>
              Cancel — I'll upload a document first
            </Button>
            <Button onClick={() => runAnalysis(true)}>
              <Sparkles className="w-4 h-4 mr-2" />
              Yes, Analyze All Together
            </Button>
          </div>
        </DialogContent>
      </Dialog>

      {/* Header */}
      <div className="flex justify-between items-start gap-4">
        <div>
          <h3 className="text-lg font-semibold">AI-Derived Requirements</h3>
          <p className="text-sm text-muted-foreground">
            {hasDocuments && secondarySourceCount > 0
              ? 'Documents analyzed first as baseline — meetings, emails & chats refine the requirements'
              : hasDocuments
              ? 'Requirements extracted from documents using the Requirements Analyst prompt'
              : 'All sources analyzed together to generate requirements'}
          </p>
        </div>
        <Button onClick={handleAnalyzeClick} disabled={isAnalyzing || !hasAnySources} data-analyze-btn>
          {isAnalyzing ? (
            <>
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
              {hasDocuments && secondarySourceCount > 0 ? 'Analyzing & Refining...' : 'Analyzing...'}
            </>
          ) : (
            <>
              <Sparkles className="w-4 h-4 mr-2" />
              {analyzeButtonLabel()}
            </>
          )}
        </Button>
      </div>

      {/* Success */}
      {showSuccess && (
        <Alert className="border-green-200 bg-green-50">
          <Check className="h-4 w-4 text-green-600" />
          <AlertDescription className="text-green-800">
            Successfully analyzed {totalSources} source{totalSources !== 1 ? 's' : ''} and extracted {requirements.length} requirement{requirements.length !== 1 ? 's' : ''}!
          </AlertDescription>
        </Alert>
      )}

      {/* No sources warning */}
      {!hasAnySources && (
        <Alert>
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>
            Add documents, meetings, emails, or chats first to enable AI requirement extraction.
          </AlertDescription>
        </Alert>
      )}

      {/* Analysis mode info banner */}
      {hasAnySources && hasDocuments && secondarySourceCount > 0 && (
        <Alert className="border-blue-200 bg-blue-50">
          <Brain className="h-4 w-4 text-blue-600" />
          <AlertDescription className="text-blue-800 text-sm">
            <strong>Two-phase analysis:</strong> Documents → baseline requirements.
            Then meetings, emails & chats refine gaps, confusions and suggestions against that baseline.
          </AlertDescription>
        </Alert>
      )}

      {/* Sources Summary */}
      {hasAnySources && (
        <div className="grid grid-cols-4 gap-3">
          {[
            { icon: <FileText className="w-5 h-5 text-blue-600" />, bg: 'bg-blue-100', count: currentHub?.stats.totalDocuments ?? 0, label: 'Documents', note: 'Baseline' },
            { icon: <Video className="w-5 h-5 text-purple-600" />, bg: 'bg-purple-100', count: totalMeetings, label: 'Meetings', note: 'Refines' },
            { icon: <Mail className="w-5 h-5 text-green-600" />, bg: 'bg-green-100', count: totalEmails, label: 'Emails', note: 'Refines' },
            { icon: <MessageSquare className="w-5 h-5 text-orange-600" />, bg: 'bg-orange-100', count: totalChats, label: 'Chats', note: 'Refines' },
          ].map(({ icon, bg, count, label, note }) => (
            <Card key={label} className="border-2">
              <CardContent className="p-4 flex items-center gap-3">
                <div className={`p-2 ${bg} rounded-lg`}>{icon}</div>
                <div>
                  <div className="text-2xl font-bold">{count}</div>
                  <div className="text-xs text-muted-foreground">{label}</div>
                  {hasDocuments && secondarySourceCount > 0 && (
                    <div className="text-xs text-muted-foreground/60 italic">{note}</div>
                  )}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Requirements List */}
      {requirements.length > 0 ? (
        <>
          {/* Selection toolbar */}
          <div className="flex items-center justify-between gap-3 px-4 py-3 bg-muted/50 rounded-lg">
            <div className="flex items-center gap-3">
              <Checkbox
                checked={selectedCount === requirements.length && requirements.length > 0}
                onCheckedChange={toggleAll}
              />
              <span className="text-sm font-medium">
                {selectedCount > 0
                  ? `${selectedCount} of ${requirements.length} selected`
                  : `Select all ${requirements.length} requirements`}
              </span>
            </div>
            {selectedCount > 0 && (
              <div className="flex items-center gap-2">
                {selectedCount >= 2 && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleMerge}
                    disabled={isMerging}
                    className="border-amber-400 text-amber-700 hover:bg-amber-50"
                  >
                    {isMerging ? (
                      <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                    ) : (
                      <GitMerge className="w-4 h-4 mr-2" />
                    )}
                    {isMerging ? 'Merging…' : `Merge ${selectedCount}`}
                  </Button>
                )}
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleSubmitForReview}
                  disabled={isSubmitting}
                >
                  {isSubmitting ? (
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  ) : (
                    <Send className="w-4 h-4 mr-2" />
                  )}
                  {isSubmitting ? 'Submitting…' : `Submit ${selectedCount} for Review`}
                </Button>
              </div>
            )}
          </div>

          <div className="grid gap-3">
            {requirements.map((req) => {
              const isExpanded = expandedCards.has(req.id);
              const r = req as any;
              const linkedGaps = (currentHub?.gaps || []).filter((g: any) => (g.relatedRequirementIds || []).includes(req.id));
              const linkedConfusions = (currentHub?.confusions || []).filter((c: any) => (c.relatedRequirementIds || []).includes(req.id));
              const linkedSuggestions = (currentHub?.suggestions || []).filter((s: any) => (s.relatedRequirementIds || []).includes(req.id));
              const hasExpandable = r.businessValue || r.acceptanceCriteria?.length || r.testScenarios?.length ||
                r.qualityIssues?.length || r.assumptions?.length || r.stakeholders?.length ||
                r.complianceFrameworks?.length || linkedGaps.length || linkedConfusions.length || linkedSuggestions.length;

              return (
                <Card
                  key={req.id}
                  className={`transition-shadow ${selectedRequirements.has(req.id) ? 'ring-2 ring-primary' : 'hover:shadow-md'}`}
                >
                  <CardContent className="p-4 space-y-3">
                    {/* Row 1: checkbox + key chip + badges + expand + delete */}
                    <div className="flex items-start gap-3 cursor-pointer" onClick={() => toggleRequirement(req.id)}>
                      <Checkbox
                        checked={selectedRequirements.has(req.id)}
                        onCheckedChange={() => toggleRequirement(req.id)}
                        onClick={(e) => e.stopPropagation()}
                        className="mt-0.5 shrink-0"
                      />
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 flex-wrap mb-2">
                          {r.requirementKey && (
                            <code className="text-[11px] font-mono bg-muted px-1.5 py-0.5 rounded text-muted-foreground">
                              {r.requirementKey}
                            </code>
                          )}
                          {r.type && (
                            <Badge className={`text-xs ${typeStyle(r.type)}`}>{r.type}</Badge>
                          )}
                          {req.category && (
                            <Badge className={`text-xs ${categoryStyle(req.category)}`}>{req.category}</Badge>
                          )}
                          {req.priority && (
                            <Badge className={`text-xs ${priorityStyle(req.priority)}`}>{req.priority}</Badge>
                          )}
                          {req.confidence != null && (
                            <Badge variant="outline" className="text-xs">
                              {Math.round(req.confidence)}% confident
                            </Badge>
                          )}
                          {r.complianceFrameworks?.length > 0 && (
                            <Badge className="text-xs bg-amber-50 text-amber-700 border border-amber-200">
                              <Shield className="w-2.5 h-2.5 mr-1" />
                              {r.complianceFrameworks[0]}{r.complianceFrameworks.length > 1 ? ` +${r.complianceFrameworks.length - 1}` : ''}
                            </Badge>
                          )}
                        </div>
                        <p className="text-sm font-medium leading-relaxed text-foreground">
                          {req.requirement}
                        </p>
                      </div>
                      <div className="flex items-center gap-1 shrink-0">
                        {hasExpandable && (
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={(e) => toggleExpand(e, req.id)}
                            className="h-8 w-8 p-0"
                            title={isExpanded ? 'Collapse details' : 'Show details'}
                          >
                            {isExpanded ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                          </Button>
                        )}
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={(e) => { e.stopPropagation(); handleDelete(req.id); }}
                          className="h-8 w-8 p-0"
                        >
                          <Trash2 className="w-4 h-4 text-destructive" />
                        </Button>
                      </div>
                    </div>

                    {/* Row 2: description */}
                    {r.description && (
                      <p className="text-sm text-muted-foreground pl-7 leading-snug">
                        {r.description}
                      </p>
                    )}

                    {/* Row 3: tech category + change type + source */}
                    <div className="flex items-center gap-2 pl-7 flex-wrap">
                      {r.technicalCategory && (
                        <Badge className={`text-xs ${techCategoryStyle(r.technicalCategory)}`}>
                          <Tag className="w-3 h-3 mr-1" />
                          {r.technicalCategory}
                        </Badge>
                      )}
                      {r.changeType && (
                        <Badge variant="outline" className="text-xs">
                          <Zap className="w-3 h-3 mr-1" />
                          {r.changeType}
                        </Badge>
                      )}
                      <div className="flex items-center gap-1 text-xs text-muted-foreground ml-auto">
                        {getSourceIcon(req.sourceType)}
                        <span className="truncate max-w-[180px]">{req.source}</span>
                      </div>
                    </div>

                    {/* Row 4: rationale (always visible) */}
                    {r.rationale && (
                      <div className="pl-7 flex items-start gap-1.5">
                        <Info className="w-3.5 h-3.5 text-muted-foreground shrink-0 mt-0.5" />
                        <p className="text-xs text-muted-foreground italic leading-snug">{r.rationale}</p>
                      </div>
                    )}

                    {/* EXPANDED SECTION */}
                    {isExpanded && (
                      <div className="pl-7 space-y-3 pt-2 border-t">

                        {/* Business Value */}
                        {r.businessValue && (
                          <div className="flex items-start gap-2">
                            <Target className="w-3.5 h-3.5 text-green-600 shrink-0 mt-0.5" />
                            <div>
                              <p className="text-xs font-semibold text-green-700 mb-0.5">Business Value</p>
                              <p className="text-xs text-muted-foreground leading-snug">{r.businessValue}</p>
                            </div>
                          </div>
                        )}

                        {/* Acceptance Criteria */}
                        {r.acceptanceCriteria?.length > 0 && (
                          <div className="flex items-start gap-2">
                            <CheckSquare className="w-3.5 h-3.5 text-blue-600 shrink-0 mt-0.5" />
                            <div>
                              <p className="text-xs font-semibold text-blue-700 mb-1">Acceptance Criteria</p>
                              <ul className="space-y-0.5">
                                {r.acceptanceCriteria.map((ac: string, i: number) => (
                                  <li key={i} className="text-xs text-muted-foreground flex items-start gap-1.5">
                                    <span className="text-blue-400 shrink-0">✓</span>
                                    <span>{ac}</span>
                                  </li>
                                ))}
                              </ul>
                            </div>
                          </div>
                        )}

                        {/* Test Scenarios */}
                        {r.testScenarios?.length > 0 && (
                          <div className="flex items-start gap-2">
                            <TestTube className="w-3.5 h-3.5 text-purple-600 shrink-0 mt-0.5" />
                            <div>
                              <p className="text-xs font-semibold text-purple-700 mb-1">Test Scenarios</p>
                              <ul className="space-y-0.5">
                                {r.testScenarios.map((ts: string, i: number) => (
                                  <li key={i} className="text-xs text-muted-foreground flex items-start gap-1.5">
                                    <span className="text-purple-400 shrink-0">›</span>
                                    <span>{ts}</span>
                                  </li>
                                ))}
                              </ul>
                            </div>
                          </div>
                        )}

                        {/* Quality Issues */}
                        {r.qualityIssues?.length > 0 && (
                          <div className="flex items-start gap-2">
                            <AlertTriangle className="w-3.5 h-3.5 text-orange-500 shrink-0 mt-0.5" />
                            <div>
                              <p className="text-xs font-semibold text-orange-600 mb-1">Quality Issues</p>
                              <ul className="space-y-0.5">
                                {r.qualityIssues.map((qi: string, i: number) => (
                                  <li key={i} className="text-xs text-orange-700 bg-orange-50 rounded px-2 py-0.5">{qi}</li>
                                ))}
                              </ul>
                            </div>
                          </div>
                        )}

                        {/* Stakeholders + Assumptions row */}
                        <div className="flex gap-4 flex-wrap">
                          {r.stakeholders?.length > 0 && (
                            <div className="flex items-start gap-2">
                              <Users className="w-3.5 h-3.5 text-slate-500 shrink-0 mt-0.5" />
                              <div>
                                <p className="text-xs font-semibold text-slate-600 mb-1">Stakeholders</p>
                                <div className="flex gap-1 flex-wrap">
                                  {r.stakeholders.map((s: string, i: number) => (
                                    <span key={i} className="text-xs bg-slate-100 text-slate-700 px-2 py-0.5 rounded-full">{s}</span>
                                  ))}
                                </div>
                              </div>
                            </div>
                          )}
                          {r.assumptions?.length > 0 && (
                            <div className="flex items-start gap-2">
                              <BookOpen className="w-3.5 h-3.5 text-yellow-600 shrink-0 mt-0.5" />
                              <div>
                                <p className="text-xs font-semibold text-yellow-700 mb-1">Assumptions</p>
                                <ul className="space-y-0.5">
                                  {r.assumptions.map((a: string, i: number) => (
                                    <li key={i} className="text-xs text-muted-foreground flex items-start gap-1">
                                      <span className="text-yellow-500">!</span>
                                      <span>{a}</span>
                                    </li>
                                  ))}
                                </ul>
                              </div>
                            </div>
                          )}
                        </div>

                        {/* Compliance Frameworks */}
                        {r.complianceFrameworks?.length > 0 && (
                          <div className="flex items-start gap-2">
                            <Globe className="w-3.5 h-3.5 text-amber-600 shrink-0 mt-0.5" />
                            <div>
                              <p className="text-xs font-semibold text-amber-700 mb-1">Compliance Frameworks</p>
                              <div className="flex gap-1 flex-wrap">
                                {r.complianceFrameworks.map((cf: string, i: number) => (
                                  <span key={i} className="text-xs bg-amber-50 text-amber-700 border border-amber-200 px-2 py-0.5 rounded">{cf}</span>
                                ))}
                              </div>
                            </div>
                          </div>
                        )}

                        {/* Linked items */}
                        {(linkedGaps.length > 0 || linkedConfusions.length > 0 || linkedSuggestions.length > 0) && (
                          <div className="flex items-start gap-2 flex-wrap pt-1 border-t">
                            <span className="text-xs text-muted-foreground font-medium shrink-0 mt-0.5">Linked:</span>
                            {linkedGaps.map((g: any) => (
                              <span key={g.id} className="inline-flex items-center gap-1 text-xs bg-orange-50 text-orange-700 border border-orange-200 px-2 py-0.5 rounded-full">
                                <AlertTriangle className="w-2.5 h-2.5" />
                                {g.title.length > 36 ? g.title.slice(0, 36) + '…' : g.title}
                              </span>
                            ))}
                            {linkedConfusions.map((c: any) => (
                              <span key={c.id} className="inline-flex items-center gap-1 text-xs bg-blue-50 text-blue-700 border border-blue-200 px-2 py-0.5 rounded-full">
                                <HelpCircle className="w-2.5 h-2.5" />
                                {c.question.length > 36 ? c.question.slice(0, 36) + '…' : c.question}
                              </span>
                            ))}
                            {linkedSuggestions.map((s: any) => (
                              <span key={s.id} className="inline-flex items-center gap-1 text-xs bg-yellow-50 text-yellow-700 border border-yellow-200 px-2 py-0.5 rounded-full">
                                <Lightbulb className="w-2.5 h-2.5" />
                                {s.title.length > 36 ? s.title.slice(0, 36) + '…' : s.title}
                              </span>
                            ))}
                          </div>
                        )}
                      </div>
                    )}

                    {/* Collapsed linked summary (when not expanded) */}
                    {!isExpanded && (linkedGaps.length > 0 || linkedConfusions.length > 0 || linkedSuggestions.length > 0) && (
                      <div className="pl-7 flex items-center gap-2 flex-wrap">
                        <span className="text-xs text-muted-foreground font-medium shrink-0">Linked:</span>
                        {linkedGaps.map((g: any) => (
                          <span key={g.id} className="inline-flex items-center gap-1 text-xs bg-orange-50 text-orange-700 border border-orange-200 px-2 py-0.5 rounded-full">
                            <AlertTriangle className="w-2.5 h-2.5" />
                            {g.title.length > 32 ? g.title.slice(0, 32) + '…' : g.title}
                          </span>
                        ))}
                        {linkedConfusions.map((c: any) => (
                          <span key={c.id} className="inline-flex items-center gap-1 text-xs bg-blue-50 text-blue-700 border border-blue-200 px-2 py-0.5 rounded-full">
                            <HelpCircle className="w-2.5 h-2.5" />
                            {c.question.length > 32 ? c.question.slice(0, 32) + '…' : c.question}
                          </span>
                        ))}
                        {linkedSuggestions.map((s: any) => (
                          <span key={s.id} className="inline-flex items-center gap-1 text-xs bg-yellow-50 text-yellow-700 border border-yellow-200 px-2 py-0.5 rounded-full">
                            <Lightbulb className="w-2.5 h-2.5" />
                            {s.title.length > 32 ? s.title.slice(0, 32) + '…' : s.title}
                          </span>
                        ))}
                      </div>
                    )}
                  </CardContent>
                </Card>
              );
            })}
          </div>
        </>
      ) : hasAnySources ? (
        <Card className="border-2 border-dashed border-primary/30 bg-primary/5">
          <CardContent className="py-12 text-center">
            <Brain className="w-16 h-16 mx-auto mb-4 text-primary" />
            <h3 className="text-lg font-semibold mb-2">Ready to Extract Requirements!</h3>
            <p className="text-muted-foreground mb-6 max-w-2xl mx-auto">
              {hasDocuments && secondarySourceCount > 0
                ? `Documents will be analyzed first to build the baseline. Then ${secondarySourceCount} secondary source${secondarySourceCount > 1 ? 's' : ''} will refine gaps, confusions and suggestions.`
                : hasDocuments
                ? `${currentHub?.stats.totalDocuments} document${(currentHub?.stats.totalDocuments ?? 0) > 1 ? 's' : ''} ready for deep requirements analysis.`
                : `${secondarySourceCount} source${secondarySourceCount > 1 ? 's' : ''} ready. No document baseline — you will be asked to confirm before analysis.`}
            </p>
            <div className="space-y-2 text-sm text-left max-w-xl mx-auto">
              <div className="flex items-start gap-2">
                <span className="text-primary mt-0.5">✓</span>
                <span className="text-muted-foreground">Documents analyzed with Principal Requirements Analyst prompt (BABOK v3 / IEEE 830)</span>
              </div>
              <div className="flex items-start gap-2">
                <span className="text-primary mt-0.5">✓</span>
                <span className="text-muted-foreground">Meetings, emails & chats refine the baseline — extracting decisions, gaps and scope changes</span>
              </div>
              <div className="flex items-start gap-2">
                <span className="text-primary mt-0.5">✓</span>
                <span className="text-muted-foreground">Each requirement gets a unique key so re-runs never create duplicates</span>
              </div>
              <div className="flex items-start gap-2">
                <span className="text-primary mt-0.5">✓</span>
                <span className="text-muted-foreground">Select 2+ requirements and merge them into one with AI assistance</span>
              </div>
            </div>
          </CardContent>
        </Card>
      ) : null}
    </div>
  );
}
