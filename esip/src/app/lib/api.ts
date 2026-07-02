const API_URL = 'http://localhost:3001/api';

export class ApiClient {
  private static getToken(): string | null {
    const auth = localStorage.getItem('auth-storage');
    if (!auth) return null;
    const parsed = JSON.parse(auth);
    return parsed.state?.accessToken || null;
  }

  private static async request(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<any> {
    const token = this.getToken();
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(`${API_URL}${endpoint}`, {
      ...options,
      headers,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'Request failed' }));
      throw new Error(error.error || `HTTP ${response.status}`);
    }

    return response.json();
  }

  static async getProjects() {
    return this.request('/projects');
  }

  static async getProject(id: string) {
    return this.request(`/projects/${id}`);
  }

  static async createProject(data: any) {
    return this.request('/projects', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  static async updateProject(id: string, data: any) {
    return this.request(`/projects/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  static async shareProject(id: string, userId: string, permission: string) {
    return this.request(`/projects/${id}/share`, {
      method: 'POST',
      body: JSON.stringify({ userId, permission }),
    });
  }

  static async getWbsTasks(projectId: string) {
    return this.request(`/wbs/project/${projectId}`);
  }

  static async createWbsTask(data: any) {
    return this.request('/wbs', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  static async updateWbsTask(id: string, data: any) {
    return this.request(`/wbs/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  static async bulkCreateWbsTasks(projectId: string, tasks: any[]) {
    return this.request('/wbs/bulk', {
      method: 'POST',
      body: JSON.stringify({ projectId, tasks }),
    });
  }

  // ===== PI APIs =====
  static async getPIs(projectId: string) {
    return this.request(`/requirements/pis/project/${projectId}`);
  }

  static async createPI(data: any) {
    return this.request('/requirements/pis', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  static async updatePI(id: string, data: any) {
    return this.request(`/requirements/pis/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  static async deletePI(id: string) {
    return this.request(`/requirements/pis/${id}`, { method: 'DELETE' });
  }

  // ===== Sprint APIs =====
  static async getSprints(projectId: string) {
    return this.request(`/requirements/sprints/project/${projectId}`);
  }

  static async createSprint(data: any) {
    return this.request('/requirements/sprints', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  static async updateSprint(id: string, data: any) {
    return this.request(`/requirements/sprints/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  static async deleteSprint(id: string) {
    return this.request(`/requirements/sprints/${id}`, { method: 'DELETE' });
  }

  // ===== Requirements APIs =====
  static async getRequirements(projectId: string, filters?: { piId?: string; sprintId?: string; category?: string; status?: string }) {
    const params = new URLSearchParams(filters as any);
    return this.request(`/requirements/project/${projectId}?${params}`);
  }

  static async createRequirement(data: any) {
    return this.request('/requirements', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  static async updateRequirement(id: string, data: any) {
    return this.request(`/requirements/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  static async deleteRequirement(id: string) {
    return this.request(`/requirements/${id}`, { method: 'DELETE' });
  }

  static async bulkCreateRequirements(projectId: string, requirements: any[]) {
    return this.request('/requirements/bulk', {
      method: 'POST',
      body: JSON.stringify({ projectId, requirements }),
    });
  }

  // ===== Teams APIs =====

  static async getTeams() {
    return this.request('/teams');
  }

  static async createTeam(name: string) {
    return this.request('/teams', {
      method: 'POST',
      body: JSON.stringify({ name }),
    });
  }

  static async inviteToTeam(teamId: string, email: string, role: string) {
    return this.request(`/teams/${teamId}/invite`, {
      method: 'POST',
      body: JSON.stringify({ email, role }),
    });
  }

  static async getArchitectureArtifacts(projectId: string) {
    return this.request(`/architecture/${projectId}`);
  }

  static async createArchitectureArtifact(projectId: string, type: string, content: any) {
    return this.request('/architecture', {
      method: 'POST',
      body: JSON.stringify({ projectId, type, content }),
    });
  }
}