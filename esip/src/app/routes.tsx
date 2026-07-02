import { createBrowserRouter, Navigate } from 'react-router';
import TeamManagement from './pages/TeamManagement';
import ProjectsListPage from './pages/ProjectsListPage';
import ProjectDetailPage from './pages/ProjectDetailPage';
import ComprehensiveRequirementsPage from './pages/ComprehensiveRequirementsPage';
import { ProtectedRoute } from './components/ProtectedRoute';
import { AppLayout } from './components/layout/AppLayout';
import Login from './pages/Login';
import Register from './pages/Register';
import Unauthorized from './pages/Unauthorized';
import ArchitecturePage from './pages/ArchitecturePage';
import ArchitectureGeneratePage from './pages/ArchitectureGeneratePage';
import DeveloperDashboard from './pages/DeveloperDashboard';
import ManagerApproval from './pages/ManagerApproval';
import { EstimationWizard } from './pages/EstimationWizard';
import { MethodSelection } from './pages/steps/MethodSelection';
import { ProjectSetup } from './pages/steps/ProjectSetup';
import { ScopeDefinition } from './pages/steps/ScopeDefinition';
import { MethodDetails } from './pages/steps/MethodDetails';
import { RiskAssessment } from './pages/steps/RiskAssessment';
import { ReviewCalculate } from './pages/steps/ReviewCalculate';
import { GenerateDeliverables } from './pages/steps/GenerateDeliverables';
import AdminMetaModelPage from './pages/AdminMetaModelPage';
import CreateArtifactTypePage from './pages/CreateArtifactTypePage';
import PersonaRegistryPage from './pages/PersonaRegistryPage';
import RequirementsCaptureBoard from './pages/RequirementsCaptureBoard';
import CreateRequirement from './pages/CreateRequirement';
import CreateProject from './pages/CreateProject';
import ProjectTeamPage from './pages/ProjectTeamPage';
import PIPlanningPage from './pages/PIPlanningPage';
import SprintBoardPage from './pages/SprintBoardPage';
import PIMetricsPage from './pages/PIMetricsPage';
import RoadmapPage from './pages/RoadmapPage';
import DocumentsPage from './pages/DocumentsPage';
import DesignerPage from './pages/DesignerPage';
import KnowledgeHubPage from './pages/KnowledgeHubPage';
import InboxPage from './pages/InboxPage';
import SolutionDesignPage from './pages/SolutionDesignPage';
import WorkspaceHomePage from './pages/WorkspaceHomePage';
import AdminUsersPage from './pages/AdminUsersPage';
import DemoSetupWizard from './pages/DemoSetupWizard';
import ActivatePage from './pages/ActivatePage';
import AdminFeatureFlagsPage from './pages/AdminFeatureFlagsPage';
import ErrorPage from './pages/ErrorPage';
import { type ReactNode } from 'react';
import { useFeatureFlag } from './store/featureFlagStore';
import type { FeatureFlagKey } from './lib/featureFlags';

/** Epic H — gate a route behind a feature flag; disabled capabilities fall back to the always-on Projects list. */
function FeatureGate({ flag, children }: { flag: FeatureFlagKey; children: ReactNode }) {
  return useFeatureFlag(flag) ? <>{children}</> : <Navigate to="/projects/list" replace />;
}

/** The `/` landing respects the Adaptive Home flag. */
function HomeLanding() {
  return <Navigate to={useFeatureFlag('adaptiveHome') ? '/home' : '/projects/list'} replace />;
}

export const router = createBrowserRouter([
  {
    // Root wrapper: catches any unhandled route-level error anywhere in the tree
    errorElement: <ErrorPage />,
    children: [
      // Public (no shell)
      { path: '/login', element: <Login /> },
      { path: '/register', element: <Register /> },
      { path: '/activate/:token', element: <ActivatePage /> },
      { path: '/unauthorized', element: <Unauthorized /> },

      // Authenticated — wrapped in the shared AppShell (AppLayout = ProtectedRoute + shell + <Outlet/>)
      {
        element: <AppLayout />,
        children: [
          { path: '/', element: <HomeLanding /> },

          // Adaptive workspace home (Epic F) — persona + working-mode aware landing (Epic H flag)
          { path: '/home', element: (<FeatureGate flag="adaptiveHome"><WorkspaceHomePage /></FeatureGate>) },

          // Projects + project-scoped pillars
          { path: '/projects/list', element: <ProjectsListPage /> },
          { path: '/projects/create', element: <CreateProject /> },
          { path: '/projects/:projectId', element: <ProjectDetailPage /> },
          { path: '/projects/:projectId/team', element: <ProjectTeamPage /> },
          { path: '/projects/:projectId/pi-planning', element: <PIPlanningPage /> },
          { path: '/projects/:projectId/sprints/:sprintId', element: <SprintBoardPage /> },
          { path: '/projects/:projectId/pi-planning/metrics', element: <PIMetricsPage /> },
          { path: '/projects/:projectId/pi-planning/metrics/:piId', element: <PIMetricsPage /> },
          { path: '/projects/:projectId/requirements', element: <RequirementsCaptureBoard /> },
          { path: '/projects/:projectId/requirements/add', element: <CreateRequirement /> },
          { path: '/projects/:projectId/requirements/:id/edit', element: <CreateRequirement /> },
          { path: '/projects/:projectId/requirements/comprehensive', element: <ComprehensiveRequirementsPage /> },
          { path: '/projects/:projectId/roadmap', element: <RoadmapPage /> },
          { path: '/demo-setup', element: <DemoSetupWizard /> },
          { path: '/projects/:projectId/documents', element: <DocumentsPage /> },
          { path: '/projects/:projectId/documents/new', element: <DesignerPage /> },
          { path: '/projects/:projectId/documents/:documentId', element: <DesignerPage /> },
          { path: '/projects/:projectId/solution', element: (<FeatureGate flag="solutionDesign"><SolutionDesignPage /></FeatureGate>) },
          { path: '/roadmap-items/:roadmapItemId/knowledge-hub', element: <KnowledgeHubPage /> },

          // Cross-role workflow inbox (Epic D) — available to every authenticated user (Epic H flag)
          { path: '/inbox', element: (<FeatureGate flag="inbox"><InboxPage /></FeatureGate>) },

          // Architecture
          { path: '/architecture', element: <ArchitecturePage /> },
          { path: '/architecture/generate', element: <ArchitectureGeneratePage /> },

          // Estimation wizard (nested steps render in its own Outlet, inside the shell)
          {
            path: '/wizard',
            element: <EstimationWizard />,
            children: [
              { index: true, element: <Navigate to="step/1" replace /> },
              { path: 'step/1', element: <MethodSelection /> },
              { path: 'step/2', element: <ProjectSetup /> },
              { path: 'step/3', element: <ScopeDefinition /> },
              { path: 'step/4', element: <MethodDetails /> },
              { path: 'step/5', element: <RiskAssessment /> },
              { path: 'step/6', element: <ReviewCalculate /> },
              { path: 'step/7', element: <GenerateDeliverables /> },
            ],
          },

          // Role-gated (inner ProtectedRoute enforces the role; outer AppLayout enforces auth)
          { path: '/teams', element: (<ProtectedRoute allowedRoles={['manager', 'admin']}><TeamManagement /></ProtectedRoute>) },
          { path: '/manager/approvals', element: (<ProtectedRoute allowedRoles={['manager', 'admin']}><ManagerApproval /></ProtectedRoute>) },
          { path: '/developer/tasks', element: (<ProtectedRoute allowedRoles={['developer']}><DeveloperDashboard /></ProtectedRoute>) },
          { path: '/admin/metamodel', element: (<ProtectedRoute allowedRoles={['admin']}><AdminMetaModelPage /></ProtectedRoute>) },
          { path: '/admin/metamodel/create', element: (<ProtectedRoute allowedRoles={['admin']}><CreateArtifactTypePage /></ProtectedRoute>) },
          { path: '/admin/personas', element: (<ProtectedRoute allowedRoles={['admin']}><PersonaRegistryPage /></ProtectedRoute>) },
          { path: '/admin/users', element: (<ProtectedRoute allowedRoles={['admin']}><FeatureGate flag="adminUsers"><AdminUsersPage /></FeatureGate></ProtectedRoute>) },
          { path: '/admin/flags', element: (<ProtectedRoute allowedRoles={['admin']}><AdminFeatureFlagsPage /></ProtectedRoute>) },
        ],
      },

      // 404 — any unregistered path shows the error page instead of redirecting to login
      { path: '*', element: <ErrorPage /> },
    ],
  },
]);
