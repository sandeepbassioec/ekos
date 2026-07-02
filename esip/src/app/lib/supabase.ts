import { createClient } from '@supabase/supabase-js';

const supabaseUrl = 'https://cbotkxhmiwgtepjspitc.supabase.co';
const supabaseAnonKey = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImNib3RreGhtaXdndGVwanNwaXRjIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NzUyOTQzMTEsImV4cCI6MjA5MDg3MDMxMX0.comiGtfDp-CsBPPSzXDqih8oqVwrJ9nEwqHxog3CD20';

export const supabase = createClient(supabaseUrl, supabaseAnonKey);

// Database helpers
export const db = {
  projects: {
    getAll: async (userId: string) => {
      const { data, error } = await supabase
        .from('Project')
        .select('*')
        .eq('userId', userId)
        .order('updatedAt', { ascending: false });
      if (error) throw error;
      return data;
    },
    getById: async (id: string, userId: string) => {
      const { data, error } = await supabase
        .from('Project')
        .select('*')
        .eq('id', id)
        .eq('userId', userId)
        .single();
      if (error) throw error;
      return data;
    },
    create: async (project: any) => {
      const { data, error } = await supabase
        .from('Project')
        .insert(project)
        .select()
        .single();
      if (error) throw error;
      return data;
    },
    update: async (id: string, userId: string, updates: any) => {
      const { data, error } = await supabase
        .from('Project')
        .update({ ...updates, updatedAt: new Date().toISOString() })
        .eq('id', id)
        .eq('userId', userId)
        .select()
        .single();
      if (error) throw error;
      return data;
    },
    delete: async (id: string, userId: string) => {
      const { error } = await supabase
        .from('Project')
        .delete()
        .eq('id', id)
        .eq('userId', userId);
      if (error) throw error;
    },
  },
  
  pis: {
    getByProject: async (projectId: string, userId: string) => {
      const { data, error } = await supabase
        .from('PI')
        .select('*')
        .eq('projectId', projectId)
        .eq('userId', userId)
        .order('piNumber', { ascending: true });
      if (error) throw error;
      return data;
    },
    create: async (pi: any) => {
      const { data, error } = await supabase
        .from('PI')
        .insert(pi)
        .select()
        .single();
      if (error) throw error;
      return data;
    },
    update: async (id: string, userId: string, updates: any) => {
      const { data, error } = await supabase
        .from('PI')
        .update({ ...updates, updatedAt: new Date().toISOString() })
        .eq('id', id)
        .eq('userId', userId)
        .select()
        .single();
      if (error) throw error;
      return data;
    },
    delete: async (id: string, userId: string) => {
      const { error } = await supabase
        .from('PI')
        .delete()
        .eq('id', id)
        .eq('userId', userId);
      if (error) throw error;
    },
  },
  
  sprints: {
    getByPI: async (piId: string, userId: string) => {
      const { data, error } = await supabase
        .from('Sprint')
        .select('*')
        .eq('piId', piId)
        .eq('userId', userId)
        .order('sprintNumber', { ascending: true });
      if (error) throw error;
      return data;
    },
    create: async (sprint: any) => {
      const { data, error } = await supabase
        .from('Sprint')
        .insert(sprint)
        .select()
        .single();
      if (error) throw error;
      return data;
    },
    update: async (id: string, userId: string, updates: any) => {
      const { data, error } = await supabase
        .from('Sprint')
        .update({ ...updates, updatedAt: new Date().toISOString() })
        .eq('id', id)
        .eq('userId', userId)
        .select()
        .single();
      if (error) throw error;
      return data;
    },
    delete: async (id: string, userId: string) => {
      const { error } = await supabase
        .from('Sprint')
        .delete()
        .eq('id', id)
        .eq('userId', userId);
      if (error) throw error;
    },
  },
  
  requirements: {
    getBySprint: async (sprintId: string, userId: string) => {
      const { data, error } = await supabase
        .from('Requirement')
        .select('*')
        .eq('sprintId', sprintId)
        .eq('userId', userId)
        .order('order', { ascending: true });
      if (error) throw error;
      return data;
    },
    create: async (requirement: any) => {
      const { data, error } = await supabase
        .from('Requirement')
        .insert(requirement)
        .select()
        .single();
      if (error) throw error;
      return data;
    },
    update: async (id: string, userId: string, updates: any) => {
      const { data, error } = await supabase
        .from('Requirement')
        .update({ ...updates, updatedAt: new Date().toISOString() })
        .eq('id', id)
        .eq('userId', userId)
        .select()
        .single();
      if (error) throw error;
      return data;
    },
    delete: async (id: string, userId: string) => {
      const { error } = await supabase
        .from('Requirement')
        .delete()
        .eq('id', id)
        .eq('userId', userId);
      if (error) throw error;
    },
  },
};

// Auth helpers
export const auth = {
  signUp: async (email: string, password: string, fullName: string) => {
    const { data, error } = await supabase.auth.signUp({
      email,
      password,
      options: {
        data: { full_name: fullName },
        emailRedirectTo: window.location.origin,
      },
    });
    if (error) {
      console.error('Supabase signUp error:', error);
      throw error;
    }
    return data;
  },

  signIn: async (email: string, password: string) => {
    const { data, error } = await supabase.auth.signInWithPassword({
      email,
      password,
    });
    if (error) {
      console.error('Supabase signIn error:', error);
      throw error;
    }
    return data;
  },

  signOut: async () => {
    const { error } = await supabase.auth.signOut();
    if (error) {
      console.error('Supabase signOut error:', error);
      throw error;
    }
  },

  getSession: async () => {
    const { data, error } = await supabase.auth.getSession();
    if (error) {
      console.error('Supabase getSession error:', error);
      throw error;
    }
    return data.session;
  },

  getUser: async () => {
    const { data, error } = await supabase.auth.getUser();
    if (error) {
      console.error('Supabase getUser error:', error);
      throw error;
    }
    return data.user;
  },
};
