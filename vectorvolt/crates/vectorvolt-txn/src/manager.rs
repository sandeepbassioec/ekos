//! [`TxnManager`] — WAL-backed write path with in-memory index maintenance.
//!
//! ## Write sequence (INSERT)
//!
//! 0. Lock the pk's write stripe (held through step 6, PP-23).
//! 1. Serialize mutation → WAL `append()` (durability before in-memory change);
//!    the frame's sequence number is the mutation's MVCC commit timestamp.
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

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU64, Ordering};

use parking_lot::{Mutex, MutexGuard, RwLock};

use vectorvolt_index::{CuckooFilter, CuckooMap, RowDescriptor, SecondaryIndex, ZoneMap};
use vectorvolt_storage::{ColumnValues, MicroPartition, PartitionReader, write_partition_keyed};
use vectorvolt_storage::{
    CompactionOptions, CompactionOutcome, CompactionStrategy, Liveness, Manifest, ManifestEdit,
    ManifestRecovery, PartitionMeta, RowLocation, WriteOptions,
};
pub use vectorvolt_wal::WalTip;
use vectorvolt_wal::{SyncCommitter, SyncStats, WalWriter};

use crate::mvcc::{CommitClock, VersionEntry, VersionIndex};
use crate::routing::select_partition;
use crate::shard_writer::{DrainedRow, ShardWriter};
use crate::{TxnError, TxnResult, ops::Mutation};

// Sentinel file_id for rows still in the write buffer.
/// `file_id` marking a descriptor that points into the write buffer rather than
/// a partition file.
pub const PENDING_FILE_ID: u16 = u16::MAX;

/// Number of per-pk write stripes (PP-23). A power of two; pks hash onto one.
const WRITE_STRIPES: usize = 64;

/// Test hook run by [`TxnManager::run_compaction`] after the merged outputs are
/// built and before they are installed.
pub type CompactionHook = Arc<dyn Fn() + Send + Sync>;

/// Compaction outputs built but not yet installed.
///
/// Returned by [`TxnManager::prepare_compaction`], which does the expensive part
/// (reading the inputs and writing the merged files) without blocking readers
/// or writers. Holds the manager's compaction lock, so two compactions never
/// plan the same inputs; hand it to [`TxnManager::install_compaction`] to swap
/// the outputs in. Dropping it without installing leaves orphan output files,
/// which the manifest never names (a disk leak, never data loss).
pub struct PreparedCompaction<'a> {
    _serial: MutexGuard<'a, ()>,
    built: Vec<CompactionOutcome>,
}

impl PreparedCompaction<'_> {
    /// Number of merged outputs waiting to be installed.
    pub fn len(&self) -> usize {
        self.built.len()
    }

    /// Whether the planner found nothing to do.
    pub fn is_empty(&self) -> bool {
        self.built.is_empty()
    }
}

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
    /// Number of logical partitions rows are routed across on flush.
    ///
    /// `select_partition(pk, partition_count)` determines which partition file
    /// a row lands in. `1` (the default) preserves the original single-file
    /// flush behaviour. Higher values distribute rows across `partition_count`
    /// separate `.vvsst` files per flush cycle, each with a unique `partition_id`
    /// in the RowDescriptor.
    partition_count: usize,
    /// MVCC Phase 1: monotonic commit-timestamp source. One per table, shared
    /// by every write path so commit order and version order agree.
    clock: CommitClock,
    /// MVCC Phase 1: per-pk version chains, independent of `pk_index`. Backs
    /// `get_at`/`scan_at`/`gc_versions_before`; does not affect the current-value
    /// read path (`get`/`scan`), which is unchanged.
    version_index: VersionIndex,
    /// Row-data/WAL encryption at rest (path to production item #1). When
    /// set, every `Mutation` payload is AES-256-GCM encrypted before it
    /// reaches the WAL, and must be decrypted before `Mutation::decode` on
    /// replay (the replay loop lives in `vectorvolt-engine`, not here — see
    /// `wal_key()` below, which callers use to mirror this same key).
    /// `None` preserves the existing on-disk format exactly (opt-in only).
    wal_key: Option<Arc<vectorvolt_crypto::KeyStore>>,
    /// Whether the WAL writer has been told `wal_key`'s key id, so every
    /// segment that receives encrypted frames records it (PP-52). Set lazily
    /// on the first commit if [`with_wal_encryption`](Self::with_wal_encryption)
    /// could not rotate the segment at configuration time.
    wal_key_bound: AtomicBool,
    /// MAC key for the WAL tail seal (PP-38), derived from `wal_key`. With it,
    /// every synced commit records the sealed tip, and the highest sequence
    /// sealed so far is kept here so concurrent commits never move it back.
    seal: Option<([u8; 32], Mutex<u64>)>,
    /// `fsync` each partition file (and its directory) before it is published.
    durable_flush: bool,
    /// Held shared by every mutation from its WAL append until its in-memory
    /// effect is applied; taken exclusively by [`TxnManager::quiesced_tip`].
    /// That gives a snapshot a WAL position at which no mutation is half done.
    apply_gate: RwLock<()>,
    /// Per-pk write ordering (PP-23). A mutation holds the stripe of every pk it
    /// touches from before its WAL append until its in-memory effect is applied,
    /// so for any one pk the WAL order, the commit timestamps and the order the
    /// index / version chain / write buffer see the writes are all the same.
    /// Writers of pks on different stripes do not contend.
    write_stripes: Box<[Mutex<()>]>,
    /// Opened, CRC-verified partition readers, by `(file_id, partition_id)`
    /// (B25). Entries are evicted whenever a file under that id is replaced or
    /// removed (flush, compaction), so an id that is later reused never serves
    /// the old file's bytes.
    readers: RwLock<HashMap<(u16, u16), Arc<PartitionReader>>>,
    /// Bumped by every eviction; a reader opened across one is not cached.
    reader_epoch: AtomicU64,
    /// Set once version history has been rebuilt from the WAL at open, after
    /// which recovery's `replay_insert` must not add versions of its own.
    history_loaded: AtomicBool,
    /// Serialises compactions with each other (never with reads or writes).
    compaction_lock: Mutex<()>,
    /// See [`CompactionHook`].
    compaction_hook: Mutex<Option<CompactionHook>>,
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
            partition_count: 1,
            clock: CommitClock::new(),
            version_index: VersionIndex::new(),
            wal_key: None,
            wal_key_bound: AtomicBool::new(false),
            seal: None,
            durable_flush: true,
            apply_gate: RwLock::new(()),
            write_stripes: (0..WRITE_STRIPES).map(|_| Mutex::new(())).collect(),
            readers: RwLock::new(HashMap::new()),
            reader_epoch: AtomicU64::new(0),
            history_loaded: AtomicBool::new(false),
            compaction_lock: Mutex::new(()),
            compaction_hook: Mutex::new(None),
        }
    }

    /// Lock the write stripes covering `pks`, in ascending stripe order so two
    /// multi-pk writers can never deadlock.
    fn lock_stripes(&self, pks: impl IntoIterator<Item = u64>) -> Vec<MutexGuard<'_, ()>> {
        let mut ids: Vec<usize> = pks.into_iter().map(stripe_of).collect();
        ids.sort_unstable();
        ids.dedup();
        ids.into_iter()
            .map(|i| self.write_stripes[i].lock())
            .collect()
    }

    /// Apply a committed live version of `pk` to every in-memory structure.
    /// Returns whether the write buffer reached its flush threshold.
    fn apply_live(&self, pk: u64, values: Vec<u64>, commit_ts: u64) -> TxnResult<bool> {
        // MVCC: record this version before anything else observes it.
        self.version_index.push_live(pk, commit_ts, values.clone());
        {
            let mut sec = self.secondary_index.write();
            for (col_idx, &val) in values.iter().enumerate().skip(1) {
                sec.insert(col_value_hash(col_idx, val), pk as u32);
            }
        }
        self.fk_filter.write().insert(pk)?;
        // The writer thread is the single producer for the ring buffer: it
        // computes the offset, writes the PENDING index entry, then pushes.
        self.writer.insert(pk, values)
    }

    /// Whether flushed partition files are synced to disk before they are
    /// published. On by default; `TableEngine` turns it off for the
    /// non-fsync durability modes, which already accept losing recent writes.
    pub fn with_durable_flush(mut self, durable: bool) -> Self {
        self.durable_flush = durable;
        self
    }

    /// The WAL sequence number at an instant when every mutation at or below
    /// it has been fully applied in memory.
    ///
    /// A flush that starts after this returns therefore captures the effect of
    /// every frame up to the returned sequence — which is what lets a snapshot
    /// record a WAL position recovery can safely skip to.
    pub fn quiesced_tip(&self) -> u64 {
        let _exclusive = self.apply_gate.write();
        self.wal.wal().tip().sequence
    }

    /// Sequence of the last frame appended to the WAL, without quiescing
    /// writers (an upper bound for every write that has already returned).
    pub fn wal_tip_sequence(&self) -> u64 {
        self.wal.wal().tip().sequence
    }

    /// Opt into WAL/row-data encryption at rest: every mutation payload is
    /// encrypted before it reaches the WAL. The caller (`TableEngine`) is
    /// responsible for decrypting `frame.payload` with the same key during
    /// WAL replay before calling `Mutation::decode` — this manager only
    /// handles the write side.
    ///
    /// Each payload is sealed as `VVE2` with AAD = [`frame_aad`] of the key id
    /// and the frame's sequence number, and the WAL writer records the key id
    /// in every segment header it writes from now on (PP-52).
    ///
    /// [`frame_aad`]: vectorvolt_wal::segment::frame_aad
    pub fn with_wal_encryption(mut self, key: Arc<vectorvolt_crypto::KeyStore>) -> Self {
        let seal_key = vectorvolt_wal::seal::derive_seal_key(key.as_bytes());
        self.seal = Some((seal_key, Mutex::new(0)));
        // Usually a no-op: `TableEngine` already opened the writer with this
        // key id. If the rotation fails here it is retried by `commit`.
        let bound = self
            .wal
            .wal()
            .set_encryption_key_id(Some(key.key_id()))
            .is_ok();
        self.wal_key_bound = AtomicBool::new(bound);
        // Partition files and the manifest are encrypted under the same key
        // (PP-53). Registering it lets code that opens a partition by path
        // alone (zone peeks, `verify_partitions`) resolve it by key id.
        vectorvolt_storage::register_partition_key(&key);
        self.wal_key = Some(key);
        self
    }

    /// Record `tip` in the WAL tail seal. Only called for synced frames, so
    /// the seal never points past what is durable.
    fn seal_tip(&self, tip: &WalTip) -> TxnResult<()> {
        if let Some((key, sealed)) = &self.seal {
            let mut sealed = sealed.lock();
            if tip.sequence > *sealed {
                vectorvolt_wal::seal::write_seal(&self.wal.wal().dir(), key, tip)?;
                *sealed = tip.sequence;
            }
        }
        Ok(())
    }

    /// The configured encryption key, if any — exposed so `TableEngine` can
    /// reuse the identical key for WAL-replay decryption without a second
    /// place to configure it.
    pub fn wal_key(&self) -> Option<&Arc<vectorvolt_crypto::KeyStore>> {
        self.wal_key.as_ref()
    }

    /// Set the number of logical partitions rows are distributed across on flush.
    ///
    /// Must be called before the first `flush()`.  `n` must be >= 1; passing 0
    /// is treated as 1 to avoid a panic in `select_partition`.
    pub fn with_partition_count(mut self, n: usize) -> Self {
        self.partition_count = n.max(1);
        self
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
    ///
    /// With a key, the payload is encrypted inside the WAL writer's append
    /// lock, once its sequence number is fixed, so the AAD always names the
    /// position the frame actually lands at — even under concurrent appends.
    #[inline]
    fn commit(&self, payload: Vec<u8>) -> TxnResult<WalTip> {
        let tip = match &self.wal_key {
            Some(key) => {
                let key_id = key.key_id();
                if !self.wal_key_bound.load(Ordering::Acquire) {
                    self.wal.wal().set_encryption_key_id(Some(key_id))?;
                    self.wal_key_bound.store(true, Ordering::Release);
                }
                self.wal.wal().append_with(|seq| {
                    let aad = vectorvolt_wal::segment::frame_aad(&key_id, seq);
                    vectorvolt_crypto::encryptor::encrypt_with_aad(key, &payload, &aad)
                        .map_err(|e| TxnError::Encryption(format!("payload encrypt failed: {e}")))
                })?
            }
            None => self.wal.append(payload)?,
        };
        if self.sync_on_commit {
            self.wal.sync_upto(tip.sequence)?;
            self.seal_tip(&tip)?;
        }
        Ok(tip)
    }

    /// Append a non-mutation record (e.g. a PITR snapshot marker) through this
    /// table's one WAL writer and sync it.
    ///
    /// Opening a second `WalWriter` on the same directory forks the sequence
    /// numbers and the hash chain (PP-12). Markers are metadata, not row data,
    /// and are written unencrypted so `WalReader::scan_snapshot_markers` can
    /// still find them; replay ignores them because they are not mutations.
    pub fn append_marker(&self, payload: Vec<u8>) -> TxnResult<WalTip> {
        let tip = self.wal.append_and_sync(payload)?;
        self.seal_tip(&tip)?;
        Ok(tip)
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

        // WAL append and in-memory apply happen under this pk's stripe, so a
        // concurrent writer of the same pk can neither commit between them nor
        // apply out of WAL order (PP-23). The frame's sequence is the commit_ts.
        let mutation = Mutation::Insert {
            pk,
            values: values.clone(),
        };
        let (tip, full) = {
            let _applying = self.apply_gate.read_recursive();
            let _stripe = self.lock_stripes([pk]);
            let tip = self.commit(mutation.encode())?;
            let full = self.apply_live(pk, values, tip.sequence)?;
            self.clock.advance_to(tip.sequence);
            (tip, full)
        };

        // Auto-flush if the buffer filled. Outside the stripe: a flush is file
        // I/O and must not stall writers of unrelated pks.
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

        // Single WAL commit for the entire batch. Every row shares the frame's
        // sequence as its commit_ts, so the batch becomes visible atomically to
        // snapshot readers.
        let mutation = Mutation::InsertBatch { rows: rows.clone() };
        let (tip, any_full) = {
            let _applying = self.apply_gate.read_recursive();
            let _stripes = self.lock_stripes(rows.iter().map(|(pk, _)| *pk));
            let tip = self.commit(mutation.encode())?;
            let mut any_full = false;
            for (pk, values) in rows {
                any_full |= self.apply_live(pk, values, tip.sequence)?;
            }
            self.clock.advance_to(tip.sequence);
            (tip, any_full)
        };

        if any_full {
            self.flush()?;
        }

        Ok(tip)
    }

    // ── READ ──────────────────────────────────────────────────────────────────

    /// Fetch a single row by primary key.
    ///
    /// Returns `Ok(None)` if the key does not exist (not in `pk_index`).
    ///
    /// Two storage tiers are checked:
    /// 1. **Pending** (`file_id == PENDING_FILE_ID`): row is still in the
    ///    write buffer or the `in_flight` map (rows transferred to the writer
    ///    thread but not yet flushed).  `in_flight` is checked first because
    ///    the writer thread populates it atomically before returning from
    ///    `Drain`, so a row_offset that no longer points into the ring belongs
    ///    there.
    /// 2. **Flushed**: row lives in a `.vvsst` partition file.  The file is
    ///    opened via `PartitionReader`, which validates the magic and CRC
    ///    before any column is read. The opened reader is cached, so only the
    ///    first read of a partition pays for the open and the CRC check (B25).
    pub fn get(&self, pk: u64) -> TxnResult<Option<Vec<u64>>> {
        let mut desc = match self.pk_index.read().get(pk) {
            Some(d) => d,
            None => return Ok(None),
        };

        // A concurrent flush or compaction can move the row and retire the
        // place the descriptor named between our index read and the row read.
        // That is not a missing row: re-read the descriptor and follow it.
        let mut attempts = 0;
        loop {
            match self.read_at(pk, desc) {
                Ok(values) => return Ok(Some(values)),
                Err(e) => {
                    attempts += 1;
                    match self.pk_index.read().get(pk) {
                        None => return Ok(None),
                        Some(fresh) if fresh != desc && attempts < 3 => desc = fresh,
                        Some(_) => return Err(e),
                    }
                }
            }
        }
    }

    /// Resolve one descriptor to its row values.
    fn read_at(&self, pk: u64, desc: RowDescriptor) -> TxnResult<Vec<u64>> {
        let row_offset = desc.row_offset() as usize;

        if desc.file_id() == PENDING_FILE_ID {
            // Try in_flight map first (row handed to writer thread but not yet
            // flushed to disk).
            if let Some(values) = self.in_flight.lock().get(&pk).cloned() {
                return Ok(values);
            }
            // Fall back to the ring buffer (row still buffered locally).
            return self
                .writer
                .row_for(pk, row_offset)
                .ok_or(TxnError::PkNotFound(pk));
        }

        let reader = self.partition_reader(desc.file_id(), desc.partition_id())?;
        Ok(reader.read_row(row_offset)?)
    }

    /// The cached reader for a flushed partition, opening (and CRC-checking)
    /// it on first use.
    fn partition_reader(&self, file_id: u16, partition_id: u16) -> TxnResult<Arc<PartitionReader>> {
        if let Some(r) = self.readers.read().get(&(file_id, partition_id)) {
            return Ok(Arc::clone(r));
        }
        let path = self
            .flush_dir
            .join(format!("{file_id:05}_{partition_id:05}.vvsst"));
        // Opened outside the lock: the open maps and checksums the whole file,
        // and reads of other partitions must not wait on it. An eviction that
        // lands while we open means the file may already be retired, so the
        // reader then serves this one read (consistent with the descriptor it
        // was opened for) but is not cached.
        let epoch = self.reader_epoch.load(Ordering::Acquire);
        // Encrypted files are decrypted once here into private memory; the
        // cache below makes that a per-file, not per-read, cost (PP-53).
        let reader = Arc::new(PartitionReader::open_with_key(
            &path,
            self.wal_key.as_deref(),
        )?);
        let mut cache = self.readers.write();
        if self.reader_epoch.load(Ordering::Acquire) != epoch {
            return Ok(reader);
        }
        Ok(Arc::clone(
            cache.entry((file_id, partition_id)).or_insert(reader),
        ))
    }

    /// Drop cached readers for partitions whose file is being replaced or
    /// removed. Must run before the file is deleted (a live mapping blocks
    /// deletion on Windows) and after a new file is published under an id, so
    /// a reused id never serves an older file's bytes.
    fn evict_readers(&self, keys: impl IntoIterator<Item = (u16, u16)>) {
        let mut cache = self.readers.write();
        self.reader_epoch.fetch_add(1, Ordering::AcqRel);
        for k in keys {
            cache.remove(&k);
        }
    }

    /// Return all primary keys currently present in the index.
    ///
    /// The list is a snapshot — concurrent inserts or deletes may not be
    /// reflected. PKs are returned in unspecified order.
    pub fn scan_all_pks(&self) -> Vec<u64> {
        self.pk_index.read().iter().map(|(pk, _)| pk).collect()
    }

    /// Scan all live rows, returning `(pk, values)` pairs.
    ///
    /// Internally calls `get(pk)` for every key in `pk_index`.  The snapshot
    /// is taken from the index at the start of the call; rows inserted or
    /// deleted concurrently may or may not appear in the result.
    ///
    /// Returns `Err` on the first storage or I/O failure.  Rows whose `get`
    /// returns `None` (race with a concurrent delete) are silently skipped.
    pub fn scan(&self) -> TxnResult<Vec<(u64, Vec<u64>)>> {
        let pks: Vec<u64> = self.pk_index.read().iter().map(|(pk, _)| pk).collect();
        let mut rows = Vec::with_capacity(pks.len());
        for pk in pks {
            if let Some(values) = self.get(pk)? {
                rows.push((pk, values));
            }
        }
        Ok(rows)
    }

    // ── MVCC (Phase 1) ───────────────────────────────────────────────────────────

    /// Read `pk` as of `read_ts`: the latest version with `commit_ts <= read_ts`,
    /// or `None` if it did not exist yet or was deleted by then.
    ///
    /// Independent of `pk_index` / `get`: it walks the version chain maintained
    /// in `version_index`, so it stays correct even after the current value has
    /// been overwritten, relocated by a flush, or removed by a delete.
    pub fn get_at(&self, pk: u64, read_ts: u64) -> Option<Vec<u64>> {
        self.version_index.get_at(pk, read_ts)
    }

    /// Fix a read snapshot: returns the current commit timestamp. A transaction
    /// should call this at BEGIN and pass the result to `get_at`/`scan_at` for
    /// the lifetime of that transaction. (Wiring this into the SQL layer's BEGIN
    /// is a later step — this just exposes the primitive.)
    pub fn begin_snapshot(&self) -> u64 {
        self.clock.current()
    }

    /// Keys live as of `read_ts` (no row data), in unspecified order.
    pub fn pks_at(&self, read_ts: u64) -> Vec<u64> {
        self.version_index.visible_pks_at(read_ts)
    }

    /// Snapshot-consistent full scan as of `read_ts`.
    ///
    /// A pk is included if it had a live (non-tombstoned) version at or before
    /// `read_ts`, even if it was inserted after the current wall-clock moment's
    /// pk_index state changed, or has since been deleted or updated further.
    pub fn scan_at(&self, read_ts: u64) -> Vec<(u64, Vec<u64>)> {
        self.version_index
            .visible_pks_at(read_ts)
            .into_iter()
            .filter_map(|pk| {
                self.version_index
                    .get_at(pk, read_ts)
                    .map(|values| (pk, values))
            })
            .collect()
    }

    /// Prune version history for `pk` older than `watermark_ts`, always keeping
    /// at least the newest version with `commit_ts <= watermark_ts` so reads
    /// exactly at the watermark keep working. Not wired into any automatic
    /// background GC yet — callers invoke it directly.
    pub fn gc_versions_before(&self, pk: u64, watermark_ts: u64) {
        self.version_index.gc_before(pk, watermark_ts);
    }

    /// Access to the shared commit clock (e.g. to observe `current()` without
    /// creating a snapshot, or for diagnostics).
    pub fn commit_clock(&self) -> &crate::mvcc::CommitClock {
        &self.clock
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
    /// The row's bytes stay in any flushed `.vvsst` file until compaction drops
    /// them (the index, not the file, decides visibility). The delete stays in
    /// force across a restart because the index rebuild honours the WAL's
    /// newest record for every pk (see `rebuild_pk_index_from_partitions`), and
    /// a compaction racing the delete never re-points the deleted pk (B23).
    pub fn delete(&self, pk: u64) -> TxnResult<WalTip> {
        let _applying = self.apply_gate.read_recursive();
        // The existence check is under the stripe too: checked outside it, a
        // concurrent delete of the same pk could pass the check, and both would
        // log a Delete for a row only one of them removed.
        let _stripe = self.lock_stripes([pk]);
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

        // MVCC: append a tombstone rather than losing the pre-delete value —
        // old readers holding an earlier snapshot must still see it.
        self.version_index.push_tombstone(pk, tip.sequence);
        self.clock.advance_to(tip.sequence);

        Ok(tip)
    }

    // ── UPSERT BATCH ──────────────────────────────────────────────────────────

    /// Insert-or-overwrite multiple rows as a single WAL record.
    ///
    /// All rows are validated before the WAL commit. For each row, if the PK
    /// already exists in the index the old entry is removed before the new row
    /// is pushed (same semantics as `upsert`). New PKs are inserted as normal.
    ///
    /// Returns `Err(SchemaMismatch)` for the first row whose value count does
    /// not match the schema. On error, no rows from the batch are modified.
    pub fn upsert_batch(&self, rows: Vec<(u64, Vec<u64>)>) -> TxnResult<WalTip> {
        for (_, values) in &rows {
            if values.len() != self.schema.len() {
                return Err(TxnError::SchemaMismatch {
                    expected: self.schema.len(),
                    got: values.len(),
                });
            }
        }

        let (tip, any_full) = {
            let _applying = self.apply_gate.read_recursive();
            let _stripes = self.lock_stripes(rows.iter().map(|(pk, _)| *pk));
            let tip = self.commit(Mutation::UpsertBatch { rows: rows.clone() }.encode())?;
            let mut any_full = false;
            for (pk, values) in rows {
                // Remove old index entry if present (upsert semantics).
                self.pk_index.write().remove(pk);
                any_full |= self.apply_live(pk, values, tip.sequence)?;
            }
            self.clock.advance_to(tip.sequence);
            (tip, any_full)
        };

        if any_full {
            self.flush()?;
        }

        Ok(tip)
    }

    // ── TRANSACTION BATCH ─────────────────────────────────────────────────────

    /// Apply a committed transaction's puts and deletes as ONE WAL record.
    ///
    /// This is the commit primitive every transaction path uses (engine MVCC,
    /// wire protocol, gRPC, HTTP multi-table). One frame means one checksum:
    /// after a crash the transaction is replayed whole or not at all, never
    /// half. Each op is a final state — a put overwrites whatever is there and
    /// a delete of an absent pk is a no-op — so applying the same batch twice
    /// leaves the same result, which is what cross-table recovery relies on.
    ///
    /// Every put is validated before anything is written; on error nothing is
    /// applied.
    pub fn apply_batch(&self, ops: Vec<(u64, Option<Vec<u64>>)>) -> TxnResult<WalTip> {
        for (_, op) in &ops {
            if let Some(values) = op
                && values.len() != self.schema.len()
            {
                return Err(TxnError::SchemaMismatch {
                    expected: self.schema.len(),
                    got: values.len(),
                });
            }
        }

        let mut any_full = false;
        let tip = {
            let _applying = self.apply_gate.read_recursive();
            let _stripes = self.lock_stripes(ops.iter().map(|(pk, _)| *pk));
            let tip = self.commit(Mutation::TxnBatch { ops: ops.clone() }.encode())?;
            let ts = tip.sequence;
            for (pk, op) in ops {
                let existing = self.pk_index.read().get(pk);
                match op {
                    Some(values) => {
                        if existing.is_some() {
                            self.pk_index.write().remove(pk);
                        }
                        any_full |= self.apply_live(pk, values, ts)?;
                    }
                    None => {
                        let Some(desc) = existing else {
                            continue;
                        };
                        self.pk_index.write().remove(pk);
                        *self
                            .pending_tombstones
                            .lock()
                            .entry((desc.file_id(), desc.partition_id()))
                            .or_insert(0) += 1;
                        self.version_index.push_tombstone(pk, ts);
                    }
                }
            }
            self.clock.advance_to(ts);
            tip
        };

        if any_full {
            self.flush()?;
        }
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
        let _applying = self.apply_gate.read_recursive();
        let _stripes = self.lock_stripes(pks.iter().copied());

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
                self.version_index.push_tombstone(pk, tip.sequence);
            }
        }
        self.clock.advance_to(tip.sequence);

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
        let (tip, full) = {
            let _applying = self.apply_gate.read_recursive();
            let _stripe = self.lock_stripes([pk]);
            let tip = self.commit(mutation.encode())?;

            // Remove stale index entry if the pk already exists.
            if self.pk_index.read().get(pk).is_some() {
                self.pk_index.write().remove(pk);
            }
            let full = self.apply_live(pk, values, tip.sequence)?;
            self.clock.advance_to(tip.sequence);
            (tip, full)
        };

        // Auto-flush on threshold.
        if full {
            self.flush()?;
        }

        Ok(tip)
    }

    /// Atomically update an existing row.
    ///
    /// Unlike `upsert`, this returns `PkNotFound` when the PK does not exist.
    /// Logs a single `Mutation::Update` WAL record, then tombstones the old
    /// physical row and inserts the new values — preventing the storage-orphan
    /// problem of a plain delete + insert (VV-TXN-004.6).
    pub fn update(&self, pk: u64, values: Vec<u64>) -> TxnResult<WalTip> {
        if values.len() != self.schema.len() {
            return Err(TxnError::SchemaMismatch {
                expected: self.schema.len(),
                got: values.len(),
            });
        }
        let (tip, full) = {
            let _applying = self.apply_gate.read_recursive();
            let _stripe = self.lock_stripes([pk]);
            // PK must exist — update is not an insert. Checked under the stripe
            // so a concurrent delete cannot slip in between check and commit.
            if self.pk_index.read().get(pk).is_none() {
                return Err(TxnError::PkNotFound(pk));
            }
            // WAL first — single Update record covers the tombstone + re-insert.
            let tip = self.commit(
                Mutation::Update {
                    pk,
                    values: values.clone(),
                }
                .encode(),
            )?;
            // Tombstone the old descriptor. The new version is appended (update
            // never leaves a gap where the row is invisible — the old version
            // stays reachable via get_at).
            self.pk_index.write().remove(pk);
            let full = self.apply_live(pk, values, tip.sequence)?;
            self.clock.advance_to(tip.sequence);
            (tip, full)
        };
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
        let (m, recovery) =
            Manifest::open_with_key(self.flush_dir.join("MANIFEST"), self.wal_key.clone())?;
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
        let prepared = self.prepare_compaction(strategy, write_options)?;
        let hook = self.compaction_hook.lock().clone();
        if let Some(hook) = hook {
            hook();
        }
        self.install_compaction(prepared)
    }

    /// Install a hook that [`run_compaction`](Self::run_compaction) runs between
    /// building and installing its outputs. For tests that need to act while a
    /// compaction is in flight; `None` removes it.
    #[doc(hidden)]
    pub fn set_compaction_hook(&self, hook: Option<CompactionHook>) {
        *self.compaction_hook.lock() = hook;
    }

    /// The expensive half of compaction: plan, read the inputs, and write the
    /// merged outputs (B24).
    ///
    /// Takes no lock that reads or writes wait on — only the compaction lock,
    /// which serialises compactions with each other. Readers keep resolving
    /// rows from the input files throughout; nothing they can see changes until
    /// [`install_compaction`](Self::install_compaction). A caller that holds its
    /// own partition readers should call this without its partition lock, then
    /// take that lock (and release its mappings) only around the install.
    pub fn prepare_compaction(
        &self,
        strategy: CompactionStrategy,
        write_options: WriteOptions,
    ) -> TxnResult<PreparedCompaction<'_>> {
        let serial = self.compaction_lock.lock();

        // Flush pending tombstone counts first, so the plan is computed from a
        // manifest that reflects every delete so far.
        self.persist_tombstones()?;

        let partitions = self.partitions();
        if partitions.is_empty() {
            return Ok(PreparedCompaction {
                _serial: serial,
                built: Vec::new(),
            });
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

        let mut built = Vec::new();
        for job in jobs {
            let file_id = self.next_file_id.load(Ordering::Relaxed);
            let partition_id = self.alloc_partition_id()?;
            let opts = CompactionOptions {
                output_dir: self.flush_dir.clone(),
                output_file_id: file_id,
                output_partition_id: partition_id,
                write_options,
                now_millis: now_millis(),
                encryption_key: self.wal_key.clone(),
            };

            let liveness = IndexLiveness { txn: self };
            match vectorvolt_storage::execute_job(&job, &self.schema, &liveness, &opts) {
                Ok(outcome) => built.push(outcome),
                Err(e) => {
                    // Nothing built so far is installed; remove it rather than
                    // leave orphans behind a failed run.
                    for o in &built {
                        let _ = std::fs::remove_file(&o.output.path);
                    }
                    return Err(e.into());
                }
            }
        }
        Ok(PreparedCompaction {
            _serial: serial,
            built,
        })
    }

    /// The cheap half of compaction: swap prepared outputs in (B24).
    ///
    /// Per job: re-point the PK index, swap the manifest, evict cached readers,
    /// delete the inputs. Only the PK index write lock is held, and only while
    /// re-pointing.
    pub fn install_compaction(
        &self,
        prepared: PreparedCompaction<'_>,
    ) -> TxnResult<Vec<CompactionOutcome>> {
        let mut outcomes = Vec::new();
        for outcome in prepared.built {
            let file_id = outcome.output.file_id;
            let partition_id = outcome.output.partition_id;
            let inputs: HashSet<(u16, u16)> = outcome.removed.iter().copied().collect();

            // Re-point the PK index at the merged partition BEFORE the manifest
            // swap makes it the live one.
            //
            // Compaction moves every surviving row to a new
            // (file_id, partition_id, row_offset). An index still holding the old
            // coordinates addresses rows in a file that is about to be deleted —
            // so every point lookup for a compacted key would fail or, worse, read
            // a stale offset in a recycled id.
            //
            // Only a key the index still places in one of the inputs is moved.
            // The merge ran without blocking writers, so since it judged a row
            // live the key may have been deleted (no descriptor) or written again
            // (a pending or newer-partition descriptor). Re-pointing those would
            // resurrect the delete or roll back the newer write (B23); the merged
            // copy is garbage instead, and is counted as such.
            let mut stale = 0u32;
            {
                let mut idx = self.pk_index.write();
                for (row_offset, pk) in outcome.output_pks.iter().enumerate() {
                    let still_input = idx
                        .get(*pk)
                        .is_some_and(|d| inputs.contains(&(d.file_id(), d.partition_id())));
                    if !still_input {
                        stale += 1;
                        continue;
                    }
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

            // The inputs no longer exist. Any tombstone count that accumulated
            // for them during the merge would otherwise be attributed to a
            // partition id that may later be reused. Rows that went stale while
            // the merge ran are the output's garbage.
            {
                let mut pending = self.pending_tombstones.lock();
                for key in &outcome.removed {
                    pending.remove(key);
                }
                if stale > 0 {
                    *pending.entry((file_id, partition_id)).or_insert(0) += stale;
                }
            }

            // Release cached mappings of the inputs before deleting them, and of
            // the output id in case it was reused.
            self.evict_readers(
                outcome
                    .removed
                    .iter()
                    .copied()
                    .chain([(file_id, partition_id)]),
            );

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

        // Group rows by their logical partition (PK-hash routing).
        // With partition_count=1 this is a no-op — all rows land in group 0.
        // Rows keep their drain order (oldest first) inside each group.
        let pc = self.partition_count;
        let mut groups: Vec<Vec<DrainedRow>> = vec![Vec::new(); pc];
        for row in &rows {
            groups[select_partition(row.pk, pc)].push(row.clone());
        }

        std::fs::create_dir_all(&self.flush_dir)?;

        let mut written: Vec<(u16, Vec<DrainedRow>)> = Vec::with_capacity(pc);
        let mut total_written = 0usize;
        let mut last_tip = None;

        for group_rows in groups.into_iter() {
            if group_rows.is_empty() {
                continue;
            }
            // One id per written file, skipping any id whose file still exists.
            // A bare counter wraps at u16::MAX and would overwrite live data.
            let pid = self.alloc_partition_id()?;
            let group_count = group_rows.len();

            let mut mp = MicroPartition::new(file_id, pid);
            for (col_idx, name) in self.schema.iter().enumerate() {
                let col_vals: Vec<u64> = group_rows.iter().map(|r| r.values[col_idx]).collect();
                let encoded = if col_idx == 0 {
                    ColumnValues::Raw(col_vals)
                } else {
                    ColumnValues::auto_encode(col_vals)
                };
                mp.push_column(name.clone(), encoded)?;
            }

            // Temp-then-rename so a reader never sees a half-written file. The
            // data is synced before the rename and the directory after it:
            // without both, a crash can leave the manifest naming a file whose
            // contents (or name) never reached the disk.
            let filename = format!("{file_id:05}_{pid:05}.vvsst");
            let tmp_filename = format!("{file_id:05}_{pid:05}.vvsst.tmp");
            let path = self.flush_dir.join(&filename);
            let tmp_path = self.flush_dir.join(&tmp_filename);
            {
                let mut file = std::fs::File::create(&tmp_path)?;
                write_partition_keyed(
                    &mut file,
                    &mp,
                    WriteOptions::default(),
                    self.wal_key.as_deref(),
                )?;
                if self.durable_flush {
                    file.sync_all()?;
                }
            }
            std::fs::rename(&tmp_path, &path)?;
            if self.durable_flush {
                sync_dir(&self.flush_dir)?;
            }
            // The id may be one compaction freed; never serve its old bytes.
            self.evict_readers([(file_id, pid)]);

            // Typed checkpoint (VV-WAL-011). Recovery does not depend on it —
            // replay reconciles against partition contents — but it gives the
            // WAL a decodable record of every file a flush produced.
            let ckpt = Mutation::Checkpoint {
                table_name: filename,
                lsn: self.wal.wal().tip().sequence,
                timestamp: now_millis(),
            };
            last_tip = Some(self.commit(ckpt.encode())?);

            total_written += group_count;
            written.push((pid, group_rows));
        }

        let tip = last_tip.unwrap_or_else(|| {
            // All groups were empty after grouping — shouldn't happen since we
            // checked rows.is_empty() above, but be defensive.
            unreachable!("flush: rows non-empty but all groups empty")
        });

        // Rebuild RowDescriptors: pending → real (file_id, partition_id, offset).
        //
        // Only a descriptor that still points at *this* buffered row is
        // replaced. Between the drain and here the pk may have been deleted
        // (no descriptor) or written again (a newer pending offset); blindly
        // re-pointing it would resurrect the delete or roll back the newer
        // write. The flushed copy is then garbage and is counted as such so
        // compaction can reclaim it.
        let _base = self
            .total_rows_flushed
            .fetch_add(total_written as u64, Ordering::Relaxed);
        {
            let mut idx = self.pk_index.write();
            let mut tombstones = self.pending_tombstones.lock();
            for (pid, group_rows) in &written {
                for (off, row) in group_rows.iter().enumerate() {
                    let still_this_row = idx.get(row.pk).is_some_and(|d| {
                        d.file_id() == PENDING_FILE_ID && d.row_offset() == row.offset
                    });
                    if still_this_row {
                        idx.insert(row.pk, RowDescriptor::new(file_id, *pid, off as u32))?;
                    } else {
                        *tombstones.entry((file_id, *pid)).or_insert(0) += 1;
                    }
                }
            }
        }

        // Every drained row now has a real descriptor, so the in-flight copies can
        // go. Done after the index rewrite, never before. A newer copy of the same
        // pk (forced out of a full ring after this drain) is left in place.
        {
            let mut inf = self.in_flight.lock();
            for row in &rows {
                if inf.get(&row.pk) == Some(&row.values) {
                    inf.remove(&row.pk);
                }
            }
        }

        // ZoneMap + Manifest: one entry per written partition group.
        let mut first_pid = 0;
        let mut first_path = self.flush_dir.join("none");

        for (g_idx, (pid, group_rows)) in written.iter().enumerate() {
            let pk_min = group_rows.iter().map(|r| r.pk).min().unwrap_or(0);
            let pk_max = group_rows.iter().map(|r| r.pk).max().unwrap_or(0);
            self.zone_map.write().set(*pid, pk_min, pk_max);

            let gpath = self.flush_dir.join(format!("{file_id:05}_{pid:05}.vvsst"));

            if g_idx == 0 {
                first_pid = *pid;
                first_path = gpath.clone();
            }

            if let Some(m) = self.manifest.lock().as_mut() {
                let size_bytes = std::fs::metadata(&gpath).map(|md| md.len()).unwrap_or(0);
                m.apply(ManifestEdit::Add(Box::new(PartitionMeta {
                    file_id,
                    partition_id: *pid,
                    path: gpath,
                    level: 0,
                    size_bytes,
                    row_count: group_rows.len() as u32,
                    created_at_millis: now_millis(),
                    min_key: pk_min,
                    max_key: pk_max,
                    tombstones: 0,
                    worm_locked: false,
                })))?;
            }
        }

        Ok(Some(FlushResult {
            file_id,
            partition_id: first_pid,
            row_count: total_written,
            path: first_path,
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

        // MVCC: once history has been rebuilt from the WAL at open, every
        // version this replay re-buffers is already in the chain under its
        // real commit_ts. Without that history, record the row as of the
        // recovery point — `current()`, not `next()`, so no timestamp that a
        // live WAL frame will use is consumed.
        if !self.history_loaded.load(Ordering::Acquire) {
            self.version_index
                .push_live(pk, self.clock.current(), values.clone());
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
    ///
    /// Also rebuilds MVCC history (PP-24) and honours durable deletes (B23):
    ///
    /// - Every mutation frame in this table's WAL becomes a version (or a
    ///   tombstone) at `commit_ts = frame sequence`, so `get_at`/`scan_at` with a
    ///   timestamp taken before the restart answer exactly as they did before
    ///   it, and the clock resumes past every sequence already used.
    /// - A pk whose newest WAL record is a delete is not indexed, whatever
    ///   partition file still holds an older copy of it. Deletes are not written
    ///   into `.vvsst` files, so without this a delete covered by a snapshot
    ///   marker (and therefore skipped by replay) came back after a restart.
    /// - A row on disk with no WAL history at all (a table whose WAL predates
    ///   this, or was removed) gets one base version at timestamp 0.
    ///
    /// The WAL is read best-effort: if it cannot be read or decrypted, no
    /// history is loaded and the index is rebuilt from the partitions alone, as
    /// before — the caller's own replay reports the error.
    pub fn rebuild_pk_index_from_partitions(
        &self,
        partitions: &std::collections::HashMap<(u16, u16), vectorvolt_storage::PartitionReader>,
    ) -> TxnResult<()> {
        let history = self.load_wal_history();
        let mut idx = self.pk_index.write();
        let mut flt = self.fk_filter.write();
        // Newest wins: partition ids are allocated in increasing order, so
        // visiting them ascending leaves each pk pointing at its latest copy.
        // HashMap order would make the winner arbitrary.
        let mut ordered: Vec<_> = partitions.iter().collect();
        ordered.sort_by_key(|(k, _)| **k);
        for ((file_id, partition_id), reader) in ordered {
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

        let Some(latest_deleted) = history else {
            return Ok(());
        };
        let indexed: Vec<(u64, RowDescriptor)> = idx.iter().collect();
        let mut base = Vec::new();
        for (pk, desc) in indexed {
            match latest_deleted.get(&pk) {
                Some(true) => {
                    idx.remove(pk);
                    *self
                        .pending_tombstones
                        .lock()
                        .entry((desc.file_id(), desc.partition_id()))
                        .or_insert(0) += 1;
                }
                Some(false) => {}
                None => {
                    if let Some(row) = partitions
                        .get(&(desc.file_id(), desc.partition_id()))
                        .and_then(|r| r.read_row(desc.row_offset() as usize).ok())
                    {
                        base.push((pk, VersionEntry::live(0, row)));
                    }
                }
            }
        }
        self.version_index.extend(base);
        Ok(())
    }

    /// Rebuild the version chains from the WAL (see
    /// [`rebuild_pk_index_from_partitions`](Self::rebuild_pk_index_from_partitions)).
    ///
    /// Returns, per pk, whether its newest record is a delete; `None` if the
    /// WAL could not be read in full, in which case nothing was loaded.
    fn load_wal_history(&self) -> Option<HashMap<u64, bool>> {
        if self.history_loaded.load(Ordering::Acquire) {
            return None;
        }
        let cfg = vectorvolt_wal::WalConfig::default()
            .with_dir(self.wal.wal().dir())
            .with_fsync(false);
        let width = self.schema.len();
        let mut versions: Vec<(u64, VersionEntry)> = Vec::new();
        let mut latest_deleted: HashMap<u64, bool> = HashMap::new();
        let mut max_seq = 0u64;
        let mut ok = true;
        let scanned = vectorvolt_wal::WalReader::open(cfg).scan_with_header(
            vectorvolt_wal::WalCursor::beginning(),
            |frame, header| {
                max_seq = max_seq.max(frame.sequence);
                let seq = frame.sequence;
                let payload =
                    match open_wal_payload(self.wal_key.as_deref(), header, seq, frame.payload) {
                        Ok(p) => p,
                        Err(_) => {
                            ok = false;
                            return false;
                        }
                    };
                let ts = frame.sequence;
                let mut put = |pk: u64, op: Option<Vec<u64>>| {
                    latest_deleted.insert(pk, op.is_none());
                    match op {
                        None => versions.push((pk, VersionEntry::tombstone(ts))),
                        // A row logged before an ALTER changed the column count
                        // cannot be served at the current width; recovery skips
                        // it too. It still counts as "not deleted".
                        Some(v) if v.len() == width => {
                            versions.push((pk, VersionEntry::live(ts, v)));
                        }
                        Some(_) => {}
                    }
                };
                match Mutation::decode(&payload) {
                    Ok(Mutation::Insert { pk, values })
                    | Ok(Mutation::Upsert { pk, values })
                    | Ok(Mutation::Update { pk, values }) => put(pk, Some(values)),
                    Ok(Mutation::Delete { pk }) => put(pk, None),
                    Ok(Mutation::InsertBatch { rows }) | Ok(Mutation::UpsertBatch { rows }) => {
                        for (pk, values) in rows {
                            put(pk, Some(values));
                        }
                    }
                    Ok(Mutation::DeleteBatch { pks }) => {
                        for pk in pks {
                            put(pk, None);
                        }
                    }
                    Ok(Mutation::TxnBatch { ops }) => {
                        for (pk, op) in ops {
                            put(pk, op);
                        }
                    }
                    // Checkpoints, DDL, markers and legacy payloads carry no row.
                    _ => {}
                }
                true
            },
        );
        if scanned.is_err() || !ok {
            return None;
        }
        self.version_index.extend(versions);
        self.clock.advance_to(max_seq);
        self.history_loaded.store(true, Ordering::Release);
        Some(latest_deleted)
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

    /// Allocate the next free partition ID, advancing the counter.
    ///
    /// The id space is `u16`, so the counter wraps; an id is only handed out
    /// when no partition file (or in-progress temp file) with that id exists,
    /// so wrapping reuses ids freed by compaction instead of overwriting live
    /// data. Errors only when every id is in use.
    pub fn alloc_partition_id(&self) -> TxnResult<u16> {
        let file_id = self.next_file_id.load(Ordering::Relaxed);
        for _ in 0..=u16::MAX as u32 {
            let pid = self.next_partition_id.fetch_add(1, Ordering::Relaxed);
            let base = format!("{file_id:05}_{pid:05}.vvsst");
            let taken = self.flush_dir.join(&base).exists()
                || self.flush_dir.join(format!("{base}.tmp")).exists();
            if !taken {
                return Ok(pid);
            }
        }
        Err(TxnError::PartitionIdsExhausted)
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// A WAL key problem found before or during replay (PP-52): the sequence it
/// concerns (a segment's base sequence for header checks) and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalKeyError {
    pub seq: u64,
    pub reason: String,
}

impl std::fmt::Display for WalKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WAL frame {}: {}", self.seq, self.reason)
    }
}

/// Check every segment header in the WAL at `wal_dir` against `key` before
/// replay reads a frame (PP-52). An encrypted segment needs a key, and the
/// key's id must equal the one the header records. Plaintext segments are
/// accepted either way, so a table that gains encryption keeps its history.
pub fn check_wal_segment_keys(
    wal_dir: &std::path::Path,
    key: Option<&vectorvolt_crypto::KeyStore>,
) -> Result<(), WalKeyError> {
    use vectorvolt_wal::segment::key_id_hex;
    let cfg = vectorvolt_wal::WalConfig::default()
        .with_dir(wal_dir)
        .with_fsync(false);
    let headers = vectorvolt_wal::WalReader::open(cfg)
        .segment_headers()
        .map_err(|e| WalKeyError {
            seq: 0,
            reason: format!("cannot read WAL segment headers: {e}"),
        })?;
    let given = key.map(|k| k.key_id());
    for h in headers {
        let Some(want) = h.key_id else { continue };
        match given {
            None => {
                return Err(WalKeyError {
                    seq: h.base_seq,
                    reason: format!(
                        "WAL segment {} is encrypted under key id {} but no key was provided",
                        h.segment_id,
                        key_id_hex(&want)
                    ),
                });
            }
            Some(got) if got != want => {
                return Err(WalKeyError {
                    seq: h.base_seq,
                    reason: format!(
                        "WAL segment {} is encrypted under key id {} but the key provided has id {}",
                        h.segment_id,
                        key_id_hex(&want),
                        key_id_hex(&got)
                    ),
                });
            }
            Some(_) => {}
        }
    }
    Ok(())
}

/// Recover the plaintext of one WAL frame payload read from the segment with
/// `header` (PP-52).
///
/// - `VVE2` payloads are decrypted with AAD = (key id, `seq`): a ciphertext
///   moved to another position, a wrong key or altered bytes all fail.
/// - `VVE1` payloads (written before AAD binding) decrypt as before, but only
///   in segments whose header does not record a key id — inside a keyed
///   segment an unbound frame can only be a downgrade.
/// - Plaintext payloads (snapshot markers, pre-encryption history) are
///   returned as they are, except that a row mutation in plaintext inside an
///   encrypted segment is refused: the writer never produces one.
pub fn open_wal_payload(
    key: Option<&vectorvolt_crypto::KeyStore>,
    header: &vectorvolt_wal::segment::SegmentHeader,
    seq: u64,
    payload: Vec<u8>,
) -> Result<Vec<u8>, WalKeyError> {
    let fail = |reason: String| WalKeyError { seq, reason };
    if vectorvolt_crypto::encryptor::is_aad_encrypted(&payload) {
        let key = key.ok_or_else(|| fail("frame is encrypted but no key was provided".into()))?;
        let aad = vectorvolt_wal::segment::frame_aad(&key.key_id(), seq);
        return vectorvolt_crypto::encryptor::decrypt_with_aad(key, &payload, &aad).map_err(|e| {
            fail(format!(
                "{e}; the frame is bound to sequence {seq} under key id {}, so it was \
                 moved, altered, or the key is wrong",
                vectorvolt_wal::segment::key_id_hex(&key.key_id())
            ))
        });
    }
    if vectorvolt_crypto::is_encrypted(&payload) {
        if header.is_encrypted() {
            return Err(fail(format!(
                "unbound (VVE1) frame inside AAD-bound WAL segment {}",
                header.segment_id
            )));
        }
        let key = key.ok_or_else(|| fail("frame is encrypted but no key was provided".into()))?;
        return vectorvolt_crypto::decrypt(key, &payload).map_err(|e| fail(e.to_string()));
    }
    if header.is_encrypted() && is_row_mutation(&payload) {
        return Err(fail(format!(
            "plaintext row mutation inside encrypted WAL segment {}",
            header.segment_id
        )));
    }
    Ok(payload)
}

/// Whether `payload` decodes as a mutation that changes row data.
fn is_row_mutation(payload: &[u8]) -> bool {
    matches!(
        Mutation::decode(payload),
        Ok(Mutation::Insert { .. }
            | Mutation::Upsert { .. }
            | Mutation::Update { .. }
            | Mutation::Delete { .. }
            | Mutation::InsertBatch { .. }
            | Mutation::UpsertBatch { .. }
            | Mutation::DeleteBatch { .. }
            | Mutation::TxnBatch { .. })
    )
}

/// The write stripe a pk belongs to (PP-23). Fibonacci hashing, so sequential
/// pks spread over every stripe instead of clustering.
#[inline]
fn stripe_of(pk: u64) -> usize {
    (pk.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> (64 - WRITE_STRIPES.trailing_zeros())) as usize
}

/// Stable per-column value hash for SecondaryIndex bucket keys.
#[inline]
fn col_value_hash(col_idx: usize, value: u64) -> u32 {
    let mixed = value
        .wrapping_add(col_idx as u64)
        .wrapping_mul(0x517c_c1b7_2722_0a95);
    (mixed ^ (mixed >> 32)) as u32
}

/// Make a rename durable by syncing its directory.
///
/// POSIX only persists a directory entry once the directory itself is synced.
/// Windows cannot open a directory for syncing this way and journals the
/// rename itself, so this is a no-op there.
fn sync_dir(dir: &std::path::Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::fs::File::open(dir)?.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let _ = dir;
    }
    Ok(())
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
