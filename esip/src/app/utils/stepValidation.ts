import { EstimationMethod, Screen, ProjectSetup } from '../store/estimationStore';

export interface StepStatus {
  isComplete: boolean;
  completionPercent: number;
  missingFields: string[];
}

/**
 * Check if Step 2 (Method Selection) is complete
 */
export const validateMethodSelection = (selectedMethods: EstimationMethod[]): StepStatus => {
  const isComplete = selectedMethods.length > 0;
  return {
    isComplete,
    completionPercent: isComplete ? 100 : 0,
    missingFields: isComplete ? [] : ['At least one estimation method required'],
  };
};

/**
 * Check if Step 3 (Project Setup) is complete
 */
export const validateProjectSetup = (project: ProjectSetup): StepStatus => {
  const missingFields: string[] = [];
  
  if (!project.name || project.name.trim() === '') {
    missingFields.push('Project name');
  }
  if (!project.type) {
    missingFields.push('Project type');
  }
  if (!project.teamSize || project.teamSize < 1) {
    missingFields.push('Team size');
  }
  if (!project.hourlyRate || project.hourlyRate < 1) {
    missingFields.push('Hourly rate');
  }
  
  const totalFields = 4;
  const completedFields = totalFields - missingFields.length;
  
  return {
    isComplete: missingFields.length === 0,
    completionPercent: Math.round((completedFields / totalFields) * 100),
    missingFields,
  };
};

/**
 * Check if Step 4 (Scope Definition) is complete
 */
export const validateScopeDefinition = (screens: Screen[]): StepStatus => {
  const missingFields: string[] = [];
  
  if (screens.length === 0) {
    missingFields.push('At least one screen/feature required');
  }
  
  // Check if all screens have required fields
  const invalidScreens = screens.filter(s => !s.name || !s.type || !s.complexity);
  if (invalidScreens.length > 0) {
    missingFields.push(`${invalidScreens.length} screen(s) missing required fields`);
  }
  
  const isComplete = missingFields.length === 0;
  const completionPercent = screens.length === 0 ? 0 : isComplete ? 100 : 50;
  
  return {
    isComplete,
    completionPercent,
    missingFields,
  };
};

/**
 * Check if Step 5 (Method Details) is complete
 */
export const validateMethodDetails = (
  screens: Screen[],
  selectedMethods: EstimationMethod[]
): StepStatus => {
  const missingFields: string[] = [];
  
  if (screens.length === 0) {
    return {
      isComplete: false,
      completionPercent: 0,
      missingFields: ['No screens defined'],
    };
  }
  
  let totalRequiredFields = 0;
  let completedFields = 0;
  
  screens.forEach((screen, index) => {
    selectedMethods.forEach(method => {
      if (method === 'PERT') {
        totalRequiredFields += 3; // O, M, P
        if (screen.pertData?.optimistic) completedFields++;
        if (screen.pertData?.mostLikely) completedFields++;
        if (screen.pertData?.pessimistic) completedFields++;
        
        // Validate PERT logic: O ≤ M ≤ P
        if (screen.pertData) {
          const { optimistic, mostLikely, pessimistic } = screen.pertData;
          if (optimistic && mostLikely && pessimistic) {
            if (optimistic > mostLikely || mostLikely > pessimistic) {
              missingFields.push(`${screen.name}: Invalid PERT values (O ≤ M ≤ P)`);
            }
          }
        }
      }
      
      if (method === 'STORY_POINTS') {
        totalRequiredFields += 1;
        if (screen.storyPoints) completedFields++;
      }
      
      if (method === 'TSHIRT') {
        totalRequiredFields += 1;
        if (screen.tshirtSize) completedFields++;
      }
    });
  });
  
  if (totalRequiredFields === 0) {
    return {
      isComplete: true,
      completionPercent: 100,
      missingFields: [],
    };
  }
  
  const completionPercent = Math.round((completedFields / totalRequiredFields) * 100);
  const isComplete = completionPercent === 100 && missingFields.length === 0;
  
  if (!isComplete && missingFields.length === 0) {
    missingFields.push(`${totalRequiredFields - completedFields} estimation field(s) incomplete`);
  }
  
  return {
    isComplete,
    completionPercent,
    missingFields,
  };
};

/**
 * Get overall wizard completion status
 */
export const getWizardCompletion = (
  selectedMethods: EstimationMethod[],
  project: ProjectSetup,
  screens: Screen[]
): { completionPercent: number; completedSteps: number; totalSteps: number } => {
  const step2 = validateMethodSelection(selectedMethods);
  const step3 = validateProjectSetup(project);
  const step4 = validateScopeDefinition(screens);
  const step5 = validateMethodDetails(screens, selectedMethods);
  
  const totalPercent = step2.completionPercent + step3.completionPercent + step4.completionPercent + step5.completionPercent;
  const completionPercent = Math.round(totalPercent / 4);
  
  const completedSteps = [step2, step3, step4, step5].filter(s => s.isComplete).length;
  
  return {
    completionPercent,
    completedSteps,
    totalSteps: 4,
  };
};
