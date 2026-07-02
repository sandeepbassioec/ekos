import { create } from 'zustand';
import { db } from '../lib/db';
import type { PI, Sprint } from '../lib/db/types';
import { toast } from 'sonner';

export type PIStatus = 'planning' | 'active' | 'completed' | 'cancelled';
export type SprintStatus = 'planning' | 'active' | 'completed';

interface PIState {
  pis: PI[];
  sprints: Sprint[];
  isLoading: boolean;
  isSaving: boolean;
  currentProjectId: string | null;

  fetchPIs: (projectId: string) => Promise<void>;
  createPI: (projectId: string, data: {
    name: string;
    startDate: string;
    objectives?: string;
    createdBy: string;
  }) => Promise<PI | null>;
  updatePI: (id: string, updates: Partial<PI>) => Promise<void>;
  deletePI: (piId: string) => Promise<void>;
  updateSprint: (id: string, updates: Partial<Sprint>) => Promise<void>;
  sprintsForPI: (piId: string) => Sprint[];
}

/** Auto-generate 6 × 2-week sprints starting from piStartDate */
function generateSprints(
  projectId: string,
  piId: string,
  piStartDate: string,
): Omit<Sprint, 'id' | 'createdAt' | 'updatedAt'>[] {
  const start = new Date(piStartDate);
  return Array.from({ length: 6 }, (_, i) => {
    const sprintStart = new Date(start);
    sprintStart.setDate(start.getDate() + i * 14);
    const sprintEnd = new Date(sprintStart);
    sprintEnd.setDate(sprintStart.getDate() + 13);
    return {
      projectId,
      piId,
      name: `Sprint ${i + 1}`,
      startDate: sprintStart.toISOString().split('T')[0],
      endDate: sprintEnd.toISOString().split('T')[0],
      status: 'planning',
      goals: '',
    };
  });
}

export const usePIStore = create<PIState>((set, get) => ({
  pis: [],
  sprints: [],
  isLoading: false,
  isSaving: false,
  currentProjectId: null,

  fetchPIs: async (projectId) => {
    set({ isLoading: true, currentProjectId: projectId });
    try {
      const [pis, sprints] = await Promise.all([
        db.pis.getByProject(projectId),
        db.sprints.getByProject(projectId),
      ]);
      // Sort PIs by startDate ascending
      pis.sort((a, b) => (a.startDate ?? '').localeCompare(b.startDate ?? ''));
      set({ pis, sprints });
    } catch (err) {
      console.error('fetchPIs failed:', err);
      toast.error('Failed to load PI data.');
    } finally {
      set({ isLoading: false });
    }
  },

  createPI: async (projectId, { name, startDate, objectives, createdBy }) => {
    set({ isSaving: true });
    try {
      const endDate = (() => {
        const d = new Date(startDate);
        d.setDate(d.getDate() + 83); // 6 sprints × 14 days = 84 days, end on day 83
        return d.toISOString().split('T')[0];
      })();

      const pi = await db.pis.create({
        projectId,
        name,
        startDate,
        endDate,
        status: 'planning',
        objectives: objectives ?? '',
      });

      // Auto-generate 6 sprints
      const sprintDefs = generateSprints(projectId, pi.id, startDate);
      const savedSprints = await Promise.all(sprintDefs.map(s => db.sprints.create(s)));

      set(state => ({
        pis: [...state.pis, pi].sort((a, b) => (a.startDate ?? '').localeCompare(b.startDate ?? '')),
        sprints: [...state.sprints, ...savedSprints],
      }));

      toast.success(`${name} created with 6 sprints.`);
      return pi;
    } catch (err: any) {
      toast.error(err.message || 'Failed to create PI.');
      return null;
    } finally {
      set({ isSaving: false });
    }
  },

  updatePI: async (id, updates) => {
    set({ isSaving: true });
    try {
      const updated = await db.pis.update(id, updates);
      set(state => ({ pis: state.pis.map(p => p.id === id ? updated : p) }));
    } catch (err: any) {
      toast.error(err.message || 'Failed to update PI.');
    } finally {
      set({ isSaving: false });
    }
  },

  deletePI: async (piId) => {
    set({ isSaving: true });
    try {
      // Delete all sprints belonging to this PI first
      const sprints = get().sprints.filter(s => s.piId === piId);
      await Promise.all(sprints.map(s => db.sprints.delete(s.id)));
      await db.pis.delete(piId);
      set(state => ({
        pis: state.pis.filter(p => p.id !== piId),
        sprints: state.sprints.filter(s => s.piId !== piId),
      }));
      toast.success('Program Increment deleted.');
    } catch (err: any) {
      toast.error(err.message || 'Failed to delete PI.');
    } finally {
      set({ isSaving: false });
    }
  },

  updateSprint: async (id, updates) => {
    set({ isSaving: true });
    try {
      const updated = await db.sprints.update(id, updates);
      set(state => ({ sprints: state.sprints.map(s => s.id === id ? updated : s) }));
    } catch (err: any) {
      toast.error(err.message || 'Failed to update sprint.');
    } finally {
      set({ isSaving: false });
    }
  },

  sprintsForPI: (piId) => get().sprints.filter(s => s.piId === piId),
}));
