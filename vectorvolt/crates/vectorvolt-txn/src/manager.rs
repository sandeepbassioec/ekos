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

use std::path::PathBuf;
use std::sync::atomic::{AtomicU16, AtomicU64, Ordering};

use parking_lot::{Mutex, RwLock};
use vectorvolt_index::{CuckooFilter, CuckooMap, RowDescriptor, SecondaryIndex, ZoneMap};
use vectorvolt_storage::{ColumnValues, MicroPartition, write_partition};
pub use vectorvolt_wal::WalTip;
use vectorvolt_wal::WalWriter;

use crate::{TxnError, TxnResult, buffer::WriteBuffer, ops::Mutation};

// Sentinel file_id for rows still in the write buffer.
const PENDING_FILE_ID: u16 = u16::MAX;

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
    wal: WalWriter,
    pub pk_index: RwLock<CuckooMap>,
    pub secondary_index: RwLock<SecondaryIndex>,
    pub zone_map: RwLock<ZoneMap>,
    pub fk_filter: RwLock<CuckooFilter>,
    write_buf: Mutex<WriteBuffer>,
    flush_dir: PathBuf,
    pub schema: Vec<String>,
    next_file_id: AtomicU16,
    next_partition_id: AtomicU16,
    total_rows_flushed: AtomicU64,
    /// Flush the WAL out of its userspace buffer on every commit.
    sync_on_commit: bool,
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
        TxnManager {
            wal,
            pk_index: RwLock::new(CuckooMap::with_capacity(64)),
            secondary_index: RwLock::new(SecondaryIndex::new()),
            zone_map: RwLock::new(ZoneMap::new()),
            fk_filter: RwLock::new(CuckooFilter::with_capacity(64)),
            write_buf: Mutex::new(WriteBuffer::new(flush_threshold, schema_width)),
            flush_dir: flush_dir.into(),
            schema,
            next_file_id: AtomicU16::new(0),
            next_partition_id: AtomicU16::new(start_partition_id),
            total_rows_flushed: AtomicU64::new(0),
            // Default off preserves the historical behaviour for direct users of
            // this crate; TableEngine opts in per its Durability setting.
            sync_on_commit: false,
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

    /// Push the WAL past the userspace buffer, if configured to.
    #[inline]
    fn sync_wal(&self) -> TxnResult<()> {
        if self.sync_on_commit {
            self.wal.flush()?;
        }
        Ok(())
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
        let tip = self.wal.append(mutation.encode())?;
        self.sync_wal()?;

        // 2-3. CuckooMap: pk → pending RowDescriptor.
        let buf_offset = self.write_buf.lock().len() as u32;
        self.pk_index
            .write()
            .insert(pk, RowDescriptor::new(PENDING_FILE_ID, 0, buf_offset))?;

        // 4. SecondaryIndex: one entry per non-PK column.
        {
            let mut sec = self.secondary_index.write();
            for (col_idx, &val) in values.iter().enumerate().skip(1) {
                sec.insert(col_value_hash(col_idx, val), pk as u32);
            }
        }

        // 5. CuckooFilter.
        self.fk_filter.write().insert(pk)?;

        // 6-7. Buffer; auto-flush if full.
        let full = self.write_buf.lock().push(pk, values);
        if full {
            self.flush()?;
        }

        Ok(tip)
    }

    // ── DELETE ─────────────────────────────────────────────────────────────────

    /// Remove a row from the in-memory index. WAL-logged.
    ///
    /// Returns `Err(PkNotFound)` if the pk does not exist in the CuckooMap.
    /// Does not remove the row from any flushed `.vvsst` file — that requires
    /// compaction (deferred to a later phase).
    pub fn delete(&self, pk: u64) -> TxnResult<WalTip> {
        if self.pk_index.read().get(pk).is_none() {
            return Err(TxnError::PkNotFound(pk));
        }

        let tip = self.wal.append(Mutation::Delete { pk }.encode())?;
        self.sync_wal()?;
        self.pk_index.write().remove(pk);
        Ok(tip)
    }

    // ── FLUSH ──────────────────────────────────────────────────────────────────

    /// Flush the write buffer to a `.vvsst` partition file.
    ///
    /// Returns `None` if the buffer was empty. On success, updates the
    /// CuckooMap so all flushed PKs point to the real (file_id, partition_id),
    /// and records the PK zone [min_pk, max_pk] in the ZoneMap.
    pub fn flush(&self) -> TxnResult<Option<FlushResult>> {
        let rows = self.write_buf.lock().drain();
        if rows.is_empty() {
            return Ok(None);
        }

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

        // Write .vvsst file.
        let filename = format!("{file_id:05}_{partition_id:05}.vvsst");
        let path = self.flush_dir.join(&filename);
        let mut file = std::fs::File::create(&path)?;
        write_partition(&mut file, &mp)?;

        // WAL checkpoint record.
        let ckpt = format!(
            "flush file_id={file_id} partition_id={partition_id} rows={row_count} path={filename}"
        );
        let tip = self.wal.append(ckpt.into_bytes())?;

        // Rebuild RowDescriptors: pending → real coordinates.
        let base = self
            .total_rows_flushed
            .fetch_add(row_count as u64, Ordering::Relaxed);
        {
            let mut idx = self.pk_index.write();
            for (off, (pk, _)) in rows.iter().enumerate() {
                let _ = idx.insert(*pk, RowDescriptor::new(file_id, partition_id, off as u32));
            }
        }

        // ZoneMap: record PK min/max for this partition.
        let pk_min = rows.iter().map(|(pk, _)| *pk).min().unwrap_or(0);
        let pk_max = rows.iter().map(|(pk, _)| *pk).max().unwrap_or(0);
        self.zone_map.write().set(partition_id, pk_min, pk_max);

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

    pub fn row_count_buffered(&self) -> usize {
        self.write_buf.lock().len()
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
