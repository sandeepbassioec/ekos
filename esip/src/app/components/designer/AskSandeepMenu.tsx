import { useState, useRef, useCallback, useEffect } from 'react';
import type { Editor } from '@tiptap/react';
import { marked } from 'marked';
import {
  Sparkles, Send, X, Check, Loader2, Target, ChevronDown,
} from 'lucide-react';
import { cn } from '../ui/utils';
import { Button } from '../ui/button';
import { toolboxRegistry } from './toolboxRegistry';
import { generateDesignerContent } from '../../lib/ai/designerAI';
import type { DesignerTemplate, WidgetTarget } from './types';
import type { SelectionMode } from './BubbleFormatMenu';

// ── Empty-line floating trigger ───────────────────────────────────────────────

function useEmptyLineCoords(editor: Editor | null) {
  const [coords, setCoords] = useState<{ top: number } | null>(null);

  useEffect(() => {
    if (!editor) return;
    const update = () => {
      const { $from } = editor.state.selection;
      const node = $from.node();
      const isEmpty = node.type.name === 'paragraph' && node.textContent === '';
      if (!isEmpty) { setCoords(null); return; }
      try {
        const pos = editor.view.coordsAtPos($from.pos);
        const editorDom = editor.view.dom as HTMLElement;
        const rect = editorDom.getBoundingClientRect();
        setCoords({ top: pos.bottom - rect.top + editorDom.scrollTop + 4 });
      } catch { setCoords(null); }
    };
    editor.on('selectionUpdate', update);
    editor.on('update', update);
    return () => { editor.off('selectionUpdate', update); editor.off('update', update); };
  }, [editor]);

  return coords;
}

interface EmptyLineTriggerProps {
  editor: Editor;
  onOpenPanel: () => void;
}

export function EmptyLineTrigger({ editor, onOpenPanel }: EmptyLineTriggerProps) {
  const coords = useEmptyLineCoords(editor);
  if (!coords) return null;

  return (
    <div className="absolute z-10 left-0 right-0 pointer-events-none" style={{ top: coords.top }}>
      <button
        onMouseDown={(e) => { e.preventDefault(); onOpenPanel(); }}
        className="pointer-events-auto flex items-center gap-1.5 rounded-md px-2.5 py-1 text-xs font-medium text-muted-foreground hover:text-primary hover:bg-primary/5 border border-dashed border-border/60 hover:border-primary/40 transition-colors select-none"
      >
        <Sparkles className="size-3 text-primary" />
        <span>Ask Sandeep</span>
      </button>
    </div>
  );
}

// ── ADR field selector ────────────────────────────────────────────────────────

const ADR_FIELDS = [
  { id: 'context',      label: 'Context',      hint: 'Why this decision was needed' },
  { id: 'decision',     label: 'Decision',     hint: 'What was decided' },
  { id: 'consequences', label: 'Consequences', hint: 'Trade-offs and outcomes' },
] as const;

type AdrField = typeof ADR_FIELDS[number]['id'];

// ── Widget type display info ──────────────────────────────────────────────────

const WIDGET_META: Record<string, { icon: string; color: string; fieldLabel: (attrs: Record<string, string>, field: string) => string }> = {
  callout:      { icon: '💬', color: 'text-blue-600 dark:text-blue-400',   fieldLabel: (a) => `body of "${a.title || 'callout'}"` },
  requirement:  { icon: '📋', color: 'text-violet-600 dark:text-violet-400', fieldLabel: (a) => `description of ${a.reqId || 'requirement'}` },
  adr:          { icon: '🏛',  color: 'text-amber-600 dark:text-amber-400',  fieldLabel: (a, f) => `${f} of "${a.title || 'ADR'}"` },
  diagram:      { icon: '◈',  color: 'text-emerald-600 dark:text-emerald-400', fieldLabel: (a) => `Mermaid source for "${a.title || 'diagram'}"` },
};

// ── Main panel ────────────────────────────────────────────────────────────────

interface AskSandeepPanelProps {
  editor: Editor;
  template: DesignerTemplate;
  widgetTarget: WidgetTarget | null;
  onClearTarget: () => void;
  selectionMode: SelectionMode | null;
  onSelectionModeClose: () => void;
  onClose: () => void;
}

export function AskSandeepPanel({
  editor,
  template,
  widgetTarget,
  onClearTarget,
  selectionMode,
  onSelectionModeClose,
  onClose,
}: AskSandeepPanelProps) {
  const [selectedActionId, setSelectedActionId] = useState<string>('write-section');
  const [userInput, setUserInput] = useState('');
  const [generating, setGenerating] = useState(false);
  const [generated, setGenerated] = useState('');
  const [error, setError] = useState('');
  const [adrField, setAdrField] = useState<AdrField>('context');
  const inputRef = useRef<HTMLTextAreaElement>(null);

  const actions = toolboxRegistry.forTemplate(template.toolboxActions);
  const activeAction = actions.find(a => a.id === selectedActionId) ?? actions[0];

  const isSelectionMode = !!selectionMode && !widgetTarget;
  const isWidgetMode = !!widgetTarget;

  const widgetMeta = widgetTarget ? WIDGET_META[widgetTarget.nodeType] : null;
  const effectiveField = widgetTarget?.nodeType === 'adr' ? adrField : widgetTarget?.targetField ?? '';

  // Reset state when target/mode changes
  useEffect(() => {
    setGenerated('');
    setError('');
    if (widgetTarget) {
      // Auto-select the best action for the widget type
      const bestAction = widgetTarget.nodeType === 'diagram' ? 'generate-diagram' : 'write-section';
      const avail = toolboxRegistry.forTemplate(template.toolboxActions);
      setSelectedActionId(avail.find(a => a.id === bestAction) ? bestAction : (avail[0]?.id ?? 'write-section'));
      setUserInput('');
      setAdrField('context');
    } else if (selectionMode) {
      setSelectedActionId('improve');
      setUserInput(selectionMode.text);
    }
    setTimeout(() => inputRef.current?.focus(), 50);
  }, [widgetTarget, selectionMode]); // eslint-disable-line react-hooks/exhaustive-deps

  const getContextText = useCallback(() => {
    if (widgetTarget) {
      const { nodeType, attrs } = widgetTarget;
      const lines = [`[${nodeType.toUpperCase()} WIDGET]`];
      Object.entries(attrs).forEach(([k, v]) => { if (v) lines.push(`${k}: ${v}`); });
      return lines.join('\n');
    }
    if (selectionMode) return selectionMode.text;
    const { from } = editor.state.selection;
    const doc = editor.state.doc;
    const start = Math.max(0, from - 500);
    const end = Math.min(doc.content.size, from + 200);
    return doc.textBetween(start, end, '\n');
  }, [editor, widgetTarget, selectionMode]);

  const getPlaceholder = () => {
    if (isWidgetMode) {
      const fieldLabel = widgetMeta?.fieldLabel(widgetTarget!.attrs, effectiveField) ?? effectiveField;
      return `Describe what to generate for the ${fieldLabel}…`;
    }
    if (isSelectionMode) return 'Add instructions, e.g. "make it more concise" or "convert to bullet points"…';
    return activeAction?.placeholder ?? 'Describe what you want…';
  };

  // Detect if prompt is asking for a BPMN process diagram
  const isBpmnRequest = (text: string) =>
    /\b(bpmn|business process|process model|process diagram|workflow diagram|swim.?lane|swimlane|process flow)\b/i.test(text);

  // Detect if prompt is asking for a Mermaid diagram
  const isDiagramRequest = (text: string) =>
    /\b(diagram|flowchart|sequence|flow|er diagram|entity.relation|class diagram|state diagram|gantt|mindmap|architecture diagram|draw|chart)\b/i.test(text);

  const handleGenerate = async () => {
    if (!userInput.trim()) return;
    setGenerating(true);
    setError('');
    setGenerated('');
    try {
      const contextText = getContextText();
      let prompt: string;
      if (!isWidgetMode && !isSelectionMode && isBpmnRequest(userInput)) {
        prompt = `Generate a valid BPMN 2.0 XML diagram for: ${userInput}\n\nContext from document:\n${contextText}\n\nReturn ONLY valid BPMN 2.0 XML starting with <?xml. Include bpmndi:BPMNDiagram layout so it renders visually. Use realistic positions and shapes. No explanation.`;
      } else if (!isWidgetMode && !isSelectionMode && isDiagramRequest(userInput)) {
        prompt = `Generate a Mermaid diagram for: ${userInput}\n\nContext from document:\n${contextText}\n\nReturn ONLY the raw Mermaid code block (no explanation, no markdown fences). Start directly with the diagram type keyword (e.g. flowchart TD, sequenceDiagram, erDiagram, classDiagram, stateDiagram-v2, gantt, mindmap). Choose the most appropriate Mermaid diagram type.`;
      } else {
        prompt = activeAction.buildPrompt(userInput, contextText, template);
      }
      const result = await generateDesignerContent(prompt);
      setGenerated(result.trim());
    } catch (err) {
      setError((err as Error).message || 'AI generation failed. Check that the AI proxy is running.');
    } finally {
      setGenerating(false);
    }
  };

  const handleApply = () => {
    if (!generated) return;

    if (isWidgetMode && widgetTarget) {
      // Update the widget's attr directly (no HTML conversion — plain text goes into attrs)
      const { pos, nodeType, attrs } = widgetTarget;
      const field = nodeType === 'adr' ? adrField : widgetTarget.targetField;
      editor.chain().focus().command(({ tr }) => {
        try {
          const currentNode = tr.doc.nodeAt(pos);
          const freshAttrs = currentNode?.attrs ?? attrs;
          tr.setNodeMarkup(pos, undefined, { ...freshAttrs, [field]: generated });
          return true;
        } catch { return false; }
      }).run();
    } else if (isSelectionMode && selectionMode) {
      const html = marked.parse(generated) as string;
      editor.chain().focus()
        .deleteRange({ from: selectionMode.from, to: selectionMode.to })
        .insertContent(html)
        .run();
    } else if (!isSelectionMode && isBpmnRequest(userInput) && /^<\?xml/.test(generated)) {
      // Insert as a BPMN widget
      const title = userInput.replace(/\b(bpmn|diagram|process|model|flow|draw|create|generate|make|a|an|the)\b/gi, '').trim().replace(/\s+/g, ' ') || 'Process';
      editor.chain().focus().insertContent({ type: 'bpmn', attrs: { title: title.charAt(0).toUpperCase() + title.slice(1), xml: generated } }).run();
    } else if (!isSelectionMode && isDiagramRequest(userInput) && /^(flowchart|sequenceDiagram|erDiagram|classDiagram|stateDiagram|gantt|mindmap|graph |pie |xychart|quadrantChart|requirementDiagram|gitGraph)/i.test(generated)) {
      // Insert as a Mermaid diagram widget
      const title = userInput.replace(/\b(diagram|flowchart|chart|draw|create|generate|make|a|an|the)\b/gi, '').trim().replace(/\s+/g, ' ') || 'Diagram';
      editor.chain().focus().insertContent({ type: 'diagram', attrs: { title: title.charAt(0).toUpperCase() + title.slice(1), source: generated } }).run();
    } else {
      const html = marked.parse(generated) as string;
      editor.chain().focus().insertContent(html).run();
    }

    setGenerated('');
    setUserInput('');
    if (isSelectionMode) onSelectionModeClose();
    if (isWidgetMode) onClearTarget();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) { e.preventDefault(); handleGenerate(); }
    if (e.key === 'Escape') onClose();
  };

  const applyLabel = isWidgetMode ? 'Update widget' : isSelectionMode ? 'Replace selection'
    : (!isWidgetMode && !isSelectionMode && isBpmnRequest(userInput) && generated) ? 'Insert BPMN'
    : (!isWidgetMode && !isSelectionMode && isDiagramRequest(userInput) && generated) ? 'Insert diagram'
    : 'Insert';

  return (
    <div className="flex flex-col h-full border-l border-border bg-card overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between gap-2 px-4 py-3 border-b border-border bg-primary/5 shrink-0">
        <div className="flex items-center gap-2 min-w-0">
          <div className="flex size-6 items-center justify-center rounded-full bg-primary/10 shrink-0">
            <Sparkles className="size-3.5 text-primary" />
          </div>
          <span className="text-sm font-semibold truncate">Ask Sandeep</span>
          <span className="rounded-full bg-primary/10 px-2 py-0.5 text-[10px] font-medium text-primary shrink-0">AI</span>
        </div>
        <button
          onClick={onClose}
          className="rounded p-1 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors shrink-0"
        >
          <X className="size-4" />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-4">

        {/* Widget target indicator */}
        {isWidgetMode && widgetTarget && (
          <div className="rounded-lg border border-primary/30 bg-primary/5 p-3">
            <div className="flex items-start justify-between gap-2">
              <div className="flex items-center gap-2 min-w-0">
                <Target className="size-3.5 text-primary shrink-0" />
                <span className="text-xs font-semibold text-primary">Targeting widget</span>
              </div>
              <button
                onClick={onClearTarget}
                className="text-[10px] text-muted-foreground hover:text-foreground transition-colors shrink-0"
              >
                clear
              </button>
            </div>
            <p className={cn('text-xs font-medium mt-1.5', widgetMeta?.color)}>
              {widgetMeta?.icon} {widgetTarget.displayName}
            </p>

            {/* ADR field picker */}
            {widgetTarget.nodeType === 'adr' && (
              <div className="mt-2.5">
                <p className="text-[10px] text-muted-foreground mb-1.5 font-medium uppercase tracking-wide">Fill field</p>
                <div className="flex gap-1.5">
                  {ADR_FIELDS.map(f => (
                    <button
                      key={f.id}
                      onClick={() => setAdrField(f.id)}
                      className={cn(
                        'flex-1 rounded px-2 py-1 text-[10px] font-medium border transition-colors',
                        adrField === f.id
                          ? 'bg-primary text-primary-foreground border-primary'
                          : 'bg-background text-muted-foreground border-border hover:border-primary/50',
                      )}
                    >
                      {f.label}
                    </button>
                  ))}
                </div>
              </div>
            )}

            {/* Non-ADR field label */}
            {widgetTarget.nodeType !== 'adr' && (
              <p className="text-[10px] text-muted-foreground mt-1">
                Output will fill: <span className="text-foreground/70 font-medium">{effectiveField}</span>
              </p>
            )}
          </div>
        )}

        {/* Selection indicator */}
        {isSelectionMode && selectionMode && (
          <div className="rounded-lg bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 p-3">
            <p className="text-[10px] font-semibold text-amber-700 dark:text-amber-400 uppercase tracking-wide mb-1">
              Working with selected text
            </p>
            <p className="text-xs text-foreground/70 line-clamp-3 leading-relaxed font-mono">
              {selectionMode.text}
            </p>
          </div>
        )}

        {/* Action chips */}
        <div>
          <p className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide mb-2">Action</p>
          <div className="flex flex-wrap gap-1.5">
            {actions.map(action => (
              <button
                key={action.id}
                onClick={() => { setSelectedActionId(action.id); setGenerated(''); setError(''); }}
                className={cn(
                  'rounded-full px-3 py-1 text-xs font-medium border transition-colors',
                  selectedActionId === action.id
                    ? 'bg-primary text-primary-foreground border-primary'
                    : 'bg-background text-muted-foreground border-border hover:border-primary/50 hover:text-foreground',
                )}
              >
                {action.label}
              </button>
            ))}
          </div>
        </div>

        {/* Prompt input */}
        <div>
          <textarea
            ref={inputRef}
            value={userInput}
            onChange={(e) => setUserInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={getPlaceholder()}
            rows={3}
            className="w-full resize-none rounded-lg border border-border bg-background px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/40"
          />
          <div className="flex items-center justify-between mt-2">
            <span className="text-[11px] text-muted-foreground">⌘↵ to generate</span>
            <Button
              size="sm"
              onClick={handleGenerate}
              disabled={generating || !userInput.trim()}
              className="gap-1.5 h-7 px-3 text-xs"
            >
              {generating ? <Loader2 className="size-3 animate-spin" /> : <Send className="size-3" />}
              {generating ? 'Generating…' : 'Generate'}
            </Button>
          </div>
        </div>

        {/* Error */}
        {error && (
          <div className="rounded-lg bg-destructive/10 px-3 py-2 text-xs text-destructive">
            {error}
          </div>
        )}

        {/* Preview */}
        {generated && (
          <div className="rounded-lg bg-muted/40 border border-border overflow-hidden">
            <div className="flex items-center justify-between px-3 py-2 bg-muted/60 border-b border-border">
              <span className="text-[11px] font-medium text-muted-foreground uppercase tracking-wide">Preview</span>
              <button
                onClick={() => setGenerated('')}
                className="text-[10px] text-muted-foreground hover:text-foreground transition-colors"
              >
                Regenerate
              </button>
            </div>
            <div
              className="max-h-56 overflow-y-auto px-3 py-2.5 text-sm text-foreground leading-relaxed prose prose-sm dark:prose-invert max-w-none"
              dangerouslySetInnerHTML={{ __html: marked.parse(generated) as string }}
            />
            <div className="flex justify-end gap-2 p-3 border-t border-border bg-muted/20">
              <Button
                variant="outline" size="sm"
                onClick={() => { setGenerated(''); setUserInput(''); if (isSelectionMode) onSelectionModeClose(); if (isWidgetMode) onClearTarget(); }}
                className="h-7 px-3 text-xs"
              >
                Discard
              </Button>
              <Button
                size="sm"
                onClick={handleApply}
                className="h-7 px-3 text-xs gap-1.5"
              >
                <Check className="size-3" />
                {applyLabel}
              </Button>
            </div>
          </div>
        )}
      </div>

      {/* Footer hint */}
      <div className="shrink-0 border-t border-border px-4 py-2.5 bg-muted/20">
        <p className="text-[10px] text-muted-foreground leading-relaxed">
          {isWidgetMode
            ? `AI output will update the ${effectiveField} field of the targeted widget.`
            : isSelectionMode
              ? 'AI output will replace your current selection.'
              : 'Hover any widget and click ✨ to target it, or output will be inserted at the cursor.'}
        </p>
      </div>
    </div>
  );
}

// ── Legacy export kept for BubbleFormatMenu integration ───────────────────────
// The BubbleFormatMenu fires selectionMode; DesignerShell forwards it to the panel.
export type { SelectionMode };

// Suppress unused import warning — ChevronDown used conditionally in future
void ChevronDown;
