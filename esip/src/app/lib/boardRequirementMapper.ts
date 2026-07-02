/**
 * Mapping helpers between the rich board UI model (types/requirements.ts Requirement)
 * and the lean DAL model (lib/db/types.ts Requirement).
 *
 * The rich fields live in DAL.metadata under a 'capture_board' boardType discriminator,
 * so board entries never collide with Comprehensive Wizard entries in the same collection.
 */

import type { Requirement as DalRequirement } from './db/types';
import type {
  Requirement as BoardRequirement,
  RequirementType,
  RequirementPriority,
  RequirementStatus,
  RequirementComplexity,
} from '../types/requirements';

export const BOARD_TYPE = 'capture_board' as const;

/** True when a DAL requirement was created by the capture board (not the comprehensive wizard). */
export function isBoardReq(r: DalRequirement): boolean {
  return r.metadata?.boardType === BOARD_TYPE;
}

/** Reconstruct the full rich board model from a DAL row. */
export function dalToBoard(r: DalRequirement): BoardRequirement {
  const m = r.metadata ?? {};
  return {
    id: r.id,
    projectId: r.projectId,
    type: (r.category as RequirementType) ?? 'feature_change',
    title: r.title,
    description: r.description ?? '',
    programIncrement: m.programIncrement,
    sprint: m.sprint,
    moduleIds: m.moduleIds ?? [],
    businessValue: m.businessValue ?? '',
    userStory: m.userStory,
    affectedScreens: m.affectedScreens ?? [],
    affectedAPIs: m.affectedAPIs ?? [],
    affectedComponents: m.affectedComponents ?? [],
    technicalNotes: m.technicalNotes,
    acceptanceCriteria: m.acceptanceCriteria ?? [],
    dependencies: m.dependencies ?? [],
    blockers: m.blockers ?? [],
    complexity: (m.complexity as RequirementComplexity) ?? 'medium',
    estimatedHours: m.estimatedHours,
    priority: (r.priority as RequirementPriority) ?? 'medium',
    status: (r.status as RequirementStatus) ?? 'draft',
    assignedTo: m.assignedTo,
    createdBy: r.createdBy,
    createdAt: r.createdAt,
    updatedAt: r.updatedAt,
    attachments: m.attachments ?? [],
    comments: m.comments ?? [],
    finalizedBy: m.finalizedBy,
    finalizedAt: m.finalizedAt,
    allocatedPiId: r.piId,
    allocatedPiName: m.allocatedPiName,
  };
}

/** Build the metadata object from board-specific fields. */
export function boardMetadata(req: Partial<BoardRequirement>): object {
  return {
    boardType: BOARD_TYPE,
    programIncrement: req.programIncrement,
    sprint: req.sprint,
    moduleIds: req.moduleIds ?? [],
    businessValue: req.businessValue ?? '',
    userStory: req.userStory,
    affectedScreens: req.affectedScreens ?? [],
    affectedAPIs: req.affectedAPIs ?? [],
    affectedComponents: req.affectedComponents ?? [],
    technicalNotes: req.technicalNotes,
    acceptanceCriteria: req.acceptanceCriteria ?? [],
    dependencies: req.dependencies ?? [],
    blockers: req.blockers ?? [],
    complexity: req.complexity ?? 'medium',
    estimatedHours: req.estimatedHours,
    screenDetails: req.screenDetails,
    bugDetails: req.bugDetails,
    attachments: req.attachments ?? [],
    comments: req.comments ?? [],
    assignedTo: req.assignedTo,
    finalizedBy: req.finalizedBy,
    finalizedAt: req.finalizedAt,
    allocatedPiName: req.allocatedPiName,
  };
}
