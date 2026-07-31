//! [`CuckooMap`] — open-addressed Cuckoo hash map for primary-key lookups.
//!
//! ## Why Cuckoo hashing?
//!
//! Standard hash maps (Robin Hood, ABSL flat_hash_map) have O(1) average lookup
//! but O(n) worst case. Cuckoo hashing guarantees **O(1) worst-case lookup** with
//! exactly 2 memory probes — ideal for a latency-sensitive PK index where p99.9
//! must be ≤ 10 µs.
//!
//! ## Layout
//!
//! Two independent hash functions `h1`, `h2` map every key to two candidate
//! buckets. A key lives in exactly one of its two candidate buckets.
//!
//! ```text
//! buckets: [ (key, value) | (key, value) | ... | EMPTY | ... ]
//!              ↑ h1(k)                           ↑ h2(k) (or vice-versa)
//! ```
//!
//! On **insert**: probe h1 → probe h2 → if both full, evict one occupant and
//! re-insert the displaced key (BFS up to `MAX_DISPLACE` steps).
//!
//! On **lookup**: probe h1 and h2, at most 2 reads.
//!
//! ## Sentinel
//!
//! `u64::MAX` is reserved as the empty-slot sentinel. Keys equal to `u64::MAX`
//! are rejected with `IndexError::MapFull` — in practice PK values never reach
//! this (they are sequence numbers or hashes with the top bit masked off).

use crate::{IndexError, IndexResult, RowDescriptor};

// ── Tuning constants ──────────────────────────────────────────────────────────

/// Sentinel value: a bucket with this key is empty.
const EMPTY: u64 = u64::MAX;

/// Maximum eviction chain length before declaring the map full.
const MAX_DISPLACE: usize = 512;

/// Target load factor numerator (insert triggers grow at len > cap * NUM / DEN).
///
/// **7/16 = 43.75%, deliberately below 50%.**
///
/// 2-way cuckoo hashing with one item per bucket has a hard load threshold at
/// **exactly 50%** — above it, insertion fails with high probability no matter how
/// large `MAX_DISPLACE` is, because the displacement graph almost surely contains a
/// component with more items than slots. This was previously 9/16 = 56%, i.e. above
/// the threshold, and produced real `MapFull` errors on clustered key distributions
/// (e.g. several dense runs separated by large gaps — exactly what per-writer key
/// ranges look like). Sequential keys happened to survive, which is why the
/// existing 500-key test passed and hid it.
const LOAD_NUM: usize = 7;
const LOAD_DEN: usize = 16;

/// How many times `insert` may grow and retry before giving up.
///
/// Each grow doubles capacity and halves load, so the probability of consecutive
/// failures falls off a cliff; 4 is far more than needed and exists only so a
/// pathological hash cannot loop forever.
const MAX_GROW_RETRIES: usize = 4;

/// Minimum initial capacity (must be a power of two).
const MIN_CAPACITY: usize = 16;

// ── Hash functions ────────────────────────────────────────────────────────────
// Two independent 64-bit multiplicative hash functions.
// Constants are nothing-up-my-sleeve primes (Knuth / Fibonacci hashing).

/// ## Why the xor-shift is not optional
///
/// A bare `(key * K) & mask` is broken for **power-of-two-strided keys**, and those
/// are common: IDs scaled by a constant, byte offsets, aligned addresses,
/// fixed-interval timestamps.
///
/// If `key = i * 2^s`, then `key * K = (i * K) << s`, so the product's low `s` bits
/// are always zero. Masking takes exactly those low bits, so the function can only
/// ever reach `capacity / 2^s` distinct buckets — and **growing the map does not
/// help**, because the collapse is structural rather than a load problem. A
/// stride-256 key set was failing to insert at capacity 2048 for precisely this
/// reason.
///
/// `x ^= x >> 32` folds the high half down first. The high bits of a multiply depend
/// on *all* input bits, so after the fold the masked low bits do too. One shift and
/// one xor.
#[inline(always)]
fn h1(key: u64, mask: usize) -> usize {
    let mut x = key.wrapping_mul(0x517c_c1b7_2722_0a95);
    x ^= x >> 32;
    (x as usize) & mask
}

#[inline(always)]
fn h2(key: u64, mask: usize) -> usize {
    let mut x = key.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    (x as usize) & mask
}

/// The two candidate buckets for `key`, **guaranteed distinct**.
///
/// `h1` and `h2` collide for some keys — unavoidable for any pair of hash
/// functions. When they do, that key has only ONE possible home, and two such keys
/// hashing to the same slot displace each other forever: the chain cycles until
/// `MAX_DISPLACE` runs out. That is how insertion was failing at **1.4% load** on a
/// map of capacity 8192 — not a capacity problem at all, and growing could never
/// fix it.
///
/// Nudging the second home to the next slot restores the two-choice property that
/// cuckoo hashing depends on. `get`, `insert`, and `remove` all route through here,
/// because if they computed candidates differently a stored key would become
/// invisible.
#[inline(always)]
fn homes(key: u64, mask: usize) -> (usize, usize) {
    let i1 = h1(key, mask);
    let mut i2 = h2(key, mask);
    if i2 == i1 {
        i2 = (i1 + 1) & mask;
    }
    (i1, i2)
}

// ── Bucket ────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct Bucket {
    key: u64,
    value: RowDescriptor,
}

impl Bucket {
    #[inline]
    const fn empty() -> Self {
        Bucket {
            key: EMPTY,
            value: RowDescriptor::from_u64(0),
        }
    }

    #[inline]
    fn is_empty(self) -> bool {
        self.key == EMPTY
    }
}

// ── CuckooMap ─────────────────────────────────────────────────────────────────

/// Primary-key index: maps a `u64` PK to its [`RowDescriptor`].
///
/// Lookup is O(1) worst case with exactly 2 memory probes.
/// Insert amortises to O(1), and grows-and-retries rather than failing —
/// see [`CuckooMap::insert`].
pub struct CuckooMap {
    buckets: Vec<Bucket>,
    /// `capacity - 1`: used as a bitmask for fast modulo (capacity is always power-of-2).
    mask: usize,
    len: usize,
}

impl CuckooMap {
    /// Create a map pre-sized to hold at least `expected` entries without resizing.
    pub fn with_capacity(expected: usize) -> Self {
        let cap = next_power_of_two((expected * LOAD_DEN / LOAD_NUM).max(MIN_CAPACITY));
        Self {
            buckets: vec![Bucket::empty(); cap],
            mask: cap - 1,
            len: 0,
        }
    }

    /// Number of entries currently stored.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// True if no entries are stored.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Allocated slot count (always a power of two).
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buckets.len()
    }

    /// Look up `key`. Returns the [`RowDescriptor`] if found.
    ///
    /// Exactly 2 memory probes — O(1) worst case.
    #[inline]
    pub fn get(&self, key: u64) -> Option<RowDescriptor> {
        debug_assert_ne!(key, EMPTY, "u64::MAX is reserved as empty sentinel");
        let b = &self.buckets;
        let (i1, i2) = homes(key, self.mask);
        if b[i1].key == key {
            return Some(b[i1].value);
        }
        if b[i2].key == key {
            return Some(b[i2].value);
        }
        None
    }

    /// Insert or update `key → value`.
    ///
    /// Grows and retries on a failed displacement chain, so insertion is **total**
    /// in practice. `Err(IndexError::MapFull)` is returned only if
    /// `MAX_GROW_RETRIES` consecutive attempts all fail, which requires a
    /// pathological hash rather than mere load.
    pub fn insert(&mut self, key: u64, value: RowDescriptor) -> IndexResult<()> {
        debug_assert_ne!(key, EMPTY, "u64::MAX is reserved as empty sentinel");

        // Grow before we hit the load limit.
        if self.len * LOAD_DEN >= self.capacity() * LOAD_NUM {
            self.grow();
        }

        // Grow and re-place the orphan rather than surfacing `MapFull`.
        //
        // Staying under the 50% threshold makes a failed chain unlikely, not
        // impossible. For the **primary key index** that distinction matters:
        // returning an error means refusing to record a row already written to the
        // WAL, which is data loss rather than backpressure. Insertion has to be
        // total.
        let mut pending = (key, value);
        for _ in 0..MAX_GROW_RETRIES {
            match self.insert_inner(pending.0, pending.1) {
                Ok(()) => return Ok(()),
                Err(orphan) => {
                    // `pending.0` is now placed; `orphan` has no home. Grow, then
                    // find the orphan a slot.
                    self.grow();
                    pending = orphan;
                }
            }
        }
        Err(IndexError::MapFull(self.capacity()))
    }

    /// Remove `key`. Returns the evicted [`RowDescriptor`] if the key existed.
    pub fn remove(&mut self, key: u64) -> Option<RowDescriptor> {
        let (i1, i2) = homes(key, self.mask);
        if self.buckets[i1].key == key {
            let v = self.buckets[i1].value;
            self.buckets[i1] = Bucket::empty();
            self.len -= 1;
            return Some(v);
        }
        if self.buckets[i2].key == key {
            let v = self.buckets[i2].value;
            self.buckets[i2] = Bucket::empty();
            self.len -= 1;
            return Some(v);
        }
        None
    }

    /// True if `key` is present.
    #[inline]
    pub fn contains(&self, key: u64) -> bool {
        self.get(key).is_some()
    }

    /// Iterate over all `(key, value)` pairs. Order is unspecified.
    pub fn iter(&self) -> impl Iterator<Item = (u64, RowDescriptor)> + '_ {
        self.buckets
            .iter()
            .filter(|b| !b.is_empty())
            .map(|b| (b.key, b.value))
    }

    // ── internals ─────────────────────────────────────────────────────────────

    /// Place `key` using cuckoo displacement.
    ///
    /// ## The eviction rule, and the bug it replaces
    ///
    /// A displaced key must go to its **alternate** home — the one it was *not*
    /// occupying. The previous loop always evicted from `i1` and then recomputed the
    /// evicted key's `i1`, which is frequently the slot just taken. Key A and key B
    /// would then hand the same slot back and forth until `MAX_DISPLACE` ran out.
    ///
    /// That is why insertion failed at **0.7% load** in a map of capacity 16384: not
    /// a capacity problem, a 2-cycle. Growing could never help, which is exactly what
    /// the symptom showed — the failure followed the map up through 2048, 8192, 16384.
    ///
    /// Tracking the slot we came *from* and moving to the other one makes each step
    /// advance through the displacement graph instead of oscillating.
    ///
    /// On failure returns the **orphan**: the entry evicted and never re-homed.
    /// Returning it is essential — dropping it silently removed an already-stored row
    /// from the primary key index, which surfaced as a map holding 238 entries after
    /// 240 successful-looking inserts.
    fn insert_inner(&mut self, key: u64, value: RowDescriptor) -> Result<(), (u64, RowDescriptor)> {
        let (i1, i2) = homes(key, self.mask);

        // Update in place if the key is already present.
        if self.buckets[i1].key == key {
            self.buckets[i1].value = value;
            return Ok(());
        }
        if self.buckets[i2].key == key {
            self.buckets[i2].value = value;
            return Ok(());
        }
        // Either home free: done in one probe.
        if self.buckets[i1].is_empty() {
            self.buckets[i1] = Bucket { key, value };
            self.len += 1;
            return Ok(());
        }
        if self.buckets[i2].is_empty() {
            self.buckets[i2] = Bucket { key, value };
            self.len += 1;
            return Ok(());
        }

        // Both homes occupied: claim i1 and cascade its occupant onward.
        let mut pos = i1;
        let mut carried_key = key;
        let mut carried_val = value;

        for _ in 0..MAX_DISPLACE {
            let evicted = self.buckets[pos];
            self.buckets[pos] = Bucket {
                key: carried_key,
                value: carried_val,
            };
            carried_key = evicted.key;
            carried_val = evicted.value;

            // Move the evicted entry to its OTHER home, not back to this one.
            let (e1, e2) = homes(carried_key, self.mask);
            pos = if e1 == pos { e2 } else { e1 };

            if self.buckets[pos].is_empty() {
                self.buckets[pos] = Bucket {
                    key: carried_key,
                    value: carried_val,
                };
                // One net new entry: the original `key`. Every step in between was
                // a swap, which does not change occupancy.
                self.len += 1;
                return Ok(());
            }
        }

        Err((carried_key, carried_val))
    }

    fn grow(&mut self) {
        let new_cap = self.capacity() * 2;
        let mut new_map = CuckooMap {
            buckets: vec![Bucket::empty(); new_cap],
            mask: new_cap - 1,
            len: 0,
        };
        for b in &self.buckets {
            if !b.is_empty() {
                // Grow never fails because we are well below the new load limit.
                // A freshly doubled map is at most ~22% loaded, so a failed chain here
                // would mean the hash is pathological rather than the map full.
                new_map
                    .insert_inner(b.key, b.value)
                    .expect("rehash into a half-empty map must not fail");
            }
        }
        *self = new_map;
    }
}

impl Default for CuckooMap {
    fn default() -> Self {
        CuckooMap::with_capacity(64)
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

    fn rd(offset: u32) -> RowDescriptor {
        RowDescriptor::new(0, 0, offset)
    }

    #[test]
    fn insert_and_get() {
        let mut m = CuckooMap::with_capacity(16);
        m.insert(1, rd(10)).unwrap();
        m.insert(2, rd(20)).unwrap();
        assert_eq!(m.get(1), Some(rd(10)));
        assert_eq!(m.get(2), Some(rd(20)));
        assert_eq!(m.get(3), None);
    }

    #[test]
    fn update_existing_key() {
        let mut m = CuckooMap::with_capacity(16);
        m.insert(42, rd(1)).unwrap();
        m.insert(42, rd(2)).unwrap();
        assert_eq!(m.get(42), Some(rd(2)));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn remove() {
        let mut m = CuckooMap::with_capacity(16);
        m.insert(7, rd(99)).unwrap();
        assert_eq!(m.remove(7), Some(rd(99)));
        assert_eq!(m.get(7), None);
        assert_eq!(m.len(), 0);
    }

    #[test]
    fn large_insert_triggers_grow() {
        let mut m = CuckooMap::with_capacity(8);
        for i in 0..500u64 {
            m.insert(i, rd(i as u32)).unwrap();
        }
        for i in 0..500u64 {
            assert_eq!(m.get(i), Some(rd(i as u32)), "key {i} missing");
        }
        assert_eq!(m.len(), 500);
    }

    #[test]
    fn iter_yields_all_entries() {
        let mut m = CuckooMap::with_capacity(16);
        for i in 0..10u64 {
            m.insert(i, rd(i as u32 * 2)).unwrap();
        }
        let mut collected: Vec<_> = m.iter().collect();
        collected.sort_by_key(|(k, _)| *k);
        for (i, (k, v)) in collected.iter().enumerate() {
            assert_eq!(*k, i as u64);
            assert_eq!(*v, rd(i as u32 * 2));
        }
    }

    #[test]
    fn h1_h2_differ() {
        // Verify the two hash functions produce different buckets for typical keys.
        let mask = 255usize;
        let mut collisions = 0usize;
        for k in 0..1000u64 {
            if h1(k, mask) == h2(k, mask) {
                collisions += 1;
            }
        }
        // Expect < 5% collision between h1 and h2 — if higher, the constants are bad.
        assert!(
            collisions < 50,
            "h1/h2 collide too often: {collisions}/1000"
        );
    }
}

#[cfg(test)]
mod load_factor_tests {
    use super::*;

    fn rd(offset: u32) -> RowDescriptor {
        RowDescriptor::new(0, 0, offset)
    }

    #[test]
    fn clustered_keys_do_not_overflow_the_map() {
        // The exact distribution that broke this: several dense runs separated by
        // large gaps — what per-writer key ranges look like when 8 threads each own
        // a 1000-wide block. Sequential keys survived the old 56% load factor, which
        // is why the pre-existing 500-key test passed and this case was missed.
        let mut m = CuckooMap::with_capacity(16);
        let mut inserted = Vec::new();
        for t in 0..8u64 {
            for i in 1..=30u64 {
                let key = t * 1_000 + i;
                m.insert(key, rd(i as u32))
                    .unwrap_or_else(|e| panic!("insert of clustered key {key} failed: {e}"));
                inserted.push(key);
            }
        }
        assert_eq!(m.len(), inserted.len());
        for k in inserted {
            assert!(
                m.get(k).is_some(),
                "clustered key {k} not found after insert"
            );
        }
    }

    #[test]
    fn load_factor_stays_below_the_two_way_cuckoo_threshold() {
        // Guards the constant itself. Above 50%, 2-way cuckoo hashing with
        // single-slot buckets fails with high probability no matter what
        // MAX_DISPLACE is, so this must never be relaxed without changing the
        // bucket structure too.
        let load = LOAD_NUM as f64 / LOAD_DEN as f64;
        assert!(
            load < 0.5,
            "target load {load:.3} is at or above the 50% threshold for 2-way cuckoo hashing"
        );
    }

    #[test]
    fn insertion_is_total_across_adversarial_key_shapes() {
        // Several distributions that stress different collision patterns. The
        // property under test is simply that insertion never fails — for a PK index,
        // refusing a row already written to the WAL would be data loss.
        let shapes: Vec<(&str, Vec<u64>)> = vec![
            ("sequential", (0..2_000).collect()),
            ("stride-256", (0..1_000).map(|i| i * 256).collect()),
            (
                "stride-power-of-two",
                (0..1_000).map(|i| i * 4_096).collect(),
            ),
            (
                "clustered",
                (0..40)
                    .flat_map(|t| (1..=50).map(move |i| t * 10_000 + i))
                    .collect(),
            ),
            ("high-bit", (0..1_000).map(|i| (1u64 << 63) | i).collect()),
            ("descending", (0..1_000).rev().collect()),
        ];

        for (name, keys) in shapes {
            let mut m = CuckooMap::with_capacity(16);
            for (n, &k) in keys.iter().enumerate() {
                m.insert(k, rd(n as u32))
                    .unwrap_or_else(|e| panic!("{name}: insert #{n} of key {k} failed: {e}"));
            }
            assert_eq!(m.len(), keys.len(), "{name}: wrong length");
            for &k in &keys {
                assert!(m.get(k).is_some(), "{name}: key {k} missing");
            }
        }
    }
}

#[cfg(test)]
mod hash_quality_tests {
    use super::*;

    /// Distinct buckets reached by `h` over `keys`, at a given capacity.
    fn spread(h: fn(u64, usize) -> usize, keys: &[u64], cap: usize) -> usize {
        let mask = cap - 1;
        let mut seen = vec![false; cap];
        for &k in keys {
            seen[h(k, mask)] = true;
        }
        seen.iter().filter(|&&b| b).count()
    }

    #[test]
    fn both_hashes_spread_power_of_two_strided_keys() {
        // The regression. A bare multiplicative hash masked to low bits collapses
        // here: for key = i * 2^s the product's low s bits are always zero, so only
        // capacity / 2^s buckets are reachable and growing cannot fix it.
        const CAP: usize = 1024;
        for stride in [2u64, 16, 256, 4_096, 65_536] {
            let keys: Vec<u64> = (0..512u64).map(|i| i * stride).collect();
            for (name, h) in [("h1", h1 as fn(u64, usize) -> usize), ("h2", h2)] {
                let reached = spread(h, &keys, CAP);
                assert!(
                    reached > 256,
                    "{name} reached only {reached}/{CAP} buckets for stride {stride} \
                     — the hash is collapsing on strided keys"
                );
            }
        }
    }

    #[test]
    fn both_hashes_spread_sequential_and_clustered_keys() {
        const CAP: usize = 1024;
        let sequential: Vec<u64> = (0..512).collect();
        let clustered: Vec<u64> = (0..16u64)
            .flat_map(|t| (0..32u64).map(move |i| t * 10_000 + i))
            .collect();
        let high_bit: Vec<u64> = (0..512u64).map(|i| (1u64 << 63) | (i * 8)).collect();

        for (label, keys) in [
            ("sequential", &sequential),
            ("clustered", &clustered),
            ("high-bit strided", &high_bit),
        ] {
            for (name, h) in [("h1", h1 as fn(u64, usize) -> usize), ("h2", h2)] {
                let reached = spread(h, keys, CAP);
                assert!(
                    reached > keys.len() / 2,
                    "{name} reached only {reached} buckets for {} {label} keys",
                    keys.len()
                );
            }
        }
    }

    /// Mimic the real engine workload: every key is inserted TWICE — once as a
    /// PENDING descriptor at insert time, once with real coordinates at flush
    /// time — with the map growing throughout.
    ///
    /// The earlier order probe only did fresh inserts and found nothing. This
    /// covers the update path, which is what the engine actually does and which
    /// exercises displacement against keys that are already present.
    #[test]
    fn reinserting_existing_keys_never_loses_one() {
        for seed in 0..300u64 {
            let n = 320u64;
            let mut order: Vec<u64> = (0..n).collect();
            let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
            for i in (1..order.len()).rev() {
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                order.swap(i, (s % (i as u64 + 1)) as usize);
            }
            let mut m = CuckooMap::with_capacity(64);
            // Phase 1: PENDING entries, as `TxnManager::insert` writes them.
            for (off, &k) in order.iter().enumerate() {
                m.insert(k, RowDescriptor::new(u16::MAX, 0, off as u32))
                    .unwrap();
            }
            // Phase 2: rewrite each with real coordinates, as `flush` does.
            for (off, &k) in order.iter().enumerate() {
                m.insert(k, RowDescriptor::new(0, 1, off as u32)).unwrap();
            }
            let missing: Vec<u64> = (0..n).filter(|k| m.get(*k).is_none()).collect();
            assert!(
                missing.is_empty(),
                "seed {seed}: {} of {n} keys lost after re-insert, e.g. {:?}",
                missing.len(),
                &missing[..missing.len().min(5)]
            );
            assert_eq!(
                m.len(),
                n as usize,
                "seed {seed}: len drifted to {}",
                m.len()
            );
        }
    }
}
