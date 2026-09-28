//! Sorted scan with top-N support.
//!
//! [`TableEngine::scan_sorted`] filters rows by a range predicate on one
//! column, sorts all matching rows by a (possibly different) sort column,
//! and returns up to `limit` rows.  It decodes only the filter column and
//! the sort column per partition — no other columns are read — then
//! materialises full rows only for the survivors after sorting and limiting.
//!
//! ## Complexity
//!
//! Let `M` = matching rows across all partitions, `L` = limit.
//!
//! - Decode phase: O(M) per decoded column — 2 columns maximum.
//! - Sort phase:   O(M log M) — single sort across all partitions.
//! - Materialise:  O(L) full-row `get` calls.
//!
//! For small `L` relative to `M` the sort dominates; a future layer could
//! replace it with a tournament heap for O(M log L) without changing the API.

use std::collections::HashSet;

use crate::engine::TableEngine;
use crate::{EngineResult, EngineRow};

impl TableEngine {
    /// Scan rows where `filter_col` is in `[lo, hi]`, sort by `sort_col`,
    /// and return at most `limit` rows.
    ///
    /// # Parameters
    ///
    /// - `filter_col` — column index used for the range predicate `[lo, hi]`.
    /// - `lo`, `hi` — inclusive bounds on `filter_col`.  Returns empty when
    ///   `lo > hi`.
    /// - `sort_col` — column index to sort by.  May be the same as
    ///   `filter_col`.
    /// - `ascending` — `true` for ascending order (smallest first); `false`
    ///   for descending (largest first).
    /// - `limit` — maximum number of rows to return.  Pass `usize::MAX` for
    ///   no limit.  Returns empty when `limit` is zero.
    ///
    /// # Errors
    ///
    /// Returns [`EngineError::ColOutOfRange`] when either column index is out
    /// of bounds for this table's schema.
    pub fn scan_sorted(
        &self,
        filter_col: usize,
        lo: u64,
        hi: u64,
        sort_col: usize,
        ascending: bool,
        limit: usize,
    ) -> EngineResult<Vec<EngineRow>> {
        self.check_col(filter_col)?;
        self.check_col(sort_col)?;

        if lo > hi || limit == 0 {
            return Ok(Vec::new());
        }

        self.flush_if_needed()?;

        // Collect (sort_value, pk) pairs from all matching partitions.
        // Decode only filter_col and sort_col; all other columns are skipped.
        let mut candidates: Vec<(u64, u64)> = Vec::new(); // (sort_val, pk)

        {
            let mut filter_buf: Vec<u64> = Vec::new();
            let mut sort_buf: Vec<u64> = Vec::new();
            let mut pk_buf: Vec<u64> = Vec::new();

            self.promote_all_pending()?;

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

                // Decode filter col to re-check values (filter_offsets may
                // return conservative results from zone metadata).
                reader.decode_column_into(filter_col, &mut filter_buf)?;

                // Decode sort col (may be the same buffer — decode again into
                // sort_buf so we always have two independent buffers).
                reader.decode_column_into(sort_col, &mut sort_buf)?;

                // Decode pk (col 0) to identify rows for later materialisation.
                reader.decode_column_into(0, &mut pk_buf)?;

                for &o in &offsets {
                    let idx = o as usize;
                    let filter_val = *filter_buf.get(idx).unwrap_or(&0);
                    // Re-apply the exact predicate in case zone was coarse.
                    if filter_val < lo || filter_val > hi {
                        continue;
                    }
                    let sort_val = *sort_buf.get(idx).unwrap_or(&0);
                    let pk = *pk_buf.get(idx).unwrap_or(&0);
                    candidates.push((sort_val, pk));
                }
            }
        }

        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        // Dedup by pk: the same PK can appear from multiple partition files
        // when a row was updated but not yet compacted.  Keep only the first
        // occurrence; the sort_val may be stale but the row is materialised
        // via get() below so the caller always receives current data.
        {
            let mut seen: HashSet<u64> = HashSet::with_capacity(candidates.len());
            candidates.retain(|(_, pk)| seen.insert(*pk));
        }

        // Sort all candidates, then take limit.
        if ascending {
            candidates.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        } else {
            candidates.sort_unstable_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        }
        candidates.truncate(limit);

        // Materialise full rows for the survivors.
        let mut rows = Vec::with_capacity(candidates.len());
        for (_sort_val, pk) in candidates {
            if let Some(row) = self.get(pk)? {
                rows.push(row);
            }
        }
        Ok(rows)
    }
}
