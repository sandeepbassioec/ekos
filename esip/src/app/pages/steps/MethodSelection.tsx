import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router';
import { useEstimationStore, EstimationMethod } from '../../store/estimationStore';
import { Button } from '../../components/ui/button';
import { Card } from '../../components/ui/card';
import { ArrowRight, Info, CheckCircle2 } from 'lucide-react';

interface MethodInfo {
  id: EstimationMethod;
  name: string;
  description: string;
  bestFor: string;
  complexity: 'Easy' | 'Medium' | 'Advanced';
  icon: string;
  pros: string[];
  cons: string[];
}

const methods: MethodInfo[] = [
  {
    id: 'PERT',
    name: 'PERT (Three-Point Estimation)',
    description: 'Uses Optimistic, Most Likely, and Pessimistic estimates to calculate expected time',
    bestFor: 'Projects with uncertainty, general software development',
    complexity: 'Medium',
    icon: '📊',
    pros: ['Industry standard', 'Accounts for uncertainty', 'Simple to understand'],
    cons: ['Requires expert judgment', 'Time-consuming for large projects'],
  },
  {
    id: 'STORY_POINTS',
    name: 'Story Points (Agile)',
    description: 'Relative sizing using Fibonacci sequence based on complexity and effort',
    bestFor: 'Agile/Scrum teams, sprint planning',
    complexity: 'Easy',
    icon: '🎯',
    pros: ['Fast', 'Team-based', 'Works well for sprints'],
    cons: ['Needs historical velocity', 'Not time-based'],
  },
  {
    id: 'TSHIRT',
    name: 'T-Shirt Sizing',
    description: 'Quick estimates using sizes: XS, S, M, L, XL, XXL',
    bestFor: 'Early-stage estimates, backlogs, roadmaps',
    complexity: 'Easy',
    icon: '👕',
    pros: ['Very fast', 'Non-technical friendly', 'Good for rough estimates'],
    cons: ['Less precise', 'Hard to defend'],
  },
  {
    id: 'FPA',
    name: 'Function Point Analysis',
    description: 'Count External Inputs, Outputs, Inquiries, Files, and Interfaces',
    bestFor: 'Data-heavy applications, enterprise software',
    complexity: 'Advanced',
    icon: '🔢',
    pros: ['Very precise', 'Language-independent', 'ISO standard'],
    cons: ['Complex', 'Time-consuming', 'Requires training'],
  },
  {
    id: 'UCP',
    name: 'Use Case Points',
    description: 'Estimate based on use cases, actors, and technical/environmental factors',
    bestFor: 'Use case driven projects, object-oriented systems',
    complexity: 'Medium',
    icon: '🎭',
    pros: ['Good for OO projects', 'Considers complexity', 'Structured approach'],
    cons: ['Requires use case documentation', 'Many factors to assess'],
  },
];

export function MethodSelection() {
  const navigate = useNavigate();
  const { selectedMethods, setSelectedMethods, setCurrentStep } = useEstimationStore();
  const [localSelection, setLocalSelection] = useState<EstimationMethod[]>(selectedMethods);

  const toggleMethod = (methodId: EstimationMethod) => {
    if (localSelection.includes(methodId)) {
      setLocalSelection(localSelection.filter((m) => m !== methodId));
    } else {
      setLocalSelection([...localSelection, methodId]);
    }
  };

  const handleNext = () => {
    if (localSelection.length === 0) {
      alert('Please select at least one estimation method');
      return;
    }
    setSelectedMethods(localSelection);
    setCurrentStep(2);
    navigate('/wizard/step/2');
  };

  // Set current step when component mounts
  useEffect(() => {
    setCurrentStep(1);
  }, [setCurrentStep]);

  useEffect(() => {
    setLocalSelection(selectedMethods);
  }, [selectedMethods]);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-3xl font-bold text-gray-900 mb-2">
          Choose Estimation Method(s)
        </h2>
        <p className="text-gray-600">
          Select 1-3 methods to use for your estimation. You can compare results later.
        </p>
      </div>

      {/* Info Box */}
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 flex items-start gap-3">
        <Info className="w-5 h-5 text-blue-600 mt-0.5" />
        <div className="text-sm text-blue-900">
          <p className="font-semibold mb-1">💡 Pro Tip:</p>
          <p>
            Selecting multiple methods allows you to compare results and increase confidence.
            For example: <strong>PERT + Story Points</strong> gives both time-based and complexity-based views.
          </p>
        </div>
      </div>

      {/* Method Cards */}
      <div className="grid md:grid-cols-2 gap-4">
        {methods.map((method) => {
          const isSelected = localSelection.includes(method.id);
          const isDisabled = localSelection.length >= 3 && !isSelected;

          return (
            <Card
              key={method.id}
              className={`p-6 cursor-pointer transition-all border-2 ${
                isSelected
                  ? 'border-blue-500 bg-blue-50 shadow-lg'
                  : isDisabled
                  ? 'border-gray-200 bg-gray-50 opacity-50 cursor-not-allowed'
                  : 'border-gray-200 hover:border-blue-300 hover:shadow-md'
              }`}
              onClick={() => !isDisabled && toggleMethod(method.id)}
            >
              <div className="flex items-start justify-between mb-4">
                <div className="flex items-center gap-3">
                  <div className="text-3xl">{method.icon}</div>
                  <div>
                    <h3 className="font-bold text-lg text-gray-900">{method.name}</h3>
                    <div className="flex items-center gap-2 mt-1">
                      <span
                        className={`text-xs px-2 py-0.5 rounded-full ${
                          method.complexity === 'Easy'
                            ? 'bg-green-100 text-green-700'
                            : method.complexity === 'Medium'
                            ? 'bg-yellow-100 text-yellow-700'
                            : 'bg-red-100 text-red-700'
                        }`}
                      >
                        {method.complexity}
                      </span>
                    </div>
                  </div>
                </div>
              </div>

              <p className="text-sm text-gray-600 mb-3">{method.description}</p>

              <div className="mb-3">
                <span className="text-xs font-semibold text-gray-700">Best for:</span>
                <p className="text-xs text-gray-600">{method.bestFor}</p>
              </div>

              <div className="grid grid-cols-2 gap-3 text-xs">
                <div>
                  <span className="font-semibold text-green-700">✅ Pros:</span>
                  <ul className="mt-1 space-y-0.5 text-gray-600">
                    {method.pros.map((pro, idx) => (
                      <li key={idx}>• {pro}</li>
                    ))}
                  </ul>
                </div>
                <div>
                  <span className="font-semibold text-red-700">⚠️ Cons:</span>
                  <ul className="mt-1 space-y-0.5 text-gray-600">
                    {method.cons.map((con, idx) => (
                      <li key={idx}>• {con}</li>
                    ))}
                  </ul>
                </div>
              </div>
            </Card>
          );
        })}
      </div>

      {/* Selected Summary */}
      {localSelection.length > 0 && (
        <div className="bg-green-50 border border-green-200 rounded-lg p-4">
          <div className="flex items-center gap-2 mb-2">
            <CheckCircle2 className="w-5 h-5 text-green-600" />
            <span className="font-semibold text-green-900">
              Selected {localSelection.length} method{localSelection.length > 1 ? 's' : ''}:
            </span>
          </div>
          <div className="flex flex-wrap gap-2">
            {localSelection.map((methodId) => {
              const method = methods.find((m) => m.id === methodId);
              return (
                <div
                  key={methodId}
                  className="bg-white border border-green-300 rounded-lg px-3 py-1 flex items-center gap-2"
                >
                  <span className="text-lg">{method?.icon}</span>
                  <span className="text-sm font-medium text-gray-900">{method?.name}</span>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Navigation */}
      <div className="flex justify-between pt-6 border-t border-gray-200">
        <div></div>
        <Button
          onClick={handleNext}
          disabled={localSelection.length === 0}
          className="bg-blue-600 hover:bg-blue-700"
        >
          Continue to Project Setup
          <ArrowRight className="w-4 h-4 ml-2" />
        </Button>
      </div>
    </div>
  );
}