//! SIMD-vs-scalar equivalence (VV-TST-005).
//!
//! The scalar kernels are the correctness oracle. These tests assert that the
//! runtime-dispatched kernel produces **bit-identical** output to its scalar twin
//! for:
//!
//! - every length from 0 to 300, so every possible vector-tail remainder is hit
//!   for 4-lane (AVX2) and 8-lane (AVX-512) widths;
//! - values spanning the full `u64` domain, concentrated around `i64::MAX` where
//!   the signed/unsigned comparison boundary lives — this is the exact class of
//!   bug the range-shift trick in `avx2.rs` exists to prevent;
//! - degenerate ranges (empty, inverted, single-point, full domain).
//!
//! A failure here means a query would return different rows depending on which
//! CPU it ran on. That is the worst possible class of bug in a database, which is
//! why this file is exhaustive rather than illustrative.

use vectorvolt_hal::simd::{self, scalar};

/// Deterministic xorshift64* PRNG. Avoids a `rand` dependency and guarantees
/// that a CI failure is reproducible from the seed alone.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed | 1) // never zero
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A value biased toward the signed/unsigned boundary, where comparison
    /// bugs hide. Roughly half the outputs exceed `i64::MAX`.
    fn next_adversarial(&mut self) -> u64 {
        match self.next() % 6 {
            0 => 0,
            1 => u64::MAX,
            2 => i64::MAX as u64,
            3 => (i64::MAX as u64) + 1,
            4 => self.next() | (1 << 63), // guaranteed high bit
            _ => self.next(),
        }
    }
}

fn assert_all_kernels_agree(values: &[u64], lo: u64, hi: u64, needle: u64, ctx: &str) {
    // filter_range — must match element-for-element, in order.
    let mut got = Vec::new();
    let mut want = Vec::new();
    simd::filter_range(values, lo, hi, &mut got);
    scalar::filter_range(values, lo, hi, &mut want);
    assert_eq!(
        got,
        want,
        "filter_range diverged ({ctx}, len={}, lo={lo}, hi={hi})",
        values.len()
    );

    // count_range must agree with both the SIMD filter and the oracle.
    let count = simd::count_range(values, lo, hi);
    assert_eq!(
        count,
        want.len(),
        "count_range disagreed with filter_range ({ctx}, len={})",
        values.len()
    );
    assert_eq!(
        count,
        scalar::count_range(values, lo, hi),
        "count_range diverged ({ctx}, len={})",
        values.len()
    );

    assert_eq!(
        simd::min_max(values),
        scalar::min_max(values),
        "min_max diverged ({ctx}, len={})",
        values.len()
    );

    assert_eq!(
        simd::sum(values),
        scalar::sum(values),
        "sum diverged ({ctx}, len={})",
        values.len()
    );

    assert_eq!(
        simd::find_first_eq(values, needle),
        scalar::find_first_eq(values, needle),
        "find_first_eq diverged ({ctx}, len={}, needle={needle})",
        values.len()
    );
}

#[test]
fn every_length_up_to_300_agrees_on_sequential_data() {
    for len in 0..=300usize {
        let values: Vec<u64> = (0..len as u64).collect();
        let lo = (len / 4) as u64;
        let hi = (len * 3 / 4) as u64;
        assert_all_kernels_agree(&values, lo, hi, (len / 2) as u64, "sequential");
    }
}

#[test]
fn every_length_up_to_300_agrees_on_random_data() {
    let mut rng = Rng::new(0x5EED_1234_ABCD_0001); // fixed seed — failures reproduce
    for len in 0..=300usize {
        let values: Vec<u64> = (0..len).map(|_| rng.next()).collect();
        let lo = rng.next();
        let hi = rng.next();
        // Normalize so the range is non-empty about half the time.
        let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
        let needle = values.get(len / 2).copied().unwrap_or(0);
        assert_all_kernels_agree(&values, lo, hi, needle, "random");
    }
}

#[test]
fn adversarial_values_around_the_signed_boundary_agree() {
    let mut rng = Rng::new(0xDEAD_BEEF);
    for len in 0..=200usize {
        let values: Vec<u64> = (0..len).map(|_| rng.next_adversarial()).collect();
        // Ranges that straddle i64::MAX — the case naive signed compare breaks.
        for &(lo, hi) in &[
            (0u64, u64::MAX),
            (i64::MAX as u64, (i64::MAX as u64) + 1),
            ((i64::MAX as u64) + 1, u64::MAX),
            (0, i64::MAX as u64),
            (u64::MAX, u64::MAX),
        ] {
            assert_all_kernels_agree(&values, lo, hi, u64::MAX, "adversarial");
        }
    }
}

#[test]
fn all_values_above_i64_max_are_found() {
    // The specific regression the range shift prevents: treated as signed,
    // every one of these is negative and would sort below `lo`.
    let values = vec![
        u64::MAX,
        u64::MAX - 1,
        (i64::MAX as u64) + 1,
        1 << 63,
        (1 << 63) + 12345,
    ];
    let count = simd::count_range(&values, (i64::MAX as u64) + 1, u64::MAX);
    assert_eq!(count, 5, "high-bit values were not matched");
    assert_eq!(
        count,
        scalar::count_range(&values, (i64::MAX as u64) + 1, u64::MAX)
    );

    assert_eq!(simd::min_max(&values), Some((1 << 63, u64::MAX)));
}

#[test]
fn inverted_and_empty_ranges_yield_nothing() {
    let values: Vec<u64> = (0..64).collect();
    // hi < lo — no element can satisfy both bounds.
    assert_eq!(simd::count_range(&values, 50, 10), 0);
    let mut out = Vec::new();
    simd::filter_range(&values, 50, 10, &mut out);
    assert!(out.is_empty());

    // Empty input.
    assert_eq!(simd::count_range(&[], 0, u64::MAX), 0);
    assert_eq!(simd::min_max(&[]), None);
    assert_eq!(simd::sum(&[]), 0);
    assert_eq!(simd::find_first_eq(&[], 1), None);
}

#[test]
fn single_point_range_matches_exact_duplicates() {
    let values = vec![7u64, 1, 7, 9, 7, 7, 2];
    let mut out = Vec::new();
    simd::filter_range(&values, 7, 7, &mut out);
    assert_eq!(out, vec![0, 2, 4, 5]);
    assert_eq!(simd::count_range(&values, 7, 7), 4);
    // find_first_eq must return the FIRST, not any.
    assert_eq!(simd::find_first_eq(&values, 7), Some(0));
}

#[test]
fn filter_range_appends_without_clearing() {
    // Callers reuse a scratch Vec across partitions; the kernel must append.
    let values: Vec<u64> = (0..16).collect();
    let mut out = vec![999u32];
    simd::filter_range(&values, 0, 2, &mut out);
    assert_eq!(
        out,
        vec![999, 0, 1, 2],
        "kernel clobbered existing contents"
    );
}

#[test]
fn sum_wraps_identically_to_scalar() {
    // Overflow must wrap, not panic, and must wrap the same way in both paths.
    let values = vec![u64::MAX; 37];
    assert_eq!(simd::sum(&values), scalar::sum(&values));
    let values = vec![u64::MAX / 2; 100];
    assert_eq!(simd::sum(&values), scalar::sum(&values));
}

#[test]
fn crc32c_agrees_across_paths_on_all_lengths() {
    let mut rng = Rng::new(0xC0FFEE);
    for len in 0..=300usize {
        let data: Vec<u8> = (0..len).map(|_| (rng.next() & 0xFF) as u8).collect();
        assert_eq!(
            simd::crc32c(&data),
            scalar::crc32c(&data),
            "crc32c diverged at len={len}"
        );
    }
}

#[test]
fn dispatch_reports_a_usable_tier() {
    let tier = simd::tier();
    assert!(tier.lanes_u64() >= 1);
    // On x86-64 CI and dev machines we expect AVX2; anywhere else, scalar is
    // acceptable. Either way the tier must be one we have kernels for.
    assert!(
        matches!(tier, simd::SimdTier::Scalar | simd::SimdTier::Avx2),
        "dispatch selected {tier:?}, which has no implemented kernels"
    );
}
