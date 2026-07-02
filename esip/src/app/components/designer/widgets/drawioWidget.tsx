import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useEffect, useRef, useContext, useCallback, useId } from 'react';
import { GripVertical, Upload, Download, Code2, Eye, Sparkles } from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

// ── Load diagrams.net viewer script once ─────────────────────────────────────

let viewerLoaded = false;
let viewerLoading = false;
const viewerCallbacks: (() => void)[] = [];

function loadViewer(): Promise<void> {
  return new Promise((resolve) => {
    if (viewerLoaded) { resolve(); return; }
    viewerCallbacks.push(resolve);
    if (viewerLoading) return;
    viewerLoading = true;
    const script = document.createElement('script');
    script.src = 'https://viewer.diagrams.net/js/viewer-static.min.js';
    script.onload = () => {
      viewerLoaded = true;
      viewerLoading = false;
      viewerCallbacks.forEach(cb => cb());
      viewerCallbacks.length = 0;
    };
    script.onerror = () => {
      viewerLoading = false;
      viewerCallbacks.forEach(cb => cb()); // resolve anyway, will show error
      viewerCallbacks.length = 0;
    };
    document.head.appendChild(script);
  });
}

// ── Default diagram XML ───────────────────────────────────────────────────────

const DEFAULT_XML = `<mxGraphModel><root><mxCell id="0"/><mxCell id="1" parent="0"/><mxCell id="2" value="Start" style="ellipse;whiteSpace=wrap;html=1;" vertex="1" parent="1"><mxGeometry x="60" y="60" width="80" height="40" as="geometry"/></mxCell><mxCell id="3" value="Process" style="rounded=1;whiteSpace=wrap;html=1;" vertex="1" parent="1"><mxGeometry x="200" y="55" width="100" height="50" as="geometry"/></mxCell><mxCell id="4" value="End" style="ellipse;whiteSpace=wrap;html=1;" vertex="1" parent="1"><mxGeometry x="360" y="60" width="80" height="40" as="geometry"/></mxCell><mxCell id="5" edge="1" source="2" target="3" parent="1"><mxGeometry relative="1" as="geometry"/></mxCell><mxCell id="6" edge="1" source="3" target="4" parent="1"><mxGeometry relative="1" as="geometry"/></mxCell></root></mxGraphModel>`;

// ── Renderer component ────────────────────────────────────────────────────────

function DiagramRenderer({ xml, containerId }: { xml: string; containerId: string }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [error, setError] = useState('');

  useEffect(() => {
    if (!containerRef.current) return;
    let cancelled = false;

    loadViewer().then(() => {
      if (cancelled || !containerRef.current) return;
      try {
        // Clear previous render
        containerRef.current.innerHTML = '';
        const div = document.createElement('div');
        div.className = 'mxgraph';
        div.style.cssText = 'max-width:100%;overflow:hidden;';
        div.setAttribute('data-mxgraph', JSON.stringify({
          xml,
          highlight: '#0000ff',
          'auto-fit': 1,
          'fit-page': 1,
          nav: false,
          resize: 1,
        }));
        containerRef.current.appendChild(div);

        // Trigger the viewer to render
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const w = window as any;
        if (w.GraphViewer) {
          w.GraphViewer.processElements();
        } else if (w.mxGraphViewer) {
          w.mxGraphViewer.processElements();
        }
        setError('');
      } catch (e) {
        if (!cancelled) setError((e as Error).message || 'Render failed');
      }
    });

    return () => { cancelled = true; };
  }, [xml]);

  return (
    <div>
      {error && <div className="px-4 py-2 text-xs text-destructive">{error} — viewer may be offline</div>}
      <div ref={containerRef} id={containerId} className="w-full" />
    </div>
  );
}

// ── Widget view ───────────────────────────────────────────────────────────────

function DrawioView({ node, updateAttributes, selected, getPos }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const [showXml, setShowXml] = useState(false);
  const [xmlDraft, setXmlDraft] = useState('');
  const [dragOver, setDragOver] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);
  const uid = useId().replace(/:/g, '');

  const xml = node.attrs.xml || DEFAULT_XML;

  const handleFile = useCallback((file: File) => {
    const reader = new FileReader();
    reader.onload = (e) => {
      const content = e.target?.result as string;
      // draw.io files are either raw mxGraphModel XML or wrapped in <mxfile>
      let extracted = content;
      const mxfileMatch = content.match(/<mxfile[^>]*>([\s\S]*?)<\/mxfile>/);
      if (mxfileMatch) {
        // Extract the diagram content — may be compressed (base64 deflate) or plain
        const diagramMatch = mxfileMatch[1].match(/<diagram[^>]*>([\s\S]*?)<\/diagram>/);
        if (diagramMatch) {
          const inner = diagramMatch[1].trim();
          // If it starts with '<' it's uncompressed XML
          extracted = inner.startsWith('<') ? inner : content;
        }
      }
      updateAttributes({
        xml: extracted,
        title: node.attrs.title || file.name.replace(/\.(drawio|xml)$/, ''),
      });
    };
    reader.readAsText(file);
  }, [updateAttributes, node.attrs.title]);

  const onDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation(); // prevent ProseMirror from taking the drop
    setDragOver(false);
    const file = e.dataTransfer.files[0];
    if (file) handleFile(file);
  }, [handleFile]);

  const handleExport = useCallback(() => {
    const blob = new Blob([`<mxfile><diagram>${xml}</diagram></mxfile>`], { type: 'application/xml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${node.attrs.title || 'diagram'}.drawio`;
    a.click();
    URL.revokeObjectURL(url);
  }, [xml, node.attrs.title]);

  const saveXml = useCallback(() => {
    updateAttributes({ xml: xmlDraft });
    setShowXml(false);
  }, [xmlDraft, updateAttributes]);

  const openXmlEditor = () => { setXmlDraft(xml); setShowXml(true); };

  return (
    <NodeViewWrapper>
      <div
        className={`group relative rounded-xl border border-border bg-card my-3 overflow-hidden shadow-sm ${selected ? 'ring-2 ring-primary/40' : ''} ${dragOver ? 'ring-2 ring-primary border-primary' : ''}`}
        onDragOver={(e) => { e.preventDefault(); e.stopPropagation(); setDragOver(true); }}
        onDragLeave={() => setDragOver(false)}
        onDrop={onDrop}
      >
        {/* Toolbar */}
        <div className="flex items-center gap-2 px-3 py-2 bg-muted/30 border-b border-border shrink-0">
          {!isPreview && (
            <div className="flex items-center justify-center w-5 cursor-grab opacity-0 group-hover:opacity-60 transition-opacity" data-drag-handle>
              <GripVertical className="size-3.5 text-muted-foreground" />
            </div>
          )}
          <span className="text-[9px] font-bold text-teal-700 dark:text-teal-400 bg-teal-100 dark:bg-teal-900/40 px-2 py-0.5 rounded">draw.io</span>
          {isPreview ? (
            <span className="flex-1 text-sm font-semibold">{node.attrs.title || 'Diagram'}</span>
          ) : (
            <input
              value={node.attrs.title || ''}
              onChange={(e) => updateAttributes({ title: e.target.value })}
              placeholder="Diagram title…"
              className="flex-1 text-sm font-semibold bg-transparent focus:outline-none placeholder:text-muted-foreground"
            />
          )}
          <div className="ml-auto flex items-center gap-1">
            {!isPreview && (
              <>
                <button
                  onMouseDown={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    document.dispatchEvent(new CustomEvent('ask-sandeep-target', {
                      bubbles: true,
                      detail: { pos: getPos?.(), nodeType: 'drawio', attrs: { ...node.attrs }, targetField: 'xml', displayName: `draw.io: ${node.attrs.title || 'Untitled'}` },
                    }));
                  }}
                  className="opacity-0 group-hover:opacity-70 hover:opacity-100 flex items-center gap-1 text-[10px] text-primary border border-primary/30 bg-primary/5 rounded px-2 py-0.5 transition-all"
                  title="Generate diagram with AI"
                >
                  <Sparkles className="size-3" /> AI
                </button>
                <label className="opacity-0 group-hover:opacity-60 hover:opacity-100 flex items-center gap-1 text-[10px] border border-border rounded px-2 py-0.5 text-muted-foreground hover:text-foreground cursor-pointer" title="Import .drawio file">
                  <Upload className="size-3" /> Import
                  <input ref={fileRef} type="file" accept=".drawio,.xml" className="hidden" onChange={(e) => { const f = e.target.files?.[0]; if (f) handleFile(f); e.target.value = ''; }} />
                </label>
              </>
            )}
            <button onClick={handleExport} className="opacity-0 group-hover:opacity-60 hover:opacity-100 p-1 text-muted-foreground hover:text-foreground transition-all" title="Export .drawio">
              <Download className="size-3.5" />
            </button>
            {!isPreview && !showXml && (
              <button onClick={openXmlEditor} className="opacity-0 group-hover:opacity-60 hover:opacity-100 p-1 text-muted-foreground hover:text-foreground transition-all" title="Edit XML">
                <Code2 className="size-3.5" />
              </button>
            )}
            {!isPreview && showXml && (
              <>
                <button onClick={saveXml} className="flex items-center gap-1 text-[10px] bg-primary text-primary-foreground rounded px-2.5 py-1 font-medium">Save</button>
                <button onClick={() => setShowXml(false)} className="flex items-center gap-1 text-[10px] border border-border rounded px-2.5 py-1 text-muted-foreground hover:text-foreground">
                  <Eye className="size-3" /> Preview
                </button>
              </>
            )}
          </div>
        </div>

        {/* Drop zone hint when empty default */}
        {!isPreview && xml === DEFAULT_XML && !showXml && (
          <div className="absolute inset-x-0 top-12 flex items-center justify-center pointer-events-none z-10">
            <div className="bg-background/80 border border-border rounded-lg px-4 py-2 text-xs text-muted-foreground flex items-center gap-2 shadow-sm">
              <Upload className="size-3.5" /> Drop a .drawio file here or click Import
            </div>
          </div>
        )}

        {/* XML editor */}
        {showXml && (
          <textarea
            value={xmlDraft}
            onChange={(e) => setXmlDraft(e.target.value)}
            className="w-full font-mono text-xs p-3 bg-muted/20 focus:outline-none resize-none"
            style={{ height: 240 }}
            spellCheck={false}
          />
        )}

        {/* Diagram renderer */}
        {!showXml && (
          <div style={{ minHeight: 280 }} className="relative">
            <DiagramRenderer xml={xml} containerId={`drawio-${uid}`} />
          </div>
        )}
      </div>
    </NodeViewWrapper>
  );
}

// ── Tiptap extension ──────────────────────────────────────────────────────────

export const DrawioExtension = Node.create({
  name: 'drawio',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      title: { default: '' },
      xml:   { default: DEFAULT_XML },
    };
  },

  parseHTML()  { return [{ tag: 'div[data-type="drawio"]' }]; },
  renderHTML({ HTMLAttributes }) { return ['div', mergeAttributes({ 'data-type': 'drawio' }, HTMLAttributes)]; },
  addNodeView() { return ReactNodeViewRenderer(DrawioView); },
});
