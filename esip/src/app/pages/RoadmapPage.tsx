import React, { useState, useEffect, useMemo } from 'react';
import { useParams, useNavigate } from 'react-router';
import { ArrowLeft, Plus, Download, User } from 'lucide-react';
import { Button } from '../components/ui/button';
import { Card, CardContent } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { useRoadmapStore } from '../store/roadmapStore';
import { useAuthStore } from '../store/authStore';
import { usePersonaStore, getPersona } from '../store/personaStore';
import { db } from '../lib/db';
import { toast } from 'sonner';
import {
  RoadmapType,
  ROADMAP_TYPES,
  Quarter,
  Month,
  RoadmapItem,
  CreateRoadmapItemDTO,
  UpdateRoadmapItemDTO,
} from '../types/roadmap';

// Components
import { RoadmapYearSelector } from '../components/roadmap/RoadmapYearSelector';
import { RoadmapTypeSelector } from '../components/roadmap/RoadmapTypeSelector';
import { RoadmapViewSwitcher } from '../components/roadmap/RoadmapViewSwitcher';
import { QuarterlyView } from '../components/roadmap/QuarterlyView';
import { MonthlyTimelineView } from '../components/roadmap/MonthlyTimelineView';
import { GanttChartView } from '../components/roadmap/GanttChartView';
import { RoadmapItemModal } from '../components/roadmap/RoadmapItemModal';
import { RoadmapOwnerSelector } from '../components/roadmap/RoadmapOwnerSelector';

export default function RoadmapPage() {
  const { projectId } = useParams();
  const navigate = useNavigate();
  const { user } = useAuthStore();
  
  const {
    roadmaps,
    currentRoadmap,
    selectedYear,
    selectedType,
    viewMode,
    isLoading,
    fetchRoadmaps,
    fetchRoadmapByYearAndType,
    createRoadmap,
    createRoadmapItem,
    updateRoadmapItem,
    deleteRoadmapItem,
    setSelectedYear,
    setSelectedType,
    setViewMode,
  } = useRoadmapStore();

  const activePersonaId = usePersonaStore((s) => s.activePersonaId);

  // C3 — the active persona foregrounds roadmap types. Offer the persona's types PLUS any
  // type that already has a roadmap, so existing roadmaps never become unreachable.
  const personaRoadmapTypes = useMemo(() => {
    const valid = new Set(ROADMAP_TYPES.map((t) => t.value));
    return (getPersona(activePersonaId).roadmapTypes || []).filter((t) => valid.has(t as RoadmapType)) as RoadmapType[];
  }, [activePersonaId]);

  const visibleRoadmapTypes = useMemo(() => {
    const all = ROADMAP_TYPES.map((t) => t.value) as RoadmapType[];
    if (personaRoadmapTypes.length === 0) return all;
    const existing = new Set(roadmaps.map((r) => r.type));
    const result = all.filter((v) => personaRoadmapTypes.includes(v) || existing.has(v));
    return result.length ? result : all;
  }, [roadmaps, personaRoadmapTypes]);

  // When the active persona changes, default the roadmap to its primary type. Fires only on
  // persona switch, so a manual type change within a persona is preserved.
  useEffect(() => {
    if (personaRoadmapTypes.length > 0 && !personaRoadmapTypes.includes(selectedType)) {
      setSelectedType(personaRoadmapTypes[0]);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activePersonaId]);

  const [project, setProject] = useState<any>(null);
  const [canEdit, setCanEdit] = useState(false);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<RoadmapItem | undefined>();
  const [defaultQuarter, setDefaultQuarter] = useState<Quarter | undefined>();
  const [defaultMonth, setDefaultMonth] = useState<Month | undefined>();
  const [ownerDialogOpen, setOwnerDialogOpen] = useState(false);
  const [selectedOwnerId, setSelectedOwnerId] = useState<string>('');
  const [ownerName, setOwnerName] = useState<string>('');
  
  // Fetch project details
  useEffect(() => {
    const fetchProject = async () => {
      if (!projectId || !user) return;

      try {
        const data = await db.projects.getById(projectId, user.id);
        setProject(data);
        setCanEdit(data.userId === user.id);
      } catch (error) {
        console.error('Error fetching project:', error);
        toast.error('Failed to fetch project');
      }
    };

    fetchProject();
  }, [projectId, user]);
  
  // Fetch roadmaps for this project
  useEffect(() => {
    if (projectId) {
      fetchRoadmaps(projectId);
    }
  }, [projectId, fetchRoadmaps]);
  
  // Fetch current roadmap when year/type changes
  useEffect(() => {
    if (projectId) {
      fetchRoadmapByYearAndType(projectId, selectedYear, selectedType);
    }
  }, [projectId, selectedYear, selectedType, fetchRoadmapByYearAndType]);
  
  // Get available years
  const availableYears = React.useMemo(() => {
    const years = [...new Set(roadmaps.map(r => r.year))];
    // Always include current year
    const currentYear = new Date().getFullYear();
    if (!years.includes(currentYear)) {
      years.push(currentYear);
    }
    return years.sort((a, b) => b - a);
  }, [roadmaps]);
  
  // Handle add new year
  const handleAddYear = async (year: number) => {
    if (!projectId) return;
    
    // Create roadmap for this year and type
    await createRoadmap({
      projectId,
      type: selectedType,
      year,
    });
    
    // Switch to the new year
    setSelectedYear(year);
  };
  
  // Handle add item
  const handleAddItem = (quarter?: Quarter, month?: Month) => {
    setDefaultQuarter(quarter);
    setDefaultMonth(month);
    setEditingItem(undefined);
    setIsModalOpen(true);
  };
  
  // Handle edit item
  const handleEditItem = (item: RoadmapItem) => {
    setEditingItem(item);
    setDefaultQuarter(undefined);
    setDefaultMonth(undefined);
    setIsModalOpen(true);
  };
  
  // Handle delete item
  const handleDeleteItem = async (itemId: string) => {
    if (confirm('Are you sure you want to delete this item?')) {
      await deleteRoadmapItem(itemId);
    }
  };
  
  // Handle form submit
  const handleFormSubmit = async (data: CreateRoadmapItemDTO | UpdateRoadmapItemDTO) => {
    if (editingItem) {
      // Update existing item
      const success = await updateRoadmapItem(editingItem.id, data as UpdateRoadmapItemDTO);
      if (success) {
        setIsModalOpen(false);
      }
    } else {
      // Create new item - ensure roadmap exists
      if (!currentRoadmap && projectId) {
        // Create roadmap first
        const roadmap = await createRoadmap({
          projectId,
          type: selectedType,
          year: selectedYear,
        });
        
        if (roadmap) {
          // Now create item
          const item = await createRoadmapItem({
            ...data as CreateRoadmapItemDTO,
            roadmapId: roadmap.id,
          });
          if (item) {
            setIsModalOpen(false);
          }
        }
      } else if (currentRoadmap) {
        const item = await createRoadmapItem(data as CreateRoadmapItemDTO);
        if (item) {
          setIsModalOpen(false);
        }
      }
    }
  };
  
  // Fetch owner details
  useEffect(() => {
    if (!currentRoadmap?.ownerId) {
      setOwnerName('');
      return;
    }
    // Note: Firebase doesn't have admin.listUsers in frontend
    // For now, show owner ID or use current user's name if they're the owner
    if (currentRoadmap.ownerId === user?.id) {
      setOwnerName(user?.fullName || user?.email || 'You');
    } else {
      setOwnerName('Team Member');
    }
  }, [currentRoadmap?.ownerId, user]);
  
  if (!project) {
    return (
      <div className="container max-w-7xl mx-auto py-8 px-4">
        <Card>
          <CardContent className="py-16 text-center">
            <div className="w-16 h-16 mx-auto mb-4 text-muted-foreground">Loading...</div>
            <h3 className="text-xl font-semibold mb-2">Loading Project...</h3>
          </CardContent>
        </Card>
      </div>
    );
  }
  
  return (
    <div className="container max-w-7xl mx-auto py-8 px-4">
      {/* Header */}
      <div className="mb-8">
        <Button
          variant="ghost"
          onClick={() => navigate(`/projects/${projectId}`)}
          className="mb-4 -ml-2"
        >
          <ArrowLeft className="w-4 h-4 mr-2" />
          Back to Project
        </Button>
        
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">Project Roadmap</h1>
            <p className="text-muted-foreground mt-1">{project.name}</p>
          </div>
          <div className="flex gap-3">
            {canEdit && (
              <Button onClick={() => handleAddItem()}>
                <Plus className="w-4 h-4 mr-2" />
                Add Item
              </Button>
            )}
            <Button variant="outline">
              <Download className="w-4 h-4 mr-2" />
              Export
            </Button>
          </div>
        </div>
      </div>
      
      {/* Controls */}
      <div className="space-y-4 mb-6">
        {/* Year Selector */}
        <div>
          <h3 className="text-sm font-medium mb-2">Select Year</h3>
          <RoadmapYearSelector
            availableYears={availableYears}
            selectedYear={selectedYear}
            onYearChange={setSelectedYear}
            onAddYear={handleAddYear}
          />
        </div>
        
        {/* Roadmap Type Selector */}
        <div>
          <h3 className="text-sm font-medium mb-2">Roadmap Type</h3>
          <div className="flex items-center gap-4">
            <div className="flex-1">
              <RoadmapTypeSelector
                selectedType={selectedType}
                onTypeChange={setSelectedType}
                allowedTypes={visibleRoadmapTypes}
              />
            </div>
            
            {/* Roadmap Owner Badge */}
            {currentRoadmap && (
              <div className="flex items-center gap-2 bg-muted px-4 py-2 rounded-lg">
                <User className="w-4 h-4 text-muted-foreground" />
                <div>
                  <div className="text-xs text-muted-foreground">Owner</div>
                  <div className="text-sm font-medium">
                    {ownerName || 'Unassigned'}
                  </div>
                </div>
                {canEdit && currentRoadmap.ownerId !== user?.id && (
                  <Badge variant="secondary" className="ml-2">
                    Assigned to you
                  </Badge>
                )}
              </div>
            )}
          </div>
        </div>
        
        {/* View Mode Switcher */}
        <div className="flex justify-end">
          <RoadmapViewSwitcher
            viewMode={viewMode}
            onViewModeChange={setViewMode}
          />
        </div>
      </div>
      
      {/* Roadmap Content */}
      {isLoading ? (
        <Card>
          <CardContent className="py-16 text-center">
            <div className="w-16 h-16 mx-auto mb-4 text-muted-foreground">Loading...</div>
            <h3 className="text-xl font-semibold mb-2">Loading Roadmap...</h3>
          </CardContent>
        </Card>
      ) : !currentRoadmap ? (
        <Card className="border-dashed">
          <CardContent className="py-16 text-center">
            <h3 className="text-xl font-semibold mb-2">No Roadmap Found</h3>
            <p className="text-muted-foreground mb-4">
              Create a {selectedType} roadmap for {selectedYear}
            </p>
            {canEdit && (
              <Button onClick={() => handleAddItem()}>
                <Plus className="w-4 h-4 mr-2" />
                Create Roadmap & Add First Item
              </Button>
            )}
          </CardContent>
        </Card>
      ) : (
        <>
          {viewMode === 'quarterly' && (
            <QuarterlyView
              items={currentRoadmap.items}
              onAddItem={(quarter) => handleAddItem(quarter)}
              onEditItem={handleEditItem}
              onDeleteItem={handleDeleteItem}
              canEdit={canEdit}
            />
          )}
          
          {viewMode === 'monthly' && (
            <MonthlyTimelineView
              items={currentRoadmap.items}
              onAddItem={(month) => handleAddItem(undefined, month)}
              onEditItem={handleEditItem}
              onDeleteItem={handleDeleteItem}
              canEdit={canEdit}
            />
          )}
          
          {viewMode === 'gantt' && (
            <GanttChartView
              items={currentRoadmap.items}
              year={selectedYear}
              onEditItem={handleEditItem}
              onDeleteItem={handleDeleteItem}
              canEdit={canEdit}
            />
          )}
        </>
      )}
      
      {/* Add/Edit Modal */}
      <RoadmapItemModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        roadmapId={currentRoadmap?.id || ''}
        item={editingItem}
        defaultQuarter={defaultQuarter}
        defaultMonth={defaultMonth}
        onSubmit={handleFormSubmit}
      />
    </div>
  );
}