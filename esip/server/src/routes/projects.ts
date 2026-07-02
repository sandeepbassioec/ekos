import { Router } from 'express';
import { prisma } from '../lib/prisma.js';
import { authenticate, AuthRequest } from '../middleware/auth.js';
import { z } from 'zod';

const router = Router();

const createProjectSchema = z.object({
  name: z.string(),
  description: z.string().optional(),
  estimationTechnique: z.string().optional(),
  teamId: z.string().optional(),
});

// Get all projects
router.get('/', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const projects = await prisma.project.findMany({
      where: {
        OR: [
          { createdBy: req.user!.userId },
          { collaborators: { some: { userId: req.user!.userId } } },
        ],
      },
      include: {
        creator: {
          select: { id: true, email: true, fullName: true },
        },
        team: {
          select: { id: true, name: true },
        },
        _count: {
          select: { wbsTasks: true },
        },
      },
      orderBy: { updatedAt: 'desc' },
    });

    res.json(projects);
  } catch (error) {
    next(error);
  }
});

// Get project by ID
router.get('/:id', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const project = await prisma.project.findUnique({
      where: { id: req.params.id },
      include: {
        creator: {
          select: { id: true, email: true, fullName: true },
        },
        team: {
          select: { id: true, name: true },
        },
        wbsTasks: {
          include: {
            creator: { select: { id: true, email: true, fullName: true } },
            updater: { select: { id: true, email: true, fullName: true } },
          },
          orderBy: { createdAt: 'asc' },
        },
        collaborators: {
          include: {
            user: { select: { id: true, email: true, fullName: true } },
          },
        },
      },
    });

    if (!project) {
      return res.status(404).json({ error: 'Project not found' });
    }

    // Check access
    const hasAccess =
      project.createdBy === req.user!.userId ||
      project.collaborators.some((c) => c.userId === req.user!.userId);

    if (!hasAccess) {
      return res.status(403).json({ error: 'Access denied' });
    }

    res.json(project);
  } catch (error) {
    next(error);
  }
});

// Create project
router.post('/', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const data = createProjectSchema.parse(req.body);

    const project = await prisma.project.create({
      data: {
        ...data,
        createdBy: req.user!.userId,
      },
      include: {
        creator: {
          select: { id: true, email: true, fullName: true },
        },
      },
    });

    res.status(201).json(project);
  } catch (error) {
    next(error);
  }
});

// Update project
router.put('/:id', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const project = await prisma.project.findUnique({
      where: { id: req.params.id },
    });

    if (!project) {
      return res.status(404).json({ error: 'Project not found' });
    }

    if (project.createdBy !== req.user!.userId) {
      return res.status(403).json({ error: 'Access denied' });
    }

    const updated = await prisma.project.update({
      where: { id: req.params.id },
      data: req.body,
      include: {
        creator: {
          select: { id: true, email: true, fullName: true },
        },
      },
    });

    res.json(updated);
  } catch (error) {
    next(error);
  }
});

// Share project
router.post('/:id/share', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const { userId, permission, expiresAt } = req.body;

    const project = await prisma.project.findUnique({
      where: { id: req.params.id },
    });

    if (!project) {
      return res.status(404).json({ error: 'Project not found' });
    }

    if (project.createdBy !== req.user!.userId) {
      return res.status(403).json({ error: 'Only creator can share' });
    }

    const collaborator = await prisma.projectCollaborator.upsert({
      where: {
        projectId_userId: {
          projectId: req.params.id,
          userId,
        },
      },
      create: {
        projectId: req.params.id,
        userId,
        permission,
        expiresAt: expiresAt ? new Date(expiresAt) : null,
        grantedBy: req.user!.userId,
      },
      update: {
        permission,
        expiresAt: expiresAt ? new Date(expiresAt) : null,
      },
      include: {
        user: { select: { id: true, email: true, fullName: true } },
      },
    });

    res.json(collaborator);
  } catch (error) {
    next(error);
  }
});

export default router;
