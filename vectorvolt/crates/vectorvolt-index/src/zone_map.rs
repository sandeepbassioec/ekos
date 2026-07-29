//! [`ZoneMap`] — per-partition min/max statistics for range-scan pruning.
//!
//! ## Design
//!
//! A ZoneMap records the minimum and maximum value seen in each micro-partition
//! (by partition_id). A range query `WHERE col BETWEEN lo AND hi` can skip any
//! partition whose `[min, max]` interval does not overlap `[lo, hi]` — a ~10 ns
//! comparison instead of a full partition scan.
//!
//! Values are stored as raw `u64` (caller handles encoding — e.g. integer
//! identity, or bitwise-reinterpreted floats using IEEE 754 order-preserving
//! encoding). Strings should be hashed into a u64 prefix for comparison.

use std::collections::HashMap;

/// Min/max statistics for one micro-partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZoneEntry {
    pub min: u64,
    pub max: u64,
}

impl ZoneEntry {
    /// True if `[self.min, self.max]` overlaps `[lo, hi]`.
    #[inline]
    pub fn overlaps(self, lo: u64, hi: u64) -> bool {
        self.min <= hi && self.max >= lo
    }
}

/// Per-partition ZoneMap: maps `partition_id → ZoneEntry`.
pub struct ZoneMap {
    entries: HashMap<u16, ZoneEntry>,
}

impl ZoneMap {
    pub fn new() -> Self {
        ZoneMap {
            entries: HashMap::new(),
        }
    }

    /// Record that `partition_id` contains `value`.
    ///
    /// Expands the partition's `[min, max]` range if `value` falls outside it.
    pub fn update(&mut self, partition_id: u16, value: u64) {
        let e = self.entries.entry(partition_id).or_insert(ZoneEntry {
            min: value,
            max: value,
        });
        if value < e.min {
            e.min = value;
        }
        if value > e.max {
            e.max = value;
        }
    }

    /// Explicitly set the zone for a partition (e.g. after bulk load).
    pub fn set(&mut self, partition_id: u16, min: u64, max: u64) {
        self.entries.insert(partition_id, ZoneEntry { min, max });
    }

    /// Get the zone entry for `partition_id`.
    #[inline]
    pub fn get(&self, partition_id: u16) -> Option<ZoneEntry> {
        self.entries.get(&partition_id).copied()
    }

    /// Return an iterator of partition IDs whose zone **overlaps** `[lo, hi]`.
    ///
    /// Partitions not in this ZoneMap are assumed to overlap (conservative).
    pub fn overlapping_partitions(&self, lo: u64, hi: u64) -> impl Iterator<Item = u16> + '_ {
        self.entries
            .iter()
            .filter(move |(_, e)| e.overlaps(lo, hi))
            .map(|(pid, _)| *pid)
    }

    /// Return all partition IDs that can be skipped for the range `[lo, hi]`.
    pub fn pruned_partitions(&self, lo: u64, hi: u64) -> Vec<u16> {
        self.entries
            .iter()
            .filter(move |(_, e)| !e.overlaps(lo, hi))
            .map(|(pid, _)| *pid)
            .collect()
    }

    /// Number of partitions tracked.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for ZoneMap {
    fn default() -> Self {
        ZoneMap::new()
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_expands_range() {
        let mut zm = ZoneMap::new();
        zm.update(0, 50);
        zm.update(0, 10);
        zm.update(0, 90);
        let e = zm.get(0).unwrap();
        assert_eq!(e.min, 10);
        assert_eq!(e.max, 90);
    }

    #[test]
    fn overlaps_basic() {
        let e = ZoneEntry { min: 10, max: 90 };
        assert!(e.overlaps(0, 10)); // touches left edge
        assert!(e.overlaps(90, 100)); // touches right edge
        assert!(e.overlaps(20, 80)); // fully inside
        assert!(e.overlaps(0, 200)); // fully outside (query contains zone)
        assert!(!e.overlaps(91, 200)); // gap on the right
        assert!(!e.overlaps(0, 9)); // gap on the left
    }

    #[test]
    fn overlapping_partitions() {
        let mut zm = ZoneMap::new();
        zm.set(0, 0, 100);
        zm.set(1, 200, 300);
        zm.set(2, 50, 150);

        let mut hits: Vec<u16> = zm.overlapping_partitions(80, 120).collect();
        hits.sort_unstable();
        // p0: [0,100] ∩ [80,120] = [80,100] ✓
        // p1: [200,300] ∩ [80,120] = ∅ ✗
        // p2: [50,150] ∩ [80,120] = [80,120] ✓
        assert_eq!(hits, vec![0, 2]);
    }

    #[test]
    fn pruned_partitions() {
        let mut zm = ZoneMap::new();
        zm.set(0, 0, 10);
        zm.set(1, 20, 30);
        zm.set(2, 15, 25);

        let mut pruned = zm.pruned_partitions(22, 28);
        pruned.sort_unstable();
        // p0: [0,10] → skip (no overlap with [22,28])
        // p1: [20,30] → keep
        // p2: [15,25] → keep
        assert_eq!(pruned, vec![0]);
    }

    #[test]
    fn single_value_partition() {
        let mut zm = ZoneMap::new();
        zm.update(5, 42);
        let e = zm.get(5).unwrap();
        assert_eq!(e.min, 42);
        assert_eq!(e.max, 42);
        assert!(e.overlaps(42, 42));
        assert!(!e.overlaps(43, 43));
    }
}
