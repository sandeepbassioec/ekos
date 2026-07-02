import { Outlet } from 'react-router';
import { ProtectedRoute } from '../ProtectedRoute';
import { AppShell } from './AppShell';

/**
 * Route element that gates authenticated routes (auth-only) and wraps them in the
 * shared AppShell. Role-gated child routes add their own inner
 * <ProtectedRoute allowedRoles=...> around their page element.
 */
export function AppLayout() {
  return (
    <ProtectedRoute>
      <AppShell>
        <Outlet />
      </AppShell>
    </ProtectedRoute>
  );
}
