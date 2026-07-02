import React, { useState, useEffect, useMemo } from 'react';
import { useNavigate, useParams } from 'react-router';
import {
  ArrowLeft, Plus, FileText, Calendar, Edit,
  Layers, AlertTriangle, HelpCircle, Shield, CheckCircle2, Clock,
  TrendingUp, BookOpen, BarChart3, Activity, Scale,
} from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { Progress } from '../components/ui/progress';
import {
  PieChart, Pie, Cell, BarChart, Bar, XAxis, YAxis, Tooltip,
  ResponsiveContainer, Legend,
} from 'recharts';
import { useAuthStore } from '../store/authStore';
import { usePersonaStore, getPersona } from '../store/personaStore';
import { useFeatureFlag } from '../store/featureFlagStore';
import { db } from '../lib/db';
import { toast } from 'sonner';
import { generateRequirementsExcelFile } from '../utils/requirementsExcelGenerator';
import { generateRequirementsPptxFile } from '../utils/requirementsPptxGenerator';

// ── Colour palette ───────────────────────────────────────────────────────────
const PIE_COLORS = ['#3B82F6', '#8B5CF6', '#F59E0B', '#10B981', '#EF4444', '#06B6D4'];
const STATUS_CHART_COLORS: Record<string, string> = {
  Draft:          '#94A3B8',
  'Under Review': '#F59E0B',
  Approved:       '#10B981',
  Implemented:    '#3B82F6',
  Rejected:       '#EF4444',
};
const ROADMAP_STATUS_COLORS: Record<string, string> = {
  planned:     '#94A3B8',
  in_progress: '#3B82F6',
  completed:   '#10B981',
  on_hold:     '#F59E0B',
  cancelled:   '#EF4444',
};

// ── Helpers ──────────────────────────────────────────────────────────────────
function StatCard({ icon, label, value, sub, color }: { icon: React.ReactNode; label: string; value: number | string; sub?: string; color: string }) {
  return (
    <Card>
      <CardContent className="pt-5 pb-4">
        <div className="flex items-start justify-between">
          <div>
            <p className="text-sm text-muted-foreground">{label}</p>
            <p className={`text-3xl font-bold mt-0.5 ${color}`}>{value}</p>
            {sub && <p className="text-xs text-muted-foreground mt-0.5">{sub}</p>}
          </div>
          <div className={`p-2 rounded-lg bg-opacity-10 ${color.replace('text-', 'bg-').replace('-600','-100').replace('-500','-100')}`}>
            {icon}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

// ── Main Page ─────────────────────────────────────────────────────────────────
export default function ProjectDetailPage() {
  const navigate = useNavigate();
  const { projectId } = useParams();
  const { user } = useAuthStore();
  // Epic E — the Solution Design pillar surfaces only for personas that do solution work,
  // and only when the capability is enabled (Epic H feature flag).
  const activePersonaId = usePersonaStore((s) => s.activePersonaId);
  const solutionEnabled = useFeatureFlag('solutionDesign');
  const showSolution = solutionEnabled && getPersona(activePersonaId).enabledNav.includes('solution');

  const [project, setProject]     = useState<any>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [exportingDeliverables, setExportingDeliverables] = useState(false);

  // Aggregated data
  const [roadmapItems, setRoadmapItems]       = useState<any[]>([]);
  const [formalReqs,   setFormalReqs]         = useState<any[]>([]);
  const [gaps,         setGaps]               = useState<any[]>([]);
  const [confusions,   setConfusions]         = useState<any[]>([]);
  const [threats,      setThreats]            = useState<any[]>([]);
  const [suggestions,  setSuggestions]        = useState<any[]>([]);
  const [securityReqs, setSecurityReqs]       = useState<any[]>([]);
  const [nfrs,         setNfrs]               = useState<any[]>([]);
  const [roadmaps,     setRoadmaps]           = useState<any[]>([]);

  const loadDashboardData = async (showRefresh = false) => {
    if (!projectId || !user) return;
    if (showRefresh) setIsRefreshing(true);
    else setIsLoading(true);

    try {
      const [proj, rms] = await Promise.all([
        db.projects.getById(projectId, user.id),
        db.roadmaps.getByProject(projectId),
      ]);
      setProject(proj);
      setRoadmaps(rms);

      const allItems: any[] = [];
      let allFR: any[] = [], allGaps: any[] = [], allConf: any[] = [];
      let allSug: any[] = [], allSec: any[] = [], allThr: any[] = [];
      let allNfrs: any[] = [];

      for (const rm of rms) {
        const items = await db.roadmapItems.getByRoadmap(rm.id);
        allItems.push(...items.map(i => ({ ...i, roadmapType: rm.type })));

        await Promise.all(items.map(async item => {
          const [fr, gaps, conf, sug, sec, thr, nfrs] = await Promise.all([
            db.formalRequirements.getByRoadmapItem(item.id),
            db.gaps.getByRoadmapItem(item.id),
            db.confusions.getByRoadmapItem(item.id),
            db.suggestions.getByRoadmapItem(item.id),
            db.securityRequirements.getByRoadmapItem(item.id),
            db.threats.getByRoadmapItem(item.id),
            db.nfrs.getByRoadmapItem(item.id),
          ]);
          allFR   = [...allFR, ...fr];
          allGaps = [...allGaps, ...gaps];
          allConf = [...allConf, ...conf];
          allSug  = [...allSug, ...sug];
          allSec  = [...allSec, ...sec];
          allThr  = [...allThr, ...thr];
          allNfrs = [...allNfrs, ...nfrs];
        }));
      }

      setRoadmapItems(allItems);
      setFormalReqs(allFR);
      setGaps(allGaps);
      setConfusions(allConf);
      setSuggestions(allSug);
      setSecurityReqs(allSec);
      setThreats(allThr);
      setNfrs(allNfrs);
    } catch (err) {
      console.error(err);
      toast.error('Failed to load project data');
    } finally {
      setIsLoading(false);
      setIsRefreshing(false);
    }
  };

  useEffect(() => { loadDashboardData(); }, [projectId, user]);

  // ── Derived stats ──────────────────────────────────────────────────────────
  const stats = useMemo(() => {
    const approvedReqs   = formalReqs.filter(r => r.status === 'Approved').length;
    const openGaps       = gaps.filter(g => g.status === 'Open').length;
    const openConfusions = confusions.filter(c => c.status === 'Open').length;
    const criticalThreats = threats.filter(t => t.level === 'Critical' || t.level === 'High').length;
    const completedItems = roadmapItems.filter(i => i.status === 'completed').length;
    const inProgressItems = roadmapItems.filter(i => i.status === 'in_progress').length;
    const approvalRate   = formalReqs.length > 0 ? Math.round((approvedReqs / formalReqs.length) * 100) : 0;
    const roadmapProgress = roadmapItems.length > 0 ? Math.round((completedItems / roadmapItems.length) * 100) : 0;

    return {
      approvedReqs, openGaps, openConfusions, criticalThreats,
      completedItems, inProgressItems, approvalRate, roadmapProgress,
    };
  }, [formalReqs, gaps, confusions, threats, roadmapItems]);

  // ── Chart data ─────────────────────────────────────────────────────────────
  const reqsByCategoryData = useMemo(() => {
    const counts: Record<string, number> = {};
    formalReqs.forEach(r => { counts[r.category] = (counts[r.category] ?? 0) + 1; });
    return Object.entries(counts).map(([name, value]) => ({ name, value }));
  }, [formalReqs]);

  const reqsByStatusData = useMemo(() => {
    const counts: Record<string, number> = {};
    formalReqs.forEach(r => { counts[r.status] = (counts[r.status] ?? 0) + 1; });
    return Object.entries(counts).map(([name, value]) => ({ name, value, fill: STATUS_CHART_COLORS[name] ?? '#94A3B8' }));
  }, [formalReqs]);

  const roadmapProgressData = useMemo(() => {
    const byType: Record<string, Record<string, number>> = {};
    roadmapItems.forEach(item => {
      const t = item.roadmapType ?? 'Unknown';
      if (!byType[t]) byType[t] = {};
      byType[t][item.status] = (byType[t][item.status] ?? 0) + 1;
    });
    return Object.entries(byType).map(([type, statuses]) => ({
      type: type.charAt(0).toUpperCase() + type.slice(1),
      ...statuses,
    }));
  }, [roadmapItems]);

  const nfrByTypeData = useMemo(() => {
    const counts: Record<string, number> = {};
    [...nfrs, ...formalReqs.filter(r => r.category === 'Non-Functional')].forEach(n => {
      const cat = n.nfrType ?? n.category ?? 'Other';
      counts[cat] = (counts[cat] ?? 0) + 1;
    });
    return Object.entries(counts).map(([name, value]) => ({ name, value }));
  }, [nfrs, formalReqs]);

  // ── Export ─────────────────────────────────────────────────────────────────
  const handleExport = async (format: 'excel' | 'pptx' | 'both') => {
    setExportingDeliverables(true);
    try {
      const allCloudGov: any[] = [];
      for (const rm of roadmaps) {
        const items = await db.roadmapItems.getByRoadmap(rm.id);
        for (const item of items) {
          const cg = await db.cloudGovernance.getByRoadmapItem(item.id);
          allCloudGov.push(...cg);
        }
      }
      const exportData = {
        projectName: project.name,
        roadmapItemTitle: 'All Roadmap Items',
        roadmapType: 'Project-Wide',
        generatedDate: new Date().toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' }),
        formalRequirements: formalReqs, gaps, confusions: confusions, suggestions,
        securityRequirements: securityReqs, threats, nfrs, cloudGovernance: allCloudGov,
      };
      if (format === 'excel' || format === 'both') generateRequirementsExcelFile(exportData);
      if (format === 'both') await new Promise(r => setTimeout(r, 500));
      if (format === 'pptx' || format === 'both') generateRequirementsPptxFile(exportData);
      toast.success('Deliverables generated — check your downloads');
    } catch (err) {
      toast.error('Failed to generate deliverables');
    } finally {
      setExportingDeliverables(false);
    }
  };

  // ── Render ─────────────────────────────────────────────────────────────────
  if (isLoading) return (
    <div className="container max-w-7xl mx-auto py-8 px-4 text-center text-muted-foreground">
      Loading project dashboard...
    </div>
  );

  if (!project) return (
    <div className="container max-w-7xl mx-auto py-8 px-4 text-center">
      <p className="mb-4">Project not found.</p>
      <Button onClick={() => navigate('/projects/list')}>Back to Projects</Button>
    </div>
  );

  return (
    <div className="container max-w-7xl mx-auto py-8 px-4 space-y-6">

      {/* ── Header ── */}
      <div>
        <Button variant="ghost" className="-ml-2 mb-3" onClick={() => navigate('/projects/list')}>
          <ArrowLeft className="w-4 h-4 mr-2" />Back to Projects
        </Button>
        <div className="flex items-start justify-between gap-4">
          <div>
            <h1 className="text-3xl font-bold">{project.name}</h1>
            <p className="text-muted-foreground mt-1">{project.description || 'No description'}</p>
            <div className="flex items-center gap-2 mt-2">
              <Badge>{project.status}</Badge>
              {project.estimationTechnique && <Badge variant="secondary">{project.estimationTechnique}</Badge>}
              <span className="text-xs text-muted-foreground">Created {new Date(project.createdAt).toLocaleDateString()}</span>
            </div>
          </div>
        </div>
      </div>

      {/* ── Quick Nav ── */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        {[
          { label: 'Requirements', sub: 'Capture & Manage', icon: <FileText className="w-5 h-5 text-blue-600" />, bg: 'bg-blue-50', path: `/projects/${projectId}/requirements` },
          { label: 'Documents', sub: 'BRD, HLD, LLD', icon: <BookOpen className="w-5 h-5 text-amber-600" />, bg: 'bg-amber-50', path: `/projects/${projectId}/documents` },
          { label: 'Roadmap', sub: 'Timeline & Planning', icon: <Calendar className="w-5 h-5 text-purple-600" />, bg: 'bg-purple-50', path: `/projects/${projectId}/roadmap` },
          { label: 'Team', sub: 'Members & Roles', icon: <TrendingUp className="w-5 h-5 text-teal-600" />, bg: 'bg-teal-50', path: `/projects/${projectId}/team` },
          { label: 'PI Planning', sub: 'Increments & Sprints', icon: <Layers className="w-5 h-5 text-indigo-600" />, bg: 'bg-indigo-50', path: `/projects/${projectId}/pi-planning` },
          ...(showSolution ? [{ label: 'Solution Design', sub: 'Options & Trade-offs', icon: <Scale className="w-5 h-5 text-rose-600" />, bg: 'bg-rose-50', path: `/projects/${projectId}/solution` }] : []),
        ].map(item => (
          <Card key={item.label} className="hover:shadow-md transition-shadow cursor-pointer"
            onClick={item.path ? () => navigate(item.path!) : item.onClick}>
            <CardContent className="pt-4 pb-3 flex items-center gap-3">
              <div className={`p-2 rounded-lg ${item.bg}`}>{item.icon}</div>
              <div>
                <p className="font-semibold text-sm">{item.label}</p>
                <p className="text-xs text-muted-foreground">{item.sub}</p>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* ── Top Stats Row ── */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <StatCard icon={<FileText className="w-5 h-5 text-blue-600" />}
          label="Formal Requirements" value={formalReqs.length}
          sub={`${stats.approvedReqs} approved · ${stats.approvalRate}% rate`}
          color="text-blue-600" />
        <StatCard icon={<AlertTriangle className="w-5 h-5 text-orange-500" />}
          label="Open Gaps" value={stats.openGaps}
          sub={`${gaps.length} total · ${gaps.filter(g=>g.status==='Resolved').length} resolved`}
          color={stats.openGaps > 0 ? 'text-orange-500' : 'text-green-600'} />
        <StatCard icon={<Shield className="w-5 h-5 text-red-500" />}
          label="Security Threats" value={threats.length}
          sub={`${stats.criticalThreats} critical/high`}
          color={stats.criticalThreats > 0 ? 'text-red-500' : 'text-green-600'} />
        <StatCard icon={<Calendar className="w-5 h-5 text-purple-600" />}
          label="Roadmap Items" value={roadmapItems.length}
          sub={`${stats.completedItems} done · ${stats.inProgressItems} in progress`}
          color="text-purple-600" />
      </div>

      {/* ── Second Stats Row ── */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <StatCard icon={<HelpCircle className="w-5 h-5 text-blue-400" />}
          label="Open Confusions" value={stats.openConfusions}
          sub={`${confusions.length} total`}
          color={stats.openConfusions > 0 ? 'text-blue-500' : 'text-green-600'} />
        <StatCard icon={<CheckCircle2 className="w-5 h-5 text-green-600" />}
          label="Approval Rate" value={`${stats.approvalRate}%`}
          sub={`${stats.approvedReqs} of ${formalReqs.length} approved`}
          color="text-green-600" />
        <StatCard icon={<TrendingUp className="w-5 h-5 text-purple-600" />}
          label="Roadmap Progress" value={`${stats.roadmapProgress}%`}
          sub={`${stats.completedItems} of ${roadmapItems.length} items done`}
          color="text-purple-600" />
        <StatCard icon={<Activity className="w-5 h-5 text-cyan-600" />}
          label="NFRs Captured" value={nfrs.length + formalReqs.filter(r => r.category === 'Non-Functional').length}
          sub={`${securityReqs.length} security reqs`}
          color="text-cyan-600" />
      </div>

      {/* ── Roadmap Progress Bar ── */}
      {roadmapItems.length > 0 && (
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Roadmap Progress</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center gap-3">
              <Progress value={stats.roadmapProgress} className="flex-1 h-3" />
              <span className="text-sm font-semibold w-10 text-right">{stats.roadmapProgress}%</span>
            </div>
            <div className="flex gap-4 flex-wrap">
              {Object.entries(ROADMAP_STATUS_COLORS).map(([status, color]) => {
                const count = roadmapItems.filter(i => i.status === status).length;
                if (count === 0) return null;
                return (
                  <div key={status} className="flex items-center gap-1.5 text-xs text-muted-foreground">
                    <div className="w-2.5 h-2.5 rounded-full" style={{ backgroundColor: color }} />
                    <span className="capitalize">{status.replace('_', ' ')}</span>
                    <span className="font-medium text-foreground">({count})</span>
                  </div>
                );
              })}
            </div>
          </CardContent>
        </Card>
      )}

      {/* ── Charts Row ── */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">

        {/* Requirements by Category pie */}
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Requirements by Category</CardTitle>
          </CardHeader>
          <CardContent>
            {reqsByCategoryData.length === 0 ? (
              <div className="h-48 flex items-center justify-center text-muted-foreground text-sm">
                No requirements yet
              </div>
            ) : (
              <ResponsiveContainer width="100%" height={200}>
                <PieChart>
                  <Pie data={reqsByCategoryData} cx="50%" cy="50%" outerRadius={70}
                    dataKey="value" label={({ name, value }) => `${name} (${value})`} labelLine={false}
                    fontSize={11}>
                    {reqsByCategoryData.map((_, i) => (
                      <Cell key={i} fill={PIE_COLORS[i % PIE_COLORS.length]} />
                    ))}
                  </Pie>
                  <Tooltip />
                </PieChart>
              </ResponsiveContainer>
            )}
          </CardContent>
        </Card>

        {/* Requirements by Status bar */}
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Requirements by Status</CardTitle>
          </CardHeader>
          <CardContent>
            {reqsByStatusData.length === 0 ? (
              <div className="h-48 flex items-center justify-center text-muted-foreground text-sm">
                No requirements yet
              </div>
            ) : (
              <ResponsiveContainer width="100%" height={200}>
                <BarChart data={reqsByStatusData} layout="vertical" margin={{ left: 20, right: 20 }}>
                  <XAxis type="number" fontSize={11} />
                  <YAxis type="category" dataKey="name" fontSize={11} width={90} />
                  <Tooltip />
                  <Bar dataKey="value" radius={[0, 4, 4, 0]}>
                    {reqsByStatusData.map((entry, i) => (
                      <Cell key={i} fill={entry.fill} />
                    ))}
                  </Bar>
                </BarChart>
              </ResponsiveContainer>
            )}
          </CardContent>
        </Card>

        {/* NFR breakdown */}
        {nfrByTypeData.length > 0 && (
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">Non-Functional Requirements by Type</CardTitle>
            </CardHeader>
            <CardContent>
              <ResponsiveContainer width="100%" height={200}>
                <BarChart data={nfrByTypeData} margin={{ left: 10, right: 10 }}>
                  <XAxis dataKey="name" fontSize={9} angle={-30} textAnchor="end" height={50} />
                  <YAxis fontSize={11} />
                  <Tooltip />
                  <Bar dataKey="value" fill="#8B5CF6" radius={[4, 4, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>
        )}

        {/* Roadmap items by type */}
        {roadmapProgressData.length > 0 && (
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">Roadmap Items by Type & Status</CardTitle>
            </CardHeader>
            <CardContent>
              <ResponsiveContainer width="100%" height={200}>
                <BarChart data={roadmapProgressData} margin={{ left: 10, right: 10 }}>
                  <XAxis dataKey="type" fontSize={11} />
                  <YAxis fontSize={11} />
                  <Tooltip />
                  <Legend fontSize={10} />
                  {['planned', 'in_progress', 'completed', 'on_hold'].map(status => (
                    <Bar key={status} dataKey={status} stackId="a"
                      fill={ROADMAP_STATUS_COLORS[status]}
                      name={status.replace('_', ' ')} radius={status === 'planned' ? [4,4,0,0] : undefined} />
                  ))}
                </BarChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>
        )}
      </div>

      {/* ── AI Insights Health ── */}
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium">AI Insights Health</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {[
              { label: 'Gaps', total: gaps.length, open: gaps.filter(g => g.status === 'Open').length, color: 'orange', icon: <AlertTriangle className="w-4 h-4" /> },
              { label: 'Confusions', total: confusions.length, open: confusions.filter(c => c.status === 'Open').length, color: 'blue', icon: <HelpCircle className="w-4 h-4" /> },
              { label: 'Threats', total: threats.length, open: threats.filter(t => t.status === 'Identified').length, color: 'red', icon: <Shield className="w-4 h-4" /> },
              { label: 'Suggestions', total: suggestions.length, open: suggestions.filter(s => s.status === 'Pending').length, color: 'green', icon: <CheckCircle2 className="w-4 h-4" /> },
            ].map(item => (
              <div key={item.label} className="space-y-2">
                <div className="flex items-center gap-2 text-sm font-medium">
                  {item.icon}
                  {item.label}
                </div>
                <Progress value={item.total > 0 ? Math.round(((item.total - item.open) / item.total) * 100) : 100} className="h-2" />
                <p className="text-xs text-muted-foreground">
                  {item.open} open · {item.total - item.open} resolved · {item.total} total
                </p>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* ── Roadmap items list ── */}
      {roadmapItems.length > 0 && (
        <Card>
          <CardHeader className="pb-2">
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm font-medium">Roadmap Items</CardTitle>
              <Button size="sm" variant="outline" onClick={() => navigate(`/projects/${projectId}/roadmap`)}>
                View Full Roadmap
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {roadmapItems.slice(0, 8).map(item => (
                <div key={item.id}
                  className="flex items-center justify-between gap-3 p-3 rounded-lg border hover:bg-muted/50 cursor-pointer transition-colors"
                  onClick={() => navigate(`/roadmap-items/${item.id}/knowledge-hub`)}>
                  <div className="flex items-center gap-3 min-w-0">
                    <BookOpen className="w-4 h-4 text-muted-foreground shrink-0" />
                    <div className="min-w-0">
                      <p className="text-sm font-medium truncate">{item.title}</p>
                      <p className="text-xs text-muted-foreground capitalize">{item.roadmapType} roadmap · {item.priority} priority</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2 shrink-0">
                    <Badge variant="outline" className="text-xs capitalize">
                      {item.status.replace('_', ' ')}
                    </Badge>
                    {item.progress > 0 && (
                      <span className="text-xs text-muted-foreground w-10 text-right">{item.progress}%</span>
                    )}
                  </div>
                </div>
              ))}
              {roadmapItems.length > 8 && (
                <p className="text-xs text-muted-foreground text-center pt-1">
                  + {roadmapItems.length - 8} more — view full roadmap
                </p>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      {/* ── Empty state (fresh project) ── */}
      {roadmapItems.length === 0 && formalReqs.length === 0 && (
        <Card className="border-dashed">
          <CardContent className="py-12 text-center">
            <Activity className="w-10 h-10 mx-auto mb-3 text-muted-foreground opacity-40" />
            <p className="font-medium text-muted-foreground">No data yet</p>
            <p className="text-sm text-muted-foreground mt-1">
              Create a roadmap, add items, and open KnowledgeHub to start capturing requirements.
            </p>
            <div className="flex gap-2 justify-center mt-4">
              <Button size="sm" onClick={() => navigate(`/projects/${projectId}/roadmap`)}>
                <Calendar className="w-4 h-4 mr-2" />Create Roadmap
              </Button>
              <Button size="sm" variant="outline" onClick={() => navigate(`/projects/${projectId}/requirements`)}>
                <FileText className="w-4 h-4 mr-2" />Add Requirements
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

    </div>
  );
}
