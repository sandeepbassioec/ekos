//! `vvctl bench-net` — micro-benchmark network round trips (VV-CTL-198).

use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use clap::Args;

use super::wire::{self, WireOpts};

#[derive(Args)]
pub struct BenchNetArgs {
    pub host: String,
    pub port: u16,
    #[arg(long, default_value = "100")]
    pub requests: u32,
    #[command(flatten)]
    pub conn: WireOpts,
}

/// Result of a run: per-request latencies and total wall time.
pub struct BenchNetReport {
    pub latencies: Vec<Duration>,
    pub total: Duration,
}

impl BenchNetReport {
    /// Latency at percentile `p` (0..=100), nearest-rank.
    pub fn percentile(&self, p: f64) -> Duration {
        let mut v = self.latencies.clone();
        v.sort();
        let rank = ((p / 100.0) * v.len() as f64).ceil() as usize;
        v[rank.saturating_sub(1).min(v.len() - 1)]
    }
}

pub fn bench(args: &BenchNetArgs) -> Result<BenchNetReport> {
    if args.requests == 0 {
        bail!("--requests must be at least 1");
    }
    let mut client = wire::open(&args.host, args.port, &args.conn)?;
    let mut latencies = Vec::with_capacity(args.requests as usize);
    let start = Instant::now();
    for i in 0..args.requests {
        let t = Instant::now();
        client
            .ping()
            .with_context(|| format!("request {i} failed"))?;
        latencies.push(t.elapsed());
    }
    Ok(BenchNetReport {
        latencies,
        total: start.elapsed(),
    })
}

pub fn run(args: BenchNetArgs) -> Result<()> {
    let r = bench(&args)?;
    let rps = r.latencies.len() as f64 / r.total.as_secs_f64().max(f64::EPSILON);
    println!(
        "bench-net: {} round trips to {}:{} in {:?} ({rps:.0} req/s)",
        r.latencies.len(),
        args.host,
        args.port,
        r.total
    );
    println!(
        "latency p50/p99/max = {:?}/{:?}/{:?}",
        r.percentile(50.0),
        r.percentile(99.0),
        r.percentile(100.0)
    );
    Ok(())
}
