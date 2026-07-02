import { Check, Circle } from 'lucide-react';
import { useEstimationStore } from '../store/estimationStore';
import {
  validateMethodSelection,
  validateProjectSetup,
  validateScopeDefinition,
  validateMethodDetails,
} from '../utils/stepValidation';

interface Step {
  number: number;
  name: string;
  shortName: string;
}

const STEPS: Step[] = [
  { number: 1, name: 'Method Selection', shortName: 'Methods' },
  { number: 2, name: 'Project Setup', shortName: 'Setup' },
  { number: 3, name: 'Scope Definition', shortName: 'Scope' },
  { number: 4, name: 'Method Details', shortName: 'Details' },
  { number: 5, name: 'Complexity Weighting', shortName: 'Complexity' },
  { number: 6, name: 'Risk Assessment', shortName: 'Risks' },
  { number: 7, name: 'Review & Calculate', shortName: 'Review' },
  { number: 8, name: 'Capacity Planning', shortName: 'Capacity' },
  { number: 9, name: 'Generate Deliverables', shortName: 'Export' },
];

export function StepIndicator() {
  const { currentStep, selectedMethods, project, screens } = useEstimationStore();

  // Check completion status for each step
  const getStepStatus = (stepNumber: number): 'complete' | 'active' | 'incomplete' => {
    if (stepNumber < currentStep) {
      // Check if step is actually complete
      if (stepNumber === 1) {
        return validateMethodSelection(selectedMethods).isComplete ? 'complete' : 'incomplete';
      }
      if (stepNumber === 2) {
        return validateProjectSetup(project).isComplete ? 'complete' : 'incomplete';
      }
      if (stepNumber === 3) {
        return validateScopeDefinition(screens).isComplete ? 'complete' : 'incomplete';
      }
      if (stepNumber === 4) {
        return validateMethodDetails(screens, selectedMethods).isComplete ? 'complete' : 'incomplete';
      }
      return 'complete';
    }
    if (stepNumber === currentStep) {
      return 'active';
    }
    return 'incomplete';
  };

  return (
    <div className="bg-white border-b border-gray-200 shadow-sm">
      <div className="max-w-7xl mx-auto px-6 py-4">
        {/* Desktop View */}
        <div className="hidden md:flex items-center justify-between">
          {STEPS.map((step, index) => {
            const status = getStepStatus(step.number);
            const isLast = index === STEPS.length - 1;

            return (
              <div key={step.number} className="flex items-center flex-1">
                {/* Step Circle */}
                <div className="flex items-center">
                  <div
                    className={`flex items-center justify-center w-10 h-10 rounded-full border-2 transition-all ${
                      status === 'complete'
                        ? 'bg-green-600 border-green-600 text-white'
                        : status === 'active'
                        ? 'bg-blue-600 border-blue-600 text-white'
                        : 'bg-white border-gray-300 text-gray-400'
                    }`}
                  >
                    {status === 'complete' ? (
                      <Check className="w-5 h-5" />
                    ) : (
                      <span className="text-sm font-semibold">{step.number}</span>
                    )}
                  </div>
                  <div className="ml-3">
                    <div
                      className={`text-sm font-semibold ${
                        status === 'active' ? 'text-blue-600' : status === 'complete' ? 'text-green-600' : 'text-gray-400'
                      }`}
                    >
                      {step.name}
                    </div>
                    {status === 'active' && (
                      <div className="text-xs text-gray-500">Current step</div>
                    )}
                  </div>
                </div>

                {/* Connector Line */}
                {!isLast && (
                  <div
                    className={`flex-1 h-0.5 mx-4 transition-all ${
                      status === 'complete' ? 'bg-green-600' : 'bg-gray-300'
                    }`}
                  />
                )}
              </div>
            );
          })}
        </div>

        {/* Mobile View */}
        <div className="md:hidden">
          <div className="flex items-center justify-between mb-2">
            <div className="text-sm font-semibold text-gray-900">
              Step {currentStep} of {STEPS.length}
            </div>
            <div className="text-sm text-gray-600">
              {STEPS.find(s => s.number === currentStep)?.name}
            </div>
          </div>
          <div className="w-full bg-gray-200 rounded-full h-2">
            <div
              className="bg-blue-600 h-2 rounded-full transition-all"
              style={{ width: `${(currentStep / STEPS.length) * 100}%` }}
            />
          </div>
        </div>
      </div>
    </div>
  );
}