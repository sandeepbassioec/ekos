//! `vvctl status` — query server status over the wire protocol (VV-CTL-194).

use std::time::Instant;

use anyhow::{Context, Result};
use clap::Args;

use super::wire::{self, WireOpts};

#[derive(Args)]
pub struct StatusArgs {
    pub host: String,
    pub port: u16,
    #[command(flatten)]
    pub conn: WireOpts,
}

/// Live numbers reported by the server.
#[derive(Debug)]
pub struct ServerStatus {
    pub rtt_ms: f64,
    pub rows: u64,
    pub partitions: u64,
    pub bytes: u64,
}

pub fn query(args: &StatusArgs) -> Result<ServerStatus> {
    let mut client = wire::open(&args.host, args.port, &args.conn)?;
    let t = Instant::now();
    client.ping().context("ping")?;
    let rtt_ms = t.elapsed().as_secs_f64() * 1e3;
    let (rows, partitions, bytes) = client.stats().context("stats")?;
    Ok(ServerStatus {
        rtt_ms,
        rows,
        partitions,
        bytes,
    })
}

pub fn run(args: StatusArgs) -> Result<()> {
    let s = query(&args)?;
    println!("status     : ok ({}:{})", args.host, args.port);
    println!("rtt        : {:.3} ms", s.rtt_ms);
    println!("rows       : {}", s.rows);
    println!("partitions : {}", s.partitions);
    println!("bytes      : {}", s.bytes);
    Ok(())
}
