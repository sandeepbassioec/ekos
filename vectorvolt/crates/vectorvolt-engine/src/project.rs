//! Column projection for `scan`.
//!
//! [`TableEngine::scan_project`] filters rows by a column range and then
//! decodes **only the requested columns**, leaving all others untouched.
//!
//! ## Why this matters
//!
//! `scan()` calls `materialize_rows`, which decodes every column for every
//! matched row.  A query that touches 2 of 100 columns pays the full 100-column
//! decode cost.  `scan_project` decodes exactly the columns in `projection`
//! plus the PK column (col 0) — a 98% decode-work reduction in that example.
//!
//! ## API contract
//!
//! - `filter_col` is the column used for the range predicate `[lo, hi]`.
//!   It does **not** need to appear in `projection`.
//! - `projection` is a list of column indices to include in the output.
//!   Duplicates are tolerated; order is preserved.
//! - `EngineRow::pk` is always set from col 0.
//! - `EngineRow::values` contains one entry per element of `projection`,
//!   in the order given, **not** in schema order.
//! - An empty `projection` still returns the matched PKs (values is empty).

use crate::engine::TableEngine;
use crate::{EngineError, EngineResult, EngineRow};

impl TableEngine {
    /// Scan rows where `values[filter_col]` is in `[lo, hi]`, returning only
    /// the columns listed in `projection`.
    ///
    /// `EngineRow::pk` is set from column 0 of each matched row.
    /// `EngineRow::values` contains exactly the projected columns in the order
    /// given by `projection`.
    ///
    /// Only the PK column and the projected columns are decoded per partition.
    /// All other columns are skipped — no I/O, no decompression.
    pub fn scan_project(
        &self,
        filter_col: usize,
        lo: u64,
        hi: u64,
        projection: &[usize],
    ) -> EngineResult<Vec<EngineRow>> {
        let col_count = self.schema.len();

        if filter_col >= col_count {
            return Err(EngineError::ColOutOfRange {
                col_idx: filter_col,
                col_count,
            });
        }
        for &c in projection {
            if c >= col_count {
                return Err(EngineError::ColOutOfRange {
                    col_idx: c,
                    col_count,
                });
            }
        }

        if self.txn.row_count_buffered() > 0 {
            self.do_flush()?;
        }

        // Determine which columns to actually decode.
        // col 0 is always needed for the PK; add it if missing from projection.
        let mut decode_set: Vec<usize> = vec![0];
        for &c in projection {
            if !decode_set.contains(&c) {
                decode_set.push(c);
            }
        }
        if !decode_set.contains(&filter_col) {
            decode_set.push(filter_col);
        }

        let mut results = Vec::new();
        let parts = self.partitions.read();

        for reader in parts.values() {
            // Zone prune on the filter column.
            if let Ok((min, max)) = reader.zone_min_max(filter_col)
                && (min > hi || max < lo)
            {
                continue;
            }

            let offsets = reader.filter_offsets(filter_col, lo, hi)?;
            if offsets.is_empty() {
                continue;
            }

            // Decode only the columns we need.
            let mut bufs: std::collections::HashMap<usize, Vec<u64>> =
                std::collections::HashMap::new();
            let mut tmp = Vec::new();
            for &c in &decode_set {
                reader.decode_column_into(c, &mut tmp)?;
                bufs.insert(c, std::mem::take(&mut tmp));
            }

            let pk_buf = bufs.get(&0).map(Vec::as_slice).unwrap_or(&[]);

            for &o in &offsets {
                let o = o as usize;
                let pk = pk_buf.get(o).copied().unwrap_or(u64::MAX);
                let values: Vec<u64> = projection
                    .iter()
                    .map(|&c| bufs.get(&c).and_then(|b| b.get(o)).copied().unwrap_or(0))
                    .collect();
                results.push(EngineRow { pk, values });
            }
        }

        Ok(results)
    }
}
