import React, { useState, useEffect } from 'react';
import { FileText, Upload, Inbox, BookOpen, ChevronRight, Loader2 } from 'lucide-react';
import { Button } from '../../ui/button';
import { Label } from '../../ui/label';
import { Textarea } from '../../ui/textarea';
import { RadioGroup, RadioGroupItem } from '../../ui/radio-group';
import { Card } from '../../ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../../ui/select';
import { db } from '../../../lib/db';
import { useProjectsStore } from '../../../store/projectsStore';
import type { DesignerDocument } from '../../../lib/db/types';

interface ProjectInputProps {
  data: any;
  onChange: (data: any) => void;
}

// ── Tiptap JSON → plain text ───────────────────────────────────────────────
function tiptapToText(node: any): string {
  if (!node) return '';
  if (node.type === 'text') return node.text || '';
  if (node.type === 'hardBreak') return '\n';

  const children = (node.content || []).map(tiptapToText).join('');

  switch (node.type) {
    case 'heading':    return `\n${'#'.repeat(node.attrs?.level ?? 1)} ${children}\n`;
    case 'paragraph':  return children + '\n';
    case 'bulletList':
    case 'orderedList': return children;
    case 'listItem':   return `• ${children}\n`;
    case 'codeBlock':  return `\`\`\`\n${children}\n\`\`\`\n`;
    case 'blockquote': return `> ${children}\n`;
    case 'table':      return children + '\n';
    case 'tableRow':   return children + '\n';
    case 'tableCell':
    case 'tableHeader': return children.trim() + '\t';
    default:           return children + (children.trim() ? '\n' : '');
  }
}

function extractTextFromTiptap(contentJson: string): string {
  try {
    const doc = JSON.parse(contentJson);
    return tiptapToText(doc).replace(/\n{3,}/g, '\n\n').trim();
  } catch {
    return contentJson; // fallback: return raw string
  }
}

const CATEGORY_LABELS: Record<string, string> = {
  brd: 'BRD',
  hld: 'HLD',
  lld: 'LLD',
  architecture: 'Architecture',
  custom: 'Custom',
};

// ── Component ──────────────────────────────────────────────────────────────
export function ProjectInput({ data, onChange }: ProjectInputProps) {
  const [inputMethod, setInputMethod] = useState<'existing' | 'manual' | 'wbs' | 'designer'>('existing');
  const [existingProjects, setExistingProjects] = useState<any[]>([]);
  const [selectedProject, setSelectedProject] = useState('');
  const [requirements, setRequirements] = useState(data?.requirements || '');

  // Designer-document mode
  const { projects, fetchProjects } = useProjectsStore();
  const [designerProjectId, setDesignerProjectId] = useState('');
  const [designerDocs, setDesignerDocs] = useState<DesignerDocument[]>([]);
  const [loadingDocs, setLoadingDocs] = useState(false);
  const [selectedDocId, setSelectedDocId] = useState('');

  useEffect(() => {
    // Legacy: Load estimation wizard projects from localStorage
    const stored = JSON.parse(localStorage.getItem('projects') || '[]');
    setExistingProjects(stored);
  }, []);

  useEffect(() => {
    fetchProjects();
  }, [fetchProjects]);

  // Load designer documents when a project is selected in designer mode
  useEffect(() => {
    if (!designerProjectId) { setDesignerDocs([]); return; }
    setLoadingDocs(true);
    db.designerDocuments.getByProject(designerProjectId)
      .then(docs => setDesignerDocs(docs))
      .catch(() => setDesignerDocs([]))
      .finally(() => setLoadingDocs(false));
  }, [designerProjectId]);

  const handleInputMethodChange = (method: typeof inputMethod) => {
    setInputMethod(method);
    onChange({ ...data, inputMethod: method });
  };

  const handleProjectSelect = (projectId: string) => {
    setSelectedProject(projectId);
    const project = existingProjects.find(p => p.id === projectId);
    if (project) {
      const wbsText = extractWBSAsText(project);
      setRequirements(wbsText);
      onChange({ ...data, projectId, projectName: project.name, requirements: wbsText, wbsData: project.wbs });
    }
  };

  const handleDesignerDocSelect = (docId: string) => {
    setSelectedDocId(docId);
    const doc = designerDocs.find(d => d.id === docId);
    if (doc) {
      const text = extractTextFromTiptap(doc.content);
      setRequirements(text);
      onChange({
        ...data,
        projectId: designerProjectId,
        projectName: doc.title,
        requirements: text,
        sourceDocId: docId,
        sourceDocTitle: doc.title,
        sourceDocCategory: doc.category,
      });
    }
  };

  const extractWBSAsText = (project: any) => {
    let text = `Project: ${project.name}\n\nDescription: ${project.description || 'N/A'}\n\nFeatures & Modules:\n\n`;
    if (project.modules) {
      project.modules.forEach((module: any) => {
        text += `Module: ${module.name}\nDescription: ${module.description || ''}\n`;
        if (module.screens) {
          text += `Screens:\n`;
          module.screens.forEach((screen: any) => {
            text += `  - ${screen.name}: ${screen.description || ''}\n`;
            if (screen.components) {
              screen.components.forEach((comp: any) => { text += `    • ${comp.name}\n`; });
            }
          });
        }
        text += `\n`;
      });
    }
    return text;
  };

  const handleRequirementsChange = (value: string) => {
    setRequirements(value);
    onChange({ ...data, requirements: value, inputMethod });
  };

  return (
    <div className="space-y-6">
      {/* Input Method Selection */}
      <div>
        <Label className="text-base mb-3 block">Select Input Method</Label>
        <RadioGroup value={inputMethod} onValueChange={handleInputMethodChange as any}>
          <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
            {/* Option 1 — Estimation Wizard */}
            <Card
              className={`p-4 cursor-pointer transition-all ${inputMethod === 'existing' ? 'border-primary border-2 bg-primary/5' : 'hover:border-primary/50'}`}
              onClick={() => handleInputMethodChange('existing')}
            >
              <div className="flex items-center space-x-2">
                <RadioGroupItem value="existing" id="existing" />
                <Label htmlFor="existing" className="cursor-pointer flex-1">
                  <div className="flex items-center gap-2">
                    <Inbox className="w-5 h-5" />
                    <div>
                      <p className="font-semibold">Existing Project</p>
                      <p className="text-xs text-muted-foreground">Import from Estimation Wizard</p>
                    </div>
                  </div>
                </Label>
              </div>
            </Card>

            {/* Option 2 — Designer Document (NEW) */}
            <Card
              className={`p-4 cursor-pointer transition-all ${inputMethod === 'designer' ? 'border-primary border-2 bg-primary/5' : 'hover:border-primary/50'}`}
              onClick={() => handleInputMethodChange('designer')}
            >
              <div className="flex items-center space-x-2">
                <RadioGroupItem value="designer" id="designer" />
                <Label htmlFor="designer" className="cursor-pointer flex-1">
                  <div className="flex items-center gap-2">
                    <BookOpen className="w-5 h-5 text-primary" />
                    <div>
                      <p className="font-semibold">Designer Document</p>
                      <p className="text-xs text-muted-foreground">Use BRD / HLD from Knowledge Hub</p>
                    </div>
                  </div>
                </Label>
              </div>
            </Card>

            {/* Option 3 — Upload WBS */}
            <Card
              className={`p-4 cursor-pointer transition-all ${inputMethod === 'wbs' ? 'border-primary border-2 bg-primary/5' : 'hover:border-primary/50'}`}
              onClick={() => handleInputMethodChange('wbs')}
            >
              <div className="flex items-center space-x-2">
                <RadioGroupItem value="wbs" id="wbs" />
                <Label htmlFor="wbs" className="cursor-pointer flex-1">
                  <div className="flex items-center gap-2">
                    <FileText className="w-5 h-5" />
                    <div>
                      <p className="font-semibold">Upload WBS</p>
                      <p className="text-xs text-muted-foreground">Import Excel/CSV WBS</p>
                    </div>
                  </div>
                </Label>
              </div>
            </Card>

            {/* Option 4 — Manual */}
            <Card
              className={`p-4 cursor-pointer transition-all ${inputMethod === 'manual' ? 'border-primary border-2 bg-primary/5' : 'hover:border-primary/50'}`}
              onClick={() => handleInputMethodChange('manual')}
            >
              <div className="flex items-center space-x-2">
                <RadioGroupItem value="manual" id="manual" />
                <Label htmlFor="manual" className="cursor-pointer flex-1">
                  <div className="flex items-center gap-2">
                    <Upload className="w-5 h-5" />
                    <div>
                      <p className="font-semibold">Manual Input</p>
                      <p className="text-xs text-muted-foreground">Type or paste requirements</p>
                    </div>
                  </div>
                </Label>
              </div>
            </Card>
          </div>
        </RadioGroup>
      </div>

      {/* Conditional: Estimation Wizard projects */}
      {inputMethod === 'existing' && (
        <div>
          <Label htmlFor="projectSelect">Select Project</Label>
          <Select value={selectedProject} onValueChange={handleProjectSelect}>
            <SelectTrigger id="projectSelect">
              <SelectValue placeholder="Choose a project from estimation wizard" />
            </SelectTrigger>
            <SelectContent>
              {existingProjects.length === 0 ? (
                <SelectItem value="none" disabled>No projects found</SelectItem>
              ) : (
                existingProjects.map((project) => (
                  <SelectItem key={project.id} value={project.id}>
                    {project.name} - {project.method || 'N/A'}
                  </SelectItem>
                ))
              )}
            </SelectContent>
          </Select>
          {existingProjects.length === 0 && (
            <p className="text-sm text-muted-foreground mt-2">
              💡 Tip: Create a project in the Estimation Wizard first
            </p>
          )}
        </div>
      )}

      {/* Conditional: Designer Documents */}
      {inputMethod === 'designer' && (
        <div className="space-y-3">
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <span className="font-medium text-foreground">Step 1</span>
            <ChevronRight className="size-3" />
            <span>Select a project</span>
            <ChevronRight className="size-3" />
            <span>Step 2</span>
            <ChevronRight className="size-3" />
            <span>Pick a document</span>
          </div>

          <div className="grid grid-cols-2 gap-3">
            {/* Project picker */}
            <div>
              <Label htmlFor="designerProject">Project</Label>
              <Select value={designerProjectId} onValueChange={setDesignerProjectId}>
                <SelectTrigger id="designerProject">
                  <SelectValue placeholder="Select project…" />
                </SelectTrigger>
                <SelectContent>
                  {projects.length === 0 ? (
                    <SelectItem value="none" disabled>No projects yet</SelectItem>
                  ) : (
                    projects.map(p => (
                      <SelectItem key={p.id} value={p.id}>{p.name}</SelectItem>
                    ))
                  )}
                </SelectContent>
              </Select>
            </div>

            {/* Document picker */}
            <div>
              <Label htmlFor="designerDoc">Document</Label>
              <Select
                value={selectedDocId}
                onValueChange={handleDesignerDocSelect}
                disabled={!designerProjectId || loadingDocs}
              >
                <SelectTrigger id="designerDoc">
                  {loadingDocs
                    ? <span className="flex items-center gap-1.5"><Loader2 className="size-3 animate-spin" /> Loading…</span>
                    : <SelectValue placeholder={designerProjectId ? 'Pick a document…' : 'Select project first'} />}
                </SelectTrigger>
                <SelectContent>
                  {designerDocs.length === 0 ? (
                    <SelectItem value="none" disabled>No designer documents found</SelectItem>
                  ) : (
                    designerDocs.map(doc => (
                      <SelectItem key={doc.id} value={doc.id}>
                        <span className="flex items-center gap-2">
                          <span className="text-[10px] font-bold uppercase text-muted-foreground">
                            {CATEGORY_LABELS[doc.category] ?? doc.category}
                          </span>
                          {doc.title}
                        </span>
                      </SelectItem>
                    ))
                  )}
                </SelectContent>
              </Select>
            </div>
          </div>

          {selectedDocId && (
            <p className="text-xs text-green-700 dark:text-green-400 font-medium">
              ✓ Document content extracted and loaded into the requirements field below. You can edit it before continuing.
            </p>
          )}
        </div>
      )}

      {/* Conditional: Upload WBS */}
      {inputMethod === 'wbs' && (
        <div className="border-2 border-dashed rounded-lg p-8 text-center">
          <Upload className="w-12 h-12 mx-auto mb-4 text-muted-foreground" />
          <h3 className="font-semibold mb-2">Upload WBS File</h3>
          <p className="text-sm text-muted-foreground mb-4">Drag and drop your Excel or CSV file here</p>
          <Button variant="outline">
            <Upload className="w-4 h-4 mr-2" />
            Choose File
          </Button>
          <p className="text-xs text-muted-foreground mt-4">Supported formats: .xlsx, .xls, .csv</p>
        </div>
      )}

      {/* Requirements textarea (all methods) */}
      <div>
        <Label htmlFor="requirements">
          {inputMethod === 'existing' ? 'Extracted Requirements (Editable)'
          : inputMethod === 'designer' ? 'Document Content (Editable)'
          : 'Requirements Document'}
        </Label>
        <Textarea
          id="requirements"
          value={requirements}
          onChange={(e) => handleRequirementsChange(e.target.value)}
          placeholder={`Enter or paste your requirements here. Example:

Project: E-Commerce Platform

Modules:
1. User Management
   - User Registration (email, password, profile)
   - User Login (email/password, OAuth)
   - Profile Management

2. Product Catalog
   - Product Listing (search, filter, sort)
   - Product Details (images, description, price)

3. Checkout & Payment
   - Shipping Address
   - Payment Gateway Integration
   - Order Confirmation`}
          className="min-h-[300px] font-mono text-sm"
        />
        <p className="text-xs text-muted-foreground mt-2">
          💡 Be specific about features, user types, and functionality. This will help generate accurate architecture.
        </p>
      </div>

      {/* Tips */}
      <Card className="bg-blue-50 border-blue-200 dark:bg-blue-950/30 dark:border-blue-800">
        <div className="p-4">
          <h4 className="font-semibold text-blue-900 dark:text-blue-300 mb-2">📝 Tips for Better Results:</h4>
          <ul className="text-sm text-blue-800 dark:text-blue-400 space-y-1">
            <li>✓ List all modules and their features</li>
            <li>✓ Mention user types (Admin, User, Guest, etc.)</li>
            <li>✓ Include database entities (User, Product, Order, etc.)</li>
            <li>✓ Describe key workflows and integrations</li>
            <li>✓ Mention third-party APIs if any (Payment, Email, SMS)</li>
          </ul>
        </div>
      </Card>
    </div>
  );
}
