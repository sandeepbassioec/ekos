-- ============================================
-- ADD ROADMAP OWNERSHIP
-- ============================================
-- Adds ownerId to track who is responsible for delivering the roadmap

ALTER TABLE "Roadmap" 
ADD COLUMN IF NOT EXISTS "ownerId" UUID REFERENCES auth.users(id);

-- Add index for faster queries
CREATE INDEX IF NOT EXISTS idx_roadmap_owner ON "Roadmap"("ownerId");

-- Add comment
COMMENT ON COLUMN "Roadmap"."ownerId" IS 'User responsible for delivering this roadmap (Product Owner, Tech Lead, Architect)';

-- Update existing roadmaps to set owner = creator (optional migration)
-- UPDATE "Roadmap" SET "ownerId" = "createdBy" WHERE "ownerId" IS NULL;
