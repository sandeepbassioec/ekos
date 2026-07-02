import { Extension } from '@tiptap/core';
import { Plugin, PluginKey } from '@tiptap/pm/state';
import type { JSONContent } from '@tiptap/react';

// ── Types ─────────────────────────────────────────────────────────────────────

export interface SmartPasteInfo {
  text: string;
  from: number;
  to: number;
  detectedTypes: DetectedType[];
}

export type DetectedType = 'requirement' | 'adr' | 'callout' | 'processSteps';

const TYPE_LABELS: Record<DetectedType, string> = {
  requirement: 'requirements',
  adr: 'ADR',
  callout: 'callouts',
  processSteps: 'process steps',
};

// ── Pattern detection ─────────────────────────────────────────────────────────

export function analyzeText(text: string): DetectedType[] {
  const types: DetectedType[] = [];
  if (/(REQ|FUNC|NFR|UC|FR)-\d+/i.test(text) || /\bthe system (SHALL|SHOULD|MAY)\b/i.test(text))
    types.push('requirement');
  if (/ADR-\d+/i.test(text) || (/^Context:/im.test(text) && /^Decision:/im.test(text)))
    types.push('adr');
  if (/(^|\n)(NOTE|WARNING|IMPORTANT|RISK|INFO)[:：\s]/im.test(text))
    types.push('callout');
  if (/^\d+[.)]\s+.+[\r\n]+\d+[.)]\s+.+[\r\n]+\d+[.)]/m.test(text))
    types.push('processSteps');
  return types;
}

export function detectedSummary(types: DetectedType[]): string {
  return types.map(t => TYPE_LABELS[t]).join(', ');
}

// ── Regex transformer: text → widget JSONContent[] ────────────────────────────

interface Step { id: string; name: string; status: 'done' | 'active' | 'todo' }

function makeProcessStepsNode(steps: string[]): JSONContent {
  const stepObjs: Step[] = steps.map((name, i) => ({
    id: String(Date.now() + i),
    name,
    status: 'todo',
  }));
  return {
    type: 'processSteps',
    attrs: {
      title: 'Process Steps',
      steps: JSON.stringify(stepObjs),
    },
  };
}

function makeCalloutNode(rawType: string, title: string, body: string): JSONContent {
  const upper = rawType.toUpperCase();
  const type =
    upper === 'WARNING' || upper === 'CAUTION' ? 'warning' :
    upper === 'RISK' ? 'danger' :
    upper === 'NOTE' || upper === 'INFO' ? 'info' :
    'info';
  return { type: 'callout', attrs: { type, title: title || rawType, body: body.trim() } };
}

function makeRequirementNode(reqId: string, text: string, idx: number): JSONContent {
  const upper = text.toUpperCase();
  const reqType = upper.includes('SHALL') ? 'SHALL' : upper.includes('SHOULD') ? 'SHOULD' : 'MAY';
  const id = reqId || `REQ-${String(idx + 1).padStart(3, '0')}`;
  return {
    type: 'requirement',
    attrs: { reqId: id.toUpperCase(), reqType, priority: 'HIGH', status: 'OPEN', description: text.trim(), tags: '' },
  };
}

function makeAdrNode(id: string, title: string, context: string, decision: string, consequences: string): JSONContent {
  return {
    type: 'adr',
    attrs: {
      adrId: id || 'ADR-001',
      title: title || 'Decision',
      status: 'PROPOSED',
      context: context.trim(),
      decision: decision.trim(),
      consequences: consequences.trim(),
    },
  };
}

export function parseToWidgetNodes(text: string): JSONContent[] {
  const lines = text.split(/\r?\n/);
  const nodes: JSONContent[] = [];
  let pendingSteps: string[] = [];

  const flushSteps = () => {
    if (pendingSteps.length >= 2) {
      nodes.push(makeProcessStepsNode(pendingSteps));
      pendingSteps = [];
    }
  };

  // Check if this is a full ADR block
  const hasContext = /^Context:/im.test(text);
  const hasDecision = /^Decision:/im.test(text);
  if (hasContext && hasDecision) {
    const adrIdMatch = text.match(/^(ADR-\d+)[:\s]+(.+)/im);
    const contextMatch = text.match(/^Context:\s*([\s\S]+?)(?=^Decision:|^Consequences:|$)/im);
    const decisionMatch = text.match(/^Decision:\s*([\s\S]+?)(?=^Consequences:|^Context:|$)/im);
    const consequencesMatch = text.match(/^Consequences:\s*([\s\S]+?)(?=^Context:|^Decision:|$)/im);
    nodes.push(makeAdrNode(
      adrIdMatch?.[1] || 'ADR-001',
      adrIdMatch?.[2]?.trim() || 'Architecture Decision',
      contextMatch?.[1]?.trim() || '',
      decisionMatch?.[1]?.trim() || '',
      consequencesMatch?.[1]?.trim() || '',
    ));
    return nodes;
  }

  let reqIdx = 0;
  for (const raw of lines) {
    const line = raw.trim();
    if (!line) {
      flushSteps();
      continue;
    }

    // Requirement: REQ-001: ... or "The system SHALL ..."
    const reqIdMatch = line.match(/^((REQ|FUNC|NFR|UC|FR)-\d+)[:\s]+(.+)/i);
    const shallMatch = line.match(/^The system (SHALL|SHOULD|MAY)\s+(.+)/i);
    if (reqIdMatch) {
      flushSteps();
      nodes.push(makeRequirementNode(reqIdMatch[1], reqIdMatch[3], reqIdx++));
      continue;
    }
    if (shallMatch) {
      flushSteps();
      nodes.push(makeRequirementNode('', line, reqIdx++));
      continue;
    }

    // Callout: NOTE: / WARNING: / RISK: etc.
    const calloutMatch = line.match(/^(NOTE|WARNING|IMPORTANT|RISK|INFO|CAUTION)[:\s*]+(.*)/i);
    if (calloutMatch) {
      flushSteps();
      nodes.push(makeCalloutNode(calloutMatch[1], calloutMatch[1], calloutMatch[2]));
      continue;
    }

    // Numbered step
    const stepMatch = line.match(/^(\d+)[.)]\s+(.+)/);
    if (stepMatch) {
      pendingSteps.push(stepMatch[2].trim());
      continue;
    }

    // Flush steps before adding paragraph
    flushSteps();

    nodes.push({
      type: 'paragraph',
      content: line ? [{ type: 'text', text: line }] : [],
    });
  }

  flushSteps();
  return nodes;
}

// ── Tiptap extension ──────────────────────────────────────────────────────────

const SMART_PASTE_KEY = new PluginKey<null>('smartPaste');

export const SmartPasteExtension = Extension.create({
  name: 'smartPaste',

  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: SMART_PASTE_KEY,
        props: {
          handlePaste(view, event) {
            const text = event.clipboardData?.getData('text/plain') ?? '';
            if (text.length < 15) return false;

            const detectedTypes = analyzeText(text);
            if (detectedTypes.length === 0) return false;

            // Let the default paste happen, then grab cursor position as 'to'
            const from = view.state.selection.from;
            setTimeout(() => {
              const to = view.state.selection.from;
              document.dispatchEvent(new CustomEvent('smart-paste-detected', {
                bubbles: true,
                detail: { text, from, to, detectedTypes } satisfies SmartPasteInfo,
              }));
            }, 0);

            return false; // don't intercept — let Tiptap insert the text normally
          },
        },
      }),
    ];
  },
});
