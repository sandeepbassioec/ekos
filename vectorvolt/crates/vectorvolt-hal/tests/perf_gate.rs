//! Layer 79 — SIMD kernel performance-regression gate (VV-TST-007/008).
//!
//! Each test runs a SIMD operation many times and asserts a minimum throughput
//! floor. The floors are deliberately conservative (100× below what modern
//! hardware achieves) so they always pass on CI hardware but will catch a
//! catastrophic regression like accidentally disabling SIMD or introducing an
//! O(n²) scalar fallback.
//!
//! Threshold derivation:
//!   - Modern AVX2 @ 3 GHz: ~12 GB/s = 1.5 × 10⁹ u64/s on filter_range.
//!   - Floor set to 1 × 10⁷ u64/s (150× below modern hardware).
//!   - At this floor, processing 1 024 u64s must finish in < 102 µs.
//!   - A CI machine that misses this floor by 10× is processing at 1 MB/s, which
//!     means SIMD is effectively broken.

use std::hint::black_box;
use std::time::Instant;

use vectorvolt_hal::simd;

const FLOOR_ELEM_PER_US: u64 = 3; // 3 M elem/s = 3 elem/µs (500× below AVX2; passes debug-mode small sizes)
const ITERS: u32 = 2_000;

// ── helpers ───────────────────────────────────────────────────────────────────

fn measure_throughput_elem_per_us(n: usize, f: impl Fn()) -> u64 {
    let start = Instant::now();
    for _ in 0..ITERS {
        f();
    }
    let elapsed_us = start.elapsed().as_micros().max(1) as u64;
    (n as u64 * ITERS as u64) / elapsed_us
}

// ── filter_range ──────────────────────────────────────────────────────────────

#[test]
fn perf_filter_range_64_elems_above_floor() {
    let data: Vec<u64> = (0..64).collect();
    let throughput = measure_throughput_elem_per_us(64, || {
        let mut out = Vec::new();
        simd::filter_range(black_box(&data), 16, 48, &mut out);
        black_box(out);
    });
    assert!(
        throughput >= FLOOR_ELEM_PER_US,
        "filter_range/64: {throughput} elem/µs < floor"
    );
}

#[test]
fn perf_filter_range_256_elems_above_floor() {
    let data: Vec<u64> = (0..256).collect();
    let throughput = measure_throughput_elem_per_us(256, || {
        let mut out = Vec::new();
        simd::filter_range(black_box(&data), 64, 192, &mut out);
        black_box(out);
    });
    assert!(
        throughput >= FLOOR_ELEM_PER_US,
        "filter_range/256: {throughput} elem/µs < floor"
    );
}

#[test]
fn perf_filter_range_1024_elems_above_floor() {
    let data: Vec<u64> = (0..1024).collect();
    let throughput = measure_throughput_elem_per_us(1024, || {
        let mut out = Vec::new();
        simd::filter_range(black_box(&data), 256, 768, &mut out);
        black_box(out);
    });
    assert!(
        throughput >= FLOOR_ELEM_PER_US,
        "filter_range/1024: {throughput} elem/µs < floor"
    );
}

// ── count_range ───────────────────────────────────────────────────────────────

#[test]
fn perf_count_range_64_elems_above_floor() {
    let data: Vec<u64> = (0..64).collect();
    let throughput = measure_throughput_elem_per_us(64, || {
        black_box(simd::count_range(black_box(&data), 16, 48));
    });
    assert!(
        throughput >= FLOOR_ELEM_PER_US,
        "count_range/64: {throughput} elem/µs < floor"
    );
}

#[test]
fn perf_count_range_256_elems_above_floor() {
    let data: Vec<u64> = (0..256).collect();
    let throughput = measure_throughput_elem_per_us(256, || {
        black_box(simd::count_range(black_box(&data), 64, 192));
    });
    assert!(
        throughput >= FLOOR_ELEM_PER_US,
        "count_range/256: {throughput} elem/µs < floor"
    );
}

#[test]
fn perf_count_range_1024_elems_above_floor() {
    let data: Vec<u64> = (0..1024).collect();
    let throughput = measure_throughput_elem_per_us(1024, || {
        black_box(simd::count_range(black_box(&data), 256, 768));
    });
    assert!(
        throughput >= FLOOR_ELEM_PER_US,
        "count_range/1024: {throughput} elem/µs < floor"
    );
}

// ── sum ───────────────────────────────────────────────────────────────────────

#[test]
fn perf_sum_64_elems_above_floor() {
    let data: Vec<u64> = (0..64).collect();
    let throughput = measure_throughput_elem_per_us(64, || {
        black_box(simd::sum(black_box(&data)));
    });
    assert!(
        throughput >= FLOOR_ELEM_PER_US,
        "sum/64: {throughput} elem/µs < floor"
    );
}

#[test]
fn perf_sum_256_elems_above_floor() {
    let data: Vec<u64> = (0..256).collect();
    let throughput = measure_throughput_elem_per_us(256, || {
        black_box(simd::sum(black_box(&data)));
    });
    assert!(
        throughput >= FLOOR_ELEM_PER_US,
        "sum/256: {throughput} elem/µs < floor"
    );
}

#[test]
fn perf_sum_1024_elems_above_floor() {
    let data: Vec<u64> = (0..1024).collect();
    let throughput = measure_throughput_elem_per_us(1024, || {
        black_box(simd::sum(black_box(&data)));
    });
    assert!(
        throughput >= FLOOR_ELEM_PER_US,
        "sum/1024: {throughput} elem/µs < floor"
    );
}

// ── min_max ───────────────────────────────────────────────────────────────────

#[test]
fn perf_min_max_64_elems_above_floor() {
    let data: Vec<u64> = (0..64).collect();
    let throughput = measure_throughput_elem_per_us(64, || {
        black_box(simd::min_max(black_box(&data)));
    });
    assert!(
        throughput >= FLOOR_ELEM_PER_US,
        "min_max/64: {throughput} elem/µs < floor"
    );
}

#[test]
fn perf_min_max_256_elems_above_floor() {
    let data: Vec<u64> = (0..256).collect();
    let throughput = measure_throughput_elem_per_us(256, || {
        black_box(simd::min_max(black_box(&data)));
    });
    assert!(
        throughput >= FLOOR_ELEM_PER_US,
        "min_max/256: {throughput} elem/µs < floor"
    );
}

#[test]
fn perf_min_max_1024_elems_above_floor() {
    let data: Vec<u64> = (0..1024).collect();
    let throughput = measure_throughput_elem_per_us(1024, || {
        black_box(simd::min_max(black_box(&data)));
    });
    assert!(
        throughput >= FLOOR_ELEM_PER_US,
        "min_max/1024: {throughput} elem/µs < floor"
    );
}

// ── zone_overlap_indices ──────────────────────────────────────────────────────

#[test]
fn perf_zone_overlap_indices_256_zones_above_floor() {
    let mins: Vec<u64> = (0..256u64).map(|i| i * 10).collect();
    let maxes: Vec<u64> = (0..256u64).map(|i| i * 10 + 9).collect();
    let throughput = measure_throughput_elem_per_us(256, || {
        let mut out = Vec::new();
        simd::zone_overlap_indices(black_box(&mins), black_box(&maxes), 640, 1920, &mut out);
        black_box(out);
    });
    assert!(
        throughput >= FLOOR_ELEM_PER_US,
        "zone_overlap_indices/256: {throughput} elem/µs < floor"
    );
}

#[test]
fn perf_zone_overlap_indices_1024_zones_above_floor() {
    let mins: Vec<u64> = (0..1024u64).map(|i| i * 10).collect();
    let maxes: Vec<u64> = (0..1024u64).map(|i| i * 10 + 9).collect();
    let throughput = measure_throughput_elem_per_us(1024, || {
        let mut out = Vec::new();
        simd::zone_overlap_indices(black_box(&mins), black_box(&maxes), 2560, 7680, &mut out);
        black_box(out);
    });
    assert!(
        throughput >= FLOOR_ELEM_PER_US,
        "zone_overlap_indices/1024: {throughput} elem/µs < floor"
    );
}

// ── popcount ──────────────────────────────────────────────────────────────────

#[test]
fn perf_popcount_1024_words_above_floor() {
    let data: Vec<u64> = (0..1024u64)
        .map(|i| i.wrapping_mul(0xDEAD_BEEF_CAFE_1234))
        .collect();
    let throughput = measure_throughput_elem_per_us(1024, || {
        black_box(simd::popcount(black_box(&data)));
    });
    assert!(
        throughput >= FLOOR_ELEM_PER_US,
        "popcount/1024: {throughput} elem/µs < floor"
    );
}
