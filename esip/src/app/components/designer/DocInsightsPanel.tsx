import { useState, useCallback, useEffect } from 'react';
import type { Editor } from '@tiptap/react';
import {
  X, Activity, User, GitBranch, FileText,
  Loader2, RefreshCw, AlertTriangle, CheckCircle,
  Info, ChevronDown, AlertCircle, Zap, Shield,
} from 'lucide-react';
import { cn } from '../ui/utils';
import { Button } from '../ui/button';
import { generateDesignerContent } from '../../lib/ai/designerAI';
import { db } from '../../lib/db';
import type { DesignerTemplate } from './types';
import type { DesignerDocument } from '../../lib/db/types';

// ── Types ─────────────────────────────────────────────────────────────────────

type InsightsTab = 'health' | 'tldr' | 'reader' | 'impact';

interface HealthIssue {
  severity: 'high' | 'medium' | 'low';
  section: string;
  issue: string;
  suggestion: string;
}
interface HealthResult {
  score: number;
  summary: string;
  issues: HealthIssue[];
}

interface TldrResult {
  keyDecisions: string[];
  openRisks: string[];
  nextActions: string[];
  stakeholders: string[];
}

interface AffectedDoc {
  docTitle: string;
  category: string;
  reason: string;
  severity: 'high' | 'medium' | 'low';
}
interface ImpactResult {
  affectedDocs: AffectedDoc[];
  summary: string;
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function extractJson<T>(raw: string): T {
  // Strip markdown fences if AI wraps output
  const stripped = raw.replace(/^```(?:json)?\s*/m, '').replace(/\s*```\s*$/m, '').trim();
  return JSON.parse(stripped) as T;
}

const ROLES = [
  'Frontend Developer',
  'Backend Developer',
  'Product Manager',
  'Security Engineer',
  'DevOps / Platform Engineer',
  'Business Stakeholder',
  'QA Engineer',
  'Data Engineer',
  'Solution Architect',
];

const SEV_CONFIG = {
  high:   { color: 'text-red-600 dark:text-red-400',    bg: 'bg-red-50 dark:bg-red-900/20',    border: 'border-red-200 dark:border-red-800',    icon: <AlertTriangle className="size-3" />, label: 'High' },
  medium: { color: 'text-amber-600 dark:text-amber-400', bg: 'bg-amber-50 dark:bg-amber-900/20', border: 'border-amber-200 dark:border-amber-800', icon: <AlertCircle className="size-3" />,  label: 'Medium' },
  low:    { color: 'text-blue-600 dark:text-blue-400',   bg: 'bg-blue-50 dark:bg-blue-900/20',   border: 'border-blue-200 dark:border-blue-800',   icon: <Info className="size-3" />,         label: 'Low' },
};

function ScoreRing({ score }: { score: number }) {
  const r = 28;
  const circ = 2 * Math.PI * r;
  const dash = (score / 100) * circ;
  const color = score >= 80 ? '#22c55e' : score >= 60 ? '#f59e0b' : '#ef4444';

  return (
    <div className="flex flex-col items-center gap-1">
      <svg width="72" height="72" viewBox="0 0 72 72" className="-rotate-90">
        <circle cx="36" cy="36" r={r} fill="none" stroke="currentColor" strokeWidth="6" className="text-border" />
        <circle
          cx="36" cy="36" r={r} fill="none"
          stroke={color} strokeWidth="6"
          strokeDasharray={`${dash} ${circ}`}
          strokeLinecap="round"
          style={{ transition: 'stroke-dasharray 0.8s ease' }}
        />
      </svg>
      <span className="text-2xl font-bold -mt-[52px] mb-[20px] relative z-10">{score}</span>
    </div>
  );
}

// ── Tab: Health Score ─────────────────────────────────────────────────────────

function HealthTab({ getDocText, template }: { getDocText: () => string; template: DesignerTemplate }) {
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<HealthResult | null>(null);
  const [error, setError] = useState('');

  const analyze = async () => {
    setLoading(true);
    setError('');
    setResult(null);
    try {
      const docText = getDocText();
      const prompt = `You are a document quality reviewer for ${template.name} documents.

DOCUMENT TEXT:
${docText.slice(0, 8000)}

TASK: Assess the health and completeness of this document.

Return ONLY a JSON object (no markdown fences, no explanation):
{
  "score": <integer 0-100>,
  "summary": "<1-2 sentence overall assessment>",
  "issues": [
    {
      "severity": "high" | "medium" | "low",
      "section": "<section or widget name>",
      "issue": "<what is missing or wrong>",
      "suggestion": "<how to fix it>"
    }
  ]
}

Scoring guide:
- 90-100: Complete, well-structured, all sections populated
- 70-89: Mostly complete, minor gaps
- 50-69: Several sections incomplete or vague
- 30-49: Major gaps, placeholder text, stale content
- 0-29: Skeleton / barely started

Check for: placeholder text (TBD, TODO, enter X here), missing requirement IDs, ADRs without decisions, empty diagrams, missing stakeholders, vague acceptance criteria. Generate 3-8 issues.`;

      const raw = await generateDesignerContent(prompt);
      setResult(extractJson<HealthResult>(raw));
    } catch (err) {
      setError((err as Error).message || 'Analysis failed');
    } finally {
      setLoading(false);
    }
  };

  const scoreColor = result
    ? result.score >= 80 ? 'text-green-600 dark:text-green-400'
      : result.score >= 60 ? 'text-amber-600 dark:text-amber-400'
      : 'text-red-600 dark:text-red-400'
    : '';

  return (
    <div className="space-y-4">
      <div className="rounded-lg bg-muted/40 p-3 text-xs text-muted-foreground leading-relaxed">
        AI scans your document for incomplete sections, placeholder text, missing requirements, and stale ADRs — then gives it a quality score.
      </div>

      {!result && !loading && (
        <Button onClick={analyze} className="w-full gap-2" size="sm">
          <Activity className="size-3.5" />
          Analyze document health
        </Button>
      )}

      {loading && (
        <div className="flex flex-col items-center gap-3 py-8">
          <Loader2 className="size-6 animate-spin text-primary" />
          <p className="text-xs text-muted-foreground">Scanning document…</p>
        </div>
      )}

      {error && (
        <div className="rounded-lg bg-destructive/10 px-3 py-2 text-xs text-destructive">{error}</div>
      )}

      {result && (
        <div className="space-y-4">
          {/* Score */}
          <div className="flex items-center gap-4 rounded-xl border border-border bg-card p-4">
            <ScoreRing score={result.score} />
            <div className="flex-1 min-w-0">
              <div className="flex items-baseline gap-1.5">
                <span className={cn('text-3xl font-bold', scoreColor)}>{result.score}</span>
                <span className="text-sm text-muted-foreground">/100</span>
              </div>
              <p className="text-xs text-muted-foreground mt-1 leading-relaxed">{result.summary}</p>
            </div>
          </div>

          {/* Issues */}
          {result.issues.length > 0 && (
            <div className="space-y-2">
              <p className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide">
                {result.issues.length} issue{result.issues.length !== 1 ? 's' : ''} found
              </p>
              {result.issues.map((issue, i) => {
                const sev = SEV_CONFIG[issue.severity];
                return (
                  <div key={i} className={cn('rounded-lg border p-3 space-y-1', sev.bg, sev.border)}>
                    <div className="flex items-center gap-1.5">
                      <span className={sev.color}>{sev.icon}</span>
                      <span className={cn('text-[10px] font-bold uppercase tracking-wide', sev.color)}>{sev.label}</span>
                      <span className="text-[10px] text-muted-foreground">·</span>
                      <span className="text-[10px] text-muted-foreground font-medium truncate">{issue.section}</span>
                    </div>
                    <p className="text-xs font-medium text-foreground">{issue.issue}</p>
                    <p className="text-[11px] text-muted-foreground leading-relaxed">→ {issue.suggestion}</p>
                  </div>
                );
              })}
            </div>
          )}

          {result.issues.length === 0 && (
            <div className="flex items-center gap-2 rounded-lg bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 p-3">
              <CheckCircle className="size-4 text-green-600 dark:text-green-400 shrink-0" />
              <p className="text-xs text-green-700 dark:text-green-400">No significant issues found.</p>
            </div>
          )}

          <button
            onClick={analyze}
            className="flex items-center gap-1.5 text-[11px] text-muted-foreground hover:text-foreground transition-colors"
          >
            <RefreshCw className="size-3" /> Re-analyze
          </button>
        </div>
      )}
    </div>
  );
}

// ── Tab: Smart TL;DR ──────────────────────────────────────────────────────────

function TldrTab({ getDocText, template }: { getDocText: () => string; template: DesignerTemplate }) {
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<TldrResult | null>(null);
  const [error, setError] = useState('');

  const extract = async () => {
    setLoading(true);
    setError('');
    setResult(null);
    try {
      const docText = getDocText();
      const prompt = `You are a technical writer extracting key information from a ${template.name}.

DOCUMENT:
${docText.slice(0, 8000)}

Extract the key information and return ONLY a JSON object (no fences):
{
  "keyDecisions": ["<decision 1>", ...],
  "openRisks": ["<risk 1>", ...],
  "nextActions": ["<action 1>", ...],
  "stakeholders": ["<role: name/team>", ...]
}

Rules:
- keyDecisions: architectural or business decisions already made (ADRs, approved requirements). Max 6.
- openRisks: identified risks, blockers, or concerns that are unresolved. Max 6.
- nextActions: concrete next steps, TODOs, or items needing attention. Max 6.
- stakeholders: people, roles, or teams mentioned. Max 8.
- Each item should be a concise, self-contained statement (10-20 words).
- If a category has nothing relevant, return an empty array.`;

      const raw = await generateDesignerContent(prompt);
      setResult(extractJson<TldrResult>(raw));
    } catch (err) {
      setError((err as Error).message || 'Extraction failed');
    } finally {
      setLoading(false);
    }
  };

  const sections: { key: keyof TldrResult; label: string; icon: React.ReactNode; color: string }[] = [
    { key: 'keyDecisions', label: 'Key Decisions', icon: <CheckCircle className="size-3.5" />, color: 'text-green-600 dark:text-green-400' },
    { key: 'openRisks',    label: 'Open Risks',    icon: <AlertTriangle className="size-3.5" />, color: 'text-red-600 dark:text-red-400' },
    { key: 'nextActions',  label: 'Next Actions',  icon: <Zap className="size-3.5" />,           color: 'text-amber-600 dark:text-amber-400' },
    { key: 'stakeholders', label: 'Stakeholders',  icon: <User className="size-3.5" />,           color: 'text-blue-600 dark:text-blue-400' },
  ];

  return (
    <div className="space-y-4">
      <div className="rounded-lg bg-muted/40 p-3 text-xs text-muted-foreground leading-relaxed">
        Auto-extracts key decisions, open risks, next actions, and stakeholders from your document into a quick-scan summary.
      </div>

      {!result && !loading && (
        <Button onClick={extract} className="w-full gap-2" size="sm">
          <FileText className="size-3.5" />
          Generate TL;DR
        </Button>
      )}

      {loading && (
        <div className="flex flex-col items-center gap-3 py-8">
          <Loader2 className="size-6 animate-spin text-primary" />
          <p className="text-xs text-muted-foreground">Extracting summary…</p>
        </div>
      )}

      {error && (
        <div className="rounded-lg bg-destructive/10 px-3 py-2 text-xs text-destructive">{error}</div>
      )}

      {result && (
        <div className="space-y-3">
          {sections.map(({ key, label, icon, color }) => {
            const items = result[key];
            if (items.length === 0) return null;
            return (
              <div key={key} className="rounded-xl border border-border bg-card overflow-hidden">
                <div className="flex items-center gap-2 px-3 py-2 bg-muted/30 border-b border-border">
                  <span className={color}>{icon}</span>
                  <span className="text-xs font-semibold">{label}</span>
                  <span className="ml-auto text-[10px] text-muted-foreground">{items.length}</span>
                </div>
                <ul className="divide-y divide-border">
                  {items.map((item, i) => (
                    <li key={i} className="px-3 py-2 text-xs text-foreground leading-relaxed">
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
            );
          })}
          <button
            onClick={extract}
            className="flex items-center gap-1.5 text-[11px] text-muted-foreground hover:text-foreground transition-colors"
          >
            <RefreshCw className="size-3" /> Refresh
          </button>
        </div>
      )}
    </div>
  );
}

// ── Tab: Role-Based Reader Mode ───────────────────────────────────────────────

function ReaderTab({ getDocText, template }: { getDocText: () => string; template: DesignerTemplate }) {
  const [selectedRole, setSelectedRole] = useState(ROLES[0]);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState('');
  const [error, setError] = useState('');
  const [roleOpen, setRoleOpen] = useState(false);

  const filter = async () => {
    setLoading(true);
    setError('');
    setResult('');
    try {
      const docText = getDocText();
      const prompt = `You are reading a ${template.name} as a ${selectedRole}.

FULL DOCUMENT:
${docText.slice(0, 8000)}

TASK: Extract and summarize ONLY the parts of this document that are directly relevant to a ${selectedRole}.
Ignore sections that have no bearing on their role.

Format your response as clean markdown with headers for each relevant section.
Start with a one-line "Why this matters to a ${selectedRole}" sentence.
Then list the relevant sections as: ## Section Name followed by a concise summary of what matters.
Keep each section summary to 2-4 sentences — just the actionable takeaways.
If nothing in the document is relevant to this role, say so clearly.`;

      setResult(await generateDesignerContent(prompt));
    } catch (err) {
      setError((err as Error).message || 'Filtering failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      <div className="rounded-lg bg-muted/40 p-3 text-xs text-muted-foreground leading-relaxed">
        Pick your role and AI filters the document to show only what's relevant to you — cutting through noise instantly.
      </div>

      {/* Role picker */}
      <div className="space-y-1.5">
        <p className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide">I am a…</p>
        <div className="relative">
          <button
            onClick={() => setRoleOpen(o => !o)}
            className="w-full flex items-center justify-between rounded-lg border border-border bg-background px-3 py-2 text-sm text-foreground hover:border-primary/50 transition-colors"
          >
            <span>{selectedRole}</span>
            <ChevronDown className={cn('size-4 text-muted-foreground transition-transform', roleOpen && 'rotate-180')} />
          </button>
          {roleOpen && (
            <div className="absolute z-20 left-0 right-0 top-full mt-1 rounded-lg border border-border bg-card shadow-lg overflow-hidden">
              {ROLES.map(role => (
                <button
                  key={role}
                  onClick={() => { setSelectedRole(role); setRoleOpen(false); setResult(''); }}
                  className={cn(
                    'w-full px-3 py-2 text-xs text-left hover:bg-muted/60 transition-colors',
                    role === selectedRole && 'font-medium text-primary bg-primary/5',
                  )}
                >
                  {role}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      <Button onClick={filter} disabled={loading} className="w-full gap-2" size="sm">
        {loading ? <Loader2 className="size-3.5 animate-spin" /> : <User className="size-3.5" />}
        {loading ? 'Filtering…' : `Filter for ${selectedRole.split(' ')[0]}`}
      </Button>

      {error && (
        <div className="rounded-lg bg-destructive/10 px-3 py-2 text-xs text-destructive">{error}</div>
      )}

      {result && (
        <div className="rounded-xl border border-border bg-card overflow-hidden">
          <div className="flex items-center justify-between px-3 py-2 bg-muted/30 border-b border-border">
            <span className="text-[11px] font-medium text-muted-foreground uppercase tracking-wide">
              Filtered for {selectedRole}
            </span>
            <button
              onClick={() => setResult('')}
              className="text-[10px] text-muted-foreground hover:text-foreground transition-colors"
            >
              clear
            </button>
          </div>
          <div
            className="p-3 text-xs text-foreground leading-relaxed prose prose-sm dark:prose-invert max-w-none
              prose-headings:text-sm prose-headings:font-semibold prose-headings:mt-3 prose-headings:mb-1
              prose-p:my-1 prose-ul:my-1 prose-li:my-0"
            dangerouslySetInnerHTML={{ __html: result.replace(/\n/g, '<br>') }}
          />
        </div>
      )}
    </div>
  );
}

// ── Tab: Cross-document Impact Analysis ──────────────────────────────────────

function ImpactTab({
  getDocText,
  template,
  projectId,
  documentId,
}: {
  getDocText: () => string;
  template: DesignerTemplate;
  projectId: string;
  documentId?: string;
}) {
  const [otherDocs, setOtherDocs] = useState<DesignerDocument[]>([]);
  const [docsLoading, setDocsLoading] = useState(true);
  const [analyzeLoading, setAnalyzeLoading] = useState(false);
  const [result, setResult] = useState<ImpactResult | null>(null);
  const [error, setError] = useState('');

  useEffect(() => {
    (async () => {
      try {
        const all = await db.designerDocuments.getByProject(projectId);
        setOtherDocs(all.filter(d => d.id !== documentId));
      } catch {
        // silently ignore — impact analysis still works with current doc alone
      } finally {
        setDocsLoading(false);
      }
    })();
  }, [projectId, documentId]);

  const analyze = async () => {
    setAnalyzeLoading(true);
    setError('');
    setResult(null);
    try {
      const docText = getDocText();

      // Build a summary of other docs (title + first 400 chars of text)
      const otherDocSummaries = otherDocs.map(d => {
        let text = '';
        try { text = JSON.parse(d.content).content?.map((n: { text?: string }) => n.text || '').join(' ').slice(0, 400) || ''; } catch { /* ignore */ }
        return `${d.category.toUpperCase()} — "${d.title}": ${text}`;
      }).join('\n\n');

      const prompt = `You are an impact analyst reviewing a ${template.name} document.

CURRENT DOCUMENT (${template.name}):
${docText.slice(0, 5000)}

OTHER DOCUMENTS IN THIS PROJECT:
${otherDocSummaries || '(no other documents found)'}

TASK: Identify which other documents would need to be updated if requirements or design decisions in the current document change.

Return ONLY a JSON object (no fences):
{
  "summary": "<1-2 sentence overall impact assessment>",
  "affectedDocs": [
    {
      "docTitle": "<document title>",
      "category": "<brd|hld|lld>",
      "reason": "<why this document would be affected — be specific about which section>",
      "severity": "high" | "medium" | "low"
    }
  ]
}

If there are no other documents, explain in the summary what types of downstream documents would typically be affected.
Severity: high = must be updated immediately, medium = likely needs review, low = may be affected.`;

      const raw = await generateDesignerContent(prompt);
      setResult(extractJson<ImpactResult>(raw));
    } catch (err) {
      setError((err as Error).message || 'Analysis failed');
    } finally {
      setAnalyzeLoading(false);
    }
  };

  if (docsLoading) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader2 className="size-5 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="rounded-lg bg-muted/40 p-3 text-xs text-muted-foreground leading-relaxed">
        AI identifies which other documents in this project would need updating if requirements in this document change.
      </div>

      {otherDocs.length > 0 ? (
        <div className="rounded-lg border border-border bg-card p-3 space-y-1">
          <p className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide mb-2">
            {otherDocs.length} other doc{otherDocs.length !== 1 ? 's' : ''} in project
          </p>
          {otherDocs.map(d => (
            <div key={d.id} className="flex items-center gap-2">
              <span className="text-[10px] font-bold text-muted-foreground uppercase w-7">{d.category}</span>
              <span className="text-xs text-foreground truncate">{d.title}</span>
            </div>
          ))}
        </div>
      ) : (
        <div className="flex items-start gap-2 rounded-lg bg-muted/40 p-3">
          <Info className="size-3.5 text-muted-foreground shrink-0 mt-0.5" />
          <p className="text-xs text-muted-foreground leading-relaxed">
            No other documents in this project yet. The analysis will still show which document types would typically be impacted.
          </p>
        </div>
      )}

      {!result && !analyzeLoading && (
        <Button onClick={analyze} className="w-full gap-2" size="sm">
          <GitBranch className="size-3.5" />
          Analyze impact
        </Button>
      )}

      {analyzeLoading && (
        <div className="flex flex-col items-center gap-3 py-8">
          <Loader2 className="size-6 animate-spin text-primary" />
          <p className="text-xs text-muted-foreground">Analysing cross-document impact…</p>
        </div>
      )}

      {error && (
        <div className="rounded-lg bg-destructive/10 px-3 py-2 text-xs text-destructive">{error}</div>
      )}

      {result && (
        <div className="space-y-3">
          <div className="rounded-lg bg-muted/40 border border-border px-3 py-2">
            <p className="text-xs text-foreground leading-relaxed">{result.summary}</p>
          </div>

          {result.affectedDocs.length > 0 ? (
            <div className="space-y-2">
              <p className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide">
                {result.affectedDocs.length} document{result.affectedDocs.length !== 1 ? 's' : ''} affected
              </p>
              {result.affectedDocs.map((doc, i) => {
                const sev = SEV_CONFIG[doc.severity];
                return (
                  <div key={i} className={cn('rounded-lg border p-3 space-y-1', sev.bg, sev.border)}>
                    <div className="flex items-center gap-1.5">
                      <span className={sev.color}>{sev.icon}</span>
                      <span className={cn('text-[10px] font-bold uppercase', sev.color)}>{sev.label}</span>
                      <span className="text-[10px] text-muted-foreground uppercase font-medium ml-1">{doc.category}</span>
                    </div>
                    <p className="text-xs font-semibold text-foreground">{doc.docTitle}</p>
                    <p className="text-[11px] text-muted-foreground leading-relaxed">{doc.reason}</p>
                  </div>
                );
              })}
            </div>
          ) : (
            <div className="flex items-center gap-2 rounded-lg bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 p-3">
              <Shield className="size-4 text-green-600 dark:text-green-400 shrink-0" />
              <p className="text-xs text-green-700 dark:text-green-400">No downstream documents need updating.</p>
            </div>
          )}

          <button
            onClick={analyze}
            className="flex items-center gap-1.5 text-[11px] text-muted-foreground hover:text-foreground transition-colors"
          >
            <RefreshCw className="size-3" /> Re-analyze
          </button>
        </div>
      )}
    </div>
  );
}

// ── Main Panel ────────────────────────────────────────────────────────────────

const TABS: { id: InsightsTab; label: string; icon: React.ReactNode }[] = [
  { id: 'health',  label: 'Health',  icon: <Activity className="size-3.5" /> },
  { id: 'tldr',   label: 'TL;DR',   icon: <FileText className="size-3.5" /> },
  { id: 'reader', label: 'Reader',  icon: <User className="size-3.5" /> },
  { id: 'impact', label: 'Impact',  icon: <GitBranch className="size-3.5" /> },
];

interface DocInsightsPanelProps {
  editor: Editor;
  template: DesignerTemplate;
  projectId: string;
  documentId?: string;
  onClose: () => void;
}

export function DocInsightsPanel({ editor, template, projectId, documentId, onClose }: DocInsightsPanelProps) {
  const [activeTab, setActiveTab] = useState<InsightsTab>('health');
  const getDocText = useCallback(() => editor.getText(), [editor]);

  return (
    <div className="flex flex-col h-full border-l border-border bg-card overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between gap-2 px-4 py-3 border-b border-border bg-violet-500/5 shrink-0">
        <div className="flex items-center gap-2 min-w-0">
          <div className="flex size-6 items-center justify-center rounded-full bg-violet-500/10 shrink-0">
            <Activity className="size-3.5 text-violet-500" />
          </div>
          <span className="text-sm font-semibold truncate">Doc Insights</span>
          <span className="rounded-full bg-violet-500/10 px-2 py-0.5 text-[10px] font-medium text-violet-600 dark:text-violet-400 shrink-0">AI</span>
        </div>
        <button
          onClick={onClose}
          className="rounded p-1 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors shrink-0"
        >
          <X className="size-4" />
        </button>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-border shrink-0 bg-muted/20">
        {TABS.map(tab => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={cn(
              'flex-1 flex flex-col items-center gap-0.5 py-2.5 text-[10px] font-medium transition-colors border-b-2',
              activeTab === tab.id
                ? 'text-violet-600 dark:text-violet-400 border-violet-500'
                : 'text-muted-foreground border-transparent hover:text-foreground',
            )}
          >
            {tab.icon}
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      <div className="flex-1 overflow-y-auto p-4">
        {activeTab === 'health'  && <HealthTab  getDocText={getDocText} template={template} />}
        {activeTab === 'tldr'    && <TldrTab    getDocText={getDocText} template={template} />}
        {activeTab === 'reader'  && <ReaderTab  getDocText={getDocText} template={template} />}
        {activeTab === 'impact'  && (
          <ImpactTab
            getDocText={getDocText}
            template={template}
            projectId={projectId}
            documentId={documentId}
          />
        )}
      </div>
    </div>
  );
}
