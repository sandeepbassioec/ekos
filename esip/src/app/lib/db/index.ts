/**
 * Database Abstraction Layer - Main Export
 *
 * Change DB_PROVIDER to switch between Firebase and Supabase
 */

import { firebaseDatabase } from './firebase-impl';
import { initFirebase } from './firebase-config';
import type { IDatabase } from './interface';

// ============================================
// SWITCH PROVIDER HERE
// ============================================
const DB_PROVIDER: 'firebase' | 'supabase' = 'firebase';

let databaseInstance: IDatabase | null = null;

export function getDatabase(): IDatabase {
  if (!databaseInstance) {
    switch (DB_PROVIDER) {
      case 'firebase':
        initFirebase();
        databaseInstance = firebaseDatabase;
        break;
      case 'supabase':
        throw new Error('Supabase provider removed. Use Firebase.');
      default:
        throw new Error(`Unknown provider: ${DB_PROVIDER}`);
    }
  }
  return databaseInstance;
}

// Convenience exports
export const db = {
  get projects() { return getDatabase().projects; },
  get roadmaps() { return getDatabase().roadmaps; },
  get roadmapItems() { return getDatabase().roadmapItems; },
  get documents() { return getDatabase().documents; },
  get meetings() { return getDatabase().meetings; },
  get emails() { return getDatabase().emails; },
  get chats() { return getDatabase().chats; },
  get aiRequirements() { return getDatabase().aiRequirements; },
  get gaps() { return getDatabase().gaps; },
  get confusions() { return getDatabase().confusions; },
  get suggestions() { return getDatabase().suggestions; },
  get securityRequirements() { return getDatabase().securityRequirements; },
  get threats() { return getDatabase().threats; },
  get nfrs() { return getDatabase().nfrs; },
  get cloudGovernance() { return getDatabase().cloudGovernance; },
  get formalRequirements() { return getDatabase().formalRequirements; },
  get pis() { return getDatabase().pis; },
  get sprints() { return getDatabase().sprints; },
  get requirements() { return getDatabase().requirements; },
  get stories() { return getDatabase().stories; },
  get workflowRequests() { return getDatabase().workflowRequests; },
  get tradeoffAnalyses() { return getDatabase().tradeoffAnalyses; },
  get projectMembers() { return getDatabase().projectMembers; },
  get designerDocuments() { return getDatabase().designerDocuments; },
  get users() { return getDatabase().users; },
  get invites() { return getDatabase().invites; },
  get conflicts() { return getDatabase().conflicts; },
  get sharedContentViews() { return getDatabase().sharedContentViews; },
};

export const auth = {
  get signUp() { return getDatabase().auth.signUp.bind(getDatabase().auth); },
  get signIn() { return getDatabase().auth.signIn.bind(getDatabase().auth); },
  get signOut() { return getDatabase().auth.signOut.bind(getDatabase().auth); },
  get getSession() { return getDatabase().auth.getSession.bind(getDatabase().auth); },
  get getUser() { return getDatabase().auth.getUser.bind(getDatabase().auth); },
};

// Type exports
export type { IDatabase, AuthProvider } from './interface';
export type { AuthUser, AuthSession, Project, Roadmap, RoadmapItem, Document, Meeting, Email, Chat, AIRequirement, PI, Sprint, Requirement, Story, StoryType, StoryStatus, WorkflowRequest, TradeoffAnalysis, TradeoffCriterion, SolutionOption, UserProfile, Invite, LicenseTier, AccountStatus, InviteStatus, Conflict, SharedContentView, SourceEvent, DesignerDocument, DesignerDocumentCategory, DesignerDocumentStatus } from './types';
