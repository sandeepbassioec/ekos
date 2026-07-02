import { useState } from 'react';
import type { Editor } from '@tiptap/react';
import { Sparkles, Wand2, X, CheckCheck, Loader2 } from 'lucide-react';
import { cn } from '../ui/utils';
import { generateDesignerContent } from '../../lib/ai/designerAI';
import { parseToWidgetNodes, detectedSummary } from './SmartPasteExtension';
import type { SmartPasteInfo } from './SmartPasteExtension';
import type { DesignerTemplate } from './types';
import type { JSONContent } from '@tiptap/react';

interface SmartPasteBannerProps {
  editor: Editor;
  info: SmartPasteInfo;
  template: DesignerTemplate;
  onDismiss: () => void;
}

export function SmartPasteBanner({ editor, info, template, onDismiss }: SmartPasteBannerProps) {
  const [aiLoading, setAiLoading] = useState(false);
  const [applied, setApplied] = useState(false);

  const summary = detectedSummary(info.detectedTypes);

  function replaceRange(nodes: JSONContent[]) {
    const docSize = editor.state.doc.content.size;
    const from = Math.min(info.from, docSize);
    const to = Math.min(info.to, docSize);

    editor.chain()
      .focus()
      .command(({ tr, dispatch }) => {
        if (dispatch) {
          tr.delete(from, to);
          dispatch(tr);
        }
        return true;
      })
      .insertContentAt(from, nodes)
      .run();

    setApplied(true);
    setTimeout(onDismiss, 800);
  }

  const handleApplyRegex = () => {
    const nodes = parseToWidgetNodes(info.text);
    if (nodes.length > 0) replaceRange(nodes);
    else onDismiss();
  };

  const handleAiFormat = async () => {
    setAiLoading(true);
    try {
      const prompt = `You are formatting pasted text into structured document widgets for a ${template.name}.

PASTED TEXT:
${info.text}

AVAILABLE WIDGET TYPES:
- requirement: { type: "requirement", attrs: { reqId: "REQ-001", reqType: "SHALL"|"SHOULD"|"MAY", priority: "HIGH"|"MEDIUM"|"LOW", status: "OPEN", description: "...", tags: "" } }
- callout: { type: "callout", attrs: { type: "info"|"warning"|"success"|"danger", title: "...", body: "..." } }
- adr: { type: "adr", attrs: { adrId: "ADR-001", title: "...", status: "PROPOSED", context: "...", decision: "...", consequences: "..." } }
- processSteps: { type: "processSteps", attrs: { title: "Process Steps", steps: "<JSON array of {id,name,status}>" } }
- paragraph: { type: "paragraph", content: [{ type: "text", text: "..." }] }

TASK: Convert the pasted text into an array of widget nodes. Preserve all information — don't omit any content.
Return ONLY a valid JSON array (no fences, no explanation):
[{ "type": "...", "attrs": { ... } }, ...]

Rules:
- Use requirement widgets for any requirement-like statements (SHALL/SHOULD/MAY, REQ-NNN, functional requirements)
- Use callout widgets for notes, warnings, risks, decisions
- Use adr widgets for architecture decision records
- Use processSteps for numbered sequential steps
- Use paragraph for everything else
- For processSteps.attrs.steps, serialize as a JSON string: "[{\\"id\\":\\"1\\",\\"name\\":\\"..\\",\\"status\\":\\"todo\\"}]"`;

      const raw = await generateDesignerContent(prompt);
      const stripped = raw.replace(/^```(?:json)?\s*/m, '').replace(/\s*```\s*$/m, '').trim();
      const nodes: JSONContent[] = JSON.parse(stripped);
      replaceRange(nodes);
    } catch (err) {
      console.error('Smart paste AI failed:', err);
      // Fallback to regex
      handleApplyRegex();
    } finally {
      setAiLoading(false);
    }
  };

  if (applied) {
    return (
      <div className="flex items-center gap-2 rounded-lg border border-green-200 dark:border-green-800 bg-green-50 dark:bg-green-900/20 px-3 py-2 text-xs text-green-700 dark:text-green-400 mb-3 animate-in fade-in">
        <CheckCheck className="size-3.5 shrink-0" />
        <span>Widgets applied successfully.</span>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-3 rounded-lg border border-primary/30 bg-primary/5 px-3 py-2.5 mb-3">
      <div className="flex size-6 items-center justify-center rounded-full bg-primary/10 shrink-0">
        <Wand2 className="size-3.5 text-primary" />
      </div>

      <div className="flex-1 min-w-0">
        <p className="text-xs font-semibold text-foreground leading-tight">Smart Paste detected</p>
        <p className="text-[11px] text-muted-foreground mt-0.5 truncate">
          Recognised: <span className="text-foreground/80 font-medium">{summary}</span>
        </p>
      </div>

      <div className="flex items-center gap-1.5 shrink-0">
        <button
          onClick={handleApplyRegex}
          className="flex items-center gap-1 rounded-md bg-primary text-primary-foreground text-[11px] font-medium px-2.5 py-1 hover:bg-primary/90 transition-colors"
        >
          <Wand2 className="size-3" />
          Apply widgets
        </button>
        <button
          onClick={handleAiFormat}
          disabled={aiLoading}
          className={cn(
            'flex items-center gap-1 rounded-md border border-border bg-background text-[11px] font-medium px-2.5 py-1 hover:border-primary/50 transition-colors text-foreground',
            aiLoading && 'opacity-60 pointer-events-none',
          )}
        >
          {aiLoading ? <Loader2 className="size-3 animate-spin" /> : <Sparkles className="size-3 text-primary" />}
          {aiLoading ? 'Formatting…' : 'AI format'}
        </button>
        <button
          onClick={onDismiss}
          className="rounded p-1 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
          title="Keep as plain text"
        >
          <X className="size-3.5" />
        </button>
      </div>
    </div>
  );
}
