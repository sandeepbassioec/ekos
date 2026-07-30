//! Explicit cache prefetch — L1 / L2 / L3 / streaming.
//!
//! ## What a prefetch actually buys
//!
//! The hardware prefetcher already handles *sequential* access well: it detects
//! a forward stride and pulls lines ahead of you for free. Explicit prefetch
//! wins in the cases it cannot predict:
//!
//! - **Pointer-chasing / indirection.** Reading `descriptor` then following it
//!   to a partition offset. Issue the prefetch for the target the moment you
//!   have the address, do unrelated work, then read — the ~80 ns DRAM latency
//!   overlaps with your work instead of stalling the pipeline.
//! - **Indexed gather.** Walking a bitmap of row offsets: the addresses are
//!   data-dependent, so the prefetcher sees no stride.
//! - **Non-temporal streaming.** A full column scan pollutes L1/L2/L3 with data
//!   you will never re-read, evicting your hot index. [`Locality::Stream`]
//!   loads into a line that is marked for early eviction, preserving cache for
//!   data that matters.
//!
//! ## Latency ladder (why the tier matters)
//!
//! | Target | Cost if hit |
//! |---|---|
//! | L1 | ~1 ns |
//! | L2 | ~4 ns |
//! | L3 | ~15–40 ns |
//! | DRAM | ~80–100 ns |
//!
//! Prefetching into L1 when the data will not be read for a long time is
//! counterproductive — the line gets evicted before use and you paid for
//! nothing. Match the tier to how soon you will touch it.
//!
//! ## Safety
//!
//! Every function here is safe and takes a raw pointer. A prefetch is
//! architecturally a *hint*: it never faults, never traps, and never changes
//! program-visible state. Passing a dangling, null, or unmapped address is
//! defined to be a no-op, which is why these are not `unsafe fn`.
//!
//! ## Usage
//!
//! ```
//! use vectorvolt_hal::prefetch::{prefetch, Locality};
//!
//! let data: Vec<u64> = (0..1024).collect();
//! // Warm the line we will need ~20 instructions from now.
//! prefetch(data.as_ptr().wrapping_add(512), Locality::L1);
//! ```

/// Which cache level a prefetch should target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locality {
    /// Pull into L1 (and L2, L3). Use when the read is imminent — within a few
    /// dozen instructions. Maps to `PREFETCHT0` / `PRFM PLDL1KEEP`.
    L1,
    /// Pull into L2 (and L3), skipping L1. Use when the read is ~hundreds of
    /// instructions away. Maps to `PREFETCHT1` / `PRFM PLDL2KEEP`.
    L2,
    /// Pull into L3 only. Use for bulk staging of a working set you are about
    /// to walk. Maps to `PREFETCHT2` / `PRFM PLDL3KEEP`.
    L3,
    /// Non-temporal: fetch with minimal cache pollution, marked for early
    /// eviction. Use for streaming scans of data read exactly once.
    /// Maps to `PREFETCHNTA` / `PRFM PLDL1STRM`.
    Stream,
}

/// Prefetch the cache line containing `ptr` for reading.
///
/// Never faults — safe for any address, including null or unmapped.
#[inline(always)]
pub fn prefetch<T>(ptr: *const T, locality: Locality) {
    #[cfg(target_arch = "x86_64")]
    {
        use core::arch::x86_64::{
            _MM_HINT_NTA, _MM_HINT_T0, _MM_HINT_T1, _MM_HINT_T2, _mm_prefetch,
        };
        let p = ptr as *const i8;
        // SAFETY: `_mm_prefetch` is architecturally a hint. It cannot fault,
        // cannot trap on an invalid address, and has no observable effect on
        // program state. SSE is baseline on every x86-64 CPU.
        unsafe {
            match locality {
                Locality::L1 => _mm_prefetch::<_MM_HINT_T0>(p),
                Locality::L2 => _mm_prefetch::<_MM_HINT_T1>(p),
                Locality::L3 => _mm_prefetch::<_MM_HINT_T2>(p),
                Locality::Stream => _mm_prefetch::<_MM_HINT_NTA>(p),
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: PRFM is a hint instruction — it cannot fault or trap on an
        // invalid address, and touches no registers (nostack, preserves_flags).
        unsafe {
            match locality {
                Locality::L1 => core::arch::asm!(
                    "prfm pldl1keep, [{p}]", p = in(reg) ptr,
                    options(nostack, preserves_flags)
                ),
                Locality::L2 => core::arch::asm!(
                    "prfm pldl2keep, [{p}]", p = in(reg) ptr,
                    options(nostack, preserves_flags)
                ),
                Locality::L3 => core::arch::asm!(
                    "prfm pldl3keep, [{p}]", p = in(reg) ptr,
                    options(nostack, preserves_flags)
                ),
                Locality::Stream => core::arch::asm!(
                    "prfm pldl1strm, [{p}]", p = in(reg) ptr,
                    options(nostack, preserves_flags)
                ),
            }
        }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        // Portable no-op: correctness never depends on a prefetch.
        let _ = (ptr, locality);
    }
}

/// Prefetch the cache line containing `ptr` with intent to write.
///
/// On x86-64 there is no distinct write-prefetch for normal memory, so this
/// issues an L1 prefetch — the line arrives in a state the store can upgrade.
/// On aarch64 this emits `PRFM PSTL1KEEP`, which requests the line in an
/// exclusive state directly and avoids a second coherence round-trip.
#[inline(always)]
pub fn prefetch_write<T>(ptr: *const T) {
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: hint instruction — cannot fault. See `prefetch`.
        unsafe {
            core::arch::asm!(
                "prfm pstl1keep, [{p}]", p = in(reg) ptr,
                options(nostack, preserves_flags)
            );
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        prefetch(ptr, Locality::L1);
    }
}

/// How many elements ahead to prefetch when streaming a slice of `T`.
///
/// Chosen so the prefetch is issued roughly one DRAM latency (~100 ns) before
/// the data is consumed. At a few elements per nanosecond of scan throughput
/// that lands around 8 cache lines ahead; we express it in elements so callers
/// can use it directly as a slice index offset.
#[inline(always)]
pub const fn stream_distance<T>() -> usize {
    const LOOKAHEAD_LINES: usize = 8;
    let per_line = crate::cache::CACHE_LINE / core::mem::size_of::<T>();
    let per_line = if per_line == 0 { 1 } else { per_line };
    LOOKAHEAD_LINES * per_line
}

/// Issue a prefetch for the element `stream_distance::<T>()` ahead of `idx`,
/// clamped to the slice — the idiomatic way to soften a sequential scan.
///
/// ```
/// use vectorvolt_hal::prefetch::{prefetch_ahead, Locality};
/// let data: Vec<u64> = (0..4096).collect();
/// let mut sum = 0u64;
/// for i in 0..data.len() {
///     prefetch_ahead(&data, i, Locality::L1);
///     sum = sum.wrapping_add(data[i]);
/// }
/// # assert!(sum > 0);
/// ```
#[inline(always)]
pub fn prefetch_ahead<T>(slice: &[T], idx: usize, locality: Locality) {
    let target = idx + stream_distance::<T>();
    if target < slice.len() {
        // SAFETY of the pointer arithmetic: `target` is checked in bounds.
        prefetch(unsafe { slice.as_ptr().add(target) }, locality);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefetch_all_localities_is_a_noop_on_valid_memory() {
        let v: Vec<u64> = (0..256).collect();
        for loc in [Locality::L1, Locality::L2, Locality::L3, Locality::Stream] {
            prefetch(v.as_ptr(), loc);
            prefetch(v.as_ptr().wrapping_add(128), loc);
        }
        prefetch_write(v.as_ptr());
        // Data must be untouched — a prefetch has no program-visible effect.
        assert_eq!(v[0], 0);
        assert_eq!(v[255], 255);
    }

    #[test]
    fn prefetch_tolerates_null_and_dangling() {
        // Architecturally guaranteed not to fault. If this segfaults, the
        // platform is violating the ISA contract.
        prefetch(core::ptr::null::<u64>(), Locality::L1);
        prefetch(usize::MAX as *const u64, Locality::Stream);
    }

    #[test]
    fn stream_distance_scales_with_element_size() {
        // 8 u64 per line × 8 lines
        assert_eq!(stream_distance::<u64>(), 64);
        // 64 u8 per line × 8 lines
        assert_eq!(stream_distance::<u8>(), 512);
        // Oversized elements still get a nonzero distance.
        assert!(stream_distance::<[u8; 4096]>() >= 8);
    }

    #[test]
    fn prefetch_ahead_never_reads_out_of_bounds() {
        let v: Vec<u64> = (0..10).collect();
        // Every index, including ones where lookahead exceeds the slice.
        for i in 0..v.len() {
            prefetch_ahead(&v, i, Locality::L1);
        }
        assert_eq!(v.len(), 10);
    }
}
