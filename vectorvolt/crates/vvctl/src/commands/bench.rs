use std::time::Instant;

use anyhow::{Result, bail};
use clap::Args;
use vectorvolt_engine::TableEngine;

const FLUSH_THRESHOLD: usize = 1_048_576;

/// `vvctl bench` — micro-benchmark INSERT, GET, and SCAN latency.
///
/// Runs entirely in-process against a temporary on-disk table so the numbers
/// reflect real WAL + mmap + CuckooMap overhead — no mocking.
#[derive(Args)]
pub struct BenchArgs {
    /// Number of rows to insert (default 100 000).
    #[arg(short = 'n', long, default_value = "100000")]
    pub rows: usize,

    /// Warm-up rows (discarded from percentile calculation).
    #[arg(long, default_value = "1000")]
    pub warmup: usize,

    /// Flush every N inserts (0 = manual flush only at the end).
    #[arg(long, default_value = "0")]
    pub flush_every: usize,

    /// Skip GET benchmark.
    #[arg(long)]
    pub no_get: bool,

    /// Skip SCAN benchmark.
    #[arg(long)]
    pub no_scan: bool,
}

pub fn run(args: BenchArgs) -> Result<()> {
    if args.rows == 0 {
        bail!("--rows must be > 0");
    }
    if args.warmup >= args.rows {
        bail!("--warmup must be < --rows");
    }

    let dir = tempfile::tempdir()?;
    let schema = vec!["id".to_owned(), "value".to_owned(), "ts".to_owned()];
    let engine = TableEngine::open(schema, dir.path(), FLUSH_THRESHOLD)?;

    // ── INSERT ────────────────────────────────────────────────────────────────
    println!("\nVectorVolt in-process micro-benchmark");
    println!("rows={} warmup={} (release build required for real numbers)", args.rows, args.warmup);
    println!("{}", "─".repeat(60));
    println!("INSERT includes WAL write() syscall (~4-6 µs floor on Windows).");
    println!("GET uses lazy O(1) mmap reads — sub-microsecond at p50.");
    println!("SCAN uses full column decode + zone pruning — cost scales with partition size.");

    let insert_nanos = bench_inserts(&engine, args.rows, args.warmup, args.flush_every)?;

    // Flush everything to disk before reads (creates one partition with all rows).
    engine.flush()?;

    // ── GET ───────────────────────────────────────────────────────────────────
    if !args.no_get {
        let get_nanos = bench_gets(&engine, args.rows, args.warmup)?;
        print_section("GET (pk lookup)", &get_nanos);
    }

    // ── SCAN ──────────────────────────────────────────────────────────────────
    if !args.no_scan {
        let scan_nanos = bench_scans(&engine, args.rows, args.warmup)?;
        print_section("SCAN (col range)", &scan_nanos);
    }

    print_section("INSERT (WAL + index)", &insert_nanos);
    println!();

    Ok(())
}

// ── Benchmark kernels ─────────────────────────────────────────────────────────

fn bench_inserts(
    engine: &TableEngine,
    rows: usize,
    warmup: usize,
    flush_every: usize,
) -> Result<Vec<u64>> {
    let mut samples = Vec::with_capacity(rows - warmup);

    for i in 0..rows {
        let pk = i as u64 + 1;
        let ts = pk * 1_000;

        let t0 = Instant::now();
        engine.insert(pk, vec![pk, pk * 7, ts])?;
        let elapsed = t0.elapsed().as_nanos() as u64;

        if flush_every > 0 && (i + 1) % flush_every == 0 {
            engine.flush()?;
        }

        if i >= warmup {
            samples.push(elapsed);
        }
    }

    Ok(samples)
}

fn bench_gets(engine: &TableEngine, rows: usize, warmup: usize) -> Result<Vec<u64>> {
    let measured = rows - warmup;
    let mut samples = Vec::with_capacity(measured);

    for i in 0..measured {
        let pk = (i % rows) as u64 + 1;
        let t0 = Instant::now();
        let _ = engine.get(pk)?;
        samples.push(t0.elapsed().as_nanos() as u64);
    }

    Ok(samples)
}

fn bench_scans(engine: &TableEngine, rows: usize, warmup: usize) -> Result<Vec<u64>> {
    let measured = rows - warmup;
    let window = (rows / 100).max(1) as u64; // scan 1% of the value range
    let mut samples = Vec::with_capacity(measured);

    for i in 0..measured {
        let lo = (i as u64 * 7) % rows as u64 + 1;
        let hi = lo + window;
        let t0 = Instant::now();
        let _ = engine.scan(1, lo, hi)?;
        samples.push(t0.elapsed().as_nanos() as u64);
    }

    Ok(samples)
}

// ── Reporting ─────────────────────────────────────────────────────────────────

fn print_section(label: &str, samples: &[u64]) {
    if samples.is_empty() {
        return;
    }

    let stats = Stats::compute(samples);
    let ops_per_sec = 1_000_000_000u64.checked_div(stats.mean_ns).unwrap_or(u64::MAX);

    println!();
    println!("  {label}");
    println!("  ├─ samples   : {}", samples.len());
    println!("  ├─ mean      : {}", fmt_ns(stats.mean_ns));
    println!("  ├─ p50       : {}", fmt_ns(stats.p50));
    println!("  ├─ p99       : {}", fmt_ns(stats.p99));
    println!("  ├─ p999      : {}", fmt_ns(stats.p999));
    println!("  ├─ min       : {}", fmt_ns(stats.min));
    println!("  ├─ max       : {}", fmt_ns(stats.max));
    println!("  └─ throughput: {} ops/sec", fmt_ops(ops_per_sec));
}

struct Stats {
    mean_ns: u64,
    p50: u64,
    p99: u64,
    p999: u64,
    min: u64,
    max: u64,
}

impl Stats {
    fn compute(samples: &[u64]) -> Self {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();

        let n = sorted.len();
        let sum: u64 = sorted.iter().sum();

        Stats {
            mean_ns: sum / n as u64,
            p50: sorted[n * 50 / 100],
            p99: sorted[(n * 99 / 100).min(n - 1)],
            p999: sorted[(n * 999 / 1000).min(n - 1)],
            min: sorted[0],
            max: sorted[n - 1],
        }
    }
}

fn fmt_ns(ns: u64) -> String {
    if ns < 1_000 {
        format!("{ns} ns")
    } else if ns < 1_000_000 {
        format!("{:.2} µs", ns as f64 / 1_000.0)
    } else {
        format!("{:.2} ms", ns as f64 / 1_000_000.0)
    }
}

fn fmt_ops(ops: u64) -> String {
    if ops >= 1_000_000 {
        format!("{:.1}M", ops as f64 / 1_000_000.0)
    } else if ops >= 1_000 {
        format!("{:.1}K", ops as f64 / 1_000.0)
    } else {
        format!("{ops}")
    }
}
