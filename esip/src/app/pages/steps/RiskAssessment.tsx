import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router';
import { useEstimationStore, RiskFactor } from '../../store/estimationStore';
import { Button } from '../../components/ui/button';
import { Label } from '../../components/ui/label';
import { Input } from '../../components/ui/input';
import { Card } from '../../components/ui/card';
import { ArrowRight, ArrowLeft, Plus, Trash2, AlertTriangle, CheckCircle2 } from 'lucide-react';

// Pre-defined common risks in software development
const PREDEFINED_RISKS: Array<{ name: string; category: string; defaultImpact: 'LOW' | 'MEDIUM' | 'HIGH' }> = [
  // Technical Risks
  { name: 'New/Unfamiliar Technology Stack', category: 'Technical', defaultImpact: 'HIGH' },
  { name: 'Complex Third-party API Integration', category: 'Technical', defaultImpact: 'MEDIUM' },
  { name: 'Legacy System Integration', category: 'Technical', defaultImpact: 'HIGH' },
  { name: 'Performance & Scalability Requirements', category: 'Technical', defaultImpact: 'MEDIUM' },
  { name: 'Security & Compliance (GDPR, SOC2, etc.)', category: 'Technical', defaultImpact: 'HIGH' },
  { name: 'Cross-browser/Cross-platform Compatibility', category: 'Technical', defaultImpact: 'LOW' },
  { name: 'Real-time Data Synchronization', category: 'Technical', defaultImpact: 'MEDIUM' },
  { name: 'Complex Data Migration', category: 'Technical', defaultImpact: 'HIGH' },
  
  // Team & Resource Risks
  { name: 'Team Ramp-up/Learning Curve', category: 'Team', defaultImpact: 'MEDIUM' },
  { name: 'Limited Team Availability', category: 'Team', defaultImpact: 'HIGH' },
  { name: 'Key Person Dependency', category: 'Team', defaultImpact: 'MEDIUM' },
  { name: 'Distributed/Remote Team Coordination', category: 'Team', defaultImpact: 'LOW' },
  { name: 'Lack of Domain Expertise', category: 'Team', defaultImpact: 'MEDIUM' },
  
  // Project & Process Risks
  { name: 'Unclear/Changing Requirements', category: 'Project', defaultImpact: 'HIGH' },
  { name: 'Tight Deadlines/Aggressive Timeline', category: 'Project', defaultImpact: 'HIGH' },
  { name: 'Multiple Stakeholder Dependencies', category: 'Project', defaultImpact: 'MEDIUM' },
  { name: 'Inadequate Testing Resources', category: 'Project', defaultImpact: 'MEDIUM' },
  { name: 'Scope Creep Likelihood', category: 'Project', defaultImpact: 'MEDIUM' },
  
  // External Risks
  { name: 'Third-party Vendor Dependencies', category: 'External', defaultImpact: 'MEDIUM' },
  { name: 'Regulatory/Compliance Changes', category: 'External', defaultImpact: 'LOW' },
  { name: 'Market/Business Model Uncertainty', category: 'External', defaultImpact: 'LOW' },
];

export function RiskAssessment() {
  const navigate = useNavigate();
  const { 
    contingency, 
    setContingency, 
    riskFactors, 
    addRiskFactor, 
    removeRiskFactor, 
    setCurrentStep 
  } = useEstimationStore();

  const [showCustomRiskForm, setShowCustomRiskForm] = useState(false);
  const [customRisk, setCustomRisk] = useState({
    name: '',
    impact: 'MEDIUM' as 'LOW' | 'MEDIUM' | 'HIGH',
  });
  const [selectedCategory, setSelectedCategory] = useState<string>('All');

  // Get unique categories
  const categories = ['All', ...Array.from(new Set(PREDEFINED_RISKS.map(r => r.category)))];

  // Filter risks by category
  const filteredPredefinedRisks = selectedCategory === 'All' 
    ? PREDEFINED_RISKS 
    : PREDEFINED_RISKS.filter(r => r.category === selectedCategory);

  // Check if a predefined risk is selected
  const isRiskSelected = (riskName: string) => {
    return riskFactors.some(r => r.name === riskName);
  };

  // Toggle predefined risk
  const togglePredefinedRisk = (risk: typeof PREDEFINED_RISKS[0]) => {
    if (isRiskSelected(risk.name)) {
      // Remove risk
      const index = riskFactors.findIndex(r => r.name === risk.name);
      if (index !== -1) removeRiskFactor(index);
    } else {
      // Add risk
      addRiskFactor({
        name: risk.name,
        impact: risk.defaultImpact,
        probability: risk.defaultImpact === 'HIGH' ? 70 : risk.defaultImpact === 'MEDIUM' ? 50 : 30,
      });
    }
  };

  // Add custom risk
  const handleAddCustomRisk = () => {
    if (!customRisk.name.trim()) {
      alert('Please enter a risk name');
      return;
    }

    addRiskFactor({
      name: customRisk.name,
      impact: customRisk.impact,
      probability: customRisk.impact === 'HIGH' ? 70 : customRisk.impact === 'MEDIUM' ? 50 : 30,
    });

    setCustomRisk({ name: '', impact: 'MEDIUM' });
    setShowCustomRiskForm(false);
  };

  // Auto-calculate contingency based on selected risks
  const calculateSuggestedContingency = () => {
    if (riskFactors.length === 0) return 10;
    
    // Enhanced calculation using probability × impact weighting
    const impactWeights = { LOW: 1, MEDIUM: 2, HIGH: 3 };
    
    // Calculate total risk score (sum of impact × probability)
    const totalRiskScore = riskFactors.reduce((sum, risk) => {
      const impactWeight = impactWeights[risk.impact];
      const probabilityWeight = risk.probability / 100; // Convert to decimal
      return sum + (impactWeight * probabilityWeight);
    }, 0);

    // Risk scoring guidelines:
    // 0.0-1.0 = Low risk (5-10% contingency)
    // 1.0-2.0 = Medium risk (10-20% contingency)
    // 2.0-3.0 = High risk (20-30% contingency)
    // 3.0+ = Very high risk (30-50% contingency)
    
    let baseContingency = 10;
    if (totalRiskScore >= 3.0) {
      baseContingency = 30 + Math.min(20, Math.floor(totalRiskScore - 3) * 5);
    } else if (totalRiskScore >= 2.0) {
      baseContingency = 20 + Math.floor((totalRiskScore - 2) * 10);
    } else if (totalRiskScore >= 1.0) {
      baseContingency = 10 + Math.floor((totalRiskScore - 1) * 10);
    } else {
      baseContingency = 5 + Math.floor(totalRiskScore * 5);
    }
    
    return Math.min(50, Math.max(5, baseContingency));
  };

  // Calculate total risk score for display
  const calculateTotalRiskScore = () => {
    const impactWeights = { LOW: 1, MEDIUM: 2, HIGH: 3 };
    return riskFactors.reduce((sum, risk) => {
      const impactWeight = impactWeights[risk.impact];
      const probabilityWeight = risk.probability / 100;
      return sum + (impactWeight * probabilityWeight);
    }, 0);
  };

  const totalRiskScore = calculateTotalRiskScore();
  const suggestedContingency = calculateSuggestedContingency();

  // Get risk score for individual risk
  const getRiskScore = (risk: RiskFactor) => {
    const impactWeights = { LOW: 1, MEDIUM: 2, HIGH: 3 };
    return (impactWeights[risk.impact] * risk.probability) / 100;
  };

  // Get risk level color based on score
  const getRiskScoreColor = (score: number) => {
    if (score >= 2.0) return 'text-red-700 bg-red-100 border-red-300';
    if (score >= 1.0) return 'text-orange-700 bg-orange-100 border-orange-300';
    if (score >= 0.5) return 'text-yellow-700 bg-yellow-100 border-yellow-300';
    return 'text-green-700 bg-green-100 border-green-300';
  };

  // Apply suggested contingency
  const applySuggestedContingency = () => {
    setContingency(suggestedContingency);
  };

  const getImpactBadge = (impact: 'LOW' | 'MEDIUM' | 'HIGH') => {
    const styles = {
      LOW: 'bg-green-100 text-green-700 border-green-300',
      MEDIUM: 'bg-yellow-100 text-yellow-700 border-yellow-300',
      HIGH: 'bg-red-100 text-red-700 border-red-300',
    };
    return styles[impact];
  };

  const getCategoryIcon = (category: string) => {
    const icons: Record<string, string> = {
      Technical: '⚙️',
      Team: '👥',
      Project: '📋',
      External: '🌐',
    };
    return icons[category] || '📌';
  };

  const handleNext = () => {
    setCurrentStep(7);
    navigate('/wizard/step/7');
  };

  const handleBack = () => {
    setCurrentStep(5);
    navigate('/wizard/step/5');
  };

  // Set current step when component mounts
  useEffect(() => {
    setCurrentStep(6);
  }, [setCurrentStep]);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-3xl font-bold text-gray-900 mb-2">Risk Assessment & Contingency</h2>
        <p className="text-gray-600">
          Identify project risks and calculate appropriate contingency buffer
        </p>
      </div>

      {/* Stats Summary */}
      <div className="grid grid-cols-4 gap-4">
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <div className="text-sm text-blue-700 font-medium">Risks Identified</div>
          <div className="text-3xl font-bold text-blue-900">{riskFactors.length}</div>
        </div>
        <div className="bg-orange-50 border border-orange-200 rounded-lg p-4">
          <div className="text-sm text-orange-700 font-medium">Total Risk Score</div>
          <div className="text-3xl font-bold text-orange-900">{totalRiskScore.toFixed(2)}</div>
        </div>
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <div className="text-sm text-red-700 font-medium">High Impact</div>
          <div className="text-3xl font-bold text-red-900">
            {riskFactors.filter(r => r.impact === 'HIGH').length}
          </div>
        </div>
        <div className="bg-purple-50 border border-purple-200 rounded-lg p-4">
          <div className="text-sm text-purple-700 font-medium">Suggested Buffer</div>
          <div className="text-3xl font-bold text-purple-900">{suggestedContingency}%</div>
        </div>
      </div>

      {/* Current Contingency Setting */}
      <Card className="p-6 border-2 border-purple-300 bg-purple-50">
        <div className="flex items-start justify-between mb-4">
          <div>
            <h3 className="font-semibold text-gray-900 text-lg mb-1">
              Contingency Buffer: <span className="text-purple-600">{contingency}%</span>
            </h3>
            <p className="text-sm text-gray-600">
              Based on {riskFactors.length} identified risk{riskFactors.length !== 1 ? 's' : ''}
            </p>
          </div>
          {suggestedContingency !== contingency && (
            <Button
              size="sm"
              onClick={applySuggestedContingency}
              className="bg-purple-600 hover:bg-purple-700"
            >
              Apply Suggested {suggestedContingency}%
            </Button>
          )}
        </div>

        {/* Manual slider */}
        <div className="space-y-2">
          <input
            type="range"
            min="0"
            max="50"
            step="5"
            value={contingency}
            onChange={(e) => setContingency(Number(e.target.value))}
            className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-purple-600"
          />
          <div className="flex justify-between text-xs text-gray-600">
            <span>0%</span>
            <span>10%</span>
            <span>20%</span>
            <span>30%</span>
            <span>40%</span>
            <span>50%</span>
          </div>
        </div>
      </Card>

      {/* Risk Selection */}
      <div>
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-xl font-semibold text-gray-900">Select Project Risks</h3>
          <div className="flex items-center gap-2">
            {/* Category Filter */}
            <select
              value={selectedCategory}
              onChange={(e) => setSelectedCategory(e.target.value)}
              className="px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              {categories.map(cat => (
                <option key={cat} value={cat}>
                  {cat === 'All' ? 'All Categories' : `${getCategoryIcon(cat)} ${cat}`}
                </option>
              ))}
            </select>
            <Button
              size="sm"
              onClick={() => setShowCustomRiskForm(!showCustomRiskForm)}
              className="bg-blue-600 hover:bg-blue-700"
            >
              <Plus className="w-4 h-4 mr-2" />
              Add Custom Risk
            </Button>
          </div>
        </div>

        {/* Custom Risk Form */}
        {showCustomRiskForm && (
          <Card className="p-4 mb-4 border-2 border-blue-300 bg-blue-50">
            <h4 className="font-semibold text-gray-900 mb-3">Add Custom Risk</h4>
            <div className="grid md:grid-cols-2 gap-3">
              <div className="md:col-span-1">
                <Label htmlFor="customRiskName">Risk Name</Label>
                <Input
                  id="customRiskName"
                  value={customRisk.name}
                  onChange={(e) => setCustomRisk({ ...customRisk, name: e.target.value })}
                  placeholder="e.g., Payment gateway integration delays"
                  className="mt-1"
                  autoFocus
                />
              </div>
              <div>
                <Label htmlFor="customRiskImpact">Impact Level</Label>
                <select
                  id="customRiskImpact"
                  value={customRisk.impact}
                  onChange={(e) => setCustomRisk({ ...customRisk, impact: e.target.value as any })}
                  className="mt-1 w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="LOW">🟢 Low Impact</option>
                  <option value="MEDIUM">🟡 Medium Impact</option>
                  <option value="HIGH">🔴 High Impact</option>
                </select>
              </div>
            </div>
            <div className="flex gap-2 mt-3">
              <Button onClick={handleAddCustomRisk} size="sm" className="bg-blue-600 hover:bg-blue-700">
                Add Risk
              </Button>
              <Button variant="outline" size="sm" onClick={() => setShowCustomRiskForm(false)}>
                Cancel
              </Button>
            </div>
          </Card>
        )}

        {/* Predefined Risks Grid */}
        <div className="grid md:grid-cols-2 gap-3">
          {filteredPredefinedRisks.map((risk, index) => {
            const isSelected = isRiskSelected(risk.name);
            return (
              <button
                key={index}
                onClick={() => togglePredefinedRisk(risk)}
                className={`text-left p-4 rounded-lg border-2 transition-all ${
                  isSelected
                    ? 'border-blue-500 bg-blue-50 shadow-md'
                    : 'border-gray-200 bg-white hover:border-gray-300 hover:shadow-sm'
                }`}
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      {isSelected ? (
                        <CheckCircle2 className="w-5 h-5 text-blue-600 flex-shrink-0" />
                      ) : (
                        <div className="w-5 h-5 rounded border-2 border-gray-300 flex-shrink-0"></div>
                      )}
                      <span className="text-sm font-medium text-gray-900">{risk.name}</span>
                    </div>
                    <div className="ml-7 flex items-center gap-2">
                      <span className="text-xs text-gray-500">{getCategoryIcon(risk.category)} {risk.category}</span>
                      <span className={`text-xs px-2 py-0.5 rounded border ${getImpactBadge(risk.defaultImpact)}`}>
                        {risk.defaultImpact}
                      </span>
                    </div>
                  </div>
                </div>
              </button>
            );
          })}
        </div>
      </div>

      {/* Selected Risks Summary */}
      {riskFactors.length > 0 && (
        <Card className="p-6 bg-yellow-50 border-2 border-yellow-300">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-2">
              <AlertTriangle className="w-5 h-5 text-yellow-700" />
              <h3 className="font-semibold text-gray-900 text-lg">
                Selected Risks ({riskFactors.length})
              </h3>
            </div>
            <div className="text-sm text-gray-700">
              Total Risk Score: <span className="font-bold text-orange-700">{totalRiskScore.toFixed(2)}</span>
            </div>
          </div>
          <div className="space-y-2">
            {riskFactors.map((risk, index) => {
              const riskScore = getRiskScore(risk);
              return (
                <div
                  key={index}
                  className="flex items-center justify-between bg-white p-3 rounded-lg border border-yellow-200"
                >
                  <div className="flex-1">
                    <div className="font-medium text-gray-900 mb-1">{risk.name}</div>
                    <div className="text-xs text-gray-600">
                      Probability: {risk.probability}% | Risk Score: {riskScore.toFixed(2)}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className={`text-xs px-2 py-1 rounded border font-semibold ${getRiskScoreColor(riskScore)}`}>
                      {riskScore.toFixed(2)}
                    </span>
                    <span className={`text-xs px-2 py-1 rounded border ${getImpactBadge(risk.impact)}`}>
                      {risk.impact}
                    </span>
                    <button
                      onClick={() => removeRiskFactor(index)}
                      className="p-1.5 hover:bg-red-100 rounded text-red-600"
                      title="Remove risk"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
          
          {/* Risk Score Legend */}
          <div className="mt-4 pt-4 border-t border-yellow-300">
            <div className="text-xs text-gray-700 font-medium mb-2">Risk Score Calculation:</div>
            <div className="text-xs text-gray-600">
              Score = Impact Weight (LOW=1, MEDIUM=2, HIGH=3) × Probability (%)
            </div>
          </div>
        </Card>
      )}

      {/* Guidance */}
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
        <h3 className="font-semibold text-blue-900 mb-2">💡 Contingency Guidelines:</h3>
        <ul className="text-sm text-blue-800 space-y-1">
          <li>• <strong>0-10%:</strong> Mature tech stack, experienced team, clear requirements, minimal risks</li>
          <li>• <strong>10-20%:</strong> Standard projects with moderate complexity (recommended default: 15%)</li>
          <li>• <strong>20-30%:</strong> New technology, complex integrations, multiple high-impact risks</li>
          <li>• <strong>30-50%:</strong> High uncertainty, R&D heavy, changing requirements, many unknowns</li>
        </ul>
      </div>

      {/* Navigation */}
      <div className="flex justify-between pt-6 border-t border-gray-200">
        <Button variant="outline" onClick={handleBack}>
          <ArrowLeft className="w-4 h-4 mr-2" />
          Back
        </Button>
        <Button onClick={handleNext} className="bg-blue-600 hover:bg-blue-700">
          Review & Calculate ({riskFactors.length} risks identified)
          <ArrowRight className="w-4 h-4 ml-2" />
        </Button>
      </div>
    </div>
  );
}