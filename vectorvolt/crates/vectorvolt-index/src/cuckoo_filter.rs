//! [`CuckooFilter`] — probabilistic set membership for FK existence checks.
//!
//! ## Design
//!
//! A Cuckoo filter is a space-efficient probabilistic data structure that
//! answers "is key K in this set?" with:
//! - **Zero false negatives** (if K was inserted, it is always found). This is the
//!   one guarantee that must never break — it held only after the displacement
//!   rollback in [`CuckooFilter::insert`] was added; before that, an exhausted
//!   displacement chain silently discarded an earlier key's fingerprint.
//! - **Tunable false-positive rate** (configurable at construction time).
//! - **O(1) worst-case lookup**: exactly 2 bucket probes, ~150 ns on hot cache.
//! - **Supports deletions** — unlike Bloom filters.
//!
//! ## Implementation
//!
//! Each bucket holds `BUCKET_SIZE` fingerprints (u8). Fingerprints are 8-bit
//! hash values derived from the key. A key maps to two buckets via:
//!
//! ```text
//! i1 = h(key) % num_buckets
//! i2 = (i1 XOR h(fingerprint)) % num_buckets
//! ```
//!
//! The XOR construction means the alternate bucket can be computed from
//! either bucket + the fingerprint, enabling displacement-based insertion.
//!
//! ## False positive rate
//!
//! At ≤ 95% load: FP ≈ 2 / 2^8 ≈ 0.78% per lookup.
//! For FK checks this is acceptable — a false positive causes one wasted
//! row fetch, not data corruption.
//!
//! ## Sentinel
//!
//! Fingerprint `0` is reserved as the empty-slot marker.
//! Keys whose fingerprint hashes to 0 are mapped to fingerprint 1.

use crate::IndexResult;

// ── Tuning ────────────────────────────────────────────────────────────────────

/// Number of fingerprint slots per bucket.
const BUCKET_SIZE: usize = 4;

/// Maximum displacement steps before declaring the filter full.
const MAX_KICKS: usize = 500;

// ── Hash helpers ──────────────────────────────────────────────────────────────

/// 8-bit fingerprint from key (never 0).
#[inline(always)]
fn fingerprint(key: u64) -> u8 {
    let x = key.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    let fp = (x >> 56) as u8;
    if fp == 0 { 1 } else { fp }
}

/// Primary bucket index.
///
/// The `x ^= x >> 32` is load-bearing. `num_buckets` is always a power of two, so
/// `% num_buckets` is a low-bit mask — and the low bits of a multiply depend only on
/// the low bits of the input. For `key = i * 2^s` the product's low `s` bits are
/// always zero, so without the fold this reached only `num_buckets / 2^s` buckets:
/// measured 64 of 1024 at stride 16. Strided keys are ordinary (scaled IDs, byte
/// offsets, aligned addresses, fixed-interval timestamps), and unlike a hash map a
/// filter cannot be grown out of the problem.
#[inline(always)]
fn index1(key: u64, num_buckets: usize) -> usize {
    let mut x = key.wrapping_mul(0x517c_c1b7_2722_0a95);
    x ^= x >> 32;
    (x as usize) % num_buckets
}

/// Alternate bucket index (symmetric with XOR).
#[inline(always)]
fn index2(i1: usize, fp: u8, num_buckets: usize) -> usize {
    // Mix the fingerprint with a prime so that similar fingerprints spread apart.
    let hash = (fp as usize).wrapping_mul(0x9e37_79b9);
    (i1 ^ hash) % num_buckets
}

// ── Bucket ────────────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
struct Bucket {
    slots: [u8; BUCKET_SIZE],
}

impl Bucket {
    fn insert(&mut self, fp: u8) -> bool {
        for slot in self.slots.iter_mut() {
            if *slot == 0 {
                *slot = fp;
                return true;
            }
        }
        false
    }

    fn contains(&self, fp: u8) -> bool {
        self.slots.contains(&fp)
    }

    fn remove(&mut self, fp: u8) -> bool {
        for slot in self.slots.iter_mut() {
            if *slot == fp {
                *slot = 0;
                return true;
            }
        }
        false
    }

    /// Evict the first non-empty slot and return it (for displacement).
    fn evict_first(&mut self) -> Option<u8> {
        for slot in self.slots.iter_mut() {
            if *slot != 0 {
                let fp = *slot;
                *slot = 0;
                return Some(fp);
            }
        }
        None
    }
}

// ── CuckooFilter ─────────────────────────────────────────────────────────────

/// Probabilistic set: ~0.78% false-positive rate, O(1) worst-case lookup.
///
/// When the cuckoo structure fills beyond its displacement limit, an exact
/// overflow `HashSet` absorbs further insertions so `insert()` never fails.
pub struct CuckooFilter {
    buckets: Vec<Bucket>,
    num_buckets: usize,
    count: usize,
    /// Exact spillover for keys that couldn't fit in the cuckoo structure.
    overflow: std::collections::HashSet<u64>,
}

impl CuckooFilter {
    /// Create a filter sized for at least `capacity` items.
    /// Actual bucket count is rounded up to the next power of two.
    pub fn with_capacity(capacity: usize) -> Self {
        // Each bucket holds BUCKET_SIZE items; target ≤ 95% load.
        let needed = capacity.div_ceil(BUCKET_SIZE);
        let num_buckets = next_power_of_two(needed.max(4));
        CuckooFilter {
            buckets: vec![Bucket::default(); num_buckets],
            num_buckets,
            count: 0,
            overflow: std::collections::HashSet::new(),
        }
    }

    /// Insert `key` into the filter. Always succeeds.
    ///
    /// When the cuckoo structure is full (displacement chain exceeds `MAX_KICKS`),
    /// the key is stored in an exact overflow `HashSet` instead of returning an
    /// error, so `insert()` is infallible and the filter unbounded.
    pub fn insert(&mut self, key: u64) -> IndexResult<()> {
        let fp = fingerprint(key);
        let i1 = index1(key, self.num_buckets);
        let i2 = index2(i1, fp, self.num_buckets);

        if self.buckets[i1].insert(fp) || self.buckets[i2].insert(fp) {
            self.count += 1;
            return Ok(());
        }

        // Displacement, recording each swap so it can be undone.
        //
        // The recording is not bookkeeping for its own sake. Previously, when the
        // chain exhausted `MAX_KICKS`, the fingerprint still being carried was simply
        // dropped — and it belonged to a key inserted *earlier*, which then became a
        // **false negative**. That breaks the filter's one absolute guarantee, and
        // for an FK existence check it means reporting that a row does not exist when
        // it does.
        //
        // A cuckoo filter cannot be grown out of this the way a hash map can: it
        // stores 8-bit fingerprints, not keys, so `index1` is unrecoverable and
        // rehashing is impossible. Rolling back is the only way to leave every
        // previously-inserted key findable.
        let mut chain: Vec<(usize, u8, u8)> = Vec::new(); // (bucket, evicted, placed)
        let mut cur_idx = i1;
        let mut cur_fp = fp;

        for _ in 0..MAX_KICKS {
            let evicted_fp = self.buckets[cur_idx]
                .evict_first()
                .expect("both homes were full, so this bucket has an occupant");
            self.buckets[cur_idx].insert(cur_fp);
            chain.push((cur_idx, evicted_fp, cur_fp));

            cur_fp = evicted_fp;
            cur_idx = index2(cur_idx, cur_fp, self.num_buckets);
            if self.buckets[cur_idx].insert(cur_fp) {
                self.count += 1;
                return Ok(());
            }
        }

        // Undo every swap, newest first, restoring the pre-insert state exactly.
        for (idx, evicted, placed) in chain.into_iter().rev() {
            self.buckets[idx].remove(placed);
            self.buckets[idx].insert(evicted);
        }

        // The cuckoo structure has no room for this key; the exact overflow set
        // guarantees it is still findable.
        self.overflow.insert(key);
        self.count += 1;
        Ok(())
    }

    /// True if `key` **might** be in the set. False positives are possible (~0.78%).
    /// Keys in the overflow set are exact (zero FP for those).
    #[inline]
    pub fn contains(&self, key: u64) -> bool {
        if self.overflow.contains(&key) {
            return true;
        }
        let fp = fingerprint(key);
        let i1 = index1(key, self.num_buckets);
        let i2 = index2(i1, fp, self.num_buckets);
        self.buckets[i1].contains(fp) || self.buckets[i2].contains(fp)
    }

    /// Remove `key` from the filter. Returns `true` if found and removed.
    pub fn remove(&mut self, key: u64) -> bool {
        if self.overflow.remove(&key) {
            self.count -= 1;
            return true;
        }
        let fp = fingerprint(key);
        let i1 = index1(key, self.num_buckets);
        let i2 = index2(i1, fp, self.num_buckets);
        if self.buckets[i1].remove(fp) {
            self.count -= 1;
            return true;
        }
        if self.buckets[i2].remove(fp) {
            self.count -= 1;
            return true;
        }
        false
    }

    /// Number of items inserted (not unique keys — duplicates count once each).
    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Allocated fingerprint slot count (buckets × BUCKET_SIZE).
    #[inline]
    pub fn capacity(&self) -> usize {
        self.num_buckets * BUCKET_SIZE
    }
}

impl Default for CuckooFilter {
    fn default() -> Self {
        CuckooFilter::with_capacity(64)
    }
}

/// Number of keys in the exact overflow (diagnostic).
impl CuckooFilter {
    pub fn overflow_len(&self) -> usize {
        self.overflow.len()
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn next_power_of_two(n: usize) -> usize {
    if n <= 1 {
        return 1;
    }
    let mut p = 1usize;
    while p < n {
        p <<= 1;
    }
    p
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_contains() {
        let mut f = CuckooFilter::with_capacity(128);
        f.insert(1).unwrap();
        f.insert(2).unwrap();
        f.insert(100).unwrap();
        assert!(f.contains(1));
        assert!(f.contains(2));
        assert!(f.contains(100));
        // Key not inserted — should not be present (no false positives for
        // these specific values with these hash constants).
        // Note: we can't guarantee zero FP in general, so we skip assert_false.
    }

    #[test]
    fn remove_makes_absent() {
        let mut f = CuckooFilter::with_capacity(64);
        f.insert(42).unwrap();
        assert!(f.contains(42));
        f.remove(42);
        // After remove, the key is gone (unless another key shares the fingerprint —
        // unlikely for our test values).
        assert!(!f.contains(42));
    }

    #[test]
    fn large_insert_no_error() {
        let mut f = CuckooFilter::with_capacity(2048);
        for i in 0..1000u64 {
            f.insert(i).expect("insert should not fail below capacity");
        }
        assert_eq!(f.len(), 1000);
        for i in 0..1000u64 {
            assert!(f.contains(i), "key {i} should be present");
        }
    }

    #[test]
    fn fingerprint_never_zero() {
        // Verify fingerprint(k) != 0 for all k in a wide range.
        for k in 0u64..10_000 {
            assert_ne!(fingerprint(k), 0, "fingerprint({k}) was 0");
        }
    }

    #[test]
    fn false_positive_rate_reasonable() {
        // Insert 500 keys, query 500 different keys — FP rate should be < 5%.
        let mut f = CuckooFilter::with_capacity(1024);
        for i in 0u64..500 {
            f.insert(i).unwrap();
        }
        let mut fps = 0usize;
        for i in 500u64..1000 {
            if f.contains(i) {
                fps += 1;
            }
        }
        // With 8-bit fingerprints we expect ~0.78% FP; allow up to 5% for variance.
        assert!(fps < 25, "false positive rate too high: {fps}/500");
    }
}

#[cfg(test)]
mod hash_quality_tests {
    use super::*;

    /// Distinct primary buckets reached over `keys`.
    fn spread(keys: &[u64], num_buckets: usize) -> usize {
        let mut seen = vec![false; num_buckets];
        for &k in keys {
            seen[index1(k, num_buckets)] = true;
        }
        seen.iter().filter(|&&b| b).count()
    }

    #[test]
    fn index1_spreads_power_of_two_strided_keys() {
        // Same defect class as CuckooMap::h1 had: a multiplicative hash reduced by
        // `% num_buckets` takes the LOW bits of the product, and `num_buckets` is
        // always a power of two so the modulo IS a low-bit mask. For key = i * 2^s
        // the product's low s bits are always zero, collapsing the reachable bucket
        // set to num_buckets / 2^s.
        //
        // For a FILTER this is not a correctness bug — insert is infallible via the
        // exact overflow set — but it destroys the space claim, because keys spill
        // out of 8-bit fingerprints into a HashSet of full u64s.
        const N: usize = 1024;
        for stride in [2u64, 16, 256, 4_096] {
            let keys: Vec<u64> = (0..512u64).map(|i| i * stride).collect();
            let reached = spread(&keys, N);
            assert!(
                reached > 200,
                "index1 reached only {reached}/{N} buckets for stride {stride} \
                 — the hash is collapsing on strided keys"
            );
        }
    }

    #[test]
    fn strided_keys_do_not_spill_into_the_overflow_set() {
        // The observable consequence. A well-spread hash keeps everything in
        // fingerprint buckets; a collapsing one dumps keys into the exact overflow
        // HashSet, which costs 8x the memory per key and defeats the point.
        let mut f = CuckooFilter::with_capacity(4096);
        let keys: Vec<u64> = (0..1_000u64).map(|i| i * 256).collect();
        for &k in &keys {
            f.insert(k).unwrap();
        }
        assert_eq!(f.len(), keys.len());
        assert!(
            f.overflow_len() < keys.len() / 20,
            "{} of {} strided keys spilled to the exact overflow set — index1 is \
             not spreading them across buckets",
            f.overflow_len(),
            keys.len()
        );
    }

    #[test]
    fn no_false_negatives_across_adversarial_key_shapes() {
        // The filter's one inviolable contract: if a key was inserted, `contains`
        // must return true. False POSITIVES are permitted by design; false
        // negatives never are.
        let shapes: Vec<(&str, Vec<u64>)> = vec![
            ("sequential", (0..2_000).collect()),
            ("stride-256", (0..1_000).map(|i| i * 256).collect()),
            ("stride-65536", (0..1_000).map(|i| i * 65_536).collect()),
            (
                "clustered",
                (0..40u64)
                    .flat_map(|t| (1..=50u64).map(move |i| t * 10_000 + i))
                    .collect(),
            ),
            (
                "high-bit",
                (0..1_000u64).map(|i| (1u64 << 63) | (i * 8)).collect(),
            ),
            ("max-adjacent", (0..500u64).map(|i| u64::MAX - i).collect()),
        ];

        for (name, keys) in shapes {
            let mut f = CuckooFilter::with_capacity(keys.len() * 2);
            for &k in &keys {
                f.insert(k).unwrap();
            }
            for &k in &keys {
                assert!(
                    f.contains(k),
                    "{name}: FALSE NEGATIVE for key {k} — the filter lost a key it was given"
                );
            }
        }
    }

    #[test]
    fn alternate_bucket_is_an_involution() {
        // Cuckoo filters depend on index2 being self-inverse: a displaced
        // fingerprint lives in its alternate bucket, and `contains` must be able to
        // get back. If this broke, lookups would miss displaced entries.
        for num_buckets in [4usize, 16, 256, 1024] {
            for fp in 1u8..=255 {
                for i1 in 0..num_buckets {
                    let i2 = index2(i1, fp, num_buckets);
                    assert!(
                        i2 < num_buckets,
                        "index2 out of range: {i2} >= {num_buckets}"
                    );
                    assert_eq!(
                        index2(i2, fp, num_buckets),
                        i1,
                        "index2 is not an involution at num_buckets={num_buckets}, fp={fp}, i1={i1}"
                    );
                }
            }
        }
    }
}
