import { useNavigate } from 'react-router';
import { useEstimationStore } from '../../store/estimationStore';
import { useComplexityStore, ComplexityCategory } from '../../store/complexityStore';
import { Button } from '../../components/ui/button';
import { Card } from '../../components/ui/card';
import { ArrowRight, ArrowLeft, Info, TrendingUp, TrendingDown, Minus } from 'lucide-react';
import { useEffect, useState } from 'react';

const CATEGORY_INFO: Record<ComplexityCategory, { icon: string; color: string; description: string }> = {
  TECHNICAL: {
    icon: '⚙️',
    color: 'blue',
    description: 'Technical implementation challenges and system complexity',
  },
  DOMAIN: {
    icon: '🎯',
    color: 'purple',
    description: 'Business domain knowledge and requirements complexity',
  },
  TEAM: {
    icon: '👥',
    color: 'green',
    description: 'Team capabilities, experience, and collaboration factors',
  },
};

export function ComplexityWeighting() {
  const navigate = useNavigate();
  const { setCurrentStep, screens } = useEstimationStore();
  const {
    factors,
    overallComplexityScore,
    confidenceLevel,
    updateFactor,
    resetFactors,
    calculateComplexityScore,
    toggleFactor,
  } = useComplexityStore();

  const [selectedCategory, setSelectedCategory] = useState<ComplexityCategory | 'ALL'>('ALL');

  // Calculate on mount and set current step
  useEffect(() => {
    setCurrentStep(5);
    calculateComplexityScore();
  }, [setCurrentStep, calculateComplexityScore]);

  const handleNext = () => {
    setCurrentStep(6);
    navigate('/wizard/step/6');
  };

  const handleBack = () => {
    setCurrentStep(4);
    navigate('/wizard/step/4');
  };

  // Filter factors by category
  const filteredFactors = selectedCategory === 'ALL'
    ? factors
    : factors.filter(f => f.category === selectedCategory);

  // Group by category
  const factorsByCategory = {
    TECHNICAL: factors.filter(f => f.category === 'TECHNICAL'),
    DOMAIN: factors.filter(f => f.category === 'DOMAIN'),
    TEAM: factors.filter(f => f.category === 'TEAM'),
  };

  // Calculate category averages
  const categoryAverages = {
    TECHNICAL: factorsByCategory.TECHNICAL.filter(f => f.enabled).reduce((sum, f) => sum + f.weight, 0) / factorsByCategory.TECHNICAL.filter(f => f.enabled).length || 1.0,
    DOMAIN: factorsByCategory.DOMAIN.filter(f => f.enabled).reduce((sum, f) => sum + f.weight, 0) / factorsByCategory.DOMAIN.filter(f => f.enabled).length || 1.0,
    TEAM: factorsByCategory.TEAM.filter(f => f.enabled).reduce((sum, f) => sum + f.weight, 0) / factorsByCategory.TEAM.filter(f => f.enabled).length || 1.0,
  };

  const getImpactIcon = (weight: number) => {
    if (weight > 1.2) return <TrendingUp className="w-4 h-4 text-red-600" />;
    if (weight < 0.8) return <TrendingDown className="w-4 h-4 text-green-600" />;
    return <Minus className="w-4 h-4 text-gray-600" />;
  };

  const getImpactLabel = (weight: number) => {
    if (weight >= 1.5) return 'Very High Impact';
    if (weight >= 1.2) return 'High Impact';
    if (weight >= 0.8) return 'Medium Impact';
    if (weight >= 0.5) return 'Low Impact';
    return 'Minimal Impact';
  };

  const getImpactColor = (weight: number) => {
    if (weight >= 1.5) return 'text-red-700 bg-red-50 border-red-200';
    if (weight >= 1.2) return 'text-orange-700 bg-orange-50 border-orange-200';
    if (weight >= 0.8) return 'text-gray-700 bg-gray-50 border-gray-200';
    if (weight >= 0.5) return 'text-green-700 bg-green-50 border-green-200';
    return 'text-blue-700 bg-blue-50 border-blue-200';
  };

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 80) return 'text-green-700 bg-green-50';
    if (confidence >= 60) return 'text-yellow-700 bg-yellow-50';
    return 'text-red-700 bg-red-50';
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-3xl font-bold text-gray-900 mb-2">Complexity Weighting</h2>
        <p className="text-gray-600">
          Adjust complexity factors to refine your estimates based on project challenges
        </p>
      </div>

      {/* Summary Cards */}
      <div className="grid md:grid-cols-3 gap-4">
        <Card className="p-4 bg-gradient-to-br from-blue-50 to-blue-100 border-blue-200">
          <div className="text-sm font-medium text-blue-900 mb-1">Overall Complexity</div>
          <div className="text-3xl font-bold text-blue-700">
            {overallComplexityScore.toFixed(2)}x
          </div>
          <div className="text-xs text-blue-600 mt-1">
            {overallComplexityScore > 1.2 ? '⬆️ Increases estimates' : overallComplexityScore < 0.8 ? '⬇️ Reduces estimates' : '➡️ Neutral impact'}
          </div>
        </Card>

        <Card className="p-4 bg-gradient-to-br from-green-50 to-green-100 border-green-200">
          <div className="text-sm font-medium text-green-900 mb-1">Confidence Level</div>
          <div className="text-3xl font-bold text-green-700">
            {confidenceLevel}%
          </div>
          <div className="text-xs text-green-600 mt-1">
            {confidenceLevel >= 80 ? '✅ High confidence' : confidenceLevel >= 60 ? '⚠️ Medium confidence' : '❌ Low confidence'}
          </div>
        </Card>

        <Card className="p-4 bg-gradient-to-br from-purple-50 to-purple-100 border-purple-200">
          <div className="text-sm font-medium text-purple-900 mb-1">Active Factors</div>
          <div className="text-3xl font-bold text-purple-700">
            {factors.filter(f => f.enabled).length}/{factors.length}
          </div>
          <div className="text-xs text-purple-600 mt-1">
            {factors.filter(f => f.enabled && f.weight > 1.2).length} high-risk factors
          </div>
        </Card>
      </div>

      {/* Category Tabs */}
      <div className="flex gap-2 border-b border-gray-200">
        <button
          onClick={() => setSelectedCategory('ALL')}
          className={`px-4 py-2 font-medium border-b-2 transition-colors ${
            selectedCategory === 'ALL'
              ? 'border-blue-600 text-blue-600'
              : 'border-transparent text-gray-600 hover:text-gray-900'
          }`}
        >
          All Factors ({factors.length})
        </button>
        {Object.entries(CATEGORY_INFO).map(([category, info]) => (
          <button
            key={category}
            onClick={() => setSelectedCategory(category as ComplexityCategory)}
            className={`px-4 py-2 font-medium border-b-2 transition-colors ${
              selectedCategory === category
                ? `border-${info.color}-600 text-${info.color}-600`
                : 'border-transparent text-gray-600 hover:text-gray-900'
            }`}
          >
            {info.icon} {category.charAt(0) + category.slice(1).toLowerCase()} ({factorsByCategory[category as ComplexityCategory].length})
          </button>
        ))}
      </div>

      {/* Category Description */}
      {selectedCategory !== 'ALL' && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 flex items-start gap-3">
          <Info className="w-5 h-5 text-blue-600 flex-shrink-0 mt-0.5" />
          <div>
            <div className="font-semibold text-blue-900 mb-1">
              {CATEGORY_INFO[selectedCategory].icon} {selectedCategory.charAt(0) + selectedCategory.slice(1).toLowerCase()} Complexity
            </div>
            <div className="text-sm text-blue-700">
              {CATEGORY_INFO[selectedCategory].description}
            </div>
            <div className="text-sm font-semibold text-blue-900 mt-2">
              Average Weight: {categoryAverages[selectedCategory].toFixed(2)}x
            </div>
          </div>
        </div>
      )}

      {/* Complexity Factors */}
      <div className="space-y-3">
        {filteredFactors.map((factor) => (
          <Card key={factor.id} className={`p-4 transition-all ${factor.enabled ? 'border-gray-300' : 'border-gray-200 opacity-60'}`}>
            <div className="flex items-start justify-between gap-4">
              {/* Factor Info */}
              <div className="flex-1">
                <div className="flex items-center gap-2 mb-1">
                  <input
                    type="checkbox"
                    checked={factor.enabled}
                    onChange={() => toggleFactor(factor.id)}
                    className="w-4 h-4 text-blue-600 rounded"
                  />
                  <h4 className="font-semibold text-gray-900">{factor.name}</h4>
                  {getImpactIcon(factor.weight)}
                </div>
                <p className="text-sm text-gray-600 ml-6">{factor.description}</p>
              </div>

              {/* Weight Slider */}
              <div className="w-64">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-medium text-gray-700">Weight:</span>
                  <span className="text-lg font-bold text-gray-900">{factor.weight.toFixed(1)}x</span>
                </div>
                <input
                  type="range"
                  min="0.5"
                  max="2.0"
                  step="0.1"
                  value={factor.weight}
                  onChange={(e) => {
                    updateFactor(factor.id, { weight: Number(e.target.value) });
                    calculateComplexityScore();
                  }}
                  disabled={!factor.enabled}
                  className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-600"
                />
                <div className="flex justify-between text-xs text-gray-500 mt-1">
                  <span>0.5x</span>
                  <span>1.0x</span>
                  <span>2.0x</span>
                </div>
                <div className={`mt-2 text-xs font-medium px-2 py-1 rounded border text-center ${getImpactColor(factor.weight)}`}>
                  {getImpactLabel(factor.weight)}
                </div>
              </div>
            </div>
          </Card>
        ))}
      </div>

      {/* Impact Summary */}
      <Card className="p-6 bg-gradient-to-br from-gray-50 to-gray-100 border-gray-300">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">📊 Complexity Impact Summary</h3>
        
        <div className="grid md:grid-cols-2 gap-6">
          <div>
            <div className="text-sm font-medium text-gray-700 mb-2">Estimates will be multiplied by:</div>
            <div className="text-4xl font-bold text-blue-700 mb-2">
              {overallComplexityScore.toFixed(2)}x
            </div>
            <div className="text-sm text-gray-600">
              {overallComplexityScore > 1.0 
                ? `+${((overallComplexityScore - 1) * 100).toFixed(0)}% increase in effort`
                : `-${((1 - overallComplexityScore) * 100).toFixed(0)}% decrease in effort`
              }
            </div>
          </div>

          <div>
            <div className="text-sm font-medium text-gray-700 mb-2">Confidence in estimates:</div>
            <div className={`inline-flex items-center gap-2 px-4 py-2 rounded-lg ${getConfidenceColor(confidenceLevel)}`}>
              <span className="text-3xl font-bold">{confidenceLevel}%</span>
            </div>
            <div className="text-sm text-gray-600 mt-2">
              {confidenceLevel >= 80 
                ? 'High confidence - well-understood project'
                : confidenceLevel >= 60
                ? 'Medium confidence - some uncertainties'
                : 'Low confidence - high risk and uncertainty'
              }
            </div>
          </div>
        </div>

        <div className="mt-4 pt-4 border-t border-gray-300">
          <Button
            variant="outline"
            size="sm"
            onClick={resetFactors}
            className="text-gray-700 hover:bg-gray-200"
          >
            Reset All to Default (1.0x)
          </Button>
        </div>
      </Card>

      {/* Navigation */}
      <div className="flex justify-between pt-6 border-t border-gray-200">
        <Button variant="outline" onClick={handleBack}>
          <ArrowLeft className="w-4 h-4 mr-2" />
          Back
        </Button>
        <Button onClick={handleNext} className="bg-blue-600 hover:bg-blue-700">
          Continue to Risk Assessment
          <ArrowRight className="w-4 h-4 ml-2" />
        </Button>
      </div>
    </div>
  );
}