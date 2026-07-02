// Smart Questionnaires with Conditional Logic
import { Questionnaire } from '../types/business-requirements';

export const QUESTIONNAIRES: Record<string, Questionnaire> = {
  // Discovery Questionnaire
  business_discovery: {
    id: 'business_discovery',
    name: 'Business Discovery',
    category: 'discovery',
    questions: [
      {
        id: 'q1_industry',
        text: 'What industry does this project belong to?',
        type: 'single_choice',
        options: [
          { value: 'healthcare', label: 'Healthcare & Medical', icon: '🏥', description: 'Hospitals, clinics, medical services' },
          { value: 'banking', label: 'Banking & Finance', icon: '🏦', description: 'Banks, fintech, financial services' },
          { value: 'ecommerce', label: 'E-commerce & Retail', icon: '🛒', description: 'Online stores, retail platforms' },
          { value: 'manufacturing', label: 'Manufacturing', icon: '🏭', description: 'Production, supply chain' },
          { value: 'insurance', label: 'Insurance', icon: '🛡️', description: 'Insurance providers' },
          { value: 'logistics', label: 'Logistics & Transportation', icon: '🚚', description: 'Shipping, freight, delivery' },
          { value: 'custom', label: 'Other/Custom', icon: '⚙️', description: 'Custom industry' },
        ],
        validation: { required: true },
      },
      {
        id: 'q2_project_type',
        text: 'What type of system are you building?',
        type: 'multiple_choice',
        options: [
          { value: 'web_app', label: 'Web Application', icon: '🌐' },
          { value: 'mobile_app', label: 'Mobile Application', icon: '📱' },
          { value: 'api', label: 'Backend API/Microservices', icon: '🔌' },
          { value: 'data_platform', label: 'Data Platform/Pipeline', icon: '📊' },
          { value: 'integration', label: 'Integration Layer', icon: '🔗' },
          { value: 'portal', label: 'Customer/Partner Portal', icon: '🚪' },
        ],
        validation: { required: true },
      },
      {
        id: 'q3_user_volume',
        text: 'Expected number of users?',
        type: 'single_choice',
        options: [
          { value: 'small', label: '< 1,000 users', autoSuggests: [{ type: 'artifact', value: 'Simple Architecture', reason: 'Low volume allows simpler setup', priority: 'medium' }] },
          { value: 'medium', label: '1,000 - 50,000 users', autoSuggests: [{ type: 'capability', value: 'Caching', reason: 'Medium traffic benefits from caching', priority: 'medium' }] },
          { value: 'large', label: '50,000 - 1M users', autoSuggests: [{ type: 'capability', value: 'Load Balancing', reason: 'High traffic requires load balancing', priority: 'high' }, { type: 'capability', value: 'CDN', reason: 'Improve performance with CDN', priority: 'high' }] },
          { value: 'enterprise', label: '> 1M users', autoSuggests: [{ type: 'capability', value: 'Auto-scaling', reason: 'Enterprise scale needs auto-scaling', priority: 'high' }, { type: 'capability', value: 'Multi-region Deployment', reason: 'Global users need multi-region', priority: 'high' }] },
        ],
        validation: { required: true },
      },
      {
        id: 'q4_data_sensitivity',
        text: 'Does the system handle sensitive data?',
        type: 'multiple_choice',
        options: [
          { 
            value: 'pii', 
            label: 'Personal Identifiable Information (PII)', 
            triggers: ['q_gdpr', 'q_encryption'],
            autoSuggests: [
              { type: 'compliance', value: 'Data Privacy Compliance', reason: 'PII requires privacy measures', priority: 'high' },
              { type: 'capability', value: 'Data Masking', reason: 'Protect PII in logs/reports', priority: 'high' },
            ]
          },
          { 
            value: 'payment', 
            label: 'Payment/Credit Card Data', 
            triggers: ['q_pci'],
            autoSuggests: [
              { type: 'compliance', value: 'PCI-DSS Compliance', reason: 'Payment data requires PCI certification', priority: 'high' },
              { type: 'integration', value: 'Tokenization Service', reason: 'Never store raw card data', priority: 'high' },
            ]
          },
          { 
            value: 'health', 
            label: 'Health/Medical Records', 
            triggers: ['q_hipaa'],
            autoSuggests: [
              { type: 'compliance', value: 'HIPAA Compliance', reason: 'PHI requires HIPAA controls', priority: 'high' },
              { type: 'capability', value: 'Audit Logging', reason: 'Track all PHI access', priority: 'high' },
            ]
          },
          { 
            value: 'financial', 
            label: 'Financial Records', 
            triggers: ['q_sox'],
            autoSuggests: [{ type: 'compliance', value: 'SOX Compliance', reason: 'Financial data needs SOX controls', priority: 'high' }]
          },
          { value: 'none', label: 'No sensitive data', autoSuggests: [] },
        ],
        validation: { required: true },
      },
      {
        id: 'q5_integrations',
        text: 'What external systems need to be integrated?',
        type: 'multiple_choice',
        options: [
          { value: 'payment_gateway', label: 'Payment Gateways', triggers: ['q_payment_providers'] },
          { value: 'crm', label: 'CRM Systems (Salesforce, HubSpot)', autoSuggests: [{ type: 'integration', value: 'CRM Connector', reason: 'Sync customer data', priority: 'medium' }] },
          { value: 'erp', label: 'ERP Systems (SAP, Oracle)', autoSuggests: [{ type: 'integration', value: 'ERP Adapter', reason: 'Enterprise data sync', priority: 'high' }] },
          { value: 'email', label: 'Email Services (SendGrid, SES)', autoSuggests: [{ type: 'capability', value: 'Email Templates', reason: 'Manage email content', priority: 'low' }] },
          { value: 'sms', label: 'SMS/Notification Services', autoSuggests: [{ type: 'capability', value: 'Notification Manager', reason: 'Track notification status', priority: 'low' }] },
          { value: 'storage', label: 'Cloud Storage (S3, Azure Blob)', autoSuggests: [{ type: 'capability', value: 'File Manager', reason: 'Organize uploaded files', priority: 'medium' }] },
          { value: 'analytics', label: 'Analytics (Google Analytics, Mixpanel)', autoSuggests: [{ type: 'capability', value: 'Event Tracking', reason: 'Capture user events', priority: 'low' }] },
          { value: 'none', label: 'No integrations needed' },
        ],
      },
    ],
  },

  // Payment-specific questionnaire (triggered conditionally)
  payment_details: {
    id: 'payment_details',
    name: 'Payment Requirements',
    category: 'technical',
    questions: [
      {
        id: 'q_payment_providers',
        text: 'Which payment providers do you need to support?',
        type: 'multiple_choice',
        options: [
          { value: 'stripe', label: 'Stripe', autoSuggests: [{ type: 'integration', value: 'Stripe SDK', reason: 'Official integration', priority: 'high' }] },
          { value: 'paypal', label: 'PayPal', autoSuggests: [{ type: 'integration', value: 'PayPal SDK', reason: 'Official integration', priority: 'high' }] },
          { value: 'square', label: 'Square', autoSuggests: [{ type: 'integration', value: 'Square SDK', reason: 'Official integration', priority: 'high' }] },
          { value: 'braintree', label: 'Braintree', autoSuggests: [{ type: 'integration', value: 'Braintree SDK', reason: 'Official integration', priority: 'high' }] },
          { value: 'custom', label: 'Custom/Regional Gateway' },
        ],
        conditionalLogic: {
          showIf: [{ questionId: 'q5_integrations', operator: 'contains', value: 'payment_gateway' }],
          operator: 'AND',
        },
      },
      {
        id: 'q_payment_methods',
        text: 'What payment methods should be supported?',
        type: 'multiple_choice',
        options: [
          { value: 'credit_card', label: 'Credit/Debit Cards' },
          { value: 'bank_transfer', label: 'Bank Transfer/ACH' },
          { value: 'wallet', label: 'Digital Wallets (Apple Pay, Google Pay)' },
          { value: 'crypto', label: 'Cryptocurrency', autoSuggests: [{ type: 'integration', value: 'Crypto Payment Processor', reason: 'Handle crypto payments', priority: 'medium' }] },
          { value: 'invoice', label: 'Invoice/Net Terms' },
        ],
      },
      {
        id: 'q_payment_features',
        text: 'What payment features are required?',
        type: 'multiple_choice',
        options: [
          { value: 'recurring', label: 'Recurring/Subscription Billing', autoSuggests: [{ type: 'capability', value: 'Subscription Manager', reason: 'Manage recurring payments', priority: 'high' }] },
          { value: 'refunds', label: 'Refunds/Chargebacks', autoSuggests: [{ type: 'capability', value: 'Refund Processor', reason: 'Handle refund logic', priority: 'medium' }] },
          { value: 'split', label: 'Split Payments/Multi-vendor', autoSuggests: [{ type: 'capability', value: 'Payment Splitting', reason: 'Complex payment distribution', priority: 'high' }] },
          { value: 'fraud', label: 'Fraud Detection', autoSuggests: [{ type: 'integration', value: 'Fraud Detection Service', reason: 'Prevent fraudulent transactions', priority: 'high' }] },
          { value: 'reconciliation', label: 'Payment Reconciliation', autoSuggests: [{ type: 'capability', value: 'Reconciliation Engine', reason: 'Match payments with orders', priority: 'medium' }] },
        ],
      },
    ],
  },

  // Compliance questionnaires
  hipaa_compliance: {
    id: 'hipaa_compliance',
    name: 'HIPAA Compliance',
    category: 'compliance',
    industry: 'healthcare',
    questions: [
      {
        id: 'q_hipaa_phi',
        text: 'What types of Protected Health Information (PHI) will be stored?',
        type: 'multiple_choice',
        options: [
          { value: 'demographics', label: 'Patient Demographics', autoSuggests: [{ type: 'capability', value: 'Field-level Encryption', reason: 'Encrypt sensitive fields', priority: 'high' }] },
          { value: 'clinical', label: 'Clinical Notes/Records', autoSuggests: [{ type: 'capability', value: 'Access Control', reason: 'Role-based PHI access', priority: 'high' }] },
          { value: 'lab_results', label: 'Lab Results', autoSuggests: [{ type: 'capability', value: 'Audit Logging', reason: 'Track all result access', priority: 'high' }] },
          { value: 'imaging', label: 'Medical Imaging', autoSuggests: [{ type: 'capability', value: 'Secure File Storage', reason: 'HIPAA-compliant file storage', priority: 'high' }] },
          { value: 'prescriptions', label: 'Prescription Data', autoSuggests: [{ type: 'compliance', value: 'EPCS Compliance', reason: 'Electronic prescriptions', priority: 'high' }] },
        ],
        validation: { required: true },
      },
      {
        id: 'q_hipaa_baa',
        text: 'Will you need Business Associate Agreements (BAA) with vendors?',
        type: 'boolean',
        helpText: 'Required if third-party services process PHI',
        autoSuggests: [{ type: 'compliance', value: 'BAA Documentation', reason: 'Legal requirement for PHI processing', priority: 'high' }],
      },
      {
        id: 'q_hipaa_breach',
        text: 'Do you have breach notification procedures in place?',
        type: 'boolean',
        helpText: 'HIPAA requires notification within 60 days of breach discovery',
        autoSuggests: [{ type: 'capability', value: 'Breach Detection & Notification', reason: 'Compliance requirement', priority: 'high' }],
      },
    ],
  },

  pci_compliance: {
    id: 'pci_compliance',
    name: 'PCI-DSS Compliance',
    category: 'compliance',
    questions: [
      {
        id: 'q_pci_cardholder_data',
        text: 'How will cardholder data be handled?',
        type: 'single_choice',
        options: [
          { 
            value: 'tokenization', 
            label: 'Tokenization (Recommended)', 
            description: 'Store tokens, never raw card data',
            autoSuggests: [{ type: 'integration', value: 'Payment Tokenization', reason: 'Avoid PCI scope', priority: 'high' }]
          },
          { 
            value: 'storage', 
            label: 'Store Encrypted Data', 
            description: 'Full PCI-DSS compliance required',
            autoSuggests: [
              { type: 'compliance', value: 'PCI-DSS Level 1 Certification', reason: 'Storing card data requires certification', priority: 'high' },
              { type: 'capability', value: 'Key Management System', reason: 'Secure encryption keys', priority: 'high' },
            ]
          },
          { 
            value: 'passthrough', 
            label: 'Pass-through Only', 
            description: 'Never touch card data',
            autoSuggests: [{ type: 'integration', value: 'Hosted Payment Page', reason: 'Reduce PCI scope', priority: 'high' }]
          },
        ],
        validation: { required: true },
      },
      {
        id: 'q_pci_transaction_volume',
        text: 'Expected annual transaction volume?',
        type: 'single_choice',
        options: [
          { value: 'level4', label: '< 20,000 transactions', description: 'PCI Level 4' },
          { value: 'level3', label: '20,000 - 1M transactions', description: 'PCI Level 3' },
          { value: 'level2', label: '1M - 6M transactions', description: 'PCI Level 2' },
          { value: 'level1', label: '> 6M transactions', description: 'PCI Level 1 - Requires annual audit' },
        ],
      },
    ],
  },

  // Technical questionnaire
  scale_expectations: {
    id: 'scale_expectations',
    name: 'Scale & Performance',
    category: 'technical',
    questions: [
      {
        id: 'q_scale_concurrent',
        text: 'Expected peak concurrent users?',
        type: 'single_choice',
        options: [
          { value: '100', label: '< 100', autoSuggests: [{ type: 'artifact', value: 'Single Server', reason: 'Low concurrency', priority: 'low' }] },
          { value: '1000', label: '100 - 1,000', autoSuggests: [{ type: 'capability', value: 'Connection Pooling', reason: 'Optimize connections', priority: 'medium' }] },
          { value: '10000', label: '1,000 - 10,000', autoSuggests: [{ type: 'capability', value: 'Horizontal Scaling', reason: 'Scale across servers', priority: 'high' }] },
          { value: '100000', label: '> 10,000', autoSuggests: [{ type: 'capability', value: 'Microservices Architecture', reason: 'Independent scaling', priority: 'high' }] },
        ],
      },
      {
        id: 'q_scale_response_time',
        text: 'Required API response time?',
        type: 'single_choice',
        options: [
          { value: '5000', label: '< 5 seconds (Standard)' },
          { value: '1000', label: '< 1 second (Fast)', autoSuggests: [{ type: 'capability', value: 'Caching Layer', reason: 'Meet response time SLA', priority: 'high' }] },
          { value: '200', label: '< 200ms (Real-time)', autoSuggests: [{ type: 'capability', value: 'Edge Computing', reason: 'Ultra-low latency', priority: 'high' }] },
        ],
      },
      {
        id: 'q_scale_data_volume',
        text: 'Expected data volume?',
        type: 'single_choice',
        options: [
          { value: 'small', label: '< 100GB', autoSuggests: [{ type: 'artifact', value: 'Relational Database', reason: 'Standard RDBMS works', priority: 'medium' }] },
          { value: 'medium', label: '100GB - 10TB', autoSuggests: [{ type: 'capability', value: 'Database Partitioning', reason: 'Manage large datasets', priority: 'medium' }] },
          { value: 'large', label: '> 10TB', autoSuggests: [{ type: 'capability', value: 'Distributed Database', reason: 'Scale data storage', priority: 'high' }] },
        ],
      },
    ],
  },
};

export function getQuestionnaire(id: string): Questionnaire | undefined {
  return QUESTIONNAIRES[id];
}

export function getQuestionnairesByCategory(category: string): Questionnaire[] {
  return Object.values(QUESTIONNAIRES).filter((q) => q.category === category);
}

export function getQuestionnairesByIndustry(industry: string): Questionnaire[] {
  return Object.values(QUESTIONNAIRES).filter((q) => !q.industry || q.industry === industry);
}
