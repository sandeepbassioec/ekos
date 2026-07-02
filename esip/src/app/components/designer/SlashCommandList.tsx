import { useState, useEffect, useImperativeHandle, forwardRef, useCallback } from 'react';
import type { Editor } from '@tiptap/core';
import type { Range } from '@tiptap/core';

export interface SlashCommandItem {
  title: string;
  description: string;
  emoji: string;
  category: string;
  aliases?: string[];
  command: (editor: Editor, range: Range) => void;
}

interface SlashCommandListProps {
  items: SlashCommandItem[];
  command: (item: SlashCommandItem) => void;
  editor: Editor;
}

export interface SlashCommandListRef {
  onKeyDown: (event: KeyboardEvent) => boolean;
}

export const SlashCommandList = forwardRef<SlashCommandListRef, SlashCommandListProps>(
  ({ items, command }, ref) => {
    const [selected, setSelected] = useState(0);

    useEffect(() => { setSelected(0); }, [items]);

    const upHandler = useCallback(() => {
      setSelected(s => (s - 1 + items.length) % items.length);
    }, [items.length]);

    const downHandler = useCallback(() => {
      setSelected(s => (s + 1) % items.length);
    }, [items.length]);

    const enterHandler = useCallback(() => {
      if (items[selected]) command(items[selected]);
    }, [items, selected, command]);

    useImperativeHandle(ref, () => ({
      onKeyDown: (event: KeyboardEvent) => {
        if (event.key === 'ArrowUp')    { upHandler();    return true; }
        if (event.key === 'ArrowDown')  { downHandler();  return true; }
        if (event.key === 'Enter')      { enterHandler(); return true; }
        return false;
      },
    }));

    if (!items.length) {
      return (
        <div className="bg-card border border-border rounded-xl shadow-xl p-3 text-sm text-muted-foreground">
          No results
        </div>
      );
    }

    const categories = [...new Set(items.map(i => i.category))];

    return (
      <div className="bg-card border border-border rounded-xl shadow-xl overflow-hidden w-72 max-h-80 overflow-y-auto">
        <div className="px-3 py-2 border-b border-border bg-muted/30">
          <p className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide">Insert widget</p>
        </div>
        {categories.map(cat => {
          const catItems = items.filter(i => i.category === cat);
          return (
            <div key={cat}>
              <p className="text-[9px] font-bold uppercase tracking-widest text-muted-foreground/60 px-3 pt-2 pb-1">{cat}</p>
              {catItems.map((item) => {
                const globalIdx = items.indexOf(item);
                return (
                  <button
                    key={item.title}
                    onClick={() => command(item)}
                    onMouseEnter={() => setSelected(globalIdx)}
                    className={`w-full flex items-center gap-3 px-3 py-2 text-left transition-colors ${globalIdx === selected ? 'bg-primary/10' : 'hover:bg-muted/50'}`}
                  >
                    <span className="text-lg leading-none shrink-0">{item.emoji}</span>
                    <div className="min-w-0">
                      <p className={`text-sm font-semibold ${globalIdx === selected ? 'text-primary' : 'text-foreground'}`}>{item.title}</p>
                      <p className="text-[11px] text-muted-foreground truncate">{item.description}</p>
                    </div>
                  </button>
                );
              })}
            </div>
          );
        })}
      </div>
    );
  }
);

SlashCommandList.displayName = 'SlashCommandList';
