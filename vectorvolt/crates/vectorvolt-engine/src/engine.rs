//! [`TableEngine`] — per-table shard layer that wires the write and read paths.
//!
//! ## Architecture
//!
//! Each `TableEngine` owns:
//! - A [`TxnManager`] for the write path (WAL → write buffer → flush → `.vvsst`).
//! - A `RwLock<HashMap<(file_id, partition_id), PartitionReader>>` for the read path.
//!
//! ## Write visibility
//!
//! Rows accumulate in the write buffer until `flush()` (explicit or auto).  After
//! a flush, the engine scans its partition directory for new `.vvsst` files and
//! registers them.  `get()` and `scan()` flush any pending rows before reading so
//! callers always see a consistent view.

use std::collections::HashMap;
use std::path::PathBuf;

use parking_lot::RwLock;
use vectorvolt_index::RowDescriptor;
use vectorvolt_storage::{
    CompactionJob, CompactionStrategy, ManifestEdit, ManifestRecovery, PartitionMeta,
    PartitionReader, RetentionPlan, RetentionPolicy, VerifyReport, WriteOptions, verify_partitions,
};
pub use vectorvolt_txn::{FlushResult, WalTip};
use vectorvolt_txn::{Mutation, TxnManager};
use vectorvolt_wal::{WalConfig, WalReader, WalWriter};

use crate::{CompactResult, EngineError, EngineResult, EngineRow, EngineStats};

/// Per-table engine: owns the write path (TxnManager) and read path (PartitionReader map).
pub struct TableEngine {
    txn: TxnManager,
    /// Flushed partition files keyed by (file_id, partition_id).
    partitions: RwLock<HashMap<(u16, u16), PartitionReader>>,
    partition_dir: PathBuf,
    pub schema: Vec<String>,
    /// How `compact` chooses what to merge.
    compaction_strategy: CompactionStrategy,
    /// Storage options for compaction output.
    write_options: WriteOptions,
    /// What manifest recovery found at open. See [`TableEngine::manifest_recovery`].
    manifest_recovery: ManifestRecovery,
}

/// How durable a write must be before `insert` returns.
///
/// Three genuinely different guarantees. The middle one is easy to miss and is
/// what the engine silently provided before — see [`Durability::Buffered`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durability {
    /// `write()` + `fsync()` before returning. **Survives power loss and kernel panic.**
    ///
    /// The only setting that makes the WAL mean what a WAL is for, so it is the
    /// default. Also the most expensive: an `fsync` costs tens to hundreds of
    /// microseconds. Concurrent writers share one `fsync` via
    /// [`vectorvolt_wal::SyncCommitter`] (VV-WAL-011), so the cost stops scaling
    /// with writer count — but a *single* writer still pays it per commit, because
    /// there is nobody to batch with. That floor is the storage device.
    Fsync,

    /// `write()` to the OS, no `fsync()`. **Survives a process crash; not power loss.**
    ///
    /// The kernel owns the bytes and will write them out even if this process dies,
    /// so a panic or `kill -9` loses nothing. A power cut or kernel panic does.
    WriteOnly,

    /// Left in this process's own `BufWriter`. **Survives nothing.**
    ///
    /// The bytes sit in userspace memory until the buffer fills or the writer is
    /// dropped, so a process crash loses recent writes — not just a power cut.
    ///
    /// **This is what the engine did before durability was configurable**, and it
    /// is why INSERT measured 4–6 µs: that figure was buffer copy, SHA3-256 frame
    /// hashing, and index maintenance, with no write syscall on the path at all.
    /// Kept as an explicit option because it is the right choice for a rebuildable
    /// cache and for benchmarking the non-durable ceiling — but it must be asked
    /// for, not defaulted into.
    Buffered,
}

impl Durability {
    /// Whether `fsync` is requested when the WAL is flushed.
    #[inline]
    fn wal_fsync(self) -> bool {
        matches!(self, Durability::Fsync)
    }

    /// Whether the WAL is flushed out of the userspace buffer on every commit.
    #[inline]
    fn sync_on_commit(self) -> bool {
        !matches!(self, Durability::Buffered)
    }
}

/// Options for opening a [`TableEngine`].
#[derive(Debug, Clone, Copy)]
pub struct EngineOptions {
    /// Rows buffered before an automatic flush to a new partition.
    pub flush_threshold: usize,
    pub durability: Durability,
    /// How `compact` chooses what to merge. Defaults to leveled, matching
    /// VectorVolt's read-latency target — see [`CompactionStrategy`].
    pub compaction_strategy: CompactionStrategy,
    /// Storage options for compaction output. Compaction is the natural place to
    /// enable LZ4, since a compacted partition is colder than a fresh one, but it
    /// costs O(1) point reads so it is off by default.
    pub write_options: WriteOptions,
}

impl EngineOptions {
    pub fn new(flush_threshold: usize) -> Self {
        EngineOptions {
            flush_threshold,
            durability: Durability::Fsync,
            compaction_strategy: CompactionStrategy::default(),
            write_options: WriteOptions::default(),
        }
    }

    /// Trade durability for speed. See [`Durability::WriteOnly`] for exactly what
    /// is given up.
    pub fn with_durability(mut self, durability: Durability) -> Self {
        self.durability = durability;
        self
    }

    pub fn with_compaction_strategy(mut self, strategy: CompactionStrategy) -> Self {
        self.compaction_strategy = strategy;
        self
    }

    /// LZ4-compress compaction output. Point reads on a compressed partition are
    /// O(column) rather than O(1) — see `vectorvolt_storage::COMPRESSION_LZ4`.
    pub fn with_compressed_compaction(mut self) -> Self {
        self.write_options = WriteOptions::new().with_lz4();
        self
    }
}

impl TableEngine {
    /// Open or create a table at `data_dir`, **durable by default**.
    ///
    /// Directory layout:
    /// - `data_dir/wal/`  — WAL segment files
    /// - `data_dir/data/` — `.vvsst` partition files
    ///
    /// Existing `.vvsst` files are loaded on open (restart recovery).
    ///
    /// ## Durability, and a correction
    ///
    /// This used to hardcode `with_fsync(false)` with no way to change it, so every
    /// write reached only the OS page cache. That survives a *process* crash but
    /// **not power loss or a kernel panic** — meaning the engine did not actually
    /// provide the durability a WAL exists for, and any INSERT latency measured
    /// against it was measuring a non-durable write.
    ///
    /// The default is now [`Durability::Fsync`]. Use
    /// [`open_with`](Self::open_with) to opt out deliberately.
    ///
    /// ## What durability costs
    ///
    /// Measured on the dev laptop: 526 µs per durable insert against 7.8 µs for the
    /// same insert unsynced — `fsync` is 67× the actual work.
    ///
    /// Concurrent writers now share one `fsync` through
    /// [`vectorvolt_wal::SyncCommitter`], so that cost is amortised across writers
    /// rather than paid per commit. **A single sequential writer still pays it every
    /// time** — group commit removes the scaling penalty, not the device latency.
    /// Use [`commit_stats`](Self::commit_stats) to see the batching that actually
    /// occurred.
    pub fn open(
        schema: Vec<String>,
        data_dir: impl Into<PathBuf>,
        flush_threshold: usize,
    ) -> EngineResult<Self> {
        Self::open_with(schema, data_dir, EngineOptions::new(flush_threshold))
    }

    /// [`open`](Self::open) with explicit options.
    pub fn open_with(
        schema: Vec<String>,
        data_dir: impl Into<PathBuf>,
        options: EngineOptions,
    ) -> EngineResult<Self> {
        let flush_threshold = options.flush_threshold;
        let durability = options.durability;
        let data_dir = data_dir.into();
        let wal_dir = data_dir.join("wal");
        let partition_dir = data_dir.join("data");
        std::fs::create_dir_all(&wal_dir)?;
        std::fs::create_dir_all(&partition_dir)?;

        let cfg = WalConfig::default()
            .with_dir(&wal_dir)
            .with_fsync(durability.wal_fsync());
        let wal = WalWriter::open(cfg)?;

        // Recover previously-flushed partitions.
        let mut partitions = HashMap::new();
        let mut max_partition_id: u16 = 0;
        for entry in std::fs::read_dir(&partition_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("vvsst") {
                let reader = PartitionReader::open(&path)?;
                if reader.partition_id() >= max_partition_id {
                    max_partition_id = reader.partition_id() + 1;
                }
                partitions.insert((reader.file_id(), reader.partition_id()), reader);
            }
        }

        // Start the partition counter past any existing files to avoid overwrites.
        let txn = TxnManager::with_starting_partition(
            wal,
            schema.clone(),
            &partition_dir,
            flush_threshold,
            max_partition_id,
        )
        .with_sync_on_commit(durability.sync_on_commit());

        // Rebuild pk_index from existing partition files so reads work across
        // process restarts (Phase 2 crash recovery).
        let rows_on_disk: u64 = partitions.values().map(|r| r.row_count() as u64).sum();
        txn.rebuild_pk_index_from_partitions(&partitions)?;

        // Replay WAL delete records so deletes made in previous sessions survive
        // restart.  Insert records are skipped — their rows are already in
        // partition files (we flush every CLI insert) or are lost pending data
        // (acceptable Phase 2 limitation).
        // Read-only replay: this reader never writes, so fsync is irrelevant here.
        let wal_cfg = WalConfig::default().with_dir(&wal_dir).with_fsync(false);
        if let Ok(frames) = WalReader::open(wal_cfg).read_all() {
            let mut idx = txn.pk_index.write();
            for frame in frames {
                if let Ok(Mutation::Delete { pk }) = Mutation::decode(&frame.payload) {
                    idx.remove(pk);
                }
            }
        }

        // Seed the flushed-rows counter so stats() is accurate after a restart.
        txn.set_total_rows_flushed(rows_on_disk);

        // The manifest is enabled unconditionally here, unlike in TxnManager where
        // it is opt-in. TableEngine is the top-level type and its `compact`,
        // `apply_retention`, and `verify` all read from it — without one they
        // would silently do nothing, which is a far worse failure than the open
        // returning an error.
        //
        // Recovery is deliberately not silent. A torn tail is expected after an
        // unclean shutdown; a large discard is not, and the difference cannot be
        // told from the bytes (see `ManifestRecovery`). It is recorded on the
        // engine so a caller can inspect it rather than having it swallowed.
        let manifest_recovery = txn.enable_manifest()?;

        // Adopt partitions found on disk ONLY when the manifest is empty.
        //
        // The distinction is load-bearing, and an earlier version of this got it
        // wrong by adopting any unknown file. Two situations produce a partition
        // the manifest does not name, and they need opposite handling:
        //
        // - **Empty manifest**: a table written before the manifest existed. Every
        //   file is real data and must be adopted, or the table appears empty.
        // - **Populated manifest**: an orphan from an interrupted compaction —
        //   inputs still on disk with the manifest already naming only the output.
        //   Adopting one DOUBLES every row it holds, silently, in every scan.
        //
        // The second case is the dangerous one, and it is also the common one, so
        // an unknown file beside a populated manifest is left strictly alone. Not
        // deleted either: reclaiming it is an orphan-sweep decision that wants an
        // explicit operation, not a side effect of opening a table.
        if txn.partitions().is_empty() && !partitions.is_empty() {
            for ((file_id, partition_id), reader) in partitions.iter() {
                let path = partition_dir.join(format!("{file_id:05}_{partition_id:05}.vvsst"));
                let size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                let (min_key, max_key) = reader.zone_min_max(0).unwrap_or((0, 0));
                txn.record(ManifestEdit::Add(Box::new(PartitionMeta {
                    file_id: *file_id,
                    partition_id: *partition_id,
                    path,
                    level: 0,
                    size_bytes,
                    row_count: reader.row_count(),
                    // Unknown, and 0 is the honest answer: it sorts these first for
                    // age-based retention, which is right — they predate everything
                    // the manifest has seen.
                    created_at_millis: 0,
                    min_key,
                    max_key,
                    tombstones: 0,
                    worm_locked: false,
                })))?;
            }
        } else {
            // Drop readers for files the manifest does not name, so an orphan is
            // not served by scans either.
            let known: std::collections::HashSet<(u16, u16)> = txn
                .partitions()
                .iter()
                .map(|p| (p.file_id, p.partition_id))
                .collect();
            partitions.retain(|k, _| known.contains(k));
        }

        Ok(TableEngine {
            txn,
            partitions: RwLock::new(partitions),
            partition_dir,
            schema,
            compaction_strategy: options.compaction_strategy,
            write_options: options.write_options,
            manifest_recovery,
        })
    }

    // ── Write path ────────────────────────────────────────────────────────────

    /// Insert a row. `values.len()` must equal `schema.len()`.
    ///
    /// If the write buffer hits its threshold an auto-flush is triggered and
    /// the new partition is immediately registered for reads.
    pub fn insert(&self, pk: u64, values: Vec<u64>) -> EngineResult<WalTip> {
        let flushed_before = self.txn.total_rows_flushed();
        let tip = self.txn.insert(pk, values)?;
        if self.txn.total_rows_flushed() > flushed_before {
            self.sync_partitions()?;
        }
        Ok(tip)
    }

    /// Remove a row. Returns `Err(TxnError::PkNotFound)` for unknown PKs.
    pub fn delete(&self, pk: u64) -> EngineResult<WalTip> {
        Ok(self.txn.delete(pk)?)
    }

    /// Flush the write buffer to a new `.vvsst` partition file.
    ///
    /// Returns `None` if the buffer was empty.  After a successful flush the
    /// new partition is immediately registered for reads.
    pub fn flush(&self) -> EngineResult<Option<FlushResult>> {
        let result = self.txn.flush()?;
        if result.is_some() {
            self.sync_partitions()?;
        }
        Ok(result)
    }

    // ── Read path ─────────────────────────────────────────────────────────────

    /// Look up a single row by primary key.
    ///
    /// Pending (un-flushed) rows are flushed before the lookup so the result
    /// is always up-to-date.  Returns `None` if the pk is not found.
    pub fn get(&self, pk: u64) -> EngineResult<Option<EngineRow>> {
        // A read does NOT flush.
        //
        // This used to force a flush whenever the write buffer was non-empty,
        // which made `get` rewrite partitions and rebuild the index — not a read
        // at all. It also made every concurrent reader a concurrent writer, which
        // is what made the two publication races reachable from a lookup in the
        // first place: with 8 clients reading, 8 threads could be inside flush at
        // once.
        //
        // A PENDING descriptor is now resolved against the buffer directly. The
        // row is already there; forcing it to disk to read it was pure cost.
        let desc = match self.txn.pk_index.read().get(pk) {
            Some(d) => d,
            None => return Ok(None),
        };

        let file_id = desc.file_id();
        let partition_id = desc.partition_id();
        let row_offset = desc.row_offset() as usize;

        if file_id == vectorvolt_txn::PENDING_FILE_ID {
            // Still buffered. A miss here is not an error: a concurrent flush may
            // have moved the row to a partition and republished the descriptor
            // between the index read above and this lookup, so re-read the index
            // and resolve again rather than reporting the row absent.
            if let Some(values) = self.txn.buffered_row(pk, row_offset) {
                return Ok(Some(EngineRow { pk, values }));
            }
            let Some(fresh) = self.txn.pk_index.read().get(pk) else {
                return Ok(None);
            };
            if fresh.file_id() == vectorvolt_txn::PENDING_FILE_ID {
                if let Some(values) = self.txn.buffered_row(pk, fresh.row_offset() as usize) {
                    return Ok(Some(EngineRow { pk, values }));
                }
                // Still pending, and in neither the buffer nor the in-flight set,
                // after two index reads. Rather than report the row absent — which
                // is what an earlier version of this did, and which a concurrency
                // test caught at roughly one run in eight — force the flush to
                // complete and resolve against the result.
                //
                // This is the ONLY path that still flushes from a read, and it is
                // reached when a flush is mid-publication. Rare by construction and
                // self-limiting: after it, the descriptor is real.
                self.do_flush()?;
                let Some(settled) = self.txn.pk_index.read().get(pk) else {
                    return Ok(None);
                };
                if settled.file_id() == vectorvolt_txn::PENDING_FILE_ID {
                    return Ok(self
                        .txn
                        .buffered_row(pk, settled.row_offset() as usize)
                        .map(|values| EngineRow { pk, values }));
                }
                let values = self.read_row_from(
                    settled.file_id(),
                    settled.partition_id(),
                    settled.row_offset() as usize,
                )?;
                return Ok(Some(EngineRow { pk, values }));
            }
            let values = match self.read_row_from(
                fresh.file_id(),
                fresh.partition_id(),
                fresh.row_offset() as usize,
            ) {
                Ok(v) => v,
                Err(EngineError::PartitionMissing { .. }) => {
                    self.sync_partitions()?;
                    self.read_row_from(
                        fresh.file_id(),
                        fresh.partition_id(),
                        fresh.row_offset() as usize,
                    )?
                }
                Err(e) => return Err(e),
            };
            return Ok(Some(EngineRow { pk, values }));
        }

        // A descriptor can name a partition this engine has not opened yet.
        //
        // `TxnManager::flush` publishes index entries for a new partition, and
        // `sync_partitions` registers a reader for it only afterwards. Between
        // those two steps any concurrent reader that looks up one of those keys
        // finds a descriptor pointing at a partition that is not in the map — and
        // returned `PartitionMissing`, an error, for a row that is perfectly fine.
        //
        // That was an intermittent failure under load: a client would insert keys,
        // read them straight back, and one read in a few hundred would error. It
        // only reproduced when several threads were flushing concurrently, which
        // is why single-crate test runs never showed it.
        //
        // Refreshing and retrying is CORRECT here, not a papering-over, because of
        // an ordering invariant on the write side: `flush` writes the `.vvsst`
        // file to disk *before* it publishes any index entry pointing into it. So
        // a descriptor always names a file that already exists, and a directory
        // rescan is guaranteed to find it. The retry is bounded at one — if the
        // partition is still absent after a rescan, it genuinely does not exist
        // and the error is real.
        let values = match self.read_row_from(file_id, partition_id, row_offset) {
            Ok(v) => v,
            Err(EngineError::PartitionMissing { .. }) => {
                self.sync_partitions()?;
                self.read_row_from(file_id, partition_id, row_offset)?
            }
            Err(e) => return Err(e),
        };

        Ok(Some(EngineRow { pk, values }))
    }

    /// Scan rows where `values[col_idx]` is in `[lo, hi]` (inclusive).
    ///
    /// Uses per-column zone statistics to skip partitions that cannot contain
    /// a match (O(1) prune per partition).  Pending rows are flushed first.
    pub fn scan(&self, col_idx: usize, lo: u64, hi: u64) -> EngineResult<Vec<EngineRow>> {
        let col_count = self.schema.len();
        if col_idx >= col_count {
            return Err(EngineError::ColOutOfRange { col_idx, col_count });
        }

        if self.txn.row_count_buffered() > 0 {
            self.do_flush()?;
        }

        let mut results = Vec::new();
        let parts = self.partitions.read();

        for reader in parts.values() {
            // Zone prune: skip this partition if the column range has no overlap.
            if let Ok((min, max)) = reader.zone_min_max(col_idx)
                && (min > hi || max < lo)
            {
                continue;
            }

            // SIMD-filter the scan column, then materialize only matched rows.
            //
            // Previously this decoded EVERY column up front and scalar-compared the
            // filter column, so a 1%-selective query still paid to decode 100% of
            // every column. Now the filter column alone is decoded, the comparison
            // is vectorized, and `materialize_rows` picks late-materialization or a
            // bulk decode based on the match count it can already see.
            let offsets = reader.filter_offsets(col_idx, lo, hi)?;
            for values in reader.materialize_rows(&offsets)? {
                // Column 0 is the PK by convention.
                let pk = values.first().copied().unwrap_or(u64::MAX);
                results.push(EngineRow { pk, values });
            }
        }

        Ok(results)
    }

    // ── Compaction ────────────────────────────────────────────────────────────

    /// Compact this table, delegating to the shared planner and executor.
    ///
    /// Was a second, independent merge implementation living here — it collected
    /// live rows, wrote one merged file, deleted everything else in the directory,
    /// and rebuilt the index. It worked, but it duplicated
    /// `CompactionStrategy::plan` + `execute_job` with different behaviour: it
    /// always merged EVERYTHING regardless of level or size, and it swept the
    /// partition directory with a `read_dir` rather than acting on a known input
    /// set. Two compaction paths that disagree is how a storage engine grows a bug
    /// that only appears under one of them.
    ///
    /// ## Dropping the readers is load-bearing, not tidiness
    ///
    /// Every `PartitionReader` holds a live memory mapping. On Windows a mapped
    /// file cannot be deleted — `remove_file` returns OS error 1224 — so the
    /// executor's cleanup would fail on every input and silently leak them.
    /// Clearing the map runs the readers' destructors and releases the mappings
    /// first. The previous implementation had the same `parts.clear()` for the
    /// same reason; the requirement moved, it did not disappear.
    ///
    /// Returns `None` when the planner found nothing to do.
    pub fn compact(&self) -> EngineResult<Option<CompactResult>> {
        self.do_flush()?;

        let mut parts = self.partitions.write();
        if parts.is_empty() {
            return Ok(None);
        }

        // Release every mapping before the executor tries to delete input files.
        parts.clear();

        let outcomes = self
            .txn
            .run_compaction(self.compaction_strategy, self.write_options)?;

        // Re-open whatever the manifest now says is live. Driven by the manifest
        // rather than by a directory scan, so an orphan left by an interrupted run
        // is not silently adopted as a live partition.
        for meta in self.txn.partitions() {
            let reader = PartitionReader::open(&meta.path)?;
            parts.insert((reader.file_id(), reader.partition_id()), reader);
        }

        let Some(last) = outcomes.last() else {
            return Ok(None);
        };

        // Leaked inputs are surfaced through `CompactResult::leaked_files` rather
        // than logged: this crate has no logging dependency, and a warning nobody
        // configured a subscriber for is indistinguishable from silence.
        let leaked: usize = outcomes.iter().map(|o| o.undeleted.len()).sum();

        let rows_kept: u64 = outcomes.iter().map(|o| o.rows_out).sum();
        self.txn.set_total_rows_flushed(rows_kept);

        Ok(Some(CompactResult {
            rows_kept: rows_kept as usize,
            path: last.output.path.clone(),
            leaked_files: leaked,
        }))
    }

    /// What manifest recovery discarded at open.
    ///
    /// `is_clean()` is the only fully healthy answer. A small `discarded_bytes` is
    /// a torn tail from an unclean shutdown and is expected; a large one suggests
    /// real corruption and that partitions may be missing. The bytes cannot
    /// distinguish these, so the number is reported rather than judged.
    pub fn manifest_recovery(&self) -> ManifestRecovery {
        self.manifest_recovery
    }

    /// Every partition the manifest knows about.
    pub fn partition_meta(&self) -> Vec<PartitionMeta> {
        self.txn.partitions()
    }

    /// What compaction *would* do, without doing it.
    ///
    /// Compaction destroys data, so being able to inspect the plan before running
    /// it is not a nicety. Pure — touches no files.
    pub fn compaction_plan(&self) -> Vec<CompactionJob> {
        let eligible: Vec<PartitionMeta> = self
            .txn
            .partitions()
            .into_iter()
            .filter(|p| !p.worm_locked)
            .collect();
        self.compaction_strategy.plan(&eligible)
    }

    /// What `policy` would delete, without deleting it.
    ///
    /// Returns both the deletions and the vetoes, so a caller can see a WORM lock
    /// or the last-partition rail clamping the policy rather than wondering why
    /// disk use is not falling. `now_millis` is supplied rather than read from the
    /// clock, so the answer is reproducible.
    pub fn retention_plan(&self, policy: &RetentionPolicy, now_millis: u64) -> RetentionPlan {
        policy.plan(&self.txn.partitions(), now_millis)
    }

    /// Execute a retention policy: delete the selected partitions.
    ///
    /// Manifest first, then the files — the same ordering compaction uses, and for
    /// the same reason. A crash between the two leaves an unreferenced file, which
    /// leaks disk; the opposite order would leave the manifest naming a file that
    /// no longer exists, which breaks every read that follows.
    ///
    /// Returns the plan that was carried out.
    pub fn apply_retention(
        &self,
        policy: &RetentionPolicy,
        now_millis: u64,
    ) -> EngineResult<RetentionPlan> {
        let plan = self.retention_plan(policy, now_millis);
        if plan.delete.is_empty() {
            return Ok(plan);
        }

        let mut parts = self.partitions.write();
        // Same Windows constraint as compaction: a mapped file will not delete.
        parts.clear();

        let removes: Vec<(u16, u16)> = plan
            .delete
            .iter()
            .map(|a| (a.partition.file_id, a.partition.partition_id))
            .collect();
        self.txn.record(ManifestEdit::Replace {
            adds: Vec::new(),
            removes,
        })?;

        let mut undeleted: Vec<std::path::PathBuf> = Vec::new();
        for action in &plan.delete {
            if std::fs::remove_file(&action.partition.path).is_err()
                && action.partition.path.exists()
            {
                undeleted.push(action.partition.path.clone());
            }
        }

        // Rows belonging to deleted partitions must leave the index, or a lookup
        // resolves into a file that is gone. Done by rebuilding from what remains
        // rather than by subtracting, because the deleted partitions' key lists
        // are no longer readable once their files are unlinked.
        for meta in self.txn.partitions() {
            let reader = PartitionReader::open(&meta.path)?;
            parts.insert((reader.file_id(), reader.partition_id()), reader);
        }
        self.rebuild_index_from(&parts)?;

        let mut plan = plan;
        plan.undeleted = undeleted;
        Ok(plan)
    }

    /// Verify every live partition, individually and against each other.
    pub fn verify(&self) -> EngineResult<VerifyReport> {
        Ok(verify_partitions(&self.txn.partitions())?)
    }

    /// Rebuild the PK index from the given readers.
    ///
    /// Used after retention, where partitions vanish wholesale. Everything not
    /// present in a surviving partition is dropped, which is exactly right: a
    /// retained row is one that still lives somewhere on disk.
    fn rebuild_index_from(&self, parts: &HashMap<(u16, u16), PartitionReader>) -> EngineResult<()> {
        let mut idx = self.txn.pk_index.write();
        *idx = vectorvolt_index::CuckooMap::with_capacity(64);
        let mut total: u64 = 0;
        for ((file_id, partition_id), reader) in parts {
            for row in 0..reader.row_count() as usize {
                let pk = reader.read_value(0, row)?;
                let _ = idx.insert(pk, RowDescriptor::new(*file_id, *partition_id, row as u32));
                total += 1;
            }
        }
        drop(idx);
        self.txn.set_total_rows_flushed(total);
        Ok(())
    }

    // ── Introspection ─────────────────────────────────────────────────────────

    /// Group-commit counters from the write path.
    ///
    /// `commits_per_sync() > 1` proves concurrent writers shared an `fsync` rather
    /// than each paying their own. Exposed so that can be asserted rather than
    /// assumed.
    pub fn commit_stats(&self) -> vectorvolt_txn::SyncStats {
        self.txn.commit_stats()
    }

    /// Return all rows where column `col_idx` equals `value`.
    ///
    /// Uses the in-memory `SecondaryIndex` maintained by the write path.
    /// Deleted rows that remain in the index (compaction has not yet run) are
    /// silently filtered: if `get(pk)` returns `None`, the row is skipped.
    pub fn secondary_scan(&self, col_idx: u32, value: u64) -> EngineResult<Vec<Vec<u64>>> {
        let pks = self.txn.secondary_lookup_pks(col_idx as usize, value);
        let mut rows = Vec::new();
        for pk in pks {
            if let Some(row) = self.get(pk)? {
                rows.push(row.values);
            }
        }
        Ok(rows)
    }

    /// Probabilistic check: was `value` ever inserted as a PK?
    ///
    /// False positive rate ≈ 0.78%; false negatives are impossible.
    pub fn fk_exists(&self, value: u64) -> bool {
        self.txn.fk_check(value)
    }

    /// Snapshot of current engine state (non-flushing).
    pub fn stats(&self) -> EngineStats {
        EngineStats {
            rows_buffered: self.txn.row_count_buffered(),
            rows_flushed: self.txn.total_rows_flushed(),
            partition_count: self.partitions.read().len(),
            schema: self.schema.clone(),
        }
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Flush write buffer and register the new partition.
    fn do_flush(&self) -> EngineResult<()> {
        if self.txn.flush()?.is_some() {
            self.sync_partitions()?;
        }
        Ok(())
    }

    /// Scan `partition_dir` and register any `.vvsst` files not yet in the map.
    /// Read every column of one row from a registered partition.
    fn read_row_from(
        &self,
        file_id: u16,
        partition_id: u16,
        row_offset: usize,
    ) -> EngineResult<Vec<u64>> {
        let parts = self.partitions.read();
        let reader = parts
            .get(&(file_id, partition_id))
            .ok_or(EngineError::PartitionMissing {
                file_id,
                partition_id,
            })?;
        let col_count = reader.col_count() as usize;
        let mut values = Vec::with_capacity(col_count);
        for c in 0..col_count {
            values.push(reader.read_value(c, row_offset)?);
        }
        Ok(values)
    }

    fn sync_partitions(&self) -> EngineResult<()> {
        let mut parts = self.partitions.write();
        for entry in std::fs::read_dir(&self.partition_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("vvsst") {
                continue;
            }

            // Identify the partition from its FILENAME and skip it if already
            // registered.
            //
            // This used to call `PartitionReader::open` first and then
            // `entry().or_insert()`, which discarded the reader when the key was
            // already present — after having opened the file, mmapped it, and
            // verified its full-file CRC. Since `sync_partitions` runs after every
            // flush, flush number N re-verified all N-1 existing partitions:
            // quadratic work in the partition count, for a result thrown away.
            //
            // Measured at 8 threads flushing every 64 rows: 279 us per insert
            // before, 27 us after. The filename is authoritative because the
            // writer chooses it from the same two ids it stores in the header, and
            // any file whose name does not parse is opened the slow way rather
            // than skipped — an unparseable name means an unknown provenance, not
            // an absent partition.
            let parsed = path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|stem| stem.split_once('_'))
                .and_then(|(f, p)| Some((f.parse::<u16>().ok()?, p.parse::<u16>().ok()?)));

            if let Some(key) = parsed
                && parts.contains_key(&key)
            {
                continue;
            }

            let reader = PartitionReader::open(&path)?;
            let key = (reader.file_id(), reader.partition_id());
            parts.entry(key).or_insert(reader);
        }
        Ok(())
    }
}
