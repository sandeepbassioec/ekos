import React, { useState, useEffect } from 'react';
import { Download, FileText, Eye, Edit3, Save } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import { Textarea } from '../ui/textarea';
import { Label } from '../ui/label';
import { MermaidDiagram } from './MermaidDiagram';
import { toast } from 'sonner';

interface Module {
  id: string;
  name: string;
  description: string;
  screens: Screen[];
}

interface Screen {
  id: string;
  name: string;
  description: string;
  components?: string[];
}

export function BFAGenerator() {
  const [modules, setModules] = useState<Module[]>([]);
  const [selectedModule, setSelectedModule] = useState<string>('');
  const [userJourneyDiagram, setUserJourneyDiagram] = useState('');
  const [processFlowDiagram, setProcessFlowDiagram] = useState('');
  const [useCaseDiagram, setUseCaseDiagram] = useState('');
  const [editMode, setEditMode] = useState(false);
  const [customDiagram, setCustomDiagram] = useState('');

  useEffect(() => {
    // Load sample data or from estimation data
    loadModulesFromEstimation();
  }, []);

  const loadModulesFromEstimation = () => {
    // Load from localStorage or API
    const sampleModules: Module[] = [
      {
        id: 'auth',
        name: 'Authentication Module',
        description: 'User authentication and authorization',
        screens: [
          { id: 'login', name: 'Login Screen', description: 'User login with email/password' },
          { id: 'register', name: 'Register Screen', description: 'New user registration' },
          { id: 'forgot-password', name: 'Forgot Password', description: 'Password recovery flow' },
        ],
      },
      {
        id: 'dashboard',
        name: 'Dashboard Module',
        description: 'Main dashboard and analytics',
        screens: [
          { id: 'main-dashboard', name: 'Main Dashboard', description: 'Overview and key metrics' },
          { id: 'analytics', name: 'Analytics Screen', description: 'Detailed analytics and reports' },
        ],
      },
      {
        id: 'project',
        name: 'Project Management Module',
        description: 'Project creation and management',
        screens: [
          { id: 'project-list', name: 'Project List', description: 'List all projects' },
          { id: 'project-detail', name: 'Project Detail', description: 'View project details' },
          { id: 'create-project', name: 'Create Project', description: 'Create new project' },
        ],
      },
    ];
    
    setModules(sampleModules);
    if (sampleModules.length > 0) {
      setSelectedModule(sampleModules[0].id);
      generateDiagrams(sampleModules[0]);
    }
  };

  const generateDiagrams = (module: Module) => {
    // Generate User Journey Diagram
    const userJourney = `graph TD
    A[User Arrives] --> B{Authenticated?}
    B -->|No| C[Login Screen]
    B -->|Yes| D[Dashboard]
    C --> E{Valid Credentials?}
    E -->|Yes| D
    E -->|No| C
    D --> F[${module.name}]
    ${module.screens.map((screen, idx) => 
      `F --> S${idx}[${screen.name}]`
    ).join('\n    ')}
    ${module.screens.map((screen, idx) => 
      `S${idx} --> G[Complete Action]`
    ).join('\n    ')}
    G --> H[Success/Feedback]`;
    setUserJourneyDiagram(userJourney);

    // Generate Process Flow Diagram
    const processFlow = `flowchart TB
    Start([Start]) --> Input[User Input]
    Input --> Validate{Validation}
    Validate -->|Invalid| Error[Show Error]
    Error --> Input
    Validate -->|Valid| Process[Process Request]
    Process --> Backend[Backend API Call]
    Backend --> DB[(Database)]
    DB --> Response[Generate Response]
    Response --> Display[Display Results]
    Display --> End([End])`;
    setProcessFlowDiagram(processFlow);

    // Generate Use Case Diagram
    const useCase = `graph LR
    User((User)) --> UC1[${module.screens[0]?.name || 'Screen 1'}]
    User --> UC2[${module.screens[1]?.name || 'Screen 2'}]
    ${module.screens.length > 2 ? `User --> UC3[${module.screens[2].name}]` : ''}
    Admin((Admin)) --> UC1
    Admin --> UC4[Manage Settings]
    Admin --> UC5[View Reports]
    UC1 --> System[System Backend]
    UC2 --> System
    ${module.screens.length > 2 ? 'UC3 --> System' : ''}
    UC4 --> System
    UC5 --> System
    System --> DB[(Database)]`;
    setUseCaseDiagram(useCase);
    setCustomDiagram(userJourney);
  };

  const handleModuleChange = (moduleId: string) => {
    setSelectedModule(moduleId);
    const module = modules.find(m => m.id === moduleId);
    if (module) {
      generateDiagrams(module);
    }
  };

  const exportToPDF = async () => {
    toast.info('PDF export will be implemented with jsPDF');
    // Will implement jsPDF export
  };

  const exportToImage = async () => {
    toast.info('Image export functionality coming soon');
    // Will implement SVG to PNG conversion
  };

  const saveCustomDiagram = () => {
    setEditMode(false);
    toast.success('Custom diagram saved!');
    // Save to localStorage or API
    localStorage.setItem(`bfa-custom-${selectedModule}`, customDiagram);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold">Business Flow Architecture (BFA)</h2>
          <p className="text-muted-foreground">Generate visual business process flows and user journeys</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={exportToImage}>
            <Download className="w-4 h-4 mr-2" />
            Export PNG
          </Button>
          <Button onClick={exportToPDF}>
            <FileText className="w-4 h-4 mr-2" />
            Export PDF
          </Button>
        </div>
      </div>

      {/* Module Selection */}
      <Card>
        <CardHeader>
          <CardTitle>Select Module</CardTitle>
          <CardDescription>Choose a module to generate business flow diagrams</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-2">
            {modules.map((module) => (
              <Button
                key={module.id}
                variant={selectedModule === module.id ? 'default' : 'outline'}
                onClick={() => handleModuleChange(module.id)}
              >
                {module.name}
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Diagram Tabs */}
      <Tabs defaultValue="user-journey" className="w-full">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="user-journey">User Journey</TabsTrigger>
          <TabsTrigger value="process-flow">Process Flow</TabsTrigger>
          <TabsTrigger value="use-case">Use Case</TabsTrigger>
          <TabsTrigger value="custom">Custom</TabsTrigger>
        </TabsList>

        <TabsContent value="user-journey" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>User Journey Diagram</CardTitle>
              <CardDescription>Visual representation of user navigation flow</CardDescription>
            </CardHeader>
            <CardContent>
              <MermaidDiagram chart={userJourneyDiagram} id="user-journey" />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="process-flow" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Process Flow Diagram</CardTitle>
              <CardDescription>Step-by-step business process flow</CardDescription>
            </CardHeader>
            <CardContent>
              <MermaidDiagram chart={processFlowDiagram} id="process-flow" />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="use-case" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Use Case Diagram</CardTitle>
              <CardDescription>Actor-system interaction model</CardDescription>
            </CardHeader>
            <CardContent>
              <MermaidDiagram chart={useCaseDiagram} id="use-case" />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="custom" className="space-y-4">
          <Card>
            <CardHeader>
              <div className="flex justify-between items-center">
                <div>
                  <CardTitle>Custom Diagram</CardTitle>
                  <CardDescription>Edit or create custom Mermaid diagrams</CardDescription>
                </div>
                <div className="flex gap-2">
                  {editMode ? (
                    <>
                      <Button variant="outline" onClick={() => setEditMode(false)}>
                        <Eye className="w-4 h-4 mr-2" />
                        Preview
                      </Button>
                      <Button onClick={saveCustomDiagram}>
                        <Save className="w-4 h-4 mr-2" />
                        Save
                      </Button>
                    </>
                  ) : (
                    <Button onClick={() => setEditMode(true)}>
                      <Edit3 className="w-4 h-4 mr-2" />
                      Edit
                    </Button>
                  )}
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              {editMode ? (
                <>
                  <div>
                    <Label>Mermaid Diagram Code</Label>
                    <Textarea
                      value={customDiagram}
                      onChange={(e) => setCustomDiagram(e.target.value)}
                      className="font-mono text-sm min-h-[300px]"
                      placeholder="Enter Mermaid diagram code here..."
                    />
                  </div>
                  <div className="text-sm text-muted-foreground">
                    <p className="font-semibold mb-2">Mermaid Syntax Examples:</p>
                    <ul className="list-disc list-inside space-y-1">
                      <li>Flowchart: <code>graph TD; A--&gt;B;</code></li>
                      <li>Sequence: <code>sequenceDiagram; A-&gt;&gt;B: Hello</code></li>
                      <li>Class: <code>classDiagram; Class01 &lt;|-- Class02</code></li>
                    </ul>
                  </div>
                </>
              ) : (
                <MermaidDiagram chart={customDiagram} id="custom" />
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Module Details */}
      {selectedModule && (
        <Card>
          <CardHeader>
            <CardTitle>Module Details</CardTitle>
          </CardHeader>
          <CardContent>
            {modules.find(m => m.id === selectedModule) && (
              <div className="space-y-4">
                <div>
                  <h3 className="font-semibold text-lg">
                    {modules.find(m => m.id === selectedModule)?.name}
                  </h3>
                  <p className="text-muted-foreground">
                    {modules.find(m => m.id === selectedModule)?.description}
                  </p>
                </div>
                <div>
                  <h4 className="font-semibold mb-2">Screens:</h4>
                  <ul className="space-y-2">
                    {modules.find(m => m.id === selectedModule)?.screens.map((screen) => (
                      <li key={screen.id} className="pl-4 border-l-2 border-primary">
                        <p className="font-medium">{screen.name}</p>
                        <p className="text-sm text-muted-foreground">{screen.description}</p>
                      </li>
                    ))}
                  </ul>
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  );
}