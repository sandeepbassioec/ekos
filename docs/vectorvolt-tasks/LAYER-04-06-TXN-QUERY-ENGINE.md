# Layer 4 — Transactions  ·  Layer 5 — Query  ·  Layer 6 — Engine & Partitioning
Task-level traceability. Status: ✅ done · 🟡 partial · ⬜ pending · 🧊 env-gated · 🔬 research

Source docs read in full: `VECTORVOLT_FEATURE_MATRIX.md`, `vision/QUERY_EXECUTION_MODEL.md`,
`vision/VECTORVOLT_LOCKED_DECISIONS.md`, `vision/VECTORVOLT_CORE_ARCHITECTURE.md` §4/§7/§8.3.
Code audited line-by-line: `vectorvolt-txn/src/*.rs` + `tests/`, `vectorvolt-query/src/*.rs` +
`tests/`, `vectorvolt-engine/src/*.rs` + `tests/`, `vectorvolt-storage/src/{column,reader}.rs`,
`vectorvolt-index/src/{row_descriptor,secondary_index,zone_map}.rs`, `vvctl/src/commands/{table,bench}.rs`.

## Rollup

| Layer | Features | Tasks total | ✅ | 🟡 | ⬜ | 🧊 |
|---|---|---|---|---|---|---|
| Layer 4 — Transactions (VV-TXN-001..021) | 21 | 122 | 7 | 0 | 14 | 0 |
| Layer 5 — Query Execution (VV-QRY-001..020) | 20 | 107 | 5 | 1 | 14 | 0 |
| Layer 6 — Engine & Partitioning (VV-ENG-001..022) | 22 | 138 | 4 | 1 | 17 | 0 |
| **Total** | **63** | **367** | **16** | **2** | **45** | **0** |

**Note on VV-ENG-004:** downgraded ✅ → 🟡 relative to the matrix during this audit. See the
feature's own entry for the code-verified reason (compaction exists only on an unmerged branch).
The matrix should be corrected or the branch merged.

---

# LAYER 4 — TRANSACTIONS / CONCURRENCY (`vectorvolt-txn`)

## VV-TXN-001 — WAL-first write ordering  ✅
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §5 (WAL); QUERY_EXECUTION_MODEL.md §2.7 write path

| Task | Status | Detail |
|---|---|---|
| VV-TXN-001.1 | ✅ | `insert()` calls `wal.append()` before any index mutation — `manager.rs:110-115` |
| VV-TXN-001.2 | ✅ | `delete()` calls `wal.append()` before `CuckooMap` removal — `manager.rs:155-156` |
| VV-TXN-001.3 | ✅ | Flush appends a WAL checkpoint frame after the partition is written — `manager.rs:195-199` |
| VV-TXN-001.4 | ✅ | Integration test proves WAL frame count == mutation count (2 inserts + 1 delete = 3 frames) — `txn_integration.rs:144-166` |
| VV-TXN-001.5 | ⬜ | `WalConfig::default().with_fsync(false)` is hardcoded in `TableEngine::open` (`engine.rs:55,91`) — no API path exists to enable fsync; needs to become a config knob (VV-ENG-017) |
| VV-TXN-001.6 | ⬜ | No test proves behavior when the process dies between WAL append and index write (e.g. `manager.rs:115-121`) — durability today relies entirely on VV-TXN-006 rebuild-on-reopen, untested for this exact race |

## VV-TXN-002 — Write buffer + threshold auto-flush  ✅
**Spec:** CORE_ARCHITECTURE §5 (buffering before flush)

| Task | Status | Detail |
|---|---|---|
| VV-TXN-002.1 | ✅ | `WriteBuffer::push` returns `true` at capacity — `buffer.rs:22-25`, tested `buffer.rs:51-62` |
| VV-TXN-002.2 | ✅ | `TxnManager::insert` auto-triggers `flush()` when the buffer hits threshold — `manager.rs:134-138` |
| VV-TXN-002.3 | ✅ | Test `auto_flush_on_threshold` confirms exactly one `.vvsst` file after 3 inserts at threshold=3 — `txn_integration.rs:122-142` |
| VV-TXN-002.4 | ⬜ | Buffer capacity is fixed at construction time (`TxnManager::new(..., flush_threshold)`) — no runtime resize; blocked on VV-ENG-018 hot-reload |
| VV-TXN-002.5 | ⬜ | No backpressure: `insert()` blocks synchronously on `flush()` when the buffer fills (`manager.rs:135-138`) instead of applying LD-BATCH adaptive sizing or LD-BACK token-bucket rejection (blocked by VV-WAL-011/012) |
| VV-TXN-002.6 | ⬜ | Buffer is row-major `Vec<(u64, Vec<u64>)>` (`buffer.rs:7`) — every row heap-allocates its own `Vec<u64>`; not arena-backed (LD-GC), tracked under VV-TXN-019 |

## VV-TXN-003 — Index maintenance on insert  ✅
**Spec:** CORE_ARCHITECTURE §4.1 (Index Writer)

| Task | Status | Detail |
|---|---|---|
| VV-TXN-003.1 | ✅ | `CuckooMap` gets a `PENDING_FILE_ID` `RowDescriptor` at insert time — `manager.rs:117-121` |
| VV-TXN-003.2 | ✅ | `SecondaryIndex` updated for every non-PK column via `col_value_hash` — `manager.rs:123-129` |
| VV-TXN-003.3 | ✅ | `CuckooFilter` insert for FK existence — `manager.rs:131-132` |
| VV-TXN-003.4 | ✅ | Test `insert_updates_index` asserts `pk_index` + `fk_filter` are populated — `txn_integration.rs:32-46` |
| VV-TXN-003.5 | ⬜ | `col_value_hash` (`manager.rs:273-279`) is an ad hoc multiplicative mix with no documented collision-rate test for `SecondaryIndex` bucket keys |
| VV-TXN-003.6 | ⬜ | Index maintenance runs synchronously inline with `insert()` — CORE_ARCHITECTURE §4.1's diagram calls for an "Index Updater (dedicated core)" running asynchronously; today it blocks the caller on the same thread |

## VV-TXN-004 — Delete (index removal + WAL tombstone)  ✅
**Spec:** QUERY_EXECUTION_MODEL.md §2.7

| Task | Status | Detail |
|---|---|---|
| VV-TXN-004.1 | ✅ | `delete()` rejects an unknown pk with `PkNotFound` before writing anything — `manager.rs:151-152` |
| VV-TXN-004.2 | ✅ | WAL tombstone frame via `Mutation::Delete` — `manager.rs:155` |
| VV-TXN-004.3 | ✅ | `CuckooMap.remove(pk)` — `manager.rs:156` |
| VV-TXN-004.4 | ✅ | Tests `delete_removes_from_index` / `delete_missing_pk_errors` — `txn_integration.rs:63-82` |
| VV-TXN-004.5 | ⬜ | Documented gap in the code itself (`manager.rs:148-149`): delete does not remove row bytes from an already-flushed `.vvsst` — the row is orphaned until compaction (VV-ENG-004/VV-STO-008) runs; no tombstone bitmap exists yet to tell compaction which rows to drop |
| VV-TXN-004.6 | ⬜ | No `update()` exists at all — an UPDATE today is only achievable as delete+insert, which permanently orphans the old row's storage slot (same issue as .5) |

## VV-TXN-005 — Flush → partition + descriptor rebuild  ✅
**Spec:** CORE_ARCHITECTURE §5

| Task | Status | Detail |
|---|---|---|
| VV-TXN-005.1 | ✅ | Drains `WriteBuffer`, builds one `MicroPartition` column at a time — `manager.rs:168-187` |
| VV-TXN-005.2 | ✅ | Writes `{file_id:05}_{partition_id:05}.vvsst` — `manager.rs:190-193` |
| VV-TXN-005.3 | ✅ | Appends a WAL checkpoint text record — `manager.rs:196-199` |
| VV-TXN-005.4 | ✅ | Rebuilds `RowDescriptor`s from `PENDING` → real `(file_id, partition_id, offset)` — `manager.rs:206-210` |
| VV-TXN-005.5 | ✅ | Updates `ZoneMap` with PK min/max for the new partition — `manager.rs:213-215` |
| VV-TXN-005.6 | ✅ | Test `flush_writes_vvsst_file` validates file readback + descriptor rewrite — `txn_integration.rs:86-113` |
| VV-TXN-005.7 | ⬜ | The WAL checkpoint record (`manager.rs:196-198`) is an ad hoc formatted string (`"flush file_id=... partition_id=..."`), not a typed `Mutation` variant — nothing parses it back on replay; should become `Mutation::Checkpoint{..}` so recovery tooling doesn't need string parsing |
| VV-TXN-005.8 | ⬜ | **Discrepancy:** `TxnManager.zone_map` (`vectorvolt_index::ZoneMap`, keyed on PK min/max only, `manager.rs:50,213-215`) is written on every flush but never *read* anywhere in `vectorvolt-txn`, `vectorvolt-engine`, or `vectorvolt-query`. Actual range-scan pruning (`query/engine.rs:40-44`, `vectorvolt-engine/engine.rs:200-204`) calls `PartitionReader::zone_min_max(col_idx)` — a separate, per-column zone table stored directly in the `.vvsst` header (`reader.rs:192-199`, populated from `ColumnValues::zone_min_max()` at write time). `TxnManager`'s own `ZoneMap` field appears to be dead weight / a duplicate source of truth that needs reconciling or removing |

## VV-TXN-006 — Index rebuild from partitions on reopen  ✅
**Spec:** CORE_ARCHITECTURE §5 (recovery)

| Task | Status | Detail |
|---|---|---|
| VV-TXN-006.1 | ✅ | `rebuild_pk_index_from_partitions` iterates every `PartitionReader`, reads column 0, rebuilds `CuckooMap` + `CuckooFilter` — `manager.rs:234-252` |
| VV-TXN-006.2 | ✅ | `TableEngine::open` calls this after loading on-disk partitions — `engine.rs:85` |
| VV-TXN-006.3 | ⬜ | No test in `txn_integration.rs` directly exercises `rebuild_pk_index_from_partitions` — only indirectly covered via `vectorvolt-engine`'s restart path; add an explicit unit test |
| VV-TXN-006.4 | ⬜ | Rebuild assumes column 0 is always the PK (`reader.read_column(0)`, `manager.rs:241`) with no cross-check against `TxnManager.schema[0]` |
| VV-TXN-006.5 | ⬜ | O(total rows on disk) full scan on every process restart — no persisted index snapshot; will not scale once large tables exist (relates to VV-STO-017 CoW snapshots) |

## VV-TXN-007 — WAL delete replay on reopen  ✅
**Spec:** CORE_ARCHITECTURE §5 (recovery)

| Task | Status | Detail |
|---|---|---|
| VV-TXN-007.1 | ✅ | `TableEngine::open` replays the WAL and applies only `Mutation::Delete` frames to `pk_index` — `engine.rs:91-99` |
| VV-TXN-007.2 | ✅ | Code comment documents the insert-record-skip rationale — `engine.rs:87-90` |
| VV-TXN-007.3 | ⬜ | Gap named in the code's own comment: WAL insert records for rows still pending in the write buffer at process death are silently lost on restart, never replayed into a fresh buffer — this violates LD-REPLAY's "same WAL log → identical state" guarantee whenever any insert never reached a flush |
| VV-TXN-007.4 | ⬜ | No test exercises "insert (no flush) → kill process → reopen → assert row status", documenting the current lossy behavior explicitly or proving a fix once made |
| VV-TXN-007.5 | ⬜ | WAL is never truncated/compacted after successful delete replay — unbounded growth across restarts (blocked by VV-WAL-013 log compaction) |

## VV-TXN-008 — LMAX Disruptor ring buffer (single-writer, lock-free)  ⬜
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §4.1
**Blocked by:** VV-HAL-002/003 (cache-line alignment / false-sharing avoidance — delivered in S1a per the matrix, not yet consumed here)

Verified: today's ingestion path is `write_buf: Mutex<WriteBuffer>` (`manager.rs:52`) — a `parking_lot::Mutex`, not a lock-free ring buffer.

| Task | Status | Detail |
|---|---|---|
| VV-TXN-008.1 | ⬜ | Design a 64-byte cache-aligned `CommandFrame` (per CORE_ARCHITECTURE §3.2 example), replacing today's `(u64, Vec<u64>)` tuple row |
| VV-TXN-008.2 | ⬜ | Choose ring size (power-of-2 slots) and modulo-free index masking |
| VV-TXN-008.3 | ⬜ | Implement single-producer claim/publish sequence counters (CAS-free, single writer) |
| VV-TXN-008.4 | ⬜ | Implement multi-consumer read-cursor tracking (WAL writer, index updater as independent consumers) |
| VV-TXN-008.5 | ⬜ | Replace `Mutex<WriteBuffer>` in `TxnManager` with the ring buffer as the ingestion path |
| VV-TXN-008.6 | ⬜ | Wire the three wait strategies (BusySpin/Yielding/Blocking, CORE_ARCHITECTURE §4.3) behind a configurable trait |
| VV-TXN-008.7 | ⬜ | Loom concurrency test proving no lost/duplicated writes under producer/consumer interleaving (VV-TST-016) |
| VV-TXN-008.8 | ⬜ | Criterion benchmark: insert latency before/after (no benchmark harness exists yet for this crate — blocked by VV-TST-007) |

## VV-TXN-009 — One-writer-per-shard model  ⬜
**Blocked by:** VV-ENG-006 (multi-table catalog — "shard" has no explicit meaning yet), VV-TXN-008

| Task | Status | Detail |
|---|---|---|
| VV-TXN-009.1 | ⬜ | Define what a "shard" is at this layer — today 1 `TxnManager` == 1 table, with no explicit shard abstraction or enforcement |
| VV-TXN-009.2 | ⬜ | Add a guard preventing two threads from concurrently calling `insert`/`delete`/`flush` on one `TxnManager` — today `write_buf`'s mutex incidentally serializes buffer pushes, but the WAL lock and index locks are separate, so two concurrent inserters can interleave WAL order and index-update order differently (a latent correctness gap) |
| VV-TXN-009.3 | ⬜ | Explicit test asserting "WAL order == index update order" under two concurrent inserters — does not exist today |
| VV-TXN-009.4 | ⬜ | Wire `shard_id` into routing once VV-ENG-011/013 partition strategy exists |

## VV-TXN-010 — MVCC version chains  ⬜
**Spec:** CORE_ARCHITECTURE §4.1 ("MVCC Engine" in the write pipeline diagram); LD-SNAP
**Blocked by:** VV-TXN-008 (ring buffer), VV-ENG-008 (type system — version headers need a typed row layout)

Verified: zero MVCC code exists in `vectorvolt-txn` today — `RowDescriptor` (`row_descriptor.rs`) encodes exactly one physical location per PK, with no version dimension at all.

| Task | Status | Detail |
|---|---|---|
| VV-TXN-010.1 | ⬜ | Design row-version header layout (begin_ts/end_ts/next-version pointer, plus the 7-byte telemetry from VV-TXN-012) |
| VV-TXN-010.1.1 | ⬜ | Decide begin_ts/end_ts width and epoch/wraparound handling — must be deterministic (LD-REPLAY forbids `Instant::now()`), so this needs a logical or injected clock from day one (see VV-TXN-021) |
| VV-TXN-010.2 | ⬜ | Design version-chain storage location — inline in the arena (VV-TXN-019) vs. a separate undo log — before touching the `.vvsst` format |
| VV-TXN-010.3 | ⬜ | Implement `VersionChain` append on UPDATE/DELETE (UPDATE doesn't exist at all yet — VV-TXN-004.6) |
| VV-TXN-010.4 | ⬜ | Implement version-visibility check: given a reader's snapshot timestamp, walk the chain to the visible version |
| VV-TXN-010.5 | ⬜ | Reconcile with `RowDescriptor`'s single-location model — needs either a new indirection level or a widened encoding (today's `u64` packs file_id:16/partition_id:16/row_offset:32 exactly full, `row_descriptor.rs:1-7`, no spare bits) |
| VV-TXN-010.6 | ⬜ | Garbage-collect superseded versions (feeds VV-TXN-013) |
| VV-TXN-010.7 | ⬜ | Integration test: writer updates a row twice while a reader holds an old snapshot — reader must see the version current at snapshot-open time |

## VV-TXN-011 — Snapshot isolation on read paths  ⬜
**Blocked by:** VV-TXN-010

| Task | Status | Detail |
|---|---|---|
| VV-TXN-011.1 | ⬜ | Define a `Snapshot` type carrying a fixed begin_ts, passed into read calls |
| VV-TXN-011.2 | ⬜ | Thread the snapshot through `TableEngine::get`/`scan` and `QueryEngine::pk_lookup`/`range_scan`/`secondary_scan` — today none of these take a timestamp at all; every read sees "whatever is currently flushed" |
| VV-TXN-011.3 | ⬜ | Implement snapshot pinning so a long scan isn't GC'd out from under it (interacts with VV-TXN-013) |
| VV-TXN-011.4 | ⬜ | Reconcile with LD-READ-02 stale-by-default semantics, which is really a cluster-layer (VV-CLU) concept — document the boundary between local snapshot isolation and cluster read staleness |
| VV-TXN-011.5 | ⬜ | Test: concurrent writer advancing rows past a reader's open snapshot must not change that reader's already-returned or in-flight results |

## VV-TXN-012 — 7-byte inline MVCC telemetry header  ⬜
**Spec:** CORE_ARCHITECTURE §11.2 (latency_ns u32 / tier_source u8 / latch_waits u8 / index_hits u8)
**Blocked by:** VV-TXN-010 (row-version header must exist first)

| Task | Status | Detail |
|---|---|---|
| VV-TXN-012.1 | ⬜ | Define the 7-byte struct and its position inside the VV-TXN-010.1 version header |
| VV-TXN-012.2 | ⬜ | Instrument `latency_ns` on the insert path (needs a cycle timer — VV-HAL-037 exists per S1a but is not wired into `vectorvolt-txn` at all today) |
| VV-TXN-012.3 | ⬜ | Instrument `tier_source` (blocked by VV-STO-019 L0-L4 tiers existing) |
| VV-TXN-012.4 | ⬜ | Instrument `latch_waits` (meaningless until real latches exist beyond today's `parking_lot::Mutex`/`RwLock`) |
| VV-TXN-012.5 | ⬜ | Instrument `index_hits` across `CuckooMap`/`SecondaryIndex`/`CuckooFilter` calls in `insert()` |
| VV-TXN-012.6 | ⬜ | Expose telemetry as a table-scannable virtual column through `vectorvolt-query` (CORE_ARCHITECTURE §11.2: "querying this telemetry is a table scan") |
| VV-TXN-012.7 | ⬜ | `vvctl debug txn <fingerprint>` CLI surface (blocked by VV-TXN-015 fingerprint and VV-NET-001 server existing) |

## VV-TXN-013 — Epoch-based reclamation  ⬜
**Blocked by:** VV-TXN-010, VV-TXN-011

| Task | Status | Detail |
|---|---|---|
| VV-TXN-013.1 | ⬜ | Choose or hand-roll an EBR strategy (crossbeam-epoch style) for retired row versions |
| VV-TXN-013.2 | ⬜ | Define the epoch-advance trigger (all open snapshots have moved past a version) |
| VV-TXN-013.3 | ⬜ | Wire epoch tracking into the VV-TXN-011.1 snapshot handles |
| VV-TXN-013.4 | ⬜ | Reclaim path frees old-version memory back to the arena (VV-TXN-019), not the global allocator (LD-GC) |
| VV-TXN-013.5 | ⬜ | Stress test: many short-lived readers pinning/unpinning epochs concurrently with a writer — no use-after-free, no leak |

## VV-TXN-014 — Explicit BEGIN/COMMIT/ABORT transactions  ⬜
**Spec:** CORE_ARCHITECTURE §6.5 (gRPC TxnBegin/TxnCommit/TxnAbort — cross-layer with VV-NET-004)
**Blocked by:** VV-TXN-010, VV-TXN-015

Verified: today every `insert`/`delete` call is its own atomic WAL append (`manager.rs:110-115,155-156`) — there is no multi-statement grouping of any kind.

| Task | Status | Detail |
|---|---|---|
| VV-TXN-014.1 | ⬜ | Define a `TransactionContext` type that accumulates multiple `Mutation`s before one WAL commit point |
| VV-TXN-014.2 | ⬜ | Implement BEGIN: allocate a `TransactionContext` + fingerprint, no WAL write yet |
| VV-TXN-014.3 | ⬜ | Implement per-statement buffering visible only within the open transaction (requires VV-TXN-010 version chains) |
| VV-TXN-014.4 | ⬜ | Implement COMMIT: single WAL frame/group marking all buffered mutations durable atomically |
| VV-TXN-014.5 | ⬜ | Implement ABORT: discard buffered mutations (optionally a WAL abort marker for observability) |
| VV-TXN-014.6 | ⬜ | Enforce LD-ACID: reject/route cross-shard multi-statement transactions to VV-TXN-020 Saga rather than allow silent partial atomicity |
| VV-TXN-014.7 | ⬜ | Integration test: BEGIN, 3 inserts, ABORT — none of the 3 rows visible to a subsequent GET |

## VV-TXN-015 — 128-bit transaction fingerprint (FNV-1a, ~5ns)  ⬜
**Spec:** CORE_ARCHITECTURE §6.5

| Task | Status | Detail |
|---|---|---|
| VV-TXN-015.1 | ⬜ | Implement FNV-1a 128-bit hash over (shard_id, sequence, timestamp) or equivalent stable input |
| VV-TXN-015.2 | ⬜ | Benchmark to confirm the ~5ns target (blocked by VV-TST-007, no criterion harness exists yet) |
| VV-TXN-015.3 | ⬜ | Return the fingerprint from `insert`/`delete` alongside `WalTip` — today only `WalTip` is returned (`manager.rs:102,150`), a breaking API change to plan for |
| VV-TXN-015.4 | ⬜ | Wire fingerprint into WAL frame metadata for `vvctl debug txn <fingerprint>` lookup |
| VV-TXN-015.5 | ⬜ | Collision test at realistic write volumes |

## VV-TXN-016 — `TransactionContext` async cold store  ⬜
**Blocked by:** VV-TXN-014

| Task | Status | Detail |
|---|---|---|
| VV-TXN-016.1 | ⬜ | Define "cold store" here — spillover for open transactions whose buffered mutations exceed the arena's live-tier budget (LD-ARENA) |
| VV-TXN-016.2 | ⬜ | Implement async write-through to NVMe (VV-STO tier L2) for oversized open transactions |
| VV-TXN-016.3 | ⬜ | Document the LD-GC boundary explicitly — cold spillover is the sanctioned "ok to allocate/block here" escape hatch |
| VV-TXN-016.4 | ⬜ | Test: an open transaction with more buffered mutations than fits in one arena tier still commits correctly |

## VV-TXN-017 — `TransactionPriority` QoS (Critical/High/Normal/Low)  ⬜
**Blocked by:** VV-TXN-008 (ring buffer needs priority-aware slotting)

| Task | Status | Detail |
|---|---|---|
| VV-TXN-017.1 | ⬜ | Define the `TransactionPriority` enum |
| VV-TXN-017.2 | ⬜ | Add a priority field to the mutation envelope — today `Mutation` (`ops.rs:17-20`) has no priority field |
| VV-TXN-017.3 | ⬜ | Implement priority-based admission at the ring-buffer producer side |
| VV-TXN-017.4 | ⬜ | Expose priority as a WAL/histogram dimension (CORE_ARCHITECTURE §11.1 shows `priority="critical"`) |
| VV-TXN-017.5 | ⬜ | Test: under saturated load, Critical-priority inserts observably preempt Normal/Low ones |

## VV-TXN-018 — Strict-priority per-core scheduler  ⬜
**Blocked by:** VV-TXN-017, VV-HAL-024/025 (core pinning, role map)

| Task | Status | Detail |
|---|---|---|
| VV-TXN-018.1 | ⬜ | Define per-core run queues keyed by `TransactionPriority` |
| VV-TXN-018.2 | ⬜ | Implement strict-priority (not weighted-fair) dequeue order |
| VV-TXN-018.3 | ⬜ | Integrate with VV-HAL-024 core pinning so writer threads run on their assigned isolated cores |
| VV-TXN-018.4 | ⬜ | Starvation test: document and verify the chosen policy for Low-priority progress under sustained high-priority load |

## VV-TXN-019 — Arena allocation, zero-GC hot path (Live/Stable)  ⬜
**Spec:** LD-GC, LD-ARENA
**Blocked by:** VV-HAL-029 (`LocalNumaArena` bump allocator)

| Task | Status | Detail |
|---|---|---|
| VV-TXN-019.1 | ⬜ | Design `ArenaAllocator` with Live + Stable regions per LD-GC |
| VV-TXN-019.2 | ⬜ | Replace `WriteBuffer`'s `Vec<(u64, Vec<u64>)>` (`buffer.rs:7`) — one heap alloc per row today — with arena-backed storage |
| VV-TXN-019.3 | ⬜ | Implement bump-reset reclamation triggered by snapshot completion, not a GC pass |
| VV-TXN-019.4 | ⬜ | Implement arena size configuration per LD-ARENA tiers (feeds VV-ENG-019) |
| VV-TXN-019.5 | ⬜ | Implement emergency-snapshot trigger on arena overflow |
| VV-TXN-019.6 | ⬜ | Benchmark zero-allocation confirmation on the insert hot path — today every insert allocates via `values.clone()` (`manager.rs:113`) plus the buffer's own push |
| VV-TXN-019.7 | ⬜ | Loom/stress test for the Live→Stable handoff under concurrent snapshot |

## VV-TXN-020 — Saga / 2PC for cross-shard  ⬜
**Spec:** LD-ACID
**Blocked by:** VV-ENG-006 (multi-table/shard catalog — "cross-shard" has no meaning while one `TxnManager` == one table with no shard concept)

| Task | Status | Detail |
|---|---|---|
| VV-TXN-020.1 | ⬜ | Define the shard abstraction this depends on (see VV-TXN-009.1) before Saga can be scoped |
| VV-TXN-020.2 | ⬜ | Design compensating-transaction record format (the "undo" side of a Saga step) |
| VV-TXN-020.3 | ⬜ | Implement Saga coordinator: sequence of local ACID steps + compensations on failure |
| VV-TXN-020.4 | ⬜ | Implement 2PC as the alternative path for stronger atomicity needs |
| VV-TXN-020.5 | ⬜ | Add a test/lint asserting no cross-shard distributed-lock primitive is ever introduced (LD-ACID rationale) |
| VV-TXN-020.6 | ⬜ | Integration test: 3-step saga, step 2 fails, step 1's compensation runs, net side effect is zero |

## VV-TXN-021 — Deterministic replay (`DeterminismGuard`)  ⬜
**Spec:** LD-REPLAY
**Blocked by:** VV-TXN-010 (MVCC needs deterministic timestamps), VV-TST-009 (deterministic simulation testing)

| Task | Status | Detail |
|---|---|---|
| VV-TXN-021.1 | ⬜ | Audit confirms `vectorvolt-txn` currently makes zero direct `Instant::now()`/`rand::random()` calls — it is accidentally deterministic today only because it has no timestamps or randomness at all yet |
| VV-TXN-021.2 | ⬜ | Define the `DeterminismGuard` trait: `now()`, `random()`, any other handler-visible non-determinism source |
| VV-TXN-021.3 | ⬜ | Route the future VV-TXN-010.2 MVCC timestamp source through `DeterminismGuard` from day one, not retrofitted later |
| VV-TXN-021.4 | ⬜ | Implement a `SimulatedClock` test double for VV-TST-009 |
| VV-TXN-021.5 | ⬜ | Replay test: same WAL segment replayed on two fresh `TxnManager` instances produces byte-identical resulting `.vvsst` files |

---

# LAYER 5 — QUERY EXECUTION (`vectorvolt-query`)

## VV-QRY-001 — PK point lookup (cuckoo → descriptor → partition)  ✅
**Spec:** QUERY_EXECUTION_MODEL.md §2.1

| Task | Status | Detail |
|---|---|---|
| VV-QRY-001.1 | ✅ | `CuckooMap.get(pk)` → `RowDescriptor` — `query/engine.rs:22` |
| VV-QRY-001.2 | ✅ | `TableEngine::get` uses the true O(1) `reader.read_value(c, row_offset)` — `vectorvolt-engine/engine.rs:174-176`, backed by VV-STO-007 |
| VV-QRY-001.3 | 🟡 | **Discrepancy:** `vectorvolt-query`'s own `QueryEngine::fetch_row` instead calls `reader.read_column(c)` then `.get(row_offset)` (`query/engine.rs:143-144`). `read_column` is documented "Deserializes the full column" (`reader.rs:240`) and allocates a full `Vec`/dict every call — the O(n)-per-column path, not the O(1) `read_value` path `vectorvolt-engine::get()` already uses. Two point-lookup implementations exist for the same conceptual operation with very different cost profiles |
| VV-QRY-001.4 | ⬜ | Fix `QueryEngine::fetch_row` to call `read_value()` per column instead of `read_column()->get()` |
| VV-QRY-001.5 | ✅ | Tests: `pk_lookup_found`/`pk_lookup_not_found` (query engine.rs unit tests), `pk_lookup_finds_correct_row`/`pk_lookup_missing_key_returns_none` (`query_integration.rs`), `get_returns_correct_values_after_explicit_flush` (`engine_integration.rs`) |
| VV-QRY-001.6 | ⬜ | `vvctl bench` (`bench.rs:5,46`) only benchmarks the `vectorvolt-engine::TableEngine` path — `vectorvolt-query::QueryEngine`'s (likely much worse, per .3) latency is never measured |

## VV-QRY-002 — Range scan with zone-map pruning  ✅
**Spec:** QUERY_EXECUTION_MODEL.md §2.3

| Task | Status | Detail |
|---|---|---|
| VV-QRY-002.1 | ✅ | Per-partition `reader.zone_min_max(col_idx)` prune before touching row data — `query/engine.rs:40-44`, `vectorvolt-engine/engine.rs:200-204` |
| VV-QRY-002.2 | ✅ | Zone-pruned partitions skipped entirely, no column read — both implementations |
| VV-QRY-002.3 | ✅ | Test `range_scan_zone_prune_skips_partition` — `query/engine.rs` unit tests |
| VV-QRY-002.4 | ✅ | Test `scan_across_two_partitions` — `engine_integration.rs:142-159` |
| VV-QRY-002.5 | 🟡 | Both implementations decode **every** column of a surviving partition eagerly (`query/engine.rs:56-59`, `vectorvolt-engine/engine.rs:207-209`) even when only 1-2 columns are needed by the caller — tracked jointly with VV-QRY-007/009 |
| VV-QRY-002.6 | ⬜ | No binary-search-within-partition step for partially-overlapping partitions — QUERY_EXECUTION_MODEL.md §2.3 specifies "binary search within partition, then stream" for partial overlap; today it's a full linear scan of every row regardless of overlap type |
| VV-QRY-002.7 | ⬜ | Range predicate is single-column only; multi-column composite ranges require callers to intersect result sets manually |

## VV-QRY-003 — Secondary index scan (roaring)  ✅
**Spec:** QUERY_EXECUTION_MODEL.md §2.2

| Task | Status | Detail |
|---|---|---|
| VV-QRY-003.1 | ✅ | `SecondaryIndex` maps value_hash → `RoaringBitmap` of row IDs — `secondary_index.rs:24-27` |
| VV-QRY-003.2 | ✅ | `secondary_scan` resolves each bitmap PK via `pk_lookup` — `query/engine.rs:96-103` |
| VV-QRY-003.3 | ✅ | Tests `secondary_scan_resolves_rows`, `full_query_workflow` — `query_integration.rs` |
| VV-QRY-003.4 | ⬜ | Documented-but-unenforced gap: "Hash collisions may produce false positives — callers should re-check" (`query/engine.rs:89-90`) — no caller anywhere in the codebase performs that re-check today |
| VV-QRY-003.5 | ⬜ | `secondary_scan` calls `pk_lookup` per match (`query/engine.rs:98-100`) — 1,000 matches means 1,000 independent CuckooMap probes, not the "Row Descriptor batch → bulk fetch" pattern QUERY_EXECUTION_MODEL.md §2.2 specifies |
| VV-QRY-003.6 | ⬜ | No test proves the ~500ns/1K-row target or exercises multi-predicate bitmap AND/OR (no such method exists on `SecondaryIndex` at all) |

## VV-QRY-004 — FK existence check (zero I/O)  ✅
**Spec:** QUERY_EXECUTION_MODEL.md §2.4

| Task | Status | Detail |
|---|---|---|
| VV-QRY-004.1 | ✅ | `fk_exists` is a single `CuckooFilter` probe, no partition/mmap access — `query/engine.rs:112-113` |
| VV-QRY-004.2 | ✅ | Tests `fk_exists_hit_and_miss`, `fk_exists_inserted_keys` — both crates |
| VV-QRY-004.3 | ⬜ | No authoritative second-tier fallback — QUERY_EXECUTION_MODEL.md §2.4 specifies "Filter says MAYBE → PK Hash Cache" as a second stage; today a true positive is trusted directly with no confirmation step |
| VV-QRY-004.4 | ⬜ | VV-IDX-010 "Primary-Key Hash Cache" does not exist — `fk_exists` cannot distinguish a false positive from a real hit without a full `pk_lookup`, which defeats "zero I/O" whenever exactness matters |

## VV-QRY-005 — Deterministic algorithm selection (no cost-based optimizer)  ✅
**Spec:** QUERY_EXECUTION_MODEL.md §1 (locked stance — no cost model to be built, ever)

| Task | Status | Detail |
|---|---|---|
| VV-QRY-005.1 | ✅ | `QueryEngine` exposes exactly one method per access pattern with no plan enumeration, no statistics-driven branching — `query/engine.rs` entire file |
| VV-QRY-005.2 | ✅ | Verified: no `Planner`/`CostModel`/`Statistics` type exists anywhere in `vectorvolt-query` or `vectorvolt-engine` |
| VV-QRY-005.3 | ⬜ | §3 "Multi-Pattern Queries" pipeline (intersecting a PK-filtered set with a bitmap set with a zone-pruned set) is not implemented as a composable pipeline — each method returns its own `Vec<QueryRow>`/`Vec<EngineRow>` with no shared row-ID-set representation to intersect against another pattern's output |
| VV-QRY-005.4 | ⬜ | Add a durable note (CONTRIBUTING or module doc comment) restating the §1 warning against ever introducing a cost model, since nothing structurally prevents a future contributor from doing so |
| VV-QRY-005.5 | ⬜ | This feature's "done" status covers only the 4 access patterns actually implemented — graph/hash-join/sort-merge/vector (§2.5/§4/§2.6) are tracked separately below, not yet part of the taxonomy in code |

## VV-QRY-006 — SIMD range scan  ⬜
**Spec:** QUERY_EXECUTION_MODEL.md §2.3; CORE_ARCHITECTURE §7.1
**Blocked by:** VV-HAL-010 (SIMD range filter kernel — delivered in S1a: `filter_range`, 1.26-1.38× measured)

| Task | Status | Detail |
|---|---|---|
| VV-QRY-006.1 | ⬜ | Wire `vectorvolt-hal`'s `filter_range` kernel into `QueryEngine::range_scan`'s comparison loop (`query/engine.rs:61-74`, currently a plain scalar `if val >= lo && val <= hi`) |
| VV-QRY-006.2 | ⬜ | Wire the same kernel into `TableEngine::scan`'s comparison loop (`vectorvolt-engine/engine.rs:212-219`) |
| VV-QRY-006.3 | ⬜ | Decide SIMD-compare strategy for `DictEncoded` columns — compare on the (small) dictionary vs. the expanded indices |
| VV-QRY-006.4 | ⬜ | Preserve the known XOR-sign-bit correctness fix for PKs above `i64::MAX` (S1a finding #4) at the query-layer call site; add a regression test using PKs > `i64::MAX` |
| VV-QRY-006.5 | ⬜ | Criterion benchmark on a realistic ~100K-row partition (blocked by VV-TST-007, no harness exists yet in this crate) |
| VV-QRY-006.6 | ⬜ | SIMD-vs-scalar equivalence test at the query-crate level (mirrors VV-TST-005, exercised end-to-end through `range_scan`/`scan`) |

## VV-QRY-007 — Lazy per-row column fetch in scan  🟡
**Spec:** matrix note — "`get` is lazy; `scan` is not"

| Task | Status | Detail |
|---|---|---|
| VV-QRY-007.1 | ✅ | `get`/`pk_lookup` in `vectorvolt-engine` correctly use `read_value` for O(1) lazy per-row fetch — see VV-QRY-001.2 |
| VV-QRY-007.2 | ⬜ | `TableEngine::scan` decodes **all** columns for every surviving partition up front via `read_column` (`vectorvolt-engine/engine.rs:207-209`) before it knows which rows even match — the opposite of lazy |
| VV-QRY-007.3 | ⬜ | Same issue in `QueryEngine::range_scan` (`query/engine.rs:55-59`) |
| VV-QRY-007.4 | ⬜ | Redesign `scan` to: decode only the predicate column first → collect matching row offsets → `read_value()` only the needed columns for only the matching offsets |
| VV-QRY-007.5 | ⬜ | Add a projection parameter (`Vec<usize>` of wanted column indices) to `scan`/`range_scan` so callers needing 2 of 10 columns don't pay for the other 8 |
| VV-QRY-007.6 | ⬜ | Benchmark eager-decode vs. redesigned lazy scan on a wide-schema (10+ column), low-match-rate partition |

## VV-QRY-008 — SIMD roaring bitmap iterate  ⬜
**Spec:** QUERY_EXECUTION_MODEL.md §2.2
**Blocked by:** VV-HAL-013 (SIMD popcount/bitmap iterate kernel)

| Task | Status | Detail |
|---|---|---|
| VV-QRY-008.1 | ⬜ | Profile whether the `roaring` crate's own iterator already vectorizes internally before assuming custom SIMD is needed |
| VV-QRY-008.2 | ⬜ | If needed, wire VV-HAL-013 into `SecondaryIndex::get`/`get_vec` (`secondary_index.rs:59,81`) iteration |
| VV-QRY-008.3 | ⬜ | Implement bitmap AND/OR/NOT SIMD-accelerated set ops for multi-predicate secondary scans — `SecondaryIndex` has no intersect/union method today (see VV-QRY-005.3) |
| VV-QRY-008.4 | ⬜ | Benchmark against the plain `roaring::RoaringBitmap::iter()` baseline |

## VV-QRY-009 — Late materialization (filter before fetch)  ⬜
**Spec:** S1a HAL finding #3 — `filter_range` capped ~1.35× by scalar `Vec::push` per match; late materialization is the real fix, not wider vectors

| Task | Status | Detail |
|---|---|---|
| VV-QRY-009.1 | ⬜ | Change "matching rows" representation from eagerly-materialized `Vec<QueryRow>`/`Vec<EngineRow>` to a bitmask/row-ID list first |
| VV-QRY-009.2 | ⬜ | Fetch full row values only for the final matching set, after all predicates are evaluated against predicate columns alone |
| VV-QRY-009.3 | ⬜ | Apply to `range_scan`/`scan` — lands together with the VV-QRY-007 redesign |
| VV-QRY-009.4 | ⬜ | Apply to `secondary_scan` — materialize the RoaringBitmap match set fully before any `pk_lookup` call (today interleaved: bitmap iterate then immediate per-row lookup, `query/engine.rs:96-103`) |
| VV-QRY-009.5 | ⬜ | Benchmark late-materialized bitmask path vs. today's eager-`Vec::push` path at a low (1%) selectivity |

## VV-QRY-010 — Vectorized batch execution (columnar batches)  ⬜
**Blocked by:** VV-QRY-006, VV-QRY-008, VV-QRY-009

| Task | Status | Detail |
|---|---|---|
| VV-QRY-010.1 | ⬜ | Define a `ColumnBatch` type (Arrow-like fixed-size chunks) as the common currency between scan/filter/aggregate stages, replacing today's row-at-a-time `Vec<QueryRow>` |
| VV-QRY-010.2 | ⬜ | Rewrite `range_scan`/`scan` to emit `ColumnBatch`es instead of individual rows |
| VV-QRY-010.3 | ⬜ | Rewrite row reconstruction to consume batches, transposing back to row form only for callers that need it |
| VV-QRY-010.4 | ⬜ | Choose a batch size (partition-sized ~100K rows per CORE_ARCHITECTURE §7.3, or a smaller cache-resident chunk e.g. 1024/4096) |
| VV-QRY-010.5 | ⬜ | Benchmark against the row-at-a-time baseline |

## VV-QRY-011 — SIMD hash join (AVX2 scatter/gather)  ⬜
**Spec:** QUERY_EXECUTION_MODEL.md §4
**Blocked by:** VV-QRY-010, VV-ENG-006 (multi-table catalog — joins need two tables interacting, which doesn't exist yet)

| Task | Status | Detail |
|---|---|---|
| VV-QRY-011.1 | ⬜ | Define the join API surface, e.g. `join(left_catalog, right_catalog, left_col, right_col)` |
| VV-QRY-011.2 | ⬜ | Build phase: hash the smaller relation's join column into a hash table (AVX2 scatter) |
| VV-QRY-011.3 | ⬜ | Probe phase: SIMD probe for each row of the larger relation |
| VV-QRY-011.4 | ⬜ | Validate via VV-QRY-004's `fk_exists` before the join, enforcing the FK-driven join-graph assumption (§5) up front, not as an afterthought |
| VV-QRY-011.5 | ⬜ | Correctness test against a naive nested-loop oracle (nested loop itself is never shipped per §4, but is a valid test oracle) |
| VV-QRY-011.6 | ⬜ | Benchmark against the oracle |

## VV-QRY-012 — SIMD sort-merge join  ⬜
**Blocked by:** VV-QRY-010

| Task | Status | Detail |
|---|---|---|
| VV-QRY-012.1 | ⬜ | Implement sorted columnar merge via SIMD comparison for range joins / already-sorted-column joins |
| VV-QRY-012.2 | ⬜ | Decide the "already sorted" precondition check — require a declared sort order in the catalog (ties to VV-ENG-013) vs. sort on the fly |
| VV-QRY-012.3 | ⬜ | Correctness test against the nested-loop oracle |
| VV-QRY-012.4 | ⬜ | Benchmark against hash join (VV-QRY-011), documenting which join-key distribution favors which — still a design-time choice, not a runtime cost decision |

## VV-QRY-013 — Cascade DELETE via bitmap child discovery  ⬜
**Blocked by:** VV-QRY-004, VV-TXN-004 (today's `delete()` has zero cascade concept — `manager.rs:150-158` removes only the target row's own index entry), VV-ENG-009 (DDL — FK relationships need to be declared somewhere)

| Task | Status | Detail |
|---|---|---|
| VV-QRY-013.1 | ⬜ | Define FK relationship declaration (parent table/column → child table/column) — no schema/DDL concept for this exists yet |
| VV-QRY-013.2 | ⬜ | Given a parent PK deletion, use `SecondaryIndex`/`CuckooFilter` bitmap lookups on the child table to discover dependent rows |
| VV-QRY-013.3 | ⬜ | Recursively cascade through multi-level FK chains, with cycle detection |
| VV-QRY-013.4 | ⬜ | Wire into a new `cascade_delete` entry point, WAL-logging every cascaded row deletion individually so replay (VV-TXN-021/LD-REPLAY) reproduces it exactly |
| VV-QRY-013.5 | ⬜ | Integration test: 3-level parent→child→grandchild cascade, verify all rows removed and WAL contains one tombstone per removed row |

## VV-QRY-014 — Aggregate pushdown (SUM/COUNT/MIN/MAX SIMD)  ⬜
**Blocked by:** VV-HAL-012 (SIMD min/max, done S1a), VV-HAL-014 (SIMD sum, done S1a)

| Task | Status | Detail |
|---|---|---|
| VV-QRY-014.1 | ⬜ | Wire the HAL `min_max` kernel into a new `QueryEngine::aggregate_min_max(col_idx)` entry point — today `vectorvolt-query` has no dependency on `vectorvolt-hal` at all |
| VV-QRY-014.2 | ⬜ | Wire the HAL `sum` kernel into `QueryEngine::aggregate_sum(col_idx)` |
| VV-QRY-014.3 | ⬜ | Implement COUNT pushdown — trivial `row_count()` unconditionally, or masked popcount when combined with a filter predicate (ties to VV-QRY-008/009) |
| VV-QRY-014.4 | ⬜ | Push aggregates below the zone-map prune (skip out-of-range partitions before aggregating) |
| VV-QRY-014.5 | ⬜ | Handle `DictEncoded` columns — SIMD sum over dictionary indices needs an index→value gather step first |
| VV-QRY-014.6 | ⬜ | Benchmark against a naive iterate-and-sum baseline on `Raw` and `DictEncoded` columns separately |

## VV-QRY-015 — Graph traversal via CSR  ⬜
**Spec:** QUERY_EXECUTION_MODEL.md §2.5; CORE_ARCHITECTURE §7.2
**Blocked by:** VV-IDX-013 (CSR graph adjacency, Layer 2, itself ⬜), VV-IDX-014 (reverse CSR)

| Task | Status | Detail |
|---|---|---|
| VV-QRY-015.1 | ⬜ | Cross-layer blocker: VV-IDX-013 CSR adjacency structure does not exist in `vectorvolt-index` yet — must land there first |
| VV-QRY-015.2 | ⬜ | Implement single-hop traversal: root PK → CSR offset lookup → neighbor PK array slice |
| VV-QRY-015.3 | ⬜ | Implement multi-hop traversal with a depth parameter, reusing the single-hop step |
| VV-QRY-015.4 | ⬜ | Implement a traversal predicate/filter (stop condition, edge-type filter) |
| VV-QRY-015.5 | ⬜ | Batch-collect leaf Row Descriptors and bulk-fetch via the fixed VV-QRY-001.4 read path |
| VV-QRY-015.6 | ⬜ | Benchmark against the ~400ns/1K-neighbors target |

## VV-QRY-016 — `GRAPH_TRAVERSE()` + `CREATE GRAPH_EDGE`  ⬜
**Blocked by:** VV-QRY-015, VV-QRY-018 (SQL parser — these are SQL-surface entry points)

| Task | Status | Detail |
|---|---|---|
| VV-QRY-016.1 | ⬜ | Define `GRAPH_TRAVERSE(root, path, depth, predicate)` signature mapping onto VV-QRY-015's traversal primitive |
| VV-QRY-016.2 | ⬜ | Define `CREATE GRAPH_EDGE` DDL mapping onto CSR construction/mutation (CSR is not naturally mutable in place — likely a staged/rebuild approach) |
| VV-QRY-016.3 | ⬜ | Parser support for both (entirely blocked by VV-QRY-018) |
| VV-QRY-016.4 | ⬜ | Integration test via the SQL surface once VV-QRY-018/019 exist |

## VV-QRY-017 — Graph analytics (path count/connectivity/centrality)  ⬜
**Blocked by:** VV-QRY-015

| Task | Status | Detail |
|---|---|---|
| VV-QRY-017.1 | ⬜ | Implement bounded-depth-BFS path count between two nodes |
| VV-QRY-017.2 | ⬜ | Implement connectivity/reachability check |
| VV-QRY-017.3 | ⬜ | Implement degree centrality as the MVP measure (betweenness/PageRank are much larger scope — decide the cut explicitly rather than open-ending it) |
| VV-QRY-017.4 | ⬜ | Benchmark on a synthetic graph fixture (blocked by VV-TST-012, or hand-build a small fixture in the interim) |

## VV-QRY-018 — SQL parser (`sqlparser-rs`) → plan  ⬜
**Blocked by:** VV-ENG-007/008/009 (schema registry, type system, DDL — SQL needs typed columns and real DDL, not today's names-only `Vec<String>` schema)

| Task | Status | Detail |
|---|---|---|
| VV-QRY-018.1 | ⬜ | Add `sqlparser-rs`, parse `SELECT`/`INSERT`/`DELETE`/basic `CREATE TABLE` into its AST |
| VV-QRY-018.2 | ⬜ | Map `WHERE col = ?` to `pk_lookup` (PK column) or `secondary_scan` (indexed non-PK column) as a fixed decision table — this mapping logic *is* "the planner" per QUERY_EXECUTION_MODEL.md §1, implemented as a fixed lookup, never a cost model |
| VV-QRY-018.3 | ⬜ | Map `WHERE col BETWEEN x AND y` to `range_scan` |
| VV-QRY-018.4 | ⬜ | Reject unsupported constructs (joins, aggregates, subqueries) with a clear error rather than silently mis-executing |
| VV-QRY-018.5 | ⬜ | Map basic `CREATE TABLE` to VV-ENG-009 DDL calls |
| VV-QRY-018.6 | ⬜ | Test suite: one test per supported SQL shape, one per explicitly-rejected shape |

## VV-QRY-019 — PostgreSQL wire protocol (port 5432)  ⬜
**Spec:** LD-PG-WIRE
**Blocked by:** VV-NET-001 (server binary doesn't exist — this needs a listening process), VV-QRY-018

| Task | Status | Detail |
|---|---|---|
| VV-QRY-019.1 | ⬜ | Implement the PG startup/auth handshake subset (trust mode initially, no real auth) |
| VV-QRY-019.2 | ⬜ | Implement the simple query protocol (`Q` message) round-tripping to VV-QRY-018's parser/executor |
| VV-QRY-019.3 | ⬜ | Implement PG wire result-set encoding (RowDescription/DataRow) from `QueryRow`/`EngineRow` |
| VV-QRY-019.4 | ⬜ | Test with a real `psql`/`libpq` client against the running server |
| VV-QRY-019.5 | ⬜ | Document LD-PG-WIRE's explicit non-goals (no WAL access, no command submission over this port) |

## VV-QRY-020 — Vector search `cosine_similarity()` + `VECTOR(n)` type  ⬜
**Spec:** QUERY_EXECUTION_MODEL.md §2.6; CORE_ARCHITECTURE §7.4
**Blocked by:** VV-ENG-008 (type system — `VECTOR(n)` is a new column type), VV-IDX-012 (HNSW index, Layer 2, ⬜), VV-HAL-016 (SIMD cosine similarity FMA)

| Task | Status | Detail |
|---|---|---|
| VV-QRY-020.1 | ⬜ | Add `VECTOR(n)` as a type-system variant once VV-ENG-008's enum exists — a fixed-width float array per row, not representable in today's single-`u64`-per-value column at all |
| VV-QRY-020.2 | ⬜ | Implement exact `cosine_similarity()` full-scan path first (O(n), correctness oracle) before HNSW |
| VV-QRY-020.3 | ⬜ | Land HNSW index construction (cross-layer blocker: VV-IDX-012) |
| VV-QRY-020.4 | ⬜ | Land HNSW query (`ef_search`, beam search) using VV-HAL-016 SIMD FMA distance computation |
| VV-QRY-020.5 | ⬜ | Integrate as a 5th access-pattern method (`vector_search(query_vec, k)`), HNSW-only per §2.6 ("no choice between exact and approximate") |
| VV-QRY-020.6 | ⬜ | Recall/accuracy test: HNSW top-K vs. the exact-scan oracle from .2 |
| VV-QRY-020.7 | ⬜ | Benchmark against the 5-50µs target |

---

# LAYER 6 — ENGINE / PARTITIONING & CONTROL (`vectorvolt-engine`)

## VV-ENG-001 — `TableEngine` unified read+write per table  ✅
**Spec:** CORE_ARCHITECTURE §15 crate map

| Task | Status | Detail |
|---|---|---|
| VV-ENG-001.1 | ✅ | Single struct owns both `TxnManager` (write) and the `PartitionReader` map (read) — `engine.rs:28-34` |
| VV-ENG-001.2 | ✅ | `open()` wires WAL dir + data dir + recovery in one call — `engine.rs:44-110` |
| VV-ENG-001.3 | ✅ | insert/delete/flush (write) and get/scan/stats (read) share one schema and one partition registry — `engine.rs:112-235` |
| VV-ENG-001.4 | ✅ | Integration tests exercise both sides together — all 15 tests in `engine_integration.rs` |
| VV-ENG-001.5 | ⬜ | One `TableEngine` == exactly one table == exactly one directory; nothing enforces the mapping between instances and physical tables at a process level yet (that's VV-ENG-006's job) |

## VV-ENG-002 — Partition recovery on open  ✅
**Spec:** CORE_ARCHITECTURE §5 (recovery)

| Task | Status | Detail |
|---|---|---|
| VV-ENG-002.1 | ✅ | Scans `data_dir/data/*.vvsst`, opens each via `PartitionReader::open`, tracks max partition_id — `engine.rs:61-71` |
| VV-ENG-002.2 | ✅ | `TxnManager::with_starting_partition` seeded past existing max — `engine.rs:74-80` |
| VV-ENG-002.3 | ✅ | `rebuild_pk_index_from_partitions` restores CuckooMap/CuckooFilter — `engine.rs:85` (see VV-TXN-006) |
| VV-ENG-002.4 | ⬜ | No corruption-handling policy: a single `.vvsst` failing its CRC check (`reader.rs:86-91,110-115`) fails the entire `TableEngine::open()` via `?` (`engine.rs:65`) — no quarantine/skip-and-continue path exists |
| VV-ENG-002.5 | ⬜ | No test exercises recovery with a corrupted partition file present |

## VV-ENG-003 — Auto-flush detection + partition registration  ✅
**Spec:** CORE_ARCHITECTURE §5

| Task | Status | Detail |
|---|---|---|
| VV-ENG-003.1 | ✅ | `insert()` compares `total_rows_flushed` before/after to detect an internal auto-flush, then calls `sync_partitions()` — `engine.rs:118-125` |
| VV-ENG-003.2 | ✅ | `sync_partitions` scans the directory and registers any new `.vvsst` — `engine.rs:248-261` |
| VV-ENG-003.3 | ✅ | Test `auto_flush_registers_partition` — `engine_integration.rs:52-62` |
| VV-ENG-003.4 | ⬜ | Detection relies on a before/after counter comparison (`engine.rs:119,121`) rather than a direct signal from `TxnManager` — fragile against internal refactors; `flush()`/`do_flush()` already get this more directly via the `Option` return (lines 137-140), worth unifying |
| VV-ENG-003.5 | ⬜ | `sync_partitions` re-scans the *entire* directory on every flush (`engine.rs:250`) — O(existing file count) work each time; fine at current scale, will not scale once VV-STO-015 tiered compaction produces many files |

## VV-ENG-004 — Compaction orchestration  🟡 *(matrix says ✅ — corrected during this audit)*
**Spec:** feeds VV-STO-008

**Discrepancy found:** the feature matrix marks this ✅ "done", and its own S0 baseline entry
states *"Open PR not yet merged: `feat/compaction` (partition compaction, VV-STO-008)"* —
contradicting the ✅ in the headline table. Verified directly against the working tree audited
for this report: `vectorvolt-engine/src/engine.rs` has **no `compact()` method**, and grepping
`compact` (case-insensitive) across `vectorvolt-engine` and `vectorvolt-storage` returns zero
matches. The implementation exists only on the unmerged branch `feat/compaction`
(commit `bc91976`, not on `main`, not on the branch this report audits).

| Task | Status | Detail |
|---|---|---|
| VV-ENG-004.1 | 🟡 | `TableEngine::compact()` exists on branch `feat/compaction` (`bc91976`) — merges all on-disk partitions into one sorted `.vvsst`, reclaims tombstoned rows, rebuilds CuckooMap — not on `main` |
| VV-ENG-004.2 | 🟡 | `vvctl table compact <dir>` CLI wrapper exists on the same branch — not on `main` |
| VV-ENG-004.3 | ⬜ | Merge `feat/compaction` into `main` (re-verify it still applies cleanly after the HAL/lazy-read work) and re-run the full suite |
| VV-ENG-004.4 | ⬜ | No compaction *strategy* selection exists (VV-STO-015 leveled vs. size-tiered) — the branch does a naive merge-all-into-one |
| VV-ENG-004.5 | ⬜ | No automatic/scheduled compaction trigger — today `vvctl`-invoked only, manual |
| VV-ENG-004.6 | ⬜ | Correct `VECTORVOLT_FEATURE_MATRIX.md`'s VV-ENG-004 and VV-STO-008 status once this is confirmed with the project owner, or merge the branch to make ✅ actually true |

## VV-ENG-005 — Stats introspection  ✅
**Spec:** CORE_ARCHITECTURE §11 (observability)

| Task | Status | Detail |
|---|---|---|
| VV-ENG-005.1 | ✅ | `EngineStats{rows_buffered, rows_flushed, partition_count, schema}` — `stats.rs:1-13` |
| VV-ENG-005.2 | ✅ | `TableEngine::stats()` is non-flushing — `engine.rs:227-235` |
| VV-ENG-005.3 | ✅ | Test `stats_are_accurate` — `engine_integration.rs:170-189` |
| VV-ENG-005.4 | ⬜ | No per-partition breakdown (row count/size per partition) — only aggregate totals |
| VV-ENG-005.5 | ⬜ | No latency/telemetry stats surfaced here (ties to VV-TXN-012 once it exists) — `vvctl table stats` (`table.rs:214-224`) only prints the 4 fields above |

## VV-ENG-006 — Multi-table catalog in one process  ⬜
**Spec:** matrix note — "One engine per table today; no catalog"

| Task | Status | Detail |
|---|---|---|
| VV-ENG-006.1 | ⬜ | Design a `Catalog` type owning `HashMap<TableName, TableEngine>` — today every `vvctl table` subcommand calls `TableEngine::open()` fresh per invocation (`table.rs:121,133,157,177,205,216,228`); there is no long-lived multi-table process at all |
| VV-ENG-006.2 | ⬜ | Implement table registration/lookup by name |
| VV-ENG-006.3 | ⬜ | Implement table listing (`vvctl table list` equivalent) |
| VV-ENG-006.4 | ⬜ | Verify concurrent access to different tables in one catalog doesn't add cross-table lock contention beyond each `TableEngine`'s own internal locks |
| VV-ENG-006.5 | ⬜ | Decide catalog persistence — today the table list is implicit ("any directory with a `schema.txt`", `table.rs:20,239-252`), no central registry file exists |
| VV-ENG-006.6 | ⬜ | Migrate `vvctl table` commands to a process-wide catalog instead of reopening a fresh `TableEngine` (with a fresh WAL writer) on every single CLI call — today every `get`/`scan`/`stats` invocation reopens and replays the WAL (`engine.rs:44-110`) just to serve one read; this will not work at all once a real long-running server process holds the WAL writer |
| VV-ENG-006.7 | ⬜ | Integration test: two tables in one catalog, concurrent writers on each, no interference |

## VV-ENG-007 — Schema registry + typed columns  ⬜
**Spec:** matrix note — "Everything is `u64`; schema is `schema.txt`"
**Blocked by:** VV-ENG-008 (type system — this feature is the registry that stores VV-ENG-008's types)

| Task | Status | Detail |
|---|---|---|
| VV-ENG-007.1 | ⬜ | Replace `schema.txt` (`table.rs:20,113,123,239-252` — one bare column name per line, no type info) with a structured schema file |
| VV-ENG-007.2 | ⬜ | Design the on-disk format: versioned, column name + type + nullability + per-column constraints — none of this exists; `TableEngine::open` takes a bare `Vec<String>` of names (`engine.rs:45`) |
| VV-ENG-007.3 | ⬜ | Implement schema load/validate inside `vectorvolt-engine` itself, replacing the CLI-only `load_schema` helper (`table.rs:239-252`) that today lives entirely in `vvctl` — the engine crate has no notion of a schema *file* at all |
| VV-ENG-007.4 | ⬜ | Add schema-specific validation errors distinct from today's generic `SchemaMismatch { expected, got }` (`txn/error.rs:18`) — name mismatch and type mismatch, not just column-count mismatch |
| VV-ENG-007.5 | ⬜ | Implement a registry keyed by table name mapping to its schema (ties to VV-ENG-006's catalog) |
| VV-ENG-007.6 | ⬜ | Migration path: transparently upgrade an existing bare `schema.txt` to the new structured format in place |
| VV-ENG-007.7 | ⬜ | Test: open a table created under the old bare `schema.txt` format, confirm transparent migration |

## VV-ENG-008 — Real type system (i64/f64/bool/string/bytes/ts/decimal)  ⬜
**Spec:** matrix note — "everything is `u64`"

Verified in code: `ColumnValues` (`storage/column.rs:22-33`) has exactly two variants,
`Raw(Vec<u64>)` and `DictEncoded{dict: Vec<u64>, indices: Vec<u32>}` — every value in every
column, including the PK, is a raw `u64`. `WriteBuffer` (`txn/buffer.rs:7`) stores
`Vec<(u64, Vec<u64>)>`. `Mutation::Insert` (`txn/ops.rs:18`) is `{pk: u64, values: Vec<u64>}`.
There is no type tag anywhere in the write path, the WAL payload encoding, or the `.vvsst`
column-metadata layout (`format.rs`'s per-column metadata carries only an encoding byte for
Raw/Dict, never a *value*-type byte).

| Task | Status | Detail |
|---|---|---|
| VV-ENG-008.1 | ⬜ | Design the `ColumnType` enum: at minimum `I64, F64, Bool, String, Bytes, Timestamp, Decimal(precision, scale)` |
| VV-ENG-008.2 | ⬜ | Decide value representation for fixed-width types (`I64`/`F64`/`Bool`/`Timestamp`) — likely reuse the existing 8-byte-per-row `Raw` slot via bit-reinterpretation (e.g. `f64::to_bits()`), matching `zone_map.rs`'s own doc comment anticipating "bitwise-reinterpreted floats using IEEE 754 order-preserving encoding" (`zone_map.rs:11-12`), which nothing implements yet |
| VV-ENG-008.3 | ⬜ | Design null representation — there is no NULL concept today; every row must supply exactly `schema.len()` values (`manager.rs:103-108`). Choose between a sentinel value (unsafe, collides with real data) and a separate per-row null-bitmap column (safer, standard) |
| VV-ENG-008.4 | ⬜ | Design variable-length (string/bytes) columnar storage — today's format has no offset/length-array concept at all; both `Raw` and `DictEncoded` assume fixed 8-byte (or 4-byte index) slots. Needs an "offsets + heap" layout: cumulative `Vec<u32>` byte offsets plus a contiguous byte blob, as a new `ColumnValues::Utf8`/`Bytes` variant |
| VV-ENG-008.4.1 | ⬜ | Decide whether variable-length columns also get dictionary encoding (very likely yes — the existing "≤256 distinct values" numeric-surrogate optimization in `ColumnValues::auto_encode`, `column.rs:38`, is exactly the low-cardinality string use case) |
| VV-ENG-008.4.2 | ⬜ | Update the `.vvsst` header/column-metadata format (`format.rs::COL_META_SIZE`) to carry an offset-table location + heap location for variable-length columns — a breaking on-disk format change requiring a version bump (`format.rs::VERSION`, checked at `reader.rs:97-99`) |
| VV-ENG-008.5 | ⬜ | Define per-type comparison/ordering semantics — today `zone_min_max` (`column.rs:76-95`, `zone_map.rs`) compares raw `u64`s directly. Signed `i64` needs the sign-bit XOR trick already used by the SIMD kernels; `f64` needs IEEE-754 total-order bit manipulation; `String`/`Bytes` need byte-lexicographic compare, which a single-scalar zone range doesn't naturally support — likely needs a truncated-prefix zone map for strings (`zone_map.rs:13` already anticipates this: "Strings should be hashed into a u64 prefix for comparison") |
| VV-ENG-008.6 | ⬜ | Define schema evolution semantics: ALTER TABLE ADD/DROP/RENAME COLUMN against already-written `.vvsst` files — choose rewrite-on-read (lazy), eager rewrite, or schema-versioned partitions (a partition remembers the schema version it was written under; the reader reconciles against the table's current schema) |
| VV-ENG-008.7 | ⬜ | Define the migration path for **existing** `.vvsst` files: since every current partition is implicitly "all columns are `I64`", the simplest migration is "type `I64` reads the existing byte layout unchanged" for both `Raw` and `DictEncoded` — verify this holds so purely-numeric legacy tables need no physical rewrite; only tables adding a `String`/variable-length column require one |
| VV-ENG-008.8 | ⬜ | Update `Mutation::encode`/`decode` (`txn/ops.rs:29-77`) to carry a type tag per value instead of assuming `u64` — a WAL wire-format change, another version-bump consideration |
| VV-ENG-008.9 | ⬜ | Update `TxnManager::insert` (`manager.rs:102-141`), `col_value_hash` (`manager.rs:273-279` — currently hashes a raw `u64`; needs per-type hashing for `SecondaryIndex` bucket keys), and `WriteBuffer` (`buffer.rs`) to carry typed values instead of `Vec<u64>` |
| VV-ENG-008.10 | ⬜ | Update `QueryRow`/`EngineRow` (currently `Vec<u64>` — `query/row.rs:1-9`, `engine/row.rs:1-8`) to a typed `Value` enum per column |
| VV-ENG-008.11 | ⬜ | Test matrix: round-trip every type through insert→flush→get→scan, including NULL, including a table mixing fixed- and variable-length columns |
| VV-ENG-008.12 | ⬜ | Backward-compat test: confirm all ~35 existing integration tests across `vectorvolt-txn`/`vectorvolt-query`/`vectorvolt-engine` still pass unmodified against the new typed engine when every column happens to be declared `I64` |

## VV-ENG-009 — DDL: CREATE/ALTER/DROP TABLE  ⬜
**Blocked by:** VV-ENG-007 (schema registry), VV-ENG-008 (type system)

| Task | Status | Detail |
|---|---|---|
| VV-ENG-009.1 | ⬜ | Formalize `CREATE TABLE` beyond today's `vvctl table create <dir> <columns...>` (`table.rs:107-128`) — today "creation" is just "make a directory, write a name-only `schema.txt`, open an engine" with zero typing or constraints |
| VV-ENG-009.2 | ⬜ | Add column-type declarations to `CREATE TABLE` (needs VV-ENG-008 types) |
| VV-ENG-009.3 | ⬜ | Add an explicit primary-key declaration in DDL, replacing today's hardcoded "column 0 is always the PK" convention — assumed throughout `manager.rs` (flush/rebuild logic) and `query/engine.rs:64` (`fetch_row`'s PK recovery) |
| VV-ENG-009.4 | ⬜ | Add partition-key declaration in DDL (feeds VV-ENG-010/011/012/013 directly) |
| VV-ENG-009.5 | ⬜ | Implement `ALTER TABLE ADD COLUMN` — decide default-value backfill semantics (relates to VV-ENG-008.6) |
| VV-ENG-009.6 | ⬜ | Implement `ALTER TABLE DROP COLUMN` — decide whether old partitions retain dead-column bytes (cheap, lazy) or get rewritten (expensive, immediate reclaim) |
| VV-ENG-009.7 | ⬜ | Implement `DROP TABLE` — cleanly stop the writer, delete WAL + data directories, deregister from the catalog (VV-ENG-006) |
| VV-ENG-009.8 | ⬜ | Guard against dropping a table mid-Saga/2PC (VV-TXN-020), once that exists |
| VV-ENG-009.9 | ⬜ | Integration test per DDL verb |

## VV-ENG-010 — Partition control by Entity  ⬜
**Spec:** "User-requested" — the finest partitioning granularity: routing individual rows by a
routing/entity key, independent of the table-wide default strategy
**Blocked by:** VV-ENG-013 (strategy: hash/range/list/composite), VV-ENG-011 (table-level control — entity routing is table-scoped)

**What "entity" means here vs. table vs. schema:** an *entity* is a single logical row-identity
(e.g. one customer, one tenant, one account) whose rows may need deterministic co-location
(e.g. "always route customer 8834's orders to shard 3") independent of the table's default
partition strategy. Today `RowDescriptor` (`row_descriptor.rs`) encodes only physical location
`(file_id, partition_id, row_offset)` — no logical routing key exists anywhere, and there is no
per-entity override mechanism in the codebase at all.

| Task | Status | Detail |
|---|---|---|
| VV-ENG-010.1 | ⬜ | Define the entity routing-key concept: which row column(s) constitute the "entity key" for routing (may differ from the PK — e.g. PK = order_id, entity key = customer_id) |
| VV-ENG-010.2 | ⬜ | Design a sparse per-entity override map (entity-key value → explicit partition assignment) for cases where the default strategy (VV-ENG-013) would place related entities apart |
| VV-ENG-010.3 | ⬜ | Implement entity-key extraction at insert time — `TxnManager::insert` (`manager.rs:102`) has no concept of a routing key distinct from the PK today; needs either a new parameter or a schema-declared `PARTITION BY ENTITY(col)` annotation (from VV-ENG-009 DDL) |
| VV-ENG-010.4 | ⬜ | Implement routing: compute (or look up an override for) the target partition before a row is buffered/flushed — today `flush()` (`manager.rs:167-226`) always writes to the *next sequential* partition_id; there is no data-driven partition placement anywhere |
| VV-ENG-010.5 | ⬜ | Co-location test: insert many rows sharing one entity key across multiple flush cycles, assert they all land in the same partition file |
| VV-ENG-010.6 | ⬜ | Pruning integration: reads for a known entity key should jump directly to that entity's partition(s), extending the existing ZoneMap-prune pattern (`query/engine.rs:40-44`) from numeric ranges to entity-key lookups |
| VV-ENG-010.7 | ⬜ | Catalog metadata: persist the entity-routing table as durable catalog state (ties to VV-ENG-006's catalog, VV-ENG-012's schema-level metadata) |
| VV-ENG-010.8 | ⬜ | Rebalancing hook: when one entity's data outgrows its assigned partition, split/reassign — ties into VV-ENG-015 placement policy and eventual VV-CLU-015 shard rebalancing at the cluster layer (cross-layer dependency, flagged) |

## VV-ENG-011 — Partition control by Table  ⬜
**Spec:** "User-requested" — this granularity already loosely exists today (every table gets
its own directory and sequential partition-id space) but with zero explicit strategy choice

| Task | Status | Detail |
|---|---|---|
| VV-ENG-011.1 | ⬜ | Formalize "one partition-id space per table" — today implicit via `TxnManager.next_partition_id` (`manager.rs:56`), a per-table atomic counter with no declared policy behind it |
| VV-ENG-011.2 | ⬜ | Allow a table-level default strategy declaration (hash/range/list/composite, from VV-ENG-013) distinct from any per-entity override (VV-ENG-010) |
| VV-ENG-011.3 | ⬜ | Allow a table-level target partition count/size — today partitions are created ad hoc, one per flush (`manager.rs:174`'s `fetch_add`), with unbounded uncontrolled growth |
| VV-ENG-011.4 | ⬜ | Implement table-level partition-count/size introspection — `vvctl table stats` (`table.rs:214-224`, `stats.rs`) reports only an aggregate `partition_count`, no per-partition breakdown |
| VV-ENG-011.5 | ⬜ | Wire the declared strategy into `TxnManager::flush` so new partitions are created per-strategy instead of always "next sequential ID" (`manager.rs:174`) |
| VV-ENG-011.6 | ⬜ | Test: two tables with different declared strategies (one range-partitioned by timestamp, one hash-partitioned by PK) behave correctly and independently |

## VV-ENG-012 — Partition control by Schema  ⬜
**Spec:** "User-requested" — coarsest granularity: policy applied across every table in a
schema/namespace
**Blocked by:** VV-ENG-006 (a "schema" is a grouping of tables, needs the catalog first), VV-ENG-011

| Task | Status | Detail |
|---|---|---|
| VV-ENG-012.1 | ⬜ | Define the "schema" concept in the catalog: a named grouping of tables — today no grouping above "one directory = one table" exists (see VV-ENG-006.1) |
| VV-ENG-012.2 | ⬜ | Design schema-level default partition policy inherited by tables that don't declare their own override — three-level precedence: entity override > table declaration > schema default |
| VV-ENG-012.3 | ⬜ | Implement policy inheritance resolution at `CREATE TABLE` time (VV-ENG-009): pull the schema's default when the table specifies none |
| VV-ENG-012.4 | ⬜ | Persist schema-level policy alongside the per-table schema registry (VV-ENG-007) — decide the on-disk location (e.g. a `schema.meta` file at the schema-directory root) |
| VV-ENG-012.5 | ⬜ | Multi-tenancy tie-in: a "schema" is a natural candidate for the tenant-isolation unit once VV-ENG-020/LD-MT-01 lands (tenant = shard group = schema) — document this mapping explicitly so the two features converge |
| VV-ENG-012.6 | ⬜ | Test: schema default set, two tables without table-level overrides inherit it; a third table with an explicit override does not |

## VV-ENG-023 — Partition control by Aggregate Root  ⬜
**Added 2026-07-30.** Verified absent from the entire doc set before adding: all 17 prior "aggregate"
mentions were SQL `SUM`/`COUNT` or "aggregate totals". This is the DDD sense, and it was a real gap.
**Blocked by:** VV-ENG-006 (catalog), VV-ENG-010 (entity routing), VV-ENG-013 (strategy engine)

### Why this is the most valuable of the four granularities

An **aggregate root** is a cluster of entities across *multiple tables* sharing one root identity,
where the aggregate is the transactional boundary. `Order` + `OrderLine` + `Payment` for order 42 are
one aggregate, rooted at `order_id = 42`.

The four granularities, ordered coarse → fine:

| Granularity | Unit | Feature |
|---|---|---|
| Schema | every table in a namespace | VV-ENG-012 |
| Table | one table | VV-ENG-011 |
| **Aggregate root** | **related rows across several tables, sharing a root key** | **VV-ENG-023** |
| Entity | one row identity within one table | VV-ENG-010 |

**Why it matters more than the others:** LD-ACID gives full ACID *at shard level* and requires Saga
or 2PC across shards. A transaction over `Order` + `OrderLine` + `Payment` therefore stays fast and
fully ACID **only if all three land on the same shard.** Aggregate-root partitioning is what
guarantees that. Without it, the most common real-world transaction shape — write a parent and its
children atomically — is a cross-shard distributed transaction by accident.

| Task | Status | Detail |
|---|---|---|
| VV-ENG-023.1 | ⬜ | Declare an aggregate in the catalog: a root table, a root key column, and the member tables with the column in each that references the root |
| VV-ENG-023.2 | ⬜ | Validate at declaration: every member must have a resolvable path to the root key, and a table may belong to at most one aggregate (overlapping aggregates make co-location unsatisfiable) |
| VV-ENG-023.3 | ⬜ | Route **all** member-table rows by the *root* key, not by their own PK — the whole mechanism |
| VV-ENG-023.4 | ⬜ | Extract the root key on insert for a member row whose root key is present in the row |
| VV-ENG-023.5 | ⬜ | Handle the indirect case: a member two hops from the root (`OrderLine` → `Order`), requiring a lookup or a denormalized root key column. Prefer requiring the column — a lookup on the write path costs a read |
| VV-ENG-023.6 | ⬜ | Reject or explicitly warn on an insert whose root key cannot be determined, rather than silently defaulting to some shard and breaking co-location |
| VV-ENG-023.7 | ⬜ | Aggregate-root partitioning overrides table-level strategy for member tables; document the precedence: entity override > aggregate root > table > schema |
| VV-ENG-023.8 | ⬜ | Expose "is this transaction single-shard?" so the txn layer can take the fast path instead of speculatively preparing a Saga (ties to VV-TXN-020) |
| VV-ENG-023.9 | ⬜ | Handle skew: one enormous aggregate can hot-spot a shard. Detect and report; do **not** silently split, which would destroy the co-location guarantee the feature exists to provide |
| VV-ENG-023.10 | ⬜ | Test: insert an Order with 50 OrderLines and 3 Payments; assert every row lands in the same partition |
| VV-ENG-023.11 | ⬜ | Test: a transaction spanning all three tables of one aggregate is reported as single-shard |
| VV-ENG-023.12 | ⬜ | Test: two different aggregates may land on different shards — co-location is *within* an aggregate, not across |
| VV-ENG-023.13 | ⬜ | YAML declaration example in the config reference (VV-CFG-038) |

## VV-ENG-013 — Partition strategy: hash / range / list / composite  ⬜
**Blocked by:** nothing structurally — this is the strategy engine VV-ENG-010/011/012 all call into

| Task | Status | Detail |
|---|---|---|
| VV-ENG-013.1 | ⬜ | Implement `PartitionStrategy::Hash(column, num_buckets)` — deterministic `hash(value) % num_buckets` routing |
| VV-ENG-013.2 | ⬜ | Implement `PartitionStrategy::Range(column, boundaries)` — ordered boundary list, binary search for target partition (natural fit with the existing `ZoneMap`/range-scan machinery in `zone_map.rs`) |
| VV-ENG-013.3 | ⬜ | Implement `PartitionStrategy::List(column, value_to_partition_map)` — explicit enumerated value assignment |
| VV-ENG-013.4 | ⬜ | Implement `PartitionStrategy::Composite(Vec<PartitionStrategy>)` — e.g. hash by tenant then range by date within tenant |
| VV-ENG-013.5 | ⬜ | Define a common `route(&self, row: &[Value]) -> PartitionId` trait so VV-ENG-010/011/012 remain strategy-agnostic |
| VV-ENG-013.6 | ⬜ | Unit test per strategy variant plus one composite test combining two levels |

## VV-ENG-014 — Partition pruning at plan time  ⬜
**Blocked by:** VV-ENG-013, VV-QRY-018 (SQL layer — "plan time" implies a plan exists)

| Task | Status | Detail |
|---|---|---|
| VV-ENG-014.1 | ⬜ | Extend the existing runtime ZoneMap-prune pattern (`query/engine.rs:40-44`, `vectorvolt-engine/engine.rs:200-204` — currently the *only* pruning that exists, happening at scan-execution time) to consult VV-ENG-013's strategy metadata before a `PartitionReader` is even opened |
| VV-ENG-014.2 | ⬜ | For hash-partitioned tables: given an equality predicate on the hash column, compute the exact target partition and skip all others outright — stronger than zone-map bounding |
| VV-ENG-014.3 | ⬜ | For range/list-partitioned tables: resolve predicate bounds against partition boundaries directly from catalog metadata, without opening any file |
| VV-ENG-014.4 | ⬜ | Wire pruning results into VV-QRY-018's plan-mapping step once the SQL layer exists |
| VV-ENG-014.5 | ⬜ | Benchmark: partition-count-scaling test proving pruning avoids opening `PartitionReader`s — today `TableEngine::scan` (`engine.rs:196-223`) iterates every *registered* partition and only zone-checks after all are already open in the map, so today's "pruning" saves scan work but never the file-open/mmap cost |

## VV-ENG-015 — Per-partition placement policy (core/NUMA node)  ⬜
**Blocked by:** VV-HAL-024/025/028 (core pinning, role map, NUMA topology — none exist yet)

| Task | Status | Detail |
|---|---|---|
| VV-ENG-015.1 | ⬜ | Define a `PlacementPolicy` recording which core(s)/NUMA node a partition's writer/readers should run on |
| VV-ENG-015.2 | ⬜ | Wire placement into partition creation so new partitions get NUMA-local memory (blocked entirely on VV-HAL-028/029) |
| VV-ENG-015.3 | ⬜ | Implement placement metadata storage in the catalog (ties to VV-ENG-006/007) |
| VV-ENG-015.4 | ⬜ | Document this as blocked on Layer 0 HAL primitives rather than attempting a stub implementation |
| VV-ENG-015.5 | ⬜ | Test (once unblocked): two partitions placed on different NUMA nodes, reads served by correctly-pinned threads |

## VV-ENG-016 — Shard-per-core mapping  ⬜
**Blocked by:** VV-ENG-011 (table-level partitioning as the shard unit), VV-TXN-009 (one-writer-per-shard model), VV-HAL-024 (core pinning)

| Task | Status | Detail |
|---|---|---|
| VV-ENG-016.1 | ⬜ | Define the mapping function shard_id → core_id, informed by `CpuTopology` once VV-HAL-001 exists |
| VV-ENG-016.2 | ⬜ | Wire one `TxnManager` writer thread per shard onto its assigned core |
| VV-ENG-016.3 | ⬜ | Ensure reader threads can run on any core (CORE_ARCHITECTURE §4.2: readers are not core-pinned to a shard) while writers are strictly pinned |
| VV-ENG-016.4 | ⬜ | Test: assert (via platform-conditional affinity inspection) the shard-N writer thread runs on its assigned core |

## VV-ENG-017 — Config system (TOML file + env + CLI precedence)  ⬜
**Spec:** matrix note — "Zero config-file loading exists today"

Verified: every tunable observed across the audited crates is a hardcoded literal or positional
CLI argument, never a config file. `WalConfig::default().with_fsync(false)` is hardcoded in
`TableEngine::open` (`engine.rs:55,91`); `FLUSH_THRESHOLD: usize = 1_048_576` is a `const` in
`vvctl/src/commands/table.rs:21` **and independently redefined** in `bench.rs:7` (two copies of
conceptually one tunable); `CuckooMap::with_capacity(64)`/`CuckooFilter::with_capacity(64)` are
hardcoded in `TxnManager::with_starting_partition` (`manager.rs:83,86`). No `toml`/`config`/
`figment` crate dependency exists anywhere in the workspace.

| Task | Status | Detail |
|---|---|---|
| VV-ENG-017.1 | ⬜ | Choose the file format — TOML per the feature name (`toml` crate + `serde`) |
| VV-ENG-017.2 | ⬜ | Define `VectorVoltConfig` covering, at minimum, every hardcoded value found above: WAL fsync mode, flush threshold, initial index capacities, data/WAL directory layout |
| VV-ENG-017.3 | ⬜ | Implement file loading (`vectorvolt.toml` at a well-known location, `--config` override) |
| VV-ENG-017.4 | ⬜ | Implement environment-variable layer (e.g. `VV_FLUSH_THRESHOLD`) with a documented naming convention |
| VV-ENG-017.5 | ⬜ | Implement the CLI-flag layer and thread it through `vvctl table` subcommands, unifying today's two independently-hardcoded `FLUSH_THRESHOLD` constants (`table.rs:21`, `bench.rs:7`) into one config-sourced value |
| VV-ENG-017.6 | ⬜ | Implement and test precedence explicitly: file < env < CLI — a test with all three layers set to different values must show CLI winning |
| VV-ENG-017.7 | ⬜ | Validation with good error messages: reject an invalid value (e.g. `flush_threshold = "not a number"` or `= 0`) naming the offending key/file, not a generic parse panic |
| VV-ENG-017.8 | ⬜ | Encode LD-ARENA's edge (64MB)/standard-cloud (4-16GB)/large-cloud (64GB) tiers as named presets (`--profile edge\|cloud\|large`) instead of requiring manual tuning of every value |
| VV-ENG-017.9 | ⬜ | Make the effective config printable/loggable at startup with per-key source annotation ("file"/"env"/"CLI"/"default") |
| VV-ENG-017.10 | ⬜ | Tests: round-trip (write TOML, load, compare), precedence (.6), validation-error (.7), preset (.8) |

## VV-ENG-018 — Config hot-reload  ⬜
**Blocked by:** VV-ENG-017

| Task | Status | Detail |
|---|---|---|
| VV-ENG-018.1 | ⬜ | Classify every VV-ENG-017.2 config key as hot-reloadable vs. restart-only. Proposed split based on what's actually touched at runtime today: **restart-only** — WAL/data directory paths, initial `CuckooMap`/`CuckooFilter` capacity (one-time `with_capacity` allocation, `manager.rs:83,86`), WAL fsync mode (`WalConfig` consumed once by `WalWriter::open`, `engine.rs:56`); **hot-reloadable** — flush threshold (today baked into `WriteBuffer::capacity` at construction, `buffer.rs:8,13` — would need to become a runtime-checked value to actually be hot-reloadable), backpressure/batching thresholds (once VV-WAL-011/012 exist), log level |
| VV-ENG-018.2 | ⬜ | Implement a file-watcher or signal-triggered (SIGHUP/Windows equivalent) reload of the TOML file |
| VV-ENG-018.3 | ⬜ | Implement safe application of hot-reloadable keys without pausing the write path (a `Mutex`-guarded swap is acceptable only if the read stays off the absolute hot path) |
| VV-ENG-018.4 | ⬜ | Reject (log + ignore, don't crash) a reload attempt that touches a restart-only key, with a clear operator-facing message |
| VV-ENG-018.5 | ⬜ | Test: change a hot-reloadable key, trigger reload, confirm new inserts observe it; change a restart-only key, trigger reload, confirm it's ignored with a logged warning |

## VV-ENG-019 — Arena sizing config (64MB edge → 64GB cloud)  ⬜
**Spec:** LD-ARENA
**Blocked by:** VV-TXN-019 (arena allocator must exist first), VV-ENG-017 (config system delivers the value)

| Task | Status | Detail |
|---|---|---|
| VV-ENG-019.1 | ⬜ | Expose arena size as a config key per VV-ENG-017.8's preset system |
| VV-ENG-019.2 | ⬜ | Wire the configured size into `ArenaAllocator::new` once VV-TXN-019 lands |
| VV-ENG-019.3 | ⬜ | Implement the "arena overflow triggers emergency snapshot" behavior (LD-ARENA) as a config-driven threshold |
| VV-ENG-019.4 | ⬜ | Validate arena size against available system memory at startup (clear error, not an OOM crash, for an impossible request) |
| VV-ENG-019.5 | ⬜ | Test across the three documented presets (64MB / 4-16GB / 64GB) |

## VV-ENG-020 — Multi-tenancy shard groups  ⬜
**Spec:** LD-MT-01..08
**Blocked by:** VV-ENG-006 (catalog), VV-ENG-012 (schema-level partition control — the natural tenant boundary, see VV-ENG-012.5)

| Task | Status | Detail |
|---|---|---|
| VV-ENG-020.1 | ⬜ | Define `TenantId` and the shard-group ownership model (LD-MT-01: dedicated shard groups per tenant) |
| VV-ENG-020.2 | ⬜ | Implement engine-level cross-tenant query rejection (LD-MT-02: "prohibited at the engine level, not just policy") — today there is no tenant concept anywhere, so nothing prevents any query from reading any table; needs a `TenantId` check before any partition is touched, not an application-layer-only guard |
| VV-ENG-020.3 | ⬜ | Implement separate memory arenas per tenant (depends on VV-TXN-019 existing) |
| VV-ENG-020.4 | ⬜ | Implement separate WAL segments per tenant — today one WAL dir per table (`engine.rs:50`), extend to one dir per (tenant, table) |
| VV-ENG-020.5 | ⬜ | Implement separate `.vvsst` file namespaces per tenant (mostly falls out of .4's directory structure) |
| VV-ENG-020.6 | ⬜ | Data residency: allow a tenant's shard group to be pinned to a region/placement policy (ties to VV-ENG-015 and the cluster layer VV-CLU for actual multi-region — cross-layer, flagged) |
| VV-ENG-020.7 | ⬜ | Shared vs. dedicated infrastructure mode toggle (LD-MT-06/07) |
| VV-ENG-020.8 | ⬜ | Test: two tenants, attempt a cross-tenant read, assert engine-level rejection (not a bypassable application check) |

## VV-ENG-021 — Per-tenant metering  ⬜
**Spec:** LD-MT-04
**Blocked by:** VV-ENG-020

| Task | Status | Detail |
|---|---|---|
| VV-ENG-021.1 | ⬜ | Command-count metering: per-tenant counter on every insert/delete, extending the existing `total_rows_flushed: AtomicU64` pattern (`manager.rs:57,260-267`) to a tenant-scoped equivalent |
| VV-ENG-021.2 | ⬜ | Storage-GB metering: sum `.vvsst` file sizes per tenant — `EngineStats` (`stats.rs`) today reports `partition_count` but never byte size |
| VV-ENG-021.3 | ⬜ | Projection-lag metering per tenant (blocked by VV-PRJ-004 existing at all, Layer 15, currently ⬜) |
| VV-ENG-021.4 | ⬜ | Expose metering via `vvctl` and/or the future Prometheus endpoint (VV-OBS-005) |
| VV-ENG-021.5 | ⬜ | Test: counters are tenant-scoped (tenant A's activity does not inflate tenant B's counters) |

## VV-ENG-022 — Tenant onboarding <100ms  ⬜
**Spec:** LD-MT-03
**Blocked by:** VV-ENG-020, VV-CTL-001 (control-plane API surface — onboarding is a control-plane operation; Layer 9 crate doesn't exist)

| Task | Status | Detail |
|---|---|---|
| VV-ENG-022.1 | ⬜ | Define the onboarding operation: allocate `TenantId`, create shard group/directories, register in the catalog — essentially VV-ENG-020's setup as a single fast API call |
| VV-ENG-022.2 | ⬜ | Benchmark today's closest equivalent (`TableEngine::open` creating fresh WAL+data dirs, `engine.rs:49-53`) as a directional baseline — full tenant onboarding also includes catalog registration (VV-ENG-006) and arena allocation (VV-TXN-019), neither of which exist yet to include in a real number |
| VV-ENG-022.3 | ⬜ | Implement the <100ms budget as a perf-gated test once the full onboarding path exists (ties to VV-TST-008 CI performance gate) |
| VV-ENG-022.4 | ⬜ | Failure/rollback path: a partial onboarding failure (e.g. directory created but catalog registration fails) must not leave orphaned state — needs a cleanup/retry design |

---

*End of Layer 4-6 task-level traceability. Cross-references into Layer 0 (HAL), Layer 2 (Index),
Layer 7 (Network), Layer 9 (Control Plane), and Layer 15 (Projections) are called out inline as
"Blocked by VV-XXX" wherever this audit found a real dependency in the code or docs.*
