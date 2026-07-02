import React, { useState, useEffect } from 'react';
import { useNavigate, useSearchParams, useParams } from 'react-router';
import { ArrowLeft, Plus, Trash2, Save } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../components/ui/card';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Textarea } from '../components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../components/ui/select';
import { Checkbox } from '../components/ui/checkbox';
import { Badge } from '../components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../components/ui/tabs';
import { RequirementType, RequirementPriority, RequirementComplexity, AcceptanceCriterion, FieldDefinition } from '../types/requirements';
import { toast } from 'sonner';
import { useRequirementsStore } from '../store/requirementsStore';
import { useAuthStore } from '../store/authStore';
import { dalToBoard, isBoardReq, boardMetadata } from '../lib/boardRequirementMapper';

export default function CreateRequirement() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { id, projectId, moduleId } = useParams();
  const template = searchParams.get('template') as RequirementType | null;
  const isEditing = !!id;

  const { requirements: dalReqs, fetchRequirements, createRequirement, updateRequirement } = useRequirementsStore();
  const { user } = useAuthStore();

  // Load project requirements on mount so edit mode can find the existing record
  useEffect(() => {
    if (projectId) fetchRequirements(projectId);
  }, [projectId, fetchRequirements]);

  const [formData, setFormData] = useState({
    type: template || 'new_screen' as RequirementType,
    title: '',
    description: '',
    businessValue: '',
    priority: 'medium' as RequirementPriority,
    complexity: 'medium' as RequirementComplexity,
    estimatedHours: 0,
    affectedScreens: [] as string[],
    affectedAPIs: [] as string[],
    programIncrement: '' as string,
    sprint: '' as string,
  });

  const [userStory, setUserStory] = useState({
    asA: '',
    iWant: '',
    soThat: '',
  });

  const [acceptanceCriteria, setAcceptanceCriteria] = useState<AcceptanceCriterion[]>([]);
  const [newCriterion, setNewCriterion] = useState('');

  // Screen-specific fields
  const [screenDetails, setScreenDetails] = useState({
    screenName: '',
    screenType: 'form' as 'list' | 'form' | 'detail' | 'dashboard' | 'modal' | 'other',
    hasDataTable: false,
    hasForms: true,
    hasCharts: false,
    hasFileUpload: false,
    operations: {
      create: true,
      read: true,
      update: true,
      delete: false,
      export: false,
      import: false,
    },
  });

  const [fields, setFields] = useState<FieldDefinition[]>([]);
  const [newField, setNewField] = useState<FieldDefinition>({
    name: '',
    label: '',
    type: 'text',
    required: false,
  });

  // Bug-specific fields
  const [bugDetails, setBugDetails] = useState({
    severity: 'medium' as 'critical' | 'high' | 'medium' | 'low',
    stepsToReproduce: [''],
    expectedBehavior: '',
    actualBehavior: '',
    environment: '',
  });

  // Populate form when editing an existing requirement
  useEffect(() => {
    if (!isEditing || !id || dalReqs.length === 0) return;
    const dalReq = dalReqs.find(r => r.id === id && isBoardReq(r));
    if (!dalReq) return;
    const board = dalToBoard(dalReq);
    setFormData({
      type: board.type,
      title: board.title,
      description: board.description,
      businessValue: board.businessValue,
      priority: board.priority,
      complexity: board.complexity,
      estimatedHours: board.estimatedHours ?? 0,
      affectedScreens: board.affectedScreens,
      affectedAPIs: board.affectedAPIs,
      programIncrement: board.programIncrement ?? '',
      sprint: board.sprint ?? '',
    });
    setUserStory(board.userStory ?? { asA: '', iWant: '', soThat: '' });
    setAcceptanceCriteria(board.acceptanceCriteria);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isEditing, id, dalReqs]);

  const requirementTypes = [
    { value: 'new_screen', label: 'New Screen', icon: '🖥️' },
    { value: 'update_screen', label: 'Update Screen', icon: '✏️' },
    { value: 'bug_fix', label: 'Bug Fix', icon: '🐛' },
    { value: 'feature_change', label: 'Feature Change', icon: '⚡' },
    { value: 'api_development', label: 'API Development', icon: '🔌' },
    { value: 'integration', label: 'Integration', icon: '🔗' },
    { value: 'data_migration', label: 'Data Migration', icon: '📦' },
  ];

  const handleAddCriterion = () => {
    if (newCriterion.trim()) {
      setAcceptanceCriteria([
        ...acceptanceCriteria,
        { id: Date.now().toString(), description: newCriterion, completed: false },
      ]);
      setNewCriterion('');
    }
  };

  const handleRemoveCriterion = (id: string) => {
    setAcceptanceCriteria(acceptanceCriteria.filter((ac) => ac.id !== id));
  };

  const handleAddField = () => {
    if (newField.name && newField.label) {
      setFields([...fields, { ...newField }]);
      setNewField({ name: '', label: '', type: 'text', required: false });
    } else {
      toast.error('Field name and label are required');
    }
  };

  const handleRemoveField = (index: number) => {
    setFields(fields.filter((_, i) => i !== index));
  };

  const handleSave = async () => {
    if (!formData.title || !formData.description) {
      toast.error('Title and description are required');
      return;
    }

    const meta = boardMetadata({
      ...formData,
      userStory,
      acceptanceCriteria,
      moduleIds: [],
      affectedComponents: [],
      technicalNotes: '',
      dependencies: [],
      blockers: [],
      attachments: [],
      comments: [],
      screenDetails: (formData.type === 'new_screen' || formData.type === 'update_screen')
        ? { ...screenDetails, fields } : undefined,
      bugDetails: formData.type === 'bug_fix' ? bugDetails : undefined,
    });

    try {
      if (isEditing && id) {
        await updateRequirement(id, {
          title: formData.title,
          description: formData.description,
          category: formData.type,
          priority: formData.priority,
          metadata: meta,
        });
        toast.success('Requirement updated successfully!');
      } else {
        await createRequirement({
          projectId: projectId || '',
          title: formData.title,
          description: formData.description,
          category: formData.type,
          priority: formData.priority,
          status: 'draft',
          tags: ['board'],
          createdBy: user?.uid ?? 'unknown',
          metadata: meta,
        });
        toast.success('Requirement created successfully!');
      }

      navigate(projectId ? `/projects/${projectId}/requirements` : '/requirements/board');
    } catch (err) {
      toast.error('Failed to save requirement. Please try again.');
    }
  };

  // Determine back navigation path
  const getBackPath = () => {
    if (projectId) {
      return `/projects/${projectId}/requirements`;
    }
    return '/requirements/board';
  };

  const getBackLabel = () => {
    if (projectId) {
      return 'Back to Project Requirements';
    }
    return 'Back to Requirements';
  };

  return (
    <div className="container max-w-5xl mx-auto py-8 px-4">
      <div className="mb-8">
        <Button variant="ghost" onClick={() => navigate(getBackPath())}>
          <ArrowLeft className="w-4 h-4 mr-2" />
          {getBackLabel()}
        </Button>
        <h1 className="text-3xl font-bold mt-4">Create New Requirement</h1>
        <p className="text-muted-foreground">Add a new requirement from Product Owner</p>
      </div>

      <div className="space-y-6">
        {/* Basic Information */}
        <Card>
          <CardHeader>
            <CardTitle>Basic Information</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <Label>Requirement Type *</Label>
              <Select value={formData.type} onValueChange={(v) => setFormData({ ...formData, type: v as RequirementType })}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {requirementTypes.map((type) => (
                    <SelectItem key={type.value} value={type.value}>
                      <span className="mr-2">{type.icon}</span>
                      {type.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label>Title *</Label>
              <Input
                placeholder="e.g., Create User Registration Screen"
                value={formData.title}
                onChange={(e) => setFormData({ ...formData, title: e.target.value })}
              />
            </div>

            <div>
              <Label>Description *</Label>
              <Textarea
                placeholder="Detailed description of the requirement..."
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                rows={4}
              />
            </div>

            <div className="grid grid-cols-3 gap-4">
              <div>
                <Label>Priority</Label>
                <Select value={formData.priority} onValueChange={(v) => setFormData({ ...formData, priority: v as RequirementPriority })}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="critical">Critical</SelectItem>
                    <SelectItem value="high">High</SelectItem>
                    <SelectItem value="medium">Medium</SelectItem>
                    <SelectItem value="low">Low</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div>
                <Label>Complexity</Label>
                <Select value={formData.complexity} onValueChange={(v) => setFormData({ ...formData, complexity: v as RequirementComplexity })}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="simple">Simple</SelectItem>
                    <SelectItem value="medium">Medium</SelectItem>
                    <SelectItem value="complex">Complex</SelectItem>
                    <SelectItem value="very_complex">Very Complex</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div>
                <Label>Estimated Hours</Label>
                <Input
                  type="number"
                  value={formData.estimatedHours}
                  onChange={(e) => setFormData({ ...formData, estimatedHours: parseInt(e.target.value) || 0 })}
                />
              </div>
            </div>
          </CardContent>
        </Card>

        {/* PI & Sprint Assignment */}
        <Card className="border-2 border-purple-200 bg-purple-50/30">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <span>📅</span>
              Program Increment & Sprint Assignment
            </CardTitle>
            <CardDescription>
              Assign this requirement to a specific PI and Sprint (optional - can be assigned later)
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label>Program Increment</Label>
                <Select
                  value={formData.programIncrement || '__none__'}
                  onValueChange={(v) => setFormData({ ...formData, programIncrement: v === '__none__' ? '' : v, sprint: '' })}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select PI (Optional)" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="__none__">Unassigned (Backlog)</SelectItem>
                    <SelectItem value="PI-1">PI 1 (Jan - Mar 2024)</SelectItem>
                    <SelectItem value="PI-2">PI 2 (Apr - Jun 2024)</SelectItem>
                    <SelectItem value="PI-3">PI 3 (Jul - Sep 2024)</SelectItem>
                    <SelectItem value="PI-4">PI 4 (Oct - Dec 2024)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div>
                <Label>Sprint</Label>
                <Select
                  value={formData.sprint || '__none__'}
                  onValueChange={(v) => setFormData({ ...formData, sprint: v === '__none__' ? '' : v })}
                  disabled={!formData.programIncrement}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={formData.programIncrement ? "Select Sprint" : "Select PI first"} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="__none__">Unassigned (PI Backlog)</SelectItem>
                    <SelectItem value="Sprint-1">Sprint 1 (Week 1-2)</SelectItem>
                    <SelectItem value="Sprint-2">Sprint 2 (Week 3-4)</SelectItem>
                    <SelectItem value="Sprint-3">Sprint 3 (Week 5-6)</SelectItem>
                    <SelectItem value="Sprint-4">Sprint 4 (Week 7-8)</SelectItem>
                    <SelectItem value="Sprint-5">Sprint 5 (Week 9-10)</SelectItem>
                    <SelectItem value="Sprint-6">Sprint 6 (Week 11-12)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            
            {formData.programIncrement && (
              <div className="bg-blue-50 border border-blue-200 rounded-lg p-3">
                <p className="text-sm text-blue-900">
                  <strong>📍 Assignment:</strong> This requirement will be part of{' '}
                  <strong>{formData.programIncrement}</strong>
                  {formData.sprint && (
                    <>, specifically in <strong>{formData.sprint}</strong></>
                  )}
                  {!formData.sprint && (
                    <> backlog (not assigned to specific sprint yet)</>
                  )}
                </p>
              </div>
            )}
            
            {!formData.programIncrement && (
              <div className="bg-gray-50 border border-gray-200 rounded-lg p-3">
                <p className="text-sm text-gray-600">
                  <strong>📦 Unassigned:</strong> This requirement will go into the product backlog. 
                  You can assign it to a PI/Sprint later from the requirements board.
                </p>
              </div>
            )}
          </CardContent>
        </Card>

        {/* User Story */}
        <Card>
          <CardHeader>
            <CardTitle>User Story (Optional)</CardTitle>
            <CardDescription>As a [role], I want [feature], so that [benefit]</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <Label>As a...</Label>
              <Input
                placeholder="e.g., Customer"
                value={userStory.asA}
                onChange={(e) => setUserStory({ ...userStory, asA: e.target.value })}
              />
            </div>
            <div>
              <Label>I want...</Label>
              <Input
                placeholder="e.g., to register my account"
                value={userStory.iWant}
                onChange={(e) => setUserStory({ ...userStory, iWant: e.target.value })}
              />
            </div>
            <div>
              <Label>So that...</Label>
              <Input
                placeholder="e.g., I can access personalized features"
                value={userStory.soThat}
                onChange={(e) => setUserStory({ ...userStory, soThat: e.target.value })}
              />
            </div>
            {userStory.asA && userStory.iWant && userStory.soThat && (
              <div className="bg-blue-50 border border-blue-200 rounded p-4 mt-4">
                <p className="text-sm text-blue-900">
                  <strong>Story:</strong> As a <strong>{userStory.asA}</strong>, I want <strong>{userStory.iWant}</strong>, so that <strong>{userStory.soThat}</strong>.
                </p>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Type-specific Details */}
        <Tabs value={formData.type} onValueChange={(v) => setFormData({ ...formData, type: v as RequirementType })}>
          <TabsList>
            <TabsTrigger value="new_screen">Screen Details</TabsTrigger>
            <TabsTrigger value="bug_fix">Bug Details</TabsTrigger>
            <TabsTrigger value="api_development">API Details</TabsTrigger>
          </TabsList>

          {/* Screen Details */}
          <TabsContent value="new_screen">
            <Card>
              <CardHeader>
                <CardTitle>Screen Details</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <Label>Screen Name</Label>
                    <Input
                      placeholder="e.g., User Registration"
                      value={screenDetails.screenName}
                      onChange={(e) => setScreenDetails({ ...screenDetails, screenName: e.target.value })}
                    />
                  </div>
                  <div>
                    <Label>Screen Type</Label>
                    <Select value={screenDetails.screenType} onValueChange={(v: any) => setScreenDetails({ ...screenDetails, screenType: v })}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="list">List/Table</SelectItem>
                        <SelectItem value="form">Form</SelectItem>
                        <SelectItem value="detail">Detail View</SelectItem>
                        <SelectItem value="dashboard">Dashboard</SelectItem>
                        <SelectItem value="modal">Modal/Dialog</SelectItem>
                        <SelectItem value="other">Other</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <div>
                  <Label className="mb-3 block">UI Components</Label>
                  <div className="grid grid-cols-2 gap-3">
                    <div className="flex items-center space-x-2">
                      <Checkbox
                        checked={screenDetails.hasDataTable}
                        onCheckedChange={(c) => setScreenDetails({ ...screenDetails, hasDataTable: !!c })}
                      />
                      <Label>Has Data Table</Label>
                    </div>
                    <div className="flex items-center space-x-2">
                      <Checkbox
                        checked={screenDetails.hasForms}
                        onCheckedChange={(c) => setScreenDetails({ ...screenDetails, hasForms: !!c })}
                      />
                      <Label>Has Forms</Label>
                    </div>
                    <div className="flex items-center space-x-2">
                      <Checkbox
                        checked={screenDetails.hasCharts}
                        onCheckedChange={(c) => setScreenDetails({ ...screenDetails, hasCharts: !!c })}
                      />
                      <Label>Has Charts</Label>
                    </div>
                    <div className="flex items-center space-x-2">
                      <Checkbox
                        checked={screenDetails.hasFileUpload}
                        onCheckedChange={(c) => setScreenDetails({ ...screenDetails, hasFileUpload: !!c })}
                      />
                      <Label>Has File Upload</Label>
                    </div>
                  </div>
                </div>

                <div>
                  <Label className="mb-3 block">Data Operations (CRUD)</Label>
                  <div className="grid grid-cols-3 gap-3">
                    {Object.entries(screenDetails.operations).map(([key, value]) => (
                      <div key={key} className="flex items-center space-x-2">
                        <Checkbox
                          checked={value}
                          onCheckedChange={(c) => setScreenDetails({
                            ...screenDetails,
                            operations: { ...screenDetails.operations, [key]: !!c },
                          })}
                        />
                        <Label className="capitalize">{key}</Label>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Form Fields */}
                {screenDetails.hasForms && (
                  <div>
                    <Label className="mb-3 block">Form Fields</Label>
                    <div className="space-y-3">
                      {fields.map((field, idx) => (
                        <div key={idx} className="flex items-center gap-2 p-3 border rounded">
                          <div className="flex-1">
                            <div className="font-medium">{field.label}</div>
                            <div className="text-xs text-muted-foreground">
                              {field.name} • {field.type} {field.required && '• Required'}
                            </div>
                          </div>
                          <Button variant="ghost" size="sm" onClick={() => handleRemoveField(idx)}>
                            <Trash2 className="w-4 h-4 text-red-500" />
                          </Button>
                        </div>
                      ))}

                      <div className="border rounded p-4 space-y-3">
                        <div className="grid grid-cols-3 gap-3">
                          <Input
                            placeholder="Field Name (e.g., email)"
                            value={newField.name}
                            onChange={(e) => setNewField({ ...newField, name: e.target.value })}
                          />
                          <Input
                            placeholder="Label (e.g., Email Address)"
                            value={newField.label}
                            onChange={(e) => setNewField({ ...newField, label: e.target.value })}
                          />
                          <Select value={newField.type} onValueChange={(v: any) => setNewField({ ...newField, type: v })}>
                            <SelectTrigger>
                              <SelectValue placeholder="Type" />
                            </SelectTrigger>
                            <SelectContent>
                              <SelectItem value="text">Text</SelectItem>
                              <SelectItem value="email">Email</SelectItem>
                              <SelectItem value="number">Number</SelectItem>
                              <SelectItem value="date">Date</SelectItem>
                              <SelectItem value="select">Select</SelectItem>
                              <SelectItem value="textarea">Textarea</SelectItem>
                              <SelectItem value="checkbox">Checkbox</SelectItem>
                              <SelectItem value="file">File</SelectItem>
                            </SelectContent>
                          </Select>
                        </div>
                        <div className="flex items-center justify-between">
                          <div className="flex items-center space-x-2">
                            <Checkbox
                              checked={newField.required}
                              onCheckedChange={(c) => setNewField({ ...newField, required: !!c })}
                            />
                            <Label>Required Field</Label>
                          </div>
                          <Button onClick={handleAddField} size="sm">
                            <Plus className="w-4 h-4 mr-2" />
                            Add Field
                          </Button>
                        </div>
                      </div>
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          </TabsContent>

          {/* Bug Details */}
          <TabsContent value="bug_fix">
            <Card>
              <CardHeader>
                <CardTitle>Bug Details</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <Label>Severity</Label>
                  <Select value={bugDetails.severity} onValueChange={(v: any) => setBugDetails({ ...bugDetails, severity: v })}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="critical">Critical - System Down</SelectItem>
                      <SelectItem value="high">High - Major Feature Broken</SelectItem>
                      <SelectItem value="medium">Medium - Feature Partially Working</SelectItem>
                      <SelectItem value="low">Low - Minor Issue</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div>
                  <Label>Expected Behavior</Label>
                  <Textarea
                    placeholder="What should happen..."
                    value={bugDetails.expectedBehavior}
                    onChange={(e) => setBugDetails({ ...bugDetails, expectedBehavior: e.target.value })}
                  />
                </div>

                <div>
                  <Label>Actual Behavior</Label>
                  <Textarea
                    placeholder="What actually happens..."
                    value={bugDetails.actualBehavior}
                    onChange={(e) => setBugDetails({ ...bugDetails, actualBehavior: e.target.value })}
                  />
                </div>

                <div>
                  <Label>Environment</Label>
                  <Input
                    placeholder="e.g., Production, Chrome 120, Windows 11"
                    value={bugDetails.environment}
                    onChange={(e) => setBugDetails({ ...bugDetails, environment: e.target.value })}
                  />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* API Details */}
          <TabsContent value="api_development">
            <Card>
              <CardContent className="py-8 text-center text-muted-foreground">
                API details form coming soon...
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>

        {/* Acceptance Criteria */}
        <Card>
          <CardHeader>
            <CardTitle>Acceptance Criteria</CardTitle>
            <CardDescription>Define when this requirement is considered done</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {acceptanceCriteria.map((ac) => (
              <div key={ac.id} className="flex items-center gap-2 p-3 border rounded">
                <div className="flex-1 text-sm">{ac.description}</div>
                <Button variant="ghost" size="sm" onClick={() => handleRemoveCriterion(ac.id)}>
                  <Trash2 className="w-4 h-4 text-red-500" />
                </Button>
              </div>
            ))}

            <div className="flex gap-2">
              <Input
                placeholder="Add acceptance criterion..."
                value={newCriterion}
                onChange={(e) => setNewCriterion(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && handleAddCriterion()}
              />
              <Button onClick={handleAddCriterion}>
                <Plus className="w-4 h-4" />
              </Button>
            </div>
          </CardContent>
        </Card>

        {/* Actions */}
        <div className="flex justify-end gap-3">
          <Button variant="outline" onClick={() => navigate(getBackPath())}>
            Cancel
          </Button>
          <Button onClick={handleSave} className="bg-blue-600 hover:bg-blue-700">
            <Save className="w-4 h-4 mr-2" />
            Save Requirement
          </Button>
        </div>
      </div>
    </div>
  );
}