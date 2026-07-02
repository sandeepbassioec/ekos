import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useContext } from 'react';
import { PreviewModeContext } from '../PreviewModeContext';

type BadgeStatus = 'done' | 'in-progress' | 'blocked' | 'review' | 'planned';

const STYLES: Record<BadgeStatus, { cls: string; dot: string; label: string }> = {
  'done':        { cls: 'bg-green-50 text-green-700 border-green-200 dark:bg-green-950/40 dark:text-green-400 dark:border-green-800',   dot: 'bg-green-500',  label: 'Done' },
  'in-progress': { cls: 'bg-blue-50 text-blue-700 border-blue-200 dark:bg-blue-950/40 dark:text-blue-400 dark:border-blue-800',       dot: 'bg-blue-500',   label: 'In Progress' },
  'blocked':     { cls: 'bg-red-50 text-red-700 border-red-200 dark:bg-red-950/40 dark:text-red-400 dark:border-red-800',             dot: 'bg-red-500',    label: 'Blocked' },
  'review':      { cls: 'bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-950/40 dark:text-amber-400 dark:border-amber-800', dot: 'bg-amber-500',  label: 'Review' },
  'planned':     { cls: 'bg-slate-50 text-slate-600 border-slate-200 dark:bg-slate-800 dark:text-slate-400 dark:border-slate-700',    dot: 'bg-slate-400',  label: 'Planned' },
};

const STATUS_ORDER: BadgeStatus[] = ['done', 'in-progress', 'blocked', 'review', 'planned'];

function StatusBadgeView({ node, updateAttributes }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const [showMenu, setShowMenu] = useState(false);
  const status = (node.attrs.status || 'in-progress') as BadgeStatus;
  const cfg = STYLES[status];
  const label = node.attrs.label || cfg.label;

  return (
    <NodeViewWrapper as="span" className="inline">
      <span className="relative inline-block">
        <span
          onClick={!isPreview ? () => setShowMenu(s => !s) : undefined}
          className={`inline-flex items-center gap-1 rounded px-2 py-0.5 text-[11px] font-semibold border select-none transition-all ${cfg.cls} ${!isPreview ? 'cursor-pointer hover:opacity-90' : ''}`}
        >
          <span className={`w-1.5 h-1.5 rounded-full ${cfg.dot}`} />
          {label}
        </span>
        {!isPreview && showMenu && (
          <div className="absolute z-50 top-full left-0 mt-1 bg-card border border-border rounded-lg shadow-xl overflow-hidden min-w-[140px]" onMouseLeave={() => setShowMenu(false)}>
            {STATUS_ORDER.map(s => {
              const c = STYLES[s];
              return (
                <button
                  key={s}
                  onClick={() => { updateAttributes({ status: s, label: c.label }); setShowMenu(false); }}
                  className="w-full flex items-center gap-2 px-3 py-1.5 text-xs hover:bg-muted transition-colors text-left"
                >
                  <span className={`w-2 h-2 rounded-full ${c.dot}`} />
                  {c.label}
                </button>
              );
            })}
            <div className="border-t border-border px-2 py-1.5">
              <input
                placeholder="Custom label…"
                defaultValue={label}
                onClick={(e) => e.stopPropagation()}
                onKeyDown={(e) => { if (e.key === 'Enter') { updateAttributes({ label: (e.target as HTMLInputElement).value }); setShowMenu(false); } }}
                className="w-full text-xs bg-muted rounded px-2 py-0.5 focus:outline-none"
              />
            </div>
          </div>
        )}
      </span>
    </NodeViewWrapper>
  );
}

export const StatusBadgeExtension = Node.create({
  name: 'statusBadge',
  group: 'inline',
  inline: true,
  atom: true,

  addAttributes() {
    return {
      status: { default: 'in-progress' },
      label:  { default: '' },
    };
  },

  parseHTML() { return [{ tag: 'span[data-type="status-badge"]' }]; },
  renderHTML({ HTMLAttributes }) { return ['span', mergeAttributes({ 'data-type': 'status-badge' }, HTMLAttributes)]; },
  addNodeView() { return ReactNodeViewRenderer(StatusBadgeView); },
});
