import React, { useState } from 'react';
import { ArrowLeft, ArrowRight, Save } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../components/ui/card';
import { Input } from '../components/ui/input';
import { Label } from '../components/ui/label';
import { Textarea } from '../components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../components/ui/select';
import { Progress } from '../components/ui/progress';
import { useNavigate } from 'react-router';
import { MetaModelAPI } from '../lib/metamodel-store';
import { toast } from 'sonner';
import { ArtifactType, FieldDefinition, FieldType } from '../types/metamodel';

export default function CreateArtifactTypePage() {
  const navigate = useNavigate();
  const [currentStep, setCurrentStep] = useState(1);
  const [artifactData, setArtifactData] = useState({
    name: '',
    category: 'application' as ArtifactType['category'],
    icon: '📦',
    description: '',
    displayOrder: 100,
  });
  const [fields, setFields] = useState<Omit<FieldDefinition, 'id' | 'artifactTypeId' | 'createdAt'>[]>([]);
  const [currentField, setCurrentField] = useState<Partial<FieldDefinition>>({
    fieldType: 'text',
    isRequired: false,
    displayOrder: 1,
  });

  const steps = [
    { id: 1, title: 'Basic Information' },
    { id: 2, title: 'Field Definitions' },
    { id: 3, title: 'Complexity Rules' },
    { id: 4, title: 'Templates' },
  ];

  const progress = (currentStep / steps.length) * 100;

  const handleNext = () => {
    if (currentStep === 1 && !artifactData.name) {
      toast.error('Artifact name is required');
      return;
    }
    if (currentStep < steps.length) {
      setCurrentStep(currentStep + 1);
    }
  };

  const handlePrev = () => {
    if (currentStep > 1) setCurrentStep(currentStep - 1);
  };

  const handleSave = () => {
    const newArtifact = MetaModelAPI.createArtifactType({
      ...artifactData,
      isActive: true,
    });

    // Save fields
    fields.forEach((field) => {
      MetaModelAPI.createFieldDefinition({
        ...field,
        artifactTypeId: newArtifact.id,
      } as Omit<FieldDefinition, 'id' | 'createdAt'>);
    });

    toast.success('Artifact type created successfully!');
    navigate('/admin/metamodel');
  };

  const addField = () => {
    if (!currentField.fieldName || !currentField.fieldLabel) {
      toast.error('Field name and label are required');
      return;
    }

    setFields([
      ...fields,
      {
        fieldName: currentField.fieldName!,
        fieldLabel: currentField.fieldLabel!,
        fieldType: currentField.fieldType as FieldType,
        isRequired: currentField.isRequired || false,
        displayOrder: fields.length + 1,
        fieldOptions: currentField.fieldOptions,
        validationRules: currentField.validationRules,
        placeholder: currentField.placeholder,
        helpText: currentField.helpText,
      },
    ]);

    setCurrentField({
      fieldType: 'text',
      isRequired: false,
      displayOrder: fields.length + 2,
    });
    toast.success('Field added');
  };

  const removeField = (index: number) => {
    setFields(fields.filter((_, i) => i !== index));
  };

  return (
    <div className="container max-w-4xl mx-auto py-8 px-4">
      <div className="mb-8">
        <Button variant="ghost" onClick={() => navigate('/admin/metamodel')}>
          <ArrowLeft className="w-4 h-4 mr-2" />
          Back to Artifact Types
        </Button>
        <h1 className="text-3xl font-bold mt-4 mb-2">Create New Artifact Type</h1>
        <p className="text-muted-foreground">
          Define a new artifact type with custom fields and rules
        </p>
      </div>

      {/* Progress */}
      <div className="mb-8">
        <div className="flex justify-between mb-4">
          {steps.map((step) => (
            <div
              key={step.id}
              className={`text-sm font-medium ${
                step.id === currentStep
                  ? 'text-primary'
                  : step.id < currentStep
                  ? 'text-green-600'
                  : 'text-muted-foreground'
              }`}
            >
              Step {step.id}: {step.title}
            </div>
          ))}
        </div>
        <Progress value={progress} className="h-2" />
      </div>

      {/* Step 1: Basic Information */}
      {currentStep === 1 && (
        <Card>
          <CardHeader>
            <CardTitle>Basic Information</CardTitle>
            <CardDescription>Define the artifact type name, category, and icon</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <Label htmlFor="name">Artifact Name *</Label>
              <Input
                id="name"
                value={artifactData.name}
                onChange={(e) => setArtifactData({ ...artifactData, name: e.target.value })}
                placeholder="e.g., Background Job, RDMA Module"
              />
            </div>

            <div>
              <Label htmlFor="category">Category *</Label>
              <Select
                value={artifactData.category}
                onValueChange={(value) => setArtifactData({ ...artifactData, category: value as any })}
              >
                <SelectTrigger id="category">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="application">Application</SelectItem>
                  <SelectItem value="infrastructure">Infrastructure</SelectItem>
                  <SelectItem value="integration">Integration</SelectItem>
                  <SelectItem value="dependency">Dependency</SelectItem>
                  <SelectItem value="system">System</SelectItem>
                  <SelectItem value="computation">Computation</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div>
              <Label htmlFor="icon">Icon (Emoji)</Label>
              <Input
                id="icon"
                value={artifactData.icon}
                onChange={(e) => setArtifactData({ ...artifactData, icon: e.target.value })}
                placeholder="📦"
                maxLength={2}
              />
            </div>

            <div>
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                value={artifactData.description}
                onChange={(e) => setArtifactData({ ...artifactData, description: e.target.value })}
                placeholder="Brief description of this artifact type"
              />
            </div>
          </CardContent>
        </Card>
      )}

      {/* Step 2: Field Definitions */}
      {currentStep === 2 && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Add Field</CardTitle>
              <CardDescription>Define custom fields for this artifact type</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="fieldName">Field Name *</Label>
                  <Input
                    id="fieldName"
                    value={currentField.fieldName || ''}
                    onChange={(e) => setCurrentField({ ...currentField, fieldName: e.target.value })}
                    placeholder="e.g., schedule_type"
                  />
                </div>
                <div>
                  <Label htmlFor="fieldLabel">Field Label *</Label>
                  <Input
                    id="fieldLabel"
                    value={currentField.fieldLabel || ''}
                    onChange={(e) => setCurrentField({ ...currentField, fieldLabel: e.target.value })}
                    placeholder="e.g., Schedule Type"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label htmlFor="fieldType">Field Type *</Label>
                  <Select
                    value={currentField.fieldType}
                    onValueChange={(value) => setCurrentField({ ...currentField, fieldType: value as FieldType })}
                  >
                    <SelectTrigger id="fieldType">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="text">Text</SelectItem>
                      <SelectItem value="number">Number</SelectItem>
                      <SelectItem value="select">Select (Dropdown)</SelectItem>
                      <SelectItem value="multiselect">Multi-select</SelectItem>
                      <SelectItem value="textarea">Text Area</SelectItem>
                      <SelectItem value="boolean">Boolean (Checkbox)</SelectItem>
                      <SelectItem value="date">Date</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="flex items-center gap-4 pt-6">
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={currentField.isRequired}
                      onChange={(e) => setCurrentField({ ...currentField, isRequired: e.target.checked })}
                      className="w-4 h-4"
                    />
                    <span className="text-sm">Required</span>
                  </label>
                </div>
              </div>

              {(currentField.fieldType === 'select' || currentField.fieldType === 'multiselect') && (
                <div>
                  <Label htmlFor="options">Options (comma-separated)</Label>
                  <Input
                    id="options"
                    value={currentField.fieldOptions?.join(', ') || ''}
                    onChange={(e) =>
                      setCurrentField({
                        ...currentField,
                        fieldOptions: e.target.value.split(',').map((s) => s.trim()),
                      })
                    }
                    placeholder="e.g., Option 1, Option 2, Option 3"
                  />
                </div>
              )}

              <div>
                <Label htmlFor="placeholder">Placeholder Text</Label>
                <Input
                  id="placeholder"
                  value={currentField.placeholder || ''}
                  onChange={(e) => setCurrentField({ ...currentField, placeholder: e.target.value })}
                  placeholder="Hint text for the field"
                />
              </div>

              <Button onClick={addField}>Add Field</Button>
            </CardContent>
          </Card>

          {fields.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle>Defined Fields ({fields.length})</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {fields.map((field, index) => (
                    <div key={index} className="flex items-center justify-between p-3 border rounded">
                      <div>
                        <div className="font-semibold">{field.fieldLabel}</div>
                        <div className="text-sm text-muted-foreground">
                          {field.fieldName} • {field.fieldType}
                          {field.isRequired && ' • Required'}
                        </div>
                      </div>
                      <Button variant="destructive" size="sm" onClick={() => removeField(index)}>
                        Remove
                      </Button>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      )}

      {/* Step 3: Placeholder for Complexity Rules */}
      {currentStep === 3 && (
        <Card>
          <CardHeader>
            <CardTitle>Complexity Rules</CardTitle>
            <CardDescription>Define rules to automatically calculate complexity (Coming soon)</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground">Complexity rules configuration will be available in the next update.</p>
          </CardContent>
        </Card>
      )}

      {/* Step 4: Placeholder for Templates */}
      {currentStep === 4 && (
        <Card>
          <CardHeader>
            <CardTitle>WBS & Story Templates</CardTitle>
            <CardDescription>Configure templates for WBS and user stories (Coming soon)</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground">Template configuration will be available in the next update.</p>
          </CardContent>
        </Card>
      )}

      {/* Navigation */}
      <div className="flex justify-between mt-8">
        <Button variant="outline" onClick={handlePrev} disabled={currentStep === 1}>
          <ArrowLeft className="w-4 h-4 mr-2" />
          Previous
        </Button>
        {currentStep < steps.length ? (
          <Button onClick={handleNext}>
            Next
            <ArrowRight className="w-4 h-4 ml-2" />
          </Button>
        ) : (
          <Button onClick={handleSave}>
            <Save className="w-4 h-4 mr-2" />
            Save Artifact Type
          </Button>
        )}
      </div>
    </div>
  );
}
