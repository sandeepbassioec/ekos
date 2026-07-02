import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useContext } from 'react';
import { GripVertical, Sparkles } from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

const STATUSES = ['PROPOSED', 'ACCEPTED', 'SUPERSEDED', 'DEPRECATED'] as const;
type AdrStatus = typeof STATUSES[number];

const STATUS_STYLE: Record<AdrStatus, string> = {
  PROPOSED:   'bg-amber-50 text-amber-700 border border-amber-200 dark:bg-amber-950/40 dark:text-amber-400 dark:border-amber-800',
  ACCEPTED:   'bg-green-50 text-green-700 border border-green-200 dark:bg-green-950/40 dark:text-green-400 dark:border-green-800',
  SUPERSEDED: 'bg-slate-100 text-slate-500 border border-slate-200 dark:bg-slate-800/50 dark:text-slate-500 dark:border-slate-700 line-through',
  DEPRECATED: 'bg-red-50 text-red-600 border border-red-200 dark:bg-red-950/40 dark:text-red-400 dark:border-red-800',
};

function cycle<T>(arr: readonly T[], current: T): T {
  const idx = arr.indexOf(current);
  return arr[(idx + 1) % arr.length];
}

function EditableField({ label, value, onSave, multiline = false, readOnly = false }: {
  label: string; value: string; onSave: (v: string) => void; multiline?: boolean; readOnly?: boolean;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value);

  const commit = () => { onSave(draft); setEditing(false); };

  return (
    <div className="py-2 border-b border-border/50 last:border-0">
      <span className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground block mb-0.5">{label}</span>
      {!readOnly && editing ? (
        multiline ? (
          <textarea
            autoFocus
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onBlur={commit}
            rows={3}
            className="w-full text-sm bg-transparent border-0 focus:outline-none resize-none text-foreground leading-relaxed"
          />
        ) : (
          <input
            autoFocus
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onBlur={commit}
            onKeyDown={(e) => e.key === 'Enter' && commit()}
            className="w-full text-sm bg-transparent border-0 focus:outline-none text-foreground"
          />
        )
      ) : (
        <p
          className={`text-sm text-foreground leading-relaxed min-h-[1.25rem] ${!readOnly ? 'cursor-text' : ''}`}
          onClick={!readOnly ? () => { setDraft(value); setEditing(true); } : undefined}
        >
          {value || (!readOnly && <span className="italic text-muted-foreground/60">Click to add {label.toLowerCase()}…</span>)}
        </p>
      )}
    </div>
  );
}

function AdrView({ node, updateAttributes, selected, getPos }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const [editId, setEditId] = useState(false);
  const [editTitle, setEditTitle] = useState(false);
  const status = (node.attrs.status || 'PROPOSED') as AdrStatus;

  return (
    <NodeViewWrapper>
      <div className={`group relative rounded-xl border border-border bg-card my-3 overflow-hidden shadow-sm ${selected ? 'ring-2 ring-primary/40' : ''}`}>
        {/* Header */}
        <div className="flex items-center gap-2.5 px-3 py-2.5 bg-violet-50/50 dark:bg-violet-950/20 border-b border-border">
          {!isPreview && (
            <div className="flex items-center justify-center w-5 cursor-grab opacity-0 group-hover:opacity-60 transition-opacity" data-drag-handle>
              <GripVertical className="size-3.5 text-muted-foreground" />
            </div>
          )}
          <div className="flex size-6 items-center justify-center rounded bg-violet-100 dark:bg-violet-900/40">
            <span className="text-[9px] font-bold text-violet-700 dark:text-violet-400">ADR</span>
          </div>
          {!isPreview && editId ? (
            <input autoFocus value={node.attrs.adrId}
              onChange={(e) => updateAttributes({ adrId: e.target.value })}
              onBlur={() => setEditId(false)}
              onKeyDown={(e) => e.key === 'Enter' && setEditId(false)}
              className="text-xs font-mono text-muted-foreground bg-transparent border-b border-border focus:outline-none w-20"
            />
          ) : (
            <span
              className={`text-xs font-mono text-muted-foreground ${!isPreview ? 'cursor-text' : ''}`}
              onClick={!isPreview ? () => setEditId(true) : undefined}
            >
              {node.attrs.adrId || 'ADR-001'}
            </span>
          )}

          {!isPreview && editTitle ? (
            <input autoFocus value={node.attrs.title}
              onChange={(e) => updateAttributes({ title: e.target.value })}
              onBlur={() => setEditTitle(false)}
              onKeyDown={(e) => e.key === 'Enter' && setEditTitle(false)}
              className="flex-1 text-sm font-semibold bg-transparent border-b border-border focus:outline-none text-foreground"
            />
          ) : (
            <span
              className={`flex-1 text-sm font-semibold text-foreground truncate ${!isPreview ? 'cursor-text' : ''}`}
              onClick={!isPreview ? () => setEditTitle(true) : undefined}
            >
              {node.attrs.title || 'Decision title…'}
            </span>
          )}

          <span
            className={`text-[10px] font-bold px-2.5 py-0.5 rounded-full shrink-0 transition-colors ${STATUS_STYLE[status]} ${!isPreview ? 'cursor-pointer' : ''}`}
            onClick={!isPreview ? () => updateAttributes({ status: cycle(STATUSES, status) }) : undefined}
          >
            {status}
          </span>
          {!isPreview && (
            <button
              onMouseDown={(e) => {
                e.preventDefault();
                e.stopPropagation();
                document.dispatchEvent(new CustomEvent('ask-sandeep-target', {
                  bubbles: true,
                  detail: { pos: getPos?.(), nodeType: 'adr', attrs: { ...node.attrs }, targetField: 'context', displayName: `${node.attrs.adrId || 'ADR'}: ${node.attrs.title || 'decision'}` },
                }));
              }}
              className="opacity-0 group-hover:opacity-70 hover:opacity-100 rounded p-0.5 hover:bg-primary/10 transition-all shrink-0"
              title="Ask Sandeep AI"
            >
              <Sparkles className="size-3.5 text-primary" />
            </button>
          )}
        </div>

        {/* Fields */}
        <div className="px-4 py-1">
          <EditableField label="Context" value={node.attrs.context} onSave={(v) => updateAttributes({ context: v })} multiline readOnly={isPreview} />
          <EditableField label="Decision" value={node.attrs.decision} onSave={(v) => updateAttributes({ decision: v })} multiline readOnly={isPreview} />
          <EditableField label="Consequences" value={node.attrs.consequences} onSave={(v) => updateAttributes({ consequences: v })} multiline readOnly={isPreview} />
        </div>
      </div>
    </NodeViewWrapper>
  );
}

export const AdrExtension = Node.create({
  name: 'adr',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      adrId:        { default: 'ADR-001' },
      title:        { default: '' },
      status:       { default: 'PROPOSED' },
      context:      { default: '' },
      decision:     { default: '' },
      consequences: { default: '' },
    };
  },

  parseHTML() { return [{ tag: 'div[data-type="adr"]' }]; },
  renderHTML({ HTMLAttributes }) { return ['div', mergeAttributes({ 'data-type': 'adr' }, HTMLAttributes)]; },
  addNodeView() { return ReactNodeViewRenderer(AdrView); },
});
