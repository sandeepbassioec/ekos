# ESIP AI Proxy

A tiny, dependency-light server that sits between the ESIP browser app and the AI providers
(**AWS Bedrock**, **Groq**). It exists for two reasons:

1. **Keep API keys server-side.** They are never shipped in the browser bundle.
2. **Avoid browser CORS.** The browser calls this proxy (same-origin/allowed-origin); the proxy
   calls AWS/Groq.

> This replaces the previous setup where the browser called `bedrock-runtime.amazonaws.com`
> directly with a hardcoded key (CORS failure + leaked secret) and `wbsGenerator.ts` called Groq
> with a hardcoded key.

## Endpoints

| Method | Path | Body | Returns |
|--------|------|------|---------|
| GET | `/health` | — | `{ ok, providers: { bedrock, groq } }` |
| POST | `/api/ai/bedrock` | `{ prompt }` | `{ text }` |
| POST | `/api/ai/groq` | `{ messages, model?, temperature?, max_tokens? }` | `{ content }` |

## Setup

```bash
cd ai-proxy
npm install
cp .env.example .env     # then fill in BEDROCK_API_KEY and GROQ_API_KEY
npm start                # http://localhost:8787  (npm run dev for auto-reload)
```

Point the frontend at the proxy by setting, in the **frontend** `.env`:

```
VITE_AI_PROXY_URL=http://localhost:8787
```

(If unset, the client defaults to `http://localhost:8787`, so local dev works with no extra config.)

## ⚠️ Rotate the leaked keys

The Bedrock and Groq keys that were previously committed to the frontend source **must be
revoked/rotated** in the AWS and Groq consoles. Put the new keys only in this proxy's `.env`.

## Notes

- Bedrock auth uses the Bedrock **API-key bearer** scheme. If AWS requires SigV4 signing in your
  account, replace the `fetch` in `index.js` with `@aws-sdk/client-bedrock-runtime` — the client
  contract (`{ prompt }` → `{ text }`) does not change.
- For production, deploy this anywhere (Render, Fly.io, a Lambda behind API Gateway, etc.) and set
  `ALLOWED_ORIGIN` to your deployed frontend origin.
