import React, { useEffect, useCallback, useState } from 'react';
import { useNavigate, useParams } from 'react-router';
import { ArrowLeft, GripVertical, Clock, CheckCircle2, PlayCircle, Eye, Inbox } from 'lucide-react';
import { DndProvider, useDrag, useDrop } from 'react-dnd';
import { HTML5Backend } from 'react-dnd-html5-backend';
import { Badge } from '../components/ui/badge';
import { usePIStore } from '../store/piStore';
import { useRequirementsStore } from '../store/requirementsStore';
import type { Requirement } from '../lib/db/types';

// ── Constants ─────────────────────────────────────────────────────────────────

const DND_TYPE = 'SPRINT_REQ';

const COLUMNS: { id: string; label: string; icon: React.ReactNode; color: string }[] = [
  { id: 'backlog',     label: 'Backlog',     icon: <Inbox className="w-4 h-4" />,        color: 'border-gray-300 bg-gray-50' },
  { id: 'in_progress', label: 'In Progress', icon: <PlayCircle className="w-4 h-4" />,   color: 'border-blue-300 bg-blue-50' },
  { id: 'in_review',   label: 'In Review',   icon: <Eye className="w-4 h-4" />,           color: 'border-amber-300 bg-amber-50' },
  { id: 'done',        label: 'Done',        icon: <CheckCircle2 className="w-4 h-4" />, color: 'border-green-300 bg-green-50' },
];

const PRIORITY_COLORS: Record<string, string> = {
  critical: 'bg-red-100 text-red-700',
  high:     'bg-orange-100 text-orange-700',
  medium:   'bg-yellow-100 text-yellow-700',
  low:      'bg-gray-100 text-gray-600',
};

const CAT_COLORS: Record<string, string> = {
  functional:    'bg-indigo-100 text-indigo-700',
  non_functional:'bg-purple-100 text-purple-700',
  technical:     'bg-cyan-100 text-cyan-700',
  business:      'bg-pink-100 text-pink-700',
};

// ── Drag item ─────────────────────────────────────────────────────────────────

interface DragItem { id: string; currentStatus: string }

// ── RequirementCard ───────────────────────────────────────────────────────────

function RequirementCard({ req, onMove }: { req: Requirement; onMove: (id: string, status: string) => void }) {
  const [{ isDragging }, drag] = useDrag<DragItem, void, { isDragging: boolean }>(() => ({
    type: DND_TYPE,
    item: { id: req.id, currentStatus: req.status },
    collect: m => ({ isDragging: m.isDragging() }),
  }), [req.id, req.status]);

  return (
    <div
      ref={drag as unknown as React.Ref<HTMLDivElement>}
      style={{ opacity: isDragging ? 0.4 : 1 }}
      className="bg-white rounded-lg border border-gray-200 p-3 shadow-sm cursor-grab active:cursor-grabbing flex flex-col gap-2 hover:shadow-md transition-shadow"
    >
      <div className="flex items-start gap-2">
        <GripVertical className="w-4 h-4 text-gray-300 mt-0.5 shrink-0" />
        <span className="text-sm font-medium text-gray-800 leading-snug">{req.title}</span>
      </div>
      {req.description && (
        <p className="text-xs text-gray-500 line-clamp-2 ml-6">{req.description}</p>
      )}
      <div className="flex flex-wrap gap-1 ml-6">
        {req.priority && (
          <span className={`text-[10px] font-semibold px-1.5 py-0.5 rounded uppercase tracking-wide ${PRIORITY_COLORS[req.priority] ?? 'bg-gray-100 text-gray-600'}`}>
            {req.priority}
          </span>
        )}
        {req.category && (
          <span className={`text-[10px] font-semibold px-1.5 py-0.5 rounded uppercase tracking-wide ${CAT_COLORS[req.category] ?? 'bg-gray-100 text-gray-600'}`}>
            {req.category}
          </span>
        )}
      </div>
    </div>
  );
}

// ── KanbanColumn ──────────────────────────────────────────────────────────────

function KanbanColumn({
  col,
  requirements,
  onMove,
}: {
  col: typeof COLUMNS[0];
  requirements: Requirement[];
  onMove: (id: string, status: string) => void;
}) {
  const [{ isOver, canDrop }, drop] = useDrop<DragItem, void, { isOver: boolean; canDrop: boolean }>(() => ({
    accept: DND_TYPE,
    drop: item => { if (item.currentStatus !== col.id) onMove(item.id, col.id); },
    canDrop: item => item.currentStatus !== col.id,
    collect: m => ({ isOver: m.isOver(), canDrop: m.canDrop() }),
  }), [col.id, onMove]);

  const highlight = isOver && canDrop;

  return (
    <div
      ref={drop as unknown as React.Ref<HTMLDivElement>}
      className={`flex flex-col min-h-[400px] rounded-xl border-2 transition-colors ${col.color} ${highlight ? 'ring-2 ring-indigo-400 ring-offset-1' : ''}`}
    >
      {/* Header */}
      <div className="flex items-center gap-2 px-4 py-3 border-b border-inherit">
        <span className="text-gray-600">{col.icon}</span>
        <span className="text-sm font-semibold text-gray-700">{col.label}</span>
        <span className="ml-auto text-xs font-medium bg-white rounded-full px-2 py-0.5 border border-gray-200 text-gray-500">
          {requirements.length}
        </span>
      </div>

      {/* Cards */}
      <div className="flex flex-col gap-2 p-3 flex-1">
        {requirements.length === 0 ? (
          <div className={`flex-1 flex items-center justify-center rounded-lg border-2 border-dashed ${highlight ? 'border-indigo-400 bg-indigo-50/50' : 'border-gray-200'} min-h-[80px]`}>
            <span className="text-xs text-gray-400">Drop here</span>
          </div>
        ) : (
          requirements.map(req => (
            <RequirementCard key={req.id} req={req} onMove={onMove} />
          ))
        )}
        {requirements.length > 0 && highlight && (
          <div className="h-12 rounded-lg border-2 border-dashed border-indigo-400 bg-indigo-50/50 flex items-center justify-center">
            <span className="text-xs text-indigo-500">Drop here</span>
          </div>
        )}
      </div>
    </div>
  );
}

// ── SprintBoardPage ───────────────────────────────────────────────────────────

export default function SprintBoardPage() {
  const { projectId, sprintId } = useParams<{ projectId: string; sprintId: string }>();
  const navigate = useNavigate();

  const { sprints, fetchPIs } = usePIStore();
  const { requirements, fetchRequirements, updateRequirement } = useRequirementsStore();

  const sprint = sprints.find(s => s.id === sprintId);

  useEffect(() => {
    if (!projectId) return;
    fetchPIs(projectId);
    fetchRequirements(projectId);
  }, [projectId]);

  const handleMove = useCallback(async (reqId: string, newStatus: string) => {
    await updateRequirement(reqId, { status: newStatus });
    if (projectId) await fetchRequirements(projectId);
  }, [projectId, updateRequirement, fetchRequirements]);

  const sprintReqs = requirements.filter(r => r.sprintId === sprintId);

  const grouped: Record<string, Requirement[]> = {};
  for (const col of COLUMNS) {
    grouped[col.id] = sprintReqs.filter(r => {
      if (col.id === 'backlog') return !r.status || r.status === 'draft' || r.status === 'backlog' || r.status === 'open';
      if (col.id === 'in_progress') return r.status === 'in_progress';
      if (col.id === 'in_review') return r.status === 'in_review' || r.status === 'review';
      if (col.id === 'done') return r.status === 'done' || r.status === 'completed' || r.status === 'finalized';
      return false;
    });
  }

  const doneCount = grouped['done'].length;
  const totalCount = sprintReqs.length;
  const pct = totalCount > 0 ? Math.round((doneCount / totalCount) * 100) : 0;

  return (
    <DndProvider backend={HTML5Backend}>
      <div className="flex flex-col gap-6 p-6 min-h-full bg-gray-50">
        {/* Header */}
        <div className="flex items-center gap-3">
          <button
            onClick={() => navigate(`/projects/${projectId}/pi-planning`)}
            className="flex items-center gap-1.5 text-sm text-gray-500 hover:text-gray-800 transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
            PI Planning
          </button>
        </div>

        <div className="flex items-start justify-between gap-4 flex-wrap">
          <div>
            <h1 className="text-2xl font-bold text-gray-900">{sprint?.name ?? 'Sprint Board'}</h1>
            <p className="text-sm text-gray-500 mt-0.5 flex items-center gap-3">
              {sprint?.startDate && sprint?.endDate && (
                <span>
                  {new Date(sprint.startDate).toLocaleDateString('en-GB', { day: 'numeric', month: 'short' })}
                  {' — '}
                  {new Date(sprint.endDate).toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric' })}
                </span>
              )}
              <span className="flex items-center gap-1">
                <Clock className="w-3 h-3" />
                {sprint?.status ?? 'planning'}
              </span>
            </p>
          </div>

          {/* Progress pill */}
          {totalCount > 0 && (
            <div className="flex items-center gap-3 bg-white border border-gray-200 rounded-xl px-4 py-2 shadow-sm">
              <div className="flex flex-col items-end">
                <span className="text-xs text-gray-500">Progress</span>
                <span className="text-lg font-bold text-gray-800">{pct}%</span>
              </div>
              <div className="w-24 h-2 bg-gray-100 rounded-full overflow-hidden">
                <div
                  className={`h-full rounded-full transition-all ${pct >= 90 ? 'bg-green-500' : pct >= 50 ? 'bg-indigo-500' : 'bg-amber-400'}`}
                  style={{ width: `${pct}%` }}
                />
              </div>
              <span className="text-xs text-gray-400">{doneCount}/{totalCount}</span>
            </div>
          )}
        </div>

        {/* Board */}
        {sprintReqs.length === 0 ? (
          <div className="flex-1 flex flex-col items-center justify-center gap-3 py-20 text-center">
            <Inbox className="w-12 h-12 text-gray-300" />
            <p className="text-gray-500 font-medium">No requirements allocated to this sprint</p>
            <p className="text-sm text-gray-400">Allocate requirements from the PI Planning board first.</p>
            <button
              onClick={() => navigate(`/projects/${projectId}/pi-planning`)}
              className="mt-2 text-sm text-indigo-600 hover:underline"
            >
              Go to PI Planning →
            </button>
          </div>
        ) : (
          <div className="grid grid-cols-4 gap-4 overflow-x-auto pb-4">
            {COLUMNS.map(col => (
              <KanbanColumn
                key={col.id}
                col={col}
                requirements={grouped[col.id]}
                onMove={handleMove}
              />
            ))}
          </div>
        )}
      </div>
    </DndProvider>
  );
}
