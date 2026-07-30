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

    /// Pin this thread to the first recommended writer core before benchmarking.
    ///
    /// On a hybrid CPU an unpinned thread can migrate between a P-core and an
    /// E-core mid-measurement, which shows up as variance rather than as an
    /// obvious error. Compare a run with and without this flag.
    #[arg(long)]
    pub pin: bool,
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
    if simd::is_capped() {
        // Loud, because a capped run is NOT this machine's best and a reader
        // pasting this output elsewhere must not mistake it for hardware limits.
        println!(
            "  SIMD tier      : {}   << CAPPED from {} via {}",
            tier.name(),
            simd::detect().name(),
            simd::TIER_ENV
        );
        println!("  !! Speedups below will read 1.00x — both columns run the same kernel.");
    } else {
        println!("  SIMD tier      : {}", tier.name());
    }
    println!("  u64 lanes/instr: {}", tier.lanes_u64());
    println!("  hw CRC-32C     : {}", simd::has_hw_crc32c());
    println!("  cache line     : {} bytes", vectorvolt_hal::CACHE_LINE);
    println!(
        "  base clock     : {:.2} GHz (RDTSC-calibrated)",
        timer.ghz()
    );
    // ── Topology ──────────────────────────────────────────────────────────────
    let topo = vectorvolt_hal::CpuTopology::detect();
    println!("  topology       : {}", topo.summary());
    if let Some(l1) = topo.cache.l1d_bytes {
        println!("  L1d / L2 / L3  : {}", fmt_cache(&topo.cache));
        let _ = l1;
    }
    if topo.hybrid {
        println!(
            "  P-cores        : {:?}",
            topo.cores_of_class(vectorvolt_hal::CoreClass::Performance)
        );
        println!(
            "  E-cores        : {:?}",
            topo.cores_of_class(vectorvolt_hal::CoreClass::Efficient)
        );
        println!(
            "  !! hybrid CPU — latency figures from this machine are unreliable:\n     \
             a thread migrating P-core -> E-core can swing a measurement ~2x."
        );
    }

    let roles = vectorvolt_hal::CoreRoleMap::derive(&topo);
    println!("\n  Recommended core roles (advice only — nothing is pinned yet)");
    println!("  {}", roles.summary());
    let conflicts = roles.writer_reader_conflicts(&topo);
    if conflicts.is_empty() {
        println!("  writer/reader physical-core isolation: OK");
    } else {
        println!("  !! writer/reader share a physical core: {conflicts:?}");
    }

    if args.no_bench {
        return Ok(());
    }

    // ── Optional pin ──────────────────────────────────────────────────────────
    // Held for the whole benchmark; dropping it would restore the old affinity
    // and quietly un-pin us partway through.
    let _pin_guard = if args.pin {
        match roles.writers.first() {
            Some(&core) => match vectorvolt_hal::pin_current_thread_to(core) {
                Ok(guard) => {
                    let observed = vectorvolt_hal::current_core();
                    println!("\n  pinned to writer core {core} (now running on {observed:?})");
                    if observed != Some(core) {
                        // Affinity takes effect at the next scheduling decision,
                        // so a mismatch here is usually just timing — say so
                        // rather than implying the pin failed.
                        println!("  (not yet migrated — affinity applies at the next reschedule)");
                    }
                    Some(guard)
                }
                Err(e) => {
                    println!("\n  !! pin to core {core} failed: {e}");
                    None
                }
            },
            None => {
                println!("\n  !! no writer cores recommended — nothing to pin to");
                None
            }
        }
    } else {
        println!(
            "\n  unpinned{}",
            if topo.hybrid {
                " — on this hybrid CPU the thread may migrate P<->E mid-run; try --pin"
            } else {
                ""
            }
        );
        None
    };

    // ── Benchmark ─────────────────────────────────────────────────────────────
    // Spread values across the full u64 domain, including above i64::MAX, so the
    // measurement exercises the same range-shift path production queries hit.
    let values: Vec<u64> = (0..args.rows as u64)
        .map(|i| i.wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .collect();
    let lo = u64::MAX / 4;
    let hi = u64::MAX / 2;
    let needle = values[args.rows / 2];

    // Pre-fault and warm the array BEFORE the first timed kernel.
    //
    // Without this the first kernel measured absorbs the first-touch page faults
    // on this freshly allocated buffer, and its `min` varies by ~2x between
    // process launches for reasons that have nothing to do with the kernel. That
    // artifact previously made `count_range` look 2x slower under `--pin`, which
    // was measurement order, not pinning. Two passes: one to fault the pages, one
    // to leave the cache in the state a steady-state loop would see.
    for _ in 0..2 {
        std::hint::black_box(scalar::sum(&values));
    }

    println!(
        "\nScalar vs {} — {} elements, best of {}",
        tier.name(),
        args.rows,
        args.iters
    );
    println!("{}", "─".repeat(62));
    println!(
        "  {:<16} {:>11} {:>11} {:>9} {:>9}",
        "kernel",
        "scalar",
        tier.name(),
        "speedup",
        "jitter"
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
    let s = measure(args.iters, &timer, || {
        scratch.clear();
        scalar::filter_range(&values, lo, hi, &mut scratch);
        scratch.len()
    });
    let v = measure(args.iters, &timer, || {
        scratch.clear();
        simd::filter_range(&values, lo, hi, &mut scratch);
        scratch.len()
    });
    print_row("filter_range", s, v);

    // CRC-32C exercises a different feature bit (SSE4.2) than the u64 kernels.
    let bytes: Vec<u8> = (0..args.rows).map(|i| (i & 0xFF) as u8).collect();
    let s = measure(args.iters.min(3), &timer, || scalar::crc32c(&bytes));
    let v = measure(args.iters, &timer, || simd::crc32c(&bytes));
    print_row("crc32c", s, v);

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
    let s = measure(iters, timer, &mut scalar_fn);
    let v = measure(iters, timer, &mut simd_fn);
    print_row(label, s, v);
}

/// Timing distribution for one kernel.
#[derive(Debug, Clone, Copy)]
struct Sample {
    /// Fastest observed run — the closest estimate of the kernel's true cost,
    /// since every noise source (preemption, interrupts, eviction by another
    /// process, core migration) makes a run slower and never faster.
    min_ns: u64,
    /// 99th percentile. This is where scheduling noise lives.
    p99_ns: u64,
}

impl Sample {
    /// `p99 / min` — how much worse a bad run is than a good one.
    ///
    /// **This is the number core pinning moves, and `min_ns` is not.** Taking the
    /// minimum of many runs already discards the slow samples that a P-core →
    /// E-core migration produces, so a best-of comparison of pinned vs unpinned
    /// shows nothing. Jitter is what a latency SLA actually cares about.
    fn jitter(&self) -> f64 {
        if self.min_ns == 0 {
            return 1.0;
        }
        self.p99_ns as f64 / self.min_ns as f64
    }
}

/// Time `iters` runs and return the distribution.
fn measure<F, R>(iters: usize, timer: &CycleTimer, mut f: F) -> Sample
where
    F: FnMut() -> R,
{
    // One untimed warm-up pass to fault in pages and prime the cache, so the
    // first timed iteration is not penalised for the others' benefit.
    std::hint::black_box(f());

    let mut samples = Vec::with_capacity(iters);
    for _ in 0..iters {
        let sw = Stopwatch::start();
        std::hint::black_box(f());
        samples.push(sw.elapsed_ns(timer));
    }
    samples.sort_unstable();

    Sample {
        min_ns: samples[0],
        p99_ns: samples[(samples.len() * 99 / 100).min(samples.len() - 1)],
    }
}

fn print_row(label: &str, scalar: Sample, simd_s: Sample) {
    let speedup = if simd_s.min_ns > 0 {
        scalar.min_ns as f64 / simd_s.min_ns as f64
    } else {
        f64::INFINITY
    };
    println!(
        "  {:<16} {:>11} {:>11} {:>8.2}x {:>8.2}x",
        label,
        fmt_ns(scalar.min_ns),
        fmt_ns(simd_s.min_ns),
        speedup,
        simd_s.jitter()
    );
}

fn fmt_cache(c: &vectorvolt_hal::CacheHierarchy) -> String {
    fn kb(v: Option<usize>) -> String {
        match v {
            Some(b) if b >= 1024 * 1024 => format!("{:.1} MB", b as f64 / (1024.0 * 1024.0)),
            Some(b) => format!("{} KB", b / 1024),
            None => "?".into(),
        }
    }
    format!(
        "{} / {} / {}  (line {} B)",
        kb(c.l1d_bytes),
        kb(c.l2_bytes),
        kb(c.l3_bytes),
        c.line_bytes
    )
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
