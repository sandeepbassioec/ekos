# Graph-Relational Bridge
## VectorVolt — Property Graph Views Over Relational Data

**Status:** Architecture Capture (Phase 4)
**Captured:** 2026-07-09

---

## 1. The Problem

Most systems force a choice: relational (joins, foreign keys, SQL) or graph (traversals,
path queries, Cypher). This split creates friction:

- A parts catalog lives in relational tables (OEM → Model → Part)
- A supplier network is a graph (Supplier → Distributor → Dealer → Customer)
- An insurance claim graph is mixed (Policy has relational attributes; Fraud rings are graph)

Maintaining two separate systems (PostgreSQL + Neo4j) means dual ingestion, dual consistency,
dual backup, and dual query interfaces.

**VectorVolt's answer:** Relational data IS the source of truth. Graph views are derived
automatically from FK relationships. One storage engine. One query interface. Both paradigms.

---

## 2. Automatic Graph View Generation

VectorVolt inspects FK relationships in the schema and automatically constructs a graph
view with zero additional configuration:

```sql
-- Original relational schema:
CREATE TABLE customers (id BIGINT PRIMARY KEY, name TEXT, region TEXT);
CREATE TABLE orders    (id BIGINT PRIMARY KEY, customer_id BIGINT REFERENCES customers(id), total DECIMAL);
CREATE TABLE line_items(id BIGINT PRIMARY KEY, order_id   BIGINT REFERENCES orders(id),    sku TEXT);

-- VectorVolt automatically derives the graph:
-- customers --[placed]--> orders --[contains]--> line_items
-- (edge direction follows FK direction)
```

The graph view is not a copy. It is a **live projection** over the same Micro-Partitions
that serve relational queries. No data duplication. Schema changes propagate automatically.

---

## 3. Graph Query Interface

VectorVolt exposes a graph query API alongside its SQL interface:

```rust
// Graph traversal via the Rust client SDK:
let result = client
    .graph()
    .from("customers", customer_id)
    .traverse("placed")       // follow the orders FK edge
    .traverse("contains")     // follow the line_items FK edge
    .depth(2)
    .filter("line_items.sku = 'PART-7734'")
    .collect()
    .await?;

// Equivalent "hybrid" SQL extension:
SELECT * FROM GRAPH_TRAVERSE(
    root      => (customers WHERE id = 999),
    path      => 'placed.contains',
    depth     => 2,
    predicate => "line_items.sku = 'PART-7734'"
);
```

Under the hood, VectorVolt's CSR graph index (see READ_PATH_ARCHITECTURE.md) serves this
query as a sequential scan — no joins, no nested loops, no pointer-chasing.

---

## 4. Cross-Dataset Graph Queries

The most powerful use case: running graph queries that connect multiple unrelated relational
datasets via a discovered or declared relationship.

```sql
-- Declare a cross-table relationship (not a FK — a semantic link):
CREATE GRAPH_EDGE connected_supplier
    FROM parts(manufacturer_id)
    TO  suppliers(id)
    WEIGHT = suppliers.reliability_score;

-- Now traverse: which parts have unreliable suppliers who also supply another OEM's parts?
SELECT GRAPH_TRAVERSE(
    root      => (oems WHERE id = 'HONDA'),
    path      => 'makes.uses_part.connected_supplier',
    depth     => 3,
    predicate => "suppliers.reliability_score < 0.7"
);
```

This returns all Honda parts whose suppliers also supply another OEM — impossible in
pure relational SQL without 4-way joins, trivial as a graph traversal.

---

## 5. Storage: CSR Projection Layer

The graph view uses the same CSR (Compressed Sparse Row) structure from READ_PATH_ARCHITECTURE.md,
but the CSR is **derived from FK relationships at build time**, not hand-authored:

```
Schema change detected (new FK or FK dropped)
         ↓
CSR Rebuild Workflow (background, does not block OLTP)
         ↓
New CSR written to Micro-Partition file
         ↓
Atomic pointer swap — old CSR evicted, new CSR live
         ↓
Admin panel shows: "Graph index rebuilt: 2.3M edges, 0.4s"
```

For schemas with millions of rows and complex FK graphs, the CSR rebuild is a background
workflow that runs off the hot path. During the rebuild, the old CSR serves queries until
the new one is ready.

---

## 6. Use Cases

| Domain | Relational Source | Graph Query |
|---|---|---|
| Auto parts (OEM) | `parts`, `models`, `oems`, `suppliers` | "All parts for Honda Civic 2022 with unreliable suppliers" |
| Insurance | `policies`, `claims`, `claimants` | "Fraud ring detection — claimants sharing addresses" |
| Financial | `accounts`, `transactions`, `counterparties` | "Transaction chain from Account A to Account B in ≤ 3 hops" |
| Collision repair | `vehicles`, `estimates`, `repair_shops` | "All repair shops that serviced vehicles from the same accident" |
| Supply chain | `factories`, `distributors`, `dealers` | "End-to-end supply path for Part SKU-7734" |

---

## 7. Phase Placement

```
Phase 3:   Basic graph projection: CSR auto-derived from FK relationships
           Depth-N traversal via CSR (using existing CSR engine from Phase 2)
           Graph query extension to the client SDK

Phase 4:   Cross-table GRAPH_EDGE declarations (semantic links beyond FK)
           GRAPH_TRAVERSE SQL extension
           Weighted graph traversals (edge weights from column values)
           CSR Rebuild Workflow for schema changes

Phase 5:   Graph + relational hybrid queries in one planner
           Neighborhood pre-fetch integration (PREDICTIVE_PREFETCH_ENGINE)
           Graph analytics (path counting, connectivity, basic centrality)
```
