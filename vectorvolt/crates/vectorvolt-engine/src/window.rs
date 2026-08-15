//! Window / running-aggregate functions.
//!
//! All four functions operate on a predicate-filtered row set and return a
//! `Vec<WindowRow>` — a (pk, computed value) pair for each surviving row.
//!
//! ## Functions
//!
//! | Method | Sort order | `window_val` |
//! |--------|------------|--------------|
//! | `running_sum(filter, lo, hi, agg_col)` | ascending pk | prefix sum of `agg_col` (wrapping) |
//! | `running_count(filter, lo, hi)` | ascending pk | 1-based row number |
//! | `rank_by(filter, lo, hi, sort_col, asc)` | sort_col order | standard rank (gaps after ties) |
//! | `dense_rank_by(filter, lo, hi, sort_col, asc)` | sort_col order | dense rank (no gaps) |
//!
//! ## Tie-breaking
//!
//! For `rank_by` and `dense_rank_by`, rows with equal `sort_col` values are
//! in ascending pk order within the tie group (deterministic output).
//!
//! ## Overflow
//!
//! `running_sum` uses wrapping addition — consistent with `increment_field`.

use crate::engine::TableEngine;
use crate::{EngineResult, EngineRow};

/// A single (pk, computed window value) pair returned by every window function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowRow {
    pub pk: u64,
    pub window_val: u64,
}

impl TableEngine {
    /// Cumulative sum of `agg_col` over `filter_col ∈ [lo, hi]`, in ascending pk order.
    ///
    /// `window_val` is the prefix sum up to and including this row (wrapping on
    /// `u64` overflow).  Returns `Ok(Vec::new())` when no rows match.
    pub fn running_sum(
        &self,
        filter_col: usize,
        lo: u64,
        hi: u64,
        agg_col: usize,
    ) -> EngineResult<Vec<WindowRow>> {
        self.check_col(filter_col)?;
        self.check_col(agg_col)?;
        let mut rows = self.scan_deduped(filter_col, lo, hi)?;
        rows.sort_unstable_by_key(|r| r.pk);
        let mut cumsum: u64 = 0;
        Ok(rows
            .into_iter()
            .map(|r| {
                cumsum = cumsum.wrapping_add(r.values[agg_col]);
                WindowRow {
                    pk: r.pk,
                    window_val: cumsum,
                }
            })
            .collect())
    }

    /// 1-based row number for each row in `filter_col ∈ [lo, hi]`, in ascending pk order.
    ///
    /// `window_val` is 1 for the row with the smallest pk, 2 for the next, and so on.
    pub fn running_count(
        &self,
        filter_col: usize,
        lo: u64,
        hi: u64,
    ) -> EngineResult<Vec<WindowRow>> {
        self.check_col(filter_col)?;
        let mut rows = self.scan_deduped(filter_col, lo, hi)?;
        rows.sort_unstable_by_key(|r| r.pk);
        Ok(rows
            .into_iter()
            .enumerate()
            .map(|(i, r)| WindowRow {
                pk: r.pk,
                window_val: i as u64 + 1,
            })
            .collect())
    }

    /// Standard rank of each row sorted by `sort_col`.
    ///
    /// Rows with equal `sort_col` receive the same rank; the next distinct
    /// value receives rank = previous rank + number of tied rows (SQL-style
    /// gap).  Result is ordered by `sort_col` (ascending or descending per
    /// `ascending`), with pk as the tiebreaker within equal values.
    pub fn rank_by(
        &self,
        filter_col: usize,
        lo: u64,
        hi: u64,
        sort_col: usize,
        ascending: bool,
    ) -> EngineResult<Vec<WindowRow>> {
        self.check_col(filter_col)?;
        self.check_col(sort_col)?;
        let mut rows = self.scan_deduped(filter_col, lo, hi)?;
        sort_rows(&mut rows, sort_col, ascending);
        Ok(assign_rank(&rows, sort_col, false))
    }

    /// Dense rank of each row sorted by `sort_col`.
    ///
    /// Rows with equal `sort_col` receive the same rank; the next distinct
    /// value receives the next consecutive rank (no gap).  Result is ordered
    /// by `sort_col`, with pk as the tiebreaker within equal values.
    pub fn dense_rank_by(
        &self,
        filter_col: usize,
        lo: u64,
        hi: u64,
        sort_col: usize,
        ascending: bool,
    ) -> EngineResult<Vec<WindowRow>> {
        self.check_col(filter_col)?;
        self.check_col(sort_col)?;
        let mut rows = self.scan_deduped(filter_col, lo, hi)?;
        sort_rows(&mut rows, sort_col, ascending);
        Ok(assign_rank(&rows, sort_col, true))
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn sort_rows(rows: &mut [EngineRow], sort_col: usize, ascending: bool) {
    if ascending {
        rows.sort_unstable_by(|a, b| {
            a.values[sort_col]
                .cmp(&b.values[sort_col])
                .then(a.pk.cmp(&b.pk))
        });
    } else {
        rows.sort_unstable_by(|a, b| {
            b.values[sort_col]
                .cmp(&a.values[sort_col])
                .then(a.pk.cmp(&b.pk))
        });
    }
}

/// Assign standard (dense=false) or dense (dense=true) rank to a pre-sorted slice.
fn assign_rank(rows: &[EngineRow], sort_col: usize, dense: bool) -> Vec<WindowRow> {
    if rows.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::with_capacity(rows.len());
    let mut rank: u64 = 1;
    let mut prev_val = rows[0].values[sort_col];

    for (pos, row) in (1_u64..).zip(rows.iter()) {
        let val = row.values[sort_col];
        if val != prev_val {
            rank = if dense { rank + 1 } else { pos };
            prev_val = val;
        }
        result.push(WindowRow {
            pk: row.pk,
            window_val: rank,
        });
    }

    result
}
