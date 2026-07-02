import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { FLAG_DEFAULTS, type FeatureFlagKey } from '../lib/featureFlags';

/**
 * Runtime feature-flag overrides (Epic H). Defaults live in `lib/featureFlags`; this store holds
 * per-user toggles (persisted), admin-editable from the Capabilities page. A flag with no override
 * falls back to its registry default.
 */
interface FeatureFlagState {
  overrides: Partial<Record<FeatureFlagKey, boolean>>;
  setFlag: (key: FeatureFlagKey, value: boolean) => void;
  resetFlag: (key: FeatureFlagKey) => void;
  resetAll: () => void;
}

export const useFeatureFlagStore = create<FeatureFlagState>()(
  persist(
    (set) => ({
      overrides: {},
      setFlag: (key, value) => set((s) => ({ overrides: { ...s.overrides, [key]: value } })),
      resetFlag: (key) =>
        set((s) => {
          const next = { ...s.overrides };
          delete next[key];
          return { overrides: next };
        }),
      resetAll: () => set({ overrides: {} }),
    }),
    { name: 'esip-feature-flags' },
  ),
);

/** Reactive flag read for components. */
export function useFeatureFlag(key: FeatureFlagKey): boolean {
  const override = useFeatureFlagStore((s) => s.overrides[key]);
  return override ?? FLAG_DEFAULTS[key];
}

/** Non-reactive flag read for guards/handlers outside the render cycle. */
export function isFeatureEnabled(key: FeatureFlagKey): boolean {
  return useFeatureFlagStore.getState().overrides[key] ?? FLAG_DEFAULTS[key];
}
