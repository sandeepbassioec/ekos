import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { EstimationMethod, ProjectSetup, Screen, RiskFactor, EstimationResults, CapacityPlan } from './estimationStore';
import { db, auth } from '../lib/db';

// Saved Project (snapshot of estimation state)
export interface SavedProject {
  id: string;
  name: string;
  description?: string;
  createdAt: string;
  updatedAt: string;
  
  // Tags for organization
  tags?: string[];
  status: 'DRAFT' | 'IN_PROGRESS' | 'COMPLETED' | 'ARCHIVED';
  
  // Estimation data
  selectedMethods: EstimationMethod[];
  project: ProjectSetup;
  screens: Screen[];
  contingency: number;
  riskFactors: RiskFactor[];
  results: EstimationResults | null;
  capacityPlan: CapacityPlan | null;
  
  // Metadata
  isTemplate?: boolean; // Can be used as a template
  templateName?: string;
  
  // Backend fields
  estimationTechnique?: string;
  teamId?: string;
}

// Projects Store State
interface ProjectsState {
  // All saved projects
  projects: SavedProject[];
  
  // Current active project ID
  currentProjectId: string | null;
  
  // Loading state
  isLoading: boolean;
  
  // Actions
  fetchProjects: () => Promise<void>;
  saveProject: (project: Omit<SavedProject, 'id' | 'createdAt' | 'updatedAt'>) => Promise<string>;
  updateProject: (id: string, updates: Partial<SavedProject>) => Promise<void>;
  deleteProject: (id: string) => Promise<void>;
  loadProject: (id: string) => SavedProject | null;
  duplicateProject: (id: string) => Promise<string | null>;
  setCurrentProject: (id: string | null) => void;
  
  // Template management
  saveAsTemplate: (id: string, templateName: string) => void;
  getTemplates: () => SavedProject[];
  
  // Project list management
  getAllProjects: () => SavedProject[];
  getProjectsByStatus: (status: SavedProject['status']) => SavedProject[];
  searchProjects: (query: string) => SavedProject[];
}

export const useProjectsStore = create<ProjectsState>()(
  persist(
    (set, get) => ({
      projects: [],
      currentProjectId: null,
      isLoading: false,

      fetchProjects: async () => {
        set({ isLoading: true });
        try {
          const user = await auth.getUser();
          if (!user) {
            set({ isLoading: false });
            return;
          }
          const backendProjects = await db.projects.getAll(user.id);
          const projects: SavedProject[] = backendProjects.map((p: any) => ({
            id: p.id,
            name: p.name,
            description: p.description,
            createdAt: p.createdAt,
            updatedAt: p.updatedAt,
            status: p.status?.toUpperCase() || 'DRAFT',
            selectedMethods: [],
            project: {} as ProjectSetup,
            screens: [],
            contingency: 0,
            riskFactors: [],
            results: null,
            capacityPlan: null,
            estimationTechnique: p.estimationTechnique,
            teamId: p.teamId,
          }));
          set({ projects, isLoading: false });
        } catch (error) {
          console.error('Failed to fetch projects:', error);
          set({ isLoading: false });
        }
      },

      saveProject: async (projectData) => {
        try {
          const user = await auth.getUser();
          if (!user) throw new Error('User not authenticated');
          const created = await db.projects.create({
            userId: user.id,
            name: projectData.name,
            description: projectData.description,
            estimationTechnique: projectData.selectedMethods?.[0] || 'PERT',
            status: projectData.status?.toLowerCase() || 'draft',
          });
          const newProject: SavedProject = {
            ...projectData,
            id: created.id,
            createdAt: created.createdAt,
            updatedAt: created.updatedAt,
          };
          set((state) => ({
            projects: [...state.projects, newProject],
            currentProjectId: newProject.id,
          }));
          return newProject.id;
        } catch (error) {
          console.error('Failed to save project:', error);
          throw error;
        }
      },

      updateProject: async (id, updates) => {
        try {
          const user = await auth.getUser();
          if (!user) throw new Error('User not authenticated');
          await db.projects.update(id, user.id, {
            name: updates.name,
            description: updates.description,
            status: updates.status?.toLowerCase(),
          });
          set((state) => ({
            projects: state.projects.map((project) =>
              project.id === id
                ? { ...project, ...updates, updatedAt: new Date().toISOString() }
                : project
            ),
          }));
        } catch (error) {
          console.error('Failed to update project:', error);
          throw error;
        }
      },

      deleteProject: async (id) => {
        try {
          const user = await auth.getUser();
          if (!user) throw new Error('User not authenticated');
          await db.projects.delete(id, user.id);
          set((state) => ({
            projects: state.projects.filter((project) => project.id !== id),
            currentProjectId: state.currentProjectId === id ? null : state.currentProjectId,
          }));
        } catch (error) {
          console.error('Failed to delete project:', error);
          throw error;
        }
      },

      loadProject: (id) => {
        const project = get().projects.find((p) => p.id === id);
        if (project) {
          set({ currentProjectId: id });
        }
        return project || null;
      },

      duplicateProject: async (id) => {
        const project = get().projects.find((p) => p.id === id);
        if (!project) return null;
        try {
          const user = await auth.getUser();
          if (!user) throw new Error('User not authenticated');
          const duplicated = await db.projects.create({
            userId: user.id,
            name: `${project.name} (Copy)`,
            description: project.description,
            status: 'draft',
            estimationTechnique: project.estimationTechnique,
          });
          const duplicatedProject: SavedProject = {
            ...project,
            id: duplicated.id,
            name: `${project.name} (Copy)`,
            createdAt: duplicated.createdAt,
            updatedAt: duplicated.updatedAt,
            status: 'DRAFT',
            isTemplate: false,
          };
          set((state) => ({
            projects: [...state.projects, duplicatedProject],
          }));
          return duplicatedProject.id;
        } catch (error) {
          console.error('Failed to duplicate project:', error);
          return null;
        }
      },

      setCurrentProject: (id) => {
        set({ currentProjectId: id });
      },

      saveAsTemplate: (id, templateName) => {
        set((state) => ({
          projects: state.projects.map((project) =>
            project.id === id
              ? {
                  ...project,
                  isTemplate: true,
                  templateName,
                  updatedAt: new Date().toISOString(),
                }
              : project
          ),
        }));
      },

      getTemplates: () => {
        return get().projects.filter((p) => p.isTemplate);
      },

      getAllProjects: () => {
        return get().projects.filter((p) => !p.isTemplate);
      },

      getProjectsByStatus: (status) => {
        return get().projects.filter((p) => p.status === status && !p.isTemplate);
      },

      searchProjects: (query) => {
        const lowerQuery = query.toLowerCase();
        return get().projects.filter(
          (p) =>
            !p.isTemplate &&
            (p.name.toLowerCase().includes(lowerQuery) ||
              p.description?.toLowerCase().includes(lowerQuery) ||
              p.tags?.some((tag) => tag.toLowerCase().includes(lowerQuery)))
        );
      },
    }),
    {
      name: 'estimation-projects-storage',
      version: 1,
      partialize: (state) => ({
        projects: state.projects.slice(-50),
        currentProjectId: state.currentProjectId,
      }),
    }
  )
);