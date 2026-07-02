import { Extension } from '@tiptap/core';
import type { Editor, Range } from '@tiptap/core';
import Suggestion from '@tiptap/suggestion';
import { createRoot } from 'react-dom/client';
import { createElement, createRef } from 'react';
import { SlashCommandList } from './SlashCommandList';
import type { SlashCommandItem, SlashCommandListRef } from './SlashCommandList';

// ── Widget command definitions ────────────────────────────────────────────────

const COMMANDS: SlashCommandItem[] = [
  // Widgets
  {
    title: 'Callout',
    description: 'Info, warning, decision or risk block',
    emoji: 'ℹ️',
    category: 'Widgets',
    aliases: ['note', 'info', 'warning', 'risk', 'decision'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).insertContent({ type: 'callout', attrs: { type: 'info', title: 'Note', body: '' } }).run();
    },
  },
  {
    title: 'Requirement',
    description: 'REQ card with SHALL / status / priority',
    emoji: '📋',
    category: 'Widgets',
    aliases: ['req', 'shall', 'requirement'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).insertContent({ type: 'requirement' }).run();
    },
  },
  {
    title: 'ADR',
    description: 'Architecture Decision Record',
    emoji: '🏛',
    category: 'Widgets',
    aliases: ['adr', 'decision', 'architecture'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).insertContent({ type: 'adr' }).run();
    },
  },
  {
    title: 'Metrics',
    description: 'KPI / OKR tiles with trend arrows',
    emoji: '📊',
    category: 'Widgets',
    aliases: ['kpi', 'okr', 'metrics', 'dashboard'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).insertContent({ type: 'metrics' }).run();
    },
  },
  {
    title: 'Process Steps',
    description: 'Visual step-by-step workflow flow',
    emoji: '⚡',
    category: 'Widgets',
    aliases: ['steps', 'process', 'workflow'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).insertContent({ type: 'processSteps' }).run();
    },
  },
  {
    title: 'NFR & Quality Attributes',
    description: 'Structured non-functional requirements across Performance, Security, Compliance…',
    emoji: '🎯',
    category: 'Widgets',
    aliases: ['nfr', 'quality', 'non-functional', 'performance', 'security', 'sla', 'availability', 'compliance'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).insertContent({ type: 'nfr', attrs: { data: '' } }).run();
    },
  },
  {
    title: 'Trade-off Analysis',
    description: 'Compare 2–4 options across weighted criteria',
    emoji: '⚖️',
    category: 'Widgets',
    aliases: ['tradeoff', 'trade', 'compare', 'options', 'decision', 'versus', 'vs'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).insertContent({ type: 'tradeoff', attrs: { data: '' } }).run();
    },
  },
  {
    title: 'Flow Diagram',
    description: 'Interactive draw.io-style flowchart with shapes',
    emoji: '⬡',
    category: 'Widgets',
    aliases: ['flow', 'flowchart', 'flowdiagram', 'drawio'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).insertContent({ type: 'flowDiagram', attrs: { title: '', xml: '' } }).run();
    },
  },
  {
    title: 'Diagram',
    description: 'Mermaid flowchart / sequence / ER',
    emoji: '◈',
    category: 'Widgets',
    aliases: ['mermaid', 'diagram', 'sequence', 'chart'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).insertContent({ type: 'diagram' }).run();
    },
  },
  {
    title: 'Status Badge',
    description: 'Inline Done / In Progress / Blocked pill',
    emoji: '🏷',
    category: 'Widgets',
    aliases: ['status', 'badge', 'done', 'blocked'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).insertContent({ type: 'statusBadge' }).run();
    },
  },
  {
    title: 'Divider',
    description: 'Section separator with optional label',
    emoji: '─',
    category: 'Widgets',
    aliases: ['hr', 'divider', 'separator', 'section'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).insertContent({ type: 'widgetDivider' }).run();
    },
  },

  // Formatting
  {
    title: 'Heading 1',
    description: 'Large section title',
    emoji: 'H1',
    category: 'Formatting',
    aliases: ['h1', 'title', 'heading1'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).setHeading({ level: 1 }).run();
    },
  },
  {
    title: 'Heading 2',
    description: 'Medium section heading',
    emoji: 'H2',
    category: 'Formatting',
    aliases: ['h2', 'heading2', 'section'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).setHeading({ level: 2 }).run();
    },
  },
  {
    title: 'Heading 3',
    description: 'Small subsection heading',
    emoji: 'H3',
    category: 'Formatting',
    aliases: ['h3', 'heading3', 'subsection'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).setHeading({ level: 3 }).run();
    },
  },
  {
    title: 'Bullet List',
    description: 'Unordered list of items',
    emoji: '•',
    category: 'Formatting',
    aliases: ['ul', 'bullet', 'list', 'unordered'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).toggleBulletList().run();
    },
  },
  {
    title: 'Numbered List',
    description: 'Ordered / numbered list',
    emoji: '1.',
    category: 'Formatting',
    aliases: ['ol', 'numbered', 'ordered'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).toggleOrderedList().run();
    },
  },
  {
    title: 'Quote',
    description: 'Block quote for citations',
    emoji: '❝',
    category: 'Formatting',
    aliases: ['blockquote', 'quote', 'cite'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).toggleBlockquote().run();
    },
  },
  {
    title: 'Code Block',
    description: 'Monospace code snippet',
    emoji: '</>',
    category: 'Formatting',
    aliases: ['code', 'pre', 'snippet', 'codeblock'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).toggleCodeBlock().run();
    },
  },
  {
    title: 'Table',
    description: 'Spreadsheet-style table',
    emoji: '⊞',
    category: 'Formatting',
    aliases: ['table', 'grid'],
    command(editor, range) {
      editor.chain().focus().deleteRange(range).insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run();
    },
  },
];

// ── Extension ─────────────────────────────────────────────────────────────────

export const SlashCommandExtension = Extension.create({
  name: 'slashCommand',

  addProseMirrorPlugins() {
    return [
      Suggestion({
        editor: this.editor,
        char: '/',
        allowSpaces: false,
        startOfLine: false,
        command: ({ editor, range, props }: { editor: Editor; range: Range; props: SlashCommandItem }) => {
          props.command(editor, range);
        },
        items: ({ query }: { query: string }) => {
          const q = query.toLowerCase().trim();
          if (!q) return COMMANDS;
          return COMMANDS.filter(item =>
            item.title.toLowerCase().includes(q) ||
            item.description.toLowerCase().includes(q) ||
            (item.aliases || []).some(a => a.includes(q))
          );
        },
        render: () => {
          let container: HTMLElement | null = null;
          let root: ReturnType<typeof createRoot> | null = null;
          const listRef = createRef<SlashCommandListRef>();

          const renderList = (props: { items: SlashCommandItem[]; command: (item: SlashCommandItem) => void; editor: Editor; clientRect?: (() => DOMRect | null) | null }) => {
            if (!root || !container) return;
            root.render(
              createElement(SlashCommandList, {
                ref: listRef,
                items: props.items,
                command: props.command,
                editor: props.editor,
              })
            );
          };

          const position = (clientRect: (() => DOMRect | null) | null | undefined) => {
            if (!container || !clientRect) return;
            const rect = clientRect();
            if (!rect) return;
            container.style.top = `${rect.bottom + window.scrollY + 6}px`;
            container.style.left = `${rect.left + window.scrollX}px`;
          };

          return {
            onStart(props: { items: SlashCommandItem[]; command: (item: SlashCommandItem) => void; editor: Editor; clientRect?: (() => DOMRect | null) | null }) {
              container = document.createElement('div');
              container.style.cssText = 'position:absolute;z-index:9999;';
              document.body.appendChild(container);
              root = createRoot(container);
              renderList(props);
              position(props.clientRect);
            },
            onUpdate(props: { items: SlashCommandItem[]; command: (item: SlashCommandItem) => void; editor: Editor; clientRect?: (() => DOMRect | null) | null }) {
              renderList(props);
              position(props.clientRect);
            },
            onKeyDown({ event }: { event: KeyboardEvent }) {
              return listRef.current?.onKeyDown(event) ?? false;
            },
            onExit() {
              if (root) { root.unmount(); root = null; }
              if (container) { container.remove(); container = null; }
            },
          };
        },
      }),
    ];
  },
});
