# Read Path Architecture
## VectorVolt — Nanosecond Read Algorithms for Billions of Records

**Status:** Architecture Capture (implement Phase 2 storage engine)
**Captured:** 2026-07-09

---

## 1. The Read Problem

Writes in VectorVolt are solved: kernel-bypass network path delivers requests to the WAL in nanoseconds.
The read path is harder. Every read must answer:

```
WHERE is the data? (index lookup)
IS it in memory? (buffer management)
WHAT version? (MVCC snapshot)
IS it valid? (FK constraints without I/O)
```

All of this must complete in under 200 nanoseconds for point lookups.

This document defines the data structures and algorithms that make that possible for billions of
relational rows and billions of graph nodes.

---

## 2. The Fundamental Rule

> Never access disk on the hot read path. Never chase a pointer that isn't already in L3 cache.

Every data structure below is designed to be **cache-line-conscious**: fitting the relevant data
into 64-byte cache lines, using contiguous arrays instead of pointer-linked structures, and
enabling SIMD vectorization on the CPU's integer units.

---

## 3. Primary Key Point Lookup — O(1), ~120 ns

### Structure: Cuckoo Hash Map + Row Descriptor

Traditional B-Trees have O(log n) pointer-chasing lookups that blow the cache. We replace them
with a **Cuckoo Hash Map** over all primary keys.

```rust
// The Row Descriptor is a bit-packed u64 — NOT a pointer
// (file_id: 12 bits, partition_id: 20 bits, offset: 32 bits)
pub struct RowDescriptor(u64);

// The primary key index — pinned to 2MB Huge Pages
pub struct PrimaryKeyIndex {
    map: CuckooHashMap<u64, RowDescriptor>,
}
```

**Why Cuckoo, not Robin Hood or Linear Probing:**
- O(1) worst-case lookup (B-Tree is O(log n), Robin Hood is O(1) average)
- Deletion support (needed for tombstone clearing)
- Fits well in L3 cache for tables up to ~1 billion rows

**The Row Descriptor** is not a pointer. It encodes the physical location as a packed integer.
No pointer dereferencing. One DRAM access resolves the location.

**Latency breakdown:**
- Hash `id=101` → slot index: **~10 ns** (L3 cache)
- Read Row Descriptor at slot: **~10 ns** (same cache line)
- Map to Micro-Partition, read row data: **~100 ns** (DRAM)

**Total: ~120 ns** for a point lookup on a table with 10 billion rows.

---

## 4. Secondary Index Lookups — ~500 ns for 1,000 matching rows

### Structure: Roaring Bitmap Inverted Index

Traditional B-Tree secondary indexes require traversal. We store inverted indexes as
**Roaring Bitmaps**: compressed bitsets that exploit run-length encoding and SIMD traversal.

```rust
pub struct SecondaryIndex {
    // key: dictionary-encoded column value
    // value: compressed bitmap of all matching Row Descriptors
    map: HashMap<u32, RoaringBitmap>,
}
```

**Why Roaring Bitmap:**
- 2-8x more compressed than plain bitmaps
- Supports AND / OR / NOT in microseconds using SIMD
- CPU can process 128 bits (two u64 chunks) per cycle with AVX-512

**Query: `WHERE customer_id = 999`**

1. Hash lookup for value `999` in dictionary: **~30 ns**
2. Retrieve Roaring Bitmap: **~20 ns** (L3 hit, bitmap is resident)
3. SIMD iterate over bitmap, collect Row Descriptors: **~2 µs** for 10,000 matches
4. Bulk read rows from Micro-Partitions: **~500 ns** (DRAM gather)

**Total: < 3 µs** for 10,000 matching rows.

---

## 5. Range Scans — ~3 µs for 10,000 rows

### Structure: Zone Maps + Binary Search + SIMD Streaming

For range queries (`WHERE order_date BETWEEN X AND Y`), we avoid scanning all partitions using
**Zone Maps**: per-partition min/max summaries.

```rust
pub struct ZoneMap {
    pub partition_id: u32,
    pub min_value:    i64,
    pub max_value:    i64,
    pub min_pk:       u64,
    pub max_pk:       u64,
    pub row_count:    u32,
}

// All zone maps stored as a flat array — fits in L3 cache
pub struct ZoneMapIndex(Vec<ZoneMap>);
```

**Query: `WHERE order_date > 1735689600`**

1. SIMD scan all Zone Maps for `max_value >= threshold`: **~50 ns** (eliminates 99% of partitions)
2. For surviving partitions: binary search on the dictionary-encoded date column: **~200 ns**
3. SIMD stream matching row range: **~2.7 µs** for 10,000 rows at DRAM bandwidth

**Total: ~3 µs** for a range scan returning 10,000 rows.

---

## 6. Graph Traversal — ~400 ns per neighborhood

### Structure: Compressed Sparse Row (CSR)

Graph databases (Neo4j-style traversals) are the worst case for cache because they involve random
pointer-chasing. We eliminate pointer-chasing by storing edges as **Compressed Sparse Row (CSR)**:
a single, giant, contiguous array where all outgoing edges for each node are stored sequentially.

```rust
pub struct CsrGraph {
    // node_index[node_id] = start offset in edges array
    pub node_index:    Vec<u64>,   // N+1 entries, one per node
    // edges: all outgoing neighbor IDs, contiguous per node
    pub edges:         Vec<u64>,   // total edge count entries
}

pub struct ReverseCsrGraph {
    // same structure but for INCOMING edges (needed for CASCADE DELETE)
    pub node_index:    Vec<u64>,
    pub edges:         Vec<u64>,
}
```

**Traversal: "Find all friends of User X"**

1. Read `node_index[X]` and `node_index[X+1]` to get edge slice: **~20 ns** (likely L3 hit)
2. Sequential read of `edges[start..end]`: scales at DRAM bandwidth (~20 GB/s)
   - 1,000 neighbors = 8 KB = **~400 ns**
   - 10,000 neighbors = 80 KB = **~4 µs**

Sequential reads saturate the prefetcher. There are **zero cache misses** for the traversal
because the edges are physically contiguous, not pointer-scattered.

### Incoming Edge Traversal (CASCADE DELETE)

`ReverseCsrGraph` stores the same structure for incoming edges. Finding all nodes that point
to node X is identical to finding outgoing edges: one index read + sequential scan.

---

## 7. Foreign Key Validation — ~150 ns, Zero I/O

For writes with FK constraints (`INSERT INTO orders WHERE customer_id = 999 must exist`), we
must validate existence **without reading the actual row**. Two-tier:

### Tier 1: Cuckoo Filter (L3-resident, probabilistic)

```rust
// 8-10 bits per key, 99.99% accuracy, supports deletion
// For 10B rows: ~12 GB — fits in L3 on a 256-core NUMA node with large L3
pub struct ExistenceFilter {
    filter: CuckooFilter<u64>,
}
```

- Filter says **NO** → immediate abort, zero I/O. Latency: **~10 ns**
- Filter says **YES** → proceed to Tier 2

### Tier 2: Primary-Key Hash Cache (DRAM, authoritative)

```rust
// Minimal hash table: key (8B) + active flag (1B) = 9B per row
// For 10B rows: ~90 GB DRAM
pub struct PrimaryKeyCache {
    entries: Vec<(u64, u8)>,  // (pk, is_active)
}
```

- Hash lookup: **~100 ns** (single DRAM round-trip)
- Returns `Active = true / false` — authoritative, no false positives

**Total FK validation: ~150 ns.** No buffer pool access. No disk I/O.

---

## 8. Merge-on-Read Delta Logs — Writes Never Block Reads

For updates, we **do not** read the current row before writing the delta. Reads reconstruct the
current state by combining the base version with pending deltas at read time.

```rust
// Per-core write log — never touches the main row store
pub struct DeltaLog {
    entries: Vec<DeltaEntry>,
}

pub struct DeltaEntry {
    pub txn_id:     u64,
    pub pk:         u64,
    pub delta_kind: DeltaKind,  // Update(field, value) | Delete | Insert
    pub timestamp:  u64,
}
```

**Write path:** Append `(txn_id, pk, delta)` to the per-core log. **~50 ns**.
No read of the existing row. No lock contention with concurrent reads.

**Read path reconstruction:**
1. Point-lookup base version via Cuckoo Hash Map
2. Scan the small delta log for any pending deltas on this PK (typically 0-3 entries)
3. Apply deltas via bitwise operations: **~20 ns**

The base version was pre-loaded by the Predictive Pre-fetch Engine (see companion document).
When it arrives, the data is already in L3. The delta scan costs a few nanoseconds.

---

## 9. CASCADE DELETE — ~7 µs for 10 Million Children

**Scenario:** `DELETE FROM customers WHERE id = 999`, cascade to all orders.

```
Step 1 — FK validation (100 ns):
  Confirm customer 999 exists via Primary-Key Cache

Step 2 — Child discovery (1-5 µs):
  Query Roaring Bitmap on orders.customer_id = 999
  Returns bitmap of all affected order_ids

Step 3 — Bulk soft-delete (2 µs):
  For customer 999: append Delete Tombstone to DeltaLog
  For each order_id in bitmap: set Deleted=1 in Primary-Key Cache
  All done via SIMD iteration over the bitmap — no row reads

Step 4 — Background compaction (async, off hot path):
  A low-priority background thread rewrites Micro-Partitions
  to permanently remove tombstoned rows
```

**Total visible latency to the client: ~7 µs** for 10 million cascade-deleted rows.
Physical cleanup is deferred and invisible.

---

## 10. Data Layout — Micro-Partitioned Columnar Vectors

All row data is stored in **Micro-Partitions**: fixed-size (default: 1M rows) columnar files.

Within each Micro-Partition:
- Each column is stored as a contiguous array (Struct of Arrays, not Array of Structs)
- String/enum columns are **dictionary-encoded**: store a u32 code, not the string
- Numeric columns stored as fixed-width integers (no variable-length encoding)
- Partitions are `mmap`'d with `MAP_POPULATE` — OS pre-faults all pages into physical RAM

```
Micro-Partition layout:
  [ZoneMap: 64B]
  [Column 0 data: order_id u64 × 1M]
  [Column 1 data: customer_id u32 × 1M]
  [Column 2 data: amount_cents i64 × 1M]
  [Column 3 codes: status_code u8 × 1M]
  [Dictionary: status_code → "PENDING" | "SHIPPED" | "CANCELLED"]
```

A full scan of one column in a 1M-row Micro-Partition: **~500 µs** at DRAM bandwidth.
SIMD vectorization processes 8-16 values per CPU cycle.

---

## 11. Latency Summary

| Operation | Data Volume | Algorithm | Latency |
|---|---|---|---|
| Primary key point lookup | 1 row | Cuckoo Hash + Row Descriptor | **~120 ns** |
| FK existence check | 1 ID | Cuckoo Filter + PK Hash Cache | **~150 ns** |
| Secondary index lookup | 1,000 rows | Roaring Bitmap + SIMD Gather | **~500 ns** |
| Range scan | 10,000 rows | Zone Map + Binary Search + SIMD | **~3 µs** |
| Graph outgoing traversal | 1,000 edges | CSR Sequential Scan | **~400 ns** |
| Graph incoming traversal | 1,000 edges | Reverse CSR Sequential Scan | **~400 ns** |
| Range scan + join | 1,000 rows | SIMD Gather + Hash Join | **~2 µs** |
| Cascade delete discovery | 10M children | Roaring Bitmap SIMD iterate | **~5 µs** |
| Cascade delete apply | 10M tombstones | PK Cache bulk update | **~2 µs** |

---

## 12. What We Never Do on the Hot Read Path

- Never call `malloc()` during query execution — all buffers pre-allocated
- Never acquire a mutex for reads — MVCC snapshots are read-only, lock-free
- Never parse JSON or variable-length strings — all strings are dictionary-encoded
- Never use virtual function calls — branch prediction unfriendly
- Never read a full row to validate FK existence — Cuckoo Filter handles it
- Never chase a pointer through more than one DRAM round-trip — Row Descriptor is the last pointer

---

## 13. Phase Placement

```
Phase 2:  Cuckoo Hash primary index, Row Descriptor, Micro-Partition mmap
          Roaring Bitmap secondary index
          Zone Map + binary search range scans
          PK Hash Cache + Cuckoo Filter for FK validation

Phase 3:  CSR graph traversal engine
          Reverse CSR for incoming edges
          DeltaLog Merge-on-Read

Phase 4:  SIMD vectorization passes (AVX-512 gather)
          Dictionary encoding for all string columns
          Cascade delete via Roaring Bitmap child discovery

Phase 5:  Full MVCC snapshot isolation on all read paths
          Columnar vectorized joins (hash join, sort-merge join)
```

---

*Every algorithm above is cache-line-conscious, branchless, and SIMD-vectorizable.*
*The goal is not to make queries fast. The goal is to make cache misses impossible.*
