import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { auth } from '../lib/db';
import type { AuthUser } from '../lib/db';

export type User = AuthUser;

interface AuthState {
  user: User | null;
  accessToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;

  login: (email: string, password: string) => Promise<void>;
  register: (
    email: string,
    password: string,
    fullName: string,
    role?: string,
    extras?: { licenseTier?: string; defaultPersonaId?: string; status?: string; invitedBy?: string },
  ) => Promise<void>;
  logout: () => Promise<void>;
  checkAuth: () => Promise<void>;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      accessToken: null,
      isAuthenticated: false,
      isLoading: false,

      login: async (email: string, password: string) => {
        set({ isLoading: true });
        try {
          const { access_token, user } = await auth.signIn(email, password);
          set({
            user,
            accessToken: access_token,
            isAuthenticated: true,
            isLoading: false,
          });
        } catch (error: any) {
          set({ isLoading: false });
          throw new Error(error.message || 'Invalid email or password');
        }
      },

      register: async (email: string, password: string, fullName: string, role?: string, extras?) => {
        set({ isLoading: true });
        try {
          const { access_token, user } = await auth.signUp(email, password, fullName, role || 'manager', extras);
          set({
            user,
            accessToken: access_token,
            isAuthenticated: true,
            isLoading: false,
          });
        } catch (error: any) {
          set({ isLoading: false });
          throw error;
        }
      },

      logout: async () => {
        try {
          await auth.signOut();
          set({ user: null, accessToken: null, isAuthenticated: false });
        } catch (error) {
          console.error('Logout failed:', error);
        }
      },

      checkAuth: async () => {
        try {
          const session = await auth.getSession();
          if (session) {
            set({
              user: session.user,
              accessToken: session.access_token,
              isAuthenticated: true,
            });
          }
        } catch (error) {
          console.error('Auth check failed:', error);
          set({ user: null, accessToken: null, isAuthenticated: false });
        }
      },
    }),
    {
      name: 'auth-storage',
    }
  )
);