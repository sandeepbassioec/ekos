import React, { useState } from 'react';
import { useNavigate } from 'react-router';
import {
  ArrowLeft, Save, Sparkles, FolderOpen, Archive,
  CheckCircle2,
} from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent } from '../components/ui/card';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Textarea } from '../components/ui/textarea';
import { useAuthStore } from '../store/authStore';
import { db } from '../lib/db';
import { toast } from 'sonner';
import type { ProjectOnboardingMode } from '../lib/db/types';

const ONBOARDING_MODES: {
  id: ProjectOnboardingMode;
  icon: React.ReactNode;
  title: string;
  subtitle: string;
  bullets: string[];
  color: string;
  border: string;
}[] = [
  {
    id: 'new',
    icon: <Sparkles className="w-6 h-6 text-blue-600" />,
    title: 'Fresh Start',
    subtitle: 'Brand new project — nothing exists yet.',
    bullets: [
      'Build requirements from scratch',
      'Full PI + Sprint planning from day one',
      'AI analysis as you upload documents',
    ],
    color: 'bg-blue-50',
    border: 'border-blue-200',
  },
  {
    id: 'active',
    icon: <FolderOpen className="w-6 h-6 text-emerald-600" />,
    title: 'Active Project',
    subtitle: 'Project is already in flight — upload where you are today.',
    bullets: [
      'Upload current PRD / specs to build a baseline',
      'Paste open decision threads still unresolved',
      'ESIP tracks everything new from this moment forward',
    ],
    color: 'bg-emerald-50',
    border: 'border-emerald-200',
  },
  {
    id: 'retrospective',
    icon: <Archive className="w-6 h-6 text-amber-600" />,
    title: 'Institutional Memory',
    subtitle: 'Project is complete — preserve it for future reference.',
    bullets: [
      'Upload final documents for searchable archive',
      'No active planning — pure knowledge base',
      '"Why did we decide X?" answered in seconds',
    ],
    color: 'bg-amber-50',
    border: 'border-amber-200',
  },
];

export default function CreateProject() {
  const navigate = useNavigate();
  const { user } = useAuthStore();
  const [step, setStep] = useState<'mode' | 'details'>('mode');
  const [isLoading, setIsLoading] = useState(false);

  const [onboardingMode, setOnboardingMode] = useState<ProjectOnboardingMode>('new');
  const [formData, setFormData] = useState({ name: '', description: '', estimationTechnique: '' });
  const [errors, setErrors] = useState<Record<string, string>>({});

  const selectedMode = ONBOARDING_MODES.find(m => m.id === onboardingMode)!;

  const handleChange = (field: string, value: string) => {
    setFormData(prev => ({ ...prev, [field]: value }));
    if (errors[field]) setErrors(prev => ({ ...prev, [field]: '' }));
  };

  const validate = () => {
    const e: Record<string, string> = {};
    if (!formData.name.trim()) e.name = 'Project name is required';
    if (!formData.description.trim()) e.description = 'Project description is required';
    setErrors(e);
    return Object.keys(e).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!validate()) { toast.error('Please fill in all required fields'); return; }
    if (!user) { toast.error('You must be logged in'); return; }

    setIsLoading(true);
    try {
      const project = await db.projects.create({
        userId: user.id,
        name: formData.name,
        description: formData.description,
        estimationTechnique: formData.estimationTechnique || undefined,
        status: 'draft',
        onboardingMode,
      });
      toast.success('Project created!');
      // Always go to team setup first — the natural next step
      navigate(`/projects/${project.id}/team`);
    } catch (error: any) {
      toast.error(error.message || 'Failed to create project');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-indigo-50 to-purple-50 py-8">
      <div className="max-w-3xl mx-auto px-6">

        <div className="mb-6">
          <Button variant="ghost" onClick={() => step === 'details' ? setStep('mode') : navigate('/projects/list')} className="mb-4 -ml-2">
            <ArrowLeft className="w-4 h-4 mr-2" />
            {step === 'details' ? 'Back' : 'Back to Projects'}
          </Button>
          <h1 className="text-3xl font-bold text-gray-900">
            {step === 'mode' ? 'How are you starting?' : 'Project Details'}
          </h1>
          <p className="text-gray-600 mt-1">
            {step === 'mode'
              ? 'Choose how this project begins — this shapes what ESIP asks you to do first.'
              : 'Name your project and give a brief description.'}
          </p>
          {/* Step indicator */}
          <div className="flex items-center gap-2 mt-3">
            {['mode', 'details'].map((s, i) => (
              <React.Fragment key={s}>
                <div className={`flex items-center gap-1.5 text-sm ${step === s ? 'text-primary font-medium' : 'text-muted-foreground'}`}>
                  <div className={`w-5 h-5 rounded-full flex items-center justify-center text-xs font-semibold border-2 ${step === s ? 'border-primary bg-primary text-white' : (i === 0 && step === 'details') ? 'border-primary bg-primary/10 text-primary' : 'border-muted-foreground/30 text-muted-foreground'}`}>
                    {i === 0 && step === 'details' ? <CheckCircle2 className="w-3 h-3" /> : i + 1}
                  </div>
                  {s === 'mode' ? 'Starting point' : 'Details'}
                </div>
                {i === 0 && <div className="flex-1 h-px bg-muted-foreground/20 mx-1" />}
              </React.Fragment>
            ))}
          </div>
        </div>

        {/* STEP 1 — Onboarding mode */}
        {step === 'mode' && (
          <div className="space-y-4">
            {ONBOARDING_MODES.map(mode => (
              <label
                key={mode.id}
                className={`flex gap-4 p-5 rounded-xl border-2 cursor-pointer transition-all ${
                  onboardingMode === mode.id
                    ? `${mode.border} ${mode.color} shadow-sm`
                    : 'border-transparent bg-white hover:border-muted-foreground/20 hover:shadow-sm'
                }`}
              >
                <input
                  type="radio"
                  name="onboardingMode"
                  value={mode.id}
                  checked={onboardingMode === mode.id}
                  onChange={() => setOnboardingMode(mode.id)}
                  className="mt-1 shrink-0"
                />
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    {mode.icon}
                    <span className="font-semibold text-gray-900">{mode.title}</span>
                  </div>
                  <p className="text-sm text-gray-600 mb-2">{mode.subtitle}</p>
                  <ul className="space-y-1">
                    {mode.bullets.map(b => (
                      <li key={b} className="flex items-start gap-1.5 text-sm text-gray-500">
                        <span className="text-green-500 mt-0.5 shrink-0">✓</span>
                        {b}
                      </li>
                    ))}
                  </ul>
                </div>
              </label>
            ))}

            <div className="pt-2">
              <Button className="w-full" size="lg" onClick={() => setStep('details')}>
                Continue with {selectedMode.title}
                <ArrowLeft className="w-4 h-4 ml-2 rotate-180" />
              </Button>
            </div>
          </div>
        )}

        {/* STEP 2 — Project details */}
        {step === 'details' && (
          <form onSubmit={handleSubmit}>
            {/* Mode reminder chip */}
            <div className={`flex items-center gap-2 px-3 py-2 rounded-lg ${selectedMode.color} border ${selectedMode.border} mb-5 text-sm`}>
              {selectedMode.icon}
              <span className="font-medium">{selectedMode.title}</span>
              <span className="text-muted-foreground">— {selectedMode.subtitle}</span>
            </div>

            <Card>
              <CardContent className="space-y-5 pt-5">
                <div>
                  <Label htmlFor="name">Project Name <span className="text-red-500">*</span></Label>
                  <Input
                    id="name"
                    value={formData.name}
                    onChange={e => handleChange('name', e.target.value)}
                    placeholder="e.g., FieldOps Mobile 2.0"
                    className={`mt-1 ${errors.name ? 'border-red-500' : ''}`}
                  />
                  {errors.name && <p className="text-red-500 text-xs mt-1">{errors.name}</p>}
                </div>

                <div>
                  <Label htmlFor="description">Description <span className="text-red-500">*</span></Label>
                  <Textarea
                    id="description"
                    value={formData.description}
                    onChange={e => handleChange('description', e.target.value)}
                    placeholder="Brief overview of the project goals and objectives"
                    rows={3}
                    className={`mt-1 ${errors.description ? 'border-red-500' : ''}`}
                  />
                  {errors.description && <p className="text-red-500 text-xs mt-1">{errors.description}</p>}
                </div>

                {onboardingMode !== 'retrospective' && (
                  <div>
                    <Label htmlFor="estimationTechnique">Estimation Technique</Label>
                    <select
                      id="estimationTechnique"
                      value={formData.estimationTechnique}
                      onChange={e => handleChange('estimationTechnique', e.target.value)}
                      className="mt-1 w-full border rounded-md px-3 py-2 text-sm border-gray-300 bg-background"
                    >
                      <option value="">Select (optional)</option>
                      <option value="Function Point Analysis">Function Point Analysis</option>
                      <option value="Use Case Points">Use Case Points</option>
                      <option value="Story Points">Story Points</option>
                      <option value="Wideband Delphi">Wideband Delphi</option>
                      <option value="COCOMO">COCOMO</option>
                      <option value="Other">Other</option>
                    </select>
                  </div>
                )}

                <div className="flex gap-3 pt-2">
                  <Button type="button" variant="outline" onClick={() => setStep('mode')} className="flex-1" disabled={isLoading}>
                    Back
                  </Button>
                  <Button type="submit" disabled={isLoading} className="flex-1">
                    {isLoading ? 'Creating…' : (
                      <>
                        <Save className="w-4 h-4 mr-2" />
                        Create & Set Up Team
                      </>
                    )}
                  </Button>
                </div>
              </CardContent>
            </Card>
          </form>
        )}
      </div>
    </div>
  );
}
