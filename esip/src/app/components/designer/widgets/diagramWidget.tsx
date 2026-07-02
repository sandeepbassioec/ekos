import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useEffect, useRef, useContext } from 'react';
import { GripVertical, Eye, Code2, RefreshCw, Sparkles } from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

let mermaidReady = false;
async function getMermaid() {
  if (mermaidReady) return (await import('mermaid')).default;
  const m = (await import('mermaid')).default;
  m.initialize({ startOnLoad: false, theme: 'neutral', securityLevel: 'loose', fontFamily: 'inherit' });
  mermaidReady = true;
  return m;
}

const DEFAULT_SOURCE = `sequenceDiagram
  Client->>API GW: POST /login
  API GW->>Auth Service: validateCreds()
  Auth Service-->>API GW: JWT token
  API GW-->>Client: 200 { token }`;

function DiagramView({ node, updateAttributes, selected, getPos }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const [mode, setMode] = useState<'preview' | 'edit'>('preview');
  const [svg, setSvg] = useState('');
  const [error, setError] = useState('');
  const [rendering, setRendering] = useState(false);
  const [draft, setDraft] = useState(node.attrs.source || DEFAULT_SOURCE);
  const renderIdRef = useRef(0);

  const render = async (src: string) => {
    setRendering(true);
    setError('');
    const id = ++renderIdRef.current;
    try {
      const mermaid = await getMermaid();
      const uniqueId = `mermaid-${Date.now()}-${Math.floor(Math.random() * 10000)}`;
      const { svg: out } = await mermaid.render(uniqueId, src);
      if (id === renderIdRef.current) setSvg(out);
    } catch (e) {
      if (id === renderIdRef.current) setError((e as Error).message || 'Invalid diagram syntax');
    } finally {
      if (id === renderIdRef.current) setRendering(false);
    }
  };

  useEffect(() => {
    if (mode === 'preview') render(node.attrs.source || DEFAULT_SOURCE);
  }, [node.attrs.source, mode]);

  const applyDraft = () => {
    updateAttributes({ source: draft });
    setMode('preview');
  };

  return (
    <NodeViewWrapper>
      <div className={`group relative rounded-xl border border-border bg-card my-3 overflow-hidden shadow-sm ${selected ? 'ring-2 ring-primary/40' : ''}`}>
        {/* Toolbar */}
        <div className="flex items-center gap-2 px-3 py-2 bg-muted/30 border-b border-border">
          {!isPreview && (
            <div className="flex items-center justify-center w-5 cursor-grab opacity-0 group-hover:opacity-60 transition-opacity" data-drag-handle>
              <GripVertical className="size-3.5 text-muted-foreground" />
            </div>
          )}
          <span className="text-[9px] font-bold text-violet-700 dark:text-violet-400 bg-violet-100 dark:bg-violet-900/40 px-2 py-0.5 rounded">MERMAID</span>
          {isPreview ? (
            <span className="flex-1 text-sm font-semibold text-foreground">{node.attrs.title || 'Diagram'}</span>
          ) : (
            <input
              value={node.attrs.title || ''}
              onChange={(e) => updateAttributes({ title: e.target.value })}
              placeholder="Diagram title…"
              className="flex-1 text-sm font-semibold bg-transparent focus:outline-none text-foreground placeholder:text-muted-foreground"
            />
          )}
          <div className="flex items-center gap-1 ml-auto">
            {!isPreview && (
              <>
                <button
                  onMouseDown={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    document.dispatchEvent(new CustomEvent('ask-sandeep-target', {
                      bubbles: true,
                      detail: { pos: getPos?.(), nodeType: 'diagram', attrs: { ...node.attrs }, targetField: 'source', displayName: `Diagram: ${node.attrs.title || 'Untitled'}` },
                    }));
                  }}
                  className="opacity-60 hover:opacity-100 flex items-center gap-1 text-[10px] text-primary border border-primary/30 bg-primary/5 rounded px-2 py-0.5 transition-all"
                  title="Generate Mermaid with AI"
                >
                  <Sparkles className="size-3" /> AI
                </button>
                <button
                  onClick={() => { if (mode === 'edit') applyDraft(); else { setDraft(node.attrs.source || DEFAULT_SOURCE); setMode('edit'); } }}
                  className="flex items-center gap-1 text-[10px] text-muted-foreground hover:text-foreground transition-colors border border-border rounded px-2 py-0.5"
                >
                  {mode === 'preview' ? <><Code2 className="size-3" /> Edit</> : <><Eye className="size-3" /> Preview</>}
                </button>
              </>
            )}
            {mode === 'preview' && (
              <button onClick={() => render(node.attrs.source || DEFAULT_SOURCE)} className="text-muted-foreground hover:text-foreground transition-colors p-0.5" title="Re-render">
                <RefreshCw className={`size-3.5 ${rendering ? 'animate-spin' : ''}`} />
              </button>
            )}
          </div>
        </div>

        {/* Preview */}
        {mode === 'preview' && (
          <div className="p-4 bg-muted/10 flex justify-center items-center min-h-[100px]">
            {rendering && <div className="text-sm text-muted-foreground animate-pulse">Rendering diagram…</div>}
            {error && !rendering && (
              <div className="text-sm text-destructive bg-destructive/10 rounded px-3 py-2 font-mono whitespace-pre-wrap max-w-full">{error}</div>
            )}
            {!rendering && !error && svg && (
              <div
                className="max-w-full overflow-auto"
                dangerouslySetInnerHTML={{ __html: svg }}
              />
            )}
          </div>
        )}

        {/* Editor */}
        {mode === 'edit' && (
          <div className="flex flex-col">
            <textarea
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              rows={10}
              spellCheck={false}
              className="w-full bg-[#1e1e2e] dark:bg-[#13131f] text-[#cdd6f4] font-mono text-sm p-4 focus:outline-none resize-none leading-relaxed border-b border-border"
              placeholder="Enter Mermaid diagram source…"
            />
            <div className="flex items-center justify-between px-3 py-2 bg-muted/20">
              <a
                href="https://mermaid.js.org/syntax/flowchart.html"
                target="_blank"
                rel="noopener noreferrer"
                className="text-[10px] text-muted-foreground hover:text-foreground transition-colors"
                onClick={(e) => e.stopPropagation()}
              >
                Mermaid syntax reference ↗
              </a>
              <div className="flex items-center gap-2">
                <button onClick={() => setMode('preview')} className="text-xs text-muted-foreground hover:text-foreground transition-colors">Cancel</button>
                <button onClick={applyDraft} className="text-xs bg-primary text-primary-foreground px-3 py-1 rounded font-medium">Apply</button>
              </div>
            </div>
          </div>
        )}
      </div>
    </NodeViewWrapper>
  );
}

export const DiagramExtension = Node.create({
  name: 'diagram',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      title:  { default: '' },
      source: { default: DEFAULT_SOURCE },
    };
  },

  parseHTML() { return [{ tag: 'div[data-type="diagram"]' }]; },
  renderHTML({ HTMLAttributes }) { return ['div', mergeAttributes({ 'data-type': 'diagram' }, HTMLAttributes)]; },
  addNodeView() { return ReactNodeViewRenderer(DiagramView); },
});
