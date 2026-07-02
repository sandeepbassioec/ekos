import { CheckCircle2, XCircle, Loader2, ChevronDown, ChevronUp, X, Cpu } from 'lucide-react';
import { useKnowledgeHubStore } from '../../store/knowledgeHubStore';
import { cn } from '../ui/utils';

function fmt(n: number): string {
  return n >= 1000 ? `${(n / 1000).toFixed(1)}k` : String(n);
}

export function AnalysisProgressModal() {
  const analysisProgress = useKnowledgeHubStore((s) => s.analysisProgress);
  const minimizeProgress = useKnowledgeHubStore((s) => s.minimizeProgress);
  const maximizeProgress = useKnowledgeHubStore((s) => s.maximizeProgress);
  const dismissProgress = useKnowledgeHubStore((s) => s.dismissProgress);

  if (!analysisProgress) return null;

  const { isMinimized, steps, totalInputTokens, totalOutputTokens, completedAt } = analysisProgress;

  const doneCount = steps.filter((s) => s.status === 'done').length;
  const errorCount = steps.filter((s) => s.status === 'error').length;
  const total = steps.length;
  const isComplete = completedAt !== undefined;
  const runningStep = steps.find((s) => s.status === 'running');
  const pct = total > 0 ? Math.round(((doneCount + errorCount) / total) * 100) : 0;

  if (isMinimized) {
    return (
      <div className="fixed bottom-4 right-4 z-50">
        <button
          onClick={maximizeProgress}
          className="flex items-center gap-2 rounded-full border border-border bg-card px-4 py-2 shadow-lg hover:bg-accent transition-colors text-sm"
        >
          {isComplete ? (
            errorCount === total ? (
              <XCircle className="size-4 text-destructive shrink-0" />
            ) : (
              <CheckCircle2 className="size-4 text-green-500 shrink-0" />
            )
          ) : (
            <Loader2 className="size-4 animate-spin text-primary shrink-0" />
          )}
          <span className="font-medium">
            {isComplete
              ? errorCount > 0
                ? `Analysis done (${errorCount} error${errorCount > 1 ? 's' : ''})`
                : 'Analysis complete'
              : `Analyzing… ${doneCount}/${total}`}
          </span>
          {(totalInputTokens > 0 || totalOutputTokens > 0) && (
            <span className="text-muted-foreground text-xs">
              ↑{fmt(totalInputTokens)} ↓{fmt(totalOutputTokens)}
            </span>
          )}
          <ChevronUp className="size-3.5 opacity-60" />
        </button>
      </div>
    );
  }

  return (
    <div className="fixed bottom-4 right-4 z-50 w-96 rounded-lg border border-border bg-card shadow-xl flex flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-2 border-b border-border px-4 py-3 bg-muted/40">
        <Cpu className="size-4 text-primary shrink-0" />
        <span className="flex-1 font-medium text-sm truncate">
          {isComplete ? 'AI Analysis complete' : 'AI Analysis in progress…'}
        </span>
        <button
          onClick={minimizeProgress}
          className="p-1 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
          title="Minimize"
        >
          <ChevronDown className="size-4" />
        </button>
        {isComplete && (
          <button
            onClick={dismissProgress}
            className="p-1 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
            title="Dismiss"
          >
            <X className="size-4" />
          </button>
        )}
      </div>

      {/* Progress bar */}
      {!isComplete && (
        <div className="h-1 bg-muted">
          <div
            className="h-full bg-primary transition-all duration-300"
            style={{ width: `${pct}%` }}
          />
        </div>
      )}

      {/* Step list */}
      <div className="flex-1 overflow-y-auto max-h-64 p-3 space-y-1">
        {steps.map((step) => (
          <div
            key={step.id}
            className={cn(
              'flex items-center gap-2 rounded px-2 py-1.5 text-sm',
              step.status === 'running' && 'bg-primary/5',
            )}
          >
            <span className="shrink-0">
              {step.status === 'pending' && (
                <span className="inline-block size-4 rounded-full border-2 border-muted-foreground/30" />
              )}
              {step.status === 'running' && (
                <Loader2 className="size-4 animate-spin text-primary" />
              )}
              {step.status === 'done' && (
                <CheckCircle2 className="size-4 text-green-500" />
              )}
              {step.status === 'error' && (
                <XCircle className="size-4 text-destructive" />
              )}
            </span>
            <span
              className={cn(
                'flex-1 truncate',
                step.status === 'pending' && 'text-muted-foreground',
                step.status === 'running' && 'font-medium',
                step.status === 'error' && 'text-destructive',
              )}
            >
              {step.label}
            </span>
            {step.status === 'done' && step.inputTokens !== undefined && (
              <span className="text-xs text-muted-foreground shrink-0">
                ↑{fmt(step.inputTokens)} ↓{fmt(step.outputTokens ?? 0)}
              </span>
            )}
          </div>
        ))}
        {steps.length === 0 && (
          <p className="text-sm text-muted-foreground text-center py-2">Preparing…</p>
        )}
      </div>

      {/* Token summary footer */}
      <div className="border-t border-border px-4 py-2 flex items-center justify-between text-xs text-muted-foreground bg-muted/20">
        <div className="flex gap-3">
          <span>In: <span className="font-mono font-medium text-foreground">{fmt(totalInputTokens)}</span></span>
          <span>Out: <span className="font-mono font-medium text-foreground">{fmt(totalOutputTokens)}</span></span>
          <span>Total: <span className="font-mono font-medium text-foreground">{fmt(totalInputTokens + totalOutputTokens)}</span></span>
        </div>
        <span>
          {isComplete
            ? (errorCount > 0 ? `${errorCount} error${errorCount > 1 ? 's' : ''}` : `${doneCount}/${total} done`)
            : runningStep
              ? `${doneCount}/${total} done`
              : `${doneCount}/${total} done`}
        </span>
      </div>
    </div>
  );
}
