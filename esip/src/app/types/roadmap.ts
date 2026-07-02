// ============================================
// ROADMAP TYPES & INTERFACES
// ============================================

// Roadmap Types
export type RoadmapType = 'product' | 'technical' | 'architectural';

// Roadmap Item Status
export type RoadmapItemStatus = 'pending' | 'in_progress' | 'completed' | 'blocked' | 'cancelled';

// Roadmap Item Priority
export type RoadmapItemPriority = 'low' | 'medium' | 'high' | 'critical';

// Quarter
export type Quarter = 'Q1' | 'Q2' | 'Q3' | 'Q4';

// Month (1-12)
export type Month = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12;

// View Mode
export type RoadmapViewMode = 'quarterly' | 'monthly' | 'gantt';

// ============================================
// INTERFACES
// ============================================

/**
 * Roadmap - Main roadmap entity
 */
export interface Roadmap {
  id: string;
  projectId: string;
  type: RoadmapType;
  year: number;
  createdBy: string;
  ownerId?: string; // User responsible for delivering this roadmap
  createdAt: string;
  updatedAt: string;
}

/**
 * RoadmapItem - Individual roadmap milestone/task
 */
export interface RoadmapItem {
  id: string;
  roadmapId: string;
  
  // Basic Info
  title: string;
  description?: string;
  
  // Timeline
  quarter?: Quarter;
  month?: Month;
  startDate?: string; // ISO date string
  endDate?: string; // ISO date string
  
  // Status & Priority
  status: RoadmapItemStatus;
  priority: RoadmapItemPriority;
  
  // Assignment & Dependencies
  assignedTo?: string;
  dependencies: string[]; // Array of RoadmapItem IDs
  
  // Metadata
  createdAt: string;
  updatedAt: string;
}

/**
 * Create Roadmap DTO
 */
export interface CreateRoadmapDTO {
  projectId: string;
  type: RoadmapType;
  year: number;
  ownerId?: string; // Assign to specific user
}

/**
 * Create RoadmapItem DTO
 */
export interface CreateRoadmapItemDTO {
  roadmapId: string;
  title: string;
  description?: string;
  quarter?: Quarter;
  month?: Month;
  startDate?: string;
  endDate?: string;
  status?: RoadmapItemStatus;
  priority?: RoadmapItemPriority;
  assignedTo?: string;
  dependencies?: string[];
}

/**
 * Update RoadmapItem DTO
 */
export interface UpdateRoadmapItemDTO {
  title?: string;
  description?: string;
  quarter?: Quarter;
  month?: Month;
  startDate?: string;
  endDate?: string;
  status?: RoadmapItemStatus;
  priority?: RoadmapItemPriority;
  assignedTo?: string;
  dependencies?: string[];
}

/**
 * Roadmap with Items - Composite view
 */
export interface RoadmapWithItems extends Roadmap {
  items: RoadmapItem[];
}

/**
 * Roadmap Filters
 */
export interface RoadmapFilters {
  projectId: string;
  type?: RoadmapType;
  year?: number;
}

/**
 * RoadmapItem Filters
 */
export interface RoadmapItemFilters {
  roadmapId: string;
  status?: RoadmapItemStatus;
  priority?: RoadmapItemPriority;
  quarter?: Quarter;
  month?: Month;
}

// ============================================
// CONSTANTS
// ============================================

export const ROADMAP_TYPES: { value: RoadmapType; label: string; description: string }[] = [
  { 
    value: 'product', 
    label: 'Product Roadmap', 
    description: 'Features, releases, and product milestones' 
  },
  { 
    value: 'technical', 
    label: 'Technical Roadmap', 
    description: 'Tech stack, infrastructure, and technical debt' 
  },
  { 
    value: 'architectural', 
    label: 'Architectural Roadmap', 
    description: 'System design, architecture decisions, and integrations' 
  },
];

export const ROADMAP_ITEM_STATUSES: { value: RoadmapItemStatus; label: string; color: string }[] = [
  { value: 'pending', label: 'Pending', color: 'bg-gray-100 text-gray-700' },
  { value: 'in_progress', label: 'In Progress', color: 'bg-blue-100 text-blue-700' },
  { value: 'completed', label: 'Completed', color: 'bg-green-100 text-green-700' },
  { value: 'blocked', label: 'Blocked', color: 'bg-red-100 text-red-700' },
  { value: 'cancelled', label: 'Cancelled', color: 'bg-gray-100 text-gray-500' },
];

export const ROADMAP_ITEM_PRIORITIES: { value: RoadmapItemPriority; label: string; color: string }[] = [
  { value: 'low', label: 'Low', color: 'bg-green-100 text-green-700' },
  { value: 'medium', label: 'Medium', color: 'bg-yellow-100 text-yellow-700' },
  { value: 'high', label: 'High', color: 'bg-orange-100 text-orange-700' },
  { value: 'critical', label: 'Critical', color: 'bg-red-100 text-red-700' },
];

export const QUARTERS: { value: Quarter; label: string; months: number[] }[] = [
  { value: 'Q1', label: 'Q1 (Jan-Mar)', months: [1, 2, 3] },
  { value: 'Q2', label: 'Q2 (Apr-Jun)', months: [4, 5, 6] },
  { value: 'Q3', label: 'Q3 (Jul-Sep)', months: [7, 8, 9] },
  { value: 'Q4', label: 'Q4 (Oct-Dec)', months: [10, 11, 12] },
];

export const MONTHS: { value: Month; label: string; short: string }[] = [
  { value: 1, label: 'January', short: 'Jan' },
  { value: 2, label: 'February', short: 'Feb' },
  { value: 3, label: 'March', short: 'Mar' },
  { value: 4, label: 'April', short: 'Apr' },
  { value: 5, label: 'May', short: 'May' },
  { value: 6, label: 'June', short: 'Jun' },
  { value: 7, label: 'July', short: 'Jul' },
  { value: 8, label: 'August', short: 'Aug' },
  { value: 9, label: 'September', short: 'Sep' },
  { value: 10, label: 'October', short: 'Oct' },
  { value: 11, label: 'November', short: 'Nov' },
  { value: 12, label: 'December', short: 'Dec' },
];

export const VIEW_MODES: { value: RoadmapViewMode; label: string; icon: string }[] = [
  { value: 'quarterly', label: 'Quarterly View', icon: 'LayoutGrid' },
  { value: 'monthly', label: 'Monthly Timeline', icon: 'Calendar' },
  { value: 'gantt', label: 'Gantt Chart', icon: 'GanttChart' },
];

// ============================================
// UTILITY FUNCTIONS
// ============================================

/**
 * Get quarter from month number
 */
export function getQuarterFromMonth(month: Month): Quarter {
  if (month >= 1 && month <= 3) return 'Q1';
  if (month >= 4 && month <= 6) return 'Q2';
  if (month >= 7 && month <= 9) return 'Q3';
  return 'Q4';
}

/**
 * Get months in a quarter
 */
export function getMonthsInQuarter(quarter: Quarter): number[] {
  const q = QUARTERS.find(q => q.value === quarter);
  return q?.months || [];
}

/**
 * Get status color
 */
export function getStatusColor(status: RoadmapItemStatus): string {
  const s = ROADMAP_ITEM_STATUSES.find(s => s.value === status);
  return s?.color || 'bg-gray-100 text-gray-700';
}

/**
 * Get priority color
 */
export function getPriorityColor(priority: RoadmapItemPriority): string {
  const p = ROADMAP_ITEM_PRIORITIES.find(p => p.value === priority);
  return p?.color || 'bg-gray-100 text-gray-700';
}

/**
 * Check if item is overdue
 */
export function isItemOverdue(item: RoadmapItem): boolean {
  if (!item.endDate || item.status === 'completed' || item.status === 'cancelled') {
    return false;
  }
  return new Date(item.endDate) < new Date();
}

/**
 * Calculate item progress percentage
 */
export function calculateProgress(item: RoadmapItem): number {
  if (item.status === 'completed') return 100;
  if (item.status === 'cancelled') return 0;
  if (item.status === 'pending') return 0;
  if (item.status === 'blocked') return 50;
  
  // For in_progress, calculate based on dates if available
  if (item.startDate && item.endDate) {
    const start = new Date(item.startDate).getTime();
    const end = new Date(item.endDate).getTime();
    const now = new Date().getTime();
    
    if (now < start) return 0;
    if (now > end) return 100;
    
    const progress = ((now - start) / (end - start)) * 100;
    return Math.min(Math.max(progress, 0), 100);
  }
  
  return 50; // Default for in_progress
}

/**
 * Sort items by priority and date
 */
export function sortRoadmapItems(items: RoadmapItem[]): RoadmapItem[] {
  const priorityOrder: Record<RoadmapItemPriority, number> = {
    critical: 4,
    high: 3,
    medium: 2,
    low: 1,
  };
  
  return [...items].sort((a, b) => {
    // First by priority
    const priorityDiff = priorityOrder[b.priority] - priorityOrder[a.priority];
    if (priorityDiff !== 0) return priorityDiff;
    
    // Then by start date
    if (a.startDate && b.startDate) {
      return new Date(a.startDate).getTime() - new Date(b.startDate).getTime();
    }
    
    // Then by created date
    return new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime();
  });
}