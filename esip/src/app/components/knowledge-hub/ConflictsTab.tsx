import React, { useState } from 'react';
import { Swords, Trash2, CheckCircle2, XCircle, ChevronDown, ChevronUp, ExternalLink } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
import { Textarea } from '../ui/textarea';
import { Label } from '../ui/label';
import { useKnowledgeHubStore } from '../../store/knowledgeHubStore';
import { toast } from 'sonner';

interface ConflictsTabProps {
  roadmapItemId: string;
  conflicts: any[];
  onRefresh: () => void;
  onNavigateToSource?: (sourceType: string, sourceId: string) => void;
}

const severityColors: Record<string, string> = {
  Critical: 'border-l-red-500 bg-red-50/30',
  High: 'border-l-orange-500 bg-orange-50/30',
  Medium: 'border-l-yellow-500 bg-yellow-50/30',
};
const severityBadge: Record<string, string> = {
  Critical: 'bg-red-100 text-red-800 border-red-200',
  High: 'bg-orange-100 text-orange-800 border-orange-200',
  Medium: 'bg-yellow-100 text-yellow-800 border-yellow-200',
};
const statusBadge: Record<string, string> = {
  Open: 'bg-blue-100 text-blue-800',
  Resolved: 'bg-green-100 text-green-800',
  Dismissed: 'bg-gray-100 text-gray-600',
};
const sourceIcon: Record<string, string> = { Email: '📧', Meeting: '🎥', Chat: '💬' };

export function ConflictsTab({ conflicts, onRefresh, onNavigateToSource }: ConflictsTabProps) {
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [resolvingId, setResolvingId] = useState<string | null>(null);
  const [resolution, setResolution] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const { resolveConflict, dismissConflict, deleteConflict } = useKnowledgeHubStore((s) => ({
    resolveConflict: s.resolveConflict,
    dismissConflict: s.dismissConflict,
    deleteConflict: s.deleteConflict,
  }));

  const handleResolve = async (conflictId: string) => {
    if (!resolution.trim()) return;
    setIsSubmitting(true);
    try {
      await resolveConflict(conflictId, resolution);
      setResolvingId(null);
      setResolution('');
      onRefresh();
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDismiss = async (conflictId: string) => {
    await dismissConflict(conflictId);
    onRefresh();
  };

  const handleDelete = async (conflictId: string) => {
    await deleteConflict(conflictId);
    onRefresh();
  };

  const openCount = conflicts.filter(c => c.status === 'Open').length;
  const criticalCount = conflicts.filter(c => c.severity === 'Critical' || c.severity === 'High').length;

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <Swords className="w-5 h-5 text-red-500" />
        <div>
          <h3 className="font-semibold">Requirement Conflicts</h3>
          <p className="text-xs text-muted-foreground">
            {conflicts.length} total · {openCount} open · {criticalCount} critical/high
          </p>
        </div>
      </div>

      {conflicts.length === 0 && (
        <Card>
          <CardContent className="py-16 text-center text-muted-foreground">
            <Swords className="w-10 h-10 mx-auto mb-3 opacity-30" />
            <p className="font-medium">No conflicts detected</p>
            <p className="text-sm mt-1">Conflicts are detected automatically when new emails, meetings, or chats are added</p>
          </CardContent>
        </Card>
      )}

      <div className="space-y-3">
        {conflicts.map((conflict: any) => (
          <Card key={conflict.id} className={`border-l-4 ${severityColors[conflict.severity] || 'border-l-yellow-500'}`}>
            <CardContent className="pt-4 pb-3">
              <div className="flex items-start justify-between gap-3">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className="font-medium text-sm">{conflict.title}</span>
                    <span className={`text-xs px-2 py-0.5 rounded-full font-medium border ${severityBadge[conflict.severity] || ''}`}>{conflict.severity}</span>
                    <span className={`text-xs px-2 py-0.5 rounded-full font-medium ${statusBadge[conflict.status] || ''}`}>{conflict.status}</span>
                  </div>
                  <p className="text-sm text-muted-foreground mt-1 line-clamp-2">{conflict.description}</p>

                  {/* Source attribution */}
                  <div className="flex items-center gap-1.5 mt-2 flex-wrap">
                    <span className="text-xs text-muted-foreground">Triggered by:</span>
                    <button
                      className="inline-flex items-center gap-1 text-xs bg-muted hover:bg-muted/70 px-2 py-0.5 rounded-full transition-colors"
                      onClick={() => onNavigateToSource?.(conflict.triggerSourceType, conflict.triggerSourceId)}
                    >
                      <span>{sourceIcon[conflict.triggerSourceType] || '📄'}</span>
                      <span className="font-medium">{conflict.triggerSourceName}</span>
                      <ExternalLink className="w-2.5 h-2.5" />
                    </button>
                    {conflict.conflictingRequirementKey && (
                      <>
                        <span className="text-xs text-muted-foreground">vs req:</span>
                        <code className="text-xs font-mono bg-primary/10 text-primary px-2 py-0.5 rounded border border-primary/20">
                          {conflict.conflictingRequirementKey}
                        </code>
                      </>
                    )}
                  </div>
                </div>

                <div className="flex items-center gap-1 shrink-0">
                  {conflict.status === 'Open' && (
                    <>
                      <Button size="sm" variant="outline" className="h-7 text-xs text-green-700 border-green-300" onClick={() => { setResolvingId(conflict.id); setResolution(''); }}>
                        <CheckCircle2 className="w-3 h-3 mr-1" />Resolve
                      </Button>
                      <Button size="sm" variant="outline" className="h-7 text-xs text-gray-600" onClick={() => handleDismiss(conflict.id)}>
                        <XCircle className="w-3 h-3 mr-1" />Dismiss
                      </Button>
                    </>
                  )}
                  <Button size="sm" variant="ghost" className="h-7 w-7 p-0" onClick={() => setExpandedId(expandedId === conflict.id ? null : conflict.id)}>
                    {expandedId === conflict.id ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                  </Button>
                  <Button size="sm" variant="ghost" className="h-7 w-7 p-0 text-destructive hover:text-destructive" onClick={() => handleDelete(conflict.id)}>
                    <Trash2 className="w-3.5 h-3.5" />
                  </Button>
                </div>
              </div>

              {/* Expanded: excerpt + resolution */}
              {expandedId === conflict.id && (
                <div className="mt-3 pt-3 border-t space-y-2">
                  {conflict.triggerExcerpt && (
                    <div className="bg-orange-50 border border-orange-200 rounded p-2">
                      <p className="text-xs font-medium text-orange-700 mb-1">Conflicting excerpt from {conflict.triggerSourceType}</p>
                      <p className="text-sm text-orange-800 italic">"{conflict.triggerExcerpt}"</p>
                    </div>
                  )}
                  {conflict.conflictingRequirementText && (
                    <div className="bg-blue-50 border border-blue-200 rounded p-2">
                      <p className="text-xs font-medium text-blue-700 mb-1">Existing requirement text</p>
                      <p className="text-sm text-blue-800">{conflict.conflictingRequirementText}</p>
                    </div>
                  )}
                  {conflict.status === 'Resolved' && conflict.resolution && (
                    <div className="bg-green-50 rounded p-2">
                      <p className="text-xs font-medium text-green-700 mb-1">Resolution</p>
                      <p className="text-sm text-green-800">{conflict.resolution}</p>
                    </div>
                  )}
                </div>
              )}

              {/* Inline resolve form */}
              {resolvingId === conflict.id && (
                <div className="mt-3 pt-3 border-t space-y-2">
                  <Label className="text-xs">How was this conflict resolved?</Label>
                  <Textarea value={resolution} onChange={e => setResolution(e.target.value)} rows={2} placeholder="Describe the decision made to resolve this conflict..." />
                  <div className="flex gap-2 justify-end">
                    <Button variant="outline" size="sm" onClick={() => setResolvingId(null)}>Cancel</Button>
                    <Button size="sm" onClick={() => handleResolve(conflict.id)} disabled={!resolution.trim() || isSubmitting}>
                      {isSubmitting ? 'Saving...' : 'Mark Resolved'}
                    </Button>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
