import { useAuthStore } from '../store/authStore';
import type { LicenseTier } from './db/types';

/**
 * License tiers & entitlements (Epic G / catalog Vol 13 + 17).
 *
 * A seat's LICENSE TIER conveys capability breadth, orthogonal to the org ROLE. Viewer and
 * Reviewer are free; Contributor and Enterprise are paid. `getEntitlements` collapses both
 * axes into simple booleans the UI can gate on. MVP wires the model + helper + admin
 * assignment; broad read-only enforcement across every surface is a later increment.
 */

export type { LicenseTier };

export const LICENSE_TIERS: { value: LicenseTier; label: string; blurb: string; free: boolean }[] = [
  { value: 'viewer', label: 'Viewer', blurb: 'Read-only access across the workspace.', free: true },
  { value: 'reviewer', label: 'Reviewer', blurb: 'Read plus participate in reviews & approvals.', free: true },
  { value: 'contributor', label: 'Contributor', blurb: 'Full authoring across entitled personas.', free: false },
  { value: 'enterprise', label: 'Enterprise', blurb: 'Authoring plus tenant administration.', free: false },
];

export const licenseLabel = (tier?: string) => LICENSE_TIERS.find((t) => t.value === tier)?.label ?? tier ?? '—';

export interface Entitlements {
  tier: LicenseTier;
  canView: boolean;
  canReview: boolean; // act on reviews/approvals (inbox)
  canEdit: boolean; // author/edit artifacts
  canAdmin: boolean; // tenant admin / user management
}

export function getEntitlements(role?: string, tier?: string): Entitlements {
  const t = (tier as LicenseTier) || 'contributor';
  const canEdit = t === 'contributor' || t === 'enterprise';
  const canReview = canEdit || t === 'reviewer';
  const canAdmin = t === 'enterprise' || role === 'admin';
  return { tier: t, canView: true, canReview, canEdit, canAdmin };
}

/** Entitlements for the currently signed-in user. */
export function useEntitlements(): Entitlements {
  const user = useAuthStore((s) => s.user);
  return getEntitlements(user?.role, user?.licenseTier);
}
