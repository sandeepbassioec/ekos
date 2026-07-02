// Meta-Model Data Store (Mock Database)
import { 
  ArtifactType, 
  FieldDefinition, 
  ComplexityRule, 
  WBSTemplate, 
  StoryTemplate,
  ArchitectureTemplate,
  CapacityTemplate,
  ChangeContextType,
  ArtifactInstance
} from '../types/metamodel';

const STORAGE_KEYS = {
  ARTIFACT_TYPES: 'metamodel_artifact_types',
  FIELD_DEFINITIONS: 'metamodel_field_definitions',
  COMPLEXITY_RULES: 'metamodel_complexity_rules',
  WBS_TEMPLATES: 'metamodel_wbs_templates',
  STORY_TEMPLATES: 'metamodel_story_templates',
  ARCHITECTURE_TEMPLATES: 'metamodel_architecture_templates',
  CAPACITY_TEMPLATES: 'metamodel_capacity_templates',
  CHANGE_CONTEXTS: 'metamodel_change_contexts',
  ARTIFACT_INSTANCES: 'metamodel_artifact_instances',
};

// Initialize default data
export function initializeMetaModel() {
  if (!localStorage.getItem(STORAGE_KEYS.ARTIFACT_TYPES)) {
    const defaultArtifacts: ArtifactType[] = [
      { id: 'artifact_screen', name: 'Screen', category: 'application', icon: '📱', isActive: true, displayOrder: 1, createdAt: new Date().toISOString() },
      { id: 'artifact_business_logic', name: 'Business Logic', category: 'application', icon: '⚙️', isActive: true, displayOrder: 2, createdAt: new Date().toISOString() },
      { id: 'artifact_data_table', name: 'Data Table', category: 'application', icon: '🗄️', isActive: true, displayOrder: 3, createdAt: new Date().toISOString() },
      { id: 'artifact_api', name: 'API Endpoint', category: 'application', icon: '🔌', isActive: true, displayOrder: 4, createdAt: new Date().toISOString() },
      { id: 'artifact_integration', name: 'Integration', category: 'integration', icon: '🔗', isActive: true, displayOrder: 5, createdAt: new Date().toISOString() },
      { id: 'artifact_dependency', name: 'Dependency', category: 'dependency', icon: '📚', isActive: true, displayOrder: 6, createdAt: new Date().toISOString() },
      { id: 'artifact_bg_job', name: 'Background Job', category: 'infrastructure', icon: '⏰', isActive: false, displayOrder: 7, createdAt: new Date().toISOString() },
    ];
    localStorage.setItem(STORAGE_KEYS.ARTIFACT_TYPES, JSON.stringify(defaultArtifacts));

    const defaultFields: FieldDefinition[] = [
      // Screen fields
      { id: 'f1', artifactTypeId: 'artifact_screen', fieldName: 'screen_name', fieldLabel: 'Screen Name', fieldType: 'text', isRequired: true, displayOrder: 1, createdAt: new Date().toISOString() },
      { id: 'f2', artifactTypeId: 'artifact_screen', fieldName: 'screen_type', fieldLabel: 'Screen Type', fieldType: 'select', fieldOptions: ['Web', 'Mobile', 'Desktop'], isRequired: true, displayOrder: 2, createdAt: new Date().toISOString() },
      { id: 'f3', artifactTypeId: 'artifact_screen', fieldName: 'complexity', fieldLabel: 'UI Complexity', fieldType: 'select', fieldOptions: ['Simple', 'Medium', 'Complex'], isRequired: false, displayOrder: 3, createdAt: new Date().toISOString() },
    ];
    localStorage.setItem(STORAGE_KEYS.FIELD_DEFINITIONS, JSON.stringify(defaultFields));

    localStorage.setItem(STORAGE_KEYS.COMPLEXITY_RULES, JSON.stringify([]));
    localStorage.setItem(STORAGE_KEYS.WBS_TEMPLATES, JSON.stringify([]));
    localStorage.setItem(STORAGE_KEYS.STORY_TEMPLATES, JSON.stringify([]));
    localStorage.setItem(STORAGE_KEYS.ARCHITECTURE_TEMPLATES, JSON.stringify([]));
    localStorage.setItem(STORAGE_KEYS.CAPACITY_TEMPLATES, JSON.stringify([]));
    localStorage.setItem(STORAGE_KEYS.ARTIFACT_INSTANCES, JSON.stringify([]));

    const defaultContexts: ChangeContextType[] = [
      { id: 'ctx_new_feature', name: 'New Feature / Module', description: 'Adding new functionality', applicableArtifactCategories: ['application', 'infrastructure'], isActive: true },
      { id: 'ctx_enhancement', name: 'Enhancement', description: 'Improving existing feature', applicableArtifactCategories: ['application'], isActive: true },
      { id: 'ctx_bug_fix', name: 'Bug Fix', description: 'Fixing defects', applicableArtifactCategories: ['application'], isActive: true },
    ];
    localStorage.setItem(STORAGE_KEYS.CHANGE_CONTEXTS, JSON.stringify(defaultContexts));
  }
}

// CRUD operations
export const MetaModelAPI = {
  // Artifact Types
  getArtifactTypes: (): ArtifactType[] => JSON.parse(localStorage.getItem(STORAGE_KEYS.ARTIFACT_TYPES) || '[]'),
  getActiveArtifactTypes: (): ArtifactType[] => JSON.parse(localStorage.getItem(STORAGE_KEYS.ARTIFACT_TYPES) || '[]').filter((a: ArtifactType) => a.isActive),
  getArtifactType: (id: string): ArtifactType | undefined => JSON.parse(localStorage.getItem(STORAGE_KEYS.ARTIFACT_TYPES) || '[]').find((a: ArtifactType) => a.id === id),
  createArtifactType: (data: Omit<ArtifactType, 'id' | 'createdAt'>): ArtifactType => {
    const types = JSON.parse(localStorage.getItem(STORAGE_KEYS.ARTIFACT_TYPES) || '[]');
    const newType: ArtifactType = { ...data, id: `artifact_${Date.now()}`, createdAt: new Date().toISOString() };
    types.push(newType);
    localStorage.setItem(STORAGE_KEYS.ARTIFACT_TYPES, JSON.stringify(types));
    return newType;
  },
  updateArtifactType: (id: string, data: Partial<ArtifactType>): ArtifactType | null => {
    const types = JSON.parse(localStorage.getItem(STORAGE_KEYS.ARTIFACT_TYPES) || '[]');
    const index = types.findIndex((t: ArtifactType) => t.id === id);
    if (index === -1) return null;
    types[index] = { ...types[index], ...data };
    localStorage.setItem(STORAGE_KEYS.ARTIFACT_TYPES, JSON.stringify(types));
    return types[index];
  },
  deleteArtifactType: (id: string): boolean => {
    const types = JSON.parse(localStorage.getItem(STORAGE_KEYS.ARTIFACT_TYPES) || '[]');
    const filtered = types.filter((t: ArtifactType) => t.id !== id);
    localStorage.setItem(STORAGE_KEYS.ARTIFACT_TYPES, JSON.stringify(filtered));
    return true;
  },

  // Field Definitions
  getFieldDefinitions: (artifactTypeId?: string): FieldDefinition[] => {
    const fields = JSON.parse(localStorage.getItem(STORAGE_KEYS.FIELD_DEFINITIONS) || '[]');
    return artifactTypeId ? fields.filter((f: FieldDefinition) => f.artifactTypeId === artifactTypeId) : fields;
  },
  createFieldDefinition: (data: Omit<FieldDefinition, 'id' | 'createdAt'>): FieldDefinition => {
    const fields = JSON.parse(localStorage.getItem(STORAGE_KEYS.FIELD_DEFINITIONS) || '[]');
    const newField: FieldDefinition = { ...data, id: `field_${Date.now()}`, createdAt: new Date().toISOString() };
    fields.push(newField);
    localStorage.setItem(STORAGE_KEYS.FIELD_DEFINITIONS, JSON.stringify(fields));
    return newField;
  },
  updateFieldDefinition: (id: string, data: Partial<FieldDefinition>): FieldDefinition | null => {
    const fields = JSON.parse(localStorage.getItem(STORAGE_KEYS.FIELD_DEFINITIONS) || '[]');
    const index = fields.findIndex((f: FieldDefinition) => f.id === id);
    if (index === -1) return null;
    fields[index] = { ...fields[index], ...data };
    localStorage.setItem(STORAGE_KEYS.FIELD_DEFINITIONS, JSON.stringify(fields));
    return fields[index];
  },
  deleteFieldDefinition: (id: string): boolean => {
    const fields = JSON.parse(localStorage.getItem(STORAGE_KEYS.FIELD_DEFINITIONS) || '[]');
    const filtered = fields.filter((f: FieldDefinition) => f.id !== id);
    localStorage.setItem(STORAGE_KEYS.FIELD_DEFINITIONS, JSON.stringify(filtered));
    return true;
  },

  // Complexity Rules
  getComplexityRules: (artifactTypeId?: string): ComplexityRule[] => {
    const rules = JSON.parse(localStorage.getItem(STORAGE_KEYS.COMPLEXITY_RULES) || '[]');
    return artifactTypeId ? rules.filter((r: ComplexityRule) => r.artifactTypeId === artifactTypeId) : rules;
  },
  createComplexityRule: (data: Omit<ComplexityRule, 'id' | 'createdAt'>): ComplexityRule => {
    const rules = JSON.parse(localStorage.getItem(STORAGE_KEYS.COMPLEXITY_RULES) || '[]');
    const newRule: ComplexityRule = { ...data, id: `rule_${Date.now()}`, createdAt: new Date().toISOString() };
    rules.push(newRule);
    localStorage.setItem(STORAGE_KEYS.COMPLEXITY_RULES, JSON.stringify(rules));
    return newRule;
  },

  // WBS Templates
  getWBSTemplates: (artifactTypeId?: string): WBSTemplate[] => {
    const templates = JSON.parse(localStorage.getItem(STORAGE_KEYS.WBS_TEMPLATES) || '[]');
    return artifactTypeId ? templates.filter((t: WBSTemplate) => t.artifactTypeId === artifactTypeId) : templates;
  },
  createWBSTemplate: (data: Omit<WBSTemplate, 'id' | 'createdAt'>): WBSTemplate => {
    const templates = JSON.parse(localStorage.getItem(STORAGE_KEYS.WBS_TEMPLATES) || '[]');
    const newTemplate: WBSTemplate = { ...data, id: `wbs_${Date.now()}`, createdAt: new Date().toISOString() };
    templates.push(newTemplate);
    localStorage.setItem(STORAGE_KEYS.WBS_TEMPLATES, JSON.stringify(templates));
    return newTemplate;
  },

  // Story Templates
  getStoryTemplates: (artifactTypeId?: string): StoryTemplate[] => {
    const templates = JSON.parse(localStorage.getItem(STORAGE_KEYS.STORY_TEMPLATES) || '[]');
    return artifactTypeId ? templates.filter((t: StoryTemplate) => t.artifactTypeId === artifactTypeId) : templates;
  },
  createStoryTemplate: (data: Omit<StoryTemplate, 'id' | 'createdAt'>): StoryTemplate => {
    const templates = JSON.parse(localStorage.getItem(STORAGE_KEYS.STORY_TEMPLATES) || '[]');
    const newTemplate: StoryTemplate = { ...data, id: `story_${Date.now()}`, createdAt: new Date().toISOString() };
    templates.push(newTemplate);
    localStorage.setItem(STORAGE_KEYS.STORY_TEMPLATES, JSON.stringify(templates));
    return newTemplate;
  },

  // Artifact Instances (user-created artifacts)
  getArtifactInstances: (projectId?: string): ArtifactInstance[] => {
    const instances = JSON.parse(localStorage.getItem(STORAGE_KEYS.ARTIFACT_INSTANCES) || '[]');
    return projectId ? instances.filter((i: ArtifactInstance) => i.projectId === projectId) : instances;
  },
  createArtifactInstance: (data: Omit<ArtifactInstance, 'id' | 'createdAt' | 'updatedAt'>): ArtifactInstance => {
    const instances = JSON.parse(localStorage.getItem(STORAGE_KEYS.ARTIFACT_INSTANCES) || '[]');
    const now = new Date().toISOString();
    const newInstance: ArtifactInstance = { ...data, id: `instance_${Date.now()}`, createdAt: now, updatedAt: now };
    instances.push(newInstance);
    localStorage.setItem(STORAGE_KEYS.ARTIFACT_INSTANCES, JSON.stringify(instances));
    return newInstance;
  },

  // Change Contexts
  getChangeContexts: (): ChangeContextType[] => JSON.parse(localStorage.getItem(STORAGE_KEYS.CHANGE_CONTEXTS) || '[]'),
};
