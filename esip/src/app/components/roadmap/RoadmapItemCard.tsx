import React from 'react';
import { Edit, Trash2, AlertCircle, Calendar, User, Link as LinkIcon, BookOpen } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import { Progress } from '../ui/progress';
import { useNavigate } from 'react-router';
import { 
  RoadmapItem, 
  getStatusColor, 
  getPriorityColor,
  calculateProgress,
  isItemOverdue,
} from '../../types/roadmap';

interface RoadmapItemCardProps {
  item: RoadmapItem;
  onEdit: (item: RoadmapItem) => void;
  onDelete: (itemId: string) => void;
  canEdit: boolean;
}

export function RoadmapItemCard({ item, onEdit, onDelete, canEdit }: RoadmapItemCardProps) {
  const navigate = useNavigate();
  const progress = calculateProgress(item);
  const overdue = isItemOverdue(item);
  
  const formatDate = (date?: string) => {
    if (!date) return null;
    return new Date(date).toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  };
  
  const handleOpenKnowledgeHub = () => {
    navigate(`/roadmap-items/${item.id}/knowledge-hub`);
  };
  
  return (
    <Card className={`hover:shadow-md transition-shadow ${overdue ? 'border-red-300' : ''}`}>
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between gap-2">
          <CardTitle className="text-sm font-semibold line-clamp-2">{item.title}</CardTitle>
          <div className="flex gap-1">
            <Button
              variant="ghost"
              size="sm"
              onClick={handleOpenKnowledgeHub}
              className="h-7 w-7 p-0"
              title="Open Knowledge Hub"
            >
              <BookOpen className="w-3 h-3" />
            </Button>
            {canEdit && (
              <>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => onEdit(item)}
                  className="h-7 w-7 p-0"
                >
                  <Edit className="w-3 h-3" />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => onDelete(item.id)}
                  className="h-7 w-7 p-0 text-red-600 hover:text-red-700"
                >
                  <Trash2 className="w-3 h-3" />
                </Button>
              </>
            )}
          </div>
        </div>
        
        <div className="flex gap-2 mt-2 flex-wrap">
          <Badge className={getStatusColor(item.status)} variant="secondary">
            {item.status.replace('_', ' ')}
          </Badge>
          <Badge className={getPriorityColor(item.priority)} variant="secondary">
            {item.priority}
          </Badge>
          {overdue && (
            <Badge className="bg-red-100 text-red-700" variant="secondary">
              <AlertCircle className="w-3 h-3 mr-1" />
              Overdue
            </Badge>
          )}
        </div>
      </CardHeader>
      
      <CardContent className="space-y-3">
        {/* Description */}
        {item.description && (
          <p className="text-xs text-muted-foreground line-clamp-2">{item.description}</p>
        )}
        
        {/* Progress Bar */}
        {item.status !== 'pending' && item.status !== 'cancelled' && (
          <div className="space-y-1">
            <div className="flex justify-between text-xs">
              <span className="text-muted-foreground">Progress</span>
              <span className="font-medium">{Math.round(progress)}%</span>
            </div>
            <Progress value={progress} className="h-2" />
          </div>
        )}
        
        {/* Dates */}
        {(item.startDate || item.endDate) && (
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <Calendar className="w-3 h-3" />
            <span>
              {formatDate(item.startDate)} {item.startDate && item.endDate && '→'} {formatDate(item.endDate)}
            </span>
          </div>
        )}
        
        {/* Assigned To */}
        {item.assignedTo && (
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <User className="w-3 h-3" />
            <span>{item.assignedTo}</span>
          </div>
        )}
        
        {/* Dependencies */}
        {item.dependencies && item.dependencies.length > 0 && (
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <LinkIcon className="w-3 h-3" />
            <span>{item.dependencies.length} dependencies</span>
          </div>
        )}
      </CardContent>
    </Card>
  );
}