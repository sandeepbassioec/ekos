//! Distinct-value enumeration and frequency counting for a single column.
//!
//! ## Functions
//!
//! | Method | Returns |
//! |--------|---------|
//! | `distinct_values(col, lo, hi)` | Sorted unique values of `col` where `col ∈ [lo, hi]` |
//! | `value_counts(col, lo, hi)` | `(value, count)` pairs sorted by value ascending |
//!
//! ## Correctness: scan + get pattern
//!
//! `scan()` can return the same PK from multiple partition files when a row
//! has been updated but not yet compacted (stale physical bytes remain in the
//! old partition).  A raw scan therefore cannot be used as the source of
//! truth for a row's current values.
//!
//! Both functions follow the engine-wide pattern documented in the invariants:
//! *"Scan returns tombstoned rows — always filter with `engine.get()` check."*
//!
//! 1. **Candidate PKs**: `scan(col, 0, MAX)` collects every unique PK that
//!    appears in any partition (with zone pruning).
//! 2. **Authoritative lookup**: for each unique PK, `get(pk)` returns the
//!    current row via the pk-index — the only source of truth for which
//!    partition file and row offset hold the live data.
//! 3. **Filter and collect**: the range `[lo, hi]` is applied to the
//!    authoritative value, so a row updated from an in-range value to an
//!    out-of-range value is correctly excluded.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::EngineResult;
use crate::engine::TableEngine;

impl TableEngine {
    /// Return sorted unique current values of `col_idx` where the value is
    /// in `[lo, hi]`.
    pub fn distinct_values(&self, col_idx: usize, lo: u64, hi: u64) -> EngineResult<Vec<u64>> {
        self.check_col(col_idx)?;
        if lo > hi {
            return Ok(Vec::new());
        }

        let mut seen: HashSet<u64> = HashSet::new();
        let mut result: BTreeSet<u64> = BTreeSet::new();

        // scan(col_idx, 0, MAX) gives us all rows (possibly with stale dupes).
        // We only need one call to get() per unique PK.
        for row in self.scan(col_idx, 0, u64::MAX)? {
            if seen.insert(row.pk)
                && let Some(current) = self.get(row.pk)?
                && current.values[col_idx] >= lo
                && current.values[col_idx] <= hi
            {
                result.insert(current.values[col_idx]);
            }
        }
        Ok(result.into_iter().collect())
    }

    /// Return `(value, count)` pairs for every distinct current value of
    /// `col_idx` in `[lo, hi]`, sorted by value ascending.
    ///
    /// Each logical row (deduplicated by PK) contributes exactly once.
    pub fn value_counts(&self, col_idx: usize, lo: u64, hi: u64) -> EngineResult<Vec<(u64, u64)>> {
        self.check_col(col_idx)?;
        if lo > hi {
            return Ok(Vec::new());
        }

        let mut seen: HashSet<u64> = HashSet::new();
        let mut counts: BTreeMap<u64, u64> = BTreeMap::new();

        for row in self.scan(col_idx, 0, u64::MAX)? {
            if seen.insert(row.pk)
                && let Some(current) = self.get(row.pk)?
            {
                let val = current.values[col_idx];
                if val >= lo && val <= hi {
                    *counts.entry(val).or_insert(0) += 1;
                }
            }
        }
        Ok(counts.into_iter().collect())
    }
}
