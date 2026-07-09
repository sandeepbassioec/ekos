# Query Execution Model
## VectorVolt — Deterministic Algorithm Selection Over Cost-Based Planning

**Status:** Architecture Reference (Phase 2 foundations, all phases)
**Captured:** 2026-07-09

---

## 1. The Design Decision: No Cost-Model Optimizer

Every mainstream relational database has a query planner that:
1. Parses the query into a logical plan
2. Enumerates candidate physical plans (index scan vs table scan, nested loop vs hash join)
3. Estimates costs using column statistics and cardinality estimates
4. Selects the plan with the lowest estimated cost
5. Caches the plan for reuse

This process takes 50–500 µs before a single byte of data is read.

**VectorVolt deliberately does not have a cost-based optimizer.**

For sub-microsecond OLTP, a planner that costs 100 µs would dominate the entire transaction budget. The solution: make the algorithm selection a design-time decision, not a runtime decision. Every access pattern has exactly one correct algorithm. The "planner" is this document.

> *If you find yourself writing a cost model for VectorVolt, stop. You are adding latency, not removing it. Design the data layout so only one algorithm is possible for each access pattern.*

---

## 2. The Access Pattern Taxonomy

VectorVolt recognizes 7 distinct access patterns. Each maps to exactly one algorithm. No runtime selection occurs.

### 2.1 Primary Key Point Lookup

**Pattern:** `SELECT * FROM t WHERE id = ?`  
**Algorithm:** Cuckoo Hash → Row Descriptor → Micro-Partition  
**Latency:** ~120 ns  

```
Query: id = 7334892
         ↓
Cuckoo Hash index (L3-resident, Huge Pages, 9 bytes/row)
         ↓
Row Descriptor (u64: file_id | partition_id | offset — bit-packed)
         ↓
Micro-Partition mmap: seek to offset, read column projection
         ↓
Return result (zero copy — pointer into mmap'd region)
```

The Cuckoo Hash is the only data structure used for PK lookup. There is no decision to make.

### 2.2 Secondary Index Scan (Equality or Set Membership)

**Pattern:** `SELECT * FROM t WHERE status = 'active'`  
**Algorithm:** Roaring Bitmap → SIMD iterate → Row Descriptor batch → Micro-Partition bulk fetch  
**Latency:** ~500 ns per 1,000 matching rows  

```
Query: status = 'active'
         ↓
Roaring Bitmap index (one bitmap per distinct value of status)
         ↓
SIMD iterate (AVX2: 4 u64 per cycle; AVX-512: 8 u64 per cycle)
         ↓
Row Descriptor batch (all matching rows in sorted order)
         ↓
Micro-Partition bulk fetch (sequential within each partition — cache-friendly)
```

Roaring Bitmap's compressed representation ensures the entire index for a column fits in DRAM. SIMD iteration over the uncompressed bit array is the fastest known algorithm for set intersection and union on modern CPUs.

### 2.3 Range Scan

**Pattern:** `SELECT * FROM t WHERE created_at BETWEEN ? AND ?`  
**Algorithm:** Zone Map prune → binary search → SIMD stream  
**Latency:** ~3 µs per 10,000 rows after pruning  

```
Query: created_at BETWEEN 2026-01-01 AND 2026-06-30
         ↓
Zone Map index: for each Micro-Partition, compare [min, max] against [lo, hi]
  → Partitions entirely outside range: skip (zero I/O, zero CPU)
  → Partitions entirely inside range:  stream all rows (no per-row check)
  → Partitions partially overlapping:  binary search within partition, then stream
         ↓
SIMD stream remaining rows (columnar layout = sequential memory = hardware prefetch wins)
```

Zone Maps eliminate ~99% of Micro-Partitions for typical range queries. The remaining work is sequential memory access — the fastest possible pattern for the hardware prefetcher.

### 2.4 Foreign Key Validation

**Pattern:** `INSERT INTO orders(customer_id, ...) VALUES (?, ...)`  
**Algorithm:** Cuckoo Filter (L3) → PK Hash Cache (DRAM) — zero I/O  
**Latency:** ~150 ns  

```
INSERT: customer_id = 8834
         ↓
Cuckoo Filter (1 bit/row, L3-resident): does customer 8834 definitely NOT exist?
  → Filter says NO (definitely absent): reject immediately — ~10 ns
  → Filter says MAYBE: proceed to full lookup
         ↓
PK Hash Cache (9 bytes/row, DRAM): authoritative existence check
  → Not found: reject with FK constraint violation — ~60 ns
  → Found: FK valid, proceed with write — ~150 ns total
```

The Cuckoo Filter eliminates ~99.9% of false FK lookups before they reach DRAM. No disk I/O for FK validation under any circumstances.

### 2.5 Graph Traversal

**Pattern:** `GRAPH_TRAVERSE(root, path, depth, predicate)` or `.graph().from().traverse()`  
**Algorithm:** CSR (Compressed Sparse Row) sequential scan  
**Latency:** ~400 ns per 1,000 neighbors at depth 1  

```
Traversal: from customer 999, follow 'placed' → 'contains', depth 2
         ↓
CSR adjacency array for 'placed' relationship:
  [offset: 999] → [offset_start, offset_end] in the neighbor array
  Neighbor array [offset_start..offset_end] = list of order IDs
  → Sequential memory scan (hardware prefetcher handles this automatically)
         ↓
For each order ID: CSR lookup for 'contains'
  → Same sequential scan pattern, one level deeper
         ↓
Batch collect all leaf node Row Descriptors
         ↓
Micro-Partition bulk fetch (same as §2.2)
```

The key insight: CSR stores graph edges as sequential arrays, not pointer-linked nodes. Traversal is a sequential memory scan, not pointer-chasing. The hardware prefetcher loads the next 8 neighbors while the CPU processes the current 8 — effectively hiding DRAM latency for traversal.

### 2.6 Vector Similarity Search

**Pattern:** `SELECT * FROM t ORDER BY cosine_similarity(embedding, ?) DESC LIMIT 10`  
**Algorithm:** HNSW (Hierarchical Navigable Small World) approximate nearest neighbor  
**Latency:** ~5–50 µs depending on index size and ef_search parameter  

```
Query vector: [0.23, -0.41, 0.87, ...]  (768 dimensions)
         ↓
HNSW index: navigate the hierarchical graph from the entry point
  Layer L (sparse): greedy descent to the nearest node — log(n) hops
  Layer 0 (dense):  beam search with ef_search candidates
         ↓
Return top-K neighbors by cosine similarity (AVX2/AVX-512 FMA for distance computation)
```

HNSW is the only vector search algorithm used. There is no choice between exact and approximate — HNSW is approximate nearest neighbor. For exact search, use a full column scan with `cosine_similarity()` (returns exact results, costs O(n)).

### 2.7 Write Path (INSERT / UPDATE / DELETE)

**Pattern:** Any mutation  
**Algorithm:** WAL append → Ring Buffer → Delta Log → async index update  
**Latency:** ~200–900 ns end-to-end  

```
Mutation arrives at gRPC server
         ↓
CommandParser: parse, assign shard, assign priority (~10 ns)
         ↓
Ring Buffer: single writer enqueues CommandFrame (~5 ns — one cache line write)
         ↓
WAL Writer (dedicated core): append frame to segment, hash chain (~80 ns)
         ↓
MVCC Engine: allocate new version, update VersionChain (~50 ns)
         ↓
Index Writer: async update Cuckoo Hash, Roaring Bitmap, Zone Map (~100–600 ns)
         ↓
GroupCommit: batch N writes into 1 fsync (~50 µs amortized across batch)
         ↓
Return fingerprint to client
```

Writes never block reads. Reads never block writes. Delta Logs hold pending writes that haven't yet been merged into Micro-Partitions — reads reconstruct the latest state by merging Delta Log + Micro-Partition at read time (Merge-on-Read).

---

## 3. Multi-Pattern Queries

Real queries combine multiple access patterns. VectorVolt executes them as a pipeline:

```sql
SELECT o.*, li.*
FROM orders o
JOIN line_items li ON li.order_id = o.id
WHERE o.customer_id = 999        -- PK lookup (§2.1) on customer → filter orders
  AND o.status = 'pending'       -- Secondary index scan (§2.2) on status
  AND o.created_at > '2026-01-01' -- Range scan (§2.3) on created_at
  AND li.sku = 'PART-7734'       -- Secondary index scan (§2.2) on sku
```

Execution pipeline:
```
1. Cuckoo Hash lookup: customer_id = 999 → Row Descriptors for customer's orders
2. Roaring Bitmap: status = 'pending' → bitmap of all pending orders
3. Intersect: customer orders ∩ pending orders → candidate set
4. Zone Map prune: created_at > '2026-01-01' → surviving partitions only
5. Roaring Bitmap: sku = 'PART-7734' → bitmap of line_items with that sku
6. Merge: surviving orders → fetch line_items via FK link (Cuckoo Filter validated)
7. Return: projected columns only (columnar layout = zero wasted bandwidth)
```

The pipeline is determined by the query structure, not by statistics. No cardinality estimates. No join order enumeration. The optimizer is the schema designer.

---

## 4. Join Algorithms

VectorVolt supports two join algorithms. The choice is **determined by the join type, not by statistics**:

### Hash Join (Phase 5)

Used for: equality joins on non-PK columns, large table × large table  
Mechanism: SIMD-vectorized hash build + probe  
Build phase: hash the smaller relation into a hash table (AVX2 scatter)  
Probe phase: for each row in the larger relation, SIMD probe the hash table  

### Sort-Merge Join (Phase 5)

Used for: range joins, joins on already-sorted columns  
Mechanism: sorted columnar merge via SIMD comparison  

### Nested Loop Join (never used)

Nested loop join is O(n×m). At VectorVolt's row counts and latency targets, nested loop is never the correct algorithm. It does not exist in the implementation.

---

## 5. Why This is Correct for OLTP

The cost-based optimizer is a solution to a problem VectorVolt doesn't have:

| Problem | Cost-Based Optimizer Solves | VectorVolt Solution |
|---|---|---|
| Unknown query patterns | Adapts plan at runtime | Workload is OLTP — patterns are known at design time |
| Unknown data distribution | Cardinality estimates from statistics | Zone Maps give exact partition-level stats; Roaring Bitmap cardinality is O(1) |
| Multiple index choices | Picks cheapest index | Each column has exactly one index type (Roaring Bitmap or none) |
| Join order uncertainty | Enumerate join orders | OLTP joins are foreign-key driven — the join graph is the schema |

The cost of a cost-based optimizer is paid on every query. The cost of a wrong access pattern is paid forever. VectorVolt makes the access pattern correct at design time, paying zero runtime cost.

---

## 6. Statistics Without a Planner

VectorVolt maintains statistics — not for plan selection, but for observability and the HITL Index Advisor:

- **Zone Maps:** per-Micro-Partition min/max for every column — used for range pruning, not plan selection
- **Roaring Bitmap cardinality:** O(1) count of distinct values — exposed via admin panel
- **MVCC telemetry:** per-row `latch_waits`, `index_hits`, `latency_ns` — used for Index Advisor ML model
- **Query fingerprint histogram:** p50/p95/p99 latency per access pattern — exposed via Prometheus

The HITL Index Advisor (Phase 4) uses these statistics to recommend new secondary indexes — not to change existing query plans, but to change the schema so the optimal fixed algorithm applies.

---

## 7. Phase Placement

```
Phase 2:  §2.1 PK lookup, §2.2 secondary index, §2.3 range scan, §2.4 FK validation
          §2.7 write path, §2.6 HNSW vector search
Phase 3:  §2.5 graph traversal (CSR engine)
Phase 4:  §3 multi-pattern query pipeline (query compiler)
Phase 5:  §4 hash join + sort-merge join (vectorized, SIMD)
Phase 6+: Query fingerprint → access pattern profiling → adaptive pre-fetch tuning
```

---

*The fastest query is one that never touches memory it doesn't need.*
*Zone Maps are the filter. CSR is the traversal. SIMD is the scan.*
*There is nothing left to optimize.*
