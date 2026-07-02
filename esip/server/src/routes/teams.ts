import { Router } from 'express';
import { prisma } from '../lib/prisma.js';
import { authenticate, AuthRequest } from '../middleware/auth.js';
import { z } from 'zod';
import crypto from 'crypto';

const router = Router();

const createTeamSchema = z.object({
  name: z.string(),
});

const inviteSchema = z.object({
  email: z.string().email(),
  role: z.string(),
  projectId: z.string().optional(),
});

// Get user's teams
router.get('/', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const teams = await prisma.team.findMany({
      where: {
        OR: [
          { createdBy: req.user!.userId },
          { members: { some: { userId: req.user!.userId } } },
        ],
      },
      include: {
        creator: { select: { id: true, email: true, fullName: true } },
        members: {
          include: {
            user: { select: { id: true, email: true, fullName: true, role: true } },
          },
        },
        _count: {
          select: { projects: true },
        },
      },
    });

    res.json(teams);
  } catch (error) {
    next(error);
  }
});

// Create team
router.post('/', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const data = createTeamSchema.parse(req.body);

    const team = await prisma.team.create({
      data: {
        name: data.name,
        createdBy: req.user!.userId,
        members: {
          create: {
            userId: req.user!.userId,
            role: 'manager',
          },
        },
      },
      include: {
        creator: { select: { id: true, email: true, fullName: true } },
        members: {
          include: {
            user: { select: { id: true, email: true, fullName: true } },
          },
        },
      },
    });

    res.status(201).json(team);
  } catch (error) {
    next(error);
  }
});

// Invite to team
router.post('/:id/invite', authenticate, async (req: AuthRequest, res, next) => {
  try {
    const data = inviteSchema.parse(req.body);
    const token = crypto.randomBytes(32).toString('hex');
    const expiresAt = new Date();
    expiresAt.setDate(expiresAt.getDate() + 7); // 7 days

    const invitation = await prisma.invitation.create({
      data: {
        email: data.email,
        teamId: req.params.id,
        projectId: data.projectId,
        role: data.role,
        token,
        expiresAt,
        createdBy: req.user!.userId,
      },
    });

    // TODO: Send email with invitation link
    // const inviteLink = `${process.env.APP_URL}/invite?token=${token}`;

    res.status(201).json({
      ...invitation,
      inviteLink: `http://localhost:5173/invite?token=${token}`,
    });
  } catch (error) {
    next(error);
  }
});

// Accept invitation
router.post('/accept-invite', async (req, res, next) => {
  try {
    const { token } = req.body;

    const invitation = await prisma.invitation.findUnique({
      where: { token },
      include: { team: true, project: true },
    });

    if (!invitation) {
      return res.status(404).json({ error: 'Invitation not found' });
    }

    if (invitation.acceptedAt) {
      return res.status(400).json({ error: 'Invitation already accepted' });
    }

    if (new Date() > invitation.expiresAt) {
      return res.status(400).json({ error: 'Invitation expired' });
    }

    res.json({ invitation });
  } catch (error) {
    next(error);
  }
});

export default router;
