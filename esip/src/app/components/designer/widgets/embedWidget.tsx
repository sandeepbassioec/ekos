import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useContext } from 'react';
import { GripVertical, Link, ExternalLink, AlertCircle } from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

// ─── Eraser.io ────────────────────────────────────────────────────────────────
// Share URL:  https://app.eraser.io/workspace/ABC123?origin=share
// Embed URL:  same URL works directly in an iframe
function normalizeEraserUrl(raw: string): string | null {
  try {
    const u = new URL(raw.trim());
    if (!u.hostname.includes('eraser.io')) return null;
    // ensure origin=share param is present
    u.searchParams.set('origin', 'share');
    return u.toString();
  } catch {
    return null;
  }
}

// ─── Figma ────────────────────────────────────────────────────────────────────
// Share URLs:
//   https://www.figma.com/design/ID/...
//   https://www.figma.com/proto/ID/...
//   https://www.figma.com/board/ID/...   (FigJam)
// Embed URL: https://www.figma.com/embed?embed_host=share&url=<encoded>
function normalizeFigmaUrl(raw: string): string | null {
  try {
    const u = new URL(raw.trim());
    if (!u.hostname.includes('figma.com')) return null;
    // Already an embed URL — return as-is
    if (u.pathname === '/embed') return u.toString();
    return `https://www.figma.com/embed?embed_host=share&url=${encodeURIComponent(u.toString())}`;
  } catch {
    return null;
  }
}

// ─── Generic embed config ─────────────────────────────────────────────────────
interface EmbedConfig {
  name: string;
  badge: string;
  badgeCls: string;
  placeholder: string;
  hint: string;
  docsUrl: string;
  normalize: (raw: string) => string | null;
  /** Extra sandbox permissions beyond the default set */
  extraSandbox?: string;
  /** Fixed aspect-ratio height as percentage of width (e.g. 56.25 = 16:9). Undefined = fixed px height. */
  aspectRatio?: number;
  defaultHeight?: number;
}

const EMBED_CONFIGS: Record<string, EmbedConfig> = {
  eraser: {
    name: 'Eraser',
    badge: 'Eraser',
    badgeCls: 'text-amber-700 dark:text-amber-300 bg-amber-100 dark:bg-amber-900/40',
    placeholder: 'https://app.eraser.io/workspace/ABC…?origin=share',
    hint: 'In Eraser: Share → Copy link',
    docsUrl: 'https://docs.eraser.io',
    normalize: normalizeEraserUrl,
    extraSandbox: 'allow-popups allow-popups-to-escape-sandbox',
    defaultHeight: 520,
  },
  figma: {
    name: 'Figma',
    badge: 'Figma',
    badgeCls: 'text-pink-700 dark:text-pink-300 bg-pink-100 dark:bg-pink-900/40',
    placeholder: 'https://www.figma.com/design/…  or  /proto/…',
    hint: 'In Figma: Share → Copy link  (set to "Anyone with the link can view")',
    docsUrl: 'https://www.figma.com/developers/embed',
    normalize: normalizeFigmaUrl,
    extraSandbox: 'allow-popups allow-popups-to-escape-sandbox allow-top-navigation',
    aspectRatio: 56.25,   // 16:9
  },
};

// ─── Shared NodeView component ────────────────────────────────────────────────
function EmbedView({ node, updateAttributes, selected }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const embedType = (node.attrs.embedType as string) || 'eraser';
  const cfg = EMBED_CONFIGS[embedType] ?? EMBED_CONFIGS.eraser;

  const [inputVal, setInputVal] = useState('');
  const [error, setError] = useState('');

  const embedUrl = node.attrs.embedUrl as string;

  const commit = (raw: string) => {
    const url = cfg.normalize(raw.trim());
    if (!url) {
      setError(`Not a valid ${cfg.name} share URL. ${cfg.hint}`);
      return;
    }
    setError('');
    updateAttributes({ embedUrl: url });
    setInputVal('');
  };

  const sandbox = [
    'allow-scripts',
    'allow-same-origin',
    'allow-forms',
    'allow-presentation',
    cfg.extraSandbox,
  ].filter(Boolean).join(' ');

  const height = cfg.aspectRatio ? undefined : (cfg.defaultHeight ?? 480);

  return (
    <NodeViewWrapper>
      <div className={`group relative rounded-xl border border-border bg-card my-3 overflow-hidden shadow-sm ${selected ? 'ring-2 ring-primary/40' : ''}`}>

        {/* Toolbar */}
        <div className="flex items-center gap-2 px-3 py-2 bg-muted/30 border-b border-border">
          {!isPreview && (
            <div data-drag-handle className="w-5 flex items-center justify-center cursor-grab opacity-0 group-hover:opacity-60 transition-opacity">
              <GripVertical className="size-3.5 text-muted-foreground" />
            </div>
          )}
          <span className={`text-[9px] font-bold px-2 py-0.5 rounded shrink-0 ${cfg.badgeCls}`}>{cfg.badge}</span>

          {isPreview ? (
            <span className="flex-1 text-sm font-semibold truncate">{node.attrs.title || `${cfg.name} Embed`}</span>
          ) : (
            <input
              value={node.attrs.title || ''}
              onChange={(e) => updateAttributes({ title: e.target.value })}
              placeholder={`${cfg.name} embed title…`}
              className="flex-1 text-sm font-semibold bg-transparent focus:outline-none placeholder:text-muted-foreground min-w-0"
            />
          )}

          {embedUrl && (
            <a
              href={embedUrl}
              target="_blank"
              rel="noreferrer"
              className="ml-auto opacity-0 group-hover:opacity-70 hover:opacity-100 flex items-center gap-1 text-[10px] text-muted-foreground hover:text-foreground transition-opacity"
              title={`Open in ${cfg.name}`}
            >
              <ExternalLink className="size-3" /> Open
            </a>
          )}
        </div>

        {/* URL input (shown when no embed URL set, or in edit mode) */}
        {!isPreview && (
          <div className="px-3 pt-2 pb-1 border-b border-border/50 bg-muted/10">
            <div className="flex items-center gap-2">
              <Link className="size-3.5 text-muted-foreground shrink-0" />
              <input
                value={inputVal}
                onChange={(e) => { setInputVal(e.target.value); setError(''); }}
                onKeyDown={(e) => { if (e.key === 'Enter') { e.preventDefault(); commit(inputVal); } }}
                placeholder={cfg.placeholder}
                className="flex-1 text-xs bg-transparent focus:outline-none placeholder:text-muted-foreground/60 font-mono"
              />
              <button
                onMouseDown={(e) => { e.preventDefault(); commit(inputVal); }}
                disabled={!inputVal.trim()}
                className="text-[10px] font-medium border border-border rounded px-2 py-0.5 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors disabled:opacity-40"
              >
                {embedUrl ? 'Replace' : 'Embed'}
              </button>
            </div>
            {error && (
              <div className="flex items-start gap-1.5 mt-1.5 text-[10px] text-destructive">
                <AlertCircle className="size-3 mt-0.5 shrink-0" />
                <span>{error}</span>
              </div>
            )}
            {!embedUrl && !error && (
              <p className="mt-1 text-[10px] text-muted-foreground/70">{cfg.hint}</p>
            )}
          </div>
        )}

        {/* Iframe or empty state */}
        {embedUrl ? (
          cfg.aspectRatio ? (
            <div style={{ position: 'relative', paddingBottom: `${cfg.aspectRatio}%`, height: 0 }}>
              <iframe
                src={embedUrl}
                sandbox={sandbox}
                allowFullScreen
                style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: '100%', border: 'none', display: 'block' }}
                title={`${cfg.name} embed`}
              />
            </div>
          ) : (
            <iframe
              src={embedUrl}
              sandbox={sandbox}
              allowFullScreen
              style={{ width: '100%', height: height, border: 'none', display: 'block' }}
              title={`${cfg.name} embed`}
            />
          )
        ) : (
          !isPreview && (
            <div className="flex flex-col items-center justify-center gap-2 text-muted-foreground py-10 select-none pointer-events-none">
              <span className={`text-3xl font-bold ${cfg.badgeCls} px-4 py-2 rounded-xl opacity-40`}>{cfg.badge}</span>
              <p className="text-sm">Paste a {cfg.name} share link above to embed</p>
            </div>
          )
        )}
      </div>
    </NodeViewWrapper>
  );
}

// ─── Extension factory ────────────────────────────────────────────────────────
function makeEmbedExtension(nodeName: string, embedType: string) {
  return Node.create({
    name: nodeName,
    group: 'block',
    atom: true,
    draggable: true,

    addAttributes() {
      return {
        title:     { default: '' },
        embedType: { default: embedType },
        embedUrl:  { default: '' },
      };
    },

    parseHTML()  { return [{ tag: `div[data-type="${nodeName}"]` }]; },
    renderHTML({ HTMLAttributes }) {
      return ['div', mergeAttributes({ 'data-type': nodeName }, HTMLAttributes)];
    },
    addNodeView() { return ReactNodeViewRenderer(EmbedView); },
  });
}

export const EraserExtension   = makeEmbedExtension('eraser', 'eraser');
export const FigmaExtension     = makeEmbedExtension('figma',  'figma');
