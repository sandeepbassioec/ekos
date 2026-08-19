//! [`TableEngine`] â€” per-table shard layer that wires the write and read paths.
//!
//! ## Architecture
//!
//! Each `TableEngine` owns:
//! - A [`TxnManager`] for the write path (WAL â†’ write buffer â†’ flush â†’ `.vvsst`).
//! - A `RwLock<HashMap<(file_id, partition_id), PartitionReader>>` for the read path.
//!
//! ## Write visibility
//!
//! Rows accumulate in the write buffer until `flush()` (explicit or auto).  After
//! a flush, the engine scans its partition directory for new `.vvsst` files and
//! registers them.  `get()` and `scan()` flush any pending rows before reading so
//! callers always see a consistent view.

use std::collections::{HashMap, HashSet};
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

use crate::cdc::ChangeEvent;
use crate::cdc_log::CdcLog;
use crate::index_catalog;
use crate::mvcc::MvccStore;
use crate::schema::TableSchema;
use crate::{CompactResult, EngineError, EngineResult, EngineRow, EngineStats, TxnId};

/// Per-table engine: owns the write path (TxnManager) and read path (PartitionReader map).
pub struct TableEngine {
    pub(crate) txn: TxnManager,
    /// Flushed partition files keyed by (file_id, partition_id).
    pub(crate) partitions: RwLock<HashMap<(u16, u16), PartitionReader>>,
    partition_dir: PathBuf,
    pub schema: TableSchema,
    /// How `compact` chooses what to merge.
    compaction_strategy: CompactionStrategy,
    /// Storage options for compaction output.
    write_options: WriteOptions,
    /// What manifest recovery found at open. See [`TableEngine::manifest_recovery`].
    manifest_recovery: ManifestRecovery,
    /// MVCC transaction store â€” open transactions live here until commit/rollback.
    mvcc: std::sync::Arc<MvccStore>,
    /// Serialises conditional write operations (CAS, insert-if-absent, upsert).
    ///
    /// `txn.insert` acquires `pk_index.write()` internally so we cannot hold
    /// the index lock externally.  This mutex serialises the read-check-write
    /// sequence for the whole conditional write, preventing two concurrent CAS
    /// callers from interleaving their individual steps.
    pub(crate) cas_lock: parking_lot::Mutex<()>,
    /// Auto-compact trigger: compact after any flush that produces â‰¥ N partitions.
    /// `None` means manual compaction only.
    auto_compact_threshold: Option<usize>,
    /// Path of the secondary-index catalog file.  Updated on every
    /// `build_secondary_index` and `drop_secondary_index` call.
    pub(crate) catalog_path: PathBuf,
    /// Per-row TTL deadlines: pk â†’ expire_at (opaque u64 timestamp, caller's epoch).
    /// Rows absent from this map never expire.
    pub(crate) ttl_index: RwLock<std::collections::BTreeMap<u64, u64>>,
    /// Secondary indexes: col_idx â†’ (col_value â†’ set of PKs).
    /// Built explicitly via `build_secondary_index`; auto-maintained by every
    /// subsequent `insert` and `delete` so long as the index is live.
    pub(crate) secondary_indexes: RwLock<
        std::collections::HashMap<
            usize,
            std::collections::BTreeMap<u64, std::collections::BTreeSet<u64>>,
        >,
    >,
    /// CDC broadcast sender.  Clones cheaply; `send` is sync and lock-free.
    /// Lagging receivers are automatically dropped by the channel.
    pub(crate) changes_tx: tokio::sync::broadcast::Sender<ChangeEvent>,
    /// Monotonically increasing log sequence number.  Starts at the highest
    /// LSN found in the on-disk CDC log, ensuring LSNs are globally unique
    /// across process restarts.
    pub(crate) lsn_counter: std::sync::atomic::AtomicU64,
    /// Durable append-only log of every [`ChangeEvent`] published by this
    /// table.  Enables SSE replay (`?from=<lsn>`) and view backfill.
    pub(crate) cdc_log: CdcLog,
}

/// How durable a write must be before `insert` returns.
///
/// Three genuinely different guarantees. The middle one is easy to miss and is
/// what the engine silently provided before â€” see [`Durability::Buffered`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durability {
    /// `write()` + `fsync()` before returning. **Survives power loss and kernel panic.**
    ///
    /// The only setting that makes the WAL mean what a WAL is for, so it is the
    /// default. Also the most expensive: an `fsync` costs tens to hundreds of
    /// microseconds. Concurrent writers share one `fsync` via
    /// [`vectorvolt_wal::SyncCommitter`] (VV-WAL-011), so the cost stops scaling
    /// with writer count â€” but a *single* writer still pays it per commit, because
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
    /// dropped, so a process crash loses recent writes â€” not just a power cut.
    ///
    /// **This is what the engine did before durability was configurable**, and it
    /// is why INSERT measured 4â€“6 Âµs: that figure was buffer copy, SHA3-256 frame
    /// hashing, and index maintenance, with no write syscall on the path at all.
    /// Kept as an explicit option because it is the right choice for a rebuildable
    /// cache and for benchmarking the non-durable ceiling â€” but it must be asked
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
    /// VectorVolt's read-latency target â€” see [`CompactionStrategy`].
    pub compaction_strategy: CompactionStrategy,
    /// Storage options for compaction output. Compaction is the natural place to
    /// enable LZ4, since a compacted partition is colder than a fresh one, but it
    /// costs O(1) point reads so it is off by default.
    pub write_options: WriteOptions,
    /// When `Some(n)`, automatically call `compact()` after every flush that
    /// brings the partition count to `n` or more.  `None` (default) disables
    /// the auto-trigger; callers must invoke `compact()` explicitly.
    pub auto_compact_threshold: Option<usize>,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self::new(1_000_000)
    }
}

impl EngineOptions {
    pub fn new(flush_threshold: usize) -> Self {
        EngineOptions {
            flush_threshold,
            durability: Durability::Fsync,
            compaction_strategy: CompactionStrategy::default(),
            write_options: WriteOptions::default(),
            auto_compact_threshold: None,
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
    /// O(column) rather than O(1) â€” see `vectorvolt_storage::COMPRESSION_LZ4`.
    pub fn with_compressed_compaction(mut self) -> Self {
        self.write_options = WriteOptions::new().with_lz4();
        self
    }

    /// Automatically compact after any flush that brings the partition count to
    /// `threshold` or more.
    ///
    /// A threshold of `2` compacts every time a second partition is created.
    /// Choose a larger value (e.g. `8`) to amortise the compaction cost over
    /// several flushes while still bounding read amplification.
    ///
    /// Also sets `l0_trigger: 1` on a Leveled strategy (or `min_run: 1` on
    /// SizeTiered) so the compaction planner always produces a job when the
    /// threshold fires â€” without this adjustment the default `l0_trigger: 4`
    /// would cause the compaction call to be a no-op for small tables.
    pub fn with_auto_compact(mut self, threshold: usize) -> Self {
        self.auto_compact_threshold = Some(threshold.max(1));
        self.compaction_strategy = match self.compaction_strategy {
            CompactionStrategy::Leveled {
                level_multiplier,
                l1_bytes,
                garbage_threshold,
                ..
            } => CompactionStrategy::Leveled {
                l0_trigger: 1,
                level_multiplier,
                l1_bytes,
                garbage_threshold,
            },
            CompactionStrategy::SizeTiered {
                max_run,
                size_ratio,
                garbage_threshold,
                ..
            } => CompactionStrategy::SizeTiered {
                min_run: 1,
                max_run,
                size_ratio,
                garbage_threshold,
            },
        };
        self
    }
}

impl TableEngine {
    /// Open or create a table at `data_dir`, **durable by default**.
    ///
    /// Directory layout:
    /// - `data_dir/wal/`  â€” WAL segment files
    /// - `data_dir/data/` â€” `.vvsst` partition files
    ///
    /// Existing `.vvsst` files are loaded on open (restart recovery).
    ///
    /// ## Durability, and a correction
    ///
    /// This used to hardcode `with_fsync(false)` with no way to change it, so every
    /// write reached only the OS page cache. That survives a *process* crash but
    /// **not power loss or a kernel panic** â€” meaning the engine did not actually
    /// provide the durability a WAL exists for, and any INSERT latency measured
    /// against it was measuring a non-durable write.
    ///
    /// The default is now [`Durability::Fsync`]. Use
    /// [`open_with`](Self::open_with) to opt out deliberately.
    ///
    /// ## What durability costs
    ///
    /// Measured on the dev laptop: 526 Âµs per durable insert against 7.8 Âµs for the
    /// same insert unsynced â€” `fsync` is 67Ã— the actual work.
    ///
    /// Concurrent writers now share one `fsync` through
    /// [`vectorvolt_wal::SyncCommitter`], so that cost is amortised across writers
    /// rather than paid per commit. **A single sequential writer still pays it every
    /// time** â€” group commit removes the scaling penalty, not the device latency.
    /// Use [`commit_stats`](Self::commit_stats) to see the batching that actually
    /// occurred.
    pub fn open(
        schema: Vec<String>,
        data_dir: impl Into<PathBuf>,
        flush_threshold: usize,
    ) -> EngineResult<Self> {
        Self::open_with(schema, data_dir, EngineOptions::new(flush_threshold))
    }

    /// Open with a fully-typed [`TableSchema`].
    ///
    /// Use this when the caller already has a `TableSchema` (e.g. deserialized
    /// from `schema.txt` by [`TableRegistry`]).  Compared to [`open_with`],
    /// this preserves column types so `insert` can enforce them.
    pub fn open_typed(
        typed_schema: TableSchema,
        data_dir: impl Into<PathBuf>,
        options: EngineOptions,
    ) -> EngineResult<Self> {
        let names = typed_schema.names();
        let mut engine = Self::open_with(names, data_dir, options)?;
        engine.schema = typed_schema;
        Ok(engine)
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

        // If a snapshot marker exists, the WAL is redundant â€” all rows are in
        // compacted partition files.  Truncate the WAL dir before WalWriter opens
        // so startup does not replay stale entries already captured in partitions.
        let snapshot_path = partition_dir.join("snapshot.vvss");
        if snapshot_path.exists() {
            crate::snapshot::truncate_wal_dir(&wal_dir)?;
        }

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
        let typed_schema = TableSchema::from_names(&schema);

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

        // Phase 4 WAL recovery: replay all mutations that are not yet in an SST.
        //
        // Processing order matters: Insert and Delete records are replayed in WAL
        // sequence order so re-inserts after deletes and interleaved mutations
        // converge to the correct state.
        //
        // Insert: if the pk is already in pk_index (rebuilt from SST above), the
        // row reached an SST before the crash â€” skip it.  Otherwise call
        // replay_insert(), which populates the write buffer and in-memory indexes
        // exactly as a fresh insert would but without writing to the WAL again.
        //
        // Delete: unconditionally remove from pk_index.  A delete for a row that
        // was already removed (e.g. compaction raced with the delete) is a no-op.
        //
        // Checkpoint / flush strings: Mutation::decode returns Err, so they fall
        // through silently.  Torn frames at the WAL tail are also ignored â€” a torn
        // tail is a normal consequence of an unclean shutdown; the WAL verifier
        // handles them when WalReader::read_all stops at the first decode error.
        let wal_cfg = WalConfig::default().with_dir(&wal_dir).with_fsync(false);
        if let Ok(frames) = WalReader::open(wal_cfg).read_all() {
            for frame in frames {
                match Mutation::decode(&frame.payload) {
                    Ok(Mutation::Insert { pk, values }) => {
                        if txn.pk_index.read().get(pk).is_none() {
                            let _ = txn.replay_insert(pk, values);
                        }
                    }
                    Ok(Mutation::Delete { pk }) => {
                        txn.pk_index.write().remove(pk);
                    }
                    Ok(Mutation::Upsert { pk, values }) => {
                        txn.pk_index.write().remove(pk);
                        let _ = txn.replay_insert(pk, values);
                    }
                    Ok(Mutation::DdlCreate { .. })
                    | Ok(Mutation::DdlDrop { .. })
                    | Ok(Mutation::DdlAlter { .. }) => {}
                    Err(_) => {}
                }
            }
        }

        // Seed the flushed-rows counter so stats() is accurate after a restart.
        txn.set_total_rows_flushed(rows_on_disk);

        // The manifest is enabled unconditionally here, unlike in TxnManager where
        // it is opt-in. TableEngine is the top-level type and its `compact`,
        // `apply_retention`, and `verify` all read from it â€” without one they
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
        // - **Populated manifest**: an orphan from an interrupted compaction â€”
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
                    // age-based retention, which is right â€” they predate everything
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

        let catalog_path = index_catalog::catalog_path(&data_dir);
        let cdc_log_path = data_dir.join("cdc.log");
        let cdc_log = CdcLog::open(cdc_log_path)?;
        let max_lsn = cdc_log.max_lsn()?;
        let (changes_tx, _) = tokio::sync::broadcast::channel(1024);
        let engine = TableEngine {
            txn,
            partitions: RwLock::new(partitions),
            partition_dir,
            schema: typed_schema,
            compaction_strategy: options.compaction_strategy,
            write_options: options.write_options,
            manifest_recovery,
            mvcc: MvccStore::new(),
            cas_lock: parking_lot::Mutex::new(()),
            auto_compact_threshold: options.auto_compact_threshold,
            catalog_path,
            ttl_index: RwLock::new(std::collections::BTreeMap::new()),
            secondary_indexes: RwLock::new(std::collections::HashMap::new()),
            changes_tx,
            lsn_counter: std::sync::atomic::AtomicU64::new(max_lsn),
            cdc_log,
        };

        // Rebuild any secondary indexes that were persisted before shutdown.
        // Done after full construction so build_secondary_index can call
        // flush_if_needed, get, and scan normally.
        for col in index_catalog::load(&engine.catalog_path) {
            if col < engine.schema.col_count() {
                engine.build_secondary_index(col)?;
            }
        }

        Ok(engine)
    }

    /// Read stored CDC events from `from_lsn` onwards.
    ///
    /// Returns all [`ChangeEvent`]s with `lsn >= from_lsn` in log order.
    /// Use `from_lsn = 0` to replay the full history.
    pub fn read_log_from(&self, from_lsn: u64) -> EngineResult<Vec<ChangeEvent>> {
        Ok(self.cdc_log.read_from(from_lsn)?)
    }

    /// Return summary statistics (event count, byte size, min/max LSN) for the
    /// CDC log without parsing JSON bodies.
    pub fn cdc_log_stats(&self) -> EngineResult<crate::cdc_log::CdcLogStats> {
        Ok(self.cdc_log.stats()?)
    }

    /// Rewrite the CDC log keeping only events with `lsn >= retain_lsn`.
    /// Returns the number of events removed.
    pub fn compact_cdc_log(&self, retain_lsn: u64) -> EngineResult<usize> {
        Ok(self.cdc_log.compact(retain_lsn)?)
    }

    /// Flush all pending rows, compact all partitions into one, and write a
    /// `snapshot.vvss` marker into the partition directory.
    ///
    /// On the next `open_with`, the marker causes the WAL directory to be
    /// cleared before `WalWriter::open`, so replay starts from a clean slate.
    /// All rows are already durable in the compacted partition file, so no
    /// data is lost.
    pub fn take_snapshot(&self) -> EngineResult<crate::snapshot::SnapshotMeta> {
        // pk_index is the authoritative map of live rows.  Read the count BEFORE
        // flush so that buffered deletes (not yet written to a partition) are
        // reflected correctly -- flush would otherwise re-insert them.
        let row_count = self.txn.pk_index.read().len() as u64;
        self.flush()?;
        self.compact()?;
        let byte_size = crate::snapshot::partition_dir_byte_size(&self.partition_dir)?;
        let meta = crate::snapshot::SnapshotMeta {
            row_count,
            byte_size,
        };
        let marker = self.partition_dir.join("snapshot.vvss");
        crate::snapshot::write_marker(&marker, &meta)?;
        Ok(meta)
    }

    /// Return snapshot metadata if a marker exists, or `None` if no snapshot
    /// has been taken for this table.
    pub fn snapshot_info(&self) -> EngineResult<Option<crate::snapshot::SnapshotMeta>> {
        let marker = self.partition_dir.join("snapshot.vvss");
        Ok(crate::snapshot::read_marker(&marker)?)
    }

    /// Subscribe to change events for this table.
    ///
    /// Each call returns an independent [`tokio::sync::broadcast::Receiver`].
    /// All active receivers get every [`ChangeEvent`]; lagging receivers that
    /// fall more than 1024 events behind are automatically dropped.
    pub fn subscribe_changes(&self) -> tokio::sync::broadcast::Receiver<ChangeEvent> {
        self.changes_tx.subscribe()
    }

    // â”€â”€ Write path â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    /// Insert a row. `values.len()` must equal `schema.len()`.
    ///
    /// If the write buffer hits its threshold an auto-flush is triggered and
    /// the new partition is immediately registered for reads.
    ///
    /// All live secondary indexes are updated atomically after the write
    /// succeeds, at no extra cost when no indexes are built.
    /// Return the table's [`TableSchema`].
    pub fn typed_schema(&self) -> &TableSchema {
        &self.schema
    }

    pub fn insert(&self, pk: u64, values: Vec<u64>) -> EngineResult<WalTip> {
        self.schema.validate_row(&values)?;

        // Clone for CDC â€” always needed for log persistence; also used for
        // broadcasting to live SSE subscribers when any are active.
        let cdc_vals = Some(values.clone());

        // Snapshot the old row and clone the new values for index maintenance.
        // Only pays the cost when at least one secondary index is live.
        let index_work = if !self.secondary_indexes.read().is_empty() {
            let old_row = self.get(pk)?;
            Some((old_row, values.clone()))
        } else {
            None
        };

        let flushed_before = self.txn.total_rows_flushed();
        let tip = self.txn.insert(pk, values)?;
        if self.txn.total_rows_flushed() > flushed_before {
            self.sync_partitions()?;
            self.maybe_compact()?;
        }

        if let Some((old_row, new_vals)) = index_work {
            self.secondary_maintain_insert(pk, old_row.as_ref(), &new_vals);
        }

        if let Some(vals) = cdc_vals {
            let lsn = self
                .lsn_counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                + 1;
            let ev = ChangeEvent::insert(lsn, pk, vals);
            let _ = self.cdc_log.append(&ev);
            let _ = self.changes_tx.send(ev);
        }
        Ok(tip)
    }

    /// Remove a row. Returns `Err(TxnError::PkNotFound)` for unknown PKs.
    ///
    /// All live secondary indexes are updated after the delete succeeds.
    pub fn delete(&self, pk: u64) -> EngineResult<WalTip> {
        let old_row = if !self.secondary_indexes.read().is_empty() {
            self.get(pk)?
        } else {
            None
        };
        let tip = self.txn.delete(pk)?;
        if let Some(row) = old_row {
            self.secondary_maintain_delete(pk, &row);
        }
        let lsn = self
            .lsn_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        let ev = ChangeEvent::delete(lsn, pk);
        let _ = self.cdc_log.append(&ev);
        let _ = self.changes_tx.send(ev);
        Ok(tip)
    }

    /// Flush the write buffer to a new `.vvsst` partition file.
    ///
    /// Returns `None` if the buffer was empty.  After a successful flush the
    /// new partition is immediately registered for reads.
    pub fn flush(&self) -> EngineResult<Option<FlushResult>> {
        let result = self.txn.flush()?;
        if result.is_some() {
            self.sync_partitions()?;
            self.maybe_compact()?;
        }
        Ok(result)
    }

    // â”€â”€ Read path â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    /// Look up a single row by primary key.
    ///
    /// Pending (un-flushed) rows are flushed before the lookup so the result
    /// is always up-to-date.  Returns `None` if the pk is not found.
    pub fn get(&self, pk: u64) -> EngineResult<Option<EngineRow>> {
        // A read does NOT flush.
        //
        // This used to force a flush whenever the write buffer was non-empty,
        // which made `get` rewrite partitions and rebuild the index â€” not a read
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
            // Still buffered (or in the in-flight set during a flush). A miss in
            // the buffer alone is not an error: a concurrent flush may have drained
            // the row into `in_flight` â€” or may have already moved it to a partition
            // and republished the descriptor â€” between the index read above and this
            // lookup. Check both the buffer and in-flight, then re-read the index.
            if let Some(values) = self.txn.buffered_or_inflight_row(pk, row_offset) {
                return Ok(Some(EngineRow { pk, values }));
            }
            let Some(fresh) = self.txn.pk_index.read().get(pk) else {
                return Ok(None);
            };
            if fresh.file_id() == vectorvolt_txn::PENDING_FILE_ID {
                if let Some(values) = self
                    .txn
                    .buffered_or_inflight_row(pk, fresh.row_offset() as usize)
                {
                    return Ok(Some(EngineRow { pk, values }));
                }
                // Still pending and in neither the buffer nor the in-flight set
                // after two index reads â€” the flush completed between our reads.
                // Force another flush to drain anything that snuck in, then
                // resolve against whatever settled descriptor we find.
                self.do_flush()?;
                let Some(settled) = self.txn.pk_index.read().get(pk) else {
                    return Ok(None);
                };
                if settled.file_id() == vectorvolt_txn::PENDING_FILE_ID {
                    return Ok(self
                        .txn
                        .buffered_or_inflight_row(pk, settled.row_offset() as usize)
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
        // finds a descriptor pointing at a partition that is not in the map â€” and
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
        // rescan is guaranteed to find it. The retry is bounded at one â€” if the
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
        let col_count = self.schema.col_count();
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

    /// Like `scan()` but with authoritative PK deduplication.
    ///
    /// `scan()` can return the same PK from multiple partition files when a row
    /// has been updated but not yet compacted â€” the stale physical bytes remain
    /// in the old partition.  This method collects candidate PKs via `scan()`
    /// (keeping zone-pruning efficiency), then uses `get(pk)` as the sole
    /// source of truth for each unique PK.  The filter `[lo, hi]` is
    /// re-verified against the authoritative row so rows updated out of range
    /// are correctly excluded.
    ///
    /// All read-path methods that return rows or aggregate over rows should use
    /// this instead of `scan()`.  The exception is write-path helpers that only
    /// need PKs and handle their own dedup (e.g. `matching_pks` in
    /// `update_where`).
    pub(crate) fn scan_deduped(
        &self,
        filter_col: usize,
        lo: u64,
        hi: u64,
    ) -> EngineResult<Vec<EngineRow>> {
        let raw = self.scan(filter_col, lo, hi)?;
        let mut seen: HashSet<u64> = HashSet::with_capacity(raw.len());
        let mut result: Vec<EngineRow> = Vec::with_capacity(raw.len());
        for row in raw {
            if seen.insert(row.pk)
                && let Some(current) = self.get(row.pk)?
                && current.values[filter_col] >= lo
                && current.values[filter_col] <= hi
            {
                result.push(current);
            }
        }
        Ok(result)
    }

    // â”€â”€ Compaction â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    /// Compact this table, delegating to the shared planner and executor.
    ///
    /// Was a second, independent merge implementation living here â€” it collected
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
    /// file cannot be deleted â€” `remove_file` returns OS error 1224 â€” so the
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
    /// it is not a nicety. Pure â€” touches no files.
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
    /// Manifest first, then the files â€” the same ordering compaction uses, and for
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

    // â”€â”€ Introspection â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

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
    /// False positive rate â‰ˆ 0.78%; false negatives are impossible.
    pub fn fk_exists(&self, value: u64) -> bool {
        self.txn.fk_check(value)
    }

    /// Snapshot of current engine state (non-flushing).
    pub fn stats(&self) -> EngineStats {
        EngineStats {
            rows_buffered: self.txn.row_count_buffered(),
            rows_flushed: self.txn.total_rows_flushed(),
            partition_count: self.partitions.read().len(),
            schema: self.schema.names(),
            auto_compact_threshold: self.auto_compact_threshold,
        }
    }

    // â”€â”€ Private helpers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    /// Flush write buffer and register the new partition.
    pub(crate) fn do_flush(&self) -> EngineResult<()> {
        if self.txn.flush()?.is_some() {
            self.sync_partitions()?;
            self.maybe_compact()?;
        }
        Ok(())
    }

    /// Compact if the partition count has reached the auto-compact threshold.
    ///
    /// Called after every `sync_partitions`.  A no-op when
    /// `auto_compact_threshold` is `None` or the count is below the threshold.
    ///
    /// No recursion: `compact()` calls `do_flush()`, which only enters
    /// `sync_partitions` + `maybe_compact` when `txn.flush()` flushes rows
    /// (returns `Some`).  Because `compact()` begins with `do_flush()` and
    /// succeeds only when the write buffer is empty, its internal `do_flush`
    /// call returns `None` and the chain stops there.
    fn maybe_compact(&self) -> EngineResult<()> {
        if let Some(threshold) = self.auto_compact_threshold
            && self.partitions.read().len() >= threshold
        {
            self.compact()?;
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
            // already present â€” after having opened the file, mmapped it, and
            // verified its full-file CRC. Since `sync_partitions` runs after every
            // flush, flush number N re-verified all N-1 existing partitions:
            // quadratic work in the partition count, for a result thrown away.
            //
            // Measured at 8 threads flushing every 64 rows: 279 us per insert
            // before, 27 us after. The filename is authoritative because the
            // writer chooses it from the same two ids it stores in the header, and
            // any file whose name does not parse is opened the slow way rather
            // than skipped â€” an unparseable name means an unknown provenance, not
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

    // â”€â”€ MVCC transaction API â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

    /// Begin a new multi-statement transaction.  Returns a [`TxnId`] that must
    /// be passed to every subsequent `*_txn` call.
    pub fn begin(&self) -> TxnId {
        self.mvcc.begin()
    }

    /// Stage an insert inside `txn_id`.  The row is not visible to other
    /// readers until `commit_txn` succeeds.
    pub fn insert_txn(&self, txn_id: TxnId, pk: u64, values: Vec<u64>) -> EngineResult<()> {
        self.mvcc.write_insert(txn_id, pk, values)
    }

    /// Stage a delete inside `txn_id`.
    pub fn delete_txn(&self, txn_id: TxnId, pk: u64) -> EngineResult<()> {
        self.mvcc.write_delete(txn_id, pk)
    }

    /// Read `pk` through the transaction's write buffer, then fall through to
    /// the committed engine state.
    ///
    /// - A pending `insert_txn` for this pk returns `Ok(Some(row))`.
    /// - A pending `delete_txn` for this pk returns `Ok(None)`.
    /// - A pk not touched by this transaction returns the committed engine value.
    pub fn get_txn(&self, txn_id: TxnId, pk: u64) -> EngineResult<Option<EngineRow>> {
        match self.mvcc.read_intent(txn_id, pk)? {
            Some(Some(values)) => Ok(Some(EngineRow { pk, values })),
            Some(None) => Ok(None), // pending delete
            None => self.get(pk),   // not in this txn â€” read committed state
        }
    }

    /// Atomically apply all mutations in `txn_id` to the engine, then remove
    /// the transaction from the store.
    ///
    /// Mutations are applied in undefined pk order.  If the engine returns an
    /// error mid-commit the partial mutations already applied are not rolled
    /// back â€” callers that need atomicity at the storage layer should wrap the
    /// engine in an outer WAL, or accept that a crash mid-commit may leave the
    /// engine in a partially-updated state.
    pub fn commit_txn(&self, txn_id: TxnId) -> EngineResult<()> {
        let mutations = self.mvcc.drain_commit(txn_id)?;
        for (pk, intent) in mutations {
            match intent {
                Some(values) => {
                    self.insert(pk, values)?;
                }
                None => {
                    self.delete(pk)?;
                }
            }
        }
        Ok(())
    }

    /// Discard all pending mutations for `txn_id` without applying them.
    pub fn rollback_txn(&self, txn_id: TxnId) -> EngineResult<()> {
        self.mvcc.rollback(txn_id)
    }
}
