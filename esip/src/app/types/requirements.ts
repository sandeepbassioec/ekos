// Product Owner Requirements Capture

export type RequirementType = 
  | 'new_screen'           // Naya screen banana hai
  | 'update_screen'        // Existing screen update karna hai
  | 'bug_fix'              // Bug fix karna hai
  | 'feature_change'       // Feature change/enhancement
  | 'api_development'      // Backend API banana hai
  | 'integration'          // Third-party integration
  | 'data_migration'       // Data migration
  | 'performance'          // Performance improvement
  | 'security'             // Security fix/enhancement
  | 'ui_ux_change';        // UI/UX change

export type RequirementPriority = 'critical' | 'high' | 'medium' | 'low';
export type RequirementStatus = 'draft' | 'review' | 'approved' | 'in_progress' | 'completed' | 'rejected' | 'finalized';
export type RequirementComplexity = 'simple' | 'medium' | 'complex' | 'very_complex';

export interface Requirement {
  id: string;
  projectId: string;
  type: RequirementType;
  title: string;
  description: string;
  
  // Sprint & PI Context
  programIncrement?: string;  // e.g., 'PI-1', 'PI-2'
  sprint?: string;            // e.g., 'Sprint-1', 'Sprint-2'
  moduleIds: string[];        // Can belong to multiple modules
  
  // Business Context
  businessValue: string;
  userStory?: {
    asA: string;      // As a [role]
    iWant: string;    // I want [feature]
    soThat: string;   // So that [benefit]
  };
  
  // Technical Details
  affectedScreens: string[];
  affectedAPIs: string[];
  affectedComponents: string[];
  technicalNotes?: string;
  
  // Acceptance Criteria
  acceptanceCriteria: AcceptanceCriterion[];
  
  // Dependencies
  dependencies: string[]; // Other requirement IDs
  blockers: string[];
  
  // Estimation
  estimatedHours?: number;
  complexity: RequirementComplexity;
  
  // Assignment
  priority: RequirementPriority;
  status: RequirementStatus;
  assignedTo?: string;

  // Finalisation & PI allocation (Step 3)
  finalizedBy?: string;
  finalizedAt?: string;
  allocatedPiId?: string;    // FK to the PI document
  allocatedPiName?: string;  // display name cached for quick render
  
  // Metadata
  createdBy: string;
  createdAt: string;
  updatedAt: string;
  
  // Attachments
  attachments: Attachment[];
  
  // Comments/Discussion
  comments: Comment[];
}

export interface AcceptanceCriterion {
  id: string;
  description: string;
  completed: boolean;
}

export interface Attachment {
  id: string;
  name: string;
  url: string;
  type: 'image' | 'document' | 'mockup' | 'other';
  uploadedAt: string;
}

export interface Comment {
  id: string;
  userId: string;
  userName: string;
  text: string;
  createdAt: string;
}

// Screen-specific details
export interface ScreenRequirement extends Requirement {
  type: 'new_screen' | 'update_screen';
  screenDetails: {
    screenName: string;
    screenType: 'list' | 'form' | 'detail' | 'dashboard' | 'modal' | 'other';
    hasDataTable: boolean;
    hasForms: boolean;
    hasCharts: boolean;
    hasFileUpload: boolean;
    
    // Fields for forms
    fields?: FieldDefinition[];
    
    // Data operations
    operations: {
      create: boolean;
      read: boolean;
      update: boolean;
      delete: boolean;
      export: boolean;
      import: boolean;
    };
    
    // UI Elements
    uiElements: {
      buttons: string[];
      tabs: string[];
      sections: string[];
    };
  };
}

export interface FieldDefinition {
  name: string;
  label: string;
  type: 'text' | 'number' | 'email' | 'phone' | 'date' | 'select' | 'multiselect' | 'checkbox' | 'radio' | 'textarea' | 'file';
  required: boolean;
  validation?: string;
  options?: string[]; // For select/radio
}

// Bug-specific details
export interface BugRequirement extends Requirement {
  type: 'bug_fix';
  bugDetails: {
    severity: 'critical' | 'high' | 'medium' | 'low';
    stepsToReproduce: string[];
    expectedBehavior: string;
    actualBehavior: string;
    environment: string;
    browserVersion?: string;
    deviceType?: string;
    screenshots?: string[];
  };
}

// API-specific details
export interface APIRequirement extends Requirement {
  type: 'api_development';
  apiDetails: {
    endpoint: string;
    method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
    requestBody?: object;
    responseFormat?: object;
    authentication: boolean;
    rateLimit?: string;
    errorCodes: { code: number; description: string }[];
  };
}

// Template for common requirement types
export interface RequirementTemplate {
  id: string;
  name: string;
  type: RequirementType;
  description: string;
  defaultAcceptanceCriteria: string[];
  estimatedHours: number;
  complexity: RequirementComplexity;
}

// Bulk import from Excel/CSV
export interface BulkRequirementImport {
  requirements: Partial<Requirement>[];
  errors: { row: number; error: string }[];
}