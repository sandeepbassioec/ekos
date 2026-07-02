/**
 * NFR & Quality Attributes widget.
 * Structured capture of non-functional requirements across ISO 25010 quality categories.
 * Users select targets via sliders/dropdowns — no free-form typing required.
 */
import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useContext } from 'react';
import { GripVertical, ChevronDown, ChevronUp, Plus, Trash2, Check } from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

// ── Types ─────────────────────────────────────────────────────────────────────

type RatingLevel = 'not_applicable' | 'low' | 'medium' | 'high' | 'critical';
type Status = 'draft' | 'agreed' | 'verified';

interface NFRItem {
  id: string;
  label: string;         // e.g. "Response Time (p95)"
  target: string;        // e.g. "< 200ms"
  rating: RatingLevel;
  status: Status;
  notes: string;
}

interface NFRCategory {
  id: string;
  name: string;
  icon: string;
  color: string;
  items: NFRItem[];
  expanded: boolean;
}

interface NFRData {
  title: string;
  categories: NFRCategory[];
}

// ── Preset catalogue ──────────────────────────────────────────────────────────

const PRESET_CATEGORIES: Omit<NFRCategory, 'expanded'>[] = [
  {
    id: 'performance',
    name: 'Performance & Scalability',
    icon: '⚡',
    color: 'text-amber-700 bg-amber-50 border-amber-200',
    items: [
      { id: 'p1', label: 'Response Time (p95)', target: '< 200ms', rating: 'high', status: 'draft', notes: '' },
      { id: 'p2', label: 'Throughput', target: '1,000 req/s', rating: 'medium', status: 'draft', notes: '' },
      { id: 'p3', label: 'Concurrent Users', target: '10,000', rating: 'medium', status: 'draft', notes: '' },
      { id: 'p4', label: 'Data Volume', target: '1 TB', rating: 'medium', status: 'draft', notes: '' },
    ],
  },
  {
    id: 'availability',
    name: 'Availability & Reliability',
    icon: '🟢',
    color: 'text-green-700 bg-green-50 border-green-200',
    items: [
      { id: 'a1', label: 'Uptime SLA', target: '99.9%', rating: 'critical', status: 'draft', notes: '' },
      { id: 'a2', label: 'RTO (Recovery Time Objective)', target: '< 1 hour', rating: 'high', status: 'draft', notes: '' },
      { id: 'a3', label: 'RPO (Recovery Point Objective)', target: '< 15 min', rating: 'high', status: 'draft', notes: '' },
      { id: 'a4', label: 'Failover Strategy', target: 'Active-Active', rating: 'high', status: 'draft', notes: '' },
    ],
  },
  {
    id: 'security',
    name: 'Security',
    icon: '🔒',
    color: 'text-red-700 bg-red-50 border-red-200',
    items: [
      { id: 's1', label: 'Authentication', target: 'MFA required', rating: 'critical', status: 'draft', notes: '' },
      { id: 's2', label: 'Authorisation', target: 'RBAC + ABAC', rating: 'critical', status: 'draft', notes: '' },
      { id: 's3', label: 'Data Encryption (at rest)', target: 'AES-256', rating: 'critical', status: 'draft', notes: '' },
      { id: 's4', label: 'Data Encryption (in transit)', target: 'TLS 1.3', rating: 'critical', status: 'draft', notes: '' },
      { id: 's5', label: 'Audit Logging', target: 'All write ops', rating: 'high', status: 'draft', notes: '' },
    ],
  },
  {
    id: 'maintainability',
    name: 'Maintainability & Observability',
    icon: '🔧',
    color: 'text-blue-700 bg-blue-50 border-blue-200',
    items: [
      { id: 'm1', label: 'Code Coverage', target: '≥ 80%', rating: 'medium', status: 'draft', notes: '' },
      { id: 'm2', label: 'Logging', target: 'Structured JSON + correlation IDs', rating: 'high', status: 'draft', notes: '' },
      { id: 'm3', label: 'Distributed Tracing', target: 'OpenTelemetry', rating: 'medium', status: 'draft', notes: '' },
      { id: 'm4', label: 'Mean Time to Detect (MTTD)', target: '< 5 min', rating: 'high', status: 'draft', notes: '' },
    ],
  },
  {
    id: 'compliance',
    name: 'Compliance & Data Governance',
    icon: '📋',
    color: 'text-purple-700 bg-purple-50 border-purple-200',
    items: [
      { id: 'co1', label: 'Data Residency', target: 'EU only', rating: 'medium', status: 'draft', notes: '' },
      { id: 'co2', label: 'GDPR / Data Privacy', target: 'Full compliance', rating: 'critical', status: 'draft', notes: '' },
      { id: 'co3', label: 'Data Retention', target: '7 years', rating: 'medium', status: 'draft', notes: '' },
      { id: 'co4', label: 'Right to Erasure', target: '< 30 days', rating: 'medium', status: 'draft', notes: '' },
    ],
  },
  {
    id: 'usability',
    name: 'Usability & Accessibility',
    icon: '👤',
    color: 'text-indigo-700 bg-indigo-50 border-indigo-200',
    items: [
      { id: 'u1', label: 'Accessibility Standard', target: 'WCAG 2.1 AA', rating: 'high', status: 'draft', notes: '' },
      { id: 'u2', label: 'Mobile Support', target: 'iOS + Android', rating: 'medium', status: 'draft', notes: '' },
      { id: 'u3', label: 'Internationalisation', target: 'EN, FR, DE, ES', rating: 'low', status: 'draft', notes: '' },
    ],
  },
];

function buildDefault(): NFRData {
  return {
    title: 'NFR & Quality Attributes',
    categories: PRESET_CATEGORIES.map(c => ({ ...c, expanded: true })),
  };
}

// ── Helpers ───────────────────────────────────────────────────────────────────

const RATING_CONFIG: Record<RatingLevel, { label: string; color: string; dot: string }> = {
  not_applicable: { label: 'N/A',      color: 'text-gray-400 bg-gray-100',        dot: 'bg-gray-300' },
  low:            { label: 'Low',      color: 'text-gray-600 bg-gray-100',        dot: 'bg-gray-400' },
  medium:         { label: 'Medium',   color: 'text-blue-700 bg-blue-100',        dot: 'bg-blue-400' },
  high:           { label: 'High',     color: 'text-amber-700 bg-amber-100',      dot: 'bg-amber-400' },
  critical:       { label: 'Critical', color: 'text-red-700 bg-red-100',          dot: 'bg-red-500' },
};

const STATUS_CONFIG: Record<Status, { label: string; color: string; icon: string }> = {
  draft:    { label: 'Draft',    color: 'text-gray-500 border-gray-300',         icon: '○' },
  agreed:   { label: 'Agreed',   color: 'text-blue-600 border-blue-300',         icon: '◉' },
  verified: { label: 'Verified', color: 'text-green-600 border-green-300',       icon: '●' },
};

const RATINGS: RatingLevel[] = ['not_applicable', 'low', 'medium', 'high', 'critical'];
const STATUSES: Status[] = ['draft', 'agreed', 'verified'];

// ── NFR Row ───────────────────────────────────────────────────────────────────

function NFRRow({ item, editing, onChange, onDelete }: {
  item: NFRItem;
  editing: boolean;
  onChange: (patch: Partial<NFRItem>) => void;
  onDelete: () => void;
}) {
  const ratingCfg = RATING_CONFIG[item.rating];
  const statusCfg = STATUS_CONFIG[item.status];

  return (
    <tr className="border-t border-border/40 hover:bg-muted/10 transition-colors group/row">
      {/* Label */}
      <td className="py-2 px-3 text-sm">
        {editing ? (
          <input
            value={item.label}
            onChange={e => onChange({ label: e.target.value })}
            className="w-full text-sm bg-transparent border-b border-border/60 focus:outline-none focus:border-primary"
          />
        ) : (
          <span className="font-medium text-foreground">{item.label}</span>
        )}
      </td>

      {/* Target */}
      <td className="py-2 px-3 text-sm">
        {editing ? (
          <input
            value={item.target}
            onChange={e => onChange({ target: e.target.value })}
            placeholder="e.g. < 200ms"
            className="w-full text-sm bg-muted/30 border border-border/60 rounded px-2 py-0.5 focus:outline-none focus:ring-1 focus:ring-primary/40"
          />
        ) : (
          <code className="text-xs bg-muted/40 px-1.5 py-0.5 rounded font-mono text-foreground">
            {item.target || '—'}
          </code>
        )}
      </td>

      {/* Priority / Rating */}
      <td className="py-2 px-3 text-center">
        {editing ? (
          <select
            value={item.rating}
            onChange={e => onChange({ rating: e.target.value as RatingLevel })}
            className="text-[11px] border border-border rounded px-1.5 py-0.5 bg-background"
          >
            {RATINGS.map(r => (
              <option key={r} value={r}>{RATING_CONFIG[r].label}</option>
            ))}
          </select>
        ) : (
          item.rating !== 'not_applicable' && (
            <span className={`text-[10px] font-semibold px-2 py-0.5 rounded-full ${ratingCfg.color}`}>
              {ratingCfg.label}
            </span>
          )
        )}
      </td>

      {/* Status */}
      <td className="py-2 px-3 text-center">
        {editing ? (
          <select
            value={item.status}
            onChange={e => onChange({ status: e.target.value as Status })}
            className="text-[11px] border border-border rounded px-1.5 py-0.5 bg-background"
          >
            {STATUSES.map(s => (
              <option key={s} value={s}>{STATUS_CONFIG[s].label}</option>
            ))}
          </select>
        ) : (
          <span className={`text-[10px] font-medium border px-2 py-0.5 rounded-full ${statusCfg.color}`}>
            {statusCfg.icon} {statusCfg.label}
          </span>
        )}
      </td>

      {/* Notes */}
      <td className="py-2 px-3 text-sm text-muted-foreground">
        {editing ? (
          <input
            value={item.notes}
            onChange={e => onChange({ notes: e.target.value })}
            placeholder="Optional notes…"
            className="w-full text-xs bg-transparent border-b border-border/40 focus:outline-none focus:border-primary/50"
          />
        ) : (
          <span className="text-xs">{item.notes}</span>
        )}
      </td>

      {/* Delete */}
      {editing && (
        <td className="py-2 px-2">
          <button onClick={onDelete} className="opacity-0 group-hover/row:opacity-60 hover:!opacity-100 text-destructive transition-opacity">
            <Trash2 className="size-3" />
          </button>
        </td>
      )}
    </tr>
  );
}

// ── NFR Category Section ──────────────────────────────────────────────────────

function NFRCategorySection({ cat, editing, onChange }: {
  cat: NFRCategory;
  editing: boolean;
  onChange: (patch: Partial<NFRCategory>) => void;
}) {
  const toggleExpand = () => onChange({ expanded: !cat.expanded });

  const updateItem = (id: string, patch: Partial<NFRItem>) =>
    onChange({ items: cat.items.map(i => i.id === id ? { ...i, ...patch } : i) });

  const deleteItem = (id: string) =>
    onChange({ items: cat.items.filter(i => i.id !== id) });

  const addItem = () => {
    const id = `i${Date.now()}`;
    onChange({ items: [...cat.items, { id, label: 'New Requirement', target: '', rating: 'medium', status: 'draft', notes: '' }] });
  };

  // Summary stats
  const visible = cat.items.filter(i => i.rating !== 'not_applicable');
  const verified = cat.items.filter(i => i.status === 'verified').length;
  const critical = cat.items.filter(i => i.rating === 'critical').length;

  return (
    <div className={`rounded-xl border mb-3 overflow-hidden ${cat.color.split(' ').filter(c => c.startsWith('border-')).join(' ')} border`}>
      {/* Category header */}
      <button
        onClick={toggleExpand}
        className={`w-full flex items-center gap-2 px-4 py-2.5 text-left transition-colors ${cat.color.split(' ').filter(c => c.startsWith('bg-') || c.startsWith('text-')).join(' ')}`}
      >
        <span className="text-base leading-none">{cat.icon}</span>
        <span className="text-sm font-semibold flex-1">{cat.name}</span>
        <div className="flex items-center gap-2 ml-auto">
          {critical > 0 && (
            <span className="text-[10px] bg-red-100 text-red-700 px-1.5 py-0.5 rounded-full font-semibold">
              {critical} critical
            </span>
          )}
          <span className="text-[10px] text-muted-foreground">
            {verified}/{visible.length} verified
          </span>
          {cat.expanded ? <ChevronUp className="size-3.5 opacity-60" /> : <ChevronDown className="size-3.5 opacity-60" />}
        </div>
      </button>

      {/* Table */}
      {cat.expanded && (
        <div className="bg-white/80">
          <table className="w-full text-sm">
            <thead className="bg-muted/20">
              <tr className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">
                <th className="text-left py-1.5 px-3 w-48">Requirement</th>
                <th className="text-left py-1.5 px-3 w-48">Target / Threshold</th>
                <th className="text-center py-1.5 px-3 w-24">Priority</th>
                <th className="text-center py-1.5 px-3 w-24">Status</th>
                <th className="text-left py-1.5 px-3">Notes</th>
                {editing && <th className="w-8" />}
              </tr>
            </thead>
            <tbody>
              {cat.items.map(item => (
                <NFRRow
                  key={item.id}
                  item={item}
                  editing={editing}
                  onChange={patch => updateItem(item.id, patch)}
                  onDelete={() => deleteItem(item.id)}
                />
              ))}
            </tbody>
          </table>
          {editing && (
            <div className="px-3 py-2 border-t border-border/20">
              <button
                onClick={addItem}
                className="flex items-center gap-1 text-xs text-primary hover:opacity-70 transition-opacity"
              >
                <Plus className="size-3" /> Add requirement
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

// ── NFRView ───────────────────────────────────────────────────────────────────

function NFRView({ node, updateAttributes, selected }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const [editing, setEditing] = useState(false);
  const [addingCat, setAddingCat] = useState(false);
  const [newCatName, setNewCatName] = useState('');

  const raw = node.attrs.data as string;
  const data: NFRData = (() => {
    try { return raw ? { ...buildDefault(), ...JSON.parse(raw) } : buildDefault(); }
    catch { return buildDefault(); }
  })();

  const save = (next: NFRData) => updateAttributes({ data: JSON.stringify(next) });

  const updateCategory = (id: string, patch: Partial<NFRCategory>) =>
    save({ ...data, categories: data.categories.map(c => c.id === id ? { ...c, ...patch } : c) });

  const addCategory = () => {
    if (!newCatName.trim()) return;
    const id = `cat${Date.now()}`;
    save({
      ...data,
      categories: [...data.categories, {
        id, name: newCatName, icon: '📌',
        color: 'text-gray-700 bg-gray-50 border-gray-200',
        items: [], expanded: true,
      }],
    });
    setNewCatName('');
    setAddingCat(false);
  };

  // Summary
  const allItems = data.categories.flatMap(c => c.items);
  const verified = allItems.filter(i => i.status === 'verified').length;
  const agreed = allItems.filter(i => i.status === 'agreed').length;
  const critical = allItems.filter(i => i.rating === 'critical').length;

  return (
    <NodeViewWrapper>
      <div className={`group relative rounded-xl border border-border bg-card my-3 shadow-sm overflow-hidden ${selected ? 'ring-2 ring-primary/40' : ''}`}>

        {/* Header */}
        <div className="flex items-center gap-2 px-4 py-2.5 bg-muted/30 border-b border-border">
          {!isPreview && (
            <div data-drag-handle className="w-5 flex items-center justify-center cursor-grab opacity-0 group-hover:opacity-60 transition-opacity">
              <GripVertical className="size-3.5 text-muted-foreground" />
            </div>
          )}
          <span className="text-[9px] font-bold text-purple-700 bg-purple-100 px-2 py-0.5 rounded shrink-0">NFR</span>
          {editing ? (
            <input
              value={data.title}
              onChange={e => save({ ...data, title: e.target.value })}
              className="flex-1 text-sm font-semibold bg-transparent focus:outline-none"
            />
          ) : (
            <span className="flex-1 text-sm font-semibold">{data.title}</span>
          )}

          {/* Summary pills */}
          <div className="flex items-center gap-1.5 ml-auto">
            {critical > 0 && (
              <span className="text-[10px] bg-red-100 text-red-700 px-2 py-0.5 rounded-full font-semibold">
                {critical} critical
              </span>
            )}
            <span className="text-[10px] bg-green-100 text-green-700 px-2 py-0.5 rounded-full">
              <Check className="size-2.5 inline mr-0.5" />{verified} verified
            </span>
            <span className="text-[10px] bg-blue-100 text-blue-700 px-2 py-0.5 rounded-full">
              {agreed} agreed
            </span>
            {!isPreview && (
              <button
                onClick={() => setEditing(e => !e)}
                className="ml-1 text-[10px] text-muted-foreground hover:text-foreground border border-border rounded px-2 py-0.5 transition-colors"
              >
                {editing ? 'Done' : 'Edit'}
              </button>
            )}
          </div>
        </div>

        {/* Categories */}
        <div className="p-4">
          {data.categories.map(cat => (
            <NFRCategorySection
              key={cat.id}
              cat={cat}
              editing={editing}
              onChange={patch => updateCategory(cat.id, patch)}
            />
          ))}

          {/* Add category */}
          {editing && (
            addingCat ? (
              <div className="flex items-center gap-2 mt-1">
                <input
                  value={newCatName}
                  onChange={e => setNewCatName(e.target.value)}
                  placeholder="Category name…"
                  autoFocus
                  onKeyDown={e => { if (e.key === 'Enter') addCategory(); if (e.key === 'Escape') setAddingCat(false); }}
                  className="flex-1 text-sm border border-border rounded px-3 py-1.5 focus:outline-none focus:ring-1 focus:ring-primary/40"
                />
                <button onClick={addCategory} className="text-xs bg-primary text-primary-foreground px-3 py-1.5 rounded font-medium">Add</button>
                <button onClick={() => setAddingCat(false)} className="text-xs text-muted-foreground hover:text-foreground">Cancel</button>
              </div>
            ) : (
              <button
                onClick={() => setAddingCat(true)}
                className="flex items-center gap-1 text-xs text-primary hover:opacity-70 mt-1"
              >
                <Plus className="size-3" /> Add category
              </button>
            )
          )}
        </div>
      </div>
    </NodeViewWrapper>
  );
}

// ── Tiptap Extension ──────────────────────────────────────────────────────────

export const NFRExtension = Node.create({
  name: 'nfr',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return { data: { default: '' } };
  },

  parseHTML()  { return [{ tag: 'div[data-type="nfr"]' }]; },
  renderHTML({ HTMLAttributes }) {
    return ['div', mergeAttributes({ 'data-type': 'nfr' }, HTMLAttributes)];
  },
  addNodeView() { return ReactNodeViewRenderer(NFRView); },
});
