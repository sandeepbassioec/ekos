#!/bin/bash

echo "🔄 Starting Supabase Migration..."
echo ""

# Generate Prisma Client
echo "📦 Generating Prisma Client..."
npm run db:generate

# Push schema to Supabase
echo "📊 Pushing schema to Supabase..."
npm run db:push

echo ""
echo "✅ Migration complete!"
echo ""
echo "Next steps:"
echo "1. Start backend: npm run dev"
echo "2. Open frontend in browser"
echo "3. Open console and run: window.migrateToSupabase()"
