import express from 'express';
import cors from 'cors';
import dotenv from 'dotenv';
import authRoutes from './routes/auth.js';
import projectRoutes from './routes/projects.js';
import wbsRoutes from './routes/wbs.js';
import teamRoutes from './routes/teams.js';
import architectureRoutes from './routes/architecture.js';
import requirementsRoutes from './routes/requirements.js';
import { errorHandler } from './middleware/errorHandler.js';

dotenv.config();

const app = express();
const PORT = process.env.PORT || 3001;

app.use(cors());
app.use(express.json({ limit: '50mb' }));

// Health check
app.get('/health', (req, res) => {
  res.json({ status: 'ok' });
});

// Routes
app.use('/api/auth', authRoutes);
app.use('/api/projects', projectRoutes);
app.use('/api/wbs', wbsRoutes);
app.use('/api/teams', teamRoutes);
app.use('/api/architecture', architectureRoutes);
app.use('/api/requirements', requirementsRoutes);

// Error handling
app.use(errorHandler);

app.listen(PORT, () => {
  console.log(`🚀 Server running on http://localhost:${PORT}`);
});