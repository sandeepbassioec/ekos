import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Checkbox } from '../ui/checkbox';
import { Textarea } from '../ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { DataRequirements } from '../../types/comprehensive-requirements';

interface DataStepProps {
  data?: DataRequirements;
  onChange: (data: DataRequirements) => void;
}

export function DataStep({ data, onChange }: DataStepProps) {
  const defaultData: DataRequirements = data || {
    dataModel: { entities: [], relationships: '' },
    dataMigration: {
      required: false,
      sourceSystems: [],
      strategy: 'phased',
      dataVolume: '',
      dataCleansing: '',
      mappingDocument: '',
    },
    dataRetention: { policy: '', archivalSchedule: '', purgeSchedule: '' },
    backupRecovery: {
      backupFrequency: 'daily',
      retentionPeriod: '',
      recoveryProcedure: '',
      testingFrequency: '',
    },
    dataPrivacy: {
      piiHandling: '',
      dataMasking: false,
      anonymization: false,
      rightToDeletion: false,
    },
    masterData: { referenceSources: [], syncFrequency: '' },
  };

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">📊 Data Model</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Entity Relationships</Label>
            <Textarea placeholder="Describe main entities and their relationships" rows={4} />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🔄 Data Migration</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center space-x-2">
            <Checkbox
              checked={defaultData.dataMigration.required}
              onCheckedChange={(c) =>
                onChange({
                  ...defaultData,
                  dataMigration: { ...defaultData.dataMigration, required: !!c },
                })
              }
            />
            <Label>Data Migration Required</Label>
          </div>

          {defaultData.dataMigration.required && (
            <>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label>Source Systems</Label>
                  <Input placeholder="e.g., Legacy Oracle DB, Excel files" />
                </div>
                <div>
                  <Label>Migration Strategy</Label>
                  <Select defaultValue="phased">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="big_bang">Big Bang</SelectItem>
                      <SelectItem value="phased">Phased</SelectItem>
                      <SelectItem value="parallel_run">Parallel Run</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div>
                  <Label>Data Volume</Label>
                  <Input placeholder="e.g., 500GB, 10M records" />
                </div>
              </div>
            </>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">💾 Backup & Recovery</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Backup Frequency</Label>
              <Select defaultValue="daily">
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="real_time">Real-time</SelectItem>
                  <SelectItem value="hourly">Hourly</SelectItem>
                  <SelectItem value="daily">Daily</SelectItem>
                  <SelectItem value="weekly">Weekly</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label>Retention Period</Label>
              <Input placeholder="e.g., 30 days, 1 year" />
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🔒 Data Privacy</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>PII Handling Policy</Label>
            <Textarea placeholder="How is personally identifiable information handled?" rows={3} />
          </div>
          <div className="space-y-2">
            <div className="flex items-center space-x-2">
              <Checkbox />
              <Label>Data Masking</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox />
              <Label>Anonymization</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox />
              <Label>Right to Deletion (GDPR)</Label>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
