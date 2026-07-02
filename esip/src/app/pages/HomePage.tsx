import { useNavigate } from 'react-router';
import { Calculator, Rocket, FileSpreadsheet, Presentation, TrendingUp, Sparkles, Calendar, FolderOpen, LogOut, GitBranch, Lightbulb, ClipboardList } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent } from '../components/ui/card';
import { useAuthStore } from '../store/authStore';
import { Badge } from '../components/ui/badge';

export default function HomePage() {
  const navigate = useNavigate();
  const { user, logout } = useAuthStore();

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 via-indigo-50 to-purple-50">
      <div className="bg-white border-b border-gray-200 shadow-sm">
        <div className="max-w-7xl mx-auto px-6 py-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Calculator className="w-8 h-8 text-blue-600" />
              <div>
                <h1 className="text-3xl font-bold text-gray-900">
                  Development Estimation Platform
                </h1>
                <p className="text-gray-600">Professional project estimation with 15+ methods</p>
              </div>
            </div>
            <div className="flex items-center gap-4">
              <div className="text-sm text-gray-600">
                {user?.fullName || user?.email} <span className="text-blue-600">({user?.role})</span>
              </div>
              <Button
                variant="outline"
                onClick={() => navigate('/projects')}
                className="flex items-center gap-2"
              >
                <FolderOpen className="w-4 h-4" />
                My Projects
              </Button>
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

      <div className="max-w-7xl mx-auto px-6 py-12">
        <div className="text-center mb-12">
          <h2 className="text-4xl font-bold text-gray-900 mb-4">
            Comprehensive Software Estimation Wizard
          </h2>
          <p className="text-xl text-gray-600 max-w-3xl mx-auto mb-8">
            Create professional development estimates using industry-standard methods.
            Generate Excel and PowerPoint deliverables ready for stakeholders.
          </p>
          <Button
            onClick={() => navigate('/wizard')}
            className="bg-blue-600 hover:bg-blue-700 text-white px-8 py-6 text-lg"
          >
            <Rocket className="w-5 h-5 mr-2" />
            Start New Estimation Wizard
          </Button>
        </div>

        <div className="grid md:grid-cols-3 gap-6 mt-16">
          <Card className="p-6">
            <div className="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center mb-4">
              <TrendingUp className="w-6 h-6 text-blue-600" />
            </div>
            <h3 className="text-xl font-bold text-gray-900 mb-2">15+ Estimation Methods</h3>
            <p className="text-gray-600">
              PERT, Story Points, T-Shirt sizing, Function Point Analysis, Use Case Points, and more
            </p>
          </Card>

          <Card className="p-6">
            <div className="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center mb-4">
              <FileSpreadsheet className="w-6 h-6 text-green-600" />
            </div>
            <h3 className="text-xl font-bold text-gray-900 mb-2">Excel Deliverables</h3>
            <p className="text-gray-600">
              Professional .xlsx files with proper formatting, colors, and Work Breakdown Structure
            </p>
          </Card>

          <Card className="p-6">
            <div className="w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center mb-4">
              <Presentation className="w-6 h-6 text-purple-600" />
            </div>
            <h3 className="text-xl font-bold text-gray-900 mb-2">PowerPoint Reports</h3>
            <p className="text-gray-600">
              CXO-ready presentations with executive summaries and detailed breakdowns
            </p>
          </Card>
        </div>

        {/* New Architecture Section */}
        <div className="mt-12">
          <Card className="bg-gradient-to-br from-amber-50 to-orange-50 border-2 border-amber-300 p-8">
            <div className="flex items-start gap-4">
              <div className="w-16 h-16 bg-amber-500 rounded-xl flex items-center justify-center flex-shrink-0">
                <GitBranch className="w-8 h-8 text-white" />
              </div>
              <div className="flex-1">
                <h3 className="text-2xl font-bold text-gray-900 mb-2">
                  🏗️ Architecture & Design Documents
                </h3>
                <p className="text-gray-700 mb-4">
                  Generate comprehensive architecture documentation including BFA (Business Flow), 
                  DFA (Data Flow), HLD (High-Level Design), and LLD (Low-Level Design) with 
                  interactive Mermaid diagrams.
                </p>
                <Button
                  onClick={() => navigate('/architecture')}
                  className="bg-amber-600 hover:bg-amber-700 text-white"
                >
                  <GitBranch className="w-4 h-4 mr-2" />
                  Create Architecture Documents
                </Button>
              </div>
            </div>
          </Card>
        </div>

        {/* NEW: Business Requirements Capture */}
        <div className="mt-8">
          <Card className="bg-gradient-to-br from-green-50 to-emerald-50 border-2 border-green-300 p-8">
            <div className="flex items-start gap-4">
              <div className="w-16 h-16 bg-green-500 rounded-xl flex items-center justify-center flex-shrink-0">
                <Lightbulb className="w-8 h-8 text-white" />
              </div>
              <div className="flex-1">
                <div className="flex items-center gap-2 mb-2">
                  <h3 className="text-2xl font-bold text-gray-900">
                    🎯 Smart Business Requirements Capture
                  </h3>
                  <span className="bg-green-500 text-white text-xs font-bold px-2 py-1 rounded">NEW</span>
                </div>
                <p className="text-gray-700 mb-4">
                  Industry-specific questionnaires with smart recommendations. Select your industry 
                  (Healthcare, Banking, E-commerce, etc.), choose business processes, and let our 
                  intelligent system suggest capabilities, integrations, and compliance requirements. 
                  Automatically generates WBS, user stories, and architecture components.
                </p>
                <div className="flex flex-wrap gap-2 mb-4">
                  <span className="text-xs bg-green-100 text-green-700 px-3 py-1 rounded-full">6 Industry Templates</span>
                  <span className="text-xs bg-green-100 text-green-700 px-3 py-1 rounded-full">Smart Questionnaires</span>
                  <span className="text-xs bg-green-100 text-green-700 px-3 py-1 rounded-full">Auto Suggestions</span>
                  <span className="text-xs bg-green-100 text-green-700 px-3 py-1 rounded-full">Compliance Checks</span>
                </div>
                <Button
                  onClick={() => navigate('/business/requirements/wizard')}
                  className="bg-green-600 hover:bg-green-700 text-white"
                >
                  <Sparkles className="w-4 h-4 mr-2" />
                  Start Requirements Capture
                </Button>
              </div>
            </div>
          </Card>
        </div>

        {/* NEW: Product Owner Requirements Board */}
        <div className="mt-8">
          <Card className="bg-gradient-to-br from-cyan-50 to-blue-50 border-2 border-cyan-300 p-8">
            <div className="flex items-start gap-4">
              <div className="w-16 h-16 bg-cyan-500 rounded-xl flex items-center justify-center flex-shrink-0">
                <ClipboardList className="w-8 h-8 text-white" />
              </div>
              <div className="flex-1">
                <div className="flex items-center gap-2 mb-2">
                  <h3 className="text-2xl font-bold text-gray-900">
                    📋 Product Owner Requirements Backlog
                  </h3>
                  <span className="bg-cyan-500 text-white text-xs font-bold px-2 py-1 rounded">NEW</span>
                </div>
                <p className="text-gray-700 mb-4">
                  Capture specific project requirements directly from Product Owners. Add new screens, 
                  update existing features, log bugs, and track changes. Each requirement includes 
                  acceptance criteria, user stories, technical details, and effort estimation. 
                  Perfect for Agile teams managing product backlogs.
                </p>
                <div className="grid grid-cols-2 md:grid-cols-5 gap-2 mb-4">
                  <div className="text-center bg-white rounded p-2 border border-cyan-200">
                    <div className="text-2xl">🖥️</div>
                    <div className="text-xs font-semibold">New Screens</div>
                  </div>
                  <div className="text-center bg-white rounded p-2 border border-cyan-200">
                    <div className="text-2xl">✏️</div>
                    <div className="text-xs font-semibold">Updates</div>
                  </div>
                  <div className="text-center bg-white rounded p-2 border border-cyan-200">
                    <div className="text-2xl">🐛</div>
                    <div className="text-xs font-semibold">Bug Fixes</div>
                  </div>
                  <div className="text-center bg-white rounded p-2 border border-cyan-200">
                    <div className="text-2xl">⚡</div>
                    <div className="text-xs font-semibold">Features</div>
                  </div>
                  <div className="text-center bg-white rounded p-2 border border-cyan-200">
                    <div className="text-2xl">🔌</div>
                    <div className="text-xs font-semibold">APIs</div>
                  </div>
                </div>
                <Button
                  onClick={() => navigate('/requirements/capture')}
                  className="bg-cyan-600 hover:bg-cyan-700 text-white"
                >
                  <ClipboardList className="w-4 h-4 mr-2" />
                  Manage Requirements Backlog
                </Button>
              </div>
            </div>
          </Card>
        </div>

        {/* Main Actions */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
          {/* Projects */}
          <Card className="hover:shadow-lg transition-shadow cursor-pointer border-2 border-blue-200" onClick={() => navigate('/projects/list')}>
            <CardContent className="pt-6">
              <div className="flex items-center justify-between mb-4">
                <div className="w-12 h-12 bg-blue-500 rounded-lg flex items-center justify-center">
                  <FolderOpen className="w-6 h-6 text-white" />
                </div>
                <div className="text-3xl font-bold text-blue-600">5</div>
              </div>
              <h3 className="font-semibold text-lg mb-2">Projects</h3>
              <p className="text-sm text-muted-foreground">
                Manage projects, modules, and requirements
              </p>
            </CardContent>
          </Card>

          {/* Estimation Wizard */}
          <Card className="hover:shadow-lg transition-shadow cursor-pointer" onClick={() => navigate('/wizard')}>
            <CardContent className="pt-6">
              <div className="flex items-center justify-between mb-4">
                <div className="w-12 h-12 bg-purple-500 rounded-lg flex items-center justify-center">
                  <Calculator className="w-6 h-6 text-white" />
                </div>
                <Sparkles className="w-6 h-6 text-purple-500" />
              </div>
              <h3 className="font-semibold text-lg mb-2">Estimation Wizard</h3>
              <p className="text-sm text-muted-foreground">
                5 methods: PERT, Story Points, T-Shirt sizing
              </p>
            </CardContent>
          </Card>

          {/* Architecture */}
          <Card className="hover:shadow-lg transition-shadow cursor-pointer" onClick={() => navigate('/architecture')}>
            <CardContent className="pt-6">
              <div className="flex items-center justify-between mb-4">
                <div className="w-12 h-12 bg-green-500 rounded-lg flex items-center justify-center">
                  <GitBranch className="w-6 h-6 text-white" />
                </div>
                <Badge className="bg-green-100 text-green-700">Auto-Gen</Badge>
              </div>
              <h3 className="font-semibold text-lg mb-2">Architecture</h3>
              <p className="text-sm text-muted-foreground">
                Generate HLD, LLD, ER Diagrams, Data Flows
              </p>
            </CardContent>
          </Card>
        </div>

        <div className="mt-12 bg-gradient-to-r from-purple-50 to-pink-50 border-2 border-purple-200 rounded-xl p-8">
          <div className="flex items-center gap-2 mb-4">
            <Sparkles className="w-6 h-6 text-purple-600" />
            <h3 className="text-2xl font-bold text-purple-900">✨ What's New</h3>
          </div>
          
          <div className="grid md:grid-cols-2 gap-6">
            <div className="bg-white rounded-lg p-5 border border-purple-200">
              <div className="flex items-center gap-3 mb-3">
                <div className="w-10 h-10 bg-purple-100 rounded-lg flex items-center justify-center">
                  <TrendingUp className="w-5 h-5 text-purple-600" />
                </div>
                <h4 className="font-bold text-gray-900">Method Comparison</h4>
              </div>
              <p className="text-sm text-gray-600 mb-3">
                Compare multiple estimation methods side-by-side with variance analysis, 
                visual charts, and smart recommendations.
              </p>
              <div className="flex flex-wrap gap-2">
                <span className="text-xs bg-purple-100 text-purple-700 px-2 py-1 rounded">Bar Charts</span>
                <span className="text-xs bg-purple-100 text-purple-700 px-2 py-1 rounded">Radar Charts</span>
                <span className="text-xs bg-purple-100 text-purple-700 px-2 py-1 rounded">Variance Analysis</span>
              </div>
              <p className="text-xs text-purple-600 mt-3 font-semibold">
                💡 Tip: Select 2+ methods in Step 1 to enable
              </p>
            </div>

            <div className="bg-white rounded-lg p-5 border border-purple-200">
              <div className="flex items-center gap-3 mb-3">
                <div className="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
                  <Calendar className="w-5 h-5 text-blue-600" />
                </div>
                <h4 className="font-bold text-gray-900">Capacity Planning</h4>
              </div>
              <p className="text-sm text-gray-600 mb-3">
                Break down estimates into executable sprints with resource allocation, 
                utilization tracking, and overallocation warnings.
              </p>
              <div className="flex flex-wrap gap-2">
                <span className="text-xs bg-blue-100 text-blue-700 px-2 py-1 rounded">Sprint Planning</span>
                <span className="text-xs bg-blue-100 text-blue-700 px-2 py-1 rounded">Utilization %</span>
                <span className="text-xs bg-blue-100 text-blue-700 px-2 py-1 rounded">Timeline View</span>
              </div>
              <p className="text-xs text-blue-600 mt-3 font-semibold">
                💡 Tip: Configure 1-4 week sprints in Step 8
              </p>
            </div>
          </div>
        </div>

        <div className="mt-12 bg-blue-50 border border-blue-200 rounded-lg p-6">
          <h4 className="font-bold text-blue-900 mb-2">What You'll Get:</h4>
          <ul className="space-y-2 text-blue-800">
            <li>✅ Comprehensive estimates with Development, QA, and Hardening phases</li>
            <li>✅ Risk assessment and contingency planning</li>
            <li>✅ Multiple estimation methods for comparison</li>
            <li>✅ Professional Excel and PowerPoint deliverables</li>
            <li>✅ Save drafts and resume later with localStorage</li>
          </ul>
        </div>
      </div>
    </div>
  );
}