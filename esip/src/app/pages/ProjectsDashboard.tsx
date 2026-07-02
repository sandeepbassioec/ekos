import { useState } from 'react';
import { useNavigate } from 'react-router';
import { useProjectsStore, SavedProject } from '../store/projectsStore';
import { useEstimationStore } from '../store/estimationStore';
import { Button } from '../components/ui/button';
import { Card } from '../components/ui/card';
import { Input } from '../components/ui/input';
import { Badge } from '../components/ui/badge';
import {
  FolderOpen,
  Plus,
  Search,
  Trash2,
  Copy,
  Edit,
  Calendar,
  TrendingUp,
  FileText,
  LayoutTemplate,
  ArrowLeft,
  Star,
} from 'lucide-react';
import { toast } from 'sonner';

export function ProjectsDashboard() {
  const navigate = useNavigate();
  const {
    projects,
    deleteProject,
    duplicateProject,
    loadProject,
    getAllProjects,
    getTemplates,
    searchProjects,
  } = useProjectsStore();
  
  const { resetEstimation, setCurrentProjectId } = useEstimationStore();
  
  const [searchQuery, setSearchQuery] = useState('');
  const [filterStatus, setFilterStatus] = useState<'ALL' | SavedProject['status']>('ALL');
  const [showTemplates, setShowTemplates] = useState(false);

  // Get filtered projects
  const getFilteredProjects = () => {
    if (showTemplates) {
      return getTemplates();
    }
    
    const baseProjects = searchQuery ? searchProjects(searchQuery) : getAllProjects();
    
    if (filterStatus === 'ALL') {
      return baseProjects;
    }
    
    return baseProjects.filter((p) => p.status === filterStatus);
  };

  const filteredProjects = getFilteredProjects();

  const handleLoadProject = (project: SavedProject) => {
    const estimationStore = useEstimationStore.getState();
    
    // Load project data into estimation store
    estimationStore.setSelectedMethods(project.selectedMethods);
    estimationStore.setProject(project.project);
    
    // Load screens
    project.screens.forEach((screen) => {
      const existingScreen = estimationStore.screens.find(s => s.id === screen.id);
      if (!existingScreen) {
        estimationStore.addScreen(screen);
      }
    });
    
    // Load other data
    estimationStore.setContingency(project.contingency);
    project.riskFactors.forEach((risk) => {
      estimationStore.addRiskFactor(risk);
    });
    
    if (project.capacityPlan) {
      estimationStore.setCapacityPlan(project.capacityPlan);
    }
    
    // IMPORTANT: Set current project ID so we know we're editing an existing project
    setCurrentProjectId(project.id);
    
    toast.success(`Loaded project: ${project.name}`);
    navigate('/wizard/step/1');
  };

  const handleDuplicateProject = (id: string) => {
    const newId = duplicateProject(id);
    if (newId) {
      toast.success('Project duplicated successfully!');
    } else {
      toast.error('Failed to duplicate project');
    }
  };

  const handleDeleteProject = (id: string, name: string) => {
    if (window.confirm(`Are you sure you want to delete "${name}"?`)) {
      deleteProject(id);
      toast.success('Project deleted');
    }
  };

  const handleNewProject = () => {
    resetEstimation();
    navigate('/wizard/step/1');
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    
    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
    if (diffDays < 365) return `${Math.floor(diffDays / 30)} months ago`;
    return date.toLocaleDateString();
  };

  const getStatusColor = (status: SavedProject['status']) => {
    switch (status) {
      case 'DRAFT':
        return 'bg-gray-100 text-gray-700';
      case 'IN_PROGRESS':
        return 'bg-blue-100 text-blue-700';
      case 'COMPLETED':
        return 'bg-green-100 text-green-700';
      case 'ARCHIVED':
        return 'bg-orange-100 text-orange-700';
      default:
        return 'bg-gray-100 text-gray-700';
    }
  };

  const getStatusIcon = (status: SavedProject['status']) => {
    switch (status) {
      case 'DRAFT':
        return '📝';
      case 'IN_PROGRESS':
        return '🚧';
      case 'COMPLETED':
        return '✅';
      case 'ARCHIVED':
        return '📦';
      default:
        return '📄';
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-indigo-50 to-purple-50">
      {/* Header */}
      <div className="bg-white border-b border-gray-200 shadow-sm">
        <div className="max-w-7xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <Button variant="ghost" size="sm" onClick={() => navigate('/')}>
                <ArrowLeft className="w-4 h-4 mr-2" />
                Home
              </Button>
              <div className="h-6 w-px bg-gray-300"></div>
              <div className="flex items-center gap-3">
                <FolderOpen className="w-6 h-6 text-blue-600" />
                <h1 className="text-2xl font-bold text-gray-900">My Projects</h1>
              </div>
            </div>
            <Button onClick={handleNewProject} className="bg-blue-600 hover:bg-blue-700">
              <Plus className="w-4 h-4 mr-2" />
              New Project
            </Button>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="max-w-7xl mx-auto px-6 py-8">
        {/* Search and Filters */}
        <div className="bg-white rounded-lg shadow-sm p-6 mb-6">
          <div className="flex flex-col md:flex-row gap-4">
            {/* Search */}
            <div className="flex-1">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
                <Input
                  placeholder="Search projects..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-10"
                />
              </div>
            </div>

            {/* Status Filter */}
            <div className="flex gap-2">
              <Button
                variant={filterStatus === 'ALL' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setFilterStatus('ALL')}
              >
                All
              </Button>
              <Button
                variant={filterStatus === 'DRAFT' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setFilterStatus('DRAFT')}
              >
                Draft
              </Button>
              <Button
                variant={filterStatus === 'IN_PROGRESS' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setFilterStatus('IN_PROGRESS')}
              >
                In Progress
              </Button>
              <Button
                variant={filterStatus === 'COMPLETED' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setFilterStatus('COMPLETED')}
              >
                Completed
              </Button>
            </div>

            {/* Templates Toggle */}
            <Button
              variant={showTemplates ? 'default' : 'outline'}
              size="sm"
              onClick={() => setShowTemplates(!showTemplates)}
            >
              <LayoutTemplate className="w-4 h-4 mr-2" />
              Templates
            </Button>
          </div>
        </div>

        {/* Projects Grid */}
        {filteredProjects.length === 0 ? (
          <div className="text-center py-16">
            <FolderOpen className="w-16 h-16 text-gray-300 mx-auto mb-4" />
            <h3 className="text-xl font-semibold text-gray-700 mb-2">
              {showTemplates ? 'No templates yet' : 'No projects found'}
            </h3>
            <p className="text-gray-500 mb-6">
              {showTemplates
                ? 'Save a project as a template to reuse it later'
                : searchQuery
                ? 'Try adjusting your search query'
                : 'Create your first estimation project to get started'}
            </p>
            {!showTemplates && (
              <Button onClick={handleNewProject} className="bg-blue-600 hover:bg-blue-700">
                <Plus className="w-4 h-4 mr-2" />
                Create New Project
              </Button>
            )}
          </div>
        ) : (
          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            {filteredProjects.map((project) => (
              <Card
                key={project.id}
                className="p-6 hover:shadow-lg transition-shadow cursor-pointer group"
                onClick={() => handleLoadProject(project)}
              >
                {/* Header */}
                <div className="flex items-start justify-between mb-4">
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-2">
                      <h3 className="font-bold text-lg text-gray-900 truncate">
                        {project.name}
                      </h3>
                      {project.isTemplate && (
                        <Star className="w-4 h-4 text-yellow-500 fill-yellow-500" />
                      )}
                    </div>
                    <Badge className={getStatusColor(project.status)}>
                      {getStatusIcon(project.status)} {project.status}
                    </Badge>
                  </div>
                </div>

                {/* Description */}
                {project.description && (
                  <p className="text-sm text-gray-600 mb-4 line-clamp-2">
                    {project.description}
                  </p>
                )}

                {/* Metadata */}
                <div className="space-y-2 mb-4">
                  <div className="flex items-center gap-2 text-sm text-gray-600">
                    <FileText className="w-4 h-4" />
                    <span>{project.screens.length} screens</span>
                  </div>
                  <div className="flex items-center gap-2 text-sm text-gray-600">
                    <TrendingUp className="w-4 h-4" />
                    <span>{project.selectedMethods.length} methods</span>
                  </div>
                  <div className="flex items-center gap-2 text-sm text-gray-600">
                    <Calendar className="w-4 h-4" />
                    <span>Updated {formatDate(project.updatedAt)}</span>
                  </div>
                  {project.results && (
                    <div className="text-sm font-semibold text-blue-600">
                      {project.results.totalHours}h • ${project.results.totalCost.toLocaleString()}
                    </div>
                  )}
                </div>

                {/* Tags */}
                {project.tags && project.tags.length > 0 && (
                  <div className="flex flex-wrap gap-2 mb-4">
                    {project.tags.slice(0, 3).map((tag, idx) => (
                      <span
                        key={idx}
                        className="text-xs bg-purple-100 text-purple-700 px-2 py-1 rounded"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                )}

                {/* Actions */}
                <div className="flex gap-2 pt-4 border-t border-gray-200 opacity-0 group-hover:opacity-100 transition-opacity">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleLoadProject(project);
                    }}
                  >
                    <Edit className="w-4 h-4 mr-1" />
                    Edit
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDuplicateProject(project.id);
                    }}
                  >
                    <Copy className="w-4 h-4 mr-1" />
                    Copy
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDeleteProject(project.id, project.name);
                    }}
                    className="text-red-600 hover:text-red-700 hover:bg-red-50"
                  >
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </Card>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}