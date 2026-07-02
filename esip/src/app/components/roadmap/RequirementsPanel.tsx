import React, { useState, useEffect, useRef } from 'react';
import { X, FileText, Users, Mail, MessageSquare, Brain, Upload, Plus, AlertTriangle, Shield, Zap, ChevronDown, ChevronRight, CheckCircle, AlertCircle, Lightbulb, Cloud } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card';
import { Badge } from '../ui/badge';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Textarea } from '../ui/textarea';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '../ui/dialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { RoadmapItem } from '../../types/roadmap';
import { useKnowledgeHubStore } from '../../store/knowledgeHubStore';
import { db } from '../../lib/db';
import { toast } from 'sonner';
import { NFRStep } from '../wizard-steps/NFRStep';
import { refineRequirements } from '../../lib/ai/bedrock';
import type { NonFunctionalRequirements } from '../../types/comprehensive-requirements';

interface RequirementsPanelProps {
  item: RoadmapItem;
  onClose: () => void;
}

export function RequirementsPanel({ item, onClose }: RequirementsPanelProps) {
  const {
    currentHub,
    uploadDocument,
    createMeeting,
    createEmail,
    createChat,
    fetchKnowledgeHub,
    getDocumentDownloadUrl,
    analyzeAllSources,
    isLoading,
    isAnalyzing,
  } = useKnowledgeHubStore();

  // Safely handle undefined arrays
  const safeDocuments = currentHub?.documents || [];
  const safeMeetings = currentHub?.meetings || [];
  const safeEmails = currentHub?.emails || [];
  const safeChats = currentHub?.chats || [];
  const safeAiRequirements = currentHub?.aiRequirements || [];
  const safeGaps = currentHub?.gaps || [];
  const safeConfusions = currentHub?.confusions || [];
  const safeSuggestions = currentHub?.suggestions || [];
  const safeSecurityRequirements = currentHub?.securityRequirements || [];
  const safeThreats = currentHub?.threats || [];
  const safeNFRs = currentHub?.nfrs || [];
  const safeCloudGovernance = currentHub?.cloudGovernance || [];

  // Navigation state
  const [activeSection, setActiveSection] = useState('requirements');
  const [expandedGroups, setExpandedGroups] = useState<string[]>(['ai-analysis', 'sources']);

  // Modal states - Source Data
  const [documentModalOpen, setDocumentModalOpen] = useState(false);
  const [meetingModalOpen, setMeetingModalOpen] = useState(false);
  const [emailModalOpen, setEmailModalOpen] = useState(false);
  const [chatModalOpen, setChatModalOpen] = useState(false);

  // Modal states - Requirements
  const [gapModalOpen, setGapModalOpen] = useState(false);
  const [confusionModalOpen, setConfusionModalOpen] = useState(false);
  const [securityReqModalOpen, setSecurityReqModalOpen] = useState(false);
  const [threatModalOpen, setThreatModalOpen] = useState(false);
  const [nfrModalOpen, setNfrModalOpen] = useState(false);

  // Resolution modals
  const [gapResolutionModalOpen, setGapResolutionModalOpen] = useState(false);
  const [confusionResolutionModalOpen, setConfusionResolutionModalOpen] = useState(false);
  const [selectedGap, setSelectedGap] = useState<any>(null);
  const [selectedConfusion, setSelectedConfusion] = useState<any>(null);
  const [isResolving, setIsResolving] = useState(false);

  // Form states
  const [docType, setDocType] = useState('');
  const [chatPlatform, setChatPlatform] = useState('');
  const [gapSeverity, setGapSeverity] = useState('');
  const [securityCategory, setSecurityCategory] = useState('');
  const [securityLevel, setSecurityLevel] = useState('');
  const [threatCategory, setThreatCategory] = useState('');
  const [threatLevel, setThreatLevel] = useState('');
  const [threatLikelihood, setThreatLikelihood] = useState('');
  const [nfrData, setNfrData] = useState<NonFunctionalRequirements | undefined>(undefined);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const toggleGroup = (groupId: string) => {
    setExpandedGroups(prev =>
      prev.includes(groupId)
        ? prev.filter(id => id !== groupId)
        : [...prev, groupId]
    );
  };

  useEffect(() => {
    fetchKnowledgeHub(item.id);
  }, [item.id, fetchKnowledgeHub]);

  // AI Analysis handler
  const handleAnalyzeWithAI = async () => {
    await analyzeAllSources(item.id);
  };

  // Gap handler
  const handleGapSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setIsSubmitting(true);

    const formData = new FormData(e.currentTarget);
    const suggestionsText = formData.get('suggestions') as string;
    const suggestions = suggestionsText ? suggestionsText.split('\n').filter(s => s.trim()) : [];

    try {
      await db.gaps.create({
        roadmapItemId: item.id,
        title: formData.get('title') as string,
        description: formData.get('description') as string,
        severity: gapSeverity as any,
        status: 'Open',
        sourceType: 'Manual',
        suggestions,
      });
      toast.success('Gap added successfully');
      setGapModalOpen(false);
      setGapSeverity('');
      (e.target as HTMLFormElement).reset();
      await fetchKnowledgeHub(item.id);
    } catch (error) {
      console.error('Error adding gap:', error);
      toast.error(`Failed to add gap: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsSubmitting(false);
    }
  };

  // Confusion handler
  const handleConfusionSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setIsSubmitting(true);

    const formData = new FormData(e.currentTarget);

    try {
      await db.confusions.create({
        roadmapItemId: item.id,
        question: formData.get('question') as string,
        context: formData.get('context') as string,
        sources: [],
        status: 'Open',
      });
      toast.success('Confusion added successfully');
      setConfusionModalOpen(false);
      (e.target as HTMLFormElement).reset();
      await fetchKnowledgeHub(item.id);
    } catch (error) {
      toast.error('Failed to add confusion');
    } finally {
      setIsSubmitting(false);
    }
  };

  // Security Requirement handler
  const handleSecurityReqSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setIsSubmitting(true);

    const formData = new FormData(e.currentTarget);
    const controls = (formData.get('controls') as string)
      .split('\n')
      .filter(c => c.trim());
    const testCriteria = (formData.get('testCriteria') as string)
      .split('\n')
      .filter(t => t.trim());

    try {
      await db.securityRequirements.create({
        roadmapItemId: item.id,
        title: formData.get('title') as string,
        description: formData.get('description') as string,
        category: securityCategory as any,
        level: securityLevel as any,
        standard: formData.get('standard') as string,
        controls,
        testCriteria,
        status: 'Draft',
      });
      toast.success('Security requirement added successfully');
      setSecurityReqModalOpen(false);
      setSecurityCategory('');
      setSecurityLevel('');
      (e.target as HTMLFormElement).reset();
      await fetchKnowledgeHub(item.id);
    } catch (error) {
      toast.error('Failed to add security requirement');
    } finally {
      setIsSubmitting(false);
    }
  };

  // Threat handler
  const handleThreatSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setIsSubmitting(true);

    const formData = new FormData(e.currentTarget);
    const mitigations = (formData.get('mitigations') as string)
      .split('\n')
      .filter(m => m.trim());

    try {
      await db.threats.create({
        roadmapItemId: item.id,
        threat: formData.get('threat') as string,
        description: formData.get('description') as string,
        category: threatCategory as any,
        level: threatLevel as any,
        likelihood: threatLikelihood as any,
        impact: formData.get('impact') as string,
        mitigations,
        status: 'Identified',
      });
      toast.success('Threat added successfully');
      setThreatModalOpen(false);
      setThreatCategory('');
      setThreatLevel('');
      setThreatLikelihood('');
      (e.target as HTMLFormElement).reset();
      await fetchKnowledgeHub(item.id);
    } catch (error) {
      toast.error('Failed to add threat');
    } finally {
      setIsSubmitting(false);
    }
  };

  // Gap resolution handler
  const handleGapResolution = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!selectedGap) return;

    setIsResolving(true);
    const formData = new FormData(e.currentTarget);
    const resolution = formData.get('resolution') as string;

    try {
      toast.info('AI is refining requirements based on your resolution...');

      // Call AI to refine requirements based on resolution
      const refinedRequirements = await refineRequirements(
        'gap',
        selectedGap.description,
        selectedGap.title,
        resolution
      );

      // Update gap with resolution
      await db.gaps.update(selectedGap.id, {
        status: 'Resolved',
        resolution,
        resolvedAt: new Date().toISOString(),
      });

      // Save refined requirements as AI requirements
      for (const req of refinedRequirements) {
        await db.aiRequirements.create({
          roadmapItemId: item.id,
          requirement: req,
          source: `Gap Resolution: ${selectedGap.title}`,
          sourceType: 'AI',
        });
      }

      toast.success(`Gap resolved! ${refinedRequirements.length} new requirements added`);
      setGapResolutionModalOpen(false);
      setSelectedGap(null);
      (e.target as HTMLFormElement).reset();
      await fetchKnowledgeHub(item.id);
    } catch (error) {
      console.error('Error resolving gap:', error);
      toast.error(`Failed to resolve gap: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsResolving(false);
    }
  };

  // Confusion resolution handler
  const handleConfusionResolution = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!selectedConfusion) return;

    setIsResolving(true);
    const formData = new FormData(e.currentTarget);
    const clarification = formData.get('clarification') as string;

    try {
      toast.info('AI is refining requirements based on your clarification...');

      // Call AI to refine requirements based on clarification
      const refinedRequirements = await refineRequirements(
        'confusion',
        selectedConfusion.context,
        selectedConfusion.question,
        clarification
      );

      // Update confusion with clarification
      await db.confusions.update(selectedConfusion.id, {
        status: 'Resolved',
        clarification,
        resolvedBy: 'User',
        resolvedAt: new Date().toISOString(),
      });

      // Save refined requirements as AI requirements
      for (const req of refinedRequirements) {
        await db.aiRequirements.create({
          roadmapItemId: item.id,
          requirement: req,
          source: `Confusion Clarification: ${selectedConfusion.question}`,
          sourceType: 'AI',
        });
      }

      toast.success(`Confusion resolved! ${refinedRequirements.length} new requirements added`);
      setConfusionResolutionModalOpen(false);
      setSelectedConfusion(null);
      (e.target as HTMLFormElement).reset();
      await fetchKnowledgeHub(item.id);
    } catch (error) {
      console.error('Error resolving confusion:', error);
      toast.error(`Failed to resolve confusion: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsResolving(false);
    }
  };

  // NFR handler - Convert comprehensive structure to individual database records
  const handleNFRSubmit = async () => {
    if (!nfrData) {
      toast.error('Please fill in at least one NFR section');
      return;
    }

    setIsSubmitting(true);

    try {
      const promises = [];

      // Performance NFRs
      if (nfrData.performance.responseTime) {
        promises.push(db.nfrs.create({
          roadmapItemId: item.id,
          category: 'Performance',
          requirement: 'Response Time',
          metric: 'Response Time',
          target: nfrData.performance.responseTime,
          measurement: 'Average response time under normal load',
          priority: 'High',
          status: 'Draft',
        }));
      }
      if (nfrData.performance.throughput) {
        promises.push(db.nfrs.create({
          roadmapItemId: item.id,
          category: 'Performance',
          requirement: 'System Throughput',
          metric: 'Requests per second',
          target: nfrData.performance.throughput,
          measurement: 'Measured under load testing',
          priority: 'High',
          status: 'Draft',
        }));
      }
      if (nfrData.performance.concurrentUsers > 0) {
        promises.push(db.nfrs.create({
          roadmapItemId: item.id,
          category: 'Performance',
          requirement: 'Concurrent Users Support',
          metric: 'Concurrent Users',
          target: nfrData.performance.concurrentUsers.toString(),
          measurement: 'Load testing with concurrent sessions',
          priority: 'High',
          status: 'Draft',
        }));
      }
      if (nfrData.performance.pageLoadTime) {
        promises.push(db.nfrs.create({
          roadmapItemId: item.id,
          category: 'Performance',
          requirement: 'Page Load Time',
          metric: 'Page Load Time',
          target: nfrData.performance.pageLoadTime,
          measurement: 'Lighthouse performance score',
          priority: 'Medium',
          status: 'Draft',
        }));
      }

      // Scalability NFRs
      if (nfrData.scalability.expectedUserGrowth) {
        promises.push(db.nfrs.create({
          roadmapItemId: item.id,
          category: 'Scalability',
          requirement: 'User Growth Capacity',
          metric: 'User Growth Rate',
          target: nfrData.scalability.expectedUserGrowth,
          measurement: 'System supports projected growth',
          priority: 'High',
          status: 'Draft',
        }));
      }
      if (nfrData.scalability.peakLoadScenario) {
        promises.push(db.nfrs.create({
          roadmapItemId: item.id,
          category: 'Scalability',
          requirement: 'Peak Load Handling',
          metric: 'Peak Load Capacity',
          target: nfrData.scalability.peakLoadScenario,
          measurement: 'Stress testing at peak scenarios',
          priority: 'Critical',
          status: 'Draft',
        }));
      }

      // Availability NFRs
      if (nfrData.availability.uptimeSLA) {
        promises.push(db.nfrs.create({
          roadmapItemId: item.id,
          category: 'Availability',
          requirement: 'System Uptime SLA',
          metric: 'Uptime Percentage',
          target: nfrData.availability.uptimeSLA,
          measurement: 'Monthly uptime tracking',
          priority: 'Critical',
          status: 'Draft',
        }));
      }
      if (nfrData.availability.disasterRecovery.rto) {
        promises.push(db.nfrs.create({
          roadmapItemId: item.id,
          category: 'Availability',
          requirement: 'Recovery Time Objective',
          metric: 'RTO',
          target: nfrData.availability.disasterRecovery.rto,
          measurement: 'DR testing and validation',
          priority: 'Critical',
          status: 'Draft',
        }));
      }
      if (nfrData.availability.disasterRecovery.rpo) {
        promises.push(db.nfrs.create({
          roadmapItemId: item.id,
          category: 'Availability',
          requirement: 'Recovery Point Objective',
          metric: 'RPO',
          target: nfrData.availability.disasterRecovery.rpo,
          measurement: 'Backup frequency validation',
          priority: 'Critical',
          status: 'Draft',
        }));
      }

      // Usability NFRs
      if (nfrData.usability.accessibilityLevel !== 'None') {
        promises.push(db.nfrs.create({
          roadmapItemId: item.id,
          category: 'Usability',
          requirement: 'Accessibility Compliance',
          metric: 'Accessibility Level',
          target: nfrData.usability.accessibilityLevel,
          measurement: 'Accessibility audit and testing',
          priority: 'High',
          status: 'Draft',
        }));
      }
      if (nfrData.usability.mobileResponsive) {
        promises.push(db.nfrs.create({
          roadmapItemId: item.id,
          category: 'Usability',
          requirement: 'Mobile Responsive Design',
          metric: 'Responsive Design',
          target: 'Full mobile responsiveness',
          measurement: 'Cross-device testing',
          priority: 'High',
          status: 'Draft',
        }));
      }

      // Maintainability NFRs
      if (nfrData.maintainability.codeCoverageTarget > 0) {
        promises.push(db.nfrs.create({
          roadmapItemId: item.id,
          category: 'Maintainability',
          requirement: 'Code Coverage Target',
          metric: 'Test Coverage Percentage',
          target: `${nfrData.maintainability.codeCoverageTarget}%`,
          measurement: 'Code coverage reports',
          priority: 'Medium',
          status: 'Draft',
        }));
      }

      await Promise.all(promises);

      toast.success(`${promises.length} NFRs added successfully`);
      setNfrModalOpen(false);
      setNfrData(undefined);
      await fetchKnowledgeHub(item.id);
    } catch (error) {
      toast.error('Failed to add NFRs');
    } finally {
      setIsSubmitting(false);
    }
  };

  // View document handler
  const handleViewDocument = async (docId: string) => {
    try {
      const url = await getDocumentDownloadUrl(docId);
      if (url) {
        // Open document in new tab
        window.open(url, '_blank');
      }
    } catch (error) {
      console.error('Error viewing document:', error);
    }
  };

  // Document upload handler
  const handleDocumentUpload = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setIsSubmitting(true);

    try {
      const formData = new FormData(e.currentTarget);
      const file = formData.get('file') as File;
      const name = formData.get('name') as string;

      console.log('Upload attempt:', { file, name, docType });

      if (!file || !name || !docType) {
        console.error('Missing required fields:', { file: !!file, name: !!name, docType });
        setIsSubmitting(false);
        return;
      }

      const success = await uploadDocument({
        roadmapItemId: item.id,
        name,
        type: docType as any,
        version: '1.0',
        file,
      });

      console.log('Upload result:', success);

      if (success) {
        setDocumentModalOpen(false);
        setDocType('');
        (e.target as HTMLFormElement).reset();
      }
    } catch (error) {
      console.error('Upload error:', error);
    } finally {
      setIsSubmitting(false);
    }
  };

  // Meeting handler
  const handleMeetingSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setIsSubmitting(true);

    const formData = new FormData(e.currentTarget);

    const success = await createMeeting({
      roadmapItemId: item.id,
      title: formData.get('title') as string,
      date: formData.get('date') as string,
      participants: (formData.get('participants') as string).split(',').map(p => p.trim()),
      transcript: formData.get('notes') as string,
      keyDecisions: [],
    });

    if (success) {
      setMeetingModalOpen(false);
      (e.target as HTMLFormElement).reset();
    }
    setIsSubmitting(false);
  };

  // Email handler
  const handleEmailSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setIsSubmitting(true);

    const formData = new FormData(e.currentTarget);

    const success = await createEmail({
      roadmapItemId: item.id,
      subject: formData.get('subject') as string,
      from: formData.get('sender') as string,
      to: [(formData.get('receiver') as string)],
      date: formData.get('date') as string,
      body: formData.get('body') as string,
    });

    if (success) {
      setEmailModalOpen(false);
      (e.target as HTMLFormElement).reset();
    }
    setIsSubmitting(false);
  };

  // Chat handler
  const handleChatSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setIsSubmitting(true);

    const formData = new FormData(e.currentTarget);

    if (!chatPlatform) {
      setIsSubmitting(false);
      return;
    }

    const success = await createChat({
      roadmapItemId: item.id,
      platform: chatPlatform as 'Teams' | 'Slack' | 'Other',
      date: formData.get('date') as string,
      participants: (formData.get('participants') as string).split(',').map(p => p.trim()),
      messages: formData.get('content') as string,
    });

    if (success) {
      setChatModalOpen(false);
      setChatPlatform('');
      (e.target as HTMLFormElement).reset();
    }
    setIsSubmitting(false);
  };

  return (
    <div className="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4">
      <div className="bg-white rounded-lg shadow-2xl max-w-6xl w-full max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="p-6 border-b bg-gradient-to-r from-blue-50 to-indigo-50">
          <div className="flex items-start justify-between gap-4">
            <div className="flex-1">
              <h2 className="text-2xl font-bold text-gray-900 mb-2">{item.title}</h2>
              <p className="text-gray-600">{item.description || 'No description'}</p>
              <div className="flex gap-2 mt-3">
                <Badge variant="secondary">{item.status.replace('_', ' ')}</Badge>
                <Badge variant="secondary">{item.priority}</Badge>
                {item.startDate && (
                  <Badge variant="outline">
                    {new Date(item.startDate).toLocaleDateString()} - {item.endDate ? new Date(item.endDate).toLocaleDateString() : 'Ongoing'}
                  </Badge>
                )}
              </div>
            </div>
            <div className="flex items-center gap-2">
              <Button
                onClick={handleAnalyzeWithAI}
                disabled={isAnalyzing}
                className="bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-700 hover:to-indigo-700"
              >
                <Brain className="w-4 h-4 mr-2" />
                {isAnalyzing ? 'Analyzing...' : 'Analyze with AI'}
              </Button>
              <Button variant="ghost" size="sm" onClick={onClose}>
                <X className="w-5 h-5" />
              </Button>
            </div>
          </div>
        </div>

        {/* Content - Sidebar Layout */}
        <div className="flex-1 flex overflow-hidden">
          {/* Sidebar Navigation */}
          <div className="w-64 border-r bg-gray-50 overflow-y-auto">
            <div className="p-4 space-y-1">
              {/* AI Analysis Group */}
              <div>
                <button
                  onClick={() => toggleGroup('ai-analysis')}
                  className="flex items-center justify-between w-full px-3 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-100 rounded"
                >
                  <div className="flex items-center gap-2">
                    <Brain className="w-4 h-4" />
                    <span>AI Analysis</span>
                  </div>
                  {expandedGroups.includes('ai-analysis') ? (
                    <ChevronDown className="w-4 h-4" />
                  ) : (
                    <ChevronRight className="w-4 h-4" />
                  )}
                </button>
                {expandedGroups.includes('ai-analysis') && (
                  <div className="ml-6 mt-1 space-y-1">
                    <button
                      onClick={() => setActiveSection('requirements')}
                      className={`flex items-center gap-2 w-full px-3 py-2 text-sm rounded ${
                        activeSection === 'requirements'
                          ? 'bg-blue-100 text-blue-700 font-medium'
                          : 'text-gray-600 hover:bg-gray-100'
                      }`}
                    >
                      <CheckCircle className="w-3 h-3" />
                      <span>Requirements</span>
                      <span className="ml-auto text-xs">{safeAiRequirements.length}</span>
                    </button>
                    <button
                      onClick={() => setActiveSection('gaps')}
                      className={`flex items-center gap-2 w-full px-3 py-2 text-sm rounded ${
                        activeSection === 'gaps'
                          ? 'bg-blue-100 text-blue-700 font-medium'
                          : 'text-gray-600 hover:bg-gray-100'
                      }`}
                    >
                      <AlertTriangle className="w-3 h-3" />
                      <span>Gaps</span>
                      <span className="ml-auto text-xs">{safeGaps.length}</span>
                    </button>
                    <button
                      onClick={() => setActiveSection('confusions')}
                      className={`flex items-center gap-2 w-full px-3 py-2 text-sm rounded ${
                        activeSection === 'confusions'
                          ? 'bg-blue-100 text-blue-700 font-medium'
                          : 'text-gray-600 hover:bg-gray-100'
                      }`}
                    >
                      <AlertCircle className="w-3 h-3" />
                      <span>Confusions</span>
                      <span className="ml-auto text-xs">{safeConfusions.length}</span>
                    </button>
                    <button
                      onClick={() => setActiveSection('suggestions')}
                      className={`flex items-center gap-2 w-full px-3 py-2 text-sm rounded ${
                        activeSection === 'suggestions'
                          ? 'bg-blue-100 text-blue-700 font-medium'
                          : 'text-gray-600 hover:bg-gray-100'
                      }`}
                    >
                      <Lightbulb className="w-3 h-3" />
                      <span>Suggestions</span>
                      <span className="ml-auto text-xs">{safeSuggestions.length}</span>
                    </button>
                  </div>
                )}
              </div>

              {/* Security Group */}
              <div>
                <button
                  onClick={() => toggleGroup('security')}
                  className="flex items-center justify-between w-full px-3 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-100 rounded"
                >
                  <div className="flex items-center gap-2">
                    <Shield className="w-4 h-4" />
                    <span>Security</span>
                  </div>
                  {expandedGroups.includes('security') ? (
                    <ChevronDown className="w-4 h-4" />
                  ) : (
                    <ChevronRight className="w-4 h-4" />
                  )}
                </button>
                {expandedGroups.includes('security') && (
                  <div className="ml-6 mt-1 space-y-1">
                    <button
                      onClick={() => setActiveSection('security-requirements')}
                      className={`flex items-center gap-2 w-full px-3 py-2 text-sm rounded ${
                        activeSection === 'security-requirements'
                          ? 'bg-blue-100 text-blue-700 font-medium'
                          : 'text-gray-600 hover:bg-gray-100'
                      }`}
                    >
                      <Shield className="w-3 h-3" />
                      <span>Requirements</span>
                      <span className="ml-auto text-xs">{safeSecurityRequirements.length}</span>
                    </button>
                    <button
                      onClick={() => setActiveSection('threats')}
                      className={`flex items-center gap-2 w-full px-3 py-2 text-sm rounded ${
                        activeSection === 'threats'
                          ? 'bg-blue-100 text-blue-700 font-medium'
                          : 'text-gray-600 hover:bg-gray-100'
                      }`}
                    >
                      <AlertTriangle className="w-3 h-3" />
                      <span>Threats (STRIDE)</span>
                      <span className="ml-auto text-xs">{safeThreats.length}</span>
                    </button>
                  </div>
                )}
              </div>

              {/* NFRs */}
              <button
                onClick={() => setActiveSection('nfrs')}
                className={`flex items-center justify-between w-full px-3 py-2 text-sm font-semibold rounded ${
                  activeSection === 'nfrs'
                    ? 'bg-blue-100 text-blue-700'
                    : 'text-gray-700 hover:bg-gray-100'
                }`}
              >
                <div className="flex items-center gap-2">
                  <Zap className="w-4 h-4" />
                  <span>NFRs</span>
                </div>
                <span className="text-xs">{safeNFRs.length}</span>
              </button>

              {/* Cloud Governance */}
              <button
                onClick={() => setActiveSection('cloud-governance')}
                className={`flex items-center justify-between w-full px-3 py-2 text-sm font-semibold rounded ${
                  activeSection === 'cloud-governance'
                    ? 'bg-blue-100 text-blue-700'
                    : 'text-gray-700 hover:bg-gray-100'
                }`}
              >
                <div className="flex items-center gap-2">
                  <Cloud className="w-4 h-4" />
                  <span>Cloud Governance</span>
                </div>
                <span className="text-xs">{safeCloudGovernance.length}</span>
              </button>

              {/* Sources Group */}
              <div>
                <button
                  onClick={() => toggleGroup('sources')}
                  className="flex items-center justify-between w-full px-3 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-100 rounded"
                >
                  <div className="flex items-center gap-2">
                    <FileText className="w-4 h-4" />
                    <span>Source Data</span>
                  </div>
                  {expandedGroups.includes('sources') ? (
                    <ChevronDown className="w-4 h-4" />
                  ) : (
                    <ChevronRight className="w-4 h-4" />
                  )}
                </button>
                {expandedGroups.includes('sources') && (
                  <div className="ml-6 mt-1 space-y-1">
                    <button
                      onClick={() => setActiveSection('documents')}
                      className={`flex items-center gap-2 w-full px-3 py-2 text-sm rounded ${
                        activeSection === 'documents'
                          ? 'bg-blue-100 text-blue-700 font-medium'
                          : 'text-gray-600 hover:bg-gray-100'
                      }`}
                    >
                      <FileText className="w-3 h-3" />
                      <span>Documents</span>
                      <span className="ml-auto text-xs">{safeDocuments.length}</span>
                    </button>
                    <button
                      onClick={() => setActiveSection('meetings')}
                      className={`flex items-center gap-2 w-full px-3 py-2 text-sm rounded ${
                        activeSection === 'meetings'
                          ? 'bg-blue-100 text-blue-700 font-medium'
                          : 'text-gray-600 hover:bg-gray-100'
                      }`}
                    >
                      <Users className="w-3 h-3" />
                      <span>Meetings</span>
                      <span className="ml-auto text-xs">{safeMeetings.length}</span>
                    </button>
                    <button
                      onClick={() => setActiveSection('emails')}
                      className={`flex items-center gap-2 w-full px-3 py-2 text-sm rounded ${
                        activeSection === 'emails'
                          ? 'bg-blue-100 text-blue-700 font-medium'
                          : 'text-gray-600 hover:bg-gray-100'
                      }`}
                    >
                      <Mail className="w-3 h-3" />
                      <span>Emails</span>
                      <span className="ml-auto text-xs">{safeEmails.length}</span>
                    </button>
                    <button
                      onClick={() => setActiveSection('chats')}
                      className={`flex items-center gap-2 w-full px-3 py-2 text-sm rounded ${
                        activeSection === 'chats'
                          ? 'bg-blue-100 text-blue-700 font-medium'
                          : 'text-gray-600 hover:bg-gray-100'
                      }`}
                    >
                      <MessageSquare className="w-3 h-3" />
                      <span>Chats</span>
                      <span className="ml-auto text-xs">{safeChats.length}</span>
                    </button>
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Main Content Area */}
          <div className="flex-1 overflow-y-auto p-6">

            {/* Gap Analysis Section */}
            {activeSection === 'gaps' && (
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>Gap Analysis</CardTitle>
                      <CardDescription>Missing or incomplete requirements identified by AI</CardDescription>
                    </div>
                    <Button size="sm" onClick={() => setGapModalOpen(true)}>
                      <Plus className="w-4 h-4 mr-2" />
                      Add Gap
                    </Button>
                  </div>
                </CardHeader>
                <CardContent>
                  {safeGaps.length === 0 ? (
                    <div className="text-center py-12">
                      <AlertTriangle className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                      <h3 className="text-lg font-semibold mb-2">No Gaps Identified</h3>
                      <p className="text-muted-foreground">
                        Click "Analyze with AI" to identify missing requirements and gaps
                      </p>
                    </div>
                  ) : (
                    <div className="space-y-4">
                      {safeGaps.map((gap: any) => (
                        <div key={gap.id} className="border rounded-lg p-4 hover:bg-gray-50">
                          <div className="flex items-start justify-between mb-3">
                            <div className="flex items-center gap-2">
                              <Badge
                                variant={
                                  gap.severity === 'Critical' ? 'destructive' :
                                  gap.severity === 'High' ? 'default' :
                                  'secondary'
                                }
                              >
                                {gap.severity}
                              </Badge>
                              <Badge variant="outline">{gap.status}</Badge>
                            </div>
                            <span className="text-xs text-muted-foreground">
                              {new Date(gap.createdAt).toLocaleDateString()}
                            </span>
                          </div>
                          <h4 className="font-semibold mb-2">{gap.title}</h4>
                          <p className="text-sm text-gray-700 mb-3">{gap.description}</p>
                          {gap.sourceType && (
                            <div className="flex items-center gap-2 text-xs text-muted-foreground mb-2">
                              <FileText className="w-3 h-3" />
                              Source: {gap.sourceType}
                            </div>
                          )}
                          {gap.suggestions && gap.suggestions.length > 0 && (
                            <div className="mt-3 p-3 bg-blue-50 rounded border border-blue-100">
                              <div className="font-medium text-sm mb-2">AI Suggestions:</div>
                              <ul className="space-y-1">
                                {gap.suggestions.map((suggestion: string, idx: number) => (
                                  <li key={idx} className="text-sm text-blue-900">• {suggestion}</li>
                                ))}
                              </ul>
                            </div>
                          )}
                          {gap.resolution && (
                            <div className="mt-3 p-3 bg-green-50 rounded border border-green-100">
                              <div className="font-medium text-sm mb-1">Resolution:</div>
                              <p className="text-sm text-green-900">{gap.resolution}</p>
                              {gap.resolvedAt && (
                                <p className="text-xs text-green-700 mt-1">
                                  Resolved on {new Date(gap.resolvedAt).toLocaleDateString()}
                                </p>
                              )}
                            </div>
                          )}
                          {gap.status === 'Open' && (
                            <div className="mt-3 flex justify-end">
                              <Button
                                size="sm"
                                variant="outline"
                                onClick={() => {
                                  setSelectedGap(gap);
                                  setGapResolutionModalOpen(true);
                                }}
                              >
                                <CheckCircle className="w-4 h-4 mr-2" />
                                Resolve Gap
                              </Button>
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            )}

            {/* AI Requirements Section */}
            {activeSection === 'requirements' && (
              <Card>
                <CardHeader>
                  <CardTitle>AI-Extracted Requirements</CardTitle>
                  <CardDescription>
                    Requirements automatically extracted from documents, meetings, and communications
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  {isLoading ? (
                    <div className="text-center py-8 text-muted-foreground">Loading...</div>
                  ) : safeAiRequirements.length === 0 ? (
                    <div className="text-center py-12">
                      <Brain className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                      <h3 className="text-lg font-semibold mb-2">No AI Requirements Yet</h3>
                      <p className="text-muted-foreground mb-4">
                        Upload documents to the Documents tab to extract requirements automatically
                      </p>
                    </div>
                  ) : (
                    <div className="space-y-4">
                      {safeAiRequirements.map((req) => (
                        <div key={req.id} className="border rounded-lg p-4">
                          <div className="flex items-start justify-between mb-2">
                            <Badge>{req.category || req.sourceType}</Badge>
                            <span className="text-xs text-muted-foreground">
                              {new Date(req.createdAt).toLocaleDateString()}
                            </span>
                          </div>
                          <p className="text-gray-900 mb-2">{req.requirement}</p>
                          {req.source && (
                            <p className="text-sm text-muted-foreground">
                              Source: {req.source}
                            </p>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            )}

            {/* Documents Section */}
            {activeSection === 'documents' && (
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>Documents</CardTitle>
                      <CardDescription>BRDs, FRDs, and other requirement documents</CardDescription>
                    </div>
                    <Button size="sm" onClick={() => setDocumentModalOpen(true)}>
                      <Upload className="w-4 h-4 mr-2" />
                      Upload Document
                    </Button>
                  </div>
                </CardHeader>
                <CardContent>
                  {safeDocuments.length === 0 ? (
                    <div className="text-center py-12">
                      <FileText className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                      <h3 className="text-lg font-semibold mb-2">No Documents Uploaded</h3>
                      <p className="text-muted-foreground">
                        Upload BRDs, FRDs, and other documents for AI analysis
                      </p>
                    </div>
                  ) : (
                    <div className="space-y-3">
                      {safeDocuments.map((doc) => (
                        <div key={doc.id} className="flex items-center justify-between p-3 border rounded-lg hover:bg-gray-50">
                          <div className="flex items-center gap-3">
                            <FileText className="w-5 h-5 text-blue-600" />
                            <div>
                              <div className="font-medium">{doc.name}</div>
                              <div className="text-sm text-muted-foreground">
                                {doc.type} • Uploaded {new Date(doc.uploadedAt).toLocaleDateString()}
                              </div>
                            </div>
                          </div>
                          <Button variant="outline" size="sm" onClick={() => handleViewDocument(doc.id)}>
                            View
                          </Button>
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            )}

            {/* Meetings Section */}
            {activeSection === 'meetings' && (
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>Meetings</CardTitle>
                      <CardDescription>Meeting notes and requirements discussions</CardDescription>
                    </div>
                    <Button size="sm" onClick={() => setMeetingModalOpen(true)}>
                      <Plus className="w-4 h-4 mr-2" />
                      Add Meeting
                    </Button>
                  </div>
                </CardHeader>
                <CardContent>
                  {safeMeetings.length === 0 ? (
                    <div className="text-center py-12">
                      <Users className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                      <h3 className="text-lg font-semibold mb-2">No Meetings Recorded</h3>
                      <p className="text-muted-foreground">
                        Add meeting notes to capture requirements from discussions
                      </p>
                    </div>
                  ) : (
                    <div className="space-y-3">
                      {safeMeetings.map((meeting) => (
                        <div key={meeting.id} className="p-4 border rounded-lg">
                          <div className="flex items-start justify-between mb-2">
                            <h4 className="font-semibold">{meeting.title}</h4>
                            <span className="text-sm text-muted-foreground">
                              {new Date(meeting.date).toLocaleDateString()}
                            </span>
                          </div>
                          <p className="text-sm text-muted-foreground mb-2">
                            Participants: {meeting.participants.join(', ')}
                          </p>
                          <p className="text-sm">{meeting.transcript || meeting.notes || 'No notes'}</p>
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            )}

            {/* Emails Section */}
            {activeSection === 'emails' && (
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>Emails</CardTitle>
                      <CardDescription>Email communications with stakeholders</CardDescription>
                    </div>
                    <Button size="sm" onClick={() => setEmailModalOpen(true)}>
                      <Plus className="w-4 h-4 mr-2" />
                      Add Email
                    </Button>
                  </div>
                </CardHeader>
                <CardContent>
                  {safeEmails.length === 0 ? (
                    <div className="text-center py-12">
                      <Mail className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                      <h3 className="text-lg font-semibold mb-2">No Emails</h3>
                      <p className="text-muted-foreground">
                        Add email communications to extract requirements
                      </p>
                    </div>
                  ) : (
                    <div className="space-y-3">
                      {safeEmails.map((email) => (
                        <div key={email.id} className="p-4 border rounded-lg">
                          <div className="flex items-start justify-between mb-2">
                            <h4 className="font-semibold">{email.subject}</h4>
                            <span className="text-sm text-muted-foreground">
                              {new Date(email.date).toLocaleDateString()}
                            </span>
                          </div>
                          <p className="text-sm text-muted-foreground mb-2">
                            From: {email.from} → To: {email.to.join(', ')}
                          </p>
                          <p className="text-sm line-clamp-2">{email.body}</p>
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            )}

            {/* Chats Section */}
            {activeSection === 'chats' && (
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>Chats</CardTitle>
                      <CardDescription>Chat conversations about requirements</CardDescription>
                    </div>
                    <Button size="sm" onClick={() => setChatModalOpen(true)}>
                      <Plus className="w-4 h-4 mr-2" />
                      Add Chat
                    </Button>
                  </div>
                </CardHeader>
                <CardContent>
                  {safeChats.length === 0 ? (
                    <div className="text-center py-12">
                      <MessageSquare className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                      <h3 className="text-lg font-semibold mb-2">No Chats</h3>
                      <p className="text-muted-foreground">
                        Add chat conversations to capture informal requirements
                      </p>
                    </div>
                  ) : (
                    <div className="space-y-3">
                      {safeChats.map((chat) => (
                        <div key={chat.id} className="p-4 border rounded-lg">
                          <div className="flex items-start justify-between mb-2">
                            <div className="flex items-center gap-2">
                              <Badge variant="outline">{chat.platform}</Badge>
                              <span className="text-sm text-muted-foreground">
                                {new Date(chat.date).toLocaleDateString()}
                              </span>
                            </div>
                          </div>
                          <p className="text-sm text-muted-foreground mb-1">
                            Participants: {chat.participants.join(', ')}
                          </p>
                          <p className="text-sm line-clamp-3">{chat.messages}</p>
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            )}

            {/* Confusions Section */}
            {activeSection === 'confusions' && (
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>Confusions</CardTitle>
                      <CardDescription>Contradictions and ambiguities identified by AI</CardDescription>
                    </div>
                    <Button size="sm" onClick={() => setConfusionModalOpen(true)}>
                      <Plus className="w-4 h-4 mr-2" />
                      Add Confusion
                    </Button>
                  </div>
                </CardHeader>
                <CardContent>
                  {safeConfusions.length === 0 ? (
                    <div className="text-center py-12">
                      <AlertCircle className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                      <h3 className="text-lg font-semibold mb-2">No Confusions Identified</h3>
                      <p className="text-muted-foreground">
                        Click "Analyze with AI" to detect contradictions and ambiguities
                      </p>
                    </div>
                  ) : (
                    <div className="space-y-4">
                      {safeConfusions.map((confusion: any) => (
                        <div key={confusion.id} className="border rounded-lg p-4 bg-amber-50 border-amber-200">
                          <div className="flex items-start justify-between mb-3">
                            <Badge variant="outline" className="bg-amber-100 text-amber-800 border-amber-300">
                              {confusion.status}
                            </Badge>
                            <span className="text-xs text-muted-foreground">
                              {new Date(confusion.createdAt).toLocaleDateString()}
                            </span>
                          </div>
                          <h4 className="font-semibold mb-2 text-amber-900">{confusion.question}</h4>
                          <p className="text-sm text-amber-800 mb-3">{confusion.context}</p>
                          {confusion.sources && confusion.sources.length > 0 && (
                            <div className="mb-3">
                              <div className="text-xs font-medium text-amber-900 mb-1">Conflicting Sources:</div>
                              <div className="flex flex-wrap gap-1">
                                {confusion.sources.map((source: any, idx: number) => (
                                  <Badge key={idx} variant="outline" className="text-xs">
                                    {source.type}
                                  </Badge>
                                ))}
                              </div>
                            </div>
                          )}
                          {confusion.clarification && (
                            <div className="mt-3 p-3 bg-green-50 rounded border border-green-200">
                              <div className="font-medium text-sm mb-1 text-green-900">Clarification:</div>
                              <p className="text-sm text-green-800">{confusion.clarification}</p>
                              {confusion.resolvedAt && (
                                <p className="text-xs text-green-700 mt-1">
                                  Resolved on {new Date(confusion.resolvedAt).toLocaleDateString()}
                                </p>
                              )}
                            </div>
                          )}
                          {confusion.status === 'Open' && (
                            <div className="mt-3 flex justify-end">
                              <Button
                                size="sm"
                                variant="outline"
                                onClick={() => {
                                  setSelectedConfusion(confusion);
                                  setConfusionResolutionModalOpen(true);
                                }}
                              >
                                <CheckCircle className="w-4 h-4 mr-2" />
                                Clarify Confusion
                              </Button>
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            )}

            {/* Suggestions Section */}
            {activeSection === 'suggestions' && (
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>AI Suggestions</CardTitle>
                      <CardDescription>Recommendations and best practices from AI analysis</CardDescription>
                    </div>
                  </div>
                </CardHeader>
                <CardContent>
                  {safeSuggestions.length === 0 ? (
                    <div className="text-center py-12">
                      <Lightbulb className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                      <h3 className="text-lg font-semibold mb-2">No Suggestions Yet</h3>
                      <p className="text-muted-foreground">
                        Click "Analyze with AI" to get improvement recommendations
                      </p>
                    </div>
                  ) : (
                    <div className="space-y-4">
                      {safeSuggestions.map((suggestion: any) => (
                        <div key={suggestion.id} className="border rounded-lg p-4 hover:bg-gray-50">
                          <div className="flex items-start justify-between mb-3">
                            <div className="flex items-center gap-2">
                              <Badge
                                variant={
                                  suggestion.type === 'Best Practice' ? 'default' :
                                  suggestion.type === 'Risk Mitigation' ? 'destructive' :
                                  'secondary'
                                }
                              >
                                {suggestion.type}
                              </Badge>
                              <Badge variant="outline">{suggestion.status}</Badge>
                            </div>
                            <span className="text-xs text-muted-foreground">
                              {new Date(suggestion.createdAt).toLocaleDateString()}
                            </span>
                          </div>
                          <h4 className="font-semibold mb-2">{suggestion.title}</h4>
                          <p className="text-sm text-gray-700 mb-3">{suggestion.description}</p>
                          <div className="grid grid-cols-2 gap-3 text-sm">
                            {suggestion.impact && (
                              <div>
                                <span className="font-medium text-gray-600">Impact: </span>
                                <span className="text-gray-900">{suggestion.impact}</span>
                              </div>
                            )}
                            {suggestion.effort && (
                              <div>
                                <span className="font-medium text-gray-600">Effort: </span>
                                <span className="text-gray-900">{suggestion.effort}</span>
                              </div>
                            )}
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            )}

            {/* Security Requirements Section */}
            {activeSection === 'security-requirements' && (
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>Security Requirements</CardTitle>
                      <CardDescription>Authentication, authorization, encryption, and compliance</CardDescription>
                    </div>
                    <Button size="sm" onClick={() => setSecurityReqModalOpen(true)}>
                      <Plus className="w-4 h-4 mr-2" />
                      Add Requirement
                    </Button>
                  </div>
                </CardHeader>
                <CardContent>
                  {safeSecurityRequirements.length === 0 ? (
                    <div className="text-center py-12">
                      <Shield className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                      <h3 className="text-lg font-semibold mb-2">No Security Requirements</h3>
                      <p className="text-muted-foreground">
                        Click "Analyze with AI" to extract security requirements
                      </p>
                    </div>
                  ) : (
                    <div className="space-y-4">
                      {safeSecurityRequirements.map((secReq: any) => (
                        <div key={secReq.id} className="border rounded-lg p-4 bg-blue-50 border-blue-200">
                          <div className="flex items-start justify-between mb-3">
                            <div className="flex items-center gap-2">
                              <Badge
                                variant={
                                  secReq.level === 'Critical' ? 'destructive' :
                                  secReq.level === 'High' ? 'default' :
                                  'secondary'
                                }
                              >
                                {secReq.level}
                              </Badge>
                              <Badge variant="outline" className="bg-blue-100 text-blue-800">
                                {secReq.category}
                              </Badge>
                              <Badge variant="outline">{secReq.status}</Badge>
                            </div>
                            <span className="text-xs text-muted-foreground">
                              {new Date(secReq.createdAt).toLocaleDateString()}
                            </span>
                          </div>
                          <h4 className="font-semibold mb-2 text-blue-900">{secReq.title}</h4>
                          <p className="text-sm text-blue-800 mb-3">{secReq.description}</p>
                          {secReq.standard && (
                            <div className="mb-2">
                              <span className="text-xs font-medium text-blue-900">Standard: </span>
                              <span className="text-xs text-blue-800">{secReq.standard}</span>
                            </div>
                          )}
                          {secReq.controls && secReq.controls.length > 0 && (
                            <div className="mb-3">
                              <div className="text-xs font-medium text-blue-900 mb-1">Security Controls:</div>
                              <ul className="space-y-1">
                                {secReq.controls.map((control: string, idx: number) => (
                                  <li key={idx} className="text-sm text-blue-800">• {control}</li>
                                ))}
                              </ul>
                            </div>
                          )}
                          {secReq.testCriteria && secReq.testCriteria.length > 0 && (
                            <div>
                              <div className="text-xs font-medium text-blue-900 mb-1">Test Criteria:</div>
                              <ul className="space-y-1">
                                {secReq.testCriteria.map((criteria: string, idx: number) => (
                                  <li key={idx} className="text-sm text-blue-800">• {criteria}</li>
                                ))}
                              </ul>
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            )}

            {/* Threats (STRIDE) Section */}
            {activeSection === 'threats' && (
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>Threat Analysis (STRIDE)</CardTitle>
                      <CardDescription>Security threats identified using STRIDE methodology</CardDescription>
                    </div>
                    <Button size="sm" onClick={() => setThreatModalOpen(true)}>
                      <Plus className="w-4 h-4 mr-2" />
                      Add Threat
                    </Button>
                  </div>
                </CardHeader>
                <CardContent>
                  {safeThreats.length === 0 ? (
                    <div className="text-center py-12">
                      <AlertTriangle className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                      <h3 className="text-lg font-semibold mb-2">No Threats Identified</h3>
                      <p className="text-muted-foreground mb-4">
                        Click "Analyze with AI" to perform STRIDE threat analysis
                      </p>
                      <div className="grid grid-cols-2 gap-2 max-w-md mx-auto text-xs text-muted-foreground">
                        <div>• Spoofing</div>
                        <div>• Tampering</div>
                        <div>• Repudiation</div>
                        <div>• Information Disclosure</div>
                        <div>• Denial of Service</div>
                        <div>• Elevation of Privilege</div>
                      </div>
                    </div>
                  ) : (
                    <div className="space-y-4">
                      {safeThreats.map((threat: any) => (
                        <div key={threat.id} className="border rounded-lg p-4 bg-red-50 border-red-200">
                          <div className="flex items-start justify-between mb-3">
                            <div className="flex items-center gap-2">
                              <Badge
                                variant={
                                  threat.level === 'Critical' ? 'destructive' :
                                  threat.level === 'High' ? 'default' :
                                  'secondary'
                                }
                              >
                                {threat.level}
                              </Badge>
                              <Badge variant="outline" className="bg-red-100 text-red-800">
                                {threat.category}
                              </Badge>
                              <Badge variant="outline">{threat.status}</Badge>
                            </div>
                            <span className="text-xs text-muted-foreground">
                              {new Date(threat.createdAt).toLocaleDateString()}
                            </span>
                          </div>
                          <h4 className="font-semibold mb-2 text-red-900">{threat.threat}</h4>
                          <p className="text-sm text-red-800 mb-3">{threat.description}</p>
                          <div className="grid grid-cols-2 gap-3 mb-3 text-sm">
                            <div>
                              <span className="font-medium text-red-900">Likelihood: </span>
                              <span className="text-red-800">{threat.likelihood}</span>
                            </div>
                            <div>
                              <span className="font-medium text-red-900">Impact: </span>
                              <span className="text-red-800">{threat.impact}</span>
                            </div>
                          </div>
                          {threat.mitigations && threat.mitigations.length > 0 && (
                            <div className="p-3 bg-green-50 rounded border border-green-200">
                              <div className="font-medium text-sm mb-2 text-green-900">Mitigations:</div>
                              <ul className="space-y-1">
                                {threat.mitigations.map((mitigation: string, idx: number) => (
                                  <li key={idx} className="text-sm text-green-800">• {mitigation}</li>
                                ))}
                              </ul>
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            )}

            {/* NFRs Section */}
            {activeSection === 'nfrs' && (
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>Non-Functional Requirements</CardTitle>
                      <CardDescription>Performance, scalability, reliability, and compliance targets</CardDescription>
                    </div>
                    <Button size="sm" onClick={() => setNfrModalOpen(true)}>
                      <Plus className="w-4 h-4 mr-2" />
                      Add NFR
                    </Button>
                  </div>
                </CardHeader>
                <CardContent>
                  {safeNFRs.length === 0 ? (
                    <div className="text-center py-12">
                      <Zap className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                      <h3 className="text-lg font-semibold mb-2">No NFRs Defined</h3>
                      <p className="text-muted-foreground">
                        Click "Analyze with AI" to extract non-functional requirements
                      </p>
                    </div>
                  ) : (
                    <div className="space-y-4">
                      {safeNFRs.map((nfr: any) => (
                        <div key={nfr.id} className="border rounded-lg p-4 hover:bg-gray-50">
                          <div className="flex items-start justify-between mb-3">
                            <div className="flex items-center gap-2">
                              <Badge
                                variant={
                                  nfr.priority === 'Critical' ? 'destructive' :
                                  nfr.priority === 'High' ? 'default' :
                                  'secondary'
                                }
                              >
                                {nfr.priority}
                              </Badge>
                              <Badge variant="outline">{nfr.category}</Badge>
                              <Badge variant="outline">{nfr.status}</Badge>
                            </div>
                            <span className="text-xs text-muted-foreground">
                              {new Date(nfr.createdAt).toLocaleDateString()}
                            </span>
                          </div>
                          <h4 className="font-semibold mb-2">{nfr.requirement}</h4>
                          <div className="grid grid-cols-2 gap-3 text-sm">
                            {nfr.metric && (
                              <div>
                                <span className="font-medium text-gray-600">Metric: </span>
                                <span className="text-gray-900">{nfr.metric}</span>
                              </div>
                            )}
                            {nfr.target && (
                              <div>
                                <span className="font-medium text-gray-600">Target: </span>
                                <span className="text-gray-900">{nfr.target}</span>
                              </div>
                            )}
                            {nfr.measurement && (
                              <div className="col-span-2">
                                <span className="font-medium text-gray-600">Measurement: </span>
                                <span className="text-gray-900">{nfr.measurement}</span>
                              </div>
                            )}
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            )}

            {/* Cloud Governance Section */}
            {activeSection === 'cloud-governance' && (
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>Cloud Governance & Controls</CardTitle>
                      <CardDescription>Cost management, resource governance, security, compliance, and cloud best practices</CardDescription>
                    </div>
                    <Button size="sm" onClick={() => alert('Cloud governance form coming soon')}>
                      <Plus className="w-4 h-4 mr-2" />
                      Add Control
                    </Button>
                  </div>
                </CardHeader>
                <CardContent>
                  {safeCloudGovernance.length === 0 ? (
                    <div className="text-center py-12">
                      <Cloud className="w-16 h-16 mx-auto text-muted-foreground mb-4" />
                      <h3 className="text-lg font-semibold mb-2">No Cloud Governance Requirements</h3>
                      <p className="text-muted-foreground">
                        Click "Analyze with AI" to extract cloud governance requirements
                      </p>
                    </div>
                  ) : (
                    <div className="space-y-4">
                      {safeCloudGovernance.map((cloudGov: any) => (
                        <div key={cloudGov.id} className="border rounded-lg p-4 bg-indigo-50 border-indigo-200">
                          <div className="flex items-start justify-between mb-3">
                            <div className="flex items-center gap-2">
                              <Badge
                                variant={
                                  cloudGov.priority === 'Critical' ? 'destructive' :
                                  cloudGov.priority === 'High' ? 'default' :
                                  'secondary'
                                }
                              >
                                {cloudGov.priority}
                              </Badge>
                              <Badge variant="outline" className="bg-indigo-100 text-indigo-800">
                                {cloudGov.area}
                              </Badge>
                              <Badge variant="outline">{cloudGov.status}</Badge>
                            </div>
                            <span className="text-xs text-muted-foreground">
                              {new Date(cloudGov.createdAt).toLocaleDateString()}
                            </span>
                          </div>
                          <h4 className="font-semibold mb-2 text-indigo-900">{cloudGov.requirement}</h4>
                          <p className="text-sm text-indigo-800">{cloudGov.details}</p>
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            )}
          </div>
        </div>

        {/* Footer Info */}
        <div className="p-4 border-t bg-gray-50">
          <div className="text-sm text-muted-foreground text-center">
            <Brain className="w-4 h-4 inline-block mr-2" />
            <strong>Knowledge Hub:</strong> Upload documents, add meetings, emails, or chats.
            AI automatically extracts requirements from all sources.
          </div>
        </div>
      </div>

      {/* Document Upload Modal */}
      <Dialog open={documentModalOpen} onOpenChange={setDocumentModalOpen}>
        <DialogContent aria-describedby="document-upload-description">
          <DialogHeader>
            <DialogTitle>Upload Document</DialogTitle>
            <DialogDescription id="document-upload-description">
              Upload BRD, FRD, or other requirement documents for AI analysis
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleDocumentUpload} className="space-y-4">
            <div>
              <Label htmlFor="doc-name">Document Name *</Label>
              <Input id="doc-name" name="name" required placeholder="e.g., User Authentication BRD" />
            </div>
            <div>
              <Label htmlFor="doc-type">Document Type *</Label>
              <Select value={docType} onValueChange={setDocType} required>
                <SelectTrigger id="doc-type">
                  <SelectValue placeholder="Select type" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="BRD">BRD (Business Requirements)</SelectItem>
                  <SelectItem value="FRD">FRD (Functional Requirements)</SelectItem>
                  <SelectItem value="PRD">PRD (Product Requirements)</SelectItem>
                  <SelectItem value="Technical Spec">Technical Specification</SelectItem>
                  <SelectItem value="Design Doc">Design Document</SelectItem>
                  <SelectItem value="Other">Other</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label htmlFor="doc-file">File *</Label>
              <Input id="doc-file" name="file" type="file" required accept=".pdf,.doc,.docx,.txt" />
            </div>
            <div className="flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={() => setDocumentModalOpen(false)} disabled={isSubmitting}>Cancel</Button>
              <Button type="submit" disabled={isSubmitting || !docType}>
                {isSubmitting ? 'Uploading...' : 'Upload'}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      {/* Meeting Modal */}
      <Dialog open={meetingModalOpen} onOpenChange={setMeetingModalOpen}>
        <DialogContent aria-describedby="meeting-description">
          <DialogHeader>
            <DialogTitle>Add Meeting</DialogTitle>
            <DialogDescription id="meeting-description">
              Record meeting notes and discussions about requirements
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleMeetingSubmit} className="space-y-4">
            <div>
              <Label htmlFor="meeting-title">Meeting Title *</Label>
              <Input id="meeting-title" name="title" required placeholder="e.g., Requirements Review Meeting" />
            </div>
            <div>
              <Label htmlFor="meeting-date">Date *</Label>
              <Input id="meeting-date" name="date" type="date" required />
            </div>
            <div>
              <Label htmlFor="meeting-participants">Participants (comma-separated) *</Label>
              <Input id="meeting-participants" name="participants" required placeholder="John Doe, Jane Smith, Bob Johnson" />
            </div>
            <div>
              <Label htmlFor="meeting-notes">Meeting Notes *</Label>
              <Textarea id="meeting-notes" name="notes" required rows={5} placeholder="Key discussion points, decisions made, action items..." />
            </div>
            <div className="flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={() => setMeetingModalOpen(false)} disabled={isSubmitting}>Cancel</Button>
              <Button type="submit" disabled={isSubmitting}>
                {isSubmitting ? 'Adding...' : 'Add Meeting'}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      {/* Email Modal */}
      <Dialog open={emailModalOpen} onOpenChange={setEmailModalOpen}>
        <DialogContent aria-describedby="email-description">
          <DialogHeader>
            <DialogTitle>Add Email</DialogTitle>
            <DialogDescription id="email-description">
              Add email communications with stakeholders
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleEmailSubmit} className="space-y-4">
            <div>
              <Label htmlFor="email-subject">Subject *</Label>
              <Input id="email-subject" name="subject" required placeholder="e.g., Login Feature Requirements" />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="email-sender">From *</Label>
                <Input id="email-sender" name="sender" required placeholder="sender@example.com" />
              </div>
              <div>
                <Label htmlFor="email-receiver">To *</Label>
                <Input id="email-receiver" name="receiver" required placeholder="receiver@example.com" />
              </div>
            </div>
            <div>
              <Label htmlFor="email-date">Date *</Label>
              <Input id="email-date" name="date" type="date" required />
            </div>
            <div>
              <Label htmlFor="email-body">Email Body *</Label>
              <Textarea id="email-body" name="body" required rows={6} placeholder="Paste the email content here..." />
            </div>
            <div className="flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={() => setEmailModalOpen(false)} disabled={isSubmitting}>Cancel</Button>
              <Button type="submit" disabled={isSubmitting}>
                {isSubmitting ? 'Adding...' : 'Add Email'}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      {/* Chat Modal */}
      <Dialog open={chatModalOpen} onOpenChange={setChatModalOpen}>
        <DialogContent aria-describedby="chat-description">
          <DialogHeader>
            <DialogTitle>Add Chat</DialogTitle>
            <DialogDescription id="chat-description">
              Add chat conversations from Slack, Teams, or other platforms
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleChatSubmit} className="space-y-4">
            <div>
              <Label htmlFor="chat-platform">Platform *</Label>
              <Select value={chatPlatform} onValueChange={setChatPlatform} required>
                <SelectTrigger id="chat-platform">
                  <SelectValue placeholder="Select platform" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Slack">Slack</SelectItem>
                  <SelectItem value="Microsoft Teams">Microsoft Teams</SelectItem>
                  <SelectItem value="Google Chat">Google Chat</SelectItem>
                  <SelectItem value="Discord">Discord</SelectItem>
                  <SelectItem value="WhatsApp">WhatsApp</SelectItem>
                  <SelectItem value="Other">Other</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label htmlFor="chat-date">Date *</Label>
              <Input id="chat-date" name="date" type="date" required />
            </div>
            <div>
              <Label htmlFor="chat-participants">Participants (comma-separated) *</Label>
              <Input id="chat-participants" name="participants" required placeholder="John, Jane, Bob" />
            </div>
            <div>
              <Label htmlFor="chat-content">Chat Content *</Label>
              <Textarea id="chat-content" name="content" required rows={8} placeholder="Paste the chat conversation here..." />
            </div>
            <div className="flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={() => setChatModalOpen(false)} disabled={isSubmitting}>Cancel</Button>
              <Button type="submit" disabled={isSubmitting || !chatPlatform}>
                {isSubmitting ? 'Adding...' : 'Add Chat'}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      {/* Gap Modal */}
      <Dialog open={gapModalOpen} onOpenChange={setGapModalOpen}>
        <DialogContent aria-describedby="gap-description">
          <DialogHeader>
            <DialogTitle>Add Gap</DialogTitle>
            <DialogDescription id="gap-description">
              Identify a missing or incomplete requirement
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleGapSubmit} className="space-y-4">
            <div>
              <Label htmlFor="gap-title">Title *</Label>
              <Input id="gap-title" name="title" required placeholder="e.g., Missing authentication flow" />
            </div>
            <div>
              <Label htmlFor="gap-desc">Description *</Label>
              <Textarea id="gap-desc" name="description" required rows={3} placeholder="Describe what's missing or incomplete..." />
            </div>
            <div>
              <Label htmlFor="gap-severity">Severity *</Label>
              <Select value={gapSeverity} onValueChange={setGapSeverity} required>
                <SelectTrigger id="gap-severity">
                  <SelectValue placeholder="Select severity" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Critical">Critical</SelectItem>
                  <SelectItem value="High">High</SelectItem>
                  <SelectItem value="Medium">Medium</SelectItem>
                  <SelectItem value="Low">Low</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div>
              <Label htmlFor="gap-suggestions">Suggestions (one per line)</Label>
              <Textarea id="gap-suggestions" name="suggestions" rows={3} placeholder="Suggestion 1&#10;Suggestion 2" />
            </div>
            <div className="flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={() => setGapModalOpen(false)} disabled={isSubmitting}>Cancel</Button>
              <Button type="submit" disabled={isSubmitting || !gapSeverity}>
                {isSubmitting ? 'Adding...' : 'Add Gap'}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      {/* Confusion Modal */}
      <Dialog open={confusionModalOpen} onOpenChange={setConfusionModalOpen}>
        <DialogContent aria-describedby="confusion-description">
          <DialogHeader>
            <DialogTitle>Add Confusion</DialogTitle>
            <DialogDescription id="confusion-description">
              Document a contradiction or ambiguity that needs clarification
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleConfusionSubmit} className="space-y-4">
            <div>
              <Label htmlFor="confusion-question">Question *</Label>
              <Input id="confusion-question" name="question" required placeholder="e.g., Should users login with email or username?" />
            </div>
            <div>
              <Label htmlFor="confusion-context">Context *</Label>
              <Textarea id="confusion-context" name="context" required rows={4} placeholder="Describe the conflicting information or ambiguity..." />
            </div>
            <div className="flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={() => setConfusionModalOpen(false)} disabled={isSubmitting}>Cancel</Button>
              <Button type="submit" disabled={isSubmitting}>
                {isSubmitting ? 'Adding...' : 'Add Confusion'}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      {/* Security Requirement Modal */}
      <Dialog open={securityReqModalOpen} onOpenChange={setSecurityReqModalOpen}>
        <DialogContent aria-describedby="security-req-description" className="max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Add Security Requirement</DialogTitle>
            <DialogDescription id="security-req-description">
              Define authentication, authorization, encryption, or compliance requirements
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleSecurityReqSubmit} className="space-y-4">
            <div>
              <Label htmlFor="security-title">Title *</Label>
              <Input id="security-title" name="title" required placeholder="e.g., Multi-factor authentication" />
            </div>
            <div>
              <Label htmlFor="security-desc">Description *</Label>
              <Textarea id="security-desc" name="description" required rows={3} placeholder="Describe the security requirement..." />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="security-category">Category *</Label>
                <Select value={securityCategory} onValueChange={setSecurityCategory} required>
                  <SelectTrigger id="security-category">
                    <SelectValue placeholder="Select category" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="Authentication">Authentication</SelectItem>
                    <SelectItem value="Authorization">Authorization</SelectItem>
                    <SelectItem value="Data Protection">Data Protection</SelectItem>
                    <SelectItem value="Network Security">Network Security</SelectItem>
                    <SelectItem value="Compliance">Compliance</SelectItem>
                    <SelectItem value="Other">Other</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div>
                <Label htmlFor="security-level">Level *</Label>
                <Select value={securityLevel} onValueChange={setSecurityLevel} required>
                  <SelectTrigger id="security-level">
                    <SelectValue placeholder="Select level" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="Critical">Critical</SelectItem>
                    <SelectItem value="High">High</SelectItem>
                    <SelectItem value="Medium">Medium</SelectItem>
                    <SelectItem value="Low">Low</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div>
              <Label htmlFor="security-standard">Standard (optional)</Label>
              <Input id="security-standard" name="standard" placeholder="e.g., OWASP Top 10, ISO 27001" />
            </div>
            <div>
              <Label htmlFor="security-controls">Security Controls (one per line)</Label>
              <Textarea id="security-controls" name="controls" rows={3} placeholder="Control 1&#10;Control 2" />
            </div>
            <div>
              <Label htmlFor="security-test-criteria">Test Criteria (one per line)</Label>
              <Textarea id="security-test-criteria" name="testCriteria" rows={3} placeholder="Test criterion 1&#10;Test criterion 2" />
            </div>
            <div className="flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={() => setSecurityReqModalOpen(false)} disabled={isSubmitting}>Cancel</Button>
              <Button type="submit" disabled={isSubmitting || !securityCategory || !securityLevel}>
                {isSubmitting ? 'Adding...' : 'Add Requirement'}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      {/* Threat Modal */}
      <Dialog open={threatModalOpen} onOpenChange={setThreatModalOpen}>
        <DialogContent aria-describedby="threat-description" className="max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Add Threat</DialogTitle>
            <DialogDescription id="threat-description">
              Identify a security threat using STRIDE methodology
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleThreatSubmit} className="space-y-4">
            <div>
              <Label htmlFor="threat-name">Threat Name *</Label>
              <Input id="threat-name" name="threat" required placeholder="e.g., SQL Injection Attack" />
            </div>
            <div>
              <Label htmlFor="threat-desc">Description *</Label>
              <Textarea id="threat-desc" name="description" required rows={3} placeholder="Describe the threat..." />
            </div>
            <div>
              <Label htmlFor="threat-category">STRIDE Category *</Label>
              <Select value={threatCategory} onValueChange={setThreatCategory} required>
                <SelectTrigger id="threat-category">
                  <SelectValue placeholder="Select category" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Spoofing">Spoofing</SelectItem>
                  <SelectItem value="Tampering">Tampering</SelectItem>
                  <SelectItem value="Repudiation">Repudiation</SelectItem>
                  <SelectItem value="Information Disclosure">Information Disclosure</SelectItem>
                  <SelectItem value="Denial of Service">Denial of Service</SelectItem>
                  <SelectItem value="Elevation of Privilege">Elevation of Privilege</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="threat-level">Threat Level *</Label>
                <Select value={threatLevel} onValueChange={setThreatLevel} required>
                  <SelectTrigger id="threat-level">
                    <SelectValue placeholder="Select level" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="Critical">Critical</SelectItem>
                    <SelectItem value="High">High</SelectItem>
                    <SelectItem value="Medium">Medium</SelectItem>
                    <SelectItem value="Low">Low</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div>
                <Label htmlFor="threat-likelihood">Likelihood *</Label>
                <Select value={threatLikelihood} onValueChange={setThreatLikelihood} required>
                  <SelectTrigger id="threat-likelihood">
                    <SelectValue placeholder="Select likelihood" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="Very High">Very High</SelectItem>
                    <SelectItem value="High">High</SelectItem>
                    <SelectItem value="Medium">Medium</SelectItem>
                    <SelectItem value="Low">Low</SelectItem>
                    <SelectItem value="Very Low">Very Low</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            <div>
              <Label htmlFor="threat-impact">Impact *</Label>
              <Input id="threat-impact" name="impact" required placeholder="Describe the potential impact..." />
            </div>
            <div>
              <Label htmlFor="threat-mitigations">Mitigations (one per line)</Label>
              <Textarea id="threat-mitigations" name="mitigations" rows={3} placeholder="Mitigation 1&#10;Mitigation 2" />
            </div>
            <div className="flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={() => setThreatModalOpen(false)} disabled={isSubmitting}>Cancel</Button>
              <Button type="submit" disabled={isSubmitting || !threatCategory || !threatLevel || !threatLikelihood}>
                {isSubmitting ? 'Adding...' : 'Add Threat'}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      {/* Gap Resolution Modal */}
      <Dialog open={gapResolutionModalOpen} onOpenChange={setGapResolutionModalOpen}>
        <DialogContent aria-describedby="gap-resolution-description">
          <DialogHeader>
            <DialogTitle>Resolve Gap</DialogTitle>
            <DialogDescription id="gap-resolution-description">
              Provide a resolution for this gap. AI will generate refined requirements based on your input.
            </DialogDescription>
          </DialogHeader>
          {selectedGap && (
            <div className="space-y-4 mb-4">
              <div className="p-3 bg-gray-50 rounded border">
                <div className="font-medium text-sm mb-1">Gap: {selectedGap.title}</div>
                <p className="text-sm text-gray-700">{selectedGap.description}</p>
              </div>
            </div>
          )}
          <form onSubmit={handleGapResolution} className="space-y-4">
            <div>
              <Label htmlFor="gap-resolution">Resolution *</Label>
              <Textarea
                id="gap-resolution"
                name="resolution"
                required
                rows={4}
                placeholder="Describe how you want to resolve this gap..."
              />
            </div>
            <div className="flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={() => setGapResolutionModalOpen(false)} disabled={isResolving}>
                Cancel
              </Button>
              <Button type="submit" disabled={isResolving}>
                {isResolving ? 'Resolving...' : 'Resolve & Refine Requirements'}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      {/* Confusion Clarification Modal */}
      <Dialog open={confusionResolutionModalOpen} onOpenChange={setConfusionResolutionModalOpen}>
        <DialogContent aria-describedby="confusion-clarification-description">
          <DialogHeader>
            <DialogTitle>Clarify Confusion</DialogTitle>
            <DialogDescription id="confusion-clarification-description">
              Provide clarification for this confusion. AI will generate refined requirements based on your input.
            </DialogDescription>
          </DialogHeader>
          {selectedConfusion && (
            <div className="space-y-4 mb-4">
              <div className="p-3 bg-amber-50 rounded border border-amber-200">
                <div className="font-medium text-sm mb-1 text-amber-900">Question: {selectedConfusion.question}</div>
                <p className="text-sm text-amber-800">{selectedConfusion.context}</p>
              </div>
            </div>
          )}
          <form onSubmit={handleConfusionResolution} className="space-y-4">
            <div>
              <Label htmlFor="confusion-clarification">Clarification *</Label>
              <Textarea
                id="confusion-clarification"
                name="clarification"
                required
                rows={4}
                placeholder="Provide clarification to resolve this confusion..."
              />
            </div>
            <div className="flex justify-end gap-2">
              <Button type="button" variant="outline" onClick={() => setConfusionResolutionModalOpen(false)} disabled={isResolving}>
                Cancel
              </Button>
              <Button type="submit" disabled={isResolving}>
                {isResolving ? 'Clarifying...' : 'Clarify & Refine Requirements'}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      {/* NFR Modal - Comprehensive Form */}
      <Dialog open={nfrModalOpen} onOpenChange={setNfrModalOpen}>
        <DialogContent aria-describedby="nfr-description" className="max-w-4xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Add Non-Functional Requirements</DialogTitle>
            <DialogDescription id="nfr-description">
              Define comprehensive performance, scalability, availability, and quality targets
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <NFRStep data={nfrData} onChange={setNfrData} />
          </div>
          <div className="flex justify-end gap-2 pt-4 border-t">
            <Button type="button" variant="outline" onClick={() => setNfrModalOpen(false)} disabled={isSubmitting}>
              Cancel
            </Button>
            <Button onClick={handleNFRSubmit} disabled={isSubmitting || !nfrData}>
              {isSubmitting ? 'Saving...' : 'Save NFRs'}
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
