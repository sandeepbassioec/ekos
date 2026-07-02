// Business Requirements Capture Types

// Industry Templates
export type Industry = 
  | 'healthcare'
  | 'banking'
  | 'ecommerce'
  | 'manufacturing'
  | 'insurance'
  | 'logistics'
  | 'education'
  | 'government'
  | 'retail'
  | 'telecom'
  | 'custom';

export interface IndustryTemplate {
  id: string;
  name: string;
  industry: Industry;
  icon: string;
  description: string;
  processTemplates: ProcessTemplate[];
  complianceRequirements: string[];
  commonIntegrations: string[];
  defaultQuestionnaires: string[];
  stakeholderRoles: string[];
}

// Business Process Mapping
export interface ProcessTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  steps: ProcessStep[];
  triggerType: 'manual' | 'automated' | 'scheduled' | 'event';
  stakeholders: string[];
}

export interface ProcessStep {
  id: string;
  name: string;
  description: string;
  stepType: 'data_collection' | 'validation' | 'decision' | 'integration' | 'notification' | 'storage' | 'approval';
  requiredCapabilities: string[];
  technicalNeeds: string[];
  complianceChecks: string[];
  dataInputs: DataElement[];
  dataOutputs: DataElement[];
  order: number;
}

export interface DataElement {
  name: string;
  type: string;
  required: boolean;
  sensitive: boolean;
  complianceTag?: string;
}

// Questionnaire Engine
export interface Questionnaire {
  id: string;
  name: string;
  category: 'discovery' | 'compliance' | 'technical' | 'integration' | 'stakeholder';
  industry?: Industry;
  questions: Question[];
  targetStakeholder?: string;
}

export type QuestionType = 
  | 'single_choice'
  | 'multiple_choice'
  | 'text'
  | 'number'
  | 'range'
  | 'boolean'
  | 'matrix'
  | 'scale';

export interface Question {
  id: string;
  text: string;
  type: QuestionType;
  options?: QuestionOption[];
  validation?: {
    required?: boolean;
    min?: number;
    max?: number;
    pattern?: string;
  };
  conditionalLogic?: ConditionalLogic;
  helpText?: string;
  suggestedFollowups?: string[];
  impactAnalysis?: ImpactAnalysis;
}

export interface QuestionOption {
  value: string;
  label: string;
  icon?: string;
  description?: string;
  triggers?: string[]; // IDs of follow-up questions
  autoSuggests?: AutoSuggestion[];
}

export interface ConditionalLogic {
  showIf: {
    questionId: string;
    operator: 'equals' | 'not_equals' | 'contains' | 'greater_than' | 'less_than';
    value: any;
  }[];
  operator: 'AND' | 'OR';
}

export interface AutoSuggestion {
  type: 'capability' | 'integration' | 'compliance' | 'artifact' | 'risk';
  value: string;
  reason: string;
  priority: 'high' | 'medium' | 'low';
}

export interface ImpactAnalysis {
  complexity: 'simple' | 'medium' | 'complex';
  estimatedHours?: number;
  requiredSkills?: string[];
  dependencies?: string[];
  risks?: string[];
}

// Business Capabilities
export interface BusinessCapability {
  id: string;
  name: string;
  category: string;
  description: string;
  requiredForProcesses: string[];
  technicalRequirements: TechnicalRequirement[];
  complianceImplications: string[];
  estimatedComplexity: 'simple' | 'medium' | 'complex';
}

export interface TechnicalRequirement {
  type: 'frontend' | 'backend' | 'database' | 'integration' | 'infrastructure';
  description: string;
  artifacts: string[];
  estimatedHours: number;
}

// Stakeholder Perspectives
export interface StakeholderView {
  role: string;
  relevantQuestions: string[];
  outputFormat: 'user_stories' | 'technical_specs' | 'architecture' | 'test_cases' | 'compliance_checklist';
  customFields: string[];
}

// Context-Aware Requirements
export interface RequirementContext {
  projectId: string;
  industry: Industry;
  selectedProcesses: string[];
  answers: Record<string, any>;
  detectedCapabilities: string[];
  suggestedIntegrations: string[];
  complianceRequirements: string[];
  stakeholders: string[];
  capturedAt: string;
}

// Requirement Translation
export interface RequirementTranslation {
  id: string;
  contextId: string;
  businessRequirements: BusinessRequirement[];
  generatedArtifacts: GeneratedArtifact[];
  generatedWBS: WBSItem[];
  generatedStories: UserStory[];
  architectureComponents: ArchitectureComponent[];
  estimatedEffort: EffortEstimate;
  risks: Risk[];
}

export interface BusinessRequirement {
  id: string;
  type: 'functional' | 'non_functional' | 'compliance' | 'integration' | 'data';
  title: string;
  description: string;
  priority: 'must_have' | 'should_have' | 'could_have' | 'wont_have';
  source: string; // questionnaire ID or process ID
  relatedProcesses: string[];
  relatedCapabilities: string[];
  acceptanceCriteria: string[];
  stakeholders: string[];
}

export interface GeneratedArtifact {
  type: string;
  name: string;
  action: 'CREATE' | 'MODIFY' | 'DELETE';
  properties: Record<string, any>;
  derivedFrom: string[];
  estimatedHours: number;
}

export interface WBSItem {
  id: string;
  title: string;
  description: string;
  phase: string;
  estimatedHours: number;
  dependencies: string[];
  assignedRole?: string;
  derivedFrom: string;
}

export interface UserStory {
  id: string;
  title: string;
  asA: string;
  iWant: string;
  soThat: string;
  acceptanceCriteria: string[];
  priority: 'high' | 'medium' | 'low';
  estimatedPoints: number;
  derivedFrom: string;
}

export interface ArchitectureComponent {
  id: string;
  name: string;
  type: 'frontend' | 'backend' | 'database' | 'integration' | 'infrastructure';
  description: string;
  technologies: string[];
  connectsTo: string[];
  derivedFrom: string[];
}

export interface EffortEstimate {
  totalHours: number;
  breakdown: {
    frontend: number;
    backend: number;
    database: number;
    integration: number;
    testing: number;
    deployment: number;
  };
  confidence: 'low' | 'medium' | 'high';
}

export interface Risk {
  id: string;
  title: string;
  description: string;
  category: 'technical' | 'compliance' | 'integration' | 'resource' | 'timeline';
  probability: 'low' | 'medium' | 'high';
  impact: 'low' | 'medium' | 'high';
  mitigation: string;
  derivedFrom: string;
}
