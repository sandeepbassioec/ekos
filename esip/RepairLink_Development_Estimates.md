# RepairLink Application - Development Estimates
## Wide Band Delphi + PERT Analysis

**Project:** RepairLink Dashboard - React TypeScript Application  
**Estimation Method:** Wide Band Delphi with PERT Three-Point Estimation  
**Component Library:** OEC OUI Components & Theme Library  
**Date:** March 10, 2026  
**Version:** 1.0 (Screen 1 - Overview Page)

---

## Executive Summary

This document provides detailed development, QA, and hardening estimates for the RepairLink Overview page using the Wide Band Delphi estimation method combined with PERT (Program Evaluation and Review Technique) formula.

**PERT Formula:**  
- **Expected Time (E)** = (Optimistic + 4 × Most Likely + Pessimistic) / 6
- **Standard Deviation (σ)** = (Pessimistic - Optimistic) / 6

---

## 1. SCREEN 1: OVERVIEW PAGE

### 1.1 Work Breakdown Structure (WBS)

```
RepairLink Overview Page
│
├── 1.1 Foundation & Setup
│   ├── 1.1.1 Project scaffolding with TypeScript
│   ├── 1.1.2 OEC OUI library integration
│   ├── 1.1.3 Routing setup (React Router)
│   └── 1.1.4 Base layout structure (Header, Sidebar, Content)
│
├── 1.2 Header Component
│   ├── 1.2.1 Global header with navigation tabs
│   ├── 1.2.2 CollisionLink/RepairLink counters (badge components)
│   └── 1.2.3 Search bar with dropdown (OEM selector)
│
├── 1.3 Sidebar Navigation
│   ├── 1.3.1 RepairLink section header
│   ├── 1.3.2 Collapsible menu items (RepairLink Tools, Setup)
│   ├── 1.3.3 Menu items (Program Settlement, View Archive, etc.)
│   └── 1.3.4 Active state management
│
├── 1.4 Overview Tab Navigation
│   ├── 1.4.1 Tab switcher (Overview/Transaction)
│   ├── 1.4.2 Tab icons (home, transaction)
│   └── 1.4.3 Active tab state
│
├── 1.5 Search & Filter Bar
│   ├── 1.5.1 Filter button with icon
│   ├── 1.5.2 Search input with icon
│   ├── 1.5.3 Clear search functionality
│   └── 1.5.4 Search state management
│
├── 1.6 Activity Section
│   ├── 1.6.1 Activity section header
│   ├── 1.6.2 New Orders card component
│   ├── 1.6.3 Pending Orders card component
│   ├── 1.6.4 Trading Partners Requests card component
│   ├── 1.6.5 Manage button click handlers
│   └── 1.6.6 Activity cards grid layout
│
├── 1.7 Analytics Section
│   ├── 1.7.1 Analytics section header
│   ├── 1.7.2 Metric card component (reusable)
│   ├── 1.7.3 12 Metric cards implementation:
│   │   ├── Avg. Response Time
│   │   ├── Response Rate Percentile
│   │   ├── Dealer Rank
│   │   ├── Orders Received
│   │   ├── Orders Processed
│   │   ├── Part Qty. Sold
│   │   ├── Total Sales Opportunity
│   │   ├── Dollars Sold
│   │   ├── Unsold Opportunity
│   │   ├── Total PMA Reimbursement
│   │   ├── PMA Parts in Stock
│   │   ├── PMA Parts Not Stocked
│   │   ├── Total Trading Partners
│   │   ├── Active Trading Partners
│   │   └── Inactive Trading Partners
│   └── 1.7.4 Analytics grid layout (responsive 3-column)
│
├── 1.8 Filter Modal
│   ├── 1.8.1 Modal component (OEC OUI)
│   ├── 1.8.2 Date Range section
│   │   ├── Start Date picker
│   │   └── End Date picker
│   ├── 1.8.3 OEM Filter section
│   │   ├── Checkbox group
│   │   ├── Lexus checkbox with counter (2)
│   │   ├── Toyota checkbox with counter (2)
│   │   └── Mazda checkbox with counter (2)
│   ├── 1.8.4 Salesperson Filter section
│   │   ├── Checkbox group
│   │   ├── Lexus checkbox with counter (2)
│   │   ├── Toyota checkbox with counter (2)
│   │   └── Mazda checkbox with counter (2)
│   ├── 1.8.5 Modal footer (Apply/Cancel buttons)
│   └── 1.8.6 Filter state management
│
├── 1.9 Search Functionality
│   ├── 1.9.1 Search algorithm implementation
│   ├── 1.9.2 Filter cards based on search query
│   ├── 1.9.3 Highlight matched results (optional)
│   └── 1.9.4 Empty state when no results
│
├── 1.10 Filter Logic
│   ├── 1.10.1 Date range filtering
│   ├── 1.10.2 OEM filtering
│   ├── 1.10.3 Salesperson filtering
│   ├── 1.10.4 Combined filter logic
│   └── 1.10.5 Filter persistence (session storage)
│
├── 1.11 API Integration
│   ├── 1.11.1 API client setup (axios/fetch)
│   ├── 1.11.2 Activity data endpoint integration
│   ├── 1.11.3 Analytics data endpoint integration
│   ├── 1.11.4 Filter options endpoint integration
│   ├── 1.11.5 Error handling
│   ├── 1.11.6 Loading states
│   └── 1.11.7 Data transformation/mapping
│
├── 1.12 State Management
│   ├── 1.12.1 Setup state management (Context/Redux)
│   ├── 1.12.2 Activity state
│   ├── 1.12.3 Analytics state
│   ├── 1.12.4 Filter state
│   ├── 1.12.5 Search state
│   └── 1.12.6 UI state (loading, errors)
│
├── 1.13 TypeScript Types & Interfaces
│   ├── 1.13.1 Activity card types
│   ├── 1.13.2 Analytics metric types
│   ├── 1.13.3 Filter option types
│   ├── 1.13.4 API response types
│   └── 1.13.5 Component prop types
│
└── 1.14 Responsive Design & Polish
    ├── 1.14.1 Mobile responsive layout
    ├── 1.14.2 Tablet responsive layout
    ├── 1.14.3 Desktop optimization
    ├── 1.14.4 Loading skeletons
    └── 1.14.5 Animations & transitions
```

---

## 2. DEVELOPMENT ESTIMATES (Screen 1)

### 2.1 Frontend Development

| WBS ID | Task | Optimistic (hrs) | Most Likely (hrs) | Pessimistic (hrs) | Expected (hrs) | Std Dev | Notes |
|--------|------|------------------|-------------------|-------------------|----------------|---------|-------|
| **1.1** | **Foundation & Setup** | | | | **6.0** | **0.83** | |
| 1.1.1 | Project scaffolding with TypeScript | 1 | 2 | 4 | 2.2 | 0.50 | CRA/Vite + TS config |
| 1.1.2 | OEC OUI library integration | 0.5 | 1 | 2 | 1.1 | 0.25 | NPM install + config |
| 1.1.3 | Routing setup (React Router) | 0.5 | 1 | 2 | 1.1 | 0.25 | Basic routes |
| 1.1.4 | Base layout structure | 1 | 1.5 | 3 | 1.7 | 0.33 | Header/Sidebar/Content |
| **1.2** | **Header Component** | | | | **4.5** | **0.67** | |
| 1.2.1 | Global header with navigation tabs | 1.5 | 2.5 | 4 | 2.6 | 0.42 | 11+ navigation items |
| 1.2.2 | CollisionLink/RepairLink counters | 0.5 | 1 | 2 | 1.1 | 0.25 | Badge components (OUI) |
| 1.2.3 | Search bar with dropdown | 0.5 | 0.75 | 1.5 | 0.8 | 0.17 | OEM selector dropdown |
| **1.3** | **Sidebar Navigation** | | | | **5.3** | **0.75** | |
| 1.3.1 | RepairLink section header | 0.25 | 0.5 | 1 | 0.5 | 0.13 | Simple header |
| 1.3.2 | Collapsible menu items | 1.5 | 2.5 | 4 | 2.6 | 0.42 | Dropdown icons + state |
| 1.3.3 | Menu items (7 items) | 1 | 2 | 3 | 2.0 | 0.33 | Link components |
| 1.3.4 | Active state management | 0.15 | 0.25 | 0.5 | 0.3 | 0.06 | Active styling |
| **1.4** | **Overview Tab Navigation** | | | | **1.8** | **0.25** | |
| 1.4.1 | Tab switcher | 0.5 | 1 | 1.5 | 1.0 | 0.17 | 2 tabs with OUI |
| 1.4.2 | Tab icons | 0.25 | 0.5 | 1 | 0.5 | 0.13 | Icon integration |
| 1.4.3 | Active tab state | 0.15 | 0.25 | 0.5 | 0.3 | 0.06 | State management |
| **1.5** | **Search & Filter Bar** | | | | **3.3** | **0.50** | |
| 1.5.1 | Filter button with icon | 0.25 | 0.5 | 1 | 0.5 | 0.13 | OUI button |
| 1.5.2 | Search input with icon | 0.5 | 1 | 2 | 1.1 | 0.25 | OUI input field |
| 1.5.3 | Clear search functionality | 0.5 | 1 | 1.5 | 1.0 | 0.17 | Clear button logic |
| 1.5.4 | Search state management | 0.5 | 0.75 | 1.5 | 0.8 | 0.17 | useState/useReducer |
| **1.6** | **Activity Section** | | | | **5.8** | **0.67** | |
| 1.6.1 | Activity section header | 0.25 | 0.5 | 1 | 0.5 | 0.13 | Title component |
| 1.6.2 | New Orders card | 0.75 | 1 | 2 | 1.1 | 0.21 | Card with color (green) |
| 1.6.3 | Pending Orders card | 0.75 | 1 | 2 | 1.1 | 0.21 | Card with color (orange) |
| 1.6.4 | Trading Partners card | 0.75 | 1 | 2 | 1.1 | 0.21 | Card with color (blue) |
| 1.6.5 | Manage button handlers | 0.5 | 1 | 1.5 | 1.0 | 0.17 | Navigation logic |
| 1.6.6 | Grid layout | 0.5 | 1 | 1.5 | 1.0 | 0.17 | Responsive 3-column |
| **1.7** | **Analytics Section** | | | | **10.3** | **1.17** | |
| 1.7.1 | Analytics section header | 0.25 | 0.5 | 1 | 0.5 | 0.13 | Title component |
| 1.7.2 | Reusable metric card component | 1 | 2 | 4 | 2.2 | 0.50 | Label + Value display |
| 1.7.3 | 12 Metric cards implementation | 3 | 5 | 8 | 5.2 | 0.83 | Data mapping + display |
| 1.7.4 | Analytics grid layout | 1.5 | 2.5 | 4 | 2.6 | 0.42 | Responsive 3-column grid |
| **1.8** | **Filter Modal** | | | | **10.5** | **1.33** | |
| 1.8.1 | Modal component (OEC OUI) | 0.5 | 1 | 2 | 1.1 | 0.25 | Modal wrapper |
| 1.8.2 | Date Range section | 2 | 3 | 5 | 3.2 | 0.50 | 2 date pickers |
| 1.8.3 | OEM Filter section | 1.5 | 2 | 4 | 2.3 | 0.42 | 3 checkboxes + counters |
| 1.8.4 | Salesperson Filter section | 1.5 | 2 | 4 | 2.3 | 0.42 | 3 checkboxes + counters |
| 1.8.5 | Modal footer (Apply/Cancel) | 0.5 | 0.75 | 1.5 | 0.8 | 0.17 | Button actions |
| 1.8.6 | Filter state management | 0.5 | 1 | 2 | 1.1 | 0.25 | Complex state logic |
| **1.9** | **Search Functionality** | | | | **5.3** | **0.75** | |
| 1.9.1 | Search algorithm | 1 | 2 | 3 | 2.0 | 0.33 | Filter logic |
| 1.9.2 | Filter cards based on search | 1 | 2 | 4 | 2.2 | 0.50 | Apply filters to data |
| 1.9.3 | Highlight matched results | 0.5 | 0.75 | 1.5 | 0.8 | 0.17 | Optional feature |
| 1.9.4 | Empty state | 0.15 | 0.25 | 0.5 | 0.3 | 0.06 | No results message |
| **1.10** | **Filter Logic** | | | | **7.3** | **1.00** | |
| 1.10.1 | Date range filtering | 1 | 2 | 4 | 2.2 | 0.50 | Date comparison logic |
| 1.10.2 | OEM filtering | 0.75 | 1.5 | 3 | 1.6 | 0.38 | Multi-select filter |
| 1.10.3 | Salesperson filtering | 0.75 | 1.5 | 3 | 1.6 | 0.38 | Multi-select filter |
| 1.10.4 | Combined filter logic | 1 | 1.5 | 3 | 1.7 | 0.33 | AND/OR conditions |
| 1.10.5 | Filter persistence | 0.15 | 0.25 | 0.5 | 0.3 | 0.06 | Session storage |
| **1.11** | **API Integration** | | | | **10.7** | **1.33** | |
| 1.11.1 | API client setup | 0.5 | 1 | 2 | 1.1 | 0.25 | Axios/Fetch wrapper |
| 1.11.2 | Activity data endpoint | 1 | 1.5 | 3 | 1.7 | 0.33 | GET /activity |
| 1.11.3 | Analytics data endpoint | 1 | 2 | 4 | 2.2 | 0.50 | GET /analytics |
| 1.11.4 | Filter options endpoint | 1 | 1.5 | 3 | 1.7 | 0.33 | GET /filter-options |
| 1.11.5 | Error handling | 1 | 1.5 | 3 | 1.7 | 0.33 | Try-catch + error UI |
| 1.11.6 | Loading states | 0.75 | 1.5 | 2.5 | 1.5 | 0.29 | Skeletons/spinners |
| 1.11.7 | Data transformation | 0.5 | 1 | 2 | 1.1 | 0.25 | Map API to UI format |
| **1.12** | **State Management** | | | | **6.5** | **0.83** | |
| 1.12.1 | Setup state management | 1 | 2 | 4 | 2.2 | 0.50 | Context/Redux config |
| 1.12.2 | Activity state | 0.5 | 0.75 | 1.5 | 0.8 | 0.17 | Actions + reducers |
| 1.12.3 | Analytics state | 0.5 | 0.75 | 1.5 | 0.8 | 0.17 | Actions + reducers |
| 1.12.4 | Filter state | 0.75 | 1.5 | 2.5 | 1.5 | 0.29 | Complex state |
| 1.12.5 | Search state | 0.25 | 0.5 | 1 | 0.5 | 0.13 | Simple state |
| 1.12.6 | UI state (loading, errors) | 0.5 | 0.75 | 1.5 | 0.8 | 0.17 | Boolean flags |
| **1.13** | **TypeScript Types** | | | | **4.5** | **0.67** | |
| 1.13.1 | Activity card types | 0.5 | 0.75 | 1.5 | 0.8 | 0.17 | Interfaces |
| 1.13.2 | Analytics metric types | 0.75 | 1.5 | 2.5 | 1.5 | 0.29 | Multiple types |
| 1.13.3 | Filter option types | 0.5 | 1 | 2 | 1.1 | 0.25 | Enums + interfaces |
| 1.13.4 | API response types | 0.5 | 0.75 | 1.5 | 0.8 | 0.17 | Response schemas |
| 1.13.5 | Component prop types | 0.15 | 0.25 | 0.5 | 0.3 | 0.06 | Props interfaces |
| **1.14** | **Responsive & Polish** | | | | **8.3** | **1.00** | |
| 1.14.1 | Mobile responsive layout | 2 | 3 | 5 | 3.2 | 0.50 | Breakpoints + testing |
| 1.14.2 | Tablet responsive layout | 1.5 | 2 | 4 | 2.2 | 0.42 | Medium breakpoints |
| 1.14.3 | Desktop optimization | 1 | 1.5 | 3 | 1.7 | 0.33 | Large screens |
| 1.14.4 | Loading skeletons | 0.5 | 0.75 | 1.5 | 0.8 | 0.17 | Placeholder UI |
| 1.14.5 | Animations & transitions | 0.25 | 0.5 | 1 | 0.5 | 0.13 | Smooth UX |

**Total Frontend Development:** **90 hours** (Rounded from 89.7 hours)

---

## 3. QA ESTIMATES (Screen 1)

### 3.1 Test Planning & Design

| Task | Optimistic (hrs) | Most Likely (hrs) | Pessimistic (hrs) | Expected (hrs) | Std Dev |
|------|------------------|-------------------|-------------------|----------------|---------|
| Test strategy document | 1 | 2 | 3 | 2.0 | 0.33 |
| Test case design - UI components | 2 | 4 | 6 | 4.0 | 0.67 |
| Test case design - Search functionality | 1 | 2 | 4 | 2.2 | 0.50 |
| Test case design - Filter functionality | 2 | 3 | 5 | 3.2 | 0.50 |
| Test case design - API integration | 1.5 | 2.5 | 4 | 2.6 | 0.42 |
| Test case design - Responsive design | 1 | 2 | 3 | 2.0 | 0.33 |
| Test data preparation | 1 | 1.5 | 3 | 1.7 | 0.33 |
| **Subtotal** | | | | **17.7** | |

### 3.2 Test Execution

| Task | Optimistic (hrs) | Most Likely (hrs) | Pessimistic (hrs) | Expected (hrs) | Std Dev |
|------|------------------|-------------------|-------------------|----------------|---------|
| Functional testing - Activity cards | 2 | 3 | 5 | 3.2 | 0.50 |
| Functional testing - Analytics cards | 3 | 5 | 8 | 5.2 | 0.83 |
| Functional testing - Search | 2 | 4 | 6 | 4.0 | 0.67 |
| Functional testing - Filters | 3 | 5 | 8 | 5.2 | 0.83 |
| Functional testing - Navigation | 1.5 | 2.5 | 4 | 2.6 | 0.42 |
| UI/UX testing | 2 | 3 | 5 | 3.2 | 0.50 |
| Responsive testing (Mobile/Tablet/Desktop) | 3 | 5 | 7 | 5.0 | 0.67 |
| Cross-browser testing (Chrome, Firefox, Safari, Edge) | 4 | 6 | 10 | 6.3 | 1.00 |
| API integration testing | 2 | 3 | 5 | 3.2 | 0.50 |
| Error handling testing | 1.5 | 2 | 4 | 2.2 | 0.42 |
| Loading states testing | 1 | 1.5 | 3 | 1.7 | 0.33 |
| **Subtotal** | | | | **41.8** | |

### 3.3 Regression Testing

| Task | Optimistic (hrs) | Most Likely (hrs) | Pessimistic (hrs) | Expected (hrs) | Std Dev |
|------|------------------|-------------------|-------------------|----------------|---------|
| Regression test cycle 1 | 3 | 5 | 8 | 5.2 | 0.83 |
| Regression test cycle 2 | 2 | 3 | 5 | 3.2 | 0.50 |
| Bug retesting | 2 | 4 | 6 | 4.0 | 0.67 |
| **Subtotal** | | | | **12.4** | |

### 3.4 Automated Testing (Optional but Recommended)

| Task | Optimistic (hrs) | Most Likely (hrs) | Pessimistic (hrs) | Expected (hrs) | Std Dev |
|------|------------------|-------------------|-------------------|----------------|---------|
| Unit test setup (Jest/Vitest) | 1 | 2 | 3 | 2.0 | 0.33 |
| Unit tests - Components | 4 | 6 | 10 | 6.3 | 1.00 |
| Unit tests - Utilities/Hooks | 2 | 3 | 5 | 3.2 | 0.50 |
| Integration tests (React Testing Library) | 3 | 5 | 8 | 5.2 | 0.83 |
| E2E tests setup (Playwright/Cypress) | 2 | 3 | 5 | 3.2 | 0.50 |
| E2E tests - Critical user flows | 4 | 6 | 10 | 6.3 | 1.00 |
| **Subtotal** | | | | **26.2** | |

**Total QA (Manual):** **72 hours** (Rounded from 71.9 hours)  
**Total QA (with Automation):** **98 hours** (Rounded from 98.1 hours)

---

## 4. HARDENING ESTIMATES (Screen 1)

### 4.1 Performance Optimization

| Task | Optimistic (hrs) | Most Likely (hrs) | Pessimistic (hrs) | Expected (hrs) | Std Dev |
|------|------------------|-------------------|-------------------|----------------|---------|
| Performance audit (Lighthouse) | 0.5 | 1 | 2 | 1.1 | 0.25 |
| Code splitting & lazy loading | 2 | 3 | 5 | 3.2 | 0.50 |
| Optimize bundle size | 1.5 | 2.5 | 4 | 2.6 | 0.42 |
| Memoization (React.memo, useMemo, useCallback) | 1.5 | 2.5 | 4 | 2.6 | 0.42 |
| Image optimization | 0.5 | 1 | 2 | 1.1 | 0.25 |
| API response caching | 1 | 2 | 3 | 2.0 | 0.33 |
| Debouncing search input | 0.5 | 0.75 | 1.5 | 0.8 | 0.17 |
| Virtual scrolling (if needed for large datasets) | 2 | 4 | 6 | 4.0 | 0.67 |
| Performance testing & validation | 1 | 2 | 3 | 2.0 | 0.33 |
| **Subtotal** | | | | **19.4** | |

### 4.2 Security

| Task | Optimistic (hrs) | Most Likely (hrs) | Pessimistic (hrs) | Expected (hrs) | Std Dev |
|------|------------------|-------------------|-------------------|----------------|---------|
| Security audit | 1 | 2 | 3 | 2.0 | 0.33 |
| Input validation & sanitization | 1.5 | 2 | 4 | 2.3 | 0.42 |
| XSS prevention | 1 | 1.5 | 3 | 1.7 | 0.33 |
| CSRF protection (if applicable) | 0.5 | 1 | 2 | 1.1 | 0.25 |
| Secure API communication (HTTPS, headers) | 0.5 | 1 | 2 | 1.1 | 0.25 |
| Authentication/Authorization checks | 1 | 2 | 4 | 2.2 | 0.50 |
| Environment variables security | 0.25 | 0.5 | 1 | 0.5 | 0.13 |
| Dependency vulnerability scan | 0.5 | 1 | 2 | 1.1 | 0.25 |
| **Subtotal** | | | | **12.0** | |

### 4.3 Accessibility (WCAG 2.1 AA Compliance)

| Task | Optimistic (hrs) | Most Likely (hrs) | Pessimistic (hrs) | Expected (hrs) | Std Dev |
|------|------------------|-------------------|-------------------|----------------|---------|
| Accessibility audit (axe DevTools) | 1 | 1.5 | 3 | 1.7 | 0.33 |
| Semantic HTML fixes | 1 | 2 | 3 | 2.0 | 0.33 |
| ARIA labels & roles | 2 | 3 | 5 | 3.2 | 0.50 |
| Keyboard navigation | 2 | 3 | 5 | 3.2 | 0.50 |
| Focus management | 1.5 | 2 | 4 | 2.3 | 0.42 |
| Color contrast fixes | 1 | 1.5 | 3 | 1.7 | 0.33 |
| Screen reader testing | 2 | 3 | 5 | 3.2 | 0.50 |
| Accessibility documentation | 0.5 | 1 | 2 | 1.1 | 0.25 |
| **Subtotal** | | | | **18.4** | |

### 4.4 Code Quality & Maintainability

| Task | Optimistic (hrs) | Most Likely (hrs) | Pessimistic (hrs) | Expected (hrs) | Std Dev |
|------|------------------|-------------------|-------------------|----------------|---------|
| Code review & refactoring | 3 | 5 | 8 | 5.2 | 0.83 |
| ESLint/Prettier setup & fixes | 1 | 1.5 | 3 | 1.7 | 0.33 |
| TypeScript strict mode compliance | 1.5 | 2.5 | 4 | 2.6 | 0.42 |
| Code documentation (JSDoc/TSDoc) | 2 | 3 | 5 | 3.2 | 0.50 |
| Storybook setup (component documentation) | 2 | 3 | 5 | 3.2 | 0.50 |
| Error boundary implementation | 0.5 | 1 | 2 | 1.1 | 0.25 |
| Logging & monitoring setup | 1 | 2 | 3 | 2.0 | 0.33 |
| **Subtotal** | | | | **19.0** | |

### 4.5 Production Readiness

| Task | Optimistic (hrs) | Most Likely (hrs) | Pessimistic (hrs) | Expected (hrs) | Std Dev |
|------|------------------|-------------------|-------------------|----------------|---------|
| Build optimization | 1 | 2 | 3 | 2.0 | 0.33 |
| Environment configuration (dev/staging/prod) | 1 | 1.5 | 3 | 1.7 | 0.33 |
| Error tracking setup (Sentry/LogRocket) | 1 | 2 | 3 | 2.0 | 0.33 |
| Analytics integration (Google Analytics/Mixpanel) | 1 | 1.5 | 3 | 1.7 | 0.33 |
| SEO optimization (meta tags, sitemap) | 0.5 | 1 | 2 | 1.1 | 0.25 |
| PWA setup (if required) | 2 | 4 | 6 | 4.0 | 0.67 |
| Deployment pipeline testing | 1 | 2 | 3 | 2.0 | 0.33 |
| Production smoke testing | 1 | 1.5 | 3 | 1.7 | 0.33 |
| **Subtotal** | | | | **16.2** | |

**Total Hardening:** **85 hours** (Rounded from 85.0 hours)

---

## 5. SUMMARY ESTIMATES - SCREEN 1 (OVERVIEW PAGE)

### 5.1 By Phase

| Phase | Estimated Hours | Estimated Days (8h/day) | Standard Deviation |
|-------|----------------|------------------------|-------------------|
| **Development** | 90 | 11.3 | ±4.2 |
| **QA (Manual)** | 72 | 9.0 | ±3.8 |
| **QA (with Automation)** | 98 | 12.3 | ±4.5 |
| **Hardening** | 85 | 10.6 | ±3.6 |
| **TOTAL (Manual QA)** | **247** | **30.9** | ±11.6 |
| **TOTAL (Automated QA)** | **273** | **34.1** | ±12.3 |

### 5.2 By Resource Role

| Role | Estimated Hours | % of Total |
|------|----------------|-----------|
| Frontend Developer (Senior) | 60 | 24% |
| Frontend Developer (Mid-level) | 30 | 12% |
| QA Engineer (Manual) | 72 | 29% |
| QA Automation Engineer | 26 | 11% |
| DevOps/Performance Engineer | 20 | 8% |
| Security Engineer | 12 | 5% |
| Accessibility Specialist | 18 | 7% |
| Code Reviewer/Tech Lead | 10 | 4% |
| **TOTAL** | **248** | **100%** |

### 5.3 Confidence Levels

| Estimate Range | Probability | Hours | Days |
|----------------|-------------|-------|------|
| **Best Case (Optimistic)** | 1% | 180 | 22.5 |
| **Expected Case** | 50% | 247 | 30.9 |
| **Conservative (Expected + 1σ)** | 84% | 259 | 32.4 |
| **Worst Case (Pessimistic)** | 99% | 315 | 39.4 |

---

## 6. ASSUMPTIONS

1. **OEC OUI Library:** Pre-built components (buttons, inputs, cards, modals, date pickers) are available and well-documented, reducing development time by ~40%
2. **Design Handoff:** Figma designs are complete and approved with no major changes expected
3. **API Availability:** Backend APIs are available for integration testing (or mock APIs are provided)
4. **Team Experience:** Team has experience with React, TypeScript, and OEC OUI library
5. **No Server-Side Work:** Estimates exclude backend/server-side development as requested
6. **Browser Support:** Modern browsers (Chrome, Firefox, Safari, Edge - latest 2 versions)
7. **Responsive Design:** Mobile, Tablet, Desktop (no smartwatch or TV support)
8. **Single Language:** English only (no i18n/localization)
9. **Standard Work Day:** 8 hours/day, 5 days/week
10. **No Major Blockers:** No critical dependencies or third-party integration delays

---

## 7. RISKS & MITIGATION

| Risk | Impact | Probability | Mitigation Strategy |
|------|--------|-------------|---------------------|
| OEC OUI library components don't meet design requirements | High | Medium | Allocate 20% buffer for custom component development |
| API integration issues or delays | High | Medium | Use mock data early, parallel development with backend |
| Complex filter logic edge cases | Medium | High | Early prototype and review with stakeholders |
| Browser compatibility issues | Medium | Low | Test early and often across browsers |
| Performance issues with large datasets | High | Medium | Implement pagination/virtual scrolling from start |
| Accessibility compliance gaps | Medium | Medium | Conduct early accessibility audit and iterate |
| Scope creep from additional screens | High | High | Formal change request process, update estimates |

---

## 8. NEXT STEPS

1. **Review & Approval:** Stakeholder review of estimates for Screen 1
2. **Additional Screens:** Await next screen specifications to update estimates
3. **Team Assignment:** Allocate developers, QA, and hardening specialists
4. **Sprint Planning:** Break down into 2-week sprints
5. **Kickoff Meeting:** Align team on technical approach and timeline

---

## 9. CHANGE LOG

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| 1.0 | March 10, 2026 | Initial estimates for Screen 1 (Overview Page) | Estimation Team |

---

**Note:** This document will be updated as additional screens and requirements are provided. All estimates follow the Wide Band Delphi method with PERT three-point estimation for maximum accuracy.
