/**
 * Trade-off / Options Analysis widget.
 * Compare 2–4 options across configurable criteria.
 * Each cell has a 1–5 score; winner is auto-highlighted.
 */
import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useContext } from 'react';
import { GripVertical, Plus, Trash2, Trophy, ChevronDown } from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

// ── Types ─────────────────────────────────────────────────────────────────────

interface Option {
  id: string;
  name: string;
  description: string;
}

interface Criterion {
  id: string;
  name: string;
  weight: number; // 1 = normal, 2 = important, 3 = critical
}

interface TradeoffData {
  title: string;
  context: string;
  options: Option[];
  criteria: Criterion[];
  scores: Record<string, Record<string, number>>; // scores[criterionId][optionId]
  decision: string;
}

// ── Defaults ──────────────────────────────────────────────────────────────────

const DEFAULT_DATA: TradeoffData = {
  title: 'Options Analysis',
  context: '',
  options: [
    { id: 'o1', name: 'Option A', description: '' },
    { id: 'o2', name: 'Option B', description: '' },
    { id: 'o3', name: 'Option C', description: '' },
  ],
  criteria: [
    { id: 'c1', name: 'Cost',          weight: 2 },
    { id: 'c2', name: 'Complexity',    weight: 2 },
    { id: 'c3', name: 'Risk',          weight: 3 },
    { id: 'c4', name: 'Time to Deliver', weight: 1 },
    { id: 'c5', name: 'Scalability',   weight: 1 },
  ],
  scores: {},
  decision: '',
};

// ── Score helpers ─────────────────────────────────────────────────────────────

const SCORE_COLORS = ['', 'bg-red-400', 'bg-orange-400', 'bg-yellow-400', 'bg-lime-400', 'bg-green-500'];
const SCORE_LABELS = ['', '1 – Very Poor', '2 – Poor', '3 – Neutral', '4 – Good', '5 – Excellent'];

function getScore(data: TradeoffData, criterionId: string, optionId: string): number {
  return data.scores[criterionId]?.[optionId] ?? 0;
}

function weightedTotal(data: TradeoffData, optionId: string): number {
  return data.criteria.reduce((sum, c) => {
    return sum + getScore(data, c.id, optionId) * c.weight;
  }, 0);
}

function maxPossible(data: TradeoffData): number {
  return data.criteria.reduce((sum, c) => sum + 5 * c.weight, 0);
}

const WEIGHT_LABELS: Record<number, string> = { 1: 'Low', 2: 'Medium', 3: 'High' };
const WEIGHT_COLORS: Record<number, string> = {
  1: 'text-gray-500 bg-gray-100',
  2: 'text-blue-700 bg-blue-100',
  3: 'text-red-700 bg-red-100',
};

// ── ScoreCell ─────────────────────────────────────────────────────────────────

function ScoreCell({
  score, editing, onChange,
}: { score: number; editing: boolean; onChange: (v: number) => void }) {
  const [open, setOpen] = useState(false);

  if (!editing) {
    return (
      <div className="flex items-center justify-center">
        {score === 0 ? (
          <span className="text-gray-300 text-xs">—</span>
        ) : (
          <span className={`w-7 h-7 rounded-full flex items-center justify-center text-white text-xs font-bold ${SCORE_COLORS[score]}`}>
            {score}
          </span>
        )}
      </div>
    );
  }

  return (
    <div className="relative flex items-center justify-center">
      <button
        onClick={() => setOpen(o => !o)}
        className={`w-7 h-7 rounded-full flex items-center justify-center text-white text-xs font-bold border-2 border-dashed border-gray-300 transition-all ${score > 0 ? SCORE_COLORS[score] : 'bg-gray-100 text-gray-400'}`}
        title={score > 0 ? SCORE_LABELS[score] : 'Click to score'}
      >
        {score > 0 ? score : '+'}
      </button>
      {open && (
        <div className="absolute z-50 top-9 left-1/2 -translate-x-1/2 bg-white border border-gray-200 rounded-xl shadow-xl p-2 flex gap-1.5 min-w-[160px]">
          {[1, 2, 3, 4, 5].map(v => (
            <button
              key={v}
              onClick={() => { onChange(v); setOpen(false); }}
              className={`w-7 h-7 rounded-full flex items-center justify-center text-white text-xs font-bold transition-transform hover:scale-110 ${SCORE_COLORS[v]} ${score === v ? 'ring-2 ring-offset-1 ring-gray-600' : ''}`}
              title={SCORE_LABELS[v]}
            >
              {v}
            </button>
          ))}
          {score > 0 && (
            <button
              onClick={() => { onChange(0); setOpen(false); }}
              className="w-7 h-7 rounded-full flex items-center justify-center text-xs text-gray-400 border border-gray-200 hover:bg-gray-50"
              title="Clear"
            >
              ✕
            </button>
          )}
        </div>
      )}
    </div>
  );
}

// ── TradeoffView ──────────────────────────────────────────────────────────────

function TradeoffView({ node, updateAttributes, selected }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const [editing, setEditing] = useState(false);

  const raw = node.attrs.data as string;
  const data: TradeoffData = (() => {
    try { return raw ? { ...DEFAULT_DATA, ...JSON.parse(raw) } : { ...DEFAULT_DATA }; }
    catch { return { ...DEFAULT_DATA }; }
  })();

  const save = (next: TradeoffData) => updateAttributes({ data: JSON.stringify(next) });

  const setField = <K extends keyof TradeoffData>(key: K, value: TradeoffData[K]) =>
    save({ ...data, [key]: value });

  const setScore = (criterionId: string, optionId: string, value: number) => {
    const scores = { ...data.scores, [criterionId]: { ...(data.scores[criterionId] ?? {}), [optionId]: value } };
    save({ ...data, scores });
  };

  const addOption = () => {
    if (data.options.length >= 4) return;
    const id = `o${Date.now()}`;
    save({ ...data, options: [...data.options, { id, name: `Option ${data.options.length + 1}`, description: '' }] });
  };

  const removeOption = (id: string) => {
    const options = data.options.filter(o => o.id !== id);
    save({ ...data, options });
  };

  const addCriterion = () => {
    const id = `c${Date.now()}`;
    save({ ...data, criteria: [...data.criteria, { id, name: 'New Criterion', weight: 1 }] });
  };

  const removeCriterion = (id: string) => save({ ...data, criteria: data.criteria.filter(c => c.id !== id) });

  const updateOption = (id: string, patch: Partial<Option>) =>
    save({ ...data, options: data.options.map(o => o.id === id ? { ...o, ...patch } : o) });

  const updateCriterion = (id: string, patch: Partial<Criterion>) =>
    save({ ...data, criteria: data.criteria.map(c => c.id === id ? { ...c, ...patch } : c) });

  // Winner calculation
  const totals = data.options.map(o => ({ id: o.id, score: weightedTotal(data, o.id) }));
  const maxScore = Math.max(...totals.map(t => t.score));
  const maxPoss = maxPossible(data);
  const winnerId = maxScore > 0 ? (totals.find(t => t.score === maxScore)?.id ?? null) : null;

  return (
    <NodeViewWrapper>
      <div className={`group relative rounded-xl border border-border bg-card my-3 shadow-sm overflow-hidden ${selected ? 'ring-2 ring-primary/40' : ''}`}>

        {/* Header */}
        <div className="flex items-center gap-2 px-3 py-2 bg-muted/30 border-b border-border">
          {!isPreview && (
            <div data-drag-handle className="w-5 flex items-center justify-center cursor-grab opacity-0 group-hover:opacity-60 transition-opacity">
              <GripVertical className="size-3.5 text-muted-foreground" />
            </div>
          )}
          <span className="text-[9px] font-bold text-amber-700 bg-amber-100 px-2 py-0.5 rounded shrink-0">TRADEOFF</span>
          {editing ? (
            <input
              value={data.title}
              onChange={e => setField('title', e.target.value)}
              className="flex-1 text-sm font-semibold bg-transparent focus:outline-none"
            />
          ) : (
            <span className="flex-1 text-sm font-semibold">{data.title || 'Options Analysis'}</span>
          )}
          {!isPreview && (
            <button
              onClick={() => setEditing(e => !e)}
              className="ml-auto text-[10px] text-muted-foreground hover:text-foreground border border-border rounded px-2 py-0.5 transition-colors"
            >
              {editing ? 'Done' : 'Edit'}
            </button>
          )}
        </div>

        {/* Context */}
        {(editing || data.context) && (
          <div className="px-4 pt-3 pb-1">
            {editing ? (
              <input
                value={data.context}
                onChange={e => setField('context', e.target.value)}
                placeholder="Describe the decision context… e.g. 'Choose between REST, gRPC, and GraphQL for the public API'"
                className="w-full text-sm text-muted-foreground bg-muted/20 rounded px-2 py-1.5 focus:outline-none focus:ring-1 focus:ring-primary/30 border border-border/50"
              />
            ) : (
              <p className="text-sm text-muted-foreground italic">{data.context}</p>
            )}
          </div>
        )}

        {/* Table */}
        <div className="overflow-x-auto px-4 py-3">
          <table className="w-full text-sm border-collapse">
            <thead>
              <tr>
                {/* Criterion label col */}
                <th className="text-left text-xs font-semibold text-muted-foreground pb-2 pr-3 w-36">Criterion</th>
                <th className="text-center text-xs text-muted-foreground pb-2 pr-3 w-16">Weight</th>
                {/* Option cols */}
                {data.options.map(opt => (
                  <th key={opt.id} className={`text-center pb-2 px-2 min-w-[90px] ${winnerId === opt.id ? 'text-green-700' : 'text-foreground'}`}>
                    <div className="flex flex-col items-center gap-1">
                      {editing ? (
                        <div className="flex items-center gap-1">
                          <input
                            value={opt.name}
                            onChange={e => updateOption(opt.id, { name: e.target.value })}
                            className="text-xs font-semibold bg-transparent border-b border-border focus:outline-none text-center w-20"
                          />
                          {data.options.length > 2 && (
                            <button onClick={() => removeOption(opt.id)} className="text-destructive hover:opacity-70">
                              <Trash2 className="size-3" />
                            </button>
                          )}
                        </div>
                      ) : (
                        <span className="text-xs font-semibold flex items-center gap-1">
                          {winnerId === opt.id && <Trophy className="size-3 text-amber-500" />}
                          {opt.name}
                        </span>
                      )}
                      {editing && (
                        <input
                          value={opt.description}
                          onChange={e => updateOption(opt.id, { description: e.target.value })}
                          placeholder="e.g. PostgreSQL"
                          className="text-[10px] text-muted-foreground bg-transparent border-b border-border/40 focus:outline-none text-center w-24"
                        />
                      )}
                      {!editing && opt.description && (
                        <span className="text-[10px] text-muted-foreground">{opt.description}</span>
                      )}
                    </div>
                  </th>
                ))}
                {editing && data.options.length < 4 && (
                  <th className="pb-2 px-2">
                    <button onClick={addOption} className="text-xs text-primary hover:opacity-70 flex items-center gap-0.5">
                      <Plus className="size-3" /> Add
                    </button>
                  </th>
                )}
              </tr>
            </thead>
            <tbody>
              {data.criteria.map((crit, i) => (
                <tr key={crit.id} className={i % 2 === 0 ? 'bg-muted/10' : ''}>
                  <td className="py-2 pr-3">
                    {editing ? (
                      <div className="flex items-center gap-1">
                        <input
                          value={crit.name}
                          onChange={e => updateCriterion(crit.id, { name: e.target.value })}
                          className="text-xs bg-transparent border-b border-border focus:outline-none flex-1"
                        />
                        <button onClick={() => removeCriterion(crit.id)} className="text-destructive hover:opacity-70 shrink-0">
                          <Trash2 className="size-3" />
                        </button>
                      </div>
                    ) : (
                      <span className="text-xs font-medium text-foreground">{crit.name}</span>
                    )}
                  </td>
                  <td className="py-2 pr-3 text-center">
                    {editing ? (
                      <select
                        value={crit.weight}
                        onChange={e => updateCriterion(crit.id, { weight: Number(e.target.value) })}
                        className="text-[10px] border border-border rounded px-1 py-0.5 bg-background"
                      >
                        <option value={1}>Low</option>
                        <option value={2}>Med</option>
                        <option value={3}>High</option>
                      </select>
                    ) : (
                      <span className={`text-[9px] font-semibold px-1.5 py-0.5 rounded ${WEIGHT_COLORS[crit.weight]}`}>
                        {WEIGHT_LABELS[crit.weight]}
                      </span>
                    )}
                  </td>
                  {data.options.map(opt => (
                    <td key={opt.id} className={`py-2 px-2 text-center ${winnerId === opt.id ? 'bg-green-50/60' : ''}`}>
                      <ScoreCell
                        score={getScore(data, crit.id, opt.id)}
                        editing={editing}
                        onChange={v => setScore(crit.id, opt.id, v)}
                      />
                    </td>
                  ))}
                  {editing && data.options.length < 4 && <td />}
                </tr>
              ))}
            </tbody>

            {/* Totals row */}
            <tfoot>
              <tr className="border-t-2 border-border">
                <td className="pt-2 text-xs font-bold text-foreground">Weighted Score</td>
                <td className="pt-2 text-[10px] text-muted-foreground text-center">/{maxPoss}</td>
                {data.options.map(opt => {
                  const total = weightedTotal(data, opt.id);
                  const pct = maxPoss > 0 ? Math.round((total / maxPoss) * 100) : 0;
                  return (
                    <td key={opt.id} className={`pt-2 px-2 text-center ${winnerId === opt.id ? 'bg-green-50/60' : ''}`}>
                      <div className="flex flex-col items-center gap-1">
                        <span className={`text-sm font-bold ${winnerId === opt.id ? 'text-green-700' : 'text-foreground'}`}>
                          {total > 0 ? total : '—'}
                        </span>
                        {total > 0 && (
                          <div className="w-12 h-1.5 bg-gray-100 rounded-full overflow-hidden">
                            <div
                              className={`h-full rounded-full ${winnerId === opt.id ? 'bg-green-500' : 'bg-indigo-400'}`}
                              style={{ width: `${pct}%` }}
                            />
                          </div>
                        )}
                      </div>
                    </td>
                  );
                })}
                {editing && data.options.length < 4 && <td />}
              </tr>
            </tfoot>
          </table>

          {/* Add criterion */}
          {editing && (
            <button onClick={addCriterion} className="mt-2 flex items-center gap-1 text-xs text-primary hover:opacity-70">
              <Plus className="size-3" /> Add criterion
            </button>
          )}
        </div>

        {/* Decision / recommendation */}
        {(editing || data.decision) && (
          <div className={`mx-4 mb-4 rounded-lg border px-3 py-2 ${winnerId ? 'border-green-200 bg-green-50/60' : 'border-border bg-muted/10'}`}>
            <p className="text-[10px] font-bold uppercase tracking-wide text-muted-foreground mb-1">Decision / Recommendation</p>
            {editing ? (
              <textarea
                value={data.decision}
                onChange={e => setField('decision', e.target.value)}
                placeholder="Record the final decision and rationale… e.g. 'We chose Option B (PostgreSQL) due to lower operational complexity and team familiarity.'"
                rows={2}
                className="w-full text-sm bg-transparent focus:outline-none resize-none"
              />
            ) : (
              <p className="text-sm text-foreground">{data.decision}</p>
            )}
          </div>
        )}
      </div>
    </NodeViewWrapper>
  );
}

// ── Tiptap Extension ──────────────────────────────────────────────────────────

export const TradeoffExtension = Node.create({
  name: 'tradeoff',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return { data: { default: '' } };
  },

  parseHTML()  { return [{ tag: 'div[data-type="tradeoff"]' }]; },
  renderHTML({ HTMLAttributes }) {
    return ['div', mergeAttributes({ 'data-type': 'tradeoff' }, HTMLAttributes)];
  },
  addNodeView() { return ReactNodeViewRenderer(TradeoffView); },
});
