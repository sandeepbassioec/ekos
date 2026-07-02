import { Router } from 'express';
import { prisma } from '../lib/prisma.js';
import { authenticate, AuthRequest } from '../middleware/auth.js';

const router = Router();

// Get architecture artifacts for project
router.get('/:projectId', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const artifacts = await prisma.architectureArtifact.findMany({
      where: { projectId: req.params.projectId },
      include: {
        creator: { select: { id: true, email: true, fullName: true } },
      },
      orderBy: { createdAt: 'desc' },
    });

    res.json(artifacts);
  } catch (error) {
    next(error);
  }
});

// Create architecture artifact
router.post('/', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const { projectId, type, content } = req.body;

    const artifact = await prisma.architectureArtifact.create({
      data: {
        projectId,
        type,
        content: JSON.stringify(content),
        createdBy: req.user!.userId,
      },
      include: {
        creator: { select: { id: true, email: true, fullName: true } },
      },
    });

    res.status(201).json(artifact);
  } catch (error) {
    next(error);
  }
});

export default router;
