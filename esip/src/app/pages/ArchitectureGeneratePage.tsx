import React from 'react';
import { useNavigate } from 'react-router';
import { ArrowLeft, FileText, GitBranch, Database, Code } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../components/ui/tabs';
import { BFAGenerator } from '../components/architecture/BFAGenerator';
import { DFAGenerator } from '../components/architecture/DFAGenerator';
import { HLDGenerator } from '../components/architecture/HLDGenerator';
import { LLDGenerator } from '../components/architecture/LLDGenerator';

export default function ArchitectureGeneratePage() {
  const navigate = useNavigate();

  return (
    <div className="container mx-auto py-8 px-4 max-w-7xl">
      {/* Header */}
      <div className="mb-8">
        <Button variant="ghost" onClick={() => navigate(-1)} className="mb-4">
          <ArrowLeft className="w-4 h-4 mr-2" />
          Back to Configuration
        </Button>
        
        <h1 className="text-3xl font-bold mb-2">Architecture Document Generation</h1>
        <p className="text-muted-foreground">
          Generate comprehensive architecture documents and diagrams for your project
        </p>
      </div>

      {/* Document Type Selection */}
      <Tabs defaultValue="bfa" className="w-full">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="bfa">
            <GitBranch className="w-4 h-4 mr-2" />
            BFA
          </TabsTrigger>
          <TabsTrigger value="dfa">
            <Database className="w-4 h-4 mr-2" />
            DFA
          </TabsTrigger>
          <TabsTrigger value="hld">
            <FileText className="w-4 h-4 mr-2" />
            HLD
          </TabsTrigger>
          <TabsTrigger value="lld">
            <Code className="w-4 h-4 mr-2" />
            LLD
          </TabsTrigger>
        </TabsList>

        {/* Business Flow Architecture */}
        <TabsContent value="bfa">
          <BFAGenerator />
        </TabsContent>

        {/* Data Flow Architecture */}
        <TabsContent value="dfa">
          <DFAGenerator />
        </TabsContent>

        {/* High-Level Design */}
        <TabsContent value="hld">
          <HLDGenerator />
        </TabsContent>

        {/* Low-Level Design */}
        <TabsContent value="lld">
          <LLDGenerator />
        </TabsContent>
      </Tabs>
    </div>
  );
}