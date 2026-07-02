import type { Editor } from '@tiptap/react';
import {
  Table2, Code2, ImageIcon, ChevronDown,
  Share2, FileSpreadsheet, PenLine, LayoutGrid, FileImage,
  Network, GitBranch, Box, Layers, Workflow, MonitorPlay, Scale, ShieldCheck,
} from 'lucide-react';
import { cn } from '../ui/utils';
import { useState, useRef, useEffect } from 'react';
import type { MatrixPreset } from './widgets/matrixWidget';
import { MATRIX_PRESETS } from './widgets/matrixWidget';
import type { KrokiType } from './widgets/krokiWidget';

interface InsertToolbarProps {
  editor: Editor;
  onInsertImage: () => void;
}

// ─── Matrix dropdown ──────────────────────────────────────────────────────────
const MATRIX_OPTIONS: { preset: MatrixPreset; label: string }[] = [
  { preset: 'raci',   label: 'RACI Matrix' },
  { preset: 'crud',   label: 'CRUD Matrix' },
  { preset: 'sipoc',  label: 'SIPOC Diagram' },
  { preset: 'bmc',    label: 'Business Model Canvas' },
  { preset: 'custom', label: 'Custom Grid' },
];

function MatrixDropdown({ editor }: { editor: Editor }) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => { if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false); };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [open]);

  const insert = (preset: MatrixPreset) => {
    const { default: def } = MATRIX_PRESETS[preset];
    editor.chain().focus().insertContent({
      type: 'matrix',
      attrs: { preset, data: JSON.stringify(def), bmcData: '{}', title: '' },
    }).run();
    setOpen(false);
  };

  return (
    <div ref={ref} className="relative">
      <button
        onMouseDown={(e) => { e.preventDefault(); setOpen(o => !o); }}
        className="flex items-center gap-1.5 rounded px-2.5 py-1 text-xs font-medium transition-colors text-muted-foreground hover:bg-accent hover:text-foreground select-none"
        title="Insert matrix or grid"
      >
        <Table2 className="size-3.5" />
        Matrix
        <ChevronDown className="size-3" />
      </button>
      {open && (
        <div className="absolute top-full left-0 mt-1 z-50 min-w-[180px] rounded-lg border border-border bg-popover shadow-md py-1">
          {MATRIX_OPTIONS.map(({ preset, label }) => (
            <button
              key={preset}
              onMouseDown={(e) => { e.preventDefault(); insert(preset); }}
              className="w-full text-left px-3 py-1.5 text-xs hover:bg-accent text-foreground transition-colors"
            >
              <span className={`inline-block text-[8px] font-bold px-1 py-0.5 rounded mr-2 ${MATRIX_PRESETS[preset].badgeColor}`}>
                {MATRIX_PRESETS[preset].badge}
              </span>
              {label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

// ─── Diagrams dropdown ────────────────────────────────────────────────────────
type DiagramItem =
  | { kind: 'node'; nodeType: string; attrs?: Record<string, string>; icon: React.ReactNode; label: string; badgeCls: string; badgeText: string; }
  | { kind: 'sep' };

const DIAGRAM_GROUPS: { heading: string; items: DiagramItem[] }[] = [
  {
    heading: 'Interactive',
    items: [
      {
        kind: 'node', nodeType: 'flowDiagram', attrs: {},
        icon: <Workflow className="size-3.5" />,
        label: 'Flow Diagram',
        badgeText: 'Flow', badgeCls: 'text-violet-700 dark:text-violet-300 bg-violet-100 dark:bg-violet-900/40',
      },
      {
        kind: 'node', nodeType: 'bpmn', attrs: {},
        icon: <span className="text-[9px] font-bold leading-none">BP</span>,
        label: 'BPMN Process',
        badgeText: 'BPMN', badgeCls: 'text-rose-700 dark:text-rose-300 bg-rose-100 dark:bg-rose-900/40',
      },
    ],
  },
  {
    heading: 'File Import',
    items: [
      {
        kind: 'node', nodeType: 'drawio', attrs: {},
        icon: <Share2 className="size-3.5" />,
        label: 'draw.io',
        badgeText: 'XML', badgeCls: 'text-teal-700 dark:text-teal-300 bg-teal-100 dark:bg-teal-900/40',
      },
      {
        kind: 'node', nodeType: 'visio', attrs: {},
        icon: <FileSpreadsheet className="size-3.5" />,
        label: 'Visio (.vsdx)',
        badgeText: 'VSDX', badgeCls: 'text-blue-700 dark:text-blue-300 bg-blue-100 dark:bg-blue-900/40',
      },
      {
        kind: 'node', nodeType: 'excalidraw', attrs: {},
        icon: <PenLine className="size-3.5" />,
        label: 'Excalidraw',
        badgeText: 'EX', badgeCls: 'text-purple-700 dark:text-purple-300 bg-purple-100 dark:bg-purple-900/40',
      },
      {
        kind: 'node', nodeType: 'svgImport', attrs: {},
        icon: <FileImage className="size-3.5" />,
        label: 'SVG File',
        badgeText: 'SVG', badgeCls: 'text-slate-700 dark:text-slate-300 bg-slate-100 dark:bg-slate-800',
      },
    ],
  },
  {
    heading: 'Embed (URL)',
    items: [
      {
        kind: 'node', nodeType: 'eraser', attrs: {},
        icon: <MonitorPlay className="size-3.5" />,
        label: 'Eraser.io',
        badgeText: 'Eraser', badgeCls: 'text-amber-700 dark:text-amber-300 bg-amber-100 dark:bg-amber-900/40',
      },
      {
        kind: 'node', nodeType: 'figma', attrs: {},
        icon: <MonitorPlay className="size-3.5" />,
        label: 'Figma',
        badgeText: 'Figma', badgeCls: 'text-pink-700 dark:text-pink-300 bg-pink-100 dark:bg-pink-900/40',
      },
    ],
  },
  {
    heading: 'Code / Text',
    items: [
      {
        kind: 'node', nodeType: 'kroki', attrs: { diagramType: 'plantuml' as KrokiType },
        icon: <Layers className="size-3.5" />,
        label: 'PlantUML',
        badgeText: 'UML', badgeCls: 'text-orange-700 dark:text-orange-300 bg-orange-100 dark:bg-orange-900/40',
      },
      {
        kind: 'node', nodeType: 'kroki', attrs: { diagramType: 'd2' as KrokiType },
        icon: <GitBranch className="size-3.5" />,
        label: 'D2 Language',
        badgeText: 'D2', badgeCls: 'text-cyan-700 dark:text-cyan-300 bg-cyan-100 dark:bg-cyan-900/40',
      },
      {
        kind: 'node', nodeType: 'kroki', attrs: { diagramType: 'graphviz' as KrokiType },
        icon: <Network className="size-3.5" />,
        label: 'Graphviz DOT',
        badgeText: 'DOT', badgeCls: 'text-green-700 dark:text-green-300 bg-green-100 dark:bg-green-900/40',
      },
      {
        kind: 'node', nodeType: 'kroki', attrs: { diagramType: 'structurizr' as KrokiType },
        icon: <Box className="size-3.5" />,
        label: 'Structurizr C4',
        badgeText: 'C4', badgeCls: 'text-indigo-700 dark:text-indigo-300 bg-indigo-100 dark:bg-indigo-900/40',
      },
    ],
  },
];

function DiagramsDropdown({ editor }: { editor: Editor }) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => { if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false); };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [open]);

  return (
    <div ref={ref} className="relative">
      <button
        onMouseDown={(e) => { e.preventDefault(); setOpen(o => !o); }}
        className="flex items-center gap-1.5 rounded px-2.5 py-1 text-xs font-medium transition-colors text-muted-foreground hover:bg-accent hover:text-foreground select-none"
        title="Insert diagram"
      >
        <LayoutGrid className="size-3.5" />
        Diagrams
        <ChevronDown className="size-3" />
      </button>
      {open && (
        <div className="absolute top-full left-0 mt-1 z-50 min-w-[210px] rounded-lg border border-border bg-popover shadow-md py-1.5">
          {DIAGRAM_GROUPS.map((group, gi) => (
            <div key={gi}>
              {gi > 0 && <div className="my-1 border-t border-border" />}
              <p className="px-3 pt-1 pb-0.5 text-[9px] font-semibold uppercase tracking-widest text-muted-foreground">{group.heading}</p>
              {group.items.map((item, ii) => {
                if (item.kind === 'sep') return <div key={ii} className="my-1 border-t border-border" />;
                return (
                  <button
                    key={ii}
                    onMouseDown={(e) => {
                      e.preventDefault();
                      editor.chain().focus().insertContent({ type: item.nodeType, attrs: item.attrs ?? {} }).run();
                      setOpen(false);
                    }}
                    className="w-full text-left flex items-center gap-2.5 px-3 py-1.5 text-xs hover:bg-accent text-foreground transition-colors"
                  >
                    <span className={`text-[8px] font-bold px-1.5 py-0.5 rounded w-9 text-center shrink-0 ${item.badgeCls}`}>{item.badgeText}</span>
                    <span className="flex items-center gap-1.5 text-muted-foreground">
                      {item.icon}
                      <span className="text-foreground">{item.label}</span>
                    </span>
                  </button>
                );
              })}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

// ─── Main toolbar ─────────────────────────────────────────────────────────────
export function InsertToolbar({ editor, onInsertImage }: InsertToolbarProps) {
  const btn = (
    onClick: () => void,
    title: string,
    icon: React.ReactNode,
    label: string,
    active?: boolean,
  ) => (
    <button
      key={title}
      onMouseDown={(e) => { e.preventDefault(); onClick(); }}
      title={title}
      className={cn(
        'flex items-center gap-1.5 rounded px-2.5 py-1 text-xs font-medium transition-colors select-none',
        active
          ? 'bg-primary/10 text-primary'
          : 'text-muted-foreground hover:bg-accent hover:text-foreground',
      )}
    >
      {icon}
      {label}
    </button>
  );

  return (
    <div className="flex items-center gap-0.5 px-4 py-1 border-b border-border bg-muted/20 shrink-0">
      {btn(
        () => editor.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(),
        'Insert table',
        <Table2 className="size-3.5" />,
        'Table',
        editor.isActive('table'),
      )}
      {btn(
        () => editor.chain().focus().toggleCodeBlock().run(),
        'Insert code block',
        <Code2 className="size-3.5" />,
        'Code',
        editor.isActive('codeBlock'),
      )}
      {btn(
        onInsertImage,
        'Insert image',
        <ImageIcon className="size-3.5" />,
        'Image',
      )}
      <MatrixDropdown editor={editor} />
      <DiagramsDropdown editor={editor} />
      {btn(
        () => editor.chain().focus().insertContent({ type: 'tradeoff', attrs: { data: '' } }).run(),
        'Trade-off analysis',
        <Scale className="size-3.5" />,
        'Trade-off',
      )}
      {btn(
        () => editor.chain().focus().insertContent({ type: 'nfr', attrs: { data: '' } }).run(),
        'NFR & Quality Attributes',
        <ShieldCheck className="size-3.5" />,
        'NFR',
      )}
    </div>
  );
}
