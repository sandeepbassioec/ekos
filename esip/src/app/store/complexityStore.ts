import { create } from 'zustand';
import { persist } from 'zustand/middleware';

// Complexity Factor Types
export type ComplexityCategory = 'TECHNICAL' | 'DOMAIN' | 'TEAM';

export interface ComplexityFactor {
  id: string;
  category: ComplexityCategory;
  name: string;
  description: string;
  weight: number; // 0.5 to 2.0 (1.0 = no impact)
  enabled: boolean;
}

// Default Complexity Factors
export const DEFAULT_COMPLEXITY_FACTORS: ComplexityFactor[] = [
  // TECHNICAL COMPLEXITY
  {
    id: 'tech-integration',
    category: 'TECHNICAL',
    name: 'Third-Party Integrations',
    description: 'Number and complexity of external API integrations',
    weight: 1.0,
    enabled: true,
  },
  {
    id: 'tech-architecture',
    category: 'TECHNICAL',
    name: 'Architectural Complexity',
    description: 'Microservices, distributed systems, event-driven architecture',
    weight: 1.0,
    enabled: true,
  },
  {
    id: 'tech-performance',
    category: 'TECHNICAL',
    name: 'Performance Requirements',
    description: 'High concurrency, real-time processing, optimization needs',
    weight: 1.0,
    enabled: true,
  },
  {
    id: 'tech-security',
    category: 'TECHNICAL',
    name: 'Security Requirements',
    description: 'Authentication, authorization, encryption, compliance',
    weight: 1.0,
    enabled: true,
  },
  {
    id: 'tech-data',
    category: 'TECHNICAL',
    name: 'Data Complexity',
    description: 'Complex data models, migrations, transformations',
    weight: 1.0,
    enabled: true,
  },
  {
    id: 'tech-ui',
    category: 'TECHNICAL',
    name: 'UI/UX Complexity',
    description: 'Rich interactions, animations, responsive design',
    weight: 1.0,
    enabled: true,
  },
  {
    id: 'tech-testing',
    category: 'TECHNICAL',
    name: 'Testing Requirements',
    description: 'Unit, integration, E2E, performance, security testing',
    weight: 1.0,
    enabled: true,
  },
  {
    id: 'tech-legacy',
    category: 'TECHNICAL',
    name: 'Legacy System Integration',
    description: 'Working with outdated systems, tech debt',
    weight: 1.0,
    enabled: true,
  },

  // DOMAIN COMPLEXITY
  {
    id: 'domain-knowledge',
    category: 'DOMAIN',
    name: 'Domain Knowledge',
    description: 'Team familiarity with business domain',
    weight: 1.0,
    enabled: true,
  },
  {
    id: 'domain-rules',
    category: 'DOMAIN',
    name: 'Business Rules Complexity',
    description: 'Complex workflows, approval chains, business logic',
    weight: 1.0,
    enabled: true,
  },
  {
    id: 'domain-compliance',
    category: 'DOMAIN',
    name: 'Regulatory Compliance',
    description: 'GDPR, HIPAA, SOC2, industry-specific regulations',
    weight: 1.0,
    enabled: true,
  },
  {
    id: 'domain-stakeholders',
    category: 'DOMAIN',
    name: 'Stakeholder Complexity',
    description: 'Number of stakeholders, conflicting requirements',
    weight: 1.0,
    enabled: true,
  },
  {
    id: 'domain-scope',
    category: 'DOMAIN',
    name: 'Scope Stability',
    description: 'How often requirements change',
    weight: 1.0,
    enabled: true,
  },
  {
    id: 'domain-documentation',
    category: 'DOMAIN',
    name: 'Documentation Quality',
    description: 'Completeness and clarity of requirements',
    weight: 1.0,
    enabled: true,
  },

  // TEAM COMPLEXITY
  {
    id: 'team-experience',
    category: 'TEAM',
    name: 'Team Experience',
    description: 'Team familiarity with technology stack',
    weight: 1.0,
    enabled: true,
  },
  {
    id: 'team-size',
    category: 'TEAM',
    name: 'Team Size & Distribution',
    description: 'Remote teams, timezone differences, communication overhead',
    weight: 1.0,
    enabled: true,
  },
  {
    id: 'team-collaboration',
    category: 'TEAM',
    name: 'Team Collaboration',
    description: 'How well the team works together',
    weight: 1.0,
    enabled: true,
  },
  {
    id: 'team-availability',
    category: 'TEAM',
    name: 'Team Availability',
    description: 'Part-time members, shared resources, interruptions',
    weight: 1.0,
    enabled: true,
  },
  {
    id: 'team-turnover',
    category: 'TEAM',
    name: 'Team Stability',
    description: 'Risk of team members leaving, onboarding overhead',
    weight: 1.0,
    enabled: true,
  },
];

// Complexity State
interface ComplexityState {
  factors: ComplexityFactor[];
  overallComplexityScore: number;
  confidenceLevel: number; // 0-100
  
  // Actions
  updateFactor: (id: string, updates: Partial<ComplexityFactor>) => void;
  resetFactors: () => void;
  calculateComplexityScore: () => void;
  calculateConfidence: () => void;
  toggleFactor: (id: string) => void;
}

// Create Store
export const useComplexityStore = create<ComplexityState>()(
  persist(
    (set, get) => ({
      factors: DEFAULT_COMPLEXITY_FACTORS,
      overallComplexityScore: 1.0,
      confidenceLevel: 85,

      updateFactor: (id, updates) =>
        set((state) => ({
          factors: state.factors.map((factor) =>
            factor.id === id ? { ...factor, ...updates } : factor
          ),
        })),

      resetFactors: () =>
        set({
          factors: DEFAULT_COMPLEXITY_FACTORS,
          overallComplexityScore: 1.0,
          confidenceLevel: 85,
        }),

      calculateComplexityScore: () => {
        const { factors } = get();
        
        // Average weight of all enabled factors
        const enabledFactors = factors.filter(f => f.enabled);
        if (enabledFactors.length === 0) {
          set({ overallComplexityScore: 1.0 });
          return;
        }
        
        const totalWeight = enabledFactors.reduce((sum, f) => sum + f.weight, 0);
        const avgWeight = totalWeight / enabledFactors.length;
        
        set({ overallComplexityScore: avgWeight });
        
        // Recalculate confidence after complexity changes
        get().calculateConfidence();
      },

      calculateConfidence: () => {
        const { factors, overallComplexityScore } = get();
        
        // Base confidence: 85%
        let confidence = 85;
        
        // Reduce confidence based on complexity
        // Higher complexity = lower confidence
        if (overallComplexityScore > 1.0) {
          const complexityPenalty = (overallComplexityScore - 1.0) * 20;
          confidence -= complexityPenalty;
        } else {
          // Lower complexity = higher confidence
          const complexityBonus = (1.0 - overallComplexityScore) * 10;
          confidence += complexityBonus;
        }
        
        // Check for high-risk factors
        const highRiskFactors = factors.filter(f => f.enabled && f.weight > 1.5);
        if (highRiskFactors.length > 0) {
          confidence -= highRiskFactors.length * 3;
        }
        
        // Clamp between 50% and 95%
        confidence = Math.max(50, Math.min(95, confidence));
        
        set({ confidenceLevel: Math.round(confidence) });
      },

      toggleFactor: (id) => {
        const { factors } = get();
        const factor = factors.find(f => f.id === id);
        if (factor) {
          get().updateFactor(id, { enabled: !factor.enabled });
          get().calculateComplexityScore();
        }
      },
    }),
    {
      name: 'complexity-storage',
    }
  )
);
