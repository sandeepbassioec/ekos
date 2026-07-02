// AI Service - Abstraction layer for multiple AI providers
import { GoogleGenerativeAI } from '@google/generative-ai';

export type AIProvider = 'gemini' | 'claude' | 'openai' | 'groq';

export interface AIConfig {
  provider: AIProvider;
  apiKey?: string;
  model?: string;
}

export interface ExtractedRequirement {
  type: 'new_screen' | 'update_screen' | 'bug_fix' | 'feature_change' | 'api_development' | 'integration' | 'data_migration' | 'performance' | 'security' | 'ui_ux_change';
  title: string;
  description: string;
  priority: 'critical' | 'high' | 'medium' | 'low';
  complexity: 'simple' | 'medium' | 'complex' | 'very_complex';
  estimatedHours?: number;
  
  // User Story
  userStory?: {
    asA: string;
    iWant: string;
    soThat: string;
  };
  
  // Acceptance Criteria
  acceptanceCriteria: string[];
  
  // Screen details (if applicable)
  screenDetails?: {
    screenName: string;
    screenType: 'list' | 'form' | 'detail' | 'dashboard' | 'modal' | 'other';
    fields?: Array<{
      name: string;
      label: string;
      type: string;
      required: boolean;
    }>;
    operations: {
      create: boolean;
      read: boolean;
      update: boolean;
      delete: boolean;
    };
  };
  
  // Technical details
  affectedScreens: string[];
  affectedAPIs: string[];
  technicalNotes?: string;
}

export interface GapAnalysis {
  missingInfo: string[];
  ambiguousRequirements: string[];
  technicalConcerns: string[];
  recommendations: string[];
  clarificationQuestions: string[];
}

export interface BRDAnalysisResult {
  requirements: ExtractedRequirement[];
  gapAnalysis: GapAnalysis;
  summary: {
    totalRequirements: number;
    byType: Record<string, number>;
    totalEstimatedHours: number;
    riskLevel: 'low' | 'medium' | 'high';
  };
}

// Gemini API Integration using Official SDK
async function analyzeWithGemini(text: string, apiKey?: string): Promise<BRDAnalysisResult> {
  const API_KEY = apiKey || 'AIzaSyDVK8xJ_8qB9X7_DEMO_KEY_REPLACE_ME';
  
  // Validate API key
  if (!apiKey && API_KEY.includes('DEMO')) {
    throw new Error('Please provide a valid Gemini API key. Get one free from: https://aistudio.google.com/app/apikey');
  }
  
  try {
    // Initialize Gemini AI with official SDK
    const genAI = new GoogleGenerativeAI(API_KEY);
    
    // Use gemini-1.5-flash model (free tier)
    const model = genAI.getGenerativeModel({ model: 'gemini-1.5-flash' });
    
    const prompt = `You are an expert business analyst. Analyze the following BRD (Business Requirements Document) and extract structured requirements.

BRD Content:
${text.substring(0, 15000)} 

Instructions:
1. Extract all requirements mentioned in the document
2. Categorize each requirement as: new_screen, update_screen, bug_fix, feature_change, api_development, integration, data_migration, performance, security, or ui_ux_change
3. For each requirement, extract:
   - Title (concise, 5-10 words)
   - Description (detailed explanation)
   - Priority (critical, high, medium, low)
   - Complexity (simple, medium, complex, very_complex)
   - Estimated hours (if mentioned, otherwise estimate based on complexity)
   - User story (As a [role], I want [feature], so that [benefit])
   - Acceptance criteria (list of testable conditions)
   - Screen details (if it's a screen requirement)
   - Affected screens/APIs
   - Technical notes

4. Perform gap analysis:
   - Missing information
   - Ambiguous requirements
   - Technical concerns
   - Recommendations
   - Clarification questions for the product owner

5. Provide a summary with:
   - Total requirements
   - Breakdown by type
   - Total estimated hours
   - Overall risk level

Return ONLY valid JSON in this exact format (no markdown, no code blocks):
{
  "requirements": [
    {
      "type": "new_screen",
      "title": "Example Title",
      "description": "Detailed description",
      "priority": "high",
      "complexity": "medium",
      "estimatedHours": 24,
      "userStory": {
        "asA": "User",
        "iWant": "feature",
        "soThat": "benefit"
      },
      "acceptanceCriteria": ["criterion 1", "criterion 2"],
      "screenDetails": {
        "screenName": "Screen Name",
        "screenType": "form",
        "fields": [{"name": "field", "label": "Field", "type": "text", "required": true}],
        "operations": {"create": true, "read": false, "update": false, "delete": false}
      },
      "affectedScreens": ["Screen1"],
      "affectedAPIs": ["/api/endpoint"]
    }
  ],
  "gapAnalysis": {
    "missingInfo": ["item 1"],
    "ambiguousRequirements": ["item 1"],
    "technicalConcerns": ["concern 1"],
    "recommendations": ["rec 1"],
    "clarificationQuestions": ["question 1"]
  },
  "summary": {
    "totalRequirements": 1,
    "byType": {"new_screen": 1},
    "totalEstimatedHours": 24,
    "riskLevel": "medium"
  }
}`;

    // Generate content using official SDK
    const result = await model.generateContent(prompt);
    const response = await result.response;
    const generatedText = response.text();
    
    if (!generatedText) {
      throw new Error('No response from Gemini. Please try again.');
    }

    // Clean up the response (remove markdown code blocks if present)
    let cleanedText = generatedText.trim();
    if (cleanedText.startsWith('```json')) {
      cleanedText = cleanedText.substring(7);
    }
    if (cleanedText.startsWith('```')) {
      cleanedText = cleanedText.substring(3);
    }
    if (cleanedText.endsWith('```')) {
      cleanedText = cleanedText.substring(0, cleanedText.length - 3);
    }
    
    const parsedResult = JSON.parse(cleanedText.trim());
    return parsedResult;
  } catch (error) {
    console.error('Gemini API Error:', error);
    if (error instanceof Error) {
      throw error;
    }
    throw new Error('Failed to analyze document with AI. Please check your API key and try again.');
  }
}

// Groq API Integration (FREE & FAST!)
async function analyzeWithGroq(text: string, apiKey: string): Promise<BRDAnalysisResult> {
  const API_URL = 'https://api.groq.com/openai/v1/chat/completions';
  
  const systemPrompt = `You are an expert business analyst. Analyze BRD documents and extract structured requirements in JSON format.`;
  
  const userPrompt = `Analyze the following BRD (Business Requirements Document) and extract structured requirements.

BRD Content:
${text.substring(0, 20000)}

Instructions:
1. Extract all requirements mentioned in the document
2. Categorize each requirement as: new_screen, update_screen, bug_fix, feature_change, api_development, integration, data_migration, performance, security, or ui_ux_change
3. For each requirement, extract:
   - Title (concise, 5-10 words)
   - Description (detailed explanation)
   - Priority (critical, high, medium, low)
   - Complexity (simple, medium, complex, very_complex)
   - Estimated hours (if mentioned, otherwise estimate based on complexity)
   - User story (As a [role], I want [feature], so that [benefit])
   - Acceptance criteria (list of testable conditions)
   - Screen details (if it's a screen requirement)
   - Affected screens/APIs
   - Technical notes

4. Perform gap analysis:
   - Missing information
   - Ambiguous requirements
   - Technical concerns
   - Recommendations
   - Clarification questions for the product owner

5. Provide a summary with:
   - Total requirements
   - Breakdown by type
   - Total estimated hours
   - Overall risk level

Return ONLY valid JSON in this exact format (no markdown, no code blocks):
{
  "requirements": [
    {
      "type": "new_screen",
      "title": "Example Title",
      "description": "Detailed description",
      "priority": "high",
      "complexity": "medium",
      "estimatedHours": 24,
      "userStory": {
        "asA": "User",
        "iWant": "feature",
        "soThat": "benefit"
      },
      "acceptanceCriteria": ["criterion 1", "criterion 2"],
      "screenDetails": {
        "screenName": "Screen Name",
        "screenType": "form",
        "fields": [{"name": "field", "label": "Field", "type": "text", "required": true}],
        "operations": {"create": true, "read": false, "update": false, "delete": false}
      },
      "affectedScreens": ["Screen1"],
      "affectedAPIs": ["/api/endpoint"]
    }
  ],
  "gapAnalysis": {
    "missingInfo": ["item 1"],
    "ambiguousRequirements": ["item 1"],
    "technicalConcerns": ["concern 1"],
    "recommendations": ["rec 1"],
    "clarificationQuestions": ["question 1"]
  },
  "summary": {
    "totalRequirements": 1,
    "byType": {"new_screen": 1},
    "totalEstimatedHours": 24,
    "riskLevel": "medium"
  }
}`;

  try {
    const response = await fetch(API_URL, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${apiKey}`,
      },
      body: JSON.stringify({
        model: 'llama-3.3-70b-versatile', // Updated to current Groq model
        messages: [
          { role: 'system', content: systemPrompt },
          { role: 'user', content: userPrompt }
        ],
        temperature: 0.2,
        max_tokens: 8000,
        top_p: 1,
        stream: false,
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(`Groq API error: ${response.status} - ${errorData.error?.message || response.statusText}`);
    }

    const data = await response.json();
    const generatedText = data.choices?.[0]?.message?.content;
    
    if (!generatedText) {
      throw new Error('No response from Groq API');
    }

    // Clean up the response (remove markdown code blocks if present)
    let cleanedText = generatedText.trim();
    if (cleanedText.startsWith('```json')) {
      cleanedText = cleanedText.substring(7);
    }
    if (cleanedText.startsWith('```')) {
      cleanedText = cleanedText.substring(3);
    }
    if (cleanedText.endsWith('```')) {
      cleanedText = cleanedText.substring(0, cleanedText.length - 3);
    }
    
    const result = JSON.parse(cleanedText.trim());
    return result;
  } catch (error) {
    console.error('Groq API Error:', error);
    if (error instanceof Error) {
      throw error;
    }
    throw new Error('Failed to analyze document with Groq AI. Please check your API key and try again.');
  }
}

// Claude API Integration (for production)
async function analyzeWithClaude(text: string, apiKey: string): Promise<BRDAnalysisResult> {
  const API_URL = 'https://api.anthropic.com/v1/messages';
  
  const prompt = `You are an expert business analyst. Analyze the following BRD and extract structured requirements...
  
BRD Content:
${text}

[Same detailed prompt as Gemini]

Return ONLY valid JSON.`;

  try {
    const response = await fetch(API_URL, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'x-api-key': apiKey,
        'anthropic-version': '2023-06-01',
      },
      body: JSON.stringify({
        model: 'claude-3-5-sonnet-20241022',
        max_tokens: 8192,
        temperature: 0.2,
        messages: [{
          role: 'user',
          content: prompt,
        }],
      }),
    });

    if (!response.ok) {
      throw new Error(`Claude API error: ${response.statusText}`);
    }

    const data = await response.json();
    const generatedText = data.content?.[0]?.text;
    
    if (!generatedText) {
      throw new Error('No response from Claude');
    }

    // Clean up
    let cleanedText = generatedText.trim();
    if (cleanedText.startsWith('```json')) {
      cleanedText = cleanedText.substring(7);
    }
    if (cleanedText.startsWith('```')) {
      cleanedText = cleanedText.substring(3);
    }
    if (cleanedText.endsWith('```')) {
      cleanedText = cleanedText.substring(0, cleanedText.length - 3);
    }
    
    const result = JSON.parse(cleanedText.trim());
    return result;
  } catch (error) {
    console.error('Claude API Error:', error);
    throw error;
  }
}

// Main function - routes to appropriate provider
export async function analyzeBRD(
  text: string, 
  config: AIConfig = { provider: 'gemini' }
): Promise<BRDAnalysisResult> {
  if (config.provider === 'claude' && config.apiKey) {
    return analyzeWithClaude(text, config.apiKey);
  } else if (config.provider === 'gemini') {
    return analyzeWithGemini(text, config.apiKey);
  } else if (config.provider === 'groq') {
    return analyzeWithGroq(text, config.apiKey);
  } else {
    throw new Error('Invalid AI provider configuration');
  }
}

// Mock function for testing (no API calls)
export function mockAnalyzeBRD(text: string): BRDAnalysisResult {
  return {
    requirements: [
      {
        type: 'new_screen',
        title: 'User Registration Screen',
        description: 'Create a new user registration screen with email, password, and phone number fields',
        priority: 'high',
        complexity: 'medium',
        estimatedHours: 24,
        userStory: {
          asA: 'New User',
          iWant: 'to register my account',
          soThat: 'I can access the platform',
        },
        acceptanceCriteria: [
          'Email validation should work',
          'Password must be 8+ characters',
          'Phone number OTP verification',
          'Form auto-saves on input',
        ],
        screenDetails: {
          screenName: 'User Registration',
          screenType: 'form',
          fields: [
            { name: 'email', label: 'Email Address', type: 'email', required: true },
            { name: 'password', label: 'Password', type: 'password', required: true },
            { name: 'phone', label: 'Phone Number', type: 'phone', required: true },
            { name: 'fullName', label: 'Full Name', type: 'text', required: true },
          ],
          operations: { create: true, read: false, update: false, delete: false },
        },
        affectedScreens: ['Registration'],
        affectedAPIs: ['/api/auth/register', '/api/auth/verify-otp'],
      },
      {
        type: 'bug_fix',
        title: 'Fix login button crash on iOS',
        description: 'Login button causes app crash on iOS devices when clicked multiple times rapidly',
        priority: 'critical',
        complexity: 'simple',
        estimatedHours: 4,
        acceptanceCriteria: [
          'No crash on rapid clicks',
          'Loading state shows correctly',
          'Error handling works',
        ],
        affectedScreens: ['Login'],
        affectedAPIs: ['/api/auth/login'],
        technicalNotes: 'Likely a race condition in async handling',
      },
      {
        type: 'feature_change',
        title: 'Add auto-save to forms',
        description: 'Implement auto-save functionality for all forms to prevent data loss',
        priority: 'medium',
        complexity: 'medium',
        estimatedHours: 16,
        acceptanceCriteria: [
          'Auto-save every 30 seconds',
          'Show "Saved" indicator',
          'Restore data on page reload',
        ],
        affectedScreens: ['All forms'],
        affectedAPIs: ['/api/forms/autosave'],
      },
    ],
    gapAnalysis: {
      missingInfo: [
        'No authentication method specified (OAuth, JWT, Session?)',
        'Database schema not defined',
        'API rate limiting requirements not mentioned',
      ],
      ambiguousRequirements: [
        'User registration mentions "phone number" but validation rules unclear',
        '"All forms" is vague - need specific list of affected forms',
      ],
      technicalConcerns: [
        'No error handling strategy defined',
        'Security requirements (HTTPS, encryption) not specified',
        'Performance targets not mentioned (page load time, API response time)',
      ],
      recommendations: [
        'Add API documentation with request/response schemas',
        'Define security requirements (OWASP compliance)',
        'Specify browser/device compatibility matrix',
        'Add accessibility requirements (WCAG 2.1)',
      ],
      clarificationQuestions: [
        'What authentication method should be used?',
        'Should we support social login (Google, Facebook)?',
        'What are the password complexity requirements?',
        'How long should OTP be valid?',
        'What happens if user email already exists?',
      ],
    },
    summary: {
      totalRequirements: 3,
      byType: {
        new_screen: 1,
        bug_fix: 1,
        feature_change: 1,
      },
      totalEstimatedHours: 44,
      riskLevel: 'medium',
    },
  };
}