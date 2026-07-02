import React, { useState } from 'react';
import { Swords, Plus, Trash2, ChevronDown, ChevronUp } from 'lucide-react';
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
import type { Threat } from '../../lib/db/types';

interface ThreatsTabProps {
  roadmapItemId: string;
  threats: Threat[];
  onRefresh: () => void;
}

const strideColors: Record<string, string> = {
  Spoofing: 'bg-red-100 text-red-800',
  Tampering: 'bg-orange-100 text-orange-800',
  Repudiation: 'bg-yellow-100 text-yellow-800',
  'Information Disclosure': 'bg-purple-100 text-purple-800',
  'Denial of Service': 'bg-blue-100 text-blue-800',
  'Elevation of Privilege': 'bg-pink-100 text-pink-800',
};

const levelColors: Record<string, string> = {
  Critical: 'border-red-500',
  High: 'border-orange-500',
  Medium: 'border-yellow-500',
  Low: 'border-green-500',
};

const statusColors: Record<string, string> = {
  Identified: 'bg-gray-100 text-gray-700',
  Mitigated: 'bg-green-100 text-green-800',
  Accepted: 'bg-blue-100 text-blue-800',
  Transferred: 'bg-purple-100 text-purple-800',
};

export function ThreatsTab({ roadmapItemId, threats, onRefresh }: ThreatsTabProps) {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const [threat, setThreat] = useState('');
  const [description, setDescription] = useState('');
  const [category, setCategory] = useState<Threat['category']>('Spoofing');
  const [level, setLevel] = useState<Threat['level']>('High');
  const [likelihood, setLikelihood] = useState<Threat['likelihood']>('Medium');
  const [impact, setImpact] = useState('');
  const [mitigations, setMitigations] = useState('');

  const handleCreate = async () => {
    if (!threat || !description) return;
    setIsSubmitting(true);
    try {
      await db.threats.create({
        roadmapItemId,
        threat,
        description,
        category,
        level,
        likelihood,
        impact,
        mitigations: mitigations.split('\n').map(m => m.trim()).filter(Boolean),
        status: 'Identified',
      });
      toast.success('Threat added');
      setThreat(''); setDescription(''); setImpact(''); setMitigations('');
      setIsDialogOpen(false);
      onRefresh();
    } catch {
      toast.error('Failed to add threat');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleStatusChange = async (t: Threat, status: Threat['status']) => {
    try {
      await db.threats.update(t.id, { status });
      toast.success(`Status updated`);
      onRefresh();
    } catch {
      toast.error('Failed to update');
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await db.threats.delete(id);
      toast.success('Deleted');
      onRefresh();
    } catch {
      toast.error('Failed to delete');
    }
  };

  const criticalCount = threats.filter(t => t.level === 'Critical' || t.level === 'High').length;

  const strideGroups = ['Spoofing', 'Tampering', 'Repudiation', 'Information Disclosure', 'Denial of Service', 'Elevation of Privilege'];
  const groupCounts = strideGroups.map(cat => threats.filter(t => t.category === cat).length);

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Swords className="w-5 h-5 text-red-600" />
          <div>
            <h3 className="font-semibold">Threat Analysis (STRIDE)</h3>
            <p className="text-xs text-muted-foreground">
              {threats.length} total · {criticalCount} critical/high
            </p>
          </div>
        </div>
        <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
          <DialogTrigger asChild>
            <Button size="sm"><Plus className="w-4 h-4 mr-2" />Add Threat</Button>
          </DialogTrigger>
          <DialogContent className="max-w-lg">
            <DialogHeader><DialogTitle>Add Threat</DialogTitle></DialogHeader>
            <div className="space-y-4 pt-2">
              <div className="space-y-1">
                <Label>Threat Name *</Label>
                <Input value={threat} onChange={e => setThreat(e.target.value)} placeholder="e.g. Session token theft via XSS" />
              </div>
              <div className="space-y-1">
                <Label>Description *</Label>
                <Textarea value={description} onChange={e => setDescription(e.target.value)} rows={2} />
              </div>
              <div className="grid grid-cols-3 gap-3">
                <div className="space-y-1">
                  <Label>STRIDE Category</Label>
                  <Select value={category} onValueChange={v => setCategory(v as Threat['category'])}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      {['Spoofing', 'Tampering', 'Repudiation', 'Information Disclosure', 'Denial of Service', 'Elevation of Privilege'].map(c => <SelectItem key={c} value={c}>{c}</SelectItem>)}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-1">
                  <Label>Level</Label>
                  <Select value={level} onValueChange={v => setLevel(v as Threat['level'])}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      {['Critical', 'High', 'Medium', 'Low'].map(l => <SelectItem key={l} value={l}>{l}</SelectItem>)}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-1">
                  <Label>Likelihood</Label>
                  <Select value={likelihood} onValueChange={v => setLikelihood(v as Threat['likelihood'])}>
                    <SelectTrigger><SelectValue /></SelectTrigger>
                    <SelectContent>
                      {['Very High', 'High', 'Medium', 'Low', 'Very Low'].map(l => <SelectItem key={l} value={l}>{l}</SelectItem>)}
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <div className="space-y-1">
                <Label>Impact</Label>
                <Input value={impact} onChange={e => setImpact(e.target.value)} placeholder="e.g. Unauthorized access to user data" />
              </div>
              <div className="space-y-1">
                <Label>Mitigations (one per line)</Label>
                <Textarea value={mitigations} onChange={e => setMitigations(e.target.value)} rows={3} placeholder="e.g. Implement CSP headers..." />
              </div>
              <div className="flex justify-end gap-2">
                <Button variant="outline" onClick={() => setIsDialogOpen(false)}>Cancel</Button>
                <Button onClick={handleCreate} disabled={!threat || !description || isSubmitting}>
                  {isSubmitting ? 'Adding...' : 'Add'}
                </Button>
              </div>
            </div>
          </DialogContent>
        </Dialog>
      </div>

      {/* STRIDE summary bar */}
      {threats.length > 0 && (
        <div className="grid grid-cols-6 gap-2">
          {strideGroups.map((cat, i) => (
            <div key={cat} className="text-center p-2 rounded-lg bg-muted/50">
              <div className="text-lg font-bold">{groupCounts[i]}</div>
              <div className="text-xs text-muted-foreground leading-tight">{cat.split(' ')[0]}</div>
            </div>
          ))}
        </div>
      )}

      {threats.length === 0 && (
        <Card>
          <CardContent className="py-16 text-center text-muted-foreground">
            <Swords className="w-10 h-10 mx-auto mb-3 opacity-30" />
            <p className="font-medium">No threats identified</p>
            <p className="text-sm mt-1">Run AI analysis or add manually</p>
          </CardContent>
        </Card>
      )}

      <div className="space-y-3">
        {threats.map(t => (
          <Card key={t.id} className={`border-l-4 ${levelColors[t.level]}`}>
            <CardContent className="pt-4 pb-3">
              <div className="flex items-start justify-between gap-3">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className="font-medium text-sm">{t.threat}</span>
                    <span className={`text-xs px-2 py-0.5 rounded-full font-medium ${strideColors[t.category]}`}>{t.category}</span>
                    <span className="text-xs px-2 py-0.5 rounded-full bg-gray-100 text-gray-700">{t.level}</span>
                    <span className="text-xs text-muted-foreground">Likelihood: {t.likelihood}</span>
                    <span className={`text-xs px-2 py-0.5 rounded-full ${statusColors[t.status]}`}>{t.status}</span>
                  </div>
                  <p className="text-sm text-muted-foreground mt-1 line-clamp-2">{t.description}</p>
                </div>
                <div className="flex items-center gap-1 shrink-0">
                  {t.status === 'Identified' && (
                    <Button size="sm" variant="outline" className="h-7 text-xs" onClick={() => handleStatusChange(t, 'Mitigated')}>
                      Mark Mitigated
                    </Button>
                  )}
                  <Button size="sm" variant="ghost" className="h-7 w-7 p-0" onClick={() => setExpandedId(expandedId === t.id ? null : t.id)}>
                    {expandedId === t.id ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                  </Button>
                  <Button size="sm" variant="ghost" className="h-7 w-7 p-0 text-destructive hover:text-destructive" onClick={() => handleDelete(t.id)}>
                    <Trash2 className="w-3.5 h-3.5" />
                  </Button>
                </div>
              </div>

              {expandedId === t.id && (
                <div className="mt-3 pt-3 border-t space-y-2">
                  {t.impact && (
                    <div>
                      <p className="text-xs font-medium text-muted-foreground mb-1">Impact</p>
                      <p className="text-sm">{t.impact}</p>
                    </div>
                  )}
                  {t.mitigations && t.mitigations.length > 0 && (
                    <div>
                      <p className="text-xs font-medium text-muted-foreground mb-1">Mitigations</p>
                      <ul className="space-y-1">
                        {t.mitigations.map((m, i) => <li key={i} className="text-sm flex gap-2"><span className="text-green-600">→</span>{m}</li>)}
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
