import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import { useParams, useNavigate } from 'react-router';
import {
  ArrowLeft, FileText, Video, Mail, MessageSquare, Brain, Search,
  AlertTriangle, HelpCircle, Lightbulb, Shield, Swords, Gauge, Cloud, ClipboardList, Download, Zap,
} from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../components/ui/tabs';
import { usePersonaStore, getPersona } from '../store/personaStore';
import { Input } from '../components/ui/input';
import { Badge } from '../components/ui/badge';
import { useKnowledgeHubStore } from '../store/knowledgeHubStore';
import { db } from '../lib/db';
import { getFirestore, doc, getDoc } from 'firebase/firestore';
import { toast } from 'sonner';

import { DocumentsTab } from '../components/knowledge-hub/DocumentsTab';
import { MeetingsTab } from '../components/knowledge-hub/MeetingsTab';
import { EmailsTab } from '../components/knowledge-hub/EmailsTab';
import { ChatsTab } from '../components/knowledge-hub/ChatsTab';
import { AIRequirementsTab } from '../components/knowledge-hub/AIRequirementsTab';
import { GapsTab } from '../components/knowledge-hub/GapsTab';
import { ConfusionsTab } from '../components/knowledge-hub/ConfusionsTab';
import { SuggestionsTab } from '../components/knowledge-hub/SuggestionsTab';
import { ConflictsTab } from '../components/knowledge-hub/ConflictsTab';
import { SecurityRequirementsTab } from '../components/knowledge-hub/SecurityRequirementsTab';
import { ThreatsTab } from '../components/knowledge-hub/ThreatsTab';
import { NFRsTab } from '../components/knowledge-hub/NFRsTab';
import { CloudGovernanceTab } from '../components/knowledge-hub/CloudGovernanceTab';
import { RequirementsTab } from '../components/knowledge-hub/RequirementsTab';
import { DeliverablesTab } from '../components/knowledge-hub/DeliverablesTab';

export default function KnowledgeHubPage() {
  const { roadmapItemId } = useParams();
  const navigate = useNavigate();

  const { currentHub, isLoading, fetchKnowledgeHub, aiSearch, runCommand, isSearching, lastSearchResult } = useKnowledgeHubStore();

  const [roadmapItem, setRoadmapItem] = useState<any>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [activeTab, setActiveTab] = useState('documents');
  // Persona-aware AI Insights (Epic C): the active persona foregrounds a subset of insight tabs.
  // Inputs + Requirements/Deliverables stay universal; only the AI-Insights row is scoped.
  const activePersonaId = usePersonaStore((s) => s.activePersonaId);
  const insightTabs = getPersona(activePersonaId).knowledgeHubTabs;
  const showInsight = (key: string) => !insightTabs || insightTabs.length === 0 || insightTabs.includes(key);
  const [showSearchResult, setShowSearchResult] = useState(false);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [highlightedIndex, setHighlightedIndex] = useState(-1);
  const searchInputRef = useRef<HTMLInputElement>(null);

  const handleSearch = async (query?: string) => {
    const q = (query ?? searchQuery).trim();
    if (!q) return;
    setSearchQuery(q);
    setShowSuggestions(false);
    setShowSearchResult(true);
    await aiSearch(q);
  };

  // ── Slash command registry ─────────────────────────────────────────────────
  type CommandCategory = 'navigation' | 'ai' | 'export' | 'filter';
  interface SlashCommand {
    command: string;
    label: string;
    description: string;
    icon: string;
    category: CommandCategory;
  }
  const COMMANDS: SlashCommand[] = [
    // Navigation
    { command: '/requirements', label: '/requirements', description: 'Jump to Formal Requirements tab',     icon: '📋', category: 'navigation' },
    { command: '/gaps',         label: '/gaps',         description: 'Jump to Gaps tab',                    icon: '⚠️', category: 'navigation' },
    { command: '/threats',      label: '/threats',      description: 'Jump to Threat Analysis tab',         icon: '🔐', category: 'navigation' },
    { command: '/security',     label: '/security',     description: 'Jump to Security Requirements tab',   icon: '🛡️', category: 'navigation' },
    { command: '/nfr',          label: '/nfr',          description: 'Jump to Non-Functional Requirements', icon: '📊', category: 'navigation' },
    { command: '/documents',    label: '/documents',    description: 'Jump to Documents tab',               icon: '📄', category: 'navigation' },
    { command: '/meetings',     label: '/meetings',     description: 'Jump to Meetings tab',                icon: '🎥', category: 'navigation' },
    { command: '/emails',       label: '/emails',       description: 'Jump to Emails tab',                  icon: '📧', category: 'navigation' },
    { command: '/chats',        label: '/chats',        description: 'Jump to Chats tab',                   icon: '💬', category: 'navigation' },
    // AI actions
    { command: '/analyze',      label: '/analyze',      description: 'Run full AI analysis of all sources', icon: '✨', category: 'ai' },
    { command: '/summary',      label: '/summary',      description: 'Generate executive summary',          icon: '📝', category: 'ai' },
    { command: '/openitems',    label: '/openitems',    description: 'List all unresolved gaps & confusions',icon: '🔓', category: 'ai' },
    { command: '/risks',        label: '/risks',        description: 'Generate risk brief from threats',    icon: '🚨', category: 'ai' },
    // Export
    { command: '/excel',        label: '/excel',        description: 'Download Requirements Excel',         icon: '📗', category: 'export' },
    { command: '/pptx',         label: '/pptx',         description: 'Download PowerPoint presentation',    icon: '📘', category: 'export' },
    { command: '/export',       label: '/export',       description: 'Download both Excel + PowerPoint',    icon: '📦', category: 'export' },
    // Filters
    { command: '/critical',     label: '/critical',     description: 'Show all critical priority items',    icon: '🔴', category: 'filter' },
    { command: '/approved',     label: '/approved',     description: 'Show all approved requirements',      icon: '✅', category: 'filter' },
    { command: '/pending',      label: '/pending',      description: 'Show items needing review',           icon: '🕐', category: 'filter' },
  ];

  const CATEGORY_LABELS: Record<CommandCategory, string> = {
    navigation: 'Navigate',
    ai: 'AI Actions',
    export: 'Export',
    filter: 'Filter',
  };

  const isCommandMode = searchQuery.startsWith('/');

  const filteredCommands = useMemo(() => {
    if (!isCommandMode) return [];
    if (searchQuery === '/') return COMMANDS;
    const q = searchQuery.toLowerCase();
    return COMMANDS.filter(c => c.command.startsWith(q) || c.description.toLowerCase().includes(q.slice(1)));
  }, [searchQuery, isCommandMode]);

  // Group filtered commands by category
  const groupedCommands = useMemo(() => {
    const groups: Partial<Record<CommandCategory, SlashCommand[]>> = {};
    filteredCommands.forEach(cmd => {
      if (!groups[cmd.category]) groups[cmd.category] = [];
      groups[cmd.category]!.push(cmd);
    });
    return groups;
  }, [filteredCommands]);

  const handleCommand = async (command: string) => {
    setShowSuggestions(false);
    setSearchQuery(command);

    const result = await runCommand(command);
    if (!result) return;

    if (result.type === 'navigate') {
      setActiveTab(result.tab);
      setSearchQuery('');
      setShowSearchResult(false);
    } else if (result.type === 'export') {
      const hub = currentHub;
      if (!hub) return;
      const { generateRequirementsExcelFile } = await import('../utils/requirementsExcelGenerator');
      const { generateRequirementsPptxFile }  = await import('../utils/requirementsPptxGenerator');
      const exportData = {
        projectName: project?.name ?? '',
        roadmapItemTitle: roadmapItem?.title ?? '',
        roadmapType: roadmapItem?.roadmap?.type ?? '',
        generatedDate: new Date().toLocaleDateString(),
        formalRequirements: hub.formalRequirements ?? [],
        gaps: hub.gaps ?? [], confusions: hub.confusions ?? [],
        suggestions: hub.suggestions ?? [],
        securityRequirements: hub.securityRequirements ?? [],
        threats: hub.threats ?? [], nfrs: hub.nfrs ?? [],
        cloudGovernance: hub.cloudGovernance ?? [],
      };
      if (result.format === 'excel' || result.format === 'both') generateRequirementsExcelFile(exportData);
      if (result.format === 'both') await new Promise(r => setTimeout(r, 400));
      if (result.format === 'pptx'  || result.format === 'both') generateRequirementsPptxFile(exportData);
      setSearchQuery('');
    } else if (result.type === 'text') {
      if (result.content === '__TRIGGER_ANALYZE__') {
        setSearchQuery('');
        // Scroll to the Analyze button and trigger it programmatically
        const analyzeBtn = document.querySelector('[data-analyze-btn]') as HTMLButtonElement;
        if (analyzeBtn) analyzeBtn.click();
        return;
      }
      setShowSearchResult(true);
    }
  };

  // ── Generate contextual suggestions from hub data ──────────────────────────
  const allSuggestions = useMemo(() => {
    if (!currentHub) return [];
    const s: string[] = [];

    const formalReqs   = currentHub.formalRequirements ?? [];
    const gaps         = currentHub.gaps ?? [];
    const confusions   = currentHub.confusions ?? [];
    const threats      = currentHub.threats ?? [];
    const nfrs         = currentHub.nfrs ?? [];
    const secReqs      = currentHub.securityRequirements ?? [];
    const suggestions  = currentHub.suggestions ?? [];
    const docs         = currentHub.documents;
    const meetings     = currentHub.meetings;
    const emails       = currentHub.emails;
    const chats        = currentHub.chats;

    // Always-available questions
    s.push('What are the main requirements for this roadmap item?');
    s.push('Summarise everything captured in this knowledge hub');

    // Document-based
    if (docs.length > 0) {
      s.push('What does the requirements document say about authentication?');
      s.push('What are the key functional requirements from the documents?');
      if (docs.length > 1) s.push(`What are the main differences across the ${docs.length} documents?`);
    }

    // Gaps
    const openGaps = (gaps as any[]).filter((g: any) => g.status === 'Open');
    if (openGaps.length > 0) {
      s.push(`What gaps are still unresolved? (${openGaps.length} open)`);
      s.push('Which gaps are critical or high severity?');
      const titles = openGaps.slice(0, 2).map((g: any) => g.title).filter(Boolean);
      titles.forEach((t: string) => s.push(`Tell me more about the gap: "${t}"`));
    }

    // Confusions
    const openConf = (confusions as any[]).filter((c: any) => c.status === 'Open');
    if (openConf.length > 0) {
      s.push(`What confusions or ambiguities are still open? (${openConf.length})`);
    }

    // Threats
    if (threats.length > 0) {
      s.push('What are the identified security threats?');
      const critical = (threats as any[]).filter((t: any) => t.level === 'Critical' || t.level === 'High');
      if (critical.length > 0) s.push(`What are the ${critical.length} critical or high severity threats?`);
      s.push('What mitigations are recommended for the threats?');
    }

    // Security requirements
    if (secReqs.length > 0) {
      s.push('What are the security requirements?');
      s.push('What authentication and authorization requirements exist?');
    }

    // NFRs
    const nfrCategories = [...new Set([
      ...(nfrs as any[]).map((n: any) => n.category),
      ...(formalReqs as any[]).filter((r: any) => r.category === 'Non-Functional').map((r: any) => r.nfrType),
    ])].filter(Boolean);
    if (nfrCategories.includes('Performance')) s.push('What are the performance targets and SLAs?');
    if (nfrCategories.includes('Availability')) s.push('What are the availability and uptime requirements?');
    if (nfrCategories.includes('Scalability')) s.push('What are the scalability requirements?');
    if (nfrCategories.includes('Compliance')) s.push('What compliance requirements need to be met?');
    if (nfrCategories.length > 0) s.push('List all non-functional requirements with their metrics');

    // Formal requirements
    if (formalReqs.length > 0) {
      s.push(`How many requirements are approved vs draft? (${formalReqs.length} total)`);
      const criticalReqs = (formalReqs as any[]).filter((r: any) => r.priority === 'Critical');
      if (criticalReqs.length > 0) s.push(`What are the ${criticalReqs.length} critical priority requirements?`);
      s.push('Which requirements still need approval?');
    }

    // Meeting-based
    if (meetings.length > 0) {
      s.push('What key decisions were made in the meetings?');
      if (meetings.length > 1) s.push('What were the main topics discussed across all meetings?');
    }

    // Email-based
    if (emails.length > 0) {
      s.push('What stakeholder concerns were raised in emails?');
      s.push('Are there any scope changes mentioned in the emails?');
    }

    // Suggestions
    const accepted = (suggestions as any[]).filter((s: any) => s.status === 'Accepted');
    if (accepted.length > 0) s.push(`What suggestions have been accepted? (${accepted.length})`);
    if (suggestions.length > 0) s.push('What recommendations were made by the AI analysis?');

    return s;
  }, [currentHub]);

  // Filter suggestions by what user has typed
  const filteredSuggestions = useMemo(() => {
    if (!searchQuery.trim()) return allSuggestions.slice(0, 6);
    const q = searchQuery.toLowerCase();
    return allSuggestions.filter(s => s.toLowerCase().includes(q)).slice(0, 8);
  }, [searchQuery, allSuggestions]);

  useEffect(() => {
    const fetchRoadmapItem = async () => {
      if (!roadmapItemId) return;
      try {
        const item = await db.roadmapItems.getById(roadmapItemId);

        // Fetch roadmap — fall back to empty object if missing
        let roadmap: any = { id: item.roadmapId, type: '', projectId: '' };
        try { roadmap = await db.roadmaps.getById(item.roadmapId); } catch {}

        // Fetch project directly without userId auth check
        let project: any = { id: roadmap.projectId, name: 'Project' };
        try {
          const firestore = getFirestore();
          const snap = await getDoc(doc(firestore, 'projects', roadmap.projectId));
          if (snap.exists()) project = { id: snap.id, ...snap.data() };
        } catch {}

        setRoadmapItem({ ...item, roadmap: { ...roadmap, project } });
      } catch (error) {
        console.error('Error fetching roadmap item:', error);
        toast.error('Failed to fetch roadmap item');
      }
    };
    fetchRoadmapItem();
  }, [roadmapItemId]);

  useEffect(() => {
    if (roadmapItemId) {
      fetchKnowledgeHub(roadmapItemId);
    }
  }, [roadmapItemId, fetchKnowledgeHub]);

  const refresh = useCallback(() => {
    if (roadmapItemId) fetchKnowledgeHub(roadmapItemId);
  }, [roadmapItemId, fetchKnowledgeHub]);

  const navigateToSource = useCallback((sourceType: string, _sourceId: string) => {
    const tabMap: Record<string, string> = { Email: 'emails', Meeting: 'meetings', Chat: 'chats', Document: 'documents' };
    const tab = tabMap[sourceType];
    if (tab) setActiveTab(tab);
  }, []);

  if (!roadmapItem) {
    return (
      <div className="container max-w-7xl mx-auto py-8 px-4">
        <Card>
          <CardContent className="py-16 text-center">
            <div className="text-muted-foreground">Loading...</div>
          </CardContent>
        </Card>
      </div>
    );
  }

  const project = roadmapItem.roadmap?.project;
  const roadmap = roadmapItem.roadmap;

  const hub = currentHub;

  // Derived insight counts
  const gapsCount = hub?.gaps?.length ?? 0;
  const confusionsCount = hub?.confusions?.length ?? 0;
  const suggestionsCount = hub?.suggestions?.length ?? 0;
  const securityCount = hub?.securityRequirements?.length ?? 0;
  const threatsCount = hub?.threats?.length ?? 0;
  const nfrsCount = hub?.nfrs?.length ?? 0;
  const cloudCount = hub?.cloudGovernance?.length ?? 0;
  const formalReqCount = hub?.formalRequirements?.length ?? 0;
  const conflictsCount = hub?.stats?.totalConflicts ?? 0;
  const coverageScore = hub?.stats?.coverageScore ?? 100;
  const openConflictsCount = (hub?.conflicts ?? []).filter((c: any) => c.status === 'Open').length;
  const totalInsights = gapsCount + confusionsCount + suggestionsCount + securityCount + threatsCount + nfrsCount + cloudCount + conflictsCount;

  return (
    <div className="container max-w-7xl mx-auto py-8 px-4">
      {/* Header */}
      <div className="mb-8">
        <Button
          variant="ghost"
          onClick={() => navigate(`/projects/${project?.id}/roadmap`)}
          className="mb-4 -ml-2"
        >
          <ArrowLeft className="w-4 h-4 mr-2" />
          Back to Roadmap
        </Button>

        <div className="flex items-start justify-between gap-6">
          <div className="flex-1">
            <h1 className="text-3xl font-bold">{roadmapItem.title}</h1>
            <p className="text-muted-foreground mt-1">
              {project?.name} • {roadmap?.type} Roadmap
            </p>
            <div className="flex items-center gap-2 mt-3">
              <Badge variant={roadmapItem.status === 'Completed' ? 'default' : 'secondary'}>
                {roadmapItem.status}
              </Badge>
              <Badge variant="outline">{roadmapItem.priority} Priority</Badge>
            </div>
          </div>

          {hub && (
            <Card className="w-72 shrink-0">
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium">Knowledge Hub Stats</CardTitle>
              </CardHeader>
              <CardContent className="space-y-1.5 text-sm">
                <div className="grid grid-cols-2 gap-x-4 gap-y-1">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Documents</span>
                    <span className="font-medium">{hub.stats.totalDocuments}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Meetings</span>
                    <span className="font-medium">{hub.stats.totalMeetings}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Emails</span>
                    <span className="font-medium">{hub.stats.totalEmails}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Chats</span>
                    <span className="font-medium">{hub.stats.totalChats}</span>
                  </div>
                </div>
                <div className="border-t pt-1.5 grid grid-cols-2 gap-x-4 gap-y-1">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">AI Reqs</span>
                    <span className="font-medium">{hub.stats.totalRequirements}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Gaps</span>
                    <span className="font-medium">{gapsCount}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Confusions</span>
                    <span className="font-medium">{confusionsCount}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Threats</span>
                    <span className="font-medium">{threatsCount}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Formal Reqs</span>
                    <span className="font-medium text-primary">{formalReqCount}</span>
                  </div>
                  {conflictsCount > 0 && (
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Conflicts</span>
                      <span className={`font-medium ${openConflictsCount > 0 ? 'text-red-600' : 'text-muted-foreground'}`}>{conflictsCount}</span>
                    </div>
                  )}
                </div>
                <div className="border-t pt-1.5 flex justify-between items-center">
                  <span className="text-xs text-muted-foreground flex items-center gap-1">
                    <Zap className="w-3 h-3" /> Coverage
                  </span>
                  <span className={`text-sm font-bold ${coverageScore === 100 ? 'text-green-600' : coverageScore >= 70 ? 'text-yellow-600' : 'text-red-600'}`}>
                    {coverageScore}%
                  </span>
                </div>
                {totalInsights > 0 && (
                  <div className="border-t pt-1.5 flex justify-between font-medium">
                    <span className="text-muted-foreground">Total Insights</span>
                    <span className="text-primary">{totalInsights}</span>
                  </div>
                )}
              </CardContent>
            </Card>
          )}
        </div>
      </div>

      {/* AI Search Box */}
      <Card className="mb-6 border-2 border-primary/20">
        <CardContent className="pt-6 pb-5">
          <div className="flex items-center gap-3">
            <Search className="w-5 h-5 text-muted-foreground shrink-0" />
            {/* Input + suggestions dropdown */}
            <div className="relative flex-1">
              <Input
                ref={searchInputRef}
                placeholder="Ask anything or type / for commands — e.g. /summary, /gaps, /export"
                value={searchQuery}
                onChange={(e) => {
                  setSearchQuery(e.target.value);
                  setShowSuggestions(true);
                  setHighlightedIndex(-1);
                  setShowSearchResult(false);
                }}
                onFocus={() => setShowSuggestions(true)}
                onBlur={() => setTimeout(() => setShowSuggestions(false), 150)}
                onKeyDown={(e) => {
                  const list = isCommandMode ? filteredCommands : filteredSuggestions;
                  if (e.key === 'ArrowDown') {
                    e.preventDefault();
                    setHighlightedIndex(i => Math.min(i + 1, list.length - 1));
                  } else if (e.key === 'ArrowUp') {
                    e.preventDefault();
                    setHighlightedIndex(i => Math.max(i - 1, -1));
                  } else if (e.key === 'Enter') {
                    if (highlightedIndex >= 0) {
                      if (isCommandMode) handleCommand(filteredCommands[highlightedIndex].command);
                      else handleSearch(filteredSuggestions[highlightedIndex]);
                    } else if (isCommandMode && filteredCommands.length === 1) {
                      handleCommand(filteredCommands[0].command);
                    } else {
                      handleSearch();
                    }
                  } else if (e.key === 'Escape') {
                    setShowSuggestions(false);
                  }
                }}
                className="w-full text-base"
              />
              {/* Dropdown — command palette OR suggestions */}
              {showSuggestions && !isSearching && (isCommandMode ? filteredCommands.length > 0 : filteredSuggestions.length > 0) && (
                <div className="absolute top-full left-0 right-0 z-50 mt-1 bg-background border border-border rounded-xl shadow-xl overflow-hidden max-h-96 overflow-y-auto">

                  {/* Command palette */}
                  {isCommandMode && (
                    <>
                      <div className="px-3 py-2 text-xs font-semibold text-muted-foreground bg-muted/60 border-b flex items-center gap-2">
                        <span className="text-primary font-mono">/</span>
                        Commands — use ↑↓ to navigate, Enter to run
                      </div>
                      {(Object.keys(groupedCommands) as CommandCategory[]).map(cat => (
                        <div key={cat}>
                          <div className="px-3 py-1 text-xs font-medium text-muted-foreground bg-muted/30 uppercase tracking-wide">
                            {CATEGORY_LABELS[cat]}
                          </div>
                          {groupedCommands[cat]!.map((cmd, i) => {
                            const globalIdx = filteredCommands.indexOf(cmd);
                            return (
                              <button
                                key={cmd.command}
                                className={`w-full text-left px-3 py-2.5 flex items-center gap-3 transition-colors border-b border-border/40 last:border-0 ${
                                  globalIdx === highlightedIndex ? 'bg-primary/10' : 'hover:bg-muted/50'
                                }`}
                                onMouseDown={() => handleCommand(cmd.command)}
                                onMouseEnter={() => setHighlightedIndex(globalIdx)}
                              >
                                <span className="text-base w-5 text-center shrink-0">{cmd.icon}</span>
                                <div className="flex-1 min-w-0">
                                  <span className="font-mono text-sm font-medium text-primary">{cmd.command}</span>
                                  <span className="text-xs text-muted-foreground ml-2">{cmd.description}</span>
                                </div>
                                {globalIdx === highlightedIndex && (
                                  <span className="text-xs text-muted-foreground shrink-0">↵</span>
                                )}
                              </button>
                            );
                          })}
                        </div>
                      ))}
                    </>
                  )}

                  {/* Suggestions */}
                  {!isCommandMode && (
                    <>
                      <div className="px-3 py-1.5 text-xs font-medium text-muted-foreground bg-muted/50 border-b">
                        Suggested questions · Type <span className="font-mono text-primary">/</span> for commands
                      </div>
                      {filteredSuggestions.map((suggestion, i) => (
                        <button
                          key={i}
                          className={`w-full text-left px-3 py-2 text-sm flex items-center gap-2 transition-colors border-b border-border/30 last:border-0 ${
                            i === highlightedIndex ? 'bg-primary/10 text-primary' : 'hover:bg-muted/50'
                          }`}
                          onMouseDown={() => handleSearch(suggestion)}
                          onMouseEnter={() => setHighlightedIndex(i)}
                        >
                          <Search className="w-3 h-3 text-muted-foreground shrink-0" />
                          {suggestion}
                        </button>
                      ))}
                    </>
                  )}
                </div>
              )}
            </div>
            <Button disabled={!searchQuery.trim() || isSearching} onClick={() => handleSearch()}>
              <Brain className="w-4 h-4 mr-2" />
              {isSearching ? 'Searching...' : 'Ask AI'}
            </Button>
            {showSearchResult && (
              <Button variant="ghost" size="sm" onClick={() => { setShowSearchResult(false); setSearchQuery(''); }}>
                Clear
              </Button>
            )}
          </div>
          <p className="text-xs text-muted-foreground mt-2 ml-8">
            Type <kbd className="font-mono text-primary bg-primary/10 px-1 rounded text-xs">/</kbd> for commands (navigate, AI, export) · Or ask a question in plain English
          </p>

          {/* Search Result */}
          {showSearchResult && lastSearchResult && (
            <div className="mt-4 border-t pt-4 space-y-3">
              <div className="flex items-start gap-2">
                <Brain className="w-4 h-4 text-primary shrink-0 mt-0.5" />
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-2">
                    <span className="text-sm font-medium">AI Answer</span>
                    <span className="text-xs px-2 py-0.5 rounded-full bg-primary/10 text-primary">
                      {lastSearchResult.confidence}% confidence
                    </span>
                  </div>
                  <p className="text-sm text-foreground whitespace-pre-wrap leading-relaxed">
                    {lastSearchResult.answer}
                  </p>
                </div>
              </div>

              {lastSearchResult.sources.length > 0 && (
                <div className="ml-6">
                  <p className="text-xs font-medium text-muted-foreground mb-2">Sources used:</p>
                  <div className="flex flex-wrap gap-2">
                    {lastSearchResult.sources.map((src, i) => (
                      <div key={i} className="flex items-center gap-1.5 text-xs bg-muted rounded-lg px-2 py-1">
                        {src.type === 'document'    && <span>📄</span>}
                        {src.type === 'meeting'     && <span>🎥</span>}
                        {src.type === 'email'       && <span>📧</span>}
                        {src.type === 'chat'        && <span>💬</span>}
                        {src.type === 'requirement' && <span>📋</span>}
                        {src.type === 'gap'         && <span>⚠️</span>}
                        <span className="font-medium">{src.title}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Loading state */}
          {showSearchResult && isSearching && (
            <div className="mt-4 border-t pt-4 flex items-center gap-2 text-sm text-muted-foreground">
              <Brain className="w-4 h-4 text-primary animate-pulse" />
              Searching across all sources...
            </div>
          )}
        </CardContent>
      </Card>

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        {/* Input Sources Row */}
        <div className="mb-1">
          <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-1 px-1">Input Sources</p>
          <TabsList className="grid w-full grid-cols-5">
            <TabsTrigger value="documents" className="flex items-center gap-1.5 text-xs">
              <FileText className="w-3.5 h-3.5" />
              Docs ({hub?.stats.totalDocuments ?? 0})
            </TabsTrigger>
            <TabsTrigger value="meetings" className="flex items-center gap-1.5 text-xs">
              <Video className="w-3.5 h-3.5" />
              Meetings ({hub?.stats.totalMeetings ?? 0})
            </TabsTrigger>
            <TabsTrigger value="emails" className="flex items-center gap-1.5 text-xs">
              <Mail className="w-3.5 h-3.5" />
              Emails ({hub?.stats.totalEmails ?? 0})
            </TabsTrigger>
            <TabsTrigger value="chats" className="flex items-center gap-1.5 text-xs">
              <MessageSquare className="w-3.5 h-3.5" />
              Chats ({hub?.stats.totalChats ?? 0})
            </TabsTrigger>
            <TabsTrigger value="ai-requirements" className="flex items-center gap-1.5 text-xs">
              <Brain className="w-3.5 h-3.5" />
              AI Reqs ({hub?.stats.totalRequirements ?? 0})
            </TabsTrigger>
          </TabsList>
        </div>

        {/* Formal Requirements + Deliverables Row */}
        <div className="mb-1">
          <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-1 px-1">Requirements & Deliverables</p>
          <TabsList className="w-full grid grid-cols-2">
            <TabsTrigger value="requirements" className="flex items-center gap-1.5 text-xs">
              <ClipboardList className="w-3.5 h-3.5" />
              Formal Requirements ({formalReqCount})
            </TabsTrigger>
            <TabsTrigger value="deliverables" className="flex items-center gap-1.5 text-xs">
              <Download className="w-3.5 h-3.5" />
              Generate Deliverables
            </TabsTrigger>
          </TabsList>
        </div>

        {/* AI Insights Row */}
        <div className="mb-4">
          <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-1 px-1">AI Insights</p>
          <TabsList className="flex w-full flex-wrap gap-1">
            {showInsight('gaps') && (
              <TabsTrigger value="gaps" className="flex items-center gap-1 text-xs">
                <AlertTriangle className="w-3 h-3" />
                Gaps ({gapsCount})
              </TabsTrigger>
            )}
            {showInsight('confusions') && (
              <TabsTrigger value="confusions" className="flex items-center gap-1 text-xs">
                <HelpCircle className="w-3 h-3" />
                Confusions ({confusionsCount})
              </TabsTrigger>
            )}
            {showInsight('suggestions') && (
              <TabsTrigger value="suggestions" className="flex items-center gap-1 text-xs">
                <Lightbulb className="w-3 h-3" />
                Suggestions ({suggestionsCount})
              </TabsTrigger>
            )}
            {showInsight('security') && (
              <TabsTrigger value="security" className="flex items-center gap-1 text-xs">
                <Shield className="w-3 h-3" />
                Security ({securityCount})
              </TabsTrigger>
            )}
            {showInsight('threats') && (
              <TabsTrigger value="threats" className="flex items-center gap-1 text-xs">
                <Swords className="w-3 h-3" />
                Threats ({threatsCount})
              </TabsTrigger>
            )}
            {showInsight('conflicts') && conflictsCount > 0 && (
              <TabsTrigger value="conflicts" className="flex items-center gap-1 text-xs relative">
                <Swords className="w-3 h-3 text-red-500" />
                Conflicts ({conflictsCount})
                {openConflictsCount > 0 && <span className="absolute -top-1 -right-1 w-2 h-2 rounded-full bg-red-500" />}
              </TabsTrigger>
            )}
            {showInsight('nfrs') && (
              <TabsTrigger value="nfrs" className="flex items-center gap-1 text-xs">
                <Gauge className="w-3 h-3" />
                NFRs ({nfrsCount})
              </TabsTrigger>
            )}
            {showInsight('cloud') && (
              <TabsTrigger value="cloud" className="flex items-center gap-1 text-xs">
                <Cloud className="w-3 h-3" />
                Cloud ({cloudCount})
              </TabsTrigger>
            )}
          </TabsList>
        </div>

        {/* Deliverables tab */}
        <TabsContent value="deliverables" className="mt-0">
          <DeliverablesTab
            roadmapItemTitle={roadmapItem?.title ?? ''}
            roadmapType={roadmap?.type ?? ''}
            projectName={project?.name ?? ''}
            hub={hub}
          />
        </TabsContent>

        {/* Formal Requirements tab */}
        <TabsContent value="requirements" className="mt-0">
          <RequirementsTab
            roadmapItemId={roadmapItemId!}
            projectId={project?.id ?? ''}
            requirements={(hub?.formalRequirements ?? []) as any[]}
          />
        </TabsContent>

        {/* Input source tabs */}
        <TabsContent value="documents" className="mt-0">
          <DocumentsTab roadmapItemId={roadmapItemId!} documents={hub?.documents ?? []} />
        </TabsContent>
        <TabsContent value="meetings" className="mt-0">
          <MeetingsTab roadmapItemId={roadmapItemId!} meetings={hub?.meetings ?? []} />
        </TabsContent>
        <TabsContent value="emails" className="mt-0">
          <EmailsTab roadmapItemId={roadmapItemId!} emails={hub?.emails ?? []} />
        </TabsContent>
        <TabsContent value="chats" className="mt-0">
          <ChatsTab roadmapItemId={roadmapItemId!} chats={hub?.chats ?? []} />
        </TabsContent>
        <TabsContent value="ai-requirements" className="mt-0">
          <AIRequirementsTab roadmapItemId={roadmapItemId!} projectId={project?.id ?? ''} requirements={hub?.aiRequirements ?? []} />
        </TabsContent>

        {/* AI Insight tabs */}
        <TabsContent value="gaps" className="mt-0">
          <GapsTab
            roadmapItemId={roadmapItemId!}
            gaps={(hub?.gaps ?? []) as any[]}
            onRefresh={refresh}
            aiRequirements={(hub?.aiRequirements ?? []) as any[]}
          />
        </TabsContent>
        <TabsContent value="confusions" className="mt-0">
          <ConfusionsTab
            roadmapItemId={roadmapItemId!}
            confusions={(hub?.confusions ?? []) as any[]}
            onRefresh={refresh}
            aiRequirements={(hub?.aiRequirements ?? []) as any[]}
          />
        </TabsContent>
        <TabsContent value="suggestions" className="mt-0">
          <SuggestionsTab
            roadmapItemId={roadmapItemId!}
            suggestions={(hub?.suggestions ?? []) as any[]}
            onRefresh={refresh}
            aiRequirements={(hub?.aiRequirements ?? []) as any[]}
          />
        </TabsContent>
        <TabsContent value="security" className="mt-0">
          <SecurityRequirementsTab
            roadmapItemId={roadmapItemId!}
            requirements={(hub?.securityRequirements ?? []) as any[]}
            onRefresh={refresh}
          />
        </TabsContent>
        <TabsContent value="threats" className="mt-0">
          <ThreatsTab
            roadmapItemId={roadmapItemId!}
            threats={(hub?.threats ?? []) as any[]}
            onRefresh={refresh}
          />
        </TabsContent>
        <TabsContent value="nfrs" className="mt-0">
          <NFRsTab
            roadmapItemId={roadmapItemId!}
            nfrs={(hub?.nfrs ?? []) as any[]}
            onRefresh={refresh}
          />
        </TabsContent>
        <TabsContent value="cloud" className="mt-0">
          <CloudGovernanceTab
            roadmapItemId={roadmapItemId!}
            items={(hub?.cloudGovernance ?? []) as any[]}
            onRefresh={refresh}
          />
        </TabsContent>
        <TabsContent value="conflicts" className="mt-0">
          <ConflictsTab
            roadmapItemId={roadmapItemId!}
            conflicts={(hub?.conflicts ?? []) as any[]}
            onRefresh={refresh}
            onNavigateToSource={navigateToSource}
          />
        </TabsContent>
      </Tabs>
    </div>
  );
}
