import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type WorkingMode = 'design' | 'estimation' | 'requirements' | 'review';

/**
 * Persona model (Epic B + C). Personas are CONFIG (data), not hard-coded in components —
 * the active persona reshapes the workspace: foregrounded nav (`enabledNav`), default
 * working mode, and which pillar facets surface (`knowledgeHubTabs`, `roadmapTypes`).
 * MVP holds the catalog here; a later increment moves it into the metamodel/DAL so it is
 * admin-editable + per-tenant (catalog Vol 05 + Vol 11).
 */
export interface PersonaDef {
  id: string;
  code: string; // P1..P14 (master-index persona table)
  label: string;
  description: string;
  enabledNav: string[]; // work-surface nav items this persona foregrounds (AppShell)
  defaultWorkingMode: WorkingMode; // seeds Epic F working-mode engine
  knowledgeHubTabs: string[]; // AI-Insight tabs this persona foregrounds (Epic C / Vol 08)
  roadmapTypes: string[]; // roadmap types this persona uses; default-wired in the Roadmap pillar (Epic C / C3 / Vol 09)
  requirementTypes: string[]; // requirement-type chips foregrounded in the Requirements board (Epic C / C1). Empty = all.
}

// AI-Insight tab keys available in the Knowledge Hub (see KnowledgeHubPage AI Insights row).
export const KH_INSIGHT_TABS = ['gaps', 'confusions', 'suggestions', 'security', 'threats', 'nfrs', 'cloud'] as const;

export const PERSONAS: PersonaDef[] = [
  { id: 'enterprise-architect', code: 'P1', label: 'Enterprise Architect', description: 'Portfolio, capabilities, and multi-year transformation across the enterprise.', enabledNav: ['projects', 'architecture', 'solution'], defaultWorkingMode: 'design', knowledgeHubTabs: ['gaps', 'suggestions', 'security', 'nfrs', 'cloud'], roadmapTypes: ['product', 'technical', 'architectural'], requirementTypes: ['integration', 'api_development', 'data_migration', 'security', 'performance'] },
  { id: 'solution-architect', code: 'P2', label: 'Solution Architect', description: 'Solution design, options & trade-offs, HLD/LLD, NFRs, sizing.', enabledNav: ['projects', 'architecture', 'estimation', 'solution'], defaultWorkingMode: 'design', knowledgeHubTabs: ['gaps', 'confusions', 'suggestions', 'security', 'nfrs'], roadmapTypes: ['technical', 'architectural'], requirementTypes: ['feature_change', 'api_development', 'integration', 'performance', 'security', 'data_migration'] },
  { id: 'application-architect', code: 'P3', label: 'Application Architect', description: 'Application structure, components, APIs, data contracts, estimates.', enabledNav: ['projects', 'architecture', 'estimation', 'solution'], defaultWorkingMode: 'design', knowledgeHubTabs: ['gaps', 'confusions', 'suggestions', 'security', 'nfrs'], roadmapTypes: ['technical', 'architectural'], requirementTypes: ['new_screen', 'update_screen', 'feature_change', 'bug_fix', 'api_development', 'ui_ux_change'] },
  { id: 'integration-architect', code: 'P4', label: 'Integration Architect', description: 'Integration patterns, message/event flows, canonical models.', enabledNav: ['projects', 'architecture', 'solution'], defaultWorkingMode: 'design', knowledgeHubTabs: ['gaps', 'confusions', 'suggestions', 'security', 'nfrs'], roadmapTypes: ['technical', 'architectural'], requirementTypes: ['integration', 'api_development', 'data_migration'] },
  { id: 'api-architect', code: 'P5', label: 'API Architect', description: 'API strategy, contracts, governance, gateway/policy design.', enabledNav: ['projects', 'architecture', 'solution'], defaultWorkingMode: 'design', knowledgeHubTabs: ['gaps', 'security', 'nfrs', 'suggestions'], roadmapTypes: ['technical'], requirementTypes: ['api_development', 'integration', 'security'] },
  { id: 'data-architect', code: 'P7', label: 'Data Architect', description: 'Conceptual/logical/physical data models, lineage, governance.', enabledNav: ['projects', 'architecture', 'solution'], defaultWorkingMode: 'design', knowledgeHubTabs: ['gaps', 'nfrs', 'security', 'suggestions'], roadmapTypes: ['technical', 'architectural'], requirementTypes: ['data_migration', 'integration', 'performance'] },
  { id: 'security-architect', code: 'P8', label: 'Security Architect', description: 'Threat models, control catalogs, zero-trust, security architecture.', enabledNav: ['projects', 'architecture', 'solution'], defaultWorkingMode: 'design', knowledgeHubTabs: ['security', 'threats', 'gaps', 'nfrs'], roadmapTypes: ['technical', 'architectural'], requirementTypes: ['security', 'integration', 'api_development'] },
  { id: 'cloud-architect', code: 'P9', label: 'Cloud Architect', description: 'Landing zones, cloud topologies, cost/governance, resilience.', enabledNav: ['projects', 'architecture', 'solution'], defaultWorkingMode: 'design', knowledgeHubTabs: ['cloud', 'nfrs', 'security', 'gaps'], roadmapTypes: ['technical', 'architectural'], requirementTypes: ['performance', 'integration', 'data_migration', 'security'] },
  { id: 'product-manager', code: 'P13', label: 'Product Manager', description: 'Product roadmap and BRDs; rolls roadmap items into requirements.', enabledNav: ['projects'], defaultWorkingMode: 'requirements', knowledgeHubTabs: ['gaps', 'confusions', 'suggestions'], roadmapTypes: ['product'], requirementTypes: ['new_screen', 'feature_change', 'ui_ux_change', 'update_screen'] },
  { id: 'presales-architect', code: 'P14', label: 'Presales / Bid Architect', description: 'RFP/RFI responses, proposals, tenders, and win themes.', enabledNav: ['projects', 'solution'], defaultWorkingMode: 'requirements', knowledgeHubTabs: ['gaps', 'suggestions'], roadmapTypes: ['product'], requirementTypes: ['new_screen', 'feature_change', 'integration', 'api_development'] },
];

export function getPersona(id: string): PersonaDef {
  return PERSONAS.find((p) => p.id === id) ?? PERSONAS[2]; // default: Application Architect
}

interface PersonaState {
  activePersonaId: string;
  setActivePersona: (id: string) => void;
}

export const usePersonaStore = create<PersonaState>()(
  persist(
    (set) => ({
      activePersonaId: 'application-architect', // current ESIP baseline persona
      setActivePersona: (id: string) => set({ activePersonaId: id }),
    }),
    { name: 'esip-active-persona' },
  ),
);
