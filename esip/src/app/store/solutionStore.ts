import { create } from 'zustand';
import { db } from '../lib/db';
import { useAuthStore } from './authStore';
import type { TradeoffAnalysis, TradeoffCriterion, SolutionOption } from '../lib/db/types';

/**
 * Solution-design store (Epic E / catalog Vol 04 SA-03/04).
 *
 * Holds the Solution Architect's trade-off / decision analyses: candidate options scored
 * against weighted criteria, ranked into a recommendation. Persistence goes through the
 * swappable DAL (`lib/db`) — Firebase today; one analysis is a single document with its
 * criteria + options inline, so no composite index is needed.
 */

export type { TradeoffAnalysis, TradeoffCriterion, SolutionOption };

/** Small client-side id for inline sub-objects (criteria/options). */
export function newId(prefix: string): string {
  const rand =
    typeof crypto !== 'undefined' && 'randomUUID' in crypto
      ? crypto.randomUUID().slice(0, 8)
      : Math.random().toString(36).slice(2, 10);
  return `${prefix}_${rand}`;
}

const DEFAULT_CRITERIA = (): TradeoffCriterion[] => [
  { id: newId('c'), name: 'Cost', weight: 4 },
  { id: newId('c'), name: 'Delivery time', weight: 3 },
  { id: newId('c'), name: 'Risk', weight: 4 },
  { id: newId('c'), name: 'Scalability', weight: 3 },
  { id: newId('c'), name: 'Security', weight: 4 },
];

const blankOption = (name: string): SolutionOption => ({
  id: newId('o'),
  name,
  description: '',
  scores: {},
  pros: [],
  cons: [],
});

export interface OptionRanking {
  optionId: string;
  name: string;
  /** weighted score 0..100 (% of the maximum achievable given the criteria weights) */
  percent: number;
  rank: number; // 1 = best
}

/**
 * Rank options by weighted score. Each option's raw total = Σ(weight × score); the max
 * achievable = Σ(weight × 5). Percent = total / max. Options with no scores rank last at 0%.
 */
export function rankOptions(analysis: Pick<TradeoffAnalysis, 'criteria' | 'options'>): OptionRanking[] {
  const maxPerOption = analysis.criteria.reduce((sum, c) => sum + c.weight * 5, 0);
  const scored = analysis.options.map((opt) => {
    const total = analysis.criteria.reduce((sum, c) => sum + c.weight * (opt.scores[c.id] || 0), 0);
    const percent = maxPerOption > 0 ? Math.round((total / maxPerOption) * 100) : 0;
    return { optionId: opt.id, name: opt.name, percent };
  });
  return scored
    .sort((a, b) => b.percent - a.percent)
    .map((s, i) => ({ ...s, rank: i + 1 }));
}

/** The recommended option: the manual pin if set, else the top-ranked option. */
export function recommendedOptionId(analysis: TradeoffAnalysis): string | undefined {
  if (analysis.recommendedOptionId) return analysis.recommendedOptionId;
  const ranking = rankOptions(analysis);
  return ranking[0]?.percent > 0 ? ranking[0].optionId : undefined;
}

interface SolutionState {
  analyses: TradeoffAnalysis[];
  isLoading: boolean;

  fetchByProject: (projectId: string) => Promise<void>;
  createAnalysis: (projectId: string, title: string) => Promise<TradeoffAnalysis | null>;
  updateAnalysis: (id: string, updates: Partial<TradeoffAnalysis>) => Promise<void>;
  deleteAnalysis: (id: string) => Promise<void>;
}

export const useSolutionStore = create<SolutionState>((set) => ({
  analyses: [],
  isLoading: false,

  fetchByProject: async (projectId: string) => {
    set({ isLoading: true });
    try {
      const analyses = await db.tradeoffAnalyses.getByProject(projectId);
      set({ analyses, isLoading: false });
    } catch (error) {
      console.error('Failed to load trade-off analyses:', error);
      set({ isLoading: false });
    }
  },

  createAnalysis: async (projectId: string, title: string) => {
    const user = useAuthStore.getState().user;
    try {
      const created = await db.tradeoffAnalyses.create({
        projectId,
        title: title.trim() || 'Untitled analysis',
        description: '',
        criteria: DEFAULT_CRITERIA(),
        options: [blankOption('Option A'), blankOption('Option B')],
        status: 'draft',
        createdBy: user?.id || 'system',
      });
      set((s) => ({ analyses: [created, ...s.analyses] }));
      return created;
    } catch (error) {
      console.error('Failed to create analysis:', error);
      throw error;
    }
  },

  updateAnalysis: async (id: string, updates: Partial<TradeoffAnalysis>) => {
    try {
      const updated = await db.tradeoffAnalyses.update(id, updates);
      set((s) => ({ analyses: s.analyses.map((a) => (a.id === id ? { ...a, ...updated } : a)) }));
    } catch (error) {
      console.error('Failed to update analysis:', error);
      throw error;
    }
  },

  deleteAnalysis: async (id: string) => {
    try {
      await db.tradeoffAnalyses.delete(id);
      set((s) => ({ analyses: s.analyses.filter((a) => a.id !== id) }));
    } catch (error) {
      console.error('Failed to delete analysis:', error);
      throw error;
    }
  },
}));
