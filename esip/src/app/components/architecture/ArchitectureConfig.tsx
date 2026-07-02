import React, { useState } from 'react';
import { useNavigate } from 'react-router';
import { Save, ArrowRight } from 'lucide-react';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Checkbox } from '../ui/checkbox';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';

export interface ArchitectureConfig {
  projectId: string;
  projectName: string;
  
  // Technology Stack
  techStack: {
    frontend: string[];
    backend: string[];
    database: string[];
    infrastructure: string[];
    integration: string[];
  };
  
  // Requirements
  requirements: {
    systemType: string;
    expectedUsers: string;
    securityLevel: string;
    compliance: string[];
    deploymentModel: string;
  };
  
  // Integration Points
  integrations: {
    thirdPartyApis: string;
    legacySystems: string;
  };
}

const techOptions = {
  frontend: ['React', 'Angular', 'Vue.js', 'Next.js', 'Svelte', 'React Native', 'Flutter'],
  backend: ['Node.js', 'Java Spring Boot', '.NET Core', 'Python Django', 'Python FastAPI', 'Go', 'Ruby on Rails'],
  database: ['PostgreSQL', 'MySQL', 'MongoDB', 'Redis', 'Cassandra', 'Oracle', 'SQL Server'],
  infrastructure: ['AWS', 'Azure', 'Google Cloud', 'Docker', 'Kubernetes', 'Terraform'],
  integration: ['REST API', 'GraphQL', 'gRPC', 'WebSocket', 'Message Queue', 'Event-Driven'],
};

const complianceOptions = ['GDPR', 'HIPAA', 'SOC2', 'ISO 27001', 'PCI DSS', 'CCPA'];

interface ArchitectureConfigFormProps {
  onSave: (config: ArchitectureConfig) => void;
  initialConfig?: Partial<ArchitectureConfig>;
}

export function ArchitectureConfigForm({ onSave, initialConfig }: ArchitectureConfigFormProps) {
  const navigate = useNavigate();
  
  const [config, setConfig] = useState<ArchitectureConfig>({
    projectId: initialConfig?.projectId || '',
    projectName: initialConfig?.projectName || '',
    techStack: {
      frontend: initialConfig?.techStack?.frontend || [],
      backend: initialConfig?.techStack?.backend || [],
      database: initialConfig?.techStack?.database || [],
      infrastructure: initialConfig?.techStack?.infrastructure || [],
      integration: initialConfig?.techStack?.integration || [],
    },
    requirements: {
      systemType: initialConfig?.requirements?.systemType || '',
      expectedUsers: initialConfig?.requirements?.expectedUsers || '',
      securityLevel: initialConfig?.requirements?.securityLevel || '',
      compliance: initialConfig?.requirements?.compliance || [],
      deploymentModel: initialConfig?.requirements?.deploymentModel || '',
    },
    integrations: {
      thirdPartyApis: initialConfig?.integrations?.thirdPartyApis || '',
      legacySystems: initialConfig?.integrations?.legacySystems || '',
    },
  });

  const handleTechStackChange = (category: keyof typeof config.techStack, value: string) => {
    setConfig(prev => ({
      ...prev,
      techStack: {
        ...prev.techStack,
        [category]: prev.techStack[category].includes(value)
          ? prev.techStack[category].filter(item => item !== value)
          : [...prev.techStack[category], value],
      },
    }));
  };

  const handleComplianceChange = (value: string) => {
    setConfig(prev => ({
      ...prev,
      requirements: {
        ...prev.requirements,
        compliance: prev.requirements.compliance.includes(value)
          ? prev.requirements.compliance.filter(item => item !== value)
          : [...prev.requirements.compliance, value],
      },
    }));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    // Save to localStorage
    localStorage.setItem('architecture-config', JSON.stringify(config));
    
    // Call parent callback
    onSave(config);
    
    // Navigate to document generation
    navigate('/architecture/generate');
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      {/* Project Info */}
      <Card>
        <CardHeader>
          <CardTitle>Project Information</CardTitle>
          <CardDescription>Basic project details</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label htmlFor="projectName">Project Name</Label>
            <Input
              id="projectName"
              value={config.projectName}
              onChange={(e) => setConfig(prev => ({ ...prev, projectName: e.target.value }))}
              placeholder="Enter project name"
              required
            />
          </div>
        </CardContent>
      </Card>

      {/* Technology Stack */}
      <Card>
        <CardHeader>
          <CardTitle>Technology Stack</CardTitle>
          <CardDescription>Select the technologies for your project</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {Object.entries(techOptions).map(([category, options]) => (
            <div key={category}>
              <Label className="text-base capitalize mb-3 block">{category}</Label>
              <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
                {options.map((option) => (
                  <div key={option} className="flex items-center space-x-2">
                    <Checkbox
                      id={`${category}-${option}`}
                      checked={config.techStack[category as keyof typeof config.techStack].includes(option)}
                      onCheckedChange={() => handleTechStackChange(category as keyof typeof config.techStack, option)}
                    />
                    <label
                      htmlFor={`${category}-${option}`}
                      className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 cursor-pointer"
                    >
                      {option}
                    </label>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </CardContent>
      </Card>

      {/* System Requirements */}
      <Card>
        <CardHeader>
          <CardTitle>System Requirements</CardTitle>
          <CardDescription>Define system characteristics and constraints</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label htmlFor="systemType">System Type</Label>
            <Select
              value={config.requirements.systemType}
              onValueChange={(value) => setConfig(prev => ({
                ...prev,
                requirements: { ...prev.requirements, systemType: value }
              }))}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select system type" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="web">Web Application</SelectItem>
                <SelectItem value="mobile">Mobile Application</SelectItem>
                <SelectItem value="desktop">Desktop Application</SelectItem>
                <SelectItem value="hybrid">Hybrid Application</SelectItem>
                <SelectItem value="enterprise">Enterprise System</SelectItem>
                <SelectItem value="microservices">Microservices Architecture</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div>
            <Label htmlFor="expectedUsers">Expected Users (Concurrent)</Label>
            <Select
              value={config.requirements.expectedUsers}
              onValueChange={(value) => setConfig(prev => ({
                ...prev,
                requirements: { ...prev.requirements, expectedUsers: value }
              }))}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select user range" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="0-100">0-100 users</SelectItem>
                <SelectItem value="100-1000">100-1,000 users</SelectItem>
                <SelectItem value="1000-10000">1,000-10,000 users</SelectItem>
                <SelectItem value="10000-100000">10,000-100,000 users</SelectItem>
                <SelectItem value="100000+">100,000+ users</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div>
            <Label htmlFor="securityLevel">Security Level</Label>
            <Select
              value={config.requirements.securityLevel}
              onValueChange={(value) => setConfig(prev => ({
                ...prev,
                requirements: { ...prev.requirements, securityLevel: value }
              }))}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select security level" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="low">Low (Public content)</SelectItem>
                <SelectItem value="medium">Medium (User authentication)</SelectItem>
                <SelectItem value="high">High (Sensitive data)</SelectItem>
                <SelectItem value="critical">Critical (Financial/Healthcare)</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div>
            <Label htmlFor="deploymentModel">Deployment Model</Label>
            <Select
              value={config.requirements.deploymentModel}
              onValueChange={(value) => setConfig(prev => ({
                ...prev,
                requirements: { ...prev.requirements, deploymentModel: value }
              }))}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select deployment model" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="cloud">Cloud (Public)</SelectItem>
                <SelectItem value="private-cloud">Private Cloud</SelectItem>
                <SelectItem value="on-premise">On-Premise</SelectItem>
                <SelectItem value="hybrid">Hybrid (Cloud + On-Premise)</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div>
            <Label className="text-base mb-3 block">Compliance Requirements</Label>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
              {complianceOptions.map((option) => (
                <div key={option} className="flex items-center space-x-2">
                  <Checkbox
                    id={`compliance-${option}`}
                    checked={config.requirements.compliance.includes(option)}
                    onCheckedChange={() => handleComplianceChange(option)}
                  />
                  <label
                    htmlFor={`compliance-${option}`}
                    className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 cursor-pointer"
                  >
                    {option}
                  </label>
                </div>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Integration Points */}
      <Card>
        <CardHeader>
          <CardTitle>Integration Points</CardTitle>
          <CardDescription>External systems and APIs</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label htmlFor="thirdPartyApis">Third-party APIs</Label>
            <Input
              id="thirdPartyApis"
              value={config.integrations.thirdPartyApis}
              onChange={(e) => setConfig(prev => ({
                ...prev,
                integrations: { ...prev.integrations, thirdPartyApis: e.target.value }
              }))}
              placeholder="e.g., Stripe, Twilio, SendGrid (comma-separated)"
            />
          </div>

          <div>
            <Label htmlFor="legacySystems">Legacy Systems</Label>
            <Input
              id="legacySystems"
              value={config.integrations.legacySystems}
              onChange={(e) => setConfig(prev => ({
                ...prev,
                integrations: { ...prev.integrations, legacySystems: e.target.value }
              }))}
              placeholder="e.g., SAP, Oracle ERP, Mainframe (comma-separated)"
            />
          </div>
        </CardContent>
      </Card>

      {/* Action Buttons */}
      <div className="flex justify-end gap-3">
        <Button type="button" variant="outline" onClick={() => navigate(-1)}>
          Cancel
        </Button>
        <Button type="submit">
          <Save className="w-4 h-4 mr-2" />
          Save & Continue
          <ArrowRight className="w-4 h-4 ml-2" />
        </Button>
      </div>
    </form>
  );
}
