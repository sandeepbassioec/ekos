import React, { useMemo, useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import { Edit, Trash2, FileText, BookOpen } from 'lucide-react';
import { useNavigate } from 'react-router';
import {
  RoadmapItem,
  getStatusColor,
  getPriorityColor,
  MONTHS,
} from '../../types/roadmap';
import { RequirementsPanel } from './RequirementsPanel';

interface GanttChartViewProps {
  items: RoadmapItem[];
  year: number;
  onEditItem: (item: RoadmapItem) => void;
  onDeleteItem: (itemId: string) => void;
  canEdit: boolean;
}

export function GanttChartView({
  items,
  year,
  onEditItem,
  onDeleteItem,
  canEdit,
}: GanttChartViewProps) {
  const [selectedItem, setSelectedItem] = useState<RoadmapItem | null>(null);
  const navigate = useNavigate();
  // Filter items with dates
  const itemsWithDates = useMemo(() => {
    return items.filter(item => item.startDate && item.endDate)
      .sort((a, b) => {
        const dateA = new Date(a.startDate!).getTime();
        const dateB = new Date(b.startDate!).getTime();
        return dateA - dateB;
      });
  }, [items]);
  
  // Calculate position and width for each item
  const getItemPosition = (item: RoadmapItem) => {
    if (!item.startDate || !item.endDate) return { left: 0, width: 0 };
    
    const yearStart = new Date(year, 0, 1).getTime();
    const yearEnd = new Date(year, 11, 31).getTime();
    const yearDuration = yearEnd - yearStart;
    
    const itemStart = new Date(item.startDate).getTime();
    const itemEnd = new Date(item.endDate).getTime();
    
    const left = ((itemStart - yearStart) / yearDuration) * 100;
    const width = ((itemEnd - itemStart) / yearDuration) * 100;
    
    return {
      left: Math.max(0, Math.min(left, 100)),
      width: Math.max(0, Math.min(width, 100 - left)),
    };
  };
  
  if (itemsWithDates.length === 0) {
    return (
      <Card>
        <CardContent className="py-16 text-center">
          <p className="text-muted-foreground mb-2">No items with dates to display in Gantt view</p>
          <p className="text-sm text-muted-foreground">Add start and end dates to roadmap items to see them here</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <>
    <Card>
      <CardHeader>
        <CardTitle>Gantt Chart - {year}</CardTitle>
      </CardHeader>
      <CardContent className="overflow-x-auto">
        <div className="min-w-[800px]">
          {/* Month Headers */}
          <div className="flex border-b mb-4 pb-2">
            {MONTHS.map((month) => (
              <div key={month.value} className="flex-1 text-center text-xs font-semibold text-muted-foreground">
                {month.short}
              </div>
            ))}
          </div>
          
          {/* Timeline Grid */}
          <div className="space-y-3">
            {itemsWithDates.map((item) => {
              const position = getItemPosition(item);
              
              return (
                <div key={item.id} className="relative">
                  {/* Item Label */}
                  <div className="flex items-center gap-2 mb-1">
                    <div className="w-64 truncate text-sm font-medium">
                      {item.title}
                    </div>
                    <div className="flex gap-1">
                      <Badge className={getStatusColor(item.status)} variant="secondary">
                        {item.status.replace('_', ' ')}
                      </Badge>
                      <Badge className={getPriorityColor(item.priority)} variant="secondary">
                        {item.priority}
                      </Badge>
                    </div>
                    <div className="flex gap-1 ml-auto">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => navigate(`/roadmap-items/${item.id}/knowledge-hub`)}
                        className="h-6 w-6 p-0 text-primary"
                        title="Open KnowledgeHub"
                      >
                        <BookOpen className="w-3 h-3" />
                      </Button>
                      {canEdit && (
                        <>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => onEditItem(item)}
                            className="h-6 w-6 p-0"
                          >
                            <Edit className="w-3 h-3" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => onDeleteItem(item.id)}
                            className="h-6 w-6 p-0 text-red-600"
                          >
                            <Trash2 className="w-3 h-3" />
                          </Button>
                        </>
                      )}
                    </div>
                  </div>
                  
                  {/* Timeline Bar */}
                  <div className="relative h-8 bg-gray-100 rounded">
                    {/* Month Dividers */}
                    {MONTHS.map((month, idx) => (
                      <div
                        key={month.value}
                        className="absolute top-0 bottom-0 border-l border-gray-200"
                        style={{ left: `${(idx / 12) * 100}%` }}
                      />
                    ))}
                    
                    {/* Item Bar */}
                    <div
                      className={`absolute top-1 bottom-1 rounded ${
                        item.status === 'completed' ? 'bg-green-500' :
                        item.status === 'in_progress' ? 'bg-blue-500' :
                        item.status === 'blocked' ? 'bg-red-500' :
                        'bg-gray-400'
                      } hover:opacity-80 hover:shadow-lg transition-all cursor-pointer`}
                      style={{
                        left: `${position.left}%`,
                        width: `${position.width}%`,
                      }}
                      onClick={() => setSelectedItem(item)}
                      title={`Click to view requirements: ${item.title}`}
                    >
                      <div className="text-xs text-white px-2 py-1 truncate flex items-center gap-1">
                        <FileText className="w-3 h-3" />
                        {position.width > 15 && item.title}
                      </div>
                    </div>
                  </div>
                  
                  {/* Dates */}
                  <div className="text-xs text-muted-foreground mt-1 ml-64">
                    {new Date(item.startDate!).toLocaleDateString()} → {new Date(item.endDate!).toLocaleDateString()}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </CardContent>
    </Card>

    {/* Requirements Panel */}
    {selectedItem && (
      <RequirementsPanel
        item={selectedItem}
        onClose={() => setSelectedItem(null)}
      />
    )}
  </>
  );
}
