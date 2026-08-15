//! Column-store aggregate pushdown.
//!
//! `count_range`, `sum_range`, `min_range`, `max_range`, and `avg_range` each
//! decode **only the target column** per partition and compute the aggregate
//! directly, never materialising full rows.  This is the core column-store
//! advantage: a query touching 1 of 20 columns pays 1/20th of the I/O even
//! when the result set is large.
//!
//! ## Architecture
//!
//! Each aggregate follows the same three-phase loop:
//!
//! 1. **Zone prune** — skip the partition if its zone-min/max has no overlap
//!    with `[lo, hi]`.  For `count_range` this eliminates all I/O.
//! 2. **Single-column decode** — call `filter_offsets` to get matching row
//!    positions inside this partition, then `decode_column_into` to decode just
//!    the scan column.  No other column is touched.
//! 3. **Scalar aggregate** — gather values at matched offsets and fold them.

use crate::engine::TableEngine;
use crate::{EngineError, EngineResult};

impl TableEngine {
    // ── helpers ───────────────────────────────────────────────────────────────

    fn check_col(&self, col_idx: usize) -> EngineResult<()> {
        let col_count = self.schema.len();
        if col_idx >= col_count {
            return Err(EngineError::ColOutOfRange { col_idx, col_count });
        }
        Ok(())
    }

    /// Flush the write buffer if any buffered rows exist.
    fn flush_if_needed(&self) -> EngineResult<()> {
        if self.txn.row_count_buffered() > 0 {
            self.do_flush()?;
        }
        Ok(())
    }

    // ── public aggregate API ──────────────────────────────────────────────────

    /// Count rows where `col_idx` is in `[lo, hi]`.
    ///
    /// Uses zone pruning to skip partitions entirely and decodes only the
    /// target column — **no full-row materialisation**.
    pub fn count_range(&self, col_idx: usize, lo: u64, hi: u64) -> EngineResult<u64> {
        self.check_col(col_idx)?;
        self.flush_if_needed()?;

        let mut count = 0u64;
        let parts = self.partitions.read();
        for reader in parts.values() {
            if let Ok((min, max)) = reader.zone_min_max(col_idx)
                && (min > hi || max < lo)
            {
                continue; // zone prune
            }
            let offsets = reader.filter_offsets(col_idx, lo, hi)?;
            count += offsets.len() as u64;
        }
        Ok(count)
    }

    /// Sum values in `col_idx` for rows where `col_idx` is in `[lo, hi]`.
    ///
    /// Returns `0` for an empty result set.  Wraps on `u64` overflow.
    pub fn sum_range(&self, col_idx: usize, lo: u64, hi: u64) -> EngineResult<u64> {
        self.check_col(col_idx)?;
        self.flush_if_needed()?;

        let mut total = 0u64;
        let mut col_buf = Vec::new();
        let parts = self.partitions.read();
        for reader in parts.values() {
            if let Ok((min, max)) = reader.zone_min_max(col_idx)
                && (min > hi || max < lo)
            {
                continue;
            }
            let offsets = reader.filter_offsets(col_idx, lo, hi)?;
            if offsets.is_empty() {
                continue;
            }
            reader.decode_column_into(col_idx, &mut col_buf)?;
            for &o in &offsets {
                total = total.wrapping_add(*col_buf.get(o as usize).unwrap_or(&0));
            }
        }
        Ok(total)
    }

    /// Minimum value in `col_idx` for rows where `col_idx` is in `[lo, hi]`.
    ///
    /// Returns `None` if no rows match.
    pub fn min_range(&self, col_idx: usize, lo: u64, hi: u64) -> EngineResult<Option<u64>> {
        self.check_col(col_idx)?;
        self.flush_if_needed()?;

        let mut result: Option<u64> = None;
        let mut col_buf = Vec::new();
        let parts = self.partitions.read();
        for reader in parts.values() {
            if let Ok((min, max)) = reader.zone_min_max(col_idx)
                && (min > hi || max < lo)
            {
                continue;
            }
            let offsets = reader.filter_offsets(col_idx, lo, hi)?;
            if offsets.is_empty() {
                continue;
            }
            reader.decode_column_into(col_idx, &mut col_buf)?;
            for &o in &offsets {
                let v = *col_buf.get(o as usize).unwrap_or(&u64::MAX);
                result = Some(match result {
                    None => v,
                    Some(cur) => cur.min(v),
                });
            }
        }
        Ok(result)
    }

    /// Maximum value in `col_idx` for rows where `col_idx` is in `[lo, hi]`.
    ///
    /// Returns `None` if no rows match.
    pub fn max_range(&self, col_idx: usize, lo: u64, hi: u64) -> EngineResult<Option<u64>> {
        self.check_col(col_idx)?;
        self.flush_if_needed()?;

        let mut result: Option<u64> = None;
        let mut col_buf = Vec::new();
        let parts = self.partitions.read();
        for reader in parts.values() {
            if let Ok((min, max)) = reader.zone_min_max(col_idx)
                && (min > hi || max < lo)
            {
                continue;
            }
            let offsets = reader.filter_offsets(col_idx, lo, hi)?;
            if offsets.is_empty() {
                continue;
            }
            reader.decode_column_into(col_idx, &mut col_buf)?;
            for &o in &offsets {
                let v = *col_buf.get(o as usize).unwrap_or(&0);
                result = Some(match result {
                    None => v,
                    Some(cur) => cur.max(v),
                });
            }
        }
        Ok(result)
    }

    /// Average of `col_idx` for rows where `col_idx` is in `[lo, hi]`.
    ///
    /// Returns `None` if no rows match.
    pub fn avg_range(&self, col_idx: usize, lo: u64, hi: u64) -> EngineResult<Option<f64>> {
        self.check_col(col_idx)?;
        self.flush_if_needed()?;

        let mut total = 0u64;
        let mut count = 0u64;
        let mut col_buf = Vec::new();
        let parts = self.partitions.read();
        for reader in parts.values() {
            if let Ok((min, max)) = reader.zone_min_max(col_idx)
                && (min > hi || max < lo)
            {
                continue;
            }
            let offsets = reader.filter_offsets(col_idx, lo, hi)?;
            if offsets.is_empty() {
                continue;
            }
            reader.decode_column_into(col_idx, &mut col_buf)?;
            for &o in &offsets {
                total = total.wrapping_add(*col_buf.get(o as usize).unwrap_or(&0));
                count += 1;
            }
        }
        if count == 0 {
            Ok(None)
        } else {
            Ok(Some(total as f64 / count as f64))
        }
    }
}
