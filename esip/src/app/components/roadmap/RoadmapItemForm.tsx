import React, { useState, useEffect } from 'react';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Textarea } from '../ui/textarea';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';
import {
  RoadmapItem,
  CreateRoadmapItemDTO,
  UpdateRoadmapItemDTO,
  Quarter,
  Month,
  RoadmapItemStatus,
  RoadmapItemPriority,
  QUARTERS,
  MONTHS,
  ROADMAP_ITEM_STATUSES,
  ROADMAP_ITEM_PRIORITIES,
} from '../../types/roadmap';

interface RoadmapItemFormProps {
  roadmapId: string;
  item?: RoadmapItem; // If editing
  defaultQuarter?: Quarter;
  defaultMonth?: Month;
  onSubmit: (data: CreateRoadmapItemDTO | UpdateRoadmapItemDTO) => void;
  onCancel: () => void;
}

export function RoadmapItemForm({
  roadmapId,
  item,
  defaultQuarter,
  defaultMonth,
  onSubmit,
  onCancel,
}: RoadmapItemFormProps) {
  const [formData, setFormData] = useState({
    title: item?.title || '',
    description: item?.description || '',
    quarter: item?.quarter || defaultQuarter || '' as Quarter | '',
    month: item?.month?.toString() || defaultMonth?.toString() || '',
    startDate: item?.startDate || '',
    endDate: item?.endDate || '',
    status: item?.status || 'pending' as RoadmapItemStatus,
    priority: item?.priority || 'medium' as RoadmapItemPriority,
    assignedTo: item?.assignedTo || '',
  });
  
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    const data: CreateRoadmapItemDTO | UpdateRoadmapItemDTO = {
      title: formData.title,
      description: formData.description || undefined,
      quarter: formData.quarter || undefined,
      month: formData.month ? parseInt(formData.month) as Month : undefined,
      startDate: formData.startDate || undefined,
      endDate: formData.endDate || undefined,
      status: formData.status,
      priority: formData.priority,
      assignedTo: formData.assignedTo || undefined,
    };
    
    if (!item) {
      // Creating new item
      onSubmit({ ...data, roadmapId } as CreateRoadmapItemDTO);
    } else {
      // Updating existing item
      onSubmit(data as UpdateRoadmapItemDTO);
    }
  };
  
  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      {/* Title */}
      <div>
        <Label htmlFor="title">Title *</Label>
        <Input
          id="title"
          value={formData.title}
          onChange={(e) => setFormData({ ...formData, title: e.target.value })}
          placeholder="e.g., Launch MVP, Upgrade Database"
          required
        />
      </div>
      
      {/* Description */}
      <div>
        <Label htmlFor="description">Description</Label>
        <Textarea
          id="description"
          value={formData.description}
          onChange={(e) => setFormData({ ...formData, description: e.target.value })}
          placeholder="Detailed description of the roadmap item..."
          rows={3}
        />
      </div>
      
      {/* Quarter and Month */}
      <div className="grid grid-cols-2 gap-4">
        <div>
          <Label htmlFor="quarter">Quarter</Label>
          <Select
            value={formData.quarter}
            onValueChange={(value) => setFormData({ ...formData, quarter: value as Quarter })}
          >
            <SelectTrigger id="quarter">
              <SelectValue placeholder="Select quarter" />
            </SelectTrigger>
            <SelectContent>
              {QUARTERS.map((q) => (
                <SelectItem key={q.value} value={q.value}>
                  {q.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        
        <div>
          <Label htmlFor="month">Month</Label>
          <Select
            value={formData.month}
            onValueChange={(value) => setFormData({ ...formData, month: value })}
          >
            <SelectTrigger id="month">
              <SelectValue placeholder="Select month" />
            </SelectTrigger>
            <SelectContent>
              {MONTHS.map((m) => (
                <SelectItem key={m.value} value={m.value.toString()}>
                  {m.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
      
      {/* Start and End Dates */}
      <div className="grid grid-cols-2 gap-4">
        <div>
          <Label htmlFor="startDate">Start Date</Label>
          <Input
            id="startDate"
            type="date"
            value={formData.startDate}
            onChange={(e) => setFormData({ ...formData, startDate: e.target.value })}
          />
        </div>
        
        <div>
          <Label htmlFor="endDate">End Date</Label>
          <Input
            id="endDate"
            type="date"
            value={formData.endDate}
            onChange={(e) => setFormData({ ...formData, endDate: e.target.value })}
          />
        </div>
      </div>
      
      {/* Status and Priority */}
      <div className="grid grid-cols-2 gap-4">
        <div>
          <Label htmlFor="status">Status *</Label>
          <Select
            value={formData.status}
            onValueChange={(value) => setFormData({ ...formData, status: value as RoadmapItemStatus })}
          >
            <SelectTrigger id="status">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {ROADMAP_ITEM_STATUSES.map((s) => (
                <SelectItem key={s.value} value={s.value}>
                  {s.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        
        <div>
          <Label htmlFor="priority">Priority *</Label>
          <Select
            value={formData.priority}
            onValueChange={(value) => setFormData({ ...formData, priority: value as RoadmapItemPriority })}
          >
            <SelectTrigger id="priority">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {ROADMAP_ITEM_PRIORITIES.map((p) => (
                <SelectItem key={p.value} value={p.value}>
                  {p.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
      
      {/* Assigned To */}
      <div>
        <Label htmlFor="assignedTo">Assigned To</Label>
        <Input
          id="assignedTo"
          value={formData.assignedTo}
          onChange={(e) => setFormData({ ...formData, assignedTo: e.target.value })}
          placeholder="e.g., John Doe, Frontend Team"
        />
      </div>
      
      {/* Form Actions */}
      <div className="flex justify-end gap-3 pt-4">
        <Button type="button" variant="outline" onClick={onCancel}>
          Cancel
        </Button>
        <Button type="submit" disabled={!formData.title}>
          {item ? 'Update Item' : 'Add Item'}
        </Button>
      </div>
    </form>
  );
}
