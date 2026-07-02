import React, { useState, useMemo } from 'react';
import {
  ClipboardList, Plus, Trash2, Pencil, Sparkles, ChevronDown, ChevronUp,
  CheckCircle2, Clock, AlertTriangle, Filter, Ban, Unlock,
} from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
import { Badge } from '../ui/badge';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Textarea } from '../ui/textarea';
import {
  Dialog, DialogContent, DialogHeader, DialogTitle,
} from '../ui/dialog';
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '../ui/select';
import { useKnowledgeHubStore } from '../../store/knowledgeHubStore';
import { toast } from 'sonner';
import type {
  FormalRequirement,
  FormalRequirementCategory,
  FunctionalType,
  NFRType,
  ConstraintType,
} from '../../lib/db/types';

interface RequirementsTabProps {
  roadmapItemId: string;
  projectId: string;
  requirements: FormalRequirement[];
}

// ── Constants ────────────────────────────────────────────────────────────────

const FUNCTIONAL_TYPES: FunctionalType[] = [
  'Feature', 'Screen', 'API', 'Integration', 'Bug Fix', 'Data Migration', 'Business Rule', 'Other',
];

const NFR_TYPES: NFRType[] = [
  'Performance', 'Security', 'Availability', 'Scalability', 'Concurrency',
  'Reliability', 'Maintainability', 'Usability', 'Compliance', 'Disaster Recovery',
  'Observability', 'Other',
];

const CONSTRAINT_TYPES: ConstraintType[] = ['Technical', 'Budget', 'Timeline', 'Regulatory', 'Other'];

const STATUS_COLORS: Record<string, string> = {
  'Draft':        'bg-gray-100 text-gray-700',
  'Under Review': 'bg-yellow-100 text-yellow-800',
  'Approved':     'bg-green-100 text-green-800',
  'Implemented':  'bg-blue-100 text-blue-800',
  'Rejected':     'bg-red-100 text-red-800',
  'Blocked':      'bg-red-200 text-red-900 font-semibold',
};

const PRIORITY_COLORS: Record<string, string> = {
  Critical: 'bg-red-100 text-red-800 border-red-200',
  High:     'bg-orange-100 text-orange-800 border-orange-200',
  Medium:   'bg-yellow-100 text-yellow-800 border-yellow-200',
  Low:      'bg-green-100 text-green-800 border-green-200',
};

const CATEGORY_COLORS: Record<string, string> = {
  Functional:           'bg-blue-100 text-blue-800',
  'Non-Functional':     'bg-purple-100 text-purple-800',
  Constraint:           'bg-orange-100 text-orange-800',
  'Acceptance Criteria':'bg-teal-100 text-teal-800',
};

// Auto-generate requirement ID
function generateReqId(category: FormalRequirementCategory, existing: FormalRequirement[]): string {
  const prefix = category === 'Functional' ? 'REQ-F' :
                 category === 'Non-Functional' ? 'REQ-NF' :
                 category === 'Constraint' ? 'REQ-C' : 'REQ-AC';
  const same = existing.filter(r => r.requirementId.startsWith(prefix));
  const next = (same.length + 1).toString().padStart(3, '0');
  return `${prefix}-${next}`;
}

// ── Form ─────────────────────────────────────────────────────────────────────

interface FormState {
  category: FormalRequirementCategory;
  functionalType: FunctionalType;
  nfrType: NFRType;
  constraintType: ConstraintType;
  title: string;
  description: string;
  priority: FormalRequirement['priority'];
  status: FormalRequirement['status'];
  asA: string; iWant: string; soThat: string;
  acceptanceCriteria: string;
  metric: string; target: string; measurement: string;
}

const defaultForm = (): FormState => ({
  category: 'Functional',
  functionalType: 'Feature',
  nfrType: 'Performance',
  constraintType: 'Technical',
  title: '', description: '',
  priority: 'Medium', status: 'Draft',
  asA: '', iWant: '', soThat: '',
  acceptanceCriteria: '',
  metric: '', target: '', measurement: '',
});

function fromRequirement(r: FormalRequirement): FormState {
  return {
    category: r.category,
    functionalType: r.functionalType ?? 'Feature',
    nfrType: r.nfrType ?? 'Performance',
    constraintType: r.constraintType ?? 'Technical',
    title: r.title,
    description: r.description,
    priority: r.priority,
    status: r.status,
    asA: r.userStory?.asA ?? '',
    iWant: r.userStory?.iWant ?? '',
    soThat: r.userStory?.soThat ?? '',
    acceptanceCriteria: r.acceptanceCriteria.join('\n'),
    metric: r.metric ?? '',
    target: r.target ?? '',
    measurement: r.measurement ?? '',
  };
}

function RequirementForm({
  form,
  setForm,
  onSubmit,
  onCancel,
  submitLabel,
  isSubmitting,
}: {
  form: FormState;
  setForm: React.Dispatch<React.SetStateAction<FormState>>;
  onSubmit: () => void;
  onCancel: () => void;
  submitLabel: string;
  isSubmitting: boolean;
}) {
  const set = (k: keyof FormState) => (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) =>
    setForm(f => ({ ...f, [k]: e.target.value }));

  return (
    <div className="space-y-4 pt-2 max-h-[70vh] overflow-y-auto pr-1">
      {/* Category + Type row */}
      <div className="grid grid-cols-2 gap-3">
        <div className="space-y-1">
          <Label>Category *</Label>
          <Select value={form.category} onValueChange={v => setForm(f => ({ ...f, category: v as FormalRequirementCategory }))}>
            <SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>
              {(['Functional', 'Non-Functional', 'Constraint', 'Acceptance Criteria'] as FormalRequirementCategory[]).map(c =>
                <SelectItem key={c} value={c}>{c}</SelectItem>
              )}
            </SelectContent>
          </Select>
        </div>
        <div className="space-y-1">
          <Label>Sub-type</Label>
          {form.category === 'Functional' && (
            <Select value={form.functionalType} onValueChange={v => setForm(f => ({ ...f, functionalType: v as FunctionalType }))}>
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>{FUNCTIONAL_TYPES.map(t => <SelectItem key={t} value={t}>{t}</SelectItem>)}</SelectContent>
            </Select>
          )}
          {form.category === 'Non-Functional' && (
            <Select value={form.nfrType} onValueChange={v => setForm(f => ({ ...f, nfrType: v as NFRType }))}>
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>{NFR_TYPES.map(t => <SelectItem key={t} value={t}>{t}</SelectItem>)}</SelectContent>
            </Select>
          )}
          {form.category === 'Constraint' && (
            <Select value={form.constraintType} onValueChange={v => setForm(f => ({ ...f, constraintType: v as ConstraintType }))}>
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>{CONSTRAINT_TYPES.map(t => <SelectItem key={t} value={t}>{t}</SelectItem>)}</SelectContent>
            </Select>
          )}
          {form.category === 'Acceptance Criteria' && (
            <Input disabled value="Acceptance Criteria" className="opacity-50" />
          )}
        </div>
      </div>

      {/* Title */}
      <div className="space-y-1">
        <Label>Title *</Label>
        <Input value={form.title} onChange={set('title')} placeholder="Short, clear requirement title" />
      </div>

      {/* Description */}
      <div className="space-y-1">
        <Label>Description *</Label>
        <Textarea value={form.description} onChange={set('description')} rows={3}
          placeholder="Detailed description. Use IEEE 830 format where possible: [Actor] SHALL [action] [condition]" />
      </div>

      {/* Priority + Status */}
      <div className="grid grid-cols-2 gap-3">
        <div className="space-y-1">
          <Label>Priority</Label>
          <Select value={form.priority} onValueChange={v => setForm(f => ({ ...f, priority: v as FormalRequirement['priority'] }))}>
            <SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>{['Critical','High','Medium','Low'].map(p => <SelectItem key={p} value={p}>{p}</SelectItem>)}</SelectContent>
          </Select>
        </div>
        <div className="space-y-1">
          <Label>Status</Label>
          <Select value={form.status} onValueChange={v => setForm(f => ({ ...f, status: v as FormalRequirement['status'] }))}>
            <SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>{['Draft','Under Review','Approved','Implemented','Rejected'].map(s => <SelectItem key={s} value={s}>{s}</SelectItem>)}</SelectContent>
          </Select>
        </div>
      </div>

      {/* User Story — Functional only */}
      {form.category === 'Functional' && (
        <div className="space-y-2 p-3 bg-muted/50 rounded-lg">
          <Label className="text-xs font-semibold uppercase tracking-wide">User Story (optional)</Label>
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <span className="text-xs text-muted-foreground w-14 shrink-0">As a</span>
              <Input value={form.asA} onChange={set('asA')} placeholder="role / persona" className="h-8 text-sm" />
            </div>
            <div className="flex items-center gap-2">
              <span className="text-xs text-muted-foreground w-14 shrink-0">I want</span>
              <Input value={form.iWant} onChange={set('iWant')} placeholder="feature or capability" className="h-8 text-sm" />
            </div>
            <div className="flex items-center gap-2">
              <span className="text-xs text-muted-foreground w-14 shrink-0">So that</span>
              <Input value={form.soThat} onChange={set('soThat')} placeholder="benefit or outcome" className="h-8 text-sm" />
            </div>
          </div>
        </div>
      )}

      {/* NFR Metrics */}
      {form.category === 'Non-Functional' && (
        <div className="space-y-2 p-3 bg-muted/50 rounded-lg">
          <Label className="text-xs font-semibold uppercase tracking-wide">Measurable Target</Label>
          <div className="grid grid-cols-3 gap-2">
            <div className="space-y-1">
              <Label className="text-xs">Metric</Label>
              <Input value={form.metric} onChange={set('metric')} placeholder="e.g. Response time p95" className="h-8 text-sm" />
            </div>
            <div className="space-y-1">
              <Label className="text-xs">Target</Label>
              <Input value={form.target} onChange={set('target')} placeholder="e.g. &lt; 200ms" className="h-8 text-sm" />
            </div>
            <div className="space-y-1">
              <Label className="text-xs">How to Measure</Label>
              <Input value={form.measurement} onChange={set('measurement')} placeholder="e.g. APM / Datadog" className="h-8 text-sm" />
            </div>
          </div>
        </div>
      )}

      {/* Acceptance Criteria */}
      <div className="space-y-1">
        <Label>Acceptance Criteria (one per line)</Label>
        <Textarea value={form.acceptanceCriteria} onChange={set('acceptanceCriteria')} rows={4}
          placeholder="Each line is one testable acceptance criterion&#10;e.g. System returns results within 200ms&#10;e.g. Error message shown when input is invalid" />
      </div>

      <div className="flex gap-2 pt-1">
        <Button onClick={onSubmit} disabled={!form.title || !form.description || isSubmitting} className="flex-1">
          {isSubmitting ? 'Saving...' : submitLabel}
        </Button>
        <Button variant="outline" onClick={onCancel} disabled={isSubmitting}>Cancel</Button>
      </div>
    </div>
  );
}

// ── Import from AI dialog ────────────────────────────────────────────────────

function AIImportDialog({
  hub,
  roadmapItemId,
  projectId,
  existingRequirements,
  onClose,
  onImported,
}: {
  hub: any;
  roadmapItemId: string;
  projectId: string;
  existingRequirements: FormalRequirement[];
  onClose: () => void;
  onImported: () => void;
}) {
  const { createFormalRequirement } = useKnowledgeHubStore();

  type AIItem = { id: string; label: string; category: FormalRequirementCategory; subType: string; description: string; priority: string; source: string };

  const candidates: AIItem[] = useMemo(() => {
    const items: AIItem[] = [];
    (hub?.aiRequirements ?? []).forEach((r: any) => items.push({
      id: `air-${r.id}`, label: r.requirement?.substring(0, 80) ?? 'AI Requirement',
      category: 'Functional', subType: r.category ?? 'Feature',
      description: r.requirement ?? '', priority: r.priority ?? 'Medium', source: 'AI',
    }));
    (hub?.nfrs ?? []).forEach((n: any) => items.push({
      id: `nfr-${n.id}`, label: n.requirement?.substring(0, 80) ?? 'NFR',
      category: 'Non-Functional', subType: n.category ?? 'Performance',
      description: n.requirement ?? '', priority: n.priority ?? 'Medium', source: 'AI',
    }));
    (hub?.securityRequirements ?? []).forEach((s: any) => items.push({
      id: `sec-${s.id}`, label: s.title ?? 'Security Requirement',
      category: 'Non-Functional', subType: 'Security',
      description: s.description ?? '', priority: s.level ?? 'High', source: 'AI',
    }));
    (hub?.gaps ?? []).filter((g: any) => g.status !== 'Resolved').forEach((g: any) => items.push({
      id: `gap-${g.id}`, label: g.title ?? 'Gap',
      category: 'Functional', subType: 'Feature',
      description: g.description ?? '', priority: g.severity ?? 'Medium', source: 'AI (Gap)',
    }));
    (hub?.suggestions ?? []).filter((s: any) => s.status === 'Accepted').forEach((s: any) => items.push({
      id: `sug-${s.id}`, label: s.title ?? 'Suggestion',
      category: 'Functional', subType: 'Feature',
      description: s.description ?? '', priority: 'Low', source: 'AI (Suggestion)',
    }));
    return items;
  }, [hub]);

  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [importing, setImporting] = useState(false);

  const toggle = (id: string) => {
    setSelected(prev => { const n = new Set(prev); n.has(id) ? n.delete(id) : n.add(id); return n; });
  };

  const handleImport = async () => {
    const toImport = candidates.filter(c => selected.has(c.id));
    if (!toImport.length) return;
    setImporting(true);
    let count = 0;
    for (const item of toImport) {
      const reqId = generateReqId(item.category, [...existingRequirements, ...Array(count).fill({ requirementId: item.category === 'Non-Functional' ? 'REQ-NF-000' : 'REQ-F-000' })]);
      await createFormalRequirement({
        roadmapItemId, projectId,
        requirementId: reqId,
        category: item.category,
        functionalType: item.category === 'Functional' ? (item.subType as FunctionalType) : undefined,
        nfrType: item.category === 'Non-Functional' ? (item.subType as NFRType) : undefined,
        title: item.label,
        description: item.description,
        priority: (['Critical','High','Medium','Low'].includes(item.priority) ? item.priority : 'Medium') as FormalRequirement['priority'],
        status: 'Draft',
        acceptanceCriteria: [],
        source: 'AI',
      });
      count++;
    }
    toast.success(`Imported ${count} requirements`);
    setImporting(false);
    onImported();
    onClose();
  };

  return (
    <div className="space-y-4">
      <p className="text-sm text-muted-foreground">
        Select AI-extracted items to import as formal requirements. Items already imported are not shown.
      </p>
      {candidates.length === 0 ? (
        <div className="text-center py-8 text-muted-foreground text-sm">
          No AI items available. Run "Analyze with AI" first.
        </div>
      ) : (
        <>
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">{candidates.length} items available</span>
            <Button size="sm" variant="ghost" className="h-7 text-xs"
              onClick={() => setSelected(selected.size === candidates.length ? new Set() : new Set(candidates.map(c => c.id)))}>
              {selected.size === candidates.length ? 'Deselect All' : 'Select All'}
            </Button>
          </div>
          <div className="space-y-2 max-h-80 overflow-y-auto">
            {candidates.map(item => (
              <div key={item.id}
                className={`p-3 rounded-lg border cursor-pointer transition-colors ${selected.has(item.id) ? 'border-primary bg-primary/5' : 'border-border hover:border-primary/50'}`}
                onClick={() => toggle(item.id)}>
                <div className="flex items-start gap-3">
                  <input type="checkbox" checked={selected.has(item.id)} onChange={() => toggle(item.id)}
                    className="mt-0.5 accent-primary" onClick={e => e.stopPropagation()} />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap mb-1">
                      <span className="text-sm font-medium line-clamp-1">{item.label}</span>
                      <span className={`text-xs px-1.5 py-0.5 rounded-full ${CATEGORY_COLORS[item.category]}`}>{item.category}</span>
                      <span className="text-xs px-1.5 py-0.5 rounded-full bg-gray-100 text-gray-700">{item.subType}</span>
                    </div>
                    <p className="text-xs text-muted-foreground line-clamp-2">{item.description}</p>
                    <span className="text-xs text-muted-foreground/60">Source: {item.source}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
          <div className="flex gap-2 pt-1">
            <Button onClick={handleImport} disabled={!selected.size || importing} className="flex-1">
              {importing ? 'Importing...' : `Import ${selected.size} Selected`}
            </Button>
            <Button variant="outline" onClick={onClose}>Cancel</Button>
          </div>
        </>
      )}
    </div>
  );
}

// ── Main Component ────────────────────────────────────────────────────────────

export function RequirementsTab({ roadmapItemId, projectId, requirements }: RequirementsTabProps) {
  const { createFormalRequirement, updateFormalRequirement, deleteFormalRequirement, currentHub } = useKnowledgeHubStore();

  const [filterCategory, setFilterCategory] = useState<string>('All');
  const [filterNFRType, setFilterNFRType] = useState<string>('All');
  const [filterStatus, setFilterStatus] = useState<string>('All');
  const [search, setSearch] = useState('');
  const [expandedId, setExpandedId] = useState<string | null>(null);

  const [isAddOpen, setIsAddOpen] = useState(false);
  const [editingReq, setEditingReq] = useState<FormalRequirement | null>(null);
  const [isImportOpen, setIsImportOpen] = useState(false);
  const [form, setForm] = useState<FormState>(defaultForm());
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Block dialog
  const [blockingReq, setBlockingReq] = useState<FormalRequirement | null>(null);
  const [blockForm, setBlockForm] = useState({ type: 'gap' as 'gap' | 'confusion' | 'conflict', id: '', title: '', reason: '' });

  const filtered = useMemo(() => {
    let list = requirements;
    if (filterCategory !== 'All') list = list.filter(r => r.category === filterCategory);
    if (filterNFRType !== 'All') list = list.filter(r => r.nfrType === filterNFRType);
    if (filterStatus !== 'All') list = list.filter(r => r.status === filterStatus);
    if (search) list = list.filter(r =>
      r.title.toLowerCase().includes(search.toLowerCase()) ||
      r.description.toLowerCase().includes(search.toLowerCase()) ||
      r.requirementId.toLowerCase().includes(search.toLowerCase())
    );
    return list;
  }, [requirements, filterCategory, filterNFRType, filterStatus, search]);

  const stats = useMemo(() => ({
    total: requirements.length,
    functional: requirements.filter(r => r.category === 'Functional').length,
    nfr: requirements.filter(r => r.category === 'Non-Functional').length,
    approved: requirements.filter(r => r.status === 'Approved').length,
    draft: requirements.filter(r => r.status === 'Draft').length,
    blocked: requirements.filter(r => r.status === 'Blocked').length,
  }), [requirements]);

  const openAdd = () => { setForm(defaultForm()); setIsAddOpen(true); };
  const openEdit = (r: FormalRequirement) => { setForm(fromRequirement(r)); setEditingReq(r); };

  const handleCreate = async () => {
    setIsSubmitting(true);
    const reqId = generateReqId(form.category, requirements);
    await createFormalRequirement({
      roadmapItemId, projectId, requirementId: reqId,
      category: form.category,
      functionalType: form.category === 'Functional' ? form.functionalType : undefined,
      nfrType: form.category === 'Non-Functional' ? form.nfrType : undefined,
      constraintType: form.category === 'Constraint' ? form.constraintType : undefined,
      title: form.title, description: form.description,
      priority: form.priority, status: form.status,
      userStory: form.asA ? { asA: form.asA, iWant: form.iWant, soThat: form.soThat } : undefined,
      acceptanceCriteria: form.acceptanceCriteria.split('\n').map(s => s.trim()).filter(Boolean),
      metric: form.metric || undefined,
      target: form.target || undefined,
      measurement: form.measurement || undefined,
      source: 'Manual',
    });
    setIsSubmitting(false);
    setIsAddOpen(false);
  };

  const handleUpdate = async () => {
    if (!editingReq) return;
    setIsSubmitting(true);
    await updateFormalRequirement(editingReq.id, {
      category: form.category,
      functionalType: form.category === 'Functional' ? form.functionalType : undefined,
      nfrType: form.category === 'Non-Functional' ? form.nfrType : undefined,
      constraintType: form.category === 'Constraint' ? form.constraintType : undefined,
      title: form.title, description: form.description,
      priority: form.priority, status: form.status,
      userStory: form.asA ? { asA: form.asA, iWant: form.iWant, soThat: form.soThat } : undefined,
      acceptanceCriteria: form.acceptanceCriteria.split('\n').map(s => s.trim()).filter(Boolean),
      metric: form.metric || undefined,
      target: form.target || undefined,
      measurement: form.measurement || undefined,
    });
    setIsSubmitting(false);
    setEditingReq(null);
  };

  const handleStatusChange = async (req: FormalRequirement, status: FormalRequirement['status']) => {
    await updateFormalRequirement(req.id, { status });
  };

  const handleDelete = async (id: string) => {
    if (confirm('Delete this requirement?')) await deleteFormalRequirement(id);
  };

  const handleBlock = async () => {
    if (!blockingReq || !blockForm.reason.trim()) return;
    setIsSubmitting(true);
    await updateFormalRequirement(blockingReq.id, {
      status: 'Blocked',
      blockedBy: {
        type: blockForm.type,
        id: blockForm.id || 'manual',
        title: blockForm.title || 'Manual block',
        reason: blockForm.reason,
        blockedAt: new Date().toISOString(),
      },
    });
    setIsSubmitting(false);
    setBlockingReq(null);
    toast.success('Requirement marked as blocked');
  };

  const handleUnblock = async (req: FormalRequirement) => {
    await updateFormalRequirement(req.id, { status: 'Draft', blockedBy: undefined });
    toast.success('Requirement unblocked');
  };

  const statusFlow: FormalRequirement['status'][] = ['Draft', 'Under Review', 'Approved', 'Implemented'];

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <ClipboardList className="w-5 h-5 text-primary" />
          <div>
            <h3 className="font-semibold">Formal Requirements</h3>
            <p className="text-xs text-muted-foreground">
              {stats.total} total · {stats.functional} functional · {stats.nfr} non-functional · {stats.approved} approved
            </p>
          </div>
        </div>
        <div className="flex gap-2">
          <Button size="sm" variant="outline" onClick={() => setIsImportOpen(true)}>
            <Sparkles className="w-4 h-4 mr-2" />
            Import from AI
          </Button>
          <Button size="sm" onClick={openAdd}>
            <Plus className="w-4 h-4 mr-2" />
            Add Requirement
          </Button>
        </div>
      </div>

      {/* Stats row */}
      {requirements.length > 0 && (
        <div className="grid grid-cols-6 gap-2">
          {[
            { label: 'Total', value: stats.total, color: 'text-foreground' },
            { label: 'Functional', value: stats.functional, color: 'text-blue-600' },
            { label: 'Non-Functional', value: stats.nfr, color: 'text-purple-600' },
            { label: 'Approved', value: stats.approved, color: 'text-green-600' },
            { label: 'Draft', value: stats.draft, color: 'text-gray-500' },
            { label: 'Blocked', value: stats.blocked, color: stats.blocked > 0 ? 'text-red-600' : 'text-gray-400' },
          ].map(s => (
            <Card key={s.label} className="text-center">
              <CardContent className="py-3 px-2">
                <div className={`text-xl font-bold ${s.color}`}>{s.value}</div>
                <div className="text-xs text-muted-foreground">{s.label}</div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Filters */}
      <div className="flex items-center gap-2 flex-wrap">
        <Filter className="w-4 h-4 text-muted-foreground shrink-0" />
        <div className="flex gap-1 flex-wrap">
          {['All', 'Functional', 'Non-Functional', 'Constraint', 'Acceptance Criteria'].map(cat => (
            <button key={cat}
              className={`text-xs px-3 py-1 rounded-full border transition-colors ${filterCategory === cat ? 'bg-primary text-primary-foreground border-primary' : 'bg-background border-border hover:border-primary/50'}`}
              onClick={() => { setFilterCategory(cat); if (cat !== 'Non-Functional') setFilterNFRType('All'); }}>
              {cat}
            </button>
          ))}
        </div>
        {filterCategory === 'Non-Functional' && (
          <div className="flex gap-1 flex-wrap ml-2 border-l pl-2">
            {['All', ...NFR_TYPES].map(t => (
              <button key={t}
                className={`text-xs px-2 py-0.5 rounded-full border transition-colors ${filterNFRType === t ? 'bg-purple-600 text-white border-purple-600' : 'bg-background border-border hover:border-purple-300'}`}
                onClick={() => setFilterNFRType(t)}>
                {t}
              </button>
            ))}
          </div>
        )}
        <div className="flex gap-1 ml-auto">
          {['All', 'Draft', 'Under Review', 'Approved', 'Implemented', 'Rejected', 'Blocked'].map(s => (
            <button key={s}
              className={`text-xs px-2 py-0.5 rounded-full border transition-colors ${filterStatus === s ? 'bg-secondary text-secondary-foreground border-secondary' : 'bg-background border-border hover:border-secondary/50'}`}
              onClick={() => setFilterStatus(s)}>
              {s}
            </button>
          ))}
        </div>
      </div>

      {/* Search */}
      <Input
        placeholder="Search by title, description, or ID..."
        value={search}
        onChange={e => setSearch(e.target.value)}
        className="h-8 text-sm"
      />

      {/* Empty state */}
      {requirements.length === 0 && (
        <Card>
          <CardContent className="py-16 text-center text-muted-foreground">
            <ClipboardList className="w-10 h-10 mx-auto mb-3 opacity-30" />
            <p className="font-medium">No formal requirements yet</p>
            <p className="text-sm mt-1">Add manually or import from the AI analysis results</p>
          </CardContent>
        </Card>
      )}

      {/* Requirements list */}
      <div className="space-y-2">
        {filtered.map(req => (
          <Card key={req.id} className={`border-l-4 ${req.status === 'Blocked' ? 'border-l-red-600 bg-red-50/30' : req.priority === 'Critical' ? 'border-l-red-500' : req.priority === 'High' ? 'border-l-orange-500' : req.priority === 'Medium' ? 'border-l-yellow-500' : 'border-l-green-500'}`}>
            <CardContent className="pt-3 pb-3">
              <div className="flex items-start justify-between gap-3">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 flex-wrap mb-1">
                    <span className="text-xs font-mono text-muted-foreground">{req.requirementId}</span>
                    <span className="font-medium text-sm">{req.title}</span>
                    <span className={`text-xs px-2 py-0.5 rounded-full font-medium ${CATEGORY_COLORS[req.category]}`}>
                      {req.nfrType ?? req.functionalType ?? req.constraintType ?? req.category}
                    </span>
                    <span className={`text-xs px-2 py-0.5 rounded-full border font-medium ${PRIORITY_COLORS[req.priority]}`}>
                      {req.priority}
                    </span>
                    <span className={`text-xs px-2 py-0.5 rounded-full ${STATUS_COLORS[req.status] ?? ''}`}>
                      {req.status === 'Blocked' && <Ban className="w-3 h-3 inline mr-0.5" />}
                      {req.status}
                    </span>
                    {req.source === 'AI' && <Badge variant="secondary" className="text-xs">AI</Badge>}
                  </div>
                  <p className="text-sm text-muted-foreground line-clamp-2">{req.description}</p>
                  {req.category === 'Non-Functional' && req.metric && (
                    <div className="flex gap-3 mt-1 text-xs text-muted-foreground">
                      <span>Metric: <strong className="text-foreground">{req.metric}</strong></span>
                      {req.target && <span>Target: <strong className="text-foreground">{req.target}</strong></span>}
                    </div>
                  )}
                  {/* Blocked-by banner */}
                  {req.status === 'Blocked' && req.blockedBy && (
                    <div className="mt-2 flex items-start gap-2 text-xs bg-red-100 border border-red-200 rounded px-2 py-1.5">
                      <Ban className="w-3.5 h-3.5 text-red-600 shrink-0 mt-0.5" />
                      <div>
                        <span className="font-medium text-red-800">Blocked by {req.blockedBy.type}:</span>{' '}
                        <span className="text-red-700">{req.blockedBy.title}</span>
                        {req.blockedBy.reason && <p className="mt-0.5 text-red-600 italic">{req.blockedBy.reason}</p>}
                      </div>
                    </div>
                  )}
                </div>

                <div className="flex items-center gap-1 shrink-0">
                  {/* Quick status advance (skip for Blocked/Rejected/Implemented) */}
                  {req.status !== 'Implemented' && req.status !== 'Rejected' && req.status !== 'Blocked' && (
                    <Button size="sm" variant="outline" className="h-7 text-xs"
                      onClick={() => {
                        const idx = statusFlow.indexOf(req.status);
                        if (idx < statusFlow.length - 1) handleStatusChange(req, statusFlow[idx + 1]);
                      }}>
                      <CheckCircle2 className="w-3 h-3 mr-1" />
                      {statusFlow[statusFlow.indexOf(req.status) + 1] ?? ''}
                    </Button>
                  )}
                  {/* Block / Unblock */}
                  {req.status !== 'Blocked' && req.status !== 'Implemented' && req.status !== 'Rejected' && (
                    <Button size="sm" variant="ghost" className="h-7 text-xs text-red-600 hover:text-red-700 hover:bg-red-50 px-2"
                      onClick={() => { setBlockingReq(req); setBlockForm({ type: 'gap', id: '', title: '', reason: '' }); }}>
                      <Ban className="w-3 h-3 mr-1" />Block
                    </Button>
                  )}
                  {req.status === 'Blocked' && (
                    <Button size="sm" variant="ghost" className="h-7 text-xs text-green-600 hover:text-green-700 hover:bg-green-50 px-2"
                      onClick={() => handleUnblock(req)}>
                      <Unlock className="w-3 h-3 mr-1" />Unblock
                    </Button>
                  )}
                  <Button size="sm" variant="ghost" className="h-7 w-7 p-0"
                    onClick={() => setExpandedId(expandedId === req.id ? null : req.id)}>
                    {expandedId === req.id ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                  </Button>
                  <Button size="sm" variant="ghost" className="h-7 w-7 p-0" onClick={() => openEdit(req)}>
                    <Pencil className="w-3.5 h-3.5" />
                  </Button>
                  <Button size="sm" variant="ghost" className="h-7 w-7 p-0 text-destructive hover:text-destructive"
                    onClick={() => handleDelete(req.id)}>
                    <Trash2 className="w-3.5 h-3.5" />
                  </Button>
                </div>
              </div>

              {/* Expanded details */}
              {expandedId === req.id && (
                <div className="mt-3 pt-3 border-t space-y-3">
                  {req.userStory?.asA && (
                    <div>
                      <p className="text-xs font-medium text-muted-foreground mb-1">User Story</p>
                      <div className="text-sm space-y-0.5">
                        <p><span className="text-muted-foreground">As a</span> {req.userStory.asA}</p>
                        <p><span className="text-muted-foreground">I want</span> {req.userStory.iWant}</p>
                        <p><span className="text-muted-foreground">So that</span> {req.userStory.soThat}</p>
                      </div>
                    </div>
                  )}
                  {req.acceptanceCriteria.length > 0 && (
                    <div>
                      <p className="text-xs font-medium text-muted-foreground mb-1">Acceptance Criteria</p>
                      <ul className="space-y-1">
                        {req.acceptanceCriteria.map((ac, i) => (
                          <li key={i} className="text-sm flex gap-2">
                            <span className="text-green-600 shrink-0">✓</span>{ac}
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                  {req.measurement && (
                    <div>
                      <p className="text-xs font-medium text-muted-foreground mb-1">How to Measure</p>
                      <p className="text-sm">{req.measurement}</p>
                    </div>
                  )}
                </div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>

      {/* No results from filter */}
      {requirements.length > 0 && filtered.length === 0 && (
        <div className="text-center py-8 text-muted-foreground text-sm">No requirements match the current filters.</div>
      )}

      {/* Add dialog */}
      <Dialog open={isAddOpen} onOpenChange={open => !open && setIsAddOpen(false)}>
        <DialogContent className="max-w-2xl">
          <DialogHeader><DialogTitle>Add Requirement</DialogTitle></DialogHeader>
          <RequirementForm form={form} setForm={setForm} onSubmit={handleCreate} onCancel={() => setIsAddOpen(false)} submitLabel="Add Requirement" isSubmitting={isSubmitting} />
        </DialogContent>
      </Dialog>

      {/* Edit dialog */}
      <Dialog open={!!editingReq} onOpenChange={open => !open && setEditingReq(null)}>
        <DialogContent className="max-w-2xl">
          <DialogHeader><DialogTitle>Edit — {editingReq?.requirementId}</DialogTitle></DialogHeader>
          <RequirementForm form={form} setForm={setForm} onSubmit={handleUpdate} onCancel={() => setEditingReq(null)} submitLabel="Save Changes" isSubmitting={isSubmitting} />
        </DialogContent>
      </Dialog>

      {/* AI Import dialog */}
      <Dialog open={isImportOpen} onOpenChange={open => !open && setIsImportOpen(false)}>
        <DialogContent className="max-w-2xl">
          <DialogHeader><DialogTitle>Import from AI Analysis</DialogTitle></DialogHeader>
          <AIImportDialog
            hub={currentHub}
            roadmapItemId={roadmapItemId}
            projectId={projectId}
            existingRequirements={requirements}
            onClose={() => setIsImportOpen(false)}
            onImported={() => {}}
          />
        </DialogContent>
      </Dialog>

      {/* Block dialog */}
      <Dialog open={!!blockingReq} onOpenChange={open => !open && setBlockingReq(null)}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Ban className="w-4 h-4 text-red-600" />
              Block Requirement — {blockingReq?.requirementId}
            </DialogTitle>
          </DialogHeader>
          <div className="space-y-4 pt-2">
            <p className="text-sm text-muted-foreground">
              Mark this requirement as blocked and link it to the blocker. The reason will appear prominently on the card.
            </p>
            <div className="space-y-1">
              <Label>Blocked by type</Label>
              <Select value={blockForm.type} onValueChange={v => setBlockForm(f => ({ ...f, type: v as 'gap' | 'confusion' | 'conflict' }))}>
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="gap">Gap</SelectItem>
                  <SelectItem value="confusion">Confusion</SelectItem>
                  <SelectItem value="conflict">Conflict</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1">
              <Label>Blocker title / reference <span className="text-muted-foreground">(optional)</span></Label>
              <Input value={blockForm.title} onChange={e => setBlockForm(f => ({ ...f, title: e.target.value }))}
                placeholder={`e.g. "Performance SLA gap – mobile API"`} />
            </div>
            <div className="space-y-1">
              <Label>Reason <span className="text-red-500">*</span></Label>
              <Textarea value={blockForm.reason} onChange={e => setBlockForm(f => ({ ...f, reason: e.target.value }))}
                rows={3} placeholder="Why is this requirement blocked? What needs to be resolved first?" />
            </div>
            <div className="flex gap-2 justify-end pt-1">
              <Button variant="outline" size="sm" onClick={() => setBlockingReq(null)}>Cancel</Button>
              <Button size="sm" variant="destructive" onClick={handleBlock}
                disabled={!blockForm.reason.trim() || isSubmitting}>
                <Ban className="w-3.5 h-3.5 mr-1" />
                {isSubmitting ? 'Saving...' : 'Mark as Blocked'}
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
