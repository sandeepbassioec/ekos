import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useContext } from 'react';
import { AlertCircle, Info, CheckCircle, AlertTriangle, GripVertical, Sparkles } from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

const CONFIGS = {
  info:    { bg: 'bg-blue-50 dark:bg-blue-950/30',   border: 'border-blue-200 dark:border-blue-800',   bar: 'bg-blue-500',   icon: Info,          iconCls: 'text-blue-500',   titleCls: 'text-blue-700 dark:text-blue-400',   label: 'Note' },
  warning: { bg: 'bg-amber-50 dark:bg-amber-950/30', border: 'border-amber-200 dark:border-amber-800', bar: 'bg-amber-500',  icon: AlertTriangle,  iconCls: 'text-amber-500',  titleCls: 'text-amber-700 dark:text-amber-400', label: 'Warning' },
  success: { bg: 'bg-green-50 dark:bg-green-950/30', border: 'border-green-200 dark:border-green-800', bar: 'bg-green-500',  icon: CheckCircle,   iconCls: 'text-green-500',  titleCls: 'text-green-700 dark:text-green-400', label: 'Decision' },
  danger:  { bg: 'bg-red-50 dark:bg-red-950/30',     border: 'border-red-200 dark:border-red-800',     bar: 'bg-red-500',    icon: AlertCircle,   iconCls: 'text-red-500',    titleCls: 'text-red-700 dark:text-red-400',     label: 'Risk' },
} as const;

type CalloutType = keyof typeof CONFIGS;

function CalloutView({ node, updateAttributes, selected, getPos }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const type = (node.attrs.type || 'info') as CalloutType;
  const cfg = CONFIGS[type];
  const Icon = cfg.icon;
  const [editTitle, setEditTitle] = useState(false);
  const [editBody, setEditBody] = useState(false);

  return (
    <NodeViewWrapper>
      <div className={`group relative flex rounded-lg border my-3 overflow-hidden ${cfg.bg} ${cfg.border} ${selected ? 'ring-2 ring-primary/40' : ''}`}>
        {!isPreview && (
          <div className="flex items-center justify-center w-6 shrink-0 cursor-grab opacity-0 group-hover:opacity-60 transition-opacity hover:opacity-100" data-drag-handle>
            <GripVertical className="size-3.5 text-muted-foreground" />
          </div>
        )}
        <div className={`w-1 shrink-0 ${cfg.bar}`} />
        <div className="flex-1 px-3 py-2.5 min-w-0">
          <div className="flex items-start gap-2 mb-1">
            <Icon className={`size-4 shrink-0 mt-0.5 ${cfg.iconCls}`} />
            <div className="flex-1 min-w-0">
              {!isPreview && editTitle ? (
                <input
                  autoFocus
                  value={node.attrs.title}
                  onChange={(e) => updateAttributes({ title: e.target.value })}
                  onBlur={() => setEditTitle(false)}
                  onKeyDown={(e) => { if (e.key === 'Enter') { e.preventDefault(); setEditTitle(false); } }}
                  className={`text-sm font-semibold bg-transparent border-b border-current focus:outline-none w-full ${cfg.titleCls}`}
                />
              ) : (
                <span
                  className={`text-sm font-semibold block ${cfg.titleCls} ${!isPreview ? 'cursor-text' : ''}`}
                  onClick={!isPreview ? () => setEditTitle(true) : undefined}
                >
                  {node.attrs.title || cfg.label}
                </span>
              )}
            </div>
            {!isPreview && <div className="flex items-center gap-1.5 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                onMouseDown={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  document.dispatchEvent(new CustomEvent('ask-sandeep-target', {
                    bubbles: true,
                    detail: { pos: getPos?.(), nodeType: 'callout', attrs: { ...node.attrs }, targetField: 'body', displayName: `Callout: ${node.attrs.title || cfg.label}` },
                  }));
                }}
                className="rounded p-0.5 hover:bg-primary/10 hover:text-primary transition-colors"
                title="Ask Sandeep AI"
              >
                <Sparkles className="size-3 text-primary/60" />
              </button>
              <select
                value={type}
                onChange={(e) => { updateAttributes({ type: e.target.value }); }}
                className="text-[10px] bg-white/70 dark:bg-black/30 border border-current/20 rounded px-1.5 py-0.5 cursor-pointer"
                onMouseDown={(e) => e.stopPropagation()}
              >
                <option value="info">ℹ️ Info</option>
                <option value="warning">⚠️ Warning</option>
                <option value="success">✅ Decision</option>
                <option value="danger">🚫 Risk</option>
              </select>
            </div>}
          </div>
          {!isPreview && editBody ? (
            <textarea
              autoFocus
              value={node.attrs.body}
              onChange={(e) => updateAttributes({ body: e.target.value })}
              onBlur={() => setEditBody(false)}
              rows={3}
              className="w-full text-sm bg-transparent border-0 focus:outline-none resize-none text-foreground/80 leading-relaxed pl-6"
            />
          ) : (
            <p
              className={`text-sm text-foreground/70 leading-relaxed pl-6 min-h-[1.25rem] ${!isPreview ? 'cursor-text' : ''}`}
              onClick={!isPreview ? () => setEditBody(true) : undefined}
            >
              {node.attrs.body || <span className="italic text-muted-foreground">{!isPreview ? 'Click to add content…' : ''}</span>}
            </p>
          )}
        </div>
      </div>
    </NodeViewWrapper>
  );
}

export const CalloutExtension = Node.create({
  name: 'callout',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      type:  { default: 'info' },
      title: { default: '' },
      body:  { default: '' },
    };
  },

  parseHTML() { return [{ tag: 'div[data-type="callout"]' }]; },
  renderHTML({ HTMLAttributes }) { return ['div', mergeAttributes({ 'data-type': 'callout' }, HTMLAttributes)]; },
  addNodeView() { return ReactNodeViewRenderer(CalloutView); },
});
