import React, { useState, useEffect } from 'react';
import { Download, FileText, Plus, Trash2, Edit2 } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { MermaidDiagram } from './MermaidDiagram';
import { toast } from 'sonner';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '../ui/dialog';

interface Entity {
  id: string;
  name: string;
  attributes: string[];
}

interface Relationship {
  from: string;
  to: string;
  type: '1:1' | '1:N' | 'N:M';
  label: string;
}

export function DFAGenerator() {
  const [entities, setEntities] = useState<Entity[]>([]);
  const [relationships, setRelationships] = useState<Relationship[]>([]);
  const [erdDiagram, setErdDiagram] = useState('');
  const [dfdLevel0, setDfdLevel0] = useState('');
  const [dfdLevel1, setDfdLevel1] = useState('');
  const [apiDataFlow, setApiDataFlow] = useState('');
  const [dbSchema, setDbSchema] = useState('');
  
  // Dialog state
  const [isAddEntityOpen, setIsAddEntityOpen] = useState(false);
  const [newEntity, setNewEntity] = useState({ name: '', attributes: '' });

  useEffect(() => {
    loadSampleData();
  }, []);

  const loadSampleData = () => {
    const sampleEntities: Entity[] = [
      {
        id: 'user',
        name: 'User',
        attributes: ['id', 'email', 'password', 'fullName', 'role', 'createdAt'],
      },
      {
        id: 'project',
        name: 'Project',
        attributes: ['id', 'name', 'description', 'status', 'ownerId', 'createdAt'],
      },
      {
        id: 'estimate',
        name: 'Estimate',
        attributes: ['id', 'projectId', 'method', 'totalHours', 'confidence', 'createdAt'],
      },
      {
        id: 'team',
        name: 'Team',
        attributes: ['id', 'name', 'description', 'createdAt'],
      },
    ];

    const sampleRelationships: Relationship[] = [
      { from: 'user', to: 'project', type: '1:N', label: 'owns' },
      { from: 'project', to: 'estimate', type: '1:N', label: 'has' },
      { from: 'user', to: 'team', type: 'N:M', label: 'member of' },
    ];

    setEntities(sampleEntities);
    setRelationships(sampleRelationships);
    generateDiagrams(sampleEntities, sampleRelationships);
  };

  const generateDiagrams = (entitiesData: Entity[], relationshipsData: Relationship[]) => {
    // Generate ERD
    const erd = `erDiagram
    ${entitiesData.map(entity => `${entity.name} {
        ${entity.attributes.map(attr => `string ${attr}`).join('\n        ')}
    }`).join('\n    ')}
    ${relationshipsData.map(rel => {
      const fromEntity = entitiesData.find(e => e.id === rel.from)?.name;
      const toEntity = entitiesData.find(e => e.id === rel.to)?.name;
      const relationSymbol = rel.type === '1:1' ? '||--||' : rel.type === '1:N' ? '||--o{' : '}o--o{';
      return `${fromEntity} ${relationSymbol} ${toEntity} : "${rel.label}"`;
    }).join('\n    ')}`;
    setErdDiagram(erd);

    // Generate DFD Level 0 (Context Diagram)
    const level0 = `graph TB
    User((User/Admin)) -->|Login/Register| System[Estimation Platform]
    System -->|Estimates/Reports| User
    Client((Client)) -->|View Reports| System
    System -->|Deliverables| Client
    System -->|Store Data| DB[(Database)]
    DB -->|Retrieve Data| System
    System -->|Send Notifications| Email[Email Service]
    System -->|Export Files| Storage[File Storage]`;
    setDfdLevel0(level0);

    // Generate DFD Level 1
    const level1 = `graph TB
    subgraph "Authentication"
        A1[Login Process] -->|Validate| A2[Auth Service]
        A2 -->|Token| A3[Session Store]
    end
    subgraph "Project Management"
        P1[Create Project] -->|Project Data| P2[Project Service]
        P2 -->|Save| P3[(Projects DB)]
    end
    subgraph "Estimation"
        E1[Input Data] -->|Scope| E2[Calculation Engine]
        E2 -->|Results| E3[Estimate Store]
        E3 -->|Retrieve| E4[Report Generator]
    end
    subgraph "Export"
        X1[Generate Excel] -->|Data| X2[Export Service]
        X2 -->|Files| X3[File Storage]
    end
    A3 --> P1
    P3 --> E1
    E3 --> X1`;
    setDfdLevel1(level1);

    // Generate API Data Flow
    const apiFlow = `sequenceDiagram
    participant C as Client
    participant A as API Gateway
    participant AS as Auth Service
    participant PS as Project Service
    participant ES as Estimation Service
    participant DB as Database
    C->>A: POST /api/login
    A->>AS: Validate Credentials
    AS->>DB: Query User
    DB-->>AS: User Data
    AS-->>A: JWT Token
    A-->>C: Auth Response
    C->>A: POST /api/projects
    A->>AS: Verify Token
    AS-->>A: User Validated
    A->>PS: Create Project
    PS->>DB: Insert Project
    DB-->>PS: Project ID
    PS-->>A: Success
    A-->>C: Project Created
    C->>A: POST /api/estimates
    A->>ES: Calculate Estimate
    ES->>DB: Save Estimate
    ES->>DB: Fetch Related Data
    DB-->>ES: Data
    ES-->>A: Estimate Results
    A-->>C: Calculation Response`;
    setApiDataFlow(apiFlow);

    // Generate Database Schema
    const schema = `classDiagram
    class User {
        +UUID id
        +String email
        +String passwordHash
        +String fullName
        +Enum role
        +DateTime createdAt
        +login()
        +logout()
    }
    class Project {
        +UUID id
        +String name
        +String description
        +Enum status
        +UUID ownerId
        +DateTime createdAt
        +addEstimate()
        +getMembers()
    }
    class Estimate {
        +UUID id
        +UUID projectId
        +String method
        +Float totalHours
        +Float confidence
        +JSON breakdown
        +DateTime createdAt
        +calculate()
        +export()
    }
    class Team {
        +UUID id
        +String name
        +String description
        +DateTime createdAt
        +addMember()
        +removeMember()
    }
    class TeamMember {
        +UUID teamId
        +UUID userId
        +Enum role
        +DateTime joinedAt
    }
    User "1" --> "*" Project : owns
    Project "1" --> "*" Estimate : has
    User "*" --> "*" Team : member_of
    Team "1" --> "*" TeamMember : has
    User "1" --> "*" TeamMember : is`;
    setDbSchema(schema);
  };

  const addEntity = () => {
    if (!newEntity.name) {
      toast.error('Entity name is required');
      return;
    }

    const entity: Entity = {
      id: newEntity.name.toLowerCase().replace(/\s+/g, '_'),
      name: newEntity.name,
      attributes: newEntity.attributes.split(',').map(attr => attr.trim()).filter(Boolean),
    };

    const updatedEntities = [...entities, entity];
    setEntities(updatedEntities);
    generateDiagrams(updatedEntities, relationships);
    
    setNewEntity({ name: '', attributes: '' });
    setIsAddEntityOpen(false);
    toast.success('Entity added successfully');
  };

  const deleteEntity = (entityId: string) => {
    const updatedEntities = entities.filter(e => e.id !== entityId);
    const updatedRelationships = relationships.filter(
      r => r.from !== entityId && r.to !== entityId
    );
    
    setEntities(updatedEntities);
    setRelationships(updatedRelationships);
    generateDiagrams(updatedEntities, updatedRelationships);
    toast.success('Entity deleted');
  };

  const exportToPDF = () => {
    toast.info('PDF export will be implemented');
  };

  const exportToSQL = () => {
    // Generate SQL CREATE statements
    const sqlStatements = entities.map(entity => {
      const columns = entity.attributes.map(attr => `  ${attr} VARCHAR(255)`).join(',\n');
      return `CREATE TABLE ${entity.name.toLowerCase()}s (\n${columns}\n);`;
    }).join('\n\n');

    // Create downloadable file
    const blob = new Blob([sqlStatements], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'database_schema.sql';
    a.click();
    
    toast.success('SQL schema exported');
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold">Data Flow Architecture (DFA)</h2>
          <p className="text-muted-foreground">Generate data models, ERD, and data flow diagrams</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={exportToSQL}>
            <Download className="w-4 h-4 mr-2" />
            Export SQL
          </Button>
          <Button onClick={exportToPDF}>
            <FileText className="w-4 h-4 mr-2" />
            Export PDF
          </Button>
        </div>
      </div>

      {/* Entity Management */}
      <Card>
        <CardHeader>
          <div className="flex justify-between items-center">
            <div>
              <CardTitle>Database Entities</CardTitle>
              <CardDescription>Manage your data entities and attributes</CardDescription>
            </div>
            <Dialog open={isAddEntityOpen} onOpenChange={setIsAddEntityOpen}>
              <DialogTrigger asChild>
                <Button>
                  <Plus className="w-4 h-4 mr-2" />
                  Add Entity
                </Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>Add New Entity</DialogTitle>
                  <DialogDescription>Define a new database entity with its attributes</DialogDescription>
                </DialogHeader>
                <div className="space-y-4">
                  <div>
                    <Label htmlFor="entityName">Entity Name</Label>
                    <Input
                      id="entityName"
                      value={newEntity.name}
                      onChange={(e) => setNewEntity({ ...newEntity, name: e.target.value })}
                      placeholder="e.g., Customer, Order, Product"
                    />
                  </div>
                  <div>
                    <Label htmlFor="entityAttributes">Attributes (comma-separated)</Label>
                    <Input
                      id="entityAttributes"
                      value={newEntity.attributes}
                      onChange={(e) => setNewEntity({ ...newEntity, attributes: e.target.value })}
                      placeholder="e.g., id, name, email, createdAt"
                    />
                  </div>
                </div>
                <DialogFooter>
                  <Button variant="outline" onClick={() => setIsAddEntityOpen(false)}>
                    Cancel
                  </Button>
                  <Button onClick={addEntity}>Add Entity</Button>
                </DialogFooter>
              </DialogContent>
            </Dialog>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {entities.map((entity) => (
              <Card key={entity.id} className="relative">
                <CardHeader className="pb-3">
                  <div className="flex justify-between items-start">
                    <CardTitle className="text-base">{entity.name}</CardTitle>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => deleteEntity(entity.id)}
                      className="h-6 w-6 p-0 text-destructive"
                    >
                      <Trash2 className="w-3 h-3" />
                    </Button>
                  </div>
                </CardHeader>
                <CardContent className="pt-0">
                  <ul className="text-sm space-y-1 text-muted-foreground">
                    {entity.attributes.slice(0, 5).map((attr, idx) => (
                      <li key={idx}>• {attr}</li>
                    ))}
                    {entity.attributes.length > 5 && (
                      <li className="text-xs italic">+{entity.attributes.length - 5} more</li>
                    )}
                  </ul>
                </CardContent>
              </Card>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Diagram Tabs */}
      <Tabs defaultValue="erd" className="w-full">
        <TabsList className="grid w-full grid-cols-5">
          <TabsTrigger value="erd">ERD</TabsTrigger>
          <TabsTrigger value="dfd-0">DFD Level 0</TabsTrigger>
          <TabsTrigger value="dfd-1">DFD Level 1</TabsTrigger>
          <TabsTrigger value="api">API Flow</TabsTrigger>
          <TabsTrigger value="schema">DB Schema</TabsTrigger>
        </TabsList>

        <TabsContent value="erd" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Entity Relationship Diagram (ERD)</CardTitle>
              <CardDescription>Visual representation of database entities and their relationships</CardDescription>
            </CardHeader>
            <CardContent>
              <MermaidDiagram chart={erdDiagram} id="erd-diagram" />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="dfd-0" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Data Flow Diagram - Level 0 (Context Diagram)</CardTitle>
              <CardDescription>High-level view of system interactions with external entities</CardDescription>
            </CardHeader>
            <CardContent>
              <MermaidDiagram chart={dfdLevel0} id="dfd-level-0" />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="dfd-1" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Data Flow Diagram - Level 1</CardTitle>
              <CardDescription>Detailed view of major processes and data flows</CardDescription>
            </CardHeader>
            <CardContent>
              <MermaidDiagram chart={dfdLevel1} id="dfd-level-1" />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="api" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>API Data Flow</CardTitle>
              <CardDescription>Sequence diagram showing API request/response flow</CardDescription>
            </CardHeader>
            <CardContent>
              <MermaidDiagram chart={apiDataFlow} id="api-data-flow" />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="schema" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Database Schema (Class Diagram)</CardTitle>
              <CardDescription>Detailed database schema with data types and relationships</CardDescription>
            </CardHeader>
            <CardContent>
              <MermaidDiagram chart={dbSchema} id="db-schema" />
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Summary Stats */}
      <div className="grid grid-cols-3 gap-4">
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium">Total Entities</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold">{entities.length}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium">Total Relationships</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold">{relationships.length}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium">Total Attributes</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold">
              {entities.reduce((sum, e) => sum + e.attributes.length, 0)}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}