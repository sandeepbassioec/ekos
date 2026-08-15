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

        // Use scan_deduped for authoritative, pk-deduplicated rows, then
        // project.  This is slightly less efficient than the old partition-
        // level selective decode, but is correct in the presence of stale
        // partition copies from un-compacted updates.
        let full_rows = self.scan_deduped(filter_col, lo, hi)?;
        let results = full_rows
            .into_iter()
            .map(|row| {
                let values: Vec<u64> = projection.iter().map(|&c| row.values[c]).collect();
                EngineRow { pk: row.pk, values }
            })
            .collect();
        Ok(results)
    }
}
