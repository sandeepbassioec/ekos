import React, { useState } from 'react';
import { Lightbulb, Plus, Trash2, ThumbsUp, ThumbsDown, Clock, ChevronDown, ChevronUp, GitMerge } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
import { Badge } from '../ui/badge';
import { Textarea } from '../ui/textarea';
import { Label } from '../ui/label';
import { Input } from '../ui/input';
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger,
} from '../ui/dialog';
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '../ui/select';
import { db } from '../../lib/db';
import { toast } from 'sonner';
import type { Suggestion } from '../../lib/db/types';
import { useKnowledgeHubStore } from '../../store/knowledgeHubStore';

interface SuggestionsTabProps {
  roadmapItemId: string;
  suggestions: Suggestion[];
  onRefresh: () => void;
  aiRequirements?: { id: string; requirementKey?: string; requirement: string }[];
}

const typeColors: Record<string, string> = {
  Enhancement: 'bg-blue-100 text-blue-800',
  Alternative: 'bg-purple-100 text-purple-800',
  'Best Practice': 'bg-green-100 text-green-800',
  'Risk Mitigation': 'bg-orange-100 text-orange-800',
};

const statusColors: Record<string, string> = {
  Pending: 'bg-gray-100 text-gray-700',
  Accepted: 'bg-green-100 text-green-800',
  Rejected: 'bg-red-100 text-red-800',
  Implemented: 'bg-blue-100 text-blue-800',
};

const sourceTypeIcon: Record<string, string> = {
  document: '📄',
  meeting: '🎥',
  email: '📧',
  chat: '💬',
  manual: '✏️',
  'ai-merge': '🤖',
};

const eventTypeColors: Record<string, string> = {
  created: 'bg-blue-100 text-blue-700',
  resolved: 'bg-green-100 text-green-700',
  clarified: 'bg-yellow-100 text-yellow-700',
  updated: 'bg-gray-100 text-gray-700',
  merged: 'bg-purple-100 text-purple-700',
};

function relativeTime(ms: number): string {
  const diff = Date.now() - ms;
  const mins = Math.floor(diff / 60000);
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

function StalenessBadge({ createdAt, status }: { createdAt: string | number | undefined; status: string }) {
  if (!createdAt) return null;
  if (status !== 'Open' && status !== 'Pending') return null;
  const daysOld = Math.floor((Date.now() - new Date(createdAt).getTime()) / 86400000);
  if (daysOld > 14) {
    return (
      <span className="inline-flex items-center gap-1 text-xs font-medium text-red-700">
        <span className="w-2 h-2 rounded-full bg-red-500 inline-block" />
        Stale {daysOld}d
      </span>
    );
  }
  if (daysOld > 7) {
    return (
      <span className="inline-flex items-center gap-1" title={`${daysOld} days old`}>
        <span className="w-2 h-2 rounded-full bg-amber-400 inline-block" />
      </span>
    );
  }
  return null;
}

interface SourceEvent {
  type: 'created' | 'resolved' | 'clarified' | 'updated' | 'merged';
  sourceType: 'document' | 'meeting' | 'email' | 'chat' | 'manual' | 'ai-merge';
  sourceName: string;
  excerpt?: string;
  at: number;
}

function SourceTimeline({ events }: { events: SourceEvent[] }) {
  return (
    <div>
      <p className="text-xs font-medium text-muted-foreground mb-1.5">Source Events</p>
      <div className="space-y-1.5">
        {events.map((ev, i) => (
          <div key={i} className="flex items-start gap-2 text-xs">
            <span className="shrink-0 mt-0.5">{sourceTypeIcon[ev.sourceType] ?? '📄'}</span>
            <span className={`shrink-0 px-1.5 py-0.5 rounded text-xs font-medium ${eventTypeColors[ev.type] ?? 'bg-gray-100 text-gray-700'}`}>{ev.type}</span>
            <span className="font-medium text-foreground truncate max-w-[140px]">{ev.sourceName}</span>
            {ev.excerpt && <span className="text-muted-foreground italic truncate flex-1">"{ev.excerpt}"</span>}
            <span className="shrink-0 text-muted-foreground">{relativeTime(ev.at)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

export function SuggestionsTab({ roadmapItemId, suggestions, onRefresh, aiRequirements = [] }: SuggestionsTabProps) {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [type, setType] = useState<Suggestion['type']>('Enhancement');
  const [impact, setImpact] = useState('');
  const [effort, setEffort] = useState('');

  // Multi-select + merge state
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  const [isMergeLoading, setIsMergeLoading] = useState(false);
  const [mergePreview, setMergePreview] = useState<{ isRelated: boolean; confidence: number; reason: string; merged: Record<string, any> } | null>(null);
  const [showWarning, setShowWarning] = useState(false);

  const getMergePreview = useKnowledgeHubStore((s) => s.getMergePreview);
  const executeSuggestionMerge = useKnowledgeHubStore((s) => s.executeSuggestionMerge);

  const handleCreate = async () => {
    if (!title || !description) return;
    setIsSubmitting(true);
    try {
      await db.suggestions.create({
        roadmapItemId,
        title,
        description,
        type,
        status: 'Pending',
        generatedBy: 'Manual',
        impact: impact || undefined,
        effort: effort || undefined,
      });
      toast.success('Suggestion added');
      setTitle(''); setDescription(''); setImpact(''); setEffort(''); setType('Enhancement');
      setIsDialogOpen(false);
      onRefresh();
    } catch {
      toast.error('Failed to add suggestion');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleStatusChange = async (suggestion: Suggestion, status: Suggestion['status']) => {
    try {
      await db.suggestions.update(suggestion.id, { status });
      toast.success(`Marked as ${status}`);
      onRefresh();
    } catch {
      toast.error('Failed to update');
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await db.suggestions.delete(id);
      toast.success('Deleted');
      onRefresh();
    } catch {
      toast.error('Failed to delete');
    }
  };

  const toggleSelect = (id: string) => {
    setSelectedIds(prev => {
      const next = new Set(prev);
      if (next.has(id)) { next.delete(id); } else { next.add(id); }
      return next;
    });
    setMergePreview(null);
    setShowWarning(false);
  };

  const handleMergeClick = async () => {
    const ids = Array.from(selectedIds);
    setIsMergeLoading(true);
    setMergePreview(null);
    setShowWarning(false);
    try {
      const preview = await getMergePreview('suggestion', ids);
      if (!preview) { toast.error('Could not get merge preview'); return; }
      setMergePreview(preview);
      if (preview.confidence < 50) {
        setShowWarning(true);
      } else {
        await doMerge(ids, preview);
      }
    } catch {
      toast.error('Merge preview failed');
    } finally {
      setIsMergeLoading(false);
    }
  };

  const doMerge = async (ids: string[], preview: { confidence: number; reason: string; merged: Record<string, any> }) => {
    const ok = await executeSuggestionMerge(ids, preview.merged, preview.confidence, preview.reason);
    if (ok) {
      toast.success('Suggestions merged');
      setSelectedIds(new Set());
      setMergePreview(null);
      setShowWarning(false);
      onRefresh();
    } else {
      toast.error('Merge failed');
    }
  };

  const pendingCount = suggestions.filter(s => s.status === 'Pending').length;
  const selectedCount = selectedIds.size;
  const anySelected = selectedCount > 0;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Lightbulb className="w-5 h-5 text-yellow-500" />
          <div>
            <h3 className="font-semibold">Suggestions & Recommendations</h3>
            <p className="text-xs text-muted-foreground">
              {suggestions.length} total · {pendingCount} pending review
            </p>
          </div>
        </div>
        <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
          <DialogTrigger asChild>
            <Button size="sm"><Plus className="w-4 h-4 mr-2" />Add Suggestion</Button>
          </DialogTrigger>
          <DialogContent className="max-w-lg">
            <DialogHeader><DialogTitle>Add Suggestion</DialogTitle></DialogHeader>
            <div className="space-y-4 pt-2">
              <div className="space-y-1">
                <Label>Title *</Label>
                <Input value={title} onChange={e => setTitle(e.target.value)} placeholder="e.g. Use OAuth2 for authentication" />
              </div>
              <div className="space-y-1">
                <Label>Description *</Label>
                <Textarea value={description} onChange={e => setDescription(e.target.value)} rows={3} placeholder="Detailed explanation of the suggestion..." />
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <Label>Type</Label>
                  <Select value={type} onValueChange={v => setType(v as Suggestion['type'])}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      {['Enhancement', 'Alternative', 'Best Practice', 'Risk Mitigation'].map(t => <SelectItem key={t} value={t}>{t}</SelectItem>)}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-1">
                  <Label>Effort</Label>
                  <Input value={effort} onChange={e => setEffort(e.target.value)} placeholder="e.g. Low, 2 days" />
                </div>
              </div>
              <div className="space-y-1">
                <Label>Impact</Label>
                <Input value={impact} onChange={e => setImpact(e.target.value)} placeholder="e.g. Reduces security risk significantly" />
              </div>
              <div className="flex justify-end gap-2">
                <Button variant="outline" onClick={() => setIsDialogOpen(false)}>Cancel</Button>
                <Button onClick={handleCreate} disabled={!title || !description || isSubmitting}>
                  {isSubmitting ? 'Adding...' : 'Add'}
                </Button>
              </div>
            </div>
          </DialogContent>
        </Dialog>
      </div>

      {/* Merge Toolbar */}
      {anySelected && (
        <div className="rounded-lg border border-amber-300 bg-amber-50 px-4 py-2.5 space-y-2">
          <div className="flex items-center justify-between gap-3">
            <span className="text-sm font-medium text-amber-800">{selectedCount} selected</span>
            <div className="flex items-center gap-2">
              <Button
                size="sm"
                variant="outline"
                className="h-7 text-xs border-amber-400 text-amber-800 hover:bg-amber-100"
                onClick={handleMergeClick}
                disabled={selectedCount < 2 || isMergeLoading}
              >
                {isMergeLoading ? (
                  <span className="flex items-center gap-1"><Clock className="w-3 h-3 animate-spin" />Analyzing...</span>
                ) : (
                  <span className="flex items-center gap-1"><GitMerge className="w-3 h-3" />Merge</span>
                )}
              </Button>
              <Button
                size="sm"
                variant="ghost"
                className="h-7 text-xs text-amber-700 hover:bg-amber-100"
                onClick={() => { setSelectedIds(new Set()); setMergePreview(null); setShowWarning(false); }}
              >
                Clear
              </Button>
            </div>
          </div>
          {showWarning && mergePreview && mergePreview.confidence < 50 && (
            <div className="rounded border border-red-300 bg-red-50 p-3 space-y-2">
              <p className="text-xs font-medium text-red-800">
                AI confidence: {mergePreview.confidence}% — {mergePreview.reason}. These may be unrelated.
              </p>
              <p className="text-xs text-red-700">Merge anyway?</p>
              <div className="flex gap-2">
                <Button
                  size="sm"
                  variant="destructive"
                  className="h-7 text-xs"
                  onClick={() => doMerge(Array.from(selectedIds), mergePreview)}
                >
                  Yes, force merge
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  className="h-7 text-xs"
                  onClick={() => { setShowWarning(false); setMergePreview(null); }}
                >
                  Cancel
                </Button>
              </div>
            </div>
          )}
        </div>
      )}

      {suggestions.length === 0 && (
        <Card>
          <CardContent className="py-16 text-center text-muted-foreground">
            <Lightbulb className="w-10 h-10 mx-auto mb-3 opacity-30" />
            <p className="font-medium">No suggestions yet</p>
            <p className="text-sm mt-1">Run AI analysis or add manually</p>
          </CardContent>
        </Card>
      )}

      <div className="space-y-3">
        {suggestions.map(s => {
          const linkedReqs = aiRequirements.filter(r => ((s as any).relatedRequirementIds || []).includes(r.id));
          const sourceEvents: SourceEvent[] = (s as any).sourceEvents ?? [];
          const isSelected = selectedIds.has(s.id);
          const isHovered = hoveredId === s.id;
          const showCheckbox = anySelected || isHovered;
          return (
          <Card
            key={s.id}
            className={isSelected ? 'ring-2 ring-amber-400' : ''}
            onMouseEnter={() => setHoveredId(s.id)}
            onMouseLeave={() => setHoveredId(null)}
          >
            <CardContent className="pt-4 pb-3">
              <div className="flex items-start justify-between gap-3">
                {/* Checkbox */}
                <div className={`shrink-0 mt-0.5 transition-opacity ${showCheckbox ? 'opacity-100' : 'opacity-0'}`}>
                  <input
                    type="checkbox"
                    checked={isSelected}
                    onChange={() => toggleSelect(s.id)}
                    className="w-4 h-4 accent-amber-500 cursor-pointer"
                    aria-label={`Select suggestion ${s.title}`}
                  />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className="font-medium text-sm">{s.title}</span>
                    <span className={`text-xs px-2 py-0.5 rounded-full font-medium ${typeColors[s.type]}`}>{s.type}</span>
                    <span className={`text-xs px-2 py-0.5 rounded-full font-medium ${statusColors[s.status]}`}>{s.status}</span>
                    <StalenessBadge createdAt={(s as any).createdAt} status={s.status} />
                    {s.generatedBy === 'AI' && <Badge variant="secondary" className="text-xs">AI</Badge>}
                  </div>
                  <p className="text-sm text-muted-foreground mt-1 line-clamp-2">{s.description}</p>
                  {(s.impact || s.effort) && (
                    <div className="flex gap-3 mt-1">
                      {s.impact && <span className="text-xs text-muted-foreground">Impact: {s.impact}</span>}
                      {s.effort && <span className="text-xs text-muted-foreground">Effort: {s.effort}</span>}
                    </div>
                  )}
                  {linkedReqs.length > 0 && (
                    <div className="flex flex-wrap items-center gap-1.5 mt-2">
                      <span className="text-xs text-muted-foreground">Reqs:</span>
                      {linkedReqs.map(r => (
                        <code key={r.id} className="text-xs font-mono bg-primary/10 text-primary px-2 py-0.5 rounded border border-primary/20">
                          {r.requirementKey || r.id.slice(0, 8)}
                        </code>
                      ))}
                    </div>
                  )}
                </div>
                <div className="flex items-center gap-1 shrink-0">
                  {s.status === 'Pending' && (
                    <>
                      <Button size="sm" variant="outline" className="h-7 text-xs text-green-700 border-green-300" onClick={() => handleStatusChange(s, 'Accepted')}>
                        <ThumbsUp className="w-3 h-3 mr-1" />Accept
                      </Button>
                      <Button size="sm" variant="outline" className="h-7 text-xs text-red-700 border-red-300" onClick={() => handleStatusChange(s, 'Rejected')}>
                        <ThumbsDown className="w-3 h-3 mr-1" />Reject
                      </Button>
                    </>
                  )}
                  {s.status === 'Accepted' && (
                    <Button size="sm" variant="outline" className="h-7 text-xs" onClick={() => handleStatusChange(s, 'Implemented')}>
                      Mark Implemented
                    </Button>
                  )}
                  <Button size="sm" variant="ghost" className="h-7 w-7 p-0" onClick={() => setExpandedId(expandedId === s.id ? null : s.id)}>
                    {expandedId === s.id ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                  </Button>
                  <Button size="sm" variant="ghost" className="h-7 w-7 p-0 text-destructive hover:text-destructive" onClick={() => handleDelete(s.id)}>
                    <Trash2 className="w-3.5 h-3.5" />
                  </Button>
                </div>
              </div>

              {/* Expanded */}
              {expandedId === s.id && (sourceEvents.length > 0) && (
                <div className="mt-3 pt-3 border-t space-y-2">
                  {sourceEvents.length > 0 && <SourceTimeline events={sourceEvents} />}
                </div>
              )}
            </CardContent>
          </Card>
          );
        })}
      </div>
    </div>
  );
}
