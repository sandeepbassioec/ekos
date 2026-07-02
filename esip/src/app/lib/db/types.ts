/**
 * Database Abstraction Layer - Type Definitions
 */

// Auth Types
export interface AuthUser {
  id: string;
  email: string;
  fullName?: string;
  role?: string;
  licenseTier?: string; // Epic G — seat/license tier (viewer|reviewer|contributor|enterprise)
  status?: string; // active | disabled
}

// ============================================
// Accounts, licensing & invites (Epic G / catalog Vol 13 + 17)
// ============================================
export type LicenseTier = 'viewer' | 'reviewer' | 'contributor' | 'enterprise';
export type AccountStatus = 'active' | 'disabled';
export type InviteStatus = 'pending' | 'accepted' | 'revoked';

/** The `users/{uid}` profile doc — the source of truth for authorization (Firebase Auth can't hold custom claims client-side). */
export interface UserProfile {
  id: string; // = Firebase Auth uid
  email: string;
  fullName?: string;
  role?: string;
  licenseTier?: LicenseTier;
  status?: AccountStatus;
  defaultPersonaId?: string; // primary persona assigned to this user
  invitedBy?: string;
  createdAt?: string;
  updatedAt?: string;
}

/** An admin-issued invitation; the invitee activates it to create their account. */
export interface Invite {
  id: string;
  email: string;
  role: string;
  licenseTier: LicenseTier;
  defaultPersonaId?: string;
  token: string;
  status: InviteStatus;
  invitedByName?: string;
  createdAt: string;
  updatedAt: string;
}

export interface AuthSession {
  access_token: string;
  user: AuthUser;
}

// Project Types
export type ProjectOnboardingMode = 'new' | 'active' | 'retrospective';

export interface Project {
  id: string;
  userId: string;
  name: string;
  description?: string;
  status: 'draft' | 'in_progress' | 'completed' | 'archived';
  estimationTechnique?: string;
  teamId?: string;
  onboardingMode?: ProjectOnboardingMode;
  createdAt: string;
  updatedAt: string;
}

// ============================================
// Project Team Members
// ============================================
export type ProjectRole =
  | 'delivery_manager'
  | 'product_owner'
  | 'tech_lead'
  | 'qa_lead'
  | 'developer'
  | 'scrum_master'
  | 'stakeholder';

export const PROJECT_ROLE_LABELS: Record<ProjectRole, string> = {
  delivery_manager: 'Delivery Manager',
  product_owner: 'Product Owner',
  tech_lead: 'Tech Lead',
  qa_lead: 'QA Lead',
  developer: 'Developer',
  scrum_master: 'Scrum Master',
  stakeholder: 'Stakeholder',
};

export const PROJECT_ROLE_DESCRIPTIONS: Record<ProjectRole, string> = {
  delivery_manager: 'Creates & manages PIs and Sprints. PI confidence tracking.',
  product_owner: 'Finalises requirements, allocates to PI, approves Functional & Compliance stories.',
  tech_lead: 'Approves Technical, DevOps & Infrastructure stories. Flags cross-story dependencies.',
  qa_lead: 'Approves Testing & Automation stories. Generates BDD test cases.',
  developer: 'Works stories in sprints. Can suggest merges and edits.',
  scrum_master: 'Manages sprint health, removes blockers. No PI admin rights.',
  stakeholder: 'Read-only on PI board and stories. Can comment.',
};

export interface ProjectMember {
  id: string;
  projectId: string;
  userId: string;
  userEmail: string;
  userFullName?: string;
  projectRoles: ProjectRole[];
  addedBy: string;
  addedAt: string;
  updatedAt: string;
}

// Roadmap Types
export interface Roadmap {
  id: string;
  projectId: string;
  year: number;
  type: 'product' | 'technology' | 'platform';
  createdBy: string;
  ownerId: string;
  createdAt: string;
  updatedAt: string;
}

export interface RoadmapItem {
  id: string;
  roadmapId: string;
  title: string;
  description?: string;
  startDate: string;
  endDate: string;
  status: 'planned' | 'in_progress' | 'completed' | 'on_hold' | 'cancelled';
  priority: 'low' | 'medium' | 'high' | 'critical';
  category?: string;
  assignedTo?: string;
  dependencies: string[];
  progress: number;
  createdAt: string;
  updatedAt: string;
}

// Knowledge Hub Types
export interface Document {
  id: string;
  roadmapItemId: string;
  name: string;
  type: 'BRD' | 'FRD' | 'TRD' | 'Other';
  version?: string;
  storagePath: string;
  fileSize: number;
  extractedText?: string;
  uploadedAt: string;
}

export interface Meeting {
  id: string;
  roadmapItemId: string;
  title: string;
  date: string;
  participants: string[];
  transcript?: string;
  notes?: string;
  keyDecisions: string[];
  createdAt: string;
}

export interface Email {
  id: string;
  roadmapItemId: string;
  subject: string;
  from: string;
  to: string[];
  body: string;
  date: string;
  createdAt: string;
}

export interface Chat {
  id: string;
  roadmapItemId: string;
  platform: 'Teams' | 'Slack' | 'Other';
  title?: string;
  channelName?: string;
  participants: string[];
  messages: string;
  date: string;
  createdAt: string;
}

export interface AIRequirement {
  id: string;
  roadmapItemId: string;
  requirementKey?: string;
  requirement: string;
  description?: string;
  technicalCategory?: string;
  changeType?: string;
  rationale?: string;
  source: string;
  sourceType: 'Document' | 'Meeting' | 'Email' | 'Chat';
  sourceId: string;
  category: 'Functional' | 'Non-Functional' | 'Security' | 'Performance' | 'Usability' | 'Scalability';
  priority: 'High' | 'Medium' | 'Low';
  confidence: number;
  extractedAt: string;
}

// ============================================
// Traceability — Lifecycle event on Gap / Confusion / Suggestion
// ============================================
export interface SourceEvent {
  id: string;
  type: 'created' | 'resolved' | 'clarified' | 'updated' | 'reopened' | 'conflict-flagged' | 'merged';
  sourceType: 'document' | 'meeting' | 'email' | 'chat' | 'manual' | 'ai-merge';
  sourceId?: string;
  sourceName?: string;
  excerpt?: string;
  actorId?: string;
  actorName?: string;
  actorType: 'ai' | 'human';
  at: number;
}

// Conflict detected when a new source contradicts an existing requirement
export interface Conflict {
  id: string;
  roadmapItemId: string;
  title: string;
  description: string;
  triggerSourceType: 'Email' | 'Meeting' | 'Chat';
  triggerSourceId: string;
  triggerSourceName: string;
  triggerExcerpt: string;
  conflictingRequirementId?: string;
  conflictingRequirementKey?: string;
  conflictingRequirementText?: string;
  severity: 'Critical' | 'High' | 'Medium';
  status: 'Open' | 'Resolved' | 'Dismissed';
  resolution?: string;
  createdAt: string;
  updatedAt: string;
}

// Redacted content shared by PO with selected team members
export interface SharedContentView {
  id: string;
  roadmapItemId: string;
  sourceType: 'Email' | 'Meeting' | 'Chat';
  sourceId: string;
  sourceName: string;
  title: string;
  redactedContent: string;
  sharedWith: string[];
  sharedByUserId: string;
  sharedByName?: string;
  note?: string;
  sharedAt: string;
  createdAt: string;
  updatedAt: string;
}

// Gap Analysis
export interface Gap {
  id: string;
  roadmapItemId: string;
  title: string;
  description: string;
  severity: 'Critical' | 'High' | 'Medium' | 'Low';
  status: 'Open' | 'In Progress' | 'Resolved' | 'Closed';
  sourceType: 'Document' | 'Meeting' | 'Email' | 'Chat' | 'AI' | 'Manual';
  sourceId?: string;
  suggestions: string[];
  resolution?: string;
  resolvedAt?: string;
  relatedRequirementIds?: string[];
  sourceEvents?: SourceEvent[];
  createdAt: string;
  updatedAt: string;
}

// Confusions
export interface Confusion {
  id: string;
  roadmapItemId: string;
  question: string;
  context: string;
  sources: Array<{ type: string; id: string; reference?: string }>;
  status: 'Open' | 'Clarified' | 'Resolved';
  clarification?: string;
  resolvedBy?: string;
  resolvedAt?: string;
  relatedRequirementIds?: string[];
  sourceEvents?: SourceEvent[];
  createdAt: string;
  updatedAt: string;
}

// Suggestions
export interface Suggestion {
  id: string;
  roadmapItemId: string;
  title: string;
  description: string;
  type: 'Enhancement' | 'Alternative' | 'Best Practice' | 'Risk Mitigation';
  status: 'Pending' | 'Accepted' | 'Rejected' | 'Implemented';
  generatedBy: 'AI' | 'Manual';
  impact?: string;
  effort?: string;
  response?: string;
  relatedRequirementIds?: string[];
  sourceEvents?: SourceEvent[];
  createdAt: string;
  updatedAt: string;
}

// Security Requirements
export interface SecurityRequirement {
  id: string;
  roadmapItemId: string;
  title: string;
  description: string;
  category: 'Authentication' | 'Authorization' | 'Data Protection' | 'Network Security' | 'Compliance' | 'Other';
  level: 'Critical' | 'High' | 'Medium' | 'Low';
  standard?: string;
  controls: string[];
  testCriteria: string[];
  status: 'Draft' | 'Reviewed' | 'Approved';
  createdAt: string;
  updatedAt: string;
}

// Threat Analysis (STRIDE)
export interface Threat {
  id: string;
  roadmapItemId: string;
  threat: string;
  description: string;
  category: 'Spoofing' | 'Tampering' | 'Repudiation' | 'Information Disclosure' | 'Denial of Service' | 'Elevation of Privilege';
  level: 'Critical' | 'High' | 'Medium' | 'Low';
  likelihood: 'Very High' | 'High' | 'Medium' | 'Low' | 'Very Low';
  impact: string;
  mitigations: string[];
  status: 'Identified' | 'Mitigated' | 'Accepted' | 'Transferred';
  createdAt: string;
  updatedAt: string;
}

// Non-Functional Requirements
export interface NonFunctionalRequirement {
  id: string;
  roadmapItemId: string;
  category: 'Performance' | 'Scalability' | 'Reliability' | 'Availability' | 'Maintainability' | 'Usability' | 'Compliance';
  requirement: string;
  metric: string;
  target: string;
  measurement: string;
  priority: 'Critical' | 'High' | 'Medium' | 'Low';
  status: 'Draft' | 'Approved' | 'Met' | 'Not Met';
  createdAt: string;
  updatedAt: string;
}

// Alias for backward compatibility
export type NFR = NonFunctionalRequirement;

// Cloud Governance Requirements
export interface CloudGovernance {
  id: string;
  roadmapItemId: string;
  area: 'Cost Management' | 'Resource Governance' | 'Cloud Security' | 'Compliance' | 'Data Residency' | 'Disaster Recovery' | 'Networking';
  requirement: string;
  details: string;
  priority: 'Critical' | 'High' | 'Medium' | 'Low';
  status: 'Draft' | 'Approved' | 'Implemented';
  createdAt: string;
  updatedAt: string;
}

// Formal Requirements (per roadmap item — manual + AI-imported)
export type FormalRequirementCategory = 'Functional' | 'Non-Functional' | 'Constraint' | 'Acceptance Criteria';

export type FunctionalType =
  | 'Feature' | 'Screen' | 'API' | 'Integration'
  | 'Bug Fix' | 'Data Migration' | 'Business Rule' | 'Other';

export type NFRType =
  | 'Performance' | 'Security' | 'Availability' | 'Scalability'
  | 'Concurrency' | 'Reliability' | 'Maintainability' | 'Usability'
  | 'Compliance' | 'Disaster Recovery' | 'Observability' | 'Other';

export type ConstraintType = 'Technical' | 'Budget' | 'Timeline' | 'Regulatory' | 'Other';

export interface FormalRequirement {
  id: string;
  roadmapItemId: string;
  projectId: string;

  // Auto-generated ID shown to user: REQ-F-001, REQ-NF-001, REQ-C-001
  requirementId: string;

  // Classification
  category: FormalRequirementCategory;
  functionalType?: FunctionalType;   // only when category = Functional
  nfrType?: NFRType;                 // only when category = Non-Functional
  constraintType?: ConstraintType;   // only when category = Constraint

  // Content
  title: string;
  description: string;

  // Priority + Status
  priority: 'Critical' | 'High' | 'Medium' | 'Low';
  status: 'Draft' | 'Under Review' | 'Approved' | 'Implemented' | 'Rejected' | 'Blocked';
  blockedBy?: { type: 'gap' | 'confusion' | 'conflict'; id: string; title: string; reason: string; blockedAt: string };

  // User Story (Functional only)
  userStory?: { asA: string; iWant: string; soThat: string };

  // Acceptance Criteria
  acceptanceCriteria: string[];

  // NFR specifics
  metric?: string;
  target?: string;
  measurement?: string;

  // Source tracking
  source: 'Manual' | 'AI';
  aiSourceType?: 'Document' | 'Meeting' | 'Email' | 'Chat' | 'Gap' | 'Confusion' | 'Suggestion';
  aiSourceId?: string;
  aiConfidence?: number;

  // Metadata
  createdAt: string;
  updatedAt: string;
}

// ============================================
// Program Increment / Sprint / project-level Requirement
// (SAFe-style planning + Comprehensive Requirements Wizard output)
// ============================================
export interface PI {
  id: string;
  projectId: string;
  name: string;
  startDate?: string;
  endDate?: string;
  status: string;
  objectives?: string;
  createdAt: string;
  updatedAt: string;
}

export interface Sprint {
  id: string;
  projectId: string;
  piId?: string;
  name: string;
  startDate?: string;
  endDate?: string;
  status: string;
  goals?: string;
  createdAt: string;
  updatedAt: string;
}

export interface Requirement {
  id: string;
  projectId: string;
  piId?: string;
  sprintId?: string;
  title: string;
  description?: string;
  category: string;
  priority: string;
  status: string;
  tags?: string[];
  metadata?: any;
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}

// ============================================
// User Stories (SAFe Step 4 — AI Story Decomposition)
// ============================================
export type StoryType =
  // ── UI / Frontend ──────────────────────────────────────────────
  | 'ui_new_screen'        // Brand-new page / route
  | 'ui_component'         // New reusable component or widget
  | 'ui_update'            // Change to an existing screen
  | 'ui_modal'             // Modal / drawer / popup / dialog
  // ── API / Backend ──────────────────────────────────────────────
  | 'api_endpoint'         // New or updated REST / GraphQL endpoint
  | 'backend_service'      // Business logic, service layer, orchestration
  | 'background_job'       // Scheduled job, queue consumer, async worker
  // ── Data ───────────────────────────────────────────────────────
  | 'database_schema'      // New table, column, index, or constraint
  | 'data_migration'       // One-time or repeatable data migration script
  // ── Integration ────────────────────────────────────────────────
  | 'integration_external' // Third-party / external API integration
  | 'integration_internal' // Internal microservice / platform API call
  // ── Testing ────────────────────────────────────────────────────
  | 'unit_test'            // Unit / component tests
  | 'integration_test'     // API / service-level integration tests
  | 'e2e_test'             // End-to-end / automation / regression tests
  // ── DevOps / Infra ─────────────────────────────────────────────
  | 'devops'               // CI/CD pipeline, deployment config, IaC, env
  // ── Cross-cutting ──────────────────────────────────────────────
  | 'security'             // Auth, authorisation, secrets, pen-test findings
  | 'performance'          // Caching, query optimisation, load testing baseline
  | 'compliance'           // Regulatory, audit trail, data-retention, GDPR/PCI
  | 'documentation';       // ADR, API docs, runbook, onboarding guide
export type StoryStatus = 'draft' | 'ready' | 'in_progress' | 'done';

export interface Story {
  id: string;
  projectId: string;
  requirementId: string;
  piId?: string;
  title: string;
  description?: string;
  type: StoryType;
  acceptanceCriteria?: string[];
  storyPoints?: number;
  status: StoryStatus;
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}

// ============================================
// Cross-role workflow requests (Vol 12 / Epic D)
// ============================================
export type WorkflowRequestType = 'clarification' | 'missing-requirement' | 'review' | 'approval' | 'hand-off' | 'staleness-alert';
export type WorkflowRequestStatus = 'open' | 'in-progress' | 'responded' | 'approved' | 'rejected' | 'resolved' | 'withdrawn';

export interface WorkflowRequest {
  id: string;
  projectId?: string;
  type: WorkflowRequestType;
  title: string;
  description?: string;
  raiserId: string; // who raised it
  raiserName?: string;
  targetRole?: string; // routed to a role …
  targetUserId?: string; // … and/or a specific user
  subjectType?: string; // optional subject artifact (requirement/deliverable/roadmap-item/confusion/gap/…)
  subjectId?: string;
  status: WorkflowRequestStatus;
  response?: string; // responder's answer / decision note
  resolvedById?: string;
  resolvedByName?: string;
  metadata?: Record<string, unknown>;
  createdAt: string;
  updatedAt: string;
}

// ============================================
// Solution design — Trade-off / Decision Analysis
// (Solution Architect signature artifact — catalog Vol 04 SA-03/SA-04 / Epic E)
// One analysis = one Firestore doc holding its criteria + options inline
// (no sub-collections, so no composite index is needed).
// ============================================
export interface TradeoffCriterion {
  id: string;
  name: string;
  weight: number; // relative importance, 1..5
}

export interface SolutionOption {
  id: string;
  name: string;
  description?: string;
  scores: Record<string, number>; // criterionId -> raw score 1..5
  pros: string[];
  cons: string[];
}

export type TradeoffStatus = 'draft' | 'in-review' | 'decided';

export interface TradeoffAnalysis {
  id: string;
  projectId: string;
  title: string;
  description?: string;
  criteria: TradeoffCriterion[];
  options: SolutionOption[];
  recommendedOptionId?: string; // manual pin; when absent the UI shows the computed top option
  decisionRationale?: string; // ADR-lite note explaining the chosen option
  status: TradeoffStatus;
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}

// ============================================
// Designer — authored rich-text documents
// ============================================
export type DesignerDocumentCategory = 'brd' | 'hld' | 'lld' | 'architecture' | 'custom';
export type DesignerDocumentStatus = 'draft' | 'review' | 'approved';

export interface DesignerDocument {
  id: string;
  projectId: string;
  title: string;
  templateId: string;
  category: DesignerDocumentCategory;
  content: string;           // Tiptap JSON stringified
  status: DesignerDocumentStatus;
  version: number;
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}
