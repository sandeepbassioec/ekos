//! [`ZoneMap`] — per-partition min/max statistics for range-scan pruning.
//!
//! ## Design
//!
//! A ZoneMap records the minimum and maximum value seen in each micro-partition
//! (by partition_id). A range query `WHERE col BETWEEN lo AND hi` can skip any
//! partition whose `[min, max]` interval does not overlap `[lo, hi]` — a ~10 ns
//! comparison instead of a full partition scan.
//!
//! ## Encoding contract
//!
//! Values are stored as raw `u64` and compared with **unsigned** ordering, so the
//! caller must supply an order-preserving `u64` encoding. This is not optional:
//! get it wrong and pruning drops rows instead of merely failing to help.
//!
//! | Column type | Encoding |
//! |---|---|
//! | `u64` / `u32` / … | identity |
//! | `i64` / `i32` / … | `x ^ (1 << 63)` — the same range shift the SIMD scan path uses |
//! | `f64` | IEEE 754 order-preserving transform |
//! | strings | order-preserving u64 prefix (a *hash* is not order-preserving and cannot be range-pruned) |
//!
//! Raw two's-complement `i64` is the trap: `-1i64 as u64` is `u64::MAX`, so a
//! column of small negatives yields the zone `[0, u64::MAX]` and a query like
//! `BETWEEN -10 AND 10` arrives with `lo > hi`. See
//! `signed_values_need_the_range_shift`.
//!
//! ## Which query API to use
//!
//! A zone answers "maybe" or "definitely not" — never "definitely yes". So the
//! only safe formulation starts from the real partition list and subtracts:
//!
//! ```text
//! to_scan = all_partitions_on_disk - zone_map.pruned_partitions(lo, hi)
//! ```
//!
//! Use [`ZoneMap::pruned_partitions`] for that. Do **not** use
//! [`ZoneMap::overlapping_partitions`] as a scan list: it can only yield ids the
//! map has actually seen, so any partition missing from the map — not yet
//! registered, or lost to an id collision — is silently skipped rather than
//! conservatively scanned.

use std::collections::HashMap;
use vectorvolt_hal::simd;

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
    ///
    /// **Replaces** any existing zone for `partition_id` rather than widening it —
    /// correct for the flush path, which allocates a fresh id per partition, but it
    /// means an id collision destroys the previous zone. `TxnManager` allocates ids
    /// from an `AtomicU16`, so id 65_535 is followed by id 0.
    ///
    /// Transposed bounds are normalized rather than stored. An inverted zone does
    /// not fail cleanly: `min <= hi && max >= lo` makes it match wide queries and
    /// reject narrow ones landing between the two bounds, so a single swapped
    /// argument would drop rows from *some* queries and not others. Transposed
    /// bounds have no second valid reading, so swapping them is a complete fix —
    /// no `debug_assert`, because the hazard should be closed in release too.
    pub fn set(&mut self, partition_id: u16, min: u64, max: u64) {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        self.entries.insert(partition_id, ZoneEntry { min, max });
    }

    /// Get the zone entry for `partition_id`.
    #[inline]
    pub fn get(&self, partition_id: u16) -> Option<ZoneEntry> {
        self.entries.get(&partition_id).copied()
    }

    /// Return an iterator of partition IDs whose zone **overlaps** `[lo, hi]`.
    ///
    /// Yields only ids present in this map. That makes it unsafe as a scan list —
    /// an unregistered partition is omitted, not conservatively included. Use
    /// [`ZoneMap::pruned_partitions`] and subtract from the real partition list
    /// instead. This method is for inspecting the map, not for planning a scan.
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

// ── ZoneVec ───────────────────────────────────────────────────────────────────

/// SoA (Structure-of-Arrays) zone store for SIMD-accelerated range pruning.
///
/// `ZoneMap` uses a `HashMap<u16, ZoneEntry>` — good for random access but
/// unsuitable for the tight vector loop that is the hot path of a range scan.
/// `ZoneVec` keeps three parallel arrays (`pids`, `mins`, `maxes`) so the min
/// and max values sit in cache-friendly linear memory. The HAL's
/// `simd::zone_overlap_indices` kernel then processes 4 zones per instruction
/// on AVX2 hardware.
///
/// ## Building one
///
/// Push zones directly with [`ZoneVec::push`], or convert an existing
/// [`ZoneMap`] with [`ZoneVec::from_zone_map`]. Iteration order is not
/// guaranteed.
///
/// ## Querying
///
/// [`ZoneVec::overlapping_pids`] returns the ids of every partition whose zone
/// overlaps the query range. The same caveats as [`ZoneMap::overlapping_partitions`]
/// apply: unregistered partitions are absent, so subtract from the real partition
/// list rather than using this as a scan list directly.
pub struct ZoneVec {
    pids: Vec<u32>,
    mins: Vec<u64>,
    maxes: Vec<u64>,
}

impl ZoneVec {
    pub fn new() -> Self {
        ZoneVec {
            pids: Vec::new(),
            mins: Vec::new(),
            maxes: Vec::new(),
        }
    }

    /// Number of zones stored.
    #[inline]
    pub fn len(&self) -> usize {
        self.pids.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pids.is_empty()
    }

    /// Append a zone for `pid` with bounds `[min, max]`.
    pub fn push(&mut self, pid: u32, min: u64, max: u64) {
        self.pids.push(pid);
        self.mins.push(min);
        self.maxes.push(max);
    }

    /// Build a `ZoneVec` from an existing [`ZoneMap`].
    pub fn from_zone_map(zm: &ZoneMap) -> Self {
        let mut v = ZoneVec::with_capacity(zm.len());
        for (&pid, &e) in &zm.entries {
            v.push(pid as u32, e.min, e.max);
        }
        v
    }

    fn with_capacity(n: usize) -> Self {
        ZoneVec {
            pids: Vec::with_capacity(n),
            mins: Vec::with_capacity(n),
            maxes: Vec::with_capacity(n),
        }
    }

    /// Return the partition IDs whose zone overlaps `[lo, hi]`.
    ///
    /// Uses the HAL SIMD kernel for the inner loop — AVX2 on capable hardware,
    /// portable scalar otherwise.
    pub fn overlapping_pids(&self, lo: u64, hi: u64) -> Vec<u32> {
        let mut indices: Vec<u32> = Vec::new();
        simd::zone_overlap_indices(&self.mins, &self.maxes, lo, hi, &mut indices);
        indices.iter().map(|&i| self.pids[i as usize]).collect()
    }
}

impl Default for ZoneVec {
    fn default() -> Self {
        ZoneVec::new()
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

// ── adversarial tests ─────────────────────────────────────────────────────────
//
// A ZoneMap can only fail in one direction that matters: claiming a partition
// holds no matches when it does. That drops rows from a query result with no
// error, so every test below is aimed at that single failure mode.

#[cfg(test)]
mod adversarial_tests {
    use super::*;

    /// `overlaps` must agree with brute-force set intersection, exhaustively.
    ///
    /// The hand-written cases above check six points. This checks every pair of
    /// intervals in a small universe, which is the only way to be sure the two
    /// `<=`/`>=` are not subtly swapped.
    #[test]
    fn overlaps_matches_brute_force_intersection() {
        const N: u64 = 12;
        for min in 0..N {
            for max in min..N {
                let e = ZoneEntry { min, max };
                for lo in 0..N {
                    for hi in lo..N {
                        let truth = (min..=max).any(|v| (lo..=hi).contains(&v));
                        assert_eq!(
                            e.overlaps(lo, hi),
                            truth,
                            "zone [{min},{max}] vs query [{lo},{hi}]"
                        );
                    }
                }
            }
        }
    }

    /// The u64 domain edges. No arithmetic happens in `overlaps`, so these should
    /// hold — asserted so a future "optimisation" using `max + 1` cannot slip in
    /// and wrap silently.
    #[test]
    fn saturating_edges_do_not_wrap() {
        let full = ZoneEntry {
            min: 0,
            max: u64::MAX,
        };
        assert!(full.overlaps(0, 0));
        assert!(full.overlaps(u64::MAX, u64::MAX));
        assert!(full.overlaps(u64::MAX / 2, u64::MAX / 2));

        let top = ZoneEntry {
            min: u64::MAX,
            max: u64::MAX,
        };
        assert!(top.overlaps(u64::MAX, u64::MAX));
        assert!(!top.overlaps(0, u64::MAX - 1));

        let bottom = ZoneEntry { min: 0, max: 0 };
        assert!(bottom.overlaps(0, 0));
        assert!(!bottom.overlaps(1, u64::MAX));
    }

    /// An inverted zone (`min > max`) prunes **arbitrarily**, and `set` normalizes
    /// it away for exactly that reason.
    ///
    /// My first guess was that an inverted zone would prune everything — bad, but
    /// at least uniform and therefore obvious. It does not. `min <= hi && max >= lo`
    /// on `[min=90, max=10]` matches any query wide enough to straddle the
    /// transposition and rejects any query landing in the gap between them:
    #[test]
    fn inverted_zone_prunes_arbitrarily_so_set_normalizes() {
        let bad = ZoneEntry { min: 90, max: 10 };
        assert!(bad.overlaps(0, u64::MAX), "wide query: matches");
        assert!(
            !bad.overlaps(50, 50),
            "gap query: pruned — a false negative"
        );
        assert!(bad.overlaps(0, 95), "straddling query: matches");

        // Silent, data-dependent row loss is the worst failure shape there is, so
        // `set` swaps the bounds rather than storing them.
        let mut zm = ZoneMap::new();
        zm.set(0, 90, 10); // transposed at the call site
        assert_eq!(
            zm.get(0).unwrap(),
            ZoneEntry { min: 10, max: 90 },
            "set() must normalize transposed bounds"
        );
        assert!(zm.pruned_partitions(50, 50).is_empty());
    }

    /// `update` cannot produce an inverted zone regardless of insertion order.
    ///
    /// This is why `update` is the safe path and `set` is the sharp one.
    #[test]
    fn update_never_inverts() {
        for order in [
            vec![5u64, 1, 9, 3, 7],
            vec![9, 7, 5, 3, 1],
            vec![1, 3, 5, 7, 9],
            vec![u64::MAX, 0, u64::MAX / 2],
        ] {
            let mut zm = ZoneMap::new();
            for v in &order {
                zm.update(0, *v);
            }
            let e = zm.get(0).unwrap();
            assert!(
                e.min <= e.max,
                "update produced inverted zone for {order:?}"
            );
            // And every value it was given must still be inside the zone.
            for v in &order {
                assert!(
                    e.min <= *v && *v <= e.max,
                    "value {v} fell outside its zone"
                );
            }
        }
    }

    /// Every value fed to `update` must survive as a query hit — the no-false-prune
    /// property, tested across adversarial value shapes rather than sorted runs.
    #[test]
    fn no_value_is_ever_pruned_away() {
        let shapes: Vec<(&str, Vec<u64>)> = vec![
            ("sequential", (0..64).collect()),
            ("descending", (0..64).rev().collect()),
            ("strided", (0..64).map(|i| i * 4096).collect()),
            ("clustered", (0..64).map(|i| 1_000_000 + i % 4).collect()),
            (
                "signed-as-u64",
                (0..64).map(|i| (i as i64 - 32) as u64).collect(),
            ),
            (
                "u64-edges",
                vec![0, 1, u64::MAX, u64::MAX - 1, u64::MAX / 2],
            ),
        ];

        for (name, values) in shapes {
            let mut zm = ZoneMap::new();
            for (i, v) in values.iter().enumerate() {
                zm.update((i % 8) as u16, *v);
            }
            for (i, v) in values.iter().enumerate() {
                let pid = (i % 8) as u16;
                assert!(
                    !zm.pruned_partitions(*v, *v).contains(&pid),
                    "{name}: partition {pid} pruned for point query {v} that it contains"
                );
                let hits: Vec<u16> = zm.overlapping_partitions(*v, *v).collect();
                assert!(
                    hits.contains(&pid),
                    "{name}: partition {pid} missing from overlaps for value {v}"
                );
            }
        }
    }

    /// **Raw two's-complement `i64` breaks pruning.** Documents the encoding
    /// contract that the module doc previously left implicit for signed integers.
    ///
    /// `-1i64 as u64` is `u64::MAX`, so a column of small negatives produces a
    /// zone spanning the top of the u64 domain, and an ordinary query like
    /// `WHERE x BETWEEN -10 AND 10` cannot be expressed as a single u64 interval
    /// at all. The caller must apply the same order-preserving shift the SIMD scan
    /// path uses: `x ^ (1 << 63)`.
    #[test]
    fn signed_values_need_the_range_shift() {
        const SHIFT: u64 = 1 << 63;
        let signed: Vec<i64> = vec![-100, -1, 0, 1, 100];

        // Unshifted: the zone is [0, u64::MAX] — every query "overlaps", so
        // pruning is useless, and any narrower query is simply wrong.
        let mut raw = ZoneMap::new();
        for v in &signed {
            raw.update(0, *v as u64);
        }
        let e = raw.get(0).unwrap();
        assert_eq!(
            (e.min, e.max),
            (0, u64::MAX),
            "raw i64 should collapse the zone to the whole domain"
        );
        // The honest consequence: a query for [-10, 10] is unrepresentable, and
        // the naive attempt prunes the partition that holds the answer.
        let (bad_lo, bad_hi) = ((-10i64) as u64, 10i64 as u64); // (huge, 10)
        assert!(bad_lo > bad_hi, "naive signed bounds arrive inverted");

        // Shifted: order is preserved, the zone is tight, and range queries work.
        let mut shifted = ZoneMap::new();
        for v in &signed {
            shifted.update(0, (*v as u64) ^ SHIFT);
        }
        let e = shifted.get(0).unwrap();
        assert_eq!(e.min, (-100i64 as u64) ^ SHIFT);
        assert_eq!(e.max, (100i64 as u64) ^ SHIFT);

        let lo = ((-10i64) as u64) ^ SHIFT;
        let hi = (10i64 as u64) ^ SHIFT;
        assert!(lo < hi, "shifted bounds must be ordered");
        assert!(
            e.overlaps(lo, hi),
            "shifted zone missed a range it contains"
        );
        // And a range that genuinely holds nothing is still pruned.
        let far_lo = (1_000i64 as u64) ^ SHIFT;
        let far_hi = (2_000i64 as u64) ^ SHIFT;
        assert!(!e.overlaps(far_lo, far_hi));
    }

    /// `overlapping_partitions` returns only TRACKED partitions.
    ///
    /// The module doc says untracked partitions "are assumed to overlap
    /// (conservative)" — but this iterator physically cannot yield an id it has
    /// never seen. A caller using it as its scan list silently skips every
    /// partition missing from the map. `pruned_partitions` is the safe API: it
    /// returns only definite skips, so unknown partitions are scanned by default.
    #[test]
    fn overlapping_partitions_omits_untracked_ids() {
        let mut zm = ZoneMap::new();
        zm.set(0, 0, 100);
        // Partitions 1 and 2 exist on disk but were never registered.

        let hits: Vec<u16> = zm.overlapping_partitions(0, u64::MAX).collect();
        assert_eq!(hits, vec![0], "iterator invented an untracked partition");

        // The safe formulation: start from the real partition list and remove only
        // what the ZoneMap can prove is empty.
        let on_disk: Vec<u16> = vec![0, 1, 2];
        let pruned = zm.pruned_partitions(200, 300); // p0 [0,100] cannot match
        let to_scan: Vec<u16> = on_disk
            .iter()
            .copied()
            .filter(|p| !pruned.contains(p))
            .collect();
        assert_eq!(
            to_scan,
            vec![1, 2],
            "untracked partitions must survive pruning"
        );
    }

    /// `set` on an existing id REPLACES rather than widens.
    ///
    /// Correct for the flush path (one `set` per new partition id), but it means a
    /// partition id collision destroys the previous zone. `TxnManager` allocates
    /// ids from an `AtomicU16`, so id 65_535 is followed by id 0 — and that
    /// wraparound silently overwrites partition 0's zone with the new partition's,
    /// making partition 0's rows unfindable by range scan.
    #[test]
    fn set_replaces_and_id_reuse_loses_a_zone() {
        let mut zm = ZoneMap::new();
        zm.set(7, 0, 100);
        zm.set(7, 500, 600); // same id, disjoint range
        assert_eq!(zm.get(7).unwrap(), ZoneEntry { min: 500, max: 600 });
        assert_eq!(
            zm.pruned_partitions(0, 100),
            vec![7],
            "the first zone was replaced, so its own range now prunes it"
        );

        // The wraparound the u16 id space makes reachable.
        let wrapped = 65_535u16.wrapping_add(1);
        assert_eq!(wrapped, 0, "u16 partition ids wrap to 0");
    }

    /// A zone is a lossy summary: it can say "maybe", never "definitely".
    ///
    /// Holes inside `[min, max]` produce false POSITIVES — a scanned partition
    /// with no matching row. That is the acceptable direction and is asserted here
    /// so nobody later "fixes" it into a false negative.
    #[test]
    fn holes_produce_false_positives_not_false_negatives() {
        let mut zm = ZoneMap::new();
        zm.update(0, 10);
        zm.update(0, 1_000_000); // nothing in between

        // 500 is not in the partition, but the zone must still admit it.
        assert!(
            zm.get(0).unwrap().overlaps(500, 500),
            "a hole was pruned — that would be a false negative"
        );
        assert!(zm.pruned_partitions(500, 500).is_empty());
    }
}
