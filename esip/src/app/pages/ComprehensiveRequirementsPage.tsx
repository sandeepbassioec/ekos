import React, { useState } from 'react';
import { useNavigate, useParams } from 'react-router';
import { ComprehensiveRequirementsWizard } from '../components/ComprehensiveRequirementsWizard';
import { ComprehensiveRequirements } from '../types/comprehensive-requirements';
import { toast } from 'sonner';

export default function ComprehensiveRequirementsPage() {
  const navigate = useNavigate();
  const { projectId } = useParams();
  const [projectName, setProjectName] = useState('Current Project'); // TODO: Get from API

  const handleComplete = (requirements: ComprehensiveRequirements) => {
    console.log('Comprehensive Requirements Captured:', requirements);
    toast.success('All requirements captured successfully!');
    // Navigate to WBS generation or estimation
    navigate(`/projects/${projectId}/wbs-generation`);
  };

  const handleCancel = () => {
    toast.info('Progress saved. You can continue later.');
    navigate(`/projects/${projectId || 'list'}`);
  };

  return (
    <ComprehensiveRequirementsWizard
      projectId={projectId || 'default-project'}
      projectName={projectName}
      onComplete={handleComplete}
      onCancel={handleCancel}
    />
  );
}
