-- Development Estimation Platform - Supabase Setup
-- Run this SQL in your Supabase SQL Editor

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Projects Table
CREATE TABLE IF NOT EXISTS "Project" (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL,
  description TEXT,
  status TEXT DEFAULT 'draft',
  "estimationTechnique" TEXT,
  "createdBy" UUID NOT NULL REFERENCES auth.users(id),
  "teamId" UUID,
  "createdAt" TIMESTAMPTZ DEFAULT NOW(),
  "updatedAt" TIMESTAMPTZ DEFAULT NOW()
);

-- PI (Program Increment) Table
CREATE TABLE IF NOT EXISTS "PI" (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  "projectId" UUID NOT NULL REFERENCES "Project"(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  "startDate" TEXT,
  "endDate" TEXT,
  status TEXT DEFAULT 'planned',
  objectives TEXT,
  "createdAt" TIMESTAMPTZ DEFAULT NOW(),
  "updatedAt" TIMESTAMPTZ DEFAULT NOW()
);

-- Sprint Table
CREATE TABLE IF NOT EXISTS "Sprint" (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  "projectId" UUID NOT NULL REFERENCES "Project"(id) ON DELETE CASCADE,
  "piId" UUID REFERENCES "PI"(id) ON DELETE SET NULL,
  name TEXT NOT NULL,
  "startDate" TEXT,
  "endDate" TEXT,
  status TEXT DEFAULT 'planned',
  goals TEXT,
  "createdAt" TIMESTAMPTZ DEFAULT NOW(),
  "updatedAt" TIMESTAMPTZ DEFAULT NOW()
);

-- Requirement Table
CREATE TABLE IF NOT EXISTS "Requirement" (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  "projectId" UUID NOT NULL REFERENCES "Project"(id) ON DELETE CASCADE,
  "piId" UUID REFERENCES "PI"(id) ON DELETE SET NULL,
  "sprintId" UUID REFERENCES "Sprint"(id) ON DELETE SET NULL,
  title TEXT NOT NULL,
  description TEXT,
  category TEXT NOT NULL,
  priority TEXT DEFAULT 'medium',
  status TEXT DEFAULT 'draft',
  tags TEXT,
  metadata TEXT,
  "createdBy" UUID NOT NULL REFERENCES auth.users(id),
  "createdAt" TIMESTAMPTZ DEFAULT NOW(),
  "updatedAt" TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_project_created_by ON "Project"("createdBy");
CREATE INDEX IF NOT EXISTS idx_project_updated_at ON "Project"("updatedAt");
CREATE INDEX IF NOT EXISTS idx_pi_project_id ON "PI"("projectId");
CREATE INDEX IF NOT EXISTS idx_sprint_project_id ON "Sprint"("projectId");
CREATE INDEX IF NOT EXISTS idx_sprint_pi_id ON "Sprint"("piId");
CREATE INDEX IF NOT EXISTS idx_requirement_project_id ON "Requirement"("projectId");
CREATE INDEX IF NOT EXISTS idx_requirement_pi_id ON "Requirement"("piId");
CREATE INDEX IF NOT EXISTS idx_requirement_sprint_id ON "Requirement"("sprintId");
CREATE INDEX IF NOT EXISTS idx_requirement_category ON "Requirement"(category);
CREATE INDEX IF NOT EXISTS idx_requirement_status ON "Requirement"(status);

-- Updated at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW."updatedAt" = NOW();
  RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply updated_at trigger to all tables
DROP TRIGGER IF EXISTS update_project_updated_at ON "Project";
CREATE TRIGGER update_project_updated_at BEFORE UPDATE ON "Project" FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_pi_updated_at ON "PI";
CREATE TRIGGER update_pi_updated_at BEFORE UPDATE ON "PI" FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_sprint_updated_at ON "Sprint";
CREATE TRIGGER update_sprint_updated_at BEFORE UPDATE ON "Sprint" FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_requirement_updated_at ON "Requirement";
CREATE TRIGGER update_requirement_updated_at BEFORE UPDATE ON "Requirement" FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Row Level Security (RLS) Policies
ALTER TABLE "Project" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "PI" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "Sprint" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "Requirement" ENABLE ROW LEVEL SECURITY;

-- Project policies
CREATE POLICY "Users can view their own projects" ON "Project" FOR SELECT USING (auth.uid() = "createdBy");
CREATE POLICY "Users can insert their own projects" ON "Project" FOR INSERT WITH CHECK (auth.uid() = "createdBy");
CREATE POLICY "Users can update their own projects" ON "Project" FOR UPDATE USING (auth.uid() = "createdBy");
CREATE POLICY "Users can delete their own projects" ON "Project" FOR DELETE USING (auth.uid() = "createdBy");

-- PI policies
CREATE POLICY "Users can view PIs for their projects" ON "PI" FOR SELECT USING (
  EXISTS (SELECT 1 FROM "Project" WHERE id = "PI"."projectId" AND "createdBy" = auth.uid())
);
CREATE POLICY "Users can insert PIs for their projects" ON "PI" FOR INSERT WITH CHECK (
  EXISTS (SELECT 1 FROM "Project" WHERE id = "projectId" AND "createdBy" = auth.uid())
);
CREATE POLICY "Users can update PIs for their projects" ON "PI" FOR UPDATE USING (
  EXISTS (SELECT 1 FROM "Project" WHERE id = "projectId" AND "createdBy" = auth.uid())
);
CREATE POLICY "Users can delete PIs for their projects" ON "PI" FOR DELETE USING (
  EXISTS (SELECT 1 FROM "Project" WHERE id = "projectId" AND "createdBy" = auth.uid())
);

-- Sprint policies
CREATE POLICY "Users can view sprints for their projects" ON "Sprint" FOR SELECT USING (
  EXISTS (SELECT 1 FROM "Project" WHERE id = "Sprint"."projectId" AND "createdBy" = auth.uid())
);
CREATE POLICY "Users can insert sprints for their projects" ON "Sprint" FOR INSERT WITH CHECK (
  EXISTS (SELECT 1 FROM "Project" WHERE id = "projectId" AND "createdBy" = auth.uid())
);
CREATE POLICY "Users can update sprints for their projects" ON "Sprint" FOR UPDATE USING (
  EXISTS (SELECT 1 FROM "Project" WHERE id = "projectId" AND "createdBy" = auth.uid())
);
CREATE POLICY "Users can delete sprints for their projects" ON "Sprint" FOR DELETE USING (
  EXISTS (SELECT 1 FROM "Project" WHERE id = "projectId" AND "createdBy" = auth.uid())
);

-- Requirement policies
CREATE POLICY "Users can view requirements for their projects" ON "Requirement" FOR SELECT USING (
  EXISTS (SELECT 1 FROM "Project" WHERE id = "Requirement"."projectId" AND "createdBy" = auth.uid())
);
CREATE POLICY "Users can insert requirements for their projects" ON "Requirement" FOR INSERT WITH CHECK (
  EXISTS (SELECT 1 FROM "Project" WHERE id = "projectId" AND "createdBy" = auth.uid())
);
CREATE POLICY "Users can update requirements for their projects" ON "Requirement" FOR UPDATE USING (
  EXISTS (SELECT 1 FROM "Project" WHERE id = "projectId" AND "createdBy" = auth.uid())
);
CREATE POLICY "Users can delete requirements for their projects" ON "Requirement" FOR DELETE USING (
  EXISTS (SELECT 1 FROM "Project" WHERE id = "projectId" AND "createdBy" = auth.uid())
);
