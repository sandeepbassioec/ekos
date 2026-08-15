//! Group-by aggregates.
//!
//! Each function filters rows by a range predicate on `filter_col`, groups
//! matching rows by `group_col`, and computes an aggregate over `agg_col`.
//! Results are returned as a `Vec<(group_value, aggregate)>` sorted
//! ascending by `group_value` for deterministic output.
//!
//! ## Dedup correctness
//!
//! All four methods route through `scan_deduped` so each logical row
//! contributes exactly once, even when a row has been updated since the last
//! compaction and its physical bytes appear in more than one partition file.

use std::collections::BTreeMap;

use crate::EngineResult;
use crate::engine::TableEngine;

impl TableEngine {
    // ── internal helper ───────────────────────────────────────────────────────

    fn check_groupby_args(
        &self,
        filter_col: usize,
        group_col: usize,
        agg_col: usize,
    ) -> EngineResult<()> {
        self.check_col(filter_col)?;
        self.check_col(group_col)?;
        self.check_col(agg_col)?;
        Ok(())
    }

    // ── public group-by API ───────────────────────────────────────────────────

    /// Count rows per `group_col` value where `filter_col` is in `[lo, hi]`.
    ///
    /// Returns `Vec<(group_value, count)>` sorted ascending by `group_value`.
    pub fn group_by_count(
        &self,
        filter_col: usize,
        lo: u64,
        hi: u64,
        group_col: usize,
    ) -> EngineResult<Vec<(u64, u64)>> {
        self.check_col(filter_col)?;
        self.check_col(group_col)?;
        if lo > hi {
            return Ok(Vec::new());
        }
        let mut acc: BTreeMap<u64, u64> = BTreeMap::new();
        for row in self.scan_deduped(filter_col, lo, hi)? {
            *acc.entry(row.values[group_col]).or_insert(0) += 1;
        }
        Ok(acc.into_iter().collect())
    }

    /// Sum `agg_col` per `group_col` value where `filter_col` is in `[lo, hi]`.
    ///
    /// Returns `Vec<(group_value, sum)>` sorted ascending by `group_value`.
    /// Sums wrap on `u64` overflow.
    pub fn group_by_sum(
        &self,
        filter_col: usize,
        lo: u64,
        hi: u64,
        group_col: usize,
        agg_col: usize,
    ) -> EngineResult<Vec<(u64, u64)>> {
        self.check_groupby_args(filter_col, group_col, agg_col)?;
        if lo > hi {
            return Ok(Vec::new());
        }
        let mut acc: BTreeMap<u64, u64> = BTreeMap::new();
        for row in self.scan_deduped(filter_col, lo, hi)? {
            let e = acc.entry(row.values[group_col]).or_insert(0);
            *e = e.wrapping_add(row.values[agg_col]);
        }
        Ok(acc.into_iter().collect())
    }

    /// Minimum of `agg_col` per `group_col` value where `filter_col` is in
    /// `[lo, hi]`.
    ///
    /// Returns `Vec<(group_value, min)>` sorted ascending by `group_value`.
    pub fn group_by_min(
        &self,
        filter_col: usize,
        lo: u64,
        hi: u64,
        group_col: usize,
        agg_col: usize,
    ) -> EngineResult<Vec<(u64, u64)>> {
        self.check_groupby_args(filter_col, group_col, agg_col)?;
        if lo > hi {
            return Ok(Vec::new());
        }
        let mut acc: BTreeMap<u64, u64> = BTreeMap::new();
        for row in self.scan_deduped(filter_col, lo, hi)? {
            let av = row.values[agg_col];
            acc.entry(row.values[group_col])
                .and_modify(|cur| *cur = (*cur).min(av))
                .or_insert(av);
        }
        Ok(acc.into_iter().collect())
    }

    /// Maximum of `agg_col` per `group_col` value where `filter_col` is in
    /// `[lo, hi]`.
    ///
    /// Returns `Vec<(group_value, max)>` sorted ascending by `group_value`.
    pub fn group_by_max(
        &self,
        filter_col: usize,
        lo: u64,
        hi: u64,
        group_col: usize,
        agg_col: usize,
    ) -> EngineResult<Vec<(u64, u64)>> {
        self.check_groupby_args(filter_col, group_col, agg_col)?;
        if lo > hi {
            return Ok(Vec::new());
        }
        let mut acc: BTreeMap<u64, u64> = BTreeMap::new();
        for row in self.scan_deduped(filter_col, lo, hi)? {
            let av = row.values[agg_col];
            acc.entry(row.values[group_col])
                .and_modify(|cur| *cur = (*cur).max(av))
                .or_insert(av);
        }
        Ok(acc.into_iter().collect())
    }
}
