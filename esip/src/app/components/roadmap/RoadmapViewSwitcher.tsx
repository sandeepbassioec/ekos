import React from 'react';
import { LayoutGrid, Calendar, GanttChart } from 'lucide-react';
import { Button } from '../ui/button';
import { RoadmapViewMode } from '../../types/roadmap';

interface RoadmapViewSwitcherProps {
  viewMode: RoadmapViewMode;
  onViewModeChange: (mode: RoadmapViewMode) => void;
}

const viewModeConfig: Record<RoadmapViewMode, { icon: React.ReactNode; label: string }> = {
  quarterly: { icon: <LayoutGrid className="w-4 h-4" />, label: 'Quarterly' },
  monthly: { icon: <Calendar className="w-4 h-4" />, label: 'Monthly' },
  gantt: { icon: <GanttChart className="w-4 h-4" />, label: 'Gantt' },
};

export function RoadmapViewSwitcher({ viewMode, onViewModeChange }: RoadmapViewSwitcherProps) {
  return (
    <div className="flex items-center gap-1 bg-muted rounded-lg p-1">
      {(Object.keys(viewModeConfig) as RoadmapViewMode[]).map((mode) => (
        <Button
          key={mode}
          variant={viewMode === mode ? 'default' : 'ghost'}
          size="sm"
          onClick={() => onViewModeChange(mode)}
          className="gap-2"
        >
          {viewModeConfig[mode].icon}
          <span className="hidden sm:inline">{viewModeConfig[mode].label}</span>
        </Button>
      ))}
    </div>
  );
}
