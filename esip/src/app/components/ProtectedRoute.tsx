import { Navigate } from 'react-router';
import { useAuthStore } from '../store/authStore';
import { useEffect } from 'react';

interface ProtectedRouteProps {
  children: React.ReactNode;
  allowedRoles?: string[];
}

export function ProtectedRoute({ children, allowedRoles }: ProtectedRouteProps) {
  const { isAuthenticated, user } = useAuthStore();

  // Debug logging
  useEffect(() => {
    console.log('ProtectedRoute check:', { isAuthenticated, user, allowedRoles });
  }, [isAuthenticated, user, allowedRoles]);

  if (!isAuthenticated) {
    console.log('Not authenticated, redirecting to login');
    return <Navigate to="/login" replace />;
  }

  if (allowedRoles && user && !allowedRoles.includes(user.role)) {
    console.log('User role not allowed:', user.role, 'Required:', allowedRoles);
    return <Navigate to="/unauthorized" replace />;
  }

  return <>{children}</>;
}