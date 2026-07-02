import { create } from 'zustand';

export interface AppNotification {
  id: string;
  type: 'ai-analysis-complete' | 'info';
  title: string;
  body: string;
  link?: string;
  read: boolean;
  createdAt: number;
}

interface NotificationState {
  notifications: AppNotification[];
  addNotification: (n: Omit<AppNotification, 'id' | 'read' | 'createdAt'>) => void;
  markRead: (id: string) => void;
  markAllRead: () => void;
  clearAll: () => void;
}

let _counter = 0;

export const useNotificationStore = create<NotificationState>((set) => ({
  notifications: [],

  addNotification: (n) =>
    set((state) => ({
      notifications: [
        {
          ...n,
          id: `notif-${++_counter}`,
          read: false,
          createdAt: Date.now(),
        },
        ...state.notifications,
      ].slice(0, 50),
    })),

  markRead: (id) =>
    set((state) => ({
      notifications: state.notifications.map((n) =>
        n.id === id ? { ...n, read: true } : n,
      ),
    })),

  markAllRead: () =>
    set((state) => ({
      notifications: state.notifications.map((n) => ({ ...n, read: true })),
    })),

  clearAll: () => set({ notifications: [] }),
}));
