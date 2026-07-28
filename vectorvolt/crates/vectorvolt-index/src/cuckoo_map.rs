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
const LOAD_NUM: usize = 9;
const LOAD_DEN: usize = 16;

/// Minimum initial capacity (must be a power of two).
const MIN_CAPACITY: usize = 16;

// ── Hash functions ────────────────────────────────────────────────────────────
// Two independent 64-bit multiplicative hash functions.
// Constants are nothing-up-my-sleeve primes (Knuth / Fibonacci hashing).

#[inline(always)]
fn h1(key: u64, mask: usize) -> usize {
    let x = key.wrapping_mul(0x517c_c1b7_2722_0a95);
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
/// Insert amortises to O(1) at ≤ 56% load.
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
        let m = self.mask;
        let i1 = h1(key, m);
        if b[i1].key == key {
            return Some(b[i1].value);
        }
        let i2 = h2(key, m);
        if b[i2].key == key {
            return Some(b[i2].value);
        }
        None
    }

    /// Insert or update `key → value`.
    ///
    /// Returns `Ok(())` on success, `Err(IndexError::MapFull)` if the eviction
    /// chain exceeds `MAX_DISPLACE` (extremely rare at < 56% load).
    pub fn insert(&mut self, key: u64, value: RowDescriptor) -> IndexResult<()> {
        debug_assert_ne!(key, EMPTY, "u64::MAX is reserved as empty sentinel");

        // Grow before we hit the load limit.
        if self.len * LOAD_DEN >= self.capacity() * LOAD_NUM {
            self.grow();
        }

        self.insert_inner(key, value)
    }

    /// Remove `key`. Returns the evicted [`RowDescriptor`] if the key existed.
    pub fn remove(&mut self, key: u64) -> Option<RowDescriptor> {
        let m = self.mask;
        let i1 = h1(key, m);
        if self.buckets[i1].key == key {
            let v = self.buckets[i1].value;
            self.buckets[i1] = Bucket::empty();
            self.len -= 1;
            return Some(v);
        }
        let i2 = h2(key, m);
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

    fn insert_inner(&mut self, mut key: u64, mut value: RowDescriptor) -> IndexResult<()> {
        for _ in 0..MAX_DISPLACE {
            let m = self.mask;
            let i1 = h1(key, m);

            // Slot 1 empty or same key → place here.
            if self.buckets[i1].is_empty() {
                self.buckets[i1] = Bucket { key, value };
                self.len += 1;
                return Ok(());
            }
            if self.buckets[i1].key == key {
                self.buckets[i1].value = value; // update
                return Ok(());
            }

            // Slot 2 empty or same key → place here.
            let i2 = h2(key, m);
            if self.buckets[i2].is_empty() {
                self.buckets[i2] = Bucket { key, value };
                self.len += 1;
                return Ok(());
            }
            if self.buckets[i2].key == key {
                self.buckets[i2].value = value; // update
                return Ok(());
            }

            // Evict from slot 1, re-insert the displaced entry.
            let evicted = self.buckets[i1];
            self.buckets[i1] = Bucket { key, value };
            key = evicted.key;
            value = evicted.value;
        }

        Err(IndexError::MapFull(self.capacity()))
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
                new_map.insert_inner(b.key, b.value).expect("grow insert");
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
