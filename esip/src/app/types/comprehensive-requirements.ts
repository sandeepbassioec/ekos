// Comprehensive Requirements Types - Covers All Industries

// ============================================
// 1. NON-FUNCTIONAL REQUIREMENTS (NFRs)
// ============================================
export interface NonFunctionalRequirements {
  performance: {
    responseTime: string; // e.g., "< 2 seconds"
    throughput: string; // e.g., "1000 requests/sec"
    concurrentUsers: number;
    pageLoadTime: string;
    apiResponseTime: string;
  };
  scalability: {
    expectedUserGrowth: string;
    dataVolumeGrowth: string;
    peakLoadScenario: string;
    scalingStrategy: 'horizontal' | 'vertical' | 'both';
  };
  availability: {
    uptimeSLA: string; // e.g., "99.9%"
    maintenanceWindow: string;
    disasterRecovery: {
      rto: string; // Recovery Time Objective
      rpo: string; // Recovery Point Objective
    };
  };
  reliability: {
    mtbf: string; // Mean Time Between Failures
    errorRate: string;
    dataIntegrity: string;
  };
  usability: {
    accessibilityLevel: 'WCAG 2.1 A' | 'WCAG 2.1 AA' | 'WCAG 2.1 AAA' | 'None';
    mobileResponsive: boolean;
    screenReaderSupport: boolean;
    keyboardNavigation: boolean;
    uxStandards: string;
  };
  maintainability: {
    codeQualityStandards: string;
    documentationLevel: 'minimal' | 'standard' | 'comprehensive';
    codeCoverageTarget: number; // percentage
    technicalDebtPolicy: string;
  };
}

// ============================================
// 2. SECURITY & COMPLIANCE REQUIREMENTS
// ============================================
export interface SecurityRequirements {
  authentication: {
    methods: ('email_password' | 'oauth' | 'saml' | 'sso' | 'ldap' | 'biometric' | 'mfa')[];
    mfaRequired: boolean;
    sessionTimeout: number; // minutes
    passwordPolicy: {
      minLength: number;
      requireSpecialChars: boolean;
      requireNumbers: boolean;
      expiryDays: number;
    };
  };
  authorization: {
    model: 'rbac' | 'abac' | 'custom';
    roles: string[];
    permissionsMatrix: string; // URL or description
  };
  dataEncryption: {
    atRest: boolean;
    inTransit: boolean;
    algorithm: string; // e.g., "AES-256"
    keyManagement: string;
  };
  compliance: {
    standards: ('GDPR' | 'HIPAA' | 'SOC2' | 'PCI_DSS' | 'ISO27001' | 'SOX' | 'CCPA' | 'Other')[];
    dataResidency: string; // e.g., "EU only"
    rightToForgotten: boolean; // GDPR
    consentManagement: boolean;
  };
  auditLogging: {
    enabled: boolean;
    logRetention: number; // days
    loggedEvents: string[];
    siemIntegration: string;
  };
  securityTesting: {
    penetrationTesting: boolean;
    vulnerabilityScanning: boolean;
    owaspTop10: boolean;
    frequency: string;
  };
}

// ============================================
// 3. INTEGRATION REQUIREMENTS
// ============================================
export interface IntegrationRequirement {
  id: string;
  name: string;
  type: 'rest_api' | 'soap' | 'graphql' | 'webhook' | 'file_based' | 'database' | 'message_queue' | 'other';
  direction: 'inbound' | 'outbound' | 'bidirectional';
  description: string;
  endpoint?: string;
  authentication: string;
  dataFormat: 'json' | 'xml' | 'csv' | 'other';
  frequency: 'real_time' | 'batch' | 'scheduled';
  errorHandling: string;
  retryPolicy: string;
  rateLimit?: string;
  sla?: string;
  documentation?: string;
}

export interface IntegrationRequirements {
  externalAPIs: IntegrationRequirement[];
  thirdPartyServices: {
    name: string;
    purpose: string;
    provider: string;
  }[];
  legacySystems: {
    name: string;
    technology: string;
    integrationMethod: string;
  }[];
  ssoProviders: string[];
  messagingServices: {
    type: 'email' | 'sms' | 'push' | 'whatsapp';
    provider: string;
  }[];
}

// ============================================
// 4. DATA REQUIREMENTS
// ============================================
export interface DataRequirements {
  dataModel: {
    entities: {
      name: string;
      description: string;
      attributes: string[];
    }[];
    relationships: string;
  };
  dataMigration: {
    required: boolean;
    sourceSystems: string[];
    strategy: 'big_bang' | 'phased' | 'parallel_run';
    dataVolume: string;
    dataCleansing: string;
    mappingDocument: string;
  };
  dataRetention: {
    policy: string;
    archivalSchedule: string;
    purgeSchedule: string;
  };
  backupRecovery: {
    backupFrequency: 'real_time' | 'hourly' | 'daily' | 'weekly';
    retentionPeriod: string;
    recoveryProcedure: string;
    testingFrequency: string;
  };
  dataPrivacy: {
    piiHandling: string;
    dataMasking: boolean;
    anonymization: boolean;
    rightToDeletion: boolean;
  };
  masterData: {
    referenceSources: string[];
    syncFrequency: string;
  };
}

// ============================================
// 5. INFRASTRUCTURE & DEPLOYMENT
// ============================================
export interface InfrastructureRequirements {
  hosting: {
    type: 'cloud' | 'on_premise' | 'hybrid';
    provider?: 'aws' | 'azure' | 'gcp' | 'other';
    region?: string;
    managedServices: boolean;
  };
  database: {
    type: 'rdbms' | 'nosql' | 'newsql' | 'data_warehouse';
    engine: string; // e.g., "PostgreSQL", "MongoDB"
    replication: boolean;
    sharding: boolean;
  };
  cdn: {
    required: boolean;
    provider?: string;
    caching: string;
  };
  cicd: {
    pipelineTool: string; // e.g., "Jenkins", "GitHub Actions"
    deploymentFrequency: string;
    environments: string[]; // e.g., ["dev", "staging", "prod"]
    rollbackStrategy: string;
  };
  containerization: {
    docker: boolean;
    kubernetes: boolean;
    orchestration: string;
  };
  autoScaling: {
    enabled: boolean;
    triggers: string[];
    minInstances: number;
    maxInstances: number;
  };
}

// ============================================
// 6. TESTING REQUIREMENTS
// ============================================
export interface TestingRequirements {
  testCoverage: {
    unitTest: number; // percentage
    integrationTest: number;
    e2eTest: number;
    targetCoverage: number;
  };
  automatedTesting: {
    framework: string;
    ciIntegration: boolean;
    testDataStrategy: string;
  };
  performanceTesting: {
    loadTesting: boolean;
    stressTesting: boolean;
    enduranceTesting: boolean;
    tools: string[];
    scenarios: string[];
  };
  securityTesting: {
    owaspTop10: boolean;
    penetrationTesting: boolean;
    frequency: string;
    tools: string[];
  };
  uat: {
    required: boolean;
    participants: string[];
    acceptanceCriteria: string;
    signOffProcess: string;
  };
  testData: {
    source: 'mock' | 'anonymized_prod' | 'synthetic';
    refreshStrategy: string;
  };
}

// ============================================
// 7. OPERATIONAL REQUIREMENTS
// ============================================
export interface OperationalRequirements {
  monitoring: {
    apmTool?: string; // e.g., "New Relic", "Datadog"
    customDashboards: boolean;
    metrics: string[];
    alerting: {
      channels: ('email' | 'sms' | 'slack' | 'pagerduty')[];
      escalationPolicy: string;
    };
  };
  logging: {
    centralized: boolean;
    tool?: string; // e.g., "ELK", "Splunk"
    logLevel: string;
    retention: number; // days
  };
  supportSLA: {
    severityLevels: {
      level: 'critical' | 'high' | 'medium' | 'low';
      responseTime: string;
      resolutionTime: string;
    }[];
    supportHours: string;
    channels: string[];
  };
  runbooks: {
    incidentResponse: boolean;
    troubleshootingGuides: boolean;
    escalationProcedures: boolean;
  };
  changeManagement: {
    approvalProcess: string;
    changeWindow: string;
    rollbackPlan: boolean;
  };
}

// ============================================
// 8. BUSINESS CONSTRAINTS
// ============================================
export interface BusinessConstraints {
  budget: {
    totalBudget: number;
    currency: string;
    phaseWiseAllocation: {
      phase: string;
      amount: number;
    }[];
  };
  timeline: {
    goLiveDate?: string;
    milestones: {
      name: string;
      deadline: string;
    }[];
    criticalPath: string;
  };
  resources: {
    teamSize: number;
    skillMatrix: {
      role: string;
      count: number;
      skills: string[];
    }[];
    availability: string;
  };
  technologyStack: {
    mandated: string[];
    restricted: string[];
    preferredVendors: string[];
  };
  approvals: {
    stakeholders: string[];
    approvalStages: string[];
    signOffCriteria: string;
  };
}

// ============================================
// 9. USER EXPERIENCE REQUIREMENTS
// ============================================
export interface UXRequirements {
  branding: {
    colorScheme: string;
    typography: string;
    logoUsage: string;
    brandGuidelines: string;
  };
  designSystem: {
    framework: 'material' | 'bootstrap' | 'tailwind' | 'custom' | 'other';
    componentLibrary: string;
    designTokens: boolean;
  };
  accessibility: {
    level: 'WCAG 2.1 A' | 'WCAG 2.1 AA' | 'WCAG 2.1 AAA';
    screenReaderSupport: boolean;
    keyboardNavigation: boolean;
    colorContrastRatio: string;
  };
  localization: {
    languages: string[];
    translationWorkflow: string;
    rtlSupport: boolean;
  };
  responsiveDesign: {
    breakpoints: {
      desktop: string;
      tablet: string;
      mobile: string;
    };
    mobileFirst: boolean;
  };
  browserSupport: {
    browsers: {
      name: string;
      version: string;
    }[];
  };
}

// ============================================
// 10. TRAINING & DOCUMENTATION
// ============================================
export interface TrainingDocumentation {
  userTraining: {
    required: boolean;
    materials: ('videos' | 'pdfs' | 'interactive' | 'workshops')[];
    duration: string;
    audience: string[];
  };
  adminTraining: {
    required: boolean;
    topics: string[];
    duration: string;
  };
  technicalDocumentation: {
    architecture: boolean;
    apiDocs: boolean;
    deploymentGuides: boolean;
    databaseSchema: boolean;
  };
  knowledgeBase: {
    faqs: boolean;
    howToGuides: boolean;
    troubleshooting: boolean;
    platform: string;
  };
}

// ============================================
// 11. CLOUD GOVERNANCE & CONTROLS
// ============================================
export interface CloudGovernanceRequirements {
  costManagement: {
    monthlyBudget: string;
    costAlerts: boolean;
    reservedInstancesStrategy: string;
    spotInstancesAllowed: boolean;
    costAllocationTags: string[];
  };
  resourceGovernance: {
    namingConvention: string;
    taggingPolicy: {
      requiredTags: string[]; // e.g., ["Environment", "Owner", "Project"]
      optionalTags: string[];
    };
    organizationStructure: string; // e.g., "Multi-account with AWS Organizations"
  };
  cloudSecurity: {
    iamStrategy: string;
    vpcDesign: string;
    encryptionAtRest: boolean;
    encryptionInTransit: boolean;
    secretsManagement: string; // e.g., "AWS Secrets Manager", "Azure Key Vault"
  };
  compliance: {
    frameworks: ('CIS' | 'AWS Well-Architected' | 'Azure CAF' | 'GCP Best Practices' | 'Other')[];
    auditLogging: boolean;
    complianceReports: string;
  };
  dataResidency: {
    allowedRegions: string[];
    dataClassification: string;
    crossRegionReplication: boolean;
  };
  disasterRecovery: {
    backupStrategy: string;
    multiRegionDeployment: boolean;
    failoverAutomation: boolean;
  };
  networking: {
    multiVpc: boolean;
    vpcPeering: boolean;
    transitGateway: boolean;
    privateSubnets: boolean;
  };
}

// ============================================
// 12. REPORTING & ANALYTICS
// ============================================
export interface ReportingAnalytics {
  businessReports: {
    name: string;
    type: 'dashboard' | 'report' | 'kpi';
    frequency: string;
    audience: string[];
  }[];
  operationalReports: {
    systemHealth: boolean;
    usageStatistics: boolean;
    performanceMetrics: boolean;
  };
  complianceReports: {
    auditReports: boolean;
    regulatoryFilings: boolean;
    frequency: string;
  };
  exportFormats: ('pdf' | 'excel' | 'csv' | 'json')[];
  realTimeAnalytics: boolean;
}

// ============================================
// MASTER COMPREHENSIVE REQUIREMENTS
// ============================================
export interface ComprehensiveRequirements {
  projectId: string;
  projectName: string;
  industry: string;
  projectType: string;
  capturedAt: string;
  capturedBy: string;
  
  // All requirement categories
  nonFunctional?: NonFunctionalRequirements;
  security?: SecurityRequirements;
  integration?: IntegrationRequirements;
  data?: DataRequirements;
  infrastructure?: InfrastructureRequirements;
  testing?: TestingRequirements;
  operations?: OperationalRequirements;
  businessConstraints?: BusinessConstraints;
  ux?: UXRequirements;
  training?: TrainingDocumentation;
  cloudGovernance?: CloudGovernanceRequirements;
  reporting?: ReportingAnalytics;
  
  // Metadata
  completionPercentage: number;
  lastUpdated: string;
}
