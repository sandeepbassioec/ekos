import { useEffect, useMemo, useState } from 'react';
import { Inbox, Plus, Send, Check, X, Undo2, ChevronDown, ChevronUp } from 'lucide-react';
import { useWorkflowStore, type WorkflowRequest, type RaiseRequestInput } from '../store/workflowStore';
import { useAuthStore } from '../store/authStore';
import { useRequirementsStore } from '../store/requirementsStore';
import { boardMetadata } from '../lib/boardRequirementMapper';
import { Button } from '../components/ui/button';
import { Badge } from '../components/ui/badge';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '../components/ui/tabs';
import type { WorkflowRequestType, WorkflowRequestStatus } from '../lib/db/types';
import { toast } from 'sonner';

/**
 * Cross-role Inbox (Epic D / catalog Vol 12). One place to raise lightweight requests to
 * another role — clarification, missing-requirement, review, approval, hand-off — and to act
 * on the ones routed to you. Backed by the swappable DAL via `workflowStore`.
 *
 * v1 routes by the platform ROLE (the identity the inbox query matches). Routing to a named
 * person (user directory picker) and persona-scoped routing land with Epic G (directory/entitlements).
 */

const TYPES: { value: WorkflowRequestType; label: string }[] = [
  { value: 'clarification', label: 'Clarification' },
  { value: 'missing-requirement', label: 'Missing requirement' },
  { value: 'review', label: 'Review' },
  { value: 'approval', label: 'Approval' },
  { value: 'hand-off', label: 'Hand-off' },
];

// Roles the inbox can deliver to (must match the stored auth role on `users/{uid}`).
const ROUTE_ROLES = ['manager', 'admin', 'developer'];

const STATUS_VARIANT: Record<WorkflowRequestStatus, 'default' | 'secondary' | 'destructive' | 'outline'> = {
  open: 'secondary',
  'in-progress': 'secondary',
  responded: 'default',
  approved: 'default',
  rejected: 'destructive',
  resolved: 'outline',
  withdrawn: 'outline',
};

const typeLabel = (t: string) => TYPES.find((x) => x.value === t)?.label ?? t;
const isActionable = (s: WorkflowRequestStatus) => s === 'open' || s === 'in-progress';

function EmptyState({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex flex-col items-center justify-center rounded-lg border border-dashed border-border py-12 text-center text-sm text-muted-foreground">
      <Inbox className="mb-2 size-6 opacity-50" />
      {children}
    </div>
  );
}

/** One request, with the actions available given the viewer's relationship to it. */
function RequestCard({
  req,
  mine,
  onRespond,
  onDecide,
  onResolve,
  onWithdraw,
  onApproveAIBatch,
}: {
  req: WorkflowRequest;
  mine: boolean; // true = I raised it; false = it's assigned to me
  onRespond: (id: string, note: string) => void;
  onDecide: (id: string, decision: 'approved' | 'rejected', note: string) => void;
  onResolve: (id: string) => void;
  onWithdraw: (id: string) => void;
  onApproveAIBatch: (id: string, note: string) => Promise<void>;
}) {
  const [note, setNote] = useState('');
  const [expanded, setExpanded] = useState(false);
  const [approving, setApproving] = useState(false);
  const actionable = isActionable(req.status);
  const isApproval = req.type === 'approval';
  const isAIBatch = req.subjectType === 'ai-requirements-batch';
  const aiReqs: any[] = isAIBatch ? ((req.metadata?.aiRequirements as any[]) ?? []) : [];

  const handleApproveAIBatch = async () => {
    setApproving(true);
    try {
      await onApproveAIBatch(req.id, note);
    } finally {
      setApproving(false);
    }
  };

  return (
    <div className="rounded-lg border border-border bg-card p-4">
      <div className="mb-1 flex flex-wrap items-center gap-2">
        <Badge variant="outline">{typeLabel(req.type)}</Badge>
        <Badge variant={STATUS_VARIANT[req.status]}>{req.status}</Badge>
        {isAIBatch && <Badge variant="outline" className="border-purple-300 text-purple-700 bg-purple-50">AI Requirements</Badge>}
        <h3 className="font-medium">{req.title}</h3>
      </div>
      {req.description && <p className="mb-2 text-sm text-muted-foreground">{req.description}</p>}
      <p className="text-xs text-muted-foreground">
        {mine ? (
          <>To: {req.targetRole ? <span className="font-medium">{req.targetRole}</span> : 'a specific user'}</>
        ) : (
          <>From: <span className="font-medium">{req.raiserName || 'Unknown'}</span></>
        )}
      </p>

      {/* Expandable requirements list for AI batch review */}
      {isAIBatch && aiReqs.length > 0 && (
        <div className="mt-3 rounded-md border border-purple-200 bg-purple-50/40">
          <button
            className="w-full flex items-center justify-between px-3 py-2 text-sm font-medium text-purple-800"
            onClick={() => setExpanded(v => !v)}
          >
            <span>{aiReqs.length} requirement{aiReqs.length !== 1 ? 's' : ''} pending review</span>
            {expanded ? <ChevronUp className="size-4" /> : <ChevronDown className="size-4" />}
          </button>
          {expanded && (
            <div className="border-t border-purple-200 divide-y divide-purple-100">
              {aiReqs.map((r: any, i: number) => (
                <div key={i} className="px-3 py-2 text-sm">
                  <div className="font-medium text-foreground">{r.suggestedTitle}</div>
                  <div className="flex items-center gap-2 mt-0.5">
                    <span className="text-xs text-muted-foreground capitalize">{r.suggestedType?.replace(/_/g, ' ')}</span>
                    <span className="text-xs text-muted-foreground">·</span>
                    <span className="text-xs text-muted-foreground capitalize">{r.suggestedPriority} priority</span>
                    {r.confidence != null && (
                      <>
                        <span className="text-xs text-muted-foreground">·</span>
                        <span className="text-xs text-muted-foreground">{Math.round(r.confidence * 100)}% confidence</span>
                      </>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {req.response && (
        <div className="mt-2 rounded-md bg-muted p-2 text-sm">
          <span className="text-xs font-medium text-muted-foreground">
            {req.resolvedByName ? `${req.resolvedByName}: ` : 'Response: '}
          </span>
          {req.response}
        </div>
      )}

      {/* Actions for the recipient of an actionable request */}
      {!mine && actionable && (
        <div className="mt-3 space-y-2">
          <textarea
            value={note}
            onChange={(e) => setNote(e.target.value)}
            placeholder={isAIBatch ? 'Review note (optional)…' : isApproval ? 'Decision note (optional)…' : 'Type your response…'}
            rows={2}
            className="w-full rounded-md border border-border bg-background p-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
          />
          <div className="flex flex-wrap gap-2">
            {isAIBatch ? (
              <>
                <Button size="sm" onClick={handleApproveAIBatch} disabled={approving}>
                  <Check className="size-4" /> {approving ? 'Creating…' : 'Approve & Add to Board'}
                </Button>
                <Button size="sm" variant="destructive" onClick={() => onDecide(req.id, 'rejected', note)}>
                  <X className="size-4" /> Reject
                </Button>
              </>
            ) : isApproval ? (
              <>
                <Button size="sm" onClick={() => onDecide(req.id, 'approved', note)}>
                  <Check className="size-4" /> Approve
                </Button>
                <Button size="sm" variant="destructive" onClick={() => onDecide(req.id, 'rejected', note)}>
                  <X className="size-4" /> Reject
                </Button>
              </>
            ) : (
              <Button size="sm" disabled={!note.trim()} onClick={() => onRespond(req.id, note)}>
                <Send className="size-4" /> Respond
              </Button>
            )}
          </div>
        </div>
      )}

      {/* Recipient can close out a request they've already answered */}
      {!mine && req.status === 'responded' && (
        <div className="mt-3">
          <Button size="sm" variant="outline" onClick={() => onResolve(req.id)}>
            <Check className="size-4" /> Mark resolved
          </Button>
        </div>
      )}

      {/* Raiser can withdraw while it's still open */}
      {mine && actionable && (
        <div className="mt-3">
          <Button size="sm" variant="outline" onClick={() => onWithdraw(req.id)}>
            <Undo2 className="size-4" /> Withdraw
          </Button>
        </div>
      )}
    </div>
  );
}

export default function InboxPage() {
  const user = useAuthStore((s) => s.user);
  const { assigned, raisedByMe, isLoading, refresh, raiseRequest, respond, decide, resolve, withdraw } =
    useWorkflowStore();
  const { createRequirement } = useRequirementsStore();

  const [tab, setTab] = useState('assigned');
  const [showForm, setShowForm] = useState(false);

  // New-request form state
  const [form, setForm] = useState<RaiseRequestInput>({ type: 'clarification', title: '', targetRole: 'manager' });
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const awaitingApproval = useMemo(
    () => assigned.filter((r) => r.type === 'approval' && (isActionable(r.status) || r.status === 'responded')),
    [assigned],
  );

  const handleApproveAIBatch = async (id: string, note: string) => {
    const req = assigned.find(r => r.id === id);
    if (!req?.metadata) return;
    const aiReqs: any[] = (req.metadata.aiRequirements as any[]) ?? [];
    const projId = (req.metadata.projectId as string) ?? req.projectId ?? '';
    if (!projId || !aiReqs.length) {
      await decide(id, 'approved', note);
      return;
    }
    try {
      await Promise.all(aiReqs.map(r =>
        createRequirement({
          projectId: projId,
          title: r.suggestedTitle,
          description: r.suggestedDescription,
          category: r.suggestedType ?? 'feature_change',
          priority: r.suggestedPriority ?? 'medium',
          status: 'draft',
          tags: ['board', 'ai-extracted'],
          createdBy: user?.id ?? 'unknown',
          metadata: boardMetadata({
            type: r.suggestedType,
            title: r.suggestedTitle,
            description: r.suggestedDescription,
            priority: r.suggestedPriority,
          }),
        })
      ));
      await decide(id, 'approved', note || `Approved — ${aiReqs.length} requirement${aiReqs.length !== 1 ? 's' : ''} added to the board.`);
      toast.success(`${aiReqs.length} requirement${aiReqs.length !== 1 ? 's' : ''} added to the project requirements board.`);
    } catch {
      toast.error('Failed to create requirements. Please try again.');
    }
  };

  const submit = async () => {
    if (!form.title.trim()) return;
    setSubmitting(true);
    try {
      await raiseRequest(form);
      setForm({ type: 'clarification', title: '', targetRole: 'manager' });
      setShowForm(false);
      setTab('raised');
    } catch {
      // store logs the error; keep the form open so the user can retry
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="mx-auto max-w-4xl p-6">
      <div className="mb-6 flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold">Inbox</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Raise clarifications, reviews, approvals and hand-offs to another role, and act on the ones
            routed to you{user?.role ? <> (<span className="font-medium">{user.role}</span>)</> : null}.
          </p>
        </div>
        <Button onClick={() => setShowForm((v) => !v)} variant={showForm ? 'outline' : 'default'}>
          <Plus className="size-4" /> New request
        </Button>
      </div>

      {showForm && (
        <div className="mb-6 space-y-3 rounded-lg border border-border bg-card p-4">
          <div className="grid gap-3 sm:grid-cols-2">
            <label className="text-sm">
              <span className="mb-1 block text-muted-foreground">Type</span>
              <select
                value={form.type}
                onChange={(e) => setForm((f) => ({ ...f, type: e.target.value as WorkflowRequestType }))}
                className="w-full rounded-md border border-border bg-background p-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                {TYPES.map((t) => (
                  <option key={t.value} value={t.value}>{t.label}</option>
                ))}
              </select>
            </label>
            <label className="text-sm">
              <span className="mb-1 block text-muted-foreground">Route to role</span>
              <select
                value={form.targetRole}
                onChange={(e) => setForm((f) => ({ ...f, targetRole: e.target.value }))}
                className="w-full rounded-md border border-border bg-background p-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                {ROUTE_ROLES.map((r) => (
                  <option key={r} value={r}>{r}</option>
                ))}
              </select>
            </label>
          </div>
          <label className="block text-sm">
            <span className="mb-1 block text-muted-foreground">Title</span>
            <input
              value={form.title}
              onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))}
              placeholder="What do you need?"
              className="w-full rounded-md border border-border bg-background p-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
            />
          </label>
          <label className="block text-sm">
            <span className="mb-1 block text-muted-foreground">Details (optional)</span>
            <textarea
              value={form.description ?? ''}
              onChange={(e) => setForm((f) => ({ ...f, description: e.target.value }))}
              rows={3}
              placeholder="Add context…"
              className="w-full rounded-md border border-border bg-background p-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
            />
          </label>
          <div className="flex gap-2">
            <Button onClick={submit} disabled={submitting || !form.title.trim()}>
              <Send className="size-4" /> {submitting ? 'Sending…' : 'Send request'}
            </Button>
            <Button variant="ghost" onClick={() => setShowForm(false)}>Cancel</Button>
          </div>
        </div>
      )}

      <Tabs value={tab} onValueChange={setTab}>
        <TabsList className="w-full">
          <TabsTrigger value="assigned">Assigned to me ({assigned.length})</TabsTrigger>
          <TabsTrigger value="approvals">Awaiting my approval ({awaitingApproval.length})</TabsTrigger>
          <TabsTrigger value="raised">Raised by me ({raisedByMe.length})</TabsTrigger>
        </TabsList>

        <TabsContent value="assigned" className="mt-4 space-y-3">
          {isLoading && assigned.length === 0 ? (
            <EmptyState>Loading…</EmptyState>
          ) : assigned.length === 0 ? (
            <EmptyState>Nothing assigned to you.</EmptyState>
          ) : (
            assigned.map((r) => (
              <RequestCard key={r.id} req={r} mine={false} onRespond={respond} onDecide={decide} onResolve={resolve} onWithdraw={withdraw} onApproveAIBatch={handleApproveAIBatch} />
            ))
          )}
        </TabsContent>

        <TabsContent value="approvals" className="mt-4 space-y-3">
          {awaitingApproval.length === 0 ? (
            <EmptyState>No approvals waiting on you.</EmptyState>
          ) : (
            awaitingApproval.map((r) => (
              <RequestCard key={r.id} req={r} mine={false} onRespond={respond} onDecide={decide} onResolve={resolve} onWithdraw={withdraw} onApproveAIBatch={handleApproveAIBatch} />
            ))
          )}
        </TabsContent>

        <TabsContent value="raised" className="mt-4 space-y-3">
          {raisedByMe.length === 0 ? (
            <EmptyState>You haven't raised any requests yet.</EmptyState>
          ) : (
            raisedByMe.map((r) => (
              <RequestCard key={r.id} req={r} mine={true} onRespond={respond} onDecide={decide} onResolve={resolve} onWithdraw={withdraw} onApproveAIBatch={handleApproveAIBatch} />
            ))
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
}
