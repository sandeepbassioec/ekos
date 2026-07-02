import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Checkbox } from '../ui/checkbox';
import { TestingRequirements } from '../../types/comprehensive-requirements';

export function TestingStep({ data, onChange }: { data?: TestingRequirements; onChange: (data: TestingRequirements) => void }) {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🎯 Test Coverage Targets</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-3 gap-4">
            <div>
              <Label>Unit Test Coverage (%)</Label>
              <Input type="number" defaultValue="80" />
            </div>
            <div>
              <Label>Integration Test Coverage (%)</Label>
              <Input type="number" defaultValue="70" />
            </div>
            <div>
              <Label>E2E Test Coverage (%)</Label>
              <Input type="number" defaultValue="60" />
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🤖 Automated Testing</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Testing Framework</Label>
            <Input placeholder="e.g., Jest, Pytest, JUnit" />
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox defaultChecked />
            <Label>CI/CD Integration</Label>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">⚡ Performance Testing</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Load Testing</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Stress Testing</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Endurance Testing</Label>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">✅ UAT (User Acceptance Testing)</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center space-x-2">
            <Checkbox defaultChecked />
            <Label>UAT Required</Label>
          </div>
          <div>
            <Label>UAT Participants</Label>
            <Input placeholder="e.g., Product Owner, Key Users" />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
