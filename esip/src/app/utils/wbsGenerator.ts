import { Screen, ComplexityLevel } from '../store/estimationStore';

// Groq calls go through the server-side proxy (see ../../../ai-proxy) so the API
// key stays OFF the client. Configure VITE_AI_PROXY_URL; defaults to the local proxy.
const AI_PROXY_URL = import.meta.env.VITE_AI_PROXY_URL || 'http://localhost:8787';

export interface WBSGenerationRequest {
  projectName: string;
  requirements: any[];
  estimationMethod: string;
}

export async function generateWBSFromRequirements(request: WBSGenerationRequest): Promise<Screen[]> {
  const { projectName, requirements, estimationMethod } = request;

  // Build comprehensive prompt from requirements
  const requirementsText = requirements.map((req, idx) => {
    return `${idx + 1}. ${req.title || req.name || 'Unnamed Requirement'}
   Description: ${req.description || 'No description'}
   Type: ${req.type || 'Unknown'}
   Priority: ${req.priority || 'medium'}
   ${req.acceptanceCriteria ? `Acceptance Criteria: ${JSON.stringify(req.acceptanceCriteria)}` : ''}`;
  }).join('\n\n');

  const prompt = `You are an expert software development estimator. Given the following project requirements, generate a comprehensive Work Breakdown Structure (WBS) with screens, features, APIs, and components.

Project Name: ${projectName}
Estimation Method: ${estimationMethod}

Requirements:
${requirementsText}

INSTRUCTIONS:
1. Analyze all requirements and break them down into specific deliverable items
2. Each item should be a Screen, Feature, API, or Component
3. Assign complexity level (LOW, MEDIUM, HIGH) based on:
   - LOW: Simple CRUD, basic forms, simple lists
   - MEDIUM: Complex forms, multiple integrations, business logic
   - HIGH: Complex algorithms, real-time features, advanced integrations
4. Create logical hierarchy where needed (parent-child relationships)
5. Be specific and actionable - each item should be estimable

OUTPUT FORMAT (JSON Array):
[
  {
    "name": "User Registration Screen",
    "description": "Complete user registration with email verification",
    "type": "SCREEN",
    "complexity": "MEDIUM",
    "parentId": null
  },
  {
    "name": "User Authentication API",
    "description": "JWT-based authentication endpoints",
    "type": "API",
    "complexity": "HIGH",
    "parentId": null
  }
]

Generate 8-15 WBS items. Return ONLY the JSON array, no other text.`;

  try {
    const response = await fetch(`${AI_PROXY_URL}/api/ai/groq`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        model: 'llama-3.3-70b-versatile',
        messages: [
          {
            role: 'system',
            content: 'You are an expert software project estimator. Always return valid JSON arrays only.',
          },
          {
            role: 'user',
            content: prompt,
          },
        ],
        temperature: 0.3,
        max_tokens: 4000,
      }),
    });

    if (!response.ok) {
      throw new Error(`AI proxy (groq) error: ${response.statusText}`);
    }

    const data = await response.json();
    // Proxy returns the model's text as { content }
    const content = data.content || '[]';

    // Extract JSON from markdown code blocks if present
    let jsonContent = content.trim();
    if (jsonContent.startsWith('```json')) {
      jsonContent = jsonContent.replace(/```json\n?/g, '').replace(/```\n?/g, '');
    } else if (jsonContent.startsWith('```')) {
      jsonContent = jsonContent.replace(/```\n?/g, '');
    }

    const wbsItems = JSON.parse(jsonContent);

    // Convert to Screen format
    const screens: Screen[] = wbsItems.map((item: any, index: number) => ({
      id: `wbs-${Date.now()}-${index}`,
      name: item.name || `Item ${index + 1}`,
      description: item.description || '',
      type: (item.type || 'FEATURE').toUpperCase() as 'SCREEN' | 'FEATURE' | 'API' | 'COMPONENT',
      complexity: (item.complexity || 'MEDIUM').toUpperCase() as ComplexityLevel,
      parentId: item.parentId || undefined,
      level: 0,
      isExpanded: true,
    }));

    return screens;
  } catch (error) {
    console.error('Error generating WBS:', error);
    throw error;
  }
}
