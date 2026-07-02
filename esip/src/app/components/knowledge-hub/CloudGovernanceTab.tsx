import React, { useState } from 'react';
import { Cloud, Plus, Trash2 } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
import { Textarea } from '../ui/textarea';
import { Label } from '../ui/label';
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger,
} from '../ui/dialog';
import {
  Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '../ui/select';
import { db } from '../../lib/db';
import { toast } from 'sonner';
import type { CloudGovernance } from '../../lib/db/types';

interface CloudGovernanceTabProps {
  roadmapItemId: string;
  items: CloudGovernance[];
  onRefresh: () => void;
}

const areaColors: Record<string, string> = {
  'Cost Management': 'bg-green-100 text-green-800',
  'Resource Governance': 'bg-blue-100 text-blue-800',
  'Cloud Security': 'bg-red-100 text-red-800',
  Compliance: 'bg-orange-100 text-orange-800',
  'Data Residency': 'bg-purple-100 text-purple-800',
  'Disaster Recovery': 'bg-yellow-100 text-yellow-800',
  Networking: 'bg-cyan-100 text-cyan-800',
};

const areaIcons: Record<string, string> = {
  'Cost Management': '💰',
  'Resource Governance': '⚙️',
  'Cloud Security': '🔒',
  Compliance: '📋',
  'Data Residency': '🌍',
  'Disaster Recovery': '🔄',
  Networking: '🌐',
};

const priorityBorder: Record<string, string> = {
  Critical: 'border-l-red-500',
  High: 'border-l-orange-500',
  Medium: 'border-l-yellow-500',
  Low: 'border-l-green-500',
};

const statusColors: Record<string, string> = {
  Draft: 'bg-gray-100 text-gray-700',
  Approved: 'bg-green-100 text-green-800',
  Implemented: 'bg-blue-100 text-blue-800',
};

const areas: CloudGovernance['area'][] = [
  'Cost Management', 'Resource Governance', 'Cloud Security',
  'Compliance', 'Data Residency', 'Disaster Recovery', 'Networking',
];

export function CloudGovernanceTab({ roadmapItemId, items, onRefresh }: CloudGovernanceTabProps) {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [area, setArea] = useState<CloudGovernance['area']>('Cloud Security');
  const [requirement, setRequirement] = useState('');
  const [details, setDetails] = useState('');
  const [priority, setPriority] = useState<CloudGovernance['priority']>('High');

  const handleCreate = async () => {
    if (!requirement || !details) return;
    setIsSubmitting(true);
    try {
      await db.cloudGovernance.create({
        roadmapItemId,
        area,
        requirement,
        details,
        priority,
        status: 'Draft',
      });
      toast.success('Cloud governance requirement added');
      setRequirement(''); setDetails('');
      setIsDialogOpen(false);
      onRefresh();
    } catch {
      toast.error('Failed to add');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleStatusChange = async (item: CloudGovernance, status: CloudGovernance['status']) => {
    try {
      await db.cloudGovernance.update(item.id, { status });
      toast.success('Status updated');
      onRefresh();
    } catch {
      toast.error('Failed to update');
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await db.cloudGovernance.delete(id);
      toast.success('Deleted');
      onRefresh();
    } catch {
      toast.error('Failed to delete');
    }
  };

  // Group by area
  const grouped = areas.map(a => ({
    area: a,
    items: items.filter(i => i.area === a),
  })).filter(g => g.items.length > 0);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Cloud className="w-5 h-5 text-cyan-500" />
          <div>
            <h3 className="font-semibold">Cloud Governance</h3>
            <p className="text-xs text-muted-foreground">
              {items.length} total · {grouped.length} areas covered
            </p>
          </div>
        </div>
        <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
          <DialogTrigger asChild>
            <Button size="sm"><Plus className="w-4 h-4 mr-2" />Add Requirement</Button>
          </DialogTrigger>
          <DialogContent className="max-w-lg">
            <DialogHeader><DialogTitle>Add Cloud Governance Requirement</DialogTitle></DialogHeader>
            <div className="space-y-4 pt-2">
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <Label>Area</Label>
                  <Select value={area} onValueChange={v => setArea(v as CloudGovernance['area'])}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      {areas.map(a => <SelectItem key={a} value={a}>{a}</SelectItem>)}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-1">
                  <Label>Priority</Label>
                  <Select value={priority} onValueChange={v => setPriority(v as CloudGovernance['priority'])}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      {['Critical', 'High', 'Medium', 'Low'].map(p => <SelectItem key={p} value={p}>{p}</SelectItem>)}
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <div className="space-y-1">
                <Label>Requirement *</Label>
                <Textarea value={requirement} onChange={e => setRequirement(e.target.value)} rows={2} placeholder="e.g. All data must be stored in ap-south-1 region only" />
              </div>
              <div className="space-y-1">
                <Label>Details *</Label>
                <Textarea value={details} onChange={e => setDetails(e.target.value)} rows={3} placeholder="Detailed explanation and implementation notes..." />
              </div>
              <div className="flex justify-end gap-2">
                <Button variant="outline" onClick={() => setIsDialogOpen(false)}>Cancel</Button>
                <Button onClick={handleCreate} disabled={!requirement || !details || isSubmitting}>
                  {isSubmitting ? 'Adding...' : 'Add'}
                </Button>
              </div>
            </div>
          </DialogContent>
        </Dialog>
      </div>

      {items.length === 0 && (
        <Card>
          <CardContent className="py-16 text-center text-muted-foreground">
            <Cloud className="w-10 h-10 mx-auto mb-3 opacity-30" />
            <p className="font-medium">No cloud governance requirements yet</p>
            <p className="text-sm mt-1">Run AI analysis or add manually</p>
          </CardContent>
        </Card>
      )}

      {grouped.map(({ area: groupArea, items: groupItems }) => (
        <div key={groupArea}>
          <div className="flex items-center gap-2 mb-2">
            <span className="text-base">{areaIcons[groupArea]}</span>
            <span className={`text-xs px-2 py-0.5 rounded-full font-medium ${areaColors[groupArea]}`}>{groupArea}</span>
            <span className="text-xs text-muted-foreground">{groupItems.length} item{groupItems.length > 1 ? 's' : ''}</span>
          </div>
          <div className="space-y-2">
            {groupItems.map(item => (
              <Card key={item.id} className={`border-l-4 ${priorityBorder[item.priority]}`}>
                <CardContent className="pt-3 pb-3">
                  <div className="flex items-start justify-between gap-3">
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium">{item.requirement}</p>
                      <p className="text-sm text-muted-foreground mt-1 line-clamp-2">{item.details}</p>
                      <div className="flex items-center gap-2 mt-1">
                        <span className="text-xs text-muted-foreground">{item.priority} priority</span>
                        <span className={`text-xs px-2 py-0.5 rounded-full ${statusColors[item.status]}`}>{item.status}</span>
                      </div>
                    </div>
                    <div className="flex items-center gap-1 shrink-0">
                      {item.status === 'Draft' && (
                        <Button size="sm" variant="outline" className="h-7 text-xs" onClick={() => handleStatusChange(item, 'Approved')}>Approve</Button>
                      )}
                      {item.status === 'Approved' && (
                        <Button size="sm" variant="outline" className="h-7 text-xs text-blue-700 border-blue-300" onClick={() => handleStatusChange(item, 'Implemented')}>
                          Mark Implemented
                        </Button>
                      )}
                      <Button size="sm" variant="ghost" className="h-7 w-7 p-0 text-destructive hover:text-destructive" onClick={() => handleDelete(item.id)}>
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
