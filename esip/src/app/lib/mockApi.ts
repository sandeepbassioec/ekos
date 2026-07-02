// Mock API for authentication and data management
// This simulates the backend server functionality

interface User {
  id: string;
  email: string;
  fullName?: string;
  role: string;
  passwordHash: string;
  createdAt: string;
}

interface Project {
  id: string;
  name: string;
  description?: string;
  ownerId: string;
  status: string;
  createdAt: string;
}

// In-memory storage (simulating database)
const storage = {
  users: new Map<string, User>(),
  projects: new Map<string, Project>(),
  sessions: new Map<string, string>(), // token -> userId
};

// Initialize with a default user
const defaultUser: User = {
  id: 'default-user-1',
  email: 'admin@esip.com',
  fullName: 'Admin User',
  role: 'admin',
  passwordHash: btoa('admin123'), // Hash the password properly
  createdAt: new Date().toISOString(),
};
storage.users.set(defaultUser.email, defaultUser);

// Helper to hash password (simplified - in production use bcrypt)
const hashPassword = (password: string) => {
  return btoa(password); // Simple base64 encoding for mock
};

const verifyPassword = (password: string, hash: string) => {
  return btoa(password) === hash;
};

// Generate token
const generateToken = () => {
  return Math.random().toString(36).substring(2) + Date.now().toString(36);
};

export const mockApi = {
  auth: {
    register: async (email: string, password: string, fullName?: string, role = 'manager') => {
      // Check if user exists
      if (storage.users.has(email)) {
        throw new Error('User already exists');
      }

      // Create new user
      const user: User = {
        id: `user-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        email,
        fullName: fullName || email.split('@')[0],
        role,
        passwordHash: hashPassword(password),
        createdAt: new Date().toISOString(),
      };

      storage.users.set(email, user);

      // Generate tokens
      const accessToken = generateToken();
      const refreshToken = generateToken();
      storage.sessions.set(accessToken, user.id);

      // Return user without password
      const { passwordHash, ...userWithoutPassword } = user;
      return {
        user: userWithoutPassword,
        accessToken,
        refreshToken,
      };
    },

    login: async (email: string, password: string) => {
      // Find user
      const user = storage.users.get(email);
      if (!user) {
        throw new Error('Invalid credentials');
      }

      // Verify password
      if (!verifyPassword(password, user.passwordHash)) {
        throw new Error('Invalid credentials');
      }

      // Generate tokens
      const accessToken = generateToken();
      const refreshToken = generateToken();
      storage.sessions.set(accessToken, user.id);

      // Return user without password
      const { passwordHash, ...userWithoutPassword } = user;
      return {
        user: userWithoutPassword,
        accessToken,
        refreshToken,
      };
    },

    me: async (accessToken: string) => {
      const userId = storage.sessions.get(accessToken);
      if (!userId) {
        throw new Error('Unauthorized');
      }

      // Find user by ID
      const user = Array.from(storage.users.values()).find(u => u.id === userId);
      if (!user) {
        throw new Error('User not found');
      }

      const { passwordHash, ...userWithoutPassword } = user;
      return userWithoutPassword;
    },

    logout: async (accessToken: string) => {
      storage.sessions.delete(accessToken);
      return { success: true };
    },
  },

  projects: {
    list: async (accessToken: string) => {
      const userId = storage.sessions.get(accessToken);
      if (!userId) {
        throw new Error('Unauthorized');
      }

      return Array.from(storage.projects.values()).filter(p => p.ownerId === userId);
    },

    create: async (accessToken: string, data: { name: string; description?: string }) => {
      const userId = storage.sessions.get(accessToken);
      if (!userId) {
        throw new Error('Unauthorized');
      }

      const project: Project = {
        id: `project-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        name: data.name,
        description: data.description,
        ownerId: userId,
        status: 'active',
        createdAt: new Date().toISOString(),
      };

      storage.projects.set(project.id, project);
      return project;
    },

    get: async (accessToken: string, projectId: string) => {
      const userId = storage.sessions.get(accessToken);
      if (!userId) {
        throw new Error('Unauthorized');
      }

      const project = storage.projects.get(projectId);
      if (!project || project.ownerId !== userId) {
        throw new Error('Project not found');
      }

      return project;
    },
  },
};