import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Checkbox } from '../ui/checkbox';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { UXRequirements } from '../../types/comprehensive-requirements';

export function UXStep({ data, onChange }: { data?: UXRequirements; onChange: (data: UXRequirements) => void }) {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🎨 Branding & Design</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Color Scheme</Label>
            <Input placeholder="e.g., #3B82F6, #10B981" />
          </div>
          <div>
            <Label>Typography</Label>
            <Input placeholder="e.g., Inter, Roboto" />
          </div>
          <div>
            <Label>Brand Guidelines URL</Label>
            <Input type="url" placeholder="https://..." />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🧩 Design System</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Design Framework</Label>
            <Select defaultValue="material">
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="material">Material Design</SelectItem>
                <SelectItem value="bootstrap">Bootstrap</SelectItem>
                <SelectItem value="tailwind">Tailwind CSS</SelectItem>
                <SelectItem value="custom">Custom Design System</SelectItem>
                <SelectItem value="other">Other</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div>
            <Label>Component Library</Label>
            <Input placeholder="e.g., Ant Design, Material-UI, Chakra UI" />
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Design Tokens</Label>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">♿ Accessibility</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>WCAG Level</Label>
            <Select defaultValue="WCAG 2.1 AA">
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="WCAG 2.1 A">WCAG 2.1 Level A</SelectItem>
                <SelectItem value="WCAG 2.1 AA">WCAG 2.1 Level AA</SelectItem>
                <SelectItem value="WCAG 2.1 AAA">WCAG 2.1 Level AAA</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <div className="flex items-center space-x-2">
              <Checkbox />
              <Label>Screen Reader Support</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox />
              <Label>Full Keyboard Navigation</Label>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🌍 Localization</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Supported Languages</Label>
            <Input placeholder="e.g., English, Spanish, French" />
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>RTL (Right-to-Left) Support</Label>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">📱 Responsive Design</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-3 gap-4">
            <div>
              <Label>Desktop Breakpoint</Label>
              <Input placeholder="e.g., 1280px" />
            </div>
            <div>
              <Label>Tablet Breakpoint</Label>
              <Input placeholder="e.g., 768px" />
            </div>
            <div>
              <Label>Mobile Breakpoint</Label>
              <Input placeholder="e.g., 375px" />
            </div>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox defaultChecked />
            <Label>Mobile-First Approach</Label>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🌐 Browser Support</CardTitle>
        </CardHeader>
        <CardContent>
          <Input placeholder="e.g., Chrome (last 2 versions), Firefox (last 2 versions), Safari 14+" />
        </CardContent>
      </Card>
    </div>
  );
}
