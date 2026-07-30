//! Runtime-dispatched SIMD kernels.
//!
//! ## How dispatch works
//!
//! One binary contains every kernel variant. At first use, [`tier`] runs `cpuid`
//! once, caches the answer in a `OnceLock`, and every subsequent call is a
//! branch on a cached enum — no per-call feature detection, no separate builds,
//! no `-C target-cpu=native` requirement. A binary built on a Zen 2 laptop runs
//! AVX2 kernels there and scalar kernels on a pre-2013 CPU, correctly, with no
//! recompile.
//!
//! ## Contract every kernel upholds
//!
//! Each public function here is bit-exact identical in output to the function of
//! the same name in [`scalar`], which is the oracle. `tests/simd_equivalence.rs`
//! enforces this over randomized data at every length from 0 to 300 — covering
//! every possible vector-tail remainder — plus adversarial values around
//! `i64::MAX` where signed/unsigned comparison bugs live.
//!
//! ## Usage
//!
//! ```
//! use vectorvolt_hal::simd;
//!
//! let values: Vec<u64> = (0..1000).collect();
//! assert_eq!(simd::count_range(&values, 100, 199), 100);
//! assert_eq!(simd::min_max(&values), Some((0, 999)));
//!
//! let mut hits = Vec::new();
//! simd::filter_range(&values, 10, 12, &mut hits);
//! assert_eq!(hits, vec![10, 11, 12]);
//! ```

pub mod scalar;

#[cfg(target_arch = "x86_64")]
pub mod avx2;

use std::sync::OnceLock;

/// Widest SIMD instruction set available on this CPU, as detected at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimdTier {
    /// No vector kernels — portable scalar code.
    Scalar,
    /// 256-bit AVX2. VectorVolt's baseline tier on x86-64.
    Avx2,
    /// 512-bit AVX-512.
    ///
    /// Detection and kernels are **not implemented yet** (tracked as VV-HAL-007);
    /// [`detect`] never returns this variant today. It exists so callers can
    /// match exhaustively without a breaking change when kernels land.
    Avx512,
    /// ARM NEON.
    ///
    /// Kernels **not implemented yet** (VV-HAL-008) — writing untested vector
    /// intrinsics for hardware we cannot execute on would be worse than the
    /// scalar fallback. aarch64 currently reports [`SimdTier::Scalar`].
    Neon,
}

impl SimdTier {
    /// `u64` lanes processed per vector instruction.
    #[inline]
    pub const fn lanes_u64(self) -> usize {
        match self {
            SimdTier::Scalar => 1,
            SimdTier::Avx2 | SimdTier::Neon => 4,
            SimdTier::Avx512 => 8,
        }
    }

    /// Short human-readable name, for logs and `vvctl` output.
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            SimdTier::Scalar => "scalar",
            SimdTier::Avx2 => "avx2",
            SimdTier::Avx512 => "avx512",
            SimdTier::Neon => "neon",
        }
    }
}

static TIER: OnceLock<SimdTier> = OnceLock::new();
static HAS_SSE42: OnceLock<bool> = OnceLock::new();

/// Probe the CPU for the widest supported instruction set.
///
/// Prefer [`tier`], which caches this. Calling `detect` repeatedly re-executes
/// `cpuid`, which is serializing and costs hundreds of cycles.
pub fn detect() -> SimdTier {
    #[cfg(target_arch = "x86_64")]
    {
        // AVX-512 is deliberately not probed — see `SimdTier::Avx512`.
        if std::arch::is_x86_feature_detected!("avx2") {
            SimdTier::Avx2
        } else {
            SimdTier::Scalar
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        SimdTier::Scalar
    }
}

/// The SIMD tier in use, detected once and cached for the process lifetime.
#[inline]
pub fn tier() -> SimdTier {
    *TIER.get_or_init(detect)
}

/// Whether the hardware `crc32` instruction is available.
#[inline]
pub fn has_hw_crc32c() -> bool {
    *HAS_SSE42.get_or_init(|| {
        #[cfg(target_arch = "x86_64")]
        {
            std::arch::is_x86_feature_detected!("sse4.2")
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            false
        }
    })
}

// ── Dispatchers ───────────────────────────────────────────────────────────────

/// Append the index of every element in `lo..=hi` to `out`. Unsigned comparison.
#[inline]
pub fn filter_range(values: &[u64], lo: u64, hi: u64, out: &mut Vec<u32>) {
    #[cfg(target_arch = "x86_64")]
    if tier() == SimdTier::Avx2 {
        // SAFETY: `tier()` confirmed AVX2 support via cpuid.
        unsafe { avx2::filter_range(values, lo, hi, out) };
        return;
    }
    scalar::filter_range(values, lo, hi, out);
}

/// Count elements in `lo..=hi`. Unsigned comparison.
#[inline]
pub fn count_range(values: &[u64], lo: u64, hi: u64) -> usize {
    #[cfg(target_arch = "x86_64")]
    if tier() == SimdTier::Avx2 {
        // SAFETY: `tier()` confirmed AVX2 support via cpuid.
        return unsafe { avx2::count_range(values, lo, hi) };
    }
    scalar::count_range(values, lo, hi)
}

/// `(min, max)` of `values`, or `None` if empty. Unsigned comparison.
///
/// This is the kernel behind zone-map construction, so its correctness on
/// values above `i64::MAX` directly determines whether range pruning is sound.
#[inline]
pub fn min_max(values: &[u64]) -> Option<(u64, u64)> {
    #[cfg(target_arch = "x86_64")]
    if tier() == SimdTier::Avx2 {
        // SAFETY: `tier()` confirmed AVX2 support via cpuid.
        return unsafe { avx2::min_max(values) };
    }
    scalar::min_max(values)
}

/// Wrapping sum of `values`.
#[inline]
pub fn sum(values: &[u64]) -> u64 {
    #[cfg(target_arch = "x86_64")]
    if tier() == SimdTier::Avx2 {
        // SAFETY: `tier()` confirmed AVX2 support via cpuid.
        return unsafe { avx2::sum(values) };
    }
    scalar::sum(values)
}

/// Index of the first element equal to `needle`.
#[inline]
pub fn find_first_eq(values: &[u64], needle: u64) -> Option<usize> {
    #[cfg(target_arch = "x86_64")]
    if tier() == SimdTier::Avx2 {
        // SAFETY: `tier()` confirmed AVX2 support via cpuid.
        return unsafe { avx2::find_first_eq(values, needle) };
    }
    scalar::find_first_eq(values, needle)
}

/// Total set bits across `words`.
///
/// No vector variant: `POPCNT` is already one instruction per word, and SIMD
/// popcount only wins on bitmaps far larger than our index buckets.
#[inline]
pub fn popcount(words: &[u64]) -> u64 {
    scalar::popcount(words)
}

/// CRC-32C (Castagnoli). Hardware-accelerated when `sse4.2` is present.
///
/// **Different polynomial from the WAL's existing CRC-32/ISO-HDLC** — see
/// [`scalar::crc32c`]. Not a drop-in replacement for `crc32fast`.
#[inline]
pub fn crc32c(data: &[u8]) -> u32 {
    #[cfg(target_arch = "x86_64")]
    if has_hw_crc32c() {
        // SAFETY: `has_hw_crc32c()` confirmed SSE4.2 support via cpuid.
        return unsafe { avx2::crc32c(data) };
    }
    scalar::crc32c(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_is_cached_and_stable() {
        let a = tier();
        let b = tier();
        assert_eq!(a, b);
        assert_eq!(a, detect(), "cached tier disagrees with a fresh probe");
    }

    #[test]
    fn lane_counts_are_sane() {
        assert_eq!(SimdTier::Scalar.lanes_u64(), 1);
        assert_eq!(SimdTier::Avx2.lanes_u64(), 4);
        assert_eq!(SimdTier::Avx512.lanes_u64(), 8);
        assert_eq!(SimdTier::Neon.lanes_u64(), 4);
    }

    #[test]
    fn names_are_stable_for_logging() {
        assert_eq!(SimdTier::Avx2.name(), "avx2");
        assert_eq!(SimdTier::Scalar.name(), "scalar");
    }

    #[test]
    fn dispatch_matches_oracle_on_a_basic_case() {
        let v: Vec<u64> = (0..100).collect();
        assert_eq!(count_range(&v, 10, 20), scalar::count_range(&v, 10, 20));
        assert_eq!(min_max(&v), scalar::min_max(&v));
        assert_eq!(sum(&v), scalar::sum(&v));
        assert_eq!(find_first_eq(&v, 55), scalar::find_first_eq(&v, 55));
    }

    #[test]
    fn hw_and_software_crc32c_agree() {
        // Exercises both paths on the same input if hardware CRC is present.
        for input in [
            b"".as_slice(),
            b"a",
            b"123456789",
            b"The quick brown fox jumps over the lazy dog",
        ] {
            assert_eq!(
                crc32c(input),
                scalar::crc32c(input),
                "hardware CRC-32C diverged from software for {input:?}"
            );
        }
    }
}
