/**
 * AWS Bedrock AI Service for Requirements Analysis
 */

import type {
  AIAnalysisRequest,
  AIAnalysisResult,
  CoreRequirement,
  Gap,
  Confusion,
  Suggestion,
  SecurityRequirement,
  Threat,
  NonFunctionalRequirement,
} from '../../types/ai-requirements';

import {
  REQUIREMENTS_ANALYST_PROMPT,
  TEAMS_MEETING_TRANSCRIPT_ANALYZER_PROMPT,
  OUTLOOK_ANALYZER_PROMPT,
  TEAMS_CHAT_ANALYZER_PROMPT,
  DIFFERENTIAL_ANALYSIS_PROMPT,
  MERGE_VALIDATION_PROMPT,
} from '../../config/aiPrompts';

// AI calls go through the server-side proxy (see ../../../../ai-proxy) so the API
// key stays OFF the client and we avoid the browser→AWS CORS problem. Configure
// VITE_AI_PROXY_URL in the frontend .env; defaults to the local proxy.
const AI_PROXY_URL = import.meta.env.VITE_AI_PROXY_URL || 'http://localhost:8787';

interface BedrockCallResult {
  text: string;
  inputTokens: number;
  outputTokens: number;
}

export type AnalysisProgressEvent =
  | { type: 'step-start'; stepId: string }
  | { type: 'step-done'; stepId: string; inputTokens: number; outputTokens: number }
  | { type: 'step-error'; stepId: string; error: string };

/**
 * Call AI proxy (Bedrock / Groq fallback). Returns text + token usage.
 */
async function callBedrock(prompt: string): Promise<BedrockCallResult> {
  try {
    console.log('AI proxy (bedrock) call with prompt:', prompt.substring(0, 100) + '...');

    const response = await fetch(`${AI_PROXY_URL}/api/ai/bedrock`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ prompt }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error('AI proxy (bedrock) error response:', errorText);
      throw new Error(`AI proxy error: ${response.status} - ${errorText}`);
    }

    const data = await response.json();
    console.log('AI proxy (bedrock) response received');

    return {
      text: data.text ?? '',
      inputTokens: data.inputTokens ?? 0,
      outputTokens: data.outputTokens ?? 0,
    };
  } catch (error) {
    console.error('AI proxy (bedrock) error:', error);
    throw error;
  }
}

/** Convenience wrapper for callers that only need the text. */
async function callBedrockText(prompt: string): Promise<string> {
  return (await callBedrock(prompt)).text;
}

/** Strip non-printable control characters that break JSON.parse */
function sanitizeJson(raw: string): string {
  // eslint-disable-next-line no-control-regex
  return raw.replace(/[\x00-\x08\x0B\x0C\x0E-\x1F]/g, '');
}

/**
 * Depth-aware truncation repair.
 * Tracks string boundaries so curly braces inside string values are not counted.
 * Cuts at the last position between top-level properties of the root object,
 * then closes all remaining open structures.
 */
function repairTruncatedJson(raw: string): string {
  const trimmed = raw.trimEnd();
  const opens: string[] = [];
  let inString = false;
  let escaped = false;
  let lastDepth1End = -1;

  for (let i = 0; i < trimmed.length; i++) {
    const ch = trimmed[i];
    if (escaped) { escaped = false; continue; }
    if (ch === '\\' && inString) { escaped = true; continue; }
    if (ch === '"') { inString = !inString; continue; }
    if (inString) continue;
    if (ch === '{' || ch === '[') {
      opens.push(ch === '{' ? '}' : ']');
    } else if (ch === '}' || ch === ']') {
      opens.pop();
      if (opens.length === 1) lastDepth1End = i;
    }
  }

  if (opens.length === 0) return trimmed;

  const cutAt = lastDepth1End > 0 ? lastDepth1End + 1 : trimmed.length;
  const cut = trimmed.slice(0, cutAt);

  const closers: string[] = [];
  inString = false; escaped = false;
  for (let i = 0; i < cut.length; i++) {
    const ch = cut[i];
    if (escaped) { escaped = false; continue; }
    if (ch === '\\' && inString) { escaped = true; continue; }
    if (ch === '"') { inString = !inString; continue; }
    if (inString) continue;
    if (ch === '{' || ch === '[') closers.push(ch === '{' ? '}' : ']');
    else if (ch === '}' || ch === ']') closers.pop();
  }
  return cut + closers.reverse().join('');
}

interface ParsedBedrockResult {
  result: AIAnalysisResult;
  inputTokens: number;
  outputTokens: number;
}

/**
 * Call Bedrock and parse the JSON result, with sanitize + repair fallback.
 * Returns parsed result plus token usage for progress tracking.
 */
async function callBedrockAndParse(prompt: string): Promise<ParsedBedrockResult> {
  const { text, inputTokens, outputTokens } = await callBedrock(prompt);
  const jsonMatch = text.match(/\{[\s\S]*\}/);
  if (!jsonMatch) throw new Error('Invalid JSON response from AI');

  const raw = jsonMatch[0];

  // Attempt 1: direct parse
  try {
    return { result: JSON.parse(raw) as AIAnalysisResult, inputTokens, outputTokens };
  } catch (_) { /* fall through */ }

  // Attempt 2: sanitize control chars, then parse
  try {
    return { result: JSON.parse(sanitizeJson(raw)) as AIAnalysisResult, inputTokens, outputTokens };
  } catch (_) { /* fall through */ }

  // Attempt 3: repair truncated response (max_tokens cutoff), then parse
  try {
    return { result: JSON.parse(sanitizeJson(repairTruncatedJson(raw))) as AIAnalysisResult, inputTokens, outputTokens };
  } catch (err) {
    console.error('JSON parse failed after all repair attempts. Raw length:', raw.length);
    throw new Error(`AI returned unparseable JSON: ${(err as Error).message}`);
  }
}

/**
 * Analyze all sources and extract comprehensive requirements.
 *
 * Strategy:
 *   1. If documents exist → analyze docs first with RequirementsAnalystPrompt → build baseline
 *      Then refine baseline by passing it to each meeting / email / chat specialist prompt.
 *   2. If NO documents (noDocumentMode = true) → analyze all sources in parallel with empty
 *      baseline (user has already confirmed this in the UI).
 */
export async function analyzeRequirements(
  request: AIAnalysisRequest,
  sourceTexts: {
    documents: { id: string; name: string; content: string }[];
    meetings: { id: string; title: string; notes: string }[];
    emails: { id: string; subject: string; body: string }[];
    chats: { id: string; content: string }[];
  },
  noDocumentMode = false,
  onProgress?: (event: AnalysisProgressEvent) => void,
): Promise<AIAnalysisResult> {

  const hasDocuments = sourceTexts.documents.length > 0;
  const hasSecondarySources =
    sourceTexts.meetings.length > 0 ||
    sourceTexts.emails.length > 0 ||
    sourceTexts.chats.length > 0;

  if (!hasDocuments && !hasSecondarySources) {
    throw new Error('No source content to analyze');
  }

  try {
    let baseline: AIAnalysisResult = emptyResult();

    // ── PHASE 1: Build baseline from documents ──────────────────────────────
    if (hasDocuments) {
      const docResults = await Promise.all(
        sourceTexts.documents.map(async (doc) => {
          const stepId = `doc-${doc.id}`;
          onProgress?.({ type: 'step-start', stepId });
          const prompt = REQUIREMENTS_ANALYST_PROMPT
            .replace(/\{\{DOCUMENT_CONTENT\}\}/g, doc.content)
            .replace(/\{\{SOURCE_ID\}\}/g, doc.id);
          try {
            const { result, inputTokens, outputTokens } = await callBedrockAndParse(prompt);
            onProgress?.({ type: 'step-done', stepId, inputTokens, outputTokens });
            return result;
          } catch (err) {
            onProgress?.({ type: 'step-error', stepId, error: (err as Error).message });
            throw err;
          }
        })
      );
      baseline = mergeResults(docResults);
    }

    // ── PHASE 2: Refine baseline with secondary sources ─────────────────────
    // If noDocumentMode, secondary sources run with empty baseline (already confirmed by user)
    if (hasSecondarySources) {
      const baselineJson = JSON.stringify(baseline);
      const refinementPromises: Promise<AIAnalysisResult>[] = [];

      for (const meeting of sourceTexts.meetings) {
        const stepId = `meeting-${meeting.id}`;
        onProgress?.({ type: 'step-start', stepId });
        const prompt = TEAMS_MEETING_TRANSCRIPT_ANALYZER_PROMPT
          .replace(/\{\{ANALYZED_REQUIREMENTS_JSON\}\}/g, baselineJson)
          .replace(/\{\{MEETING_TITLE\}\}/g, meeting.title)
          .replace(/\{\{MEETING_DATE\}\}/g, new Date().toISOString().split('T')[0])
          .replace(/\{\{ATTENDEES_LIST\}\}/g, '')
          .replace(/\{\{TRANSCRIPT_CONTENT\}\}/g, meeting.notes)
          .replace(/\{\{SOURCE_ID\}\}/g, meeting.id);
        refinementPromises.push(
          callBedrockAndParse(prompt).then(({ result, inputTokens, outputTokens }) => {
            onProgress?.({ type: 'step-done', stepId, inputTokens, outputTokens });
            return result;
          }).catch((err) => {
            onProgress?.({ type: 'step-error', stepId, error: (err as Error).message });
            throw err;
          })
        );
      }

      for (const email of sourceTexts.emails) {
        const stepId = `email-${email.id}`;
        onProgress?.({ type: 'step-start', stepId });
        const prompt = OUTLOOK_ANALYZER_PROMPT
          .replace(/\{\{ANALYZED_REQUIREMENTS_JSON\}\}/g, baselineJson)
          .replace(/\{\{EMAIL_FROM\}\}/g, email.subject)
          .replace(/\{\{EMAIL_TO\}\}/g, '')
          .replace(/\{\{EMAIL_CC\}\}/g, '')
          .replace(/\{\{EMAIL_DATE\}\}/g, new Date().toISOString().split('T')[0])
          .replace(/\{\{EMAIL_SUBJECT\}\}/g, email.subject)
          .replace(/\{\{EMAIL_THREAD_CONTENT\}\}/g, email.body)
          .replace(/\{\{SOURCE_ID\}\}/g, email.id);
        refinementPromises.push(
          callBedrockAndParse(prompt).then(({ result, inputTokens, outputTokens }) => {
            onProgress?.({ type: 'step-done', stepId, inputTokens, outputTokens });
            return result;
          }).catch((err) => {
            onProgress?.({ type: 'step-error', stepId, error: (err as Error).message });
            throw err;
          })
        );
      }

      for (const chat of sourceTexts.chats) {
        const stepId = `chat-${chat.id}`;
        onProgress?.({ type: 'step-start', stepId });
        const prompt = TEAMS_CHAT_ANALYZER_PROMPT
          .replace(/\{\{ANALYZED_REQUIREMENTS_JSON\}\}/g, baselineJson)
          .replace(/\{\{CHAT_NAME\}\}/g, '')
          .replace(/\{\{CHAT_DATE_RANGE\}\}/g, '')
          .replace(/\{\{PARTICIPANTS_LIST\}\}/g, '')
          .replace(/\{\{CHAT_CONTENT\}\}/g, chat.content)
          .replace(/\{\{SOURCE_ID\}\}/g, chat.id);
        refinementPromises.push(
          callBedrockAndParse(prompt).then(({ result, inputTokens, outputTokens }) => {
            onProgress?.({ type: 'step-done', stepId, inputTokens, outputTokens });
            return result;
          }).catch((err) => {
            onProgress?.({ type: 'step-error', stepId, error: (err as Error).message });
            throw err;
          })
        );
      }

      const refinements = await Promise.all(refinementPromises);
      // Merge baseline + refinements (refinements ADD to baseline, not replace)
      baseline = mergeResults([baseline, ...refinements]);
    }

    const result = baseline;

    // Add metadata
    result.coreRequirements = result.coreRequirements.map(req => ({
      ...req,
      roadmapItemId: request.roadmapItemId,
      extractedBy: 'AI' as const,
      status: 'Draft' as const,
    }));

    result.gaps = result.gaps.map(gap => ({
      ...gap,
      roadmapItemId: request.roadmapItemId,
      identifiedBy: 'AI' as const,
      status: 'Open' as const,
    }));

    result.confusions = result.confusions.map(confusion => ({
      ...confusion,
      roadmapItemId: request.roadmapItemId,
      status: 'Open' as const,
    }));

    result.suggestions = result.suggestions.map(suggestion => ({
      ...suggestion,
      roadmapItemId: request.roadmapItemId,
      generatedBy: 'AI' as const,
      status: 'Pending' as const,
    }));

    result.securityRequirements = result.securityRequirements.map(req => ({
      ...req,
      roadmapItemId: request.roadmapItemId,
      status: 'Draft' as const,
    }));

    result.threats = result.threats.map(threat => ({
      ...threat,
      roadmapItemId: request.roadmapItemId,
      status: 'Identified' as const,
    }));

    result.nfrs = result.nfrs.map(nfr => ({
      ...nfr,
      roadmapItemId: request.roadmapItemId,
      status: 'Draft' as const,
    }));

    return result;
  } catch (error) {
    console.error('Error analyzing requirements:', error);
    throw error;
  }
}

/** Empty result skeleton */
function emptyResult(): AIAnalysisResult {
  return {
    coreRequirements: [],
    gaps: [],
    confusions: [],
    suggestions: [],
    securityRequirements: [],
    threats: [],
    nfrs: [],
    cloudGovernance: [],
  };
}

/** Merge multiple AIAnalysisResult objects into one */
function mergeResults(results: AIAnalysisResult[]): AIAnalysisResult {
  return {
    coreRequirements: results.flatMap(r => r.coreRequirements || []),
    gaps: results.flatMap(r => r.gaps || []),
    confusions: results.flatMap(r => r.confusions || []),
    suggestions: results.flatMap(r => r.suggestions || []),
    securityRequirements: results.flatMap(r => r.securityRequirements || []),
    threats: results.flatMap(r => r.threats || []),
    nfrs: results.flatMap(r => r.nfrs || []),
    cloudGovernance: results.flatMap(r => r.cloudGovernance || []),
  };
}

/**
 * Refine requirements based on gap/confusion resolution
 */
export async function refineRequirements(
  targetType: 'gap' | 'confusion',
  originalContext: string,
  question: string,
  answer: string
): Promise<string[]> {
  const prompt = `You are an expert requirements analyst. A ${targetType} was identified in the requirements analysis:

**Original Context:**
${originalContext}

**${targetType === 'gap' ? 'Gap' : 'Confusion'}:**
${question}

**Resolution/Answer:**
${answer}

Based on this resolution, generate refined or new requirements that should be added. Return as a JSON array of requirement strings.

Format: ["requirement 1", "requirement 2", ...]`;

  try {
    const response = await callBedrockText(prompt);
    const jsonMatch = response.match(/\[[\s\S]*\]/);
    if (!jsonMatch) {
      throw new Error('Invalid JSON response from AI');
    }
    return JSON.parse(jsonMatch[0]);
  } catch (error) {
    console.error('Error refining requirements:', error);
    throw error;
  }
}


/**
 * Search across all KnowledgeHub sources using natural language.
 * Returns a direct answer with citations back to specific sources.
 */
export async function searchKnowledgeHub(
  question: string,
  sources: {
    documents: { id: string; name: string; content: string }[];
    meetings: { id: string; title: string; notes: string }[];
    emails: { id: string; subject: string; body: string }[];
    chats: { id: string; title?: string; content: string }[];
    formalRequirements: { id: string; requirementId: string; title: string; description: string; category: string }[];
    gaps: { id: string; title: string; description: string; status: string }[];
  }
): Promise<{ answer: string; sources: { type: string; id: string; title: string; excerpt: string }[]; confidence: number }> {

  // Build a concise context — truncate long content to save tokens
  const MAX_CHARS = 1500;
  const truncate = (text: string) => text.length > MAX_CHARS ? text.substring(0, MAX_CHARS) + '...' : text;

  const contextParts: string[] = [];

  if (sources.documents.length > 0) {
    contextParts.push('## DOCUMENTS');
    sources.documents.forEach(d => contextParts.push(`[DOC:${d.id}] ${d.name}\n${truncate(d.content)}`));
  }
  if (sources.meetings.length > 0) {
    contextParts.push('## MEETINGS');
    sources.meetings.forEach(m => contextParts.push(`[MTG:${m.id}] ${m.title}\n${truncate(m.notes)}`));
  }
  if (sources.emails.length > 0) {
    contextParts.push('## EMAILS');
    sources.emails.forEach(e => contextParts.push(`[EMAIL:${e.id}] ${e.subject}\n${truncate(e.body)}`));
  }
  if (sources.chats.length > 0) {
    contextParts.push('## CHATS');
    sources.chats.forEach(c => contextParts.push(`[CHAT:${c.id}] ${c.title ?? 'Chat'}\n${truncate(c.content)}`));
  }
  if (sources.formalRequirements.length > 0) {
    contextParts.push('## FORMAL REQUIREMENTS');
    sources.formalRequirements.slice(0, 30).forEach(r =>
      contextParts.push(`[REQ:${r.id}] ${r.requirementId} — ${r.title} (${r.category})\n${r.description}`)
    );
  }
  if (sources.gaps.length > 0) {
    contextParts.push('## IDENTIFIED GAPS');
    sources.gaps.filter(g => g.status === 'Open').slice(0, 15).forEach(g =>
      contextParts.push(`[GAP:${g.id}] ${g.title}\n${g.description}`)
    );
  }

  if (contextParts.length === 0) {
    return {
      answer: 'No source data available. Please upload documents, meetings, emails or chats first.',
      sources: [],
      confidence: 0,
    };
  }

  const prompt = `You are a knowledgeable assistant analyzing a project's KnowledgeHub. Answer the user's question based ONLY on the provided source materials below. Be specific and cite your sources using their IDs (e.g. [DOC:xxx], [MTG:xxx], [REQ:xxx]).

## SOURCE MATERIALS
${contextParts.join('\n\n')}

## USER QUESTION
${question}

## INSTRUCTIONS
1. Answer the question directly and concisely based on the sources above.
2. If the answer is not found in the sources, say so clearly.
3. Cite which specific sources support each part of your answer.
4. List the source IDs you used.
5. Rate your confidence (0-100) based on how directly the sources answer the question.

Return ONLY valid JSON:
{
  "answer": "Your detailed answer here with inline citations like [DOC:id] or [REQ:id]",
  "usedSourceIds": ["DOC:id1", "MTG:id2"],
  "confidence": 85
}`;

  const raw = await callBedrockText(prompt);
  const match = raw.match(/\{[\s\S]*\}/);
  if (!match) throw new Error('Invalid response from AI');

  const result = JSON.parse(match[0]);

  // Map source IDs back to readable source objects
  const usedSources: { type: string; id: string; title: string; excerpt: string }[] = [];
  (result.usedSourceIds ?? []).forEach((ref: string) => {
    const [typePrefix, id] = ref.split(':');
    if (typePrefix === 'DOC') {
      const doc = sources.documents.find(d => d.id === id);
      if (doc) usedSources.push({ type: 'document', id, title: doc.name, excerpt: doc.content.substring(0, 150) + '...' });
    } else if (typePrefix === 'MTG') {
      const mtg = sources.meetings.find(m => m.id === id);
      if (mtg) usedSources.push({ type: 'meeting', id, title: mtg.title, excerpt: mtg.notes.substring(0, 150) + '...' });
    } else if (typePrefix === 'EMAIL') {
      const email = sources.emails.find(e => e.id === id);
      if (email) usedSources.push({ type: 'email', id, title: email.subject, excerpt: email.body.substring(0, 150) + '...' });
    } else if (typePrefix === 'CHAT') {
      const chat = sources.chats.find(c => c.id === id);
      if (chat) usedSources.push({ type: 'chat', id, title: chat.title ?? 'Chat', excerpt: chat.content.substring(0, 150) + '...' });
    } else if (typePrefix === 'REQ') {
      const req = sources.formalRequirements.find(r => r.id === id);
      if (req) usedSources.push({ type: 'requirement', id, title: `${req.requirementId} — ${req.title}`, excerpt: req.description.substring(0, 150) });
    } else if (typePrefix === 'GAP') {
      const gap = sources.gaps.find(g => g.id === id);
      if (gap) usedSources.push({ type: 'gap', id, title: gap.title, excerpt: gap.description.substring(0, 150) });
    }
  });

  return {
    answer: result.answer ?? 'No answer generated.',
    sources: usedSources,
    confidence: result.confidence ?? 0,
  };
}

/**
 * /summary — Generate a concise executive summary of everything in the hub
 */
export async function summarizeHub(hub: {
  documents: any[]; meetings: any[]; emails: any[]; chats: any[];
  formalRequirements: any[]; gaps: any[]; confusions: any[];
  suggestions: any[]; securityRequirements: any[]; threats: any[]; nfrs: any[];
}): Promise<string> {
  const counts = {
    docs:     hub.documents.length,
    meetings: hub.meetings.length,
    emails:   hub.emails.length,
    chats:    hub.chats.length,
    reqs:     hub.formalRequirements.length,
    approvedReqs: hub.formalRequirements.filter((r: any) => r.status === 'Approved').length,
    openGaps: hub.gaps.filter((g: any) => g.status === 'Open').length,
    openConf: hub.confusions.filter((c: any) => c.status === 'Open').length,
    criticalThreats: hub.threats.filter((t: any) => t.level === 'Critical' || t.level === 'High').length,
    nfrs:     hub.nfrs.length,
    secReqs:  hub.securityRequirements.length,
  };

  const reqSample = hub.formalRequirements.slice(0, 10)
    .map((r: any) => `- ${r.requirementId}: ${r.title} [${r.category}, ${r.priority}, ${r.status}]`).join('\n');
  const gapSample = hub.gaps.filter((g: any) => g.status === 'Open').slice(0, 5)
    .map((g: any) => `- ${g.title} [${g.severity}]`).join('\n');
  const threatSample = hub.threats.slice(0, 5)
    .map((t: any) => `- ${t.threat} [${t.category}, ${t.level}]`).join('\n');

  const prompt = `You are a senior business analyst. Generate a concise executive summary (max 250 words) of this project's KnowledgeHub.

DATA SNAPSHOT:
- Sources: ${counts.docs} docs, ${counts.meetings} meetings, ${counts.emails} emails, ${counts.chats} chats
- Requirements: ${counts.reqs} total, ${counts.approvedReqs} approved
- Open gaps: ${counts.openGaps}, Open confusions: ${counts.openConf}
- Threats: ${counts.criticalThreats} critical/high, Security reqs: ${counts.secReqs}
- NFRs: ${counts.nfrs}

SAMPLE REQUIREMENTS:
${reqSample || 'None yet'}

OPEN GAPS:
${gapSample || 'None'}

THREATS:
${threatSample || 'None identified'}

Write a 3-4 paragraph executive summary covering: what has been captured, the current state of requirements, key risks/gaps that need attention, and overall readiness. Be direct and professional. Do NOT return JSON — return plain readable text.`;

  return callBedrockText(prompt);
}

/**
 * /openitems — List all unresolved gaps and open confusions
 */
export async function listOpenItems(hub: {
  gaps: any[]; confusions: any[];
}): Promise<string> {
  const openGaps = hub.gaps.filter((g: any) => g.status === 'Open');
  const openConf = hub.confusions.filter((c: any) => c.status === 'Open');

  if (openGaps.length === 0 && openConf.length === 0) {
    return '✅ No open items — all gaps and confusions have been resolved.';
  }

  const prompt = `You are a business analyst. Summarise these open items clearly and concisely for a project stakeholder.

OPEN GAPS (${openGaps.length}):
${openGaps.map((g: any) => `- [${g.severity}] ${g.title}: ${g.description}`).join('\n') || 'None'}

OPEN CONFUSIONS (${openConf.length}):
${openConf.map((c: any) => `- ${c.question}: ${c.context}`).join('\n') || 'None'}

Produce a clear numbered list grouped by severity/priority. For each item note what action is needed. End with a 1-sentence overall assessment. Plain text only, no JSON.`;

  return callBedrockText(prompt);
}

/**
 * Merge 2+ AI requirements into one comprehensive requirement via AI.
 */
export async function mergeAIRequirements(
  requirements: Array<{
    requirement: string;
    description?: string;
    rationale?: string;
    requirementKey?: string;
  }>
): Promise<{
  requirementKey: string;
  requirement: string;
  description: string;
  technicalCategory: string;
  changeType: string;
  rationale: string;
  type: string;
  priority: string;
  confidence: number;
}> {
  const list = requirements
    .map((r, i) => `${i + 1}. ${r.requirement}${r.description ? `\n   Details: ${r.description}` : ''}`)
    .join('\n');

  const prompt = `You are a senior requirements analyst. Merge the following requirements into ONE single comprehensive, unambiguous requirement. Preserve ALL intent from every item — do not lose any functionality.

Requirements to merge:
${list}

Return ONLY valid JSON (no markdown):
{
  "requirementKey": "merged-kebab-slug-max-5-words",
  "requirement": "IEEE-830: [ACTOR] SHALL [ACTION] [OBJECT] [CONDITION]",
  "description": "Component affected + what changes + business outcome (max 150 chars)",
  "technicalCategory": "UI|API|Database|Integration|Business Logic|Security|Infrastructure",
  "changeType": "New Screen|Screen Change|New API|API Change|Database Change|New Integration|Business Rule|Configuration|Other",
  "rationale": "Why this merged requirement exists (max 100 chars)",
  "type": "Functional|Non-Functional|Security|Performance|Compliance",
  "priority": "Critical|High|Medium|Low",
  "confidence": 90
}`;

  const text = await callBedrockText(prompt);
  const match = text.match(/\{[\s\S]*\}/);
  if (!match) throw new Error('AI merge returned no valid JSON');
  return JSON.parse(match[0]);
}

/**
 * Differential analysis: when a new email/meeting/chat is added, check it against
 * existing open gaps and confusions. Returns what it resolves, updates, or conflicts.
 */
export async function analyzeNewSource(params: {
  sourceType: 'email' | 'meeting' | 'chat';
  sourceName: string;
  content: string;
  openGaps: Array<{ id: string; title: string; description: string }>;
  openConfusions: Array<{ id: string; question: string; context: string }>;
  aiRequirements: Array<{ requirementKey?: string; requirement: string }>;
}): Promise<{
  resolvedGaps: Array<{ id: string; excerpt: string }>;
  updatedGaps: Array<{ id: string; excerpt: string; update: string }>;
  clarifiedConfusions: Array<{ id: string; excerpt: string }>;
  newGaps: Array<{ title: string; description: string; severity: string; suggestions: string[] }>;
  newConfusions: Array<{ question: string; context: string }>;
  newSuggestions: Array<{ title: string; description: string; type: string; impact?: string; effort?: string }>;
  conflicts: Array<{ title: string; description: string; conflictingRequirementKey?: string; excerpt: string; severity: string }>;
}> {
  const label = params.sourceType === 'email' ? 'Email' : params.sourceType === 'meeting' ? 'Meeting Transcript' : 'Teams Chat';
  const prompt = DIFFERENTIAL_ANALYSIS_PROMPT
    .replace(/\{\{SOURCE_TYPE\}\}/g, params.sourceType)
    .replace(/\{\{SOURCE_TYPE_UPPER\}\}/g, label)
    .replace('{{SOURCE_NAME}}', params.sourceName)
    .replace('{{SOURCE_CONTENT}}', params.content.slice(0, 8000))
    .replace('{{OPEN_GAPS_JSON}}', params.openGaps.length ? JSON.stringify(params.openGaps.slice(0, 20)) : '[]')
    .replace('{{OPEN_CONFUSIONS_JSON}}', params.openConfusions.length ? JSON.stringify(params.openConfusions.slice(0, 20)) : '[]')
    .replace('{{REQUIREMENTS_JSON}}', params.aiRequirements.length ? JSON.stringify(params.aiRequirements.slice(0, 20).map(r => ({ key: r.requirementKey, text: r.requirement }))) : '[]');

  const text = await callBedrockText(prompt);
  const match = text.match(/\{[\s\S]*\}/);
  if (!match) return { resolvedGaps: [], updatedGaps: [], clarifiedConfusions: [], newGaps: [], newConfusions: [], newSuggestions: [], conflicts: [] };

  try {
    const parsed = JSON.parse(sanitizeJson(match[0]));
    return {
      resolvedGaps: parsed.resolvedGaps || [],
      updatedGaps: parsed.updatedGaps || [],
      clarifiedConfusions: parsed.clarifiedConfusions || [],
      newGaps: parsed.newGaps || [],
      newConfusions: parsed.newConfusions || [],
      newSuggestions: parsed.newSuggestions || [],
      conflicts: parsed.conflicts || [],
    };
  } catch {
    return { resolvedGaps: [], updatedGaps: [], clarifiedConfusions: [], newGaps: [], newConfusions: [], newSuggestions: [], conflicts: [] };
  }
}

/**
 * Validate whether two Gap / Confusion / Suggestion items are related enough to merge.
 * Returns AI verdict + a merged draft the UI can show before the user confirms.
 */
export async function validateMerge(
  itemType: 'gap' | 'confusion' | 'suggestion',
  itemA: Record<string, any>,
  itemB: Record<string, any>,
): Promise<{
  isRelated: boolean;
  confidence: number;
  reason: string;
  merged: Record<string, any>;
}> {
  const prompt = MERGE_VALIDATION_PROMPT
    .replace(/\{\{ITEM_TYPE\}\}/g, itemType)
    .replace('{{ITEM_A_JSON}}', JSON.stringify(itemA, null, 2))
    .replace('{{ITEM_B_JSON}}', JSON.stringify(itemB, null, 2));

  const text = await callBedrockText(prompt);
  const match = text.match(/\{[\s\S]*\}/);
  if (!match) return { isRelated: false, confidence: 0, reason: 'AI returned no response', merged: {} };

  try {
    const parsed = JSON.parse(sanitizeJson(match[0]));
    return {
      isRelated: parsed.isRelated ?? false,
      confidence: parsed.confidence ?? 0,
      reason: parsed.reason ?? '',
      merged: parsed.merged ?? {},
    };
  } catch {
    return { isRelated: false, confidence: 0, reason: 'Parse error', merged: {} };
  }
}

/**
 * /risks — Generate a risk brief from threats and critical gaps
 */
export async function generateRiskBrief(hub: {
  threats: any[]; gaps: any[]; securityRequirements: any[];
}): Promise<string> {
  const criticalThreats = hub.threats.filter((t: any) => t.level === 'Critical' || t.level === 'High');
  const criticalGaps    = hub.gaps.filter((g: any) => (g.severity === 'Critical' || g.severity === 'High') && g.status === 'Open');

  if (criticalThreats.length === 0 && criticalGaps.length === 0) {
    return '✅ No critical or high severity threats or gaps identified.';
  }

  const prompt = `You are a security and risk analyst. Generate a concise risk brief.

CRITICAL/HIGH THREATS (${criticalThreats.length}):
${criticalThreats.map((t: any) => `- [${t.level}] ${t.threat} (${t.category}): ${t.description}\n  Mitigations: ${(t.mitigations ?? []).join('; ') || 'None defined'}`).join('\n') || 'None'}

CRITICAL/HIGH OPEN GAPS (${criticalGaps.length}):
${criticalGaps.map((g: any) => `- [${g.severity}] ${g.title}: ${g.description}`).join('\n') || 'None'}

SECURITY REQUIREMENTS COUNT: ${hub.securityRequirements.length}

Produce a risk brief with: (1) Top risks ranked by severity, (2) Recommended immediate actions, (3) Overall risk level (Low/Medium/High/Critical). Keep it under 200 words. Plain text only.`;

  return callBedrockText(prompt);
}
