//! Cache-line geometry and alignment primitives.
//!
//! ## Why this matters
//!
//! A CPU never reads one byte from memory — it reads a whole **cache line**
//! (64 bytes on every x86-64 and on aarch64 Apple/Graviton parts). Two things
//! follow, and both are latency-critical:
//!
//! 1. **False sharing.** If two threads write to two different fields that
//!    happen to land in the same cache line, the line ping-pongs between their
//!    L1 caches over the coherence fabric. Each bounce costs ~100 ns — worse
//!    than a DRAM read, for data that was already in cache. [`CachePadded`]
//!    forces one-field-per-line so this cannot happen.
//!
//! 2. **Straddling.** A 64-byte struct at an unaligned address occupies *two*
//!    cache lines, doubling the fetch cost of touching it. `#[repr(align(64))]`
//!    guarantees a hot struct is fetched with one line fill.
//!
//! ## Usage
//!
//! ```
//! use vectorvolt_hal::cache::{CachePadded, CACHE_LINE};
//! use std::sync::atomic::{AtomicU64, Ordering};
//!
//! // Per-core counters that never invalidate each other's cache lines.
//! let counters: Vec<CachePadded<AtomicU64>> =
//!     (0..4).map(|_| CachePadded::new(AtomicU64::new(0))).collect();
//!
//! counters[0].fetch_add(1, Ordering::Relaxed);
//! assert_eq!(counters[0].load(Ordering::Relaxed), 1);
//! assert_eq!(std::mem::align_of_val(&counters[0]), CACHE_LINE);
//! ```

use core::ops::{Deref, DerefMut};

/// Cache-line size in bytes.
///
/// 64 on x86-64 (all vendors) and on aarch64 implementations we target
/// (Graviton, Apple Silicon, Ampere). Some POWER parts use 128; if VectorVolt
/// is ever ported there this constant is the single place to change.
pub const CACHE_LINE: usize = 64;

/// Typical L1d capacity, used only as a sizing default before topology
/// detection runs. Never used for correctness.
pub const L1D_DEFAULT_BYTES: usize = 32 * 1024;
/// Typical L2 capacity default. See [`L1D_DEFAULT_BYTES`].
pub const L2_DEFAULT_BYTES: usize = 1024 * 1024;
/// Typical L3 capacity default. See [`L1D_DEFAULT_BYTES`].
pub const L3_DEFAULT_BYTES: usize = 32 * 1024 * 1024;

/// A value aligned to a cache-line boundary.
///
/// Guarantees the value starts at an address divisible by [`CACHE_LINE`], so
/// reading it costs exactly one line fill instead of two. Unlike
/// [`CachePadded`], this does **not** pad the tail — a smaller value may share
/// the remainder of its line with whatever the allocator places next. Use this
/// for read-mostly data; use [`CachePadded`] for anything written concurrently.
#[repr(C, align(64))]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CacheAligned<T>(pub T);

impl<T> CacheAligned<T> {
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        CacheAligned(value)
    }

    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for CacheAligned<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for CacheAligned<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

/// A value that occupies a whole cache line to itself — eliminates false sharing.
///
/// The alignment attribute makes the type's *size* round up to a multiple of 64
/// as well as its alignment, so consecutive `CachePadded<T>` in a `Vec` are
/// guaranteed to sit on distinct cache lines. That is exactly what you want for
/// per-core counters, per-shard sequence numbers, and producer/consumer cursors
/// in a ring buffer.
///
/// Costs 64 bytes minimum per value. Only use it for genuinely
/// concurrently-written data; padding cold fields wastes cache.
#[repr(C, align(64))]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CachePadded<T> {
    value: T,
}

impl<T> CachePadded<T> {
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        CachePadded { value }
    }

    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.value
    }

    #[inline(always)]
    pub fn get(&self) -> &T {
        &self.value
    }

    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> Deref for CachePadded<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for CachePadded<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

/// Number of cache lines a byte-length spans.
#[inline(always)]
pub const fn lines_for(bytes: usize) -> usize {
    bytes.div_ceil(CACHE_LINE)
}

/// True if `ptr` sits on a cache-line boundary.
#[inline(always)]
pub fn is_line_aligned<T>(ptr: *const T) -> bool {
    (ptr as usize).is_multiple_of(CACHE_LINE)
}

/// True if the object at `ptr` of `len` bytes fits in a single cache line.
#[inline(always)]
pub fn fits_one_line<T>(ptr: *const T, len: usize) -> bool {
    let start = ptr as usize;
    start / CACHE_LINE == (start + len.saturating_sub(1)) / CACHE_LINE
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::AtomicU64;

    #[test]
    fn cache_padded_is_line_sized_and_aligned() {
        assert_eq!(core::mem::align_of::<CachePadded<u8>>(), CACHE_LINE);
        assert_eq!(core::mem::size_of::<CachePadded<u8>>(), CACHE_LINE);
        assert_eq!(core::mem::size_of::<CachePadded<u64>>(), CACHE_LINE);
    }

    #[test]
    fn cache_aligned_is_aligned() {
        assert_eq!(core::mem::align_of::<CacheAligned<u32>>(), CACHE_LINE);
    }

    #[test]
    fn padded_neighbours_do_not_share_a_line() {
        // The whole point: adjacent entries must land on different lines.
        let v: Vec<CachePadded<AtomicU64>> =
            (0..8).map(|_| CachePadded::new(AtomicU64::new(0))).collect();
        for w in v.windows(2) {
            let a = &w[0] as *const _ as usize;
            let b = &w[1] as *const _ as usize;
            assert_ne!(
                a / CACHE_LINE,
                b / CACHE_LINE,
                "adjacent CachePadded shared a cache line — false sharing possible"
            );
        }
    }

    #[test]
    fn deref_gives_access_to_inner() {
        let c = CachePadded::new(AtomicU64::new(41));
        c.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        assert_eq!(c.load(core::sync::atomic::Ordering::Relaxed), 42);
    }

    #[test]
    fn lines_for_rounds_up() {
        assert_eq!(lines_for(0), 0);
        assert_eq!(lines_for(1), 1);
        assert_eq!(lines_for(64), 1);
        assert_eq!(lines_for(65), 2);
        assert_eq!(lines_for(128), 2);
    }

    #[test]
    fn alignment_predicates_agree_with_reality() {
        let x = CacheAligned::new(0u64);
        assert!(is_line_aligned(&x as *const _));
        assert!(fits_one_line(&x as *const _, 8));
    }
}
