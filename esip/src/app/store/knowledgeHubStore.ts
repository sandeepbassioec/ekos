/**
 * Knowledge Hub Store
 * Manages documents, meetings, emails, chats, and AI requirements
 */

// Tracks which "hubId+actorId" pairs have already received a staleness WorkflowRequest
// this session — prevents flooding Firestore with duplicate alerts on every refresh.
const _stalenessNotifiedThisSession = new Set<string>();

import { create } from 'zustand';
import { toast } from 'sonner';
import { db } from '../lib/db';
import { analyzeRequirements, searchKnowledgeHub, summarizeHub, listOpenItems, generateRiskBrief, mergeAIRequirements as mergeAIRequirementsAI, analyzeNewSource as analyzeNewSourceAI, validateMerge } from '../lib/ai/bedrock';
import type { AnalysisProgressEvent } from '../lib/ai/bedrock';
import { useNotificationStore } from './notificationStore';
import { parseDocument } from '../services/documentParser';
import { isFeatureEnabled } from './featureFlagStore';
import { useWorkflowStore } from './workflowStore';
import { useAuthStore } from './authStore';
import type {
  Document,
  Meeting,
  Email,
  Chat,
  AIRequirement,
  KnowledgeHub,
  CreateDocumentDTO,
  CreateMeetingDTO,
  CreateEmailDTO,
  CreateChatDTO,
} from '../types/knowledge-hub';
import type { FormalRequirement } from '../lib/db/types';

export interface AISearchResult {
  answer: string;
  sources: { type: string; id: string; title: string; excerpt: string }[];
  confidence: number;
}

export interface AnalysisStep {
  id: string;
  label: string;
  status: 'pending' | 'running' | 'done' | 'error';
  inputTokens?: number;
  outputTokens?: number;
}

export interface AnalysisProgress {
  isMinimized: boolean;
  roadmapItemId: string;
  steps: AnalysisStep[];
  totalInputTokens: number;
  totalOutputTokens: number;
  startedAt: number;
  completedAt?: number;
}

interface KnowledgeHubState {
  // Data
  currentHub: KnowledgeHub | null;
  isLoading: boolean;
  isAnalyzing: boolean;
  isSearching: boolean;
  lastSearchResult: AISearchResult | null;
  analysisProgress: AnalysisProgress | null;
  
  // Actions
  fetchKnowledgeHub: (roadmapItemId: string) => Promise<void>;
  
  // Document actions
  uploadDocument: (data: CreateDocumentDTO) => Promise<Document | null>;
  deleteDocument: (documentId: string) => Promise<boolean>;
  getDocumentDownloadUrl: (documentId: string) => Promise<string | null>;
  
  // Meeting actions
  createMeeting: (data: CreateMeetingDTO) => Promise<Meeting | null>;
  updateMeeting: (meetingId: string, data: Partial<Meeting>) => Promise<boolean>;
  deleteMeeting: (meetingId: string) => Promise<boolean>;
  
  // Document actions
  updateDocument: (documentId: string, data: Pick<Document, 'name' | 'type' | 'version'>) => Promise<boolean>;

  // Email actions
  createEmail: (data: CreateEmailDTO) => Promise<Email | null>;
  updateEmail: (emailId: string, data: Partial<Omit<Email, 'id' | 'roadmapItemId' | 'createdAt'>>) => Promise<boolean>;
  deleteEmail: (emailId: string) => Promise<boolean>;

  // Chat actions
  createChat: (data: CreateChatDTO) => Promise<Chat | null>;
  updateChat: (chatId: string, data: Partial<Omit<Chat, 'id' | 'roadmapItemId' | 'createdAt'>>) => Promise<boolean>;
  deleteChat: (chatId: string) => Promise<boolean>;
  
  // AI Search + Command actions
  aiSearch: (question: string) => Promise<boolean>;
  runCommand: (command: string) => Promise<{ type: 'text'; content: string } | { type: 'navigate'; tab: string } | { type: 'export'; format: 'excel' | 'pptx' | 'both' } | null>;

  // AI Requirements actions
  analyzeAllSources: (roadmapItemId: string, noDocumentMode?: boolean) => Promise<boolean>;
  mergeAIRequirements: (ids: string[]) => Promise<boolean>;
  minimizeProgress: () => void;
  maximizeProgress: () => void;
  dismissProgress: () => void;
  deleteAIRequirement: (requirementId: string) => Promise<boolean>;

  // Differential re-analysis when a new email/meeting/chat is added
  analyzeNewSource: (sourceType: 'email' | 'meeting' | 'chat', sourceId: string, sourceName: string, content: string) => Promise<void>;

  // Smart merge for Gaps / Confusions / Suggestions (with AI validation)
  getMergePreview: (entityType: 'gap' | 'confusion' | 'suggestion', ids: string[]) => Promise<{ isRelated: boolean; confidence: number; reason: string; merged: Record<string, any> } | null>;
  executeGapMerge: (ids: string[], merged: Record<string, any>, aiConfidence: number, aiReason: string) => Promise<boolean>;
  executeConfusionMerge: (ids: string[], merged: Record<string, any>, aiConfidence: number, aiReason: string) => Promise<boolean>;
  executeSuggestionMerge: (ids: string[], merged: Record<string, any>, aiConfidence: number, aiReason: string) => Promise<boolean>;

  // Conflict management
  resolveConflict: (conflictId: string, resolution: string) => Promise<boolean>;
  dismissConflict: (conflictId: string) => Promise<boolean>;
  deleteConflict: (conflictId: string) => Promise<boolean>;

  // Formal Requirements actions
  createFormalRequirement: (data: Omit<FormalRequirement, 'id' | 'createdAt' | 'updatedAt'>) => Promise<boolean>;
  updateFormalRequirement: (id: string, updates: Partial<FormalRequirement>) => Promise<boolean>;
  deleteFormalRequirement: (id: string) => Promise<boolean>;

  // Reset
  reset: () => void;
}

/**
 * Cross-role review gate for AI output (catalog C4 / Vol 12, bridging Epic D's workflow
 * system to the AI pipeline). When AI extraction adds new requirements to a roadmap item,
 * route them through a human review before they're treated as confirmed: raise a `review`
 * WorkflowRequest to the manager role, surfaced in the Inbox.
 *
 * Idempotent-ish on purpose — skipped when nothing new was added, or when the same raiser
 * already has an open/in-progress review for this item, so re-runs don't spam the reviewer.
 * Goes through the existing DAL/workflow-store seam (no direct Firestore), is gated by the
 * `aiReviewGate` feature flag (Epic H pattern), and is best-effort: a failure here never
 * breaks the analysis it follows.
 */
async function raiseAIReviewGate(roadmapItemId: string, addedRequirementCount: number): Promise<void> {
  if (!isFeatureEnabled('aiReviewGate')) return;
  if (addedRequirementCount <= 0) return;

  const user = useAuthStore.getState().user;
  if (!user) return;

  try {
    // Don't pile on: if I already raised a still-pending review for this item, leave it.
    // Single-field query (raiserId) — no composite index needed, matching the workflow seam.
    const mine = await db.workflowRequests.getRaisedBy(user.id);
    const hasOpenReview = mine.some(
      (r) =>
        r.type === 'review' &&
        r.subjectType === 'roadmap-item' &&
        r.subjectId === roadmapItemId &&
        (r.status === 'open' || r.status === 'in-progress'),
    );
    if (hasOpenReview) return;

    // Resolve the item title + owning project so the request is readable and routable.
    // Non-fatal: fall back to the id if the lookup fails.
    let itemLabel = roadmapItemId;
    let projectId: string | undefined;
    try {
      const item = await db.roadmapItems.getById(roadmapItemId);
      itemLabel = item.title || roadmapItemId;
      const roadmap = await db.roadmaps.getById(item.roadmapId);
      projectId = roadmap.projectId;
    } catch (lookupError) {
      console.warn('AI review gate: could not resolve roadmap item/project context:', lookupError);
    }

    const plural = addedRequirementCount === 1 ? '' : 's';
    await useWorkflowStore.getState().raiseRequest({
      type: 'review',
      title: `Review ${addedRequirementCount} AI-extracted requirement${plural} for ${itemLabel}`,
      description: `AI analysis added ${addedRequirementCount} new requirement${plural} to "${itemLabel}". Review and confirm before they're treated as approved.`,
      targetRole: 'manager',
      projectId,
      subjectType: 'roadmap-item',
      subjectId: roadmapItemId,
    });
  } catch (error) {
    // Best-effort: the analysis already saved; don't fail it because the gate couldn't be raised.
    console.error('AI review gate: failed to raise review request:', error);
  }
}

export const useKnowledgeHubStore = create<KnowledgeHubState>((set, get) => ({
  currentHub: null,
  isLoading: false,
  isAnalyzing: false,
  isSearching: false,
  lastSearchResult: null,
  analysisProgress: null,
  
  /**
   * Fetch all knowledge hub data for a roadmap item
   */
  fetchKnowledgeHub: async (roadmapItemId: string) => {
    set({ isLoading: true });
    try {
      const [documents, meetings, emails, chats, aiRequirements, gaps, confusions, suggestions, securityRequirements, threats, nfrs, cloudGovernance, formalRequirements, conflicts] = await Promise.all([
        db.documents.getByRoadmapItem(roadmapItemId),
        db.meetings.getByRoadmapItem(roadmapItemId),
        db.emails.getByRoadmapItem(roadmapItemId),
        db.chats.getByRoadmapItem(roadmapItemId),
        db.aiRequirements.getByRoadmapItem(roadmapItemId),
        db.gaps.getByRoadmapItem(roadmapItemId),
        db.confusions.getByRoadmapItem(roadmapItemId),
        db.suggestions.getByRoadmapItem(roadmapItemId),
        db.securityRequirements.getByRoadmapItem(roadmapItemId),
        db.threats.getByRoadmapItem(roadmapItemId),
        db.nfrs.getByRoadmapItem(roadmapItemId),
        db.cloudGovernance.getByRoadmapItem(roadmapItemId),
        db.formalRequirements.getByRoadmapItem(roadmapItemId),
        db.conflicts.getByRoadmapItem(roadmapItemId),
      ]);

      // Coverage score: % of AI requirements with all linked gaps/confusions resolved
      const totalReqs = aiRequirements.length;
      const coveredReqs = totalReqs === 0 ? 0 : aiRequirements.filter((req: any) => {
        const linkedGaps = gaps.filter((g: any) => (g.relatedRequirementIds || []).includes(req.id));
        const linkedConfs = confusions.filter((c: any) => (c.relatedRequirementIds || []).includes(req.id));
        return (
          linkedGaps.every((g: any) => g.status === 'Resolved' || g.status === 'Closed') &&
          linkedConfs.every((c: any) => c.status === 'Resolved')
        );
      }).length;
      const coverageScore = totalReqs === 0 ? 100 : Math.round((coveredReqs / totalReqs) * 100);

      const hub: KnowledgeHub = {
        roadmapItemId,
        documents,
        meetings,
        emails,
        chats,
        aiRequirements,
        gaps,
        confusions,
        suggestions,
        securityRequirements,
        threats,
        nfrs,
        cloudGovernance,
        formalRequirements,
        conflicts,
        stats: {
          totalDocuments: documents.length,
          totalMeetings: meetings.length,
          totalEmails: emails.length,
          totalChats: chats.length,
          totalRequirements: aiRequirements.length,
          totalConflicts: conflicts.length,
          coverageScore,
        },
      };
      set({ currentHub: hub, isLoading: false });

      // Staleness alert: in-memory notification for items open > 7 days
      const SEVEN_DAYS_MS = 7 * 24 * 60 * 60 * 1000;
      const now = Date.now();
      const staleGaps = gaps.filter((g: any) => g.status === 'Open' && (now - new Date(g.createdAt).getTime()) > SEVEN_DAYS_MS);
      const staleConfs = confusions.filter((c: any) => c.status === 'Open' && (now - new Date(c.createdAt).getTime()) > SEVEN_DAYS_MS);
      const staleCount = staleGaps.length + staleConfs.length;
      if (staleCount > 0) {
        useNotificationStore.getState().addNotification({
          type: 'info',
          title: `${staleCount} stale open item${staleCount > 1 ? 's' : ''}`,
          body: `${staleGaps.length} gap${staleGaps.length !== 1 ? 's' : ''} and ${staleConfs.length} confusion${staleConfs.length !== 1 ? 's' : ''} have been open for over 7 days.`,
          link: undefined,
        });

        // Cross-user staleness: create WorkflowRequests for all human actors involved in stale items
        // so every stakeholder sees it in their Inbox, not just the current user.
        const currentUserId = useAuthStore.getState().user?.id;
        const staleItems = [...staleGaps, ...staleConfs];
        const involvedActorIds = new Set<string>();
        for (const item of staleItems) {
          const events: any[] = (item as any).sourceEvents ?? [];
          for (const evt of events) {
            if (evt.actorType === 'human' && evt.actorId && evt.actorId !== currentUserId) {
              involvedActorIds.add(evt.actorId);
            }
          }
        }
        const wrTitle = `${staleCount} stale open item${staleCount > 1 ? 's' : ''} need attention`;
        const wrDesc = `${staleGaps.length} gap${staleGaps.length !== 1 ? 's' : ''} and ${staleConfs.length} confusion${staleConfs.length !== 1 ? 's' : ''} on this roadmap item have been open for over 7 days. Please review and resolve them.`;
        for (const actorId of involvedActorIds) {
          const dedupeKey = `${roadmapItemId}:${actorId}`;
          if (_stalenessNotifiedThisSession.has(dedupeKey)) continue;
          _stalenessNotifiedThisSession.add(dedupeKey);
          db.workflowRequests.create({
            type: 'staleness-alert',
            title: wrTitle,
            description: wrDesc,
            raiserId: currentUserId ?? 'system',
            targetUserId: actorId,
            subjectType: 'roadmap-item',
            subjectId: roadmapItemId,
            status: 'open',
            metadata: { staleGapIds: staleGaps.map((g: any) => g.id), staleConfIds: staleConfs.map((c: any) => c.id) },
          }).catch((err: unknown) => console.warn('Staleness WorkflowRequest create failed:', err));
        }
      }
      if (conflicts.filter((c: any) => c.status === 'Open').length > 0) {
        useNotificationStore.getState().addNotification({
          type: 'info',
          title: `${conflicts.filter((c: any) => c.status === 'Open').length} open conflict${conflicts.filter((c: any) => c.status === 'Open').length > 1 ? 's' : ''} detected`,
          body: 'A source contradicts one or more existing requirements. Review the Conflicts tab.',
          link: undefined,
        });
      }
    } catch (error) {
      console.error('Error fetching knowledge hub:', error);
      toast.error('Failed to load knowledge hub');
      set({ isLoading: false });
    }
  },
  
  /**
   * Upload a document
   */
  uploadDocument: async (data: CreateDocumentDTO) => {
    try {
      // Extract text up front so AI analysis sees the real document content,
      // not just the filename. Non-fatal: if parsing fails (unsupported/corrupt
      // file) we still upload, just with empty extractedText.
      let extractedText = '';
      try {
        const parsed = await parseDocument(data.file);
        extractedText = parsed.text || '';
      } catch (parseError) {
        console.warn('Document text extraction failed; AI analysis will be limited for this file:', parseError);
      }

      const document = await db.documents.create({
        roadmapItemId: data.roadmapItemId,
        name: data.name,
        type: data.type,
        version: data.version,
        storagePath: '',
        fileSize: data.file.size,
        extractedText,
      }, data.file);
      toast.success(
        extractedText
          ? 'Document uploaded successfully'
          : 'Document uploaded — text could not be extracted, so AI analysis will be limited for this file'
      );
      await get().fetchKnowledgeHub(data.roadmapItemId);
      return document;
    } catch (error: any) {
      console.error('Error uploading document:', error);

      // Check if it's a Firebase Storage permission error
      if (error?.code === 'storage/unauthorized' || error?.message?.includes('unauthorized')) {
        toast.error('Upload blocked: Firebase Storage rules need to be updated. Check FIREBASE-STORAGE-FIX.md');
      } else {
        toast.error(`Failed to upload document: ${error?.message || 'Unknown error'}`);
      }
      return null;
    }
  },
  
  /**
   * Delete a document
   */
  deleteDocument: async (documentId: string) => {
    try {
      await db.documents.delete(documentId);
      toast.success('Document deleted');
      set((state) => {
        if (!state.currentHub) return state;
        return {
          currentHub: {
            ...state.currentHub,
            documents: state.currentHub.documents.filter(d => d.id !== documentId),
            stats: {
              ...state.currentHub.stats,
              totalDocuments: state.currentHub.stats.totalDocuments - 1,
            },
          },
        };
      });
      return true;
    } catch (error) {
      console.error('Error deleting document:', error);
      toast.error('Failed to delete document');
      return false;
    }
  },
  
  /**
   * Get signed download URL for a document
   */
  getDocumentDownloadUrl: async (documentId: string) => {
    try {
      return await db.documents.getDownloadUrl(documentId);
    } catch (error) {
      console.error('Error getting download URL:', error);
      toast.error('Failed to get download URL');
      return null;
    }
  },
  
  /**
   * Create a meeting
   */
  createMeeting: async (data: CreateMeetingDTO) => {
    try {
      const meeting = await db.meetings.create(data);
      toast.success('Meeting added successfully');
      await get().fetchKnowledgeHub(data.roadmapItemId);
      // Differential analysis: fire-and-forget
      const notes = (data as any).notes || (data as any).transcript || '';
      if (notes) get().analyzeNewSource('meeting', meeting.id, data.title, notes);
      return meeting;
    } catch (error) {
      console.error('Error creating meeting:', error);
      toast.error('Failed to create meeting');
      return null;
    }
  },
  
  /**
   * Update a meeting
   */
  updateMeeting: async (meetingId: string, data: Partial<Meeting>) => {
    try {
      await db.meetings.update(meetingId, data);
      toast.success('Meeting updated');

      // Update local state
      set((state) => {
        if (!state.currentHub) return state;
        return {
          currentHub: {
            ...state.currentHub,
            meetings: state.currentHub.meetings.map(m =>
              m.id === meetingId ? { ...m, ...data } : m
            ),
          },
        };
      });

      return true;
    } catch (error) {
      console.error('Error updating meeting:', error);
      toast.error('Failed to update meeting');
      return false;
    }
  },
  
  /**
   * Delete a meeting
   */
  deleteMeeting: async (meetingId: string) => {
    try {
      await db.meetings.delete(meetingId);
      toast.success('Meeting deleted');

      // Update local state
      set((state) => {
        if (!state.currentHub) return state;
        return {
          currentHub: {
            ...state.currentHub,
            meetings: state.currentHub.meetings.filter(m => m.id !== meetingId),
            stats: {
              ...state.currentHub.stats,
              totalMeetings: state.currentHub.stats.totalMeetings - 1,
            },
          },
        };
      });

      return true;
    } catch (error) {
      console.error('Error deleting meeting:', error);
      toast.error('Failed to delete meeting');
      return false;
    }
  },
  
  /**
   * Create an email
   */
  updateDocument: async (documentId: string, data: Pick<Document, 'name' | 'type' | 'version'>) => {
    try {
      const updated = await db.documents.update(documentId, data);
      toast.success('Document updated');
      set((state) => {
        if (!state.currentHub) return state;
        return {
          currentHub: {
            ...state.currentHub,
            documents: state.currentHub.documents.map(d => d.id === documentId ? { ...d, ...updated } : d),
          },
        };
      });
      return true;
    } catch (error) {
      console.error('Error updating document:', error);
      toast.error('Failed to update document');
      return false;
    }
  },

  createEmail: async (data: CreateEmailDTO) => {
    try {
      const email = await db.emails.create(data);
      toast.success('Email added successfully');
      await get().fetchKnowledgeHub(data.roadmapItemId);
      // Differential analysis: fire-and-forget
      const body = (data as any).body || (data as any).content || '';
      const subject = (data as any).subject || 'Email';
      if (body) get().analyzeNewSource('email', email.id, subject, body);
      return email;
    } catch (error) {
      console.error('Error creating email:', error);
      toast.error('Failed to create email');
      return null;
    }
  },
  
  /**
   * Delete an email
   */
  deleteEmail: async (emailId: string) => {
    try {
      await db.emails.delete(emailId);
      toast.success('Email deleted');

      // Update local state
      set((state) => {
        if (!state.currentHub) return state;
        return {
          currentHub: {
            ...state.currentHub,
            emails: state.currentHub.emails.filter(e => e.id !== emailId),
            stats: {
              ...state.currentHub.stats,
              totalEmails: state.currentHub.stats.totalEmails - 1,
            },
          },
        };
      });

      return true;
    } catch (error) {
      console.error('Error deleting email:', error);
      toast.error('Failed to delete email');
      return false;
    }
  },
  
  /**
   * Create a chat
   */
  updateEmail: async (emailId: string, data: Partial<Omit<Email, 'id' | 'roadmapItemId' | 'createdAt'>>) => {
    try {
      const updated = await db.emails.update(emailId, data);
      toast.success('Email updated');
      set((state) => {
        if (!state.currentHub) return state;
        return {
          currentHub: {
            ...state.currentHub,
            emails: state.currentHub.emails.map(e => e.id === emailId ? { ...e, ...updated } : e),
          },
        };
      });
      return true;
    } catch (error) {
      console.error('Error updating email:', error);
      toast.error('Failed to update email');
      return false;
    }
  },

  updateChat: async (chatId: string, data: Partial<Omit<Chat, 'id' | 'roadmapItemId' | 'createdAt'>>) => {
    try {
      const updated = await db.chats.update(chatId, data);
      toast.success('Chat updated');
      set((state) => {
        if (!state.currentHub) return state;
        return {
          currentHub: {
            ...state.currentHub,
            chats: state.currentHub.chats.map(c => c.id === chatId ? { ...c, ...updated } : c),
          },
        };
      });
      return true;
    } catch (error) {
      console.error('Error updating chat:', error);
      toast.error('Failed to update chat');
      return false;
    }
  },

  createChat: async (data: CreateChatDTO) => {
    try {
      const chat = await db.chats.create(data);
      toast.success('Chat added successfully');
      await get().fetchKnowledgeHub(data.roadmapItemId);
      // Differential analysis: fire-and-forget
      const content = (data as any).content || (data as any).messages || '';
      const name = (data as any).name || (data as any).title || 'Chat';
      if (content) get().analyzeNewSource('chat', chat.id, name, typeof content === 'string' ? content : JSON.stringify(content));
      return chat;
    } catch (error) {
      console.error('Error creating chat:', error);
      toast.error('Failed to create chat');
      return null;
    }
  },
  
  /**
   * Delete a chat
   */
  deleteChat: async (chatId: string) => {
    try {
      await db.chats.delete(chatId);
      toast.success('Chat deleted');

      // Update local state
      set((state) => {
        if (!state.currentHub) return state;
        return {
          currentHub: {
            ...state.currentHub,
            chats: state.currentHub.chats.filter(c => c.id !== chatId),
            stats: {
              ...state.currentHub.stats,
              totalChats: state.currentHub.stats.totalChats - 1,
            },
          },
        };
      });

      return true;
    } catch (error) {
      console.error('Error deleting chat:', error);
      toast.error('Failed to delete chat');
      return false;
    }
  },
  
  // ── AI Search ───────────────────────────────────────────────────────────

  aiSearch: async (question: string) => {
    const hub = get().currentHub;
    if (!hub) { toast.error('No knowledge hub loaded'); return false; }

    set({ isSearching: true, lastSearchResult: null });
    try {
      const result = await searchKnowledgeHub(question, {
        documents: hub.documents.map(d => ({
          id: d.id, name: d.name, content: (d as any).extractedText || d.name,
        })),
        meetings: hub.meetings.map(m => ({
          id: m.id, title: m.title, notes: (m as any).transcript || (m as any).notes || '',
        })),
        emails: hub.emails.map(e => ({
          id: e.id, subject: e.subject, body: e.body,
        })),
        chats: hub.chats.map(c => ({
          id: c.id, title: (c as any).title, content: c.messages,
        })),
        formalRequirements: (hub.formalRequirements ?? []).map((r: any) => ({
          id: r.id, requirementId: r.requirementId, title: r.title,
          description: r.description, category: r.category,
        })),
        gaps: (hub.gaps ?? []).map((g: any) => ({
          id: g.id, title: g.title, description: g.description, status: g.status,
        })),
      });
      set({ lastSearchResult: result, isSearching: false });
      return true;
    } catch (err) {
      console.error('AI search error:', err);
      toast.error('Search failed — please try again');
      set({ isSearching: false });
      return false;
    }
  },

  runCommand: async (command: string) => {
    const hub = get().currentHub;
    if (!hub) return null;

    // Navigation commands — no AI needed
    const navMap: Record<string, string> = {
      '/gaps':         'gaps',
      '/threats':      'threats',
      '/security':     'security',
      '/nfr':          'nfrs',
      '/requirements': 'requirements',
      '/documents':    'documents',
      '/meetings':     'meetings',
      '/emails':       'emails',
      '/chats':        'chats',
    };
    if (navMap[command]) return { type: 'navigate', tab: navMap[command] };

    // Export commands — no AI needed
    if (command === '/excel') return { type: 'export', format: 'excel' };
    if (command === '/pptx')  return { type: 'export', format: 'pptx' };
    if (command === '/export') return { type: 'export', format: 'both' };

    // Filter shortcuts — run as AI search queries
    const filterMap: Record<string, string> = {
      '/critical': 'List all critical priority requirements and critical/high severity gaps and threats',
      '/approved': 'List all approved requirements and what has been formally signed off',
      '/pending':  'List all requirements still in Draft or Under Review that need attention',
    };
    if (filterMap[command]) {
      set({ isSearching: true, lastSearchResult: null });
      try {
        const result = await searchKnowledgeHub(filterMap[command], {
          documents: hub.documents.map(d => ({ id: d.id, name: d.name, content: (d as any).extractedText || d.name })),
          meetings:  hub.meetings.map(m => ({ id: m.id, title: m.title, notes: (m as any).transcript || '' })),
          emails:    hub.emails.map(e => ({ id: e.id, subject: e.subject, body: e.body })),
          chats:     hub.chats.map(c => ({ id: c.id, title: (c as any).title, content: c.messages })),
          formalRequirements: (hub.formalRequirements ?? []).map((r: any) => ({ id: r.id, requirementId: r.requirementId, title: r.title, description: r.description, category: r.category })),
          gaps: (hub.gaps ?? []).map((g: any) => ({ id: g.id, title: g.title, description: g.description, status: g.status })),
        });
        set({ lastSearchResult: result, isSearching: false });
        return { type: 'text', content: result.answer };
      } catch {
        set({ isSearching: false });
        return null;
      }
    }

    // AI commands
    set({ isSearching: true, lastSearchResult: null });
    try {
      let text = '';

      if (command === '/summary') {
        text = await summarizeHub({
          documents: hub.documents, meetings: hub.meetings,
          emails: hub.emails, chats: hub.chats,
          formalRequirements: hub.formalRequirements ?? [],
          gaps: hub.gaps ?? [], confusions: hub.confusions ?? [],
          suggestions: hub.suggestions ?? [],
          securityRequirements: hub.securityRequirements ?? [],
          threats: hub.threats ?? [], nfrs: hub.nfrs ?? [],
        });
      } else if (command === '/openitems') {
        text = await listOpenItems({ gaps: hub.gaps ?? [], confusions: hub.confusions ?? [] });
      } else if (command === '/risks') {
        text = await generateRiskBrief({
          threats: hub.threats ?? [], gaps: hub.gaps ?? [],
          securityRequirements: hub.securityRequirements ?? [],
        });
      } else if (command === '/analyze') {
        // Trigger full analysis — handled externally, just signal it
        set({ isSearching: false });
        return { type: 'text', content: '__TRIGGER_ANALYZE__' };
      }

      set({
        lastSearchResult: { answer: text, sources: [], confidence: 100 },
        isSearching: false,
      });
      return { type: 'text', content: text };
    } catch (err) {
      console.error('Command error:', err);
      set({ isSearching: false });
      return null;
    }
  },

  minimizeProgress: () =>
    set((s) => s.analysisProgress ? { analysisProgress: { ...s.analysisProgress, isMinimized: true } } : s),
  maximizeProgress: () =>
    set((s) => s.analysisProgress ? { analysisProgress: { ...s.analysisProgress, isMinimized: false } } : s),
  dismissProgress: () => set({ analysisProgress: null }),

  /**
   * Analyze all sources with AWS Bedrock AI
   */
  analyzeAllSources: async (roadmapItemId: string, noDocumentMode = false) => {
    set({ isAnalyzing: true });

    try {
      const currentHub = get().currentHub;
      if (!currentHub) {
        toast.error('No knowledge hub data found');
        return false;
      }

      // Check if there are any sources to analyze
      const hasSources = currentHub.documents.length > 0 ||
                        currentHub.meetings.length > 0 ||
                        currentHub.emails.length > 0 ||
                        currentHub.chats.length > 0;

      if (!hasSources) {
        toast.error('No documents, meetings, emails, or chats to analyze');
        return false;
      }

      // Build initial step list so the modal can show pending steps immediately
      const steps: AnalysisStep[] = [
        ...currentHub.documents.map(doc => ({
          id: `doc-${doc.id}`,
          label: `Document: ${doc.name}`,
          status: 'pending' as const,
        })),
        ...currentHub.meetings.map(m => ({
          id: `meeting-${m.id}`,
          label: `Meeting: ${m.title}`,
          status: 'pending' as const,
        })),
        ...currentHub.emails.map(e => ({
          id: `email-${e.id}`,
          label: `Email: ${e.subject}`,
          status: 'pending' as const,
        })),
        ...currentHub.chats.map(c => ({
          id: `chat-${c.id}`,
          label: `Chat thread`,
          status: 'pending' as const,
        })),
      ];

      set({
        analysisProgress: {
          isMinimized: false,
          roadmapItemId,
          steps,
          totalInputTokens: 0,
          totalOutputTokens: 0,
          startedAt: Date.now(),
        },
      });

      // Progress callback — updates each step in place
      const onProgress = (event: AnalysisProgressEvent) => {
        set((state) => {
          if (!state.analysisProgress) return state;
          if (event.type === 'step-start') {
            return {
              analysisProgress: {
                ...state.analysisProgress,
                steps: state.analysisProgress.steps.map((s) =>
                  s.id === event.stepId ? { ...s, status: 'running' } : s,
                ),
              },
            };
          }
          if (event.type === 'step-done') {
            return {
              analysisProgress: {
                ...state.analysisProgress,
                totalInputTokens: state.analysisProgress.totalInputTokens + event.inputTokens,
                totalOutputTokens: state.analysisProgress.totalOutputTokens + event.outputTokens,
                steps: state.analysisProgress.steps.map((s) =>
                  s.id === event.stepId
                    ? { ...s, status: 'done', inputTokens: event.inputTokens, outputTokens: event.outputTokens }
                    : s,
                ),
              },
            };
          }
          if (event.type === 'step-error') {
            return {
              analysisProgress: {
                ...state.analysisProgress,
                steps: state.analysisProgress.steps.map((s) =>
                  s.id === event.stepId ? { ...s, status: 'error' } : s,
                ),
              },
            };
          }
          return state;
        });
      };

      // Prepare source texts
      const sourceTexts = {
        documents: currentHub.documents.map(doc => ({
          id: doc.id,
          name: doc.name,
          content: doc.extractedText || `Document: ${doc.name}`,
        })),
        meetings: currentHub.meetings.map(meeting => ({
          id: meeting.id,
          title: meeting.title,
          notes: meeting.transcript || meeting.notes || '',
        })),
        emails: currentHub.emails.map(email => ({
          id: email.id,
          subject: email.subject,
          body: email.body,
        })),
        chats: currentHub.chats.map(chat => ({
          id: chat.id,
          content: chat.messages,
        })),
      };

      // Call AWS Bedrock — noDocumentMode skips baseline requirement check
      const result = await analyzeRequirements({ roadmapItemId }, sourceTexts, noDocumentMode, onProgress);

      // ── Phase 1: Save requirements first — need their Firestore IDs to build traceability links ──
      const mapSourceType = (t?: string): 'Document' | 'Meeting' | 'Email' | 'Chat' => {
        switch ((t || '').toLowerCase()) {
          case 'meeting': return 'Meeting';
          case 'email': return 'Email';
          case 'chat': return 'Chat';
          default: return 'Document';
        }
      };
      const mapCategory = (t?: string): 'Functional' | 'Non-Functional' | 'Security' | 'Performance' | 'Usability' | 'Scalability' => {
        switch (t) {
          case 'Non-Functional': return 'Non-Functional';
          case 'Security': return 'Security';
          case 'Performance': return 'Performance';
          default: return 'Functional';
        }
      };

      // Seed the key→ID map from existing requirements (so re-runs can still resolve old keys)
      const keyToId = new Map<string, string>();
      (currentHub.aiRequirements || []).forEach((r: any) => {
        if (r.requirementKey && r.id) keyToId.set(r.requirementKey, r.id);
      });

      if (result.coreRequirements && result.coreRequirements.length > 0) {
        const existingKeys = new Set(Array.from(keyToId.keys()));
        const newReqs = result.coreRequirements.filter((req: any) =>
          !(req.requirementKey && existingKeys.has(req.requirementKey))
        );
        const savedReqs = await Promise.all(newReqs.map((req: any) =>
          db.aiRequirements.create({
            roadmapItemId,
            requirementKey: req.requirementKey || undefined,
            requirement: req.requirement,
            description: req.description || undefined,
            technicalCategory: req.technicalCategory || undefined,
            changeType: req.changeType || undefined,
            rationale: req.rationale || undefined,
            type: req.type || undefined,
            source: req.source?.reference || req.source?.type || 'AI',
            sourceType: mapSourceType(req.source?.type),
            sourceId: req.source?.id || '',
            category: mapCategory(req.type),
            priority: req.priority === 'Critical' ? 'High' : (req.priority || 'Medium'),
            confidence: typeof req.confidence === 'number' ? req.confidence : 0,
            businessValue: req.businessValue || undefined,
            acceptanceCriteria: Array.isArray(req.acceptanceCriteria) ? req.acceptanceCriteria : undefined,
            testScenarios: Array.isArray(req.testScenarios) ? req.testScenarios : undefined,
            qualityIssues: Array.isArray(req.qualityIssues) ? req.qualityIssues : undefined,
            assumptions: Array.isArray(req.assumptions) ? req.assumptions : undefined,
            stakeholders: Array.isArray(req.stakeholders) ? req.stakeholders : undefined,
            complianceFrameworks: Array.isArray(req.complianceFrameworks) ? req.complianceFrameworks : undefined,
            domain: req.domain || (result as any).domain || undefined,
          })
        ));
        newReqs.forEach((req: any, i: number) => {
          if (req.requirementKey && savedReqs[i]?.id) keyToId.set(req.requirementKey, savedReqs[i].id);
        });
      }

      // Resolves AI-output requirementKey slugs → actual Firestore requirement IDs
      const resolveReqKeys = (keys?: string[]): string[] =>
        (keys || []).map((k: string) => keyToId.get(k)).filter((id): id is string => !!id);

      // ── Phase 2: Save everything else in parallel ──────────────────────────────
      const savePromises: Promise<any>[] = [];

      // Gaps — with traceability back to the requirements they belong to
      if (result.gaps && result.gaps.length > 0) {
        result.gaps.forEach((gap: any) => {
          savePromises.push(db.gaps.create({
            roadmapItemId,
            title: gap.title,
            description: gap.description,
            severity: gap.severity,
            status: 'Open',
            sourceType: gap.source?.type || 'AI',
            sourceId: gap.source?.id,
            suggestions: gap.suggestions || [],
            relatedRequirementIds: resolveReqKeys(gap.relatedRequirementKeys),
          }));
        });
      }

      // Confusions — with traceability back to the requirements they belong to
      if (result.confusions && result.confusions.length > 0) {
        result.confusions.forEach((confusion: any) => {
          savePromises.push(db.confusions.create({
            roadmapItemId,
            question: confusion.question,
            context: confusion.context,
            sources: confusion.sources || [],
            status: 'Open',
            relatedRequirementIds: resolveReqKeys(confusion.relatedRequirementKeys),
          }));
        });
      }

      // Suggestions — with traceability back to the requirements they belong to
      if (result.suggestions && result.suggestions.length > 0) {
        result.suggestions.forEach((suggestion: any) => {
          savePromises.push(db.suggestions.create({
            roadmapItemId,
            title: suggestion.title,
            description: suggestion.description,
            type: suggestion.type,
            impact: suggestion.impact,
            effort: suggestion.effort,
            status: 'Pending',
            generatedBy: 'AI',
            relatedRequirementIds: resolveReqKeys(suggestion.relatedRequirementKeys),
          }));
        });
      }

      // Save security requirements
      if (result.securityRequirements && result.securityRequirements.length > 0) {
        result.securityRequirements.forEach((secReq: any) => {
          savePromises.push(db.securityRequirements.create({
            roadmapItemId,
            title: secReq.title,
            description: secReq.description,
            category: secReq.category,
            level: secReq.level,
            standard: secReq.standard,
            controls: secReq.controls || [],
            testCriteria: secReq.testCriteria || [],
            status: 'Draft',
          }));
        });
      }

      // Save threats
      if (result.threats && result.threats.length > 0) {
        result.threats.forEach((threat: any) => {
          savePromises.push(db.threats.create({
            roadmapItemId,
            threat: threat.threat,
            description: threat.description,
            category: threat.category,
            level: threat.level,
            likelihood: threat.likelihood,
            impact: threat.impact,
            mitigations: threat.mitigations || [],
            status: 'Identified',
          }));
        });
      }

      // Save NFRs
      if (result.nfrs && result.nfrs.length > 0) {
        result.nfrs.forEach((nfr: any) => {
          savePromises.push(db.nfrs.create({
            roadmapItemId,
            category: nfr.category,
            requirement: nfr.requirement,
            metric: nfr.metric,
            target: nfr.target,
            measurement: nfr.measurement,
            priority: nfr.priority,
            status: 'Draft',
          }));
        });
      }

      // Save Cloud Governance
      if (result.cloudGovernance && result.cloudGovernance.length > 0) {
        result.cloudGovernance.forEach((cloudGov: any) => {
          savePromises.push(db.cloudGovernance.create({
            roadmapItemId,
            area: cloudGov.area,
            requirement: cloudGov.requirement,
            details: cloudGov.details,
            priority: cloudGov.priority,
            status: 'Draft',
          }));
        });
      }

      // Wait for all saves to complete
      await Promise.all(savePromises);

      const reqCount = result.coreRequirements?.length || 0;
      const gapCount = result.gaps?.length || 0;

      // Mark progress complete
      set((state) => ({
        analysisProgress: state.analysisProgress
          ? { ...state.analysisProgress, completedAt: Date.now() }
          : null,
      }));

      // Push a bell notification so the user knows if they backgrounded it
      useNotificationStore.getState().addNotification({
        type: 'ai-analysis-complete',
        title: 'AI Analysis complete',
        body: `Found ${reqCount} requirements, ${gapCount} gaps across ${steps.length} source${steps.length !== 1 ? 's' : ''}.`,
        link: `/roadmap-items/${roadmapItemId}/knowledge-hub`,
      });

      // Gate the freshly-extracted requirements behind a human review (catalog C4 / Vol 12)
      await raiseAIReviewGate(roadmapItemId, reqCount);

      // Refresh knowledge hub to show new data
      await get().fetchKnowledgeHub(roadmapItemId);

      return true;
    } catch (error: any) {
      console.error('Error analyzing sources:', error);
      toast.error(`AI analysis failed: ${error?.message || 'Unknown error'}`);
      // Mark any still-running steps as error
      set((state) => ({
        analysisProgress: state.analysisProgress
          ? {
              ...state.analysisProgress,
              completedAt: Date.now(),
              steps: state.analysisProgress.steps.map((s) =>
                s.status === 'running' || s.status === 'pending' ? { ...s, status: 'error' } : s,
              ),
            }
          : null,
      }));
      return false;
    } finally {
      set({ isAnalyzing: false });
    }
  },
  
  /**
   * Merge 2+ selected AI requirements into one via AI, replacing the originals.
   */
  mergeAIRequirements: async (ids: string[]) => {
    const currentHub = get().currentHub;
    if (!currentHub || ids.length < 2) return false;
    const toMerge = currentHub.aiRequirements.filter(r => ids.includes(r.id));
    if (toMerge.length < 2) return false;

    const mapCategory = (t?: string): 'Functional' | 'Non-Functional' | 'Security' | 'Performance' | 'Usability' | 'Scalability' => {
      switch (t) {
        case 'Non-Functional': return 'Non-Functional';
        case 'Security': return 'Security';
        case 'Performance': return 'Performance';
        default: return 'Functional';
      }
    };

    try {
      const merged = await mergeAIRequirementsAI(toMerge);
      await Promise.all(ids.map(id => db.aiRequirements.delete(id)));
      await db.aiRequirements.create({
        roadmapItemId: currentHub.roadmapItemId,
        requirementKey: merged.requirementKey,
        requirement: merged.requirement,
        description: merged.description,
        technicalCategory: merged.technicalCategory,
        changeType: merged.changeType,
        rationale: merged.rationale,
        source: 'AI Merge',
        sourceType: 'Document',
        sourceId: '',
        category: mapCategory(merged.type),
        priority: merged.priority === 'Critical' ? 'High' : ((merged.priority as any) || 'Medium'),
        confidence: merged.confidence,
      });
      await get().fetchKnowledgeHub(currentHub.roadmapItemId);
      toast.success(`${ids.length} requirements merged into 1`);
      return true;
    } catch (error) {
      console.error('Error merging requirements:', error);
      toast.error('Failed to merge requirements');
      return false;
    }
  },

  /**
   * Delete an AI requirement
   */
  deleteAIRequirement: async (requirementId: string) => {
    try {
      await db.aiRequirements.delete(requirementId);
      toast.success('AI requirement deleted');

      // Update local state
      set((state) => {
        if (!state.currentHub) return state;
        return {
          currentHub: {
            ...state.currentHub,
            aiRequirements: state.currentHub.aiRequirements.filter(r => r.id !== requirementId),
            stats: {
              ...state.currentHub.stats,
              totalRequirements: state.currentHub.stats.totalRequirements - 1,
            },
          },
        };
      });

      return true;
    } catch (error) {
      console.error('Error deleting AI requirement:', error);
      toast.error('Failed to delete AI requirement');
      return false;
    }
  },
  
  // ── Differential re-analysis ────────────────────────────────────────────

  analyzeNewSource: async (sourceType: 'email' | 'meeting' | 'chat', sourceId: string, sourceName: string, content: string) => {
    const currentHub = get().currentHub;
    if (!currentHub) return;

    const openGaps = (currentHub.gaps || []).filter((g: any) => g.status === 'Open').map((g: any) => ({ id: g.id, title: g.title, description: g.description }));
    const openConfs = (currentHub.confusions || []).filter((c: any) => c.status === 'Open').map((c: any) => ({ id: c.id, question: c.question, context: c.context }));

    try {
      const result = await analyzeNewSourceAI({ sourceType, sourceName, content, openGaps, openConfusions: openConfs, aiRequirements: currentHub.aiRequirements });

      const eventBase = { sourceType: sourceType as any, sourceId, sourceName, actorType: 'ai' as const, at: Date.now() };
      const newEventId = () => `${Date.now()}-${Math.random().toString(36).slice(2)}`;

      // Resolve gaps
      for (const resolved of result.resolvedGaps) {
        const gap = (currentHub.gaps || []).find((g: any) => g.id === resolved.id);
        const events: any[] = [...(gap?.sourceEvents || []), { id: newEventId(), type: 'resolved', ...eventBase, excerpt: resolved.excerpt }];
        await db.gaps.update(resolved.id, { status: 'Resolved', resolution: `Resolved via ${sourceType}: ${sourceName}. "${resolved.excerpt}"`, sourceEvents: events });
      }

      // Clarify confusions
      for (const clarified of result.clarifiedConfusions) {
        const conf = (currentHub.confusions || []).find((c: any) => c.id === clarified.id);
        const events: any[] = [...(conf?.sourceEvents || []), { id: newEventId(), type: 'clarified', ...eventBase, excerpt: clarified.excerpt }];
        await db.confusions.update(clarified.id, { status: 'Resolved', clarification: `Clarified via ${sourceType}: ${sourceName}. "${clarified.excerpt}"`, sourceEvents: events });
      }

      // Save new conflicts
      for (const conflict of result.conflicts) {
        await db.conflicts.create({
          roadmapItemId: currentHub.roadmapItemId,
          title: conflict.title,
          description: conflict.description,
          triggerSourceType: ({ email: 'Email', meeting: 'Meeting', chat: 'Chat' } as any)[sourceType],
          triggerSourceId: sourceId,
          triggerSourceName: sourceName,
          triggerExcerpt: conflict.excerpt,
          conflictingRequirementKey: conflict.conflictingRequirementKey,
          severity: (conflict.severity as any) || 'Medium',
          status: 'Open',
        });
      }

      // Save new gaps from this source
      for (const gap of result.newGaps) {
        const events: any[] = [{ id: newEventId(), type: 'created', ...eventBase }];
        await db.gaps.create({ roadmapItemId: currentHub.roadmapItemId, title: gap.title, description: gap.description, severity: (gap.severity as any) || 'Medium', status: 'Open', sourceType: ({ email: 'Email', meeting: 'Meeting', chat: 'Chat' } as any)[sourceType], sourceId, suggestions: gap.suggestions || [], sourceEvents: events });
      }

      // Save new confusions from this source
      for (const conf of result.newConfusions) {
        const events: any[] = [{ id: newEventId(), type: 'created', ...eventBase }];
        await db.confusions.create({ roadmapItemId: currentHub.roadmapItemId, question: conf.question, context: conf.context, sources: [{ type: sourceType, id: sourceId }], status: 'Open', sourceEvents: events });
      }

      // Save any new requirements identified by differential analysis
      if (Array.isArray(result.newRequirements) && result.newRequirements.length > 0) {
        const existingHub = get().currentHub;
        const existingKeys = new Set((existingHub?.aiRequirements || []).map((r: any) => r.requirementKey).filter(Boolean));
        const freshReqs = (result.newRequirements as any[]).filter((req: any) => !req.requirementKey || !existingKeys.has(req.requirementKey));
        await Promise.all(freshReqs.map((req: any) =>
          db.aiRequirements.create({
            roadmapItemId: currentHub.roadmapItemId,
            requirementKey: req.requirementKey || undefined,
            requirement: req.requirement,
            description: req.description || undefined,
            technicalCategory: req.technicalCategory || undefined,
            changeType: req.changeType || undefined,
            rationale: req.rationale || undefined,
            type: req.type || undefined,
            source: sourceName,
            sourceType: { email: 'Email' as const, meeting: 'Meeting' as const, chat: 'Chat' as const }[sourceType as 'email' | 'meeting' | 'chat'] ?? 'Document' as const,
            sourceId,
            category: mapCategory(req.type),
            priority: req.priority === 'Critical' ? 'High' : (req.priority || 'Medium'),
            confidence: typeof req.confidence === 'number' ? req.confidence : 0,
            businessValue: req.businessValue || undefined,
            acceptanceCriteria: Array.isArray(req.acceptanceCriteria) ? req.acceptanceCriteria : undefined,
            testScenarios: Array.isArray(req.testScenarios) ? req.testScenarios : undefined,
            qualityIssues: Array.isArray(req.qualityIssues) ? req.qualityIssues : undefined,
            assumptions: Array.isArray(req.assumptions) ? req.assumptions : undefined,
            stakeholders: Array.isArray(req.stakeholders) ? req.stakeholders : undefined,
            complianceFrameworks: Array.isArray(req.complianceFrameworks) ? req.complianceFrameworks : undefined,
          })
        ));
      }

      await get().fetchKnowledgeHub(currentHub.roadmapItemId);

      const resolvedCount = result.resolvedGaps.length + result.clarifiedConfusions.length;
      const newCount = result.newGaps.length + result.newConfusions.length + (result.newRequirements?.length || 0);
      const conflictCount = result.conflicts.length;
      const parts = [];
      if (resolvedCount > 0) parts.push(`${resolvedCount} item${resolvedCount > 1 ? 's' : ''} resolved`);
      if (newCount > 0) parts.push(`${newCount} new item${newCount > 1 ? 's' : ''} found`);
      if (conflictCount > 0) parts.push(`⚠️ ${conflictCount} conflict${conflictCount > 1 ? 's' : ''} detected`);
      if (parts.length > 0) toast.info(`Analysis: ${parts.join(' · ')}`);
    } catch (err) {
      console.warn('Differential analysis failed (non-blocking):', err);
    }
  },

  // ── Smart Merge for Gaps / Confusions / Suggestions ─────────────────────

  getMergePreview: async (entityType: 'gap' | 'confusion' | 'suggestion', ids: string[]) => {
    const currentHub = get().currentHub;
    if (!currentHub || ids.length < 2) return null;

    const collection: any[] = entityType === 'gap' ? (currentHub.gaps || []) : entityType === 'confusion' ? (currentHub.confusions || []) : (currentHub.suggestions || []);
    const items = collection.filter((item: any) => ids.includes(item.id));
    if (items.length < 2) return null;

    try {
      return await validateMerge(entityType, items[0], items[1]);
    } catch {
      toast.error('Failed to get AI merge preview');
      return null;
    }
  },

  executeGapMerge: async (ids: string[], merged: Record<string, any>, aiConfidence: number, aiReason: string) => {
    const currentHub = get().currentHub;
    if (!currentHub) return false;
    try {
      const sourceItems = (currentHub.gaps || []).filter((g: any) => ids.includes(g.id));
      const allSourceEvents = sourceItems.flatMap((g: any) => g.sourceEvents || []);
      await Promise.all(ids.map(id => db.gaps.delete(id)));
      await db.gaps.create({
        roadmapItemId: currentHub.roadmapItemId,
        title: merged.title || 'Merged Gap',
        description: merged.description || '',
        severity: (merged.severity as any) || 'Medium',
        status: 'Open',
        sourceType: 'AI',
        suggestions: merged.suggestions || [],
        sourceEvents: [...allSourceEvents, { id: `${Date.now()}`, type: 'merged' as any, sourceType: 'ai-merge' as any, actorType: 'human' as any, at: Date.now(), excerpt: `AI confidence: ${aiConfidence}% — ${aiReason}` }],
      });
      await get().fetchKnowledgeHub(currentHub.roadmapItemId);
      toast.success(`${ids.length} gaps merged`);
      return true;
    } catch { toast.error('Merge failed'); return false; }
  },

  executeConfusionMerge: async (ids: string[], merged: Record<string, any>, aiConfidence: number, aiReason: string) => {
    const currentHub = get().currentHub;
    if (!currentHub) return false;
    try {
      const sourceItems = (currentHub.confusions || []).filter((c: any) => ids.includes(c.id));
      const allSourceEvents = sourceItems.flatMap((c: any) => c.sourceEvents || []);
      await Promise.all(ids.map(id => db.confusions.delete(id)));
      await db.confusions.create({
        roadmapItemId: currentHub.roadmapItemId,
        question: merged.question || merged.title || 'Merged Confusion',
        context: merged.context || merged.description || '',
        sources: sourceItems.flatMap((c: any) => c.sources || []),
        status: 'Open',
        sourceEvents: [...allSourceEvents, { id: `${Date.now()}`, type: 'merged' as any, sourceType: 'ai-merge' as any, actorType: 'human' as any, at: Date.now(), excerpt: `AI confidence: ${aiConfidence}% — ${aiReason}` }],
      });
      await get().fetchKnowledgeHub(currentHub.roadmapItemId);
      toast.success(`${ids.length} confusions merged`);
      return true;
    } catch { toast.error('Merge failed'); return false; }
  },

  executeSuggestionMerge: async (ids: string[], merged: Record<string, any>, aiConfidence: number, aiReason: string) => {
    const currentHub = get().currentHub;
    if (!currentHub) return false;
    try {
      const sourceItems = (currentHub.suggestions || []).filter((s: any) => ids.includes(s.id));
      const allSourceEvents = sourceItems.flatMap((s: any) => s.sourceEvents || []);
      await Promise.all(ids.map(id => db.suggestions.delete(id)));
      await db.suggestions.create({
        roadmapItemId: currentHub.roadmapItemId,
        title: merged.title || 'Merged Suggestion',
        description: merged.description || '',
        type: (merged.type as any) || 'Enhancement',
        status: 'Pending',
        generatedBy: 'AI',
        impact: merged.impact,
        effort: merged.effort,
        sourceEvents: [...allSourceEvents, { id: `${Date.now()}`, type: 'merged' as any, sourceType: 'ai-merge' as any, actorType: 'human' as any, at: Date.now(), excerpt: `AI confidence: ${aiConfidence}% — ${aiReason}` }],
      });
      await get().fetchKnowledgeHub(currentHub.roadmapItemId);
      toast.success(`${ids.length} suggestions merged`);
      return true;
    } catch { toast.error('Merge failed'); return false; }
  },

  // ── Conflict management ──────────────────────────────────────────────────

  resolveConflict: async (conflictId: string, resolution: string) => {
    try {
      await db.conflicts.update(conflictId, { status: 'Resolved', resolution });
      toast.success('Conflict resolved');
      const hub = get().currentHub;
      if (hub) {
        set({ currentHub: { ...hub, conflicts: (hub.conflicts || []).map((c: any) => c.id === conflictId ? { ...c, status: 'Resolved', resolution } : c) } });
      }
      return true;
    } catch { toast.error('Failed to resolve conflict'); return false; }
  },

  dismissConflict: async (conflictId: string) => {
    try {
      await db.conflicts.update(conflictId, { status: 'Dismissed' });
      toast.success('Conflict dismissed');
      const hub = get().currentHub;
      if (hub) {
        set({ currentHub: { ...hub, conflicts: (hub.conflicts || []).map((c: any) => c.id === conflictId ? { ...c, status: 'Dismissed' } : c) } });
      }
      return true;
    } catch { toast.error('Failed to dismiss conflict'); return false; }
  },

  deleteConflict: async (conflictId: string) => {
    try {
      await db.conflicts.delete(conflictId);
      toast.success('Conflict deleted');
      const hub = get().currentHub;
      if (hub) {
        set({ currentHub: { ...hub, conflicts: (hub.conflicts || []).filter((c: any) => c.id !== conflictId), stats: { ...hub.stats, totalConflicts: hub.stats.totalConflicts - 1 } } });
      }
      return true;
    } catch { toast.error('Failed to delete conflict'); return false; }
  },

  // ── Formal Requirements ─────────────────────────────────────────────────

  createFormalRequirement: async (data: Omit<FormalRequirement, 'id' | 'createdAt' | 'updatedAt'>) => {
    try {
      const created = await db.formalRequirements.create(data);
      set((state) => {
        if (!state.currentHub) return state;
        return {
          currentHub: {
            ...state.currentHub,
            formalRequirements: [...(state.currentHub.formalRequirements ?? []), created],
          },
        };
      });
      toast.success('Requirement added');
      return true;
    } catch (error) {
      console.error('Error creating formal requirement:', error);
      toast.error('Failed to add requirement');
      return false;
    }
  },

  updateFormalRequirement: async (id: string, updates: Partial<FormalRequirement>) => {
    try {
      const updated = await db.formalRequirements.update(id, updates);
      set((state) => {
        if (!state.currentHub) return state;
        return {
          currentHub: {
            ...state.currentHub,
            formalRequirements: (state.currentHub.formalRequirements ?? []).map((r: any) =>
              r.id === id ? { ...r, ...updated } : r
            ),
          },
        };
      });
      toast.success('Requirement updated');
      return true;
    } catch (error) {
      console.error('Error updating formal requirement:', error);
      toast.error('Failed to update requirement');
      return false;
    }
  },

  deleteFormalRequirement: async (id: string) => {
    try {
      await db.formalRequirements.delete(id);
      set((state) => {
        if (!state.currentHub) return state;
        return {
          currentHub: {
            ...state.currentHub,
            formalRequirements: (state.currentHub.formalRequirements ?? []).filter((r: any) => r.id !== id),
          },
        };
      });
      toast.success('Requirement deleted');
      return true;
    } catch (error) {
      console.error('Error deleting formal requirement:', error);
      toast.error('Failed to delete requirement');
      return false;
    }
  },

  /**
   * Reset store
   */
  reset: () => {
    set({
      currentHub: null,
      isLoading: false,
      isAnalyzing: false,
      analysisProgress: null,
    });
  },
}));