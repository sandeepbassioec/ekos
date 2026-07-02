const AI_PROXY_URL = import.meta.env.VITE_AI_PROXY_URL || 'http://localhost:8787';

export async function generateDesignerContent(prompt: string): Promise<string> {
  const response = await fetch(`${AI_PROXY_URL}/api/ai/bedrock`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ prompt }),
  });
  if (!response.ok) {
    const err = await response.text();
    throw new Error(`AI error ${response.status}: ${err}`);
  }
  const data = await response.json();
  return data.text ?? '';
}
