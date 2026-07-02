import React, { useState, useEffect } from 'react';
import { ArrowLeft, ArrowRight, Save, Sparkles } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../components/ui/card';
import { Progress } from '../components/ui/progress';
import { useNavigate } from 'react-router';
import { INDUSTRY_TEMPLATES, getIndustryTemplate, getProcessesByIndustry } from '../lib/industry-templates';
import { QUESTIONNAIRES, getQuestionnaire } from '../lib/questionnaires';
import { IndustryTemplate, ProcessTemplate, Question, AutoSuggestion } from '../types/business-requirements';
import { Badge } from '../components/ui/badge';
import { Checkbox } from '../components/ui/checkbox';
import { Label } from '../components/ui/label';
import { toast } from 'sonner';

export default function BusinessRequirementsWizard() {
  const navigate = useNavigate();
  const [currentStep, setCurrentStep] = useState(1);
  const [selectedIndustry, setSelectedIndustry] = useState<IndustryTemplate | null>(null);
  const [selectedProcesses, setSelectedProcesses] = useState<string[]>([]);
  const [answers, setAnswers] = useState<Record<string, any>>({});
  const [autoSuggestions, setAutoSuggestions] = useState<AutoSuggestion[]>([]);

  const steps = [
    { id: 1, title: 'Industry Selection' },
    { id: 2, title: 'Business Processes' },
    { id: 3, title: 'Discovery Questions' },
    { id: 4, title: 'Smart Recommendations' },
    { id: 5, title: 'Review & Generate' },
  ];

  const progress = (currentStep / steps.length) * 100;

  const handleNext = () => {
    if (currentStep === 1 && !selectedIndustry) {
      toast.error('Please select an industry');
      return;
    }
    if (currentStep === 2 && selectedProcesses.length === 0) {
      toast.error('Please select at least one business process');
      return;
    }
    if (currentStep < steps.length) {
      setCurrentStep(currentStep + 1);
    }
  };

  const handlePrev = () => {
    if (currentStep > 1) setCurrentStep(currentStep - 1);
  };

  const handleIndustrySelect = (template: IndustryTemplate) => {
    setSelectedIndustry(template);
    setSelectedProcesses([]);
    setAnswers({});
    toast.success(`${template.name} template loaded`);
  };

  const toggleProcess = (processId: string) => {
    if (selectedProcesses.includes(processId)) {
      setSelectedProcesses(selectedProcesses.filter((p) => p !== processId));
    } else {
      setSelectedProcesses([...selectedProcesses, processId]);
    }
  };

  const handleAnswer = (questionId: string, value: any, question: Question) => {
    setAnswers({ ...answers, [questionId]: value });

    // Collect auto-suggestions from selected options
    if (question.options && Array.isArray(value)) {
      const suggestions: AutoSuggestion[] = [];
      value.forEach((v: string) => {
        const option = question.options?.find((o) => o.value === v);
        if (option?.autoSuggests) {
          suggestions.push(...option.autoSuggests);
        }
      });
      setAutoSuggestions([...autoSuggestions, ...suggestions]);
    } else if (question.options) {
      const option = question.options.find((o) => o.value === value);
      if (option?.autoSuggests) {
        setAutoSuggestions([...autoSuggestions, ...option.autoSuggests]);
      }
    }
  };

  const handleGenerate = () => {
    // Save requirement context
    const context = {
      industry: selectedIndustry?.id,
      processes: selectedProcesses,
      answers,
      suggestions: autoSuggestions,
      timestamp: new Date().toISOString(),
    };
    
    localStorage.setItem('business_requirements_context', JSON.stringify(context));
    toast.success('Requirements captured! Generating artifacts...');
    
    // Navigate to generation page
    navigate('/requirements/generate');
  };

  return (
    <div className="container max-w-6xl mx-auto py-8 px-4">
      <div className="mb-8">
        <Button variant="ghost" onClick={() => navigate('/')}>
          <ArrowLeft className="w-4 h-4 mr-2" />
          Back to Home
        </Button>
        <h1 className="text-3xl font-bold mt-4 mb-2">Business Requirements Capture</h1>
        <p className="text-muted-foreground">
          Smart, industry-specific requirement gathering with AI-powered suggestions
        </p>
      </div>

      {/* Progress */}
      <div className="mb-8">
        <div className="flex justify-between mb-4">
          {steps.map((step) => (
            <div
              key={step.id}
              className={`text-sm font-medium ${
                step.id === currentStep
                  ? 'text-primary'
                  : step.id < currentStep
                  ? 'text-green-600'
                  : 'text-muted-foreground'
              }`}
            >
              {step.id}. {step.title}
            </div>
          ))}
        </div>
        <Progress value={progress} className="h-2" />
      </div>

      {/* Step 1: Industry Selection */}
      {currentStep === 1 && (
        <div>
          <Card className="mb-6">
            <CardHeader>
              <CardTitle>Select Your Industry</CardTitle>
              <CardDescription>Choose the industry that best matches your project</CardDescription>
            </CardHeader>
          </Card>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {INDUSTRY_TEMPLATES.map((template) => (
              <Card
                key={template.id}
                className={`cursor-pointer transition-all hover:shadow-lg ${
                  selectedIndustry?.id === template.id ? 'ring-2 ring-primary' : ''
                }`}
                onClick={() => handleIndustrySelect(template)}
              >
                <CardHeader>
                  <div className="flex items-center gap-3 mb-2">
                    <div className="text-4xl">{template.icon}</div>
                    <CardTitle className="text-lg">{template.name}</CardTitle>
                  </div>
                  <CardDescription className="text-sm">{template.description}</CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="space-y-2 text-xs text-muted-foreground">
                    <div>
                      <strong>Processes:</strong> {template.processTemplates.length}
                    </div>
                    <div>
                      <strong>Compliance:</strong> {template.complianceRequirements.slice(0, 2).join(', ')}
                      {template.complianceRequirements.length > 2 && '...'}
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      )}

      {/* Step 2: Business Processes */}
      {currentStep === 2 && selectedIndustry && (
        <div>
          <Card className="mb-6">
            <CardHeader>
              <CardTitle>Select Business Processes</CardTitle>
              <CardDescription>
                Choose the processes that are part of your project (from {selectedIndustry.name})
              </CardDescription>
            </CardHeader>
          </Card>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {selectedIndustry.processTemplates.map((processId) => {
              const isSelected = selectedProcesses.includes(processId);
              return (
                <Card
                  key={processId}
                  className={`cursor-pointer transition-all hover:shadow-md ${
                    isSelected ? 'ring-2 ring-primary bg-primary/5' : ''
                  }`}
                  onClick={() => toggleProcess(processId)}
                >
                  <CardHeader>
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <CardTitle className="text-base">
                          {processId.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
                        </CardTitle>
                        <CardDescription className="text-xs mt-1">
                          Business process template
                        </CardDescription>
                      </div>
                      <Checkbox checked={isSelected} />
                    </div>
                  </CardHeader>
                </Card>
              );
            })}
          </div>

          {selectedProcesses.length > 0 && (
            <Card className="mt-6">
              <CardContent className="pt-6">
                <div className="flex items-center gap-2 text-sm">
                  <span className="font-semibold">Selected:</span>
                  {selectedProcesses.map((p) => (
                    <Badge key={p} variant="secondary">
                      {p.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
                    </Badge>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      )}

      {/* Step 3: Discovery Questions */}
      {currentStep === 3 && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Discovery Questions</CardTitle>
              <CardDescription>Answer these questions to help us understand your requirements</CardDescription>
            </CardHeader>
          </Card>

          {renderQuestionnaire(QUESTIONNAIRES.business_discovery, answers, handleAnswer)}
        </div>
      )}

      {/* Step 4: Smart Recommendations */}
      {currentStep === 4 && (
        <div>
          <Card className="mb-6">
            <CardHeader>
              <div className="flex items-center gap-2">
                <Sparkles className="w-5 h-5 text-yellow-500" />
                <CardTitle>Smart Recommendations</CardTitle>
              </div>
              <CardDescription>
                Based on your answers, we recommend the following capabilities and considerations
              </CardDescription>
            </CardHeader>
          </Card>

          <div className="space-y-4">
            {['capability', 'integration', 'compliance', 'risk'].map((type) => {
              const suggestions = autoSuggestions.filter((s) => s.type === type);
              if (suggestions.length === 0) return null;

              return (
                <Card key={type}>
                  <CardHeader>
                    <CardTitle className="text-lg capitalize">{type}s</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-3">
                      {suggestions.map((suggestion, idx) => (
                        <div key={idx} className="flex items-start gap-3 p-3 border rounded">
                          <Badge
                            variant={
                              suggestion.priority === 'high'
                                ? 'destructive'
                                : suggestion.priority === 'medium'
                                ? 'default'
                                : 'secondary'
                            }
                          >
                            {suggestion.priority}
                          </Badge>
                          <div className="flex-1">
                            <div className="font-semibold">{suggestion.value}</div>
                            <div className="text-sm text-muted-foreground">{suggestion.reason}</div>
                          </div>
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              );
            })}

            {autoSuggestions.length === 0 && (
              <Card>
                <CardContent className="py-8 text-center text-muted-foreground">
                  Answer the discovery questions to see personalized recommendations
                </CardContent>
              </Card>
            )}
          </div>
        </div>
      )}

      {/* Step 5: Review */}
      {currentStep === 5 && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Review & Generate</CardTitle>
              <CardDescription>Review your selections and generate technical artifacts</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <Label className="text-sm font-semibold">Industry:</Label>
                <div className="text-lg flex items-center gap-2 mt-1">
                  <span>{selectedIndustry?.icon}</span>
                  <span>{selectedIndustry?.name}</span>
                </div>
              </div>

              <div>
                <Label className="text-sm font-semibold">Business Processes ({selectedProcesses.length}):</Label>
                <div className="flex flex-wrap gap-2 mt-2">
                  {selectedProcesses.map((p) => (
                    <Badge key={p} variant="secondary">
                      {p.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
                    </Badge>
                  ))}
                </div>
              </div>

              <div>
                <Label className="text-sm font-semibold">Answers Provided:</Label>
                <div className="text-muted-foreground mt-1">{Object.keys(answers).length} questions answered</div>
              </div>

              <div>
                <Label className="text-sm font-semibold">Recommendations:</Label>
                <div className="text-muted-foreground mt-1">{autoSuggestions.length} suggestions generated</div>
              </div>
            </CardContent>
          </Card>

          <Card className="bg-primary/5 border-primary/20">
            <CardContent className="pt-6">
              <div className="flex items-center gap-3">
                <Sparkles className="w-8 h-8 text-primary" />
                <div className="flex-1">
                  <div className="font-semibold">Ready to Generate</div>
                  <div className="text-sm text-muted-foreground">
                    We'll create WBS items, user stories, architecture components, and effort estimates
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Navigation */}
      <div className="flex justify-between mt-8">
        <Button variant="outline" onClick={handlePrev} disabled={currentStep === 1}>
          <ArrowLeft className="w-4 h-4 mr-2" />
          Previous
        </Button>
        {currentStep < steps.length ? (
          <Button onClick={handleNext}>
            Next
            <ArrowRight className="w-4 h-4 ml-2" />
          </Button>
        ) : (
          <Button onClick={handleGenerate} className="bg-gradient-to-r from-blue-600 to-purple-600">
            <Sparkles className="w-4 h-4 mr-2" />
            Generate Artifacts
          </Button>
        )}
      </div>
    </div>
  );
}

// Helper function to render questionnaire
function renderQuestionnaire(
  questionnaire: any,
  answers: Record<string, any>,
  onAnswer: (qId: string, value: any, question: Question) => void
) {
  return questionnaire.questions.map((question: Question) => {
    // Check conditional logic
    if (question.conditionalLogic) {
      const shouldShow = question.conditionalLogic.showIf.every((condition) => {
        const answer = answers[condition.questionId];
        if (condition.operator === 'equals') return answer === condition.value;
        if (condition.operator === 'contains') return Array.isArray(answer) && answer.includes(condition.value);
        return false;
      });
      if (!shouldShow) return null;
    }

    return (
      <Card key={question.id}>
        <CardHeader>
          <CardTitle className="text-base">{question.text}</CardTitle>
          {question.helpText && <CardDescription className="text-xs">{question.helpText}</CardDescription>}
        </CardHeader>
        <CardContent>
          {question.type === 'single_choice' && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {question.options?.map((option) => (
                <Card
                  key={option.value}
                  className={`cursor-pointer transition-all hover:shadow-md ${
                    answers[question.id] === option.value ? 'ring-2 ring-primary bg-primary/5' : ''
                  }`}
                  onClick={() => onAnswer(question.id, option.value, question)}
                >
                  <CardContent className="p-4">
                    <div className="flex items-start gap-2">
                      {option.icon && <span className="text-2xl">{option.icon}</span>}
                      <div className="flex-1">
                        <div className="font-semibold text-sm">{option.label}</div>
                        {option.description && (
                          <div className="text-xs text-muted-foreground mt-1">{option.description}</div>
                        )}
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}

          {question.type === 'multiple_choice' && (
            <div className="space-y-2">
              {question.options?.map((option) => (
                <div
                  key={option.value}
                  className="flex items-start gap-3 p-3 border rounded cursor-pointer hover:bg-accent"
                  onClick={() => {
                    const currentAnswers = answers[question.id] || [];
                    const newAnswers = currentAnswers.includes(option.value)
                      ? currentAnswers.filter((v: string) => v !== option.value)
                      : [...currentAnswers, option.value];
                    onAnswer(question.id, newAnswers, question);
                  }}
                >
                  <Checkbox checked={(answers[question.id] || []).includes(option.value)} />
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      {option.icon && <span>{option.icon}</span>}
                      <Label className="font-medium cursor-pointer">{option.label}</Label>
                    </div>
                    {option.description && (
                      <div className="text-xs text-muted-foreground mt-1">{option.description}</div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}

          {question.type === 'boolean' && (
            <div className="flex items-center gap-3">
              <Button
                variant={answers[question.id] === true ? 'default' : 'outline'}
                onClick={() => onAnswer(question.id, true, question)}
              >
                Yes
              </Button>
              <Button
                variant={answers[question.id] === false ? 'default' : 'outline'}
                onClick={() => onAnswer(question.id, false, question)}
              >
                No
              </Button>
            </div>
          )}
        </CardContent>
      </Card>
    );
  });
}
