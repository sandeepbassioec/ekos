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
//! ## Capability is fixed for the process lifetime
//!
//! [`tier`] caches its answer in a `OnceLock`. The CPU cannot gain or lose AVX2 while we run, so
//! this is sound — but it has one consequence worth stating: **a hardware change requires a process
//! restart to take effect.** Resize a VM from a host with AVX-512 to one without, and a
//! still-running process keeps the tier it detected at startup.
//!
//! In practice this is safe. Hypervisors mask `cpuid` to the lowest common denominator across a
//! migration pool precisely so a live-migrated process cannot lose an instruction set mid-execution,
//! and AWS does not live-migrate between instance types. Restart-to-adopt is the correct and
//! intended behaviour, not a limitation to work around.
//!
//! ## This module is the reference shape for `Reconciled<T>` (VV-CFG-035)
//!
//! The trio [`detect`] / [`tier`] / [`is_capped`] is a worked example of the pattern every
//! hardware-dependent setting will need:
//!
//! | Here | General form | Meaning |
//! |---|---|---|
//! | [`detect`] | `available` | what the hardware actually offers |
//! | [`tier`] | `effective` | what we will really use |
//! | [`is_capped`] | `was_clamped` | whether the two differ, so it can be reported |
//!
//! Core pinning, arena sizing, shard count, and huge-page reservation all need the same treatment:
//! an *intended* value from config, clamped against live hardware, with the clamp surfaced rather
//! than silently applied. The difference is that those clamps are *lossy* — a config asking to pin
//! to core 40 on a 4-core box has to be reduced, and an operator must be told. Here the "clamp" is
//! only ever a deliberate downward override, so it cannot surprise anyone.
//!
//! See `docs/vectorvolt-tasks/LAYER-18-DEPLOYMENT-AUTOCONFIG.md`.
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
//!
//! // The reconciliation trio: available / effective / was-it-clamped.
//! let available = simd::detect();
//! let effective = simd::tier();
//! assert!(!simd::is_capped() || effective != available);
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

/// Environment variable that caps the SIMD tier.
///
/// See [`tier`] for why this exists and why it can only cap downward.
pub const TIER_ENV: &str = "VV_SIMD_TIER";

/// Probe the CPU for the widest supported instruction set.
///
/// Ignores [`TIER_ENV`] — this is the raw hardware answer. Use [`tier`] for the
/// value the dispatchers actually honour.
///
/// Prefer [`tier`], which caches. Calling `detect` repeatedly re-executes
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

/// Rank used to compare tiers. Higher is wider.
#[inline]
const fn rank(t: SimdTier) -> u8 {
    match t {
        SimdTier::Scalar => 0,
        SimdTier::Neon => 1,
        SimdTier::Avx2 => 2,
        SimdTier::Avx512 => 3,
    }
}

/// Parse a tier name, case-insensitively. `None` if unrecognised.
fn parse_tier(s: &str) -> Option<SimdTier> {
    match s.trim().to_ascii_lowercase().as_str() {
        "scalar" | "none" | "off" => Some(SimdTier::Scalar),
        "neon" => Some(SimdTier::Neon),
        "avx2" => Some(SimdTier::Avx2),
        "avx512" | "avx-512" => Some(SimdTier::Avx512),
        _ => None,
    }
}

/// The SIMD tier the dispatchers use: detected hardware, optionally capped by
/// the `VV_SIMD_TIER` environment variable. Resolved once, cached for the
/// process lifetime.
///
/// ## Why an override exists
///
/// Two problems come from pure auto-detection:
///
/// 1. **Untested fallbacks.** On any modern CPU the scalar path never executes,
///    so it rots silently until someone runs on old hardware.
/// 2. **Incomparable measurements.** Comparing AVX2 on one machine against
///    scalar on another confounds the kernel with the cache hierarchy, clock,
///    and memory bandwidth. To attribute a difference to the *kernel*, you must
///    vary only the kernel — same box, same cache state.
///
/// So: `VV_SIMD_TIER=scalar vvctl hal` forces the scalar path on hardware that
/// would otherwise pick AVX2, making an honest same-box comparison possible.
///
/// ## It can only cap downward
///
/// The variable sets a **ceiling**, never a floor. Requesting a tier the CPU
/// lacks clamps to what is actually available rather than executing an illegal
/// instruction. Forcing *down* always works; forcing *up* is physically
/// impossible and is silently clamped — [`describe`](crate::describe) and
/// `vvctl hal` always print the effective tier, so the clamp is visible rather
/// than mysterious.
///
/// An unrecognised value is ignored (treated as no cap) rather than panicking:
/// a typo'd env var should not take down a database.
#[inline]
pub fn tier() -> SimdTier {
    *TIER.get_or_init(|| {
        let detected = detect();
        match std::env::var(TIER_ENV).ok().as_deref().and_then(parse_tier) {
            // Cap downward only. `min` by rank, so an over-request clamps.
            Some(requested) if rank(requested) < rank(detected) => requested,
            _ => detected,
        }
    })
}

/// True if [`tier`] was capped below what the hardware supports.
///
/// Worth surfacing in any benchmark report — a figure measured under a cap is
/// not this machine's best, and a reader needs to know that.
#[inline]
pub fn is_capped() -> bool {
    rank(tier()) < rank(detect())
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
        assert_eq!(a, b, "tier() is not stable across calls");
        // Without VV_SIMD_TIER set, tier() must equal raw detection. If the test
        // runner sets the cap, tier() may legitimately be lower.
        if std::env::var(TIER_ENV).is_err() {
            assert_eq!(a, detect(), "cached tier disagrees with a fresh probe");
            assert!(!is_capped());
        }
    }

    #[test]
    fn tier_names_round_trip_through_parsing() {
        for t in [
            SimdTier::Scalar,
            SimdTier::Neon,
            SimdTier::Avx2,
            SimdTier::Avx512,
        ] {
            assert_eq!(
                parse_tier(t.name()),
                Some(t),
                "{} failed round-trip",
                t.name()
            );
        }
    }

    #[test]
    fn parse_tier_is_lenient_about_form_and_strict_about_nonsense() {
        assert_eq!(parse_tier("AVX2"), Some(SimdTier::Avx2));
        assert_eq!(parse_tier("  avx-512 "), Some(SimdTier::Avx512));
        assert_eq!(parse_tier("off"), Some(SimdTier::Scalar));
        assert_eq!(parse_tier("none"), Some(SimdTier::Scalar));
        // A typo must be ignored, not panic — this runs inside a database.
        assert_eq!(parse_tier("avx3"), None);
        assert_eq!(parse_tier(""), None);
    }

    #[test]
    fn rank_orders_tiers_by_width() {
        assert!(rank(SimdTier::Scalar) < rank(SimdTier::Neon));
        assert!(rank(SimdTier::Neon) < rank(SimdTier::Avx2));
        assert!(rank(SimdTier::Avx2) < rank(SimdTier::Avx512));
    }

    #[test]
    fn cap_logic_only_ever_lowers() {
        // Replicates tier()'s resolution without touching the process-wide
        // OnceLock, so this stays independent of test execution order.
        let resolve = |detected: SimdTier, requested: Option<SimdTier>| match requested {
            Some(r) if rank(r) < rank(detected) => r,
            _ => detected,
        };

        // Downward: honoured.
        assert_eq!(
            resolve(SimdTier::Avx2, Some(SimdTier::Scalar)),
            SimdTier::Scalar
        );
        // Upward: clamped to hardware, never granted.
        assert_eq!(
            resolve(SimdTier::Avx2, Some(SimdTier::Avx512)),
            SimdTier::Avx2
        );
        // Equal: no change.
        assert_eq!(
            resolve(SimdTier::Avx2, Some(SimdTier::Avx2)),
            SimdTier::Avx2
        );
        // Unset: hardware wins.
        assert_eq!(resolve(SimdTier::Avx2, None), SimdTier::Avx2);
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
