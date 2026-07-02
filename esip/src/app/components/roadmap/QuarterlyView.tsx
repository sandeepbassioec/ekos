import React from 'react';
import { Plus } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Button } from '../ui/button';
import { RoadmapItem, Quarter, QUARTERS } from '../../types/roadmap';
import { RoadmapItemCard } from './RoadmapItemCard';

interface QuarterlyViewProps {
  items: RoadmapItem[];
  onAddItem: (quarter: Quarter) => void;
  onEditItem: (item: RoadmapItem) => void;
  onDeleteItem: (itemId: string) => void;
  canEdit: boolean;
}

export function QuarterlyView({ 
  items, 
  onAddItem, 
  onEditItem, 
  onDeleteItem,
  canEdit,
}: QuarterlyViewProps) {
  const getItemsByQuarter = (quarter: Quarter) => {
    return items.filter(item => item.quarter === quarter);
  };
  
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      {QUARTERS.map((quarter) => {
        const quarterItems = getItemsByQuarter(quarter.value);
        
        return (
          <div key={quarter.value} className="space-y-3">
            {/* Quarter Header */}
            <Card className="bg-gradient-to-br from-blue-50 to-purple-50">
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-lg">{quarter.label}</CardTitle>
                  {canEdit && (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => onAddItem(quarter.value)}
                      className="h-8 w-8 p-0"
                    >
                      <Plus className="w-4 h-4" />
                    </Button>
                  )}
                </div>
                <div className="text-xs text-muted-foreground">
                  {quarterItems.length} {quarterItems.length === 1 ? 'item' : 'items'}
                </div>
              </CardHeader>
            </Card>
            
            {/* Quarter Items */}
            <div className="space-y-3">
              {quarterItems.length === 0 ? (
                <Card className="border-dashed">
                  <CardContent className="py-8 text-center">
                    <p className="text-sm text-muted-foreground mb-3">No items in {quarter.value}</p>
                    {canEdit && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => onAddItem(quarter.value)}
                      >
                        <Plus className="w-3 h-3 mr-2" />
                        Add Item
                      </Button>
                    )}
                  </CardContent>
                </Card>
              ) : (
                quarterItems.map((item) => (
                  <RoadmapItemCard
                    key={item.id}
                    item={item}
                    onEdit={onEditItem}
                    onDelete={onDeleteItem}
                    canEdit={canEdit}
                  />
                ))
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}
