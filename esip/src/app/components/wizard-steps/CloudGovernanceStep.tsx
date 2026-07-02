import React, { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { Checkbox } from '../ui/checkbox';
import { Button } from '../ui/button';
import { X } from 'lucide-react';
import { CloudGovernanceRequirements } from '../../types/comprehensive-requirements';

interface CloudGovernanceStepProps {
  data?: CloudGovernanceRequirements;
  onChange: (data: CloudGovernanceRequirements) => void;
}

export function CloudGovernanceStep({ data, onChange }: CloudGovernanceStepProps) {
  const defaultData: CloudGovernanceRequirements = data || {
    costManagement: {
      monthlyBudget: '',
      costAlerts: false,
      reservedInstancesStrategy: '',
      spotInstancesAllowed: false,
      costAllocationTags: [],
    },
    resourceGovernance: {
      namingConvention: '',
      taggingPolicy: {
        requiredTags: [],
        optionalTags: [],
      },
      organizationStructure: '',
    },
    cloudSecurity: {
      iamStrategy: '',
      vpcDesign: '',
      encryptionAtRest: false,
      encryptionInTransit: false,
      secretsManagement: '',
    },
    compliance: {
      frameworks: [],
      auditLogging: false,
      complianceReports: '',
    },
    dataResidency: {
      allowedRegions: [],
      dataClassification: '',
      crossRegionReplication: false,
    },
    disasterRecovery: {
      backupStrategy: '',
      multiRegionDeployment: false,
      failoverAutomation: false,
    },
    networking: {
      multiVpc: false,
      vpcPeering: false,
      transitGateway: false,
      privateSubnets: false,
    },
  };

  const [newRequiredTag, setNewRequiredTag] = useState('');
  const [newOptionalTag, setNewOptionalTag] = useState('');
  const [newRegion, setNewRegion] = useState('');
  const [newCostTag, setNewCostTag] = useState('');
  const [selectedFramework, setSelectedFramework] = useState('');

  const updateData = (section: keyof CloudGovernanceRequirements, field: string, value: any) => {
    onChange({
      ...defaultData,
      [section]: {
        ...defaultData[section],
        [field]: value,
      },
    });
  };

  const updateNestedData = (
    section: keyof CloudGovernanceRequirements,
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

  const addRequiredTag = () => {
    if (newRequiredTag.trim()) {
      updateNestedData(
        'resourceGovernance',
        'taggingPolicy',
        'requiredTags',
        [...defaultData.resourceGovernance.taggingPolicy.requiredTags, newRequiredTag.trim()]
      );
      setNewRequiredTag('');
    }
  };

  const removeRequiredTag = (tag: string) => {
    updateNestedData(
      'resourceGovernance',
      'taggingPolicy',
      'requiredTags',
      defaultData.resourceGovernance.taggingPolicy.requiredTags.filter(t => t !== tag)
    );
  };

  const addOptionalTag = () => {
    if (newOptionalTag.trim()) {
      updateNestedData(
        'resourceGovernance',
        'taggingPolicy',
        'optionalTags',
        [...defaultData.resourceGovernance.taggingPolicy.optionalTags, newOptionalTag.trim()]
      );
      setNewOptionalTag('');
    }
  };

  const removeOptionalTag = (tag: string) => {
    updateNestedData(
      'resourceGovernance',
      'taggingPolicy',
      'optionalTags',
      defaultData.resourceGovernance.taggingPolicy.optionalTags.filter(t => t !== tag)
    );
  };

  const addRegion = () => {
    if (newRegion.trim()) {
      updateData('dataResidency', 'allowedRegions', [...defaultData.dataResidency.allowedRegions, newRegion.trim()]);
      setNewRegion('');
    }
  };

  const removeRegion = (region: string) => {
    updateData(
      'dataResidency',
      'allowedRegions',
      defaultData.dataResidency.allowedRegions.filter(r => r !== region)
    );
  };

  const addCostTag = () => {
    if (newCostTag.trim()) {
      updateData('costManagement', 'costAllocationTags', [...defaultData.costManagement.costAllocationTags, newCostTag.trim()]);
      setNewCostTag('');
    }
  };

  const removeCostTag = (tag: string) => {
    updateData(
      'costManagement',
      'costAllocationTags',
      defaultData.costManagement.costAllocationTags.filter(t => t !== tag)
    );
  };

  const addFramework = () => {
    if (selectedFramework && !defaultData.compliance.frameworks.includes(selectedFramework as any)) {
      updateData('compliance', 'frameworks', [...defaultData.compliance.frameworks, selectedFramework]);
      setSelectedFramework('');
    }
  };

  const removeFramework = (framework: string) => {
    updateData(
      'compliance',
      'frameworks',
      defaultData.compliance.frameworks.filter(f => f !== framework)
    );
  };

  return (
    <div className="space-y-6">
      {/* Cost Management */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">💰 Cost Management & Optimization</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Monthly Budget</Label>
              <Input
                placeholder="e.g., $10,000/month"
                value={defaultData.costManagement.monthlyBudget}
                onChange={(e) => updateData('costManagement', 'monthlyBudget', e.target.value)}
              />
            </div>
            <div>
              <Label>Reserved Instances Strategy</Label>
              <Input
                placeholder="e.g., 50% reserved, 50% on-demand"
                value={defaultData.costManagement.reservedInstancesStrategy}
                onChange={(e) => updateData('costManagement', 'reservedInstancesStrategy', e.target.value)}
              />
            </div>
          </div>

          <div className="space-y-3">
            <div className="flex items-center space-x-2">
              <Checkbox
                checked={defaultData.costManagement.costAlerts}
                onCheckedChange={(c) => updateData('costManagement', 'costAlerts', !!c)}
              />
              <Label>Enable Cost Alerts & Budgets</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                checked={defaultData.costManagement.spotInstancesAllowed}
                onCheckedChange={(c) => updateData('costManagement', 'spotInstancesAllowed', !!c)}
              />
              <Label>Allow Spot/Preemptible Instances</Label>
            </div>
          </div>

          <div>
            <Label>Cost Allocation Tags</Label>
            <div className="flex gap-2 mt-2">
              <Input
                placeholder="Add tag (e.g., CostCenter)"
                value={newCostTag}
                onChange={(e) => setNewCostTag(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && addCostTag()}
              />
              <Button type="button" onClick={addCostTag}>Add</Button>
            </div>
            <div className="flex flex-wrap gap-2 mt-2">
              {defaultData.costManagement.costAllocationTags.map((tag) => (
                <div key={tag} className="flex items-center gap-1 bg-blue-100 text-blue-800 px-2 py-1 rounded text-sm">
                  <span>{tag}</span>
                  <button type="button" onClick={() => removeCostTag(tag)}>
                    <X className="w-3 h-3" />
                  </button>
                </div>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Resource Governance */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🏗️ Resource Governance</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Naming Convention</Label>
            <Input
              placeholder="e.g., {environment}-{app}-{resource}-{region}"
              value={defaultData.resourceGovernance.namingConvention}
              onChange={(e) => updateData('resourceGovernance', 'namingConvention', e.target.value)}
            />
          </div>

          <div>
            <Label>Organization Structure</Label>
            <Input
              placeholder="e.g., Multi-account with AWS Organizations"
              value={defaultData.resourceGovernance.organizationStructure}
              onChange={(e) => updateData('resourceGovernance', 'organizationStructure', e.target.value)}
            />
          </div>

          <div>
            <Label>Required Tags</Label>
            <div className="flex gap-2 mt-2">
              <Input
                placeholder="Add required tag (e.g., Environment)"
                value={newRequiredTag}
                onChange={(e) => setNewRequiredTag(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && addRequiredTag()}
              />
              <Button type="button" onClick={addRequiredTag}>Add</Button>
            </div>
            <div className="flex flex-wrap gap-2 mt-2">
              {defaultData.resourceGovernance.taggingPolicy.requiredTags.map((tag) => (
                <div key={tag} className="flex items-center gap-1 bg-red-100 text-red-800 px-2 py-1 rounded text-sm">
                  <span>{tag}</span>
                  <button type="button" onClick={() => removeRequiredTag(tag)}>
                    <X className="w-3 h-3" />
                  </button>
                </div>
              ))}
            </div>
          </div>

          <div>
            <Label>Optional Tags</Label>
            <div className="flex gap-2 mt-2">
              <Input
                placeholder="Add optional tag (e.g., Project)"
                value={newOptionalTag}
                onChange={(e) => setNewOptionalTag(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && addOptionalTag()}
              />
              <Button type="button" onClick={addOptionalTag}>Add</Button>
            </div>
            <div className="flex flex-wrap gap-2 mt-2">
              {defaultData.resourceGovernance.taggingPolicy.optionalTags.map((tag) => (
                <div key={tag} className="flex items-center gap-1 bg-gray-100 text-gray-800 px-2 py-1 rounded text-sm">
                  <span>{tag}</span>
                  <button type="button" onClick={() => removeOptionalTag(tag)}>
                    <X className="w-3 h-3" />
                  </button>
                </div>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Cloud Security */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🔐 Cloud Security</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>IAM Strategy</Label>
              <Input
                placeholder="e.g., Least privilege with role-based access"
                value={defaultData.cloudSecurity.iamStrategy}
                onChange={(e) => updateData('cloudSecurity', 'iamStrategy', e.target.value)}
              />
            </div>
            <div>
              <Label>VPC Design</Label>
              <Input
                placeholder="e.g., Multi-tier with public/private subnets"
                value={defaultData.cloudSecurity.vpcDesign}
                onChange={(e) => updateData('cloudSecurity', 'vpcDesign', e.target.value)}
              />
            </div>
            <div>
              <Label>Secrets Management</Label>
              <Input
                placeholder="e.g., AWS Secrets Manager, Azure Key Vault"
                value={defaultData.cloudSecurity.secretsManagement}
                onChange={(e) => updateData('cloudSecurity', 'secretsManagement', e.target.value)}
              />
            </div>
          </div>

          <div className="space-y-3">
            <div className="flex items-center space-x-2">
              <Checkbox
                checked={defaultData.cloudSecurity.encryptionAtRest}
                onCheckedChange={(c) => updateData('cloudSecurity', 'encryptionAtRest', !!c)}
              />
              <Label>Encryption at Rest</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                checked={defaultData.cloudSecurity.encryptionInTransit}
                onCheckedChange={(c) => updateData('cloudSecurity', 'encryptionInTransit', !!c)}
              />
              <Label>Encryption in Transit (TLS/SSL)</Label>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Compliance */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">✅ Compliance Frameworks</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Compliance Frameworks</Label>
            <div className="flex gap-2 mt-2">
              <Select value={selectedFramework} onValueChange={setSelectedFramework}>
                <SelectTrigger>
                  <SelectValue placeholder="Select framework" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="CIS">CIS Benchmarks</SelectItem>
                  <SelectItem value="AWS Well-Architected">AWS Well-Architected Framework</SelectItem>
                  <SelectItem value="Azure CAF">Azure Cloud Adoption Framework</SelectItem>
                  <SelectItem value="GCP Best Practices">GCP Best Practices</SelectItem>
                  <SelectItem value="Other">Other</SelectItem>
                </SelectContent>
              </Select>
              <Button type="button" onClick={addFramework}>Add</Button>
            </div>
            <div className="flex flex-wrap gap-2 mt-2">
              {defaultData.compliance.frameworks.map((framework) => (
                <div key={framework} className="flex items-center gap-1 bg-green-100 text-green-800 px-2 py-1 rounded text-sm">
                  <span>{framework}</span>
                  <button type="button" onClick={() => removeFramework(framework)}>
                    <X className="w-3 h-3" />
                  </button>
                </div>
              ))}
            </div>
          </div>

          <div>
            <Label>Compliance Reports</Label>
            <Input
              placeholder="e.g., Quarterly SOC 2 reports"
              value={defaultData.compliance.complianceReports}
              onChange={(e) => updateData('compliance', 'complianceReports', e.target.value)}
            />
          </div>

          <div className="flex items-center space-x-2">
            <Checkbox
              checked={defaultData.compliance.auditLogging}
              onCheckedChange={(c) => updateData('compliance', 'auditLogging', !!c)}
            />
            <Label>Enable Comprehensive Audit Logging</Label>
          </div>
        </CardContent>
      </Card>

      {/* Data Residency */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🌍 Data Residency & Sovereignty</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Allowed Regions</Label>
            <div className="flex gap-2 mt-2">
              <Input
                placeholder="Add region (e.g., us-east-1, eu-west-1)"
                value={newRegion}
                onChange={(e) => setNewRegion(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && addRegion()}
              />
              <Button type="button" onClick={addRegion}>Add</Button>
            </div>
            <div className="flex flex-wrap gap-2 mt-2">
              {defaultData.dataResidency.allowedRegions.map((region) => (
                <div key={region} className="flex items-center gap-1 bg-purple-100 text-purple-800 px-2 py-1 rounded text-sm">
                  <span>{region}</span>
                  <button type="button" onClick={() => removeRegion(region)}>
                    <X className="w-3 h-3" />
                  </button>
                </div>
              ))}
            </div>
          </div>

          <div>
            <Label>Data Classification</Label>
            <Input
              placeholder="e.g., PII - Restricted, Public - Unrestricted"
              value={defaultData.dataResidency.dataClassification}
              onChange={(e) => updateData('dataResidency', 'dataClassification', e.target.value)}
            />
          </div>

          <div className="flex items-center space-x-2">
            <Checkbox
              checked={defaultData.dataResidency.crossRegionReplication}
              onCheckedChange={(c) => updateData('dataResidency', 'crossRegionReplication', !!c)}
            />
            <Label>Allow Cross-Region Replication</Label>
          </div>
        </CardContent>
      </Card>

      {/* Disaster Recovery */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🚨 Disaster Recovery</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Backup Strategy</Label>
            <Input
              placeholder="e.g., Daily snapshots, 30-day retention"
              value={defaultData.disasterRecovery.backupStrategy}
              onChange={(e) => updateData('disasterRecovery', 'backupStrategy', e.target.value)}
            />
          </div>

          <div className="space-y-3">
            <div className="flex items-center space-x-2">
              <Checkbox
                checked={defaultData.disasterRecovery.multiRegionDeployment}
                onCheckedChange={(c) => updateData('disasterRecovery', 'multiRegionDeployment', !!c)}
              />
              <Label>Multi-Region Deployment</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                checked={defaultData.disasterRecovery.failoverAutomation}
                onCheckedChange={(c) => updateData('disasterRecovery', 'failoverAutomation', !!c)}
              />
              <Label>Automated Failover</Label>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Networking */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🌐 Cloud Networking</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center space-x-2">
            <Checkbox
              checked={defaultData.networking.multiVpc}
              onCheckedChange={(c) => updateData('networking', 'multiVpc', !!c)}
            />
            <Label>Multi-VPC Architecture</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox
              checked={defaultData.networking.vpcPeering}
              onCheckedChange={(c) => updateData('networking', 'vpcPeering', !!c)}
            />
            <Label>VPC Peering / VNet Peering</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox
              checked={defaultData.networking.transitGateway}
              onCheckedChange={(c) => updateData('networking', 'transitGateway', !!c)}
            />
            <Label>Transit Gateway / Virtual WAN</Label>
          </div>
          <div className="flex items-center space-x-2">
            <Checkbox
              checked={defaultData.networking.privateSubnets}
              onCheckedChange={(c) => updateData('networking', 'privateSubnets', !!c)}
            />
            <Label>Use Private Subnets for Workloads</Label>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
