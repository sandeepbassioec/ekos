-- ============================================
-- KNOWLEDGE HUB FOR ROADMAP ITEMS
-- ============================================
-- Multi-document BRDs, Teams meetings, Emails, Chats all in one place

-- ============================================
-- 1. DOCUMENTS TABLE (Multiple BRDs, Specs, etc.)
-- ============================================
CREATE TABLE IF NOT EXISTS "Document" (
  "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  "roadmapItemId" UUID NOT NULL REFERENCES "RoadmapItem"("id") ON DELETE CASCADE,
  "name" TEXT NOT NULL,
  "type" TEXT NOT NULL CHECK ("type" IN ('BRD', 'Specification', 'Technical Doc', 'Architecture Doc', 'Other')),
  "version" TEXT, -- e.g., "v1.0", "v2.1"
  "fileUrl" TEXT NOT NULL, -- Supabase Storage URL
  "fileSize" BIGINT, -- in bytes
  "mimeType" TEXT,
  "extractedText" TEXT, -- For AI search
  "uploadedBy" UUID NOT NULL REFERENCES auth.users(id),
  "uploadedAt" TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  "updatedAt" TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_document_roadmap_item ON "Document"("roadmapItemId");
CREATE INDEX IF NOT EXISTS idx_document_uploaded_by ON "Document"("uploadedBy");
CREATE INDEX IF NOT EXISTS idx_document_type ON "Document"("type");

-- ============================================
-- 2. MEETINGS TABLE (Teams recordings/transcripts)
-- ============================================
CREATE TABLE IF NOT EXISTS "Meeting" (
  "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  "roadmapItemId" UUID NOT NULL REFERENCES "RoadmapItem"("id") ON DELETE CASCADE,
  "title" TEXT NOT NULL,
  "date" TIMESTAMP WITH TIME ZONE NOT NULL,
  "recordingUrl" TEXT, -- Optional: Supabase Storage URL for video
  "transcript" TEXT, -- Manual paste or auto-generated
  "participants" TEXT[], -- Array of participant names/emails
  "keyDecisions" TEXT[], -- AI-extracted or manually added
  "notes" TEXT,
  "createdBy" UUID NOT NULL REFERENCES auth.users(id),
  "createdAt" TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  "updatedAt" TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_meeting_roadmap_item ON "Meeting"("roadmapItemId");
CREATE INDEX IF NOT EXISTS idx_meeting_date ON "Meeting"("date");

-- ============================================
-- 3. EMAILS TABLE (Email threads)
-- ============================================
CREATE TABLE IF NOT EXISTS "Email" (
  "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  "roadmapItemId" UUID NOT NULL REFERENCES "RoadmapItem"("id") ON DELETE CASCADE,
  "subject" TEXT NOT NULL,
  "from" TEXT NOT NULL, -- Email sender
  "to" TEXT[], -- Recipients
  "cc" TEXT[], -- CC recipients
  "date" TIMESTAMP WITH TIME ZONE NOT NULL,
  "body" TEXT NOT NULL, -- Pasted email content
  "attachments" TEXT[], -- Links to uploaded attachments
  "createdBy" UUID NOT NULL REFERENCES auth.users(id),
  "createdAt" TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  "updatedAt" TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_email_roadmap_item ON "Email"("roadmapItemId");
CREATE INDEX IF NOT EXISTS idx_email_date ON "Email"("date");

-- ============================================
-- 4. CHATS TABLE (Teams/Slack chat conversations)
-- ============================================
CREATE TABLE IF NOT EXISTS "Chat" (
  "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  "roadmapItemId" UUID NOT NULL REFERENCES "RoadmapItem"("id") ON DELETE CASCADE,
  "platform" TEXT NOT NULL CHECK ("platform" IN ('Teams', 'Slack', 'Other')),
  "title" TEXT, -- e.g., "Discussion about Auth Flow"
  "date" TIMESTAMP WITH TIME ZONE NOT NULL,
  "participants" TEXT[], -- Array of participant names
  "messages" TEXT NOT NULL, -- Pasted chat log
  "channelName" TEXT, -- e.g., "#backend-team"
  "createdBy" UUID NOT NULL REFERENCES auth.users(id),
  "createdAt" TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  "updatedAt" TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_chat_roadmap_item ON "Chat"("roadmapItemId");
CREATE INDEX IF NOT EXISTS idx_chat_date ON "Chat"("date");
CREATE INDEX IF NOT EXISTS idx_chat_platform ON "Chat"("platform");

-- ============================================
-- 5. AI DERIVED REQUIREMENTS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS "AIRequirement" (
  "id" UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  "roadmapItemId" UUID NOT NULL REFERENCES "RoadmapItem"("id") ON DELETE CASCADE,
  "requirement" TEXT NOT NULL,
  "source" TEXT NOT NULL, -- e.g., "BRD v2.1", "Teams Meeting - March 15", "Email from CISO"
  "sourceType" TEXT NOT NULL CHECK ("sourceType" IN ('Document', 'Meeting', 'Email', 'Chat')),
  "sourceId" UUID, -- Reference to the source record
  "category" TEXT, -- e.g., "Functional", "Non-Functional", "Security"
  "priority" TEXT CHECK ("priority" IN ('High', 'Medium', 'Low')),
  "confidence" NUMERIC(3,2), -- AI confidence score (0.00 to 1.00)
  "createdAt" TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  "updatedAt" TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ai_requirement_roadmap_item ON "AIRequirement"("roadmapItemId");
CREATE INDEX IF NOT EXISTS idx_ai_requirement_source_type ON "AIRequirement"("sourceType");
CREATE INDEX IF NOT EXISTS idx_ai_requirement_category ON "AIRequirement"("category");

-- ============================================
-- 6. RLS POLICIES
-- ============================================

-- Document Policies
ALTER TABLE "Document" ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view documents for their project roadmap items"
  ON "Document" FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM "RoadmapItem" ri
      JOIN "Roadmap" r ON ri."roadmapId" = r.id
      JOIN "Project" p ON r."projectId" = p.id
      WHERE ri.id = "Document"."roadmapItemId"
      AND p."createdBy" = auth.uid()
    )
  );

CREATE POLICY "Users can upload documents to their project roadmap items"
  ON "Document" FOR INSERT
  WITH CHECK (
    EXISTS (
      SELECT 1 FROM "RoadmapItem" ri
      JOIN "Roadmap" r ON ri."roadmapId" = r.id
      JOIN "Project" p ON r."projectId" = p.id
      WHERE ri.id = "Document"."roadmapItemId"
      AND p."createdBy" = auth.uid()
    )
  );

CREATE POLICY "Users can update their own documents"
  ON "Document" FOR UPDATE
  USING ("uploadedBy" = auth.uid());

CREATE POLICY "Users can delete their own documents"
  ON "Document" FOR DELETE
  USING ("uploadedBy" = auth.uid());

-- Meeting Policies
ALTER TABLE "Meeting" ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view meetings for their project roadmap items"
  ON "Meeting" FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM "RoadmapItem" ri
      JOIN "Roadmap" r ON ri."roadmapId" = r.id
      JOIN "Project" p ON r."projectId" = p.id
      WHERE ri.id = "Meeting"."roadmapItemId"
      AND p."createdBy" = auth.uid()
    )
  );

CREATE POLICY "Users can add meetings to their project roadmap items"
  ON "Meeting" FOR INSERT
  WITH CHECK (
    EXISTS (
      SELECT 1 FROM "RoadmapItem" ri
      JOIN "Roadmap" r ON ri."roadmapId" = r.id
      JOIN "Project" p ON r."projectId" = p.id
      WHERE ri.id = "Meeting"."roadmapItemId"
      AND p."createdBy" = auth.uid()
    )
  );

CREATE POLICY "Users can update their own meetings"
  ON "Meeting" FOR UPDATE
  USING ("createdBy" = auth.uid());

CREATE POLICY "Users can delete their own meetings"
  ON "Meeting" FOR DELETE
  USING ("createdBy" = auth.uid());

-- Email Policies
ALTER TABLE "Email" ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view emails for their project roadmap items"
  ON "Email" FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM "RoadmapItem" ri
      JOIN "Roadmap" r ON ri."roadmapId" = r.id
      JOIN "Project" p ON r."projectId" = p.id
      WHERE ri.id = "Email"."roadmapItemId"
      AND p."createdBy" = auth.uid()
    )
  );

CREATE POLICY "Users can add emails to their project roadmap items"
  ON "Email" FOR INSERT
  WITH CHECK (
    EXISTS (
      SELECT 1 FROM "RoadmapItem" ri
      JOIN "Roadmap" r ON ri."roadmapId" = r.id
      JOIN "Project" p ON r."projectId" = p.id
      WHERE ri.id = "Email"."roadmapItemId"
      AND p."createdBy" = auth.uid()
    )
  );

CREATE POLICY "Users can update their own emails"
  ON "Email" FOR UPDATE
  USING ("createdBy" = auth.uid());

CREATE POLICY "Users can delete their own emails"
  ON "Email" FOR DELETE
  USING ("createdBy" = auth.uid());

-- Chat Policies
ALTER TABLE "Chat" ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view chats for their project roadmap items"
  ON "Chat" FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM "RoadmapItem" ri
      JOIN "Roadmap" r ON ri."roadmapId" = r.id
      JOIN "Project" p ON r."projectId" = p.id
      WHERE ri.id = "Chat"."roadmapItemId"
      AND p."createdBy" = auth.uid()
    )
  );

CREATE POLICY "Users can add chats to their project roadmap items"
  ON "Chat" FOR INSERT
  WITH CHECK (
    EXISTS (
      SELECT 1 FROM "RoadmapItem" ri
      JOIN "Roadmap" r ON ri."roadmapId" = r.id
      JOIN "Project" p ON r."projectId" = p.id
      WHERE ri.id = "Chat"."roadmapItemId"
      AND p."createdBy" = auth.uid()
    )
  );

CREATE POLICY "Users can update their own chats"
  ON "Chat" FOR UPDATE
  USING ("createdBy" = auth.uid());

CREATE POLICY "Users can delete their own chats"
  ON "Chat" FOR DELETE
  USING ("createdBy" = auth.uid());

-- AI Requirement Policies
ALTER TABLE "AIRequirement" ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view AI requirements for their project roadmap items"
  ON "AIRequirement" FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM "RoadmapItem" ri
      JOIN "Roadmap" r ON ri."roadmapId" = r.id
      JOIN "Project" p ON r."projectId" = p.id
      WHERE ri.id = "AIRequirement"."roadmapItemId"
      AND p."createdBy" = auth.uid()
    )
  );

CREATE POLICY "Users can manage AI requirements for their project roadmap items"
  ON "AIRequirement" FOR ALL
  USING (
    EXISTS (
      SELECT 1 FROM "RoadmapItem" ri
      JOIN "Roadmap" r ON ri."roadmapId" = r.id
      JOIN "Project" p ON r."projectId" = p.id
      WHERE ri.id = "AIRequirement"."roadmapItemId"
      AND p."createdBy" = auth.uid()
    )
  );