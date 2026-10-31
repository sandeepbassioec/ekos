//! Bump allocator backed by a single mmap'd region (VV-HAL-029).
//!
//! [`BumpArena`] carves allocations from a single contiguous OS reservation.
//! `reset()` rewinds the bump pointer in O(1) — no per-object free, no GC.
//! The type is `!Send` because it is designed to live on one NUMA node.
//!
//! ## Platform notes
//!
//! * **Linux** — uses `mmap(MAP_PRIVATE | MAP_ANONYMOUS)`.
//! * **Other** — falls back to a `Vec<u8>` as backing storage.

use std::marker::PhantomData;

/// A `!Send`, bump-pointer allocator backed by a single OS reservation.
///
/// Allocations are never individually freed; call [`reset`](Self::reset) to
/// reclaim all memory in O(1) once the arena epoch ends.
pub struct BumpArena {
    base: *mut u8,
    capacity: usize,
    offset: usize,
    /// Makes the struct `!Send + !Sync` at the type level.
    _not_send: PhantomData<*mut ()>,
    /// Tracks whether backing memory is from mmap (true) or Vec (false).
    _from_mmap: bool,
    /// When backed by Vec, the Vec allocation is kept alive here.
    _backing: Option<Vec<u8>>,
}

impl BumpArena {
    /// Create a bump arena with at least `capacity` bytes of backing storage.
    pub fn new(capacity: usize) -> Self {
        platform::create(capacity)
    }

    /// Allocate `size` bytes aligned to `align` (must be a power of two).
    ///
    /// Returns `None` when the remaining space is insufficient.
    pub fn alloc(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        debug_assert!(align.is_power_of_two());
        if size == 0 {
            return Some(self.base); // zero-size, return stable address
        }
        let aligned = align_up(self.base as usize + self.offset, align) - self.base as usize;
        let end = aligned.checked_add(size)?;
        if end > self.capacity {
            return None;
        }
        self.offset = end;
        Some(unsafe { self.base.add(aligned) })
    }

    /// Reset the bump pointer to the beginning, reclaiming all prior allocations.
    ///
    /// All pointers returned by [`alloc`](Self::alloc) become dangling.
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    /// Bytes consumed by live allocations.
    pub fn used(&self) -> usize {
        self.offset
    }

    /// Bytes still available.
    pub fn remaining(&self) -> usize {
        self.capacity.saturating_sub(self.offset)
    }

    /// Total capacity of this arena.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Drop for BumpArena {
    fn drop(&mut self) {
        platform::release(self);
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

#[inline]
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

// ── Linux ─────────────────────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
mod platform {
    use super::BumpArena;
    use std::marker::PhantomData;

    pub(super) fn create(capacity: usize) -> BumpArena {
        let cap = capacity.max(4096);
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                cap,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        if ptr == libc::MAP_FAILED || ptr.is_null() {
            panic!("BumpArena: mmap failed for {cap} bytes");
        }
        BumpArena {
            base: ptr as *mut u8,
            capacity: cap,
            offset: 0,
            _not_send: PhantomData,
            _from_mmap: true,
            _backing: None,
        }
    }

    pub(super) fn release(arena: &mut BumpArena) {
        if arena._from_mmap && !arena.base.is_null() {
            unsafe { libc::munmap(arena.base.cast(), arena.capacity) };
            arena.base = std::ptr::null_mut();
        }
    }
}

// ── Non-Linux fallback ────────────────────────────────────────────────────────

#[cfg(not(target_os = "linux"))]
mod platform {
    use super::BumpArena;
    use std::marker::PhantomData;

    pub(super) fn create(capacity: usize) -> BumpArena {
        let cap = capacity.max(4096);
        let mut backing = vec![0u8; cap];
        let base = backing.as_mut_ptr();
        BumpArena {
            base,
            capacity: cap,
            offset: 0,
            _not_send: PhantomData,
            _from_mmap: false,
            _backing: Some(backing),
        }
    }

    pub(super) fn release(_arena: &mut BumpArena) {
        // _backing drop handles deallocation.
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // 1 — new 1MB arena
    #[test]
    fn arena_new_1mb() {
        let a = BumpArena::new(1024 * 1024);
        assert_eq!(a.capacity(), 1024 * 1024);
    }

    // 2 — alloc 64 bytes returns Some
    #[test]
    fn arena_alloc_64_returns_some() {
        let mut a = BumpArena::new(4096);
        assert!(a.alloc(64, 1).is_some());
    }

    // 3 — alloc alignment correct
    #[test]
    fn arena_alloc_alignment_correct() {
        let mut a = BumpArena::new(4096);
        let p = a.alloc(1, 64).expect("alloc");
        assert_eq!(p as usize % 64, 0, "pointer not 64-byte aligned");
    }

    // 4 — used() advances
    #[test]
    fn arena_used_advances() {
        let mut a = BumpArena::new(4096);
        a.alloc(128, 1).unwrap();
        assert!(a.used() >= 128);
    }

    // 5 — remaining() decreases
    #[test]
    fn arena_remaining_decreases() {
        let mut a = BumpArena::new(4096);
        let before = a.remaining();
        a.alloc(128, 1).unwrap();
        assert!(a.remaining() < before);
    }

    // 6 — reset() zeroes used
    #[test]
    fn arena_reset_zeroes_used() {
        let mut a = BumpArena::new(4096);
        a.alloc(256, 1).unwrap();
        a.reset();
        assert_eq!(a.used(), 0);
    }

    // 7 — alloc after reset ok
    #[test]
    fn arena_alloc_after_reset_ok() {
        let mut a = BumpArena::new(4096);
        a.alloc(2048, 1).unwrap();
        a.reset();
        assert!(a.alloc(2048, 1).is_some());
    }

    // 8 — alloc beyond capacity returns None
    #[test]
    fn arena_alloc_beyond_capacity_returns_none() {
        let mut a = BumpArena::new(4096);
        assert!(a.alloc(a.capacity() + 1, 1).is_none());
    }

    // 9 — capacity() is constant
    #[test]
    fn arena_capacity_is_constant() {
        let mut a = BumpArena::new(8192);
        let cap = a.capacity();
        a.alloc(1024, 1).unwrap();
        a.reset();
        assert_eq!(a.capacity(), cap);
    }

    // 10 — multiple allocs non-overlapping
    #[test]
    fn arena_multiple_allocs_non_overlapping() {
        let mut a = BumpArena::new(65536);
        let p1 = a.alloc(64, 1).unwrap() as usize;
        let p2 = a.alloc(64, 1).unwrap() as usize;
        assert!(p2 >= p1 + 64, "allocations overlap: p1={p1:#x} p2={p2:#x}");
    }

    // 11 — alloc 1 byte
    #[test]
    fn arena_alloc_1_byte() {
        let mut a = BumpArena::new(4096);
        assert!(a.alloc(1, 1).is_some());
    }

    // 12 — alloc with align=64
    #[test]
    fn arena_alloc_align_64() {
        let mut a = BumpArena::new(4096);
        let p = a.alloc(16, 64).expect("alloc");
        assert_eq!(p as usize % 64, 0);
    }

    // 13 — drop doesn't leak (no double-free, no panic)
    #[test]
    fn arena_drop_no_panic() {
        let a = BumpArena::new(4096);
        drop(a);
    }
}
