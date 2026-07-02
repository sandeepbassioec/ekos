import { useState, useEffect } from 'react';
import type { Editor } from '@tiptap/react';
import { cn } from '../ui/utils';

const CELL_COLORS: { label: string; value: string | null; display: string }[] = [
  { label: 'None',   value: null,      display: 'bg-background border-2 border-dashed' },
  { label: 'Blue',   value: '#dbeafe', display: 'bg-blue-100' },
  { label: 'Green',  value: '#dcfce7', display: 'bg-green-100' },
  { label: 'Yellow', value: '#fef9c3', display: 'bg-yellow-100' },
  { label: 'Orange', value: '#ffedd5', display: 'bg-orange-100' },
  { label: 'Red',    value: '#fee2e2', display: 'bg-red-100' },
  { label: 'Purple', value: '#f3e8ff', display: 'bg-purple-100' },
  { label: 'Pink',   value: '#fce7f3', display: 'bg-pink-100' },
  { label: 'Gray',   value: '#f3f4f6', display: 'bg-gray-100' },
];

interface TableControlsProps {
  editor: Editor;
}

export function TableControls({ editor }: TableControlsProps) {
  const [inTable, setInTable] = useState(false);
  const [showColors, setShowColors] = useState(false);

  useEffect(() => {
    const update = () => setInTable(editor.isActive('table'));
    editor.on('selectionUpdate', update);
    editor.on('update', update);
    return () => { editor.off('selectionUpdate', update); editor.off('update', update); };
  }, [editor]);

  // Close color picker when leaving table
  useEffect(() => { if (!inTable) setShowColors(false); }, [inTable]);

  if (!inTable) return null;

  const act = (cmd: () => boolean, label: string, danger?: boolean) => (
    <button
      key={label}
      onMouseDown={(e) => { e.preventDefault(); cmd(); }}
      title={label}
      className={cn(
        'rounded px-2 py-0.5 text-[10px] font-medium transition-colors',
        danger
          ? 'text-destructive hover:bg-destructive/10'
          : 'text-muted-foreground hover:bg-accent hover:text-foreground',
      )}
    >
      {label}
    </button>
  );

  const sep = <div className="w-px h-4 bg-border mx-0.5 shrink-0" />;

  return (
    <div className="flex items-center gap-0.5 px-4 py-1 border-b border-border bg-primary/5 shrink-0 flex-wrap">
      <span className="text-[10px] font-semibold text-primary mr-2">TABLE</span>

      {act(() => editor.chain().focus().addRowBefore().run(), '↑ Row')}
      {act(() => editor.chain().focus().addRowAfter().run(), '↓ Row')}
      {act(() => editor.chain().focus().deleteRow().run(), '✕ Row', true)}
      {sep}
      {act(() => editor.chain().focus().addColumnBefore().run(), '← Col')}
      {act(() => editor.chain().focus().addColumnAfter().run(), '→ Col')}
      {act(() => editor.chain().focus().deleteColumn().run(), '✕ Col', true)}
      {sep}
      {act(() => editor.chain().focus().deleteTable().run(), 'Delete table', true)}
      {sep}

      {/* Cell background color picker */}
      <div className="relative">
        <button
          onMouseDown={(e) => { e.preventDefault(); setShowColors(v => !v); }}
          className="flex items-center gap-1.5 rounded px-2 py-0.5 text-[10px] font-medium text-muted-foreground hover:bg-accent hover:text-foreground transition-colors border border-border"
        >
          <span>Cell color</span>
          <span className="opacity-60">▾</span>
        </button>

        {showColors && (
          <div className="absolute top-full left-0 mt-1 z-40 flex gap-1 p-2 rounded-lg border border-border bg-card shadow-xl">
            {CELL_COLORS.map(c => (
              <button
                key={c.label}
                title={c.label}
                onMouseDown={(e) => {
                  e.preventDefault();
                  // Try setCellAttribute (Tiptap table command) first
                  try {
                    (editor.chain().focus() as any).setCellAttribute('backgroundColor', c.value).run();
                  } catch {
                    editor.chain().focus().updateAttributes('tableCell', { backgroundColor: c.value }).run();
                  }
                  setShowColors(false);
                }}
                className={cn(
                  'size-5 rounded border border-border/60 transition-transform hover:scale-125 flex items-center justify-center',
                  c.display,
                )}
              >
                {!c.value && <span className="text-[9px] text-muted-foreground">×</span>}
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
