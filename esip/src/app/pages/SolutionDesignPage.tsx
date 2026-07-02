import { useEffect, useMemo, useState } from 'react';
import { useNavigate, useParams } from 'react-router';
import { ArrowLeft, Plus, Scale, Trash2, Save, Star, Trophy } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { Input } from '../components/ui/input';
import {
  useSolutionStore,
  rankOptions,
  recommendedOptionId,
  newId,
  type TradeoffAnalysis,
} from '../store/solutionStore';
import type { TradeoffStatus } from '../lib/db/types';
import { toast } from 'sonner';

/**
 * Solution Design — Trade-off / Decision Analysis (Epic E / catalog Vol 04 SA-03/04).
 * The Solution Architect's signature workspace: compare candidate options against weighted
 * criteria, capture pros/cons, and land a recommendation (computed or manually pinned).
 * DAL-backed via solutionStore. AI-assisted scoring + ADR/HLD generation come later (Vol 21/04).
 */

const STATUSES: { value: TradeoffStatus; label: string }[] = [
  { value: 'draft', label: 'Draft' },
  { value: 'in-review', label: 'In review' },
  { value: 'decided', label: 'Decided' },
];

const STATUS_VARIANT: Record<TradeoffStatus, 'secondary' | 'default' | 'outline'> = {
  draft: 'secondary',
  'in-review': 'default',
  decided: 'outline',
};

const clone = (a: TradeoffAnalysis): TradeoffAnalysis => JSON.parse(JSON.stringify(a));
const linesToArr = (s: string) => s.split('\n').map((x) => x.trim()).filter(Boolean);

export default function SolutionDesignPage() {
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();
  const { analyses, isLoading, fetchByProject, createAnalysis, updateAnalysis, deleteAnalysis } = useSolutionStore();

  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [draft, setDraft] = useState<TradeoffAnalysis | null>(null);
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (projectId) fetchByProject(projectId);
  }, [projectId, fetchByProject]);

  // Load the selected analysis into an editable working copy.
  useEffect(() => {
    const found = analyses.find((a) => a.id === selectedId);
    setDraft(found ? clone(found) : null);
    setDirty(false);
  }, [selectedId]); // eslint-disable-line react-hooks/exhaustive-deps

  const ranking = useMemo(() => (draft ? rankOptions(draft) : []), [draft]);
  const recId = draft ? recommendedOptionId(draft) : undefined;
  const rankFor = (oid: string) => ranking.find((r) => r.optionId === oid);

  // ── draft mutators ──────────────────────────────────────────────────────────
  const patch = (updater: (d: TradeoffAnalysis) => void) =>
    setDraft((prev) => {
      if (!prev) return prev;
      const next = clone(prev);
      updater(next);
      return next;
    });
  const touch = (updater: (d: TradeoffAnalysis) => void) => {
    patch(updater);
    setDirty(true);
  };

  const handleCreate = async () => {
    if (!projectId) return;
    const created = await createAnalysis(projectId, `Analysis ${analyses.length + 1}`);
    if (created) setSelectedId(created.id);
  };

  const handleSave = async () => {
    if (!draft) return;
    setSaving(true);
    try {
      const { id, criteria, options, title, description, recommendedOptionId: rec, decisionRationale, status } = draft;
      await updateAnalysis(id, { criteria, options, title, description, recommendedOptionId: rec, decisionRationale, status });
      setDirty(false);
      toast.success('Analysis saved');
    } catch {
      toast.error('Failed to save analysis');
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this analysis?')) return;
    await deleteAnalysis(id);
    if (selectedId === id) setSelectedId(null);
  };

  return (
    <div className="container max-w-6xl mx-auto py-8 px-4">
      {/* Header */}
      <Button variant="ghost" className="-ml-2 mb-3" onClick={() => navigate(`/projects/${projectId}`)}>
        <ArrowLeft className="w-4 h-4 mr-2" /> Back to Project
      </Button>
      <div className="flex items-start justify-between gap-4 mb-6">
        <div>
          <h1 className="text-3xl font-bold flex items-center gap-2">
            <Scale className="w-7 h-7 text-rose-600" /> Solution Design
          </h1>
          <p className="text-muted-foreground mt-1">
            Compare solution options against weighted criteria and land a recommendation — the Solution Architect's
            options &amp; trade-off analysis.
          </p>
        </div>
        <Button onClick={handleCreate} className="shrink-0">
          <Plus className="w-4 h-4 mr-2" /> New analysis
        </Button>
      </div>

      {/* Analysis list */}
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 mb-6">
        {isLoading && analyses.length === 0 ? (
          <p className="text-sm text-muted-foreground">Loading…</p>
        ) : analyses.length === 0 ? (
          <Card className="border-dashed sm:col-span-2 lg:col-span-3">
            <CardContent className="py-10 text-center text-sm text-muted-foreground">
              No trade-off analyses yet. Create one to compare candidate solution options.
            </CardContent>
          </Card>
        ) : (
          analyses.map((a) => {
            const rec = recommendedOptionId(a);
            const recName = a.options.find((o) => o.id === rec)?.name;
            const recPct = rankOptions(a).find((r) => r.optionId === rec)?.percent;
            const active = a.id === selectedId;
            return (
              <Card
                key={a.id}
                className={`cursor-pointer transition-shadow hover:shadow-md ${active ? 'ring-2 ring-primary' : ''}`}
                onClick={() => setSelectedId(active ? null : a.id)}
              >
                <CardContent className="pt-4 pb-3">
                  <div className="flex items-start justify-between gap-2 mb-1">
                    <p className="font-semibold">{a.title}</p>
                    <Badge variant={STATUS_VARIANT[a.status]}>{a.status}</Badge>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    {a.options.length} options · {a.criteria.length} criteria
                  </p>
                  {recName ? (
                    <p className="text-xs mt-2 flex items-center gap-1 text-foreground">
                      <Trophy className="w-3.5 h-3.5 text-amber-500" />
                      <span className="font-medium">{recName}</span>
                      {recPct != null && <span className="text-muted-foreground">({recPct}%)</span>}
                    </p>
                  ) : (
                    <p className="text-xs mt-2 text-muted-foreground">No recommendation yet</p>
                  )}
                </CardContent>
              </Card>
            );
          })
        )}
      </div>

      {/* Editor */}
      {draft && (
        <Card>
          <CardHeader className="pb-3">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <Input
                value={draft.title}
                onChange={(e) => touch((d) => { d.title = e.target.value; })}
                className="max-w-md text-lg font-semibold"
              />
              <div className="flex items-center gap-2">
                <select
                  value={draft.status}
                  onChange={(e) => touch((d) => { d.status = e.target.value as TradeoffStatus; })}
                  className="rounded-md border border-border bg-background px-2 py-1.5 text-sm"
                >
                  {STATUSES.map((s) => <option key={s.value} value={s.value}>{s.label}</option>)}
                </select>
                <Button size="sm" onClick={handleSave} disabled={!dirty || saving}>
                  <Save className="w-4 h-4 mr-1" /> {saving ? 'Saving…' : dirty ? 'Save' : 'Saved'}
                </Button>
                <Button size="sm" variant="outline" onClick={() => handleDelete(draft.id)}>
                  <Trash2 className="w-4 h-4" />
                </Button>
              </div>
            </div>
          </CardHeader>

          <CardContent className="space-y-6">
            <textarea
              value={draft.description ?? ''}
              onChange={(e) => touch((d) => { d.description = e.target.value; })}
              rows={2}
              placeholder="What decision does this analysis support?"
              className="w-full rounded-md border border-border bg-background p-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
            />

            {/* Criteria */}
            <section>
              <h3 className="text-sm font-semibold mb-2">Decision criteria & weights</h3>
              <div className="space-y-2">
                {draft.criteria.map((c) => (
                  <div key={c.id} className="flex items-center gap-2">
                    <Input
                      value={c.name}
                      onChange={(e) => touch((d) => { const t = d.criteria.find((x) => x.id === c.id); if (t) t.name = e.target.value; })}
                      className="flex-1"
                    />
                    <label className="text-xs text-muted-foreground">weight</label>
                    <select
                      value={c.weight}
                      onChange={(e) => touch((d) => { const t = d.criteria.find((x) => x.id === c.id); if (t) t.weight = parseInt(e.target.value, 10); })}
                      className="rounded-md border border-border bg-background px-2 py-1.5 text-sm"
                    >
                      {[1, 2, 3, 4, 5].map((w) => <option key={w} value={w}>{w}</option>)}
                    </select>
                    <Button
                      size="icon"
                      variant="ghost"
                      onClick={() => touch((d) => {
                        d.criteria = d.criteria.filter((x) => x.id !== c.id);
                        d.options.forEach((o) => { delete o.scores[c.id]; });
                      })}
                    >
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </div>
                ))}
              </div>
              <Button
                size="sm"
                variant="outline"
                className="mt-2"
                onClick={() => touch((d) => { d.criteria.push({ id: newId('c'), name: 'New criterion', weight: 3 }); })}
              >
                <Plus className="w-4 h-4 mr-1" /> Add criterion
              </Button>
            </section>

            {/* Options */}
            <section>
              <h3 className="text-sm font-semibold mb-2">Solution options</h3>
              <div className="space-y-4">
                {draft.options.map((o) => {
                  const r = rankFor(o.id);
                  const isRec = recId === o.id;
                  return (
                    <Card key={o.id} className={isRec ? 'border-amber-400' : ''}>
                      <CardContent className="pt-4 space-y-3">
                        <div className="flex items-center gap-2">
                          <Input
                            value={o.name}
                            onChange={(e) => touch((d) => { const t = d.options.find((x) => x.id === o.id); if (t) t.name = e.target.value; })}
                            className="flex-1 font-medium"
                          />
                          {r && (
                            <Badge variant={r.rank === 1 ? 'default' : 'secondary'}>
                              {r.percent}% · #{r.rank}
                            </Badge>
                          )}
                          <Button
                            size="sm"
                            variant={isRec ? 'default' : 'outline'}
                            onClick={() => touch((d) => { d.recommendedOptionId = isRec ? undefined : o.id; })}
                            title={isRec ? 'Pinned as recommended' : 'Pin as recommended'}
                          >
                            <Star className={`w-4 h-4 ${isRec ? 'fill-current' : ''}`} />
                          </Button>
                          <Button
                            size="icon"
                            variant="ghost"
                            onClick={() => touch((d) => { d.options = d.options.filter((x) => x.id !== o.id); if (d.recommendedOptionId === o.id) d.recommendedOptionId = undefined; })}
                          >
                            <Trash2 className="w-4 h-4" />
                          </Button>
                        </div>

                        <Input
                          value={o.description ?? ''}
                          onChange={(e) => touch((d) => { const t = d.options.find((x) => x.id === o.id); if (t) t.description = e.target.value; })}
                          placeholder="Short description"
                          className="text-sm"
                        />

                        {/* Score matrix */}
                        <div className="flex flex-wrap gap-3">
                          {draft.criteria.map((c) => (
                            <label key={c.id} className="text-xs text-muted-foreground">
                              <span className="block mb-0.5">{c.name}</span>
                              <select
                                value={o.scores[c.id] ?? 0}
                                onChange={(e) => touch((d) => { const t = d.options.find((x) => x.id === o.id); if (t) t.scores[c.id] = parseInt(e.target.value, 10); })}
                                className="rounded-md border border-border bg-background px-2 py-1 text-sm text-foreground"
                              >
                                <option value={0}>–</option>
                                {[1, 2, 3, 4, 5].map((s) => <option key={s} value={s}>{s}</option>)}
                              </select>
                            </label>
                          ))}
                        </div>

                        {/* Pros / Cons */}
                        <div className="grid gap-3 sm:grid-cols-2">
                          <label className="text-xs text-muted-foreground">
                            <span className="block mb-1">Pros (one per line)</span>
                            <textarea
                              value={o.pros.join('\n')}
                              onChange={(e) => touch((d) => { const t = d.options.find((x) => x.id === o.id); if (t) t.pros = linesToArr(e.target.value); })}
                              rows={3}
                              className="w-full rounded-md border border-border bg-background p-2 text-sm text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                            />
                          </label>
                          <label className="text-xs text-muted-foreground">
                            <span className="block mb-1">Cons (one per line)</span>
                            <textarea
                              value={o.cons.join('\n')}
                              onChange={(e) => touch((d) => { const t = d.options.find((x) => x.id === o.id); if (t) t.cons = linesToArr(e.target.value); })}
                              rows={3}
                              className="w-full rounded-md border border-border bg-background p-2 text-sm text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
                            />
                          </label>
                        </div>
                      </CardContent>
                    </Card>
                  );
                })}
              </div>
              <Button
                size="sm"
                variant="outline"
                className="mt-3"
                onClick={() => touch((d) => { d.options.push({ id: newId('o'), name: `Option ${String.fromCharCode(65 + d.options.length)}`, description: '', scores: {}, pros: [], cons: [] }); })}
              >
                <Plus className="w-4 h-4 mr-1" /> Add option
              </Button>
            </section>

            {/* Ranking + recommendation */}
            <section>
              <h3 className="text-sm font-semibold mb-2">Ranking</h3>
              {ranking.length === 0 ? (
                <p className="text-sm text-muted-foreground">Add options and score them to see a ranking.</p>
              ) : (
                <div className="space-y-2">
                  {ranking.map((r) => {
                    const isRec = recId === r.optionId;
                    return (
                      <div key={r.optionId} className="flex items-center gap-3">
                        <span className="w-40 truncate text-sm flex items-center gap-1">
                          {isRec && <Trophy className="w-3.5 h-3.5 text-amber-500 shrink-0" />}
                          {r.name}
                        </span>
                        <div className="flex-1 h-2.5 rounded-full bg-muted overflow-hidden">
                          <div className={`h-full ${isRec ? 'bg-amber-500' : 'bg-primary'}`} style={{ width: `${r.percent}%` }} />
                        </div>
                        <span className="w-10 text-right text-sm font-medium">{r.percent}%</span>
                      </div>
                    );
                  })}
                </div>
              )}
            </section>

            {/* Decision rationale (ADR-lite) */}
            <section>
              <h3 className="text-sm font-semibold mb-2">Decision rationale</h3>
              <textarea
                value={draft.decisionRationale ?? ''}
                onChange={(e) => touch((d) => { d.decisionRationale = e.target.value; })}
                rows={3}
                placeholder="Why is the recommended option the right call? (becomes the basis for an ADR)"
                className="w-full rounded-md border border-border bg-background p-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
              />
            </section>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
