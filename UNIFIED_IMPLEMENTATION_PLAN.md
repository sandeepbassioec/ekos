# VectorVolt — Unified Implementation Plan
*Version 1.0 | 2026-10-31 | Status: DRAFT FOR REVIEW*

> **What this is.** One plan that reconciles:
> - `IMPLEMENTATION_PLAN.md` (root, v2.0/3.0): the C#-era 8-phase learning plan.
> - `vectorvolt/IMPLEMENTATION_PLAN.md`: the Rust sprint log, VV-CFG-010, S22 and S23 designs, and the known gaps.
> - `REQUIREMENTS.md` (v3.0).
> - The locked decisions in `docs/vision/VECTORVOLT_LOCKED_DECISIONS.md`.
> - A code audit of `main` @ `3db4bef` (2026-09-30).
>
> **Nothing is deleted.** Both source plans stay in the repo unchanged as annexes. Every concept, sprint, VV-ID and acceptance criterion from them appears here, either in full or as a row pointing at its full text.
>
> **How the status was derived.** Every status below comes from reading the code on 2026-10-31. No build or test run was done for this document. Wherever a status says "verified", a reviewer checked the cited line directly.

**Status legend:**
- ✅ implemented and wired
- 🟡 partial: exists but is unwired, incomplete, or has known defects
- 🧪 stub or simulation that presents itself as real
- ⬜ not started
- 🧊 hardware-gated
- 🔬 research

---

## 0. Ground truth (where the project actually is)

| Item | Fact |
|---|---|
| Language/runtime | **Rust** (edition 2024, stable toolchain), static musl targets. The original C# prototype (`src/`, `tests/`) is frozen after Phase 1 concepts 1.1 and 1.3. |
| Code size | 35 crates in `vectorvolt/crates`, about 234K LOC, 1,108 commits, last commit 2026-09-30 (PR #646) |
| Sprint history | S1–S10 (the original roadmap) was all merged by 2026-09-04. A second series S11–S380 followed: Linux port, XDP, crypto, gRPC, TPM, OTel, Prometheus, S22 semantic partitioning, S23 domain events, then a long SQL surface run. After that came MVCC Phases 1–3, "Gaps #1–6", vv-architect, and WAL encryption part A. |
| Current item | **Path-to-production item #1, encryption at rest.** Part A (WAL payload AES-256-GCM) is done but **not wired into `vvctl serve`**. Part B (row data, CDC) is not started. |
| Stale trackers | `vectorvolt/IMPLEMENTATION_PLAN.md` ("L121 next"), `docs/MASTER_IMPLEMENTATION_PLAN.md`, `docs/status/*`, `docs/traceability/*`, `docs/vectorvolt-tasks/*` and `docs/VECTORVOLT_FEATURE_MATRIX.md` all predate most of the code. **From now on this file is the tracker** (see §9). |
| Missing input | The full "path to production" list was never committed. Track P below is a reconstruction; **the owner must confirm or extend it.** |

---

## 1. Guiding principles (merged)

From the root plan:
1. **Learning first, speed second.** Each concept is taught physics → code → proof.
2. **Test first, code second.** Write failing tests, then the implementation.
3. **One concept at a time.**
4. **Working code only.** Every step compiles, is tested, and is benchmarked.
5. **This may take years**, and that is acceptable.

From the Rust plan, plus new rules:

6. **Layer workflow.** Feature branch → implementation → fmt + clippy + test → commit with `Refs #N` / `Closes #N` (GitHub Issues) → push → PR.
7. **Pre-commit gate.** Run from `C:\VectorVolt\ekos\vectorvolt\`; all four must pass:
   ```powershell
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   python ..\docs\traceability\generate.py   # only if catalog/code changed (traceability.yml CI fails on drift)
   ```
8. **New: correctness before breadth.** Add no new surface area (SQL statements, subcommands, crates) while any Track P0 durability item is open.
9. **New: honest status.** A feature is ✅ only when it is wired into a shipped binary and an integration test exercises the real path. Stubs must say so in their CLI output and API response (for example, a 501 or `"stub": true`). Tests named after a feature must test that feature.
10. **New: one tracker.** Update this file at the end of every session. §9 lists the docs to retire or regenerate.

---

## 2. Requirements reconciliation: decisions that need ratification

`REQUIREMENTS.md` v3.0 still describes .NET 10. The Rust implementation and the locked decisions (LDs) contradict it in the places below. **Each row needs an owner decision.** Once decided, update `REQUIREMENTS.md` to v4.0. The "Proposed" column is a recommendation, not a decision.

| # | Topic | REQUIREMENTS.md says | Implementation / LDs say | Proposed |
|---|---|---|---|---|
| D-01 | Runtime | .NET 10, NativeMemory, NativeAOT (LD-DIST-01) | Rust, static musl binary | **Ratify Rust.** Rewrite §3 and the Technology Stack in Rust terms (`#[repr(C, align(64))]`, `std::arch` intrinsics, mmap/jemalloc arenas). Re-express inviolable rule 1 as "no heap allocation on the hot path" and rule 10 as "no dynamic dispatch / reflection-equivalent on the hot path". |
| D-02 | Consensus | §16: Raft/Paxos is permanently out of scope | LD-SWIM: SWIM plus per-shard Raft (`raft-rs`) | Decide. Either accept Raft (LD-SWIM) and strike it from §16, or keep the REQUIREMENTS stance (log replication plus epoch fencing only) and delete the Raft simulation. |
| D-03 | SQL | Rule 8: SQL is not the source of truth; SQL exists only as a projection target | LD-PG-WIRE: pgwire :5432 and a large SQL layer | Rule 8 refers to *external SQL databases*, so it can coexist with an engine-native SQL query surface. Clarify the wording. Separately decide whether SQL DML stays (LD-PG-WIRE says SELECT plus limited DDL only). |
| D-04 | Wire format | FlatBuffers zero-copy; protobuf for SDKs | Custom 24-byte "VV" frame (about 125 opcodes), an unused 64-byte VVNE frame, tonic gRPC | Pick **one** binary protocol plus gRPC. Retire the VVNE frame, or make it the only one. Drop FlatBuffers from the requirements unless it is adopted. |
| D-05 | User code in engine | LD-INTC-01: user code never runs in the core engine; Wasm is research (LD-MIG-05) | Vision docs propose CIP/ARP WASM plugins, `.so` PODs, and LLM-generated core code | Keep LD-INTC-01 for VectorVolt. Confine CIP/ARP to EKOS (`ekosd`), outside the engine. |
| D-06 | Admin UI | LD-CLI-01: `vvctl` is the only tool, no admin portal | Svelte admin SPA on :8443 is planned; `vectorvolt-control` HTTP API exists | Decide. The proposal is a read-only web dashboard (REQUIREMENTS §5.26 already allows one) with all mutations through `vvctl`. |
| D-07 | Recovery semantics | LD-REC-01: WAL stores commands, and replay runs them through ShardExecutor | WAL stores state mutations (`Mutation` opcodes) | Decide. Mutation logging is simpler and is what exists. If kept, amend LD-REC-01. Command logging is required only if YAML command handlers (Concept 2.11) become the write model. |
| D-08 | Materialized views | LD-MV-01: views update atomically with the source mutation | Views update asynchronously from CDC | Decide. Either make views synchronous inside the commit, or reclassify them as eventually consistent (matches LD-PROJ). |
| D-09 | SDK tiers | LD-SDK-03: C#, Java, Python, Node, Go, Rust, C++ | LD-SDK-01: C#, Java, TS, Python, Go (Rust primary) | Adopt the LD-SDK-01 list and move C++ to Tier 2. |
| D-10 | Performance targets | 20–50 ns engine execution, < 1–2 µs end to end | LDs: p99 < 10 µs memory path, < 1 ms with fsync, 500K cmd/s per shard. FEATURE_MATRIX: < 100 ns reads | Make the LD targets the committed SLOs and the REQUIREMENTS ns targets the research targets. Record the hardware tier (HARDWARE_SUPPORT_MATRIX Tier B). |
| D-11 | Multi-tenancy model | LD-MT-01..08: engine-level isolation | Nothing is implemented; RepairLink doc uses app-level `tenant_id` | Keep LD-MT; schedule it in Phase 6 (§5). |
| D-12 | Phase numbering | Root plan Phases 1–8 | MASTER plan Phases 1–7+; vision docs disagree with each other | Use **this document's §5 phases** (which are the root plan's numbering) as canonical. |

### Decisions taken (owner, 2026-10-31)

| # | Decision |
|---|---|
| D-01 | **Rust confirmed.** REQUIREMENTS.md is out of date; the implementation plan is authoritative. REQUIREMENTS.md v4.0 must be rewritten to match. |
| D-02 | **Raft is required** (LD-SWIM stands). Strike Raft from REQUIREMENTS §16. The cluster network transport (6.11) and a real Raft-driven WAL become committed work. |
| D-03 | **Full SQL engine support** is a product commitment (DML + DDL + txns over pgwire/HTTP). Amend rule 8 to refer to *external* SQL stores only. PP-70 changes from "mark as unsupported" to "make every statement real". |
| — | Track P bugs (P0), transaction atomicity (P1) and the network-port exposure (PP-31/32) are approved to fix now. |
| — | PP-33 (PAM credentials) and PP-41 (ConsoleApp1) are **intentional, deferred by owner**. |
| — | All features identified in this plan are to be incorporated. |
| D-04…D-12 | Still open. |

### Progress log

**2026-10-31 — branch `fix/path-to-production-p0` (not committed yet).** Full test suite on Windows: 9,494 passed and 3 failed. All 3 were then resolved: 2 were bugs in the new tests, 1 was a timing flake. `vectorvolt-pam` was excluded from the run because it needs Linux libpam.

**Fixed:**
- **PP-01** Snapshot marker v2 now records the WAL position it covers; the WAL is no longer wiped on open.
- **PP-02** A corrupt WAL frame now fails the open instead of silently skipping replay.
- **PP-03** A torn tail is truncated to the last intact frame. Mid-log corruption still refuses to open.
- **PP-04** Chain hash is carried across an empty segment.
- **PP-05** Partition files and their directory are fsynced before publishing.
- **PP-06** Flush checkpoint is now a typed record.
- **PP-07** Partition-id allocation skips any id whose file still exists; it errors only when all ids are in use.
- **PP-08, PP-09, PP-10** Replay now reconciles each pk's final WAL state against what is actually on disk, so updates are no longer lost, the newest copy wins, and nothing is re-buffered needlessly.
- **PP-11** Ring-overflow rows are carried into the next flush.
- **PP-12** PITR marker goes through the engine's single WAL writer. True restore-to-LSN is still open as PP-12b.
- **PP-13** An unreadable partition is quarantined and reported by `verify`.
- **Flush race:** a flush no longer resurrects deletes or rolls back newer writes made during it.
- **PP-20, PP-21** New `Mutation::TxnBatch`: one WAL record per transaction. All commit paths use it via `apply_batch`: engine MVCC, wire protocol, gRPC, HTTP multi-table and SQL.
- **PP-22** Intent log now stores write sets plus per-table progress records, so recovery re-applies only tables that had not landed.
- **PP-25** A process-wide SQL commit lock covers the SERIALIZABLE check-then-apply. Autocommit writes are still unlocked; full per-pk locking arrives with PP-23.
- **PP-31** gRPC: bearer token (`--grpc-token` / `VECTORVOLT_GRPC_TOKEN`) and `--grpc-host`. gRPC stays off without a token when `--secret` is set or the host is not loopback.
- **PP-32** Metrics bind loopback by default; `--metrics-host` overrides it.
- **New tests:** `engine/tests/recovery_p0.rs` (12), `engine/tests/txn_atomicity.rs` (4), WAL writer tests (4), TxnBatch codec (2), intent-log write-set recovery (1), gRPC token auth (1).

**Still open in P0/P1:**
- PP-14 (`vectorvolt-store` put flush), PP-15 (reference domains)
- PP-23 (true single-writer / per-pk ordering), PP-24 (MVCC Phase 1b on-disk versions)
- PP-12b (true point-in-time restore)
- Physical WAL truncation after a snapshot (WAL compaction, Phase 3.5)
- Pre-existing: 42 clippy errors in `sql/executor.rs` under toolchain 1.98. These are not from this work, but CI's `-D warnings` will fail on them.

---

## 3. Track P — Path to Production (do first, in order)

Every ID here is new: `PP-xx`. Where a VV-ID already exists, it is cited.

### P0 — Durability & data-loss bugs (blockers; verified in code)

| ID | Defect | Location | Fix | Acceptance test |
|---|---|---|---|---|
| PP-01 | Once a snapshot is taken, **every later open wipes the WAL**, so post-snapshot unflushed writes are lost | `engine.rs:358`, `snapshot.rs:67` | Make the snapshot marker record the LSN it covers. On open, truncate only frames ≤ that LSN, or remove the marker after a successful flush. | Snapshot → insert → reopen: the row is present. |
| PP-02 | Replay is wrapped in `if let Ok(read_all())`: **one bad frame skips the entire replay** | `engine.rs:422` | Stream frames. Stop at the first bad frame and report it. Never skip silently. | Corrupt frame N: frames < N are replayed and an error/metric is raised. |
| PP-03 | **Torn WAL tail makes the table unopenable** | `wal/writer.rs:126` | On open, detect a partial last frame and truncate to the last valid frame (after verifying CRC and hash), then log it. | Truncate the file mid-frame → the table opens with N-1 frames. |
| PP-04 | Chain hash resets to GENESIS after a crash right after segment rotation | `wal/writer.rs:128-132` | Carry `prev_hash` from the previous segment's last frame. | `verify()` passes across a crash-at-rotation. |
| PP-05 | `.vvsst` flush has **no fsync of the file or directory** before rename and manifest add | `txn/manager.rs:1023-1027` | `sync_all` the file, rename, then fsync the directory, then write the manifest. | Fault-injected crash after the manifest write → no missing or empty partitions. |
| PP-06 | Flush checkpoint is an **ad-hoc plaintext string** (VV-WAL-011) | `txn/manager.rs:1029` | Use the typed `Mutation::Checkpoint{file_id, pid, rows, lsn}`. Replay uses it to skip already-flushed mutations. | Replay after flush does not re-buffer flushed rows. |
| PP-07 | **Partition ID is an AtomicU16 that wraps** and `file_id` is always 0, so files are eventually overwritten | `txn/manager.rs:987-989` | Use a u32/u64 monotonic id persisted in the manifest; refuse to reuse an existing filename. | 70K flushes → no overwrite. |
| PP-08 | **UPDATE / CAS / upsert / field ops are written as Insert-on-existing-pk**, and replay skips Inserts for known pks, so updates are lost on restart (VV-TXN-004.6) | `engine/cond.rs:42-100`, `field_update.rs:40-90`, `sql/executor.rs:14077-14082`, `engine.rs:437-440` | Route every overwrite through `Mutation::Update`/`Upsert`. Make replay last-writer-wins by LSN. Count orphans in `pending_tombstones`. | Update a flushed row → crash → reopen → the new value is returned. |
| PP-09 | `rebuild_pk_index_from_partitions` iterates a HashMap, so **which version wins for a pk is random** | `txn/manager.rs` | Order by partition sequence/LSN, newest wins. | Same pk in 2 SSTs → the newest is always returned. |
| PP-10 | Upsert/Update/Batch frames are replayed unconditionally, so they are re-flushed on every restart | `engine.rs` replay | Covered by PP-06 (checkpoint LSN skip). | Restart ×3 → partition count is unchanged. |
| PP-11 | Ring-full drain moves rows to `in_flight` and nothing flushes them | `txn/shard_writer.rs:125-133` | Flush `in_flight` or apply back-pressure (ties to Concept 2.5). | Insert > ring capacity → all rows durable after flush. |
| PP-12 | PITR `create_snapshot` **opens a second WalWriter** on the live WAL, forking seq and chain; restore is not point-in-time | `engine/pitr.rs:23-62` | Write markers through the engine's single writer. Implement restore-to-LSN. | `verify()` stays green; restore to LSN X matches state at X. |
| PP-13 | A single bad `.vvsst` fails the whole open | `engine.rs` open | Quarantine the file, report it, and continue read-only or refuse according to policy. | Corrupt one file → open reports it and the rest is served. |
| PP-14 | `vectorvolt-store` `put` never flushes the WAL BufWriter; the WAL is never truncated | `store.rs` | Flush per durability mode; truncate on checkpoint. | Crash after put (fsync mode) → value present. |
| PP-15 | Reference domains (`ledger`, `orders`, `workflow`) rewrite JSON non-atomically, with no fsync, and outside the engine | respective crates | Rebuild them on `TableEngine` and WAL (Phase 7.2), or mark them as demos. | Ledger trial balance holds after crash injection. |

### P1 — Transaction atomicity & isolation

| ID | Defect | Fix | Acceptance |
|---|---|---|---|
| PP-20 | Wire `CommitTxn`, gRPC `CommitTxn` and HTTP `/multi_txn` loop over insert/delete (wire ignores errors) and are **not atomic** (`net/server.rs:658-697`, `server/grpc.rs:138-172`, `http/multi_txn.rs:58-79`) | Single commit path. Write a TXN-BEGIN/ops/TXN-COMMIT frame group, or one batch frame, and make replay all-or-nothing. All three surfaces call the same `TxnManager` API. | Crash mid-commit → none of the txn is visible. |
| PP-21 | Engine `commit_txn` writes one WAL record per op with no rollback (`engine.rs:1517-1530`) | Same as PP-20. | Same as PP-20. |
| PP-22 | MVCC Phase 3 intent log has no write-set, and txn IDs reset on restart, so recovery marks lost txns Committed | Log the write-set in PREPARE, use durable txn IDs, implement real roll-forward/rollback, write `RolledBack`. | Kill between PREPARE and table 2 commit → after restart, both tables are consistent. |
| PP-23 | No per-pk write serialization; commit_ts is taken after WAL append (`txn/manager.rs:271-275`) | Per-pk/shard ordering: assign the ts under the same critical section as the WAL append. This is where true single-writer (Concept 2.1) lands. | Concurrent same-pk writers → version order equals WAL order. |
| PP-24 | MVCC history is wrong after restart: SST rows have no versions, and replayed deletes add no tombstone | **MVCC Phase 1b**: on-disk multi-version `.vvsst` (format bump), tombstones on replay. | `get_at`/`scan_at` are correct after restart. |
| PP-25 | SERIALIZABLE check-then-apply has a TOCTOU window (`sql/executor.rs:11832-11862`) | Validate and apply under the txn commit lock. | Concurrent conflicting txns → exactly one SQLSTATE 40001. |

### P2 — Security hardening

| ID | Defect | Fix |
|---|---|---|
| PP-30 | `--secret` gives the key `Scope::ReadWrite`, but `UseTable`/DDL require Admin, so **`--multi` with auth is unusable** (`serve.rs:504`, `net/server.rs:546-551`) | Add `--key-scope` / multiple keys (key file). Decide whether UseTable needs ReadOnly. |
| PP-31 | gRPC :50051 starts **by default with no auth or TLS** (`serve.rs:792-818`) | Default off, or require the same auth and TLS as the wire listener. |
| PP-32 | Metrics bind **0.0.0.0:9100** by default (`metrics/lib.rs:114`) | Bind loopback by default, make it configurable, and add optional auth. |
| PP-33 | `vectorvolt-pam` hardcodes `admin/admin` and `vectorvolt/password` and seeds every `/etc/passwd` user (`pam/lib.rs:40-53`) | Remove them. Use the system user database. Run the Linux tests. |
| PP-34 | HTTP and control bearer tokens are compared with `==` (not constant time); tenant tokens are stored in plaintext | Use constant-time compare; hash stored tokens. |
| PP-35 | pgwire: no auth, no SSLRequest, single-read framing (`reader/session.rs`) | Add SCRAM or MD5 auth, TLS, and proper message framing (otherwise most drivers fail). |
| PP-36 | No mTLS anywhere; TLS only on the wire listener | TLS for HTTP, gRPC and pgwire; optional mTLS. REQUIREMENTS §6.5 requires TLS 1.3 on all external communication. |
| PP-37 | `RbacStore` enforces nothing | Wire RBAC into every listener (REQUIREMENTS §6.5, LD-MT). |
| PP-38 | WAL hash chain is unkeyed; truncation and last-frame edits are undetectable (documented in `tamper_detection.rs`) | Keyed chain (HMAC) when encryption is on, plus a signed tail checkpoint. |
| PP-39 | `TableEngine::fk_exists` returns the raw Cuckoo-filter answer over the network (VV-QRY-004.3 only fixed in the query crate). The filter starts at 64 entries and never grows, so it spills to an O(N) HashSet with about 3% FPR | Recheck positives against storage. Make the filter growable; remove deleted keys. |
| PP-40 | pk `u64::MAX` collides with the CuckooMap empty sentinel (debug_assert only) | Reject it explicitly or change the sentinel scheme. |
| PP-41 | **Leaked Azure DevOps PAT** in `ConsoleApp1/ConsoleApp1/Program.cs` (initial commit); `BackupCodes.pdf` at the repo root | Revoke the token, scrub history, and move personal files out of the repo. |

### P3 — Path-to-production item #1: Encryption at rest

| ID | Item | Status |
|---|---|---|
| PP-50 | Part A: WAL payload AES-256-GCM (`TxnManager::with_wal_encryption`, `TableEngine::open_with_encryption`), 4 tests | ✅ library (`cbef5fe`) |
| PP-51 | **Wire into `vvctl serve --encrypt-at-rest`** and `TableRegistry` (replace `open_typed`), including `--multi` (currently rejected at `serve.rs:389`) | ⬜ |
| PP-52 | Key ID plus encryption flag in the segment header. **Fail loudly** on a wrong key or on a plaintext/encrypted mismatch (today frames are silently dropped). Add AAD = (segment, seq). | ⬜ |
| PP-53 | **Part B**: `.vvsst` encryption. Decrypt into a private anonymous mapping on open (keeps zero-copy). Format version bump. | ⬜ |
| PP-54 | Encrypt `cdc.log` (plaintext row JSON today), manifest, txn intent log, PITR markers, and the FTS/vector/HNSW sidecars (VV-SEC-006b) | ⬜ |
| PP-55 | `KeyStore`: stop copying out of `memfd_secret` into the heap; mlock/VirtualLock; key rotation (re-encrypt on compaction); nonce accounting | ⬜ |
| PP-56 | Snapshot encryption (REQUIREMENTS §5.7 "with compression and encryption") | ⬜ |

### P4 — Honest surfaces (stubs presented as real)

| ID | Item | Action |
|---|---|---|
| PP-60 | `vectorvolt-control` `/cluster/nodes`, `/shards`, `remove`, `stats`, `snapshot` return hardcoded/fake data | Return 501 until real, or back them with `ClusterNode`/engine. |
| PP-61 | vvctl `ping`, `status`, `disconnect`, `bench-net`, `server status/stop`, `projection *` only print text | Implement against the wire protocol, or remove. |
| PP-62 | Orphan vvctl modules: `reader`, `sql`, `vector`, `control`, `cluster_topology`, `connect_simple` | Wire them into `main.rs` or delete them. |
| PP-63 | `vectorvolt/sdk/{python,go,typescript}` speak a length-prefixed JSON protocol **no server implements** | Rebuild on gRPC (LD-SDK-01) or the VV wire protocol. Root `sdk/csharp` and `sdk/python` are real (TCP / HTTP); consolidate them under one `sdk/` tree. |
| PP-64 | Rust `vectorvolt-sdk` opens a new TCP connection per call, so `begin_txn`/`commit_txn` always fail; `scan()` pk is wrong | Use a connection pool with a txn-pinned connection. |
| PP-65 | 45 `vvctl/tests/table_*_stub.rs` are named after features they don't test | Rename or delete them. |
| PP-66 | "HNSW" is brute force; Kafka sink is an mpsc stand-in; Postgres sink only logs SQL; Redis is a closure; webhook send always returns Ok (so the outbox and DLQ never see failures) | Label them stand-ins (S22/S23 follow-ups below). |
| PP-67 | `--transport xdp` is accepted but ignored; XDP rings are never mmap'd | Reject the flag until Phase 4.6 is done. |
| PP-68 | Comments claim CRC-32C but `crc32fast` is IEEE CRC-32 (`wal/frame.rs:109`, `net/frame.rs:12`) | Fix the comment now. Switching to HAL CRC-32C is a format bump; bundle it with PP-53. |
| PP-69 | `vectorvolt-otel` panics on a bad endpoint despite "fallback" text; 4 of 7 metrics are never populated | Fix. |
| PP-70 | `sql/executor.rs` is a 34,920-line monolith; many handlers return ack strings (e.g. "REDACTION ENABLED") without behavior; the audit log hardcodes timestamps | Split by statement family. Mark no-op statements as `unsupported`. |

### P5 — Build, CI, release

| ID | Item |
|---|---|
| PP-80 | The 11 workflows in `vectorvolt/.github/workflows/` **never run** (the git root is `C:\VectorVolt\ekos`). Move the needed ones (`release`, `linux-ci`, `bench`, `publish-*`, `sdk`) to the root `.github/workflows/`, fix their paths, and re-enable `ci.yml`/`sdk.yml` (currently disabled for billing). |
| PP-81 | `release.yml` lints with `--all-features`, which pulls in `tss-esapi`; split the TPM build. |
| PP-82 | Packaging ships only `vvctl`: no systemd unit, default config, Dockerfile or image. Brew sha256 values are placeholders. |
| PP-83 | vv-architect K8s/Terraform output: no container command, only port 5995, placeholder AMI. Fix it to run `vvctl serve` and expose HTTP/gRPC/metrics. Add HugePages/CPU-pinning resources per REQUIREMENTS §5.25 and LD-K8S-01. |
| PP-84 | `.cargo/config.toml` (vectorvolt and ekos) hardcodes a machine-specific MSVC path. Move it to a documented local override. |
| PP-85 | The perf gate (LD-GATE: CI rejects a > 5% p99 regression) must run in live CI. |

### Previously known correctness gaps (from `vectorvolt/IMPLEMENTATION_PLAN.md`, carried over)

| Gap | VV-ID | Status on 2026-10-31 |
|---|---|---|
| No false-positive re-check on Cuckoo Filter FK hits (`query/engine.rs:89-90`) | VV-QRY-004.3 | 🟡 Fixed in `query/engine.rs:101-110`; **still open** in `TableEngine::fk_exists` → PP-39 |
| No secondary scan batch fetch (`query/engine.rs:98-100`) | VV-QRY-003.5 | ⬜ still open. Also, `secondary_scan` doesn't re-check values (hash collisions, stale entries; `pk as u32` truncation). |
| Zone pruning doesn't save file-open cost | VV-ENG-014 | ✅ PR #630 |
| No binary search in partition for range scans | VV-QRY-002.6 | ✅ PR #631 |
| WAL checkpoint is an ad-hoc string, not a typed Mutation | VV-WAL-011 | ⬜ → PP-06 |
| UPDATE = delete+insert orphans storage | VV-TXN-004.6 | 🟡 partly fixed → PP-08 |
| Compaction only on unmerged `feat/compaction` branch | VV-ENG-004 | ✅ merged (#14/#21) and real. Note: `compact()` holds the partition write lock for the whole run and blocks reads. |
| 400× slower read path in vectorvolt-query | VV-QRY-001.4 | ✅ fixed (#25). Residual: `TxnManager::get` reopens and CRC-checks the partition per call; `scan_deduped`/`scan_live` and SQL UPDATE/DELETE do O(N) `get()`. |

---

## 4. Track F — Feature roadmap already designed in Rust (carried over in full)

### 4.1 Rust sprint roadmap S1–S10 (original) — all merged

| Sprint | Theme | Status (merged) | Real status after audit |
|---|---|---|---|
| S1 | HAL foundations: SIMD, topology, NUMA, huge pages, wait strategies | ✅ | 🟡 SIMD wired; NUMA/hugepages/wait strategies exist but are unused by data paths |
| S2 | SIMD wire-up + CLI UX: storage/query/index SIMD, Zstd, Criterion gate | ✅ | ✅ |
| S3 | Type system + config + txn read/write path + partition routing (layers L80–L121, VV-ENG-008…VV-CTL-017; full table in the annex) | ✅ (L121 alias merged #221) | ✅ with P0 defects |
| S4 | Network + MVCC: vectorvolt-net, TCP :50051, TLS, BEGIN/COMMIT/ABORT, Prometheus | ✅ | 🟡 commit not atomic (PP-20) |
| S5 | Storage tiering L0–L4, snapshots/PITR, Kafka/Postgres sinks | ✅ | 🟡/🧪 tiering unwired; PITR defects (PP-01, PP-12); sinks are stand-ins |
| S6 | Cluster: SWIM gossip, 4096 vnodes, Raft, < 2 s failover | ✅ | 🧪 no network transport; Raft simulated |
| S7 | Control plane + admin UI, Rust + C# SDK, RBAC, package manager hooks | ✅ | 🧪/🟡 fake endpoints; RBAC unenforced |
| S8 | SQL parser, pgwire :5432, HNSW vector index | ✅ | 🟡 pgwire minimal; HNSW brute force |
| S9 | Graph engine: CSR, GRAPH_TRAVERSE(), cascade delete, SIMD joins | ✅ | 🟡 real but small |
| S10 | Polyglot SDKs (Python/Go/Java/TS), chaos + DST, reference domains | ✅ | 🧪 SDK protocol mismatch; chaos not hooked into I/O; domains are JSON toys |

Later merged sprints: S11 Linux port (io_uring, O_DIRECT, NUMA, mlock, affinity) · S12–13 eBPF/XDP · S14 AES-GCM, memfd_secret · S15 SIMD close · S16 gRPC · S17 TPM sealing · S18 completions, support bundle · S19 OpenTelemetry · S20 cluster CLI · S21 Prometheus · **S22** · **S23** · S24 EXPLAIN, schema registry · S25 CDC bridge, SQL txns, FTS · S26 vector search · S27 planner · S28 POST /sql · S29 HNSW · S30–32 UPDATE, aggregates, ALTER · S33 joins · S34–S380 SQL surface · MVCC Phases 1–3 · Gaps #1–6 · vv-architect · encryption part A.

### 4.2 Flagged milestone — Master Config Schema (VV-CFG-010)

> Original note: "Do after S3 control-plane layers complete. Do NOT implement yet — shape is dictated by runtime APIs that must exist first." S3 is now complete, so this is **unblocked**.

Expand `engine.yaml` into a versioned multi-section configuration authority consumed by `vvctl serve`, `vvctl catalog` and the server binaries. Sections: `hal` (cpu_cores, numa_node, huge_pages), `threads` (writer_count, ring_buffer_size, writer_priority), `observability` (prometheus_addr, trace_level, jaeger_endpoint), `storage` (compaction_strategy, l0_flush_threshold, block_size), `network` (bind_addr, tls_cert, tls_key, max_connections, keepalive_secs), `tables` (per-table flush_threshold, durability, partitions, entity, aggregate_root, members, columns), `query` (default_projection, max_scan_rows, index_hint). The full target YAML is in the annex `vectorvolt/IMPLEMENTATION_PLAN.md` §"Flagged Milestone".

| Section | Depends on | Status |
|---|---|---|
| hal | vectorvolt-hal affinity API | ✅ ready to wire |
| threads | txn ring buffer + shard writers | ✅ ready |
| observability | tracing, Prometheus | ✅ exists (S19/S21) — ready |
| storage | storage compaction | ✅ exists — ready |
| network | vectorvolt-net | ✅ exists — ready |
| tables | ShardCatalog + AggregateRouter | ✅ ready |
| query | DSL/SQL executor | ✅ ready |

Approach (unchanged):
1. A versioned top-level `EngineConfig` with optional sections.
2. Each section is its own `Deserialize` struct with defaults.
3. Per-section env overrides (`VECTORVOLT_HAL_CPU_CORES`, …).
4. Per-table overrides of the global flush, durability and partitions settings.
5. `vvctl tune --apply` writes a fully populated template from `SystemProfile`.

Also add the new security keys from PP-30 to PP-36 and encryption from PP-51.

### 4.3 S22 — Semantic Partitioning + Materialized Read Models + External Cache Tier

Merged as S22, but the audit shows `vectorvolt-semantic` is **not used by vvctl**, the Redis cache is a stand-in, and the HTTP-callback reducer is real but unwired. The full problem statement, goals and YAML shape are in the annex `vectorvolt/IMPLEMENTATION_PLAN.md` §S22.

| VV-ID | Name | Audit status |
|---|---|---|
| VV-PRJ-010 | SemanticPartitionKey (`partition_by:`) | 🟡 crate exists, unwired |
| VV-PRJ-011 | DateRangePartition (year/month/day buckets) | 🟡 router exists, unwired |
| VV-PRJ-012 | CustomerPartition (Fibonacci hash on FK) | 🟡 router exists, unwired |
| VV-PRJ-013 | PartitionPruner (skip non-matching buckets) | ⬜ verify |
| VV-PRJ-014 | ReadModelRegistry (register/start/stop/replay/snapshot) | 🟡 |
| VV-PRJ-015 | InProcessReadCache (LRU, TTL, `Arc` snapshots) | 🟡 |
| VV-PRJ-016 | RedisReadCache (write-through on miss) | 🧪 stand-in |
| VV-PRJ-017 | YamlReadModelConfig (`read_models:`) | ⬜ verify wiring |
| VV-PRJ-018 | CustomReducer (Rhai `reduce(state,row)`) | 🟡 |
| VV-PRJ-019 | ReducerCallbackSink (HTTP POST `{state,row}`) | 🟡 |
| VV-PRJ-020 | ExpressionPartitionStrategy (Rhai formula mod shard_count) | ⬜ verify |
| VV-PRJ-021 | ExplicitShardMap (value → shard) | ⬜ verify |
| VV-CFG-040 | PartitionConfig YAML section | ⬜ verify |
| VV-CFG-041 | CacheConfig YAML section | ⬜ verify |

Layers S22-L1…L16 (as designed): PartitionConfig → SemanticPartitionKey/PartitionStrategy → DateRangeRouter → CustomerRouter → ExpressionRouter → ExplicitShardMap → PartitionPruner → ReadModelRegistry → InProcessReadCache → RedisReadCache → RhaiReducer → HttpCallbackReducer → CacheConfig → YamlReadModelConfig + `vvctl serve` auto-register → `vvctl projection list/status/replay/invalidate-cache` → integration tests.

Dependencies: VV-ENG-023 AggregateRouter ✅, VV-SNK-041 WalSinkBridge ✅ (note: it re-reads the whole WAL on every poll), VV-ENG-020 ✅, `redis = "0.25"`, `dashmap = "5"`.

**Acceptance criteria (still open until wired into serve):**
- [ ] A `WHERE created_at BETWEEN '2025-01-01' AND '2025-01-31'` query on a monthly-partitioned table reads exactly one partition file (partition-access counter).
- [ ] `customer_order_count` is served from the in-process cache (< 1 µs) after hydration.
- [ ] A Redis cache hit is verified against a real Redis or a trait double.
- [ ] Removing or modifying `read_models:` and restarting drops the stale model.
- [ ] YAML-only; no code changes needed to add a read model.

### 4.4 S23 — Typed Domain Events + YAML Event Routing + Real Broker Integration

Merged as S23. The audit shows: the webhook sink is real (HMAC-SHA256, retry) but **fire-and-forget always returns Ok**; Redis Streams is a closure stand-in; the Kafka sink is mpsc; the outbox and DLQ are in-memory; only `SchemaVersionRegistry` is used by vvctl. The full YAML shape is in the annex §S23.

| VV-ID | Name | Audit status |
|---|---|---|
| VV-EVT-001 | DomainEventSchema (name, semver, trigger, direct/computed/nested/lookup fields) | 🟡 |
| VV-EVT-002 | DomainEventEmitter (post-commit, `when:` Rhai filter) | 🟡 unwired |
| VV-EVT-003 | DomainEventEnvelope (JSON + MessagePack, schema_version) | ✅ crate |
| VV-EVT-004 | KafkaBrokerSink (real rdkafka, `kafka` feature) | 🧪 mpsc |
| VV-EVT-005 | RedisStreamSink (XADD, maxlen, consumer group) | 🧪 closure |
| VV-EVT-006 | HttpWebhookSink (HMAC, back-off) | 🟡 errors swallowed |
| VV-EVT-007 | InProcessChannelSink (broadcast) | ✅ crate |
| VV-EVT-008 | EventOutbox (WAL OUTBOX frame, at-least-once) | 🧪 in-memory, not in WAL |
| VV-EVT-009 | YamlEventRouteConfig (`domain_events:`) | 🟡 unwired |
| VV-EVT-010 | ConsumerCrudCallback (E2E via gRPC WriteService) | ⬜ |
| VV-EVT-011 | EventReplayApi (`vvctl events replay --from --to`) | ⬜ |
| VV-EVT-012 | DomainEventDlq (`vvctl events dlq list/replay/drop`) | 🧪 in-memory |
| VV-EVT-013 | SchemaVersionRegistry (`vvctl events schema list/show/diff`) | ✅ |
| VV-EVT-014 | ComputedEventField (`expr:`) | 🟡 |
| VV-EVT-015 | NestedEventField (`object:`) | 🟡 |
| VV-EVT-016 | LookupEnrichmentField (`lookup:` cross-table) | 🟡 |

Layers S23-L1…L16 (as designed): schema parser → SchemaVersionRegistry → envelope → computed → nested → lookup → emitter → outbox → in-process channel → Kafka → Redis Streams → webhook → YAML route config + serve wiring → DLQ → replay → E2E test.

Dependencies: WalSinkBridge ✅, gRPC WriteService ✅ (needs PP-31 auth), in-process sink before Kafka, S22 registry recommended, `rdkafka = "0.36"` behind the `kafka` feature, `rmp-serde = "1"`.

**Acceptance criteria (open):**
- [ ] Inserting an order delivers `OrderPlaced` to the in-process channel.
- [ ] The `when: "values[1] > 100000"` filter suppresses small orders.
- [ ] With the broker down, events accumulate in the **WAL** outbox and deliver on recovery.
- [ ] The DLQ captures envelopes after `retry_max`; `vvctl events dlq list` shows them.
- [ ] An external gRPC consumer performs CRUD in response to a webhook (E2E).
- [ ] YAML-only routing changes.

Traceability (S22 + S23 vs existing VV-IDs), carried over: PRJ-010..013 extend ENG-023; PRJ-014..015 replace PRJ-003..008 ProjectionStore; PRJ-016 promotes the planned Redis sink (SNK-014..035) to a read cache; PRJ-017 extends ENG-020; PRJ-018/019 extend the Reducer trait; PRJ-020/021 generalise AggregateRouter; CFG-040..041 extend CFG-037; EVT-001..003 promote SNK-040 Sink; EVT-004 replaces SNK-013; EVT-005 implements the planned Redis Streams sink; EVT-008 adds an OUTBOX WAL frame; EVT-011 extends Layer 67 CDC replay; EVT-013 adds compatibility checking; EVT-014..016 are rich field types.

---

## 5. Track C — Capability phases (root plan, every concept, mapped to Rust)

Each concept keeps its root ID. "Learn / Test / Build / Prove" content stays in the annex root `IMPLEMENTATION_PLAN.md`. Its *Build/Prove* deliverables are the acceptance criteria here, translated from C# to Rust (BenchmarkDotNet → Criterion; P/Invoke → `libc` / `windows-sys`; Expression Trees → compiled closures or a bytecode VM).

### Phase 1 — Silicon Foundation (Memory & CPU)

| Concept | Rust equivalent | Status | Remaining |
|---|---|---|---|
| 1.1 Cache lines & 64-B structs (CommandFrame, WalRecord) | `hal::cache::{CachePadded, CacheAligned}`; 64-B `WireFrameHeader` (unused VVNE) | 🟡 | Make hot-path structs 64-B aligned with `size_of` assertions; the WAL frame header is 97 B of overhead today. C# version done (36 tests). |
| 1.2 False sharing | `CachePadded` exists | 🟡 | Pad ring head/tail and counters; Criterion proof (padded vs unpadded). |
| 1.3 Zero-allocation arena | `hal::arena::BumpArena` (no users) | 🟡 | Use it on the hot path; allocation-counting test (e.g. dhat) proving zero allocations per command. |
| 1.4 HugePages & TLB (2 MB / 1 GB) | `HugePageBuf` (Windows); Linux `MAP_HUGETLB` deferred 🧊 | 🟡 | Linux 2 MB/1 GB; `perf stat` dTLB proof. |
| 1.5 Thread pinning & NUMA | `hal::affinity`, `numa`, `LocalNumaArena` (Windows); `mbind` deferred | 🟡 | Pin shard writers; NUMA-local arenas; used by data path. |
| 1.6 Platform detection & CPU dispatch | `simd` OnceLock dispatch (AVX2/scalar), `SystemProfile`, `topology` | 🟡 | AVX-512 and NEON detection/kernels (enum exists); CPUID 0x1F topology (Intel handover). |
| 1.7 OS jitter eradication | `vvctl tune`, `mlock_range`, `scripts/vv-probe-linux.sh` | 🟡 | Boot-config guide (nohz_full, isolcpus, max_cstate, THP=never); mlockall at startup; Windows MMCSS; cyclictest proof. |

### Phase 2 — Single-Writer Engine

| Concept | Rust equivalent | Status | Remaining |
|---|---|---|---|
| 2.1 Single-writer principle | `ShardWriter` thread per table | 🟡 | WAL/MVCC/index/flush still run on caller threads under locks. Make one writer own the whole mutation (PP-23). |
| 2.2 Ring buffer (Disruptor) + POPCNT | SPSC `ring_buffer.rs`; `simd::popcount` | 🟡 | Remove the per-op `mpsc` + `sync_channel(1)` round-trip; pre-allocated slots; claim → publish. |
| 2.3 Wait strategies | `hal::wait::{BusySpin, Yielding, Blocking}` (no users); Sleeping missing | 🟡 | Add Sleeping; per-consumer YAML (LD-RB-01). |
| 2.4 Consumer DAGs (SequenceBarrier) | — | ⬜ | Journaler ∥ Replicator → Executor → Projector ∥ Emitter (LD-RB-02). |
| 2.5 Back-pressure (Block/Reject/DropOldest, per tenant) | only the unused `GroupCommitBatcher` | ⬜ | Live write path (LD-BACK token bucket, 429 + Retry-After; LD-RB-03). Fixes PP-11. |
| 2.6 Idempotency keys (Bloom + log, window) | — | ⬜ | LD-IDEM-01 says mandatory; 128-bit key per command; cached result. |
| 2.7 Two-phase pending/post/void/timeout | — | ⬜ | LD-2PC-01; deterministic engine clock; WAL-logged transitions; ledger domain uses it. |
| 2.8 Software prefetch + ring lookahead | `hal::prefetch` | 🟡 | Prefetch entity N+1 while processing N; perf proof. |
| 2.9 Branchless code | SIMD kernels | 🟡 | Branchless routing/validation utilities + benchmarks. |
| 2.10 Transaction context & lifecycle | TxnManager, intent log | 🟡 | Prepare → Execute → Commit/Compensate; atomic (PP-20…22). |
| 2.11 YAML command compiler | YAML schema/config; Rhai in events/semantic | ⬜ | YAML `commands:` (validations, mutations, emit) compiled to closures; zero-alloc execution. Decide with D-07. |

### Phase 3 — Durability & WAL

| Concept | Rust equivalent | Status | Remaining |
|---|---|---|---|
| 3.1 Page cache vs Direct I/O, io_uring | `storage::direct_io::DirectWriter`, `wal::UringWalAppender` (no callers; `open` truncates!) | 🟡 | Wire them behind config; fix the truncate. |
| 3.2 Sequential NVMe & group commit | `SyncCommitter` (live), `GroupCommitBatcher` (unused); adaptive window | 🟡 | `WalWriter::flush` holds the append mutex across fsync. Double-buffer so appends continue during sync; LD-BATCH 1–10,000 per fsync. |
| 3.3 Hash-chain integrity + HW CRC | SHA3-256 chain, IEEE CRC32 | 🟡 | PP-03/04/38/68; verify on open (optionally); HW CRC-32C. |
| 3.4 Deterministic recovery | replay in `open_with_key` | 🟡 | PP-01…13; replay-equivalence test (10K ops, snapshot, 10K, crash, compare); recovery speed benchmark. |
| 3.5 WAL compaction & tiered storage | WAL never truncated; `tier.rs`/`TieredWriter` unwired | ⬜/🟡 | Checkpoint-based WAL truncation; L0–L4 tiering (LD-COLD, LD-RETAIN: archive to L4 before delete). |
| 3.6 Offline WAL compiler (CSV/SQL → WAL) | `vvctl table import` (not a WAL compiler) | ⬜ | `vvctl migrate import`; 1M-row CSV → WAL → boot → verify chain. |
| 3.7 Projection dispatcher (cursor-based, bulk, multi-target) | `projection` crate (in-memory), CDC log, sinks (stand-ins), webhooks (real) | 🟡 | Durable per-target cursors (LD-KAFKA offsets); real Postgres COPY / SQL Server TVP / Mongo / Elastic targets; rebuild from WAL; lag metric. |
| 3.8 Interceptor pipeline (Continue/Drop, error policies) | partial `when:` filters in events | ⬜ | Ordered pipeline; skip/retry/dead-letter/circuit-breaker; LD-INTC-01. |

### Phase 4 — Kernel-Bypass Networking

| Concept | Rust equivalent | Status | Remaining |
|---|---|---|---|
| 4.1 TCP baseline | VV wire protocol, thread-per-connection | ✅ | Baseline latency benchmark; consider an epoll/io_uring reactor. |
| 4.2 DPDK PMD | — | ⬜🧊 | |
| 4.3 Windows RIO | — | ⬜ | |
| 4.4 Zero-copy deserialization | binary frames, but `Vec<u64>` decode | 🟡 | `bytes::Bytes` / borrowed views (locked decision: `WireFrameHeader` + zero-copy). |
| 4.5 RDMA replication | — | ⬜🧊 | |
| 4.6 eBPF/XDP pre-filter | `vectorvolt-xdp` AF_XDP skeleton; no BPF program; rings not mmap'd | 🧪 | PP-67, then a real implementation. |
| 4.7 NIC flow steering | — | ⬜🧊 | |
| 4.8 PTP / IEEE 1588 | — | ⬜🧊 | |
| 4.9 Cloud network adapters | `TransportKind` trait | 🟡 | EFA/InfiniBand/gVNIC adapters (transport chain TCP → AF_XDP → DPDK → RDMA). |

### Phase 5 — Read Fabric & Vectorization

| Concept | Rust equivalent | Status | Remaining |
|---|---|---|---|
| 5.1 CQRS read/write separation | `vectorvolt-reader` exists but serve shares one registry | 🟡 | LD-READ-01 separate reader binary; LD-READ-02 stale-by-default reads. |
| 5.2 Document mode (pre-joined) | — | ⬜ | REQUIREMENTS §5.3 `read_models: type: document`. |
| 5.3 Columnar mode | `.vvsst` SoA, dictionary/RLE/bit-packing, LZ4/Zstd | ✅ | Whole-column compression makes point reads O(column). Consider block compression. |
| 5.4 SIMD vectorized scanning | AVX2 filter/count/min-max/sum, zone overlap | ✅ | AVX-512/NEON; 1M/10M/100M benchmarks. |
| 5.5 Compiled query filter engine | DSL + SQL planner/executor (interpreted) | 🟡 | Compile predicates to closures; zero-alloc. |
| 5.6 Tiered read routing (hot/warm/cold) | — | ⬜ | Depends on 3.5. |
| 5.7 Materialized views | `views` over CDC (async) | 🟡 | Decide D-08; survive restart via replay (backfill exists). |

### Phase 6 — Control Plane & Multi-Tenancy

| Concept | Rust equivalent | Status | Remaining |
|---|---|---|---|
| 6.1 YAML topology parsing | YAML config/schema, VvConfig | 🟡 | VV-CFG-010 (§4.2). |
| 6.2 Shard Catalog & Policy Engine | `ShardCatalog` (file-based) ✅; policy engine ⬜ | 🟡 | Policies (quotas, retention, rebalancing). |
| 6.3 Consistent hashing (rendezvous) | `HashRing` (vnodes), Fibonacci partition routing | 🟡 | LD-VNODE 4096 vnodes + catalog epoch; decide rendezvous vs vnode ring. |
| 6.4 Multi-tenancy | `TenantRegistry` (unused) | ⬜ | LD-MT-01..08. |
| 6.5 Epoch fencing & failover | `EpochFence` (in-memory) | 🧪 | Wire it once a transport exists (LD-EPOCH, LD-FAILOVER < 2 s). |
| 6.6 Gateway & polyglot SDKs | TCP + HTTP + gRPC + pgwire; C# SDK and root Python SDK real | 🟡 | PP-63/64; one protocol (D-04); SDK reconnect/retry/routing. |
| 6.7 Schema registry | `schema_registry`, events `SchemaVersionRegistry` | ✅ | Compatibility rules (reject replay-unsafe changes). |
| 6.8 Auto-scaling & shard splitting | `ShardMap::split` (in-memory) | ⬜ | |
| 6.9 Live shard migration & drain | — | ⬜ | |
| 6.10 Zero-overhead observability (TSC ring, shadow exporter) | Prometheus (partial), OTel (feature-gated), `CycleTimer` unused on hot path | 🟡 | TSC telemetry ring; shadow-thread exporter; < 2 ns overhead proof. |
| 6.11 Health monitoring | SWIM structures; `DirectProber` always Suspect | 🧪 | **Cluster network transport** (UDP/TCP gossip) — prerequisite for 6.5/6.8/6.9. |
| 6.12 vvctl CLI + TUI | Extensive CLI (serve, catalog, config, connect, table, wal, http, graph, chaos, ledger, linux, xdp, cluster, cdc, events, query, tune, …) | 🟡 | PP-61/62; TUI dashboard (ratatui) ⬜; LD-CLI-02 (vvctl talks to the cluster via gRPC). |
| 6.13 Real-time web dashboard | — (root `DASHBOARD.html` is static) | ⬜ | Read-only, metrics from a separate thread (LD-DASH-01/02). |
| 6.14 QoS tiers | — | ⬜ | LD-QOS-01, LD-MT-08. |
| 6.15 Cluster-wide versionstamps | per-table `CommitClock` | ⬜ | LD-VS-01. |

### Phase 7 — Testing, Sample App & Research

| Item | Rust equivalent | Status | Remaining |
|---|---|---|---|
| 7.1 Deterministic simulation testing | `vectorvolt-chaos` DstHarness (not hooked into I/O) | 🟡 | Mock clock/disk/network traits through engine and WAL; this is how P0 bugs get found. |
| 7.2 Sample application | ledger/orders/workflow (JSON toys) | 🟡 | Rebuild on the engine (RD-1..3 from the locked decisions); billions of records. |
| 7.3 Load testing | Criterion benches, `perf_gate.rs` | 🟡 | p50/p99/p99.9 end to end; LD-GATE in CI (PP-85). |
| 7.4 Chaos testing | — | ⬜ | Kill leader, corrupt WAL, partition, memory fill, projection kill, node add/remove. |
| 7.5.1 Command gateway (anti-corruption) | HTTP gateway | ✅ | |
| 7.5.2 Assembly/static analyzer | `vv-architect scan-code` (C#/Java/TS/Go/Rust entities + FKs), `scan-db --sqlite`, sizing, events | 🟡 | Method classification (auto/mixed/gateway-only), Python, Postgres/MySQL scan-db, OpenAPI/Proto; fix manifests (PP-83). |
| 7.5.3 eBPF shadow mode | — | ⬜ | |
| 7.5.4 CDC reverse-ingestion (Debezium → WalReplicator) | — | ⬜ | LD-MIG-04 recommended default. |
| 7.5.5 End-to-end migration demo | — | ⬜ | |
| 7.6 Reference Java app testbed | — | ⬜ | LD-TEST-01 gates Phase 7 completion. |
| 7.7 Platform distribution & packaging | manifests for apt/rpm/brew/scoop/choco/winget; CI inert | 🟡 | PP-80…83; Docker image; Helm chart with HugePages/CPU pinning (LD-K8S-01). |
| 7.8 Research documentation | many vision docs | 🟡 | Benchmarks with methodology vs the latency budget; comparison vs LMAX/TigerBeetle/FDB/VoltDB. |

### Phase 8 — Research modules (optional)

| Module | Status |
|---|---|
| 8.1 Learned indexes | ⬜🔬 |
| 8.2 In-network compute (P4) | ⬜🧊🔬 |
| 8.3 Unikernel deployment (musl static binary eases this) | ⬜🔬 |
| 8.4 Wasm sandbox | ⬜🔬 (LD-MIG-05; see D-05) |
| 8.5 TTL / expiration | 🟡 engine TTL (BTreeMap, explicit `now`); branchless check and background reaper not implemented |
| 8.6 Probabilistic structures | 🟡 Cuckoo filter (PP-39); Bloom/HLL/Count-Min ⬜ |
| 8.7 Transactional watches | 🟡 CDC SSE subscribe with filters; per-key watch registry ⬜ |
| 8.8 Write-verify (read-after-write) | ⬜ |
| 8.9 Built-in load generator (`vvctl loadgen`) | 🟡 `vvctl bench` (in-process) |
| 8.10 Warm restart (shared memory) | ⬜ |
| Requirements extras: coroutine latency hiding; atomic multi-op commands | ⬜ / 🟡 (batch + CAS exist; not one WAL record) |

### REQUIREMENTS.md items with no root-plan concept (added so nothing is lost)

| Requirement | Section | Status | Notes |
|---|---|---|---|
| Composite shards (multi-entity single-shard ACID); per-command topology override | §5.8 | 🟡 | `AggregateRouter` co-locates members; override ⬜ |
| Cross-shard sagas (reserve → confirm → commit, idempotent steps) | §5.9, §10 | ⬜ | Today only the cross-table intent log (PP-22) |
| Post-commit orchestration / DLL sandbox, isolated process | §5.1, §5.11 | ⬜ | `vectorvolt-workflow` is an unrelated toy |
| Online migration backfill (`migration: strategy: backfill`) | §5.5 | ⬜ | Pairs with 7.5.4 |
| Engine-generated IDs before WAL append (LD-ID-01) | §5.2 | ⬜ | Clients supply pks today |
| WAL segments 256 MB configurable; fsync policies sync/group/async | §5.6 | 🟡 | Durability Fsync/WriteOnly/Buffered exist |
| Snapshot scheduling (time/count/adaptive), compressed + encrypted | §5.7 | ⬜ | Plus PP-01, PP-56 |
| Replication factor, active-passive/active-active per shard, self-healing | §6.3 | 🧪 | Needs cluster transport (6.11) |
| Structured JSON logging with correlation IDs (cold path); rolling upgrades with format versioning | §6.6 | 🟡 | `tracing` + `--log-format`; upgrade/versioning plan ⬜ (see `docs/MULTI_CLUSTER_ROLLOUT_STRATEGY.md`) |
| External profiling (eBPF on Linux, ETW on Windows); admin ops pause/resume shard, force snapshot, rebuild projections, drain queues | §6.6 | ⬜ | |
| K8s: CPU Manager static, HugePages, HostNetwork, SR-IOV, Multus, Topology Manager; CRDs/operator; K8s never orders txns | §6.9, LD-K8S-01/02 | ⬜ | Fold into PP-83 / 7.7 |
| PTP fallback to wall clock; determinism guard (no wall clock/random on hot path) | §6.10, §12, LD-REPLAY | 🟡 | WAL uses `SystemTime`; no `DeterminismGuard` |
| Cloud adapter matrix (AWS EFA, Azure IB, GCP gVNIC, Oracle, IBM) | §6.8 | ⬜ | 4.9 |
| Testing pyramid 70/20/10, deterministic replay tests, DST | §12 | 🟡 | Many tests; few replay-equivalence tests |
| Hardware tech table (CRC32/AES-NI, POPCNT, RDTSC, mlockall, MMCSS, CXL-ready arena, P4-ready seq, UINTR-ready transport) | §7 | 🟡 | See Phase 1/4; Intel silicon handover (`intel/INTEL_SILICON_RESEARCH_HANDOVER.md`: DSA, MKTME, CPUID 0x1F, SiliconManifest probe-once pattern) |
| Out of scope permanently (§16): global secondary indexes, analytics in engine, GPU, PMem, SIMD JSON, custom TCP stack, TSX | §16 | — | Raft entry under review (D-02); SQL/analytics surface under review (D-03) |

### Locked architecture decisions (Rust plan, carried over)

- Single operator CLI `vvctl`.
- Three server binaries: `vectorvolt-server` :50051, `vectorvolt-reader` :5432, `vectorvolt-control` :8443 (today only `vvctl serve` runs; LD-RUNTIME-01).
- Build targets: `x86_64/aarch64-unknown-linux-musl`, `x86_64-pc-windows-msvc`; no glibc.
- YAML-primary config (precedence: generated < file < env < CLI).
- `WireFrameHeader` `#[repr(C, align(64))]`, CRC-32C per frame, `bytes::Bytes` zero-copy (**not yet true**: PP-68, 4.4).
- Transport chain TCP → AF_XDP → DPDK → RDMA.
- ShardCatalog self-hosted (no ZooKeeper or etcd).

Plus all LDs in `docs/vision/VECTORVOLT_LOCKED_DECISIONS.md` (LD-ACID, SNAP, REPLAY, CHAOS, COLD, PITR, RETAIN, SWIM, VNODE, EPOCH, FAILOVER, QUORUM, GC, ARENA, BATCH, BACK, READ-01/02, RUNTIME-01, PROJ, KAFKA, DLQ, SDK-01, PG-WIRE, MT-01..08, OBS, GATE, ADAPT) and the REQUIREMENTS LDs (§11, §15), subject to §2.

---

## 5A. Track VA — vv-architect (migration analyzer + deployment blueprint)

*Audited 2026-10-31. The crate has 1,981 LOC and 52 unit tests, no CLI or integration tests. It landed in PRs #639–#644. Those commits cite a "VV-ARCHITECT section" of the plan that no longer exists in any tracked doc, so this section restores the spec (VA-25).*

**Purpose.** vv-architect implements Concept 7.5.2 (assembly/static analyzer, REQUIREMENTS §15.4) and the `vvctl migrate analyze` surface (§5.16.7). It also produces a sized deployment blueprint and manifests (§5.25, §6.9, LD-K8S-01, PP-83).

**What exists today:**
- `scan-code <dir>`: tree-sitter entity and FK scan for C#, Java, TS, Go and Rust.
- `scan-db --sqlite`: reads FKs only.
- An aggregate-root / reference classifier.
- An event-name suggester (a list of 21 verbs).
- Sizing rules driven by `--tps/--data-gb/--repl-factor`.
- `vectorvolt-blueprint.toml`.
- Terraform (AWS) output and K8s StatefulSet output.

**Status by capability:**

| Capability | Status |
|---|---|
| AST entity scan (C#/Java/TS/Go/Rust) | 🟡 classes/structs only; no records/interfaces; every class counts as an entity |
| FK detection | 🟡 name-suffix only; navigation properties and JPA/EF attributes ignored |
| Root/reference classification | 🟡 buggy: unknown FK targets (CorrelationId, TenantId) disqualify roots; lookup tables reported as roots |
| SQLite scan | 🟡 FKs only; opens read-write and creates an empty DB on a mistyped path |
| Postgres/MySQL/SQL Server scan (`--connection`) | ⬜ deferred in d995f54 |
| Python, OpenAPI/Proto scanners | ⬜ deferred (b3ae96f, d3c5ee7) |
| Go methods → events | ⬜ deferred (038a0c2) |
| Method classification auto / mixed / gateway-only | ⬜ |
| .NET DLL/IL analysis (`--dll`) | ⬜ |
| YAML mutation / stub generation | ⬜ |
| Blueprint consumable by vvctl | ⬜ TOML is ignored silently by `--config` and rejected by `--master-config` |
| S22 `partitioning:` / S23 `domain_events:` output | ⬜ only bare event names |
| Sizing model | 🟡 replication math inconsistent; no read/write ratio; overflow panics |
| Terraform | 🟡 placeholder AMI; no VPC/SG/user_data; gp3 root volume although it says NVMe |
| K8s manifest | 🧪 no container command; only port 5995; no HugePages, probes, Raft seeds or image |
| Dockerfile / Helm chart | ⬜ |
| `vvctl migrate analyze` front end | ⬜ |

**Backlog (suggested order: VA-01, 02, 03, 07, 09, 10, 08, 04, 05, 14; then 06, 11–13; then 19–23, 18):**

| ID | Item | Acceptance | Size |
|---|---|---|---|
| VA-01 | Fix root/reference classification | Only known entities count as references; referenced leaf entities become reference data; transitive aggregates; tests for CorrelationId, Customer→Order→OrderItem, Product as a lookup | M |
| VA-02 | SQLite read-only + hardening | A missing path errors out; FK matching is case-insensitive; quotes are escaped; virtual/FTS tables are excluded | S |
| VA-03 | CLI correctness | Clap validates manifest flags before any write; finite positive sizing inputs; checked math; `assert_cmd` tests | S |
| VA-04 | Emit consumable `vectorvolt.yaml` | `VvConfig::from_file` + `validate()` accept it with zero warnings; round-trip test | M |
| VA-05 | Emit S22/S23 YAML | Output deserializes into `PartitionConfig` / `YamlEventRouteConfig`; events carry trigger and version | M |
| VA-06 | Engine-side consumers | `serve` applies `tables.*` aggregate config and loads partitioning/events YAML; `EngineConfig` rejects or warns on unknown fields | L |
| VA-07 | Bind-host flags | gRPC host ✅ (`--grpc-host`, 2026-10-31); metrics host ✅ (`--metrics-host`); pgwire host still hardcoded | S |
| VA-08 | Dockerfile + image (PP-82) | `docker run` serves 5995 and `/metrics` | M |
| VA-09 | K8s manifest fix (PP-83) | Real `vvctl serve` command; every port named; probes; passes kubeconform | M |
| VA-10 | HugePages + CPU pinning | `hugepages-2Mi`, hugetlbfs mount, integer CPU with requests == limits; static CPU Manager / Topology Manager documented | S |
| VA-11 | Raft/cluster wiring | Seeds from headless DNS, node id from pod name, cluster ports; gated on the 6.11 transport | M |
| VA-12 | Helm chart | `hugepages.enabled/size`, `cpu.cores`, `memory.size` values; `helm lint`/`template` in CI | M |
| VA-13 | Terraform realism | AMI variable, VPC/SG, user_data, i4i/r-family types, NVMe data volume; `terraform validate` passes | M |
| VA-14 | Sizing model v2 | Replication-aware; read/write ratio; writes to primaries only; golden tests | M |
| VA-15 | Scanner hardening | Default excludes plus `--exclude`; TSX grammar; Java multi-declarators; records/structs; namespace-qualified keys; cross-file Rust impls; warnings surfaced | M |
| VA-16 | Type-based relationships | Navigation properties, collections, `@ManyToOne`, `[ForeignKey]`, `DbSet<T>`; entity filter | L |
| VA-17 | Go methods + better event verbs | Go receiver methods; `VerbNoun`, `Async` suffix and snake_case handled | S |
| VA-18 | Postgres/MySQL `scan-db --connection` | Reads `information_schema` FKs/PKs/columns; tested against a CI service DB | L |
| VA-19 | Columns/PK in blueprint | Per-table YAML schemas accepted by `yaml_schema::parse_yaml_schema` | M |
| VA-20 | Method classification | auto / mixed / gateway-only from AST call sites, with line numbers and a % summary (7.5.2, §15.4) | L |
| VA-21 | .NET DLL analysis (`--dll`) | IL reader; 50-class sample assembly checked against known labels | L |
| VA-22 | YAML mutation / stub generation | Golden-file tests | L |
| VA-23 | `vvctl migrate analyze --dll/--connection` | Delegates to vv-architect (§5.16.7) | S |
| VA-24 | Python + OpenAPI/Proto scanners | Entities and FKs, with tests | M |
| VA-25 | Restore the spec | This section; update the stale module docs (`main.rs:17-21`, `blueprint.rs:7-9`) | S |

---

## 6. Crate map (35 crates, audited)

| Crate | Status | Notes |
|---|---|---|
| vectorvolt-hal | 🟡 | SIMD wired; other primitives unused |
| vectorvolt-wal | 🟡 | P0 recovery defects |
| vectorvolt-txn | 🟡 | P0/P1 defects |
| vectorvolt-engine | 🟡 | core; P0/P1 defects |
| vectorvolt-storage | ✅ | `.vvsst` v3, manifest, compaction; tier/direct-io/prefetch unwired |
| vectorvolt-index | ✅ | Cuckoo map/filter, Roaring, zone maps (PP-39/40) |
| vectorvolt-query | 🟡 | DSL + SQL (34.9K-line executor, PP-70) |
| vectorvolt-crypto | 🟡 | AES-GCM; KeyStore weakness (PP-55) |
| vectorvolt-store | 🟡 | KV LSM used by EKOS; PP-14 |
| vectorvolt-ledger / orders / workflow | 🧪 | JSON toys (PP-15) |
| vectorvolt-tpm | 🟡 | real behind the `tpm` feature only |
| vectorvolt-net | ✅ | wire protocol, auth, TLS; PP-20/30 |
| vectorvolt-http | ✅ | ~70 routes; PP-20/34 |
| vectorvolt-server | 🟡 | gRPC used; `VvServer` (VVNE) unused |
| vectorvolt-reader | 🟡 | minimal pgwire (PP-35) |
| vectorvolt-control | 🧪 | fake endpoints; RBAC unenforced |
| vectorvolt-cluster | 🧪 | data structures + simulation; no transport |
| vectorvolt-replication | 🧪 | in-process channel; unused |
| vectorvolt-sink / sink-sdk | 🧪 | stand-ins; bridge re-reads whole WAL |
| vectorvolt-events | 🟡 | see §4.4 |
| vectorvolt-projection | 🟡 | in-memory |
| vectorvolt-semantic | 🟡 | unwired (§4.3) |
| vectorvolt-graph | ✅ | small, real |
| vectorvolt-vector | 🧪 | brute-force "HNSW" |
| vectorvolt-otel / metrics | 🟡 | PP-69, PP-32 |
| vectorvolt-pam | 🟡 | PP-33; not used by the DB |
| vectorvolt-xdp | 🧪 | skeleton |
| vectorvolt-chaos | 🟡 | not hooked into I/O |
| vectorvolt-sdk | 🟡 | PP-64 |
| vvctl | ✅/🧪 | real core; stubs PP-61/62/65 |
| vv-architect | 🟡 | PP-83 |

Outside `vectorvolt/`: `ekos/` (EKOS kernel + ekosd, depends on `vectorvolt-store`, dormant since July); root `sdk/csharp` and `sdk/python` (real clients); frozen C# prototype `src/`, `tests/`.

---

## 7. Recommended order of work

1. **Now:** PP-41 (revoke the PAT).
2. **Sprint P-A (durability):** PP-01 → PP-02/03/04 → PP-05/06/07 → PP-08/09/10 → PP-11/12/13. Build the DST harness wiring from 7.1 alongside, so these are proven under fault injection.
3. **Sprint P-B (atomicity):** PP-20/21 → PP-22 → PP-23 (true single-writer, Concept 2.1/2.2) → PP-24 (MVCC Phase 1b; format bump, so bundle it with PP-53 and the CRC-32C switch PP-68) → PP-25.
4. **Sprint P-C (security):** PP-30…PP-40.
5. **Sprint P-D (encryption item #1):** PP-51/52 → PP-53/54 (single format bump with PP-24/68) → PP-55/56.
6. **Sprint P-E (honesty + CI):** PP-60…70, PP-80…85, plus §9 doc reconciliation.
7. Then return to features in this order: VV-CFG-010 → finish wiring S22/S23 → Phase 2 gaps (2.4 DAG, 2.5 back-pressure, 2.6 idempotency, 2.7 two-phase) → cluster network transport (6.11, prerequisite for 6.5/6.8/6.9) → Phase 3.5/3.7 tiering and real projections → Phase 6 tenancy/QoS → Phase 7 proof → Phase 8 research.

Decisions D-01…D-12 should be taken before Sprint P-B: D-02 and D-07 affect the commit/WAL design.

---

## 8. Open questions for the owner

1. What were path-to-production items #2…#N? (Not in the repo.)
2. Decisions D-01…D-12 (§2).
3. Which hardware tier is the committed SLO target (dev laptop vs EC2 g4dn.4xlarge vs Graviton)?
4. Is the 34.9K-line SQL surface (S34–S380) a product commitment, or should unbacked statements be marked unsupported (PP-70)?
5. Should EKOS (`ekos/`) development resume, and should it stay pinned to the `vectorvolt-store` path dependency?

---

## 9. Documentation reconciliation

| Doc | Action |
|---|---|
| `REQUIREMENTS.md` | Bump to v4.0 after §2 decisions (Rust stack, ratified contradictions). |
| `IMPLEMENTATION_PLAN.md` (root) | Keep as the **learning syllabus annex** (Learn/Test/Build/Prove text); add a header pointing here. |
| `vectorvolt/IMPLEMENTATION_PLAN.md` | Keep as the **S3 layer log + S22/S23/VV-CFG-010 design annex**; add a header pointing here; fix the "L121 NEXT" line. |
| `vectorvolt/README.md` | Add a "known limitations" section (single-node only, non-atomic txns until PP-20, SDK status). |
| `vectorvolt/CHANGELOG.md` | Stops at Layer 68; add S3–S380, MVCC, Gaps, encryption. |
| `docs/MASTER_IMPLEMENTATION_PLAN.md`, `docs/status/*`, `docs/VECTORVOLT_FEATURE_MATRIX.md`, `docs/vectorvolt-tasks/*` | Mark as historical snapshots (2026-07), or regenerate from code. |
| `docs/traceability/catalog.yaml` | Covers only early WAL/store work; its IDs clash with feature-matrix IDs (e.g. VV-WAL-012). Re-key it to the VV-* feature IDs and add PP-* items. |
| Vision docs | Fix internal contradictions (partition size 100K vs 1M; RowDescriptor 12/20/32 vs 16/16/32; tier naming L0–L4 vs Tier 1–3; "no cost-based optimizer" vs `planner.rs` cost-based; phase numbers) when each area is next touched. |

---

## 10. Gap register (code audit 2026-10-31)

Everything still open for VectorVolt to be complete, from the plan above plus a code sweep (stubs, swallowed errors, panics, state-only features). Each row cross-references its existing ID where one exists, so §3–§5A stay the design detail and this table is the **execution tracker**. Batches run in order A → N; each batch lands as one PR and its rows flip to ✅ here.

Priority: **P0** data loss / security · **P1** wrong results / crash · **P2** missing or fake feature · **P3** cleanup, docs, CI. Size: S = hours, M = days, L = week+. Status: ⬜ open · 🟡 partial · ✅ done · ⏸ needs owner decision.

Out of scope by owner decision: PP-33 (PAM credentials), PP-41 (ConsoleApp1).

### A — P0 data loss and security

| # | Gap | Where | Ref | Size | Status |
|---|---|---|---|---|---|
| A1 | Row-level security policies stored, never applied | `sql/executor.rs:1695` | S78 | M | ✅ SELECT, JOIN right side, UPDATE, DELETE all filtered; unsupported policy expressions rejected `[0A000]`. No user/role model yet, so policies apply to every session. |
| A2 | "Column encryption" reverses the string | `executor.rs:3505-3535` | S142 | M | ✅ Real AES-256-GCM bound to table.col.pk; servers now set a key derived from the master key (C1). |
| A3 | pgwire has no authentication (always AuthenticationOk) | `reader/session.rs:54` | PP-35 | M | ✅ Cleartext password = `--secret`, constant-time check, FATAL 28P01; malformed startup → 08P01; SSLRequest answered `N`. Cleartext is loopback-only until D1 TLS. |
| A4 | SQL security state is per connection; reconnect clears restrictions | `tcp_server.rs:94,125`, `reader/session.rs:40` | — | L | ✅ One shared executor state per server (`new_session()`); txn/cursors/prepared stay per connection. Any client can still change shared settings via SQL until a privilege model exists. |
| A5 | Table tokens / rate limits only checked on SELECT | `executor.rs:12053,12062` | S81/S82 | S | ✅ `check_table_access()` on SELECT, INSERT, INSERT…SELECT, UPDATE, DELETE, TRUNCATE, COPY FROM. |
| A6 | Wire protocol allocates client-declared length (≤ 4 GiB) before the limit check | `net/connection.rs:75`, `server/lib.rs:130` | — | S | ✅ VVNE header decoded (size-checked) before the payload buffer is allocated. |
| A7 | DDL WAL append/flush errors ignored | `engine/registry.rs:376` | — | S | ✅ `log_ddl` returns errors; `drop_table` keeps the table registered if `schema.txt` can't be removed. |
| A8 | Undecryptable WAL frames silently skipped on replay | `engine.rs:539` | PP-52 | M | ✅ Encrypted frame must decrypt; wrong key or missing key → `WalDecrypt`; plaintext (markers, pre-encryption) frames still replay. |
| A9 | ledger/orders/workflow rewrite JSON without fsync/atomic rename | `ledger.rs:55`, `workflow/engine.rs:40` | PP-15 | M | ✅ tmp + fsync + rename + dir fsync in ledger/orders/workflow (and ledger accounts). Open: `Ledger::post` writes two files non-atomically. |
| A10 | `vectorvolt-store` put never flushes its WAL buffer | `store.rs` | PP-14 | S | ✅ put/delete flush (fsync per config); sealed WAL segments truncated after SST flush (open segment kept). |
| A11 | Store compaction empties the manifest on mid-way failure | `store.rs:148-164` | — | S | ✅ SST written as `.tmp` → rename + dir fsync; manifest swapped only after success; leftover `.tmp` cleaned on open. |
| A12 | WAL hash chain unkeyed (tail edits / truncation undetected) | wal | PP-38 | M | ✅ Keyed tail seal (`wal/seal.rs`) written after each synced commit when encryption is on; truncation/rewrite/forged seal fail the open. Batch C: a deleted `tail.seal` is now rejected once a table has been sealed. Residual: forging the table-marker frame or rolling back to exactly the marker still needs a keyed frame chain. |
| A13 | HTTP/control tokens compared with `==`; tenant tokens plaintext | `http/server.rs:125`, `control/server.rs:60` | PP-34 | S | ✅ Constant-time bearer compare (HTTP, control); tenant tokens stored as SHA-256, never serialized. |
| A14 | RBAC store never enforced by any listener | control + listeners | PP-37 | L | 🟡 Control plane enforces RBAC per route (users + roles in config; 401/403). HTTP/gRPC/pgwire still single-credential → per-user roles land with B30. |

### B — P1 correctness and crashes

| # | Gap | Where | Ref | Size | Status |
|---|---|---|---|---|---|
| B1 | No per-pk write ordering | `txn/manager.rs` | PP-23 | L | ✅ 64 per-pk stripe locks held from WAL append through apply; commit_ts = WAL sequence. Same-stripe writers serialise through fsync. |
| B2 | MVCC history wrong after restart (one version per row on disk) | txn/storage | PP-24 | L | ✅ Version history rebuilt from the WAL on open (no format bump); valid while the WAL is kept in full — G10 must persist versions before truncating. |
| B3 | PITR restore is not point-in-time | `pitr.rs` | PP-12b | M | ✅ `restore_to_lsn` / `_marker` / `_timestamp` into a fresh dir via WAL fold up to X. Open: retention deletes aren't WAL-logged, so restore after retention brings those rows back. |
| B4 | PITR ignores its flush result | `pitr.rs:25` | — | S | ✅ Flush error returned; no marker after a failed flush. |
| B5 | CHECK constraints never checked | `executor.rs:14226` | S59 | M | ✅ CHECK evaluated on INSERT/upsert/UPDATE/INSERT…SELECT (23514); unenforceable CHECKs rejected at DDL. |
| B6 | FOREIGN KEY constraints never checked | `executor.rs:14233` | S60 | M | 🟡 Child-side FK check (23503) and parent RESTRICT on DELETE/UPDATE. ON DELETE CASCADE / SET NULL not parsed yet. |
| B7 | Triggers never fire | `executor.rs:15368` | S66 | M | 🟡 BEFORE/AFTER × INSERT/UPDATE/DELETE fire per row (SQL-body procs), depth cap 16. Not yet: modifying NEW; firing for INSERT…SELECT/COPY. |
| B8 | Statement timeout checked after the query finishes | `executor.rs:1542` | S106 | M | ✅ Deadline checked inside scan/filter/join/aggregate/update/delete/insert loops → 57014. |
| B9 | ROLLBACK TO SAVEPOINT misses tables opened before the savepoint | `executor.rs:15562` | S53 | M | ✅ SAVEPOINT snapshots every table's staged writes; ROLLBACK TO restores them; 3B001 / 25P01 errors. |
| B10 | Unsupported view/join predicates evaluate to true | `executor.rs:14350` | — | S | ✅ `eval_str_where` returns errors (0A000) for unsupported forms; adds IN, BETWEEN, LIKE/ILIKE, qualified names. |
| B11 | Autocommit SQL writes take no commit lock | executor | PP-25 | S | ✅ Autocommit writes take the (re-entrant) SQL commit lock; reads don't. |
| B12 | pgwire panics on a message length < 4 or negative | `reader/wire.rs:73-80` | — | S | ✅ Message length must be 4…1 MiB; fuzz test (20k hostile inputs) never panics. |
| B13 | pgwire assumes one read per message (no reassembly, > 64 KB breaks) | `reader/session.rs:68` | PP-35 | M | ✅ Buffered framing: pipelined, split and large (≤ 1 MiB) messages handled in order; bad framing → FATAL 08P01. |
| B14 | SQL TCP server: no auth, unbounded `read_line` | `tcp_server.rs` | — | S | 🟡 1 MiB line cap; optional `AUTH <secret>` first line. No server path constructs `SqlTcpServer` with a secret yet. |
| B15 | NaN in vector search panics | `vector/hnsw.rs:81` | — | S | ✅ `try_insert`/`try_search` reject NaN/inf; `total_cmp`; engine `insert_vector`/`search_vector` now return `InvalidVector`. |
| B16 | 1,084 `Mutex.lock().unwrap()` in the executor (poisoning) | `executor.rs` | — | M | ✅ `lock_safe()` recovers poisoned mutexes across all ~1,331 call sites. |
| B17 | CDC log append errors ignored | `engine.rs:763,794,841,1669,1688` | — | S | 🟡 CDC append failures logged, counted and exposed as a gap range (`cdc_health()`); partial records repaired. Not yet: rebuilding the CDC log from the WAL. |
| B18 | Index-rebuild insert errors ignored | `engine.rs:1337` | — | S | ✅ Index rebuild returns insert errors. |
| B19 | Replication leader assigns seq after the write (reordering) | `replication/leader.rs:47-75` | — | S | ✅ Leader holds the sender lock across write, seq assignment and send. |
| B20 | Network FK check trusts the Cuckoo filter; filter never grows | `TableEngine::fk_exists` | PP-39 | M | ✅ Filter positives confirmed against the pk index; filter rebuilt at 2× live count; deleted keys dropped at rebuild. |
| B21 | pk `u64::MAX` collides with the empty-slot sentinel | index | PP-40 | S | ✅ `u64::MAX` stored in a dedicated slot; works as a normal key. |
| B22 | Secondary scan doesn't re-check values; `pk as u32` truncation | query | VV-QRY-003.5 | M | ✅ Candidates re-checked against real values; u64 pks; batched fetch. |
| B23 | Deletes don't remove rows from flushed `.vvsst` | `txn/manager.rs:551` | — | M | ✅ Deletes survive flush, compaction and restart; compaction install no longer undoes racing writes. |
| B24 | Compaction holds the write lock for the whole run | storage | — | M | ✅ Merge built without partition locks (`prepare_compaction`); locks held only for `install_compaction`. |
| B25 | `TxnManager::get` reopens/CRC-checks per call; SQL UPDATE/DELETE O(N) | txn/query | VV-QRY-001.4 | M | ✅ Cached, CRC-checked partition readers with epoch invalidation: 109.8 µs → 12.4 µs per flushed read (8.9×). |
| B26 | `WalWriter::flush` holds the append lock across fsync | wal | 3.2 | M | ✅ fsync runs outside the append lock on a cloned handle; `flush_covering()` reports only synced tips. |
| B27 | io_uring WAL appender truncates on open | `wal/uring_writer.rs` | 3.1 | S | ✅ io_uring appender opens without truncating and appends at the end. |
| B28 | Webhook sink always returns Ok | events | PP-66 | S | ✅ Webhook `send` returns the real delivery result after retries; `send_fire_and_forget` kept separately. |
| B29 | CDC sink bridge re-reads the whole WAL per poll | sink/bridge | — | M | ✅ Bridge keeps a cursor; new polls read only new frames, across segment rotation. |
| B30 | `--secret` key scope blocks `--multi` with auth | `serve.rs`, `net/server.rs` | PP-30 | S | ✅ `--key-scope` (Admin default with `--multi`) and `--key-file` (`key_id scope hex` lines). |
| B31 | OTel panics on a bad endpoint; 4/7 metrics never populated | otel | PP-69 | S | 🟡 Bad endpoint returns an error/no-op instead of panicking. Not yet: the 4 unpopulated metrics (live in vectorvolt-metrics/serve); `--features otlp` doesn't compile against opentelemetry 0.26. |

### C — Encryption at rest (path-to-production item #1)

| # | Gap | Ref | Size | Status |
|---|---|---|---|---|
| C1 | `--encrypt-at-rest` with `--multi` / `TableRegistry` | PP-51 | M | ✅ `TableRegistry::open_with_encryption`; `--multi --encrypt-at-rest` works; SQL column key derived from the master key and set on the pgwire executor (closes A2's remainder). |
| C2 | Key ID + flag in segment header; fail loudly on mismatch; AAD = (segment, seq) | PP-52 | M | ✅ Segment header flag + key id (v2 header); AAD = key id + seq (VVE2); wrong/missing key fails naming both ids; VVE1 legacy still opens. |
| C3 | `.vvsst` encryption (format bump) | PP-53 | L | ✅ Encrypted `.vvsst` v4 (whole-file VVE2, AAD = identity + key id), decrypted once at open into zeroed memory; `VVMANIF2` encrypted manifest; mixed plaintext/encrypted compaction; warm reads unchanged (214 vs 282 ns). |
| C4 | Encrypt CDC log, manifest, intent log, PITR markers, sidecars | PP-54 | M | ✅ Per-record sealed cdc.log and intent log, sealed `--multi` schema.txt and index catalog, encrypted catalog WAL, MAC'd PITR markers; files bound to a WAL-recorded table id. Residual: plaintext legacy prefixes are accepted unauthenticated (migration). |
| C5 | KeyStore: no heap copy, mlock, rotation, nonce accounting | PP-55 | M | ✅ Single protected key copy (memfd_secret / mlock / VirtualLock), zeroized; key_id(); nonce use counter + needs_rotation(); KeyRing rotation (re-encrypt on compaction still open). |
| C6 | Encrypted + compressed snapshots | PP-56 | M | ✅ `snapshot.vvss` sealed; unauthenticated markers rejected on keyed tables. Partitions already column-compressed (no second layer). |

### D — Network and security hardening

| # | Gap | Ref | Size | Status |
|---|---|---|---|---|
| D1 | TLS for HTTP, gRPC, pgwire; optional mTLS | PP-36 | L | ✅ `--tls-cert`/`--tls-key` apply to the wire, HTTP, gRPC and pgwire listeners; `--tls-client-ca` requires client certs (mTLS); pgwire SSLRequest answers `S`/`N`, `--pgwire-require-tls` refuses plaintext; non-loopback plaintext binds warn. |
| D2 | pgwire SCRAM/MD5, SSLRequest, extended protocol | PP-35 | L | ✅ SCRAM-SHA-256 default (`--pgwire-auth scram\|md5\|password`), verifier-only storage, constant-time proofs; extended protocol (Parse/Bind/Describe/Execute/Close/Sync/Flush, row limits, error-until-Sync). Residual: no SCRAM-PLUS; text params containing `'` refused (lexer has no quote escape); Describe probes by executing; all columns typed text. |
| D3 | `--pgwire-host` flag (hardcoded 127.0.0.1) | VA-07 | S | ✅ `--pgwire-host` (default 127.0.0.1). |
| D4 | IP allowlist (S83) never called; `max_connections` (S116) not enforced | — | S | ✅ pgwire admission rejects disallowed IPs (28000) and connections over the shared `max_connections` / `--pgwire-max-connections` (53300); native listener enforces its own limit. |
| D5 | Masking, grants, freeze, schema lock, idle timeout, memory limit never enforced | S159/252/273/278/283/295 | L | ✅ Result masks applied, grants enforced on tables that ever had a GRANT (pgwire sets the authenticated role), frozen tables reject writes, schema-locked tables reject DDL, idle sessions expire, memory limit aborts with 53200 (estimated row bytes). |

### E — Stand-ins presented as real

| # | Gap | Ref | Size | Status |
|---|---|---|---|---|
| E1 | ~300 of 353 executor settings are state-only; implement or return unsupported | PP-70 | L | ✅ 19 SET forms with real effect kept; 14 PG client-compat settings accepted as no-ops; ~78 state-only settings removed and now return 0A000 naming the setting (tests `e1_settings.rs`). Residual: SHOW/RESET of removed settings still return defaults. |
| E2 | Split the 34,955-line `executor.rs` | PP-70 | L | ✅ `executor.rs` split into `sql/executor/` (13 files, largest ~3k lines; tests in 4 parts); pure move, public paths unchanged. |
| E3 | Control API returns hardcoded data | PP-60 | M | ✅ `ControlBackend` over a real `TableRegistry` (`vvctl control start --dir`); health/tables/stats/snapshot are real, cluster endpoints return 501 "not available". Residual: CLI default token `vvctl-token` still hardcoded. |
| E4 | vvctl ping/status/disconnect/bench-net/server/projection only print | PP-61 | M | ✅ ping/status/server status/bench-net do real wire round trips with latency; disconnect, server stop and projection fail with "not implemented" (no opcode). `vvctl sql -e` now exits non-zero on error. |
| E5 | Orphan vvctl modules | PP-62 | S | ✅ control/sql/vector/reader wired into `vvctl`; print-only `connect_simple.rs` deleted. |
| E6 | `table_*_stub.rs` tests test nothing they're named for | PP-65 | S | ✅ 44 meaningless `table_*_stub.rs` deleted; chaos/ledger kept as `chaos_ledger_commands.rs`; real `vector_command`/`sql_command` tests. |
| E7 | "HNSW" is brute force | PP-66 | L | ✅ `vectorvolt-vector` HNSW is a real multi-layer graph (recall@10 ≥ 0.9, visits < N/3); SQL HNSW pruning bug fixed (recall 0.014 → passing). Residual: SQL HNSW ef fixed at max(k,100). |
| E8 | Kafka sink is mpsc; Postgres sink collects strings; flush no-op | PP-66 | L | ✅ `KafkaSink` (rskafka, flush awaits acks) and `PostgresSink` (tokio-postgres, parameterized upserts, one txn per batch, flush awaits COMMIT); old mpsc kept as `ChannelKafkaSink` test double. Residual: PG NoTls only; Kafka single partition, no retries; `vvctl serve` does not build sinks from config yet. |
| E9 | Redis Streams sink / Redis cache have no client | PP-66, VV-PRJ-016 | M | ✅ `RedisStreamSink` (XADD MAXLEN ~, reconnect) and `RedisReadCache` (GET/SET EX/DEL) on the `redis` crate; live tests `#[ignore]` behind VV_TEST_REDIS. |
| E10 | Outbox and DLQ in memory, not in WAL | VV-EVT-008/012 | M | ✅ `EventOutbox::open` / `EventDlq::open`: fsynced journal replayed and compacted on open, torn tail skipped; restart tests. Residual: callers still use in-memory `new()`. |
| E11 | XDP: no eBPF program, TX frame 0, busy-spin | PP-67, 4.6 | L | ✅ real mmap'd rings, TX frame allocator with completion reclaim, poll(2) + NEED_WAKEUP instead of busy-spin, hand-built XSKMAP redirect program loaded via bpf(2) (clear error + TCP fallback on failure). Residual: lint-checked only for Linux, never run on a kernel; needs 5.7+, single queue. |
| E12 | Python/Go/TS SDKs speak a protocol no server implements | PP-63 | L | ✅ Python/Go/TS SDKs rewritten to the real 24-byte `protocol.rs` framing (persistent connection, auth, table/row ops, txns); golden-byte tests. No SQL opcode exists, so query/execute raise not-supported. Breaking API change (rows are u64 lists). |
| E13 | Rust SDK connection per call breaks transactions | PP-64 | M | ✅ SDK keeps one persistent connection; transactions span calls; dropped only on transport errors. |
| E14 | CRC-32C comments over IEEE CRC-32 | PP-68 | S | ✅ net and WAL frame comments now say IEEE CRC-32 (no on-disk change); HAL's real CRC-32C left as is. |

### F — Cluster, Raft, replication (D-02: Raft required)

| # | Gap | Ref | Size | Status |
|---|---|---|---|---|
| F1 | Cluster network transport (gossip + Raft RPC) | 6.11 | L | ⬜ |
| F2 | Real Raft: persisted term/vote/log, network replication, snapshot install | LD-SWIM | L | ⬜ |
| F3 | SWIM probe always Suspect | 6.11 | M | ⬜ |
| F4 | `--cluster-seeds` recorded only; `cluster join/leave/init` local only | — | M | ⬜ |
| F5 | Replication: network, catch-up, acks/sync, replication factor | §6.3 | L | ⬜ |
| F6 | Epoch fencing + < 2 s failover | 6.5 | L | ⬜ |
| F7 | Shard split, auto-scale, live migration, drain | 6.8/6.9 | L | ⬜ |
| F8 | Cross-shard sagas; cluster versionstamps | §5.9, 6.15 | L | ⬜ |

### G — Engine core (Phase 2–3)

| # | Gap | Ref | Size | Status |
|---|---|---|---|---|
| G1 | True single writer owns the whole mutation | 2.1 | L | ⬜ |
| G2 | Ring buffer claim/publish, no per-op mpsc round-trip | 2.2 | M | ⬜ |
| G3 | Sleeping wait strategy; per-consumer config | 2.3 | S | ⬜ |
| G4 | Consumer DAG | 2.4 | L | ⬜ |
| G5 | Back-pressure (block/reject/drop-oldest, per tenant, 429) | 2.5 | M | ⬜ |
| G6 | Idempotency keys | 2.6 | M | ⬜ |
| G7 | Two-phase pending/post/void/timeout | 2.7 | L | ⬜ |
| G8 | YAML command compiler | 2.11, D-07 | L | ⏸ |
| G9 | Engine-generated IDs | LD-ID-01 | M | ⬜ |
| G10 | Physical WAL truncation after snapshot | 3.5 | M | ⬜ |
| G11 | Tiering L0–L4, archive before delete | 3.5 | L | ⬜ |
| G12 | Direct I/O + io_uring behind config | 3.1 | M | ⬜ |
| G13 | Offline WAL compiler (`vvctl migrate import`) | 3.6 | M | ⬜ |
| G14 | Projections: durable cursors, real targets, lag metric | 3.7 | L | ⬜ |
| G15 | Interceptor pipeline | 3.8 | M | ⬜ |
| G16 | Snapshot scheduling | §5.7 | S | ⬜ |
| G17 | Determinism guard | LD-REPLAY | M | ⬜ |
| G18 | TTL reaper + branchless check | 8.5 | S | ⬜ |

### H — Performance and hardware (Phase 1, 4, 5)

| # | Gap | Ref | Size | Status |
|---|---|---|---|---|
| H1 | 64-B aligned hot structs; false-sharing padding | 1.1/1.2 | M | ⬜ |
| H2 | Zero-alloc arena on hot path | 1.3 | M | ⬜ |
| H3 | Linux huge pages, NUMA binding, pinned writers | 1.4/1.5 | M | ⬜ |
| H4 | AVX-512 / NEON; CPUID 0x1F | 1.6 | M | ⬜ |
| H5 | OS jitter guide, mlockall, MMCSS | 1.7 | S | ⬜ |
| H6 | Prefetch + branchless routing | 2.8/2.9 | M | ⬜ |
| H7 | Zero-copy decode (`bytes::Bytes`) | 4.4 | M | ⬜ |
| H8 | One wire format (VVNE retire/adopt) | D-04 | M | ⏸ |
| H9 | Event-loop network server | 4.1 | L | ⬜ |
| H10 | Compiled predicates | 5.5 | M | ⬜ |
| H11 | Separate reader binary; stale-by-default reads | 5.1 | M | ⬜ |
| H12 | Document mode | 5.2 | L | ⬜ |
| H13 | Hot/warm/cold read routing | 5.6 | M | ⬜ |
| H14 | Materialized views sync vs eventual; survive restart | 5.7, D-08 | M | ⏸ |
| H15 | TSC telemetry ring + shadow exporter | 6.10 | M | ⬜ |
| H16 | DPDK, RIO, RDMA, flow steering, PTP, cloud adapters | 4.2–4.9 | L | ⬜ |

### I — Config, S22, S23

| # | Gap | Ref | Size | Status |
|---|---|---|---|---|
| I1 | Master config schema | VV-CFG-010 | L | ⬜ |
| I2 | S22 semantic partitioning wired into serve; pruner | VV-PRJ-010..013 | L | ⬜ |
| I3 | S22 read models + cache wired; YAML-only | VV-PRJ-014..019 | M | ⬜ |
| I4 | S23 emitter + YAML routes; computed/nested/lookup fields | VV-EVT-001..016 | M | ⬜ |
| I5 | S23 replay + DLQ commands; E2E gRPC consumer | VV-EVT-010..012 | M | ⬜ |

### J — Tenancy, control plane, tools (Phase 6)

| # | Gap | Ref | Size | Status |
|---|---|---|---|---|
| J1 | Multi-tenancy | LD-MT, 6.4 | L | ⬜ |
| J2 | Shard policy engine | 6.2 | M | ⬜ |
| J3 | 4096 vnodes + catalog epoch | 6.3 | M | ⬜ |
| J4 | QoS tiers | 6.14 | M | ⬜ |
| J5 | TUI + read-only web dashboard | 6.12/6.13, D-06 | M | ⏸ |
| J6 | vvctl → cluster over gRPC | LD-CLI-02 | M | ⬜ |
| J7 | Admin ops (pause/resume, force snapshot, rebuild, drain) | §6.6 | M | ⬜ |
| J8 | Schema registry compatibility rules | 6.7 | S | ⬜ |
| J9 | Three server binaries | LD-RUNTIME-01 | M | ⬜ |

### K — vv-architect

| # | Gap | Ref | Size | Status |
|---|---|---|---|---|
| K1 | Root/reference classification | VA-01 | M | ⬜ |
| K2 | SQLite read-only + hardening | VA-02 | S | ⬜ |
| K3 | CLI validation; checked sizing math | VA-03 | S | ⬜ |
| K4 | Consumable `vectorvolt.yaml`; engine consumers | VA-04/06 | M | ⬜ |
| K5 | Emit S22/S23 YAML | VA-05 | M | ⬜ |
| K6 | Dockerfile, Helm, fixed K8s manifest, HugePages/pinning | VA-08/09/10/12, PP-82/83 | M | ⬜ |
| K7 | Terraform realism | VA-13 | M | ⬜ |
| K8 | Sizing model v2 | VA-14 | M | ⬜ |
| K9 | Scanner hardening, type-based relationships, Go methods | VA-15/16/17 | L | ⬜ |
| K10 | Postgres/MySQL scan; Python, OpenAPI/Proto | VA-18/24 | L | ⬜ |
| K11 | Method classification, .NET DLL, stub gen, `migrate analyze` | VA-20..23 | L | ⬜ |
| K12 | Raft/cluster wiring in manifests | VA-11 | M | ⬜ |

### L — Testing, sample app, migration (Phase 7)

| # | Gap | Ref | Size | Status |
|---|---|---|---|---|
| L1 | DST through engine + WAL (mock clock/disk/net) | 7.1 | L | ⬜ |
| L2 | Replay-equivalence test | 3.4 | M | ⬜ |
| L3 | Chaos suite | 7.4 | L | ⬜ |
| L4 | Sample apps on the engine | 7.2 | L | ⬜ |
| L5 | E2E load tests; perf gate in CI | 7.3, PP-85 | M | ⬜ |
| L6 | CDC reverse ingestion; backfill; migration demo | 7.5.4/7.5.5 | L | ⬜ |
| L7 | Reference Java app testbed | 7.6 | L | ⬜ |
| L8 | Full suite green on Linux CI | — | M | 🟡 |
| L9 | Privileged Linux CI job for root/network tests | — | S | ⬜ |
| L10 | Benchmarks with methodology + comparisons | 7.8 | M | ⬜ |

### M — Build, CI, release

| # | Gap | Ref | Size | Status |
|---|---|---|---|---|
| M1 | Move `vectorvolt/.github/workflows` to the repo root | PP-80 | M | ⬜ |
| M2 | Split TPM out of release lint | PP-81 | S | ⬜ |
| M3 | systemd unit, default config, image; real Brew sha256 | PP-82 | M | ⬜ |
| M4 | `.cargo/config.toml` hardcoded MSVC path | PP-84 | S | ⬜ |
| M5 | Pin toolchain (`rust-toolchain.toml`, CI uses 1.99) | — | S | ⬜ |
| M6 | `ekos/target/` files tracked in git | — | S | ⬜ |
| M7 | musl static builds | locked | M | ⬜ |
| M8 | Rolling upgrades + format versioning | §6.6 | M | ⬜ |

### N — Decisions and docs

| # | Gap | Ref | Size | Status |
|---|---|---|---|---|
| N1 | Decisions D-04…D-12 | §2 | S | ⏸ |
| N2 | REQUIREMENTS.md v4.0 | D-01..03 | M | ⬜ |
| N3 | Path-to-production items #2…#N | §8 | S | ⏸ |
| N4 | SLO hardware tier | §8 | S | ⏸ |
| N5 | EKOS future | §8 | S | ⏸ |
| N6 | README limitations, CHANGELOG, historical doc banners | §9 | S | ⬜ |
| N7 | Traceability catalog re-key | §9 | S | ⬜ |

### Gap register log

- 2026-10-31: register created. PR #647 (PP-01…13, 20–22, 25, 31, 32 + CI green-up) precedes batch A.
- 2026-10-31: **Batch A** (branch `feat/gaps-batch-a`): A1, A3–A13 ✅; A2 and A14 🟡 (remainders folded into C1 and B30). New tests in query, reader, net, engine, wal, store, ledger/orders/workflow, control.
- 2026-10-31: **Batch B** (branch `feat/gaps-batch-b`): 26 of 31 ✅; B6, B7, B14, B17, B31 🟡 with the remainder written in each row.
- 2026-10-31: **Batch C paused** (branch `feat/gaps-batch-c`, WIP commit, not gate-checked): C2, C5 done; C1 partial. **Resume:** finish C1 (+A2 key wiring), then wave 2 = C3 + C4 + C6, then run the full gate (fmt, clippy 1.99 win + linux-target, workspace tests), mark rows, open the batch C PR stacked on #649. Also note A12: a deleted `tail.seal` is still accepted.
- 2026-11-04: **Batch C complete** (branch `feat/gaps-batch-c`, rebased on main): C1–C6 ✅ (A2 remainder closed via C1). Storage key registry keeps every live owner; intent log encrypted through `TableRegistry::encryption_key()`.
- 2026-11-04: **Batch D complete** (branch `feat/gaps-batch-d`): D1–D5 ✅ — TLS/mTLS on all listeners, pgwire SCRAM/MD5 + extended protocol, `--pgwire-host`, IP allowlist and connection limits enforced, D5 SQL rules enforced.
- 2026-11-04: **Batch E part 1** (branch `feat/gaps-batch-e`): E3–E7, E10, E13, E14 ✅. E1, E2, E8, E9, E11, E12 remain for part 2.
- 2026-11-04: **Batch E part 2** (branch `feat/gaps-batch-e2`): E1, E2, E8, E9, E11, E12 ✅. **Batch E complete.**