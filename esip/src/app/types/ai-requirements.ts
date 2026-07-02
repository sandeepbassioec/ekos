/**
 * AI-Driven Requirements Analysis Types
 * Core requirements, gaps, confusions, security, threats, NFRs
 */

// ============================================
// CORE REQUIREMENTS (AI-Extracted)
// ============================================

export type AIRequirementType = 'Functional' | 'Non-Functional' | 'Security' | 'Performance' | 'Compliance' | 'Other';
export type RequirementPriority = 'Critical' | 'High' | 'Medium' | 'Low';
export type RequirementStatus = 'Draft' | 'Reviewed' | 'Approved' | 'Implemented';

export interface CoreRequirement {
  id: string;
  roadmapItemId: string;
  requirement: string;
  type: AIRequirementType;
  priority: RequirementPriority;
  status: RequirementStatus;
  source: RequirementSource;
  extractedBy: 'AI' | 'Manual';
  confidence: number; // 0-100 for AI-extracted requirements
  createdAt: string;
  updatedAt: string;
}

export interface RequirementSource {
  type: 'document' | 'meeting' | 'email' | 'chat' | 'manual';
  id?: string; // ID of source document/meeting/email/chat
  reference?: string; // Page number, timestamp, etc.
}

// ============================================
// GAP ANALYSIS
// ============================================

export type GapSeverity = 'Critical' | 'High' | 'Medium' | 'Low';
export type GapStatus = 'Open' | 'In Progress' | 'Resolved' | 'Closed';

export interface Gap {
  id: string;
  roadmapItemId: string;
  title: string;
  description: string;
  severity: GapSeverity;
  status: GapStatus;
  source: RequirementSource;
  identifiedBy: 'AI' | 'Manual';
  suggestions: string[];
  resolution?: string;
  resolvedAt?: string;
  createdAt: string;
  updatedAt: string;
}

// ============================================
// CONFUSIONS
// ============================================

export type ConfusionStatus = 'Open' | 'Clarified' | 'Resolved';

export interface Confusion {
  id: string;
  roadmapItemId: string;
  question: string;
  context: string;
  sources: RequirementSource[]; // Can have multiple conflicting sources
  status: ConfusionStatus;
  clarification?: string;
  resolvedBy?: string;
  resolvedAt?: string;
  createdAt: string;
  updatedAt: string;
}

// ============================================
// SUGGESTIONS
// ============================================

export type SuggestionType = 'Enhancement' | 'Alternative' | 'Best Practice' | 'Risk Mitigation';
export type SuggestionStatus = 'Pending' | 'Accepted' | 'Rejected' | 'Implemented';

export interface Suggestion {
  id: string;
  roadmapItemId: string;
  title: string;
  description: string;
  type: SuggestionType;
  status: SuggestionStatus;
  generatedBy: 'AI' | 'Manual';
  impact?: string;
  effort?: string;
  response?: string;
  createdAt: string;
  updatedAt: string;
}

// ============================================
// SECURITY REQUIREMENTS
// ============================================

export type SecurityCategory = 'Authentication' | 'Authorization' | 'Data Protection' | 'Network Security' | 'Compliance' | 'Other';
export type SecurityLevel = 'Critical' | 'High' | 'Medium' | 'Low';

export interface SecurityRequirement {
  id: string;
  roadmapItemId: string;
  title: string;
  description: string;
  category: SecurityCategory;
  level: SecurityLevel;
  standard?: string; // e.g., "OWASP Top 10", "ISO 27001"
  controls: string[];
  testCriteria: string[];
  status: 'Draft' | 'Reviewed' | 'Approved';
  createdAt: string;
  updatedAt: string;
}

// ============================================
// THREAT ANALYSIS
// ============================================

export type ThreatLevel = 'Critical' | 'High' | 'Medium' | 'Low';
export type ThreatLikelihood = 'Very High' | 'High' | 'Medium' | 'Low' | 'Very Low';
export type ThreatCategory = 'Spoofing' | 'Tampering' | 'Repudiation' | 'Information Disclosure' | 'Denial of Service' | 'Elevation of Privilege';

export interface Threat {
  id: string;
  roadmapItemId: string;
  threat: string;
  description: string;
  category: ThreatCategory; // STRIDE categories
  level: ThreatLevel;
  likelihood: ThreatLikelihood;
  impact: string;
  mitigations: string[];
  status: 'Identified' | 'Mitigated' | 'Accepted' | 'Transferred';
  createdAt: string;
  updatedAt: string;
}

// ============================================
// NON-FUNCTIONAL REQUIREMENTS (NFR)
// ============================================

export type NFRCategory = 'Performance' | 'Scalability' | 'Reliability' | 'Availability' | 'Maintainability' | 'Usability' | 'Compliance';

export interface NonFunctionalRequirement {
  id: string;
  roadmapItemId: string;
  category: NFRCategory;
  requirement: string;
  metric: string; // e.g., "Response time", "Uptime", "Concurrent users"
  target: string; // e.g., "< 200ms", "99.9%", "10,000 users"
  measurement: string; // How to measure it
  priority: RequirementPriority;
  status: 'Draft' | 'Approved' | 'Met' | 'Not Met';
  createdAt: string;
  updatedAt: string;
}

// ============================================
// AI ANALYSIS
// ============================================

export interface AIAnalysisRequest {
  roadmapItemId: string;
  sources: {
    documents: string[]; // Document IDs
    meetings: string[]; // Meeting IDs
    emails: string[]; // Email IDs
    chats: string[]; // Chat IDs
  };
}

export interface AIAnalysisResult {
  coreRequirements: Omit<CoreRequirement, 'id' | 'createdAt' | 'updatedAt'>[];
  gaps: Omit<Gap, 'id' | 'createdAt' | 'updatedAt'>[];
  confusions: Omit<Confusion, 'id' | 'createdAt' | 'updatedAt'>[];
  suggestions: Omit<Suggestion, 'id' | 'createdAt' | 'updatedAt'>[];
  securityRequirements: Omit<SecurityRequirement, 'id' | 'createdAt' | 'updatedAt'>[];
  threats: Omit<Threat, 'id' | 'createdAt' | 'updatedAt'>[];
  nfrs: Omit<NonFunctionalRequirement, 'id' | 'createdAt' | 'updatedAt'>[];
}

// ============================================
// RESOLUTION
// ============================================

export interface Resolution {
  id: string;
  roadmapItemId: string;
  targetType: 'gap' | 'confusion';
  targetId: string;
  answer: string;
  refinedRequirements: string[]; // AI-generated refined requirements based on answer
  createdAt: string;
}
