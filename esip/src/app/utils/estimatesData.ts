// Centralized estimates data for both screens

export const screen1Estimates = {
  development: 90,
  qaManual: 72,
  qaAutomated: 98,
  hardening: 85
};

export const screen2Estimates = {
  development: 125,
  qaManual: 98,
  qaAutomated: 130,
  hardening: 95
};

export const getTotalEstimates = () => ({
  development: screen1Estimates.development + screen2Estimates.development,
  qaManual: screen1Estimates.qaManual + screen2Estimates.qaManual,
  qaAutomated: screen1Estimates.qaAutomated + screen2Estimates.qaAutomated,
  hardening: screen1Estimates.hardening + screen2Estimates.hardening,
  totalManual: 
    screen1Estimates.development + 
    screen1Estimates.qaManual + 
    screen1Estimates.hardening +
    screen2Estimates.development + 
    screen2Estimates.qaManual + 
    screen2Estimates.hardening,
  totalAutomated:
    screen1Estimates.development + 
    screen1Estimates.qaAutomated + 
    screen1Estimates.hardening +
    screen2Estimates.development + 
    screen2Estimates.qaAutomated + 
    screen2Estimates.hardening
});

export const screen2WBS = [
  // 2.1 Data Table Component
  { id: '2.1', task: 'Data Table Component', o: null, ml: null, p: null, expected: 22.5, stdDev: 2.5 },
  { id: '2.1.1', task: 'Table header with sortable columns (11+ columns)', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { id: '2.1.2', task: 'Table row component with dynamic data', o: 1.5, ml: 2.5, p: 4, expected: 2.6, stdDev: 0.42 },
  { id: '2.1.3', task: 'Column sorting logic (ascending/descending)', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { id: '2.1.4', task: 'Status badge components (New, Opened, Pending)', o: 1, ml: 1.5, p: 3, expected: 1.7, stdDev: 0.33 },
  { id: '2.1.5', task: 'Photo & Notes icon indicators', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  { id: '2.1.6', task: 'Row actions menu (three dots)', o: 1, ml: 2, p: 3, expected: 2.0, stdDev: 0.33 },
  { id: '2.1.7', task: 'Responsive table with horizontal scroll', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { id: '2.1.8', task: 'Table pagination/infinite scroll', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { id: '2.1.9', task: 'Empty state when no data', o: 0.25, ml: 0.5, p: 1, expected: 0.5, stdDev: 0.13 },
  { id: '2.1.10', task: 'Loading state with skeletons', o: 0.75, ml: 1.5, p: 2.5, expected: 1.5, stdDev: 0.29 },
  { id: '2.1.11', task: 'Cell text truncation with tooltips', o: 0.25, ml: 0.5, p: 1, expected: 0.5, stdDev: 0.13 },
  
  // 2.2 Tab Navigation
  { id: '2.2', task: 'Tab Navigation (All/Open/Closed)', o: null, ml: null, p: null, expected: 3.5, stdDev: 0.5 },
  { id: '2.2.1', task: 'Tab switcher component', o: 0.5, ml: 1, p: 1.5, expected: 1.0, stdDev: 0.17 },
  { id: '2.2.2', task: 'Tab counters (dynamic counts)', o: 0.5, ml: 0.75, p: 1.5, expected: 0.8, stdDev: 0.17 },
  { id: '2.2.3', task: 'Active tab state management', o: 0.25, ml: 0.5, p: 1, expected: 0.5, stdDev: 0.13 },
  { id: '2.2.4', task: 'Filter data based on tab selection', o: 0.75, ml: 1.5, p: 2.5, expected: 1.5, stdDev: 0.29 },
  
  // 2.3 Column Customization Modal
  { id: '2.3', task: 'Column Customization Modal', o: null, ml: null, p: null, expected: 18.0, stdDev: 2.0 },
  { id: '2.3.1', task: 'Modal component (OEC OUI)', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  { id: '2.3.2', task: 'Default tab selector (Overview/Transaction)', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  { id: '2.3.3', task: 'Column list with visibility toggles (eye icons)', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { id: '2.3.4', task: 'Drag & Drop functionality (react-dnd)', o: 3, ml: 5, p: 8, expected: 5.2, stdDev: 0.83 },
  { id: '2.3.5', task: 'Column reorder state management', o: 1.5, ml: 2.5, p: 4, expected: 2.6, stdDev: 0.42 },
  { id: '2.3.6', task: 'Column visibility state management', o: 1, ml: 1.5, p: 3, expected: 1.7, stdDev: 0.33 },
  { id: '2.3.7', task: 'Save/Cancel actions', o: 0.5, ml: 0.75, p: 1.5, expected: 0.8, stdDev: 0.17 },
  { id: '2.3.8', task: 'Persist settings (localStorage)', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  { id: '2.3.9', task: 'Reset to default functionality', o: 0.5, ml: 1, p: 1.5, expected: 1.0, stdDev: 0.17 },
  { id: '2.3.10', task: 'Instructions text and UI polish', o: 0.15, ml: 0.25, p: 0.5, expected: 0.3, stdDev: 0.06 },
  
  // 2.4 Filter Modal
  { id: '2.4', task: 'Enhanced Filter Modal', o: null, ml: null, p: null, expected: 12.5, stdDev: 1.5 },
  { id: '2.4.1', task: 'Modal component (OEC OUI)', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  { id: '2.4.2', task: 'Transaction Status filter (6 checkboxes + counters)', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { id: '2.4.3', task: 'OEM filter (3 checkboxes + counters)', o: 1, ml: 1.5, p: 3, expected: 1.7, stdDev: 0.33 },
  { id: '2.4.4', task: 'Salesperson filter (3 checkboxes + counters)', o: 1, ml: 1.5, p: 3, expected: 1.7, stdDev: 0.33 },
  { id: '2.4.5', task: 'Filter state management', o: 1.5, ml: 2.5, p: 4, expected: 2.6, stdDev: 0.42 },
  { id: '2.4.6', task: 'Apply/Cancel actions', o: 0.5, ml: 0.75, p: 1.5, expected: 0.8, stdDev: 0.17 },
  { id: '2.4.7', task: 'Clear all filters', o: 0.25, ml: 0.5, p: 1, expected: 0.5, stdDev: 0.13 },
  { id: '2.4.8', task: 'Filter count badges', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  
  // 2.5 Search Functionality
  { id: '2.5', task: 'Table Search Functionality', o: null, ml: null, p: null, expected: 6.5, stdDev: 0.83 },
  { id: '2.5.1', task: 'Search input with icon', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  { id: '2.5.2', task: 'Search algorithm across all columns', o: 1.5, ml: 2.5, p: 4, expected: 2.6, stdDev: 0.42 },
  { id: '2.5.3', task: 'Debounced search (performance)', o: 0.5, ml: 0.75, p: 1.5, expected: 0.8, stdDev: 0.17 },
  { id: '2.5.4', task: 'Highlight search results', o: 0.75, ml: 1.5, p: 2.5, expected: 1.5, stdDev: 0.29 },
  { id: '2.5.5', task: 'Clear search functionality', o: 0.15, ml: 0.25, p: 0.5, expected: 0.3, stdDev: 0.06 },
  { id: '2.5.6', task: 'Empty search results state', o: 0.15, ml: 0.25, p: 0.5, expected: 0.3, stdDev: 0.06 },
  
  // 2.6 More Options Menu
  { id: '2.6', task: 'More Options Dropdown', o: null, ml: null, p: null, expected: 8.5, stdDev: 1.0 },
  { id: '2.6.1', task: 'Dropdown menu component', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  { id: '2.6.2', task: 'Quick Print action', o: 1, ml: 2, p: 3, expected: 2.0, stdDev: 0.33 },
  { id: '2.6.3', task: 'Advanced Print action', o: 1.5, ml: 2.5, p: 4, expected: 2.6, stdDev: 0.42 },
  { id: '2.6.4', task: 'Attached Note action', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  { id: '2.6.5', task: 'Attached Photo action', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  { id: '2.6.6', task: 'Archive action', o: 0.25, ml: 0.5, p: 1, expected: 0.5, stdDev: 0.13 },
  { id: '2.6.7', task: 'Customize Columns action link', o: 0.15, ml: 0.25, p: 0.5, expected: 0.3, stdDev: 0.06 },
  
  // 2.7 Filter Logic
  { id: '2.7', task: 'Combined Filter Logic', o: null, ml: null, p: null, expected: 10.0, stdDev: 1.17 },
  { id: '2.7.1', task: 'Transaction status filtering', o: 1.5, ml: 2.5, p: 4, expected: 2.6, stdDev: 0.42 },
  { id: '2.7.2', task: 'OEM filtering', o: 1, ml: 1.5, p: 3, expected: 1.7, stdDev: 0.33 },
  { id: '2.7.3', task: 'Salesperson filtering', o: 1, ml: 1.5, p: 3, expected: 1.7, stdDev: 0.33 },
  { id: '2.7.4', task: 'Combined filter logic (AND/OR)', o: 1.5, ml: 2.5, p: 4, expected: 2.6, stdDev: 0.42 },
  { id: '2.7.5', task: 'Filter + Search combined logic', o: 0.75, ml: 1.5, p: 2.5, expected: 1.5, stdDev: 0.29 },
  
  // 2.8 API Integration
  { id: '2.8', task: 'API Integration', o: null, ml: null, p: null, expected: 12.0, stdDev: 1.33 },
  { id: '2.8.1', task: 'API client setup (reused from Screen 1)', o: 0.25, ml: 0.5, p: 1, expected: 0.5, stdDev: 0.13 },
  { id: '2.8.2', task: 'Transaction list endpoint', o: 1.5, ml: 2.5, p: 4, expected: 2.6, stdDev: 0.42 },
  { id: '2.8.3', task: 'Filter options endpoint', o: 1, ml: 1.5, p: 3, expected: 1.7, stdDev: 0.33 },
  { id: '2.8.4', task: 'Column preferences endpoint', o: 1, ml: 1.5, p: 3, expected: 1.7, stdDev: 0.33 },
  { id: '2.8.5', task: 'Error handling', o: 1, ml: 2, p: 3, expected: 2.0, stdDev: 0.33 },
  { id: '2.8.6', task: 'Loading states', o: 1, ml: 1.5, p: 3, expected: 1.7, stdDev: 0.33 },
  { id: '2.8.7', task: 'Data transformation/mapping', o: 0.75, ml: 1.5, p: 2.5, expected: 1.5, stdDev: 0.29 },
  { id: '2.8.8', task: 'Pagination/infinite scroll data fetching', o: 0.25, ml: 0.5, p: 1, expected: 0.5, stdDev: 0.13 },
  
  // 2.9 State Management
  { id: '2.9', task: 'State Management', o: null, ml: null, p: null, expected: 10.0, stdDev: 1.17 },
  { id: '2.9.1', task: 'Table data state', o: 0.75, ml: 1.5, p: 2.5, expected: 1.5, stdDev: 0.29 },
  { id: '2.9.2', task: 'Filter state', o: 1, ml: 2, p: 3, expected: 2.0, stdDev: 0.33 },
  { id: '2.9.3', task: 'Search state', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  { id: '2.9.4', task: 'Column visibility state', o: 0.75, ml: 1.5, p: 2.5, expected: 1.5, stdDev: 0.29 },
  { id: '2.9.5', task: 'Column order state', o: 0.75, ml: 1.5, p: 2.5, expected: 1.5, stdDev: 0.29 },
  { id: '2.9.6', task: 'Sort state (column + direction)', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  { id: '2.9.7', task: 'Tab state (All/Open/Closed)', o: 0.25, ml: 0.5, p: 1, expected: 0.5, stdDev: 0.13 },
  { id: '2.9.8', task: 'UI state (loading, errors)', o: 0.5, ml: 1, p: 1.5, expected: 1.0, stdDev: 0.17 },
  
  // 2.10 TypeScript Types
  { id: '2.10', task: 'TypeScript Types & Interfaces', o: null, ml: null, p: null, expected: 6.0, stdDev: 0.67 },
  { id: '2.10.1', task: 'Transaction row types', o: 1, ml: 1.5, p: 3, expected: 1.7, stdDev: 0.33 },
  { id: '2.10.2', task: 'Column configuration types', o: 0.75, ml: 1.5, p: 2.5, expected: 1.5, stdDev: 0.29 },
  { id: '2.10.3', task: 'Filter option types', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  { id: '2.10.4', task: 'API response types', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  { id: '2.10.5', task: 'Component prop types', o: 0.25, ml: 0.5, p: 1, expected: 0.5, stdDev: 0.13 },
  { id: '2.10.6', task: 'Enum types (Status, etc.)', o: 0.15, ml: 0.25, p: 0.5, expected: 0.3, stdDev: 0.06 },
  
  // 2.11 Responsive Design
  { id: '2.11', task: 'Responsive Design & Polish', o: null, ml: null, p: null, expected: 9.5, stdDev: 1.08 },
  { id: '2.11.1', task: 'Mobile responsive table (horizontal scroll)', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { id: '2.11.2', task: 'Tablet responsive layout', o: 1.5, ml: 2.5, p: 4, expected: 2.6, stdDev: 0.42 },
  { id: '2.11.3', task: 'Desktop optimization', o: 1, ml: 2, p: 3, expected: 2.0, stdDev: 0.33 },
  { id: '2.11.4', task: 'Loading skeletons for table', o: 0.75, ml: 1.5, p: 2.5, expected: 1.5, stdDev: 0.29 },
  { id: '2.11.5', task: 'Animations & transitions', o: 0.15, ml: 0.25, p: 0.5, expected: 0.3, stdDev: 0.06 },
];

export const screen2QA = [
  { category: 'Test Planning', task: 'Test strategy document', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  { category: 'Test Planning', task: 'Test case design - Data table', o: 3, ml: 5, p: 8, expected: 5.2, stdDev: 0.83 },
  { category: 'Test Planning', task: 'Test case design - Column customization', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { category: 'Test Planning', task: 'Test case design - Filters', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { category: 'Test Planning', task: 'Test case design - Search', o: 1, ml: 2, p: 4, expected: 2.2, stdDev: 0.50 },
  { category: 'Test Planning', task: 'Test case design - Drag & drop', o: 1.5, ml: 2.5, p: 4, expected: 2.6, stdDev: 0.42 },
  { category: 'Test Planning', task: 'Test data preparation', o: 1.5, ml: 2, p: 4, expected: 2.3, stdDev: 0.42 },
  { category: 'Test Planning', subtotal: true, expected: 19.8 },
  
  { category: 'Test Execution', task: 'Functional testing - Table display & sorting', o: 4, ml: 6, p: 10, expected: 6.3, stdDev: 1.00 },
  { category: 'Test Execution', task: 'Functional testing - Column customization', o: 3, ml: 5, p: 8, expected: 5.2, stdDev: 0.83 },
  { category: 'Test Execution', task: 'Functional testing - Drag & drop', o: 2, ml: 4, p: 6, expected: 4.0, stdDev: 0.67 },
  { category: 'Test Execution', task: 'Functional testing - Filters', o: 3, ml: 5, p: 8, expected: 5.2, stdDev: 0.83 },
  { category: 'Test Execution', task: 'Functional testing - Search', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { category: 'Test Execution', task: 'Functional testing - Tab switching', o: 1.5, ml: 2.5, p: 4, expected: 2.6, stdDev: 0.42 },
  { category: 'Test Execution', task: 'Functional testing - More options menu', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { category: 'Test Execution', task: 'UI/UX testing', o: 2, ml: 4, p: 6, expected: 4.0, stdDev: 0.67 },
  { category: 'Test Execution', task: 'Responsive testing (Mobile/Tablet/Desktop)', o: 4, ml: 6, p: 10, expected: 6.3, stdDev: 1.00 },
  { category: 'Test Execution', task: 'Cross-browser testing', o: 5, ml: 7, p: 12, expected: 7.5, stdDev: 1.17 },
  { category: 'Test Execution', task: 'API integration testing', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { category: 'Test Execution', task: 'Error handling testing', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { category: 'Test Execution', task: 'Performance testing (large datasets)', o: 2, ml: 4, p: 6, expected: 4.0, stdDev: 0.67 },
  { category: 'Test Execution', subtotal: true, expected: 57.9 },
  
  { category: 'Regression', task: 'Regression test cycle 1', o: 3, ml: 5, p: 8, expected: 5.2, stdDev: 0.83 },
  { category: 'Regression', task: 'Regression test cycle 2', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { category: 'Regression', task: 'Bug retesting', o: 3, ml: 5, p: 8, expected: 5.2, stdDev: 0.83 },
  { category: 'Regression', subtotal: true, expected: 13.6 },
  
  { category: 'Automation', task: 'Unit tests - Table components', o: 3, ml: 5, p: 8, expected: 5.2, stdDev: 0.83 },
  { category: 'Automation', task: 'Unit tests - Modal components', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { category: 'Automation', task: 'Unit tests - Utilities/Hooks', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { category: 'Automation', task: 'Integration tests', o: 4, ml: 6, p: 10, expected: 6.3, stdDev: 1.00 },
  { category: 'Automation', task: 'E2E tests - Critical user flows', o: 5, ml: 8, p: 12, expected: 8.2, stdDev: 1.17 },
  { category: 'Automation', task: 'E2E tests - Drag & drop scenarios', o: 2, ml: 4, p: 6, expected: 4.0, stdDev: 0.67 },
  { category: 'Automation', subtotal: true, expected: 30.1 },
];

export const screen2Hardening = [
  { category: 'Performance', task: 'Table virtualization for large datasets', o: 3, ml: 5, p: 8, expected: 5.2, stdDev: 0.83 },
  { category: 'Performance', task: 'Optimize drag & drop performance', o: 1.5, ml: 2.5, p: 4, expected: 2.6, stdDev: 0.42 },
  { category: 'Performance', task: 'Memoization of table cells', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { category: 'Performance', task: 'Debouncing/throttling optimization', o: 1, ml: 1.5, p: 3, expected: 1.7, stdDev: 0.33 },
  { category: 'Performance', task: 'Performance audit & validation', o: 1, ml: 2, p: 3, expected: 2.0, stdDev: 0.33 },
  { category: 'Performance', subtotal: true, expected: 14.7 },
  
  { category: 'Security', task: 'Input validation for search/filters', o: 1, ml: 2, p: 3, expected: 2.0, stdDev: 0.33 },
  { category: 'Security', task: 'XSS prevention in table cells', o: 1, ml: 1.5, p: 3, expected: 1.7, stdDev: 0.33 },
  { category: 'Security', task: 'Secure column preferences storage', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  { category: 'Security', subtotal: true, expected: 4.8 },
  
  { category: 'Accessibility', task: 'Table keyboard navigation', o: 3, ml: 5, p: 8, expected: 5.2, stdDev: 0.83 },
  { category: 'Accessibility', task: 'Drag & drop keyboard alternative', o: 2, ml: 4, p: 6, expected: 4.0, stdDev: 0.67 },
  { category: 'Accessibility', task: 'ARIA labels for table', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { category: 'Accessibility', task: 'Screen reader testing', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { category: 'Accessibility', task: 'Focus management in modals', o: 1, ml: 2, p: 3, expected: 2.0, stdDev: 0.33 },
  { category: 'Accessibility', subtotal: true, expected: 17.6 },
  
  { category: 'Code Quality', task: 'Code review & refactoring', o: 3, ml: 5, p: 8, expected: 5.2, stdDev: 0.83 },
  { category: 'Code Quality', task: 'TypeScript strict mode compliance', o: 1.5, ml: 2.5, p: 4, expected: 2.6, stdDev: 0.42 },
  { category: 'Code Quality', task: 'Code documentation', o: 2, ml: 3, p: 5, expected: 3.2, stdDev: 0.50 },
  { category: 'Code Quality', task: 'Error boundaries', o: 0.5, ml: 1, p: 2, expected: 1.1, stdDev: 0.25 },
  { category: 'Code Quality', subtotal: true, expected: 12.1 },
];
