import React, { useState } from 'react';
import { HelpCircle, Plus, Trash2, CheckCircle2, Clock, ChevronDown, ChevronUp, GitMerge } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
import { Badge } from '../ui/badge';
import { Textarea } from '../ui/textarea';
import { Label } from '../ui/label';
import { Input } from '../ui/input';
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger,
} from '../ui/dialog';
import { db } from '../../lib/db';
import { toast } from 'sonner';
import type { Confusion } from '../../lib/db/types';
import { useKnowledgeHubStore } from '../../store/knowledgeHubStore';

interface ConfusionsTabProps {
  roadmapItemId: string;
  confusions: Confusion[];
  onRefresh: () => void;
  aiRequirements?: { id: string; requirementKey?: string; requirement: string }[];
}

const statusColors: Record<string, string> = {
  Open: 'bg-blue-100 text-blue-800',
  Clarified: 'bg-yellow-100 text-yellow-800',
  Resolved: 'bg-green-100 text-green-800',
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

export function ConfusionsTab({ roadmapItemId, confusions, onRefresh, aiRequirements = [] }: ConfusionsTabProps) {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [clarifyingId, setClarifyingId] = useState<string | null>(null);
  const [clarification, setClarification] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [question, setQuestion] = useState('');
  const [context, setContext] = useState('');

  // Multi-select + merge state
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  const [isMergeLoading, setIsMergeLoading] = useState(false);
  const [mergePreview, setMergePreview] = useState<{ isRelated: boolean; confidence: number; reason: string; merged: Record<string, any> } | null>(null);
  const [showWarning, setShowWarning] = useState(false);

  const getMergePreview = useKnowledgeHubStore((s) => s.getMergePreview);
  const executeConfusionMerge = useKnowledgeHubStore((s) => s.executeConfusionMerge);

  const handleCreate = async () => {
    if (!question || !context) return;
    setIsSubmitting(true);
    try {
      await db.confusions.create({
        roadmapItemId,
        question,
        context,
        sources: [],
        status: 'Open',
      });
      toast.success('Confusion added');
      setQuestion(''); setContext('');
      setIsDialogOpen(false);
      onRefresh();
    } catch {
      toast.error('Failed to add confusion');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleClarify = async (confusion: Confusion) => {
    if (!clarification.trim()) return;
    setIsSubmitting(true);
    try {
      await db.confusions.update(confusion.id, { status: 'Resolved', clarification });
      toast.success('Confusion resolved');
      setClarifyingId(null);
      setClarification('');
      onRefresh();
    } catch {
      toast.error('Failed to update confusion');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await db.confusions.delete(id);
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
      const preview = await getMergePreview('confusion', ids);
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
    const ok = await executeConfusionMerge(ids, preview.merged, preview.confidence, preview.reason);
    if (ok) {
      toast.success('Confusions merged');
      setSelectedIds(new Set());
      setMergePreview(null);
      setShowWarning(false);
      onRefresh();
    } else {
      toast.error('Merge failed');
    }
  };

  const openCount = confusions.filter(c => c.status === 'Open').length;
  const selectedCount = selectedIds.size;
  const anySelected = selectedCount > 0;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <HelpCircle className="w-5 h-5 text-blue-500" />
          <div>
            <h3 className="font-semibold">Confusions & Ambiguities</h3>
            <p className="text-xs text-muted-foreground">
              {confusions.length} total · {openCount} open
            </p>
          </div>
        </div>
        <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
          <DialogTrigger asChild>
            <Button size="sm"><Plus className="w-4 h-4 mr-2" />Add Confusion</Button>
          </DialogTrigger>
          <DialogContent className="max-w-lg">
            <DialogHeader><DialogTitle>Add Confusion / Ambiguity</DialogTitle></DialogHeader>
            <div className="space-y-4 pt-2">
              <div className="space-y-1">
                <Label>Question / Ambiguity *</Label>
                <Input value={question} onChange={e => setQuestion(e.target.value)} placeholder="e.g. Is real-time sync required for all users?" />
              </div>
              <div className="space-y-1">
                <Label>Context *</Label>
                <Textarea value={context} onChange={e => setContext(e.target.value)} rows={3} placeholder="Where did this confusion arise? Which document or statement is unclear?" />
              </div>
              <div className="flex justify-end gap-2">
                <Button variant="outline" onClick={() => setIsDialogOpen(false)}>Cancel</Button>
                <Button onClick={handleCreate} disabled={!question || !context || isSubmitting}>
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

      {confusions.length === 0 && (
        <Card>
          <CardContent className="py-16 text-center text-muted-foreground">
            <HelpCircle className="w-10 h-10 mx-auto mb-3 opacity-30" />
            <p className="font-medium">No confusions identified</p>
            <p className="text-sm mt-1">Run AI analysis or add manually</p>
          </CardContent>
        </Card>
      )}

      <div className="space-y-3">
        {confusions.map(confusion => {
          const linkedReqs = aiRequirements.filter(r => ((confusion as any).relatedRequirementIds || []).includes(r.id));
          const sourceEvents: SourceEvent[] = (confusion as any).sourceEvents ?? [];
          const isSelected = selectedIds.has(confusion.id);
          const isHovered = hoveredId === confusion.id;
          const showCheckbox = anySelected || isHovered;
          return (
          <Card
            key={confusion.id}
            className={`border-l-4 border-l-blue-400 ${isSelected ? 'ring-2 ring-amber-400' : ''}`}
            onMouseEnter={() => setHoveredId(confusion.id)}
            onMouseLeave={() => setHoveredId(null)}
          >
            <CardContent className="pt-4 pb-3">
              <div className="flex items-start justify-between gap-3">
                {/* Checkbox */}
                <div className={`shrink-0 mt-0.5 transition-opacity ${showCheckbox ? 'opacity-100' : 'opacity-0'}`}>
                  <input
                    type="checkbox"
                    checked={isSelected}
                    onChange={() => toggleSelect(confusion.id)}
                    className="w-4 h-4 accent-amber-500 cursor-pointer"
                    aria-label={`Select confusion ${confusion.question}`}
                  />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className="font-medium text-sm">{confusion.question}</span>
                    <span className={`text-xs px-2 py-0.5 rounded-full font-medium ${statusColors[confusion.status]}`}>{confusion.status}</span>
                    <StalenessBadge createdAt={(confusion as any).createdAt} status={confusion.status} />
                  </div>
                  <p className="text-sm text-muted-foreground mt-1 line-clamp-2">{confusion.context}</p>
                </div>
                <div className="flex items-center gap-1 shrink-0">
                  {confusion.status === 'Open' && (
                    <Button size="sm" variant="outline" className="h-7 text-xs" onClick={() => { setClarifyingId(confusion.id); setClarification(''); }}>
                      <CheckCircle2 className="w-3 h-3 mr-1" />
                      Clarify
                    </Button>
                  )}
                  <Button size="sm" variant="ghost" className="h-7 w-7 p-0" onClick={() => setExpandedId(expandedId === confusion.id ? null : confusion.id)}>
                    {expandedId === confusion.id ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                  </Button>
                  <Button size="sm" variant="ghost" className="h-7 w-7 p-0 text-destructive hover:text-destructive" onClick={() => handleDelete(confusion.id)}>
                    <Trash2 className="w-3.5 h-3.5" />
                  </Button>
                </div>
              </div>

              {expandedId === confusion.id && (confusion.clarification || linkedReqs.length > 0 || sourceEvents.length > 0) && (
                <div className="mt-3 pt-3 border-t space-y-2">
                  {confusion.clarification && (
                    <div className="bg-green-50 rounded p-2">
                      <p className="text-xs font-medium text-green-700 mb-1">Clarification</p>
                      <p className="text-sm text-green-800">{confusion.clarification}</p>
                    </div>
                  )}
                  {sourceEvents.length > 0 && <SourceTimeline events={sourceEvents} />}
                  {linkedReqs.length > 0 && (
                    <div>
                      <p className="text-xs font-medium text-muted-foreground mb-1.5">Linked requirements</p>
                      <div className="flex flex-wrap gap-1.5">
                        {linkedReqs.map(r => (
                          <code key={r.id} className="text-xs font-mono bg-primary/10 text-primary px-2 py-0.5 rounded border border-primary/20">
                            {r.requirementKey || r.id.slice(0, 8)}
                          </code>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              )}

              {clarifyingId === confusion.id && (
                <div className="mt-3 pt-3 border-t space-y-2">
                  <Label className="text-xs">Clarification / Answer</Label>
                  <Textarea value={clarification} onChange={e => setClarification(e.target.value)} rows={2} placeholder="Provide the clarification or answer..." />
                  <div className="flex gap-2 justify-end">
                    <Button variant="outline" size="sm" onClick={() => setClarifyingId(null)}>Cancel</Button>
                    <Button size="sm" onClick={() => handleClarify(confusion)} disabled={!clarification.trim() || isSubmitting}>
                      {isSubmitting ? 'Saving...' : 'Save Clarification'}
                    </Button>
                  </div>
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
