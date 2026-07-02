import { create } from 'zustand';
import { persist } from 'zustand/middleware';

// Estimation Methods
export type EstimationMethod = 'PERT' | 'FPA' | 'UCP' | 'STORY_POINTS' | 'TSHIRT';

export type ProjectType = 'WEB' | 'MOBILE' | 'DESKTOP' | 'API' | 'FULLSTACK';

export type ComplexityLevel = 'LOW' | 'MEDIUM' | 'HIGH';

// Screen/Feature Definition
export interface Screen {
  id: string;
  name: string;
  description: string;
  type: 'SCREEN' | 'FEATURE' | 'API' | 'COMPONENT';
  complexity: ComplexityLevel;
  parentId?: string; // For nested features
  level?: number; // 0 = parent, 1 = child, 2 = grandchild (max 3 levels)
  isExpanded?: boolean; // For UI state
  businessPhase?: 'DISCOVERY' | 'DESIGN' | 'DEVELOPMENT' | 'TESTING' | 'DEPLOYMENT'; // Business perspective
  businessValue?: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL'; // Business perspective
  pertData?: {
    optimistic: number;
    mostLikely: number;
    pessimistic: number;
  };
  fpaData?: {
    externalInputs: number;
    externalOutputs: number;
    externalInquiries: number;
    internalLogicalFiles: number;
    externalInterfaceFiles: number;
  };
  ucpData?: {
    // Actor weights
    simpleActors: number; // Weight: 1
    averageActors: number; // Weight: 2
    complexActors: number; // Weight: 3
    // Use Case weights
    simpleUseCases: number; // Weight: 5
    averageUseCases: number; // Weight: 10
    complexUseCases: number; // Weight: 15
  };
  storyPoints?: number;
  tshirtSize?: 'XS' | 'S' | 'M' | 'L' | 'XL' | 'XXL';
}

// Project Setup
export interface ProjectSetup {
  name: string;
  type: ProjectType;
  techStack: string[];
  teamSize: number;
  hourlyRate: number;
  startDate?: Date;
  targetDate?: Date;
}

// Risk Factors
export interface RiskFactor {
  name: string;
  impact: 'LOW' | 'MEDIUM' | 'HIGH';
  probability: number; // 0-100
}

// Estimation Results
export interface EstimationResults {
  totalHours: number;
  totalCost: number;
  timelineWeeks: number;
  confidence: number; // 0-100
  breakdown: {
    development: number;
    testing: number;
    hardening: number;
  };
}

// Method Comparison
export interface MethodResults {
  method: EstimationMethod;
  results: EstimationResults;
}

export interface ComparisonAnalysis {
  methods: MethodResults[];
  variance: number; // Standard deviation of total hours
  averageHours: number;
  minHours: number;
  maxHours: number;
  recommendedMethod: EstimationMethod | null;
  confidenceLevel: 'LOW' | 'MEDIUM' | 'HIGH';
}

// Capacity Planning
export type SprintDuration = 1 | 2 | 3 | 4; // weeks

export interface TeamMember {
  id: string;
  name: string;
  role: string;
  allocation: number; // 0-100% (e.g., 50 = half-time, 100 = full-time)
  availableHoursPerWeek: number;
}

export interface CapacityPlan {
  sprintDuration: SprintDuration;
  teamMembers: TeamMember[];
  productivityFactor: number; // 0-100% (accounts for meetings, overhead, etc. - typically 60-80%)
  includeBufferTime: boolean;
  bufferPercentage: number; // 10-30%
  startDate: Date | null;
  sprints: Sprint[];
}

export interface Sprint {
  id: string;
  number: number;
  startDate: Date;
  endDate: Date;
  plannedHours: number;
  capacityHours: number;
  utilizationPercent: number;
  phase: 'DEVELOPMENT' | 'QA' | 'HARDENING' | 'MIXED';
  tasks: {
    development: number;
    testing: number;
    hardening: number;
  };
}

// Store State
interface EstimationState {
  // Current Step
  currentStep: number;
  
  // Current Project ID (for tracking if we're editing an existing project)
  currentProjectId: string | null;
  
  // Step 1: Method Selection
  selectedMethods: EstimationMethod[];
  
  // Step 2: Project Setup
  project: ProjectSetup;
  
  // Step 3: Scope Definition
  screens: Screen[];
  
  // Step 5: Risk Assessment
  contingency: number;
  riskFactors: RiskFactor[];
  
  // Step 6: Results
  results: EstimationResults | null;
  
  // Method Comparison
  comparisonMode: boolean;
  methodResults: MethodResults[];
  comparisonAnalysis: ComparisonAnalysis | null;
  
  // Capacity Planning
  capacityPlan: CapacityPlan | null;
  
  // Last Saved Timestamp
  lastSaved: string | null;
  
  // Actions
  setCurrentStep: (step: number) => void;
  setCurrentProjectId: (id: string | null) => void;
  setSelectedMethods: (methods: EstimationMethod[]) => void;
  setProject: (project: ProjectSetup) => void;
  addScreen: (screen: Screen) => void;
  updateScreen: (id: string, updates: Partial<Screen>) => void;
  removeScreen: (id: string) => void;
  setContingency: (value: number) => void;
  addRiskFactor: (risk: RiskFactor) => void;
  removeRiskFactor: (index: number) => void;
  calculateResults: () => void;
  setCapacityPlan: (plan: CapacityPlan) => void;
  generateCapacityPlan: () => void;
  addTeamMember: (member: TeamMember) => void;
  updateTeamMember: (id: string, updates: Partial<TeamMember>) => void;
  removeTeamMember: (id: string) => void;
  setComparisonMode: (enabled: boolean) => void;
  calculateMethodComparison: () => void;
  resetEstimation: () => void;
  saveDraft: () => void;
}

// Initial State
const initialState = {
  currentStep: 1,
  currentProjectId: null,
  selectedMethods: [] as EstimationMethod[],
  project: {
    name: '',
    type: 'WEB' as ProjectType,
    techStack: [],
    teamSize: 1,
    hourlyRate: 150,
  },
  screens: [] as Screen[],
  contingency: 15,
  riskFactors: [] as RiskFactor[],
  results: null,
  comparisonMode: false,
  methodResults: [] as MethodResults[],
  comparisonAnalysis: null,
  capacityPlan: null,
  lastSaved: null,
};

// Create Store with Persistence
export const useEstimationStore = create<EstimationState>()(
  persist(
    (set, get) => ({
      ...initialState,

      setCurrentStep: (step) => set({ currentStep: step }),

      setCurrentProjectId: (id) => set({ currentProjectId: id }),

      setSelectedMethods: (methods) => set({ selectedMethods: methods }),

      setProject: (project) => set({ project }),

      addScreen: (screen) =>
        set((state) => ({
          screens: [...state.screens, screen],
        })),

      updateScreen: (id, updates) =>
        set((state) => ({
          screens: state.screens.map((screen) =>
            screen.id === id ? { ...screen, ...updates } : screen
          ),
        })),

      removeScreen: (id) =>
        set((state) => {
          // Find all children and grandchildren recursively
          const getAllDescendants = (parentId: string): string[] => {
            const children = state.screens.filter(s => s.parentId === parentId);
            const childIds = children.map(c => c.id);
            const grandchildIds = children.flatMap(c => getAllDescendants(c.id));
            return [...childIds, ...grandchildIds];
          };

          const descendantIds = getAllDescendants(id);
          const idsToRemove = [id, ...descendantIds];

          return {
            screens: state.screens.filter((screen) => !idsToRemove.includes(screen.id)),
          };
        }),

      setContingency: (value) => set({ contingency: value }),

      addRiskFactor: (risk) =>
        set((state) => ({
          riskFactors: [...state.riskFactors, risk],
        })),

      removeRiskFactor: (index) =>
        set((state) => ({
          riskFactors: state.riskFactors.filter((_, i) => i !== index),
        })),

      calculateResults: () => {
        const state = get();
        const { screens, selectedMethods, contingency, project } = state;

        if (screens.length === 0) {
          set({ results: null });
          return;
        }

        // Calculate based on selected methods
        let totalHours = 0;

        screens.forEach((screen) => {
          // PERT Calculation
          if (selectedMethods.includes('PERT') && screen.pertData) {
            const { optimistic, mostLikely, pessimistic } = screen.pertData;
            const expected = (optimistic + 4 * mostLikely + pessimistic) / 6;
            totalHours += expected;
          }

          // Story Points (basic conversion)
          if (selectedMethods.includes('STORY_POINTS') && screen.storyPoints) {
            totalHours += screen.storyPoints * 8; // 8 hrs per point default
          }

          // T-Shirt Sizing
          if (selectedMethods.includes('TSHIRT') && screen.tshirtSize) {
            const sizeMap = { XS: 8, S: 20, M: 40, L: 80, XL: 120, XXL: 200 };
            totalHours += sizeMap[screen.tshirtSize] || 40;
          }

          // Function Point Analysis (FPA)
          if (selectedMethods.includes('FPA') && screen.fpaData) {
            const { externalInputs, externalOutputs, externalInquiries, internalLogicalFiles, externalInterfaceFiles } = screen.fpaData;
            // Using average complexity weights
            const functionPoints = 
              (externalInputs * 4) + 
              (externalOutputs * 5) + 
              (externalInquiries * 4) + 
              (internalLogicalFiles * 10) + 
              (externalInterfaceFiles * 7);
            // Convert FP to hours (industry standard: 6-10 hrs/FP, using 8)
            totalHours += functionPoints * 8;
          }

          // Use Case Points (UCP) - simplified
          if (selectedMethods.includes('UCP') && screen.ucpData) {
            // Simplified: reuse story points as use case complexity indicator
            // UCP = Unadjusted Use Case Points * Technical/Environmental factors
            // For simplicity, use story points * 10 hours per UCP
            const { simpleActors, averageActors, complexActors, simpleUseCases, averageUseCases, complexUseCases } = screen.ucpData;
            const ucp = 
              (simpleActors * 1) + 
              (averageActors * 2) + 
              (complexActors * 3) + 
              (simpleUseCases * 5) + 
              (averageUseCases * 10) + 
              (complexUseCases * 15);
            totalHours += ucp * 10;
          }
        });

        // Apply contingency
        const hoursWithContingency = totalHours * (1 + contingency / 100);

        // Calculate breakdown (typical ratios)
        const development = hoursWithContingency * 0.50;
        const testing = hoursWithContingency * 0.30;
        const hardening = hoursWithContingency * 0.20;

        // Calculate timeline (assuming 40 hrs/week per person)
        const timelineWeeks = Math.ceil(hoursWithContingency / (40 * project.teamSize));

        // Calculate cost
        const totalCost = hoursWithContingency * project.hourlyRate;

        // Confidence based on number of screens and method
        const confidence = Math.min(95, 60 + screens.length * 5);

        set({
          results: {
            totalHours: Math.round(hoursWithContingency),
            totalCost: Math.round(totalCost),
            timelineWeeks,
            confidence,
            breakdown: {
              development: Math.round(development),
              testing: Math.round(testing),
              hardening: Math.round(hardening),
            },
          },
        });
      },

      setCapacityPlan: (plan) => set({ capacityPlan: plan }),

      generateCapacityPlan: () => {
        const state = get();
        const { results, project, capacityPlan } = state;

        if (!results || !project.teamSize) {
          set({ capacityPlan: null });
          return;
        }

        const totalHours = results.totalHours;
        const teamSize = project.teamSize;
        const productivityFactor = capacityPlan ? capacityPlan.productivityFactor : 0.75;
        const includeBufferTime = capacityPlan ? capacityPlan.includeBufferTime : false;
        const bufferPercentage = capacityPlan ? capacityPlan.bufferPercentage : 20;
        const sprintDuration = capacityPlan ? capacityPlan.sprintDuration : 2;
        const teamMembers = capacityPlan ? capacityPlan.teamMembers : [];

        // Calculate total available hours per sprint
        const totalAvailableHoursPerSprint = teamSize * 40 * sprintDuration * productivityFactor;

        // Calculate total planned hours with buffer
        const totalPlannedHours = totalHours * (1 + bufferPercentage / 100);

        // Calculate number of sprints
        const numberOfSprints = Math.ceil(totalPlannedHours / totalAvailableHoursPerSprint);

        // Generate sprints
        const sprints: Sprint[] = [];
        let remainingHours = totalPlannedHours;
        let currentStartDate = new Date(project.startDate || Date.now());
        let currentEndDate = new Date(currentStartDate);
        currentEndDate.setDate(currentEndDate.getDate() + sprintDuration * 7);

        for (let i = 1; i <= numberOfSprints; i++) {
          const plannedHours = Math.min(remainingHours, totalAvailableHoursPerSprint);
          const capacityHours = totalAvailableHoursPerSprint;
          const utilizationPercent = (plannedHours / capacityHours) * 100;

          const sprint: Sprint = {
            id: `sprint-${i}`,
            number: i,
            startDate: currentStartDate,
            endDate: currentEndDate,
            plannedHours,
            capacityHours,
            utilizationPercent,
            phase: 'DEVELOPMENT',
            tasks: {
              development: plannedHours * 0.5,
              testing: plannedHours * 0.3,
              hardening: plannedHours * 0.2,
            },
          };

          sprints.push(sprint);

          remainingHours -= plannedHours;
          currentStartDate = new Date(currentEndDate);
          currentEndDate.setDate(currentEndDate.getDate() + sprintDuration * 7);
        }

        set({
          capacityPlan: {
            sprintDuration,
            teamMembers,
            productivityFactor,
            includeBufferTime,
            bufferPercentage,
            startDate: project.startDate || new Date(),
            sprints,
          },
        });
      },

      addTeamMember: (member) =>
        set((state) => {
          if (!state.capacityPlan) return state;
          return {
            capacityPlan: {
              ...state.capacityPlan,
              teamMembers: [...state.capacityPlan.teamMembers, member],
            },
          };
        }),

      updateTeamMember: (id, updates) =>
        set((state) => {
          if (!state.capacityPlan) return state;
          return {
            capacityPlan: {
              ...state.capacityPlan,
              teamMembers: state.capacityPlan.teamMembers.map((member) =>
                member.id === id ? { ...member, ...updates } : member
              ),
            },
          };
        }),

      removeTeamMember: (id) =>
        set((state) => {
          if (!state.capacityPlan) return state;
          return {
            capacityPlan: {
              ...state.capacityPlan,
              teamMembers: state.capacityPlan.teamMembers.filter((member) => member.id !== id),
            },
          };
        }),

      setComparisonMode: (enabled) => set({ comparisonMode: enabled }),

      calculateMethodComparison: () => {
        const state = get();
        const { screens, selectedMethods, contingency, project } = state;

        if (screens.length === 0) {
          set({ methodResults: [], comparisonAnalysis: null });
          return;
        }

        // Calculate based on selected methods
        const methodResults: MethodResults[] = selectedMethods.map(method => {
          let totalHours = 0;

          screens.forEach((screen) => {
            // PERT Calculation
            if (method === 'PERT' && screen.pertData) {
              const { optimistic, mostLikely, pessimistic } = screen.pertData;
              const expected = (optimistic + 4 * mostLikely + pessimistic) / 6;
              totalHours += expected;
            }

            // Story Points (basic conversion)
            if (method === 'STORY_POINTS' && screen.storyPoints) {
              totalHours += screen.storyPoints * 8; // 8 hrs per point default
            }

            // T-Shirt Sizing
            if (method === 'TSHIRT' && screen.tshirtSize) {
              const sizeMap = { XS: 8, S: 20, M: 40, L: 80, XL: 120, XXL: 200 };
              totalHours += sizeMap[screen.tshirtSize] || 40;
            }

            // Function Point Analysis (FPA)
            if (method === 'FPA' && screen.fpaData) {
              const { externalInputs, externalOutputs, externalInquiries, internalLogicalFiles, externalInterfaceFiles } = screen.fpaData;
              // Using average complexity weights
              const functionPoints = 
                (externalInputs * 4) + 
                (externalOutputs * 5) + 
                (externalInquiries * 4) + 
                (internalLogicalFiles * 10) + 
                (externalInterfaceFiles * 7);
              // Convert FP to hours (industry standard: 6-10 hrs/FP, using 8)
              totalHours += functionPoints * 8;
            }

            // Use Case Points (UCP) - simplified
            if (method === 'UCP' && screen.ucpData) {
              // Simplified: reuse story points as use case complexity indicator
              // UCP = Unadjusted Use Case Points * Technical/Environmental factors
              // For simplicity, use story points * 10 hours per UCP
              const { simpleActors, averageActors, complexActors, simpleUseCases, averageUseCases, complexUseCases } = screen.ucpData;
              const ucp = 
                (simpleActors * 1) + 
                (averageActors * 2) + 
                (complexActors * 3) + 
                (simpleUseCases * 5) + 
                (averageUseCases * 10) + 
                (complexUseCases * 15);
              totalHours += ucp * 10;
            }
          });

          // Apply contingency
          const hoursWithContingency = totalHours * (1 + contingency / 100);

          // Calculate breakdown (typical ratios)
          const development = hoursWithContingency * 0.50;
          const testing = hoursWithContingency * 0.30;
          const hardening = hoursWithContingency * 0.20;

          // Calculate timeline (assuming 40 hrs/week per person)
          const timelineWeeks = Math.ceil(hoursWithContingency / (40 * project.teamSize));

          // Calculate cost
          const totalCost = hoursWithContingency * project.hourlyRate;

          // Confidence based on number of screens and method
          const confidence = Math.min(95, 60 + screens.length * 5);

          return {
            method,
            results: {
              totalHours: Math.round(hoursWithContingency),
              totalCost: Math.round(totalCost),
              timelineWeeks,
              confidence,
              breakdown: {
                development: Math.round(development),
                testing: Math.round(testing),
                hardening: Math.round(hardening),
              },
            },
          };
        });

        // Calculate comparison analysis
        const totalHoursArray = methodResults.map(result => result.results.totalHours);
        
        // Filter out zero values to avoid division issues
        const validHours = totalHoursArray.filter(h => h > 0);
        
        if (validHours.length === 0) {
          set({ methodResults: [], comparisonAnalysis: null });
          return;
        }
        
        const averageHours = validHours.reduce((sum, hours) => sum + hours, 0) / validHours.length;
        const variance = Math.sqrt(validHours.reduce((sum, hours) => sum + Math.pow(hours - averageHours, 2), 0) / validHours.length);
        const minHours = Math.min(...totalHoursArray);
        const maxHours = Math.max(...totalHoursArray);
        
        // Find method with lowest NON-ZERO hours for recommendation
        const validResults = methodResults.filter(mr => mr.results.totalHours > 0);
        const recommendedMethod = validResults.length > 0 
          ? validResults.reduce((best, current) => current.results.totalHours < best.results.totalHours ? current : best).method
          : methodResults[0]?.method || null;
          
        const confidenceLevel = variance < 10 ? 'HIGH' : variance < 20 ? 'MEDIUM' : 'LOW';

        set({
          methodResults,
          comparisonAnalysis: {
            methods: methodResults,
            variance,
            averageHours,
            minHours,
            maxHours,
            recommendedMethod,
            confidenceLevel,
          },
        });
      },

      resetEstimation: () => set(initialState),

      saveDraft: () => {
        // Zustand persist already handles this automatically
        set({ lastSaved: new Date().toISOString() });
        console.log('Draft saved automatically!');
      },
    }),
    {
      name: 'estimation-wizard-storage', // localStorage key
      version: 1,
      // Add storage error handling with proper recovery
      storage: {
        getItem: (name) => {
          try {
            const str = localStorage.getItem(name);
            if (!str) return null;
            const parsed = JSON.parse(str);
            return parsed;
          } catch (error) {
            console.error('Failed to load from localStorage:', error);
            // Clear corrupted data
            localStorage.removeItem(name);
            return null;
          }
        },
        setItem: (name, value) => {
          try {
            // Try to save normally first
            localStorage.setItem(name, JSON.stringify(value));
          } catch (error) {
            console.error('localStorage save failed:', error);
            // If quota exceeded, clear ALL localStorage and save minimal state
            if (error instanceof Error && error.name === 'QuotaExceededError') {
              console.warn('⚠️ LocalStorage quota exceeded. Clearing all storage...');
              
              // Clear everything
              localStorage.clear();
              
              // Save only absolute essentials
              const minimalState = {
                state: {
                  currentStep: value.state?.currentStep || 1,
                  selectedMethods: value.state?.selectedMethods || [],
                  project: value.state?.project || initialState.project,
                  screens: (value.state?.screens || []).slice(0, 20), // Limit to 20 screens max
                  contingency: value.state?.contingency || 15,
                  riskFactors: [],
                  comparisonMode: false,
                  methodResults: [],
                  comparisonAnalysis: null,
                  capacityPlan: null,
                  results: null,
                  lastSaved: null,
                },
                version: 1,
              };
              
              try {
                localStorage.setItem(name, JSON.stringify(minimalState));
                console.log('✅ Minimal state saved successfully');
              } catch (e) {
                console.error('❌ Failed to save even minimal state. LocalStorage may be disabled.', e);
                // Continue without localStorage
              }
            }
          }
        },
        removeItem: (name) => localStorage.removeItem(name),
      },
      // Exclude large computed data from persistence - ONLY save user input
      partialize: (state) => ({
        currentStep: state.currentStep,
        currentProjectId: state.currentProjectId,
        selectedMethods: state.selectedMethods,
        project: state.project,
        screens: state.screens,
        contingency: state.contingency,
        riskFactors: state.riskFactors,
        comparisonMode: state.comparisonMode,
        // Explicitly exclude computed data
        // methodResults: [],
        // comparisonAnalysis: null,
        // capacityPlan: null,
        // results: null,
        lastSaved: state.lastSaved,
      }),
      // Handle hydration errors gracefully
      onRehydrateStorage: () => (state, error) => {
        if (error) {
          console.error('Failed to rehydrate storage:', error);
        } else if (state) {
          console.log('✅ Storage rehydrated successfully');
          // Recalculate comparison if needed
          if (state.comparisonMode && state.selectedMethods.length > 1) {
            state.calculateMethodComparison();
          }
        }
      },
    }
  )
);