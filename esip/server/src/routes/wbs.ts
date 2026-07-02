import { Router } from 'express';
import { prisma } from '../lib/prisma.js';
import { authenticate, AuthRequest, authorize } from '../middleware/auth.js';
import { z } from 'zod';

const router = Router();

const createWbsSchema = z.object({
  projectId: z.string(),
  parentId: z.string().optional(),
  name: z.string(),
  description: z.string().optional(),
  complexity: z.enum(['simple', 'medium', 'complex']).optional(),
  estimatedHours: z.number().optional(),
});

// Get WBS for project
router.get('/project/:projectId', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const tasks = await prisma.wbsTask.findMany({
      where: { projectId: req.params.projectId },
      include: {
        creator: { select: { id: true, email: true, fullName: true } },
        updater: { select: { id: true, email: true, fullName: true } },
      },
      orderBy: { createdAt: 'asc' },
    });

    res.json(tasks);
  } catch (error) {
    next(error);
  }
});

// Create WBS task
router.post('/', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const data = createWbsSchema.parse(req.body);

    const task = await prisma.wbsTask.create({
      data: {
        ...data,
        createdBy: req.user!.userId,
      },
      include: {
        creator: { select: { id: true, email: true, fullName: true } },
      },
    });

    res.status(201).json(task);
  } catch (error) {
    next(error);
  }
});

// Update WBS task (developer or manager)
router.put('/:id', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const task = await prisma.wbsTask.findUnique({
      where: { id: req.params.id },
      include: { project: true },
    });

    if (!task) {
      return res.status(404).json({ error: 'Task not found' });
    }

    const updateData: any = {
      updatedBy: req.user!.userId,
    };

    // Developer can update their estimate
    if (req.user!.role === 'developer') {
      if (req.body.developerEstimatedHours !== undefined) {
        updateData.developerEstimatedHours = req.body.developerEstimatedHours;
        updateData.developerNotes = req.body.developerNotes;
        updateData.status = 'pending';
      }
    }

    // Manager can approve/reject
    if (req.user!.role === 'manager' || req.user!.role === 'admin') {
      if (req.body.status === 'approved') {
        updateData.status = 'approved';
        updateData.estimatedHours = req.body.developerEstimatedHours || task.developerEstimatedHours;
        updateData.managerNotes = req.body.managerNotes;
      } else if (req.body.status === 'rejected') {
        updateData.status = 'rejected';
        updateData.managerNotes = req.body.managerNotes;
      } else {
        Object.assign(updateData, req.body);
      }
    }

    const updated = await prisma.wbsTask.update({
      where: { id: req.params.id },
      data: updateData,
      include: {
        creator: { select: { id: true, email: true, fullName: true } },
        updater: { select: { id: true, email: true, fullName: true } },
      },
    });

    res.json(updated);
  } catch (error) {
    next(error);
  }
});

// Bulk create WBS tasks
router.post('/bulk', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const { projectId, tasks } = req.body;

    const created = await prisma.$transaction(
      tasks.map((task: any) =>
        prisma.wbsTask.create({
          data: {
            projectId,
            ...task,
            createdBy: req.user!.userId,
          },
        })
      )
    );

    res.status(201).json(created);
  } catch (error) {
    next(error);
  }
});

export default router;
