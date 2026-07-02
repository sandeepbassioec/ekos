import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useContext, useCallback } from 'react';
import { GripVertical, Play, RefreshCw, AlertCircle } from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

// ─── Supported types ──────────────────────────────────────────────────────────
export type KrokiType = 'plantuml' | 'd2' | 'graphviz' | 'structurizr';

const KROKI_META: Record<KrokiType, { label: string; badge: string; badgeCls: string; example: string }> = {
  plantuml: {
    label: 'PlantUML',
    badge: 'PlantUML',
    badgeCls: 'text-orange-700 dark:text-orange-300 bg-orange-100 dark:bg-orange-900/40',
    example: `@startuml
actor User
User -> Server : Request
Server -> DB : Query
DB --> Server : Result
Server --> User : Response
@enduml`,
  },
  d2: {
    label: 'D2',
    badge: 'D2',
    badgeCls: 'text-cyan-700 dark:text-cyan-300 bg-cyan-100 dark:bg-cyan-900/40',
    example: `user -> server: Request
server -> database: Query
database -> server: Result
server -> user: Response`,
  },
  graphviz: {
    label: 'Graphviz',
    badge: 'Graphviz',
    badgeCls: 'text-green-700 dark:text-green-300 bg-green-100 dark:bg-green-900/40',
    example: `digraph G {
  rankdir=LR
  User -> Server [label="Request"]
  Server -> DB [label="Query"]
  DB -> Server [label="Result"]
  Server -> User [label="Response"]
}`,
  },
  structurizr: {
    label: 'Structurizr',
    badge: 'Structurizr',
    badgeCls: 'text-indigo-700 dark:text-indigo-300 bg-indigo-100 dark:bg-indigo-900/40',
    example: `workspace {
  model {
    user = person "User"
    system = softwareSystem "My System" {
      api = container "API"
      db = container "Database"
    }
    user -> api "Uses"
    api -> db "Reads/Writes"
  }
  views {
    systemContext system {
      include *
      autoLayout
    }
  }
}`,
  },
};

// ─── Kroki fetch ──────────────────────────────────────────────────────────────
async function fetchKroki(type: KrokiType, code: string): Promise<string> {
  const res = await fetch(`https://kroki.io/${type}/svg`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ diagram_source: code }),
  });
  if (!res.ok) {
    const body = await res.text().catch(() => '');
    throw new Error(body || `Kroki returned ${res.status}`);
  }
  return res.text();
}

// ─── NodeView ─────────────────────────────────────────────────────────────────
function KrokiView({ node, updateAttributes, selected }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const diagramType = (node.attrs.diagramType || 'plantuml') as KrokiType;
  const meta = KROKI_META[diagramType] ?? KROKI_META.plantuml;

  const [code, setCode] = useState<string>(node.attrs.code || meta.example);
  const [svg, setSvg] = useState<string>(node.attrs.svg || '');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const render = useCallback(async (src?: string) => {
    const source = src ?? code;
    if (!source.trim()) return;
    setLoading(true);
    setError('');
    try {
      const result = await fetchKroki(diagramType, source);
      setSvg(result);
      updateAttributes({ code: source, svg: result });
    } catch (err) {
      setError((err as Error).message || 'Render failed');
    } finally {
      setLoading(false);
    }
  }, [code, diagramType, updateAttributes]);

  // Auto-render on first mount if code exists but no svg yet
  const [didAutoRender, setDidAutoRender] = useState(false);
  if (!didAutoRender && code.trim() && !svg) {
    setDidAutoRender(true);
    render(code);
  }

  const handleCodeChange = (val: string) => {
    setCode(val);
    updateAttributes({ code: val });
  };

  return (
    <NodeViewWrapper>
      <div className={`group relative rounded-xl border border-border bg-card my-3 overflow-hidden shadow-sm ${selected ? 'ring-2 ring-primary/40' : ''}`}>
        {/* Header */}
        <div className="flex items-center gap-2 px-3 py-2 bg-muted/30 border-b border-border">
          {!isPreview && (
            <div data-drag-handle className="w-5 flex items-center justify-center cursor-grab opacity-0 group-hover:opacity-60 transition-opacity">
              <GripVertical className="size-3.5 text-muted-foreground" />
            </div>
          )}
          <span className={`text-[9px] font-bold px-2 py-0.5 rounded shrink-0 ${meta.badgeCls}`}>{meta.badge}</span>
          {isPreview ? (
            <span className="flex-1 text-sm font-semibold truncate">{node.attrs.title || meta.label + ' Diagram'}</span>
          ) : (
            <input
              value={node.attrs.title || ''}
              onChange={(e) => updateAttributes({ title: e.target.value })}
              placeholder={`${meta.label} diagram title…`}
              className="flex-1 text-sm font-semibold bg-transparent focus:outline-none placeholder:text-muted-foreground min-w-0"
            />
          )}
          {!isPreview && (
            <button
              onMouseDown={(e) => { e.preventDefault(); render(); }}
              disabled={loading}
              className="ml-auto flex items-center gap-1.5 text-[10px] font-medium border border-border rounded px-2 py-0.5 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors disabled:opacity-50"
            >
              {loading
                ? <><RefreshCw className="size-3 animate-spin" /> Rendering…</>
                : <><Play className="size-3" /> {svg ? 'Re-render' : 'Render'}</>}
            </button>
          )}
        </div>

        {/* Two-pane: editor + preview */}
        {!isPreview ? (
          <div className="flex divide-x divide-border" style={{ minHeight: 220 }}>
            {/* Code editor */}
            <div className="flex-1 flex flex-col" style={{ minWidth: 0 }}>
              <textarea
                value={code}
                onChange={(e) => handleCodeChange(e.target.value)}
                onKeyDown={(e) => {
                  if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') { e.preventDefault(); render(); }
                }}
                spellCheck={false}
                className="flex-1 resize-none font-mono text-xs leading-relaxed p-3 bg-transparent focus:outline-none text-foreground min-h-[200px]"
                placeholder={`Paste your ${meta.label} code here…\nCtrl+Enter to render`}
              />
              <p className="px-3 py-1 text-[9px] text-muted-foreground border-t border-border">Ctrl+Enter to render · powered by <a href="https://kroki.io" target="_blank" rel="noreferrer" className="underline hover:text-foreground">kroki.io</a></p>
            </div>

            {/* Preview */}
            <div className="flex-1 p-3 overflow-auto relative" style={{ maxHeight: 500 }}>
              {error && (
                <div className="flex items-start gap-2 text-destructive text-xs p-2 bg-destructive/10 rounded">
                  <AlertCircle className="size-3.5 mt-0.5 shrink-0" />
                  <pre className="whitespace-pre-wrap break-all">{error}</pre>
                </div>
              )}
              {loading && (
                <div className="absolute inset-0 flex items-center justify-center bg-background/60 z-10">
                  <RefreshCw className="size-6 animate-spin text-muted-foreground" />
                </div>
              )}
              {svg && !error && <div dangerouslySetInnerHTML={{ __html: svg }} />}
              {!svg && !error && !loading && (
                <div className="flex items-center justify-center h-full text-muted-foreground text-sm opacity-60 select-none">
                  Press Render to preview
                </div>
              )}
            </div>
          </div>
        ) : (
          // Preview mode: just show the diagram
          <div className="p-3 overflow-auto" style={{ maxHeight: 600 }}>
            {svg
              ? <div dangerouslySetInnerHTML={{ __html: svg }} />
              : <p className="text-sm text-muted-foreground italic">{meta.label} diagram (not yet rendered)</p>}
          </div>
        )}
      </div>
    </NodeViewWrapper>
  );
}

// ─── Extension ────────────────────────────────────────────────────────────────
export const KrokiExtension = Node.create({
  name: 'kroki',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      title:       { default: '' },
      diagramType: { default: 'plantuml' },
      code:        { default: '' },
      svg:         { default: '' },
    };
  },

  parseHTML()  { return [{ tag: 'div[data-type="kroki"]' }]; },
  renderHTML({ HTMLAttributes }) {
    return ['div', mergeAttributes({ 'data-type': 'kroki' }, HTMLAttributes)];
  },
  addNodeView() { return ReactNodeViewRenderer(KrokiView); },
});
