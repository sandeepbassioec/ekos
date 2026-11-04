//! `vvctl ping` — send ping requests to a running VectorVolt server (VV-CTL-191).

use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use clap::Args;

use super::wire::{self, WireOpts};

#[derive(Args)]
pub struct PingArgs {
    pub host: String,
    pub port: u16,
    #[arg(long, default_value = "1")]
    pub count: u32,
    #[command(flatten)]
    pub conn: WireOpts,
}

/// Round-trip latencies of `count` wire-protocol pings.
pub fn ping_rtts(args: &PingArgs) -> Result<Vec<Duration>> {
    let mut client = wire::open(&args.host, args.port, &args.conn)?;
    let mut rtts = Vec::with_capacity(args.count as usize);
    for seq in 0..args.count {
        let t = Instant::now();
        client
            .ping()
            .with_context(|| format!("ping seq={seq} to {}:{}", args.host, args.port))?;
        rtts.push(t.elapsed());
    }
    Ok(rtts)
}

pub fn run(args: PingArgs) -> Result<()> {
    if args.count == 0 {
        bail!("--count must be at least 1");
    }
    let rtts = ping_rtts(&args)?;
    let ms: Vec<f64> = rtts.iter().map(|d| d.as_secs_f64() * 1e3).collect();
    for (i, m) in ms.iter().enumerate() {
        println!(
            "pong from {}:{} seq={i} time={m:.3} ms",
            args.host, args.port
        );
    }
    let min = ms.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = ms.iter().cloned().fold(0.0, f64::max);
    let avg = ms.iter().sum::<f64>() / ms.len() as f64;
    println!(
        "{} sent, {} received; rtt min/avg/max = {min:.3}/{avg:.3}/{max:.3} ms",
        args.count,
        rtts.len()
    );
    Ok(())
}
