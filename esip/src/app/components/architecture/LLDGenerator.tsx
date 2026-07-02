import React, { useState, useEffect } from 'react';
import { Download, FileText, Plus, Code2, Database, Webhook } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import { MermaidDiagram } from './MermaidDiagram';
import { toast } from 'sonner';
import { Badge } from '../ui/badge';

interface APIEndpoint {
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  path: string;
  description: string;
  auth: boolean;
  requestBody?: string;
  responseBody?: string;
}

interface DatabaseTable {
  name: string;
  columns: Array<{
    name: string;
    type: string;
    nullable: boolean;
    primaryKey: boolean;
    foreignKey?: string;
  }>;
}

export function LLDGenerator() {
  const [apiEndpoints, setApiEndpoints] = useState<APIEndpoint[]>([]);
  const [dbTables, setDbTables] = useState<DatabaseTable[]>([]);
  const [classDiagram, setClassDiagram] = useState('');
  const [sequenceDiagram, setSequenceDiagram] = useState('');
  const [apiSpec, setApiSpec] = useState('');
  const [dbSchemaSQL, setDbSchemaSQL] = useState('');

  useEffect(() => {
    loadSampleData();
  }, []);

  const loadSampleData = () => {
    const sampleEndpoints: APIEndpoint[] = [
      {
        method: 'POST',
        path: '/api/auth/login',
        description: 'User login with email and password',
        auth: false,
        requestBody: '{ "email": "string", "password": "string" }',
        responseBody: '{ "token": "string", "user": { "id": "string", "email": "string", "role": "string" } }',
      },
      {
        method: 'POST',
        path: '/api/auth/register',
        description: 'Register new user account',
        auth: false,
        requestBody: '{ "email": "string", "password": "string", "fullName": "string" }',
        responseBody: '{ "token": "string", "user": { "id": "string", "email": "string" } }',
      },
      {
        method: 'GET',
        path: '/api/projects',
        description: 'Get all projects for authenticated user',
        auth: true,
        responseBody: '{ "projects": [{ "id": "string", "name": "string", "status": "string" }] }',
      },
      {
        method: 'POST',
        path: '/api/projects',
        description: 'Create a new project',
        auth: true,
        requestBody: '{ "name": "string", "description": "string", "clientName": "string" }',
        responseBody: '{ "project": { "id": "string", "name": "string", "createdAt": "date" } }',
      },
      {
        method: 'GET',
        path: '/api/projects/:id',
        description: 'Get project details by ID',
        auth: true,
        responseBody: '{ "project": { "id": "string", "name": "string", "estimates": [] } }',
      },
      {
        method: 'POST',
        path: '/api/estimates',
        description: 'Create new estimation',
        auth: true,
        requestBody: '{ "projectId": "string", "method": "string", "data": {} }',
        responseBody: '{ "estimate": { "id": "string", "totalHours": "number", "confidence": "number" } }',
      },
      {
        method: 'POST',
        path: '/api/estimates/:id/export',
        description: 'Export estimate to Excel/PowerPoint',
        auth: true,
        requestBody: '{ "format": "xlsx|pptx" }',
        responseBody: '{ "downloadUrl": "string", "expiresAt": "date" }',
      },
    ];

    const sampleTables: DatabaseTable[] = [
      {
        name: 'users',
        columns: [
          { name: 'id', type: 'UUID', nullable: false, primaryKey: true },
          { name: 'email', type: 'VARCHAR(255)', nullable: false, primaryKey: false },
          { name: 'password_hash', type: 'VARCHAR(255)', nullable: false, primaryKey: false },
          { name: 'full_name', type: 'VARCHAR(255)', nullable: false, primaryKey: false },
          { name: 'role', type: 'ENUM', nullable: false, primaryKey: false },
          { name: 'created_at', type: 'TIMESTAMP', nullable: false, primaryKey: false },
          { name: 'updated_at', type: 'TIMESTAMP', nullable: false, primaryKey: false },
        ],
      },
      {
        name: 'projects',
        columns: [
          { name: 'id', type: 'UUID', nullable: false, primaryKey: true },
          { name: 'name', type: 'VARCHAR(255)', nullable: false, primaryKey: false },
          { name: 'description', type: 'TEXT', nullable: true, primaryKey: false },
          { name: 'client_name', type: 'VARCHAR(255)', nullable: true, primaryKey: false },
          { name: 'status', type: 'ENUM', nullable: false, primaryKey: false },
          { name: 'owner_id', type: 'UUID', nullable: false, primaryKey: false, foreignKey: 'users.id' },
          { name: 'created_at', type: 'TIMESTAMP', nullable: false, primaryKey: false },
        ],
      },
      {
        name: 'estimates',
        columns: [
          { name: 'id', type: 'UUID', nullable: false, primaryKey: true },
          { name: 'project_id', type: 'UUID', nullable: false, primaryKey: false, foreignKey: 'projects.id' },
          { name: 'method', type: 'VARCHAR(50)', nullable: false, primaryKey: false },
          { name: 'total_hours', type: 'DECIMAL(10,2)', nullable: false, primaryKey: false },
          { name: 'confidence', type: 'DECIMAL(5,2)', nullable: false, primaryKey: false },
          { name: 'breakdown', type: 'JSONB', nullable: false, primaryKey: false },
          { name: 'created_at', type: 'TIMESTAMP', nullable: false, primaryKey: false },
        ],
      },
    ];

    setApiEndpoints(sampleEndpoints);
    setDbTables(sampleTables);
    generateDiagrams(sampleEndpoints, sampleTables);
    generateAPISpec(sampleEndpoints);
    generateDatabaseSchema(sampleTables);
  };

  const generateDiagrams = (endpoints: APIEndpoint[], tables: DatabaseTable[]) => {
    // Generate Class Diagram
    const classDiag = `
classDiagram
    class User {
        -UUID id
        -String email
        -String passwordHash
        -String fullName
        -Role role
        -DateTime createdAt
        +login(email, password) Token
        +register(userData) User
        +logout() void
        +updateProfile(data) User
    }
    
    class Project {
        -UUID id
        -String name
        -String description
        -String clientName
        -Status status
        -UUID ownerId
        -DateTime createdAt
        +create(projectData) Project
        +update(projectData) Project
        +delete() boolean
        +addEstimate(estimate) Estimate
        +getEstimates() List~Estimate~
    }
    
    class Estimate {
        -UUID id
        -UUID projectId
        -String method
        -Decimal totalHours
        -Decimal confidence
        -JSON breakdown
        -DateTime createdAt
        +calculate(inputData) EstimateResult
        +export(format) File
        +getBreakdown() JSON
        +updateConfidence(value) void
    }
    
    class EstimationService {
        <<service>>
        +calculatePERT(data) Result
        +calculateStoryPoints(data) Result
        +calculateFPA(data) Result
        +applyRiskFactors(result) Result
    }
    
    class ReportService {
        <<service>>
        +generateExcel(estimate) File
        +generatePowerPoint(estimate) File
        +generatePDF(estimate) File
    }
    
    class AuthService {
        <<service>>
        +authenticate(credentials) Token
        +validateToken(token) User
        +refreshToken(token) Token
        +revokeToken(token) boolean
    }
    
    User "1" --> "*" Project : owns
    Project "1" --> "*" Estimate : contains
    EstimationService ..> Estimate : calculates
    ReportService ..> Estimate : exports
    AuthService ..> User : authenticates
`;
    setClassDiagram(classDiag);

    // Generate Sequence Diagram
    const seqDiag = `
sequenceDiagram
    actor User
    participant UI as React UI
    participant API as API Gateway
    participant Auth as Auth Service
    participant Project as Project Service
    participant Est as Estimation Service
    participant DB as Database
    participant Cache as Redis Cache
    
    User->>UI: Click "Create Estimate"
    UI->>UI: Show estimation wizard
    User->>UI: Fill estimation data
    UI->>API: POST /api/estimates
    
    API->>Auth: Validate JWT token
    Auth->>Cache: Check token
    Cache-->>Auth: Token valid
    Auth-->>API: User authenticated
    
    API->>Est: Calculate estimate
    Est->>Est: Apply PERT algorithm
    Est->>Est: Calculate risk factors
    Est->>Est: Generate breakdown
    
    Est->>DB: Save estimate
    DB-->>Est: Estimate saved
    
    Est->>Cache: Cache estimate result
    Cache-->>Est: Cached
    
    Est-->>API: Estimation result
    API-->>UI: Success response
    UI->>UI: Show results
    UI-->>User: Display calculations
    
    User->>UI: Click "Export Excel"
    UI->>API: POST /api/estimates/:id/export
    API->>Project: Generate Excel report
    Project->>DB: Fetch estimate data
    DB-->>Project: Estimate data
    Project->>Project: Generate XLSX file
    Project->>Project: Upload to S3
    Project-->>API: Download URL
    API-->>UI: File URL
    UI-->>User: Download file
`;
    setSequenceDiagram(seqDiag);
  };

  const generateAPISpec = (endpoints: APIEndpoint[]) => {
    const spec = `# API Specification
## Estimation Platform REST API v1.0

### Base URL
\`\`\`
Production: https://api.estimation-platform.com/v1
Staging: https://api-staging.estimation-platform.com/v1
Development: http://localhost:3000/api/v1
\`\`\`

### Authentication
All authenticated endpoints require a Bearer token in the Authorization header:
\`\`\`
Authorization: Bearer <JWT_TOKEN>
\`\`\`

### Response Format
All responses follow this structure:
\`\`\`json
{
  "success": boolean,
  "data": object,
  "error": {
    "code": "string",
    "message": "string"
  }
}
\`\`\`

### Error Codes
- \`400\` - Bad Request
- \`401\` - Unauthorized
- \`403\` - Forbidden
- \`404\` - Not Found
- \`422\` - Validation Error
- \`500\` - Internal Server Error

---

## Endpoints

${endpoints.map((endpoint, idx) => `
### ${idx + 1}. ${endpoint.method} ${endpoint.path}

**Description:** ${endpoint.description}

**Authentication:** ${endpoint.auth ? '✅ Required' : '❌ Not Required'}

${endpoint.requestBody ? `**Request Body:**
\`\`\`json
${endpoint.requestBody}
\`\`\`` : ''}

${endpoint.responseBody ? `**Response:**
\`\`\`json
${endpoint.responseBody}
\`\`\`` : ''}

**Status Codes:**
- \`200\` - Success
- \`${endpoint.method === 'POST' ? '201 - Created' : '200 - OK'}\`
- \`400\` - Bad Request
- \`${endpoint.auth ? '401 - Unauthorized' : ''}\`
- \`500\` - Server Error

---
`).join('\n')}

## Rate Limiting
- **Authenticated users:** 1000 requests per hour
- **Unauthenticated:** 100 requests per hour
- Rate limit headers included in response:
  - \`X-RateLimit-Limit\`
  - \`X-RateLimit-Remaining\`
  - \`X-RateLimit-Reset\`

## Pagination
For list endpoints, use query parameters:
- \`page\`: Page number (default: 1)
- \`limit\`: Items per page (default: 20, max: 100)

Example: \`GET /api/projects?page=2&limit=50\`

## Filtering & Sorting
- \`filter[field]\`: Filter by field value
- \`sort\`: Sort by field (prefix with \`-\` for descending)

Example: \`GET /api/projects?filter[status]=active&sort=-createdAt\`

## Webhook Events
Subscribe to events via webhook configuration:
- \`project.created\`
- \`project.updated\`
- \`estimate.created\`
- \`estimate.exported\`
- \`user.registered\`
`;

    setApiSpec(spec);
  };

  const generateDatabaseSchema = (tables: DatabaseTable[]) => {
    const sql = tables.map(table => {
      const columns = table.columns.map(col => {
        const parts = [
          `  ${col.name}`,
          col.type,
          col.primaryKey ? 'PRIMARY KEY' : '',
          !col.nullable ? 'NOT NULL' : '',
          col.name === 'created_at' || col.name === 'updated_at' ? 'DEFAULT CURRENT_TIMESTAMP' : '',
        ].filter(Boolean).join(' ');
        return parts;
      }).join(',\n');

      const foreignKeys = table.columns
        .filter(col => col.foreignKey)
        .map(col => `  FOREIGN KEY (${col.name}) REFERENCES ${col.foreignKey}`)
        .join(',\n');

      return `CREATE TABLE ${table.name} (
${columns}${foreignKeys ? ',\n' + foreignKeys : ''}
);

CREATE INDEX idx_${table.name}_created_at ON ${table.name}(created_at);
${table.columns.find(col => col.foreignKey) ? `CREATE INDEX idx_${table.name}_foreign_keys ON ${table.name}(${table.columns.filter(c => c.foreignKey).map(c => c.name).join(', ')});` : ''}
`;
    }).join('\n\n');

    setDbSchemaSQL(`-- Database Schema for Estimation Platform
-- PostgreSQL 14+
-- Generated: ${new Date().toISOString()}

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create ENUM types
CREATE TYPE user_role AS ENUM ('admin', 'manager', 'developer', 'client');
CREATE TYPE project_status AS ENUM ('draft', 'active', 'completed', 'archived');

${sql}

-- Add indexes for performance
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_projects_owner_status ON projects(owner_id, status);
CREATE INDEX idx_estimates_project ON estimates(project_id);

-- Add constraints
ALTER TABLE users ADD CONSTRAINT uk_users_email UNIQUE (email);
ALTER TABLE projects ADD CONSTRAINT chk_project_name_length CHECK (length(name) >= 3);
ALTER TABLE estimates ADD CONSTRAINT chk_estimate_hours_positive CHECK (total_hours > 0);
ALTER TABLE estimates ADD CONSTRAINT chk_confidence_range CHECK (confidence >= 0 AND confidence <= 100);

-- Sample data for testing
INSERT INTO users (id, email, password_hash, full_name, role, created_at, updated_at)
VALUES 
  (uuid_generate_v4(), 'admin@esip.com', '$2b$10$...', 'Admin User', 'admin', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
  (uuid_generate_v4(), 'manager@esip.com', '$2b$10$...', 'Manager User', 'manager', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

-- End of schema
`);
  };

  const exportAPISpec = () => {
    const blob = new Blob([apiSpec], { type: 'text/markdown' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'API_Specification.md';
    a.click();
    toast.success('API specification exported');
  };

  const exportDBSchema = () => {
    const blob = new Blob([dbSchemaSQL], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'database_schema.sql';
    a.click();
    toast.success('Database schema exported');
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold">Low-Level Design (LLD)</h2>
          <p className="text-muted-foreground">Detailed technical specifications, API docs, and database schema</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={exportAPISpec}>
            <Download className="w-4 h-4 mr-2" />
            Export API Spec
          </Button>
          <Button onClick={exportDBSchema}>
            <FileText className="w-4 h-4 mr-2" />
            Export DB Schema
          </Button>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-3 gap-4">
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Webhook className="w-4 h-4" />
              API Endpoints
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold">{apiEndpoints.length}</div>
            <p className="text-xs text-muted-foreground mt-1">RESTful endpoints defined</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Database className="w-4 h-4" />
              Database Tables
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold">{dbTables.length}</div>
            <p className="text-xs text-muted-foreground mt-1">Tables in schema</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Code2 className="w-4 h-4" />
              Total Columns
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold">
              {dbTables.reduce((sum, t) => sum + t.columns.length, 0)}
            </div>
            <p className="text-xs text-muted-foreground mt-1">Database columns</p>
          </CardContent>
        </Card>
      </div>

      {/* Diagrams & Specs Tabs */}
      <Tabs defaultValue="class" className="w-full">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="class">Class Diagram</TabsTrigger>
          <TabsTrigger value="sequence">Sequence</TabsTrigger>
          <TabsTrigger value="api">API Spec</TabsTrigger>
          <TabsTrigger value="database">DB Schema</TabsTrigger>
        </TabsList>

        <TabsContent value="class">
          <Card>
            <CardHeader>
              <CardTitle>Class Diagram</CardTitle>
              <CardDescription>Detailed class structure with methods and relationships</CardDescription>
            </CardHeader>
            <CardContent>
              <MermaidDiagram chart={classDiagram} id="class-diagram" />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="sequence">
          <Card>
            <CardHeader>
              <CardTitle>Sequence Diagram</CardTitle>
              <CardDescription>Detailed interaction flow for estimate creation and export</CardDescription>
            </CardHeader>
            <CardContent>
              <MermaidDiagram chart={sequenceDiagram} id="sequence-diagram" />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="api" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>API Endpoints</CardTitle>
              <CardDescription>Complete REST API specification</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {apiEndpoints.map((endpoint, idx) => (
                  <Card key={idx} className="border-l-4 border-l-blue-500">
                    <CardContent className="pt-4">
                      <div className="flex items-start justify-between mb-2">
                        <div className="flex items-center gap-3">
                          <Badge 
                            variant={endpoint.method === 'GET' ? 'default' : 'secondary'}
                            className={
                              endpoint.method === 'GET' ? 'bg-blue-500' :
                              endpoint.method === 'POST' ? 'bg-green-500' :
                              endpoint.method === 'PUT' ? 'bg-yellow-500' :
                              endpoint.method === 'DELETE' ? 'bg-red-500' : ''
                            }
                          >
                            {endpoint.method}
                          </Badge>
                          <code className="text-sm font-mono">{endpoint.path}</code>
                        </div>
                        {endpoint.auth && (
                          <Badge variant="outline" className="text-xs">🔒 Auth Required</Badge>
                        )}
                      </div>
                      <p className="text-sm text-muted-foreground">{endpoint.description}</p>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Full API Documentation</CardTitle>
            </CardHeader>
            <CardContent>
              <pre className="text-xs bg-muted p-4 rounded overflow-auto max-h-[400px] whitespace-pre-wrap">
                {apiSpec}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="database" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Database Tables</CardTitle>
              <CardDescription>Schema structure with columns and relationships</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {dbTables.map((table, idx) => (
                  <Card key={idx}>
                    <CardHeader className="pb-3">
                      <CardTitle className="text-base">{table.name}</CardTitle>
                    </CardHeader>
                    <CardContent>
                      <div className="overflow-auto">
                        <table className="w-full text-sm">
                          <thead>
                            <tr className="border-b">
                              <th className="text-left py-2">Column</th>
                              <th className="text-left py-2">Type</th>
                              <th className="text-left py-2">Constraints</th>
                            </tr>
                          </thead>
                          <tbody>
                            {table.columns.map((col, colIdx) => (
                              <tr key={colIdx} className="border-b">
                                <td className="py-2 font-mono">{col.name}</td>
                                <td className="py-2">{col.type}</td>
                                <td className="py-2">
                                  <div className="flex gap-1 flex-wrap">
                                    {col.primaryKey && <Badge variant="default" className="text-xs">PK</Badge>}
                                    {!col.nullable && <Badge variant="secondary" className="text-xs">NOT NULL</Badge>}
                                    {col.foreignKey && <Badge variant="outline" className="text-xs">FK</Badge>}
                                  </div>
                                </td>
                              </tr>
                            ))}
                          </tbody>
                        </table>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>SQL Schema</CardTitle>
              <CardDescription>Complete PostgreSQL schema with indexes and constraints</CardDescription>
            </CardHeader>
            <CardContent>
              <pre className="text-xs bg-muted p-4 rounded overflow-auto max-h-[400px] font-mono whitespace-pre-wrap">
                {dbSchemaSQL}
              </pre>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
