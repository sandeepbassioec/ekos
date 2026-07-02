import { useEffect, useState } from 'react';
import { useNavigate, useLocation } from 'react-router';
import { useEstimationStore } from '../../store/estimationStore';
import { generateWBSFromRequirements } from '../../utils/wbsGenerator';
import { MethodSelection } from './MethodSelection';
import { Card } from '../../components/ui/card';
import { Loader2, Sparkles, CheckCircle2 } from 'lucide-react';
import { toast } from 'sonner';

export function MethodSelectionWrapper() {
  const navigate = useNavigate();
  const location = useLocation();
  const { 
    selectedMethods, 
    currentProjectId, 
    project,
    screens,
    addScreen,
    setCurrentStep 
  } = useEstimationStore();
  
  const [isGeneratingWBS, setIsGeneratingWBS] = useState(false);
  const [generationComplete, setGenerationComplete] = useState(false);
  const [allowView, setAllowView] = useState(false);

  // Check if user navigated back manually (via Back button)
  const isManualNavigation = location.state?.fromBack || false;

  // Check if method already selected and WBS exists
  useEffect(() => {
    // If user manually clicked Back button, allow them to see this page
    if (isManualNavigation) {
      setAllowView(true);
      return;
    }

    if (currentProjectId && selectedMethods.length > 0 && screens.length > 0) {
      // Skip step 1, already selected
      setTimeout(() => {
        navigate('/wizard/step/2', { replace: true });
      }, 100);
    } else {
      setAllowView(true);
    }
  }, [currentProjectId, selectedMethods, screens, navigate, isManualNavigation]);

  // Watch for method selection to trigger WBS generation
  useEffect(() => {
    const handleWBSGeneration = async () => {
      // Only generate if:
      // 1. We have a projectId (coming from requirements capture)
      // 2. Methods were just selected
      // 3. No screens exist yet
      // 4. Not already generating
      if (currentProjectId && selectedMethods.length > 0 && screens.length === 0 && !isGeneratingWBS && !generationComplete) {
        // Load requirements
        const allRequirements: any[] = [];
        
        const comprehensiveReqs = localStorage.getItem(`comprehensive-requirements-${currentProjectId}`);
        if (comprehensiveReqs) {
          try {
            const parsed = JSON.parse(comprehensiveReqs);
            allRequirements.push(...(parsed.functionalRequirements || []));
            allRequirements.push(...(parsed.nonFunctionalRequirements || []));
            allRequirements.push(...(parsed.securityRequirements || []));
            allRequirements.push(...(parsed.integrationRequirements || []));
          } catch (e) {
            console.error('Error loading comprehensive requirements:', e);
          }
        }

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

        if (allRequirements.length === 0) {
          // No requirements, skip generation
          console.log('No requirements found, skipping WBS generation');
          navigate('/wizard/step/2');
          return;
        }

        // Generate WBS
        setIsGeneratingWBS(true);
        
        try {
          toast.info('🤖 Analyzing requirements and generating WBS...', {
            duration: 10000,
          });

          const wbsItems = await generateWBSFromRequirements({
            projectName: project.name || 'Project',
            requirements: allRequirements,
            estimationMethod: selectedMethods[0], // Use first selected method
          });

          // Add generated screens to store
          wbsItems.forEach((screen) => {
            addScreen(screen);
          });

          setGenerationComplete(true);
          
          toast.success(`✅ Generated ${wbsItems.length} WBS items from ${allRequirements.length} requirements!`, {
            description: 'You can now review and estimate each item',
          });

          // Navigate to next step
          setTimeout(() => {
            setCurrentStep(2);
            navigate('/wizard/step/2');
          }, 1500);
        } catch (error) {
          console.error('Error generating WBS:', error);
          toast.error('Failed to generate WBS. You can add items manually.', {
            description: error instanceof Error ? error.message : 'Unknown error',
          });
          
          // Navigate anyway
          setTimeout(() => {
            setCurrentStep(2);
            navigate('/wizard/step/2');
          }, 2000);
        } finally {
          setIsGeneratingWBS(false);
        }
      }
    };

    handleWBSGeneration();
  }, [selectedMethods, currentProjectId, screens, isGeneratingWBS, generationComplete, project.name, addScreen, setCurrentStep, navigate]);

  // Show loading state while generating
  if (isGeneratingWBS) {
    return (
      <div className="space-y-6">
        <Card className="p-12">
          <div className="flex flex-col items-center justify-center text-center">
            <div className="relative mb-6">
              <Loader2 className="w-16 h-16 text-blue-600 animate-spin" />
              <Sparkles className="w-8 h-8 text-yellow-500 absolute top-0 right-0 animate-pulse" />
            </div>
            <h3 className="text-2xl font-bold mb-2">AI is Generating Your WBS...</h3>
            <p className="text-muted-foreground mb-4">
              Analyzing requirements and breaking down into estimable work items
            </p>
            <div className="flex items-center gap-2 text-sm text-blue-600">
              <span className="inline-block w-2 h-2 bg-blue-600 rounded-full animate-pulse"></span>
              <span>This may take 10-30 seconds</span>
            </div>
          </div>
        </Card>
      </div>
    );
  }

  // Show skip message
  if (!allowView && !isGeneratingWBS) {
    return (
      <div className="space-y-6">
        <Card className="p-12">
          <div className="flex flex-col items-center justify-center text-center">
            <CheckCircle2 className="w-16 h-16 text-green-600 mb-4" />
            <h3 className="text-2xl font-bold mb-2">Estimation Method Already Selected</h3>
            <p className="text-muted-foreground">
              Proceeding to project setup...
            </p>
          </div>
        </Card>
      </div>
    );
  }

  // Show method selection
  return <MethodSelection />;
}