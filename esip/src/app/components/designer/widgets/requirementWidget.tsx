import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useContext } from 'react';
import { GripVertical, Sparkles } from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

const REQ_TYPES = ['SHALL', 'SHOULD', 'MAY'] as const;
const PRIORITIES = ['HIGH', 'MEDIUM', 'LOW'] as const;
const STATUSES = ['OPEN', 'IN-PROGRESS', 'VERIFIED', 'CLOSED'] as const;

const TYPE_STYLE: Record<string, string> = {
  SHALL: 'bg-blue-50 text-blue-700 dark:bg-blue-950/50 dark:text-blue-400 border border-blue-200 dark:border-blue-800',
  SHOULD: 'bg-violet-50 text-violet-700 dark:bg-violet-950/50 dark:text-violet-400 border border-violet-200 dark:border-violet-800',
  MAY: 'bg-slate-50 text-slate-600 dark:bg-slate-800/50 dark:text-slate-400 border border-slate-200 dark:border-slate-700',
};

const PRI_STYLE: Record<string, string> = {
  HIGH: 'bg-red-50 text-red-700 dark:bg-red-950/50 dark:text-red-400',
  MEDIUM: 'bg-amber-50 text-amber-700 dark:bg-amber-950/50 dark:text-amber-400',
  LOW: 'bg-green-50 text-green-600 dark:bg-green-950/50 dark:text-green-400',
};

const STATUS_STYLE: Record<string, string> = {
  OPEN: 'bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-400',
  'IN-PROGRESS': 'bg-blue-100 text-blue-700 dark:bg-blue-900/50 dark:text-blue-400',
  VERIFIED: 'bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-400',
  CLOSED: 'bg-slate-100 text-slate-500 dark:bg-slate-800 dark:text-slate-500',
};

function cycle<T>(arr: readonly T[], current: T): T {
  const idx = arr.indexOf(current);
  return arr[(idx + 1) % arr.length];
}

function RequirementView({ node, updateAttributes, selected, getPos }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const [editId, setEditId] = useState(false);
  const [editDesc, setEditDesc] = useState(false);
  const [editTags, setEditTags] = useState(false);

  const reqType = node.attrs.reqType || 'SHALL';
  const priority = node.attrs.priority || 'HIGH';
  const status = node.attrs.status || 'OPEN';

  return (
    <NodeViewWrapper>
      <div className={`group relative rounded-xl border border-border bg-card my-3 overflow-hidden shadow-sm ${selected ? 'ring-2 ring-primary/40' : ''}`}>
        {/* Header */}
        <div className="flex items-center gap-2 px-3 py-2 bg-muted/40 border-b border-border">
          {!isPreview && (
            <div className="flex items-center justify-center w-5 cursor-grab opacity-0 group-hover:opacity-60 transition-opacity" data-drag-handle>
              <GripVertical className="size-3.5 text-muted-foreground" />
            </div>
          )}
          {!isPreview && editId ? (
            <input
              autoFocus
              value={node.attrs.reqId}
              onChange={(e) => updateAttributes({ reqId: e.target.value })}
              onBlur={() => setEditId(false)}
              onKeyDown={(e) => e.key === 'Enter' && setEditId(false)}
              className="text-xs font-bold font-mono text-primary bg-transparent border-b border-primary focus:outline-none w-24"
            />
          ) : (
            <span
              className={`text-xs font-bold font-mono text-primary ${!isPreview ? 'cursor-text' : ''}`}
              onClick={!isPreview ? () => setEditId(true) : undefined}
            >
              {node.attrs.reqId || 'REQ-001'}
            </span>
          )}
          <div className="flex items-center gap-1.5 ml-auto flex-wrap">
            <span className={`text-[10px] font-bold px-2 py-0.5 rounded-full ${TYPE_STYLE[reqType]} ${!isPreview ? 'cursor-pointer' : ''}`}
              onClick={!isPreview ? () => updateAttributes({ reqType: cycle(REQ_TYPES, reqType) }) : undefined}>
              {reqType}
            </span>
            <span className={`text-[10px] font-bold px-2 py-0.5 rounded-full ${PRI_STYLE[priority]} ${!isPreview ? 'cursor-pointer' : ''}`}
              onClick={!isPreview ? () => updateAttributes({ priority: cycle(PRIORITIES, priority) }) : undefined}>
              {priority}
            </span>
            <span className={`text-[10px] font-semibold px-2 py-0.5 rounded-full ${STATUS_STYLE[status]} ${!isPreview ? 'cursor-pointer' : ''}`}
              onClick={!isPreview ? () => updateAttributes({ status: cycle(STATUSES, status) }) : undefined}>
              {status}
            </span>
            {!isPreview && (
              <button
                onMouseDown={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  document.dispatchEvent(new CustomEvent('ask-sandeep-target', {
                    bubbles: true,
                    detail: { pos: getPos?.(), nodeType: 'requirement', attrs: { ...node.attrs }, targetField: 'description', displayName: `${node.attrs.reqId || 'REQ'}: requirement` },
                  }));
                }}
                className="opacity-0 group-hover:opacity-70 hover:opacity-100 rounded p-0.5 hover:bg-primary/10 transition-all"
                title="Ask Sandeep AI"
              >
                <Sparkles className="size-3 text-primary" />
              </button>
            )}
          </div>
        </div>

        {/* Description */}
        <div className="px-4 pt-3 pb-2">
          {!isPreview && editDesc ? (
            <textarea
              autoFocus
              value={node.attrs.description}
              onChange={(e) => updateAttributes({ description: e.target.value })}
              onBlur={() => setEditDesc(false)}
              rows={3}
              className="w-full text-sm bg-transparent border-0 focus:outline-none resize-none text-foreground leading-relaxed"
            />
          ) : (
            <p
              className={`text-sm text-foreground leading-relaxed min-h-[1.5rem] ${!isPreview ? 'cursor-text' : ''}`}
              onClick={!isPreview ? () => setEditDesc(true) : undefined}
            >
              {node.attrs.description || (
                <span className="italic text-muted-foreground">
                  {!isPreview ? `The system ${reqType} [describe the requirement]…` : ''}
                </span>
              )}
            </p>
          )}
        </div>

        {/* Tags */}
        <div className="flex items-center gap-2 px-4 pb-3">
          {!isPreview && editTags ? (
            <input
              autoFocus
              value={node.attrs.tags}
              onChange={(e) => updateAttributes({ tags: e.target.value })}
              onBlur={() => setEditTags(false)}
              onKeyDown={(e) => e.key === 'Enter' && setEditTags(false)}
              placeholder="Security, Auth, Sprint 2 (comma-separated)"
              className="flex-1 text-xs bg-transparent border-b border-border focus:outline-none text-muted-foreground"
            />
          ) : (
            <div
              className={`flex flex-wrap gap-1 ${!isPreview ? 'cursor-text' : ''}`}
              onClick={!isPreview ? () => setEditTags(true) : undefined}
            >
              {node.attrs.tags
                ? node.attrs.tags.split(',').map((tag: string) => tag.trim()).filter(Boolean).map((tag: string) => (
                    <span key={tag} className="text-[10px] bg-muted text-muted-foreground px-2 py-0.5 rounded-full">
                      {tag}
                    </span>
                  ))
                : !isPreview && <span className="text-[10px] text-muted-foreground/50 italic">+ add tags</span>
              }
            </div>
          )}
        </div>
      </div>
    </NodeViewWrapper>
  );
}

export const RequirementExtension = Node.create({
  name: 'requirement',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      reqId:       { default: 'REQ-001' },
      reqType:     { default: 'SHALL' },
      priority:    { default: 'HIGH' },
      status:      { default: 'OPEN' },
      description: { default: '' },
      tags:        { default: '' },
    };
  },

  parseHTML() { return [{ tag: 'div[data-type="requirement"]' }]; },
  renderHTML({ HTMLAttributes }) { return ['div', mergeAttributes({ 'data-type': 'requirement' }, HTMLAttributes)]; },
  addNodeView() { return ReactNodeViewRenderer(RequirementView); },
});
