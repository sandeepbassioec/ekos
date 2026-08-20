//! [`TxnManager`] — WAL-backed write path with in-memory index maintenance.
//!
//! ## Write sequence (INSERT)
//!
//! 1. Serialize mutation → WAL `append()` (durability before in-memory change).
//! 2. Reserve a row_offset for this PK (buffer length before push).
//! 3. Update CuckooMap: pk → RowDescriptor(PENDING, 0, row_offset).
//! 4. Update SecondaryIndex for every non-PK column.
//! 5. Insert pk into CuckooFilter.
//! 6. Append row to WriteBuffer.
//! 7. If buffer is full, auto-flush.
//!
//! ## Flush sequence
//!
//! 1. Drain write buffer.
//! 2. Build MicroPartition + write `.vvsst` file.
//! 3. Append a WAL checkpoint frame (flush record).
//! 4. Rebuild RowDescriptors in CuckooMap → real (file_id, partition_id).
//! 5. Update ZoneMap with PK min/max for the new partition.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};

use parking_lot::{Mutex, RwLock};

use vectorvolt_index::{CuckooFilter, CuckooMap, RowDescriptor, SecondaryIndex, ZoneMap};
use vectorvolt_storage::{ColumnValues, MicroPartition, write_partition};
use vectorvolt_storage::{
    CompactionOptions, CompactionOutcome, CompactionStrategy, Liveness, Manifest, ManifestEdit,
    ManifestRecovery, PartitionMeta, RowLocation, WriteOptions,
};
pub use vectorvolt_wal::WalTip;
use vectorvolt_wal::{SyncCommitter, SyncStats, WalWriter};

use crate::shard_writer::ShardWriter;
use crate::{TxnError, TxnResult, ops::Mutation};

// Sentinel file_id for rows still in the write buffer.
/// `file_id` marking a descriptor that points into the write buffer rather than
/// a partition file.
pub const PENDING_FILE_ID: u16 = u16::MAX;

/// Metadata returned after a successful partition flush.
#[derive(Debug, Clone)]
pub struct FlushResult {
    pub file_id: u16,
    pub partition_id: u16,
    pub row_count: usize,
    pub path: PathBuf,
    pub wal_tip: WalTip,
}

/// WAL-backed transaction manager for a single table.
pub struct TxnManager {
    wal: SyncCommitter,
    pub pk_index: Arc<RwLock<CuckooMap>>,
    pub secondary_index: RwLock<SecondaryIndex>,
    pub zone_map: RwLock<ZoneMap>,
    pub fk_filter: RwLock<CuckooFilter>,
    /// Per-shard writer thread (VV-TXN-009).
    ///
    /// Owns the ring buffer outright — no mutex wraps the push or drain path.
    /// All write-side operations are serialised through a single OS thread,
    /// making insert a true SPSC (single-producer, single-consumer) operation.
    writer: ShardWriter,
    flush_dir: PathBuf,
    pub schema: Vec<String>,
    next_file_id: AtomicU16,
    next_partition_id: AtomicU16,
    total_rows_flushed: AtomicU64,
    /// Flush the WAL out of its userspace buffer on every commit.
    sync_on_commit: bool,
    /// Durable record of which partitions exist, once enabled.
    ///
    /// `Option` because [`TxnManager::new`] is infallible and opening a manifest
    /// is I/O that can fail. Making the constructor fallible would break every
    /// caller to add a feature most of them do not use yet, so it is opted into
    /// via [`TxnManager::enable_manifest`], which surfaces both the error and
    /// what recovery had to discard.
    manifest: Mutex<Option<Manifest>>,
    /// Tombstone counts not yet written to the manifest, per partition.
    ///
    /// Deletes are frequent and a manifest edit costs an `fsync`, so writing one
    /// per delete would put a disk sync on the delete path. They are accumulated
    /// here and persisted in a batch at the next flush or compaction.
    ///
    /// **Losing these on a crash is acceptable and deliberate.** A tombstone count
    /// only decides how eagerly compaction reclaims space; a lost count makes
    /// compaction lazier, never incorrect. The PK index — which decides what is
    /// actually visible — is rebuilt from the WAL, and that is the durable record
    /// that matters.
    pending_tombstones: Mutex<HashMap<(u16, u16), u32>>,
    /// Rows drained from the write buffer but not yet republished in the index.
    ///
    /// Exists so a flush does not have to hold the write-buffer lock across file
    /// I/O. A row must always be findable — buffered, in flight, or in a partition
    /// — and this is the middle state. Reads consult it after the buffer.
    ///
    /// Shared with the writer thread so it can populate `in_flight` atomically
    /// inside its Drain handler before sending the reply. That ordering means any
    /// subsequent RowFor (which arrives after the reply) always sees the row in
    /// `in_flight`, closing the TOCTOU window the old Mutex path had.
    in_flight: Arc<Mutex<HashMap<u64, Vec<u64>>>>,
}

impl TxnManager {
    pub fn new(
        wal: WalWriter,
        schema: Vec<String>,
        flush_dir: impl Into<PathBuf>,
        flush_threshold: usize,
    ) -> Self {
        Self::with_starting_partition(wal, schema, flush_dir, flush_threshold, 0)
    }

    /// Like [`new`] but starts the partition counter at `start_partition_id`.
    /// Use this when re-opening a table that already has flushed partitions on
    /// disk — pass `max_existing_partition_id + 1` to avoid overwriting them.
    pub fn with_starting_partition(
        wal: WalWriter,
        schema: Vec<String>,
        flush_dir: impl Into<PathBuf>,
        flush_threshold: usize,
        start_partition_id: u16,
    ) -> Self {
        let schema_width = schema.len();
        let pk_index = Arc::new(RwLock::new(CuckooMap::with_capacity(64)));
        let in_flight = Arc::new(Mutex::new(HashMap::new()));
        let writer = ShardWriter::new(
            flush_threshold,
            schema_width,
            Arc::clone(&pk_index),
            Arc::clone(&in_flight),
        );
        TxnManager {
            wal: SyncCommitter::new(wal),
            pk_index,
            secondary_index: RwLock::new(SecondaryIndex::new()),
            zone_map: RwLock::new(ZoneMap::new()),
            fk_filter: RwLock::new(CuckooFilter::with_capacity(64)),
            writer,
            flush_dir: flush_dir.into(),
            schema,
            next_file_id: AtomicU16::new(0),
            next_partition_id: AtomicU16::new(start_partition_id),
            total_rows_flushed: AtomicU64::new(0),
            // Default off preserves the historical behaviour for direct users of
            // this crate; TableEngine opts in per its Durability setting.
            sync_on_commit: false,
            manifest: Mutex::new(None),
            pending_tombstones: Mutex::new(HashMap::new()),
            in_flight,
        }
    }

    /// Flush the WAL out of its `BufWriter` after every mutation.
    ///
    /// Without this, `wal.append()` only copies into userspace memory — nothing
    /// reaches the OS until the buffer fills or the writer drops, so a process
    /// crash loses recent writes. Whether the flush also calls `fsync` is set by
    /// [`vectorvolt_wal::WalConfig::with_fsync`].
    pub fn with_sync_on_commit(mut self, sync: bool) -> Self {
        self.sync_on_commit = sync;
        self
    }

    /// Append one WAL record, honouring the configured durability.
    ///
    /// When syncing, this goes through [`SyncCommitter`], so concurrent writers
    /// share one `fsync` instead of each paying their own. On the dev laptop an
    /// `fsync` costs 526 us against 7.8 us for the same insert unsynced — 67x — so
    /// amortising it across writers is the whole game.
    ///
    /// A *single* writer still pays one sync per commit; there is nobody to batch
    /// with. Group commit removes the scaling penalty, not the device latency.
    #[inline]
    fn commit(&self, payload: Vec<u8>) -> TxnResult<WalTip> {
        if self.sync_on_commit {
            Ok(self.wal.append_and_sync(payload)?)
        } else {
            Ok(self.wal.append(payload)?)
        }
    }

    /// Group-commit counters: commits, `fsync`s issued, and commits served by
    /// another thread's sync. `commits_per_sync() > 1` means batching happened.
    pub fn commit_stats(&self) -> SyncStats {
        self.wal.stats()
    }

    // ── INSERT ─────────────────────────────────────────────────────────────────

    /// Insert a new row. `values.len()` must equal `schema.len()`.
    ///
    /// Returns the WAL tip after the committed frame. If the write buffer hits
    /// its flush threshold after this insert, an automatic flush is triggered.
    pub fn insert(&self, pk: u64, values: Vec<u64>) -> TxnResult<WalTip> {
        if values.len() != self.schema.len() {
            return Err(TxnError::SchemaMismatch {
                expected: self.schema.len(),
                got: values.len(),
            });
        }

        // 1. WAL first.
        let mutation = Mutation::Insert {
            pk,
            values: values.clone(),
        };
        let tip = self.commit(mutation.encode())?;

        // 2. SecondaryIndex: one entry per non-PK column. Done before the row is
        //    buffered, while `values` is still borrowable.
        {
            let mut sec = self.secondary_index.write();
            for (col_idx, &val) in values.iter().enumerate().skip(1) {
                sec.insert(col_value_hash(col_idx, val), pk as u32);
            }
        }

        // 3. CuckooFilter.
        self.fk_filter.write().insert(pk)?;

        // 4-5. Delegate buffer offset, index entry, and push to the writer thread.
        //
        // The writer thread is the single producer for the ring buffer. It computes
        // the offset, writes the pk_index entry (BEFORE the push), and then pushes
        // the row — all as one atomic sequence with no other thread racing on these
        // three operations. This eliminates the ABBA-style concurrency bug the old
        // Mutex path had when two threads interleaved their read-offset / push steps.
        let full = self.writer.insert(pk, values)?;

        // 6. Auto-flush if the buffer filled.
        if full {
            self.flush()?;
        }

        Ok(tip)
    }

    // ── INSERT BATCH ──────────────────────────────────────────────────────────

    /// Insert multiple rows as a single WAL record.
    ///
    /// All rows are encoded together into one `InsertBatch` WAL frame and
    /// committed in a single `commit()` call. Each row is then applied to the
    /// in-memory indexes and write buffer individually.
    ///
    /// Returns the WAL tip after the batch frame. Errors if any row's value
    /// count does not match the schema, or if WAL commit fails; in either case
    /// no rows from this batch are guaranteed to be visible.
    pub fn insert_batch(&self, rows: Vec<(u64, Vec<u64>)>) -> TxnResult<WalTip> {
        for (i, (_, values)) in rows.iter().enumerate() {
            if values.len() != self.schema.len() {
                return Err(TxnError::SchemaMismatch {
                    expected: self.schema.len(),
                    got: values.len(),
                });
            }
            let _ = i;
        }

        // Single WAL commit for the entire batch.
        let mutation = Mutation::InsertBatch { rows: rows.clone() };
        let tip = self.commit(mutation.encode())?;

        // Apply each row to in-memory structures.
        let mut any_full = false;
        for (pk, values) in rows {
            {
                let mut sec = self.secondary_index.write();
                for (col_idx, &val) in values.iter().enumerate().skip(1) {
                    sec.insert(col_value_hash(col_idx, val), pk as u32);
                }
            }
            self.fk_filter.write().insert(pk)?;
            let full = self.writer.insert(pk, values)?;
            if full {
                any_full = true;
            }
        }

        if any_full {
            self.flush()?;
        }

        Ok(tip)
    }

    // ── DELETE ─────────────────────────────────────────────────────────────────

    /// All PKs whose column `col_idx` matches `value` via the SecondaryIndex.
    ///
    /// The index stores `pk as u32`; PKs > `u32::MAX` are not reachable this
    /// way (pre-existing limitation of the index storage format).
    /// Deleted rows remain in the index until compaction; callers that need a
    /// live view should verify each returned PK with `pk_index`.
    pub fn secondary_lookup_pks(&self, col_idx: usize, value: u64) -> Vec<u64> {
        let hash = col_value_hash(col_idx, value);
        self.secondary_index
            .read()
            .get_vec(hash)
            .into_iter()
            .map(|id| id as u64)
            .collect()
    }

    /// Probabilistic FK existence check. Returns `true` when `value` was ever
    /// inserted. False positive rate ≈ 0.78%; false negatives are impossible.
    pub fn fk_check(&self, value: u64) -> bool {
        self.fk_filter.read().contains(value)
    }

    /// Remove a row from the in-memory index. WAL-logged.
    ///
    /// Returns `Err(PkNotFound)` if the pk does not exist in the CuckooMap.
    /// Does not remove the row from any flushed `.vvsst` file — that requires
    /// compaction (deferred to a later phase).
    pub fn delete(&self, pk: u64) -> TxnResult<WalTip> {
        // The descriptor is captured BEFORE the row leaves the index, because it
        // is the only record of which partition the dead row lives in. Without it
        // compaction has no way to know a partition is mostly garbage, and
        // garbage-driven collection can never fire.
        let Some(desc) = self.pk_index.read().get(pk) else {
            return Err(TxnError::PkNotFound(pk));
        };
        let home = (desc.file_id(), desc.partition_id());

        let tip = self.commit(Mutation::Delete { pk }.encode())?;
        self.pk_index.write().remove(pk);
        *self.pending_tombstones.lock().entry(home).or_insert(0) += 1;
        Ok(tip)
    }

    // ── DELETE BATCH ──────────────────────────────────────────────────────────

    /// Delete multiple rows as a single WAL record.
    ///
    /// All PKs are encoded together into one `DeleteBatch` WAL frame and
    /// committed in a single `commit()` call. Each PK is then removed from
    /// the in-memory pk_index individually.
    ///
    /// Returns `Err(PkNotFound)` for the first PK that is not in the index.
    /// On error, no rows from the batch are removed (all-or-nothing validation
    /// happens before the WAL commit).
    pub fn delete_batch(&self, pks: Vec<u64>) -> TxnResult<WalTip> {
        // Validate all PKs exist and capture tombstone homes before mutating.
        let mut homes = Vec::with_capacity(pks.len());
        {
            let idx = self.pk_index.read();
            for &pk in &pks {
                let desc = idx.get(pk).ok_or(TxnError::PkNotFound(pk))?;
                homes.push((pk, (desc.file_id(), desc.partition_id())));
            }
        }

        // Single WAL commit for the entire batch.
        let tip = self.commit(Mutation::DeleteBatch { pks: pks.clone() }.encode())?;

        // Remove each PK from the index and accumulate tombstones.
        {
            let mut idx = self.pk_index.write();
            let mut tombstones = self.pending_tombstones.lock();
            for (pk, home) in homes {
                idx.remove(pk);
                *tombstones.entry(home).or_insert(0) += 1;
            }
        }

        Ok(tip)
    }

    // ── UPSERT ─────────────────────────────────────────────────────────────────

    /// Insert or overwrite a row.  WAL-logged as a single `Mutation::Upsert`.
    ///
    /// If `pk` already exists in the index, the old index entry is removed
    /// before the new row is pushed (equivalent to delete + insert, but with a
    /// single WAL record).  If `pk` is absent, behaves identically to `insert`.
    pub fn upsert(&self, pk: u64, values: Vec<u64>) -> TxnResult<WalTip> {
        if values.len() != self.schema.len() {
            return Err(TxnError::SchemaMismatch {
                expected: self.schema.len(),
                got: values.len(),
            });
        }

        // WAL first — a single Upsert record covers both the remove and the push.
        let mutation = Mutation::Upsert {
            pk,
            values: values.clone(),
        };
        let tip = self.commit(mutation.encode())?;

        // Remove stale index entry if the pk already exists.
        if self.pk_index.read().get(pk).is_some() {
            self.pk_index.write().remove(pk);
        }

        // Secondary index + filter.
        {
            let mut sec = self.secondary_index.write();
            for (col_idx, &val) in values.iter().enumerate().skip(1) {
                sec.insert(col_value_hash(col_idx, val), pk as u32);
            }
        }
        self.fk_filter.write().insert(pk)?;

        // Push to the writer thread; auto-flush on threshold.
        let full = self.writer.insert(pk, values)?;
        if full {
            self.flush()?;
        }

        Ok(tip)
    }

    // ── FLUSH ──────────────────────────────────────────────────────────────────

    /// Flush the write buffer to a `.vvsst` partition file.
    ///
    /// Returns `None` if the buffer was empty. On success, updates the
    /// CuckooMap so all flushed PKs point to the real (file_id, partition_id),
    /// and records the PK zone [min_pk, max_pk] in the ZoneMap.
    /// Open (or create) the partition manifest in the flush directory.
    ///
    /// Returns what recovery found. A non-clean [`ManifestRecovery`] means bytes
    /// were discarded — see its docs; a small count is a torn tail from an
    /// unclean shutdown, a large one suggests real corruption. It is returned
    /// rather than logged so a caller can decide which it is.
    pub fn enable_manifest(&self) -> TxnResult<ManifestRecovery> {
        std::fs::create_dir_all(&self.flush_dir)?;
        let (m, recovery) = Manifest::open(self.flush_dir.join("MANIFEST"))?;
        *self.manifest.lock() = Some(m);
        Ok(recovery)
    }

    /// Every partition the manifest knows about, or empty if it is not enabled.
    ///
    /// This is the input to `CompactionStrategy::plan`, `RetentionPolicy::plan`,
    /// and `verify_partitions`.
    pub fn partitions(&self) -> Vec<PartitionMeta> {
        let mut parts: Vec<PartitionMeta> = self
            .manifest
            .lock()
            .as_ref()
            .map(|m| m.partitions())
            .unwrap_or_default();

        // Overlay tombstones not yet persisted. A planner asking what to compact
        // must see deletes that happened seconds ago, not only those that survived
        // a flush — otherwise a table that is deleted-then-compacted reclaims
        // nothing, which is exactly the case garbage collection exists for.
        let pending = self.pending_tombstones.lock();
        if !pending.is_empty() {
            for p in &mut parts {
                if let Some(extra) = pending.get(&(p.file_id, p.partition_id)) {
                    // Saturating and clamped to the row count: a partition cannot
                    // have more dead rows than rows, and a count above 1.0 would
                    // make `garbage_ratio` nonsensical.
                    p.tombstones = p.tombstones.saturating_add(*extra).min(p.row_count);
                }
            }
        }
        parts
    }

    /// Write accumulated tombstone counts into the manifest.
    ///
    /// Called at flush and before compaction. Batched rather than per-delete
    /// because each manifest edit is an `fsync`.
    fn persist_tombstones(&self) -> TxnResult<()> {
        let drained: Vec<((u16, u16), u32)> = {
            let mut pending = self.pending_tombstones.lock();
            if pending.is_empty() {
                return Ok(());
            }
            pending.drain().collect()
        };
        for ((file_id, partition_id), extra) in drained {
            let current = self.manifest.lock().as_ref().and_then(|m| {
                m.get(file_id, partition_id)
                    .map(|p| (p.tombstones, p.row_count))
            });
            // A partition that no longer exists (compacted away) has nothing to
            // record. Dropping the count is correct: its rows are gone.
            let Some((existing, row_count)) = current else {
                continue;
            };
            self.record(ManifestEdit::SetTombstones {
                file_id,
                partition_id,
                tombstones: existing.saturating_add(extra).min(row_count),
            })?;
        }
        Ok(())
    }

    /// Apply an edit to the manifest, if one is enabled.
    ///
    /// `Ok(false)` when no manifest is open — a caller updating statistics should
    /// not have to care, but should be able to tell.
    pub fn record(&self, edit: ManifestEdit) -> TxnResult<bool> {
        match self.manifest.lock().as_mut() {
            Some(m) => {
                m.apply(edit)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    /// Plan and run compaction, returning what each job did.
    ///
    /// The full loop: plan from the manifest, merge, swap the manifest, delete the
    /// inputs. Requires a manifest — without one there is no level, size, or
    /// tombstone information to plan from, so this returns an empty vector rather
    /// than guessing.
    ///
    /// ## Ordering, and why it is this way
    ///
    /// Per job: **write the merged file, then swap the manifest, then delete the
    /// inputs.** Each step is durable before the next begins, so every crash point
    /// leaves a recoverable state:
    ///
    /// - crash after the write: an orphan file, invisible to the manifest;
    /// - crash after the swap: orphan *inputs*, also invisible;
    /// - crash after the deletes: nothing to clean up.
    ///
    /// Every failure leaks disk. None loses a row, and none duplicates one. The
    /// swap is a single [`ManifestEdit::Replace`] record so it cannot tear halfway
    /// and leave both the inputs and the output live at once.
    ///
    /// A failed job does not abort the run — later jobs are independent, and one
    /// unreadable partition should not stop the rest of compaction. Failures are
    /// returned so the caller can see them.
    pub fn run_compaction(
        &self,
        strategy: CompactionStrategy,
        write_options: WriteOptions,
    ) -> TxnResult<Vec<CompactionOutcome>> {
        // Flush pending tombstone counts first, so the plan is computed from a
        // manifest that reflects every delete so far.
        self.persist_tombstones()?;

        let partitions = self.partitions();
        if partitions.is_empty() {
            return Ok(Vec::new());
        }

        // A WORM-locked partition must not be rewritten: the hold applies to those
        // bytes, and compaction destroys them. Filtered before planning rather
        // than after, so a locked partition is never even considered.
        let eligible: Vec<PartitionMeta> = partitions
            .iter()
            .filter(|p| !p.worm_locked)
            .cloned()
            .collect();
        let jobs = strategy.plan(&eligible);

        let mut outcomes = Vec::new();
        for job in jobs {
            let file_id = self.next_file_id.load(Ordering::Relaxed);
            let partition_id = self.next_partition_id.fetch_add(1, Ordering::Relaxed);
            let opts = CompactionOptions {
                output_dir: self.flush_dir.clone(),
                output_file_id: file_id,
                output_partition_id: partition_id,
                write_options,
                now_millis: now_millis(),
            };

            let liveness = IndexLiveness { txn: self };
            let outcome = vectorvolt_storage::execute_job(&job, &self.schema, &liveness, &opts)?;

            // Re-point the PK index at the merged partition BEFORE the manifest
            // swap makes it the live one.
            //
            // Compaction moves every surviving row to a new
            // (file_id, partition_id, row_offset). An index still holding the old
            // coordinates addresses rows in a file that is about to be deleted —
            // so every point lookup for a compacted key would fail or, worse, read
            // a stale offset in a recycled id. This is the step whose absence makes
            // compaction silently destructive rather than merely wasteful.
            {
                let mut idx = self.pk_index.write();
                for (row_offset, pk) in outcome.output_pks.iter().enumerate() {
                    // Same reasoning as the flush path: a key that fails to
                    // re-index after compaction points at partitions that are
                    // about to be deleted. Silently dropping it loses the row.
                    idx.insert(
                        *pk,
                        RowDescriptor::new(file_id, partition_id, row_offset as u32),
                    )?;
                }
            }
            // The merged partition's key range, so range scans can still prune it.
            if !outcome.output_pks.is_empty() {
                self.zone_map.write().set(
                    partition_id,
                    outcome.output.min_key,
                    outcome.output.max_key,
                );
            }

            // Manifest swap: one record, so recovery sees the old set or the new
            // set and never both.
            self.record(ManifestEdit::Replace {
                adds: vec![outcome.output.clone()],
                removes: outcome.removed.clone(),
            })?;

            // The merged output has no dead rows by construction, and the inputs
            // no longer exist. Any tombstone count that accumulated for them
            // during the merge would otherwise be attributed to a partition id
            // that may later be reused.
            {
                let mut pending = self.pending_tombstones.lock();
                for key in &outcome.removed {
                    pending.remove(key);
                }
            }

            // Only now are the inputs safe to remove. A failure here is not
            // fatal — the manifest no longer names them, so they are already
            // invisible to reads — but it IS reported, because an undeleted input
            // is leaked disk and a leak nobody is told about is one nobody cleans
            // up. On Windows this is routine: a live mmap holds the file and
            // `remove_file` returns OS error 1224 until every reader is dropped.
            let mut outcome = outcome;
            for path in outcome.input_paths.clone() {
                if std::fs::remove_file(&path).is_err() && path.exists() {
                    outcome.undeleted.push(path);
                }
            }
            outcomes.push(outcome);
        }
        Ok(outcomes)
    }

    pub fn flush(&self) -> TxnResult<Option<FlushResult>> {
        // A row must ALWAYS be findable: buffered, in flight, or in a partition.
        //
        // The first version of this fix held the write-buffer lock across the
        // whole flush, including the file write. Correct, and measured at 311 us
        // per insert with 8 threads flushing every 64 rows — 28x the unflushed
        // case, because every writer blocked on file I/O.
        //
        // Instead the drained rows move to `in_flight` while the buffer lock is
        // still held, so there is no instant at which a row is in neither place,
        // and the lock is then released before any I/O. Reads consult the buffer
        // and then in-flight.
        //
        // The clone is the price: a flush batch exists twice until it is
        // published. Bounded by the flush threshold and cheap next to the file
        // write it overlaps with.
        //
        // The writer thread drains the ring buffer and populates `in_flight`
        // atomically before sending the reply — so by the time `drain()` returns
        // here, any concurrent reader with a PENDING descriptor will already find
        // the row in `in_flight`. No additional lock is needed.
        let rows = self.writer.drain()?;
        if rows.is_empty() {
            return Ok(None);
        }

        // Persist tombstone counts alongside the flush — a natural batching point,
        // and it keeps the manifest close to reality without an fsync per delete.
        self.persist_tombstones()?;

        let file_id = self.next_file_id.load(Ordering::Relaxed);
        let partition_id = self.next_partition_id.fetch_add(1, Ordering::Relaxed);
        let row_count = rows.len();

        // Build MicroPartition.
        let mut mp = MicroPartition::new(file_id, partition_id);
        for (col_idx, name) in self.schema.iter().enumerate() {
            let col_vals: Vec<u64> = rows.iter().map(|(_, vals)| vals[col_idx]).collect();
            let encoded = if col_idx == 0 {
                ColumnValues::Raw(col_vals)
            } else {
                ColumnValues::auto_encode(col_vals)
            };
            mp.push_column(name.clone(), encoded)?;
        }

        // Write .vvsst file — via a temp path, renamed into place atomically.
        //
        // The directory is created here rather than assumed. Previously a manager
        // pointed at a non-existent flush_dir failed at flush time with a bare
        // NotFound, and once enable_manifest started calling create_dir_all,
        // whether flush worked at all depended on whether a manifest was enabled —
        // an unrelated setting silently changing a core operation.
        //
        // The temp-then-rename pattern closes the race with `sync_partitions`.
        // That function opens every `.vvsst` file it finds in the directory.
        // `File::create` makes the file visible to `read_dir` IMMEDIATELY — before
        // `write_partition` has written even the magic bytes. A concurrent
        // `sync_partitions` call (triggered by another thread's completed flush)
        // would therefore open the half-empty file and return `InvalidMagic`.
        // Writing to `{filename}.tmp` first and renaming atomically into the final
        // name means a `.vvsst` file only appears in the directory once it is
        // fully written.  `sync_partitions` skips `.tmp` files (wrong extension).
        std::fs::create_dir_all(&self.flush_dir)?;
        let filename = format!("{file_id:05}_{partition_id:05}.vvsst");
        let tmp_filename = format!("{file_id:05}_{partition_id:05}.vvsst.tmp");
        let path = self.flush_dir.join(&filename);
        let tmp_path = self.flush_dir.join(&tmp_filename);
        {
            let mut file = std::fs::File::create(&tmp_path)?;
            write_partition(&mut file, &mp)?;
        } // file closed before rename — kernel buffer visible to other opens
        std::fs::rename(&tmp_path, &path)?;

        // WAL checkpoint record.
        let ckpt = format!(
            "flush file_id={file_id} partition_id={partition_id} rows={row_count} path={filename}"
        );
        let tip = self.commit(ckpt.into_bytes())?;

        // Rebuild RowDescriptors: pending → real coordinates.
        let base = self
            .total_rows_flushed
            .fetch_add(row_count as u64, Ordering::Relaxed);
        {
            let mut idx = self.pk_index.write();
            for (off, (pk, _)) in rows.iter().enumerate() {
                // Propagated, not discarded. `CuckooMap::insert` can fail after
                // exhausting its grow retries, and `let _ =` here meant the key
                // silently left the index while its row sat in a partition on
                // disk — the row is durable and permanently unreachable, which is
                // indistinguishable from data loss to anyone reading.
                //
                // A flush that cannot index what it wrote has not succeeded, so it
                // reports that rather than returning Ok.
                idx.insert(*pk, RowDescriptor::new(file_id, partition_id, off as u32))?;
            }
        }

        // Every drained row now has a real descriptor, so the in-flight copies can
        // go. Done after the index rewrite, never before: dropping them earlier
        // would reopen the window this exists to close.
        {
            let mut inf = self.in_flight.lock();
            for (pk, _) in &rows {
                inf.remove(pk);
            }
        }

        // ZoneMap: record PK min/max for this partition.
        let pk_min = rows.iter().map(|(pk, _)| *pk).min().unwrap_or(0);
        let pk_max = rows.iter().map(|(pk, _)| *pk).max().unwrap_or(0);
        self.zone_map.write().set(partition_id, pk_min, pk_max);

        // Manifest: record the partition so compaction planning, retention, and
        // verification can see it. Written AFTER the .vvsst file is on disk and
        // after the WAL checkpoint, so the manifest can never name a partition
        // that does not exist. The reverse — a file with no manifest entry — is
        // survivable: it leaks disk until a scan for orphans finds it, whereas a
        // manifest entry pointing at nothing breaks every read that follows it.
        if let Some(m) = self.manifest.lock().as_mut() {
            let size_bytes = std::fs::metadata(&path).map(|md| md.len()).unwrap_or(0);
            m.apply(ManifestEdit::Add(Box::new(PartitionMeta {
                file_id,
                partition_id,
                path: path.clone(),
                level: 0,
                size_bytes,
                row_count: row_count as u32,
                created_at_millis: now_millis(),
                min_key: pk_min,
                max_key: pk_max,
                tombstones: 0,
                worm_locked: false,
            })))?;
        }

        let _ = base; // used by fetch_add above

        Ok(Some(FlushResult {
            file_id,
            partition_id,
            row_count,
            path,
            wal_tip: tip,
        }))
    }

    // ── Recovery ───────────────────────────────────────────────────────────────

    /// Re-apply a WAL Insert mutation during crash recovery WITHOUT writing to
    /// the WAL again.
    ///
    /// This is the WAL replay path: called by `TableEngine::open_with` for every
    /// Insert frame whose PK is not yet reflected in a partition file. It does
    /// everything `insert()` does except step 1 (WAL append), so recovered rows
    /// are immediately readable via `get()` and reachable via the secondary index.
    ///
    /// The caller is responsible for the existence check: only call this when
    /// `pk_index.get(pk).is_none()`, because `replay_insert` will insert a
    /// PENDING descriptor and fail if the pk is already mapped.
    pub fn replay_insert(&self, pk: u64, values: Vec<u64>) -> TxnResult<()> {
        if values.len() != self.schema.len() {
            return Err(TxnError::SchemaMismatch {
                expected: self.schema.len(),
                got: values.len(),
            });
        }

        // SecondaryIndex — one entry per non-PK column.
        {
            let mut sec = self.secondary_index.write();
            for (col_idx, &val) in values.iter().enumerate().skip(1) {
                sec.insert(col_value_hash(col_idx, val), pk as u32);
            }
        }

        // CuckooFilter.
        self.fk_filter.write().insert(pk)?;

        // Delegate to the writer thread — same atomicity invariant as `insert()`.
        let full = self.writer.insert(pk, values)?;

        if full {
            self.flush()?;
        }

        Ok(())
    }

    /// Rebuild the pk_index and CuckooFilter from a set of already-loaded
    /// PartitionReaders.  Call this after `with_starting_partition` when
    /// re-opening a table that has existing `.vvsst` files on disk so that
    /// `get()` can find rows from previous sessions.
    pub fn rebuild_pk_index_from_partitions(
        &self,
        partitions: &std::collections::HashMap<(u16, u16), vectorvolt_storage::PartitionReader>,
    ) -> TxnResult<()> {
        let mut idx = self.pk_index.write();
        let mut flt = self.fk_filter.write();
        for ((file_id, partition_id), reader) in partitions {
            let pk_col = reader.read_column(0).map_err(TxnError::Storage)?;
            let pks = pk_col.to_raw_vec();
            for (offset, &pk) in pks.iter().enumerate() {
                idx.insert(
                    pk,
                    RowDescriptor::new(*file_id, *partition_id, offset as u32),
                )?;
                flt.insert(pk)?;
            }
        }
        Ok(())
    }

    // ── Accessors ──────────────────────────────────────────────────────────────

    /// A row still sitting in the write buffer, by the offset its descriptor
    /// recorded.
    ///
    /// Lets a reader resolve a PENDING descriptor without forcing a flush. See
    /// `WriteBuffer::row_for` for why the pk is re-checked.
    pub fn buffered_row(&self, pk: u64, offset: usize) -> Option<Vec<u64>> {
        self.writer.row_for(pk, offset)
    }

    /// Check the write buffer first, then the in-flight set.
    ///
    /// Rows drained from the buffer during a flush live in `in_flight` until
    /// their pk_index entries are rewritten from PENDING to real coordinates.
    /// During that window `buffered_row` misses them; this method covers both
    /// places so a reader with a PENDING descriptor never falsely reports the
    /// row absent.
    pub fn buffered_or_inflight_row(&self, pk: u64, offset: usize) -> Option<Vec<u64>> {
        if let Some(vals) = self.writer.row_for(pk, offset) {
            return Some(vals);
        }
        self.in_flight.lock().get(&pk).cloned()
    }

    pub fn row_count_buffered(&self) -> usize {
        self.writer.len()
    }

    pub fn total_rows_flushed(&self) -> u64 {
        self.total_rows_flushed.load(Ordering::Relaxed)
    }

    /// Seed the flushed-rows counter (used after re-open to reflect on-disk state).
    pub fn set_total_rows_flushed(&self, count: u64) {
        self.total_rows_flushed.store(count, Ordering::Relaxed);
    }

    /// Allocate and return the next partition ID, advancing the counter.
    pub fn alloc_partition_id(&self) -> u16 {
        self.next_partition_id.fetch_add(1, Ordering::Relaxed)
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Stable per-column value hash for SecondaryIndex bucket keys.
#[inline]
fn col_value_hash(col_idx: usize, value: u64) -> u32 {
    let mixed = value
        .wrapping_add(col_idx as u64)
        .wrapping_mul(0x517c_c1b7_2722_0a95);
    (mixed ^ (mixed >> 32)) as u32
}

/// Wall-clock milliseconds since the Unix epoch.
///
/// Saturates to 0 if the system clock is before the epoch rather than panicking.
/// A nonsense clock should not take down a flush, and retention already treats a
/// nonsensical timestamp safely (see `RetentionPolicy::plan`).
fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Liveness backed by the PK index.
///
/// A row is live only if the index still points at *that exact location*. That
/// covers both failure modes in one check: a deleted row has no index entry, and a
/// superseded row has an entry pointing somewhere newer. Comparing only the PK
/// would keep stale copies alive forever.
struct IndexLiveness<'a> {
    txn: &'a TxnManager,
}

impl Liveness for IndexLiveness<'_> {
    fn is_live(&self, pk: u64, at: RowLocation) -> bool {
        match self.txn.pk_index.read().get(pk) {
            Some(desc) => {
                desc.file_id() == at.file_id
                    && desc.partition_id() == at.partition_id
                    && desc.row_offset() == at.row_offset
            }
            None => false,
        }
    }
}
