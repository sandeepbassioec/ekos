-- ============================================
-- ROADMAP FEATURE - DATABASE SCHEMA
-- ============================================
-- This migration adds roadmap functionality to projects
-- Includes: Product, Technical, and Architectural roadmaps
-- With quarterly/monthly views and Gantt chart support
-- ============================================

-- Enable UUID extension if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================
-- TABLE: Roadmap
-- ============================================
-- Stores roadmap metadata for each project
CREATE TABLE IF NOT EXISTS "Roadmap" (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  "projectId" UUID NOT NULL REFERENCES "Project"(id) ON DELETE CASCADE,
  type TEXT NOT NULL CHECK (type IN ('product', 'technical', 'architectural')),
  year INTEGER NOT NULL CHECK (year >= 2000 AND year <= 2100),
  "createdBy" UUID NOT NULL REFERENCES auth.users(id),
  "createdAt" TIMESTAMPTZ DEFAULT NOW(),
  "updatedAt" TIMESTAMPTZ DEFAULT NOW(),
  
  -- Ensure only one roadmap per project-type-year combination
  UNIQUE("projectId", type, year)
);

-- Index for faster queries
CREATE INDEX IF NOT EXISTS idx_roadmap_project ON "Roadmap"("projectId");
CREATE INDEX IF NOT EXISTS idx_roadmap_year ON "Roadmap"(year);
CREATE INDEX IF NOT EXISTS idx_roadmap_type ON "Roadmap"(type);

-- ============================================
-- TABLE: RoadmapItem
-- ============================================
-- Stores individual roadmap items/milestones
CREATE TABLE IF NOT EXISTS "RoadmapItem" (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  "roadmapId" UUID NOT NULL REFERENCES "Roadmap"(id) ON DELETE CASCADE,
  
  -- Basic Information
  title TEXT NOT NULL,
  description TEXT,
  
  -- Timeline Information
  quarter TEXT CHECK (quarter IN ('Q1', 'Q2', 'Q3', 'Q4')),
  month INTEGER CHECK (month >= 1 AND month <= 12),
  "startDate" DATE,
  "endDate" DATE,
  
  -- Status & Priority
  status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'completed', 'blocked', 'cancelled')),
  priority TEXT NOT NULL DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high', 'critical')),
  
  -- Assignment & Dependencies
  "assignedTo" TEXT, -- Can store user ID or team name
  dependencies JSONB DEFAULT '[]', -- Array of RoadmapItem IDs that this depends on
  
  -- Metadata
  "createdAt" TIMESTAMPTZ DEFAULT NOW(),
  "updatedAt" TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for faster queries
CREATE INDEX IF NOT EXISTS idx_roadmapitem_roadmap ON "RoadmapItem"("roadmapId");
CREATE INDEX IF NOT EXISTS idx_roadmapitem_status ON "RoadmapItem"(status);
CREATE INDEX IF NOT EXISTS idx_roadmapitem_priority ON "RoadmapItem"(priority);
CREATE INDEX IF NOT EXISTS idx_roadmapitem_quarter ON "RoadmapItem"(quarter);
CREATE INDEX IF NOT EXISTS idx_roadmapitem_dates ON "RoadmapItem"("startDate", "endDate");

-- ============================================
-- TRIGGERS: Auto-update updatedAt timestamp
-- ============================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW."updatedAt" = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_roadmap_updated_at
  BEFORE UPDATE ON "Roadmap"
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_roadmapitem_updated_at
  BEFORE UPDATE ON "RoadmapItem"
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- COMMENTS
-- ============================================
COMMENT ON TABLE "Roadmap" IS 'Stores roadmap metadata for projects (Product, Technical, Architectural)';
COMMENT ON TABLE "RoadmapItem" IS 'Stores individual roadmap items with timeline, status, and dependencies';
COMMENT ON COLUMN "RoadmapItem".quarter IS 'Quarter assignment (Q1, Q2, Q3, Q4) - optional';
COMMENT ON COLUMN "RoadmapItem".month IS 'Month assignment (1-12) - optional';
COMMENT ON COLUMN "RoadmapItem".dependencies IS 'JSONB array of RoadmapItem UUIDs that this item depends on';

-- ============================================
-- SAMPLE DATA (Optional - for testing)
-- ============================================
-- Uncomment below to insert sample data

/*
-- Insert a sample roadmap for 2024
INSERT INTO "Roadmap" (id, "projectId", type, year, "createdBy")
VALUES (
  uuid_generate_v4(),
  (SELECT id FROM "Project" LIMIT 1), -- Use first project
  'product',
  2024,
  (SELECT id FROM auth.users LIMIT 1) -- Use first user
);

-- Insert sample roadmap items
INSERT INTO "RoadmapItem" ("roadmapId", title, description, quarter, status, priority, "startDate", "endDate")
VALUES
  (
    (SELECT id FROM "Roadmap" WHERE type = 'product' AND year = 2024 LIMIT 1),
    'MVP Launch',
    'Launch minimum viable product with core features',
    'Q1',
    'completed',
    'critical',
    '2024-01-01',
    '2024-03-31'
  ),
  (
    (SELECT id FROM "Roadmap" WHERE type = 'product' AND year = 2024 LIMIT 1),
    'User Dashboard Redesign',
    'Modernize dashboard UI/UX with analytics',
    'Q2',
    'in_progress',
    'high',
    '2024-04-01',
    '2024-06-30'
  );
*/

-- ============================================
-- END OF MIGRATION
-- ============================================
