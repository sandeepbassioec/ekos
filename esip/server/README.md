# ESIP Backend Server

## 🚀 Quick Start with Supabase

### Automated Setup (Recommended)

**Windows:**
```bash
setup-supabase.bat
```

**Mac/Linux:**
```bash
chmod +x setup-supabase.sh
./setup-supabase.sh
```

### Manual Setup

1. Install dependencies:
```bash
npm install
```

2. Environment is already configured with Supabase credentials in `.env`

3. Initialize database:
```bash
npm run db:push
```

4. Start development server:
```bash
npm run dev
```

Server runs on http://localhost:3001

📖 **For detailed Supabase setup instructions, see [SUPABASE_SETUP.md](./SUPABASE_SETUP.md)**

## 🗄️ Database

- **Provider**: PostgreSQL (Supabase Cloud)
- **Dashboard**: https://supabase.com/dashboard/project/cbotkxhmiwgtepjspitc
- **Connection**: Configured in `.env`

## 📊 Database Management

| Command | Description |
|---------|-------------|
| `npm run db:push` | Push schema to database |
| `npm run db:migrate` | Create and run migration |
| `npm run db:generate` | Generate Prisma Client |
| `npm run db:studio` | Open Prisma Studio GUI |

## API Endpoints

### Auth
- POST /api/auth/register
- POST /api/auth/login
- GET /api/auth/me

### Projects
- GET /api/projects
- GET /api/projects/:id
- POST /api/projects
- PUT /api/projects/:id
- POST /api/projects/:id/share

### WBS
- GET /api/wbs/project/:projectId
- POST /api/wbs
- PUT /api/wbs/:id
- POST /api/wbs/bulk

### Teams
- GET /api/teams
- POST /api/teams
- POST /api/teams/:id/invite
- POST /api/teams/accept-invite

### Architecture
- GET /api/architecture/:projectId
- POST /api/architecture