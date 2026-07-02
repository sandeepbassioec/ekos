import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { usePersonaStore, getPersona, type WorkingMode } from './personaStore';

/**
 * Working-mode personalization (Epic F / catalog Vol 14).
 *
 * A "working mode" is the lens the user is currently working through — Design, Requirements,
 * Estimation, or Review. It seeds from the active persona's `defaultWorkingMode`, but the user
 * can switch it, and the choice is REMEMBERED PER PERSONA (the suite adapts to how you like to
 * work in each role). Persisted to localStorage today; moves to the per-user profile in a later
 * increment (Vol 13/14) behind the same hook so callers don't change.
 */

export type { WorkingMode };

export const WORKING_MODES: { value: WorkingMode; label: string; blurb: string }[] = [
  { value: 'design', label: 'Design', blurb: 'Architect solutions, options, and diagrams.' },
  { value: 'requirements', label: 'Requirements', blurb: 'Capture, classify, and refine requirements.' },
  { value: 'estimation', label: 'Estimation', blurb: 'Size and plan delivery effort.' },
  { value: 'review', label: 'Review', blurb: 'Approvals, reviews, and cross-role requests.' },
];

interface WorkingModeState {
  modeByPersona: Record<string, WorkingMode>;
  setMode: (personaId: string, mode: WorkingMode) => void;
}

export const useWorkingModeStore = create<WorkingModeState>()(
  persist(
    (set) => ({
      modeByPersona: {},
      setMode: (personaId, mode) =>
        set((s) => ({ modeByPersona: { ...s.modeByPersona, [personaId]: mode } })),
    }),
    { name: 'esip-working-mode' },
  ),
);

/** Current working mode for the active persona — the user's saved choice, else the persona default. */
export function useWorkingMode(): WorkingMode {
  const activePersonaId = usePersonaStore((s) => s.activePersonaId);
  const saved = useWorkingModeStore((s) => s.modeByPersona[activePersonaId]);
  return saved ?? getPersona(activePersonaId).defaultWorkingMode;
}
