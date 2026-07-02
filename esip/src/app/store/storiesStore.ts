import { create } from 'zustand';
import { db } from '../lib/db';
import type { Story, StoryType, StoryStatus } from '../lib/db/types';

export type { Story, StoryType, StoryStatus };

interface StoriesState {
  stories: Story[];
  isLoading: boolean;
  fetchByRequirement: (requirementId: string) => Promise<void>;
  bulkCreate: (stories: Array<Omit<Story, 'id' | 'createdAt' | 'updatedAt'>>) => Promise<Story[]>;
  updateStory: (id: string, updates: Partial<Story>) => Promise<void>;
  deleteStory: (id: string) => Promise<void>;
  clearStories: () => void;
}

export const useStoriesStore = create<StoriesState>((set, get) => ({
  stories: [],
  isLoading: false,

  fetchByRequirement: async (requirementId: string) => {
    set({ isLoading: true });
    try {
      const stories = await db.stories.getByRequirement(requirementId);
      set({ stories, isLoading: false });
    } catch {
      set({ isLoading: false });
    }
  },

  bulkCreate: async (stories) => {
    const created = await db.stories.bulkCreate(stories);
    set(s => ({ stories: [...s.stories, ...created] }));
    return created;
  },

  updateStory: async (id, updates) => {
    const updated = await db.stories.update(id, updates);
    set(s => ({ stories: s.stories.map(x => x.id === id ? updated : x) }));
  },

  deleteStory: async (id) => {
    await db.stories.delete(id);
    set(s => ({ stories: s.stories.filter(x => x.id !== id) }));
  },

  clearStories: () => set({ stories: [] }),
}));
