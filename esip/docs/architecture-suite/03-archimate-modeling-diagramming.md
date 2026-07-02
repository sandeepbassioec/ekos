# Volume 03 — ArchiMate 3.2 Metamodel + Modeling & Diagramming Engine

**Prefix:** MOD | **Path:** `esip/docs/architecture-suite/03-archimate-modeling-diagramming.md`
**Related Volumes:** 01 TOGAF · 02 Zachman/DoDAF/MODAF/UAF · 04 SA · 05 Role-Based · 06 AI Generation Engine · 07 Requirements · 08 KH · 09 Portfolio · 10 Deliverables/Export · 11 Platform · 18 Interop/Import

> **Build phase:** The **interactive maxGraph canvas is Phase 3** per `BUILD-PLAN-MVP-AND-PHASING.md`. The **MVP uses Mermaid + AI-generated diagrams as the bridge** until the full canvas lands. Per the master-index ruling (§6 ruling #1), `Must` means required in the product, not required in the MVP — priority is decoupled from build sequence.

---

## Table of Contents

1. [Capability Area: ArchiMate 3.2 Metamodel — Strategy Layer](#1-archimate-32-metamodel--strategy-layer)
2. [Capability Area: ArchiMate 3.2 Metamodel — Business Layer](#2-archimate-32-metamodel--business-layer)
3. [Capability Area: ArchiMate 3.2 Metamodel — Application Layer](#3-archimate-32-metamodel--application-layer)
4. [Capability Area: ArchiMate 3.2 Metamodel — Technology Layer](#4-archimate-32-metamodel--technology-layer)
5. [Capability Area: ArchiMate 3.2 Metamodel — Physical Layer](#5-archimate-32-metamodel--physical-layer)
6. [Capability Area: ArchiMate 3.2 Metamodel — Motivation Aspect](#6-archimate-32-metamodel--motivation-aspect)
7. [Capability Area: ArchiMate 3.2 Metamodel — Implementation & Migration Aspect](#7-archimate-32-metamodel--implementation--migration-aspect)
8. [Capability Area: ArchiMate 3.2 Relationships & Derivation Rules](#8-archimate-32-relationships--derivation-rules)
9. [Capability Area: ArchiMate 3.2 Viewpoints](#9-archimate-32-viewpoints)
10. [Capability Area: Model vs. View Separation](#10-model-vs-view-separation)
11. [Capability Area: Diagramming Engine — Canvas & Navigation](#11-diagramming-engine--canvas--navigation)
12. [Capability Area: Notation Support — UML](#12-notation-support--uml)
13. [Capability Area: Notation Support — BPMN 2.0](#13-notation-support--bpmn-20)
14. [Capability Area: Notation Support — C4 Model](#14-notation-support--c4-model)
15. [Capability Area: Notation Support — ERD, DFD, Flowcharts](#15-notation-support--erd-dfd-flowcharts)
16. [Capability Area: Notation Support — Infrastructure & Cloud](#16-notation-support--infrastructure--cloud)
17. [Capability Area: Notation Support — Additional Diagrams](#17-notation-support--additional-diagrams)
18. [Capability Area: Shape Libraries & Stencils](#18-shape-libraries--stencils)
19. [Capability Area: Connectors, Routing & Layout](#19-connectors-routing--layout)
20. [Capability Area: Themes, Styling & Conditional Formatting](#20-themes-styling--conditional-formatting)
21. [Capability Area: Layers, Swimlanes & Grouping](#21-layers-swimlanes--grouping)
22. [Capability Area: Model Intelligence](#22-model-intelligence)
23. [Capability Area: Collaboration & Lifecycle](#23-collaboration--lifecycle)
24. [Capability Area: Interoperability — Import](#24-interoperability--import)
25. [Capability Area: Interoperability — Export](#25-interoperability--export)
26. [Capability Area: AI Integration Hooks](#26-ai-integration-hooks)

---

## 1. ArchiMate 3.2 Metamodel — Strategy Layer

### 1.1 Strategy Elements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-001 | Resource Element | The system SHALL support the **Resource** element representing assets owned or controlled by an organization that are used to perform capabilities or value streams. Attributes: name, description, documentation. | Functional | Enterprise Architect | Must |
| REQ-MOD-002 | Capability Element | The system SHALL support the **Capability** element representing an ability that an active structure element possesses. Capabilities SHALL be organized hierarchically (capability maps). Attributes: name, description, documentation. | Functional | Enterprise Architect | Must |
| REQ-MOD-003 | Course of Action Element | The system SHALL support the **Course of Action** element representing an approach or plan for configuring a capability or resource. | Functional | Enterprise Architect | Must |
| REQ-MOD-004 | Value Stream Element | The system SHALL support the **Value Stream** element (ArchiMate 3.1+) representing a sequence of activities that create an overall result for a stakeholder. Value streams SHALL be decomposable into value stream stages. | Functional | Enterprise Architect | Must |

**Rationale:** Strategy layer elements are the foundation for capability-based planning aligned with TOGAF ADM Phase B and business motivation alignment. ArchiMate 3.2 spec §4.

**Acceptance Criteria:**
- All four strategy elements render with correct ArchiMate notation (icons, border styles per spec)
- Capability elements support at least 5 levels of hierarchy
- Value stream stages can be linked to business processes

---

## 2. ArchiMate 3.2 Metamodel — Business Layer

### 2.1 Active Structure Elements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-010 | Business Actor | The system SHALL support **Business Actor** representing a business entity capable of performing behavior. | Functional | EA, BA | Must |
| REQ-MOD-011 | Business Role | The system SHALL support **Business Role** representing the responsibility for performing specific behavior. Roles SHALL be assignable to actors. | Functional | EA, BA | Must |
| REQ-MOD-012 | Business Collaboration | The system SHALL support **Business Collaboration** representing an aggregate of two or more business roles cooperating to perform collective behavior. | Functional | EA | Must |
| REQ-MOD-013 | Business Interface | The system SHALL support **Business Interface** representing a point of access where business services are made available to the environment. | Functional | EA, BA | Must |

### 2.2 Behavior Elements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-014 | Business Process | The system SHALL support **Business Process** representing a sequence or flow of activities creating a specific outcome. Processes SHALL support decomposition (sub-processes). | Functional | EA, BA | Must |
| REQ-MOD-015 | Business Function | The system SHALL support **Business Function** representing a collection of business behavior based on a chosen set of criteria. | Functional | EA, BA | Must |
| REQ-MOD-016 | Business Interaction | The system SHALL support **Business Interaction** representing a behavior performed by a collaboration of two or more business roles. | Functional | EA | Must |
| REQ-MOD-017 | Business Event | The system SHALL support **Business Event** representing a business state change. Events SHALL be usable as triggers and results in process flows. | Functional | EA, BA | Must |
| REQ-MOD-018 | Business Service | The system SHALL support **Business Service** representing explicitly defined exposed business behavior. | Functional | EA, BA | Must |

### 2.3 Passive Structure Elements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-019 | Business Object | The system SHALL support **Business Object** representing a concept used within a particular business domain. | Functional | EA, BA | Must |
| REQ-MOD-020 | Contract | The system SHALL support **Contract** representing a formal or informal specification of an agreement between parties. | Functional | EA, BA | Must |
| REQ-MOD-021 | Representation | The system SHALL support **Representation** representing a perceptible form of information carried by a business object. | Functional | EA | Should |
| REQ-MOD-022 | Product | The system SHALL support **Product** representing a coherent collection of services and/or passive structure elements, accompanied by a contract, which is offered as a whole to customers. | Functional | EA, BA | Must |

**Acceptance Criteria:**
- All 13 Business layer elements render with correct ArchiMate 3.2 notation
- Business processes support swimlane rendering
- Business services can be linked to application services (cross-layer)

---

## 3. ArchiMate 3.2 Metamodel — Application Layer

### 3.1 Active Structure Elements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-030 | Application Component | The system SHALL support **Application Component** representing a modular, deployable, replaceable part of a software system. Components SHALL support internal/external distinction. | Functional | SA, EA | Must |
| REQ-MOD-031 | Application Collaboration | The system SHALL support **Application Collaboration** representing an aggregate of two or more application components cooperating to perform collective behavior. | Functional | SA | Must |
| REQ-MOD-032 | Application Interface | The system SHALL support **Application Interface** representing a point of access where application services are made available. | Functional | SA, EA | Must |

### 3.2 Behavior Elements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-033 | Application Function | The system SHALL support **Application Function** representing automated behavior performed by an application component. | Functional | SA | Must |
| REQ-MOD-034 | Application Interaction | The system SHALL support **Application Interaction** representing behavior performed by a collaboration of application components. | Functional | SA | Must |
| REQ-MOD-035 | Application Process | The system SHALL support **Application Process** representing a sequence of application behaviors achieving a specific result. | Functional | SA | Must |
| REQ-MOD-036 | Application Event | The system SHALL support **Application Event** representing an application state change. | Functional | SA | Must |
| REQ-MOD-037 | Application Service | The system SHALL support **Application Service** representing an explicitly defined exposed application behavior. | Functional | SA, EA | Must |

### 3.3 Passive Structure Elements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-038 | Data Object | The system SHALL support **Data Object** representing data structured for automated processing. Data objects SHALL be linkable to business objects via realization. | Functional | SA, DA | Must |

**Acceptance Criteria:**
- All 9 Application layer elements render correctly
- Application components support nesting (composite component pattern)
- Data objects link to business objects with realization relationship

---

## 4. ArchiMate 3.2 Metamodel — Technology Layer

### 4.1 Active Structure Elements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-040 | Node | The system SHALL support **Node** representing a computational or physical resource hosting, manipulating, or interacting with other resources. | Functional | TA, SA | Must |
| REQ-MOD-041 | Device | The system SHALL support **Device** representing a physical IT resource upon which system software and artifacts may be stored or deployed. | Functional | TA | Must |
| REQ-MOD-042 | System Software | The system SHALL support **System Software** representing software that provides or contributes to an environment for storing, executing, and using software. | Functional | TA | Must |
| REQ-MOD-043 | Technology Collaboration | The system SHALL support **Technology Collaboration** representing an aggregate of two or more technology elements cooperating to perform collective behavior. | Functional | TA | Must |
| REQ-MOD-044 | Technology Interface | The system SHALL support **Technology Interface** representing a point of access where technology services are made available. | Functional | TA | Must |
| REQ-MOD-045 | Path | The system SHALL support **Path** representing a link between two or more nodes through which data, energy, or materials can be transported. | Functional | TA | Must |
| REQ-MOD-046 | Communication Network | The system SHALL support **Communication Network** representing a set of structures that connects nodes for transmission, routing, and reception of data. | Functional | TA | Must |

### 4.2 Behavior Elements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-047 | Technology Function | The system SHALL support **Technology Function** representing a collection of technology behavior performed by a node. | Functional | TA | Must |
| REQ-MOD-048 | Technology Process | The system SHALL support **Technology Process** representing a sequence of technology behaviors. | Functional | TA | Must |
| REQ-MOD-049 | Technology Interaction | The system SHALL support **Technology Interaction** representing behavior performed by a collaboration of nodes. | Functional | TA | Must |
| REQ-MOD-050 | Technology Event | The system SHALL support **Technology Event** representing a technology state change. | Functional | TA | Must |
| REQ-MOD-051 | Technology Service | The system SHALL support **Technology Service** representing an explicitly defined exposed technology behavior. | Functional | TA, SA | Must |

### 4.3 Passive Structure Elements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-052 | Artifact | The system SHALL support **Artifact** representing a piece of data that is used or produced in a software development process or deployed in and operated by an IT system. | Functional | TA, SA | Must |

**Acceptance Criteria:**
- All 13 Technology layer elements render correctly per ArchiMate spec
- Deployment diagrams can show artifacts deployed on devices/nodes
- Communication networks render with correct topology

---

## 5. ArchiMate 3.2 Metamodel — Physical Layer

### 5.1 Physical Elements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-060 | Equipment | The system SHALL support **Equipment** representing one or more physical machines, tools, or instruments that can create, use, store, move, or transform materials. | Functional | EA, TA | Should |
| REQ-MOD-061 | Facility | The system SHALL support **Facility** representing a physical structure or environment. | Functional | EA | Should |
| REQ-MOD-062 | Distribution Network | The system SHALL support **Distribution Network** representing a physical network used to transport materials or energy. | Functional | EA | Should |
| REQ-MOD-063 | Material | The system SHALL support **Material** representing tangible physical matter or physical elements. | Functional | EA | Should |

**Rationale:** Physical layer supports IoT, manufacturing, and physical enterprise scenarios per ArchiMate 3.x spec §8.

**Acceptance Criteria:**
- Physical elements render with distinct notation from Technology layer
- Physical elements can be related to Technology layer via association/serving

---

## 6. ArchiMate 3.2 Metamodel — Motivation Aspect

### 6.1 Motivation Elements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-070 | Stakeholder | The system SHALL support **Stakeholder** representing the role of an individual, team, or organization that represents their interests in relation to the outcome of the architecture. | Functional | EA, BA | Must |
| REQ-MOD-071 | Driver | The system SHALL support **Driver** representing an external or internal condition that motivates an organization to change. | Functional | EA | Must |
| REQ-MOD-072 | Assessment | The system SHALL support **Assessment** representing the result of an analysis of the state of affairs of the enterprise with respect to a driver. Assessments SHALL include SWOT-type classification (strength/weakness/opportunity/threat). | Functional | EA | Must |
| REQ-MOD-073 | Goal | The system SHALL support **Goal** representing a high-level statement of intent, direction, or desired end state for an organization and its stakeholders. | Functional | EA, BA | Must |
| REQ-MOD-074 | Outcome | The system SHALL support **Outcome** representing an end result achieved by the enterprise. | Functional | EA | Must |
| REQ-MOD-075 | Principle | The system SHALL support **Principle** representing a qualitative statement of intent that should be met by the architecture. | Functional | EA | Must |
| REQ-MOD-076 | Requirement | The system SHALL support **Requirement** (motivation) representing a statement of need that must be met by the architecture. Requirements in this context are distinct from ESIP requirements management (V07) but SHALL support bi-directional linking. | Functional | EA, BA | Must |
| REQ-MOD-077 | Constraint | The system SHALL support **Constraint** representing a factor that limits or prevents the realization of goals. | Functional | EA | Must |
| REQ-MOD-078 | Meaning | The system SHALL support **Meaning** representing the knowledge or expertise present in, or the interpretation given to, a concept in a particular context. | Functional | EA | Should |
| REQ-MOD-079 | Value | The system SHALL support **Value** representing the relative worth, utility, or importance of a concept. | Functional | EA | Must |

**Acceptance Criteria:**
- All 10 motivation elements render with correct ArchiMate notation
- Stakeholder → Goal → Requirement → Constraint chains can be visualized
- Motivation elements link to core elements (e.g., driver influences goal, principle constrains architecture)
- REQ-MOD-076 links to Volume 07 requirement records via ID reference

---

## 7. ArchiMate 3.2 Metamodel — Implementation & Migration Aspect

### 7.1 Implementation & Migration Elements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-080 | Work Package | The system SHALL support **Work Package** representing a series of actions identified and designed to achieve specific results within a specified time. Work packages SHALL be linkable to TOGAF ADM phases. | Functional | EA, PM | Must |
| REQ-MOD-081 | Deliverable | The system SHALL support **Deliverable** representing a precisely defined result of a work package. Deliverables SHALL be linkable to Volume 10 export artifacts. | Functional | EA, PM | Must |
| REQ-MOD-082 | Implementation Event | The system SHALL support **Implementation Event** representing a state change related to implementation or migration that triggers work packages or occurs as a result. | Functional | EA | Must |
| REQ-MOD-083 | Plateau | The system SHALL support **Plateau** representing a relatively stable state of the architecture that exists during a limited period of time. Plateaus SHALL map to Architecture Roadmap states (V01). | Functional | EA | Must |
| REQ-MOD-084 | Gap | The system SHALL support **Gap** representing a statement of the difference between the current plateau and the desired plateau. Gap elements SHALL be auto-populatable from gap analysis results (V01/V09). | Functional | EA | Must |

**Acceptance Criteria:**
- Migration roadmap diagrams can be built using plateaus and gaps
- Work packages and deliverables render in Gantt-like timeline views
- Gap elements display difference descriptions and severity ratings

---

## 8. ArchiMate 3.2 Relationships & Derivation Rules

### 8.1 Structural Relationships

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-090 | Composition Relationship | The system SHALL support **Composition** — a whole-part relationship where the part cannot exist independently. Rendered as filled diamond on parent end. | Functional | EA | Must |
| REQ-MOD-091 | Aggregation Relationship | The system SHALL support **Aggregation** — a whole-part relationship where the part can exist independently. Rendered as open diamond on parent end. | Functional | EA | Must |
| REQ-MOD-092 | Assignment Relationship | The system SHALL support **Assignment** — links active structure elements to behavior/passive structure or roles to actors. Rendered as filled circle on structure end. | Functional | EA | Must |
| REQ-MOD-093 | Realization Relationship | The system SHALL support **Realization** — a causal relationship where one element plays a critical role in bringing another into existence. Rendered as dashed line with open arrowhead. | Functional | EA | Must |

### 8.2 Dependency Relationships

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-094 | Serving Relationship | The system SHALL support **Serving** — one element provides functionality to another. Rendered as solid line with open arrowhead. | Functional | EA | Must |
| REQ-MOD-095 | Access Relationship | The system SHALL support **Access** — behavior element accesses a passive structure element. Supports read (R), write (W), read-write (RW), unspecified modes. Rendered as dashed line with open arrowhead. | Functional | EA | Must |
| REQ-MOD-096 | Influence Relationship | The system SHALL support **Influence** — one element affects the implementation or achievement of another (motivation). Supports +/- qualifiers and strength labels. Rendered as dashed line with open arrowhead. | Functional | EA | Must |
| REQ-MOD-097 | Association Relationship | The system SHALL support **Association** — undirected or directed relationship not covered by other types. | Functional | EA | Must |

### 8.3 Dynamic Relationships

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-098 | Triggering Relationship | The system SHALL support **Triggering** — a temporal/causal relationship where one behavior element causes the initiation of another. | Functional | EA | Must |
| REQ-MOD-099 | Flow Relationship | The system SHALL support **Flow** — transfer of information, goods, or energy between elements. Supports direction labels. | Functional | EA | Must |

### 8.4 Other Relationships

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-100 | Specialization Relationship | The system SHALL support **Specialization** — one element is a special kind of another (inheritance-like). | Functional | EA | Must |
| REQ-MOD-101 | Junction (AND/OR) | The system SHALL support **Junction** elements (AND/OR) for combining or branching relationships in dynamic models. | Functional | EA | Must |

### 8.5 Relationship Derivation Rules

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-102 | Derived Relationship Computation | The system SHALL implement ArchiMate 3.2 relationship derivation rules: a relationship between A→C can be derived if A→B and B→C exist, subject to the allowed derivation table. The engine SHALL highlight derived (indirect) relationships distinctly from direct. | Functional | EA | Must |
| REQ-MOD-103 | Allowed Relationship Matrix Enforcement | The system SHALL enforce the ArchiMate 3.2 allowed-relationship matrix: for each source element type × target element type pair, only permitted relationship types SHALL be creatable. Violations SHALL be flagged as validation errors. The matrix SHALL be stored as configuration and updatable for future spec versions. | Functional | EA | Must |
| REQ-MOD-104 | Cross-Layer Relationship Rules | The system SHALL enforce cross-layer relationship rules: realization links lower-layer to upper-layer; serving links same or lower to upper; assignment links roles to behavior within a layer; associations are unrestricted. | Functional | EA | Must |

**Acceptance Criteria:**
- All 13 relationship types render with correct ArchiMate 3.2 visual notation
- Attempting an invalid relationship displays a spec-referenced error message
- Derived relationships are visually distinguished (e.g., dashed with lighter color)
- Derivation chain depth configurable (default: 3 hops)

---

## 9. ArchiMate 3.2 Viewpoints

### 9.1 Standard Viewpoints

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-110 | Basic Viewpoints — Organization | The system SHALL provide the **Organization Viewpoint** showing actors, roles, and their relationships. Pre-configured element palette and layout defaults. | Functional | EA, BA | Must |
| REQ-MOD-111 | Business Process Viewpoint | The system SHALL provide the **Business Process Viewpoint** pre-configured for process, event, function, role, and object elements with swimlane layout. | Functional | EA, BA | Must |
| REQ-MOD-112 | Application Usage Viewpoint | The system SHALL provide the **Application Usage Viewpoint** showing how business processes are supported by application services and components. | Functional | EA, SA | Must |
| REQ-MOD-113 | Technology Viewpoint | The system SHALL provide the **Technology Viewpoint** showing infrastructure topology: nodes, devices, system software, artifacts, and their relationships. | Functional | TA | Must |
| REQ-MOD-114 | Motivation Viewpoint | The system SHALL provide the **Motivation Viewpoint** showing stakeholders, drivers, goals, principles, requirements, and constraints. | Functional | EA | Must |
| REQ-MOD-115 | Strategy Viewpoint | The system SHALL provide the **Strategy Viewpoint** showing resources, capabilities, courses of action, and value streams. | Functional | EA | Must |
| REQ-MOD-116 | Capability Map Viewpoint | The system SHALL provide the **Capability Map Viewpoint** showing hierarchical capability decomposition with heat-map overlays (maturity, investment, gap). | Functional | EA | Must |
| REQ-MOD-117 | Value Stream Viewpoint | The system SHALL provide the **Value Stream Viewpoint** showing value stream stages, triggering relationships, and associated capabilities/roles. | Functional | EA, BA | Must |
| REQ-MOD-118 | Implementation & Migration Viewpoint | The system SHALL provide the **Implementation & Migration Viewpoint** showing work packages, deliverables, plateaus, and gaps on a timeline. | Functional | EA, PM | Must |
| REQ-MOD-119 | Layered Viewpoint | The system SHALL provide the **Layered Viewpoint** rendering all active layers (Strategy, Business, Application, Technology) in horizontal bands with cross-layer relationships. | Functional | EA | Must |
| REQ-MOD-120 | Stakeholder Map Viewpoint | The system SHALL provide the **Stakeholder Map Viewpoint** showing stakeholders, their interests, and influence on goals/outcomes. | Functional | EA, BA | Must |
| REQ-MOD-121 | Custom Viewpoint Definition | The system SHALL allow users to define **Custom Viewpoints** by specifying: allowed element types, allowed relationship types, default layout, default palette. Custom viewpoints SHALL be saveable as named templates. | Functional | EA | Must |
| REQ-MOD-122 | Viewpoint Conformance Checking | The system SHALL validate that elements and relationships in a view conform to the viewpoint's allowed types and flag non-conforming items. | Functional | EA | Should |

**Acceptance Criteria:**
- All 11 standard viewpoints available as one-click templates
- Custom viewpoints persist in user/project settings
- Viewpoint palette auto-filters to allowed element types

---

## 10. Model vs. View Separation

### 10.1 Single Source of Truth Model

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-130 | Centralized Model Repository | The system SHALL maintain a **single centralized model repository** where each ArchiMate element and relationship is stored once, independent of how many views reference it. Deleting a view SHALL NOT delete model elements. | Functional | EA | Must |
| REQ-MOD-131 | Element Reuse Across Views | The system SHALL allow any model element to appear in multiple views simultaneously. Changes to an element's properties SHALL propagate to all views in which it appears. | Functional | EA | Must |
| REQ-MOD-132 | View-Specific Layout Independence | The system SHALL store layout (position, size, visual style overrides) per view separately from element semantic properties. The same element MAY have different positions/sizes in different views. | Functional | EA | Must |
| REQ-MOD-133 | Model Browser / Navigator | The system SHALL provide a **Model Navigator** panel listing all model elements by layer/type, independent of any view. Elements SHALL be drag-droppable from the navigator onto any view. | Functional | EA | Must |
| REQ-MOD-134 | Element Usage Map | For any selected element, the system SHALL display all views in which that element appears, with navigation links. | Functional | EA | Must |
| REQ-MOD-135 | Orphan Element Detection | The system SHALL detect and report model elements that exist in the repository but appear in zero views, and offer bulk-add or delete. | Functional | EA | Should |
| REQ-MOD-136 | Model-Level Property Management | The system SHALL support editing element properties (name, description, documentation, custom attributes) from the model navigator without requiring a view to be open. | Functional | EA | Must |

**Acceptance Criteria:**
- Renaming an element in the model navigator updates all views within 500ms
- Model browser loads 10,000+ elements without pagination lag
- Element usage map shows view list within 1 second

---

## 11. Diagramming Engine — Canvas & Navigation

### 11.1 Canvas Fundamentals

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-140 | Infinite Canvas | The system SHALL provide an **infinite canvas** with no hard boundaries. Elements SHALL be placeable at any coordinate. | Functional | All | Must |
| REQ-MOD-141 | Pan & Zoom | The system SHALL support smooth pan (mouse drag, middle-click, touch two-finger) and zoom (scroll wheel, pinch, keyboard ±, explicit percentage entry). Zoom range: 5%–3200%. | Functional | All | Must |
| REQ-MOD-142 | Minimap / Overview | The system SHALL display an optional **minimap** (thumbnail of full canvas) with a viewport indicator, clickable for navigation. | Functional | All | Must |
| REQ-MOD-143 | Grid | The system SHALL support configurable grids: none, dot, line. Grid spacing configurable (default: 10px). Elements SHALL snap to grid when enabled. | Functional | All | Must |
| REQ-MOD-144 | Rulers & Guides | The system SHALL display horizontal and vertical **rulers** with current scale units (px, pt, mm, in). Users SHALL be able to drag **guide lines** from rulers; elements snap to guides. | Functional | All | Should |
| REQ-MOD-145 | Snap to Elements | The system SHALL provide **smart snapping** to element edges, centers, and baselines of neighboring elements during drag, with visual snap indicators. | Functional | All | Must |
| REQ-MOD-146 | Fit to Screen / Fit Selection | The system SHALL support one-click "Fit to Screen" (all elements) and "Fit Selection" (selected elements) keyboard shortcuts. | Functional | All | Must |
| REQ-MOD-147 | Scroll & Keyboard Navigation | The system SHALL support keyboard navigation: arrow keys to pan, +/- to zoom, Ctrl+A to select all, Ctrl+Z/Y for undo/redo (minimum 100 steps). | Functional | All | Must |
| REQ-MOD-148 | Multi-Select & Lasso | The system SHALL support multi-select via Shift+click and lasso/rubber-band selection. | Functional | All | Must |
| REQ-MOD-149 | Context Menu | The system SHALL provide right-click context menus on canvas, elements, and connectors with relevant actions. | Functional | All | Must |
| REQ-MOD-150 | Canvas Page Management | The system SHALL support **multiple pages/sheets** per diagram file. Pages SHALL be renameable, reorderable, duplicatable, and deletable. | Functional | All | Must |

**Acceptance Criteria:**
- Canvas renders 1,000 elements at 60 fps on modern hardware
- Zoom is smooth with no jitter at all zoom levels
- Undo history survives page switches within a session

---

## 12. Notation Support — UML

### 12.1 UML Diagram Types

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-160 | UML Class Diagram | The system SHALL support **UML Class Diagrams** including: class, abstract class, interface, enumeration, data type, signal, exception. Relationships: association (directed/undirected/bidirectional), dependency, generalization, realization, aggregation, composition. Multiplicity labels, visibility modifiers (+/-/#/~), stereotypes, compartments (attributes/methods). | Functional | SA, DA | Must |
| REQ-MOD-161 | UML Sequence Diagram | The system SHALL support **UML Sequence Diagrams** including: lifelines, activation boxes, synchronous/asynchronous messages, return messages, self-messages, combined fragments (alt/opt/loop/par/break/ref/critical/ignore/consider/neg/assert/strict), interaction use, create/destroy messages. Auto-spacing and numbering. | Functional | SA, BA | Must |
| REQ-MOD-162 | UML Component Diagram | The system SHALL support **UML Component Diagrams** including: components, provided/required interfaces (lollipop/socket notation), ports, connectors, subsystems, packages. | Functional | SA | Must |
| REQ-MOD-163 | UML Deployment Diagram | The system SHALL support **UML Deployment Diagrams** including: nodes, devices, execution environments, artifacts, deploy relationships, communication paths. | Functional | SA, TA | Must |
| REQ-MOD-164 | UML Use Case Diagram | The system SHALL support **UML Use Case Diagrams** including: actors, use cases, system boundary, include/extend/generalize relationships. | Functional | BA, SA | Must |
| REQ-MOD-165 | UML State Machine Diagram | The system SHALL support **UML State Machine Diagrams** including: simple/composite/orthogonal states, initial/final pseudostates, transitions (with guard/trigger/effect), fork/join/choice/junction/history pseudostates, entry/do/exit actions. | Functional | SA | Must |
| REQ-MOD-166 | UML Activity Diagram | The system SHALL support **UML Activity Diagrams** including: initial/final/flow-final nodes, actions, decision/merge nodes, fork/join nodes, object nodes, swimlanes (partitions), call behavior actions, send/accept signal actions, pins, exception handlers. | Functional | BA, SA | Must |
| REQ-MOD-167 | UML Object Diagram | The system SHALL support **UML Object Diagrams** showing object instances, slot values, and link relationships. | Functional | SA | Should |
| REQ-MOD-168 | UML Package Diagram | The system SHALL support **UML Package Diagrams** including: packages, package merge/import/access dependencies, and contained element summary. | Functional | SA | Should |

**Acceptance Criteria:**
- All UML diagrams conform to UML 2.5.1 notation standard
- Sequence diagram auto-renumbers messages on insert/delete
- State machine validates: no transitions from final states, reachability warnings

---

## 13. Notation Support — BPMN 2.0

### 13.1 BPMN Elements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-170 | BPMN Flow Objects | The system SHALL support all BPMN 2.0 **Flow Objects**: events (start/intermediate/end; none/message/timer/error/escalation/cancel/compensation/conditional/link/signal/terminate/multiple/parallel-multiple variations), activities (task subtypes: user/manual/service/send/receive/script/business rule/call; sub-process collapsed/expanded; multi-instance; looping; compensation), gateways (exclusive/inclusive/parallel/event-based/complex). | Functional | BA, PM | Must |
| REQ-MOD-171 | BPMN Connecting Objects | The system SHALL support BPMN **Connecting Objects**: sequence flow (conditional/default), message flow, association (directed/undirected/bidirectional), data association. | Functional | BA | Must |
| REQ-MOD-172 | BPMN Swimlanes | The system SHALL support BPMN **Swimlanes**: pools (representing processes) and lanes (representing participants/roles). Lanes SHALL be resizable and reorderable. Horizontal and vertical orientation. | Functional | BA | Must |
| REQ-MOD-173 | BPMN Artifacts | The system SHALL support BPMN **Artifacts**: data object, data store, data input/output, group, text annotation. | Functional | BA | Must |
| REQ-MOD-174 | BPMN Collaboration Diagrams | The system SHALL support **BPMN Collaboration Diagrams** with multiple pools connected by message flows. | Functional | BA | Must |
| REQ-MOD-175 | BPMN Choreography Diagrams | The system SHALL support **BPMN Choreography Diagrams** showing interactions between participants as choreography tasks and sub-choreographies. | Functional | BA | Should |
| REQ-MOD-176 | BPMN Validation | The system SHALL validate BPMN models for: every process has exactly one start event (or conditional start), sequence flow does not cross pool boundaries (use message flow), gateways have correct branching cardinality, and SHALL highlight errors. | Functional | BA | Must |

**Acceptance Criteria:**
- All BPMN 2.0 standard markers and icons render per OMG spec
- BPMN XML export is valid per OMG BPMN 2.0 XSD schema
- Collaboration diagrams with 5+ pools render without overlap

---

## 14. Notation Support — C4 Model

### 14.1 C4 Levels

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-180 | C4 System Context Diagram | The system SHALL support **C4 Level 1 — System Context** showing the target system, users (person shapes), and external systems with relationship labels (technology + purpose). Structured cards style (colored boxes with title/description/technology labels). | Functional | SA, EA | Must |
| REQ-MOD-181 | C4 Container Diagram | The system SHALL support **C4 Level 2 — Container** showing containers (web apps, APIs, databases, message buses) inside the system boundary with technology annotations. | Functional | SA | Must |
| REQ-MOD-182 | C4 Component Diagram | The system SHALL support **C4 Level 3 — Component** showing components within a container with responsibilities and relationships. | Functional | SA | Must |
| REQ-MOD-183 | C4 Code Diagram | The system SHALL support **C4 Level 4 — Code** (UML class diagram projection) auto-generated from code analysis where available, or manually constructed. | Functional | SA | Should |
| REQ-MOD-184 | C4 Supplementary Diagrams | The system SHALL support **C4 Supplementary Diagrams**: System Landscape (all systems + relationships), Dynamic (numbered sequence of interactions), Deployment (infrastructure mapping). | Functional | SA, TA | Must |
| REQ-MOD-185 | C4 Drill-Down Navigation | The system SHALL support click-through navigation between C4 levels: clicking a system in context drill-down to container, clicking container drill-down to component. | Functional | SA | Should |

**Acceptance Criteria:**
- C4 cards render with title, description, and technology badges in correct C4 style
- System Context→Container drill-down maintains element identity linkage
- C4 export produces Structurizr DSL format

---

## 15. Notation Support — ERD, DFD, Flowcharts

### 15.1 ERD

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-190 | ERD Crow's Foot Notation | The system SHALL support **ERD with Crow's Foot notation**: entities (strong/weak), attributes (simple/composite/derived/multi-valued/key), relationships with crow's foot cardinality symbols (one-and-only-one, zero-or-one, one-or-many, zero-or-many). Identifying/non-identifying relationships. | Functional | DA, SA | Must |
| REQ-MOD-191 | ERD IDEF1X Notation | The system SHALL support **ERD IDEF1X notation**: entity boxes, primary key compartments, foreign key notation, mandatory/optional relationships, categorization structures. | Functional | DA | Should |
| REQ-MOD-192 | ERD Chen Notation | The system SHALL support **ERD Chen notation**: entity rectangles, relationship diamonds, attribute ellipses, participation lines. | Functional | DA | Should |
| REQ-MOD-193 | ERD Auto-Layout | The system SHALL support auto-layout algorithms optimized for ERD: minimize line crossings, align related entities, group by domain area. | Functional | DA | Must |

### 15.2 DFD

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-194 | DFD Level 0 (Context Diagram) | The system SHALL support **DFD Level 0** showing the system as a single process with external entities and data flows. | Functional | BA, SA | Must |
| REQ-MOD-195 | DFD Level 1+ Decomposition | The system SHALL support **DFD decomposition** (Levels 1, 2, n): parent process expands to child DFD. Data stores (Gane-Sarson open rectangle) and external entities (rectangle). Balanced flows between levels. | Functional | BA, SA | Must |
| REQ-MOD-196 | DFD Balance Validation | The system SHALL validate DFD balance: data flows entering/leaving a process at one level must appear in the decomposed level. Unbalanced flows flagged as errors. | Functional | BA | Should |

### 15.3 Flowcharts

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-197 | Standard Flowchart Shapes | The system SHALL support standard **flowchart shapes**: terminal (oval/rounded rect), process (rect), decision (diamond), data I/O (parallelogram), document (wave bottom), multi-document, predefined process (double-bar rect), stored data (cylinder), manual input (sloped top), preparation (hexagon), off-page connector (pentagon), on-page connector (circle), delay (D-shape), annotation (open bracket). | Functional | All | Must |
| REQ-MOD-198 | Cross-Functional Flowchart | The system SHALL support **cross-functional flowcharts** (swim-lane flowcharts) with horizontal or vertical lanes per department/role. | Functional | BA, PM | Must |

**Acceptance Criteria:**
- ERD crow's foot renders correct notation for all 4 cardinality types
- DFD context diagram enforces single process rule at level 0
- Flowchart shapes palette organized by ISO 5807 categories

---

## 16. Notation Support — Infrastructure & Cloud

### 16.1 Cloud Provider Icon Sets

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-200 | AWS Architecture Icons | The system SHALL include the current **AWS Architecture Icons** set (all service categories: Compute, Storage, Database, Networking, Security, Analytics, ML, etc.). Icons SHALL be versioned and updatable. | Functional | TA, SA | Must |
| REQ-MOD-201 | Azure Architecture Icons | The system SHALL include the current **Microsoft Azure Architecture Icons** set covering all Azure service families. | Functional | TA, SA | Must |
| REQ-MOD-202 | GCP Architecture Icons | The system SHALL include the current **Google Cloud Platform Architecture Icons** set. | Functional | TA, SA | Must |
| REQ-MOD-203 | Generic Network Shapes | The system SHALL include generic network/infrastructure shapes: router, switch, firewall, load balancer, server rack, server, desktop, laptop, mobile device, database, storage array, cloud bubble, internet globe, VPN gateway. | Functional | TA | Must |
| REQ-MOD-204 | Icon Set Update Mechanism | The system SHALL provide a mechanism to download and install updated icon sets without full application update. Version of each icon set displayed in settings. | Functional | Admin | Should |

### 16.2 Network Diagram Features

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-205 | Network Zone / Boundary Containers | The system SHALL support **network zone containers** (VPC, subnet, security zone, DMZ) as resizable group containers with distinct visual borders and background colors. | Functional | TA | Must |
| REQ-MOD-206 | Connection Line Types | The system SHALL support network connection styles: ethernet, fiber, wireless (dashed), VPN tunnel (dotted + lock icon), and direction arrows for traffic flow. Labels for bandwidth, protocol, port. | Functional | TA | Must |

**Acceptance Criteria:**
- AWS/Azure/GCP icon sets each contain 200+ service icons
- Icon tooltips show service name and category
- Zone containers auto-resize when child elements are moved outside boundary

---

## 17. Notation Support — Additional Diagrams

### 17.1 Mind Maps

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-210 | Mind Map | The system SHALL support **Mind Maps**: central topic node, branch topics, sub-branches with unlimited depth. Auto-radial/balanced layout. Collapsible branches. Color coding by branch. Image/icon attachment to nodes. | Functional | All | Must |

### 17.2 Wireframes & Mockups

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-211 | Wireframe/Mockup Shapes | The system SHALL provide a **wireframe shape library**: browser frame, mobile frame, buttons, input fields, checkboxes, radio buttons, dropdowns, tables, nav bars, tabs, modals, sliders, progress bars, tooltips, icons (Font Awesome subset). Sketch/lo-fi styling option. | Functional | BA, UX | Should |

### 17.3 Org Charts

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-212 | Org Chart | The system SHALL support **Org Charts**: person card (name, title, photo placeholder), manager-subordinate relationship lines, dotted-line reporting, department grouping. Auto-layout: top-down, left-right, radial. Import from CSV (name, title, manager). | Functional | All | Must |

### 17.4 Value Stream Maps

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-213 | Value Stream Map (Lean VSM) | The system SHALL support **Lean Value Stream Maps** (VSM): process boxes, inventory triangles, push/pull arrows, supplier/customer boxes, information flows (electronic/manual), kaizen bursts, timeline bar (value-add vs. lead time). | Functional | BA, PM | Should |

### 17.5 Timeline & Gantt

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-214 | Roadmap Timeline | The system SHALL support **visual roadmap timeline diagrams** with swimlane rows (per initiative/team/product), milestone markers, phase bars, dependency arrows, quarter/month scale. Data linkable to V09 Portfolio items. | Functional | EA, PM | Must |

---

## 18. Shape Libraries & Stencils

### 18.1 Library Management

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-220 | Built-In Shape Libraries | The system SHALL include built-in shape libraries for: ArchiMate 3.2, UML 2.5, BPMN 2.0, C4, Flowchart, ERD (Crow's Foot/IDEF1X/Chen), Network/Infrastructure, AWS, Azure, GCP, Mind Map, Wireframe, Org Chart, Lean VSM. Each library SHALL be independently enable/disable-able per workspace. | Functional | All | Must |
| REQ-MOD-221 | Custom Shape Library Creation | The system SHALL allow users to create **custom shape libraries**: define SVG-based shapes, set connection points, define properties schema, name and version the library. Custom libraries SHALL be importable/exportable as packages. | Functional | Admin, EA | Must |
| REQ-MOD-222 | Shape Search | The system SHALL provide a **unified shape search** across all enabled libraries by name, tags, and category. Results display as icon grid. | Functional | All | Must |
| REQ-MOD-223 | Favorites / Recently Used | The system SHALL track and display **recently used shapes** and allow users to mark shapes as favorites for quick access. | Functional | All | Should |
| REQ-MOD-224 | Shape Properties Schema | Each shape in a library SHALL define a **property schema** (field name, type, default, validation) that auto-populates the properties panel when the shape is placed. | Functional | Admin | Should |
| REQ-MOD-225 | Organization-Wide Shared Libraries | The system SHALL support organization-wide shared libraries administered by an Admin, pushed to all users in a workspace. | Functional | Admin | Must |

**Acceptance Criteria:**
- Shape search returns results within 200ms across all enabled libraries
- Custom SVG shapes render at all zoom levels without rasterization
- Shared library updates propagate to all users within 30 seconds

---

## 19. Connectors, Routing & Layout

### 19.1 Connector Types & Routing

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-230 | Connector Styles | The system SHALL support connector styles: straight line, elbow (orthogonal), curved (bezier), step (right-angle step). Per-connector style override. | Functional | All | Must |
| REQ-MOD-231 | Auto-Routing | The system SHALL provide **auto-routing**: connectors SHALL automatically route around elements to avoid overlap. Re-routing SHALL trigger on element move. | Functional | All | Must |
| REQ-MOD-232 | Manual Route Waypoints | The system SHALL allow users to add, move, and delete **waypoints** on connectors for manual routing overrides. Waypoints SHALL persist on element move (with best-effort anchoring). | Functional | All | Must |
| REQ-MOD-233 | Connection Point Anchors | Each shape SHALL support **connection point anchors** (default: center + 4 cardinal midpoints + 8 corner/edge points). Custom connection points SHALL be definable per shape type. | Functional | All | Must |
| REQ-MOD-234 | Connector Labels | Connectors SHALL support up to 3 **label positions** (source, center, target). Labels SHALL be draggable along the connector path. | Functional | All | Must |
| REQ-MOD-235 | Connector Bidirectionality | The system SHALL support unidirectional, bidirectional, and no-arrow connector ends. End markers: open arrow, filled arrow, diamond, circle, cross, half-arrow, crow's foot variants. | Functional | All | Must |

### 19.2 Auto-Layout Algorithms

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-236 | Hierarchical / Tree Layout | The system SHALL provide **hierarchical layout** (Sugiyama algorithm): top-down, bottom-up, left-right, right-left. Configurable rank separation and node separation. | Functional | All | Must |
| REQ-MOD-237 | Force-Directed Layout | The system SHALL provide **force-directed layout** for network-style diagrams: configurable spring stiffness, charge, gravity. | Functional | All | Must |
| REQ-MOD-238 | Organic / Spring Layout | The system SHALL provide **organic/spring layout** suitable for mind maps and general graphs. | Functional | All | Should |
| REQ-MOD-239 | Grid Layout | The system SHALL provide **grid layout**: arrange selected elements in a regular grid with configurable rows/columns and spacing. | Functional | All | Must |
| REQ-MOD-240 | Radial Layout | The system SHALL provide **radial layout** with configurable root node, ring spacing, and angular distribution. | Functional | All | Should |
| REQ-MOD-241 | Circular Layout | The system SHALL provide **circular layout** distributing nodes evenly on a circle. | Functional | All | Should |
| REQ-MOD-242 | Lane-Based Layout | The system SHALL provide **swimlane-aware layout**: elements sorted into swimlanes based on assigned lane attribute; within-lane hierarchical layout. | Functional | BA | Must |
| REQ-MOD-243 | Partial / Incremental Layout | The system SHALL support **partial layout** on selected elements only, without disturbing unselected elements. | Functional | All | Should |
| REQ-MOD-244 | Layout Preservation | After auto-layout, the system SHALL support undo to restore the previous layout in a single step. | Functional | All | Must |

**Acceptance Criteria:**
- Auto-routing completes in < 100ms for diagrams with ≤ 500 connectors
- Hierarchical layout on a 200-node diagram completes in < 2 seconds
- Force-directed layout converges and stabilizes without oscillation

---

## 20. Themes, Styling & Conditional Formatting

### 20.1 Themes & Styling

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-250 | Global Themes | The system SHALL provide built-in diagram **themes** (light, dark, high-contrast, corporate-neutral, blueprint). Applying a theme SHALL update all shape colors, fonts, and connector styles globally. | Functional | All | Must |
| REQ-MOD-251 | Custom Theme Editor | The system SHALL provide a **custom theme editor** allowing definition of: default shape fill color, border color, font family/size/color, connector color/width/dash pattern, background color, grid color. Custom themes SHALL be saveable and shareable. | Functional | Admin, EA | Must |
| REQ-MOD-252 | Per-Element Style Override | The system SHALL allow **per-element style overrides** for any attribute (fill, border, font, opacity, shadow, gradient) without breaking theme inheritance. Reset-to-theme SHALL be available. | Functional | All | Must |
| REQ-MOD-253 | Style Copy / Paste | The system SHALL support **style copy/paste**: copy the visual style of one element and apply to selected others (format painter). | Functional | All | Must |
| REQ-MOD-254 | Organization Brand Colors | Admins SHALL be able to define an organization-wide **brand palette** (primary, secondary, accent, neutral colors) available as swatches in all color pickers. | Functional | Admin | Must |

### 20.2 Conditional Formatting

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-255 | Conditional Formatting Rules | The system SHALL support **conditional formatting rules**: if element property matches condition (equals, contains, greater-than, is-empty), apply visual style (fill color, border color, icon overlay, badge). | Functional | EA | Must |
| REQ-MOD-256 | Heat Map Overlay | The system SHALL support **heat map overlays** on capability maps and other diagrams: map a numeric property (e.g., maturity 1-5, risk score) to a color gradient (red-yellow-green or custom). | Functional | EA | Must |
| REQ-MOD-257 | Status Badges | The system SHALL support **status badge overlays** on elements: configurable icons/colors for lifecycle states (current, target, planned, decommission, under review). | Functional | EA, PM | Must |
| REQ-MOD-258 | Conditional Rule Management | The system SHALL provide a conditional formatting rule manager: list, edit, reorder (priority), enable/disable, and delete rules. Rules SHALL be scoped per diagram or globally per project. | Functional | EA | Should |

**Acceptance Criteria:**
- Theme switch applies to all elements in < 500ms for 500-element diagram
- Conditional formatting re-evaluates on property change within 200ms
- Heat map gradient renders correctly with 5-point scale

---

## 21. Layers, Swimlanes & Grouping

### 21.1 Layers

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-260 | Layer Management Panel | The system SHALL provide a **Layers panel** listing all layers in a diagram. Operations: add, rename, delete, reorder (drag), duplicate. | Functional | All | Must |
| REQ-MOD-261 | Layer Visibility Toggle | The system SHALL support **per-layer visibility toggle** (show/hide). Hidden layer elements are invisible and non-interactive but preserved in the model. | Functional | All | Must |
| REQ-MOD-262 | Layer Lock | The system SHALL support **per-layer locking**: locked layer elements are visible but not selectable/moveable. | Functional | All | Must |
| REQ-MOD-263 | Element Layer Assignment | The system SHALL allow elements to be moved between layers via: right-click > Move to Layer, layers panel drag, or keyboard shortcut. Multi-element layer assignment SHALL be supported. | Functional | All | Must |
| REQ-MOD-264 | Default Diagram Layers | ArchiMate diagrams SHALL pre-create layers matching ArchiMate layers: Strategy, Business, Application, Technology, Physical, Motivation, Implementation. Elements SHALL be auto-assigned to matching layers on creation. | Functional | EA | Must |

### 21.2 Swimlanes & Pools

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-265 | Swimlane Container | The system SHALL support **swimlane containers**: horizontally or vertically partitioned containers with labeled lanes. Elements dropped into a lane are assigned to it. | Functional | BA, PM | Must |
| REQ-MOD-266 | Lane Resize & Reorder | Swimlane lanes SHALL be resizable by dragging borders. Lanes SHALL be reorderable by drag. Resizing a lane SHALL push adjacent lanes. | Functional | BA | Must |
| REQ-MOD-267 | Lane Auto-Resize | Swimlane containers SHALL auto-resize to contain all child elements when elements are moved to the boundary. | Functional | BA | Must |

### 21.3 Grouping

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-268 | Visual Grouping | The system SHALL support **visual groups** (labeled bounding box containers) that move with contained elements. Groups are visual only (no semantic model impact) distinct from ArchiMate composition. | Functional | All | Must |
| REQ-MOD-269 | Collapse / Expand Groups | Groups and swimlane containers SHALL support **collapse/expand**: collapsed group shows as a single summary box; expand restores full contents. | Functional | All | Must |
| REQ-MOD-270 | Nested Containers | The system SHALL support containers nested within containers to at least 5 levels deep without performance degradation. | Functional | All | Must |

**Acceptance Criteria:**
- Layer toggle shows/hides elements within 100ms
- Swimlane auto-resize does not cause infinite resize loops
- Nested containers at depth 5 render without visual glitches

---

## 22. Model Intelligence

### 22.1 Metamodel Validation

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-280 | Real-Time Metamodel Validation | The system SHALL perform **real-time ArchiMate metamodel validation** as elements and relationships are created/modified. Violations SHALL be highlighted inline (element border turns red, tooltip shows violation rule + spec reference). | Functional | EA | Must |
| REQ-MOD-281 | Validation Report | The system SHALL generate a **structured validation report** listing all violations in the model: element/relationship ID, violation type, severity (error/warning/info), rule reference, and suggested fix. Report SHALL be exportable as CSV/PDF. | Functional | EA | Must |
| REQ-MOD-282 | Relationship Matrix Viewer | The system SHALL display an **interactive relationship matrix** (N×N grid) showing all relationships between selected element sets. Cells clickable to navigate to relationships in view. | Functional | EA | Must |
| REQ-MOD-283 | Custom Validation Rules | The system SHALL support **custom validation rules** defined via a rule DSL or expression builder: specify element type filter, property condition, relationship presence/absence condition, and error message. | Functional | EA, Admin | Should |
| REQ-MOD-284 | Model Completeness Score | The system SHALL calculate and display a **model completeness score** per viewpoint: percentage of required element types present, percentage of elements with mandatory properties filled, percentage of required relationships present. | Functional | EA | Should |

### 22.2 Impact & Dependency Analysis

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-285 | Impact Analysis | For any selected element, the system SHALL compute and visualize the **impact footprint**: all elements that directly or indirectly depend on the selected element (downstream impact). Configurable hop depth (1–10). Results shown as highlighted sub-graph and exportable list. | Functional | EA, SA | Must |
| REQ-MOD-286 | Dependency Analysis | For any selected element, the system SHALL compute **upstream dependencies**: all elements that the selected element directly or indirectly depends upon. Configurable hop depth. | Functional | EA, SA | Must |
| REQ-MOD-287 | Cross-Layer Dependency Tracing | The system SHALL trace dependencies **across layers**: e.g., a business process → application service → application component → technology node path shall be navigable in a single impact trace. | Functional | EA | Must |
| REQ-MOD-288 | Circular Dependency Detection | The system SHALL detect and flag **circular dependencies** in the model: cycles in realization, serving, and triggering relationship graphs. Cycles highlighted and listed with chain description. | Functional | EA, SA | Must |
| REQ-MOD-289 | What-If Analysis Workspace | The system SHALL support a **what-if workspace mode**: create a named branch of the model, make experimental changes (add/remove/modify elements/relationships), run impact analysis on the changes, and compare against the baseline. Changes are isolated until explicitly merged. | Functional | EA | Should |

### 22.3 Traceability to Requirements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-290 | Element-to-Requirement Traceability | The system SHALL support linking any model element to one or more **ESIP Requirement records** (V07). Links displayed as a traceability badge on the element and as entries in the requirement's coverage matrix. | Functional | EA, BA | Must |
| REQ-MOD-291 | Traceability Matrix View | The system SHALL generate an **element-to-requirement traceability matrix** showing: each model element, its linked requirements, coverage percentage, and uncovered elements. Filterable by layer, viewpoint, requirement status. | Functional | EA, BA | Must |
| REQ-MOD-292 | Traceability Gap Report | The system SHALL identify **traceability gaps**: requirements with no linked model elements (unaddressed requirements) and model elements with no linked requirements (unjustified elements). | Functional | EA, BA | Should |

### 22.4 Search & Metrics

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-293 | Global Model Search | The system SHALL provide **global search** across the model repository: search by name, description, property value, element type, and relationship type. Results navigate to element in model browser and highlight in all views. | Functional | All | Must |
| REQ-MOD-294 | Model Metrics Dashboard | The system SHALL provide a **model metrics dashboard** showing: total element count by type, total relationship count by type, elements per layer, views per element (mean/max), model growth over time (requires version history), orphan element count, validation error/warning counts. | Functional | EA, Admin | Should |
| REQ-MOD-295 | Cross-View Consistency Check | The system SHALL detect **cross-view inconsistencies**: elements appearing in views that contradict each other (e.g., same application component assigned to different technology nodes in two deployment views). Configurable consistency rules. | Functional | EA | Should |

**Acceptance Criteria:**
- Impact analysis on a 2,000-element model completes in < 3 seconds
- Global search returns results in < 500ms
- Validation report generated in < 5 seconds for models with 5,000 elements
- Traceability matrix exports correctly with 100% of links preserved

---

## 23. Collaboration & Lifecycle

### 23.1 Real-Time Co-Editing

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-300 | Real-Time Multi-User Editing | The system SHALL support **real-time collaborative editing** of diagrams: multiple users editing the same diagram simultaneously with conflict-free merged updates (CRDT or OT-based). User cursors and selections visible to all collaborators with user color coding and name labels. | Functional | All | Must |
| REQ-MOD-301 | Presence Indicators | The system SHALL display **presence indicators**: avatar list of currently active users in a diagram, with user name and last-active timestamp. | Functional | All | Must |
| REQ-MOD-302 | Collaborative Pointer | Each collaborator SHALL have a **named cursor/pointer** visible to others, updating in near real-time (< 100ms latency on LAN). | Functional | All | Must |
| REQ-MOD-303 | Multiplayer Selection Highlighting | Elements selected by other users SHALL be highlighted in that user's color so conflicts in editing are visually apparent. | Functional | All | Must |

### 23.2 Comments & Annotations

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-304 | Element-Level Comments | The system SHALL support **threaded comments** on individual elements and connectors: text, @mention, emoji reactions, resolve/reopen. Comments visible as a badge on the element. | Functional | All | Must |
| REQ-MOD-305 | Canvas-Level Comments | The system SHALL support **free-floating comments** anchored to canvas coordinates (not tied to an element), useful for reviewing empty areas or overall layout. | Functional | All | Should |
| REQ-MOD-306 | Comment Notifications | Users SHALL receive **notifications** (in-app + email, configurable) when @mentioned in a comment or when a comment thread they participated in is updated. | Functional | All | Must |
| REQ-MOD-307 | Comment Export | The system SHALL allow export of all comments in a diagram as a structured report (CSV/PDF) including: element name, comment author, timestamp, comment text, status. | Functional | EA, PM | Should |

### 23.3 Version History & Visual Diff

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-308 | Automatic Version Snapshots | The system SHALL automatically create **named version snapshots** at configurable intervals (default: every 15 minutes when changes are present) and on explicit user save-as-version actions. | Functional | All | Must |
| REQ-MOD-309 | Named Milestone Versions | Users SHALL be able to manually create **named milestone versions** (e.g., "Approved Baseline v1.2") with description and tag. | Functional | EA, PM | Must |
| REQ-MOD-310 | Version History Browser | The system SHALL provide a **version history timeline** listing all snapshots and milestones: date, author, label, change summary. Users can preview any version. | Functional | All | Must |
| REQ-MOD-311 | Visual Diff | The system SHALL provide **visual diff** between any two versions: elements added (green highlight), removed (red highlight), moved (orange highlight), property-changed (yellow highlight). Diff navigable as a change list. | Functional | EA | Must |
| REQ-MOD-312 | Restore Previous Version | Users SHALL be able to **restore** any previous version as the current version (creates a new snapshot for safety before restore). | Functional | EA | Must |
| REQ-MOD-313 | Version Branching | The system SHALL support creating a **branch** from any version, allowing parallel development of diagram variants. Branches SHALL be independently versioned. Merge of branches SHALL highlight conflicts for manual resolution. | Functional | EA | Should |

### 23.4 Locking

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-314 | Element-Level Locking | The system SHALL support **per-element locking**: a user can lock an element to prevent others from editing it. Locked elements shown with lock icon overlay. Only the lock owner or Admin can unlock. | Functional | EA | Should |
| REQ-MOD-315 | Diagram-Level Locking | The system SHALL support **diagram-level locking** (read-only mode for non-owners): only the diagram owner or Admin can edit; others can view and comment. | Functional | EA, Admin | Must |

### 23.5 Templates

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-316 | Diagram Templates | The system SHALL provide a **template gallery**: pre-built diagram templates for each supported notation type. Templates SHALL be usable to create new diagrams with pre-populated example elements. | Functional | All | Must |
| REQ-MOD-317 | Save as Template | Users SHALL be able to **save any diagram as a template** (private or organization-shared). Templates support placeholder variables (e.g., `{{SystemName}}`) prompted on template instantiation. | Functional | EA, Admin | Must |

**Acceptance Criteria:**
- Real-time co-edit conflict-free for 10 concurrent users on a 500-element diagram
- Visual diff renders within 2 seconds for version pairs with < 1,000 element changes
- Automatic snapshots do not block user interactions (background save)

---

## 24. Interoperability — Import

> **Ownership boundary (reconciliation — MERGE):** **Volume 18 (`REQ-IMP-*`) is the canonical owner of all diagram/model import** (draw.io/Visio/Sparx EA/AMEFF/Mermaid/PlantUML/Lucidchart/XMI + competitor EA tools, import wizard, metamodel mapping, round-trip). The requirements below (**REQ-MOD-320–327**) are **retained as cross-references, not as separate scope** — they are **owned by V18; V03 only renders the elements produced by the V18 import pipeline** onto the canvas (see REQ-MOD-351 result-receiver contract). These rows are **not deleted**; they are annotated below so existing IDs/links remain valid. V03's **export-format** requirements (§25, REQ-MOD-330–340) remain owned by V03 and are unaffected.

### 24.1 Import Formats

> **Each requirement in this table is owned by V18 (`REQ-IMP-010–018`).** V03 consumes the V18 import output and renders it; it does not implement parsing/format drivers. Treat the rows below as the V03↔V18 rendering contract.

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-320 | Visio VSDX Import | The system SHALL import **Microsoft Visio .vsdx** files: pages, shapes (mapped to nearest ESIP shape library equivalents), connectors, text labels, basic formatting (fill color, border, font). Import report SHALL list unmapped shapes. | Integration | All | Must |
| REQ-MOD-321 | draw.io / diagrams.net Import | The system SHALL import **draw.io XML** (.drawio, .xml) files: full fidelity for shapes, connectors, labels, layers, pages, and formatting. Custom draw.io shapes mapped to ESIP shape IDs or embedded as SVG. | Integration | All | Must |
| REQ-MOD-322 | XMI Import | The system SHALL import **XMI 2.x** (XML Metadata Interchange) for UML models: class diagrams, component diagrams, state machines. Mapping report for XMI-version differences. | Integration | SA, DA | Must |
| REQ-MOD-323 | ArchiMate Model Exchange File Format (AMEFF) Import | The system SHALL import **ArchiMate Model Exchange File Format** (Open Group AMEFF 3.x): elements, relationships, views, properties, and organization structure. 100% element/relationship preservation. | Integration | EA | Must |
| REQ-MOD-324 | Mermaid Import | The system SHALL import **Mermaid diagram syntax** (flowchart, sequence, class, state, ERD, Gantt, pie, quadrant chart, mindmap types). Parsed diagrams rendered as editable ESIP diagrams. | Integration | All | Must |
| REQ-MOD-325 | PlantUML Import | The system SHALL import **PlantUML** source (.puml, .plantuml): sequence, class, component, state, activity, object, deployment, use case, timing diagrams. | Integration | SA | Must |
| REQ-MOD-326 | CSV/Excel Import for Org Charts | The system SHALL import **CSV/Excel** files for org chart generation: required columns name + manager (or hierarchy path); optional: title, department, email, photo URL. | Integration | All | Should |
| REQ-MOD-327 | Import Conflict Resolution | When importing a diagram whose elements conflict with existing model elements (same name/ID), the system SHALL offer conflict resolution: skip, overwrite, create-new, or merge. | Integration | EA | Must |

**Acceptance Criteria:**
- AMEFF import round-trips with 100% element and relationship preservation
- Visio VSDX import handles files up to 50MB
- Mermaid import supports all Mermaid 10.x diagram types

**Cross-reference / canonical ownership (V18):** REQ-MOD-320 → V18 (Visio import); REQ-MOD-321 → V18 (draw.io import); REQ-MOD-322 → V18 (XMI import); REQ-MOD-323 → V18 (AMEFF import); REQ-MOD-324 → V18 (Mermaid import); REQ-MOD-325 → V18 (PlantUML import); REQ-MOD-326 → V18 (CSV/Excel org-chart import); REQ-MOD-327 → V18 (import conflict resolution). All eight are **owned and implemented by Volume 18**; V03's responsibility is limited to **rendering the resulting elements as editable diagrams** via the V18 → V03 hand-off (REQ-MOD-351). Retained here for ID/link stability — **do not implement independently in V03**.

---

## 25. Interoperability — Export

### 25.1 Export Formats

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-330 | PNG Export | The system SHALL export diagrams as **PNG**: configurable resolution (72–600 DPI), transparent or white background, optional padding, crop to content or fixed page size. | Integration | All | Must |
| REQ-MOD-331 | SVG Export | The system SHALL export diagrams as **SVG**: vector output with embedded fonts, correct layering, optional CSS styling. SVG SHALL be renderable in all modern browsers. | Integration | All | Must |
| REQ-MOD-332 | PDF Export | The system SHALL export diagrams as **PDF**: single-page or multi-page (one per diagram page), configurable paper size (A4/A3/Letter/Tabloid/custom) and orientation. Embedded fonts. Optional diagram title/footer. | Integration | All | Must |
| REQ-MOD-333 | Visio VSDX Export | The system SHALL export diagrams as **Visio .vsdx** with correct shape types, connectors, labels, and basic formatting. | Integration | All | Must |
| REQ-MOD-334 | draw.io XML Export | The system SHALL export diagrams as **draw.io XML** (.drawio) with full layout fidelity. | Integration | All | Must |
| REQ-MOD-335 | XMI Export | The system SHALL export UML models as **XMI 2.x**, suitable for import into Eclipse, Enterprise Architect, MagicDraw, and other UML tools. | Integration | SA, DA | Must |
| REQ-MOD-336 | ArchiMate Model Exchange File Format (AMEFF) Export | The system SHALL export ArchiMate models as **AMEFF 3.x** XML, conformant with the Open Group schema. All elements, relationships, views, properties, and organization structure SHALL be preserved. | Integration | EA | Must |
| REQ-MOD-337 | Mermaid Export | The system SHALL export supported diagram types as **Mermaid syntax**: flowchart, sequence, class, state, ERD, Gantt. | Integration | All | Should |
| REQ-MOD-338 | PlantUML Export | The system SHALL export supported diagram types as **PlantUML** source. | Integration | SA | Should |
| REQ-MOD-339 | Embed / Share Link | The system SHALL generate a **shareable link** for any diagram: view-only (default) or editable (with permission). Embeddable as an iframe with configurable dimensions. | Integration | All | Must |
| REQ-MOD-340 | Bulk Export | The system SHALL support **bulk export** of multiple diagrams or an entire project folder: package as ZIP containing each diagram in selected format. | Integration | All | Should |

**Acceptance Criteria:**
- AMEFF export validates against the Open Group AMEFF 3.x XSD schema
- PDF export of a 20-page diagram completes in < 30 seconds
- SVG output renders identically to canvas view in Chrome/Firefox/Safari

---

## 26. AI Integration Hooks

*Note: AI model selection, prompt engineering, training, and generation quality are specified in Volume 06. This section specifies the integration contracts and surface points in the modeling/diagramming engine that AI capabilities hook into.*

### 26.1 Text/Requirements → Diagram Generation

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-350 | AI Generate Trigger Surface | The system SHALL expose a **"Generate Diagram" command surface** accepting: (a) free-text description, (b) a set of requirement record IDs from V07, (c) an uploaded document, or (d) a code repository reference. The command SHALL pass this context payload to the V06 AI generation engine via a defined internal API contract. | AI/ML | All | Must |
| REQ-MOD-351 | AI Generation Result Receiver | The system SHALL receive from the V06 AI generation engine a **structured diagram payload**: element list (type, name, properties), relationship list (type, source, target, properties), layout hints (initial positions, grouping), and target viewpoint/notation. The engine SHALL render this payload onto the canvas as a fully editable diagram. | AI/ML | All | Must |
| REQ-MOD-352 | Incremental AI Refinement | The system SHALL support **incremental AI refinement**: user selects existing diagram elements, provides a natural-language instruction ("add a messaging layer between these two components"), and the system passes: selected elements context + instruction to V06, then merges the result into the existing diagram without disturbing unselected elements. | AI/ML | EA, SA | Must |
| REQ-MOD-353 | Generation Progress & Streaming | During AI generation, the system SHALL display a **progress state**: spinner + status message. If the V06 engine supports streaming output, the system SHALL render elements incrementally as they arrive. | AI/ML | All | Should |
| REQ-MOD-354 | Generation History | The system SHALL maintain a **generation history** per diagram: each AI generation event logged with timestamp, input summary, and result. User can view and re-apply any previous generation as a new layer. | AI/ML | EA | Should |

### 26.2 Diagram → Narrative Generation

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-355 | AI Explain Diagram Trigger | The system SHALL expose an **"Explain Diagram" command** that serializes the current view (elements, relationships, layout, properties) into a structured context payload and passes it to the V06 AI engine for narrative generation. | AI/ML | All | Must |
| REQ-MOD-356 | AI Explain Selection Trigger | The system SHALL expose an **"Explain Selection" command** on one or more selected elements, passing only the selected sub-graph to V06 for a focused narrative. | AI/ML | All | Must |
| REQ-MOD-357 | Narrative Output Receiver | The system SHALL receive from V06 a **natural-language narrative** (Markdown) and display it in an in-panel "Diagram Story" pane with: copy-to-clipboard, insert-into-V08-article, and export-as-Word/PDF actions. | AI/ML | All | Must |
| REQ-MOD-358 | Narrative Audience Targeting | The "Explain Diagram" command SHALL accept an **audience parameter** (executive, architect, developer, non-technical stakeholder) passed to V06 to tune narrative depth and language. | AI/ML | EA | Must |

### 26.3 AI-Assisted Modeling

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-359 | AI Element Suggestion | As the user types an element name, the system SHALL offer **AI-powered suggestions** for element type, related elements already in the model, and recommended relationship types, via a lightweight call to V06 inference endpoint. | AI/ML | EA, SA | Should |
| REQ-MOD-360 | AI Relationship Suggestion | When a new connector is drawn between two elements, the system SHALL query V06 for the most probable **relationship type and direction** given the element types and names, pre-populating the relationship type selector. | AI/ML | EA | Should |
| REQ-MOD-361 | AI Layout Suggestion | The system SHALL provide an **"AI Layout" command** that passes the current element/relationship graph to V06 for a semantically-aware layout suggestion (e.g., placing related business elements near their application services). V06 returns preferred position hints; the diagramming engine applies them as a layout option. | AI/ML | EA | Should |
| REQ-MOD-362 | AI Completeness Suggestions | After computing model completeness (REQ-MOD-284), the system SHALL optionally query V06 for **natural-language suggestions** on which element types or relationship types are commonly expected for the current viewpoint but are missing. | AI/ML | EA | Should |

### 26.4 AI API Contract

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-363 | Internal AI API Schema | The system SHALL define and publish an **internal AI integration API schema** (OpenAPI or GraphQL): endpoints for generate-diagram, explain-diagram, element-suggest, relationship-suggest, layout-suggest. Schema SHALL be versioned. V06 SHALL implement this contract; this volume SHALL consume it. | Integration | Platform | Must |
| REQ-MOD-364 | AI Call Timeout & Fallback | All AI integration calls SHALL enforce a **configurable timeout** (default: 30 seconds). On timeout or error, the system SHALL display a user-friendly message and allow retry without data loss. | NFR | All | Must |
| REQ-MOD-365 | AI Usage Audit Log | Every AI call made from the diagramming engine SHALL be logged to a **usage audit record** (timestamp, user, diagram ID, endpoint, input token estimate, response time, success/failure) accessible to Admins in V11 platform settings. | Governance | Admin | Must |

**Acceptance Criteria:**
- Generate-diagram API call schema is versioned and passes OpenAPI 3.1 validation
- AI explain-diagram call completes within 30 seconds for diagrams with ≤ 200 elements
- Incremental AI refinement does not alter elements outside the selection boundary
- All AI calls appear in the audit log within 5 seconds of completion

---

## Reconciliation Additions

## 27. Canvas Accessibility (WCAG)

> **Gap closure (G-2):** WCAG/keyboard/screen-reader support for the **interactive maxGraph canvas/editor itself** (not just exported HTML/SVG, which V10 covers). This is the most-failed enterprise accessibility area. These are **Must for the Phase-3 canvas** (per the build-phase banner above); they do not apply to the MVP Mermaid/AI-generated-diagram bridge, which inherits V10 export a11y.

### 27.1 Canvas Accessibility Requirements

| ID | Title | SHALL Description | Category | Persona(s) | Priority |
|----|-------|-------------------|----------|------------|----------|
| REQ-MOD-A11Y-001 | Keyboard-Only Canvas Navigation | The system SHALL allow full operation of the diagramming canvas using keyboard only — no pointer required — including selecting, moving, resizing, connecting, creating, and deleting elements, plus pan/zoom, via documented key bindings. | UX | All | Must (Phase 3 canvas) |
| REQ-MOD-A11Y-002 | Deterministic Focus & Tab Order | The system SHALL maintain a visible focus indicator and a deterministic, logical tab order across canvas elements, connectors, and toolbars, so keyboard and assistive-technology users can traverse a diagram predictably (e.g. reading order or model order). | UX | All | Must (Phase 3 canvas) |
| REQ-MOD-A11Y-003 | ARIA Live-Region Change Announcements | The system SHALL announce diagram changes (element added/removed/moved/connected, selection change, validation result) to screen readers via ARIA live regions, so non-visual users receive feedback on canvas mutations. | UX | All | Must (Phase 3 canvas) |
| REQ-MOD-A11Y-004 | Non-Colour-Coded Diagram Semantics | The system SHALL convey all diagram semantics (element type, layer, state, validation severity, selection) by means in addition to colour — e.g. text labels, icons, patterns, or shape — so meaning is not lost for colour-blind users or in high-contrast mode (WCAG 1.4.1 Use of Colour). | UX | All | Must (Phase 3 canvas) |
| REQ-MOD-A11Y-005 | Screen-Reader Element Descriptions | The system SHALL expose an accessible name and description for every canvas element and connector (type, label, key properties, and incident relationships) to assistive technology, enabling a screen-reader user to understand a diagram without seeing it. | UX | All | Must (Phase 3 canvas) |

**Rationale:** WCAG 2.1 AA for interactive content (the canvas, not just static exports) is a hard enterprise-procurement and public-sector requirement; closes reconciliation gap G-2. Complements V10 export a11y and the V19 WCAG 2.1 AA commitment.

**Acceptance Criteria:**
- A representative ArchiMate view can be created, edited, navigated, and explained end-to-end with keyboard only.
- Screen-reader (NVDA/JAWS/VoiceOver) reads element type, name, and relationships for any focused element.
- Diagram passes an automated + manual WCAG 2.1 AA audit of the canvas, including colour-independence and live-region announcements.

---

## Appendix A — Requirement Summary

| Capability Area | Requirement IDs | Count |
|---|---|---|
| 1. Strategy Layer Elements | REQ-MOD-001–004 | 4 |
| 2. Business Layer Elements | REQ-MOD-010–022 | 13 |
| 3. Application Layer Elements | REQ-MOD-030–038 | 9 |
| 4. Technology Layer Elements | REQ-MOD-040–052 | 13 |
| 5. Physical Layer Elements | REQ-MOD-060–063 | 4 |
| 6. Motivation Aspect Elements | REQ-MOD-070–079 | 10 |
| 7. Implementation & Migration Elements | REQ-MOD-080–084 | 5 |
| 8. Relationships & Derivation Rules | REQ-MOD-090–104 | 15 |
| 9. Viewpoints | REQ-MOD-110–122 | 13 |
| 10. Model vs. View Separation | REQ-MOD-130–136 | 7 |
| 11. Canvas & Navigation | REQ-MOD-140–150 | 11 |
| 12. UML Notation | REQ-MOD-160–168 | 9 |
| 13. BPMN 2.0 Notation | REQ-MOD-170–176 | 7 |
| 14. C4 Model Notation | REQ-MOD-180–185 | 6 |
| 15. ERD, DFD, Flowcharts | REQ-MOD-190–198 | 9 |
| 16. Infrastructure & Cloud | REQ-MOD-200–206 | 7 |
| 17. Additional Diagrams | REQ-MOD-210–214 | 5 |
| 18. Shape Libraries & Stencils | REQ-MOD-220–225 | 6 |
| 19. Connectors, Routing & Layout | REQ-MOD-230–244 | 15 |
| 20. Themes, Styling & Conditional Formatting | REQ-MOD-250–258 | 9 |
| 21. Layers, Swimlanes & Grouping | REQ-MOD-260–270 | 11 |
| 22. Model Intelligence | REQ-MOD-280–295 | 16 |
| 23. Collaboration & Lifecycle | REQ-MOD-300–317 | 18 |
| 24. Interoperability — Import | REQ-MOD-320–327 | 8 |
| 25. Interoperability — Export | REQ-MOD-330–340 | 11 |
| 26. AI Integration Hooks | REQ-MOD-350–365 | 16 |
| 27. Canvas Accessibility (WCAG) *(Reconciliation Addition, G-2)* | REQ-MOD-A11Y-001–005 | 5 |
| **TOTAL** | | **276 baseline + 5 reconciliation (A11Y) = 281** |

---

## Appendix B — Persona Key

| Code | Role |
|------|------|
| EA | Enterprise Architect |
| BA | Business Analyst |
| SA | Solution Architect |
| TA | Technology/Infrastructure Architect |
| DA | Data Architect |
| PM | Program/Project Manager |
| Admin | Platform Administrator |
| UX | UX/Product Designer |
| All | All above personas |

---

## Appendix C — Boundary Notes & Assumptions

1. **V06 Boundary:** AI model IDs, prompt templates, fine-tuning, token budgets, hallucination mitigation, and generation quality metrics are fully specified in Volume 06. This volume specifies only the surface API contract and UI integration points.

2. **V10 Boundary:** Rendering diagrams into styled Word/PPT/PDF deliverables (branded templates, cover pages, appendices) is specified in Volume 10. This volume covers raw export formats (PNG/SVG/PDF/Visio/draw.io/XMI/AMEFF).

3. **V07 Boundary:** Requirements management (capturing, classifying, tracing, baselining ESIP requirements) is in Volume 07. The ArchiMate Motivation `Requirement` element (REQ-MOD-076) is a model element only; its ESIP management lifecycle lives in V07.

4. **V01 Boundary:** TOGAF ADM phase linkage for work packages/plateaus is specified in Volume 01. This volume specifies the model elements and their properties that carry phase annotations.

5. **ArchiMate 3.2 vs 3.1:** The Physical layer and Value Stream element were introduced in ArchiMate 3.0/3.1. This volume targets 3.2 as the conformance target. Any 3.2-specific changes from 3.1 SHALL be absorbed without a breaking schema change.

6. **AMEFF Conformance:** Open Group AMEFF schema version 3.2 is the target. Import/export SHALL pass the Open Group's published XSD validator.

7. **BPMN XML Export:** BPMN export targets the OMG BPMN 2.0 XSD. Choreography and conversation diagrams are lower priority (Should) due to limited tooling ecosystem support.

8. **Real-Time Collaboration:** Assumes a backend capable of CRDT or OT (Operational Transformation). Implementation detail deferred to V11 Platform. This volume specifies the user-visible behavior contract.

9. **Cloud Icon Set Licensing:** AWS, Azure, and GCP icon sets are subject to their respective provider usage policies. ESIP SHALL include current versions at ship time and provide an update mechanism for subsequent version changes.

10. **UML Conformance Level:** UML 2.5.1 is the target. Not all UML diagram types support lossless XMI round-trip across all tools; import/export SHALL aim for structural fidelity with a documented delta list.
