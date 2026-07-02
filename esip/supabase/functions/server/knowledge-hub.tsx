/**
 * Knowledge Hub Routes
 * Handles documents, meetings, emails, chats, and AI search
 */

import { Hono } from 'npm:hono';
import { createClient } from 'jsr:@supabase/supabase-js@2';
import { analyzeAllSources } from './ai-analyzer.tsx';

const app = new Hono();

// Initialize Supabase client
const getSupabaseClient = (authToken?: string) => {
  const supabaseUrl = Deno.env.get('SUPABASE_URL')!;
  const supabaseKey = authToken 
    ? Deno.env.get('SUPABASE_ANON_KEY')!
    : Deno.env.get('SUPABASE_SERVICE_ROLE_KEY')!;
  
  return createClient(supabaseUrl, supabaseKey, {
    global: {
      headers: authToken ? { Authorization: `Bearer ${authToken}` } : {},
    },
  });
};

// ============================================
// DOCUMENT ROUTES
// ============================================

// Get all documents for a roadmap item
app.get('/documents/:roadmapItemId', async (c) => {
  const roadmapItemId = c.req.param('roadmapItemId');
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  
  const supabase = getSupabaseClient(authToken);
  
  const { data, error } = await supabase
    .from('Document')
    .select('*')
    .eq('roadmapItemId', roadmapItemId)
    .order('uploadedAt', { ascending: false });
  
  if (error) {
    console.error('Error fetching documents:', error);
    return c.json({ error: 'Failed to fetch documents' }, 500);
  }
  
  return c.json({ documents: data });
});

// Upload document
app.post('/documents', async (c) => {
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  const supabase = getSupabaseClient(authToken);
  
  // Get user
  const { data: { user }, error: authError } = await supabase.auth.getUser(authToken);
  if (authError || !user) {
    return c.json({ error: 'Unauthorized' }, 401);
  }
  
  const formData = await c.req.formData();
  const file = formData.get('file') as File;
  const roadmapItemId = formData.get('roadmapItemId') as string;
  const name = formData.get('name') as string;
  const type = formData.get('type') as string;
  const version = formData.get('version') as string | null;
  
  if (!file || !roadmapItemId || !name || !type) {
    return c.json({ error: 'Missing required fields' }, 400);
  }
  
  // Upload file to Supabase Storage
  const bucketName = 'make-ee10ef9d-documents';
  const fileName = `${roadmapItemId}/${Date.now()}-${file.name}`;
  
  // Create bucket if not exists
  const serviceSupabase = getSupabaseClient();
  const { data: buckets } = await serviceSupabase.storage.listBuckets();
  const bucketExists = buckets?.some(bucket => bucket.name === bucketName);
  
  if (!bucketExists) {
    await serviceSupabase.storage.createBucket(bucketName, { public: false });
  }
  
  // Upload file
  const fileBuffer = await file.arrayBuffer();
  const { data: uploadData, error: uploadError } = await serviceSupabase.storage
    .from(bucketName)
    .upload(fileName, fileBuffer, {
      contentType: file.type,
      upsert: false,
    });
  
  if (uploadError) {
    console.error('Error uploading file:', uploadError);
    return c.json({ error: 'Failed to upload file' }, 500);
  }
  
  // Get public URL (will be converted to signed URL on retrieval)
  const fileUrl = `${bucketName}/${fileName}`;
  
  // Extract text from PDF (if applicable)
  let extractedText = '';
  if (file.type === 'application/pdf') {
    // TODO: Implement PDF text extraction
    // For now, we'll leave it empty and implement in next iteration
    extractedText = '';
  }
  
  // Create document record
  const { data: document, error: dbError } = await supabase
    .from('Document')
    .insert({
      roadmapItemId,
      name,
      type,
      version,
      fileUrl,
      fileSize: file.size,
      mimeType: file.type,
      extractedText,
      uploadedBy: user.id,
    })
    .select()
    .single();
  
  if (dbError) {
    console.error('Error creating document record:', dbError);
    return c.json({ error: 'Failed to create document record' }, 500);
  }
  
  return c.json({ document });
});

// Delete document
app.delete('/documents/:documentId', async (c) => {
  const documentId = c.req.param('documentId');
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  
  const supabase = getSupabaseClient(authToken);
  
  // Get document to delete file from storage
  const { data: document } = await supabase
    .from('Document')
    .select('fileUrl')
    .eq('id', documentId)
    .single();
  
  if (document?.fileUrl) {
    const serviceSupabase = getSupabaseClient();
    await serviceSupabase.storage
      .from('make-ee10ef9d-documents')
      .remove([document.fileUrl.replace('make-ee10ef9d-documents/', '')]);
  }
  
  const { error } = await supabase
    .from('Document')
    .delete()
    .eq('id', documentId);
  
  if (error) {
    console.error('Error deleting document:', error);
    return c.json({ error: 'Failed to delete document' }, 500);
  }
  
  return c.json({ success: true });
});

// Get signed URL for document download
app.get('/documents/:documentId/download', async (c) => {
  const documentId = c.req.param('documentId');
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  
  const supabase = getSupabaseClient(authToken);
  
  const { data: document } = await supabase
    .from('Document')
    .select('fileUrl')
    .eq('id', documentId)
    .single();
  
  if (!document) {
    return c.json({ error: 'Document not found' }, 404);
  }
  
  const serviceSupabase = getSupabaseClient();
  const { data, error } = await serviceSupabase.storage
    .from('make-ee10ef9d-documents')
    .createSignedUrl(document.fileUrl.replace('make-ee10ef9d-documents/', ''), 3600);
  
  if (error) {
    console.error('Error creating signed URL:', error);
    return c.json({ error: 'Failed to create download URL' }, 500);
  }
  
  return c.json({ signedUrl: data.signedUrl });
});

// ============================================
// MEETING ROUTES
// ============================================

// Get all meetings for a roadmap item
app.get('/meetings/:roadmapItemId', async (c) => {
  const roadmapItemId = c.req.param('roadmapItemId');
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  
  const supabase = getSupabaseClient(authToken);
  
  const { data, error } = await supabase
    .from('Meeting')
    .select('*')
    .eq('roadmapItemId', roadmapItemId)
    .order('date', { ascending: false });
  
  if (error) {
    console.error('Error fetching meetings:', error);
    return c.json({ error: 'Failed to fetch meetings' }, 500);
  }
  
  return c.json({ meetings: data });
});

// Create meeting
app.post('/meetings', async (c) => {
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  const supabase = getSupabaseClient(authToken);
  
  const { data: { user }, error: authError } = await supabase.auth.getUser(authToken);
  if (authError || !user) {
    return c.json({ error: 'Unauthorized' }, 401);
  }
  
  const body = await c.req.json();
  const { roadmapItemId, title, date, transcript, participants, keyDecisions, notes } = body;
  
  if (!roadmapItemId || !title || !date || !transcript) {
    return c.json({ error: 'Missing required fields' }, 400);
  }
  
  const { data: meeting, error } = await supabase
    .from('Meeting')
    .insert({
      roadmapItemId,
      title,
      date,
      transcript,
      participants: participants || [],
      keyDecisions: keyDecisions || [],
      notes,
      createdBy: user.id,
    })
    .select()
    .single();
  
  if (error) {
    console.error('Error creating meeting:', error);
    return c.json({ error: 'Failed to create meeting' }, 500);
  }
  
  return c.json({ meeting });
});

// Update meeting
app.put('/meetings/:meetingId', async (c) => {
  const meetingId = c.req.param('meetingId');
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  const supabase = getSupabaseClient(authToken);
  
  const body = await c.req.json();
  
  const { data: meeting, error } = await supabase
    .from('Meeting')
    .update(body)
    .eq('id', meetingId)
    .select()
    .single();
  
  if (error) {
    console.error('Error updating meeting:', error);
    return c.json({ error: 'Failed to update meeting' }, 500);
  }
  
  return c.json({ meeting });
});

// Delete meeting
app.delete('/meetings/:meetingId', async (c) => {
  const meetingId = c.req.param('meetingId');
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  const supabase = getSupabaseClient(authToken);
  
  const { error } = await supabase
    .from('Meeting')
    .delete()
    .eq('id', meetingId);
  
  if (error) {
    console.error('Error deleting meeting:', error);
    return c.json({ error: 'Failed to delete meeting' }, 500);
  }
  
  return c.json({ success: true });
});

// ============================================
// EMAIL ROUTES
// ============================================

// Get all emails for a roadmap item
app.get('/emails/:roadmapItemId', async (c) => {
  const roadmapItemId = c.req.param('roadmapItemId');
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  
  const supabase = getSupabaseClient(authToken);
  
  const { data, error } = await supabase
    .from('Email')
    .select('*')
    .eq('roadmapItemId', roadmapItemId)
    .order('date', { ascending: false });
  
  if (error) {
    console.error('Error fetching emails:', error);
    return c.json({ error: 'Failed to fetch emails' }, 500);
  }
  
  return c.json({ emails: data });
});

// Create email
app.post('/emails', async (c) => {
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  const supabase = getSupabaseClient(authToken);
  
  const { data: { user }, error: authError } = await supabase.auth.getUser(authToken);
  if (authError || !user) {
    return c.json({ error: 'Unauthorized' }, 401);
  }
  
  const body = await c.req.json();
  const { roadmapItemId, subject, from, to, cc, date, body: emailBody } = body;
  
  if (!roadmapItemId || !subject || !from || !to || !date || !emailBody) {
    return c.json({ error: 'Missing required fields' }, 400);
  }
  
  const { data: email, error } = await supabase
    .from('Email')
    .insert({
      roadmapItemId,
      subject,
      from,
      to,
      cc: cc || [],
      date,
      body: emailBody,
      createdBy: user.id,
    })
    .select()
    .single();
  
  if (error) {
    console.error('Error creating email:', error);
    return c.json({ error: 'Failed to create email' }, 500);
  }
  
  return c.json({ email });
});

// Delete email
app.delete('/emails/:emailId', async (c) => {
  const emailId = c.req.param('emailId');
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  const supabase = getSupabaseClient(authToken);
  
  const { error } = await supabase
    .from('Email')
    .delete()
    .eq('id', emailId);
  
  if (error) {
    console.error('Error deleting email:', error);
    return c.json({ error: 'Failed to delete email' }, 500);
  }
  
  return c.json({ success: true });
});

// ============================================
// CHAT ROUTES
// ============================================

// Get all chats for a roadmap item
app.get('/chats/:roadmapItemId', async (c) => {
  const roadmapItemId = c.req.param('roadmapItemId');
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  
  const supabase = getSupabaseClient(authToken);
  
  const { data, error } = await supabase
    .from('Chat')
    .select('*')
    .eq('roadmapItemId', roadmapItemId)
    .order('date', { ascending: false });
  
  if (error) {
    console.error('Error fetching chats:', error);
    return c.json({ error: 'Failed to fetch chats' }, 500);
  }
  
  return c.json({ chats: data });
});

// Create chat
app.post('/chats', async (c) => {
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  const supabase = getSupabaseClient(authToken);
  
  const { data: { user }, error: authError } = await supabase.auth.getUser(authToken);
  if (authError || !user) {
    return c.json({ error: 'Unauthorized' }, 401);
  }
  
  const body = await c.req.json();
  const { roadmapItemId, platform, title, date, participants, messages, channelName } = body;
  
  if (!roadmapItemId || !platform || !date || !messages) {
    return c.json({ error: 'Missing required fields' }, 400);
  }
  
  const { data: chat, error } = await supabase
    .from('Chat')
    .insert({
      roadmapItemId,
      platform,
      title,
      date,
      participants: participants || [],
      messages,
      channelName,
      createdBy: user.id,
    })
    .select()
    .single();
  
  if (error) {
    console.error('Error creating chat:', error);
    return c.json({ error: 'Failed to create chat' }, 500);
  }
  
  return c.json({ chat });
});

// Delete chat
app.delete('/chats/:chatId', async (c) => {
  const chatId = c.req.param('chatId');
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  const supabase = getSupabaseClient(authToken);
  
  const { error } = await supabase
    .from('Chat')
    .delete()
    .eq('id', chatId);
  
  if (error) {
    console.error('Error deleting chat:', error);
    return c.json({ error: 'Failed to delete chat' }, 500);
  }
  
  return c.json({ success: true });
});

// ============================================
// KNOWLEDGE HUB AGGREGATE
// ============================================

// Get all knowledge hub data for a roadmap item
app.get('/knowledge-hub/:roadmapItemId', async (c) => {
  const roadmapItemId = c.req.param('roadmapItemId');
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  
  const supabase = getSupabaseClient(authToken);
  
  // Fetch all data in parallel
  const [documentsRes, meetingsRes, emailsRes, chatsRes, requirementsRes] = await Promise.all([
    supabase.from('Document').select('*').eq('roadmapItemId', roadmapItemId).order('uploadedAt', { ascending: false }),
    supabase.from('Meeting').select('*').eq('roadmapItemId', roadmapItemId).order('date', { ascending: false }),
    supabase.from('Email').select('*').eq('roadmapItemId', roadmapItemId).order('date', { ascending: false }),
    supabase.from('Chat').select('*').eq('roadmapItemId', roadmapItemId).order('date', { ascending: false }),
    supabase.from('AIRequirement').select('*').eq('roadmapItemId', roadmapItemId).order('createdAt', { ascending: false }),
  ]);
  
  const documents = documentsRes.data || [];
  const meetings = meetingsRes.data || [];
  const emails = emailsRes.data || [];
  const chats = chatsRes.data || [];
  const aiRequirements = requirementsRes.data || [];
  
  return c.json({
    roadmapItemId,
    documents,
    meetings,
    emails,
    chats,
    aiRequirements,
    stats: {
      totalDocuments: documents.length,
      totalMeetings: meetings.length,
      totalEmails: emails.length,
      totalChats: chats.length,
      totalRequirements: aiRequirements.length,
      totalAIRequirements: aiRequirements.length,
    },
  });
});

// ============================================
// AI ANALYSIS ROUTES
// ============================================

// Analyze all sources and generate requirements
app.post('/analyze-all-sources/:roadmapItemId', async (c) => {
  const roadmapItemId = c.req.param('roadmapItemId');
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  
  const supabase = getSupabaseClient(authToken);
  
  // Verify user is authorized
  const { data: { user }, error: authError } = await supabase.auth.getUser(authToken);
  if (authError || !user) {
    return c.json({ error: 'Unauthorized' }, 401);
  }
  
  try {
    // Fetch all sources
    const [documentsRes, meetingsRes, emailsRes, chatsRes] = await Promise.all([
      supabase.from('Document').select('*').eq('roadmapItemId', roadmapItemId),
      supabase.from('Meeting').select('*').eq('roadmapItemId', roadmapItemId),
      supabase.from('Email').select('*').eq('roadmapItemId', roadmapItemId),
      supabase.from('Chat').select('*').eq('roadmapItemId', roadmapItemId),
    ]);
    
    const documents = documentsRes.data || [];
    const meetings = meetingsRes.data || [];
    const emails = emailsRes.data || [];
    const chats = chatsRes.data || [];
    
    // Check if there's any content to analyze
    const totalSources = documents.length + meetings.length + emails.length + chats.length;
    if (totalSources === 0) {
      return c.json({ error: 'No sources available to analyze' }, 400);
    }
    
    // Analyze using AI
    console.log(`Analyzing ${totalSources} sources for roadmap item ${roadmapItemId}`);
    const requirements = await analyzeAllSources(documents, meetings, emails, chats);
    
    // Delete existing AI requirements for this roadmap item
    await supabase
      .from('AIRequirement')
      .delete()
      .eq('roadmapItemId', roadmapItemId);
    
    // Insert new requirements
    if (requirements.length > 0) {
      const requirementsToInsert = requirements.map(req => ({
        roadmapItemId,
        requirement: req.requirement,
        source: req.source,
        sourceType: req.sourceType,
        sourceId: req.sourceId,
        category: req.category,
        priority: req.priority,
        confidence: req.confidence,
      }));
      
      const { error: insertError } = await supabase
        .from('AIRequirement')
        .insert(requirementsToInsert);
      
      if (insertError) {
        console.error('Error inserting AI requirements:', insertError);
        return c.json({ error: 'Failed to save AI requirements' }, 500);
      }
    }
    
    console.log(`Successfully extracted ${requirements.length} requirements`);
    
    return c.json({
      success: true,
      totalRequirements: requirements.length,
      analyzedSources: {
        documents: documents.length,
        meetings: meetings.length,
        emails: emails.length,
        chats: chats.length,
      },
    });
  } catch (error: any) {
    console.error('Error in AI analysis:', error);
    return c.json({ 
      error: error.message || 'Failed to analyze sources',
      details: error.toString(),
    }, 500);
  }
});

// Delete AI requirement
app.delete('/ai-requirements/:requirementId', async (c) => {
  const requirementId = c.req.param('requirementId');
  const authToken = c.req.header('Authorization')?.split(' ')[1];
  const supabase = getSupabaseClient(authToken);
  
  const { error } = await supabase
    .from('AIRequirement')
    .delete()
    .eq('id', requirementId);
  
  if (error) {
    console.error('Error deleting AI requirement:', error);
    return c.json({ error: 'Failed to delete AI requirement' }, 500);
  }
  
  return c.json({ success: true });
});

export default app;