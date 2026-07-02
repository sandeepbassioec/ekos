import React, { useState } from 'react';
import { useNavigate } from 'react-router';
import {
  CheckCircle2, ChevronRight, Loader2, Rocket, User2, Layers,
  Target, FileText, ArrowLeft, Play, ExternalLink,
} from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { toast } from 'sonner';
import { db } from '../lib/db';
import { useAuthStore } from '../store/authStore';

// ── Demo seed data ─────────────────────────────────────────────────────────────
const PROJECT_TEMPLATES = [
  {
    id: 'po',
    persona: 'Product Owner',
    icon: <User2 className="size-5 text-indigo-600" />,
    description: 'Pre-seeded with a RetailNext modernization project: 2 PIs, 14 requirements at various stages, PI allocations, and acceptance criteria.',
    available: true,
  },
  {
    id: 'tech',
    persona: 'Tech Lead',
    icon: <Layers className="size-5 text-slate-400" />,
    description: 'Architecture decisions, solution trade-offs, and NFR library. Coming soon.',
    available: false,
  },
  {
    id: 'ea',
    persona: 'Enterprise Architect',
    icon: <Target className="size-5 text-slate-400" />,
    description: 'TOGAF phases, ArchiMate layers, capability roadmap. Coming soon.',
    available: false,
  },
];

interface SeedStep {
  label: string;
  status: 'pending' | 'running' | 'done' | 'error';
  detail?: string;
}

// ── Seed runner ────────────────────────────────────────────────────────────────
async function runPOSeed(
  projectName: string,
  userId: string,
  onStep: (index: number, status: SeedStep['status'], detail?: string) => void,
): Promise<{ projectId: string }> {

  // Step 0: Project
  onStep(0, 'running');
  const project = await db.projects.create({
    userId,
    name: projectName,
    description: 'Modernize the RetailNext in-store platform — unified checkout, loyalty, inventory management and staff tools across 200+ stores.',
    status: 'IN_PROGRESS',
    estimationTechnique: 'story_points',
    onboardingMode: 'active_project',
  });
  onStep(0, 'done', `Project ID: ${project.id}`);

  // Step 1: PIs
  onStep(1, 'running');
  const pi1 = await db.pis.create({
    projectId: project.id,
    name: 'PI-1 · Q1 2026',
    startDate: '2026-01-05',
    endDate: '2026-03-27',
    status: 'active',
    objectives: 'Unified Checkout v1, Loyalty Integration, POS Performance baseline',
  });
  const pi2 = await db.pis.create({
    projectId: project.id,
    name: 'PI-2 · Q2 2026',
    startDate: '2026-04-06',
    endDate: '2026-06-26',
    status: 'planning',
    objectives: 'Staff Tools, Inventory API, Accessibility & Compliance hardening',
  });
  onStep(1, 'done', `PI-1: ${pi1.id.slice(-6)}, PI-2: ${pi2.id.slice(-6)}`);

  // Step 2: Requirements
  onStep(2, 'running');
  const baseReqs = [
    // Finalized + allocated to PI-1
    {
      title: 'Unified Checkout Flow — Guest & Member',
      category: 'feature', priority: 'critical', status: 'finalized',
      piId: pi1.id,
      metadata: {
        type: 'new_screen', complexity: 'high', estimatedHours: 80,
        allocatedPiName: pi1.name, programIncrement: pi1.name,
        userStory: 'As a store associate, I want a unified checkout that handles both guest and loyalty member payments so that I can complete transactions without switching systems.',
        acceptanceCriteria: [
          { id: '1', text: 'Guest checkout completes in < 3 steps', completed: true },
          { id: '2', text: 'Member lookup by phone number or card swipe', completed: true },
          { id: '3', text: 'Supports split tender (cash + card)', completed: false },
        ],
        finalizedBy: userId,
        finalizedAt: new Date().toISOString(),
      },
    },
    {
      title: 'Loyalty Points Earn & Redeem at POS',
      category: 'feature', priority: 'high', status: 'finalized',
      piId: pi1.id,
      metadata: {
        type: 'feature_change', complexity: 'medium', estimatedHours: 40,
        allocatedPiName: pi1.name, programIncrement: pi1.name,
        userStory: 'As a loyalty member, I want my points to be automatically applied at checkout so that I get my rewards without manual entry.',
        acceptanceCriteria: [
          { id: '1', text: 'Points balance displayed before payment', completed: true },
          { id: '2', text: 'Partial redemption supported (e.g. use 500 of 1200 points)', completed: false },
        ],
        finalizedBy: userId,
        finalizedAt: new Date().toISOString(),
      },
    },
    {
      title: 'POS Performance — Page Load < 1s',
      category: 'nfr', priority: 'high', status: 'finalized',
      piId: pi1.id,
      metadata: {
        type: 'performance', complexity: 'high', estimatedHours: 60,
        allocatedPiName: pi1.name, programIncrement: pi1.name,
        userStory: 'As a store associate, I want the checkout screen to load in under 1 second so that customer queues move faster.',
        acceptanceCriteria: [
          { id: '1', text: 'Lighthouse Performance score ≥ 90', completed: false },
          { id: '2', text: 'Time-to-interactive < 1s on 4G connection', completed: false },
        ],
        finalizedBy: userId,
        finalizedAt: new Date().toISOString(),
      },
    },
    {
      title: 'Real-time Inventory Sync (POS ↔ WMS)',
      category: 'integration', priority: 'high', status: 'finalized',
      piId: pi1.id,
      metadata: {
        type: 'integration', complexity: 'high', estimatedHours: 56,
        allocatedPiName: pi1.name, programIncrement: pi1.name,
        userStory: 'As a store manager, I want inventory to update in real-time when items are sold so that I can trust stock counts for replenishment decisions.',
        acceptanceCriteria: [
          { id: '1', text: 'Stock decrement propagates to WMS within 5 seconds of sale', completed: true },
          { id: '2', text: 'Out-of-stock items show alert during checkout', completed: false },
        ],
        finalizedBy: userId,
        finalizedAt: new Date().toISOString(),
      },
    },
    // Finalized + allocated to PI-2
    {
      title: 'Staff Task Management — Daily Shift Planner',
      category: 'feature', priority: 'medium', status: 'finalized',
      piId: pi2.id,
      metadata: {
        type: 'new_screen', complexity: 'medium', estimatedHours: 48,
        allocatedPiName: pi2.name, programIncrement: pi2.name,
        userStory: 'As a store manager, I want to assign and track daily tasks for each staff member so that all opening/closing duties are completed.',
        acceptanceCriteria: [
          { id: '1', text: 'Tasks can be assigned to a staff member with due time', completed: false },
          { id: '2', text: 'Staff can mark tasks complete from mobile device', completed: false },
        ],
        finalizedBy: userId,
        finalizedAt: new Date().toISOString(),
      },
    },
    {
      title: 'Accessibility Audit — WCAG 2.1 AA Compliance',
      category: 'compliance', priority: 'high', status: 'finalized',
      piId: pi2.id,
      metadata: {
        type: 'security', complexity: 'medium', estimatedHours: 32,
        allocatedPiName: pi2.name, programIncrement: pi2.name,
        userStory: 'As a compliance officer, I want all POS screens to meet WCAG 2.1 AA so that our platform is accessible and meets legal requirements.',
        acceptanceCriteria: [
          { id: '1', text: 'Color contrast ratio ≥ 4.5:1 for all text', completed: false },
          { id: '2', text: 'All interactive elements reachable by keyboard', completed: false },
          { id: '3', text: 'Screen reader announces all status changes', completed: false },
        ],
        finalizedBy: userId,
        finalizedAt: new Date().toISOString(),
      },
    },
    // Approved (ready to finalize)
    {
      title: 'Customer-Facing Display — Live Order Summary',
      category: 'feature', priority: 'medium', status: 'approved',
      metadata: {
        type: 'new_screen', complexity: 'low', estimatedHours: 24,
        userStory: 'As a customer, I want to see a live order summary on the customer-facing screen so that I can verify my items and total before payment.',
        acceptanceCriteria: [
          { id: '1', text: 'Screen updates as items are scanned', completed: false },
        ],
      },
    },
    {
      title: 'Returns & Exchange Flow',
      category: 'feature', priority: 'medium', status: 'approved',
      metadata: {
        type: 'update_screen', complexity: 'medium', estimatedHours: 36,
        userStory: 'As a store associate, I want to process returns and exchanges from the same POS screen so that I do not need a separate terminal.',
        acceptanceCriteria: [
          { id: '1', text: 'Receipt lookup by transaction ID or loyalty card', completed: false },
          { id: '2', text: 'Refund issued to original payment method', completed: false },
        ],
      },
    },
    // In review
    {
      title: 'Gift Card Balance Check & Activation',
      category: 'feature', priority: 'low', status: 'review',
      metadata: {
        type: 'feature_change', complexity: 'low', estimatedHours: 16,
        userStory: 'As a customer, I want to check my gift card balance and activate new cards at any POS so that I can use my gift card immediately.',
        acceptanceCriteria: [],
      },
    },
    {
      title: 'API Rate Limiting & Throttling',
      category: 'nfr', priority: 'high', status: 'review',
      metadata: {
        type: 'api_development', complexity: 'medium', estimatedHours: 20,
        userStory: 'As the platform team, we want API rate limiting so that individual stores cannot degrade service for others.',
        acceptanceCriteria: [],
      },
    },
    // Draft
    {
      title: 'Multi-Language Support (ES, ZH, FR)',
      category: 'feature', priority: 'low', status: 'draft',
      metadata: { type: 'ui_ux_change', complexity: 'high', estimatedHours: 80, acceptanceCriteria: [] },
    },
    {
      title: 'Offline Mode — Queue Transactions When Disconnected',
      category: 'feature', priority: 'critical', status: 'draft',
      metadata: { type: 'feature_change', complexity: 'high', estimatedHours: 96, acceptanceCriteria: [] },
    },
    {
      title: 'PCI DSS Tokenization Audit Trail',
      category: 'compliance', priority: 'critical', status: 'draft',
      metadata: { type: 'security', complexity: 'medium', estimatedHours: 40, acceptanceCriteria: [] },
    },
    {
      title: 'Mobile App for Store Associates',
      category: 'feature', priority: 'medium', status: 'draft',
      metadata: { type: 'new_screen', complexity: 'high', estimatedHours: 120, acceptanceCriteria: [] },
    },
  ];

  for (const r of baseReqs) {
    await db.requirements.create({
      projectId: project.id,
      piId: r.piId,
      title: r.title,
      description: `Requirement for ${projectName}: ${r.title}`,
      category: r.category,
      priority: r.priority,
      status: r.status,
      tags: [r.category],
      metadata: r.metadata,
      createdBy: userId,
    });
  }
  onStep(2, 'done', `${baseReqs.length} requirements seeded`);

  return { projectId: project.id };
}

// ── Wizard ─────────────────────────────────────────────────────────────────────
type WizardStep = 'persona' | 'configure' | 'seeding' | 'done';

export default function DemoSetupWizard() {
  const navigate = useNavigate();
  const { user } = useAuthStore();
  const [step, setStep] = useState<WizardStep>('persona');
  const [selectedPersona, setSelectedPersona] = useState('po');
  const [projectName, setProjectName] = useState('RetailNext Platform Modernization');
  const [seedSteps, setSeedSteps] = useState<SeedStep[]>([
    { label: 'Create demo project', status: 'pending' },
    { label: 'Set up Program Increments (PI-1 & PI-2)', status: 'pending' },
    { label: 'Seed 14 requirements across all statuses', status: 'pending' },
  ]);
  const [resultProjectId, setResultProjectId] = useState('');

  const updateStep = (index: number, status: SeedStep['status'], detail?: string) => {
    setSeedSteps(prev => prev.map((s, i) => i === index ? { ...s, status, detail } : s));
  };

  const handleStartSeed = async () => {
    setStep('seeding');
    try {
      const { projectId } = await runPOSeed(
        projectName,
        user?.id ?? 'demo',
        updateStep,
      );
      setResultProjectId(projectId);
      setStep('done');
    } catch (err) {
      toast.error(`Demo setup failed: ${(err as Error).message}`);
      setStep('configure');
    }
  };

  return (
    <div className="max-w-2xl mx-auto py-10 px-4 space-y-6">
      {/* Back */}
      <button
        onClick={() => navigate(-1)}
        className="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors"
      >
        <ArrowLeft className="size-4" /> Back
      </button>

      {/* Title */}
      <div>
        <h1 className="text-2xl font-bold flex items-center gap-2">
          <Rocket className="size-6 text-indigo-600" />
          Demo Setup Wizard
        </h1>
        <p className="text-muted-foreground text-sm mt-1">
          Seed a fully-populated demo project so you can explore ESIP from a specific persona's perspective.
        </p>
      </div>

      {/* Step: Persona selection */}
      {step === 'persona' && (
        <div className="space-y-4">
          <h2 className="text-base font-semibold">1 — Choose a persona</h2>
          <div className="grid gap-3">
            {PROJECT_TEMPLATES.map(t => (
              <Card
                key={t.id}
                className={`cursor-pointer transition-all ${t.available ? '' : 'opacity-50 cursor-not-allowed'} ${selectedPersona === t.id && t.available ? 'border-indigo-400 ring-1 ring-indigo-300 bg-indigo-50/30' : ''}`}
                onClick={() => t.available && setSelectedPersona(t.id)}
              >
                <CardContent className="p-4 flex items-start gap-3">
                  <div className="mt-0.5">{t.icon}</div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-semibold">{t.persona}</span>
                      {!t.available && (
                        <span className="text-[10px] bg-muted text-muted-foreground px-1.5 py-0.5 rounded font-medium">Coming soon</span>
                      )}
                      {selectedPersona === t.id && t.available && (
                        <CheckCircle2 className="size-4 text-indigo-600 ml-auto" />
                      )}
                    </div>
                    <p className="text-xs text-muted-foreground mt-0.5">{t.description}</p>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
          <Button
            onClick={() => setStep('configure')}
            className="w-full bg-indigo-600 hover:bg-indigo-700"
          >
            Continue <ChevronRight className="size-4 ml-1" />
          </Button>
        </div>
      )}

      {/* Step: Configure */}
      {step === 'configure' && (
        <div className="space-y-4">
          <h2 className="text-base font-semibold">2 — Configure</h2>
          <div className="space-y-2">
            <Label htmlFor="projName">Demo Project Name</Label>
            <Input
              id="projName"
              value={projectName}
              onChange={e => setProjectName(e.target.value)}
              placeholder="RetailNext Platform Modernization"
            />
            <p className="text-xs text-muted-foreground">This project will be created in your workspace. You can delete it later.</p>
          </div>

          <Card className="bg-muted/30">
            <CardHeader className="pb-2 pt-4 px-4">
              <CardTitle className="text-sm">What will be seeded</CardTitle>
            </CardHeader>
            <CardContent className="px-4 pb-4 space-y-1.5">
              {[
                '1 project with description and metadata',
                '2 Program Increments (PI-1 · Q1 2026 and PI-2 · Q2 2026)',
                '14 requirements across all statuses (Draft, Review, Approved, Finalized)',
                '6 requirements finalized and allocated to a PI — ready for Story Decomposition',
                'Acceptance criteria on finalized requirements',
              ].map((item, i) => (
                <div key={i} className="flex items-start gap-2 text-sm">
                  <CheckCircle2 className="size-3.5 text-green-600 mt-0.5 shrink-0" />
                  <span>{item}</span>
                </div>
              ))}
            </CardContent>
          </Card>

          <div className="flex gap-2">
            <Button variant="outline" onClick={() => setStep('persona')} className="flex-1">Back</Button>
            <Button
              onClick={handleStartSeed}
              disabled={!projectName.trim()}
              className="flex-1 bg-indigo-600 hover:bg-indigo-700"
            >
              <Play className="size-4 mr-1.5" /> Run Setup
            </Button>
          </div>
        </div>
      )}

      {/* Step: Seeding in progress */}
      {step === 'seeding' && (
        <div className="space-y-4">
          <h2 className="text-base font-semibold">3 — Seeding demo data…</h2>
          <div className="space-y-2">
            {seedSteps.map((s, i) => (
              <div key={i} className="flex items-center gap-3 p-3 rounded-lg border border-border bg-card">
                {s.status === 'pending' && <div className="size-4 rounded-full border-2 border-border shrink-0" />}
                {s.status === 'running' && <Loader2 className="size-4 animate-spin text-indigo-600 shrink-0" />}
                {s.status === 'done' && <CheckCircle2 className="size-4 text-green-600 shrink-0" />}
                {s.status === 'error' && <div className="size-4 rounded-full bg-destructive shrink-0" />}
                <div className="flex-1 min-w-0">
                  <p className={`text-sm ${s.status === 'running' ? 'font-medium' : ''}`}>{s.label}</p>
                  {s.detail && <p className="text-xs text-muted-foreground">{s.detail}</p>}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Step: Done */}
      {step === 'done' && (
        <div className="space-y-4">
          <div className="flex flex-col items-center py-6 gap-3 text-center">
            <div className="size-14 rounded-full bg-green-100 flex items-center justify-center">
              <CheckCircle2 className="size-8 text-green-600" />
            </div>
            <h2 className="text-xl font-bold">Demo ready!</h2>
            <p className="text-muted-foreground text-sm max-w-md">
              Your Product Owner demo project has been seeded. Explore the key workflows below.
            </p>
          </div>

          <div className="grid gap-2">
            {[
              {
                icon: <FileText className="size-4 text-indigo-600" />,
                label: 'Requirements Board',
                description: 'View all 14 requirements, see finalized items, allocate to PI',
                href: `/projects/${resultProjectId}/requirements`,
              },
              {
                icon: <Layers className="size-4 text-purple-600" />,
                label: 'PI Planning',
                description: 'View PI-1 and PI-2 program increments with auto-generated sprints',
                href: `/projects/${resultProjectId}/pi-planning`,
              },
              {
                icon: <Target className="size-4 text-blue-600" />,
                label: 'Project Overview',
                description: 'Project details, team, and quick navigation',
                href: `/projects/${resultProjectId}`,
              },
            ].map(link => (
              <Card
                key={link.href}
                className="cursor-pointer hover:border-primary/50 transition-all"
                onClick={() => navigate(link.href)}
              >
                <CardContent className="p-3 flex items-center gap-3">
                  <div className="size-8 rounded-lg bg-muted flex items-center justify-center shrink-0">
                    {link.icon}
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="font-medium text-sm">{link.label}</p>
                    <p className="text-xs text-muted-foreground">{link.description}</p>
                  </div>
                  <ExternalLink className="size-3.5 text-muted-foreground shrink-0" />
                </CardContent>
              </Card>
            ))}
          </div>

          <Button
            variant="outline"
            className="w-full"
            onClick={() => navigate('/projects/list')}
          >
            Go to Projects List
          </Button>
        </div>
      )}
    </div>
  );
}
