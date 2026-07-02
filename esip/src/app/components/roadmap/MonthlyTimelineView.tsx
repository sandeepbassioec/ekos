import React from 'react';
import { Plus } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Button } from '../ui/button';
import { RoadmapItem, Month, MONTHS } from '../../types/roadmap';
import { RoadmapItemCard } from './RoadmapItemCard';

interface MonthlyTimelineViewProps {
  items: RoadmapItem[];
  onAddItem: (month: Month) => void;
  onEditItem: (item: RoadmapItem) => void;
  onDeleteItem: (itemId: string) => void;
  canEdit: boolean;
}

export function MonthlyTimelineView({ 
  items, 
  onAddItem, 
  onEditItem, 
  onDeleteItem,
  canEdit,
}: MonthlyTimelineViewProps) {
  const getItemsByMonth = (month: Month) => {
    return items.filter(item => item.month === month);
  };
  
  return (
    <div className="space-y-4">
      {MONTHS.map((monthData) => {
        const monthItems = getItemsByMonth(monthData.value);
        
        return (
          <Card key={monthData.value}>
            <CardHeader className="pb-4">
              <div className="flex items-center justify-between">
                <div>
                  <CardTitle className="text-base">{monthData.label}</CardTitle>
                  <p className="text-xs text-muted-foreground mt-1">
                    {monthItems.length} {monthItems.length === 1 ? 'item' : 'items'}
                  </p>
                </div>
                {canEdit && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => onAddItem(monthData.value)}
                  >
                    <Plus className="w-3 h-3 mr-2" />
                    Add Item
                  </Button>
                )}
              </div>
            </CardHeader>
            
            <CardContent>
              {monthItems.length === 0 ? (
                <div className="py-8 text-center border-2 border-dashed rounded-lg">
                  <p className="text-sm text-muted-foreground">No items scheduled for {monthData.short}</p>
                </div>
              ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                  {monthItems.map((item) => (
                    <RoadmapItemCard
                      key={item.id}
                      item={item}
                      onEdit={onEditItem}
                      onDelete={onDeleteItem}
                      canEdit={canEdit}
                    />
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}
