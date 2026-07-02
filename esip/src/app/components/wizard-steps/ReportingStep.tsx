import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Checkbox } from '../ui/checkbox';
import { Textarea } from '../ui/textarea';
import { ReportingAnalytics } from '../../types/comprehensive-requirements';

export function ReportingStep({ data, onChange }: { data?: ReportingAnalytics; onChange: (data: ReportingAnalytics) => void }) {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">📊 Business Reports & Dashboards</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Key Reports Required</Label>
            <Textarea placeholder="List required reports (e.g., Sales Dashboard, User Analytics, Performance Metrics)" rows={4} />
          </div>
          <div>
            <Label>KPIs to Track</Label>
            <Textarea placeholder="e.g., Revenue, Active Users, Conversion Rate" rows={3} />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🔧 Operational Reports</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>System Health Reports</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Usage Statistics</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Performance Metrics</Label>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">📋 Compliance & Audit Reports</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Audit Reports</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Regulatory Compliance Reports</Label>
          </div>
          <div>
            <Label>Report Frequency</Label>
            <Input placeholder="e.g., Monthly, Quarterly, On-demand" />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">💾 Export Formats</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          <div className="flex items-center space-x-2">
            <Checkbox defaultChecked />
            <Label>PDF</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox defaultChecked />
            <Label>Excel (XLSX)</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>CSV</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>JSON</Label>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">⏱️ Real-Time Analytics</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Real-Time Data Visualization & Analytics</Label>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
