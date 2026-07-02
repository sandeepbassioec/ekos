import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router';
import { useEstimationStore } from '../../store/estimationStore';
import { Button } from '../../components/ui/button';
import { Card } from '../../components/ui/card';
import { Download, FileSpreadsheet, Presentation, FileText, ArrowLeft, CheckCircle } from 'lucide-react';
import { generateDynamicExcelFile } from '../../utils/dynamicExcelGenerator';
import { generateDynamicPowerPointFile } from '../../utils/dynamicPptxGenerator';
import { toast } from 'sonner';

export function GenerateDeliverables() {
  const navigate = useNavigate();
  const { results, project, screens, riskFactors, contingency, selectedMethods, setCurrentStep } = useEstimationStore();
  const [isGenerating, setIsGenerating] = useState(false);

  // Set current step when component mounts
  useEffect(() => {
    setCurrentStep(9);
  }, [setCurrentStep]);

  const handleDownloadExcel = async () => {
    if (!results) {
      toast.error('No estimation results available');
      return;
    }

    setIsGenerating(true);
    try {
      const estimationData = {
        project,
        screens,
        results,
        riskFactors,
        contingency,
        selectedMethods,
      };

      await generateDynamicExcelFile(estimationData);
      
      toast.success('Excel file generated successfully!', {
        description: `${project.name} estimation downloaded`,
      });
    } catch (error) {
      console.error('Error generating Excel:', error);
      toast.error('Failed to generate Excel file', {
        description: 'Please try again or contact support',
      });
    } finally {
      setIsGenerating(false);
    }
  };

  const handleDownloadPPT = async () => {
    if (!results) {
      toast.error('No estimation results available');
      return;
    }

    setIsGenerating(true);
    try {
      const estimationData = {
        project,
        screens,
        results,
        riskFactors,
        contingency,
        selectedMethods,
      };

      await generateDynamicPowerPointFile(estimationData);
      
      toast.success('PowerPoint file generated successfully!', {
        description: `${project.name} presentation downloaded`,
      });
    } catch (error) {
      console.error('Error generating PowerPoint:', error);
      toast.error('Failed to generate PowerPoint file', {
        description: 'Please try again or contact support',
      });
    } finally {
      setIsGenerating(false);
    }
  };

  const handleBack = () => {
    setCurrentStep(8);
    navigate('/wizard/step/8');
  };

  const handleFinish = () => {
    navigate('/');
  };

  if (!results) {
    return <div>No results to generate</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold text-gray-900 mb-2">Generate Deliverables</h2>
        <p className="text-gray-600">Download professional estimation documents</p>
      </div>

      {/* Success Message */}
      <div className="bg-green-50 border border-green-200 rounded-lg p-6 text-center">
        <CheckCircle className="w-16 h-16 text-green-600 mx-auto mb-3" />
        <h3 className="text-xl font-bold text-green-900 mb-2">Estimation Complete! ✅</h3>
        <p className="text-green-700">
          Your {project.name} estimation is ready to download
        </p>
      </div>

      {/* Summary */}
      <Card className="p-6 bg-gradient-to-r from-blue-50 to-indigo-50">
        <h3 className="font-semibold text-gray-900 mb-4">Estimation Summary</h3>
        <div className="grid md:grid-cols-2 gap-4 text-sm">
          <div>
            <span className="text-gray-700">Project:</span>
            <p className="font-bold text-gray-900">{project.name}</p>
          </div>
          <div>
            <span className="text-gray-700">Total Effort:</span>
            <p className="font-bold text-gray-900">{results.totalHours} hours</p>
          </div>
          <div>
            <span className="text-gray-700">Total Cost:</span>
            <p className="font-bold text-gray-900">${results.totalCost.toLocaleString()}</p>
          </div>
          <div>
            <span className="text-gray-700">Timeline:</span>
            <p className="font-bold text-gray-900">{results.timelineWeeks} weeks</p>
          </div>
          <div>
            <span className="text-gray-700">Screens/Features:</span>
            <p className="font-bold text-gray-900">{screens.length} items</p>
          </div>
          <div>
            <span className="text-gray-700">Confidence:</span>
            <p className="font-bold text-gray-900">{results.confidence}%</p>
          </div>
        </div>
      </Card>

      {/* Download Options */}
      <div className="grid md:grid-cols-3 gap-4">
        <Button
          onClick={handleDownloadExcel}
          disabled={isGenerating}
          className="h-auto py-6 flex-col gap-2 bg-green-600 hover:bg-green-700"
        >
          <FileSpreadsheet className="w-8 h-8" />
          <span className="font-semibold">Excel (.xlsx)</span>
          <span className="text-xs opacity-90">With formulas & WBS</span>
        </Button>

        <Button
          onClick={handleDownloadPPT}
          disabled={isGenerating}
          className="h-auto py-6 flex-col gap-2 bg-orange-600 hover:bg-orange-700"
        >
          <Presentation className="w-8 h-8" />
          <span className="font-semibold">PowerPoint (.pptx)</span>
          <span className="text-xs opacity-90">CXO presentation</span>
        </Button>

        <Button
          disabled
          className="h-auto py-6 flex-col gap-2 bg-gray-400"
        >
          <FileText className="w-8 h-8" />
          <span className="font-semibold">PDF Report</span>
          <span className="text-xs opacity-90">Coming soon</span>
        </Button>
      </div>

      {isGenerating && (
        <div className="text-center text-gray-600 animate-pulse">
          Generating files... Please wait
        </div>
      )}

      {/* Actions */}
      <div className="flex justify-between pt-6 border-t border-gray-200">
        <Button variant="outline" onClick={handleBack}>
          <ArrowLeft className="w-4 h-4 mr-2" />
          Back
        </Button>
        <Button onClick={handleFinish} className="bg-blue-600 hover:bg-blue-700">
          <CheckCircle className="w-4 h-4 mr-2" />
          Finish & Return Home
        </Button>
      </div>
    </div>
  );
}