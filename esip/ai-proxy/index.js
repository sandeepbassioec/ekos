/**
 * ESIP AI Proxy
 * -------------
 * A tiny server-side proxy for the app's AI calls. Its only jobs:
 *   1. Hold the API keys server-side (never shipped to the browser).
 *   2. Avoid browser CORS — the browser calls this proxy; this proxy calls AWS/Groq.
 *
 * Endpoints:
 *   GET  /health           → liveness + which providers are configured
 *   POST /api/ai/bedrock    { prompt }                              → { text }
 *   POST /api/ai/groq       { messages, model?, temperature?, max_tokens? } → { content }
 *
 * Configure via .env (see .env.example). Run: npm install && npm start
 */

import 'dotenv/config';
import express from 'express';
import cors from 'cors';

const PORT = process.env.PORT || 8787;

// AWS Bedrock
const BEDROCK_API_KEY = process.env.BEDROCK_API_KEY || '';
const AWS_REGION = process.env.AWS_REGION || 'us-east-1';
const BEDROCK_MODEL = process.env.BEDROCK_MODEL || 'us.anthropic.claude-sonnet-4-6';

// Groq
const GROQ_API_KEY = process.env.GROQ_API_KEY || '';
const GROQ_MODEL = process.env.GROQ_MODEL || 'llama-3.3-70b-versatile';
const GROQ_API_URL = 'https://api.groq.com/openai/v1/chat/completions';

// Allowed browser origins (comma-separated). Defaults to the Vite dev server.
const ALLOWED_ORIGINS = (process.env.ALLOWED_ORIGIN || 'http://localhost:5173')
  .split(',')
  .map((o) => o.trim())
  .filter(Boolean);

const app = express();
app.use(cors({ origin: ALLOWED_ORIGINS }));
app.use(express.json({ limit: '15mb' })); // document text can be large

app.get('/health', (_req, res) => {
  res.json({
    ok: true,
    providers: {
      bedrock: Boolean(BEDROCK_API_KEY),
      groq: Boolean(GROQ_API_KEY),
    },
  });
});

/**
 * AWS Bedrock — Claude (Anthropic messages API over Bedrock).
 * Falls back to Groq automatically when BEDROCK_API_KEY is not set,
 * so the same client endpoint works with just a free Groq key.
 */
app.post('/api/ai/bedrock', async (req, res) => {
  const { prompt } = req.body || {};
  if (typeof prompt !== 'string' || !prompt.trim()) {
    return res.status(400).json({ error: 'Body must include a non-empty "prompt" string.' });
  }

  // ── Groq fallback: use when no Bedrock key is configured ─────────────────
  if (!BEDROCK_API_KEY) {
    if (!GROQ_API_KEY) {
      return res.status(500).json({
        error: 'No AI provider configured. Set BEDROCK_API_KEY or GROQ_API_KEY in ai-proxy/.env',
      });
    }
    try {
      console.log('[proxy] No BEDROCK_API_KEY — routing /api/ai/bedrock → Groq');
      const upstream = await fetch(GROQ_API_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${GROQ_API_KEY}` },
        body: JSON.stringify({
          model: GROQ_MODEL,
          messages: [{ role: 'user', content: prompt }],
          temperature: 0.3,
          max_tokens: 4000,
        }),
      });
      if (!upstream.ok) {
        const errorText = await upstream.text();
        console.error('Groq upstream error:', upstream.status, errorText);
        return res.status(upstream.status).json({ error: `Groq error ${upstream.status}`, detail: errorText });
      }
      const data = await upstream.json();
      const text = data?.choices?.[0]?.message?.content ?? '';
      const inputTokens = data?.usage?.prompt_tokens ?? 0;
      const outputTokens = data?.usage?.completion_tokens ?? 0;
      return res.json({ text, inputTokens, outputTokens });
    } catch (err) {
      console.error('Groq fallback failure:', err);
      return res.status(502).json({ error: 'Failed to reach Groq', detail: String(err?.message || err) });
    }
  }

  // ── Primary: AWS Bedrock ──────────────────────────────────────────────────
  try {
    const upstream = await fetch(
      `https://bedrock-runtime.${AWS_REGION}.amazonaws.com/model/${BEDROCK_MODEL}/invoke`,
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${BEDROCK_API_KEY}`,
          'x-api-key': BEDROCK_API_KEY,
          Accept: 'application/json',
        },
        body: JSON.stringify({
          anthropic_version: 'bedrock-2023-05-31',
          max_tokens: 8192,
          temperature: 0.3,
          messages: [{ role: 'user', content: prompt }],
        }),
      }
    );

    if (!upstream.ok) {
      const errorText = await upstream.text();
      console.error('Bedrock upstream error:', upstream.status, errorText);
      return res.status(upstream.status).json({ error: `Bedrock error ${upstream.status}`, detail: errorText });
    }

    const data = await upstream.json();
    const text = data?.content?.[0]?.text ?? '';
    const inputTokens = data?.usage?.input_tokens ?? 0;
    const outputTokens = data?.usage?.output_tokens ?? 0;
    res.json({ text, inputTokens, outputTokens });
  } catch (err) {
    console.error('Bedrock proxy failure:', err);
    res.status(502).json({ error: 'Failed to reach Bedrock', detail: String(err?.message || err) });
  }
});

/**
 * Groq — OpenAI-compatible chat completions (used by BRD import + WBS generation).
 */
app.post('/api/ai/groq', async (req, res) => {
  if (!GROQ_API_KEY) {
    return res.status(500).json({ error: 'GROQ_API_KEY is not configured on the proxy.' });
  }
  const { messages, model, temperature, max_tokens } = req.body || {};
  if (!Array.isArray(messages) || messages.length === 0) {
    return res.status(400).json({ error: 'Body must include a non-empty "messages" array.' });
  }

  try {
    const upstream = await fetch(GROQ_API_URL, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${GROQ_API_KEY}`,
      },
      body: JSON.stringify({
        model: model || GROQ_MODEL,
        messages,
        temperature: temperature ?? 0.3,
        max_tokens: max_tokens ?? 4000,
      }),
    });

    if (!upstream.ok) {
      const errorText = await upstream.text();
      console.error('Groq upstream error:', upstream.status, errorText);
      return res.status(upstream.status).json({ error: `Groq error ${upstream.status}`, detail: errorText });
    }

    const data = await upstream.json();
    const content = data?.choices?.[0]?.message?.content ?? '';
    res.json({ content });
  } catch (err) {
    console.error('Groq proxy failure:', err);
    res.status(502).json({ error: 'Failed to reach Groq', detail: String(err?.message || err) });
  }
});

app.listen(PORT, () => {
  console.log(`ESIP AI proxy listening on http://localhost:${PORT}`);
  console.log(`  Allowed origins: ${ALLOWED_ORIGINS.join(', ')}`);
  console.log(`  Bedrock: ${BEDROCK_API_KEY ? 'configured' : 'NOT configured (set BEDROCK_API_KEY)'} (model ${BEDROCK_MODEL})`);
  console.log(`  Groq:    ${GROQ_API_KEY ? 'configured' : 'NOT configured (set GROQ_API_KEY)'} (model ${GROQ_MODEL})`);
});
