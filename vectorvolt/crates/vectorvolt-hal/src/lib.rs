//! `vectorvolt-hal` — the silicon layer.
//!
//! VectorVolt's performance claims rest on controlling things most databases
//! leave to the compiler and the OS: which cache level data lands in, which core
//! a thread runs on, and how many values a single instruction touches. This crate
//! is where that control lives. Every layer above it — index probes, column
//! scans, zone-map construction — calls into here for its hot loops.
//!
//! ## Modules
//!
//! | Module | Controls |
//! |---|---|
//! | [`cache`] | Cache-line alignment and false-sharing elimination |
//! | [`prefetch`] | Explicit L1 / L2 / L3 / streaming prefetch |
//! | [`simd`] | Runtime-dispatched AVX2 and scalar kernels |
//! | [`timer`] | `RDTSC` cycle-accurate measurement |
//!
//! ## Design rules this crate follows
//!
//! 1. **SIMD is never required for correctness.** Every vector kernel has a
//!    scalar twin that is the correctness oracle, and dispatch falls back to it
//!    on any CPU lacking the feature. A binary built here runs anywhere.
//! 2. **Feature detection happens once.** `cpuid` is serializing and costs
//!    hundreds of cycles; it runs at first use and is cached for the process.
//! 3. **Prefetch is always safe.** Prefetch instructions are architectural hints
//!    that cannot fault, so those functions take raw pointers without `unsafe`.
//! 4. **`unsafe` is confined and justified.** It appears only where a
//!    `#[target_feature]` intrinsic requires it, and every block carries a
//!    `SAFETY` comment naming the runtime check that makes it sound.
//!
//! ## Usage
//!
//! ```
//! use vectorvolt_hal::{simd, CycleTimer, Stopwatch};
//!
//! println!("{}", vectorvolt_hal::describe());
//!
//! let column: Vec<u64> = (0..10_000).collect();
//! let timer = CycleTimer::calibrate();
//! let sw = Stopwatch::start();
//! let matches = simd::count_range(&column, 5_000, 5_999);
//! let ns = sw.elapsed_ns(&timer);
//!
//! assert_eq!(matches, 1_000);
//! # let _ = ns;
//! ```

pub mod affinity;
pub mod cache;
pub mod prefetch;
pub mod simd;
pub mod timer;
pub mod topology;

pub use affinity::{
    AffinityError, AffinityGuard, cached_topology, current_core, pin_current_thread_to,
    pin_current_thread_to_set,
};
pub use cache::{CACHE_LINE, CacheAligned, CachePadded};
pub use prefetch::{Locality, prefetch, prefetch_ahead, prefetch_write};
pub use simd::SimdTier;
pub use timer::{CycleTimer, Stopwatch, cycles};
pub use topology::{CacheHierarchy, CoreClass, CoreRoleMap, CpuTopology, LogicalCore, NumaNode};

/// One-line summary of detected hardware capabilities.
///
/// Emitted at server startup and by `vvctl version` so a latency report always
/// carries the hardware context that produced it — a p50 measured on a scalar
/// fallback means something very different from one measured on AVX2.
pub fn describe() -> String {
    let tier = simd::tier();
    let cap = if simd::is_capped() {
        // Never let a capped measurement look like the machine's best.
        format!(
            " (CAPPED from {} via {})",
            simd::detect().name(),
            simd::TIER_ENV
        )
    } else {
        String::new()
    };
    format!(
        "simd={}{} lanes={} hw_crc32c={} cache_line={}B cores={}",
        tier.name(),
        cap,
        tier.lanes_u64(),
        simd::has_hw_crc32c(),
        CACHE_LINE,
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_reports_detected_hardware() {
        let d = describe();
        assert!(d.contains("simd="), "missing simd tier: {d}");
        assert!(d.contains("cache_line=64B"), "unexpected cache line: {d}");
        // Whatever tier we detected must be named in the output.
        assert!(d.contains(simd::tier().name()), "tier not reported: {d}");
    }

    #[test]
    fn reexports_are_reachable() {
        assert_eq!(CACHE_LINE, 64);
        let _ = CacheAligned::new(0u64);
        let _ = CachePadded::new(0u64);
        prefetch(&0u64 as *const u64, Locality::L1);
        let _ = cycles();
    }
}
