//! Layer 79 — Criterion SIMD kernel benchmarks (VV-TST-006).
//!
//! Measures the throughput of the HAL SIMD kernels introduced in S2
//! (filter_range, count_range, sum, min_max, zone_overlap_indices).
//!
//! Run with:
//! ```text
//! cargo bench -p vectorvolt-hal
//! ```
//!
//! Save a baseline before a change:
//! ```text
//! cargo bench -p vectorvolt-hal -- --save-baseline before
//! ```
//! Then compare after:
//! ```text
//! cargo bench -p vectorvolt-hal -- --baseline before
//! ```

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use vectorvolt_hal::simd;

// ── helpers ───────────────────────────────────────────────────────────────────

fn make_data(n: usize) -> Vec<u64> {
    (0..n as u64).collect()
}

fn make_zones(n: usize) -> (Vec<u64>, Vec<u64>) {
    let mins: Vec<u64> = (0..n as u64).map(|i| i * 10).collect();
    let maxes: Vec<u64> = (0..n as u64).map(|i| i * 10 + 9).collect();
    (mins, maxes)
}

// ── filter_range ──────────────────────────────────────────────────────────────

fn bench_filter_range(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd/filter_range");
    for &n in &[64usize, 256, 1024, 4096, 16384] {
        let data = make_data(n);
        let lo = (n / 4) as u64;
        let hi = (n * 3 / 4) as u64;
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let mut out = Vec::new();
                simd::filter_range(black_box(&data), black_box(lo), black_box(hi), &mut out);
                black_box(out);
            });
        });
    }
    group.finish();
}

// ── count_range ───────────────────────────────────────────────────────────────

fn bench_count_range(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd/count_range");
    for &n in &[64usize, 256, 1024, 4096, 16384] {
        let data = make_data(n);
        let lo = (n / 4) as u64;
        let hi = (n * 3 / 4) as u64;
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                black_box(simd::count_range(
                    black_box(&data),
                    black_box(lo),
                    black_box(hi),
                ))
            });
        });
    }
    group.finish();
}

// ── sum ───────────────────────────────────────────────────────────────────────

fn bench_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd/sum");
    for &n in &[64usize, 256, 1024, 4096, 16384] {
        let data = make_data(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(simd::sum(black_box(&data))));
        });
    }
    group.finish();
}

// ── min_max ───────────────────────────────────────────────────────────────────

fn bench_min_max(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd/min_max");
    for &n in &[64usize, 256, 1024, 4096, 16384] {
        let data = make_data(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(simd::min_max(black_box(&data))));
        });
    }
    group.finish();
}

// ── zone_overlap_indices ──────────────────────────────────────────────────────

fn bench_zone_overlap_indices(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd/zone_overlap_indices");
    for &n in &[64usize, 256, 1024, 4096] {
        let (mins, maxes) = make_zones(n);
        let lo = (n as u64 * 10) / 4;
        let hi = (n as u64 * 10) * 3 / 4;
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let mut out = Vec::new();
                simd::zone_overlap_indices(
                    black_box(&mins),
                    black_box(&maxes),
                    black_box(lo),
                    black_box(hi),
                    &mut out,
                );
                black_box(out);
            });
        });
    }
    group.finish();
}

// ── popcount ──────────────────────────────────────────────────────────────────

fn bench_popcount(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd/popcount");
    for &n in &[64usize, 256, 1024, 4096] {
        let data: Vec<u64> = (0..n as u64)
            .map(|i| i.wrapping_mul(0xDEAD_BEEF_CAFE_1234))
            .collect();
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| black_box(simd::popcount(black_box(&data))));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_filter_range,
    bench_count_range,
    bench_sum,
    bench_min_max,
    bench_zone_overlap_indices,
    bench_popcount,
);
criterion_main!(benches);
