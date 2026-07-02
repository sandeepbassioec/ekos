// Meta-Model Type Definitions

export type FieldType = 'text' | 'number' | 'select' | 'multiselect' | 'textarea' | 'boolean' | 'date' | 'json' | 'code';

export interface ArtifactType {
  id: string;
  name: string;
  category: 'application' | 'infrastructure' | 'integration' | 'dependency' | 'system' | 'computation';
  icon: string;
  description?: string;
  isActive: boolean;
  displayOrder: number;
  createdBy?: string;
  createdAt: string;
}

export interface FieldDefinition {
  id: string;
  artifactTypeId: string;
  fieldName: string;
  fieldLabel: string;
  fieldType: FieldType;
  fieldOptions?: string[]; // for select/multiselect
  isRequired: boolean;
  validationRules?: {
    pattern?: string;
    min?: number;
    max?: number;
    minLength?: number;
    maxLength?: number;
  };
  dependsOn?: {
    field: string;
    value: any;
    operator?: 'equals' | 'not_equals' | 'contains' | 'greater_than' | 'less_than';
  };
  placeholder?: string;
  helpText?: string;
  displayOrder: number;
  createdAt: string;
}

export type ComplexityLevel = 'SIMPLE' | 'MEDIUM' | 'COMPLEX';

export interface ComplexityRule {
  id: string;
  artifactTypeId: string;
  ruleName: string;
  ruleLogic: {
    field: string;
    operator?: 'equals' | 'greater_than' | 'less_than' | 'contains';
    value?: any;
    mapping?: Record<string, ComplexityLevel>;
  };
  defaultComplexity: ComplexityLevel;
  createdAt: string;
}

export type ActionType = 'CREATE' | 'MODIFY' | 'DELETE' | 'UPGRADE';

export interface WBSTemplate {
  id: string;
  artifactTypeId: string;
  action: ActionType;
  taskTemplate: string; // with {placeholders}
  defaultHours: number;
  complexityMultiplier: Record<ComplexityLevel, number>;
  isActive: boolean;
  createdAt: string;
}

export interface StoryTemplate {
  id: string;
  artifactTypeId: string;
  action: ActionType;
  template: string; // with {placeholders}
  actorOptions: string[];
  createdAt: string;
}

export type DiagramType = 'c4' | 'uml' | 'deployment' | 'sequence' | 'erd';

export interface ArchitectureTemplate {
  id: string;
  artifactTypeId: string;
  diagramType: DiagramType;
  placementRules: {
    layer?: string;
    connectsTo?: string[];
    position?: 'frontend' | 'backend' | 'database' | 'infrastructure' | 'external';
  };
  defaultIcon: string;
  createdAt: string;
}

export type ResourceType = 'cpu' | 'ram' | 'storage' | 'network' | 'gpu';

export interface CapacityTemplate {
  id: string;
  artifactTypeId: string;
  resourceType: ResourceType;
  baseValue: number;
  calculationFormula: string; // e.g., "{complexity_factor} * {instances}"
  unit: string; // vCPU, GB, TB, Mbps
  createdAt: string;
}

export interface ChangeContextType {
  id: string;
  name: string;
  description: string;
  applicableArtifactCategories: string[];
  isActive: boolean;
}

// Runtime artifact instance (user-created)
export interface ArtifactInstance {
  id: string;
  artifactTypeId: string;
  projectId: string;
  action: ActionType;
  name: string;
  fieldValues: Record<string, any>; // dynamic fields
  calculatedComplexity?: ComplexityLevel;
  createdAt: string;
  updatedAt: string;
}
