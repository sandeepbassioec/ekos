import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router';
import { Plus, Search, FolderOpen, Clock, CheckCircle, AlertCircle, TrendingUp, Users, LogOut, FileText, Layers, Shield, Database, Code, Trash2, Rocket } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../components/ui/card';
import { Input } from '../components/ui/input';
import { Badge } from '../components/ui/badge';
import { Progress } from '../components/ui/progress';
import { Project } from '../types/project';
import { useAuthStore } from '../store/authStore';
import { db } from '../lib/db';
import { toast } from 'sonner';

export default function ProjectsListPage() {
  const navigate = useNavigate();
  const { user, logout } = useAuthStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [projects, setProjects] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchProjects = async () => {
      if (!user?.id) {
        setIsLoading(false);
        return;
      }

      try {
        const data = await db.projects.getAll(user.id);
        setProjects(data || []);
      } catch (error) {
        toast.error('Failed to fetch projects');
        console.error(error);
      } finally {
        setIsLoading(false);
      }
    };

    fetchProjects();
  }, [user?.id]);

  const stats = [
    { label: 'Total Projects', value: projects.length, icon: FolderOpen, color: 'bg-blue-500' },
    { label: 'Draft', value: projects.filter(p => p.status === 'draft').length, icon: Clock, color: 'bg-yellow-500' },
    { label: 'Active', value: projects.filter(p => p.status === 'active').length, icon: TrendingUp, color: 'bg-green-500' },
    { label: 'Completed', value: projects.filter(p => p.status === 'completed').length, icon: CheckCircle, color: 'bg-purple-500' },
  ];

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'planning': return 'bg-blue-100 text-blue-700';
      case 'in_progress': return 'bg-yellow-100 text-yellow-700';
      case 'completed': return 'bg-green-100 text-green-700';
      case 'on_hold': return 'bg-orange-100 text-orange-700';
      case 'cancelled': return 'bg-red-100 text-red-700';
      default: return 'bg-gray-100 text-gray-700';
    }
  };

  const getPhaseColor = (phase: string) => {
    switch (phase) {
      case 'requirements': return 'bg-purple-100 text-purple-700';
      case 'design': return 'bg-blue-100 text-blue-700';
      case 'development': return 'bg-yellow-100 text-yellow-700';
      case 'testing': return 'bg-orange-100 text-orange-700';
      case 'deployment': return 'bg-green-100 text-green-700';
      default: return 'bg-gray-100 text-gray-700';
    }
  };

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  const handleCreateProject = () => {
    // Navigate to project creation form
    navigate('/projects/create');
  };

  const handleDeleteProject = async (projectId: string, projectName: string, e: React.MouseEvent) => {
    e.stopPropagation(); // Prevent card click

    const confirmed = window.confirm(
      `Are you sure you want to delete "${projectName}"?\n\n` +
      `This will permanently delete:\n` +
      `• The project\n` +
      `• All roadmaps\n` +
      `• All roadmap items\n` +
      `• All documents, meetings, emails, chats\n` +
      `• All AI-extracted requirements\n\n` +
      `This action cannot be undone.`
    );

    if (!confirmed) return;

    try {
      setIsLoading(true);
      await db.projects.delete(projectId, user!.id);
      toast.success('Project deleted successfully');

      // Refresh projects list
      const data = await db.projects.getAll(user!.id);
      setProjects(data || []);
    } catch (error) {
      console.error('Error deleting project:', error);
      toast.error('Failed to delete project');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-indigo-50 to-purple-50">
      {/* Platform Header */}
      <div className="bg-white border-b border-gray-200 shadow-sm sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 bg-gradient-to-br from-blue-600 to-indigo-600 rounded-lg flex items-center justify-center">
                <Code className="w-6 h-6 text-white" />
              </div>
              <div>
                <h1 className="text-2xl font-bold text-gray-900">
                  DevPlatform
                </h1>
                <p className="text-xs text-gray-600">Requirements • Estimation • Architecture • Documentation</p>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <div className="text-sm text-gray-600">
                {user?.fullName || user?.email} <span className="text-blue-600">({user?.role})</span>
              </div>
              <Button
                variant="outline"
                onClick={handleLogout}
                className="flex items-center gap-2"
              >
                <LogOut className="w-4 h-4" />
                Logout
              </Button>
            </div>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto py-8 px-6">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-3xl font-bold text-gray-900">Project Workspace</h2>
              <p className="text-gray-600 mt-1">
                Manage requirements, estimations, and architectural documentation for all your projects
              </p>
            </div>
            <div className="flex items-center gap-2">
              <Button variant="outline" onClick={() => navigate('/demo-setup')}>
                <Rocket className="w-4 h-4 mr-2 text-indigo-600" />
                Demo Setup
              </Button>
              <Button onClick={handleCreateProject} className="bg-blue-600 hover:bg-blue-700">
                <Plus className="w-4 h-4 mr-2" />
                Create New Project
              </Button>
            </div>
          </div>
        </div>

        {/* Platform Features Info */}
        <div className="grid grid-cols-1 md:grid-cols-5 gap-3 mb-8">
          <Card className="border-l-4 border-l-purple-500">
            <CardContent className="pt-4 pb-4">
              <div className="flex items-center gap-2">
                <FileText className="w-5 h-5 text-purple-600" />
                <div>
                  <div className="text-sm font-semibold">Requirements</div>
                  <div className="text-xs text-muted-foreground">Functional & Non-Functional</div>
                </div>
              </div>
            </CardContent>
          </Card>
          <Card className="border-l-4 border-l-blue-500">
            <CardContent className="pt-4 pb-4">
              <div className="flex items-center gap-2">
                <TrendingUp className="w-5 h-5 text-blue-600" />
                <div>
                  <div className="text-sm font-semibold">Estimation</div>
                  <div className="text-xs text-muted-foreground">PERT, FPA, UCP & More</div>
                </div>
              </div>
            </CardContent>
          </Card>
          <Card className="border-l-4 border-l-green-500">
            <CardContent className="pt-4 pb-4">
              <div className="flex items-center gap-2">
                <Shield className="w-5 h-5 text-green-600" />
                <div>
                  <div className="text-sm font-semibold">Security</div>
                  <div className="text-xs text-muted-foreground">Analysis & Compliance</div>
                </div>
              </div>
            </CardContent>
          </Card>
          <Card className="border-l-4 border-l-orange-500">
            <CardContent className="pt-4 pb-4">
              <div className="flex items-center gap-2">
                <Layers className="w-5 h-5 text-orange-600" />
                <div>
                  <div className="text-sm font-semibold">Architecture</div>
                  <div className="text-xs text-muted-foreground">HLD & LLD Generation</div>
                </div>
              </div>
            </CardContent>
          </Card>
          <Card className="border-l-4 border-l-pink-500">
            <CardContent className="pt-4 pb-4">
              <div className="flex items-center gap-2">
                <Database className="w-5 h-5 text-pink-600" />
                <div>
                  <div className="text-sm font-semibold">Data Design</div>
                  <div className="text-xs text-muted-foreground">ER & Flow Diagrams</div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Stats Cards */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
          {stats.map((stat, idx) => (
            <Card key={idx}>
              <CardContent className="pt-6">
                <div className="flex items-center justify-between">
                  <div>
                    <div className="text-2xl font-bold">{stat.value}</div>
                    <div className="text-sm text-muted-foreground">{stat.label}</div>
                  </div>
                  <div className={`w-12 h-12 rounded-lg ${stat.color} flex items-center justify-center`}>
                    <stat.icon className="w-6 h-6 text-white" />
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>

        {/* Search */}
        <Card className="mb-6">
          <CardContent className="pt-6">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-muted-foreground" />
              <Input
                placeholder="Search projects by name, client, or industry..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-10"
              />
            </div>
          </CardContent>
        </Card>

        {/* Projects Grid */}
        {isLoading ? (
          <Card>
            <CardContent className="py-16 text-center">
              <FolderOpen className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-xl font-semibold mb-2">Loading Projects...</h3>
              <p className="text-muted-foreground mb-6">
                Please wait while we fetch your projects
              </p>
            </CardContent>
          </Card>
        ) : projects.length === 0 ? (
          <Card>
            <CardContent className="py-16 text-center">
              <FolderOpen className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
              <h3 className="text-xl font-semibold mb-2">No Projects Yet</h3>
              <p className="text-muted-foreground mb-6">
                Start by creating your first project
              </p>
              <Button onClick={handleCreateProject} className="bg-blue-600 hover:bg-blue-700">
                <Plus className="w-4 h-4 mr-2" />
                Create First Project
              </Button>
            </CardContent>
          </Card>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {projects.map((project) => (
              <Card 
                key={project.id} 
                className="cursor-pointer hover:shadow-lg transition-shadow"
                onClick={() => navigate(`/projects/${project.id}`)}
              >
                <CardHeader>
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <CardTitle className="text-xl mb-2">{project.name}</CardTitle>
                      <CardDescription className="line-clamp-2">
                        {project.description || 'No description'}
                      </CardDescription>
                    </div>
                    <div className="flex items-center gap-2">
                      <Badge className={getStatusColor(project.status)}>
                        {project.status}
                      </Badge>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={(e) => handleDeleteProject(project.id, project.name, e)}
                        className="text-red-600 hover:text-red-700 hover:bg-red-50 h-8 w-8 p-0"
                      >
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>
                </CardHeader>
                <CardContent>
                  <div className="space-y-3">
                    {/* Project Info */}
                    <div className="grid grid-cols-2 gap-4 text-sm">
                      <div>
                        <div className="text-muted-foreground">Estimation</div>
                        <div className="font-semibold">
                          {project.estimationTechnique || 'Not set'}
                        </div>
                      </div>
                      <div>
                        <div className="text-muted-foreground">Created</div>
                        <div className="font-semibold">
                          {new Date(project.createdAt).toLocaleDateString()}
                        </div>
                      </div>
                    </div>

                    {/* Action Button */}
                    <div className="pt-2 border-t">
                      <Button 
                        variant="ghost" 
                        size="sm"
                        className="w-full"
                        onClick={(e) => {
                          e.stopPropagation();
                          navigate(`/projects/${project.id}`);
                        }}
                      >
                        View Details →
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}