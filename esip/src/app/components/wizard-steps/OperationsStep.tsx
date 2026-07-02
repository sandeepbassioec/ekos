import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Checkbox } from '../ui/checkbox';
import { OperationalRequirements } from '../../types/comprehensive-requirements';

export function OperationsStep({ data, onChange }: { data?: OperationalRequirements; onChange: (data: OperationalRequirements) => void }) {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">📊 Monitoring & Alerting</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>APM Tool</Label>
              <Input placeholder="e.g., New Relic, Datadog, Dynatrace" />
            </div>
            <div>
              <Label>Alert Channels</Label>
              <Input placeholder="e.g., Email, Slack, PagerDuty" />
            </div>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox defaultChecked />
            <Label>Custom Dashboards</Label>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">📝 Logging</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Logging Tool</Label>
              <Input placeholder="e.g., ELK Stack, Splunk, CloudWatch" />
            </div>
            <div>
              <Label>Log Retention (days)</Label>
              <Input type="number" defaultValue="90" />
            </div>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox defaultChecked />
            <Label>Centralized Logging</Label>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🎯 Support SLA</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Critical - Response Time</Label>
              <Input placeholder="e.g., 1 hour" />
            </div>
            <div>
              <Label>Critical - Resolution Time</Label>
              <Input placeholder="e.g., 4 hours" />
            </div>
            <div>
              <Label>High - Response Time</Label>
              <Input placeholder="e.g., 4 hours" />
            </div>
            <div>
              <Label>High - Resolution Time</Label>
              <Input placeholder="e.g., 1 business day" />
            </div>
          </div>
          <div>
            <Label>Support Hours</Label>
            <Input placeholder="e.g., 24/7, Business hours (9-5 EST)" />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">📖 Runbooks & Documentation</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Incident Response Procedures</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Troubleshooting Guides</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox />
            <Label>Escalation Procedures</Label>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
