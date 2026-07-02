import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useEffect, useRef, useContext, useCallback } from 'react';
import { GripVertical, Upload, Layers, AlertCircle, FileSpreadsheet } from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

// ─── Types ────────────────────────────────────────────────────────────────────

interface VisioShape {
  id: string;
  text: string;
  isConnector: boolean;
  pinX: number; pinY: number;
  width: number; height: number;
  beginX: number; beginY: number;
  endX: number; endY: number;
  fillColor: string;
  lineColor: string;
  isCircle: boolean;
}

interface VisioPage {
  name: string;
  pageWidth: number;  // inches
  pageHeight: number; // inches
  shapes: VisioShape[];
}

interface VisioData {
  pages: VisioPage[];
  fileName: string;
}

// ─── VSDX Parser ──────────────────────────────────────────────────────────────

const VISIO_NS = 'http://schemas.microsoft.com/office/visio/2012/main';

// Namespace-aware query helpers with fallback to unqualified tags
function gEls(ctx: Element | Document, name: string): Element[] {
  const ns = Array.from(ctx.getElementsByTagNameNS(VISIO_NS, name));
  if (ns.length) return ns;
  return Array.from(ctx.getElementsByTagName(name));
}

function gEl(ctx: Element | Document, name: string): Element | undefined {
  return gEls(ctx, name)[0];
}

function gVal(ctx: Element | Document, name: string, fallback = ''): string {
  return gEl(ctx, name)?.textContent?.trim() || fallback;
}

function gFloat(ctx: Element | Document, name: string, fallback = 0): number {
  const v = gVal(ctx, name, '');
  return v ? (parseFloat(v) || fallback) : fallback;
}

function parseVisioColor(raw: string): string {
  if (!raw || raw.toUpperCase().startsWith('THEME') || raw === 'Inherit') return '';
  if (raw.startsWith('#')) {
    // Ensure 6-char hex
    const hex = raw.slice(1);
    return hex.length === 3
      ? '#' + hex.split('').map(c => c + c).join('')
      : raw;
  }
  // RGB(r,g,b) notation
  const m = raw.match(/RGB\s*\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)/i);
  if (m) {
    return '#' + [m[1], m[2], m[3]].map(n => parseInt(n).toString(16).padStart(2, '0')).join('');
  }
  return '';
}

function extractShapeText(el: Element): string {
  const textEl = gEl(el, 'Text');
  if (!textEl) return '';
  // Collect raw text nodes, skip para/char/field marker elements
  return Array.from(textEl.childNodes)
    .filter(n => n.nodeType === 3) // Node.TEXT_NODE = 3
    .map(n => n.textContent || '')
    .join(' ')
    .replace(/\s+/g, ' ')
    .trim();
}

function parseShapeEl(el: Element): VisioShape | null {
  // Skip shapes without any positioning info
  const xform   = gEl(el, 'XForm');
  const xform1d = gEl(el, 'XForm1D');
  if (!xform && !xform1d) return null;

  const id = el.getAttribute('ID') || '';
  const isConnector = !!xform1d;

  const pinX   = xform ? gFloat(xform, 'PinX')   : 0;
  const pinY   = xform ? gFloat(xform, 'PinY')   : 0;
  const width  = xform ? gFloat(xform, 'Width', 1) : 1;
  const height = xform ? gFloat(xform, 'Height', 0.5) : 0.5;

  const beginX = xform1d ? gFloat(xform1d, 'BeginX') : 0;
  const beginY = xform1d ? gFloat(xform1d, 'BeginY') : 0;
  const endX   = xform1d ? gFloat(xform1d, 'EndX')   : 0;
  const endY   = xform1d ? gFloat(xform1d, 'EndY')   : 0;

  const fillEl = gEl(el, 'Fill');
  const lineEl = gEl(el, 'Line');
  const fillColor = fillEl ? parseVisioColor(gVal(fillEl, 'FillForegnd')) : '';
  const lineColor = lineEl ? parseVisioColor(gVal(lineEl, 'LineColor'))  : '';

  // Heuristic: circle/ellipse shapes often have "Ellipse" or "Circle" in NameU
  const nameU = (el.getAttribute('NameU') || el.getAttribute('Name') || '').toLowerCase();
  const isCircle = nameU.includes('ellipse') || nameU.includes('circle') ||
    (!isConnector && Math.abs(width - height) < 0.15 && width <= 1.2);

  return {
    id, text: extractShapeText(el), isConnector,
    pinX, pinY, width, height,
    beginX, beginY, endX, endY,
    fillColor, lineColor, isCircle,
  };
}

async function parseVsdx(buffer: ArrayBuffer): Promise<VisioPage[]> {
  const { default: JSZip } = await import('jszip');
  const zip = await JSZip.loadAsync(buffer);

  // Discover all page XML files (visio/pages/pageN.xml)
  const pageFiles = Object.keys(zip.files)
    .filter(f => /^visio\/pages\/page\d+\.xml$/i.test(f))
    .sort((a, b) => {
      const na = parseInt(a.match(/(\d+)/)?.[1] || '0');
      const nb = parseInt(b.match(/(\d+)/)?.[1] || '0');
      return na - nb;
    });

  if (pageFiles.length === 0) throw new Error('No diagram pages found. This may not be a valid .vsdx file.');

  // Get human-readable page names from pages.xml
  let pageNames: string[] = [];
  try {
    const pagesXmlText = await zip.file('visio/pages/pages.xml')?.async('text');
    if (pagesXmlText) {
      const pagesDoc = new DOMParser().parseFromString(pagesXmlText, 'text/xml');
      pageNames = gEls(pagesDoc, 'Page').map((p, i) =>
        p.getAttribute('Name') || p.getAttribute('NameU') || `Page ${i + 1}`
      );
    }
  } catch { /* use index-based names */ }

  const pages: VisioPage[] = [];

  for (let i = 0; i < pageFiles.length; i++) {
    const xmlText = await zip.file(pageFiles[i])?.async('text');
    if (!xmlText) continue;

    const doc = new DOMParser().parseFromString(xmlText, 'text/xml');

    // Page dimensions — look in PageSheet > PageProps, default to US Letter landscape
    const pageWidth  = gFloat(doc, 'PageWidth', 0)  || 11;
    const pageHeight = gFloat(doc, 'PageHeight', 0) || 8.5;

    // Parse Shape elements that have geometric positioning
    const rawShapes = gEls(doc, 'Shape');
    const shapes = rawShapes
      .map(parseShapeEl)
      .filter((s): s is VisioShape => s !== null);

    pages.push({
      name: pageNames[i] || `Page ${i + 1}`,
      pageWidth,
      pageHeight,
      shapes,
    });
  }

  return pages;
}

// ─── SVG Renderer ─────────────────────────────────────────────────────────────

const PPI = 72; // pixels per inch at 72 dpi

function xmlEscape(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function renderVisioPage(page: VisioPage, containerPx: number): string {
  // Scale to fit the container width
  const scale = containerPx / (page.pageWidth * PPI);
  const W = (page.pageWidth  * PPI * scale).toFixed(0);
  const H = (page.pageHeight * PPI * scale).toFixed(0);

  const connectors: string[] = [];
  const shapes: string[] = [];

  for (const s of page.shapes) {
    if (s.isConnector) {
      // Visio Y origin is bottom-left → flip for SVG
      const x1 = (s.beginX * PPI * scale).toFixed(1);
      const y1 = ((page.pageHeight - s.beginY) * PPI * scale).toFixed(1);
      const x2 = (s.endX * PPI * scale).toFixed(1);
      const y2 = ((page.pageHeight - s.endY) * PPI * scale).toFixed(1);
      connectors.push(
        `<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="#64748b" stroke-width="1.2" marker-end="url(#arr)"/>`
      );
    } else {
      if (s.width <= 0 || s.height <= 0) continue;

      // Top-left corner in SVG coords
      const x = ((s.pinX - s.width  / 2) * PPI * scale);
      const y = ((page.pageHeight - s.pinY - s.height / 2) * PPI * scale);
      const w = (s.width  * PPI * scale);
      const h = (s.height * PPI * scale);

      const fill   = s.fillColor || '#eef2ff';
      const stroke = s.lineColor || '#6366f1';
      const fs = Math.max(6, Math.min(11, Math.min(w, h) * 0.28)).toFixed(1);
      const cx = (x + w / 2).toFixed(1);
      const cy = (y + h / 2).toFixed(1);

      let geom: string;
      if (s.isCircle) {
        geom = `<ellipse cx="${cx}" cy="${cy}" rx="${(w / 2).toFixed(1)}" ry="${(h / 2).toFixed(1)}" fill="${fill}" stroke="${stroke}" stroke-width="1"/>`;
      } else {
        const rx = Math.min(4, w * 0.08).toFixed(1);
        geom = `<rect x="${x.toFixed(1)}" y="${y.toFixed(1)}" width="${w.toFixed(1)}" height="${h.toFixed(1)}" rx="${rx}" fill="${fill}" stroke="${stroke}" stroke-width="1"/>`;
      }

      const label = s.text
        ? `<text x="${cx}" y="${cy}" text-anchor="middle" dominant-baseline="middle" font-size="${fs}" font-family="system-ui,sans-serif" fill="#1e293b">${xmlEscape(s.text.slice(0, 80))}</text>`
        : '';

      shapes.push(geom + label);
    }
  }

  return `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}" style="max-width:100%;display:block;">
  <defs>
    <marker id="arr" markerWidth="7" markerHeight="7" refX="5" refY="3" orient="auto">
      <path d="M0,0 L0,6 L8,3 z" fill="#64748b"/>
    </marker>
  </defs>
  <rect width="${W}" height="${H}" fill="white"/>
  ${connectors.join('\n  ')}
  ${shapes.join('\n  ')}
</svg>`;
}

// ─── Widget view ──────────────────────────────────────────────────────────────

function VisioView({ node, updateAttributes, selected }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const [activePage, setActivePage] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [dragOver, setDragOver] = useState(false);
  const [svgHtml, setSvgHtml] = useState('');
  const containerRef = useRef<HTMLDivElement>(null);

  // Parse stored JSON safely
  const data: VisioData | null = (() => {
    try { return node.attrs.data ? JSON.parse(node.attrs.data) as VisioData : null; } catch { return null; }
  })();

  const pages = data?.pages ?? [];
  const page  = pages[activePage] ?? null;

  // Re-render SVG when active page or container width changes
  useEffect(() => {
    if (!page) { setSvgHtml(''); return; }
    const render = () => {
      const w = containerRef.current?.clientWidth;
      if (!w) return;
      setSvgHtml(renderVisioPage(page, Math.max(w - 24, 300)));
    };
    render();
    const ro = new ResizeObserver(render);
    if (containerRef.current) ro.observe(containerRef.current);
    return () => ro.disconnect();
  }, [page, activePage]);

  // Reset active page when data changes (new file loaded)
  useEffect(() => { setActivePage(0); }, [data?.fileName]);

  const handleFile = useCallback(async (file: File) => {
    const ext = file.name.split('.').pop()?.toLowerCase();
    if (ext === 'vsd') {
      setError('Binary .vsd format cannot be read in the browser. Open the file in Visio (or Visio Viewer) and use File → Save As → .vsdx to convert it first.');
      return;
    }
    if (ext !== 'vsdx') {
      setError('Please upload a .vsdx or .vsd Visio file.');
      return;
    }
    setLoading(true);
    setError('');
    try {
      const buffer = await file.arrayBuffer();
      const parsedPages = await parseVsdx(buffer);
      updateAttributes({
        data: JSON.stringify({ pages: parsedPages, fileName: file.name } satisfies VisioData),
        title: node.attrs.title || file.name.replace(/\.vsdx$/i, ''),
      });
    } catch (e) {
      setError((e as Error).message || 'Failed to parse Visio file.');
    } finally {
      setLoading(false);
    }
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
        className={`group relative rounded-xl border border-border bg-card my-3 overflow-hidden shadow-sm transition-shadow ${selected ? 'ring-2 ring-primary/40' : ''} ${dragOver ? 'ring-2 ring-blue-500 border-blue-400' : ''}`}
        onDragOver={(e) => { e.preventDefault(); e.stopPropagation(); setDragOver(true); }}
        onDragLeave={() => setDragOver(false)}
        onDrop={onDrop}
      >
        {/* ── Toolbar ── */}
        <div className="flex items-center gap-2 px-3 py-2 bg-muted/30 border-b border-border">
          {!isPreview && (
            <div className="flex items-center justify-center w-5 cursor-grab opacity-0 group-hover:opacity-60 transition-opacity" data-drag-handle>
              <GripVertical className="size-3.5 text-muted-foreground" />
            </div>
          )}

          <span className="text-[9px] font-bold text-blue-700 dark:text-blue-400 bg-blue-100 dark:bg-blue-900/40 px-2 py-0.5 rounded shrink-0">
            Visio
          </span>

          {isPreview ? (
            <span className="flex-1 text-sm font-semibold truncate">{node.attrs.title || 'Visio Diagram'}</span>
          ) : (
            <input
              value={node.attrs.title || ''}
              onChange={(e) => updateAttributes({ title: e.target.value })}
              placeholder="Diagram title…"
              className="flex-1 text-sm font-semibold bg-transparent focus:outline-none placeholder:text-muted-foreground min-w-0"
            />
          )}

          {pages.length > 1 && (
            <span className="flex items-center gap-1 text-[10px] bg-muted rounded px-1.5 py-0.5 text-muted-foreground shrink-0">
              <Layers className="size-3" /> {pages.length} pages
            </span>
          )}

          {data && (
            <span className="hidden sm:inline text-[10px] text-muted-foreground truncate max-w-[120px] shrink-0" title={data.fileName}>
              {data.fileName}
            </span>
          )}

          {!isPreview && (
            <label className="ml-auto opacity-0 group-hover:opacity-70 hover:opacity-100 flex items-center gap-1 text-[10px] border border-border rounded px-2 py-0.5 text-muted-foreground hover:text-foreground cursor-pointer shrink-0 transition-opacity">
              <Upload className="size-3" />
              {pages.length > 0 ? 'Replace' : 'Import'}
              <input
                type="file"
                accept=".vsdx,.vsd"
                className="hidden"
                onChange={(e) => {
                  const f = e.target.files?.[0];
                  if (f) handleFile(f);
                  e.currentTarget.value = '';
                }}
              />
            </label>
          )}
        </div>

        {/* ── Page tabs (multi-page files) ── */}
        {pages.length > 1 && (
          <div className="flex items-center gap-1 px-3 py-1.5 bg-muted/20 border-b border-border overflow-x-auto no-print">
            {pages.map((p, i) => (
              <button
                key={i}
                onClick={() => setActivePage(i)}
                className={`text-[10px] px-3 py-1 rounded-md whitespace-nowrap transition-colors font-medium ${
                  i === activePage
                    ? 'bg-primary text-primary-foreground'
                    : 'text-muted-foreground hover:text-foreground hover:bg-muted'
                }`}
              >
                {p.name}
              </button>
            ))}
          </div>
        )}

        {/* ── Content ── */}
        <div ref={containerRef} className="relative" style={{ minHeight: 260 }}>
          {/* Loading */}
          {loading && (
            <div className="flex items-center justify-center h-48 gap-2 text-sm text-muted-foreground">
              <div className="size-4 border-2 border-primary border-t-transparent rounded-full animate-spin" />
              Parsing Visio file…
            </div>
          )}

          {/* Error */}
          {!loading && error && (
            <div className="m-4 p-3 flex items-start gap-2 bg-destructive/10 border border-destructive/20 rounded-lg text-sm text-destructive">
              <AlertCircle className="size-4 mt-0.5 shrink-0" />
              <span>{error}</span>
            </div>
          )}

          {/* Empty / drop zone */}
          {!loading && !error && pages.length === 0 && (
            <div className="flex flex-col items-center justify-center h-48 gap-2 text-muted-foreground pointer-events-none select-none">
              <FileSpreadsheet className="size-10 opacity-25" />
              <p className="text-sm font-medium">Drop a Visio file here, or click Import</p>
              <p className="text-xs opacity-60">Supports multi-page .vsdx files</p>
            </div>
          )}

          {/* SVG diagram */}
          {!loading && !error && svgHtml && (
            <div
              className="overflow-auto p-3"
              /* SVG is generated server-side from user's own file — no XSS risk */
              dangerouslySetInnerHTML={{ __html: svgHtml }}
            />
          )}
        </div>
      </div>
    </NodeViewWrapper>
  );
}

// ─── Tiptap extension ─────────────────────────────────────────────────────────

export const VisioExtension = Node.create({
  name: 'visio',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      title: { default: '' },
      data:  { default: '' },
    };
  },

  parseHTML()  { return [{ tag: 'div[data-type="visio"]' }]; },
  renderHTML({ HTMLAttributes }) {
    return ['div', mergeAttributes({ 'data-type': 'visio' }, HTMLAttributes)];
  },
  addNodeView() { return ReactNodeViewRenderer(VisioView); },
});
