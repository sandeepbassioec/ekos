//! [`SecondaryIndex`] — column-value → row-ID set, backed by Roaring Bitmaps.
//!
//! ## Design
//!
//! Each distinct column value hash (u64 truncated to u32 for the bitmap) maps
//! to a [`RoaringBitmap`] of row IDs. A secondary scan:
//!
//! 1. Hashes the query value to get a bucket key.
//! 2. Looks up the bitmap (O(1) hash-map probe).
//! 3. Iterates or intersects the bitmap (~500 ns per 1 K rows).
//!
//! False positives are possible (two values can share a hash). The storage
//! engine must re-check the actual column value after fetching each row.
//!
//! ## Row IDs
//!
//! Row IDs are `u32` offsets into a partition's row array. They map directly
//! to the `row_offset` field of [`RowDescriptor`].

use roaring::RoaringBitmap;
use std::collections::HashMap;

/// Maps a column-value hash → set of row IDs (Roaring Bitmap).
pub struct SecondaryIndex {
    /// `value_hash → RoaringBitmap of row_ids`
    map: HashMap<u32, RoaringBitmap>,
}

impl SecondaryIndex {
    /// Create an empty secondary index.
    pub fn new() -> Self {
        SecondaryIndex {
            map: HashMap::new(),
        }
    }

    /// Register that `row_id` has the column value `value_hash`.
    ///
    /// `value_hash` is a u32-truncated hash of the actual column value.
    /// The caller is responsible for hashing (use e.g. FNV-1a or xxh32).
    pub fn insert(&mut self, value_hash: u32, row_id: u32) {
        self.map.entry(value_hash).or_default().insert(row_id);
    }

    /// Remove `row_id` from the bitmap for `value_hash`.
    /// No-op if either is absent.
    pub fn remove(&mut self, value_hash: u32, row_id: u32) {
        if let Some(bm) = self.map.get_mut(&value_hash) {
            bm.remove(row_id);
            if bm.is_empty() {
                self.map.remove(&value_hash);
            }
        }
    }

    /// Return an iterator over all row IDs that match `value_hash`.
    ///
    /// Returns `None` if no rows have this hash (fast-path skip).
    pub fn get(&self, value_hash: u32) -> Option<impl Iterator<Item = u32> + '_> {
        self.map.get(&value_hash).map(|bm| bm.iter())
    }

    /// True if any row has `value_hash`.
    #[inline]
    pub fn contains_hash(&self, value_hash: u32) -> bool {
        self.map.contains_key(&value_hash)
    }

    /// Total number of (hash, row_id) pairs stored.
    pub fn cardinality(&self) -> u64 {
        self.map.values().map(|bm| bm.len()).sum()
    }

    /// Number of distinct value hashes indexed.
    pub fn distinct_values(&self) -> usize {
        self.map.len()
    }

    /// Collect all row IDs matching `value_hash` into a `Vec<u32>`.
    /// Returns an empty vec if the hash is not present.
    pub fn get_vec(&self, value_hash: u32) -> Vec<u32> {
        match self.map.get(&value_hash) {
            Some(bm) => bm.iter().collect(),
            None => Vec::new(),
        }
    }
}

impl Default for SecondaryIndex {
    fn default() -> Self {
        SecondaryIndex::new()
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut idx = SecondaryIndex::new();
        idx.insert(42, 0);
        idx.insert(42, 5);
        idx.insert(42, 100);
        idx.insert(99, 7);

        let mut rows = idx.get_vec(42);
        rows.sort_unstable();
        assert_eq!(rows, vec![0, 5, 100]);

        let rows99 = idx.get_vec(99);
        assert_eq!(rows99, vec![7]);

        assert!(idx.get(0).is_none());
    }

    #[test]
    fn remove_entry() {
        let mut idx = SecondaryIndex::new();
        idx.insert(1, 10);
        idx.insert(1, 20);
        idx.remove(1, 10);

        let rows = idx.get_vec(1);
        assert_eq!(rows, vec![20]);
    }

    #[test]
    fn remove_last_cleans_up_bucket() {
        let mut idx = SecondaryIndex::new();
        idx.insert(7, 3);
        idx.remove(7, 3);
        assert!(!idx.contains_hash(7));
        assert_eq!(idx.distinct_values(), 0);
    }

    #[test]
    fn cardinality() {
        let mut idx = SecondaryIndex::new();
        idx.insert(1, 0);
        idx.insert(1, 1);
        idx.insert(2, 0);
        assert_eq!(idx.cardinality(), 3);
        assert_eq!(idx.distinct_values(), 2);
    }

    #[test]
    fn large_insert() {
        let mut idx = SecondaryIndex::new();
        for row_id in 0..10_000u32 {
            idx.insert(row_id % 100, row_id);
        }
        assert_eq!(idx.cardinality(), 10_000);
        assert_eq!(idx.distinct_values(), 100);
        for hash in 0..100u32 {
            let rows = idx.get_vec(hash);
            // Each hash maps to 100 rows: hash, hash+100, hash+200, ...
            assert_eq!(rows.len(), 100);
        }
    }
}
