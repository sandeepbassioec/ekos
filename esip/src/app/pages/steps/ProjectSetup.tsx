import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router';
import { useEstimationStore, ProjectType } from '../../store/estimationStore';
import { Button } from '../../components/ui/button';
import { Input } from '../../components/ui/input';
import { Label } from '../../components/ui/label';
import { ArrowRight, ArrowLeft } from 'lucide-react';

export function ProjectSetup() {
  const navigate = useNavigate();
  const { project, setProject, setCurrentStep } = useEstimationStore();
  
  const [formData, setFormData] = useState({
    name: project.name || '',
    type: project.type || 'WEB' as ProjectType,
    teamSize: project.teamSize || 2,
    hourlyRate: project.hourlyRate || 150,
  });

  // Set current step when component mounts
  useEffect(() => {
    setCurrentStep(2);
  }, [setCurrentStep]);

  const [errors, setErrors] = useState<Record<string, string>>({});

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    // Validation
    const newErrors: Record<string, string> = {};
    if (!formData.name) newErrors.name = 'Project name is required';
    if (formData.teamSize < 1) newErrors.teamSize = 'Minimum 1 team member';
    if (formData.teamSize > 50) newErrors.teamSize = 'Maximum 50 team members';
    if (formData.hourlyRate < 50) newErrors.hourlyRate = 'Minimum $50/hr';
    if (formData.hourlyRate > 500) newErrors.hourlyRate = 'Maximum $500/hr';

    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      return;
    }

    setProject({
      ...project,
      name: formData.name,
      type: formData.type,
      teamSize: formData.teamSize,
      hourlyRate: formData.hourlyRate,
      techStack: [],
    });
    setCurrentStep(3);
    navigate('/wizard/step/3');
  };

  const handleBack = () => {
    setCurrentStep(1);
    navigate('/wizard/step/1', { state: { fromBack: true } });
  };

  const burnRate = (formData.teamSize * formData.hourlyRate * 40).toLocaleString();

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-3xl font-bold text-gray-900 mb-2">Project Setup</h2>
        <p className="text-gray-600">
          Provide basic information about your project
        </p>
      </div>

      {/* Form Fields */}
      <div className="space-y-6">
        {/* Project Name */}
        <div>
          <Label htmlFor="name" className="text-base font-semibold">
            Project Name <span className="text-red-500">*</span>
          </Label>
          <Input
            id="name"
            value={formData.name}
            onChange={(e) => {
              setFormData({ ...formData, name: e.target.value });
              setErrors({ ...errors, name: '' });
            }}
            placeholder="e.g., RepairLink Dashboard, Mobile App v2.0"
            className="mt-2"
          />
          {errors.name && (
            <p className="text-sm text-red-600 mt-1">{errors.name}</p>
          )}
        </div>

        {/* Project Type */}
        <div>
          <Label htmlFor="type" className="text-base font-semibold">
            Project Type <span className="text-red-500">*</span>
          </Label>
          <select
            id="type"
            value={formData.type}
            onChange={(e) => setFormData({ ...formData, type: e.target.value as ProjectType })}
            className="mt-2 w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="WEB">Web Application</option>
            <option value="MOBILE">Mobile App (iOS/Android)</option>
            <option value="DESKTOP">Desktop Application</option>
            <option value="API">Backend API Only</option>
            <option value="FULLSTACK">Full Stack (Frontend + Backend)</option>
          </select>
          <p className="text-xs text-gray-500 mt-1">
            {formData.type === 'WEB' && '💻 React, Angular, Vue, etc.'}
            {formData.type === 'MOBILE' && '📱 React Native, Flutter, native iOS/Android'}
            {formData.type === 'DESKTOP' && '🖥️ Electron, .NET, Java Swing, etc.'}
            {formData.type === 'API' && '🔌 REST API, GraphQL, microservices'}
            {formData.type === 'FULLSTACK' && '🌐 Complete web solution with database'}
          </p>
        </div>

        <div className="grid md:grid-cols-2 gap-6">
          {/* Team Size */}
          <div>
            <Label htmlFor="teamSize" className="text-base font-semibold">
              Team Size <span className="text-red-500">*</span>
            </Label>
            <Input
              id="teamSize"
              type="number"
              min="1"
              max="50"
              value={formData.teamSize}
              onChange={(e) => {
                setFormData({ ...formData, teamSize: parseInt(e.target.value) || 0 });
                setErrors({ ...errors, teamSize: '' });
              }}
              className="mt-2"
            />
            <p className="text-xs text-gray-500 mt-1">
              Number of developers working on this project
            </p>
            {errors.teamSize && (
              <p className="text-sm text-red-600 mt-1">{errors.teamSize}</p>
            )}
          </div>

          {/* Hourly Rate */}
          <div>
            <Label htmlFor="hourlyRate" className="text-base font-semibold">
              Hourly Rate (USD) <span className="text-red-500">*</span>
            </Label>
            <Input
              id="hourlyRate"
              type="number"
              min="50"
              max="500"
              step="10"
              value={formData.hourlyRate}
              onChange={(e) => {
                setFormData({ ...formData, hourlyRate: parseInt(e.target.value) || 0 });
                setErrors({ ...errors, hourlyRate: '' });
              }}
              className="mt-2"
            />
            <p className="text-xs text-gray-500 mt-1">
              Blended rate for the team ($/hour)
            </p>
            {errors.hourlyRate && (
              <p className="text-sm text-red-600 mt-1">{errors.hourlyRate}</p>
            )}
          </div>
        </div>

        {/* Preview Box */}
        <div className="bg-gradient-to-r from-blue-50 to-indigo-50 border border-blue-200 rounded-lg p-4">
          <h3 className="font-semibold text-blue-900 mb-3">📊 Quick Preview</h3>
          <div className="grid md:grid-cols-3 gap-4 text-sm">
            <div>
              <span className="text-blue-700 font-medium">Project:</span>
              <p className="text-gray-900 font-semibold">
                {formData.name || 'Not set'}
              </p>
            </div>
            <div>
              <span className="text-blue-700 font-medium">Type:</span>
              <p className="text-gray-900 font-semibold">
                {formData.type?.replace('_', ' ') || 'Not set'}
              </p>
            </div>
            <div>
              <span className="text-blue-700 font-medium">Estimated Burn Rate:</span>
              <p className="text-gray-900 font-semibold">
                ${burnRate}/week
              </p>
              <p className="text-xs text-gray-600">
                Based on 40 hrs/week per person
              </p>
            </div>
          </div>
        </div>

        {/* Tips */}
        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
          <h3 className="font-semibold text-yellow-900 mb-2">💡 Tips for Accurate Estimates</h3>
          <ul className="text-sm text-yellow-800 space-y-1">
            <li>• <strong>Team Size:</strong> Count only developers actively coding (not PMs or designers)</li>
            <li>• <strong>Hourly Rate:</strong> Use blended rate if team has mixed seniority</li>
            <li>• <strong>Full Stack:</strong> Consider separate frontend/backend rates if significantly different</li>
          </ul>
        </div>
      </div>

      {/* Navigation */}
      <div className="flex justify-between pt-6 border-t border-gray-200">
        <Button type="button" variant="outline" onClick={handleBack}>
          <ArrowLeft className="w-4 h-4 mr-2" />
          Back
        </Button>
        <Button type="submit" className="bg-blue-600 hover:bg-blue-700">
          Continue to Scope Definition
          <ArrowRight className="w-4 h-4 ml-2" />
        </Button>
      </div>
    </form>
  );
}