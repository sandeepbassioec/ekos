//! Secondary indexes — on-demand build, auto-maintained on writes.
//!
//! A secondary index maps a non-PK column value to the set of PKs that hold
//! that value, enabling O(log n) exact and range lookups without a full scan.
//!
//! ## Lifecycle
//!
//! An index is built explicitly with [`TableEngine::build_secondary_index`],
//! which performs a full partition scan.  Once built, every subsequent
//! `insert` and `delete` on the engine automatically updates all live indexes
//! so they stay consistent with the table.  Dropping an index stops
//! maintenance for that column; rebuilding it restores consistency from the
//! full partition snapshot.
//!
//! If no index is built, `insert` and `delete` pay no extra cost.
//!
//! ## API
//!
//! | Method | Description |
//! |--------|-------------|
//! | `build_secondary_index(col)` | Scan all partitions and build the index for `col`. |
//! | `secondary_lookup(col, value)` | PKs where `col == value`. |
//! | `secondary_range_lookup(col, lo, hi)` | PKs where `col` is in `[lo, hi]`. |
//! | `secondary_get(col, value)` | Full rows where `col == value`. |
//! | `drop_secondary_index(col)` | Remove the index for `col` from memory. |
//! | `has_secondary_index(col)` | Whether an index currently exists for `col`. |
//! | `secondary_index_entry_count(col)` | Number of distinct values in the index. |

use std::collections::{BTreeMap, BTreeSet};

use crate::engine::TableEngine;
use crate::{EngineResult, EngineRow};

impl TableEngine {
    /// Scan all partitions and build a secondary index for `col_idx`.
    ///
    /// Any previous index for the same column is replaced.  The index maps
    /// each distinct value of `col_idx` to the set of PKs that hold it.
    ///
    /// Cost: O(N) rows decoded for `col_idx` only — no other columns are
    /// touched.  Zone pruning is not applied (all partitions are scanned
    /// because the index must be complete).
    pub fn build_secondary_index(&self, col_idx: usize) -> EngineResult<()> {
        self.check_col(col_idx)?;
        self.flush_if_needed()?;

        let mut index: BTreeMap<u64, BTreeSet<u64>> = BTreeMap::new();

        {
            let mut col_buf: Vec<u64> = Vec::new();
            let mut pk_buf: Vec<u64> = Vec::new();
            let parts = self.partitions.read();
            for reader in parts.values() {
                reader.decode_column_into(col_idx, &mut col_buf)?;
                reader.decode_column_into(0, &mut pk_buf)?;
                let len = col_buf.len().min(pk_buf.len());
                for i in 0..len {
                    index.entry(col_buf[i]).or_default().insert(pk_buf[i]);
                }
            }
        }

        self.secondary_indexes.write().insert(col_idx, index);
        Ok(())
    }

    /// Return the PKs where `col_idx == value`.
    ///
    /// Returns an empty `Vec` when no rows match or when no index has been
    /// built for `col_idx`.  The returned PKs are sorted ascending.
    pub fn secondary_lookup(&self, col_idx: usize, value: u64) -> EngineResult<Vec<u64>> {
        self.check_col(col_idx)?;
        let indexes = self.secondary_indexes.read();
        let Some(index) = indexes.get(&col_idx) else {
            return Ok(Vec::new());
        };
        let pks = index
            .get(&value)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default();
        Ok(pks)
    }

    /// Return the PKs where `col_idx` is in `[lo, hi]`.
    ///
    /// Returns an empty `Vec` when no rows match, when `lo > hi`, or when no
    /// index has been built for `col_idx`.  The returned PKs are sorted
    /// ascending within each value group (groups themselves are in ascending
    /// value order, so the overall list is also sorted ascending by PK within
    /// equal values, but not globally sorted across values).
    pub fn secondary_range_lookup(
        &self,
        col_idx: usize,
        lo: u64,
        hi: u64,
    ) -> EngineResult<Vec<u64>> {
        self.check_col(col_idx)?;
        if lo > hi {
            return Ok(Vec::new());
        }
        let indexes = self.secondary_indexes.read();
        let Some(index) = indexes.get(&col_idx) else {
            return Ok(Vec::new());
        };
        let pks: Vec<u64> = index
            .range(lo..=hi)
            .flat_map(|(_, set)| set.iter().copied())
            .collect();
        Ok(pks)
    }

    /// Fetch full rows for every PK returned by a secondary lookup.
    ///
    /// Convenience wrapper: calls [`secondary_lookup`] then [`get`] for each
    /// PK, skipping any that have been deleted since the index was built.
    pub fn secondary_get(&self, col_idx: usize, value: u64) -> EngineResult<Vec<EngineRow>> {
        let pks = self.secondary_lookup(col_idx, value)?;
        let mut rows = Vec::with_capacity(pks.len());
        for pk in pks {
            if let Some(row) = self.get(pk)? {
                rows.push(row);
            }
        }
        Ok(rows)
    }

    /// Remove the secondary index for `col_idx` from memory.
    ///
    /// A no-op when no index exists for that column.
    pub fn drop_secondary_index(&self, col_idx: usize) {
        self.secondary_indexes.write().remove(&col_idx);
    }

    /// Whether a secondary index currently exists for `col_idx`.
    pub fn has_secondary_index(&self, col_idx: usize) -> bool {
        self.secondary_indexes.read().contains_key(&col_idx)
    }

    /// Number of distinct values in the secondary index for `col_idx`.
    ///
    /// Returns `0` when no index exists.
    pub fn secondary_index_entry_count(&self, col_idx: usize) -> usize {
        self.secondary_indexes
            .read()
            .get(&col_idx)
            .map(|idx| idx.len())
            .unwrap_or(0)
    }

    // ── Write-path maintenance ────────────────────────────────────────────────

    /// Update all live secondary indexes after a successful `insert`.
    ///
    /// `old_row` is `None` for a pure insert, or `Some` when replacing an
    /// existing row (upsert path).  `new_values` is the full column slice that
    /// was just written.
    ///
    /// Fast path: returns immediately when no indexes are live.
    pub(crate) fn secondary_maintain_insert(
        &self,
        pk: u64,
        old_row: Option<&EngineRow>,
        new_values: &[u64],
    ) {
        let mut indexes = self.secondary_indexes.write();
        if indexes.is_empty() {
            return;
        }
        for (col_idx, btree) in indexes.iter_mut() {
            // Remove pk from the old bucket when replacing an existing row.
            if let Some(old) = old_row {
                let old_val = old.values[*col_idx];
                if let Some(set) = btree.get_mut(&old_val) {
                    set.remove(&pk);
                }
            }
            // Insert pk into the new bucket.
            let new_val = new_values[*col_idx];
            btree.entry(new_val).or_default().insert(pk);
        }
    }

    /// Update all live secondary indexes after a successful `delete`.
    ///
    /// Fast path: a no-op when no indexes are live.
    pub(crate) fn secondary_maintain_delete(&self, pk: u64, old_row: &EngineRow) {
        let mut indexes = self.secondary_indexes.write();
        if indexes.is_empty() {
            return;
        }
        for (col_idx, btree) in indexes.iter_mut() {
            let old_val = old_row.values[*col_idx];
            if let Some(set) = btree.get_mut(&old_val) {
                set.remove(&pk);
            }
        }
    }
}
