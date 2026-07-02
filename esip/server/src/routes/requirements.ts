import { Router } from 'express';
import { prisma } from '../lib/prisma.js';
import { authenticate, AuthRequest } from '../middleware/auth.js';
import { z } from 'zod';

const router = Router();

const createPISchema = z.object({
  projectId: z.string(),
  name: z.string(),
  startDate: z.string().optional(),
  endDate: z.string().optional(),
  status: z.string().optional(),
  objectives: z.string().optional(),
});

const createSprintSchema = z.object({
  projectId: z.string(),
  piId: z.string().optional(),
  name: z.string(),
  startDate: z.string().optional(),
  endDate: z.string().optional(),
  status: z.string().optional(),
  goals: z.string().optional(),
});

const createRequirementSchema = z.object({
  projectId: z.string(),
  piId: z.string().optional(),
  sprintId: z.string().optional(),
  title: z.string(),
  description: z.string().optional(),
  category: z.string(),
  priority: z.string().optional(),
  status: z.string().optional(),
  tags: z.array(z.string()).optional(),
  metadata: z.any().optional(),
});

// ===== PI ENDPOINTS =====

router.get('/pis/project/:projectId', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const pis = await prisma.pI.findMany({
      where: { projectId: req.params.projectId },
      include: {
        sprints: true,
        _count: { select: { requirements: true } },
      },
      orderBy: { createdAt: 'asc' },
    });
    res.json(pis);
  } catch (error) {
    next(error);
  }
});

router.post('/pis', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const data = createPISchema.parse(req.body);
    const pi = await prisma.pI.create({ data });
    res.status(201).json(pi);
  } catch (error) {
    next(error);
  }
});

router.put('/pis/:id', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const pi = await prisma.pI.update({
      where: { id: req.params.id },
      data: req.body,
    });
    res.json(pi);
  } catch (error) {
    next(error);
  }
});

router.delete('/pis/:id', authenticate, async (req: AuthRequest, res, next) => {
  try {
    await prisma.pI.delete({ where: { id: req.params.id } });
    res.status(204).send();
  } catch (error) {
    next(error);
  }
});

// ===== SPRINT ENDPOINTS =====

router.get('/sprints/project/:projectId', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const sprints = await prisma.sprint.findMany({
      where: { projectId: req.params.projectId },
      include: {
        pi: true,
        _count: { select: { requirements: true } },
      },
      orderBy: { createdAt: 'asc' },
    });
    res.json(sprints);
  } catch (error) {
    next(error);
  }
});

router.post('/sprints', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const data = createSprintSchema.parse(req.body);
    const sprint = await prisma.sprint.create({ data });
    res.status(201).json(sprint);
  } catch (error) {
    next(error);
  }
});

router.put('/sprints/:id', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const sprint = await prisma.sprint.update({
      where: { id: req.params.id },
      data: req.body,
    });
    res.json(sprint);
  } catch (error) {
    next(error);
  }
});

router.delete('/sprints/:id', authenticate, async (req: AuthRequest, res, next) => {
  try {
    await prisma.sprint.delete({ where: { id: req.params.id } });
    res.status(204).send();
  } catch (error) {
    next(error);
  }
});

// ===== REQUIREMENT ENDPOINTS =====

router.get('/project/:projectId', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const { piId, sprintId, category, status } = req.query;
    
    const where: any = { projectId: req.params.projectId };
    if (piId) where.piId = piId as string;
    if (sprintId) where.sprintId = sprintId as string;
    if (category) where.category = category as string;
    if (status) where.status = status as string;

    const requirements = await prisma.requirement.findMany({
      where,
      include: {
        pi: true,
        sprint: true,
      },
      orderBy: { createdAt: 'asc' },
    });
    
    res.json(requirements);
  } catch (error) {
    next(error);
  }
});

router.post('/', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const data = createRequirementSchema.parse(req.body);
    const requirement = await prisma.requirement.create({
      data: {
        ...data,
        tags: data.tags ? JSON.stringify(data.tags) : null,
        metadata: data.metadata ? JSON.stringify(data.metadata) : null,
        createdBy: req.user!.userId,
      },
      include: {
        pi: true,
        sprint: true,
      },
    });
    res.status(201).json(requirement);
  } catch (error) {
    next(error);
  }
});

router.put('/:id', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const { tags, metadata, ...rest } = req.body;
    const requirement = await prisma.requirement.update({
      where: { id: req.params.id },
      data: {
        ...rest,
        tags: tags ? JSON.stringify(tags) : undefined,
        metadata: metadata ? JSON.stringify(metadata) : undefined,
      },
      include: {
        pi: true,
        sprint: true,
      },
    });
    res.json(requirement);
  } catch (error) {
    next(error);
  }
});

router.delete('/:id', authenticate, async (req: AuthRequest, res, next) => {
  try {
    await prisma.requirement.delete({ where: { id: req.params.id } });
    res.status(204).send();
  } catch (error) {
    next(error);
  }
});

router.post('/bulk', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const { projectId, requirements } = req.body;
    const created = await Promise.all(
      requirements.map((r: any) =>
        prisma.requirement.create({
          data: {
            ...r,
            projectId,
            tags: r.tags ? JSON.stringify(r.tags) : null,
            metadata: r.metadata ? JSON.stringify(r.metadata) : null,
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
