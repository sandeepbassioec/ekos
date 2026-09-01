//! [`TieredWriter`] — tier-aware partition writer for VectorVolt (VV-STO-016).
//!
//! Buffers rows in an L0 in-memory accumulator and flushes to a named tier
//! directory with the tier's configured compression.

use std::path::PathBuf;

use crate::{
    StorageError, StorageResult,
    column::ColumnValues,
    partition::MicroPartition,
    tier::{StorageTier, TierStats, TieredStorageConfig},
    writer::{WriteOptions, write_partition_with},
};

fn tier_label(tier: StorageTier) -> &'static str {
    match tier {
        StorageTier::L0 => "l0",
        StorageTier::L1 => "l1",
        StorageTier::L2 => "l2",
        StorageTier::L3 => "l3",
        StorageTier::L4 => "l4",
    }
}

/// Tier-aware partition writer.
///
/// Rows accumulate in an in-memory L0 buffer. `flush_to_tier` drains the
/// buffer to a `.vvsst` file in the target tier's compression mode.
/// `maybe_flush` auto-promotes to L1 once the L0 byte budget is exhausted.
pub struct TieredWriter {
    config: TieredStorageConfig,
    l0_rows: Vec<(u64, Vec<u64>)>,
    l0_bytes: u64,
    base_dir: PathBuf,
    schema: Vec<String>,
    tier_stats: Vec<TierStats>,
    next_file_id: u16,
}

impl TieredWriter {
    pub fn new(
        base_dir: impl Into<PathBuf>,
        schema: Vec<String>,
        config: TieredStorageConfig,
    ) -> Self {
        let tier_stats = vec![
            TierStats {
                tier: StorageTier::L0,
                bytes_used: 0,
                file_count: 0,
                row_count: 0,
            },
            TierStats {
                tier: StorageTier::L1,
                bytes_used: 0,
                file_count: 0,
                row_count: 0,
            },
            TierStats {
                tier: StorageTier::L2,
                bytes_used: 0,
                file_count: 0,
                row_count: 0,
            },
            TierStats {
                tier: StorageTier::L3,
                bytes_used: 0,
                file_count: 0,
                row_count: 0,
            },
            TierStats {
                tier: StorageTier::L4,
                bytes_used: 0,
                file_count: 0,
                row_count: 0,
            },
        ];
        Self {
            config,
            l0_rows: Vec::new(),
            l0_bytes: 0,
            base_dir: base_dir.into(),
            schema,
            tier_stats,
            next_file_id: 0,
        }
    }

    /// Append a row to the L0 buffer.
    pub fn write_row(&mut self, pk: u64, values: Vec<u64>) {
        self.l0_bytes += (1 + values.len() as u64) * 8;
        self.l0_rows.push((pk, values));
    }

    /// Flush the L0 buffer to the given tier, returning the path written.
    pub fn flush_to_tier(&mut self, tier: StorageTier) -> StorageResult<PathBuf> {
        if self.l0_rows.is_empty() {
            return Err(StorageError::EmptyPartition);
        }

        let file_id = self.next_file_id;
        self.next_file_id = self.next_file_id.wrapping_add(1);

        let row_count = self.l0_rows.len();
        let mut partition = MicroPartition::new(file_id, 0);

        // pk column
        let pks: Vec<u64> = self.l0_rows.iter().map(|(pk, _)| *pk).collect();
        partition.push_column("pk", ColumnValues::Raw(pks))?;

        // value columns — one per schema entry after the pk
        let col_count = self.l0_rows.first().map(|(_, v)| v.len()).unwrap_or(0);
        for col_idx in 0..col_count {
            let col_name = self
                .schema
                .get(col_idx + 1)
                .cloned()
                .unwrap_or_else(|| format!("col{}", col_idx));
            let vals: Vec<u64> = self
                .l0_rows
                .iter()
                .map(|(_, v)| v.get(col_idx).copied().unwrap_or(0))
                .collect();
            partition.push_column(&col_name, ColumnValues::auto_encode(vals))?;
        }

        let tier_idx = tier as usize;
        let opts = WriteOptions {
            compression: self.config.tiers[tier_idx].compression,
        };

        let filename = format!("{:05}_{}.vvsst", file_id, tier_label(tier));
        let path = self.base_dir.join(&filename);
        std::fs::create_dir_all(&self.base_dir)?;

        let mut file = std::fs::File::create(&path)?;
        write_partition_with(&mut file, &partition, opts)?;

        let file_size = path.metadata()?.len();

        self.tier_stats[tier_idx].bytes_used += file_size;
        self.tier_stats[tier_idx].file_count += 1;
        self.tier_stats[tier_idx].row_count += row_count as u64;

        self.l0_rows.clear();
        self.l0_bytes = 0;

        Ok(path)
    }

    /// Flush to L1 if the L0 byte budget is exhausted.
    pub fn maybe_flush(&mut self) -> StorageResult<Option<PathBuf>> {
        if self.l0_bytes >= self.config.tiers[0].max_bytes {
            let path = self.flush_to_tier(StorageTier::L1)?;
            Ok(Some(path))
        } else {
            Ok(None)
        }
    }

    pub fn tier_stats(&self) -> &[TierStats] {
        &self.tier_stats
    }

    /// Number of rows currently buffered in L0.
    pub fn buffered_rows(&self) -> usize {
        self.l0_rows.len()
    }

    /// Estimated bytes buffered in L0.
    pub fn buffered_bytes(&self) -> u64 {
        self.l0_bytes
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn simple_writer(dir: &TempDir) -> TieredWriter {
        TieredWriter::new(
            dir.path(),
            vec!["pk".into(), "val".into()],
            TieredStorageConfig::default(),
        )
    }

    #[test]
    fn new_has_zero_stats() {
        let dir = TempDir::new().unwrap();
        let w = simple_writer(&dir);
        assert_eq!(w.tier_stats().len(), 5);
        assert_eq!(w.tier_stats()[0].bytes_used, 0);
    }

    #[test]
    fn write_row_increments_bytes() {
        let dir = TempDir::new().unwrap();
        let mut w = simple_writer(&dir);
        w.write_row(1, vec![100]);
        assert!(w.buffered_bytes() > 0);
        assert_eq!(w.buffered_rows(), 1);
    }

    #[test]
    fn flush_empty_errors() {
        let dir = TempDir::new().unwrap();
        let mut w = simple_writer(&dir);
        let result = w.flush_to_tier(StorageTier::L0);
        assert!(matches!(result, Err(StorageError::EmptyPartition)));
    }

    #[test]
    fn flush_to_l0_creates_file() {
        let dir = TempDir::new().unwrap();
        let mut w = simple_writer(&dir);
        w.write_row(1, vec![10]);
        let path = w.flush_to_tier(StorageTier::L0).unwrap();
        assert!(path.exists());
        let name = path.file_name().unwrap().to_str().unwrap();
        assert!(name.ends_with("_l0.vvsst"));
    }

    #[test]
    fn flush_to_l1_creates_file() {
        let dir = TempDir::new().unwrap();
        let mut w = simple_writer(&dir);
        w.write_row(1, vec![10]);
        let path = w.flush_to_tier(StorageTier::L1).unwrap();
        let name = path.file_name().unwrap().to_str().unwrap();
        assert!(name.ends_with("_l1.vvsst"));
    }

    #[test]
    fn flush_clears_buffer() {
        let dir = TempDir::new().unwrap();
        let mut w = simple_writer(&dir);
        w.write_row(1, vec![10]);
        w.flush_to_tier(StorageTier::L0).unwrap();
        assert_eq!(w.buffered_rows(), 0);
        assert_eq!(w.buffered_bytes(), 0);
    }

    #[test]
    fn flush_updates_tier_stats() {
        let dir = TempDir::new().unwrap();
        let mut w = simple_writer(&dir);
        w.write_row(1, vec![10]);
        w.write_row(2, vec![20]);
        w.flush_to_tier(StorageTier::L1).unwrap();
        assert_eq!(w.tier_stats()[1].file_count, 1);
        assert_eq!(w.tier_stats()[1].row_count, 2);
        assert!(w.tier_stats()[1].bytes_used > 0);
    }

    #[test]
    fn sequential_flushes_increment_file_id() {
        let dir = TempDir::new().unwrap();
        let mut w = simple_writer(&dir);
        w.write_row(1, vec![10]);
        let p1 = w.flush_to_tier(StorageTier::L0).unwrap();
        w.write_row(2, vec![20]);
        let p2 = w.flush_to_tier(StorageTier::L0).unwrap();
        assert_ne!(p1, p2);
    }

    #[test]
    fn maybe_flush_none_below_threshold() {
        let dir = TempDir::new().unwrap();
        let mut w = simple_writer(&dir);
        w.write_row(1, vec![10]);
        let result = w.maybe_flush().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn maybe_flush_some_above_threshold() {
        let dir = TempDir::new().unwrap();
        let mut cfg = TieredStorageConfig::default();
        cfg.tiers[0].max_bytes = 1; // force immediate flush
        let mut w = TieredWriter::new(dir.path(), vec!["pk".into(), "v".into()], cfg);
        w.write_row(1, vec![99]);
        let result = w.maybe_flush().unwrap();
        assert!(result.is_some());
        let path = result.unwrap();
        let name = path.file_name().unwrap().to_str().unwrap();
        assert!(
            name.ends_with("_l1.vvsst"),
            "expected L1 file, got {}",
            name
        );
    }

    #[test]
    fn flush_to_l4_creates_file() {
        let dir = TempDir::new().unwrap();
        let mut w = simple_writer(&dir);
        w.write_row(1, vec![42]);
        let path = w.flush_to_tier(StorageTier::L4).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn multi_col_schema_writes_correctly() {
        let dir = TempDir::new().unwrap();
        let mut w = TieredWriter::new(
            dir.path(),
            vec!["pk".into(), "a".into(), "b".into(), "c".into()],
            TieredStorageConfig::default(),
        );
        w.write_row(1, vec![10, 20, 30]);
        let path = w.flush_to_tier(StorageTier::L0).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn file_named_with_zero_padded_id() {
        let dir = TempDir::new().unwrap();
        let mut w = simple_writer(&dir);
        w.write_row(1, vec![10]);
        let path = w.flush_to_tier(StorageTier::L2).unwrap();
        let name = path.file_name().unwrap().to_str().unwrap();
        assert!(
            name.starts_with("00000"),
            "expected zero-padded id in {}",
            name
        );
    }
}
