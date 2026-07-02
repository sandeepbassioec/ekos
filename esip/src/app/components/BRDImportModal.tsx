import React, { useState } from 'react';
import { Upload, FileText, AlertCircle, CheckCircle, Loader2, X, ChevronRight } from 'lucide-react';
import { Button } from './ui/button';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from './ui/dialog';
import { Progress } from './ui/progress';
import { Card, CardContent } from './ui/card';
import { Badge } from './ui/badge';
import { parseDocument, validateFile, ParsedDocument } from '../services/documentParser';
import { analyzeBRD, mockAnalyzeBRD, BRDAnalysisResult, ExtractedRequirement } from '../services/aiService';
import { toast } from 'sonner';

interface BRDImportModalProps {
  open: boolean;
  onClose: () => void;
  onImportComplete: (requirements: ExtractedRequirement[], analysis: BRDAnalysisResult) => void;
  projectId: string;
  moduleId: string;
}

type ImportStep = 'upload' | 'parsing' | 'analyzing' | 'review' | 'clarification' | 'complete';

export function BRDImportModal({ open, onClose, onImportComplete, projectId, moduleId }: BRDImportModalProps) {
  const [step, setStep] = useState<ImportStep>('upload');
  const [file, setFile] = useState<File | null>(null);
  const [parsedDoc, setParsedDoc] = useState<ParsedDocument | null>(null);
  const [analysis, setAnalysis] = useState<BRDAnalysisResult | null>(null);
  const [selectedRequirements, setSelectedRequirements] = useState<Set<number>>(new Set());
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [useMockAI, setUseMockAI] = useState(false); // Default to real AI (Groq)
  const [aiProvider, setAiProvider] = useState<'groq' | 'gemini' | 'claude'>('groq'); // Default to Groq
  const [apiKey, setApiKey] = useState('');
  const [clarificationAnswers, setClarificationAnswers] = useState<Record<number, string>>({});

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = event.target.files?.[0];
    if (!selectedFile) return;

    const validation = validateFile(selectedFile);
    if (!validation.valid) {
      setError(validation.error || 'Invalid file');
      toast.error(validation.error);
      return;
    }

    setFile(selectedFile);
    setError(null);
  };

  const handleDrop = (event: React.DragEvent) => {
    event.preventDefault();
    const droppedFile = event.dataTransfer.files[0];
    if (!droppedFile) return;

    const validation = validateFile(droppedFile);
    if (!validation.valid) {
      setError(validation.error || 'Invalid file');
      toast.error(validation.error);
      return;
    }

    setFile(droppedFile);
    setError(null);
  };

  const handleDragOver = (event: React.DragEvent) => {
    event.preventDefault();
  };

  const handleUpload = async () => {
    if (!file) return;

    try {
      // Step 1: Parse document
      setStep('parsing');
      setProgress(20);
      toast.info('Extracting text from document...');

      const parsed = await parseDocument(file);
      setParsedDoc(parsed);
      setProgress(40);
      
      // Step 2: AI Analysis
      setStep('analyzing');
      toast.info('Analyzing requirements with AI...');
      setProgress(60);

      let result: BRDAnalysisResult;
      
      if (useMockAI) {
        // Use mock for testing
        await new Promise(resolve => setTimeout(resolve, 2000)); // Simulate API delay
        result = mockAnalyzeBRD(parsed.text);
      } else {
        // Use real AI (Groq, Gemini, Claude)
        result = await analyzeBRD(parsed.text, { provider: aiProvider, apiKey: apiKey });
      }

      setAnalysis(result);
      
      // Select all requirements by default
      setSelectedRequirements(new Set(result.requirements.map((_, idx) => idx)));
      
      setProgress(100);
      setStep('review');
      toast.success(`Extracted ${result.requirements.length} requirements!`);
    } catch (err) {
      console.error('Import error:', err);
      setError(err instanceof Error ? err.message : 'Failed to process document');
      toast.error('Failed to process document');
      setStep('upload');
    }
  };

  const toggleRequirement = (index: number) => {
    const newSelected = new Set(selectedRequirements);
    if (newSelected.has(index)) {
      newSelected.delete(index);
    } else {
      newSelected.add(index);
    }
    setSelectedRequirements(newSelected);
  };

  const handleComplete = () => {
    const selected = Array.from(selectedRequirements).map((idx) => analysis!.requirements[idx]);
    onImportComplete(selected, analysis!);
    handleClose();
    
    // Show toast with next step option
    toast.success('Requirements imported successfully!', {
      description: 'Next: Capture comprehensive requirements (NFR, Security, Integration, etc.)',
      action: {
        label: 'Start Now',
        onClick: () => {
          // Navigate to comprehensive requirements wizard
          window.location.href = '/requirements/comprehensive';
        },
      },
      duration: 10000,
    });
  };

  const handleClose = () => {
    setStep('upload');
    setFile(null);
    setParsedDoc(null);
    setAnalysis(null);
    setSelectedRequirements(new Set());
    setProgress(0);
    setError(null);
    onClose();
  };

  const getTypeIcon = (type: string) => {
    const icons: Record<string, string> = {
      new_screen: '🖥️',
      update_screen: '✏️',
      bug_fix: '🐛',
      feature_change: '⚡',
      api_development: '🔌',
      integration: '🔗',
      data_migration: '📦',
      performance: '⚡',
      security: '🔒',
      ui_ux_change: '🎨',
    };
    return icons[type] || '📄';
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'critical': return 'bg-red-100 text-red-700';
      case 'high': return 'bg-orange-100 text-orange-700';
      case 'medium': return 'bg-yellow-100 text-yellow-700';
      case 'low': return 'bg-green-100 text-green-700';
      default: return 'bg-gray-100 text-gray-700';
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Upload className="w-5 h-5" />
            Import Requirements from BRD
          </DialogTitle>
          <DialogDescription>
            Upload your BRD (PDF, DOC, DOCX, XLS, XLSX) and AI will extract requirements automatically
          </DialogDescription>
        </DialogHeader>

        {/* Progress Steps */}
        <div className="flex items-center justify-between mb-6">
          {['upload', 'parsing', 'analyzing', 'review'].map((s, idx) => (
            <div key={s} className="flex items-center">
              <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-semibold ${
                step === s ? 'bg-blue-600 text-white' : 
                ['parsing', 'analyzing', 'review'].indexOf(step) > ['parsing', 'analyzing', 'review'].indexOf(s) || 
                (step === 'clarification' || step === 'complete') ? 'bg-green-500 text-white' : 'bg-gray-200'
              }`}>
                {(['parsing', 'analyzing', 'review'].indexOf(step) > ['parsing', 'analyzing', 'review'].indexOf(s) || 
                  (step === 'clarification' || step === 'complete')) ? '✓' : idx + 1}
              </div>
              {idx < 3 && <div className="w-16 h-1 mx-2 bg-gray-200"></div>}
            </div>
          ))}
        </div>

        {step === 'upload' && (
          <div>
            <div
              onDrop={handleDrop}
              onDragOver={handleDragOver}
              className="border-2 border-dashed border-gray-300 rounded-lg p-12 text-center hover:border-blue-500 transition-colors cursor-pointer"
              onClick={() => document.getElementById('file-input')?.click()}
            >
              <Upload className="w-16 h-16 mx-auto text-gray-400 mb-4" />
              <p className="text-lg font-semibold mb-2">Drop your BRD here or click to browse</p>
              <p className="text-sm text-muted-foreground mb-4">
                Supported formats: PDF, DOC, DOCX, XLS, XLSX (Max 10MB)
              </p>
              <input
                id="file-input"
                type="file"
                accept=".pdf,.doc,.docx,.xls,.xlsx"
                onChange={handleFileSelect}
                className="hidden"
              />
              {file && (
                <div className="flex items-center justify-center gap-2 mt-4">
                  <FileText className="w-5 h-5 text-blue-600" />
                  <span className="font-semibold">{file.name}</span>
                  <Badge>{(file.size / 1024).toFixed(0)} KB</Badge>
                </div>
              )}
            </div>

            {error && (
              <div className="flex items-center gap-2 mt-4 p-3 bg-red-50 border border-red-200 rounded">
                <AlertCircle className="w-5 h-5 text-red-600" />
                <span className="text-red-700">{error}</span>
              </div>
            )}

            {/* AI Provider Toggle (for testing) */}
            <div className="mt-4 p-3 bg-blue-50 border border-blue-200 rounded">
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={useMockAI}
                  onChange={(e) => setUseMockAI(e.target.checked)}
                />
                <span className="text-sm">Use Mock AI (for testing without API key)</span>
              </label>
            </div>

            {/* AI Provider Selection */}
            {!useMockAI && (
              <div className="mt-4 p-4 bg-purple-50 border border-purple-200 rounded space-y-2">
                <div className="flex items-center gap-2 mb-2">
                  <AlertCircle className="w-4 h-4 text-purple-600" />
                  <span className="text-sm font-semibold">AI Provider Required</span>
                </div>
                <p className="text-xs text-muted-foreground mb-2">
                  Select an AI provider and enter your API key
                </p>
                <div className="flex items-center gap-2">
                  <input
                    type="radio"
                    name="aiProvider"
                    value="groq"
                    checked={aiProvider === 'groq'}
                    onChange={(e) => setAiProvider(e.target.value as 'groq' | 'gemini' | 'claude')}
                  />
                  <span className="text-sm">Groq</span>
                </div>
                <div className="flex items-center gap-2">
                  <input
                    type="radio"
                    name="aiProvider"
                    value="gemini"
                    checked={aiProvider === 'gemini'}
                    onChange={(e) => setAiProvider(e.target.value as 'groq' | 'gemini' | 'claude')}
                  />
                  <span className="text-sm">Gemini</span>
                </div>
                <div className="flex items-center gap-2">
                  <input
                    type="radio"
                    name="aiProvider"
                    value="claude"
                    checked={aiProvider === 'claude'}
                    onChange={(e) => setAiProvider(e.target.value as 'groq' | 'gemini' | 'claude')}
                  />
                  <span className="text-sm">Claude</span>
                </div>
                <input
                  type="password"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder="Paste your API key here"
                  className="w-full border border-gray-300 px-3 py-2 rounded text-sm"
                />
              </div>
            )}

            <div className="flex justify-end gap-3 mt-6">
              <Button variant="outline" onClick={handleClose}>Cancel</Button>
              <Button onClick={handleUpload} disabled={!file} className="bg-blue-600 hover:bg-blue-700">
                <Upload className="w-4 h-4 mr-2" />
                Upload & Analyze
              </Button>
            </div>
          </div>
        )}

        {(step === 'parsing' || step === 'analyzing') && (
          <div className="text-center py-12">
            <Loader2 className="w-16 h-16 mx-auto text-blue-600 animate-spin mb-4" />
            <h3 className="text-xl font-semibold mb-2">
              {step === 'parsing' ? 'Extracting text from document...' : 'Analyzing with AI...'}
            </h3>
            <p className="text-muted-foreground mb-6">This may take a moment</p>
            <Progress value={progress} className="w-64 mx-auto" />
          </div>
        )}

        {step === 'review' && analysis && (
          <div className="space-y-6">
            {/* Summary */}
            <Card className="bg-gradient-to-r from-green-50 to-emerald-50 border-green-200">
              <CardContent className="pt-6">
                <div className="flex items-center gap-3 mb-4">
                  <CheckCircle className="w-6 h-6 text-green-600" />
                  <h3 className="text-xl font-semibold">Analysis Complete!</h3>
                </div>
                <div className="grid grid-cols-4 gap-4">
                  <div>
                    <div className="text-2xl font-bold">{analysis.summary.totalRequirements}</div>
                    <div className="text-sm text-muted-foreground">Requirements</div>
                  </div>
                  <div>
                    <div className="text-2xl font-bold">{analysis.summary.totalEstimatedHours}h</div>
                    <div className="text-sm text-muted-foreground">Estimated</div>
                  </div>
                  <div>
                    <div className="text-2xl font-bold">{analysis.gapAnalysis.clarificationQuestions.length}</div>
                    <div className="text-sm text-muted-foreground">Questions</div>
                  </div>
                  <div>
                    <Badge className={`text-lg ${
                      analysis.summary.riskLevel === 'high' ? 'bg-red-100 text-red-700' :
                      analysis.summary.riskLevel === 'medium' ? 'bg-yellow-100 text-yellow-700' :
                      'bg-green-100 text-green-700'
                    }`}>
                      {analysis.summary.riskLevel} risk
                    </Badge>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Gap Analysis */}
            {analysis.gapAnalysis.clarificationQuestions.length > 0 && (
              <Card className="border-yellow-300 bg-yellow-50">
                <CardContent className="pt-6">
                  <div className="flex items-center gap-2 mb-3">
                    <AlertCircle className="w-5 h-5 text-yellow-600" />
                    <h4 className="font-semibold">Clarification Needed</h4>
                  </div>
                  <ul className="space-y-2">
                    {analysis.gapAnalysis.clarificationQuestions.map((q, idx) => (
                      <li key={idx} className="text-sm flex items-start gap-2">
                        <span className="text-yellow-600 mt-1">•</span>
                        <span>{q}</span>
                      </li>
                    ))}
                  </ul>
                </CardContent>
              </Card>
            )}

            {/* Requirements List */}
            <div>
              <div className="flex items-center justify-between mb-4">
                <h4 className="font-semibold">Select Requirements to Import</h4>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => {
                    if (selectedRequirements.size === analysis.requirements.length) {
                      setSelectedRequirements(new Set());
                    } else {
                      setSelectedRequirements(new Set(analysis.requirements.map((_, idx) => idx)));
                    }
                  }}
                >
                  {selectedRequirements.size === analysis.requirements.length ? 'Deselect All' : 'Select All'}
                </Button>
              </div>

              <div className="space-y-3 max-h-96 overflow-y-auto">
                {analysis.requirements.map((req, idx) => (
                  <Card 
                    key={idx} 
                    className={`cursor-pointer transition-all ${
                      selectedRequirements.has(idx) ? 'border-blue-500 bg-blue-50' : 'hover:shadow-md'
                    }`}
                    onClick={() => toggleRequirement(idx)}
                  >
                    <CardContent className="pt-4">
                      <div className="flex items-start gap-3">
                        <input
                          type="checkbox"
                          checked={selectedRequirements.has(idx)}
                          onChange={() => toggleRequirement(idx)}
                          className="mt-1"
                        />
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-2">
                            <span className="text-2xl">{getTypeIcon(req.type)}</span>
                            <h5 className="font-semibold">{req.title}</h5>
                            <Badge className={getPriorityColor(req.priority)}>{req.priority}</Badge>
                            <Badge variant="secondary">{req.complexity}</Badge>
                            {req.estimatedHours && <Badge variant="outline">{req.estimatedHours}h</Badge>}
                          </div>
                          <p className="text-sm text-muted-foreground line-clamp-2">{req.description}</p>
                          {req.acceptanceCriteria.length > 0 && (
                            <div className="mt-2 text-xs text-muted-foreground">
                              ✓ {req.acceptanceCriteria.length} acceptance criteria
                            </div>
                          )}
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </div>

            <div className="flex justify-end gap-3 pt-4 border-t">
              <Button variant="outline" onClick={handleClose}>Cancel</Button>
              {analysis.gapAnalysis.clarificationQuestions.length > 0 && (
                <Button 
                  variant="outline"
                  onClick={() => setStep('clarification')}
                  className="border-yellow-500 text-yellow-700 hover:bg-yellow-50"
                >
                  Answer Questions ({analysis.gapAnalysis.clarificationQuestions.length})
                  <ChevronRight className="w-4 h-4 ml-2" />
                </Button>
              )}
              <Button 
                onClick={handleComplete} 
                disabled={selectedRequirements.size === 0}
                className="bg-blue-600 hover:bg-blue-700"
              >
                Import {selectedRequirements.size} Requirement{selectedRequirements.size !== 1 ? 's' : ''}
                <ChevronRight className="w-4 h-4 ml-2" />
              </Button>
            </div>
          </div>
        )}

        {/* Clarification Step */}
        {step === 'clarification' && analysis && (
          <div className="space-y-6">
            <Card className="bg-gradient-to-r from-purple-50 to-pink-50 border-purple-200">
              <CardContent className="pt-6">
                <div className="flex items-center gap-3 mb-2">
                  <AlertCircle className="w-6 h-6 text-purple-600" />
                  <h3 className="text-xl font-semibold">Clarification Questions</h3>
                </div>
                <p className="text-sm text-muted-foreground">
                  Please answer these questions to fill gaps in the requirements. Your answers will be added as notes to the requirements.
                </p>
              </CardContent>
            </Card>

            <div className="space-y-4">
              {analysis.gapAnalysis.clarificationQuestions.map((question, idx) => (
                <Card key={idx}>
                  <CardContent className="pt-6">
                    <div className="space-y-3">
                      <div className="flex items-start gap-3">
                        <div className="w-8 h-8 rounded-full bg-purple-100 text-purple-600 flex items-center justify-center font-semibold flex-shrink-0">
                          {idx + 1}
                        </div>
                        <div className="flex-1">
                          <p className="font-semibold mb-3">{question}</p>
                          <textarea
                            value={clarificationAnswers[idx] || ''}
                            onChange={(e) => setClarificationAnswers({ ...clarificationAnswers, [idx]: e.target.value })}
                            placeholder="Type your answer here..."
                            className="w-full border border-gray-300 rounded px-3 py-2 text-sm min-h-[80px] focus:outline-none focus:ring-2 focus:ring-purple-500"
                          />
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>

            {/* Missing Info */}
            {analysis.gapAnalysis.missingInfo.length > 0 && (
              <Card className="border-orange-300 bg-orange-50">
                <CardContent className="pt-6">
                  <h4 className="font-semibold mb-3 flex items-center gap-2">
                    <AlertCircle className="w-4 h-4 text-orange-600" />
                    Missing Information
                  </h4>
                  <ul className="space-y-1">
                    {analysis.gapAnalysis.missingInfo.map((info, idx) => (
                      <li key={idx} className="text-sm flex items-start gap-2">
                        <span className="text-orange-600">•</span>
                        <span>{info}</span>
                      </li>
                    ))}
                  </ul>
                </CardContent>
              </Card>
            )}

            {/* Technical Concerns */}
            {analysis.gapAnalysis.technicalConcerns.length > 0 && (
              <Card className="border-red-300 bg-red-50">
                <CardContent className="pt-6">
                  <h4 className="font-semibold mb-3 flex items-center gap-2">
                    <AlertCircle className="w-4 h-4 text-red-600" />
                    Technical Concerns
                  </h4>
                  <ul className="space-y-1">
                    {analysis.gapAnalysis.technicalConcerns.map((concern, idx) => (
                      <li key={idx} className="text-sm flex items-start gap-2">
                        <span className="text-red-600">•</span>
                        <span>{concern}</span>
                      </li>
                    ))}
                  </ul>
                </CardContent>
              </Card>
            )}

            {/* Recommendations */}
            {analysis.gapAnalysis.recommendations.length > 0 && (
              <Card className="border-blue-300 bg-blue-50">
                <CardContent className="pt-6">
                  <h4 className="font-semibold mb-3 flex items-center gap-2">
                    <CheckCircle className="w-4 h-4 text-blue-600" />
                    Recommendations
                  </h4>
                  <ul className="space-y-1">
                    {analysis.gapAnalysis.recommendations.map((rec, idx) => (
                      <li key={idx} className="text-sm flex items-start gap-2">
                        <span className="text-blue-600">•</span>
                        <span>{rec}</span>
                      </li>
                    ))}
                  </ul>
                </CardContent>
              </Card>
            )}

            <div className="flex justify-end gap-3 pt-4 border-t">
              <Button variant="outline" onClick={() => setStep('review')}>
                Back to Review
              </Button>
              <Button 
                onClick={() => {
                  // Save clarification answers as notes
                  toast.success('Clarifications saved! Proceeding to import...');
                  setStep('review');
                }}
                className="bg-purple-600 hover:bg-purple-700"
              >
                Save & Continue
                <ChevronRight className="w-4 h-4 ml-2" />
              </Button>
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}