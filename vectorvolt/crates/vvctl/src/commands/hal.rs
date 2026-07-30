//! `vvctl hal` — report detected hardware capabilities and measure the SIMD win.
//!
//! Two jobs:
//!
//! 1. **Provenance.** Print what silicon features were actually detected, so any
//!    latency number from `vvctl bench` can be read in context. A p50 measured on
//!    a scalar fallback means something very different from one measured on AVX2.
//! 2. **Verification.** Time every kernel against its scalar twin on the same
//!    data. If the speedup is ~1.0×, the SIMD path silently is not engaging and
//!    that is a bug worth seeing immediately rather than discovering later.

use anyhow::{Result, bail};
use clap::Args;
use vectorvolt_hal::{
    CycleTimer, Stopwatch,
    simd::{self, scalar},
};

#[derive(Args)]
pub struct HalArgs {
    /// Elements per kernel invocation.
    #[arg(short = 'n', long, default_value = "1000000")]
    pub rows: usize,

    /// Timed repetitions per kernel (best-of is reported to cut scheduler noise).
    #[arg(long, default_value = "20")]
    pub iters: usize,

    /// Skip the benchmark; print capabilities only.
    #[arg(long)]
    pub no_bench: bool,
}

pub fn run(args: HalArgs) -> Result<()> {
    if args.rows == 0 {
        bail!("--rows must be > 0");
    }
    if args.iters == 0 {
        bail!("--iters must be > 0");
    }

    // ── Capabilities ──────────────────────────────────────────────────────────
    let tier = simd::tier();
    let timer = CycleTimer::calibrate();

    println!("\nVectorVolt HAL — detected hardware");
    println!("{}", "─".repeat(62));
    println!("  SIMD tier      : {}", tier.name());
    println!("  u64 lanes/instr: {}", tier.lanes_u64());
    println!("  hw CRC-32C     : {}", simd::has_hw_crc32c());
    println!("  cache line     : {} bytes", vectorvolt_hal::CACHE_LINE);
    println!("  base clock     : {:.2} GHz (RDTSC-calibrated)", timer.ghz());
    println!(
        "  logical cores  : {}",
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(0)
    );

    if args.no_bench {
        return Ok(());
    }

    // ── Benchmark ─────────────────────────────────────────────────────────────
    // Spread values across the full u64 domain, including above i64::MAX, so the
    // measurement exercises the same range-shift path production queries hit.
    let values: Vec<u64> = (0..args.rows as u64)
        .map(|i| i.wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .collect();
    let lo = u64::MAX / 4;
    let hi = u64::MAX / 2;
    let needle = values[args.rows / 2];

    println!("\nScalar vs {} — {} elements, best of {}", tier.name(), args.rows, args.iters);
    println!("{}", "─".repeat(62));
    println!(
        "  {:<16} {:>12} {:>12} {:>10}",
        "kernel", "scalar", tier.name(), "speedup"
    );

    let mut scratch = Vec::with_capacity(args.rows);

    bench_row(
        "count_range",
        args.iters,
        &timer,
        || scalar::count_range(&values, lo, hi),
        || simd::count_range(&values, lo, hi),
    );

    bench_row(
        "min_max",
        args.iters,
        &timer,
        || scalar::min_max(&values),
        || simd::min_max(&values),
    );

    bench_row(
        "sum",
        args.iters,
        &timer,
        || scalar::sum(&values),
        || simd::sum(&values),
    );

    bench_row(
        "find_first_eq",
        args.iters,
        &timer,
        || scalar::find_first_eq(&values, needle),
        || simd::find_first_eq(&values, needle),
    );

    // filter_range writes into a Vec, so clear between runs and report separately.
    let scalar_ns = best_of(args.iters, &timer, || {
        scratch.clear();
        scalar::filter_range(&values, lo, hi, &mut scratch);
        scratch.len()
    });
    let simd_ns = best_of(args.iters, &timer, || {
        scratch.clear();
        simd::filter_range(&values, lo, hi, &mut scratch);
        scratch.len()
    });
    print_row("filter_range", scalar_ns, simd_ns, tier.name());

    // CRC-32C exercises a different feature bit (SSE4.2) than the u64 kernels.
    let bytes: Vec<u8> = (0..args.rows).map(|i| (i & 0xFF) as u8).collect();
    let scalar_ns = best_of(args.iters.min(3), &timer, || scalar::crc32c(&bytes));
    let simd_ns = best_of(args.iters, &timer, || simd::crc32c(&bytes));
    print_row("crc32c", scalar_ns, simd_ns, "hw");

    println!(
        "\n  Note: crc32c scalar is bitwise (8 shifts/byte), so its speedup is\n  \
         larger than the vector kernels' — different baseline, not a better win."
    );
    println!();

    Ok(())
}

/// Time both variants and print one comparison row.
fn bench_row<A, B, RA, RB>(
    label: &str,
    iters: usize,
    timer: &CycleTimer,
    mut scalar_fn: A,
    mut simd_fn: B,
) where
    A: FnMut() -> RA,
    B: FnMut() -> RB,
{
    let scalar_ns = best_of(iters, timer, &mut scalar_fn);
    let simd_ns = best_of(iters, timer, &mut simd_fn);
    print_row(label, scalar_ns, simd_ns, simd::tier().name());
}

/// Fastest of `iters` runs, in nanoseconds.
///
/// Best-of rather than mean: we want the cost of the kernel, and every source of
/// noise here (preemption, interrupts, cache eviction by another process) makes a
/// run slower, never faster. The minimum is the closest estimate of the true cost.
fn best_of<F, R>(iters: usize, timer: &CycleTimer, mut f: F) -> u64
where
    F: FnMut() -> R,
{
    // One untimed warm-up pass to fault in pages and prime the cache, so the
    // first timed iteration is not penalised for the others' benefit.
    std::hint::black_box(f());

    let mut best = u64::MAX;
    for _ in 0..iters {
        let sw = Stopwatch::start();
        std::hint::black_box(f());
        let ns = sw.elapsed_ns(timer);
        if ns < best {
            best = ns;
        }
    }
    best
}

fn print_row(label: &str, scalar_ns: u64, simd_ns: u64, tier_name: &str) {
    let speedup = if simd_ns > 0 {
        scalar_ns as f64 / simd_ns as f64
    } else {
        f64::INFINITY
    };
    let _ = tier_name;
    println!(
        "  {:<16} {:>12} {:>12} {:>9.2}x",
        label,
        fmt_ns(scalar_ns),
        fmt_ns(simd_ns),
        speedup
    );
}

fn fmt_ns(ns: u64) -> String {
    if ns < 1_000 {
        format!("{ns} ns")
    } else if ns < 1_000_000 {
        format!("{:.1} µs", ns as f64 / 1_000.0)
    } else {
        format!("{:.2} ms", ns as f64 / 1_000_000.0)
    }
}
