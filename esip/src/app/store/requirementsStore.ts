import { create } from 'zustand';
import { db } from '../lib/db';
import type { PI, Sprint, Requirement } from '../lib/db/types';

/**
 * Requirements / PI / Sprint store.
 *
 * Persistence goes through the swappable Database Abstraction Layer (`lib/db`),
 * the same seam every other store uses. Firebase is the primary provider today;
 * a customer can hook in a different database by implementing `IDatabase` and
 * flipping `DB_PROVIDER` in `lib/db/index.ts` — no store changes required.
 *
 * (Previously this imported a removed Supabase `db` helper with flat method names
 *  — db.getPIs, db.createPI, db.bulkCreateRequirements — that no longer existed,
 *  so every action threw `TypeError: db.getPIs is not a function` at runtime.)
 */

// Re-export the canonical DAL types so existing imports keep working.
export type { PI, Sprint, Requirement };

interface RequirementsState {
  pis: PI[];
  sprints: Sprint[];
  requirements: Requirement[];
  isLoading: boolean;

  // PI Actions
  fetchPIs: (projectId: string) => Promise<void>;
  createPI: (pi: Partial<PI>) => Promise<PI>;
  updatePI: (id: string, updates: Partial<PI>) => Promise<void>;
  deletePI: (id: string) => Promise<void>;

  // Sprint Actions
  fetchSprints: (projectId: string) => Promise<void>;
  createSprint: (sprint: Partial<Sprint>) => Promise<Sprint>;
  updateSprint: (id: string, updates: Partial<Sprint>) => Promise<void>;
  deleteSprint: (id: string) => Promise<void>;

  // Requirement Actions
  fetchRequirements: (projectId: string, filters?: any) => Promise<void>;
  createRequirement: (requirement: Partial<Requirement>) => Promise<Requirement>;
  updateRequirement: (id: string, updates: Partial<Requirement>) => Promise<void>;
  deleteRequirement: (id: string) => Promise<void>;
  bulkCreateRequirements: (projectId: string, requirements: Partial<Requirement>[] | any) => Promise<void>;
}

export const useRequirementsStore = create<RequirementsState>((set, get) => ({
  pis: [],
  sprints: [],
  requirements: [],
  isLoading: false,

  // ── PI Actions ────────────────────────────────────────────────────────────
  fetchPIs: async (projectId: string) => {
    set({ isLoading: true });
    try {
      const pis = await db.pis.getByProject(projectId);
      set({ pis, isLoading: false });
    } catch (error) {
      console.error('Failed to fetch PIs:', error);
      set({ isLoading: false });
    }
  },

  createPI: async (pi: Partial<PI>) => {
    try {
      const created = await db.pis.create({
        projectId: pi.projectId || '',
        name: pi.name || 'Untitled PI',
        startDate: pi.startDate,
        endDate: pi.endDate,
        status: pi.status || 'planned',
        objectives: pi.objectives,
      });
      set((state) => ({ pis: [...state.pis, created] }));
      return created;
    } catch (error) {
      console.error('Failed to create PI:', error);
      throw error;
    }
  },

  updatePI: async (id: string, updates: Partial<PI>) => {
    try {
      const updated = await db.pis.update(id, updates);
      set((state) => ({ pis: state.pis.map((pi) => (pi.id === id ? { ...pi, ...updated } : pi)) }));
    } catch (error) {
      console.error('Failed to update PI:', error);
      throw error;
    }
  },

  deletePI: async (id: string) => {
    try {
      await db.pis.delete(id);
      set((state) => ({ pis: state.pis.filter((pi) => pi.id !== id) }));
    } catch (error) {
      console.error('Failed to delete PI:', error);
      throw error;
    }
  },

  // ── Sprint Actions ────────────────────────────────────────────────────────
  fetchSprints: async (projectId: string) => {
    set({ isLoading: true });
    try {
      const sprints = await db.sprints.getByProject(projectId);
      set({ sprints, isLoading: false });
    } catch (error) {
      console.error('Failed to fetch sprints:', error);
      set({ isLoading: false });
    }
  },

  createSprint: async (sprint: Partial<Sprint>) => {
    try {
      const created = await db.sprints.create({
        projectId: sprint.projectId || '',
        piId: sprint.piId,
        name: sprint.name || 'Untitled Sprint',
        startDate: sprint.startDate,
        endDate: sprint.endDate,
        status: sprint.status || 'planned',
        goals: sprint.goals,
      });
      set((state) => ({ sprints: [...state.sprints, created] }));
      return created;
    } catch (error) {
      console.error('Failed to create sprint:', error);
      throw error;
    }
  },

  updateSprint: async (id: string, updates: Partial<Sprint>) => {
    try {
      const updated = await db.sprints.update(id, updates);
      set((state) => ({ sprints: state.sprints.map((s) => (s.id === id ? { ...s, ...updated } : s)) }));
    } catch (error) {
      console.error('Failed to update sprint:', error);
      throw error;
    }
  },

  deleteSprint: async (id: string) => {
    try {
      await db.sprints.delete(id);
      set((state) => ({ sprints: state.sprints.filter((s) => s.id !== id) }));
    } catch (error) {
      console.error('Failed to delete sprint:', error);
      throw error;
    }
  },

  // ── Requirement Actions ───────────────────────────────────────────────────
  fetchRequirements: async (projectId: string, _filters?: any) => {
    set({ isLoading: true });
    try {
      const requirements = await db.requirements.getByProject(projectId);
      set({ requirements, isLoading: false });
    } catch (error) {
      console.error('Failed to fetch requirements:', error);
      set({ isLoading: false });
    }
  },

  createRequirement: async (requirement: Partial<Requirement>) => {
    try {
      const created = await db.requirements.create({
        projectId: requirement.projectId || '',
        piId: requirement.piId,
        sprintId: requirement.sprintId,
        title: requirement.title || 'Untitled Requirement',
        description: requirement.description,
        category: requirement.category || 'Functional',
        priority: requirement.priority || 'Medium',
        status: requirement.status || 'Draft',
        tags: requirement.tags || [],
        metadata: requirement.metadata ?? null,
        createdBy: requirement.createdBy || 'system',
      });
      set((state) => ({ requirements: [...state.requirements, created] }));
      return created;
    } catch (error) {
      console.error('Failed to create requirement:', error);
      throw error;
    }
  },

  updateRequirement: async (id: string, updates: Partial<Requirement>) => {
    try {
      const updated = await db.requirements.update(id, updates);
      set((state) => ({ requirements: state.requirements.map((r) => (r.id === id ? { ...r, ...updated } : r)) }));
    } catch (error) {
      console.error('Failed to update requirement:', error);
      throw error;
    }
  },

  deleteRequirement: async (id: string) => {
    try {
      await db.requirements.delete(id);
      set((state) => ({ requirements: state.requirements.filter((r) => r.id !== id) }));
    } catch (error) {
      console.error('Failed to delete requirement:', error);
      throw error;
    }
  },

  bulkCreateRequirements: async (projectId: string, requirements: Partial<Requirement>[] | any) => {
    try {
      // The Comprehensive Requirements Wizard passes a single ComprehensiveRequirements
      // object rather than an array — normalize so either shape works.
      const items: any[] = Array.isArray(requirements) ? requirements : [requirements];
      const payloads = items.map((item) => ({
        piId: item?.piId,
        sprintId: item?.sprintId,
        title: item?.title || 'Comprehensive Requirements',
        description: item?.description,
        category: item?.category || 'Comprehensive',
        priority: item?.priority || 'Medium',
        status: item?.status || 'Draft',
        tags: item?.tags || [],
        // Preserve the full payload (e.g. the ComprehensiveRequirements object).
        // Firestore is configured with ignoreUndefinedProperties, so nested
        // undefined fields from skipped wizard steps are dropped safely.
        metadata: item,
        createdBy: item?.createdBy || 'system',
      }));
      const created = await db.requirements.bulkCreate(projectId, payloads);
      set((state) => ({ requirements: [...state.requirements, ...created] }));
    } catch (error) {
      console.error('Failed to bulk create requirements:', error);
      throw error;
    }
  },
}));
