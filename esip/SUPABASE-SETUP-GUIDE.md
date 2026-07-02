# Supabase Setup Guide

## Quick Setup Steps

### 1. Run the SQL Script
1. Open Supabase SQL Editor: https://supabase.com/dashboard/project/cbotkxhmiwgtepjspitc/sql
2. Copy the entire content from `/supabase-setup.sql`
3. Click "Run" to create all tables

### 2. Disable Email Confirmation (for testing)
1. Go to: https://supabase.com/dashboard/project/cbotkxhmiwgtepjspitc/auth/providers
2. Scroll to "Email" section
3. **Uncheck** "Confirm email" 
4. Click "Save"

### 3. Create Your First User
**Option A: Via App UI**
- Navigate to `/register` in your app
- Fill in the form and submit
- You'll be auto-logged in

**Option B: Via Supabase Dashboard**
1. Go to: https://supabase.com/dashboard/project/cbotkxhmiwgtepjspitc/auth/users
2. Click "Add user" > "Create new user"
3. Enter email and password
4. Click "Create user"

### 4. Login
- Navigate to `/login`
- Use your credentials
- You're in! 🎉

### 5. Migrate Existing Data (Optional)
If you have data in localStorage:
1. Login to your account
2. Open browser console (F12)
3. Run: `window.migrateToSupabase()`
4. Check console for migration results

## Troubleshooting

### 401 Unauthorized Error
- **Cause**: User doesn't exist or wrong credentials
- **Fix**: Create account via `/register` first

### Email Confirmation Required
- **Cause**: Supabase has email confirmation enabled
- **Fix**: Disable it in Auth settings (step 2 above) OR check your email inbox

### Tables Not Found
- **Cause**: SQL script not run
- **Fix**: Run the SQL script from `/supabase-setup.sql`

## Current Configuration

- **Project URL**: https://cbotkxhmiwgtepjspitc.supabase.co
- **Anon Key**: Already configured in code
- **Tables Created**: Project, PI, Sprint, Requirement
- **RLS Enabled**: Yes (Row Level Security for data isolation)
