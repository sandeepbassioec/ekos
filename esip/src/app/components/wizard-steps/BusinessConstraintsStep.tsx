import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Textarea } from '../ui/textarea';
import { BusinessConstraints } from '../../types/comprehensive-requirements';

export function BusinessConstraintsStep({ data, onChange }: { data?: BusinessConstraints; onChange: (data: BusinessConstraints) => void }) {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">💰 Budget</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Total Budget</Label>
              <Input type="number" placeholder="e.g., 500000" />
            </div>
            <div>
              <Label>Currency</Label>
              <Input defaultValue="USD" />
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">📅 Timeline & Milestones</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Go-Live Date</Label>
            <Input type="date" />
          </div>
          <div>
            <Label>Critical Milestones</Label>
            <Textarea placeholder="List key milestones and deadlines" rows={4} />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">👥 Resources & Team</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Team Size</Label>
              <Input type="number" placeholder="e.g., 10" />
            </div>
            <div>
              <Label>Availability</Label>
              <Input placeholder="e.g., Full-time, Part-time" />
            </div>
          </div>
          <div>
            <Label>Skill Matrix</Label>
            <Textarea placeholder="List required skills and roles (e.g., 2 Backend Devs, 1 UI/UX Designer)" rows={3} />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🔧 Technology Constraints</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Mandated Technologies</Label>
            <Input placeholder="e.g., React, Node.js, PostgreSQL" />
          </div>
          <div>
            <Label>Restricted/Prohibited Technologies</Label>
            <Input placeholder="e.g., GPL licenses, certain cloud providers" />
          </div>
          <div>
            <Label>Preferred Vendors</Label>
            <Input placeholder="e.g., Microsoft, AWS" />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">✅ Approval & Sign-Off</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Key Stakeholders</Label>
            <Textarea placeholder="List stakeholders who need to approve (e.g., CTO, CFO, Product Owner)" rows={3} />
          </div>
          <div>
            <Label>Sign-Off Criteria</Label>
            <Textarea placeholder="What needs to be completed before sign-off?" rows={3} />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
