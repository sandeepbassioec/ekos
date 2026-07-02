import React, { useState } from 'react';
import { Plus } from 'lucide-react';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../ui/dialog';
import { Input } from '../ui/input';
import { Label } from '../ui/label';

interface RoadmapYearSelectorProps {
  availableYears: number[];
  selectedYear: number;
  onYearChange: (year: number) => void;
  onAddYear: (year: number) => void;
}

export function RoadmapYearSelector({
  availableYears,
  selectedYear,
  onYearChange,
  onAddYear,
}: RoadmapYearSelectorProps) {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [newYear, setNewYear] = useState<string>(String(new Date().getFullYear() + 1));
  
  const currentYear = new Date().getFullYear();
  
  const handleAddYear = () => {
    const year = parseInt(newYear);
    if (year >= 2000 && year <= 2100 && !availableYears.includes(year)) {
      onAddYear(year);
      setIsDialogOpen(false);
      setNewYear(String(year + 1));
    }
  };
  
  const getYearLabel = (year: number) => {
    if (year === currentYear) return 'Current Year';
    if (year < currentYear) return 'Past';
    return 'Future';
  };
  
  const sortedYears = [...availableYears].sort((a, b) => b - a);
  
  return (
    <>
      <div className="flex items-center gap-2 flex-wrap">
        {sortedYears.map((year) => (
          <Button
            key={year}
            variant={selectedYear === year ? 'default' : 'outline'}
            onClick={() => onYearChange(year)}
            className="relative"
          >
            {year}
            {year === currentYear && (
              <Badge className="ml-2 bg-green-500 text-white text-xs">Current</Badge>
            )}
          </Button>
        ))}
        
        <Button
          variant="outline"
          size="icon"
          onClick={() => setIsDialogOpen(true)}
          className="border-dashed"
        >
          <Plus className="w-4 h-4" />
        </Button>
      </div>
      
      {/* Add New Year Dialog */}
      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add New Year</DialogTitle>
            <DialogDescription>
              Create a roadmap for a new year. You can plan for future years or review past years.
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-4 py-4">
            <div>
              <Label htmlFor="year">Year</Label>
              <Input
                id="year"
                type="number"
                min="2000"
                max="2100"
                value={newYear}
                onChange={(e) => setNewYear(e.target.value)}
                placeholder="2025"
              />
              <p className="text-sm text-muted-foreground mt-1">
                {parseInt(newYear) === currentYear && 'This is the current year'}
                {parseInt(newYear) < currentYear && 'Historical roadmap'}
                {parseInt(newYear) > currentYear && 'Future planning'}
              </p>
            </div>
          </div>
          
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsDialogOpen(false)}>
              Cancel
            </Button>
            <Button 
              onClick={handleAddYear}
              disabled={
                !newYear || 
                parseInt(newYear) < 2000 || 
                parseInt(newYear) > 2100 ||
                availableYears.includes(parseInt(newYear))
              }
            >
              Add Year
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
