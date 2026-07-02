import { useEffect, useMemo, type ReactNode } from 'react';
import { useNavigate } from 'react-router';
import {
  Network, Scale, Calendar, FileText, Sparkles, Calculator,
  FolderKanban, Inbox, CheckSquare, Plus, ArrowRight, Clock,
} from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { useAuthStore } from '../store/authStore';
import { usePersonaStore, getPersona, type WorkingMode } from '../store/personaStore';
import { useWorkingMode, useWorkingModeStore, WORKING_MODES } from '../store/workingModeStore';
import { useProjectsStore } from '../store/projectsStore';

/**
 * Adaptive workspace home (Epic F). Lands the user into "what you're doing right now":
 * it reshapes by the active PERSONA (which capabilities surface) and the current WORKING MODE
 * (which actions lead). Switching mode here re-tailors the page; the choice is remembered per persona.
 */

interface HomeAction {
  key: string;
  label: string;
  sub: string;
  icon: ReactNode;
  to: string;
  show: boolean;
}

function buildActions(mode: WorkingMode, enabledNav: string[], role: string, projectId?: string): HomeAction[] {
  const projPath = (suffix: string) => (projectId ? `/projects/${projectId}${suffix}` : '/projects/list');
  const has = (k: string) => enabledNav.includes(k);

  switch (mode) {
    case 'design':
      return [
        { key: 'architecture', label: 'Architecture', sub: 'HLD, LLD & diagrams', icon: <Network className="w-5 h-5 text-orange-600" />, to: '/architecture', show: has('architecture') },
        { key: 'solution', label: 'Solution Design', sub: 'Options & trade-offs', icon: <Scale className="w-5 h-5 text-rose-600" />, to: projPath('/solution'), show: has('solution') },
        { key: 'roadmap', label: 'Roadmap', sub: 'Plan the timeline', icon: <Calendar className="w-5 h-5 text-purple-600" />, to: projPath('/roadmap'), show: true },
      ];
    case 'requirements':
      return [
        { key: 'reqs', label: 'Requirements Board', sub: 'Capture & manage', icon: <FileText className="w-5 h-5 text-blue-600" />, to: projPath('/requirements'), show: true },
        { key: 'comprehensive', label: 'Comprehensive Requirements', sub: 'Full requirement set', icon: <Sparkles className="w-5 h-5 text-indigo-600" />, to: projPath('/requirements/comprehensive'), show: true },
        { key: 'roadmap', label: 'Roadmap', sub: 'Plan the timeline', icon: <Calendar className="w-5 h-5 text-purple-600" />, to: projPath('/roadmap'), show: true },
      ];
    case 'estimation':
      return [
        { key: 'wizard', label: 'Estimation Wizard', sub: 'PERT, FPA, UCP', icon: <Calculator className="w-5 h-5 text-green-600" />, to: '/wizard', show: has('estimation') },
        { key: 'projects', label: 'Projects', sub: 'Open or create', icon: <FolderKanban className="w-5 h-5 text-blue-600" />, to: '/projects/list', show: true },
      ];
    case 'review':
      return [
        { key: 'inbox', label: 'Inbox', sub: 'Clarifications & approvals', icon: <Inbox className="w-5 h-5 text-blue-600" />, to: '/inbox', show: true },
        { key: 'approvals', label: 'Approvals', sub: 'Pending decisions', icon: <CheckSquare className="w-5 h-5 text-green-600" />, to: '/manager/approvals', show: role === 'manager' || role === 'admin' },
        { key: 'projects', label: 'Projects', sub: 'Open or create', icon: <FolderKanban className="w-5 h-5 text-blue-600" />, to: '/projects/list', show: true },
      ];
  }
}

export default function WorkspaceHomePage() {
  const navigate = useNavigate();
  const user = useAuthStore((s) => s.user);
  const activePersonaId = usePersonaStore((s) => s.activePersonaId);
  const persona = getPersona(activePersonaId);
  const mode = useWorkingMode();
  const setMode = useWorkingModeStore((s) => s.setMode);

  const { projects, fetchProjects } = useProjectsStore();
  useEffect(() => {
    fetchProjects();
  }, [fetchProjects]);

  const recent = useMemo(
    () => [...projects].sort((a, b) => (b.updatedAt || '').localeCompare(a.updatedAt || '')).slice(0, 5),
    [projects],
  );
  const primaryProjectId = recent[0]?.id;

  const actions = buildActions(mode, persona.enabledNav, user?.role ?? '', primaryProjectId).filter((a) => a.show);
  const modeMeta = WORKING_MODES.find((m) => m.value === mode);
  const firstName = (user?.fullName || user?.email || 'there').split(/[ @.]/)[0];

  return (
    <div className="mx-auto max-w-5xl p-6 space-y-6">
      {/* Greeting */}
      <div>
        <h1 className="text-2xl font-semibold">Welcome back, {firstName}</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Working as <span className="font-medium text-foreground">{persona.label}</span> — pick a mode and pick up where you left off.
        </p>
      </div>

      {/* Working-mode switcher (the personalization centerpiece) */}
      <div>
        <div className="mb-2 flex items-center gap-2 text-sm font-medium">
          <span>Working mode</span>
          <Badge variant="secondary">{modeMeta?.label}</Badge>
        </div>
        <div className="inline-flex flex-wrap gap-1 rounded-lg border border-border bg-card p-1">
          {WORKING_MODES.map((m) => {
            const active = m.value === mode;
            return (
              <button
                key={m.value}
                onClick={() => setMode(activePersonaId, m.value)}
                className={
                  'rounded-md px-3 py-1.5 text-sm transition-colors ' +
                  (active ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground')
                }
              >
                {m.label}
              </button>
            );
          })}
        </div>
        {modeMeta && <p className="mt-2 text-sm text-muted-foreground">{modeMeta.blurb}</p>}
      </div>

      {/* Mode-tailored actions */}
      <div>
        <h2 className="mb-2 text-sm font-semibold">Start here</h2>
        {!primaryProjectId && (mode === 'design' || mode === 'requirements') && (
          <p className="mb-2 text-xs text-muted-foreground">
            No project yet — these open the projects list so you can create or choose one first.
          </p>
        )}
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {actions.map((a) => (
            <Card key={a.key} className="cursor-pointer transition-shadow hover:shadow-md" onClick={() => navigate(a.to)}>
              <CardContent className="pt-4 pb-3 flex items-center gap-3">
                <div className="rounded-lg bg-muted p-2">{a.icon}</div>
                <div className="min-w-0">
                  <p className="font-semibold text-sm">{a.label}</p>
                  <p className="text-xs text-muted-foreground truncate">{a.sub}</p>
                </div>
                <ArrowRight className="w-4 h-4 text-muted-foreground ml-auto shrink-0" />
              </CardContent>
            </Card>
          ))}
        </div>
      </div>

      {/* Recent projects */}
      <div>
        <div className="mb-2 flex items-center justify-between">
          <h2 className="text-sm font-semibold">Recent projects</h2>
          <div className="flex gap-2">
            <Button size="sm" variant="outline" onClick={() => navigate('/projects/list')}>All projects</Button>
            <Button size="sm" onClick={() => navigate('/projects/create')}>
              <Plus className="w-4 h-4 mr-1" /> New project
            </Button>
          </div>
        </div>
        {recent.length === 0 ? (
          <Card className="border-dashed">
            <CardContent className="py-8 text-center text-sm text-muted-foreground">
              No projects yet. Create your first to start capturing requirements, roadmaps, and designs.
            </CardContent>
          </Card>
        ) : (
          <div className="space-y-2">
            {recent.map((p) => (
              <div
                key={p.id}
                className="flex items-center justify-between gap-3 rounded-lg border border-border p-3 hover:bg-muted/50 cursor-pointer transition-colors"
                onClick={() => navigate(`/projects/${p.id}`)}
              >
                <div className="min-w-0">
                  <p className="text-sm font-medium truncate">{p.name}</p>
                  {p.description && <p className="text-xs text-muted-foreground truncate">{p.description}</p>}
                </div>
                <div className="flex items-center gap-2 shrink-0 text-xs text-muted-foreground">
                  <Badge variant="outline" className="capitalize">{String(p.status).toLowerCase()}</Badge>
                  {p.updatedAt && (
                    <span className="flex items-center gap-1">
                      <Clock className="w-3 h-3" />
                      {new Date(p.updatedAt).toLocaleDateString()}
                    </span>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
