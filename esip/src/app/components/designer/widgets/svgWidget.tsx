import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useContext, useCallback } from 'react';
import { GripVertical, Upload, ImageIcon } from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

// Remove anything executable from uploaded SVGs
function sanitizeSvg(raw: string): string {
  return raw
    .replace(/<script[\s\S]*?<\/script>/gi, '')
    .replace(/\son\w+\s*=\s*"[^"]*"/gi, '')
    .replace(/\son\w+\s*=\s*'[^']*'/gi, '')
    .replace(/javascript\s*:/gi, '');
}

function SvgView({ node, updateAttributes, selected }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const [dragOver, setDragOver] = useState(false);
  const [error, setError] = useState('');

  const svg = node.attrs.svg as string;

  const handleFile = useCallback((file: File) => {
    if (!file.name.toLowerCase().endsWith('.svg')) {
      setError('Please upload an SVG file (.svg)');
      return;
    }
    setError('');
    const reader = new FileReader();
    reader.onload = (e) => {
      const content = e.target?.result as string;
      updateAttributes({
        svg: sanitizeSvg(content),
        title: node.attrs.title || file.name.replace(/\.svg$/i, ''),
      });
    };
    reader.readAsText(file);
  }, [updateAttributes, node.attrs.title]);

  const onDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragOver(false);
    const file = e.dataTransfer.files[0];
    if (file) handleFile(file);
  }, [handleFile]);

  return (
    <NodeViewWrapper>
      <div
        className={`group relative rounded-xl border border-border bg-card my-3 overflow-hidden shadow-sm ${selected ? 'ring-2 ring-primary/40' : ''} ${dragOver ? 'ring-2 ring-primary border-primary' : ''}`}
        onDragOver={(e) => { e.preventDefault(); e.stopPropagation(); setDragOver(true); }}
        onDragLeave={() => setDragOver(false)}
        onDrop={onDrop}
      >
        {/* Toolbar */}
        <div className="flex items-center gap-2 px-3 py-2 bg-muted/30 border-b border-border">
          {!isPreview && (
            <div data-drag-handle className="w-5 flex items-center justify-center cursor-grab opacity-0 group-hover:opacity-60 transition-opacity">
              <GripVertical className="size-3.5 text-muted-foreground" />
            </div>
          )}
          <span className="text-[9px] font-bold text-slate-700 dark:text-slate-300 bg-slate-100 dark:bg-slate-800 px-2 py-0.5 rounded shrink-0">SVG</span>
          {isPreview ? (
            <span className="flex-1 text-sm font-semibold truncate">{node.attrs.title || 'SVG Diagram'}</span>
          ) : (
            <input
              value={node.attrs.title || ''}
              onChange={(e) => updateAttributes({ title: e.target.value })}
              placeholder="Diagram title…"
              className="flex-1 text-sm font-semibold bg-transparent focus:outline-none placeholder:text-muted-foreground min-w-0"
            />
          )}
          {!isPreview && (
            <label className="ml-auto opacity-0 group-hover:opacity-70 hover:opacity-100 flex items-center gap-1 text-[10px] border border-border rounded px-2 py-0.5 text-muted-foreground hover:text-foreground cursor-pointer transition-opacity">
              <Upload className="size-3" /> {svg ? 'Replace' : 'Import'}
              <input type="file" accept=".svg" className="hidden" onChange={(e) => { const f = e.target.files?.[0]; if (f) handleFile(f); e.currentTarget.value = ''; }} />
            </label>
          )}
        </div>

        {/* Content */}
        <div className="relative" style={{ minHeight: 180 }}>
          {error && <p className="m-3 text-sm text-destructive">{error}</p>}
          {!svg && !error && (
            <div className="flex flex-col items-center justify-center h-44 gap-2 text-muted-foreground pointer-events-none select-none">
              <ImageIcon className="size-10 opacity-25" />
              <p className="text-sm font-medium">Drop an SVG file here, or click Import</p>
            </div>
          )}
          {svg && (
            <div
              className="p-3 overflow-auto"
              style={{ maxHeight: 600 }}
              dangerouslySetInnerHTML={{ __html: svg }}
            />
          )}
        </div>
      </div>
    </NodeViewWrapper>
  );
}

export const SvgImportExtension = Node.create({
  name: 'svgImport',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      title: { default: '' },
      svg:   { default: '' },
    };
  },

  parseHTML()  { return [{ tag: 'div[data-type="svg-import"]' }]; },
  renderHTML({ HTMLAttributes }) {
    return ['div', mergeAttributes({ 'data-type': 'svg-import' }, HTMLAttributes)];
  },
  addNodeView() { return ReactNodeViewRenderer(SvgView); },
});
