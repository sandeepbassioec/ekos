import { create } from 'zustand';
import { db, auth } from '../lib/db';
import { toast } from 'sonner';
import {
  Roadmap,
  RoadmapItem,
  RoadmapWithItems,
  CreateRoadmapDTO,
  CreateRoadmapItemDTO,
  UpdateRoadmapItemDTO,
  RoadmapType,
  RoadmapViewMode,
} from '../types/roadmap';

// ============================================
// ROADMAP STORE STATE
// ============================================
interface RoadmapStore {
  // State
  roadmaps: Roadmap[];
  roadmapItems: RoadmapItem[];
  currentRoadmap: RoadmapWithItems | null;
  selectedYear: number;
  selectedType: RoadmapType;
  viewMode: RoadmapViewMode;
  isLoading: boolean;

  // Actions - Roadmap
  fetchRoadmaps: (projectId: string) => Promise<void>;
  fetchRoadmapByYearAndType: (projectId: string, year: number, type: RoadmapType) => Promise<void>;
  createRoadmap: (data: CreateRoadmapDTO) => Promise<Roadmap | null>;
  deleteRoadmap: (roadmapId: string) => Promise<boolean>;

  // Actions - RoadmapItem
  fetchRoadmapItems: (roadmapId: string) => Promise<void>;
  createRoadmapItem: (data: CreateRoadmapItemDTO) => Promise<RoadmapItem | null>;
  updateRoadmapItem: (itemId: string, data: UpdateRoadmapItemDTO) => Promise<boolean>;
  deleteRoadmapItem: (itemId: string) => Promise<boolean>;

  // UI State Actions
  setSelectedYear: (year: number) => void;
  setSelectedType: (type: RoadmapType) => void;
  setViewMode: (mode: RoadmapViewMode) => void;

  // Reset
  reset: () => void;
}

// ============================================
// STORE IMPLEMENTATION
// ============================================
export const useRoadmapStore = create<RoadmapStore>((set, get) => ({
  // Initial State
  roadmaps: [],
  roadmapItems: [],
  currentRoadmap: null,
  selectedYear: new Date().getFullYear(),
  selectedType: 'product',
  viewMode: 'quarterly',
  isLoading: false,

  // ============================================
  // ROADMAP ACTIONS
  // ============================================

  /**
   * Fetch all roadmaps for a project
   */
  fetchRoadmaps: async (projectId: string) => {
    set({ isLoading: true });
    try {
      const roadmaps = await db.roadmaps.getByProject(projectId);
      set({ roadmaps, isLoading: false });
    } catch (error) {
      console.error('Error fetching roadmaps:', error);
      toast.error('Failed to fetch roadmaps');
      set({ isLoading: false });
    }
  },

  /**
   * Fetch roadmap by year and type (with items)
   */
  fetchRoadmapByYearAndType: async (projectId: string, year: number, type: RoadmapType) => {
    set({ isLoading: true });
    try {
      const roadmapData = await db.roadmaps.getByYearAndType(projectId, year, type);
      if (!roadmapData) {
        set({ currentRoadmap: null, roadmapItems: [], isLoading: false });
        return;
      }
      const itemsData = await db.roadmapItems.getByRoadmap(roadmapData.id);
      const roadmapWithItems: RoadmapWithItems = {
        ...roadmapData,
        items: itemsData,
      };
      set({
        currentRoadmap: roadmapWithItems,
        roadmapItems: itemsData,
        isLoading: false
      });
    } catch (error) {
      console.error('Error fetching roadmap:', error);
      toast.error('Failed to fetch roadmap');
      set({ isLoading: false });
    }
  },

  /**
   * Create a new roadmap
   */
  createRoadmap: async (data: CreateRoadmapDTO) => {
    try {
      const user = await auth.getUser();
      if (!user) {
        toast.error('You must be logged in');
        return null;
      }
      const roadmap = await db.roadmaps.create({
        ...data,
        createdBy: user.id,
        ownerId: data.ownerId || user.id,
      });
      toast.success('Roadmap created successfully');
      set((state) => ({ roadmaps: [...state.roadmaps, roadmap] }));
      if (data.year === get().selectedYear && data.type === get().selectedType) {
        await get().fetchRoadmapByYearAndType(data.projectId, data.year, data.type);
      }
      return roadmap;
    } catch (error: any) {
      console.error('Error creating roadmap:', error);
      if (error.message?.includes('duplicate') || error.message?.includes('exists')) {
        toast.error(`${data.type} roadmap for ${data.year} already exists`);
      } else {
        toast.error('Failed to create roadmap');
      }
      return null;
    }
  },

  /**
   * Delete a roadmap
   */
  deleteRoadmap: async (roadmapId: string) => {
    try {
      await db.roadmaps.delete(roadmapId);
      toast.success('Roadmap deleted successfully');
      set((state) => ({
        roadmaps: state.roadmaps.filter(r => r.id !== roadmapId),
        currentRoadmap: state.currentRoadmap?.id === roadmapId ? null : state.currentRoadmap,
        roadmapItems: state.currentRoadmap?.id === roadmapId ? [] : state.roadmapItems,
      }));
      return true;
    } catch (error) {
      console.error('Error deleting roadmap:', error);
      toast.error('Failed to delete roadmap');
      return false;
    }
  },

  // ============================================
  // ROADMAP ITEM ACTIONS
  // ============================================

  /**
   * Fetch items for a roadmap
   */
  fetchRoadmapItems: async (roadmapId: string) => {
    set({ isLoading: true });
    try {
      const roadmapItems = await db.roadmapItems.getByRoadmap(roadmapId);
      set({ roadmapItems, isLoading: false });
    } catch (error) {
      console.error('Error fetching roadmap items:', error);
      toast.error('Failed to fetch roadmap items');
      set({ isLoading: false });
    }
  },

  /**
   * Create a new roadmap item
   */
  createRoadmapItem: async (data: CreateRoadmapItemDTO) => {
    try {
      // Validate required fields
      if (!data.title || data.title.trim() === '') {
        toast.error('Title is required');
        return null;
      }

      const item = await db.roadmapItems.create({
        ...data,
        dependencies: data.dependencies || [],
      });
      toast.success('Roadmap item added successfully');
      set((state) => ({
        roadmapItems: [...state.roadmapItems, item],
        currentRoadmap: state.currentRoadmap ? {
          ...state.currentRoadmap,
          items: [...state.currentRoadmap.items, item],
        } : null,
      }));
      return item;
    } catch (error: any) {
      console.error('Error creating roadmap item:', error);
      // Show specific error message if available
      const errorMessage = error?.message || 'Failed to create roadmap item';
      toast.error(errorMessage);
      return null;
    }
  },

  /**
   * Update a roadmap item
   */
  updateRoadmapItem: async (itemId: string, data: UpdateRoadmapItemDTO) => {
    try {
      await db.roadmapItems.update(itemId, data);
      toast.success('Roadmap item updated successfully');
      set((state) => ({
        roadmapItems: state.roadmapItems.map(item =>
          item.id === itemId ? { ...item, ...data } : item
        ),
        currentRoadmap: state.currentRoadmap ? {
          ...state.currentRoadmap,
          items: state.currentRoadmap.items.map(item =>
            item.id === itemId ? { ...item, ...data } : item
          ),
        } : null,
      }));
      return true;
    } catch (error) {
      console.error('Error updating roadmap item:', error);
      toast.error('Failed to update roadmap item');
      return false;
    }
  },

  /**
   * Delete a roadmap item
   */
  deleteRoadmapItem: async (itemId: string) => {
    try {
      await db.roadmapItems.delete(itemId);
      toast.success('Roadmap item deleted successfully');
      set((state) => ({
        roadmapItems: state.roadmapItems.filter(item => item.id !== itemId),
        currentRoadmap: state.currentRoadmap ? {
          ...state.currentRoadmap,
          items: state.currentRoadmap.items.filter(item => item.id !== itemId),
        } : null,
      }));
      return true;
    } catch (error) {
      console.error('Error deleting roadmap item:', error);
      toast.error('Failed to delete roadmap item');
      return false;
    }
  },

  // ============================================
  // UI STATE ACTIONS
  // ============================================

  setSelectedYear: (year: number) => {
    set({ selectedYear: year });
  },

  setSelectedType: (type: RoadmapType) => {
    set({ selectedType: type });
  },

  setViewMode: (mode: RoadmapViewMode) => {
    set({ viewMode: mode });
  },

  // ============================================
  // RESET
  // ============================================

  reset: () => {
    set({
      roadmaps: [],
      roadmapItems: [],
      currentRoadmap: null,
      selectedYear: new Date().getFullYear(),
      selectedType: 'product',
      viewMode: 'quarterly',
      isLoading: false,
    });
  },
}));
