/**
 * Database Abstraction Layer - Interface
 */

import type {
  Project,
  Roadmap,
  RoadmapItem,
  Document,
  Meeting,
  Email,
  Chat,
  AIRequirement,
  Gap,
  Confusion,
  Suggestion,
  SecurityRequirement,
  Threat,
  NFR,
  CloudGovernance,
  FormalRequirement,
  Conflict,
  SharedContentView,
  PI,
  Sprint,
  Requirement,
  Story,
  WorkflowRequest,
  TradeoffAnalysis,
  UserProfile,
  Invite,
  AuthUser,
  AuthSession,
  ProjectMember,
  ProjectRole,
  DesignerDocument,
} from './types';

export interface AuthProvider {
  signUp(
    email: string,
    password: string,
    fullName: string,
    role?: string,
    extras?: { licenseTier?: string; defaultPersonaId?: string; status?: string; invitedBy?: string },
  ): Promise<AuthSession>;
  signIn(email: string, password: string): Promise<AuthSession>;
  signOut(): Promise<void>;
  getSession(): Promise<AuthSession | null>;
  getUser(): Promise<AuthUser | null>;
}

export interface IDatabase {
  auth: AuthProvider;

  projects: {
    getAll(userId: string): Promise<Project[]>;
    getById(id: string, userId: string): Promise<Project>;
    create(project: Omit<Project, 'id' | 'createdAt' | 'updatedAt'>): Promise<Project>;
    update(id: string, userId: string, updates: Partial<Project>): Promise<Project>;
    delete(id: string, userId: string): Promise<void>;
  };

  roadmaps: {
    getByProject(projectId: string): Promise<Roadmap[]>;
    getById(id: string): Promise<Roadmap>;
    getByYearAndType(projectId: string, year: number, type: string): Promise<Roadmap | null>;
    create(roadmap: Omit<Roadmap, 'id' | 'createdAt' | 'updatedAt'>): Promise<Roadmap>;
    delete(id: string): Promise<void>;
  };

  roadmapItems: {
    getByRoadmap(roadmapId: string): Promise<RoadmapItem[]>;
    getById(id: string): Promise<RoadmapItem>;
    create(item: Omit<RoadmapItem, 'id' | 'createdAt' | 'updatedAt'>): Promise<RoadmapItem>;
    update(id: string, updates: Partial<RoadmapItem>): Promise<RoadmapItem>;
    delete(id: string): Promise<void>;
  };

  documents: {
    getByRoadmapItem(roadmapItemId: string): Promise<Document[]>;
    create(doc: Omit<Document, 'id' | 'uploadedAt'>, file: File): Promise<Document>;
    update(id: string, updates: Pick<Document, 'name' | 'type' | 'version'>): Promise<Document>;
    delete(id: string): Promise<void>;
    getDownloadUrl(id: string): Promise<string>;
  };

  meetings: {
    getByRoadmapItem(roadmapItemId: string): Promise<Meeting[]>;
    create(meeting: Omit<Meeting, 'id' | 'createdAt'>): Promise<Meeting>;
    update(id: string, updates: Partial<Meeting>): Promise<Meeting>;
    delete(id: string): Promise<void>;
  };

  emails: {
    getByRoadmapItem(roadmapItemId: string): Promise<Email[]>;
    create(email: Omit<Email, 'id' | 'createdAt'>): Promise<Email>;
    update(id: string, updates: Partial<Omit<Email, 'id' | 'roadmapItemId' | 'createdAt'>>): Promise<Email>;
    delete(id: string): Promise<void>;
  };

  chats: {
    getByRoadmapItem(roadmapItemId: string): Promise<Chat[]>;
    create(chat: Omit<Chat, 'id' | 'createdAt'>): Promise<Chat>;
    update(id: string, updates: Partial<Omit<Chat, 'id' | 'roadmapItemId' | 'createdAt'>>): Promise<Chat>;
    delete(id: string): Promise<void>;
  };

  aiRequirements: {
    getByRoadmapItem(roadmapItemId: string): Promise<AIRequirement[]>;
    create(data: Omit<AIRequirement, 'id' | 'extractedAt'>): Promise<AIRequirement>;
    delete(id: string): Promise<void>;
  };

  gaps: {
    getByRoadmapItem(roadmapItemId: string): Promise<Gap[]>;
    create(gap: Omit<Gap, 'id' | 'createdAt' | 'updatedAt'>): Promise<Gap>;
    update(id: string, updates: Partial<Gap>): Promise<Gap>;
    delete(id: string): Promise<void>;
  };

  confusions: {
    getByRoadmapItem(roadmapItemId: string): Promise<Confusion[]>;
    create(confusion: Omit<Confusion, 'id' | 'createdAt' | 'updatedAt'>): Promise<Confusion>;
    update(id: string, updates: Partial<Confusion>): Promise<Confusion>;
    delete(id: string): Promise<void>;
  };

  suggestions: {
    getByRoadmapItem(roadmapItemId: string): Promise<Suggestion[]>;
    create(suggestion: Omit<Suggestion, 'id' | 'createdAt' | 'updatedAt'>): Promise<Suggestion>;
    update(id: string, updates: Partial<Suggestion>): Promise<Suggestion>;
    delete(id: string): Promise<void>;
  };

  securityRequirements: {
    getByRoadmapItem(roadmapItemId: string): Promise<SecurityRequirement[]>;
    create(secReq: Omit<SecurityRequirement, 'id' | 'createdAt' | 'updatedAt'>): Promise<SecurityRequirement>;
    update(id: string, updates: Partial<SecurityRequirement>): Promise<SecurityRequirement>;
    delete(id: string): Promise<void>;
  };

  threats: {
    getByRoadmapItem(roadmapItemId: string): Promise<Threat[]>;
    create(threat: Omit<Threat, 'id' | 'createdAt' | 'updatedAt'>): Promise<Threat>;
    update(id: string, updates: Partial<Threat>): Promise<Threat>;
    delete(id: string): Promise<void>;
  };

  nfrs: {
    getByRoadmapItem(roadmapItemId: string): Promise<NFR[]>;
    create(nfr: Omit<NFR, 'id' | 'createdAt' | 'updatedAt'>): Promise<NFR>;
    update(id: string, updates: Partial<NFR>): Promise<NFR>;
    delete(id: string): Promise<void>;
  };

  cloudGovernance: {
    getByRoadmapItem(roadmapItemId: string): Promise<CloudGovernance[]>;
    create(cloudGov: Omit<CloudGovernance, 'id' | 'createdAt' | 'updatedAt'>): Promise<CloudGovernance>;
    update(id: string, updates: Partial<CloudGovernance>): Promise<CloudGovernance>;
    delete(id: string): Promise<void>;
  };

  formalRequirements: {
    getByRoadmapItem(roadmapItemId: string): Promise<FormalRequirement[]>;
    getByProject(projectId: string): Promise<FormalRequirement[]>;
    create(req: Omit<FormalRequirement, 'id' | 'createdAt' | 'updatedAt'>): Promise<FormalRequirement>;
    update(id: string, updates: Partial<FormalRequirement>): Promise<FormalRequirement>;
    delete(id: string): Promise<void>;
  };

  pis: {
    getByProject(projectId: string): Promise<PI[]>;
    create(pi: Omit<PI, 'id' | 'createdAt' | 'updatedAt'>): Promise<PI>;
    update(id: string, updates: Partial<PI>): Promise<PI>;
    delete(id: string): Promise<void>;
  };

  sprints: {
    getByProject(projectId: string): Promise<Sprint[]>;
    create(sprint: Omit<Sprint, 'id' | 'createdAt' | 'updatedAt'>): Promise<Sprint>;
    update(id: string, updates: Partial<Sprint>): Promise<Sprint>;
    delete(id: string): Promise<void>;
  };

  requirements: {
    getByProject(projectId: string): Promise<Requirement[]>;
    create(req: Omit<Requirement, 'id' | 'createdAt' | 'updatedAt'>): Promise<Requirement>;
    bulkCreate(projectId: string, reqs: Array<Omit<Requirement, 'id' | 'projectId' | 'createdAt' | 'updatedAt'>>): Promise<Requirement[]>;
    update(id: string, updates: Partial<Requirement>): Promise<Requirement>;
    delete(id: string): Promise<void>;
  };

  // User stories (SAFe Step 4 — AI Story Decomposition)
  stories: {
    getByProject(projectId: string): Promise<Story[]>;
    getByRequirement(requirementId: string): Promise<Story[]>;
    create(story: Omit<Story, 'id' | 'createdAt' | 'updatedAt'>): Promise<Story>;
    bulkCreate(stories: Array<Omit<Story, 'id' | 'createdAt' | 'updatedAt'>>): Promise<Story[]>;
    update(id: string, updates: Partial<Story>): Promise<Story>;
    delete(id: string): Promise<void>;
  };

  // Cross-role workflow requests (Vol 12 / Epic D). Single-field queries only,
  // so no Firestore composite index is needed; the store merges the three feeds.
  workflowRequests: {
    getRaisedBy(userId: string): Promise<WorkflowRequest[]>;
    getAssignedToRole(role: string): Promise<WorkflowRequest[]>;
    getAssignedToUser(userId: string): Promise<WorkflowRequest[]>;
    create(req: Omit<WorkflowRequest, 'id' | 'createdAt' | 'updatedAt'>): Promise<WorkflowRequest>;
    update(id: string, updates: Partial<WorkflowRequest>): Promise<WorkflowRequest>;
    delete(id: string): Promise<void>;
  };

  // Solution-design trade-off analyses (Vol 04 SA-03/04 / Epic E). Project-scoped, single-field query.
  tradeoffAnalyses: {
    getByProject(projectId: string): Promise<TradeoffAnalysis[]>;
    getById(id: string): Promise<TradeoffAnalysis>;
    create(analysis: Omit<TradeoffAnalysis, 'id' | 'createdAt' | 'updatedAt'>): Promise<TradeoffAnalysis>;
    update(id: string, updates: Partial<TradeoffAnalysis>): Promise<TradeoffAnalysis>;
    delete(id: string): Promise<void>;
  };

  // User profiles + admin user management (Epic G). Profiles are the `users/{uid}` docs.
  users: {
    getAll(): Promise<UserProfile[]>;
    getById(id: string): Promise<UserProfile>;
    update(id: string, updates: Partial<UserProfile>): Promise<UserProfile>;
  };

  // Admin-issued invitations (Epic G). Single-field token lookup, so no composite index.
  invites: {
    getAll(): Promise<Invite[]>;
    getByToken(token: string): Promise<Invite | null>;
    create(invite: Omit<Invite, 'id' | 'createdAt' | 'updatedAt'>): Promise<Invite>;
    update(id: string, updates: Partial<Invite>): Promise<Invite>;
    delete(id: string): Promise<void>;
  };

  // Conflicts detected when a new source contradicts existing requirements.
  conflicts: {
    getByRoadmapItem(roadmapItemId: string): Promise<Conflict[]>;
    create(conflict: Omit<Conflict, 'id' | 'createdAt' | 'updatedAt'>): Promise<Conflict>;
    update(id: string, updates: Partial<Conflict>): Promise<Conflict>;
    delete(id: string): Promise<void>;
  };

  // PO-redacted source content shared with selected team members.
  sharedContentViews: {
    getByRoadmapItem(roadmapItemId: string): Promise<SharedContentView[]>;
    getByUser(userId: string): Promise<SharedContentView[]>;
    create(view: Omit<SharedContentView, 'id' | 'createdAt' | 'updatedAt'>): Promise<SharedContentView>;
    update(id: string, updates: Partial<SharedContentView>): Promise<SharedContentView>;
    delete(id: string): Promise<void>;
  };

  // Project-level team members with project-scoped roles.
  projectMembers: {
    getByProject(projectId: string): Promise<ProjectMember[]>;
    getByUser(userId: string): Promise<ProjectMember[]>;
    create(member: Omit<ProjectMember, 'id' | 'addedAt' | 'updatedAt'>): Promise<ProjectMember>;
    update(id: string, updates: { projectRoles: ProjectRole[] }): Promise<ProjectMember>;
    delete(id: string): Promise<void>;
  };

  // Designer authored rich-text documents.
  designerDocuments: {
    getByProject(projectId: string): Promise<DesignerDocument[]>;
    getById(id: string): Promise<DesignerDocument | null>;
    create(doc: Omit<DesignerDocument, 'id' | 'createdAt' | 'updatedAt'>): Promise<DesignerDocument>;
    update(id: string, updates: Partial<Omit<DesignerDocument, 'id' | 'createdAt'>>): Promise<DesignerDocument>;
    delete(id: string): Promise<void>;
  };
}
