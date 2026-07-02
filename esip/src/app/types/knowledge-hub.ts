/**
 * Knowledge Hub Types
 * Multi-document BRDs, Teams meetings, Emails, Chats for Roadmap Items
 */

// ============================================
// DOCUMENT TYPES
// ============================================

export type DocumentType = 'BRD' | 'Specification' | 'Technical Doc' | 'Architecture Doc' | 'Other';

export interface Document {
  id: string;
  roadmapItemId: string;
  name: string;
  type: DocumentType;
  version?: string;
  fileUrl: string;
  fileSize?: number;
  mimeType?: string;
  extractedText?: string;
  uploadedBy: string;
  uploadedAt: string;
  updatedAt: string;
}

export interface CreateDocumentDTO {
  roadmapItemId: string;
  name: string;
  type: DocumentType;
  version?: string;
  file: File;
}

export interface UpdateDocumentDTO {
  name?: string;
  type?: DocumentType;
  version?: string;
}

// ============================================
// MEETING TYPES
// ============================================

export interface Meeting {
  id: string;
  roadmapItemId: string;
  title: string;
  date: string;
  recordingUrl?: string;
  transcript: string;
  participants: string[];
  keyDecisions: string[];
  notes?: string;
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreateMeetingDTO {
  roadmapItemId: string;
  title: string;
  date: string;
  transcript: string;
  participants: string[];
  keyDecisions?: string[];
  notes?: string;
  recordingFile?: File;
}

export interface UpdateMeetingDTO {
  title?: string;
  date?: string;
  transcript?: string;
  participants?: string[];
  keyDecisions?: string[];
  notes?: string;
}

// ============================================
// EMAIL TYPES
// ============================================

export interface Email {
  id: string;
  roadmapItemId: string;
  subject: string;
  from: string;
  to: string[];
  cc?: string[];
  date: string;
  body: string;
  attachments?: string[];
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreateEmailDTO {
  roadmapItemId: string;
  subject: string;
  from: string;
  to: string[];
  cc?: string[];
  date: string;
  body: string;
}

export interface UpdateEmailDTO {
  subject?: string;
  from?: string;
  to?: string[];
  cc?: string[];
  date?: string;
  body?: string;
}

// ============================================
// CHAT TYPES
// ============================================

export type ChatPlatform = 'Teams' | 'Slack' | 'Other';

export interface Chat {
  id: string;
  roadmapItemId: string;
  platform: ChatPlatform;
  title?: string;
  date: string;
  participants: string[];
  messages: string;
  channelName?: string;
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreateChatDTO {
  roadmapItemId: string;
  platform: ChatPlatform;
  title?: string;
  date: string;
  participants: string[];
  messages: string;
  channelName?: string;
}

export interface UpdateChatDTO {
  platform?: ChatPlatform;
  title?: string;
  date?: string;
  participants?: string[];
  messages?: string;
  channelName?: string;
}

// ============================================
// AI REQUIREMENT TYPES
// ============================================

export type SourceType = 'Document' | 'Meeting' | 'Email' | 'Chat';
export type RequirementCategory = 'Functional' | 'Non-Functional' | 'Security' | 'Performance' | 'Other';
export type RequirementPriority = 'High' | 'Medium' | 'Low';

export interface AIRequirement {
  id: string;
  roadmapItemId: string;
  requirementKey?: string;
  requirement: string;
  description?: string;
  technicalCategory?: string;
  changeType?: string;
  rationale?: string;
  type?: string;
  source: string;
  sourceType: SourceType;
  sourceId?: string;
  category?: RequirementCategory;
  priority?: RequirementPriority;
  confidence?: number;
  acceptanceCriteria?: string[];
  businessValue?: string;
  testScenarios?: string[];
  qualityIssues?: string[];
  assumptions?: string[];
  stakeholders?: string[];
  complianceFrameworks?: string[];
  domain?: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreateAIRequirementDTO {
  roadmapItemId: string;
  requirementKey?: string;
  requirement: string;
  description?: string;
  technicalCategory?: string;
  changeType?: string;
  rationale?: string;
  type?: string;
  source: string;
  sourceType: SourceType;
  sourceId?: string;
  category?: RequirementCategory;
  priority?: RequirementPriority;
  confidence?: number;
  acceptanceCriteria?: string[];
  businessValue?: string;
  testScenarios?: string[];
  qualityIssues?: string[];
  assumptions?: string[];
  stakeholders?: string[];
  complianceFrameworks?: string[];
  domain?: string;
}

// ============================================
// KNOWLEDGE HUB AGGREGATE
// ============================================

export interface KnowledgeHub {
  roadmapItemId: string;
  documents: Document[];
  meetings: Meeting[];
  emails: Email[];
  chats: Chat[];
  aiRequirements: AIRequirement[];
  gaps?: any[];
  confusions?: any[];
  suggestions?: any[];
  securityRequirements?: any[];
  threats?: any[];
  nfrs?: any[];
  cloudGovernance?: any[];
  formalRequirements?: any[];
  conflicts?: any[];
  stats: {
    totalDocuments: number;
    totalMeetings: number;
    totalEmails: number;
    totalChats: number;
    totalRequirements: number;
    totalConflicts: number;
    coverageScore: number;
  };
}

// ============================================
// SEARCH TYPES
// ============================================

export interface SearchQuery {
  roadmapItemId: string;
  query: string;
  contentTypes?: ('documents' | 'meetings' | 'emails' | 'chats')[];
}

export interface SearchResult {
  id: string;
  type: 'document' | 'meeting' | 'email' | 'chat';
  title: string;
  snippet: string;
  source: string;
  date: string;
  relevance: number;
}

export interface AISearchResponse {
  answer: string;
  sources: SearchResult[];
  confidence: number;
}
