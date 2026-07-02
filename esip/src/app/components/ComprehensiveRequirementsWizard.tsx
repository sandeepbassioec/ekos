import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router';
import { 
  CheckCircle, 
  Circle, 
  ChevronRight, 
  ChevronLeft,
  Shield,
  Zap,
  Database,
  Cloud,
  TestTube,
  Activity,
  DollarSign,
  Palette,
  BookOpen,
  BarChart3,
  Save
} from 'lucide-react';
import { Button } from './ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './ui/card';
import { Progress } from './ui/progress';
import { Badge } from './ui/badge';
import { ComprehensiveRequirements } from '../types/comprehensive-requirements';
import { toast } from 'sonner';
import { useRequirementsStore } from '../store/requirementsStore';

// Import step components
import { NFRStep } from './wizard-steps/NFRStep';
import { SecurityStep } from './wizard-steps/SecurityStep';
import { IntegrationStep } from './wizard-steps/IntegrationStep';
import { DataStep } from './wizard-steps/DataStep';
import { InfrastructureStep } from './wizard-steps/InfrastructureStep';
import { TestingStep } from './wizard-steps/TestingStep';
import { OperationsStep } from './wizard-steps/OperationsStep';
import { BusinessConstraintsStep } from './wizard-steps/BusinessConstraintsStep';
import { UXStep } from './wizard-steps/UXStep';
import { TrainingStep } from './wizard-steps/TrainingStep';
import { ReportingStep } from './wizard-steps/ReportingStep';

interface WizardStep {
  id: number;
  key: string;
  title: string;
  description: string;
  icon: React.ReactNode;
  required: boolean;
  component: React.ComponentType<any>;
}

const wizardSteps: WizardStep[] = [
  {
    id: 1,
    key: 'nonFunctional',
    title: 'Non-Functional Requirements',
    description: 'Performance, scalability, availability',
    icon: <Zap className="w-5 h-5" />,
    required: true,
    component: NFRStep,
  },
  {
    id: 2,
    key: 'security',
    title: 'Security & Compliance',
    description: 'Authentication, authorization, compliance',
    icon: <Shield className="w-5 h-5" />,
    required: true,
    component: SecurityStep,
  },
  {
    id: 3,
    key: 'integration',
    title: 'Integration Requirements',
    description: 'APIs, third-party services, legacy systems',
    icon: <Database className="w-5 h-5" />,
    required: false,
    component: IntegrationStep,
  },
  {
    id: 4,
    key: 'data',
    title: 'Data Requirements',
    description: 'Data model, migration, backup',
    icon: <Database className="w-5 h-5" />,
    required: true,
    component: DataStep,
  },
  {
    id: 5,
    key: 'infrastructure',
    title: 'Infrastructure & Deployment',
    description: 'Hosting, cloud, CI/CD',
    icon: <Cloud className="w-5 h-5" />,
    required: true,
    component: InfrastructureStep,
  },
  {
    id: 6,
    key: 'testing',
    title: 'Testing Strategy',
    description: 'Test coverage, automation, UAT',
    icon: <TestTube className="w-5 h-5" />,
    required: true,
    component: TestingStep,
  },
  {
    id: 7,
    key: 'operations',
    title: 'Operations & Support',
    description: 'Monitoring, logging, SLAs',
    icon: <Activity className="w-5 h-5" />,
    required: true,
    component: OperationsStep,
  },
  {
    id: 8,
    key: 'businessConstraints',
    title: 'Business Constraints',
    description: 'Budget, timeline, resources',
    icon: <DollarSign className="w-5 h-5" />,
    required: true,
    component: BusinessConstraintsStep,
  },
  {
    id: 9,
    key: 'ux',
    title: 'User Experience',
    description: 'Design system, branding, accessibility',
    icon: <Palette className="w-5 h-5" />,
    required: false,
    component: UXStep,
  },
  {
    id: 10,
    key: 'training',
    title: 'Training & Documentation',
    description: 'User guides, technical docs',
    icon: <BookOpen className="w-5 h-5" />,
    required: false,
    component: TrainingStep,
  },
  {
    id: 11,
    key: 'reporting',
    title: 'Reporting & Analytics',
    description: 'Dashboards, reports, KPIs',
    icon: <BarChart3 className="w-5 h-5" />,
    required: false,
    component: ReportingStep,
  },
];

interface ComprehensiveRequirementsWizardProps {
  projectId: string;
  projectName: string;
  onComplete: (requirements: ComprehensiveRequirements) => void;
  onCancel: () => void;
}

export function ComprehensiveRequirementsWizard({
  projectId,
  projectName,
  onComplete,
  onCancel,
}: ComprehensiveRequirementsWizardProps) {
  const [currentStep, setCurrentStep] = useState(0);
  const [completedSteps, setCompletedSteps] = useState<Set<number>>(new Set());
  const [requirements, setRequirements] = useState<Partial<ComprehensiveRequirements>>({
    projectId,
    projectName,
    industry: '',
    projectType: '',
    capturedAt: new Date().toISOString(),
    capturedBy: 'current-user',
    completionPercentage: 0,
    lastUpdated: new Date().toISOString(),
  });

  const { fetchRequirements, createRequirement, bulkCreateRequirements } = useRequirementsStore();

  // Load from Supabase on mount
  useEffect(() => {
    const loadRequirements = async () => {
      try {
        await fetchRequirements(projectId);
      } catch (error) {
        console.error('Failed to load requirements:', error);
      }
    };
    loadRequirements();
  }, [projectId, fetchRequirements]);

  // Auto-save to Supabase
  useEffect(() => {
    if (requirements.projectId) {
      // Debounced save - implementation can be added if needed
    }
  }, [requirements, projectId]);

  const currentStepConfig = wizardSteps[currentStep];
  const CurrentStepComponent = currentStepConfig.component;

  const handleStepDataChange = (stepKey: string, data: any) => {
    setRequirements((prev) => ({
      ...prev,
      [stepKey]: data,
      lastUpdated: new Date().toISOString(),
    }));
  };

  const handleNext = () => {
    // Mark current step as completed
    setCompletedSteps((prev) => new Set(prev).add(currentStep));
    
    if (currentStep < wizardSteps.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      handleComplete();
    }
  };

  const handlePrevious = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleSkip = () => {
    if (currentStep < wizardSteps.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      handleComplete();
    }
  };

  const handleComplete = () => {
    const completionPercentage = Math.round((completedSteps.size / wizardSteps.length) * 100);
    const finalRequirements: ComprehensiveRequirements = {
      ...requirements,
      completionPercentage,
      lastUpdated: new Date().toISOString(),
    } as ComprehensiveRequirements;

    // Save to Supabase
    bulkCreateRequirements(projectId, finalRequirements);
    
    toast.success('Comprehensive requirements saved successfully!');
    onComplete(finalRequirements);
  };

  const progressPercentage = Math.round(((currentStep + 1) / wizardSteps.length) * 100);

  return (
    <div className="min-h-screen bg-gray-50 py-8">
      <div className="container max-w-7xl mx-auto px-4">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold mb-2">Comprehensive Requirements Capture</h1>
          <p className="text-muted-foreground">
            {projectName} • Step {currentStep + 1} of {wizardSteps.length}
          </p>
          <Progress value={progressPercentage} className="mt-4 h-2" />
        </div>

        <div className="grid grid-cols-12 gap-6">
          {/* Sidebar - Steps Navigator */}
          <div className="col-span-3">
            <Card className="sticky top-4">
              <CardHeader>
                <CardTitle className="text-sm">Progress</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                {wizardSteps.map((step, idx) => (
                  <button
                    key={step.id}
                    onClick={() => setCurrentStep(idx)}
                    className={`w-full flex items-center gap-3 p-3 rounded-lg text-left transition-all ${
                      idx === currentStep
                        ? 'bg-blue-100 border-2 border-blue-500'
                        : completedSteps.has(idx)
                        ? 'bg-green-50 border border-green-300 hover:bg-green-100'
                        : 'bg-gray-50 border border-gray-200 hover:bg-gray-100'
                    }`}
                  >
                    {completedSteps.has(idx) ? (
                      <CheckCircle className="w-5 h-5 text-green-600 flex-shrink-0" />
                    ) : (
                      <Circle className="w-5 h-5 text-gray-400 flex-shrink-0" />
                    )}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className={`text-xs font-semibold ${
                          idx === currentStep ? 'text-blue-700' : 'text-gray-700'
                        }`}>
                          {step.title}
                        </span>
                        {step.required && (
                          <Badge variant="outline" className="text-xs">Required</Badge>
                        )}
                      </div>
                    </div>
                    {step.icon}
                  </button>
                ))}
              </CardContent>
            </Card>
          </div>

          {/* Main Content Area */}
          <div className="col-span-9">
            <Card>
              <CardHeader>
                <div className="flex items-center gap-4">
                  <div className="w-12 h-12 rounded-full bg-blue-100 flex items-center justify-center">
                    {currentStepConfig.icon}
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-1">
                      <CardTitle>{currentStepConfig.title}</CardTitle>
                      {currentStepConfig.required && (
                        <Badge variant="destructive" className="text-xs">Required</Badge>
                      )}
                    </div>
                    <CardDescription>{currentStepConfig.description}</CardDescription>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                {/* Render current step component */}
                <CurrentStepComponent
                  data={requirements[currentStepConfig.key]}
                  onChange={(data: any) => handleStepDataChange(currentStepConfig.key, data)}
                />
              </CardContent>
            </Card>

            {/* Navigation */}
            <div className="flex justify-between items-center mt-6">
              <Button
                variant="outline"
                onClick={handlePrevious}
                disabled={currentStep === 0}
              >
                <ChevronLeft className="w-4 h-4 mr-2" />
                Previous
              </Button>

              <div className="flex gap-3">
                {!currentStepConfig.required && (
                  <Button variant="ghost" onClick={handleSkip}>
                    Skip
                  </Button>
                )}
                <Button variant="outline" onClick={onCancel}>
                  Save & Exit
                </Button>
                <Button onClick={handleNext} className="bg-blue-600 hover:bg-blue-700">
                  {currentStep === wizardSteps.length - 1 ? (
                    <>
                      <Save className="w-4 h-4 mr-2" />
                      Complete
                    </>
                  ) : (
                    <>
                      Next
                      <ChevronRight className="w-4 h-4 ml-2" />
                    </>
                  )}
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}