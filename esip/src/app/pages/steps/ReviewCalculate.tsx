import { useEffect } from 'react';
import { useNavigate } from 'react-router';
import { useEstimationStore } from '../../store/estimationStore';
import { Button } from '../../components/ui/button';
import { Card } from '../../components/ui/card';
import { MethodComparison } from '../../components/MethodComparison';
import { ArrowRight, ArrowLeft, Calculator } from 'lucide-react';

export function ReviewCalculate() {
  const navigate = useNavigate();
  const { 
    screens, 
    project, 
    contingency, 
    results, 
    selectedMethods,
    calculateResults, 
    calculateMethodComparison,
    setCurrentStep 
  } = useEstimationStore();

  useEffect(() => {
    setCurrentStep(7);
    calculateResults();
    if (selectedMethods.length > 1) {
      calculateMethodComparison();
    }
  }, [setCurrentStep, calculateResults, calculateMethodComparison, selectedMethods]);

  const handleRecalculate = () => {
    calculateResults();
    if (selectedMethods.length > 1) {
      calculateMethodComparison();
    }
  };

  const handleNext = () => {
    setCurrentStep(8);
    navigate('/wizard/step/8');
  };

  const handleBack = () => {
    setCurrentStep(6);
    navigate('/wizard/step/6');
  };

  if (!results) {
    return <div>Calculating...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold text-gray-900 mb-2">Review & Calculate</h2>
        <p className="text-gray-600">Final estimation results based on your inputs</p>
        <div className="mt-3 text-sm text-gray-500">
          Currently using: <strong className="text-blue-600">{selectedMethods.join(', ')}</strong>
          {selectedMethods.length === 1 && (
            <span className="ml-2 text-orange-600 font-semibold">
              (Select 2+ methods in Step 1 to enable comparison)
            </span>
          )}
        </div>
      </div>

      {/* Summary Cards */}
      <div className="grid md:grid-cols-3 gap-4">
        <Card className="p-6 bg-gradient-to-br from-blue-50 to-indigo-50 border-blue-200">
          <div className="text-sm text-blue-600 font-semibold mb-1">Total Effort</div>
          <div className="text-3xl font-bold text-blue-700">{results.totalHours} hrs</div>
          <div className="text-xs text-blue-600 mt-1">
            With {contingency}% contingency
          </div>
        </Card>

        <Card className="p-6 bg-gradient-to-br from-green-50 to-emerald-50 border-green-200">
          <div className="text-sm text-green-600 font-semibold mb-1">Total Cost</div>
          <div className="text-3xl font-bold text-green-700">
            ${results.totalCost.toLocaleString()}
          </div>
          <div className="text-xs text-green-600 mt-1">
            @ ${project.hourlyRate}/hr
          </div>
        </Card>

        <Card className="p-6 bg-gradient-to-br from-purple-50 to-pink-50 border-purple-200">
          <div className="text-sm text-purple-600 font-semibold mb-1">Timeline</div>
          <div className="text-3xl font-bold text-purple-700">{results.timelineWeeks} weeks</div>
          <div className="text-xs text-purple-600 mt-1">
            Team of {project.teamSize}
          </div>
        </Card>
      </div>

      {/* Breakdown */}
      <Card className="p-6">
        <h3 className="font-semibold text-gray-900 mb-4">Effort Breakdown</h3>
        <div className="space-y-3">
          <div className="flex justify-between items-center">
            <span className="text-gray-700">Development</span>
            <span className="font-bold text-gray-900">{results.breakdown.development} hrs</span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-gray-700">Testing</span>
            <span className="font-bold text-gray-900">{results.breakdown.testing} hrs</span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-gray-700">Hardening</span>
            <span className="font-bold text-gray-900">{results.breakdown.hardening} hrs</span>
          </div>
        </div>
      </Card>

      {/* Screens Summary */}
      <Card className="p-6">
        <h3 className="font-semibold text-gray-900 mb-4">Screens/Features ({screens.length})</h3>
        <div className="space-y-2">
          {screens.map((screen) => (
            <div key={screen.id} className="flex justify-between items-center text-sm">
              <span className="text-gray-700">{screen.name}</span>
              <span className="text-gray-600">{screen.complexity} complexity</span>
            </div>
          ))}
        </div>
      </Card>

      <div className="bg-green-50 border border-green-200 rounded-lg p-4 text-center">
        <div className="text-sm text-green-700 mb-2">Confidence Level</div>
        <div className="text-2xl font-bold text-green-800">{results.confidence}%</div>
      </div>

      {/* Method Comparison */}
      <MethodComparison />

      <Button
        onClick={handleRecalculate}
        variant="outline"
        className="w-full"
      >
        <Calculator className="w-4 h-4 mr-2" />
        Recalculate
      </Button>

      <div className="flex justify-between pt-6 border-t border-gray-200">
        <Button variant="outline" onClick={handleBack}>
          <ArrowLeft className="w-4 h-4 mr-2" />
          Back
        </Button>
        <Button onClick={handleNext} className="bg-blue-600 hover:bg-blue-700">
          Capacity Planning
          <ArrowRight className="w-4 h-4 ml-2" />
        </Button>
      </div>
    </div>
  );
}