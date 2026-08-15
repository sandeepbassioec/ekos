//! Multi-predicate compound scan (AND semantics).
//!
//! [`TableEngine::scan_where`] accepts a list of `(col_idx, lo, hi)` predicates
//! and returns rows that satisfy **all** of them simultaneously.
//!
//! ## Algorithm
//!
//! For each partition:
//! 1. **Zone prune**: if any predicate column's zone has no overlap with its
//!    `[lo, hi]`, the entire partition is skipped.
//! 2. **Column-by-column filter**: the first predicate column is decoded and
//!    matching row indices are collected; for each remaining predicate that
//!    column is decoded and only the surviving indices are kept.
//! 3. **Late materialisation**: all columns are decoded only for surviving rows.
//!
//! Decoding work scales with matching rows at each stage, not total rows — a
//! selective first predicate makes later checks nearly free.

use std::collections::HashSet;

use crate::engine::TableEngine;
use crate::{EngineResult, EngineRow};

impl TableEngine {
    /// Return rows where every `(col_idx, lo, hi)` predicate holds (AND semantics).
    ///
    /// A row is included when `lo <= values[col_idx] <= hi` is true for **every**
    /// predicate in the slice.  An empty slice returns all rows (full scan).
    /// If any predicate has `lo > hi` the result is always empty.
    ///
    /// # Errors
    ///
    /// Returns `Err` when any `col_idx` is out of range, or on I/O failure.
    pub fn scan_where(&self, predicates: &[(usize, u64, u64)]) -> EngineResult<Vec<EngineRow>> {
        for &(col_idx, _, _) in predicates {
            self.check_col(col_idx)?;
        }

        // Any predicate with lo > hi can never match any row.
        if predicates.iter().any(|&(_, lo, hi)| lo > hi) {
            return Ok(Vec::new());
        }

        self.flush_if_needed()?;

        if predicates.is_empty() {
            return self.scan(0, 0, u64::MAX);
        }

        let mut results = Vec::new();
        let parts = self.partitions.read();

        for reader in parts.values() {
            // Zone prune: skip this partition if ANY predicate column's zone
            // has no overlap with its [lo, hi].
            let mut skip = false;
            for &(col_idx, lo, hi) in predicates {
                if let Ok((min, max)) = reader.zone_min_max(col_idx)
                    && (min > hi || max < lo)
                {
                    skip = true;
                    break;
                }
            }
            if skip {
                continue;
            }

            // Decode the first predicate column and collect matching row indices.
            let mut col_buf: Vec<u64> = Vec::new();
            let (first_col, first_lo, first_hi) = predicates[0];
            reader.decode_column_into(first_col, &mut col_buf)?;
            let mut candidates: Vec<usize> = col_buf
                .iter()
                .enumerate()
                .filter(|&(_, &v)| v >= first_lo && v <= first_hi)
                .map(|(i, _)| i)
                .collect();
            col_buf.clear();

            // Narrow candidates against each remaining predicate.
            for &(col_idx, lo, hi) in &predicates[1..] {
                if candidates.is_empty() {
                    break;
                }
                reader.decode_column_into(col_idx, &mut col_buf)?;
                candidates.retain(|&i| col_buf[i] >= lo && col_buf[i] <= hi);
                col_buf.clear();
            }

            // Late-materialise: read all columns only for surviving rows.
            let col_count = reader.col_count() as usize;
            for row_idx in candidates {
                let mut values = Vec::with_capacity(col_count);
                for c in 0..col_count {
                    values.push(reader.read_value(c, row_idx)?);
                }
                let pk = values[0];
                results.push(EngineRow { pk, values });
            }
        }

        // Dedup by pk via get(): the same PK can appear in multiple partition
        // files.  For each unique PK, get() provides the authoritative current
        // row; we then re-verify all predicates on the current values so rows
        // updated out of the predicate range are correctly excluded.
        let mut seen: HashSet<u64> = HashSet::with_capacity(results.len());
        let mut verified: Vec<EngineRow> = Vec::with_capacity(results.len());
        for row in results {
            if seen.insert(row.pk)
                && let Some(current) = self.get(row.pk)?
                && predicates
                    .iter()
                    .all(|&(col, lo, hi)| current.values[col] >= lo && current.values[col] <= hi)
            {
                verified.push(current);
            }
        }
        Ok(verified)
    }
}
