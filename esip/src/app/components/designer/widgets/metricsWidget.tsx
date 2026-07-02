import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useContext } from 'react';
import { GripVertical, Plus, X, TrendingUp, TrendingDown, Minus } from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

interface Metric {
  id: string;
  value: string;
  label: string;
  delta: string;
  trend: 'up' | 'down' | 'neutral';
}

const TREND_STYLE = {
  up:      { val: 'text-green-600 dark:text-green-400', delta: 'text-green-600 dark:text-green-400', icon: TrendingUp },
  down:    { val: 'text-red-500 dark:text-red-400',     delta: 'text-red-500 dark:text-red-400',     icon: TrendingDown },
  neutral: { val: 'text-primary',                       delta: 'text-muted-foreground',               icon: Minus },
};

function parseMetrics(raw: string): Metric[] {
  try { return JSON.parse(raw); } catch { return []; }
}

function MetricTile({ metric, onUpdate, onRemove, readOnly = false }: {
  metric: Metric;
  onUpdate: (m: Metric) => void;
  onRemove: () => void;
  readOnly?: boolean;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(metric);
  const cfg = TREND_STYLE[metric.trend];
  const Icon = cfg.icon;

  if (!readOnly && editing) {
    return (
      <div className="rounded-lg border border-primary/40 bg-primary/5 p-2 flex flex-col gap-1.5 text-xs">
        <input placeholder="Value (e.g. 99.97%)" value={draft.value} onChange={(e) => setDraft(d => ({ ...d, value: e.target.value }))} className="bg-transparent border-b border-border focus:outline-none font-bold text-sm" />
        <input placeholder="Label (e.g. Uptime)" value={draft.label} onChange={(e) => setDraft(d => ({ ...d, label: e.target.value }))} className="bg-transparent border-b border-border focus:outline-none text-xs text-muted-foreground" />
        <input placeholder="Delta (e.g. ▲ +0.12%)" value={draft.delta} onChange={(e) => setDraft(d => ({ ...d, delta: e.target.value }))} className="bg-transparent border-b border-border focus:outline-none" />
        <div className="flex items-center gap-1.5 mt-1">
          <span className="text-muted-foreground">Trend:</span>
          {(['up', 'down', 'neutral'] as const).map(t => (
            <button key={t} onClick={() => setDraft(d => ({ ...d, trend: t }))}
              className={`px-2 py-0.5 rounded text-[10px] font-semibold border transition-colors ${draft.trend === t ? 'bg-primary text-primary-foreground border-primary' : 'border-border text-muted-foreground hover:border-primary/50'}`}>
              {t}
            </button>
          ))}
          <button onClick={() => { onUpdate(draft); setEditing(false); }} className="ml-auto text-[10px] bg-primary text-primary-foreground px-2 py-0.5 rounded font-semibold">Done</button>
        </div>
      </div>
    );
  }

  return (
    <div
      className={`group/tile relative rounded-lg border border-border bg-background p-3 text-center transition-colors ${!readOnly ? 'cursor-pointer hover:border-primary/40' : ''}`}
      onClick={!readOnly ? () => { setDraft(metric); setEditing(true); } : undefined}
    >
      {!readOnly && (
        <button
          onClick={(e) => { e.stopPropagation(); onRemove(); }}
          className="absolute top-1 right-1 opacity-0 group-hover/tile:opacity-100 transition-opacity text-muted-foreground hover:text-destructive"
        >
          <X className="size-3" />
        </button>
      )}
      <div className={`text-2xl font-black leading-none mb-1 ${cfg.val}`}>{metric.value || '—'}</div>
      <div className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground mb-1">{metric.label || 'Label'}</div>
      {metric.delta && (
        <div className={`flex items-center justify-center gap-0.5 text-[11px] font-medium ${cfg.delta}`}>
          <Icon className="size-3" />
          {metric.delta}
        </div>
      )}
    </div>
  );
}

function MetricsView({ node, updateAttributes, selected }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const metrics = parseMetrics(node.attrs.metrics);
  const [title, setTitle] = useState(node.attrs.title || '');

  const save = (updated: Metric[]) => updateAttributes({ metrics: JSON.stringify(updated) });

  const addMetric = () => save([...metrics, { id: Date.now().toString(), value: '', label: 'New Metric', delta: '', trend: 'neutral' }]);
  const updateMetric = (id: string, m: Metric) => save(metrics.map(x => x.id === id ? m : x));
  const removeMetric = (id: string) => save(metrics.filter(x => x.id !== id));

  return (
    <NodeViewWrapper>
      <div className={`group relative rounded-xl border border-border bg-card my-3 overflow-hidden shadow-sm ${selected ? 'ring-2 ring-primary/40' : ''}`}>
        <div className="flex items-center gap-2 px-3 py-2 bg-muted/30 border-b border-border">
          {!isPreview && (
            <div className="flex items-center justify-center w-5 cursor-grab opacity-0 group-hover:opacity-60 transition-opacity" data-drag-handle>
              <GripVertical className="size-3.5 text-muted-foreground" />
            </div>
          )}
          <span className="text-[10px] font-bold uppercase tracking-wide text-muted-foreground">📊</span>
          {isPreview ? (
            <span className="flex-1 text-sm font-semibold text-foreground">{title || 'Metrics / KPIs'}</span>
          ) : (
            <input
              value={title}
              onChange={(e) => { setTitle(e.target.value); updateAttributes({ title: e.target.value }); }}
              placeholder="Metrics / KPIs"
              className="flex-1 text-sm font-semibold bg-transparent focus:outline-none text-foreground placeholder:text-muted-foreground"
            />
          )}
          {!isPreview && (
            <button onClick={addMetric} className="flex items-center gap-1 text-[10px] text-primary hover:text-primary/80 transition-colors opacity-0 group-hover:opacity-100">
              <Plus className="size-3" /> Add
            </button>
          )}
        </div>
        <div className={`grid gap-2.5 p-3 ${metrics.length <= 2 ? 'grid-cols-2' : 'grid-cols-4'}`}>
          {metrics.map(m => (
            <MetricTile key={m.id} metric={m} onUpdate={(updated) => updateMetric(m.id, updated)} onRemove={() => removeMetric(m.id)} readOnly={isPreview} />
          ))}
          {metrics.length === 0 && (
            <div className="col-span-2 text-center py-6 text-muted-foreground/60 text-sm cursor-pointer" onClick={addMetric}>
              <div className="text-2xl mb-1">📊</div>
              Click "+ Add" to add your first metric
            </div>
          )}
        </div>
      </div>
    </NodeViewWrapper>
  );
}

const defaultMetrics: Metric[] = [
  { id: '1', value: '99.97%', label: 'Uptime',     delta: '▲ +0.12%', trend: 'up' },
  { id: '2', value: '142ms',  label: 'P99 Latency', delta: '▼ −18ms',  trend: 'down' },
  { id: '3', value: '84%',    label: 'Test Coverage', delta: '▲ +6%',  trend: 'up' },
  { id: '4', value: '3',      label: 'Open P1 Bugs', delta: '▲ +1',    trend: 'down' },
];

export const MetricsExtension = Node.create({
  name: 'metrics',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      title:   { default: 'Key Metrics' },
      metrics: { default: JSON.stringify(defaultMetrics) },
    };
  },

  parseHTML() { return [{ tag: 'div[data-type="metrics"]' }]; },
  renderHTML({ HTMLAttributes }) { return ['div', mergeAttributes({ 'data-type': 'metrics' }, HTMLAttributes)]; },
  addNodeView() { return ReactNodeViewRenderer(MetricsView); },
});
