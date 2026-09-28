//! Column-store aggregate pushdown.
//!
//! Scalar aggregates (`count_range` / `sum_range` / `min_range` / `max_range` /
//! `avg_range`) and analytical aggregates (`histogram_range` /
//! `percentile_range`) each decode **only the target column** per partition,
//! never materialising full rows.
//!
//! ## Architecture
//!
//! Every aggregate follows the same three-phase loop:
//!
//! 1. **Zone prune** — skip the partition if its zone-min/max has no overlap
//!    with `[lo, hi]`.  For `count_range` this eliminates all I/O.
//! 2. **Single-column decode** — call `filter_offsets` to get matching row
//!    positions inside this partition, then `decode_column_into` to decode just
//!    the scan column.  No other column is touched.
//! 3. **Scalar/analytical aggregate** — gather values at matched offsets and fold.

/// One bucket in a [`TableEngine::histogram_range`] result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistogramBucket {
    /// Inclusive lower bound of this bucket.
    pub lo: u64,
    /// Inclusive upper bound of this bucket.
    pub hi: u64,
    /// Number of column values that fall in `[lo, hi]`.
    pub count: u64,
}

use crate::engine::TableEngine;
use crate::{EngineError, EngineResult};

impl TableEngine {
    // ── helpers ───────────────────────────────────────────────────────────────

    pub(crate) fn check_col(&self, col_idx: usize) -> EngineResult<()> {
        let col_count = self.schema.col_count();
        if col_idx >= col_count {
            return Err(EngineError::ColOutOfRange { col_idx, col_count });
        }
        Ok(())
    }

    /// Flush the write buffer if any buffered rows exist.
    pub(crate) fn flush_if_needed(&self) -> EngineResult<()> {
        if self.txn.row_count_buffered() > 0 {
            self.do_flush()?;
        }
        Ok(())
    }

    // ── public aggregate API ──────────────────────────────────────────────────

    /// Count rows where `col_idx` is in `[lo, hi]`.
    ///
    /// Uses `scan_deduped` so each logical row is counted exactly once even
    /// when stale partition copies exist from un-compacted updates.
    pub fn count_range(&self, col_idx: usize, lo: u64, hi: u64) -> EngineResult<u64> {
        self.check_col(col_idx)?;
        Ok(self.scan_deduped(col_idx, lo, hi)?.len() as u64)
    }

    /// Sum values in `col_idx` for rows where `col_idx` is in `[lo, hi]`.
    ///
    /// Returns `0` for an empty result set.  Wraps on `u64` overflow.
    pub fn sum_range(&self, col_idx: usize, lo: u64, hi: u64) -> EngineResult<u64> {
        self.check_col(col_idx)?;
        Ok(self
            .scan_deduped(col_idx, lo, hi)?
            .iter()
            .fold(0u64, |acc, r| acc.wrapping_add(r.values[col_idx])))
    }

    /// Minimum value in `col_idx` for rows where `col_idx` is in `[lo, hi]`.
    ///
    /// Returns `None` if no rows match.
    pub fn min_range(&self, col_idx: usize, lo: u64, hi: u64) -> EngineResult<Option<u64>> {
        self.check_col(col_idx)?;
        Ok(self
            .scan_deduped(col_idx, lo, hi)?
            .iter()
            .map(|r| r.values[col_idx])
            .reduce(u64::min))
    }

    /// Maximum value in `col_idx` for rows where `col_idx` is in `[lo, hi]`.
    ///
    /// Returns `None` if no rows match.
    pub fn max_range(&self, col_idx: usize, lo: u64, hi: u64) -> EngineResult<Option<u64>> {
        self.check_col(col_idx)?;
        Ok(self
            .scan_deduped(col_idx, lo, hi)?
            .iter()
            .map(|r| r.values[col_idx])
            .reduce(u64::max))
    }

    /// Average of `col_idx` for rows where `col_idx` is in `[lo, hi]`.
    ///
    /// Returns `None` if no rows match.
    pub fn avg_range(&self, col_idx: usize, lo: u64, hi: u64) -> EngineResult<Option<f64>> {
        self.check_col(col_idx)?;
        let rows = self.scan_deduped(col_idx, lo, hi)?;
        if rows.is_empty() {
            return Ok(None);
        }
        let total: u64 = rows
            .iter()
            .fold(0u64, |acc, r| acc.wrapping_add(r.values[col_idx]));
        Ok(Some(total as f64 / rows.len() as f64))
    }

    // ── analytical aggregates ─────────────────────────────────────────────────

    /// Divide `[lo, hi]` into `n_buckets` equal-width buckets and count how
    /// many values in `col_idx` fall into each bucket.
    ///
    /// Bucket boundaries are computed from `lo`/`hi` alone; only rows whose
    /// `col_idx` value falls in `[lo, hi]` are counted.  Each bucket is
    /// `[bucket_lo, bucket_hi]` inclusive (the last bucket absorbs any
    /// rounding remainder so the final bucket always ends exactly at `hi`).
    ///
    /// Returns an empty `Vec` when `n_buckets` is zero or `lo > hi`.
    /// Only the target column is decoded per partition — no full-row I/O.
    pub fn histogram_range(
        &self,
        col_idx: usize,
        lo: u64,
        hi: u64,
        n_buckets: usize,
    ) -> EngineResult<Vec<HistogramBucket>> {
        self.check_col(col_idx)?;
        if n_buckets == 0 || lo > hi {
            return Ok(Vec::new());
        }
        self.flush_if_needed()?;

        // Build bucket boundary array: n_buckets + 1 edges.
        let span = hi - lo;
        let width = span / n_buckets as u64; // may be 0 when hi-lo < n_buckets
        let mut edges: Vec<u64> = (0..n_buckets as u64)
            .map(|i| lo + i * width.max(1))
            .collect();
        edges.push(hi + 1); // sentinel past the end

        let mut counts = vec![0u64; n_buckets];

        let mut col_buf = Vec::new();
        self.promote_all_pending()?;
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
                // Binary search for the bucket this value belongs to.
                // edges[i] <= v < edges[i+1]  →  bucket i
                // The last bucket edge is hi+1 so values == hi land in bucket n-1.
                let bucket = edges
                    .partition_point(|&edge| edge <= v)
                    .saturating_sub(1)
                    .min(n_buckets - 1);
                counts[bucket] += 1;
            }
        }

        // Build result: last bucket always ends at hi.
        let buckets = (0..n_buckets)
            .map(|i| {
                let b_lo = edges[i];
                let b_hi = if i + 1 == n_buckets {
                    hi
                } else {
                    edges[i + 1] - 1
                };
                HistogramBucket {
                    lo: b_lo,
                    hi: b_hi,
                    count: counts[i],
                }
            })
            .collect();
        Ok(buckets)
    }

    /// Return the `percentile`-th percentile value of `col_idx` for rows where
    /// `col_idx` is in `[lo, hi]`.
    ///
    /// `percentile` must be in `[0.0, 100.0]`.  Uses the "nearest rank" method:
    /// the value at index `⌈(p/100) * n⌉ - 1` in the sorted list of matching
    /// values.  P0 returns the minimum; P100 returns the maximum.
    ///
    /// Returns `None` when no rows match, `Err` when `percentile` is out of
    /// range or `col_idx` is invalid.
    pub fn percentile_range(
        &self,
        col_idx: usize,
        lo: u64,
        hi: u64,
        percentile: f64,
    ) -> EngineResult<Option<u64>> {
        if !(0.0..=100.0).contains(&percentile) {
            return Err(EngineError::ColOutOfRange {
                col_idx: 0,
                col_count: 0,
            });
        }
        self.check_col(col_idx)?;
        self.flush_if_needed()?;

        let mut values: Vec<u64> = Vec::new();
        let mut col_buf = Vec::new();
        self.promote_all_pending()?;
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
                values.push(*col_buf.get(o as usize).unwrap_or(&0));
            }
        }

        if values.is_empty() {
            return Ok(None);
        }

        values.sort_unstable();
        let n = values.len();
        // Nearest-rank: index = ceil(p/100 * n) - 1, clamped to [0, n-1].
        let idx = if percentile == 0.0 {
            0
        } else {
            let rank = (percentile / 100.0 * n as f64).ceil() as usize;
            rank.saturating_sub(1).min(n - 1)
        };
        Ok(Some(values[idx]))
    }
}
