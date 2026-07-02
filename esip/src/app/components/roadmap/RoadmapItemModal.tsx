import React from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '../ui/dialog';
import { RoadmapItemForm } from './RoadmapItemForm';
import {
  RoadmapItem,
  CreateRoadmapItemDTO,
  UpdateRoadmapItemDTO,
  Quarter,
  Month,
} from '../../types/roadmap';

interface RoadmapItemModalProps {
  isOpen: boolean;
  onClose: () => void;
  roadmapId: string;
  item?: RoadmapItem;
  defaultQuarter?: Quarter;
  defaultMonth?: Month;
  onSubmit: (data: CreateRoadmapItemDTO | UpdateRoadmapItemDTO) => void;
}

export function RoadmapItemModal({
  isOpen,
  onClose,
  roadmapId,
  item,
  defaultQuarter,
  defaultMonth,
  onSubmit,
}: RoadmapItemModalProps) {
  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{item ? 'Edit Roadmap Item' : 'Add Roadmap Item'}</DialogTitle>
          <DialogDescription>
            {item 
              ? 'Update the roadmap item details below.'
              : 'Add a new milestone or deliverable to your roadmap.'}
          </DialogDescription>
        </DialogHeader>
        
        <RoadmapItemForm
          roadmapId={roadmapId}
          item={item}
          defaultQuarter={defaultQuarter}
          defaultMonth={defaultMonth}
          onSubmit={onSubmit}
          onCancel={onClose}
        />
      </DialogContent>
    </Dialog>
  );
}
