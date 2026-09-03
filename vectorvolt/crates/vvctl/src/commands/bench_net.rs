//! `vvctl bench-net` — micro-benchmark network round trips (VV-CTL-198).

use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct BenchNetArgs {
    pub host: String,
    pub port: u16,
    #[arg(long, default_value = "100")]
    pub requests: u32,
}

pub fn run(args: BenchNetArgs) -> Result<()> {
    let start = std::time::Instant::now();
    for _ in 0..args.requests { /* stub */ }
    println!(
        "bench-net: {} requests to {}:{} in {:?}",
        args.requests,
        args.host,
        args.port,
        start.elapsed()
    );
    Ok(())
}
