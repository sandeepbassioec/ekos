import { Routes, Route, useNavigate, Navigate } from 'react-router';
import { useEstimationStore } from '../store/estimationStore';
import { useProjectsStore } from '../store/projectsStore';
import { Button } from '../components/ui/button';
import { ArrowLeft, Save, X, FolderOpen } from 'lucide-react';
import { StepIndicator } from '../components/StepIndicator';
import { toast } from 'sonner';
import { MethodSelectionWrapper } from './steps/MethodSelectionWrapper';
import { ProjectSetup } from './steps/ProjectSetup';
import { ScopeDefinition } from './steps/ScopeDefinition';
import { MethodDetails } from './steps/MethodDetails';
import { ComplexityWeighting } from './steps/ComplexityWeighting';
import { RiskAssessment } from './steps/RiskAssessment';
import { ReviewCalculate } from './steps/ReviewCalculate';
import { CapacityPlanning } from './steps/CapacityPlanning';
import { GenerateDeliverables } from './steps/GenerateDeliverables';
import { useEffect, useState } from 'react';

export function EstimationWizard() {
  const navigate = useNavigate();
  const { 
    saveDraft, 
    lastSaved, 
    selectedMethods, 
    project, 
    screens, 
    contingency, 
    riskFactors, 
    results, 
    capacityPlan, 
    currentProjectId, 
    setCurrentProjectId,
    addScreen 
  } = useEstimationStore();
  const { saveProject, updateProject } = useProjectsStore();

  const [projectRequirements, setProjectRequirements] = useState<any[]>([]);
  const [isGeneratingWBS, setIsGeneratingWBS] = useState(false);
  const [wbsGenerated, setWbsGenerated] = useState(false);

  // Load requirements when wizard opens with a projectId
  useEffect(() => {
    if (currentProjectId) {
      // Load requirements from localStorage for this project
      const allRequirements: any[] = [];
      
      // Load from comprehensive requirements
      const comprehensiveReqs = localStorage.getItem(`comprehensive-requirements-${currentProjectId}`);
      if (comprehensiveReqs) {
        try {
          const parsed = JSON.parse(comprehensiveReqs);
          allRequirements.push(...(parsed.functionalRequirements || []));
          allRequirements.push(...(parsed.nonFunctionalRequirements || []));
          allRequirements.push(...(parsed.securityRequirements || []));
        } catch (e) {
          console.error('Error loading comprehensive requirements:', e);
        }
      }

      // Load from module-based requirements
      const requirementsKey = `project-${currentProjectId}-requirements`;
      const moduleReqs = localStorage.getItem(requirementsKey);
      if (moduleReqs) {
        try {
          const parsed = JSON.parse(moduleReqs);
          allRequirements.push(...parsed);
        } catch (e) {
          console.error('Error loading module requirements:', e);
        }
      }

      setProjectRequirements(allRequirements);
      console.log(`Loaded ${allRequirements.length} requirements for project ${currentProjectId}`);
    }
  }, [currentProjectId]);

  const handleBack = () => {
    navigate('/');
  };

  const handleSave = () => {
    saveDraft();
    toast.success('Draft saved successfully! ✅');
  };

  // Save or Update Project
  const handleSaveProject = () => {
    const projectData = {
      name: project.name || 'Untitled Project',
      description: `${screens.length} screens, ${selectedMethods.length} methods`,
      status: results ? 'COMPLETED' : screens.length > 0 ? 'IN_PROGRESS' : 'DRAFT',
      selectedMethods,
      project,
      screens,
      contingency,
      riskFactors,
      results,
      capacityPlan,
    };

    // Check if we're editing an existing project
    if (currentProjectId) {
      // UPDATE existing project
      updateProject(currentProjectId, projectData);
      toast.success('Project updated! ✅', {
        description: `"${project.name}" has been updated`,
      });
    } else {
      // CREATE new project
      const projectId = saveProject(projectData);
      // Set current project ID so future saves update instead of creating duplicates
      setCurrentProjectId(projectId);
      toast.success('Project saved! 🎉', {
        description: 'You can now access it from My Projects',
        action: {
          label: 'View Projects',
          onClick: () => navigate('/projects'),
        },
      });
    }
  };

  // Save As New (always creates a new project)
  const handleSaveAsProject = () => {
    const projectId = saveProject({
      name: `${project.name || 'Untitled Project'} (Copy)`,
      description: `${screens.length} screens, ${selectedMethods.length} methods`,
      status: results ? 'COMPLETED' : screens.length > 0 ? 'IN_PROGRESS' : 'DRAFT',
      selectedMethods,
      project,
      screens,
      contingency,
      riskFactors,
      results,
      capacityPlan,
    });
    
    // Update current project ID to the new copy
    setCurrentProjectId(projectId);
    
    toast.success('Project saved as new! 🎉', {
      description: 'A new copy has been created',
      action: {
        label: 'View Projects',
        onClick: () => navigate('/projects'),
      },
    });
  };

  const handleViewProjects = () => {
    navigate('/projects');
  };

  // Format last saved time
  const getLastSavedText = () => {
    if (!lastSaved) return null;
    
    const savedDate = new Date(lastSaved);
    const now = new Date();
    const diffMs = now.getTime() - savedDate.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    
    if (diffMins < 1) return 'Saved just now';
    if (diffMins < 60) return `Saved ${diffMins}m ago`;
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `Saved ${diffHours}h ago`;
    const diffDays = Math.floor(diffHours / 24);
    return `Saved ${diffDays}d ago`;
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-indigo-50 to-purple-50">
      {/* Header */}
      <div className="bg-white border-b border-gray-200 shadow-sm">
        <div className="max-w-6xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <Button variant="ghost" size="sm" onClick={handleBack}>
                <ArrowLeft className="w-4 h-4 mr-2" />
                Back to Home
              </Button>
              <div className="h-6 w-px bg-gray-300"></div>
              <h1 className="text-2xl font-bold text-gray-900">
                Estimation Wizard
              </h1>
            </div>
            <div className="flex items-center gap-3">
              <Button variant="ghost" size="sm" onClick={handleViewProjects}>
                <FolderOpen className="w-4 h-4 mr-2" />
                My Projects
              </Button>
              <div className="flex flex-col items-end gap-2">
                <div className="flex gap-2">
                  <Button variant="outline" size="sm" onClick={handleSave}>
                    <Save className="w-4 h-4 mr-2" />
                    Save Draft
                  </Button>
                  <Button size="sm" onClick={handleSaveProject} className="bg-green-600 hover:bg-green-700">
                    <FolderOpen className="w-4 h-4 mr-2" />
                    Save Project
                  </Button>
                  <Button size="sm" onClick={handleSaveAsProject} className="bg-green-600 hover:bg-green-700">
                    <FolderOpen className="w-4 h-4 mr-2" />
                    Save as New
                  </Button>
                </div>
                {lastSaved && (
                  <span className="text-xs text-gray-500">{getLastSavedText()}</span>
                )}
              </div>
              <Button variant="ghost" size="sm" onClick={handleBack}>
                <X className="w-4 h-4" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      {/* Step Indicators */}
      <StepIndicator />

      {/* Main Content */}
      <div className="max-w-6xl mx-auto px-6 py-8">
        <div className="bg-white rounded-lg shadow-lg p-8">
          <Routes>
            <Route index element={<Navigate to="/wizard/step/1" replace />} />
            <Route path="step/1" element={<MethodSelectionWrapper />} />
            <Route path="step/2" element={<ProjectSetup />} />
            <Route path="step/3" element={<ScopeDefinition />} />
            <Route path="step/4" element={<MethodDetails />} />
            <Route path="step/5" element={<ComplexityWeighting />} />
            <Route path="step/6" element={<RiskAssessment />} />
            <Route path="step/7" element={<ReviewCalculate />} />
            <Route path="step/8" element={<CapacityPlanning />} />
            <Route path="step/9" element={<GenerateDeliverables />} />
          </Routes>
        </div>
      </div>
    </div>
  );
}