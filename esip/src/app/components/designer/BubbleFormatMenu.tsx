import { useState, useEffect } from 'react';
import type { Editor } from '@tiptap/react';
import { Bold, Italic, Heading1, Heading2, Heading3, List, ListOrdered, Sparkles } from 'lucide-react';
import { cn } from '../ui/utils';

export interface SelectionMode {
  text: string;
  from: number;
  to: number;
  coords: { top: number; left: number };
}

interface Coords { top: number; left: number }

function useSelectionCoords(editor: Editor | null): Coords | null {
  const [coords, setCoords] = useState<Coords | null>(null);

  useEffect(() => {
    if (!editor) return;

    const update = () => {
      const { from, to } = editor.state.selection;
      if (from === to) { setCoords(null); return; }
      try {
        const start = editor.view.coordsAtPos(from);
        const end = editor.view.coordsAtPos(to);
        const editorDom = editor.view.dom as HTMLElement;
        const rect = editorDom.getBoundingClientRect();
        const midX = (start.left + end.left) / 2 - rect.left;
        setCoords({ top: start.top - rect.top - 44, left: Math.max(0, midX - 160) });
      } catch {
        setCoords(null);
      }
    };

    editor.on('selectionUpdate', update);
    return () => editor.off('selectionUpdate', update);
  }, [editor]);

  return coords;
}

interface BubbleFormatMenuProps {
  editor: Editor;
  onAskSandeep?: (mode: SelectionMode) => void;
}

export function BubbleFormatMenu({ editor, onAskSandeep }: BubbleFormatMenuProps) {
  const coords = useSelectionCoords(editor);

  if (!coords) return null;

  const btn = (active: boolean, onMD: () => void, title: string, children: React.ReactNode) => (
    <button
      key={title}
      onMouseDown={(e) => { e.preventDefault(); onMD(); }}
      title={title}
      className={cn(
        'flex size-7 items-center justify-center rounded transition-colors text-sm',
        active ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground',
      )}
    >
      {children}
    </button>
  );

  const handleAskSandeep = (e: React.MouseEvent) => {
    e.preventDefault();
    if (!onAskSandeep) return;
    const { from, to } = editor.state.selection;
    if (from === to) return;
    const text = editor.state.doc.textBetween(from, to, '\n');
    // Position the Ask Sandeep panel below the selection end
    try {
      const endPos = editor.view.coordsAtPos(to);
      const editorDom = editor.view.dom as HTMLElement;
      const rect = editorDom.getBoundingClientRect();
      onAskSandeep({
        text,
        from,
        to,
        coords: {
          top: endPos.bottom - rect.top + editorDom.scrollTop + 8,
          left: 0,
        },
      });
    } catch {
      // fallback coords
      onAskSandeep({ text, from, to, coords: { top: coords.top + 60, left: 0 } });
    }
  };

  return (
    <div
      className="absolute z-30 flex items-center gap-0.5 rounded-lg border border-border bg-card shadow-lg px-1 py-1 pointer-events-auto"
      style={{ top: coords.top, left: coords.left }}
      onMouseDown={(e) => e.stopPropagation()}
    >
      {btn(editor.isActive('bold'),     () => editor.chain().focus().toggleBold().run(),               'Bold',         <Bold className="size-3.5" />)}
      {btn(editor.isActive('italic'),   () => editor.chain().focus().toggleItalic().run(),             'Italic',       <Italic className="size-3.5" />)}
      <div className="w-px h-4 bg-border mx-0.5" />
      {btn(editor.isActive('heading', { level: 1 }), () => editor.chain().focus().toggleHeading({ level: 1 }).run(), 'H1', <Heading1 className="size-3.5" />)}
      {btn(editor.isActive('heading', { level: 2 }), () => editor.chain().focus().toggleHeading({ level: 2 }).run(), 'H2', <Heading2 className="size-3.5" />)}
      {btn(editor.isActive('heading', { level: 3 }), () => editor.chain().focus().toggleHeading({ level: 3 }).run(), 'H3', <Heading3 className="size-3.5" />)}
      <div className="w-px h-4 bg-border mx-0.5" />
      {btn(editor.isActive('bulletList'),  () => editor.chain().focus().toggleBulletList().run(),  'Bullet list',   <List className="size-3.5" />)}
      {btn(editor.isActive('orderedList'), () => editor.chain().focus().toggleOrderedList().run(), 'Numbered list', <ListOrdered className="size-3.5" />)}
      {onAskSandeep && (
        <>
          <div className="w-px h-4 bg-border mx-0.5" />
          <button
            onMouseDown={handleAskSandeep}
            title="Ask Sandeep about selection"
            className="flex items-center gap-1 rounded px-2 h-7 text-xs font-medium text-primary hover:bg-primary/10 transition-colors"
          >
            <Sparkles className="size-3" />
            Ask Sandeep
          </button>
        </>
      )}
    </div>
  );
}
