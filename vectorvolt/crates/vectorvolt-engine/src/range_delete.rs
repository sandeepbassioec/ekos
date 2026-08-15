//! Bulk range-delete.
//!
//! [`TableEngine::delete_range`] deletes every row where `filter_col` is in
//! `[lo, hi]`, returning the number of rows removed.
//!
//! ## Architecture
//!
//! 1. **Flush** — ensures all buffered writes are visible to the column-store
//!    read path before the scan.
//! 2. **Zone-prune** — skip partitions whose `filter_col` zone-min/max has no
//!    overlap with `[lo, hi]`.
//! 3. **Two-column decode** — decode only the filter column (to re-check the
//!    exact predicate) and the PK column (col 0, needed to call `delete`).
//!    No other columns are decoded; no full rows are materialised.
//! 4. **Delete** — call the single-row `delete` for each matched PK.
//!    `PkNotFound` is silently ignored (another thread may have already
//!    deleted the row); any other error propagates immediately.
//!
//! ## Atomicity
//!
//! Each individual `delete` is atomic, but the range as a whole is not: if
//! `delete_range` returns an error mid-way, rows deleted before the error are
//! gone and rows after it are still present.  Wrap in a transaction if
//! all-or-nothing semantics are required.

use crate::EngineResult;
use crate::engine::TableEngine;

impl TableEngine {
    /// Delete every row where `filter_col` is in `[lo, hi]`.
    ///
    /// Returns the number of rows deleted.  Returns `0` immediately when
    /// `lo > hi`.  Rows that disappear between the scan and the delete (e.g.
    /// deleted by a concurrent writer) are not counted.
    ///
    /// # Errors
    ///
    /// Returns [`crate::EngineError::ColOutOfRange`] when `filter_col` is not
    /// a valid column index.  Any storage I/O error during the scan also
    /// propagates.
    pub fn delete_range(&self, filter_col: usize, lo: u64, hi: u64) -> EngineResult<usize> {
        self.check_col(filter_col)?;

        if lo > hi {
            return Ok(0);
        }

        self.flush_if_needed()?;

        // Collect all matching PKs first, then delete.
        // We release the partitions read-lock before calling delete() so that
        // delete() can acquire its own write locks without risk of inversion.
        let pks_to_delete: Vec<u64> = {
            let mut filter_buf: Vec<u64> = Vec::new();
            let mut pk_buf: Vec<u64> = Vec::new();
            let mut pks: Vec<u64> = Vec::new();

            let parts = self.partitions.read();
            for reader in parts.values() {
                // Zone-prune on filter_col.
                if let Ok((min, max)) = reader.zone_min_max(filter_col)
                    && (min > hi || max < lo)
                {
                    continue;
                }

                let offsets = reader.filter_offsets(filter_col, lo, hi)?;
                if offsets.is_empty() {
                    continue;
                }

                reader.decode_column_into(filter_col, &mut filter_buf)?;
                reader.decode_column_into(0, &mut pk_buf)?;

                for &o in &offsets {
                    let idx = o as usize;
                    let fv = *filter_buf.get(idx).unwrap_or(&0);
                    // Re-apply the exact predicate (zone metadata may be coarser).
                    if fv < lo || fv > hi {
                        continue;
                    }
                    let pk = *pk_buf.get(idx).unwrap_or(&0);
                    pks.push(pk);
                }
            }
            pks
        };

        // Delete each matched PK.  PkNotFound means a concurrent delete beat
        // us — not an error from the caller's perspective.
        let mut deleted = 0usize;
        for pk in pks_to_delete {
            match self.delete(pk) {
                Ok(_) => deleted += 1,
                Err(crate::EngineError::Txn(vectorvolt_txn::TxnError::PkNotFound(_))) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(deleted)
    }
}
