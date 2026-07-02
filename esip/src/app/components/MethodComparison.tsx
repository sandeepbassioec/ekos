import { useEffect } from 'react';
import { useEstimationStore, EstimationMethod } from '../store/estimationStore';
import { Card } from './ui/card';
import { 
  BarChart, 
  Bar, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  Legend, 
  ResponsiveContainer,
  RadarChart,
  PolarGrid,
  PolarAngleAxis,
  PolarRadiusAxis,
  Radar
} from 'recharts';
import { TrendingUp, AlertCircle, CheckCircle, Info } from 'lucide-react';

const METHOD_LABELS: Record<EstimationMethod, string> = {
  PERT: 'PERT (3-Point)',
  STORY_POINTS: 'Story Points',
  TSHIRT: 'T-Shirt Sizing',
  FPA: 'Function Point Analysis',
  UCP: 'Use Case Points'
};

const METHOD_COLORS: Record<EstimationMethod, string> = {
  PERT: '#3b82f6',
  STORY_POINTS: '#10b981',
  TSHIRT: '#f59e0b',
  FPA: '#8b5cf6',
  UCP: '#ec4899'
};

export function MethodComparison() {
  const { 
    selectedMethods, 
    methodResults, 
    comparisonAnalysis,
    calculateMethodComparison 
  } = useEstimationStore();

  useEffect(() => {
    if (selectedMethods.length > 1) {
      calculateMethodComparison();
    }
  }, [selectedMethods, calculateMethodComparison]);

  if (selectedMethods.length < 2) {
    return (
      <div className="space-y-4">
        <div className="h-1 bg-gradient-to-r from-purple-400 via-pink-400 to-red-400 rounded"></div>
        <Card className="p-8 bg-gradient-to-br from-blue-50 via-indigo-50 to-purple-50 border-2 border-blue-300 shadow-lg">
          <div className="flex items-start gap-4">
            <div className="w-12 h-12 bg-blue-600 rounded-full flex items-center justify-center flex-shrink-0">
              <Info className="w-6 h-6 text-white" />
            </div>
            <div className="flex-1">
              <h3 className="text-2xl font-bold text-blue-900 mb-3">
                🎯 Method Comparison Feature Available
              </h3>
              <div className="bg-white rounded-lg p-4 mb-4 border-2 border-blue-200">
                <p className="text-base text-gray-700 mb-3">
                  You currently have <strong className="text-blue-600 text-lg">{selectedMethods.length} method</strong> selected: <strong>{selectedMethods[0]}</strong>
                </p>
                <p className="text-base text-gray-700 font-semibold">
                  ✨ To unlock powerful side-by-side comparison with charts and insights:
                </p>
              </div>
              <div className="space-y-3 mb-4">
                <div className="flex items-start gap-3 bg-white rounded-lg p-3 border border-blue-200">
                  <span className="bg-blue-600 text-white rounded-full w-6 h-6 flex items-center justify-center font-bold text-sm flex-shrink-0">1</span>
                  <div>
                    <p className="font-semibold text-gray-900">Go to Step 1: Method Selection</p>
                    <p className="text-sm text-gray-600">Click "Back" button below to navigate</p>
                  </div>
                </div>
                <div className="flex items-start gap-3 bg-white rounded-lg p-3 border border-blue-200">
                  <span className="bg-blue-600 text-white rounded-full w-6 h-6 flex items-center justify-center font-bold text-sm flex-shrink-0">2</span>
                  <div>
                    <p className="font-semibold text-gray-900">Select 2-3 additional methods</p>
                    <p className="text-sm text-gray-600">Try: Story Points, T-Shirt Sizing, or FPA</p>
                  </div>
                </div>
                <div className="flex items-start gap-3 bg-white rounded-lg p-3 border border-blue-200">
                  <span className="bg-blue-600 text-white rounded-full w-6 h-6 flex items-center justify-center font-bold text-sm flex-shrink-0">3</span>
                  <div>
                    <p className="font-semibold text-gray-900">Complete estimates for all methods</p>
                    <p className="text-sm text-gray-600">Fill in values in Step 4 for each method</p>
                  </div>
                </div>
                <div className="flex items-start gap-3 bg-gradient-to-r from-green-50 to-emerald-50 rounded-lg p-3 border-2 border-green-300">
                  <span className="bg-green-600 text-white rounded-full w-6 h-6 flex items-center justify-center font-bold text-sm flex-shrink-0">4</span>
                  <div>
                    <p className="font-semibold text-green-900">Return here to see comparison!</p>
                    <p className="text-sm text-green-700">Charts, variance analysis, and recommendations</p>
                  </div>
                </div>
              </div>
              <div className="bg-purple-100 border-2 border-purple-300 rounded-lg p-4">
                <p className="text-sm font-semibold text-purple-900 mb-2">
                  📊 What You'll Get:
                </p>
                <ul className="text-sm text-purple-800 space-y-1">
                  <li>✅ Side-by-side comparison table</li>
                  <li>✅ Bar & Radar charts</li>
                  <li>✅ Variance analysis & confidence scoring</li>
                  <li>✅ Smart method recommendation</li>
                </ul>
              </div>
            </div>
          </div>
        </Card>
        <div className="h-1 bg-gradient-to-r from-red-400 via-pink-400 to-purple-400 rounded"></div>
      </div>
    );
  }

  if (!comparisonAnalysis || methodResults.length === 0) {
    return (
      <Card className="p-6 bg-yellow-50 border-yellow-200">
        <div className="flex items-start gap-3">
          <AlertCircle className="w-5 h-5 text-yellow-600 mt-0.5" />
          <div>
            <h3 className="font-semibold text-yellow-900 mb-1">Calculating Comparison...</h3>
            <p className="text-sm text-yellow-700">
              Loading method comparison analysis. If this persists, make sure you've filled in estimates for ALL selected methods in Step 4.
            </p>
          </div>
        </div>
      </Card>
    );
  }

  // Check if any method has 0 hours and show warning
  const zeroHourMethods = methodResults.filter(mr => mr.results.totalHours === 0);
  const hasZeroHours = zeroHourMethods.length > 0;

  const { variance, averageHours, minHours, maxHours, recommendedMethod, confidenceLevel } = comparisonAnalysis;
  const variancePercent = (variance / averageHours) * 100;

  // Prepare data for bar chart - filter out 0 hour methods to avoid duplicate keys
  const barChartData = methodResults
    .filter(mr => mr.results.totalHours > 0)
    .map(mr => ({
      method: METHOD_LABELS[mr.method],
      hours: mr.results.totalHours,
      cost: mr.results.totalCost,
      weeks: mr.results.timelineWeeks
    }));

  // Prepare data for radar chart - only include methods with valid data
  const validMethodResults = methodResults.filter(mr => mr.results.totalHours > 0);
  const maxHoursForRadar = Math.max(...validMethodResults.map(mr => mr.results.totalHours));
  const maxCostForRadar = Math.max(...validMethodResults.map(mr => mr.results.totalCost));
  const maxWeeksForRadar = Math.max(...validMethodResults.map(mr => mr.results.timelineWeeks));
  
  const radarChartData = [
    {
      metric: 'Hours',
      ...Object.fromEntries(validMethodResults.map(mr => [
        METHOD_LABELS[mr.method], 
        (mr.results.totalHours / maxHoursForRadar) * 100
      ]))
    },
    {
      metric: 'Cost',
      ...Object.fromEntries(validMethodResults.map(mr => [
        METHOD_LABELS[mr.method], 
        (mr.results.totalCost / maxCostForRadar) * 100
      ]))
    },
    {
      metric: 'Timeline',
      ...Object.fromEntries(validMethodResults.map(mr => [
        METHOD_LABELS[mr.method], 
        (mr.results.timelineWeeks / maxWeeksForRadar) * 100
      ]))
    },
    {
      metric: 'Confidence',
      ...Object.fromEntries(validMethodResults.map(mr => [
        METHOD_LABELS[mr.method], 
        mr.results.confidence
      ]))
    }
  ];

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-xl font-semibold text-gray-900 mb-2 flex items-center gap-2">
          <TrendingUp className="w-5 h-5" />
          Method Comparison Analysis
        </h3>
        <p className="text-sm text-gray-600">
          Comparing {selectedMethods.length} estimation methods to help you choose the best approach
        </p>
      </div>

      {/* Summary Cards */}
      <div className="grid md:grid-cols-4 gap-4">
        <Card className="p-4 bg-gradient-to-br from-blue-50 to-indigo-50 border-blue-200">
          <div className="text-sm text-blue-600 font-semibold mb-1">Average Estimate</div>
          <div className="text-2xl font-bold text-blue-700">{Math.round(averageHours)} hrs</div>
          <div className="text-xs text-blue-600 mt-1">
            Across all methods
          </div>
        </Card>

        <Card className="p-4 bg-gradient-to-br from-green-50 to-emerald-50 border-green-200">
          <div className="text-sm text-green-600 font-semibold mb-1">Range</div>
          <div className="text-2xl font-bold text-green-700">
            {minHours} - {maxHours}
          </div>
          <div className="text-xs text-green-600 mt-1">
            Min to Max hours
          </div>
        </Card>

        <Card className="p-4 bg-gradient-to-br from-purple-50 to-pink-50 border-purple-200">
          <div className="text-sm text-purple-600 font-semibold mb-1">Variance</div>
          <div className="text-2xl font-bold text-purple-700">
            {variancePercent.toFixed(1)}%
          </div>
          <div className="text-xs text-purple-600 mt-1">
            Standard deviation
          </div>
        </Card>

        <Card className={`p-4 border-2 ${
          confidenceLevel === 'HIGH' ? 'bg-gradient-to-br from-green-50 to-emerald-50 border-green-300' :
          confidenceLevel === 'MEDIUM' ? 'bg-gradient-to-br from-yellow-50 to-orange-50 border-yellow-300' :
          'bg-gradient-to-br from-red-50 to-pink-50 border-red-300'
        }`}>
          <div className={`text-sm font-semibold mb-1 ${
            confidenceLevel === 'HIGH' ? 'text-green-600' :
            confidenceLevel === 'MEDIUM' ? 'text-yellow-600' :
            'text-red-600'
          }`}>
            Confidence Level
          </div>
          <div className={`text-2xl font-bold ${
            confidenceLevel === 'HIGH' ? 'text-green-700' :
            confidenceLevel === 'MEDIUM' ? 'text-yellow-700' :
            'text-red-700'
          }`}>
            {confidenceLevel}
          </div>
          <div className={`text-xs mt-1 ${
            confidenceLevel === 'HIGH' ? 'text-green-600' :
            confidenceLevel === 'MEDIUM' ? 'text-yellow-600' :
            'text-red-600'
          }`}>
            {confidenceLevel === 'HIGH' ? 'Low variance' : 
             confidenceLevel === 'MEDIUM' ? 'Moderate variance' : 
             'High variance'}
          </div>
        </Card>
      </div>

      {/* Warning for missing data */}
      {hasZeroHours && (
        <Card className="p-4 bg-orange-50 border-2 border-orange-300">
          <div className="flex items-start gap-3">
            <AlertCircle className="w-5 h-5 text-orange-600 mt-0.5" />
            <div className="flex-1">
              <h4 className="font-semibold text-orange-900 mb-2">⚠️ Missing Estimate Data</h4>
              <p className="text-sm text-orange-700 mb-2">
                The following methods show <strong>0 hours</strong> because estimates haven't been filled in Step 4:
              </p>
              <ul className="list-disc list-inside text-sm text-orange-700 space-y-1">
                {zeroHourMethods.map(zm => (
                  <li key={zm.method}>
                    <strong>{METHOD_LABELS[zm.method]}</strong> - Go to Step 4 and fill in the required fields
                  </li>
                ))}
              </ul>
              <p className="text-xs text-orange-600 mt-2 font-semibold">
                💡 Tip: For FPA, you need to fill in External Inputs, Outputs, Inquiries, Files, etc.
              </p>
            </div>
          </div>
        </Card>
      )}

      {/* Insights */}
      <Card className="p-6">
        <h4 className="font-semibold text-gray-900 mb-3 flex items-center gap-2">
          {confidenceLevel === 'HIGH' ? <CheckCircle className="w-5 h-5 text-green-600" /> : 
           <AlertCircle className="w-5 h-5 text-yellow-600" />}
          Key Insights
        </h4>
        <div className="space-y-2 text-sm">
          {confidenceLevel === 'LOW' && (
            <div className="flex gap-2 p-3 bg-red-50 border border-red-200 rounded-md">
              <AlertCircle className="w-4 h-4 text-red-600 mt-0.5" />
              <div className="text-red-700">
                <strong>High Variance Detected:</strong> Your methods show significant differences 
                ({variancePercent.toFixed(0)}% variance). This suggests uncertainty in scope or method suitability. 
                Consider refining estimates or using PERT for better accuracy.
              </div>
            </div>
          )}
          
          {confidenceLevel === 'MEDIUM' && (
            <div className="flex gap-2 p-3 bg-yellow-50 border border-yellow-200 rounded-md">
              <AlertCircle className="w-4 h-4 text-yellow-600 mt-0.5" />
              <div className="text-yellow-700">
                <strong>Moderate Variance:</strong> Methods show some differences 
                ({variancePercent.toFixed(0)}% variance). Results are reasonably consistent, 
                but consider reviewing outliers.
              </div>
            </div>
          )}

          {confidenceLevel === 'HIGH' && (
            <div className="flex gap-2 p-3 bg-green-50 border border-green-200 rounded-md">
              <CheckCircle className="w-4 h-4 text-green-600 mt-0.5" />
              <div className="text-green-700">
                <strong>High Confidence:</strong> Methods show strong agreement 
                ({variancePercent.toFixed(0)}% variance). Estimates are consistent and reliable.
              </div>
            </div>
          )}

          <div className="p-3 bg-blue-50 border border-blue-200 rounded-md">
            <div className="text-blue-700">
              <strong>Recommended Method:</strong> Based on your data, <strong className="text-blue-900">{METHOD_LABELS[recommendedMethod!]}</strong> provides 
              the most conservative estimate ({methodResults.find(mr => mr.method === recommendedMethod)?.results.totalHours} hours).
              Consider using this for client commitments.
            </div>
          </div>
        </div>
      </Card>

      {/* Comparison Table */}
      <Card className="p-6">
        <h4 className="font-semibold text-gray-900 mb-4">Detailed Comparison</h4>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-200">
                <th className="text-left py-3 px-4 font-semibold text-gray-700">Method</th>
                <th className="text-right py-3 px-4 font-semibold text-gray-700">Total Hours</th>
                <th className="text-right py-3 px-4 font-semibold text-gray-700">Total Cost</th>
                <th className="text-right py-3 px-4 font-semibold text-gray-700">Timeline</th>
                <th className="text-right py-3 px-4 font-semibold text-gray-700">Confidence</th>
                <th className="text-right py-3 px-4 font-semibold text-gray-700">Dev/QA/Hard</th>
              </tr>
            </thead>
            <tbody>
              {methodResults.map((mr) => {
                const isRecommended = mr.method === recommendedMethod;
                const isLowest = mr.results.totalHours === minHours;
                const isHighest = mr.results.totalHours === maxHours;

                return (
                  <tr 
                    key={mr.method} 
                    className={`border-b border-gray-100 ${
                      isRecommended ? 'bg-blue-50' : 'hover:bg-gray-50'
                    }`}
                  >
                    <td className="py-3 px-4">
                      <div className="flex items-center gap-2">
                        <div 
                          className="w-3 h-3 rounded-full" 
                          style={{ backgroundColor: METHOD_COLORS[mr.method] }}
                        />
                        <span className="font-medium">{METHOD_LABELS[mr.method]}</span>
                        {isRecommended && (
                          <span className="text-xs bg-blue-100 text-blue-700 px-2 py-0.5 rounded-full font-semibold">
                            Recommended
                          </span>
                        )}
                      </div>
                    </td>
                    <td className="text-right py-3 px-4 font-semibold">
                      {mr.results.totalHours}
                      {isLowest && <span className="ml-1 text-green-600 text-xs">↓</span>}
                      {isHighest && <span className="ml-1 text-red-600 text-xs">↑</span>}
                    </td>
                    <td className="text-right py-3 px-4">
                      ${mr.results.totalCost.toLocaleString()}
                    </td>
                    <td className="text-right py-3 px-4">
                      {mr.results.timelineWeeks} weeks
                    </td>
                    <td className="text-right py-3 px-4">
                      {mr.results.confidence}%
                    </td>
                    <td className="text-right py-3 px-4 text-xs text-gray-600">
                      {mr.results.breakdown.development}h / {mr.results.breakdown.testing}h / {mr.results.breakdown.hardening}h
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </Card>

      {/* Bar Chart */}
      <Card className="p-6">
        <h4 className="font-semibold text-gray-900 mb-4">Hours Comparison</h4>
        <ResponsiveContainer width="100%" height={300}>
          <BarChart data={barChartData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="method" />
            <YAxis />
            <Tooltip />
            <Legend />
            <Bar dataKey="hours" fill="#3b82f6" name="Total Hours" />
          </BarChart>
        </ResponsiveContainer>
      </Card>

      {/* Radar Chart */}
      {methodResults.length <= 3 && (
        <Card className="p-6">
          <h4 className="font-semibold text-gray-900 mb-4">Multi-Dimensional Comparison</h4>
          <ResponsiveContainer width="100%" height={400}>
            <RadarChart data={radarChartData}>
              <PolarGrid />
              <PolarAngleAxis dataKey="metric" />
              <PolarRadiusAxis angle={90} domain={[0, 100]} />
              {methodResults.map((mr) => (
                <Radar
                  key={mr.method}
                  name={METHOD_LABELS[mr.method]}
                  dataKey={METHOD_LABELS[mr.method]}
                  stroke={METHOD_COLORS[mr.method]}
                  fill={METHOD_COLORS[mr.method]}
                  fillOpacity={0.3}
                />
              ))}
              <Legend />
            </RadarChart>
          </ResponsiveContainer>
        </Card>
      )}
    </div>
  );
}