import React, { useState, useEffect } from 'react';
import { Sparkles, CheckCircle2, AlertCircle, Loader2 } from 'lucide-react';
import { Button } from '../../ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../../ui/card';
import { Badge } from '../../ui/badge';
import { toast } from 'sonner';

interface RequirementsAnalysisProps {
  data: any;
  onChange: (data: any) => void;
}

interface AnalysisResult {
  modules: Array<{
    name: string;
    features: string[];
    screens: string[];
  }>;
  entities: Array<{
    name: string;
    attributes: string[];
    relationships: string[];
  }>;
  userTypes: string[];
  integrations: string[];
  techSuggestions: {
    frontend: string[];
    backend: string[];
    database: string[];
  };
}

export function RequirementsAnalysis({ data, onChange }: RequirementsAnalysisProps) {
  const [analyzing, setAnalyzing] = useState(false);
  const [analysis, setAnalysis] = useState<AnalysisResult | null>(data?.analysis || null);

  useEffect(() => {
    // Auto-analyze if we have requirements but no analysis
    if (data?.requirements && !analysis) {
      handleAnalyze();
    }
  }, []);

  const handleAnalyze = () => {
    if (!data?.requirements) {
      toast.error('No requirements to analyze');
      return;
    }

    setAnalyzing(true);
    
    // Simulate AI analysis with rule-based parsing
    setTimeout(() => {
      const result = analyzeRequirements(data.requirements);
      setAnalysis(result);
      onChange({ ...data, analysis: result });
      setAnalyzing(false);
      toast.success('Analysis complete!');
    }, 2000);
  };

  const analyzeRequirements = (text: string): AnalysisResult => {
    const lines = text.split('\n').filter(line => line.trim());
    
    // Extract modules (lines starting with numbers or "Module:")
    const modules: any[] = [];
    let currentModule: any = null;
    
    lines.forEach(line => {
      const trimmed = line.trim();
      
      // Detect module headers
      if (/^\d+\.|^Module:|^##/.test(trimmed)) {
        if (currentModule) {
          modules.push(currentModule);
        }
        const moduleName = trimmed
          .replace(/^\d+\./, '')
          .replace(/^Module:/, '')
          .replace(/^##/, '')
          .trim();
        currentModule = {
          name: moduleName,
          features: [],
          screens: [],
        };
      } else if (currentModule && trimmed.startsWith('-')) {
        // Features/screens
        const feature = trimmed.replace(/^-/, '').trim();
        currentModule.features.push(feature);
        
        // Try to extract screen names (text before parenthesis or colon)
        const screenMatch = feature.match(/^([^(:]+)/);
        if (screenMatch) {
          currentModule.screens.push(screenMatch[1].trim());
        }
      }
    });
    
    if (currentModule) {
      modules.push(currentModule);
    }

    // Extract entities (common database-like terms)
    const entityKeywords = ['user', 'product', 'order', 'customer', 'payment', 'cart', 'item', 'category', 'invoice', 'transaction', 'account', 'profile', 'address', 'review', 'rating', 'comment', 'post', 'article', 'project', 'task', 'team', 'member', 'role', 'permission'];
    const foundEntities = new Set<string>();
    
    const textLower = text.toLowerCase();
    entityKeywords.forEach(keyword => {
      if (textLower.includes(keyword)) {
        foundEntities.add(keyword.charAt(0).toUpperCase() + keyword.slice(1));
      }
    });

    // Build entity objects with likely attributes
    const entities = Array.from(foundEntities).map(entityName => {
      const attributes = ['id', 'createdAt', 'updatedAt'];
      const nameLower = entityName.toLowerCase();
      
      // Add common attributes based on entity type
      if (nameLower.includes('user') || nameLower.includes('customer')) {
        attributes.push('email', 'password', 'name', 'phone');
      }
      if (nameLower.includes('product')) {
        attributes.push('name', 'description', 'price', 'quantity', 'category');
      }
      if (nameLower.includes('order')) {
        attributes.push('orderNumber', 'status', 'total', 'userId');
      }
      if (nameLower.includes('payment')) {
        attributes.push('amount', 'method', 'status', 'transactionId');
      }
      
      // Infer relationships
      const relationships: string[] = [];
      if (nameLower.includes('order') && foundEntities.has('User')) {
        relationships.push('belongsTo User');
      }
      if (nameLower.includes('product') && foundEntities.has('Category')) {
        relationships.push('belongsTo Category');
      }
      if (nameLower.includes('item') && foundEntities.has('Order')) {
        relationships.push('belongsTo Order');
      }
      
      return {
        name: entityName,
        attributes,
        relationships,
      };
    });

    // Extract user types
    const userTypes = new Set<string>();
    const userKeywords = ['admin', 'user', 'customer', 'guest', 'manager', 'client', 'vendor', 'staff', 'moderator'];
    userKeywords.forEach(keyword => {
      if (textLower.includes(keyword)) {
        userTypes.add(keyword.charAt(0).toUpperCase() + keyword.slice(1));
      }
    });

    // Extract integrations
    const integrations: string[] = [];
    const integrationKeywords = {
      'stripe': 'Stripe Payment',
      'paypal': 'PayPal',
      'twilio': 'Twilio SMS',
      'sendgrid': 'SendGrid Email',
      'aws': 'AWS S3',
      'oauth': 'OAuth Authentication',
      'google': 'Google APIs',
      'facebook': 'Facebook Login',
      'email': 'Email Service',
      'sms': 'SMS Service',
      'payment': 'Payment Gateway',
    };
    
    Object.entries(integrationKeywords).forEach(([keyword, name]) => {
      if (textLower.includes(keyword)) {
        integrations.push(name);
      }
    });

    // Suggest tech stack based on requirements
    const techSuggestions = {
      frontend: ['React', 'TypeScript', 'Tailwind CSS'],
      backend: ['Node.js', 'Express'],
      database: ['PostgreSQL'],
    };

    if (textLower.includes('real-time') || textLower.includes('chat') || textLower.includes('notification')) {
      techSuggestions.backend.push('Socket.io');
      techSuggestions.database.push('Redis');
    }

    if (entities.length > 10 || textLower.includes('scale') || textLower.includes('enterprise')) {
      techSuggestions.backend.push('Microservices');
      techSuggestions.database.push('MongoDB');
    }

    return {
      modules: modules.length > 0 ? modules : [{
        name: 'Main Module',
        features: ['Feature 1', 'Feature 2'],
        screens: ['Home', 'Dashboard'],
      }],
      entities: entities.length > 0 ? entities : [{
        name: 'User',
        attributes: ['id', 'email', 'password', 'name'],
        relationships: [],
      }],
      userTypes: Array.from(userTypes),
      integrations,
      techSuggestions,
    };
  };

  if (analyzing) {
    return (
      <div className="flex flex-col items-center justify-center py-20">
        <Loader2 className="w-16 h-16 animate-spin text-primary mb-4" />
        <h3 className="text-xl font-semibold mb-2">Analyzing Requirements...</h3>
        <p className="text-muted-foreground">
          Extracting modules, entities, and relationships
        </p>
      </div>
    );
  }

  if (!analysis) {
    return (
      <div className="flex flex-col items-center justify-center py-20">
        <Sparkles className="w-16 h-16 text-muted-foreground mb-4" />
        <h3 className="text-xl font-semibold mb-2">Ready to Analyze</h3>
        <p className="text-muted-foreground mb-6">
          Click below to start analyzing your requirements
        </p>
        <Button onClick={handleAnalyze} size="lg">
          <Sparkles className="w-5 h-5 mr-2" />
          Start Analysis
        </Button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Analysis Complete Header */}
      <div className="flex items-center justify-between p-4 bg-green-50 border border-green-200 rounded-lg">
        <div className="flex items-center gap-3">
          <CheckCircle2 className="w-6 h-6 text-green-600" />
          <div>
            <h3 className="font-semibold text-green-900">Analysis Complete</h3>
            <p className="text-sm text-green-700">
              Found {analysis.modules.length} modules, {analysis.entities.length} entities, and {analysis.userTypes.length} user types
            </p>
          </div>
        </div>
        <Button variant="outline" onClick={handleAnalyze}>
          Re-analyze
        </Button>
      </div>

      {/* Modules */}
      <Card>
        <CardHeader>
          <CardTitle>Identified Modules ({analysis.modules.length})</CardTitle>
          <CardDescription>Core features and functionalities extracted from requirements</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {analysis.modules.map((module, idx) => (
              <Card key={idx} className="border-l-4 border-l-blue-500">
                <CardHeader className="pb-3">
                  <CardTitle className="text-base">{module.name}</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="space-y-2">
                    <div>
                      <p className="text-sm font-semibold mb-1">Features:</p>
                      <div className="flex flex-wrap gap-1">
                        {module.features.slice(0, 8).map((feature, fIdx) => (
                          <Badge key={fIdx} variant="secondary" className="text-xs">
                            {feature}
                          </Badge>
                        ))}
                        {module.features.length > 8 && (
                          <Badge variant="outline" className="text-xs">
                            +{module.features.length - 8} more
                          </Badge>
                        )}
                      </div>
                    </div>
                    {module.screens.length > 0 && (
                      <div>
                        <p className="text-sm font-semibold mb-1">Screens:</p>
                        <div className="flex flex-wrap gap-1">
                          {module.screens.map((screen, sIdx) => (
                            <Badge key={sIdx} variant="outline" className="text-xs">
                              {screen}
                            </Badge>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Entities */}
      <Card>
        <CardHeader>
          <CardTitle>Database Entities ({analysis.entities.length})</CardTitle>
          <CardDescription>Extracted data models and their attributes</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {analysis.entities.map((entity, idx) => (
              <Card key={idx}>
                <CardHeader className="pb-3">
                  <CardTitle className="text-sm">{entity.name}</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="space-y-2">
                    <div>
                      <p className="text-xs font-semibold mb-1">Attributes:</p>
                      <ul className="text-xs text-muted-foreground space-y-0.5">
                        {entity.attributes.map((attr, aIdx) => (
                          <li key={aIdx}>• {attr}</li>
                        ))}
                      </ul>
                    </div>
                    {entity.relationships.length > 0 && (
                      <div>
                        <p className="text-xs font-semibold mb-1">Relations:</p>
                        <ul className="text-xs text-muted-foreground space-y-0.5">
                          {entity.relationships.map((rel, rIdx) => (
                            <li key={rIdx}>→ {rel}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* User Types & Integrations */}
      <div className="grid grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle>User Types</CardTitle>
            <CardDescription>Actors in the system</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex flex-wrap gap-2">
              {analysis.userTypes.length > 0 ? (
                analysis.userTypes.map((type, idx) => (
                  <Badge key={idx} className="text-sm">
                    {type}
                  </Badge>
                ))
              ) : (
                <p className="text-sm text-muted-foreground">No user types detected</p>
              )}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Third-party Integrations</CardTitle>
            <CardDescription>External services</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex flex-wrap gap-2">
              {analysis.integrations.length > 0 ? (
                analysis.integrations.map((integration, idx) => (
                  <Badge key={idx} variant="secondary" className="text-sm">
                    {integration}
                  </Badge>
                ))
              ) : (
                <p className="text-sm text-muted-foreground">No integrations detected</p>
              )}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Tech Suggestions */}
      <Card className="bg-purple-50 border-purple-200">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Sparkles className="w-5 h-5 text-purple-600" />
            Recommended Tech Stack
          </CardTitle>
          <CardDescription>Based on your requirements</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-3 gap-4">
            <div>
              <p className="text-sm font-semibold mb-2">Frontend:</p>
              <div className="space-y-1">
                {analysis.techSuggestions.frontend.map((tech, idx) => (
                  <Badge key={idx} variant="outline" className="text-xs">
                    {tech}
                  </Badge>
                ))}
              </div>
            </div>
            <div>
              <p className="text-sm font-semibold mb-2">Backend:</p>
              <div className="space-y-1">
                {analysis.techSuggestions.backend.map((tech, idx) => (
                  <Badge key={idx} variant="outline" className="text-xs">
                    {tech}
                  </Badge>
                ))}
              </div>
            </div>
            <div>
              <p className="text-sm font-semibold mb-2">Database:</p>
              <div className="space-y-1">
                {analysis.techSuggestions.database.map((tech, idx) => (
                  <Badge key={idx} variant="outline" className="text-xs">
                    {tech}
                  </Badge>
                ))}
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
