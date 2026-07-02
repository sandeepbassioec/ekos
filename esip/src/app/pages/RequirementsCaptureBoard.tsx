import React, { useState, useEffect, useMemo, useCallback } from 'react';
import { useNavigate, useParams } from 'react-router';
import {
  ArrowLeft, Plus, Filter, Search, FileText, Bug, Zap, Code, Download,
  Upload, Sparkles, Target, Layers, CheckCircle2, ChevronDown,
} from 'lucide-react';
import { StoryDecompositionPanel } from '../components/requirements/StoryDecompositionPanel';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../components/ui/card';
import { Input } from '../components/ui/input';
import { Badge } from '../components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../components/ui/tabs';
import type { Requirement, RequirementType, RequirementStatus } from '../types/requirements';
import type { PI } from '../lib/db/types';
import { BRDImportModal } from '../components/BRDImportModal';
import { toast } from 'sonner';
import { usePersonaStore, getPersona } from '../store/personaStore';
import { useRequirementsStore } from '../store/requirementsStore';
import { useAuthStore } from '../store/authStore';
import { useTeamStore } from '../store/teamStore';
import { usePIStore } from '../store/piStore';
import { dalToBoard, isBoardReq, boardMetadata, BOARD_TYPE } from '../lib/boardRequirementMapper';

export default function RequirementsCaptureBoard() {
  const navigate = useNavigate();
  const { projectId } = useParams<{ projectId: string }>();
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedType, setSelectedType] = useState<RequirementType | 'all'>('all');
  const [showImportModal, setShowImportModal] = useState(false);
  const [selectedPI, setSelectedPI] = useState<string>('all');
  const [selectedSprint, setSelectedSprint] = useState<string>('all');
  const [showSprintContext, setShowSprintContext] = useState(false);

  const { requirements: dalReqs, fetchRequirements, updateRequirement, bulkCreateRequirements, isLoading } = useRequirementsStore();
  const { user } = useAuthStore();
  const { hasRole, fetchTeam } = useTeamStore();
  const { pis, fetchPIs } = usePIStore();

  useEffect(() => {
    if (!projectId) return;
    fetchRequirements(projectId);
    fetchTeam(projectId);
    fetchPIs(projectId);
  }, [projectId]);

  const orgRole = user?.role ?? '';
  const canManage = (user && hasRole(user.id, 'product_owner', 'delivery_manager'))
    || orgRole === 'admin' || orgRole === 'manager';

  const boardRequirements = useMemo(
    () => dalReqs.filter(isBoardReq).map(dalToBoard),
    [dalReqs],
  );

  const activePersonaId = usePersonaStore((s) => s.activePersonaId);
  const persona = getPersona(activePersonaId);
  const personaReqTypes = persona.requirementTypes ?? [];
  const showReqType = (t: string) => personaReqTypes.length === 0 || personaReqTypes.includes(t);

  useEffect(() => {
    if (selectedType !== 'all' && !showReqType(selectedType)) setSelectedType('all');
  }, [activePersonaId]);

  const filteredRequirements = useMemo(() => {
    let filtered = boardRequirements;
    if (selectedPI !== 'all') filtered = filtered.filter(req => req.allocatedPiId === selectedPI || req.programIncrement === selectedPI);
    if (selectedSprint !== 'all') filtered = filtered.filter(req => req.sprint === selectedSprint);
    if (selectedType !== 'all') filtered = filtered.filter(req => req.type === selectedType);
    if (searchQuery) {
      const q = searchQuery.toLowerCase();
      filtered = filtered.filter(req => req.title.toLowerCase().includes(q) || req.description.toLowerCase().includes(q));
    }
    return filtered;
  }, [boardRequirements, selectedPI, selectedSprint, selectedType, searchQuery]);

  const requirementTypes: { value: RequirementType; label: string; icon: string }[] = [
    { value: 'new_screen',      label: 'New Screen',      icon: '🖥️' },
    { value: 'update_screen',   label: 'Update Screen',   icon: '✏️' },
    { value: 'bug_fix',         label: 'Bug Fix',         icon: '🐛' },
    { value: 'feature_change',  label: 'Feature Change',  icon: '⚡' },
    { value: 'api_development', label: 'API Dev',         icon: '🔌' },
    { value: 'integration',     label: 'Integration',     icon: '🔗' },
    { value: 'data_migration',  label: 'Data Migration',  icon: '📦' },
    { value: 'performance',     label: 'Performance',     icon: '⚡' },
    { value: 'security',        label: 'Security',        icon: '🔒' },
    { value: 'ui_ux_change',    label: 'UI/UX',           icon: '🎨' },
  ];

  const finalizedCount = filteredRequirements.filter(r => r.status === 'finalized').length;

  const stats = [
    { label: 'Total', value: filteredRequirements.length, icon: FileText, color: 'bg-blue-500' },
    { label: 'Finalized', value: finalizedCount, icon: CheckCircle2, color: 'bg-indigo-500' },
    { label: 'In Progress', value: filteredRequirements.filter(r => r.status === 'in_progress').length, icon: Zap, color: 'bg-yellow-500' },
    { label: 'Critical', value: filteredRequirements.filter(r => r.priority === 'critical').length, icon: Bug, color: 'bg-red-500' },
  ];

  // ── Handlers ─────────────────────────────────────────────────────────────────

  const handleFinalize = useCallback(async (reqId: string) => {
    const dalReq = dalReqs.find(r => r.id === reqId);
    if (!dalReq || !user) return;
    await updateRequirement(reqId, {
      status: 'finalized',
      metadata: {
        ...dalReq.metadata,
        finalizedBy: user.id,
        finalizedAt: new Date().toISOString(),
      },
    });
    toast.success('Requirement finalized!');
  }, [dalReqs, updateRequirement, user]);

  const handleAllocatePI = useCallback(async (reqId: string, pi: PI) => {
    const dalReq = dalReqs.find(r => r.id === reqId);
    if (!dalReq) return;
    await updateRequirement(reqId, {
      piId: pi.id,
      metadata: {
        ...dalReq.metadata,
        programIncrement: pi.name,
        allocatedPiName: pi.name,
      },
    });
    toast.success(`Allocated to ${pi.name}`);
  }, [dalReqs, updateRequirement]);

  const handleUnallocate = useCallback(async (reqId: string) => {
    const dalReq = dalReqs.find(r => r.id === reqId);
    if (!dalReq) return;
    await updateRequirement(reqId, {
      piId: undefined,
      metadata: {
        ...dalReq.metadata,
        programIncrement: undefined,
        allocatedPiName: undefined,
      },
    });
    toast.success('Removed from PI');
  }, [dalReqs, updateRequirement]);

  const handleMove = useCallback(async (reqId: string, newPI: string, newSprint: string) => {
    const dalReq = dalReqs.find(r => r.id === reqId);
    if (!dalReq) return;
    await updateRequirement(reqId, {
      metadata: { ...dalReq.metadata, programIncrement: newPI || undefined, sprint: newSprint || undefined },
    });
    toast.success('Requirement moved successfully!');
  }, [dalReqs, updateRequirement]);

  useEffect(() => {
    setShowSprintContext(selectedPI !== 'all' || selectedSprint !== 'all');
  }, [selectedPI, selectedSprint]);

  // ── Shared card renderer ──────────────────────────────────────────────────────

  const renderCards = (reqs: Requirement[]) => (
    <div className="space-y-4">
      {reqs.map(req => (
        <RequirementCard
          key={req.id}
          requirement={req}
          projectId={projectId}
          projectPIs={pis}
          canManage={canManage}
          onFinalize={handleFinalize}
          onAllocatePI={handleAllocatePI}
          onUnallocate={handleUnallocate}
          onMove={handleMove}
        />
      ))}
    </div>
  );

  return (
    <div className="container max-w-7xl mx-auto py-8 px-4">
      {/* Header */}
      <div className="mb-8">
        {projectId && (
          <Button variant="ghost" onClick={() => navigate(`/projects/${projectId}`)} className="mb-4 -ml-2">
            <ArrowLeft className="w-4 h-4 mr-2" />
            Back to Project
          </Button>
        )}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">Requirements Board</h1>
            <p className="text-muted-foreground mt-1">Project-wide requirements across all modules</p>
          </div>
          <div className="flex gap-3">
            <Button onClick={() => setShowImportModal(true)} className="bg-purple-600 hover:bg-purple-700">
              <Upload className="w-4 h-4 mr-2" />Import from BRD
            </Button>
            {projectId && (
              <Button
                onClick={() => navigate(`/projects/${projectId}/requirements/comprehensive`)}
                className="bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-700 hover:to-indigo-700"
              >
                <Sparkles className="w-4 h-4 mr-2" />Comprehensive View
              </Button>
            )}
          </div>
        </div>
      </div>

      {/* PI & Sprint Filters — now uses live PIs */}
      <Card className="mb-6 border-2 border-blue-200">
        <CardHeader>
          <div className="flex items-center gap-4">
            <Layers className="w-6 h-6 text-blue-600" />
            <div>
              <CardTitle>Filter by Program Increment & Sprint</CardTitle>
              <CardDescription>View requirements for specific PI/Sprint or see all</CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="text-sm font-medium mb-2 block">Program Increment</label>
              <select
                value={selectedPI}
                onChange={e => setSelectedPI(e.target.value)}
                className="w-full border border-gray-300 rounded-md px-3 py-2 bg-white"
              >
                <option value="all">All Program Increments</option>
                {pis.map(pi => (
                  <option key={pi.id} value={pi.id}>{pi.name}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="text-sm font-medium mb-2 block">Sprint</label>
              <select
                value={selectedSprint}
                onChange={e => setSelectedSprint(e.target.value)}
                className="w-full border border-gray-300 rounded-md px-3 py-2 bg-white"
              >
                <option value="all">All Sprints</option>
                {['Sprint 1','Sprint 2','Sprint 3','Sprint 4','Sprint 5','Sprint 6'].map(s => (
                  <option key={s} value={s}>{s}</option>
                ))}
              </select>
            </div>
          </div>
        </CardContent>
      </Card>

      {showSprintContext && (
        <Card className="mb-6 border-2 border-purple-200 bg-gradient-to-r from-purple-50 to-blue-50">
          <CardHeader>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-4">
                <Target className="w-6 h-6 text-purple-600" />
                <div>
                  <CardTitle>
                    {selectedPI !== 'all' && selectedSprint !== 'all'
                      ? `${pis.find(p => p.id === selectedPI)?.name ?? selectedPI} — ${selectedSprint}`
                      : selectedPI !== 'all'
                        ? (pis.find(p => p.id === selectedPI)?.name ?? selectedPI)
                        : selectedSprint}
                  </CardTitle>
                  <CardDescription>Filtered view · {filteredRequirements.length} requirements</CardDescription>
                </div>
              </div>
              <Button variant="outline" size="sm">
                <Download className="w-4 h-4 mr-2" />Export Backlog
              </Button>
            </div>
          </CardHeader>
        </Card>
      )}

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
        {stats.map((stat, idx) => (
          <Card key={idx}>
            <CardContent className="pt-6">
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-2xl font-bold">{stat.value}</div>
                  <div className="text-sm text-muted-foreground">{stat.label}</div>
                </div>
                <div className={`w-12 h-12 rounded-lg ${stat.color} flex items-center justify-center`}>
                  <stat.icon className="w-6 h-6 text-white" />
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Search & Add */}
      <Card className="mb-6">
        <CardContent className="pt-6">
          <div className="flex gap-4">
            <div className="flex-1">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-muted-foreground" />
                <Input
                  placeholder="Search requirements..."
                  value={searchQuery}
                  onChange={e => setSearchQuery(e.target.value)}
                  className="pl-10"
                />
              </div>
            </div>
            <Button variant="outline"><Filter className="w-4 h-4 mr-2" />More Filters</Button>
            <Button
              onClick={() => projectId && navigate(`/projects/${projectId}/requirements/add`)}
              className="bg-blue-600 hover:bg-blue-700"
            >
              <Plus className="w-4 h-4 mr-2" />Add Requirement
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Type quick-filter */}
      <div className="flex gap-2 mb-6 flex-wrap">
        {personaReqTypes.length > 0 && (
          <div className="basis-full text-xs text-muted-foreground">
            Types tailored for <span className="font-medium">{persona.label}</span>
          </div>
        )}
        <Button
          variant={selectedType === 'all' ? 'default' : 'outline'}
          size="sm"
          onClick={() => setSelectedType('all')}
        >
          All ({boardRequirements.length})
        </Button>
        {requirementTypes.filter(t => showReqType(t.value)).map(type => (
          <Button
            key={type.value}
            variant={selectedType === type.value ? 'default' : 'outline'}
            size="sm"
            onClick={() => setSelectedType(type.value)}
          >
            <span className="mr-1">{type.icon}</span>
            {type.label} ({boardRequirements.filter(r => r.type === type.value).length})
          </Button>
        ))}
      </div>

      {/* Requirements tabs — now includes Finalized */}
      {isLoading ? (
        <div className="text-center py-16 text-muted-foreground">Loading requirements…</div>
      ) : (
        <Tabs defaultValue="all" className="w-full">
          <TabsList className="flex-wrap h-auto gap-1">
            <TabsTrigger value="all">All ({filteredRequirements.length})</TabsTrigger>
            <TabsTrigger value="draft">Draft</TabsTrigger>
            <TabsTrigger value="review">In Review</TabsTrigger>
            <TabsTrigger value="approved">Approved</TabsTrigger>
            <TabsTrigger value="finalized" className="text-indigo-600 data-[state=active]:bg-indigo-600 data-[state=active]:text-white">
              ✓ Finalized ({finalizedCount})
            </TabsTrigger>
            <TabsTrigger value="in_progress">In Progress</TabsTrigger>
            <TabsTrigger value="completed">Completed</TabsTrigger>
          </TabsList>

          <TabsContent value="all" className="mt-6">
            {filteredRequirements.length === 0 ? (
              <Card>
                <CardContent className="py-16 text-center">
                  <FileText className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                  <h3 className="text-xl font-semibold mb-2">
                    {boardRequirements.length === 0 ? 'No Requirements Yet' : 'No Requirements Match Filters'}
                  </h3>
                  <p className="text-muted-foreground mb-6">
                    {boardRequirements.length === 0
                      ? 'Start by adding your first requirement'
                      : 'Try adjusting your filters or search criteria'}
                  </p>
                  {boardRequirements.length === 0 && (
                    <Button
                      onClick={() => projectId && navigate(`/projects/${projectId}/requirements/add`)}
                      className="bg-blue-600 hover:bg-blue-700"
                    >
                      <Plus className="w-4 h-4 mr-2" />Add First Requirement
                    </Button>
                  )}
                </CardContent>
              </Card>
            ) : renderCards(filteredRequirements)}
          </TabsContent>

          {(['draft', 'review', 'approved', 'finalized', 'in_progress', 'completed'] as RequirementStatus[]).map(status => (
            <TabsContent key={status} value={status} className="mt-6">
              {status === 'finalized' && filteredRequirements.filter(r => r.status === 'finalized').length === 0 ? (
                <Card>
                  <CardContent className="py-12 text-center">
                    <CheckCircle2 className="w-12 h-12 mx-auto text-indigo-300 mb-3" />
                    <h3 className="font-semibold text-gray-700 mb-1">No finalized requirements yet</h3>
                    <p className="text-sm text-muted-foreground">
                      {canManage
                        ? 'Use the "Finalize" button on any requirement card to lock it for PI allocation.'
                        : 'The Product Owner finalises requirements before they can be allocated to a PI.'}
                    </p>
                  </CardContent>
                </Card>
              ) : (
                renderCards(filteredRequirements.filter(r => r.status === status))
              )}
            </TabsContent>
          ))}
        </Tabs>
      )}

      {/* Quick Start Templates */}
      <Card className="mt-8">
        <CardHeader>
          <CardTitle>Quick Start Templates</CardTitle>
          <CardDescription>Use these templates for common requirement types</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {[
              { template: 'new_screen',     icon: '🖥️', label: 'New Screen',     desc: 'Create a brand new screen/page' },
              { template: 'bug_fix',        icon: '🐛', label: 'Bug Fix',         desc: 'Report and track a bug' },
              { template: 'feature_change', icon: '⚡', label: 'Feature Change',  desc: 'Modify existing functionality' },
            ].map(({ template, icon, label, desc }) => (
              <Card
                key={template}
                className="cursor-pointer hover:shadow-md transition-shadow"
                onClick={() => projectId && navigate(`/projects/${projectId}/requirements/add?template=${template}`)}
              >
                <CardContent className="pt-6">
                  <div className="text-3xl mb-2">{icon}</div>
                  <div className="font-semibold">{label}</div>
                  <div className="text-sm text-muted-foreground">{desc}</div>
                </CardContent>
              </Card>
            ))}
          </div>
        </CardContent>
      </Card>

      <BRDImportModal
        open={showImportModal}
        onClose={() => setShowImportModal(false)}
        onImportComplete={extractedReqs => {
          if (!projectId) return;
          const payloads = extractedReqs.map(req => ({
            title: req.title,
            description: req.description,
            category: req.type,
            priority: req.priority,
            status: 'draft',
            tags: ['board'],
            createdBy: user?.id ?? 'unknown',
            metadata: boardMetadata({
              type: req.type,
              programIncrement: selectedPI !== 'all' ? selectedPI : undefined,
              sprint: selectedSprint !== 'all' ? selectedSprint : undefined,
              moduleIds: [],
              businessValue: '',
              userStory: req.userStory,
              affectedScreens: req.affectedScreens,
              affectedAPIs: req.affectedAPIs,
              affectedComponents: [],
              technicalNotes: req.technicalNotes,
              acceptanceCriteria: req.acceptanceCriteria.map((ac, i) => ({ id: `ac-${i}`, description: ac, completed: false })),
              dependencies: [],
              blockers: [],
              complexity: req.complexity,
              estimatedHours: req.estimatedHours,
              attachments: [],
              comments: [],
            }),
          }));
          bulkCreateRequirements(projectId, payloads).then(() => {
            toast.success(`Imported ${payloads.length} requirements from BRD!`);
          });
          setShowImportModal(false);
        }}
        projectId={projectId || 'unknown'}
        moduleId="not-applicable"
      />
    </div>
  );
}

// ── Requirement Card ──────────────────────────────────────────────────────────

interface RequirementCardProps {
  requirement: Requirement;
  projectId?: string;
  projectPIs: PI[];
  canManage: boolean;
  onFinalize: (id: string) => void;
  onAllocatePI: (id: string, pi: PI) => void;
  onUnallocate: (id: string) => void;
  onMove: (id: string, pi: string, sprint: string) => void;
}

function RequirementCard({
  requirement, projectId, projectPIs, canManage,
  onFinalize, onAllocatePI, onUnallocate,
}: RequirementCardProps) {
  const navigate = useNavigate();
  const [showPIPicker, setShowPIPicker] = useState(false);
  const [showDecompose, setShowDecompose] = useState(false);

  const typeConfig: Record<string, { icon: string }> = {
    new_screen: { icon: '🖥️' }, update_screen: { icon: '✏️' }, bug_fix: { icon: '🐛' },
    feature_change: { icon: '⚡' }, api_development: { icon: '🔌' }, integration: { icon: '🔗' },
    data_migration: { icon: '📦' }, performance: { icon: '⚡' }, security: { icon: '🔒' },
    ui_ux_change: { icon: '🎨' },
  };
  const icon = typeConfig[requirement.type]?.icon ?? '📋';

  const priorityColor: Record<string, string> = {
    critical: 'bg-red-100 text-red-700',
    high: 'bg-orange-100 text-orange-700',
    medium: 'bg-yellow-100 text-yellow-700',
    low: 'bg-green-100 text-green-700',
  };

  const isFinalized = requirement.status === 'finalized';
  const isAllocated = !!requirement.allocatedPiId;

  return (
    <Card className={`hover:shadow-md transition-shadow relative ${isFinalized ? 'border-indigo-200 bg-indigo-50/30' : ''}`}>
      <CardContent className="pt-5 pb-4">
        <div className="flex items-start gap-4">
          <div className="text-3xl shrink-0">{icon}</div>

          {/* Main content */}
          <div
            className="flex-1 min-w-0 cursor-pointer"
            onClick={() => navigate(`/projects/${projectId}/requirements/${requirement.id}/edit`)}
          >
            <div className="flex items-start justify-between gap-2 mb-1">
              <div className="flex items-center gap-2 flex-wrap min-w-0">
                <h3 className="font-semibold text-base truncate">{requirement.title}</h3>
                {isFinalized && (
                  <span className="inline-flex items-center gap-1 text-xs bg-indigo-100 text-indigo-700 px-2 py-0.5 rounded-full font-medium shrink-0">
                    <CheckCircle2 className="w-3 h-3" />Finalized
                  </span>
                )}
                {isAllocated && (
                  <span className="inline-flex items-center gap-1 text-xs bg-purple-100 text-purple-700 px-2 py-0.5 rounded-full font-medium shrink-0">
                    <Layers className="w-3 h-3" />{requirement.allocatedPiName}
                  </span>
                )}
                {!isAllocated && requirement.programIncrement && (
                  <Badge variant="outline" className="text-xs shrink-0">{requirement.programIncrement}</Badge>
                )}
                {requirement.sprint && (
                  <Badge variant="outline" className="text-xs shrink-0">{requirement.sprint}</Badge>
                )}
              </div>
              <div className="flex gap-2 shrink-0">
                <Badge className={priorityColor[requirement.priority] ?? 'bg-gray-100 text-gray-700'}>
                  {requirement.priority}
                </Badge>
                {!isFinalized && (
                  <Badge variant="secondary">{requirement.status}</Badge>
                )}
              </div>
            </div>

            <p className="text-sm text-muted-foreground line-clamp-2 mb-2">{requirement.description}</p>

            <div className="flex items-center gap-4 text-xs text-muted-foreground">
              <span>Complexity: {requirement.complexity}</span>
              {requirement.estimatedHours && <span>Est: {requirement.estimatedHours}h</span>}
              {requirement.acceptanceCriteria.length > 0 && (
                <span>AC: {requirement.acceptanceCriteria.filter(a => a.completed).length}/{requirement.acceptanceCriteria.length}</span>
              )}
            </div>
          </div>

          {/* Action buttons (stop propagation so they don't navigate) */}
          {canManage && (
            <div className="flex flex-col gap-2 shrink-0" onClick={e => e.stopPropagation()}>

              {/* Finalize button — only when not already finalized */}
              {!isFinalized && (
                <Button
                  size="sm"
                  variant="outline"
                  className="h-8 text-xs border-indigo-300 text-indigo-600 hover:bg-indigo-50 hover:border-indigo-500"
                  onClick={() => onFinalize(requirement.id)}
                >
                  <CheckCircle2 className="w-3.5 h-3.5 mr-1" />
                  Finalize
                </Button>
              )}

              {/* PI Allocation — only when finalized */}
              {isFinalized && (
                <div className="relative">
                  <Button
                    size="sm"
                    variant={isAllocated ? 'outline' : 'default'}
                    className={`h-8 text-xs ${isAllocated ? 'border-purple-300 text-purple-700' : 'bg-purple-600 hover:bg-purple-700'}`}
                    onClick={() => setShowPIPicker(v => !v)}
                    disabled={projectPIs.length === 0}
                    title={projectPIs.length === 0 ? 'No PIs exist yet — create one in PI Planning first' : undefined}
                  >
                    <Layers className="w-3.5 h-3.5 mr-1" />
                    {isAllocated ? 'Change PI' : 'Allocate to PI'}
                    <ChevronDown className="w-3 h-3 ml-1" />
                  </Button>

                  {showPIPicker && (
                    <div className="absolute right-0 top-full mt-1 w-64 bg-white border rounded-lg shadow-xl z-20 py-1">
                      <p className="text-xs text-muted-foreground px-3 py-1.5 border-b font-medium">Select Program Increment</p>
                      {projectPIs.map(pi => (
                        <button
                          key={pi.id}
                          className={`w-full text-left px-3 py-2 text-sm hover:bg-purple-50 transition-colors flex items-center justify-between ${requirement.allocatedPiId === pi.id ? 'bg-purple-50 text-purple-700 font-medium' : ''}`}
                          onClick={() => { onAllocatePI(requirement.id, pi); setShowPIPicker(false); }}
                        >
                          <span>{pi.name}</span>
                          {requirement.allocatedPiId === pi.id && <CheckCircle2 className="w-3.5 h-3.5 text-purple-500" />}
                        </button>
                      ))}
                      {isAllocated && (
                        <>
                          <div className="border-t my-1" />
                          <button
                            className="w-full text-left px-3 py-2 text-xs text-red-500 hover:bg-red-50 transition-colors"
                            onClick={() => { onUnallocate(requirement.id); setShowPIPicker(false); }}
                          >
                            Remove from PI
                          </button>
                        </>
                      )}
                    </div>
                  )}
                </div>
              )}

              {/* No PIs hint */}
              {isFinalized && projectPIs.length === 0 && (
                <p className="text-xs text-amber-600 max-w-[140px] leading-tight">
                  Create a PI first in PI Planning
                </p>
              )}

              {/* Decompose into Stories — only when finalized + PI allocated */}
              {isFinalized && isAllocated && (
                <Button
                  size="sm"
                  variant="outline"
                  className={`h-8 text-xs ${showDecompose ? 'border-indigo-500 bg-indigo-50 text-indigo-700' : 'border-indigo-200 text-indigo-600 hover:bg-indigo-50'}`}
                  onClick={() => setShowDecompose(v => !v)}
                >
                  <Sparkles className="w-3.5 h-3.5 mr-1" />
                  Stories
                </Button>
              )}
            </div>
          )}
        </div>

        {/* Story Decomposition Panel — inline below card content */}
        {showDecompose && projectId && (
          <StoryDecompositionPanel
            requirement={requirement}
            projectId={projectId}
            piId={requirement.allocatedPiId}
            onClose={() => setShowDecompose(false)}
          />
        )}
      </CardContent>
    </Card>
  );
}
