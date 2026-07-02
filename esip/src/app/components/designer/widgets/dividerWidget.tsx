import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState } from 'react';
import { GripVertical } from 'lucide-react';

const STYLES = {
  line:   { cls: 'border-t border-border', label: false },
  dashed: { cls: 'border-t border-dashed border-border', label: false },
  thick:  { cls: 'border-t-2 border-border', label: false },
  label:  { cls: 'border-t border-border', label: true },
} as const;

type DividerStyle = keyof typeof STYLES;

function DividerView({ node, updateAttributes, selected }: NodeViewProps) {
  const [editLabel, setEditLabel] = useState(false);
  const style = (node.attrs.style || 'line') as DividerStyle;
  const cfg = STYLES[style];

  return (
    <NodeViewWrapper>
      <div className={`group relative my-6 ${selected ? 'opacity-80' : ''}`}>
        <div className="flex items-center justify-center w-5 absolute -left-6 top-1/2 -translate-y-1/2 cursor-grab opacity-0 group-hover:opacity-60 transition-opacity" data-drag-handle>
          <GripVertical className="size-3.5 text-muted-foreground" />
        </div>

        {cfg.label ? (
          <div className="flex items-center gap-3">
            <div className={`flex-1 ${cfg.cls}`} />
            {editLabel ? (
              <input
                autoFocus
                value={node.attrs.label}
                onChange={(e) => updateAttributes({ label: e.target.value })}
                onBlur={() => setEditLabel(false)}
                onKeyDown={(e) => e.key === 'Enter' && setEditLabel(false)}
                className="text-xs font-semibold text-muted-foreground bg-transparent border-b border-border focus:outline-none px-1 shrink-0"
              />
            ) : (
              <span
                className="text-xs font-semibold text-muted-foreground uppercase tracking-widest shrink-0 cursor-text"
                onClick={() => setEditLabel(true)}
              >
                {node.attrs.label || 'Section'}
              </span>
            )}
            <div className={`flex-1 ${cfg.cls}`} />
          </div>
        ) : (
          <div className={cfg.cls} />
        )}

        {/* Style switcher — visible on hover */}
        <div className="absolute right-0 -top-5 opacity-0 group-hover:opacity-100 transition-opacity flex gap-1">
          {(Object.keys(STYLES) as DividerStyle[]).map(s => (
            <button
              key={s}
              onClick={() => updateAttributes({ style: s })}
              className={`text-[9px] px-1.5 py-0.5 rounded border transition-colors ${style === s ? 'bg-primary text-primary-foreground border-primary' : 'bg-card border-border text-muted-foreground hover:border-primary/50'}`}
            >
              {s}
            </button>
          ))}
        </div>
      </div>
    </NodeViewWrapper>
  );
}

export const DividerExtension = Node.create({
  name: 'widgetDivider',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      style: { default: 'line' },
      label: { default: 'Section' },
    };
  },

  parseHTML() { return [{ tag: 'div[data-type="widget-divider"]' }]; },
  renderHTML({ HTMLAttributes }) { return ['div', mergeAttributes({ 'data-type': 'widget-divider' }, HTMLAttributes)]; },
  addNodeView() { return ReactNodeViewRenderer(DividerView); },
});
