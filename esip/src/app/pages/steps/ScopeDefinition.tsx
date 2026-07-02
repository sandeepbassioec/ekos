import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router';
import { useEstimationStore, Screen } from '../../store/estimationStore';
import { Button } from '../../components/ui/button';
import { Input } from '../../components/ui/input';
import { Label } from '../../components/ui/label';
import { Card } from '../../components/ui/card';
import {
  ArrowRight,
  ArrowLeft,
  Plus,
  Trash2,
  Edit,
  Save,
  X,
  Code,
  Briefcase,
  AlertCircle,
  ChevronRight,
  ChevronDown,
  Copy,
} from 'lucide-react';

type ViewMode = 'DEV' | 'BUSINESS';

interface HierarchicalScreen extends Screen {
  children: HierarchicalScreen[];
}

export function ScopeDefinition() {
  const navigate = useNavigate();
  const { screens, addScreen, removeScreen, updateScreen, setCurrentStep } = useEstimationStore();
  
  const [viewMode, setViewMode] = useState<ViewMode>('DEV');
  const [showAddForm, setShowAddForm] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    type: 'SCREEN' as 'SCREEN' | 'FEATURE' | 'API' | 'COMPONENT',
    complexity: 'MEDIUM' as 'LOW' | 'MEDIUM' | 'HIGH',
    parentId: undefined as string | undefined,
    businessPhase: 'DEVELOPMENT' as 'DISCOVERY' | 'DESIGN' | 'DEVELOPMENT' | 'TESTING' | 'DEPLOYMENT',
    businessValue: 'MEDIUM' as 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL',
  });

  // Calculate level based on parent
  const getItemLevel = (parentId?: string): number => {
    if (!parentId) return 0;
    const parent = screens.find(s => s.id === parentId);
    if (!parent) return 0;
    return (parent.level || 0) + 1;
  };

  // Build hierarchical structure (3 levels)
  const buildHierarchy = (): HierarchicalScreen[] => {
    // Level 0: Root items
    const roots = screens.filter(s => !s.parentId);
    
    return roots.map(root => {
      // Level 1: Children of root
      const children = screens.filter(s => s.parentId === root.id);
      
      return {
        ...root,
        children: children.map(child => {
          // Level 2: Grandchildren
          const grandchildren = screens.filter(s => s.parentId === child.id);
          
          return {
            ...child,
            children: grandchildren.map(gc => ({
              ...gc,
              children: [] // Max 3 levels
            }))
          };
        })
      };
    });
  };

  const hierarchicalScreens = buildHierarchy();

  const toggleExpand = (id: string) => {
    const screen = screens.find(s => s.id === id);
    if (screen) {
      updateScreen(id, { isExpanded: !screen.isExpanded });
    }
  };

  const handleAddScreen = () => {
    if (!formData.name.trim()) {
      alert('Please enter a name');
      return;
    }

    const level = getItemLevel(formData.parentId);

    const newScreen: Screen = {
      id: Date.now().toString(),
      name: formData.name,
      description: formData.description,
      type: formData.type,
      complexity: formData.complexity,
      parentId: formData.parentId,
      level,
      isExpanded: true,
      businessPhase: formData.businessPhase,
      businessValue: formData.businessValue,
    };

    addScreen(newScreen);
    
    // Reset form
    setFormData({
      name: '',
      description: '',
      type: 'SCREEN',
      complexity: 'MEDIUM',
      parentId: undefined,
      businessPhase: 'DEVELOPMENT',
      businessValue: 'MEDIUM',
    });
    setShowAddForm(false);
  };

  const handleDuplicate = (screen: Screen) => {
    const duplicate: Screen = {
      ...screen,
      id: Date.now().toString(),
      name: `${screen.name} (Copy)`,
    };
    addScreen(duplicate);
  };

  const handleEdit = (screen: Screen) => {
    setEditingId(screen.id);
    setFormData({
      name: screen.name,
      description: screen.description,
      type: screen.type,
      complexity: screen.complexity,
      parentId: screen.parentId,
      businessPhase: screen.businessPhase || 'DEVELOPMENT',
      businessValue: screen.businessValue || 'MEDIUM',
    });
    setShowAddForm(true);
  };

  const handleUpdate = () => {
    if (!formData.name.trim() || !editingId) {
      alert('Please enter a name');
      return;
    }

    updateScreen(editingId, {
      name: formData.name,
      description: formData.description,
      type: formData.type,
      complexity: formData.complexity,
      businessPhase: formData.businessPhase,
      businessValue: formData.businessValue,
    });

    // Reset form
    setFormData({
      name: '',
      description: '',
      type: 'SCREEN',
      complexity: 'MEDIUM',
      parentId: undefined,
      businessPhase: 'DEVELOPMENT',
      businessValue: 'MEDIUM',
    });
    setEditingId(null);
    setShowAddForm(false);
  };

  const handleCancelEdit = () => {
    setEditingId(null);
    setShowAddForm(false);
    setFormData({
      name: '',
      description: '',
      type: 'SCREEN',
      complexity: 'MEDIUM',
      parentId: undefined,
      businessPhase: 'DEVELOPMENT',
      businessValue: 'MEDIUM',
    });
  };

  const handleAddChild = (parentId: string) => {
    const parent = screens.find(s => s.id === parentId);
    const parentLevel = parent?.level || 0;
    
    // Max 3 levels (0, 1, 2)
    if (parentLevel >= 2) {
      alert('Maximum nesting level (3 levels) reached! Cannot add more children.');
      return;
    }

    setFormData({
      ...formData,
      parentId,
      type: 'SCREEN',
    });
    setShowAddForm(true);
  };

  const handleNext = () => {
    if (screens.length === 0) {
      alert('Please add at least one screen/feature');
      return;
    }
    setCurrentStep(4);
    navigate('/wizard/step/4');
  };

  const handleBack = () => {
    setCurrentStep(2);
    navigate('/wizard/step/2');
  };

  // Set current step when component mounts
  useEffect(() => {
    setCurrentStep(3);
  }, [setCurrentStep]);

  const getComplexityBadge = (complexity: 'LOW' | 'MEDIUM' | 'HIGH') => {
    const styles = {
      LOW: 'bg-green-100 text-green-700 border-green-300',
      MEDIUM: 'bg-yellow-100 text-yellow-700 border-yellow-300',
      HIGH: 'bg-red-100 text-red-700 border-red-300',
    };
    return styles[complexity];
  };

  const getBusinessValueBadge = (value: string) => {
    const styles = {
      LOW: 'bg-gray-100 text-gray-600 border-gray-300',
      MEDIUM: 'bg-blue-100 text-blue-700 border-blue-300',
      HIGH: 'bg-purple-100 text-purple-700 border-purple-300',
      CRITICAL: 'bg-red-100 text-red-700 border-red-300',
    };
    return styles[value as keyof typeof styles] || styles.MEDIUM;
  };

  const getTotalCount = () => {
    const level0 = screens.filter(s => (s.level || 0) === 0).length;
    const level1 = screens.filter(s => (s.level || 0) === 1).length;
    const level2 = screens.filter(s => (s.level || 0) === 2).length;
    return { level0, level1, level2, total: screens.length };
  };

  const counts = getTotalCount();

  // Render a row at any level
  const renderRow = (item: HierarchicalScreen, level: number) => {
    const hasChildren = item.children.length > 0;
    const canAddChild = level < 2; // Max 3 levels (0, 1, 2)
    
    // Indentation based on level
    const indentClass = level === 0 ? '' : level === 1 ? 'pl-8' : 'pl-16';
    const bgClass = level === 0 && hasChildren ? 'bg-blue-50' : level === 1 ? 'bg-gray-50' : 'bg-white';
    const borderClass = level === 1 ? 'border-l-4 border-blue-300' : level === 2 ? 'border-l-4 border-purple-400' : '';

    return (
      <div key={item.id}>
        {/* Row */}
        <div className={`grid gap-4 px-4 py-3 items-center hover:bg-gray-100 ${
          viewMode === 'DEV'
            ? 'grid-cols-[40px_1fr_120px_120px_200px]'
            : 'grid-cols-[40px_1fr_120px_120px_200px]'
        } ${bgClass} ${borderClass}`}>
          
          {/* Expand/Collapse */}
          <div className="flex items-center gap-1">
            {hasChildren && (
              <button
                onClick={() => toggleExpand(item.id)}
                className="p-1 hover:bg-gray-200 rounded"
              >
                {item.isExpanded ? (
                  <ChevronDown className="w-4 h-4" />
                ) : (
                  <ChevronRight className="w-4 h-4" />
                )}
              </button>
            )}
          </div>

          {/* Name with indentation */}
          <div className={`font-semibold text-gray-900 flex items-center gap-2 ${indentClass}`}>
            {level === 0 && item.type === 'FEATURE' && '📁'}
            {level === 1 && '↳ '}
            {level === 2 && '↳↳ '}
            <span className={level === 0 ? 'font-bold' : level === 1 ? 'font-semibold' : 'font-normal'}>
              {item.name}
            </span>
            {hasChildren && (
              <span className={`text-xs px-2 py-0.5 rounded-full ${
                level === 0 ? 'bg-blue-600 text-white' : 
                level === 1 ? 'bg-purple-600 text-white' : 
                'bg-gray-600 text-white'
              }`}>
                {item.children.length}
              </span>
            )}
            {level === 2 && (
              <span className="text-xs bg-purple-100 text-purple-700 px-2 py-0.5 rounded border border-purple-300">
                Level 3
              </span>
            )}
          </div>

          {/* Type */}
          <div>
            <span className="text-xs px-2 py-1 rounded bg-gray-100 border border-gray-300 text-gray-700">
              {item.type}
            </span>
          </div>

          {/* Complexity or Business Value */}
          {viewMode === 'DEV' ? (
            <div>
              <span className={`text-xs px-2 py-1 rounded border ${getComplexityBadge(item.complexity)}`}>
                {item.complexity}
              </span>
            </div>
          ) : (
            <div>
              <span className={`text-xs px-2 py-1 rounded border ${getBusinessValueBadge(item.businessValue || 'MEDIUM')}`}>
                {item.businessValue || 'MEDIUM'}
              </span>
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-1 justify-end">
            {canAddChild && (
              <button
                onClick={() => handleAddChild(item.id)}
                className="p-1.5 hover:bg-blue-100 rounded text-blue-600"
                title={`Add child (Level ${level + 2})`}
              >
                <Plus className="w-4 h-4" />
              </button>
            )}
            {!canAddChild && (
              <span className="text-xs text-gray-400 px-2" title="Max level reached">
                🔒
              </span>
            )}
            <button
              onClick={() => handleDuplicate(item)}
              className="p-1.5 hover:bg-gray-100 rounded text-gray-600"
              title="Duplicate"
            >
              <Copy className="w-4 h-4" />
            </button>
            <button
              onClick={() => handleEdit(item)}
              className="p-1.5 hover:bg-gray-100 rounded text-gray-600"
              title="Edit"
            >
              <Edit className="w-4 h-4" />
            </button>
            <button
              onClick={() => removeScreen(item.id)}
              className="p-1.5 hover:bg-red-100 rounded text-red-600"
              title="Delete"
            >
              <Trash2 className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Render children recursively */}
        {item.isExpanded && item.children.map(child => renderRow(child, level + 1))}
      </div>
    );
  };

  return (
    <div className="space-y-6">
      {/* Header with View Toggle */}
      <div className="flex items-start justify-between">
        <div>
          <h2 className="text-3xl font-bold text-gray-900 mb-2">Define Project Scope</h2>
          <p className="text-gray-600">
            {viewMode === 'DEV' 
              ? 'Technical breakdown with complexity and implementation details'
              : 'Business-focused view with phases, value, and deliverables'}
          </p>
          <p className="text-sm text-purple-600 mt-1">
            ✨ Supports 3-level nesting: Module → Screen → Component
          </p>
        </div>

        {/* View Mode Toggle */}
        <div className="flex items-center gap-2 bg-gray-100 rounded-lg p-1">
          <button
            onClick={() => setViewMode('DEV')}
            className={`flex items-center gap-2 px-4 py-2 rounded-md font-medium transition-all ${
              viewMode === 'DEV'
                ? 'bg-white text-blue-600 shadow-sm'
                : 'text-gray-600 hover:text-gray-900'
            }`}
          >
            <Code className="w-4 h-4" />
            Dev View
          </button>
          <button
            onClick={() => setViewMode('BUSINESS')}
            className={`flex items-center gap-2 px-4 py-2 rounded-md font-medium transition-all ${
              viewMode === 'BUSINESS'
                ? 'bg-white text-purple-600 shadow-sm'
                : 'text-gray-600 hover:text-gray-900'
            }`}
          >
            <Briefcase className="w-4 h-4" />
            Business View
          </button>
        </div>
      </div>

      {/* Stats Bar */}
      <div className="grid grid-cols-5 gap-4">
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-3">
          <div className="text-sm text-blue-700 font-medium">Total Items</div>
          <div className="text-2xl font-bold text-blue-900">{counts.total}</div>
        </div>
        <div className="bg-green-50 border border-green-200 rounded-lg p-3">
          <div className="text-sm text-green-700 font-medium">Level 1 (Root)</div>
          <div className="text-2xl font-bold text-green-900">{counts.level0}</div>
        </div>
        <div className="bg-purple-50 border border-purple-200 rounded-lg p-3">
          <div className="text-sm text-purple-700 font-medium">Level 2</div>
          <div className="text-2xl font-bold text-purple-900">{counts.level1}</div>
        </div>
        <div className="bg-orange-50 border border-orange-200 rounded-lg p-3">
          <div className="text-sm text-orange-700 font-medium">Level 3</div>
          <div className="text-2xl font-bold text-orange-900">{counts.level2}</div>
        </div>
        <div className="bg-gray-50 border border-gray-200 rounded-lg p-3 flex items-center justify-center">
          <Button
            onClick={() => setShowAddForm(true)}
            size="sm"
            className="bg-blue-600 hover:bg-blue-700 w-full"
          >
            <Plus className="w-4 h-4 mr-2" />
            Add New
          </Button>
        </div>
      </div>

      {/* Add/Edit Form */}
      {showAddForm && (
        <Card className="p-6 border-2 border-blue-300 bg-blue-50">
          <h3 className="font-semibold text-gray-900 mb-4">
            {editingId 
              ? '✏️ Edit Item' 
              : formData.parentId 
              ? '➕ Add Child Item' 
              : '➕ Add New Module/Feature'}
          </h3>
          
          <div className="grid md:grid-cols-2 gap-4">
            {/* Name */}
            <div className="md:col-span-2">
              <Label htmlFor="name">Name <span className="text-red-500">*</span></Label>
              <Input
                id="name"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                placeholder="e.g., User Management, Login Screen, Payment API"
                className="mt-1"
                autoFocus
              />
            </div>

            {/* Description */}
            <div className="md:col-span-2">
              <Label htmlFor="description">Description</Label>
              <textarea
                id="description"
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                placeholder="Brief description..."
                rows={2}
                className="mt-1 w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            {/* Parent (if creating child) */}
            {formData.parentId && (
              <div className="md:col-span-2 bg-yellow-50 border border-yellow-300 rounded p-3">
                <span className="text-sm font-medium text-yellow-800">
                  📁 Child of: {screens.find(s => s.id === formData.parentId)?.name}
                  {' '}(Level {getItemLevel(formData.parentId) + 1})
                </span>
                <button
                  onClick={() => setFormData({ ...formData, parentId: undefined })}
                  className="ml-3 text-sm text-blue-600 hover:underline"
                >
                  Make top-level
                </button>
              </div>
            )}

            {/* Type */}
            <div>
              <Label htmlFor="type">Type</Label>
              <select
                id="type"
                value={formData.type}
                onChange={(e) => setFormData({ ...formData, type: e.target.value as any })}
                className="mt-1 w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="FEATURE">📁 Feature Module</option>
                <option value="SCREEN">🖥️ Screen/Page</option>
                <option value="API">🔌 API Endpoint</option>
                <option value="COMPONENT">🧩 Component</option>
              </select>
            </div>

            {/* Complexity (Dev View) */}
            {viewMode === 'DEV' && (
              <div>
                <Label htmlFor="complexity">Complexity</Label>
                <select
                  id="complexity"
                  value={formData.complexity}
                  onChange={(e) => setFormData({ ...formData, complexity: e.target.value as 'LOW' | 'MEDIUM' | 'HIGH' })}
                  className="mt-1 w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="LOW">🟢 Low - Simple</option>
                  <option value="MEDIUM">🟡 Medium - Moderate</option>
                  <option value="HIGH">🔴 High - Complex</option>
                </select>
              </div>
            )}

            {/* Business Phase (Business View) */}
            {viewMode === 'BUSINESS' && (
              <>
                <div>
                  <Label htmlFor="businessPhase">Phase</Label>
                  <select
                    id="businessPhase"
                    value={formData.businessPhase}
                    onChange={(e) => setFormData({ ...formData, businessPhase: e.target.value as any })}
                    className="mt-1 w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-purple-500"
                  >
                    <option value="DISCOVERY">🔍 Discovery</option>
                    <option value="DESIGN">🎨 Design</option>
                    <option value="DEVELOPMENT">⚙️ Development</option>
                    <option value="TESTING">🧪 Testing</option>
                    <option value="DEPLOYMENT">🚀 Deployment</option>
                  </select>
                </div>

                <div>
                  <Label htmlFor="businessValue">Business Value</Label>
                  <select
                    id="businessValue"
                    value={formData.businessValue}
                    onChange={(e) => setFormData({ ...formData, businessValue: e.target.value as any })}
                    className="mt-1 w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-purple-500"
                  >
                    <option value="LOW">⬇️ Low</option>
                    <option value="MEDIUM">➡️ Medium</option>
                    <option value="HIGH">⬆️ High</option>
                    <option value="CRITICAL">🔥 Critical</option>
                  </select>
                </div>
              </>
            )}
          </div>

          {/* Actions */}
          <div className="flex gap-2 mt-4">
            {editingId ? (
              <>
                <Button onClick={handleUpdate} className="bg-blue-600 hover:bg-blue-700">
                  <Save className="w-4 h-4 mr-2" />
                  Update Item
                </Button>
                <Button variant="outline" onClick={handleCancelEdit}>
                  Cancel
                </Button>
              </>
            ) : (
              <>
                <Button onClick={handleAddScreen} className="bg-blue-600 hover:bg-blue-700">
                  <Plus className="w-4 h-4 mr-2" />
                  Add Item
                </Button>
                <Button variant="outline" onClick={() => {
                  setShowAddForm(false);
                  setFormData({
                    name: '',
                    description: '',
                    type: 'SCREEN',
                    complexity: 'MEDIUM',
                    parentId: undefined,
                    businessPhase: 'DEVELOPMENT',
                    businessValue: 'MEDIUM',
                  });
                }}>
                  Cancel
                </Button>
              </>
            )}
          </div>
        </Card>
      )}

      {/* Table View */}
      {screens.length > 0 && (
        <div className="border border-gray-300 rounded-lg overflow-hidden bg-white">
          {/* Table Header */}
          <div className={`grid gap-4 px-4 py-3 font-semibold text-sm border-b border-gray-300 ${
            viewMode === 'DEV' 
              ? 'grid-cols-[40px_1fr_120px_120px_200px]' 
              : 'grid-cols-[40px_1fr_120px_120px_200px]'
          } bg-gray-50`}>
            <div></div>
            <div>Name & Hierarchy</div>
            <div>Type</div>
            {viewMode === 'DEV' ? (
              <>
                <div>Complexity</div>
                <div className="text-right">Actions</div>
              </>
            ) : (
              <>
                <div>Business Value</div>
                <div className="text-right">Actions</div>
              </>
            )}
          </div>

          {/* Table Rows - Recursive */}
          <div className="divide-y divide-gray-200">
            {hierarchicalScreens.map(item => renderRow(item, 0))}
          </div>
        </div>
      )}

      {/* Empty State */}
      {screens.length === 0 && !showAddForm && (
        <div className="text-center py-12 bg-gray-50 rounded-lg border-2 border-dashed border-gray-300">
          <div className="text-4xl mb-3">📋</div>
          <h3 className="text-lg font-semibold text-gray-900 mb-2">No items added yet</h3>
          <p className="text-gray-600 mb-4">Start by adding your first module or screen</p>
          <Button
            onClick={() => setShowAddForm(true)}
            className="bg-blue-600 hover:bg-blue-700"
          >
            <Plus className="w-4 h-4 mr-2" />
            Add First Item
          </Button>
        </div>
      )}

      {/* Navigation */}
      <div className="flex justify-between pt-6 border-t border-gray-200">
        <Button variant="outline" onClick={handleBack}>
          <ArrowLeft className="w-4 h-4 mr-2" />
          Back
        </Button>
        <Button
          onClick={handleNext}
          disabled={screens.length === 0}
          className="bg-blue-600 hover:bg-blue-700"
        >
          Continue to Estimation ({screens.length} items)
          <ArrowRight className="w-4 h-4 ml-2" />
        </Button>
      </div>
    </div>
  );
}