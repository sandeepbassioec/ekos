import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useContext, useCallback, useRef, useEffect } from 'react';
import { GripVertical, Upload, PenLine } from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

// ─── Types ────────────────────────────────────────────────────────────────────
interface ExElement {
  type: string;
  x: number; y: number;
  width: number; height: number;
  angle?: number;
  strokeColor?: string;
  backgroundColor?: string;
  fillStyle?: string;
  strokeWidth?: number;
  strokeStyle?: string;
  opacity?: number;
  text?: string;
  fontSize?: number;
  textAlign?: string;
  points?: [number, number][];
  endArrowhead?: string | null;
  startArrowhead?: string | null;
}

// ─── SVG renderer ─────────────────────────────────────────────────────────────
function esc(s: string) {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function renderExcalidraw(elements: ExElement[], containerW: number): string {
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;

  for (const el of elements) {
    if (el.type === 'arrow' || el.type === 'line') {
      if (el.points) {
        for (const [px, py] of el.points) {
          minX = Math.min(minX, el.x + px); minY = Math.min(minY, el.y + py);
          maxX = Math.max(maxX, el.x + px); maxY = Math.max(maxY, el.y + py);
        }
      }
    } else {
      minX = Math.min(minX, el.x); minY = Math.min(minY, el.y);
      maxX = Math.max(maxX, el.x + (el.width || 0)); maxY = Math.max(maxY, el.y + (el.height || 0));
    }
  }
  if (!isFinite(minX)) { minX = 0; minY = 0; maxX = 800; maxY = 600; }

  const PAD = 24;
  const cW = maxX - minX + PAD * 2;
  const cH = maxY - minY + PAD * 2;
  const scale = containerW / cW;
  const W = containerW;
  const H = cH * scale;

  const tx = (v: number) => ((v - minX + PAD) * scale).toFixed(1);
  const ty = (v: number) => ((v - minY + PAD) * scale).toFixed(1);
  const ts = (v: number) => (v * scale).toFixed(1);

  const parts: string[] = [];

  for (const el of elements) {
    if ((el.opacity ?? 100) < 5) continue;

    const stroke = el.strokeColor || '#1e1e1e';
    const bg = el.backgroundColor;
    const fill = (!bg || bg === 'transparent' || bg === 'none' || el.fillStyle === 'none')
      ? 'none'
      : el.fillStyle === 'solid' ? bg : bg + '55';
    const sw = ts((el.strokeWidth || 1));
    const dash = el.strokeStyle === 'dashed' ? ` stroke-dasharray="${ts(8)},${ts(4)}"`
                : el.strokeStyle === 'dotted' ? ` stroke-dasharray="${ts(2)},${ts(4)}"` : '';
    const op = el.opacity !== undefined && el.opacity < 100 ? ` opacity="${(el.opacity / 100).toFixed(2)}"` : '';
    const cx = el.x + (el.width || 0) / 2;
    const cy = el.y + (el.height || 0) / 2;
    const rot = el.angle ? ` transform="rotate(${(el.angle * 180 / Math.PI).toFixed(1)} ${tx(cx)} ${ty(cy)})"` : '';
    const base = `fill="${fill}" stroke="${stroke}" stroke-width="${sw}"${dash}${op}${rot}`;

    switch (el.type) {
      case 'rectangle':
        parts.push(`<rect x="${tx(el.x)}" y="${ty(el.y)}" width="${ts(el.width)}" height="${ts(el.height)}" rx="4" ${base}/>`);
        break;

      case 'ellipse':
        parts.push(`<ellipse cx="${tx(el.x + el.width / 2)}" cy="${ty(el.y + el.height / 2)}" rx="${ts(el.width / 2)}" ry="${ts(el.height / 2)}" ${base}/>`);
        break;

      case 'diamond': {
        const dcx = el.x + el.width / 2, dcy = el.y + el.height / 2;
        const pts = `${tx(dcx)},${ty(el.y)} ${tx(el.x + el.width)},${ty(dcy)} ${tx(dcx)},${ty(el.y + el.height)} ${tx(el.x)},${ty(dcy)}`;
        parts.push(`<polygon points="${pts}" ${base}/>`);
        break;
      }

      case 'text': {
        if (!el.text) break;
        const fs = ts(el.fontSize || 20);
        const anchor = el.textAlign === 'center' ? 'middle' : el.textAlign === 'right' ? 'end' : 'start';
        const lh = ts((el.fontSize || 20) * 1.3);
        const lines = el.text.split('\n');
        const tspans = lines.map((l, i) =>
          `<tspan x="${tx(el.x)}" dy="${i === 0 ? '0' : lh}">${esc(l || ' ')}</tspan>`
        ).join('');
        parts.push(`<text x="${tx(el.x)}" y="${ty(el.y + (el.fontSize || 20))}" font-size="${fs}" font-family="system-ui,sans-serif" fill="${stroke}" text-anchor="${anchor}"${op}>${tspans}</text>`);
        break;
      }

      case 'arrow':
      case 'line': {
        if (!el.points || el.points.length < 2) break;
        const pStr = el.points.map(([px, py]) => `${tx(el.x + px)},${ty(el.y + py)}`).join(' ');
        const me = el.type === 'arrow' && el.endArrowhead !== null ? ` marker-end="url(#exarr)"` : '';
        const ms = el.type === 'arrow' && el.startArrowhead ? ` marker-start="url(#exarr)"` : '';
        parts.push(`<polyline points="${pStr}" fill="none" stroke="${stroke}" stroke-width="${sw}"${dash}${me}${ms}${op}/>`);
        break;
      }

      case 'freedraw': {
        if (!el.points || el.points.length < 2) break;
        const d = el.points.map(([px, py], i) => `${i === 0 ? 'M' : 'L'}${tx(el.x + px)},${ty(el.y + py)}`).join(' ');
        parts.push(`<path d="${d}" fill="none" stroke="${stroke}" stroke-width="${sw}" stroke-linejoin="round" stroke-linecap="round"${op}/>`);
        break;
      }
    }
  }

  return `<svg xmlns="http://www.w3.org/2000/svg" width="${W.toFixed(0)}" height="${H.toFixed(0)}" viewBox="0 0 ${W.toFixed(0)} ${H.toFixed(0)}" style="max-width:100%;display:block;">
  <defs>
    <marker id="exarr" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
      <path d="M0,0 L0,6 L9,3 z" fill="context-stroke"/>
    </marker>
  </defs>
  <rect width="${W.toFixed(0)}" height="${H.toFixed(0)}" fill="white"/>
  ${parts.join('\n  ')}
</svg>`;
}

// ─── NodeView ─────────────────────────────────────────────────────────────────
function ExcalidrawView({ node, updateAttributes, selected }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const [dragOver, setDragOver] = useState(false);
  const [error, setError] = useState('');
  const containerRef = useRef<HTMLDivElement>(null);
  const [svgHtml, setSvgHtml] = useState('');

  const rawData = node.attrs.data as string;
  const elements: ExElement[] = rawData ? JSON.parse(rawData) : [];

  useEffect(() => {
    if (!elements.length || !containerRef.current) return;
    const w = containerRef.current.clientWidth || 700;
    setSvgHtml(renderExcalidraw(elements, w));

    const ro = new ResizeObserver(entries => {
      const cw = entries[0]?.contentRect.width;
      if (cw) setSvgHtml(renderExcalidraw(elements, cw));
    });
    ro.observe(containerRef.current);
    return () => ro.disconnect();
  }, [rawData]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleFile = useCallback((file: File) => {
    if (!file.name.toLowerCase().endsWith('.excalidraw')) {
      setError('Please upload an Excalidraw file (.excalidraw)');
      return;
    }
    setError('');
    const reader = new FileReader();
    reader.onload = (e) => {
      try {
        const json = JSON.parse(e.target?.result as string);
        const els: ExElement[] = (json.elements || []).filter((el: ExElement) => !('isDeleted' in el && (el as any).isDeleted));
        updateAttributes({
          data: JSON.stringify(els),
          title: node.attrs.title || file.name.replace(/\.excalidraw$/i, ''),
        });
      } catch {
        setError('Could not parse this Excalidraw file');
      }
    };
    reader.readAsText(file);
  }, [updateAttributes, node.attrs.title]);

  const onDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault(); e.stopPropagation();
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
        <div className="flex items-center gap-2 px-3 py-2 bg-muted/30 border-b border-border">
          {!isPreview && (
            <div data-drag-handle className="w-5 flex items-center justify-center cursor-grab opacity-0 group-hover:opacity-60 transition-opacity">
              <GripVertical className="size-3.5 text-muted-foreground" />
            </div>
          )}
          <span className="text-[9px] font-bold text-purple-700 dark:text-purple-300 bg-purple-100 dark:bg-purple-900/40 px-2 py-0.5 rounded shrink-0">Excalidraw</span>
          {isPreview ? (
            <span className="flex-1 text-sm font-semibold truncate">{node.attrs.title || 'Excalidraw Diagram'}</span>
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
              <Upload className="size-3" /> {rawData ? 'Replace' : 'Import'}
              <input type="file" accept=".excalidraw" className="hidden" onChange={(e) => { const f = e.target.files?.[0]; if (f) handleFile(f); e.currentTarget.value = ''; }} />
            </label>
          )}
        </div>

        <div ref={containerRef} className="relative" style={{ minHeight: 180 }}>
          {error && <p className="m-3 text-sm text-destructive">{error}</p>}
          {!rawData && !error && (
            <div className="flex flex-col items-center justify-center h-44 gap-2 text-muted-foreground pointer-events-none select-none">
              <PenLine className="size-10 opacity-25" />
              <p className="text-sm font-medium">Drop an .excalidraw file here, or click Import</p>
              <p className="text-xs opacity-60">Export from excalidraw.com: Save as → .excalidraw</p>
            </div>
          )}
          {svgHtml && (
            <div className="p-3 overflow-auto" style={{ maxHeight: 600 }} dangerouslySetInnerHTML={{ __html: svgHtml }} />
          )}
        </div>
      </div>
    </NodeViewWrapper>
  );
}

export const ExcalidrawExtension = Node.create({
  name: 'excalidraw',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      title: { default: '' },
      data:  { default: '' },
    };
  },

  parseHTML()  { return [{ tag: 'div[data-type="excalidraw"]' }]; },
  renderHTML({ HTMLAttributes }) {
    return ['div', mergeAttributes({ 'data-type': 'excalidraw' }, HTMLAttributes)];
  },
  addNodeView() { return ReactNodeViewRenderer(ExcalidrawView); },
});
