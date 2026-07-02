import React from 'react';
import { Package, Code, Network } from 'lucide-react';
import { Tabs, TabsList, TabsTrigger } from '../ui/tabs';
import { RoadmapType, ROADMAP_TYPES } from '../../types/roadmap';

interface RoadmapTypeSelectorProps {
  selectedType: RoadmapType;
  onTypeChange: (type: RoadmapType) => void;
  /** When provided, only these types are offered (Epic C / C3 — persona foregrounding). Defaults to all. */
  allowedTypes?: RoadmapType[];
}

const typeIcons: Record<RoadmapType, React.ReactNode> = {
  product: <Package className="w-4 h-4 mr-2" />,
  technical: <Code className="w-4 h-4 mr-2" />,
  architectural: <Network className="w-4 h-4 mr-2" />,
};

// Static class lookup — Tailwind can't see dynamically-built `grid-cols-${n}` strings.
const COLS_CLASS: Record<number, string> = { 1: 'grid-cols-1', 2: 'grid-cols-2', 3: 'grid-cols-3' };

export function RoadmapTypeSelector({ selectedType, onTypeChange, allowedTypes }: RoadmapTypeSelectorProps) {
  const types = allowedTypes && allowedTypes.length > 0
    ? ROADMAP_TYPES.filter((t) => allowedTypes.includes(t.value))
    : ROADMAP_TYPES;
  return (
    <Tabs value={selectedType} onValueChange={(v) => onTypeChange(v as RoadmapType)}>
      <TabsList className={`grid w-full ${COLS_CLASS[types.length] ?? 'grid-cols-3'}`}>
        {types.map((type) => (
          <TabsTrigger key={type.value} value={type.value} className="flex items-center">
            {typeIcons[type.value]}
            <span className="hidden sm:inline">{type.label}</span>
            <span className="sm:hidden">{type.value.charAt(0).toUpperCase() + type.value.slice(1)}</span>
          </TabsTrigger>
        ))}
      </TabsList>
    </Tabs>
  );
}
