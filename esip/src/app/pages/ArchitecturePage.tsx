import React from 'react';
import { ArchitectureWizard } from '../components/architecture/ArchitectureWizard';

export default function ArchitecturePage() {
  return (
    <div className="container mx-auto py-8 px-4">
      <div className="mb-8">
        <h1 className="text-3xl font-bold mb-2">Architecture & Design Generation</h1>
        <p className="text-muted-foreground">
          Transform your requirements into comprehensive architecture documents
        </p>
      </div>

      <ArchitectureWizard />
    </div>
  );
}