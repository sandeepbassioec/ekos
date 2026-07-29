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
use vectorvolt_storage::PartitionReader;
pub use vectorvolt_txn::{FlushResult, WalTip};
use vectorvolt_txn::{Mutation, TxnManager};
use vectorvolt_wal::{WalConfig, WalReader, WalWriter};

use crate::{EngineError, EngineResult, EngineRow, EngineStats};

/// Per-table engine: owns the write path (TxnManager) and read path (PartitionReader map).
pub struct TableEngine {
    txn: TxnManager,
    /// Flushed partition files keyed by (file_id, partition_id).
    partitions: RwLock<HashMap<(u16, u16), PartitionReader>>,
    partition_dir: PathBuf,
    pub schema: Vec<String>,
}

impl TableEngine {
    /// Open or create a table at `data_dir`.
    ///
    /// Directory layout:
    /// - `data_dir/wal/`  — WAL segment files
    /// - `data_dir/data/` — `.vvsst` partition files
    ///
    /// Existing `.vvsst` files are loaded on open (restart durability).
    pub fn open(
        schema: Vec<String>,
        data_dir: impl Into<PathBuf>,
        flush_threshold: usize,
    ) -> EngineResult<Self> {
        let data_dir = data_dir.into();
        let wal_dir = data_dir.join("wal");
        let partition_dir = data_dir.join("data");
        std::fs::create_dir_all(&wal_dir)?;
        std::fs::create_dir_all(&partition_dir)?;

        let cfg = WalConfig::default().with_dir(&wal_dir).with_fsync(false);
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
        );

        // Rebuild pk_index from existing partition files so reads work across
        // process restarts (Phase 2 crash recovery).
        let rows_on_disk: u64 = partitions.values().map(|r| r.row_count() as u64).sum();
        txn.rebuild_pk_index_from_partitions(&partitions)?;

        // Replay WAL delete records so deletes made in previous sessions survive
        // restart.  Insert records are skipped — their rows are already in
        // partition files (we flush every CLI insert) or are lost pending data
        // (acceptable Phase 2 limitation).
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

        Ok(TableEngine {
            txn,
            partitions: RwLock::new(partitions),
            partition_dir,
            schema,
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
        if self.txn.row_count_buffered() > 0 {
            self.do_flush()?;
        }

        let desc = match self.txn.pk_index.read().get(pk) {
            Some(d) => d,
            None => return Ok(None),
        };

        let file_id = desc.file_id();
        let partition_id = desc.partition_id();
        let row_offset = desc.row_offset() as usize;

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

            // Decode all columns eagerly (SoA layout amortises the decode cost).
            let all_cols: Vec<Vec<u64>> = (0..reader.col_count() as usize)
                .map(|c| reader.read_column(c).map(|cv| cv.to_raw_vec()))
                .collect::<Result<_, _>>()?;

            let scan_vals = &all_cols[col_idx];
            for offset in 0..scan_vals.len() {
                let val = scan_vals[offset];
                if val >= lo && val <= hi {
                    let pk = all_cols[0][offset];
                    let values: Vec<u64> = all_cols.iter().map(|c| c[offset]).collect();
                    results.push(EngineRow { pk, values });
                }
            }
        }

        Ok(results)
    }

    // ── Introspection ─────────────────────────────────────────────────────────

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
    fn sync_partitions(&self) -> EngineResult<()> {
        let mut parts = self.partitions.write();
        for entry in std::fs::read_dir(&self.partition_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("vvsst") {
                continue;
            }
            let reader = PartitionReader::open(&path)?;
            let key = (reader.file_id(), reader.partition_id());
            parts.entry(key).or_insert(reader);
        }
        Ok(())
    }
}
