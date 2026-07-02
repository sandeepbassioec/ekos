import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Checkbox } from '../ui/checkbox';
import { Textarea } from '../ui/textarea';
import { TrainingDocumentation } from '../../types/comprehensive-requirements';

export function TrainingStep({ data, onChange }: { data?: TrainingDocumentation; onChange: (data: TrainingDocumentation) => void }) {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">👥 User Training</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center space-x-2">
            <Checkbox defaultChecked />
            <Label>User Training Required</Label>
          </div>
          <div>
            <Label>Training Materials</Label>
            <div className="space-y-2 mt-2">
              <div className="flex items-center space-x-2">
                <Checkbox />
                <Label>Video Tutorials</Label>
              </div>
              <div className="flex items-center space-x-2">
                <Checkbox />
                <Label>PDF Guides</Label>
              </div>
              <div className="flex items-center space-x-2">
                <Checkbox />
                <Label>Interactive Tutorials</Label>
              </div>
              <div className="flex items-center space-x-2">
                <Checkbox />
                <Label>Workshops/Webinars</Label>
              </div>
            </div>
          </div>
          <div>
            <Label>Training Duration</Label>
            <Input placeholder="e.g., 2 days, 4 hours" />
          </div>
          <div>
            <Label>Target Audience</Label>
            <Textarea placeholder="Who needs training? (e.g., End users, Admins, Power users)" rows={2} />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🔧 Admin Training</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Admin Training Required</Label>
          </div>
          <div>
            <Label>Training Topics</Label>
            <Textarea placeholder="e.g., User management, System configuration, Report generation" rows={3} />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">📚 Technical Documentation</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center space-x-2">
            <Checkbox defaultChecked />
            <Label>Architecture Documentation</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox defaultChecked />
            <Label>API Documentation</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Deployment Guides</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Database Schema Documentation</Label>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">💡 Knowledge Base</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <div className="flex items-center space-x-2">
              <Checkbox />
              <Label>FAQs</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox />
              <Label>How-To Guides</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox />
              <Label>Troubleshooting Articles</Label>
            </div>
          </div>
          <div>
            <Label>Knowledge Base Platform</Label>
            <Input placeholder="e.g., Confluence, Notion, GitBook" />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
