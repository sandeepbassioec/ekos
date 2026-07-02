import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { Checkbox } from '../ui/checkbox';
import { NonFunctionalRequirements } from '../../types/comprehensive-requirements';

interface NFRStepProps {
  data?: NonFunctionalRequirements;
  onChange: (data: NonFunctionalRequirements) => void;
}

export function NFRStep({ data, onChange }: NFRStepProps) {
  const defaultData: NonFunctionalRequirements = data || {
    performance: {
      responseTime: '',
      throughput: '',
      concurrentUsers: 0,
      pageLoadTime: '',
      apiResponseTime: '',
    },
    scalability: {
      expectedUserGrowth: '',
      dataVolumeGrowth: '',
      peakLoadScenario: '',
      scalingStrategy: 'horizontal',
    },
    availability: {
      uptimeSLA: '',
      maintenanceWindow: '',
      disasterRecovery: {
        rto: '',
        rpo: '',
      },
    },
    reliability: {
      mtbf: '',
      errorRate: '',
      dataIntegrity: '',
    },
    usability: {
      accessibilityLevel: 'WCAG 2.1 AA',
      mobileResponsive: true,
      screenReaderSupport: false,
      keyboardNavigation: false,
      uxStandards: '',
    },
    maintainability: {
      codeQualityStandards: '',
      documentationLevel: 'standard',
      codeCoverageTarget: 80,
      technicalDebtPolicy: '',
    },
  };

  const updateData = (section: keyof NonFunctionalRequirements, field: string, value: any) => {
    onChange({
      ...defaultData,
      [section]: {
        ...defaultData[section],
        [field]: value,
      },
    });
  };

  const updateNestedData = (
    section: keyof NonFunctionalRequirements,
    subsection: string,
    field: string,
    value: any
  ) => {
    onChange({
      ...defaultData,
      [section]: {
        ...defaultData[section],
        [subsection]: {
          ...(defaultData[section] as any)[subsection],
          [field]: value,
        },
      },
    });
  };

  return (
    <div className="space-y-6">
      {/* Performance */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">⚡ Performance Requirements</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Response Time</Label>
              <Input
                placeholder="e.g., < 2 seconds"
                value={defaultData.performance.responseTime}
                onChange={(e) => updateData('performance', 'responseTime', e.target.value)}
              />
            </div>
            <div>
              <Label>Throughput</Label>
              <Input
                placeholder="e.g., 1000 requests/sec"
                value={defaultData.performance.throughput}
                onChange={(e) => updateData('performance', 'throughput', e.target.value)}
              />
            </div>
            <div>
              <Label>Concurrent Users</Label>
              <Input
                type="number"
                placeholder="e.g., 10000"
                value={defaultData.performance.concurrentUsers}
                onChange={(e) => updateData('performance', 'concurrentUsers', parseInt(e.target.value) || 0)}
              />
            </div>
            <div>
              <Label>Page Load Time</Label>
              <Input
                placeholder="e.g., < 3 seconds"
                value={defaultData.performance.pageLoadTime}
                onChange={(e) => updateData('performance', 'pageLoadTime', e.target.value)}
              />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Scalability */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">📈 Scalability Requirements</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Expected User Growth</Label>
              <Input
                placeholder="e.g., 50% year-over-year"
                value={defaultData.scalability.expectedUserGrowth}
                onChange={(e) => updateData('scalability', 'expectedUserGrowth', e.target.value)}
              />
            </div>
            <div>
              <Label>Data Volume Growth</Label>
              <Input
                placeholder="e.g., 100GB/month"
                value={defaultData.scalability.dataVolumeGrowth}
                onChange={(e) => updateData('scalability', 'dataVolumeGrowth', e.target.value)}
              />
            </div>
            <div>
              <Label>Peak Load Scenario</Label>
              <Input
                placeholder="e.g., Black Friday - 10x traffic"
                value={defaultData.scalability.peakLoadScenario}
                onChange={(e) => updateData('scalability', 'peakLoadScenario', e.target.value)}
              />
            </div>
            <div>
              <Label>Scaling Strategy</Label>
              <Select
                value={defaultData.scalability.scalingStrategy}
                onValueChange={(v: any) => updateData('scalability', 'scalingStrategy', v)}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="horizontal">Horizontal (Add more servers)</SelectItem>
                  <SelectItem value="vertical">Vertical (Bigger servers)</SelectItem>
                  <SelectItem value="both">Both</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Availability */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🟢 Availability & Reliability</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Uptime SLA</Label>
              <Input
                placeholder="e.g., 99.9%"
                value={defaultData.availability.uptimeSLA}
                onChange={(e) => updateData('availability', 'uptimeSLA', e.target.value)}
              />
            </div>
            <div>
              <Label>Maintenance Window</Label>
              <Input
                placeholder="e.g., Sunday 2-4 AM EST"
                value={defaultData.availability.maintenanceWindow}
                onChange={(e) => updateData('availability', 'maintenanceWindow', e.target.value)}
              />
            </div>
            <div>
              <Label>RTO (Recovery Time Objective)</Label>
              <Input
                placeholder="e.g., 1 hour"
                value={defaultData.availability.disasterRecovery.rto}
                onChange={(e) => updateNestedData('availability', 'disasterRecovery', 'rto', e.target.value)}
              />
            </div>
            <div>
              <Label>RPO (Recovery Point Objective)</Label>
              <Input
                placeholder="e.g., 15 minutes"
                value={defaultData.availability.disasterRecovery.rpo}
                onChange={(e) => updateNestedData('availability', 'disasterRecovery', 'rpo', e.target.value)}
              />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Usability */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">👥 Usability & Accessibility</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Accessibility Level</Label>
              <Select
                value={defaultData.usability.accessibilityLevel}
                onValueChange={(v: any) => updateData('usability', 'accessibilityLevel', v)}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="None">None</SelectItem>
                  <SelectItem value="WCAG 2.1 A">WCAG 2.1 Level A</SelectItem>
                  <SelectItem value="WCAG 2.1 AA">WCAG 2.1 Level AA</SelectItem>
                  <SelectItem value="WCAG 2.1 AAA">WCAG 2.1 Level AAA</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label>UX Standards</Label>
              <Input
                placeholder="e.g., Material Design 3.0"
                value={defaultData.usability.uxStandards}
                onChange={(e) => updateData('usability', 'uxStandards', e.target.value)}
              />
            </div>
          </div>

          <div className="space-y-3">
            <div className="flex items-center space-x-2">
              <Checkbox
                checked={defaultData.usability.mobileResponsive}
                onCheckedChange={(c) => updateData('usability', 'mobileResponsive', !!c)}
              />
              <Label>Mobile Responsive Design</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                checked={defaultData.usability.screenReaderSupport}
                onCheckedChange={(c) => updateData('usability', 'screenReaderSupport', !!c)}
              />
              <Label>Screen Reader Support</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                checked={defaultData.usability.keyboardNavigation}
                onCheckedChange={(c) => updateData('usability', 'keyboardNavigation', !!c)}
              />
              <Label>Full Keyboard Navigation</Label>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Maintainability */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🔧 Maintainability</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Code Quality Standards</Label>
              <Input
                placeholder="e.g., SonarQube Quality Gate"
                value={defaultData.maintainability.codeQualityStandards}
                onChange={(e) => updateData('maintainability', 'codeQualityStandards', e.target.value)}
              />
            </div>
            <div>
              <Label>Documentation Level</Label>
              <Select
                value={defaultData.maintainability.documentationLevel}
                onValueChange={(v: any) => updateData('maintainability', 'documentationLevel', v)}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="minimal">Minimal</SelectItem>
                  <SelectItem value="standard">Standard</SelectItem>
                  <SelectItem value="comprehensive">Comprehensive</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label>Code Coverage Target (%)</Label>
              <Input
                type="number"
                placeholder="e.g., 80"
                value={defaultData.maintainability.codeCoverageTarget}
                onChange={(e) =>
                  updateData('maintainability', 'codeCoverageTarget', parseInt(e.target.value) || 80)
                }
              />
            </div>
            <div>
              <Label>Technical Debt Policy</Label>
              <Input
                placeholder="e.g., Max 5% per sprint"
                value={defaultData.maintainability.technicalDebtPolicy}
                onChange={(e) => updateData('maintainability', 'technicalDebtPolicy', e.target.value)}
              />
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
