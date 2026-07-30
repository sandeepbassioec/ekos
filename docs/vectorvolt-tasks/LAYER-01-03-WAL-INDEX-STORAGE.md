# Layer 1 — WAL  ·  Layer 2 — Index  ·  Layer 3 — Storage
Task-level traceability. Status: ✅ done · 🟡 partial · ⬜ pending · 🧊 env-gated · 🔬 research

**Scope:** VV-WAL-001..018, VV-IDX-001..014, VV-STO-001..024 (per `docs/VECTORVOLT_FEATURE_MATRIX.md`).
**Verification method:** every ✅/🟡 claim below cites a file:line actually read in this pass. Branch
inspected: working tree at time of writing (`git status` showed branch `feat/hal-simd` checked out,
env metadata says `feat/lazy-column-read` — both were inspected; see note under VV-STO-008 and
VV-QRY-007 cross-reference). `main` and `feat/compaction` were also read via `git show` where noted.

---

## Rollup

| Layer | Features | Tasks total | ✅ | 🟡 | ⬜ | 🧊 |
|---|---|---|---|---|---|---|
| L1 — WAL (`vectorvolt-wal`) | 18 | 99 | 50 | 1 | 34 | 14 |
| L2 — Index (`vectorvolt-index`) | 14 | 80 | 32 | 1 | 47 | 0 |
| L3 — Storage (`vectorvolt-storage`) | 24 | 149 | 32 | 6 | 111 | 0 |
| **Total** | **56** | **328** | **114** | **8** | **192** | **14** |

**Headline finding:** L1 and L2 are functionally complete for their Phase-2 scope (18/18 and 14/14
features have real, tested code; the ⬜ tasks inside "done" features are hardening/edge-case gaps,
not missing functionality). L3 has its core write/read/format path done (8 of 24 features touch
real code) but **every encoding beyond raw+dictionary, all compression, all tiering/PITR/retention,
and partition compaction itself are either absent from the branch reviewed or unmerged** — see
VV-STO-008 for a load-bearing branch discrepancy.

---

# LAYER 1 — WAL (`vectorvolt-wal`)

## VV-WAL-001 — Segmented append-only log ✅
**Spec:** VECTORVOLT_CORE_ARCHITECTURE.md §5; segment header documented in `segment.rs:1-16`
**Evidence:** `vectorvolt-wal/src/segment.rs`, `vectorvolt-wal/src/writer.rs`

| Task | Status | Detail |
|---|---|---|
| VV-WAL-001.1 | ✅ | `SegmentHeader` (64-byte header: magic/version/segment_id/base_seq/created_ns) — `segment.rs:44-101` |
| VV-WAL-001.2 | ✅ | `SegmentWriter::create` — allocates file, writes header — `segment.rs:115-141` |
| VV-WAL-001.3 | ✅ | `SegmentWriter::append_bytes` — `segment.rs:144-149` |
| VV-WAL-001.4 | ✅ | `SegmentWriter::would_exceed` size-limit check — `segment.rs:182-184` |
| VV-WAL-001.5 | ✅ | Auto-rotation: `WriterInner::rotate` invoked when `would_exceed` trips — `writer.rs:42-58,78-80` |
| VV-WAL-001.6 | ✅ | `SegmentReader::open`/`next_frame`/`all_frames` — `segment.rs:196-243` |
| VV-WAL-001.7 | ✅ | Segment discovery: `list_segment_ids` sorts ascending from directory scan — `segment.rs:249-259` |
| VV-WAL-001.8 | ✅ | Test: `segment_rotation` forces multiple files via tiny `segment_max_bytes` — `writer.rs:214-232`; test: `multi_segment_verify` confirms chain integrity across rotations — `wal_integration.rs:81-101` |

## VV-WAL-002 — Frame format (97-byte overhead, exact) ✅
**Spec:** frame layout diagram — `frame.rs:1-20`
**Evidence:** `vectorvolt-wal/src/frame.rs`

| Task | Status | Detail |
|---|---|---|
| VV-WAL-002.1 | ✅ | Constants: `FRAME_HEADER_SIZE=61`, `FRAME_TAIL_SIZE=36`, `FRAME_OVERHEAD=97` — `frame.rs:27-30` |
| VV-WAL-002.2 | ✅ | `Frame::encode()` — magic/len/seq/ts/prev_hash/type/payload_len/payload/hash/crc — `frame.rs:86-113` |
| VV-WAL-002.3 | ✅ | `Frame::decode()` — full field parse + `frame_len`/`payload_len` consistency checks — `frame.rs:116-198` |
| VV-WAL-002.4 | ✅ | `PayloadType` enum (Data/Checkpoint/EndOfSegment) + `TryFrom<u8>` — `frame.rs:36-56` |
| VV-WAL-002.5 | ✅ | Test: `roundtrip` encode→decode equality — `frame.rs:206-222` |

## VV-WAL-003 — CRC32 per frame ✅
**Spec:** "CRC32C checksum (hardware-accelerated...)" — CORE_ARCHITECTURE.md §5 — **not yet accelerated, see gap below**
**Evidence:** `vectorvolt-wal/src/frame.rs`

| Task | Status | Detail |
|---|---|---|
| VV-WAL-003.1 | ✅ | CRC32 (software `crc32fast`, CRC-32/ISO-HDLC — **not** CRC-32C) computed at encode — `frame.rs:109-110` |
| VV-WAL-003.2 | ✅ | CRC verified at decode, `WalError::Crc32Mismatch` on mismatch — `frame.rs:170-173` |
| VV-WAL-003.3 | ✅ | Test: `detects_bit_flip` — flips a payload byte, asserts decode error — `frame.rs:224-239` |
| VV-WAL-003.4 | ⬜ | Wire hardware CRC-32C (delivered in `vectorvolt-hal` on branch `feat/hal-simd` — 44.6× faster, see Sprint S1a log) into the WAL frame tail. **Blocked by design, not just effort**: S1a's own finding #5 states "CRC-32C ≠ the WAL's CRC-32/ISO-HDLC... adopting it in the WAL is a breaking format change" — needs a WAL format version bump + dual-read back-compat path before this can land |

## VV-WAL-004 — SHA3-256 hash chain (tamper detection) ✅
**Spec:** CORE_ARCHITECTURE.md §5; frame tail — `frame.rs:15`
**Evidence:** `vectorvolt-wal/src/frame.rs`, `vectorvolt-wal/src/reader.rs`

| Task | Status | Detail |
|---|---|---|
| VV-WAL-004.1 | ✅ | `Frame::compute_hash()` — SHA3-256 over magic+len+seq+ts+prev_hash+type+payload — `frame.rs:71-83` |
| VV-WAL-004.2 | ✅ | Chain propagation: `WriterInner.chain_hash` stamped onto every new frame as `prev_hash` — `writer.rs:30,68-71,83` |
| VV-WAL-004.3 | ✅ | `frame_hash` verified on decode — `frame.rs:175-188` |
| VV-WAL-004.4 | ✅ | `WalReader::verify()` walks the full chain from `GENESIS_HASH` — `reader.rs:115-154` |
| VV-WAL-004.5 | ✅ | Test: `verify_chain` — 5 frames verified — `reader.rs:242-254` |
| VV-WAL-004.6 | ⬜ | No test corrupts `prev_hash`/`frame_hash` mid-chain and asserts `WalError::ChainBroken` is returned with the correct `sequence`. Add an adversarial chain-break test (today only the happy path and single-frame bit-flip in `frame.rs` are covered) |

## VV-WAL-005 — 64 MiB segment auto-rotation ✅
**Spec:** CORE_ARCHITECTURE.md §5 diagram "64 MiB, auto-rotate at capacity"
**Evidence:** `vectorvolt-wal/src/config.rs`, `vectorvolt-wal/src/writer.rs`

| Task | Status | Detail |
|---|---|---|
| VV-WAL-005.1 | ✅ | Default `segment_max_bytes = 64 * 1024 * 1024` — `config.rs:33` |
| VV-WAL-005.2 | ✅ | `WalConfig::with_segment_max_bytes` override — `config.rs:54-57` |
| VV-WAL-005.3 | ✅ | Rotation trigger tied to `would_exceed` in `append_frame` — `writer.rs:78-80` |
| VV-WAL-005.4 | ✅ | Test: `segment_rotation` with 512-byte limit — `writer.rs:213-232` |

## VV-WAL-006 — Crash recovery (chain_hash + next_seq) ✅
**Spec:** CORE_ARCHITECTURE.md §5 "Recovery: On restart..."
**Evidence:** `vectorvolt-wal/src/writer.rs`

| Task | Status | Detail |
|---|---|---|
| VV-WAL-006.1 | ✅ | `WalWriter::open` detects existing segments via `list_segment_ids` — `writer.rs:110-113` |
| VV-WAL-006.2 | ✅ | Replays last segment to recover `next_seq`/`chain_hash` from the last frame (or segment header if empty) — `writer.rs:121-135` |
| VV-WAL-006.3 | ✅ | Reopens the last segment in append mode at the correct byte offset — `writer.rs:137-147` |
| VV-WAL-006.4 | ✅ | Test: `append_and_recover` — reopen after drop, tip sequence correct — `writer.rs:194-211` |
| VV-WAL-006.5 | ✅ | Test: `crash_recovery` (simulated crash mid-write, no explicit close) — `wal_integration.rs:39-79` |

## VV-WAL-007 — Replay / `read_all` / `read_range` / `verify` 🟡
**Spec:** CORE_ARCHITECTURE.md §5; API surface in `lib.rs:44-56`
**Evidence:** `vectorvolt-wal/src/reader.rs`
**Gap:** `verify()`'s error-reporting path is a known placeholder, not a missing feature — see .4 below.

| Task | Status | Detail |
|---|---|---|
| VV-WAL-007.1 | ✅ | `WalCursor` + segment-boundary-aware `scan()` — `reader.rs:16-28,47-89` |
| VV-WAL-007.2 | ✅ | `read_all()` — `reader.rs:92-99` |
| VV-WAL-007.3 | ✅ | `read_range(from, to)` inclusive — `reader.rs:101-112` |
| VV-WAL-007.4 | 🟡 | `verify()` correctly detects a broken chain (returns `Err` when count of successfully-scanned frames ≠ total frame count) but the `ChainBroken` error it constructs has a **self-admitted placeholder value**: `actual: hex(&GENESIS_HASH), // placeholder; real value in the frame` — `reader.rs:132-150`. The failure is detected; the diagnostic message lies about which hash actually broke the chain. Fix: thread the real mismatched hash through the re-walk loop instead of hardcoding `GENESIS_HASH` |
| VV-WAL-007.5 | ✅ | `stats()` — segment/frame/byte counts, first/last sequence, chain tip — `reader.rs:157-188` |
| VV-WAL-007.6 | ✅ | Test: `read_all_frames` — `reader.rs:226-240` |

## VV-WAL-008 — `GroupCommitBatcher` (N writes → 1 fsync) ✅
**Spec:** CORE_ARCHITECTURE.md §5 "Group Commit Batcher"
**Evidence:** `vectorvolt-wal/src/group_commit.rs`

| Task | Status | Detail |
|---|---|---|
| VV-WAL-008.1 | ✅ | `Submission`/`CommitFuture` (oneshot-channel-backed async handle) — `group_commit.rs:31-44` |
| VV-WAL-008.2 | ✅ | Bounded `crossbeam_channel` + background Tokio task with tick+shutdown `select!` — `group_commit.rs:93-124` |
| VV-WAL-008.3 | ✅ | `drain_and_flush`: batch collection bounded by `max_batch_bytes` — `group_commit.rs:126-151` |
| VV-WAL-008.4 | ✅ | One `WalWriter::flush()` (one fsync) for the whole batch — `group_commit.rs:160-168` |
| VV-WAL-008.5 | ✅ | Graceful shutdown drains the queue before stopping — `group_commit.rs:108-114,204-207` |
| VV-WAL-008.6 | ✅ | Test: `concurrent_commits` — 100 concurrent submits, all unique sequences 1..100 — `group_commit.rs:221-247` |

## VV-WAL-009 — Backpressure on bounded channel ✅
**Spec:** `group_commit.rs:13-16` doc comment
**Evidence:** `vectorvolt-wal/src/group_commit.rs`

| Task | Status | Detail |
|---|---|---|
| VV-WAL-009.1 | ✅ | `submit()` uses `try_send`; full channel → `Err(WalError::Backpressure)` (never blocks) — `group_commit.rs:191-200` |
| VV-WAL-009.2 | ✅ | `metrics.record_drop()` on backpressure — `group_commit.rs:197`, `metrics.rs:81-83` |
| VV-WAL-009.3 | ✅ | Test: `backpressure_when_channel_full` (2-slot channel, 3rd submit rejected) — `group_commit.rs:273-296` |

## VV-WAL-010 — `BatcherMetrics` (EMA batch size/latency) ✅
**Spec:** `metrics.rs:1-7` doc comment
**Evidence:** `vectorvolt-wal/src/metrics.rs`

| Task | Status | Detail |
|---|---|---|
| VV-WAL-010.1 | ✅ | Atomic counters: fsync/frames/bytes/batch count — `metrics.rs:16-33` |
| VV-WAL-010.2 | ✅ | EMA batch size, α=0.1 fixed-point — `metrics.rs:68-72` |
| VV-WAL-010.3 | ✅ | EMA flush latency (µs) — `metrics.rs:74-78` |
| VV-WAL-010.4 | ✅ | Peak batch size via CAS loop — `metrics.rs:54-66` |
| VV-WAL-010.5 | ✅ | `MetricsSnapshot` + `Display` impl — `metrics.rs:121-163` |
| VV-WAL-010.6 | ✅ | Test: `metrics_recorded` — `group_commit.rs:249-271` |

## VV-WAL-011 — Adaptive batching 1→10,000 cmds/fsync ⬜
**Spec:** Feature matrix note "LD-BATCH: currently fixed window"
**Evidence:** none — `BatcherConfig` is a fixed `max_batch_bytes`/`flush_interval` pair, no feedback loop (`group_commit.rs:47-65`)

| Task | Status | Detail |
|---|---|---|
| VV-WAL-011.1 | ⬜ | Define adaptive-controller state: target latency, batch-size bounds `[1, 10_000]` |
| VV-WAL-011.2 | ⬜ | Feedback signal: measure per-tick queue depth / flush latency, derive an up/down adjustment to an explicit `max_batch_frames` (today only `max_batch_bytes` exists) |
| VV-WAL-011.3 | ⬜ | Wire controller into `drain_and_flush` replacing the fixed byte threshold |
| VV-WAL-011.4 | ⬜ | Expose `current_target_batch_size` on `BatcherMetrics` |
| VV-WAL-011.5 | ⬜ | Test: under synthetic low load, batch size converges toward 1; under high load, toward 10,000 |
| VV-WAL-011.6 | ⬜ | Regression test: adaptive controller does not regress p99 vs. the current fixed-window baseline |

## VV-WAL-012 — Token-bucket backpressure per shard + 429/Retry-After ⬜
**Spec:** Feature matrix "LD-BACK"
**Evidence:** none — current backpressure is a single global bounded channel per `GroupCommitBatcher` (`group_commit.rs:68-72`), no per-shard token bucket, no retry-after signal

| Task | Status | Detail |
|---|---|---|
| VV-WAL-012.1 | ⬜ | `TokenBucket` struct: capacity, refill_rate, tokens, last_refill |
| VV-WAL-012.2 | ⬜ | Per-shard bucket registry keyed by `shard_id` |
| VV-WAL-012.3 | ⬜ | Integrate into `GroupCommitBatcher::submit()` — consume a token before `try_send` |
| VV-WAL-012.4 | ⬜ | Compute `retry_after_ms` from bucket refill rate on rejection |
| VV-WAL-012.5 | ⬜ | New `WalError::RateLimited { retry_after_ms }` variant |
| VV-WAL-012.6 | ⬜ | Unit tests: bucket depletion, refill-over-time, per-shard isolation |

## VV-WAL-013 — WAL log compaction (post-snapshot + projection safe point) ⬜
**Spec:** STORAGE_TIERING.md Part 4 "Log Compaction"
**Evidence:** none — no compaction-eligibility or segment-deletion code exists in `vectorvolt-wal`

| Task | Status | Detail |
|---|---|---|
| VV-WAL-013.1 | ⬜ | Compaction-eligibility check: all projection workers past `segment_last_lsn` AND snapshot covers the LSN range (depends on unbuilt VV-PRJ-004 offsets and a snapshot mechanism) |
| VV-WAL-013.2 | ⬜ | Per-segment "safe to delete" marker derived from `list_segment_ids` + each segment's last LSN |
| VV-WAL-013.3 | ⬜ | Archive-before-delete step (depends on unbuilt VV-STO-019..022 L3/L4 tiers) |
| VV-WAL-013.4 | ⬜ | `WalWriter`/`WalReader` API: `compact(before_lsn)` removing fully-archived segments |
| VV-WAL-013.5 | ⬜ | Test: compaction preserves reachable data, refuses to delete un-archived segments |
| VV-WAL-013.6 | ⬜ | Test: PITR still resolves post-compaction (blocked on VV-STO-023) |

## VV-WAL-014 — WAL-as-event-log for projections (LSN offsets) ⬜
**Spec:** STORAGE_TIERING.md Part 4
**Evidence:** none — `Frame.sequence` is de facto an LSN but there is no `Lsn` type, no `read_events()` API, no offset store

| Task | Status | Detail |
|---|---|---|
| VV-WAL-014.1 | ⬜ | `Lsn` newtype distinct from raw `sequence: u64` |
| VV-WAL-014.2 | ⬜ | `WalReader::read_events(shard_id, offset, batch_size)` returning typed domain events, not raw `Frame` |
| VV-WAL-014.3 | ⬜ | Per-worker offset persistence (`projection_offsets` store) |
| VV-WAL-014.4 | ⬜ | `ProjectionWorker::tick()` skeleton per STORAGE_TIERING.md Part 4 |
| VV-WAL-014.5 | ⬜ | Consumer-lag computation (`tip.sequence - worker.offset`) |
| VV-WAL-014.6 | ⬜ | Test: two workers at different offsets both replay correctly from the same WAL |
| VV-WAL-014.7 | ⬜ | Test: offset persists across restart |

## VV-WAL-015 — WAL Shipper (stream segments to hot standby) ⬜
**Spec:** CORE_ARCHITECTURE.md §6.4 (RDMA path), TCP fallback implied
**Evidence:** none

| Task | Status | Detail |
|---|---|---|
| VV-WAL-015.1 | ⬜ | `WalShipper` trait: `ship(segment_bytes) -> Result<()>` |
| VV-WAL-015.2 | ⬜ | TCP baseline implementation (RDMA is 🧊, see VV-WAL-016) |
| VV-WAL-015.3 | ⬜ | Ack-based flow control (standby acks last received LSN) |
| VV-WAL-015.4 | ⬜ | Reconnect/backoff on standby disconnect |
| VV-WAL-015.5 | ⬜ | Ship on segment rotation, not per-frame (integrate with `SegmentWriter::rotate`) |
| VV-WAL-015.6 | ⬜ | Test: standby WAL matches primary after N writes (loopback TCP) |
| VV-WAL-015.7 | ⬜ | Test: shipper resumes correctly after standby restart mid-stream |

## VV-WAL-016 — RDMA WAL transport 🧊
**Environment needed:** Linux + Mellanox/ConnectX or InfiniBand RDMA-capable NIC. Cannot be built or tested in this (Windows) environment.

| Task | Status | Detail |
|---|---|---|
| VV-WAL-016.1 | 🧊 | `WalTransport` trait abstraction shared with VV-NET-013 so RDMA/TCP are swappable |
| VV-WAL-016.2 | 🧊 | `rdma-core` binding integration spike |
| VV-WAL-016.3 | 🧊 | RDMA WRITE path: register WAL segment memory region, post to replica |
| VV-WAL-016.4 | 🧊 | Fallback-to-TCP when no RDMA NIC is detected at startup |
| VV-WAL-016.5 | 🧊 | Hardware validation suite — Linux + RDMA NIC only |

## VV-WAL-017 — io_uring WAL writes 🧊
**Environment needed:** Linux kernel with io_uring support (5.1+, ideally 5.11+ for reliable feature set).

| Task | Status | Detail |
|---|---|---|
| VV-WAL-017.1 | 🧊 | `io_uring` integration spike (`tokio-uring` or raw `liburing` bindings) |
| VV-WAL-017.2 | 🧊 | Async submission queue for `append_bytes`, replacing `BufWriter::write_all` (`segment.rs:144-149`) |
| VV-WAL-017.3 | 🧊 | Completion-queue polling integrated with `GroupCommitBatcher` tick |
| VV-WAL-017.4 | 🧊 | Non-Linux fallback to current `std::fs` path via trait boundary |
| VV-WAL-017.5 | 🧊 | Benchmark: io_uring vs. `BufWriter` fsync latency (Linux only) |

## VV-WAL-018 — O_DIRECT bypass page cache 🧊
**Environment needed:** Linux (O_DIRECT semantics are Linux-specific; Windows has no equivalent flag).

| Task | Status | Detail |
|---|---|---|
| VV-WAL-018.1 | 🧊 | O_DIRECT open flag + aligned buffer allocator (512B/4KB alignment) |
| VV-WAL-018.2 | 🧊 | Adapt `SegmentWriter` to aligned writes (currently arbitrary-length `append_bytes`, `segment.rs:144-149`) |
| VV-WAL-018.3 | 🧊 | Fallback for the final partial-page write (pad + truncate on close) |
| VV-WAL-018.4 | 🧊 | Benchmark vs. buffered I/O (Linux only) |

---

# LAYER 2 — Index (`vectorvolt-index`)

## VV-IDX-001 — Cuckoo hash PK index (2-probe, O(1) worst case) ✅
**Spec:** READ_PATH_ARCHITECTURE.md §3
**Evidence:** `vectorvolt-index/src/cuckoo_map.rs`

| Task | Status | Detail |
|---|---|---|
| VV-IDX-001.1 | ✅ | Two independent multiplicative hash fns `h1`/`h2` — `cuckoo_map.rs:52-65` |
| VV-IDX-001.2 | ✅ | `Bucket` struct + `EMPTY = u64::MAX` sentinel — `cuckoo_map.rs:36,69-88` |
| VV-IDX-001.3 | ✅ | `get()`: exactly 2 probes, O(1) worst case — `cuckoo_map.rs:136-149` |
| VV-IDX-001.4 | ✅ | `insert_inner()`: BFS-style eviction chain up to `MAX_DISPLACE=512` — `cuckoo_map.rs:202-238` |
| VV-IDX-001.5 | ✅ | `remove()` — `cuckoo_map.rs:167-184` |
| VV-IDX-001.6 | ✅ | Wired live into the write path: `TxnManager::insert`/`delete` call `pk_index.write().insert/remove` — `vectorvolt-txn/src/manager.rs:117-121,150-157` |
| VV-IDX-001.7 | ✅ | Test: `insert_and_get`, `update_existing_key`, `remove` — `cuckoo_map.rs:286-312` |
| VV-IDX-001.8 | ✅ | Test: `large_insert_triggers_grow` (500 keys, verifies grow correctness) — `cuckoo_map.rs:314-324`; `h1_h2_differ` collision sanity — `cuckoo_map.rs:340-355` |
| VV-IDX-001.9 | ⬜ | No test forces the `MAX_DISPLACE=512` exhaustion path (`Err(IndexError::MapFull)`). Add an adversarial test (e.g. many keys colliding on both `h1`/`h2` mod a tiny capacity) to prove the full-map error path actually fires |

## VV-IDX-002 — `RowDescriptor` bit-packed u64 ✅
**Spec:** READ_PATH_ARCHITECTURE.md §3, RowDescriptor comment
**Evidence:** `vectorvolt-index/src/row_descriptor.rs`

| Task | Status | Detail |
|---|---|---|
| VV-IDX-002.1 | ✅ | Bit-pack encode: `file_id:16 << 48 \| partition_id:16 << 32 \| row_offset:32` — `row_descriptor.rs:19-22` |
| VV-IDX-002.2 | ✅ | Accessors `file_id()`/`partition_id()`/`row_offset()` — `row_descriptor.rs:26-40` |
| VV-IDX-002.3 | ✅ | `as_u64()`/`from_u64()` round trip (used for storage/serialization) — `row_descriptor.rs:44-52` |
| VV-IDX-002.4 | ✅ | Test: `round_trip`, `max_values`, `zero_values`, `fields_do_not_bleed_into_each_other` — `row_descriptor.rs:71-112` |

## VV-IDX-003 — Roaring bitmap secondary index 🟡
**Spec:** READ_PATH_ARCHITECTURE.md §4 — claims "Supports AND / OR / NOT in microseconds using SIMD"
**Evidence:** `vectorvolt-index/src/secondary_index.rs`
**Gap:** storage + single-value lookup is real; the boolean combinator API the spec promises does not exist.

| Task | Status | Detail |
|---|---|---|
| VV-IDX-003.1 | ✅ | `HashMap<u32, RoaringBitmap>` value-hash → row-id set — `secondary_index.rs:24-27` |
| VV-IDX-003.2 | ✅ | `insert`/`remove` with empty-bucket cleanup — `secondary_index.rs:41-54` |
| VV-IDX-003.3 | ✅ | `get`/`get_vec`/`contains_hash` accessors — `secondary_index.rs:56-86` |
| VV-IDX-003.4 | ✅ | `cardinality()`/`distinct_values()` stats — `secondary_index.rs:70-77` |
| VV-IDX-003.5 | ✅ | Test: `insert_and_get`, `remove_entry`, `remove_last_cleans_up_bucket`, `cardinality`, `large_insert` (10K rows) — `secondary_index.rs:101-163` |
| VV-IDX-003.6 | ⬜ | **Doc-vs-code gap:** no `and()`/`or()`/`not()` bitmap-combinator API exists on `SecondaryIndex`. `RoaringBitmap` itself supports these ops natively (it's the underlying `roaring` crate type), but nothing in this crate exposes multi-predicate intersection/union to callers. Add `SecondaryIndex::intersect(&[u32]) -> RoaringBitmap` etc., plus tests |

## VV-IDX-004 — Zone map (min/max per partition) ✅
**Spec:** READ_PATH_ARCHITECTURE.md §5
**Evidence:** `vectorvolt-index/src/zone_map.rs`
**Note:** this is a *second*, independent zone-map implementation from the one embedded per-column in `.vvsst` files (`vectorvolt-storage`, see VV-STO-006). Both are real and tested; they serve different call sites (this one is a standalone in-memory PK-zone index, the storage one is per-column file metadata). Not a bug, but worth knowing there are two.

| Task | Status | Detail |
|---|---|---|
| VV-IDX-004.1 | ✅ | `ZoneEntry{min,max}` + `overlaps()` — `zone_map.rs:18-29` |
| VV-IDX-004.2 | ✅ | `update()` expanding min/max range — `zone_map.rs:46-57` |
| VV-IDX-004.3 | ✅ | `set()` explicit bulk-load — `zone_map.rs:60-62` |
| VV-IDX-004.4 | ✅ | `overlapping_partitions()`/`pruned_partitions()` iterators — `zone_map.rs:73-87` |
| VV-IDX-004.5 | ✅ | Test: `update_expands_range`, `overlaps_basic`, `overlapping_partitions`, `pruned_partitions`, `single_value_partition` — `zone_map.rs:112-172` |
| VV-IDX-004.6 | ✅ | Wired live: `TxnManager` holds `zone_map: RwLock<ZoneMap>` updated on flush — `vectorvolt-txn/src/manager.rs:19,50` |

## VV-IDX-005 — Cuckoo filter (FK existence, infallible insert) ✅
**Spec:** READ_PATH_ARCHITECTURE.md §7 Tier 1
**Evidence:** `vectorvolt-index/src/cuckoo_filter.rs`

| Task | Status | Detail |
|---|---|---|
| VV-IDX-005.1 | ✅ | `Bucket` with `BUCKET_SIZE=4` fingerprint slots — `cuckoo_filter.rs:73-114` |
| VV-IDX-005.2 | ✅ | `fingerprint()`/`index1()`/`index2()` XOR-symmetric hash scheme — `cuckoo_filter.rs:49-69` |
| VV-IDX-005.3 | ✅ | `insert()` with `MAX_KICKS=500` displacement — `cuckoo_filter.rs:150-183` |
| VV-IDX-005.4 | ✅ | `contains()`/`remove()` — `cuckoo_filter.rs:187-216` |
| VV-IDX-005.5 | ✅ | Test: `insert_and_contains`, `remove_makes_absent`, `large_insert_no_error`(1000), `fingerprint_never_zero`, `false_positive_rate_reasonable` — `cuckoo_filter.rs:268-328` |

## VV-IDX-006 — Auto-grow on load factor ✅
**Spec:** Feature matrix note "CuckooMap"; also covers the S0-delivered `CuckooFilter` overflow
**Evidence:** `vectorvolt-index/src/cuckoo_map.rs`, `vectorvolt-index/src/cuckoo_filter.rs`

| Task | Status | Detail |
|---|---|---|
| VV-IDX-006.1 | ✅ | `CuckooMap` load-factor check before insert (`LOAD_NUM=9/LOAD_DEN=16` ≈ 56%) — `cuckoo_map.rs:158-160` |
| VV-IDX-006.2 | ✅ | `grow()`: doubles capacity, full rehash — `cuckoo_map.rs:240-254` |
| VV-IDX-006.3 | ✅ | `CuckooFilter` exact overflow `HashSet` absorbs keys beyond the displacement limit, making `insert()` infallible (delivered Sprint S0 per commit history) — `cuckoo_filter.rs:122-183,243-247` |
| VV-IDX-006.4 | ✅ | Test: `large_insert_triggers_grow` (CuckooMap, 500 keys) — `cuckoo_map.rs:314-324`; `large_insert_no_error` (CuckooFilter, 1000 keys) — `cuckoo_filter.rs:294-303` |

## VV-IDX-007 — Huge-page-backed index allocation ⬜
**Spec:** CORE_ARCHITECTURE.md §3.4; feature matrix "Needs VV-HAL-031"
**Evidence:** none — `CuckooMap.buckets` is a plain `Vec<Bucket>` (`cuckoo_map.rs:97`); VV-HAL-031 (huge pages) is itself unbuilt on this branch

| Task | Status | Detail |
|---|---|---|
| VV-IDX-007.1 | ⬜ | `HugePageVec<T>` allocator wrapping `MAP_HUGETLB`/Windows large-page VirtualAlloc (depends on VV-HAL-031) |
| VV-IDX-007.2 | ⬜ | Swap `CuckooMap`'s backing store for `HugePageVec` via a generic allocator parameter or feature flag |
| VV-IDX-007.3 | ⬜ | Fallback to normal `Vec` when huge pages are unavailable/unpermitted |
| VV-IDX-007.4 | ⬜ | Benchmark: TLB miss rate before/after (meaningful signal is Linux-only via `perf`) |
| VV-IDX-007.5 | ⬜ | Test: existing `CuckooMap` suite passes unchanged against the huge-page-backed variant |

## VV-IDX-008 — Cache-line-packed bucket layout ⬜
**Spec:** Feature matrix "Currently 16B buckets, want 64B"
**Evidence:** current `Bucket{key: u64, value: RowDescriptor}` is 16 bytes (`cuckoo_map.rs:69-73`), i.e. 4 entries per 64B cache line if packed contiguously — but they are *not* explicitly grouped that way today

| Task | Status | Detail |
|---|---|---|
| VV-IDX-008.1 | ⬜ | Redesign to a `CacheLine` type holding 4 `(key, value)` pairs = one 64B line |
| VV-IDX-008.2 | ⬜ | Update `h1`/`h2` to address cache-line index + in-line slot rather than per-entry bucket |
| VV-IDX-008.3 | ⬜ | SIMD-friendly `[u64; 4]` key array per line (feeds VV-IDX-009) |
| VV-IDX-008.4 | ⬜ | Rewrite `insert`/`get`/`remove`/`grow` for the new layout |
| VV-IDX-008.5 | ⬜ | Test: existing `CuckooMap` test suite passes unchanged against the new layout |
| VV-IDX-008.6 | ⬜ | Benchmark: cache-line layout vs. current 16B-bucket layout, lookup latency |

## VV-IDX-009 — SIMD bucket probe (compare 8 fingerprints at once) ⬜
**Spec:** CORE_ARCHITECTURE.md §7.1 SIMD strategy; depends on VV-IDX-008
**Evidence:** none — `CuckooMap::get` is scalar (`cuckoo_map.rs:136-149`)

| Task | Status | Detail |
|---|---|---|
| VV-IDX-009.1 | ⬜ | AVX2 kernel: load 8-wide key/fingerprint vector, `_mm256_cmpeq_epi64`/`epi8` compare |
| VV-IDX-009.2 | ⬜ | Scalar fallback with identical semantics (correctness oracle, per the S1a HAL lesson "hand-written SIMD must beat `-O3`, not naive scalar") |
| VV-IDX-009.3 | ⬜ | Wire into `CuckooMap::get()` via `vectorvolt-hal`'s SIMD tier dispatch (that dispatcher exists on branch `feat/hal-simd` today, not merged into `vectorvolt-index`) |
| VV-IDX-009.4 | ⬜ | SIMD-vs-scalar equivalence test (mirrors VV-TST-005 pattern) |
| VV-IDX-009.5 | ⬜ | Benchmark vs. the current 2-probe scalar `get()` |

## VV-IDX-010 — Primary-Key Hash Cache (9 bytes/row, authoritative) ⬜
**Spec:** READ_PATH_ARCHITECTURE.md §7 Tier 2
**Evidence:** none — no `(u64, u8)` authoritative existence cache exists; only `CuckooFilter` (probabilistic) and `CuckooMap` (location, not existence-only) exist

| Task | Status | Detail |
|---|---|---|
| VV-IDX-010.1 | ⬜ | `PrimaryKeyCache` struct: packed `(u64 key, u8 is_active)` array per spec |
| VV-IDX-010.2 | ⬜ | Insert/update wired from `TxnManager` insert/delete (Layer 4 integration point) |
| VV-IDX-010.3 | ⬜ | Lookup API returning `Active`/`Inactive`/`Unknown` tri-state |
| VV-IDX-010.4 | ⬜ | Capacity-planning helper (10B rows → ~90 GB, per spec) |
| VV-IDX-010.5 | ⬜ | Test: authoritative correctness — zero false positives/negatives (contrast with `CuckooFilter`'s ~0.78% FP) |
| VV-IDX-010.6 | ⬜ | Test: Tier-1→Tier-2 flow — filter says "maybe" → cache resolves definitively |

## VV-IDX-011 — L3-residency budget enforcement for filter ⬜
**Spec:** READ_PATH_ARCHITECTURE.md §7 "fits in L3 on a 256-core NUMA node with large L3"
**Evidence:** none — no budget calculator or runtime check exists; `CpuTopology`/L3-size detection (VV-HAL-001) is itself unbuilt

| Task | Status | Detail |
|---|---|---|
| VV-IDX-011.1 | ⬜ | L3-size input from `CpuTopology` (blocked on VV-HAL-001) |
| VV-IDX-011.2 | ⬜ | Budget calculator: `filter_bytes = num_buckets * BUCKET_SIZE`; warn/reject when configured capacity exceeds the L3 budget |
| VV-IDX-011.3 | ⬜ | Runtime metric: `filter_bytes` vs. L3 budget surfaced via stats |
| VV-IDX-011.4 | ⬜ | Test: budget-calculator math against known L3 sizes (mocked `CpuTopology`, since real detection is unbuilt) |

## VV-IDX-012 — HNSW vector index ⬜
**Spec:** CORE_ARCHITECTURE.md §7.4
**Evidence:** none

| Task | Status | Detail |
|---|---|---|
| VV-IDX-012.1 | ⬜ | `VectorId` + `HnswGraph` skeleton (layers, entry point, neighbor lists) |
| VV-IDX-012.2 | ⬜ | Insert: greedy search + neighbor-selection heuristic (`M`, `efConstruction`) |
| VV-IDX-012.3 | ⬜ | Search: `efSearch` beam search returning top-k |
| VV-IDX-012.4 | ⬜ | Distance-function trait (cosine/L2/dot) — hard dependency on the unbuilt `VECTOR(n)` column type (Layer 5/6, out of this scope) |
| VV-IDX-012.5 | ⬜ | Persistence format for the HNSW graph (new file format or extension of `.vvsst`) |
| VV-IDX-012.6 | ⬜ | Delete/tombstone handling (HNSW doesn't delete cleanly — needs lazy-tombstone + periodic rebuild) |
| VV-IDX-012.7 | ⬜ | Test: recall@k vs. brute-force on synthetic vectors |
| VV-IDX-012.8 | ⬜ | Benchmark: insert/search throughput at 1M/10M/100M vectors |

## VV-IDX-013 — CSR graph adjacency ⬜
**Spec:** READ_PATH_ARCHITECTURE.md §6
**Evidence:** none

| Task | Status | Detail |
|---|---|---|
| VV-IDX-013.1 | ⬜ | `CsrGraph` struct: `node_index: Vec<u64>` (N+1) + `edges: Vec<u64>` |
| VV-IDX-013.2 | ⬜ | Builder: accumulate `(src, dst)` pairs, sort by `src`, compute offsets |
| VV-IDX-013.3 | ⬜ | Neighbor iterator: `edges[node_index[n]..node_index[n+1]]` |
| VV-IDX-013.4 | ⬜ | Incremental append policy (new edges without full rebuild), or documented rebuild-only limitation |
| VV-IDX-013.5 | ⬜ | Test: traversal correctness against a hand-built small graph |
| VV-IDX-013.6 | ⬜ | Benchmark: sequential-scan traversal latency vs. pointer-linked equivalent (spec target ~400ns/1K edges) |

## VV-IDX-014 — Reverse CSR (incoming edges / CASCADE) ⬜
**Spec:** READ_PATH_ARCHITECTURE.md §6, §9
**Evidence:** none; depends on VV-IDX-013

| Task | Status | Detail |
|---|---|---|
| VV-IDX-014.1 | ⬜ | `ReverseCsrGraph` mirrors `CsrGraph` for incoming edges |
| VV-IDX-014.2 | ⬜ | Builder: derive reverse edges from forward CSR (sort by `dst`) or dual-build |
| VV-IDX-014.3 | ⬜ | Incoming-neighbor iterator (same shape as `CsrGraph`) |
| VV-IDX-014.4 | ⬜ | Wire into CASCADE DELETE child-discovery (spec §9), cross-referencing `SecondaryIndex`/`RoaringBitmap` (VV-IDX-003) for bulk tombstoning |
| VV-IDX-014.5 | ⬜ | Test: reverse edges match forward edges by construction |
| VV-IDX-014.6 | ⬜ | Benchmark: cascade-delete discovery for 10M synthetic children (spec target ~5µs) |

---

# LAYER 3 — Storage Engine (`vectorvolt-storage`)

## VV-STO-001 — Micro-partition `.vvsst` binary format ✅
**Spec:** `format.rs:1-31` layout diagram
**Evidence:** `vectorvolt-storage/src/format.rs`, `writer.rs`, `reader.rs`

| Task | Status | Detail |
|---|---|---|
| VV-STO-001.1 | ✅ | Format constants: `MAGIC="VVSST001"`, `VERSION=1`, `HEADER_FIXED_SIZE=32`, `COL_META_SIZE=56` — `format.rs:33-46` |
| VV-STO-001.2 | ✅ | `write_partition()`: header → schema → column-meta → data → footer sequence — `writer.rs:27-154` |
| VV-STO-001.3 | ✅ | `PartitionReader::parse()`: full round-trip decode incl. magic/version checks — `reader.rs:71-167` |
| VV-STO-001.4 | ✅ | Test: `write_produces_valid_magic`, `write_empty_partition_errors` — `writer.rs:172-186` |
| VV-STO-001.5 | ✅ | Test: `multi_column_partition`, `file_id_and_partition_id_preserved` — `storage_integration.rs:60-82,198-206` |

## VV-STO-002 — Struct-of-Arrays columnar layout ✅
**Spec:** READ_PATH_ARCHITECTURE.md §10
**Evidence:** `vectorvolt-storage/src/partition.rs`, `writer.rs`

| Task | Status | Detail |
|---|---|---|
| VV-STO-002.1 | ✅ | `MicroPartition.columns: Vec<ColumnValues>` (SoA layout) — `partition.rs:11-20` |
| VV-STO-002.2 | ✅ | Row-count consistency enforced across columns (`RowCountMismatch`) — `partition.rs:34-51` |
| VV-STO-002.3 | ✅ | On-disk column-contiguous data section (each column serialized sequentially) — `writer.rs:130-146` |
| VV-STO-002.4 | ✅ | Test: `mismatched_row_count_returns_error` — `partition.rs:80-87` |

## VV-STO-003 — Dictionary encoding ✅
**Spec:** `column.rs:1-18` doc comment
**Evidence:** `vectorvolt-storage/src/column.rs`, `writer.rs`, `reader.rs`

| Task | Status | Detail |
|---|---|---|
| VV-STO-003.1 | ✅ | `ColumnValues::auto_encode()` — dict if ≤256 distinct, raw fallback otherwise — `column.rs:38-63` |
| VV-STO-003.2 | ✅ | `get()`/`to_raw_vec()` decode for both encodings — `column.rs:98-113` |
| VV-STO-003.3 | ✅ | `zone_min_max()` over dict only for `DictEncoded` (O(distinct), not O(n)) — `column.rs:86-94` |
| VV-STO-003.4 | ✅ | On-disk dict + index-array layout (dict values then u32 indices) — `writer.rs:137-144`, `reader.rs:260-278` |
| VV-STO-003.5 | ✅ | Test: `auto_encode_chooses_dict_for_low_cardinality`/`_raw_for_high_cardinality`, `dict_boundary_at_256` — `column.rs:131-186` |
| VV-STO-003.6 | ✅ | Test: `dict_encoded_column_round_trip` (500 rows, 7 distinct) — `storage_integration.rs:41-57` |

## VV-STO-004 — Header + file CRC verification ✅
**Spec:** `format.rs` doc comment; `lib.rs:15` "Two CRC32 checksums protect the header and the entire file"
**Evidence:** `vectorvolt-storage/src/writer.rs`, `reader.rs`

| Task | Status | Detail |
|---|---|---|
| VV-STO-004.1 | ✅ | Header CRC32 over first 28 bytes at write — `writer.rs:110-111` |
| VV-STO-004.2 | ✅ | File CRC32 over the whole buffer minus footer at write — `writer.rs:149-150` |
| VV-STO-004.3 | ✅ | File CRC checked first on open (cheapest corruption gate) — `reader.rs:82-91` |
| VV-STO-004.4 | ✅ | Header CRC checked after magic/version parse — `reader.rs:108-115` |
| VV-STO-004.5 | ✅ | Test: `header_crc_is_correct`, `file_crc_is_correct` — `writer.rs:188-207` |
| VV-STO-004.6 | ✅ | Test: `corrupted_file_crc_detected`, `corrupted_header_crc_detected` — `storage_integration.rs:126-184` |

## VV-STO-005 — mmap reader with `populate()` ✅
**Spec:** `reader.rs:1-17` doc comment
**Evidence:** `vectorvolt-storage/src/reader.rs`

| Task | Status | Detail |
|---|---|---|
| VV-STO-005.1 | ✅ | File open + `unsafe` mmap with `.populate()` — `reader.rs:63-69` (documented as "Only `unsafe` block in workspace") |
| VV-STO-005.2 | ✅ | Safety invariant documented — SSTables are immutable once written — `reader.rs:9-17,65-66` (comment-only; no runtime enforcement, see gap) |
| VV-STO-005.3 | ✅ | Windows `populate()` no-op behavior acknowledged in doc comment — `reader.rs:11-12` |
| VV-STO-005.4 | ⬜ | No test verifies `populate()` actually pre-faults pages (would need OS page-fault counters — meaningful only on Linux). Add a Linux-gated benchmark/test; also consider whether the "immutable once written" safety invariant should be enforced (e.g. read-only file permissions) rather than only documented |

## VV-STO-006 — Zone map population at write time ✅
**Spec:** READ_PATH_ARCHITECTURE.md §5, §10
**Evidence:** `vectorvolt-storage/src/writer.rs`, `reader.rs`

| Task | Status | Detail |
|---|---|---|
| VV-STO-006.1 | ✅ | Per-column `zone_min_max()` computed before write — `writer.rs:61` |
| VV-STO-006.2 | ✅ | Zone stats serialized into column-metadata block — `writer.rs:125-126` |
| VV-STO-006.3 | ✅ | `zone_min_max()` reader accessor — `reader.rs:193-199` |
| VV-STO-006.4 | ✅ | Test: `zone_min_max_correct`, `zone_enables_partition_pruning` — `storage_integration.rs:84-123` |

## VV-STO-007 — O(1) `read_value` point read ✅
**Spec:** feature matrix "400× GET speedup, 200ns p50"
**Evidence:** `vectorvolt-storage/src/reader.rs`, `vectorvolt-engine/src/engine.rs`

| Task | Status | Detail |
|---|---|---|
| VV-STO-007.1 | ✅ | Raw-encoding O(1) offset arithmetic (`data_offset + row_offset*8`) — `reader.rs:213-219` |
| VV-STO-007.2 | ✅ | Dict-encoding O(1) two-hop lookup (index array, then dict) — `reader.rs:220-233` |
| VV-STO-007.3 | ✅ | Wired into `TableEngine::get()`, replacing a full `read_column()` decode per lookup — `vectorvolt-engine/src/engine.rs:150-179` (loops `read_value` per column, not per row) |
| VV-STO-007.4 | ✅ | Bounds-checked slicing (`try_into` → `InvalidData` error, not a panic) — `reader.rs:215-217,224-226,229-231` |
| VV-STO-007.5 | ⬜ | No test in `storage_integration.rs` exercises `read_value()` directly — only `read_column()` round trips are tested there. Add an explicit `read_value()` unit/integration test asserting point-read correctness independent of the full-column decode path |

## VV-STO-008 — Partition compaction (merge N→1, reclaim tombstones) — **⬜ on the branch reviewed; 🟡 exists unmerged on `feat/compaction`**
**Spec:** feature matrix marks this ✅ done — **that claim is only true for a different branch than the one checked out for this review.**

**What is actually on disk (branch as checked out during this pass):**
- `vectorvolt-engine/src/` contains only `engine.rs`, `error.rs`, `lib.rs`, `row.rs`, `stats.rs` — **no `compact.rs`**.
- `TableEngine` (`engine.rs:28-262`) has **no `compact()` method** of any kind.
- `vectorvolt-engine/Cargo.toml` has **no `vectorvolt-index` dependency** — confirmed by reading the file directly; the `feat/compaction` commit message itself states it *adds* that dependency, which is absent here.
- `vectorvolt-txn/src/manager.rs:148-149` still says, verbatim: *"Does not remove the row from any flushed `.vvsst` file — that requires compaction (deferred to a later phase)."*

**What exists on the separate, unmerged branch `feat/compaction`** (commit `bc91976`, read via `git show feat/compaction:...`, not present in the working tree inspected for this document):
- `vectorvolt-engine/src/compact.rs` — `CompactResult { rows_kept, path }`.
- `TableEngine::compact()` in `engine.rs:243-...` on that branch — flushes pending rows, collects all PK-live rows across partitions by cross-checking `pk_index`, sorts by PK, writes one merged partition, and (per the commit message) atomically deletes old files and rebuilds `CuckooMap` entries.
- `vvctl table compact <dir>` CLI wrapper (per commit message).

This is exactly the scenario the task brief warned about — the doc says done, the code on the branch under review says otherwise. Tasks below are decomposed against what's on `feat/compaction`, marked 🟡 (implemented and demonstrably working there, per its own commit-message demo trace) rather than ✅ (not verified against the branch actually reviewed) or ⬜ (code does exist, just not merged).

| Task | Status | Detail |
|---|---|---|
| VV-STO-008.1 | 🟡 | `CompactResult` struct — `feat/compaction:vectorvolt-engine/src/compact.rs` (not on reviewed branch) |
| VV-STO-008.2 | 🟡 | Live-row collection: cross-references `pk_index` against on-disk PK column per partition to filter tombstoned rows — `feat/compaction:vectorvolt-engine/src/engine.rs:257-282` |
| VV-STO-008.3 | 🟡 | PK-sort of merged output for zone-pruning — `feat/compaction:...engine.rs:288-289` |
| VV-STO-008.4 | 🟡 | Single merged-partition write + atomic old-file deletion + `CuckooMap` rebuild — per commit message (not independently re-read line-by-line beyond the excerpt above) |
| VV-STO-008.5 | 🟡 | `vvctl table compact <dir>` CLI wrapper — per commit message |
| VV-STO-008.6 | ⬜ | **Action item, not a code task:** merge `feat/compaction` into the branch under active development (`feat/hal-simd` / `feat/lazy-column-read`) or `main`, or explicitly decide it is superseded. Until merged, every downstream feature that assumes compaction exists (VV-STO-015 tiered compaction, VV-STO-016 checksum workflow, VV-WAL-013 WAL compaction) is blocked on a merge decision, not new engineering |

## VV-STO-009 — RLE encoding ⬜
**Spec:** feature matrix; no format support exists — `format.rs` only defines `ENCODING_RAW=0`/`ENCODING_DICT=1`
**Evidence:** none

| Task | Status | Detail |
|---|---|---|
| VV-STO-009.1 | ⬜ | Sampling heuristic to detect run-length-friendly columns in `ColumnValues::auto_encode` |
| VV-STO-009.2 | ⬜ | New `ColumnValues::RleEncoded { runs: Vec<(u64, u32)> }` variant |
| VV-STO-009.3 | ⬜ | New `ENCODING_RLE=2` format constant; column-meta layout reused (`data_offset`/`data_byte_len`) |
| VV-STO-009.4 | ⬜ | Writer support: serialize `(value:8B, run_len:4B)` pairs |
| VV-STO-009.5 | ⬜ | Reader support: `read_column()` decode + a prefix-sum index so `read_value()` stays O(log runs) instead of breaking the O(1) point-read invariant that `read_value` currently guarantees for Raw/Dict |
| VV-STO-009.6 | ⬜ | `zone_min_max()` over RLE (min/max of distinct run values) |
| VV-STO-009.7 | ⬜ | Test: round trip, zone stats, point-read correctness |
| VV-STO-009.8 | ⬜ | Format version bump (`VERSION=2`) + back-compat test: `VERSION=1` files still open under the new reader |

## VV-STO-010 — Bit-packing encoding ⬜
**Spec:** feature matrix
**Evidence:** none

| Task | Status | Detail |
|---|---|---|
| VV-STO-010.1 | ⬜ | Compute bits-needed-per-value (max bit-width) at write time |
| VV-STO-010.2 | ⬜ | New `ENCODING_BITPACKED=3` constant + a `bit_width` field (extends or repurposes reserved bytes in the 56-byte column-meta layout) |
| VV-STO-010.3 | ⬜ | Writer: pack `row_count` values into `ceil(row_count * bit_width / 8)` bytes |
| VV-STO-010.4 | ⬜ | Reader: `read_value()` unaligned bit extraction across byte boundaries |
| VV-STO-010.5 | ⬜ | Reader: `read_column()` bulk unpack |
| VV-STO-010.6 | ⬜ | Test: round trip across bit widths 1..63 |
| VV-STO-010.7 | ⬜ | Benchmark: bit-packed vs. raw u64 storage size and read latency |

## VV-STO-011 — Delta / Frame-of-Reference encoding ⬜
**Spec:** feature matrix
**Evidence:** none

| Task | Status | Detail |
|---|---|---|
| VV-STO-011.1 | ⬜ | Frame-of-reference: base = `min(values)`, per-row delta stored as u32/u16 depending on max delta |
| VV-STO-011.2 | ⬜ | New `ENCODING_DELTA=4` constant + base value stored in column meta |
| VV-STO-011.3 | ⬜ | Writer: compute base + deltas |
| VV-STO-011.4 | ⬜ | Reader: `read_value() = base + delta[row]` — stays O(1) |
| VV-STO-011.5 | ⬜ | `zone_min_max()` trivial (`base`, `base + max_delta`) |
| VV-STO-011.6 | ⬜ | Test: round trip, deltas always ≥0 by construction (base = min) |
| VV-STO-011.7 | ⬜ | Stretch: combine with bit-packing for the delta array (cross-reference VV-STO-010) |

## VV-STO-012 — LZ4 / Zstd block compression ⬜
**Spec:** feature matrix "No compression codec wired today" — confirmed: no `lz4`/`zstd` crate in `Cargo.toml`
**Evidence:** none

| Task | Status | Detail |
|---|---|---|
| VV-STO-012.1 | ⬜ | Codec decision: `lz4_flex` (pure Rust, no C deps, fits the project's near-zero-`unsafe` posture) vs. `zstd` (better ratio, needs `zstd-sys`/C bindings) |
| VV-STO-012.2 | ⬜ | Block framing: compress each column's data section as one block; store `compressed_len`/`uncompressed_len` in column meta |
| VV-STO-012.3 | ⬜ | Format version bump (`VERSION=2`) with `ENCODING_LZ4=5`/`ENCODING_ZSTD=6` |
| VV-STO-012.4 | ⬜ | Writer: compress column bytes into the block |
| VV-STO-012.5 | ⬜ | Reader: decompress-on-read for `read_column()` — this **breaks the O(1) `read_value()` guarantee** (VV-STO-007) unless a decompressed cache or sub-block index is added; must be designed explicitly, not bolted on |
| VV-STO-012.6 | ⬜ | Compression-ratio metric collection (bytes in/out per column) |
| VV-STO-012.7 | ⬜ | Benchmark: ratio + decode latency across realistic column distributions |
| VV-STO-012.8 | ⬜ | Back-compat test: `VERSION=1` uncompressed files still open under the `VERSION=2` reader |

## VV-STO-013 — SIMD-accelerated column scan ⬜
**Spec:** feature matrix "Needs VV-HAL-010" (delivered on branch `feat/hal-simd`, not merged into `vectorvolt-storage`)
**Evidence:** `TableEngine::scan()` decodes all columns eagerly with a scalar loop — `vectorvolt-engine/src/engine.rs:206-219`

| Task | Status | Detail |
|---|---|---|
| VV-STO-013.1 | ⬜ | Wire `vectorvolt-hal`'s `filter_range`/`find_first_eq` kernels into `PartitionReader` or a new `scan_range()` API |
| VV-STO-013.2 | ⬜ | Runtime SIMD-tier dispatch at scan call time |
| VV-STO-013.3 | ⬜ | Update `TableEngine::scan()` (`engine.rs:185-223`) to call the SIMD scan API instead of the manual `for offset in 0..scan_vals.len()` loop |
| VV-STO-013.4 | ⬜ | Test: SIMD scan produces identical results to the current scalar scan across data regimes |
| VV-STO-013.5 | ⬜ | Benchmark: scan latency before/after (S1a measured 1.2–2.6× for the underlying kernels in isolation; end-to-end scan gain still unmeasured) |

## VV-STO-014 — Prefetch-driven sequential scan ⬜
**Spec:** feature matrix "Needs VV-HAL-018..023" (delivered on `feat/hal-simd`, not merged here)
**Evidence:** `read_column()`'s sequential `Cursor` read loop has no prefetch hints — `reader.rs:250-282`

| Task | Status | Detail |
|---|---|---|
| VV-STO-014.1 | ⬜ | Wire `vectorvolt-hal` prefetch hints (`PREFETCHT0/T1/T2/NTA`) into `read_column()`'s sequential read loop |
| VV-STO-014.2 | ⬜ | Tune prefetch distance (cache lines ahead) empirically |
| VV-STO-014.3 | ⬜ | Non-temporal (NTA) hint path for full-table scans unlikely to be re-read soon |
| VV-STO-014.4 | ⬜ | Benchmark: prefetch-distance sweep vs. scan latency at 8KB/64KB/512KB/8MB column sizes (mirror the S1a HAL benchmark table shape) |
| VV-STO-014.5 | ⬜ | Test: functional correctness unaffected by prefetch hints (advisory only, must not change results) |

## VV-STO-015 — Tiered compaction strategy (leveled/size-tiered choice) ⬜
**Spec:** feature matrix "Engine has naive merge-all" — **depends on VV-STO-008 first being merged**
**Evidence:** none in the reviewed branch; the naive "merge everything" strategy referenced by the matrix exists only on `feat/compaction` (see VV-STO-008)

| Task | Status | Detail |
|---|---|---|
| VV-STO-015.1 | ⬜ | `CompactionStrategy` enum `{SizeTiered, Leveled}` |
| VV-STO-015.2 | ⬜ | Size-tiered: group partitions into similar-size buckets, merge within bucket past a count threshold |
| VV-STO-015.3 | ⬜ | Leveled: L0..LN levels with per-level size targets, promote on overflow |
| VV-STO-015.4 | ⬜ | Victim-partition selection policy (configurable size-tier ratio threshold) |
| VV-STO-015.5 | ⬜ | Replace the naive "merge everything" `compact()` (once VV-STO-008 is merged) with strategy-driven partial compaction |
| VV-STO-015.6 | ⬜ | Metrics: compaction write-amplification, partitions-merged-per-run |
| VV-STO-015.7 | ⬜ | Test: size-tiered convergence (partition count bounded under sustained inserts) |
| VV-STO-015.8 | ⬜ | Test: leveled strategy respects per-level size targets |

## VV-STO-016 — Cross-partition CRC-32C + `ChecksumVerifyWorkflow` ⬜
**Spec:** STORAGE_ENGINE_ADVANCED.md §5
**Evidence:** none — existing file CRC (VV-STO-004) is CRC-32/ISO-HDLC via `crc32fast`, computed once at write and checked at open, not periodically re-verified in the background

| Task | Status | Detail |
|---|---|---|
| VV-STO-016.1 | ⬜ | `PartitionCatalogEntry{partition_id, row_count, checksum_crc32c, written_at, last_verified}` per spec §5 |
| VV-STO-016.2 | ⬜ | Compute + persist CRC-32C at write time — **must not be conflated with** the existing CRC-32/ISO-HDLC file checksum (different polynomial, different values, per S1a HAL finding #5) |
| VV-STO-016.3 | ⬜ | Background `ChecksumVerifyWorkflow`: periodic re-read + compare against catalog |
| VV-STO-016.4 | ⬜ | Alert path: `partition_corruption` event on mismatch (stub emitter until an observability layer exists) |
| VV-STO-016.5 | ⬜ | Quorum-witness cross-check — deferred; depends on the unbuilt cluster layer |
| VV-STO-016.6 | ⬜ | Test: corruption detected on a scheduled verify pass (inject bit-flip after write, before verify) |
| VV-STO-016.7 | ⬜ | Test: verify workflow does not block concurrent reads |

## VV-STO-017 — Copy-on-Write snapshots (non-blocking) ⬜
**Spec:** STORAGE_TIERING.md Part 3
**Evidence:** none — VectorVolt has no arena/page allocator today (VV-HAL-029 unbuilt); storage is file-based `.vvsst`, not arena-paged, so the spec's page-level CoW model needs a storage-appropriate reinterpretation

| Task | Status | Detail |
|---|---|---|
| VV-STO-017.1 | ⬜ | Reinterpret CoW at the file level: a snapshot = the set of partition file paths at a point in time (partitions are already immutable once written, which does most of the work for free) |
| VV-STO-017.2 | ⬜ | Reference counting for in-flight snapshots vs. compaction-driven file deletion (compaction must not delete a file a live snapshot still references) |
| VV-STO-017.3 | ⬜ | Snapshot manifest serialization: `snapshot_id`, `lsn`, partition-file list, CRC |
| VV-STO-017.4 | ⬜ | Confirm live engine writes never block snapshot readers (largely true already; formalize with a manifest-based test) |
| VV-STO-017.5 | ⬜ | Test: snapshot manifest captures the exact partition set at a point in time |
| VV-STO-017.6 | ⬜ | Test: writes after the snapshot don't appear in the snapshot's view |

## VV-STO-018 — Incremental snapshots (1–5% dirty pages) ⬜
**Spec:** STORAGE_TIERING.md Part 3 — depends on VV-STO-017
**Evidence:** none

| Task | Status | Detail |
|---|---|---|
| VV-STO-018.1 | ⬜ | Dirty-partition tracking since the last snapshot (new partitions + compactions since `last_snapshot_id`) |
| VV-STO-018.2 | ⬜ | Incremental manifest: `base_snapshot_id` + delta partition list |
| VV-STO-018.3 | ⬜ | Restore: apply base + incrementals in order |
| VV-STO-018.4 | ⬜ | Test: incremental snapshot lists only new/changed partitions |
| VV-STO-018.5 | ⬜ | Test: restore-from-incremental-chain matches restore-from-full-snapshot-at-the-same-point |

## VV-STO-019 — L0–L4 five-level tier hierarchy ⬜
**Spec:** STORAGE_TIERING.md Part 1
**Evidence:** none

| Task | Status | Detail |
|---|---|---|
| VV-STO-019.1 | ⬜ | `StorageTier` enum `{L0 CpuCache, L1 DramArena, L2 NvmeLocal, L3 NetworkSsd, L4 ObjectStorage}` |
| VV-STO-019.2 | ⬜ | Tier-aware placement metadata (which tier currently holds a given partition/segment) |
| VV-STO-019.3 | ⬜ | L1→L2 transition trigger (arena fill / snapshot interval) — blocked on the arena layer not existing |
| VV-STO-019.4 | ⬜ | L2→L3/L4 transition: `ObjectArchiveWorker` skeleton, trait-based backend for S3/Azure/GCS |
| VV-STO-019.5 | ⬜ | `[shard.retention]` TOML config parsing — blocked on VV-ENG-017 (config system, unbuilt) |
| VV-STO-019.6 | ⬜ | Test: tier-transition state machine (mocked clock/triggers) |
| VV-STO-019.7 | ⬜ | Test: "never delete before archive" invariant enforced |

## VV-STO-020 — Retention policies (time/size/compliance/count) ⬜
**Spec:** STORAGE_TIERING.md Part 6
**Evidence:** none

| Task | Status | Detail |
|---|---|---|
| VV-STO-020.1 | ⬜ | `RetentionPolicy{wal_retention_days, snapshot_count_keep, archive_to_l3_after, archive_to_l4_after, compliance_lock, delete_after}` |
| VV-STO-020.2 | ⬜ | Periodic per-shard policy-evaluation loop |
| VV-STO-020.3 | ⬜ | Time-based deletion eligibility check |
| VV-STO-020.4 | ⬜ | Size-based (aggregate bytes over budget) eligibility check |
| VV-STO-020.5 | ⬜ | Snapshot-count-based (keep last N) eligibility check |
| VV-STO-020.6 | ⬜ | Test: each policy type independently, and combined (most-restrictive wins) |

## VV-STO-021 — WORM compliance lock (S3 Object Lock / Azure / GCS) ⬜
**Spec:** STORAGE_TIERING.md Part 6
**Evidence:** none

| Task | Status | Detail |
|---|---|---|
| VV-STO-021.1 | ⬜ | `ComplianceLockViolation` error + enforcement check before any L4 delete |
| VV-STO-021.2 | ⬜ | S3 backend: Object Lock Governance/Compliance mode |
| VV-STO-021.3 | ⬜ | Azure backend: Immutable Blob Storage policy |
| VV-STO-021.4 | ⬜ | GCS backend: Object Hold / retention policy |
| VV-STO-021.5 | ⬜ | Backend-agnostic `ComplianceBackend` trait so .2–.4 are swappable |
| VV-STO-021.6 | ⬜ | Test: delete attempt before lock expiry is rejected regardless of operator override |
| VV-STO-021.7 | ⬜ | Test: delete succeeds after `compliance_lock_duration` elapses |

## VV-STO-022 — Object-storage archive worker (L4) ⬜
**Spec:** STORAGE_TIERING.md Part 1, Part 6
**Evidence:** none

| Task | Status | Detail |
|---|---|---|
| VV-STO-022.1 | ⬜ | `ObjectArchiveWorker` trait: `upload`/`verify`/`delete` |
| VV-STO-022.2 | ⬜ | S3 implementation |
| VV-STO-022.3 | ⬜ | Azure Blob implementation |
| VV-STO-022.4 | ⬜ | GCS implementation |
| VV-STO-022.5 | ⬜ | Write-then-verify-then-delete-from-L2 sequence ("nothing ever deleted from L2 before archived to L4") |
| VV-STO-022.6 | ⬜ | Retry/backoff on transient upload failure |
| VV-STO-022.7 | ⬜ | Test: mocked object-store backend round trip (no real cloud credentials needed) |
| VV-STO-022.8 | ⬜ | Test: L2 file is NOT deleted if L4 verify fails |

## VV-STO-023 — PITR + PITR Catalog (any millisecond) ⬜
**Spec:** STORAGE_TIERING.md Part 2
**Evidence:** none

| Task | Status | Detail |
|---|---|---|
| VV-STO-023.1 | ⬜ | `PitrCatalog{snapshots: Vec<SnapshotRecord>, wal_segments: BTreeMap<Lsn, WalSegmentRecord>}` |
| VV-STO-023.2 | ⬜ | Population hooks on snapshot creation / WAL segment rotation |
| VV-STO-023.3 | ⬜ | Restore algorithm: latest snapshot ≤ T, gather WAL segments in `[snapshot.lsn, T]`, replay to first entry > T |
| VV-STO-023.4 | ⬜ | Target-LSN stop condition during replay — `WalReader::scan()`'s existing early-stop-via-bool-return (`reader.rs:47-89`) is directly reusable here |
| VV-STO-023.5 | ⬜ | PITR health metrics: `oldest_recovery_point_age`, `archive_lag_bytes` (stub emitters) |
| VV-STO-023.6 | ⬜ | Test: restore to an arbitrary millisecond matches expected state (deterministic small-scale scenario) |
| VV-STO-023.7 | ⬜ | Test: restoration performance sanity check at moderate scale (proportional smoke test, not the full 100GB/<60s claim) |

## VV-STO-024 — Merge-on-Read Delta Log (per-core, append-only) ⬜
**Spec:** READ_PATH_ARCHITECTURE.md §8
**Evidence:** none — updates currently have no delta-log path; `TxnManager::delete` mutates the index directly (`vectorvolt-txn/src/manager.rs:150-157`) and there is no `Update` mutation type visible in this pass

| Task | Status | Detail |
|---|---|---|
| VV-STO-024.1 | ⬜ | `DeltaLog{entries: Vec<DeltaEntry>}` per spec §8 |
| VV-STO-024.2 | ⬜ | `DeltaEntry{txn_id, pk, delta_kind: Update(field,value)\|Delete\|Insert, timestamp}` |
| VV-STO-024.3 | ⬜ | Per-core (or per-shard) append-only `DeltaLog` instance, no read-before-write |
| VV-STO-024.4 | ⬜ | Read-path reconstruction: base version via `CuckooMap` + scan pending deltas for the PK + apply |
| VV-STO-024.5 | ⬜ | Delta-log truncation once merged into a flush (bound unbounded growth) |
| VV-STO-024.6 | ⬜ | Test: update-without-prior-read correctness (write path never touches the base row) |
| VV-STO-024.7 | ⬜ | Test: read reconstructs the correct final value across multiple pending deltas on the same PK |
| VV-STO-024.8 | ⬜ | Benchmark: delta-application latency (spec target ~20ns) |

---

## Cross-cutting notes

1. **Branch discrepancy (load-bearing):** the branch checked out for this review has `vectorvolt-engine/Cargo.toml` with no `vectorvolt-index` dependency and no `compact.rs`/`compact()` anywhere. Partition compaction (VV-STO-008) is real, tested, and demoed per its own commit message — but only on the separate, unmerged branch `feat/compaction` (commit `bc91976`). `docs/VECTORVOLT_FEATURE_MATRIX.md` marking VV-STO-008 ✅ "done" is accurate for that branch, not for the one reviewed here. Everything downstream of compaction (VV-STO-015, VV-STO-016, VV-WAL-013) is blocked on this merge decision.
2. **`feat/hal-simd` SIMD/prefetch kernels are not visible from L2/L3:** Sprint S1a delivered real, tested, benchmarked SIMD range-filter/equality-scan/min-max/sum kernels and L1/L2/L3/NTA prefetch hints in `vectorvolt-hal` — but nothing in `vectorvolt-index` or `vectorvolt-storage` calls into that crate yet. VV-IDX-009, VV-STO-013, VV-STO-014 all cite the same underlying gap: the kernels exist, the call sites don't.
3. **`feat/lazy-column-read` in progress:** the environment's active branch name suggests work on making `TableEngine::scan()` lazy. As read in this pass, `scan()` still eagerly decodes every column via `read_column()` for every partition it doesn't zone-prune (`vectorvolt-engine/src/engine.rs:206-219`), in contrast to `get()`, which is already O(1) via `read_value()` (VV-STO-007). This is Layer 5/6 territory (`VV-QRY-007` in the feature matrix, already correctly marked 🟡 there) but the underlying enabler — `PartitionReader::read_value()` — already exists in Layer 3 and is ready to be called per-row from `scan()` without any new storage-layer work.
4. **Two independent zone-map implementations exist** — the standalone `vectorvolt-index::ZoneMap` (VV-IDX-004) and the per-column zone stats embedded in `.vvsst` file metadata (VV-STO-006). Both are real, tested, and correct; they are not the same code path. Worth consolidating or at least documenting the split explicitly if this becomes confusing during Phase 3+ work.
5. **WAL CRC vs. hardware CRC-32C:** the WAL's frame CRC (VV-WAL-003) is software CRC-32/ISO-HDLC via `crc32fast`. The 44.6×-faster hardware CRC-32C delivered in `vectorvolt-hal` (S1a) uses a different polynomial and cannot be substituted in without a WAL format version bump — flagged explicitly under VV-WAL-003.4 so it isn't mistaken for a trivial swap.
