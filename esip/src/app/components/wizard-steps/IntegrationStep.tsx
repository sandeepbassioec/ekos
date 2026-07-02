import React, { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Button } from '../ui/button';
import { Plus, Trash2 } from 'lucide-react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { Textarea } from '../ui/textarea';
import { IntegrationRequirements, IntegrationRequirement } from '../../types/comprehensive-requirements';

interface IntegrationStepProps {
  data?: IntegrationRequirements;
  onChange: (data: IntegrationRequirements) => void;
}

export function IntegrationStep({ data, onChange }: IntegrationStepProps) {
  const defaultData: IntegrationRequirements = data || {
    externalAPIs: [],
    thirdPartyServices: [],
    legacySystems: [],
    ssoProviders: [],
    messagingServices: [],
  };

  const [newAPI, setNewAPI] = useState<Partial<IntegrationRequirement>>({
    name: '',
    type: 'rest_api',
    direction: 'outbound',
    description: '',
  });

  const addAPI = () => {
    if (newAPI.name && newAPI.description) {
      onChange({
        ...defaultData,
        externalAPIs: [
          ...defaultData.externalAPIs,
          {
            id: `api-${Date.now()}`,
            name: newAPI.name,
            type: newAPI.type as any,
            direction: newAPI.direction as any,
            description: newAPI.description,
            endpoint: '',
            authentication: '',
            dataFormat: 'json',
            frequency: 'real_time',
            errorHandling: '',
            retryPolicy: '',
          },
        ],
      });
      setNewAPI({ name: '', type: 'rest_api', direction: 'outbound', description: '' });
    }
  };

  const removeAPI = (id: string) => {
    onChange({
      ...defaultData,
      externalAPIs: defaultData.externalAPIs.filter((api) => api.id !== id),
    });
  };

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🔌 External APIs</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {defaultData.externalAPIs.map((api) => (
            <Card key={api.id} className="border-l-4 border-l-blue-500">
              <CardContent className="pt-4">
                <div className="flex items-start justify-between mb-3">
                  <div>
                    <h4 className="font-semibold">{api.name}</h4>
                    <p className="text-sm text-muted-foreground">{api.description}</p>
                  </div>
                  <Button variant="ghost" size="sm" onClick={() => removeAPI(api.id)}>
                    <Trash2 className="w-4 h-4 text-red-500" />
                  </Button>
                </div>
                <div className="grid grid-cols-3 gap-2 text-xs">
                  <span className="px-2 py-1 bg-blue-100 rounded">Type: {api.type}</span>
                  <span className="px-2 py-1 bg-green-100 rounded">Direction: {api.direction}</span>
                  <span className="px-2 py-1 bg-purple-100 rounded">Format: {api.dataFormat}</span>
                </div>
              </CardContent>
            </Card>
          ))}

          <div className="border rounded p-4 space-y-3">
            <h5 className="font-semibold text-sm">Add New API Integration</h5>
            <div className="grid grid-cols-2 gap-3">
              <Input
                placeholder="API Name"
                value={newAPI.name}
                onChange={(e) => setNewAPI({ ...newAPI, name: e.target.value })}
              />
              <Select
                value={newAPI.type}
                onValueChange={(v) => setNewAPI({ ...newAPI, type: v as any })}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="rest_api">REST API</SelectItem>
                  <SelectItem value="soap">SOAP</SelectItem>
                  <SelectItem value="graphql">GraphQL</SelectItem>
                  <SelectItem value="webhook">Webhook</SelectItem>
                  <SelectItem value="file_based">File Based</SelectItem>
                  <SelectItem value="database">Database</SelectItem>
                  <SelectItem value="message_queue">Message Queue</SelectItem>
                  <SelectItem value="other">Other</SelectItem>
                </SelectContent>
              </Select>
              <Select
                value={newAPI.direction}
                onValueChange={(v) => setNewAPI({ ...newAPI, direction: v as any })}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="inbound">Inbound</SelectItem>
                  <SelectItem value="outbound">Outbound</SelectItem>
                  <SelectItem value="bidirectional">Bidirectional</SelectItem>
                </SelectContent>
              </Select>
              <Input
                placeholder="Description"
                value={newAPI.description}
                onChange={(e) => setNewAPI({ ...newAPI, description: e.target.value })}
              />
            </div>
            <Button onClick={addAPI} size="sm" className="w-full">
              <Plus className="w-4 h-4 mr-2" />
              Add API
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🛠️ Third-Party Services</CardTitle>
        </CardHeader>
        <CardContent>
          <Textarea
            placeholder="List third-party services (Payment Gateway, Email Provider, SMS, etc.)"
            rows={4}
          />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🔗 SSO Providers</CardTitle>
        </CardHeader>
        <CardContent>
          <Input placeholder="e.g., Okta, Auth0, Azure AD, Google Workspace" />
        </CardContent>
      </Card>
    </div>
  );
}
