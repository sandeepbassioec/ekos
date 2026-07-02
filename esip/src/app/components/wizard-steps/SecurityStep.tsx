import React, { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Checkbox } from '../ui/checkbox';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { Plus, Trash2 } from 'lucide-react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { SecurityRequirements } from '../../types/comprehensive-requirements';

interface SecurityStepProps {
  data?: SecurityRequirements;
  onChange: (data: SecurityRequirements) => void;
}

export function SecurityStep({ data, onChange }: SecurityStepProps) {
  const defaultData: SecurityRequirements = data || {
    authentication: {
      methods: ['email_password'],
      mfaRequired: false,
      sessionTimeout: 30,
      passwordPolicy: {
        minLength: 8,
        requireSpecialChars: true,
        requireNumbers: true,
        expiryDays: 90,
      },
    },
    authorization: {
      model: 'rbac',
      roles: [],
      permissionsMatrix: '',
    },
    dataEncryption: {
      atRest: false,
      inTransit: true,
      algorithm: '',
      keyManagement: '',
    },
    compliance: {
      standards: [],
      dataResidency: '',
      rightToForgotten: false,
      consentManagement: false,
    },
    auditLogging: {
      enabled: true,
      logRetention: 90,
      loggedEvents: [],
      siemIntegration: '',
    },
    securityTesting: {
      penetrationTesting: false,
      vulnerabilityScanning: false,
      owaspTop10: false,
      frequency: '',
    },
  };

  const [newRole, setNewRole] = useState('');
  const [newEvent, setNewEvent] = useState('');

  const authMethods = [
    { value: 'email_password', label: 'Email & Password' },
    { value: 'oauth', label: 'OAuth 2.0' },
    { value: 'saml', label: 'SAML SSO' },
    { value: 'sso', label: 'Single Sign-On' },
    { value: 'ldap', label: 'LDAP/Active Directory' },
    { value: 'biometric', label: 'Biometric' },
    { value: 'mfa', label: 'Multi-Factor Auth' },
  ];

  const complianceStandards = [
    { value: 'GDPR', label: 'GDPR (Europe)' },
    { value: 'HIPAA', label: 'HIPAA (Healthcare - US)' },
    { value: 'SOC2', label: 'SOC 2' },
    { value: 'PCI_DSS', label: 'PCI DSS (Payment Cards)' },
    { value: 'ISO27001', label: 'ISO 27001' },
    { value: 'SOX', label: 'SOX (Finance - US)' },
    { value: 'CCPA', label: 'CCPA (California)' },
    { value: 'Other', label: 'Other' },
  ];

  const toggleAuthMethod = (method: string) => {
    const methods = defaultData.authentication.methods;
    const newMethods = methods.includes(method as any)
      ? methods.filter((m) => m !== method)
      : [...methods, method as any];
    
    onChange({
      ...defaultData,
      authentication: {
        ...defaultData.authentication,
        methods: newMethods,
      },
    });
  };

  const toggleCompliance = (standard: string) => {
    const standards = defaultData.compliance.standards;
    const newStandards = standards.includes(standard as any)
      ? standards.filter((s) => s !== standard)
      : [...standards, standard as any];
    
    onChange({
      ...defaultData,
      compliance: {
        ...defaultData.compliance,
        standards: newStandards,
      },
    });
  };

  const addRole = () => {
    if (newRole.trim()) {
      onChange({
        ...defaultData,
        authorization: {
          ...defaultData.authorization,
          roles: [...defaultData.authorization.roles, newRole.trim()],
        },
      });
      setNewRole('');
    }
  };

  const removeRole = (role: string) => {
    onChange({
      ...defaultData,
      authorization: {
        ...defaultData.authorization,
        roles: defaultData.authorization.roles.filter((r) => r !== role),
      },
    });
  };

  const addEvent = () => {
    if (newEvent.trim()) {
      onChange({
        ...defaultData,
        auditLogging: {
          ...defaultData.auditLogging,
          loggedEvents: [...defaultData.auditLogging.loggedEvents, newEvent.trim()],
        },
      });
      setNewEvent('');
    }
  };

  const removeEvent = (event: string) => {
    onChange({
      ...defaultData,
      auditLogging: {
        ...defaultData.auditLogging,
        loggedEvents: defaultData.auditLogging.loggedEvents.filter((e) => e !== event),
      },
    });
  };

  return (
    <div className="space-y-6">
      {/* Authentication */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🔐 Authentication</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label className="mb-3 block">Authentication Methods</Label>
            <div className="grid grid-cols-2 gap-3">
              {authMethods.map((method) => (
                <div key={method.value} className="flex items-center space-x-2">
                  <Checkbox
                    checked={defaultData.authentication.methods.includes(method.value as any)}
                    onCheckedChange={() => toggleAuthMethod(method.value)}
                  />
                  <Label>{method.label}</Label>
                </div>
              ))}
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Session Timeout (minutes)</Label>
              <Input
                type="number"
                value={defaultData.authentication.sessionTimeout}
                onChange={(e) =>
                  onChange({
                    ...defaultData,
                    authentication: {
                      ...defaultData.authentication,
                      sessionTimeout: parseInt(e.target.value) || 30,
                    },
                  })
                }
              />
            </div>
            <div className="flex items-center space-x-2 pt-8">
              <Checkbox
                checked={defaultData.authentication.mfaRequired}
                onCheckedChange={(c) =>
                  onChange({
                    ...defaultData,
                    authentication: {
                      ...defaultData.authentication,
                      mfaRequired: !!c,
                    },
                  })
                }
              />
              <Label>Require Multi-Factor Authentication</Label>
            </div>
          </div>

          <div className="border-t pt-4">
            <Label className="mb-3 block">Password Policy</Label>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label>Minimum Length</Label>
                <Input
                  type="number"
                  value={defaultData.authentication.passwordPolicy.minLength}
                  onChange={(e) =>
                    onChange({
                      ...defaultData,
                      authentication: {
                        ...defaultData.authentication,
                        passwordPolicy: {
                          ...defaultData.authentication.passwordPolicy,
                          minLength: parseInt(e.target.value) || 8,
                        },
                      },
                    })
                  }
                />
              </div>
              <div>
                <Label>Password Expiry (days)</Label>
                <Input
                  type="number"
                  value={defaultData.authentication.passwordPolicy.expiryDays}
                  onChange={(e) =>
                    onChange({
                      ...defaultData,
                      authentication: {
                        ...defaultData.authentication,
                        passwordPolicy: {
                          ...defaultData.authentication.passwordPolicy,
                          expiryDays: parseInt(e.target.value) || 90,
                        },
                      },
                    })
                  }
                />
              </div>
            </div>
            <div className="flex gap-4 mt-3">
              <div className="flex items-center space-x-2">
                <Checkbox
                  checked={defaultData.authentication.passwordPolicy.requireSpecialChars}
                  onCheckedChange={(c) =>
                    onChange({
                      ...defaultData,
                      authentication: {
                        ...defaultData.authentication,
                        passwordPolicy: {
                          ...defaultData.authentication.passwordPolicy,
                          requireSpecialChars: !!c,
                        },
                      },
                    })
                  }
                />
                <Label>Require Special Characters</Label>
              </div>
              <div className="flex items-center space-x-2">
                <Checkbox
                  checked={defaultData.authentication.passwordPolicy.requireNumbers}
                  onCheckedChange={(c) =>
                    onChange({
                      ...defaultData,
                      authentication: {
                        ...defaultData.authentication,
                        passwordPolicy: {
                          ...defaultData.authentication.passwordPolicy,
                          requireNumbers: !!c,
                        },
                      },
                    })
                  }
                />
                <Label>Require Numbers</Label>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Authorization */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🛡️ Authorization</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Authorization Model</Label>
            <Select
              value={defaultData.authorization.model}
              onValueChange={(v: any) =>
                onChange({
                  ...defaultData,
                  authorization: {
                    ...defaultData.authorization,
                    model: v,
                  },
                })
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="rbac">RBAC (Role-Based Access Control)</SelectItem>
                <SelectItem value="abac">ABAC (Attribute-Based Access Control)</SelectItem>
                <SelectItem value="custom">Custom</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div>
            <Label className="mb-2 block">Roles</Label>
            <div className="flex flex-wrap gap-2 mb-3">
              {defaultData.authorization.roles.map((role) => (
                <Badge key={role} variant="secondary" className="flex items-center gap-2">
                  {role}
                  <Trash2
                    className="w-3 h-3 cursor-pointer text-red-500"
                    onClick={() => removeRole(role)}
                  />
                </Badge>
              ))}
            </div>
            <div className="flex gap-2">
              <Input
                placeholder="Add role (e.g., Admin, Manager, User)"
                value={newRole}
                onChange={(e) => setNewRole(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && addRole()}
              />
              <Button onClick={addRole} size="sm">
                <Plus className="w-4 h-4" />
              </Button>
            </div>
          </div>

          <div>
            <Label>Permissions Matrix</Label>
            <Input
              placeholder="URL or description of permissions matrix"
              value={defaultData.authorization.permissionsMatrix}
              onChange={(e) =>
                onChange({
                  ...defaultData,
                  authorization: {
                    ...defaultData.authorization,
                    permissionsMatrix: e.target.value,
                  },
                })
              }
            />
          </div>
        </CardContent>
      </Card>

      {/* Data Encryption */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🔒 Data Encryption</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="flex items-center space-x-2">
              <Checkbox
                checked={defaultData.dataEncryption.atRest}
                onCheckedChange={(c) =>
                  onChange({
                    ...defaultData,
                    dataEncryption: {
                      ...defaultData.dataEncryption,
                      atRest: !!c,
                    },
                  })
                }
              />
              <Label>Encryption at Rest</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                checked={defaultData.dataEncryption.inTransit}
                onCheckedChange={(c) =>
                  onChange({
                    ...defaultData,
                    dataEncryption: {
                      ...defaultData.dataEncryption,
                      inTransit: !!c,
                    },
                  })
                }
              />
              <Label>Encryption in Transit (TLS/SSL)</Label>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Encryption Algorithm</Label>
              <Input
                placeholder="e.g., AES-256"
                value={defaultData.dataEncryption.algorithm}
                onChange={(e) =>
                  onChange({
                    ...defaultData,
                    dataEncryption: {
                      ...defaultData.dataEncryption,
                      algorithm: e.target.value,
                    },
                  })
                }
              />
            </div>
            <div>
              <Label>Key Management</Label>
              <Input
                placeholder="e.g., AWS KMS, HashiCorp Vault"
                value={defaultData.dataEncryption.keyManagement}
                onChange={(e) =>
                  onChange({
                    ...defaultData,
                    dataEncryption: {
                      ...defaultData.dataEncryption,
                      keyManagement: e.target.value,
                    },
                  })
                }
              />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Compliance */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">📋 Compliance Requirements</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label className="mb-3 block">Compliance Standards</Label>
            <div className="grid grid-cols-2 gap-3">
              {complianceStandards.map((standard) => (
                <div key={standard.value} className="flex items-center space-x-2">
                  <Checkbox
                    checked={defaultData.compliance.standards.includes(standard.value as any)}
                    onCheckedChange={() => toggleCompliance(standard.value)}
                  />
                  <Label>{standard.label}</Label>
                </div>
              ))}
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <Label>Data Residency</Label>
              <Input
                placeholder="e.g., EU only, US only"
                value={defaultData.compliance.dataResidency}
                onChange={(e) =>
                  onChange({
                    ...defaultData,
                    compliance: {
                      ...defaultData.compliance,
                      dataResidency: e.target.value,
                    },
                  })
                }
              />
            </div>
            <div className="space-y-2">
              <div className="flex items-center space-x-2">
                <Checkbox
                  checked={defaultData.compliance.rightToForgotten}
                  onCheckedChange={(c) =>
                    onChange({
                      ...defaultData,
                      compliance: {
                        ...defaultData.compliance,
                        rightToForgotten: !!c,
                      },
                    })
                  }
                />
                <Label>Right to be Forgotten (GDPR)</Label>
              </div>
              <div className="flex items-center space-x-2">
                <Checkbox
                  checked={defaultData.compliance.consentManagement}
                  onCheckedChange={(c) =>
                    onChange({
                      ...defaultData,
                      compliance: {
                        ...defaultData.compliance,
                        consentManagement: !!c,
                      },
                    })
                  }
                />
                <Label>Consent Management</Label>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Audit Logging */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">📝 Audit & Logging</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="flex items-center space-x-2">
              <Checkbox
                checked={defaultData.auditLogging.enabled}
                onCheckedChange={(c) =>
                  onChange({
                    ...defaultData,
                    auditLogging: {
                      ...defaultData.auditLogging,
                      enabled: !!c,
                    },
                  })
                }
              />
              <Label>Enable Audit Logging</Label>
            </div>
            <div>
              <Label>Log Retention (days)</Label>
              <Input
                type="number"
                value={defaultData.auditLogging.logRetention}
                onChange={(e) =>
                  onChange({
                    ...defaultData,
                    auditLogging: {
                      ...defaultData.auditLogging,
                      logRetention: parseInt(e.target.value) || 90,
                    },
                  })
                }
              />
            </div>
          </div>

          <div>
            <Label>SIEM Integration</Label>
            <Input
              placeholder="e.g., Splunk, ELK Stack"
              value={defaultData.auditLogging.siemIntegration}
              onChange={(e) =>
                onChange({
                  ...defaultData,
                  auditLogging: {
                    ...defaultData.auditLogging,
                    siemIntegration: e.target.value,
                  },
                })
              }
            />
          </div>

          <div>
            <Label className="mb-2 block">Logged Events</Label>
            <div className="flex flex-wrap gap-2 mb-3">
              {defaultData.auditLogging.loggedEvents.map((event) => (
                <Badge key={event} variant="secondary" className="flex items-center gap-2">
                  {event}
                  <Trash2
                    className="w-3 h-3 cursor-pointer text-red-500"
                    onClick={() => removeEvent(event)}
                  />
                </Badge>
              ))}
            </div>
            <div className="flex gap-2">
              <Input
                placeholder="Add event (e.g., Login, User Creation)"
                value={newEvent}
                onChange={(e) => setNewEvent(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && addEvent()}
              />
              <Button onClick={addEvent} size="sm">
                <Plus className="w-4 h-4" />
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Security Testing */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">🔍 Security Testing</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-3">
            <div className="flex items-center space-x-2">
              <Checkbox
                checked={defaultData.securityTesting.penetrationTesting}
                onCheckedChange={(c) =>
                  onChange({
                    ...defaultData,
                    securityTesting: {
                      ...defaultData.securityTesting,
                      penetrationTesting: !!c,
                    },
                  })
                }
              />
              <Label>Penetration Testing</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                checked={defaultData.securityTesting.vulnerabilityScanning}
                onCheckedChange={(c) =>
                  onChange({
                    ...defaultData,
                    securityTesting: {
                      ...defaultData.securityTesting,
                      vulnerabilityScanning: !!c,
                    },
                  })
                }
              />
              <Label>Vulnerability Scanning</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                checked={defaultData.securityTesting.owaspTop10}
                onCheckedChange={(c) =>
                  onChange({
                    ...defaultData,
                    securityTesting: {
                      ...defaultData.securityTesting,
                      owaspTop10: !!c,
                    },
                  })
                }
              />
              <Label>OWASP Top 10 Testing</Label>
            </div>
            <div>
              <Label>Testing Frequency</Label>
              <Input
                placeholder="e.g., Quarterly"
                value={defaultData.securityTesting.frequency}
                onChange={(e) =>
                  onChange({
                    ...defaultData,
                    securityTesting: {
                      ...defaultData.securityTesting,
                      frequency: e.target.value,
                    },
                  })
                }
              />
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
