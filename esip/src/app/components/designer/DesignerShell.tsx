import { useState, useEffect, useCallback, useRef } from 'react';
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Collaboration from '@tiptap/extension-collaboration';
import * as Y from 'yjs';
import { IndexeddbPersistence } from 'y-indexeddb';
import { WebsocketProvider } from 'y-websocket';
import Placeholder from '@tiptap/extension-placeholder';
import { Table, TableRow, TableHeader, TableCell } from '@tiptap/extension-table';
import { Image } from '@tiptap/extension-image';
import {
  CalloutExtension, RequirementExtension, AdrExtension,
  MetricsExtension, ProcessStepsExtension, DiagramExtension,
  StatusBadgeExtension, DividerExtension, TradeoffExtension, NFRExtension,
} from './widgets';
import { FlowDiagramExtension } from './widgets/flowDiagramWidget';
import { BpmnExtension } from './widgets/bpmnWidget';
import { MatrixExtension } from './widgets/matrixWidget';
import { DrawioExtension } from './widgets/drawioWidget';
import { VisioExtension } from './widgets/visioWidget';
import { ExcalidrawExtension } from './widgets/excalidrawWidget';
import { SvgImportExtension } from './widgets/svgWidget';
import { KrokiExtension } from './widgets/krokiWidget';
import { EraserExtension, FigmaExtension } from './widgets/embedWidget';
import { SlashCommandExtension } from './SlashCommandExtension';
import { useNavigate } from 'react-router';
import {
  FileText, ChevronRight, Save, ArrowLeft, CheckCircle,
  Clock, Eye, Loader2, Sparkles, Activity, Pencil, Printer,
} from 'lucide-react';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import {
  DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
} from '../ui/dropdown-menu';
import { cn } from '../ui/utils';
import { db } from '../../lib/db';
import { useAuthStore } from '../../store/authStore';
import { toast } from 'sonner';
import { templateRegistry } from './templateRegistry';
import { EmptyLineTrigger, AskSandeepPanel } from './AskSandeepMenu';
import { DocInsightsPanel } from './DocInsightsPanel';
import { PreviewModeContext } from './PreviewModeContext';
import { SmartPasteExtension } from './SmartPasteExtension';
import { SmartPasteBanner } from './SmartPasteBanner';
import type { SmartPasteInfo } from './SmartPasteExtension';
import { BubbleFormatMenu, type SelectionMode } from './BubbleFormatMenu';
import { InsertToolbar } from './InsertToolbar';
import { TableControls } from './TableControls';
import { ImageDialog } from './ImageDialog';
import type { DesignerTemplate, WidgetTarget } from './types';
import type { DesignerDocumentCategory, DesignerDocument, DesignerDocumentStatus } from '../../lib/db/types';

// TableCell extended with backgroundColor for colorful cells
const ColoredTableCell = TableCell.extend({
  addAttributes() {
    return {
      ...this.parent?.(),
      backgroundColor: {
        default: null,
        parseHTML: (element: HTMLElement) => element.style.backgroundColor || null,
        renderHTML: (attributes: Record<string, unknown>) => {
          if (!attributes.backgroundColor) return {};
          return { style: `background-color: ${attributes.backgroundColor}` };
        },
      },
    };
  },
});

interface DesignerShellProps {
  projectId: string;
  documentId?: string;
}

const STATUS_CONFIG: Record<DesignerDocumentStatus, { label: string; icon: React.ReactNode }> = {
  draft:    { label: 'Draft',      icon: <Clock className="size-3" /> },
  review:   { label: 'In Review',  icon: <Eye className="size-3" /> },
  approved: { label: 'Approved',   icon: <CheckCircle className="size-3" /> },
};

// ── Category metadata ─────────────────────────────────────────────────────────
interface CategoryMeta {
  id: DesignerDocumentCategory;
  name: string;
  tagline: string;
  badge: string;
  accentBg: string;
  accentText: string;
  variants: number;
}

const CATEGORIES: CategoryMeta[] = [
  {
    id: 'brd',
    name: 'Business Requirements Document',
    tagline: 'Capture what needs to be built — stakeholders, requirements, scope, and success criteria.',
    badge: 'BRD',
    accentBg: 'bg-blue-50 dark:bg-blue-950/30',
    accentText: 'text-blue-700 dark:text-blue-400',
    variants: 5,
  },
  {
    id: 'hld',
    name: 'High-Level Design',
    tagline: 'Architecture overview — system context, major components, integrations, and ADRs.',
    badge: 'HLD',
    accentBg: 'bg-violet-50 dark:bg-violet-950/30',
    accentText: 'text-violet-700 dark:text-violet-400',
    variants: 3,
  },
  {
    id: 'lld',
    name: 'Low-Level Design',
    tagline: 'Detailed component design — data models, API contracts, sequence flows, and error handling.',
    badge: 'LLD',
    accentBg: 'bg-emerald-50 dark:bg-emerald-950/30',
    accentText: 'text-emerald-700 dark:text-emerald-400',
    variants: 3,
  },
];

// ── Step 1: Category Picker ───────────────────────────────────────────────────
function CategoryPicker({ onSelect }: { onSelect: (categoryId: DesignerDocumentCategory) => void }) {
  return (
    <div className="flex flex-1 items-center justify-center bg-background">
      <div className="w-full max-w-2xl px-6">
        <div className="mb-8 text-center">
          <div className="inline-flex size-12 items-center justify-center rounded-xl bg-primary/10 mb-3">
            <FileText className="size-6 text-primary" />
          </div>
          <h2 className="text-2xl font-semibold">New Document</h2>
          <p className="text-muted-foreground mt-1">What type of document do you want to create?</p>
        </div>
        <div className="grid gap-3">
          {CATEGORIES.map(cat => (
            <button
              key={cat.id}
              onClick={() => onSelect(cat.id)}
              className="group flex items-center gap-4 rounded-xl border border-border bg-card p-4 text-left hover:border-primary/50 hover:shadow-md transition-all"
            >
              <div className={cn('flex size-12 shrink-0 items-center justify-center rounded-xl', cat.accentBg)}>
                <span className={cn('text-sm font-bold', cat.accentText)}>{cat.badge}</span>
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <p className="font-semibold text-sm">{cat.name}</p>
                  <span className="rounded-full bg-muted px-2 py-0.5 text-[10px] text-muted-foreground">
                    {cat.variants} templates
                  </span>
                </div>
                <p className="text-xs text-muted-foreground mt-0.5 line-clamp-1">{cat.tagline}</p>
              </div>
              <ChevronRight className="size-4 text-muted-foreground group-hover:text-primary transition-colors shrink-0" />
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

// ── Step 2: Variant Picker ────────────────────────────────────────────────────
function VariantPicker({
  categoryId,
  onSelect,
  onBack,
}: {
  categoryId: DesignerDocumentCategory;
  onSelect: (template: DesignerTemplate) => void;
  onBack: () => void;
}) {
  const cat = CATEGORIES.find(c => c.id === categoryId)!;
  const templates = templateRegistry.getByCategory(categoryId);

  return (
    <div className="flex flex-1 items-center justify-center bg-background">
      <div className="w-full max-w-2xl px-6">
        <div className="mb-6">
          <button
            onClick={onBack}
            className="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors mb-4"
          >
            <ArrowLeft className="size-3.5" />
            All document types
          </button>
          <div className="flex items-center gap-3 mb-1">
            <div className={cn('flex size-9 items-center justify-center rounded-lg', cat.accentBg)}>
              <span className={cn('text-xs font-bold', cat.accentText)}>{cat.badge}</span>
            </div>
            <div>
              <h2 className="text-xl font-semibold">{cat.name}</h2>
              <p className="text-xs text-muted-foreground">{cat.tagline}</p>
            </div>
          </div>
        </div>

        <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-3">
          Choose a template
        </p>
        <div className="grid gap-2.5">
          {templates.map(t => (
            <button
              key={t.id}
              onClick={() => onSelect(t)}
              className="group flex items-center gap-4 rounded-xl border border-border bg-card p-4 text-left hover:border-primary/50 hover:shadow-md transition-all"
            >
              <div className={cn('flex size-10 shrink-0 items-center justify-center rounded-lg text-xs font-bold', cat.accentBg, cat.accentText)}>
                {t.badge}
              </div>
              <div className="flex-1 min-w-0">
                <p className="font-semibold text-sm">{t.name}</p>
                <p className="text-xs text-muted-foreground mt-0.5 line-clamp-2 leading-relaxed">{t.description}</p>
              </div>
              <ChevronRight className="size-4 text-muted-foreground group-hover:text-primary transition-colors shrink-0" />
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

// ── Main Shell ────────────────────────────────────────────────────────────────
export function DesignerShell({ projectId, documentId }: DesignerShellProps) {
  const navigate = useNavigate();
  const { user } = useAuthStore();

  const [selectedCategoryId, setSelectedCategoryId] = useState<DesignerDocumentCategory | null>(null);
  const [activeTemplate, setActiveTemplate] = useState<DesignerTemplate | null>(null);
  const [docId, setDocId] = useState<string | undefined>(documentId);
  const [docStatus, setDocStatus] = useState<DesignerDocumentStatus>('draft');
  const [docTitle, setDocTitle] = useState('');
  const [saving, setSaving] = useState(false);
  const [loading, setLoading] = useState(!!documentId);
  const [imageDialogOpen, setImageDialogOpen] = useState(false);
  const [selectionMode, setSelectionMode] = useState<SelectionMode | null>(null);
  const [askSandeepOpen, setAskSandeepOpen] = useState(false);
  const [insightsOpen, setInsightsOpen] = useState(false);
  const [isPreview, setIsPreview] = useState(false);
  const [printSize, setPrintSize] = useState<'a4' | 'a3' | 'a3p' | 'a1'>('a3');
  const [printMenuOpen, setPrintMenuOpen] = useState(false);
  const [collabPeers, setCollabPeers] = useState(0);
  const [collabConnected, setCollabConnected] = useState(false);
  const [editorKey, setEditorKey] = useState(0);
  // Cursor overlay: map of clientId -> { name, color, top, left }
  const [remoteCursors, setRemoteCursors] = useState<Map<number, { name: string; color: string; top: number; left: number }>>(new Map());
  const editorContainerRef = useRef<HTMLDivElement>(null);
  // Stable Y.Doc — never recreated; only providers change
  const ydocRef = useRef<Y.Doc>(new Y.Doc());
  const providerRef = useRef<WebsocketProvider | null>(null);
  const persistRef = useRef<IndexeddbPersistence | null>(null);
  // Stable ref to the editor so awareness callbacks always see the latest instance
  const editorRef = useRef<ReturnType<typeof useEditor>>(null);

  const handlePrint = (size: 'a4' | 'a3' | 'a3p' | 'a1') => {
    setPrintMenuOpen(false);
    // Inject a temporary @page rule for the chosen size
    const sizeMap = { a4: 'A4 portrait', a3: 'A3 landscape', a3p: 'A3 portrait', a1: 'A1 landscape' };
    const style = document.createElement('style');
    style.id = '__print-size';
    style.textContent = `@media print { @page { size: ${sizeMap[size]}; margin: 15mm; } }`;
    document.head.appendChild(style);
    window.print();
    setTimeout(() => document.getElementById('__print-size')?.remove(), 1000);
  };
  const [widgetTarget, setWidgetTarget] = useState<WidgetTarget | null>(null);
  const [smartPasteInfo, setSmartPasteInfo] = useState<SmartPasteInfo | null>(null);

  const editor = useEditor({
    extensions: [
      StarterKit.configure({ history: false }),
      Collaboration.configure({ document: ydocRef.current }),
      Placeholder.configure({
        placeholder: ({ node }) => {
          if (node.type.name === 'heading') return 'Section heading…';
          return 'Type here, or click "Ask Sandeep" to generate content with AI…';
        },
        showOnlyWhenEditable: true,
      }),
      Table.configure({ resizable: false }),
      TableRow,
      TableHeader,
      ColoredTableCell,
      Image.configure({ inline: false, allowBase64: true }),
      CalloutExtension,
      RequirementExtension,
      AdrExtension,
      MetricsExtension,
      ProcessStepsExtension,
      DiagramExtension,
      FlowDiagramExtension,
      BpmnExtension,
      MatrixExtension,
      DrawioExtension,
      VisioExtension,
      ExcalidrawExtension,
      SvgImportExtension,
      KrokiExtension,
      EraserExtension,
      FigmaExtension,
      StatusBadgeExtension,
      DividerExtension,
      TradeoffExtension,
      NFRExtension,
      SlashCommandExtension,
      SmartPasteExtension,
    ],
    content: '',
    editorProps: {
      attributes: { class: 'focus:outline-none min-h-[calc(100vh-14rem)] max-w-none' },
    },
  }, [editorKey]);

  // Keep editorRef current so awareness callbacks always see the latest editor instance
  useEffect(() => { editorRef.current = editor; }, [editor]);

  // Load existing document — skip both pickers
  useEffect(() => {
    if (!documentId || !editor) return;
    (async () => {
      setLoading(true);
      try {
        const doc = await db.designerDocuments.getById(documentId);
        if (!doc) { toast.error('Document not found'); return; }
        const template = templateRegistry.getById(doc.templateId);
        if (template) {
          setActiveTemplate(template);
          setSelectedCategoryId(doc.category as DesignerDocumentCategory);
        }
        setDocTitle(doc.title);
        setDocStatus(doc.status);
        editor.commands.setContent(JSON.parse(doc.content));
      } catch {
        toast.error('Failed to load document');
      } finally {
        setLoading(false);
      }
    })();
  }, [documentId, editor]);

  // Listen for widget sparkle-button events
  useEffect(() => {
    const handler = (e: Event) => {
      const detail = (e as CustomEvent).detail as WidgetTarget;
      setWidgetTarget(detail);
      setAskSandeepOpen(true);
      setInsightsOpen(false);
      setSelectionMode(null);
    };
    document.addEventListener('ask-sandeep-target', handler);
    return () => document.removeEventListener('ask-sandeep-target', handler);
  }, []);

  // Listen for smart-paste events
  useEffect(() => {
    const handler = (e: Event) => {
      if (!isPreview) setSmartPasteInfo((e as CustomEvent).detail as SmartPasteInfo);
    };
    document.addEventListener('smart-paste-detected', handler);
    return () => document.removeEventListener('smart-paste-detected', handler);
  }, [isPreview]);

  useEffect(() => {
    if (!printMenuOpen) return;
    const close = (e: MouseEvent) => {
      if (!(e.target as Element)?.closest?.('.no-print')) setPrintMenuOpen(false);
    };
    document.addEventListener('mousedown', close);
    return () => document.removeEventListener('mousedown', close);
  }, [printMenuOpen]);

  // ── Yjs collaboration setup ─────────────────────────────────────────────────
  // Y.Doc is stable (never recreated). Providers/persistence swap when docId changes.
  useEffect(() => {
    const roomId = docId || 'unsaved';
    const ydoc = ydocRef.current;

    // Tear down previous providers
    providerRef.current?.destroy();
    persistRef.current?.destroy();

    // Resolve local user identity (stable per tab via sessionStorage)
    let localUser: { name: string; color: string } | null = null;
    try {
      localUser = JSON.parse(sessionStorage.getItem('esip-collab-user') || 'null');
    } catch { /* ignore */ }
    if (!localUser) {
      const COLORS = ['#7c3aed', '#0891b2', '#059669', '#d97706', '#dc2626', '#db2777'];
      localUser = {
        name: user?.email?.split('@')[0] || 'Guest',
        color: COLORS[Math.floor(Math.random() * COLORS.length)],
      };
      sessionStorage.setItem('esip-collab-user', JSON.stringify(localUser));
    }
    const resolvedUser = localUser;

    // IndexedDB persistence — fires 'synced' once after initial load
    const persist = new IndexeddbPersistence(`esip-${roomId}`, ydoc);
    persistRef.current = persist;
    persist.on('synced', () => setEditorKey(k => k + 1));

    // WebSocket provider — connects to local collab server (run: pnpm collab)
    // Falls back gracefully when the server is not running
    try {
      const wsUrl = import.meta.env.VITE_COLLAB_WS_URL || 'ws://localhost:1234';
      const provider = new WebsocketProvider(wsUrl, `esip-${roomId}`, ydoc, {
        connect: true,
      });
      providerRef.current = provider;
      provider.awareness.setLocalStateField('user', resolvedUser);

      provider.on('status', ({ status }: { status: string }) => {
        setCollabConnected(status === 'connected');
      });

      provider.awareness.on('change', () => {
        setCollabPeers(provider.awareness.getStates().size - 1);
        // Rebuild cursor overlay positions
        refreshCursors(provider);
      });
    } catch { /* server not running — local-only mode */ }

    return () => {
      providerRef.current?.destroy();
      persistRef.current?.destroy();
    };
  }, [docId]); // eslint-disable-line react-hooks/exhaustive-deps

  // Update cursor overlay whenever the editor selection changes
  useEffect(() => {
    if (!editor || !providerRef.current) return;
    const provider = providerRef.current;

    const broadcastCursor = () => {
      const { from, to } = editor.state.selection;
      provider.awareness.setLocalStateField('cursor', { anchor: from, head: to });
      refreshCursors(provider);
    };

    editor.on('selectionUpdate', broadcastCursor);
    editor.on('focus', broadcastCursor);
    return () => {
      editor.off('selectionUpdate', broadcastCursor);
      editor.off('focus', broadcastCursor);
    };
  }, [editor]); // eslint-disable-line react-hooks/exhaustive-deps

  const refreshCursors = (provider: WebsocketProvider) => {
    const ed = editorRef.current;
    if (!ed || ed.isDestroyed) return;
    const next = new Map<number, { name: string; color: string; top: number; left: number }>();
    provider.awareness.getStates().forEach((state, clientId) => {
      if (clientId === provider.awareness.clientID) return;
      if (!state.user || !state.cursor) return;
      try {
        const pos = Math.min(state.cursor.head, ed.state.doc.content.size - 1);
        if (pos < 0) return;
        const coords = ed.view.coordsAtPos(pos);
        const container = editorContainerRef.current;
        if (!container) return;
        const rect = container.getBoundingClientRect();
        next.set(clientId, {
          name: state.user.name,
          color: state.user.color,
          top: coords.top - rect.top + container.scrollTop,
          left: coords.left - rect.left,
        });
      } catch { /* position out of range */ }
    });
    setRemoteCursors(next);
  };

  useEffect(() => {
    if (!editor) return;
    editor.setEditable(!isPreview);
    if (isPreview) {
      setAskSandeepOpen(false);
      setInsightsOpen(false);
      setWidgetTarget(null);
      setSelectionMode(null);
    }
  }, [editor, isPreview]);

  const handleSelectTemplate = useCallback((template: DesignerTemplate) => {
    setActiveTemplate(template);
    setDocTitle(template.name);
    if (editor) editor.commands.setContent(template.initialContent);
  }, [editor]);

  const handleSave = async () => {
    if (!editor || !activeTemplate || !user) return;
    setSaving(true);
    try {
      const content = JSON.stringify(editor.getJSON());
      if (docId) {
        await db.designerDocuments.update(docId, {
          title: docTitle || activeTemplate.name,
          content,
          status: docStatus,
          updatedAt: new Date().toISOString(),
        });
        toast.success('Document saved');
      } else {
        const created = await db.designerDocuments.create({
          projectId,
          title: docTitle || activeTemplate.name,
          templateId: activeTemplate.id,
          category: activeTemplate.category,
          content,
          status: 'draft',
          version: 1,
          createdBy: user.id,
        });
        setDocId(created.id);
        toast.success('Document created');
      }
    } catch (err) {
      toast.error('Save failed: ' + (err as Error).message);
    } finally {
      setSaving(false);
    }
  };

  const handleStatusChange = async (newStatus: DesignerDocumentStatus) => {
    setDocStatus(newStatus);
    if (docId) {
      try {
        await db.designerDocuments.update(docId, { status: newStatus });
        toast.success(`Status updated to ${STATUS_CONFIG[newStatus].label}`);
      } catch {
        toast.error('Failed to update status');
      }
    }
  };

  // ── Routing ───────────────────────────────────────────────────────────────
  if (loading) {
    return (
      <div className="flex flex-1 items-center justify-center">
        <Loader2 className="size-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!selectedCategoryId) {
    return <CategoryPicker onSelect={setSelectedCategoryId} />;
  }

  if (!activeTemplate) {
    return (
      <VariantPicker
        categoryId={selectedCategoryId}
        onSelect={handleSelectTemplate}
        onBack={() => setSelectedCategoryId(null)}
      />
    );
  }

  const statusCfg = STATUS_CONFIG[docStatus];
  const cat = CATEGORIES.find(c => c.id === activeTemplate.category);

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Main toolbar */}
      <div className="flex h-12 items-center gap-3 shrink-0 border-b border-border bg-card px-4">
        <Button
          variant="ghost" size="sm"
          onClick={() => navigate(`/projects/${projectId}/documents`)}
          className="gap-1.5 -ml-1"
        >
          <ArrowLeft className="size-4" />
          <span className="hidden sm:inline">Documents</span>
        </Button>
        <div className="w-px h-5 bg-border" />

        {isPreview ? (
          <span className="flex-1 text-sm font-medium text-foreground/80 truncate min-w-0 select-none">
            {docTitle || activeTemplate.name}
          </span>
        ) : (
          <input
            value={docTitle}
            onChange={(e) => setDocTitle(e.target.value)}
            placeholder="Document title…"
            className="flex-1 bg-transparent text-sm font-medium focus:outline-none placeholder:text-muted-foreground min-w-0"
          />
        )}

        <Badge
          variant="secondary"
          className={cn('hidden sm:inline-flex text-xs shrink-0', cat?.accentBg, cat?.accentText)}
        >
          {activeTemplate.badge}
        </Badge>

        {isPreview ? (
          <span className={cn('hidden sm:inline-flex items-center gap-1 h-7 rounded-md border px-2.5 text-xs shrink-0', 'border-border text-muted-foreground')}>
            {statusCfg.icon}
            {statusCfg.label}
          </span>
        ) : (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm" className="gap-1.5 h-7 text-xs shrink-0">
                {statusCfg.icon}
                {statusCfg.label}
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              {(Object.entries(STATUS_CONFIG) as [DesignerDocumentStatus, typeof STATUS_CONFIG[DesignerDocumentStatus]][]).map(([key, cfg]) => (
                <DropdownMenuItem key={key} onClick={() => handleStatusChange(key)} className="gap-2">
                  {cfg.icon}
                  <span className={cn(key === docStatus && 'font-medium')}>{cfg.label}</span>
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
        )}

        {!isPreview && (
          <>
            <Button
              size="sm"
              variant={askSandeepOpen ? 'default' : 'outline'}
              onClick={() => {
                setAskSandeepOpen(o => !o);
                setInsightsOpen(false);
                if (!askSandeepOpen) { setWidgetTarget(null); setSelectionMode(null); }
              }}
              className="gap-1.5 h-7 text-xs shrink-0"
            >
              <Sparkles className="size-3" />
              <span className="hidden sm:inline">Ask Sandeep</span>
            </Button>

            <Button
              size="sm"
              variant={insightsOpen ? 'default' : 'outline'}
              onClick={() => {
                setInsightsOpen(o => !o);
                setAskSandeepOpen(false);
                setWidgetTarget(null);
                setSelectionMode(null);
              }}
              className="gap-1.5 h-7 text-xs shrink-0"
            >
              <Activity className="size-3" />
              <span className="hidden sm:inline">Insights</span>
            </Button>
          </>
        )}

        {/* Live collab indicator */}
        {collabPeers > 0 && (
          <div className="no-print flex items-center gap-1 text-xs text-emerald-600 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-800 rounded px-2 py-0.5">
            <span className="size-1.5 rounded-full bg-emerald-500 animate-pulse inline-block" />
            {collabPeers + 1} live
          </div>
        )}
        {collabConnected && collabPeers === 0 && (
          <div className="no-print flex items-center gap-1 text-xs text-muted-foreground" title="Synced — no other editors">
            <span className="size-1.5 rounded-full bg-emerald-500 inline-block" />
          </div>
        )}

        {/* Print / Export PDF */}
        <div className="relative no-print">
          <Button
            size="sm"
            variant="outline"
            onClick={() => setPrintMenuOpen(o => !o)}
            className="gap-1.5 h-7 text-xs shrink-0"
            title="Export PDF"
          >
            <Printer className="size-3" />
            <span className="hidden sm:inline">Export</span>
          </Button>
          {printMenuOpen && (
            <div className="absolute right-0 top-full mt-1 z-50 min-w-[160px] rounded-lg border border-border bg-popover shadow-md py-1">
              <div className="px-3 py-1 text-[10px] font-semibold text-muted-foreground uppercase tracking-wide">Page size</div>
              {([['a4','A4 Portrait'],['a3','A3 Landscape'],['a3p','A3 Portrait'],['a1','A1 Landscape']] as const).map(([key, label]) => (
                <button
                  key={key}
                  onClick={() => { setPrintSize(key); handlePrint(key); }}
                  className={`w-full text-left px-3 py-1.5 text-xs hover:bg-accent transition-colors flex items-center justify-between ${printSize === key ? 'text-primary font-medium' : 'text-foreground'}`}
                >
                  {label}
                  {printSize === key && <span className="text-[9px] text-primary">✓</span>}
                </button>
              ))}
            </div>
          )}
        </div>

        <Button
          size="sm"
          variant={isPreview ? 'default' : 'outline'}
          onClick={() => setIsPreview(p => !p)}
          className="gap-1.5 h-7 text-xs shrink-0"
        >
          {isPreview ? <Pencil className="size-3" /> : <Eye className="size-3" />}
          <span className="hidden sm:inline">{isPreview ? 'Edit' : 'Preview'}</span>
        </Button>

        {!isPreview && (
          <Button size="sm" onClick={handleSave} disabled={saving} className="gap-1.5 h-7 text-xs shrink-0">
            {saving ? <Loader2 className="size-3 animate-spin" /> : <Save className="size-3" />}
            {saving ? 'Saving…' : 'Save'}
          </Button>
        )}
      </div>

      {/* Insert toolbar — edit mode only */}
      {editor && !isPreview && (
        <InsertToolbar editor={editor} onInsertImage={() => setImageDialogOpen(true)} />
      )}

      {/* Table context toolbar — edit mode only */}
      {editor && !isPreview && <TableControls editor={editor} />}

      {/* Editor canvas + Ask Sandeep panel */}
      <div className="flex-1 flex overflow-hidden">
        <div ref={editorContainerRef} className={cn('flex-1 overflow-auto bg-background relative', isPreview && 'bg-muted/20')}>
          {/* Remote cursor overlays */}
          {Array.from(remoteCursors.entries()).map(([clientId, cur]) => (
            <div
              key={clientId}
              className="pointer-events-none absolute z-50 flex items-start gap-0.5 no-print"
              style={{ top: cur.top, left: cur.left, transform: 'translateY(-2px)' }}
            >
              {/* Caret */}
              <div className="w-0.5 h-5 rounded-full" style={{ backgroundColor: cur.color }} />
              {/* Name label */}
              <div
                className="rounded px-1.5 py-0.5 text-[10px] font-semibold text-white leading-none mt-0.5 whitespace-nowrap"
                style={{ backgroundColor: cur.color }}
              >
                {cur.name}
              </div>
            </div>
          ))}
          <div className={cn('mx-auto px-8 py-10 relative', isPreview ? 'max-w-4xl' : 'max-w-3xl')}>
            {smartPasteInfo && editor && activeTemplate && !isPreview && (
              <SmartPasteBanner
                editor={editor}
                info={smartPasteInfo}
                template={activeTemplate}
                onDismiss={() => setSmartPasteInfo(null)}
              />
            )}
            {editor && !isPreview && (
              <>
                <EmptyLineTrigger
                  editor={editor}
                  onOpenPanel={() => { setAskSandeepOpen(true); setWidgetTarget(null); setSelectionMode(null); }}
                />
                <BubbleFormatMenu
                  editor={editor}
                  onAskSandeep={(mode) => { setSelectionMode(mode); setWidgetTarget(null); setAskSandeepOpen(true); setInsightsOpen(false); }}
                />
              </>
            )}
            <PreviewModeContext.Provider value={isPreview}>
            <EditorContent
            editor={editor}
            className="prose prose-sm dark:prose-invert max-w-none
              prose-headings:font-semibold prose-headings:text-foreground
              prose-h1:text-2xl prose-h1:mt-0 prose-h1:mb-4
              prose-h2:text-xl prose-h2:mt-8 prose-h2:mb-3 prose-h2:border-b prose-h2:border-border prose-h2:pb-1
              prose-h3:text-base prose-h3:mt-5 prose-h3:mb-2
              prose-p:text-foreground prose-p:leading-relaxed
              prose-li:text-foreground prose-li:my-0.5
              prose-strong:text-foreground
              prose-code:bg-muted prose-code:rounded prose-code:px-1.5 prose-code:py-0.5 prose-code:text-sm prose-code:font-mono prose-code:text-foreground
              prose-pre:bg-muted prose-pre:border prose-pre:border-border prose-pre:rounded-lg prose-pre:p-4
              prose-img:rounded-lg prose-img:border prose-img:border-border prose-img:max-w-full
              [&_table]:w-full [&_table]:border-collapse [&_table]:text-sm [&_table]:my-4
              [&_th]:border [&_th]:border-border [&_th]:bg-muted/60 [&_th]:px-3 [&_th]:py-2 [&_th]:text-left [&_th]:font-semibold [&_th]:text-foreground
              [&_td]:border [&_td]:border-border [&_td]:px-3 [&_td]:py-2 [&_td]:text-foreground [&_td]:align-top
              [&_tr:hover_td]:bg-muted/30
              [&_.is-editor-empty_.is-empty]:before:content-[attr(data-placeholder)] [&_.is-editor-empty_.is-empty]:before:text-muted-foreground [&_.is-editor-empty_.is-empty]:before:pointer-events-none [&_.is-editor-empty_.is-empty]:before:float-left [&_.is-editor-empty_.is-empty]:before:h-0"
          />
            </PreviewModeContext.Provider>
          </div>
        </div>

        {/* Ask Sandeep right panel */}
        {askSandeepOpen && editor && activeTemplate && (
          <div className="w-80 xl:w-96 shrink-0 flex flex-col overflow-hidden">
            <AskSandeepPanel
              editor={editor}
              template={activeTemplate}
              widgetTarget={widgetTarget}
              onClearTarget={() => setWidgetTarget(null)}
              selectionMode={selectionMode}
              onSelectionModeClose={() => setSelectionMode(null)}
              onClose={() => { setAskSandeepOpen(false); setWidgetTarget(null); setSelectionMode(null); }}
            />
          </div>
        )}

        {/* Doc Insights right panel */}
        {insightsOpen && editor && activeTemplate && (
          <div className="w-80 xl:w-96 shrink-0 flex flex-col overflow-hidden">
            <DocInsightsPanel
              editor={editor}
              template={activeTemplate}
              projectId={projectId}
              documentId={docId}
              onClose={() => setInsightsOpen(false)}
            />
          </div>
        )}
      </div>

      {/* Image insertion dialog */}
      {editor && (
        <ImageDialog
          editor={editor}
          open={imageDialogOpen}
          onClose={() => setImageDialogOpen(false)}
        />
      )}
    </div>
  );
}
