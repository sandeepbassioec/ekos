import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Checkbox } from '../ui/checkbox';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { InfrastructureRequirements } from '../../types/comprehensive-requirements';

interface InfrastructureStepProps {
  data?: InfrastructureRequirements;
  onChange: (data: InfrastructureRequirements) => void;
}

export function InfrastructureStep({ data, onChange }: InfrastructureStepProps) {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">☁️ Hosting & Cloud</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Hosting Type</Label>
              <Select defaultValue="cloud">
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="cloud">Cloud</SelectItem>
                  <SelectItem value="on_premise">On-Premise</SelectItem>
                  <SelectItem value="hybrid">Hybrid</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label>Cloud Provider</Label>
              <Select>
                <SelectTrigger><SelectValue placeholder="Select provider" /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="aws">AWS</SelectItem>
                  <SelectItem value="azure">Microsoft Azure</SelectItem>
                  <SelectItem value="gcp">Google Cloud</SelectItem>
                  <SelectItem value="other">Other</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Use Managed Services (PaaS/SaaS)</Label>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🗄️ Database</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Database Type</Label>
              <Select defaultValue="rdbms">
                <SelectTrigger><SelectValue /></SelectTrigger>
                <SelectContent>
                  <SelectItem value="rdbms">RDBMS (SQL)</SelectItem>
                  <SelectItem value="nosql">NoSQL</SelectItem>
                  <SelectItem value="newsql">NewSQL</SelectItem>
                  <SelectItem value="data_warehouse">Data Warehouse</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label>Database Engine</Label>
              <Input placeholder="e.g., PostgreSQL, MongoDB" />
            </div>
          </div>
          <div className="space-y-2">
            <div className="flex items-center space-x-2">
              <Checkbox />
              <Label>Replication</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox />
              <Label>Sharding</Label>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🚀 CI/CD Pipeline</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>CI/CD Tool</Label>
              <Input placeholder="e.g., Jenkins, GitHub Actions, GitLab CI" />
            </div>
            <div>
              <Label>Deployment Frequency</Label>
              <Input placeholder="e.g., Daily, Weekly" />
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">📦 Containerization & Orchestration</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Docker Containers</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Kubernetes Orchestration</Label>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">⚡ Auto-Scaling</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Enable Auto-Scaling</Label>
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Min Instances</Label>
              <Input type="number" defaultValue="2" />
            </div>
            <div>
              <Label>Max Instances</Label>
              <Input type="number" defaultValue="10" />
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
