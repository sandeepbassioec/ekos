import { create } from 'zustand';
import { db } from '../lib/db';
import { useAuthStore } from './authStore';
import type { WorkflowRequest, WorkflowRequestType } from '../lib/db/types';

/**
 * Cross-role workflow store (Epic D / catalog Vol 12).
 *
 * Architects raise lightweight requests — clarifications, missing-requirement flags,
 * reviews, approvals, hand-offs — routed to a ROLE and/or a specific USER. The recipient
 * acts from their Inbox. Persistence goes through the swappable DAL (`lib/db`), the same
 * seam every other store uses; Firebase is primary today.
 *
 * The inbox is assembled from three single-field Firestore queries (assigned-to-my-role,
 * assigned-to-me, raised-by-me) and merged client-side, so no composite index is needed.
 */

export type { WorkflowRequest };

function currentUser() {
  return useAuthStore.getState().user;
}

/** Merge request lists, de-duplicating by id, newest-updated first. */
function mergeById(...lists: WorkflowRequest[][]): WorkflowRequest[] {
  const map = new Map<string, WorkflowRequest>();
  for (const list of lists) for (const r of list) map.set(r.id, r);
  return Array.from(map.values()).sort((a, b) =>
    (b.updatedAt || '').localeCompare(a.updatedAt || ''),
  );
}

export interface RaiseRequestInput {
  type: WorkflowRequestType;
  title: string;
  description?: string;
  targetRole?: string;
  targetUserId?: string;
  projectId?: string;
  subjectType?: string;
  subjectId?: string;
  metadata?: Record<string, unknown>;
}

interface WorkflowState {
  assigned: WorkflowRequest[]; // routed to me (by role and/or my userId)
  raisedByMe: WorkflowRequest[]; // requests I opened
  isLoading: boolean;

  refresh: () => Promise<void>;
  raiseRequest: (input: RaiseRequestInput) => Promise<WorkflowRequest | null>;
  respond: (id: string, response: string) => Promise<void>;
  decide: (id: string, decision: 'approved' | 'rejected', note?: string) => Promise<void>;
  resolve: (id: string) => Promise<void>;
  withdraw: (id: string) => Promise<void>;
}

export const useWorkflowStore = create<WorkflowState>((set) => {
  // Replace a just-updated request in whichever feed(s) hold it.
  const applyUpdate = (updated: WorkflowRequest) => {
    const replace = (list: WorkflowRequest[]) =>
      list.map((r) => (r.id === updated.id ? { ...r, ...updated } : r));
    set((s) => ({ assigned: replace(s.assigned), raisedByMe: replace(s.raisedByMe) }));
  };

  return {
    assigned: [],
    raisedByMe: [],
    isLoading: false,

    refresh: async () => {
      const user = currentUser();
      if (!user) return;
      set({ isLoading: true });
      try {
        const [byRole, byUser, mine] = await Promise.all([
          user.role ? db.workflowRequests.getAssignedToRole(user.role) : Promise.resolve([]),
          db.workflowRequests.getAssignedToUser(user.id),
          db.workflowRequests.getRaisedBy(user.id),
        ]);
        set({ assigned: mergeById(byRole, byUser), raisedByMe: mergeById(mine), isLoading: false });
      } catch (error) {
        console.error('Failed to load workflow inbox:', error);
        set({ isLoading: false });
      }
    },

    raiseRequest: async (input) => {
      const user = currentUser();
      if (!user) return null;
      try {
        const created = await db.workflowRequests.create({
          type: input.type,
          title: input.title,
          description: input.description,
          raiserId: user.id,
          raiserName: user.fullName || user.email,
          targetRole: input.targetRole,
          targetUserId: input.targetUserId,
          projectId: input.projectId,
          subjectType: input.subjectType,
          subjectId: input.subjectId,
          metadata: input.metadata,
          status: 'open',
        });
        set((s) => ({ raisedByMe: [created, ...s.raisedByMe] }));
        return created;
      } catch (error) {
        console.error('Failed to raise request:', error);
        throw error;
      }
    },

    respond: async (id, response) => {
      const user = currentUser();
      const updated = await db.workflowRequests.update(id, {
        status: 'responded',
        response,
        resolvedById: user?.id,
        resolvedByName: user?.fullName || user?.email,
      });
      applyUpdate(updated);
    },

    decide: async (id, decision, note) => {
      const user = currentUser();
      const updated = await db.workflowRequests.update(id, {
        status: decision,
        response: note,
        resolvedById: user?.id,
        resolvedByName: user?.fullName || user?.email,
      });
      applyUpdate(updated);
    },

    resolve: async (id) => {
      const updated = await db.workflowRequests.update(id, { status: 'resolved' });
      applyUpdate(updated);
    },

    withdraw: async (id) => {
      const updated = await db.workflowRequests.update(id, { status: 'withdrawn' });
      applyUpdate(updated);
    },
  };
});
