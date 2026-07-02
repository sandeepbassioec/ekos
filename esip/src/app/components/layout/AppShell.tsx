import { useEffect, useState, type ReactNode, type ComponentType } from 'react';
import { Link, useLocation, useNavigate } from 'react-router';
import {
  FolderKanban, Network, Calculator, Users, CheckSquare, ListTodo, Shield,
  PanelLeftClose, PanelLeft, Bell, LogOut, Settings, ChevronDown, UserCog,
  Home, SlidersHorizontal, Flag,
  LayoutDashboard, FileText, Map, CalendarDays, FolderOpen,
} from 'lucide-react';
import { db } from '../../lib/db';
import { useAuthStore } from '../../store/authStore';
import { usePersonaStore, PERSONAS, getPersona } from '../../store/personaStore';
import { useWorkflowStore } from '../../store/workflowStore';
import { useWorkingMode, useWorkingModeStore, WORKING_MODES } from '../../store/workingModeStore';
import { useNotificationStore } from '../../store/notificationStore';
import { licenseLabel } from '../../lib/entitlements';
import { useFeatureFlag } from '../../store/featureFlagStore';
import type { FeatureFlagKey } from '../../lib/featureFlags';
import { Button } from '../ui/button';
import {
  DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem,
  DropdownMenuLabel, DropdownMenuSeparator,
} from '../ui/dropdown-menu';
import { cn } from '../ui/utils';
import { AnalysisProgressModal } from '../ai/AnalysisProgressModal';

function formatTime(ts: number): string {
  const diff = Math.floor((Date.now() - ts) / 1000);
  if (diff < 60) return 'just now';
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
}

/**
 * Shared application shell (Epic A): one header + sidebar + content area wrapping
 * all authenticated routes — replacing the per-page inline back-buttons.
 * UI-consistency: composed only from `components/ui/*` + Tailwind tokens + lucide icons.
 * Persona switcher and inbox badge are wired minimally here; Epic B (persona model)
 * and Epic D (workflow inbox) flesh them out.
 */

interface NavItem {
  key: string;
  label: string;
  to: string;
  icon: ComponentType<{ className?: string }>;
  roles?: string[]; // when set, only shown to these roles
  orgFunction?: boolean; // org/admin function: gated by role only, NOT by persona
  flag?: FeatureFlagKey; // when set, hidden unless this capability flag is enabled (Epic H)
}

const NAV: NavItem[] = [
  // Adaptive home — persona/working-mode aware landing (Epic F), available to everyone
  { key: 'home', label: 'Home', to: '/home', icon: Home, orgFunction: true, flag: 'adaptiveHome' },
  // Work surface — filtered by the active persona's enabledNav
  { key: 'projects', label: 'Projects', to: '/projects/list', icon: FolderKanban },
  { key: 'architecture', label: 'Architecture', to: '/architecture', icon: Network },
  { key: 'estimation', label: 'Estimation', to: '/wizard', icon: Calculator },
  // Org/admin functions — gated by role only (persona-agnostic)
  { key: 'inbox', label: 'Inbox', to: '/inbox', icon: Bell, orgFunction: true, flag: 'inbox' },
  { key: 'teams', label: 'Teams', to: '/teams', icon: Users, roles: ['manager', 'admin'], orgFunction: true },
  { key: 'approvals', label: 'Approvals', to: '/manager/approvals', icon: CheckSquare, roles: ['manager', 'admin'], orgFunction: true },
  { key: 'tasks', label: 'My Tasks', to: '/developer/tasks', icon: ListTodo, roles: ['developer'], orgFunction: true },
  { key: 'users', label: 'Users', to: '/admin/users', icon: Users, roles: ['admin'], orgFunction: true, flag: 'adminUsers' },
  { key: 'personas', label: 'Personas', to: '/admin/personas', icon: UserCog, roles: ['admin'], orgFunction: true },
  { key: 'flags', label: 'Capabilities', to: '/admin/flags', icon: Flag, roles: ['admin'], orgFunction: true },
  { key: 'admin', label: 'Admin', to: '/admin/metamodel', icon: Shield, roles: ['admin'], orgFunction: true },
];

interface ProjectNavItem {
  key: string;
  label: string;
  suffix: string;
  icon: ComponentType<{ className?: string }>;
}

const PROJECT_NAV: ProjectNavItem[] = [
  { key: 'overview',     label: 'Overview',     suffix: '',              icon: LayoutDashboard },
  { key: 'requirements', label: 'Requirements', suffix: '/requirements', icon: FileText },
  { key: 'roadmap',      label: 'Roadmap',      suffix: '/roadmap',      icon: Map },
  { key: 'documents',    label: 'Documents',    suffix: '/documents',    icon: FolderOpen },
  { key: 'team',         label: 'Team',         suffix: '/team',         icon: Users },
  { key: 'pi-planning',  label: 'PI Planning',  suffix: '/pi-planning',  icon: CalendarDays },
];

export function AppShell({ children }: { children: ReactNode }) {
  const [collapsed, setCollapsed] = useState(false);
  const [currentProjectName, setCurrentProjectName] = useState<string | null>(null);
  const { user, logout } = useAuthStore();
  const navigate = useNavigate();
  const location = useLocation();
  const { activePersonaId, setActivePersona } = usePersonaStore();
  const persona = getPersona(activePersonaId);

  // Detect if we're inside a project route and extract the projectId
  const projectIdMatch = location.pathname.match(/^\/projects\/([^/]+)/);
  const currentProjectId = projectIdMatch?.[1];
  const inProject = !!(currentProjectId && currentProjectId !== 'list' && currentProjectId !== 'create');

  useEffect(() => {
    if (!inProject || !currentProjectId || !user) {
      setCurrentProjectName(null);
      return;
    }
    db.projects.getById(currentProjectId, user.id).then(p => {
      setCurrentProjectName(p?.name ?? null);
    }).catch(() => setCurrentProjectName(null));
  }, [currentProjectId, inProject, user]);

  // Working-mode personalization (Epic F): seeded from the persona, switchable, remembered per persona.
  const workingMode = useWorkingMode();
  const setWorkingMode = useWorkingModeStore((s) => s.setMode);

  // Feature flags (Epic H) — capability toggles; all default on.
  const flagInbox = useFeatureFlag('inbox');
  const flagWorkingMode = useFeatureFlag('workingMode');
  const flagVals: Partial<Record<FeatureFlagKey, boolean>> = {
    adaptiveHome: useFeatureFlag('adaptiveHome'),
    inbox: flagInbox,
    adminUsers: useFeatureFlag('adminUsers'),
  };

  // Workflow inbox (Epic D): load once on shell mount; badge the bell with open items routed to me.
  const assigned = useWorkflowStore((s) => s.assigned);
  const refreshInbox = useWorkflowStore((s) => s.refresh);
  useEffect(() => {
    if (flagInbox) refreshInbox();
  }, [flagInbox, refreshInbox]);

  // AI analysis notifications (bell dropdown)
  const notifications = useNotificationStore((s) => s.notifications);
  const markRead = useNotificationStore((s) => s.markRead);
  const markAllRead = useNotificationStore((s) => s.markAllRead);
  const unreadCount = notifications.filter((n) => !n.read).length;

  const role = user?.role ?? '';
  const navItems = NAV.filter((n) => {
    const roleOk = !n.roles || n.roles.includes(role);
    if (!roleOk) return false;
    if (n.flag && !flagVals[n.flag]) return false; // capability disabled (Epic H)
    // Org functions show by role; work items show only if the active persona foregrounds them.
    return n.orgFunction || persona.enabledNav.includes(n.key);
  });

  const initials = (user?.fullName || user?.email || 'U')
    .split(/[ @.]/)
    .filter(Boolean)
    .slice(0, 2)
    .map((s) => s[0]?.toUpperCase())
    .join('');

  const handleLogout = async () => {
    await logout();
    navigate('/login', { replace: true });
  };

  const isActive = (to: string) =>
    to === '/projects/list'
      ? location.pathname.startsWith('/projects') || location.pathname.startsWith('/roadmap-items')
      : location.pathname.startsWith(to);

  return (
    <div className="flex h-screen w-full overflow-hidden bg-background text-foreground">
      {/* Sidebar */}
      <aside className={cn('flex flex-col border-r border-border bg-card transition-all duration-200', collapsed ? 'w-16' : 'w-60')}>
        <div className="flex h-14 items-center gap-2 border-b border-border px-4">
          <Link to="/" className="flex items-center gap-2 font-semibold">
            <div className="flex size-7 items-center justify-center rounded-md bg-primary text-xs font-bold text-primary-foreground">E</div>
            {!collapsed && <span>ESIP</span>}
          </Link>
        </div>
        <nav className="flex-1 overflow-y-auto p-2">
          {navItems.map((item) => {
            const Icon = item.icon;
            const active = isActive(item.to);
            return (
              <Link
                key={item.to}
                to={item.to}
                title={item.label}
                className={cn(
                  'mb-1 flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors',
                  active
                    ? 'bg-accent font-medium text-accent-foreground'
                    : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground',
                  collapsed && 'justify-center px-0',
                )}
              >
                <Icon className="size-4 shrink-0" />
                {!collapsed && <span>{item.label}</span>}
              </Link>
            );
          })}

          {/* Project-scoped nav — shown when inside /projects/:id/* */}
          {inProject && (
            <>
              <div className="mx-1 my-2 border-t border-border" />
              {!collapsed && (
                <div className="flex items-center gap-1.5 px-3 pb-1 pt-0.5">
                  <FolderOpen className="size-3 text-muted-foreground shrink-0" />
                  <span className="truncate text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
                    {currentProjectName ?? 'Project'}
                  </span>
                </div>
              )}
              {PROJECT_NAV.map((item) => {
                const to = `/projects/${currentProjectId}${item.suffix}`;
                const active = item.suffix === ''
                  ? location.pathname === `/projects/${currentProjectId}`
                  : location.pathname.startsWith(to);
                const Icon = item.icon;
                return (
                  <Link
                    key={item.key}
                    to={to}
                    title={item.label}
                    className={cn(
                      'mb-1 flex items-center gap-3 rounded-md py-2 text-sm transition-colors',
                      active
                        ? 'bg-accent font-medium text-accent-foreground'
                        : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground',
                      collapsed ? 'justify-center px-0' : 'pl-6 pr-3',
                    )}
                  >
                    <Icon className="size-4 shrink-0" />
                    {!collapsed && <span>{item.label}</span>}
                  </Link>
                );
              })}
            </>
          )}
        </nav>
        <div className="border-t border-border p-2">
          <Button variant="ghost" size="sm" className="w-full justify-center" onClick={() => setCollapsed((c) => !c)} title={collapsed ? 'Expand' : 'Collapse'}>
            {collapsed ? <PanelLeft className="size-4" /> : (<><PanelLeftClose className="size-4" /><span className="ml-2">Collapse</span></>)}
          </Button>
        </div>
      </aside>

      {/* Main column */}
      <div className="flex flex-1 flex-col overflow-hidden">
        {/* Header */}
        <header className="flex h-14 shrink-0 items-center justify-between gap-4 border-b border-border bg-card px-4">
          <div className="flex items-center gap-2">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm" className="gap-2">
                <span className="hidden text-muted-foreground sm:inline">Working as</span>
                <span className="font-medium">{persona.label}</span>
                <ChevronDown className="size-3.5 opacity-60" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start" className="w-64">
              <DropdownMenuLabel>Switch persona</DropdownMenuLabel>
              <DropdownMenuSeparator />
              {PERSONAS.map((p) => (
                <DropdownMenuItem key={p.id} onClick={() => setActivePersona(p.id)} className="gap-2">
                  <span className="inline-flex w-9 shrink-0 justify-center rounded bg-muted px-1 text-xs text-muted-foreground">{p.code}</span>
                  <span className={cn(p.id === activePersonaId && 'font-medium')}>{p.label}</span>
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>

          {flagWorkingMode && (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm" className="gap-2">
                <SlidersHorizontal className="size-3.5 opacity-70" />
                <span className="hidden text-muted-foreground sm:inline">Mode</span>
                <span className="font-medium">{WORKING_MODES.find((m) => m.value === workingMode)?.label}</span>
                <ChevronDown className="size-3.5 opacity-60" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start" className="w-56">
              <DropdownMenuLabel>Working mode</DropdownMenuLabel>
              <DropdownMenuSeparator />
              {WORKING_MODES.map((m) => (
                <DropdownMenuItem key={m.value} onClick={() => setWorkingMode(activePersonaId, m.value)} className="flex-col items-start gap-0.5">
                  <span className={cn(m.value === workingMode && 'font-medium')}>{m.label}</span>
                  <span className="text-xs text-muted-foreground">{m.blurb}</span>
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
          )}
          </div>

          <div className="flex items-center gap-2">
            {flagInbox && (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="icon" title="Notifications" className="relative">
                  <Bell className="size-4" />
                  {unreadCount > 0 && (
                    <span className="absolute -right-0.5 -top-0.5 flex h-4 min-w-4 items-center justify-center rounded-full bg-destructive px-1 text-[10px] font-semibold leading-none text-destructive-foreground">
                      {unreadCount > 9 ? '9+' : unreadCount}
                    </span>
                  )}
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-80">
                <div className="flex items-center justify-between px-3 py-2">
                  <DropdownMenuLabel className="p-0 font-semibold">Notifications</DropdownMenuLabel>
                  {unreadCount > 0 && (
                    <button
                      onClick={(e) => { e.preventDefault(); markAllRead(); }}
                      className="text-xs text-muted-foreground hover:text-foreground transition-colors"
                    >
                      Mark all read
                    </button>
                  )}
                </div>
                <DropdownMenuSeparator />
                {notifications.length === 0 && (
                  <div className="px-3 py-6 text-center text-sm text-muted-foreground">
                    No notifications yet
                  </div>
                )}
                {notifications.slice(0, 8).map((n) => (
                  <DropdownMenuItem
                    key={n.id}
                    onClick={() => {
                      markRead(n.id);
                      if (n.link) navigate(n.link);
                    }}
                    className="flex-col items-start gap-0.5 px-3 py-2.5 cursor-pointer"
                  >
                    <div className="flex items-center gap-2 w-full">
                      {!n.read && (
                        <span className="h-2 w-2 rounded-full bg-primary shrink-0" />
                      )}
                      <span className={cn('flex-1 text-sm font-medium truncate', n.read && 'pl-4')}>
                        {n.title}
                      </span>
                      <span className="text-[10px] text-muted-foreground shrink-0">
                        {formatTime(n.createdAt)}
                      </span>
                    </div>
                    <p className={cn('text-xs text-muted-foreground line-clamp-2', n.read ? 'pl-4' : 'pl-4')}>
                      {n.body}
                    </p>
                  </DropdownMenuItem>
                ))}
                <DropdownMenuSeparator />
                <DropdownMenuItem onClick={() => navigate('/inbox')} className="justify-center text-sm text-muted-foreground gap-1">
                  View workflow inbox →
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
            )}

            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="sm" className="gap-2">
                  <span className="flex size-7 items-center justify-center rounded-full bg-primary text-xs font-semibold text-primary-foreground">{initials}</span>
                  <span className="hidden max-w-[10rem] truncate sm:inline">{user?.fullName || user?.email}</span>
                  <ChevronDown className="size-3.5 opacity-60" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-60">
                <DropdownMenuLabel>
                  <div className="flex flex-col">
                    <span className="truncate">{user?.fullName || 'User'}</span>
                    <span className="truncate text-xs font-normal text-muted-foreground">{user?.email}</span>
                    {role && <span className="mt-1 text-xs font-normal text-muted-foreground">Role: {role} · {licenseLabel(user?.licenseTier)}</span>}
                  </div>
                </DropdownMenuLabel>
                <DropdownMenuSeparator />
                <DropdownMenuItem disabled className="gap-2"><Settings className="size-4" /> Preferences (soon)</DropdownMenuItem>
                <DropdownMenuItem onClick={handleLogout} className="gap-2"><LogOut className="size-4" /> Log out</DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </header>

        {/* Routed page renders here */}
        <main className="flex-1 overflow-auto">{children}</main>
      </div>

      {/* Floating AI analysis progress — persists across navigation */}
      <AnalysisProgressModal />
    </div>
  );
}
