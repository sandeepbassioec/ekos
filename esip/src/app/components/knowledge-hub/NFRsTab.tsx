import React, { useState } from 'react';
import { Gauge, Plus, Trash2, ChevronDown, ChevronUp } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
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
import type { NonFunctionalRequirement } from '../../lib/db/types';

interface NFRsTabProps {
  roadmapItemId: string;
  nfrs: NonFunctionalRequirement[];
  onRefresh: () => void;
}

const categoryColors: Record<string, string> = {
  Performance: 'bg-blue-100 text-blue-800',
  Scalability: 'bg-purple-100 text-purple-800',
  Reliability: 'bg-green-100 text-green-800',
  Availability: 'bg-emerald-100 text-emerald-800',
  Maintainability: 'bg-yellow-100 text-yellow-800',
  Usability: 'bg-pink-100 text-pink-800',
  Compliance: 'bg-orange-100 text-orange-800',
};

const priorityColors: Record<string, string> = {
  Critical: 'border-l-red-500',
  High: 'border-l-orange-500',
  Medium: 'border-l-yellow-500',
  Low: 'border-l-green-500',
};

const statusColors: Record<string, string> = {
  Draft: 'bg-gray-100 text-gray-700',
  Approved: 'bg-green-100 text-green-800',
  Met: 'bg-blue-100 text-blue-800',
  'Not Met': 'bg-red-100 text-red-800',
};

export function NFRsTab({ roadmapItemId, nfrs, onRefresh }: NFRsTabProps) {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [category, setCategory] = useState<NonFunctionalRequirement['category']>('Performance');
  const [requirement, setRequirement] = useState('');
  const [metric, setMetric] = useState('');
  const [target, setTarget] = useState('');
  const [measurement, setMeasurement] = useState('');
  const [priority, setPriority] = useState<NonFunctionalRequirement['priority']>('High');

  const handleCreate = async () => {
    if (!requirement || !metric) return;
    setIsSubmitting(true);
    try {
      await db.nfrs.create({
        roadmapItemId,
        category,
        requirement,
        metric,
        target,
        measurement,
        priority,
        status: 'Draft',
      });
      toast.success('NFR added');
      setRequirement(''); setMetric(''); setTarget(''); setMeasurement('');
      setIsDialogOpen(false);
      onRefresh();
    } catch {
      toast.error('Failed to add NFR');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleStatusChange = async (nfr: NonFunctionalRequirement, status: NonFunctionalRequirement['status']) => {
    try {
      await db.nfrs.update(nfr.id, { status });
      toast.success('Status updated');
      onRefresh();
    } catch {
      toast.error('Failed to update');
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await db.nfrs.delete(id);
      toast.success('Deleted');
      onRefresh();
    } catch {
      toast.error('Failed to delete');
    }
  };

  // Group by category for display
  const categories = ['Performance', 'Scalability', 'Reliability', 'Availability', 'Maintainability', 'Usability', 'Compliance'];
  const grouped = categories.map(cat => ({
    cat,
    items: nfrs.filter(n => n.category === cat),
  })).filter(g => g.items.length > 0);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Gauge className="w-5 h-5 text-purple-500" />
          <div>
            <h3 className="font-semibold">Non-Functional Requirements</h3>
            <p className="text-xs text-muted-foreground">
              {nfrs.length} total across {grouped.length} categories
            </p>
          </div>
        </div>
        <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
          <DialogTrigger asChild>
            <Button size="sm"><Plus className="w-4 h-4 mr-2" />Add NFR</Button>
          </DialogTrigger>
          <DialogContent className="max-w-lg">
            <DialogHeader><DialogTitle>Add Non-Functional Requirement</DialogTitle></DialogHeader>
            <div className="space-y-4 pt-2">
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <Label>Category</Label>
                  <Select value={category} onValueChange={v => setCategory(v as NonFunctionalRequirement['category'])}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      {categories.map(c => <SelectItem key={c} value={c}>{c}</SelectItem>)}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-1">
                  <Label>Priority</Label>
                  <Select value={priority} onValueChange={v => setPriority(v as NonFunctionalRequirement['priority'])}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      {['Critical', 'High', 'Medium', 'Low'].map(p => <SelectItem key={p} value={p}>{p}</SelectItem>)}
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <div className="space-y-1">
                <Label>Requirement *</Label>
                <Textarea value={requirement} onChange={e => setRequirement(e.target.value)} rows={2} placeholder="e.g. API response time must be under 200ms for 95th percentile" />
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <Label>Metric *</Label>
                  <Input value={metric} onChange={e => setMetric(e.target.value)} placeholder="e.g. Response time (p95)" />
                </div>
                <div className="space-y-1">
                  <Label>Target</Label>
                  <Input value={target} onChange={e => setTarget(e.target.value)} placeholder="e.g. &lt; 200ms" />
                </div>
              </div>
              <div className="space-y-1">
                <Label>How to Measure</Label>
                <Input value={measurement} onChange={e => setMeasurement(e.target.value)} placeholder="e.g. APM tool (Datadog / New Relic)" />
              </div>
              <div className="flex justify-end gap-2">
                <Button variant="outline" onClick={() => setIsDialogOpen(false)}>Cancel</Button>
                <Button onClick={handleCreate} disabled={!requirement || !metric || isSubmitting}>
                  {isSubmitting ? 'Adding...' : 'Add'}
                </Button>
              </div>
            </div>
          </DialogContent>
        </Dialog>
      </div>

      {nfrs.length === 0 && (
        <Card>
          <CardContent className="py-16 text-center text-muted-foreground">
            <Gauge className="w-10 h-10 mx-auto mb-3 opacity-30" />
            <p className="font-medium">No NFRs yet</p>
            <p className="text-sm mt-1">Run AI analysis or add manually</p>
          </CardContent>
        </Card>
      )}

      {grouped.map(({ cat, items }) => (
        <div key={cat}>
          <div className="flex items-center gap-2 mb-2">
            <span className={`text-xs px-2 py-0.5 rounded-full font-medium ${categoryColors[cat]}`}>{cat}</span>
            <span className="text-xs text-muted-foreground">{items.length} requirements</span>
          </div>
          <div className="space-y-2">
            {items.map(nfr => (
              <Card key={nfr.id} className={`border-l-4 ${priorityColors[nfr.priority]}`}>
                <CardContent className="pt-3 pb-3">
                  <div className="flex items-start justify-between gap-3">
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium line-clamp-2">{nfr.requirement}</p>
                      <div className="flex items-center gap-3 mt-1 flex-wrap">
                        <span className="text-xs text-muted-foreground">Metric: <span className="font-medium text-foreground">{nfr.metric}</span></span>
                        {nfr.target && <span className="text-xs text-muted-foreground">Target: <span className="font-medium text-foreground">{nfr.target}</span></span>}
                        <span className={`text-xs px-2 py-0.5 rounded-full ${statusColors[nfr.status]}`}>{nfr.status}</span>
                      </div>
                      {nfr.measurement && expandedId === nfr.id && (
                        <p className="text-xs text-muted-foreground mt-1">Measurement: {nfr.measurement}</p>
                      )}
                    </div>
                    <div className="flex items-center gap-1 shrink-0">
                      {nfr.status === 'Draft' && (
                        <Button size="sm" variant="outline" className="h-7 text-xs" onClick={() => handleStatusChange(nfr, 'Approved')}>Approve</Button>
                      )}
                      {nfr.status === 'Approved' && (
                        <Button size="sm" variant="outline" className="h-7 text-xs text-green-700 border-green-300" onClick={() => handleStatusChange(nfr, 'Met')}>Mark Met</Button>
                      )}
                      {nfr.measurement && (
                        <Button size="sm" variant="ghost" className="h-7 w-7 p-0" onClick={() => setExpandedId(expandedId === nfr.id ? null : nfr.id)}>
                          {expandedId === nfr.id ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                        </Button>
                      )}
                      <Button size="sm" variant="ghost" className="h-7 w-7 p-0 text-destructive hover:text-destructive" onClick={() => handleDelete(nfr.id)}>
                        <Trash2 className="w-3.5 h-3.5" />
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}
