import React, { useState, useEffect } from 'react';
import { Download, FileText, Settings, Shield, Cloud, Layers } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import { MermaidDiagram } from './MermaidDiagram';
import { toast } from 'sonner';
import { Textarea } from '../ui/textarea';

interface ArchConfig {
  projectName: string;
  techStack: {
    frontend: string[];
    backend: string[];
    database: string[];
    infrastructure: string[];
  };
  requirements: {
    systemType: string;
    expectedUsers: string;
    securityLevel: string;
    deploymentModel: string;
  };
}

export function HLDGenerator() {
  const [config, setConfig] = useState<ArchConfig | null>(null);
  const [systemArchDiagram, setSystemArchDiagram] = useState('');
  const [deploymentDiagram, setDeploymentDiagram] = useState('');
  const [securityDiagram, setSecurityDiagram] = useState('');
  const [componentDiagram, setComponentDiagram] = useState('');
  const [hldDocument, setHldDocument] = useState('');

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = () => {
    const savedConfig = localStorage.getItem('architecture-config');
    if (savedConfig) {
      const parsedConfig = JSON.parse(savedConfig);
      setConfig(parsedConfig);
      generateDiagrams(parsedConfig);
      generateHLDDocument(parsedConfig);
    } else {
      // Use sample config
      const sampleConfig: ArchConfig = {
        projectName: 'Estimation Platform',
        techStack: {
          frontend: ['React', 'TypeScript'],
          backend: ['Node.js', 'Express'],
          database: ['PostgreSQL', 'Redis'],
          infrastructure: ['AWS', 'Docker'],
        },
        requirements: {
          systemType: 'web',
          expectedUsers: '1000-10000',
          securityLevel: 'high',
          deploymentModel: 'cloud',
        },
      };
      setConfig(sampleConfig);
      generateDiagrams(sampleConfig);
      generateHLDDocument(sampleConfig);
    }
  };

  const generateDiagrams = (cfg: ArchConfig) => {
    // System Architecture Diagram
    const sysArch = `
graph TB
    subgraph "Client Layer"
        Web[Web Browser]
        Mobile[Mobile App]
    end
    
    subgraph "CDN & Load Balancer"
        CDN[CloudFront CDN]
        LB[Application Load Balancer]
    end
    
    subgraph "Application Layer"
        API1[API Server 1]
        API2[API Server 2]
        API3[API Server 3]
    end
    
    subgraph "Service Layer"
        Auth[Auth Service]
        Project[Project Service]
        Estimation[Estimation Service]
        Report[Report Service]
    end
    
    subgraph "Data Layer"
        PrimaryDB[(Primary DB<br/>PostgreSQL)]
        ReplicaDB[(Replica DB<br/>Read Only)]
        Cache[(Redis Cache)]
        FileStore[S3 File Storage]
    end
    
    subgraph "External Services"
        Email[Email Service]
        Analytics[Analytics]
        Monitoring[Monitoring]
    end
    
    Web --> CDN
    Mobile --> CDN
    CDN --> LB
    LB --> API1
    LB --> API2
    LB --> API3
    
    API1 --> Auth
    API1 --> Project
    API1 --> Estimation
    API1 --> Report
    
    Auth --> Cache
    Auth --> PrimaryDB
    Project --> PrimaryDB
    Project --> Cache
    Estimation --> PrimaryDB
    Report --> FileStore
    
    PrimaryDB --> ReplicaDB
    
    API1 -.->|Send Emails| Email
    API1 -.->|Track Events| Analytics
    API1 -.->|Logs/Metrics| Monitoring
`;
    setSystemArchDiagram(sysArch);

    // Deployment Architecture
    const deployment = `
graph TB
    subgraph "AWS Cloud"
        subgraph "VPC"
            subgraph "Public Subnet"
                ALB[Application<br/>Load Balancer]
                NAT[NAT Gateway]
            end
            
            subgraph "Private Subnet - AZ1"
                API1[API Server 1]
                APP1[App Server 1]
            end
            
            subgraph "Private Subnet - AZ2"
                API2[API Server 2]
                APP2[App Server 2]
            end
            
            subgraph "Database Subnet"
                RDS_Primary[(RDS Primary)]
                RDS_Replica[(RDS Replica)]
            end
        end
        
        subgraph "Managed Services"
            S3[S3 Bucket]
            CloudFront[CloudFront CDN]
            ElastiCache[ElastiCache<br/>Redis]
            SES[Simple Email Service]
        end
        
        subgraph "Monitoring"
            CloudWatch[CloudWatch]
            XRay[X-Ray]
        end
    end
    
    Internet((Internet)) --> CloudFront
    CloudFront --> ALB
    ALB --> API1
    ALB --> API2
    
    API1 --> APP1
    API2 --> APP2
    
    APP1 --> RDS_Primary
    APP2 --> RDS_Primary
    APP1 --> ElastiCache
    APP2 --> ElastiCache
    
    RDS_Primary --> RDS_Replica
    
    APP1 --> S3
    APP2 --> S3
    APP1 -.-> SES
    
    API1 -.-> CloudWatch
    API2 -.-> CloudWatch
    APP1 -.-> XRay
`;
    setDeploymentDiagram(deployment);

    // Security Architecture
    const security = `
graph TB
    subgraph "Security Layers"
        subgraph "Perimeter Security"
            WAF[WAF<br/>Web Application Firewall]
            DDoS[DDoS Protection<br/>AWS Shield]
        end
        
        subgraph "Network Security"
            SG1[Security Group<br/>Public Subnet]
            SG2[Security Group<br/>Private Subnet]
            NACL[Network ACL]
        end
        
        subgraph "Application Security"
            Auth[JWT Authentication]
            RBAC[Role-Based Access Control]
            RateLimit[Rate Limiting]
            InputVal[Input Validation]
        end
        
        subgraph "Data Security"
            Encryption[Data Encryption<br/>at Rest & Transit]
            Backup[Automated Backups]
            Secrets[Secrets Manager]
        end
        
        subgraph "Monitoring & Compliance"
            IDS[Intrusion Detection]
            Audit[Audit Logging]
            Compliance[Compliance Checks]
        end
    end
    
    User((User)) --> WAF
    WAF --> DDoS
    DDoS --> SG1
    SG1 --> Auth
    Auth --> RBAC
    RBAC --> SG2
    SG2 --> Encryption
    
    Auth -.-> Secrets
    Encryption -.-> Backup
    RBAC -.-> Audit
    
    InputVal --> RateLimit
    Audit --> IDS
    IDS --> Compliance
`;
    setSecurityDiagram(security);

    // Component Architecture
    const components = `
graph TB
    subgraph "Frontend Components"
        UI[UI Components]
        State[State Management<br/>Zustand]
        Router[React Router]
        API_Client[API Client]
    end
    
    subgraph "Backend Components"
        Gateway[API Gateway]
        Middleware[Middleware Layer]
        
        subgraph "Business Logic"
            AuthCtrl[Auth Controller]
            ProjectCtrl[Project Controller]
            EstimateCtrl[Estimate Controller]
            ReportCtrl[Report Controller]
        end
        
        subgraph "Services"
            AuthSvc[Auth Service]
            ProjectSvc[Project Service]
            EstimateSvc[Estimation Engine]
            ReportSvc[Report Generator]
        end
        
        subgraph "Data Access"
            ORM[Prisma ORM]
            CacheLayer[Cache Manager]
        end
    end
    
    UI --> State
    State --> Router
    Router --> API_Client
    API_Client --> Gateway
    
    Gateway --> Middleware
    Middleware --> AuthCtrl
    Middleware --> ProjectCtrl
    Middleware --> EstimateCtrl
    Middleware --> ReportCtrl
    
    AuthCtrl --> AuthSvc
    ProjectCtrl --> ProjectSvc
    EstimateCtrl --> EstimateSvc
    ReportCtrl --> ReportSvc
    
    AuthSvc --> ORM
    ProjectSvc --> ORM
    EstimateSvc --> ORM
    ReportSvc --> ORM
    
    AuthSvc --> CacheLayer
    ProjectSvc --> CacheLayer
    
    ORM --> DB[(Database)]
    CacheLayer --> Cache[(Redis)]
`;
    setComponentDiagram(components);
  };

  const generateHLDDocument = (cfg: ArchConfig) => {
    const doc = `# High-Level Design Document
## ${cfg.projectName}

---

## 1. Executive Summary

This document provides a comprehensive high-level design for the ${cfg.projectName}. The system is designed to be scalable, secure, and maintainable, following industry best practices and modern architectural patterns.

**System Type:** ${cfg.requirements.systemType}
**Expected Users:** ${cfg.requirements.expectedUsers}
**Security Level:** ${cfg.requirements.securityLevel}
**Deployment Model:** ${cfg.requirements.deploymentModel}

---

## 2. Technology Stack

### Frontend
${cfg.techStack.frontend.map(tech => `- ${tech}`).join('\n')}

### Backend
${cfg.techStack.backend.map(tech => `- ${tech}`).join('\n')}

### Database
${cfg.techStack.database.map(tech => `- ${tech}`).join('\n')}

### Infrastructure
${cfg.techStack.infrastructure.map(tech => `- ${tech}`).join('\n')}

---

## 3. System Architecture Overview

The system follows a **microservices-based architecture** with clear separation of concerns:

### 3.1 Client Layer
- Web application (React/TypeScript)
- Responsive design for desktop and mobile
- Progressive Web App (PWA) capabilities

### 3.2 API Gateway Layer
- Centralized request routing
- Authentication and authorization
- Rate limiting and throttling
- Request/response transformation

### 3.3 Service Layer
- **Authentication Service**: User management, JWT tokens, session handling
- **Project Service**: Project CRUD operations, team management
- **Estimation Service**: Calculation engine, multiple estimation methods
- **Report Service**: Excel and PowerPoint generation, export functionality

### 3.4 Data Layer
- **Primary Database**: PostgreSQL for transactional data
- **Read Replicas**: For read-heavy operations
- **Cache Layer**: Redis for session management and performance
- **File Storage**: S3 for generated documents and assets

---

## 4. Key Design Decisions

### 4.1 Scalability
- **Horizontal Scaling**: API servers can be scaled independently
- **Load Balancing**: Application Load Balancer distributes traffic
- **Database Replication**: Read replicas reduce primary DB load
- **Caching Strategy**: Redis reduces database queries by 70%

### 4.2 Availability
- **Multi-AZ Deployment**: Services deployed across multiple availability zones
- **Auto-scaling**: Automatic scaling based on CPU/memory metrics
- **Health Checks**: Continuous monitoring with automatic recovery
- **Target SLA**: 99.9% uptime

### 4.3 Security
- **Authentication**: JWT tokens with refresh mechanism
- **Authorization**: Role-based access control (RBAC)
- **Encryption**: TLS 1.3 for data in transit, AES-256 for data at rest
- **Network Security**: VPC, Security Groups, Network ACLs
- **DDoS Protection**: AWS Shield and WAF

### 4.4 Performance
- **Response Time**: < 200ms for API calls (95th percentile)
- **Throughput**: 1000+ requests/second
- **Caching**: 80% cache hit rate target
- **CDN**: Static assets served via CloudFront

---

## 5. Data Flow

### 5.1 Authentication Flow
1. User submits credentials
2. API Gateway validates request
3. Auth Service verifies credentials
4. JWT token generated and cached
5. Token returned to client

### 5.2 Estimation Flow
1. User inputs project details
2. Frontend validates and sends to API
3. Estimation Service processes data
4. Calculations performed using selected method
5. Results stored in database and cache
6. Response returned with calculations

### 5.3 Report Generation Flow
1. User requests report export
2. Report Service fetches data from DB
3. Excel/PowerPoint generated using libraries
4. File uploaded to S3
5. Pre-signed URL returned to user

---

## 6. Integration Points

### 6.1 External Services
- **Email Service**: AWS SES for transactional emails
- **File Storage**: AWS S3 for document storage
- **Monitoring**: CloudWatch for logs and metrics
- **Analytics**: Custom analytics integration

### 6.2 API Specifications
- **Protocol**: REST over HTTPS
- **Format**: JSON
- **Versioning**: URI versioning (e.g., /api/v1/)
- **Authentication**: Bearer token (JWT)

---

## 7. Deployment Architecture

### 7.1 Environment Setup
- **Development**: Local Docker containers
- **Staging**: AWS staging environment
- **Production**: Multi-region AWS deployment

### 7.2 CI/CD Pipeline
1. Code push triggers GitHub Actions
2. Automated tests run
3. Docker images built
4. Images pushed to ECR
5. ECS tasks updated
6. Health checks performed

### 7.3 Infrastructure as Code
- Terraform for infrastructure provisioning
- CloudFormation for AWS resources
- Docker Compose for local development

---

## 8. Monitoring & Observability

### 8.1 Logging
- Centralized logging with CloudWatch
- Structured JSON logs
- Log retention: 30 days

### 8.2 Metrics
- Application metrics (response time, error rate)
- Infrastructure metrics (CPU, memory, disk)
- Business metrics (user activity, conversions)

### 8.3 Alerting
- PagerDuty integration for critical alerts
- Slack notifications for warnings
- Email alerts for informational events

---

## 9. Disaster Recovery

### 9.1 Backup Strategy
- **Database**: Automated daily backups, 30-day retention
- **Files**: S3 versioning enabled
- **Configuration**: Version controlled in Git

### 9.2 Recovery Objectives
- **RTO (Recovery Time Objective)**: 4 hours
- **RPO (Recovery Point Objective)**: 1 hour

### 9.3 Failover Process
1. Detect primary failure
2. Promote read replica to primary
3. Update DNS records
4. Verify system health
5. Communicate to stakeholders

---

## 10. Security Considerations

### 10.1 Authentication & Authorization
- Multi-factor authentication (MFA) support
- Role-based access control (RBAC)
- Session timeout: 24 hours
- Password policy: 12+ characters, complexity requirements

### 10.2 Data Protection
- Personal data encrypted at rest
- Sensitive fields hashed (passwords, tokens)
- Audit trail for data access
- GDPR compliance measures

### 10.3 Compliance
- SOC 2 Type II controls
- GDPR data protection requirements
- Regular security audits
- Penetration testing quarterly

---

## 11. Future Enhancements

### 11.1 Short Term (3-6 months)
- Mobile native applications
- Advanced analytics dashboard
- AI-powered estimation suggestions
- Real-time collaboration features

### 11.2 Long Term (6-12 months)
- Multi-tenant architecture
- Marketplace for estimation templates
- Integration with project management tools
- Machine learning for effort prediction

---

## 12. Conclusion

This high-level design provides a robust, scalable, and secure foundation for the ${cfg.projectName}. The architecture is designed to handle current requirements while maintaining flexibility for future growth and feature additions.

**Document Version:** 1.0
**Last Updated:** ${new Date().toLocaleDateString()}
**Status:** Draft

---

## Appendices

### A. Glossary
- **API**: Application Programming Interface
- **CDN**: Content Delivery Network
- **JWT**: JSON Web Token
- **RBAC**: Role-Based Access Control
- **RTO**: Recovery Time Objective
- **RPO**: Recovery Point Objective
- **SLA**: Service Level Agreement

### B. References
- AWS Architecture Best Practices
- OWASP Security Guidelines
- REST API Design Standards
- PostgreSQL Performance Tuning Guide
`;

    setHldDocument(doc);
  };

  const exportToWord = () => {
    toast.info('Word export will be implemented with docx library');
  };

  const exportToPDF = () => {
    const blob = new Blob([hldDocument], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'HLD_Document.txt';
    a.click();
    toast.success('HLD document exported');
  };

  if (!config) {
    return <div>Loading configuration...</div>;
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold">High-Level Design (HLD)</h2>
          <p className="text-muted-foreground">Comprehensive system architecture and design documentation</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={exportToWord}>
            <Download className="w-4 h-4 mr-2" />
            Export DOCX
          </Button>
          <Button onClick={exportToPDF}>
            <FileText className="w-4 h-4 mr-2" />
            Download Document
          </Button>
        </div>
      </div>

      {/* Project Info Card */}
      <Card className="bg-gradient-to-r from-blue-50 to-indigo-50 border-blue-200">
        <CardHeader>
          <CardTitle>{config.projectName}</CardTitle>
          <CardDescription>High-Level Design Specification</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
            <div>
              <p className="text-muted-foreground">System Type</p>
              <p className="font-semibold capitalize">{config.requirements.systemType}</p>
            </div>
            <div>
              <p className="text-muted-foreground">Expected Users</p>
              <p className="font-semibold">{config.requirements.expectedUsers}</p>
            </div>
            <div>
              <p className="text-muted-foreground">Security Level</p>
              <p className="font-semibold capitalize">{config.requirements.securityLevel}</p>
            </div>
            <div>
              <p className="text-muted-foreground">Deployment</p>
              <p className="font-semibold capitalize">{config.requirements.deploymentModel}</p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Architecture Diagrams */}
      <Tabs defaultValue="system" className="w-full">
        <TabsList className="grid w-full grid-cols-5">
          <TabsTrigger value="system">
            <Layers className="w-4 h-4 mr-2" />
            System
          </TabsTrigger>
          <TabsTrigger value="deployment">
            <Cloud className="w-4 h-4 mr-2" />
            Deployment
          </TabsTrigger>
          <TabsTrigger value="security">
            <Shield className="w-4 h-4 mr-2" />
            Security
          </TabsTrigger>
          <TabsTrigger value="components">
            <Settings className="w-4 h-4 mr-2" />
            Components
          </TabsTrigger>
          <TabsTrigger value="document">
            <FileText className="w-4 h-4 mr-2" />
            Document
          </TabsTrigger>
        </TabsList>

        <TabsContent value="system">
          <Card>
            <CardHeader>
              <CardTitle>System Architecture Diagram</CardTitle>
              <CardDescription>Complete system architecture with all layers and components</CardDescription>
            </CardHeader>
            <CardContent>
              <MermaidDiagram chart={systemArchDiagram} id="system-arch" />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="deployment">
          <Card>
            <CardHeader>
              <CardTitle>Deployment Architecture</CardTitle>
              <CardDescription>Infrastructure layout and deployment strategy</CardDescription>
            </CardHeader>
            <CardContent>
              <MermaidDiagram chart={deploymentDiagram} id="deployment-arch" />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="security">
          <Card>
            <CardHeader>
              <CardTitle>Security Architecture</CardTitle>
              <CardDescription>Multi-layered security implementation</CardDescription>
            </CardHeader>
            <CardContent>
              <MermaidDiagram chart={securityDiagram} id="security-arch" />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="components">
          <Card>
            <CardHeader>
              <CardTitle>Component Architecture</CardTitle>
              <CardDescription>Detailed component interaction and data flow</CardDescription>
            </CardHeader>
            <CardContent>
              <MermaidDiagram chart={componentDiagram} id="component-arch" />
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="document">
          <Card>
            <CardHeader>
              <CardTitle>HLD Document</CardTitle>
              <CardDescription>Complete high-level design specification document</CardDescription>
            </CardHeader>
            <CardContent>
              <Textarea
                value={hldDocument}
                readOnly
                className="font-mono text-xs min-h-[600px] whitespace-pre-wrap"
              />
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
