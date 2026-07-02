import type { ToolboxAction, DesignerTemplate } from './types';

const ACTIONS: ToolboxAction[] = [
  {
    id: 'write-section',
    label: 'Write section',
    description: 'Generate content for this section',
    placeholder: 'What should this section cover? e.g. "Customer Portal login flow for enterprise users"',
    buildPrompt: (userInput, surroundingText, template) =>
      `${template.systemPrompt}

DOCUMENT CONTEXT (surrounding text):
${surroundingText || '(none)'}

TASK: Write the content for this section.
USER GUIDANCE: ${userInput}

Write 2–4 paragraphs (or bullet points if more appropriate). Be specific, professional, and ready to paste directly into the document. Do NOT include a heading — just the body content.`,
  },
  {
    id: 'improve',
    label: 'Improve writing',
    description: 'Improve and expand selected text',
    placeholder: 'What aspect to improve? e.g. "make it more precise" or "add more detail about compliance"',
    buildPrompt: (userInput, surroundingText, template) =>
      `${template.systemPrompt}

EXISTING TEXT TO IMPROVE:
${surroundingText || '(none)'}

TASK: Improve this text. ${userInput ? `Focus on: ${userInput}` : 'Make it clearer, more precise, and professional.'}

Return only the improved text, ready to replace the original. No preamble.`,
  },
  {
    id: 'add-requirements',
    label: 'Generate requirements',
    description: 'Add formal IEEE-830 requirements',
    placeholder: 'Describe the feature area, e.g. "user authentication with SSO and MFA"',
    buildPrompt: (userInput, surroundingText, template) => {
      const countMatch = userInput.match(/\b(\d+)\b/);
      const count = countMatch ? `exactly ${countMatch[1]}` : '4–6';
      return `${template.systemPrompt}

CONTEXT:
${surroundingText || '(none)'}

TASK: Generate formal requirements using IEEE-830 convention (SHALL / SHOULD / MAY).
FEATURE AREA: ${userInput}

Format each as: REQ-NNN: The system SHALL/SHOULD/MAY [action] [condition].
Generate ${count} requirements covering functional behaviour, error conditions, and constraints. Do not generate more than the specified count.`;
    },
  },
  {
    id: 'add-risks',
    label: 'Identify risks',
    description: 'Generate a risk register for this area',
    placeholder: 'Describe the area to assess, e.g. "third-party payment gateway integration"',
    buildPrompt: (userInput, surroundingText, template) =>
      `${template.systemPrompt}

CONTEXT:
${surroundingText || '(none)'}

TASK: Identify risks and mitigation strategies.
AREA: ${userInput}

Format as a list:
Risk [N]: [Title] — Likelihood: High/Medium/Low · Impact: High/Medium/Low
Description: [what could go wrong]
Mitigation: [how to address it]

Generate 4–6 risks.`,
  },
  {
    id: 'add-stakeholders',
    label: 'Stakeholder analysis',
    description: 'Generate a stakeholder table',
    placeholder: 'Describe the project context, e.g. "enterprise SaaS platform migration for a retail chain"',
    buildPrompt: (userInput, surroundingText, template) =>
      `${template.systemPrompt}

CONTEXT:
${surroundingText || '(none)'}

TASK: Generate a stakeholder analysis.
PROJECT CONTEXT: ${userInput}

For each stakeholder provide:
- Role / Title
- Interest in this project
- Engagement level (Inform / Consult / Collaborate / Responsible)
- Key concerns

Generate 5–7 stakeholders. Format as a structured list.`,
  },
  {
    id: 'generate-diagram',
    label: 'Generate diagram',
    description: 'Generate or update Mermaid diagram source',
    placeholder: 'Describe the diagram, e.g. "OAuth2 authorisation code flow with PKCE" or "ER diagram for orders and users"',
    buildPrompt: (userInput, surroundingText, _template) =>
      `You are a technical diagram expert. Generate valid Mermaid diagram code.

CURRENT DIAGRAM SOURCE (if any):
${surroundingText || '(none)'}

TASK: Generate Mermaid source code for: ${userInput}

Rules:
- Output ONLY the raw Mermaid source code — no markdown fences, no explanation, no preamble
- Start directly with the diagram type keyword (sequenceDiagram, flowchart, erDiagram, graph, etc.)
- Use clear, concise labels on all nodes and edges
- Ensure the diagram is syntactically valid Mermaid
- Keep it readable — 8–16 nodes maximum`,
  },
  {
    id: 'summarize',
    label: 'Summarize',
    description: 'Condense into a concise paragraph',
    placeholder: 'Any specific focus for the summary? e.g. "focus on executive audience"',
    buildPrompt: (userInput, surroundingText, template) =>
      `${template.systemPrompt}

TEXT TO SUMMARIZE:
${surroundingText || '(none)'}

TASK: Write a concise summary (1–2 paragraphs). ${userInput ? `Audience / focus: ${userInput}` : ''}

Return only the summary, no preamble.`,
  },
];

export const toolboxRegistry = {
  all(): ToolboxAction[] { return ACTIONS; },
  getById(id: string): ToolboxAction | undefined { return ACTIONS.find(a => a.id === id); },
  forTemplate(actionIds: string[]): ToolboxAction[] {
    return actionIds.map(id => ACTIONS.find(a => a.id === id)).filter(Boolean) as ToolboxAction[];
  },
  register(action: ToolboxAction): void { ACTIONS.push(action); },
};
