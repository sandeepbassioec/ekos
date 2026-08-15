//! Predicate-filtered bulk field updates.
//!
//! These three methods mirror [`TableEngine::update_field`],
//! [`TableEngine::increment_field`], and [`TableEngine::decrement_field`]
//! but apply to **every row that matches a set of AND predicates** rather
//! than a single primary key.
//!
//! ## Atomicity
//!
//! Each individual row update acquires `cas_lock` exactly as the single-row
//! variants do, so no concurrent writer can interleave within a single row's
//! read-modify-write.  The bulk operation as a whole is **not** atomic across
//! all rows — a snapshot of matching PKs is taken first, then each PK is
//! updated independently.  Rows inserted between the scan and the update pass
//! are not affected.  Rows deleted between the scan and their update are
//! silently skipped (the single-row `update_field` / `increment_field` /
//! `decrement_field` returns `Ok(None)` for missing PKs, which is counted
//! as zero here).
//!
//! ## Predicate semantics
//!
//! Identical to [`TableEngine::scan_where`]:
//! - All predicates are ANDed together.
//! - An empty slice matches every row.
//! - Any predicate with `lo > hi` matches nothing (returns `Ok(0)`).
//!
//! ## Return value
//!
//! The number of rows successfully updated (PKs that existed at update time).

use crate::EngineResult;
use crate::engine::TableEngine;

impl TableEngine {
    /// Set `values[target_col] = new_val` for every row matching `predicates`.
    ///
    /// Returns the number of rows updated.
    pub fn update_field_where(
        &self,
        predicates: &[(usize, u64, u64)],
        target_col: usize,
        new_val: u64,
    ) -> EngineResult<usize> {
        self.check_col(target_col)?;
        let pks = self.matching_pks(predicates)?;
        let mut count = 0;
        for pk in pks {
            if self.update_field(pk, target_col, new_val)?.is_some() {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Add `delta` (wrapping) to `values[target_col]` for every matching row.
    ///
    /// Returns the number of rows updated.
    pub fn increment_field_where(
        &self,
        predicates: &[(usize, u64, u64)],
        target_col: usize,
        delta: u64,
    ) -> EngineResult<usize> {
        self.check_col(target_col)?;
        let pks = self.matching_pks(predicates)?;
        let mut count = 0;
        for pk in pks {
            if self.increment_field(pk, target_col, delta)?.is_some() {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Subtract `delta` (saturating at 0) from `values[target_col]` for every matching row.
    ///
    /// Returns the number of rows updated.
    pub fn decrement_field_where(
        &self,
        predicates: &[(usize, u64, u64)],
        target_col: usize,
        delta: u64,
    ) -> EngineResult<usize> {
        self.check_col(target_col)?;
        let pks = self.matching_pks(predicates)?;
        let mut count = 0;
        for pk in pks {
            if self.decrement_field(pk, target_col, delta)?.is_some() {
                count += 1;
            }
        }
        Ok(count)
    }

    // ── Private ───────────────────────────────────────────────────────────────

    fn matching_pks(&self, predicates: &[(usize, u64, u64)]) -> EngineResult<Vec<u64>> {
        // scan_where (like scan) may return the same PK from multiple partition
        // files when the row has been updated since the last compaction — the
        // old partition still holds the physical bytes. Dedup so each PK is
        // updated exactly once.
        let mut pks: Vec<u64> = self
            .scan_where(predicates)?
            .into_iter()
            .map(|r| r.pk)
            .collect();
        pks.sort_unstable();
        pks.dedup();
        Ok(pks)
    }
}
