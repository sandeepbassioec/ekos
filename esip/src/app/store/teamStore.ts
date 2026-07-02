import { create } from 'zustand';
import { db } from '../lib/db';
import type { ProjectMember, ProjectRole, UserProfile } from '../lib/db/types';
import { toast } from 'sonner';

interface TeamState {
  members: ProjectMember[];
  allUsers: UserProfile[];
  isLoading: boolean;
  isSaving: boolean;
  currentProjectId: string | null;

  fetchTeam: (projectId: string) => Promise<void>;
  fetchAllUsers: () => Promise<void>;
  addMember: (projectId: string, userId: string, roles: ProjectRole[], addedBy: string) => Promise<void>;
  updateMemberRoles: (memberId: string, roles: ProjectRole[]) => Promise<void>;
  removeMember: (memberId: string) => Promise<void>;
  /** All roles the user holds on the current project. Empty array if not a member. */
  currentUserRoles: (userId: string) => ProjectRole[];
  /** True if the user holds at least one of the given roles. */
  hasRole: (userId: string, ...roles: ProjectRole[]) => boolean;
  isMember: (userId: string) => boolean;
}

export const useTeamStore = create<TeamState>((set, get) => ({
  members: [],
  allUsers: [],
  isLoading: false,
  isSaving: false,
  currentProjectId: null,

  fetchTeam: async (projectId) => {
    set({ isLoading: true, currentProjectId: projectId });
    try {
      const members = await db.projectMembers.getByProject(projectId);
      set({ members });
    } catch (err) {
      console.error('fetchTeam failed:', err);
    } finally {
      set({ isLoading: false });
    }
  },

  fetchAllUsers: async () => {
    try {
      const allUsers = await db.users.getAll();
      set({ allUsers });
    } catch (err) {
      console.error('fetchAllUsers failed:', err);
    }
  },

  addMember: async (projectId, userId, roles, addedBy) => {
    const existing = get().members.find(m => m.userId === userId);
    if (existing) { toast.error('This user is already a team member.'); return; }
    const user = get().allUsers.find(u => u.id === userId);
    if (!user) return;

    set({ isSaving: true });
    try {
      const member = await db.projectMembers.create({
        projectId,
        userId,
        userEmail: user.email,
        userFullName: user.fullName,
        projectRoles: roles,
        addedBy,
      });
      set(s => ({ members: [...s.members, member] }));
      toast.success(`${user.fullName || user.email} added to team.`);
    } catch (err: any) {
      toast.error(err.message || 'Failed to add member.');
    } finally {
      set({ isSaving: false });
    }
  },

  updateMemberRoles: async (memberId, roles) => {
    set({ isSaving: true });
    try {
      const updated = await db.projectMembers.update(memberId, { projectRoles: roles });
      set(s => ({ members: s.members.map(m => m.id === memberId ? updated : m) }));
      toast.success('Roles updated.');
    } catch (err: any) {
      toast.error(err.message || 'Failed to update roles.');
    } finally {
      set({ isSaving: false });
    }
  },

  removeMember: async (memberId) => {
    set({ isSaving: true });
    try {
      await db.projectMembers.delete(memberId);
      set(s => ({ members: s.members.filter(m => m.id !== memberId) }));
      toast.success('Member removed.');
    } catch (err: any) {
      toast.error(err.message || 'Failed to remove member.');
    } finally {
      set({ isSaving: false });
    }
  },

  currentUserRoles: (userId) => {
    const m = get().members.find(m => m.userId === userId);
    return m?.projectRoles ?? [];
  },

  hasRole: (userId, ...roles) => {
    const m = get().members.find(m => m.userId === userId);
    if (!m) return false;
    return roles.some(r => m.projectRoles.includes(r));
  },

  isMember: (userId) => get().members.some(m => m.userId === userId),
}));
