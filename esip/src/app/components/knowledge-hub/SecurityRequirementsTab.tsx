import React, { useState } from 'react';
import { Shield, Plus, Trash2, ChevronDown, ChevronUp } from 'lucide-react';
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
import type { SecurityRequirement } from '../../lib/db/types';

interface SecurityRequirementsTabProps {
  roadmapItemId: string;
  requirements: SecurityRequirement[];
  onRefresh: () => void;
}

const levelColors: Record<string, string> = {
  Critical: 'bg-red-100 text-red-800 border-red-200',
  High: 'bg-orange-100 text-orange-800 border-orange-200',
  Medium: 'bg-yellow-100 text-yellow-800 border-yellow-200',
  Low: 'bg-green-100 text-green-800 border-green-200',
};

const categoryIcons: Record<string, string> = {
  Authentication: '🔐',
  Authorization: '🔑',
  'Data Protection': '🛡️',
  'Network Security': '🌐',
  Compliance: '📋',
  Other: '⚙️',
};

const statusColors: Record<string, string> = {
  Draft: 'bg-gray-100 text-gray-700',
  Reviewed: 'bg-yellow-100 text-yellow-800',
  Approved: 'bg-green-100 text-green-800',
};

export function SecurityRequirementsTab({ roadmapItemId, requirements, onRefresh }: SecurityRequirementsTabProps) {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [category, setCategory] = useState<SecurityRequirement['category']>('Authentication');
  const [level, setLevel] = useState<SecurityRequirement['level']>('High');
  const [controls, setControls] = useState('');
  const [testCriteria, setTestCriteria] = useState('');

  const handleCreate = async () => {
    if (!title || !description) return;
    setIsSubmitting(true);
    try {
      await db.securityRequirements.create({
        roadmapItemId,
        title,
        description,
        category,
        level,
        controls: controls.split('\n').map(c => c.trim()).filter(Boolean),
        testCriteria: testCriteria.split('\n').map(t => t.trim()).filter(Boolean),
        status: 'Draft',
      });
      toast.success('Security requirement added');
      setTitle(''); setDescription(''); setControls(''); setTestCriteria('');
      setIsDialogOpen(false);
      onRefresh();
    } catch {
      toast.error('Failed to add');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleStatusChange = async (req: SecurityRequirement, status: SecurityRequirement['status']) => {
    try {
      await db.securityRequirements.update(req.id, { status });
      toast.success(`Status updated to ${status}`);
      onRefresh();
    } catch {
      toast.error('Failed to update');
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await db.securityRequirements.delete(id);
      toast.success('Deleted');
      onRefresh();
    } catch {
      toast.error('Failed to delete');
    }
  };

  const criticalCount = requirements.filter(r => r.level === 'Critical' || r.level === 'High').length;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Shield className="w-5 h-5 text-red-500" />
          <div>
            <h3 className="font-semibold">Security Requirements</h3>
            <p className="text-xs text-muted-foreground">
              {requirements.length} total · {criticalCount} critical/high
            </p>
          </div>
        </div>
        <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
          <DialogTrigger asChild>
            <Button size="sm"><Plus className="w-4 h-4 mr-2" />Add Requirement</Button>
          </DialogTrigger>
          <DialogContent className="max-w-lg">
            <DialogHeader><DialogTitle>Add Security Requirement</DialogTitle></DialogHeader>
            <div className="space-y-4 pt-2">
              <div className="space-y-1">
                <Label>Title *</Label>
                <Input value={title} onChange={e => setTitle(e.target.value)} placeholder="e.g. Multi-factor authentication required" />
              </div>
              <div className="space-y-1">
                <Label>Description *</Label>
                <Textarea value={description} onChange={e => setDescription(e.target.value)} rows={3} />
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1">
                  <Label>Category</Label>
                  <Select value={category} onValueChange={v => setCategory(v as SecurityRequirement['category'])}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      {['Authentication', 'Authorization', 'Data Protection', 'Network Security', 'Compliance', 'Other'].map(c => <SelectItem key={c} value={c}>{c}</SelectItem>)}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-1">
                  <Label>Level</Label>
                  <Select value={level} onValueChange={v => setLevel(v as SecurityRequirement['level'])}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      {['Critical', 'High', 'Medium', 'Low'].map(l => <SelectItem key={l} value={l}>{l}</SelectItem>)}
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <div className="space-y-1">
                <Label>Controls (one per line)</Label>
                <Textarea value={controls} onChange={e => setControls(e.target.value)} rows={2} placeholder="e.g. Implement TOTP, Enforce session timeout" />
              </div>
              <div className="space-y-1">
                <Label>Test Criteria (one per line)</Label>
                <Textarea value={testCriteria} onChange={e => setTestCriteria(e.target.value)} rows={2} placeholder="e.g. MFA prompt appears on login" />
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

      {requirements.length === 0 && (
        <Card>
          <CardContent className="py-16 text-center text-muted-foreground">
            <Shield className="w-10 h-10 mx-auto mb-3 opacity-30" />
            <p className="font-medium">No security requirements yet</p>
            <p className="text-sm mt-1">Run AI analysis or add manually</p>
          </CardContent>
        </Card>
      )}

      <div className="space-y-3">
        {requirements.map(req => (
          <Card key={req.id} className={`border-l-4 ${req.level === 'Critical' ? 'border-l-red-500' : req.level === 'High' ? 'border-l-orange-500' : req.level === 'Medium' ? 'border-l-yellow-500' : 'border-l-green-500'}`}>
            <CardContent className="pt-4 pb-3">
              <div className="flex items-start justify-between gap-3">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className="text-base">{categoryIcons[req.category]}</span>
                    <span className="font-medium text-sm">{req.title}</span>
                    <span className={`text-xs px-2 py-0.5 rounded-full font-medium border ${levelColors[req.level]}`}>{req.level}</span>
                    <span className="text-xs px-2 py-0.5 rounded-full bg-blue-50 text-blue-700">{req.category}</span>
                    <span className={`text-xs px-2 py-0.5 rounded-full ${statusColors[req.status]}`}>{req.status}</span>
                  </div>
                  <p className="text-sm text-muted-foreground mt-1 line-clamp-2">{req.description}</p>
                </div>
                <div className="flex items-center gap-1 shrink-0">
                  {req.status === 'Draft' && (
                    <Button size="sm" variant="outline" className="h-7 text-xs" onClick={() => handleStatusChange(req, 'Reviewed')}>
                      Mark Reviewed
                    </Button>
                  )}
                  {req.status === 'Reviewed' && (
                    <Button size="sm" variant="outline" className="h-7 text-xs text-green-700 border-green-300" onClick={() => handleStatusChange(req, 'Approved')}>
                      Approve
                    </Button>
                  )}
                  <Button size="sm" variant="ghost" className="h-7 w-7 p-0" onClick={() => setExpandedId(expandedId === req.id ? null : req.id)}>
                    {expandedId === req.id ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                  </Button>
                  <Button size="sm" variant="ghost" className="h-7 w-7 p-0 text-destructive hover:text-destructive" onClick={() => handleDelete(req.id)}>
                    <Trash2 className="w-3.5 h-3.5" />
                  </Button>
                </div>
              </div>

              {expandedId === req.id && (
                <div className="mt-3 pt-3 border-t grid grid-cols-2 gap-4">
                  {req.controls && req.controls.length > 0 && (
                    <div>
                      <p className="text-xs font-medium text-muted-foreground mb-1">Controls</p>
                      <ul className="space-y-1">
                        {req.controls.map((c, i) => <li key={i} className="text-sm flex gap-2"><span className="text-primary">•</span>{c}</li>)}
                      </ul>
                    </div>
                  )}
                  {req.testCriteria && req.testCriteria.length > 0 && (
                    <div>
                      <p className="text-xs font-medium text-muted-foreground mb-1">Test Criteria</p>
                      <ul className="space-y-1">
                        {req.testCriteria.map((t, i) => <li key={i} className="text-sm flex gap-2"><span className="text-green-600">✓</span>{t}</li>)}
                      </ul>
                    </div>
                  )}
                </div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
