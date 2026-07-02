import React, { useEffect, useState, useCallback } from 'react';
import { useNavigate, useParams } from 'react-router';
import {
  ArrowLeft, Plus, ChevronDown, ChevronUp, Edit2, Trash2,
  Calendar, Target, Flag, CheckCircle2, Clock, AlertCircle,
  Layers, Save, X, LayoutGrid, List, GripVertical, Inbox,
  TrendingUp, Package,
} from 'lucide-react';
import { DndProvider, useDrag, useDrop } from 'react-dnd';
import { HTML5Backend } from 'react-dnd-html5-backend';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Textarea } from '../components/ui/textarea';
import { useAuthStore } from '../store/authStore';
import { useTeamStore } from '../store/teamStore';
import { useFeatureFlag } from '../store/featureFlagStore';
import { usePIStore } from '../store/piStore';
import { useRequirementsStore } from '../store/requirementsStore';
import type { PI, Sprint, Requirement } from '../lib/db/types';
import { db } from '../lib/db';

// ── Status helpers ────────────────────────────────────────────────────────────

const PI_STATUS_CONFIG: Record<string, { label: string; color: string; icon: React.ReactNode }> = {
  planning:  { label: 'Planning',   color: 'bg-blue-100 text-blue-700',   icon: <Clock className="w-3 h-3" /> },
  active:    { label: 'Active',     color: 'bg-green-100 text-green-700', icon: <CheckCircle2 className="w-3 h-3" /> },
  completed: { label: 'Completed',  color: 'bg-gray-100 text-gray-600',   icon: <Flag className="w-3 h-3" /> },
  cancelled: { label: 'Cancelled',  color: 'bg-red-100 text-red-600',     icon: <AlertCircle className="w-3 h-3" /> },
};

const SPRINT_STATUS_CONFIG: Record<string, { label: string; color: string }> = {
  planning:  { label: 'Planning',  color: 'bg-blue-100 text-blue-700' },
  active:    { label: 'Active',    color: 'bg-emerald-100 text-emerald-700' },
  completed: { label: 'Completed', color: 'bg-gray-100 text-gray-500' },
};

function formatDate(d?: string) {
  if (!d) return '—';
  return new Date(d).toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric' });
}

function suggestPIName(existingPIs: PI[]): string {
  const n = existingPIs.length + 1;
  const now = new Date();
  const quarter = Math.ceil((now.getMonth() + 1) / 3);
  return `PI ${n} — Q${quarter} ${now.getFullYear()}`;
}

// ── Sprint mini-card ──────────────────────────────────────────────────────────

function SprintCard({
  sprint,
  canManage,
  showBoard,
  onGoalSave,
  onStatusChange,
  onOpenBoard,
}: {
  sprint: Sprint;
  canManage: boolean;
  showBoard: boolean;
  onGoalSave: (id: string, goals: string) => void;
  onStatusChange: (id: string, status: string) => void;
  onOpenBoard: (sprintId: string) => void;
}) {
  const [editingGoal, setEditingGoal] = useState(false);
  const [goalDraft, setGoalDraft] = useState(sprint.goals ?? '');
  const cfg = SPRINT_STATUS_CONFIG[sprint.status] ?? SPRINT_STATUS_CONFIG.planning;

  const handleSave = () => {
    onGoalSave(sprint.id, goalDraft);
    setEditingGoal(false);
  };

  return (
    <div className="rounded-lg border bg-white p-3 flex flex-col gap-2 hover:shadow-sm transition-shadow">
      <div className="flex items-center justify-between">
        <span className="text-sm font-semibold text-gray-800">{sprint.name}</span>
        {canManage ? (
          <select
            value={sprint.status}
            onChange={e => onStatusChange(sprint.id, e.target.value)}
            className={`text-xs px-2 py-0.5 rounded-full border-0 font-medium cursor-pointer ${cfg.color}`}
          >
            <option value="planning">Planning</option>
            <option value="active">Active</option>
            <option value="completed">Completed</option>
          </select>
        ) : (
          <span className={`text-xs px-2 py-0.5 rounded-full font-medium ${cfg.color}`}>{cfg.label}</span>
        )}
      </div>

      <div className="flex items-center gap-1 text-xs text-muted-foreground">
        <Calendar className="w-3 h-3 shrink-0" />
        <span>{formatDate(sprint.startDate)} – {formatDate(sprint.endDate)}</span>
      </div>

      {editingGoal ? (
        <div className="flex flex-col gap-1">
          <Textarea
            value={goalDraft}
            onChange={e => setGoalDraft(e.target.value)}
            placeholder="Sprint goal…"
            rows={2}
            className="text-xs"
            autoFocus
          />
          <div className="flex gap-1">
            <Button size="sm" className="h-6 text-xs px-2" onClick={handleSave}>
              <Save className="w-3 h-3 mr-1" />Save
            </Button>
            <Button size="sm" variant="ghost" className="h-6 text-xs px-2" onClick={() => { setEditingGoal(false); setGoalDraft(sprint.goals ?? ''); }}>
              <X className="w-3 h-3" />
            </Button>
          </div>
        </div>
      ) : (
        <div
          className={`text-xs text-muted-foreground min-h-[1.5rem] leading-relaxed ${canManage ? 'cursor-pointer hover:text-gray-700 hover:bg-gray-50 rounded px-1 -mx-1' : ''}`}
          onClick={() => canManage && setEditingGoal(true)}
          title={canManage ? 'Click to set sprint goal' : undefined}
        >
          {sprint.goals || (canManage ? <span className="italic opacity-60">Click to add goal…</span> : <span className="opacity-40">No goal set</span>)}
        </div>
      )}
      {showBoard && (
        <button
          onClick={() => onOpenBoard(sprint.id)}
          className="mt-1 text-xs text-indigo-600 hover:text-indigo-800 hover:underline text-left"
        >
          Open Board →
        </button>
      )}
    </div>
  );
}

// ── PI card ───────────────────────────────────────────────────────────────────

function PICard({
  pi,
  sprints,
  canManage,
  showBoard,
  onUpdatePI,
  onDeletePI,
  onUpdateSprint,
  onOpenBoard,
}: {
  pi: PI;
  sprints: Sprint[];
  canManage: boolean;
  showBoard: boolean;
  onUpdatePI: (id: string, updates: Partial<PI>) => void;
  onDeletePI: (id: string) => void;
  onUpdateSprint: (id: string, updates: Partial<Sprint>) => void;
  onOpenBoard: (sprintId: string) => void;
}) {
  const [expanded, setExpanded] = useState(true);
  const [editingObjectives, setEditingObjectives] = useState(false);
  const [objectivesDraft, setObjectivesDraft] = useState(pi.objectives ?? '');
  const [confirmDelete, setConfirmDelete] = useState(false);

  const cfg = PI_STATUS_CONFIG[pi.status] ?? PI_STATUS_CONFIG.planning;
  const completedSprints = sprints.filter(s => s.status === 'completed').length;

  const handleObjectivesSave = () => {
    onUpdatePI(pi.id, { objectives: objectivesDraft });
    setEditingObjectives(false);
  };

  return (
    <Card className="overflow-hidden">
      {/* PI header */}
      <CardHeader className="pb-3 pt-4 px-5">
        <div className="flex items-start justify-between gap-3">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <CardTitle className="text-base font-bold text-gray-900">{pi.name}</CardTitle>
              <span className={`inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full font-medium ${cfg.color}`}>
                {cfg.icon}{cfg.label}
              </span>
              {canManage && (
                <select
                  value={pi.status}
                  onChange={e => onUpdatePI(pi.id, { status: e.target.value })}
                  className="text-xs border rounded px-1.5 py-0.5 text-muted-foreground bg-transparent"
                >
                  <option value="planning">Planning</option>
                  <option value="active">Active</option>
                  <option value="completed">Completed</option>
                  <option value="cancelled">Cancelled</option>
                </select>
              )}
            </div>
            <div className="flex items-center gap-3 mt-1 text-xs text-muted-foreground flex-wrap">
              <span className="flex items-center gap-1">
                <Calendar className="w-3 h-3" />
                {formatDate(pi.startDate)} – {formatDate(pi.endDate)}
              </span>
              <span className="flex items-center gap-1">
                <Layers className="w-3 h-3" />
                {sprints.length} sprints · {completedSprints} completed
              </span>
            </div>
          </div>

          <div className="flex items-center gap-1 shrink-0">
            {canManage && (
              <>
                {confirmDelete ? (
                  <div className="flex items-center gap-1">
                    <span className="text-xs text-red-600 mr-1">Delete PI?</span>
                    <Button size="sm" variant="destructive" className="h-7 text-xs px-2"
                      onClick={() => { setConfirmDelete(false); onDeletePI(pi.id); }}>
                      Yes
                    </Button>
                    <Button size="sm" variant="ghost" className="h-7 text-xs px-2"
                      onClick={() => setConfirmDelete(false)}>
                      No
                    </Button>
                  </div>
                ) : (
                  <Button size="sm" variant="ghost" className="h-7 w-7 p-0 text-red-400 hover:text-red-600 hover:bg-red-50"
                    onClick={() => setConfirmDelete(true)}
                    title="Delete PI">
                    <Trash2 className="w-3.5 h-3.5" />
                  </Button>
                )}
              </>
            )}
            <Button size="sm" variant="ghost" className="h-7 w-7 p-0"
              onClick={() => setExpanded(e => !e)}>
              {expanded ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
            </Button>
          </div>
        </div>

        {/* Objectives */}
        <div className="mt-2">
          {editingObjectives ? (
            <div className="flex flex-col gap-1">
              <Textarea
                value={objectivesDraft}
                onChange={e => setObjectivesDraft(e.target.value)}
                placeholder="PI objectives…"
                rows={2}
                className="text-sm"
                autoFocus
              />
              <div className="flex gap-1">
                <Button size="sm" className="h-7 text-xs px-2" onClick={handleObjectivesSave}>
                  <Save className="w-3 h-3 mr-1" />Save
                </Button>
                <Button size="sm" variant="ghost" className="h-7 text-xs px-2"
                  onClick={() => { setEditingObjectives(false); setObjectivesDraft(pi.objectives ?? ''); }}>
                  <X className="w-3 h-3" />
                </Button>
              </div>
            </div>
          ) : (
            <div
              className={`flex items-start gap-1.5 text-sm text-muted-foreground ${canManage ? 'cursor-pointer hover:text-gray-700 group' : ''}`}
              onClick={() => canManage && setEditingObjectives(true)}
            >
              <Target className="w-3.5 h-3.5 mt-0.5 shrink-0 text-purple-400" />
              <span className="leading-relaxed">
                {pi.objectives || (canManage
                  ? <span className="italic opacity-60">Click to add PI objectives…</span>
                  : <span className="opacity-40">No objectives set</span>)}
              </span>
              {canManage && <Edit2 className="w-3 h-3 opacity-0 group-hover:opacity-50 mt-0.5 shrink-0" />}
            </div>
          )}
        </div>

        {/* Sprint progress bar */}
        {sprints.length > 0 && (
          <div className="mt-3">
            <div className="flex justify-between text-xs text-muted-foreground mb-1">
              <span>Sprint progress</span>
              <span>{completedSprints}/{sprints.length}</span>
            </div>
            <div className="h-1.5 bg-gray-100 rounded-full overflow-hidden">
              <div
                className="h-full bg-gradient-to-r from-indigo-400 to-purple-500 rounded-full transition-all"
                style={{ width: `${sprints.length ? (completedSprints / sprints.length) * 100 : 0}%` }}
              />
            </div>
          </div>
        )}
      </CardHeader>

      {/* Sprint grid */}
      {expanded && sprints.length > 0 && (
        <CardContent className="px-5 pb-5 pt-0">
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
            {sprints
              .slice()
              .sort((a, b) => (a.startDate ?? '').localeCompare(b.startDate ?? ''))
              .map(s => (
                <SprintCard
                  key={s.id}
                  sprint={s}
                  canManage={canManage}
                  showBoard={showBoard}
                  onGoalSave={(id, goals) => onUpdateSprint(id, { goals })}
                  onStatusChange={(id, status) => onUpdateSprint(id, { status })}
                  onOpenBoard={onOpenBoard}
                />
              ))}
          </div>
        </CardContent>
      )}
    </Card>
  );
}

// ── Create PI dialog ──────────────────────────────────────────────────────────

function CreatePIDialog({
  existingPIs,
  onClose,
  onCreate,
  isSaving,
}: {
  existingPIs: PI[];
  onClose: () => void;
  onCreate: (name: string, startDate: string, objectives: string) => void;
  isSaving: boolean;
}) {
  const [name, setName] = useState(suggestPIName(existingPIs));
  const [startDate, setStartDate] = useState(() => new Date().toISOString().split('T')[0]);
  const [objectives, setObjectives] = useState('');
  const [errors, setErrors] = useState<Record<string, string>>({});

  const endDate = (() => {
    if (!startDate) return '—';
    const d = new Date(startDate);
    d.setDate(d.getDate() + 83);
    return formatDate(d.toISOString().split('T')[0]);
  })();

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const e2: Record<string, string> = {};
    if (!name.trim()) e2.name = 'Name is required';
    if (!startDate) e2.startDate = 'Start date is required';
    if (Object.keys(e2).length) { setErrors(e2); return; }
    onCreate(name.trim(), startDate, objectives.trim());
  };

  return (
    <div className="fixed inset-0 bg-black/40 z-50 flex items-center justify-center p-4" onClick={onClose}>
      <div className="bg-white rounded-xl shadow-2xl w-full max-w-md" onClick={e => e.stopPropagation()}>
        <div className="flex items-center justify-between px-6 py-4 border-b">
          <h2 className="text-base font-semibold text-gray-900">New Program Increment</h2>
          <Button variant="ghost" size="sm" className="h-7 w-7 p-0" onClick={onClose}>
            <X className="w-4 h-4" />
          </Button>
        </div>

        <form onSubmit={handleSubmit} className="px-6 py-5 space-y-4">
          <div>
            <Label htmlFor="piName">PI Name <span className="text-red-500">*</span></Label>
            <Input
              id="piName"
              value={name}
              onChange={e => setName(e.target.value)}
              placeholder="e.g., PI 1 — Q3 2026"
              className={`mt-1 ${errors.name ? 'border-red-500' : ''}`}
              autoFocus
            />
            {errors.name && <p className="text-xs text-red-500 mt-1">{errors.name}</p>}
          </div>

          <div>
            <Label htmlFor="piStart">Start Date <span className="text-red-500">*</span></Label>
            <Input
              id="piStart"
              type="date"
              value={startDate}
              onChange={e => setStartDate(e.target.value)}
              className={`mt-1 ${errors.startDate ? 'border-red-500' : ''}`}
            />
            {errors.startDate && <p className="text-xs text-red-500 mt-1">{errors.startDate}</p>}
            {startDate && (
              <p className="text-xs text-muted-foreground mt-1">
                Ends <strong>{endDate}</strong> · 6 × 2-week sprints auto-generated
              </p>
            )}
          </div>

          <div>
            <Label htmlFor="piObj">PI Objectives</Label>
            <Textarea
              id="piObj"
              value={objectives}
              onChange={e => setObjectives(e.target.value)}
              placeholder="Key outcomes this PI aims to deliver…"
              rows={3}
              className="mt-1"
            />
          </div>

          <div className="flex gap-3 pt-1">
            <Button type="button" variant="outline" className="flex-1" onClick={onClose} disabled={isSaving}>
              Cancel
            </Button>
            <Button type="submit" className="flex-1" disabled={isSaving}>
              {isSaving ? 'Creating…' : (
                <>
                  <Plus className="w-4 h-4 mr-1.5" />
                  Create PI
                </>
              )}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}

// ── Board: drag-and-drop types ────────────────────────────────────────────────

const DND_TYPE = 'REQUIREMENT';

interface DragItem { id: string; currentPiId: string | undefined; }

// ── Priority / category helpers ───────────────────────────────────────────────

const PRIORITY_COLORS: Record<string, string> = {
  Critical: 'bg-red-100 text-red-700',
  High:     'bg-orange-100 text-orange-700',
  Medium:   'bg-yellow-100 text-yellow-700',
  Low:      'bg-gray-100 text-gray-600',
};

const CAT_COLORS: Record<string, string> = {
  Functional:    'bg-blue-50 text-blue-700',
  'Non-Functional': 'bg-purple-50 text-purple-700',
  Technical:     'bg-teal-50 text-teal-700',
  Security:      'bg-red-50 text-red-700',
  Compliance:    'bg-amber-50 text-amber-700',
  Performance:   'bg-green-50 text-green-700',
};

// ── Draggable requirement card ────────────────────────────────────────────────

function RequirementCard({ req, onDrop }: { req: Requirement; onDrop: (id: string, piId: string | undefined) => void }) {
  const [{ isDragging }, drag] = useDrag<DragItem, void, { isDragging: boolean }>(() => ({
    type: DND_TYPE,
    item: { id: req.id, currentPiId: req.piId },
    collect: m => ({ isDragging: m.isDragging() }),
  }), [req.id, req.piId]);

  const priBg = PRIORITY_COLORS[req.priority] ?? 'bg-gray-100 text-gray-600';
  const catBg = CAT_COLORS[req.category] ?? 'bg-gray-50 text-gray-600';

  return (
    <div
      ref={drag}
      className={`group rounded-lg border bg-white p-3 cursor-grab active:cursor-grabbing shadow-sm hover:shadow-md transition-all select-none ${isDragging ? 'opacity-40 scale-95' : ''}`}
    >
      <div className="flex items-start gap-2">
        <GripVertical className="w-3.5 h-3.5 text-gray-300 group-hover:text-gray-400 mt-0.5 shrink-0" />
        <div className="flex-1 min-w-0">
          <p className="text-xs font-medium text-gray-800 leading-snug line-clamp-2">{req.title}</p>
          <div className="flex items-center gap-1.5 flex-wrap mt-1.5">
            <span className={`text-[10px] font-semibold px-1.5 py-0.5 rounded-full ${priBg}`}>{req.priority}</span>
            <span className={`text-[10px] px-1.5 py-0.5 rounded-full ${catBg}`}>{req.category}</span>
            {req.status === 'Finalized' && (
              <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-emerald-50 text-emerald-700">✓ Finalized</span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

// ── Droppable board column ────────────────────────────────────────────────────

function BoardColumn({
  piId,
  title,
  subtitle,
  requirements,
  capacity,
  isBacklog,
  onDrop,
}: {
  piId: string | undefined;
  title: string;
  subtitle: string;
  requirements: Requirement[];
  capacity: number;
  isBacklog?: boolean;
  onDrop: (reqId: string, targetPiId: string | undefined) => void;
}) {
  const [{ isOver, canDrop }, drop] = useDrop<DragItem, void, { isOver: boolean; canDrop: boolean }>(() => ({
    accept: DND_TYPE,
    drop: (item) => { if (item.currentPiId !== piId) onDrop(item.id, piId); },
    canDrop: (item) => item.currentPiId !== piId,
    collect: m => ({ isOver: m.isOver(), canDrop: m.canDrop() }),
  }), [piId, onDrop]);

  const used = requirements.length;
  const pct = Math.min(100, (used / capacity) * 100);
  const barColor = pct >= 90 ? 'bg-red-400' : pct >= 70 ? 'bg-amber-400' : 'bg-indigo-400';

  return (
    <div
      ref={drop}
      className={`flex flex-col min-w-[220px] max-w-[260px] rounded-xl border-2 transition-all ${
        isOver && canDrop ? 'border-indigo-400 bg-indigo-50/60 shadow-lg' :
        isBacklog ? 'border-dashed border-gray-200 bg-gray-50/50' :
        'border-gray-200 bg-white/70'
      }`}
    >
      {/* Column header */}
      <div className={`px-4 pt-4 pb-3 ${isBacklog ? '' : 'border-b border-gray-100'}`}>
        <div className="flex items-start justify-between gap-2">
          <div className="min-w-0">
            <h3 className={`text-sm font-bold truncate ${isBacklog ? 'text-gray-500' : 'text-gray-800'}`}>{title}</h3>
            <p className="text-[11px] text-muted-foreground mt-0.5 leading-tight">{subtitle}</p>
          </div>
          <span className={`text-xs font-bold px-2 py-0.5 rounded-full shrink-0 ${
            isBacklog ? 'bg-gray-100 text-gray-500' : 'bg-indigo-50 text-indigo-600'
          }`}>{used}</span>
        </div>

        {!isBacklog && (
          <div className="mt-2.5">
            <div className="flex justify-between text-[10px] text-muted-foreground mb-1">
              <span>Capacity</span>
              <span>{used}/{capacity}</span>
            </div>
            <div className="h-1 bg-gray-100 rounded-full overflow-hidden">
              <div className={`h-full rounded-full transition-all ${barColor}`} style={{ width: `${pct}%` }} />
            </div>
          </div>
        )}
      </div>

      {/* Requirement cards */}
      <div className="flex-1 flex flex-col gap-2 p-3 min-h-[120px]">
        {requirements.length === 0 && (
          <div className={`flex-1 flex items-center justify-center rounded-lg border border-dashed ${
            isOver && canDrop ? 'border-indigo-300 bg-indigo-50' : 'border-gray-200'
          } min-h-[80px]`}>
            <p className="text-[11px] text-muted-foreground text-center px-2">
              {isOver && canDrop ? 'Drop here' : isBacklog ? 'All requirements allocated' : 'Drag requirements here'}
            </p>
          </div>
        )}
        {requirements.map(r => (
          <RequirementCard key={r.id} req={r} onDrop={onDrop} />
        ))}
        {requirements.length > 0 && isOver && canDrop && (
          <div className="rounded-lg border-2 border-dashed border-indigo-300 h-10 bg-indigo-50 flex items-center justify-center">
            <p className="text-[11px] text-indigo-400">Drop here</p>
          </div>
        )}
      </div>
    </div>
  );
}

// ── PI Board view ─────────────────────────────────────────────────────────────

function PIBoardView({
  pis,
  requirements,
  canManage,
  onAllocate,
}: {
  pis: PI[];
  requirements: Requirement[];
  canManage: boolean;
  onAllocate: (reqId: string, piId: string | undefined) => Promise<void>;
}) {
  const backlog = requirements.filter(r => !r.piId);
  const allocated = (piId: string) => requirements.filter(r => r.piId === piId);

  const handleDrop = useCallback(async (reqId: string, targetPiId: string | undefined) => {
    await onAllocate(reqId, targetPiId);
  }, [onAllocate]);

  if (pis.length === 0) {
    return (
      <div className="flex items-center justify-center py-20 text-center">
        <div>
          <Package className="w-12 h-12 text-gray-300 mx-auto mb-3" />
          <p className="text-sm font-medium text-gray-500">No Program Increments yet</p>
          <p className="text-xs text-muted-foreground mt-1">Create a PI in the Timeline tab first, then come back to allocate requirements.</p>
        </div>
      </div>
    );
  }

  const totalAllocated = requirements.filter(r => !!r.piId).length;
  const totalReqs = requirements.length;

  return (
    <div>
      {/* Summary bar */}
      <div className="flex items-center gap-4 mb-5 p-3 bg-white/70 rounded-xl border text-xs text-muted-foreground">
        <span className="flex items-center gap-1.5">
          <TrendingUp className="w-3.5 h-3.5 text-indigo-400" />
          <strong className="text-gray-700">{totalAllocated}</strong> / {totalReqs} requirements allocated
        </span>
        <span className="flex items-center gap-1.5">
          <Inbox className="w-3.5 h-3.5 text-amber-400" />
          <strong className="text-gray-700">{backlog.length}</strong> in backlog
        </span>
        {!canManage && (
          <span className="ml-auto flex items-center gap-1 text-amber-600">
            <AlertCircle className="w-3.5 h-3.5" /> View only — only managers can allocate requirements
          </span>
        )}
      </div>

      {/* Board */}
      <div className="flex gap-4 overflow-x-auto pb-4 items-start">
        {/* Backlog */}
        <BoardColumn
          piId={undefined}
          title="Backlog"
          subtitle={`${backlog.length} unallocated`}
          requirements={backlog}
          capacity={999}
          isBacklog
          onDrop={canManage ? handleDrop : () => {}}
        />

        {/* Divider */}
        <div className="shrink-0 w-px bg-gray-200 self-stretch mx-1" />

        {/* PI columns */}
        {pis.map(pi => {
          const piReqs = allocated(pi.id);
          const capacity = 12; // default capacity
          return (
            <BoardColumn
              key={pi.id}
              piId={pi.id}
              title={pi.name}
              subtitle={`${formatDate(pi.startDate)} – ${formatDate(pi.endDate)}`}
              requirements={piReqs}
              capacity={capacity}
              onDrop={canManage ? handleDrop : () => {}}
            />
          );
        })}
      </div>

      <p className="text-[11px] text-muted-foreground mt-4">
        {canManage
          ? 'Drag and drop requirements between the Backlog and PI columns to allocate them.'
          : 'Requirement allocation is read-only for your role.'}
      </p>
    </div>
  );
}

// ── Main page ─────────────────────────────────────────────────────────────────

export default function PIPlanningPage() {
  const navigate = useNavigate();
  const { projectId } = useParams<{ projectId: string }>();
  const { user } = useAuthStore();
  const { hasRole, fetchTeam } = useTeamStore();
  const { pis, sprints, isLoading, isSaving, fetchPIs, createPI, updatePI, deletePI, updateSprint, sprintsForPI } = usePIStore();
  const sprintBoardEnabled = useFeatureFlag('sprintBoard');
  const { requirements, fetchRequirements, updateRequirement } = useRequirementsStore();
  const [showCreate, setShowCreate] = useState(false);
  const [project, setProject] = useState<{ name: string } | null>(null);
  const [view, setView] = useState<'timeline' | 'board'>('timeline');

  useEffect(() => {
    if (!projectId) return;
    fetchPIs(projectId);
    fetchTeam(projectId);
    fetchRequirements(projectId);
    if (user) {
      db.projects.getAll(user.id).then(all => {
        const p = all.find(x => x.id === projectId);
        if (p) setProject({ name: p.name });
      }).catch(() => {});
    }
  }, [projectId]);

  const orgRole = user?.role ?? '';
  const canManage = (user && hasRole(user.id, 'delivery_manager')) || orgRole === 'admin' || orgRole === 'manager';

  const handleCreate = async (name: string, startDate: string, objectives: string) => {
    if (!projectId || !user) return;
    const pi = await createPI(projectId, { name, startDate, objectives, createdBy: user.id });
    if (pi) setShowCreate(false);
  };

  const handleAllocate = useCallback(async (reqId: string, piId: string | undefined) => {
    await updateRequirement(reqId, { piId: piId ?? undefined });
    // Refresh requirements to get updated piId
    if (projectId) await fetchRequirements(projectId);
  }, [projectId, updateRequirement, fetchRequirements]);

  return (
    <DndProvider backend={HTML5Backend}>
      <div className="min-h-screen bg-gradient-to-br from-indigo-50 via-purple-50 to-blue-50 py-6">
        <div className="max-w-full mx-auto px-4 sm:px-6 xl:px-10">

          {/* Header */}
          <div className="mb-6">
            <Button variant="ghost" onClick={() => navigate(`/projects/${projectId}`)} className="-ml-2 mb-3">
              <ArrowLeft className="w-4 h-4 mr-2" />
              {project?.name ?? 'Project'}
            </Button>

            <div className="flex items-start justify-between gap-4 flex-wrap">
              <div>
                <h1 className="text-2xl font-bold text-gray-900">PI Planning</h1>
                <p className="text-sm text-muted-foreground mt-0.5">
                  {pis.length} PI{pis.length !== 1 ? 's' : ''} · {sprints.length} sprints · {requirements.length} requirements
                </p>
              </div>

              <div className="flex items-center gap-2 flex-wrap">
                {/* View switcher */}
                <div className="flex rounded-lg border bg-white overflow-hidden">
                  <button
                    onClick={() => setView('timeline')}
                    className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium transition-colors ${view === 'timeline' ? 'bg-indigo-600 text-white' : 'text-muted-foreground hover:bg-gray-50'}`}
                  >
                    <List className="w-3.5 h-3.5" />Timeline
                  </button>
                  <button
                    onClick={() => setView('board')}
                    className={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium transition-colors ${view === 'board' ? 'bg-indigo-600 text-white' : 'text-muted-foreground hover:bg-gray-50'}`}
                  >
                    <LayoutGrid className="w-3.5 h-3.5" />Board
                  </button>
                </div>

                <Button variant="outline" onClick={() => navigate(`/projects/${projectId}/pi-planning/metrics`)}>
                  <TrendingUp className="w-4 h-4 mr-2" />
                  Metrics
                </Button>

                {canManage && view === 'timeline' && (
                  <Button onClick={() => setShowCreate(true)} disabled={isSaving}>
                    <Plus className="w-4 h-4 mr-2" />
                    New PI
                  </Button>
                )}
              </div>
            </div>

            {!canManage && (
              <div className="mt-3 flex items-center gap-2 text-xs text-amber-700 bg-amber-50 border border-amber-200 rounded-lg px-3 py-2">
                <AlertCircle className="w-3.5 h-3.5 shrink-0" />
                Only Delivery Managers, managers, or admins can create or modify Program Increments.
              </div>
            )}
          </div>

          {/* Loading */}
          {isLoading && (
            <div className="space-y-4">
              {[1, 2].map(i => (
                <div key={i} className="rounded-xl bg-white/60 animate-pulse h-40 border" />
              ))}
            </div>
          )}

          {/* ── Timeline view ── */}
          {!isLoading && view === 'timeline' && (
            <>
              {pis.length > 0 ? (
                <div className="space-y-4">
                  {pis.map(pi => (
                    <PICard
                      key={pi.id}
                      pi={pi}
                      sprints={sprintsForPI(pi.id)}
                      canManage={canManage}
                      showBoard={sprintBoardEnabled}
                      onUpdatePI={updatePI}
                      onDeletePI={deletePI}
                      onUpdateSprint={updateSprint}
                      onOpenBoard={(sprintId) => navigate(`/projects/${projectId}/sprints/${sprintId}`)}
                    />
                  ))}
                </div>
              ) : (
                <Card className="text-center py-16 bg-white/60">
                  <CardContent>
                    <div className="w-14 h-14 bg-indigo-100 rounded-full flex items-center justify-center mx-auto mb-4">
                      <Layers className="w-7 h-7 text-indigo-500" />
                    </div>
                    <h3 className="text-lg font-semibold text-gray-800 mb-1">No Program Increments yet</h3>
                    <p className="text-sm text-muted-foreground mb-5 max-w-sm mx-auto">
                      A PI spans 6 sprints (12 weeks). Once created, it's visible to the whole team.
                    </p>
                    {canManage ? (
                      <Button onClick={() => setShowCreate(true)}>
                        <Plus className="w-4 h-4 mr-2" />Create First PI
                      </Button>
                    ) : (
                      <p className="text-xs text-muted-foreground italic">Ask your Delivery Manager to create the first PI.</p>
                    )}
                  </CardContent>
                </Card>
              )}

              {!isLoading && (
                <div className="mt-6 p-4 bg-white/60 rounded-xl border text-xs text-muted-foreground">
                  <p className="font-medium text-gray-700 mb-1">SAFe Cadence</p>
                  <p>Each <strong>Program Increment</strong> = <strong>6 × 2-week sprints</strong> = 12 weeks. Sprint dates are auto-calculated from the PI start date. Use the <strong>Board</strong> tab to allocate requirements to PIs.</p>
                </div>
              )}
            </>
          )}

          {/* ── Board view ── */}
          {!isLoading && view === 'board' && (
            <PIBoardView
              pis={pis}
              requirements={requirements}
              canManage={canManage}
              onAllocate={handleAllocate}
            />
          )}
        </div>

        {/* Create dialog */}
        {showCreate && (
          <CreatePIDialog
            existingPIs={pis}
            onClose={() => setShowCreate(false)}
            onCreate={handleCreate}
            isSaving={isSaving}
          />
        )}
      </div>
    </DndProvider>
  );
}
