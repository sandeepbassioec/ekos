/**
 * AI Analyzer for Knowledge Hub
 * Batch processes all documents, meetings, emails, chats to extract requirements
 * Using Amazon Bedrock (Claude)
 */

interface AIRequirement {
  requirement: string;
  source: string;
  sourceType: 'Document' | 'Meeting' | 'Email' | 'Chat';
  sourceId: string;
  category: 'Functional' | 'Non-Functional' | 'Security' | 'Performance' | 'Usability' | 'Scalability';
  priority: 'High' | 'Medium' | 'Low';
  confidence: number; // 0.0 to 1.0
}

/**
 * Sign AWS request using Signature Version 4
 */
async function signAwsRequest(
  method: string,
  url: string,
  headers: Record<string, string>,
  body: string,
  accessKeyId: string,
  secretAccessKey: string,
  region: string
): Promise<Record<string, string>> {
  const encoder = new TextEncoder();
  
  // Parse URL
  const urlObj = new URL(url);
  const host = urlObj.hostname;
  const canonicalUri = urlObj.pathname;
  const canonicalQuerystring = urlObj.search.slice(1);
  
  // Create canonical headers
  const datetime = new Date().toISOString().replace(/[:-]|\.\d{3}/g, '');
  const date = datetime.slice(0, 8);
  
  headers['host'] = host;
  headers['x-amz-date'] = datetime;
  
  const sortedHeaders = Object.keys(headers).sort();
  const canonicalHeaders = sortedHeaders
    .map(key => `${key.toLowerCase()}:${headers[key].trim()}\n`)
    .join('');
  const signedHeaders = sortedHeaders.map(key => key.toLowerCase()).join(';');
  
  // Create payload hash
  const payloadHash = await crypto.subtle.digest('SHA-256', encoder.encode(body));
  const payloadHashHex = Array.from(new Uint8Array(payloadHash))
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
  
  // Create canonical request
  const canonicalRequest = [
    method,
    canonicalUri,
    canonicalQuerystring,
    canonicalHeaders,
    signedHeaders,
    payloadHashHex
  ].join('\n');
  
  // Create string to sign
  const algorithm = 'AWS4-HMAC-SHA256';
  const credentialScope = `${date}/${region}/bedrock/aws4_request`;
  const canonicalRequestHash = await crypto.subtle.digest('SHA-256', encoder.encode(canonicalRequest));
  const canonicalRequestHashHex = Array.from(new Uint8Array(canonicalRequestHash))
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
  
  const stringToSign = [
    algorithm,
    datetime,
    credentialScope,
    canonicalRequestHashHex
  ].join('\n');
  
  // Calculate signature
  async function hmac(key: Uint8Array, data: string): Promise<Uint8Array> {
    const cryptoKey = await crypto.subtle.importKey(
      'raw',
      key,
      { name: 'HMAC', hash: 'SHA-256' },
      false,
      ['sign']
    );
    const signature = await crypto.subtle.sign('HMAC', cryptoKey, encoder.encode(data));
    return new Uint8Array(signature);
  }
  
  let key = encoder.encode(`AWS4${secretAccessKey}`);
  key = await hmac(key, date);
  key = await hmac(key, region);
  key = await hmac(key, 'bedrock');
  key = await hmac(key, 'aws4_request');
  const signature = await hmac(key, stringToSign);
  const signatureHex = Array.from(signature)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
  
  // Add authorization header
  headers['Authorization'] = [
    `${algorithm} Credential=${accessKeyId}/${credentialScope}`,
    `SignedHeaders=${signedHeaders}`,
    `Signature=${signatureHex}`
  ].join(', ');
  
  return headers;
}

/**
 * Analyze all sources and extract requirements using Amazon Bedrock (Claude)
 */
export async function analyzeAllSources(
  documents: any[],
  meetings: any[],
  emails: any[],
  chats: any[]
): Promise<AIRequirement[]> {
  const awsAccessKeyId = Deno.env.get('AWS_ACCESS_KEY_ID');
  const awsSecretAccessKey = Deno.env.get('AWS_SECRET_ACCESS_KEY');
  const awsRegion = Deno.env.get('AWS_REGION') || 'us-east-1';
  
  if (!awsAccessKeyId || !awsSecretAccessKey) {
    throw new Error('AWS credentials not configured. Please set AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY in Supabase Edge Function secrets.');
  }

  // Combine all content with source attribution
  let combinedContent = '# KNOWLEDGE BASE FOR REQUIREMENT EXTRACTION\n\n';

  // Add documents
  if (documents.length > 0) {
    combinedContent += '## DOCUMENTS\n\n';
    documents.forEach((doc) => {
      combinedContent += `### ${doc.name} (${doc.type})${doc.version ? ` - ${doc.version}` : ''}\n`;
      combinedContent += `Source ID: ${doc.id}\n`;
      if (doc.extractedText) {
        combinedContent += `${doc.extractedText}\n\n`;
      }
    });
  }

  // Add meetings
  if (meetings.length > 0) {
    combinedContent += '## MEETINGS\n\n';
    meetings.forEach((meeting) => {
      combinedContent += `### ${meeting.title} - ${new Date(meeting.date).toLocaleDateString()}\n`;
      combinedContent += `Source ID: ${meeting.id}\n`;
      combinedContent += `Participants: ${meeting.participants?.join(', ') || 'N/A'}\n`;
      if (meeting.transcript) {
        combinedContent += `Transcript:\n${meeting.transcript}\n\n`;
      }
      if (meeting.notes) {
        combinedContent += `Notes:\n${meeting.notes}\n\n`;
      }
      if (meeting.keyDecisions?.length > 0) {
        combinedContent += `Key Decisions:\n${meeting.keyDecisions.join('\n')}\n\n`;
      }
    });
  }

  // Add emails
  if (emails.length > 0) {
    combinedContent += '## EMAILS\n\n';
    emails.forEach((email) => {
      combinedContent += `### ${email.subject} - ${new Date(email.date).toLocaleDateString()}\n`;
      combinedContent += `Source ID: ${email.id}\n`;
      combinedContent += `From: ${email.from}\n`;
      combinedContent += `To: ${email.to?.join(', ') || 'N/A'}\n`;
      combinedContent += `${email.body}\n\n`;
    });
  }

  // Add chats
  if (chats.length > 0) {
    combinedContent += '## CHAT CONVERSATIONS\n\n';
    chats.forEach((chat) => {
      combinedContent += `### ${chat.title || 'Untitled'} (${chat.platform}) - ${new Date(chat.date).toLocaleDateString()}\n`;
      combinedContent += `Source ID: ${chat.id}\n`;
      combinedContent += `Participants: ${chat.participants?.join(', ') || 'N/A'}\n`;
      if (chat.channelName) {
        combinedContent += `Channel: ${chat.channelName}\n`;
      }
      combinedContent += `${chat.messages}\n\n`;
    });
  }

  // Check if there's any content
  if (documents.length === 0 && meetings.length === 0 && emails.length === 0 && chats.length === 0) {
    return [];
  }

  const systemPrompt = `You are an expert business analyst and requirements engineer. Your task is to analyze the provided knowledge base (documents, meetings, emails, and chats) and extract clear, actionable requirements.

For each requirement you extract:
1. State the requirement clearly and concisely
2. Categorize it as: Functional, Non-Functional, Security, Performance, Usability, or Scalability
3. Assign priority: High, Medium, or Low
4. Provide a confidence score (0.0 to 1.0) based on how explicitly stated the requirement is
5. Reference the source (document name, meeting title, email subject, or chat title)

Return your analysis as a JSON array with this exact structure:
{
  "requirements": [
    {
      "requirement": "Clear statement of the requirement",
      "source": "Document name or meeting title",
      "sourceType": "Document" | "Meeting" | "Email" | "Chat",
      "sourceId": "The Source ID mentioned in the content",
      "category": "Functional" | "Non-Functional" | "Security" | "Performance" | "Usability" | "Scalability",
      "priority": "High" | "Medium" | "Low",
      "confidence": 0.95
    }
  ]
}

Focus on extracting:
- Functional requirements (features, capabilities, behaviors)
- Non-functional requirements (performance, scalability, usability)
- Security requirements (authentication, authorization, data protection)
- Business rules and constraints
- Integration requirements
- User experience requirements

Only extract requirements that are explicitly stated or strongly implied. Do not make assumptions.`;

  try {
    // Use Claude 3.5 Sonnet via Bedrock
    const modelId = 'anthropic.claude-3-5-sonnet-20241022-v2:0';
    const url = `https://bedrock-runtime.${awsRegion}.amazonaws.com/model/${modelId}/invoke`;
    
    const requestBody = {
      anthropic_version: "bedrock-2023-05-31",
      max_tokens: 4096,
      temperature: 0.3,
      system: systemPrompt,
      messages: [
        {
          role: "user",
          content: combinedContent
        }
      ]
    };
    
    const body = JSON.stringify(requestBody);
    
    let headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'Accept': 'application/json'
    };
    
    // Sign the request
    headers = await signAwsRequest(
      'POST',
      url,
      headers,
      body,
      awsAccessKeyId,
      awsSecretAccessKey,
      awsRegion
    );

    const response = await fetch(url, {
      method: 'POST',
      headers,
      body
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error('Bedrock API error:', errorText);
      throw new Error(`Bedrock API error: ${response.status} - ${errorText}`);
    }

    const data = await response.json();
    
    // Extract text from Claude response
    const content = data.content?.[0]?.text;
    if (!content) {
      throw new Error('No content in Bedrock response');
    }

    // Parse the JSON response
    // Claude might wrap JSON in markdown code blocks, so let's handle that
    let jsonText = content.trim();
    if (jsonText.startsWith('```json')) {
      jsonText = jsonText.replace(/```json\n?/g, '').replace(/```\n?/g, '');
    } else if (jsonText.startsWith('```')) {
      jsonText = jsonText.replace(/```\n?/g, '');
    }
    
    const parsed = JSON.parse(jsonText);
    
    // Handle both array and object with requirements key
    const requirements = Array.isArray(parsed) ? parsed : (parsed.requirements || []);

    console.log(`Extracted ${requirements.length} requirements from AI analysis using Claude via Bedrock`);
    
    return requirements;
  } catch (error) {
    console.error('Error in AI analysis:', error);
    throw error;
  }
}
