#!/bin/bash

# DevPlatform Supabase Setup Script
# This script sets up your PostgreSQL database on Supabase

echo "🚀 DevPlatform - Supabase Setup"
echo "================================"
echo ""

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
  echo "📦 Installing dependencies..."
  npm install
  echo "✅ Dependencies installed!"
  echo ""
fi

# Generate Prisma Client
echo "🔧 Generating Prisma Client..."
npm run db:generate
echo "✅ Prisma Client generated!"
echo ""

# Push schema to database
echo "📊 Pushing database schema to Supabase..."
echo "⚠️  This will create all tables in your PostgreSQL database."
read -p "Continue? (y/n) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
  npm run db:push
  
  if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Database schema successfully created on Supabase!"
    echo ""
    echo "🎉 Setup complete! You can now:"
    echo "   1. Run 'npm run dev' to start the development server"
    echo "   2. Visit your Supabase dashboard at:"
    echo "      https://supabase.com/dashboard/project/cbotkxhmiwgtepjspitc"
    echo "   3. Run 'npm run db:studio' to open Prisma Studio"
    echo ""
  else
    echo ""
    echo "❌ Error pushing schema. Please check your connection string in .env"
    echo "   and make sure Supabase is accessible."
    exit 1
  fi
else
  echo "Setup cancelled."
  exit 0
fi
