# Predictive Pre-fetch Engine
## VectorVolt + EKOS — Speculative Memory Loading via Markov Chain Prediction

**Status:** Architecture Capture (Phase 4 — after behavioral fingerprinting is live)
**Captured:** 2026-07-09

---

## 1. The Core Insight

A cache is **reactive**. Data is loaded only *after* a miss. Every miss costs:

```
L3 miss → DRAM:   ~100 ns
DRAM miss → NVMe: ~10 µs
NVMe miss → cold: ~100 µs
```

We do not want a cache. We want a **Pre-fetch Buffer**: a speculative loader that moves data
into L3 *before* the client sends the query. When the request arrives, the data is already
in the CPU register pipeline.

This requires predicting the future. Database workloads are mathematically predictable because
they obey **temporal locality** (recently accessed data is accessed again), **spatial locality**
(nearby rows are accessed next), and **transactional correlation** (accessing `Order 123`
almost always precedes accessing `Customer 456`).

---

## 2. The Transaction Dependency Graph

EKOS maintains an in-memory **Transaction Dependency Graph** — a Markov Chain over primary keys.

```rust
pub struct TransactionDependencyGraph {
    // Directed graph: accessing node A → next access is node B with probability p
    // Stored as adjacency list of (target_pk, weight) pairs
    pub edges: DashMap<u64, Vec<(u64, f32)>>,

    // Temporal decay factor — applied to edge weights every 5 minutes
    pub decay_rate: f32,  // default: 0.5 (half-life = 5 min)
}
```

**How the graph is built:**

Every transaction processed by VectorVolt emits a sequence of PKs accessed:

```
T1 accessed: Customer(999) → Order(101) → LineItem(5001), LineItem(5002)
```

EKOS records edges:
- `Customer(999) → Order(101)`: weight += 1.0
- `Order(101) → LineItem(5001)`: weight += 1.0
- `Order(101) → LineItem(5002)`: weight += 1.0

After billions of transactions, edge weights converge to accurate transition probabilities.
`Customer(999) → Order(*)` has weight ~0.97. The graph knows: accessing a customer almost
certainly means the caller will next touch their orders.

**The Decay Factor:**

Workloads change. Black Friday access patterns are different from normal operation.
Every 5 minutes, EKOS applies:

```rust
for (_, weight) in edge.iter_mut() {
    *weight *= self.decay_rate;  // 0.5 per 5 min = half-life of 5 min
}
```

Stale edges fade. New patterns emerge and dominate within minutes.

---

## 3. Three-Tier Memory Hierarchy

Not all predictions are equally confident. We map confidence levels to memory tiers:

```
Tier 1: CPU L3 / 2MB Huge Pages   (probability > 95%)
  Latency:  < 10 ns
  Size:      5-10% of working set
  Pinned:    mlock() — OS cannot evict
  NUMA:      local to the CPU core serving the query

Tier 2: DRAM Buffer Pool           (probability 60-95%)
  Latency:  ~100 ns
  Size:      40-60% of working set
  Resident:  MAP_POPULATE mmap, but swappable under extreme pressure
  Managed:   EKOS pre-fetch threads on background cores

Tier 3: NVMe (on-disk)             (probability < 60%)
  Latency:  ~10 µs
  Size:      100% of dataset
  Access:    io_uring async reads, never blocking the hot path
```

**Promotion and demotion:**

```rust
// EKOS pre-fetch loop — runs on dedicated background CPU core
async fn prefetch_loop(graph: &TransactionDependencyGraph, tier1: &HugePagePool, tier2: &BufferPool) {
    loop {
        let predictions = graph.top_predictions(window_ms = 100);
        for (pk, probability) in predictions {
            if probability > 0.95 {
                tier1.pin(pk).await;       // mlock into L3
            } else if probability > 0.60 {
                tier2.prefetch(pk).await;  // async DRAM load
            }
            // Below 0.60: leave on NVMe, not worth the bandwidth
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
```

---

## 4. Query-Time Speculation

Beyond the background pre-fetch loop, EKOS also performs **per-query speculation**.

When VectorVolt's parser sees an incoming query, it sends the parsed AST to EKOS **before
executing it** (during the ~50 ns parsing phase). EKOS:

1. Extracts PKs from the query predicates
2. Looks up the dependency graph for each PK
3. Identifies the most likely *next* PKs
4. Launches a low-priority speculative read thread on a spare CPU core

By the time the main core finishes executing the current query and the client sends the next
request, the speculative core has already loaded the most likely next rows into Tier 2 or Tier 1.

```
Client: SELECT * FROM orders WHERE id = 101        ← main core executing
EKOS:   ORDER 101 → CUSTOMER 999 (probability 0.97) ← speculation: pre-load customer 999
Client: SELECT * FROM customers WHERE id = 999      ← arrives 200 µs later
        → Already in Tier 1. Cost: ~10 ns instead of ~100 ns
```

**Speculative promotion:**

If the same PK is speculatively loaded and then actually accessed twice within 1 second,
EKOS upgrades it from Tier 2 to Tier 1 (mlock).

---

## 5. Graph Neighborhood Pre-fetch

For graph queries (Depth-2: User → Friends → Friends' posts), we pre-build and cache
**entire traversal neighborhoods** as contiguous memory blobs.

```rust
pub struct HotSubgraph {
    pub root_node_id:  u64,
    pub depth:         u8,
    pub nodes:         Vec<NodeRecord>,   // contiguous, all depth-2 nodes
    pub edges:         Vec<(u64, u64)>,   // all edges within the neighborhood
    pub last_access:   u64,               // Unix timestamp
}
```

EKOS monitors the top 1% most-accessed graph nodes (by access frequency from the dependency
graph). For each of these "hot nodes", it pre-builds the HotSubgraph and stores it in a
**dedicated Hot Graph Buffer** (Tier 1, mlock'd).

**Query: "Find friends of User X" where User X is in the top 1%:**

Instead of traversing the CSR (which requires reading node_index + sequential edge scan),
VectorVolt reads the pre-joined HotSubgraph blob directly: **one sequential DRAM read**.

| Approach | Operation | Latency |
|---|---|---|
| Cold CSR traversal | index read + edge scan | ~400 ns |
| Hot subgraph (depth-2, 100 nodes) | one blob read from Tier 1 | **~500 ns** |
| Hot subgraph (depth-2, 1,000 nodes) | one blob read from DRAM | **~2 µs** |

The subgraph eliminates **all** random-access pointer chasing for covered queries.

**Invalidation:** When any node in a HotSubgraph is modified (INSERT edge, DELETE edge),
EKOS immediately marks the subgraph stale and rebuilds asynchronously.

---

## 6. Negative Caching — Knowing What NOT to Fetch

The Cuckoo Filter and Primary-Key Cache serve a second purpose: **negative prediction**.

If EKOS determines via the dependency graph that a PK *might* be accessed, it first checks
the Cuckoo Filter. If the filter says the PK does not exist (definitive `false` — no false
negatives in absence), EKOS **skips the pre-fetch entirely** and marks the prediction as a
"known miss". This prevents wasting bandwidth on loading rows that don't exist.

For random-access queries with no predictable pattern, EKOS will also detect low overall
prediction confidence and simply not pre-fetch — saving memory bandwidth for queries where
prediction accuracy is high.

---

## 7. Working Set Boundary

The pre-fetch engine cannot pre-load everything. If the working set exceeds available DRAM,
we rely on the Markov Chain to load only the **most likely** data.

```
If total working set > available DRAM:
  → Tier 1 (mlock):   top 5% most frequently predicted PKs
  → Tier 2 (DRAM):    next 40% by prediction probability
  → Tier 3 (NVMe):    remainder — loaded on-demand via io_uring

If total working set ≤ available DRAM:
  → All data resident in Tier 2
  → Tier 1 promotion for the hottest 5% only
```

The EKOS admin panel shows current working set coverage: what % of the last hour's queries
were served from Tier 1 / Tier 2 / Tier 3. This guides hardware sizing decisions.

---

## 8. Interaction with ARP (Autonomous Remediation Protocol)

The Predictive Pre-fetch Engine and ARP share the same dependency graph. When ARP detects
an anomaly (e.g., a specific query is consistently slow), it can inspect the pre-fetch engine
state to ask:

- Was the slow query a Tier 3 miss (data not in DRAM)?
- Did prediction fail for this PK (low graph weight)?
- Is the dataset growing past DRAM capacity?

ARP can then classify the anomaly as a **resource issue** (not a code bug) and escalate to
the admin panel with a recommendation to add RAM or adjust Tier 1 pin budget — instead of
synthesizing a patch for a non-existent code defect.

---

## 9. Crate Addition

```
ekos/crates/ekos-prefetch/
  src/
    dependency_graph.rs   ← Markov Chain, edge weights, temporal decay
    prefetch_loop.rs      ← background thread: Tier 1/2 promotion
    speculative.rs        ← per-query AST inspection, speculation thread
    hot_subgraph.rs       ← graph neighborhood pre-computation
    negative_cache.rs     ← Cuckoo Filter integration for miss prediction
  Cargo.toml
```

Integrated into `ekosd` as a background task. Shares CPU affinity with behavioral
fingerprinting (both run on the "observer" core group, separate from OLTP cores).

---

## 10. Phase Placement

```
Phase 2:   Cuckoo Filter + PK Hash Cache (FK validation — already in READ_PATH_ARCHITECTURE)
           Markov Chain dependency graph (lightweight, starts recording from day 1)

Phase 3:   Background pre-fetch loop (Tier 2 DRAM pre-loading)
           Negative caching integration

Phase 4:   Tier 1 mlock promotion (2MB Huge Pages pinning)
           Per-query AST-based speculation thread
           Hot subgraph pre-computation and invalidation

Phase 5:   Admin panel: working set coverage dashboard
           ARP integration: distinguish code bugs from resource misses
           Decay rate auto-tuning based on workload variability
```

---

*The goal is not to make cache hits more likely.*
*The goal is to make cache misses impossible for the queries that matter most.*
