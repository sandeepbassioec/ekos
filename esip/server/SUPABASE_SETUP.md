# Supabase Migration Guide

## ✅ Setup Complete!

Your DevPlatform backend is now configured to use **Supabase PostgreSQL** instead of local SQLite.

## 🔗 Connection Details

- **Project URL**: https://cbotkxhmiwgtepjspitc.supabase.co
- **Database**: PostgreSQL (cloud-hosted)
- **Connection String**: Configured in `/server/.env`

## 📦 Next Steps

### 1. Install Dependencies (if not already done)

```bash
cd server
npm install
```

### 2. Generate Prisma Client

```bash
npm run db:generate
```

### 3. Push Database Schema to Supabase

Choose one of the following methods:

#### Option A: Quick Push (Recommended for Development)
```bash
npm run db:push
```

This will:
- Create all tables in your Supabase database
- Apply the schema without creating migration files
- Great for rapid development

#### Option B: Create Migration (Recommended for Production)
```bash
npm run db:migrate
```

This will:
- Create a migration file
- Apply the migration to your database
- Track all schema changes

### 4. Verify Database Setup

Visit Supabase Dashboard:
```
https://supabase.com/dashboard/project/cbotkxhmiwgtepjspitc
```

Go to **Table Editor** and you should see:
- ✅ User
- ✅ Team
- ✅ TeamMember
- ✅ Project
- ✅ WbsTask
- ✅ ProjectCollaborator
- ✅ ArchitectureArtifact
- ✅ Invitation
- ✅ AuditLog

### 5. Start Development Server

```bash
npm run dev
```

Your server will now connect to Supabase PostgreSQL! 🎉

## 🔧 Database Management Commands

| Command | Description |
|---------|-------------|
| `npm run db:push` | Push schema changes to database (no migrations) |
| `npm run db:migrate` | Create and apply migration |
| `npm run db:migrate:deploy` | Apply pending migrations (production) |
| `npm run db:generate` | Generate Prisma Client |
| `npm run db:studio` | Open Prisma Studio (database GUI) |

## 🌐 Benefits of Supabase

✅ **Cloud-hosted** - No local database files  
✅ **Persistent** - Data survives server restarts  
✅ **Accessible** - Access from anywhere  
✅ **Backups** - Automatic backups included  
✅ **Scalable** - Grows with your application  
✅ **Real-time** - Built-in real-time subscriptions  
✅ **Authentication** - Built-in auth (optional)  
✅ **Storage** - File storage included  

## 🔐 Security Notes

- ✅ `.env` file is already in `.gitignore`
- ✅ Never commit database credentials to Git
- ✅ Connection string uses URL-encoded password (`%24` = `$`)
- ⚠️ Change `JWT_SECRET` in production

## 📊 Accessing Your Data

### Via Prisma Studio (Local GUI)
```bash
npm run db:studio
```

### Via Supabase Dashboard
1. Visit: https://supabase.com/dashboard/project/cbotkxhmiwgtepjspitc
2. Navigate to **Table Editor**
3. View/edit data directly

### Via SQL Editor
1. Go to Supabase Dashboard → **SQL Editor**
2. Write custom queries
3. View query results

## 🚨 Troubleshooting

### Connection Error
If you get connection errors:
1. Check that your IP is allowed in Supabase (it should be by default)
2. Verify the connection string in `.env`
3. Make sure the password is URL-encoded (`$` → `%24`)

### Migration Errors
If migrations fail:
```bash
# Reset and try again
npm run db:push
```

### Schema Drift
If Prisma complains about schema drift:
```bash
# Generate new migration
npm run db:migrate

# Or force push
npm run db:push --force-reset
```

## 📝 What Changed?

### Before (SQLite)
```prisma
datasource db {
  provider = "sqlite"
  url      = "file:../estimations.db"
}
```

### After (PostgreSQL)
```prisma
datasource db {
  provider = "postgresql"
  url      = env("DATABASE_URL")
}
```

### Database Location
- **Before**: `/server/estimations.db` (local file)
- **After**: Supabase Cloud (PostgreSQL)

## 🎯 Testing the Connection

Create a test file `/server/test-connection.js`:

```javascript
import { PrismaClient } from '@prisma/client';
import dotenv from 'dotenv';

dotenv.config();

const prisma = new PrismaClient();

async function testConnection() {
  try {
    await prisma.$connect();
    console.log('✅ Successfully connected to Supabase PostgreSQL!');
    
    const userCount = await prisma.user.count();
    console.log(`📊 Current user count: ${userCount}`);
    
    await prisma.$disconnect();
  } catch (error) {
    console.error('❌ Connection failed:', error);
    process.exit(1);
  }
}

testConnection();
```

Run it:
```bash
node test-connection.js
```

## 🚀 Ready to Go!

Your backend is now fully configured with Supabase. All your existing API endpoints will work exactly the same, but now with cloud-based persistence!

**Next command to run:**
```bash
cd server
npm run db:push
npm run dev
```
