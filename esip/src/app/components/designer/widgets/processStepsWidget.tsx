import { Node, mergeAttributes } from '@tiptap/core';
import { ReactNodeViewRenderer, NodeViewWrapper } from '@tiptap/react';
import type { NodeViewProps } from '@tiptap/react';
import { useState, useContext } from 'react';
import { GripVertical, Plus, X, Check } from 'lucide-react';
import { PreviewModeContext } from '../PreviewModeContext';

type StepStatus = 'done' | 'active' | 'todo';

interface Step {
  id: string;
  name: string;
  status: StepStatus;
}

function cycle(current: StepStatus): StepStatus {
  if (current === 'done') return 'active';
  if (current === 'active') return 'todo';
  return 'done';
}

const STEP_CIRCLE: Record<StepStatus, string> = {
  done:   'bg-green-50 border-green-500 text-green-700 dark:bg-green-950/40 dark:border-green-600 dark:text-green-400',
  active: 'bg-primary/10 border-primary text-primary',
  todo:   'bg-muted border-border text-muted-foreground',
};

function parseSteps(raw: string): Step[] {
  try { return JSON.parse(raw); } catch { return []; }
}

function ProcessStepsView({ node, updateAttributes, selected }: NodeViewProps) {
  const isPreview = useContext(PreviewModeContext);
  const steps = parseSteps(node.attrs.steps);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [draftName, setDraftName] = useState('');

  const save = (updated: Step[]) => updateAttributes({ steps: JSON.stringify(updated) });
  const addStep = () => {
    const newStep: Step = { id: Date.now().toString(), name: 'New Step', status: 'todo' };
    save([...steps, newStep]);
    setTimeout(() => { setEditingId(newStep.id); setDraftName(newStep.name); }, 50);
  };
  const updateStep = (id: string, patch: Partial<Step>) => save(steps.map(s => s.id === id ? { ...s, ...patch } : s));
  const removeStep = (id: string) => save(steps.filter(s => s.id !== id));
  const commitEdit = (id: string) => { updateStep(id, { name: draftName }); setEditingId(null); };

  return (
    <NodeViewWrapper>
      <div className={`group relative rounded-xl border border-border bg-card my-3 overflow-hidden shadow-sm ${selected ? 'ring-2 ring-primary/40' : ''}`}>
        <div className="flex items-center gap-2 px-3 py-2 bg-muted/30 border-b border-border">
          {!isPreview && (
            <div className="flex items-center justify-center w-5 cursor-grab opacity-0 group-hover:opacity-60 transition-opacity" data-drag-handle>
              <GripVertical className="size-3.5 text-muted-foreground" />
            </div>
          )}
          <span className="text-[10px]">⚡</span>
          <span className="text-sm font-semibold text-foreground flex-1">
            {node.attrs.title || 'Process Steps'}
          </span>
          <div className="flex items-center gap-2 text-[10px] text-muted-foreground">
            <span className="text-green-600 dark:text-green-400">{steps.filter(s => s.status === 'done').length} done</span>
            <span>·</span>
            <span>{steps.filter(s => s.status === 'todo').length} remaining</span>
          </div>
          {!isPreview && (
            <button onClick={addStep} className="flex items-center gap-1 text-[10px] text-primary hover:text-primary/80 transition-colors opacity-0 group-hover:opacity-100 ml-2">
              <Plus className="size-3" /> Step
            </button>
          )}
        </div>

        <div className="px-4 py-4 overflow-x-auto">
          <div className="flex items-start min-w-max gap-0">
            {steps.map((step, i) => (
              <div key={step.id} className="flex items-start">
                <div className="group/step flex flex-col items-center min-w-[88px] max-w-[120px] relative">
                  <div
                    onClick={!isPreview ? () => updateStep(step.id, { status: cycle(step.status) }) : undefined}
                    className={`w-9 h-9 rounded-full border-2 flex items-center justify-center font-bold text-sm transition-all mb-1.5 ${STEP_CIRCLE[step.status]} ${!isPreview ? 'cursor-pointer' : ''}`}
                    title={!isPreview ? 'Click to change status' : undefined}
                  >
                    {step.status === 'done' ? <Check className="size-4" /> : i + 1}
                  </div>
                  {!isPreview && editingId === step.id ? (
                    <input
                      autoFocus
                      value={draftName}
                      onChange={(e) => setDraftName(e.target.value)}
                      onBlur={() => commitEdit(step.id)}
                      onKeyDown={(e) => e.key === 'Enter' && commitEdit(step.id)}
                      className="text-[10px] text-center bg-transparent border-b border-border focus:outline-none w-full font-semibold text-foreground"
                    />
                  ) : (
                    <span
                      className={`text-[11px] font-semibold text-center leading-tight ${step.status === 'active' ? 'text-primary' : step.status === 'done' ? 'text-muted-foreground' : 'text-foreground'} ${!isPreview ? 'cursor-text' : ''}`}
                      onClick={!isPreview ? () => { setEditingId(step.id); setDraftName(step.name); } : undefined}
                    >
                      {step.name}
                    </span>
                  )}
                  {!isPreview && (
                    <button
                      onClick={(e) => { e.stopPropagation(); removeStep(step.id); }}
                      className="absolute -top-1 -right-1 opacity-0 group-hover/step:opacity-100 transition-opacity text-muted-foreground hover:text-destructive"
                    >
                      <X className="size-2.5" />
                    </button>
                  )}
                </div>
                {i < steps.length - 1 && (
                  <div className="flex items-center pb-6 mx-1">
                    <div className={`h-0.5 w-6 ${step.status === 'done' ? 'bg-green-500' : 'bg-border'}`} />
                    <div className={`text-lg leading-none mb-0.5 -ml-1 ${step.status === 'done' ? 'text-green-500' : 'text-border'}`}>›</div>
                  </div>
                )}
              </div>
            ))}
            {steps.length === 0 && (
              <div className="text-sm text-muted-foreground/60 cursor-pointer py-4" onClick={addStep}>
                Click "+ Step" to build your process flow
              </div>
            )}
          </div>
        </div>
      </div>
    </NodeViewWrapper>
  );
}

const defaultSteps: Step[] = [
  { id: '1', name: 'Requirements', status: 'done' },
  { id: '2', name: 'HLD Review',   status: 'done' },
  { id: '3', name: 'LLD Design',   status: 'active' },
  { id: '4', name: 'Development',  status: 'todo' },
  { id: '5', name: 'QA Testing',   status: 'todo' },
  { id: '6', name: 'Go Live',      status: 'todo' },
];

export const ProcessStepsExtension = Node.create({
  name: 'processSteps',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      title: { default: 'Process Steps' },
      steps: { default: JSON.stringify(defaultSteps) },
    };
  },

  parseHTML() { return [{ tag: 'div[data-type="process-steps"]' }]; },
  renderHTML({ HTMLAttributes }) { return ['div', mergeAttributes({ 'data-type': 'process-steps' }, HTMLAttributes)]; },
  addNodeView() { return ReactNodeViewRenderer(ProcessStepsView); },
});
