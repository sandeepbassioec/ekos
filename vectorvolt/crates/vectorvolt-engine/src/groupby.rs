//! Group-by aggregates.
//!
//! Each function filters rows by a range predicate on `filter_col`, groups
//! matching rows by `group_col`, and computes an aggregate over `agg_col`.
//! Results are returned as a `Vec<(group_value, aggregate)>` sorted
//! ascending by `group_value` for deterministic output.
//!
//! ## Architecture
//!
//! Like the scalar aggregates in `aggregate.rs`, these functions:
//!
//! 1. **Zone-prune** — skip partitions where `filter_col` has no overlap with
//!    `[lo, hi]`.
//! 2. **Two-column decode** — decode only `filter_col` (to re-check the
//!    predicate) + `group_col` (for grouping) + `agg_col` (for the aggregate).
//!    No full-row materialisation.
//! 3. **BTreeMap accumulation** — keyed on `group_col` value, updated in-place.

use crate::EngineResult;
use crate::engine::TableEngine;
use std::collections::BTreeMap;

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
    /// `agg_col` is ignored (pass any valid column index).
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
        self.flush_if_needed()?;

        let mut acc: BTreeMap<u64, u64> = BTreeMap::new();
        let mut filter_buf: Vec<u64> = Vec::new();
        let mut group_buf: Vec<u64> = Vec::new();

        let parts = self.partitions.read();
        for reader in parts.values() {
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
            reader.decode_column_into(group_col, &mut group_buf)?;
            for &o in &offsets {
                let idx = o as usize;
                let fv = *filter_buf.get(idx).unwrap_or(&0);
                if fv < lo || fv > hi {
                    continue;
                }
                let gv = *group_buf.get(idx).unwrap_or(&0);
                *acc.entry(gv).or_insert(0) += 1;
            }
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
        self.flush_if_needed()?;

        let mut acc: BTreeMap<u64, u64> = BTreeMap::new();
        let mut filter_buf: Vec<u64> = Vec::new();
        let mut group_buf: Vec<u64> = Vec::new();
        let mut agg_buf: Vec<u64> = Vec::new();

        let parts = self.partitions.read();
        for reader in parts.values() {
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
            reader.decode_column_into(group_col, &mut group_buf)?;
            reader.decode_column_into(agg_col, &mut agg_buf)?;
            for &o in &offsets {
                let idx = o as usize;
                let fv = *filter_buf.get(idx).unwrap_or(&0);
                if fv < lo || fv > hi {
                    continue;
                }
                let gv = *group_buf.get(idx).unwrap_or(&0);
                let av = *agg_buf.get(idx).unwrap_or(&0);
                *acc.entry(gv).or_insert(0) = acc.get(&gv).unwrap_or(&0).wrapping_add(av);
            }
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
        self.flush_if_needed()?;

        let mut acc: BTreeMap<u64, u64> = BTreeMap::new();
        let mut filter_buf: Vec<u64> = Vec::new();
        let mut group_buf: Vec<u64> = Vec::new();
        let mut agg_buf: Vec<u64> = Vec::new();

        let parts = self.partitions.read();
        for reader in parts.values() {
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
            reader.decode_column_into(group_col, &mut group_buf)?;
            reader.decode_column_into(agg_col, &mut agg_buf)?;
            for &o in &offsets {
                let idx = o as usize;
                let fv = *filter_buf.get(idx).unwrap_or(&0);
                if fv < lo || fv > hi {
                    continue;
                }
                let gv = *group_buf.get(idx).unwrap_or(&0);
                let av = *agg_buf.get(idx).unwrap_or(&0);
                acc.entry(gv)
                    .and_modify(|cur| *cur = (*cur).min(av))
                    .or_insert(av);
            }
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
        self.flush_if_needed()?;

        let mut acc: BTreeMap<u64, u64> = BTreeMap::new();
        let mut filter_buf: Vec<u64> = Vec::new();
        let mut group_buf: Vec<u64> = Vec::new();
        let mut agg_buf: Vec<u64> = Vec::new();

        let parts = self.partitions.read();
        for reader in parts.values() {
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
            reader.decode_column_into(group_col, &mut group_buf)?;
            reader.decode_column_into(agg_col, &mut agg_buf)?;
            for &o in &offsets {
                let idx = o as usize;
                let fv = *filter_buf.get(idx).unwrap_or(&0);
                if fv < lo || fv > hi {
                    continue;
                }
                let gv = *group_buf.get(idx).unwrap_or(&0);
                let av = *agg_buf.get(idx).unwrap_or(&0);
                acc.entry(gv)
                    .and_modify(|cur| *cur = (*cur).max(av))
                    .or_insert(av);
            }
        }
        Ok(acc.into_iter().collect())
    }
}
