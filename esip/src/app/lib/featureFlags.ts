/**
 * Feature flags (Epic H — ports & foundations).
 *
 * A central capability registry so every MVP capability can be toggled per build/tenant. This
 * is the back-compat seam: flags default ON (current behavior); turning one OFF reverts the app
 * to its pre-MVP behavior for that capability (nav, routes, and entry points all respect it).
 * MVP holds the registry + per-user override here; a later increment moves overrides to the
 * tenant config zone (catalog Vol 13) behind the same hooks so callers don't change.
 */

export type FeatureFlagKey =
  | 'adaptiveHome'
  | 'workingMode'
  | 'inbox'
  | 'aiReviewGate'
  | 'solutionDesign'
  | 'adminUsers'
  | 'sprintBoard';

export interface FeatureFlagDef {
  key: FeatureFlagKey;
  label: string;
  description: string;
  default: boolean;
  epic: string;
}

export const FEATURE_FLAGS: FeatureFlagDef[] = [
  { key: 'adaptiveHome', label: 'Adaptive Home', description: 'Persona/working-mode aware home landing. Off → land on Projects.', default: true, epic: 'F' },
  { key: 'workingMode', label: 'Working Modes', description: 'Per-persona working-mode switcher in the header & home.', default: true, epic: 'F' },
  { key: 'inbox', label: 'Cross-role Inbox', description: 'Workflow requests, approvals & the header inbox bell.', default: true, epic: 'D' },
  { key: 'aiReviewGate', label: 'AI Review Gate', description: 'Route AI-extracted requirements through a manager review request in the Inbox before they are treated as confirmed. Off → AI output is applied with no review (prior behavior).', default: true, epic: 'D' },
  { key: 'solutionDesign', label: 'Solution Design', description: 'Trade-off / decision analysis pillar for architects.', default: true, epic: 'E' },
  { key: 'adminUsers', label: 'User Administration', description: 'Admin user management + invite/activation.', default: true, epic: 'G' },
  { key: 'sprintBoard', label: 'Sprint Kanban Board', description: 'Per-sprint Kanban execution board. Off (default) — teams use Jira/ADO. On — lightweight built-in board for teams without external tooling.', default: false, epic: 'SAFe' },
];

export const FLAG_DEFAULTS: Record<FeatureFlagKey, boolean> = FEATURE_FLAGS.reduce(
  (acc, f) => ((acc[f.key] = f.default), acc),
  {} as Record<FeatureFlagKey, boolean>,
);
