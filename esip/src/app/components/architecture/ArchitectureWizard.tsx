import React, { useState } from 'react';
import { ArrowRight, ArrowLeft, Sparkles, FileText, Database, GitBranch, Code } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card';
import { Progress } from '../ui/progress';
import { ProjectInput } from './steps/ProjectInput';
import { RequirementsAnalysis } from './steps/RequirementsAnalysis';
import { useNavigate } from 'react-router';

interface ArchitectureWizardProps {
  onComplete?: (data: any) => void;
}

export function ArchitectureWizard({ onComplete }: ArchitectureWizardProps) {
  const navigate = useNavigate();
  const [currentStep, setCurrentStep] = useState(1);
  const [wizardData, setWizardData] = useState({
    projectSource: '',
    requirements: '',
    analysis: null,
  });

  const steps = [
    { id: 1, title: 'Project Input', icon: FileText, component: ProjectInput },
    { id: 2, title: 'Requirements Analysis', icon: Sparkles, component: RequirementsAnalysis },
    { id: 3, title: 'Entity Mapping', icon: Database },
    { id: 4, title: 'Flow Design', icon: GitBranch },
    { id: 5, title: 'Generate Documents', icon: Code },
  ];

  const handleDataChange = (newData: any) => {
    setWizardData(newData);
  };

  const nextStep = () => {
    if (currentStep < steps.length) {
      setCurrentStep(currentStep + 1);
    } else {
      // Final step - save and navigate
      localStorage.setItem('architecture-wizard-data', JSON.stringify(wizardData));
      if (onComplete) {
        onComplete(wizardData);
      }
      navigate('/architecture/generate');
    }
  };

  const prevStep = () => {
    if (currentStep > 1) {
      setCurrentStep(currentStep - 1);
    }
  };

  const canProceed = () => {
    switch (currentStep) {
      case 1:
        return wizardData.requirements && wizardData.requirements.length > 50;
      case 2:
        return wizardData.analysis !== null;
      default:
        return true;
    }
  };

  const progress = (currentStep / steps.length) * 100;
  const CurrentStepComponent = steps[currentStep - 1].component;

  return (
    <div className="max-w-6xl mx-auto">
      {/* Progress Header */}
      <div className="mb-8">
        <div className="flex justify-between items-center mb-4">
          {steps.map((step) => {
            const Icon = step.icon;
            return (
              <div
                key={step.id}
                className={`flex items-center ${
                  step.id < steps.length ? 'flex-1' : ''
                }`}
              >
                <div
                  className={`flex flex-col items-center ${
                    step.id === currentStep
                      ? 'text-primary'
                      : step.id < currentStep
                      ? 'text-green-600'
                      : 'text-muted-foreground'
                  }`}
                >
                  <div
                    className={`w-12 h-12 rounded-full flex items-center justify-center mb-2 ${
                      step.id === currentStep
                        ? 'bg-primary text-primary-foreground'
                        : step.id < currentStep
                        ? 'bg-green-600 text-white'
                        : 'bg-muted'
                    }`}
                  >
                    <Icon className="w-6 h-6" />
                  </div>
                  <span className="text-xs font-medium text-center">
                    {step.title}
                  </span>
                </div>
                {step.id < steps.length && (
                  <div className="flex-1 h-1 mx-4 bg-muted relative">
                    <div
                      className={`absolute top-0 left-0 h-full transition-all ${
                        step.id < currentStep ? 'bg-green-600 w-full' : 'w-0'
                      }`}
                    />
                  </div>
                )}
              </div>
            );
          })}
        </div>
        <Progress value={progress} className="h-2" />
      </div>

      {/* Step Content */}
      <Card>
        <CardHeader>
          <CardTitle>{steps[currentStep - 1].title}</CardTitle>
          <CardDescription>
            Step {currentStep} of {steps.length}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {/* Step-specific content */}
          <div className="min-h-[400px]">
            {CurrentStepComponent ? (
              <CurrentStepComponent 
                data={wizardData} 
                onChange={handleDataChange}
              />
            ) : (
              <div className="text-center py-20 text-muted-foreground">
                <p>Step {currentStep} - Coming Soon</p>
              </div>
            )}
          </div>

          {/* Navigation */}
          <div className="flex justify-between mt-8 pt-4 border-t">
            <Button
              variant="outline"
              onClick={prevStep}
              disabled={currentStep === 1}
            >
              <ArrowLeft className="w-4 h-4 mr-2" />
              Previous
            </Button>
            <Button onClick={nextStep} disabled={!canProceed()}>
              {currentStep === steps.length ? 'Generate Documents' : 'Next'}
              <ArrowRight className="w-4 h-4 ml-2" />
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}