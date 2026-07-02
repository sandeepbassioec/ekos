import React, { useState, useEffect } from 'react';
import {
  Sparkles, Loader2, X, CheckCircle2, ChevronDown, ChevronUp, Trash2,
  Monitor, LayoutGrid, PanelLeft, Layers, Zap, Timer, Database, ArrowRightLeft,
  Network, FlaskConical, TestTube, Microscope, Container, ShieldCheck, Gauge,
  Scale, BookOpen,
} from 'lucide-react';
import { Button } from '../ui/button';
import { toast } from 'sonner';
import { useStoriesStore } from '../../store/storiesStore';
import { useAuthStore } from '../../store/authStore';
import type { Story, StoryType } from '../../lib/db/types';
import type { Requirement } from '../../types/requirements';

const AI_PROXY_URL = import.meta.env.VITE_AI_PROXY_URL || 'http://localhost:8787';

// ── Story type catalogue ──────────────────────────────────────────────────────
type TypeMeta = {
  label: string;
  group: string;
  groupColor: string;    // group-level badge colour
  badgeColor: string;    // per-type badge colour
  icon: React.ReactNode;
};

const TYPE_META: Record<StoryType, TypeMeta> = {
  // UI / Frontend
  ui_new_screen:        { label: 'New Screen',       group: 'UI / Frontend',      groupColor: 'text-sky-700 bg-sky-100 dark:text-sky-300 dark:bg-sky-900/40',        badgeColor: 'text-sky-700 bg-sky-100 dark:text-sky-300 dark:bg-sky-900/40',        icon: <Monitor className="size-3" /> },
  ui_component:         { label: 'Component',        group: 'UI / Frontend',      groupColor: 'text-sky-700 bg-sky-100 dark:text-sky-300 dark:bg-sky-900/40',        badgeColor: 'text-cyan-700 bg-cyan-100 dark:text-cyan-300 dark:bg-cyan-900/40',      icon: <LayoutGrid className="size-3" /> },
  ui_update:            { label: 'Screen Update',    group: 'UI / Frontend',      groupColor: 'text-sky-700 bg-sky-100 dark:text-sky-300 dark:bg-sky-900/40',        badgeColor: 'text-blue-700 bg-blue-100 dark:text-blue-300 dark:bg-blue-900/40',     icon: <Layers className="size-3" /> },
  ui_modal:             { label: 'Modal / Popup',    group: 'UI / Frontend',      groupColor: 'text-sky-700 bg-sky-100 dark:text-sky-300 dark:bg-sky-900/40',        badgeColor: 'text-indigo-700 bg-indigo-100 dark:text-indigo-300 dark:bg-indigo-900/40', icon: <PanelLeft className="size-3" /> },
  // API / Backend
  api_endpoint:         { label: 'API Endpoint',     group: 'API / Backend',      groupColor: 'text-violet-700 bg-violet-100 dark:text-violet-300 dark:bg-violet-900/40', badgeColor: 'text-violet-700 bg-violet-100 dark:text-violet-300 dark:bg-violet-900/40', icon: <Zap className="size-3" /> },
  backend_service:      { label: 'Backend Service',  group: 'API / Backend',      groupColor: 'text-violet-700 bg-violet-100 dark:text-violet-300 dark:bg-violet-900/40', badgeColor: 'text-purple-700 bg-purple-100 dark:text-purple-300 dark:bg-purple-900/40', icon: <Container className="size-3" /> },
  background_job:       { label: 'Background Job',   group: 'API / Backend',      groupColor: 'text-violet-700 bg-violet-100 dark:text-violet-300 dark:bg-violet-900/40', badgeColor: 'text-fuchsia-700 bg-fuchsia-100 dark:text-fuchsia-300 dark:bg-fuchsia-900/40', icon: <Timer className="size-3" /> },
  // Data
  database_schema:      { label: 'DB Schema',        group: 'Data',               groupColor: 'text-amber-700 bg-amber-100 dark:text-amber-300 dark:bg-amber-900/40',  badgeColor: 'text-amber-700 bg-amber-100 dark:text-amber-300 dark:bg-amber-900/40',  icon: <Database className="size-3" /> },
  data_migration:       { label: 'Data Migration',   group: 'Data',               groupColor: 'text-amber-700 bg-amber-100 dark:text-amber-300 dark:bg-amber-900/40',  badgeColor: 'text-orange-700 bg-orange-100 dark:text-orange-300 dark:bg-orange-900/40', icon: <ArrowRightLeft className="size-3" /> },
  // Integration
  integration_external: { label: 'External Integration', group: 'Integration',   groupColor: 'text-teal-700 bg-teal-100 dark:text-teal-300 dark:bg-teal-900/40',     badgeColor: 'text-teal-700 bg-teal-100 dark:text-teal-300 dark:bg-teal-900/40',     icon: <Network className="size-3" /> },
  integration_internal: { label: 'Internal Integration', group: 'Integration',   groupColor: 'text-teal-700 bg-teal-100 dark:text-teal-300 dark:bg-teal-900/40',     badgeColor: 'text-emerald-700 bg-emerald-100 dark:text-emerald-300 dark:bg-emerald-900/40', icon: <ArrowRightLeft className="size-3" /> },
  // Testing
  unit_test:            { label: 'Unit Tests',        group: 'Testing',            groupColor: 'text-green-700 bg-green-100 dark:text-green-300 dark:bg-green-900/40', badgeColor: 'text-green-700 bg-green-100 dark:text-green-300 dark:bg-green-900/40', icon: <FlaskConical className="size-3" /> },
  integration_test:     { label: 'Integration Tests', group: 'Testing',            groupColor: 'text-green-700 bg-green-100 dark:text-green-300 dark:bg-green-900/40', badgeColor: 'text-lime-700 bg-lime-100 dark:text-lime-300 dark:bg-lime-900/40',     icon: <TestTube className="size-3" /> },
  e2e_test:             { label: 'E2E / Automation',  group: 'Testing',            groupColor: 'text-green-700 bg-green-100 dark:text-green-300 dark:bg-green-900/40', badgeColor: 'text-emerald-700 bg-emerald-100 dark:text-emerald-300 dark:bg-emerald-900/40', icon: <Microscope className="size-3" /> },
  // DevOps
  devops:               { label: 'DevOps / Infra',   group: 'DevOps',             groupColor: 'text-slate-700 bg-slate-100 dark:text-slate-300 dark:bg-slate-800',   badgeColor: 'text-slate-700 bg-slate-100 dark:text-slate-300 dark:bg-slate-800',   icon: <Container className="size-3" /> },
  // Cross-cutting
  security:             { label: 'Security',          group: 'Cross-cutting',      groupColor: 'text-red-700 bg-red-100 dark:text-red-300 dark:bg-red-900/40',        badgeColor: 'text-red-700 bg-red-100 dark:text-red-300 dark:bg-red-900/40',        icon: <ShieldCheck className="size-3" /> },
  performance:          { label: 'Performance',       group: 'Cross-cutting',      groupColor: 'text-red-700 bg-red-100 dark:text-red-300 dark:bg-red-900/40',        badgeColor: 'text-rose-700 bg-rose-100 dark:text-rose-300 dark:bg-rose-900/40',    icon: <Gauge className="size-3" /> },
  compliance:           { label: 'Compliance',        group: 'Cross-cutting',      groupColor: 'text-red-700 bg-red-100 dark:text-red-300 dark:bg-red-900/40',        badgeColor: 'text-pink-700 bg-pink-100 dark:text-pink-300 dark:bg-pink-900/40',   icon: <Scale className="size-3" /> },
  documentation:        { label: 'Documentation',     group: 'Cross-cutting',      groupColor: 'text-red-700 bg-red-100 dark:text-red-300 dark:bg-red-900/40',        badgeColor: 'text-gray-700 bg-gray-100 dark:text-gray-300 dark:bg-gray-800',      icon: <BookOpen className="size-3" /> },
};

// Display order — grouped so the board reads top-to-bottom like a delivery pipeline
const GROUP_ORDER = ['UI / Frontend', 'API / Backend', 'Data', 'Integration', 'Testing', 'DevOps', 'Cross-cutting'];
const TYPE_ORDER = Object.keys(TYPE_META) as StoryType[];

// ── AI prompt ─────────────────────────────────────────────────────────────────
interface AIStory {
  title: string;
  type: StoryType;
  description: string;
  acceptanceCriteria: string[];
  storyPoints: number;
}

function buildPrompt(req: Requirement): string {
  const ac = Array.isArray((req as any).acceptanceCriteria)
    ? (req as any).acceptanceCriteria.map((a: any) => `  - ${a.text ?? a}`).join('\n')
    : 'None';

  return `You are a senior SAFe Agile coach and solution architect. Your job is to decompose a single product requirement into ALL the engineering user stories needed to deliver it end-to-end, across every layer of a modern software system.

════════════════════════════════════════════════════════
REQUIREMENT TO DECOMPOSE
════════════════════════════════════════════════════════
Title:       ${req.title}
Type:        ${(req as any).type ?? req.category}
Priority:    ${req.priority}
Description: ${req.description ?? 'Not provided'}
User Story:  ${(req as any).userStory ?? 'Not provided'}
Acceptance Criteria:
${ac}

════════════════════════════════════════════════════════
YOUR TASK
════════════════════════════════════════════════════════
Think through EVERY layer of the system needed to implement this requirement.
For each layer that is relevant, generate one or more stories.

Go through this checklist systematically — skip a layer ONLY if it is provably not needed:

UI / Frontend
  ☐ ui_new_screen      — Does this require a brand-new page or route?
  ☐ ui_component       — Does it need a new reusable component, widget, or design-system element?
  ☐ ui_update          — Does it modify an existing screen (new field, tab, button, section)?
  ☐ ui_modal           — Does it open a modal, drawer, dialog, or tooltip?

API / Backend
  ☐ api_endpoint       — Does it need a new REST or GraphQL endpoint (GET/POST/PUT/DELETE)?
                         Consider: list endpoint, detail endpoint, create, update, delete, search, export — each may be a separate story.
  ☐ backend_service    — Does it introduce new business logic, validation, calculation, or orchestration in the service layer?
  ☐ background_job     — Does it need a scheduled job, event-driven worker, queue consumer, or async process?

Data
  ☐ database_schema    — Does it require a new table, new columns, indexes, foreign keys, or enum changes?
  ☐ data_migration     — Does it need a one-time migration or seed script to transform existing data?

Integration
  ☐ integration_external — Does it call a third-party API (payment gateway, SMS, email, maps, analytics, etc.)?
  ☐ integration_internal — Does it call another internal microservice, platform API, or event bus topic?

Testing
  ☐ unit_test          — New business logic, utilities, or transformations that need unit test coverage.
  ☐ integration_test   — New endpoints or service calls that need API-level integration tests.
  ☐ e2e_test           — End-to-end automation test for the key user journey this requirement introduces.

DevOps
  ☐ devops             — Does delivery require new environment variables, feature flags, infrastructure changes, CI/CD pipeline updates, container config, or a deployment runbook?

Cross-cutting
  ☐ security           — Are there auth/authorisation rules, RBAC gates, secrets, encryption, or pen-test requirements?
  ☐ performance        — Are there SLA/latency targets, caching layers, DB query optimisation, or load-test baselines?
  ☐ compliance         — Are there audit logs, data-retention rules, PII masking, GDPR/PCI obligations, or regulatory sign-off?
  ☐ documentation      — Are there ADRs, API docs (OpenAPI), runbooks, or onboarding guides to write?

════════════════════════════════════════════════════════
OUTPUT FORMAT
════════════════════════════════════════════════════════
Return a raw JSON array — NO markdown fences, NO prose, ONLY the array.
Each element must have exactly these fields:

[
  {
    "title": "As a [actor], I [action] so that [outcome]",
    "type": "<one of the type values below>",
    "description": "What needs to be built, changed, or verified — 1-3 sentences.",
    "acceptanceCriteria": [
      "Given [context], when [action], then [expected result]",
      "..."
    ],
    "storyPoints": <Fibonacci: 1 | 2 | 3 | 5 | 8 | 13>
  }
]

Allowed type values (use exactly as written):
  ui_new_screen, ui_component, ui_update, ui_modal,
  api_endpoint, backend_service, background_job,
  database_schema, data_migration,
  integration_external, integration_internal,
  unit_test, integration_test, e2e_test,
  devops, security, performance, compliance, documentation

Rules:
• Generate a separate story for EACH distinct endpoint, screen, or concern — do not bundle them.
• Minimum 6 stories. There is no maximum — be thorough.
• Story titles must name the specific screen, endpoint, table, or integration — not generic labels.
• Acceptance criteria must be concrete and testable (Given/When/Then preferred).
• Story points reflect complexity of that single story, not the whole requirement.
• Skip a type only if you can state a clear reason it is not needed.`;
}

// ── AI call ───────────────────────────────────────────────────────────────────
async function decomposeWithAI(req: Requirement): Promise<AIStory[]> {
  const res = await fetch(`${AI_PROXY_URL}/api/ai/bedrock`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ prompt: buildPrompt(req) }),
  });
  if (!res.ok) throw new Error(`AI proxy error: ${res.status}`);
  const { text } = await res.json();

  const match = text.match(/\[[\s\S]*\]/);
  if (!match) throw new Error('AI did not return a valid JSON array');

  const parsed: AIStory[] = JSON.parse(match[0]);
  // Sanitise: replace any unknown type with 'backend_service'
  const knownTypes = new Set(Object.keys(TYPE_META));
  return parsed.map(s => ({
    ...s,
    type: knownTypes.has(s.type) ? s.type : 'backend_service' as StoryType,
  }));
}

// ── Story card ────────────────────────────────────────────────────────────────
function StoryCard({ story, onDelete }: { story: Story; onDelete: (id: string) => void }) {
  const [expanded, setExpanded] = useState(false);
  const meta = TYPE_META[story.type] ?? TYPE_META.backend_service;

  return (
    <div className="rounded-lg border border-border bg-card p-3 space-y-1.5">
      <div className="flex items-start gap-2">
        <span className={`inline-flex items-center gap-1 text-[10px] font-bold px-1.5 py-0.5 rounded shrink-0 mt-0.5 whitespace-nowrap ${meta.badgeColor}`}>
          {meta.icon}{meta.label}
        </span>
        <span className="flex-1 text-sm font-medium leading-snug">{story.title}</span>
        <div className="flex items-center gap-1 shrink-0">
          {story.storyPoints !== undefined && story.storyPoints > 0 && (
            <span className="text-[10px] font-bold bg-muted px-1.5 py-0.5 rounded">{story.storyPoints}pt</span>
          )}
          <button onClick={() => setExpanded(v => !v)} className="text-muted-foreground hover:text-foreground transition-colors">
            {expanded ? <ChevronUp className="size-3.5" /> : <ChevronDown className="size-3.5" />}
          </button>
          <button onClick={() => onDelete(story.id)} className="text-muted-foreground hover:text-destructive transition-colors">
            <Trash2 className="size-3.5" />
          </button>
        </div>
      </div>

      {expanded && (
        <div className="pl-1 space-y-2 pt-1">
          {story.description && (
            <p className="text-xs text-muted-foreground leading-relaxed">{story.description}</p>
          )}
          {story.acceptanceCriteria && story.acceptanceCriteria.length > 0 && (
            <div>
              <p className="text-[10px] font-semibold uppercase text-muted-foreground mb-1">Acceptance Criteria</p>
              <ul className="space-y-0.5">
                {story.acceptanceCriteria.map((ac, i) => (
                  <li key={i} className="text-xs flex gap-1.5">
                    <CheckCircle2 className="size-3 text-green-500 shrink-0 mt-0.5" />
                    <span>{ac}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

// ── Main panel ────────────────────────────────────────────────────────────────
interface StoryDecompositionPanelProps {
  requirement: Requirement;
  projectId: string;
  piId?: string;
  onClose: () => void;
}

export function StoryDecompositionPanel({ requirement, projectId, piId, onClose }: StoryDecompositionPanelProps) {
  const { user } = useAuthStore();
  const { stories, isLoading, fetchByRequirement, bulkCreate, deleteStory, clearStories } = useStoriesStore();
  const [generating, setGenerating] = useState(false);

  useEffect(() => {
    fetchByRequirement(requirement.id);
    return () => clearStories();
  }, [requirement.id]);

  const handleGenerate = async () => {
    setGenerating(true);
    try {
      const aiStories = await decomposeWithAI(requirement);
      const toCreate = aiStories.map(s => ({
        projectId,
        requirementId: requirement.id,
        piId,
        title: s.title,
        description: s.description,
        type: s.type,
        acceptanceCriteria: s.acceptanceCriteria,
        storyPoints: s.storyPoints,
        status: 'draft' as const,
        createdBy: user?.id ?? 'unknown',
      }));
      await bulkCreate(toCreate);
      toast.success(`${toCreate.length} stories generated across ${new Set(toCreate.map(s => TYPE_META[s.type]?.group)).size} layers`);
    } catch (err) {
      toast.error(`Story generation failed: ${(err as Error).message}`);
    } finally {
      setGenerating(false);
    }
  };

  // Group stories by their layer group, preserving GROUP_ORDER
  const grouped = GROUP_ORDER.reduce<{ group: string; groupColor: string; stories: Story[] }[]>((acc, g) => {
    const groupStories = stories.filter(s => (TYPE_META[s.type]?.group ?? 'Cross-cutting') === g);
    if (groupStories.length > 0) {
      const firstMeta = TYPE_META[groupStories[0].type];
      acc.push({ group: g, groupColor: firstMeta?.groupColor ?? '', stories: groupStories });
    }
    return acc;
  }, []);

  const totalPoints = stories.reduce((sum, s) => sum + (s.storyPoints ?? 0), 0);
  const layerCount = new Set(grouped.map(g => g.group)).size;

  return (
    <div className="mt-3 rounded-xl border border-indigo-200 bg-indigo-50/40 dark:bg-indigo-950/20 dark:border-indigo-800 p-4 space-y-3">
      {/* Header */}
      <div className="flex items-center gap-2">
        <Sparkles className="size-4 text-indigo-600" />
        <span className="text-sm font-semibold text-indigo-700 dark:text-indigo-300">Story Decomposition</span>
        {stories.length > 0 && (
          <>
            <span className="text-[10px] bg-indigo-100 text-indigo-700 dark:bg-indigo-900/50 dark:text-indigo-300 px-2 py-0.5 rounded-full font-medium">
              {stories.length} stories
            </span>
            <span className="text-[10px] bg-indigo-100 text-indigo-700 dark:bg-indigo-900/50 dark:text-indigo-300 px-2 py-0.5 rounded-full font-medium">
              {totalPoints}pt
            </span>
            <span className="text-[10px] bg-indigo-100 text-indigo-700 dark:bg-indigo-900/50 dark:text-indigo-300 px-2 py-0.5 rounded-full font-medium">
              {layerCount} layers
            </span>
          </>
        )}
        <button onClick={onClose} className="ml-auto text-muted-foreground hover:text-foreground">
          <X className="size-4" />
        </button>
      </div>

      {/* Generate */}
      <div className="flex items-start gap-3">
        <Button
          size="sm"
          className="h-8 text-xs bg-indigo-600 hover:bg-indigo-700 shrink-0"
          onClick={handleGenerate}
          disabled={generating}
        >
          {generating
            ? <><Loader2 className="size-3.5 mr-1.5 animate-spin" />Analysing all layers…</>
            : <><Sparkles className="size-3.5 mr-1.5" />{stories.length > 0 ? 'Re-generate' : 'Generate with AI'}</>}
        </Button>
        <p className="text-xs text-muted-foreground leading-relaxed">
          {stories.length === 0
            ? 'AI will walk through every system layer — UI screens, API endpoints, DB schema, integrations, tests, DevOps, security — and generate a story for each piece of work.'
            : `${stories.length} stories across ${layerCount} layers. Re-generate to replace all.`}
        </p>
      </div>

      {/* Loading */}
      {isLoading && (
        <div className="flex items-center gap-2 text-muted-foreground text-sm py-2">
          <Loader2 className="size-4 animate-spin" /> Loading stories…
        </div>
      )}

      {/* Stories grouped by layer */}
      {!isLoading && stories.length > 0 && (
        <div className="space-y-4">
          {grouped.map(({ group, groupColor, stories: groupStories }) => (
            <div key={group}>
              {/* Group header */}
              <div className="flex items-center gap-2 mb-2">
                <span className={`text-[10px] font-bold px-2 py-0.5 rounded ${groupColor}`}>
                  {group}
                </span>
                <span className="text-[10px] text-muted-foreground">
                  {groupStories.length} {groupStories.length === 1 ? 'story' : 'stories'} · {groupStories.reduce((s, x) => s + (x.storyPoints ?? 0), 0)}pt
                </span>
              </div>
              <div className="space-y-1.5 pl-0.5">
                {groupStories
                  .sort((a, b) => TYPE_ORDER.indexOf(a.type) - TYPE_ORDER.indexOf(b.type))
                  .map(story => (
                    <StoryCard key={story.id} story={story} onDelete={deleteStory} />
                  ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
