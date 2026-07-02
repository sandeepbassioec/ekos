// Project Management Types

export interface Project {
  id: string;
  name: string;
  description: string;
  clientName: string;
  industry: string;
  
  // Project Details
  startDate: string;
  endDate?: string;
  estimatedBudget?: number;
  currency: string;
  
  // Status
  status: 'planning' | 'in_progress' | 'on_hold' | 'completed' | 'cancelled';
  phase: 'requirements' | 'design' | 'development' | 'testing' | 'deployment';
  
  // Team
  projectManager: string;
  teamMembers: string[];
  
  // Progress
  completionPercentage: number;
  
  // Modules
  modules: ProjectModule[];
  
  // Metadata
  createdBy: string;
  createdAt: string;
  updatedAt: string;
  
  // Linked Data
  businessRequirementsId?: string; // From Industry Wizard
  totalRequirements: number;
  totalEstimatedHours: number;
  
  // Deliverables Generated
  hasWBS: boolean;
  hasEstimation: boolean;
  hasCapacityPlan: boolean;
  hasArchitecture: boolean;
  hasHLD: boolean;
  hasLLD: boolean;
  hasERDiagram: boolean;
  hasDataFlow: boolean;
  hasEntityFlow: boolean;
}

export interface ProjectModule {
  id: string;
  projectId: string;
  name: string;
  description: string;
  priority: 'high' | 'medium' | 'low';
  status: 'not_started' | 'in_progress' | 'completed';
  
  // Ownership
  moduleOwner?: string;
  assignedTeam: string[];
  
  // Requirements
  requirementIds: string[];
  totalRequirements: number;
  
  // Estimation
  estimatedHours: number;
  actualHours?: number;
  
  // Progress
  completionPercentage: number;
  
  // Technical Details
  technologies: string[];
  dependencies: string[]; // Other module IDs
  
  // Deliverables
  hasComponentDiagram: boolean;
  hasSequenceDiagram: boolean;
  
  createdAt: string;
  updatedAt: string;
}

// Project Creation Steps
export type ProjectCreationStep = 
  | 'basic_info'
  | 'modules'
  | 'team_assignment'
  | 'timeline'
  | 'review';

export interface ProjectCreationData {
  // Step 1: Basic Info
  name: string;
  description: string;
  clientName: string;
  industry: string;
  currency: string;
  estimatedBudget?: number;
  
  // Step 2: Modules
  modules: Partial<ProjectModule>[];
  
  // Step 3: Team
  projectManager: string;
  teamMembers: string[];
  
  // Step 4: Timeline
  startDate: string;
  endDate?: string;
  
  // Optional: Link to Business Requirements
  linkedBusinessRequirementsId?: string;
}

// Project Dashboard Stats
export interface ProjectStats {
  totalProjects: number;
  activeProjects: number;
  completedProjects: number;
  totalRequirements: number;
  totalEstimatedHours: number;
  teamUtilization: number;
}

// Requirement Context (for requirement capture)
export interface RequirementContext {
  projectId: string;
  projectName: string;
  moduleId: string;
  moduleName: string;
}
