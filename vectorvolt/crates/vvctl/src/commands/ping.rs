//! `vvctl ping` — send ping requests to a running VectorVolt server (VV-CTL-191).

use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct PingArgs {
    pub host: String,
    pub port: u16,
    #[arg(long, default_value = "1")]
    pub count: u32,
}

pub fn run(args: PingArgs) -> Result<()> {
    for i in 0..args.count {
        println!("ping {} {}:{} seq={}", i + 1, args.host, args.port, i);
    }
    Ok(())
}
